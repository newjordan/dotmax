//! `lasers` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O lasers.rs && ./lasers [style-name]
//! ```

const DEFAULT_STYLE: &str = "charge-and-fire";

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
    pub mod lasers {
//! Laser / light-beam progress bars — eleven structurally distinct beam styles.
//!
//! Every style is stateless: all animation comes from `ctx.time` and all
//! charge / fill comes from `ctx.eased`. Lines are the medium — beams, sweeps,
//! fans, grids, and reflections. Color tints are additive; structure drives the
//! variety.
//!
//! Styles in this module:
//! 1. `charge-and-fire`     — laser cannon charges its meter and fires a beam
//! 2. `scanning-line`       — a vertical sweep beam travels back and forth
//! 3. `security-grid`       — emitter posts with criss-crossing tripwires
//! 4. `prism-dispersion`    — glass prism splits a white beam into a spectrum
//! 5. `laser-light-show`    — stage projectors throw sweeping coloured fans
//! 6. `range-finder`        — handset pings a crosshair target over a ruler
//! 7. `mirror-bounce`       — beam ricochets along a chain of hatched mirrors
//! 8. `fiber-pulse`         — light zigzags down a curving optical fiber
//! 9. `plasma-bolt`         — plasma-ball electrode discharges forked lightning
//! 10. `particle-accelerator` — synchrotron ring; a bunch laps the lit arc
//! 11. `disco-fan`          — radial beams sweeping from a corner origin

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─── deterministic hash helpers ─────────────────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

// ─── step-bounded Bresenham line ─────────────────────────────────────────────

/// Draw a straight line from `(x0,y0)` to `(x1,y1)` in dot-space using
/// integer Bresenham. At most `max_steps` pixels are emitted so that a 1×1
/// grid can never loop a million times. `draw::dot_i` ignores OOB writes.
fn beam_line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, max_steps: usize) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x0;
    let mut cy = y0;
    let steps = (dx + dy + 2) as usize;
    let limit = steps.min(max_steps);
    for _ in 0..limit {
        draw::dot_i(grid, cx, cy);
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

// ─── public registry ─────────────────────────────────────────────────────────

/// All styles in the `lasers` theme.
///
/// Returns eleven laser / light-beam bars, each with a structurally distinct
/// beam topology. Progress (`ctx.eased`) drives charge, fill, or target
/// position; time (`ctx.time`) drives sweep, animation, and flicker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(ChargeAndFire),
        Box::new(ScanningLine),
        Box::new(SecurityGrid),
        Box::new(PrismDispersion),
        Box::new(LaserLightShow),
        Box::new(RangeFinder),
        Box::new(MirrorBounce),
        Box::new(FiberPulse),
        Box::new(PlasmaBolt),
        Box::new(ParticleAccelerator),
        Box::new(DiscoFan),
    ]
}

// ─── dot-rectangle helpers ───────────────────────────────────────────────────

/// Filled rectangle `[x0,x1]×[y0,y1]` (inclusive, signed), clipped to the grid.
fn fill_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let (w, h) = draw::dot_dims(grid);
    let (xa, xb) = (x0.max(0), x1.min(w as i32 - 1));
    let (ya, yb) = (y0.max(0), y1.min(h as i32 - 1));
    for y in ya..=yb {
        for x in xa..=xb {
            draw::dot_i(grid, x, y);
        }
    }
}

/// Rectangle outline `[x0,x1]×[y0,y1]` (inclusive, signed).
fn rect_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    fill_i(grid, x0, y0, x1, y0);
    fill_i(grid, x0, y1, x1, y1);
    fill_i(grid, x0, y0, x0, y1);
    fill_i(grid, x1, y0, x1, y1);
}

/// Tint every cell overlapping the dot rectangle `[x0,x1]×[y0,y1]`.
fn tint_dots(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    if x1 < 0 || y1 < 0 || x1 < x0 || y1 < y0 {
        return;
    }
    let (cx0, cx1) = ((x0.max(0) / 2) as usize, (x1 / 2) as usize);
    for cy in (y0.max(0) / 4) as usize..=(y1 / 4) as usize {
        draw::tint_row(grid, cy, cx0, cx1, color);
    }
}

// ─── 1. Charge and fire ──────────────────────────────────────────────────────

/// A side-on laser cannon: finned housing, grip, barrel and muzzle. Its
/// segmented charge meter fills with eased while the beam it fires reaches
/// further across the bar, sparking where it lands.
struct ChargeAndFire;
impl ProgressStyle for ChargeAndFire {
    fn name(&self) -> &str {
        "charge-and-fire"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Side-on laser cannon: its charge meter fills and the beam it fires reaches across, sparking at the tip"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        if h < 8 || w < 24 {
            // Too small for the cannon — just the beam.
            let reach = (ctx.eased * w as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, h / 2);
            }
            return Ok(());
        }

        // Design space is 16 dots tall; `k` scales it to the grid.
        let k = (h as f32 / 16.0).min(w as f32 / 60.0).max(0.5);
        let s = |v: f32| (v * k).round() as i32;
        let oy = hi / 2 - s(8.0);
        let steel = Color::rgb(150, 156, 172);

        // Housing outline, cooling fins and grip.
        rect_i(grid, 0, oy + s(3.0), s(15.0), oy + s(12.0));
        for f in 0..4 {
            let fx = s(3.0 + f as f32 * 3.0);
            fill_i(grid, fx, oy, fx, oy + s(2.0));
        }
        fill_i(grid, s(3.0), oy + s(13.0), s(6.0), oy + s(15.0));
        // Barrel and muzzle flange.
        fill_i(grid, s(16.0), oy + s(6.0), s(23.0), oy + s(9.0));
        fill_i(grid, s(24.0), oy + s(5.0), s(25.0), oy + s(10.0));
        tint_dots(grid, 0, oy, s(25.0), oy + s(15.0), steel);

        // Charge meter: four segments that light one by one.
        let charge = ctx.eased.clamp(0.0, 1.0);
        for seg in 0..4 {
            let x0 = s(2.0 + seg as f32 * 3.0);
            let x1 = x0 + s(1.0);
            if charge > seg as f32 / 4.0 {
                fill_i(grid, x0, oy + s(5.0), x1, oy + s(10.0));
                let c = ctx.palette.sample(seg as f32 / 3.0);
                tint_dots(grid, x0, oy + s(5.0), x1, oy + s(10.0), c);
            } else {
                fill_i(grid, x0, oy + s(5.0), x1, oy + s(5.0));
                fill_i(grid, x0, oy + s(10.0), x1, oy + s(10.0));
            }
        }

        // The beam: a solid two-dot core with dashes racing along its halo.
        let x0 = s(26.0);
        let tip = x0 + ((wi - 1 - x0).max(0) as f32 * charge) as i32;
        let (core_a, core_b) = (oy + s(7.0), oy + s(8.0));
        let core_mid = (core_a + core_b) / 2;
        if charge > 0.01 && tip > x0 {
            fill_i(grid, x0, core_a, tip, core_b);
            let shift = (ctx.time * 40.0) as i32;
            for x in x0..=tip {
                if (x - shift).rem_euclid(5) < 2 {
                    draw::dot_i(grid, x, core_a - 1);
                    draw::dot_i(grid, x, core_b + 1);
                }
            }
            for cx in (x0 / 2)..=(tip / 2) {
                let t = (cx * 2 - x0) as f32 / (wi - x0).max(1) as f32;
                let c = ctx.palette.sample(t);
                tint_dots(grid, cx * 2, core_a - 1, cx * 2, core_b + 1, c);
            }
            // Muzzle flash flickers on alternate frames.
            if (ctx.time * 12.0) as i32 % 2 == 0 {
                fill_i(grid, x0, core_a - s(3.0), x0, core_a - s(2.0));
                fill_i(grid, x0, core_b + s(2.0), x0, core_b + s(3.0));
            }
            // Impact sparks spray from the tip.
            let frame = (ctx.time * 12.0) as u32;
            for r in 0..6u32 {
                let ang = hashf(frame.wrapping_mul(7).wrapping_add(r)) * 2.0 * PI;
                let len = (2.0 + hashf(frame.wrapping_mul(13).wrapping_add(r)) * 3.0) * k;
                let ex = tip + (ang.cos() * len) as i32;
                let ey = core_mid + (ang.sin() * len * 0.8) as i32;
                beam_line(grid, tip, core_mid, ex, ey, 8);
            }
            let spark = Color::rgb(255, 250, 225);
            tint_dots(grid, tip - 4, core_mid - 6, tip + 6, core_mid + 6, spark);
        }
        Ok(())
    }
}

// ─── 2. Scanning line ────────────────────────────────────────────────────────

/// A bright vertical beam sweeps left↔right; the swept region fills with eased.
struct ScanningLine;
impl ProgressStyle for ScanningLine {
    fn name(&self) -> &str {
        "scanning-line"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Vertical beam sweeps back and forth; swept fraction fills to eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Sweep: triangle wave in time.
        let period = 3.0f32;
        let t_norm = (ctx.time % period) / period; // 0..1
        let sweep_frac = if t_norm < 0.5 {
            t_norm * 2.0
        } else {
            2.0 - t_norm * 2.0
        };
        let sweep_x = (sweep_frac * w as f32) as usize;
        let sweep_x = sweep_x.min(w.saturating_sub(1));

        // Filled region up to eased progress: sparse horizontal lines.
        let filled = ((ctx.eased * w as f32) as usize).min(w);
        let mid = h / 2;
        // Three horizontal rails.
        draw::hline(
            grid,
            0,
            filled.saturating_sub(1),
            mid.saturating_sub(1).min(h - 1),
        );
        draw::hline(grid, 0, filled.saturating_sub(1), mid);
        draw::hline(
            grid,
            0,
            filled.saturating_sub(1),
            (mid + 1).min(h.saturating_sub(1)),
        );

        // Sweep beam: full-height vertical line with a narrow bright core.
        draw::vline(grid, sweep_x, 0, h.saturating_sub(1));
        // One-dot wings on each side for width.
        if sweep_x > 0 {
            draw::vline(grid, sweep_x - 1, h / 4, h * 3 / 4);
        }
        if sweep_x + 1 < w {
            draw::vline(grid, sweep_x + 1, h / 4, h * 3 / 4);
        }

        // Tint: filled region gets gradient; beam cell gets bright end.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        let beam_cell = (sweep_x / 2).min(cells_w.saturating_sub(1));
        let bright = ctx.palette.sample(1.0);
        for cy in 0..cells_h {
            draw::tint_row(grid, cy, beam_cell, beam_cell, bright);
        }

        Ok(())
    }
}

// ─── 3. Security grid ────────────────────────────────────────────────────────

/// A vault corridor of emitter posts with red tripwire beams criss-crossing
/// between each pair. Bays behind the progress front are disarmed (dim green,
/// dotted); bays ahead stay armed, with light pulses racing along the beams.
struct SecurityGrid;
impl ProgressStyle for SecurityGrid {
    fn name(&self) -> &str {
        "security-grid"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Corridor of emitter posts with red tripwires criss-crossing between them; bays disarm to green with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bound = (w + h) * 2;
        let bay = (hi * 3 / 4).max(6);
        let bays = ((wi - 2) / bay).max(1);
        let bay_w = ((wi - 2) / bays).max(2);
        let front = ctx.eased.clamp(0.0, 1.0) * bays as f32;
        let shift = (ctx.time * 24.0) as i32;
        let blink = (ctx.time * 3.0) as i32 % 2 == 0;
        let armed_col = Color::rgb(255, 50, 60);
        let safe_col = Color::rgb(70, 200, 110);
        let post_col = Color::rgb(140, 145, 165);

        for b in 0..bays {
            let x0 = b * bay_w + 2;
            let x1 = x0 + bay_w - 2;
            let disarmed = (b as f32) < front.floor();
            // Beam rows: X between the posts plus a waist-height line.
            for (ya, yb) in [(0, hi - 1), (hi - 1, 0)] {
                if disarmed {
                    for i in (0..=(x1 - x0)).step_by(3) {
                        let y = ya + (yb - ya) * i / (x1 - x0).max(1);
                        draw::dot_i(grid, x0 + i, y);
                    }
                } else {
                    beam_line(grid, x0, ya, x1, yb, bound);
                }
            }
            let waist = hi / 2;
            if !disarmed {
                draw::hline(grid, x0 as usize, x1.max(x0) as usize, waist as usize);
                // Pulses racing along the waist beam: gaps that move.
                for x in x0..=x1 {
                    if (x + shift).rem_euclid(9) == 0 {
                        draw::dot_i(grid, x, waist - 1);
                        draw::dot_i(grid, x, waist + 1);
                    }
                }
            }
            let col = if disarmed { safe_col } else { armed_col };
            for cy in 0..(hi / 4).max(1) {
                draw::tint_row(
                    grid,
                    cy as usize,
                    ((x0 + 1) / 2) as usize,
                    ((x1 - 1) / 2).max(0) as usize,
                    col,
                );
            }
        }

        // Emitter posts between bays, with blinking status lamps on top.
        for p in 0..=bays {
            let x = (p * bay_w).min(wi - 2);
            fill_i(grid, x, 1, x + 1, hi - 1);
            let armed = (p as f32) >= front.floor() && p < bays;
            if !armed || blink {
                fill_i(grid, x, 0, x + 1, 0);
            }
            let lamp = if armed { armed_col } else { safe_col };
            tint_dots(grid, x, 0, x + 1, hi - 1, post_col);
            tint_dots(grid, x, 0, x + 1, 0, lamp);
        }
        Ok(())
    }
}

// ─── 4. Prism dispersion ────────────────────────────────────────────────────

/// Seven-stop visible spectrum, red (`f = 0`) to violet (`f = 1`).
fn spectrum(f: f32) -> Color {
    const STOPS: [(u8, u8, u8); 7] = [
        (255, 45, 45),
        (255, 145, 20),
        (255, 230, 30),
        (60, 225, 80),
        (40, 140, 255),
        (95, 75, 240),
        (180, 80, 255),
    ];
    let i = ((f.clamp(0.0, 1.0) * 6.0).round() as usize).min(6);
    let (r, g, b) = STOPS[i];
    Color::rgb(r, g, b)
}

/// A glass triangular prism: a white beam strikes its left face, bends
/// through the glass, and leaves the right face as a rainbow fan that
/// reaches further across the bar with eased.
struct PrismDispersion;
impl ProgressStyle for PrismDispersion {
    fn name(&self) -> &str {
        "prism-dispersion"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "A white beam strikes a glass prism and fans out as a red-to-violet spectrum that reaches across with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        if h < 8 || w < 20 {
            let reach = (ctx.eased * w as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, h / 2);
            }
            return Ok(());
        }
        let bound = (w + h) * 2;

        // Prism: apex at the top, base on the bottom row.
        let px = (wi / 4).max(hi * 3 / 4);
        let half = (hi as f32 * 0.62) as i32;
        let base_y = hi - 1;
        let left_x = |y: i32| px - half * y / base_y.max(1);
        let right_x = |y: i32| px + half * y / base_y.max(1);
        beam_line(grid, px, 0, px - half, base_y, bound);
        beam_line(grid, px, 0, px + half, base_y, bound);
        beam_line(grid, px - half, base_y, px + half, base_y, bound);
        // A glint inside the glass, parallel to the left face.
        beam_line(grid, px + 1, hi / 4 + 1, px - half / 3, hi * 3 / 4, bound);
        tint_dots(
            grid,
            px - half,
            0,
            px + half,
            base_y,
            Color::rgb(190, 225, 255),
        );

        // Incoming white beam with photon gaps drifting toward the prism.
        let y_in = hi * 5 / 16;
        let xl = left_x(y_in);
        let shift = (ctx.time * 30.0) as i32;
        for x in 0..=xl {
            if (x - shift).rem_euclid(12) >= 2 {
                draw::dot_i(grid, x, y_in);
            }
        }
        tint_dots(grid, 0, y_in, xl - 1, y_in, Color::rgb(240, 240, 240));

        // Refracted path through the glass, bending toward the base.
        let y_out = y_in + hi / 6;
        let xr = right_x(y_out);
        beam_line(grid, xl, y_in, xr, y_out, bound);

        // Spectrum fan: rays from the exit point toward the right edge.
        let e = ctx.eased.clamp(0.0, 1.0);
        let n = (hi / 2).max(4);
        let span = (wi - 1 - xr).max(1);
        let tip_x = xr + (span as f32 * e) as i32;
        let frame = (ctx.time * 10.0) as u32;
        for i in 0..n {
            let ty = (i as f32 / (n - 1) as f32 * (hi - 1) as f32) as i32;
            let ey = y_out + ((ty - y_out) as f32 * e) as i32;
            beam_line(grid, xr, y_out, tip_x, ey, bound);
            // A twinkle travels down each ray.
            let along = ((ctx.time * 1.3 + hashf(i as u32)) % 1.0) * e;
            let tx = xr + (span as f32 * along) as i32;
            let tyy = y_out + ((ty - y_out) as f32 * along) as i32;
            if hash(frame.wrapping_add(i as u32)) % 3 != 0 {
                draw::dot_i(grid, tx, tyy - 1);
            }
        }

        // Colour each fan cell by where it sits across the spread.
        for cx in (xr / 2 + 1)..=(tip_x / 2) {
            let x = cx * 2 + 1;
            let t = (x - xr) as f32 / span as f32;
            let top = y_out as f32 * (1.0 - t);
            let bot = y_out as f32 + (hi - 1 - y_out) as f32 * t;
            for cy in 0..(hi / 4) {
                let yc = (cy * 4 + 2) as f32;
                let f = ((yc - top) / (bot - top).max(1.0)).clamp(0.0, 1.0);
                draw::tint_row(grid, cy as usize, cx as usize, cx as usize, spectrum(f));
            }
        }
        Ok(())
    }
}

// ─── 5. Laser light show ────────────────────────────────────────────────────

/// Draw a beam from `(x0,y0)` to `(x1,y1)`, tinting every cell it crosses.
/// `skip` drops dots where `(i + shift) % 7 == 0` for a shimmering beam.
fn tinted_beam(
    grid: &mut BrailleGrid,
    (x0, y0): (i32, i32),
    (x1, y1): (i32, i32),
    shift: i32,
    color: Color,
) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 512);
    for i in 0..=steps {
        let x = x0 + (x1 - x0) * i / steps;
        let y = y0 + (y1 - y0) * i / steps;
        if (i + shift).rem_euclid(7) != 0 {
            draw::dot_i(grid, x, y);
        }
        tint_dots(grid, x, y, x, y, color);
    }
}

/// Concert laser rig: projectors along the stage floor each throw a fan of
/// coloured beams up into the haze, sweeping side to side with time. More
/// projectors switch on as eased grows.
struct LaserLightShow;
impl ProgressStyle for LaserLightShow {
    fn name(&self) -> &str {
        "laser-light-show"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Concert laser rig: stage projectors throw sweeping fans of coloured beams; more projectors fire with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        const SHOW: [(u8, u8, u8); 5] = [
            (60, 255, 90),
            (255, 60, 200),
            (60, 200, 255),
            (255, 220, 50),
            (255, 70, 60),
        ];
        let n_proj = (wi / 16).clamp(1, 8);
        let active = (ctx.eased.clamp(0.0, 1.0) * n_proj as f32).ceil() as i32;
        let floor_y = hi - 1;
        let gap = wi / n_proj;
        let shift = (ctx.time * 20.0) as i32;

        for p in 0..n_proj {
            let px = gap / 2 + p * gap;
            let (r, g, b) = SHOW[p as usize % SHOW.len()];
            let col = Color::rgb(r, g, b);
            // Projector housing on the stage floor.
            fill_i(grid, px - 2, floor_y - 1, px + 2, floor_y);
            tint_dots(
                grid,
                px - 2,
                floor_y - 1,
                px + 2,
                floor_y,
                Color::rgb(120, 125, 145),
            );
            if p >= active {
                continue;
            }
            // Fan of three beams, sweeping with a per-projector phase.
            let sweep = (ctx.time * (1.1 + p as f32 * 0.23) + p as f32 * 1.7).sin() * 0.9;
            let rise = (floor_y - 2).max(1) as f32;
            for k in -1..=1 {
                let ang = sweep + k as f32 * 0.35;
                let ex = px + (ang.tan() * rise).clamp(-(wi as f32), wi as f32) as i32;
                tinted_beam(grid, (px, floor_y - 2), (ex, 0), shift + k * 3, col);
            }
            // Lens glow.
            tint_dots(
                grid,
                px - 1,
                floor_y - 2,
                px + 1,
                floor_y - 2,
                Color::rgb(255, 255, 255),
            );
            draw::dot_i(grid, px, floor_y - 2);
        }
        Ok(())
    }
}

// ─── 6. Range finder ────────────────────────────────────────────────────────

/// A laser range finder: a boxy handset with a lens fires a pulse at a
/// crosshair target and catches the echo. The target stands further away as
/// eased grows, over a ruler of distance ticks; the pulse shuttles out and
/// back with time.
struct RangeFinder;
impl ProgressStyle for RangeFinder {
    fn name(&self) -> &str {
        "range-finder"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Laser range finder pings a crosshair target that stands further away with progress, over a distance ruler"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = (hi - 2) / 2;

        // Handset: body outline, lens on the front, eyepiece on the back.
        let bw = (hi * 3 / 4).clamp(2, 12);
        let bh = (hi / 2).max(1);
        rect_i(grid, 1, mid - bh / 2, bw, mid + bh / 2);
        fill_i(grid, 0, mid - 1, 0, mid + 1);
        fill_i(grid, bw + 1, mid - 1, bw + 2, mid + 1);
        fill_i(grid, 3, mid - 1, 4, mid);
        let steel = Color::rgb(150, 156, 172);
        tint_dots(grid, 0, mid - bh / 2, bw + 2, mid + bh / 2, steel);

        // Target: crosshair reticle at the eased distance.
        let r = (hi / 3).clamp(1, 6);
        let x_lens = bw + 3;
        let near = x_lens + r + 4;
        let span = (wi - 1 - (r + 2) - near).max(0);
        let tx = near + (span as f32 * ctx.eased.clamp(0.0, 1.0)) as i32;
        let steps = (r * 8).max(8) as usize;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            draw::dot_i(
                grid,
                tx + (a.cos() * r as f32).round() as i32,
                mid + (a.sin() * r as f32).round() as i32,
            );
        }
        fill_i(grid, tx - r - 2, mid, tx - r + 1, mid);
        fill_i(grid, tx + r - 1, mid, tx + r + 2, mid);
        fill_i(grid, tx, mid - r - 2, tx, mid - r + 1);
        fill_i(grid, tx, mid + r - 1, tx, mid + r + 2);
        draw::dot_i(grid, tx, mid);
        let lock = Color::rgb(255, 70, 70);
        tint_dots(grid, tx - r - 2, mid - r - 2, tx + r + 2, mid + r + 2, lock);

        // Beam: a faint dotted path plus a bright pulse going out and back.
        let path = (tx - r - 3 - x_lens).max(0);
        for x in (x_lens..x_lens + path).step_by(3) {
            draw::dot_i(grid, x, mid);
        }
        let phase = (ctx.time * 1.2).fract();
        let along = if phase < 0.5 {
            phase * 2.0
        } else {
            2.0 - phase * 2.0
        };
        let px = x_lens + (path as f32 * along) as i32;
        fill_i(grid, px - 2, mid, px + 2, mid);
        fill_i(grid, px - 1, mid - 1, px + 1, mid + 1);
        if path > 0 {
            tint_dots(
                grid,
                x_lens,
                mid,
                x_lens + path,
                mid,
                ctx.palette.sample(0.3),
            );
            tint_dots(
                grid,
                px - 2,
                mid - 1,
                px + 2,
                mid + 1,
                ctx.palette.sample(1.0),
            );
        }

        // Distance ruler along the bottom, ticked up to the target.
        let ry = hi - 1;
        if ry > mid + r {
            draw::hline(grid, x_lens as usize, tx.max(x_lens) as usize, ry as usize);
            for x in (x_lens..=tx).step_by(4) {
                let tall = (x - x_lens) % 16 == 0;
                draw::dot_i(grid, x, ry - 1);
                if tall {
                    draw::dot_i(grid, x, ry - 2);
                }
            }
            tint_dots(grid, x_lens, ry, tx, ry, Color::rgb(120, 125, 145));
        }
        Ok(())
    }
}

// ─── 7. Mirror bounce ───────────────────────────────────────────────────────

/// A chain of hatched flat mirrors, alternating ceiling and floor. The beam
/// enters on the left and ricochets mirror to mirror, reaching further down
/// the chain as eased grows; each mirror it strikes glints, and photon
/// pulses stream along the lit path.
struct MirrorBounce;
impl ProgressStyle for MirrorBounce {
    fn name(&self) -> &str {
        "mirror-bounce"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "A beam ricochets between hatched mirrors on the ceiling and floor, reaching further down the chain with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let spacing = hi.max(4);
        let half_m = (spacing / 3).max(1);
        let (top_y, bot_y) = (2.min(hi - 1), (hi - 3).max(0));
        let silver = Color::rgb(170, 178, 200);
        let n = (wi - spacing / 2) / spacing + 1;
        let front = ctx.eased.clamp(0.0, 1.0) * wi as f32;
        let shift = (ctx.time * 24.0) as i32;

        // Mirrors: a reflective face with hatching on the back.
        let mirror_at = |i: i32| {
            (
                spacing / 2 + i * spacing,
                if i % 2 == 0 { top_y } else { bot_y },
            )
        };
        for i in 0..n {
            let (mx, my) = mirror_at(i);
            fill_i(grid, mx - half_m, my, mx + half_m, my);
            let back = if my == top_y { my - 1 } else { my + 1 };
            for x in (mx - half_m..=mx + half_m).step_by(2) {
                draw::dot_i(grid, x, back);
                draw::dot_i(grid, x + 1, back + (back - my));
            }
            tint_dots(
                grid,
                mx - half_m,
                my.min(back) - 1,
                mx + half_m,
                my.max(back) + 1,
                silver,
            );
        }

        // Beam path: enters at mid-left, then mirror to mirror.
        let mut prev = (0, hi / 2);
        let mut travelled = 0;
        for i in 0..=n {
            let next = if i < n {
                let (mx, my) = mirror_at(i);
                (mx, if my == top_y { my + 1 } else { my - 1 })
            } else {
                (wi - 1, hi / 2)
            };
            if prev.0 as f32 >= front {
                break;
            }
            let seg_w = (next.0 - prev.0).max(1);
            let reach = ((front - prev.0 as f32) / seg_w as f32).min(1.0);
            let end = (
                prev.0 + (seg_w as f32 * reach) as i32,
                prev.1 + ((next.1 - prev.1) as f32 * reach) as i32,
            );
            let steps = (end.0 - prev.0)
                .abs()
                .max((end.1 - prev.1).abs())
                .clamp(1, 512);
            for s in 0..=steps {
                let x = prev.0 + (end.0 - prev.0) * s / steps;
                let y = prev.1 + (end.1 - prev.1) * s / steps;
                draw::dot_i(grid, x, y);
                if (travelled + s - shift).rem_euclid(10) < 2 {
                    draw::dot_i(grid, x + 1, y);
                    draw::dot_i(grid, x, y + 1);
                }
                let t = x as f32 / wi as f32;
                tint_dots(grid, x, y, x, y, ctx.palette.sample(t));
            }
            travelled += steps;
            if reach >= 1.0 && i < n {
                // Glint where the beam strikes the mirror.
                let (mx, my) = mirror_at(i);
                if (ctx.time * 6.0 + i as f32) as i32 % 2 == 0 {
                    fill_i(grid, mx - 1, my, mx + 1, my);
                }
                tint_dots(grid, mx - 1, my, mx + 1, my, Color::rgb(255, 255, 255));
            }
            prev = next;
        }
        Ok(())
    }
}

// ─── 8. Fiber pulse ─────────────────────────────────────────────────────────

/// A gently curving optical fiber: two cladding walls around a core. Light
/// zigzags down the core by total internal reflection, lighting the fiber up
/// to eased; bright pulses ride the zigzag with time.
struct FiberPulse;
impl ProgressStyle for FiberPulse {
    fn name(&self) -> &str {
        "fiber-pulse"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Curving optical fiber: light zigzags down the core by total internal reflection, lit up to progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let hf = (hi - 1) as f32;
        // Fiber centre line bends gently; walls sit `half` dots either side.
        let bend = hf / 8.0;
        let half = (hf / 2.0 - bend - 1.0).max(1.0);
        let centre = |x: i32| hf / 2.0 + bend * (x as f32 / wi as f32 * 2.0 * PI).sin();
        let front = (ctx.eased.clamp(0.0, 1.0) * wi as f32) as i32;
        let clad = Color::rgb(90, 95, 120);

        for x in 0..wi {
            let c = centre(x);
            let (top, bot) = ((c - half).round() as i32, (c + half).round() as i32);
            // Cladding: solid where lit, dotted ahead.
            if x < front || x % 3 == 0 {
                draw::dot_i(grid, x, top - 1);
                draw::dot_i(grid, x, bot + 1);
            }
            // Zigzag ray: a triangle wave between the walls.
            if x < front {
                let period = (half * 3.0).max(2.0);
                let ph = (x as f32 / period).fract();
                let tri = if ph < 0.5 { ph * 2.0 } else { 2.0 - ph * 2.0 };
                let y = (c - half + tri * half * 2.0).round() as i32;
                draw::dot_i(grid, x, y);
                // Travelling pulses: a thicker blob riding the ray.
                let shift = ctx.time * 28.0;
                if (x as f32 - shift).rem_euclid(period * 2.0) < 3.0 {
                    draw::dot_i(grid, x, y - 1);
                    draw::dot_i(grid, x, y + 1);
                }
            }
        }
        // Colour: lit core in the palette gradient, cladding grey ahead.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let lit = (cx as i32) * 2 < front;
            let col = if lit {
                ctx.palette.sample(cx as f32 / cw.max(1) as f32)
            } else {
                clad
            };
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        // Glowing end face at the light front.
        if front > 0 && front < wi {
            let c = centre(front);
            line_v(
                grid,
                front,
                (c - half).round() as i32,
                (c + half).round() as i32,
            );
            tint_dots(grid, front, 0, front, hi - 1, Color::rgb(255, 255, 255));
        }
        Ok(())
    }
}

/// Vertical run `[y0, y1]` at column `x` (signed, clipped).
fn line_v(grid: &mut BrailleGrid, x: i32, y0: i32, y1: i32) {
    fill_i(grid, x, y0.min(y1), x, y0.max(y1));
}

// ─── 9. Plasma bolt ─────────────────────────────────────────────────────────

/// A glowing plasma-ball electrode on the left discharges a forked lightning
/// bolt whose reach grows with eased. The bolt's zigzag path and branches
/// re-strike every frame; a crackling corona surrounds the electrode.
struct PlasmaBolt;
impl ProgressStyle for PlasmaBolt {
    fn name(&self) -> &str {
        "plasma-bolt"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "A plasma-ball electrode discharges a forked lightning bolt that re-strikes each frame; reach = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bound = (w + h) * 2;
        let frame = (ctx.time * 12.0) as u32;
        let mid = hi / 2;

        // Electrode: a filled plasma ball with a flickering corona.
        let r = ((hi - 2) / 2).clamp(0, 6);
        let bx = r;
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    draw::dot_i(grid, bx + x, mid + y);
                }
            }
        }
        for k in 0..10u32 {
            let ang = hashf(frame.wrapping_mul(31).wrapping_add(k)) * 2.0 * PI;
            let rr = r as f32 + 1.5 + hashf(frame.wrapping_add(k * 7)) * 2.0;
            draw::dot_i(
                grid,
                bx + (ang.cos() * rr) as i32,
                mid + (ang.sin() * rr) as i32,
            );
        }
        tint_dots(
            grid,
            0,
            mid - r - 3,
            bx + r + 3,
            mid + r + 3,
            ctx.palette.sample(0.0),
        );

        // Main bolt: zigzag from the ball's edge to the reach point.
        let x_start = bx + r + 1;
        let reach = x_start + ((wi - 1 - x_start).max(0) as f32 * ctx.eased.clamp(0.0, 1.0)) as i32;
        if reach <= x_start {
            return Ok(());
        }
        let seg = (hi / 3).max(3);
        let amp = (hi / 2 - 1).max(0);
        let (mut px, mut py) = (x_start, mid);
        let mut n = 0u32;
        while px < reach {
            let nx = (px + seg).min(reach);
            let jitter = hashf(frame.wrapping_mul(97).wrapping_add(n)) * 2.0 - 1.0;
            let ny = if nx >= reach {
                mid
            } else {
                mid + (jitter * amp as f32) as i32
            };
            beam_line(grid, px, py, nx, ny, bound);
            beam_line(grid, px, py + 1, nx, ny + 1, bound);
            // Every few kinks a short fork splits off.
            if hash(frame.wrapping_add(n * 13)) % 3 == 0 {
                let fy = if ny < mid { ny - seg } else { ny + seg };
                beam_line(grid, nx, ny, nx + seg, fy, bound);
            }
            let c = ctx.palette.sample(px as f32 / wi as f32);
            tint_dots(grid, px, 0, nx, hi - 1, c);
            px = nx;
            py = ny;
            n += 1;
        }
        // White-hot strike point.
        tint_dots(
            grid,
            reach - 2,
            0,
            reach + 2,
            hi - 1,
            Color::rgb(255, 255, 255),
        );
        for k in 0..5u32 {
            let ang = hashf(frame.wrapping_mul(5).wrapping_add(k + 50)) * 2.0 * PI;
            beam_line(
                grid,
                reach,
                mid,
                reach + (ang.cos() * 3.0) as i32,
                mid + (ang.sin() * 3.0) as i32,
                6,
            );
        }
        Ok(())
    }
}

// ─── 10. Particle accelerator ───────────────────────────────────────────────

/// A synchrotron ring seen from above: a double-walled oval beam pipe with
/// dipole magnets. The energised arc of the ring grows with eased while a
/// particle bunch laps it, faster and with a longer streak as it charges.
struct ParticleAccelerator;
impl ProgressStyle for ParticleAccelerator {
    fn name(&self) -> &str {
        "particle-accelerator"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Synchrotron ring with magnets: the energised arc grows with progress as a particle bunch laps ever faster"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        if h < 8 || w < 16 {
            let reach = (ctx.eased * w as f32) as usize;
            if reach > 0 {
                draw::hline(grid, 0, reach - 1, h / 2);
            }
            return Ok(());
        }
        let (cx, cy) = ((w as f32 - 1.0) / 2.0, (h as f32 - 1.0) / 2.0);
        let (ao, bo) = (cx - 1.0, cy - 0.5);
        let (am, bm) = (ao - 2.0, bo - 2.0);
        let (ai, bi) = (ao - 4.0, bo - 4.0);
        // Angle 0 is the bottom of the ring (injection point), running clockwise.
        let at = |a: f32, b: f32, ang: f32| {
            (
                (cx - ang.sin() * a).round() as i32,
                (cy + ang.cos() * b).round() as i32,
            )
        };
        let steps = ((ao + bo) * 4.0) as usize;
        let pipe = Color::rgb(95, 100, 125);

        // Beam pipe walls.
        for s in 0..steps {
            let ang = s as f32 / steps as f32 * 2.0 * PI;
            let (x, y) = at(ao, bo, ang);
            draw::dot_i(grid, x, y);
            let (x, y) = at(ai.max(1.0), bi.max(0.5), ang);
            draw::dot_i(grid, x, y);
            draw::tint_row(
                grid,
                (y.max(0) / 4) as usize,
                (x.max(0) / 2) as usize,
                (x.max(0) / 2) as usize,
                pipe,
            );
            let (x, y) = at(ao, bo, ang);
            tint_dots(grid, x, y, x, y, pipe);
        }
        // Dipole magnets: short bars straddling the pipe.
        for m in 0..10 {
            let ang = (m as f32 + 0.5) / 10.0 * 2.0 * PI;
            let (x0, y0) = at(ao + 1.0, bo + 1.0, ang);
            let (x1, y1) = at(ai - 1.0, bi - 1.0, ang);
            beam_line(grid, x0, y0, x1, y1, 16);
            beam_line(grid, x0 + 1, y0, x1 + 1, y1, 16);
        }

        // Energised arc: the beam inside the pipe, length = eased.
        let e = ctx.eased.clamp(0.0, 1.0);
        let lit = (steps as f32 * e) as usize;
        for s in 0..lit {
            let f = s as f32 / steps as f32;
            let (x, y) = at(am, bm, f * 2.0 * PI);
            draw::dot_i(grid, x, y);
            tint_dots(grid, x, y, x, y, ctx.palette.sample(f));
        }

        // Particle bunch laps the ring; speed and streak grow with charge.
        let lap = ctx.time * (0.25 + e * 0.9);
        let head = lap.fract() * 2.0 * PI;
        let tail = 0.25 + e * 1.1;
        let n_tail = (tail * am / 2.0) as usize + 2;
        for i in 0..n_tail {
            let ang = head - tail * i as f32 / n_tail as f32;
            let (x, y) = at(am, bm, ang);
            draw::dot_i(grid, x, y);
            if i < 3 {
                fill_i(grid, x - 1, y - 1, x + 1, y + 1);
            }
            let c = if i < n_tail / 3 {
                Color::rgb(255, 255, 255)
            } else {
                ctx.palette.sample(1.0)
            };
            tint_dots(grid, x, y, x, y, c);
        }
        Ok(())
    }
}
// ─── 11. Disco fan ──────────────────────────────────────────────────────────

/// Radial beams sweep from a corner origin; sweep angle grows with eased + time.
struct DiscoFan;
impl ProgressStyle for DiscoFan {
    fn name(&self) -> &str {
        "disco-fan"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Radial fan of beams sweeps from a corner; beam count and arc = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Fan origin: bottom-left corner.
        let ox = 0i32;
        let oy = h as i32 - 1;

        // Number of beams.
        let n_beams = ((ctx.eased * 9.0 + 1.0) as usize).clamp(1, 10);

        // The fan sweeps continuously via time, occupying an arc that grows with eased.
        let arc = ctx.eased * PI * 0.9 + 0.1; // arc in radians, 0.1..~π·0.9
        let sweep_center = PI / 2.0 * (0.4 + 0.6 * ctx.eased) - (ctx.time * 0.6).sin() * arc * 0.3; // centre oscillates

        for b in 0..n_beams {
            let angle_frac = if n_beams == 1 {
                0.5f32
            } else {
                b as f32 / (n_beams - 1) as f32
            };
            let angle = sweep_center - arc / 2.0 + angle_frac * arc;

            // Beam extends to the far edge of the grid.
            let beam_len = ((w as f32).hypot(h as f32) as i32 + 2).max(2);
            let end_x = ox + (angle.cos() * beam_len as f32) as i32;
            let end_y = oy - (angle.sin() * beam_len as f32) as i32;

            beam_line(grid, ox, oy, end_x, end_y, (w + h) * 2);

            // Tint each beam column.
            let t = b as f32 / n_beams.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            // Paint a stripe along the beam.
            let x_end = end_x.clamp(0, cells_w as i32 - 1) as usize;
            let x_start = 0usize;
            let (lo, hi) = if x_start <= x_end {
                (x_start, x_end)
            } else {
                (x_end, x_start)
            };
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, lo, hi.min(cells_w.saturating_sub(1)), color);
            }
        }

        // Bright origin point.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, ox + dx, oy + dy);
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
    let styles = progress::styles::lasers::styles();
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
