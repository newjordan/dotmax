//! Quantum-mechanics progress bars for dotmax.
//!
//! Eleven styles, each grounded in a distinct quantum phenomenon.  Every bar is a
//! pure function of `(ctx.progress, ctx.eased, ctx.time)` — no mutable state.
//!
//! ## Styles (11 total)
//!
//! | Name | Phenomenon | Key idea |
//! |------|-----------|---------|
//! | `qubit-superposition` | Qubit |0⟩↔|1⟩ | Rabi wave between ket levels, collapsed to |1⟩ behind progress |
//! | `bloch-sphere` | Bloch sphere | round sphere travels; arrow tips |0⟩→|1⟩, precesses with time |
//! | `wavefunction-collapse` | Measurement | probability cloud sharpens to spike as eased→1 |
//! | `quantum-tunneling` | Tunneling | wave packet partially transmits through a barrier |
//! | `harmonic-oscillator` | QHO eigenstates | |ψₙ|² in a parabolic well, n from eased |
//! | `particle-in-box` | PIB standing modes | sin²(nπx/L) mode n from eased |
//! | `spin-precession` | Spin precession | arrow cones about B, travels with eased, traces a wave |
//! | `energy-levels` | Orbital filling | Bohr-like levels filling with electrons |
//! | `decoherence` | Decoherence | sharp fringes wash out as eased increases |
//! | `quantum-walk` | Quantum random walk | twin ballistic peaks spread linearly from the start site |
//! | `electron-buildup` | Single-particle double-slit | particle hits build up fringes on a screen as eased→1 |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `quantum` theme.
///
/// Returns 11 bars, each modelling a distinct quantum-mechanical phenomenon.
/// Safe to render at every size from 1×1 to 80×8 or beyond.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(QubitSuperposition),
        Box::new(BlochSphere),
        Box::new(WavefunctionCollapse),
        Box::new(QuantumTunneling),
        Box::new(HarmonicOscillator),
        Box::new(ParticleInBox),
        Box::new(SpinPrecession),
        Box::new(EnergyLevels),
        Box::new(Decoherence),
        Box::new(QuantumWalk),
        Box::new(DoubleSlit),
    ]
}

// ---------------------------------------------------------------------------
// Helper: integer-step Bresenham line in dot space
// ---------------------------------------------------------------------------
#[inline]
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let px = x0 + (x1 - x0) * i / steps;
        let py = y0 + (y1 - y0) * i / steps;
        draw::dot_i(grid, px, py);
    }
}

// ---------------------------------------------------------------------------
// 1. Qubit Superposition
//    Two basis levels, labelled |1⟩ (top) and |0⟩ (bottom) in a tiny dot
//    font.  Ahead of the progress front the qubit is in superposition: its
//    amplitude swings between the two levels as a travelling Rabi wave.
//    Behind the front it has been measured — collapsed to a solid |1⟩ line.
// ---------------------------------------------------------------------------

/// 3×5 dot glyphs for ket labels: `|`, `0`, `1`, `⟩` (row-major bitmasks).
const KET_FONT: [[u8; 5]; 4] = [
    [0b010, 0b010, 0b010, 0b010, 0b010], // |
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b100, 0b010, 0b001, 0b010, 0b100], // ⟩
];

/// Draw a ket `|d⟩` with its top-left at `(x, y)`; returns its width in dots.
fn ket(grid: &mut BrailleGrid, x: i32, y: i32, digit: usize) -> i32 {
    let glyphs = [0usize, 1 + digit.min(1), 3];
    let mut cx = x;
    for (n, &g) in glyphs.iter().enumerate() {
        for (row, bits) in KET_FONT[g].iter().enumerate() {
            for col in 0..3 {
                if bits & (0b100 >> col) != 0 {
                    draw::dot_i(grid, cx + col, y + row as i32);
                }
            }
        }
        // The bar of `|` is centred, so it only needs two columns.
        cx += if n == 0 { 2 } else { 4 } + i32::from(n == 0);
    }
    cx - x
}

struct QubitSuperposition;
impl ProgressStyle for QubitSuperposition {
    fn name(&self) -> &str {
        "qubit-superposition"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Qubit levels |0⟩ and |1⟩: a Rabi wave swings between them, collapsing to a solid |1⟩ behind progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (w, h) = (dw as i32, dh as i32);
        if h < 12 || w < 24 {
            // Too small for labels: wave ahead, collapsed line behind.
            let front = (ctx.eased * dw as f32) as i32;
            for x in 0..w {
                let y = if x < front {
                    0
                } else {
                    let s = ((x as f32 * 0.4 - ctx.time * 6.0).sin() + 1.0) * 0.5;
                    (s * (h - 1) as f32).round() as i32
                };
                draw::dot_i(grid, x, y);
            }
            return Ok(());
        }

        // Level rows sit level with the label centres.
        let (top_y, bot_y) = (2, h - 3);
        let lw = ket(grid, 0, 0, 1);
        ket(grid, 0, h - 5, 0);
        let label_col = Color::rgb(200, 205, 225);
        draw::tint_row(grid, 0, 0, (lw / 2) as usize, label_col);
        draw::tint_row(
            grid,
            ((h - 1) / 4) as usize,
            0,
            (lw / 2) as usize,
            label_col,
        );

        let x0 = lw + 2;
        let span = (w - x0).max(1);
        let front = x0 + (span as f32 * ctx.eased.clamp(0.0, 1.0)) as i32;
        let shift = (ctx.time * 12.0) as i32;

        // Measured region: collapsed onto |1⟩ as a solid, bright level.
        if front > x0 {
            for y in top_y - 1..=top_y + 1 {
                draw::hline(grid, x0 as usize, (front - 1) as usize, y as usize);
            }
            for cx in (x0 / 2)..=((front - 1) / 2) {
                let t = (cx * 2 - x0) as f32 / span as f32;
                draw::tint_row(grid, 0, cx as usize, cx as usize, ctx.palette.sample(t));
            }
        }

        // Superposed region: dashed levels plus the Rabi wave between them.
        let amp = (bot_y - top_y) as f32 * 0.5;
        let mid = (top_y + bot_y) as f32 * 0.5;
        let mut prev: Option<(i32, i32)> = None;
        for x in front.max(x0)..w {
            if (x + shift).rem_euclid(4) == 0 {
                draw::dot_i(grid, x, top_y);
                draw::dot_i(grid, x, bot_y);
            }
            let y = (mid - amp * (x as f32 * 0.28 - ctx.time * 5.0).sin()).round() as i32;
            if let Some((px, py)) = prev {
                line(grid, px, py, x, y);
            }
            prev = Some((x, y));
        }
        // Collapse flash: a short vertical line at the measurement front.
        if front > x0 && front < w {
            line(grid, front, top_y - 1, front, bot_y);
        }
        let wave_col = Color::rgb(150, 160, 255);
        for cy in 0..(h / 4) {
            let lo = (front.max(x0) / 2) as usize;
            draw::tint_row(grid, cy as usize, lo, (w / 2) as usize, wave_col);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Bloch Sphere
//    A round wireframe sphere — silhouette, tilted equator, polar axis with
//    |0⟩ at the north pole — carrying a state-vector arrow.  The polar angle
//    θ = π·eased tips the arrow from |0⟩ to |1⟩ while the azimuth φ precesses
//    with time.  The sphere also rolls along the bar with progress, leaving a
//    tinted trail.
// ---------------------------------------------------------------------------
struct BlochSphere;
impl ProgressStyle for BlochSphere {
    fn name(&self) -> &str {
        "bloch-sphere"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Round Bloch sphere: state arrow tips from |0⟩ to |1⟩ with progress and precesses with time as the sphere travels"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (w, h) = (dw as i32, dh as i32);
        // Round sphere: radius set by the shorter side.
        let r = ((h - 1) / 2).min((w - 1) / 2).max(1);
        let rf = r as f32;
        let e = ctx.eased.clamp(0.0, 1.0);
        let cx = r + ((w - 1 - 2 * r).max(0) as f32 * e) as i32;
        let cy = (h - 1) / 2;
        let cyf = (h - 1) as f32 / 2.0;

        // Trail: the path already travelled, along the equator line.
        if cx - r > 1 {
            draw::hline(grid, 0, (cx - r - 2) as usize, cy as usize);
            for c in 0..=((cx - r - 2) / 2) {
                let t = c as f32 * 2.0 / w as f32;
                draw::tint_row(
                    grid,
                    (cy / 4) as usize,
                    c as usize,
                    c as usize,
                    ctx.palette.sample(t),
                );
            }
        }

        // Silhouette circle.
        let steps = (rf * 8.0) as usize + 8;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            draw::dot_i(
                grid,
                cx + (rf * a.cos()).round() as i32,
                (cyf + rf * a.sin()).round() as i32,
            );
        }
        // Equator: front half solid, back half dotted.
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            if a.sin() < 0.0 && s % 3 != 0 {
                continue;
            }
            draw::dot_i(
                grid,
                cx + (rf * a.cos()).round() as i32,
                (cyf + rf * 0.3 * a.sin()).round() as i32,
            );
        }
        // Polar axis (dashed) with pole caps.
        for y in (cy - r)..=(cy + r) {
            if y % 2 == 0 {
                draw::dot_i(grid, cx, y);
            }
        }
        draw::dot_i(grid, cx - 1, cy - r);
        draw::dot_i(grid, cx + 1, cy - r);
        let sphere_col = Color::rgb(120, 130, 170);
        for c in ((cx - r) / 2)..=((cx + r) / 2) {
            for row in 0..=(h - 1) / 4 {
                draw::tint_row(
                    grid,
                    row as usize,
                    c.max(0) as usize,
                    c.max(0) as usize,
                    sphere_col,
                );
            }
        }

        // State vector: θ from progress, φ precessing with time.
        let theta = PI * e;
        let phi = ctx.time * 2.5;
        let vx = theta.sin() * phi.cos();
        let vy = theta.sin() * phi.sin(); // toward the viewer
        let vz = theta.cos();
        let tip_x = cx + (vx * rf * 0.95).round() as i32;
        let tip_y = (cyf - vz * rf * 0.95 + vy * rf * 0.3).round() as i32;
        line(grid, cx, cy, tip_x, tip_y);
        line(grid, cx + 1, cy, tip_x + 1, tip_y);
        // Bright tip blob.
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1)] {
            draw::dot_i(grid, tip_x + dx, tip_y + dy);
        }
        let arrow_col = ctx.palette.sample(e);
        let (ax0, ax1) = (cx.min(tip_x) / 2, cx.max(tip_x + 1) / 2);
        let (ay0, ay1) = (cy.min(tip_y) / 4, cy.max(tip_y) / 4);
        for row in ay0.max(0)..=ay1 {
            draw::tint_row(
                grid,
                row as usize,
                ax0.max(0) as usize,
                ax1.max(0) as usize,
                arrow_col,
            );
        }
        let tip_col = Color::rgb(255, 255, 255);
        draw::tint_row(
            grid,
            (tip_y.max(0) / 4) as usize,
            (tip_x.max(0) / 2) as usize,
            (tip_x.max(0) / 2) as usize,
            tip_col,
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Wavefunction Collapse
//    Before measurement: |ψ(x)|² is a broad Gaussian centred at mid-bar.
//    After measurement (eased→1): it narrows to a sharp delta spike.
//    Width σ(p) = σ_max · (1 − eased) + σ_min
//    We render the probability density as a column-height bar chart using
//    vblock glyphs — the shape is structural, not just a color wash.
// ---------------------------------------------------------------------------
struct WavefunctionCollapse;
impl ProgressStyle for WavefunctionCollapse {
    fn name(&self) -> &str {
        "wavefunction-collapse"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Wavefunction collapse: |ψ(x)|² spreads as a broad Gaussian then sharpens to a spike at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let cwf = cw as f32;

        // Width of the Gaussian: broad at eased=0, narrow at eased=1
        let sigma_max = cwf * 0.35;
        let sigma_min = cwf.max(4.0) * 0.025;
        let sigma = sigma_max * (1.0 - ctx.eased) + sigma_min;

        // Measurement position: oscillates before collapse, locks at centre
        let center_norm = 0.5_f32 + 0.15 * (ctx.time * 1.8).sin() * (1.0 - ctx.eased);
        let mean = center_norm * cwf;

        // Scan columns, compute Gaussian height, draw vblock + tint
        for cx in 0..cw {
            let xf = cx as f32 + 0.5;
            let exponent = -0.5 * ((xf - mean) / sigma.max(0.1)).powi(2);
            let amplitude = exponent.exp().clamp(0.0, 1.0);

            // Use vblock in each cell of this column (top-down fill)
            for cy in 0..ch {
                let row_norm = 1.0 - cy as f32 / ch as f32; // 0 at bottom, 1 at top
                if amplitude >= row_norm {
                    draw::vblock(grid, cx, cy, 8);
                } else {
                    // partial block for the transition row
                    let partial = ((amplitude - (row_norm - 1.0 / ch as f32)) * ch as f32 * 8.0)
                        .clamp(0.0, 8.0) as usize;
                    if partial > 0 {
                        draw::vblock(grid, cx, cy, partial);
                    }
                }
            }

            // Tint by palette
            let t = cx as f32 / cwf;
            let col = ctx.palette.sample(t * amplitude + ctx.eased * (1.0 - t));
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Quantum Tunneling
//    A Gaussian wave packet travels rightward and hits a rectangular barrier
//    centred at 60% of the bar width.  The transmitted fraction = eased
//    (classically forbidden for small eased; quantum tunneling allows it).
//    Left of barrier: incident + reflected standing-wave pattern.
//    Right of barrier: transmitted wave of amplitude ~ sqrt(eased).
//    Both regions rendered as a sinusoidal waveform in dot space.
// ---------------------------------------------------------------------------
struct QuantumTunneling;
impl ProgressStyle for QuantumTunneling {
    fn name(&self) -> &str {
        "quantum-tunneling"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Quantum tunneling: wave packet hits a barrier, transmitted amplitude grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let dwf = dw as f32;
        let dhf = dh as f32;

        let mid_y = (dhf / 2.0) as i32;
        // Barrier position and width
        let barrier_x0 = (dwf * 0.55) as usize;
        let barrier_x1 = (dwf * 0.65) as usize;
        let barrier_h = (dhf * 0.7).max(2.0) as usize;
        let bar_top = ((dhf - barrier_h as f32) / 2.0) as usize;

        // Draw barrier outline
        for y in bar_top..(bar_top + barrier_h).min(dh) {
            draw::dot(grid, barrier_x0.min(dw.saturating_sub(1)), y);
            draw::dot(grid, barrier_x1.min(dw.saturating_sub(1)), y);
        }
        draw::hline(
            grid,
            barrier_x0,
            barrier_x1.min(dw.saturating_sub(1)),
            bar_top,
        );
        draw::hline(
            grid,
            barrier_x0,
            barrier_x1.min(dw.saturating_sub(1)),
            (bar_top + barrier_h)
                .saturating_sub(1)
                .min(dh.saturating_sub(1)),
        );

        // Evanescent fill inside barrier (dots spacing ∝ tunneling decay)
        let decay = (1.0 - ctx.eased).max(0.01);
        for xi in barrier_x0..=barrier_x1.min(dw.saturating_sub(1)) {
            let d = xi.saturating_sub(barrier_x0) as f32;
            let envelope =
                (-decay * 2.5 * d / (barrier_x1.saturating_sub(barrier_x0) as f32).max(1.0)).exp();
            if (d as usize % (2.max((1.0 / envelope.max(0.1)) as usize))) == 0 {
                draw::dot(grid, xi, mid_y as usize);
            }
        }

        // Incident + reflected wave (left region, standing wave)
        let k = 4.0 * PI / dwf; // wavenumber
        let omega = k * 2.0; // angular frequency
        let refl_amp = (1.0 - ctx.eased).sqrt();
        let inc_amp = 1.0_f32;
        let wave_amp = (dhf * 0.35).max(1.0);

        let mut prev: Option<(i32, i32)> = None;
        for xi in 0..barrier_x0 {
            let xf = xi as f32;
            let incident = inc_amp * (k * xf - omega * ctx.time).sin();
            let reflected = refl_amp * (k * xf + omega * ctx.time).sin();
            let total = (incident + reflected) * wave_amp;
            let py = (mid_y as f32 - total) as i32;
            let py = py.clamp(0, dh as i32 - 1);
            draw::dot_i(grid, xi as i32, py);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, xi as i32, py);
            }
            prev = Some((xi as i32, py));
        }

        // Transmitted wave (right region)
        let trans_amp = ctx.eased.sqrt();
        let mut prev: Option<(i32, i32)> = None;
        for xi in (barrier_x1 + 1)..dw {
            let xf = xi as f32;
            let transmitted = trans_amp * (k * xf - omega * ctx.time).sin() * wave_amp;
            let py = (mid_y as f32 - transmitted) as i32;
            let py = py.clamp(0, dh as i32 - 1);
            draw::dot_i(grid, xi as i32, py);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, xi as i32, py);
            }
            prev = Some((xi as i32, py));
        }

        // Axis
        draw::hline(grid, 0, dw.saturating_sub(1), mid_y as usize);

        // Tint: transmitted side keyed by eased
        let (cw, ch) = grid.dimensions();
        let split_cell = barrier_x1 / 2;
        for cx in split_cell.min(cw)..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t * ctx.eased);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Quantum Harmonic Oscillator Eigenstates
//    |ψₙ(x)|² for the n-th energy eigenstate of a 1D QHO.
//    ψₙ(x) ∝ Hₙ(x)·exp(−x²/2) where Hₙ are Hermite polynomials.
//    Energy level n = floor(eased · N_max) — bar literally climbs the
//    ladder rung by rung.  We compute Hermite up to n≤6 via recurrence.
//    The probability density is rendered as a dot-height waveform.
// ---------------------------------------------------------------------------
struct HarmonicOscillator;
impl ProgressStyle for HarmonicOscillator {
    fn name(&self) -> &str {
        "harmonic-oscillator"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "QHO eigenstate |ψₙ|² inside its parabolic well; level n climbs the energy ladder with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let dwf = dw as f32;
        let dhf = dh as f32;

        // Energy level n (0..6)
        let n_max = 6_usize;
        let n = (ctx.eased * (n_max + 1) as f32).floor() as usize;
        let n = n.min(n_max);

        // x range over which ψₙ is significant: ±x_max
        let x_max = 3.5_f32 + n as f32 * 0.4;

        // Hermite polynomial Hₙ(x) via recurrence: H₀=1, H₁=2x, Hₙ=2x·Hₙ₋₁−2(n-1)Hₙ₋₂
        let hermite = |x: f32| -> f32 {
            if n == 0 {
                return 1.0;
            }
            let mut h_prev2 = 1.0_f32;
            let mut h_prev1 = 2.0 * x;
            if n == 1 {
                return h_prev1;
            }
            let mut h = 0.0_f32;
            for k in 2..=(n as i32) {
                h = 2.0 * x * h_prev1 - 2.0 * (k - 1) as f32 * h_prev2;
                h_prev2 = h_prev1;
                h_prev1 = h;
            }
            h
        };

        // Normalise: sample the peak density
        let mut peak = 0.0_f32;
        let norm_samples = 64_usize;
        for i in 0..norm_samples {
            let xf = -x_max + 2.0 * x_max * i as f32 / norm_samples as f32;
            let hn = hermite(xf);
            let psi = hn * (-xf * xf / 2.0).exp();
            let dens = psi * psi;
            if dens > peak {
                peak = dens;
            }
        }
        let peak = peak.max(1e-9);

        // Animated phase shimmer on top
        let phase = ctx.time * 0.8;

        // Render: for each dot column compute density → dot height
        let mut prev: Option<(i32, i32)> = None;
        for xi in 0..dw {
            let xf = -x_max + 2.0 * x_max * xi as f32 / dwf;
            let hn = hermite(xf);
            let psi = hn * (-xf * xf / 2.0).exp();
            let density = (psi * psi / peak).clamp(0.0, 1.0);
            // Add a mild shimmer based on phase (re-part of wavefunction, not probability)
            let shimmer = 1.0 + 0.08 * (2.0 * xf + phase).cos();
            let display_h = (density * shimmer.clamp(0.0, 1.1) * dhf).clamp(0.0, dhf);

            let bot = dh as i32 - 1;
            let top = (bot as f32 - display_h) as i32;
            let top = top.clamp(0, bot);
            let mid_dot = (top + bot) / 2;
            draw::dot_i(grid, xi as i32, mid_dot);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, xi as i32, mid_dot);
            }
            prev = Some((xi as i32, mid_dot));
        }

        // The confining potential V(x) ∝ x²: a dotted parabolic well.
        let bot = dh as i32 - 1;
        for xi in (0..dw).step_by(2) {
            let u = (2.0 * xi as f32 / dwf - 1.0).abs();
            let y = bot - (u * u * bot as f32 * 1.6) as i32;
            if y >= 0 {
                draw::dot_i(grid, xi as i32, y);
            }
        }
        // Energy ladder: one rung per level reached so far, on the left.
        for k in 0..=n {
            let y = bot - 2 - (k as i32 * (bot - 2)) / n_max as i32;
            draw::hline(grid, 0, 3, y.max(0) as usize);
        }

        // Horizontal baseline
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        // Tint by energy level fraction
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let level_t = n as f32 / n_max as f32;
            let col = ctx.palette.sample(t * 0.5 + level_t * 0.5);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Particle in a Box — standing modes
//    ψₙ(x) = sqrt(2/L)·sin(nπx/L),  n = 1,2,...
//    |ψₙ(x)|² = (2/L)·sin²(nπx/L) — a standing wave with n antinodes.
//    n is driven by eased: n = max(1, round(eased · N_max)).
//    The box walls are drawn on left and right; density as dot waveform.
// ---------------------------------------------------------------------------
struct ParticleInBox;
impl ProgressStyle for ParticleInBox {
    fn name(&self) -> &str {
        "particle-in-box"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Particle in a box: sin²(nπx/L) standing mode — mode n steps up with progress, walls visible"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let dhf = dh as f32;

        // Mode number n (1-based)
        let n_max = 7_usize;
        let n = (ctx.eased * n_max as f32).ceil() as usize;
        let n = n.clamp(1, n_max);

        // Box walls
        draw::vline(grid, 0, 0, dh.saturating_sub(1));
        draw::vline(grid, dw.saturating_sub(1), 0, dh.saturating_sub(1));

        // Interior x range (excluding walls, 1 dot each side)
        let x_start = 1_usize;
        let x_end = dw.saturating_sub(2);
        let box_w = x_end.saturating_sub(x_start).max(1);

        // Time-dependent phase (animated but density is |ψ|² so purely real)
        let phase = ctx.time * 1.2;

        let mut prev: Option<(i32, i32)> = None;
        for xi in x_start..=x_end {
            let xrel = (xi - x_start) as f32 / box_w as f32; // 0..1
            let density = (n as f32 * PI * xrel).sin().powi(2).clamp(0.0, 1.0);
            // Mild animated shimmer (Re part oscillates, density = time-averaged)
            let shimmer = 0.85 + 0.15 * (n as f32 * PI * xrel - phase * n as f32).sin().abs();
            let display_h = (density * shimmer * (dhf - 2.0)).clamp(0.0, dhf - 2.0);

            // Draw the wavefunction peak dot
            let base = dh.saturating_sub(2) as i32;
            let peak_dot = (base as f32 - display_h) as i32;
            let peak_dot = peak_dot.clamp(0, base);
            draw::dot_i(grid, xi as i32, peak_dot);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, xi as i32, peak_dot);
            }
            prev = Some((xi as i32, peak_dot));
        }

        // Floor of the box
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        // Tint: colour maps the density (hot = high probability)
        let (cw, ch) = grid.dimensions();
        let x_start_c = 0_usize;
        let x_end_c = cw;
        for cx in x_start_c..x_end_c {
            let xrel = cx as f32 / cw.max(1) as f32;
            let density = (n as f32 * PI * xrel).sin().powi(2).clamp(0.0, 1.0);
            let col = ctx.palette.sample(density);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Spin Precession
//    A spin arrow pivots on a point and sweeps a cone about a vertical
//    magnetic-field axis B (dashed, arrow-headed).  The cone's rim ellipse
//    and sides are drawn faintly; the arrow's azimuth φ advances with time.
//    The whole precessing spin travels along the bar with progress, its tip
//    leaving a sinusoidal trace behind it over a filled baseline.
// ---------------------------------------------------------------------------
struct SpinPrecession;
impl ProgressStyle for SpinPrecession {
    fn name(&self) -> &str {
        "spin-precession"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Spin arrow cones around a vertical B-field axis; the precessing spin travels with progress, tracing a wave behind it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (w, h) = (dw as i32, dh as i32);
        let e = ctx.eased.clamp(0.0, 1.0);
        if h < 8 || w < 16 {
            let reach = (e * dw as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, dh - 1);
            }
            return Ok(());
        }

        // Cone geometry: pivot at the bottom, rim ellipse near the top.
        let rim_y = 3.0_f32;
        let pivot_y = h - 2;
        let ra = (h as f32 * 0.5).min(w as f32 / 4.0);
        let rb = 2.0_f32;
        let cx = ra.ceil() as i32 + 1 + ((w - 3 - 2 * ra.ceil() as i32).max(0) as f32 * e) as i32;
        let cxf = cx as f32;
        let phi = ctx.time * 4.0;

        // Baseline fill and the tip's trace behind the spin.
        let back = cx - ra.ceil() as i32 - 2;
        if back > 0 {
            draw::hline(grid, 0, back as usize, (h - 1) as usize);
            let mut prev: Option<(i32, i32)> = None;
            for x in 0..=back {
                let a = phi - (cx - x) as f32 * 0.22;
                let y = (rim_y + rb * a.cos()).round() as i32;
                if let Some((px, py)) = prev {
                    line(grid, px, py, x, y);
                }
                prev = Some((x, y));
            }
            for c in 0..=(back / 2) {
                let col = ctx.palette.sample(c as f32 * 2.0 / w as f32);
                for row in [0, (h - 1) / 4] {
                    draw::tint_row(grid, row as usize, c as usize, c as usize, col);
                }
            }
        }

        // B-field axis: dashed, with an arrowhead on top.
        for y in 1..=pivot_y {
            if y % 2 == 1 {
                draw::dot_i(grid, cx, y);
            }
        }
        draw::dot_i(grid, cx, 0);
        draw::dot_i(grid, cx - 1, 1);
        draw::dot_i(grid, cx + 1, 1);

        // Cone: rim ellipse (back half sparse) and faint sides.
        let steps = (ra * 6.0) as usize + 8;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            if a.cos() < 0.0 && s % 2 == 0 {
                continue;
            }
            let x = (cxf + ra * a.sin()).round() as i32;
            let y = (rim_y + rb * a.cos()).round() as i32;
            draw::dot_i(grid, x, y);
        }
        let ri = ra.round() as i32;
        for (sx, step) in [(-ri, 3), (ri, 3)] {
            let n = (pivot_y - rim_y as i32).max(1);
            for i in (0..=n).step_by(step) {
                draw::dot_i(grid, cx + sx * (n - i) / n, rim_y as i32 + i);
            }
        }
        let cone_col = Color::rgb(110, 120, 160);
        for c in ((cx - ri) / 2).max(0)..=((cx + ri) / 2) {
            for row in 0..=(h - 1) / 4 {
                draw::tint_row(grid, row as usize, c as usize, c as usize, cone_col);
            }
        }

        // The spin itself: a thick arrow from pivot to its rim position.
        let tip_x = (cxf + ra * phi.sin()).round() as i32;
        let tip_y = (rim_y + rb * phi.cos()).round() as i32;
        line(grid, cx, pivot_y, tip_x, tip_y);
        line(grid, cx + 1, pivot_y, tip_x + 1, tip_y);
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, -1), (1, -1), (0, 1), (2, 0)] {
            draw::dot_i(grid, tip_x + dx, tip_y + dy);
        }
        draw::dot_i(grid, cx - 1, pivot_y + 1);
        draw::dot_i(grid, cx + 2, pivot_y + 1);
        let arrow_col = ctx.palette.sample(1.0);
        let steps = (pivot_y - tip_y).max(1);
        for i in 0..=steps {
            let x = cx + (tip_x - cx) * i / steps;
            let y = pivot_y - i;
            draw::tint_row(
                grid,
                (y.max(0) / 4) as usize,
                (x.max(0) / 2) as usize,
                (x.max(0) / 2) as usize,
                arrow_col,
            );
        }
        draw::tint_row(
            grid,
            (tip_y.max(0) / 4) as usize,
            (tip_x.max(0) / 2) as usize,
            (tip_x.max(0) / 2) as usize,
            Color::rgb(255, 255, 255),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Energy Level Diagram with Electron Filling
//    N_levels horizontal lines (orbitals) stacked from bottom to top.
//    Progress fills them with electrons (Aufbau principle).
//    Each electron is a small filled circle (3-dot cross) on its level.
//    Levels are drawn as horizontal line segments; filled ones are tinted.
//    eased determines how many electrons have been placed.
// ---------------------------------------------------------------------------
struct EnergyLevels;
impl ProgressStyle for EnergyLevels {
    fn name(&self) -> &str {
        "energy-levels"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Energy level diagram: Aufbau filling — electrons populate orbitals from bottom as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let dwf = dw as f32;
        let dhf = dh as f32;

        // Number of levels — scale with height
        let n_levels = (dh / 3).clamp(2, 8) as usize;
        // Each level holds 2 electrons (spin up/down)
        let n_electrons_max = n_levels * 2;
        let n_filled = (ctx.eased * n_electrons_max as f32).round() as usize;

        // Level y positions (equal spacing from bottom to top)
        // Level 0 is the lowest energy (bottom of display)
        for lv in 0..n_levels {
            let y_frac = 1.0 - (lv as f32 + 0.5) / n_levels as f32;
            let y_dot = (y_frac * (dhf - 2.0) + 1.0) as usize;
            let y_dot = y_dot.min(dh.saturating_sub(1));

            // Draw the level line (horizontal bar across 40% of width, centred)
            let lw = (dwf * 0.4).max(4.0) as usize;
            let lx0 = ((dwf - lw as f32) / 2.0) as usize;
            let lx1 = (lx0 + lw).min(dw.saturating_sub(1));
            draw::hline(grid, lx0, lx1, y_dot);

            // Draw electrons (up to 2 per level)
            let elec_base = lv * 2; // first electron index for this level
            for spin in 0..2_usize {
                let elec_idx = elec_base + spin;
                if elec_idx >= n_filled {
                    break;
                }
                // Place electron on left or right of level line
                let ex = if spin == 0 {
                    lx0.saturating_sub(3)
                } else {
                    (lx1 + 3).min(dw.saturating_sub(1))
                };
                // Cross-shaped electron marker
                draw::dot_i(grid, ex as i32, y_dot as i32);
                draw::dot_i(grid, ex as i32 - 1, y_dot as i32);
                draw::dot_i(grid, ex as i32 + 1, y_dot as i32);
                draw::dot_i(grid, ex as i32, y_dot as i32 - 1);
                draw::dot_i(grid, ex as i32, y_dot as i32 + 1);
            }

            // Energy axis tick on the left
            draw::dot(grid, 0, y_dot);
            draw::dot(grid, 1, y_dot);
        }

        // Vertical energy axis
        draw::vline(grid, 0, 0, dh.saturating_sub(1));

        // Tint: filled levels are brighter
        let (cw, ch) = grid.dimensions();
        let filled_levels = (n_filled + 1) / 2; // levels with at least one electron
        for lv in 0..filled_levels.min(n_levels) {
            let y_frac = 1.0 - (lv as f32 + 0.5) / n_levels as f32;
            let cy = (y_frac * (ch as f32 - 1.0)) as usize;
            let cy = cy.min(ch.saturating_sub(1));
            let t = lv as f32 / n_levels.max(1) as f32;
            let col = ctx.palette.sample(1.0 - t); // bottom levels warm, top cool
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), col);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Decoherence
//    An interference pattern starts crisp and washes out as the system
//    interacts with the environment (eased → 1 = fully decohered).
//    Fringe visibility V = 1 − eased.
//    Pattern: I(x) = (1/2)[1 + V·cos(k·x + phase(t))]
//    Rendered as a dot height waveform modulated by V.
//    At eased=0: vivid fringes; at eased=1: flat uniform intensity.
// ---------------------------------------------------------------------------
struct Decoherence;
impl ProgressStyle for Decoherence {
    fn name(&self) -> &str {
        "decoherence"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Decoherence: sharp interference fringes wash out to flat noise as entanglement grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let dwf = dw as f32;
        let dhf = dh as f32;

        // Fringe visibility: 1 = fully coherent, 0 = fully decohered
        let visibility = 1.0 - ctx.eased;
        // Wave vector: ~6 fringes across the bar
        let k = 6.0 * 2.0 * PI / dwf.max(1.0);
        let phase = ctx.time * 1.5;

        let mut prev: Option<(i32, i32)> = None;
        for xi in 0..dw {
            let xf = xi as f32;
            // Intensity: central uniform term + fringe term
            let intensity = 0.5 * (1.0 + visibility * (k * xf + phase).cos());
            let intensity = intensity.clamp(0.0, 1.0);
            let display_h = (intensity * (dhf - 1.0)).clamp(0.0, dhf - 1.0);
            let py = (dhf - 1.0 - display_h) as i32;
            let py = py.clamp(0, dh as i32 - 1);
            draw::dot_i(grid, xi as i32, py);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, xi as i32, py);
            }
            prev = Some((xi as i32, py));
        }

        // Zero-line (fully decohered reference)
        let zero_y = (dhf * 0.5) as usize;
        for xi in (0..dw).step_by(4) {
            draw::dot(grid, xi, zero_y.min(dh.saturating_sub(1)));
        }

        // Tint: cool when coherent, warm when decohered
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = 1.0 - visibility; // 0=coherent(start color), 1=decohered(end color)
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Quantum Random Walk
//    A 1-D lattice of sites drawn as a dotted baseline with a start marker
//    at the centre.  The walker's probability distribution is a lollipop
//    plot: the signature quantum-walk shape — two sharp ballistic peaks
//    racing outward (spreading linearly, not √n) with interference ripples
//    between them.  Peak separation grows with eased; the ripples shimmer
//    with time.
// ---------------------------------------------------------------------------
struct QuantumWalk;
impl ProgressStyle for QuantumWalk {
    fn name(&self) -> &str {
        "quantum-walk"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Quantum walk on a lattice: twin ballistic peaks race outward from the start site with progress, ripples between"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (w, h) = (dw as i32, dh as i32);
        let e = ctx.eased.clamp(0.0, 1.0);
        let base = h - 1;
        let cx = (w - 1) / 2;
        let cxf = (w - 1) as f32 / 2.0;
        // Peak distance from the start site: ballistic, ∝ progress.
        let spread = 2.0 + e * (cxf - 3.0).max(0.0);
        let max_h = (h - 2).max(1) as f32;

        // Lattice baseline: one dot per site.
        for x in (0..w).step_by(2) {
            draw::dot_i(grid, x, base);
        }
        // Start site marker.
        line(grid, cx, base, cx, base - (h / 4).max(1));
        draw::dot_i(grid, cx - 1, base - 1);
        draw::dot_i(grid, cx + 1, base - 1);

        // Probability stems on even sites.
        for x in (0..w).step_by(2) {
            let d = (x as f32 - cxf).abs();
            let p = if d > spread + 1.5 {
                0.0
            } else {
                // Sharp leading peak, rippling plateau inside it.
                let edge = (-(d - spread).powi(2) / 3.0).exp();
                let inner = if d < spread {
                    let r = (d * 0.9 - ctx.time * 3.0).sin() * 0.5 + 0.5;
                    0.12 + 0.2 * r * (d / spread)
                } else {
                    0.0
                };
                edge.max(inner)
            };
            let top = base - 1 - (p * max_h).round() as i32;
            if top < base - 1 {
                line(grid, x, base - 1, x, top);
                draw::dot_i(grid, x + 1, top);
                let col = if p > 0.6 {
                    ctx.palette.sample(1.0)
                } else {
                    ctx.palette.sample(p)
                };
                for row in (top.max(0) / 4)..=(base / 4) {
                    draw::tint_row(grid, row as usize, (x / 2) as usize, (x / 2) as usize, col);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Double Slit
//    The famous single-particle build-up.  A source on the left sends
//    wavefronts at a barrier pierced by two slits; circular wavelets spread
//    from each slit toward a detector screen on the right.  Individual hits
//    land on the screen one at a time, sampled from the two-slit intensity
//    cos²·envelope — sparse speckle early, crisp interference fringes as
//    eased → 1.
// ---------------------------------------------------------------------------
struct DoubleSlit;
impl ProgressStyle for DoubleSlit {
    fn name(&self) -> &str {
        "electron-buildup"
    }
    fn theme(&self) -> &str {
        "quantum"
    }
    fn describe(&self) -> &str {
        "Single-electron double-slit: wavelets spread from two slits and particle hits build up interference fringes with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (w, h) = (dw as i32, dh as i32);
        let hf = (h - 1) as f32;
        let cy = hf / 2.0;
        let e = ctx.eased.clamp(0.0, 1.0);

        // Geometry: source | barrier with two slits | wavelets | hit band | screen.
        let bx = ((w / 5) & !1).max(2);
        let screen_x = w - 2;
        let band_x0 = (w * 3 / 4).min(screen_x - 1);
        let slit_off = (h / 5).max(1);
        let slits = [cy as i32 - slit_off, cy as i32 + slit_off];

        // Barrier wall, open only at the slits.
        for y in 0..h {
            if slits.iter().all(|&s| y != s && y != s + 1) {
                draw::dot_i(grid, bx, y);
                draw::dot_i(grid, bx + 1, y);
            }
        }
        let wall = Color::rgb(140, 145, 165);
        for row in 0..=(h - 1) / 4 {
            draw::tint_row(
                grid,
                row as usize,
                (bx / 2) as usize,
                ((bx + 1) / 2) as usize,
                wall,
            );
        }

        // Source and plane wavefronts marching toward the barrier.
        line(grid, 0, cy as i32 - 1, 0, cy as i32 + 1);
        let gap = 6.0_f32;
        let drift = (ctx.time * 10.0) % gap;
        let mut x = 2.0 + drift;
        while (x as i32) < bx - 1 {
            for y in ((cy - hf * 0.3) as i32)..=((cy + hf * 0.3) as i32) {
                if y % 2 == 0 {
                    draw::dot_i(grid, x as i32, y);
                }
            }
            x += gap;
        }

        // Circular wavelets from each slit, clipped to the gap before the band.
        let reach = (band_x0 - bx - 6).max(1) as f32;
        let n_rings = (reach / gap) as i32 + 1;
        for &sy in &slits {
            for k in 0..n_rings {
                let r = k as f32 * gap + drift;
                if r < 1.0 || r > reach {
                    continue;
                }
                let steps = (r * 3.0) as i32 + 4;
                for s in 0..=steps {
                    let a = -PI / 2.0 + PI * s as f32 / steps as f32;
                    let px = bx + 2 + (a.cos() * r) as i32;
                    let py = sy + (a.sin() * r * 0.8) as i32;
                    if px < band_x0 && (0..h).contains(&py) && s % 3 == 0 {
                        draw::dot_i(grid, px, py);
                    }
                }
            }
        }
        let wave_col = Color::rgb(120, 150, 255);
        for row in 0..=(h - 1) / 4 {
            draw::tint_row(
                grid,
                row as usize,
                ((bx + 2) / 2) as usize,
                ((band_x0 - 1).max(0) / 2) as usize,
                wave_col,
            );
        }

        // Detector screen.
        line(grid, screen_x, 0, screen_x, h - 1);
        line(grid, screen_x + 1, 0, screen_x + 1, h - 1);

        // Particle hits: deterministic samples from the fringe intensity.
        let fringe = |y: f32| {
            let u = (y - cy) / hf.max(1.0);
            let env = (-u * u * 3.0).exp();
            (u * PI * 3.0).cos().powi(2) * env
        };
        let band_w = (screen_x - band_x0).max(1);
        let hits = (e * (band_w * h) as f32 * 0.55) as u32;
        let mut placed = 0u32;
        let mut i = 0u32;
        while placed < hits && i < hits.saturating_mul(6) + 16 {
            let y = (hashq(i * 2 + 1) * hf).round();
            if hashq(i * 2 + 7) < fringe(y) {
                let x = band_x0 + (hashq(i * 3 + 5) * band_w as f32) as i32;
                draw::dot_i(grid, x.min(screen_x - 2), y as i32);
                draw::dot_i(grid, x.min(screen_x - 2) + 1, y as i32);
                placed += 1;
            }
            i += 1;
        }
        // Fringe colour: bright where the intensity peaks.
        for row in 0..=(h - 1) / 4 {
            let yc = (row * 4 + 2) as f32;
            let col = ctx.palette.sample(fringe(yc).sqrt());
            draw::tint_row(
                grid,
                row as usize,
                (band_x0 / 2) as usize,
                (w / 2) as usize,
                col,
            );
        }
        // The newest hit flashes white.
        if hits > 0 {
            let f = (ctx.time * 12.0) as u32;
            let y = (hashq(f * 11 + 3) * hf) as i32;
            let x = band_x0 + (hashq(f * 13 + 1) * band_w as f32) as i32;
            draw::dot_i(grid, x, y);
        }
        Ok(())
    }
}

/// Deterministic hash of `n` to `[0, 1)`.
#[inline]
fn hashq(n: u32) -> f32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);
    x ^= x >> 13;
    (x % 10_000) as f32 / 10_000.0
}
