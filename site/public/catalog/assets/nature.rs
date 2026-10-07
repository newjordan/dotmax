//! `nature` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O nature.rs && ./nature [style-name]
//! ```

const DEFAULT_STYLE: &str = "growing-vine";

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
    pub mod nature {
//! Nature / weather / seasons progress bars.
//!
//! Ten animated styles built entirely from `draw::` helpers. Every bar reads
//! `ctx.eased` for its fill/growth amount and `ctx.time` for looping
//! animation, so they look alive even when progress is held constant.
//! Most styles paint their own natural colours (sky, sea, rock, snow, leaves)
//! cell by cell; the rain gauge still tints its water from the bar palette.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── registry ─────────────────────────────────────────────────────────────────

/// All styles in the `nature` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(GrowingVine),
        Box::new(GrassBlade),
        Box::new(Sunrise),
        Box::new(RainGauge),
        Box::new(TreeRings),
        Box::new(MountainSnow),
        Box::new(FourSeasons),
        Box::new(FlowerBloom),
        Box::new(FallingLeaves),
        Box::new(LightningBolt),
    ]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Integer sine rounded to i32 — keeps per-bar code terse.
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

/// Straight line of tinted dots from `a` to `b` (inclusive), `thick` dots wide.
fn line(grid: &mut BrailleGrid, a: (i32, i32), b: (i32, i32), thick: i32, color: Color) {
    let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
    for s in 0..=steps {
        let x = a.0 + (b.0 - a.0) * s / steps;
        let y = a.1 + (b.1 - a.1) * s / steps;
        for k in 0..thick.max(1) {
            paint(grid, x + k, y, color);
        }
    }
}

// ── 1. Growing vine ──────────────────────────────────────────────────────────

struct GrowingVine;
impl ProgressStyle for GrowingVine {
    fn name(&self) -> &str {
        "growing-vine"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A vine climbs left-to-right; leaf crosses sprout at every 10% milestone"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = (h / 2) as i32;
        let reach = (ctx.eased * w as f32).round() as usize;

        // Sine-swaying stem.
        for x in 0..reach {
            let sway = isin(x as f32 * 0.3 + ctx.time * 2.0, (h as f32 * 0.2).max(1.0));
            draw::dot_i(grid, x as i32, mid + sway);
        }

        // Leaf crosses at each completed 10% threshold.
        for step in 1..=10usize {
            let threshold = step as f32 * 0.1;
            if ctx.eased < threshold {
                break;
            }
            let lx = (threshold * w as f32).round() as i32;
            let sway = isin(lx as f32 * 0.3 + ctx.time * 2.0, (h as f32 * 0.2).max(1.0));
            let ly = mid + sway;
            // Small cross: horizontal + vertical arms of length 2.
            for dx in -2i32..=2 {
                draw::dot_i(grid, lx + dx, ly);
            }
            for dy in -2i32..=2 {
                draw::dot_i(grid, lx, ly + dy);
            }
        }

        // Tint the filled span green.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        let color = leafy(0.4);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, filled_cells.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 2. Grass blades ──────────────────────────────────────────────────────────

struct GrassBlade;
impl ProgressStyle for GrassBlade {
    fn name(&self) -> &str {
        "grass-blades"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Individual grass blades grow taller with progress and sway in the wind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let blade_spacing = 3usize;
        let base_y = h.saturating_sub(1);
        let max_growth = h.saturating_sub(1);

        let blade_count = w / blade_spacing;
        for b in 0..blade_count {
            let bx = b * blade_spacing + blade_spacing / 2;
            // Each blade grows to a slightly different height (pseudo-random via hash).
            let variety = ((b as f32 * 7.3).sin() * 0.5 + 0.5) * 0.4 + 0.6; // 0.6..1.0
            let height = (ctx.eased * max_growth as f32 * variety).round() as usize;
            if height == 0 {
                continue;
            }

            // Sway: low-frequency sine, each blade phase-shifted by position.
            let sway = isin(ctx.time * 1.8 + bx as f32 * 0.5, (h as f32 * 0.12).max(1.0));
            let tip_x = (bx as i32 + sway).max(0) as usize;

            let top_y = base_y.saturating_sub(height);
            // Stem.
            for y in top_y..=base_y {
                let frac = (base_y - y) as f32 / height.max(1) as f32;
                let sx = (bx as f32 + sway as f32 * frac).round() as usize;
                draw::dot(grid, sx.min(w.saturating_sub(1)), y);
            }
            // Tip dot — slightly to the side.
            draw::dot(grid, tip_x.min(w.saturating_sub(1)), top_y);
        }

        // Gradient tint.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if cw <= 1 {
                0.0
            } else {
                cx as f32 / (cw - 1) as f32
            };
            let color = leafy(0.2 + 0.7 * t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        // Not-yet-grown stubble stays a dry straw colour.
        for cy in 0..ch {
            draw::tint_row(
                grid,
                cy,
                filled_cells,
                cw.saturating_sub(1),
                Color::rgb(130, 120, 75),
            );
        }

        Ok(())
    }
}

// ── 3. Sunrise ───────────────────────────────────────────────────────────────

struct Sunrise;
impl ProgressStyle for Sunrise {
    fn name(&self) -> &str {
        "sunrise"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A rayed sun climbs out of the sea, red at dawn turning gold, its glitter shimmering on the water"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let p = ctx.eased;
        let horizon = ((h as f32 * 0.66) as i32).max(1);
        let r = (h as f32 * 0.3).min(w as f32 * 0.2).max(1.0);
        // Sun climbs from just below the horizon to high in the sky while
        // drifting right, so 20% and 90% sit in clearly different places.
        let sx = w as f32 * (0.12 + 0.5 * p);
        let sy = horizon as f32 + r * 0.6 - p * (horizon as f32 - r * 0.2);
        let sun = Color::rgb(255, (90.0 + 150.0 * p) as u8, (40.0 * p) as u8);
        let ray = Color::rgb(255, (60.0 + 120.0 * p) as u8, 20);
        let sea = Color::rgb(40, 80, 160);

        // Sea: dashed swell lines scrolling with time below the horizon.
        for y in horizon..h as i32 {
            let row = y - horizon;
            for x in 0..w as i32 {
                let phase =
                    (x + row * 5 + (ctx.time * 6.0) as i32 * if row % 2 == 0 { 1 } else { -1 })
                        .rem_euclid(9);
                if row == 0 || phase < 4 {
                    paint(grid, x, y, sea);
                }
            }
        }

        // Rays: 12 spokes that rotate slowly and pulse in length.
        let pulse = (ctx.time * 3.0).sin() * 0.5 + 0.5;
        for k in 0..12 {
            let a = k as f32 / 12.0 * 2.0 * PI + ctx.time * 0.25;
            let r0 = r + 1.5;
            let r1 = r + 2.5 + pulse * 2.0 * if k % 2 == 0 { 1.0 } else { 0.5 };
            for step in 0..=((r1 - r0) * 2.0) as i32 {
                let d = r0 + step as f32 * 0.5;
                let x = (sx + a.cos() * d * 1.2).round() as i32;
                let y = (sy + a.sin() * d).round() as i32;
                if y < horizon {
                    paint(grid, x, y, ray);
                }
            }
        }

        // Sun disc, clipped by the horizon.
        let (x0, x1) = ((sx - r * 1.2).floor() as i32, (sx + r * 1.2).ceil() as i32);
        let (y0, y1) = ((sy - r).floor() as i32, (sy + r).ceil() as i32);
        for y in y0..=y1.min(horizon - 1) {
            for x in x0..=x1 {
                let dx = (x as f32 - sx) / (r * 1.2);
                let dy = (y as f32 - sy) / r;
                if dx * dx + dy * dy <= 1.0 {
                    paint(grid, x, y, sun);
                }
            }
        }

        // Glitter path: flickering dashes on the water under the sun.
        for y in horizon + 1..h as i32 {
            let spread = (r * 1.4 * (1.0 - (y - horizon) as f32 * 0.12)).max(1.0) as i32;
            for x in (sx as i32 - spread)..=(sx as i32 + spread) {
                let n = (x * 7 + y * 13 + (ctx.time * 8.0) as i32 * 5).rem_euclid(5);
                if n < 2 {
                    paint(grid, x, y, ray);
                }
            }
        }

        Ok(())
    }
}

// ── 4. Rain gauge ────────────────────────────────────────────────────────────

struct RainGauge;
impl ProgressStyle for RainGauge {
    fn name(&self) -> &str {
        "rain-gauge"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Raindrops fall from a cloud; the water level rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Gauge outline.
        draw::rect_outline(grid, 0, 0, w, h);

        // Water fill from the bottom up.
        let fill_h = (ctx.eased * (h.saturating_sub(2)) as f32).round() as usize;
        let water_top = h.saturating_sub(1 + fill_h);
        if fill_h > 0 {
            draw::fill_rect(grid, 1, water_top, w.saturating_sub(2).max(1), fill_h);
        }

        // Animated raindrops — 5 drops cycling at different rates.
        let unfilled_h = water_top;
        if unfilled_h > 1 {
            for d in 0..5usize {
                let phase = d as f32 / 5.0;
                let drop_cycle = (ctx.time * 1.5 + phase).fract();
                let drop_y = (drop_cycle * unfilled_h as f32) as usize;
                let drop_x =
                    (((d as f32 * 1.618 + 0.5) % 1.0) * (w.saturating_sub(2)) as f32) as usize + 1;
                // Draw as a 1-dot pip.
                draw::dot(
                    grid,
                    drop_x.min(w.saturating_sub(2)),
                    drop_y.min(unfilled_h.saturating_sub(1)),
                );
            }
        }

        // Tint water blue via palette.
        let (cw, ch) = grid.dimensions();
        let water_top_cell = (water_top / 4).min(ch.saturating_sub(1));
        for cy in water_top_cell..ch {
            let t = (cy - water_top_cell) as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 5. Tree rings ────────────────────────────────────────────────────────────

struct TreeRings;
impl ProgressStyle for TreeRings {
    fn name(&self) -> &str {
        "tree-rings"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A sawn log grows longer with progress; its cut end shows growth rings that add up year by year as sawdust falls"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let bark = Color::rgb(120, 75, 40);
        let wood = Color::rgb(225, 180, 120);
        let ring = Color::rgb(170, 110, 60);
        let ry = (h as f32 / 2.0 - 0.5).max(0.5);
        let rx = ry.max(0.5);
        let cy = h as f32 / 2.0 - 0.5;
        // The cut face slides right as the log lengthens.
        let ex = rx + ctx.eased * (w as f32 - 2.0 * rx - 1.0).max(0.0);

        // Bark body: top and bottom edges plus wavy grain lines.
        for x in 0..ex as i32 {
            paint(grid, x, 0, bark);
            paint(grid, x, h as i32 - 1, bark);
            for k in 1..3 {
                if x as f32 >= ex - rx {
                    break;
                }
                let gy = (h as f32 * k as f32 / 3.0
                    + (x as f32 * 0.25 + k as f32 * 2.0).sin() * 1.2)
                    as i32;
                if (x + k * 5) % 9 < 6 {
                    paint(grid, x, gy, bark);
                }
            }
        }

        // End face: pale wood with concentric rings, more rings with progress.
        let years = 1 + (ctx.eased * 2.5) as i32;
        let (x0, x1) = ((ex - rx).floor() as i32, (ex + rx).ceil() as i32);
        for y in 0..h as i32 {
            for x in x0..=x1 {
                let fx = (x as f32 - ex) / rx;
                let fy = (y as f32 - cy) / ry;
                let d = (fx * fx + fy * fy).sqrt();
                if d > 1.0 {
                    continue;
                }
                let band = (d * (years as f32 + 0.5)).fract();
                if d > 0.88 {
                    paint(grid, x, y, bark);
                } else if band < 0.3 || d < 0.15 {
                    paint(grid, x, y, ring);
                } else {
                    tint_at(grid, x, y, wood);
                }
            }
        }

        // Sawdust sprinkling down from the fresh cut.
        for k in 0..4 {
            let t = (ctx.time * 1.3 + k as f32 * 0.25).fract();
            let sx = (ex + rx + 1.0 + k as f32 + t * 2.0) as i32;
            let sy = (cy + t * (h as f32 / 2.0)) as i32;
            paint(grid, sx, sy, wood);
        }

        Ok(())
    }
}

// ── 6. Mountain snow ─────────────────────────────────────────────────────────

struct MountainSnow;
impl ProgressStyle for MountainSnow {
    fn name(&self) -> &str {
        "mountain-snow"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A jagged grey mountain range under falling snowflakes; white snowcaps creep down the slopes with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let rock = Color::rgb(95, 105, 135);
        let snow = Color::rgb(240, 245, 255);
        let hf = h as f32;
        // Peaks: (x fraction, height fraction) — a jagged range.
        let peaks = [
            (0.1, 0.55),
            (0.27, 0.95),
            (0.42, 0.65),
            (0.58, 1.0),
            (0.74, 0.7),
            (0.9, 0.85),
        ];
        let ridge = |x: f32| -> f32 {
            let mut top = 0.0f32;
            for &(px, ph) in &peaks {
                let d = (x - px * w as f32).abs();
                top = top.max(ph * hf - d * 0.75);
            }
            // Small crags on the slopes.
            top + (x * 0.9).sin() * 0.6
        };
        // Snowline falls from the summits towards the valleys.
        let snow_depth = 1.0 + ctx.eased.powf(1.5) * hf * 0.65;

        for x in 0..w as i32 {
            let top = ridge(x as f32);
            let peak_y = hf - top;
            for y in peak_y.max(0.0).ceil() as i32..h as i32 {
                // Snow edge is ragged: it dips deeper on some columns.
                let ragged = ((x * 13) % 5) as f32 * 0.4;
                let snowy = (y as f32 - peak_y) < snow_depth - ragged && (y as f32) < hf - 1.0;
                if snowy {
                    paint(grid, x, y, snow);
                } else if (x + y) % 2 == 0 || y as f32 - peak_y < 1.0 {
                    // Rock: an outlined, hatched face.
                    paint(grid, x, y, rock);
                }
            }
        }

        // Snowflakes drifting down over the range.
        for i in 0..(w / 6).max(1) as i32 {
            let t = (ctx.time * 0.4 + i as f32 * 0.29).fract();
            let x = ((i * 23) % w as i32) as f32 + (ctx.time * 2.0 + i as f32).sin() * 2.0;
            let y = t * hf;
            let (xi, yi) = (x.round() as i32, y as i32);
            if yi as f32 + 1.0 < hf - ridge(x) {
                paint(grid, xi, yi, snow);
            }
        }

        Ok(())
    }
}

// ── 7. Four seasons ──────────────────────────────────────────────────────────

struct FourSeasons;
impl ProgressStyle for FourSeasons {
    fn name(&self) -> &str {
        "four-seasons"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "The year sweeps by in four hatched bands, each revealing its emblem: spring flower, summer sun, autumn leaf, winter snowflake"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let filled = (ctx.eased * w as f32).round() as usize;

        // Season bands: a light diagonal hatch, one quarter per season.
        let season_colors = [
            Color::rgb(60, 160, 60),   // spring
            Color::rgb(220, 170, 40),  // summer
            Color::rgb(180, 90, 45),   // autumn
            Color::rgb(110, 160, 210), // winter
        ];
        let quarter = (w as f32 / 4.0).max(1.0);
        for x in 0..filled as i32 {
            let k = ((x as f32 / quarter) as usize).min(3);
            for y in 0..h as i32 {
                if (x + y).rem_euclid(4) == 0 {
                    paint(grid, x, y, season_colors[k]);
                }
            }
        }

        // Each season's emblem appears once the fill has swept past it:
        // a flower, a rayed sun, a falling leaf and a snowflake.
        let cy = h as f32 / 2.0 - 0.5;
        for k in 0..4 {
            let cx = quarter * (k as f32 + 0.5);
            if (filled as f32) < cx + 4.0 {
                break;
            }
            let (x0, y0) = (cx.round() as i32, cy.round() as i32);
            match k {
                0 => {
                    // Flower: petal ring on a stem with a leaf.
                    let pink = Color::rgb(255, 120, 180);
                    let stem = leafy(0.5);
                    for y in y0..h as i32 {
                        paint(grid, x0, y, stem);
                    }
                    paint(grid, x0 + 1, y0 + 4, stem);
                    paint(grid, x0 + 2, y0 + 3, stem);
                    let sway = isin(ctx.time * 2.0, 1.0);
                    for j in 0..6 {
                        let a = j as f32 / 6.0 * 2.0 * PI;
                        paint(
                            grid,
                            x0 + sway + (a.cos() * 2.4).round() as i32,
                            y0 - 2 + (a.sin() * 2.0).round() as i32,
                            pink,
                        );
                    }
                    paint(grid, x0 + sway, y0 - 2, Color::rgb(255, 220, 60));
                }
                1 => {
                    // Sun: a disc with turning rays.
                    let sun = Color::rgb(255, 225, 60);
                    for dy in -2..=2i32 {
                        for dx in -3..=3i32 {
                            if dx * dx + 2 * dy * dy <= 9 {
                                paint(grid, x0 + dx, y0 + dy, sun);
                            }
                        }
                    }
                    for j in 0..8 {
                        let a = j as f32 / 8.0 * 2.0 * PI + ctx.time * 0.6;
                        for d in [4.5f32, 5.5] {
                            paint(
                                grid,
                                x0 + (a.cos() * d * 1.3).round() as i32,
                                y0 + (a.sin() * d).round() as i32,
                                sun,
                            );
                        }
                    }
                }
                2 => {
                    // Leaf: a pointed blade with a stem, bobbing as it falls.
                    let leaf = Color::rgb(240, 110, 30);
                    let bob = isin(ctx.time * 2.0, 1.5);
                    let (lx, ly) = (x0, y0 + bob);
                    for (dy, half) in [(-3, 0), (-2, 1), (-1, 2), (0, 3), (1, 2), (2, 1)] {
                        for dx in -half..=half {
                            paint(grid, lx + dx, ly + dy, leaf);
                        }
                    }
                    paint(grid, lx, ly + 3, leaf);
                    paint(grid, lx + 1, ly + 4, leaf);
                }
                _ => {
                    // Snowflake: six arms with little side branches, turning.
                    let snow = Color::rgb(240, 248, 255);
                    paint(grid, x0, y0, snow);
                    for j in 0..6 {
                        let a = j as f32 / 6.0 * PI * 2.0 + ctx.time * 0.5;
                        for d in 1..=5 {
                            let d = d as f32 * 0.9;
                            paint(
                                grid,
                                x0 + (a.cos() * d * 1.2).round() as i32,
                                y0 + (a.sin() * d).round() as i32,
                                snow,
                            );
                        }
                        let (bx, by) = (x0 as f32 + a.cos() * 3.0 * 1.2, y0 as f32 + a.sin() * 3.0);
                        for s in [-1.0f32, 1.0] {
                            let b = a + s * 0.9;
                            paint(
                                grid,
                                (bx + b.cos() * 1.5 * 1.2).round() as i32,
                                (by + b.sin() * 1.5).round() as i32,
                                snow,
                            );
                        }
                    }
                }
            }
        }

        // Track outline.
        draw::hline(grid, 0, w.saturating_sub(1), 0);
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        Ok(())
    }
}

// ── 8. Flower bloom ──────────────────────────────────────────────────────────

struct FlowerBloom;
impl ProgressStyle for FlowerBloom {
    fn name(&self) -> &str {
        "flower-bloom"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A row of tulips sprouts left to right; each bud swells and opens into a three-petal cup, swaying"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        const PETALS: [Color; 4] = [
            Color::rgb(235, 50, 70),
            Color::rgb(255, 120, 190),
            Color::rgb(255, 200, 40),
            Color::rgb(170, 90, 230),
        ];
        let stem = Color::rgb(70, 180, 70);
        let ground = h as i32 - 1;
        let n = (w / 12).max(1);
        let spacing = w as f32 / n as f32;
        let cup_h = (h as f32 * 0.4).max(2.0);

        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let base_x = (spacing * (i as f32 + 0.5)) as i32;
            let sway = isin(ctx.time * 1.6 + i as f32 * 0.9, 1.0);
            // Stem grows to full height in the first half, then the bud opens.
            let grow = (local * 2.0).min(1.0);
            let open = ((local - 0.5) * 2.0).clamp(0.0, 1.0);
            let top = ground - ((h as f32 - cup_h - 1.0).max(1.0) * grow) as i32;
            for y in top..=ground {
                let x = base_x + if y < ground - 3 { sway } else { 0 };
                paint(grid, x, y, stem);
            }
            // A long leaf curling off the stem near the ground.
            if grow > 0.5 {
                let side = if i % 2 == 0 { 1 } else { -1 };
                for k in 1..=3 {
                    paint(grid, base_x + side * k, ground - k + 1, stem);
                }
            }
            if grow < 1.0 {
                continue;
            }

            // Cup: straight-sided top flaring as it opens, rounded bottom.
            let cx = base_x + sway;
            let cup_top = top - cup_h as i32;
            let rx = 1.5 + 1.5 * open + local;
            let rx_top = rx * (0.2 + 1.0 * open);
            let color = PETALS[i % PETALS.len()];
            let rows = cup_h as i32;
            for dy in 0..=rows {
                let f = dy as f32 / rows as f32;
                let hw = if f < 0.6 {
                    rx_top + (rx - rx_top) * f / 0.6
                } else {
                    rx * (1.0 - ((f - 0.6) / 0.4).powi(2)).max(0.0).sqrt()
                };
                let hwi = hw.round() as i32;
                for dx in -hwi..=hwi {
                    // Notches between the three petal tips once open.
                    let notch = open > 0.3
                        && dy < 2
                        && dx.abs() as f32 > hw * 0.25
                        && (dx.abs() as f32) < hw * 0.65;
                    if !notch {
                        paint(grid, cx + dx, cup_top + dy, color);
                    }
                }
            }
        }

        Ok(())
    }
}

// ── 9. Falling leaves ────────────────────────────────────────────────────────

struct FallingLeaves;
impl ProgressStyle for FallingLeaves {
    fn name(&self) -> &str {
        "falling-leaves"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Red, orange and gold leaves tumble down swaying and pile up into a heap that spreads with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        const AUTUMN: [Color; 4] = [
            Color::rgb(220, 60, 40),
            Color::rgb(245, 140, 30),
            Color::rgb(240, 200, 50),
            Color::rgb(170, 90, 40),
        ];
        let ground = h as i32 - 1;
        let reach = (ctx.eased * w as f32) as i32;

        // Leaf pile: a lumpy heap out to `reach`, tapering at the front edge.
        for x in 0..reach {
            let lump = ((x as f32 * 0.7).sin() + (x as f32 * 0.23).sin()) * 0.6;
            let taper = ((reach - x) as f32 / 4.0).min(1.0);
            let height = ((2.5 + lump) * taper).max(1.0) as i32;
            let c = AUTUMN[((x / 2) as usize * 7 + 3) % AUTUMN.len()];
            for y in (ground - height + 1)..=ground {
                if (x + y) % 4 != 0 {
                    paint(grid, x, y, c);
                }
            }
        }

        // Falling leaves: each drifts down on its own lane, rocking side to side.
        let count = 6 + (w / 12) as i32;
        for i in 0..count {
            let speed = 0.35 + (i % 3) as f32 * 0.12;
            let t = (ctx.time * speed + i as f32 * 0.37).fract();
            let lane = ((i * 37) % w.max(1) as i32) as f32;
            let x = lane + (ctx.time * 2.5 + i as f32).sin() * 3.0 + t * 6.0;
            let y = t * (ground - 2) as f32;
            let (lx, ly) = (x.round() as i32, y.round() as i32);
            let c = AUTUMN[i as usize % AUTUMN.len()];
            // A tiny leaf that flips between two poses as it tumbles.
            if (ctx.time * 3.0 + i as f32).sin() > 0.0 {
                for &(dx, dy) in &[(0, 0), (1, 0), (2, 0), (1, -1), (2, -1), (0, 1)] {
                    paint(grid, lx + dx, ly + dy, c);
                }
            } else {
                for &(dx, dy) in &[(0, 0), (1, 0), (1, 1), (0, -1), (1, -1), (2, 1)] {
                    paint(grid, lx + dx, ly + dy, c);
                }
            }
        }

        Ok(())
    }
}

// ── 10. Lightning bolt ───────────────────────────────────────────────────────

struct LightningBolt;
impl ProgressStyle for LightningBolt {
    fn name(&self) -> &str {
        "lightning-bolt"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A storm cloud rolls in with progress; zigzag lightning bolts strike the ground from its leading edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cloud = Color::rgb(150, 150, 170);
        let bolt = Color::rgb(255, 235, 90);
        let reach = (ctx.eased * w as f32).max(6.0);
        let base = (h as f32 * 0.36).max(1.0);

        // Cloud bank: a row of overlapping puffs out to `reach`, billowing.
        let puffs = ((reach / 7.0) as i32).max(1);
        for i in 0..puffs {
            let cx = 3.0 + i as f32 * (reach - 3.0) / puffs as f32;
            let r = base
                * (0.55
                    + 0.3 * (i as f32 * 2.3).sin().abs()
                    + 0.08 * (ctx.time * 1.5 + i as f32).sin());
            let (x0, x1) = ((cx - r * 1.6) as i32, (cx + r * 1.6) as i32);
            for y in 0..=base as i32 {
                for x in x0..=x1.min(reach as i32) {
                    let dx = (x as f32 - cx) / (r * 1.6);
                    let dy = (y as f32 - base * 0.75) / r;
                    if dx * dx + dy * dy <= 1.0 {
                        paint(grid, x, y, cloud);
                    }
                }
            }
        }

        // Bolts flash on and off; each strike gets a fresh jagged shape.
        let beat = ctx.time * 2.2;
        let strike = beat.floor() as i32;
        if beat.fract() < 0.45 {
            for b in 0..2 {
                let seed = (strike * 31 + b * 17).rem_euclid(97);
                let x0 = if b == 0 {
                    reach - 6.0
                } else {
                    (seed as f32 / 97.0) * (reach - 6.0)
                } as i32;
                if b == 1 && (reach < 30.0 || seed % 3 == 0) {
                    continue;
                }
                let top = base as i32 + 1;
                let bottom = h as i32 - 1;
                let span = (bottom - top).max(1);
                let jog = 2 + seed % 3;
                // Classic bolt: slant right, kick back left, slant right again.
                let p1 = (x0 + jog + 1, top + span * 2 / 5);
                let p2 = (x0 - 1, top + span * 3 / 5);
                let p3 = (x0 + jog, bottom);
                line(grid, (x0, top), p1, 2, bolt);
                line(grid, p1, p2, 1, bolt);
                line(grid, p2, p3, 2, bolt);
                // Impact sparks.
                if beat.fract() < 0.2 {
                    paint(grid, p3.0 - 2, bottom, bolt);
                    paint(grid, p3.0 + 3, bottom, bolt);
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
    let styles = progress::styles::nature::styles();
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
