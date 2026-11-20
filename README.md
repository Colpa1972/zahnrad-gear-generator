# Zahnrad-Generator – SolidCraft-Plugin (Rust/Wasm)

Evolventen-Zahnräder und Getriebe für SolidCraft (getestet mit 0.44/0.45). Version **0.9.0**.

Pro Art ein eigener Eintrag mit passendem Dialog (Menü „Plugins“ und Knopf in der Leiste):

| Eintrag | Inhalt |
|---|---|
| Stirnrad | gerade, schräg, Pfeil; Profilverschiebung, Bohrung |
| Hohlrad | innenverzahnt |
| Zahnstange | |
| Schnecke / Schneckenrad | Gangzahl, Durchmesserzahl q |
| Paar Z1/Z2, Hohlrad-Paar, Zahnstangen-Paar, Schnecken-Paar | richtiger Achsabstand, Gegensteigung, Zahnlücken-Phase; Übersetzung i im Körpernamen |
| Getriebe Z1…Zn | bis 4 Stufen (Z1–Z8), je Stufe Modul und Schrägung, je Zahnrad Dicke; Gesamtübersetzung im Körpernamen |

Die Bohrung ist standardmäßig 0 (keine Bohrung).

## Installation
`releases/zahnrad-0.9.0.zip` in SolidCraft als Plugin installieren (Ordner `zahnrad-0.9.0/` mit `plugin.json` und `zahnrad.wasm`).

## Bauen
    cargo test
    RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort
    wasm-tools component new target/wasm32-unknown-unknown/release/zahnrad.wasm -o package/zahnrad.wasm
    cargo run --example gen_manifest package/plugin.json

## Grenzen von SolidCraft (Plugin-Schnittstelle commands@0.1)
- Der Dialog wird automatisch aus dem Schema erzeugt: einspaltig, Felder nach Schlüsselnamen sortiert (daher Präfix `NN_`), keine Beschreibungen, Bilder oder Farben.
- Jeder Eintrag wird ein eigener Knopf; kein Untermenü, kein eigenes Symbol (RFC 0015 offen).
- Pro Lauf höchstens etwa 64 Aufrufe an SolidCraft (Getriebe: Feld „Ab Stufe“), Manifest höchstens 16 KiB.
