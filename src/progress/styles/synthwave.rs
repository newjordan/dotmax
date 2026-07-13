//! Synthwave progress bars — outrun sunsets, neon grids, chrome and VHS.
//!
//! Every style is a little 1985-that-never-was: a striped sun climbs as
//! progress rises, perspective grids scroll toward the viewer, neon tubes
//! flicker on, chrome gleams sweep past. Palette is hot pink / violet /
//! electric cyan around a sunset core. Deterministic in `(progress, time)`.

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

// ─── theme colors — sunset neon ─────────────────────────────────────────────

/// Hot pink, the signature neon.
const SW_PINK: Color = Color::rgb(255, 64, 168);
/// Electric cyan for horizons and highlights.
const SW_CYAN: Color = Color::rgb(64, 230, 255);
/// Deep violet for dark structure.
const SW_VIOLET: Color = Color::rgb(122, 74, 226);
/// Dusk purple for unlit track and far grid.
const SW_DUSK: Color = Color::rgb(88, 44, 128);
/// Sunset orange, the middle of the sun ramp.
const SW_ORANGE: Color = Color::rgb(255, 138, 66);
/// Sun-core yellow.
const SW_YELLOW: Color = Color::rgb(255, 216, 102);
/// White-hot sparkle.
const SW_WHITE: Color = Color::rgb(255, 244, 248);

/// Blend two colors at `t` in `0.0..=1.0`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let l = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(l(a.r, b.r), l(a.g, b.g), l(a.b, b.b))
}

/// Sun ramp: yellow at the top, orange through the middle, pink at the base.
fn sun_ramp(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        mix(SW_YELLOW, SW_ORANGE, t * 2.0)
    } else {
        mix(SW_ORANGE, SW_PINK, (t - 0.5) * 2.0)
    }
}

/// All styles in the `synthwave` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Sunrise),
        Box::new(GridRun),
        Box::new(NeonSign),
        Box::new(ChromeFade),
        Box::new(VhsTracking),
        Box::new(RetroEq),
        Box::new(LaserHorizon),
        Box::new(Starfall),
        Box::new(Outrun),
        Box::new(NeonWave),
    ]
}

/// The striped outrun sun climbs above a scrolling grid as progress rises.
struct Sunrise;
impl ProgressStyle for Sunrise {
    fn name(&self) -> &str {
        "sunrise"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Striped sun rising over a scrolling neon grid"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let horizon = h as i32 - 5;
        let cx = w as i32 / 2;
        let r = 7i32;
        // Sun center climbs from just-peeking to fully risen; a linear blend
        // keeps the first percent visible instead of hiding in the ease-in.
        let rise = 0.5 * ctx.progress + 0.5 * ctx.eased;
        let cy = horizon + r - 1 - (rise * (horizon + r - 5) as f32).round() as i32;
        let stripe_shift = (ctx.time * 4.0) as i32;
        for dy in -r..=r {
            let y = cy + dy;
            if y < 0 || y >= horizon {
                continue;
            }
            // Lower half of the disc carries the classic scan-gap stripes.
            if dy > 0 && (y + stripe_shift).rem_euclid(3) == 0 {
                continue;
            }
            let half = ((r * r - dy * dy) as f32).sqrt() as i32;
            for x in (cx - half)..=(cx + half) {
                draw::dot_i(grid, x, y);
            }
            let ramp = (dy + r) as f32 / (2 * r) as f32;
            let c0 = ((cx - half) / 2).max(0) as usize;
            let c1 = ((cx + half) / 2).max(0) as usize;
            draw::tint_row(grid, (y / 4) as usize, c0, c1, sun_ramp(ramp));
        }
        // Horizon line, lit outward from center with progress.
        let lit = (ctx.eased * cx as f32).round() as i32;
        draw::hline(grid, 0, w - 1, horizon as usize);
        draw::tint_row(grid, (horizon / 4) as usize, 0, w / 2, SW_DUSK);
        if lit > 0 {
            let c0 = ((cx - lit) / 2).max(0) as usize;
            let c1 = ((cx + lit) / 2) as usize;
            draw::tint_row(grid, (horizon / 4) as usize, c0, c1, SW_CYAN);
        }
        // Perspective floor: converging verticals plus rolling horizontals.
        let floor_cell = (horizon as usize / 4 + 1).min(ctx.height - 1);
        for k in -6..=6i32 {
            let bx = cx + k * 9;
            let steps = h as i32 - 1 - horizon;
            for s in 1..=steps {
                let x = cx + (bx - cx) * s / steps.max(1);
                draw::dot_i(grid, x, horizon + s);
            }
        }
        for i in 0..3 {
            let f = (ctx.time * 0.75 + i as f32 / 3.0).fract();
            let y = horizon + 1 + (f * f * (h as i32 - 2 - horizon) as f32) as i32;
            draw::hline(grid, 0, w - 1, y as usize);
        }
        for cy2 in floor_cell..ctx.height {
            draw::tint_row(grid, cy2, 0, ctx.width - 1, SW_VIOLET);
        }
        Ok(())
    }
}

/// A wall of light sweeps down a perspective grid toward the viewer.
struct GridRun;
impl ProgressStyle for GridRun {
    fn name(&self) -> &str {
        "gridrun"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Light wall racing down an endless neon grid"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let vp_y = 3i32;
        let cx = w as i32 / 2;
        // Twinkling stars above the vanishing point.
        let slot = (ctx.time * 2.0) as i32;
        for i in 0..10 {
            let sx = (hash2(i, 11) * w as f32) as i32;
            let sy = (hash2(i, 23) * vp_y as f32) as i32;
            if hash3(i, 0, slot) > 0.35 {
                draw::dot_i(grid, sx, sy);
            }
        }
        // The light wall: progress pushes it from the horizon to the viewer.
        let wall_y = vp_y + (ctx.eased * (h as i32 - 1 - vp_y) as f32).round() as i32;
        // Converging verticals, dotted, with a clear band above the wall so
        // the wall stays crisp against the lattice.
        for k in -8..=8i32 {
            let bx = cx + k * 10;
            let steps = h as i32 - 1 - vp_y;
            for s in (0..=steps).step_by(2) {
                let y = vp_y + s;
                if y < wall_y - 2 || y > wall_y + 2 {
                    let x = cx + (bx - cx) * s / steps.max(1);
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Rolling horizontals, accelerating as they near the viewer.
        for i in 0..4 {
            let f = (ctx.time * 0.5 + i as f32 * 0.25).fract();
            let y = vp_y + (f * f * (h as i32 - 1 - vp_y) as f32) as i32;
            if y < wall_y - 2 || y > wall_y + 2 {
                draw::hline(grid, 0, w - 1, y as usize);
            }
        }
        for dy in 0..2 {
            draw::hline(grid, 0, w - 1, (wall_y + dy).min(h as i32 - 1) as usize);
        }
        // Color: dim violet grid, pink glow below the wall, cyan wall crest.
        for cy in 0..ctx.height {
            let row_mid = cy as i32 * 4 + 2;
            let c = if row_mid < wall_y {
                SW_DUSK
            } else if row_mid < wall_y + 4 {
                SW_CYAN
            } else {
                SW_PINK
            };
            draw::tint_row(grid, cy, 0, ctx.width - 1, c);
        }
        Ok(())
    }
}

/// A neon tube border lights up clockwise; the readout buzzes in the middle.
struct NeonSign;
impl ProgressStyle for NeonSign {
    fn name(&self) -> &str {
        "neon-sign"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Neon tube border flickering on, percent in lights"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (x0, y0, x1, y1) = (1i32, 1i32, w as i32 - 2, h as i32 - 2);
        // Walk the tube perimeter clockwise from the top-left corner.
        let top = x1 - x0;
        let right = y1 - y0;
        let perim = 2 * (top + right);
        let lit = (ctx.eased * perim as f32).round() as i32;
        let slot = (ctx.time * 4.0) as i32;
        for s in 0..perim {
            let (x, y) = if s < top {
                (x0 + s, y0)
            } else if s < top + right {
                (x1, y0 + (s - top))
            } else if s < 2 * top + right {
                (x1 - (s - top - right), y1)
            } else {
                (x0, y1 - (s - 2 * top - right))
            };
            let on = s < lit;
            // Fresh tube segments flicker as they warm up; old ones buzz rarely.
            let seg = s / 6;
            let fresh = on && lit - s < perim / 8;
            let buzz = hash3(seg, 3, slot);
            let bright = on && !(fresh && buzz < 0.4) && buzz >= 0.05;
            // Unlit tube is sparse glass; lit tube is a solid run of light,
            // so the fill reads even without color — and flickering segments
            // actually drop out of the tube, not just dim.
            if !on && s % 3 != 0 {
                continue;
            }
            if on && !bright && s % 2 != 0 {
                continue;
            }
            draw::dot_i(grid, x, y);
            let cell = ((x / 2) as usize, (y / 4) as usize);
            let _ = grid.set_cell_color(
                cell.0,
                cell.1,
                if bright {
                    SW_PINK
                } else if on {
                    SW_VIOLET
                } else {
                    SW_DUSK
                },
            );
        }
        // Percent readout in cyan lights, center stage.
        if let Some(label) = &ctx.label {
            let chars: Vec<char> = label.chars().collect();
            let cw = ctx.width;
            let cx0 = cw.saturating_sub(chars.len()) / 2;
            let cy = ctx.height / 2;
            for (i, c) in chars.iter().enumerate() {
                draw::glyph(grid, cx0 + i, cy, *c);
                let flick = hash3(i as i32, 9, slot) > 0.08;
                let _ = grid.set_cell_color(cx0 + i, cy, if flick { SW_CYAN } else { SW_DUSK });
            }
        }
        Ok(())
    }
}

/// A chrome bar with mirror banding and a gleam that sweeps past.
struct ChromeFade;
impl ProgressStyle for ChromeFade {
    fn name(&self) -> &str {
        "chrome-fade"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Mirror-chrome fill with a sweeping gleam"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let top = (h / 4).max(1);
        let bot = h.saturating_sub(h / 4 + 1).max(top);
        let filled = (ctx.eased * w as f32).round() as usize;
        // Track rails.
        draw::hline(grid, 0, w - 1, top.saturating_sub(2));
        draw::hline(grid, 0, w - 1, (bot + 2).min(h - 1));
        for cy in 0..ctx.height {
            draw::tint_row(grid, cy, 0, ctx.width - 1, SW_DUSK);
        }
        // Chrome body.
        for x in 0..filled {
            for y in top..=bot {
                draw::dot(grid, x, y);
            }
        }
        // Vertical leading edge.
        if filled > 0 && filled < w {
            draw::vline(grid, filled, top.saturating_sub(1), bot + 1);
        }
        // Banding: sky above the seam, sunset metal below. The seam bows
        // gently with time like a horizon reflected in curved chrome.
        let gleam = ((ctx.time * 0.25).fract() * (w as f32 + 24.0)) as i32 - 12;
        for cy in (top / 4)..=(bot / 4) {
            let row_mid = cy as f32 * 4.0 + 2.0;
            let seam = (h as f32 / 2.0) + (TAU * 0.25 * ctx.time).sin() * 1.5;
            let c = if row_mid < seam - 2.0 {
                mix(SW_CYAN, SW_WHITE, 0.55)
            } else if row_mid < seam {
                SW_WHITE
            } else if row_mid < seam + 3.0 {
                SW_ORANGE
            } else {
                SW_PINK
            };
            let hi_cell = (filled / 2).min(ctx.width.saturating_sub(1));
            draw::tint_row(grid, cy, 0, hi_cell, c);
            // Gleam: a diagonal white flash sliding along the filled chrome.
            for gx in 0..3i32 {
                let x = gleam + gx - (cy as i32 * 2);
                if x >= 0 && (x as usize) < filled {
                    let _ = grid.set_cell_color((x / 2) as usize, cy, SW_WHITE);
                }
            }
        }
        // Sparkle at the leading edge.
        if filled > 1 && filled < w - 1 {
            let sx = filled as i32;
            let sy = (top + bot) as i32 / 2;
            let pulse = ((ctx.time * TAU * 0.5).sin() * 2.0) as i32 + 2;
            draw::hline(
                grid,
                (sx - pulse).max(0) as usize,
                (sx + pulse) as usize,
                sy as usize,
            );
            draw::vline(
                grid,
                sx as usize,
                (sy - pulse).max(0) as usize,
                (sy + pulse) as usize,
            );
            let _ = grid.set_cell_color((sx / 2) as usize, (sy / 4) as usize, SW_WHITE);
        }
        Ok(())
    }
}

/// Tape fill with tracking jitter that settles as the signal locks in.
struct VhsTracking;
impl ProgressStyle for VhsTracking {
    fn name(&self) -> &str {
        "vhs-tracking"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "VHS fill stabilizing as tracking locks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32) as i32;
        let slot = (ctx.time * 6.0) as i32;
        let unstable = 1.0 - ctx.progress;
        // The picture: full-height fill, rows shoved sideways in bands of two
        // dots. Jitter is worst near the leading edge and calms with progress.
        for y in 0..h as i32 {
            let band = y / 2;
            let j = ((hash3(band, 1, slot) - 0.5) * 7.0 * unstable) as i32;
            let x1 = (filled + j).clamp(0, w as i32);
            for x in 0..x1 {
                draw::dot_i(grid, x, y);
            }
        }
        // A head-switch noise band rolls up the frame.
        let band_y = (((1.0 - (ctx.time * 0.5).fract()) * h as f32) as i32).min(h as i32 - 2);
        for y in band_y..(band_y + 2).min(h as i32) {
            for x in 0..w as i32 {
                if hash3(x, y, slot) > 0.45 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Washed-tape tint with chroma-fringe rows near the noise band.
        for cy in 0..ctx.height {
            let row_mid = cy as i32 * 4 + 2;
            let c = if (row_mid - band_y).abs() < 3 {
                SW_WHITE
            } else if (row_mid - band_y).abs() < 6 {
                if cy % 2 == 0 {
                    SW_PINK
                } else {
                    SW_CYAN
                }
            } else {
                mix(SW_CYAN, SW_WHITE, 0.35)
            };
            draw::tint_row(grid, cy, 0, ctx.width - 1, c);
        }
        Ok(())
    }
}

/// Bouncing equalizer columns wake up left to right with progress.
struct RetroEq;
impl ProgressStyle for RetroEq {
    fn name(&self) -> &str {
        "retro-eq"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Neon equalizer columns waking up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let bar_w = 4usize;
        let gap = 2usize;
        let n = w / (bar_w + gap);
        let margin = (w - n * (bar_w + gap) + gap) / 2;
        let active = (ctx.eased * n as f32).round() as usize;
        for i in 0..n {
            let x0 = i * (bar_w + gap) + margin;
            let t = i as f32 / n.max(1) as f32;
            let height = if i < active {
                // Rates quantized to quarter-hertz so the 4s loop is seamless.
                let rate = 0.5 + 0.25 * (hash2(i as i32, 5) * 4.0).floor();
                let bounce = (TAU * (ctx.time * rate + hash2(i as i32, 9))).sin().abs();
                (2.0 + bounce * (h as f32 - 3.0)) as usize
            } else {
                1
            };
            for y in (h - height.min(h))..h {
                for x in x0..(x0 + bar_w).min(w) {
                    draw::dot(grid, x, y);
                }
            }
            // Column color ramps pink→cyan; the cap cell burns white.
            let color = mix(SW_PINK, SW_CYAN, t);
            let top_cell = (h - height.min(h)) / 4;
            for cy in top_cell..ctx.height {
                let c0 = x0 / 2;
                let c1 = (x0 + bar_w - 1) / 2;
                let c = if cy == top_cell && i < active {
                    SW_WHITE
                } else {
                    color
                };
                draw::tint_row(grid, cy, c0, c1, c);
            }
        }
        Ok(())
    }
}

/// A fan of sky lasers switches on beam by beam over a dark grid.
struct LaserHorizon;
impl ProgressStyle for LaserHorizon {
    fn name(&self) -> &str {
        "laser-horizon"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Sky lasers fanning on over the horizon"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let horizon = h as i32 - 4;
        let cx = w as i32 / 2;
        // Dark floor grid.
        for k in -5..=5i32 {
            let bx = cx + k * 11;
            let steps = h as i32 - 1 - horizon;
            for s in 1..=steps {
                draw::dot_i(grid, cx + (bx - cx) * s / steps.max(1), horizon + s);
            }
        }
        draw::hline(grid, 0, w - 1, horizon as usize);
        // Beams fan out over 180°, switching on with progress; the whole fan
        // breathes side to side with time.
        let beams = 9;
        let lit = (ctx.eased * beams as f32).round() as i32;
        let sway = (TAU * 0.25 * ctx.time).sin() * 0.18;
        let mut pulse_cells: Vec<(usize, usize)> = Vec::new();
        for b in 0..beams {
            if b >= lit {
                continue;
            }
            let ang = std::f32::consts::PI * (0.08 + 0.84 * b as f32 / (beams - 1) as f32) + sway;
            let (dx, dy) = (ang.cos(), -ang.sin());
            let mut i = 0f32;
            loop {
                let x = cx as f32 + dx * i;
                let y = horizon as f32 + dy * i;
                if x < 0.0 || x >= w as f32 || y < 0.0 {
                    break;
                }
                // Solid beam with a bright pulse racing outward.
                draw::dot_i(grid, x as i32, y as i32);
                if ((ctx.time * 1.0 + b as f32 * 0.25).fract() * 60.0 - i).abs() < 3.0 {
                    pulse_cells.push((x as usize / 2, y as usize / 4));
                }
                i += 1.0;
            }
        }
        // Tint: pink beams in the sky, cyan horizon, violet floor.
        for cy in 0..ctx.height {
            let row_mid = cy as i32 * 4 + 2;
            if (row_mid - horizon).abs() <= 2 {
                draw::tint_row(grid, cy, 0, ctx.width - 1, SW_CYAN);
            } else if row_mid > horizon {
                draw::tint_row(grid, cy, 0, ctx.width - 1, SW_VIOLET);
            }
        }
        // Sky rows: paint only cells the beams touched, then re-flash pulses.
        let sky_cells = (horizon as usize) / 4;
        for cy in 0..sky_cells {
            for cxl in 0..ctx.width {
                let ch = grid.get_char(cxl, cy);
                if ch != '\u{2800}' && ch != ' ' {
                    let _ = grid.set_cell_color(cxl, cy, SW_PINK);
                }
            }
        }
        for (px, py) in pulse_cells {
            let _ = grid.set_cell_color(px, py, SW_WHITE);
        }
        Ok(())
    }
}

/// Meteors streak across a twinkling sky while the ground bar fills.
struct Starfall;
impl ProgressStyle for Starfall {
    fn name(&self) -> &str {
        "starfall"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Shooting stars over a filling neon skyline"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let ground = h - 3;
        let slot = (ctx.time * 2.0) as i32;
        // Star field.
        for i in 0..26 {
            let sx = (hash2(i, 31) * w as f32) as i32;
            let sy = (hash2(i, 47) * (ground as f32 - 2.0)) as i32;
            if hash3(i, 1, slot) > 0.3 {
                draw::dot_i(grid, sx, sy);
            }
        }
        // Shooting stars: heads on quarter-hertz cycles with six-dot tails.
        let mut head_cells: Vec<(usize, usize)> = Vec::new();
        for m in 0..3i32 {
            let f = (ctx.time * 0.25 + m as f32 / 3.0).fract();
            let sx = hash2(m, 77) * w as f32 * 0.7;
            let head_x = sx + f * w as f32 * 0.9;
            let head_y = f * (ground as f32 - 2.0);
            for k in 0..6i32 {
                let x = head_x as i32 - k * 2;
                let y = head_y as i32 - k;
                draw::dot_i(grid, x, y);
                if k == 0 && x >= 0 && y >= 0 {
                    head_cells.push((x as usize / 2, y as usize / 4));
                }
            }
        }
        // Sky tint: violet field, then white-hot meteor heads on top.
        for cy in 0..(ground / 4) {
            for cxl in 0..ctx.width {
                let ch = grid.get_char(cxl, cy);
                if ch != '\u{2800}' && ch != ' ' {
                    let _ = grid.set_cell_color(cxl, cy, SW_VIOLET);
                }
            }
        }
        for (hx, hy) in head_cells {
            let _ = grid.set_cell_color(hx, hy, SW_WHITE);
        }
        // Ground bar: solid pink fill over a sparse dotted track, so the
        // read survives in monochrome too.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, ground + 1);
        }
        for y in ground..h {
            for x in 0..filled {
                draw::dot(grid, x, y);
            }
        }
        let gcell = ground / 4;
        draw::tint_row(grid, gcell, 0, ctx.width - 1, SW_DUSK);
        if filled > 0 {
            draw::tint_row(grid, gcell, 0, filled / 2, SW_PINK);
            let _ = grid.set_cell_color((filled / 2).min(ctx.width - 1), gcell, SW_CYAN);
        }
        Ok(())
    }
}

/// A little coupe drives the bar toward the sun, dashes streaming past.
struct Outrun;
impl ProgressStyle for Outrun {
    fn name(&self) -> &str {
        "outrun"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Coupe cruising the bar toward the sun"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let road = h as i32 - 4;
        // Destination sun, low on the right.
        let (sun_x, sun_r) = (w as i32 - 7, 5i32);
        let stripe_shift = (ctx.time * 3.0) as i32;
        for dy in -sun_r..=0 {
            let y = road - 2 + dy;
            if y < 0 || (dy > -2 && (y + stripe_shift).rem_euclid(2) == 0) {
                continue;
            }
            let half = ((sun_r * sun_r - dy * dy) as f32).sqrt() as i32;
            for x in (sun_x - half)..=(sun_x + half) {
                draw::dot_i(grid, x, y);
            }
            draw::tint_row(
                grid,
                (y / 4) as usize,
                ((sun_x - half) / 2).max(0) as usize,
                ((sun_x + half) / 2) as usize,
                sun_ramp((dy + sun_r) as f32 / sun_r as f32),
            );
        }
        // Road with center dashes streaming left as the car "moves".
        draw::hline(grid, 0, w - 1, road as usize);
        let dash_off = ((ctx.time * 8.0) as i32) % 8;
        let mut x = -dash_off;
        while x < w as i32 {
            for k in 0..4 {
                draw::dot_i(grid, x + k, road + 2);
            }
            x += 8;
        }
        // The coupe: an 8×3 wedge with a spoiler, riding at eased.
        let car_x = (ctx.eased * (w as f32 - 12.0)) as i32 + 1;
        let car_y = road - 3;
        for k in 2..8 {
            draw::dot_i(grid, car_x + k, car_y);
        }
        for k in 0..9 {
            draw::dot_i(grid, car_x + k, car_y + 1);
            draw::dot_i(grid, car_x + k, car_y + 2);
        }
        // Wheel bumps.
        draw::dot_i(grid, car_x + 2, car_y + 3);
        draw::dot_i(grid, car_x + 6, car_y + 3);
        // Exhaust puffs trailing off behind.
        let slot = (ctx.time * 4.0) as i32;
        for p in 1..5i32 {
            let px = car_x - p * 3 - ((ctx.time * 8.0) as i32 % 3);
            if px >= 0 && hash3(p, 2, slot) > 0.35 {
                draw::dot_i(grid, px, car_y + 1 + (hash3(p, 5, slot) * 2.0) as i32);
            }
        }
        // Tints: pink car, cyan road, violet dashes.
        for cxl in 0..ctx.width {
            draw::tint_row(grid, (road / 4) as usize, cxl, cxl, SW_CYAN);
        }
        let car_cell_y = (car_y / 4).max(0) as usize;
        for cxl in (car_x / 2).max(0)..=((car_x + 9) / 2).min(ctx.width as i32 - 1) {
            let _ = grid.set_cell_color(cxl as usize, car_cell_y, SW_PINK);
        }
        Ok(())
    }
}

/// Twin neon sine ribbons snake across the bar as far as progress allows.
struct NeonWave;
impl ProgressStyle for NeonWave {
    fn name(&self) -> &str {
        "neon-wave"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Pink and cyan sine ribbons racing to the edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h as f32 / 2.0;
        let filled = (ctx.eased * w as f32).round() as usize;
        // Dim track line so the remaining path stays visible.
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, mid as usize);
        }
        for cy in 0..ctx.height {
            draw::tint_row(grid, cy, 0, ctx.width - 1, SW_DUSK);
        }
        // Two ribbons, opposite phase velocities, meeting glow in white.
        for x in 0..filled {
            let xf = x as f32;
            let y1 = mid + (xf * 0.12 + TAU * 0.5 * ctx.time).sin() * (mid - 2.0);
            let y2 = mid + (xf * 0.155 - TAU * 0.5 * ctx.time + 1.2).sin() * (mid - 2.0);
            for (y, c) in [(y1, SW_PINK), (y2, SW_CYAN)] {
                let yi = y as i32;
                draw::dot_i(grid, x as i32, yi);
                draw::dot_i(grid, x as i32, yi + 1);
                let cell = (x / 2, (yi.max(0) as usize) / 4);
                let close = (y1 - y2).abs() < 2.5;
                let _ = grid.set_cell_color(
                    cell.0,
                    cell.1.min(ctx.height - 1),
                    if close { SW_WHITE } else { c },
                );
            }
        }
        // Leading spark.
        if filled > 0 && filled < w {
            let xf = filled as f32;
            let y = mid + (xf * 0.12 + TAU * 0.5 * ctx.time).sin() * (mid - 2.0);
            for d in 0..3i32 {
                draw::dot_i(grid, filled as i32 + d, y as i32);
            }
            let _ = grid.set_cell_color(
                (filled / 2).min(ctx.width - 1),
                ((y as usize) / 4).min(ctx.height - 1),
                SW_WHITE,
            );
        }
        Ok(())
    }
}
