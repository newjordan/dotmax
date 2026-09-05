//! `gameboy` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O gameboy.rs && ./gameboy [style-name]
//! ```

const DEFAULT_STYLE: &str = "pokemon-hp";

// ===========================================================================
// Minimal runtime — a drop-in stand-in for the dotmax types the styles use.
// Identical braille dot mapping and glyph-override semantics to the crate.
// ===========================================================================

/// RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug)]
pub enum DotmaxError {
    OutOfBounds {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
}

/// A `width x height` cell canvas; every cell is 2x4 braille dots, an
/// optional glyph override, and an optional color.
pub struct BrailleGrid {
    width: usize,
    height: usize,
    patterns: Vec<u8>,
    characters: Vec<Option<char>>,
    colors: Vec<Option<Color>>,
}

impl BrailleGrid {
    pub fn new(width: usize, height: usize) -> Result<Self, DotmaxError> {
        let width = width.max(1);
        let height = height.max(1);
        Ok(Self {
            width,
            height,
            patterns: vec![0; width * height],
            characters: vec![None; width * height],
            colors: vec![None; width * height],
        })
    }

    #[must_use]
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_dot(&mut self, dot_x: usize, dot_y: usize) -> Result<(), DotmaxError> {
        if dot_x >= self.width * 2 || dot_y >= self.height * 4 {
            return Err(DotmaxError::OutOfBounds {
                x: dot_x,
                y: dot_y,
                width: self.width * 2,
                height: self.height * 4,
            });
        }
        let index = (dot_y / 4) * self.width + dot_x / 2;
        let bit = match (dot_x % 2, dot_y % 4) {
            (0, 0) => 0x01,
            (0, 1) => 0x02,
            (0, 2) => 0x04,
            (0, 3) => 0x40,
            (1, 0) => 0x08,
            (1, 1) => 0x10,
            (1, 2) => 0x20,
            _ => 0x80,
        };
        self.patterns[index] |= bit;
        Ok(())
    }

    pub fn set_char(&mut self, x: usize, y: usize, character: char) -> Result<(), DotmaxError> {
        self.check_cell(x, y)?;
        self.characters[y * self.width + x] = Some(character);
        Ok(())
    }

    pub fn set_cell_color(&mut self, x: usize, y: usize, color: Color) -> Result<(), DotmaxError> {
        self.check_cell(x, y)?;
        self.colors[y * self.width + x] = Some(color);
        Ok(())
    }

    pub fn enable_color_support(&mut self) {}

    #[must_use]
    pub fn get_char(&self, x: usize, y: usize) -> char {
        if x >= self.width || y >= self.height {
            return '\u{2800}';
        }
        let index = y * self.width + x;
        if let Some(ch) = self.characters[index] {
            return ch;
        }
        char::from_u32(0x2800 + u32::from(self.patterns[index])).unwrap_or('\u{2800}')
    }

    #[must_use]
    pub fn get_color(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.colors[y * self.width + x]
    }

    fn check_cell(&self, x: usize, y: usize) -> Result<(), DotmaxError> {
        if x >= self.width || y >= self.height {
            return Err(DotmaxError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        Ok(())
    }
}

pub mod progress {
//! Modular loading / progress bars for braille terminals.
//!
//! This module is built to be **lifted out and dropped into other programs**.
//! Everything a bar needs arrives through one immutable [`BarContext`], and
//! every bar is a stateless implementor of [`ProgressStyle`] — so a bar is a
//! pure function of `(progress, time)` and can be rendered anywhere a
//! [`BrailleGrid`] exists.
//!
//! # Anatomy
//!
//! - [`easing`] — the tweening core (`f32 -> f32` curves), dependency-free.
//! - [`BarContext`] — the per-frame inputs (progress, elapsed time, size, palette).
//! - [`ProgressStyle`] — the one trait every bar implements.
//! - [`draw`] — small braille drawing helpers shared by all themed bars.
//! - [`all_styles`] / [`styles_for_theme`] — the registry of every bundled bar.
//!
//! # Quick start
//!
//! ```
//! use dotmax::BrailleGrid;
//! use dotmax::progress::{all_styles, BarContext};
//!
//! let styles = all_styles();
//! let style = &styles[0];
//!
//! let mut grid = BrailleGrid::new(40, 3).unwrap();
//! let ctx = BarContext::new(0.42, 1.5, 40, 3);
//! style.render(&mut grid, &ctx).unwrap();
//! ```
//!
//! # Injecting your own bar
//!
//! Implement [`ProgressStyle`] on any type and render it exactly like a
//! bundled one — no registration required:
//!
//! ```
//! use dotmax::BrailleGrid;
//! use dotmax::progress::{BarContext, ProgressStyle, draw};
//! use dotmax::DotmaxError;
//!
//! struct MyBar;
//! impl ProgressStyle for MyBar {
//!     fn name(&self) -> &str { "my-bar" }
//!     fn theme(&self) -> &str { "custom" }
//!     fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
//!         let (w, h) = draw::dot_dims(grid);
//!         let filled = (ctx.eased * w as f32) as usize;
//!         draw::fill_rect(grid, 0, 0, filled, h);
//!         Ok(())
//!     }
//! }
//! ```

pub mod easing {
//! Tweening / easing math — the shared interpolation core for progress bars.
//!
//! All easing functions are pure `f32 -> f32` maps on the unit interval: they
//! take a normalized time `t` in `[0.0, 1.0]` and return an eased value
//! (also nominally in `[0.0, 1.0]`, though `Back` and `Elastic` deliberately
//! overshoot). This makes them trivial to extract and reuse anywhere — there
//! is no dependency on the rest of dotmax in this file.
//!
//! # Example
//!
//! ```
//! use dotmax::progress::easing::{Easing, ease, lerp};
//!
//! // Ease a value 30% of the way through with a cubic curve.
//! let e = ease(Easing::CubicInOut, 0.3);
//!
//! // Interpolate between two endpoints using the eased fraction.
//! let pixels = lerp(0.0, 100.0, e);
//! assert!(pixels >= 0.0 && pixels <= 100.0);
//! ```

use std::f32::consts::PI;

/// Catalogue of easing curves (Robert Penner's set plus a few extras).
///
/// Variants are grouped as `In` (accelerate from zero), `Out` (decelerate to
/// one), and `InOut` (accelerate then decelerate). Pass any variant to
/// [`ease`] together with a normalized time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Easing {
    /// No easing; returns `t` unchanged.
    Linear,
    /// Quadratic acceleration (`t²`).
    QuadIn,
    /// Quadratic deceleration.
    QuadOut,
    /// Quadratic acceleration then deceleration.
    QuadInOut,
    /// Cubic acceleration (`t³`).
    CubicIn,
    /// Cubic deceleration.
    CubicOut,
    /// Cubic acceleration then deceleration.
    CubicInOut,
    /// Quartic acceleration (`t⁴`).
    QuartIn,
    /// Quartic deceleration.
    QuartOut,
    /// Quartic acceleration then deceleration.
    QuartInOut,
    /// Quintic acceleration (`t⁵`).
    QuintIn,
    /// Quintic deceleration.
    QuintOut,
    /// Quintic acceleration then deceleration.
    QuintInOut,
    /// Sinusoidal acceleration.
    SineIn,
    /// Sinusoidal deceleration.
    SineOut,
    /// Sinusoidal acceleration then deceleration.
    SineInOut,
    /// Exponential acceleration.
    ExpoIn,
    /// Exponential deceleration.
    ExpoOut,
    /// Exponential acceleration then deceleration.
    ExpoInOut,
    /// Circular acceleration.
    CircIn,
    /// Circular deceleration.
    CircOut,
    /// Circular acceleration then deceleration.
    CircInOut,
    /// Anticipatory pull-back before accelerating (overshoots below 0).
    BackIn,
    /// Overshoots past 1 then settles.
    BackOut,
    /// Pull-back at both ends.
    BackInOut,
    /// Spring-like oscillation accelerating in.
    ElasticIn,
    /// Spring-like oscillation decelerating out.
    ElasticOut,
    /// Spring-like oscillation at both ends.
    ElasticInOut,
    /// Accelerating bounce (mirror of `BounceOut`).
    BounceIn,
    /// Decelerating bounce, like a ball settling.
    BounceOut,
    /// Bounce at both ends.
    BounceInOut,
}

/// Linear interpolation between `a` and `b` by fraction `t`.
///
/// `t` is not clamped; pass an eased value from [`ease`] for curved motion.
#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    t.mul_add(b - a, a)
}

/// Clamp `t` into `[0.0, 1.0]`.
#[must_use]
pub fn clamp01(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// Apply the given easing curve to a normalized time `t`.
///
/// `t` is clamped to `[0.0, 1.0]` before evaluation. The result is generally
/// in `[0.0, 1.0]` but `Back` and `Elastic` variants overshoot by design.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn ease(kind: Easing, t: f32) -> f32 {
    let t = clamp01(t);
    match kind {
        Easing::Linear => t,
        Easing::QuadIn => t * t,
        Easing::QuadOut => t * (2.0 - t),
        Easing::QuadInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                (4.0 - 2.0 * t).mul_add(t, -1.0)
            }
        }
        Easing::CubicIn => t * t * t,
        Easing::CubicOut => {
            let f = t - 1.0;
            f * f * f + 1.0
        }
        Easing::CubicInOut => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let f = 2.0f32.mul_add(t, -2.0);
                0.5f32.mul_add(f * f * f, 1.0)
            }
        }
        Easing::QuartIn => t * t * t * t,
        Easing::QuartOut => {
            let f = t - 1.0;
            1.0 - f * f * f * f
        }
        Easing::QuartInOut => {
            if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                let f = t - 1.0;
                (-8.0f32).mul_add(f * f * f * f, 1.0)
            }
        }
        Easing::QuintIn => t * t * t * t * t,
        Easing::QuintOut => {
            let f = t - 1.0;
            f * f * f * f * f + 1.0
        }
        Easing::QuintInOut => {
            if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                let f = 2.0f32.mul_add(t, -2.0);
                0.5f32.mul_add(f * f * f * f * f, 1.0)
            }
        }
        Easing::SineIn => 1.0 - (t * PI / 2.0).cos(),
        Easing::SineOut => (t * PI / 2.0).sin(),
        Easing::SineInOut => 0.5 * (1.0 - (PI * t).cos()),
        Easing::ExpoIn => {
            if t <= 0.0 {
                0.0
            } else {
                (10.0f32 * (t - 1.0)).exp2()
            }
        }
        Easing::ExpoOut => {
            if t >= 1.0 {
                1.0
            } else {
                1.0 - (-10.0f32 * t).exp2()
            }
        }
        Easing::ExpoInOut => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else if t < 0.5 {
                0.5 * (20.0f32 * t - 10.0).exp2()
            } else {
                (-0.5f32).mul_add((-20.0f32 * t + 10.0).exp2(), 1.0)
            }
        }
        Easing::CircIn => 1.0 - (1.0 - t * t).sqrt(),
        Easing::CircOut => {
            let f = t - 1.0;
            (1.0 - f * f).sqrt()
        }
        Easing::CircInOut => {
            if t < 0.5 {
                0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt())
            } else {
                let f = (-2.0f32).mul_add(t, 2.0);
                0.5 * ((1.0 - f * f).sqrt() + 1.0)
            }
        }
        Easing::BackIn => {
            const C1: f32 = 1.701_58;
            const C3: f32 = C1 + 1.0;
            C3.mul_add(t * t * t, -(C1 * t * t))
        }
        Easing::BackOut => {
            const C1: f32 = 1.701_58;
            const C3: f32 = C1 + 1.0;
            let f = t - 1.0;
            C3.mul_add(f * f * f, C1 * f * f) + 1.0
        }
        Easing::BackInOut => {
            const C1: f32 = 1.701_58;
            const C2: f32 = C1 * 1.525;
            if t < 0.5 {
                let f = 2.0 * t;
                0.5 * (f * f * (C2.mul_add(f, f) - C2))
            } else {
                let f = 2.0f32.mul_add(t, -2.0);
                0.5 * f.mul_add(f * C2.mul_add(f, f) + C2, 2.0)
            }
        }
        Easing::ElasticIn => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                const C4: f32 = 2.0 * PI / 3.0;
                -(10.0f32 * (t - 1.0)).exp2() * ((t - 1.0) * 10.0 - 0.75).mul_add(C4, 0.0).sin()
            }
        }
        Easing::ElasticOut => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                const C4: f32 = 2.0 * PI / 3.0;
                (-10.0f32 * t).exp2() * (t * 10.0 - 0.75).mul_add(C4, 0.0).sin() + 1.0
            }
        }
        Easing::ElasticInOut => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                const C5: f32 = 2.0 * PI / 4.5;
                let s = (20.0f32 * t - 11.125) * C5;
                if t < 0.5 {
                    -0.5 * (20.0f32 * t - 10.0).exp2() * s.sin()
                } else {
                    0.5f32.mul_add((-20.0f32 * t + 10.0).exp2() * s.sin(), 1.0)
                }
            }
        }
        Easing::BounceIn => 1.0 - bounce_out(1.0 - t),
        Easing::BounceOut => bounce_out(t),
        Easing::BounceInOut => {
            if t < 0.5 {
                0.5 * (1.0 - bounce_out(1.0 - 2.0 * t))
            } else {
                0.5f32.mul_add(bounce_out(2.0f32.mul_add(t, -1.0)), 0.5)
            }
        }
    }
}

/// The canonical decelerating "bounce" curve, used to build all bounce easings.
fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1.mul_add(t * t, 0.75)
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1.mul_add(t * t, 0.9375)
    } else {
        let t = t - 2.625 / D1;
        N1.mul_add(t * t, 0.984_375)
    }
}

/// Every easing variant, in catalogue order — handy for demos and pickers.
pub const ALL_EASINGS: [Easing; 31] = [
    Easing::Linear,
    Easing::QuadIn,
    Easing::QuadOut,
    Easing::QuadInOut,
    Easing::CubicIn,
    Easing::CubicOut,
    Easing::CubicInOut,
    Easing::QuartIn,
    Easing::QuartOut,
    Easing::QuartInOut,
    Easing::QuintIn,
    Easing::QuintOut,
    Easing::QuintInOut,
    Easing::SineIn,
    Easing::SineOut,
    Easing::SineInOut,
    Easing::ExpoIn,
    Easing::ExpoOut,
    Easing::ExpoInOut,
    Easing::CircIn,
    Easing::CircOut,
    Easing::CircInOut,
    Easing::BackIn,
    Easing::BackOut,
    Easing::BackInOut,
    Easing::ElasticIn,
    Easing::ElasticOut,
    Easing::ElasticInOut,
    Easing::BounceIn,
    Easing::BounceOut,
    Easing::BounceInOut,
];



}

use crate::{BrailleGrid, Color, DotmaxError};

pub use easing::{ease, lerp, Easing};

/// Per-frame inputs handed to a [`ProgressStyle`].
///
/// A bar reads everything it needs from here and writes only into the grid,
/// which keeps bars stateless and trivially reusable across programs.
#[derive(Debug, Clone)]
pub struct BarContext {
    /// Raw completion fraction in `[0.0, 1.0]`.
    pub progress: f32,
    /// Eased completion fraction. Defaults to `progress`; set via
    /// [`BarContext::with_easing`] to apply a tween for non-linear fill.
    pub eased: f32,
    /// Seconds elapsed since the bar started — drives looping animation so
    /// bars can shimmer, scroll, or pulse independently of `progress`.
    pub time: f32,
    /// Target width in terminal **cells** (dots wide = `width * 2`).
    pub width: usize,
    /// Target height in terminal **cells** (dots tall = `height * 4`).
    pub height: usize,
    /// Optional accent palette. Bars should fall back gracefully when colors
    /// are absent (the grid may not have color support enabled).
    pub palette: Palette,
    /// Optional short label some bars render alongside the fill (e.g. "42%").
    pub label: Option<String>,
}

impl BarContext {
    /// Build a context with linear (un-eased) progress and a default palette.
    #[must_use]
    pub fn new(progress: f32, time: f32, width: usize, height: usize) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        Self {
            progress,
            eased: progress,
            time,
            width,
            height,
            palette: Palette::default(),
            label: None,
        }
    }

    /// Apply an easing curve, populating [`BarContext::eased`].
    #[must_use]
    pub fn with_easing(mut self, kind: Easing) -> Self {
        self.eased = ease(kind, self.progress);
        self
    }

    /// Override the accent palette.
    #[must_use]
    pub const fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Attach a text label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A two-stop accent palette plus a track (background) color.
///
/// Bars may interpolate between `start` and `end` across the fill, and use
/// `track` for the unfilled remainder. All optional — a bar that ignores
/// color still renders correctly in monochrome.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Color at the left / start of the fill.
    pub start: Color,
    /// Color at the right / leading edge of the fill.
    pub end: Color,
    /// Color of the unfilled track.
    pub track: Color,
}

impl Palette {
    /// Sample the start→end gradient at fraction `t` in `[0.0, 1.0]`.
    #[must_use]
    pub fn sample(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::rgb(
            lerp(f32::from(self.start.r), f32::from(self.end.r), t) as u8,
            lerp(f32::from(self.start.g), f32::from(self.end.g), t) as u8,
            lerp(f32::from(self.start.b), f32::from(self.end.b), t) as u8,
        )
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            start: Color::rgb(0, 200, 255),
            end: Color::rgb(120, 80, 255),
            track: Color::rgb(40, 40, 50),
        }
    }
}

/// A loading-bar style. One stateless type per visual style.
///
/// `render` draws the bar's current frame into `grid` based on `ctx`. The grid
/// is sized by the caller; a style should respect [`BarContext::width`] /
/// [`BarContext::height`] or simply fill the grid it is given.
pub trait ProgressStyle {
    /// Stable, kebab-case identifier (unique within a theme).
    fn name(&self) -> &str;
    /// Theme this style belongs to (e.g. `"animals"`, `"tech"`).
    fn theme(&self) -> &str;
    /// One-line human description for galleries / pickers.
    fn describe(&self) -> &str {
        "a loading bar"
    }
    /// Draw one frame of the bar into `grid`.
    ///
    /// # Errors
    /// Returns a [`DotmaxError`] only if the grid rejects a write that the
    /// style did not bounds-check; bundled helpers in [`draw`] never do.
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError>;
}

/// Braille drawing helpers shared by every bundled bar.
///
/// All coordinates are in **dot space** (the grid's `width*2 × height*4`
/// pixel lattice). Every setter silently ignores out-of-bounds writes, so
/// bars can be written without defensive bounds checks.
pub mod draw {
    use crate::BrailleGrid;

    /// Grid size in dots: `(width * 2, height * 4)`.
    #[must_use]
    pub fn dot_dims(grid: &BrailleGrid) -> (usize, usize) {
        let (w, h) = grid.dimensions();
        (w * 2, h * 4)
    }

    /// Set a single dot, ignoring out-of-bounds coordinates.
    pub fn dot(grid: &mut BrailleGrid, x: usize, y: usize) {
        let (w, h) = dot_dims(grid);
        if x < w && y < h {
            let _ = grid.set_dot(x, y);
        }
    }

    /// Set a dot from signed coordinates, ignoring negatives / overflow.
    pub fn dot_i(grid: &mut BrailleGrid, x: i32, y: i32) {
        if x >= 0 && y >= 0 {
            dot(grid, x as usize, y as usize);
        }
    }

    /// Horizontal run of dots from `x0` to `x1` (inclusive) at row `y`.
    pub fn hline(grid: &mut BrailleGrid, x0: usize, x1: usize, y: usize) {
        let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        for x in lo..=hi {
            dot(grid, x, y);
        }
    }

    /// Vertical run of dots from `y0` to `y1` (inclusive) at column `x`.
    pub fn vline(grid: &mut BrailleGrid, x: usize, y0: usize, y1: usize) {
        let (lo, hi) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        for y in lo..=hi {
            dot(grid, x, y);
        }
    }

    /// Filled rectangle of dots: `[x0, x0+w) × [y0, y0+h)`.
    pub fn fill_rect(grid: &mut BrailleGrid, x0: usize, y0: usize, w: usize, h: usize) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                dot(grid, x, y);
            }
        }
    }

    /// Unfilled rectangle outline of dots.
    pub fn rect_outline(grid: &mut BrailleGrid, x0: usize, y0: usize, w: usize, h: usize) {
        if w == 0 || h == 0 {
            return;
        }
        let (x1, y1) = (x0 + w - 1, y0 + h - 1);
        hline(grid, x0, x1, y0);
        hline(grid, x0, x1, y1);
        vline(grid, x0, y0, y1);
        vline(grid, x1, y0, y1);
    }

    /// Tint a horizontal span of **cells** on a row, enabling color support
    /// first. Out-of-range cells are skipped. Cell `x` spans dots `[x*2, x*2+2)`.
    pub fn tint_row(
        grid: &mut BrailleGrid,
        cell_y: usize,
        cell_x0: usize,
        cell_x1: usize,
        color: crate::Color,
    ) {
        grid.enable_color_support();
        let (w, h) = grid.dimensions();
        if cell_y >= h {
            return;
        }
        let hi = cell_x1.min(w.saturating_sub(1));
        for x in cell_x0..=hi {
            let _ = grid.set_cell_color(x, cell_y, color);
        }
    }

    /// Horizontal block-eighths ` ▏▎▍▌▋▊▉█` — for sub-cell-precise *smooth* bars.
    pub const H_BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    /// Vertical block-eighths ` ▁▂▃▄▅▆▇█` — for equalizer columns and *blocky* bars.
    pub const V_BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    /// Shading ramp ` ░▒▓█` — for coarse density / dithered texture.
    pub const SHADES: [char; 5] = [' ', '░', '▒', '▓', '█'];

    /// Place an arbitrary glyph in a **cell**, ignoring out-of-bounds. The cell's
    /// braille dots are overwritten by this glyph. Use for block/symbol styles.
    pub fn glyph(grid: &mut BrailleGrid, cell_x: usize, cell_y: usize, c: char) {
        let _ = grid.set_char(cell_x, cell_y, c);
    }

    /// Draw a single smooth horizontal bar in row `cell_y` filled to `frac`
    /// (`0.0..=1.0`) using eighth-width block glyphs.
    ///
    /// This is the classic crisp, sub-character-precise progress bar. It mixes
    /// full `█` cells with one partial edge glyph for smoothness no braille dot
    /// run can match.
    pub fn hbar(grid: &mut BrailleGrid, cell_y: usize, frac: f32) {
        let (w, _) = grid.dimensions();
        let frac = frac.clamp(0.0, 1.0);
        let eighths = (frac * (w * 8) as f32).round() as usize;
        let full = eighths / 8;
        let rem = eighths % 8;
        for x in 0..full.min(w) {
            glyph(grid, x, cell_y, '█');
        }
        if rem > 0 && full < w {
            glyph(grid, full, cell_y, H_BLOCKS[rem]);
        }
    }

    /// Set a column cell to a vertical fill `level` in `0..=8` (eighths) — for
    /// equalizer / spectrum columns. Level 0 clears nothing visible.
    pub fn vblock(grid: &mut BrailleGrid, cell_x: usize, cell_y: usize, level: usize) {
        glyph(grid, cell_x, cell_y, V_BLOCKS[level.min(8)]);
    }

    /// Shade a cell at coarse density `level` in `0..=4` using ` ░▒▓█`.
    pub fn shade(grid: &mut BrailleGrid, cell_x: usize, cell_y: usize, level: usize) {
        glyph(grid, cell_x, cell_y, SHADES[level.min(4)]);
    }
}

/// Render a one-shot frame of `style` to plain text lines — the easiest way to
/// drop a dotmax bar into a program that isn't using [`TerminalRenderer`].
///
/// Each returned `String` is one row of braille characters. Width/height come
/// from `ctx`. Drive it by calling repeatedly with an increasing `progress`
/// and `time` and reprinting (e.g. with carriage returns or cursor moves).
///
/// [`TerminalRenderer`]: crate::TerminalRenderer
///
/// ```
/// use dotmax::progress::{all_styles, BarContext, render_lines};
///
/// let style = &all_styles()[0];
/// let ctx = BarContext::new(0.6, 0.0, 30, 2);
/// let lines = render_lines(style.as_ref(), &ctx).unwrap();
/// assert_eq!(lines.len(), 2);
/// ```
///
/// # Errors
/// Propagates any [`DotmaxError`] from grid allocation or the style's `render`.
pub fn render_lines(
    style: &dyn ProgressStyle,
    ctx: &BarContext,
) -> Result<Vec<String>, DotmaxError> {
    let mut grid = BrailleGrid::new(ctx.width.max(1), ctx.height.max(1))?;
    style.render(&mut grid, ctx)?;
    // Use `get_char`, not `to_unicode_grid`: the former reflects both braille
    // dots AND block/shade/glyph cells written via `set_char`, so styles built
    // from block elements render correctly here too.
    let (w, h) = grid.dimensions();
    let mut lines = Vec::with_capacity(h);
    for y in 0..h {
        let mut row = String::with_capacity(w);
        for x in 0..w {
            row.push(grid.get_char(x, y));
        }
        lines.push(row);
    }
    Ok(lines)
}

/// Render a one-shot frame of `style` to a single newline-joined string.
///
/// # Errors
/// Propagates any [`DotmaxError`] from [`render_lines`].
pub fn render_string(style: &dyn ProgressStyle, ctx: &BarContext) -> Result<String, DotmaxError> {
    Ok(render_lines(style, ctx)?.join("\n"))
}

pub mod styles {
    pub mod gameboy {
//! Game Boy / handheld LCD progress bars.
//!
//! Eleven styles evoking the washed-out dot-matrix LCD panels of classic
//! handheld gaming: Pokémon HP, Nokia Snake, Tetris GB, Dr. Mario, Kirby,
//! Tamagotchi, Game & Watch, Link's Awakening hearts, Mole Whack, Wario
//! treasure, and LCD Pinball.
//!
//! Every style leans into the `░▒▓` shade ramp to simulate the ghosting and
//! segment-dimming of a genuine reflective-LCD panel. Animation is a pure
//! function of `ctx.time`; fill extent is `ctx.eased`. All writes go through
//! `draw::` helpers — out-of-bounds coordinates are silently discarded.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::cmp::Ordering;
use std::f32::consts::PI;

/// All styles in the `gameboy` theme.
///
/// Returns one boxed implementor per style, in display order. Each struct is a
/// zero-size private unit type — no heap allocation beyond the `Box` itself.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PokemonHp),
        Box::new(NokiaSnake),
        Box::new(TetrisGb),
        Box::new(DrMario),
        Box::new(KirbyInhale),
        Box::new(Tamagotchi),
        Box::new(GameWatch),
        Box::new(HeartContainers),
        Box::new(MoleWhack),
        Box::new(WarioTreasure),
        Box::new(LcdPinball),
    ]
}

// ─── 1. Pokémon HP Bar ───────────────────────────────────────────────────────

/// HP bar drains/fills with a Poké Ball that wobbles mid-capture, settling at 100%.
struct PokemonHp;
impl ProgressStyle for PokemonHp {
    fn name(&self) -> &str {
        "pokemon-hp"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Pokémon-style HP bar with a Poké Ball that wobbles during capture and settles at full"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (_w, _h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── LCD panel background (shade 1 = ░) ──────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── HP label "HP" in the leftmost 2 cells (shade 4 = █) ─────────────
        // Represented as thin vertical bars to mimic dot-matrix lettering.
        if cw >= 2 {
            draw::shade(grid, 0, 0, 4);
            if ch > 1 {
                draw::shade(grid, 0, 1, 4);
            }
        }

        // ── HP track — the green segmented bar ──────────────────────────────
        let bar_start_cell = if cw > 3 { 2 } else { 0 };
        let bar_w = cw.saturating_sub(bar_start_cell + 1).max(1);
        // Background of HP track (shade 2 = ▒)
        for cx in bar_start_cell..bar_start_cell + bar_w {
            for cy in 0..ch {
                draw::shade(grid, cx, cy, 2);
            }
        }
        // Filled HP (shade 4)
        let filled = (ctx.eased * bar_w as f32).round() as usize;
        for cx in bar_start_cell..bar_start_cell + filled.min(bar_w) {
            for cy in 0..ch {
                draw::shade(grid, cx, cy, 4);
            }
        }

        // ── Poké Ball in the last cell ───────────────────────────────────────
        if cw > 0 {
            let ball_cx = cw.saturating_sub(1);
            // Wobble angle — dampens to zero as progress → 1.0
            let settle = 1.0 - ctx.eased;
            let wobble = (ctx.time * 8.0).sin() * settle * 0.5;
            // Express wobble as a shade level: rocking between ▒ and █
            let shade_lvl = if wobble.abs() > 0.25 { 3usize } else { 4 };
            for cy in 0..ch {
                draw::shade(grid, ball_cx, cy, shade_lvl);
            }
            // Central equator line on the ball — always shade 2
            if ch >= 2 {
                draw::shade(grid, ball_cx, ch / 2, 2);
            }
        }

        // ── Palette tint — green HP, red when low ───────────────────────────
        let health_t = ctx.eased; // 0=red, 1=green via palette
        let filled_cells = bar_start_cell + filled.min(bar_w);
        for cy in 0..ch {
            if filled_cells > bar_start_cell {
                draw::tint_row(
                    grid,
                    cy,
                    bar_start_cell,
                    filled_cells.saturating_sub(1),
                    ctx.palette.sample(health_t),
                );
            }
        }
        Ok(())
    }
}

// ─── 2. Nokia Snake ──────────────────────────────────────────────────────────

/// Snake grows segment by segment as progress rises, eating pellets on a dim LCD grid.
struct NokiaSnake;
impl ProgressStyle for NokiaSnake {
    fn name(&self) -> &str {
        "nokia-snake"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Nokia-style snake grows from left; segments = progress, pellets sparkle with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── Snake body in dot space ──────────────────────────────────────────
        // Snake travels in a boustrophedon (row-alternating) path.
        let snake_dots = (ctx.eased * (w * h) as f32) as usize;
        let mid_y = h / 2;

        for seg in 0..snake_dots.min(w * h) {
            // Boustrophedon: even rows go right, odd rows go left.
            let row = seg / w;
            let col_in_row = seg % w;
            let x = if row % 2 == 0 {
                col_in_row
            } else {
                w.saturating_sub(1 + col_in_row)
            };
            let y = row;
            draw::dot(grid, x, y.min(h.saturating_sub(1)));
        }

        // ── Pellets: 2-dot squares at regular intervals, dim if eaten ────────
        let pellet_spacing = (w / 5).max(2);
        let pellet_count = w / pellet_spacing;
        let eaten = (ctx.eased * pellet_count as f32) as usize;
        for p in 0..pellet_count {
            let px = p * pellet_spacing + pellet_spacing / 2;
            if p >= eaten {
                // Uneaten pellet — blink with time
                let blink = ((ctx.time * 3.0 + p as f32 * 0.7).sin() > 0.0) as usize;
                let shade = 2 + blink; // ▒ or ▓
                draw::dot(grid, px.min(w.saturating_sub(1)), mid_y);
                draw::dot(grid, px.saturating_sub(1), mid_y);
                let _ = shade; // shade already expressed via dot presence/absence
            }
            // Eaten pellets leave no trace (already set by snake body or absent)
        }

        // ── Head: 3×3 block at the snake tip ─────────────────────────────────
        if snake_dots > 0 {
            let head_seg = snake_dots.saturating_sub(1);
            let row = head_seg / w;
            let col_in_row = head_seg % w;
            let hx = if row % 2 == 0 {
                col_in_row
            } else {
                w.saturating_sub(1 + col_in_row)
            };
            let hy = row.min(h.saturating_sub(1));
            for dy in 0..2i32 {
                for dx in 0..2i32 {
                    draw::dot_i(grid, hx as i32 + dx, hy as i32 + dy);
                }
            }
        }

        // ── Dim LCD background: only cells the snake left blank ─────────────
        // (a shade glyph overrides braille dots, so it must come last)
        for cy in 0..ch {
            for cx in 0..cw {
                if grid.get_char(cx, cy) == '\u{2800}' {
                    draw::shade(grid, cx, cy, 1);
                }
            }
        }

        // ── Palette tint on filled region ────────────────────────────────────
        let filled_cells = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            if filled_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    filled_cells.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 3. Tetris GB ────────────────────────────────────────────────────────────

/// Shaded blocks stack in a well from the bottom; height = progress.
struct TetrisGb;
impl ProgressStyle for TetrisGb {
    fn name(&self) -> &str {
        "tetris-gb"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Shaded Tetris blocks stack in a GB-style well; block density = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();

        // ── Well background (shade 1) ─────────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Stacked blocks — each cell gets a shade based on depth ────────────
        // Blocks fill from the bottom. Stack height = eased fraction of rows.
        let stack_rows = (ctx.eased * ch as f32).round() as usize;
        let base_row = ch.saturating_sub(stack_rows);

        // Tetromino columns vary phase slightly for the ragged skyline.
        let col_w = 2usize; // 2 cells per "block column"
        let block_cols = (cw / col_w).max(1);

        for bc in 0..block_cols {
            // Each block column has a small phase offset in its height.
            let phase = (bc as f32 / block_cols as f32) * PI;
            let col_extra = (phase.sin() * 1.0).round() as i32;
            let col_base = (base_row as i32 - col_extra).max(0) as usize;

            for row in col_base..ch {
                // Shade deepens toward the bottom — deeper = denser.
                let depth = ch.saturating_sub(row);
                let shade_lvl = match depth {
                    d if d >= stack_rows => 4,
                    d if d * 4 >= stack_rows * 3 => 3,
                    d if d * 2 >= stack_rows => 2,
                    _ => 1,
                };
                for col_off in 0..col_w {
                    let cx = bc * col_w + col_off;
                    if cx < cw {
                        draw::shade(grid, cx, row, shade_lvl);
                    }
                }
            }

            // Block dividers: a shade-1 gap row every 2 rows from the bottom.
            // (Makes the stack look like individual pieces.)
            let mut div_row = ch.saturating_sub(2);
            while div_row > col_base {
                for col_off in 0..col_w {
                    let cx = bc * col_w + col_off;
                    if cx < cw && div_row < ch {
                        draw::shade(grid, cx, div_row, 1);
                    }
                }
                div_row = div_row.saturating_sub(2);
            }
        }

        // ── Currently falling piece — a single 2×2 block that drops with time ─
        let fall_period = 0.8f32;
        let fall_t = (ctx.time % fall_period) / fall_period;
        let fall_row = (fall_t * base_row.max(1) as f32) as usize;
        let piece_col = ((ctx.time * 0.3) as usize % block_cols.max(1)) * col_w;
        for col_off in 0..col_w {
            let cx = piece_col + col_off;
            if cx < cw && fall_row < base_row {
                draw::shade(grid, cx, fall_row, 4);
            }
        }

        // ── Palette tint on filled stack ─────────────────────────────────────
        for cy in base_row..ch {
            let t = (cy.saturating_sub(base_row)) as f32 / stack_rows.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─── 4. Dr. Mario ────────────────────────────────────────────────────────────

/// Pills appear and stack; each cleared row is progress.
struct DrMario;
impl ProgressStyle for DrMario {
    fn name(&self) -> &str {
        "dr-mario"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Dr. Mario pills stack in a bottle; rows cleared = progress; cleared rows flash"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();

        // ── Bottle background (shade 1) ───────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Bottle walls (shade 4 left + right column) ────────────────────────
        for cy in 0..ch {
            draw::shade(grid, 0, cy, 4);
            if cw > 1 {
                draw::shade(grid, cw.saturating_sub(1), cy, 4);
            }
        }

        // Usable interior width.
        let inner_cw = cw.saturating_sub(2).max(1);
        let inner_start = 1usize;

        // ── Pills fill from the bottom, alternating two-cell pill pairs ────────
        let pill_cells = (ctx.eased * (inner_cw * ch) as f32) as usize;
        let pill_rows_filled = (pill_cells / inner_cw.max(1)).min(ch);

        let base_row = ch.saturating_sub(pill_rows_filled);

        for row in base_row..ch {
            let is_cleared = row % 3 == 0; // every 3rd row is a "cleared" row
                                           // Cleared rows flash with time.
            let flash = is_cleared && ((ctx.time * 6.0) as usize % 2 == 0);
            let shade_lvl: usize = if flash { 1 } else { 3 };

            // Pills: alternating pairs of shade 3 / shade 4 within the row.
            let mut cx = inner_start;
            let mut pill_idx = 0usize;
            while cx < inner_start + inner_cw {
                let lvl = if pill_idx % 2 == 0 {
                    shade_lvl
                } else {
                    shade_lvl.saturating_sub(1).max(2)
                };
                draw::shade(grid, cx, row, lvl);
                if cx + 1 < inner_start + inner_cw {
                    draw::shade(grid, cx + 1, row, lvl.min(4));
                }
                cx += 2;
                pill_idx += 1;
            }
        }

        // ── Falling pill at the top ────────────────────────────────────────────
        let drop_t = (ctx.time * 1.5).fract();
        let drop_row = (drop_t * base_row.max(1) as f32) as usize;
        let drop_col = inner_start + ((ctx.time * 0.5) as usize % inner_cw.max(1));
        if drop_row < base_row && drop_col < inner_start + inner_cw {
            draw::shade(grid, drop_col, drop_row, 4);
            if drop_col + 1 < inner_start + inner_cw {
                draw::shade(grid, drop_col + 1, drop_row, 3);
            }
        }

        // ── Palette tint on pill rows ─────────────────────────────────────────
        for row in base_row..ch {
            let t = (row.saturating_sub(base_row)) as f32 / pill_rows_filled.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(
                grid,
                row,
                inner_start,
                inner_start + inner_cw.saturating_sub(1),
                color,
            );
        }
        Ok(())
    }
}

// ─── 5. Kirby Inhale ─────────────────────────────────────────────────────────

/// Kirby "inhale" meter — cheeks puff with eased, stars inhaled with time.
struct KirbyInhale;
impl ProgressStyle for KirbyInhale {
    fn name(&self) -> &str {
        "kirby-inhale"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Kirby puffs up as the meter fills; stars streak toward his mouth with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── Dim LCD background ────────────────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Inhale power bar across the bottom rows ───────────────────────────
        // A meter of shade glyphs showing current inhale strength.
        let meter_row = ch.saturating_sub(1);
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..cw {
            let lvl = if cx < filled_cells { 4 } else { 2 };
            draw::shade(grid, cx, meter_row, lvl);
        }

        // ── Kirby body in dot space — circle that expands with eased ──────────
        // Kirby is in the right third of the bar.
        let kirby_cx = (w * 3 / 4) as i32;
        let kirby_cy = (h / 2) as i32;
        // Base radius + puff proportional to eased.
        let min_r = (h / 4).max(2) as i32;
        let max_r = (h / 2).max(3) as i32;
        let r = min_r + ((ctx.eased * (max_r - min_r) as f32).round() as i32);

        // Draw Kirby's body circle.
        let steps = (2.0 * PI * r as f32 * 1.5) as usize + 8;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            let dx = (a.cos() * r as f32).round() as i32;
            let dy = (a.sin() * r as f32 * 0.7).round() as i32; // slightly squished
            draw::dot_i(grid, kirby_cx + dx, kirby_cy + dy);
        }
        // Filled interior dots.
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx * 100 + dy * dy * 100 * 100 / 49 <= r * r * 100 {
                    draw::dot_i(grid, kirby_cx + dx, kirby_cy + dy);
                }
            }
        }

        // Eyes — two dots above center.
        let eye_y = kirby_cy - r / 3;
        draw::dot_i(grid, kirby_cx - r / 3, eye_y);
        draw::dot_i(grid, kirby_cx + r / 3, eye_y);

        // Mouth — a small open "O" when inhaling (progress < 1.0).
        if ctx.eased < 0.95 {
            let mouth_y = kirby_cy + r / 3;
            draw::dot_i(grid, kirby_cx - 1, mouth_y);
            draw::dot_i(grid, kirby_cx + 1, mouth_y);
            draw::dot_i(grid, kirby_cx, mouth_y + 1);
        }

        // ── Inhaled stars streaking toward Kirby ──────────────────────────────
        let star_count = 5usize;
        for i in 0..star_count {
            let phase = i as f32 / star_count as f32;
            let t = (ctx.time * 1.5 + phase).fract(); // 0→1 travel
                                                      // Stars come from the left.
            let sx = ((1.0 - t) * kirby_cx as f32) as i32;
            let sy = kirby_cy + ((phase * 2.0 - 1.0) * (h / 4) as f32) as i32;
            draw::dot_i(grid, sx, sy);
            draw::dot_i(grid, sx + 1, sy);
        }

        // ── Palette tint — bluer when empty, fuller when puffed ──────────────
        let body_cell_x = (kirby_cx as usize / 2).saturating_sub(r as usize / 2);
        let body_cell_w = (r as usize).min(cw.saturating_sub(body_cell_x));
        for cy in 0..ch.saturating_sub(1) {
            if body_cell_w > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    body_cell_x,
                    body_cell_x + body_cell_w.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
            // Star trail region tinted at lower intensity.
            if body_cell_x > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    body_cell_x.saturating_sub(1),
                    ctx.palette.sample(ctx.eased * 0.4),
                );
            }
        }
        Ok(())
    }
}

// ─── 6. Tamagotchi ───────────────────────────────────────────────────────────

/// Tamagotchi pet hunger/growth meter; pet animates (toddles) with time.
struct Tamagotchi;
impl ProgressStyle for Tamagotchi {
    fn name(&self) -> &str {
        "tamagotchi"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Tamagotchi hunger/growth bar; the pixel-art pet toddles and blinks with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, _h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── LCD background (shade 1) ───────────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Hunger bar: shade segments across the top row ─────────────────────
        let bar_cells = cw.saturating_sub(4).max(1); // leave right side for pet
        let bar_filled = (ctx.eased * bar_cells as f32).round() as usize;
        for cx in 0..bar_cells {
            let lvl = if cx < bar_filled { 4usize } else { 2 };
            draw::shade(grid, cx, 0, lvl);
        }
        // Meter ticks: shade-1 every 2 cells.
        let mut tick = 2usize;
        while tick < bar_cells {
            draw::shade(grid, tick, 0, 1);
            tick += 3;
        }

        // ── Pet sprite in dot space (right 8 dots of the bar) ─────────────────
        // Pet is a simple 5×7 pixel-art egg-blob.
        let pet_x = (w.saturating_sub(10)) as i32;
        let pet_y = 1i32;
        // Walk animation: bobble up/down with time.
        let walk_frame = ((ctx.time * 4.0) as usize) % 2;
        let bob = i32::from(walk_frame != 0);

        // Body: oval.
        let bx = pet_x;
        let by = pet_y + bob;
        // 5×5 body (filled dots).
        for dy in 0..5i32 {
            for dx in 0..5i32 {
                // Rounded corners: skip (0,0),(4,0),(0,4),(4,4)
                if (dx == 0 || dx == 4) && (dy == 0 || dy == 4) {
                    continue;
                }
                draw::dot_i(grid, bx + dx, by + dy);
            }
        }
        // Eyes: two gaps in the body at row 1.
        // (draw inverse: no-op — the eye is absence of dots. Represent with shade.)
        // Feet: two dots at the bottom.
        draw::dot_i(grid, bx + 1, by + 5);
        draw::dot_i(grid, bx + 3, by + 5);

        // Blink: at blink moment erase the eye-row dots.
        let blink = (ctx.time * 2.5).fract() > 0.85;
        if !blink {
            // Eyes open: leave body dots as-is, mark eye positions with a shade glyph.
            // (We can't erase dots, so we use shade glyphs in cell space.)
            let eye_cell_x = (bx as usize) / 2;
            let eye_cell_y = (by as usize) / 4;
            if eye_cell_x + 1 < cw && eye_cell_y < ch {
                // Use shade-2 to simulate lighter "white of eye" in those cells.
                draw::shade(grid, eye_cell_x, eye_cell_y, 2);
                if eye_cell_x + 2 < cw {
                    draw::shade(grid, eye_cell_x + 2, eye_cell_y, 2);
                }
            }
        }

        // Growth: pet cell shaded by hunger/growth level.
        let pet_shade = match ctx.eased {
            e if e >= 0.8 => 4,
            e if e >= 0.5 => 3,
            e if e >= 0.2 => 2,
            _ => 1,
        };
        let pet_cell_x = (pet_x.max(0) as usize) / 2;
        let pet_cell_w = 3usize;
        for cy in 1..ch {
            for px in 0..pet_cell_w {
                let cx = pet_cell_x + px;
                if cx < cw {
                    draw::shade(grid, cx, cy, pet_shade.min(4));
                }
            }
        }

        // ── Palette tint ──────────────────────────────────────────────────────
        for cy in 0..ch {
            if bar_filled > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    bar_filled.min(cw).saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 7. Game & Watch ─────────────────────────────────────────────────────────

/// Flat LCD segment figure juggles; ball position cycles through discrete segment states.
struct GameWatch;
impl ProgressStyle for GameWatch {
    fn name(&self) -> &str {
        "game-watch"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Game & Watch segment LCD: a juggler tosses balls; active count = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();

        // ── LCD panel — ALL cells start at shade 1 (the "ghosted" segment look) ─
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Juggler figure: fixed LCD segments in a 3-column region ──────────
        // Figures are discrete: head, body, hands, feet.
        // Position figure in the left quarter.
        let fig_col = cw / 4;

        // Head (shade 4 at top).
        if fig_col < cw {
            draw::shade(grid, fig_col, 0, 4);
        }
        // Body (shade 3 for 1..ch-2).
        for cy in 1..ch.saturating_sub(1) {
            if fig_col < cw {
                draw::shade(grid, fig_col, cy, 3);
            }
        }
        // Arms: the frame toggles which arm is raised.
        let arm_frame = ((ctx.time * 3.0) as usize) % 2;
        if arm_frame == 0 {
            // Left arm up.
            if fig_col > 0 {
                draw::shade(grid, fig_col.saturating_sub(1), 0, 4);
            }
            if fig_col + 1 < cw && ch > 1 {
                draw::shade(grid, fig_col + 1, 1, 3);
            }
        } else {
            // Right arm up.
            if fig_col + 1 < cw {
                draw::shade(grid, fig_col + 1, 0, 4);
            }
            if fig_col > 0 && ch > 1 {
                draw::shade(grid, fig_col.saturating_sub(1), 1, 3);
            }
        }
        // Feet.
        if ch > 1 {
            if fig_col > 0 {
                draw::shade(grid, fig_col.saturating_sub(1), ch.saturating_sub(1), 3);
            }
            if fig_col + 1 < cw {
                draw::shade(grid, fig_col + 1, ch.saturating_sub(1), 3);
            }
        }

        // ── Balls: discrete arc positions cycling with time ───────────────────
        // Number of active balls = progress * max_balls.
        let max_balls = ((cw.saturating_sub(fig_col + 2)) / 3).max(1);
        let active_balls = (ctx.eased * max_balls as f32).round() as usize;

        // Arc has 4 discrete positions (game & watch style).
        let arc_positions: [(i32, i32); 4] = [
            (0, 0),  // hand level
            (1, -1), // going up
            (2, -2), // apex
            (3, -1), // coming down
        ];

        for b in 0..active_balls.min(max_balls) {
            let phase_offset = b as f32 / active_balls.max(1) as f32;
            let arc_idx = ((ctx.time * 2.0 + phase_offset * 4.0) as usize) % 4;
            let (arc_dx, arc_dy) = arc_positions[arc_idx];
            let ball_col =
                (fig_col as i32 + fig_col as i32 / 2 + arc_dx + b as i32 * 2).max(0) as usize;
            let ball_row = (ch as i32 / 2 + arc_dy).clamp(0, ch as i32 - 1) as usize;
            if ball_col < cw {
                draw::shade(grid, ball_col, ball_row, 4);
            }
        }

        // ── Score track: shade segments across the right portion ──────────────
        let score_start = cw * 3 / 4;
        let score_w = cw.saturating_sub(score_start);
        for cx in score_start..score_start + score_w {
            if cx < cw {
                let seg_active =
                    cx.saturating_sub(score_start) < active_balls * score_w / max_balls.max(1);
                draw::shade(grid, cx, ch / 2, if seg_active { 4 } else { 2 });
            }
        }

        // ── Palette tint on active regions ────────────────────────────────────
        for cy in 0..ch {
            let active_w = (ctx.eased * cw as f32) as usize;
            if active_w > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    active_w.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 8. Link's Awakening Hearts ──────────────────────────────────────────────

/// Heart containers fill one at a time, like Link's Awakening health display.
struct HeartContainers;
impl ProgressStyle for HeartContainers {
    fn name(&self) -> &str {
        "hearts"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Link's Awakening heart containers: each fills one at a time; last pulses"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();

        // ── LCD background ────────────────────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Hearts: each heart = 2 cells wide, displayed in the center rows ───
        let heart_w = 2usize; // cells per heart
        let max_hearts = (cw / heart_w).max(1);
        let full_hearts = (ctx.eased * max_hearts as f32) as usize;
        let partial_frac = (ctx.eased * max_hearts as f32).fract();

        // Pulse animation on the leading (currently-filling) heart.
        let pulse = (ctx.time * 6.0).sin() * 0.5 + 0.5; // 0..1

        for h_idx in 0..max_hearts {
            let cx_start = h_idx * heart_w;
            let heart_shade = match h_idx.cmp(&full_hearts) {
                Ordering::Less => {
                    // Fully filled heart.
                    4usize
                }
                Ordering::Equal => {
                    // Partially filled — shade by partial fraction + pulse.
                    let lvl = (partial_frac * 3.0 + pulse * 0.5) as usize;
                    (lvl + 1).min(4)
                }
                Ordering::Greater => {
                    // Empty heart container.
                    2
                }
            };

            // Each heart: top row is 2 bumps (shade 4 at edges, 3 in middle),
            // bottom row is a V-shape (shade 4).
            // We use cell rows from top to bottom.
            let top_row = ch / 4;
            let bot_row = ch / 2;

            // Top bumps.
            for cy in top_row..=top_row {
                if cy < ch {
                    for off in 0..heart_w {
                        let cx = cx_start + off;
                        if cx < cw {
                            draw::shade(grid, cx, cy, heart_shade);
                        }
                    }
                }
            }
            // Bottom V.
            for cy in bot_row..=bot_row {
                if cy < ch {
                    // Middle cell shade (V tip).
                    let mid_cx = cx_start + heart_w / 2;
                    if mid_cx < cw {
                        draw::shade(grid, mid_cx, cy, heart_shade);
                    }
                    // Flanks slightly dimmer.
                    if cx_start < cw {
                        draw::shade(grid, cx_start, cy, heart_shade.saturating_sub(1).max(1));
                    }
                    if cx_start + heart_w.saturating_sub(1) < cw {
                        draw::shade(
                            grid,
                            cx_start + heart_w.saturating_sub(1),
                            cy,
                            heart_shade.saturating_sub(1).max(1),
                        );
                    }
                }
            }
            // Body fill between rows.
            if bot_row > top_row + 1 {
                for cy in top_row + 1..bot_row {
                    for off in 0..heart_w {
                        let cx = cx_start + off;
                        if cx < cw && cy < ch {
                            draw::shade(grid, cx, cy, heart_shade.saturating_sub(1).max(1));
                        }
                    }
                }
            }
        }

        // ── Palette tint: filled hearts ───────────────────────────────────────
        let filled_cells = (full_hearts * heart_w).min(cw);
        for cy in 0..ch {
            if filled_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    filled_cells.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 9. Mole Whack ───────────────────────────────────────────────────────────

/// Moles pop up in parabolic arcs; bonked moles = progress.
struct MoleWhack;
impl ProgressStyle for MoleWhack {
    fn name(&self) -> &str {
        "mole-whack"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Whack-a-Mole: moles pop from holes in parabolic arcs; bonked count = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── Ground line ───────────────────────────────────────────────────────
        let ground_y = h.saturating_sub(3);
        draw::hline(grid, 0, w.saturating_sub(1), ground_y);
        draw::hline(grid, 0, w.saturating_sub(1), ground_y + 1);

        // ── Holes: shade-2 ovals at fixed x positions below the ground ────────
        let hole_count = (cw / 4).max(1);
        let hole_spacing = cw / hole_count;

        for h_idx in 0..hole_count {
            let hole_cx = h_idx * hole_spacing + hole_spacing / 2;
            if hole_cx < cw && ch > 0 {
                draw::shade(grid, hole_cx, ch.saturating_sub(1), 2);
                if hole_cx + 1 < cw {
                    draw::shade(grid, hole_cx + 1, ch.saturating_sub(1), 2);
                }
            }
        }

        // ── Moles: bonked_count rise from their holes ─────────────────────────
        let bonked = (ctx.eased * hole_count as f32) as usize;

        for h_idx in 0..hole_count {
            let hole_cx_cell = h_idx * hole_spacing + hole_spacing / 2;
            let hole_x = hole_cx_cell * 2;

            let is_bonked = h_idx < bonked;

            if is_bonked {
                // Bonked mole — stays peeking at the rim as a shade-3 stub.
                let mole_y = ground_y.saturating_sub(1);
                draw::dot(grid, hole_x.min(w.saturating_sub(1)), mole_y);
                draw::dot(grid, (hole_x + 1).min(w.saturating_sub(1)), mole_y);
                // Stars above (bonk effect, animated with time).
                let star_anim = ((ctx.time * 5.0 + h_idx as f32) as usize) % 3;
                if star_anim < 2 && mole_y >= 2 {
                    draw::dot(
                        grid,
                        hole_x.min(w.saturating_sub(1)),
                        mole_y.saturating_sub(2),
                    );
                    draw::dot(
                        grid,
                        (hole_x + 2).min(w.saturating_sub(1)),
                        mole_y.saturating_sub(2),
                    );
                }
            } else {
                // Un-bonked mole — parabolic pop animation.
                let phase = h_idx as f32 * 0.7;
                let t = (ctx.time * 1.8 + phase).fract(); // 0→1
                                                          // Parabola: up on first half, down on second.
                let arc = 1.0 - (t * 2.0 - 1.0).powi(2);
                let mole_dot_y = (ground_y as f32 - arc * (ground_y as f32 * 0.8)).round() as usize;

                // Mole body: 2×3 block.
                for dy in 0..3usize {
                    for dx in 0..2usize {
                        let mx = (hole_x + dx).min(w.saturating_sub(1));
                        let my = mole_dot_y + dy;
                        if my < ground_y {
                            draw::dot(grid, mx, my);
                        }
                    }
                }
                // Mole eyes (2 dots).
                if mole_dot_y + 1 < ground_y {
                    draw::dot(grid, hole_x.min(w.saturating_sub(1)), mole_dot_y);
                    draw::dot(grid, (hole_x + 1).min(w.saturating_sub(1)), mole_dot_y);
                }
            }
        }

        // ── Dim LCD background: only cells left blank by the scene ───────────
        // (a shade glyph overrides braille dots, so it must come last)
        for cy in 0..ch {
            for cx in 0..cw {
                if grid.get_char(cx, cy) == '\u{2800}' {
                    draw::shade(grid, cx, cy, 1);
                }
            }
        }

        // ── Palette tint ──────────────────────────────────────────────────────
        for cy in 0..ch {
            let bonked_cells = (bonked * cw / hole_count.max(1)).min(cw);
            if bonked_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    bonked_cells.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 10. Wario Treasure ──────────────────────────────────────────────────────

/// Coin counter / treasure meter — coins flip and stack as progress rises.
struct WarioTreasure;
impl ProgressStyle for WarioTreasure {
    fn name(&self) -> &str {
        "wario-treasure"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Wario coin meter: coins flip and land in a chest; treasure fraction = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── LCD background ────────────────────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Chest / treasure box — right half of the bar ──────────────────────
        // Chest outline in cell space.
        let chest_start = cw / 2;
        let chest_w = cw.saturating_sub(chest_start).max(1);
        // Lid (top row).
        for cx in chest_start..chest_start + chest_w {
            if cx < cw {
                draw::shade(grid, cx, 0, 4);
            }
        }
        // Sides.
        for cy in 1..ch {
            if chest_start < cw {
                draw::shade(grid, chest_start, cy, 4);
            }
            if chest_start + chest_w.saturating_sub(1) < cw {
                draw::shade(grid, chest_start + chest_w.saturating_sub(1), cy, 4);
            }
        }
        // Bottom.
        if ch > 0 {
            for cx in chest_start..chest_start + chest_w {
                if cx < cw {
                    draw::shade(grid, cx, ch.saturating_sub(1), 4);
                }
            }
        }

        // Chest fill: shade 3/4 from the bottom up proportional to eased.
        let fill_rows = (ctx.eased * (ch.saturating_sub(2)) as f32).round() as usize;
        let fill_start = ch.saturating_sub(1 + fill_rows).max(1);
        for cy in fill_start..ch.saturating_sub(1) {
            for cx in chest_start + 1..chest_start + chest_w.saturating_sub(1) {
                if cx < cw && cy < ch {
                    let t = (cy.saturating_sub(fill_start)) as f32 / fill_rows.max(1) as f32;
                    let shade_lvl = if t < 0.5 { 3usize } else { 4 };
                    draw::shade(grid, cx, cy, shade_lvl);
                }
            }
        }

        // ── Coins: stacked shade circles in the left half ─────────────────────
        let coin_area_w = chest_start.max(1);
        let total_coin_cells = coin_area_w * ch;
        let filled_coins = (ctx.eased * total_coin_cells as f32) as usize;

        // Fill coins column by column, bottom-up.
        let cols = coin_area_w;
        let rows = ch;
        for idx in 0..filled_coins.min(total_coin_cells) {
            let col = idx % cols;
            let row_from_bottom = idx / cols;
            let row = rows.saturating_sub(1 + row_from_bottom);
            if col < cw && row < ch {
                draw::shade(grid, col, row, 3);
            }
        }

        // ── Flying coin animation ─────────────────────────────────────────────
        // A coin arcs from left pile to the chest.
        let coin_t = (ctx.time * 1.2).fract();
        let coin_x = (coin_t * (w.saturating_sub(2)) as f32) as i32;
        let arc_h = (h / 2) as f32;
        let coin_y = (arc_h * (1.0 - (coin_t * PI).sin())).round() as i32;
        draw::dot_i(grid, coin_x, coin_y);
        draw::dot_i(grid, coin_x + 1, coin_y);
        // Coin flip: squeeze horizontally at 0/1 of arc, full at peak.
        let flip = (ctx.time * 6.0).sin();
        if flip.abs() > 0.3 {
            draw::dot_i(grid, coin_x, coin_y + 1);
            draw::dot_i(grid, coin_x + 1, coin_y + 1);
        }

        // ── Palette tint ──────────────────────────────────────────────────────
        // Gold on coins, green/blue on chest fill.
        let coin_cells = (ctx.eased * coin_area_w as f32) as usize;
        for cy in 0..ch {
            if coin_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    coin_cells.min(cw).saturating_sub(1),
                    ctx.palette.sample(0.9),
                );
            }
            if fill_rows > 0 && fill_start <= cy && cy < ch.saturating_sub(1) && chest_start < cw {
                draw::tint_row(
                    grid,
                    cy,
                    chest_start + 1,
                    (chest_start + chest_w.saturating_sub(2)).min(cw.saturating_sub(1)),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 11. LCD Pinball ─────────────────────────────────────────────────────────

/// LCD segment pinball: flippers, bumpers, and a ball; lit bumpers = progress.
struct LcdPinball;
impl ProgressStyle for LcdPinball {
    fn name(&self) -> &str {
        "lcd-pinball"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "LCD pinball: segment flippers and bumpers; lit bumpers = progress, ball bounces with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── Pinball table background ───────────────────────────────────────────
        for cy in 0..ch {
            for cx in 0..cw {
                draw::shade(grid, cx, cy, 1);
            }
        }

        // ── Side walls ────────────────────────────────────────────────────────
        draw::vline(grid, 0, 0, h.saturating_sub(1));
        draw::vline(grid, w.saturating_sub(1), 0, h.saturating_sub(1));

        // ── Bumpers: shade ovals in a row near the top ────────────────────────
        let bumper_count = (cw / 3).clamp(1, 5);
        let bumper_spacing = cw / (bumper_count + 1);
        let lit_bumpers = (ctx.eased * bumper_count as f32).round() as usize;

        for b in 0..bumper_count {
            let bx = (b + 1) * bumper_spacing;
            let by = ch / 3;
            if bx < cw && by < ch {
                let shade_lvl = if b < lit_bumpers { 4usize } else { 2 };
                draw::shade(grid, bx, by, shade_lvl);
                // Bumper halo.
                if bx + 1 < cw {
                    draw::shade(grid, bx + 1, by, (shade_lvl.saturating_sub(1)).max(1));
                }
                if bx > 0 {
                    draw::shade(
                        grid,
                        bx.saturating_sub(1),
                        by,
                        (shade_lvl.saturating_sub(1)).max(1),
                    );
                }
                if by > 0 {
                    draw::shade(
                        grid,
                        bx,
                        by.saturating_sub(1),
                        (shade_lvl.saturating_sub(1)).max(1),
                    );
                }
                if by + 1 < ch {
                    draw::shade(grid, bx, by + 1, (shade_lvl.saturating_sub(1)).max(1));
                }
            }
        }

        // ── Flippers at the bottom ────────────────────────────────────────────
        // Two flipper segments that angle up/down with time.
        let flip_row = ch.saturating_sub(2);
        let mid_cell = cw / 2;
        let flipper_len = (cw / 4).max(1);

        // Left flipper — angles based on time.
        let left_raised = (ctx.time * 3.0).sin() > 0.3;
        let left_shade = if left_raised { 4usize } else { 3 };
        for i in 0..flipper_len {
            let cx = mid_cell.saturating_sub(flipper_len).saturating_sub(1) + i;
            let cy = if left_raised && i < flipper_len / 2 {
                flip_row.saturating_sub(1)
            } else {
                flip_row
            };
            if cx < cw && cy < ch {
                draw::shade(grid, cx, cy, left_shade);
            }
        }

        // Right flipper.
        let right_raised = !left_raised;
        let right_shade = if right_raised { 4usize } else { 3 };
        for i in 0..flipper_len {
            let cx = mid_cell + 1 + i;
            let cy = if right_raised && i >= flipper_len / 2 {
                flip_row.saturating_sub(1)
            } else {
                flip_row
            };
            if cx < cw && cy < ch {
                draw::shade(grid, cx, cy, right_shade);
            }
        }

        // ── Ball trajectory ────────────────────────────────────────────────────
        // Ball bounces in an elliptical path around the playfield.
        let ball_t = ctx.time * 1.5;
        let ball_cx = w / 2;
        let ball_cy_center = h / 2;
        let rx = (w.saturating_sub(4) / 2) as f32;
        let ry = (h.saturating_sub(4) / 2).max(1) as f32;
        let bx = (ball_cx as f32 + ball_t.cos() * rx).round() as i32;
        let by = (ball_cy_center as f32 + (ball_t * 0.7).sin() * ry).round() as i32;
        // 2×2 ball.
        draw::dot_i(grid, bx, by);
        draw::dot_i(grid, bx + 1, by);
        draw::dot_i(grid, bx, by + 1);
        draw::dot_i(grid, bx + 1, by + 1);

        // ── Palette tint on lit bumper region ─────────────────────────────────
        for cy in 0..ch {
            let active_w = (ctx.eased * cw as f32) as usize;
            if active_w > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    active_w.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::gameboy::styles();
    let Some(style) = styles.iter().find(|s| s.name() == name) else {
        eprintln!("unknown style '{name}'. available in this file:");
        for s in &styles {
            eprintln!("  {:<18} {}", s.name(), s.describe());
        }
        std::process::exit(1);
    };

    let (width, height) = (44usize, 4usize);
    let fps = 12u64;
    let loop_frames = 96u64;
    println!("{} - {}  (ctrl-c to quit)", style.name(), style.describe());
    let mut frame = 0u64;
    loop {
        let phase = (frame % loop_frames) as f32 / loop_frames as f32;
        let progress = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
        let time = frame as f32 / fps as f32;
        let ctx = progress::BarContext::new(progress, time, width, height)
            .with_easing(progress::Easing::CubicInOut)
            .with_label(format!("{:.0}%", progress * 100.0));
        let mut grid = BrailleGrid::new(width, height).expect("grid");
        style.render(&mut grid, &ctx).expect("render");

        let mut out = String::new();
        for y in 0..height {
            let mut current: Option<Color> = None;
            for x in 0..width {
                let color = grid.get_color(x, y);
                if color != current {
                    match color {
                        Some(c) => out.push_str(&format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)),
                        None => out.push_str("\x1b[0m"),
                    }
                    current = color;
                }
                out.push(grid.get_char(x, y));
            }
            out.push_str("\x1b[0m\n");
        }
        print!("{out}");
        use std::io::Write as _;
        std::io::stdout().flush().expect("flush");
        std::thread::sleep(std::time::Duration::from_millis(1000 / fps));
        print!("\x1b[{height}A");
        frame += 1;
    }
}
