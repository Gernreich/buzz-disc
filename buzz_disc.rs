//! The two buzz discs, each from a few numbers: its cut file and its
//! SolidCraft design (2026-11-08; drawn by hand in Inkscape until then).
//!
//!     target/release/buzz_disc [--disc=NAME [--teeth=N ...]] [--out=DIR] [--solidcraft=DIR] [--no-write]
//!
//! Without --disc it writes both at their defaults; a parameter flag needs
//! --disc, since each has its own.
//!
//!   sawtooth-disc  BuzzDisc1.svg: a disc with a toothed rim
//!   plain-bar      BuzzDisc2.svg: a rectangle
//!
//! Both have two cord holes either side of the centre, on the centreline.
//!
//! THE RIM. The drawn rim was a polar wave: going round, the radius rises
//! linearly with the angle up a flank, holds at the tip along an arc about
//! the centre, and falls linearly down the next flank, with no flat at the
//! root. It was not regular: 34 teeth whose tips and roots wandered ±0.6mm
//! round the turn, two tops split in two, and a short flat where the drawing
//! closed. Here it is regular: `teeth` teeth to the turn, each flank `flank`
//! of the tooth's pitch, tips at `tip_d`, roots at `root_d`, the averages of
//! the drawn rim. The first tooth rises from the centreline.
//!
//! In the cut file a flank is a cubic Bézier with the polar curve's own end
//! tangents (within a micrometre of it over a flank's 2.3°) and a top is an
//! arc. In the SolidCraft design the flanks are drawn from their formula,
//! and the teeth are one tooth joined to the root disc and patterned round
//! it `teeth` times, so the count is a parameter too.
use lasermade_tools::flat::{f as fm, outs, write, Checks, Params, Sheet, Unit::*};
use std::f64::consts::PI;

const HERE: &str = env!("CARGO_MANIFEST_DIR");
const BED: (f64, f64) = (600.0, 305.0);
const DENSITY: f64 = 0.68; // Baltic birch, g/cm3
const MIN_EDGE: f64 = 3.0;

pub const DISCS: [(&str, &str); 2] = [("sawtooth-disc", "BuzzDisc1"), ("plain-bar", "BuzzDisc2")];

fn table(disc: &str) -> Params {
    let mut v = vec![("ply", 3.0, Len, "the ply: one lamination"), ("laminations", 1.0, Num, "discs glued face to face: the design's thickness is laminations × ply")];
    match disc {
        "sawtooth-disc" => v.extend([
            ("teeth", 34.0, Num, "teeth round the rim"),
            ("tip_d", 159.5, Len, "across the tips"),
            ("root_d", 138.2, Len, "across the roots"),
            ("flank", 0.22, Ratio, "each flank's share of a tooth's pitch; the tip has the rest"),
            ("hole_d", 5.0, Len, "a cord hole"),
            ("hole_gap", 30.0, Len, "cord hole centre to centre"),
        ]),
        _ => v.extend([
            ("length", 150.0, Len, "the bar, end to end"),
            ("width", 40.0, Len, "the bar, across"),
            ("hole_d", 5.0, Len, "a cord hole"),
            ("hole_gap", 25.0, Len, "cord hole centre to centre"),
        ]),
    }
    Params::new(&v)
}

type P = (f64, f64);

/// The rim's radius at angle th (radians), and the tooth geometry.
struct Rim {
    n: usize,
    rr: f64,
    rt: f64,
    pitch: f64,
    fa: f64,
}

impl Rim {
    fn of(p: &Params) -> Rim {
        let n = p.get("teeth") as usize;
        let pitch = 2.0 * PI / n as f64;
        Rim { n, rr: p.get("root_d") / 2.0, rt: p.get("tip_d") / 2.0, pitch, fa: p.get("flank") * pitch }
    }
    fn r(&self, th: f64) -> f64 {
        let u = th.rem_euclid(self.pitch);
        if u < self.fa {
            self.rr + (self.rt - self.rr) * u / self.fa
        } else if u > self.pitch - self.fa {
            self.rt - (self.rt - self.rr) * (u - (self.pitch - self.fa)) / self.fa
        } else {
            self.rt
        }
    }
    /// Exactly: the root disc and, each tooth, its top's sector and two
    /// flanks, a flank's area being the integral of (r² - rr²)/2 over it.
    fn area(&self) -> f64 {
        let (rr, rt) = (self.rr, self.rt);
        let top = (rt * rt - rr * rr) / 2.0 * (self.pitch - 2.0 * self.fa);
        let flank = self.fa / 2.0 * ((rt.powi(3) - rr.powi(3)) / (3.0 * (rt - rr)) - rr * rr);
        PI * rr * rr + self.n as f64 * (top + 2.0 * flank)
    }
    /// The outline as SVG path data about (cx, cy): flanks as cubics with
    /// the polar curve's end tangents, tops as arcs.
    fn d(&self, cx: f64, cy: f64) -> String {
        let at = |r: f64, th: f64| (cx + r * th.cos(), cy + r * th.sin());
        let pt = |q: P| format!("{},{}", fm(q.0), fm(q.1));
        let dr = self.rt - self.rr;
        // a flank from (r0, th0) to (r0 + s·dr, th0 + fa), s = ±1
        let flank = |r0: f64, th0: f64, s: f64| -> String {
            let p = |t: f64| at(r0 + s * dr * t, th0 + t * self.fa);
            let dp = |t: f64| {
                let (r, th) = (r0 + s * dr * t, th0 + t * self.fa);
                (s * dr * th.cos() - r * self.fa * th.sin(), s * dr * th.sin() + r * self.fa * th.cos())
            };
            let (p0, p3, d0, d3) = (p(0.0), p(1.0), dp(0.0), dp(1.0));
            format!(" C {} {} {}", pt((p0.0 + d0.0 / 3.0, p0.1 + d0.1 / 3.0)), pt((p3.0 - d3.0 / 3.0, p3.1 - d3.1 / 3.0)), pt(p3))
        };
        let mut o = format!("M {}", pt(at(self.rr, 0.0)));
        for k in 0..self.n {
            let th = k as f64 * self.pitch;
            o.push_str(&flank(self.rr, th, 1.0));
            // the top: an arc about the centre, increasing angle (sweep 1)
            o.push_str(&format!(" A {},{} 0 0 1 {}", fm(self.rt), fm(self.rt), pt(at(self.rt, th + self.pitch - self.fa))));
            o.push_str(&flank(self.rt, th + self.pitch - self.fa, -1.0));
        }
        o.push_str(" Z");
        o
    }
    /// The bounding box about the centre: the extremes of the radius
    /// function on the four axis directions and every corner.
    fn half_extent(&self) -> (f64, f64, f64, f64) {
        let mut pts: Vec<P> = vec![];
        for k in 0..self.n {
            let th = k as f64 * self.pitch;
            for u in [0.0, self.fa, self.pitch - self.fa] {
                pts.push((self.r(th + u) * (th + u).cos(), self.r(th + u) * (th + u).sin()));
            }
            // inside a flank the extremes come from sampling finely
            for i in 1..200 {
                for base in [th, th + self.pitch - self.fa] {
                    let a = base + self.fa * i as f64 / 200.0;
                    pts.push((self.r(a) * a.cos(), self.r(a) * a.sin()));
                }
            }
        }
        for q in 0..4 {
            let a = q as f64 * PI / 2.0;
            pts.push((self.r(a) * a.cos(), self.r(a) * a.sin()));
        }
        let xs = pts.iter().map(|p| p.0);
        let ys = pts.iter().map(|p| p.1);
        (xs.clone().fold(f64::INFINITY, f64::min), xs.fold(f64::NEG_INFINITY, f64::max), ys.clone().fold(f64::INFINITY, f64::min), ys.fold(f64::NEG_INFINITY, f64::max))
    }
}

// --- the SolidCraft design --------------------------------------------------------

fn design(disc: &str, p: &Params) -> (String, f64) {
    use lasermade_tools::solidcraft::Script;
    let mut s = Script::new();
    p.declare(&mut s);
    s.len("thick", "laminations*ply", "read only: the disc's thickness");
    let body = format!("buzz disc {}", disc.replace('-', " "));
    let holes = std::f64::consts::PI * p.get("hole_d").powi(2) / 4.0 * 2.0;
    let sk = s.sketch("XY", None, &format!("{body} sketch"));
    let area = if disc == "sawtooth-disc" {
        s.angle("pitch", "360 deg/teeth", "read only: one tooth's angle");
        s.angle("fa", "flank*pitch", "read only: one flank's angle");
        s.cmd("sketch.circle.center", r#"{"center":["0 mm","0 mm"],"diameter":"root_d"}"#.into());
        Rim::of(p).area()
    } else {
        s.cmd("sketch.rectangle.two_point", r#"{"p0":["-length/2","-width/2"],"p1":["length/2","width/2"]}"#.into());
        p.get("length") * p.get("width")
    };
    for sg in ["-", ""] {
        s.cmd("sketch.circle.center", format!(r#"{{"center":["{sg}hole_gap/2","0 mm"],"diameter":"hole_d"}}"#));
    }
    s.finish();
    s.extrude(&sk, "thick", false, "new", &[], &body);
    if disc == "sawtooth-disc" {
        // one tooth: the root arc, a flank up, the tip's arc, a flank down
        let t = format!("{body} tooth");
        let tk = s.sketch("XY", None, &format!("{t} sketch"));
        s.cmd("sketch.arc.center_point", r#"{"center":["0 mm","0 mm"],"start":["root_d/2","0 mm"],"end":["root_d/2*cos(pitch)","root_d/2*sin(pitch)"]}"#.into());
        s.cmd("sketch.equation_curve", r#"{"x":"(root_d/2 + (tip_d - root_d)/2*t)*cos(t*fa)","y":"(root_d/2 + (tip_d - root_d)/2*t)*sin(t*fa)","start":"0","end":"1","tolerance":0.001}"#.into());
        s.cmd("sketch.arc.center_point", r#"{"center":["0 mm","0 mm"],"start":["tip_d/2*cos(fa)","tip_d/2*sin(fa)"],"end":["tip_d/2*cos(pitch - fa)","tip_d/2*sin(pitch - fa)"]}"#.into());
        s.cmd("sketch.equation_curve", r#"{"x":"(tip_d/2 - (tip_d - root_d)/2*t)*cos(pitch - fa + t*fa)","y":"(tip_d/2 - (tip_d - root_d)/2*t)*sin(pitch - fa + t*fa)","start":"0","end":"1","tolerance":0.001}"#.into());
        s.finish();
        s.extrude(&tk, "thick", false, "join", &[&body], &t);
        s.cmd("solid.pattern.circular", format!(r#"{{"features":["{t}"],"axis":"Z","count":"teeth","name":"{t} pattern"}}"#));
    }
    s.measure();
    let th = p.get("laminations") * p.get("ply");
    (s.text(), (area - holes) * th)
}

fn run() -> Result<i32, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let pick = args.iter().find_map(|a| a.strip_prefix("--disc=").map(String::from));
    let rest: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--disc=")).collect();
    let names: Vec<(&str, &str)> = match &pick {
        Some(b) => DISCS.iter().filter(|(n, _)| n == b).copied().collect(),
        None => DISCS.to_vec(),
    };
    if names.is_empty() {
        return Err(format!("error: no disc {}; they are {}.", pick.unwrap(), DISCS.map(|b| b.0).join(", ")));
    }
    let mut rc = 0;
    let mut first = true;
    for (name, stem) in names {
        let defaults = table(name);
        let mut p = table(name);
        let rest = p.parse(rest.clone())?;
        if pick.is_none() && !p.changed(&defaults).is_empty() {
            return Err("error: a parameter flag needs --disc=NAME: each has its own.".into());
        }
        let o = outs(rest, &format!("--disc=NAME {}", p.flags()))?;
        if let Some(dir) = &o.solidcraft {
            let (text, v) = design(name, &p);
            write(&format!("{dir}/buzz-disc-{name}.solidcraft.json"), &text)?;
            if std::env::var("SC_VOLUMES").is_ok() {
                eprintln!("buzz disc {}\t{v:.4}", name.replace('-', " "));
            }
            continue;
        }
        if !first {
            println!();
        }
        first = false;
        let (gap, hd) = (p.get("hole_gap"), p.get("hole_d"));
        let holes = PI * hd * hd / 4.0 * 2.0;
        let mut c_ok = vec![];
        let (sheet, area, inner, what) = if name == "sawtooth-disc" {
            let rim = Rim::of(&p);
            let (x0, x1, y0, y1) = rim.half_extent();
            let mut sh = Sheet::new(x1 - x0, y1 - y0);
            let (cx, cy) = (-x0, -y0);
            for sg in [-1.0, 1.0] {
                sh.circle("#00ff00", cx + sg * gap / 2.0, cy, hd);
            }
            sh.path("#000000", rim.d(cx, cy));
            c_ok.push((rim.n >= 3 && rim.fa > 0.0 && 2.0 * rim.fa < rim.pitch && rim.rt > rim.rr, format!("{} teeth, {:.2}° flanks, {:.2}° tips", rim.n, rim.fa.to_degrees(), (rim.pitch - 2.0 * rim.fa).to_degrees())));
            (sh, rim.area(), rim.rr, format!("{} teeth, {} across the tips", rim.n, p.get("tip_d")))
        } else {
            let (l, w) = (p.get("length"), p.get("width"));
            let mut sh = Sheet::new(l, w);
            for sg in [-1.0, 1.0] {
                sh.circle("#00ff00", l / 2.0 + sg * gap / 2.0, w / 2.0, hd);
            }
            sh.rect("#000000", 0.0, 0.0, l, w);
            c_ok.push((l > gap + hd && w > hd, format!("{l} × {w}mm")));
            (sh, l * w, w / 2.0, format!("{} × {}mm", l, w))
        };
        let g = (area - holes) * p.get("ply") * DENSITY / 1000.0;
        println!("{name:14} {what}: {:.1} × {:.1}mm, {g:.0}g a lamination, cord holes {hd}mm, {gap}mm apart", sheet.w, sheet.h);
        let mut c = Checks::new();
        c.note(true, "cord holes either side of the centre", "on the centreline, so it spins true".into());
        let between = gap - hd;
        let edge = if name == "sawtooth-disc" { inner - gap / 2.0 - hd / 2.0 } else { (p.get("length") / 2.0 - gap / 2.0 - hd / 2.0).min(inner - hd / 2.0) };
        c.note(between >= MIN_EDGE && edge >= MIN_EDGE, "ply round the cord holes", format!("{between:.1}mm between them, {edge:.1}mm to the edge (>= {MIN_EDGE})"));
        for (ok, d) in c_ok {
            c.note(ok, "the outline", d);
        }
        c.note(sheet.w <= BED.0 && sheet.h <= BED.1, "fits the bed", format!("{} × {}mm", BED.0, BED.1));
        if c.done() != 0 {
            rc = 1;
        }
        if !o.no_write {
            let dir = o.out.clone().unwrap_or_else(|| HERE.to_string());
            let made = p.changed(&defaults);
            let with = if made.is_empty() { String::new() } else { format!(" Made with {made}.") };
            let desc = format!(
                "1 user unit = 1mm. A buzz disc, {}, drawn by buzz_disc.rs --disc={name}.{with} Cut it in 3mm Baltic birch. Green #00ff00 cuts the two cord holes first, then black #000000 frees the part.",
                name.replace('-', " ")
            );
            write(&format!("{dir}/{stem}.svg"), &sheet.svg(&format!("Buzz disc, {}", name.replace('-', " ")), &desc))?;
        }
    }
    Ok(rc)
}

fn main() {
    std::process::exit(match run() {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    });
}
