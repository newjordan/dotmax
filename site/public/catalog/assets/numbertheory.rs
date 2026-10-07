//! `numbertheory` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O numbertheory.rs && ./numbertheory [style-name]
//! ```

const DEFAULT_STYLE: &str = "sieve-eratosthenes";

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
    pub mod numbertheory {
//! Number-theory progress bars.
//!
//! Twelve styles, each grounded in a concrete mathematical structure:
//! sieves, spirals, trees, triangles, histograms, and sequence geometry.
//! Every bar maps `ctx.eased` to "how many integers have been processed" and
//! `ctx.time` to a highlight or scan animation running independently of
//! progress. All arithmetic is bounded and cannot panic.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// Trial-division primality test. Returns `false` for 0 and 1.
/// Bounded: for any n ≤ 10_000 this completes in a handful of μs.
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3u64;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

// ────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — teal into deep blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(118, 224, 202);
const TINT_END: Color = Color::rgb(44, 132, 204);

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

/// All styles in the `numbertheory` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Sieve)),
        Box::new(Tinted(UlamSpiral)),
        Box::new(Tinted(PrimeCounting)),
        Box::new(Tinted(Collatz)),
        Box::new(Tinted(FibonacciSpiral)),
        Box::new(Tinted(PascalMod)),
        Box::new(Tinted(TotientHistogram)),
        Box::new(Tinted(SternBrocot)),
        Box::new(Tinted(ContinuedFraction)),
        Box::new(Tinted(ModularCircle)),
        Box::new(Tinted(Recaman)),
        Box::new(Tinted(DigitalRoot)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Sieve of Eratosthenes
// ────────────────────────────────────────────────────────────────────────────

/// Sieve of Eratosthenes: integers 1..N laid left→right; composites are
/// crossed out one by one, primes remain lit. `eased` controls how many
/// integers are visible; `time` sweeps a highlight over the current sieve
/// multiple.
struct Sieve;
impl ProgressStyle for Sieve {
    fn name(&self) -> &str {
        "sieve-eratosthenes"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Sieve of Eratosthenes: integers cross composites, primes survive as lit dots"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // N = number of integers we lay across the width (at least 2).
        let n = w.clamp(2, 2000);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n);

        // Build sieve for 1..=n.
        let mut composite = vec![false; n + 1];
        composite[0] = true;
        if n >= 1 {
            composite[1] = true;
        }
        for p in 2..=n {
            if !composite[p] {
                let mut m = p * 2;
                while m <= n {
                    composite[m] = true;
                    m += p;
                }
            }
        }

        // Animated highlight: current sieve multiple being crossed.
        let highlight_p = {
            let p_frac = (ctx.time * 0.4).fract();
            // Pick a prime index from time.
            let prime_count = (2..=n).filter(|&k| !composite[k]).count().max(1);
            let pi = ((p_frac * prime_count as f32) as usize).min(prime_count.saturating_sub(1));
            (2..=n).filter(|&k| !composite[k]).nth(pi).unwrap_or(2)
        };

        // Draw: one dot-column per integer.
        for (k, &comp) in composite.iter().enumerate().skip(1).take(revealed) {
            let x = ((k - 1) * w / n).min(w.saturating_sub(1));
            let is_p = !comp;
            // Primes: full column; composites: half-height bottom tick.
            if is_p {
                draw::vline(grid, x, 0, h.saturating_sub(1));
                // Highlight the current sieve prime's multiples.
                if k > 1 && k % highlight_p == 0 {
                    // draw a top cap to distinguish the animated sweep target.
                    draw::dot(grid, x, 0);
                }
            } else {
                let tick = (h / 4).max(1);
                draw::vline(grid, x, h.saturating_sub(tick), h.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Ulam Spiral
// ────────────────────────────────────────────────────────────────────────────

/// Ulam spiral, stretched to the bar: the integers wind outward from a short
/// central run in a rectangular spiral that exactly tiles the bar, and only
/// the primes are lit, so the diagonal streaks of prime-rich quadratics show
/// up in every corner. `eased` reveals integers 1..N; `time` blinks the
/// spiral's growing head.
struct UlamSpiral;
impl ProgressStyle for UlamSpiral {
    fn name(&self) -> &str {
        "ulam-spiral"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Ulam spiral: integers wind outward from the centre in a rectangular spiral that tiles \
         the bar, only primes lit — the tell-tale diagonal streaks appear as progress unwinds \
         it, with a blinking head marking the newest integer"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        // Rectangular spiral: horizontal legs a+1, a+2, …; vertical legs 1, 2, …
        // After k full rings it spans (a + 2k) × 2k, so pick a = w − h.
        let a = w.saturating_sub(h) as i32;
        let x0 = (h / 2) as i32 - 1;
        let y0 = (h / 2) as i32;
        let n = (w * h).clamp(1, 6000);
        let revealed = ((ctx.eased.clamp(0.0, 1.0) * n as f32).round() as usize).min(n);

        let (mut x, mut y) = (x0, y0);
        let mut leg = 0i32; // leg index: even = horizontal, odd = vertical
        let mut left = a + 1;
        let mut head = (x, y);
        for i in 1..=revealed {
            if is_prime(i as u64) {
                draw::dot_i(grid, x, y);
            }
            head = (x, y);
            // Advance along the current leg; directions: right, up, left, down.
            let dir = leg % 4;
            match dir {
                0 => x += 1,
                1 => y -= 1,
                2 => x -= 1,
                _ => y += 1,
            }
            left -= 1;
            if left <= 0 {
                leg += 1;
                let k = leg / 2 + 1;
                left = if leg % 2 == 0 { a + k } else { k };
            }
        }
        if revealed > 0 && revealed < n && (ctx.time * 4.0).sin() > -0.2 {
            draw::fill_rect(
                grid,
                (head.0 - 1).max(0) as usize,
                (head.1 - 1).max(0) as usize,
                2,
                2,
            );
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Prime counting function π(x)
// ────────────────────────────────────────────────────────────────────────────

/// Prime counting function π(x): a curve showing how many primes are ≤ x,
/// displayed as a rising step graph filling left to right with `eased`.
/// `time` scrolls a thin vertical scan line.
struct PrimeCounting;
impl ProgressStyle for PrimeCounting {
    fn name(&self) -> &str {
        "prime-counting"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "π(x) prime-counting function: a rising step curve filling as primes accumulate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n = w.clamp(2, 3000);
        let revealed_x = ((ctx.eased * n as f32).round() as usize).min(n);

        // Precompute π(k) for k in 1..=n.
        let mut pi = 0usize;
        let mut prime_counts = vec![0usize; n + 1];
        for (k, count) in prime_counts.iter_mut().enumerate().skip(1) {
            if is_prime(k as u64) {
                pi += 1;
            }
            *count = pi;
        }
        let pi_max = prime_counts[n].max(1);

        // Draw the step curve up to revealed_x.
        for (k, &count) in prime_counts.iter().enumerate().skip(1).take(revealed_x) {
            let x = ((k - 1) * w / n).min(w.saturating_sub(1));
            // Map prime count to y (bottom = 0 primes, top = pi_max).
            let bar_h = (count * h / pi_max).min(h);
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
        }

        // Animated scan line.
        let scan_x = ((ctx.time * 0.3).fract() * w as f32) as usize;
        if scan_x < w {
            draw::vline(grid, scan_x, 0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Collatz (3n+1) trajectories
// ────────────────────────────────────────────────────────────────────────────

/// Collatz conjecture: for each seed n (swept by `eased` over 1..N), plot
/// the stopping trajectory (n→n/2 if even, 3n+1 if odd). Height encodes
/// the value normalised to the peak value in the window. `time` selects
/// which trajectory is currently highlighted.
struct Collatz;
impl ProgressStyle for Collatz {
    fn name(&self) -> &str {
        "collatz"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Collatz (3n+1): seeds swept left→right, trajectory height is stopping sequence depth"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of seeds = width in dots, max 500.
        let n_seeds = w.clamp(1, 500);
        let revealed = ((ctx.eased * n_seeds as f32).round() as usize).min(n_seeds);

        // Collatz stopping time for seed k.
        fn stopping_time(mut k: u64) -> u64 {
            let mut count = 0u64;
            while k != 1 && count < 10_000 {
                k = if k % 2 == 0 { k / 2 } else { 3 * k + 1 };
                count += 1;
            }
            count
        }

        // Collect stopping times for seeds 1..=n_seeds.
        let times: Vec<u64> = (1..=n_seeds).map(|k| stopping_time(k as u64)).collect();
        let max_t = times.iter().copied().max().unwrap_or(1).max(1);

        // Highlight column from time.
        let hi_x = ((ctx.time * 0.7).fract() * revealed as f32) as usize;

        for (i, &t) in times.iter().enumerate().take(revealed) {
            let x = (i * w / n_seeds).min(w.saturating_sub(1));
            let bar_h = ((t as f32 / max_t as f32) * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
            // Extra top dot for highlight column.
            if i == hi_x && y0 > 0 {
                draw::dot(grid, x, y0.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Fibonacci / Golden spiral
// ────────────────────────────────────────────────────────────────────────────

/// Fibonacci golden spiral: squares whose sides are Fibonacci numbers, each
/// crossed by a quarter-circle arc. `eased` builds the tiling square by square;
/// `time` runs a bead along finished spirals.
struct FibonacciSpiral;

/// A point in Fibonacci-tiling units.
type Pt = (f32, f32);

/// The Fibonacci tiling used by [`FibonacciSpiral`]: squares of side
/// 1, 1, 2, 3, 5, 8 (top-left corner, side) filling a 13×8 golden-ish
/// rectangle spanning x ∈ [-3, 10], y ∈ [-5, 3] (y down), and for each square
/// the quarter arc of the spiral through it as (centre, start corner).
const FIB_SQUARES: [(Pt, f32, Pt, Pt); 6] = [
    ((0.0, 0.0), 1.0, (1.0, 1.0), (0.0, 1.0)),
    ((1.0, 0.0), 1.0, (1.0, 1.0), (1.0, 0.0)),
    ((0.0, 1.0), 2.0, (0.0, 1.0), (2.0, 1.0)),
    ((-3.0, 0.0), 3.0, (0.0, 0.0), (0.0, 3.0)),
    ((-3.0, -5.0), 5.0, (2.0, 0.0), (-3.0, 0.0)),
    ((2.0, -5.0), 8.0, (2.0, 3.0), (2.0, -5.0)),
];

impl ProgressStyle for FibonacciSpiral {
    fn name(&self) -> &str {
        "fibonacci-spiral"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Fibonacci spiral: squares of side 1, 1, 2, 3, 5, 8 tile a 13×8 rectangle and a \
         quarter arc sweeps through each; three tilings build square by square as progress \
         rises, and a bead rides the finished spirals with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let n = if dw >= dh * 4 { 3 } else { 1 };
        let pw = dw as f32 / n as f32;
        let s = ((dh as f32 - 1.0) / 8.0).min((pw - 2.0) / 13.0).max(0.1);
        for k in 0..n {
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            // Panel origin so the 13×8 rectangle is centred in its slot.
            let ox = pw * (k as f32 + 0.5) - 6.5 * s + 3.0 * s;
            let oy = (dh as f32 - 1.0 - 8.0 * s) * 0.5 + 5.0 * s;
            let at = |x: f32, y: f32| ((ox + x * s).round() as i32, (oy + y * s).round() as i32);
            let span = local * 6.0;
            let bead = if local >= 1.0 {
                Some((ctx.time * 0.8 + k as f32).rem_euclid(6.0))
            } else {
                None
            };
            for (i, &((x, y), side, (cx, cy), (px, py))) in FIB_SQUARES.iter().enumerate() {
                let part = (span - i as f32).clamp(0.0, 1.0);
                if part <= 0.0 {
                    break;
                }
                // Square outline.
                let (x0, y0) = at(x, y);
                let (x1, y1) = at(x + side, y + side);
                for xx in x0..=x1 {
                    draw::dot_i(grid, xx, y0);
                    draw::dot_i(grid, xx, y1);
                }
                for yy in y0..=y1 {
                    draw::dot_i(grid, x0, yy);
                    draw::dot_i(grid, x1, yy);
                }
                // Quarter arc from the start corner, swept by `part`.
                let a0 = (py - cy).atan2(px - cx);
                // The end corner is diagonally opposite the start corner.
                let (qx, qy) = (2.0 * x + side - px, 2.0 * y + side - py);
                let mut da = (qy - cy).atan2(qx - cx) - a0;
                if da > PI {
                    da -= 2.0 * PI;
                } else if da < -PI {
                    da += 2.0 * PI;
                }
                let steps = (side * s * 2.0).ceil().max(4.0) as u32;
                for j in 0..=steps {
                    let a = a0 + da * part * j as f32 / steps as f32;
                    let (dx, dy) = at(cx + side * a.cos(), cy + side * a.sin());
                    draw::dot_i(grid, dx, dy);
                }
                if let Some(b) = bead {
                    if (b - i as f32) >= 0.0 && (b - i as f32) < 1.0 {
                        let a = a0 + da * (b - i as f32);
                        let (dx, dy) = at(cx + side * a.cos(), cy + side * a.sin());
                        draw::dot_i(grid, dx + 1, dy);
                        draw::dot_i(grid, dx, dy + 1);
                        draw::dot_i(grid, dx - 1, dy);
                        draw::dot_i(grid, dx, dy - 1);
                    }
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Pascal's triangle mod p
// ────────────────────────────────────────────────────────────────────────────

/// Pascal's triangle mod p, on its side: rows revealed = `eased × width`.
/// Mod 2 yields Sierpinski's gasket; `time` cycles through mod 2, 3, 5 to
/// show different self-similar patterns.
struct PascalMod;

/// `C(n, k) mod p != 0`, by Lucas' theorem: true iff every base-`p` digit of
/// `k` is ≤ the matching digit of `n`.
fn binom_nonzero_mod(mut n: u32, mut k: u32, p: u32) -> bool {
    if k > n {
        return false;
    }
    while k > 0 {
        if k % p > n % p {
            return false;
        }
        n /= p;
        k /= p;
    }
    true
}

impl ProgressStyle for PascalMod {
    fn name(&self) -> &str {
        "pascal-mod"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Pascal's triangle mod p laid on its side: row n runs along the bar, the middle \
         binomials C(n, k) stack up the height, and every entry not divisible by p is lit — \
         nested triangular holes open up as progress adds rows; p cycles 2, 3, 5 with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let p = [2u32, 3, 5][((ctx.time.max(0.0) / 6.0) as usize) % 3];
        let rows = (ctx.eased * dw as f32).ceil() as usize;
        let mid = dh as i32 / 2;
        for n in 0..rows.min(dw) {
            let nn = n as u32;
            for y in 0..dh as i32 {
                // Middle of row n sits on the bar's centre line.
                let k = (nn as i32 + 1) / 2 + (y - mid);
                if k >= 0 && binom_nonzero_mod(nn, k as u32, p) {
                    draw::dot(grid, n, y as usize);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Euler totient φ(n) histogram
// ────────────────────────────────────────────────────────────────────────────

/// Euler totient φ(n) histogram: bar height at column n is φ(n)/n, revealing
/// left to right as `eased` grows. Primes peek at the top (φ(p)=p−1≈p).
struct TotientHistogram;
impl ProgressStyle for TotientHistogram {
    fn name(&self) -> &str {
        "totient-histogram"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Euler totient φ(n): columns show φ(n)/n — primes spike to the top, composites sag"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n = w.clamp(2, 2000);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n).max(1);

        // Compute totient for k in 2..=revealed.
        fn totient(n: usize) -> usize {
            if n == 0 {
                return 0;
            }
            // Euler's product formula via trial factoring.
            let mut phi = n;
            let mut temp = n;
            let mut p = 2;
            while p * p <= temp {
                if temp % p == 0 {
                    while temp % p == 0 {
                        temp /= p;
                    }
                    phi = phi - phi / p;
                }
                p += 1;
            }
            if temp > 1 {
                phi = phi - phi / temp;
            }
            phi
        }

        for k in 2..=revealed {
            let x = ((k - 2) * w / (n.saturating_sub(1).max(1))).min(w.saturating_sub(1));
            let phi_k = totient(k);
            // Ratio φ(k)/k ∈ (0, 1].
            let ratio = phi_k as f32 / k as f32;
            let bar_h = (ratio * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Stern–Brocot tree (mediant binary tree)
// ────────────────────────────────────────────────────────────────────────────

/// Stern–Brocot tree: every positive rational appears exactly once as the
/// mediant (a+c)/(b+d) of its two neighbours. Nodes sit at their value
/// in order along their row (the tree is a binary search tree) on rows by depth, joined to their
/// parents. `eased` grows the tree level by level, left to right; `time`
/// walks a blinking path down the tree towards the golden ratio.
struct SternBrocot;
impl ProgressStyle for SternBrocot {
    fn name(&self) -> &str {
        "stern-brocot"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Stern–Brocot tree: 1/1 at the top, each node's children the mediants with its \
         neighbours (1/2, 2/1, 1/3, 2/3, 3/2, 3/1…), in sorted order — the binary tree of all \
         positive rationals grows level by level with progress while a walker blinks down \
         the path that converges on the golden ratio"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let levels = (h / 3).clamp(1, 7);
        let row = |d: usize| -> i32 {
            if levels <= 1 {
                0
            } else {
                (d as f32 * (h as f32 - 1.0) / (levels - 1) as f32).round() as i32
            }
        };
        // The tree is a binary search tree on value, so laying level d out
        // in order (slot i of 2^d) keeps every fraction sorted left to right.
        let col = |d: usize, i: usize| -> i32 {
            let f = (2 * i + 1) as f32 / (1u32 << (d + 1)) as f32;
            (f * (w as f32 - 1.0)).round() as i32
        };
        let line = |grid: &mut BrailleGrid, a: (i32, i32), b: (i32, i32)| {
            let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).clamp(1, 512);
            for i in 0..=n {
                draw::dot_i(grid, a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n);
            }
        };

        let stage = ctx.eased.clamp(0.0, 1.0) * levels as f32;
        // Breadth-first: each node carries its bounding fractions (a/b, c/d).
        let mut frontier: Vec<(u32, u32, u32, u32)> = vec![(0, 1, 1, 0)];
        for d in 0..levels {
            let frac = (stage - d as f32).clamp(0.0, 1.0);
            if frac <= 0.0 {
                break;
            }
            let count = frontier.len();
            let shown = ((frac * count as f32).ceil() as usize).min(count);
            let mut next = Vec::with_capacity(count * 2);
            for (i, &(a, b, c, dd)) in frontier.iter().enumerate() {
                let (p, q) = (a + c, b + dd);
                let me = (col(d, i), row(d));
                if i < shown {
                    draw::hline(
                        grid,
                        (me.0 - 1).max(0) as usize,
                        (me.0 + 1).max(0) as usize,
                        me.1.max(0) as usize,
                    );
                    if d > 0 {
                        line(grid, (col(d - 1, i / 2), row(d - 1)), me);
                    }
                }
                next.push((a, b, p, q));
                next.push((p, q, c, dd));
            }
            frontier = next;
        }

        // Walker: descends towards φ = [1;1,1,…] (alternating R, L, R, L…).
        let depth_shown = (stage.floor() as usize).min(levels);
        if depth_shown > 0 {
            let step = ((ctx.time.max(0.0) * 2.0) as usize) % depth_shown;
            let (mut a, mut b, mut c, mut dd) = (0u32, 1u32, 1u32, 0u32);
            let mut slot = 0usize;
            for d in 0..=step {
                let (p, q) = (a + c, b + dd);
                if d == step {
                    let (x, y) = (col(d, slot), row(d));
                    for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        draw::dot_i(grid, x + ox, y + oy);
                    }
                }
                // φ > p/q ⇔ go right (towards larger values).
                let phi = 1.618_034f32;
                if (p as f32) < phi * q as f32 {
                    a = p;
                    b = q;
                    slot = slot * 2 + 1;
                } else {
                    c = p;
                    dd = q;
                    slot *= 2;
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Continued fraction convergents of φ
// ────────────────────────────────────────────────────────────────────────────

/// Continued fraction convergents of φ = [1;1,1,1,…]: each convergent p_k/q_k
/// is a best rational approximation. `eased` reveals convergents left to
/// right, each plotted by its signed log-error from φ; they alternate above /
/// below the true value — the zig-zag visualises the approximation theorem.
struct ContinuedFraction;
impl ProgressStyle for ContinuedFraction {
    fn name(&self) -> &str {
        "continued-fraction-phi"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Convergents of φ = [1;1,1,…] — 1/1, 2/1, 3/2, 5/3, 8/5… — plotted left to right as \
         a zig-zag that alternates above and below the dotted golden-ratio line and closes in \
         on it (error on a log scale); progress adds convergents, a marker rides the newest"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let phi = (1.0 + 5f64.sqrt()) / 2.0;
        let mid = (h as f32 - 1.0) * 0.5;
        // Dotted reference line at φ.
        for x in (0..w).step_by(2) {
            draw::dot(grid, x, mid.round() as usize);
        }
        let total = 16usize;
        let pitch = (w as f32 - 1.0) / (total - 1) as f32;
        let shown = ((ctx.eased * total as f32).ceil() as usize).clamp(1, total);
        // Signed log-error: convergent k misses φ by ≈ φ^(−2k), alternating sign.
        let (mut p, mut q) = (1u64, 1u64);
        let mut prev: Option<(i32, i32)> = None;
        for k in 0..shown {
            let err = p as f64 / q as f64 - phi;
            let mag = ((err.abs().max(1e-12).ln() + 12.0) / 12.0).clamp(0.0, 1.0) as f32;
            let y = (mid - err.signum() as f32 * mag * mid).round() as i32;
            let x = (k as f32 * pitch).round() as i32;
            if let Some((px, py)) = prev {
                let n = (x - px).abs().max((y - py).abs()).max(1);
                for i in 0..=n {
                    draw::dot_i(grid, px + (x - px) * i / n, py + (y - py) * i / n);
                }
            }
            prev = Some((x, y));
            (p, q) = (p + q, p);
        }
        if let Some((x, y)) = prev {
            // Pulsing marker on the newest convergent.
            if (ctx.time * 3.0).sin() > -0.3 {
                draw::fill_rect(grid, (x - 1).max(0) as usize, (y - 1).max(0) as usize, 3, 3);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Times-table circles (cardioid / nephroid string art)
// ────────────────────────────────────────────────────────────────────────────

/// Times-table chord diagram: N points around a circle, point k joined to
/// point m·k mod N. m=2 traces a cardioid, m=3 a nephroid, m=4 a three-cusped
/// epicycloid and so on. A wide bar becomes a row of small circles, one
/// multiplier each, whose chords (dotted) are strung in turn as progress
/// advances while their envelope — the epicycloid with m−1 cusps — is traced
/// solid; `time` turns the figures slowly.
struct ModularCircle;
impl ProgressStyle for ModularCircle {
    fn name(&self) -> &str {
        "modular-circle"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Times-table chord diagrams: points round a circle, chord k→m·k mod N — a row of \
         circles for m = 2, 3, 4, … strung chord by chord as progress advances, each \
         chord envelope (cardioid, nephroid, three-cusp epicycloid…) traced solid, the \
         figures turning slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 3 || h < 3 {
            return Ok(());
        }
        // Square panels, as many as fit (at least one).
        let side = h.min(w);
        let gap = 2usize;
        let panels = ((w + gap) / (side + gap)).clamp(1, 12);
        let left = (w + gap).saturating_sub(panels * (side + gap)) / 2;
        let n_pts = 18usize;
        let r = (side as f32 - 1.0) * 0.5;
        let rot = ctx.time * 0.25;
        let total = ctx.eased.clamp(0.0, 1.0) * panels as f32;
        for k in 0..panels {
            let m = k + 2;
            let cx = (left + k * (side + gap)) as f32 + r;
            let cy = r;
            let pt = |i: usize| -> (i32, i32) {
                let a = i as f32 / n_pts as f32 * 2.0 * PI + rot - PI * 0.5;
                (
                    (cx + r * a.cos()).round() as i32,
                    (cy + r * a.sin()).round() as i32,
                )
            };
            // Dotted rim marks every panel, filled or waiting.
            for i in (0..n_pts).step_by(2) {
                let (x, y) = pt(i);
                draw::dot_i(grid, x, y);
            }
            let local = (total - k as f32).clamp(0.0, 1.0);
            let shown = (local * n_pts as f32).round() as usize;
            // Chords as dotted threads so the strings stay distinct.
            for i in 1..shown.min(n_pts) {
                let (x0, y0) = pt(i);
                let (x1, y1) = pt((m * i) % n_pts);
                let steps = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 256);
                for s in (0..=steps).step_by(3) {
                    draw::dot_i(grid, x0 + (x1 - x0) * s / steps, y0 + (y1 - y0) * s / steps);
                }
            }
            // The chords' envelope, drawn solid: the epicycloid
            // z(θ) = (m·e^{iθ} + e^{imθ}) / (m+1) with m−1 cusps.
            let env_steps = (local * 96.0) as usize;
            let mf = m as f32;
            let base = rot - PI * 0.5;
            for s in 0..=env_steps.min(96) {
                let th = s as f32 / 96.0 * 2.0 * PI;
                let ex = (mf * th.cos() + (mf * th).cos()) / (mf + 1.0);
                let ey = (mf * th.sin() + (mf * th).sin()) / (mf + 1.0);
                let (bc, bs) = (base.cos(), base.sin());
                let x = cx + r * (ex * bc - ey * bs);
                let y = cy + r * (ex * bs + ey * bc);
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Recamán's sequence
// ────────────────────────────────────────────────────────────────────────────

/// Recamán's sequence: a(0)=0; a(n) = a(n−1)−n if that is positive and new,
/// else a(n−1)+n. Each jump is drawn as a half-arc over a number line,
/// alternating above and below — the classic Recamán picture. `eased`
/// reveals terms (the newest arc grows as it is drawn); `time` runs a bead
/// along the newest arc.
struct Recaman;
impl ProgressStyle for Recaman {
    fn name(&self) -> &str {
        "recaman"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Recamán's sequence 0, 1, 3, 6, 2, 7, 13, 20, 12, 21, 11, … drawn the classic way: each \
         jump a half-arc over the number line, alternating above and below, nesting into \
         rainbows as progress adds terms; a bead runs along the newest arc"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        const TERMS: usize = 26;
        let mut seq = [0usize; TERMS];
        for n in 1..TERMS {
            let prev = seq[n - 1];
            let back = prev.wrapping_sub(n);
            seq[n] = if prev > n && !seq[..n].contains(&back) {
                back
            } else {
                prev + n
            };
        }
        let max_val = seq.iter().copied().max().unwrap_or(1).max(1) as f32;
        let max_jump = (TERMS - 1) as f32;
        let sx = (w as f32 - 1.0) / max_val;
        let mid = (h as f32 - 1.0) * 0.5;
        let reach = mid.max(0.5);

        // Number line.
        draw::hline(grid, 0, w - 1, mid.round() as usize);

        let stage = ctx.eased.clamp(0.0, 1.0) * (TERMS - 1) as f32;
        let full = stage as usize;
        for n in 1..TERMS {
            let part = if n <= full {
                1.0
            } else if n == full + 1 {
                stage.fract()
            } else {
                break;
            };
            if part <= 0.0 {
                break;
            }
            let (a, b) = (seq[n - 1] as f32, seq[n] as f32);
            let cx = (a + b) * 0.5 * sx;
            let rx = (b - a).abs() * 0.5 * sx;
            // Every arc keeps the same squashed aspect (height ∝ jump), so the
            // nested arcs stay distinct and the biggest just fills the half-height.
            let ry = reach * (n as f32 / max_jump);
            let sign = if n % 2 == 1 { -1.0 } else { 1.0 };
            let steps = ((rx + ry) * 2.0).clamp(6.0, 200.0) as usize;
            let upto = ((steps as f32) * part).round() as usize;
            // Travel from a to b: θ = π at a's side when b > a.
            let dir = if b > a { 1.0 } else { -1.0 };
            for s in 0..=upto {
                let th = s as f32 / steps as f32 * PI;
                let x = cx - dir * rx * th.cos();
                let y = mid + sign * ry * th.sin();
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
            if n == full.max(1) && part >= 1.0 {
                let th = (ctx.time.max(0.0) * 0.6).fract() * PI;
                let x = (cx - dir * rx * th.cos()).round() as i32;
                let y = (mid + sign * ry * th.sin()).round() as i32;
                for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    draw::dot_i(grid, x + ox, y + oy);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 12. Digital root / Vortex math
// ────────────────────────────────────────────────────────────────────────────

/// Digital root vortex math: place digits 1–9 around a circle; connect the
/// digital root sequence of n (n, n+n, …) as a cycle of chords. `eased`
/// selects the base number n ∈ 1..9; `time` rotates and pulses the figure.
/// The vortex math pattern mod 9 reveals hidden symmetries (3–6–9, 1–2–4–8–7–5).
struct DigitalRoot;
impl ProgressStyle for DigitalRoot {
    fn name(&self) -> &str {
        "digital-root-vortex"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Vortex math: digital-root cycles 1–9 drawn as chords on a circle, revealing mod-9 symmetry"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let rx = (w / 2).saturating_sub(1) as f32;
        let ry = (h / 2).saturating_sub(1) as f32;

        // Digital root of n mod 9 (returns 1..=9; 0→9).
        fn digital_root(n: usize) -> usize {
            if n == 0 {
                return 9;
            }
            let r = n % 9;
            if r == 0 {
                9
            } else {
                r
            }
        }

        // Base digit from eased (1–9).
        let base_digit = ((ctx.eased * 8.0).floor() as usize + 1).min(9);

        // Build the cycle: starting from base_digit, keep adding base_digit mod 9.
        let mut cycle = vec![base_digit];
        let mut cur = base_digit;
        for _ in 0..8 {
            cur = digital_root(cur + base_digit);
            if cur == cycle[0] {
                break;
            }
            cycle.push(cur);
        }

        // Rotation from time.
        let rot = ctx.time * 0.15;

        // Draw the 9 node positions on the circle.
        for d in 1..=9usize {
            let angle = (d as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let px = (cx + rx * angle.cos()).round() as usize;
            let py = (cy + ry * angle.sin()).round() as usize;
            if px < w && py < h {
                draw::dot(grid, px, py);
            }
        }

        // Draw the cycle chords.
        for i in 0..cycle.len() {
            let a = cycle[i];
            let b = cycle[(i + 1) % cycle.len()];
            let angle_a = (a as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let angle_b = (b as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let x0 = (cx + rx * angle_a.cos()).round() as i32;
            let y0 = (cy + ry * angle_a.sin()).round() as i32;
            let x1 = (cx + rx * angle_b.cos()).round() as i32;
            let y1 = (cy + ry * angle_b.sin()).round() as i32;

            let steps = ((x1 - x0).abs().max((y1 - y0).abs())).max(1) as usize;
            let steps = steps.min(512);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = (x0 as f32 + t * (x1 - x0) as f32).round() as i32;
                let py = (y0 as f32 + t * (y1 - y0) as f32).round() as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Draw the 3-6-9 axis separately for emphasis.
        for d in [3usize, 6, 9] {
            let angle = (d as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let px = (cx + rx * angle.cos()).round() as i32;
            let py = (cy + ry * angle.sin()).round() as i32;
            draw::dot_i(grid, px, py);
            draw::dot_i(grid, px + 1, py);
            draw::dot_i(grid, px, py + 1);
            draw::dot_i(grid, px - 1, py);
            draw::dot_i(grid, px, py - 1);
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
    let styles = progress::styles::numbertheory::styles();
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
