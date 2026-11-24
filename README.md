# Zahnrad-Generator – Gear generator for SolidCraft

**[English](#english) · [Deutsch](#deutsch)**

---

# English

An open-source (MIT) plugin for [SolidCraft](https://github.com/tangency-labs/solidcraft) that generates involute gears, gear pairs and gearboxes as ordinary, editable solids – including joints and a motion study, so the mechanism can be simulated right away. Written in Rust, built as a WebAssembly component (`craft:plugin`, `commands@0.1`). Tested with SolidCraft 0.46.2. Current version: **1.4.0**. The plugin dialog is in German.

## Features

| Entry | Content |
|---|---|
| Spur gear | straight, helical, herringbone; profile shift, bore |
| Internal gear | ring gear with internal teeth |
| Rack | |
| Worm / worm wheel | number of starts, diameter quotient q |
| Bevel gear | straight teeth, 90° shaft angle, cone angle from Z1/Z2 (Tredgold approximation) |
| Crown gear | face gear with teeth on the end face; the flank is the exact conjugate (envelope) of the involute pinion; face width limited inside by undercut and outside by pointed teeth (0 = automatic) |
| Pairs | spur pair, pinion + internal gear, pinion + rack, worm + wheel, bevel + bevel (apexes coincide), crown gear + spur pinion: correct centre distance, tooth phase; ratio *i* in the body names |
| Gearbox Z1 … Zn | up to 4 stages (Z1–Z8); module and helix angle per stage, thickness per gear; cumulative ratio *i* in the body names |

- **Joints and simulation (new in 1.4.0):** every gear is built as its own component. The plugin adds revolute joints (a slider joint for the rack), the gear coupling with the correct ratio and a drive motion study. Switch it off with the option "Drehgelenke + Kopplung einbauen" (build joints + coupling).
- **Body names** contain tooth count, module and ratio, e.g. `Ritzel Z15 m2` (pinion), `Rad Z45 m2 (i=3)` (wheel).
- **Ratio:** i = n_in / n_out = Z2/Z1 · Z4/Z3 · … (pitch circles, not tip circles; i > 1 means speed reduction).
- The bore is 0 by default (no bore).

## Installation

Download [`releases/zahnrad-1.4.0.zip`](releases/zahnrad-1.4.0.zip) (or from the [Releases](../../releases) page), unpack it so that the folder `zahnrad-1.4.0` (with `plugin.json` and `zahnrad.wasm`) is in SolidCraft's plugin folder (Preferences → Plugins → *Show in folder*), then install and approve it there. Review the code first: a plugin runs Wasm code inside SolidCraft.

## Building

    cargo test
    RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort
    wasm-tools component new target/wasm32-unknown-unknown/release/zahnrad.wasm -o package/zahnrad.wasm
    cargo run --example gen_manifest

## Known limits

- The dialog is generated from the schema: one column, fields sorted by key name (hence the `NN_` prefixes), no images, icons or descriptions. Every plugin entry becomes its own ribbon button. See [tangency-labs/solidcraft#20086](https://github.com/tangency-labs/solidcraft/issues/20086) for the requested improvements.
- The bevel gear is a geometric approximation (Tredgold, lofted flanks), not exact spherical involutes.
- At most about 64 host calls per run (gearbox: use the field "Ab Stufe" to continue); manifest at most 64 KiB.
- In the collision check of the motion study, the internal gear, the worm and a Z10/Z50 gearbox stage still show 0.03–0.1 mm of contact that SolidCraft could not measure.

Contributions and bug reports are welcome (GitHub issues). License: MIT, see `LICENSE`.

---

# Deutsch

Open-Source-Plugin (MIT) für [SolidCraft](https://github.com/tangency-labs/solidcraft): Evolventen-Zahnräder, Paare und Getriebe als normale, editierbare Volumenkörper – samt Gelenken und Bewegungsstudie, sodass sich der Mechanismus sofort simulieren lässt. Geschrieben in Rust, gebaut als WebAssembly-Komponente (`craft:plugin`, `commands@0.1`). Getestet mit SolidCraft 0.46.2, Version **1.4.0**. Der Plugin-Dialog ist deutsch.

## Funktionen

| Eintrag | Inhalt |
|---|---|
| Stirnrad | gerade, schräg, Pfeil; Profilverschiebung, Bohrung |
| Hohlrad | innenverzahnt |
| Zahnstange | |
| Schnecke / Schneckenrad | Gangzahl, Durchmesserzahl q |
| Kegelrad | geradverzahnt, 90°-Achswinkel, Kegelwinkel aus Z1/Z2 (Näherung nach Tredgold) |
| Kronenrad | Planrad mit Zähnen auf der Stirnseite; Flanke exakt konjugiert zur Ritzel-Evolvente (abgewälzt), Zahnbreite innen durch Unterschnitt, außen durch Spitzwerden begrenzt (0 = automatisch) |
| Paare | Stirnrad-Paar, Ritzel + Hohlrad, Ritzel + Zahnstange, Schnecke + Rad, Kegelrad-Paar (Spitzen fallen zusammen), Kronenrad + Stirnritzel: richtiger Achsabstand, Zahnlücken-Phase; Übersetzung *i* im Körpernamen |
| Getriebe Z1…Zn | bis 4 Stufen (Z1–Z8); je Stufe Modul und Schrägung, je Zahnrad Dicke; aufgelaufene Übersetzung *i* im Körpernamen |

- **Gelenke und Simulation (neu in 1.4.0):** Jedes Zahnrad wird als eigene Komponente gebaut. Das Plugin legt Drehgelenke (bei der Zahnstange ein Schiebegelenk), die Getriebekopplung mit dem richtigen Übersetzungsverhältnis und eine Antriebsstudie an. Abschaltbar über „Drehgelenke + Kopplung einbauen“.
- **Körpernamen** enthalten Zähnezahl, Modul und Übersetzung, z. B. `Ritzel Z15 m2`, `Rad Z45 m2 (i=3)`.
- **Übersetzung:** i = n_an / n_ab = Z2/Z1 · Z4/Z3 · … (Teilkreise, nicht Kopfkreise; i > 1 heißt Untersetzung ins Langsame).
- Die Bohrung ist standardmäßig 0 (keine Bohrung).

## Installation

[`releases/zahnrad-1.4.0.zip`](releases/zahnrad-1.4.0.zip) herunterladen (oder von der Seite [Releases](../../releases)), den Ordner `zahnrad-1.4.0` (mit `plugin.json` und `zahnrad.wasm`) in den Plugin-Ordner von SolidCraft legen (Einstellungen → Plugins → *Im Ordner zeigen*), dann dort installieren und freigeben. Vorher den Quelltext prüfen: Ein Plugin führt Wasm-Code in SolidCraft aus.

## Bauen

    cargo test
    RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort
    wasm-tools component new target/wasm32-unknown-unknown/release/zahnrad.wasm -o package/zahnrad.wasm
    cargo run --example gen_manifest

## Grenzen

- Der Dialog wird automatisch aus dem Schema erzeugt: einspaltig, Felder nach Schlüsselnamen sortiert (daher die Präfixe `NN_`), keine Bilder, Symbole oder Beschreibungen. Jeder Eintrag wird ein eigener Menüband-Knopf. Gewünschte Verbesserungen: [tangency-labs/solidcraft#20086](https://github.com/tangency-labs/solidcraft/issues/20086).
- Das Kegelrad ist eine geometrische Näherung (Tredgold, geloftete Flanken), keine exakten sphärischen Evolventen.
- Pro Lauf höchstens etwa 64 Aufrufe an SolidCraft (Getriebe: Feld „Ab Stufe“ zum Fortsetzen); Manifest höchstens 64 KiB.
- In der Kollisionsprüfung der Bewegungsstudie zeigen Hohlrad, Schnecke und eine Getriebestufe Z10/Z50 noch 0,03–0,1 mm Berührung, die SolidCraft nicht ausmessen konnte.

Beiträge und Fehlermeldungen sind willkommen (GitHub-Issues). Lizenz: MIT, siehe `LICENSE`.
