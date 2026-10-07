//! Sacred-geometry / parametric-curve progress bars.
//!
//! Every style maps `ctx.eased` onto the swept parameter range of a classical
//! curve so the shape **draws itself** as progress rises, while `ctx.time`
//! drives continuous rotation / animation.  Eleven distinct styles, each
//! implementing the real mathematical equations in dot space.  On a wide bar
//! each curve is laid out as a row of square panels that draw in turn, so the
//! figures keep their true proportions and the row itself reads as progress.
//!
//! Curve catalogue:
//! - `rose`          — Rhodonea / rose curve r = cos(k·θ)
//! - `spirograph`    — Hypotrochoid spirograph
//! - `epitrochoid`   — Epicycloid variant spirograph
//! - `superformula`  — Gielis superformula morphing through star / flower / polygon sets
//! - `phyllotaxis`   — Sunflower / golden-angle spiral
//! - `cardioid`      — Cardioid envelope traced over its string-art chords
//! - `astroid`       — Astroid (4-cusp hypocycloid)
//! - `lemniscate`    — Lemniscate of Bernoulli (figure-eight)
//! - `maurer-rose`   — Maurer rose (chord-connected rose points)
//! - `mystic-rose`   — Mystic rose (all chords of a pentagon)
//! - `fermat-spiral` — Fermat's / parabolic spiral revealed by progress

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — cyan into violet.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(70, 200, 255);
const TINT_END: Color = Color::rgb(164, 96, 255);

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

/// Lays the inner curve out as a row of square panels across a wide bar, so
/// each figure keeps its true proportions instead of floating alone in empty
/// space. Panels draw in turn: panel `k` of `n` sees local progress
/// `p·n − k`, so the row itself reads as the progress fill. Bars too narrow
/// for two panels render the inner style directly.
struct Gallery<S>(S);

impl<S: ProgressStyle> ProgressStyle for Gallery<S> {
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
        let (cw, ch) = grid.dimensions();
        // A square panel is twice as many cells wide as tall (cells are 2×4 dots).
        let pw = ch * 2;
        let n = (cw + 1) / (pw + 1).max(1);
        if ch == 0 || n < 2 {
            return self.0.render(grid, ctx);
        }
        let left = (cw + 1 - n * (pw + 1)) / 2;
        let mut panel = BrailleGrid::new(pw, ch)?;
        for k in 0..n {
            panel.clear();
            let local = (ctx.progress * n as f32 - k as f32).clamp(0.0, 1.0);
            let eased = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let mut sub = BarContext::new(local, ctx.time + k as f32 * 0.9, pw, ch);
            sub.eased = eased;
            sub.palette = ctx.palette;
            self.0.render(&mut panel, &sub)?;
            let ox = (left + k * (pw + 1)) * 2;
            // Unicode braille bit order: (0,0) (0,1) (0,2) (1,0) (1,1) (1,2) (0,3) (1,3).
            const BITS: [(usize, usize); 8] = [
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 1),
                (1, 2),
                (0, 3),
                (1, 3),
            ];
            for (i, &pat) in panel.get_raw_patterns().iter().enumerate() {
                let (px, py) = (i % pw, i / pw);
                for (b, &(dx, dy)) in BITS.iter().enumerate() {
                    if pat & (1 << b) != 0 {
                        draw::dot(grid, ox + px * 2 + dx, py * 4 + dy);
                    }
                }
            }
            if local <= 0.0 {
                // Waiting panel: a single centre dot marks its slot.
                draw::dot(grid, ox + pw, ch * 2);
            }
        }
        Ok(())
    }
}

/// All styles in the `geometry` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per parametric-curve bar.  The vector
/// is ordered from simplest (rose) to most elaborate (mystic rose), but all
/// styles are independent and can be used in any order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Gallery(Rose))),
        Box::new(Tinted(Gallery(Spirograph))),
        Box::new(Tinted(Gallery(Epitrochoid))),
        Box::new(Tinted(Gallery(Superformula))),
        Box::new(Tinted(Gallery(Phyllotaxis))),
        Box::new(Tinted(Gallery(Cardioid))),
        Box::new(Tinted(Gallery(Astroid))),
        Box::new(Tinted(Gallery(Lemniscate))),
        Box::new(Tinted(Gallery(MaurerRose))),
        Box::new(Tinted(Gallery(MysticRose))),
        Box::new(Tinted(Gallery(FermatSpiral))),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// Shared geometry helpers
// ────────────────────────────────────────────────────────────────────────────

/// Convert polar (r, theta) + grid center to dot-space (x, y), scale by
/// `scale` dots.  Returns signed ints so `draw::dot_i` can bounds-clip them.
#[inline]
fn polar_to_dot(cx: f32, cy: f32, r: f32, theta: f32, scale: f32) -> (i32, i32) {
    let x = cx + r * theta.cos() * scale;
    let y = cy - r * theta.sin() * scale; // y-axis flipped in screen space
    (x.round() as i32, y.round() as i32)
}

/// Compute a sensible uniform scale so a unit-radius curve fits the grid with
/// a small margin.  Uses the minimum half-dimension minus 1 dot of padding.
#[inline]
fn fit_scale(dw: usize, dh: usize) -> f32 {
    let hw = (dw as f32 / 2.0 - 1.0).max(1.0);
    let hh = (dh as f32 / 2.0 - 1.0).max(1.0);
    hw.min(hh)
}

/// Grid center in dot-space (floating-point).
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Rose curve (Rhodonea) — r = cos(k·θ)
// ────────────────────────────────────────────────────────────────────────────

struct Rose;
impl ProgressStyle for Rose {
    fn name(&self) -> &str {
        "rose"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Rhodonea rose r=cos(k·θ): petals unfurl from the center as progress rises, \
         the whole bloom slowly rotating with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // k=5 gives 5 petals (odd k → k petals, even k → 2k petals).
        let k: f32 = 5.0;
        // Full rose requires θ ∈ [0, π] for odd k; sweep up to eased·π.
        let theta_max = ctx.eased * PI;
        // Rotate the whole rose slowly with time.
        let rot = ctx.time * 0.3;

        let steps = (512.0 * ctx.eased).max(4.0) as usize;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let r = (k * theta).cos();
            let (x, y) = polar_to_dot(cx, cy, r, theta + rot, scale);
            draw::dot_i(grid, x, y);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Spirograph / hypotrochoid
// ────────────────────────────────────────────────────────────────────────────
//
//   x = (R − r)·cos θ + d·cos((R−r)/r · θ)
//   y = (R − r)·sin θ − d·sin((R−r)/r · θ)
//
// R=5, r=3, d=5 gives the classic "petals inside a ring" pattern.

struct Spirograph;
impl ProgressStyle for Spirograph {
    fn name(&self) -> &str {
        "spirograph"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Hypotrochoid spirograph: the inner-wheel trace of a circle rolling inside \
         a larger circle, drawing an intricate looping flower as progress sweeps θ"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let big_r: f32 = 5.0;
        let small_r: f32 = 3.0;
        let d: f32 = 5.0;
        let ratio = (big_r - small_r) / small_r;
        // Full period requires lcm(R,r)/r full rotations of θ.
        let theta_max = ctx.eased * 2.0 * PI * small_r; // ≈ 6π for (5,3)
        let rot = ctx.time * 0.2;

        let steps = (800.0 * ctx.eased).max(4.0) as usize;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let xf = (big_r - small_r) * theta.cos() + d * (ratio * theta).cos();
            let yf = (big_r - small_r) * theta.sin() - d * (ratio * theta).sin();
            // Normalise by the bounding radius so it fills the grid.
            let norm = big_r + d;
            let sx = cx + (xf / norm) * scale * (rot.cos());
            let sy = cy - (yf / norm) * scale;
            // Apply rotation in dot-space.
            let dx = xf / norm * scale;
            let dy = yf / norm * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            let _ = (sx, sy); // suppress unused warning
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Epitrochoid (epicycloid variant)
// ────────────────────────────────────────────────────────────────────────────
//
//   x = (R + r)·cos θ − d·cos((R+r)/r · θ)
//   y = (R + r)·sin θ − d·sin((R+r)/r · θ)

struct Epitrochoid;
impl ProgressStyle for Epitrochoid {
    fn name(&self) -> &str {
        "epitrochoid"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Epitrochoid: a small circle rolling *outside* a fixed circle traces \
         a multi-lobed crown; progress reveals the full pattern chord by chord"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // R=3, r=1, d=2.5 → 3-lobed limaçon-like shape.
        let big_r: f32 = 3.0;
        let small_r: f32 = 1.0;
        let d: f32 = 2.5;
        let ratio = (big_r + small_r) / small_r;
        let theta_max = ctx.eased * 2.0 * PI;
        let rot = ctx.time * 0.25;

        let steps = (600.0 * ctx.eased).max(4.0) as usize;
        let norm = big_r + small_r + d;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let xf = (big_r + small_r) * theta.cos() - d * (ratio * theta).cos();
            let yf = (big_r + small_r) * theta.sin() - d * (ratio * theta).sin();
            let dx = xf / norm * scale;
            let dy = yf / norm * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Gielis Superformula
// ────────────────────────────────────────────────────────────────────────────
//
//   r(φ) = [ |cos(m·φ/4)/a|^n2 + |sin(m·φ/4)/b|^n3 ]^(−1/n1)
//
// Time steps through classic parameter sets (star, triangle, flower, square,
// burst, thin star, lens), blending radially from one to the next.

/// Classic superformula parameter sets `(m, n1, n2, n3)` with a = b = 1.
const SUPER_SHAPES: [(f32, f32, f32, f32); 7] = [
    (5.0, 2.0, 7.0, 7.0),    // five-point star
    (3.0, 4.5, 10.0, 10.0),  // rounded triangle
    (6.0, 1.0, 7.0, 8.0),    // six-petal flower
    (4.0, 12.0, 15.0, 15.0), // pillowed square
    (8.0, 0.6, 1.0, 8.0),    // spiky eight-point burst
    (7.0, 0.2, 1.7, 1.7),    // thin seven-arm star
    (2.0, 1.0, 4.0, 8.0),    // lens / bean
];

/// Superformula radius r(φ) for one parameter set.
fn super_r(s: (f32, f32, f32, f32), phi: f32) -> f32 {
    let (m, n1, n2, n3) = s;
    let t1 = (m * phi / 4.0).cos().abs().powf(n2);
    let t2 = (m * phi / 4.0).sin().abs().powf(n3);
    let sum = t1 + t2;
    if sum < 1e-6 {
        return 0.0;
    }
    sum.powf(-1.0 / n1).min(50.0)
}

struct Superformula;
impl ProgressStyle for Superformula {
    fn name(&self) -> &str {
        "superformula"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Gielis superformula: one equation, many shapes — each panel morphs through a star, a \
         rounded triangle, a six-petal flower, a pillowed square, a spiky burst, a thin \
         seven-arm star and a lens; progress traces the outline, time morphs the parameters"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // Hold each shape, then blend radially into the next.
        let phase = (ctx.time.max(0.0) * 0.7).rem_euclid(SUPER_SHAPES.len() as f32);
        let i0 = (phase as usize) % SUPER_SHAPES.len();
        let i1 = (i0 + 1) % SUPER_SHAPES.len();
        let f = ((phase.fract() - 0.55) / 0.45).clamp(0.0, 1.0);
        let f = f * f * (3.0 - 2.0 * f);
        let (s0, s1) = (SUPER_SHAPES[i0], SUPER_SHAPES[i1]);

        const STEPS: usize = 240;
        let mut r0 = [0.0f32; STEPS + 1];
        let mut r1 = [0.0f32; STEPS + 1];
        let (mut m0, mut m1) = (1e-3f32, 1e-3f32);
        for i in 0..=STEPS {
            let phi = i as f32 / STEPS as f32 * 2.0 * PI;
            r0[i] = super_r(s0, phi);
            r1[i] = super_r(s1, phi);
            m0 = m0.max(r0[i]);
            m1 = m1.max(r1[i]);
        }
        let rot = -PI * 0.5 + ctx.time * 0.1;
        let shown = ((ctx.eased.clamp(0.0, 1.0) * STEPS as f32).round() as usize).max(1);
        let mut prev: Option<(i32, i32)> = None;
        for i in 0..=shown.min(STEPS) {
            let phi = i as f32 / STEPS as f32 * 2.0 * PI;
            let r = (r0[i] / m0) * (1.0 - f) + (r1[i] / m1) * f;
            let a = phi + rot;
            let px = (cx + r * a.cos() * scale).round() as i32;
            let py = (cy + r * a.sin() * scale).round() as i32;
            if let Some((qx, qy)) = prev {
                bresenham(grid, qx, qy, px, py);
            } else {
                draw::dot_i(grid, px, py);
            }
            prev = Some((px, py));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Phyllotaxis / sunflower (golden-angle spiral)
// ────────────────────────────────────────────────────────────────────────────
//
//   angle_n = n · 137.5°
//   radius_n = c · √n
//
// Plot the first eased·N points; N grows so the spiral fills as progress rises.

struct Phyllotaxis;
impl ProgressStyle for Phyllotaxis {
    fn name(&self) -> &str {
        "phyllotaxis"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Sunflower phyllotaxis: seeds appear at the golden angle 137.5°, \
         radius ∝ √n, producing the mesmerising Fibonacci spiral pattern of \
         real sunflowers and pinecones"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let golden_angle: f32 = 2.0 * PI * (1.0 - 1.0 / 1.618_033_9); // ≈ 137.508°
        let n_max: f32 = 90.0;
        let n_plot = (ctx.eased * n_max).round() as usize;
        // c chosen so the outermost seed lands near the grid edge.
        let c = scale / n_max.sqrt();
        let rot = ctx.time * 0.15;

        for n in 0..n_plot {
            let angle = n as f32 * golden_angle + rot;
            let r = c * (n as f32).sqrt();
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy - r * angle.sin()).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Cardioid via string-art (multiplication table on a circle)
// ────────────────────────────────────────────────────────────────────────────
//
//   Place N points evenly around a unit circle.
//   Draw chord from point i to point (2·i mod N).
//   The envelope of these chords is the cardioid r = a(1 − cos θ).

struct Cardioid;
impl ProgressStyle for Cardioid {
    fn name(&self) -> &str {
        "cardioid"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Cardioid: chords from point i to 2·i on a circle appear one by one while \
         the heart-shaped envelope they converge on, (2e^{iθ}+e^{2iθ})/3, traces \
         itself out as progress rises — a row of panels drawing in turn"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh) * 0.9;

        let n_total: f32 = 10.0; // points on the circle → 10 chords
        let rot = ctx.time * 0.2;
        let at = |a: f32| {
            (
                (cx + a.cos() * scale).round() as i32,
                (cy - a.sin() * scale).round() as i32,
            )
        };
        // Sparse string art: chord i → 2i, drawn one by one with progress.
        let n_chords = (ctx.eased * n_total).round() as usize;
        for i in 0..n_chords {
            let a_i = 2.0 * PI * i as f32 / n_total + rot;
            let (x0, y0) = at(a_i);
            let (x1, y1) = at(a_i * 2.0 - rot);
            bresenham(grid, x0, y0, x1, y1);
        }
        // The envelope those chords converge on — the cardioid itself,
        // z(θ) = (2·e^{iθ} + e^{2iθ}) / 3 — traced out as progress rises.
        let steps = (ctx.eased * 90.0) as usize;
        for k in 0..=steps {
            let th = 2.0 * PI * k as f32 / 90.0;
            let x = (2.0 * th.cos() + (2.0 * th).cos()) / 3.0;
            let y = (2.0 * th.sin() + (2.0 * th).sin()) / 3.0;
            let (c, s2) = (rot.cos(), rot.sin());
            let (u, v) = (x * c - y * s2, x * s2 + y * c);
            draw::dot_i(
                grid,
                (cx + u * scale).round() as i32,
                (cy - v * scale).round() as i32,
            );
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Astroid (4-cusp hypocycloid)
// ────────────────────────────────────────────────────────────────────────────
//
//   x = R·cos³θ,  y = R·sin³θ   (hypocycloid with k=4, so r = R/4)

struct Astroid;
impl ProgressStyle for Astroid {
    fn name(&self) -> &str {
        "astroid"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Astroid (4-cusp hypocycloid): x=R·cos³θ, y=R·sin³θ — a star-shaped \
         curve with four sharp cusps that sweeps closed as progress completes a \
         full 2π revolution"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let theta_max = ctx.eased * 2.0 * PI;
        let rot = ctx.time * 0.22;
        let steps = (512.0 * ctx.eased).max(4.0) as usize;

        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let xf = theta.cos().powi(3);
            let yf = theta.sin().powi(3);
            let dx = xf * scale;
            let dy = yf * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Lemniscate of Bernoulli — r² = a²·cos(2θ)
// ────────────────────────────────────────────────────────────────────────────
//
//   Cartesian form: (x²+y²)² = a²(x²−y²)
//   Parametric:     x = a·cos(t)/(1+sin²t),  y = a·sin(t)·cos(t)/(1+sin²t)

struct Lemniscate;
impl ProgressStyle for Lemniscate {
    fn name(&self) -> &str {
        "lemniscate"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Lemniscate of Bernoulli: the ∞ figure-eight (x²+y²)²=a²(x²−y²), \
         traced with the rational parametric form — both lobes materialise \
         symmetrically as progress sweeps 0→2π"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let theta_max = ctx.eased * 2.0 * PI;
        let rot = ctx.time * 0.18;
        let steps = (512.0 * ctx.eased).max(4.0) as usize;

        for i in 0..=steps {
            let t = i as f32 / steps as f32 * theta_max;
            let denom = 1.0 + t.sin().powi(2);
            if denom.abs() < 1e-6 {
                continue;
            }
            // Rational parametric lemniscate (a = 1).
            let xf = t.cos() / denom;
            let yf = t.sin() * t.cos() / denom;
            let dx = xf * scale;
            let dy = yf * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Maurer Rose
// ────────────────────────────────────────────────────────────────────────────
//
//   The rose curve r = cos(k·θ) evaluated at evenly spaced integer multiples
//   of d degrees, connected with straight chords.  d=71°, k=5 is classic.

struct MaurerRose;
impl ProgressStyle for MaurerRose {
    fn name(&self) -> &str {
        "maurer-rose"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Maurer rose: points of the rose r=cos(5θ) at 71° steps joined by straight \
         chords, the first 36 chords weaving a star web in each of a row of panels \
         as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let k: f32 = 5.0;
        let d_deg: f32 = 71.0; // step in degrees
        let d_rad = d_deg * PI / 180.0;
        let n_total: f32 = 36.0; // a tenth of the full 361-step weave, legible at small size
        let n_chords = (ctx.eased * n_total).round() as usize;
        let rot = ctx.time * 0.15;

        // Compute successive chord endpoints.
        let point = |n: usize| -> (i32, i32) {
            let theta = n as f32 * d_rad + rot;
            let r = (k * theta).cos();
            let x = (cx + r * theta.cos() * scale).round() as i32;
            let y = (cy - r * theta.sin() * scale).round() as i32;
            (x, y)
        };

        for n in 0..n_chords.saturating_sub(1) {
            let (x0, y0) = point(n);
            let (x1, y1) = point(n + 1);
            bresenham(grid, x0, y0, x1, y1);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Mystic Rose — all chords of a regular n-gon
// ────────────────────────────────────────────────────────────────────────────
//
//   Place N points equally spaced on a circle and draw every chord.
//   Total chords = N·(N−1)/2.  Progress reveals them in order.

struct MysticRose;
impl ProgressStyle for MysticRose {
    fn name(&self) -> &str {
        "mystic-rose"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Mystic rose: every pair of vertices of a regular pentagon joined by a \
         chord — the pentagon and its inscribed pentagram, ten chords appearing \
         one by one as progress rises, across a row of panels"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let n: usize = 5; // pentagon + pentagram → 10 chords
        let total_chords = n * (n - 1) / 2;
        let chords_to_draw = (ctx.eased * total_chords as f32).round() as usize;
        let rot = ctx.time * 0.12;

        let vertex = |i: usize| -> (i32, i32) {
            let angle = 2.0 * PI * i as f32 / n as f32 + rot;
            let px = (cx + angle.cos() * scale).round() as i32;
            let py = (cy - angle.sin() * scale).round() as i32;
            (px, py)
        };

        let mut drawn = 0usize;
        'outer: for i in 0..n {
            for j in (i + 1)..n {
                if drawn >= chords_to_draw {
                    break 'outer;
                }
                let (x0, y0) = vertex(i);
                let (x1, y1) = vertex(j);
                bresenham(grid, x0, y0, x1, y1);
                drawn += 1;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Fermat's Spiral (parabolic spiral)
// ────────────────────────────────────────────────────────────────────────────
//
//   r = ±a·√θ,  θ ∈ [0, eased·θ_max]
//
// Both arms (±r) give the symmetric double-armed spiral.

struct FermatSpiral;
impl ProgressStyle for FermatSpiral {
    fn name(&self) -> &str {
        "fermat-spiral"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Fermat's (parabolic) spiral r=±a√θ: both symmetric arms uncoil one and a \
         half turns from the origin as progress sweeps outward, the pair slowly \
         rotating with time, across a row of panels"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // How many full revolutions to uncoil (6π ≈ 3 turns).
        let theta_max = ctx.eased * 3.0 * PI;
        let rot = ctx.time * 0.17;
        // Normalise so the outermost point lands near the grid edge.
        // At θ_max the radius = a·√θ_max; we want that ≈ 1.0 in unit coords.
        let a = 1.0 / (3.0 * PI).sqrt(); // ensures r ≤ 1 at full sweep

        let steps = (600.0 * ctx.eased).max(4.0) as usize;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let r = a * theta.sqrt();
            // Positive arm.
            let angle = theta + rot;
            let px = (cx + r * angle.cos() * scale).round() as i32;
            let py = (cy - r * angle.sin() * scale).round() as i32;
            draw::dot_i(grid, px, py);
            // Negative arm (π offset = opposite side).
            let qx = (cx - r * angle.cos() * scale).round() as i32;
            let qy = (cy + r * angle.sin() * scale).round() as i32;
            draw::dot_i(grid, qx, qy);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Internal: integer Bresenham line rasteriser
// ────────────────────────────────────────────────────────────────────────────

/// Draw a straight line between two signed dot-space points using Bresenham's
/// algorithm.  Out-of-bounds dots are silently discarded by `draw::dot_i`.
fn bresenham(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    // Bound the number of steps to avoid infinite loops on degenerate input.
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;

    loop {
        draw::dot_i(grid, x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}
