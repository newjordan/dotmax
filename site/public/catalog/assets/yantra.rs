//! `yantra` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O yantra.rs && ./yantra [style-name]
//! ```

const DEFAULT_STYLE: &str = "sri-yantra";

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
    pub mod yantra {
//! Yantra / Mandala sacred-geometry progress styles.
//!
//! Eleven structurally distinct radially symmetric styles drawn entirely with
//! braille dots via `draw::dot_i` and the Bresenham line helper defined below.
//! Each style has a unique construction principle:
//!
//! - `sri-yantra`            — interlocking Shiva/Shakti triangles in a circle within the bhupura gate
//! - `lotus-mandala`         — eight outer and eight inner pointed petals around a seed pod
//! - `rose-window`           — a row of Gothic roses: rim, hub, lancet mullions, foiled oculi
//! - `bagua`                 — eight trigrams in Fu Xi order either side of a turning yin-yang
//! - `enneagram`             — 9-point star (1-4-2-8-5-7 cycle + triangle)
//! - `rangoli`               — a row of kolams: pulli-dot diamonds looped by a travelling pen
//! - `chartres-labyrinth`    — floor labyrinth in perspective, a walker tracing its circuits
//! - `mandala-tessellation`  — edge-sharing hexagonal mandala tiles filled tile by tile
//! - `dharma-wheel`          — 8-spoked knobbed wheel rolling along its path
//! - `vesica-rosette`        — a row of 6-petal compass rosettes cut by vesica arcs
//! - `star-of-david`         — two interlocking equilateral triangles, inked edge by edge

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — saffron into gold.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 126, 84);
const TINT_END: Color = Color::rgb(255, 204, 64);

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

/// All styles in the `yantra` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per sacred-geometry style. The eleven
/// styles span distinct construction methods — triangle systems, petal arcs,
/// tracery, trigrams, star polygons, kolam loops, labyrinth paths, rotational
/// tilings, spoked wheels, vesica arcs, and nested hexagrams — ensuring no two
/// styles are structurally alike.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(SriYantra)),
        Box::new(Tinted(LotusMandala)),
        Box::new(Tinted(RoseWindow)),
        Box::new(Tinted(Bagua)),
        Box::new(Tinted(Enneagram)),
        Box::new(Tinted(Rangoli)),
        Box::new(Tinted(ChartresLabyrinth)),
        Box::new(Tinted(MandalaTessellation)),
        Box::new(Tinted(DharmaWheel)),
        Box::new(Tinted(VesicaRosette)),
        Box::new(Tinted(StarOfDavid)),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared geometry helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Grid center in dot-space.
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

/// Convert polar coordinates centered at (cx, cy) to dot-space integers.
#[inline]
fn polar(cx: f32, cy: f32, r: f32, angle: f32) -> (i32, i32) {
    let x = cx + r * angle.cos();
    let y = cy - r * angle.sin(); // screen y-axis is flipped
    (x.round() as i32, y.round() as i32)
}

/// Step-bounded Bresenham line between two signed dot-space points.
/// Out-of-bounds dots are silently discarded by `draw::dot_i`.
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x < x1 { 1 } else { -1 };
    let sy: i32 = if y < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a circular arc from `a_start` to `a_end` (radians, CCW) at radius `r`
/// centered at (cx, cy).  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a_start: f32, a_end: f32) {
    if r < 0.5 {
        return;
    }
    let span = (a_end - a_start).abs();
    let steps = ((r * span).ceil() as usize).clamp(4, 1024);
    let mut prev: Option<(i32, i32)> = None;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = a_start + (a_end - a_start) * t;
        let p = polar(cx, cy, r, a);
        if let Some(q) = prev {
            line(grid, q.0, q.1, p.0, p.1);
        }
        prev = Some(p);
    }
}

/// Draw a full circle at radius `r` centered at (cx, cy).
fn circle(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    arc(grid, cx, cy, r, 0.0, 2.0 * PI);
}

/// Draw a triangle through three dot-space points.
fn triangle(grid: &mut BrailleGrid, ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) {
    line(grid, ax, ay, bx, by);
    line(grid, bx, by, cx, cy);
    line(grid, cx, cy, ax, ay);
}

/// Faint dotted ring: the placeholder for a frieze panel not yet reached.
fn dotted_ring(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let n = ((r * 2.0) as usize).clamp(4, 64);
    for k in 0..n {
        let (x, y) = polar(cx, cy, r, 2.0 * PI * k as f32 / n as f32);
        draw::dot_i(grid, x, y);
    }
}

/// Lays a frieze of full-height panels across the strip, filled left to right
/// with progress (the first already part-drawn at the start). The callback
/// gets `(grid, cx, cy, r, local_reveal, index)`; `local_reveal` is 0 for a
/// panel not yet reached. `wf` is the panel pitch in radii, `gap` extra dots.
fn frieze(
    grid: &mut BrailleGrid,
    eased: f32,
    wf: f32,
    gap: f32,
    mut panel: impl FnMut(&mut BrailleGrid, f32, f32, f32, f32, usize),
) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw < 2 || dh < 2 {
        return;
    }
    let r = (dw.min(dh) as f32 / 2.0 - 0.5).max(1.0);
    let pitch = (r * wf + gap).max(2.0);
    let n = ((dw as f32 / pitch) as usize).clamp(1, 64);
    let off = (dw as f32 - n as f32 * pitch) / 2.0;
    let cy = dh as f32 / 2.0 - 0.5;
    let fill = (0.08 + eased.clamp(0.0, 1.0) * 0.92) * n as f32;
    for i in 0..n {
        let cx = off + pitch * (i as f32 + 0.5) - 0.5;
        let lr = (fill - i as f32).clamp(0.0, 1.0);
        panel(grid, cx, cy, r, lr, i);
    }
}

/// Elliptical arc (screen y-down: `a = PI/2` is the bottom of the ellipse).
fn ell_arc(grid: &mut BrailleGrid, cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) {
    let steps = (((rx + ry) * (a1 - a0).abs()).ceil() as usize).clamp(4, 1024);
    for i in 0..=steps {
        let a = a0 + (a1 - a0) * i as f32 / steps as f32;
        draw::dot_i(
            grid,
            (cx + rx * a.cos()).round() as i32,
            (cy + ry * a.sin()).round() as i32,
        );
    }
}

/// Progress rail along row `y`, skipping the figure's span `[skip_lo, skip_hi]`:
/// solid up to `frac` of the width, a dotted path marching on `time` beyond.
fn rail(grid: &mut BrailleGrid, y: i32, skip_lo: i32, skip_hi: i32, frac: f32, time: f32) {
    let (dw, _) = draw::dot_dims(grid);
    let lit = (frac.clamp(0.0, 1.0) * dw as f32).round() as i32;
    let phase = (time * 6.0) as i32;
    for x in 0..dw as i32 {
        if x >= skip_lo && x <= skip_hi {
            continue;
        }
        if x < lit || (x + phase).rem_euclid(3) == 0 {
            draw::dot_i(grid, x, y);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Sri Yantra
// ─────────────────────────────────────────────────────────────────────────────

/// Sri Yantra triangles in unit-circle space (y up): `(base_y, apex_y)`.
/// Down-pointing (Shakti) triangles have their base above the apex.  At 16
/// dots tall the full nine collapse into a smudge, so the five outermost —
/// the ones that give the yantra its silhouette — are drawn, interlocking.
const SRI_TRIS: [(f32, f32); 5] = [
    (0.80, -1.00), // Shakti 1 — apex on the circle
    (-0.80, 1.00), // Shiva 1 — apex on the circle
    (0.50, -0.62), // Shakti 2
    (-0.50, 0.62), // Shiva 2
    (0.12, -0.30), // Shakti 3 — cradling the bindu
];

struct SriYantra;
impl ProgressStyle for SriYantra {
    fn name(&self) -> &str {
        "sri-yantra"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Sri Yantra: square gate with four doors, enclosing circle, and the \
         interlocking Shiva (up) and Shakti (down) triangles laid one by one around \
         a pulsing bindu, flanked by a progress path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let half = (dh as f32 / 2.0 - 0.5).max(1.0);
        let r = (half - 1.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);

        // Bhupura: the square gate, a door gap centred on each side.
        let (x0, x1) = ((cx - half).round() as i32, (cx + half).round() as i32);
        let (y0, y1) = ((cy - half).round() as i32, (cy + half - 1.0).round() as i32);
        let door = (half * 0.3).round() as i32;
        let (mx, my) = (cx.round() as i32, cy.round() as i32);
        for x in x0..=x1 {
            if (x - mx).abs() > door {
                draw::dot_i(grid, x, y0);
                draw::dot_i(grid, x, y1);
            }
        }
        for y in y0..=y1 {
            if (y - my).abs() > door {
                draw::dot_i(grid, x0, y);
                draw::dot_i(grid, x1, y);
            }
        }
        rail(grid, my, x0 - 2, x1 + 2, e, ctx.time);

        // Enclosing circle, then the triangles in sequence (outer first).
        circle(grid, cx, cy, r);
        let n = ((0.15 + e * 1.3).min(1.0) * SRI_TRIS.len() as f32).ceil() as usize;
        for &(by, ay) in SRI_TRIS.iter().take(n) {
            let hw = ((ay - by).abs() * 0.78).min((1.0 - by * by).max(0.0).sqrt());
            let bx_l = (cx - hw * r).round() as i32;
            let bx_r = (cx + hw * r).round() as i32;
            let byd = (cy - by * r).round() as i32;
            let ayd = (cy - ay * r).round() as i32;
            triangle(grid, bx_l, byd, bx_r, byd, mx, ayd);
        }

        // Bindu: the seed point, breathing on time.
        if (ctx.time * 2.0).sin() > -0.3 {
            draw::dot_i(grid, mx, my);
            draw::dot_i(grid, mx - 1, my);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Lotus Mandala
// ─────────────────────────────────────────────────────────────────────────────

struct LotusMandala;
impl ProgressStyle for LotusMandala {
    fn name(&self) -> &str {
        "lotus-mandala"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Lotus mandala: eight pointed outer petals and eight inner petals around a \
         seed-pod ring, opening petal by petal and slowly turning between two \
         progress paths"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let big = (dh as f32 / 2.0 - 0.5).max(1.0);
        let pod = (big * 0.28).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        let half = big.ceil() as i32 + 2;
        rail(
            grid,
            cy.round() as i32,
            cx as i32 - half,
            cx as i32 + half,
            e,
            ctx.time,
        );
        circle(grid, cx, cy, pod);

        // A petal: two sides swelling out from the pod and meeting in a point.
        let petal = |grid: &mut BrailleGrid, axis: f32, len: f32, width: f32| {
            let steps = (len * 3.0).ceil() as usize + 2;
            for side in [-1.0f32, 1.0] {
                let mut prev = polar(cx, cy, pod, axis + side * width * 0.6);
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let r = pod + (len - pod) * t;
                    let spread = width * (PI * t).sin().max(0.0).powf(0.7) * (1.0 - t * 0.35);
                    let p = polar(cx, cy, r, axis + side * spread / r.max(1.0) * pod * 1.6);
                    line(grid, prev.0, prev.1, p.0, p.1);
                    prev = p;
                }
            }
        };
        let spin = ctx.time * 0.15 + PI / 2.0;
        // Petals open in turn: eight outer, then eight inner between them.
        let open = (0.15 + e * 1.4).min(1.0) * 16.0;
        for k in 0..16 {
            if (k as f32) >= open {
                break;
            }
            if k < 8 {
                petal(grid, spin + k as f32 * PI / 4.0, big, 0.55);
            } else {
                let axis = spin + (k - 8) as f32 * PI / 4.0 + PI / 8.0;
                petal(grid, axis, big * 0.62, 0.45);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Rose Window (Gothic cathedral tracery)
// ─────────────────────────────────────────────────────────────────────────────

struct RoseWindow;
impl ProgressStyle for RoseWindow {
    fn name(&self) -> &str {
        "rose-window"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Rose windows: a row of Gothic roses, each a stone rim and hub with \
         eight radiating lancet mullions and a ring of foiled oculi between \
         them, the tracery glazed window by window as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx.eased, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            let rot = time * 0.12 + i as f32 * 0.2;
            circle(g, cx, cy, r);
            let hub = (r * 0.26).max(1.0);
            circle(g, cx, cy, hub);
            // Lancets: eight mullions radiating from hub to rim.
            let spokes = ((lr * 1.6).min(1.0) * 8.0).ceil() as usize;
            for k in 0..spokes.min(8) {
                let a = rot + k as f32 * PI / 4.0;
                let (x0, y0) = polar(cx, cy, hub, a);
                let (x1, y1) = polar(cx, cy, r, a);
                line(g, x0, y0, x1, y1);
            }
            // Foils: an oculus in each light between the mullions.
            let foils = (((lr - 0.4) / 0.6).clamp(0.0, 1.0) * 8.0).ceil() as usize;
            for k in 0..foils.min(8) {
                let a = rot + (k as f32 + 0.5) * PI / 4.0;
                let fx = cx + r * 0.64 * a.cos();
                let fy = cy - r * 0.64 * a.sin();
                circle(g, fx, fy, (r * 0.2).max(1.0));
            }
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Bagua (Eight Trigrams)
// ─────────────────────────────────────────────────────────────────────────────

struct Bagua;
impl ProgressStyle for Bagua {
    fn name(&self) -> &str {
        "bagua"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Bagua: the eight I-Ching trigrams in Fu Xi order (☰ ☱ ☲ ☳ | ☴ ☵ ☶ ☷), \
         solid and broken bars appearing one by one either side of a turning \
         yin-yang"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);

        // Taiji: outer circle, the S of two half-circles, and the two eyes,
        // all turning on time.
        let spin = ctx.time * 0.8;
        circle(grid, cx, cy, r);
        let (c1x, c1y) = (
            cx + r / 2.0 * (spin + PI / 2.0).cos(),
            cy - r / 2.0 * (spin + PI / 2.0).sin(),
        );
        let (c2x, c2y) = (
            cx + r / 2.0 * (spin - PI / 2.0).cos(),
            cy - r / 2.0 * (spin - PI / 2.0).sin(),
        );
        arc(grid, c1x, c1y, r / 2.0, spin - PI / 2.0, spin + PI / 2.0);
        arc(
            grid,
            c2x,
            c2y,
            r / 2.0,
            spin + PI / 2.0,
            spin + 3.0 * PI / 2.0,
        );
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            draw::dot_i(grid, c1x.round() as i32 + dx, c1y.round() as i32 + dy);
        }
        circle(grid, c2x, c2y, (r * 0.15).max(1.0));

        // Fu Xi sequence, lines listed bottom → top (true = solid yang).
        const TRIGRAMS: [[bool; 3]; 8] = [
            [true, true, true],    // ☰ Qian — heaven
            [true, true, false],   // ☱ Dui — lake
            [true, false, true],   // ☲ Li — fire
            [true, false, false],  // ☳ Zhen — thunder
            [false, true, true],   // ☴ Xun — wind
            [false, true, false],  // ☵ Kan — water
            [false, false, true],  // ☶ Gen — mountain
            [false, false, false], // ☷ Kun — earth
        ];
        // Four trigrams each side of the taiji, sized to the free width.
        let side = ((cx - r - 3.0) / 4.0).max(3.0);
        let bar_w = (side * 0.75).round().max(3.0) as i32;
        let pitch = (dh as f32 / 4.0).max(1.0);
        let shown = ((0.1 + ctx.eased.clamp(0.0, 1.0) * 1.3).min(1.0) * 8.0).ceil() as usize;
        for (i, tri) in TRIGRAMS.iter().enumerate().take(shown.min(8)) {
            let slot = i as f32;
            let x0 = if i < 4 {
                (cx - r - 3.0 - (4.0 - slot) * side + (side - bar_w as f32) / 2.0).round() as i32
            } else {
                (cx + r + 3.0 + (slot - 4.0) * side + (side - bar_w as f32) / 2.0).round() as i32
            };
            for (l, &solid) in tri.iter().enumerate() {
                // Bottom line lowest on screen.
                let y = (cy + (1.0 - l as f32) * pitch).round() as i32;
                let gap = bar_w / 3;
                for x in 0..bar_w {
                    let mid = x >= gap && x < bar_w - gap;
                    if solid || !mid {
                        draw::dot_i(grid, x0 + x, y);
                        draw::dot_i(grid, x0 + x, y + 1);
                    }
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Enneagram
// ─────────────────────────────────────────────────────────────────────────────

struct Enneagram;
impl ProgressStyle for Enneagram {
    fn name(&self) -> &str {
        "enneagram"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Enneagram: nine points on a circle, the 9-3-6 triangle and the \
         1-4-2-8-5-7 hexad traced line by line with progress, a spark running \
         the hexad's cycle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        let half = r.ceil() as i32 + 2;
        rail(
            grid,
            cy.round() as i32,
            cx as i32 - half,
            cx as i32 + half,
            e,
            ctx.time,
        );
        circle(grid, cx, cy, r);
        // Point 9 at the top, numbering clockwise.
        let pt = |n: usize| polar(cx, cy, r, PI / 2.0 - n as f32 * 2.0 * PI / 9.0);
        let seq: [(usize, usize); 9] = [
            (9, 3),
            (3, 6),
            (6, 9),
            (1, 4),
            (4, 2),
            (2, 8),
            (8, 5),
            (5, 7),
            (7, 1),
        ];
        let n = ((0.15 + e * 1.3).min(1.0) * seq.len() as f32).ceil() as usize;
        for &(a, b) in seq.iter().take(n) {
            let (ax, ay) = pt(a);
            let (bx, by) = pt(b);
            line(grid, ax, ay, bx, by);
        }
        // Spark running the hexad 1 → 4 → 2 → 8 → 5 → 7 → 1.
        let run = (ctx.time * 0.9).rem_euclid(6.0);
        let k = (run as usize).min(5);
        let f = run - k as f32;
        let (ax, ay) = pt(seq[3 + k].0);
        let (bx, by) = pt(seq[3 + k].1);
        let sx = ax + ((bx - ax) as f32 * f).round() as i32;
        let sy = ay + ((by - ay) as f32 * f).round() as i32;
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            draw::dot_i(grid, sx + dx, sy + dy);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Rangoli / Kolam
// ─────────────────────────────────────────────────────────────────────────────

struct Rangoli;
impl ProgressStyle for Rangoli {
    fn name(&self) -> &str {
        "rangoli"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Rangoli kolam: a row of kolams, each a diamond of nine pulli dots \
         with a loop drawn around every dot by a travelling pen, then framed \
         in a diamond border, kolam after kolam as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // Pen order: centre, then around the ring.
        const PULLI: [(i32, i32); 9] = [
            (0, 0),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        let time = ctx.time;
        frieze(grid, ctx.eased, 2.0, 3.0, |g, cx, cy, r, lr, _| {
            let u = r * 0.38;
            let lrad = (u * 0.62).max(0.8);
            for &(a, b) in &PULLI {
                let px = cx + (a - b) as f32 * u;
                let py = cy - (a + b) as f32 * u;
                draw::dot_i(g, px.round() as i32, py.round() as i32);
            }
            if lr <= 0.0 {
                return;
            }
            let pen = (lr * 1.25).min(1.0) * PULLI.len() as f32;
            for (k, &(a, b)) in PULLI.iter().enumerate() {
                let f = (pen - k as f32).clamp(0.0, 1.0);
                if f <= 0.0 {
                    break;
                }
                let px = cx + (a - b) as f32 * u;
                let py = cy - (a + b) as f32 * u;
                let start = PI / 2.0 + k as f32 * 0.7;
                arc(g, px, py, lrad, start, start + 2.0 * PI * f);
                if f < 1.0 && (time * 6.0).sin() > -0.5 {
                    // The pen tip, chalk on the ground.
                    let (tx, ty) = polar(px, py, lrad, start + 2.0 * PI * f);
                    draw::dot_i(g, tx + 1, ty);
                }
            }
            if lr >= 0.8 {
                let d = 2.0 * u + lrad + 0.6;
                let (l, rt) = ((cx - d).round() as i32, (cx + d).round() as i32);
                let (t, b) = ((cy - d).round() as i32, (cy + d).round() as i32);
                let (mx, my) = (cx.round() as i32, cy.round() as i32);
                line(g, l, my, mx, t);
                line(g, mx, t, rt, my);
                line(g, rt, my, mx, b);
                line(g, mx, b, l, my);
            }
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Chartres Labyrinth
// ─────────────────────────────────────────────────────────────────────────────

struct ChartresLabyrinth;
impl ProgressStyle for ChartresLabyrinth {
    fn name(&self) -> &str {
        "chartres-labyrinth"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Chartres labyrinth: the cathedral floor labyrinth seen in perspective \
         — nested circuit walls with their turning gaps — while a walker \
         traces the single path circuit by circuit to the centre, a dotted \
         trail behind and the progress path either side"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let e = ctx.eased.clamp(0.0, 1.0);
        let cx = dw as f32 / 2.0 - 0.5;
        let cy = dh as f32 / 2.0 - 0.5;
        let ry0 = (dh as f32 / 2.0 - 0.5).max(1.0);
        // Floor seen at an angle: the circle widens into an ellipse.
        let k = 2.4f32.min((dw as f32 / 2.0 - 0.5).max(1.0) / ry0);
        let s = ry0 / 3.75;
        let (g, h) = (0.42f32, 0.2f32);
        let span = (ry0 * k).round() as i32;
        let mx = cx.round() as i32;
        rail(
            grid,
            cy.round() as i32,
            mx - span - 2,
            mx + span + 2,
            e,
            ctx.time,
        );

        // Walls: outer to goal, each with its turning gap near the bottom.
        let gaps = [PI / 2.0, PI / 2.0 - g, PI / 2.0 + g, PI / 2.0 - g];
        for (j, &gc) in gaps.iter().enumerate() {
            let ry = ry0 - j as f32 * s;
            if ry < 0.8 {
                break;
            }
            ell_arc(grid, cx, cy, ry * k, ry, gc + h, gc + 2.0 * PI - h);
        }

        // The path: three circuits, reversing direction at each turn.
        let segs: [(f32, f32, f32); 3] = [
            (ry0 - 0.5 * s, PI / 2.0, PI / 2.0 + 2.0 * PI - g),
            (ry0 - 1.5 * s, PI / 2.0 - g, PI / 2.0 + g - 2.0 * PI),
            (ry0 - 2.5 * s, PI / 2.0 + g, PI / 2.0 - g + 2.0 * PI),
        ];
        let lens: Vec<f32> = segs
            .iter()
            .map(|&(ry, a0, a1)| ry * (1.0 + k) * 0.5 * (a1 - a0).abs())
            .collect();
        let total: f32 = lens.iter().sum::<f32>().max(1.0);
        let mut walk = (0.05 + e * 0.95) * total;
        let mut head = (cx, cy);
        for (&(ry, a0, a1), &len) in segs.iter().zip(lens.iter()) {
            if walk <= 0.0 {
                break;
            }
            let f = (walk / len.max(0.1)).min(1.0);
            walk -= len;
            let steps = ((len * f) as usize).clamp(1, 512);
            for i in 0..=steps {
                let a = a0 + (a1 - a0) * f * i as f32 / steps as f32;
                let (x, y) = (cx + ry * k * a.cos(), cy + ry * a.sin());
                if i % 3 == 0 {
                    draw::dot_i(grid, x.round() as i32, y.round() as i32);
                }
                head = (x, y);
            }
        }
        if walk > 0.0 {
            head = (cx, cy);
        }
        // The walker: a small bright figure.
        let (hx, hy) = (head.0.round() as i32, head.1.round() as i32);
        draw::dot_i(grid, hx, hy);
        draw::dot_i(grid, hx + 1, hy);
        if (ctx.time * 4.0).sin() > 0.0 {
            draw::dot_i(grid, hx, hy - 1);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Mandala Tessellation
// ─────────────────────────────────────────────────────────────────────────────

struct MandalaTessellation;
impl ProgressStyle for MandalaTessellation {
    fn name(&self) -> &str {
        "mandala-tessellation"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Mandala tessellation: a row of edge-sharing hexagonal tiles, each \
         laid out with six spokes, a hexagram and a turning inner hexagon — \
         the tiling fills tile by tile as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        let wf = 3f32.sqrt();
        frieze(grid, ctx.eased, wf, 0.0, |g, cx, cy, r, lr, i| {
            // Pointy-top hexagon tile; neighbours share their vertical edges.
            let v: Vec<(i32, i32)> = (0..6)
                .map(|k| polar(cx, cy, r, PI / 2.0 + k as f32 * PI / 3.0))
                .collect();
            for k in 0..6 {
                let (a, b) = (v[k], v[(k + 1) % 6]);
                line(g, a.0, a.1, b.0, b.1);
            }
            if lr <= 0.0 {
                return;
            }
            let spokes = ((lr * 1.5).min(1.0) * 6.0).ceil() as usize;
            for &(x, y) in v.iter().take(spokes.min(6)) {
                line(g, cx.round() as i32, cy.round() as i32, x, y);
            }
            if lr > 0.5 {
                // Hexagram through alternate edge midpoints.
                let m: Vec<(i32, i32)> = (0..6)
                    .map(|k| polar(cx, cy, r * 0.84, k as f32 * PI / 3.0))
                    .collect();
                for k in 0..6 {
                    let (a, b) = (m[k], m[(k + 2) % 6]);
                    line(g, a.0, a.1, b.0, b.1);
                }
            }
            if lr >= 1.0 {
                let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                let rot = dir * time * 0.4;
                let w: Vec<(i32, i32)> = (0..6)
                    .map(|k| polar(cx, cy, r * 0.38, rot + k as f32 * PI / 3.0))
                    .collect();
                for k in 0..6 {
                    let (a, b) = (w[k], w[(k + 1) % 6]);
                    line(g, a.0, a.1, b.0, b.1);
                }
            }
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Dharma Wheel
// ─────────────────────────────────────────────────────────────────────────────

struct DharmaWheel;
impl ProgressStyle for DharmaWheel {
    fn name(&self) -> &str {
        "dharma-wheel"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Dharma wheel (Dharmachakra): rim, nave and eight knobbed spokes — the \
         wheel of the law rolling along its path as progress turns it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let cy = dh as f32 / 2.0 - 0.5;
        let knob = (dh as f32 / 2.0 - 0.5).max(1.0);
        let r = (knob - 1.2).max(1.0);
        // The wheel rolls along its path: travel sets both place and turn.
        let span = (dw as f32 - 2.0 * knob - 1.0).max(0.0);
        let cx = knob + ctx.eased.clamp(0.0, 1.0) * span;
        let turn = -(cx / r) - ctx.time * 0.6;
        let ground = (cx - knob - 1.0).round() as i32;
        for x in 0..ground.max(0) {
            if x % 2 == 0 {
                draw::dot_i(grid, x, cy.round() as i32);
            }
        }
        // Rim and nave.
        circle(grid, cx, cy, r);
        circle(grid, cx, cy, (r * 0.28).max(1.0));
        // Eight spokes, each ending in a knob just outside the rim.
        for i in 0..8 {
            let a = turn + i as f32 * PI / 4.0;
            let (hx, hy) = polar(cx, cy, r * 0.28, a);
            let (rx, ry) = polar(cx, cy, r, a);
            line(grid, hx, hy, rx, ry);
            let (kx, ky) = polar(cx, cy, knob, a);
            draw::dot_i(grid, kx, ky);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Vesica Rosette
// ─────────────────────────────────────────────────────────────────────────────

struct VesicaRosette;
impl ProgressStyle for VesicaRosette {
    fn name(&self) -> &str {
        "vesica-rosette"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Vesica rosettes: a row of six-petal compass rosettes, each petal a \
         vesica lens cut by two arcs of the circle's own radius, petals \
         blooming one by one and rosette by rosette as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx.eased, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            circle(g, cx, cy, r);
            let rot = time * 0.15 + i as f32 * PI / 6.0;
            // Each arc is centred on the rim and passes through the centre;
            // six of them cut six vesica petals.
            let arcs = (lr * 6.0).ceil() as usize;
            for k in 0..arcs.min(6) {
                let a = rot + k as f32 * PI / 3.0;
                let ox = cx + r * a.cos();
                let oy = cy - r * a.sin();
                let back = a + PI;
                arc(g, ox, oy, r, back - PI / 3.0, back + PI / 3.0);
            }
            draw::dot_i(g, cx.round() as i32, cy.round() as i32);
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Star of David / Hexagram
// ─────────────────────────────────────────────────────────────────────────────

struct StarOfDavid;
impl ProgressStyle for StarOfDavid {
    fn name(&self) -> &str {
        "star-of-david"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Star of David hexagram: two interlocking equilateral triangles inked \
         edge by edge and slowly turning, on a progress path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        // Progress path either side of the star.
        let half_w = (r * 0.9).ceil() as i32 + 2;
        rail(
            grid,
            cy.round() as i32,
            cx as i32 - half_w,
            cx as i32 + half_w,
            e,
            ctx.time,
        );
        // Two interlocking equilateral triangles, slowly turning; the six
        // edges are inked in sequence over the first half of the run.
        let spin = ctx.time * 0.35;
        let up = [0, 1, 2].map(|i| polar(cx, cy, r, spin + PI / 2.0 + i as f32 * 2.0 * PI / 3.0));
        let dn = [0, 1, 2].map(|i| polar(cx, cy, r, spin - PI / 2.0 + i as f32 * 2.0 * PI / 3.0));
        let inked = (0.35 + e * 1.4).min(1.0) * 6.0;
        for k in 0..6 {
            let frac = (inked - k as f32).clamp(0.0, 1.0);
            if frac <= 0.0 {
                break;
            }
            let tri = if k % 2 == 0 { &up } else { &dn };
            let i = k / 2;
            let (ax, ay) = tri[i];
            let (bx, by) = tri[(i + 1) % 3];
            let ex = ax + ((bx - ax) as f32 * frac).round() as i32;
            let ey = ay + ((by - ay) as f32 * frac).round() as i32;
            line(grid, ax, ay, ex, ey);
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
    let styles = progress::styles::yantra::styles();
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
