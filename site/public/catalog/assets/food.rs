//! `food` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O food.rs && ./food [style-name]
//! ```

const DEFAULT_STYLE: &str = "beer-glass";

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
    pub mod food {
//! Food / kitchen progress bars — a full set of animated, braille-rendered
//! loading styles themed around cooking, eating, and the kitchen.
//!
//! Every bar is stateless: it is a pure function of `(ctx.eased, ctx.time)`.
//! Bubbles rise, steam drifts, conveyor plates slide, and popcorn pops — all
//! driven by `ctx.time` so the bars stay alive at any fixed progress value.

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

// ---------------------------------------------------------------------------
// Deterministic hash — gives a pseudo-random u32 from any u32 seed.
// ---------------------------------------------------------------------------
#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Glass beer mugs with handles, filling amber one after another under a
/// white foam head; bubbles rise through the beer over time.
struct BeerGlass;
impl ProgressStyle for BeerGlass {
    fn name(&self) -> &str {
        "beer-glass"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Row of glass beer mugs with handles filling amber in turn; foam heads and rising bubbles"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 6 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 28).max(1);
        let pitch = wi / n;
        let bw = ((pitch - 10).clamp(4, 18)) & !1;
        let glass = Color::rgb(175, 200, 220);
        let beer = Color::rgb(240, 165, 30);
        let foam = Color::rgb(250, 248, 235);
        let ih = hi - 2;
        for i in 0..n {
            let x0 = (pitch * i + (pitch - bw - 6) / 2) | 1;
            let x1 = x0 + bw + 1;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            // Glass: walls, base, and a D-shaped handle on the right.
            for y in 1..hi {
                px(grid, x0, y, glass);
                px(grid, x1, y, glass);
            }
            for x in x0..=x1 {
                px(grid, x, hi - 1, glass);
            }
            let (hy0, hy1) = (4.min(hi - 2), (hi - 4).max(4));
            for x in x1..=x1 + 4 {
                px(grid, x, hy0, glass);
                px(grid, x, hy1, glass);
            }
            for y in hy0..=hy1 {
                px(grid, x1 + 4, y, glass);
            }
            // Beer from the bottom up; the top two rows are foam.
            let level = (f * ih as f32).round() as i32;
            if level <= 0 {
                continue;
            }
            let top = hi - 1 - level;
            // Bottom-up, so the foam owns the colour of the cell it shares.
            for y in (top..hi - 1).rev() {
                for x in x0 + 1..x1 {
                    let foamy = y < top + 2 && level > 2;
                    // Bubbles: negative-space dots rising on fixed columns.
                    let col = (x - x0 - 1) % 5 == (i % 5) + 1;
                    let rise = ((ctx.time * 5.0) as i32 + x * 3) % level.max(1);
                    if !foamy && col && y == hi - 2 - rise {
                        continue;
                    }
                    px(grid, x, y, if foamy { foam } else { beer });
                }
            }
            // A full mug's head domes up over the rim.
            if f >= 1.0 {
                for x in x0..=x1 {
                    if (x + (ctx.time * 4.0) as i32) % 3 != 0 {
                        px(grid, x, 0, foam);
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// A coffee pot travels along a row of mugs, pouring a stream into each in
/// turn; filled mugs steam.
struct CoffeePour;
impl ProgressStyle for CoffeePour {
    fn name(&self) -> &str {
        "coffee-pour"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Coffee pot pours a wavering stream into a row of mugs one by one; full mugs steam"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let mug = Color::rgb(235, 235, 240);
        let coffee = Color::rgb(130, 75, 35);
        let crema = Color::rgb(205, 150, 90);
        let pot = Color::rgb(150, 160, 175);
        let steam = Color::rgb(170, 170, 185);
        let top = hi - 8;
        let pour = ctx.eased * n as f32;
        let current = (pour as i32).min(n - 1);
        for i in 0..n {
            let x0 = (pitch * i + (pitch - 16) / 2) | 1;
            let x1 = x0 + 11;
            let f = (pour - i as f32).clamp(0.0, 1.0);
            // Mug: walls, base, and a handle on the right.
            for y in top..hi {
                px(grid, x0, y, mug);
                px(grid, x1, y, mug);
            }
            for x in x0..=x1 {
                px(grid, x, hi - 1, mug);
            }
            for &(dx, dy) in &[(1, 1), (2, 1), (3, 2), (3, 3), (3, 4), (2, 5), (1, 5)] {
                px(grid, x1 + dx, top + dy, mug);
            }
            // Coffee level, crema on top.
            let level = (f * (hi - 1 - top) as f32).round() as i32;
            for y in (hi - 1 - level..hi - 1).rev() {
                for x in x0 + 1..x1 {
                    let c = if y == hi - 1 - level { crema } else { coffee };
                    px(grid, x, y, c);
                }
            }
            // Full mugs give off two curling wisps of steam.
            if f >= 1.0 && i != current {
                for k in 0..2 {
                    for y in 1..top - 1 {
                        let sx = x0
                            + 3
                            + k * 5
                            + ((ctx.time * 3.0 + y as f32 * 0.9 + k as f32 * 2.0).sin() * 1.3)
                                .round() as i32;
                        if (y + (ctx.time * 6.0) as i32 + k) % 3 != 0 {
                            px(grid, sx, y, steam);
                        }
                    }
                }
            }
        }
        // The pot hovers above the mug being filled, tipped toward its spout.
        if ctx.eased < 1.0 {
            let x0 = (pitch * current + (pitch - 16) / 2) | 1;
            let sx = x0 + 6;
            let bx = sx - 8;
            for y in 0..4 {
                for x in bx + y / 2..bx + 7 - y / 2 {
                    px(grid, x, y, pot);
                }
            }
            px(grid, bx - 1, 1, pot);
            px(grid, bx - 1, 2, pot);
            px(grid, bx + 7, 2, pot);
            px(grid, sx - 1, 3, pot);
            // Wavering stream from the spout down to the coffee surface.
            let f = (pour - current as f32).clamp(0.0, 1.0);
            let surface = hi - 1 - (f * (hi - 1 - top) as f32).round() as i32;
            for y in 4..surface {
                let wob = ((ctx.time * 12.0 + y as f32).sin() * 0.6).round() as i32;
                px(grid, sx + wob, y, coffee);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pizza slices served one by one: crust along the top, cheese tapering to a
/// point, pepperoni; the slice being served drips a stretchy cheese string.
struct PizzaSlices;
impl ProgressStyle for PizzaSlices {
    fn name(&self) -> &str {
        "pizza-slices"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Row of pepperoni pizza slices served in turn; the newest one stretches a swaying cheese string"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 6 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 18).max(1);
        let pitch = wi / n;
        let crust = Color::rgb(200, 130, 60);
        let cheese = Color::rgb(255, 210, 80);
        let pep_c = Color::rgb(210, 40, 40);
        let dim = Color::rgb(90, 85, 80);
        let sh = (hi - 2).max(3);
        let served = ctx.eased * n as f32;
        for i in 0..n {
            let cx = (pitch * i + pitch / 2) & !1;
            let f = (served - i as f32).clamp(0.0, 1.0);
            // Rows filled from the crust down for the slice being served.
            let rows = (f * sh as f32).ceil() as i32;
            for row in 0..sh {
                let hw = ((sh - row) as f32 * 7.0 / sh as f32).round() as i32;
                let y = row;
                for dx in -hw..=hw {
                    let x = cx + dx;
                    let edge = dx.abs() == hw;
                    if row < rows {
                        // Crust fills the first cell row; two cell-aligned
                        // pepperoni rounds sit in the cheese with a gap rim.
                        let pep = matches!((dx, row), (-4..=-3, 5..=6) | (2..=3, 9..=10));
                        let rim = matches!((dx, row), (-4..=-3, 4 | 7) | (2..=3, 8 | 11));
                        let c = if row < 4 {
                            crust
                        } else if pep {
                            pep_c
                        } else {
                            cheese
                        };
                        if !rim && (row != 0 || (dx + 30) % 3 != 0) {
                            px(grid, x, y, c);
                        }
                    } else if edge || row == 0 {
                        px(grid, x, y, dim);
                    }
                }
            }
            // Cheese string below the slice being served.
            if f > 0.0 && f < 1.0 {
                let sway = (ctx.time * 3.0 + i as f32).sin();
                let y0 = rows.min(sh);
                for y in y0..hi {
                    let t = (y - y0) as f32 / (hi - y0).max(1) as f32;
                    px(grid, cx + (sway * t * 3.0).round() as i32, y, cheese);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Boiling pot: a steel stock pot with a lipped rim and side handles; the
/// water rises with progress, bubbles stream up through it and burst at the
/// surface, and steam curls over the rim once it is near the boil.
struct BoilingPot;
impl ProgressStyle for BoilingPot {
    fn name(&self) -> &str {
        "boiling-pot"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Steel pot with handles fills with bubbling water; bubbles burst at the surface and steam rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 12 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let steel = Color::rgb(160, 170, 185);
        let water = Color::rgb(60, 145, 235);
        let foam = Color::rgb(210, 235, 255);
        let steam = Color::rgb(175, 175, 190);
        let pw = (wi * 6 / 10).max(6);
        let x0 = (wi - pw) / 2;
        let x1 = x0 + pw;
        let rim = (hi / 3).max(2);
        // Pot: walls, base, lipped rim, loop handles.
        for y in rim..hi {
            px(grid, x0, y, steel);
            px(grid, x1, y, steel);
        }
        for x in x0 - 2..=x1 + 2 {
            px(grid, x, rim, steel);
        }
        for x in x0..=x1 {
            px(grid, x, hi - 1, steel);
        }
        for &(dx, dy) in &[(1, 2), (2, 2), (3, 2), (3, 3), (3, 4), (2, 4), (1, 4)] {
            px(grid, x0 - dx, rim + dy, steel);
            px(grid, x1 + dx, rim + dy, steel);
        }
        // Water, with bubble gaps streaming upward on fixed columns.
        let ih = hi - 2 - rim;
        let level = (ctx.eased * ih as f32).round() as i32;
        let surface = hi - 1 - level;
        for y in surface..hi - 1 {
            for x in x0 + 1..x1 {
                let col = (x - x0) % 6 == 3;
                let rise = ((ctx.time * 7.0) as i32 + x * 5) % (level + 2);
                if col && y == hi - 2 - rise {
                    continue;
                }
                px(grid, x, y, water);
            }
        }
        if level > 0 {
            // Bursting bubbles dance just above the surface.
            for x in x0 + 1..x1 {
                let pop = ((ctx.time * 9.0) as i32 + x * 7) % 9;
                if pop == 0 || (pop == 1 && ctx.eased > 0.5) {
                    px(grid, x, surface - 1, foam);
                }
            }
        }
        // Steam over the rim once the pot is past half full.
        if ctx.eased > 0.5 {
            for k in 0..3 {
                let sx = x0 + pw * (k + 1) / 4;
                for y in 0..rim - 1 {
                    let wob = ((ctx.time * 3.0 + y as f32 * 0.8 + k as f32 * 2.0).sin() * 1.5)
                        .round() as i32;
                    if (y + (ctx.time * 6.0) as i32 + k) % 3 != 0 {
                        px(grid, sx + wob, y, steam);
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Sushi conveyor belt: salmon and tamago nigiri ride a moving belt and
/// fill it from the left as progress grows.
struct SushiConveyor;
impl ProgressStyle for SushiConveyor {
    fn name(&self) -> &str {
        "sushi-conveyor"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Salmon and tamago nigiri glide along a moving conveyor belt that fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let belt = Color::rgb(125, 125, 140);
        let rice = Color::rgb(245, 245, 240);
        let salmon = Color::rgb(250, 120, 70);
        let nori = Color::rgb(40, 90, 60);
        let egg = Color::rgb(250, 215, 70);
        // Belt: rails plus tread marks that scroll with time.
        let shift = (ctx.time * 8.0) as i32;
        for x in 0..wi {
            px(grid, x, hi - 3, belt);
            px(grid, x, hi - 1, belt);
            if (x - shift).rem_euclid(4) == 0 {
                px(grid, x, hi - 2, belt);
            }
        }
        // Dishes ride the belt; only those inside the filled span are shown.
        let pitch = 16;
        let travel = ctx.time * 8.0;
        let off = (travel as i32).rem_euclid(pitch);
        let base = (travel as i32).div_euclid(pitch);
        let fill_x = ctx.eased * wi as f32;
        let slots = wi / pitch + 2;
        for k in 0..slots {
            let x = (k - 1) * pitch + off + 2;
            if (x + 6) as f32 > fill_x {
                continue;
            }
            if (k - base).rem_euclid(2) == 0 {
                // Nigiri: rice block topped by a striped salmon slab.
                for y in hi - 7..hi - 3 {
                    for dx in 0..12 {
                        let corner = (dx == 0 || dx == 11) && y == hi - 7;
                        if !corner {
                            px(grid, x + dx, y, rice);
                        }
                    }
                }
                for y in hi - 10..hi - 7 {
                    for dx in -1..13 {
                        let corner = (dx == -1 || dx == 12) && y == hi - 10;
                        if !corner && (dx + y).rem_euclid(4) != 0 {
                            px(grid, x + dx, y, salmon);
                        }
                    }
                }
            } else {
                // Tamago: rice block, yellow omelette slab, nori belt. Drawn
                // bottom-up so the omelette owns the cell it shares with rice.
                for y in (hi - 10..hi - 3).rev() {
                    for dx in 0..12 {
                        let c = if (5..=6).contains(&dx) {
                            nori
                        } else if y < hi - 7 {
                            egg
                        } else {
                            rice
                        };
                        let corner = (dx == 0 || dx == 11) && (y == hi - 10 || y == hi - 7);
                        if !corner {
                            px(grid, x + dx, y, c);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Red-and-white striped popcorn boxes heaping up with fluffy popped kernels,
/// one box after another; a fresh kernel pops out of the filling box.
struct PopcornPopping;
impl ProgressStyle for PopcornPopping {
    fn name(&self) -> &str {
        "popcorn-popping"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Striped popcorn boxes heap up with fluffy kernels in turn while fresh ones pop into the air"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let red = Color::rgb(225, 40, 45);
        let white = Color::rgb(245, 240, 230);
        let corn = Color::rgb(255, 235, 150);
        let top = hi - 10;
        // Kernel heap layout above the box mouth, filled in this order.
        let heap = [
            (-5, 1),
            (0, 1),
            (5, 1),
            (-3, -2),
            (3, -2),
            (0, -4),
            (-7, -1),
            (7, -1),
        ];
        let puff = |grid: &mut BrailleGrid, kx: i32, ky: i32| {
            for &(dx, dy) in &[
                (0, -1),
                (1, -1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
            ] {
                px(grid, kx + dx, ky + dy, corn);
            }
        };
        for i in 0..n {
            let bx = pitch * i + pitch / 2;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            // Box: tapers from 16 dots at the mouth to 12 at the base, with
            // vertical red stripes on white.
            for y in top..hi {
                let hw = 8 - (y - top) * 2 / 9;
                for dx in -hw..=hw {
                    let stripe = (dx + 16).rem_euclid(4) < 2;
                    let edge = dx.abs() == hw || y == top || y == hi - 1;
                    if stripe {
                        px(grid, bx + dx, y, red);
                    } else if edge {
                        px(grid, bx + dx, y, white);
                    }
                }
            }
            // Heap of popped kernels grows with this box's share of progress.
            let count = (f * heap.len() as f32).round() as usize;
            for &(hx, hy) in heap.iter().take(count) {
                puff(grid, bx + hx, top - 2 + hy);
            }
            // The box being filled pops a kernel on a little arc.
            if f > 0.0 && f < 1.0 {
                let ph = (ctx.time * 1.6 + i as f32).fract();
                let kx = bx + ((ph - 0.5) * 14.0) as i32;
                let ky = top - 2 - ((ph * PI).sin() * (top - 2) as f32) as i32;
                puff(grid, kx, ky);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Egg timers: a row of upright hourglasses that run one after another.
struct EggTimer;
impl ProgressStyle for EggTimer {
    fn name(&self) -> &str {
        "egg-timer"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Row of wooden-capped hourglass egg timers — each drains a trickle of sand from its top bulb into a growing heap, one after another"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let glass = Color::rgb(170, 200, 230);
        let sand = Color::rgb(240, 200, 110);
        let wood = Color::rgb(170, 110, 60);
        let n = ((w as i32 + 9) / 22).max(1);
        // Half-width of the glass at row `r` (1..=14): two bulbs pinched at 7/8.
        let half = |r: i32| -> i32 {
            let k = if r <= 7 { r } else { 15 - r };
            (5.0 - (k - 1) as f32 * 0.75).round().max(1.0) as i32
        };
        for i in 0..n {
            let x0 = i * 22 + 4;
            let cx = x0 + 6;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            // Glass walls.
            for r in 1..=14 {
                px(grid, cx - half(r), r, glass);
                px(grid, cx + half(r), r, glass);
            }
            // Top bulb: sand sits in the funnel and sinks as it drains.
            let top_rows = ((1.0 - f) * 5.0).round() as i32;
            for r in (8 - top_rows)..=7 {
                for x in (cx - half(r) + 1)..=(cx + half(r) - 1).max(cx) {
                    px(grid, x, r, sand);
                }
            }
            // Bottom bulb: a heap that grows and narrows toward its peak.
            let heap = (f * 5.0).round() as i32;
            for k in 0..heap {
                let r = 14 - k;
                let ww = (half(r) - 1 - k / 2).max(0);
                for x in (cx - ww)..=(cx + ww) {
                    px(grid, x, r, sand);
                }
            }
            // Falling trickle through the neck while the timer runs.
            if f > 0.0 && f < 1.0 {
                let flick = (ctx.time * 10.0) as i32;
                for r in 8..=(14 - heap) {
                    if (r + flick) % 2 == 0 {
                        px(grid, cx, r, sand);
                    }
                }
            }
            // Wooden end caps and corner posts.
            for x in x0..=x0 + 12 {
                px(grid, x, 0, wood);
                px(grid, x, 15, wood);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pancake stack on a plate, growing one golden pancake at a time; a butter
/// pat sits on top and maple syrup drips down the side over time.
struct PancakeStack;
impl ProgressStyle for PancakeStack {
    fn name(&self) -> &str {
        "pancake-stack"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Golden pancakes stack up on a plate one by one, topped with butter and dripping syrup"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let plate = Color::rgb(225, 230, 240);
        let cake = Color::rgb(230, 165, 70);
        let butter = Color::rgb(255, 235, 120);
        let syrup = Color::rgb(150, 70, 20);
        let cx = wi / 2;
        let pw = (wi * 2 / 5).max(8);
        // Plate: a wide rim under the stack.
        for x in cx - pw / 2 - 6..=cx + pw / 2 + 6 {
            px(grid, x, hi - 1, plate);
        }
        // Pancakes: 3 dots tall each, rounded ends, slightly varied widths.
        let max = ((hi - 4) / 3).max(1);
        let count = (ctx.eased * max as f32).ceil() as i32;
        let mut top = hi - 1;
        for k in 0..count {
            let hw = pw / 2 - (hash(k as u32 + 11) % 4) as i32;
            let y0 = hi - 2 - k * 3;
            for dy in 0..3 {
                // Bottom row is a seam gap (only the rounded rims show), so
                // each pancake reads separately from the one below.
                let inset = i32::from(dy != 1);
                for x in cx - hw + inset..=cx + hw - inset {
                    if dy != 0 || (x - cx).abs() >= hw - 2 {
                        px(grid, x, y0 - dy, cake);
                    }
                }
            }
            top = y0 - 2;
        }
        if count > 0 {
            // Butter pat and a syrup drip running down the right side.
            for y in top - 2..top {
                for x in cx - 2..cx + 2 {
                    px(grid, x, y, butter);
                }
            }
            let drip_x = cx + pw / 2 - 4;
            let len = (hi - 2 - top).max(1);
            let reach = ((ctx.time * 3.0).fract() * len as f32) as i32;
            for y in top..=top + reach {
                px(grid, drip_x, y, syrup);
                px(grid, drip_x + 1, y, syrup);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Candy jar: a lidded glass jar fills from the bottom with brightly coloured
/// round sweets, one per cell, while the next sweet drops in from the top.
struct CandyJar;
impl ProgressStyle for CandyJar {
    fn name(&self) -> &str {
        "candy-jar"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Lidded glass jar fills with multicoloured round sweets from the bottom up as one more drops in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 12 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let glass = Color::rgb(175, 205, 225);
        let lid = Color::rgb(220, 60, 120);
        let sweets = [
            Color::rgb(240, 60, 60),
            Color::rgb(250, 210, 50),
            Color::rgb(80, 210, 90),
            Color::rgb(70, 150, 245),
            Color::rgb(245, 120, 210),
            Color::rgb(255, 150, 40),
        ];
        // Jar walls sit on odd/even dot columns so they own their cells.
        let jw = ((wi * 6 / 10) & !1).max(8);
        let x0 = ((wi - jw) / 2) | 1;
        let x1 = x0 + jw + 1;
        // Lid with a knob, then the glass: shoulders, walls, base.
        for x in x0 + 2..=x1 - 2 {
            px(grid, x, 1, lid);
            px(grid, x, 2, lid);
        }
        let mid = (x0 + x1) / 2;
        px(grid, mid, 0, lid);
        px(grid, mid + 1, 0, lid);
        for y in 4..hi {
            px(grid, x0, y, glass);
            px(grid, x1, y, glass);
        }
        px(grid, x0 + 1, 3, glass);
        px(grid, x1 - 1, 3, glass);
        for x in x0..=x1 {
            px(grid, x, hi - 1, glass);
        }
        // Sweets fill whole cells inside the jar, bottom row first.
        let (c0, c1) = ((x0 + 1) / 2, x1 / 2 - 1);
        let rows = hi / 4 - 1;
        if c1 < c0 || rows < 1 {
            return Ok(());
        }
        let per_row = c1 - c0 + 1;
        let total = per_row * rows;
        let count = (ctx.eased * total as f32).round() as i32;
        let sweet = |grid: &mut BrailleGrid, x: i32, y: i32, c: Color| {
            for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1), (0, -1), (1, 2)] {
                px(grid, x + dx, y + dy, c);
            }
        };
        for k in 0..count {
            let (cx, cy) = (c0 + k % per_row, hi / 4 - 1 - k / per_row);
            let c = sweets[(hash(k as u32 * 7 + 3) % 6) as usize];
            sweet(grid, cx * 2, cy * 4 + 1, c);
        }
        // The next sweet falls in from under the lid.
        if count < total {
            let (cx, cy) = (c0 + count % per_row, hi / 4 - 1 - count / per_row);
            let ty = cy * 4 + 1;
            let fy = 4 + ((ctx.time * 1.5).fract() * (ty - 4).max(0) as f32) as i32;
            sweet(
                grid,
                cx * 2,
                fy,
                sweets[(hash(count as u32 * 7 + 3) % 6) as usize],
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Noodle bowl: a red ramen bowl with chopsticks lifting a long wiggly
/// noodle that is slurped shorter and shorter as progress grows.
struct NoodleSlurp;
impl ProgressStyle for NoodleSlurp {
    fn name(&self) -> &str {
        "noodle-slurp"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Chopsticks lift a wiggly noodle from a red ramen bowl; it is slurped shorter as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bowl = Color::rgb(210, 45, 45);
        let rim_c = Color::rgb(240, 240, 235);
        let noodle = Color::rgb(250, 220, 120);
        let stick = Color::rgb(160, 110, 60);
        // Bowl: lower half-ellipse on the left with a pale rim.
        let bhw = (wi / 6).clamp(6, 15);
        let bx = bhw + 1;
        let rim = hi - 9;
        for y in rim..hi {
            let t = (y - rim) as f32 / (hi - rim) as f32;
            let hw = (bhw as f32 * (1.0 - t * t).sqrt()).round() as i32;
            for x in bx - hw..=bx + hw {
                px(grid, x, y, if y == rim { rim_c } else { bowl });
            }
        }
        // The lifted noodle: a wiggling strand out of the bowl to the right,
        // getting shorter as it is slurped.
        let start = bx + bhw / 2;
        let full = wi - 2 - start;
        let end = start + ((1.0 - ctx.eased) * full as f32) as i32;
        let mid = (rim / 2).max(2) as f32;
        for x in start..=end {
            let t = (x - start) as f32;
            let y = mid + (t * 0.45 - ctx.time * 7.0).sin() * (mid - 1.0) * (t / 10.0).min(1.0);
            px(grid, x, y.round() as i32, noodle);
            px(grid, x, y.round() as i32 + 1, noodle);
        }
        // Chopsticks pinch the noodle's far end.
        if end > start + 2 {
            let t = (end - start) as f32;
            let ey = (mid + (t * 0.45 - ctx.time * 7.0).sin() * (mid - 1.0) * (t / 10.0).min(1.0))
                .round() as i32;
            for k in 0..6 {
                px(grid, end + 1 + k, ey - 1 - k / 2, stick);
                px(grid, end + 1 + k, ey + 2 + k / 2, stick);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Toaster: two slices of bread sink into a chrome toaster as the lever goes
/// down, browning from pale to golden to dark, then pop up when done; heat
/// shimmers over the slots while toasting.
struct ToastBrowning;
impl ProgressStyle for ToastBrowning {
    fn name(&self) -> &str {
        "toast-browning"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Two bread slices sink into a chrome toaster and brown with progress, popping up at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let chrome = Color::rgb(185, 195, 210);
        let p = ctx.eased;
        let lerp = |a: f32, b: f32, t: f32| (a + (b - a) * t) as u8;
        // Bread colour: pale -> golden over the first half, golden -> brown after.
        let toast = if p < 0.5 {
            let t = p * 2.0;
            Color::rgb(
                lerp(245.0, 225.0, t),
                lerp(225.0, 160.0, t),
                lerp(170.0, 60.0, t),
            )
        } else {
            let t = (p - 0.5) * 2.0;
            Color::rgb(
                lerp(225.0, 140.0, t),
                lerp(160.0, 80.0, t),
                lerp(60.0, 30.0, t),
            )
        };
        let tw = (wi / 3).clamp(10, 36);
        let x0 = (wi - tw) / 2;
        let x1 = x0 + tw;
        let top = hi / 2;
        // Bread sinks while toasting and pops up (with a bounce) when done.
        let sink = if p >= 1.0 {
            -(((ctx.time * 6.0).sin().abs()) * 2.0) as i32
        } else {
            (p.min(0.15) / 0.15 * 3.0) as i32
        };
        let sw = (tw - 6) / 2;
        for s in 0..2 {
            let sx = x0 + 2 + s * (sw + 2);
            let sy = top - 6 + sink;
            for y in sy..top {
                for x in sx..sx + sw {
                    // Rounded shoulders on the top of each slice.
                    let corner = y == sy && (x == sx || x == sx + sw - 1);
                    if !corner {
                        px(grid, x, y, toast);
                    }
                }
            }
        }
        // Toaster body: rounded box with slots on top and a side lever.
        for y in top..hi {
            for x in x0..=x1 {
                let corner = (y == top || y == hi - 1) && (x == x0 || x == x1);
                let edge = x == x0 || x == x1 || y == top || y == hi - 1;
                let stripe = y == top + 3;
                if !corner && (edge || stripe) {
                    px(grid, x, y, chrome);
                }
            }
        }
        let lever_y = if p >= 1.0 {
            top + 1
        } else {
            top + 1 + (p * (hi - top - 3) as f32) as i32
        };
        for x in x1 + 1..=x1 + 3 {
            px(grid, x, lever_y, chrome);
        }
        // Heat shimmer above the slots while toasting.
        if p > 0.0 && p < 1.0 {
            for k in 0..3 {
                let hx = x0 + tw * (k + 1) / 4;
                for y in 0..top - 7 + sink {
                    let wob = ((ctx.time * 5.0 + y as f32 + k as f32).sin() * 1.2).round() as i32;
                    if (y + (ctx.time * 8.0) as i32) % 2 == 0 {
                        px(grid, hx + wob, y, Color::rgb(255, 140, 60));
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Lattice pie seen from the side: a tapered tin, crimped rim, and a domed
/// top whose woven lattice crust sweeps across with progress; steam rises
/// from the baked part.
struct PieSweep;
impl ProgressStyle for PieSweep {
    fn name(&self) -> &str {
        "pie-sweep"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Domed lattice pie in a tin: the woven golden crust sweeps across as progress grows, steaming"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let tin = Color::rgb(170, 178, 190);
        let crust = Color::rgb(230, 165, 70);
        let raw = Color::rgb(120, 105, 85);
        let steam = Color::rgb(175, 175, 190);
        let pw = (wi * 7 / 10).max(12);
        let x0 = (wi - pw) / 2;
        let x1 = x0 + pw;
        let rim = hi - 6;
        // Tin: tapered sides and base.
        for y in rim + 1..hi {
            let inset = (y - rim) / 2;
            px(grid, x0 + inset, y, tin);
            px(grid, x1 - inset, y, tin);
        }
        for x in x0 + 2..=x1 - 2 {
            px(grid, x, hi - 1, tin);
        }
        let sweep = x0 as f32 + ctx.eased * pw as f32;
        // Crimped rim: a wavy band of crust along the top of the tin.
        for x in x0 - 1..=x1 + 1 {
            let c = if (x as f32) < sweep { crust } else { raw };
            px(grid, x, rim, c);
            px(grid, x, rim + 1 + (x % 2), c);
        }
        // Domed top: woven lattice where baked, outline where not.
        let cx = (x0 + x1) as f32 / 2.0;
        let rx = pw as f32 / 2.0;
        let ry = (rim - 2) as f32;
        let dome = |x: i32, y: i32| {
            let nx = (x as f32 - cx) / rx;
            let ny = (y - rim) as f32 / ry;
            nx * nx + ny * ny <= 1.0 && y < rim
        };
        for y in 0..rim {
            for x in x0..=x1 {
                if !dome(x, y) {
                    continue;
                }
                let edge = !dome(x, y - 1) || !dome(x - 1, y) || !dome(x + 1, y);
                if (x as f32) < sweep {
                    let lattice = (x + y).rem_euclid(5) < 2 || (x - y).rem_euclid(5) < 2;
                    if edge || lattice {
                        px(grid, x, y, crust);
                    }
                } else if edge {
                    px(grid, x, y, raw);
                }
            }
        }
        // Steam curls up from the baked side of the dome.
        if ctx.eased > 0.1 {
            let sx = ((x0 as f32 + sweep) / 2.0) as i32;
            for y in 0..3 {
                let wob = ((ctx.time * 4.0 + y as f32).sin() * 1.5).round() as i32;
                if (y + (ctx.time * 6.0) as i32) % 3 != 0 {
                    px(grid, sx + wob, y, steam);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// All styles in the `food` theme.
///
/// Returns one boxed [`ProgressStyle`] per food/kitchen bar. Call this to get
/// the full set for display in a gallery or picker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(BeerGlass),
        Box::new(CoffeePour),
        Box::new(PizzaSlices),
        Box::new(BoilingPot),
        Box::new(SushiConveyor),
        Box::new(PopcornPopping),
        Box::new(EggTimer),
        Box::new(PancakeStack),
        Box::new(CandyJar),
        Box::new(NoodleSlurp),
        Box::new(ToastBrowning),
        Box::new(PieSweep),
    ]
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::food::styles();
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
