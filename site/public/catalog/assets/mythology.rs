//! `mythology` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O mythology.rs && ./mythology [style-name]
//! ```

const DEFAULT_STYLE: &str = "phoenix-rising";

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
    pub mod mythology {
//! Mythology-themed progress bars — fourteen animated braille styles drawn from
//! Greek, Norse, and world mythology.
//!
//! Each bar is structurally unique: distinct geometry, motion, and symbol
//! approach. Color alone is never the only differentiator. Every bar animates
//! with `ctx.time` and advances with `ctx.eased` progress.
//!
//! Styles:
//! 1. **phoenix-rising**   — raised-wing firebird climbs from embers, flame tail flickers
//! 2. **dragon-flight**    — horned bat-winged dragon flaps across breathing fire
//! 3. **hydra-heads**      — body sprouts swaying necks, up to nine snapping heads
//! 4. **kraken-depths**    — goggle-eyed squid surfaces and swims toward a ship
//! 5. **pegasus-gallop**   — white winged horse gallops over a stardust trail
//! 6. **medusa-gaze**      — snake-haired face whose gaze petrifies a line of warriors
//! 7. **minotaur-labyrinth** — red thread winds through a walled maze to the Minotaur
//! 8. **icarus-sun**       — winged figure climbs toward a blazing sun, shedding feathers
//! 9. **cerberus-watch**   — three-headed hound on a chain, heads bobbing and snapping
//! 10. **valkyrie-ride**   — winged-helmed rider on a galloping steed, spear levelled
//! 11. **yggdrasil-grows** — world tree grows: trunk, forked branches, crown, roots
//! 12. **ouroboros**       — scaled serpent coils a rotating ring until it bites its tail
//! 13. **trojan-horse**    — wheeled wooden horse rolls up to the gates of Troy
//! 14. **mjolnir-strike**  — Thor's hammer slams a growing rune road, lightning flies

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `mythology` theme.
///
/// Returns fourteen boxed mythology-themed bars. Each is structurally distinct
/// (geometry, motion, and symbol); color alone never separates them.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PhoenixRising),
        Box::new(DragonFlight),
        Box::new(HydraHeads),
        Box::new(KrakenDepths),
        Box::new(PegasusGallop),
        Box::new(MedusaGaze),
        Box::new(MinotaurLabyrinth),
        Box::new(IcarusSun),
        Box::new(CerberusWatch),
        Box::new(ValkyriRide),
        Box::new(YggdrasilGrows),
        Box::new(Ouroboros),
        Box::new(TrojanHorse),
        Box::new(MjolnirStrike),
    ]
}

// ---------------------------------------------------------------------------
// Shared sprite helpers
// ---------------------------------------------------------------------------
// The pictorial bars below are drawn from small ASCII sprites (`#` = dot) so
// each creature keeps a recognisable silhouette at 88x16 dots. All writes go
// through `draw`, so off-grid sprite dots are silently clipped.

/// Stamp an ASCII sprite with its top-left corner at dot `(x, y)`.
fn stamp(grid: &mut BrailleGrid, rows: &[&str], x: i32, y: i32) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            if c == b'#' {
                draw::dot_i(grid, x + dx as i32, y + dy as i32);
            }
        }
    }
}

/// Straight dot line from `(x0, y0)` to `(x1, y1)` (inclusive).
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for s in 0..=steps {
        let x = x0 + (x1 - x0) * s / steps;
        let y = y0 + (y1 - y0) * s / steps;
        draw::dot_i(grid, x, y);
    }
}

/// Tint every cell overlapping the dot rectangle `[x, x+w) × [y, y+h)`.
fn tint_box(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32, color: Color) {
    let (x1, y1) = (x + w - 1, y + h - 1);
    if w <= 0 || h <= 0 || x1 < 0 || y1 < 0 {
        return;
    }
    let (cx0, cx1) = ((x.max(0) / 2) as usize, (x1 / 2) as usize);
    for cy in (y.max(0) / 4)..=(y1 / 4) {
        draw::tint_row(grid, cy as usize, cx0, cx1, color);
    }
}

/// Fallback for grids too small to hold a sprite: a plain fill line.
fn tiny_fill(grid: &mut BrailleGrid, ctx: &BarContext, w: usize, h: usize) {
    if w == 0 || h == 0 {
        return;
    }
    let filled = (ctx.eased * w as f32).round() as usize;
    if filled > 0 {
        draw::hline(grid, 0, filled.min(w) - 1, h / 2);
    }
}

/// Side-on horse facing right, legs stretched (gallop frame A).
const HORSE_A: [&str; 13] = [
    "..................#.#...",
    "..................####..",
    ".................#######",
    "................####.###",
    "...............####...##",
    "..##..........####......",
    ".####.###########.......",
    "##..#############.......",
    "#...#############.......",
    "....#############.......",
    "....##.........##.......",
    "...##...........##......",
    "..##.............##.....",
];

/// The same horse with legs gathered under the body (gallop frame B).
const HORSE_B: [&str; 13] = [
    "..................#.#...",
    "..................####..",
    ".................#######",
    "................####.###",
    "...............####...##",
    "..##..........####......",
    ".####.###########.......",
    "#...#############.......",
    "#...#############.......",
    "....#############.......",
    ".....#.#......#.#.......",
    ".....#..#.....#..#......",
    "......#..#....#...#.....",
];

// ---------------------------------------------------------------------------
// 1. Phoenix Rising
// ---------------------------------------------------------------------------
// A heraldic firebird — raised wings, crested head, fanned tail — climbs from
// the left on a rising diagonal as progress advances. Its wings beat between
// two frames, flame licks off its tail, and a bed of embers glows beneath the
// ground it has already crossed.

const PHOENIX_UP: [&str; 12] = [
    "#....................#",
    "##........##........##",
    ".###......##......###.",
    ".####....####....####.",
    "..#####..####..#####..",
    "...######.##.######...",
    "....##############....",
    "......##########......",
    "........######........",
    ".......#.####.#.......",
    "......#..#..#..#......",
    ".....#..#....#..#.....",
];

const PHOENIX_DOWN: [&str; 12] = [
    "......................",
    "..........##..........",
    "..........##..........",
    ".........####.........",
    "##.......####.......##",
    ".#####....##....#####.",
    "...##############.....",
    "......##########......",
    "........######........",
    ".......#.####.#.......",
    "......#..#..#..#......",
    ".....#..#....#..#.....",
];

struct PhoenixRising;
impl ProgressStyle for PhoenixRising {
    fn name(&self) -> &str {
        "phoenix-rising"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A crested firebird beats its raised wings and climbs from the embers, trailing a flickering flame tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 14 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;

        // Bird climbs a diagonal: bottom-left at 0 %, upper-right at 100 %.
        let bx = (p * (wi - 22) as f32).round() as i32;
        let lift = (p * (hi - 14) as f32).round() as i32;
        let by = (hi - 14 - lift).max(0);
        let beat = ((ctx.time * 3.0) as i32).rem_euclid(2) == 0;
        stamp(grid, if beat { &PHOENIX_UP } else { &PHOENIX_DOWN }, bx, by);

        // Flame tail: flickering tongues dropping from the tail fan.
        let tail_len = 2 + ((ctx.time * 9.0).sin() * 1.5).round() as i32;
        for k in 0..5 {
            let fx = bx + 7 + k * 2;
            let len = tail_len + (k % 2) + ((ctx.time * 11.0 + k as f32).sin() > 0.0) as i32;
            for d in 0..len {
                draw::dot_i(grid, fx + ((d + k) % 2), by + 12 + d);
            }
        }

        // Ember bed along the ground the bird has crossed.
        let ground = hi - 1;
        for x in 0..(bx + 11).min(wi) {
            let flick = ((x as f32 * 1.7 + ctx.time * 10.0).sin() * 1.3).round() as i32;
            draw::dot_i(grid, x, ground);
            if flick > 0 {
                draw::dot_i(grid, x, ground - flick);
            }
        }

        // Colour: embers red, body orange, wing tips and crest gold.
        tint_box(grid, 0, hi - 4, wi, 4, Color::rgb(200, 50, 20));
        tint_box(grid, bx, by + 8, 22, 8, Color::rgb(255, 90, 20));
        tint_box(grid, bx, by + 4, 22, 4, Color::rgb(255, 150, 30));
        tint_box(grid, bx, by, 22, 4, Color::rgb(255, 215, 80));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Dragon Flight
// ---------------------------------------------------------------------------
// A side-on dragon — horned head, long neck, bat wing, clawed legs, whip
// tail — flies left→right with progress. The wing flaps between two frames,
// a cone of fire flickers from its jaws, and the ground it has passed over is
// left scorched.

const DRAGON: [&str; 12] = [
    "............................#....",
    "...........................###...",
    "..........................######.",
    ".........................####.###",
    "........................####.....",
    "........###############.####.....",
    "##....##################.........",
    ".##..##################..........",
    "..#######################........",
    "........#####....#####...........",
    "........#.#.#....#.#.#...........",
    ".......#..#..#..#..#..#..........",
];

/// Scalloped bat wing, raised (three finger spines along the top edge).
const BAT_WING_UP: [&str; 8] = [
    "#.....#.....#.........",
    "##....##....##........",
    ".##..####..####.......",
    ".###.#####.#####......",
    "..###############.....",
    "...###############....",
    ".....##############...",
    "........############..",
];

/// The same wing on the downstroke: flattened and swept back.
const BAT_WING_DOWN: [&str; 5] = [
    "#.....#.....#.........",
    "###..###...####.......",
    ".######..##########...",
    "..##################..",
    "....#################.",
];

struct DragonFlight;
impl ProgressStyle for DragonFlight {
    fn name(&self) -> &str {
        "dragon-flight"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A horned, bat-winged dragon flaps across breathing a flickering cone of fire over scorched ground"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 41 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Body travels with progress, leaving room for the fire cone.
        let dx = (ctx.eased * (wi - 41).max(0) as f32).round() as i32;
        let up = ((ctx.time * 3.0) as i32).rem_euclid(2) == 0;
        let dy = if up { 3 } else { 2 };
        stamp(grid, &DRAGON, dx, dy);
        if up {
            stamp(grid, &BAT_WING_UP, dx + 4, dy - 3);
        } else {
            stamp(grid, &BAT_WING_DOWN, dx + 4, dy + 1);
        }

        // Fire cone from the jaws: widens with distance, dots flicker.
        let (mx, my) = (dx + 33, dy + 3);
        let tick = (ctx.time * 12.0) as i32;
        for d in 0..8 {
            let spread = d / 2;
            for s in -spread..=spread {
                if (d * 7 + s * 13 + tick).rem_euclid(3) != 0 {
                    draw::dot_i(grid, mx + d, my + s);
                }
            }
        }

        // Scorched ground behind the dragon: a smouldering, flickering line.
        let ground = hi - 1;
        for x in 0..(dx + 16).min(wi) {
            if (x * 5 + tick).rem_euclid(7) != 0 {
                draw::dot_i(grid, x, ground);
            }
        }

        // Colour: body from the palette, fire gold-orange, scorch red.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.2));
        tint_box(grid, 0, hi - 2, dx + 16, 2, Color::rgb(190, 50, 20));
        tint_box(grid, mx, my - 4, 8, 9, Color::rgb(255, 150, 30));
        tint_box(grid, mx + 4, my - 4, 4, 9, Color::rgb(255, 220, 90));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Hydra Heads
// ---------------------------------------------------------------------------
// A many-headed serpent: a scaly body mound sits on the ground and a new
// neck sprouts for each step of progress (one head at 0 %, nine at 100 %).
// Each neck sways with time and ends in a horned dragon head whose jaws snap
// open and shut.

const HYDRA_HEAD_OPEN: [&str; 4] = [".##..", "#####", "##...", ".####"];
const HYDRA_HEAD_SHUT: [&str; 4] = [".##..", "#####", "#####", "....."];

struct HydraHeads;
impl ProgressStyle for HydraHeads {
    fn name(&self) -> &str {
        "hydra-heads"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A serpent body sprouts a new swaying neck with progress, up to nine snapping dragon heads"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 20 || h < 12 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let heads = 1 + (ctx.eased * 8.0).round() as i32;

        // Body: a long mound along the ground under all the necks.
        let body_w = 8 + (heads - 1) * (wi - 14) / 8;
        let bx = 2;
        for x in 0..body_w {
            let t = x as f32 / body_w.max(1) as f32;
            let hump = ((t * PI).sin() * 3.0).round() as i32 + 1;
            for y in 0..=hump {
                draw::dot_i(grid, bx + x, hi - 1 - y);
            }
        }

        // Necks: one per head, rising from the body and swaying with time.
        let gap = (body_w - 6).max(1) / (heads - 1).max(1);
        for i in 0..heads {
            let root = bx + 3 + i * gap;
            let neck_top = 3 + (i % 3);
            let mut end = (root, hi - 4);
            for y in (neck_top..=hi - 4).rev() {
                let d = (hi - 4 - y) as f32;
                let sway = (ctx.time * 2.5 + i as f32 * 1.3 + d * 0.3).sin() * d * 0.25;
                let x = root + sway.round() as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
                end = (x, y);
            }
            let open = ((ctx.time * 3.0 + i as f32 * 0.7) as i32).rem_euclid(2) == 0;
            let head = if open {
                &HYDRA_HEAD_OPEN
            } else {
                &HYDRA_HEAD_SHUT
            };
            stamp(grid, head, end.0, end.1 - 3);
        }

        // Colour: body and necks in the palette, heads a venomous green.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.3));
        tint_box(grid, 0, 0, wi, 8, Color::rgb(110, 220, 110));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Kraken Depths
// ---------------------------------------------------------------------------
// A giant squid surfaces and swims toward a ship at the far right as progress
// advances: a domed mantle with two staring eyes breaks the rolling waterline
// while a fan of tentacles undulates beneath it. At 100 % it reaches the hull.

const SHIP: [&str; 8] = [
    ".....#....",
    ".....##...",
    ".....###..",
    ".....####.",
    ".....#....",
    "##########",
    ".########.",
    "..######..",
];

struct KrakenDepths;
impl ProgressStyle for KrakenDepths {
    fn name(&self) -> &str {
        "kraken-depths"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A goggle-eyed kraken surfaces and swims toward a ship, tentacles writhing below the rolling waterline"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 36 || h < 15 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = 6;

        // Ship riding the waterline at the right edge.
        let ship_x = wi - 10;
        stamp(grid, &SHIP, ship_x, sea - 5);

        // Kraken position: centre of the mantle travels toward the ship.
        let kx = 7 + (ctx.eased * (wi - 26) as f32).round() as i32;
        let bob = ((ctx.time * 2.0).sin() * 0.8).round() as i32;
        let top = 1 + bob;

        // Mantle: a dome 13 dots wide, with two eye holes left unlit.
        for dy in 0..7i32 {
            let half = match dy {
                0 => 2,
                1 => 4,
                2 => 5,
                _ => 6,
            };
            for dx in -half..=half {
                let eye = dy == 4 && (dx == -3 || dx == 3);
                if !eye {
                    draw::dot_i(grid, kx + dx, top + dy);
                }
            }
        }

        // Tentacles: eight curling ribbons hanging from the mantle.
        for i in 0..8i32 {
            let base = kx - 6 + i * 12 / 7;
            for d in 0..(hi - top - 7) {
                let sway = (ctx.time * 3.0 + d as f32 * 0.5 + i as f32 * 0.9).sin();
                let x = base + (sway * d as f32 * 0.25).round() as i32 - (d / 3);
                draw::dot_i(grid, x, top + 7 + d);
            }
        }

        // Rolling waterline across the whole width.
        for x in 0..wi {
            let crest = ((x as f32 * 0.5 + ctx.time * 4.0).sin() * 0.9).round() as i32;
            draw::dot_i(grid, x, sea + crest);
        }

        // Colour: sea from the palette, kraken crimson, ship timber.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.0));
        tint_box(grid, kx - 9, top + 7, 20, hi, Color::rgb(170, 40, 90));
        tint_box(grid, kx - 6, top, 13, 7, Color::rgb(210, 60, 100));
        tint_box(grid, ship_x, 0, 10, sea + 3, Color::rgb(200, 150, 90));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Pegasus Gallop
// ---------------------------------------------------------------------------
// A white winged horse gallops through the sky left→right with progress. Its
// legs alternate between stretched and gathered frames, its feathered wing
// beats up and down, and a twinkling stardust trail marks the distance run.

const WING_UP: [&str; 8] = [
    "#.............",
    "###...........",
    ".####.........",
    "..#####.......",
    "...#######....",
    "....########..",
    ".....#########",
    "......########",
];

const WING_DOWN: [&str; 4] = [
    "######........",
    ".##########...",
    "...###########",
    ".....########.",
];

struct PegasusGallop;
impl ProgressStyle for PegasusGallop {
    fn name(&self) -> &str {
        "pegasus-gallop"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A white winged horse gallops across the sky, legs and feathered wings beating, over a stardust trail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 26 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        let x = (ctx.eased * (wi - 25) as f32).round() as i32;
        let stride = ((ctx.time * 6.0) as i32).rem_euclid(2) == 0;
        let y = if stride { 3 } else { 2 };
        stamp(grid, if stride { &HORSE_A } else { &HORSE_B }, x, y);
        // The wing roots at the withers and beats between two frames.
        if ((ctx.time * 3.0) as i32).rem_euclid(2) == 0 {
            stamp(grid, &WING_UP, x + 1, y - 3);
        } else {
            stamp(grid, &WING_DOWN, x, y + 3);
        }

        // Stardust trail: twinkling dots strewn behind the hooves.
        let tick = (ctx.time * 8.0) as i32;
        for tx in 0..x.min(wi) {
            let row = hi - 2 - (tx * 7).rem_euclid(4);
            if (tx * 3 + tick).rem_euclid(5) == 0 || tx % 2 == 0 && row == hi - 2 {
                draw::dot_i(grid, tx, row);
            }
        }

        // Colour: trail from the palette, the horse bright white, wing silver.
        tint_box(grid, 0, hi - 6, wi, 6, ctx.palette.sample(0.4));
        tint_box(grid, x, y, 25, 13, Color::rgb(240, 245, 255));
        tint_box(grid, x, y - 3, 12, 6, Color::rgb(190, 215, 255));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Medusa Gaze
// ---------------------------------------------------------------------------
// Medusa's face sits at the left, crowned by writhing snakes. A golden gaze
// beam shoots from her eyes and lengthens with progress; every spear-carrying
// warrior it has passed is turned to grey stone, while those still ahead of
// it march in place, bronze and alive.

const WARRIOR: [&str; 7] = [
    ".##.#", ".##.#", "#####", ".##.#", ".##.#", ".#.##", "#...#",
];

struct MedusaGaze;
impl ProgressStyle for MedusaGaze {
    fn name(&self) -> &str {
        "medusa-gaze"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Snake-haired Medusa stares down a line of warriors; her lengthening gaze turns each one to grey stone"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 30 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let (fx, fy) = (9, 7);

        // Face: an oval with two eye holes and a mouth slit left unlit.
        for dy in -5..=5i32 {
            for dx in -4..=4i32 {
                let eye = dy == -1 && (dx == -2 || dx == 2);
                let mouth = dy == 3 && dx.abs() <= 1;
                if dx * dx * 25 + dy * dy * 16 <= 400 && !eye && !mouth {
                    draw::dot_i(grid, fx + dx, fy + dy);
                }
            }
        }

        // Snake hair: seven serpents radiating from the crown, each swaying.
        for i in 0..7 {
            let base = -PI - 0.3 + i as f32 * (PI + 0.6) / 6.0;
            let len = 6 + (i % 2) * 2;
            let mut end = (fx, fy);
            for d in 0..len {
                let r = 5.0 + d as f32;
                let a = base + (ctx.time * 4.0 + d as f32 * 0.8 + i as f32).sin() * 0.22;
                let x = fx + (a.cos() * r * 1.4).round() as i32;
                let y = fy + (a.sin() * r * 0.9).round() as i32;
                draw::dot_i(grid, x, y);
                end = (x, y);
            }
            draw::dot_i(grid, end.0 + 1, end.1);
            draw::dot_i(grid, end.0, end.1 - 1);
        }

        // Gaze beam: two scrolling rays from the eyes, length = progress.
        let start = fx + 4;
        let tip = start + (ctx.eased * (wi - start) as f32).round() as i32;
        let scroll = (ctx.time * 24.0) as i32;
        for x in start..tip.min(wi) {
            if (x - scroll).rem_euclid(4) != 0 {
                draw::dot_i(grid, x, fy - 2);
            }
            if (x - scroll + 2).rem_euclid(4) != 0 {
                draw::dot_i(grid, x, fy - 1);
            }
        }

        // Warriors line up to the right; the living ones march in place.
        let mut wx = 24;
        let step = ((ctx.time * 4.0) as i32).rem_euclid(2);
        while wx + 5 <= wi {
            let stone = wx + 2 < tip;
            let lift = if stone { 0 } else { step };
            stamp(grid, &WARRIOR, wx, hi - 7 - lift);
            let color = if stone {
                Color::rgb(140, 140, 150)
            } else {
                Color::rgb(215, 150, 70)
            };
            tint_box(grid, wx, hi - 8, 5, 8, color);
            wx += 11;
        }

        // Colour: hair dark green, face pale green, gaze gold.
        tint_box(grid, 0, 0, 22, hi, Color::rgb(60, 170, 80));
        tint_box(grid, fx - 4, fy - 4, 9, 9, Color::rgb(150, 220, 150));
        tint_box(
            grid,
            start,
            fy - 2,
            tip - start,
            2,
            Color::rgb(255, 215, 80),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Minotaur Labyrinth
// ---------------------------------------------------------------------------
// A walled serpentine labyrinth: vertical walls with gaps alternating top and
// bottom. Ariadne's red thread unwinds from the entrance, snaking through
// each gap as progress advances, toward the horned, bull-headed Minotaur that
// paces in the inner chamber at the far end.

const MINOTAUR: [&str; 14] = [
    "#...........#",
    "##.........##",
    ".##.......##.",
    "..##.###.##..",
    "...#######...",
    "....#.#.#....",
    "....#####....",
    ".....###.....",
    "...#######...",
    "..#########..",
    "..##.###.##..",
    "..##.###.##..",
    "....##.##....",
    "....#...#....",
];

struct MinotaurLabyrinth;
impl ProgressStyle for MinotaurLabyrinth {
    fn name(&self) -> &str {
        "minotaur-labyrinth"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Ariadne's red thread winds through a walled labyrinth toward the horned Minotaur in its inner chamber"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 36 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = hi / 2;

        // Outer walls, with the entrance gap on the left.
        draw::hline(grid, 0, w - 1, 0);
        draw::hline(grid, 0, w - 1, h - 1);
        draw::vline(grid, w - 1, 0, h - 1);
        draw::vline(grid, 0, 0, (mid - 3) as usize);
        draw::vline(grid, 0, (mid + 3) as usize, h - 1);

        // Inner walls every 8 dots, gaps alternating top / bottom.
        let walls = (wi - 18) / 8;
        for k in 1..=walls {
            let x = (k * 8) as usize;
            if k % 2 == 1 {
                draw::vline(grid, x, 5, h - 1);
            } else {
                draw::vline(grid, x, 0, h - 6);
            }
        }

        // Ariadne's thread: a square-wave path through every gap, ending at
        // the chamber. Walk its polyline and light the first `eased` share.
        let mut pts = vec![(0, mid), (4, mid)];
        for k in 1..=walls {
            let gy = if k % 2 == 1 { 2 } else { hi - 3 };
            pts.push((k * 8 - 4, gy));
            pts.push((k * 8 + 4, gy));
        }
        pts.push((walls * 8 + 4, mid));
        pts.push((wi - 16, mid));
        let total: i32 = pts
            .windows(2)
            .map(|s| (s[1].0 - s[0].0).abs() + (s[1].1 - s[0].1).abs())
            .sum();
        let mut left = (ctx.eased * total as f32).round() as i32;
        let mut lit = Vec::new();
        for s in pts.windows(2) {
            let ((x0, y0), (x1, y1)) = (s[0], s[1]);
            let len = (x1 - x0).abs() + (y1 - y0).abs();
            let take = left.min(len);
            for d in 0..=take {
                let p = (x0 + (x1 - x0).signum() * d, y0 + (y1 - y0).signum() * d);
                draw::dot_i(grid, p.0, p.1);
                lit.push(p);
            }
            left -= take;
            if left <= 0 {
                break;
            }
        }
        // Theseus' torch: a flickering spark at the thread's tip.
        let tip = lit.last().copied().unwrap_or(pts[0]);
        if ((ctx.time * 6.0) as i32).rem_euclid(2) == 0 {
            draw::dot_i(grid, tip.0 + 1, tip.1 - 1);
            draw::dot_i(grid, tip.0 - 1, tip.1 - 1);
        }

        // The Minotaur paces in the chamber, head bobbing.
        let pace = ((ctx.time * 1.5).sin() * 2.0).round() as i32;
        let bob = ((ctx.time * 3.0) as i32).rem_euclid(2);
        let mx = wi - 15 + pace;
        stamp(grid, &MINOTAUR, mx, 1 + bob);

        // Colour: walls in the palette, thread red, Minotaur bronze, horns ivory.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.25));
        for &(x, y) in &lit {
            tint_box(grid, x, y, 1, 1, Color::rgb(235, 60, 70));
        }
        tint_box(grid, mx, 4, 13, hi - 4, Color::rgb(200, 130, 70));
        tint_box(grid, mx, 0, 13, 4, Color::rgb(240, 230, 200));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Icarus Sun
// ---------------------------------------------------------------------------
// A blazing sun with turning rays waits at the right edge. Icarus — a small
// figure with outstretched feathered wings — climbs from the sea toward it as
// progress advances. The closer he gets, the more wax melts: loose feathers
// tumble down toward the rolling waves below.

const ICARUS_UP: [&str; 8] = [
    "#...............#",
    "##......#......##",
    ".###...###...###.",
    "..####..#..####..",
    "...###########...",
    ".......###.......",
    ".......###.......",
    "......#...#......",
];

const ICARUS_DOWN: [&str; 8] = [
    ".................",
    "........#........",
    ".......###.......",
    "........#........",
    "#################",
    ".####..###..####.",
    "..##...###...##..",
    "......#...#......",
];

struct IcarusSun;
impl ProgressStyle for IcarusSun {
    fn name(&self) -> &str {
        "icarus-sun"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Winged Icarus climbs from the waves toward a blazing sun, shedding more melted feathers the closer he flies"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 40 || h < 14 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;

        // Sun: filled disc with eight rays that slowly turn.
        let (sx, sy) = (wi - 9, 6);
        for dy in -4..=4i32 {
            for dx in -5..=5i32 {
                if dx * dx * 16 + dy * dy * 25 <= 400 {
                    draw::dot_i(grid, sx + dx, sy + dy);
                }
            }
        }
        for r in 0..8 {
            let a = r as f32 * PI / 4.0 + ctx.time * 0.8;
            for d in 7..9 {
                let rx = sx + (a.cos() * d as f32 * 1.2).round() as i32;
                let ry = sy + (a.sin() * d as f32 * 0.7).round() as i32;
                draw::dot_i(grid, rx, ry);
            }
        }

        // Icarus climbs from low-left toward the sun.
        let ix = (p * (wi - 38) as f32).round() as i32;
        let iy = (hi - 10) - (p * (hi - 10) as f32).round() as i32;
        let flap = ((ctx.time * 4.0) as i32).rem_euclid(2) == 0;
        stamp(grid, if flap { &ICARUS_UP } else { &ICARUS_DOWN }, ix, iy);

        // Melting feathers: more fall as he nears the sun, drifting down-left.
        let feathers = (p * 7.0).round() as i32;
        for k in 0..feathers {
            let fall = (ctx.time * 5.0 + k as f32 * 2.3) % (hi as f32);
            let fx = ix + 2 + (k * 5) % 14 - (fall * 0.4) as i32;
            let fy = iy + 4 + fall as i32;
            if fy < hi - 2 {
                draw::dot_i(grid, fx, fy);
                draw::dot_i(grid, fx + 1, fy);
            }
        }

        // The sea: a rolling wave line along the bottom.
        for x in 0..wi {
            let wave = ((x as f32 * 0.45 - ctx.time * 3.0).sin() + 1.0).round() as i32;
            draw::dot_i(grid, x, hi - 1 - wave.min(1));
        }

        // Colour: sea blue, Icarus pale, feathers wax-gold, sun hot.
        tint_box(grid, 0, 0, wi, hi, Color::rgb(235, 215, 170));
        tint_box(grid, 0, hi - 4, wi, 4, Color::rgb(40, 120, 220));
        tint_box(grid, sx - 10, 0, 20, 16, Color::rgb(255, 200, 40));
        tint_box(grid, sx - 4, sy - 2, 8, 5, Color::rgb(255, 240, 150));
        tint_box(grid, ix, iy, 17, 8, Color::rgb(245, 235, 220));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Cerberus Watch
// ---------------------------------------------------------------------------
// The three-headed hound of the underworld pads left→right with progress on a
// chain staked at the left edge. Three pointy-eared heads on separate necks
// bob independently as they keep watch, each snapping its jaws in turn, while
// its legs alternate stride frames and its tail curls up behind.

const HOUND_A: [&str; 12] = [
    "#...................",
    ".#..................",
    ".#..................",
    "..#################.",
    "..#################.",
    "...################.",
    "...##.#.......##.#..",
    "..#....#.....#....#.",
    ".#.....#....#.....#.",
    ".#......#...#......#",
    "##......##.##......##",
    "....................",
];

const HOUND_B: [&str; 12] = [
    "#...................",
    ".#..................",
    ".#..................",
    "..#################.",
    "..#################.",
    "...################.",
    "....##.#......##.#..",
    "....#..#......#..#..",
    "....#..#......#..#..",
    "....#..#......#..#..",
    "...##.##.....##.##..",
    "....................",
];

const DOG_HEAD_SHUT: [&str; 6] = [
    ".#..#....",
    "##.##....",
    "########.",
    "##.######",
    "#########",
    ".###.....",
];
const DOG_HEAD_OPEN: [&str; 6] = [
    ".#..#....",
    "##.##....",
    "########.",
    "##.######",
    "#####....",
    ".########",
];

struct CerberusWatch;
impl ProgressStyle for CerberusWatch {
    fn name(&self) -> &str {
        "cerberus-watch"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Three-headed Cerberus pads along on its chain, each pointy-eared head bobbing and snapping as it keeps watch"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 40 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Hound body travels with progress; legs alternate stride frames.
        let x = 4 + (ctx.eased * (wi - 42) as f32).round() as i32;
        let stride = ((ctx.time * 5.0) as i32).rem_euclid(2) == 0;
        stamp(grid, if stride { &HOUND_A } else { &HOUND_B }, x, hi - 11);

        // Three necks fan out from the shoulders: one head reared high, one
        // thrust forward, one held low.
        let heads = [(16, 0), (28, 4), (22, 10)];
        let roots = [(17, 8), (18, 8), (18, 10)];
        for (i, (&(hx, hy), &(rx, ry))) in heads.iter().zip(roots.iter()).enumerate() {
            let bob =
                ((ctx.time * (2.0 + i as f32 * 0.6) + i as f32 * 2.0).sin() * 0.6).round() as i32;
            let (hx, hy) = (x + hx, hy + bob);
            line(grid, x + rx, ry, hx + 1, hy + 4);
            let open = ((ctx.time * 3.0 + i as f32 * 0.66) as i32).rem_euclid(3) == 0;
            stamp(
                grid,
                if open { &DOG_HEAD_OPEN } else { &DOG_HEAD_SHUT },
                hx,
                hy,
            );
        }

        // Stake at the left and a sagging chain to the hound's harness.
        draw::vline(grid, 0, 7, h - 1);
        draw::vline(grid, 1, 7, h - 1);
        let (cx1, cy1) = (x + 3, 9);
        let span = (cx1 - 2).max(1);
        for cx in 2..cx1 {
            let t = (cx - 2) as f32 / span as f32;
            let cy = cy1 + ((t * PI).sin() * 3.0).round() as i32;
            match cx.rem_euclid(4) {
                0 | 2 => draw::dot_i(grid, cx, cy),
                1 => {
                    draw::dot_i(grid, cx, cy - 1);
                    draw::dot_i(grid, cx, cy + 1);
                }
                _ => {}
            }
        }

        // Colour: chain steel, hound ash-brown, heads a darker brown.
        tint_box(grid, 0, 0, wi, hi, Color::rgb(170, 170, 185));
        tint_box(grid, 0, 6, 2, 10, Color::rgb(150, 100, 60));
        tint_box(grid, x, 0, 36, hi, Color::rgb(160, 110, 90));
        // Each head its own shade, so the three read apart.
        let shades = [
            Color::rgb(120, 78, 64),
            Color::rgb(205, 150, 120),
            Color::rgb(140, 92, 74),
        ];
        for (&(hx, hy), &shade) in heads.iter().zip(shades.iter()) {
            tint_box(grid, x + hx, hy, 9, 6, shade);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Valkyrie Ride
// ---------------------------------------------------------------------------
// A valkyrie in a winged helmet rides a galloping horse across the sky with
// her spear levelled ahead of her. The horse's legs alternate stride frames,
// and streaming speed lines behind her mark the distance ridden.

const VALKYRIE: [&str; 8] = [
    "#.....#", "##.#.##", "..###..", "...#...", ".#####.", "#.###.#", "..###..", "..#.#..",
];

struct ValkyriRide;
impl ProgressStyle for ValkyriRide {
    fn name(&self) -> &str {
        "valkyrie-ride"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A winged-helmed valkyrie gallops across the sky on her steed, spear levelled, trailing streaming speed lines"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 36 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        let x = (ctx.eased * (wi - 35) as f32).round() as i32;
        let stride = ((ctx.time * 6.0) as i32).rem_euclid(2) == 0;
        let y = if stride { 3 } else { 2 };
        stamp(grid, if stride { &HORSE_A } else { &HORSE_B }, x, y);

        // Rider on the horse's back, spear couched forward past its head.
        let (rx, ry) = (x + 7, y - 2);
        stamp(grid, &VALKYRIE, rx, ry);
        line(grid, rx + 6, ry + 5, x + 31, ry);
        stamp(grid, &["#..", "###", "#.."], x + 31, ry - 1);

        // Speed lines streaming behind her, scrolling with time.
        let scroll = (ctx.time * 30.0) as i32;
        for (row, phase) in [(y + 5, 0), (y + 8, 7), (y + 11, 3)] {
            for tx in 0..x.min(wi) {
                if (tx + scroll + phase).rem_euclid(9) < 5 {
                    draw::dot_i(grid, tx, row);
                }
            }
        }

        // Colour: lines from the palette, steed slate, rider gold, spear steel.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.6));
        tint_box(grid, x, y, 25, 13, Color::rgb(150, 160, 190));
        tint_box(grid, rx, ry, 7, 6, Color::rgb(250, 200, 70));
        tint_box(grid, rx + 8, ry - 1, 27, 4, Color::rgb(225, 230, 240));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Yggdrasil Grows
// ---------------------------------------------------------------------------
// The world tree grows from a sapling into a vast ash as progress advances: a
// thick trunk rises from the ground line, forked branches reach out, a broad
// dome of leaves widens over them (rustling with time), and roots spread
// down into the earth below.

struct YggdrasilGrows;
impl ProgressStyle for YggdrasilGrows {
    fn name(&self) -> &str {
        "yggdrasil-grows"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "The world tree grows from sapling to vast ash: trunk rises, branches fork, a rustling crown widens, roots spread"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 12 || h < 12 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;
        let cx = wi / 2;
        let ground = hi - 4;

        // Trunk: grows taller and thicker with progress.
        let trunk_h = 2 + (p * (ground - 5) as f32).round() as i32;
        let top = ground - trunk_h;
        let half = if p > 0.5 { 2 } else { 1 };
        for y in top..=ground {
            for dx in -half..=half {
                draw::dot_i(grid, cx + dx, y);
            }
        }

        // Crown: a dome of leaves whose half-width grows with progress.
        let rx = 3.0 + p * (wi as f32 / 2.0 - 5.0);
        let ry = 1.5 + p * (top as f32 - 0.5).max(0.0);
        let tick = (ctx.time * 6.0) as i32;
        let crown_y = top as f32;
        for y in 0..=top {
            for x in 0..wi {
                let nx = (x - cx) as f32 / rx;
                let ny = (y as f32 - crown_y) / ry.max(1.0);
                if nx * nx + ny * ny <= 1.0 && (x * 3 + y * 5 + tick).rem_euclid(5) != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Branches: forks from the trunk top out toward the crown's edge.
        for side in [-1, 1] {
            let reach = (rx * 0.8) as i32;
            line(
                grid,
                cx,
                top + 2,
                cx + side * reach,
                top - (ry * 0.5) as i32,
            );
            line(grid, cx, top + 1, cx + side * reach / 2, top - ry as i32);
        }

        // Ground line and roots spreading below it.
        draw::hline(grid, 0, w - 1, ground as usize);
        let root_reach = (p * wi as f32 * 0.4) as i32 + 2;
        for (k, side) in [(0, -1), (1, 1), (2, -1), (3, 1)] {
            let len = root_reach - k * root_reach / 5;
            line(grid, cx, ground, cx + side * len, hi - 1 - (k % 2));
        }

        // Colour: leaves green, trunk and roots bark brown, ground earthy.
        tint_box(grid, 0, 0, wi, ground, Color::rgb(70, 200, 90));
        tint_box(grid, cx - 3, top + 1, 6, trunk_h, Color::rgb(170, 110, 60));
        tint_box(grid, 0, ground, wi, 4, Color::rgb(140, 90, 50));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Ouroboros
// ---------------------------------------------------------------------------
// A thick-bodied serpent coils around an oval ring. Its body grows head-first
// around the loop with progress — scaled with a diamond pattern, tapering to
// a pointed tail — and at 100 % its open jaws close on its own tail. The whole
// ring slowly rotates with time.

struct Ouroboros;
impl ProgressStyle for Ouroboros {
    fn name(&self) -> &str {
        "ouroboros"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A scaled serpent coils around a rotating ring, its open jaws closing on its own tail at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 8 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0 - 0.5);
        let rx = w as f32 / 2.0 - 4.0;
        let ry = h as f32 / 2.0 - 2.0;

        // Body spans from the tail angle to the head angle; a gap remains
        // until progress completes. Rotation keeps it moving.
        let arc = 0.5 + ctx.eased * (2.0 * PI - 0.8);
        let rot = ctx.time * 0.5;
        let steps = (arc * rx * 1.4) as usize + 2;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let a = rot + t * arc;
            // Thickness tapers from tail (1) to body (3 rings of dots).
            let thick = if t < 0.08 {
                1
            } else if t < 0.2 {
                2
            } else {
                3
            };
            for layer in 0..thick {
                let k = layer as f32 - (thick - 1) as f32 / 2.0;
                // Diamond scale pattern: skip a lattice of dots on the body.
                if thick == 3 && layer == 1 && s % 6 == 3 {
                    continue;
                }
                let x = cx + a.cos() * (rx + k * 1.6);
                let y = cy + a.sin() * (ry + k);
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
        }

        // Head: a wedge at the leading end with open jaws and an eye.
        let ha = rot + arc;
        let (hx, hy) = (
            (cx + ha.cos() * rx).round() as i32,
            (cy + ha.sin() * ry).round() as i32,
        );
        // Tangent direction (where the head points) in dot space.
        let (tx, ty) = (-ha.sin() * rx, ha.cos() * ry);
        let len = (tx * tx + ty * ty).sqrt().max(0.01);
        let (ux, uy) = (tx / len, ty / len);
        let gape = if ctx.eased >= 1.0 {
            0.0
        } else {
            1.5 + (ctx.time * 5.0).sin()
        };
        for d in 0..5 {
            let df = d as f32;
            for side in [-1.0f32, 1.0] {
                let spread = 1.4 + df * gape * 0.3;
                let jx = hx as f32 + ux * df - uy * side * spread;
                let jy = hy as f32 + uy * df + ux * side * spread * 0.6;
                draw::dot_i(grid, jx.round() as i32, jy.round() as i32);
            }
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                draw::dot_i(grid, hx + dx, hy + dy);
            }
        }

        // Colour: body along the palette, head gold.
        let (cells_w, _) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.max(1) as f32;
            tint_box(grid, cx_c as i32 * 2, 0, 2, h as i32, ctx.palette.sample(t));
        }
        tint_box(grid, hx - 5, hy - 4, 11, 9, Color::rgb(255, 210, 70));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 13. Trojan Horse
// ---------------------------------------------------------------------------
// The great wooden horse, stood on a wheeled platform, is rolled left→right
// toward the walls of Troy — a crenellated wall between two towers with an
// arched gate. Its wheels turn as it moves; at 100 % it reaches the gate.

const TROY_WALL: [&str; 14] = [
    "###.#.###.......###.#.###",
    "#########.......#########",
    "#########.#.#.#.#########",
    "#########################",
    "#########################",
    "#########################",
    "#########################",
    "#########.......#########",
    "########.........########",
    "########.........########",
    "########.........########",
    "########.........########",
    "########.........########",
    "########.........########",
];

struct TrojanHorse;
impl ProgressStyle for TrojanHorse {
    fn name(&self) -> &str {
        "trojan-horse"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "The wooden horse rolls on turning wheels toward the towered walls and arched gate of Troy"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 52 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // The walls of Troy stand at the right edge.
        let troy = wi - 25;
        stamp(grid, &TROY_WALL, troy, hi - 14);

        // Horse on its platform, rolled toward the gate with progress.
        let x = (ctx.eased * (troy - 25) as f32).round() as i32;
        stamp(grid, &HORSE_B, x, 0);
        draw::hline(grid, x.max(0) as usize, (x + 22).max(0) as usize, 12);

        // Wheels: rings of six dots with a gap that turns as the horse rolls.
        let spin = (x / 2).rem_euclid(3);
        let ring = [(-2, 0), (-1, -1), (1, -1), (2, 0), (1, 1), (-1, 1)];
        for wx in [x + 4, x + 18] {
            draw::dot_i(grid, wx, 14);
            for (i, &(dx, dy)) in ring.iter().enumerate() {
                if (i as i32 + spin) % 3 != 0 {
                    draw::dot_i(grid, wx + dx, 14 + dy);
                }
            }
        }

        // Dust kicked up behind the platform, drifting with time.
        let tick = (ctx.time * 8.0) as i32;
        for d in 0..x.min(wi) {
            if (d * 5 + tick).rem_euclid(11) == 0 {
                draw::dot_i(grid, d, hi - 1 - (d + tick).rem_euclid(2));
            }
        }

        // Colour: horse timber, platform dark wood, Troy sandstone.
        tint_box(grid, 0, 0, wi, hi, Color::rgb(150, 120, 90));
        tint_box(grid, x, 0, 24, 13, Color::rgb(200, 140, 70));
        tint_box(grid, troy, 0, 25, hi, Color::rgb(220, 205, 160));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 14. Mjolnir Strike
// ---------------------------------------------------------------------------
// Thor's hammer — a broad steel head on a short leather-wrapped handle with
// a pommel loop — hovers over the leading edge of a glowing rune-stone road.
// It rises and slams down in a loop; each impact throws jagged lightning out
// to both sides. The road lengthens with progress, carrying the hammer.

const MJOLNIR: [&str; 12] = [
    "......##......",
    ".....#..#.....",
    "......##......",
    "......##......",
    "......##......",
    "......##......",
    "##############",
    "##############",
    "##.########.##",
    "##.#......#.##",
    "##############",
    "##############",
];

struct MjolnirStrike;
impl ProgressStyle for MjolnirStrike {
    fn name(&self) -> &str {
        "mjolnir-strike"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Thor's hammer slams down at the edge of a growing rune-stone road, throwing jagged lightning on impact"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Rune road: a solid course with carved marks above, up to the edge.
        let edge = (ctx.eased * wi as f32).round() as i32;
        for x in 0..edge.min(wi) {
            draw::dot_i(grid, x, hi - 1);
            if x % 4 != 3 && (x / 4) % 2 == 0 || x % 4 == 1 {
                draw::dot_i(grid, x, hi - 2);
            }
        }

        // Hammer: slow rise, fast slam, short rest on the road.
        let phase = (ctx.time * 1.2).rem_euclid(1.0);
        let lift = if phase < 0.6 {
            phase / 0.6 * 2.0
        } else if phase < 0.7 {
            2.0 - (phase - 0.6) / 0.1 * 2.0
        } else {
            0.0
        };
        let hx = (edge - 7).clamp(0, wi - 14);
        let hy = hi - 14 - lift.round() as i32;
        stamp(grid, &MJOLNIR, hx, hy);

        // Lightning: two jagged bolts from the head on impact.
        let impact = phase >= 0.7;
        if impact {
            let jag = ((ctx.time * 20.0) as i32).rem_euclid(2);
            for side in [-1, 1] {
                let mut bx = if side < 0 { hx - 1 } else { hx + 14 };
                let mut by = hy + 7;
                for seg in 0..7 {
                    let dy = if (seg + jag) % 2 == 0 { -1 } else { 2 };
                    for _ in 0..3 {
                        draw::dot_i(grid, bx, by);
                        bx += side;
                    }
                    by = (by + dy).clamp(0, hi - 3);
                }
            }
        }

        // Colour: road in the palette, steel head, leather handle, bolts.
        tint_box(grid, 0, hi - 4, wi, 4, ctx.palette.sample(0.1));
        if impact {
            tint_box(grid, hx - 22, hy + 4, 58, 9, Color::rgb(200, 235, 255));
        }
        tint_box(grid, hx, hy, 14, 6, Color::rgb(170, 110, 60));
        tint_box(grid, hx, hy + 6, 14, 6, Color::rgb(200, 205, 220));
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
    let styles = progress::styles::mythology::styles();
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
