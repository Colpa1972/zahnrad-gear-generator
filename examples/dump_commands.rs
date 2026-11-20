//! Schreibt die Befehlsfolge eines Eintrags als JSON-Liste (`batch`-Format) zum Nachspielen in SolidCraft.
//! Aufruf: cargo run --example dump_commands -- <eintrag> '<params-json>' <ausgabe.json>
use zahnrad::host::Recorder;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (entry, params, out) = (&a[1], a.get(2).map(String::as_str).unwrap_or("{}"), a.get(3).map(String::as_str).unwrap_or("commands.json"));
    let mut r = Recorder::default();
    let info = zahnrad::execute(&mut r, entry, params).expect("Eintrag fehlgeschlagen");
    let cmds: Vec<_> = r.calls.iter().map(|(c, p)| serde_json::json!({ "command": c, "params": p })).collect();
    std::fs::write(out, serde_json::to_string(&serde_json::json!({ "commands": cmds })).unwrap()).unwrap();
    println!("{} Befehle -> {out}\n{info}", cmds.len());
}
