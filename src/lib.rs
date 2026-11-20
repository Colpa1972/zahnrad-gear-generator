//! Zahnrad-Plugin fuer SolidCraft. Der Kern (Profile, Befehlsfolgen) ist reines Rust und
//! laeuft auch nativ; die Anbindung an die WIT-Schnittstelle steht in `glue.rs` (nur Wasm).
pub mod build;
pub mod entries;
pub mod geom;
pub mod host;

#[cfg(target_arch = "wasm32")]
mod glue;

pub use host::{Error, Host};

/// Fuehrt einen Eintrag aus; `params` ist das JSON des Dialogs.
pub fn execute(host: &mut dyn Host, entry: &str, params: &str) -> Result<String, Error> {
    entries::run(host, entry, params)
}
