//! Yantra / Mandala sacred-geometry progress styles.
//!
//! Eleven structurally distinct radially symmetric styles drawn entirely with
//! braille dots via `draw::dot_i` and the Bresenham line helper defined below.
//! Each style has a unique construction principle:
//!
//! - `sri-yantra`            — interlocking Shiva/Shakti triangles in a circle within the bhupura gate
//! - `lotus-mandala`         — eight outer and eight inner pointed petals around a seed pod
//! - `rose-window`           — a row of Gothic roses: rim, hub, lancet mullions, foiled oculi
//! - `bagua`                 — eight trigrams in Fu Xi order either side of a turning yin-yang
//! - `enneagram`             — 9-point star (1-4-2-8-5-7 cycle + triangle)
//! - `rangoli`               — a row of kolams: pulli-dot diamonds looped by a travelling pen
//! - `chartres-labyrinth`    — floor labyrinth in perspective, a walker tracing its circuits
//! - `mandala-tessellation`  — edge-sharing hexagonal mandala tiles filled tile by tile
//! - `dharma-wheel`          — 8-spoked knobbed wheel rolling along its path
//! - `vesica-rosette`        — a row of 6-petal compass rosettes cut by vesica arcs
//! - `star-of-david`         — two interlocking equilateral triangles, inked edge by edge

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — saffron into gold.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 126, 84);
const TINT_END: Color = Color::rgb(255, 204, 64);

/// Sample the theme gradient at `t` in `0.0..=1.0`.
fn sample_tint(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(
        lerp(TINT_START.r, TINT_END.r),
        lerp(TINT_START.g, TINT_END.g),
        lerp(TINT_START.b, TINT_END.b),
    )
}

/// Applies the theme's signature gradient to every cell the inner style drew,
/// drifting slowly with `time`. Styles stay monochrome-safe underneath: drop
/// the wrapper in [`styles`] for uncolored output.
struct Tinted<S>(S);

impl<S: ProgressStyle> ProgressStyle for Tinted<S> {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn theme(&self) -> &str {
        self.0.theme()
    }
    fn describe(&self) -> &str {
        self.0.describe()
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        self.0.render(grid, ctx)?;
        grid.enable_color_support();
        let (w, h) = grid.dimensions();
        for y in 0..h {
            for x in 0..w {
                let ch = grid.get_char(x, y);
                if ch != '\u{2800}' && ch != ' ' {
                    let t = (x as f32 / w.max(1) as f32 + ctx.time * 0.05).fract();
                    let tri = 1.0 - (2.0 * t - 1.0).abs();
                    let _ = grid.set_cell_color(x, y, sample_tint(tri));
                }
            }
        }
        Ok(())
    }
}

/// All styles in the `yantra` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per sacred-geometry style. The eleven
/// styles span distinct construction methods — triangle systems, petal arcs,
/// tracery, trigrams, star polygons, kolam loops, labyrinth paths, rotational
/// tilings, spoked wheels, vesica arcs, and nested hexagrams — ensuring no two
/// styles are structurally alike.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(SriYantra)),
        Box::new(Tinted(LotusMandala)),
        Box::new(Tinted(RoseWindow)),
        Box::new(Tinted(Bagua)),
        Box::new(Tinted(Enneagram)),
        Box::new(Tinted(Rangoli)),
        Box::new(Tinted(ChartresLabyrinth)),
        Box::new(Tinted(MandalaTessellation)),
        Box::new(Tinted(DharmaWheel)),
        Box::new(Tinted(VesicaRosette)),
        Box::new(Tinted(StarOfDavid)),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared geometry helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Grid center in dot-space.
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

/// Convert polar coordinates centered at (cx, cy) to dot-space integers.
#[inline]
fn polar(cx: f32, cy: f32, r: f32, angle: f32) -> (i32, i32) {
    let x = cx + r * angle.cos();
    let y = cy - r * angle.sin(); // screen y-axis is flipped
    (x.round() as i32, y.round() as i32)
}

/// Step-bounded Bresenham line between two signed dot-space points.
/// Out-of-bounds dots are silently discarded by `draw::dot_i`.
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x < x1 { 1 } else { -1 };
    let sy: i32 = if y < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a circular arc from `a_start` to `a_end` (radians, CCW) at radius `r`
/// centered at (cx, cy).  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a_start: f32, a_end: f32) {
    if r < 0.5 {
        return;
    }
    let span = (a_end - a_start).abs();
    let steps = ((r * span).ceil() as usize).clamp(4, 1024);
    let mut prev: Option<(i32, i32)> = None;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = a_start + (a_end - a_start) * t;
        let p = polar(cx, cy, r, a);
        if let Some(q) = prev {
            line(grid, q.0, q.1, p.0, p.1);
        }
        prev = Some(p);
    }
}

/// Draw a full circle at radius `r` centered at (cx, cy).
fn circle(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    arc(grid, cx, cy, r, 0.0, 2.0 * PI);
}

/// Draw a triangle through three dot-space points.
fn triangle(grid: &mut BrailleGrid, ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) {
    line(grid, ax, ay, bx, by);
    line(grid, bx, by, cx, cy);
    line(grid, cx, cy, ax, ay);
}

/// Faint dotted ring: the placeholder for a frieze panel not yet reached.
fn dotted_ring(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let n = ((r * 2.0) as usize).clamp(4, 64);
    for k in 0..n {
        let (x, y) = polar(cx, cy, r, 2.0 * PI * k as f32 / n as f32);
        draw::dot_i(grid, x, y);
    }
}

/// Lays a frieze of full-height panels across the strip, filled left to right
/// with progress (the first already part-drawn at the start). The callback
/// gets `(grid, cx, cy, r, local_reveal, index)`; `local_reveal` is 0 for a
/// panel not yet reached. `wf` is the panel pitch in radii, `gap` extra dots.
fn frieze(
    grid: &mut BrailleGrid,
    eased: f32,
    wf: f32,
    gap: f32,
    mut panel: impl FnMut(&mut BrailleGrid, f32, f32, f32, f32, usize),
) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw < 2 || dh < 2 {
        return;
    }
    let r = (dw.min(dh) as f32 / 2.0 - 0.5).max(1.0);
    let pitch = (r * wf + gap).max(2.0);
    let n = ((dw as f32 / pitch) as usize).clamp(1, 64);
    let off = (dw as f32 - n as f32 * pitch) / 2.0;
    let cy = dh as f32 / 2.0 - 0.5;
    let fill = (0.08 + eased.clamp(0.0, 1.0) * 0.92) * n as f32;
    for i in 0..n {
        let cx = off + pitch * (i as f32 + 0.5) - 0.5;
        let lr = (fill - i as f32).clamp(0.0, 1.0);
        panel(grid, cx, cy, r, lr, i);
    }
}

/// Elliptical arc (screen y-down: `a = PI/2` is the bottom of the ellipse).
fn ell_arc(grid: &mut BrailleGrid, cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) {
    let steps = (((rx + ry) * (a1 - a0).abs()).ceil() as usize).clamp(4, 1024);
    for i in 0..=steps {
        let a = a0 + (a1 - a0) * i as f32 / steps as f32;
        draw::dot_i(
            grid,
            (cx + rx * a.cos()).round() as i32,
            (cy + ry * a.sin()).round() as i32,
        );
    }
}

/// Progress rail along row `y`, skipping the figure's span `[skip_lo, skip_hi]`:
/// solid up to `frac` of the width, a dotted path marching on `time` beyond.
fn rail(grid: &mut BrailleGrid, y: i32, skip_lo: i32, skip_hi: i32, frac: f32, time: f32) {
    let (dw, _) = draw::dot_dims(grid);
    let lit = (frac.clamp(0.0, 1.0) * dw as f32).round() as i32;
    let phase = (time * 6.0) as i32;
    for x in 0..dw as i32 {
        if x >= skip_lo && x <= skip_hi {
            continue;
        }
        if x < lit || (x + phase).rem_euclid(3) == 0 {
            draw::dot_i(grid, x, y);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Sri Yantra
// ─────────────────────────────────────────────────────────────────────────────

/// Sri Yantra triangles in unit-circle space (y up): `(base_y, apex_y)`.
/// Down-pointing (Shakti) triangles have their base above the apex.  At 16
/// dots tall the full nine collapse into a smudge, so the five outermost —
/// the ones that give the yantra its silhouette — are drawn, interlocking.
const SRI_TRIS: [(f32, f32); 5] = [
    (0.80, -1.00), // Shakti 1 — apex on the circle
    (-0.80, 1.00), // Shiva 1 — apex on the circle
    (0.50, -0.62), // Shakti 2
    (-0.50, 0.62), // Shiva 2
    (0.12, -0.30), // Shakti 3 — cradling the bindu
];

struct SriYantra;
impl ProgressStyle for SriYantra {
    fn name(&self) -> &str {
        "sri-yantra"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Sri Yantra: square gate with four doors, enclosing circle, and the \
         interlocking Shiva (up) and Shakti (down) triangles laid one by one around \
         a pulsing bindu, flanked by a progress path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let half = (dh as f32 / 2.0 - 0.5).max(1.0);
        let r = (half - 1.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);

        // Bhupura: the square gate, a door gap centred on each side.
        let (x0, x1) = ((cx - half).round() as i32, (cx + half).round() as i32);
        let (y0, y1) = ((cy - half).round() as i32, (cy + half - 1.0).round() as i32);
        let door = (half * 0.3).round() as i32;
        let (mx, my) = (cx.round() as i32, cy.round() as i32);
        for x in x0..=x1 {
            if (x - mx).abs() > door {
                draw::dot_i(grid, x, y0);
                draw::dot_i(grid, x, y1);
            }
        }
        for y in y0..=y1 {
            if (y - my).abs() > door {
                draw::dot_i(grid, x0, y);
                draw::dot_i(grid, x1, y);
            }
        }
        rail(grid, my, x0 - 2, x1 + 2, e, ctx.time);

        // Enclosing circle, then the triangles in sequence (outer first).
        circle(grid, cx, cy, r);
        let n = ((0.15 + e * 1.3).min(1.0) * SRI_TRIS.len() as f32).ceil() as usize;
        for &(by, ay) in SRI_TRIS.iter().take(n) {
            let hw = ((ay - by).abs() * 0.78).min((1.0 - by * by).max(0.0).sqrt());
            let bx_l = (cx - hw * r).round() as i32;
            let bx_r = (cx + hw * r).round() as i32;
            let byd = (cy - by * r).round() as i32;
            let ayd = (cy - ay * r).round() as i32;
            triangle(grid, bx_l, byd, bx_r, byd, mx, ayd);
        }

        // Bindu: the seed point, breathing on time.
        if (ctx.time * 2.0).sin() > -0.3 {
            draw::dot_i(grid, mx, my);
            draw::dot_i(grid, mx - 1, my);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Lotus Mandala
// ─────────────────────────────────────────────────────────────────────────────

struct LotusMandala;
impl ProgressStyle for LotusMandala {
    fn name(&self) -> &str {
        "lotus-mandala"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Lotus mandala: eight pointed outer petals and eight inner petals around a \
         seed-pod ring, opening petal by petal and slowly turning between two \
         progress paths"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let big = (dh as f32 / 2.0 - 0.5).max(1.0);
        let pod = (big * 0.28).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        let half = big.ceil() as i32 + 2;
        rail(
            grid,
            cy.round() as i32,
            cx as i32 - half,
            cx as i32 + half,
            e,
            ctx.time,
        );
        circle(grid, cx, cy, pod);

        // A petal: two sides swelling out from the pod and meeting in a point.
        let petal = |grid: &mut BrailleGrid, axis: f32, len: f32, width: f32| {
            let steps = (len * 3.0).ceil() as usize + 2;
            for side in [-1.0f32, 1.0] {
                let mut prev = polar(cx, cy, pod, axis + side * width * 0.6);
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let r = pod + (len - pod) * t;
                    let spread = width * (PI * t).sin().max(0.0).powf(0.7) * (1.0 - t * 0.35);
                    let p = polar(cx, cy, r, axis + side * spread / r.max(1.0) * pod * 1.6);
                    line(grid, prev.0, prev.1, p.0, p.1);
                    prev = p;
                }
            }
        };
        let spin = ctx.time * 0.15 + PI / 2.0;
        // Petals open in turn: eight outer, then eight inner between them.
        let open = (0.15 + e * 1.4).min(1.0) * 16.0;
        for k in 0..16 {
            if (k as f32) >= open {
                break;
            }
            if k < 8 {
                petal(grid, spin + k as f32 * PI / 4.0, big, 0.55);
            } else {
                let axis = spin + (k - 8) as f32 * PI / 4.0 + PI / 8.0;
                petal(grid, axis, big * 0.62, 0.45);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Rose Window (Gothic cathedral tracery)
// ─────────────────────────────────────────────────────────────────────────────

struct RoseWindow;
impl ProgressStyle for RoseWindow {
    fn name(&self) -> &str {
        "rose-window"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Rose windows: a row of Gothic roses, each a stone rim and hub with \
         eight radiating lancet mullions and a ring of foiled oculi between \
         them, the tracery glazed window by window as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx.eased, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            let rot = time * 0.12 + i as f32 * 0.2;
            circle(g, cx, cy, r);
            let hub = (r * 0.26).max(1.0);
            circle(g, cx, cy, hub);
            // Lancets: eight mullions radiating from hub to rim.
            let spokes = ((lr * 1.6).min(1.0) * 8.0).ceil() as usize;
            for k in 0..spokes.min(8) {
                let a = rot + k as f32 * PI / 4.0;
                let (x0, y0) = polar(cx, cy, hub, a);
                let (x1, y1) = polar(cx, cy, r, a);
                line(g, x0, y0, x1, y1);
            }
            // Foils: an oculus in each light between the mullions.
            let foils = (((lr - 0.4) / 0.6).clamp(0.0, 1.0) * 8.0).ceil() as usize;
            for k in 0..foils.min(8) {
                let a = rot + (k as f32 + 0.5) * PI / 4.0;
                let fx = cx + r * 0.64 * a.cos();
                let fy = cy - r * 0.64 * a.sin();
                circle(g, fx, fy, (r * 0.2).max(1.0));
            }
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Bagua (Eight Trigrams)
// ─────────────────────────────────────────────────────────────────────────────

struct Bagua;
impl ProgressStyle for Bagua {
    fn name(&self) -> &str {
        "bagua"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Bagua: the eight I-Ching trigrams in Fu Xi order (☰ ☱ ☲ ☳ | ☴ ☵ ☶ ☷), \
         solid and broken bars appearing one by one either side of a turning \
         yin-yang"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);

        // Taiji: outer circle, the S of two half-circles, and the two eyes,
        // all turning on time.
        let spin = ctx.time * 0.8;
        circle(grid, cx, cy, r);
        let (c1x, c1y) = (
            cx + r / 2.0 * (spin + PI / 2.0).cos(),
            cy - r / 2.0 * (spin + PI / 2.0).sin(),
        );
        let (c2x, c2y) = (
            cx + r / 2.0 * (spin - PI / 2.0).cos(),
            cy - r / 2.0 * (spin - PI / 2.0).sin(),
        );
        arc(grid, c1x, c1y, r / 2.0, spin - PI / 2.0, spin + PI / 2.0);
        arc(
            grid,
            c2x,
            c2y,
            r / 2.0,
            spin + PI / 2.0,
            spin + 3.0 * PI / 2.0,
        );
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            draw::dot_i(grid, c1x.round() as i32 + dx, c1y.round() as i32 + dy);
        }
        circle(grid, c2x, c2y, (r * 0.15).max(1.0));

        // Fu Xi sequence, lines listed bottom → top (true = solid yang).
        const TRIGRAMS: [[bool; 3]; 8] = [
            [true, true, true],    // ☰ Qian — heaven
            [true, true, false],   // ☱ Dui — lake
            [true, false, true],   // ☲ Li — fire
            [true, false, false],  // ☳ Zhen — thunder
            [false, true, true],   // ☴ Xun — wind
            [false, true, false],  // ☵ Kan — water
            [false, false, true],  // ☶ Gen — mountain
            [false, false, false], // ☷ Kun — earth
        ];
        // Four trigrams each side of the taiji, sized to the free width.
        let side = ((cx - r - 3.0) / 4.0).max(3.0);
        let bar_w = (side * 0.75).round().max(3.0) as i32;
        let pitch = (dh as f32 / 4.0).max(1.0);
        let shown = ((0.1 + ctx.eased.clamp(0.0, 1.0) * 1.3).min(1.0) * 8.0).ceil() as usize;
        for (i, tri) in TRIGRAMS.iter().enumerate().take(shown.min(8)) {
            let slot = i as f32;
            let x0 = if i < 4 {
                (cx - r - 3.0 - (4.0 - slot) * side + (side - bar_w as f32) / 2.0).round() as i32
            } else {
                (cx + r + 3.0 + (slot - 4.0) * side + (side - bar_w as f32) / 2.0).round() as i32
            };
            for (l, &solid) in tri.iter().enumerate() {
                // Bottom line lowest on screen.
                let y = (cy + (1.0 - l as f32) * pitch).round() as i32;
                let gap = bar_w / 3;
                for x in 0..bar_w {
                    let mid = x >= gap && x < bar_w - gap;
                    if solid || !mid {
                        draw::dot_i(grid, x0 + x, y);
                        draw::dot_i(grid, x0 + x, y + 1);
                    }
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Enneagram
// ─────────────────────────────────────────────────────────────────────────────

struct Enneagram;
impl ProgressStyle for Enneagram {
    fn name(&self) -> &str {
        "enneagram"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Enneagram: nine points on a circle, the 9-3-6 triangle and the \
         1-4-2-8-5-7 hexad traced line by line with progress, a spark running \
         the hexad's cycle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        let half = r.ceil() as i32 + 2;
        rail(
            grid,
            cy.round() as i32,
            cx as i32 - half,
            cx as i32 + half,
            e,
            ctx.time,
        );
        circle(grid, cx, cy, r);
        // Point 9 at the top, numbering clockwise.
        let pt = |n: usize| polar(cx, cy, r, PI / 2.0 - n as f32 * 2.0 * PI / 9.0);
        let seq: [(usize, usize); 9] = [
            (9, 3),
            (3, 6),
            (6, 9),
            (1, 4),
            (4, 2),
            (2, 8),
            (8, 5),
            (5, 7),
            (7, 1),
        ];
        let n = ((0.15 + e * 1.3).min(1.0) * seq.len() as f32).ceil() as usize;
        for &(a, b) in seq.iter().take(n) {
            let (ax, ay) = pt(a);
            let (bx, by) = pt(b);
            line(grid, ax, ay, bx, by);
        }
        // Spark running the hexad 1 → 4 → 2 → 8 → 5 → 7 → 1.
        let run = (ctx.time * 0.9).rem_euclid(6.0);
        let k = (run as usize).min(5);
        let f = run - k as f32;
        let (ax, ay) = pt(seq[3 + k].0);
        let (bx, by) = pt(seq[3 + k].1);
        let sx = ax + ((bx - ax) as f32 * f).round() as i32;
        let sy = ay + ((by - ay) as f32 * f).round() as i32;
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            draw::dot_i(grid, sx + dx, sy + dy);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Rangoli / Kolam
// ─────────────────────────────────────────────────────────────────────────────

struct Rangoli;
impl ProgressStyle for Rangoli {
    fn name(&self) -> &str {
        "rangoli"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Rangoli kolam: a row of kolams, each a diamond of nine pulli dots \
         with a loop drawn around every dot by a travelling pen, then framed \
         in a diamond border, kolam after kolam as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // Pen order: centre, then around the ring.
        const PULLI: [(i32, i32); 9] = [
            (0, 0),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        let time = ctx.time;
        frieze(grid, ctx.eased, 2.0, 3.0, |g, cx, cy, r, lr, _| {
            let u = r * 0.38;
            let lrad = (u * 0.62).max(0.8);
            for &(a, b) in &PULLI {
                let px = cx + (a - b) as f32 * u;
                let py = cy - (a + b) as f32 * u;
                draw::dot_i(g, px.round() as i32, py.round() as i32);
            }
            if lr <= 0.0 {
                return;
            }
            let pen = (lr * 1.25).min(1.0) * PULLI.len() as f32;
            for (k, &(a, b)) in PULLI.iter().enumerate() {
                let f = (pen - k as f32).clamp(0.0, 1.0);
                if f <= 0.0 {
                    break;
                }
                let px = cx + (a - b) as f32 * u;
                let py = cy - (a + b) as f32 * u;
                let start = PI / 2.0 + k as f32 * 0.7;
                arc(g, px, py, lrad, start, start + 2.0 * PI * f);
                if f < 1.0 && (time * 6.0).sin() > -0.5 {
                    // The pen tip, chalk on the ground.
                    let (tx, ty) = polar(px, py, lrad, start + 2.0 * PI * f);
                    draw::dot_i(g, tx + 1, ty);
                }
            }
            if lr >= 0.8 {
                let d = 2.0 * u + lrad + 0.6;
                let (l, rt) = ((cx - d).round() as i32, (cx + d).round() as i32);
                let (t, b) = ((cy - d).round() as i32, (cy + d).round() as i32);
                let (mx, my) = (cx.round() as i32, cy.round() as i32);
                line(g, l, my, mx, t);
                line(g, mx, t, rt, my);
                line(g, rt, my, mx, b);
                line(g, mx, b, l, my);
            }
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Chartres Labyrinth
// ─────────────────────────────────────────────────────────────────────────────

struct ChartresLabyrinth;
impl ProgressStyle for ChartresLabyrinth {
    fn name(&self) -> &str {
        "chartres-labyrinth"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Chartres labyrinth: the cathedral floor labyrinth seen in perspective \
         — nested circuit walls with their turning gaps — while a walker \
         traces the single path circuit by circuit to the centre, a dotted \
         trail behind and the progress path either side"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let e = ctx.eased.clamp(0.0, 1.0);
        let cx = dw as f32 / 2.0 - 0.5;
        let cy = dh as f32 / 2.0 - 0.5;
        let ry0 = (dh as f32 / 2.0 - 0.5).max(1.0);
        // Floor seen at an angle: the circle widens into an ellipse.
        let k = 2.4f32.min((dw as f32 / 2.0 - 0.5).max(1.0) / ry0);
        let s = ry0 / 3.75;
        let (g, h) = (0.42f32, 0.2f32);
        let span = (ry0 * k).round() as i32;
        let mx = cx.round() as i32;
        rail(
            grid,
            cy.round() as i32,
            mx - span - 2,
            mx + span + 2,
            e,
            ctx.time,
        );

        // Walls: outer to goal, each with its turning gap near the bottom.
        let gaps = [PI / 2.0, PI / 2.0 - g, PI / 2.0 + g, PI / 2.0 - g];
        for (j, &gc) in gaps.iter().enumerate() {
            let ry = ry0 - j as f32 * s;
            if ry < 0.8 {
                break;
            }
            ell_arc(grid, cx, cy, ry * k, ry, gc + h, gc + 2.0 * PI - h);
        }

        // The path: three circuits, reversing direction at each turn.
        let segs: [(f32, f32, f32); 3] = [
            (ry0 - 0.5 * s, PI / 2.0, PI / 2.0 + 2.0 * PI - g),
            (ry0 - 1.5 * s, PI / 2.0 - g, PI / 2.0 + g - 2.0 * PI),
            (ry0 - 2.5 * s, PI / 2.0 + g, PI / 2.0 - g + 2.0 * PI),
        ];
        let lens: Vec<f32> = segs
            .iter()
            .map(|&(ry, a0, a1)| ry * (1.0 + k) * 0.5 * (a1 - a0).abs())
            .collect();
        let total: f32 = lens.iter().sum::<f32>().max(1.0);
        let mut walk = (0.05 + e * 0.95) * total;
        let mut head = (cx, cy);
        for (&(ry, a0, a1), &len) in segs.iter().zip(lens.iter()) {
            if walk <= 0.0 {
                break;
            }
            let f = (walk / len.max(0.1)).min(1.0);
            walk -= len;
            let steps = ((len * f) as usize).clamp(1, 512);
            for i in 0..=steps {
                let a = a0 + (a1 - a0) * f * i as f32 / steps as f32;
                let (x, y) = (cx + ry * k * a.cos(), cy + ry * a.sin());
                if i % 3 == 0 {
                    draw::dot_i(grid, x.round() as i32, y.round() as i32);
                }
                head = (x, y);
            }
        }
        if walk > 0.0 {
            head = (cx, cy);
        }
        // The walker: a small bright figure.
        let (hx, hy) = (head.0.round() as i32, head.1.round() as i32);
        draw::dot_i(grid, hx, hy);
        draw::dot_i(grid, hx + 1, hy);
        if (ctx.time * 4.0).sin() > 0.0 {
            draw::dot_i(grid, hx, hy - 1);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Mandala Tessellation
// ─────────────────────────────────────────────────────────────────────────────

struct MandalaTessellation;
impl ProgressStyle for MandalaTessellation {
    fn name(&self) -> &str {
        "mandala-tessellation"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Mandala tessellation: a row of edge-sharing hexagonal tiles, each \
         laid out with six spokes, a hexagram and a turning inner hexagon — \
         the tiling fills tile by tile as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        let wf = 3f32.sqrt();
        frieze(grid, ctx.eased, wf, 0.0, |g, cx, cy, r, lr, i| {
            // Pointy-top hexagon tile; neighbours share their vertical edges.
            let v: Vec<(i32, i32)> = (0..6)
                .map(|k| polar(cx, cy, r, PI / 2.0 + k as f32 * PI / 3.0))
                .collect();
            for k in 0..6 {
                let (a, b) = (v[k], v[(k + 1) % 6]);
                line(g, a.0, a.1, b.0, b.1);
            }
            if lr <= 0.0 {
                return;
            }
            let spokes = ((lr * 1.5).min(1.0) * 6.0).ceil() as usize;
            for &(x, y) in v.iter().take(spokes.min(6)) {
                line(g, cx.round() as i32, cy.round() as i32, x, y);
            }
            if lr > 0.5 {
                // Hexagram through alternate edge midpoints.
                let m: Vec<(i32, i32)> = (0..6)
                    .map(|k| polar(cx, cy, r * 0.84, k as f32 * PI / 3.0))
                    .collect();
                for k in 0..6 {
                    let (a, b) = (m[k], m[(k + 2) % 6]);
                    line(g, a.0, a.1, b.0, b.1);
                }
            }
            if lr >= 1.0 {
                let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                let rot = dir * time * 0.4;
                let w: Vec<(i32, i32)> = (0..6)
                    .map(|k| polar(cx, cy, r * 0.38, rot + k as f32 * PI / 3.0))
                    .collect();
                for k in 0..6 {
                    let (a, b) = (w[k], w[(k + 1) % 6]);
                    line(g, a.0, a.1, b.0, b.1);
                }
            }
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Dharma Wheel
// ─────────────────────────────────────────────────────────────────────────────

struct DharmaWheel;
impl ProgressStyle for DharmaWheel {
    fn name(&self) -> &str {
        "dharma-wheel"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Dharma wheel (Dharmachakra): rim, nave and eight knobbed spokes — the \
         wheel of the law rolling along its path as progress turns it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let cy = dh as f32 / 2.0 - 0.5;
        let knob = (dh as f32 / 2.0 - 0.5).max(1.0);
        let r = (knob - 1.2).max(1.0);
        // The wheel rolls along its path: travel sets both place and turn.
        let span = (dw as f32 - 2.0 * knob - 1.0).max(0.0);
        let cx = knob + ctx.eased.clamp(0.0, 1.0) * span;
        let turn = -(cx / r) - ctx.time * 0.6;
        let ground = (cx - knob - 1.0).round() as i32;
        for x in 0..ground.max(0) {
            if x % 2 == 0 {
                draw::dot_i(grid, x, cy.round() as i32);
            }
        }
        // Rim and nave.
        circle(grid, cx, cy, r);
        circle(grid, cx, cy, (r * 0.28).max(1.0));
        // Eight spokes, each ending in a knob just outside the rim.
        for i in 0..8 {
            let a = turn + i as f32 * PI / 4.0;
            let (hx, hy) = polar(cx, cy, r * 0.28, a);
            let (rx, ry) = polar(cx, cy, r, a);
            line(grid, hx, hy, rx, ry);
            let (kx, ky) = polar(cx, cy, knob, a);
            draw::dot_i(grid, kx, ky);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Vesica Rosette
// ─────────────────────────────────────────────────────────────────────────────

struct VesicaRosette;
impl ProgressStyle for VesicaRosette {
    fn name(&self) -> &str {
        "vesica-rosette"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Vesica rosettes: a row of six-petal compass rosettes, each petal a \
         vesica lens cut by two arcs of the circle's own radius, petals \
         blooming one by one and rosette by rosette as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx.eased, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            circle(g, cx, cy, r);
            let rot = time * 0.15 + i as f32 * PI / 6.0;
            // Each arc is centred on the rim and passes through the centre;
            // six of them cut six vesica petals.
            let arcs = (lr * 6.0).ceil() as usize;
            for k in 0..arcs.min(6) {
                let a = rot + k as f32 * PI / 3.0;
                let ox = cx + r * a.cos();
                let oy = cy - r * a.sin();
                let back = a + PI;
                arc(g, ox, oy, r, back - PI / 3.0, back + PI / 3.0);
            }
            draw::dot_i(g, cx.round() as i32, cy.round() as i32);
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Star of David / Hexagram
// ─────────────────────────────────────────────────────────────────────────────

struct StarOfDavid;
impl ProgressStyle for StarOfDavid {
    fn name(&self) -> &str {
        "star-of-david"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Star of David hexagram: two interlocking equilateral triangles inked \
         edge by edge and slowly turning, on a progress path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        // Progress path either side of the star.
        let half_w = (r * 0.9).ceil() as i32 + 2;
        rail(
            grid,
            cy.round() as i32,
            cx as i32 - half_w,
            cx as i32 + half_w,
            e,
            ctx.time,
        );
        // Two interlocking equilateral triangles, slowly turning; the six
        // edges are inked in sequence over the first half of the run.
        let spin = ctx.time * 0.35;
        let up = [0, 1, 2].map(|i| polar(cx, cy, r, spin + PI / 2.0 + i as f32 * 2.0 * PI / 3.0));
        let dn = [0, 1, 2].map(|i| polar(cx, cy, r, spin - PI / 2.0 + i as f32 * 2.0 * PI / 3.0));
        let inked = (0.35 + e * 1.4).min(1.0) * 6.0;
        for k in 0..6 {
            let frac = (inked - k as f32).clamp(0.0, 1.0);
            if frac <= 0.0 {
                break;
            }
            let tri = if k % 2 == 0 { &up } else { &dn };
            let i = k / 2;
            let (ax, ay) = tri[i];
            let (bx, by) = tri[(i + 1) % 3];
            let ex = ax + ((bx - ax) as f32 * frac).round() as i32;
            let ey = ay + ((by - ay) as f32 * frac).round() as i32;
            line(grid, ax, ay, ex, ey);
        }
        Ok(())
    }
}
