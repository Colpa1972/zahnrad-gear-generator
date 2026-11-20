use serde_json::Value;

/// Fehler des Plugins; entspricht den `plugin-error`-Fällen der WIT-Schnittstelle.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidInput(String),
    Unsupported(String),
    CommandFailed(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInput(s) | Error::Unsupported(s) | Error::CommandFailed(s) => f.write_str(s),
        }
    }
}

/// Die Verbindung zu SolidCraft: ein Befehl mit JSON-Parametern, Antwort als JSON.
pub trait Host {
    fn run(&mut self, id: &str, params: Value) -> Result<Value, Error>;
}

/// Zeichnet jeden Befehl auf (fuer Tests und zum Nachspielen in SolidCraft).
#[derive(Default)]
pub struct Recorder {
    pub calls: Vec<(String, Value)>,
    next_sketch_curve: usize,
}

impl Host for Recorder {
    fn run(&mut self, id: &str, params: Value) -> Result<Value, Error> {
        self.calls.push((id.to_string(), params));
        // Antwort mit plausiblen Kurven-IDs, damit der Aufrufer sie lesen kann
        if id == "sketch.create" {
            self.next_sketch_curve = 0;
        }
        let mut curves = Vec::new();
        if id.starts_with("sketch.line") || id.starts_with("sketch.arc") || id.starts_with("sketch.spline") {
            self.next_sketch_curve += 1;
            let p = if id.starts_with("sketch.line") { "l" } else if id.starts_with("sketch.arc") { "a" } else { "s" };
            curves.push(format!("{p}{}", self.next_sketch_curve));
        }
        Ok(serde_json::json!({ "created": { "curves": curves } }))
    }
}

/// Zaehlt die Aufrufe, die an den Host gehen (Aufruf-Budget je Lauf).
pub struct Counted<'a> {
    pub inner: &'a mut dyn Host,
    pub n: usize,
}

impl Host for Counted<'_> {
    fn run(&mut self, id: &str, params: Value) -> Result<Value, Error> {
        self.n += 1;
        self.inner.run(id, params)
    }
}
