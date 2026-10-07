//! `chemistry` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O chemistry.rs && ./chemistry [style-name]
//! ```

const DEFAULT_STYLE: &str = "periodic-table";

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
    pub mod chemistry {
//! Chemistry-themed progress bars for dotmax.
//!
//! Each bar visualises a distinct chemical phenomenon using real physical
//! behaviour — no style differs merely by palette. Braille dots represent
//! particle clouds and kinetic curves; block glyphs represent periodic table
//! cells, pH gradients, and histogram columns.
//!
//! ## Styles (11 total)
//!
//! | Name | Chemical concept |
//! |------|-----------------|
//! | `periodic-table` | Real table silhouette; elements light up in Z order |
//! | `electron-orbitals` | s sphere, p dumbbell, d cloverleaf clouds fill in turn |
//! | `titration-curve` | Sigmoid pH–volume curve draws out; drops fall via time |
//! | `reaction-kinetics` | [A] exponential decay + [B] sigmoid rise to equilibrium |
//! | `crystallization` | Two-ion checkerboard lattice grows from a seed with facets |
//! | `ph-scale` | Indicator dot slides along a shaded acid→base track |
//! | `gas-diffusion` | Particles random-walk and spread; spread = eased |
//! | `distillation` | Boiling flask → condenser → Erlenmeyer receiver fills |
//! | `bond-vibration` | Two atoms vibrate on a coiled-spring bond that stretches |
//! | `boltzmann-distribution` | Velocity histogram fills to Maxwell-Boltzmann shape |
//! | `flame-spectral-lines` | Discrete emission lines appear as progress advances |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `chemistry` theme.
///
/// Returns 11 distinct bars, each encoding a real chemical process.
/// Every bar is safe to render from a 1×1 cell grid up to 80×8 or larger.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PeriodicTable),
        Box::new(ElectronOrbitals),
        Box::new(TitrationCurve),
        Box::new(ReactionKinetics),
        Box::new(Crystallization),
        Box::new(PhScale),
        Box::new(GasDiffusion),
        Box::new(Distillation),
        Box::new(BondVibration),
        Box::new(BoltzmannDistribution),
        Box::new(FlameSpectralLines),
    ]
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Cheap deterministic hash for seeding pseudo-randomness without rand crate.
#[inline]
fn hash2(a: usize, b: usize) -> u32 {
    let mut x = (a as u32)
        .wrapping_mul(2654435761)
        .wrapping_add(b as u32)
        .wrapping_mul(2246822519);
    x ^= x >> 15;
    x = x.wrapping_mul(2246822519);
    x ^= x >> 13;
    x
}

/// Straight line of dots between two signed dot coordinates.
fn line_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for i in 0..=steps {
        draw::dot_i(grid, x0 + (x1 - x0) * i / steps, y0 + (y1 - y0) * i / steps);
    }
}

/// Filled disc of dots centred at `(cx, cy)` with radius `r` (dot units).
fn disc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let ri = r.ceil() as i32 + 1;
    let (x0, y0) = (cx.round() as i32, cy.round() as i32);
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let fx = (x0 + dx) as f32 - cx;
            let fy = (y0 + dy) as f32 - cy;
            if fx * fx + fy * fy <= r * r + 0.5 {
                draw::dot_i(grid, x0 + dx, y0 + dy);
            }
        }
    }
}

/// Tint every cell whose centre lies within `r` dots of `(cx, cy)`.
fn tint_disc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, color: crate::Color) {
    let (cw, ch) = grid.dimensions();
    for cy2 in 0..ch {
        for cx2 in 0..cw {
            let dx = cx2 as f32 * 2.0 + 1.0 - cx;
            let dy = cy2 as f32 * 4.0 + 2.0 - cy;
            if dx * dx + dy * dy <= (r + 1.5) * (r + 1.5) {
                draw::tint_row(grid, cy2, cx2, cx2, color);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 1. Periodic table
//    The real table's silhouette: 18 groups × 7 periods (tall s-block on the
//    left, the notch above the d-block, the p-block on the right, H and He at
//    the far ends of the top row) plus the detached f-block strip below.
//    Elements are claimed in atomic-number order — eased × 118 of them are
//    solid, the rest are placeholder dots — and the next element blinks.
//    Columns are coloured by block (s, d, p).
// ---------------------------------------------------------------------------
struct PeriodicTable;

/// `(group 0..18, row 0..16)` of every element in atomic-number order.
/// Periods sit on even rows 0–12; the lanthanide / actinide strip on 14 / 15
/// is laid under groups 3–16.
fn periodic_layout() -> Vec<(usize, usize)> {
    let mut v = Vec::with_capacity(118);
    v.push((0, 0));
    v.push((17, 0));
    for p in 1..3 {
        v.extend([(0, p * 2), (1, p * 2)]);
        v.extend((12..18).map(|g| (g, p * 2)));
    }
    for p in 3..5 {
        v.extend((0..18).map(|g| (g, p * 2)));
    }
    for (p, f_row) in [(5, 14), (6, 15)] {
        v.extend([(0, p * 2), (1, p * 2)]);
        v.extend((2..16).map(|g| (g, f_row)));
        v.extend((2..18).map(|g| (g, p * 2)));
    }
    v
}

impl ProgressStyle for PeriodicTable {
    fn name(&self) -> &str {
        "periodic-table"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Periodic table: the real 18-group silhouette with its f-block strip; elements light up in atomic-number order"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let pitch = (w / 18).max(1);
        let ew = pitch.saturating_sub(1).max(1);
        let x0 = w.saturating_sub(18 * pitch) / 2;
        let layout = periodic_layout();
        let lit = (ctx.eased * layout.len() as f32).round() as usize;
        let blink = (ctx.time * 4.0).fract() < 0.5;

        for (z, &(g, row)) in layout.iter().enumerate() {
            let x = x0 + g * pitch;
            let y = row * h / 16;
            let solid = z < lit || (z == lit && blink);
            if solid {
                draw::hline(grid, x, x + ew - 1, y);
            } else {
                draw::dot(grid, x + ew / 2, y);
            }
            // Colour by block: s (groups 1–2), d (3–12), p (13–18).
            let t = match g {
                0 | 1 => 0.0,
                12..=17 => 1.0,
                _ => 0.5,
            };
            let t = if row >= 14 { 0.75 } else { t };
            if solid {
                let cy = y / 4;
                draw::tint_row(grid, cy, x / 2, (x + ew - 1) / 2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Electron orbitals
//    Three textbook orbital shapes side by side, each around its own nucleus:
//    s (a sphere), p (a dumbbell of two lobes along the axis) and d (a
//    four-lobed cloverleaf). Each shape's outline is always traced;
//    eased fills the subshells in order s → p → d as shimmering electron
//    clouds whose dot pattern re-rolls every frame with time.
// ---------------------------------------------------------------------------
struct ElectronOrbitals;

/// Lobes `(cx, cy, rx, ry)` of orbital `kind` (0 = s, 1 = p, 2 = d), relative
/// to the nucleus, for an orbital of radius `r` (dots).
fn orbital_lobes(kind: usize, r: f32) -> Vec<(f32, f32, f32, f32)> {
    match kind {
        0 => vec![(0.0, 0.0, r, r)],
        1 => vec![
            (-r * 0.62, 0.0, r * 0.6, r * 0.62),
            (r * 0.62, 0.0, r * 0.6, r * 0.62),
        ],
        _ => {
            let (ox, oy) = (r * 0.62, r * 0.55);
            let (lx, ly) = (r * 0.46, r * 0.42);
            vec![
                (-ox, -oy, lx, ly),
                (ox, -oy, lx, ly),
                (-ox, oy, lx, ly),
                (ox, oy, lx, ly),
            ]
        }
    }
}

impl ProgressStyle for ElectronOrbitals {
    fn name(&self) -> &str {
        "electron-orbitals"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Electron orbitals: s-sphere, p-dumbbell and d-cloverleaf clouds fill with shimmering electrons in turn"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 6 || h < 4 {
            return Ok(());
        }
        let (fw, fh) = (w as f32, h as f32);
        let cy = (fh - 1.0) / 2.0;
        let frame = (ctx.time * 12.0) as usize;

        for kind in 0..3 {
            let cx = fw * (kind as f32 * 2.0 + 1.0) / 6.0;
            let r = (fh / 2.0 - 0.5).min(fw / 6.0 - 1.0).max(1.0);
            // Horizontal stretch: lobes may spread wider than the bar is tall.
            let k = (fw / 6.0 - 2.0).max(r) / r;
            let lobes: Vec<(f32, f32, f32, f32)> = orbital_lobes(kind, r)
                .into_iter()
                .map(|(lx, ly, rx, ry)| (lx * k, ly, rx * k.min(1.6), ry))
                .collect();
            // This subshell's share of progress.
            let fill = (ctx.eased * 3.0 - kind as f32).clamp(0.0, 1.0);
            let color = ctx.palette.sample(kind as f32 / 2.0);

            let x0 = (cx - fw / 6.0).max(0.0) as i32;
            let x1 = (cx + fw / 6.0) as i32;
            for y in 0..h as i32 {
                for x in x0..=x1 {
                    let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                    // Normalised distance to the nearest lobe centre.
                    let d = lobes
                        .iter()
                        .map(|&(lx, ly, rx, ry)| {
                            let (u, v) = ((dx - lx) / rx.max(0.5), (dy - ly) / ry.max(0.5));
                            (u * u + v * v).sqrt()
                        })
                        .fold(f32::MAX, f32::min);
                    if d > 1.0 {
                        continue;
                    }
                    let edge = d > 0.8;
                    let roll = (hash2(x as usize * 31 + y as usize, frame) % 1000) as f32 / 1000.0;
                    let dense = fill * (1.05 - d * 0.6);
                    // Solid outline so the shape reads before it fills.
                    let lit = edge || roll < dense;
                    if lit {
                        draw::dot_i(grid, x, y);
                        if fill > 0.0 {
                            let c = (x / 2) as usize;
                            draw::tint_row(grid, y as usize / 4, c, c, color);
                        }
                    }
                }
            }
            // Nucleus.
            draw::fill_rect(grid, cx as usize, cy as usize, 2, 2);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Titration curve
//    Sigmoid pH-vs-volume curve draws out left-to-right as eased advances.
//    ctx.time drives "drip" marks that fall down at regular intervals —
//    each drip is a single dot column descending from y=0.
//    Equivalence point is marked with a tick at pH 7.
// ---------------------------------------------------------------------------
struct TitrationCurve;
impl ProgressStyle for TitrationCurve {
    fn name(&self) -> &str {
        "titration-curve"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Titration: a sigmoid pH–volume curve draws out while drips fall and the equivalence point appears"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // --- Draw the sigmoid pH curve ---
        // Map progress [0,1] to volume axis [0, w).
        // pH(v) = 7 + 4·tanh(10·(v - 0.5))  — steep at equivalence
        let drawn_x = (ctx.eased * w as f32).round() as usize;

        let mut prev_y: Option<i32> = None;
        for xi in 0..drawn_x.min(w) {
            let v = xi as f32 / w as f32; // volume fraction [0,1]
            let ph = 7.0 + 4.0 * (10.0 * (v - 0.5)).tanh();
            // Map pH [3, 11] to dot row [h-1, 0]
            let norm = (ph - 3.0) / 8.0; // [0,1] low→high pH
            let row = ((1.0 - norm) * (h - 1) as f32).round() as i32;
            let row = row.clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, row);
            if let Some(py) = prev_y {
                let lo = py.min(row);
                let hi = py.max(row);
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(row);
        }

        // --- Equivalence point tick at pH 7 (v = 0.5) ---
        let eq_x = (w / 2) as i32;
        let eq_ph = 7.0_f32;
        let eq_norm = (eq_ph - 3.0) / 8.0;
        let eq_y = ((1.0 - eq_norm) * (h - 1) as f32).round() as i32;
        // Vertical tick through equivalence point
        let tick = (h / 6).max(1) as i32;
        for yy in (eq_y - tick).max(0)..=(eq_y + tick).min(h as i32 - 1) {
            draw::dot_i(grid, eq_x, yy);
        }

        // --- Drips: falling dots driven by time ---
        // One drip per 0.4 s interval; drop falls at speed 4 dots/s
        let drip_interval = 0.4_f32;
        let drip_speed = 6.0_f32; // dots per second
                                  // Only drip left of the drawn curve
        let n_drips = 5usize;
        for di in 0..n_drips {
            let t_offset = di as f32 * drip_interval;
            let t_local = (ctx.time - t_offset).rem_euclid(drip_interval * n_drips as f32);
            if t_local < 0.0 {
                continue;
            }
            let drop_y = (t_local * drip_speed) as i32;
            // x position: evenly spaced across drawn region
            let drip_x = if drawn_x > 0 {
                (di * drawn_x / n_drips.max(1)) as i32
            } else {
                0
            };
            if drop_y < h as i32 {
                draw::dot_i(grid, drip_x, drop_y);
            }
        }

        // Tint by eased
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Reaction kinetics
//    [A] = A₀·e^(−k·eased·3) — reactant decays (drawn as descending curve).
//    [B] = 1 − [A]           — product rises (drawn as ascending curve).
//    Two distinct dot-curves fill simultaneously; below A-curve is shaded
//    lightly; below B-curve filled solidly.
// ---------------------------------------------------------------------------
struct ReactionKinetics;
impl ProgressStyle for ReactionKinetics {
    fn name(&self) -> &str {
        "reaction-kinetics"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Reaction kinetics: [A] decays exponentially while [B] rises to equilibrium — both curves fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // How far along the time axis we've drawn (eased = 0..1 maps to full timeline)
        let drawn_x = (ctx.eased * w as f32).round() as usize;
        let k = 3.0_f32; // rate constant (visual scale)

        let mut prev_a: Option<i32> = None;
        let mut prev_b: Option<i32> = None;

        for xi in 0..drawn_x.min(w) {
            let t_norm = xi as f32 / w as f32; // 0..1
            let a = (-k * t_norm).exp(); // [A] in [1, 0]
            let b = 1.0 - a; // [B] in [0, 1]

            // A-curve: high concentration = near top (row 0), low = near bottom
            let row_a = ((1.0 - a) * (h - 1) as f32).round() as i32;
            let row_a = row_a.clamp(0, h as i32 - 1);
            // B-curve: low → high, inverse mapping
            let row_b = ((1.0 - b) * (h - 1) as f32).round() as i32;
            let row_b = row_b.clamp(0, h as i32 - 1);

            draw::dot_i(grid, xi as i32, row_a);
            draw::dot_i(grid, xi as i32, row_b);

            if let Some(pa) = prev_a {
                let (lo, hi) = (pa.min(row_a), pa.max(row_a));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            if let Some(pb) = prev_b {
                let (lo, hi) = (pb.min(row_b), pb.max(row_b));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }

            // Fill below the B-curve (product accumulation region)
            for y in (row_b as usize)..h {
                // Every other dot for a lighter fill texture
                if (xi + y) % 2 == 0 {
                    draw::dot(grid, xi, y);
                }
            }

            prev_a = Some(row_a);
            prev_b = Some(row_b);
        }

        // Colour: start = reactant (red-ish via palette start), end = product
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Crystallization
//    An ionic (rock-salt) lattice grows from a seed at the centre. Lattice
//    sites sit every other cell; each site is a solid 2×2 ion, alternating
//    between two ion species in a checkerboard, joined to its neighbours by
//    bonds. The crystal's faceted front advances outward with eased progress
//    (middle rows lead, outer rows lag, giving stepped facets) while free ions
//    drift through the surrounding solution, driven by time.
// ---------------------------------------------------------------------------
struct Crystallization;
impl ProgressStyle for Crystallization {
    fn name(&self) -> &str {
        "crystallization"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Crystallization: a two-ion checkerboard lattice grows from a central seed with faceted edges while free ions drift in solution"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let kc = (cw / 2) as f32;
        let jc = (ch as f32 - 1.0) / 2.0;
        let radius = 2.0 + ctx.eased * (kc + jc * 3.0);
        let inside =
            |k: usize, j: usize| (k as f32 - kc).abs() + (j as f32 - jc).abs() * 3.0 <= radius;
        let ion_a = ctx.palette.sample(0.0);
        let ion_b = ctx.palette.sample(1.0);

        // --- Lattice: ions on even cells, bonds through the odd cells ---
        for j in 0..ch {
            let mut k = cw / 2 % 2;
            while k < cw {
                if inside(k, j) {
                    let (x, y) = (k * 2, j * 4);
                    draw::fill_rect(grid, x, y + 1, 2, 2);
                    let species = if (k / 2 + j) % 2 == 0 { ion_a } else { ion_b };
                    draw::tint_row(grid, j, k, k, species);
                    if k + 2 < cw && inside(k + 2, j) {
                        draw::hline(grid, x + 2, x + 3, y + 2);
                    }
                    if j + 1 < ch && inside(k, j + 1) {
                        draw::vline(grid, x, y + 3, y + 4);
                    }
                }
                k += 2;
            }
        }

        // --- Free ions drifting in the solution outside the crystal ---
        let (w, h) = draw::dot_dims(grid);
        let n = (cw / 2).max(1);
        for i in 0..n {
            let seed = hash2(i, 17);
            let bx = (seed % w.max(1) as u32) as f32;
            let by = ((seed >> 8) % h.max(1) as u32) as f32;
            let ph = (seed >> 16) as f32 * 0.001;
            let px = bx + (ctx.time * 0.9 + ph).sin() * 3.0;
            let py = by + (ctx.time * 1.3 + ph * 1.7).cos() * 1.5;
            if px < 0.0 || py < 0.0 {
                continue;
            }
            let (k, j) = ((px as usize) / 2, (py as usize) / 4);
            let near = (k as f32 - kc).abs() + (j as f32 - jc).abs() * 3.0 <= radius + 2.0;
            if !near {
                draw::dot(grid, px as usize, py as usize);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. pH scale
//    A shaded acid→base gradient track spans the full width.
//    The measured pH = eased × 14 positions a sliding indicator dot-column
//    that moves left (acid) to right (base). Structural novelty: it is a
//    SLIDING MARKER, not just changing colour.
// ---------------------------------------------------------------------------
struct PhScale;
impl ProgressStyle for PhScale {
    fn name(&self) -> &str {
        "ph-scale"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "pH scale: a marker slides left (acid) to right (base) along a shaded gradient track"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w == 0 || h == 0 || cw == 0 || ch == 0 {
            return Ok(());
        }

        // --- Gradient track using shade glyphs ---
        // Shade 0 (acid, left) → shade 4 (base, right) per cell
        for cx in 0..cw {
            let t = cx as f32 / cw.saturating_sub(1).max(1) as f32;
            // shade level: 0-1 at acid end, 3-4 at base end
            let level = (t * 4.0).round() as usize;
            for cy in 0..ch {
                // Middle rows get the full shade; outer rows get one step lighter
                let adj = if ch > 2 && (cy == 0 || cy == ch - 1) {
                    level.saturating_sub(1)
                } else {
                    level
                };
                draw::shade(grid, cx, cy, adj.min(4));
            }
            // Tint: red at acid end → purple/blue at base end
            // We use palette.sample across the full track
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }

        // --- Sliding indicator: a bright full-height block column ---
        // Must be a glyph: every track cell already holds a shade glyph, and
        // glyphs override braille dots, so a dot-drawn marker would be
        // invisible. pH = eased × 14, position = eased × (cw-1).
        let icx = (ctx.eased * cw.saturating_sub(1) as f32).round() as usize;
        for cy in 0..ch {
            draw::glyph(grid, icx, cy, '█');
            draw::tint_row(grid, cy, icx, icx, crate::Color::rgb(255, 250, 240));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Gas diffusion
//    Particles start at a release point (left centre) and random-walk
//    outward; the spread radius = eased × half_width.  Particle positions
//    are deterministic (seeded from particle index) and the cloud density
//    decays as a Gaussian so it looks like real diffusion.
// ---------------------------------------------------------------------------
struct GasDiffusion;
impl ProgressStyle for GasDiffusion {
    fn name(&self) -> &str {
        "gas-diffusion"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Gas diffusion: molecules spread from a point source — cloud density is a Gaussian growing with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Diffusion sigma in dot units grows with eased
        let sigma_x = (ctx.eased * w as f32 * 0.5).max(0.5);
        let sigma_y = (ctx.eased * h as f32 * 0.5).max(0.5);

        // Source at left-centre
        let src_x = 0.0_f32;
        let src_y = (h as f32) / 2.0;

        // Animate: the centre of mass drifts rightward with time
        let drift = (ctx.time * 2.0).min(w as f32 * 0.5);
        let cx = src_x + drift;

        // Draw the Gaussian density cloud using dots
        // To avoid O(w*h) being too slow for tiny grids, iterate all dots
        for yi in 0..h {
            for xi in 0..w {
                let dx = xi as f32 - cx;
                let dy = yi as f32 - src_y;
                let exponent =
                    (dx * dx) / (2.0 * sigma_x * sigma_x) + (dy * dy) / (2.0 * sigma_y * sigma_y);
                let density = (-exponent).exp(); // [0, 1]

                // Threshold density against a hash value for stipple effect
                let h_val = hash2(xi, yi) as f32 / u32::MAX as f32;
                if density > h_val * 0.9 + 0.05 {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        // Shade the spreading region via tint — colour gradient from source
        let (cw, ch) = grid.dimensions();
        let spread_cells = (ctx.eased * cw as f32).round() as usize;
        for cx2 in 0..spread_cells.min(cw) {
            let t = cx2 as f32 / cw as f32;
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Distillation
//    A classic still drawn left to right: a round-bottom boiling flask whose
//    liquid boils down, a slanted condenser arm (inner tube + cooling jacket)
//    carrying vapour, and an Erlenmeyer receiver whose distillate rises with
//    eased progress. time drives bubbles, vapour flow and the falling drip.
// ---------------------------------------------------------------------------
struct Distillation;
impl ProgressStyle for Distillation {
    fn name(&self) -> &str {
        "distillation"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Distillation: a boiling flask feeds vapour down a condenser; distillate drips into an Erlenmeyer receiver that fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (fw, fh) = (w as f32, h as f32);
        let hot = ctx.palette.sample(0.0);
        let cool = ctx.palette.sample(1.0);

        // --- Boiling flask (round bottom + neck) ---
        let r = (fh * 0.31).min(fw * 0.12).max(1.0);
        let fx = (fw * 0.04 + r + 1.0).round();
        let fy = fh - 1.0 - r;
        let neck_hw = (r * 0.3).max(1.0).round();
        let ri = r.ceil() as i32 + 1;
        // Liquid level drops as the charge boils away.
        let level = fy - r * 0.15 + ctx.eased * r * 0.6;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                let (x, y) = (fx as i32 + dx, fy as i32 + dy);
                if (d - r).abs() < 0.6 && !(dy < 0 && (dx as f32).abs() < neck_hw) {
                    draw::dot_i(grid, x, y);
                } else if d < r - 1.0 && y as f32 >= level {
                    draw::dot_i(grid, x, y);
                    if x >= 0 && y >= 0 {
                        let c = (x / 2) as usize;
                        draw::tint_row(grid, (y / 4) as usize, c, c, hot);
                    }
                }
            }
        }
        let neck_top = 1;
        let neck_bot = (fy - r).round() as i32;
        draw::vline(
            grid,
            (fx - neck_hw).max(0.0) as usize,
            neck_top,
            neck_bot.max(1) as usize,
        );
        draw::vline(
            grid,
            (fx + neck_hw) as usize,
            neck_top + 2,
            neck_bot.max(1) as usize,
        );
        draw::hline(
            grid,
            (fx - neck_hw).max(0.0) as usize,
            (fx + neck_hw) as usize,
            0,
        );
        // Bubbles popping at the surface.
        for i in 0..3 {
            let ph = (ctx.time * 3.0 + i as f32 * 0.37).fract();
            let bx = fx + (i as f32 - 1.0) * r * 0.45;
            let by = level - 1.0 - ph * 2.5;
            draw::dot_i(grid, bx.round() as i32, by.round() as i32);
        }

        // --- Condenser: straight tube with a cooling jacket, then a bend
        //     down into the receiver ---
        let x0 = (fx + neck_hw) as i32;
        let x1 = (fw * 0.80).round() as i32;
        let ty = ((fh * 0.13).round() as i32).max(1); // tube top wall
        let tb = ty + 2; // tube bottom wall; lumen is ty + 1
        draw::hline(
            grid,
            x0.max(0) as usize,
            (x1 + 1).max(0) as usize,
            ty as usize,
        );
        draw::hline(
            grid,
            x0.max(0) as usize,
            (x1 - 1).max(0) as usize,
            tb as usize,
        );
        // Cooling jacket around the middle of the tube, with end caps and
        // water ports.
        let j0 = x0 + (x1 - x0) / 5;
        let j1 = x1 - (x1 - x0) / 7;
        let (jt, jb) = (ty - 1, tb + ((fh * 0.15).round() as i32).max(1));
        draw::hline(
            grid,
            j0.max(0) as usize,
            j1.max(0) as usize,
            jt.max(0) as usize,
        );
        draw::hline(grid, j0.max(0) as usize, j1.max(0) as usize, jb as usize);
        line_i(grid, j0, jt, j0, jb);
        line_i(grid, j1, jt, j1, jb);
        line_i(grid, j0 + 3, jb, j0 + 3, jb + 2);
        line_i(grid, j1 - 3, jb, j1 - 3, jb + 2);
        // Bend down into the receiver neck.
        let rim = jb + 2;
        line_i(grid, x1 + 1, ty, x1 + 1, rim - 1);
        line_i(grid, x1 - 1, tb, x1 - 1, rim - 1);
        // Vapour packets flowing along the lumen.
        for i in 0..7 {
            let u = (ctx.time * 0.3 + i as f32 / 7.0).fract();
            let vx = x0 as f32 + (x1 - x0) as f32 * u;
            draw::dot_i(grid, vx.round() as i32, ty + 1);
        }

        // --- Erlenmeyer receiver ---
        let rx = x1;
        let nhw = 2_i32;
        let body0 = rim + ((fh * 0.1).round() as i32).max(1);
        let bottom = h as i32 - 1;
        let hb = (fh * 0.5).min(fw * 0.09).max(nhw as f32 + 1.0);
        draw::dot_i(grid, rx - nhw - 1, rim);
        draw::dot_i(grid, rx + nhw + 1, rim);
        for y in rim..body0 {
            draw::dot_i(grid, rx - nhw, y);
            draw::dot_i(grid, rx + nhw, y);
        }
        let body_h = (bottom - body0).max(1) as f32;
        let half = |y: i32| nhw as f32 + (hb - nhw as f32) * ((y - body0) as f32 / body_h);
        for y in body0..=bottom {
            let hw = half(y).round() as i32;
            draw::dot_i(grid, rx - hw, y);
            draw::dot_i(grid, rx + hw, y);
        }
        draw::hline(
            grid,
            (rx - hb.round() as i32).max(0) as usize,
            (rx + hb.round() as i32).max(0) as usize,
            bottom.max(0) as usize,
        );
        // Distillate fills from the bottom with eased progress.
        let rows = ((bottom - body0) as f32 * ctx.eased).round() as i32;
        let surface = bottom - rows;
        for y in surface..bottom {
            let hw = half(y).round() as i32 - 1;
            for x in (rx - hw)..=(rx + hw) {
                draw::dot_i(grid, x, y);
                if x >= 0 && y >= 0 {
                    let c = (x / 2) as usize;
                    draw::tint_row(grid, (y / 4) as usize, c, c, cool);
                }
            }
        }
        // A drip falls from the condenser tip to the liquid surface.
        let ph = (ctx.time * 1.6).fract();
        let dy = rim as f32 + ph * (surface - rim).max(0) as f32;
        draw::dot_i(grid, rx, dy.round() as i32);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Bond vibration
//    A diatomic molecule: two solid atoms joined by a coiled-spring bond.
//    The left atom is anchored; the right atom's equilibrium position moves
//    out with eased progress (the bond stretches toward dissociation) while
//    it vibrates about that point: x(t) = x₀ + A·sin(ω·t). The spring keeps a
//    fixed coil count so its coils visibly compress and stretch.
// ---------------------------------------------------------------------------
struct BondVibration;
impl ProgressStyle for BondVibration {
    fn name(&self) -> &str {
        "bond-vibration"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Bond vibration: two atoms on a coiled-spring bond vibrate; the bond stretches across the bar with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (fw, fh) = (w as f32, h as f32);
        let mid_y = (fh - 1.0) / 2.0;
        let r = (fh * 0.36).min(fw * 0.1).max(0.5);

        // Atom A anchored at the left; atom B's rest point stretches out.
        let ax = r + 1.0;
        let amp = (r * 0.7).max(1.0);
        let min_d = 2.0 * r + 2.0 * amp + 6.0;
        let max_d = (fw - 2.0 * r - 3.0).max(min_d);
        let d0 = min_d + ctx.eased * (max_d - min_d);
        let bx = ax + d0 + amp * (2.0 * PI * 1.25 * ctx.time).sin();

        // --- Spring: fixed coil count, so it compresses / stretches ---
        let s0 = ax + r + 1.0;
        let s1 = bx - r - 1.0;
        let coils = (((s1 - s0) / 6.0) as usize).clamp(2, 7);
        let tooth = (r * 0.75).max(1.0);
        let n = coils * 2;
        for i in 0..n {
            let xa = s0 + (s1 - s0) * i as f32 / n as f32;
            let xb = s0 + (s1 - s0) * (i + 1) as f32 / n as f32;
            let ya = if i == 0 {
                mid_y
            } else if i % 2 == 0 {
                mid_y + tooth
            } else {
                mid_y - tooth
            };
            let yb = if i + 1 == n {
                mid_y
            } else if i % 2 == 0 {
                mid_y - tooth
            } else {
                mid_y + tooth
            };
            line_i(
                grid,
                xa.round() as i32,
                ya.round() as i32,
                xb.round() as i32,
                yb.round() as i32,
            );
        }

        // --- Atoms: solid discs in two element colours ---
        disc(grid, ax, mid_y, r);
        disc(grid, bx, mid_y, r);
        tint_disc(grid, ax, mid_y, r, ctx.palette.sample(0.0));
        tint_disc(grid, bx, mid_y, r, ctx.palette.sample(1.0));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Boltzmann distribution
//     Velocity histogram filling up to the Maxwell-Boltzmann shape.
//     f(v) ∝ v²·exp(−v²/(2kT)) — temperature T = 1 + eased·3.
//     Each column (velocity bin) is a vblock glyph filled to the MB height;
//     eased controls how "hot" the distribution is (broader + shifted peak).
//     time animates a slight shimmer on bar heights.
// ---------------------------------------------------------------------------
struct BoltzmannDistribution;
impl ProgressStyle for BoltzmannDistribution {
    fn name(&self) -> &str {
        "boltzmann-distribution"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Maxwell-Boltzmann: velocity histogram builds up — temperature rises with progress, broadening the peak"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Temperature: low at start (sharp tall peak), high at end (broad flat)
        let temp = 0.3 + ctx.eased * 2.7; // T in [0.3, 3.0]

        // Maxwell-Boltzmann: f(v) = C · v² · exp(−v²/(2T))
        // v ranges from 0 to v_max; we pick v_max = 4·sqrt(T) for full coverage
        let v_max = 4.0 * temp.sqrt();

        // Compute raw heights
        let mut heights: Vec<f32> = Vec::with_capacity(cw);
        let mut max_h = 0.0_f32;
        for col in 0..cw {
            let v = (col as f32 + 0.5) / cw as f32 * v_max;
            let fv = v * v * (-(v * v) / (2.0 * temp)).exp();
            heights.push(fv);
            if fv > max_h {
                max_h = fv;
            }
        }

        // Animate: slight shimmer on bar heights (time-driven ripple)
        for (col, h_val) in heights.iter_mut().enumerate() {
            let ripple = 1.0 + 0.05 * (ctx.time * 3.0 + col as f32 * 0.3).sin();
            *h_val *= ripple;
        }

        // Re-check max after shimmer
        let max_h = heights.iter().copied().fold(0.0_f32, f32::max).max(0.001);

        // Draw each column using vblock glyphs from the bottom up
        for (col, &height) in heights.iter().enumerate() {
            let norm = height / max_h; // [0,1]
                                       // Each column spans ch cells vertically; fill bottom-up
            let total_eighths = (norm * ch as f32 * 8.0).round() as usize;
            let full_cells = total_eighths / 8;
            let partial = total_eighths % 8;

            // Full cells: filled from bottom
            for row in 0..full_cells.min(ch) {
                let cy = ch - 1 - row;
                draw::vblock(grid, col, cy, 8);
            }
            // Partial cell on top
            if partial > 0 && full_cells < ch {
                let cy = ch - 1 - full_cells;
                draw::vblock(grid, col, cy, partial);
            }

            // Colour: hot bins (near peak) get end-palette colour
            let t = height / max_h;
            for cy in 0..ch {
                draw::tint_row(grid, cy, col, col, ctx.palette.sample(t));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Flame test / spectral lines
//     Discrete emission lines appear left-to-right as progress advances.
//     Each line is a narrow bright vertical column of dots at a specific
//     wavelength position. time drives a flicker in intensity (flame effect).
//     Lines are drawn as vlines of varying height (intensity); some are
//     doublets (two adjacent dots wide).
// ---------------------------------------------------------------------------
struct FlameSpectralLines;
impl ProgressStyle for FlameSpectralLines {
    fn name(&self) -> &str {
        "flame-spectral-lines"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Flame test: discrete emission lines flicker into existence as progress reveals each spectral line"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Emission line wavelength positions (as fractions of bar width)
        // Based loosely on real alkali-metal / hydrogen lines:
        // H: 656nm(red), 486nm(cyan), 434nm(violet), 410nm(deep violet)
        // Na: 589nm(yellow doublet)
        // K:  766nm(IR), 404nm(violet)
        // Ca: 423nm, 442nm, 616nm, 646nm
        let lines: &[(f32, f32, bool)] = &[
            // (position_frac, relative_intensity, is_doublet)
            (0.08, 0.60, false), // K 766 nm (deep red)
            (0.20, 0.95, false), // H 656 nm (red Hα — strong)
            (0.38, 0.60, true),  // Na 589 nm (yellow doublet)
            (0.44, 0.45, false), // Ca 558 nm
            (0.52, 0.75, false), // H 486 nm (cyan Hβ)
            (0.60, 0.50, false), // Ca 442 nm
            (0.65, 0.40, false), // Ca 423 nm
            (0.72, 0.65, false), // H 434 nm (violet Hγ)
            (0.80, 0.35, false), // K  404 nm
            (0.88, 0.50, false), // H 410 nm (deep violet Hδ)
            (0.94, 0.30, false), // Ca 393 nm (UV edge)
        ];

        let n_lines = lines.len();

        // Dark background: draw a thin horizontal baseline
        draw::hline(grid, 0, w - 1, h - 1);

        // How many lines are visible: eased × n_lines
        let visible = (ctx.eased * n_lines as f32).round() as usize;

        for (li, &(pos_frac, intensity, doublet)) in lines.iter().enumerate() {
            if li >= visible {
                break;
            }

            // Flicker: multiply intensity by a time-driven noise factor
            let flicker = 0.7 + 0.3 * (ctx.time * (3.0 + li as f32 * 0.7)).sin().abs();
            let final_intensity = (intensity * flicker).min(1.0);

            // Column position in dots
            let xd = (pos_frac * (w - 1) as f32).round() as usize;

            // Height of the line proportional to intensity
            let line_h = (final_intensity * (h - 2) as f32).round() as usize;
            let y0 = (h - 1).saturating_sub(line_h);

            // Draw the emission line
            draw::vline(grid, xd.min(w - 1), y0, h - 2);

            // Doublet: draw an adjacent column
            if doublet && xd + 1 < w {
                let doublet_h = (line_h as f32 * 0.85).round() as usize;
                let dy0 = (h - 1).saturating_sub(doublet_h);
                draw::vline(grid, xd + 1, dy0, h - 2);
            }

            // Tint each line with a colour from the palette keyed to its position
            let (cw, ch) = grid.dimensions();
            let cx = (pos_frac * (cw - 1) as f32).round() as usize;
            let col = ctx.palette.sample(pos_frac);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx.min(cw - 1), cx.min(cw - 1), col);
            }
            if doublet {
                let cx2 = (cx + 1).min(cw - 1);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, cx2, cx2, ctx.palette.sample(pos_frac + 0.01));
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
    let styles = progress::styles::chemistry::styles();
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
