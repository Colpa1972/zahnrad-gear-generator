//! Befehlsfolgen: aus Umrissen werden Skizzen, Extrusionen und Sweeps.
use crate::geom::{Item, Outline};
use crate::host::{Error, Host};
use serde_json::{json, Value};

pub fn cmd(h: &mut dyn Host, id: &str, p: Value) -> Result<Value, Error> {
    h.run(id, p).map_err(|e| match e {
        Error::CommandFailed(m) => Error::CommandFailed(format!("{id}: {m}")),
        o => o,
    })
}

/// Skizze auf XY (Hoehe z0) mit dem Umriss.
pub fn draw_profile(h: &mut dyn Host, sketch: &str, z0: f64, o: &Outline) -> Result<(), Error> {
    let mut p = json!({ "plane": "XY", "name": sketch });
    if z0 != 0.0 {
        p["offset"] = json!(z0);
    }
    cmd(h, "sketch.create", p)?;
    // Aufeinanderfolgende Linien zu einem Linienzug zusammenfassen: spart Aufrufe.
    let mut items: Vec<Item> = Vec::new();
    for it in &o.items {
        if let (Some(Item::Line(prev)), Item::Line(pts)) = (items.last_mut(), it) {
            if prev.last() == pts.first() {
                prev.extend_from_slice(&pts[1..]);
                continue;
            }
        }
        items.push(it.clone());
    }
    for it in &items {
        match it {
            Item::Spline(pts) => cmd(h, "sketch.spline.fit_point", json!({ "points": pts }))?,
            Item::Line(pts) => cmd(h, "sketch.line", json!({ "points": pts }))?,
            Item::Arc { start, mid, end } => cmd(h, "sketch.arc.three_point", json!({ "start": start, "end": end, "through": mid }))?,
        };
    }
    cmd(h, "sketch.finish", json!({}))?;
    Ok(())
}

pub struct Solid<'a> {
    pub name: &'a str,
    pub operation: &'a str,
    pub targets: Option<Vec<String>>,
}

/// Koerper aus einem Umriss zwischen z0 und z1; `twist_deg` != 0: Sweep laengs der Achse mit Drall.
pub fn make_solid(h: &mut dyn Host, s: &Solid, o: &Outline, z0: f64, z1: f64, twist_deg: f64) -> Result<(), Error> {
    let prof = format!("{} Profil", s.name);
    draw_profile(h, &prof, z0, o)?;
    let mut p = json!({ "sketch": prof, "operation": s.operation, "name": s.name });
    if s.operation == "new" {
        p["body_name"] = json!(s.name);
    }
    if let Some(t) = &s.targets {
        p["targets"] = json!(t);
    }
    if twist_deg.abs() < 1e-9 {
        p["distance"] = json!(z1 - z0);
        cmd(h, "solid.extrude", p)?;
    } else {
        let path = format!("{} Pfad", s.name);
        cmd(h, "sketch.create", json!({ "plane": "YZ", "name": path }))?;
        cmd(h, "sketch.line", json!({ "points": [[0.0, z0], [0.0, z1]] }))?;
        cmd(h, "sketch.finish", json!({}))?;
        p["path_sketch"] = json!(path);
        p["path"] = json!(["l1"]);
        p["twist"] = json!(format!("{twist_deg:.9} deg"));
        cmd(h, "solid.sweep", p)?;
    }
    Ok(())
}

/// Bohrung durch den ganzen Koerper.
pub fn bore(h: &mut dyn Host, body: &str, diameter: f64, z0: f64, z1: f64) -> Result<(), Error> {
    if diameter <= 0.0 {
        return Ok(());
    }
    cmd(
        h,
        "solid.cylinder",
        json!({ "diameter": diameter, "height": z1 - z0 + 2.0, "base": [0.0, 0.0, z0 - 1.0], "operation": "cut", "targets": [body], "name": format!("{body} Bohrung") }),
    )?;
    Ok(())
}

/// Zaehne eines Rades: ein Zahn (ggf. mit Drall, bei Bedarf in Abschnitten und als Pfeil) wird an den
/// Koerper `body` angefuegt und dann `count`-mal rund um die Z-Achse vervielfacht. Die Zahl der Aufrufe
/// haengt damit nicht von der Zaehnezahl ab.
#[allow(clippy::too_many_arguments)]
pub fn teeth(
    h: &mut dyn Host,
    body: &str,
    count: usize,
    tooth_at: &dyn Fn(f64) -> Result<Outline, String>,
    z0: f64,
    z1: f64,
    turn: &dyn Fn(f64) -> f64,
    herringbone: bool,
    max_turns: f64,
) -> Result<(), Error> {
    let mut cuts: Vec<f64> = vec![z0];
    if herringbone {
        cuts.push(0.5 * (z0 + z1));
    }
    cuts.push(z1);
    let mut segs: Vec<(f64, f64)> = Vec::new();
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let turns = ((turn(b) - turn(a)) / (2.0 * std::f64::consts::PI)).abs();
        let n = (turns / max_turns).ceil().max(1.0) as usize;
        for i in 0..n {
            segs.push((a + (b - a) * i as f64 / n as f64, a + (b - a) * (i + 1) as f64 / n as f64));
        }
    }
    let mut names: Vec<String> = Vec::new();
    for (i, (za, zb)) in segs.iter().enumerate() {
        let o = tooth_at(turn(*za)).map_err(Error::InvalidInput)?;
        let twist = (turn(*zb) - turn(*za)).to_degrees();
        let nm = format!("{body} Zahn {}", i + 1);
        make_solid(h, &Solid { name: &nm, operation: "join", targets: Some(vec![body.to_string()]) }, &o, *za, *zb, twist)?;
        names.push(nm);
    }
    if count > 1 {
        cmd(h, "solid.pattern.circular", json!({ "features": names, "axis": "Z", "count": count, "name": format!("{body} Zahnkranz") }))?;
    }
    Ok(())
}
