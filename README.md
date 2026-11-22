# Zahnrad-Generator – gear generator plugin for SolidCraft

**English:** An open-source (MIT) plugin for [SolidCraft](https://github.com/tangency-labs/solidcraft) that generates involute gears and gear pairs as ordinary editable solids. Written in Rust, built as a WebAssembly component (`craft:plugin`, `commands@0.1`). Current version: **1.2.3**. The user interface is German.

- One ribbon button "Zahnrad" with a type list; the dialog only shows the fields of the chosen type (`x-solidcraft-visible-when`, SolidCraft 0.46+)
- Single gears: spur (straight, helical, herringbone), internal, rack, worm, worm wheel, straight bevel gear (90°, Tredgold approximation), crown gear (face gear, exact conjugate flank to the involute pinion)
- Pairs with correct centre distance and tooth phase: spur pair, pinion + internal gear, pinion + rack, worm + wheel, bevel + bevel (apexes coincide), crown gear + spur pinion; the ratio *i* is shown in the body names
- Multi-stage gearbox (Z1 … Z8, up to 4 stages): module and helix angle per stage, thickness per gear, total ratio in the body names

**Install:** download [`releases/zahnrad-1.2.3.zip`](releases/zahnrad-1.2.3.zip), unpack it so the folder `zahnrad-1.2.3` (with `plugin.json` and `zahnrad.wasm`) is in SolidCraft's plugin folder (Preferences → Plugins → *Show in folder*), then install and approve it there. Review the code first: a plugin runs Wasm code in SolidCraft.

**Known limits** (SolidCraft plugin interface): the dialog is generated from the schema, so no images or icons; the bevel gear is a geometric approximation (Tredgold, tooth flanks lofted, not exact spherical involutes); the crown gear flank is computed numerically as the envelope of the pinion involute (6 radial sections lofted); usable face width limited inside by undercut (≈1.04·R·cos α) and outside by pointed teeth; at most ~64 host calls per run (gearbox: field "Ab Stufe"); manifest ≤ 64 KiB. See [tangency-labs/solidcraft#20086](https://github.com/tangency-labs/solidcraft/issues/20086) for the requested improvements (fields depending on a choice, grouped buttons).

Contributions and bug reports are welcome (GitHub issues).

---

# Deutsch

Evolventen-Zahnräder und Getriebe für SolidCraft (getestet mit 0.46.2), Version **1.2.3**. Ein Eintrag „Zahnrad“ mit Auswahl der Art; der Dialog zeigt nur die Felder, die zur gewählten Art gehören:

| Eintrag | Inhalt |
|---|---|
| Stirnrad | gerade, schräg, Pfeil; Profilverschiebung, Bohrung |
| Hohlrad | innenverzahnt |
| Zahnstange | |
| Schnecke / Schneckenrad | Gangzahl, Durchmesserzahl q |
| Kegelrad | geradverzahnt, 90°-Achswinkel, Kegelwinkel aus Z1/Z2 (Näherung nach Tredgold) |
| Kronenrad | Planrad mit Zähnen auf der Stirnseite; Flanke exakt konjugiert zur Ritzel-Evolvente (abgewälzt), Zahnbreite innen durch Unterschnitt, außen durch Spitzwerden begrenzt |
| Paar Z1/Z2, Hohlrad-Paar, Zahnstangen-Paar, Schnecken-Paar, Kegelrad-Paar, Kronenrad + Stirnritzel | richtiger Achsabstand, Gegensteigung, Zahnlücken-Phase; Übersetzung i im Körpernamen |
| Getriebe Z1…Zn | bis 4 Stufen (Z1–Z8), je Stufe Modul und Schrägung, je Zahnrad Dicke; Gesamtübersetzung im Körpernamen |

Die Bohrung ist standardmäßig 0 (keine Bohrung).

## Installation
`releases/zahnrad-1.2.3.zip` herunterladen, den Ordner `zahnrad-1.2.3` in den Plugin-Ordner von SolidCraft legen (Einstellungen → Plugins → *Im Ordner zeigen*), dann dort installieren und freigeben.

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
