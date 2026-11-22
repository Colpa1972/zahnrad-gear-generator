//! Eingabefelder, Pruefung und Ausfuehrung der Eintraege.
use crate::build::{bore, cmd, make_solid, teeth, Solid};
use crate::geom::*;
use crate::host::{Counted, Error, Host};
use serde_json::{json, Map, Value};
use std::f64::consts::PI;

const D2R: f64 = PI / 180.0;
/// Hoechster Drall je Sweep (SolidCraft erlaubt unter 4 Umdrehungen)
const MAX_TURNS: f64 = 1.0;
/// Sicherheitsgrenze fuer die Gewindelaenge.
const MAX_WORM_TURNS: f64 = 40.0;

pub struct Field {
    pub key: &'static str,
    pub title: &'static str,
    pub unit: &'static str,
    pub def: Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub choices: &'static [(&'static str, &'static str)],
    pub help: &'static str,
}

fn f(key: &'static str, title: &'static str, unit: &'static str, def: f64, min: Option<f64>, max: Option<f64>, help: &'static str) -> Field {
    Field { key, title, unit, def: json!(def), min, max, choices: &[], help }
}
fn name_field() -> Field {
    Field { key: "name", title: "Name", unit: "", def: json!("Zahnrad"), min: None, max: None, choices: &[], help: "Name des Körpers; bei mehreren Rädern im selben Dokument verschieden wählen." }
}
fn hand_field() -> Field {
    Field { key: "hand", title: "Steigungsrichtung", unit: "", def: json!("rechts"), min: None, max: None, choices: &[("rechts", "rechts"), ("links", "links")], help: "" }
}
fn check(key: &'static str, title: &'static str, def: bool) -> Field {
    Field { key, title, unit: "", def: json!(def), min: None, max: None, choices: &[], help: "" }
}

pub struct Entry {
    pub name: &'static str,
    pub menu: &'static str,
    pub ribbon: &'static str,
    pub fields: Vec<Field>,
}

fn single_entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "spur",
            menu: "Zahnrad: Stirnrad (gerade, schräg, Pfeil)",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Normalmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Zähnezahl z", "", 20.0, Some(6.0), Some(300.0), ""),
                f("alpha", "Eingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), ""),
                f("beta", "Schrägungswinkel β", "°", 0.0, Some(0.0), Some(45.0), "0 = Geradverzahnung"),
                hand_field(),
                check("herringbone", "Pfeilverzahnung (Doppelschräg)", false),
                f("x", "Profilverschiebung x", "", 0.0, Some(-1.0), Some(1.5), ""),
                f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), ""),
                f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), ""),
                f("b", "Zahnbreite b", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("bore", "Bohrung Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "internal",
            menu: "Zahnrad: Hohlrad (innenverzahnt)",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Normalmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Zähnezahl z", "", 50.0, Some(20.0), Some(400.0), ""),
                f("alpha", "Eingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), ""),
                f("beta", "Schrägungswinkel β", "°", 0.0, Some(0.0), Some(45.0), "0 = Geradverzahnung"),
                hand_field(),
                check("herringbone", "Pfeilverzahnung (Doppelschräg)", false),
                f("x", "Profilverschiebung x", "", 0.0, Some(-1.0), Some(1.5), ""),
                f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), ""),
                f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), ""),
                f("b", "Zahnbreite b", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("dout", "Außendurchmesser", "mm", 0.0, Some(0.0), None, "0 = automatisch"),
            ],
        },
        Entry {
            name: "rack",
            menu: "Zahnrad: Zahnstange",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Modul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("n", "Zähne", "", 12.0, Some(1.0), Some(200.0), ""),
                f("alpha", "Eingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), ""),
                f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), ""),
                f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), ""),
                f("b", "Breite b", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("base", "Sockelhöhe", "mm", 4.0, Some(0.5), None, "unter dem Zahnfuß"),
            ],
        },
        Entry {
            name: "worm",
            menu: "Zahnrad: Schnecke",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Axialmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("N", "Gangzahl", "", 1.0, Some(1.0), Some(8.0), ""),
                f("q", "Durchmesserzahl q", "", 10.0, Some(5.0), Some(30.0), "Teilkreis-Ø = q · m"),
                f("alpha", "Axialeingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), ""),
                hand_field(),
                f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), ""),
                f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), ""),
                f("b", "Gewindelänge", "mm", 40.0, Some(1.0), Some(500.0), ""),
                f("bore", "Bohrung Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "wheel",
            menu: "Zahnrad: Schneckenrad",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Axialmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Zähnezahl z", "", 30.0, Some(12.0), Some(300.0), ""),
                f("N", "Z1 – Gangzahl der Schnecke", "", 1.0, Some(1.0), Some(8.0), ""),
                f("q", "Durchmesserzahl q der Schnecke", "", 10.0, Some(5.0), Some(30.0), "Schrägungswinkel = Steigungswinkel der Schnecke"),
                f("alpha", "Axialeingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), ""),
                hand_field(),
                f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), ""),
                f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), ""),
                f("b", "Radbreite b", "mm", 0.0, Some(0.0), Some(500.0), "0 = automatisch"),
                f("bore", "Bohrung Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
    ]
}


fn pair_entries() -> Vec<Entry> {
    let ha = || f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), "");
    let c = || f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), "");
    let alpha = || f("alpha", "Eingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), "");
    let beta = || f("beta", "Schrägungswinkel β", "°", 0.0, Some(0.0), Some(45.0), "0 = gerade; Gegenrad passend");
    vec![
        Entry {
            name: "pair-spur",
            menu: "Getriebe: Stirnradpaar Z1 + Z2",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Normalmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Z1 – Zähnezahl Ritzel", "", 20.0, Some(6.0), Some(300.0), ""),
                f("z2", "Z2 – Zähnezahl Rad", "", 40.0, Some(6.0), Some(300.0), ""),
                alpha(),
                beta(),
                hand_field(),
                check("herringbone", "Pfeilverzahnung (Doppelschräg)", false),
                ha(),
                c(),
                f("b", "Zahnbreite b", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("bore", "Bohrung Ritzel Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
                f("bore2", "Bohrung Rad Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "pair-internal",
            menu: "Getriebe: Ritzel Z1 + Hohlrad Z2",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Normalmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Z1 – Zähnezahl Ritzel", "", 20.0, Some(6.0), Some(300.0), ""),
                f("z2", "Z2 – Zähnezahl Hohlrad", "", 60.0, Some(20.0), Some(400.0), "≥ z1 + 10"),
                alpha(),
                beta(),
                hand_field(),
                check("herringbone", "Pfeilverzahnung (Doppelschräg)", false),
                ha(),
                c(),
                f("b", "Zahnbreite b", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("bore", "Bohrung Ritzel Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
                f("dout", "Außendurchmesser Hohlrad", "mm", 0.0, Some(0.0), None, "0 = automatisch"),
            ],
        },
        Entry {
            name: "pair-rack",
            menu: "Getriebe: Ritzel Z1 + Zahnstange",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Modul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Z1 – Zähnezahl Ritzel", "", 20.0, Some(6.0), Some(300.0), ""),
                f("n", "Zähne der Zahnstange", "", 0.0, Some(0.0), Some(200.0), "0 = automatisch"),
                alpha(),
                ha(),
                c(),
                f("b", "Zahnbreite b", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("base", "Sockelhöhe der Zahnstange", "mm", 4.0, Some(0.5), None, "unter dem Zahnfuß"),
                f("bore", "Bohrung Ritzel Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "pair-worm",
            menu: "Getriebe: Schnecke + Schneckenrad Z2",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Axialmodul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("N", "Z1 – Gangzahl der Schnecke", "", 1.0, Some(1.0), Some(8.0), ""),
                f("q", "Durchmesserzahl q der Schnecke", "", 10.0, Some(5.0), Some(30.0), "Teilkreis-Ø = q · m"),
                f("z", "Z2 – Zähnezahl Schneckenrad", "", 30.0, Some(12.0), Some(300.0), ""),
                f("alpha", "Axialeingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), ""),
                hand_field(),
                ha(),
                c(),
                f("b", "Gewindelänge der Schnecke", "mm", 40.0, Some(1.0), Some(500.0), ""),
                f("bw", "Breite Schneckenrad", "mm", 0.0, Some(0.0), Some(500.0), "0 = automatisch"),
                f("bore", "Bohrung Schnecke Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
                f("bore2", "Bohrung Schneckenrad Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
    ]
}

fn bevel_entries() -> Vec<Entry> {
    let ha = || f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), "");
    let c = || f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), "");
    let alpha = || f("alpha", "Eingriffswinkel α", "°", 20.0, Some(10.0), Some(30.0), "");
    vec![
        Entry {
            name: "bevel",
            menu: "Zahnrad: Kegelrad (geradverzahnt, 90°)",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Modul m (am großen Ende)", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Z1 – Zähnezahl dieses Rades", "", 20.0, Some(6.0), Some(300.0), ""),
                f("z2", "Z2 – Zähnezahl des Gegenrads", "", 40.0, Some(6.0), Some(300.0), "bestimmt den Teilkegelwinkel"),
                alpha(),
                ha(),
                c(),
                f("b", "Zahnbreite b (Mantellinie)", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("bore", "Bohrung Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "crown",
            menu: "Zahnrad: Kronenrad (Planrad)",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Modul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Zähnezahl z", "", 40.0, Some(6.0), Some(300.0), ""),
                alpha(),
                ha(),
                c(),
                f("b", "Zahnbreite b (radial)", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("base", "Scheibendicke", "mm", 6.0, Some(0.5), None, "unter dem Zahnfuß"),
                f("bore", "Bohrung Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "pair-bevel",
            menu: "Getriebe: Kegelradpaar Z1 + Z2 (90°)",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Modul m (am großen Ende)", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Z1 – Zähnezahl Ritzel", "", 20.0, Some(6.0), Some(300.0), ""),
                f("z2", "Z2 – Zähnezahl Rad", "", 40.0, Some(6.0), Some(300.0), ""),
                alpha(),
                ha(),
                c(),
                f("b", "Zahnbreite b (Mantellinie)", "mm", 10.0, Some(0.2), Some(500.0), ""),
                f("bore", "Bohrung Ritzel Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
                f("bore2", "Bohrung Rad Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
        Entry {
            name: "pair-crown",
            menu: "Getriebe: Kronenrad Z2 + Stirnritzel Z1",
            ribbon: "",
            fields: vec![
                name_field(),
                f("m", "Modul m", "mm", 2.0, Some(0.05), Some(50.0), ""),
                f("z", "Z1 – Zähnezahl Ritzel", "", 15.0, Some(6.0), Some(300.0), ""),
                f("z2", "Z2 – Zähnezahl Kronenrad", "", 45.0, Some(6.0), Some(300.0), ""),
                alpha(),
                ha(),
                c(),
                f("b", "Zahnbreite b (radial)", "mm", 12.0, Some(0.2), Some(500.0), ""),
                f("base", "Scheibendicke Kronenrad", "mm", 6.0, Some(0.5), None, "unter dem Zahnfuß"),
                f("bore", "Bohrung Ritzel Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
                f("bore2", "Bohrung Kronenrad Ø", "mm", 0.0, Some(0.0), None, "0 = keine Bohrung"),
            ],
        },
    ]
}

fn gearbox_entry() -> Entry {
    Entry {
        name: "gearbox",
        menu: "Getriebe: mehrstufig (Z1, Z2, Z3 …)",
        ribbon: "",
        fields: vec![
            name_field(),
            f("z", "Stufe 1 · Z1 Ritzel (Welle 1, Antrieb)", "", 20.0, Some(6.0), Some(300.0), ""),
            f("z2", "Stufe 1 · Z2 Rad (Welle 2)", "", 40.0, Some(6.0), Some(300.0), ""),
            f("m", "Stufe 1 · Modul M1 [mm]", "", 2.0, Some(0.05), Some(50.0), ""),
            f("w1", "Stufe 1 · Dicke Z1 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("w2", "Stufe 1 · Dicke Z2 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("beta", "Stufe 1 · Schrägung β1 [°] (0 = gerade)", "", 0.0, Some(0.0), Some(45.0), ""),
            f("z3", "Stufe 2 · Z3 Ritzel (Welle 2, mit Z2; 0 = Ende)", "", 0.0, Some(0.0), Some(300.0), ""),
            f("z4", "Stufe 2 · Z4 Rad (Welle 3)", "", 0.0, Some(0.0), Some(300.0), ""),
            f("m2", "Stufe 2 · Modul M2 [mm] (0 = wie M1)", "", 0.0, Some(0.0), Some(50.0), ""),
            f("w3", "Stufe 2 · Dicke Z3 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("w4", "Stufe 2 · Dicke Z4 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("beta2", "Stufe 2 · Schrägung β2 [°] (0 = wie β1, -1 = gerade)", "", 0.0, Some(-1.0), Some(45.0), ""),
            f("z5", "Stufe 3 · Z5 Ritzel (Welle 3, mit Z4; 0 = Ende)", "", 0.0, Some(0.0), Some(300.0), ""),
            f("z6", "Stufe 3 · Z6 Rad (Welle 4)", "", 0.0, Some(0.0), Some(300.0), ""),
            f("m3", "Stufe 3 · Modul M3 [mm] (0 = wie M1)", "", 0.0, Some(0.0), Some(50.0), ""),
            f("w5", "Stufe 3 · Dicke Z5 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("w6", "Stufe 3 · Dicke Z6 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("beta3", "Stufe 3 · Schrägung β3 [°] (0 = wie β1, -1 = gerade)", "", 0.0, Some(-1.0), Some(45.0), ""),
            f("z7", "Stufe 4 · Z7 Ritzel (Welle 4, mit Z6; 0 = Ende)", "", 0.0, Some(0.0), Some(300.0), ""),
            f("z8", "Stufe 4 · Z8 Rad (Welle 5)", "", 0.0, Some(0.0), Some(300.0), ""),
            f("m4", "Stufe 4 · Modul M4 [mm] (0 = wie M1)", "", 0.0, Some(0.0), Some(50.0), ""),
            f("w7", "Stufe 4 · Dicke Z7 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("w8", "Stufe 4 · Dicke Z8 [mm] (0 = Standard)", "", 0.0, Some(0.0), Some(500.0), ""),
            f("beta4", "Stufe 4 · Schrägung β4 [°] (0 = wie β1, -1 = gerade)", "", 0.0, Some(-1.0), Some(45.0), ""),
            f("ab", "Ab Stufe (bei mehr als 3 Stufen: Lauf wiederholen)", "", 1.0, Some(1.0), Some(4.0), ""),
            f("alpha", "Eingriffswinkel α [°]", "", 20.0, Some(10.0), Some(30.0), ""),
            hand_field(),
            check("herringbone", "Pfeilverzahnung (Doppelschräg)", false),
            f("ha", "Kopfhöhenfaktor h*a", "", 1.0, Some(0.5), Some(1.5), ""),
            f("c", "Kopfspiel c*", "", 0.25, Some(0.0), Some(0.6), ""),
            f("b", "Standarddicke b [mm]", "", 10.0, Some(0.2), Some(500.0), ""),
            f("bore", "Wellenbohrung Ø [mm] (0 = keine)", "", 0.0, Some(0.0), None, ""),
            f("gap", "Abstand der Ebenen [mm]", "", 2.0, Some(0.0), Some(100.0), ""),
        ],
    }
}

/// Typ-Auswahl des gemeinsamen Eintrags „Zahnrad“: Anzeigename -> Eintrag.
pub const TYPES: &[(&str, &str)] = &[
    ("Stirnrad", "spur"),
    ("Hohlrad (innenverzahnt)", "internal"),
    ("Zahnstange", "rack"),
    ("Schnecke", "worm"),
    ("Schneckenrad", "wheel"),
    ("Paar: Stirnrad + Stirnrad", "pair-spur"),
    ("Paar: Ritzel + Hohlrad", "pair-internal"),
    ("Paar: Ritzel + Zahnstange", "pair-rack"),
    ("Paar: Schnecke + Schneckenrad", "pair-worm"),
    ("Kegelrad (geradverzahnt, 90°)", "bevel"),
    ("Kronenrad (Planrad)", "crown"),
    ("Paar: Kegelrad + Kegelrad", "pair-bevel"),
    ("Paar: Kronenrad + Stirnritzel", "pair-crown"),
    ("Getriebe mehrstufig (Z1, Z2, Z3 …)", "gearbox"),
];

pub fn entries() -> Vec<Entry> {
    let mut all = single_entries();
    all.extend(pair_entries());
    all.extend(bevel_entries());
    all.push(gearbox_entry());
    for e in all.iter_mut() {
        let nm = match e.name {
            "spur" => "Stirnrad",
            "internal" => "Hohlrad",
            "rack" => "Zahnstange",
            "worm" => "Schnecke",
            "wheel" => "Schneckenrad",
            "gearbox" => "Getriebe",
            "bevel" => "Kegelrad",
            "crown" => "Kronenrad",
            "pair-bevel" => "Kegelradpaar",
            "pair-crown" => "Kronenradpaar",
            _ => "Paar",
        };
        e.ribbon = match e.name {
            "spur" => "Stirnrad",
            "internal" => "Hohlrad",
            "rack" => "Zahnstange",
            "worm" => "Schnecke",
            "wheel" => "Schneckenrad",
            "pair-spur" => "Paar Z1/Z2",
            "pair-internal" => "Hohlrad-Paar",
            "pair-rack" => "Zahnstangen-Paar",
            "pair-worm" => "Schnecken-Paar",
            "bevel" => "Kegelrad",
            "crown" => "Kronenrad",
            "pair-bevel" => "Kegelrad-Paar",
            "pair-crown" => "Kronenrad-Paar",
            _ => "Getriebe Z1…Zn",
        };
        if let Some(f) = e.fields.iter_mut().find(|f| f.key == "name") {
            f.def = json!(nm);
        }
    }
    // gemeinsamer Eintrag: Vereinigung aller Felder (weiteste Grenzen), Typ als Auswahlliste
    let mut fields = vec![Field { key: "type", title: "Art", unit: "", def: json!(TYPES[0].0), min: None, max: None, choices: TYPE_CHOICES, help: "Paar und Getriebe: Z1, Z2, Z3 … sind die Zähnezahlen in Bau-Reihenfolge (Ritzel, Rad, nächstes Ritzel …). Nicht benötigte Felder werden ignoriert." }];
    for e in &all {
        for fl in &e.fields {
            if let Some(ex) = fields.iter_mut().find(|x| x.key == fl.key) {
                ex.min = match (ex.min, fl.min) { (Some(a), Some(b)) => Some(a.min(b)), _ => None };
                ex.max = match (ex.max, fl.max) { (Some(a), Some(b)) => Some(a.max(b)), _ => None };
            } else {
                fields.push(Field { key: fl.key, title: fl.title, unit: fl.unit, def: fl.def.clone(), min: fl.min, max: fl.max, choices: fl.choices, help: fl.help });
            }
        }
    }
    // Reihenfolge und Beschriftung im Dialog (SolidCraft sortiert nach Schluesselname: siehe `schema`)
    const ORDER: &[(&str, &str)] = &[
        ("type", "Art (Paar/Getriebe: Z1, Z2, Z3 … in Bau-Reihenfolge)"),
        ("name", "Name"),
        ("m", "Modul m [mm] (Schnecke: Axialmodul)"),
        ("z", "Z1 – Zähnezahl (Einzelrad / Ritzel / Getriebe Stufe 1 Ritzel)"),
        ("z2", "Z2 – Zähnezahl Gegenrad (Paar / Getriebe Stufe 1 Rad / Kegelrad: bestimmt den Kegelwinkel)"),
        ("z3", "Z3 – Getriebe Stufe 2 Ritzel (0 = keine weitere Stufe)"),
        ("z4", "Z4 – Getriebe Stufe 2 Rad"),
        ("z5", "Z5 – Getriebe Stufe 3 Ritzel (0 = keine)"),
        ("z6", "Z6 – Getriebe Stufe 3 Rad"),
        ("z7", "Z7 – Getriebe Stufe 4 Ritzel (0 = keine)"),
        ("z8", "Z8 – Getriebe Stufe 4 Rad"),
        ("m2", "Getriebe Stufe 2 · Modul M2 (0 = wie M)"),
        ("m3", "Getriebe Stufe 3 · Modul M3 (0 = wie M)"),
        ("m4", "Getriebe Stufe 4 · Modul M4 (0 = wie M)"),
        ("w1", "Getriebe · Dicke Z1 [mm] (0 = wie b)"),
        ("w2", "Getriebe · Dicke Z2 [mm] (0 = wie b)"),
        ("w3", "Getriebe · Dicke Z3 [mm] (0 = wie b)"),
        ("w4", "Getriebe · Dicke Z4 [mm] (0 = wie b)"),
        ("w5", "Getriebe · Dicke Z5 [mm] (0 = wie b)"),
        ("w6", "Getriebe · Dicke Z6 [mm] (0 = wie b)"),
        ("w7", "Getriebe · Dicke Z7 [mm] (0 = wie b)"),
        ("w8", "Getriebe · Dicke Z8 [mm] (0 = wie b)"),
        ("alpha", "Eingriffswinkel α [°]"),
        ("beta", "Schrägungswinkel β [°] (0 = gerade; Getriebe: Stufe 1)"),
        ("beta2", "Getriebe Stufe 2 · β2 [°] (0 = wie β, -1 = gerade)"),
        ("beta3", "Getriebe Stufe 3 · β3 [°] (0 = wie β, -1 = gerade)"),
        ("beta4", "Getriebe Stufe 4 · β4 [°] (0 = wie β, -1 = gerade)"),
        ("hand", "Steigungsrichtung"),
        ("herringbone", "Pfeilverzahnung (Doppelschräg)"),
        ("x", "Profilverschiebung x (nur Einzelrad)"),
        ("ha", "Kopfhöhenfaktor h*a"),
        ("c", "Kopfspiel c*"),
        ("b", "Zahnbreite b [mm] (Schnecke: Länge; Kegelrad: Mantellinie; Kronenrad: radial)"),
        ("bore", "Bohrung Ø [mm] (Ritzel / Welle / Schnecke)"),
        ("bore2", "Bohrung Gegenrad Ø [mm]"),
        ("dout", "Hohlrad: Außen-Ø [mm] (0 = automatisch)"),
        ("n", "Zahnstange: Zähne (0 = automatisch)"),
        ("base", "Zahnstange: Sockelhöhe / Kronenrad: Scheibendicke [mm]"),
        ("N", "Schnecke: Gangzahl"),
        ("q", "Schnecke: Durchmesserzahl q"),
        ("bw", "Schneckenrad: Breite [mm] (0 = automatisch)"),
        ("ab", "Getriebe: ab Stufe (bei mehr als 3 Stufen: Lauf wiederholen)"),
        ("gap", "Getriebe: Abstand der Ebenen [mm]"),
    ];
    let mut ordered: Vec<Field> = Vec::new();
    for (k, t) in ORDER {
        if let Some(pos) = fields.iter().position(|x| x.key == *k) {
            let mut fl = fields.remove(pos);
            fl.title = t;
            fl.unit = "";
            ordered.push(fl);
        }
    }
    ordered.extend(fields);
    let mut fields = ordered;
    fields.retain(|x| x.key != "hinweis");
    // gemeinsame Vorgaben fuer Felder, deren Einzelvorgabe je Art verschieden ist
    for fl in fields.iter_mut() {
        match fl.key {
            "b" => fl.def = json!(20.0),
            "z" => fl.def = json!(20.0),
            "name" => fl.def = json!("Zahnrad"),
            "n" => fl.def = json!(0.0),
            "title" => {}
            _ => {}
        }
    }
    all.insert(0, Entry { name: "gear", menu: "Zahnrad-Generator", ribbon: "Zahnrad", fields });
    all
}

const TYPE_CHOICES: &[(&str, &str)] = &[
    ("Stirnrad", "Stirnrad"),
    ("Hohlrad (innenverzahnt)", "Hohlrad (innenverzahnt)"),
    ("Zahnstange", "Zahnstange"),
    ("Schnecke", "Schnecke"),
    ("Schneckenrad", "Schneckenrad"),
    ("Paar: Stirnrad + Stirnrad", "Paar: Stirnrad + Stirnrad"),
    ("Paar: Ritzel + Hohlrad", "Paar: Ritzel + Hohlrad"),
    ("Paar: Ritzel + Zahnstange", "Paar: Ritzel + Zahnstange"),
    ("Paar: Schnecke + Schneckenrad", "Paar: Schnecke + Schneckenrad"),
    ("Kegelrad (geradverzahnt, 90°)", "Kegelrad (geradverzahnt, 90°)"),
    ("Kronenrad (Planrad)", "Kronenrad (Planrad)"),
    ("Paar: Kegelrad + Kegelrad", "Paar: Kegelrad + Kegelrad"),
    ("Paar: Kronenrad + Stirnritzel", "Paar: Kronenrad + Stirnritzel"),
    ("Getriebe mehrstufig (Z1, Z2, Z3 …)", "Getriebe mehrstufig (Z1, Z2, Z3 …)"),
];

/// JSON-Schema eines Eintrags (daraus baut SolidCraft den Dialog).
pub fn schema(e: &Entry) -> Value {
    let mut props = Map::new();
    // Gemeinsamer Eintrag: je Feld die Arten, in denen es vorkommt (-> Sichtbarkeit) und die Vorgabe je Art.
    let subs: Vec<Entry> = if e.name == "gear" { entries().into_iter().filter(|x| x.name != "gear").collect() } else { Vec::new() };
    let tkey = e.fields.iter().position(|x| x.key == "type").map(|i| format!("{:02}_type", i + 1));
    for (i, fl) in e.fields.iter().enumerate() {
        let mut p = Map::new();
        let mut title = if fl.unit.is_empty() { fl.title.to_string() } else { format!("{} [{}]", fl.title, fl.unit) };
        if fl.help.starts_with("0 =") && !title.contains("0 =") {
            title = format!("{title} ({})", fl.help);
        }
        p.insert("title".into(), json!(title));
        // Beschreibungen sind im Manifest teuer (Obergrenze 16 KiB): Hinweise stehen in den Titeln.
        match &fl.def {
            Value::String(_) if !fl.choices.is_empty() => {
                p.insert("type".into(), json!("string"));
                p.insert("enum".into(), json!(fl.choices.iter().map(|c| c.0).collect::<Vec<_>>()));
            }
            Value::String(_) => {
                p.insert("type".into(), json!("string"));
                p.insert("maxLength".into(), json!(200));
            }
            Value::Bool(_) => {
                p.insert("type".into(), json!("boolean"));
            }
            _ => {
                p.insert("type".into(), json!("number"));
                if let Some(mn) = fl.min {
                    p.insert("minimum".into(), compact(&json!(mn)));
                }
                if let Some(mx) = fl.max {
                    p.insert("maximum".into(), compact(&json!(mx)));
                }
            }
        }
        p.insert("default".into(), compact(&fl.def));
        if let (Some(tk), true) = (&tkey, fl.key != "type") {
            let mut present: Vec<(&str, &Value)> = Vec::new();
            for (disp, target) in TYPES {
                if let Some(sf) = subs.iter().find(|x| x.name == *target).and_then(|x| x.fields.iter().find(|y| y.key == fl.key)) {
                    present.push((disp, &sf.def));
                }
            }
            if present.len() < TYPES.len() && !present.is_empty() {
                let names: Vec<&str> = present.iter().map(|x| x.0).collect();
                p.insert("x-solidcraft-visible-when".into(), json!({ "field": tk, "in": names }));
            }
            {
                let differs = present.iter().any(|x| x.1 != present[0].1);
                if differs && !matches!(fl.def, Value::Bool(_)) {
                    let rules: Vec<Value> = present.iter().map(|(d, v)| json!({ "when": { "field": tk, "eq": d }, "value": compact(v) })).collect();
                    p.insert("x-solidcraft-defaults".into(), Value::Array(rules));
                }
            }
        }
        props.insert(format!("{:02}_{}", i + 1, fl.key), Value::Object(p));
    }
    json!({ "type": "object", "properties": props, "additionalProperties": false })
}

/// Eingabewerte mit Vorgaben und Bereichspruefung.
pub struct Params<'a> {
    obj: Map<String, Value>,
    entry: &'a Entry,
}

impl<'a> Params<'a> {
    pub fn parse(entry: &'a Entry, text: &str) -> Result<Self, Error> {
        let v: Value = if text.trim().is_empty() { json!({}) } else { serde_json::from_str(text).map_err(|e| Error::InvalidInput(format!("Parameter sind kein gültiges JSON: {e}")))? };
        let obj = match v {
            Value::Object(o) => o,
            _ => return Err(Error::InvalidInput("Parameter müssen ein Objekt sein".into())),
        };
        let obj: Map<String, Value> = obj
            .into_iter()
            .map(|(k, v)| {
                let b = k.as_bytes();
                if b.len() > 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() && b[2] == b'_' { (k[3..].to_string(), v) } else { (k, v) }
            })
            .collect();
        for k in obj.keys() {
            if k != "phase" && !entry.fields.iter().any(|fl| fl.key == k) {
                return Err(Error::InvalidInput(format!("Unbekannter Parameter „{k}“")));
            }
        }
        Ok(Params { obj, entry })
    }
    fn field(&self, key: &str) -> &Field {
        self.entry.fields.iter().find(|fl| fl.key == key).unwrap_or_else(|| panic!("Feld {key} fehlt in {}", self.entry.name))
    }
    pub fn num(&self, key: &str) -> Result<f64, Error> {
        let fl = self.field(key);
        let v = match self.obj.get(key) {
            None | Some(Value::Null) => fl.def.as_f64().unwrap(),
            Some(Value::Number(n)) => n.as_f64().unwrap(),
            Some(Value::String(s)) => s.trim().replace(',', ".").parse::<f64>().map_err(|_| Error::InvalidInput(format!("„{}“ ist keine Zahl", fl.title)))?,
            Some(_) => return Err(Error::InvalidInput(format!("„{}“ ist keine Zahl", fl.title))),
        };
        if !v.is_finite() {
            return Err(Error::InvalidInput(format!("„{}“ ist keine endliche Zahl", fl.title)));
        }
        if let Some(mn) = fl.min {
            if v < mn - 1e-12 {
                return Err(Error::InvalidInput(format!("„{}“ muss mindestens {mn} sein", fl.title)));
            }
        }
        if let Some(mx) = fl.max {
            if v > mx + 1e-12 {
                return Err(Error::InvalidInput(format!("„{}“ darf höchstens {mx} sein", fl.title)));
            }
        }
        Ok(v)
    }
    pub fn int(&self, key: &str) -> Result<usize, Error> {
        Ok(self.num(key)?.round() as usize)
    }
    pub fn flag(&self, key: &str) -> bool {
        match self.obj.get(key) {
            Some(Value::Bool(b)) => *b,
            _ => self.field(key).def.as_bool().unwrap_or(false),
        }
    }
    pub fn text(&self, key: &str) -> String {
        match self.obj.get(key) {
            Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
            _ => self.field(key).def.as_str().unwrap_or("").to_string(),
        }
    }
    /// Interner Zusatzwert (z. B. Phasenlage fuer Paarungen), nicht Teil des Dialogs.
    pub fn extra(&self, key: &str) -> f64 {
        self.obj.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
    }
    pub fn sign(&self) -> f64 {
        if matches!(self.text("hand").as_str(), "links" | "L") { -1.0 } else { 1.0 }
    }
}

fn bad(s: impl Into<String>) -> Error {
    Error::InvalidInput(s.into())
}

pub fn run(h: &mut dyn Host, entry: &str, params: &str) -> Result<String, Error> {
    let all = entries();
    let e = all.iter().find(|e| e.name == entry).ok_or_else(|| Error::Unsupported(format!("Unbekannter Eintrag „{entry}“")))?;
    let p = Params::parse(e, params)?;
    if entry == "gear" {
        let ty = p.text("type");
        let target = TYPES.iter().find(|t| t.0 == ty).map(|t| t.1).ok_or_else(|| bad(format!("Unbekannte Art „{ty}“")))?;
        let sub = all.iter().find(|e| e.name == target).unwrap();
        let mut obj = Map::new();
        for fl in &sub.fields {
            if let Some(v) = p.obj.get(fl.key) {
                obj.insert(fl.key.to_string(), v.clone());
            }
        }
        return run(h, target, &Value::Object(obj).to_string());
    }
    let summary = match entry {
        "spur" => spur(h, &p)?,
        "internal" => internal(h, &p)?,
        "rack" => rack(h, &p)?,
        "worm" => worm(h, &p)?,
        "wheel" => wheel(h, &p)?,
        "pair-spur" => pair_spur(h, &p)?,
        "pair-internal" => pair_internal(h, &p)?,
        "pair-rack" => pair_rack(h, &p)?,
        "pair-worm" => pair_worm(h, &p)?,
        "bevel" => bevel(h, &p)?,
        "crown" => crown(h, &p)?,
        "pair-bevel" => pair_bevel(h, &p)?,
        "pair-crown" => pair_crown(h, &p)?,
        "gearbox" => gearbox(h, &p)?,
        _ => unreachable!(),
    };
    Ok(summary.to_string())
}

/// Einzelrad mit zusammengesetzten Eingaben bauen (fuer Paarungen).
fn single(h: &mut dyn Host, entry: &str, obj: Value) -> Result<Value, Error> {
    let all = entries();
    let e = all.iter().find(|e| e.name == entry).unwrap();
    let p = Params::parse(e, &obj.to_string())?;
    match entry {
        "spur" => spur(h, &p),
        "internal" => internal(h, &p),
        "rack" => rack(h, &p),
        "worm" => worm(h, &p),
        "wheel" => wheel(h, &p),
        "bevel" => bevel(h, &p),
        "crown" => crown(h, &p),
        _ => unreachable!(),
    }
}

fn other_hand(p: &Params) -> &'static str {
    if p.sign() < 0.0 { "rechts" } else { "links" }
}

fn mv(h: &mut dyn Host, body: &str, translate: [f64; 3]) -> Result<(), Error> {
    cmd(h, "solid.move", json!({ "bodies": [body], "translate": translate, "name": format!("{body} platzieren") }))?;
    Ok(())
}

/// Stirnradpaar: Ritzel im Ursprung, Rad auf der X-Achse; Rad gegenlaeufig geneigt, Luecke zeigt zum Ritzel.
fn pair_spur(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z1, z2) = (p.num("m")?, p.int("z")?, p.int("z2")?);
    let beta = p.num("beta")?;
    let a = m / (beta * D2R).cos() * (z1 + z2) as f64 / 2.0;
    let name = format!("{} (i={})", p.text("name"), fmt_i(z2 as f64 / z1 as f64));
    let (n1, n2) = (format!("{name} Ritzel"), format!("{name} Rad"));
    let common = |nm: &str, z: usize, hand: &str, bore: f64, phase: f64| -> Result<Value, Error> {
        Ok(json!({ "name": nm, "m": m, "z": z, "alpha": p.num("alpha")?, "beta": beta, "hand": hand, "herringbone": p.flag("herringbone"),
            "ha": p.num("ha")?, "c": p.num("c")?, "b": p.num("b")?, "bore": bore, "phase": phase }))
    };
    single(h, "spur", common(&n1, z1, &p.text("hand"), p.num("bore")?, 0.0)?)?;
    single(h, "spur", common(&n2, z2, other_hand(p), p.num("bore2")?, PI - PI / z2 as f64)?)?;
    mv(h, &n2, [a, 0.0, 0.0])?;
    Ok(json!({ "a": a, "i": z2 as f64 / z1 as f64 }))
}

/// Ritzel + Hohlrad: Hohlrad mit gleicher Steigung, Mittelpunkt bei -a.
fn pair_internal(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z1, z2) = (p.num("m")?, p.int("z")?, p.int("z2")?);
    if z2 < z1 + 10 {
        return Err(bad("Hohlrad braucht mindestens 10 Zähne mehr als das Ritzel."));
    }
    let beta = p.num("beta")?;
    let a = m / (beta * D2R).cos() * (z2 - z1) as f64 / 2.0;
    let name = format!("{} (i={})", p.text("name"), fmt_i(z2 as f64 / z1 as f64));
    let (n1, n2) = (format!("{name} Ritzel"), format!("{name} Hohlrad"));
    let hand = p.text("hand");
    let base = |nm: &str, z: usize| -> Result<serde_json::Map<String, Value>, Error> {
        let v = json!({ "name": nm, "m": m, "z": z, "alpha": p.num("alpha")?, "beta": beta, "hand": hand, "herringbone": p.flag("herringbone"),
            "ha": p.num("ha")?, "c": p.num("c")?, "b": p.num("b")? });
        Ok(v.as_object().unwrap().clone())
    };
    let mut o1 = base(&n1, z1)?;
    o1.insert("bore".into(), json!(p.num("bore")?));
    single(h, "spur", Value::Object(o1))?;
    let mut o2 = base(&n2, z2)?;
    o2.insert("dout".into(), json!(p.num("dout")?));
    // Der Hohlraum hat seine Zahnluecke bei 0 Grad; das Ritzel-Zahn muss in eine Luecke: halbe Teilung weiterdrehen.
    o2.insert("phase".into(), json!(PI / z2 as f64));
    single(h, "internal", Value::Object(o2))?;
    mv(h, &n2, [-a, 0.0, 0.0])?;
    Ok(json!({ "a": a, "i": z2 as f64 / z1 as f64 }))
}

/// Ritzel + Zahnstange: Zahnstange (Zaehne nach oben) unter dem Ritzel, Teilungslinie tangiert den Teilkreis.
fn pair_rack(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z) = (p.num("m")?, p.int("z")?);
    let n = match p.int("n")? {
        0 => (z / 2 + 4).max(6),
        k => k,
    };
    let (r, pt) = (m * z as f64 / 2.0, PI * m);
    let name = p.text("name");
    let (n1, n2) = (format!("{name} Ritzel"), format!("{name} Zahnstange"));
    single(h, "spur", json!({ "name": n1, "m": m, "z": z, "alpha": p.num("alpha")?, "beta": 0.0, "ha": p.num("ha")?, "c": p.num("c")?,
        "b": p.num("b")?, "bore": p.num("bore")?, "phase": -PI / 2.0 }))?;
    single(h, "rack", json!({ "name": n2, "m": m, "n": n, "alpha": p.num("alpha")?, "ha": p.num("ha")?, "c": p.num("c")?, "b": p.num("b")?, "base": p.num("base")? }))?;
    let sx = -((n / 2) as f64) * pt;
    mv(h, &n2, [sx, -r, 0.0])?;
    Ok(json!({ "r": r, "n": n, "L": n as f64 * pt }))
}

/// Schneckengetriebe: Schnecke auf der Z-Achse, Rad (Achse parallel Y) im Abstand a auf der +X-Seite.
fn pair_worm(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, nn, q, z) = (p.num("m")?, p.int("N")?, p.num("q")?, p.int("z")?);
    let (ha, b) = (p.num("ha")?, p.num("b")?);
    let sg = p.sign();
    let gam = (nn as f64 / q).atan();
    let r = m * z as f64 / 2.0;
    let w = if p.num("bw")? > 0.0 { p.num("bw")? } else { 0.75 * (q * m + 2.0 * ha * m) };
    let a = (q + z as f64) * m / 2.0;
    let pax = PI * m;
    let name = format!("{} (i={})", p.text("name"), fmt_i(z as f64 / nn as f64));
    let (n1, n2) = (format!("{name} Schnecke"), format!("{name} Rad"));
    single(h, "worm", json!({ "name": n1, "m": m, "N": nn, "q": q, "alpha": p.num("alpha")?, "hand": p.text("hand"), "ha": ha, "c": p.num("c")?, "b": b, "bore": p.num("bore")? }))?;
    // Phasenlage: in der Mittelebene des Rades muss eine Luecke zur Schnecke zeigen
    let tw_mid = sg * (w / 2.0) * gam.tan() / r;
    let phase = PI - PI / z as f64 - tw_mid;
    single(h, "wheel", json!({ "name": n2, "m": m, "z": z, "N": nn, "q": q, "alpha": p.num("alpha")?, "hand": p.text("hand"), "ha": ha, "c": p.num("c")?,
        "b": w, "bore": p.num("bore2")?, "phase": phase }))?;
    let z0 = ((b / 2.0) / pax).round() * pax;
    // Drehung +90° um X (z -> -y, y -> z) und Verschiebung; als Matrix, damit die Drehrichtung eindeutig ist
    cmd(h, "solid.move", json!({ "bodies": [n2], "matrix": [[1.0, 0.0, 0.0, a], [0.0, 0.0, -1.0, w / 2.0], [0.0, 1.0, 0.0, z0]], "name": format!("{n2} platzieren") }))?;
    Ok(json!({ "a": a, "i": z as f64 / nn as f64, "gamma": gam.to_degrees(), "b_rad": w }))
}

// ---------------------------------------------------------------- Kegelrad, Kronenrad

/// Geometrie eines geradverzahnten Kegelrads (Achswinkel 90°, Naeherung nach Tredgold):
/// Das Zahnprofil des Ergaenzungs-Stirnrads (Ruecken­kegel) wird in der Ebene senkrecht zur Teilkegel-
/// Mantellinie am Teilkreispunkt gezeichnet, zur Spitze hin verkleinert (Loft) und rundum vervielfacht.
pub struct BevelGeo {
    pub r: f64,     // Teilkreisradius am grossen Ende
    pub delta: f64, // Teilkegelwinkel
    pub apex: f64,  // Abstand Spitze - Ebene des grossen Endes (Achse)
    pub re: f64,    // aeussere Teilkegellaenge
    pub q: P2,      // Fussbeginn am grossen Ende (Radius, z) auf der Ruecken­kegel-Mantellinie
    pub da: f64,    // Kopfkreisdurchmesser am grossen Ende
    pub tooth: Outline, // Zahn in Skizzenkoordinaten (tangential, entlang der Rueckenkegel-Mantellinie)
    pub rin: f64,   // Radius des innersten Zahnpunkts am grossen Ende
}

pub fn bevel_geo(m: f64, z: usize, z2: usize, al: f64, ha: f64, c: f64) -> Result<BevelGeo, Error> {
    let delta = (z as f64).atan2(z2 as f64);
    let (cd, sd) = (delta.cos(), delta.sin());
    let r = m * z as f64 / 2.0;
    let zv = z as f64 / cd;
    let rv = r / cd;
    let rb = rv * al.cos();
    let st = m * PI / 2.0;
    let half = move |rho: f64| st / (2.0 * rv) + inv(al) - inv((rb / rho).min(1.0).acos());
    let ra_v = limit_tip(&half, rb, rv + ha * m, 0.2 * m);
    let rf_v = rv - (ha + c) * m;
    if rf_v <= 0.0 {
        return Err(bad("Fußkreis ≤ 0: Zähnezahl zu klein oder Zahnhöhe zu groß."));
    }
    let rho0 = rf_v.max(rb);
    let rho = involute_radii(rb, rho0, ra_v, 10);
    let rin_v = (rf_v - 0.5 * m).max(0.5 * rf_v);
    let nv = zv.round().max(3.0) as usize;
    let o = external_tooth(nv, rf_v, rin_v, &rho, &half, 0.0).map_err(bad)?;
    // Skizzenkoordinaten: x = tangential, y = Abstand vom Teilkreispunkt entlang der Rueckenkegel-Mantellinie
    let tooth = o.mapped(&move |p: P2| [p[1], p[0] - rv]);
    let apex = r / delta.tan();
    // Mantellinie des Rueckenkegels (senkrecht zur Teilkegel-Mantellinie): (r + u*cd, +u*sd), u < 0 zum Fuss hin
    let uq = rf_v - rv;
    let ui = rin_v - rv;
    Ok(BevelGeo {
        r,
        delta,
        apex,
        re: r / sd,
        q: [r + uq * cd, uq * sd],
        da: 2.0 * (r + (ra_v - rv) * cd),
        tooth,
        rin: r + ui * cd,
    })
}

/// Zeichnet einen Umriss in eine beliebig im Raum liegende Skizzenebene.
fn draw_profile_on(h: &mut dyn Host, sketch: &str, plane: Value, o: &Outline) -> Result<(), Error> {
    cmd(h, "sketch.create", json!({ "plane": plane, "name": sketch }))?;
    for it in &o.items {
        match it {
            Item::Spline(pts) => cmd(h, "sketch.spline.fit_point", json!({ "points": pts }))?,
            Item::Line(pts) => cmd(h, "sketch.line", json!({ "points": pts }))?,
            Item::Arc { start, mid, end } => cmd(h, "sketch.arc.three_point", json!({ "start": start, "end": end, "through": mid }))?,
        };
    }
    cmd(h, "sketch.finish", json!({}))?;
    Ok(())
}

fn build_bevel(h: &mut dyn Host, name: &str, m: f64, z: usize, z2: usize, al: f64, ha: f64, c: f64, b: f64, bore_d: f64, phase: f64) -> Result<Value, Error> {
    let g = bevel_geo(m, z, z2, al, ha, c)?;
    if b >= 0.6 * g.re {
        return Err(bad(format!("Zahnbreite zu groß: höchstens 0,6 · Kegellänge ({:.1} mm); üblich ist ein Drittel.", 0.6 * g.re)));
    }
    let k = (g.re - b) / g.re;
    let (cd, sd) = (g.delta.cos(), g.delta.sin());
    let hh = g.apex * (1.0 - k); // Hoehe der Teilkreisebene des kleinen Endes
    // Rohling: Kegelstumpf auf dem Fusskegel zwischen zwei Ebenen senkrecht zur Achse; er umfasst den
    // Zahnfuss an beiden Enden (Skizzen-y der Zaehne zeigt entlang des Rueckenkegels, daher z = u * sin(delta)).
    let z_in = ((g.rin - g.r) / cd) * sd; // z des innersten Zahnpunkts am grossen Ende (negativ)
    let z_lo = z_in - 1.0;
    let z_hi = hh + (1.0f64).min(0.5 * (g.apex - hh));
    let root_r = |zz: f64| g.q[0] * (g.apex - zz) / (g.apex - g.q[1]);
    if bore_d / 2.0 >= root_r(z_hi) - 0.3 * m {
        return Err(bad("Bohrung zu groß für das kleine Ende des Rades."));
    }
    cmd(h, "solid.cone", json!({ "radius": root_r(z_lo), "top_radius": root_r(z_hi), "height": z_hi - z_lo, "base": [0.0, 0.0, z_lo], "axis": "Z", "operation": "new", "name": format!("{name} Kegel"), "body_name": name }))?;
    let (zb, zf) = (z_lo, z_hi);
    let (cp, sp) = (phase.cos(), phase.sin());
    let frame = |s: f64| {
        json!({ "origin": [s * g.r * cp, s * g.r * sp, g.apex * (1.0 - s)], "x_dir": [-sp, cp, 0.0], "y_dir": [cd * cp, cd * sp, sd] })
    };
    let t1 = g.tooth.mapped(&move |p: P2| [p[0] * k, p[1] * k]);
    let (s0, s1) = (format!("{name} Zahn gross"), format!("{name} Zahn klein"));
    draw_profile_on(h, &s0, frame(1.0), &g.tooth)?;
    draw_profile_on(h, &s1, frame(k), &t1)?;
    let tn = format!("{name} Zahn");
    cmd(h, "solid.loft", json!({ "sections": [{ "sketch": s0 }, { "sketch": s1 }], "operation": "join", "targets": [name], "name": tn }))?;
    if z > 1 {
        cmd(h, "solid.pattern.circular", json!({ "features": [tn], "axis": "Z", "count": z, "name": format!("{name} Zahnkranz") }))?;
    }
    if bore_d > 0.0 {
        cmd(h, "solid.cylinder", json!({ "diameter": bore_d, "height": zf - zb + 2.0, "base": [0.0, 0.0, zb - 1.0], "operation": "cut", "targets": [name], "name": format!("{name} Bohrung") }))?;
    }
    Ok(json!({ "d": 2.0 * g.r, "da": g.da, "delta": g.delta.to_degrees(), "kegellaenge": g.re, "spitze_z": g.apex, "z_von": zb, "z_bis": zf }))
}

/// Kegelrad einzeln: grosses Ende in der Ebene z = 0, Spitze auf der Z-Achse bei z = apex.
fn bevel(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    build_bevel(h, &p.text("name"), p.num("m")?, p.int("z")?, p.int("z2")?, p.num("alpha")? * D2R, p.num("ha")?, p.num("c")?, p.num("b")?, p.num("bore")?, p.extra("phase"))
}

/// Kegelradpaar 90°: Ritzel auf der Z-Achse, Rad mit Achse in X-Richtung; beide Spitzen fallen zusammen.
fn pair_bevel(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z1, z2) = (p.num("m")?, p.int("z")?, p.int("z2")?);
    let (al, ha, c, b) = (p.num("alpha")? * D2R, p.num("ha")?, p.num("c")?, p.num("b")?);
    let (r1, r2) = (m * z1 as f64 / 2.0, m * z2 as f64 / 2.0);
    let name = format!("{} (i={})", p.text("name"), fmt_i(z2 as f64 / z1 as f64));
    let (n1, n2) = (format!("{name} Ritzel"), format!("{name} Rad"));
    let a = build_bevel(h, &n1, m, z1, z2, al, ha, c, b, p.num("bore")?, 0.0)?;
    build_bevel(h, &n2, m, z2, z1, al, ha, c, b, p.num("bore2")?, PI - PI / z2 as f64)?;
    // Rad: lokales z -> -X, x -> +Z, y -> +Y; grosses Ende bei (r1, 0, r2), Spitze bei (0, 0, r2)
    cmd(h, "solid.move", json!({ "bodies": [n2], "matrix": [[0.0, 0.0, -1.0, r1], [0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, r2]], "name": format!("{n2} platzieren") }))?;
    Ok(json!({ "i": z2 as f64 / z1 as f64, "spitze": [0.0, 0.0, r2], "delta1": a["delta"] }))
}

/// Zahnquerschnitt eines Kronenrads in der Ebene x = rho (Koordinaten y, z).
fn crown_section(m: f64, zc: usize, al: f64, ha: f64, c: f64, base: f64, rho: f64) -> Result<Vec<P2>, Error> {
    // Zahndicke in der Teilebene: Teilung 2*pi*rho/z minus die (konstante) Zahndicke des Ritzels pi*m/2
    let rr = m * zc as f64 / 2.0;
    // Das Ritzel hat nur in der Mitte (rho = rr) das passende Profil; zu den Enden der Zahnbreite hin
    // wird die Luecke etwas weiter (Flankenspiel je Seite).
    let half_p = PI * rho / zc as f64 - PI * m / 4.0 - 0.03 * m - 0.06 * (rho - rr).abs();
    let (zr, zp) = (base, base + (ha + c) * m);
    // Zahnkopf etwas niedriger als das Ritzel-Fussspiel verlangt: die Ritzelzaehne beruehren die Kronenradspitzen sonst am Rand
    let hk = 0.8 * ha * m;
    let zt = zp + hk;
    let tr = half_p + (ha + c) * m * al.tan();
    let tt = half_p - hk * al.tan();
    if tt < 0.04 * m {
        return Err(bad("Kronenrad: Zahn am Kopf zu spitz (Breite zu groß oder Zähnezahl zu klein)."));
    }
    if 2.0 * tr >= 2.0 * PI * rho / zc as f64 - 0.05 * m || half_p <= 0.0 {
        return Err(bad("Kronenrad: Zähne am Fuß zu breit für die Teilung (Zahnbreite verkleinern)."));
    }
    Ok(vec![[-tr, zr - 1.0], [tr, zr - 1.0], [tr, zr], [tt, zt], [-tt, zt], [-tr, zr], [-tr, zr - 1.0]])
}

fn build_crown(h: &mut dyn Host, name: &str, m: f64, zc: usize, al: f64, ha: f64, c: f64, b: f64, base: f64, bore_d: f64) -> Result<Value, Error> {
    let rr = m * zc as f64 / 2.0;
    if b > 0.6 * rr {
        return Err(bad(format!("Zahnbreite zu groß: höchstens 0,6 · Teilkreisradius ({:.1} mm).", 0.6 * rr)));
    }
    let (ri, ro) = (rr - b / 2.0, rr + b / 2.0);
    if bore_d / 2.0 >= ri - 0.5 * m {
        return Err(bad("Bohrung zu groß für den Zahnkranz."));
    }
    let si = crown_section(m, zc, al, ha, c, base, ri)?;
    let so = crown_section(m, zc, al, ha, c, base, ro)?;
    let rd = ro.hypot(so[1][0]) + 0.3;
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * rd, "height": base, "base": [0.0, 0.0, 0.0], "operation": "new", "name": format!("{name} Scheibe"), "body_name": name }))?;
    let (s0, s1) = (format!("{name} Zahn innen"), format!("{name} Zahn aussen"));
    for (nm, rho, pts) in [(&s0, ri, &si), (&s1, ro, &so)] {
        cmd(h, "sketch.create", json!({ "plane": "YZ", "offset": rho, "name": nm }))?;
        cmd(h, "sketch.line", json!({ "points": pts }))?;
        cmd(h, "sketch.finish", json!({}))?;
    }
    let tn = format!("{name} Zahn");
    cmd(h, "solid.loft", json!({ "sections": [{ "sketch": s0 }, { "sketch": s1 }], "operation": "join", "targets": [name], "name": tn }))?;
    cmd(h, "solid.pattern.circular", json!({ "features": [tn], "axis": "Z", "count": zc, "name": format!("{name} Zahnkranz") }))?;
    bore(h, name, bore_d, 0.0, base)?;
    Ok(json!({ "d": 2.0 * rr, "ri": ri, "ro": ro, "z_teilebene": base + (ha + c) * m, "z_kopf": base + (1.8 * ha + c) * m }))
}

/// Kronenrad (Planrad): Scheibe in z = 0 .. base, Zaehne nach oben, ein Zahn bei Winkel 0.
fn crown(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    build_crown(h, &p.text("name"), p.num("m")?, p.int("z")?, p.num("alpha")? * D2R, p.num("ha")?, p.num("c")?, p.num("b")?, p.num("base")?, p.num("bore")?)
}

/// Kronenrad + Stirnritzel: Ritzelachse liegt radial (X-Richtung) ueber dem Zahnkranz und schneidet die Radachse.
fn pair_crown(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z1, z2) = (p.num("m")?, p.int("z")?, p.int("z2")?);
    let (al, ha, c, b, base) = (p.num("alpha")?, p.num("ha")?, p.num("c")?, p.num("b")?, p.num("base")?);
    let name = format!("{} (i={})", p.text("name"), fmt_i(z2 as f64 / z1 as f64));
    let (n1, n2) = (format!("{name} Ritzel"), format!("{name} Kronenrad"));
    let info = build_crown(h, &n2, m, z2, al * D2R, ha, c, b, base, p.num("bore2")?)?;
    single(h, "spur", json!({ "name": n1, "m": m, "z": z1, "alpha": al, "beta": 0.0, "ha": ha, "c": c, "b": b, "bore": p.num("bore")?, "phase": PI / z1 as f64 }))?;
    let r1 = m * z1 as f64 / 2.0;
    let (rr, zp) = (m * z2 as f64 / 2.0, base + (ha + c) * m);
    // Ritzel: lokales z -> +X, x -> -Z (Zahnluecke nach unten), y -> +Y
    cmd(h, "solid.move", json!({ "bodies": [n1], "matrix": [[0.0, 0.0, 1.0, rr - b / 2.0], [0.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, zp + r1]], "name": format!("{n1} platzieren") }))?;
    Ok(json!({ "i": z2 as f64 / z1 as f64, "kronenrad": info }))
}

/// Verzahnung eines Stirnrads (gerade, schräg, Pfeil).
fn spur(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z, al) = (p.num("m")?, p.int("z")?, p.num("alpha")? * D2R);
    let beta = p.num("beta")? * D2R;
    let (x, ha, c, b, bore_d) = (p.num("x")?, p.num("ha")?, p.num("c")?, p.num("b")?, p.num("bore")?);
    let herr = p.flag("herringbone") && beta > 0.0;
    let sg = p.sign();
    let mt = m / beta.cos();
    let at = (al.tan() / beta.cos()).atan();
    let r = mt * z as f64 / 2.0;
    let rb = r * at.cos();
    let st = mt * (PI / 2.0 + 2.0 * x * al.tan());
    let ra_t = r + (ha + x) * m;
    let rf = r - (ha + c - x) * m;
    let half = move |rho: f64| st / (2.0 * r) + inv(at) - inv((rb / rho).min(1.0).acos());
    let ra = limit_tip(&half, rb, ra_t, 0.2 * m);
    if rf <= 0.0 {
        return Err(bad("Fußkreis ≤ 0: Zähnezahl zu klein oder Zahnhöhe zu groß."));
    }
    if bore_d / 2.0 >= rf - 0.3 * m {
        return Err(bad("Bohrung zu groß für den Fußkreis."));
    }
    let rho0 = rf.max(rb);
    let rho = involute_radii(rb, rho0, ra, 10);
    let tw = move |zz: f64| sg * zz * beta.tan() / r; // Drehwinkel des Profils bei Hoehe zz (rad)
    let name = p.text("name");
    let ph = p.extra("phase");
    let rin = (rf - 0.5 * m).max(0.5 * rf);
    let tooth = |rot: f64| external_tooth(z, rf, rin, &rho, &half, rot + ph);
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * rf, "height": b, "base": [0.0, 0.0, 0.0], "operation": "new", "name": format!("{name} Kern"), "body_name": name }))?;
    if beta == 0.0 {
        teeth(h, &name, z, &tooth, 0.0, b, &|_| 0.0, false, MAX_TURNS)?;
    } else if herr {
        let tri = move |zz: f64| tw(0.5 * b - (zz - 0.5 * b).abs());
        teeth(h, &name, z, &tooth, 0.0, b, &tri, true, MAX_TURNS)?;
    } else {
        teeth(h, &name, z, &tooth, 0.0, b, &tw, false, MAX_TURNS)?;
    }
    bore(h, &name, bore_d, 0.0, b)?;
    Ok(json!({ "d": 2.0 * r, "da": 2.0 * ra, "df": 2.0 * rf, "db": 2.0 * rb, "mt": mt }))
}

/// Hohlrad: Rohling als Zylinder, der sternfoermige Hohlraum wird (ggf. mit Drall) herausgeschnitten.
fn internal(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z, al) = (p.num("m")?, p.int("z")?, p.num("alpha")? * D2R);
    let beta = p.num("beta")? * D2R;
    let (x, ha, c, b, dout) = (p.num("x")?, p.num("ha")?, p.num("c")?, p.num("b")?, p.num("dout")?);
    let herr = p.flag("herringbone") && beta > 0.0;
    let sg = p.sign();
    let mt = m / beta.cos();
    let at = (al.tan() / beta.cos()).atan();
    let r = mt * z as f64 / 2.0;
    let rb = r * at.cos();
    let si = mt * (PI / 2.0 - 2.0 * x * al.tan());
    let rfi = r + (ha + c + x) * m;
    let mut rai = r - (ha - x) * m;
    if rai < rb {
        rai = rb * 1.002;
    }
    let half = move |rho: f64| si / (2.0 * r) + inv((rb / rho).min(1.0).acos()) - inv(at);
    let rho = involute_radii(rb, rai, rfi, 10);
    let rout = if dout > 0.0 { dout / 2.0 } else { rfi + 2.0 * m + 2.0 };
    if rout < rfi + 0.5 * m {
        return Err(bad("Außendurchmesser zu klein für den Zahnfuß."));
    }
    let name = p.text("name");
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * rout, "height": b, "base": [0.0, 0.0, 0.0], "operation": "new", "name": format!("{name} Ring"), "body_name": name }))?;
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * rfi, "height": b + 2.0, "base": [0.0, 0.0, -1.0], "operation": "cut", "targets": [name], "name": format!("{name} Fusskreis") }))?;
    let tw = move |zz: f64| sg * zz * beta.tan() / r;
    let ph = p.extra("phase");
    let rx = rfi + 0.5 * (rout - rfi);
    let tooth = |rot: f64| internal_tooth(z, &rho, &half, rot + ph, rx);
    if beta == 0.0 {
        teeth(h, &name, z, &tooth, 0.0, b, &|_| 0.0, false, MAX_TURNS)?;
    } else if herr {
        let tri = move |zz: f64| tw(0.5 * b - (zz - 0.5 * b).abs());
        teeth(h, &name, z, &tooth, 0.0, b, &tri, true, MAX_TURNS)?;
    } else {
        teeth(h, &name, z, &tooth, 0.0, b, &tw, false, MAX_TURNS)?;
    }
    Ok(json!({ "d": 2.0 * r, "da": 2.0 * rai, "df": 2.0 * rfi, "dout": 2.0 * rout }))
}

fn rack(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, n, al, ha, c, b, base) = (p.num("m")?, p.int("n")?, p.num("alpha")? * D2R, p.num("ha")?, p.num("c")?, p.num("b")?, p.num("base")?);
    let o = rack_outline(n, m, al, ha, c, base).map_err(bad)?;
    let name = p.text("name");
    make_solid(h, &Solid { name: &name, operation: "new", targets: None }, &o, 0.0, b, 0.0)?;
    Ok(json!({ "pt": PI * m, "L": n as f64 * PI * m }))
}

/// Schnecke (Typ ZA, Archimedes): Gewindeprofil im Axialschnitt als Trapez, mit `solid.coil` entlang der
/// Schraubenlinie gefuehrt (je Gang ein Coil), dazu ein Kern; die Enden werden plan abgeschnitten.
/// Jede Flanke bleibt dabei eine einzige Flaeche und die Berechnung dauert nur Sekunden.
fn worm(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, nn, q, al) = (p.num("m")?, p.int("N")?, p.num("q")?, p.num("alpha")? * D2R);
    let (ha, c, b, bore_d) = (p.num("ha")?, p.num("c")?, p.num("b")?, p.num("bore")?);
    let left = p.sign() < 0.0;
    let rp = q * m / 2.0;
    let (ra, rf) = (rp + ha * m, rp - (ha + c) * m);
    let pax = PI * m; // axiale Teilung eines Ganges
    let lead = nn as f64 * pax;
    if rf <= 0.5 {
        return Err(bad("Schneckenfuß zu klein: Durchmesserzahl q erhöhen."));
    }
    if bore_d / 2.0 >= rf - 0.3 * m {
        return Err(bad("Bohrung zu groß für den Schneckenkern."));
    }
    // halbe axiale Zahndicke in Radius r
    let hw = move |r: f64| pax / 4.0 + (rp - r) * al.tan();
    if 2.0 * hw(rf) >= pax - 1e-3 {
        return Err(bad("Gewinde am Fuß breiter als die Teilung."));
    }
    if hw(ra) <= 0.02 * m {
        return Err(bad("Zahnkopf läuft spitz zu: Kopfhöhenfaktor oder Eingriffswinkel verkleinern."));
    }
    if b / lead > MAX_WORM_TURNS + 1e-9 {
        return Err(bad(&format!("Schnecke zu lang ({:.0} Umdrehungen): höchstens {:.0} mm bei dieser Steigung.", b / lead, MAX_WORM_TURNS * lead)));
    }
    let name = p.text("name");
    let pad = lead; // Ueberstand an beiden Enden, wird abgeschnitten
    let turns = (b + 2.0 * pad) / lead;
    let r0 = rf - 0.2 * m; // Profil reicht in den Kern hinein
    let (hf, ht) = (hw(rf), hw(ra));
    // Zuerst der Kern, dann die Gaenge: sie ueberlappen ihn, das Verbinden bleibt dadurch schnell.
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * rf, "height": b + 2.0 * pad, "base": [0.0, 0.0, -pad], "operation": "new", "name": format!("{name} Kern"), "body_name": name }))?;
    for k in 0..nn {
        // Gang k = gleiche Schraubenlinie, um k axiale Teilungen versetzt. Auf der XZ-Skizze zeigt y nach -Z.
        let zc = -pad + k as f64 * pax;
        let pts: Vec<[f64; 2]> = [(r0, -hf), (rf, -hf), (ra, -ht), (ra, ht), (rf, hf), (r0, hf), (r0, -hf)]
            .iter()
            .map(|&(r, dz)| [r, -(zc + dz)])
            .collect();
        let sk = format!("{name} Profil {}", k + 1);
        cmd(h, "sketch.create", json!({ "plane": "XZ", "name": sk }))?;
        cmd(h, "sketch.line", json!({ "points": pts }))?;
        cmd(h, "sketch.finish", json!({}))?;
        let mut args = json!({ "sketch": sk, "turns": turns, "pitch": lead, "operation": "join", "targets": [name],
            "name": format!("{name} Gang {}", k + 1) });
        if left {
            args["clockwise"] = json!(true);
        }
        cmd(h, "solid.coil", args)?;
    }
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * ra + 2.0, "height": b, "base": [0.0, 0.0, 0.0], "operation": "intersect", "targets": [name], "name": format!("{name} Enden") }))?;
    bore(h, &name, bore_d, 0.0, b)?;
    let gam = (lead / (2.0 * PI * rp)).atan();
    Ok(json!({ "d": 2.0 * rp, "da": 2.0 * ra, "df": 2.0 * rf, "lead": lead, "gamma": gam.to_degrees() }))
}

/// Schneckenrad (zylindrisch, ohne Hohlkehle): Schrägverzahnung mit β = Steigungswinkel der Schnecke.
fn wheel(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let (m, z, nn, q, al) = (p.num("m")?, p.int("z")?, p.int("N")?, p.num("q")?, p.num("alpha")? * D2R);
    let (ha, c, bore_d) = (p.num("ha")?, p.num("c")?, p.num("bore")?);
    let sg = p.sign();
    let r = m * z as f64 / 2.0;
    let rb = r * al.cos();
    let gam = (nn as f64 / q).atan();
    let st = m * PI / 2.0;
    let ra_t = r + ha * m;
    let rf = r - (ha + c) * m;
    if rf <= 0.0 {
        return Err(bad("Zähnezahl zu klein."));
    }
    let half = move |rho: f64| st / (2.0 * r) + inv(al) - inv((rb / rho).min(1.0).acos());
    let ra = limit_tip(&half, rb, ra_t, 0.2 * m);
    let mut b = p.num("b")?;
    if b <= 0.0 {
        b = 0.75 * (q * m + 2.0 * ha * m);
    }
    if bore_d / 2.0 >= rf - 0.3 * m {
        return Err(bad("Bohrung zu groß für den Fußkreis."));
    }
    let rho0 = rf.max(rb);
    let rho = involute_radii(rb, rho0, ra, 10);
    let tw = move |zz: f64| sg * zz * gam.tan() / r;
    let name = p.text("name");
    let ph = p.extra("phase");
    let rin = (rf - 0.5 * m).max(0.5 * rf);
    let tooth = |rot: f64| external_tooth(z, rf, rin, &rho, &half, rot + ph);
    cmd(h, "solid.cylinder", json!({ "diameter": 2.0 * rf, "height": b, "base": [0.0, 0.0, 0.0], "operation": "new", "name": format!("{name} Kern"), "body_name": name }))?;
    teeth(h, &name, z, &tooth, 0.0, b, &tw, false, MAX_TURNS)?;
    bore(h, &name, bore_d, 0.0, b)?;
    Ok(json!({ "d": 2.0 * r, "da": 2.0 * ra, "df": 2.0 * rf, "b": b, "beta": gam.to_degrees(), "a": (q + z as f64) * m / 2.0 }))
}

#[allow(clippy::too_many_arguments)]
fn gear_json(name: &str, m: f64, z: usize, al: f64, beta: f64, hand: &str, herr: bool, ha: f64, c: f64, b: f64, bore: f64, phase: f64) -> Value {
    json!({ "name": name, "m": m, "z": z, "alpha": al, "beta": beta, "hand": hand, "herringbone": herr, "ha": ha, "c": c, "b": b, "bore": bore, "phase": phase })
}

/// Aufrufe je Lauf, damit das Host-Limit von SolidCraft nicht erreicht wird.
const CALL_BUDGET: usize = 64;

struct Stage {
    z1: usize,
    z2: usize,
    m: f64,
    beta: f64,
    w1: f64,
    w2: f64,
}

/// Stufen aus den Feldern: Stufe k = (Z(2k-1), Z(2k)); die erste Stufe ohne Zaehne beendet die Liste.
/// Modul, Schraegung und Dicke je Stufe/Zahnrad; 0 = Standardwert (Stufe 1 bzw. Standarddicke).
fn stages_from_fields(p: &Params) -> Result<Vec<Stage>, Error> {
    let key = |i: usize| if i == 1 { "z".to_string() } else { format!("z{i}") };
    let (m0, beta0, b0) = (p.num("m")?, p.num("beta")?, p.num("b")?);
    let mut out = Vec::new();
    for k in 0..4 {
        let (a, b) = (p.num(&key(2 * k + 1))?, p.num(&key(2 * k + 2))?);
        if a == 0.0 && b == 0.0 {
            break;
        }
        for (i, z) in [(2 * k + 1, a), (2 * k + 2, b)] {
            if z.fract() != 0.0 || !(6.0..=300.0).contains(&z) {
                return Err(bad(format!("Z{i} = {z}: für Stufe {} wird eine ganze Zähnezahl von 6 bis 300 gebraucht.", k + 1)));
            }
        }
        let m = if k == 0 { m0 } else { p.num(&format!("m{}", k + 1))? };
        let be = if k == 0 { beta0 } else { p.num(&format!("beta{}", k + 1))? };
        let w = |i: usize| -> Result<f64, Error> {
            let v = p.num(&format!("w{i}"))?;
            Ok(if v > 0.0 { v } else { b0 })
        };
        out.push(Stage {
            z1: a as usize,
            z2: b as usize,
            m: if m > 0.0 { m } else { m0 },
            beta: if k == 0 { be } else if be < 0.0 { 0.0 } else if be > 0.0 { be } else { beta0 },
            w1: w(2 * k + 1)?,
            w2: w(2 * k + 2)?,
        });
    }
    if out.is_empty() {
        return Err(bad("Z1 und Z2 angeben (mindestens eine Stufe)."));
    }
    Ok(out)
}

/// Mehrstufiges Stirnradgetriebe: Wellen in einer Reihe auf der X-Achse, jede Stufe auf eigener Ebene (Z).
/// Zaehnezahlen heissen Z1, Z2, Z3, ... in Reihenfolge Ritzel/Rad der Stufe 1, Ritzel/Rad der Stufe 2 usw.
fn gearbox(h: &mut dyn Host, p: &Params) -> Result<Value, Error> {
    let stages = stages_from_fields(p)?;
    let from = p.int("ab")?;
    if from > stages.len() {
        return Err(bad(format!("„Ab Stufe“ = {from}, es gibt aber nur {} Stufen.", stages.len())));
    }
    let (al, gap) = (p.num("alpha")?, p.num("gap")?);
    let (ha, clr, bore) = (p.num("ha")?, p.num("c")?, p.num("bore")?);
    let name = format!("{} (i={})", p.text("name"), fmt_i(stages.iter().map(|s| s.z2 as f64 / s.z1 as f64).product()));
    let herr = p.flag("herringbone");
    let hand0 = p.sign() > 0.0;
    // Lage der Wellen (X) und Ebenen (Z) vorab
    let mut x = vec![0.0];
    let mut level = vec![0.0];
    for s in &stages {
        let a = s.m / (s.beta * D2R).cos() * (s.z1 + s.z2) as f64 / 2.0;
        x.push(x.last().unwrap() + a);
        level.push(level.last().unwrap() + s.w1.max(s.w2) + gap);
    }
    let mut c = Counted { inner: h, n: 0 };
    let mut labels: Vec<Value> = Vec::new();
    let mut next_stage: Option<usize> = None;
    let mut last_cost = 0;
    for (k, s) in stages.iter().enumerate().skip(from - 1) {
        if c.n > 0 && c.n + last_cost > CALL_BUDGET {
            next_stage = Some(k + 1);
            break;
        }
        let start = c.n;
        let zl = level[k];
        // Steigungsrichtung wechselt von Stufe zu Stufe, damit die Welle nicht doppelt axial belastet wird
        let right1 = hand0 ^ (k % 2 == 1);
        let hand = |r: bool| if r { "rechts" } else { "links" };
        let (n1, n2) = (format!("{name} Z{} Ritzel W{}", 2 * k + 1, k + 1), format!("{name} Z{} Rad W{}", 2 * k + 2, k + 2));
        single(&mut c, "spur", gear_json(&n1, s.m, s.z1, al, s.beta, hand(right1), herr, ha, clr, s.w1, bore, 0.0))?;
        single(&mut c, "spur", gear_json(&n2, s.m, s.z2, al, s.beta, hand(!right1), herr, ha, clr, s.w2, bore, PI - PI / s.z2 as f64))?;
        if x[k] != 0.0 || zl != 0.0 {
            mv(&mut c, &n1, [x[k], 0.0, zl])?;
        }
        mv(&mut c, &n2, [x[k + 1], 0.0, zl])?;
        labels.push(json!({ "stufe": k + 1, format!("Z{}", 2 * k + 1): s.z1, format!("Z{}", 2 * k + 2): s.z2, "modul": s.m, "beta": s.beta,
            "dicke": [s.w1, s.w2], "welle": [k + 1, k + 2], "x": x[k], "x2": x[k + 1], "ebene_z": zl, "i": s.z2 as f64 / s.z1 as f64 }));
        last_cost = c.n - start;
    }
    let ratio: f64 = stages.iter().map(|s| s.z2 as f64 / s.z1 as f64).product();
    let mut out = json!({ "stufen": stages.len(), "uebersetzung_gesamt": ratio, "gebaut": labels, "aufrufe": c.n });
    if let Some(n) = next_stage {
        out["weiter"] = json!(format!("Weitere Stufen: Lauf mit „Ab Stufe“ = {n} wiederholen (Aufruf-Limit von SolidCraft)."));
    }
    Ok(out)
}

/// Ganze Zahlen ohne ".0" ausgeben: spart Platz im Manifest (Obergrenze 16 KiB).
fn compact(v: &Value) -> Value {
    match v.as_f64() {
        Some(x) if x.fract() == 0.0 && x.abs() < 1e12 && v.is_f64() => json!(x as i64),
        _ => v.clone(),
    }
}

/// Übersetzung für Körpernamen: höchstens 2 Nachkommastellen, ohne überflüssige Nullen.
fn fmt_i(v: f64) -> String {
    let s = format!("{:.2}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
