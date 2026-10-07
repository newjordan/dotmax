//! Scratch QA tool: rasterize styles to a PPM contact sheet so they can be
//! judged as pictures (dots + colour) rather than as braille text.
//!
//! Run: `cargo run --release --example qa_sheet -- <out.ppm> <theme|theme/style>... [--frames p:t,p:t]`
//!
//! Each argument is a whole theme or one `theme/style`. Every style gets one
//! row of frames (default: 0.15, 0.5 and 0.9 progress), drawn at the site's
//! 44x4-cell size with each braille dot as a round-ish 3x3 pixel blob.

use dotmax::progress::{styles_for_theme, BarContext, Easing, ProgressStyle};
use dotmax::BrailleGrid;
use std::fs;
use std::io::Write;

const W: usize = 44;
const H: usize = 4;
const DOT: usize = 4; // pixel pitch per dot
const GAP: usize = 12; // pixels between frames
const LABEL_H: usize = 0;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args
        .first()
        .expect("usage: qa_sheet <out.ppm> <theme|theme/style>...");
    let mut frames: Vec<(f32, f32)> = vec![(0.15, 0.6), (0.5, 2.0), (0.9, 3.4)];
    let mut picks: Vec<Box<dyn ProgressStyle>> = Vec::new();
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        if a == "--frames" {
            let spec = it.next().expect("--frames p:t,p:t");
            frames = spec
                .split(',')
                .map(|pt| {
                    let (p, t) = pt.split_once(':').expect("p:t");
                    (p.parse().unwrap(), t.parse().unwrap())
                })
                .collect();
            continue;
        }
        let (theme, style) = a
            .split_once('/')
            .map_or((a.as_str(), None), |(t, s)| (t, Some(s)));
        for s in styles_for_theme(theme) {
            if style.map_or(true, |n| n == s.name()) {
                picks.push(s);
            }
        }
    }
    assert!(!picks.is_empty(), "no styles matched");

    let fw = W * 2 * DOT;
    let fh = H * 4 * DOT;
    let img_w = frames.len() * (fw + GAP) + GAP;
    let row_h = fh + GAP + LABEL_H;
    let img_h = picks.len() * row_h + GAP;
    let mut px = vec![[14u8, 14, 18]; img_w * img_h];

    for (ri, style) in picks.iter().enumerate() {
        eprintln!("row {ri}: {}/{}", style.theme(), style.name());
        for (fi, &(p, t)) in frames.iter().enumerate() {
            let mut g = BrailleGrid::new(W, H).unwrap();
            g.enable_color_support();
            let ctx = BarContext::new(p, t, W, H)
                .with_easing(Easing::CubicInOut)
                .with_label(format!("{:.0}%", p * 100.0));
            style.render(&mut g, &ctx).unwrap();
            let ox = GAP + fi * (fw + GAP);
            let oy = GAP + ri * row_h;
            // frame backdrop
            for y in 0..fh {
                for x in 0..fw {
                    px[(oy + y) * img_w + ox + x] = [24, 24, 30];
                }
            }
            for cy in 0..H {
                for cx in 0..W {
                    let col = g
                        .get_color(cx, cy)
                        .map_or([225, 225, 225], |c| [c.r, c.g, c.b]);
                    for di in 0..8u8 {
                        if !g.get_dot(cx, cy, di).unwrap_or(false) {
                            continue;
                        }
                        // braille dot index -> (col, row) in the 2x4 cell
                        let (dx, dy) = match di {
                            0 => (0, 0),
                            1 => (0, 1),
                            2 => (0, 2),
                            3 => (1, 0),
                            4 => (1, 1),
                            5 => (1, 2),
                            6 => (0, 3),
                            _ => (1, 3),
                        };
                        let bx = ox + (cx * 2 + dx) * DOT;
                        let by = oy + (cy * 4 + dy) * DOT;
                        for y in 0..DOT - 1 {
                            for x in 0..DOT - 1 {
                                px[(by + y) * img_w + bx + x] = col;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut f = fs::File::create(out).unwrap();
    write!(f, "P6\n{img_w} {img_h}\n255\n").unwrap();
    let bytes: Vec<u8> = px.iter().flat_map(|p| p.iter().copied()).collect();
    f.write_all(&bytes).unwrap();
}
