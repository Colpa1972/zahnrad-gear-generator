//! Anbindung an die WIT-Schnittstelle `craft:plugin@0.1.0` (world `command-plugin`, siehe wit/plugin.wit).
wit_bindgen::generate!({ path: "wit", world: "command-plugin" });

use crate::host::{Error, Host};
use craft::plugin::types::{CommandFailure, FieldFailure, PluginError};
use exports::craft::plugin::entry::Guest;

struct App;

struct Bridge;
impl Host for Bridge {
    fn run(&mut self, id: &str, params: serde_json::Value) -> Result<serde_json::Value, Error> {
        match craft::plugin::commands::run(id, &params.to_string()) {
            Ok(reply) => serde_json::from_str(&reply)
                .map_err(|e| Error::CommandFailed(format!("{id}: Antwort nicht lesbar: {e}"))),
            Err(e) => Err(Error::CommandFailed(format!("{id}: {e:?}"))),
        }
    }
}

impl Guest for App {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        crate::execute(&mut Bridge, &entry, &params).map_err(|e| match e {
            Error::InvalidInput(reason) => PluginError::InvalidInput(FieldFailure { field: "params".into(), reason }),
            Error::Unsupported(reason) => PluginError::InvalidInput(FieldFailure { field: "entry".into(), reason }),
            Error::CommandFailed(reason) => {
                let (command, reason) = match reason.split_once(": ") {
                    Some((c, r)) => (c.to_string(), r.to_string()),
                    None => ("?".to_string(), reason),
                };
                PluginError::CommandFailed(CommandFailure { command, reason })
            }
        })
    }
}

export!(App);
