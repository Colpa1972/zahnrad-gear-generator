//! Schreibt package/plugin.json aus der Eintragstabelle (eine Quelle fuer Dialog und Code).
use serde_json::json;
use zahnrad::entries::{entries, schema};

fn main() {
    let es: Vec<_> = entries()
        .iter()
        .filter(|e| e.name == "gear")
        .map(|e| {
            let mut slots = vec![json!({ "kind": "menu", "name": e.menu })];
            if !e.ribbon.is_empty() {
                slots.push(json!({ "kind": "ribbon", "name": e.ribbon }));
            }
            json!({ "name": e.name, "effect": "edit", "input_schema": schema(e), "slots": slots })
        })
        .collect();
    let manifest = json!({
        "id": "zahnrad",
        "name": "Zahnrad-Generator",
        "version": "1.2.1",
        "description": "Evolventen-Zahnräder: Einzelräder und Getriebepaare: Stirnrad (gerade, schräg, Pfeil), Hohlrad, Zahnstange, Schnecke, Schneckenrad, Kegelrad (90°), Kronenrad.",
        "capabilities": ["commands@0.1"],
        "runtime": {
            "module": "zahnrad.wasm",
            "commands": [
                "sketch.create", "sketch.line", "sketch.spline.fit_point", "sketch.arc.three_point", "sketch.finish",
                "solid.extrude", "solid.sweep", "solid.cylinder", "solid.coil", "solid.move", "solid.pattern.circular", "solid.loft", "solid.cone"
            ],
            "entries": es,
            "limits": { "fuel": 2_000_000_000u64, "memory_mib": 64, "deadline_ms": 30000 },
            "permissions": {}
        }
    });
    let text = serde_json::to_string(&manifest).unwrap();
    assert!(text.len() < 64 * 1024, "plugin.json zu gross: {}", text.len());
    let path = std::env::args().nth(1).unwrap_or_else(|| "package/plugin.json".into());
    std::fs::write(&path, text + "\n").unwrap();
    println!("{path} geschrieben");
}
