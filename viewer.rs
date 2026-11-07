//! The two buzz discs in 3D, from their cut files.
//!
//!     target/release/viewer [--out=buzz-disc-3d.html]
//!
//! Each disc is its cut file's own outline at 3 mm, read by
//! the turn3d module in lasermade-tools, stood on edge with a cord loop threaded through its
//! two centre holes and running out to a hand at each side. Pulling the hands
//! apart unwinds the twisted cord and spins the disc about it, so "Spin it" turns
//! the disc about the line between the two holes' midpoint and the hands.
use lasermade_tools::turn3d::*;
use std::f64::consts::PI;

const HERE: &str = env!("CARGO_MANIFEST_DIR");
const DISCS: [(&str, &str); 2] = [("sawtooth disc", "BuzzDisc1.svg"), ("plain bar", "BuzzDisc2.svg")];
const REACH: i64 = 170; // mm from the disc to each hand

fn build() -> Vec<(String, J)> {
    let mut out = vec![];
    for (name, f) in DISCS {
        let ps = parts(&format!("{HERE}/{f}"));
        assert!(ps.len() == 1 && ps[0].h.len() == 2, "{f} {}", ps.len());
        let p = &ps[0];
        let holes: Vec<(f64, f64)> = p
            .h
            .iter()
            .map(|h| (h.iter().fold(0.0, |s, q| s + q[0].f()) / h.len() as f64, h.iter().fold(0.0, |s, q| s + q[1].f()) / h.len() as f64))
            .collect();
        let mx = holes.iter().fold(0.0, |s, h| s + h.0) / 2.0;
        let my = holes.iter().fold(0.0, |s, h| s + h.1) / 2.0;
        let d = centred(p, Some((F(mx), F(my)))); // centred on the holes, which is where it spins
        // stood on edge facing the hands: the sheet's x becomes the vertical, its normal the cord
        let m = chain(&[Ry(PI / 2.0), T(0, 0, -1.5)]);
        // one closed loop: through hole A, round the left hand, back through
        // hole B, round the right hand, and through A again. Each pass is two
        // points, one each side of the 3mm plate, so the cord runs straight
        // through the hole rather than touching it on the way past.
        let (ax, ay) = (holes[0].0 - mx, holes[0].1 - my);
        let (bx, by) = (holes[1].0 - mx, holes[1].1 - my);
        let (a, b) = ((ay, -ax), (by, -bx));
        let hand = |side: i64, p: (f64, f64), q: (f64, f64)| -> Vec<[Num; 3]> {
            vec![
                [F((side * REACH) as f64 * 0.45), F(p.0 * 0.5), F(p.1 * 0.5)],
                [I(side * REACH), F(p.0 * 0.12), F(p.1 * 0.12)],
                [I(side * REACH), F(q.0 * 0.12), F(q.1 * 0.12)],
                [F((side * REACH) as f64 * 0.45), F(q.0 * 0.5), F(q.1 * 0.5)],
            ]
        };
        let pass = |x: i64, p: (f64, f64)| [I(x), F(p.0), F(p.1)];
        let mut lp = vec![pass(3, a), pass(-3, a)];
        lp.extend(hand(-1, a, b));
        lp.extend([pass(-3, b), pass(3, b)]);
        lp.extend(hand(1, b, a));
        lp.push(pass(3, a));
        let lp: Vec<[Num; 3]> = lp.iter().map(|v| v.map(|n| n.round(2))).collect();
        out.push((
            name.to_string(),
            design(vec![
                ("parts", J::A(vec![placed(&d, &m, STILL, "ply")])),
                ("cords", J::A(vec![cord(&lp)])),
                ("axis", nums(&[I(0), I(0), I(0), I(1), I(0), I(0)])),
                ("ex", J::N(I(0))),
                ("view", nums(&[F(-0.9), F(0.3), F(2.6)])),
            ]),
        ));
    }
    out
}

fn main() {
    let path = out_arg(&format!("{HERE}/buzz-disc-3d.html"));
    write(&path, "Buzz disc", &mut build(), "Drag to turn; right-drag, shift-drag or move two fingers to slide the view; scroll or pinch to zoom in on whatever is under the pointer; double-click a part to turn round it, and Reset view to go back. Pick a disc above. Each is its cut file’s own outline at 3 mm, stood on edge with the cord loop through its two centre holes and out to a hand at each side. Wind it up, pull the hands apart, and the cord unwinding spins the disc about it: tick Spin it to see which way.", "buzz-disc/viewer.rs", true);
    println!("  wrote {}", relpath(&path));
}
