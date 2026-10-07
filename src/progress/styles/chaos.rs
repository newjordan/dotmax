//! Chaos / strange-attractor progress bars.
//!
//! Each style implements a real dynamical system from the literature, iterating
//! from a fixed seed each frame (stateless) so the bar is a pure function of
//! `(progress, time)`. Attractor orbits are mapped into dot-space coordinates
//! and revealed progressively via `ctx.eased`. Parameter animation uses
//! `ctx.time` to drive slow rotations, phase-shifts, and bifurcations.
//!
//! Systems implemented:
//! - **Lorenz** — the canonical butterfly attractor (σ=10, ρ=28, β=8/3).
//! - **Rössler** — the tilted spiral band with its fold-over lobe (a=0.2, b=0.2, c=5.7).
//! - **Clifford** — three classic parameter sets, density-rendered side by side.
//! - **De Jong** — three classic parameter sets, density-rendered side by side.
//! - **Hénon map** — the folded boomerang attractor (a=1.4, b=0.3).
//! - **Logistic bifurcation** — bifurcation diagram swept across the bar.
//! - **Double pendulum** — two rods and bobs from a pivot riding the progress rail.
//! - **Standard map** — phase portrait whose KAM curves break up as K rises.
//! - **Gingerbreadman map** — the chaotic orbit from (−0.1, 0) filling its silhouette.
//! - **Duffing oscillator** — phase portrait of the forced anharmonic well.
//! - **Tinkerbell map** — the classic wand-trail attractor.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Theme tint — hot pink into violet.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 84, 130);
const TINT_END: Color = Color::rgb(128, 44, 212);

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

/// All styles in the `chaos` theme.
///
/// Returns 11 dynamical-system progress bars. Every style is stateless and
/// renders from a fixed seed each frame; no mutable state is held.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(LorenzAttractor)),
        Box::new(Tinted(RosslerAttractor)),
        Box::new(Tinted(CliffordAttractor)),
        Box::new(Tinted(DeJongAttractor)),
        Box::new(Tinted(HenonMap)),
        Box::new(Tinted(LogisticBifurcation)),
        Box::new(Tinted(DoublePendulum)),
        Box::new(Tinted(StandardMap)),
        Box::new(Tinted(Gingerbreadman)),
        Box::new(Tinted(DuffingOscillator)),
        Box::new(Tinted(TinkerbellMap)),
    ]
}

// ---------------------------------------------------------------------------
// 1. Lorenz Attractor
//    dx/dt = σ(y − x)        σ = 10
//    dy/dt = x(ρ − z) − y    ρ = 28
//    dz/dt = xy − βz         β = 8/3
//
//    Progress reveals orbit length. Time swings the view about the z axis.
// ---------------------------------------------------------------------------

struct LorenzAttractor;

/// Bresenham-style dot line between two signed dot coordinates.
fn seg(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 512);
    for i in 0..=n {
        let x = x0 + (x1 - x0) * i / n;
        let y = y0 + (y1 - y0) * i / n;
        draw::dot_i(grid, x, y);
    }
}

impl ProgressStyle for LorenzAttractor {
    fn name(&self) -> &str {
        "lorenz"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Lorenz butterfly attractor (σ=10 ρ=28 β=8/3) seen in the x–z plane: the orbit loops \
         round both wings as progress lengthens it, and the view swings about the z axis with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (sigma, rho, beta) = (10.0f32, 28.0f32, 8.0f32 / 3.0);
        let dt = 0.01f32;
        let (mut x, mut y, mut z) = (1.0f32, 1.0f32, 20.0f32);
        let step = |x: &mut f32, y: &mut f32, z: &mut f32| {
            // Midpoint (RK2) step — stable at dt = 0.01.
            let (dx, dy, dz) = (sigma * (*y - *x), *x * (rho - *z) - *y, *x * *y - beta * *z);
            let (mx, my, mz) = (*x + dx * dt * 0.5, *y + dy * dt * 0.5, *z + dz * dt * 0.5);
            *x += sigma * (my - mx) * dt;
            *y += (mx * (rho - mz) - my) * dt;
            *z += (mx * my - beta * mz) * dt;
        };
        // Burn in so the orbit starts on the attractor.
        for _ in 0..200 {
            step(&mut x, &mut y, &mut z);
        }
        // Gentle swing of the camera about the z axis.
        let th = (ctx.time * 0.6).sin() * 0.25;
        let (c, s) = (th.cos(), th.sin());
        // Wings span |x| ≲ 20, z ∈ [3, 48]; stretch x to use the wide bar.
        let sx = (dw as f32 * 0.3) / 22.0;
        let sz = (dh as f32 - 1.0) / 46.0;
        let cx = dw as f32 * 0.5;
        let proj = |x: f32, y: f32, z: f32| {
            let u = x * c + y * s;
            ((cx + u * sx) as i32, ((48.0 - z) * sz) as i32)
        };
        let steps = (80.0 + ctx.eased * 1100.0) as u32;
        let mut prev = proj(x, y, z);
        for _ in 0..steps {
            step(&mut x, &mut y, &mut z);
            let p = proj(x, y, z);
            seg(grid, prev.0, prev.1, p.0, p.1);
            prev = p;
        }
        // Bright head on the orbit's leading point.
        draw::fill_rect(grid, prev.0.max(0) as usize, prev.1.max(0) as usize, 2, 2);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Rössler Attractor
//    dx/dt = −y − z
//    dy/dt = x + ay          a = 0.2
//    dz/dt = b + z(x − c)   b = 0.2, c = 5.7
//
//    Oblique view: the x–y spiral band seen tilted, z lifting the fold-over
//    lobe above it. Progress reveals orbit length; time swings the view.
// ---------------------------------------------------------------------------

struct RosslerAttractor;

impl ProgressStyle for RosslerAttractor {
    fn name(&self) -> &str {
        "rossler"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Rössler attractor (a=b=0.2, c=5.7) seen from above and a little to the side: the \
         orbit winds outward round a tilted spiral band, then lifts into the fold-over lobe \
         and drops back to the centre; progress lengthens the orbit, time turns the view"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let a: f32 = 0.2;
        let b: f32 = 0.2;
        let c: f32 = 5.7;
        let dt: f32 = 0.03;

        let n_total = 1500usize;
        let n_draw = ((ctx.eased * n_total as f32) as usize).clamp(2, n_total);

        // View: swing a little about the z axis, tilt so the disc is an ellipse
        // and the z excursion (the fold) rises above it.
        let yaw = 0.35 * (ctx.time * 0.5).sin();
        let (cy_, sy_) = (yaw.cos(), yaw.sin());
        // Fit: horizontal extent ≈ 24 units, vertical ≈ disc depth + lobe.
        let hs = (dw as f32 - 2.0) / 24.0;
        let hs = hs.min(dh as f32 * 0.17).max(0.1);
        let vs = (dh as f32 - 2.0) / 22.0;
        let ox = dw as f32 * 0.5;
        let base = dh as f32 * 0.62;

        let (mut x, mut y, mut z) = (1.0f32, 0.0f32, 0.0f32);
        for _ in 0..300 {
            let (dx, dy, dz) = (-y - z, x + a * y, b + z * (x - c));
            x += dx * dt;
            y += dy * dt;
            z += dz * dt;
        }
        let project = |x: f32, y: f32, z: f32| -> (i32, i32) {
            let rx = x * cy_ - y * sy_;
            let ry = x * sy_ + y * cy_;
            let sx = ox + rx * hs;
            let sy = base - ry * vs * 0.55 - z * vs * 0.55;
            (sx.round() as i32, sy.round() as i32)
        };
        let mut prev = project(x, y, z);
        for _ in 0..n_draw {
            let (dx, dy, dz) = (-y - z, x + a * y, b + z * (x - c));
            x += dx * dt;
            y += dy * dt;
            z += dz * dt;
            let p = project(x, y, z);
            seg(grid, prev.0, prev.1, p.0, p.1);
            prev = p;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Clifford Attractor (IFS)
//    x_{n+1} = sin(a·y) + c·cos(a·x)
//    y_{n+1} = sin(b·x) + d·cos(b·y)
//
//    Three classic (a,b,c,d) sets; a breathes with time. Density-rendered.
// ---------------------------------------------------------------------------

struct CliffordAttractor;

/// Shared renderer for 2-D iterated-map attractors.
///
/// Lays out `panels` copies of the attractor across the bar (each its own
/// parameter set, chosen by `map`'s panel index), developing them left to
/// right: panel `k` iterates `300 + local·5000` points where `local` is its
/// share of progress. Each panel's bounding box is measured from a short
/// warm-up orbit, then hits accumulate in a density buffer and only cells
/// visited often enough are lit — so the attractor's filaments read as lines
/// rather than a solid blob. `aspect` caps a panel's width at `aspect × height`.
fn attractor_row(
    grid: &mut BrailleGrid,
    ctx: &BarContext,
    panels: usize,
    aspect: f32,
    seed: impl Fn(usize) -> (f32, f32),
    map: impl Fn(usize, f32, f32) -> (f32, f32),
) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw < 2 || dh < 2 {
        return;
    }
    let n = if dw >= dh * 4 { panels.max(1) } else { 1 };
    let pw = dw / n;
    let bw = ((dh as f32 * aspect) as usize)
        .min(pw.saturating_sub(2))
        .max(1);
    let mut hits = vec![0u8; bw * dh];
    for k in 0..n {
        let x0 = if n == 1 {
            // A lone figure travels along the bar with progress, leaving a dotted wake.
            let x0 = ((dw - bw) as f32 * ctx.eased) as usize;
            for x in (0..x0).step_by(3) {
                draw::dot(grid, x, dh - 1);
            }
            x0
        } else {
            k * pw + (pw - bw) / 2
        };
        let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
        if local <= 0.0 {
            // Waiting panel: a dotted baseline marks its slot.
            for x in (x0..x0 + bw).step_by(3) {
                draw::dot(grid, x, dh - 1);
            }
            continue;
        }
        let (mut x, mut y) = seed(k);
        let (mut lo_x, mut hi_x, mut lo_y, mut hi_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for i in 0..900 {
            (x, y) = map(k, x, y);
            if !x.is_finite() || !y.is_finite() || x.abs() > 1e4 || y.abs() > 1e4 {
                break;
            }
            if i >= 100 {
                lo_x = lo_x.min(x);
                hi_x = hi_x.max(x);
                lo_y = lo_y.min(y);
                hi_y = hi_y.max(y);
            }
        }
        if !(lo_x < hi_x && lo_y < hi_y) {
            continue;
        }
        let (sx, sy) = (
            (bw - 1) as f32 / (hi_x - lo_x).max(1e-3),
            (dh - 1) as f32 / (hi_y - lo_y).max(1e-3),
        );
        hits.fill(0);
        let count = (300.0 + local * 5000.0) as usize;
        for _ in 0..count {
            (x, y) = map(k, x, y);
            let px = ((x - lo_x) * sx).round();
            let py = ((hi_y - y) * sy).round();
            if px >= 0.0 && py >= 0.0 && (px as usize) < bw && (py as usize) < dh {
                let h = &mut hits[py as usize * bw + px as usize];
                *h = h.saturating_add(1);
            }
        }
        let thr = (1 + count / 1000) as u8;
        for py in 0..dh {
            for px in 0..bw {
                if hits[py * bw + px] >= thr {
                    draw::dot(grid, x0 + px, py);
                }
            }
        }
    }
}

impl ProgressStyle for CliffordAttractor {
    fn name(&self) -> &str {
        "clifford"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Three Clifford attractors (x′=sin ay+c·cos ax, y′=sin bx+d·cos by) from classic parameter \
         sets, each densifying from a sprinkle into its swirling filaments as progress reaches it; \
         parameters breathe with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        const SETS: [[f32; 4]; 3] = [
            [-1.4, 1.6, 1.0, 0.7],
            [1.7, 1.7, 0.6, 1.2],
            [-1.7, 1.3, -0.1, -1.2],
        ];
        let t = ctx.time;
        attractor_row(
            grid,
            ctx,
            3,
            1.7,
            |_| (0.1, 0.1),
            |k, x, y| {
                let [a, b, c, d] = SETS[k % 3];
                let a = a + 0.06 * (t * 0.4 + k as f32).sin();
                (
                    (a * y).sin() + c * (a * x).cos(),
                    (b * x).sin() + d * (b * y).cos(),
                )
            },
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. De Jong Attractor (IFS)
//    x_{n+1} = sin(a·y) − cos(b·x)
//    y_{n+1} = sin(c·x) − cos(d·y)
//
//    Three classic parameter sets; a breathes with time. Density-rendered.
// ---------------------------------------------------------------------------

struct DeJongAttractor;

impl ProgressStyle for DeJongAttractor {
    fn name(&self) -> &str {
        "de-jong"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Three Peter de Jong attractors (x′=sin ay−cos bx, y′=sin cx−cos dy) from classic \
         parameter sets, densifying into their folded ribbons one after another with progress; \
         parameters breathe with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        const SETS: [[f32; 4]; 3] = [
            [1.4, -2.3, 2.4, -2.1],
            [-2.7, -0.09, -0.86, -2.2],
            [-0.827, -1.637, 1.659, -0.943],
        ];
        let t = ctx.time;
        attractor_row(
            grid,
            ctx,
            3,
            1.7,
            |_| (0.1, 0.1),
            |k, x, y| {
                let [a, b, c, d] = SETS[k % 3];
                let a = a + 0.05 * (t * 0.35 + k as f32).sin();
                ((a * y).sin() - (b * x).cos(), (c * x).sin() - (d * y).cos())
            },
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Hénon Map
//    x_{n+1} = 1 − a·x² + y    a = 1.4
//    y_{n+1} = b·x              b = 0.3
//
//    Classic boomerang-shaped strange attractor. Progress = points iterated
//    and the figure's position along the bar; a breathes with time.
// ---------------------------------------------------------------------------

struct HenonMap;

impl ProgressStyle for HenonMap {
    fn name(&self) -> &str {
        "henon"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Hénon map (a=1.4, b=0.3): the boomerang-shaped strange attractor with its parallel folded \
         bands, densifying from scattered points as progress iterates it; a breathes with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let a = 1.4 - 0.01 * (1.0 + (ctx.time * 0.5).sin());
        attractor_row(
            grid,
            ctx,
            1,
            3.6,
            |_| (0.1, 0.1),
            |_, x, y| (1.0 - a * x * x + y, 0.3 * x),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Logistic Map Bifurcation Diagram
//    x_{n+1} = r · x · (1 − x),   x₀ = 0.5
//
//    The x-axis is the bar: r swept from 2.8 to 4.0 left→right.
//    ctx.eased gates how far right r has been revealed.
//    At each r column we iterate 400+ steps (discard) + 8 (plot), enough to
//    show every branch of periods 1, 2, 4, 8 and the period-3 window.
//    The y-axis is x (attractor values). Time shifts which stretch of the
//    orbit is sampled, so the chaotic band shimmers.
//
//    This is the most gorgeous bar — the entire period-doubling cascade
//    is drawn column by column as progress advances.
// ---------------------------------------------------------------------------

struct LogisticBifurcation;

impl ProgressStyle for LogisticBifurcation {
    fn name(&self) -> &str {
        "logistic-bifurcation"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Logistic-map bifurcation diagram: r runs 2.8→4.0 left to right, the long-run values \
         of x stacked up each column — one branch forks into 2, 4, 8 and dissolves into \
         chaos, with the period-3 window near r≈3.83; progress sweeps r, time stirs the chaos"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let r_min: f32 = 2.8;
        let r_max: f32 = 4.0;
        let reveal_cols = ((ctx.eased * dw as f32).round() as usize).min(dw);
        // Time picks which stretch of the long-run orbit is sampled, so the
        // chaotic band shimmers while periodic branches stay put.
        let skip = (ctx.time * 12.0).max(0.0) as usize % 64;
        // Sparse samples keep the chaos speckled so windows and forks read.
        // Eight samples cover every branch of periods 1, 2, 4, 8 and 3.
        let samples = 8usize;

        for col in 0..reveal_cols {
            let r = r_min + (col as f32 / dw.saturating_sub(1).max(1) as f32) * (r_max - r_min);
            let mut x = 0.5f32;
            for _ in 0..(400 + skip) {
                x = r * x * (1.0 - x);
            }
            for _ in 0..samples {
                x = r * x * (1.0 - x);
                let row = ((1.0 - x.clamp(0.0, 1.0)) * (dh - 1) as f32).round() as usize;
                draw::dot(grid, col, row);
            }
        }
        // Leading edge: the column being computed now.
        if reveal_cols > 0 && reveal_cols < dw && (ctx.time * 4.0).sin() > 0.0 {
            for y in (0..dh).step_by(3) {
                draw::dot(grid, reveal_cols, y);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Double Pendulum Trace
//    Lagrangian equations (equal masses m=1, equal lengths l=1, g=9.81):
//
//    θ̈₁ = [−g(2m)sinθ₁ − mg·sin(θ₁−2θ₂) − 2sin(θ₁−θ₂)·m(θ̇₂²l+θ̇₁²l·cos(θ₁−θ₂))]
//           / [l(2m − m·cos(2θ₁−2θ₂))]
//
//    θ̈₂ = [2sin(θ₁−θ₂)(θ̇₁²l·2m + g·2m·cosθ₁ + θ̇₂²l·m·cos(θ₁−θ₂))]
//           / [l(2m − m·cos(2θ₁−2θ₂))]
//
//    Semi-implicit Euler from rest, replayed each frame (24 s loop).
//    Progress moves the pivot along the rail; the lower bob leaves a trail.
// ---------------------------------------------------------------------------

struct DoublePendulum;

impl ProgressStyle for DoublePendulum {
    fn name(&self) -> &str {
        "double-pendulum"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "A real double pendulum (two rods, two bobs) swinging chaotically from a pivot that \
         rides along the top rail with progress, the lower bob trailing its recent path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Equal masses and rod lengths; full Lagrangian equations of motion.
        let g = 9.81f32;
        let dt = 0.004f32;
        let (mut a1, mut a2, mut w1, mut w2) = (1.35f32, 2.2f32, 0.0f32, 0.0f32);
        let accel = |a1: f32, a2: f32, w1: f32, w2: f32| {
            let d = a1 - a2;
            let den = 3.0 - (2.0 * d).cos();
            let n1 = -3.0 * g * a1.sin()
                - g * (a1 - 2.0 * a2).sin()
                - 2.0 * d.sin() * (w2 * w2 + w1 * w1 * d.cos());
            let n2 = 2.0 * d.sin() * (2.0 * w1 * w1 + 2.0 * g * a1.cos() + w2 * w2 * d.cos());
            (n1 / den, n2 / den)
        };
        // Stateless: replay from rest each frame; loop every 24 s to bound cost.
        let t = ctx.time.max(0.0) % 24.0;
        let total = (t / dt) as u32;
        let trail = 160u32;
        let rail = 1i32;
        let len = ((dh as f32 - 2.0) / 2.0).max(1.0);
        // Keep the swing on screen: the pivot travels between the margins.
        let margin = (len * 3.0).min(dw as f32 * 0.25);
        let px = (margin + ctx.eased * (dw as f32 - 1.0 - 2.0 * margin)) as i32;
        let tip = |a1: f32, a2: f32| {
            let (x1, y1) = (a1.sin() * len, a1.cos() * len);
            (x1 + a2.sin() * len, y1 + a2.cos() * len)
        };
        for k in 0..total {
            // Semi-implicit Euler: velocities first, then angles.
            let (e1, e2) = accel(a1, a2, w1, w2);
            w1 += e1 * dt;
            w2 += e2 * dt;
            a1 += w1 * dt;
            a2 += w2 * dt;
            if total - k <= trail && k % 2 == 0 {
                let (tx, ty) = tip(a1, a2);
                draw::dot_i(grid, px + (tx * 2.0) as i32, rail + ty as i32);
            }
        }
        // Top rail: solid up to the pivot (progress), dotted beyond.
        for x in 0..dw as i32 {
            if x <= px || x % 3 == 0 {
                draw::dot_i(grid, x, 0);
            }
        }
        // Rods and bobs. Horizontal is doubled so the swing reads on the wide bar.
        let (x1, y1) = (a1.sin() * len * 2.0, a1.cos() * len);
        let (tx, ty) = tip(a1, a2);
        let b1 = (px + x1 as i32, rail + y1 as i32);
        let b2 = (px + (tx * 2.0) as i32, rail + ty as i32);
        seg(grid, px, rail, b1.0, b1.1);
        seg(grid, b1.0, b1.1, b2.0, b2.1);
        for (bx, by) in [b1, b2] {
            draw::dot_i(grid, bx + 1, by);
            draw::dot_i(grid, bx, by + 1);
            draw::dot_i(grid, bx + 1, by + 1);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Standard Map (Chirikov–Taylor map)
//    p_{n+1} = p_n + K·sin(θ_n)   (mod 2π)
//    θ_{n+1} = θ_n + p_{n+1}      (mod 2π)
//
//    K = stochasticity parameter. Progress ramps K from 0.3 → 1.4.
//    Seven orbits launched across the momentum axis.
//    Time scrolls the portrait in θ.
// ---------------------------------------------------------------------------

struct StandardMap;

impl ProgressStyle for StandardMap {
    fn name(&self) -> &str {
        "standard-map"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Chirikov–Taylor standard map phase portrait (θ across, p up): a fan of orbits traces \
         wavy KAM curves and island chains, and as progress raises K from 0.3 to 1.4 the curves \
         break up into a chaotic sea; the portrait scrolls in θ with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let tau = 2.0 * PI;
        let k = 0.3 + ctx.eased * 1.1;
        let scroll = ctx.time * 0.25;
        let orbits = 7u32;
        for o in 0..orbits {
            // Seeds fan out across the momentum axis.
            let mut p = tau * (o as f32 + 0.5) / orbits as f32;
            let mut th = 0.6 + o as f32 * 0.41;
            for _ in 0..110 {
                p = (p + k * th.sin()).rem_euclid(tau);
                th = (th + p).rem_euclid(tau);
                let x = ((th + scroll).rem_euclid(tau) / tau * dw as f32) as usize;
                // Shift p by π so the main resonance island sits mid-bar.
                let y = ((1.0 - (p + PI).rem_euclid(tau) / tau) * dh as f32) as usize;
                draw::dot(grid, x.min(dw - 1), y.min(dh - 1));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Gingerbreadman Map
//    x_{n+1} = 1 − y + |x|
//    y_{n+1} = x
//
//    Chaotic orbit from (−0.1, 0) filling the gingerbread-man silhouette.
//    Progress iterates more points and moves the figure along the bar.
// ---------------------------------------------------------------------------

struct Gingerbreadman;

impl ProgressStyle for Gingerbreadman {
    fn name(&self) -> &str {
        "gingerbreadman"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Gingerbreadman map (x′=1−y+|x|, y′=x) from (−0.1, 0): the orbit fills the chunky \
         hexagonal gingerbread-man silhouette, densifying as progress iterates it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let wob = 0.004 * (ctx.time * 0.7).sin();
        attractor_row(
            grid,
            ctx,
            1,
            1.25,
            |_| (-0.1 + wob, 0.0),
            |_, x, y| (1.0 - y + x.abs(), x),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Duffing Oscillator Phase Portrait
//     ẋ = y
//     ẏ = −δy + αx − βx³ + γcos(ωt_sim)
//
//     δ=0.25, α=1.0, β=1.0, γ=0.3, ω=1.0 (chaotic two-well driving).
//     Plot (x, y) = (position, velocity) phase portrait.
//     Progress controls how many trajectory steps are drawn.
//     ctx.time offsets the driving phase, ctx.eased gates steps.
// ---------------------------------------------------------------------------

struct DuffingOscillator;

impl ProgressStyle for DuffingOscillator {
    fn name(&self) -> &str {
        "duffing"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Duffing oscillator phase portrait (ẏ=−δy+αx−βx³+γcos ωt): the forced orbit loops round both potential wells, its trace lengthening with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let delta: f32 = 0.25;
        let alpha: f32 = 1.0;
        let beta: f32 = 1.0;
        let gamma: f32 = 0.3;
        let omega: f32 = 1.0;
        let dt: f32 = 0.02;

        let n_total: usize = 4000;
        let n_draw = (ctx.eased * n_total as f32).max(1.0) as usize;

        // Phase portrait fits in roughly x∈[−1.6,1.6], y∈[−1.1,1.1]; keep it
        // to the middle of the bar so the two wells read as two lobes.
        let x_range = 1.7_f32;
        let y_range = 1.15_f32;

        let mut x: f32 = 1.0;
        let mut y: f32 = 0.0;
        // Driving phase starts offset by ctx.time.
        let mut t_sim: f32 = ctx.time * 0.5;

        for _ in 0..n_draw.min(n_total) {
            let force = gamma * (omega * t_sim).cos();
            let ax = y;
            let ay = -delta * y + alpha * x - beta * x * x * x + force;

            // Semi-implicit Euler keeps the orbit from drifting off in energy.
            y += ay * dt;
            x += y * dt;
            let _ = ax;
            t_sim += dt;

            let norm_y = (y + y_range) / (2.0 * y_range);
            let px = (dw as f32 * (0.5 + 0.3 * (x / x_range).clamp(-1.0, 1.0))) as i32;
            let py = ((1.0 - norm_y.clamp(0.0, 1.0)) * (dh - 1) as f32) as i32;
            draw::dot_i(grid, px, py);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Tinkerbell Map
//     x_{n+1} = x² − y² + a·x + b·y
//     y_{n+1} = 2xy + c·x + d·y
//
//     a=0.9, b=−0.6013, c=2.0, d=0.5
//     Named for the fractal boundary of its basin of attraction.
//     Progress iterates more points and moves the figure along the bar.
// ---------------------------------------------------------------------------

struct TinkerbellMap;

impl ProgressStyle for TinkerbellMap {
    fn name(&self) -> &str {
        "tinkerbell"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "The Tinkerbell attractor (x′=x²−y²+ax+by, y′=2xy+cx+dy; a=0.9 b=−0.6013 c=2 d=0.5): \
         its looping wand-trail shape densifies as it travels along the bar with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let wob = 0.004 * (ctx.time * 0.6).sin();
        attractor_row(
            grid,
            ctx,
            1,
            2.2,
            |_| (-0.72, -0.64),
            |_, x, y| {
                let [a, b, c, d] = [0.9, -0.6013, 2.0, 0.5];
                (
                    x * x - y * y + a * x + b * y,
                    2.0 * x * y + c * x + d * y + wob,
                )
            },
        );
        Ok(())
    }
}
