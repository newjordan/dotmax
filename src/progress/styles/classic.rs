//! Classic / professional progress bars — the reference theme.
//!
//! These six styles demonstrate every technique the themed bars build on:
//! solid fill, gradient tinting, discrete blocks, a scrolling indeterminate
//! shimmer, a rocket whose exhaust trail is the fill, and a ticked percentage
//! track. Copy any one of these as a starting point for a new bar.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// Fast integer hash → `[0, 1)`.
#[inline]
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) % 1000) as f32 / 1000.0
}

/// 3-D variant: hash `(x, y, z_int)` for time-slotted sparks.
#[inline]
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash2(x ^ z.wrapping_mul(1_234_567), y ^ z.wrapping_mul(7_654_321))
}

// ---------------------------------------------------------------------------
// Theme tint — the default cyan into violet. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(0, 200, 255);
const TINT_END: Color = Color::rgb(120, 80, 255);

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

/// All styles in the `classic` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(SolidFill)),
        Box::new(Gradient),
        Box::new(Tinted(Blocks)),
        Box::new(Tinted(Shimmer)),
        Box::new(Rocket),
        Box::new(Tinted(TickedTrack)),
    ]
}

/// A plain filled bar with a one-dot border.
struct SolidFill;
impl ProgressStyle for SolidFill {
    fn name(&self) -> &str {
        "solid"
    }
    fn theme(&self) -> &str {
        "classic"
    }
    fn describe(&self) -> &str {
        "Bordered solid fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        draw::rect_outline(grid, 0, 0, w, h);
        if w > 4 && h > 4 {
            let inner = w - 4;
            let filled = (ctx.eased * inner as f32).round() as usize;
            draw::fill_rect(grid, 2, 2, filled, h - 4);
            // The leading edge breathes so the bar reads as alive, not stuck.
            if filled > 0 && filled < inner {
                let mid = h / 2;
                let reach = (h as f32 * (0.2 + 0.14 * (ctx.time * PI).sin())) as usize;
                draw::vline(
                    grid,
                    2 + filled,
                    mid.saturating_sub(reach),
                    (mid + reach).min(h - 3),
                );
            }
        }
        Ok(())
    }
}

/// A solid fill tinted with the palette's start→end gradient.
struct Gradient;
impl ProgressStyle for Gradient {
    fn name(&self) -> &str {
        "gradient"
    }
    fn theme(&self) -> &str {
        "classic"
    }
    fn describe(&self) -> &str {
        "Gradient-tinted fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let filled = (ctx.eased * w as f32).round() as usize;
        // Liquid fill: the surface ripples gently instead of sitting flat.
        for x in 0..filled {
            let lift = ((x as f32 * 0.3 + ctx.time * PI).sin() * 1.4 + 1.5) as usize;
            for y in lift.min(3)..h {
                draw::dot(grid, x, y);
            }
        }
        // Tint per cell-column across the filled span.
        let (cells_w, _) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32).round() as usize;
        for cx in 0..filled_cells {
            let t = if filled_cells <= 1 {
                0.0
            } else {
                cx as f32 / (filled_cells - 1) as f32
            };
            let color = ctx.palette.sample(t);
            let (_, cells_h) = grid.dimensions();
            draw::tint_row(grid, cells_h / 2, cx, cx, color);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

/// Discrete blocks with a one-dot gap, like a segmented LED meter.
struct Blocks;
impl ProgressStyle for Blocks {
    fn name(&self) -> &str {
        "blocks"
    }
    fn theme(&self) -> &str {
        "classic"
    }
    fn describe(&self) -> &str {
        "Segmented block meter"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let seg_count = 16usize;
        let seg_w = (w / seg_count).max(1);
        let gap = 1;
        let lit = (ctx.eased * seg_count as f32).round() as usize;
        for s in 0..lit.min(seg_count) {
            let x0 = s * seg_w;
            let bw = seg_w.saturating_sub(gap).max(1);
            let full_h = h.saturating_sub(2).max(1);
            // The newest segment charges up from the baseline once a second —
            // classic LED meter behavior.
            if s + 1 == lit && lit < seg_count {
                let charge = (ctx.time * 1.0).fract();
                let ch = ((full_h as f32) * charge).max(1.0) as usize;
                draw::fill_rect(grid, x0, 1 + (full_h - ch), bw, ch);
            } else {
                draw::fill_rect(grid, x0, 1, bw, full_h);
            }
        }
        Ok(())
    }
}

/// An indeterminate scrolling shimmer (uses `time`, ignores `progress`).
struct Shimmer;
impl ProgressStyle for Shimmer {
    fn name(&self) -> &str {
        "shimmer"
    }
    fn theme(&self) -> &str {
        "classic"
    }
    fn describe(&self) -> &str {
        "Indeterminate scrolling shimmer"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let band = (w / 4).max(3);
        let travel = w + band;
        let head = ((ctx.time * 0.6).fract() * travel as f32) as i32 - band as i32;
        for i in 0..band as i32 {
            let x = head + i;
            // Triangular intensity: fuller in the middle of the band.
            let edge = ((i as f32 / band as f32) * PI).sin();
            let col_h = (edge * h as f32).round() as usize;
            let y0 = (h - col_h) / 2;
            for y in y0..y0 + col_h {
                draw::dot_i(grid, x, y as i32);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Rocket sprite — a real rocket silhouette painted at any heading.
// ---------------------------------------------------------------------------

/// What a rocket-scene dot is made of. Each cell takes the colour of the
/// material most of its dots hold, so the hull, trim and flame stay distinct
/// even at one colour per cell.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ink {
    Empty,
    Trail,
    Flame,
    Core,
    Hull,
    Trim,
}

/// A dot-space canvas of [`Ink`], flushed into a grid in one pass.
struct Canvas {
    w: usize,
    h: usize,
    ink: Vec<Ink>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            ink: vec![Ink::Empty; w * h],
        }
    }

    fn set(&mut self, x: i32, y: i32, ink: Ink) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.ink[y as usize * self.w + x as usize] = ink;
        }
    }

    /// Paint a rocket centred on `(cx, cy)` heading `ang` radians (0 = right,
    /// PI/2 = up): pointed nose cone, hull with a porthole, two swept fins and
    /// a flickering two-tone flame `flame` dots long.
    #[allow(clippy::too_many_arguments)]
    fn rocket(&mut self, cx: f32, cy: f32, ang: f32, len: f32, r: f32, flame: f32, t: f32) {
        let (ca, sa) = (ang.cos(), ang.sin());
        let half = len / 2.0;
        let nose = len * 0.3;
        let nose_at = half - nose;
        let fin = len * 0.4;
        let flare = r * 1.1 + 1.0;
        let reach = half + flame + flare + 2.0;
        let port_u = nose_at - r * 1.1;
        for y in (cy - reach).floor() as i32..=(cy + reach).ceil() as i32 {
            for x in (cx - reach).floor() as i32..=(cx + reach).ceil() as i32 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                // Rocket-local axes: u runs tail -> nose, v across the hull.
                let u = dx * ca - dy * sa;
                let v = (dx * sa + dy * ca).abs();
                let ink = if u > nose_at && u <= half {
                    let k = (u - nose_at) / nose;
                    (v <= r * (1.0 - k).powf(0.85) + 0.2).then_some(Ink::Trim)
                } else if u >= -half && u <= nose_at {
                    let porthole = r >= 2.4 && (u - port_u).powi(2) + v * v < 1.1;
                    if v <= r && !porthole {
                        Some(Ink::Hull)
                    } else if u < -half + fin && v > r && v <= r + flare * (1.0 - (u + half) / fin)
                    {
                        Some(Ink::Trim)
                    } else {
                        None
                    }
                } else if u < -half && u >= -half - 1.5 && v > r * 0.75 && v <= r + flare {
                    // Fin tips sweep back past the nozzle.
                    Some(Ink::Trim)
                } else if u < -half && u >= -half - flame {
                    let k = (-half - u) / flame.max(0.1);
                    let jag = 0.75 + 0.25 * (t * 37.0 + v * 2.3 + u * 0.9).sin();
                    let width = r * 0.8 * (1.0 - k).powf(0.6) * jag;
                    if v <= width {
                        Some(if k < 0.4 && v < width * 0.6 {
                            Ink::Core
                        } else {
                            Ink::Flame
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(ink) = ink {
                    self.set(x, y, ink);
                }
            }
        }
    }

    /// Set every inked dot, then colour each cell by its majority material.
    fn flush(&self, grid: &mut BrailleGrid, color: impl Fn(Ink, usize) -> Color) {
        grid.enable_color_support();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.ink[y * self.w + x] != Ink::Empty {
                    draw::dot(grid, x, y);
                }
            }
        }
        for cy in 0..(self.h + 3) / 4 {
            for cx in 0..(self.w + 1) / 2 {
                let mut counts = [0u8; 6];
                for y in cy * 4..(cy * 4 + 4).min(self.h) {
                    for x in cx * 2..(cx * 2 + 2).min(self.w) {
                        counts[self.ink[y * self.w + x] as usize] += 1;
                    }
                }
                // Ties go to the later (more important) material.
                let best = (1..6).rev().max_by_key(|&i| counts[i]).unwrap_or(0);
                if counts[best] > 0 {
                    let ink = [
                        Ink::Empty,
                        Ink::Trail,
                        Ink::Flame,
                        Ink::Core,
                        Ink::Hull,
                        Ink::Trim,
                    ][best];
                    let _ = grid.set_cell_color(cx, cy, color(ink, cx));
                }
            }
        }
    }
}

/// A rocket flies along the track; its exhaust trail is the fill.
struct Rocket;
impl ProgressStyle for Rocket {
    fn name(&self) -> &str {
        "rocket"
    }
    fn theme(&self) -> &str {
        "classic"
    }
    fn describe(&self) -> &str {
        "A finned rocket flies the track; its exhaust trail is the fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let r = (h as f32 * 0.19).min(w as f32 / 16.0);
        let mut cv = Canvas::new(w, h);
        let mid = h as f32 / 2.0;
        if r < 1.2 {
            // Too small for a silhouette: a plain trail with a bright nose.
            let head = (ctx.eased * w as f32) as i32;
            for x in 0..head {
                cv.set(x, mid as i32, Ink::Trail);
            }
            cv.set(head, mid as i32, Ink::Trim);
        } else {
            let len = r * 7.2;
            let flame = r * (1.9 + 0.6 * (ctx.time * 23.0).sin().abs());
            let nose = len + flame + ctx.eased * (w as f32 - len - flame - 1.0);
            let cx = nose - len / 2.0;
            let cy = mid + 0.6 * (ctx.time * 2.2).sin();
            // Unfilled track: a dotted centre line still to fly.
            for x in (nose as i32 + 2..w as i32).step_by(4) {
                cv.set(x, mid as i32, Ink::Trail);
            }
            // The trail is the fill: a solid exhaust band whose fringe puffs
            // out and thins the further it drifts behind the rocket.
            let tail = (cx - len / 2.0 - flame) as i32 + 1;
            let slot = (ctx.time * 6.0) as i32;
            for x in 0..tail.max(0) {
                let age = (tail - x) as f32 / w as f32;
                for dy in -1..=1 {
                    cv.set(x, mid as i32 + dy, Ink::Trail);
                }
                let puff = (age * r * 2.4) as i32;
                for dy in 2..=1 + puff {
                    for sign in [-1, 1] {
                        if hash3(x / 2, dy * sign, slot) > 0.35 + 0.15 * dy as f32 {
                            cv.set(x, mid as i32 + dy * sign, Ink::Trail);
                        }
                    }
                }
            }
            cv.rocket(cx, cy, 0.0, len, r, flame, ctx.time);
        }
        let cells = (w / 2).max(1) as f32;
        cv.flush(grid, |ink, cx| match ink {
            Ink::Hull => Color::rgb(232, 236, 245),
            Ink::Trim => TINT_END,
            Ink::Core => Color::rgb(255, 236, 150),
            Ink::Flame => Color::rgb(255, 140, 40),
            _ => sample_tint(cx as f32 / cells),
        });
        Ok(())
    }
}

/// A thin baseline with tick marks every 10% and a fill underline.
struct TickedTrack;
impl ProgressStyle for TickedTrack {
    fn name(&self) -> &str {
        "ticked"
    }
    fn theme(&self) -> &str {
        "classic"
    }
    fn describe(&self) -> &str {
        "Ruler track with 10% ticks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let base = h - 1;
        draw::hline(grid, 0, w - 1, base);
        for i in 0..=10 {
            let x = (i as f32 / 10.0 * (w - 1) as f32).round() as usize;
            let tick = if i % 5 == 0 { h / 2 } else { h / 4 };
            draw::vline(grid, x, base.saturating_sub(tick), base);
        }
        let filled = (ctx.eased * w as f32).round() as usize;
        draw::hline(grid, 0, filled.min(w - 1), base.saturating_sub(1));
        draw::hline(grid, 0, filled.min(w - 1), base);
        // A bobbing carriage marks the leading edge on the ruler.
        let bob = ((ctx.time * PI).sin() * 1.5 + 1.5) as usize;
        let x = filled.min(w - 1);
        draw::vline(
            grid,
            x,
            base.saturating_sub(6 - bob.min(3)),
            base.saturating_sub(2),
        );
        Ok(())
    }
}
