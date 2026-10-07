//! Cosmos / deep-space-phenomena progress bars.
//!
//! Ten structurally distinct styles, each modelling a different astrophysical
//! phenomenon: supernova shockwave, pulsar lighthouse, nebula condensation,
//! big-bang expansion, solar flare arc, aurora curtains, meteor shower,
//! total eclipse corona, gravitational-lens Einstein ring, cosmic-web
//! filaments, redshift wavefronts, and a quasar relativistic jet.
//!
//! All bars return `"cosmos"` from `theme()`. `ctx.eased` drives the
//! intensity / progress of the phenomenon; `ctx.time` drives animation.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic pseudo-random helpers — no external crates.
// ---------------------------------------------------------------------------

/// Cheap integer hash (Knuth multiplicative + avalanche).
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

/// Float in [0, 1) from index `n`.
fn hash_f(n: u32) -> f32 {
    (hash(n) % 10_000) as f32 / 10_000.0
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `cosmos` theme, in display order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Supernova),
        Box::new(Pulsar),
        Box::new(NebulaClouds),
        Box::new(BigBang),
        Box::new(SolarFlare),
        Box::new(AuroraCurtains),
        Box::new(MeteorShower),
        Box::new(TotalEclipse),
        Box::new(GravitationalLens),
        Box::new(CosmicWeb),
        Box::new(Redshift),
        Box::new(QuasarJet),
    ]
}

// ---------------------------------------------------------------------------
// 1 — Supernova
// ---------------------------------------------------------------------------
// Structural idea: an expanding shell of dots whose radius = eased * max_r.
// The shell itself is a thin ring (drawn at radius ± 1). A dense core shrinks
// as the shell grows, and a sparse debris halo scatters outside the shell.
// No other style here uses a growing circular shell as its primary element.

/// Supernova: shockwave shell radiates outward; radius tracks eased, debris halos outside.
struct Supernova;
impl ProgressStyle for Supernova {
    fn name(&self) -> &str {
        "supernova"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Supernova shockwave shell explodes outward; radius = eased * max; debris halos outside"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = cx.min(cy * 2).max(1) as f32;
        let shell_r = (ctx.eased * max_r) as i32;

        // Collapsing stellar core — solid disc that shrinks as shell expands.
        let core_r = ((1.0 - ctx.eased) * max_r * 0.25) as i32;
        for dy in -core_r..=core_r {
            let dx_max = ((core_r * core_r - dy * dy).max(0) as f32).sqrt() as i32;
            for dx in -dx_max..=dx_max {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Shell ring — draw dots at radius shell_r ± 1.
        if shell_r > 0 {
            let steps = (2.0 * PI * shell_r as f32 * 1.5) as usize;
            let steps = steps.max(8);
            for s in 0..steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                for dr in [-1i32, 0, 1] {
                    let r = (shell_r + dr).max(0);
                    let px = cx + (r as f32 * a.cos()) as i32;
                    let py = cy + (r as f32 * a.sin() * 0.5) as i32;
                    // Occasional gap for ring texture
                    if hash((s as u32).wrapping_add(dr.unsigned_abs() * 997)) % 5 != 0 {
                        draw::dot_i(grid, px, py);
                    }
                }
            }
        }

        // Debris: sparse dots between core and shell, animated via time.
        let debris_count = 40u32;
        for i in 0..debris_count {
            let angle = hash_f(i) * 2.0 * PI;
            let r_frac = hash_f(i + 1000);
            let r = (r_frac * shell_r as f32 * 0.9) as i32;
            if r <= core_r {
                continue;
            }
            // Drift outward at speed proportional to position
            let drift = (ctx.time * 0.4 * r_frac) as i32;
            let r_d = (r + drift).min(shell_r);
            let px = cx + (r_d as f32 * angle.cos()) as i32;
            let py = cy + (r_d as f32 * angle.sin() * 0.5) as i32;
            if hash(i.wrapping_add((ctx.time * 3.0) as u32)) % 3 != 0 {
                draw::dot_i(grid, px, py);
            }
        }

        // Tint: hot white core → violet shell
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0).max(1.0);
            let t = 1.0 - dist;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}
// ---------------------------------------------------------------------------
// 2 — Pulsar
// ---------------------------------------------------------------------------
// Structural idea: a spinning neutron star throws two opposed cone beams that
// sweep round like a lighthouse (angle driven by time). Below it a radio
// telescope trace records the sharp periodic pulses; the trace is drawn out
// to eased, so the pulse train *is* the progress bar.

/// Pulsar: twin lighthouse cones sweep round a neutron star; pulse trace = eased.
struct Pulsar;
impl ProgressStyle for Pulsar {
    fn name(&self) -> &str {
        "pulsar"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Pulsar: neutron star spins twin lighthouse beam cones; a radio pulse trace below records ticks out to eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;

        // Reserve the bottom rows for the pulse trace when there is room.
        let trace_h = if hi >= 12 { 4 } else { 0 };
        let cx = wi / 2;
        let cy = ((hi - trace_h - 1) / 2).max(0);
        let max_r = (w as f32) / 2.0;

        // Twin cones, one each side, spinning about the vertical axis: seen
        // side-on their projected length is R·|cos φ| and the magnetic tilt
        // rocks them up/down, so they stretch, shrink and flash past us.
        let spin = ctx.time * 2.4;
        let reach = max_r * spin.cos().abs();
        let tilt = spin.sin() * 0.18;
        let floor = hi - trace_h;
        for dir in [-1i32, 1] {
            let reach_i = reach as i32;
            let mut r = 3i32;
            while r < reach_i {
                let rf = r as f32;
                let ymid = cy as f32 + rf * tilt * dir as f32;
                let half = (0.6 + rf * 0.11).round() as i32;
                for k in -half..=half {
                    let edge = k.abs() == half;
                    // Edges solid, interior a sparse moving shimmer.
                    if !edge && (r + k - (ctx.time * 12.0) as i32).rem_euclid(3) != 0 {
                        continue;
                    }
                    let py = ymid.round() as i32 + k;
                    if py < floor {
                        draw::dot_i(grid, cx + dir * r, py);
                    }
                }
                r += 1;
            }
        }

        // Neutron star: solid core; it flares when a beam points at us.
        let flash = spin.cos().abs() < 0.3;
        let star_r = if hi >= 12 { 2 } else { 1 } + i32::from(flash);
        if flash {
            for &(dx, dy) in &[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
            ] {
                draw::dot_i(grid, cx + dx * (star_r + 2), cy + dy * (star_r + 1));
            }
        }
        for dy in -star_r..=star_r {
            for dx in -star_r..=star_r {
                if dx * dx + dy * dy <= star_r * star_r + star_r {
                    draw::dot_i(grid, cx + dx, cy + dy);
                }
            }
        }

        // Radio pulse trace: flat baseline with a sharp spike every period.
        let mut trace_cols = 0usize;
        if trace_h > 0 {
            let base = hi - 1;
            let period = 7;
            let filled = (ctx.eased * wi as f32) as i32;
            trace_cols = filled.max(0) as usize;
            for x in 0..filled {
                let ph = x % period;
                let y = match ph {
                    3 => base - 3,
                    2 | 4 => base - 1,
                    _ => base,
                };
                draw::dot_i(grid, x, y);
                if ph == 3 {
                    draw::vline(grid, x as usize, (base - 2) as usize, base as usize);
                }
            }
            // Unrecorded remainder: faint dotted baseline.
            let mut x = filled + 1;
            while x < wi {
                draw::dot_i(grid, x, base);
                x += 3;
            }
        }

        // Tint: beams in the start colour, star in the end colour, trace graded.
        let (cells_w, cells_h) = grid.dimensions();
        let trace_cell = if trace_h > 0 { cells_h - 1 } else { cells_h };
        for cy_c in 0..trace_cell {
            draw::tint_row(grid, cy_c, 0, cells_w.saturating_sub(1), ctx.palette.start);
        }
        let star_cell_y = (cy.max(0) as usize) / 4;
        let star_cell_x = (cx.max(0) as usize) / 2;
        draw::tint_row(
            grid,
            star_cell_y,
            star_cell_x.saturating_sub(1),
            star_cell_x + 1,
            ctx.palette.end,
        );
        if trace_h > 0 {
            for cx_c in 0..cells_w {
                let col = if cx_c * 2 < trace_cols {
                    ctx.palette.sample(cx_c as f32 / cells_w.max(1) as f32)
                } else {
                    ctx.palette.track
                };
                draw::tint_row(grid, cells_h - 1, cx_c, cx_c, col);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3 — Nebula cloud condensing
// ---------------------------------------------------------------------------
// Structural idea: a billowing gas cloud built from overlapping soft blobs,
// rendered with an ordered (Bayer) dither so the core reads dense and the
// edges wispy. A sinuous dark dust lane cuts through it. The cloud lights up
// left → right as eased grows; newborn stars ignite inside the lit gas and
// twinkle, and the gas billows slowly with time.

/// Nebula: dithered gas cloud with a dust lane and newborn stars; lit extent = eased.
struct NebulaClouds;
impl ProgressStyle for NebulaClouds {
    fn name(&self) -> &str {
        "nebula-clouds"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Nebula: billowing dithered gas cloud with a dark dust lane and twinkling newborn stars; lit extent = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        // 4×4 Bayer thresholds for ordered dithering.
        const BAYER: [[f32; 4]; 4] = [
            [0.0, 8.0, 2.0, 10.0],
            [12.0, 4.0, 14.0, 6.0],
            [3.0, 11.0, 1.0, 9.0],
            [15.0, 7.0, 13.0, 5.0],
        ];

        // Cloud lobes spaced along the bar with hashed vertical jitter.
        let lobes = 6u32;
        let rx = wf / lobes as f32 * 0.9;
        let ry = (hf * 0.3).max(1.0);
        let lit_edge = 8.0 + ctx.eased * wf * 1.1;
        let drift = ctx.time * 0.6;

        let stars: Vec<(i32, i32)> = (0u32..5)
            .map(|i| {
                let sx = ((i as f32 + 0.3 + hash_f(i + 200) * 0.4) / 5.0 * wf) as i32;
                let sy = (hf * (0.25 + 0.5 * hash_f(i + 300))) as i32;
                (sx, sy)
            })
            .filter(|&(sx, _)| (sx as f32) < lit_edge - 6.0)
            .collect();

        for y in 0..h {
            for x in 0..w {
                if stars
                    .iter()
                    .any(|&(sx, sy)| (x as i32 - sx).abs() <= 3 && (y as i32 - sy).abs() <= 3)
                {
                    continue;
                }
                let xf = x as f32;
                let yf = y as f32;
                let mut d = 0.0_f32;
                for i in 0..lobes {
                    let lx = (i as f32 + 0.5) / lobes as f32 * wf + (drift + i as f32).sin() * 2.0;
                    let ly = hf / 2.0 + (hash_f(i + 40) - 0.5) * hf * 0.5;
                    let dx = (xf - lx) / rx;
                    let dy = (yf - ly) / ry;
                    d += (-(dx * dx + dy * dy)).exp() * (0.45 + 0.55 * hash_f(i + 80));
                }
                // Billowing texture.
                d += 0.18 * ((xf * 0.31 + drift * 2.0).sin() * (yf * 0.7 - drift).cos());
                // Dark dust lane.
                let lane = hf / 2.0 + (xf * 0.09 + 1.0).sin() * hf * 0.22;
                if (yf - lane).abs() < 1.2 {
                    d *= 0.25;
                }
                // Reveal left → right with a soft leading edge.
                let reveal = ((lit_edge - xf) / 10.0).clamp(0.0, 1.0);
                let thresh = (BAYER[y % 4][x % 4] + 0.5) / 16.0;
                if d * reveal > thresh {
                    draw::dot(grid, x, y);
                }
            }
        }

        // Newborn stars: bright crosses inside the lit gas, twinkling, each
        // carved out of the gas so it reads against the cloud.
        for &(sx, sy) in &stars {
            let arm = 1 + i32::from((ctx.time * 3.0 + sx as f32).sin() > 0.3);
            for k in -arm..=arm {
                draw::dot_i(grid, sx + k, sy);
                draw::dot_i(grid, sx, sy + k);
            }
        }

        // Tint: gradient across the cloud.
        let (cells_w, cells_h) = grid.dimensions();
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4 — Big Bang expansion
// ---------------------------------------------------------------------------
// Structural idea: all dots radiate OUTWARD from a single origin point as
// eased increases. At progress=0 everything is at the singularity; at
// progress=1 dots fill the whole canvas. Radial particle positions scale
// linearly with eased, animated with a slow drift via time.

/// Big Bang: particles radiate from singularity; position = eased × final_coords.
struct BigBang;
impl ProgressStyle for BigBang {
    fn name(&self) -> &str {
        "big-bang"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Big-bang singularity explodes: all particles radiate outward, position scales with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;

        // Singularity flash at t=0: bright core when eased is small.
        if ctx.eased < 0.05 {
            draw::dot_i(grid, cx as i32, cy as i32);
            return Ok(());
        }

        let particle_count = 120u32;
        for i in 0..particle_count {
            // Each particle has a stable random target position on the canvas edge.
            let target_angle = hash_f(i) * 2.0 * PI;
            // Target distance: between half-way and the edge.
            let target_r_frac = 0.5 + hash_f(i + 500) * 0.5;
            let max_half = cx.min(cy * 1.8);
            let target_r = target_r_frac * max_half;

            // Current position = origin + eased * (target - origin)
            let r = ctx.eased * target_r;

            // Slow thermal drift via time.
            let drift_a = target_angle + (ctx.time * 0.05 * hash_f(i + 200) * 2.0 - 0.05);
            let px = cx + r * drift_a.cos();
            let py = cy + r * drift_a.sin() * 0.55;

            // Draw particle; brighter near leading edge.
            draw::dot_i(grid, px as i32, py as i32);
            if hash(i.wrapping_add(17)) % 4 == 0 {
                draw::dot_i(grid, px as i32 + 1, py as i32);
            }
        }

        // Tint: hot centre → cool edge
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0).max(1.0);
            let t = 1.0 - dist * 0.9;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5 — Solar flare arc
// ---------------------------------------------------------------------------
// Structural idea: the curved limb of the Sun fills the bottom of the bar
// (solid edge over a granulated surface). A coronal loop arches off it as a
// thick plasma arc whose height and footpoint spread grow with eased; bright
// knots of plasma run along the loop with time, and spicules flicker on the
// limb.

/// Solar flare: a plasma loop arches off the Sun's limb; height = eased.
struct SolarFlare;
impl ProgressStyle for SolarFlare {
    fn name(&self) -> &str {
        "solar-flare"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Solar flare: a plasma loop with racing knots arches off the Sun's curved limb; loop height and span = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;
        let hi = h as i32;
        let cx = wf / 2.0;

        // Limb: a large circle whose crown pokes up from the bottom edge.
        let crown = (hf * 0.45).max(1.0);
        let big_r = wf * 2.5;
        let limb_y = |x: f32| -> f32 {
            let dx = x - cx;
            let under = (big_r * big_r - dx * dx).max(0.0).sqrt();
            hf - crown + (big_r - under)
        };

        // Surface: solid limb edge, granulated interior.
        for x in 0..w {
            let ly = limb_y(x as f32).round() as i32;
            draw::dot_i(grid, x as i32, ly);
            for y in (ly + 1)..hi {
                let g = hash((x as u32).wrapping_mul(97) ^ (y as u32).wrapping_mul(31));
                if (x + y as usize) % 2 == 0 || g % 3 == 0 {
                    draw::dot_i(grid, x as i32, y);
                }
            }
            // Spicules: short flickering hairs on the limb.
            let sp = hash((x as u32).wrapping_add((ctx.time * 5.0) as u32 * 131));
            if x % 5 == 0 && sp % 3 == 0 {
                draw::dot_i(grid, x as i32, ly - 1);
            }
        }

        // Coronal loop: half-ellipse between two footpoints on the limb.
        let span = 8.0 + ctx.eased * (wf * 0.32);
        let foot_l = cx - span / 2.0;
        let foot_r = cx + span / 2.0;
        let base_y = limb_y(cx);
        let height = 2.0 + ctx.eased * (base_y - 1.0).max(0.0);
        let steps = (span * 1.5) as i32 + 8;
        let mut pts = Vec::with_capacity(steps as usize + 1);
        for s in 0..=steps {
            let a = PI * s as f32 / steps as f32;
            let x = cx - (span / 2.0) * a.cos();
            let y = limb_y(x) - height * a.sin();
            pts.push((x.round() as i32, y.round() as i32));
        }
        for &(x, y) in &pts {
            draw::dot_i(grid, x, y);
            draw::dot_i(grid, x, y + 1);
        }
        // Plasma knots racing up one leg and down the other.
        let n = pts.len().max(1);
        for k in 0..3 {
            let idx = ((ctx.time * 30.0) as usize + k * n / 3) % n;
            let (x, y) = pts[idx];
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    draw::dot_i(grid, x + dx, y + dy);
                }
            }
        }
        // Footpoint brightenings.
        for fx in [foot_l, foot_r] {
            let fy = limb_y(fx).round() as i32;
            draw::hline(
                grid,
                fx.max(1.0) as usize - 1,
                fx.max(0.0) as usize + 1,
                (fy - 1).max(0) as usize,
            );
        }

        // Tint: surface in the end colour, loop/corona in the start colour.
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let ly = limb_y(cx_c as f32 * 2.0 + 1.0);
            for cy_c in 0..cells_h {
                let top = cy_c as f32 * 4.0;
                let col = if top + 3.0 >= ly {
                    ctx.palette.end
                } else {
                    ctx.palette.start
                };
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6 — Aurora curtains
// ---------------------------------------------------------------------------
// Structural idea: vertical columns of dots whose height is modulated by a
// sine-wave, creating "curtains" that ripple horizontally via time.
// eased controls how many columns are lit (curtain extent).
// The ripple phase varies per column — distinct from any horizontal bar style.

/// Aurora: vertical curtain columns ripple via sine; extent = eased columns.
struct AuroraCurtains;
impl ProgressStyle for AuroraCurtains {
    fn name(&self) -> &str {
        "aurora-curtains"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Aurora curtains: vertical sine-sheet columns ripple via time; extent = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let lit_cols = ((ctx.eased * w as f32) as usize).min(w);

        for x in 0..lit_cols {
            // Each column has a phase offset based on position.
            let col_phase = x as f32 * 0.3;
            // Height envelope: sine wave scrolling over time.
            let envelope = 0.5 + 0.5 * (ctx.time * 1.8 + col_phase).sin();
            // Secondary ripple for width texture.
            let secondary = 0.2 * (ctx.time * 3.1 + col_phase * 1.7).sin();
            let col_h_frac = (envelope + secondary).clamp(0.1, 1.0);
            let col_h = (col_h_frac * h as f32) as usize;

            // Curtain hangs from the top.
            let y0 = 0usize;
            let y1 = col_h.min(h).saturating_sub(1);
            for y in y0..=y1 {
                // Density fades toward the bottom of each curtain.
                let fade = 1.0 - (y as f32 / col_h.max(1) as f32);
                if hash(
                    (x as u32)
                        .wrapping_mul(17)
                        .wrapping_add(y as u32 * 31)
                        .wrapping_add((ctx.time * 8.0) as u32),
                ) % 8
                    < (fade * 7.5 + 0.5) as u32
                {
                    draw::dot(grid, x, y);
                }
            }
        }

        // Tint: green → blue → violet across width (aurora spectrum).
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}
// ---------------------------------------------------------------------------
// 7 — Meteor shower
// ---------------------------------------------------------------------------
// Structural idea: long diagonal streaks with bright 2×2 heads and tapering
// tails rain from top-right to bottom-left over a twinkling star field. Each
// meteor loops on its own phase; the number in the sky grows with eased, and
// a horizon line along the bottom fills to eased.

/// Meteor shower: streaking meteors rain down; count and horizon fill = eased.
struct MeteorShower;
impl ProgressStyle for MeteorShower {
    fn name(&self) -> &str {
        "meteor-shower"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Meteor shower: bright-headed streaks with tapering tails rain across the stars; count and horizon fill with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;
        let horizon = hi - 1;

        // Background star field (stable, sparse, gently twinkling).
        for i in 0u32..24 {
            let sx = (hash_f(i + 9000) * w as f32) as i32;
            let sy = (hash_f(i + 9100) * (hi - 2).max(1) as f32) as i32;
            if hash(i.wrapping_add((ctx.time * 2.0) as u32)) % 5 != 0 {
                draw::dot_i(grid, sx, sy);
            }
        }

        // Meteors: each travels (−2, +1) dots per step along its own loop.
        let max_meteors = 12u32;
        let active = 3 + (ctx.eased * (max_meteors - 3) as f32) as u32;
        let period = (hi + 24).max(1);
        for m in 0..active.min(max_meteors) {
            let speed = 10.0 + hash_f(m + 300) * 8.0;
            let trail = 10 + (hash_f(m + 600) * 10.0) as i32;
            let entry_x = 2 * hi + (hash_f(m) * wi as f32) as i32;
            let s = ((ctx.time * speed) as i32
                + ((m as f32 * 0.618).fract() * period as f32) as i32)
                .rem_euclid(period);
            let head_x = entry_x - 2 * s;
            let head_y = s - 3;

            // Tail: solid near the head, broken further back.
            for k in 1..trail {
                if k > trail / 2 && k % 2 == 1 {
                    continue;
                }
                let tx = head_x + 2 * k;
                let ty = head_y - k;
                if ty < horizon {
                    draw::dot_i(grid, tx, ty);
                    draw::dot_i(grid, tx - 1, ty);
                }
            }
            // Bright head: 2×2 block.
            if head_y < horizon {
                draw::dot_i(grid, head_x, head_y);
                draw::dot_i(grid, head_x - 1, head_y);
                draw::dot_i(grid, head_x, head_y + 1);
                draw::dot_i(grid, head_x - 1, head_y + 1);
            }
        }

        // Horizon: solid to eased, dotted beyond.
        let filled = (ctx.eased * wi as f32) as i32;
        for x in 0..wi {
            if x < filled || x % 3 == 0 {
                draw::dot_i(grid, x, horizon);
            }
        }

        // Tint: cool sky at top, warmer toward the horizon.
        let (cells_w, cells_h) = grid.dimensions();
        for cy_c in 0..cells_h {
            let t = 1.0 - cy_c as f32 / cells_h.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cells_w.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8 — Total eclipse
// ---------------------------------------------------------------------------
// Structural idea: TWO discs — a star (bright, fixed left-of-center) and
// an occluder moon (moving right-to-left across the star as eased goes 0→1).
// At mid-eclipse a corona ring of dots surrounds the star's limb where the
// moon occluder covers it. No other style has two interacting geometric bodies.

/// Total eclipse: moon disc occluder crosses star limb; corona ring appears at totality.
struct TotalEclipse;
impl ProgressStyle for TotalEclipse {
    fn name(&self) -> &str {
        "total-eclipse"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Total eclipse: the dark Moon slides across the round Sun; corona streamers flare out at totality"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let star_cx = (w / 2) as i32;
        let star_cy = (h / 2) as i32;
        let star_r = ((h / 2).saturating_sub(1).max(2)) as i32;

        // Moon starts at the right edge, crosses the star, ends at the left
        // edge. At eased=0.5 the centres align (totality). Travel is capped
        // at the grid width so the moon limb stays visible at both extremes
        // and the render always reflects progress.
        let travel = w as i32;
        let moon_cx = star_cx + travel / 2 - (ctx.eased * travel as f32) as i32;
        let moon_cy = star_cy;
        let moon_r = (star_r as f32 * 0.92) as i32;

        // Draw stellar disc (skip pixels covered by moon).
        let steps = 72usize;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            for dr in 0..=star_r {
                let px = star_cx + (dr as f32 * a.cos()) as i32;
                let py = star_cy + (dr as f32 * a.sin()) as i32;
                // Check if this dot is inside the moon disc.
                let ddx = px - moon_cx;
                let ddy = py - moon_cy;
                let in_moon = ddx * ddx + ddy * ddy <= moon_r * moon_r;
                if !in_moon {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Moon disc (always drawn, covers star).
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            for dr in 0..=moon_r {
                let px = moon_cx + (dr as f32 * a.cos()) as i32;
                let py = moon_cy + (dr as f32 * a.sin()) as i32;
                // We draw only the outline to leave the moon "dark" (no dots inside).
                if dr == moon_r || dr == moon_r - 1 {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Corona streamers: radial rays that grow as the Moon covers the Sun
        // and flicker with time.
        // Overlap of the two discs: 0 apart, 1 at totality.
        let overlap =
            (1.0 - (moon_cx - star_cx).abs() as f32 / (2 * star_r).max(1) as f32).clamp(0.0, 1.0);
        let rays = 16usize;
        let max_ray = (overlap * overlap * 3.0 * star_r as f32) as i32;
        for s in 0..rays {
            let a = s as f32 / rays as f32 * 2.0 * PI + 0.1;
            let flick = hash((s as u32).wrapping_add((ctx.time * 5.0) as u32 * 31)) % 4;
            let len = (max_ray - flick as i32 * 2).max(0);
            for r in (star_r + 2)..=(star_r + 1 + len) {
                draw::dot_i(
                    grid,
                    star_cx + (r as f32 * a.cos()).round() as i32,
                    star_cy + (r as f32 * a.sin()).round() as i32,
                );
            }
        }

        // Eclipse track: a thin baseline along the bottom edge fills
        // left-to-right with eased, keeping progress legible in monochrome.
        let track_x = ((ctx.eased * (w - 1) as f32) as usize).min(w - 1);
        if track_x > 0 {
            draw::hline(grid, 0, track_x, h - 1);
        }

        // Tint palette
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9 — Gravitational lensing
// ---------------------------------------------------------------------------
// Structural idea: dots from a "background" grid are displaced radially
// outward from a central lens mass, creating an Einstein-ring arc pattern.
// The ring radius grows with eased. Pure dot-displacement geometry — unlike
// any other style which has no concept of optical warping.

/// Gravitational lens: background light bends around a mass; Einstein ring at eased radius.
struct GravitationalLens;
impl ProgressStyle for GravitationalLens {
    fn name(&self) -> &str {
        "grav-lens"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Einstein ring arc: background dots deflect around a central mass; ring radius = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let max_r = cx.min(cy * 1.8);
        // Einstein ring radius grows with eased.
        let ring_r = ctx.eased * max_r * 0.85;

        // Draw the Einstein ring arc (full ring at 100%, partial arc otherwise).
        let ring_steps = 80usize;
        let arc_frac = ctx.eased.min(1.0);
        let lit_steps = (arc_frac * ring_steps as f32) as usize;
        for s in 0..lit_steps.min(ring_steps) {
            let a = s as f32 / ring_steps as f32 * 2.0 * PI;
            // Ring thickness: 2 dots.
            for dr in [-1i32, 0, 1] {
                let r = (ring_r as i32 + dr).max(0) as f32;
                let px = cx + r * a.cos();
                let py = cy + r * a.sin() * 0.5;
                if hash((s as u32).wrapping_add(dr.unsigned_abs() * 199)) % 3 != 0 {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Lensing smear arcs — shorter arcs between ring and lens mass showing
        // deflected background sources.
        let source_count = 6u32;
        for src in 0..source_count {
            let src_angle = hash_f(src) * 2.0 * PI;
            let src_r = max_r * (0.5 + hash_f(src + 100) * 0.45);
            // Source position (outside ring).
            let _sx = cx + src_r * src_angle.cos();
            let _sy = cy + src_r * src_angle.sin() * 0.5;

            // Arc of deflected image: a short curve at ring_r, on the far side.
            let arc_center_a = src_angle + PI; // opposite side from source
            let arc_half = 0.25;
            let arc_n = 12usize;
            for k in 0..arc_n {
                let t = k as f32 / arc_n.saturating_sub(1).max(1) as f32;
                let a = (arc_center_a - arc_half) + t * 2.0 * arc_half;
                let r = ring_r * (0.9 + 0.15 * (PI * t).sin());
                let px = cx + r * a.cos();
                let py = cy + r * a.sin() * 0.5;
                if ctx.eased > hash_f(src + 200) * 0.5 {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Central lens mass: compact dot cluster.
        for dy in -1i32..=1 {
            draw::dot_i(grid, cx as i32, cy as i32 + dy);
        }
        draw::dot_i(grid, cx as i32 - 1, cy as i32);
        draw::dot_i(grid, cx as i32 + 1, cy as i32);

        // Tint
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0).max(1.0);
            let t = 1.0 - dist * 0.6;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10 — Cosmic web
// ---------------------------------------------------------------------------
// Structural idea: a set of stable node positions connected by line filaments.
// Filaments appear one by one as eased grows (like galaxy filaments forming).
// Nodes pulse in brightness via time. Structurally: graph edges, not rings,
// not shells, not columns, not sweeping beams.

/// Cosmic web: galaxy nodes connected by filaments that appear as eased grows.
struct CosmicWeb;
impl ProgressStyle for CosmicWeb {
    fn name(&self) -> &str {
        "cosmic-web"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Cosmic web: node galaxies connected by filaments appearing as eased grows; nodes pulse"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        const NUM_NODES: u32 = 10;
        // Stable node positions (hash-seeded, small margin from edges).
        let nodes: Vec<(i32, i32)> = (0..NUM_NODES)
            .map(|i| {
                let nx = (hash_f(i) * (w.saturating_sub(4)) as f32 + 2.0) as i32;
                let ny = (hash_f(i + 200) * (h.saturating_sub(2)) as f32 + 1.0) as i32;
                (nx, ny)
            })
            .collect();

        // Filaments: connect each node to its two nearest by index
        // (i → i+1, i → i+2 wrapping) — gives a sparse web structure.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for i in 0..NUM_NODES as usize {
            edges.push((i, (i + 1) % NUM_NODES as usize));
            edges.push((i, (i + 2) % NUM_NODES as usize));
            // One cross-brace to make it feel 3-D.
            edges.push((i, (i + NUM_NODES as usize / 2) % NUM_NODES as usize));
        }
        // Remove exact duplicates (normalise so a < b).
        for (a, b) in &mut edges {
            if *a > *b {
                std::mem::swap(a, b);
            }
        }
        edges.sort_unstable();
        edges.dedup();

        let lit_edges = (ctx.eased * edges.len() as f32) as usize;

        // Draw revealed filaments.
        for &(a, b) in edges.iter().take(lit_edges) {
            let (ax, ay) = nodes[a];
            let (bx, by) = nodes[b];
            let dx = (bx - ax).abs();
            let dy = (by - ay).abs();
            let steps = dx.max(dy).max(1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = ax + ((bx - ax) as f32 * t) as i32;
                let py = ay + ((by - ay) as f32 * t) as i32;
                // Filament has gaps — sparse, like gas strands.
                if hash(
                    (s as u32)
                        .wrapping_mul(a as u32 * 13 + b as u32 * 7 + 1)
                        .wrapping_add((ctx.time * 2.0) as u32),
                ) % 4
                    != 0
                {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Node dots — bright cluster, pulse via time.
        for (i, &(nx, ny)) in nodes.iter().enumerate() {
            let pulse = (ctx.time * 1.5 + i as f32 * 0.9).sin() * 0.5 + 0.5;
            // Cross cluster; size modulated by pulse.
            let size = if pulse > 0.5 { 2i32 } else { 1i32 };
            for dy in -size..=size {
                for dx in -size..=size {
                    if dx.abs() + dy.abs() <= size {
                        draw::dot_i(grid, nx + dx, ny + dy);
                    }
                }
            }
        }

        // Tint with palette across width.
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11 — Redshift wavefronts
// ---------------------------------------------------------------------------
// Structural idea: a spiral galaxy recedes to the right as eased grows. The
// light wave it sends back to the observer's dish on the left is stretched:
// wavelength λ = λ₀·(1 + z) with z rising with eased, and the wave scrolls
// with time. Along the bottom a spectrum strip shows an absorption line
// sliding redward from its rest-frame tick — the measurement itself.

/// Redshift: receding galaxy's light wave stretches; spectral line shifts = eased.
struct Redshift;
impl ProgressStyle for Redshift {
    fn name(&self) -> &str {
        "redshift"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Redshift: a receding spiral galaxy's light wave stretches toward the observer; spectral line slides redward with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let wi = w as i32;
        let hi = h as i32;
        let spec_y = hi - 1;
        let sky_h = if hi >= 12 { hi - 3 } else { hi };
        let cy = (sky_h - 1) / 2;

        // Galaxy recedes rightward with progress.
        let gx = (wf * 0.45 + ctx.eased * (wf * 0.55 - 9.0).max(0.0)) as i32;
        let disk_a = 6.0_f32;
        let disk_b = (sky_h as f32 / 4.0).max(1.0);
        // Spiral arms: two log-spiral-ish sweeps around the bulge.
        let spin = ctx.time * 0.8;
        for arm in 0..2 {
            for s in 0..24 {
                let t = s as f32 / 24.0;
                let a = spin + arm as f32 * PI + t * 3.5;
                let r = 1.5 + t * disk_a;
                draw::dot_i(
                    grid,
                    gx + (r * a.cos()).round() as i32,
                    cy + (r * a.sin() * disk_b / disk_a).round() as i32,
                );
            }
        }
        for dy in -1i32..=1 {
            for dx in -2i32..=2 {
                draw::dot_i(grid, gx + dx, cy + dy);
            }
        }

        // Observer's dish on the left, facing the galaxy.
        for k in -3i32..=3 {
            draw::dot_i(grid, 1 + (k * k) / 4, cy + k);
        }
        draw::hline(grid, 2, 4, cy.max(0) as usize);

        // The light wave: λ grows with eased (z from 0 to 2).
        let lambda = 6.0 * (1.0 + 2.0 * ctx.eased);
        let amp = (sky_h as f32 / 2.0 - 1.0).clamp(0.0, 4.0);
        let x_end = gx - disk_a as i32 - 2;
        let mut prev_y: Option<i32> = None;
        for x in 6..x_end.max(6) {
            let ph = 2.0 * PI * ((gx - x) as f32 / lambda) + ctx.time * 6.0;
            let y = cy + (amp * ph.sin()).round() as i32;
            // Bridge steep steps so the wave stays a continuous line.
            if let Some(py) = prev_y {
                if (py - y).abs() > 1 {
                    draw::vline(
                        grid,
                        x as usize,
                        py.min(y).max(0) as usize,
                        py.max(y).max(0) as usize,
                    );
                }
            }
            draw::dot_i(grid, x, y);
            prev_y = Some(y);
        }

        // Spectrum strip with an absorption line shifting redward.
        let mut line_x = 0;
        if hi >= 12 {
            let rest_x = (wf * 0.2) as i32;
            line_x = rest_x + (ctx.eased * wf * 0.65) as i32;
            for x in 0..wi {
                if (x - line_x).abs() > 1 {
                    draw::dot_i(grid, x, spec_y);
                    draw::dot_i(grid, x, spec_y - 1);
                }
            }
            draw::dot_i(grid, rest_x, spec_y - 2);
        }

        // Tint: wave shifts toward the end colour with z; spectrum graded.
        let (cells_w, cells_h) = grid.dimensions();
        let wave_col = ctx.palette.sample(ctx.eased);
        let sky_cells = if hi >= 12 { cells_h - 1 } else { cells_h };
        for cy_c in 0..sky_cells {
            draw::tint_row(grid, cy_c, 0, cells_w, wave_col);
        }
        let g_lo = ((gx - 7).max(0) as usize) / 2;
        let g_hi = ((gx + 7).max(0) as usize) / 2;
        for cy_c in 0..sky_cells {
            draw::tint_row(grid, cy_c, g_lo, g_hi, ctx.palette.end);
        }
        if hi >= 12 {
            for cx_c in 0..cells_w {
                let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
                let col = if (cx_c as i32 * 2 - line_x).abs() <= 2 {
                    ctx.palette.track
                } else {
                    ctx.palette.sample(t)
                };
                draw::tint_row(grid, cells_h - 1, cx_c, cx_c, col);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12 — Quasar jet
// ---------------------------------------------------------------------------
// Structural idea: a bright nucleus inside an edge-on accretion disc (a tall
// thin ring) fires two opposed relativistic jets out along the long axis.
// Jet length = eased * half-width; knots stream outward with time and each
// jet ends in a bow-shock radio lobe that travels out with progress.

/// Quasar jet: twin relativistic jets fire from a disc-wrapped nucleus; length = eased.
struct QuasarJet;
impl ProgressStyle for QuasarJet {
    fn name(&self) -> &str {
        "quasar-jet"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Quasar: edge-on accretion disc around a blazing nucleus fires twin jets with streaming knots; jet length = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;
        let cx = wi / 2;
        let cy = (hi - 1) / 2;
        let max_len = (cx - 7).max(1);
        let len = (ctx.eased * max_len as f32) as i32;

        // Edge-on accretion disc: a tall, thin ring around the nucleus.
        let disc_b = (hi / 2 - 1).max(1) as f32;
        let disc_a = 3.0_f32;
        let steps = 40;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            draw::dot_i(
                grid,
                cx + (disc_a * a.cos()).round() as i32,
                cy + (disc_b * a.sin()).round() as i32,
            );
        }

        // Nucleus: solid core.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }
        draw::dot_i(grid, cx, cy - 2);
        draw::dot_i(grid, cx, cy + 2);

        for dir in [-1i32, 1] {
            // Jet body: a narrow cone, solid spine with flaring edges.
            for r in 4..=len + 3 {
                let x = cx + dir * r;
                let half = (r as f32 * 0.06) as i32;
                draw::dot_i(grid, x, cy);
                if r % 2 == 0 {
                    draw::dot_i(grid, x, cy - 1 - half);
                    draw::dot_i(grid, x, cy + 1 + half);
                }
            }
            // Knots streaming outward along the jet.
            if len > 4 {
                let spacing = 9;
                let off = (ctx.time * 14.0) as i32;
                let mut k = (off % spacing) + 4;
                while k < len + 3 {
                    let x = cx + dir * k;
                    for dy in -1i32..=1 {
                        draw::dot_i(grid, x, cy + dy);
                        draw::dot_i(grid, x + dir, cy + dy);
                    }
                    k += spacing;
                }
            }
            // Bow-shock radio lobe at the jet head.
            if len > 2 {
                let tip = cx + dir * (len + 3);
                let lobe_r = 2.0 + (len as f32 / max_len as f32) * (hi as f32 / 2.0 - 3.0).max(0.0);
                let steps = 16;
                for s in 0..=steps {
                    let a = -PI / 2.0 + PI * s as f32 / steps as f32;
                    draw::dot_i(
                        grid,
                        tip - dir * 2 + dir * (lobe_r * 0.6 * a.cos()).round() as i32,
                        cy + (lobe_r * a.sin()).round() as i32,
                    );
                }
            }
        }

        // Tint: jets graded outward along the palette; disc/core in the end colour.
        let (cells_w, cells_h) = grid.dimensions();
        let ccx = cells_w / 2;
        for cx_c in 0..cells_w {
            let d = (cx_c as f32 - ccx as f32).abs() / ccx.max(1) as f32;
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, ctx.palette.sample(1.0 - d));
            }
        }

        Ok(())
    }
}
