//! Laser / light-beam progress bars — eleven structurally distinct beam styles.
//!
//! Every style is stateless: all animation comes from `ctx.time` and all
//! charge / fill comes from `ctx.eased`. Lines are the medium — beams, sweeps,
//! fans, grids, and reflections. Color tints are additive; structure drives the
//! variety.
//!
//! Styles in this module:
//! 1. `charge-and-fire`     — laser cannon charges its meter and fires a beam
//! 2. `scanning-line`       — a vertical sweep beam travels back and forth
//! 3. `security-grid`       — emitter posts with criss-crossing tripwires
//! 4. `prism-dispersion`    — glass prism splits a white beam into a spectrum
//! 5. `laser-light-show`    — stage projectors throw sweeping coloured fans
//! 6. `range-finder`        — handset pings a crosshair target over a ruler
//! 7. `mirror-bounce`       — beam ricochets along a chain of hatched mirrors
//! 8. `fiber-pulse`         — light zigzags down a curving optical fiber
//! 9. `plasma-bolt`         — plasma-ball electrode discharges forked lightning
//! 10. `particle-accelerator` — synchrotron ring; a bunch laps the lit arc
//! 11. `disco-fan`          — radial beams sweeping from a corner origin

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─── deterministic hash helpers ─────────────────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

// ─── step-bounded Bresenham line ─────────────────────────────────────────────

/// Draw a straight line from `(x0,y0)` to `(x1,y1)` in dot-space using
/// integer Bresenham. At most `max_steps` pixels are emitted so that a 1×1
/// grid can never loop a million times. `draw::dot_i` ignores OOB writes.
fn beam_line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, max_steps: usize) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x0;
    let mut cy = y0;
    let steps = (dx + dy + 2) as usize;
    let limit = steps.min(max_steps);
    for _ in 0..limit {
        draw::dot_i(grid, cx, cy);
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

// ─── public registry ─────────────────────────────────────────────────────────

/// All styles in the `lasers` theme.
///
/// Returns eleven laser / light-beam bars, each with a structurally distinct
/// beam topology. Progress (`ctx.eased`) drives charge, fill, or target
/// position; time (`ctx.time`) drives sweep, animation, and flicker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(ChargeAndFire),
        Box::new(ScanningLine),
        Box::new(SecurityGrid),
        Box::new(PrismDispersion),
        Box::new(LaserLightShow),
        Box::new(RangeFinder),
        Box::new(MirrorBounce),
        Box::new(FiberPulse),
        Box::new(PlasmaBolt),
        Box::new(ParticleAccelerator),
        Box::new(DiscoFan),
    ]
}

// ─── dot-rectangle helpers ───────────────────────────────────────────────────

/// Filled rectangle `[x0,x1]×[y0,y1]` (inclusive, signed), clipped to the grid.
fn fill_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let (w, h) = draw::dot_dims(grid);
    let (xa, xb) = (x0.max(0), x1.min(w as i32 - 1));
    let (ya, yb) = (y0.max(0), y1.min(h as i32 - 1));
    for y in ya..=yb {
        for x in xa..=xb {
            draw::dot_i(grid, x, y);
        }
    }
}

/// Rectangle outline `[x0,x1]×[y0,y1]` (inclusive, signed).
fn rect_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    fill_i(grid, x0, y0, x1, y0);
    fill_i(grid, x0, y1, x1, y1);
    fill_i(grid, x0, y0, x0, y1);
    fill_i(grid, x1, y0, x1, y1);
}

/// Tint every cell overlapping the dot rectangle `[x0,x1]×[y0,y1]`.
fn tint_dots(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    if x1 < 0 || y1 < 0 || x1 < x0 || y1 < y0 {
        return;
    }
    let (cx0, cx1) = ((x0.max(0) / 2) as usize, (x1 / 2) as usize);
    for cy in (y0.max(0) / 4) as usize..=(y1 / 4) as usize {
        draw::tint_row(grid, cy, cx0, cx1, color);
    }
}

// ─── 1. Charge and fire ──────────────────────────────────────────────────────

/// A side-on laser cannon: finned housing, grip, barrel and muzzle. Its
/// segmented charge meter fills with eased while the beam it fires reaches
/// further across the bar, sparking where it lands.
struct ChargeAndFire;
impl ProgressStyle for ChargeAndFire {
    fn name(&self) -> &str {
        "charge-and-fire"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Side-on laser cannon: its charge meter fills and the beam it fires reaches across, sparking at the tip"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        if h < 8 || w < 24 {
            // Too small for the cannon — just the beam.
            let reach = (ctx.eased * w as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, h / 2);
            }
            return Ok(());
        }

        // Design space is 16 dots tall; `k` scales it to the grid.
        let k = (h as f32 / 16.0).min(w as f32 / 60.0).max(0.5);
        let s = |v: f32| (v * k).round() as i32;
        let oy = hi / 2 - s(8.0);
        let steel = Color::rgb(150, 156, 172);

        // Housing outline, cooling fins and grip.
        rect_i(grid, 0, oy + s(3.0), s(15.0), oy + s(12.0));
        for f in 0..4 {
            let fx = s(3.0 + f as f32 * 3.0);
            fill_i(grid, fx, oy, fx, oy + s(2.0));
        }
        fill_i(grid, s(3.0), oy + s(13.0), s(6.0), oy + s(15.0));
        // Barrel and muzzle flange.
        fill_i(grid, s(16.0), oy + s(6.0), s(23.0), oy + s(9.0));
        fill_i(grid, s(24.0), oy + s(5.0), s(25.0), oy + s(10.0));
        tint_dots(grid, 0, oy, s(25.0), oy + s(15.0), steel);

        // Charge meter: four segments that light one by one.
        let charge = ctx.eased.clamp(0.0, 1.0);
        for seg in 0..4 {
            let x0 = s(2.0 + seg as f32 * 3.0);
            let x1 = x0 + s(1.0);
            if charge > seg as f32 / 4.0 {
                fill_i(grid, x0, oy + s(5.0), x1, oy + s(10.0));
                let c = ctx.palette.sample(seg as f32 / 3.0);
                tint_dots(grid, x0, oy + s(5.0), x1, oy + s(10.0), c);
            } else {
                fill_i(grid, x0, oy + s(5.0), x1, oy + s(5.0));
                fill_i(grid, x0, oy + s(10.0), x1, oy + s(10.0));
            }
        }

        // The beam: a solid two-dot core with dashes racing along its halo.
        let x0 = s(26.0);
        let tip = x0 + ((wi - 1 - x0).max(0) as f32 * charge) as i32;
        let (core_a, core_b) = (oy + s(7.0), oy + s(8.0));
        let core_mid = (core_a + core_b) / 2;
        if charge > 0.01 && tip > x0 {
            fill_i(grid, x0, core_a, tip, core_b);
            let shift = (ctx.time * 40.0) as i32;
            for x in x0..=tip {
                if (x - shift).rem_euclid(5) < 2 {
                    draw::dot_i(grid, x, core_a - 1);
                    draw::dot_i(grid, x, core_b + 1);
                }
            }
            for cx in (x0 / 2)..=(tip / 2) {
                let t = (cx * 2 - x0) as f32 / (wi - x0).max(1) as f32;
                let c = ctx.palette.sample(t);
                tint_dots(grid, cx * 2, core_a - 1, cx * 2, core_b + 1, c);
            }
            // Muzzle flash flickers on alternate frames.
            if (ctx.time * 12.0) as i32 % 2 == 0 {
                fill_i(grid, x0, core_a - s(3.0), x0, core_a - s(2.0));
                fill_i(grid, x0, core_b + s(2.0), x0, core_b + s(3.0));
            }
            // Impact sparks spray from the tip.
            let frame = (ctx.time * 12.0) as u32;
            for r in 0..6u32 {
                let ang = hashf(frame.wrapping_mul(7).wrapping_add(r)) * 2.0 * PI;
                let len = (2.0 + hashf(frame.wrapping_mul(13).wrapping_add(r)) * 3.0) * k;
                let ex = tip + (ang.cos() * len) as i32;
                let ey = core_mid + (ang.sin() * len * 0.8) as i32;
                beam_line(grid, tip, core_mid, ex, ey, 8);
            }
            let spark = Color::rgb(255, 250, 225);
            tint_dots(grid, tip - 4, core_mid - 6, tip + 6, core_mid + 6, spark);
        }
        Ok(())
    }
}

// ─── 2. Scanning line ────────────────────────────────────────────────────────

/// A bright vertical beam sweeps left↔right; the swept region fills with eased.
struct ScanningLine;
impl ProgressStyle for ScanningLine {
    fn name(&self) -> &str {
        "scanning-line"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Vertical beam sweeps back and forth; swept fraction fills to eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Sweep: triangle wave in time.
        let period = 3.0f32;
        let t_norm = (ctx.time % period) / period; // 0..1
        let sweep_frac = if t_norm < 0.5 {
            t_norm * 2.0
        } else {
            2.0 - t_norm * 2.0
        };
        let sweep_x = (sweep_frac * w as f32) as usize;
        let sweep_x = sweep_x.min(w.saturating_sub(1));

        // Filled region up to eased progress: sparse horizontal lines.
        let filled = ((ctx.eased * w as f32) as usize).min(w);
        let mid = h / 2;
        // Three horizontal rails.
        draw::hline(
            grid,
            0,
            filled.saturating_sub(1),
            mid.saturating_sub(1).min(h - 1),
        );
        draw::hline(grid, 0, filled.saturating_sub(1), mid);
        draw::hline(
            grid,
            0,
            filled.saturating_sub(1),
            (mid + 1).min(h.saturating_sub(1)),
        );

        // Sweep beam: full-height vertical line with a narrow bright core.
        draw::vline(grid, sweep_x, 0, h.saturating_sub(1));
        // One-dot wings on each side for width.
        if sweep_x > 0 {
            draw::vline(grid, sweep_x - 1, h / 4, h * 3 / 4);
        }
        if sweep_x + 1 < w {
            draw::vline(grid, sweep_x + 1, h / 4, h * 3 / 4);
        }

        // Tint: filled region gets gradient; beam cell gets bright end.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        let beam_cell = (sweep_x / 2).min(cells_w.saturating_sub(1));
        let bright = ctx.palette.sample(1.0);
        for cy in 0..cells_h {
            draw::tint_row(grid, cy, beam_cell, beam_cell, bright);
        }

        Ok(())
    }
}

// ─── 3. Security grid ────────────────────────────────────────────────────────

/// A vault corridor of emitter posts with red tripwire beams criss-crossing
/// between each pair. Bays behind the progress front are disarmed (dim green,
/// dotted); bays ahead stay armed, with light pulses racing along the beams.
struct SecurityGrid;
impl ProgressStyle for SecurityGrid {
    fn name(&self) -> &str {
        "security-grid"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Corridor of emitter posts with red tripwires criss-crossing between them; bays disarm to green with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bound = (w + h) * 2;
        let bay = (hi * 3 / 4).max(6);
        let bays = ((wi - 2) / bay).max(1);
        let bay_w = ((wi - 2) / bays).max(2);
        let front = ctx.eased.clamp(0.0, 1.0) * bays as f32;
        let shift = (ctx.time * 24.0) as i32;
        let blink = (ctx.time * 3.0) as i32 % 2 == 0;
        let armed_col = Color::rgb(255, 50, 60);
        let safe_col = Color::rgb(70, 200, 110);
        let post_col = Color::rgb(140, 145, 165);

        for b in 0..bays {
            let x0 = b * bay_w + 2;
            let x1 = x0 + bay_w - 2;
            let disarmed = (b as f32) < front.floor();
            // Beam rows: X between the posts plus a waist-height line.
            for (ya, yb) in [(0, hi - 1), (hi - 1, 0)] {
                if disarmed {
                    for i in (0..=(x1 - x0)).step_by(3) {
                        let y = ya + (yb - ya) * i / (x1 - x0).max(1);
                        draw::dot_i(grid, x0 + i, y);
                    }
                } else {
                    beam_line(grid, x0, ya, x1, yb, bound);
                }
            }
            let waist = hi / 2;
            if !disarmed {
                draw::hline(grid, x0 as usize, x1.max(x0) as usize, waist as usize);
                // Pulses racing along the waist beam: gaps that move.
                for x in x0..=x1 {
                    if (x + shift).rem_euclid(9) == 0 {
                        draw::dot_i(grid, x, waist - 1);
                        draw::dot_i(grid, x, waist + 1);
                    }
                }
            }
            let col = if disarmed { safe_col } else { armed_col };
            for cy in 0..(hi / 4).max(1) {
                draw::tint_row(
                    grid,
                    cy as usize,
                    ((x0 + 1) / 2) as usize,
                    ((x1 - 1) / 2).max(0) as usize,
                    col,
                );
            }
        }

        // Emitter posts between bays, with blinking status lamps on top.
        for p in 0..=bays {
            let x = (p * bay_w).min(wi - 2);
            fill_i(grid, x, 1, x + 1, hi - 1);
            let armed = (p as f32) >= front.floor() && p < bays;
            if !armed || blink {
                fill_i(grid, x, 0, x + 1, 0);
            }
            let lamp = if armed { armed_col } else { safe_col };
            tint_dots(grid, x, 0, x + 1, hi - 1, post_col);
            tint_dots(grid, x, 0, x + 1, 0, lamp);
        }
        Ok(())
    }
}

// ─── 4. Prism dispersion ────────────────────────────────────────────────────

/// Seven-stop visible spectrum, red (`f = 0`) to violet (`f = 1`).
fn spectrum(f: f32) -> Color {
    const STOPS: [(u8, u8, u8); 7] = [
        (255, 45, 45),
        (255, 145, 20),
        (255, 230, 30),
        (60, 225, 80),
        (40, 140, 255),
        (95, 75, 240),
        (180, 80, 255),
    ];
    let i = ((f.clamp(0.0, 1.0) * 6.0).round() as usize).min(6);
    let (r, g, b) = STOPS[i];
    Color::rgb(r, g, b)
}

/// A glass triangular prism: a white beam strikes its left face, bends
/// through the glass, and leaves the right face as a rainbow fan that
/// reaches further across the bar with eased.
struct PrismDispersion;
impl ProgressStyle for PrismDispersion {
    fn name(&self) -> &str {
        "prism-dispersion"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "A white beam strikes a glass prism and fans out as a red-to-violet spectrum that reaches across with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        if h < 8 || w < 20 {
            let reach = (ctx.eased * w as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, h / 2);
            }
            return Ok(());
        }
        let bound = (w + h) * 2;

        // Prism: apex at the top, base on the bottom row.
        let px = (wi / 4).max(hi * 3 / 4);
        let half = (hi as f32 * 0.62) as i32;
        let base_y = hi - 1;
        let left_x = |y: i32| px - half * y / base_y.max(1);
        let right_x = |y: i32| px + half * y / base_y.max(1);
        beam_line(grid, px, 0, px - half, base_y, bound);
        beam_line(grid, px, 0, px + half, base_y, bound);
        beam_line(grid, px - half, base_y, px + half, base_y, bound);
        // A glint inside the glass, parallel to the left face.
        beam_line(grid, px + 1, hi / 4 + 1, px - half / 3, hi * 3 / 4, bound);
        tint_dots(
            grid,
            px - half,
            0,
            px + half,
            base_y,
            Color::rgb(190, 225, 255),
        );

        // Incoming white beam with photon gaps drifting toward the prism.
        let y_in = hi * 5 / 16;
        let xl = left_x(y_in);
        let shift = (ctx.time * 30.0) as i32;
        for x in 0..=xl {
            if (x - shift).rem_euclid(12) >= 2 {
                draw::dot_i(grid, x, y_in);
            }
        }
        tint_dots(grid, 0, y_in, xl - 1, y_in, Color::rgb(240, 240, 240));

        // Refracted path through the glass, bending toward the base.
        let y_out = y_in + hi / 6;
        let xr = right_x(y_out);
        beam_line(grid, xl, y_in, xr, y_out, bound);

        // Spectrum fan: rays from the exit point toward the right edge.
        let e = ctx.eased.clamp(0.0, 1.0);
        let n = (hi / 2).max(4);
        let span = (wi - 1 - xr).max(1);
        let tip_x = xr + (span as f32 * e) as i32;
        let frame = (ctx.time * 10.0) as u32;
        for i in 0..n {
            let ty = (i as f32 / (n - 1) as f32 * (hi - 1) as f32) as i32;
            let ey = y_out + ((ty - y_out) as f32 * e) as i32;
            beam_line(grid, xr, y_out, tip_x, ey, bound);
            // A twinkle travels down each ray.
            let along = ((ctx.time * 1.3 + hashf(i as u32)) % 1.0) * e;
            let tx = xr + (span as f32 * along) as i32;
            let tyy = y_out + ((ty - y_out) as f32 * along) as i32;
            if hash(frame.wrapping_add(i as u32)) % 3 != 0 {
                draw::dot_i(grid, tx, tyy - 1);
            }
        }

        // Colour each fan cell by where it sits across the spread.
        for cx in (xr / 2 + 1)..=(tip_x / 2) {
            let x = cx * 2 + 1;
            let t = (x - xr) as f32 / span as f32;
            let top = y_out as f32 * (1.0 - t);
            let bot = y_out as f32 + (hi - 1 - y_out) as f32 * t;
            for cy in 0..(hi / 4) {
                let yc = (cy * 4 + 2) as f32;
                let f = ((yc - top) / (bot - top).max(1.0)).clamp(0.0, 1.0);
                draw::tint_row(grid, cy as usize, cx as usize, cx as usize, spectrum(f));
            }
        }
        Ok(())
    }
}

// ─── 5. Laser light show ────────────────────────────────────────────────────

/// Draw a beam from `(x0,y0)` to `(x1,y1)`, tinting every cell it crosses.
/// `skip` drops dots where `(i + shift) % 7 == 0` for a shimmering beam.
fn tinted_beam(
    grid: &mut BrailleGrid,
    (x0, y0): (i32, i32),
    (x1, y1): (i32, i32),
    shift: i32,
    color: Color,
) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 512);
    for i in 0..=steps {
        let x = x0 + (x1 - x0) * i / steps;
        let y = y0 + (y1 - y0) * i / steps;
        if (i + shift).rem_euclid(7) != 0 {
            draw::dot_i(grid, x, y);
        }
        tint_dots(grid, x, y, x, y, color);
    }
}

/// Concert laser rig: projectors along the stage floor each throw a fan of
/// coloured beams up into the haze, sweeping side to side with time. More
/// projectors switch on as eased grows.
struct LaserLightShow;
impl ProgressStyle for LaserLightShow {
    fn name(&self) -> &str {
        "laser-light-show"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Concert laser rig: stage projectors throw sweeping fans of coloured beams; more projectors fire with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        const SHOW: [(u8, u8, u8); 5] = [
            (60, 255, 90),
            (255, 60, 200),
            (60, 200, 255),
            (255, 220, 50),
            (255, 70, 60),
        ];
        let n_proj = (wi / 16).clamp(1, 8);
        let active = (ctx.eased.clamp(0.0, 1.0) * n_proj as f32).ceil() as i32;
        let floor_y = hi - 1;
        let gap = wi / n_proj;
        let shift = (ctx.time * 20.0) as i32;

        for p in 0..n_proj {
            let px = gap / 2 + p * gap;
            let (r, g, b) = SHOW[p as usize % SHOW.len()];
            let col = Color::rgb(r, g, b);
            // Projector housing on the stage floor.
            fill_i(grid, px - 2, floor_y - 1, px + 2, floor_y);
            tint_dots(
                grid,
                px - 2,
                floor_y - 1,
                px + 2,
                floor_y,
                Color::rgb(120, 125, 145),
            );
            if p >= active {
                continue;
            }
            // Fan of three beams, sweeping with a per-projector phase.
            let sweep = (ctx.time * (1.1 + p as f32 * 0.23) + p as f32 * 1.7).sin() * 0.9;
            let rise = (floor_y - 2).max(1) as f32;
            for k in -1..=1 {
                let ang = sweep + k as f32 * 0.35;
                let ex = px + (ang.tan() * rise).clamp(-(wi as f32), wi as f32) as i32;
                tinted_beam(grid, (px, floor_y - 2), (ex, 0), shift + k * 3, col);
            }
            // Lens glow.
            tint_dots(
                grid,
                px - 1,
                floor_y - 2,
                px + 1,
                floor_y - 2,
                Color::rgb(255, 255, 255),
            );
            draw::dot_i(grid, px, floor_y - 2);
        }
        Ok(())
    }
}

// ─── 6. Range finder ────────────────────────────────────────────────────────

/// A laser range finder: a boxy handset with a lens fires a pulse at a
/// crosshair target and catches the echo. The target stands further away as
/// eased grows, over a ruler of distance ticks; the pulse shuttles out and
/// back with time.
struct RangeFinder;
impl ProgressStyle for RangeFinder {
    fn name(&self) -> &str {
        "range-finder"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Laser range finder pings a crosshair target that stands further away with progress, over a distance ruler"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = (hi - 2) / 2;

        // Handset: body outline, lens on the front, eyepiece on the back.
        let bw = (hi * 3 / 4).clamp(2, 12);
        let bh = (hi / 2).max(1);
        rect_i(grid, 1, mid - bh / 2, bw, mid + bh / 2);
        fill_i(grid, 0, mid - 1, 0, mid + 1);
        fill_i(grid, bw + 1, mid - 1, bw + 2, mid + 1);
        fill_i(grid, 3, mid - 1, 4, mid);
        let steel = Color::rgb(150, 156, 172);
        tint_dots(grid, 0, mid - bh / 2, bw + 2, mid + bh / 2, steel);

        // Target: crosshair reticle at the eased distance.
        let r = (hi / 3).clamp(1, 6);
        let x_lens = bw + 3;
        let near = x_lens + r + 4;
        let span = (wi - 1 - (r + 2) - near).max(0);
        let tx = near + (span as f32 * ctx.eased.clamp(0.0, 1.0)) as i32;
        let steps = (r * 8).max(8) as usize;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            draw::dot_i(
                grid,
                tx + (a.cos() * r as f32).round() as i32,
                mid + (a.sin() * r as f32).round() as i32,
            );
        }
        fill_i(grid, tx - r - 2, mid, tx - r + 1, mid);
        fill_i(grid, tx + r - 1, mid, tx + r + 2, mid);
        fill_i(grid, tx, mid - r - 2, tx, mid - r + 1);
        fill_i(grid, tx, mid + r - 1, tx, mid + r + 2);
        draw::dot_i(grid, tx, mid);
        let lock = Color::rgb(255, 70, 70);
        tint_dots(grid, tx - r - 2, mid - r - 2, tx + r + 2, mid + r + 2, lock);

        // Beam: a faint dotted path plus a bright pulse going out and back.
        let path = (tx - r - 3 - x_lens).max(0);
        for x in (x_lens..x_lens + path).step_by(3) {
            draw::dot_i(grid, x, mid);
        }
        let phase = (ctx.time * 1.2).fract();
        let along = if phase < 0.5 {
            phase * 2.0
        } else {
            2.0 - phase * 2.0
        };
        let px = x_lens + (path as f32 * along) as i32;
        fill_i(grid, px - 2, mid, px + 2, mid);
        fill_i(grid, px - 1, mid - 1, px + 1, mid + 1);
        if path > 0 {
            tint_dots(
                grid,
                x_lens,
                mid,
                x_lens + path,
                mid,
                ctx.palette.sample(0.3),
            );
            tint_dots(
                grid,
                px - 2,
                mid - 1,
                px + 2,
                mid + 1,
                ctx.palette.sample(1.0),
            );
        }

        // Distance ruler along the bottom, ticked up to the target.
        let ry = hi - 1;
        if ry > mid + r {
            draw::hline(grid, x_lens as usize, tx.max(x_lens) as usize, ry as usize);
            for x in (x_lens..=tx).step_by(4) {
                let tall = (x - x_lens) % 16 == 0;
                draw::dot_i(grid, x, ry - 1);
                if tall {
                    draw::dot_i(grid, x, ry - 2);
                }
            }
            tint_dots(grid, x_lens, ry, tx, ry, Color::rgb(120, 125, 145));
        }
        Ok(())
    }
}

// ─── 7. Mirror bounce ───────────────────────────────────────────────────────

/// A chain of hatched flat mirrors, alternating ceiling and floor. The beam
/// enters on the left and ricochets mirror to mirror, reaching further down
/// the chain as eased grows; each mirror it strikes glints, and photon
/// pulses stream along the lit path.
struct MirrorBounce;
impl ProgressStyle for MirrorBounce {
    fn name(&self) -> &str {
        "mirror-bounce"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "A beam ricochets between hatched mirrors on the ceiling and floor, reaching further down the chain with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let spacing = hi.max(4);
        let half_m = (spacing / 3).max(1);
        let (top_y, bot_y) = (2.min(hi - 1), (hi - 3).max(0));
        let silver = Color::rgb(170, 178, 200);
        let n = (wi - spacing / 2) / spacing + 1;
        let front = ctx.eased.clamp(0.0, 1.0) * wi as f32;
        let shift = (ctx.time * 24.0) as i32;

        // Mirrors: a reflective face with hatching on the back.
        let mirror_at = |i: i32| {
            (
                spacing / 2 + i * spacing,
                if i % 2 == 0 { top_y } else { bot_y },
            )
        };
        for i in 0..n {
            let (mx, my) = mirror_at(i);
            fill_i(grid, mx - half_m, my, mx + half_m, my);
            let back = if my == top_y { my - 1 } else { my + 1 };
            for x in (mx - half_m..=mx + half_m).step_by(2) {
                draw::dot_i(grid, x, back);
                draw::dot_i(grid, x + 1, back + (back - my));
            }
            tint_dots(
                grid,
                mx - half_m,
                my.min(back) - 1,
                mx + half_m,
                my.max(back) + 1,
                silver,
            );
        }

        // Beam path: enters at mid-left, then mirror to mirror.
        let mut prev = (0, hi / 2);
        let mut travelled = 0;
        for i in 0..=n {
            let next = if i < n {
                let (mx, my) = mirror_at(i);
                (mx, if my == top_y { my + 1 } else { my - 1 })
            } else {
                (wi - 1, hi / 2)
            };
            if prev.0 as f32 >= front {
                break;
            }
            let seg_w = (next.0 - prev.0).max(1);
            let reach = ((front - prev.0 as f32) / seg_w as f32).min(1.0);
            let end = (
                prev.0 + (seg_w as f32 * reach) as i32,
                prev.1 + ((next.1 - prev.1) as f32 * reach) as i32,
            );
            let steps = (end.0 - prev.0)
                .abs()
                .max((end.1 - prev.1).abs())
                .clamp(1, 512);
            for s in 0..=steps {
                let x = prev.0 + (end.0 - prev.0) * s / steps;
                let y = prev.1 + (end.1 - prev.1) * s / steps;
                draw::dot_i(grid, x, y);
                if (travelled + s - shift).rem_euclid(10) < 2 {
                    draw::dot_i(grid, x + 1, y);
                    draw::dot_i(grid, x, y + 1);
                }
                let t = x as f32 / wi as f32;
                tint_dots(grid, x, y, x, y, ctx.palette.sample(t));
            }
            travelled += steps;
            if reach >= 1.0 && i < n {
                // Glint where the beam strikes the mirror.
                let (mx, my) = mirror_at(i);
                if (ctx.time * 6.0 + i as f32) as i32 % 2 == 0 {
                    fill_i(grid, mx - 1, my, mx + 1, my);
                }
                tint_dots(grid, mx - 1, my, mx + 1, my, Color::rgb(255, 255, 255));
            }
            prev = next;
        }
        Ok(())
    }
}

// ─── 8. Fiber pulse ─────────────────────────────────────────────────────────

/// A gently curving optical fiber: two cladding walls around a core. Light
/// zigzags down the core by total internal reflection, lighting the fiber up
/// to eased; bright pulses ride the zigzag with time.
struct FiberPulse;
impl ProgressStyle for FiberPulse {
    fn name(&self) -> &str {
        "fiber-pulse"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Curving optical fiber: light zigzags down the core by total internal reflection, lit up to progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let hf = (hi - 1) as f32;
        // Fiber centre line bends gently; walls sit `half` dots either side.
        let bend = hf / 8.0;
        let half = (hf / 2.0 - bend - 1.0).max(1.0);
        let centre = |x: i32| hf / 2.0 + bend * (x as f32 / wi as f32 * 2.0 * PI).sin();
        let front = (ctx.eased.clamp(0.0, 1.0) * wi as f32) as i32;
        let clad = Color::rgb(90, 95, 120);

        for x in 0..wi {
            let c = centre(x);
            let (top, bot) = ((c - half).round() as i32, (c + half).round() as i32);
            // Cladding: solid where lit, dotted ahead.
            if x < front || x % 3 == 0 {
                draw::dot_i(grid, x, top - 1);
                draw::dot_i(grid, x, bot + 1);
            }
            // Zigzag ray: a triangle wave between the walls.
            if x < front {
                let period = (half * 3.0).max(2.0);
                let ph = (x as f32 / period).fract();
                let tri = if ph < 0.5 { ph * 2.0 } else { 2.0 - ph * 2.0 };
                let y = (c - half + tri * half * 2.0).round() as i32;
                draw::dot_i(grid, x, y);
                // Travelling pulses: a thicker blob riding the ray.
                let shift = ctx.time * 28.0;
                if (x as f32 - shift).rem_euclid(period * 2.0) < 3.0 {
                    draw::dot_i(grid, x, y - 1);
                    draw::dot_i(grid, x, y + 1);
                }
            }
        }
        // Colour: lit core in the palette gradient, cladding grey ahead.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let lit = (cx as i32) * 2 < front;
            let col = if lit {
                ctx.palette.sample(cx as f32 / cw.max(1) as f32)
            } else {
                clad
            };
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        // Glowing end face at the light front.
        if front > 0 && front < wi {
            let c = centre(front);
            line_v(
                grid,
                front,
                (c - half).round() as i32,
                (c + half).round() as i32,
            );
            tint_dots(grid, front, 0, front, hi - 1, Color::rgb(255, 255, 255));
        }
        Ok(())
    }
}

/// Vertical run `[y0, y1]` at column `x` (signed, clipped).
fn line_v(grid: &mut BrailleGrid, x: i32, y0: i32, y1: i32) {
    fill_i(grid, x, y0.min(y1), x, y0.max(y1));
}

// ─── 9. Plasma bolt ─────────────────────────────────────────────────────────

/// A glowing plasma-ball electrode on the left discharges a forked lightning
/// bolt whose reach grows with eased. The bolt's zigzag path and branches
/// re-strike every frame; a crackling corona surrounds the electrode.
struct PlasmaBolt;
impl ProgressStyle for PlasmaBolt {
    fn name(&self) -> &str {
        "plasma-bolt"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "A plasma-ball electrode discharges a forked lightning bolt that re-strikes each frame; reach = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bound = (w + h) * 2;
        let frame = (ctx.time * 12.0) as u32;
        let mid = hi / 2;

        // Electrode: a filled plasma ball with a flickering corona.
        let r = ((hi - 2) / 2).clamp(0, 6);
        let bx = r;
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    draw::dot_i(grid, bx + x, mid + y);
                }
            }
        }
        for k in 0..10u32 {
            let ang = hashf(frame.wrapping_mul(31).wrapping_add(k)) * 2.0 * PI;
            let rr = r as f32 + 1.5 + hashf(frame.wrapping_add(k * 7)) * 2.0;
            draw::dot_i(
                grid,
                bx + (ang.cos() * rr) as i32,
                mid + (ang.sin() * rr) as i32,
            );
        }
        tint_dots(
            grid,
            0,
            mid - r - 3,
            bx + r + 3,
            mid + r + 3,
            ctx.palette.sample(0.0),
        );

        // Main bolt: zigzag from the ball's edge to the reach point.
        let x_start = bx + r + 1;
        let reach = x_start + ((wi - 1 - x_start).max(0) as f32 * ctx.eased.clamp(0.0, 1.0)) as i32;
        if reach <= x_start {
            return Ok(());
        }
        let seg = (hi / 3).max(3);
        let amp = (hi / 2 - 1).max(0);
        let (mut px, mut py) = (x_start, mid);
        let mut n = 0u32;
        while px < reach {
            let nx = (px + seg).min(reach);
            let jitter = hashf(frame.wrapping_mul(97).wrapping_add(n)) * 2.0 - 1.0;
            let ny = if nx >= reach {
                mid
            } else {
                mid + (jitter * amp as f32) as i32
            };
            beam_line(grid, px, py, nx, ny, bound);
            beam_line(grid, px, py + 1, nx, ny + 1, bound);
            // Every few kinks a short fork splits off.
            if hash(frame.wrapping_add(n * 13)) % 3 == 0 {
                let fy = if ny < mid { ny - seg } else { ny + seg };
                beam_line(grid, nx, ny, nx + seg, fy, bound);
            }
            let c = ctx.palette.sample(px as f32 / wi as f32);
            tint_dots(grid, px, 0, nx, hi - 1, c);
            px = nx;
            py = ny;
            n += 1;
        }
        // White-hot strike point.
        tint_dots(
            grid,
            reach - 2,
            0,
            reach + 2,
            hi - 1,
            Color::rgb(255, 255, 255),
        );
        for k in 0..5u32 {
            let ang = hashf(frame.wrapping_mul(5).wrapping_add(k + 50)) * 2.0 * PI;
            beam_line(
                grid,
                reach,
                mid,
                reach + (ang.cos() * 3.0) as i32,
                mid + (ang.sin() * 3.0) as i32,
                6,
            );
        }
        Ok(())
    }
}

// ─── 10. Particle accelerator ───────────────────────────────────────────────

/// A synchrotron ring seen from above: a double-walled oval beam pipe with
/// dipole magnets. The energised arc of the ring grows with eased while a
/// particle bunch laps it, faster and with a longer streak as it charges.
struct ParticleAccelerator;
impl ProgressStyle for ParticleAccelerator {
    fn name(&self) -> &str {
        "particle-accelerator"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Synchrotron ring with magnets: the energised arc grows with progress as a particle bunch laps ever faster"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        if h < 8 || w < 16 {
            let reach = (ctx.eased * w as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, h / 2);
            }
            return Ok(());
        }
        let (cx, cy) = ((w as f32 - 1.0) / 2.0, (h as f32 - 1.0) / 2.0);
        let (ao, bo) = (cx - 1.0, cy - 0.5);
        let (am, bm) = (ao - 2.0, bo - 2.0);
        let (ai, bi) = (ao - 4.0, bo - 4.0);
        // Angle 0 is the bottom of the ring (injection point), running clockwise.
        let at = |a: f32, b: f32, ang: f32| {
            (
                (cx - ang.sin() * a).round() as i32,
                (cy + ang.cos() * b).round() as i32,
            )
        };
        let steps = ((ao + bo) * 4.0) as usize;
        let pipe = Color::rgb(95, 100, 125);

        // Beam pipe walls.
        for s in 0..steps {
            let ang = s as f32 / steps as f32 * 2.0 * PI;
            let (x, y) = at(ao, bo, ang);
            draw::dot_i(grid, x, y);
            let (x, y) = at(ai.max(1.0), bi.max(0.5), ang);
            draw::dot_i(grid, x, y);
            draw::tint_row(
                grid,
                (y.max(0) / 4) as usize,
                (x.max(0) / 2) as usize,
                (x.max(0) / 2) as usize,
                pipe,
            );
            let (x, y) = at(ao, bo, ang);
            tint_dots(grid, x, y, x, y, pipe);
        }
        // Dipole magnets: short bars straddling the pipe.
        for m in 0..10 {
            let ang = (m as f32 + 0.5) / 10.0 * 2.0 * PI;
            let (x0, y0) = at(ao + 1.0, bo + 1.0, ang);
            let (x1, y1) = at(ai - 1.0, bi - 1.0, ang);
            beam_line(grid, x0, y0, x1, y1, 16);
            beam_line(grid, x0 + 1, y0, x1 + 1, y1, 16);
        }

        // Energised arc: the beam inside the pipe, length = eased.
        let e = ctx.eased.clamp(0.0, 1.0);
        let lit = (steps as f32 * e) as usize;
        for s in 0..lit {
            let f = s as f32 / steps as f32;
            let (x, y) = at(am, bm, f * 2.0 * PI);
            draw::dot_i(grid, x, y);
            tint_dots(grid, x, y, x, y, ctx.palette.sample(f));
        }

        // Particle bunch laps the ring; speed and streak grow with charge.
        let lap = ctx.time * (0.25 + e * 0.9);
        let head = lap.fract() * 2.0 * PI;
        let tail = 0.25 + e * 1.1;
        let n_tail = (tail * am / 2.0) as usize + 2;
        for i in 0..n_tail {
            let ang = head - tail * i as f32 / n_tail as f32;
            let (x, y) = at(am, bm, ang);
            draw::dot_i(grid, x, y);
            if i < 3 {
                fill_i(grid, x - 1, y - 1, x + 1, y + 1);
            }
            let c = if i < n_tail / 3 {
                Color::rgb(255, 255, 255)
            } else {
                ctx.palette.sample(1.0)
            };
            tint_dots(grid, x, y, x, y, c);
        }
        Ok(())
    }
}
// ─── 11. Disco fan ──────────────────────────────────────────────────────────

/// Radial beams sweep from a corner origin; sweep angle grows with eased + time.
struct DiscoFan;
impl ProgressStyle for DiscoFan {
    fn name(&self) -> &str {
        "disco-fan"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Radial fan of beams sweeps from a corner; beam count and arc = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Fan origin: bottom-left corner.
        let ox = 0i32;
        let oy = h as i32 - 1;

        // Number of beams.
        let n_beams = ((ctx.eased * 9.0 + 1.0) as usize).clamp(1, 10);

        // The fan sweeps continuously via time, occupying an arc that grows with eased.
        let arc = ctx.eased * PI * 0.9 + 0.1; // arc in radians, 0.1..~π·0.9
        let sweep_center = PI / 2.0 * (0.4 + 0.6 * ctx.eased) - (ctx.time * 0.6).sin() * arc * 0.3; // centre oscillates

        for b in 0..n_beams {
            let angle_frac = if n_beams == 1 {
                0.5f32
            } else {
                b as f32 / (n_beams - 1) as f32
            };
            let angle = sweep_center - arc / 2.0 + angle_frac * arc;

            // Beam extends to the far edge of the grid.
            let beam_len = ((w as f32).hypot(h as f32) as i32 + 2).max(2);
            let end_x = ox + (angle.cos() * beam_len as f32) as i32;
            let end_y = oy - (angle.sin() * beam_len as f32) as i32;

            beam_line(grid, ox, oy, end_x, end_y, (w + h) * 2);

            // Tint each beam column.
            let t = b as f32 / n_beams.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            // Paint a stripe along the beam.
            let x_end = end_x.clamp(0, cells_w as i32 - 1) as usize;
            let x_start = 0usize;
            let (lo, hi) = if x_start <= x_end {
                (x_start, x_end)
            } else {
                (x_end, x_start)
            };
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, lo, hi.min(cells_w.saturating_sub(1)), color);
            }
        }

        // Bright origin point.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, ox + dx, oy + dy);
            }
        }

        Ok(())
    }
}
