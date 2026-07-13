//! Demoscene progress bars — copper bars, plasma, rotozoom, sine scrollers.
//!
//! A love letter to the Amiga and C64 crack-intro canon: every style is a
//! classic real-time effect reimagined as a loading indicator. Rainbow
//! palettes cycle, rasters sweep, checkerboards spin. All of it is a pure
//! function of `(progress, time)` and loops seamlessly every four seconds.

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

// ─── palette ────────────────────────────────────────────────────────────────

/// Cycle the classic full-saturation rainbow; `t` wraps at 1.0.
fn spectrum(t: f32) -> Color {
    let t = t.rem_euclid(1.0);
    let chan = |off: f32| {
        let v = 0.5 + 0.5 * (TAU * (t + off)).cos();
        (60.0 + 195.0 * v) as u8
    };
    Color::rgb(chan(0.0), chan(2.0 / 3.0), chan(1.0 / 3.0))
}

/// Blend two colors at `t` in `0.0..=1.0`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let l = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(l(a.r, b.r), l(a.g, b.g), l(a.b, b.b))
}

/// Dim track gray-blue, for unfilled remainders.
const DS_TRACK: Color = Color::rgb(70, 70, 96);
/// White for crisp readout edges.
const DS_WHITE: Color = Color::rgb(244, 244, 252);

/// All styles in the `demoscene` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(CopperBars),
        Box::new(Plasma),
        Box::new(Rotozoom),
        Box::new(SineScroller),
        Box::new(RasterBars),
        Box::new(Twister),
        Box::new(Metaballs),
        Box::new(Tunnel),
        Box::new(Moire),
        Box::new(BobParade),
    ]
}

/// Bouncing copper gradient bars — one more joins for every sixth of progress.
struct CopperBars;
impl ProgressStyle for CopperBars {
    fn name(&self) -> &str {
        "copper-bars"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Amiga copper bars joining the bounce one by one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        // Only four two-row bars fit a sixteen-dot canvas with air between
        // them — more and the bounce reads as a solid wall.
        let total = 4;
        let visible = 1 + (ctx.eased * (total as f32 - 0.01)) as i32;
        for i in 0..visible {
            // Quarter-hertz phases keep the 4s loop seamless. Bars bounce in
            // the band above the underline so the progress read stays clean.
            let phase = TAU * (ctx.time * 0.5 + i as f32 / 4.0);
            let y0 = (0.5 + 0.5 * phase.sin()) * (h as f32 - 6.0);
            let hue = i as f32 / total as f32;
            for dy in 0..2i32 {
                let y = y0 as i32 + dy;
                for x in 0..w as i32 {
                    draw::dot_i(grid, x, y);
                }
                // Bright top scanline gives the metallic sheen.
                let c = if dy == 0 {
                    mix(spectrum(hue), DS_WHITE, 0.55)
                } else {
                    spectrum(hue)
                };
                if y >= 0 && (y as usize) < h {
                    draw::tint_row(grid, y as usize / 4, 0, ctx.width - 1, c);
                }
            }
        }
        // Crisp progress underline so the read is exact: dotted track,
        // solid two-row fill.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 2);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 2);
            draw::dot(grid, x, h - 1);
        }
        draw::tint_row(grid, ctx.height - 1, 0, ctx.width - 1, DS_WHITE);
        Ok(())
    }
}

/// Old-school plasma revealed left to right, palette forever cycling.
struct Plasma;
impl ProgressStyle for Plasma {
    fn name(&self) -> &str {
        "plasma"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Palette-cycling plasma pouring in from the left"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let t = ctx.time;
        let field = |x: f32, y: f32| {
            (x * 0.24 + TAU * 0.25 * t).sin()
                + (y * 0.55 - TAU * 0.5 * t).sin()
                + ((x * 0.14 + y * 0.31) + TAU * 0.25 * t).sin()
        };
        for y in 0..h {
            for x in 0..filled {
                let v = field(x as f32, y as f32);
                if v > -0.9 {
                    draw::dot(grid, x, y);
                }
            }
        }
        // Sparse dim dots mark the unfilled remainder.
        for y in (0..h).step_by(4) {
            for x in (filled..w).step_by(4) {
                draw::dot(grid, x, y);
            }
        }
        // Color cells by the field at their center, palette slowly cycling.
        for cy in 0..ctx.height {
            for cx in 0..ctx.width {
                if cx * 2 < filled {
                    let v = field(cx as f32 * 2.0 + 1.0, cy as f32 * 4.0 + 2.0);
                    let c = spectrum(v / 6.0 + 0.5 + t * 0.25);
                    let _ = grid.set_cell_color(cx, cy, c);
                } else {
                    let _ = grid.set_cell_color(cx, cy, DS_TRACK);
                }
            }
        }
        Ok(())
    }
}

/// A spinning, breathing checkerboard opens out from the center like an iris.
struct Rotozoom;
impl ProgressStyle for Rotozoom {
    fn name(&self) -> &str {
        "rotozoom"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Rotozooming checkerboard revealed by a growing iris"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let max_r = (cx * cx + cy * cy).sqrt();
        // A floor keeps the iris visible from the very first percent.
        let radius = (0.08 + 0.92 * ctx.eased) * max_r;
        let ang = TAU * 0.25 * ctx.time;
        let zoom = 1.3 + 0.5 * (TAU * 0.25 * ctx.time).sin();
        let (sa, ca) = ang.sin_cos();
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx;
                let dy = (y as f32 - cy) * 2.0; // braille dots are taller than wide
                let r = (dx * dx + dy * dy).sqrt();
                if r > radius {
                    continue;
                }
                let u = (dx * ca - dy * sa) * zoom;
                let v = (dx * sa + dy * ca) * zoom;
                let tile = ((u / 7.0).floor() as i32 + (v / 7.0).floor() as i32).rem_euclid(2);
                if tile == 0 {
                    draw::dot(grid, x, y);
                }
                let hue = hash2((u / 7.0).floor() as i32, (v / 7.0).floor() as i32);
                let _ = grid.set_cell_color(x / 2, y / 4, spectrum(hue * 0.4 + ctx.time * 0.25));
            }
        }
        // Bright iris rim makes the progress edge crisp.
        if radius > 2.0 && ctx.eased < 1.0 {
            let steps = (radius * 4.0) as i32;
            for s in 0..steps {
                let a = TAU * s as f32 / steps.max(1) as f32;
                let x = cx + a.cos() * radius;
                let y = cy + a.sin() * radius * 0.5;
                draw::dot_i(grid, x as i32, y as i32);
                let _ = grid.set_cell_color(
                    (x as usize / 2).min(ctx.width - 1),
                    (y.max(0.0) as usize / 4).min(ctx.height - 1),
                    DS_WHITE,
                );
            }
        }
        Ok(())
    }
}

/// The classic rainbow sine scroller, riding above a crisp baseline bar.
struct SineScroller;
impl ProgressStyle for SineScroller {
    fn name(&self) -> &str {
        "sine-scroller"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Rainbow marquee text on a sine wave"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        grid.enable_color_support();
        let cw = ctx.width;
        // Build a marquee whose length divides the scroll distance so the
        // 4s loop wraps without a seam (24 cells at 6 cells/sec).
        let label = ctx.label.clone().unwrap_or_default();
        let mut text: Vec<char> = format!(" LOADING {label} · DOTMAX ·").chars().collect();
        text.truncate(24);
        while text.len() < 24 {
            text.push(' ');
        }
        let scroll = (ctx.time * 6.0) as i32;
        for sx in 0..cw as i32 {
            let idx = (sx + scroll).rem_euclid(24) as usize;
            let c = text[idx];
            if c == ' ' {
                continue;
            }
            let wave = (TAU * (sx as f32 * 0.028 + ctx.time * 0.5)).sin();
            let cy = ((1.0 + wave) * 0.5 * (ctx.height as f32 - 2.05)) as usize;
            draw::glyph(grid, sx as usize, cy, c);
            let hue = sx as f32 / cw as f32 + ctx.time * 0.5;
            let _ = grid.set_cell_color(sx as usize, cy, spectrum(hue));
        }
        // Baseline progress bar in dots along the bottom.
        let (w, h) = draw::dot_dims(grid);
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 2);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 2);
            draw::dot(grid, x, h - 1);
        }
        for cx in 0..cw {
            if grid.get_char(cx, ctx.height - 1) != '\u{2800}' {
                let c = if cx * 2 < filled {
                    spectrum(cx as f32 / cw as f32 + ctx.time * 0.5)
                } else {
                    DS_TRACK
                };
                let _ = grid.set_cell_color(cx, ctx.height - 1, c);
            }
        }
        Ok(())
    }
}

/// Rainbow rasters sweep behind a crisp framed progress bar.
struct RasterBars;
impl ProgressStyle for RasterBars {
    fn name(&self) -> &str {
        "raster-bars"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Raster interrupt rainbows behind a framed bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let top = (h / 2).saturating_sub(3);
        let bot = (h / 2 + 3).min(h - 1);
        // Framed bar first — it owns its cells.
        draw::rect_outline(grid, 0, top, w, bot - top + 1);
        let filled = ((ctx.eased * (w as f32 - 4.0)).max(0.0) as usize).min(w.saturating_sub(4));
        for y in (top + 2)..=bot.saturating_sub(2) {
            for x in 2..(2 + filled) {
                draw::dot(grid, x, y);
            }
        }
        let bar_cell_top = top / 4;
        let bar_cell_bot = bot / 4;
        // Raster bars sweep the rows above and below, painting only cells the
        // frame doesn't own; inside the frame they tint the fill hue instead.
        for i in 0..3i32 {
            let phase = TAU * (ctx.time * 0.25 + i as f32 / 3.0);
            let y0 = ((0.5 + 0.5 * phase.sin()) * (h as f32 - 3.0)) as usize;
            let hue = i as f32 / 3.0 + ctx.time * 0.25;
            for dy in 0..3usize {
                let y = (y0 + dy).min(h - 1);
                let cy = y / 4;
                if cy >= bar_cell_top && cy <= bar_cell_bot {
                    continue;
                }
                // Half-tone texture keeps rasters visually behind the solid bar.
                for x in ((y % 2)..w).step_by(2) {
                    draw::dot(grid, x, y);
                }
                let c = if dy == 1 {
                    mix(spectrum(hue), DS_WHITE, 0.5)
                } else {
                    spectrum(hue)
                };
                draw::tint_row(grid, cy, 0, ctx.width - 1, c);
            }
        }
        // Bar tint: white frame rows, cycling fill.
        for cy in bar_cell_top..=bar_cell_bot {
            draw::tint_row(grid, cy, 0, ctx.width - 1, DS_WHITE);
        }
        if filled > 0 {
            let mid_cell = (top + 2) / 4;
            for cx in 1..=((2 + filled) / 2).min(ctx.width.saturating_sub(2)) {
                let _ =
                    grid.set_cell_color(cx, mid_cell, spectrum(cx as f32 * 0.02 + ctx.time * 0.5));
            }
        }
        Ok(())
    }
}

/// A ribbon that twists around its own axis as it grows across the bar.
struct Twister;
impl ProgressStyle for Twister {
    fn name(&self) -> &str {
        "twister"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Twisting ribbon column growing with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h as f32 / 2.0;
        let amp = mid - 1.5;
        let filled = (ctx.eased * w as f32).round() as usize;
        // Dim center track for the remainder.
        for x in (filled..w).step_by(3) {
            draw::dot(grid, x, mid as usize);
        }
        for cy in 0..ctx.height {
            draw::tint_row(grid, cy, 0, ctx.width - 1, DS_TRACK);
        }
        for x in 0..filled {
            let twist = x as f32 * 0.085 + TAU * 0.5 * ctx.time;
            // Four ribbon edges; consecutive pairs that face us get filled.
            let mut edges = [0f32; 4];
            for (k, e) in edges.iter_mut().enumerate() {
                *e = mid + amp * (twist + k as f32 * TAU / 4.0).sin();
            }
            for k in 0..4 {
                let (a, b) = (edges[k], edges[(k + 1) % 4]);
                if a < b {
                    // Front face: fill between edges, hue by face index.
                    for y in (a as i32)..=(b as i32) {
                        draw::dot_i(grid, x as i32, y);
                    }
                    let hue = k as f32 / 4.0 + 0.1;
                    let shade = ((b - a) / (2.0 * amp)).clamp(0.15, 1.0);
                    let cell_y = (((a + b) / 2.0) as usize / 4).min(ctx.height - 1);
                    let _ = grid.set_cell_color(
                        x / 2,
                        cell_y,
                        mix(Color::rgb(20, 20, 30), spectrum(hue), 0.35 + 0.65 * shade),
                    );
                }
            }
            // Crisp rims top and bottom of the silhouette.
            let lo = edges.iter().fold(f32::MAX, |m, &v| m.min(v));
            let hi = edges.iter().fold(f32::MIN, |m, &v| m.max(v));
            draw::dot_i(grid, x as i32, lo as i32);
            draw::dot_i(grid, x as i32, hi as i32);
        }
        Ok(())
    }
}

/// Blobs that merge and split, roaming exactly as far as progress allows.
struct Metaballs;
impl ProgressStyle for Metaballs {
    fn name(&self) -> &str {
        "metaballs"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Metaballs roaming the filled region"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let reach = (ctx.eased * w as f32).max(10.0);
        let t = ctx.time;
        // Three blobs on closed Lissajous orbits inside [0, reach].
        let balls = [
            (
                reach * (0.5 + 0.38 * (TAU * (t * 0.25)).sin()),
                h as f32 * (0.45 + 0.22 * (TAU * (t * 0.5 + 0.25)).sin()),
                5.6f32,
            ),
            (
                reach * (0.5 + 0.4 * (TAU * (t * 0.5 + 0.6)).sin()),
                h as f32 * (0.45 + 0.24 * (TAU * (t * 0.25 + 0.5)).cos()),
                4.6f32,
            ),
            (
                reach * (0.5 + 0.3 * (TAU * (t * 0.75 + 0.35)).cos()),
                h as f32 * (0.45 + 0.2 * (TAU * (t * 0.5)).sin()),
                3.8f32,
            ),
        ];
        let field = |x: f32, y: f32| {
            balls
                .iter()
                .map(|&(bx, by, r)| {
                    let dx = x - bx;
                    let dy = (y - by) * 1.8;
                    r * r / (dx * dx + dy * dy + 0.6)
                })
                .sum::<f32>()
        };
        for y in 0..h {
            for x in 0..(reach as usize).min(w) {
                if field(x as f32, y as f32) > 0.85 {
                    draw::dot(grid, x, y);
                }
            }
        }
        // Slime tint: hot core → green rim by field strength.
        for cy in 0..ctx.height {
            for cx in 0..ctx.width {
                let v = field(cx as f32 * 2.0 + 1.0, cy as f32 * 4.0 + 2.0);
                if v > 0.85 {
                    let hot = ((v - 0.85) / 2.0).clamp(0.0, 1.0);
                    let _ = grid.set_cell_color(
                        cx,
                        cy,
                        mix(Color::rgb(60, 220, 130), Color::rgb(235, 255, 200), hot),
                    );
                } else {
                    let _ = grid.set_cell_color(cx, cy, DS_TRACK);
                }
            }
        }
        // Baseline read.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 1);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
        }
        Ok(())
    }
}

/// A checkered tunnel revealed by a clock sweep, forever rushing inward.
struct Tunnel;
impl ProgressStyle for Tunnel {
    fn name(&self) -> &str {
        "tunnel"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Rushing tunnel revealed by a radial sweep"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0 + 5.0 * (TAU * 0.25 * ctx.time).sin();
        let cy = h as f32 / 2.0 + 2.0 * (TAU * 0.25 * ctx.time).cos();
        let sweep = ctx.eased * TAU;
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx;
                let dy = (y as f32 - cy) * 2.0;
                let ang = dy.atan2(dx).rem_euclid(TAU);
                if ang > sweep {
                    continue;
                }
                let r = (dx * dx + dy * dy).sqrt().max(1.5);
                let u = 26.0 / r + ctx.time * 1.0;
                // Pure rushing rings — angle drives only the color, which
                // keeps the monochrome read clean.
                if (u.floor() as i32).rem_euclid(2) == 0 {
                    draw::dot(grid, x, y);
                }
                // Deeper (smaller r) is darker; hue rides the ring index.
                let depth = (r / (w as f32 * 0.5)).clamp(0.0, 1.0);
                let c = mix(
                    Color::rgb(18, 18, 30),
                    spectrum(u.floor() * 0.11 + ctx.time * 0.25),
                    0.25 + 0.75 * depth,
                );
                let _ = grid.set_cell_color(x / 2, y / 4, c);
            }
        }
        // Bright sweep edge so the progress read stays crisp.
        if ctx.eased > 0.02 && ctx.eased < 0.995 {
            let steps = 40;
            for s in 0..steps {
                let rr = 2.0 + s as f32;
                let x = cx + sweep.cos() * rr;
                let y = cy + sweep.sin() * rr * 0.5;
                if x < 0.0 || x >= w as f32 || y < 0.0 || y >= h as f32 {
                    break;
                }
                draw::dot_i(grid, x as i32, y as i32);
                let _ = grid.set_cell_color(
                    (x as usize / 2).min(ctx.width - 1),
                    (y as usize / 4).min(ctx.height - 1),
                    DS_WHITE,
                );
            }
        }
        Ok(())
    }
}

/// Two expanding ring systems interfere; the fringes crawl as they separate.
struct Moire;
impl ProgressStyle for Moire {
    fn name(&self) -> &str {
        "moire"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Interference rings crawling in from the left"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let mid = h as f32 / 2.0;
        let sep = 12.0 + 7.0 * (TAU * 0.25 * ctx.time).sin();
        let (ax, ay) = (w as f32 / 2.0 - sep, mid);
        let (bx, by) = (w as f32 / 2.0 + sep, mid);
        let grow = ctx.time * 4.0; // rings expand 4 dots/sec; 16 over the loop
        for y in 0..h {
            for x in 0..filled {
                let d1 = {
                    let dx = x as f32 - ax;
                    let dy = (y as f32 - ay) * 2.0;
                    (dx * dx + dy * dy).sqrt()
                };
                let d2 = {
                    let dx = x as f32 - bx;
                    let dy = (y as f32 - by) * 2.0;
                    (dx * dx + dy * dy).sqrt()
                };
                let ring = ((d1 - grow) / 4.0).floor() as i32 + ((d2 - grow) / 4.0).floor() as i32;
                if ring.rem_euclid(2) == 0 {
                    draw::dot(grid, x, y);
                }
                if x % 2 == 0 && y % 4 == 0 {
                    // Fringe hue follows the path difference.
                    let fringe = ((d1 - d2) / 10.0).rem_euclid(1.0);
                    let _ = grid.set_cell_color(x / 2, y / 4, spectrum(fringe + ctx.time * 0.25));
                }
            }
        }
        // Track dots ahead of the reveal.
        for x in (filled..w).step_by(4) {
            draw::dot(grid, x, mid as usize);
            let _ = grid.set_cell_color(x / 2, (mid as usize) / 4, DS_TRACK);
        }
        Ok(())
    }
}

/// A parade of glowing blitter bobs joins the loop one per tenth of progress.
struct BobParade;
impl ProgressStyle for BobParade {
    fn name(&self) -> &str {
        "bob-parade"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Blitter bobs joining a Lissajous parade"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let total = 10;
        let visible = (1.0 + ctx.eased * (total as f32 - 1.0)).round() as i32;
        for i in (0..visible).rev() {
            // Trail follows the leader at fixed phase lag on a closed curve.
            let phi = TAU * (ctx.time * 0.25) - i as f32 * 0.22;
            let x = w as f32 * (0.5 + 0.42 * (2.0 * phi).sin());
            let y = h as f32 * (0.5 + 0.34 * (3.0 * phi + 1.3).sin());
            let r = if i == 0 { 3i32 } else { 2i32 };
            for dy in -r..=r {
                for dx in -(r * 2)..=(r * 2) {
                    if dx * dx + dy * dy * 4 <= r * r * 4 {
                        draw::dot_i(grid, x as i32 + dx, y as i32 + dy);
                    }
                }
            }
            let hue = i as f32 / total as f32 + ctx.time * 0.25;
            let c = if i == 0 {
                mix(spectrum(hue), DS_WHITE, 0.6)
            } else {
                spectrum(hue)
            };
            let cell = (
                (x as usize / 2).min(ctx.width - 1),
                (y as usize / 4).min(ctx.height - 1),
            );
            for dcx in -1..=1i32 {
                let cxp = cell.0 as i32 + dcx;
                if cxp >= 0 && (cxp as usize) < ctx.width {
                    let _ = grid.set_cell_color(cxp as usize, cell.1, c);
                }
            }
        }
        // Baseline read.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 1);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
        }
        Ok(())
    }
}
