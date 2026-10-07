//! `architecture` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O architecture.rs && ./architecture [style-name]
//! ```

const DEFAULT_STYLE: &str = "skyscraper";

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
    pub mod architecture {
//! Architecture / construction progress bars.
//!
//! Thirteen structurally distinct styles, each animating a different building
//! form or construction activity:
//!
//! - `skyscraper`       — glass towers top out one by one, lit windows twinkle
//! - `suspension-bridge`— red pylons + main cable; deck launched span by span
//! - `gothic-arch`      — pointed lancet arches rise, fill with stained glass
//! - `brick-wall`       — running-bond masonry laid course by course, trowel sweeps
//! - `pyramid`          — three stepped pyramids rise course by course under a sun
//! - `scaffolding`      — standards, ledgers, boards and braces go up bay by bay
//! - `geodesic-dome`    — triangulated hemisphere assembles band by band
//! - `spiral-staircase` — looking down the well: treads laid along a spiral rail
//! - `roman-aqueduct`   — two arch tiers advance over a river; water flows on top
//! - `classical-column` — Ionic colonnade rises drum by drum under an entablature
//! - `blueprint`        — a drafting pen sweeps lines onto graph paper
//! - `tower-crane`      — lattice crane hoists beams as a frame rises beside it
//! - `colosseum`        — three arcade tiers + attic, ruined far end, built left to right

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `architecture` theme.
///
/// Returns thirteen `Box<dyn ProgressStyle>` values, each implementing a
/// structurally distinct architecture/construction scene in braille dot space.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Skyscraper),
        Box::new(SuspensionBridge),
        Box::new(GothicArch),
        Box::new(BrickWall),
        Box::new(Pyramid),
        Box::new(Scaffolding),
        Box::new(GeodesicDome),
        Box::new(SpiralStaircase),
        Box::new(RomanAqueduct),
        Box::new(ClassicalColumn),
        Box::new(Blueprint),
        Box::new(TowerCrane),
        Box::new(Colosseum),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Set a dot from signed coordinates and tint its cell — the one primitive the
/// hand-coloured scenes build on. Out-of-bounds writes are ignored.
fn paint(grid: &mut BrailleGrid, x: i32, y: i32, c: Color) {
    let (dw, dh) = draw::dot_dims(grid);
    if x < 0 || y < 0 || x as usize >= dw || y as usize >= dh {
        return;
    }
    draw::dot(grid, x as usize, y as usize);
    let _ = grid.set_cell_color(x as usize / 2, y as usize / 4, c);
}

/// Coloured horizontal run `x0..=x1` at row `y`.
fn paint_h(grid: &mut BrailleGrid, x0: i32, x1: i32, y: i32, c: Color) {
    for x in x0.min(x1)..=x0.max(x1) {
        paint(grid, x, y, c);
    }
}

/// Coloured vertical run `y0..=y1` at column `x`.
fn paint_v(grid: &mut BrailleGrid, x: i32, y0: i32, y1: i32, c: Color) {
    for y in y0.min(y1)..=y0.max(y1) {
        paint(grid, x, y, c);
    }
}

/// Coloured Bresenham line between two dot-space points.
fn paint_line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut err) = (x0, y0, dx - dy);
    loop {
        paint(grid, x, y, c);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Cheap deterministic hash of three integers into `0.0..=1.0` — used for
/// window lights and stained-glass colours that flicker without state.
fn hash3(a: i32, b: i32, c: i32) -> f32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1)
        ^ (b as u32).wrapping_mul(0x85EB_CA77)
        ^ (c as u32).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Skyscraper
// ─────────────────────────────────────────────────────────────────────────────

/// A skyline of narrow glass towers tops out one by one, left to right. Each
/// finished tower gets a setback crown, an antenna with a blinking beacon and
/// twinkling lit windows; the tower going up is a bare steel frame with a
/// crane swinging on top.
struct Skyscraper;
impl ProgressStyle for Skyscraper {
    fn name(&self) -> &str {
        "skyscraper"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Glass towers top out one by one across a skyline; lit windows twinkle, beacons blink, a crane swings on the tower going up"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 4 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let ground = hi - 1;
        let steel = Color::rgb(120, 140, 172);
        let lit = Color::rgb(255, 214, 120);
        let frame = Color::rgb(150, 120, 90);
        let crane = Color::rgb(250, 190, 40);
        let beacon = Color::rgb(255, 60, 50);

        // Tower plan: (width in dots, height as a fraction of the sky).
        const PLAN: [(i32, f32); 6] = [
            (6, 0.72),
            (7, 1.0),
            (5, 0.6),
            (6, 0.88),
            (5, 0.68),
            (7, 0.95),
        ];
        let mut towers = Vec::new();
        let mut x = 2;
        while x < wi - 3 {
            let (tw, hf) = PLAN[towers.len() % PLAN.len()];
            let tw = tw.min(wi - 1 - x);
            if tw < 4 {
                break;
            }
            towers.push((x, tw, hf));
            x += tw + 3;
        }
        let built = ctx.eased * towers.len().max(1) as f32;
        let tick = (ctx.time * 1.5) as i32;

        for (k, &(x0, tw, hf)) in towers.iter().enumerate() {
            let frac = (built - k as f32).clamp(0.0, 1.0);
            if frac <= 0.0 {
                continue;
            }
            let full_h = (((hi - 3) as f32 * hf).round() as i32).max(3);
            let h = ((full_h as f32 * frac).ceil() as i32).max(1);
            let top = ground - h;
            let done = frac >= 1.0;
            let crown = ground - full_h + full_h / 4; // setback starts above here
            for y in top..ground {
                let inset = i32::from(tw >= 7 && y < crown);
                let (l, r) = (x0 + inset, x0 + tw - 1 - inset);
                // The top three floors of a rising tower are bare frame.
                let skeleton = !done && y < top + 3;
                let c = if skeleton { frame } else { steel };
                paint(grid, l, y, c);
                paint(grid, r, y, c);
                if skeleton {
                    if (ground - y) % 2 == 0 {
                        paint_h(grid, l, r, y, frame);
                    }
                } else if (ground - y) % 2 == 1 {
                    for wx in l + 1..r {
                        if hash3(wx, y, tick + k as i32 * 7) < 0.66 {
                            paint(grid, wx, y, lit);
                        }
                    }
                }
            }
            let inset = i32::from(tw >= 7 && top < crown);
            paint_h(grid, x0 + inset, x0 + tw - 1 - inset, top, steel);
            let mid = x0 + tw / 2;
            if done {
                // Antenna + blinking aviation beacon.
                paint_v(grid, mid, top - 2, top - 1, steel);
                if (ctx.time * 2.0 + k as f32).sin() > 0.0 {
                    paint(grid, mid, top - 3, beacon);
                }
            } else {
                // Tower crane on the rising frame: mast + swinging jib.
                let mast = x0 + 1;
                paint_v(grid, mast, top - 4, top - 1, crane);
                let reach = ((ctx.time * 1.3).cos() * (tw + 6) as f32) as i32;
                paint_h(grid, mast - 3, mast + reach, top - 4, crane);
                let hook = mast + reach * 3 / 4;
                paint_v(grid, hook, top - 3, top - 2, crane);
            }
        }
        paint_h(grid, 0, wi - 1, ground, Color::rgb(90, 90, 104));
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Suspension Bridge
// ─────────────────────────────────────────────────────────────────────────────

/// Two tall pylons carry a sagging main cable; the road deck is launched left
/// to right under it, suspenders dropping onto each new span, while a car
/// shuttles along the finished deck above rippling water.
struct SuspensionBridge;
impl ProgressStyle for SuspensionBridge {
    fn name(&self) -> &str {
        "suspension-bridge"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Red pylons and a sagging main cable; the deck is launched span by span with suspenders, a car crosses, water ripples below"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 8 || dh < 4 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let red = Color::rgb(222, 78, 44);
        let deck_c = Color::rgb(190, 190, 200);
        let water = Color::rgb(50, 120, 210);
        let car = Color::rgb(255, 220, 90);

        let deck_y = (hi * 5 / 8).max(2);
        let water_y = deck_y + 3;
        let t1 = wi * 27 / 100;
        let t2 = wi - 1 - t1;
        let mid = (t1 + t2) as f32 / 2.0;
        let half = ((t2 - t1) as f32 / 2.0).max(1.0);
        let top = 1;
        let sag = (deck_y - 1 - top) as f32;

        // Water first so the structure paints over it.
        for y in water_y..hi {
            for x in 0..wi {
                let v = (x as f32 * 0.45 + ctx.time * 3.0 + y as f32 * 2.1).sin();
                if v > 0.55 {
                    paint(grid, x, y, water);
                }
            }
        }

        // Main cable: parabola between tower tops, side spans to anchors.
        let cable_y = |x: i32| -> i32 {
            let xf = x as f32;
            if x >= t1 && x <= t2 {
                let u = (xf - mid) / half;
                (deck_y as f32 - 1.0 - sag * u * u).round() as i32
            } else {
                let (from, len) = if x < t1 {
                    (t1 as f32 - xf, t1 as f32)
                } else {
                    (xf - t2 as f32, (wi - 1 - t2) as f32)
                };
                let s = (from / len.max(1.0)).clamp(0.0, 1.0);
                (top as f32 + sag * s + 1.5 * s * (1.0 - s)).round() as i32
            }
        };
        let mut prev = cable_y(0);
        for x in 0..wi {
            let y = cable_y(x);
            paint_v(grid, x, prev, y, red);
            prev = y;
        }

        // Deck launched left to right; suspenders drop onto finished deck.
        let built = (ctx.eased * wi as f32).round() as i32;
        for x in 0..built.min(wi) {
            if x % 3 == 0 && x > 0 && x < wi - 1 {
                let cy = cable_y(x);
                if cy < deck_y - 1 {
                    paint_v(grid, x, cy + 1, deck_y - 1, red);
                }
            }
            paint(grid, x, deck_y, deck_c);
            if x % 2 == 0 {
                paint(grid, x, deck_y + 1, deck_c);
            }
        }

        // Pylons: twin legs with portal beams, standing in the water.
        for &tx in &[t1, t2] {
            paint_v(grid, tx - 1, 0, hi - 1, red);
            paint_v(grid, tx + 1, 0, hi - 1, red);
            paint_h(grid, tx - 1, tx + 1, 0, red);
            paint_h(grid, tx - 1, tx + 1, deck_y / 2, red);
        }

        // A car shuttles along whatever deck exists.
        if built > 6 {
            let span = (built - 4).max(1) as f32;
            let tri = 1.0 - ((ctx.time * 0.35).fract() * 2.0 - 1.0).abs();
            let cx = (tri * span) as i32;
            paint_h(grid, cx, cx + 3, deck_y - 1, car);
            paint_h(grid, cx + 1, cx + 2, deck_y - 2, car);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Gothic Arch / Cathedral
// ─────────────────────────────────────────────────────────────────────────────

/// An arcade of pointed lancet arches is raised bay by bay: piers rise, the
/// two arcs spring and meet at the point, then stained glass fills the window
/// and shimmers with time. Pinnacles crown every pier.
struct GothicArch;
impl ProgressStyle for GothicArch {
    fn name(&self) -> &str {
        "gothic-arch"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Pointed lancet arches rise bay by bay on pinnacled piers, then fill with shimmering stained glass"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 14 || dh < 6 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let stone = Color::rgb(214, 196, 160);
        let glass = [
            Color::rgb(60, 100, 230),
            Color::rgb(210, 40, 70),
            Color::rgb(245, 190, 60),
            Color::rgb(60, 180, 110),
            Color::rgb(150, 80, 220),
        ];
        let ground = hi - 1;
        let bay = 12;
        let span = bay - 2;
        let n = ((wi - 2) / bay).max(1);
        let off = ((wi - n * bay - 2) / 2) & !1; // keep piers cell-aligned
        let spring = hi * 9 / 16;
        let r = span as f32 * 0.85;
        let tick = (ctx.time * 2.0) as i32;
        let built = ctx.eased * n as f32;
        paint_h(grid, 0, wi - 1, ground, stone);

        // Height of the arch intrados above `spring` at dot column `dx` of a bay.
        let arc = |dx: i32| -> f32 {
            let d = dx.min(span - 1 - dx) as f32;
            let c = r - d; // distance to the opposite arc centre
            (r * r - c * c).max(0.0).sqrt()
        };

        for k in 0..n {
            let px = off + k * bay; // left pier
            let x0 = px + 2; // first window column
            let f = (built - k as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                continue;
            }
            // Stained glass once the arch is closed, in diamond leaded panes.
            if f >= 0.85 {
                for dx in 0..span {
                    let ay = spring - arc(dx).round() as i32;
                    for y in ay + 1..=ground {
                        if (dx + y) % 4 == 0 || (dx - y).rem_euclid(4) == 0 {
                            continue; // lead came
                        }
                        let cell = (x0 + dx) / 2 * 31 + y / 4;
                        let g = glass[(hash3(cell, k, tick) * 4.99) as usize];
                        paint(grid, x0 + dx, y, g);
                    }
                }
            }
            // Piers rise first (both sides of the bay).
            let pier_f = (f / 0.4).min(1.0);
            let pier_top = ground - ((ground - 1) as f32 * pier_f).round() as i32;
            for &p in &[px, px + bay] {
                paint_v(grid, p, pier_top, ground, stone);
                paint_v(grid, p + 1, pier_top.max(spring), ground, stone);
            }
            if pier_f >= 1.0 {
                // Pinnacle finials.
                paint(grid, px, 0, stone);
                paint(grid, px + bay, 0, stone);
            }
            // The two arcs spring together and meet at the point.
            let arch_f = ((f - 0.4) / 0.45).clamp(0.0, 1.0);
            let reach = (arch_f * (span / 2) as f32).round() as i32;
            let mut prev: Option<(i32, i32)> = None;
            for dx in 0..span {
                let side = dx.min(span - 1 - dx);
                if side >= reach {
                    prev = None;
                    continue;
                }
                let y = spring - arc(dx).round() as i32;
                let x = x0 + dx;
                match prev {
                    Some((pxx, py)) => paint_line(grid, pxx, py, x, y, stone),
                    None => paint(grid, x, y, stone),
                }
                prev = Some((x, y));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Brick Wall
// ─────────────────────────────────────────────────────────────────────────────

/// Red bricks in running bond, one course per cell row, laid brick by brick
/// left to right and course by course bottom to top. Mortar joints are the
/// gaps; the brick being bedded is still an outline while a trowel taps it.
struct BrickWall;
impl ProgressStyle for BrickWall {
    fn name(&self) -> &str {
        "brick-wall"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Red bricks in running bond are laid one by one, course by course, while a trowel taps the newest brick into its mortar bed"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 8 || dh < 4 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let reds = [
            Color::rgb(178, 62, 44),
            Color::rgb(198, 82, 54),
            Color::rgb(158, 54, 40),
            Color::rgb(210, 104, 66),
        ];
        let steel = Color::rgb(200, 200, 212);
        let handle = Color::rgb(150, 100, 60);

        let brick = 8; // 7 dots of brick + 1 dot head joint
        let courses = (hi / 4).max(1);
        let per = (wi + brick / 2) / brick + 1;
        let total = (courses * per) as f32;
        let laid = ctx.eased * total;
        let whole = laid as i32;
        let mut current: Option<(i32, i32)> = None;

        for c in 0..courses {
            let y0 = hi - 4 * (c + 1); // top row of this course
            let off = (c % 2) * (brick / 2);
            for b in 0..per {
                let idx = c * per + b;
                if idx > whole {
                    break;
                }
                let x0 = b * brick - off;
                let x1 = (x0 + brick - 2).min(wi - 1);
                if x1 < 0 {
                    continue;
                }
                let colour = reds[(hash3(b, c, 3) * 3.99) as usize];
                if idx == whole {
                    // Being bedded: outline only.
                    paint_h(grid, x0.max(0), x1, y0, colour);
                    paint_h(grid, x0.max(0), x1, y0 + 2, colour);
                    paint_v(grid, x0.max(0), y0, y0 + 2, colour);
                    paint_v(grid, x1, y0, y0 + 2, colour);
                    current = Some((x1, y0));
                } else {
                    for y in y0..y0 + 3 {
                        paint_h(grid, x0.max(0), x1, y, colour);
                    }
                }
            }
        }

        // Trowel taps the newest brick: blade + handle, bobbing with time.
        if let Some((x, y)) = current {
            let tap = ((ctx.time * 6.0).sin() * 0.5 + 0.5).round() as i32;
            let (tx, ty) = (x + 2, y - 3 + tap);
            paint_h(grid, tx, tx + 3, ty + 1, steel);
            paint_h(grid, tx + 1, tx + 2, ty + 2, steel);
            paint_line(grid, tx + 3, ty, tx + 5, ty - 2, handle);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pyramid
// ─────────────────────────────────────────────────────────────────────────────

/// Three stepped pyramids rise on the desert under a low sun, built course by
/// course, each course laid left to right. Faces are lit on the left and in
/// shadow on the right; the great pyramid wears a gold capstone when done.
struct Pyramid;
impl ProgressStyle for Pyramid {
    fn name(&self) -> &str {
        "pyramid"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Three stepped sandstone pyramids rise course by course under a desert sun; sunlit and shadowed faces, gold capstone"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 8 || dh < 4 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let lit = Color::rgb(232, 198, 132);
        let shade = Color::rgb(168, 124, 78);
        let gold = Color::rgb(255, 214, 90);
        let sun = Color::rgb(255, 160, 60);
        let sand = Color::rgb(200, 170, 110);

        let ground = hi - 1;
        let max_c = ((ground - 1) / 2).max(1);
        // (centre as a fraction of width, courses) — great, middle, small.
        let plan = [
            (0.23, max_c),
            (0.59, (max_c * 6 / 7).max(1)),
            (0.86, (max_c * 4 / 7).max(1)),
        ];
        let total: i32 = plan.iter().map(|p| p.1).sum();
        let laid = ctx.eased * total as f32;

        // Sun low in the sky, and drifting sand along the ground.
        let (sx, sy) = (wi - 22, 3);
        for dy in -2i32..=2 {
            let half = 2 - dy.abs() / 2;
            paint_h(grid, sx - half, sx + half, sy + dy, sun);
        }
        for x in 0..wi {
            if (x as f32 + ctx.time * 6.0).rem_euclid(7.0) < 5.0 {
                paint(grid, x, ground, sand);
            }
        }

        let mut before = 0;
        for (pi, &(fx, courses)) in plan.iter().enumerate() {
            let cx = (wi as f32 * fx) as i32;
            for c in 0..courses {
                let f = (laid - (before + c) as f32).clamp(0.0, 1.0);
                if f <= 0.0 {
                    break;
                }
                let hw = (courses - c) * 2;
                let (x0, x1) = (cx - hw, cx + hw);
                let reach = x0 + ((x1 - x0 + 1) as f32 * f) as i32;
                let capstone = pi == 0 && c == courses - 1 && f >= 1.0;
                for y in [ground - 1 - 2 * c, ground - 2 - 2 * c] {
                    for x in x0..reach.min(x1 + 1) {
                        let col = if capstone {
                            gold
                        } else if x < cx {
                            lit
                        } else {
                            shade
                        };
                        paint(grid, x, y, col);
                    }
                }
            }
            before += courses;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Scaffolding
// ─────────────────────────────────────────────────────────────────────────────

/// Tube-and-coupler scaffolding is erected bay by bay, lift by lift: steel
/// standards, ledgers at every lift, timber boards on top and yellow cross
/// braces. A bucket is hoisted up and down on the newest bay.
struct Scaffolding;
impl ProgressStyle for Scaffolding {
    fn name(&self) -> &str {
        "scaffolding"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Steel standards, ledgers, timber boards and cross braces go up bay by bay, lift by lift; a bucket is hoisted on the newest bay"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 12 || dh < 4 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let steel = Color::rgb(176, 186, 202);
        let brace = Color::rgb(124, 134, 150);
        let bucket = Color::rgb(240, 180, 40);
        let board = Color::rgb(178, 116, 62);
        let rope = Color::rgb(140, 140, 150);

        let ground = hi - 1;
        let bay = 10;
        let nb = ((wi - 2) / bay).max(1);
        let lifts = ((hi - 1) / 5).max(1);
        let lift_h = (hi - 1) / lifts;
        let total = (nb * lifts) as f32;
        let placed = ctx.eased * total;

        paint_h(grid, 0, wi - 1, ground, Color::rgb(100, 96, 90));
        for b in 0..nb {
            let x0 = 1 + b * bay;
            let x1 = x0 + bay;
            for l in 0..lifts {
                let f = (placed - (b * lifts + l) as f32).clamp(0.0, 1.0);
                if f <= 0.0 {
                    continue;
                }
                let yb = ground - l * lift_h; // bottom of this lift
                let yt = yb - lift_h; // ledger level on top
                                      // Standards climb first, then the ledger, brace and boards.
                let rise = (f * 2.0).min(1.0);
                let ytop = yb - (lift_h as f32 * rise).round() as i32;
                if f >= 0.5 {
                    let (bx0, bx1) = if (b + l) % 2 == 0 { (x0, x1) } else { (x1, x0) };
                    paint_line(grid, bx0, yb, bx1, yt, brace);
                }
                paint_v(grid, x0, ytop, yb, steel);
                paint_v(grid, x1, ytop, yb, steel);
                if f >= 0.5 {
                    paint_h(grid, x0, x1, yt, steel);
                }
                if f >= 0.75 {
                    paint_h(grid, x0 + 1, x1 - 1, yt + 1, board);
                }
            }
        }

        // Hoist on the newest bay: a rope and a bucket bobbing on it.
        let cur = (placed as i32 / lifts).min(nb - 1);
        let hx = 1 + cur * bay + bay / 2;
        let bob = ((ctx.time * 1.2).sin() * 0.5 + 0.5) * (hi - 5) as f32;
        let by = 1 + bob as i32;
        if ctx.eased < 1.0 {
            paint_v(grid, hx, 0, by, rope);
            paint_h(grid, hx - 1, hx + 1, by + 1, bucket);
            paint_h(grid, hx - 1, hx + 1, by + 2, bucket);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Geodesic Dome
// ─────────────────────────────────────────────────────────────────────────────

/// A geodesic dome assembled ring by ring from the base up: each band is a
/// row of triangles struck between staggered hub points, so the finished
/// hemisphere reads as a lattice of triangles. Hubs glint as light moves
/// across the dome.
struct GeodesicDome;
impl ProgressStyle for GeodesicDome {
    fn name(&self) -> &str {
        "geodesic-dome"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "A triangulated geodesic hemisphere assembles band by band from the base up; light glints across its hubs"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 8 || dh < 6 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let strut = Color::rgb(150, 180, 220);
        let hub = Color::rgb(240, 248, 255);
        let base = Color::rgb(214, 184, 142);

        let ground = hi - 1;
        let ry = (ground - 1) as f32;
        let rx = (ry * 1.6).min(wi as f32 / 2.0 - 1.0);
        let cx = wi as f32 / 2.0;
        let rings = 4;
        let seg = 8; // hubs across the visible front half of a ring
        let built = ctx.eased * rings as f32;
        let glint = (ctx.time * 0.5).fract() * seg as f32;

        // Hub `j` on ring `k` (k = rings is the apex), staggered every ring.
        let hub_at = |k: i32, j: f32| -> (i32, i32) {
            let phi = k as f32 / rings as f32 * std::f32::consts::FRAC_PI_2;
            let theta = (j + 0.5 * (k % 2) as f32) / seg as f32 * std::f32::consts::PI;
            (
                (cx - rx * phi.cos() * theta.cos()).round() as i32,
                (ground as f32 - ry * phi.sin()).round() as i32,
            )
        };

        paint_h(grid, 0, wi - 1, ground, base);
        for k in 0..rings {
            let f = (built - k as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let odd = k % 2 == 1;
            // Staggered rings have one hub fewer inside the silhouette.
            let upto = ((f * seg as f32).ceil() as i32).min(seg + 1 - i32::from(odd));
            for j in 0..upto {
                let a = hub_at(k, j as f32);
                // Ring strut to the next hub on the same ring.
                if j < seg {
                    let b = hub_at(k, (j + 1) as f32);
                    if !(odd && j == seg - 1) {
                        paint_line(grid, a.0, a.1, b.0, b.1, strut);
                    }
                }
                // Diagonal struts up to the staggered hubs on the next ring.
                for dj in [-1, 0] {
                    let jj = j + dj + i32::from(odd);
                    if (0..=seg).contains(&jj) {
                        let b = hub_at(k + 1, jj as f32);
                        paint_line(grid, a.0, a.1, b.0, b.1, strut);
                    }
                }
            }
            // Hubs glint as light sweeps across.
            for j in 0..upto {
                if (j as f32 - glint).abs() < 1.0 {
                    let a = hub_at(k, j as f32);
                    paint(grid, a.0, a.1, hub);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Spiral Staircase
// ─────────────────────────────────────────────────────────────────────────────

/// Looking down a stairwell: a brass handrail spirals from the outer wall in
/// to the newel post, and wedge-shaped stone treads are laid along it, from
/// the top landing down into the well as progress rises. The view slowly
/// turns with time, as if descending.
struct SpiralStaircase;
impl ProgressStyle for SpiralStaircase {
    fn name(&self) -> &str {
        "spiral-staircase"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Looking down a stairwell: treads are laid along a spiralling brass handrail down to the newel post; the view slowly turns"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 8 || dh < 6 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let post = Color::rgb(196, 146, 76);
        let tread = Color::rgb(232, 222, 200);
        let rail = Color::rgb(214, 150, 70);
        let riser = Color::rgb(178, 166, 146);

        let (cx, cy) = (wi as f32 / 2.0, (hi - 1) as f32 / 2.0);
        let ry = cy;
        let rx = (ry * 3.2).min(cx - 1.0); // a perspective stretch
        let turns = 1.5;
        let sweep = turns * std::f32::consts::TAU;
        let shrink = 0.8; // radius lost from outer wall to newel
        let gap = shrink / turns; // radial spacing between spiral arms
        let spin = ctx.time * 0.4;
        let at = |u: f32, r: f32| -> (i32, i32) {
            let a = u * sweep + spin;
            (
                (cx + rx * r * a.cos()).round() as i32,
                (cy + ry * r * a.sin()).round() as i32,
            )
        };

        // The handrail spiral, drawn first so laid treads own their cells.
        let samples = 360;
        let mut prev = at(0.0, 1.0);
        for s in 1..=samples {
            let u = s as f32 / samples as f32;
            let p = at(u, 1.0 - shrink * u);
            paint_line(grid, prev.0, prev.1, p.0, p.1, rail);
            prev = p;
        }

        // Treads: solid wedges filling the band between one turn of the rail
        // and the next, split by thin risers and laid from the top landing
        // (outside) down into the well.
        let tau = std::f32::consts::TAU;
        let n = 22.0;
        let laid = ctx.eased * n;
        for y in 0..hi {
            for x in 0..wi {
                let (ex, ey) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
                let rho = (ex * ex + ey * ey).sqrt();
                let a = (ey.atan2(ex) - spin).rem_euclid(tau);
                // Walk the arms passing this angle to find the one just outside.
                let mut m = 0.0;
                loop {
                    let u = (a + m * tau) / sweep;
                    if u > 1.0 {
                        break;
                    }
                    let r = 1.0 - shrink * u;
                    if rho < r - 0.04 && rho > r - gap + 0.04 {
                        let k = u * n;
                        if k < laid && k.fract() > 0.2 {
                            let c = if (k as i32) % 2 == 0 { tread } else { riser };
                            paint(grid, x, y, c);
                        }
                        break;
                    }
                    m += 1.0;
                }
            }
        }

        // Newel post at the bottom of the well.
        let (nx, ny) = (cx.round() as i32, cy.round() as i32);
        paint_h(grid, nx - 1, nx, ny, post);
        paint_h(grid, nx - 1, nx, ny + 1, post);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Roman Aqueduct
// ─────────────────────────────────────────────────────────────────────────────

/// A two-tier Roman aqueduct in the Pont du Gard manner: big round arches
/// over a river, a row of small arches above, and the water channel on top.
/// Masonry advances left to right — lower tier first, upper tier and channel
/// close behind — and water flows along the finished channel.
struct RomanAqueduct;
impl ProgressStyle for RomanAqueduct {
    fn name(&self) -> &str {
        "roman-aqueduct"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Two tiers of round stone arches advance across a river, with the water channel on top flowing as each span is finished"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 4 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let stone = Color::rgb(214, 184, 142);
        let water = Color::rgb(70, 150, 230);

        // Rows scale with height: channel on top, small tier, big tier.
        let floor = (hi / 8).max(1); // channel floor row
        let impost = hi / 2; // between the tiers
        let ground = hi - 1;
        let lower = (ctx.eased * (wi + 24) as f32) as i32;
        let (upper, channel) = (lower - 12, lower - 24);

        // Is (x, y) inside an arch opening of span `span` on `pitch`, springing
        // at row `spring` with its crown at row `crown`?
        let opening = |x: i32, y: i32, pitch: i32, span: i32, crown: i32, spring: i32| {
            let ux = (x + pitch / 2).rem_euclid(pitch) - (pitch - span) / 2;
            if ux < 0 || ux >= span || y < crown {
                return false;
            }
            let r = (spring - crown).max(1) as f32;
            let dx = ux as f32 + 0.5 - span as f32 / 2.0;
            let dy = (spring - y).max(0) as f32;
            dx * dx * (r * r) / ((span as f32 / 2.0).powi(2)) + dy * dy <= r * r
        };

        for x in 0..wi {
            for y in 0..hi {
                let lit = if y < floor {
                    x < channel && y == 0
                } else if y == floor {
                    x < channel
                } else if y < impost {
                    x < upper && !opening(x, y, 6, 4, floor + 2, impost - 2)
                } else {
                    x < lower && (y == impost || !opening(x, y, 12, 9, impost + 2, ground - 2))
                };
                if lit {
                    paint(grid, x, y, stone);
                }
            }
            // Water runs along the finished channel...
            if x < channel && ((x as f32 - ctx.time * 10.0).rem_euclid(8.0)) < 3.0 {
                paint(grid, x, (floor - 1).max(1), water);
            }
            // ...and the river ripples under the big arches.
            let open = |x| opening(x, ground, 12, 9, impost + 2, ground - 2);
            if (x as f32 * 0.7 + ctx.time * 3.0).sin() > 0.2 && open(x & !1) && open(x | 1) {
                paint(grid, x, ground, water);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Classical Column
// ─────────────────────────────────────────────────────────────────────────────

/// A colonnade of Ionic columns on a stepped stylobate. Columns go up left to
/// right: base, then fluted drums lowered one at a time on a crane rope, then
/// the scrolled capital; the entablature runs across each finished column and
/// a band of sunlight sweeps the marble.
struct ClassicalColumn;
impl ProgressStyle for ClassicalColumn {
    fn name(&self) -> &str {
        "classical-column"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Ionic columns rise drum by drum on a stepped base, gain scrolled capitals, then carry the entablature; sunlight sweeps the marble"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 12 || dh < 8 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let marble = Color::rgb(206, 200, 186);
        let glint = Color::rgb(255, 252, 240);
        let entab = Color::rgb(214, 184, 142);
        let step_c = Color::rgb(150, 140, 126);
        let rope = Color::rgb(140, 140, 150);

        let ground = hi - 1;
        let base_y = ground - 2;
        let cap_y = hi * 3 / 16; // abacus row
        let shaft_top = cap_y + 3;
        let shaft_h = (base_y - shaft_top).max(1);
        let pitch = 14;
        let n = ((wi - 4) / pitch).max(1);
        let off = (wi - n * pitch) / 2;
        let built = ctx.eased * n as f32;
        let sun = (ctx.time * 0.25).fract() * (wi + 20) as f32 - 10.0;

        // Stepped stylobate.
        paint_h(grid, 0, wi - 1, ground, step_c);
        paint_h(grid, 2, wi - 3, ground - 1, step_c);

        for k in 0..n {
            let cx = off + k * pitch + pitch / 2;
            let f = (built - k as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                continue;
            }
            let c = if (cx as f32 - sun).abs() < 5.0 {
                glint
            } else {
                marble
            };
            // Attic base.
            paint_h(grid, cx - 3, cx + 3, base_y, c);
            // Fluted shaft: three ridges, stacked drum by drum.
            let shaft_f = (f / 0.8).min(1.0);
            let rows = (shaft_h as f32 * shaft_f).round() as i32;
            let top = base_y - rows;
            for &dx in &[-2, 0, 2] {
                paint_v(grid, cx + dx, top, base_y - 1, c);
            }
            if shaft_f < 1.0 {
                // Next drum hangs on the crane rope, settling with time.
                let bob = ((ctx.time * 2.0).sin() * 0.5 + 0.5) * 3.0;
                let dy = top - 3 - bob.round() as i32;
                paint_v(grid, cx, 0, dy - 1, rope);
                paint_h(grid, cx - 2, cx + 2, dy, c);
                paint_h(grid, cx - 2, cx + 2, dy + 1, c);
            } else {
                // Ionic capital: echinus, curled volutes, flat abacus.
                paint_h(grid, cx - 4, cx + 4, cap_y, c);
                paint_h(grid, cx - 3, cx + 3, cap_y + 1, c);
                for &s in &[-1, 1] {
                    paint(grid, cx + s * 4, cap_y + 1, c);
                    paint(grid, cx + s * 4, cap_y + 2, c);
                    paint(grid, cx + s * 3, cap_y + 2, c);
                }
            }
            // Entablature across each finished column.
            if f >= 1.0 {
                let (l, r) = (cx - pitch / 2, cx + pitch / 2);
                paint_h(grid, l, r, cap_y - 1, entab);
                for x in l..=r {
                    if (x - off) % 3 == 0 {
                        paint(grid, x, cap_y - 2, entab);
                    }
                }
                paint_h(grid, l - 1, r + 1, cap_y - 3, entab);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Blueprint
// ─────────────────────────────────────────────────────────────────────────────

/// A drafting pen sweeps technical lines across graph paper; grid appears first.
struct Blueprint;
impl ProgressStyle for Blueprint {
    fn name(&self) -> &str {
        "blueprint"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Blueprint drafted: graph grid first, then walls, windows, and dimensions"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 4 {
            return Ok(());
        }

        // Phase 1 (0–0.2): faint grid lines.
        if ctx.eased > 0.0 {
            let grid_spacing = 4usize;
            let grid_p = (ctx.eased / 0.2).min(1.0);
            let grid_cols = ((dw / grid_spacing) as f32 * grid_p) as usize;
            let grid_rows = ((dh / grid_spacing) as f32 * grid_p) as usize;
            for gx in 0..=grid_cols {
                let x = gx * grid_spacing;
                // Every other dot to keep it light.
                let mut y = 0usize;
                while y < dh {
                    draw::dot(grid, x.min(dw.saturating_sub(1)), y);
                    y += 2;
                }
            }
            for gy in 0..=grid_rows {
                let y = gy * grid_spacing;
                let mut x = 0usize;
                while x < dw {
                    draw::dot(grid, x, y.min(dh.saturating_sub(1)));
                    x += 2;
                }
            }
        }

        // Phase 2 (0.2–0.6): floor plan walls sweep in (outer rectangle).
        if ctx.eased > 0.2 {
            let wall_p = ((ctx.eased - 0.2) / 0.4).min(1.0);
            let x0 = dw / 8;
            let x1 = dw - dw / 8;
            let y0 = dh / 8;
            let y1 = dh - dh / 8;
            let perim = 2 * ((x1 - x0) + (y1 - y0));
            let drawn = (wall_p * perim as f32) as usize;
            // Walk the perimeter: top, right, bottom, left.
            let top_len = x1 - x0;
            let right_len = y1 - y0;
            let bot_len = x1 - x0;
            let left_len = y1 - y0;
            let mut rem = drawn;
            // Top.
            let seg = rem.min(top_len);
            draw::hline(grid, x0, x0 + seg, y0);
            rem = rem.saturating_sub(seg);
            // Right.
            let seg = rem.min(right_len);
            draw::vline(grid, x1, y0, y0 + seg);
            rem = rem.saturating_sub(seg);
            // Bottom (reversed).
            let seg = rem.min(bot_len);
            draw::hline(grid, x1.saturating_sub(seg), x1, y1);
            rem = rem.saturating_sub(seg);
            // Left (reversed).
            let seg = rem.min(left_len);
            draw::vline(grid, x0, y1.saturating_sub(seg), y1);
        }

        // Phase 3 (0.6–0.85): interior walls and door opening.
        if ctx.eased > 0.6 {
            let int_p = ((ctx.eased - 0.6) / 0.25).min(1.0);
            let mid_x = dw / 2;
            let y0 = dh / 8;
            let y1 = dh - dh / 8;
            let wall_len = (y1 - y0).saturating_sub(4); // door gap
            let drawn = (int_p * wall_len as f32) as usize;
            draw::vline(grid, mid_x, y0, y0 + drawn);
        }

        // Phase 4 (0.85–1.0): dimension lines + pen cursor.
        if ctx.eased > 0.85 {
            let dim_p = (ctx.eased - 0.85) / 0.15;
            // Dimension line along the top.
            let y_dim = (dh / 8).saturating_sub(2);
            let x_end = (dim_p * dw as f32) as usize;
            draw::hline(grid, 0, x_end.min(dw.saturating_sub(1)), y_dim);
            // Arrow heads.
            draw::dot(grid, 0, y_dim);
            let tail = x_end.min(dw.saturating_sub(1));
            draw::dot(grid, tail, y_dim);
        }

        // Pen cursor: a moving dot at the current draw frontier.
        let pen_x = (ctx.eased * dw as f32).round() as usize;
        let pen_y = ((ctx.time * 1.5).sin() * dh as f32 * 0.1 + dh as f32 * 0.5) as usize;
        draw::dot(
            grid,
            pen_x.min(dw.saturating_sub(1)),
            pen_y.min(dh.saturating_sub(1)),
        );

        // Tint: blueprint blue.
        let (cw, ch) = grid.dimensions();
        let blue = crate::Color::rgb(30, 80, 200);
        let light = crate::Color::rgb(100, 160, 255);
        for cx2 in 0..cw {
            for cy2 in 0..ch {
                let t = cx2 as f32 / cw.max(1) as f32;
                let color = {
                    let r = (blue.r as f32 + (light.r as f32 - blue.r as f32) * t) as u8;
                    let g = (blue.g as f32 + (light.g as f32 - blue.g as f32) * t) as u8;
                    let b = (blue.b as f32 + (light.b as f32 - blue.b as f32) * t) as u8;
                    crate::Color::rgb(r, g, b)
                };
                draw::tint_row(grid, cy2, cx2, cx2, color);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Tower Crane
// ─────────────────────────────────────────────────────────────────────────────

/// A yellow tower crane — lattice mast, Warren-truss jib, counter-jib with its
/// concrete counterweight, cab and pendant ties — builds a concrete frame
/// beside it: floors go up with progress while the trolley runs out along
/// the jib and the hook lifts a steel beam.
struct TowerCrane;
impl ProgressStyle for TowerCrane {
    fn name(&self) -> &str {
        "tower-crane"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "A lattice tower crane with counterweight and trolley hoists steel beams while the building beside it rises floor by floor"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 24 || dh < 10 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let yellow = Color::rgb(250, 190, 40);
        let weight = Color::rgb(150, 150, 162);
        let cab = Color::rgb(90, 160, 230);
        let cable = Color::rgb(130, 130, 140);
        let beam = Color::rgb(214, 90, 50);
        let concrete = Color::rgb(176, 176, 186);

        let ground = hi - 1;
        let mx = (wi / 6).max(12);
        let (apex, top_chord, jib_y) = (0, 2, 4);
        let jib_end = wi - 2;
        let tail = mx - 10;

        // The building: concrete frame rising floor by floor beside the mast.
        let (bx0, bx1) = (mx + 12, wi - 4);
        let floors = ((ground - jib_y - 4) / 2).max(1);
        let level = ctx.eased * floors as f32;
        let whole = level as i32;
        for f in 0..=whole.min(floors - 1) {
            let y = ground - 2 * (f + 1);
            let len = if f < whole {
                bx1 - bx0
            } else {
                ((level - whole as f32) * (bx1 - bx0) as f32) as i32
            };
            if len <= 0 {
                continue;
            }
            paint_h(grid, bx0, bx0 + len, y, concrete);
            let mut x = bx0;
            while x <= bx0 + len {
                paint(grid, x, y + 1, concrete);
                x += 6;
            }
        }
        let roof = ground - 2 * whole;

        // Pendant ties first, so the steelwork owns the cells they cross.
        paint_line(grid, mx, apex, tail, jib_y, cable);
        paint_line(grid, mx, apex, (mx + jib_end) / 2, top_chord, cable);
        paint_line(grid, mx - 1, jib_y, mx, apex, yellow);
        paint_line(grid, mx + 1, jib_y, mx, apex, yellow);

        // Mast: two chords with alternating diagonal lacing.
        paint_v(grid, mx - 1, jib_y, ground, yellow);
        paint_v(grid, mx + 1, jib_y, ground, yellow);
        let mut y = jib_y;
        while y < ground {
            let (a, b) = if (y / 3) % 2 == 0 { (-1, 1) } else { (1, -1) };
            paint_line(grid, mx + a, y, mx + b, y + 3, yellow);
            y += 3;
        }
        // Jib (Warren truss) and counter-jib with its counterweight.
        paint_h(grid, mx, jib_end, jib_y, yellow);
        paint_h(grid, mx, jib_end - 6, top_chord, yellow);
        let mut x = mx;
        while x + 2 < jib_end - 6 {
            paint_line(grid, x, jib_y, x + 2, top_chord, yellow);
            paint_line(grid, x + 2, top_chord, x + 4, jib_y, yellow);
            x += 4;
        }
        paint_h(grid, tail, mx, jib_y, yellow);
        for wy in jib_y..jib_y + 3 {
            paint_h(grid, tail, tail + 3, wy, weight);
        }
        // Operator's cab under the jib.
        paint_h(grid, mx + 2, mx + 3, jib_y + 1, cab);
        paint_h(grid, mx + 2, mx + 3, jib_y + 2, cab);

        // Trolley shuttles out along the jib; the hook lifts and lowers a beam.
        let reach = (jib_end - 6 - (mx + 6)).max(1) as f32;
        let tri = 1.0 - ((ctx.time * 0.18).fract() * 2.0 - 1.0).abs();
        let tx = mx + 6 + (tri * reach) as i32;
        paint_h(grid, tx - 1, tx + 1, jib_y + 1, yellow);
        let floor_y = if tx >= bx0 - 3 && tx <= bx1 + 3 {
            roof - 2
        } else {
            ground - 1
        };
        let bob = (ctx.time * 1.1).sin() * 0.5 + 0.5;
        let hook = jib_y + 3 + ((floor_y - jib_y - 3).max(0) as f32 * bob) as i32;
        paint_v(grid, tx, jib_y + 2, hook - 1, cable);
        paint_h(grid, tx - 3, tx + 3, hook, beam);

        paint_h(grid, 0, wi - 1, ground, Color::rgb(140, 110, 80));
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 13. Colosseum
// ─────────────────────────────────────────────────────────────────────────────

/// The Colosseum's outer wall: three tiers of round arches under a windowed
/// attic, its far end broken down in a ruined, stepped profile. Travertine is
/// raised left to right with progress while evening light sweeps the stone.
struct Colosseum;
impl ProgressStyle for Colosseum {
    fn name(&self) -> &str {
        "colosseum"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Three tiers of travertine arches under a windowed attic rise left to right, the far end broken in a ruined profile"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 8 || dh < 8 {
            return Ok(());
        }
        grid.enable_color_support();
        let (wi, hi) = (dw as i32, dh as i32);
        let stone = Color::rgb(214, 190, 146);
        let light = Color::rgb(250, 226, 178);

        let tier = (hi / 4).max(2);
        let built = (ctx.eased * wi as f32).round() as i32;
        let sweep = (ctx.time * 0.3).fract() * (wi + 16) as f32 - 8.0;
        // Ruined profile: full height, then stepping down to two tiers.
        let ruin_from = wi * 6 / 10;
        let ruin_to = wi * 85 / 100;
        let top_at = |x: i32| -> i32 {
            if x < ruin_from {
                0
            } else if x >= ruin_to {
                2 * tier
            } else {
                let t = (x - ruin_from) * 2 * tier / (ruin_to - ruin_from).max(1);
                t - t % 2 + (x % 3 == 0) as i32
            }
        };

        for x in 0..built.min(wi) {
            let c = if (x as f32 - sweep).abs() < 6.0 {
                light
            } else {
                stone
            };
            for y in top_at(x)..hi {
                let band = y / tier; // 0 = attic, 1..=3 = arcades
                let row = y % tier;
                let ux = (x + 3).rem_euclid(6);
                let solid = if band == 0 {
                    // Attic: solid with small square windows every other bay.
                    !((1..=2).contains(&row) && (x / 6) % 2 == 0 && (2..4).contains(&ux))
                } else if row == 0 {
                    true // cornice between tiers
                } else {
                    // Round-headed arch opening, 4 dots wide on a 6-dot bay.
                    let inside = (1..5).contains(&ux);
                    let crown = row == 1 && (ux == 1 || ux == 4);
                    !inside || crown
                };
                if solid {
                    paint(grid, x, y, c);
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
    let styles = progress::styles::architecture::styles();
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
