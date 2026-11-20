//! Zahnprofile als geschlossene Umrisse aus Splines, Linien und Dreipunkt-Boegen.
//! Alle Maße in mm, Winkel im Bogenmaß. Der Umriss liegt in der XY-Ebene, die Achse ist Z.
use std::f64::consts::PI;

pub type P2 = [f64; 2];

#[derive(Clone, Debug)]
pub enum Item {
    /// Spline durch die Punkte (Fit-Point-Spline)
    Spline(Vec<P2>),
    /// Linienzug
    Line(Vec<P2>),
    /// Kreisbogen von start nach end durch mid
    Arc { start: P2, mid: P2, end: P2 },
}

impl Item {
    pub fn start(&self) -> P2 {
        match self {
            Item::Spline(p) | Item::Line(p) => p[0],
            Item::Arc { start, .. } => *start,
        }
    }
    pub fn end(&self) -> P2 {
        match self {
            Item::Spline(p) | Item::Line(p) => *p.last().unwrap(),
            Item::Arc { end, .. } => *end,
        }
    }
}

/// Geschlossener Umriss: jedes Element beginnt exakt (bitgleich) dort, wo das vorige endet.
#[derive(Clone, Debug)]
pub struct Outline {
    pub items: Vec<Item>,
}

impl Outline {
    /// Prueft Geschlossenheit und Anschluss (exakt gleiche Koordinaten).
    pub fn check_closed(&self) -> Result<(), String> {
        let n = self.items.len();
        if n < 3 {
            return Err("Umriss hat zu wenige Elemente".into());
        }
        for i in 0..n {
            let a = self.items[i].end();
            let b = self.items[(i + 1) % n].start();
            if a != b {
                return Err(format!("Umriss nicht geschlossen bei Element {i}: {a:?} != {b:?}"));
            }
        }
        Ok(())
    }

    /// Flaeche des Umrisses (Boegen und Splines als Polygon abgetastet), Vorzeichen: CCW positiv.
    pub fn area(&self) -> f64 {
        let pts = self.sample(64);
        let mut a = 0.0;
        for i in 0..pts.len() {
            let p = pts[i];
            let q = pts[(i + 1) % pts.len()];
            a += p[0] * q[1] - q[0] * p[1];
        }
        a / 2.0
    }

    /// Abtastung fuer Tests (Splines nur ueber ihre Stuetzpunkte, Boegen fein).
    pub fn sample(&self, arc_n: usize) -> Vec<P2> {
        let mut out: Vec<P2> = Vec::new();
        for it in &self.items {
            match it {
                Item::Spline(p) | Item::Line(p) => out.extend(p[..p.len() - 1].iter().copied()),
                Item::Arc { start, mid, end } => {
                    let (c, r) = circle3(*start, *mid, *end);
                    let a0 = (start[1] - c[1]).atan2(start[0] - c[0]);
                    let am = (mid[1] - c[1]).atan2(mid[0] - c[0]);
                    let a1 = (end[1] - c[1]).atan2(end[0] - c[0]);
                    let ccw = wrap(am - a0) < wrap(a1 - a0);
                    let span = if ccw { wrap(a1 - a0) } else { -wrap(a0 - a1) };
                    for i in 0..arc_n {
                        let a = a0 + span * i as f64 / arc_n as f64;
                        out.push([c[0] + r * a.cos(), c[1] + r * a.sin()]);
                    }
                }
            }
        }
        out
    }

    pub fn rotated(&self, phi: f64) -> Outline {
        let (s, c) = phi.sin_cos();
        let rot = |p: P2| [p[0] * c - p[1] * s, p[0] * s + p[1] * c];
        Outline {
            items: self
                .items
                .iter()
                .map(|it| match it {
                    Item::Spline(p) => Item::Spline(p.iter().map(|&q| rot(q)).collect()),
                    Item::Line(p) => Item::Line(p.iter().map(|&q| rot(q)).collect()),
                    Item::Arc { start, mid, end } => Item::Arc { start: rot(*start), mid: rot(*mid), end: rot(*end) },
                })
                .collect(),
        }
    }
}

fn wrap(a: f64) -> f64 {
    let t = 2.0 * PI;
    ((a % t) + t) % t
}

/// Mittelpunkt und Radius des Kreises durch drei Punkte.
pub fn circle3(a: P2, b: P2, c: P2) -> (P2, f64) {
    let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
    let a2 = a[0] * a[0] + a[1] * a[1];
    let b2 = b[0] * b[0] + b[1] * b[1];
    let c2 = c[0] * c[0] + c[1] * c[1];
    let ux = (a2 * (b[1] - c[1]) + b2 * (c[1] - a[1]) + c2 * (a[1] - b[1])) / d;
    let uy = (a2 * (c[0] - b[0]) + b2 * (a[0] - c[0]) + c2 * (b[0] - a[0])) / d;
    ([ux, uy], ((a[0] - ux).powi(2) + (a[1] - uy).powi(2)).sqrt())
}

#[inline]
pub fn polar(r: f64, a: f64) -> P2 {
    [r * a.cos(), r * a.sin()]
}

#[inline]
pub fn inv(a: f64) -> f64 {
    a.tan() - a
}

/// Kopfkreis so begrenzen, dass die Zahnkopfdicke >= min_t bleibt.
pub fn limit_tip(half: &dyn Fn(f64) -> f64, r_base: f64, r_target: f64, min_t: f64) -> f64 {
    if r_target <= r_base || half(r_target) * 2.0 * r_target >= min_t {
        return r_target;
    }
    let (mut lo, mut hi) = (r_base, r_target);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if half(mid) * 2.0 * mid >= min_t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Aussenverzahnter Umriss mit `n` Zaehnen.
/// * `rho` – Radien entlang der Flanke von innen nach aussen (erster >= rf, letzter = Kopfradius)
/// * `half` – Halbwinkel der Zahndicke bei Radius rho
pub fn external(n: usize, rf: f64, rho: &[f64], half: &dyn Fn(f64) -> f64, rot: f64) -> Result<Outline, String> {
    let step = 2.0 * PI / n as f64;
    let ra = *rho.last().unwrap();
    let rho0 = rho[0];
    let h0 = half(rho0);
    if 2.0 * h0 >= step - 1e-3 {
        return Err("Zahndicke am Fuß ≥ Teilung: Profilverschiebung oder Eingriffswinkel ändern.".into());
    }
    let radial = rho0 > rf + 1e-9;
    let ang = |k: usize| k as f64 * step + rot;
    // Eckpunkte je Zahn: Rechts-/Linkskante am Fuss, auf der Flanke, am Kopf
    let r_root: Vec<P2> = (0..n).map(|k| polar(rf, ang(k) - h0)).collect();
    let l_root: Vec<P2> = (0..n).map(|k| polar(rf, ang(k) + h0)).collect();
    let mut items = Vec::new();
    for k in 0..n {
        let prev = (k + n - 1) % n;
        // Fussbogen von der Linkskante des vorigen Zahns zur Rechtskante dieses Zahns
        let a_mid = 0.5 * ((ang(k) - h0) + (ang(k) - step + h0));
        items.push(Item::Arc { start: l_root[prev], mid: polar(rf, a_mid), end: r_root[k] });
        // rechte Flanke
        let mut right: Vec<P2> = Vec::with_capacity(rho.len() + 1);
        if radial {
            items.push(Item::Line(vec![r_root[k], polar(rho0, ang(k) - h0)]));
            right.push(items.last().unwrap().end());
        } else {
            right.push(r_root[k]);
        }
        let mut tr = [0.0; 2];
        for (i, &r) in rho.iter().enumerate().skip(1) {
            let p = polar(r, ang(k) - half(r));
            if i == rho.len() - 1 {
                tr = p;
            }
            right.push(p);
        }
        items.push(Item::Spline(right));
        // Kopfbogen
        let tl = polar(ra, ang(k) + half(ra));
        items.push(Item::Arc { start: tr, mid: polar(ra, ang(k)), end: tl });
        // linke Flanke (von aussen nach innen)
        let mut left: Vec<P2> = vec![tl];
        for &r in rho.iter().rev().skip(1) {
            left.push(polar(r, ang(k) + half(r)));
        }
        if radial {
            let last = *left.last().unwrap();
            items.push(Item::Spline(left));
            items.push(Item::Line(vec![last, l_root[k]]));
        } else {
            // Endpunkt der Flanke ist der Fusspunkt, exakt der gemeinsame Punkt
            let n_l = left.len();
            left[n_l - 1] = l_root[k];
            items.push(Item::Spline(left));
        }
    }
    let o = Outline { items };
    o.check_closed()?;
    Ok(o)
}

/// Ein einzelner Aussenzahn (Zahn 0 bei Winkel `rot`), nach innen bis Radius `rin` verlaengert,
/// damit er den Grundzylinder (Radius rf) sicher ueberlappt.
pub fn external_tooth(n: usize, rf: f64, rin: f64, rho: &[f64], half: &dyn Fn(f64) -> f64, rot: f64) -> Result<Outline, String> {
    let ra = *rho.last().unwrap();
    let rho0 = rho[0];
    let h0 = half(rho0);
    if 2.0 * h0 >= 2.0 * PI / n as f64 - 1e-3 {
        return Err("Zahndicke am Fuß ≥ Teilung: Profilverschiebung oder Eingriffswinkel ändern.".into());
    }
    if h0 <= 1e-6 {
        return Err("Zahndicke am Fuß ≤ 0: Profilverschiebung oder Eingriffswinkel ändern.".into());
    }
    let radial = rho0 > rf + 1e-9;
    let a = rot;
    let in_r = polar(rin, a - h0);
    let in_l = polar(rin, a + h0);
    let r_root = polar(rf, a - h0);
    let l_root = polar(rf, a + h0);
    // Reihenfolge: rechte Flanke, Kopfbogen, linke Flanke, dann ein Linienzug (Fuss, Sehne unter dem Fuss, Fuss).
    let mut items = Vec::new();
    let mut right: Vec<P2> = Vec::new();
    let mut base: Vec<P2> = Vec::new(); // Schlusslinienzug
    if radial {
        right.push(polar(rho0, a - h0));
    } else {
        right.push(r_root);
    }
    let mut tr = [0.0; 2];
    for (i, &r) in rho.iter().enumerate().skip(1) {
        let p = polar(r, a - half(r));
        if i == rho.len() - 1 {
            tr = p;
        }
        right.push(p);
    }
    items.push(Item::Spline(right.clone()));
    let tl = polar(ra, a + half(ra));
    items.push(Item::Arc { start: tr, mid: polar(ra, a), end: tl });
    let mut left: Vec<P2> = vec![tl];
    for &r in rho.iter().rev().skip(1) {
        left.push(polar(r, a + half(r)));
    }
    if radial {
        base.push(*left.last().unwrap());
        base.push(l_root);
    } else {
        let n_l = left.len();
        left[n_l - 1] = l_root;
        base.push(l_root);
    }
    items.push(Item::Spline(left));
    base.push(in_l);
    base.push(in_r);
    base.push(r_root);
    if radial {
        base.push(right[0]);
    }
    items.push(Item::Line(base));
    let o = Outline { items };
    o.check_closed()?;
    Ok(o)
}

/// Ein einzelner Zahn (Material) eines Hohlrads, nach aussen bis Radius `rx` verlaengert.
/// `rho` von innen (Kopfkreis) nach aussen (Fusskreis).
pub fn internal_tooth(n: usize, rho: &[f64], half_m: &dyn Fn(f64) -> f64, rot: f64, rx: f64) -> Result<Outline, String> {
    let (rai, rfi) = (rho[0], *rho.last().unwrap());
    if half_m(rai) <= 1e-4 {
        return Err("Zahnkopf läuft spitz zu: Profilverschiebung x verkleinern.".into());
    }
    if 2.0 * half_m(rfi) >= 2.0 * PI / n as f64 - 1e-3 {
        return Err("Zahndicke am Fuß ≥ Teilung: Zähnezahl oder Profilverschiebung ändern.".into());
    }
    let a = rot;
    let (hf, ht) = (half_m(rfi), half_m(rai));
    let tip_r = polar(rai, a - ht);
    let tip_l = polar(rai, a + ht);
    let root_l = polar(rfi, a + hf);
    let root_r = polar(rfi, a - hf);
    let mut items = Vec::new();
    items.push(Item::Arc { start: tip_r, mid: polar(rai, a), end: tip_l });
    let mut l: Vec<P2> = vec![tip_l];
    for &r in &rho[1..rho.len() - 1] {
        l.push(polar(r, a + half_m(r)));
    }
    l.push(root_l);
    items.push(Item::Spline(l));
    let out_l = polar(rx, a + hf);
    let out_r = polar(rx, a - hf);
    items.push(Item::Line(vec![root_l, out_l, out_r, root_r]));
    let mut r: Vec<P2> = vec![root_r];
    for &rr in rho[1..rho.len() - 1].iter().rev() {
        r.push(polar(rr, a - half_m(rr)));
    }
    r.push(tip_r);
    items.push(Item::Spline(r));
    let o = Outline { items };
    o.check_closed()?;
    Ok(o)
}

/// Sternfoermiger Hohlraum eines Hohlrads (Begrenzung der Innenverzahnung), `n` Zaehne.
/// * `rho` – Radien von innen (Kopfkreis rai) nach aussen (Fusskreis rfi)
/// * `half_m` – Halbwinkel des Zahnmaterials bei Radius rho
pub fn internal_void(n: usize, rho: &[f64], half_m: &dyn Fn(f64) -> f64, rot: f64) -> Result<Outline, String> {
    let step = 2.0 * PI / n as f64;
    let (rai, rfi) = (rho[0], *rho.last().unwrap());
    if 2.0 * half_m(rfi) >= step - 1e-3 {
        return Err("Zahndicke am Fuß ≥ Teilung: Zähnezahl oder Profilverschiebung ändern.".into());
    }
    if half_m(rai) <= 1e-4 {
        return Err("Zahnkopf läuft spitz zu: Profilverschiebung x verkleinern.".into());
    }
    let ang = |k: usize| k as f64 * step + rot;
    // Spitzenpunkte (Zahnkopf innen) und Fusspunkte
    let tip_r: Vec<P2> = (0..n).map(|k| polar(rai, ang(k) - half_m(rai))).collect();
    let tip_l: Vec<P2> = (0..n).map(|k| polar(rai, ang(k) + half_m(rai))).collect();
    let root_l: Vec<P2> = (0..n).map(|k| polar(rfi, ang(k) + half_m(rfi))).collect();
    let root_r: Vec<P2> = (0..n).map(|k| polar(rfi, ang(k) - half_m(rfi))).collect();
    let mut items = Vec::new();
    for k in 0..n {
        let nx = (k + 1) % n;
        items.push(Item::Arc { start: tip_r[k], mid: polar(rai, ang(k)), end: tip_l[k] });
        // linke Flanke des Zahns k nach aussen
        let mut l: Vec<P2> = vec![tip_l[k]];
        for &r in &rho[1..rho.len() - 1] {
            l.push(polar(r, ang(k) + half_m(r)));
        }
        l.push(root_l[k]);
        items.push(Item::Spline(l));
        // Fussbogen ueber die Zahnluecke
        let a_mid = 0.5 * ((ang(k) + half_m(rfi)) + (ang(k) + step - half_m(rfi)));
        items.push(Item::Arc { start: root_l[k], mid: polar(rfi, a_mid), end: root_r[nx] });
        // rechte Flanke des naechsten Zahns nach innen
        let mut r: Vec<P2> = vec![root_r[nx]];
        for &rr in rho[1..rho.len() - 1].iter().rev() {
            r.push(polar(rr, ang(nx) - half_m(rr)));
        }
        r.push(tip_r[nx]);
        items.push(Item::Spline(r));
    }
    let o = Outline { items };
    o.check_closed()?;
    Ok(o)
}

/// Radien entlang einer Evolventenflanke, gleichmaessig im Abrollparameter.
pub fn involute_radii(rb: f64, r_from: f64, r_to: f64, nf: usize) -> Vec<f64> {
    let t0 = ((r_from / rb).powi(2) - 1.0).max(0.0).sqrt();
    let t1 = ((r_to / rb).powi(2) - 1.0).max(0.0).sqrt();
    let mut v: Vec<f64> = (0..=nf).map(|i| rb * (1.0 + (t0 + (t1 - t0) * i as f64 / nf as f64).powi(2)).sqrt()).collect();
    v[0] = r_from;
    *v.last_mut().unwrap() = r_to;
    v
}

/// Radien gleichmaessig zwischen zwei Werten.
pub fn linear_radii(r_from: f64, r_to: f64, nf: usize) -> Vec<f64> {
    (0..=nf).map(|i| r_from + (r_to - r_from) * i as f64 / nf as f64).collect()
}

/// Zahnstange: Linienzug-Umriss in der XY-Ebene (x: Laenge, y: Hoehe, Teilkreis bei y = 0).
pub fn rack_outline(n: usize, m: f64, alpha: f64, ha: f64, c: f64, base: f64) -> Result<Outline, String> {
    let pt = PI * m;
    let (ya, yf) = (ha * m, -(ha + c) * m);
    let t = alpha.tan();
    let tip_h = pt / 4.0 - ya * t;
    let root_h = pt / 4.0 + (-yf) * t;
    if tip_h < 0.05 * m {
        return Err("Zahnkopf der Zahnstange läuft spitz zu: Kopfhöhenfaktor oder Eingriffswinkel verkleinern.".into());
    }
    if root_h * 2.0 >= pt - 1e-6 {
        return Err("Zahnlücke am Fuß verschwindet: Eingriffswinkel oder Kopfspiel verkleinern.".into());
    }
    let mut pts: Vec<P2> = Vec::new();
    let x0 = 0.0;
    let len = n as f64 * pt;
    let yb = yf - base;
    pts.push([x0, yb]);
    pts.push([len + x0, yb]);
    // von rechts nach links (CCW: unten links -> unten rechts -> oben rechts -> ... -> oben links)
    pts.push([len + x0, yf]);
    for k in (0..n).rev() {
        let cx = x0 + (k as f64 + 0.5) * pt;
        pts.push([cx + root_h, yf]);
        pts.push([cx + tip_h, ya]);
        pts.push([cx - tip_h, ya]);
        pts.push([cx - root_h, yf]);
    }
    pts.push([x0, yf]);
    // Linienzug aus Einzelstrecken, damit jede Strecke an die vorige anschliesst
    let mut items = Vec::new();
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        if a != b {
            items.push(Item::Line(vec![a, b]));
        }
    }
    let o = Outline { items };
    o.check_closed()?;
    Ok(o)
}
