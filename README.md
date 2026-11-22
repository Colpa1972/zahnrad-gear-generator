# Zahnrad-Generator – gear generator plugin for SolidCraft

**English:** An open-source (MIT) plugin for [SolidCraft](https://github.com/tangency-labs/solidcraft) that generates involute gears and gear pairs as ordinary editable solids. Written in Rust, built as a WebAssembly component (`craft:plugin`, `commands@0.1`). Current version: **1.0.0**. The user interface is German.

- Single gears: spur (straight, helical, herringbone), internal, rack, worm, worm wheel
- Pairs with correct centre distance and tooth phase: spur pair, pinion + internal gear, pinion + rack, worm + wheel; the ratio *i* is shown in the body names
- Multi-stage gearbox (Z1 … Z8, up to 4 stages): module and helix angle per stage, thickness per gear, total ratio in the body names

**Install:** download [`releases/zahnrad-0.9.0.zip`](releases/zahnrad-0.9.0.zip), unpack it so the folder `zahnrad-0.9.0` (with `plugin.json` and `zahnrad.wasm`) is in SolidCraft's plugin folder (Preferences → Plugins → *Show in folder*), then install and approve it there. Review the code first: a plugin runs Wasm code in SolidCraft.

**Known limits** (SolidCraft plugin interface): the dialog is generated from the schema, so every entry becomes its own ribbon button; no images or icons; at most ~64 host calls per run (gearbox: field "Ab Stufe"); manifest ≤ 16 KiB. See [tangency-labs/solidcraft#20086](https://github.com/tangency-labs/solidcraft/issues/20086) for the requested improvements (fields depending on a choice, grouped buttons).

Contributions and bug reports are welcome (GitHub issues).

---

# Deutsch

Evolventen-Zahnräder und Getriebe für SolidCraft (getestet mit 0.44/0.45), Version **0.9.0**. Pro Art ein eigener Eintrag mit passendem Dialog (Menü „Plugins“ und Knopf in der Leiste):

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
`releases/zahnrad-0.9.0.zip` herunterladen, den Ordner `zahnrad-0.9.0` in den Plugin-Ordner von SolidCraft legen (Einstellungen → Plugins → *Im Ordner zeigen*), dann dort installieren und freigeben.

## Bauen
    cargo test
    RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort
    wasm-tools component new target/wasm32-unknown-unknown/release/zahnrad.wasm -o package/zahnrad.wasm
    cargo run --example gen_manifest package/plugin.json

## Grenzen der Plugin-Schnittstelle
- Dialog wird automatisch aus dem Schema erzeugt: einspaltig, Felder nach Schlüsselnamen sortiert (daher Präfix `NN_`), keine Beschreibungen, Bilder oder Farben.
- Jeder Eintrag wird ein eigener Knopf; kein Untermenü, kein eigenes Symbol.
- Pro Lauf höchstens etwa 64 Aufrufe an SolidCraft (Getriebe: Feld „Ab Stufe“), Manifest höchstens 16 KiB.

## Lizenz
MIT, siehe `LICENSE`.
