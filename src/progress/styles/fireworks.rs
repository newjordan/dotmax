//! Fireworks progress bars — launches, bursts, sparklers, grand finales.
//!
//! Progress reads as a show building: rockets go up one per ten percent,
//! bursts bloom wider, salvos thicken, and everything pays off at one
//! hundred percent. Colors are a night-show palette — gold, coral, cyan,
//! violet — around white-hot flashes. Deterministic in `(progress, time)`.

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

// ─── theme colors — night show ──────────────────────────────────────────────

/// Gold, the classic shell.
const FW_GOLD: Color = Color::rgb(255, 203, 94);
/// Coral red.
const FW_CORAL: Color = Color::rgb(240, 110, 90);
/// Electric cyan.
const FW_CYAN: Color = Color::rgb(92, 220, 240);
/// Violet.
const FW_VIOLET: Color = Color::rgb(176, 140, 250);
/// White-hot flash.
const FW_FLASH: Color = Color::rgb(255, 248, 231);

/// Pick a show color for particle index `i`.
fn fw_color(i: i32) -> Color {
    match (hash2(i, 999) * 4.0) as u32 {
        0 => FW_GOLD,
        1 => FW_CORAL,
        2 => FW_CYAN,
        _ => FW_VIOLET,
    }
}

/// All styles in the `fireworks` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Launch),
        Box::new(Peony),
        Box::new(Chrysanthemum),
        Box::new(Crackle),
        Box::new(RomanCandle),
        Box::new(SparklerFill),
        Box::new(Salvo),
        Box::new(Willow),
        Box::new(StrobePop),
        Box::new(GrandFinale),
    ]
}

/// Rockets launch one per ten percent, each bursting into a starburst at apex.
struct Launch;
impl ProgressStyle for Launch {
    fn name(&self) -> &str {
        "launch"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Rockets launching one per ten percent, each bursting into a star"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let count = 10usize;
        // Ground line for composition.
        for x in (0..w).step_by(2) {
            draw::dot(grid, x, h - 1);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, Color::rgb(96, 92, 104));
        }
        let apex = (h / 4).max(1);
        let span = h.saturating_sub(1 + apex) as f32;
        let slot = (ctx.time * 6.0) as i32;
        for k in 0..count {
            let x = ((k as f32 + 0.5) / count as f32 * w as f32) as usize;
            if x >= w {
                continue;
            }
            let window = (ctx.eased * count as f32) - k as f32;
            if window >= 1.0 {
                // Arrived: a small starburst at apex, its eight rays
                // twinkling in and out around a hot center.
                let color = fw_color(k as i32);
                draw::dot(grid, x, apex);
                let _ = grid.set_cell_color(x / 2, apex / 4, FW_FLASH);
                for r in 0..8 {
                    if hash3(k as i32 * 8 + r, 5, slot) < 0.2 {
                        continue;
                    }
                    let ang = r as f32 * TAU / 8.0;
                    for d in 2..=3 {
                        let px = x as f32 + ang.cos() * d as f32 * 1.2;
                        let py = apex as f32 + ang.sin() * d as f32;
                        if px >= 0.0 && py >= 0.0 {
                            draw::dot(grid, px as usize, py as usize);
                            let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, color);
                        }
                    }
                }
            } else if window > 0.0 {
                // In flight: a white rocket body over a crackling gold trail.
                let y = (h as f32 - 1.0 - window * span) as usize;
                draw::dot(grid, x, y);
                draw::dot(grid, x, y + 1);
                let _ = grid.set_cell_color(x / 2, y / 4, FW_FLASH);
                for t in 2..7usize {
                    let ty = y + t;
                    if ty < h && hash3(k as i32, t as i32, (ctx.time * 12.0) as i32) > 0.3 {
                        let jitter = (hash3(k as i32, t as i32 + 9, slot) * 3.0) as usize;
                        draw::dot(grid, (x + jitter).saturating_sub(1), ty);
                        let _ = grid.set_cell_color(x / 2, ty / 4, FW_GOLD);
                    }
                }
            }
        }
        Ok(())
    }
}

/// One great shell blooms from the center as progress grows.
struct Peony;
impl ProgressStyle for Peony {
    fn name(&self) -> &str {
        "peony"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A shell blooming wider with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let max_r = w as f32 / 2.0 - 1.0;
        let r = ctx.eased * max_r;
        let rays = 18;
        for ray in 0..rays {
            let ang = ray as f32 * TAU / rays as f32 + ctx.time * TAU * 0.25 * 0.2;
            let color = fw_color(ray);
            // Dots along the ray, denser near the tip.
            let steps = (r / 2.0) as i32;
            for s in 0..=steps {
                let dist = r * (s as f32 / steps.max(1) as f32);
                if hash3(ray, s, 0) < 0.35 && s < steps - 1 {
                    continue;
                }
                let px = cx + ang.cos() * dist;
                let py = cy + ang.sin() * dist * 0.45; // squash to the bar
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if s == steps { FW_FLASH } else { color };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        // Twinkle at the core.
        if hash3(0, 0, (ctx.time * 6.0) as i32) > 0.4 {
            draw::dot(grid, cx as usize, cy as usize);
            let _ = grid.set_cell_color(cx as usize / 2, cy as usize / 4, FW_FLASH);
        }
        Ok(())
    }
}

/// A double shell: long drooping gold trails around a short cyan pistil.
struct Chrysanthemum;
impl ProgressStyle for Chrysanthemum {
    fn name(&self) -> &str {
        "chrysanthemum"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A double shell of long gold star trails around a cyan pistil, opening with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (wf, hf) = (w as f32, h as f32);
        let cx = wf / 2.0;
        let cy = hf * 0.45;
        // The shell opens wider with progress and breathes slightly with time.
        let breathe = 0.96 + 0.04 * (ctx.time * TAU * 0.8).sin();
        let rx = (0.12 + 0.88 * ctx.eased) * (wf / 2.0 - 2.0).max(1.0) * breathe;
        let ry = (rx * 0.5).min(hf * 0.42).max(0.5);
        let droop = hf * 0.12 * ctx.eased;
        let slot = (ctx.time * 6.0) as i32;
        // Each star leaves a visible trail — that's what makes a chrysanthemum.
        // `(petals, from, to, color)`: the outer gold shell, and an inner
        // cyan pistil shell inside it, with dark sky between the two. Rays
        // are evenly spaced by eye and clipped to the shell's ellipse.
        let shells = [(18, 0.4, 1.0, FW_GOLD), (10, 0.06, 0.26, FW_CYAN)];
        for (si, &(petals, from, to, color)) in shells.iter().enumerate() {
            for p in 0..petals {
                let ang = (p as f32 + 0.5 * si as f32) * TAU / petals as f32;
                let (ca, sa) = (ang.cos(), ang.sin());
                let r = 1.0 / ((ca / rx).powi(2) + (sa / ry).powi(2)).sqrt();
                let len = to * (0.9 + 0.1 * hash2(p, 7 + si as i32));
                let steps = ((len - from) * r).max(1.0) as i32;
                for k in 0..=steps {
                    let u = from + (len - from) * k as f32 / steps as f32;
                    // Trails thin toward the burst center, fizzing with time.
                    if hash3(p * 31 + k, si as i32, slot) < 0.25 * (1.0 - u) {
                        continue;
                    }
                    let px = cx + ca * r * u;
                    let py = cy + sa * r * u + droop * u * u;
                    if px < 0.0 || py < 0.0 {
                        continue;
                    }
                    draw::dot(grid, px as usize, py as usize);
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, color);
                }
                // A white-hot star twinkling at the tip of each outer petal.
                if si == 0 && hash3(p, 13, slot) < 0.7 {
                    let px = cx + ca * r * len;
                    let py = cy + sa * r * len + droop * len * len;
                    if px >= 0.0 && py >= 0.0 {
                        draw::dot(grid, px as usize, py as usize + 1);
                        let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, FW_FLASH);
                    }
                }
            }
        }
        Ok(())
    }
}

/// A steady bar that crackles with popping sparks as it fills.
struct Crackle;
impl ProgressStyle for Crackle {
    fn name(&self) -> &str {
        "crackle"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A bar crackling with popping sparks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let mid = h / 2;
        // The base bar: a chunky four-row band.
        for x in 0..filled {
            for dy in 0..4usize {
                draw::dot(grid, x, (mid + dy).saturating_sub(2).min(h - 1));
            }
            let _ = grid.set_cell_color(x / 2, mid / 4, FW_GOLD);
        }
        // Crackle pops above and below, densest near the leading edge.
        let slot = (ctx.time * 12.0) as i32;
        for x in 0..filled {
            let near = 1.0 - (filled - x) as f32 / w.max(1) as f32 * 1.8;
            if near <= 0.0 {
                continue;
            }
            if hash3(x as i32, 1, slot) < near * 0.7 {
                let up = 1 + (hash3(x as i32, 2, slot) * (mid as f32 - 2.0)) as usize;
                let y = if hash3(x as i32, 3, slot) > 0.5 {
                    mid.saturating_sub(2 + up)
                } else {
                    (mid + 2 + up).min(h - 1)
                };
                draw::dot(grid, x, y);
                if hash3(x as i32, 4, slot) > 0.5 {
                    draw::dot(grid, (x + 1).min(w - 1), y);
                }
                let _ = grid.set_cell_color(x / 2, y / 4, FW_FLASH);
            }
        }
        Ok(())
    }
}

/// A wrapped tube at the left fires coloured stars one after another; each
/// arcs out trailing sparks and pops at the progress point.
struct RomanCandle;
impl ProgressStyle for RomanCandle {
    fn name(&self) -> &str {
        "roman-candle"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A striped tube firing coloured stars that arc out and pop at the progress point"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (wf, hf) = (w as f32, h as f32);
        // The tube: four dots wide, wrapped in alternating paper bands.
        let tube_top = h.saturating_sub((h / 2).max(1));
        for y in tube_top..h {
            draw::hline(grid, 0, 3, y);
        }
        for cy in tube_top / 4..(h + 3) / 4 {
            let band = if cy % 2 == 0 {
                FW_CORAL
            } else {
                Color::rgb(236, 226, 200)
            };
            draw::tint_row(grid, cy, 0, 1, band);
        }
        let slot = (ctx.time * 10.0) as i32;
        // Muzzle sparks flickering at the mouth.
        for k in 0..3 {
            if hash3(k, 3, slot) < 0.5 {
                let x = 1 + (hash3(k, 4, slot) * 3.0) as i32 - 1;
                draw::dot_i(grid, x + 1, tube_top as i32 - 1 - k);
            }
        }
        if tube_top >= 1 {
            let _ = grid.set_cell_color(0, (tube_top - 1) / 4, FW_GOLD);
            let _ = grid.set_cell_color(1, (tube_top - 1) / 4, FW_GOLD);
        }
        // Stars in flight: a parabola from the mouth to the progress point.
        let (x0, y0) = (2.0f32, tube_top as f32 - 1.0);
        let tx = 8.0 + ctx.eased * (wf - 12.0).max(0.0);
        let ty = hf * 0.35;
        let lift = hf * 0.35;
        let path = |u: f32| {
            (
                x0 + (tx - x0) * u,
                y0 + (ty - y0) * u - lift * 4.0 * u * (1.0 - u),
            )
        };
        for s in 0..3 {
            let clock = ctx.time * 0.8 + s as f32 / 3.0;
            let (cycle, phase) = (clock.floor() as i32, clock.fract());
            let color = fw_color(s * 5 + cycle);
            if phase < 0.75 {
                let u = phase / 0.75;
                // Sparkling trail along the arc behind the star.
                for k in 1..12 {
                    let ut = u - k as f32 * 0.025;
                    if ut < 0.0 || hash3(s * 50 + k, cycle, slot) < 0.4 {
                        continue;
                    }
                    let (px, py) = path(ut);
                    if px >= 0.0 && py >= 0.0 {
                        draw::dot(grid, px as usize, py as usize);
                        let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, FW_GOLD);
                    }
                }
                // The star itself: a solid 2x2 ball.
                let (px, py) = path(u);
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    draw::dot_i(grid, px as i32 + dx, py as i32 + dy);
                }
                if px >= 0.0 && py >= 0.0 {
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, color);
                }
            } else {
                // The pop: a small ring of rays opening at the target.
                let a = (phase - 0.75) / 0.25;
                let rad = 1.5 + a * 4.0;
                for r in 0..8 {
                    let ang = r as f32 * TAU / 8.0;
                    let px = tx + ang.cos() * rad * 1.3;
                    let py = ty + ang.sin() * rad * 0.8 + a * 1.5;
                    if px >= 0.0 && py >= 0.0 {
                        draw::dot(grid, px as usize, py as usize);
                        let c = if a < 0.2 { FW_FLASH } else { color };
                        let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                    }
                }
            }
        }
        Ok(())
    }
}

/// A sparkler point burns along a wire, spraying rays as it goes.
struct SparklerFill;
impl ProgressStyle for SparklerFill {
    fn name(&self) -> &str {
        "sparkler-fill"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A sparkler burning along a wire"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h / 2;
        let head = (ctx.eased * w as f32) as usize;
        // Burnt wire behind: a thin steady line.
        for x in 0..head {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, Color::rgb(150, 130, 110));
        }
        // Unburnt wire ahead: sparse dots.
        for x in (head..w).step_by(3) {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, Color::rgb(90, 86, 96));
        }
        // The sparkler head: dense random short rays, re-rolled each frame.
        let slot = (ctx.time * 12.0) as i32;
        for s in 0..16i32 {
            let ang = hash3(s, 1, slot) * TAU;
            let len = 1.5 + hash3(s, 2, slot) * 5.0;
            for k in 0..len as i32 {
                let px = head as f32 + ang.cos() * k as f32;
                let py = mid as f32 + ang.sin() * k as f32 * 0.7;
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if k < 2 { FW_FLASH } else { FW_GOLD };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// Small shells burst all along the filled stretch, thicker as it grows.
struct Salvo;
impl ProgressStyle for Salvo {
    fn name(&self) -> &str {
        "salvo"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Shell bursts thickening along the fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // Thin baseline keeps the reading unambiguous.
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, FW_GOLD);
        }
        // Bursts: each has a home position and its own life cycle.
        let shells = 12;
        for s in 0..shells {
            let bx = (hash2(s, 51) * w as f32) as usize;
            if bx >= filled {
                continue;
            }
            let by = 2.0 + hash2(s, 52) * (h as f32 * 0.5);
            let age = (ctx.time * 0.5 + hash2(s, 53)).fract();
            let radius = age * 6.0;
            let fade = 1.0 - age;
            let points = 8;
            for p in 0..points {
                if hash3(s, p, 7) > fade + 0.3 {
                    continue;
                }
                let ang = p as f32 * TAU / points as f32 + hash2(s, 54) * TAU;
                let px = bx as f32 + ang.cos() * radius;
                let py = by + ang.sin() * radius * 0.6 + age * age * 3.0;
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if age < 0.15 {
                        FW_FLASH
                    } else {
                        fw_color(s * 31 + p)
                    };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// One golden willow: long drooping trails that stretch with progress.
struct Willow;
impl ProgressStyle for Willow {
    fn name(&self) -> &str {
        "willow"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Golden trails drooping wider with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0;
        let cy = 1.5;
        let trails = 16;
        let reach = ctx.eased * (w as f32 / 2.0);
        for t in 0..trails {
            let spread = (t as f32 / (trails - 1).max(1) as f32) * 2.0 - 1.0; // -1..1
            let dir = spread * reach;
            let steps = (reach * 0.9) as i32;
            for s in 0..steps {
                let frac = s as f32 / steps.max(1) as f32;
                let px = cx + dir * frac;
                let py = cy + frac * frac * (h as f32 - 2.5) + (hash3(t, s, 3) - 0.5);
                // Trails shimmer out near the tips.
                if frac > 0.6 && hash3(t, s, (ctx.time * 6.0) as i32) < frac - 0.5 {
                    continue;
                }
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if frac < 0.2 { FW_FLASH } else { FW_GOLD };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// Single-frame flashes pop across the fill, leaving afterglow rings.
struct StrobePop;
impl ProgressStyle for StrobePop {
    fn name(&self) -> &str {
        "strobe-pop"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Strobe flashes with afterglow rings"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // Baseline bar, two rows.
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
            draw::dot(grid, x, h.saturating_sub(2));
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, FW_VIOLET);
        }
        // Pops: each strobe lives two slots — a bright cross, then a ring.
        let slot = (ctx.time * 6.0) as i32;
        let pops = 2 + (ctx.progress * 6.0) as i32;
        for p in 0..pops {
            let s = slot - p; // stagger pops across recent slots
            let px = (hash3(p, 1, s) * filled.max(1) as f32) as i32;
            let py = 1 + (hash3(p, 2, s) * (h as f32 - 6.0)) as i32;
            if px as usize >= filled {
                continue;
            }
            if p % 2 == 0 {
                // Flash: a bright cross.
                draw::dot_i(grid, px, py);
                draw::dot_i(grid, px - 1, py);
                draw::dot_i(grid, px + 1, py);
                draw::dot_i(grid, px - 2, py);
                draw::dot_i(grid, px + 2, py);
                draw::dot_i(grid, px, py - 1);
                draw::dot_i(grid, px, py + 1);
                if px >= 0 && py >= 0 {
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, FW_FLASH);
                }
            } else {
                // Afterglow: a denser ring.
                for r in 0..8 {
                    let ang = r as f32 * TAU / 8.0;
                    let rx = px + (ang.cos() * 3.5) as i32;
                    let ry = py + (ang.sin() * 2.2) as i32;
                    if hash3(p, r, s) > 0.25 {
                        draw::dot_i(grid, rx, ry);
                        if rx >= 0 && ry >= 0 && (rx as usize) < w && (ry as usize) < h {
                            let _ = grid.set_cell_color(
                                rx as usize / 2,
                                ry as usize / 4,
                                fw_color(p * 7 + r),
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// The whole show at once: a fuse-gold gauge along the ground while rockets
/// climb and burst overhead — more shells in the air the further along.
struct GrandFinale;
impl ProgressStyle for GrandFinale {
    fn name(&self) -> &str {
        "grand-finale"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Rockets climbing and bursting, more shells overhead as progress builds"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (wf, hf) = (w as f32, h as f32);
        // The gauge: a solid two-dot line along the ground.
        let filled = (ctx.eased * wf).round() as usize;
        if filled > 0 && h > 0 {
            for y in h.saturating_sub(2)..h {
                draw::hline(grid, 0, filled - 1, y);
            }
            draw::tint_row(grid, (h - 1) / 4, 0, (filled - 1) / 2, FW_GOLD);
        }
        // The sky above: rockets rise then burst, more of them as progress
        // climbs. Each shell runs its own cycle with a fresh spot per cycle.
        let sky = hf - 3.0;
        if sky < 3.0 {
            return Ok(());
        }
        let shells = 2 + (ctx.eased * 8.0) as i32;
        for b in 0..shells {
            let clock = ctx.time / (1.4 + 0.5 * hash2(b, 60)) + hash2(b, 61);
            let (cycle, age) = (clock.floor() as i32, clock.fract());
            let bx = (0.06 + 0.88 * hash3(b, 62, cycle)) * wf;
            let by = sky * (0.35 + 0.2 * hash3(b, 63, cycle));
            let color = fw_color(b * 7 + cycle);
            if age < 0.25 {
                // The climb: a short white streak rising toward the break.
                let k = age / 0.25;
                let y = sky + (by - sky) * k;
                for d in 0..3 {
                    draw::dot_i(grid, bx as i32, y as i32 + d);
                }
                if y >= 0.0 {
                    let _ = grid.set_cell_color(bx as usize / 2, y as usize / 4, FW_FLASH);
                }
                continue;
            }
            // The break: rays streak outward, sag, and burn out.
            let a = (age - 0.25) / 0.75;
            let radius = (5.0 + 5.0 * hash2(b, 64)) * a.sqrt() + 1.0;
            let sag = a * a * 3.0;
            let rays = 10;
            for r in 0..rays {
                if hash3(b * 17 + r, 65, cycle) < (a - 0.6).max(0.0) * 2.5 {
                    continue;
                }
                let ang = r as f32 * TAU / rays as f32 + hash2(b, 66) * TAU;
                let (ca, sa) = (ang.cos(), ang.sin() * 0.55);
                for k in 0..=(radius * 0.4) as i32 {
                    let d = radius * 0.6 + k as f32;
                    let px = bx + ca * d * 1.4;
                    let py = by + sa * d + sag * d / radius;
                    if px >= 0.0 && py >= 0.0 && py < sky + 1.0 {
                        draw::dot(grid, px as usize, py as usize);
                        let c = if a < 0.12 { FW_FLASH } else { color };
                        let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                    }
                }
            }
        }
        // The hundred-percent moment: a full-sky strobe.
        if ctx.progress > 0.97 && (ctx.time * 6.0) as i32 % 2 == 0 {
            for x in (0..w).step_by(2) {
                draw::dot(grid, x, 0);
                let _ = grid.set_cell_color(x / 2, 0, FW_FLASH);
            }
        }
        Ok(())
    }
}
