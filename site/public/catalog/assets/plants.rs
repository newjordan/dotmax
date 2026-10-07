//! `plants` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O plants.rs && ./plants [style-name]
//! ```

const DEFAULT_STYLE: &str = "seedling";

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
    pub mod plants {
//! Plants / flora progress bars — living, growing, breathing.
//!
//! Ten structurally distinct styles, each driven by `ctx.eased` for growth
//! stage and `ctx.time` for sway / breeze animation. Every style uses a
//! completely different rendering algorithm: no two share the same structural
//! approach. Each paints its own natural colours (greens, earth, petals)
//! cell by cell rather than using the bar palette.
//!
//! | Name                | Mechanism                                              |
//! |---------------------|--------------------------------------------------------|
//! | `seedling`          | Seed tray sprouting: hooked shoot → two seed leaves    |
//! | `fern-fiddlehead`   | Logarithmic spiral unrolling (fiddlehead uncurling)    |
//! | `bamboo-shoot`      | Vertical segments shooting up one by one               |
//! | `ivy-trellis`       | Ivy with three-lobed leaves winding over a lattice     |
//! | `cactus-arms`       | Saguaros grow in turn, sprout arms and a crown flower  |
//! | `sunflower-seeds`   | Sunflowers open: petal ring round a phyllotaxis disc   |
//! | `mushroom-cap`      | Red spotted toadstools pop up one after another        |
//! | `succulent-rosette` | Potted echeveria: plump leaves fan outward in pairs    |
//! | `root-system`       | Soil cutaway: roots snake right, forking rootlets      |
//! | `bonsai`            | Potted S-trunk → branches → swelling cloud-pad foliage |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── registry ─────────────────────────────────────────────────────────────────

/// All styles in the `plants` theme.
///
/// Returns ten structurally distinct plant-growth progress bars, each
/// mapping `ctx.eased` to visible growth and `ctx.time` to living motion.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Seedling),
        Box::new(FernFiddlehead),
        Box::new(BambooShoot),
        Box::new(IvyTrellis),
        Box::new(CactusArms),
        Box::new(SunflowerSeeds),
        Box::new(MushroomCap),
        Box::new(Succulent),
        Box::new(RootSystem),
        Box::new(Bonsai),
    ]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Integer sine — keeps per-bar code terse.
#[inline]
fn isin(angle: f32, amplitude: f32) -> i32 {
    (angle.sin() * amplitude).round() as i32
}

/// Tint the cell containing dot `(x, y)`; out-of-range dots are ignored.
fn tint_at(grid: &mut BrailleGrid, x: i32, y: i32, color: Color) {
    if x >= 0 && y >= 0 {
        let (cx, cy) = (x as usize / 2, y as usize / 4);
        draw::tint_row(grid, cy, cx, cx, color);
    }
}

/// Leaf green ramp: `0.0` deep forest green → `1.0` fresh yellow-green.
fn leafy(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::rgb(
        (40.0 + 130.0 * t) as u8,
        (130.0 + 90.0 * t) as u8,
        (50.0 + 30.0 * t) as u8,
    )
}

/// Set a dot and tint its cell in one go.
fn paint(grid: &mut BrailleGrid, x: i32, y: i32, color: Color) {
    draw::dot_i(grid, x, y);
    tint_at(grid, x, y, color);
}

// ── 1. Seedling — a seed tray sprouting in turn ──────────────────────────────
//
// A row of seeds in a soil bed germinate left to right. Each one swells,
// pushes up a hooked shoot, straightens, and opens two rounded seed leaves
// (cotyledons) that sway in the breeze.

struct Seedling;
impl ProgressStyle for Seedling {
    fn name(&self) -> &str {
        "seedling"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Seeds in a soil bed sprout left to right: a hooked shoot straightens and opens two swaying seed leaves"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let soil = Color::rgb(120, 80, 45);
        let seed_c = Color::rgb(215, 180, 110);
        let stem_c = Color::rgb(150, 215, 90);
        let leaf_c = Color::rgb(70, 185, 70);
        let ground = h as i32 - 2;

        // Soil bed: a solid crust line with crumbs underneath.
        for x in 0..w as i32 {
            paint(grid, x, ground, soil);
            if x % 3 == 0 {
                paint(grid, x, ground + 1, soil);
            }
        }

        let n = (w / 14).max(1);
        let spacing = w as f32 / n as f32;
        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            let x = (spacing * (i as f32 + 0.5)) as i32;
            if local < 0.15 {
                // A dormant seed resting on the soil.
                paint(grid, x, ground - 1, seed_c);
                paint(grid, x + 1, ground - 1, seed_c);
                continue;
            }
            let sway = isin(ctx.time * 1.5 + i as f32, 1.0);
            let rise = ((local - 0.15) / 0.45).min(1.0);
            let height = (1.0 + (h as f32 * 0.4) * rise) as i32;
            let top = ground - height;
            for y in top..ground {
                let dx = if y < ground - height / 2 { sway } else { 0 };
                paint(grid, x + dx, y, stem_c);
            }
            if local < 0.6 {
                // Hooked shoot: the tip still bent over.
                paint(grid, x + sway + 1, top, stem_c);
                paint(grid, x + sway + 2, top + 1, stem_c);
                continue;
            }
            // Two seed leaves opening into rounded ovals.
            let open = ((local - 0.6) / 0.4).min(1.0);
            let rx = 1.0 + 1.2 * open;
            let ry = 0.7 + 0.5 * open;
            for side in [-1.0f32, 1.0] {
                let lcx = (x + sway) as f32 + side * (rx + 0.5);
                let lcy = top as f32 - open;
                for yy in (lcy - ry - 2.0) as i32..=(lcy + ry + 2.0) as i32 {
                    for xx in (lcx - rx - 1.0) as i32..=(lcx + rx + 1.0) as i32 {
                        let fx = (xx as f32 - lcx) / rx;
                        // Leaves tilt upward away from the stem.
                        let fy = (yy as f32 - lcy + fx * side * 1.6) / ry;
                        if fx * fx + fy * fy <= 1.0 {
                            paint(grid, xx, yy, leaf_c);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// ── 2. Fern Fiddlehead — logarithmic spiral unrolling ────────────────────────
//
// A fiddlehead starts as a tight coil (eased=0) and progressively unrolls
// into an open frond (eased=1). The spiral uses polar coordinates.
// ctx.time adds a gentle oscillation so the frond breathes.

struct FernFiddlehead;
impl ProgressStyle for FernFiddlehead {
    fn name(&self) -> &str {
        "fern-fiddlehead"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A fern fiddlehead unrolls from a tight coil into an open frond as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        // The frond unrolls left→right across the whole canvas: the rachis
        // (main stem) extends with eased, pinnae unfurl behind the advancing
        // tip, and the tip itself carries the uncurling fiddlehead coil.
        let x0 = 3.0f32;
        let y0 = hf - 2.0;
        let full_len = (wf - 12.0).max(2.0);
        let rise = hf * 0.45;

        // Breeze: a slow wave travels along the frond (0.25 Hz — seamless).
        let breeze = |s: f32| (ctx.time * PI * 0.5 + s * 3.0).sin() * 1.1 * s;

        // Rachis point at parameter s in 0..=1.
        let rachis = |s: f32| -> (f32, f32) {
            let x = x0 + s * full_len;
            let y = y0 - rise * s.powf(1.25) + breeze(s);
            (x, y)
        };

        // Draw the rachis up to the growth front, thickened near the base.
        let grown = ctx.eased.clamp(0.0, 1.0);
        let steps = (full_len * 1.5) as i32;
        for i in 0..=steps {
            let s = i as f32 / steps.max(1) as f32 * grown;
            if s > grown {
                break;
            }
            let (x, y) = rachis(s);
            draw::dot_i(grid, x.round() as i32, y.round() as i32);
            if s < 0.6 {
                draw::dot_i(grid, x.round() as i32, y.round() as i32 + 1);
            }
        }

        // Pinnae: paired leaflets sprouting along the grown rachis. Each pair
        // matures (lengthens) once the tip has passed it.
        let n_pinnae = 16i32;
        for k in 1..=n_pinnae {
            let sk = k as f32 / (n_pinnae + 1) as f32;
            if sk >= grown {
                break;
            }
            let maturity = ((grown - sk) / 0.12).clamp(0.0, 1.0);
            let len = (hf * 0.38) * (1.0 - 0.55 * sk) * maturity;
            let (px, py) = rachis(sk);
            for j in 1..=(len.round() as i32) {
                let jf = j as f32;
                // Upward leaflet sweeps back; downward one is shorter.
                draw::dot_i(
                    grid,
                    (px - jf * 0.45).round() as i32,
                    (py - jf * 0.9).round() as i32,
                );
                if jf < len * 0.7 {
                    draw::dot_i(
                        grid,
                        (px - jf * 0.35).round() as i32,
                        (py + jf * 0.8).round() as i32,
                    );
                }
            }
        }

        // Fiddlehead coil at the growth tip: a tight Archimedean curl that
        // unwinds as progress advances until only a small hook remains.
        let (tx, ty) = rachis(grown);
        let coil_turns = 0.4 + 2.4 * (1.0 - grown);
        let coil_r = 2.2 + 4.0 * (1.0 - grown);
        let coil_cx = tx + coil_r * 0.4;
        let coil_cy = ty - coil_r * 0.9;
        let coil_steps = (coil_turns * 24.0) as i32;
        for i in 0..=coil_steps {
            let t = i as f32 / coil_steps.max(1) as f32;
            let theta = t * coil_turns * 2.0 * PI + PI * 0.5;
            let r = coil_r * (1.0 - t * 0.85);
            draw::dot_i(
                grid,
                (coil_cx + r * theta.cos()).round() as i32,
                (coil_cy + r * theta.sin()).round() as i32,
            );
        }

        // Tint: deep green shifting to pale frond tip.
        let (cw, ch) = grid.dimensions();
        for row in 0..ch {
            let t = row as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = leafy(1.0 - t);
            draw::tint_row(grid, row, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 3. Bamboo Shoot — vertical segments shooting up ──────────────────────────
//
// Each segment is a cell-glyph column block. Segments appear one by one from
// the bottom as eased advances. Each joint has a horizontal line (node).
// ctx.time causes segments to sway slightly left-right as a column.

struct BambooShoot;
impl ProgressStyle for BambooShoot {
    fn name(&self) -> &str {
        "bamboo-shoot"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Bamboo culm grows segment by segment upward; joints appear at each node as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let (cw, ch) = grid.dimensions();
        let wi = w as i32;
        let hi = h as i32;
        let ground = hi - 1;

        // Dotted soil line the grove springs from.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, ground);
        }

        // A grove of culms revealed left→right: culm i shoots up while
        // `eased * n` passes through [i, i+1], each rising segment by segment.
        let n_culms = 6i32;
        let culm_max = (hi - 3).max(1) as f32;
        for i in 0..n_culms {
            let grow = (ctx.eased * n_culms as f32 - i as f32).clamp(0.0, 1.0);
            if grow <= 0.0 {
                break;
            }
            let cx = wi * (2 * i + 1) / (2 * n_culms);
            let culm_h = (grow * culm_max).round() as i32;
            // Sway: each culm leans on its own phase (0.25 Hz — seamless).
            let lean = (ctx.time * PI * 0.5 + i as f32 * 1.1).sin() * 1.6;

            for dy in 0..=culm_h {
                let xoff = (lean * dy as f32 / culm_max).round() as i32;
                let y = ground - 1 - dy;
                // Twin walls form the hollow culm.
                draw::dot_i(grid, cx - 1 + xoff, y);
                draw::dot_i(grid, cx + 1 + xoff, y);
                // Node (joint) line closes each segment.
                if dy % 4 == 3 {
                    draw::dot_i(grid, cx - 2 + xoff, y);
                    draw::dot_i(grid, cx + xoff, y);
                    draw::dot_i(grid, cx + 2 + xoff, y);
                }
            }
            // Tapered growing tip while the culm is still shooting.
            let tip_off = (lean * culm_h as f32 / culm_max).round() as i32;
            if grow < 1.0 {
                draw::dot_i(grid, cx + tip_off, ground - 2 - culm_h);
            }
            // Leaves at the upper nodes once tall enough, alternating sides.
            if culm_h >= 7 {
                let side = if i % 2 == 0 { 1i32 } else { -1 };
                for ld in 1..=4i32 {
                    draw::dot_i(
                        grid,
                        cx + tip_off + side * ld,
                        ground - 1 - culm_h + 4 - (ld + 1) / 2,
                    );
                    if culm_h >= 11 {
                        draw::dot_i(
                            grid,
                            cx + tip_off - side * ld,
                            ground - 1 - culm_h + 8 - (ld + 1) / 2,
                        );
                    }
                }
            }
        }

        // Gradient tint: deep green at base, yellow-green at tip.
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = leafy(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 4. Ivy Trellis — ivy climbing a diamond-lattice garden trellis ──────────
//
// A wooden lattice of crossed diagonal slats spans the bar. An ivy vine
// winds along it left→right with progress, sprouting three-lobed leaves on
// alternate sides; the leaves flutter between two poses with ctx.time.

struct IvyTrellis;
impl ProgressStyle for IvyTrellis {
    fn name(&self) -> &str {
        "ivy-trellis"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Ivy winds left to right across a wooden diamond-lattice trellis, sprouting fluttering three-lobed leaves"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let wood = Color::rgb(160, 115, 70);
        let vine = leafy(0.2);
        let hh = h as i32;

        // Lattice: two families of diagonal slats crossing into diamonds.
        for y in 0..hh {
            for x in 0..w as i32 {
                if (x + y).rem_euclid(10) == 0 || (x - y).rem_euclid(10) == 0 {
                    paint(grid, x, y, wood);
                }
            }
        }

        // Vine: a winding stem out to `reach`.
        let reach = (ctx.eased * w as f32) as i32;
        let mid = hh as f32 / 2.0;
        let stem_y = |x: i32| (mid + (x as f32 * 0.2).sin() * (hh as f32 * 0.22)) as i32;
        for x in 0..reach {
            paint(grid, x, stem_y(x), vine);
        }

        // Leaves on alternate sides; the newest ones near the tip are small.
        let mut i = 0;
        let mut x = 3;
        while x < reach {
            let up = i % 2 == 0;
            let sy = stem_y(x);
            let big = reach - x > 4;
            let flap = (ctx.time * 3.0 + i as f32 * 1.3).sin() > 0.0;
            let dir = if up { -1 } else { 1 };
            let c = leafy(if i % 3 == 0 { 0.5 } else { 0.25 });
            // Petiole from the stem to the leaf.
            paint(grid, x, sy + dir, c);
            let ly = sy + dir * 2;
            if big {
                // Three-lobed ivy leaf: a broad body with two side lobes.
                let lobe = if flap { 0 } else { dir };
                for dx in -2..=2 {
                    paint(grid, x + dx, ly, c);
                }
                for dx in -1..=1 {
                    paint(grid, x + dx, ly + dir, c);
                }
                paint(grid, x, ly + dir * 2, c);
                paint(grid, x - 2, ly - dir + lobe, c);
                paint(grid, x + 2, ly - dir + lobe, c);
            } else {
                paint(grid, x, ly, c);
                paint(grid, x + 1, ly, c);
            }
            i += 1;
            x += 5;
        }

        Ok(())
    }
}

// ── 5. Cactus Arms — saguaros grow, arms branch, a flower crowns them ───────
//
// Three saguaros on a sandy floor grow in turn. Each trunk rises with a
// rounded top, then its elbowed arms sprout at thresholds, and a pink flower
// opens on the crown once it is fully grown. A tumbleweed rolls past on time.

struct CactusArms;
impl ProgressStyle for CactusArms {
    fn name(&self) -> &str {
        "cactus-arms"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Three saguaro cacti rise in turn, sprout elbowed arms and a pink crown flower while a tumbleweed rolls by"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let green = Color::rgb(60, 170, 80);
        let sand = Color::rgb(210, 170, 100);
        let pink = Color::rgb(255, 110, 170);
        let weed = Color::rgb(150, 110, 60);
        let ground = h as i32 - 1;

        // Sandy floor: a dotted dune line.
        for x in 0..w as i32 {
            if x % 3 != 1 {
                paint(grid, x, ground, sand);
            }
        }

        // Tumbleweed: a little scribbled wheel rolling and bouncing (drawn
        // first so the cacti stay green when it rolls behind them).
        let span = w as f32 + 8.0;
        let tx = (ctx.time * 9.0).rem_euclid(span) - 4.0;
        let ty = ground as f32 - 2.0 - (ctx.time * 6.0).sin().abs() * 1.5;
        for k in 0..8 {
            let a = k as f32 / 8.0 * 2.0 * PI + ctx.time * 5.0;
            let rr = if k % 2 == 0 { 2.0 } else { 1.2 };
            paint(
                grid,
                (tx + a.cos() * rr * 1.2).round() as i32,
                (ty + a.sin() * rr).round() as i32,
                weed,
            );
        }

        let n = 3usize;
        let full_h = (h as f32 - 2.0).max(1.0);
        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let cx = (w as f32 * (0.18 + 0.32 * i as f32)) as i32;
            let height = (full_h * (local / 0.55).min(1.0)).max(1.0) as i32;
            let top = ground - height;
            // Trunk: 4 dots wide with a rounded crown and a rib groove.
            for y in top..ground {
                let crown = y == top;
                for dx in -2..=1 {
                    if crown && (dx == -2 || dx == 1) {
                        continue;
                    }
                    if dx == 0 && y > top + 1 && (y + i as i32) % 3 == 0 {
                        continue;
                    }
                    paint(grid, cx + dx, y, green);
                }
            }
            // Arms: out sideways, then up — left at 60%, right at 80%.
            let arms = [(0.6, -1, 0.55), (0.8, 1, 0.4)];
            for (k, &(at, side, frac)) in arms.iter().enumerate() {
                if local < at {
                    continue;
                }
                let grow = ((local - at) / 0.15).min(1.0);
                let elbow_y = ground - (full_h * frac) as i32;
                let reach = 3;
                let x_out = if side < 0 {
                    cx - 2 - reach
                } else {
                    cx + 1 + reach
                };
                for s in 1..=reach {
                    let x = if side < 0 { cx - 2 - s } else { cx + 1 + s };
                    paint(grid, x, elbow_y, green);
                    paint(grid, x, elbow_y - 1, green);
                }
                let up = ((3.0 + k as f32) * grow).round() as i32;
                for y in (elbow_y - 1 - up)..elbow_y {
                    paint(grid, x_out, y, green);
                    paint(grid, x_out - side, y, green);
                }
            }
            // Flower crown once fully grown.
            if local >= 0.98 {
                paint(grid, cx - 1, top - 1, pink);
                paint(grid, cx, top - 1, pink);
                paint(grid, cx - 2, top - 2, pink);
                paint(grid, cx + 1, top - 2, pink);
            }
        }

        Ok(())
    }
}

// ── 6. Sunflower Seeds — a field of sunflowers opening in turn ──────────────
//
// Each sunflower grows its stem, then opens: a ring of yellow petals spreads
// round a brown disc that fills with golden-angle (phyllotaxis) seeds.
// Flowers open left to right with progress; heads nod in the breeze.

struct SunflowerSeeds;
impl ProgressStyle for SunflowerSeeds {
    fn name(&self) -> &str {
        "sunflower-seeds"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A row of sunflowers opens left to right: yellow petals ring a brown disc packed with spiral seeds"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let stem = Color::rgb(70, 170, 60);
        let petal = Color::rgb(255, 205, 30);
        let seed = Color::rgb(130, 75, 30);
        let ground = h as i32 - 1;
        let n = (w / 15).max(1);
        let spacing = w as f32 / n as f32;
        let hy = h as f32 * 0.36;
        let r_out = (h as f32 * 0.34).min(spacing * 0.4).max(1.0);
        let r_disc = (r_out * 0.45).max(1.0);
        const GOLDEN: f32 = 2.399_963;

        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            let base_x = spacing * (i as f32 + 0.5);
            if local <= 0.0 {
                // Unsprouted: a seed in the soil.
                paint(grid, base_x as i32, ground, seed);
                continue;
            }
            let nod = (ctx.time * 1.3 + i as f32 * 1.1).sin() * 0.8;
            let hx = base_x + nod;
            let grow = (local * 2.0).min(1.0);
            let bloom = ((local - 0.5) * 2.0).clamp(0.0, 1.0);
            let top = ground as f32 - (ground as f32 - hy) * grow;
            for y in top as i32..=ground {
                paint(grid, base_x.round() as i32, y, stem);
            }
            // Two leaves on the stem.
            if grow > 0.6 {
                let ly = ground - 3;
                paint(grid, base_x as i32 - 1, ly, stem);
                paint(grid, base_x as i32 - 2, ly - 1, stem);
                paint(grid, base_x as i32 + 1, ly + 1, stem);
                paint(grid, base_x as i32 + 2, ly, stem);
            }
            if grow < 1.0 {
                continue;
            }
            // Petals: 12 rays that lengthen as the head opens, turning slowly.
            let len = r_disc + 0.5 + (r_out - r_disc) * (0.3 + 0.7 * bloom);
            for k in 0..12 {
                let a = k as f32 / 12.0 * 2.0 * PI + ctx.time * 0.2;
                for step in 0..=((len - r_disc) * 2.0) as i32 {
                    let d = r_disc + step as f32 * 0.5;
                    paint(
                        grid,
                        (hx + a.cos() * d * 1.25).round() as i32,
                        (hy + a.sin() * d).round() as i32,
                        petal,
                    );
                }
            }
            // Seed disc: phyllotaxis spiral fills in as the head matures.
            let seeds = 6 + (bloom * 40.0) as i32;
            for k in 0..seeds {
                let rr = r_disc * (k as f32 / 46.0).sqrt();
                let a = k as f32 * GOLDEN;
                paint(
                    grid,
                    (hx + a.cos() * rr * 1.25).round() as i32,
                    (hy + a.sin() * rr).round() as i32,
                    seed,
                );
            }
        }

        Ok(())
    }
}

// ── 7. Mushroom Cap — toadstools pop up one after another ────────────────────
//
// A row of fly-agaric toadstools: each pushes up a cream stalk, then its red
// domed cap swells, punched with white spots (negative space). Toadstools
// appear left to right with progress; the newest one breathes and puffs spores.

struct MushroomCap;
impl ProgressStyle for MushroomCap {
    fn name(&self) -> &str {
        "mushroom-cap"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Red spotted toadstools pop up left to right: a cream stalk rises, then the domed cap swells and puffs spores"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cap_c = Color::rgb(225, 45, 45);
        let stalk_c = Color::rgb(240, 225, 195);
        let spore_c = Color::rgb(200, 190, 160);
        let ground = h as i32 - 1;
        let n = (w / 18).max(1);
        let spacing = w as f32 / n as f32;
        let cap_base = (h as f32 * 0.7) as i32;

        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let cx = (spacing * (i as f32 + 0.5)) as i32;
            // Stalk rises in the first 30%, cap swells for the rest.
            let rise = (local / 0.3).min(1.0);
            let swell = ((local - 0.2) / 0.8).clamp(0.0, 1.0);
            let breathe = if local < 1.0 || i + 1 == n {
                (ctx.time * 3.0 + i as f32).sin() * 0.3
            } else {
                0.0
            };
            let base_y = ground - ((ground - cap_base) as f32 * rise) as i32;
            for y in base_y..=ground {
                for dx in -1..=1 {
                    paint(grid, cx + dx, y, stalk_c);
                }
            }
            // Skirt ring under the cap.
            if swell > 0.3 {
                paint(grid, cx - 2, base_y + 1, stalk_c);
                paint(grid, cx + 2, base_y + 1, stalk_c);
            }

            let rx = (1.5 + (spacing * 0.36 - 1.5).max(0.0) * swell + breathe).max(1.0);
            let ry = (1.0 + (h as f32 * 0.5 - 1.0) * swell + breathe).max(1.0);
            let cyf = base_y as f32;
            // White spots, as fractions of the cap radii.
            let spots = [(-0.5, -0.45), (0.15, -0.75), (0.55, -0.35), (-0.1, -0.25)];
            for y in (cyf - ry).floor() as i32..=base_y {
                for x in (cx as f32 - rx).floor() as i32..=(cx as f32 + rx).ceil() as i32 {
                    let fx = (x - cx) as f32 / rx;
                    let fy = (y as f32 - cyf) / ry;
                    if fx * fx + fy * fy > 1.0 {
                        continue;
                    }
                    let spot = swell > 0.5
                        && spots.iter().any(|&(sx, sy)| {
                            ((fx - sx) * rx).abs() < 1.0 && ((fy - sy) * ry).abs() < 1.0
                        });
                    if !spot {
                        paint(grid, x, y, cap_c);
                    }
                }
            }

            // Spores drift up off the newest cap.
            if i + 1 == ((ctx.eased * n as f32).ceil() as usize).min(n) && swell > 0.5 {
                for k in 0..3 {
                    let t = (ctx.time * 0.8 + k as f32 / 3.0).fract();
                    let sx = cx + isin(ctx.time * 2.0 + k as f32 * 2.0, 3.0) + (k - 1) * 3;
                    let sy = (cyf - ry - 1.0 - t * 4.0) as i32;
                    paint(grid, sx, sy, spore_c);
                }
            }
        }

        Ok(())
    }
}

// ── 8. Succulent Rosette — fleshy leaves fan open from a potted rosette ─────
//
// Side view of an echeveria in a terracotta pot. Upright inner leaves appear
// first; with progress the rosette fans outward, each new leaf longer and
// lower, until the outermost leaves spread almost flat. Pink-blushed tips,
// and the whole rosette breathes gently with time.

struct Succulent;
impl ProgressStyle for Succulent {
    fn name(&self) -> &str {
        "succulent-rosette"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A potted echeveria fans open: plump sage-green leaves with pink tips spread outward from the centre"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let pot = Color::rgb(190, 95, 60);
        let leaf_c = Color::rgb(120, 190, 150);
        let tip_c = Color::rgb(235, 120, 160);
        let cx = w as f32 / 2.0;
        let ground = h as i32 - 1;
        let base_y = (h as f32 - 4.0).max(0.0);

        // Pot: rim plus a tapering body.
        let half = (w as f32 * 0.12).max(2.0) as i32;
        for y in (ground - 2)..=ground {
            let inset = (y - ground + 2) * 2;
            for x in (cx as i32 - half + inset)..=(cx as i32 + half - inset) {
                paint(grid, x, y, pot);
            }
        }

        // Leaves pair up symmetrically: ring 0 is upright, ring 4 nearly flat.
        let rings = 5;
        let shown = ctx.eased * rings as f32;
        let breathe = (ctx.time * 1.8).sin() * 0.3;
        for ring in (0..rings).rev() {
            let grow = (shown - ring as f32).clamp(0.0, 1.0);
            if grow <= 0.0 {
                continue;
            }
            let tilt = 0.15 + ring as f32 * 0.32; // radians from vertical
            let len = (h as f32 * (0.55 + ring as f32 * 0.12) * grow + breathe).max(1.0);
            for side in [-1.0f32, 1.0] {
                let a = tilt * side;
                let (dx, dy) = (a.sin() * 2.2, -a.cos());
                for step in 0..=(len * 2.0) as i32 {
                    let s = step as f32 * 0.5;
                    let f = s / len;
                    // Plump leaf: widest a third of the way out, pointed tip.
                    let half_w = (1.1 * (1.0 - (f - 0.3).abs() * 1.3)).max(0.25);
                    let (px, py) = (cx + dx * s, base_y + dy * s);
                    let c = if f > 0.85 { tip_c } else { leaf_c };
                    for kk in 0..=(half_w * 4.0) as i32 {
                        let k = -half_w + kk as f32 * 0.5;
                        // Offset across the leaf, perpendicular to its axis.
                        paint(
                            grid,
                            (px - dy * k * 1.3).round() as i32,
                            (py + a.sin() * k).round() as i32,
                            c,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

// ── 9. Root System — a cutaway of roots spreading through the soil ──────────
//
// A sprout stands on the soil surface at the left. Below ground, in a
// speckled cutaway, its taproot dives down and a main root snakes right with
// progress, forking rootlets up and down whose fine tips wriggle with time.

struct RootSystem;
impl ProgressStyle for RootSystem {
    fn name(&self) -> &str {
        "root-system"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Soil cutaway under a sprout: cream roots snake right through speckled earth, forking rootlets with wriggling tips"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let soil = Color::rgb(120, 80, 45);
        let root = Color::rgb(235, 210, 160);
        let hh = h as i32;
        let surface = (hh / 4).max(1) - 1;
        let px = 6i32;

        // Soil: surface crust plus scattered speckles and pebbles.
        for x in 0..w as i32 {
            paint(grid, x, surface + 1, soil);
            for y in surface + 2..hh {
                if (x.wrapping_mul(7919) ^ y.wrapping_mul(104_729)).rem_euclid(23) == 0 {
                    paint(grid, x, y, soil);
                }
            }
        }

        // The sprout above ground: a stem with two leaves that sway.
        let sway = isin(ctx.time * 1.5, 1.0);
        for y in 0..=surface {
            paint(grid, px, y, leafy(0.6));
        }
        paint(grid, px - 1 + sway, 0, leafy(0.8));
        paint(grid, px - 2 + sway, 0, leafy(0.8));
        paint(grid, px + 1 + sway, 0, leafy(0.8));
        paint(grid, px + 2 + sway, 0, leafy(0.8));

        // Taproot down, then the main root snaking right out to `reach`.
        let depth = surface + 6;
        let reach = px + (ctx.eased * (w as f32 - px as f32 - 1.0)) as i32;
        for y in surface + 2..=depth {
            paint(grid, px, y, root);
            paint(grid, px + 1, y, root);
        }
        let main_y = |x: i32| depth + ((x as f32 * 0.18).sin() * 1.2) as i32;
        for x in px..=reach {
            paint(grid, x, main_y(x), root);
            // Thicker near the plant, thinning towards the tip.
            if x < px + (reach - px) / 2 {
                paint(grid, x, main_y(x) + 1, root);
            }
        }

        // Rootlets: fork downward every few dots, alternating steep and shallow.
        let mut k = 0;
        let mut x = px + 5;
        while x < reach - 1 {
            let steep = k % 2 == 0;
            let len = ((reach - x) / 3).clamp(1, 6);
            let y0 = main_y(x);
            let (mut rx, mut ry) = (x, y0);
            for s in 1..=len {
                rx = x + s;
                ry = if steep { y0 + s } else { y0 + (s + 1) / 2 };
                if ry >= hh {
                    break;
                }
                paint(grid, rx, ry, root);
            }
            // Fine root hair at the tip, wriggling.
            let wig = isin(ctx.time * 4.0 + x as f32, 1.0);
            paint(grid, rx + 1, ry + wig, root);
            k += 1;
            x += 6;
        }

        Ok(())
    }
}

// ── 10. Bonsai — potted tree: trunk, branches, cloud-pad canopy ──────────────
//
// A shallow terracotta pot holds a twisting trunk. Progress grows the trunk
// first, then reaches out the branches, then swells the flat cloud-like
// foliage pads at each branch tip. Pads drift slightly in the breeze.

struct Bonsai;
impl ProgressStyle for Bonsai {
    fn name(&self) -> &str {
        "bonsai"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A bonsai in a terracotta pot: the twisting trunk grows, branches reach out and cloud-pad foliage swells"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let pot = Color::rgb(190, 90, 55);
        let bark = Color::rgb(140, 95, 55);
        let leaf = Color::rgb(70, 175, 75);
        let leaf_hi = Color::rgb(140, 215, 90);
        let p = ctx.eased;
        let cx = (w / 2) as i32;
        let hh = h as i32;
        let ground = hh - 1;

        // Pot: a wide rim over a tapering body.
        let pot_top = ground - 2;
        let half = (w as i32 / 7).max(2);
        for y in pot_top..=ground {
            let inset = (y - pot_top) * 2;
            for x in (cx - half - 2 + inset)..=(cx + half + 2 - inset) {
                if y != pot_top || (x - cx).abs() <= half + 2 {
                    paint(grid, x, y, pot);
                }
            }
        }

        // Trunk: an S-curve from the soil up to the crown, 2 dots thick.
        let trunk_h = (hh - 7).max(1);
        let grown = ((p / 0.35).min(1.0) * trunk_h as f32) as i32;
        let trunk_x = |y: i32| -> i32 {
            let f = (pot_top - y) as f32 / trunk_h as f32;
            cx + ((f * PI * 1.4).sin() * (hh as f32 * 0.22)) as i32
        };
        for y in (pot_top - grown)..pot_top {
            let x = trunk_x(y);
            let thick = if y > pot_top - trunk_h / 2 { 3 } else { 2 };
            for k in 0..thick {
                paint(grid, x + k - 1, y, bark);
            }
        }
        if p < 0.3 {
            return Ok(());
        }

        // Branches and their pads: (height fraction, reach in dots, pad size).
        let unit = (w as f32 / 12.0).max(1.0);
        let branches = [
            (0.45, -2.6, 1.0),
            (0.62, 2.4, 0.9),
            (0.85, -1.2, 0.8),
            (1.0, 1.0, 1.1),
        ];
        let reach = ((p - 0.3) / 0.25).clamp(0.0, 1.0);
        let swell = ((p - 0.45) / 0.55).clamp(0.0, 1.0);
        for (k, &(fh, dir, size)) in branches.iter().enumerate() {
            let by = pot_top - (trunk_h as f32 * fh) as i32;
            let bx = trunk_x(by);
            let ex = bx + (dir * unit * reach) as i32;
            let ey = by - 1;
            let steps = (ex - bx).abs().max(1);
            for s in 0..=steps {
                let x = bx + (ex - bx) * s / steps;
                let y = by - s / 4;
                paint(grid, x, y, bark);
            }
            if swell <= 0.0 {
                continue;
            }
            // Cloud pad: a flat ellipse, wider than tall, drifting gently.
            let drift = (ctx.time * 1.2 + k as f32).sin() * 0.6;
            let rx = (unit * 1.3 * size * swell).max(1.0);
            let ry = (hh as f32 * 0.15 * size * swell).max(0.8);
            let pcx = ex as f32 + drift;
            let pcy = ey as f32 - ry * 0.4;
            for y in (pcy - ry).floor() as i32..=(pcy + ry).ceil() as i32 {
                for x in (pcx - rx).floor() as i32..=(pcx + rx).ceil() as i32 {
                    let fx = (x as f32 - pcx) / rx;
                    let fy = (y as f32 - pcy) / ry;
                    if fx * fx + fy * fy <= 1.0 {
                        let c = if fy < -0.3 { leaf_hi } else { leaf };
                        paint(grid, x, y, c);
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
    let styles = progress::styles::plants::styles();
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
