#!/usr/bin/env bash
# Baut zahnrad.wasm (Komponente) und das Paket zahnrad-1.0.0.zip.
# Voraussetzungen: Rust mit  rustup target add wasm32-unknown-unknown  und  cargo install wasm-tools
# Vorher: wit/plugin.wit aus dem SolidCraft-Quelltext hierher kopieren (siehe README).
set -euo pipefail
cd "$(dirname "$0")"
test -f wit/plugin.wit || { echo "wit/plugin.wit fehlt – aus dem SolidCraft-Quelltext kopieren (siehe README)"; exit 1; }
cargo test --release
cargo run -q --example gen_manifest
cargo build --release --target wasm32-unknown-unknown
wasm-tools component new target/wasm32-unknown-unknown/release/zahnrad.wasm -o package/zahnrad.wasm
rm -f zahnrad-1.0.0.zip
(cd package && zip -q ../zahnrad-1.0.0.zip plugin.json zahnrad.wasm)
echo "fertig: zahnrad-1.0.0.zip"
command -v solidcraft-cli >/dev/null && solidcraft-cli plugin check package || echo "(solidcraft-cli nicht gefunden: Prüfung übersprungen)"
