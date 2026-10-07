//! `fruits` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O fruits.rs && ./fruits [style-name]
//! ```

const DEFAULT_STYLE: &str = "citrus-wheel";

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
    pub mod fruits {
//! Fruits-themed progress bars — eleven distinct fruit styles drawn in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (for perpetual
//! animation) and `ctx.eased` (for progress-driven advancement). All writes
//! go through `draw::` helpers so every coordinate is silently bounds-safe.
//!
//! Styles in this file:
//! - `citrus-wheel`    — orange wheels filling segment by segment
//! - `apple-bite`      — row of red apples eaten bite by bite down to cores
//! - `banana-peel`     — curved banana peeled from the tip, flaps dangling
//! - `grape-bunch`     — swaying grape bunches ripening purple grape by grape
//! - `watermelon-slice`— half-moon slices with rind and seeded red flesh
//! - `strawberry-seeds`— leafy-topped strawberries ripening red from the tip up
//! - `pineapple-lattice`— crosshatched pineapple ripening under a leaf crown
//! - `cherry-swing`    — swinging cherry pairs ripening red pair by pair
//! - `juice-squeeze`   — lemon half squeezed flat over a filling glass
//! - `berry-pop`       — crowned blueberries popping into a row with a bounce
//! - `kiwi-rings`      — kiwi halves with seed rings, cut open one by one

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// Set dot `(x, y)` and tint its cell `c`. Colour is one per cell (2×4 dots),
/// so the last object drawn over a cell decides its colour.
fn px(grid: &mut BrailleGrid, x: i32, y: i32, c: Color) {
    if x >= 0 && y >= 0 {
        draw::dot_i(grid, x, y);
        let (cx, cy) = (x as usize / 2, y as usize / 4);
        draw::tint_row(grid, cy, cx, cx, c);
    }
}

/// All styles in the `fruits` theme.
///
/// Returns eleven [`ProgressStyle`] implementations, each using a distinct
/// geometric or motion strategy so no two differ by color alone.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(CitrusWheel),
        Box::new(AppleBite),
        Box::new(BananaPeel),
        Box::new(GrapeBunch),
        Box::new(WatermelonSlice),
        Box::new(StrawberrySeeds),
        Box::new(PineappleLattice),
        Box::new(CherrySwing),
        Box::new(JuiceSqueeze),
        Box::new(BerryPop),
        Box::new(KiwiRings),
    ]
}

// ── CitrusWheel ───────────────────────────────────────────────────────────────
// A row of orange wheels seen face-on: an orange rind ring and eight juicy
// segments split by gap membranes radiating from the centre. Segments
// fill in clockwise, wheel after wheel, with eased; the wheels slowly turn.

struct CitrusWheel;
impl ProgressStyle for CitrusWheel {
    fn name(&self) -> &str {
        "citrus-wheel"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of orange wheels — rind and eight juicy segments — filling segment by segment"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 1) / 2).clamp(1, 7);
        let cy = hi / 2;
        let n = (wi / (2 * r + 6)).max(1);
        let pitch = wi / n;
        let rind = Color::rgb(255, 125, 0);
        let pulp = Color::rgb(255, 205, 90);
        let rf = r as f32;
        let segs = ctx.eased * (n * 8) as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let spin = ctx.time * 0.5 + i as f32;
            let filled = (segs - (i * 8) as f32).clamp(0.0, 8.0);
            for dy in -r..=r {
                for dx in -r..=r {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d > rf + 0.3 {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if d > rf - 1.0 {
                        px(grid, x, y, rind);
                        continue;
                    }
                    // Clockwise angle from the top, in segment units 0..8.
                    let a = ((dx as f32).atan2(-dy as f32) + spin).rem_euclid(2.0 * PI);
                    let s = a * 8.0 / (2.0 * PI);
                    let membrane = s.fract() < 0.25 || d < 1.5 || d > rf - 1.6;
                    if s < filled && !membrane {
                        px(grid, x, y, pulp);
                    }
                }
            }
        }
        Ok(())
    }
}

// ── AppleBite ────────────────────────────────────────────────────────────────
// A row of red apples (two overlapping lobes, brown stem, green leaf) eaten
// left to right. The apple being eaten loses up to three bites and wobbles;
// finished apples are left as cream cores with their stems.

struct AppleBite;
impl ProgressStyle for AppleBite {
    fn name(&self) -> &str {
        "apple-bite"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of red apples with stems and leaves, eaten bite by bite down to their cores"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 4) / 2).clamp(1, 6);
        let cy = hi - 1 - r;
        let n = (wi / (2 * r + 8)).max(1);
        let pitch = wi / n;
        let red = Color::rgb(225, 40, 50);
        let flesh = Color::rgb(240, 225, 170);
        let stem = Color::rgb(150, 95, 45);
        let leaf = Color::rgb(80, 200, 70);
        let eaten = ctx.eased * n as f32;
        let r2 = r * r + r / 2;
        let rb = r / 2 + 1;
        for i in 0..n {
            let cx = (pitch * i + pitch / 2) & !1;
            let f = (eaten - i as f32).clamp(0.0, 1.0);
            if f >= 1.0 {
                // Core: narrow waist with flared ends and two seed gaps.
                for dy in -r..=r {
                    let hw = if dy.abs() >= r - 1 {
                        3
                    } else if dy.abs() >= r - 2 {
                        2
                    } else {
                        1
                    };
                    for dx in -hw..=hw {
                        if dx == 0 && dy.abs() == 1 {
                            continue;
                        }
                        px(grid, cx + dx, cy + dy, flesh);
                    }
                }
            } else {
                let wob = if f > 0.0 {
                    ((ctx.time * 9.0).sin() * 0.9).round() as i32
                } else {
                    0
                };
                // Bites appear at f > 0, 1/3, 2/3: right, left, right-low.
                let bites = [
                    (cx + r + 2, cy - r / 3, 0.0),
                    (cx - r - 2, cy + r / 4, 0.34),
                    (cx + r + 1, cy + r / 2 + 1, 0.67),
                ];
                for dy in -r..=r {
                    for dx in -(r + 2)..=(r + 2) {
                        let lobe = (dx + 2) * (dx + 2) + dy * dy <= r2
                            || (dx - 2) * (dx - 2) + dy * dy <= r2;
                        if !lobe {
                            continue;
                        }
                        let (x, y) = (cx + dx + wob, cy + dy);
                        let mut c = red;
                        let mut gone = false;
                        for &(bx, by, at) in &bites {
                            if f > at {
                                let d = (x - bx - wob) * (x - bx - wob) + (y - by) * (y - by);
                                if d <= rb * rb {
                                    gone = true;
                                } else if d <= (rb + 2) * (rb + 2) {
                                    c = flesh;
                                }
                            }
                        }
                        if !gone {
                            px(grid, x, y, c);
                        }
                    }
                }
                // Leaf on the right of the stem.
                for &(lx, ly) in &[(1, -2), (2, -3), (3, -3), (2, -2), (3, -2), (4, -3)] {
                    px(grid, cx + lx + wob, cy - r + ly, leaf);
                }
            }
            // Stem sits on every apple and core.
            draw::vline(
                grid,
                cx.max(0) as usize,
                (cy - r - 2).max(0) as usize,
                (cy - r).max(0) as usize,
            );
            draw::tint_row(
                grid,
                ((cy - r - 2).max(0) / 4) as usize,
                (cx / 2) as usize,
                (cx / 2) as usize,
                stem,
            );
        }
        Ok(())
    }
}

// ── BananaPeel ───────────────────────────────────────────────────────────────
// One long banana lying in a smile-shaped curve across the bar: yellow skin,
// brown stem and tip. Progress peels it from the right: the peeled part shows
// cream fruit, and three peel flaps hang from the peel line, swaying with time.

struct BananaPeel;
impl ProgressStyle for BananaPeel {
    fn name(&self) -> &str {
        "banana-peel"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Curved yellow banana peeled from the tip as progress grows; peel flaps dangle and sway"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let skin = Color::rgb(250, 215, 50);
        let fruit = Color::rgb(250, 240, 200);
        let brown = Color::rgb(140, 90, 40);
        let mid = (wi - 1) as f32 / 2.0;
        let half = (wi as f32 / 2.0 - 2.0).max(1.0);
        let hf = hi as f32;
        // Centreline sags in the middle; thickness tapers toward both ends.
        let centre = |x: i32| {
            let u = (x as f32 - mid) / half;
            (hf * 0.62 - hf * 0.42 * u * u, u)
        };
        let peel_x = wi as f32 - 2.0 - ctx.eased * (wi as f32 - 6.0);
        for x in 0..wi {
            let (yc, u) = centre(x);
            if u.abs() > 1.0 {
                continue;
            }
            let thick = (hf * 0.36 * (1.0 - u * u).max(0.0).sqrt()).max(1.0);
            let peeled = x as f32 > peel_x && u < 0.93;
            let ys = (yc - thick / 2.0).round() as i32;
            let ye = (yc + thick / 2.0).round() as i32;
            for y in ys..=ye {
                let c = if u.abs() > 0.93 {
                    brown
                } else if peeled {
                    fruit
                } else {
                    skin
                };
                // A ridge line of negative space runs along the skin.
                if !peeled && y == ys + 1 && thick > 3.0 && x % 3 != 0 {
                    continue;
                }
                if peeled && (y == ys || y == ye) && thick > 2.5 {
                    continue;
                }
                px(grid, x, y, c);
            }
        }
        // Peel flaps hang from the peel line once peeling has started.
        if ctx.eased > 0.02 {
            let x = peel_x.round() as i32;
            let (yc, _) = centre(x);
            for k in -1..=1 {
                let sway = (ctx.time * 2.5 + k as f32).sin() * 1.5;
                let y0 = yc.round() as i32;
                for y in y0..hi {
                    let t = (y - y0) as f32 / hf;
                    let fx = x + (k as f32 * (2.0 + t * 8.0) + sway * t * 2.0).round() as i32;
                    px(grid, fx, y, skin);
                    if y + 1 == hi {
                        px(grid, fx, y, brown);
                    }
                }
            }
        }
        Ok(())
    }
}

// ── GrapeBunch ───────────────────────────────────────────────────────────────
// Hanging grape bunches: rows of 3-2-1 round grapes under a brown stem and a
// green leaf. Grapes ripen (fill purple, with a glossy highlight) one at a
// time with eased; unripe grapes are dim rings. Bunches sway with time, the
// lower rows swinging further than the top.

struct GrapeBunch;
impl ProgressStyle for GrapeBunch {
    fn name(&self) -> &str {
        "grape-bunch"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Swaying grape bunches on stems; grapes ripen purple one by one as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let purple = [Color::rgb(130, 60, 200), Color::rgb(165, 85, 225)];
        let dim = Color::rgb(85, 85, 110);
        let stem = Color::rgb(150, 100, 50);
        let leaf = Color::rgb(70, 190, 70);
        let total = n * 6;
        let ripe = ctx.eased * total as f32;
        let mut g = 0;
        for i in 0..n {
            let bx = pitch * i + pitch / 2;
            let sway = (ctx.time * 1.8 + i as f32 * 1.3).sin();
            // Collect grape centres; later (lower) grapes overlap earlier ones,
            // and a one-dot gap ring around each keeps the grapes distinct.
            let mut grapes = [(0i32, 0i32, false); 6];
            let mut m = 0;
            for (j, count) in [3, 2, 1].iter().enumerate() {
                let gy = 5 + j as i32 * 4;
                let off = (sway * (j as f32 + 1.0) * 0.5).round() as i32;
                for k in 0..*count {
                    let gx = bx - (count - 1) * 3 + k * 6 + off;
                    grapes[m] = (gx, gy, (g as f32) < ripe.floor());
                    m += 1;
                    g += 1;
                }
            }
            for y in 2..hi {
                for x in (bx - 9)..=(bx + 9) {
                    let d = |q: usize| {
                        let (gx, gy, _) = grapes[q];
                        (x - gx) * (x - gx) + (y - gy) * (y - gy)
                    };
                    let Some(o) = (0..6).rev().find(|&q| d(q) <= 5) else {
                        continue;
                    };
                    if (o + 1..6).any(|q| d(q) <= 8) {
                        continue;
                    }
                    let (gx, gy, full) = grapes[o];
                    if full && !(x - gx == -1 && y - gy == -1) {
                        px(grid, x, y, purple[o % 2]);
                    } else if !full && d(o) >= 4 {
                        px(grid, x, y, dim);
                    }
                }
            }
            // Stem and leaf, drawn last so their cells keep their colours.
            for y in 0..3 {
                px(grid, bx, y, stem);
            }
            for &(lx, ly) in &[(2, 0), (3, 0), (4, 0), (5, 1), (3, 1), (4, 1), (2, 1)] {
                px(grid, bx + lx, ly, leaf);
            }
        }
        Ok(())
    }
}

// ── WatermelonSlice ───────────────────────────────────────────────────────────
// A row of half-moon watermelon slices, flat side up: green rind around the
// curve, a pale band, then red flesh with black-seed gaps. Each slice fills
// with flesh from the rind upward in turn; the juicy surface ripples with time.

struct WatermelonSlice;
impl ProgressStyle for WatermelonSlice {
    fn name(&self) -> &str {
        "watermelon-slice"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Half-moon watermelon slices with green rind and seeded red flesh, filling one by one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = (hi - 3).clamp(2, 12);
        let y0 = hi - 1 - r;
        let n = (wi / (2 * r + 2)).max(1);
        let pitch = wi / n;
        let rind = Color::rgb(40, 170, 70);
        let flesh = Color::rgb(240, 50, 70);
        let pale = Color::rgb(210, 240, 190);
        let r2 = r * r;
        let inner = (r - 2) * (r - 2);
        let flesh_r = (r - 3) * (r - 3);
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            for dy in 0..=r {
                // Surface level (rows from the rind upward), rippling.
                let ripple = (ctx.time * 4.0 + dy as f32).sin() * 0.6;
                for dx in -r..=r {
                    let d = dx * dx + dy * dy;
                    if d > r2 {
                        continue;
                    }
                    let (x, y) = (cx + dx, y0 + dy);
                    if d > inner {
                        px(grid, x, y, rind);
                    } else if d > flesh_r {
                        if dy == 0 {
                            px(grid, x, y, pale);
                        }
                    } else {
                        let depth = r as f32 - (d as f32).sqrt();
                        let filled = depth <= f * (r as f32 - 1.0) + ripple;
                        let seed = (dy % 4 == 2) && (dx + dy).rem_euclid(5) == 0 && dy < r - 4;
                        if f > 0.0 && filled && !seed {
                            px(grid, x, y, flesh);
                        } else if dy == 0 {
                            px(grid, x, y, pale);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ── StrawberrySeeds ───────────────────────────────────────────────────────────
// A row of strawberries: heart-shouldered bodies tapering to a point, with a
// spiky green calyx and stem. Each berry ripens from its tip upward with
// eased, turning from a pale outline into solid red dotted with seed gaps.
// Calyx tips flutter with time.

struct StrawberrySeeds;
impl ProgressStyle for StrawberrySeeds {
    fn name(&self) -> &str {
        "strawberry-seeds"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of strawberries with leafy green tops, ripening red and seeded from tip to shoulder"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bh = (hi - 4).clamp(2, 12);
        let y0 = hi - bh;
        let n = (wi / 18).max(1);
        let pitch = wi / n;
        let red = Color::rgb(230, 35, 60);
        let pale = Color::rgb(195, 225, 165);
        let leaf = Color::rgb(70, 185, 60);
        let ripe = ctx.eased * n as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let f = (ripe - i as f32).clamp(0.0, 1.0);
            for row in 0..bh {
                let t = row as f32 / bh as f32;
                let mut hw = 6.5 * (1.0 - t).sqrt();
                if row == 0 {
                    hw -= 1.5;
                } else if row == 1 {
                    hw -= 0.5;
                }
                let hw = hw.round().max(0.0) as i32;
                let y = y0 + row;
                let ripe_row = t >= 1.0 - f;
                for dx in -hw..=hw {
                    let edge = dx.abs() == hw || row == 0 || row == bh - 1;
                    if ripe_row {
                        let seed = row % 3 == 1 && (dx + (row / 3 % 2) * 2).rem_euclid(4) == 0;
                        if edge || !seed {
                            px(grid, cx + dx, y, red);
                        }
                    } else if edge {
                        px(grid, cx + dx, y, pale);
                    }
                }
            }
            // Calyx: a band of leaves with fluttering tips, plus a short stem.
            let fl = ((ctx.time * 3.0 + i as f32).sin() * 1.2).round() as i32;
            draw::hline(
                grid,
                (cx - 4).max(0) as usize,
                (cx + 4).max(0) as usize,
                (y0 - 1).max(0) as usize,
            );
            for &tx in &[-5, -2, 2, 5] {
                px(grid, cx + tx + fl * tx.signum(), y0 - 2, leaf);
            }
            px(grid, cx, y0 - 2, leaf);
            px(grid, cx, y0 - 3, leaf);
            px(grid, cx + 1, y0 - 4, leaf);
            draw::tint_row(
                grid,
                ((y0 - 1).max(0) / 4) as usize,
                ((cx - 5).max(0) / 2) as usize,
                ((cx + 5).max(0) / 2) as usize,
                leaf,
            );
        }
        Ok(())
    }
}

// ── PineappleLattice ──────────────────────────────────────────────────────────
// A pineapple lying on its side: an oval body with a leaf crown fanning out to
// the right. Ripe (filled) rind is solid gold cut by diagonal grooves, giving
// the crosshatched diamond skin; unripe rind is an outline with lattice nodes.
// The ripe region sweeps left to right with eased; the crown sways with time.

struct PineappleLattice;
impl ProgressStyle for PineappleLattice {
    fn name(&self) -> &str {
        "pineapple-lattice"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Pineapple on its side: crosshatched gold skin ripens left to right under a swaying leaf crown"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let crown_len = (wi / 4).clamp(2, 22);
        let x0 = 1;
        let x1 = (wi - crown_len - 1).max(x0 + 2);
        let cx = (x0 + x1) as f32 / 2.0;
        let rx = ((x1 - x0) as f32 / 2.0).max(1.0);
        let cy = (hi - 1) as f32 / 2.0;
        let ry = (hi as f32 / 2.0 - 0.5).max(1.0);
        let inside = |x: i32, y: i32| {
            let nx = (x as f32 - cx) / rx;
            let ny = (y as f32 - cy) / ry;
            nx * nx + ny * ny <= 1.0
        };
        let fill_x = x0 as f32 + ctx.eased * (x1 - x0 + 1) as f32;
        let gold = Color::rgb(245, 185, 40);
        let green = Color::rgb(120, 150, 60);
        let leaf = Color::rgb(60, 195, 80);
        for y in 0..hi {
            for x in x0..=x1 {
                if !inside(x, y) {
                    continue;
                }
                let edge = !inside(x - 1, y)
                    || !inside(x + 1, y)
                    || !inside(x, y - 1)
                    || !inside(x, y + 1);
                let a = (x + y).rem_euclid(6);
                let b = (x - y).rem_euclid(6);
                if (x as f32) < fill_x {
                    // Ripe: solid skin, diagonal grooves left as negative space.
                    if edge || (a != 0 && b != 0) {
                        px(grid, x, y, gold);
                    }
                } else if edge || (a == 0 && b == 0) {
                    px(grid, x, y, green);
                }
            }
        }
        // Leaf crown: five blades fanning from the stem end, each swaying.
        let base_y = cy.round() as i32;
        for k in 0..5 {
            let a = (k as f32 - 2.0) * 0.42 + (ctx.time * 2.2 + k as f32).sin() * 0.1;
            let len = crown_len as f32 * (1.0 - (k as f32 - 2.0).abs() * 0.18);
            let steps = len.max(1.0) as i32;
            for s in 0..=steps {
                let lx = x1 - 1 + (s as f32 * a.cos()).round() as i32;
                let ly = base_y + (s as f32 * a.sin()).round() as i32;
                px(grid, lx, ly, leaf);
                if s < steps / 2 {
                    px(grid, lx, ly + 1, leaf);
                }
            }
        }
        Ok(())
    }
}

// ── CherrySwing ───────────────────────────────────────────────────────────────
// A row of cherry pairs: two round cherries on stems that meet at a leafy top,
// each pair swinging like a pendulum with time. Pairs ripen left to right with
// eased: solid red with a glint once ripe, pale-green rings before.

struct CherrySwing;
impl ProgressStyle for CherrySwing {
    fn name(&self) -> &str {
        "cherry-swing"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of swinging cherry pairs on joined stems, ripening red one pair at a time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 6) / 2).clamp(1, 4);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let red = Color::rgb(215, 20, 50);
        let unripe = Color::rgb(170, 215, 120);
        let stem = Color::rgb(90, 160, 60);
        let leaf = Color::rgb(60, 200, 80);
        let ripe = ctx.eased * n as f32;
        let cy = hi - 1 - r;
        for i in 0..n {
            let top_x = pitch * i + pitch / 2;
            let swing = (ctx.time * 2.0 + i as f32 * 0.9).sin() * 2.0;
            let f = (ripe - i as f32).clamp(0.0, 1.0);
            for side in [-1, 1] {
                let cx = top_x + side * (r + 1) + swing.round() as i32;
                // Stem: a curve from the shared top to the cherry.
                let steps = cy - r;
                for s in 0..steps {
                    let t = s as f32 / steps.max(1) as f32;
                    let sx = top_x as f32 + (cx - top_x) as f32 * t.sqrt();
                    px(grid, sx.round() as i32, s, stem);
                }
                for dy in -r..=r {
                    for dx in -r..=r {
                        let d = dx * dx + dy * dy;
                        if d > r * r + 1 {
                            continue;
                        }
                        let (x, y) = (cx + dx, cy + dy);
                        if f >= 1.0 || (f > 0.0 && side < 0) {
                            if !(dx == -r / 2 && dy == -r / 2) {
                                px(grid, x, y, red);
                            }
                        } else if d > (r - 1) * (r - 1) {
                            px(grid, x, y, unripe);
                        }
                    }
                }
            }
            for &(lx, ly) in &[(1, 0), (2, 0), (3, 1), (2, 1), (4, 1)] {
                px(grid, top_x + lx, ly, leaf);
            }
        }
        Ok(())
    }
}

// ── JuiceSqueeze ──────────────────────────────────────────────────────────────
// A lemon half is squeezed over a glass: the lemon flattens as progress rises,
// juice drips fall from it with time, and the glass fills with yellow juice.
// Spent lemon rinds pile up beside the glass as the squeezing goes on.

struct JuiceSqueeze;
impl ProgressStyle for JuiceSqueeze {
    fn name(&self) -> &str {
        "juice-squeeze"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Lemon half squeezed flat over a glass that fills with juice; drips fall and spent rinds pile up"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let lemon = Color::rgb(240, 235, 60);
        let pith = Color::rgb(255, 250, 205);
        let juice = Color::rgb(255, 170, 30);
        let glass = Color::rgb(175, 205, 225);
        let p = ctx.eased;
        // Glass in the middle: walls and base; juice rises with progress.
        let gw = 22;
        let x0 = (wi / 2 - gw / 2) | 1;
        let x1 = x0 + gw + 1;
        let gtop = hi - 9;
        for y in gtop..hi {
            px(grid, x0, y, glass);
            px(grid, x1, y, glass);
        }
        for x in x0..=x1 {
            px(grid, x, hi - 1, glass);
        }
        let level = (p * (hi - 1 - gtop) as f32).round() as i32;
        for y in (hi - 1 - level..hi - 1).rev() {
            for x in x0 + 1..x1 {
                px(grid, x, y, juice);
            }
        }
        // Lemon half, cut face down, flattening as it is squeezed.
        let cx = (x0 + x1) / 2;
        let lh = (5.0 - p * 3.0).round() as i32;
        let lw = 9 + (p * 3.0) as i32;
        let base = gtop - 2;
        for dy in 0..=lh {
            let t = dy as f32 / lh.max(1) as f32;
            let hw = (lw as f32 * (1.0 - t * t).sqrt()).round() as i32;
            let c = if dy == 0 { pith } else { lemon };
            for dx in -hw..=hw {
                px(grid, cx + dx, base - dy, c);
            }
        }
        // Drips falling from the cut face into the glass.
        if p > 0.0 && p < 1.0 {
            let surface = hi - 2 - level;
            for k in 0..2 {
                let ph = (ctx.time * 1.8 + k as f32 * 0.5).fract();
                let dy = (ph * (surface - base).max(1) as f32) as i32;
                px(grid, cx - 2 + k * 4, base + 1 + dy, juice);
            }
        }
        // Spent rinds pile up on the left: small flattened domes.
        let spent = (p * 4.0) as i32;
        for k in 0..spent {
            let rx = x0 - 8 - k * 9;
            for dy in 0..2 {
                for dx in -3 + dy..=3 - dy {
                    px(grid, rx + dx, hi - 1 - dy, lemon);
                }
            }
        }
        Ok(())
    }
}

// ── BerryPop ──────────────────────────────────────────────────────────────────
// Blueberries pop into a row one by one with eased: each is a round blue berry
// with the star-shaped crown gap on top and a pale glint. The newest berry
// drops in and bounces with time; empty spots wait as dim rings.

struct BerryPop;
impl ProgressStyle for BerryPop {
    fn name(&self) -> &str {
        "berry-pop"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Crowned blueberries pop into a row one by one, the newest bouncing as it lands"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 2) / 2).clamp(1, 5);
        let n = (wi / (2 * r + 2)).max(1);
        let pitch = wi / n;
        let blue = Color::rgb(70, 95, 215);
        let crown = Color::rgb(45, 55, 130);
        let dim = Color::rgb(70, 75, 100);
        let popped = ctx.eased * n as f32;
        let rf = r as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let f = (popped - i as f32).clamp(0.0, 1.0);
            let newest = f > 0.0 && (popped - i as f32) < 1.5;
            // The newest berry bounces; others rest on the floor.
            let lift = if newest {
                ((ctx.time * 5.0 + i as f32).sin().abs() * (hi - 2 * r - 1) as f32) as i32
            } else {
                0
            };
            let cy = hi - 1 - r - lift;
            for dy in -r..=r {
                for dx in -r..=r {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d > rf + 0.3 {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if f <= 0.0 {
                        if d > rf - 0.8 {
                            px(grid, x, y, dim);
                        }
                        continue;
                    }
                    // Crown: an X-shaped gap just below the top of the berry.
                    let star = dy <= -r + 3 && dy > -r && (dx.abs() == (dy + r - 2).abs());
                    let glint = dx == r / 2 && dy == 0;
                    if star || glint {
                        continue;
                    }
                    let c = if dy <= -r + 3 && dx.abs() <= 2 {
                        crown
                    } else {
                        blue
                    };
                    px(grid, x, y, c);
                }
            }
        }
        Ok(())
    }
}

// ── KiwiRings ────────────────────────────────────────────────────────────────
// A row of kiwi halves seen face-on: fuzzy brown skin, green flesh, a ring of
// black seeds (gaps) around a cream core. Halves are cut one by one with
// eased (uncut ones show only the skin outline); the seed ring slowly turns.

struct KiwiRings;
impl ProgressStyle for KiwiRings {
    fn name(&self) -> &str {
        "kiwi-rings"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of kiwi halves — brown skin, green flesh, seed ring and pale core — cut open one by one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 1) / 2).clamp(1, 7);
        let cy = hi / 2;
        let n = (wi / (2 * r + 4)).max(1);
        let pitch = wi / n;
        let skin = Color::rgb(140, 95, 50);
        let flesh = Color::rgb(120, 200, 60);
        let core = Color::rgb(235, 240, 200);
        let rf = r as f32;
        let cut = ctx.eased * n as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let open = cut > i as f32 + 0.5;
            let spin = ctx.time * 0.6 + i as f32;
            for (pass, dy) in (0..2).flat_map(|p| (-r..=r).map(move |dy| (p, dy))) {
                for dx in -r..=r {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d > rf + 0.3 {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if d > rf - 1.0 {
                        // Skin is drawn in a first pass so flesh owns shared cells.
                        if pass == 0 {
                            px(grid, x, y, skin);
                        }
                    } else if !open || pass == 0 {
                        // Uncut halves show only their skin.
                    } else if d < rf * 0.3 {
                        px(grid, x, y, core);
                    } else {
                        // Seed ring: gaps at regular angles around mid-radius.
                        let a = (dy as f32).atan2(dx as f32) + spin;
                        let ring = (d - rf * 0.5).abs() < 0.9;
                        let seed = ring && (a * 12.0 / (2.0 * PI)).rem_euclid(1.0) < 0.5;
                        if !seed {
                            px(grid, x, y, flesh);
                        }
                    }
                }
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
    let styles = progress::styles::fruits::styles();
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
