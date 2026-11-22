use serde_json::{json, Value};
use zahnrad::entries::{entries, schema};
use zahnrad::geom::*;
use zahnrad::host::Recorder;
use zahnrad::{execute, Error};

fn run(entry: &str, p: Value) -> Result<(Recorder, Value), Error> {
    let mut r = Recorder::default();
    let out = execute(&mut r, entry, &p.to_string())?;
    Ok((r, serde_json::from_str(&out).unwrap()))
}
fn ids(r: &Recorder) -> Vec<&str> {
    r.calls.iter().map(|c| c.0.as_str()).collect()
}

#[test]
fn every_entry_runs_with_defaults() {
    for e in entries() {
        let (r, _) = run(e.name, json!({})).unwrap_or_else(|er| panic!("{}: {er}", e.name));
        assert!(!r.calls.is_empty(), "{}", e.name);
        // jede Skizze wird abgeschlossen
        let creates = ids(&r).iter().filter(|i| **i == "sketch.create").count();
        let finishes = ids(&r).iter().filter(|i| **i == "sketch.finish").count();
        assert_eq!(creates, finishes, "{}", e.name);
    }
}

#[test]
fn schemas_are_objects_with_defaults() {
    for e in entries() {
        let s = schema(&e);
        assert_eq!(s["type"], "object");
        for (i, fl) in e.fields.iter().enumerate() {
            let key = format!("{:02}_{}", i + 1, fl.key);
            let got = &s["properties"][key.as_str()]["default"];
            if let Some(x) = fl.def.as_f64() {
                assert_eq!(got.as_f64(), Some(x), "{}.{}", e.name, fl.key);
            } else {
                assert_eq!(got, &fl.def, "{}.{}", e.name, fl.key);
            }
        }
    }
}

#[test]
fn prefixed_schema_keys_are_accepted() {
    let (r, _) = run("gear", json!({ "01_type": "Stirnrad", "03_m": 3, "04_z": 12 })).unwrap();
    assert!(!r.calls.is_empty());
    for e in entries() {
        let _ = e;
    }
}

/// Alle Flankenpunkte liegen auf einer Evolvente des Grundkreises: Polarwinkel minus inv(alpha) ist konstant.
#[test]
fn flanks_are_true_involutes() {
    let (m, z, al) = (2.0_f64, 20usize, 20f64.to_radians());
    let r = m * z as f64 / 2.0;
    let rb = r * al.cos();
    let st = m * std::f64::consts::PI / 2.0;
    let half = move |rho: f64| st / (2.0 * r) + inv(al) - inv((rb / rho).min(1.0).acos());
    let rf = r - 1.25 * m;
    let ra = r + m;
    let rho = involute_radii(rb, rf.max(rb), ra, 10);
    let o = external(z, rf, &rho, &half, 0.0).unwrap();
    let mut checked = 0;
    for it in &o.items {
        if let Item::Spline(p) = it {
            let mut cst = Vec::new();
            for q in p {
                let rr = (q[0] * q[0] + q[1] * q[1]).sqrt();
                if rr <= rb + 1e-9 {
                    continue;
                }
                let th = q[1].atan2(q[0]);
                let a = (rb / rr).acos();
                // rechte Flanke: th + half = 0 (mod step); linke: th - half
                cst.push((th, inv(a)));
            }
            // innerhalb einer Flanke: th +- inv(a) bleibt konstant (Vorzeichen je Flanke)
            let k1: Vec<f64> = cst.iter().map(|(t, i)| t + i).collect();
            let k2: Vec<f64> = cst.iter().map(|(t, i)| t - i).collect();
            let spread = |v: &Vec<f64>| v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min);
            assert!(spread(&k1) < 1e-9 || spread(&k2) < 1e-9, "Flanke keine Evolvente: {} {}", spread(&k1), spread(&k2));
            checked += 1;
        }
    }
    assert_eq!(checked, 2 * z);
}

#[test]
fn tooth_thickness_at_pitch_circle() {
    let (m, al) = (2.5_f64, 20f64.to_radians());
    let z = 17.0;
    let r = m * z / 2.0;
    let rb = r * al.cos();
    let x = 0.3;
    let st = m * (std::f64::consts::PI / 2.0 + 2.0 * x * al.tan());
    let half = |rho: f64| st / (2.0 * r) + inv(al) - inv((rb / rho).min(1.0).acos());
    assert!((2.0 * half(r) * r - st).abs() < 1e-12);
}

#[test]
fn outlines_are_closed_and_enclose_expected_area() {
    let (_, info) = run("spur", json!({ "m": 2, "z": 24 })).unwrap();
    let d = info["d"].as_f64().unwrap();
    // Umrissflaeche liegt zwischen Fuss- und Kopfkreis
    let (m, z, al) = (2.0_f64, 24usize, 20f64.to_radians());
    let r = m * z as f64 / 2.0;
    let rb = r * al.cos();
    let st = m * std::f64::consts::PI / 2.0;
    let half = move |rho: f64| st / (2.0 * r) + inv(al) - inv((rb / rho).min(1.0).acos());
    let (rf, ra) = (r - 1.25 * m, r + m);
    let o = external(z, rf, &involute_radii(rb, rf.max(rb), ra, 10), &half, 0.3).unwrap();
    o.check_closed().unwrap();
    let a = o.area();
    assert!(a > std::f64::consts::PI * rf * rf && a < std::f64::consts::PI * ra * ra, "{a}");
    // grobe Pruefung: Flaeche ~ Kreis am Teilkreis (Zahn und Luecke gleich breit)
    let a_pitch = std::f64::consts::PI * r * r;
    assert!((a - a_pitch).abs() / a_pitch < 0.03, "{a} vs {a_pitch}");
    assert!((d - 2.0 * r).abs() < 1e-9);
}

#[test]
fn undercut_gear_gets_radial_lines() {
    // z = 8 ohne Verschiebung: Fusskreis unter dem Grundkreis
    let (r, _) = run("spur", json!({ "m": 2, "z": 8, "bore": 0 })).unwrap();
    assert!(r.calls.iter().any(|c| c.0 == "sketch.line"));
}

#[test]
fn spur_straight_is_extrude_helical_is_sweep() {
    let (r, _) = run("spur", json!({ "beta": 0, "z": 20, "b": 10, "bore": 6 })).unwrap();
    assert!(ids(&r).contains(&"solid.extrude") && !ids(&r).contains(&"solid.sweep"));
    assert_eq!(r.calls.last().unwrap().0, "solid.cylinder");
    let (r, _) = run("spur", json!({ "beta": 15, "z": 20, "b": 10 })).unwrap();
    let sw = r.calls.iter().find(|c| c.0 == "solid.sweep").unwrap();
    // twist = b*tan(beta)/r, r = mt*z/2
    let mt = 2.0 / 15f64.to_radians().cos();
    let tw = 10.0 * 15f64.to_radians().tan() / (mt * 20.0 / 2.0);
    let got: f64 = sw.1["twist"].as_str().unwrap().trim_end_matches(" deg").parse().unwrap();
    assert!((got - tw.to_degrees()).abs() < 1e-6, "{got} vs {}", tw.to_degrees());
    // Linksschraeg: negativer Drall
    let (r, _) = run("spur", json!({ "beta": 15, "hand": "L" })).unwrap();
    let got: f64 = r.calls.iter().find(|c| c.0 == "solid.sweep").unwrap().1["twist"].as_str().unwrap().trim_end_matches(" deg").parse().unwrap();
    assert!(got < 0.0);
}

#[test]
fn herringbone_is_two_sweeps_with_opposite_twist() {
    let (r, _) = run("spur", json!({ "beta": 20, "herringbone": true, "b": 14, "bore": 0 })).unwrap();
    let sw: Vec<&(String, Value)> = r.calls.iter().filter(|c| c.0 == "solid.sweep").collect();
    assert_eq!(sw.len(), 2);
    let t = |c: &(String, Value)| -> f64 { c.1["twist"].as_str().unwrap().trim_end_matches(" deg").parse().unwrap() };
    assert!((t(sw[0]) + t(sw[1])).abs() < 1e-6 && t(sw[0]) > 0.0);
    assert_eq!(sw[1].1["operation"], "join");
    assert_eq!(sw[1].1["targets"], json!(["Stirnrad Z20 m2"]));
}

#[test]
fn worm_is_one_coil_per_start_plus_core_and_flat_ends() {
    let (r, info) = run("worm", json!({ "m": 2, "N": 2, "q": 10, "b": 30, "bore": 6 })).unwrap();
    let ids: Vec<&str> = r.calls.iter().map(|c| c.0.as_str()).collect();
    assert_eq!(ids.iter().filter(|c| **c == "solid.coil").count(), 2);
    assert!(!ids.contains(&"solid.sweep"));
    // Kern, Enden abschneiden, Bohrung
    assert_eq!(ids.iter().filter(|c| **c == "solid.cylinder").count(), 3);
    let ops: Vec<&str> = r.calls.iter().filter(|c| c.0 == "solid.coil").map(|c| c.1["operation"].as_str().unwrap()).collect();
    assert_eq!(ops, ["join", "join"]);
    let cyl = r.calls.iter().find(|c| c.0 == "solid.cylinder").unwrap();
    assert_eq!(cyl.1["operation"], "new");
    let lead = info["lead"].as_f64().unwrap();
    assert!((lead - 2.0 * std::f64::consts::PI * 2.0).abs() < 1e-9);
    let coil = r.calls.iter().find(|c| c.0 == "solid.coil").unwrap();
    assert!((coil.1["pitch"].as_f64().unwrap() - lead).abs() < 1e-9);
    // Gaenge um eine axiale Teilung versetzt
    let y = |k: usize| -> f64 {
        r.calls.iter().filter(|c| c.0 == "sketch.line").nth(k).unwrap().1["points"][0][1].as_f64().unwrap()
    };
    assert!((y(0) - y(1) - std::f64::consts::PI * 2.0).abs() < 1e-9, "{} {}", y(0), y(1));
}

#[test]
fn left_hand_worm_is_a_clockwise_coil() {
    let (r, _) = run("worm", json!({ "hand": "L", "bore": 0 })).unwrap();
    assert_eq!(r.calls.iter().find(|c| c.0 == "solid.coil").unwrap().1["clockwise"], json!(true));
    let (r, _) = run("worm", json!({ "bore": 0 })).unwrap();
    assert!(r.calls.iter().find(|c| c.0 == "solid.coil").unwrap().1.get("clockwise").is_none());
}

#[test]
fn absurdly_long_worm_is_refused() {
    assert!(run("worm", json!({ "m": 1, "N": 1, "q": 10, "b": 500, "bore": 0 })).is_err());
}

#[test]
fn wheel_helix_equals_worm_lead_angle() {
    let (_, w) = run("worm", json!({ "m": 0.5, "N": 2, "q": 20, "b": 9, "bore": 0 })).unwrap();
    let (_, wh) = run("wheel", json!({ "m": 0.5, "z": 40, "N": 2, "q": 20, "ha": 0.75, "b": 3, "bore": 5.2 })).unwrap();
    assert!((w["gamma"].as_f64().unwrap() - 5.710593).abs() < 1e-5);
    assert!((wh["beta"].as_f64().unwrap() - w["gamma"].as_f64().unwrap()).abs() < 1e-9);
    assert!((wh["a"].as_f64().unwrap() - 15.0).abs() < 1e-12);
    assert!((wh["da"].as_f64().unwrap() - 20.75).abs() < 1e-9);
}

#[test]
fn internal_gear_is_ring_with_one_tooth_pattern() {
    let (r, _) = run("internal", json!({ "m": 2, "z": 40, "b": 10 })).unwrap();
    assert_eq!(r.calls[0].0, "solid.cylinder");
    assert!(r.calls.iter().any(|c| c.0 == "solid.cylinder" && c.1["operation"] == "cut"));
    let pat = r.calls.iter().find(|c| c.0 == "solid.pattern.circular").unwrap();
    assert_eq!(pat.1["count"], 40);
}

#[test]
fn call_count_does_not_depend_on_tooth_number() {
    let (a, _) = run("spur", json!({ "m": 1, "z": 12, "b": 10 })).unwrap();
    let (b, _) = run("spur", json!({ "m": 1, "z": 150, "b": 10 })).unwrap();
    assert_eq!(a.calls.len(), b.calls.len());
    assert!(b.calls.len() < 20);
}

#[test]
fn tooth_area_matches_full_outline() {
    use zahnrad::geom::*;
    let (m, z, al) = (2.0_f64, 24usize, 20f64.to_radians());
    let r = m * z as f64 / 2.0;
    let rb = r * al.cos();
    let st = m * std::f64::consts::PI / 2.0;
    let half = move |rho: f64| st / (2.0 * r) + inv(al) - inv((rb / rho).min(1.0).acos());
    let (ra, rf) = (r + m, r - 1.25 * m);
    let rho = involute_radii(rb, rf.max(rb), ra, 10);
    let full = external(z, rf, &rho, &half, 0.0).unwrap().area();
    let rin = rf - 0.5 * m;
    let tooth = external_tooth(z, rf, rin, &rho, &half, 0.0).unwrap().area();
    let h0 = half(rho[0]);
    let below = h0 * (rf * rf - rin * rin); // Sektor zwischen rin und rf
    let est = std::f64::consts::PI * rf * rf + z as f64 * (tooth - below);
    assert!((est - full).abs() / full < 0.01, "{est} vs {full}");
}

#[test]
fn rack_outline_is_closed() {
    let o = rack_outline(6, 2.0, 20f64.to_radians(), 1.0, 0.25, 4.0).unwrap();
    o.check_closed().unwrap();
    // Flaeche = Sockel + Zahnbereich: zwischen Teilungsflaeche und Hoehe
    let pt = std::f64::consts::PI * 2.0;
    let a = o.area();
    let lo = 6.0 * pt * (4.0 + 2.5 * 0.9);
    assert!(a > lo * 0.8 && a < 6.0 * pt * (4.0 + 2.5 + 2.0), "{a}");
}

#[test]
fn bad_input_is_rejected() {
    assert!(matches!(run("spur", json!({ "m": 0 })), Err(Error::InvalidInput(_))));
    assert!(matches!(run("spur", json!({ "foo": 1 })), Err(Error::InvalidInput(_))));
    assert!(matches!(run("spur", json!({ "z": "abc" })), Err(Error::InvalidInput(_))));
    assert!(matches!(run("spur", json!({ "bore": 50 })), Err(Error::InvalidInput(_))));
    assert!(matches!(run("nope", json!({})), Err(Error::Unsupported(_))));
    // Zahl als Text mit Komma ist erlaubt
    assert!(run("spur", json!({ "m": "2,5" })).is_ok());
}

#[test]
fn defaults_have_no_bore() {
    for e in zahnrad::entries::entries() {
        for f in &e.fields {
            if f.key == "bore" { assert_eq!(f.def.as_f64(), Some(0.0), "{}", e.name); }
        }
    }
}

#[test]
fn gear_entry_dispatches_every_type() {
    for (label, _) in zahnrad::entries::TYPES {
        let (r, _) = run("gear", json!({ "type": label })).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        assert!(!r.calls.is_empty(), "{label}");
    }
    assert!(run("gear", json!({ "type": "Unsinn" })).is_err());
    // nicht benoetigte Felder der Gesamtmaske werden ignoriert
    let (r, _) = run("gear", json!({ "type": "Zahnstange", "z": 17, "N": 3, "q": 12, "bore2": 5 })).unwrap();
    assert!(r.calls.iter().all(|c| c.0 != "solid.coil"));
}

#[test]
fn pair_spur_places_second_gear_at_center_distance_with_opposite_hand() {
    let (r, info) = run("pair-spur", json!({ "m": 2, "z": 20, "z2": 40, "beta": 15 })).unwrap();
    let a = info["a"].as_f64().unwrap();
    assert!((a - 2.0 * 60.0 / 2.0 / 15f64.to_radians().cos()).abs() < 1e-9);
    let mv = r.calls.iter().find(|c| c.0 == "solid.move").unwrap();
    assert_eq!(mv.1["bodies"], json!(["Paar (i=2) Rad Z40 m2"]));
    assert!((mv.1["translate"][0].as_f64().unwrap() - a).abs() < 1e-9);
    let twists: Vec<String> = r.calls.iter().filter(|c| c.0 == "solid.sweep").map(|c| c.1["twist"].as_str().unwrap().to_string()).collect();
    assert!(twists.iter().any(|t| !t.starts_with('-')) && twists.iter().any(|t| t.starts_with('-')), "{twists:?}");
}

#[test]
fn pair_internal_needs_ten_more_teeth() {
    assert!(run("pair-internal", json!({ "z": 20, "z2": 25 })).is_err());
    let (r, info) = run("pair-internal", json!({ "z": 20, "z2": 60 })).unwrap();
    assert!((info["a"].as_f64().unwrap() - 40.0).abs() < 1e-9);
    let mv = r.calls.iter().find(|c| c.0 == "solid.move").unwrap();
    assert!((mv.1["translate"][0].as_f64().unwrap() + 40.0).abs() < 1e-9);
}

#[test]
fn pair_rack_touches_pitch_circle() {
    let (r, info) = run("pair-rack", json!({ "m": 2, "z": 18 })).unwrap();
    assert!((info["r"].as_f64().unwrap() - 18.0).abs() < 1e-9);
    let mv = r.calls.iter().find(|c| c.0 == "solid.move").unwrap();
    assert!((mv.1["translate"][1].as_f64().unwrap() + 18.0).abs() < 1e-9);
}

#[test]
fn pair_worm_wheel_sits_on_worm_axis_distance_rotated_about_x() {
    let (r, info) = run("pair-worm", json!({ "m": 2, "N": 2, "q": 10, "z": 30 })).unwrap();
    assert!((info["a"].as_f64().unwrap() - 40.0).abs() < 1e-9);
    let mv = r.calls.iter().find(|c| c.0 == "solid.move").unwrap();
    let m = &mv.1["matrix"];
    assert_eq!(m[0][3].as_f64().unwrap(), 40.0);
    assert_eq!(m[1][2].as_f64().unwrap(), -1.0);
    assert_eq!(m[2][1].as_f64().unwrap(), 1.0);
}

fn zf(stages: &[(u32, u32)]) -> Value {
    let mut o = serde_json::Map::new();
    for (k, (a, b)) in stages.iter().enumerate() {
        o.insert(if k == 0 { "z".into() } else { format!("z{}", 2 * k + 1) }, json!(a));
        o.insert(format!("z{}", 2 * k + 2), json!(b));
    }
    Value::Object(o)
}

#[test]
fn gearbox_labels_stages_and_respects_call_budget() {
    let mut three = zf(&[(20, 40), (18, 54), (15, 60)]);
    three["joints"] = json!(false);
    let (r, out) = run("gearbox", three).unwrap();
    let names: Vec<String> = r.calls.iter().filter(|c| c.0 == "solid.cylinder").filter_map(|c| c.1["body_name"].as_str().map(String::from)).collect();
    // Zaehnezahl und Modul stehen im Namen; i waechst mit jedem Rad, Antriebsritzel ohne i
    for z in [20, 40, 18, 54, 15, 60] {
        assert!(names.iter().any(|n| n.contains(&format!(" Z{z} m"))), "Z{z} fehlt: {names:?}");
    }
    assert!(names.iter().any(|n| n.contains("Ritzel Z20 m2 W1 (Antrieb)")));
    assert!(names.iter().any(|n| n.contains("Rad Z60 m2 W4 (i=24, Abtrieb)")), "{names:?}");
    assert!(r.calls.len() <= 64 + 25);
    assert!((out["uebersetzung_gesamt"].as_f64().unwrap() - 24.0).abs() < 1e-9);
    let mut six = zf(&[(20, 40), (20, 40), (20, 40), (20, 40)]);
    six["joints"] = json!(false);
    let (r2, out2) = run("gearbox", six.clone()).unwrap();
    assert!(r2.calls.len() <= 64 + 25);
    assert!(out2["weiter"].is_string());
    let mut s3 = six.clone();
    s3["ab"] = json!(4);
    let (_, out3) = run("gearbox", s3).unwrap();
    assert_eq!(out3["gebaut"].as_array().unwrap().len(), 1);
    assert!(run("gearbox", json!({ "z": 20, "z2": 40, "z3": 18 })).is_err());
}

#[test]
fn only_one_menu_entry_via_gear() {
    let (r, _) = run("gear", json!({ "type": "Getriebe mehrstufig (Z1, Z2, Z3 …)", "z": 20, "z2": 40, "z3": 15, "z4": 45 })).unwrap();
    assert!(!r.calls.is_empty());
}

#[test]
fn gearbox_stage_module_overrides_base() {
    let mut p = zf(&[(20, 40), (18, 54)]);
    p["m"] = json!(1.5);
    p["m2"] = json!(0.5);
    let (_, out) = run("gearbox", p).unwrap();
    let g = out["gebaut"].as_array().unwrap();
    assert!((g[0]["x2"].as_f64().unwrap() - 1.5 * 60.0 / 2.0).abs() < 1e-9);
    assert!((g[1]["x2"].as_f64().unwrap() - g[1]["x"].as_f64().unwrap() - 0.5 * 72.0 / 2.0).abs() < 1e-9);
}

#[test]
fn gearbox_per_gear_thickness_and_stage_helix() {
    let mut p = zf(&[(20, 40), (18, 54)]);
    p["w3"] = json!(4);
    p["w4"] = json!(6);
    p["beta2"] = json!(15);
    p["b"] = json!(10);
    let (r, out) = run("gearbox", p).unwrap();
    let g = out["gebaut"].as_array().unwrap();
    assert_eq!(g[1]["dicke"], json!([4.0, 6.0]));
    assert_eq!(g[1]["beta"], json!(15.0));
    assert_eq!(g[0]["beta"], json!(0.0));
    // zweite Ebene liegt ueber der ersten (Dicke 10 + Abstand 2)
    assert!((g[1]["ebene_z"].as_f64().unwrap() - 12.0).abs() < 1e-9);
    let ext: Vec<f64> = r.calls.iter().filter(|c| c.0 == "solid.extrude").map(|c| c.1["distance"].as_f64().unwrap()).collect();
    assert!(ext.contains(&4.0) || r.calls.iter().any(|c| c.0 == "solid.sweep"));
}

#[test]
fn bevel_cone_angle_follows_tooth_ratio_and_apexes_meet() {
    let (r, out) = run("pair-bevel", json!({ "z": 16, "z2": 32, "m": 2, "b": 10 })).unwrap();
    assert!((out["delta1"].as_f64().unwrap() - (0.5f64).atan().to_degrees()).abs() < 1e-9);
    // beide Spitzen bei (0, 0, m*z2/2): Ritzel-Spitze liegt auf der Z-Achse, das Rad wird dorthin verschoben
    assert_eq!(out["spitze"], json!([0.0, 0.0, 32.0]));
    let mv = r.calls.iter().find(|c| c.0 == "solid.move").unwrap();
    assert_eq!(mv.1["matrix"], json!([[0.0, 0.0, -1.0, 16.0], [0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 32.0]]));
    assert_eq!(ids(&r).iter().filter(|i| **i == "solid.loft").count(), 2);
    assert_eq!(ids(&r).iter().filter(|i| **i == "solid.pattern.circular").count(), 2);
}

#[test]
fn bevel_rejects_too_wide_teeth_and_big_bore() {
    assert!(run("bevel", json!({ "z": 20, "z2": 40, "b": 200 })).is_err());
    assert!(run("bevel", json!({ "z": 20, "z2": 40, "bore": 30 })).is_err());
    assert!(run("bevel", json!({ "z": 20, "z2": 40, "bore": 8 })).is_ok());
}

#[test]
fn crown_is_conjugate_to_pinion_and_limited_by_undercut_and_pointing() {
    let (r, out) = run("pair-crown", json!({ "z": 15, "z2": 45, "m": 2, "b": 6 })).unwrap();
    // innen setzt bei 1,04 · R · cos(alpha) der Unterschnitt ein
    let ri = 1.04 * 45.0 * 20f64.to_radians().cos();
    assert!((out["kronenrad"]["ri"].as_f64().unwrap() - ri).abs() < 1e-9);
    let mv = r.calls.iter().find(|c| c.0 == "solid.move").unwrap();
    assert!((mv.1["matrix"][0][3].as_f64().unwrap() - ri).abs() < 1e-9);
    let loft = r.calls.iter().find(|c| c.0 == "solid.loft").unwrap();
    assert_eq!(loft.1["sections"].as_array().unwrap().len(), 6);
    // Flankenwinkel waechst nach aussen: Zahn am Fuss innen schmaler als aussen, am Kopf innen breiter
    let lines: Vec<_> = r.calls.iter().filter(|c| c.0 == "sketch.line" && c.1["points"].as_array().unwrap().len() > 10).collect();
    let (a, b) = (lines[0].1["points"].as_array().unwrap(), lines[5].1["points"].as_array().unwrap());
    assert!(a[1][0].as_f64().unwrap() < b[1][0].as_f64().unwrap());
    let top = |p: &Vec<serde_json::Value>| p[11][0].as_f64().unwrap();
    assert!(top(a) > top(b) && top(b) > 0.0);
    // zu breit: aussen laufen die Zaehne spitz zu
    assert!(run("pair-crown", json!({ "z": 15, "z2": 45, "m": 2, "b": 40 })).is_err());
    assert!(run("crown", json!({ "z": 45, "z2": 15, "b": 40 })).is_err());
    assert!(run("crown", json!({ "z": 45, "z2": 15 })).is_ok());
    // Breite 0 = automatisch, unabhaengig vom Modul (kleines Modul darf nicht an einer festen Breite scheitern)
    for m in [0.5, 1.0, 2.0, 3.0] {
        let (_, o) = run("pair-crown", json!({ "z": 15, "z2": 45, "m": m })).unwrap();
        let w = o["kronenrad"]["b"].as_f64().unwrap();
        assert!(w > 0.5 * m && w < 6.0 * m, "m={m} b={w}");
    }
}

#[test]
fn joints_are_built_per_part_with_gear_links_and_study() {
    // Stirnradpaar: zwei Komponenten, zwei Drehgelenke gegen das Gestell, eine Zahnradkopplung, eine Bewegungsstudie
    let (r, _) = run("pair-spur", json!({ "m": 2, "z": 15, "z2": 45 })).unwrap();
    let ids: Vec<&str> = r.calls.iter().map(|c| c.0.as_str()).collect();
    assert_eq!(ids.iter().filter(|i| **i == "component.create").count(), 2);
    assert_eq!(ids.iter().filter(|i| **i == "joint.as_built").count(), 2);
    let g = r.calls.iter().find(|c| c.0 == "joint.gear").unwrap();
    assert_eq!((g.1["ra"].as_i64(), g.1["rb"].as_i64()), (Some(15), Some(45)));
    assert!(ids.contains(&"motion.study"));
    // ohne Gelenke wie bisher
    let (r0, _) = run("pair-spur", json!({ "m": 2, "z": 15, "z2": 45, "joints": false })).unwrap();
    assert!(!r0.calls.iter().any(|c| c.0.starts_with("component.") || c.0.starts_with("joint.")));
    // Getriebe: eine Komponente je Welle (Rad und Ritzel einer Welle zusammen), Kopplung je Stufe
    let (rg, _) = run("gearbox", json!({ "z": 15, "z2": 45, "z3": 10, "z4": 50, "m": 2 })).unwrap();
    let comps: Vec<String> = rg.calls.iter().filter(|c| c.0 == "component.create").map(|c| c.1["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(comps.len(), 3, "{comps:?}");
    assert_eq!(rg.calls.iter().filter(|c| c.0 == "joint.gear").count(), 2);
    assert_eq!(rg.calls.iter().filter(|c| c.0 == "joint.as_built").count(), 3);
}
