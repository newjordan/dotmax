//! Fractal-themed progress bars — hard mathematics rendered in braille dots.
//!
//! Each style implements a distinct fractal or iterated-function-system formula,
//! mapping `ctx.eased` to a reveal / iteration parameter and `ctx.time` to
//! continuous animation (zoom, rotation, c-parameter drift). All coordinate
//! arithmetic is done in `f32`/`i32` and committed via `draw::dot_i` so that
//! negative values from fractal math never cause panics at small grid sizes.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic hash (no external crates)
// ---------------------------------------------------------------------------

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `fractal` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per style, in display order. Each
/// style maps `ctx.eased` to a mathematical reveal parameter (iteration depth,
/// point count, recursion level) and `ctx.time` to continuous animation.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(MandelbrotEscape),
        Box::new(JuliaSet),
        Box::new(SierpinskiTriangle),
        Box::new(KochCurve),
        Box::new(BarnsleyFern),
        Box::new(DragonCurve),
        Box::new(SierpinskiCarpet),
        Box::new(BurningShip),
        Box::new(PythagorasTree),
        Box::new(NewtonFractal),
        Box::new(CantorDust),
        Box::new(LyapunovBar),
    ]
}

// ---------------------------------------------------------------------------
// 1. Mandelbrot escape-time
// ---------------------------------------------------------------------------

struct MandelbrotEscape;
impl ProgressStyle for MandelbrotEscape {
    fn name(&self) -> &str {
        "mandelbrot-escape"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "The Mandelbrot set's cardioid-and-bulb silhouette at the bar's centre, with its \
         escape-time equipotential lines spreading outward to fill the bar as progress rises \
         and rippling with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Uniform scale: the set's body (|Im| ≤ 1.05) fits the bar height.
        let s = (dh as f32 / 2.1).max(1.0);
        let (ox, oy) = (dw as f32 * 0.5, dh as f32 * 0.5);
        let (cw, ch) = grid.dimensions();
        let mut inside = vec![0u8; cw * ch];
        // Smooth escape count μ per dot; NaN marks the set's interior.
        let mut field = vec![f32::NAN; dw * dh];

        for py in 0..dh {
            for px in 0..dw {
                let cr = -0.55 + (px as f32 + 0.5 - ox) / s;
                let ci = (py as f32 + 0.5 - oy) / s;
                let (mut zr, mut zi) = (0.0f32, 0.0f32);
                let mut n = 0u32;
                while n < 40 && zr * zr + zi * zi <= 256.0 {
                    let t = zr * zr - zi * zi + cr;
                    zi = 2.0 * zr * zi + ci;
                    zr = t;
                    n += 1;
                }
                if n >= 40 {
                    draw::dot(grid, px, py);
                    inside[(py / 4) * cw + px / 2] += 1;
                    continue;
                }
                let lz = (zr * zr + zi * zi).max(1.0001).ln() * 0.5;
                let mu = n as f32 + 1.0 - lz.max(1e-4).ln() / std::f32::consts::LN_2;
                field[py * dw + px] = mu;
            }
        }

        // Equipotential lines = where the banded μ steps between neighbours.
        // They spread from the set (high μ) outward (low μ) as progress rises.
        let mu_min = 7.0 * (1.0 - ctx.eased) - 0.5;
        let band = |mu: f32| ((mu + 1.0).ln() * 4.5 - ctx.time * 0.8).floor() as i32;
        for py in 0..dh {
            for px in 0..dw {
                let mu = field[py * dw + px];
                if mu.is_nan() || mu < mu_min {
                    continue;
                }
                let b = band(mu);
                let right = if px + 1 < dw {
                    field[py * dw + px + 1]
                } else {
                    mu
                };
                let down = if py + 1 < dh {
                    field[(py + 1) * dw + px]
                } else {
                    mu
                };
                let edge = |m: f32| !m.is_nan() && band(m) != b;
                if edge(right) || edge(down) {
                    draw::dot(grid, px, py);
                }
            }
        }

        // One colour per cell: the set itself glows, equipotentials fade outward.
        grid.enable_color_support();
        let half = cw as f32 * 0.5;
        for cy in 0..ch {
            for cx in 0..cw {
                let c = if inside[cy * cw + cx] >= 3 {
                    ctx.palette.start
                } else {
                    let d = ((cx as f32 + 0.5 - half).abs() / half.max(1.0)).min(1.0);
                    ctx.palette.sample(0.25 + d * 0.75)
                };
                let _ = grid.set_cell_color(cx, cy, c);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Julia sets  z → z² + c — basilica, rabbit, Siegel disk
// ---------------------------------------------------------------------------

struct JuliaSet;
impl ProgressStyle for JuliaSet {
    fn name(&self) -> &str {
        "julia-set"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Three filled Julia sets side by side — the basilica (c=−1), Douady's rabbit and the \
         spiralling Siegel disk — each sharpening from a round blob into its true shape as \
         progress raises its iteration depth; c trembles gently with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        // (c, half-width, half-height) of each set's bounding box.
        const SETS: [(f32, f32, f32, f32); 3] = [
            (-1.0, 0.0, 1.62, 0.78),
            (-0.123, 0.745, 1.25, 1.2),
            (-0.391, -0.587, 1.25, 1.2),
        ];
        let n = if dw >= dh * 4 { 3 } else { 1 };
        let pw = dw as f32 / n as f32;
        let wob = 0.008 * (ctx.time * 1.3).sin();
        let (cw, ch) = grid.dimensions();
        for (k, &(cr, ci, hw, hh)) in SETS.iter().take(n).enumerate() {
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let s = ((pw - 2.0) / (2.0 * hw)).min(dh as f32 / (2.0 * hh));
            let ox = pw * (k as f32 + 0.5);
            let oy = dh as f32 * 0.5;
            if local <= 0.0 {
                // Waiting panel: just its centre point, c's critical orbit start.
                draw::dot_i(grid, ox as i32, oy as i32);
                continue;
            }
            // Shallow depths give a blob; by ~30 iterations the set has its true outline.
            let max_iter = (2.0 + local * 30.0) as u32;
            let (cr, ci) = (cr + wob, ci + wob * 0.5);
            let x_lo = (ox - hw * s).max(0.0) as usize;
            let x_hi = ((ox + hw * s) as usize).min(dw);
            for py in 0..dh {
                for px in x_lo..x_hi {
                    let mut zr = (px as f32 + 0.5 - ox) / s;
                    let mut zi = (py as f32 + 0.5 - oy) / s;
                    let mut it = 0u32;
                    while it < max_iter && zr * zr + zi * zi <= 4.0 {
                        let t = zr * zr - zi * zi + cr;
                        zi = 2.0 * zr * zi + ci;
                        zr = t;
                        it += 1;
                    }
                    if it >= max_iter {
                        draw::dot(grid, px, py);
                    }
                }
            }
            // Panel colour: finished sets take the palette's end, growing ones its start.
            let c = ctx.palette.sample(0.2 + local * 0.8);
            let c0 = (pw * k as f32 / 2.0) as usize;
            let c1 = ((pw * (k + 1) as f32 / 2.0) as usize).min(cw);
            for cy in 0..ch {
                for cx in c0..c1 {
                    draw::tint_row(grid, cy, cx, cx, c);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Sierpinski triangle — Pascal's triangle mod 2
// ---------------------------------------------------------------------------

struct SierpinskiTriangle;
impl ProgressStyle for SierpinskiTriangle {
    fn name(&self) -> &str {
        "sierpinski-triangle"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "A row of Sierpinski triangles built from Pascal's triangle mod 2: each starts as a \
         solid triangle and subdivides level by level, left to right, as progress rises; a \
         chaos-game walker hops between vertices with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let rows = dh as i32;
        // Levels available at this height: 16 rows → 4 halvings.
        let dmax = 32 - (dh.max(2) as u32 - 1).leading_zeros();
        let base = 2 * rows - 1;
        let n = ((dw as i32 + base / 2) / base.max(1)).max(1);
        let pitch = dw as f32 / n as f32;
        let q = ctx.eased * n as f32;

        for i in 0..n {
            let apex = ((i as f32 + 0.5) * pitch) as i32;
            let local = (q - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                // Not started yet: just the outline of the triangle to come.
                for r in 0..rows {
                    draw::dot_i(grid, apex - r, r);
                    draw::dot_i(grid, apex + r, r);
                }
                for k in 0..rows {
                    draw::dot_i(grid, apex - rows + 1 + 2 * k, rows - 1);
                }
                continue;
            }
            // Depth 0 = solid triangle, dmax = full Pascal-mod-2 detail.
            let depth = ((local * (dmax as f32 + 0.999)) as u32).min(dmax);
            let sh = dmax - depth;
            for r in 0..rows {
                for k in 0..=r {
                    let (kb, rb) = ((k as u32) >> sh, (r as u32) >> sh);
                    if kb & (rb - kb) == 0 {
                        draw::dot_i(grid, apex - r + 2 * k, r);
                    }
                }
            }
        }

        // Chaos-game walker: midpoint hops toward random vertices of the active triangle.
        let active = (q.floor() as i32).clamp(0, n - 1);
        let apex = (active as f32 + 0.5) * pitch;
        let verts = [
            (apex, 0.0),
            (apex - (rows - 1) as f32, (rows - 1) as f32),
            (apex + (rows - 1) as f32, (rows - 1) as f32),
        ];
        let seed = (ctx.time * 10.0).max(0.0) as u32;
        let (mut wx, mut wy) = (apex, rows as f32 * 0.6);
        for j in 0..12u32 {
            let v = verts[(hash(seed.wrapping_add(j)) % 3) as usize];
            wx = (wx + v.0) * 0.5;
            wy = (wy + v.1) * 0.5;
        }

        grid.enable_color_support();
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            for cx in 0..cw {
                let x = cx as f32 * 2.0 + 1.0;
                let tri = ((x / pitch).floor() as i32).clamp(0, n - 1);
                let done = q - tri as f32;
                let c = if done <= 0.0 {
                    ctx.palette.track
                } else {
                    ctx.palette
                        .sample((cy as f32 / ch.max(1) as f32 + tri as f32 * 0.3).fract())
                };
                let _ = grid.set_cell_color(cx, cy, c);
            }
        }
        draw::dot_i(grid, wx as i32, wy as i32);
        draw::dot_i(grid, wx as i32 + 1, wy as i32);
        let _ = grid.set_cell_color(
            (wx.max(0.0) as usize) / 2,
            (wy.max(0.0) as usize) / 4,
            crate::Color::rgb(255, 255, 255),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Koch curve — recursive middle-third bumps along the bar
// ---------------------------------------------------------------------------

struct KochCurve;

/// Koch curve vertices from `a` to `b` at recursion `depth`, bump on the
/// screen-up side. Appends every vertex after `a`.
fn koch_points(a: (f32, f32), b: (f32, f32), depth: u32, pts: &mut Vec<(f32, f32)>) {
    if depth == 0 {
        pts.push(b);
        return;
    }
    let (dx, dy) = ((b.0 - a.0) / 3.0, (b.1 - a.1) / 3.0);
    let p1 = (a.0 + dx, a.1 + dy);
    let p3 = (a.0 + 2.0 * dx, a.1 + 2.0 * dy);
    // Rotate the middle third by −60° (screen y grows downward → bump goes up).
    let p2 = (
        p1.0 + dx * 0.5 + dy * 0.866_025,
        p1.1 + dy * 0.5 - dx * 0.866_025,
    );
    koch_points(a, p1, depth - 1, pts);
    koch_points(p1, p2, depth - 1, pts);
    koch_points(p2, p3, depth - 1, pts);
    koch_points(p3, b, depth - 1, pts);
}

impl ProgressStyle for KochCurve {
    fn name(&self) -> &str {
        "koch-curve"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Koch curve laid along the bar: each middle third grows a triangular bump, again and \
         again — progress rewrites the line left to right at each finer level (straight line → \
         one bump → 4 → 16 → 64), and a spark runs along the finished curve with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh == 0 {
            return Ok(());
        }
        // Each Koch span is √3/6 ≈ 0.29 as tall as it is long; use as many
        // side-by-side spans as needed to fit the bar's height.
        let max_len = ((dh as f32 - 1.0) / 0.289).max(2.0);
        let spans = ((dw as f32 / max_len).ceil() as usize).clamp(1, 16);
        let span = (dw as f32 - 1.0) / spans as f32;
        let base = dh as f32 - 1.0;

        let levels = 5.0f32;
        let stage = (ctx.eased.clamp(0.0, 1.0) * levels).min(levels - 1e-3);
        let depth = stage as u32;
        let split = stage.fract() * (dw as f32);

        let mut old = Vec::new();
        let mut new = Vec::new();
        for s in 0..spans {
            let a = (s as f32 * span, base);
            let b = ((s + 1) as f32 * span, base);
            old.push(a);
            koch_points(a, b, depth, &mut old);
            new.push(a);
            koch_points(a, b, (depth + 1).min(5), &mut new);
        }
        // Spark: a short gap that travels along the curve with time.
        let spark = ((ctx.time.max(0.0) * 0.25).fract() * dw as f32) as i32;
        let mut seg = |p: (f32, f32), q: (f32, f32)| {
            let steps = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as i32).clamp(1, 256);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let x = (p.0 + (q.0 - p.0) * t).round() as i32;
                let y = (p.1 + (q.1 - p.1) * t).round() as i32;
                if (x - spark).abs() > 1 {
                    draw::dot_i(grid, x, y);
                }
            }
        };
        for w in new.windows(2) {
            if w[0].0.max(w[1].0) <= split {
                seg(w[0], w[1]);
            }
        }
        for w in old.windows(2) {
            if w[0].0.min(w[1].0) >= split {
                seg(w[0], w[1]);
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy_cell in 0..ch {
            let t = cy_cell as f32 / ch.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                cw.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Barnsley fern — IFS with 4 affine maps
// ---------------------------------------------------------------------------

struct BarnsleyFern;
impl ProgressStyle for BarnsleyFern {
    fn name(&self) -> &str {
        "barnsley-fern"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Barnsley's four-map IFS fern laid along the bar: the frond unfurls from its stem \
         toward the tip as progress rises, with alternating leaflets, and sways with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        // Fern spans x ∈ [-2.2, 2.7], y ∈ [0, 10]; lay its axis (y) along the bar.
        let reach = 0.08 + ctx.eased * 0.92;
        let sway = (ctx.time * 0.9).sin() * 0.35;
        let mut hits = vec![0u8; dw * dh];
        let (mut x, mut y) = (0.0f32, 0.0f32);
        let count = 8_000u32;
        for i in 0..count {
            let r = hash(i.wrapping_mul(2_654_435_761) ^ 0x5bd1) % 100;
            (x, y) = match r {
                0 => (0.0, 0.16 * y),
                1..=85 => (0.85 * x + 0.04 * y, -0.04 * x + 0.85 * y + 1.6),
                86..=92 => (0.2 * x - 0.26 * y, 0.23 * x + 0.22 * y + 1.6),
                _ => (-0.15 * x + 0.28 * y, 0.26 * x + 0.24 * y + 0.44),
            };
            let along = y / 10.0;
            if i < 20 || along > reach {
                continue;
            }
            // Bend grows toward the tip so the frond sways, not slides.
            let bend = x + sway * along * along * 4.0;
            let px = (along * (dw - 1) as f32) as usize;
            let py = ((bend + 2.4) / 5.4 * (dh - 1) as f32).round();
            if py >= 0.0 && (py as usize) < dh && px < dw {
                let h = &mut hits[py as usize * dw + px];
                *h = h.saturating_add(1);
            }
        }
        // Threshold per column against that column's peak: the tip collects far
        // more points than the base, and this keeps leaflets distinct along the frond.
        for px in 0..dw {
            let peak = (0..dh).map(|py| hits[py * dw + px]).max().unwrap_or(0);
            let thr = (u16::from(peak) * 3 / 10).max(1);
            for py in 0..dh {
                if u16::from(hits[py * dw + px]) >= thr {
                    draw::dot(grid, px, py);
                }
            }
        }
        // Stem-to-tip green-ish gradient from the palette.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Dragon curve — paper-folding L-system
// ---------------------------------------------------------------------------

struct DragonCurve;

/// Straight dot line between two signed dot coordinates (bounds-safe).
fn line_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 512);
    for i in 0..=n {
        draw::dot_i(grid, x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n);
    }
}

impl ProgressStyle for DragonCurve {
    fn name(&self) -> &str {
        "dragon-curve"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Heighway dragon by paper folding: four panels show the strip folded 2, 4, 6 and 8 times, \
         each drawing itself in turn as progress rises; a walker retraces finished dragons with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        const ORDERS: [u32; 4] = [2, 4, 6, 8];
        let n = if dw >= dh * 4 { 4 } else { 1 };
        let pw = dw as f32 / n as f32;

        for k in 0..n {
            let order = ORDERS[(k + 4 - n) % 4];
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let segs = 1u32 << order;
            // Walk the fold sequence: turn left/right by the bit above the
            // lowest set bit of the 1-based segment index.
            let mut pts: Vec<(f32, f32)> = Vec::with_capacity(segs as usize + 1);
            let (mut x, mut y, mut dir) = (0i32, 0i32, 0i32);
            pts.push((0.0, 0.0));
            for i in 1..=segs {
                let (dx, dy) = [(1, 0), (0, -1), (-1, 0), (0, 1)][dir as usize];
                x += dx;
                y += dy;
                pts.push((x as f32, y as f32));
                let right = ((i >> i.trailing_zeros()) >> 1) & 1 == 0;
                dir = (dir + if right { 1 } else { 3 }) & 3;
            }
            // Centre on the bounding box and fit its radius to the panel.
            let (mut lx, mut hx, mut ly, mut hy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for &(px, py) in &pts {
                lx = lx.min(px);
                hx = hx.max(px);
                ly = ly.min(py);
                hy = hy.max(py);
            }
            let (mx, my) = ((lx + hx) * 0.5, (ly + hy) * 0.5);
            // Axis-aligned, uniform scale: right angles stay crisp in dots.
            let s = ((pw - 3.0) / (hx - lx).max(1.0)).min((dh as f32 - 1.0) / (hy - ly).max(1.0));
            let (ox, oy) = (pw * (k as f32 + 0.5), (dh as f32 - 1.0) * 0.5);
            let to_dot = |(px, py): (f32, f32)| {
                (
                    (ox + (px - mx) * s).round() as i32,
                    (oy + (py - my) * s).round() as i32,
                )
            };
            let shown = (local * segs as f32).ceil() as usize;
            if shown == 0 {
                // Waiting panel: the unfolded strip — a single short segment.
                let (a, b) = (to_dot((mx - 1.0, my)), to_dot((mx + 1.0, my)));
                line_i(grid, a.0, a.1, b.0, b.1);
                continue;
            }
            for w in pts.windows(2).take(shown) {
                let (a, b) = (to_dot(w[0]), to_dot(w[1]));
                line_i(grid, a.0, a.1, b.0, b.1);
            }
            // A walker retraces each finished dragon with time.
            if shown >= segs as usize {
                let at = to_dot(pts[(ctx.time.max(0.0) * 9.0) as usize % pts.len()]);
                draw::fill_rect(grid, at.0.max(0) as usize, at.1.max(0) as usize, 2, 2);
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Sierpinski carpet — recursive square removal
// ---------------------------------------------------------------------------

struct SierpinskiCarpet;
impl ProgressStyle for SierpinskiCarpet {
    fn name(&self) -> &str {
        "sierpinski-carpet"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "A row of square Sierpinski carpets: each starts solid, has its centre ninth punched \
         out, then the centre of every remaining ninth, filling in turn as progress rises while \
         a glint sweeps across them with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Square tiles as tall as the bar, separated by a two-dot gap.
        let side = dh;
        let n = ((dw + 2) / (side + 2)).max(1);
        let x_left = dw.saturating_sub(n * (side + 2) - 2) / 2;
        for k in 0..n {
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let x0 = x_left + k * (side + 2);
            if local <= 0.0 {
                // Waiting tile: only its outline.
                draw::rect_outline(grid, x0, 0, side.min(dw), side);
                continue;
            }
            // Depth 0 = solid, 1 = centre removed, 2 = full level-2 carpet.
            let depth = ((local * 2.999) as u32).min(2);
            for py in 0..side {
                for px in 0..side {
                    // Map dots onto the 9×9 level-2 grid.
                    let (mut u, mut v) = (px * 9 / side, py * 9 / side);
                    let (mut hole, mut level) = (false, 0);
                    // Most significant base-3 digit first.
                    let mut scale = 3;
                    while level < depth {
                        if (u / scale) % 3 == 1 && (v / scale) % 3 == 1 {
                            hole = true;
                            break;
                        }
                        u %= scale;
                        v %= scale;
                        scale /= 3;
                        level += 1;
                        if scale == 0 {
                            break;
                        }
                    }
                    if !hole {
                        draw::dot(grid, x0 + px, py);
                    }
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        let glint = (ctx.time * 0.25).fract();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let near = 1.0 - ((t - glint).abs() * 8.0).min(1.0);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t * 0.6 + near * 0.4));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Burning Ship fractal  — |Re(z)|, |Im(z)| before squaring, zoomed on the ship
// ---------------------------------------------------------------------------

struct BurningShip;
impl ProgressStyle for BurningShip {
    fn name(&self) -> &str {
        "burning-ship"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Burning Ship fractal (z → (|Re z|+i|Im z|)² + c), zoomed onto the little ship near \
         c ≈ −1.76 that gives the set its name: hull on the waterline, masts and rigging \
         rising above — a convoy of them sails in as progress rises, flames flickering with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // The ship window, masts pointing up (−Im at the top).
        let (re0, re1) = (-1.802f32, -1.698f32);
        let (im0, im1) = (-0.088f32, 0.012f32);
        // Tile width keeps the ship roughly in proportion (stretched ≈1.4×).
        let tile = ((dh as f32 * 1.45).round() as usize).clamp(4, dw.max(4));
        let tiles = (dw / tile).max(1);
        let left = (dw - (tiles * tile).min(dw)) / 2;
        let shown = ctx.eased.clamp(0.0, 1.0) * tiles as f32;
        let max_iter = 60u32;
        // Flicker: the "hot" threshold breathes with time.
        let thresh = 20 + ((ctx.time * 5.0).sin() * 2.0).round() as i32;

        for k in 0..tiles {
            let local = (shown - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                // Waiting slot: a dotted waterline.
                let y = (dh as f32 * (0.0 - im0) / (im1 - im0)).round() as usize;
                for x in (left + k * tile..left + (k + 1) * tile).step_by(3) {
                    draw::dot(grid, x, y.min(dh - 1));
                }
                continue;
            }
            // The ship rises out of the water from the waterline up.
            let rise = (1.0 - local) * dh as f32;
            for py in 0..dh {
                if (py as f32) < rise {
                    continue;
                }
                let ci = im0 + (py as f32 + 0.5) / dh as f32 * (im1 - im0);
                for px in 0..tile {
                    let cr = re0 + (px as f32 + 0.5) / tile as f32 * (re1 - re0);
                    let (mut zr, mut zi) = (0.0f32, 0.0f32);
                    let mut n = 0u32;
                    while n < max_iter {
                        let (zr2, zi2) = (zr * zr, zi * zi);
                        if zr2 + zi2 > 4.0 {
                            break;
                        }
                        let nzr = zr2 - zi2 + cr;
                        zi = 2.0 * zr.abs() * zi.abs() + ci;
                        zr = nzr;
                        n += 1;
                    }
                    if n as i32 >= thresh {
                        draw::dot(grid, left + k * tile + px, py);
                    }
                }
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Pythagoras trees — recursive squares on right triangles, a row of them
// ---------------------------------------------------------------------------

struct PythagorasTree;

/// One Pythagoras-tree square on base edge `a→b` (growing to screen-up),
/// then its two children on the top edge split at angle `alpha`.
fn pyth_tree(
    grid: &mut BrailleGrid,
    a: (f32, f32),
    b: (f32, f32),
    alpha: f32,
    depth: u32,
    budget: &mut u32,
) {
    if depth == 0 || *budget == 0 {
        return;
    }
    *budget -= 1;
    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
    let side = (ex * ex + ey * ey).sqrt();
    // Top corners: perpendicular (ey, −ex) points up on screen.
    let c = (b.0 + ey, b.1 - ex);
    let d = (a.0 + ey, a.1 - ex);
    if side < 2.2 {
        // Too small to outline: fill it so the canopy reads as foliage.
        let (lo_x, hi_x) = (
            a.0.min(b.0).min(c.0).min(d.0),
            a.0.max(b.0).max(c.0).max(d.0),
        );
        let (lo_y, hi_y) = (
            a.1.min(b.1).min(c.1).min(d.1),
            a.1.max(b.1).max(c.1).max(d.1),
        );
        let (cx, cy) = ((lo_x + hi_x) * 0.5, (lo_y + hi_y) * 0.5);
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
    } else {
        for (p, q) in [(a, b), (b, c), (c, d), (d, a)] {
            let n = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as i32).clamp(1, 128);
            for i in 0..=n {
                let t = i as f32 / n as f32;
                draw::dot_i(
                    grid,
                    (p.0 + (q.0 - p.0) * t).round() as i32,
                    (p.1 + (q.1 - p.1) * t).round() as i32,
                );
            }
        }
    }
    // Apex of the right triangle sitting on the top edge d→c.
    let (ux, uy) = ((c.0 - d.0) / side.max(1e-6), (c.1 - d.1) / side.max(1e-6));
    let l = side * alpha.cos();
    let (ca, sa) = (alpha.cos(), alpha.sin());
    let apex = (d.0 + (ux * ca + uy * sa) * l, d.1 + (uy * ca - ux * sa) * l);
    pyth_tree(grid, d, apex, alpha, depth - 1, budget);
    pyth_tree(grid, apex, c, alpha, depth - 1, budget);
}

impl ProgressStyle for PythagorasTree {
    fn name(&self) -> &str {
        "pythagoras-tree"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Pythagoras trees: a trunk square carries a right triangle whose two legs carry two \
         smaller squares, and so on — a row of trees each branching level by level as progress \
         reaches it, their crowns swaying gently with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // A 45° tree fits a 6s × 4s box; size the trunk from the height.
        let side = ((dh as f32 - 1.0) / 4.0).max(1.0);
        let slot = (side * 6.4).max(2.0);
        let trees = ((dw as f32 / slot) as usize).clamp(1, 12);
        let pitch = dw as f32 / trees as f32;
        let total = ctx.eased.clamp(0.0, 1.0) * trees as f32;
        let max_depth = 7.0f32;
        for k in 0..trees {
            let local = (total - k as f32).clamp(0.0, 1.0);
            let depth = (local * max_depth).ceil() as u32;
            let cx = pitch * (k as f32 + 0.5);
            let base = dh as f32 - 1.0;
            if depth == 0 {
                // Seed: a dot where the trunk will stand.
                draw::dot_i(grid, cx.round() as i32, base as i32);
                continue;
            }
            let sway = 0.12 * (ctx.time * 1.1 + k as f32 * 1.7).sin();
            let alpha = PI / 4.0 + sway;
            let mut budget = 400u32;
            pyth_tree(
                grid,
                (cx - side * 0.5, base),
                (cx + side * 0.5, base),
                alpha,
                depth,
                &mut budget,
            );
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.max(1) as f32; // brighter at top (crown)
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Newton fractal — basins of attraction for z³ - 1 = 0
// ---------------------------------------------------------------------------

struct NewtonFractal;
impl ProgressStyle for NewtonFractal {
    fn name(&self) -> &str {
        "newton-fractal"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Newton's method on z³−1: each cell is coloured by the cube root it converges to and \
         dotted in convergence-speed bands, so the three basins meet in chains of beads; more \
         of the plane resolves as progress raises the step budget, and the plane turns with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        const ROOTS: [(f32, f32); 3] = [(1.0, 0.0), (-0.5, 0.866_025_4), (-0.5, -0.866_025_4)];
        let max_iter = (2.0 + ctx.eased * 18.0) as u32;
        let s = (dh as f32 / 2.6).max(0.5);
        let (ox, oy) = (dw as f32 * 0.5, dh as f32 * 0.5);
        let rot = ctx.time * 0.15;
        let (cr, sr) = (rot.cos(), rot.sin());
        let (cw, ch) = grid.dimensions();
        // Per-cell vote of which root its dots reach.
        let mut votes = vec![[0u8; 3]; cw * ch];

        for py in 0..dh {
            for px in 0..dw {
                let (u, v) = ((px as f32 + 0.5 - ox) / s, (py as f32 + 0.5 - oy) / s);
                let (mut zr, mut zi) = (u * cr - v * sr, u * sr + v * cr);
                let mut hit = None;
                for it in 0..max_iter {
                    // z ← z − (z³ − 1) / (3z²)
                    let (z2r, z2i) = (zr * zr - zi * zi, 2.0 * zr * zi);
                    let (z3r, z3i) = (z2r * zr - z2i * zi, z2r * zi + z2i * zr);
                    let (nr, ni) = (z3r - 1.0, z3i);
                    let (dr, di) = (3.0 * z2r, 3.0 * z2i);
                    let den = (dr * dr + di * di).max(1e-9);
                    zr -= (nr * dr + ni * di) / den;
                    zi -= (ni * dr - nr * di) / den;
                    if let Some(r) = ROOTS
                        .iter()
                        .position(|&(rr, ri)| (zr - rr).powi(2) + (zi - ri).powi(2) < 0.01)
                    {
                        hit = Some((r, it));
                        break;
                    }
                }
                if let Some((r, it)) = hit {
                    let cell = &mut votes[(py / 4) * cw + px / 2];
                    cell[r] = cell[r].saturating_add(1);
                    // Banded by step count: the fast-converging basin cores stay solid.
                    if it < 2 || it % 2 == 0 {
                        draw::dot(grid, px, py);
                    }
                }
            }
        }

        grid.enable_color_support();
        let shades = [
            ctx.palette.start,
            crate::Color::rgb(235, 235, 245),
            ctx.palette.end,
        ];
        for cy in 0..ch {
            for cx in 0..cw {
                let v = votes[cy * cw + cx];
                let best = (0..3).max_by_key(|&i| v[i]).unwrap_or(0);
                let c = if v[best] == 0 {
                    ctx.palette.track
                } else {
                    shades[best]
                };
                let _ = grid.set_cell_color(cx, cy, c);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Cantor dust — iterated middle-third removal cascade
// ---------------------------------------------------------------------------

struct CantorDust;
impl ProgressStyle for CantorDust {
    fn name(&self) -> &str {
        "cantor-dust"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "The Cantor set as its textbook ladder: a full bar, then thirds, ninths, 27ths and 81sts \
         stacked beneath it, each level drawing in as progress rises while a highlight sweeps \
         the surviving dust with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Widest power of three that fits, so every removed third is whole dots.
        let mut width = 1usize;
        let mut depth = 0u32;
        while width * 3 <= dw && depth < 4 {
            width *= 3;
            depth += 1;
        }
        let levels = depth as usize + 1;
        let rows = (dh / levels).max(1);
        let bar = rows.saturating_sub(1).max(1);
        let x0 = (dw - width) / 2;
        let shown = ctx.eased * levels as f32;
        for lvl in 0..levels {
            let reveal = (shown - lvl as f32).clamp(0.0, 1.0);
            if reveal <= 0.0 {
                break;
            }
            let unit = width / 3usize.pow(lvl as u32);
            let cols = (reveal * width as f32) as usize;
            for x in 0..cols {
                // Keep x only if none of its first `lvl` base-3 digits is 1.
                let mut u = x / unit.max(1);
                let mut keep = true;
                for _ in 0..lvl {
                    if u % 3 == 1 {
                        keep = false;
                        break;
                    }
                    u /= 3;
                }
                if keep {
                    for r in 0..bar {
                        draw::dot(grid, x0 + x, lvl * rows + r);
                    }
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        let sweep = (ctx.time * 0.3).fract();
        for cy in 0..ch {
            for cx in 0..cw {
                let t = cx as f32 / cw.max(1) as f32;
                let near = 1.0 - ((t - sweep).abs() * 6.0).min(1.0);
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t * 0.7 + near * 0.3));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Lyapunov bar — Markus–Lyapunov fractal for the sequence AB
// ---------------------------------------------------------------------------

struct LyapunovBar;
impl ProgressStyle for LyapunovBar {
    fn name(&self) -> &str {
        "lyapunov-bar"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Markus–Lyapunov fractal for the logistic map driven by the sequence AB (a across, b up, \
         both in [3, 4]): stable regions with negative exponent are lit, forming the classic \
         swallow-tail shapes, swept in left to right by progress as the window drifts with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cols = (ctx.eased * dw as f32).ceil() as usize;
        let drift = 0.08 * (ctx.time * 0.4).sin();
        let (cw, ch) = grid.dimensions();
        let mut stable = vec![0u8; cw * ch];
        for px in 0..cols.min(dw) {
            let a = 3.0 + (px as f32 + 0.5) / dw as f32;
            for py in 0..dh {
                let b = 4.0 - (py as f32 + 0.5) / dh as f32 + drift;
                let mut x = 0.5f32;
                let mut lam = 0.0f32;
                for n in 0..60u32 {
                    let r = if n % 2 == 0 { a } else { b };
                    x = r * x * (1.0 - x);
                    if n >= 20 {
                        lam += (r * (1.0 - 2.0 * x)).abs().max(1e-6).ln();
                    }
                }
                if lam < 0.0 {
                    draw::dot(grid, px, py);
                    stable[(py / 4) * cw + px / 2] += 1;
                }
            }
        }
        // Stable cells glow; the chaotic sea between them stays dark.
        for cy in 0..ch {
            for cx in 0..cw {
                let t = (stable[cy * cw + cx] as f32 / 8.0).min(1.0);
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(1.0 - t));
            }
        }
        Ok(())
    }
}
