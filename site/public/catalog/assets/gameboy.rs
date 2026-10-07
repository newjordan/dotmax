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

    pub fn clear(&mut self) {
        self.patterns.fill(0);
        self.colors.fill(None);
    }

    pub fn clear_region(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), DotmaxError> {
        let (end_x, end_y) = (x.saturating_add(width), y.saturating_add(height));
        if end_x > self.width || end_y > self.height {
            return Err(DotmaxError::OutOfBounds {
                x: end_x.saturating_sub(1),
                y: end_y.saturating_sub(1),
                width: self.width,
                height: self.height,
            });
        }
        for row in y..end_y {
            for col in x..end_x {
                self.patterns[row * self.width + col] = 0;
                self.colors[row * self.width + col] = None;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn get_raw_patterns(&self) -> &[u8] {
        &self.patterns
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
//! Each style draws its game's recognisable objects as monochrome dot-art,
//! like a reflective-LCD panel: a Poké Ball, bevelled tetrominoes, a pill
//! bottle, an egg-shaped Tamagotchi, heart containers, a treasure chest, an
//! LCD juggler and a pinball table. Animation is a pure function of
//! `ctx.time`; fill extent is `ctx.eased`. All writes go through `draw::`
//! helpers — out-of-bounds coordinates are silently discarded.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
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

// ---------------------------------------------------------------------------
// Sprite helpers — dot-art stamps for the pictorial styles.
// ---------------------------------------------------------------------------

/// Stamp a sprite: every `#` in `rows` sets the dot at `(x + col, y + row)`.
/// Off-grid dots are dropped, so sprites may hang off any edge.
fn sprite(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + c as i32, y + r as i32);
            }
        }
    }
}

/// Triangle wave `0 → 1 → 0` with period 1.
fn tri(t: f32) -> f32 {
    1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs()
}

/// Solid block of dots from signed coordinates (clipped to the grid).
fn block(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32) {
    for yy in y..y + h {
        for xx in x..x + w {
            draw::dot_i(grid, xx, yy);
        }
    }
}

// ─── 1. Pokémon HP Bar ───────────────────────────────────────────────────────

/// A Poké Ball (solid top shell, centre band, ringed button) wobbles mid-
/// capture beside the battle HUD's `HP` bar; the wobble dies away as the bar
/// fills and the ball sits still at 100%.
struct PokemonHp;
impl ProgressStyle for PokemonHp {
    fn name(&self) -> &str {
        "pokemon-hp"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Pokémon battle HUD: a Poké Ball wobbles mid-capture beside an `HP` bar that fills with progress; the ball settles at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Poké Ball: rotate the sample point to wobble, then classify it.
        let r = ((hi - 1) as f32 / 2.0).min(7.5);
        let (cx, cy) = (r + 1.0, (hi - 1) as f32 / 2.0);
        let wobble = 0.45 * (ctx.time * 7.0).sin() * (1.0 - ctx.eased).max(0.0);
        let (s, c) = wobble.sin_cos();
        for y in 0..hi {
            for x in 0..(2.0 * r + 3.0) as i32 {
                let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                let (u, v) = (dx * c + dy * s, -dx * s + dy * c);
                let d = (u * u + v * v).sqrt();
                let on = if d > r + 0.3 {
                    false
                } else if d < 1.2 {
                    true // button
                } else if d < 2.3 {
                    false // button gap
                } else if d < 3.3 {
                    true // button ring
                } else {
                    d > r - 0.8 || v < -0.5 || v.abs() < 0.5
                };
                if on {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // "HP" label and the rounded bar outline.
        let lx = (2.0 * r + 5.0) as i32;
        let ly = hi / 2 - 3;
        sprite(
            grid,
            lx,
            ly,
            &["#.#.##.", "#.#.#.#", "###.##.", "#.#.#..", "#.#.#.."],
        );
        let bx = lx + 9;
        let bw = (wi - bx - 1).max(0);
        if bw > 4 {
            draw::hline(grid, (bx + 1) as usize, (bx + bw - 2) as usize, ly as usize);
            draw::hline(
                grid,
                (bx + 1) as usize,
                (bx + bw - 2) as usize,
                (ly + 4) as usize,
            );
            draw::vline(grid, bx as usize, (ly + 1) as usize, (ly + 3) as usize);
            draw::vline(
                grid,
                (bx + bw - 1) as usize,
                (ly + 1) as usize,
                (ly + 3) as usize,
            );
            let fill = (ctx.eased * (bw - 4) as f32).round() as i32;
            block(grid, bx + 2, ly + 2, fill, 1);
            if fill > 0 {
                block(grid, bx + 2, ly + 1, fill, 3);
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

/// Bevelled GB minos lock into a four-row strip one tetromino at a time —
/// I, O, O, L, J repeating so the strip tiles solid — while the next piece
/// drops in from the top.
struct TetrisGb;

/// One tetromino: mino cells as `(col, row)` within the 4-row strip.
type Piece = [(i32, i32); 4];

/// The repeating 5-column tiling, in lock order (bottom pieces first).
const TILING: [Piece; 5] = [
    [(0, 0), (0, 1), (0, 2), (0, 3)], // I
    [(1, 2), (2, 2), (1, 3), (2, 3)], // O (bottom)
    [(1, 0), (2, 0), (1, 1), (2, 1)], // O (top)
    [(4, 1), (4, 2), (4, 3), (3, 3)], // J
    [(3, 0), (3, 1), (3, 2), (4, 0)], // L
];

impl ProgressStyle for TetrisGb {
    fn name(&self) -> &str {
        "tetris-gb"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Game Boy Tetris: I, O, L and J tetrominoes, each with its own block texture, lock into a solid stack left-to-right while the next piece drops in; stack length = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cols = (wi / 4).max(1);
        let units = (cols + 4) / 5;
        let total = units * TILING.len() as i32;
        let placed = (ctx.eased * total as f32) as i32;

        // A mino: 3×3 on a 4-dot pitch, centred vertically. Like the GB game,
        // each piece type has its own block texture so pieces read apart.
        let top = (hi - 16).max(0) / 2;
        let mino = |grid: &mut BrailleGrid, kind: usize, col: i32, row: i32, dy: i32| {
            let art: [&str; 3] = match kind {
                0 => ["###", "###", "###"],     // I: solid
                1 | 2 => ["###", "#.#", "###"], // O: framed
                3 => ["#.#", ".#.", "#.#"],     // J: crossed
                _ => [".#.", "###", ".#."],     // L: studded
            };
            sprite(grid, col * 4, top + row * 4 + dy, &art);
        };
        for k in 0..total.min(placed + 1) {
            let piece = TILING[(k % 5) as usize];
            let base = (k / 5) * 5;
            // The next piece is still falling: it lifts off the top as time runs.
            let dy = if k == placed {
                -(((1.0 - (ctx.time * 1.5).fract()) * 16.0) as i32)
            } else {
                0
            };
            for (c, r) in piece {
                if base + c < cols {
                    mino(grid, (k % 5) as usize, base + c, r, dy);
                }
            }
        }
        Ok(())
    }
}

// ─── 4. Dr. Mario ────────────────────────────────────────────────────────────

/// The Dr. Mario bottle (neck, shoulders, glass body) full of wriggling
/// viruses that vanish as progress rises, with a two-tone capsule dropping in
/// through the neck; a row of capsules alongside fills with progress.
struct DrMario;

/// A capsule: two rounded halves with a seam.
const CAPSULE: [&str; 3] = [".##.##.", "###.###", ".##.##."];

impl ProgressStyle for DrMario {
    fn name(&self) -> &str {
        "dr-mario"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Dr. Mario: wriggling viruses vanish from the medicine bottle as a capsule drops through its neck; the capsule row beside it fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Bottle: neck on top, shoulders, then the glass body.
        let bw = 24.min(wi);
        draw::vline(grid, 8, 0, 3);
        draw::vline(grid, 15, 0, 3);
        draw::hline(grid, 0, 8, 3);
        draw::hline(grid, 15, (bw - 1).max(0) as usize, 3);
        draw::vline(grid, 0, 3, h - 1);
        draw::vline(grid, (bw - 1).max(0) as usize, 3, h - 1);
        draw::hline(grid, 0, (bw - 1).max(0) as usize, h - 1);

        // Viruses: a 5×2 grid of critters, cleared in a scattered order.
        let wiggle = (ctx.time * 4.0).fract() < 0.5;
        let virus: [&str; 3] = if wiggle {
            ["#.#", ".#.", "#.#"]
        } else {
            [".#.", "###", "#.#"]
        };
        for i in 0..10 {
            let (col, row) = (i % 5, i / 5);
            let order = ((i * 3 + 1) % 10) as f32 / 10.0;
            if order >= ctx.eased {
                sprite(grid, 2 + col * 4, hi - 9 + row * 4, &virus);
            }
        }

        // A capsule tumbling down the neck into the bottle.
        let fall = ((ctx.time * 0.8).fract() * (hi - 9).max(1) as f32) as i32;
        sprite(grid, 9, fall - 2, &[".##.", "####", "####", ".##."]);

        // Capsule row: one more capsule per step of progress.
        let slots = ((wi - bw - 4) / 8).max(0);
        let filled = (ctx.eased * slots as f32).round() as i32;
        for k in 0..slots {
            let x = bw + 4 + k * 8;
            if k < filled {
                sprite(grid, x, hi / 2 - 1, &CAPSULE);
            } else {
                draw::dot_i(grid, x + 3, hi / 2);
            }
        }
        Ok(())
    }
}

// ─── 5. Kirby Inhale ─────────────────────────────────────────────────────────

/// Pink round Kirby (red feet, rosy cheek, stubby arm) walks right with
/// progress, mouth gaping, inhaling a whirling stream of stars and Waddle
/// Dees along wind lines; the ground behind him is the track covered.
struct KirbyInhale;

/// Waddle Dee: round body, face patch with two eyes, little feet.
const WADDLE: [&str; 5] = [".###.", "#####", "#.#.#", "#####", "##.##"];
/// Five-point star.
const STAR5: [&str; 5] = ["..#..", "..#..", "#####", ".###.", "##.##"];

impl ProgressStyle for KirbyInhale {
    fn name(&self) -> &str {
        "kirby-inhale"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Pink Kirby with red feet and rosy cheeks walks right with progress, mouth wide, inhaling a stream of stars and Waddle Dees"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        // Kirby: body radius 6, feet on the bottom rows.
        let top = hi - 16;
        let travel = (wi - 34).max(0) as f32;
        let kx = 8 + (ctx.eased * travel).round() as i32;
        let bob = i32::from((t * 4.0).fract() >= 0.5);
        let cy = top + 6 + bob;
        let (mx, my) = (kx + 5, cy + 1); // mouth centre
        for dy in -6..=6 {
            for dx in -7..=6 {
                let inside = dx * dx + dy * dy <= 42;
                let arm = dx <= -6 && (0..=1).contains(&dy);
                if !(inside || arm) {
                    continue;
                }
                let (x, y) = (kx + dx, cy + dy);
                // Gaping mouth: an oval hole on the right side, with a lip.
                let (ox, oy) = (x - mx, y - my);
                let d = ox * ox + oy * oy;
                if d <= 10 && ox > -3 {
                    if d >= 7 && ox <= 0 {
                        draw::dot_i(grid, x, y);
                    }
                    continue;
                }
                // Eyes: tall ovals.
                if (dx == 0 || dx == 2) && (-4..=-2).contains(&dy) {
                    continue;
                }
                draw::dot_i(grid, x, y);
            }
        }
        // Red feet.
        sprite(grid, kx - 7, cy + 6, &[".#####.", "#######"]);
        sprite(grid, kx, cy + 6, &[".#####.", "#######"]);

        // Ground: solid behind Kirby, dashed ahead.
        let gy = hi - 1;
        for x in 0..wi {
            if x < kx - 7 || (x > kx + 7 && x % 4 == 0) {
                draw::dot_i(grid, x, gy);
            }
        }

        // Wind lines funnelling into the mouth.
        let far = wi - 1;
        let span = (far - mx).max(1) as f32;
        for i in 0..4 {
            let s = (t * 1.4 + i as f32 * 0.25).fract();
            let x = far - (s * span) as i32;
            let off = ((i % 2) * 2 - 1) as f32 * (1.0 - s) * 6.0;
            let y = my + off.round() as i32;
            for k in 0..4 {
                if x + k > mx + 2 {
                    draw::dot_i(grid, x + k, y);
                }
            }
        }
        // Stars and a Waddle Dee swirl in and vanish into the mouth.
        let mut dee_x = i32::MIN;
        for i in 0..3 {
            let s = (t * 0.6 + i as f32 / 3.0).fract();
            let x = far - (s * span) as i32;
            if x <= mx + 2 {
                continue;
            }
            let wob = ((t * 5.0 + i as f32 * 2.0).sin() * (1.0 - s) * 4.0).round() as i32;
            let y = my - 2 + wob;
            if i == 1 {
                sprite(grid, x, y, &WADDLE);
                dee_x = x;
            } else {
                sprite(grid, x, y, &STAR5);
            }
        }

        // Colour: pink body, rosy cheek, red feet, yellow stars, orange Dee.
        grid.enable_color_support();
        for cyy in 0..ch {
            for cxx in 0..cw {
                let (px, py) = (cxx as i32 * 2 + 1, cyy as i32 * 4 + 2);
                let color = if (kx - 8..=kx + 7).contains(&px) && py >= cy + 6 {
                    Color::rgb(232, 40, 70) // feet
                } else if (kx - 8..=kx + 7).contains(&px) {
                    if (kx - 3..=kx).contains(&px) && (cy..=cy + 3).contains(&py) {
                        Color::rgb(255, 80, 140) // blush
                    } else {
                        Color::rgb(255, 160, 205)
                    }
                } else if px < kx {
                    Color::rgb(120, 200, 110) // grass track
                } else if (dee_x - 1..=dee_x + 5).contains(&px) {
                    Color::rgb(255, 150, 60) // Waddle Dee
                } else {
                    Color::rgb(255, 230, 100) // stars and wind
                };
                let _ = grid.set_cell_color(cxx, cyy, color);
            }
        }
        Ok(())
    }
}

// ─── 6. Tamagotchi ───────────────────────────────────────────────────────────

/// The egg-shaped Tamagotchi with its LCD window and three buttons; the pet
/// inside grows from baby to adult with progress and toddles about, blinking,
/// while the hunger hearts beside it fill.
struct Tamagotchi;
impl ProgressStyle for Tamagotchi {
    fn name(&self) -> &str {
        "tamagotchi"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Tamagotchi: an egg-shaped toy with LCD window and three buttons; the pet grows baby→adult and toddles about while the hunger hearts fill with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;

        // Egg shell: an ellipse that narrows toward the top.
        let (cx, cy) = (11.0_f32, (hi - 1) as f32 / 2.0 + 0.5);
        let ry = (hi as f32 / 2.0).max(1.0);
        for y in 0..hi {
            for x in 0..23 {
                let v = (y as f32 - cy) / ry;
                let rx = 10.5 * (1.0 + 0.12 * v);
                let u = (x as f32 - cx) / rx;
                let d = (u * u + v * v).sqrt();
                if (0.86..1.0).contains(&d) {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // LCD window and the three buttons below it.
        draw::rect_outline(grid, 5, 3, 13, 8);
        for bx in [7, 11, 15] {
            draw::dot_i(grid, bx, 13);
        }

        // The pet: baby, child, then adult, toddling across the LCD.
        let stage: &[&str] = if ctx.eased < 0.34 {
            &[".##.", "#..#", ".##."]
        } else if ctx.eased < 0.67 {
            &[".###.", "#.#.#", "#####", ".#.#."]
        } else {
            &["#...#", ".###.", "#.#.#", "#####", ".#.#."]
        };
        let pw = stage[0].len() as i32;
        let px = 6 + (tri(ctx.time * 0.35) * (11 - pw).max(0) as f32) as i32;
        let py = 10 - stage.len() as i32;
        sprite(grid, px, py, stage);
        if (ctx.time * 0.7).fract() < 0.1 {
            // Blink: fill the eye row closed.
            draw::hline(
                grid,
                px.max(0) as usize,
                (px + pw - 1).max(0) as usize,
                (py + 1).max(0) as usize,
            );
        }

        // Hunger hearts: four of them, filling in half-heart steps.
        let halves = (ctx.eased * 8.0).round() as i32;
        for i in 0..4 {
            let left = halves - i * 2;
            let art = if left >= 2 {
                &HEART_FULL
            } else if left == 1 {
                &HEART_HALF
            } else {
                &HEART_EMPTY
            };
            sprite(grid, 28 + i * 10, hi / 2 - 3, art);
        }
        Ok(())
    }
}
// ─── 7. Game & Watch ─────────────────────────────────────────────────────────

/// Game & Watch *Ball*: the LCD juggler stands centre-stage with three nested
/// arcs of fixed ball segments overhead (unlit ones ghost faintly); balls hop
/// segment to segment and progress adds balls and lights the score ticks.
struct GameWatch;
impl ProgressStyle for GameWatch {
    fn name(&self) -> &str {
        "game-watch"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Game & Watch Ball: an LCD juggler under three arcs of ghosted ball segments; balls hop segment to segment, more join and score ticks light as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cx = wi / 2;

        // The juggler; his hands swap high/low with the throw rhythm.
        let beat = (ctx.time * 3.0).fract() < 0.5;
        let arms = if beat { "#..###...." } else { "....###..#" };
        let body = [
            "....###...",
            "....#.#...",
            "....###...",
            arms,
            ".#######..",
            "....###...",
            "....###...",
            "...#...#..",
            "..#.....#.",
        ];
        sprite(grid, cx - 5, hi - 9, &body);

        // Three nested arcs of LCD ball segments; lit balls are 2×2 blocks.
        let balls = 1 + (ctx.eased * 2.99) as i32;
        let slots = 7;
        for k in 0..3 {
            let (rx, ry) = (12.0 + k as f32 * 9.0, 5.0 + k as f32 * 1.5);
            let base = hi as f32 - 7.0;
            let tick = (ctx.time * 5.0) as i32 + k * 3;
            let span = (slots - 1) * 2;
            let pos = tick.rem_euclid(span);
            let lit = if pos < slots { pos } else { span - pos };
            for s in 0..slots {
                let a = PI * s as f32 / (slots - 1) as f32;
                let x = cx + (a.cos() * rx) as i32;
                let y = (base - a.sin() * ry) as i32;
                if k < balls && s == lit {
                    block(grid, x - 1, y - 1, 2, 2);
                } else {
                    draw::dot_i(grid, x, y); // ghost segment
                }
            }
        }

        // Score ticks along the top-left: ten LCD dashes.
        let ticks = (ctx.eased * 10.0).round() as i32;
        for i in 0..10 {
            if i < ticks {
                block(grid, i * 3, 0, 2, 1);
            } else {
                draw::dot_i(grid, i * 3, 0);
            }
        }
        Ok(())
    }
}

// ─── 8. Link's Awakening Hearts ──────────────────────────────────────────────

/// The Link's Awakening HUD: B/A item boxes (shield, sword) beside two rows
/// of seven heart containers that fill half a heart at a time; the next empty
/// heart blinks.
struct HeartContainers;

const HEART_FULL: [&str; 6] = [
    ".##.##.", "#######", "#######", ".#####.", "..###..", "...#...",
];
const HEART_HALF: [&str; 6] = [
    ".##.##.", "####..#", "####..#", ".###.#.", "..##...", "...#...",
];
const HEART_EMPTY: [&str; 6] = [
    ".##.##.", "#..#..#", "#.....#", ".#...#.", "..#.#..", "...#...",
];

impl ProgressStyle for HeartContainers {
    fn name(&self) -> &str {
        "hearts"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Link's Awakening HUD: B/A item boxes with shield and sword beside two rows of seven heart containers filling half a heart at a time; the next heart blinks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;

        // Item boxes: B (shield) and A (sword).
        if hi >= 14 && w >= 40 {
            draw::rect_outline(grid, 0, 1, 12, 14);
            draw::rect_outline(grid, 14, 1, 12, 14);
            sprite(
                grid,
                3,
                4,
                &["######", "#.##.#", "#.##.#", "######", ".####.", "..##.."],
            );
            sprite(
                grid,
                17,
                3,
                &[
                    "......#", ".....#.", "....#..", "#..#...", ".##....", ".##....", "#..#...",
                ],
            );
        }
        let x0 = if w >= 40 { 30 } else { 0 };

        // Fourteen containers, two rows of seven, in half-heart steps.
        let halves = (ctx.eased * 28.0).round() as i32;
        let blink = (ctx.time * 3.0).fract() < 0.5;
        for i in 0..14 {
            let (col, row) = (i % 7, i / 7);
            let x = x0 + col * 8;
            let y = if hi >= 14 { 1 + row * 8 } else { row * 8 };
            let left = halves - i * 2;
            let art = if left >= 2 {
                &HEART_FULL
            } else if left == 1 {
                &HEART_HALF
            } else if left == 0 && blink {
                continue; // the next heart to fill blinks out
            } else {
                &HEART_EMPTY
            };
            sprite(grid, x, y, art);
        }
        Ok(())
    }
}

// ─── 9. Mole Whack ───────────────────────────────────────────────────────────

/// A row of dirt holes; moles (round heads, beady eyes, snout) bob up and
/// down, and a red mallet winds up and slams the next one. Whacked moles stay
/// half-sunk with dizzy stars circling — the whacked count is the progress.
struct MoleWhack;

/// Mole head, top to bottom: crown, eyes, snout, shoulders.
const MOLE: [&str; 7] = [
    "..####..", ".######.", "##.##.##", "########", "###..###", "########", "########",
];
/// Mallet: barrel head with a handle sticking out to the right.
const MALLET: [&str; 5] = [
    ".######.........",
    "########........",
    "################",
    "########........",
    ".######.........",
];

impl ProgressStyle for MoleWhack {
    fn name(&self) -> &str {
        "mole-whack"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Whack-a-Mole: moles bob from a row of holes, a red mallet slams the next one; whacked moles see stars — whacked count = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        let n = ((wi - 4) / 14).max(1);
        let spacing = (wi / n).max(1);
        let whacked = ((ctx.eased * n as f32) as i32).min(n);
        let rim = hi - 3;
        // Swing cycle for the mallet on the current target mole.
        let u = (t * 1.1).fract();
        let mut mallet = (i32::MIN / 2, i32::MIN / 2);

        for i in 0..n {
            let hx = i * spacing + spacing / 2;
            // Hole: a dirt lip arc the mole pops out of.
            sprite(
                grid,
                hx - 6,
                rim,
                &["##........##", ".##........##.", "..##########.."],
            );
            // Grass tufts between holes.
            sprite(grid, hx + spacing / 2 - 1, hi - 2, &["#.#", ".#."]);

            let rise = match i.cmp(&whacked) {
                std::cmp::Ordering::Less => 3,
                std::cmp::Ordering::Equal => {
                    if u < 0.45 {
                        (u / 0.45 * 7.0) as i32
                    } else if u < 0.6 {
                        7
                    } else {
                        (7.0 - (u - 0.6) / 0.4 * 7.0) as i32
                    }
                }
                std::cmp::Ordering::Greater => {
                    (tri(t * 0.7 + i as f32 * 0.37) * 10.0 - 3.0).clamp(0.0, 6.0) as i32
                }
            };
            // Mole, clipped so it emerges from inside the hole.
            let top = rim + 1 - rise;
            for (r, row) in MOLE.iter().enumerate() {
                let y = top + r as i32;
                if y > rim {
                    break;
                }
                for (c, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        draw::dot_i(grid, hx - 4 + c as i32, y);
                    }
                }
            }
            if i < whacked {
                // Dizzy stars orbiting the bonked mole.
                for k in 0..2 {
                    let a = t * 6.0 + k as f32 * PI + i as f32;
                    let sx = hx + (a.cos() * 5.0).round() as i32;
                    let sy = top - 3 + (a.sin() * 1.5).round() as i32;
                    sprite(grid, sx - 1, sy - 1, &[".#.", "###", ".#."]);
                }
            } else if i == whacked {
                // Mallet: wound up high and right, slams down at u = 0.6.
                let swing = if u < 0.5 {
                    1.0
                } else if u < 0.6 {
                    1.0 - (u - 0.5) / 0.1
                } else {
                    0.0
                };
                let my = rim - 11 - (swing * 2.0) as i32;
                let mx = hx - 4 + (swing * 7.0) as i32;
                sprite(grid, mx, my, &MALLET);
                mallet = (mx, my);
                if (0.6..0.75).contains(&u) {
                    // Impact burst.
                    for (dx, dy) in [(-7, -2), (-6, 1), (6, -2), (7, 1)] {
                        draw::dot_i(grid, hx + dx, my + 5 + dy);
                        draw::dot_i(grid, hx + dx + dx.signum(), my + 5 + dy);
                    }
                }
            }
        }

        // Colour: red mallet head, tan handle, brown moles, yellow stars,
        // green grass.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let local = (px + spacing * 4) % spacing - spacing / 2;
                let hole = (px / spacing).min(n - 1);
                let color = if (mallet.0 - 1..mallet.0 + 8).contains(&px)
                    && (mallet.1 - 1..mallet.1 + 6).contains(&py)
                {
                    Color::rgb(236, 52, 48)
                } else if (mallet.0 + 8..mallet.0 + 17).contains(&px)
                    && (mallet.1 - 1..mallet.1 + 6).contains(&py)
                {
                    Color::rgb(222, 176, 110)
                } else if local.abs() > 5 {
                    Color::rgb(110, 200, 90) // grass
                } else if hole < whacked && py < rim - 3 {
                    Color::rgb(255, 226, 80) // dizzy stars
                } else if py >= rim {
                    Color::rgb(126, 86, 48) // dirt rim
                } else {
                    Color::rgb(214, 168, 140) // mole
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 10. Wario Treasure ──────────────────────────────────────────────────────

/// Spinning gold coins arc into an open treasure chest on the right; the
/// coin stack beside it grows with progress and the chest overflows at 100%.
struct WarioTreasure;

/// Coin spin frames: face-on, three-quarter, edge-on.
const COIN: [[&str; 5]; 3] = [
    [".###.", "#...#", "#.#.#", "#...#", ".###."],
    ["..#..", ".#.#.", ".#.#.", ".#.#.", "..#.."],
    ["..#..", "..#..", "..#..", "..#..", "..#.."],
];

impl ProgressStyle for WarioTreasure {
    fn name(&self) -> &str {
        "wario-treasure"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Wario Land treasure: spinning gold coins arc into an open treasure chest while the coin stacks beside it grow with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Treasure chest: a banded box with its lid thrown open behind.
        let chx = wi - 18;
        sprite(
            grid,
            chx + 1,
            hi - 13,
            &[".##############", "#..............#", "################"],
        );
        draw::rect_outline(grid, chx.max(0) as usize, (hi - 9).max(0) as usize, 18, 9);
        draw::hline(
            grid,
            chx.max(0) as usize,
            (chx + 17).max(0) as usize,
            (hi - 6).max(0) as usize,
        );
        block(grid, chx + 8, hi - 7, 2, 3); // lock

        // Coin stacks to the left, one coin per step of progress.
        let stacks = ((chx - 4) / 6).max(0);
        let coins = (ctx.eased * (stacks * 3) as f32).round() as i32;
        for i in 0..coins {
            let (col, lvl) = (i / 3, i % 3);
            sprite(grid, col * 6, hi - 3 - lvl * 3, &["####", "#..#", "####"]);
        }

        // Coins in flight, spinning as they arc into the chest.
        for k in 0..2 {
            let t = (ctx.time * 0.8 + k as f32 * 0.5).fract();
            let x0 = (stacks * 6).min(chx - 6) as f32;
            let x = x0 + t * (chx as f32 + 6.0 - x0);
            let y = (hi - 9) as f32 - (t * PI).sin() * (hi as f32 - 6.0);
            let frame = ((ctx.time * 8.0) as i32 + k).rem_euclid(4);
            let art = &COIN[[0, 1, 2, 1][frame as usize]];
            sprite(grid, x as i32 - 2, y as i32 - 2, art);
        }

        // At 100% the chest overflows with gold.
        if ctx.eased >= 1.0 {
            sprite(grid, chx + 3, hi - 11, &["#.#.#.#.#.#", ".#.#.#.#.#."]);
        }
        Ok(())
    }
}

// ─── 11. LCD Pinball ─────────────────────────────────────────────────────────

/// A sideways LCD pinball table: rails top and bottom, a coiled plunger on
/// the left, six round pop bumpers (lit ones solid) and a pair of flippers
/// guarding the drain on the right, with the ball ricocheting across.
struct LcdPinball;
impl ProgressStyle for LcdPinball {
    fn name(&self) -> &str {
        "lcd-pinball"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "LCD pinball table: plunger, six round pop bumpers and twin flippers; bumpers light up solid as progress rises while the ball ricochets with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Rails and the plunger spring.
        draw::hline(grid, 0, w - 1, 0);
        draw::hline(grid, 0, w - 1, h - 1);
        for y in 3..hi - 3 {
            draw::dot_i(grid, 1 + (y % 2), y);
        }

        // Pop bumpers in two staggered rows; lit ones are solid discs.
        const RING: [&str; 5] = [".###.", "#...#", "#...#", "#...#", ".###."];
        const LIT: [&str; 5] = [".###.", "#####", "#####", "#####", ".###."];
        let lit = (ctx.eased * 6.0).round() as i32;
        let span = (wi - 26).max(6);
        for k in 0..6 {
            let x = 8 + k * span / 6;
            let y = if k % 2 == 0 { 2 } else { hi - 7 };
            sprite(grid, x, y, if k < lit { &LIT } else { &RING });
        }

        // Flippers: angled bats either side of the drain, flicking up.
        let flick = (ctx.time * 1.3).fract() < 0.18;
        let (fx0, fx1) = (wi - 2, wi - 10);
        let mid = hi / 2;
        let lift = if flick { 3 } else { 0 };
        for i in 0..=8 {
            let t = i as f32 / 8.0;
            let x = fx0 + ((fx1 - fx0) as f32 * t) as i32;
            draw::dot_i(grid, x, 1 + ((mid - 2 + lift - 1) as f32 * t) as i32);
            draw::dot_i(grid, x, hi - 2 - ((mid - 2 + lift - 1) as f32 * t) as i32);
        }

        // The ball ricochets around the table.
        let bx = 5 + (tri(ctx.time * 0.45) * (wi - 18).max(0) as f32) as i32;
        let by = 2 + (tri(ctx.time * 0.83 + 0.2) * (hi - 6).max(0) as f32) as i32;
        block(grid, bx, by, 2, 2);
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
