//! `weather` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O weather.rs && ./weather [style-name]
//! ```

const DEFAULT_STYLE: &str = "hurricane";

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
    pub mod weather {
//! Weather / meteorology progress bars.
//!
//! Twelve animated styles built entirely from `draw::` helpers. Each bar reads
//! `ctx.eased` for its fill/intensity amount and `ctx.time` for looping
//! animation, so they stay alive even when progress is held constant. Every
//! style targets a distinct atmospheric phenomenon — hurricane spirals, tornado
//! funnels, accumulating snowdrifts, hailstreaks, rainbow arcs, fog density,
//! barometer needles, windsock gusts, blizzard whiteout, cumulonimbus
//! convection, frost crystal propagation, and a mercury thermometer column.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── registry ──────────────────────────────────────────────────────────────────

/// All styles in the `weather` theme.
///
/// Each style is a structurally distinct weather phenomenon — spiral arms,
/// funnel geometry, accumulation physics, optical arcs, density fields,
/// analog instruments, fluid flow, and crystal growth. Color is not the
/// distinguishing axis; the drawn geometry is.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Hurricane),
        Box::new(Tornado),
        Box::new(SnowAccumulation),
        Box::new(Hailstorm),
        Box::new(RainbowArc),
        Box::new(FogRollIn),
        Box::new(Barometer),
        Box::new(WindSock),
        Box::new(BlizzardWhiteout),
        Box::new(Cumulonimbus),
        Box::new(FrostCrystals),
        Box::new(Thermometer),
    ]
}

// ── deterministic hash ────────────────────────────────────────────────────────

/// Fast integer hash used for stable per-particle position seeds.
#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ── picture helpers ───────────────────────────────────────────────────────────

/// Colour every cell overlapping the dot box `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    if x1 < 0 || y1 < 0 || x1 < x0 || y1 < y0 {
        return;
    }
    let (cx0, cx1) = ((x0.max(0) / 2) as usize, (x1 / 2) as usize);
    let (cy0, cy1) = ((y0.max(0) / 4) as usize, (y1 / 4) as usize);
    for cy in cy0..=cy1 {
        draw::tint_row(grid, cy, cx0, cx1, color);
    }
}

/// Elliptical ring of dots centred on `(cx, cy)`.
fn ring(grid: &mut BrailleGrid, cx: f32, cy: f32, rx: f32, ry: f32) {
    let n = ((rx + ry) * 3.0).max(8.0) as usize;
    for i in 0..n {
        let a = i as f32 / n as f32 * 2.0 * PI;
        draw::dot_i(
            grid,
            (cx + a.cos() * rx).round() as i32,
            (cy + a.sin() * ry).round() as i32,
        );
    }
}

// ── 1. Hurricane ─────────────────────────────────────────────────────────────

/// Satellite-view cyclone: two spiral cloud arms around a clear eye.
struct Hurricane;
impl ProgressStyle for Hurricane {
    fn name(&self) -> &str {
        "hurricane"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "A spinning cyclone with spiral cloud arms and a red eye tracks along its forecast path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let cy = (h as f32 - 1.0) / 2.0;
        let rmax = cy.min(w as f32 / 4.0);
        let cx = rmax + 1.0 + ctx.eased * (w as f32 - 2.0 * rmax - 2.0).max(0.0);

        // Forecast track: solid behind the storm, dashed cone ahead.
        let yi = cy.round() as i32;
        for x in 0..w as i32 {
            let behind = (x as f32) < cx - rmax;
            let ahead = (x as f32) > cx + rmax && x % 4 == 0;
            if behind && x % 2 == 0 || ahead {
                draw::dot_i(grid, x, yi);
            }
        }
        paint(grid, 0, yi, w as i32 - 1, yi, Color::rgb(255, 170, 60));

        // Two spiral arms wind out of the eye wall and rotate with time.
        let spin = -ctx.time * 2.5;
        for arm in 0..2 {
            let off = arm as f32 * PI + spin;
            let steps = 70;
            for s in 0..steps {
                let th = s as f32 / steps as f32 * 1.6 * PI;
                let r = 2.2 + th / (1.6 * PI) * (rmax - 1.5);
                for dr in [0.0_f32, 0.9] {
                    let a = off + th;
                    let rr = (r - dr * (1.0 - th / (1.6 * PI))).max(2.0);
                    draw::dot_i(
                        grid,
                        (cx + a.cos() * rr).round() as i32,
                        (cy + a.sin() * rr).round() as i32,
                    );
                }
            }
        }
        // Dense eye wall.
        ring(grid, cx, cy, 1.6, 1.6);
        let ri = rmax as i32;
        paint(
            grid,
            cx as i32 - ri,
            cy as i32 - ri,
            cx as i32 + ri,
            cy as i32 + ri,
            Color::rgb(225, 240, 255),
        );
        paint(
            grid,
            cx as i32 - 1,
            cy as i32 - 1,
            cx as i32 + 1,
            cy as i32 + 1,
            Color::rgb(255, 70, 70),
        );
        Ok(())
    }
}

// ── 2. Tornado ───────────────────────────────────────────────────────────────

/// A twisting funnel hanging from a storm cloud, tearing a path along the ground.
struct Tornado;
impl ProgressStyle for Tornado {
    fn name(&self) -> &str {
        "tornado"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "A twisting funnel drops from a storm cloud and tears across the ground, scattering debris"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let t = ctx.time;

        // Storm cloud: a lumpy dark ceiling across the whole bar.
        for x in 0..wi {
            let lump = ((x as f32 * 0.35 + t * 0.8).sin() * 1.2 + 1.6) as i32;
            draw::vline(grid, x as usize, 0, lump.max(0) as usize);
        }
        paint(grid, 0, 0, wi - 1, 2, Color::rgb(120, 120, 150));

        // Funnel position follows progress.
        let span = (wi - 20).max(1) as f32;
        let fx = 10.0 + ctx.eased * span;
        let top_y = 2;
        let fh = (base - top_y).max(1) as f32;
        for y in top_y..=base {
            let u = (y - top_y) as f32 / fh;
            // The column snakes more towards its tip.
            let sway = (t * 3.0 + u * 4.0).sin() * 3.0 * u;
            let cx = fx + sway - u * 2.0;
            let hw = 10.0 * (1.0 - u).powf(1.4) + 0.8;
            // Edges plus rotating bands across the column.
            draw::dot_i(grid, (cx - hw).round() as i32, y);
            draw::dot_i(grid, (cx + hw).round() as i32, y);
            let n = (hw * 2.0) as i32;
            for k in 0..=n {
                let x = cx - hw + k as f32;
                let phase = (x - cx) / hw.max(0.5) * 2.0 + t * 9.0 + y as f32 * 0.9;
                if phase.sin() > 0.0 {
                    draw::dot_i(grid, x.round() as i32, y);
                }
            }
        }
        paint(
            grid,
            fx as i32 - 14,
            3,
            fx as i32 + 12,
            base,
            Color::rgb(170, 190, 175),
        );

        // Ground ahead is intact; behind the funnel it is torn into debris.
        let tail = fx as i32 - 4;
        if tail + 2 < wi {
            draw::hline(grid, (tail + 2).max(0) as usize, w - 1, base as usize);
        }
        for i in 0..24u32 {
            let seed = hash(i * 7 + 3);
            let x = (seed % (tail.max(1) as u32)) as i32;
            let hop = ((t * 4.0 + i as f32).sin().abs() * 3.0) as i32;
            draw::dot_i(grid, x, base - hop);
        }
        // Debris whirling around the funnel base.
        for i in 0..6 {
            let a = t * 6.0 + i as f32 * 1.05;
            draw::dot_i(
                grid,
                (fx - 4.0 + a.cos() * 6.0) as i32,
                base - 2 - (a.sin().abs() * 3.0) as i32,
            );
        }
        paint(grid, 0, base - 3, tail - 4, base, Color::rgb(170, 120, 70));
        Ok(())
    }
}

// ── 3. Snow accumulation ─────────────────────────────────────────────────────

/// Snowflakes drift down onto a rising, dune-topped drift that buries a fence.
struct SnowAccumulation;
impl ProgressStyle for SnowAccumulation {
    fn name(&self) -> &str {
        "snow-accumulation"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Star-shaped snowflakes drift down onto a white drift that rises with progress, burying a wooden fence"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let t = ctx.time;

        // Wooden fence: posts with two rails, drawn first so snow covers it.
        for x in (4..wi).step_by(12) {
            draw::vline(grid, x as usize, (base - 9) as usize, base as usize);
            draw::dot_i(grid, x, base - 10);
        }
        draw::hline(grid, 4, (wi - 1) as usize, (base - 7) as usize);
        draw::hline(grid, 4, (wi - 1) as usize, (base - 3) as usize);
        paint(grid, 0, base - 10, wi - 1, base, Color::rgb(170, 110, 60));

        // The drift: solid snow up to a dune-shaped surface set by progress.
        let depth = 1.0 + ctx.eased * (hi as f32 - 4.0);
        for x in 0..wi {
            let dune = (x as f32 * 0.15).sin() * 1.2 + (x as f32 * 0.05 + 1.0).sin() * 0.8;
            let top = (base as f32 - depth + dune).round() as i32;
            draw::vline(grid, x as usize, top.clamp(0, base) as usize, base as usize);
        }
        let surface = (base as f32 - depth - 1.0) as i32;
        paint(
            grid,
            0,
            surface + 3,
            wi - 1,
            base,
            Color::rgb(235, 245, 255),
        );

        // Falling flakes: little five-dot stars swaying as they fall.
        for i in 0..14u32 {
            let x0 = (hash(i * 5 + 2) % w as u32) as f32;
            let speed = 4.0 + (hash(i + 21) % 5) as f32;
            let fall = (t * speed + (hash(i + 40) % 64) as f32) % (hi as f32 + 3.0) - 2.0;
            let x = (x0 + (t * 1.5 + i as f32).sin() * 2.0) as i32;
            let y = fall as i32;
            if y < surface {
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x - 1, y);
                draw::dot_i(grid, x + 1, y);
                draw::dot_i(grid, x, y - 1);
                draw::dot_i(grid, x, y + 1);
                paint(grid, x, y, x, y, Color::rgb(255, 255, 255));
            }
        }
        Ok(())
    }
}

// ── 4. Hailstorm ─────────────────────────────────────────────────────────────

/// Round ice pellets pelting down from a dark cloud and piling up white.
struct Hailstorm;
impl ProgressStyle for Hailstorm {
    fn name(&self) -> &str {
        "hailstorm"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Round ice pellets pelt down from a dark cloud, bounce on impact and pile up white as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let t = ctx.time;

        // Storm cloud ceiling.
        for x in 0..wi {
            let lump = ((x as f32 * 0.4 + t * 0.6).sin() * 1.0 + 1.5) as i32;
            draw::vline(grid, x as usize, 0, lump.max(0) as usize);
        }
        paint(grid, 0, 0, wi - 1, 2, Color::rgb(100, 100, 125));

        // White pile along the ground, filling left to right with progress.
        let filled = (ctx.eased * w as f32) as i32;
        for x in 0..filled.min(wi) {
            let mound = 2 + (hash(x as u32 / 3) % 2) as i32;
            draw::vline(grid, x as usize, (base - mound + 1) as usize, base as usize);
        }
        draw::hline(grid, 0, w - 1, base as usize);

        // Falling 2x2 pellets; more of them as the storm intensifies.
        let n = 10 + (ctx.eased * 26.0) as u32;
        let fall_h = (base - 3).max(1) as f32;
        for i in 0..n {
            let x = (hash(i * 3 + 1) % w as u32) as f32;
            let speed = 22.0 + (hash(i + 5) % 10) as f32;
            let phase = (t * speed / fall_h + (hash(i + 11) % 100) as f32 / 100.0).fract();
            let y = 3.0 + phase * fall_h;
            let x = x + phase * 3.0;
            if phase > 0.93 {
                // Bounce: pellet pops back up beside its impact point.
                let b = (phase - 0.93) / 0.07;
                let by = base as f32 - 2.0 - (b * PI).sin() * 3.0;
                draw::fill_rect(grid, (x + b * 3.0) as usize, by.max(0.0) as usize, 2, 2);
            } else {
                draw::fill_rect(grid, x as usize, y as usize, 2, 2);
            }
        }
        paint(grid, 0, 3, wi - 1, base, Color::rgb(210, 235, 255));
        Ok(())
    }
}

// ── 5. Rainbow arc ───────────────────────────────────────────────────────────

/// Rainbow bands, red outside to violet inside, sweeping over the sky.
struct RainbowArc;
impl ProgressStyle for RainbowArc {
    fn name(&self) -> &str {
        "rainbow-arc"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "A red-to-violet rainbow arcs from the horizon and sweeps across the sky as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let cx = (w as f32 - 1.0) / 2.0;
        let cy = h as f32 - 0.5;
        let (rx, ry) = (cx - 1.0, h as f32 - 0.5);
        let sweep = ctx.eased * PI;

        // Six concentric half-ellipse bands, drawn from the left foot onward.
        for band in 0..6 {
            let k = 0.97 - band as f32 * 0.1;
            let n = (rx * 4.0) as usize;
            for i in 0..=n {
                let a = i as f32 / n as f32 * PI;
                if a > sweep {
                    break;
                }
                // Angle measured from the left horizon, over the top.
                let x = cx - a.cos() * rx * k;
                let y = cy - a.sin() * ry * k;
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
        }

        // Twinkles drift along the outer band.
        for i in 0..3 {
            let a = ((ctx.time * 0.3 + i as f32 / 3.0).fract()) * sweep;
            draw::dot_i(
                grid,
                (cx - a.cos() * (rx + 1.0)).round() as i32,
                (cy - a.sin() * (ry + 1.0)).round() as i32,
            );
        }

        // Colour each cell by its radial position: red outside, violet inside.
        let bands = [
            Color::rgb(255, 60, 60),
            Color::rgb(255, 150, 40),
            Color::rgb(255, 230, 60),
            Color::rgb(70, 220, 90),
            Color::rgb(60, 140, 255),
            Color::rgb(170, 90, 255),
        ];
        let (cw, ch) = grid.dimensions();
        for cyc in 0..ch {
            for cxc in 0..cw {
                let dx = ((cxc * 2) as f32 + 0.5 - cx) / rx;
                let dy = ((cyc * 4) as f32 + 1.5 - cy) / ry;
                let r = (dx * dx + dy * dy).sqrt();
                let idx = ((1.02 - r) / 0.1).floor().clamp(0.0, 5.0) as usize;
                draw::tint_row(grid, cyc, cxc, cxc, bands[idx]);
            }
        }
        Ok(())
    }
}

// ── 6. Fog rolling in ────────────────────────────────────────────────────────

/// Shade-glyph density sweeps left-to-right; denser at the leading edge.
struct FogRollIn;
impl ProgressStyle for FogRollIn {
    fn name(&self) -> &str {
        "fog-roll-in"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Dense fog rolls in from the left; the leading edge thickens with a shade-glyph gradient"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        let fog_front = (ctx.eased * cw as f32) as usize;

        for cx in 0..cw {
            // How deeply into the fog are we? 0=clear, 1=deep fog.
            if cx >= fog_front {
                break;
            }
            let depth = (fog_front - cx) as f32 / fog_front.max(1) as f32;
            // Add a slow undulation from time.
            let wave = ((cx as f32 * 0.5 + ctx.time * 1.5).sin() * 0.12 + 1.0).clamp(0.5, 1.5);
            let density = (depth * wave).clamp(0.0, 1.0);

            // Map density [0..1] → shade level [0..4].
            let level = (density * 4.0).round() as usize;

            for cy in 0..ch {
                // Row-vary the fog slightly — ground-level is denser.
                let row_extra = cy as f32 / ch.saturating_sub(1).max(1) as f32 * 0.5;
                let row_level = ((density + row_extra).clamp(0.0, 1.0) * 4.0).round() as usize;
                draw::shade(grid, cx, cy, row_level.min(4));
                let _ = level; // used above for clarity
            }
        }

        // Tint the fog grey-blue via palette.
        let (cw2, ch2) = grid.dimensions();
        for cy_c in 0..ch2 {
            let color = ctx.palette.sample(ctx.eased * 0.5);
            draw::tint_row(grid, cy_c, 0, fog_front.min(cw2).saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 7. Barometer ─────────────────────────────────────────────────────────────

/// Storm cloud icon (9 wide × 7 tall) for the low end of the dial.
const ICON_STORM: [&str; 7] = [
    "..###....",
    ".#####.#.",
    "#########",
    "#########",
    "....#....",
    "...#.....",
    "....#....",
];
/// Sun icon (9 wide × 7 tall) for the high end of the dial.
const ICON_SUN: [&str; 7] = [
    "#...#...#",
    ".#..#..#.",
    "...###...",
    "#.#####.#",
    "...###...",
    ".#..#..#.",
    "#...#...#",
];

/// Stamp an ASCII sprite: every `#` becomes a dot at `(x0 + col, y0 + row)`.
fn sprite(grid: &mut BrailleGrid, x0: i32, y0: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x0 + c as i32, y0 + r as i32);
            }
        }
    }
}

/// An analog pressure dial with a rotating needle driven by `eased`.
struct Barometer;
impl ProgressStyle for Barometer {
    fn name(&self) -> &str {
        "barometer"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Brass-rimmed pressure dial: a red needle sweeps from the storm cloud to the sun as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cx = (w as f32 - 1.0) / 2.0;
        let cy = h as f32 - 1.0;
        let ry = h as f32 - 1.0;
        let rx = (ry * 2.2).min(cx - 12.0).max(ry);

        // Brass bezel: double half-ring along the top of the dial.
        let n = (rx * 4.0) as usize;
        for i in 0..=n {
            let a = i as f32 / n as f32 * PI;
            for k in [1.0_f32, 0.93] {
                draw::dot_i(
                    grid,
                    (cx - a.cos() * rx * k).round() as i32,
                    (cy - a.sin() * ry * k).round() as i32,
                );
            }
        }
        draw::hline(
            grid,
            (cx - rx).max(0.0) as usize,
            (cx + rx) as usize,
            (hi - 1) as usize,
        );
        paint(grid, 0, 0, wi - 1, hi - 1, Color::rgb(210, 170, 80));

        // Scale ticks inside the bezel: long every fourth.
        for i in 0..=12 {
            let a = i as f32 / 12.0 * PI;
            let k0 = if i % 4 == 0 { 0.62 } else { 0.72 };
            for k in [k0, 0.8] {
                draw::dot_i(
                    grid,
                    (cx - a.cos() * rx * k).round() as i32,
                    (cy - a.sin() * ry * k).round() as i32,
                );
            }
        }

        // Weather icons at either end of the scale.
        let ix = (cx - rx) as i32 - 11;
        sprite(grid, ix, hi - 8, &ICON_STORM);
        paint(grid, ix, hi - 8, ix + 8, hi - 1, Color::rgb(120, 150, 210));
        let sx = (cx + rx) as i32 + 3;
        sprite(grid, sx, hi - 8, &ICON_SUN);
        paint(grid, sx, hi - 8, sx + 8, hi - 1, Color::rgb(255, 210, 60));

        // Needle: pivots at the hub, quivering slightly with time.
        let a = ctx.eased * PI + (ctx.time * 7.0).sin() * 0.03;
        let steps = (rx * 0.85) as i32;
        for s in 0..=steps {
            let k = s as f32 / rx;
            let x = (cx - a.cos() * rx * k).round() as i32;
            let y = (cy - a.sin() * ry * k).round() as i32;
            draw::dot_i(grid, x, y);
            if s > 2 {
                paint(grid, x, y, x, y, Color::rgb(255, 60, 60));
            }
        }
        draw::hline(grid, cx as usize - 1, cx as usize + 1, (hi - 2) as usize);
        Ok(())
    }
}

// ── 8. Wind sock ─────────────────────────────────────────────────────────────

/// A striped windsock on a pole that lifts and stretches with wind speed.
struct WindSock;
impl ProgressStyle for WindSock {
    fn name(&self) -> &str {
        "wind-sock"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Red-and-white striped windsock on a pole: it droops in calm air and streams out level as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;

        // Pole with a cap.
        let px = 3;
        draw::vline(grid, px as usize, 1, (hi - 1) as usize);
        draw::hline(grid, (px - 1) as usize, (px + 1) as usize, 1);
        draw::hline(grid, 0, (px + 3) as usize, (hi - 1) as usize);
        paint(grid, 0, 0, px + 1, hi - 1, Color::rgb(180, 180, 190));

        // The sock: droop angle falls and length grows as the wind rises.
        let mouth_y = 4.5_f32;
        let half0 = (hi as f32 * 0.22).max(1.0);
        let droop = (1.0 - p) * 0.85;
        let len = (12.0 + p * (w as f32 * 0.45)).min(wi as f32 - 8.0).max(4.0);
        let stripe = (len / 5.0).max(2.0);
        let (dx, dy) = (droop.cos(), droop.sin());
        let steps = len as i32;
        for s in 0..=steps {
            let u = s as f32 / len;
            let flutter =
                (ctx.time * (4.0 + p * 10.0) - s as f32 * 0.5).sin() * u * (0.3 + p * 0.6);
            let mx = px as f32 + 1.0 + dx * s as f32 - dy * flutter;
            let my = mouth_y + dy * s as f32 + dx * flutter;
            let half = half0 * (1.0 - 0.6 * u);
            // A solid cross-section slice; stripes alternate every `stripe` dots.
            let (nx, ny) = (-dy, dx);
            let solid = ((s as f32 / stripe) as i32) % 2 == 0;
            let n = (half * 2.0) as i32;
            for k in 0..=n {
                let o = -half + k as f32;
                draw::dot_i(
                    grid,
                    (mx + nx * o).round() as i32,
                    (my + ny * o).round() as i32,
                );
            }
            let color = if solid {
                Color::rgb(255, 80, 50)
            } else {
                Color::rgb(245, 245, 245)
            };
            paint(grid, mx as i32, my as i32, mx as i32, my as i32, color);
        }

        // Wind streaks blow past faster as the wind picks up.
        let speed = 10.0 + p * 50.0;
        for i in 0..5u32 {
            let lane = 2 + (hash(i) % (hi.max(4) as u32 - 3)) as i32;
            let x = ((ctx.time * speed + (hash(i + 9) % 200) as f32) % (wi as f32 + 20.0)) as i32;
            let len = 3 + (p * 6.0) as i32;
            draw::hline(
                grid,
                (x - len).clamp(0, wi - 1) as usize,
                x.clamp(0, wi - 1) as usize,
                lane as usize,
            );
        }
        Ok(())
    }
}

// ── 9. Blizzard whiteout ─────────────────────────────────────────────────────

/// Pine tree silhouette (7 wide × 9 tall) for the blizzard horizon.
const PINE: [&str; 9] = [
    "...#...", "..###..", "...#...", ".#####.", "..###..", "#######", ".#####.", "...#...",
    "...#...",
];

/// Wind-driven snow thickens until a cabin and pines vanish into whiteout.
struct BlizzardWhiteout;
impl ProgressStyle for BlizzardWhiteout {
    fn name(&self) -> &str {
        "blizzard-whiteout"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Wind-driven snow streaks thicken until a cabin and pine trees vanish into whiteout at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let p = ctx.eased;
        let t = ctx.time;

        // Horizon scene: snowy ground, pines and a cabin with a lit window,
        // hidden column by column as the whiteout front advances.
        let hidden = (p * w as f32) as i32;
        draw::hline(grid, hidden.clamp(0, wi - 1) as usize, w - 1, base as usize);
        for k in 0..6 {
            let px = 6 + k * (wi / 6);
            if px > hidden {
                sprite(grid, px, base - 9, &PINE);
                paint(
                    grid,
                    px,
                    base - 8,
                    px + 6,
                    base - 1,
                    Color::rgb(60, 150, 90),
                );
            }
        }
        let cab = wi / 2 - 6;
        if cab > hidden {
            sprite(
                grid,
                cab,
                base - 9,
                &[
                    ".....#.....",
                    "....###....",
                    "...#####...",
                    "..#######..",
                    ".#########.",
                    "..#.....#..",
                    "..#.##..#..",
                    "..#.##..#..",
                    "..#######..",
                ],
            );
            paint(
                grid,
                cab,
                base - 9,
                cab + 10,
                base - 1,
                Color::rgb(190, 120, 70),
            );
            paint(
                grid,
                cab + 4,
                base - 4,
                cab + 5,
                base - 3,
                Color::rgb(255, 210, 80),
            );
        }

        // Snow streaks raking sideways; more and longer as the storm grows.
        let n = 12 + (p * 60.0) as u32;
        for i in 0..n {
            let lane = (hash(i * 13 + 1) % hi as u32) as i32;
            let speed = 40.0 + (hash(i + 3) % 30) as f32;
            let x = wi - ((t * speed + (hash(i + 7) % 400) as f32) % (wi as f32 + 10.0)) as i32;
            let len = 2 + (p * 5.0) as i32;
            for k in 0..len {
                draw::dot_i(grid, x + k, lane + k / 3);
            }
        }
        // The whiteout wall itself: dense drifting flecks behind the front.
        for y in 0..hi {
            for x in 0..hidden.min(wi) {
                if hash((x * 31 + y * 7) as u32 + (t * 8.0) as u32) % 3 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        paint(grid, 0, 0, hidden - 1, hi - 1, Color::rgb(235, 240, 250));
        Ok(())
    }
}

// ── 10. Cumulonimbus ─────────────────────────────────────────────────────────

/// Filled disc of dots centred on `(cx, cy)`.
fn disc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let r2 = r * r + 0.35;
    for y in (cy - r).floor() as i32..=(cy + r).ceil() as i32 {
        for x in (cx - r).floor() as i32..=(cx + r).ceil() as i32 {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            if dx * dx + dy * dy <= r2 {
                draw::dot_i(grid, x, y);
            }
        }
    }
}

/// Cloud builds vertically into an anvil-topped thunderhead with rain and lightning.
struct Cumulonimbus;
impl ProgressStyle for Cumulonimbus {
    fn name(&self) -> &str {
        "cumulonimbus"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "A puffy cumulus towers up into an anvil-topped thunderhead, then rain and lightning fall from its base"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;
        let t = ctx.time;
        let s = hi as f32 / 16.0;
        let cx = wi as f32 / 2.0;
        let base_y = 9.0 * s;

        // Billowing base: a row of puffs that widens as the cell matures.
        let half = 6.0 + p * 12.0;
        for i in 0..=(half * 2.0 / 3.0) as i32 {
            let x = cx - half + i as f32 * 3.0;
            let bob = (t * 1.5 + x * 0.3).sin() * 0.4;
            disc(grid, x, base_y - 1.0 + bob, (2.2 + p) * s);
        }
        // Tower: puffs stacked upward to a height set by progress.
        let top = base_y - p * 7.5 * s;
        for i in 1..=((base_y - top) / 2.0).floor() as i32 {
            let y = base_y - 2.0 * i as f32;
            let wobble = (t * 2.0 + y).sin() * 0.8;
            disc(grid, cx - 4.0 + wobble, y, 3.5 * s);
            disc(grid, cx + 4.0 - wobble, y - 0.5, 3.5 * s);
        }
        // Anvil: a flat, wind-sheared shelf once the tower hits the ceiling.
        if p > 0.6 {
            let spread = (p - 0.6) / 0.4 * 26.0;
            let (a0, a1) = ((cx - 8.0 - spread * 0.6) as i32, (cx + 8.0 + spread) as i32);
            for row in 0..(3.0 * s).max(1.0) as i32 {
                draw::hline(
                    grid,
                    a0.max(0) as usize,
                    a1.clamp(0, wi - 1) as usize,
                    row as usize,
                );
            }
        }
        paint(
            grid,
            0,
            0,
            wi - 1,
            (base_y - 4.0) as i32,
            Color::rgb(245, 245, 255),
        );
        paint(
            grid,
            0,
            (base_y - 3.0) as i32,
            wi - 1,
            (base_y + 2.0) as i32,
            Color::rgb(150, 160, 185),
        );

        // Rain shafts below the base once the cloud is deep enough.
        if p > 0.35 {
            let n = ((p - 0.35) * 30.0) as u32;
            for i in 0..n {
                let rx = cx - half + (hash(i) % ((half * 2.0) as u32 + 1)) as f32;
                let fall = (t * 14.0 + (hash(i + 77) % 16) as f32) % (hi as f32 - base_y);
                let ry = (base_y + 2.0 + fall) as i32;
                draw::dot_i(grid, rx as i32 - (fall * 0.2) as i32, ry);
                draw::dot_i(grid, rx as i32 - (fall * 0.2) as i32, ry + 1);
            }
            paint(
                grid,
                0,
                (base_y + 3.0) as i32,
                wi - 1,
                hi - 1,
                Color::rgb(90, 150, 255),
            );
        }
        // Lightning: a jagged bolt flickers out of the mature cell.
        if p > 0.75 && (t * 0.9).fract() < 0.2 {
            let bx = (cx + 6.0 + (hash((t * 0.9) as u32) % 10) as f32) as i32;
            let mut y = (base_y + 1.0) as i32;
            let mut x = bx;
            while y < hi {
                draw::dot_i(grid, x, y);
                y += 1;
                x += if y % 4 < 2 { 1 } else { -1 };
            }
            paint(
                grid,
                bx - 2,
                (base_y + 2.0) as i32,
                bx + 2,
                hi - 1,
                Color::rgb(255, 240, 80),
            );
        }
        Ok(())
    }
}

// ── 11. Frost crystals ────────────────────────────────────────────────────────

/// Dendritic frost spreads inward from all four edges.
struct FrostCrystals;
impl ProgressStyle for FrostCrystals {
    fn name(&self) -> &str {
        "frost-crystals"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Dendritic frost crystals propagate inward from the edges; branching depth grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Frost depth in dots from each edge.
        let max_depth = ((w.min(h) / 2).saturating_sub(1)) as f32;
        let depth = (ctx.eased * max_depth).round() as usize;

        if depth == 0 {
            return Ok(());
        }

        // Grow dendritic crystals from left, right, top, bottom edges.
        // Each edge has a set of roots spaced every 4 dots along the edge.
        // From each root, a main spine grows inward; then side branches sprout
        // every few dots, each in the perpendicular direction.
        let branch_spacing = 4usize;
        let branch_len_frac = 0.4_f32;

        // Left edge.
        let mut root = 0usize;
        while root < h {
            let h_val = hash(root as u32 + 1000);
            // Slight y variation.
            let ry = root;
            for d in 0..depth.min(w) {
                draw::dot(grid, d, ry.min(h.saturating_sub(1)));
                // Side branches perpendicular (up/down).
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(h / 2) {
                        draw::dot(grid, d, (ry + b).min(h.saturating_sub(1)));
                        if ry >= b {
                            draw::dot(grid, d, ry - b);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Right edge.
        root = 0;
        while root < h {
            let h_val = hash(root as u32 + 2000);
            let ry = root;
            for d in 0..depth.min(w) {
                let x = w.saturating_sub(1 + d);
                draw::dot(grid, x, ry.min(h.saturating_sub(1)));
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(h / 2) {
                        draw::dot(grid, x, (ry + b).min(h.saturating_sub(1)));
                        if ry >= b {
                            draw::dot(grid, x, ry - b);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Top edge.
        root = 0;
        while root < w {
            let h_val = hash(root as u32 + 3000);
            let rx = root;
            for d in 0..depth.min(h) {
                draw::dot(grid, rx.min(w.saturating_sub(1)), d);
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(w / 2) {
                        draw::dot(grid, (rx + b).min(w.saturating_sub(1)), d);
                        if rx >= b {
                            draw::dot(grid, rx - b, d);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Bottom edge.
        root = 0;
        while root < w {
            let h_val = hash(root as u32 + 4000);
            let rx = root;
            for d in 0..depth.min(h) {
                let y = h.saturating_sub(1 + d);
                draw::dot(grid, rx.min(w.saturating_sub(1)), y);
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(w / 2) {
                        draw::dot(grid, (rx + b).min(w.saturating_sub(1)), y);
                        if rx >= b {
                            draw::dot(grid, rx - b, y);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Ice-blue tint.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let color = ctx.palette.sample(0.2 + ctx.eased * 0.6);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 12. Thermometer ──────────────────────────────────────────────────────────

/// A mercury column rises in a vertical thermometer tube.
struct Thermometer;
impl ProgressStyle for Thermometer {
    fn name(&self) -> &str {
        "thermometer"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Mercury column rises in a thermometer tube; bulb glows at bottom, tick marks show scale"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Thermometer shaft: centred horizontally, tube width = 4 dots.
        let cx = (w / 2) as i32;
        let tube_w = 4i32; // inner width
        let tube_left = cx - tube_w / 2;
        let tube_right = cx + tube_w / 2;

        // Tube occupies top 85% of the height; bulb is the bottom 15%.
        let bulb_top_y = (h as f32 * 0.82) as usize;
        let tube_top = 1usize;

        // Tube outline.
        for y in tube_top..bulb_top_y {
            draw::dot_i(grid, tube_left - 1, y as i32);
            draw::dot_i(grid, tube_right + 1, y as i32);
        }
        draw::hline(
            grid,
            tube_left.max(0) as usize,
            (tube_right + 1) as usize,
            tube_top,
        );

        // Bulb: filled circle at the bottom.
        let bulb_cx = cx;
        let bulb_cy = (bulb_top_y + h.saturating_sub(1)) as i32 / 2;
        let bulb_r = ((h.saturating_sub(bulb_top_y)) / 2).max(2) as i32;
        for dy in -bulb_r..=bulb_r {
            for dx in -bulb_r..=bulb_r {
                if dx * dx + dy * dy <= bulb_r * bulb_r + bulb_r {
                    draw::dot_i(grid, bulb_cx + dx, bulb_cy + dy);
                }
            }
        }

        // Mercury column: fills the tube from the bulb upward.
        let tube_h = bulb_top_y.saturating_sub(tube_top);
        let mercury_h = (ctx.eased * tube_h as f32).round() as usize;
        let mercury_top = bulb_top_y.saturating_sub(mercury_h);
        if mercury_h > 0 {
            for y in mercury_top..bulb_top_y {
                for x in tube_left..=tube_right {
                    draw::dot_i(grid, x, y as i32);
                }
            }
        }

        // Tick marks on the right side of the tube, every 25%.
        let tick_positions = [0.0, 0.25, 0.5, 0.75, 1.0];
        for &tp in &tick_positions {
            let ty = bulb_top_y.saturating_sub((tp * tube_h as f32).round() as usize);
            let tick_len = if (tp * 4.0).round() as usize % 2 == 0 {
                4i32
            } else {
                2i32
            };
            for dx in 0..tick_len {
                draw::dot_i(grid, tube_right + 2 + dx, ty as i32);
            }
        }

        // Mercury tint: cool blue at empty → hot red at full via palette.
        let (cw, ch) = grid.dimensions();
        let mercury_top_cell = (mercury_top / 4).min(ch.saturating_sub(1));
        let bulb_top_cell = (bulb_top_y / 4).min(ch.saturating_sub(1));
        for cy_c in mercury_top_cell..=bulb_top_cell.min(ch.saturating_sub(1)) {
            let color = ctx.palette.sample(ctx.eased);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
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
    let styles = progress::styles::weather::styles();
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
