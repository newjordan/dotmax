//! `cars` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O cars.rs && ./cars [style-name]
//! ```

const DEFAULT_STYLE: &str = "drag-tree";

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
    pub mod cars {
//! Cars / Automotive / Racing progress bars — the **cars** theme.
//!
//! Every bar is **structurally** distinct: each uses a different geometric
//! primitive, spatial layout, or temporal mechanic.  Color alone never
//! separates two styles here.
//!
//! # Style catalogue
//!
//! | name | mechanic |
//! |---|---|
//! | `drag-tree`       | Lamp tree: amber cascade → GREEN, dragster launches |
//! | `tachometer`      | Round ticked dial, redline arc, RPM digit, SHIFT lamp |
//! | `fuel-gauge`      | Round E→F dial needle + fuel-pump icon, low-fuel blink |
//! | `lap-circuit`     | Dot car circuits an oval track; lap fraction = eased |
//! | `pit-stop`        | Car on jacks, crew swaps tyres; pit board ticks off 4 tyres |
//! | `gear-shifter`    | H-pattern stick moves through gears 1→6 |
//! | `odometer`        | Framed number drums roll 00000→99999 |
//! | `tunnel-headlights` | Perspective converging road + headlight beams advance |
//! | `burnout`         | Car on the line spins its rear wheel; smoke billows back |
//! | `pistons`         | Engine cutaway: pistons on a crankshaft, spark flashes |
//! | `nitro-boost`     | Car rides a nitro flame jet past speed lines |
//! | `checkered-flag`  | Chequered flag on a pole unfurls and ripples |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Shared Bresenham line helper (bounds-safe via draw::dot_i)
// ─────────────────────────────────────────────────────────────────────────────

fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let px = x0 + (x1 - x0) * i / steps;
        let py = y0 + (y1 - y0) * i / steps;
        draw::dot_i(grid, px, py);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dot-art helpers shared by the gauge / odometer / flag styles
// ─────────────────────────────────────────────────────────────────────────────

/// Stamp a sprite: every `#` in `rows` sets the dot at `(x + col, y + row)`.
fn sprite(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, l) in rows.iter().enumerate() {
        for (c, b) in l.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + c as i32, y + r as i32);
            }
        }
    }
}

/// 3×5 digit font for odometer drums.
const DIGITS: [[&str; 5]; 10] = [
    ["###", "#.#", "#.#", "#.#", "###"],
    [".#.", "##.", ".#.", ".#.", "###"],
    ["###", "..#", "###", "#..", "###"],
    ["###", "..#", "###", "..#", "###"],
    ["#.#", "#.#", "###", "..#", "..#"],
    ["###", "#..", "###", "..#", "###"],
    ["###", "#..", "###", "#.#", "###"],
    ["###", "..#", "..#", "..#", "..#"],
    ["###", "#.#", "###", "#.#", "###"],
    ["###", "#.#", "###", "..#", "###"],
];

/// A round gauge dial: semicircle face with ticks, pivot at `(cx, cy)` on the
/// bottom edge, and a needle at `frac` (0 = far left, 1 = far right).
fn dial(grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32, frac: f32) {
    let rf = r as f32;
    let steps = (r * 8).max(8);
    for s in 0..=steps {
        let a = PI * s as f32 / steps as f32;
        draw::dot_i(
            grid,
            cx + (a.cos() * rf).round() as i32,
            cy - (a.sin() * rf).round() as i32,
        );
    }
    for t in 0..=8 {
        let a = PI * t as f32 / 8.0;
        let len = if t % 2 == 0 { 3.0 } else { 2.0 };
        line(
            grid,
            cx + (a.cos() * (rf - len)).round() as i32,
            cy - (a.sin() * (rf - len)).round() as i32,
            cx + (a.cos() * (rf - 1.0)).round() as i32,
            cy - (a.sin() * (rf - 1.0)).round() as i32,
        );
    }
    let a = PI * (1.0 - frac.clamp(0.0, 1.0));
    line(
        grid,
        cx,
        cy,
        cx + (a.cos() * (rf - 3.0)).round() as i32,
        cy - (a.sin() * (rf - 3.0)).round() as i32,
    );
    draw::fill_rect(
        grid,
        (cx - 1).max(0) as usize,
        (cy - 1).max(0) as usize,
        3,
        2,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `cars` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per automotive/racing bar.
/// Every style is geometrically distinct — they differ in their spatial
/// layout and temporal mechanic, never merely in color.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(DragTree),
        Box::new(Tachometer),
        Box::new(FuelGauge),
        Box::new(LapCircuit),
        Box::new(PitStop),
        Box::new(GearShifter),
        Box::new(Odometer),
        Box::new(TunnelHeadlights),
        Box::new(Burnout),
        Box::new(Pistons),
        Box::new(NitroBoost),
        Box::new(CheckeredFlag),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Drag-race Christmas tree
//    The starting-line light tree: a centre pole carrying two columns of
//    round lamps — three ambers then the green. Lamps light top-to-bottom:
//    progress < 0.15: pre-stage (top amber pair blinks with time)
//    0.15..0.45 / 0.45..0.70 / 0.70..0.85: amber 1 / 2 / 3 light up
//    0.85..1.00: GREEN — the green pair flickers and the staged dragster
//    (big rear slick, long body, wing) launches down the strip.
// ─────────────────────────────────────────────────────────────────────────────

struct DragTree;

impl ProgressStyle for DragTree {
    fn name(&self) -> &str {
        "drag-tree"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Drag-race Christmas tree: amber lamp pairs light top-to-bottom, then GREEN fires and the staged dragster launches down the strip"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.progress;

        // The tree: pole plus four rows of lamp pairs (3 amber, 1 green).
        draw::vline(grid, 10, 0, h - 1);
        let lit_rows = if p >= 0.85 {
            4
        } else if p >= 0.70 {
            3
        } else if p >= 0.45 {
            2
        } else {
            i32::from(p >= 0.15)
        };
        let blink = (ctx.time * 4.0).fract() < 0.5;
        for row in 0..4 {
            let y = row * 4;
            let lit = if lit_rows == 4 {
                row == 3 && (blink || p >= 1.0)
            } else {
                row < lit_rows || (row == 0 && p < 0.15 && blink)
            };
            let lamp: [&str; 3] = if lit {
                ["###", "###", "###"]
            } else {
                [".#.", "#.#", ".#."]
            };
            sprite(grid, 5, y, &lamp);
            sprite(grid, 13, y, &lamp);
            draw::hline(grid, 8, 12, (y + 1) as usize);
        }

        // Track: a dashed lane line along the bottom.
        for x in (20..wi).filter(|x| x % 4 < 2) {
            draw::dot_i(grid, x, hi - 1);
        }

        // Dragster: wing over a huge rear slick, long body, small front wheel.
        let launch = ((p - 0.85) / 0.15).clamp(0.0, 1.0);
        let cx = 20 + (launch * launch * (wi - 43).max(0) as f32) as i32;
        sprite(
            grid,
            cx,
            hi - 9,
            &[
                "#####.................",
                "..#...................",
                "..#####...............",
                "######################",
                "#....#............##..",
                "#....#...........#..#.",
                "#....#...........#..#.",
                "######............##..",
            ],
        );
        // Tyre smoke while launching.
        if launch > 0.0 && launch < 1.0 {
            for k in 0..4 {
                let sx = cx - 2 - k * 2;
                let sy = hi - 4 - (((ctx.time * 9.0 + k as f32).sin() + 1.0) as i32);
                draw::dot_i(grid, sx, sy);
            }
        }
        Ok(())
    }
}
// ─────────────────────────────────────────────────────────────────────────────
// 2. Tachometer
//    A round semicircular dial with tick marks; the needle sweeps from idle
//    (left) to redline (right) as eased grows. The redline zone is drawn as a
//    thick arc over the last fifth of the face, and a round SHIFT lamp beside
//    the dial flashes with time once progress reaches 85%. A digital readout
//    of RPM ×1000 sits to the right.
// ─────────────────────────────────────────────────────────────────────────────

struct Tachometer;

impl ProgressStyle for Tachometer {
    fn name(&self) -> &str {
        "tachometer"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Tachometer: a round ticked dial whose needle sweeps idle→redline, a thick redline arc, a flashing SHIFT lamp and an RPM digit readout"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;
        let r = (hi - 1).clamp(1, 15);
        let cx = r + 1;
        // Needle jitters slightly with time, like a revving engine.
        let rev = ctx.eased + 0.015 * (ctx.time * 23.0).sin();
        dial(grid, cx, hi - 1, r, rev);

        // Redline: a thickened arc over the last fifth of the face.
        for s in 0..=12 {
            let a = PI * 0.2 * s as f32 / 12.0;
            for rr in [r - 1, r - 2] {
                draw::dot_i(
                    grid,
                    cx + (a.cos() * rr as f32).round() as i32,
                    hi - 1 - (a.sin() * rr as f32).round() as i32,
                );
            }
        }

        // RPM ×1000 readout (0–8) and the SHIFT lamp.
        let rpm = ((ctx.eased * 8.0).round() as usize).min(8);
        let dx = cx * 2 + 4;
        for (row, bits) in DIGITS[rpm].iter().enumerate() {
            for (col, b) in bits.bytes().enumerate() {
                if b == b'#' {
                    let (px, py) = (dx + col as i32 * 2, hi / 2 - 5 + row as i32 * 2);
                    for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        draw::dot_i(grid, px + ox, py + oy);
                    }
                }
            }
        }
        let lamp_x = dx + 10;
        let lit = ctx.progress >= 0.85 && (ctx.time * 6.0).fract() < 0.5;
        let lamp: [&str; 5] = if lit {
            [".###.", "#####", "#####", "#####", ".###."]
        } else {
            [".###.", "#...#", "#...#", "#...#", ".###."]
        };
        sprite(grid, lamp_x, hi / 2 - 3, &lamp);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Fuel gauge
//    A round dashboard fuel dial: E on the left, F on the right, the needle
//    pivoting from E to F as eased grows. Beside it a fuel-pump icon (body,
//    window, nozzle hose); below 15% the pump blinks as the low-fuel warning.
// ─────────────────────────────────────────────────────────────────────────────

struct FuelGauge;

impl ProgressStyle for FuelGauge {
    fn name(&self) -> &str {
        "fuel-gauge"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Fuel gauge: a round E→F dial whose needle rises with progress, beside a fuel-pump icon that blinks as the low-fuel light below 15%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;
        let r = (hi - 1).clamp(1, 15);
        let cx = r + 5;
        dial(grid, cx, hi - 1, r, ctx.eased);
        // E and F at the dial's feet.
        sprite(grid, 0, hi - 5, &["###", "#..", "##.", "#..", "###"]);
        sprite(
            grid,
            cx + r + 2,
            hi - 5,
            &["###", "#..", "##.", "#..", "#.."],
        );

        // Fuel pump: body with a window, nozzle hose hooked on the side.
        let low = ctx.progress < 0.15 && (ctx.time * 3.0).fract() < 0.5;
        if !low {
            sprite(
                grid,
                cx + r + 8,
                hi / 2 - 6,
                &[
                    "#######..",
                    "#.....#..",
                    "#.....##.",
                    "#.....#.#",
                    "#######.#",
                    "#######.#",
                    "#######.#",
                    "#######.#",
                    "#######..",
                    "#######..",
                    "#########",
                ],
            );
        }

        // A fuel bar to the right that mirrors the tank level.
        let x0 = cx + r + 20;
        let bar = (w as i32 - x0 - 1).max(0);
        draw::rect_outline(
            grid,
            x0.max(0) as usize,
            (h / 2).saturating_sub(3),
            bar as usize,
            6,
        );
        let fill = (ctx.eased * (bar - 4).max(0) as f32) as usize;
        if fill > 0 {
            draw::fill_rect(
                grid,
                (x0 + 2).max(0) as usize,
                (h / 2).saturating_sub(1),
                fill,
                2,
            );
        }
        Ok(())
    }
}
// ─────────────────────────────────────────────────────────────────────────────
// 4. Lap circuit
//    A dot-space oval: two straight sections + two semicircular ends.
//    A "car" dot orbits the oval continuously; its position is the eased lap
//    fraction (so at 0% it's at the start/finish line, at 100% it completes).
//    time drives a second "ghost" car one lap behind for visual interest.
//    The track is drawn as an outline; the portion the lead car has covered
//    is filled with a denser double-line.
// ─────────────────────────────────────────────────────────────────────────────

struct LapCircuit;
impl ProgressStyle for LapCircuit {
    fn name(&self) -> &str {
        "lap-circuit"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Oval track: a car dot circuits the loop; lap fraction = eased; ghost car trails behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as f32;
        let cy = (dh / 2) as f32;
        // Oval: horizontal semi-axes larger than vertical
        let rx = (cx * 0.80).max(1.0);
        let ry = (cy * 0.65).max(1.0);

        // Draw oval outline
        let steps = ((rx + ry) * 4.0).max(32.0) as usize;
        let mut prev: Option<(i32, i32)> = None;
        for s in 0..=steps {
            let angle = s as f32 / steps as f32 * 2.0 * PI;
            let px = (cx + rx * angle.cos()) as i32;
            let py = (cy + ry * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ppx, ppy)) = prev {
                if (px - ppx).abs() + (py - ppy).abs() > 2 {
                    line(grid, ppx, ppy, px, py);
                }
            }
            prev = Some((px, py));
        }

        // Inner oval (slightly smaller) to give track width
        let rx2 = (rx * 0.75).max(0.5);
        let ry2 = (ry * 0.70).max(0.5);
        let steps2 = ((rx2 + ry2) * 3.0).max(20.0) as usize;
        let mut prev2: Option<(i32, i32)> = None;
        for s in 0..=steps2 {
            let angle = s as f32 / steps2 as f32 * 2.0 * PI;
            let px = (cx + rx2 * angle.cos()) as i32;
            let py = (cy + ry2 * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ppx, ppy)) = prev2 {
                if (px - ppx).abs() + (py - ppy).abs() > 2 {
                    line(grid, ppx, ppy, px, py);
                }
            }
            prev2 = Some((px, py));
        }

        // Start/finish line: vertical line at the rightmost point of the oval
        let sf_x = (cx + rx) as i32;
        let sf_inner_x = (cx + rx2) as i32;
        let sf_y0 = (cy - ry * 0.15) as i32;
        let sf_y1 = (cy + ry * 0.15) as i32;
        line(grid, sf_x, sf_y0, sf_inner_x, sf_y0);
        line(grid, sf_x, sf_y1, sf_inner_x, sf_y1);

        // Car position: start at right (angle=0), circuit counter-clockwise
        // angle = -eased * 2π  (negative = counter-clockwise from right)
        let car_angle = -ctx.eased * 2.0 * PI;
        let car_r_x = (rx + rx2) / 2.0; // midpoint of track
        let car_r_y = (ry + ry2) / 2.0;
        let car_x = (cx + car_r_x * car_angle.cos()) as i32;
        let car_y = (cy + car_r_y * car_angle.sin()) as i32;
        // Car: 3-dot symbol
        draw::dot_i(grid, car_x, car_y);
        draw::dot_i(grid, car_x - 1, car_y);
        draw::dot_i(grid, car_x + 1, car_y);

        // Ghost car (trails by ~0.25 of a lap, driven by time)
        let ghost_offset = (ctx.time * 0.15).fract() * 2.0 * PI;
        let ghost_angle = car_angle - ghost_offset;
        let gx = (cx + car_r_x * ghost_angle.cos()) as i32;
        let gy = (cy + car_r_y * ghost_angle.sin()) as i32;
        draw::dot_i(grid, gx, gy);

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Side-view race car shared by pit-stop, burnout and nitro-boost
// ─────────────────────────────────────────────────────────────────────────────

/// Race-car body facing right: rear wing, cockpit canopy with driver, long
/// nose. 30 dots wide; wheel arches are cut out by [`race_car`].
const CAR_BODY: [&str; 5] = [
    "##...........######...........",
    "##.........##..##..##.........",
    "#############..##....#######..",
    "##############################",
    "#############################.",
];

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

/// Rear/front wheel centres relative to the car's top-left.
const WHEEL_DX: [i32; 2] = [6, 23];

/// A spoked wheel of radius 3 centred at `(cx, cy)`; `spin` turns the spokes.
fn wheel(grid: &mut BrailleGrid, cx: i32, cy: i32, spin: f32) {
    for dy in -3i32..=3 {
        for dx in -3i32..=3 {
            let d = dx * dx + dy * dy;
            if (7..=12).contains(&d) {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }
    }
    draw::dot_i(grid, cx, cy);
    for k in 0..2 {
        let a = spin + k as f32 * PI;
        draw::dot_i(
            grid,
            cx + (a.cos() * 1.6).round() as i32,
            cy + (a.sin() * 1.6).round() as i32,
        );
    }
}

/// Side-view race car, top-left at `(x, y)`, 9 dots tall (wheel bottoms on
/// row `y + 8`). `wheels[i]` false leaves that wheel off (rear, front).
fn race_car(grid: &mut BrailleGrid, x: i32, y: i32, spin: f32, wheels: [bool; 2]) {
    for (r, row) in CAR_BODY.iter().enumerate() {
        for (c, b) in row.bytes().enumerate() {
            if b != b'#' {
                continue;
            }
            let (px, py) = (c as i32, r as i32);
            let arch = WHEEL_DX
                .iter()
                .any(|&wx| (px - wx) * (px - wx) + (py - 5) * (py - 5) <= 16);
            if !arch {
                draw::dot_i(grid, x + px, y + py);
            }
        }
    }
    for (i, &wx) in WHEEL_DX.iter().enumerate() {
        if wheels[i] {
            wheel(grid, x + wx, y + 5, spin);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pit stop
//    The race car sits up on jacks, wheels off the ground; a crew member at
//    each end pulls a tyre and bolts a fresh one on with a sparking wheel gun.
//    The pit board on the right ticks off the four tyres (eased → 0..4).
// ─────────────────────────────────────────────────────────────────────────────

struct PitStop;
impl ProgressStyle for PitStop {
    fn name(&self) -> &str {
        "pit-stop"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Race car up on jacks while the pit crew pulls and bolts on fresh tyres; the pit board ticks off four tyres as progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;
        let done = ((ctx.eased * 4.0) as i32).min(4);

        let board_w = 20;
        let x = ((wi - board_w) / 2 - 15).max(8);
        let y = hi - 12; // lifted two dots: wheels hang off the ground
        let ground = hi - 1;
        for gx in 0..wi - board_w - 1 {
            if gx % 3 != 0 {
                draw::dot_i(grid, gx, ground);
            }
        }

        // Tyre being swapped: rear for tyres 0-1, front for 2-3.
        let u = (t * 0.9).fract();
        let pull = (tri(u) * 7.0) as i32;
        let side = i32::from(done >= 2);
        let working = done < 4;
        race_car(
            grid,
            x,
            y,
            0.0,
            [!(working && side == 0), !(working && side == 1)],
        );
        let mut wheel_pos = (0, 0);
        if working {
            let dir = if side == 0 { -1 } else { 1 };
            let cx = x + WHEEL_DX[side as usize] + dir * pull;
            wheel(grid, cx, y + 5, t * 3.0);
            wheel_pos = (cx, y + 5);
            if pull <= 1 && (t * 12.0).fract() < 0.5 {
                // Wheel-gun sparks.
                for (dx, dy) in [(-1, -4), (1, -4), (2, 4), (-2, 4)] {
                    draw::dot_i(grid, cx + dx, y + 5 + dy);
                }
            }
        }

        // Jacks under the rear wing and the nose.
        for jx in [x + 1, x + 28] {
            for jy in y + 5..ground {
                draw::dot_i(grid, jx, jy);
            }
            sprite(grid, jx - 2, ground - 1, &[".###.", "#####"]);
        }

        // Crew: one crouched behind the rear wheel, one ahead of the nose.
        for (k, base) in [(0, x - 9), (1, x + 31)] {
            let reach = if working && side == k {
                if k == 0 {
                    -pull
                } else {
                    pull
                }
            } else {
                0
            };
            let cx = base + reach;
            let face_right = k == 0;
            sprite(grid, cx + 2, ground - 9, &["##", "##"]);
            for yy in ground - 7..ground - 3 {
                draw::dot_i(grid, cx + 2, yy);
            }
            // Arms toward the wheel, legs crouched.
            let (a0, a1) = if face_right { (3, 6) } else { (-3, 1) };
            for ax in a0..a1 {
                draw::dot_i(grid, cx + ax, ground - 5);
            }
            sprite(grid, cx, ground - 3, &["#.#..", "#..#.", "#...#"]);
        }

        // Pit board: four tyre boxes, filled as each is changed.
        let bx = wi - board_w;
        let bh = (hi - 5).max(6);
        draw::rect_outline(
            grid,
            bx.max(0) as usize,
            0,
            (board_w - 1).max(0) as usize,
            bh as usize,
        );
        for k in 0..4 {
            let tx = bx + 2 + k * 4;
            if k < done {
                block(grid, tx, 2, 3, 3);
            } else {
                sprite(grid, tx, 2, &["###", "#.#", "###"]);
            }
        }
        sprite(grid, bx + 8, 6, &DIGITS[done as usize]);
        for py in bh..hi {
            draw::dot_i(grid, bx + board_w / 2, py);
        }

        // Colour: red car, orange crew, yellow board, steel jacks, tyres.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let color = if px >= bx - 1 {
                    Color::rgb(250, 220, 60)
                } else if working && (px - wheel_pos.0).abs() <= 3 && (py - wheel_pos.1).abs() <= 3
                {
                    Color::rgb(200, 200, 210) // tyre on the move
                } else if px < x - 1 || px > x + 30 {
                    Color::rgb(255, 150, 40) // crew
                } else if py >= ground - 1 {
                    Color::rgb(160, 170, 190) // jacks
                } else {
                    Color::rgb(232, 44, 44)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Gear shifter
//    An H-pattern stick shift is drawn in dot space.
//    The horizontal bar connects gear columns; vertical lines drop to each gate.
//    As eased grows 0→1, the stick position moves through gears: 1, 2, 3, 4, 5, 6.
//    The stick is a small dot that hops between gate positions.
//    time adds a subtle vibration while in gear (engine shake).
// ─────────────────────────────────────────────────────────────────────────────

struct GearShifter;
impl ProgressStyle for GearShifter {
    fn name(&self) -> &str {
        "gear-shifter"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "H-pattern gear shifter: stick hops 1→6 through the gate as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // H-pattern: 3 columns (left / center / right), 2 rows (up / down)
        // Gears: 1=top-left, 2=bottom-left, 3=top-center, 4=bottom-center, 5=top-right, 6=bottom-right
        // Reverse (R) lives below 6 — we skip it for the progress mapping.

        let col_count = 3usize;
        let margin_x = (dw / 5).max(1);
        let margin_y = (dh / 5).max(1);
        let usable_w = dw.saturating_sub(margin_x * 2);
        let usable_h = dh.saturating_sub(margin_y * 2);

        // Column x positions
        let col_xs: [i32; 3] = [
            (margin_x) as i32,
            (margin_x + usable_w / 2) as i32,
            (margin_x + usable_w) as i32,
        ];

        // Row y positions (top row = up-gears, bottom = down-gears)
        let row_ys: [i32; 2] = [(margin_y) as i32, (margin_y + usable_h) as i32];

        // Horizontal H-bar connecting all three columns at the midpoint
        let mid_y = (margin_y + usable_h / 2) as i32;
        line(grid, col_xs[0], mid_y, col_xs[col_count - 1], mid_y);

        // Vertical gate lines from mid_y to each gear slot
        for &col_x in &col_xs {
            line(grid, col_x, mid_y, col_x, row_ys[0]);
            line(grid, col_x, mid_y, col_x, row_ys[1]);
        }

        // Gate dots at each gear position
        for &col_x in &col_xs {
            for &row_y in &row_ys {
                draw::dot_i(grid, col_x - 1, row_y);
                draw::dot_i(grid, col_x, row_y);
                draw::dot_i(grid, col_x + 1, row_y);
            }
        }

        // Gear index from eased: 0..5 → gears 1..6
        let num_gears = 6usize;
        let gear_idx = ((ctx.eased * num_gears as f32).floor() as usize).min(num_gears - 1);
        let col = gear_idx / 2;
        let row = gear_idx % 2;
        let col = col.min(col_count - 1);

        // Engine vibration when in gear
        let vib_x = if ctx.progress > 0.01 {
            ((ctx.time * 30.0).sin() * 1.5) as i32
        } else {
            0
        };

        let stick_x = col_xs[col] + vib_x;
        let stick_y = row_ys[row];

        // Stick: 3×3 filled dot cluster
        for dy in -2_i32..=2 {
            for dx in -2_i32..=2 {
                draw::dot_i(grid, stick_x + dx, stick_y + dy);
            }
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col_c = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col_c);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Odometer
//    A dashboard odometer: a row of framed number drums reading 00000→99999
//    as eased grows. The right-most drum turns continuously with time (its
//    digit slides up as the next rolls in), and a drum above a run of 9s
//    turns with it, exactly like a mechanical counter's carry.
// ─────────────────────────────────────────────────────────────────────────────

struct Odometer;

impl ProgressStyle for Odometer {
    fn name(&self) -> &str {
        "odometer"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Odometer: framed number drums count 00000→99999 with progress; the last drum rolls smoothly, each digit sliding up as the next rolls in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let drums = 5;
        let pitch = 11;
        let x0 = ((wi - drums * pitch) / 2).max(0);
        let value = (ctx.eased.clamp(0.0, 1.0) * 99_999.0).round();
        // The units drum turns continuously while the car moves.
        let turn = (ctx.time * 1.5).fract();

        for d in 0..drums {
            let x = x0 + d * pitch;
            draw::rect_outline(grid, x.max(0) as usize, 0, 10, h);
            // Place value of this drum; the units drum rolls with the fraction,
            // higher drums only roll while everything below them reads 9.
            let place = 10_f32.powi(drums - 1 - d);
            let digit = (value / place).floor() as i32 % 10;
            let below = value % place;
            let roll = if ctx.eased < 1.0 && (d == drums - 1 || below >= place - 1.0) {
                turn
            } else {
                0.0
            };
            let shift = (roll * 12.0) as i32;
            // Current digit slides up; the next one follows from below.
            for (k, dv) in [(0, digit), (1, (digit + 1) % 10)] {
                let top = hi / 2 - 5 - shift + k * 12;
                for (r, bits) in DIGITS[dv as usize].iter().enumerate() {
                    for (c, b) in bits.bytes().enumerate() {
                        let y = top + r as i32 * 2;
                        if b == b'#' && y > 0 && y + 1 < hi - 1 {
                            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                                draw::dot_i(grid, x + 2 + c as i32 * 2 + ox, y + oy);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Tunnel headlights (perspective)
//    PERSPECTIVE-primary mechanic.  Road lines converge to a vanishing point
//    at the top-center.  As eased grows, the car (represented by two headlight
//    beams) advances: beams widen = car is closer to the viewer.
//    time drives scrolling dotted center-line segments.
//    At 100% the car exits toward the viewer (beams fill the full width).
// ─────────────────────────────────────────────────────────────────────────────

struct TunnelHeadlights;
impl ProgressStyle for TunnelHeadlights {
    fn name(&self) -> &str {
        "tunnel-headlights"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Perspective tunnel: converging road lines; headlight beams widen as the car approaches"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let vp_x = (dw / 2) as i32;
        let vp_y = 0_i32; // vanishing point at the very top center

        // Road edges: from vanishing point to wide bottom corners
        let road_half_bottom = (dw as i32 / 3).max(2);
        let bottom_y = (dh as i32) - 1;

        line(grid, vp_x, vp_y, vp_x - road_half_bottom, bottom_y);
        line(grid, vp_x, vp_y, vp_x + road_half_bottom, bottom_y);

        // Tunnel ceiling / walls: two more lines slightly tighter
        let tunnel_margin = (dw as i32 / 6).max(1);
        let ceiling_y = 0_i32;
        line(grid, vp_x - tunnel_margin, ceiling_y, 0, bottom_y);
        line(
            grid,
            vp_x + tunnel_margin,
            ceiling_y,
            dw as i32 - 1,
            bottom_y,
        );

        // Scrolling dotted center line
        let seg_count = 8usize;
        let scroll_phase = (ctx.time * 0.6).fract();
        for s in 0..seg_count {
            let t_base = (s as f32 + scroll_phase) / seg_count as f32;
            let t = t_base.fract();
            // y position between vp and bottom
            let seg_y = (vp_y as f32 + (dh as f32 - vp_y as f32) * t) as i32;
            if s % 2 == 0 {
                draw::dot_i(grid, vp_x, seg_y);
                draw::dot_i(grid, vp_x, seg_y + 1);
            }
        }

        // Headlights: the car is at depth = eased (0=far, 1=right at viewer)
        // Car y position on screen: at depth=0 it's near vp, at depth=1 it's at the bottom
        let car_depth = ctx.eased;
        let car_y = (vp_y as f32 + (dh as f32 - vp_y as f32) * car_depth) as i32;
        // Headlight spread: widens with depth (closer = wider beam)
        let beam_spread = (road_half_bottom as f32 * car_depth * 0.6).max(1.0) as i32;
        let beam_up = (dh as f32 * car_depth * 0.15) as i32;

        // Left headlight beam: from car position upward-left to vp area
        let lx = vp_x - beam_spread;
        let rx = vp_x + beam_spread;
        // Beam lines: car center → top
        if car_y > vp_y + 1 {
            line(
                grid,
                lx,
                car_y,
                vp_x - beam_spread / 3,
                (car_y - beam_up).max(vp_y + 1),
            );
            line(
                grid,
                rx,
                car_y,
                vp_x + beam_spread / 3,
                (car_y - beam_up).max(vp_y + 1),
            );
        }

        // Headlight dots (the lights themselves)
        for dy in -1_i32..=1 {
            draw::dot_i(grid, lx, car_y + dy);
            draw::dot_i(grid, rx, car_y + dy);
        }
        // Car width bar at car_y
        if lx < rx {
            draw::hline(
                grid,
                lx.max(0) as usize,
                rx.min(dw as i32 - 1) as usize,
                car_y as usize,
            );
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Burnout
//    The race car sits on the start line, rocking, its rear wheel spinning
//    flat out; billowing tyre-smoke puffs roll back and up from the rear tyre
//    and black rubber marks stretch behind it. The smoke cloud's reach = eased.
// ─────────────────────────────────────────────────────────────────────────────

struct Burnout;
impl ProgressStyle for Burnout {
    fn name(&self) -> &str {
        "burnout"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Race car on the start line spinning its rear wheel; billowing tyre smoke and rubber marks reach back further as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        let jitter = i32::from((t * 14.0).fract() < 0.5);
        let x = (wi - 38).max(10) + jitter;
        let y = hi - 10;
        race_car(grid, x, y, t * 40.0, [true, true]);
        let rear = x + WHEEL_DX[0];

        // Start line ahead of the nose.
        let line_x = x + 33;
        for ly in 0..hi {
            if ly % 2 == 0 {
                draw::dot_i(grid, line_x, ly);
                draw::dot_i(grid, line_x + 1, ly);
            }
        }

        // Rubber marks and smoke reach back with progress.
        let reach = 8.0 + ctx.eased * (rear as f32 - 4.0).max(0.0);
        let mark0 = rear - reach as i32;
        for mx in mark0.max(0)..rear {
            draw::dot_i(grid, mx, hi - 1);
        }
        for i in 0..9 {
            let s = (t * 0.6 + i as f32 / 9.0).fract();
            let cx = rear - 2 - (s * reach) as i32;
            let cy = hi - 3 - (s * 7.0) as i32;
            let r = 1.5 + s * 3.5;
            let steps = (r * 5.0) as i32 + 4;
            for k in 0..steps {
                let a = k as f32 / steps as f32 * 2.0 * PI + i as f32;
                draw::dot_i(
                    grid,
                    cx + (a.cos() * r).round() as i32,
                    cy + (a.sin() * r * 0.8).round() as i32,
                );
            }
        }

        // Colour: red car, white start line, grey smoke.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let px = cx as i32 * 2 + 1;
                let color = if px >= line_x - 1 {
                    Color::rgb(240, 240, 240)
                } else if px > x {
                    Color::rgb(232, 44, 44)
                } else {
                    Color::rgb(176, 180, 190)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Pistons
//    Engine cutaway: a row of cylinders (walls, head, spark plug) with
//    pistons on connecting rods driven by a rotating crankshaft; each
//    cylinder flashes its spark at top dead centre. Cylinders fire up left
//    to right as eased grows; the rest sit cold at the bottom of the stroke.
// ─────────────────────────────────────────────────────────────────────────────

struct Pistons;
impl ProgressStyle for Pistons {
    fn name(&self) -> &str {
        "pistons"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Engine cutaway: pistons pump on connecting rods from a spinning crankshaft with spark flashes; more cylinders fire as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        let n = (wi / 11).max(1);
        let pitch = (wi / n).max(1);
        let running = (1 + (ctx.eased * (n - 1) as f32).round() as i32).min(n);
        let yc = hi - 3; // crankshaft
        let speed = 7.0 + ctx.eased * 7.0;
        let wall_bot = yc - 4;
        draw::hline(grid, 0, w.saturating_sub(1), (yc.max(0)) as usize);
        let mut flash = [false; 16];

        for i in 0..n {
            let x0 = i * pitch + (pitch - 10) / 2;
            let cx = x0 + 5;
            // Cylinder walls, head and spark plug.
            for yy in 1..=wall_bot {
                draw::dot_i(grid, x0, yy);
                draw::dot_i(grid, x0 + 9, yy);
            }
            for xx in x0..=x0 + 9 {
                draw::dot_i(grid, xx, 1);
            }
            draw::dot_i(grid, cx - 1, 0);
            draw::dot_i(grid, cx, 0);

            let on = i < running;
            let a = if on {
                t * speed + i as f32 * PI * 0.5
            } else {
                PI * 0.5
            };
            let (pin_x, pin_y) = (
                cx + (a.cos() * 2.0).round() as i32,
                yc + (a.sin() * 2.0).round() as i32,
            );
            let wrist = pin_y - 6;
            // Piston crown + skirt, connecting rod, crank pin.
            block(grid, x0 + 1, wrist - 2, 8, 2);
            draw::dot_i(grid, x0 + 1, wrist);
            draw::dot_i(grid, x0 + 8, wrist);
            line(grid, cx, wrist, pin_x, pin_y);
            block(grid, pin_x - 1, pin_y - 1, 2, 2);
            // Spark flash at top dead centre.
            if on && a.sin() < -0.8 {
                sprite(grid, cx - 3, 2, &["#.#.#.", ".####.", "#.##.#"]);
                if let Some(f) = flash.get_mut(i as usize) {
                    *f = true;
                }
            }
        }

        // Colour: hot running cylinders, cold idle ones, yellow sparks.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let i = (px / pitch).min(n - 1);
                let color = if i >= running {
                    Color::rgb(100, 110, 130)
                } else if py < 5 && flash.get(i as usize).copied().unwrap_or(false) {
                    Color::rgb(255, 232, 80)
                } else if py >= yc - 1 {
                    Color::rgb(200, 204, 214)
                } else {
                    Color::rgb(255, 128, 56)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Nitro boost
//    The race car blasts left to right (position = eased) on a jet of nitro
//    flame from its exhaust — blue core, orange tips, flickering — while
//    speed lines and road dashes stream past.
// ─────────────────────────────────────────────────────────────────────────────

struct NitroBoost;
impl ProgressStyle for NitroBoost {
    fn name(&self) -> &str {
        "nitro-boost"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Race car rockets along on a flickering nitro flame jet from its exhaust, speed lines streaming past; its position is the progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        let x = 16 + (ctx.eased * (wi - 46).max(0) as f32) as i32;
        let y = hi - 10;
        race_car(grid, x, y, t * 30.0, [true, true]);

        // Flame jet from the exhaust behind the rear wing.
        let ey = y + 3;
        let len = 11.0 + (t * 23.0).sin() * 2.0 + (t * 37.0).sin() * 1.5;
        let len_i = len as i32;
        for k in 0..len_i {
            let f = k as f32 / len;
            let hh = ((1.0 - f) * 2.6).round() as i32;
            for dy in -hh..=hh {
                let seed = (k * 7 + dy * 13 + (t * 20.0) as i32 * 3).rem_euclid(5);
                if f < 0.4 || seed != 0 {
                    draw::dot_i(grid, x - 1 - k, ey + dy);
                }
            }
        }

        // Speed lines above the car, road dashes below, streaming left.
        for i in 0..5 {
            let sp = 60.0 + i as f32 * 9.0;
            let period = (wi + 20) as f32;
            let sx = wi - ((t * sp + i as f32 * 31.0) % period) as i32;
            let sy = 1 + i * 2;
            if sy >= y && sx + 8 >= x - len_i {
                continue;
            }
            for k in 0..(6 + i % 3 * 2) {
                draw::dot_i(grid, sx + k, sy);
            }
        }
        let off = (t * 50.0) as i32;
        for gx in 0..wi {
            if (gx + off).rem_euclid(8) < 4 {
                draw::dot_i(grid, gx, hi - 1);
            }
        }

        // Colour: blue core / orange flame, red car, white streaks.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let color = if (x..=x + 30).contains(&px) && py >= y - 1 {
                    Color::rgb(232, 44, 44)
                } else if px < x && px >= x - 1 - len_i && (py - ey).abs() <= 3 {
                    if px >= x - 5 {
                        Color::rgb(90, 170, 255)
                    } else {
                        Color::rgb(255, 150, 40)
                    }
                } else {
                    Color::rgb(210, 226, 255)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Checkered flag
//    A pole on the left with a checkered flag of 4×4-dot squares. The flag
//    unfurls rightward as eased grows and ripples in the wind with time: each
//    column is displaced by a travelling sine wave that grows toward the free
//    edge, so the hoist stays pinned to the pole.
// ─────────────────────────────────────────────────────────────────────────────

struct CheckeredFlag;

impl ProgressStyle for CheckeredFlag {
    fn name(&self) -> &str {
        "checkered-flag"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Checkered flag: a black-and-white chequered flag on a pole unfurls with progress and ripples in the wind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Pole with a ball finial.
        draw::vline(grid, 1, 1, h - 1);
        draw::dot_i(grid, 0, 0);
        draw::dot_i(grid, 1, 0);
        draw::dot_i(grid, 2, 0);

        // Flag: 3 rows of squares (12 dots) hanging from the top of the pole.
        let fh = (hi - 3).clamp(1, 12);
        let fw = (4.0 + ctx.eased * (wi - 7).max(0) as f32) as i32;
        for fx in 0..fw {
            let reach = fx as f32 / fw.max(1) as f32;
            let dy = ((fx as f32 * 0.3 - ctx.time * 7.0).sin() * 1.6 * reach).round() as i32;
            for fy in 0..fh {
                if (fx / 4 + fy / 4) % 2 == 0 {
                    draw::dot_i(grid, 3 + fx, 1 + fy + dy);
                }
            }
            // Outline the top and bottom edges so white squares still read.
            draw::dot_i(grid, 3 + fx, 1 + dy);
            draw::dot_i(grid, 3 + fx, fh + dy);
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
    let styles = progress::styles::cars::styles();
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
