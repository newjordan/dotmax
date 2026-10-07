//! Nature / weather / seasons progress bars.
//!
//! Ten animated styles built entirely from `draw::` helpers. Every bar reads
//! `ctx.eased` for its fill/growth amount and `ctx.time` for looping
//! animation, so they look alive even when progress is held constant.
//! Most styles paint their own natural colours (sky, sea, rock, snow, leaves)
//! cell by cell; the rain gauge still tints its water from the bar palette.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── registry ─────────────────────────────────────────────────────────────────

/// All styles in the `nature` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(GrowingVine),
        Box::new(GrassBlade),
        Box::new(Sunrise),
        Box::new(RainGauge),
        Box::new(TreeRings),
        Box::new(MountainSnow),
        Box::new(FourSeasons),
        Box::new(FlowerBloom),
        Box::new(FallingLeaves),
        Box::new(LightningBolt),
    ]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Integer sine rounded to i32 — keeps per-bar code terse.
#[inline]
fn isin(angle: f32, amplitude: f32) -> i32 {
    (angle.sin() * amplitude).round() as i32
}

/// Tint the cell containing dot `(x, y)`; out-of-range dots are ignored.
fn tint_at(grid: &mut BrailleGrid, x: i32, y: i32, color: Color) {
    if x >= 0 && y >= 0 {
        let (cx, cy) = (x as usize / 2, y as usize / 4);
        draw::tint_row(grid, cy, cx, cx, color);
    }
}

/// Leaf green ramp: `0.0` deep forest green → `1.0` fresh yellow-green.
fn leafy(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::rgb(
        (40.0 + 130.0 * t) as u8,
        (130.0 + 90.0 * t) as u8,
        (50.0 + 30.0 * t) as u8,
    )
}

/// Set a dot and tint its cell in one go.
fn paint(grid: &mut BrailleGrid, x: i32, y: i32, color: Color) {
    draw::dot_i(grid, x, y);
    tint_at(grid, x, y, color);
}

/// Straight line of tinted dots from `a` to `b` (inclusive), `thick` dots wide.
fn line(grid: &mut BrailleGrid, a: (i32, i32), b: (i32, i32), thick: i32, color: Color) {
    let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
    for s in 0..=steps {
        let x = a.0 + (b.0 - a.0) * s / steps;
        let y = a.1 + (b.1 - a.1) * s / steps;
        for k in 0..thick.max(1) {
            paint(grid, x + k, y, color);
        }
    }
}

// ── 1. Growing vine ──────────────────────────────────────────────────────────

struct GrowingVine;
impl ProgressStyle for GrowingVine {
    fn name(&self) -> &str {
        "growing-vine"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A vine climbs left-to-right; leaf crosses sprout at every 10% milestone"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = (h / 2) as i32;
        let reach = (ctx.eased * w as f32).round() as usize;

        // Sine-swaying stem.
        for x in 0..reach {
            let sway = isin(x as f32 * 0.3 + ctx.time * 2.0, (h as f32 * 0.2).max(1.0));
            draw::dot_i(grid, x as i32, mid + sway);
        }

        // Leaf crosses at each completed 10% threshold.
        for step in 1..=10usize {
            let threshold = step as f32 * 0.1;
            if ctx.eased < threshold {
                break;
            }
            let lx = (threshold * w as f32).round() as i32;
            let sway = isin(lx as f32 * 0.3 + ctx.time * 2.0, (h as f32 * 0.2).max(1.0));
            let ly = mid + sway;
            // Small cross: horizontal + vertical arms of length 2.
            for dx in -2i32..=2 {
                draw::dot_i(grid, lx + dx, ly);
            }
            for dy in -2i32..=2 {
                draw::dot_i(grid, lx, ly + dy);
            }
        }

        // Tint the filled span green.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        let color = leafy(0.4);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, filled_cells.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 2. Grass blades ──────────────────────────────────────────────────────────

struct GrassBlade;
impl ProgressStyle for GrassBlade {
    fn name(&self) -> &str {
        "grass-blades"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Individual grass blades grow taller with progress and sway in the wind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let blade_spacing = 3usize;
        let base_y = h.saturating_sub(1);
        let max_growth = h.saturating_sub(1);

        let blade_count = w / blade_spacing;
        for b in 0..blade_count {
            let bx = b * blade_spacing + blade_spacing / 2;
            // Each blade grows to a slightly different height (pseudo-random via hash).
            let variety = ((b as f32 * 7.3).sin() * 0.5 + 0.5) * 0.4 + 0.6; // 0.6..1.0
            let height = (ctx.eased * max_growth as f32 * variety).round() as usize;
            if height == 0 {
                continue;
            }

            // Sway: low-frequency sine, each blade phase-shifted by position.
            let sway = isin(ctx.time * 1.8 + bx as f32 * 0.5, (h as f32 * 0.12).max(1.0));
            let tip_x = (bx as i32 + sway).max(0) as usize;

            let top_y = base_y.saturating_sub(height);
            // Stem.
            for y in top_y..=base_y {
                let frac = (base_y - y) as f32 / height.max(1) as f32;
                let sx = (bx as f32 + sway as f32 * frac).round() as usize;
                draw::dot(grid, sx.min(w.saturating_sub(1)), y);
            }
            // Tip dot — slightly to the side.
            draw::dot(grid, tip_x.min(w.saturating_sub(1)), top_y);
        }

        // Gradient tint.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if cw <= 1 {
                0.0
            } else {
                cx as f32 / (cw - 1) as f32
            };
            let color = leafy(0.2 + 0.7 * t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        // Not-yet-grown stubble stays a dry straw colour.
        for cy in 0..ch {
            draw::tint_row(
                grid,
                cy,
                filled_cells,
                cw.saturating_sub(1),
                Color::rgb(130, 120, 75),
            );
        }

        Ok(())
    }
}

// ── 3. Sunrise ───────────────────────────────────────────────────────────────

struct Sunrise;
impl ProgressStyle for Sunrise {
    fn name(&self) -> &str {
        "sunrise"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A rayed sun climbs out of the sea, red at dawn turning gold, its glitter shimmering on the water"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let p = ctx.eased;
        let horizon = ((h as f32 * 0.66) as i32).max(1);
        let r = (h as f32 * 0.3).min(w as f32 * 0.2).max(1.0);
        // Sun climbs from just below the horizon to high in the sky while
        // drifting right, so 20% and 90% sit in clearly different places.
        let sx = w as f32 * (0.12 + 0.5 * p);
        let sy = horizon as f32 + r * 0.6 - p * (horizon as f32 - r * 0.2);
        let sun = Color::rgb(255, (90.0 + 150.0 * p) as u8, (40.0 * p) as u8);
        let ray = Color::rgb(255, (60.0 + 120.0 * p) as u8, 20);
        let sea = Color::rgb(40, 80, 160);

        // Sea: dashed swell lines scrolling with time below the horizon.
        for y in horizon..h as i32 {
            let row = y - horizon;
            for x in 0..w as i32 {
                let phase =
                    (x + row * 5 + (ctx.time * 6.0) as i32 * if row % 2 == 0 { 1 } else { -1 })
                        .rem_euclid(9);
                if row == 0 || phase < 4 {
                    paint(grid, x, y, sea);
                }
            }
        }

        // Rays: 12 spokes that rotate slowly and pulse in length.
        let pulse = (ctx.time * 3.0).sin() * 0.5 + 0.5;
        for k in 0..12 {
            let a = k as f32 / 12.0 * 2.0 * PI + ctx.time * 0.25;
            let r0 = r + 1.5;
            let r1 = r + 2.5 + pulse * 2.0 * if k % 2 == 0 { 1.0 } else { 0.5 };
            for step in 0..=((r1 - r0) * 2.0) as i32 {
                let d = r0 + step as f32 * 0.5;
                let x = (sx + a.cos() * d * 1.2).round() as i32;
                let y = (sy + a.sin() * d).round() as i32;
                if y < horizon {
                    paint(grid, x, y, ray);
                }
            }
        }

        // Sun disc, clipped by the horizon.
        let (x0, x1) = ((sx - r * 1.2).floor() as i32, (sx + r * 1.2).ceil() as i32);
        let (y0, y1) = ((sy - r).floor() as i32, (sy + r).ceil() as i32);
        for y in y0..=y1.min(horizon - 1) {
            for x in x0..=x1 {
                let dx = (x as f32 - sx) / (r * 1.2);
                let dy = (y as f32 - sy) / r;
                if dx * dx + dy * dy <= 1.0 {
                    paint(grid, x, y, sun);
                }
            }
        }

        // Glitter path: flickering dashes on the water under the sun.
        for y in horizon + 1..h as i32 {
            let spread = (r * 1.4 * (1.0 - (y - horizon) as f32 * 0.12)).max(1.0) as i32;
            for x in (sx as i32 - spread)..=(sx as i32 + spread) {
                let n = (x * 7 + y * 13 + (ctx.time * 8.0) as i32 * 5).rem_euclid(5);
                if n < 2 {
                    paint(grid, x, y, ray);
                }
            }
        }

        Ok(())
    }
}

// ── 4. Rain gauge ────────────────────────────────────────────────────────────

struct RainGauge;
impl ProgressStyle for RainGauge {
    fn name(&self) -> &str {
        "rain-gauge"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Raindrops fall from a cloud; the water level rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Gauge outline.
        draw::rect_outline(grid, 0, 0, w, h);

        // Water fill from the bottom up.
        let fill_h = (ctx.eased * (h.saturating_sub(2)) as f32).round() as usize;
        let water_top = h.saturating_sub(1 + fill_h);
        if fill_h > 0 {
            draw::fill_rect(grid, 1, water_top, w.saturating_sub(2).max(1), fill_h);
        }

        // Animated raindrops — 5 drops cycling at different rates.
        let unfilled_h = water_top;
        if unfilled_h > 1 {
            for d in 0..5usize {
                let phase = d as f32 / 5.0;
                let drop_cycle = (ctx.time * 1.5 + phase).fract();
                let drop_y = (drop_cycle * unfilled_h as f32) as usize;
                let drop_x =
                    (((d as f32 * 1.618 + 0.5) % 1.0) * (w.saturating_sub(2)) as f32) as usize + 1;
                // Draw as a 1-dot pip.
                draw::dot(
                    grid,
                    drop_x.min(w.saturating_sub(2)),
                    drop_y.min(unfilled_h.saturating_sub(1)),
                );
            }
        }

        // Tint water blue via palette.
        let (cw, ch) = grid.dimensions();
        let water_top_cell = (water_top / 4).min(ch.saturating_sub(1));
        for cy in water_top_cell..ch {
            let t = (cy - water_top_cell) as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 5. Tree rings ────────────────────────────────────────────────────────────

struct TreeRings;
impl ProgressStyle for TreeRings {
    fn name(&self) -> &str {
        "tree-rings"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A sawn log grows longer with progress; its cut end shows growth rings that add up year by year as sawdust falls"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let bark = Color::rgb(120, 75, 40);
        let wood = Color::rgb(225, 180, 120);
        let ring = Color::rgb(170, 110, 60);
        let ry = (h as f32 / 2.0 - 0.5).max(0.5);
        let rx = ry.max(0.5);
        let cy = h as f32 / 2.0 - 0.5;
        // The cut face slides right as the log lengthens.
        let ex = rx + ctx.eased * (w as f32 - 2.0 * rx - 1.0).max(0.0);

        // Bark body: top and bottom edges plus wavy grain lines.
        for x in 0..ex as i32 {
            paint(grid, x, 0, bark);
            paint(grid, x, h as i32 - 1, bark);
            for k in 1..3 {
                if x as f32 >= ex - rx {
                    break;
                }
                let gy = (h as f32 * k as f32 / 3.0
                    + (x as f32 * 0.25 + k as f32 * 2.0).sin() * 1.2)
                    as i32;
                if (x + k * 5) % 9 < 6 {
                    paint(grid, x, gy, bark);
                }
            }
        }

        // End face: pale wood with concentric rings, more rings with progress.
        let years = 1 + (ctx.eased * 2.5) as i32;
        let (x0, x1) = ((ex - rx).floor() as i32, (ex + rx).ceil() as i32);
        for y in 0..h as i32 {
            for x in x0..=x1 {
                let fx = (x as f32 - ex) / rx;
                let fy = (y as f32 - cy) / ry;
                let d = (fx * fx + fy * fy).sqrt();
                if d > 1.0 {
                    continue;
                }
                let band = (d * (years as f32 + 0.5)).fract();
                if d > 0.88 {
                    paint(grid, x, y, bark);
                } else if band < 0.3 || d < 0.15 {
                    paint(grid, x, y, ring);
                } else {
                    tint_at(grid, x, y, wood);
                }
            }
        }

        // Sawdust sprinkling down from the fresh cut.
        for k in 0..4 {
            let t = (ctx.time * 1.3 + k as f32 * 0.25).fract();
            let sx = (ex + rx + 1.0 + k as f32 + t * 2.0) as i32;
            let sy = (cy + t * (h as f32 / 2.0)) as i32;
            paint(grid, sx, sy, wood);
        }

        Ok(())
    }
}

// ── 6. Mountain snow ─────────────────────────────────────────────────────────

struct MountainSnow;
impl ProgressStyle for MountainSnow {
    fn name(&self) -> &str {
        "mountain-snow"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A jagged grey mountain range under falling snowflakes; white snowcaps creep down the slopes with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let rock = Color::rgb(95, 105, 135);
        let snow = Color::rgb(240, 245, 255);
        let hf = h as f32;
        // Peaks: (x fraction, height fraction) — a jagged range.
        let peaks = [
            (0.1, 0.55),
            (0.27, 0.95),
            (0.42, 0.65),
            (0.58, 1.0),
            (0.74, 0.7),
            (0.9, 0.85),
        ];
        let ridge = |x: f32| -> f32 {
            let mut top = 0.0f32;
            for &(px, ph) in &peaks {
                let d = (x - px * w as f32).abs();
                top = top.max(ph * hf - d * 0.75);
            }
            // Small crags on the slopes.
            top + (x * 0.9).sin() * 0.6
        };
        // Snowline falls from the summits towards the valleys.
        let snow_depth = 1.0 + ctx.eased.powf(1.5) * hf * 0.65;

        for x in 0..w as i32 {
            let top = ridge(x as f32);
            let peak_y = hf - top;
            for y in peak_y.max(0.0).ceil() as i32..h as i32 {
                // Snow edge is ragged: it dips deeper on some columns.
                let ragged = ((x * 13) % 5) as f32 * 0.4;
                let snowy = (y as f32 - peak_y) < snow_depth - ragged && (y as f32) < hf - 1.0;
                if snowy {
                    paint(grid, x, y, snow);
                } else if (x + y) % 2 == 0 || y as f32 - peak_y < 1.0 {
                    // Rock: an outlined, hatched face.
                    paint(grid, x, y, rock);
                }
            }
        }

        // Snowflakes drifting down over the range.
        for i in 0..(w / 6).max(1) as i32 {
            let t = (ctx.time * 0.4 + i as f32 * 0.29).fract();
            let x = ((i * 23) % w as i32) as f32 + (ctx.time * 2.0 + i as f32).sin() * 2.0;
            let y = t * hf;
            let (xi, yi) = (x.round() as i32, y as i32);
            if yi as f32 + 1.0 < hf - ridge(x) {
                paint(grid, xi, yi, snow);
            }
        }

        Ok(())
    }
}

// ── 7. Four seasons ──────────────────────────────────────────────────────────

struct FourSeasons;
impl ProgressStyle for FourSeasons {
    fn name(&self) -> &str {
        "four-seasons"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "The year sweeps by in four hatched bands, each revealing its emblem: spring flower, summer sun, autumn leaf, winter snowflake"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let filled = (ctx.eased * w as f32).round() as usize;

        // Season bands: a light diagonal hatch, one quarter per season.
        let season_colors = [
            Color::rgb(60, 160, 60),   // spring
            Color::rgb(220, 170, 40),  // summer
            Color::rgb(180, 90, 45),   // autumn
            Color::rgb(110, 160, 210), // winter
        ];
        let quarter = (w as f32 / 4.0).max(1.0);
        for x in 0..filled as i32 {
            let k = ((x as f32 / quarter) as usize).min(3);
            for y in 0..h as i32 {
                if (x + y).rem_euclid(4) == 0 {
                    paint(grid, x, y, season_colors[k]);
                }
            }
        }

        // Each season's emblem appears once the fill has swept past it:
        // a flower, a rayed sun, a falling leaf and a snowflake.
        let cy = h as f32 / 2.0 - 0.5;
        for k in 0..4 {
            let cx = quarter * (k as f32 + 0.5);
            if (filled as f32) < cx + 4.0 {
                break;
            }
            let (x0, y0) = (cx.round() as i32, cy.round() as i32);
            match k {
                0 => {
                    // Flower: petal ring on a stem with a leaf.
                    let pink = Color::rgb(255, 120, 180);
                    let stem = leafy(0.5);
                    for y in y0..h as i32 {
                        paint(grid, x0, y, stem);
                    }
                    paint(grid, x0 + 1, y0 + 4, stem);
                    paint(grid, x0 + 2, y0 + 3, stem);
                    let sway = isin(ctx.time * 2.0, 1.0);
                    for j in 0..6 {
                        let a = j as f32 / 6.0 * 2.0 * PI;
                        paint(
                            grid,
                            x0 + sway + (a.cos() * 2.4).round() as i32,
                            y0 - 2 + (a.sin() * 2.0).round() as i32,
                            pink,
                        );
                    }
                    paint(grid, x0 + sway, y0 - 2, Color::rgb(255, 220, 60));
                }
                1 => {
                    // Sun: a disc with turning rays.
                    let sun = Color::rgb(255, 225, 60);
                    for dy in -2..=2i32 {
                        for dx in -3..=3i32 {
                            if dx * dx + 2 * dy * dy <= 9 {
                                paint(grid, x0 + dx, y0 + dy, sun);
                            }
                        }
                    }
                    for j in 0..8 {
                        let a = j as f32 / 8.0 * 2.0 * PI + ctx.time * 0.6;
                        for d in [4.5f32, 5.5] {
                            paint(
                                grid,
                                x0 + (a.cos() * d * 1.3).round() as i32,
                                y0 + (a.sin() * d).round() as i32,
                                sun,
                            );
                        }
                    }
                }
                2 => {
                    // Leaf: a pointed blade with a stem, bobbing as it falls.
                    let leaf = Color::rgb(240, 110, 30);
                    let bob = isin(ctx.time * 2.0, 1.5);
                    let (lx, ly) = (x0, y0 + bob);
                    for (dy, half) in [(-3, 0), (-2, 1), (-1, 2), (0, 3), (1, 2), (2, 1)] {
                        for dx in -half..=half {
                            paint(grid, lx + dx, ly + dy, leaf);
                        }
                    }
                    paint(grid, lx, ly + 3, leaf);
                    paint(grid, lx + 1, ly + 4, leaf);
                }
                _ => {
                    // Snowflake: six arms with little side branches, turning.
                    let snow = Color::rgb(240, 248, 255);
                    paint(grid, x0, y0, snow);
                    for j in 0..6 {
                        let a = j as f32 / 6.0 * PI * 2.0 + ctx.time * 0.5;
                        for d in 1..=5 {
                            let d = d as f32 * 0.9;
                            paint(
                                grid,
                                x0 + (a.cos() * d * 1.2).round() as i32,
                                y0 + (a.sin() * d).round() as i32,
                                snow,
                            );
                        }
                        let (bx, by) = (x0 as f32 + a.cos() * 3.0 * 1.2, y0 as f32 + a.sin() * 3.0);
                        for s in [-1.0f32, 1.0] {
                            let b = a + s * 0.9;
                            paint(
                                grid,
                                (bx + b.cos() * 1.5 * 1.2).round() as i32,
                                (by + b.sin() * 1.5).round() as i32,
                                snow,
                            );
                        }
                    }
                }
            }
        }

        // Track outline.
        draw::hline(grid, 0, w.saturating_sub(1), 0);
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        Ok(())
    }
}

// ── 8. Flower bloom ──────────────────────────────────────────────────────────

struct FlowerBloom;
impl ProgressStyle for FlowerBloom {
    fn name(&self) -> &str {
        "flower-bloom"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A row of tulips sprouts left to right; each bud swells and opens into a three-petal cup, swaying"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        const PETALS: [Color; 4] = [
            Color::rgb(235, 50, 70),
            Color::rgb(255, 120, 190),
            Color::rgb(255, 200, 40),
            Color::rgb(170, 90, 230),
        ];
        let stem = Color::rgb(70, 180, 70);
        let ground = h as i32 - 1;
        let n = (w / 12).max(1);
        let spacing = w as f32 / n as f32;
        let cup_h = (h as f32 * 0.4).max(2.0);

        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let base_x = (spacing * (i as f32 + 0.5)) as i32;
            let sway = isin(ctx.time * 1.6 + i as f32 * 0.9, 1.0);
            // Stem grows to full height in the first half, then the bud opens.
            let grow = (local * 2.0).min(1.0);
            let open = ((local - 0.5) * 2.0).clamp(0.0, 1.0);
            let top = ground - ((h as f32 - cup_h - 1.0).max(1.0) * grow) as i32;
            for y in top..=ground {
                let x = base_x + if y < ground - 3 { sway } else { 0 };
                paint(grid, x, y, stem);
            }
            // A long leaf curling off the stem near the ground.
            if grow > 0.5 {
                let side = if i % 2 == 0 { 1 } else { -1 };
                for k in 1..=3 {
                    paint(grid, base_x + side * k, ground - k + 1, stem);
                }
            }
            if grow < 1.0 {
                continue;
            }

            // Cup: straight-sided top flaring as it opens, rounded bottom.
            let cx = base_x + sway;
            let cup_top = top - cup_h as i32;
            let rx = 1.5 + 1.5 * open + local;
            let rx_top = rx * (0.2 + 1.0 * open);
            let color = PETALS[i % PETALS.len()];
            let rows = cup_h as i32;
            for dy in 0..=rows {
                let f = dy as f32 / rows as f32;
                let hw = if f < 0.6 {
                    rx_top + (rx - rx_top) * f / 0.6
                } else {
                    rx * (1.0 - ((f - 0.6) / 0.4).powi(2)).max(0.0).sqrt()
                };
                let hwi = hw.round() as i32;
                for dx in -hwi..=hwi {
                    // Notches between the three petal tips once open.
                    let notch = open > 0.3
                        && dy < 2
                        && dx.abs() as f32 > hw * 0.25
                        && (dx.abs() as f32) < hw * 0.65;
                    if !notch {
                        paint(grid, cx + dx, cup_top + dy, color);
                    }
                }
            }
        }

        Ok(())
    }
}

// ── 9. Falling leaves ────────────────────────────────────────────────────────

struct FallingLeaves;
impl ProgressStyle for FallingLeaves {
    fn name(&self) -> &str {
        "falling-leaves"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Red, orange and gold leaves tumble down swaying and pile up into a heap that spreads with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        const AUTUMN: [Color; 4] = [
            Color::rgb(220, 60, 40),
            Color::rgb(245, 140, 30),
            Color::rgb(240, 200, 50),
            Color::rgb(170, 90, 40),
        ];
        let ground = h as i32 - 1;
        let reach = (ctx.eased * w as f32) as i32;

        // Leaf pile: a lumpy heap out to `reach`, tapering at the front edge.
        for x in 0..reach {
            let lump = ((x as f32 * 0.7).sin() + (x as f32 * 0.23).sin()) * 0.6;
            let taper = ((reach - x) as f32 / 4.0).min(1.0);
            let height = ((2.5 + lump) * taper).max(1.0) as i32;
            let c = AUTUMN[((x / 2) as usize * 7 + 3) % AUTUMN.len()];
            for y in (ground - height + 1)..=ground {
                if (x + y) % 4 != 0 {
                    paint(grid, x, y, c);
                }
            }
        }

        // Falling leaves: each drifts down on its own lane, rocking side to side.
        let count = 6 + (w / 12) as i32;
        for i in 0..count {
            let speed = 0.35 + (i % 3) as f32 * 0.12;
            let t = (ctx.time * speed + i as f32 * 0.37).fract();
            let lane = ((i * 37) % w.max(1) as i32) as f32;
            let x = lane + (ctx.time * 2.5 + i as f32).sin() * 3.0 + t * 6.0;
            let y = t * (ground - 2) as f32;
            let (lx, ly) = (x.round() as i32, y.round() as i32);
            let c = AUTUMN[i as usize % AUTUMN.len()];
            // A tiny leaf that flips between two poses as it tumbles.
            if (ctx.time * 3.0 + i as f32).sin() > 0.0 {
                for &(dx, dy) in &[(0, 0), (1, 0), (2, 0), (1, -1), (2, -1), (0, 1)] {
                    paint(grid, lx + dx, ly + dy, c);
                }
            } else {
                for &(dx, dy) in &[(0, 0), (1, 0), (1, 1), (0, -1), (1, -1), (2, 1)] {
                    paint(grid, lx + dx, ly + dy, c);
                }
            }
        }

        Ok(())
    }
}

// ── 10. Lightning bolt ───────────────────────────────────────────────────────

struct LightningBolt;
impl ProgressStyle for LightningBolt {
    fn name(&self) -> &str {
        "lightning-bolt"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A storm cloud rolls in with progress; zigzag lightning bolts strike the ground from its leading edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cloud = Color::rgb(150, 150, 170);
        let bolt = Color::rgb(255, 235, 90);
        let reach = (ctx.eased * w as f32).max(6.0);
        let base = (h as f32 * 0.36).max(1.0);

        // Cloud bank: a row of overlapping puffs out to `reach`, billowing.
        let puffs = ((reach / 7.0) as i32).max(1);
        for i in 0..puffs {
            let cx = 3.0 + i as f32 * (reach - 3.0) / puffs as f32;
            let r = base
                * (0.55
                    + 0.3 * (i as f32 * 2.3).sin().abs()
                    + 0.08 * (ctx.time * 1.5 + i as f32).sin());
            let (x0, x1) = ((cx - r * 1.6) as i32, (cx + r * 1.6) as i32);
            for y in 0..=base as i32 {
                for x in x0..=x1.min(reach as i32) {
                    let dx = (x as f32 - cx) / (r * 1.6);
                    let dy = (y as f32 - base * 0.75) / r;
                    if dx * dx + dy * dy <= 1.0 {
                        paint(grid, x, y, cloud);
                    }
                }
            }
        }

        // Bolts flash on and off; each strike gets a fresh jagged shape.
        let beat = ctx.time * 2.2;
        let strike = beat.floor() as i32;
        if beat.fract() < 0.45 {
            for b in 0..2 {
                let seed = (strike * 31 + b * 17).rem_euclid(97);
                let x0 = if b == 0 {
                    reach - 6.0
                } else {
                    (seed as f32 / 97.0) * (reach - 6.0)
                } as i32;
                if b == 1 && (reach < 30.0 || seed % 3 == 0) {
                    continue;
                }
                let top = base as i32 + 1;
                let bottom = h as i32 - 1;
                let span = (bottom - top).max(1);
                let jog = 2 + seed % 3;
                // Classic bolt: slant right, kick back left, slant right again.
                let p1 = (x0 + jog + 1, top + span * 2 / 5);
                let p2 = (x0 - 1, top + span * 3 / 5);
                let p3 = (x0 + jog, bottom);
                line(grid, (x0, top), p1, 2, bolt);
                line(grid, p1, p2, 1, bolt);
                line(grid, p2, p3, 2, bolt);
                // Impact sparks.
                if beat.fract() < 0.2 {
                    paint(grid, p3.0 - 2, bottom, bolt);
                    paint(grid, p3.0 + 3, bottom, bolt);
                }
            }
        }

        Ok(())
    }
}
