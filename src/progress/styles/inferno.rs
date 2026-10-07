//! Fire progress bars — spreading flames, rising embers, white-hot heads.
//!
//! Everything burns on a char-red → orange → gold → white heat ramp.
//! Progress reads as fire spreading along the bar (or candles lighting,
//! or a torch cutting through) while `time` keeps the flames flickering.
//! All motion is deterministic in `(progress, time)`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::TAU;

// ─── deterministic hash ─────────────────────────────────────────────────────

/// Fast integer hash → `[0, 1)`.
#[inline]
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) % 1000) as f32 / 1000.0
}

/// 3-D variant: hash `(x, y, z_int)` for time-slotted flicker.
#[inline]
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash2(x ^ z.wrapping_mul(1_234_567), y ^ z.wrapping_mul(7_654_321))
}

// ─── theme tint — heat ramp ─────────────────────────────────────────────────

/// Charred deep red at the cold end.
const HEAT_CHAR: Color = Color::rgb(143, 29, 18);
/// Flame orange in the body.
const HEAT_ORANGE: Color = Color::rgb(255, 122, 47);
/// Molten gold near the core.
const HEAT_GOLD: Color = Color::rgb(255, 200, 74);
/// White-hot center.
const HEAT_WHITE: Color = Color::rgb(255, 243, 214);

/// Sample the char → orange → gold → white heat ramp at `t` in `0.0..=1.0`.
fn sample_tint(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8, k: f32| (f32::from(a) + (f32::from(b) - f32::from(a)) * k) as u8;
    let (from, to, k) = if t < 0.45 {
        (HEAT_CHAR, HEAT_ORANGE, t / 0.45)
    } else if t < 0.8 {
        (HEAT_ORANGE, HEAT_GOLD, (t - 0.45) / 0.35)
    } else {
        (HEAT_GOLD, HEAT_WHITE, (t - 0.8) / 0.2)
    };
    Color::rgb(
        lerp(from.r, to.r, k),
        lerp(from.g, to.g, k),
        lerp(from.b, to.b, k),
    )
}

/// Applies the heat ramp to every cell the inner style drew: hotter toward
/// the bottom (where the fire lives) with a fast per-cell flicker.
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
        let slot = (ctx.time * 8.0) as i32;
        for y in 0..h {
            for x in 0..w {
                let ch = grid.get_char(x, y);
                if ch != '\u{2800}' && ch != ' ' {
                    let depth = (y as f32 + 0.5) / h.max(1) as f32;
                    let flicker = 0.2 * hash3(x as i32, y as i32, slot);
                    let _ = grid.set_cell_color(x, y, sample_tint(0.25 + 0.5 * depth + flicker));
                }
            }
        }
        Ok(())
    }
}

/// All styles in the `inferno` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Wildfire),
        Box::new(FlameFill),
        Box::new(EmberRise),
        Box::new(Blowtorch),
        Box::new(Candle),
        Box::new(Flashover),
        Box::new(Charline),
        Box::new(Tinted(Backdraft)),
        Box::new(SolarFlare),
        Box::new(Phoenix),
    ]
}

/// Fire spreads left to right: ember bed behind, flames at the front.
struct Wildfire;
impl ProgressStyle for Wildfire {
    fn name(&self) -> &str {
        "wildfire"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A fire line spreading across the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let front = (ctx.eased * w as f32).round() as usize;
        let slot = (ctx.time * 8.0) as i32;
        // Unburnt fuel ahead: a thin ground line.
        for x in front..w {
            draw::dot(grid, x, h - 1);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, Color::rgb(92, 74, 52));
        }
        // Ember bed behind the front: lower half, glowing unevenly.
        for x in 0..front {
            let bed = h / 2 + (hash2(x as i32, 3) * 2.0) as usize;
            for y in bed.min(h - 1)..h {
                draw::dot(grid, x, y);
            }
            let glow = 0.25 + 0.25 * hash3(x as i32, 0, slot / 2);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, sample_tint(glow));
        }
        // Burning zone at the front: tall flame tongues licking upward.
        let zone = 12usize;
        for k in 0..zone {
            let x = front.saturating_sub(zone) + k;
            if x >= w {
                break;
            }
            let near = k as f32 / zone as f32;
            let flick = hash3(x as i32, 1, slot);
            let tongue = ((0.45 + 0.55 * near) * (0.65 + 0.35 * flick) * h as f32) as usize;
            for y in h.saturating_sub(tongue.max(2))..h {
                draw::dot(grid, x, y);
            }
            // Sparks leaping off the tallest tongues.
            if flick > 0.6 {
                let sy = h.saturating_sub(tongue + 2);
                draw::dot(grid, x, sy);
            }
            let heat = 0.5 + 0.5 * near;
            for cy in 0..grid.dimensions().1 {
                draw::tint_row(grid, cy, x / 2, x / 2, sample_tint(heat - cy as f32 * 0.12));
            }
        }
        Ok(())
    }
}

/// The whole filled region is living flame, tips flickering.
struct FlameFill;
impl ProgressStyle for FlameFill {
    fn name(&self) -> &str {
        "flame-fill"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A bar of living flame"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let slot = (ctx.time * 8.0) as i32;
        for x in 0..filled {
            let sway = (x as f32 * 0.4 + ctx.time * TAU * 0.5).sin();
            let flick = hash3(x as i32, 0, slot);
            let height = ((0.55 + 0.2 * sway + 0.25 * flick) * h as f32) as usize;
            let top = h.saturating_sub(height.max(2));
            for y in top..h {
                draw::dot(grid, x, y);
            }
            // Heat rises: white at the base, red at the tips.
            let (_, ch_cells) = grid.dimensions();
            for cy in 0..ch_cells {
                let frac = 1.0 - cy as f32 / ch_cells.max(1) as f32;
                draw::tint_row(grid, cy, x / 2, x / 2, sample_tint(1.0 - frac * 0.75));
            }
        }
        Ok(())
    }
}

/// A glowing bed extends along the bar; embers rise and drift above it.
struct EmberRise;
impl ProgressStyle for EmberRise {
    fn name(&self) -> &str {
        "ember-rise"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "Embers drifting up from a glowing bed"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // The bed: two glowing dot rows.
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
            draw::dot(grid, x, h.saturating_sub(2));
            let glow = 0.35 + 0.3 * hash3(x as i32, 0, (ctx.time * 6.0) as i32);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, sample_tint(glow));
        }
        // Embers rise from the lit bed, drifting sideways as they climb.
        let head = h.saturating_sub(2) as f32;
        if head > 1.0 {
            for e in 0..(w / 3) {
                let ex = (hash2(e as i32, 5) * filled.max(1) as f32) as usize;
                if ex >= filled {
                    continue;
                }
                let rate = 0.25 + ((hash2(e as i32, 6) * 3.0).round()) * 0.25;
                let climb = (ctx.time * rate + hash2(e as i32, 7)).fract();
                let y = head - climb * head;
                let drift = ((climb * TAU).sin() * 2.5) as i32;
                let px = ex as i32 + drift;
                draw::dot_i(grid, px, y as i32);
                if px >= 0 {
                    let heat = 1.0 - climb;
                    let _ = grid.set_cell_color(
                        px as usize / 2,
                        y as usize / 4,
                        sample_tint(0.4 + 0.6 * heat),
                    );
                }
            }
        }
        Ok(())
    }
}

/// A torch head cuts rightward, leaving a cooling molten line.
struct Blowtorch;
impl ProgressStyle for Blowtorch {
    fn name(&self) -> &str {
        "blowtorch"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A torch cutting a molten line"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let head = (ctx.eased * w as f32) as usize;
        let mid = h / 2;
        // The cooling cut: brightness decays with distance behind the head.
        for x in 0..head.min(w) {
            draw::dot(grid, x, mid);
            draw::dot(grid, x, mid.saturating_sub(1));
            let cool = (head - x) as f32 / w.max(1) as f32;
            let _ = grid.set_cell_color(x / 2, mid / 4, sample_tint(1.0 - cool * 1.6));
        }
        // The torch cone: a bright wedge of dots at the head.
        let slot = (ctx.time * 12.0) as i32;
        for dx in 0..5i32 {
            let spread = dx;
            for dy in -spread..=spread {
                let x = head as i32 + dx;
                let y = mid as i32 + dy;
                if hash3(x, y, slot) < 0.75 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        if head < w {
            let _ = grid.set_cell_color(head / 2, mid / 4, HEAT_WHITE);
            let _ = grid.set_cell_color(((head + 4).min(w - 1)) / 2, mid / 4, HEAT_GOLD);
        }
        // Sparks spraying off the cut point.
        for s in 0..6i32 {
            if hash3(s, 9, slot) < 0.5 {
                let sx = head as i32 + (hash3(s, 10, slot) * 8.0) as i32 - 2;
                let sy = mid as i32 + (hash3(s, 11, slot) * 8.0) as i32 - 4;
                draw::dot_i(grid, sx, sy);
            }
        }
        Ok(())
    }
}

/// Ten wax candles light up one by one: cupped tops, wicks, swaying teardrop flames.
struct Candle;
impl ProgressStyle for Candle {
    fn name(&self) -> &str {
        "candle"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A row of ten wax candles, one more lit per ten percent, flames swaying"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let count = 10usize;
        let lit = (ctx.progress * count as f32).round() as usize;
        let body_top = h.saturating_sub((h / 2).max(1)) as i32;
        let wax = Color::rgb(238, 226, 198);
        let cold_wax = Color::rgb(150, 140, 124);
        for c in 0..count {
            // Each candle is two cells wide, cell-aligned so wax and flame
            // keep their own colors.
            let x0 = (((c as f32 + 0.5) / count as f32 * w as f32) as i32 - 2) & !1;
            let burning = c < lit;
            // Wax body from the floor; a lit candle's top melts into a cup.
            for y in body_top..h as i32 {
                for x in x0..x0 + 4 {
                    let cup = burning && y == body_top && (x == x0 + 1 || x == x0 + 2);
                    if !cup {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
            let tint = if burning { wax } else { cold_wax };
            for y in (body_top.max(0) as usize..h).step_by(4) {
                draw::tint_row(
                    grid,
                    y / 4,
                    (x0.max(0) / 2) as usize,
                    (x0.max(0) / 2 + 1) as usize,
                    tint,
                );
            }
            // Wick.
            draw::dot_i(grid, x0 + 2, body_top - 1);
            if !burning {
                draw::dot_i(grid, x0 + 2, body_top - 2);
                if x0 >= 0 && body_top >= 2 {
                    let _ = grid.set_cell_color(
                        (x0 as usize + 2) / 2,
                        (body_top as usize - 2) / 4,
                        Color::rgb(96, 84, 72),
                    );
                }
                continue;
            }
            // Teardrop flame: wide gold base, tip swaying on its own phase.
            let sway = ((ctx.time * TAU * 0.6 + c as f32 * 1.3).sin() * 1.4).round() as i32;
            let flick = hash3(c as i32, 7, (ctx.time * 8.0) as i32) < 0.6;
            let fy = body_top;
            for (dy, lo, hi, shift) in [
                (2, 1, 2, 0),
                (3, 0, 3, 0),
                (4, 0, 3, 0),
                (5, 1, 2, sway / 2),
                (6, 1, 2, sway),
            ] {
                for x in x0 + lo + shift..=x0 + hi + shift {
                    draw::dot_i(grid, x, fy - dy);
                }
            }
            if flick {
                draw::dot_i(grid, x0 + 1 + sway.max(0), fy - 7);
            }
            for dy in 2..=7 {
                let y = fy - dy;
                let col = if dy <= 4 { HEAT_GOLD } else { HEAT_ORANGE };
                if y >= 0 && x0 + sway.min(0) >= 0 {
                    let cy = y as usize / 4;
                    let lo = ((x0 + sway.min(0)) / 2) as usize;
                    let _ = grid.set_cell_color(lo, cy, col);
                    let _ = grid.set_cell_color(lo + 1, cy, col);
                    let _ = grid.set_cell_color(lo + 2, cy, col);
                }
            }
        }
        Ok(())
    }
}

/// Rollover flames race along the ceiling and bank down; where the heat has
/// hung longest the floor ignites to meet them — a room going up at once.
struct Flashover;
impl ProgressStyle for Flashover {
    fn name(&self) -> &str {
        "flashover"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "Rollover flames racing along the ceiling until the floor ignites to meet them"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (wf, hf) = (w as f32, h as f32);
        let front = ctx.eased * wf;
        let slot = (ctx.time * 10.0) as i32;
        for x in 0..w {
            let xf = x as f32;
            if xf >= front {
                break;
            }
            // How long the heat has hung over this spot (0 at the front).
            let age = (front - xf) / wf.max(1.0);
            let taper = ((front - xf) / 6.0).min(1.0);
            // Ceiling layer: rolling tongues hanging down, banking lower with age.
            let tongue = (xf * 0.35 - ctx.time * TAU * 0.8).sin() * 1.5
                + (xf * 0.9 + ctx.time * TAU * 1.3).sin() * 0.8;
            let depth = ((hf * (0.3 + 0.5 * age) + tongue) * taper).max(0.0);
            for y in 0..(depth as usize).min(h) {
                let edge = depth - y as f32;
                if edge > 1.5 || hash3(x as i32, y as i32, slot) < 0.6 {
                    let heat = 1.0 - y as f32 / hf.max(1.0) * 1.3;
                    let c = sample_tint(0.3 + 0.6 * heat.max(0.0));
                    draw::dot(grid, x, y);
                    let _ = grid.set_cell_color(x / 2, y / 4, c);
                }
            }
            // Floor ignition once the layer has banked low enough.
            let floor = ((age - 0.35) / 0.4).clamp(0.0, 1.0);
            if floor > 0.0 {
                let lick = (0.6 + 0.4 * (xf * 0.5 + ctx.time * TAU).sin().abs())
                    * (0.8 + 0.2 * hash3(x as i32, 99, slot));
                let fh = (hf * 0.7 * floor * lick) as usize;
                for y in h.saturating_sub(fh)..h {
                    draw::dot(grid, x, y);
                    let up = (h - y) as f32 / hf.max(1.0);
                    let _ = grid.set_cell_color(x / 2, y / 4, sample_tint(0.85 - up * 0.7));
                }
            }
        }
        Ok(())
    }
}

/// A fuse burns along the middle: sparks at the front, smoke behind.
struct Charline;
impl ProgressStyle for Charline {
    fn name(&self) -> &str {
        "charline"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A burning fuse with sparks and smoke"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let burn = (ctx.eased * w as f32) as usize;
        let mid = h / 2;
        // Unburnt fuse ahead.
        for x in burn..w {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, Color::rgb(122, 104, 76));
        }
        // Char line behind: the consumed fuse stays as a glowing broken line,
        // so the bar keeps reading at high progress.
        for x in 0..burn {
            if hash2(x as i32, 12) < 0.8 {
                draw::dot(grid, x, mid);
                draw::dot(grid, x, (mid + 1).min(h - 1));
                let glow = 0.1 + 0.25 * hash3(x as i32, 13, (ctx.time * 4.0) as i32);
                let _ = grid.set_cell_color(x / 2, mid / 4, sample_tint(glow));
            }
        }
        // Spark cluster at the burn point.
        let slot = (ctx.time * 12.0) as i32;
        for s in 0..10i32 {
            if hash3(s, 1, slot) < 0.6 {
                let sx = burn as i32 + (hash3(s, 2, slot) * 7.0) as i32 - 3;
                let sy = mid as i32 + (hash3(s, 3, slot) * 7.0) as i32 - 3;
                draw::dot_i(grid, sx, sy);
                if sx >= 0 && sy >= 0 {
                    let _ = grid.set_cell_color(sx as usize / 2, sy as usize / 4, HEAT_GOLD);
                }
            }
        }
        if burn < w {
            let _ = grid.set_cell_color(burn / 2, mid / 4, HEAT_WHITE);
        }
        // Smoke wisps rising over the burnt stretch.
        for x in (0..burn).step_by(4) {
            let age = (burn - x) as f32 / w.max(1) as f32;
            let lift = (age * 8.0 + (ctx.time * TAU * 0.25 + x as f32 * 0.4).sin() * 1.5) as usize;
            let y = mid.saturating_sub(1 + lift.min(mid));
            if hash2(x as i32, 8) < 0.6 {
                draw::dot(grid, x, y);
                let gray = (150.0 - 90.0 * age) as u8;
                let _ = grid.set_cell_color(x / 2, y / 4, Color::rgb(gray, gray, gray));
            }
        }
        Ok(())
    }
}

/// Flame surges past the fill line and retracts, breathing like a backdraft.
struct Backdraft;
impl ProgressStyle for Backdraft {
    fn name(&self) -> &str {
        "backdraft"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A fill that surges and retracts like a backdraft"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let base = ctx.eased * w as f32;
        // The surge: a breathing overshoot ahead of the true fill.
        let surge = (0.5 + 0.5 * (ctx.time * TAU * 0.5).sin()) * w as f32 * 0.08;
        let slot = (ctx.time * 10.0) as i32;
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32;
                if fx < base {
                    draw::dot(grid, x, y);
                } else if fx < base + surge {
                    // Surge zone: ragged, flickering flame.
                    let chance = 1.0 - (fx - base) / surge.max(0.5);
                    if hash3(x as i32, y as i32, slot) < chance * 0.8 {
                        draw::dot(grid, x, y);
                    }
                }
            }
        }
        Ok(())
    }
}

/// A fixed sun's limb at the left edge hurls a side-on eruption to the right:
/// a whipping plasma jet whose reach is the progress, loops lifting off the
/// limb, hot knots racing out and a spray of particles at the jet's tip.
struct SolarFlare;
impl ProgressStyle for SolarFlare {
    fn name(&self) -> &str {
        "solar-flare"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A granulated sun's limb at the left erupts sideways: a whipping plasma jet with hot knots, lifting loops and tip spray reaches out to progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wf, hf) = (w as f32, h as f32);
        let mid = (hf - 1.0) / 2.0;

        // The sun: a round disc parked at the left edge, its centre just
        // inside the bar. It never moves.
        let sr = (mid + 0.8).max(1.5);
        let scx = sr * 0.35;
        let limb_x = |y: f32| -> f32 {
            let dy = y - mid;
            scx + (sr * sr - dy * dy).max(0.0).sqrt()
        };
        let slow = (ctx.time * 3.0) as i32;
        for yi in 0..h as i32 {
            let lx = limb_x(yi as f32);
            let dy = yi as f32 - mid;
            let lx_left = scx - (sr * sr - dy * dy).max(0.0).sqrt();
            for xi in (lx_left.max(0.0) as i32)..=(lx.max(-1.0) as i32) {
                let (ddx, ddy) = (xi as f32 - scx, dy);
                let depth = 1.0 - (ddx * ddx + ddy * ddy).sqrt() / sr; // 0 at the limb
                                                                       // Granulation: a few dark intergranular lanes drift slowly.
                if (0.15..0.5).contains(&depth) && hash3(xi / 2, yi / 2, slow) < 0.18 {
                    continue;
                }
                let c = if depth < 0.15 {
                    HEAT_ORANGE
                } else if depth < 0.5 {
                    HEAT_GOLD
                } else {
                    HEAT_WHITE
                };
                spark(grid, xi as f32, yi as f32, c);
            }
            // Spicules: flickering hairs standing off the limb.
            if lx > 0.0 && hash3(yi, 41, (ctx.time * 8.0) as i32) < 0.3 {
                streak(grid, lx + 2.0, yi as f32, lx + 3.0, yi as f32, HEAT_ORANGE);
            }
        }

        // The eruption's root on the limb and its reach (the progress).
        let x0 = limb_x(mid) + 1.0;
        let max_reach = (wf - x0 - 4.0).max(1.0);
        let reach = 2.0 + ctx.eased * (max_reach - 2.0).max(0.0);

        // Loops lifting off: expanding arcs rooted on the limb, cooling and
        // thinning as they rise, never outrunning the jet.
        for k in 0..3 {
            let p = (ctx.time * 0.35 + k as f32 / 3.0).fract();
            let a = 3.0 + p * reach * 0.7;
            let b = (a * 0.45).min(mid);
            let col = sample_tint(0.85 - 0.6 * p);
            let steps = (a * 2.0) as i32 + 8;
            for s in 0..=steps {
                if p > 0.5 && s % 2 == 1 {
                    continue;
                }
                let th = -TAU / 4.0 + s as f32 / steps as f32 * TAU / 2.0;
                spark(grid, x0 + a * th.cos(), mid + b * th.sin(), col);
            }
        }

        // The jet: a thick plasma column whipping more the further it gets,
        // tapering toward the tip, hotter at the root.
        let seg = reach.max(1.0) as i32;
        for i in 0..=seg {
            let d = i as f32;
            let f = d / reach.max(1.0);
            let y = mid + (d * 0.16 - ctx.time * 5.0).sin() * (hf * 0.22) * f;
            let half = (1.6 * (1.0 - f)).round();
            let col = sample_tint(0.95 - 0.7 * f);
            streak(grid, x0 + d, y - half, x0 + d, y + half, col);
        }

        // Hot knots racing out along the jet.
        for k in 0..3 {
            let d = (ctx.time * 0.9 + k as f32 / 3.0).fract() * reach;
            let f = d / reach.max(1.0);
            let y = mid + (d * 0.16 - ctx.time * 5.0).sin() * (hf * 0.22) * f;
            for (dx, dy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                spark(grid, x0 + d + dx, y - 0.5 + dy, HEAT_WHITE);
            }
        }

        // Particle spray bursting forward from the tip.
        let tip_y = mid + (reach * 0.16 - ctx.time * 5.0).sin() * (hf * 0.22);
        let tip_x = x0 + reach;
        let burst = (ctx.time * 6.0) as i32;
        for s in 0..14 {
            let age = (ctx.time * 1.8 + hash3(s, 7, 0)).fract();
            let ang = (hash3(s, 9, burst / 6) - 0.5) * 1.8;
            let dist = 1.0 + age * 7.0;
            let c = if age < 0.35 {
                HEAT_WHITE
            } else if age < 0.7 {
                HEAT_GOLD
            } else {
                HEAT_ORANGE
            };
            spark(
                grid,
                tip_x + ang.cos() * dist,
                tip_y + ang.sin() * dist * 0.8,
                c,
            );
        }
        Ok(())
    }
}

/// Set a dot from float coordinates and tint its cell, ignoring off-grid.
fn spark(grid: &mut BrailleGrid, x: f32, y: f32, c: Color) {
    if x < 0.0 || y < 0.0 {
        return;
    }
    let (xi, yi) = (x as usize, y as usize);
    draw::dot(grid, xi, yi);
    let _ = grid.set_cell_color(xi / 2, yi / 4, c);
}

/// A straight run of tinted dots from `(x0, y0)` to `(x1, y1)`.
fn streak(grid: &mut BrailleGrid, x0: f32, y0: f32, x1: f32, y1: f32, c: Color) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as i32;
    for k in 0..=steps {
        let t = k as f32 / steps as f32;
        spark(grid, x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, c);
    }
}

/// A firebird flies the bar: beaked head, flapping wings, flame-plume tail.
struct Phoenix;
impl ProgressStyle for Phoenix {
    fn name(&self) -> &str {
        "phoenix"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A firebird beating its wings, tail plumes streaming fire behind it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (wf, hf) = (w as f32, h as f32);
        let cy = hf * 0.55;
        // The bird's beak rides the progress point; it starts fully on-screen.
        let hx = 12.0 + ctx.eased * (wf - 18.0).max(0.0);
        let s = (hf / 16.0).clamp(0.25, 1.0);
        // Tail plumes: three wavy flame ribbons streaming back to the start,
        // cooling from gold at the body to char red at the far end.
        let tail0 = hx - 10.0 * s;
        for plume in 0..3 {
            let off = (plume as f32 - 1.0) * 2.0 * s;
            let start = if tail0 >= 0.0 { tail0 as i32 } else { -1 };
            for xi in (0..=start).rev() {
                let x = xi as f32;
                let back = tail0 - x;
                let amp = (back * 0.1).min(2.5) * s;
                let wave = (back * 0.2 - ctx.time * TAU * 0.9).sin();
                let y = cy + 0.5 + off * (1.0 + back * 0.05).min(2.2) + wave * amp;
                let heat = 1.0 - back / wf.max(1.0) * 1.4;
                if hash3(x as i32, plume, (ctx.time * 10.0) as i32) < 0.3 + heat {
                    spark(grid, x, y, sample_tint(0.15 + 0.75 * heat.max(0.0)));
                }
            }
        }
        // Body: a solid oval, white-hot at the breast.
        let (bx, rx, ry) = (hx - 6.5 * s, 4.5 * s, 2.2 * s);
        for yi in (cy - ry).floor() as i32..=(cy + ry).floor() as i32 {
            for xi in (bx - rx).floor() as i32..=(bx + rx).floor() as i32 {
                let (x, y) = (xi as f32, yi as f32);
                let (dx, dy) = ((x - bx) / rx.max(0.5), (y - cy) / ry.max(0.5));
                if dx * dx + dy * dy <= 1.0 {
                    spark(grid, x, y, HEAT_WHITE);
                }
            }
        }
        // Wing: a solid swept triangle from the shoulder, drawn over the body
        // and beating with time — high on the up-stroke, below the belly on
        // the down-stroke. The trailing edge is notched into feathers.
        let lift = 0.3 + 0.7 * (ctx.time * TAU * 1.1).sin();
        let root = cy - 1.0;
        let (ax, bx0) = (hx - 4.0 * s, hx - 13.0 * s);
        let (tx, ty) = (hx - 10.0 * s, root - lift * 9.0 * s);
        let (ylo, yhi) = (ty.min(root).floor() as i32, ty.max(root).ceil() as i32);
        for yi in ylo..=yhi {
            let y = yi as f32;
            // Fraction of the way from the root line to the tip.
            let k = if (ty - root).abs() < 0.5 {
                0.0
            } else {
                ((y - root) / (ty - root)).clamp(0.0, 1.0)
            };
            let (x0, x1) = (bx0 + (tx - bx0) * k, ax + (tx - ax) * k);
            for xi in x0.floor() as i32..=x1.floor() as i32 {
                let x = xi as f32;
                let notch = x - x0 < 2.5 * s && (xi + yi).rem_euclid(3) == 0;
                if !notch {
                    let c = if k > 0.55 { HEAT_ORANGE } else { HEAT_GOLD };
                    spark(grid, x, y, c);
                }
            }
        }
        // Head, beak and a swept-back crest.
        let hy = cy - 3.0 * s;
        streak(grid, bx + rx - 1.0, cy - 1.0, hx - s, hy, HEAT_WHITE);
        for dy in -1..=1 {
            for dx in -2..=0 {
                spark(grid, hx + dx as f32 * s, hy + dy as f32, HEAT_WHITE);
            }
        }
        streak(grid, hx + 1.0, hy, hx + 2.5 * s, hy + 1.0, HEAT_GOLD);
        let comb = (ctx.time * TAU * 2.0).sin() * 0.6;
        streak(
            grid,
            hx - 1.5 * s,
            hy - 1.5,
            hx - 4.0 * s,
            hy - 3.5 * s + comb,
            HEAT_ORANGE,
        );
        // The rebirth burst at (nearly) full: flame rays from the bird.
        if ctx.progress > 0.94 {
            let strength = (ctx.progress - 0.94) / 0.06;
            let len = strength * hf * 0.9;
            for ray in 0..14 {
                let ang = ray as f32 * TAU / 14.0 + ctx.time * TAU * 0.25;
                let l = len * (0.6 + 0.4 * hash2(ray, 3));
                let (x0, y0) = (bx + ang.cos() * 3.0 * s, cy + ang.sin() * 1.5 * s);
                let (x1, y1) = (bx + ang.cos() * l * 1.6, cy + ang.sin() * l * 0.8);
                streak(grid, x0, y0, x1, y1, sample_tint(0.9 - 0.4 * hash2(ray, 5)));
            }
        }
        Ok(())
    }
}
