//! `penrose` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O penrose.rs && ./penrose [style-name]
//! ```

const DEFAULT_STYLE: &str = "penrose-p3";

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
    pub mod penrose {
//! Penrose / aperiodic-tiling sacred-geometry progress styles.
//!
//! Ten structurally distinct styles, each built on a real aperiodic or
//! quasi-periodic mathematical construction:
//!
//! - `penrose-p3`            — Fat & thin rhomb tiling via Robinson-triangle subdivision
//! - `penrose-p2`            — Kite & dart tiling via Robinson-triangle subdivision
//! - `sun-pattern`           — The canonical Penrose "sun" (5 kites) seed cluster
//! - `girih-tiles`           — frieze of girih decagons, each with its ten-pointed strapwork star
//! - `ammann-bars`           — five families of Ammann bars, Fibonacci L/S spacing, over rhomb vertices
//! - `debruijn-pentagrid`    — de Bruijn pentagrid: 5 families of parallel lines at 72°
//! - `decagon-fractal`       — frieze of decagons, each holding a ring of ten and a 1/φ² copy
//! - `quasicrystal-diffraction` — 10-fold Bragg-spot diffraction patterns, rings in φ ratio
//! - `pinwheel-tiling`       — Pinwheel 1:2 right-triangle substitution tiling
//! - `truchet-quasi`         — Truchet arcs on a Penrose rhomb tiling, joining into endless curves

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

const PHI: f32 = 1.618_034;

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — sunset tiling.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 146, 94);
const TINT_END: Color = Color::rgb(146, 86, 255);

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

/// All styles in the `penrose` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per aperiodic-tiling bar, ordered from
/// most iconic (P3 rhombus) to most exotic (Truchet-quasi).  All styles are
/// independent and can be used in any order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(PenroseP3)),
        Box::new(Tinted(PenroseP2)),
        Box::new(Tinted(SunPattern)),
        Box::new(Tinted(GirihTiles)),
        Box::new(Tinted(AmmannBars)),
        Box::new(Tinted(DeBruijnPentagrid)),
        Box::new(Tinted(DecagonFractal)),
        Box::new(Tinted(QuasicrystalDiffraction)),
        Box::new(Tinted(PinwheelTiling)),
        Box::new(Tinted(TruchetQuasi)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// A triangle tagged with its kind: `(kind, p, q, r)`, vertices in unit space.
type MarkedTri = (bool, [f32; 2], [f32; 2], [f32; 2]);

/// Grid center in dot-space.
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 * 0.5, dh as f32 * 0.5)
}

/// Bresenham line rasteriser. Out-of-bounds dots are silently discarded.
fn bresenham(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
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

/// Draw a polygon defined by a slice of (x,y) dot-space points (closed).
fn draw_poly(grid: &mut BrailleGrid, pts: &[(i32, i32)]) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    for i in 0..n {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % n];
        bresenham(grid, x0, y0, x1, y1);
    }
}

/// Map a unit-space point (scale ~1.0) to dot-space with center + scale.
#[inline]
fn to_dot(cx: f32, cy: f32, scale: f32, ux: f32, uy: f32, rot: f32) -> (i32, i32) {
    let rx = ux * rot.cos() - uy * rot.sin();
    let ry = ux * rot.sin() + uy * rot.cos();
    (
        (cx + rx * scale).round() as i32,
        (cy - ry * scale).round() as i32,
    )
}

// ────────────────────────────────────────────────────────────────────────────
// Shared Robinson-triangle tiling (P2 and P3)
// ────────────────────────────────────────────────────────────────────────────
//
// Both Penrose tilings come from the same seed — a decagonal wheel of ten
// 36°-apex Robinson triangles — and differ only in the subdivision rule.
// Each triangle is `(red, a, b, c)` with apex `a`; two triangles of one tile
// share an edge that is never inked, so the outlines show whole rhombs (P3)
// or whole kites and darts (P2).
//
//   P3:  red  (thin-rhomb half):  P = a + (b−a)/φ → red(c,P,b) + blue(P,c,a)
//        blue (fat-rhomb half):   Q = b + (a−b)/φ, R = b + (c−b)/φ
//                                 → blue(R,c,a) + blue(Q,R,b) + red(R,Q,a)
//   P2:  red  (kite half):        Q = a + (b−a)/φ, R = b + (c−b)/φ
//                                 → blue(R,Q,b) + red(Q,a,R) + red(c,a,R)
//        blue (dart half):        P = c + (a−c)/φ → blue(b,P,a) + red(P,c,b)

/// Seed wheel of ten red triangles, every second one mirrored.
fn robinson_wheel() -> Vec<MarkedTri> {
    (0..10)
        .map(|i| {
            let ab = (2.0 * i as f32 - 1.0) * PI / 10.0;
            let ac = (2.0 * i as f32 + 1.0) * PI / 10.0;
            let mut bv = [ab.cos(), ab.sin()];
            let mut cv = [ac.cos(), ac.sin()];
            if i % 2 == 0 {
                std::mem::swap(&mut bv, &mut cv);
            }
            (true, [0.0, 0.0], bv, cv)
        })
        .collect()
}

/// One subdivision step for the rhomb (P3) or kite-and-dart (P2) rule set.
fn robinson_deflate(tris: Vec<MarkedTri>, kites: bool) -> Vec<MarkedTri> {
    let inv = 1.0 / PHI;
    let mut out = Vec::with_capacity(tris.len() * 3);
    for (red, a, b, c) in tris {
        match (kites, red) {
            (false, true) => {
                let p = lerp2(a, b, inv);
                out.push((true, c, p, b));
                out.push((false, p, c, a));
            }
            (false, false) => {
                let q = lerp2(b, a, inv);
                let r = lerp2(b, c, inv);
                out.push((false, r, c, a));
                out.push((false, q, r, b));
                out.push((true, r, q, a));
            }
            (true, true) => {
                let q = lerp2(a, b, inv);
                let r = lerp2(b, c, inv);
                out.push((false, r, q, b));
                out.push((true, q, a, r));
                out.push((true, c, a, r));
            }
            (true, false) => {
                let p = lerp2(c, a, inv);
                out.push((false, b, p, a));
                out.push((true, p, c, b));
            }
        }
    }
    out
}

/// Draw a Penrose tiling across the whole strip: tile outlines (two edges
/// of every triangle, so the shared half-edge stays blank), with the tiles of
/// one kind (`stipple_red`) given a sparse dot texture so the two shapes read
/// apart in monochrome.  Tiles appear left → right up to `ctx.eased`; the
/// wheel turns very slowly on `ctx.time` so the tiling drifts.
fn render_robinson(grid: &mut BrailleGrid, ctx: &BarContext, kites: bool, stipple_red: bool) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw == 0 || dh == 0 {
        return;
    }
    let (cx, cy) = center(dw, dh);
    // Wheel radius covers the strip's half-diagonal; four subdivisions (three
    // for the larger kites) leave tile edges ~7-11 dots long on an 88-dot strip.
    let scale = (cx * cx + cy * cy).sqrt() + 4.0;
    let mut tris = robinson_wheel();
    for _ in 0..if kites { 3 } else { 4 } {
        tris = robinson_deflate(tris, kites);
    }
    let rot = ctx.time * 0.04;
    let front = ctx.eased.clamp(0.0, 1.0) * (dw as f32 + 6.0) - 3.0;
    let (sr, cr) = rot.sin_cos();
    let to_f = |v: [f32; 2]| -> (f32, f32) {
        (
            cx + (v[0] * cr - v[1] * sr) * scale,
            cy - (v[0] * sr + v[1] * cr) * scale,
        )
    };
    for &(red, a, b, c) in &tris {
        let (pa, pb, pc) = (to_f(a), to_f(b), to_f(c));
        let mid_x = (pa.0 + pb.0 + pc.0) / 3.0;
        if mid_x > front {
            continue;
        }
        let ri = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
        let (ia, ib, ic) = (ri(pa), ri(pb), ri(pc));
        // The two halves of a tile meet along b–c, the edge left un-inked.
        bresenham(grid, ic.0, ic.1, ia.0, ia.1);
        bresenham(grid, ia.0, ia.1, ib.0, ib.1);
        if red == stipple_red {
            // Sparse texture inside the triangle (dots on a 2×2 lattice).
            let x0 = pa.0.min(pb.0).min(pc.0).floor().max(0.0) as i32;
            let x1 = pa.0.max(pb.0).max(pc.0).ceil().min(dw as f32) as i32;
            let y0 = pa.1.min(pb.1).min(pc.1).floor().max(0.0) as i32;
            let y1 = pa.1.max(pb.1).max(pc.1).ceil().min(dh as f32) as i32;
            let edge = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
                (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0)
            };
            for y in (y0..y1).filter(|y| y % 2 == 1) {
                for x in (x0..x1).filter(|x| x % 2 == 1) {
                    let (fx, fy) = (x as f32, y as f32);
                    let d1 = edge(pa, pb, fx, fy);
                    let d2 = edge(pb, pc, fx, fy);
                    let d3 = edge(pc, pa, fx, fy);
                    let inside =
                        (d1 > 0.5 && d2 > 0.5 && d3 > 0.5) || (d1 < -0.5 && d2 < -0.5 && d3 < -0.5);
                    if inside {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
        }
    }
}

#[inline]
fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Penrose P3 — rhombus tiling
// ────────────────────────────────────────────────────────────────────────────

struct PenroseP3;
impl ProgressStyle for PenroseP3 {
    fn name(&self) -> &str {
        "penrose-p3"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose P3 rhombus tiling built by Robinson-triangle subdivision: fat \
         (72°) rhombs in outline, thin (36°) rhombs stippled, laid left to right \
         across the strip as the tiling slowly drifts"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        render_robinson(grid, ctx, false, true);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Penrose P2 — kite & dart tiling
// ────────────────────────────────────────────────────────────────────────────

struct PenroseP2;
impl ProgressStyle for PenroseP2 {
    fn name(&self) -> &str {
        "penrose-p2"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose P2 kite & dart tiling built by Robinson-triangle subdivision: \
         kites in outline, darts stippled, laid left to right across the strip \
         as the tiling slowly drifts"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        render_robinson(grid, ctx, true, false);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Sun Pattern — 5 kites around a center, expanding rings
// ────────────────────────────────────────────────────────────────────────────
//
// The canonical Penrose "sun" seed: five kites sharing a vertex at the origin.
// Progress reveals rings of tiles growing outward from the center.

struct SunPattern;
impl ProgressStyle for SunPattern {
    fn name(&self) -> &str {
        "sun-pattern"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose sun: five kites (72-72-72-144°) meeting tip-to-tip at one vertex, \
         laid kite by kite and slowly turning between two progress paths"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let scale = (dh as f32 / 2.0 - 0.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        let rot = ctx.time * 0.2 + PI / 2.0;

        // Progress path either side of the sun: solid when done, dotted ahead.
        let lit = (e * dw as f32) as i32;
        let half = scale.ceil() as i32 + 2;
        let phase = (ctx.time * 6.0) as i32;
        for x in 0..dw as i32 {
            if (x - cx as i32).abs() <= half {
                continue;
            }
            if x < lit || (x + phase).rem_euclid(3) == 0 {
                draw::dot_i(grid, x, cy.round() as i32);
            }
        }

        // Kite k: tip at the centre, axis at 72k°, wings at ±36° — every
        // outer vertex sits on the unit circle, so the five close into a
        // decagon split by five spokes (the shared wing edges).
        let kites = ((0.2 + e * 1.6).min(1.0) * 5.0).ceil() as usize;
        for k in 0..kites.min(5) {
            let th = k as f32 * 2.0 * PI / 5.0;
            let pts = [
                to_dot(cx, cy, scale, 0.0, 0.0, rot),
                to_dot(
                    cx,
                    cy,
                    scale,
                    (th - PI / 5.0).cos(),
                    (th - PI / 5.0).sin(),
                    rot,
                ),
                to_dot(cx, cy, scale, th.cos(), th.sin(), rot),
                to_dot(
                    cx,
                    cy,
                    scale,
                    (th + PI / 5.0).cos(),
                    (th + PI / 5.0).sin(),
                    rot,
                ),
            ];
            draw_poly(grid, &pts);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Girih Tiles — Islamic decagon strapwork
// ────────────────────────────────────────────────────────────────────────────
//
// Girih tiles are five polygons (decagon, pentagon, hexagon, bowtie, rhombus)
// whose edges all have the same length and whose angles are multiples of 36°.
// We draw a central decagon, then surround it with pentagons and bowties,
// revealing the strapwork interior lines as progress rises.

struct GirihTiles;
impl ProgressStyle for GirihTiles {
    fn name(&self) -> &str {
        "girih-tiles"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Islamic girih frieze: a row of edge-sharing decagon tiles, each carrying the \
         ten-pointed strapwork star whose points touch every tile edge at its midpoint, \
         meeting the neighbouring star there; tiles are laid left to right with \
         progress and a spark runs round each finished star"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Decagon with vertical left/right edges: vertices at 18° + 36°k.
        let r = (dh as f32 * 0.5 - 0.5).max(1.0);
        let pitch = (2.0 * r * (PI / 10.0).cos()).max(2.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        let n_tiles = ((dw as f32 / pitch).ceil() as usize).max(1);
        let total = 0.35 + ctx.eased.clamp(0.0, 1.0) * (n_tiles as f32 - 0.35);
        let x0 = pitch * 0.5;
        for k in 0..n_tiles {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                break;
            }
            let cx = x0 + k as f32 * pitch;
            let off = PI / 10.0;
            // Tile outline (drawn as a dotted line once the straps are in).
            let verts: Vec<(f32, f32)> = (0..10)
                .map(|i| {
                    let t = off + i as f32 * PI / 5.0;
                    (cx + r * t.cos(), cy - r * t.sin())
                })
                .collect();
            let outline = (local * 2.0).min(1.0);
            let n_edges = (outline * 10.0).ceil() as usize;
            for i in 0..n_edges.min(10) {
                let (p, q) = (verts[i], verts[(i + 1) % 10]);
                let steps = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as usize).max(1);
                for s in 0..=steps {
                    if s % 3 == 0 {
                        let g = s as f32 / steps as f32;
                        draw::dot_i(
                            grid,
                            (p.0 + (q.0 - p.0) * g).round() as i32,
                            (p.1 + (q.1 - p.1) * g).round() as i32,
                        );
                    }
                }
            }
            // Strapwork: the ten-pointed star whose points sit on the edge
            // midpoints (where the straps cross into the neighbouring tile).
            let rm = r * (PI / 10.0).cos();
            let star: Vec<(f32, f32)> = (0..20)
                .map(|i| {
                    let t = off + PI / 10.0 + i as f32 * PI / 10.0;
                    let rr = if i % 2 == 0 { rm } else { rm * 0.5 };
                    (cx + rr * t.cos(), cy - rr * t.sin())
                })
                .collect();
            let straps = ((local - 0.3) / 0.7).clamp(0.0, 1.0) * 20.0;
            let full = straps.floor() as usize;
            for i in 0..20usize {
                let (p, q) = (star[i], star[(i + 1) % 20]);
                let g = match i.cmp(&full) {
                    std::cmp::Ordering::Less => 1.0,
                    std::cmp::Ordering::Equal => straps - full as f32,
                    std::cmp::Ordering::Greater => break,
                };
                bresenham(
                    grid,
                    p.0.round() as i32,
                    p.1.round() as i32,
                    (p.0 + (q.0 - p.0) * g).round() as i32,
                    (p.1 + (q.1 - p.1) * g).round() as i32,
                );
            }
            // A spark runs round the strap star once the tile is complete.
            if local >= 1.0 {
                let u = (ctx.time * 0.4 + k as f32 * 0.31).fract() * 20.0;
                let i = (u.floor() as usize).min(19);
                let g = u - i as f32;
                let (p, q) = (star[i], star[(i + 1) % 20]);
                let (x, y) = (
                    (p.0 + (q.0 - p.0) * g).round() as i32,
                    (p.1 + (q.1 - p.1) * g).round() as i32,
                );
                for (ddx, ddy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                    draw::dot_i(grid, x + ddx, y + ddy);
                }
            }
        }
        Ok(())
    }
}

/// Compute vertices of a regular n-gon centered at (cx,cy) in dot-space.
fn regular_ngon(n: usize, cx: f32, cy: f32, radius: f32, offset: f32) -> Vec<(i32, i32)> {
    (0..n)
        .map(|k| {
            let a = 2.0 * PI * k as f32 / n as f32 + offset;
            (
                (cx + radius * a.cos()).round() as i32,
                (cy - radius * a.sin()).round() as i32,
            )
        })
        .collect()
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Ammann Bars — the hidden quasiperiodic grid on a P3 rhombus tiling
// ────────────────────────────────────────────────────────────────────────────
//
// Every Penrose P3 tiling carries an "Ammann bar" decoration: each rhombus
// gets one stripe across it, and the stripes form 5 families of parallel lines
// at angles 0°, 72°, 144°, 216°, 288°.  The spacings within each family are
// quasiperiodic (long L and short S with L/S = PHI).
//
// We approximate this by drawing 5 families of parallel quasiperiodic lines.
// Each family k has angle k*36° and lines at positions …, 0, L, L+S, 2L+S, 2L+2S, …
// where L = PHI and S = 1.

struct AmmannBars;
impl ProgressStyle for AmmannBars {
    fn name(&self) -> &str {
        "ammann-bars"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Ammann bars: the five families of parallel bars that run through every \
         Penrose rhomb tiling, at 72° to one another, each spaced in the \
         never-repeating Fibonacci order of long and short gaps (L/S = φ); a faint \
         rhomb tiling lies beneath, and the families sweep in one after another \
         with progress, sliding slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        // Faint rhomb tiling: vertices of the P3 tiling only.
        let scale = (cx * cx + cy * cy).sqrt() + 4.0;
        let mut tris = robinson_wheel();
        for _ in 0..4 {
            tris = robinson_deflate(tris, false);
        }
        for &(_red, a, _b, _c) in &tris {
            draw::dot_i(
                grid,
                (cx + a[0] * scale).round() as i32,
                (cy - a[1] * scale).round() as i32,
            );
        }
        // Fibonacci word of long/short gaps.
        let s_gap = 6.0f32;
        let l_gap = s_gap * PHI;
        let half = (dw as f32 + dh as f32) * 0.5 + l_gap;
        let fams = 0.3 + ctx.eased.clamp(0.0, 1.0) * 4.7;
        for fam in 0..5usize {
            let reveal = (fams - fam as f32).clamp(0.0, 1.0);
            if reveal <= 0.0 {
                break;
            }
            let th = PI / 2.0 + fam as f32 * 2.0 * PI / 5.0;
            let (nx, ny) = (th.cos(), -th.sin()); // normal (screen space)
            let (lx, ly) = (-ny, nx); // bar direction
            let x_lim = reveal * dw as f32;
            // Each family slides slowly along its normal; the gap sequence is
            // anchored to the first bar so the Fibonacci word travels intact.
            let slide = (ctx.time * 1.5 + fam as f32 * 1.3) % (l_gap + s_gap);
            let mut pos = -half + slide;
            for n in 1..=200usize {
                if pos >= half {
                    break;
                }
                let base = (cx + nx * pos, cy + ny * pos);
                let reach = half * 1.5;
                let steps = (reach * 2.0) as usize;
                for i in 0..=steps {
                    let t = -reach + i as f32;
                    let x = base.0 + lx * t;
                    let y = base.1 + ly * t;
                    if x >= 0.0 && x <= x_lim && y >= 0.0 && y < dh as f32 {
                        draw::dot_i(grid, x.round() as i32, y.round() as i32);
                    }
                }
                // n-th letter of the Fibonacci word: L when ⌊(n+1)/φ⌋ − ⌊n/φ⌋ = 1.
                let is_l = ((n + 1) as f32 / PHI).floor() - (n as f32 / PHI).floor() >= 1.0;
                pos += if is_l { l_gap } else { s_gap };
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. De Bruijn Pentagrid — 5 families of parallel lines → dual Penrose tiling
// ────────────────────────────────────────────────────────────────────────────
//
// de Bruijn (1981): a Penrose P3 tiling is dual to a pentagrid — 5 families of
// equidistant parallel lines at angles 72°·k, each with offset γ_k ≈ arbitrary.
//
// We draw the five families, then animate the intersection points to reveal
// the dual rhombus vertices.  The dual step is: each intersection of family j
// and family k at crossing index (m,n) maps to a rhombus vertex.
//
// For progress we simply reveal the five line families one at a time, and
// overlay the computed dual vertices with `ctx.eased`.

struct DeBruijnPentagrid;
impl ProgressStyle for DeBruijnPentagrid {
    fn name(&self) -> &str {
        "debruijn-pentagrid"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "de Bruijn pentagrid: five families of evenly spaced parallel lines at 72° to \
         one another, each with its own offset, whose dual is a Penrose tiling; the \
         families are laid one after another across the strip with progress and the \
         grid drifts slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let spacing = 9.0f32;
        // Shifts γ_k (sum 0, generic) as in de Bruijn's construction.
        let gammas: [f32; 5] = [0.2, -0.35, 0.1, 0.4, -0.35];
        // Families reveal one after another; the first one is always partly in.
        let fams = 0.25 + ctx.eased.clamp(0.0, 1.0) * 4.75;
        let drift = ctx.time * 0.25;
        let cx = dw as f32 * 0.5;
        let cy = dh as f32 * 0.5;
        for (fam, &gamma) in gammas.iter().enumerate() {
            let reveal = (fams - fam as f32).clamp(0.0, 1.0);
            if reveal <= 0.0 {
                break;
            }
            // Family k: lines x·cos θ + y·sin θ = (n + γ_k)·spacing, θ = 90° + 72°k.
            let th = PI / 2.0 + fam as f32 * 2.0 * PI / 5.0;
            let (c, sn) = (th.cos(), th.sin());
            let x_lim = reveal * dw as f32;
            for y in 0..dh {
                for x in 0..dw {
                    if x as f32 > x_lim {
                        break;
                    }
                    let d = ((x as f32 - cx) * c + (y as f32 - cy) * sn) / spacing - gamma - drift;
                    let f = d - d.round();
                    // ~half-dot line width in dot units, scaled by the slope.
                    if f.abs() * spacing < 0.5 {
                        draw::dot(grid, x, y);
                    }
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Decagon Fractal — recursive decagon infill
// ────────────────────────────────────────────────────────────────────────────
//
// A decagon subdivides into 10 smaller decagons + connecting pentagons (roughly).
// We approximate with: one large decagon, then ring of 10 smaller at radius
// 1+sin(18°) * sub_r, then sub-sub-decagons, up to depth 3.
// Depth capped at 3 for cost safety.

struct DecagonFractal;
impl ProgressStyle for DecagonFractal {
    fn name(&self) -> &str {
        "decagon-fractal"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Decagon fractal: a frieze of self-similar decagons — each holds a ring of ten \
         satellites and a 1/φ²-scaled copy of itself at its heart, which repeats the \
         rule; flakes unfold left to right with progress, rings counter-turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let panels = (dw / 18).max(1);
        let pw = dw as f32 / panels as f32;
        let big = (dh as f32 * 0.5 - 0.5).min(pw * 0.5 - 0.5).max(1.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        let total = 0.3 + ctx.eased.clamp(0.0, 1.0) * (panels as f32 - 0.3);
        let shrink = 1.0 / (1.0 + PHI); // 1/φ²
        for k in 0..panels {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                break;
            }
            let cx = pw * (k as f32 + 0.5) - 0.5;
            let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
            let rot = PI / 2.0;
            // Self-similar rule: a decagon = outline + ring of ten satellites
            // near its rim + a 1/φ²-scaled copy at its heart, and the copy
            // obeys the same rule.
            let spin = rot + dir * ctx.time * 0.5;
            let mut level_r = big;
            let mut budget = local * 2.0;
            for level in 0..2 {
                if budget <= 0.0 || level_r < 1.0 {
                    break;
                }
                let part = budget.min(1.0);
                budget -= 1.0;
                draw_poly(grid, &regular_ngon(10, cx, cy, level_r, rot));
                let child = level_r * 0.16;
                let dist = level_r * 0.72;
                let n = (part * 10.0).ceil() as usize;
                let lspin = if level % 2 == 0 {
                    spin
                } else {
                    2.0 * rot - spin
                };
                for j in 0..n.min(10) {
                    let t = lspin + j as f32 * PI / 5.0;
                    let (px, py) = (cx + dist * t.cos(), cy - dist * t.sin());
                    if child >= 1.2 {
                        draw_poly(grid, &regular_ngon(10, px, py, child, rot));
                    } else {
                        draw::dot_i(grid, px.round() as i32, py.round() as i32);
                    }
                }
                level_r *= shrink;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Quasicrystal Diffraction — 5 plane waves at 72° → 10-fold interference
// ────────────────────────────────────────────────────────────────────────────
//
// The quasicrystal interference pattern is computed as:
//   f(x,y) = Σ_{k=0}^{4} cos(2π · (x·cos(k·72°) + y·sin(k·72°)) · freq)
// Normalise to [0,1] and threshold at eased to reveal the bright regions.
// Braille dot space is sampled directly — very fast per dot.

struct QuasicrystalDiffraction;
impl ProgressStyle for QuasicrystalDiffraction {
    fn name(&self) -> &str {
        "quasicrystal-diffraction"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Quasicrystal diffraction: a row of 10-fold Bragg-spot patterns (bright centre, \
         two rings of ten spots at radii in φ ratio, the outer ring turned 18°) — the \
         forbidden symmetry that won Shechtman the Nobel prize; patterns expose \
         left to right with progress and their spots twinkle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let panels = (dw / 18).max(1);
        let pw = dw as f32 / panels as f32;
        let r_max = (dh as f32 * 0.5 - 0.5).min(pw * 0.5 - 0.5).max(1.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        // Ring radii in φ ratio, outermost touching the strip edge.
        let phi = 1.618_034_f32;
        let radii = [r_max / phi, r_max];
        let total = 0.3 + ctx.eased.clamp(0.0, 1.0) * (panels as f32 - 0.3);
        let blob = |grid: &mut BrailleGrid, x: f32, y: f32, big: bool| {
            let (xi, yi) = (x.round() as i32, y.round() as i32);
            draw::dot_i(grid, xi, yi);
            if big {
                for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    draw::dot_i(grid, xi + ddx, yi + ddy);
                }
            }
        };
        for k in 0..panels {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                break;
            }
            let cx = pw * (k as f32 + 0.5) - 0.5;
            // Central (000) beam — always strong.
            blob(grid, cx, cy, true);
            for (ri, &rr) in radii.iter().enumerate() {
                let need = ri as f32 * 0.5;
                if local <= need {
                    break;
                }
                let frac = ((local - need) * 2.0).min(1.0);
                let n_spots = (frac * 10.0).ceil() as usize;
                let off = if ri % 2 == 0 {
                    PI / 2.0
                } else {
                    PI / 2.0 + PI / 10.0
                };
                for j in 0..n_spots.min(10) {
                    let t = off + j as f32 * PI / 5.0;
                    let x = cx + rr * t.cos();
                    let y = cy - rr * t.sin();
                    // Inner rings are brighter; spots twinkle on time.
                    let tw = (ctx.time * 3.0 + j as f32 * 1.7 + ri as f32 * 2.3 + k as f32).sin();
                    let big = ri == 1 && tw > -0.5;
                    blob(grid, x, y, big);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Pinwheel Tiling — 1:2 right-triangle substitution
// ────────────────────────────────────────────────────────────────────────────
//
// The pinwheel tiling (Conway & Radin 1994) is based on a right triangle
// with legs 1 and 2 and hypotenuse √5.  One triangle substitutes into 5,
// and with each generation the triangles appear at every possible rotation
// (dense in SO(2)), making it rotationally aperiodic.
//
// Deflation: a right triangle (legs a=1, b=2) splits into 5 smaller copies.
// Vertices: P (right angle), Q (on short side), R (far end of long side).
// We cap at depth 4 and cost 2000 triangles.

struct PinwheelTiling;
impl ProgressStyle for PinwheelTiling {
    fn name(&self) -> &str {
        "pinwheel-tiling"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Pinwheel tiling: a 1:2 right triangle substitutes into 5 smaller copies \
         at irrational rotations, filling the plane with triangles at every angle; \
         deflation generations bloom outward as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // The whole canvas is tiled with 1:2 right triangles: a lattice of
        // 2h×h rectangles, each split along its diagonal into two seeds.
        // As progress rises every seed deflates into 5, then 25 copies —
        // exactly the pinwheel substitution — sweeping left to right.
        type Tri = [[f32; 2]; 3]; // [P (right angle), Q (short leg), R (long leg)]
        let rh = (dh as f32 - 1.0).max(2.0); // rectangle height (short leg)
        let rw = rh * 2.0; // rectangle width (long leg)
        let cols = ((dw as f32 / rw).ceil() as usize + 1).min(64);
        let period = cols as f32 * rw;

        // The tiling drifts one full lattice period per 4 s loop (seamless).
        let drift = (ctx.time * 0.25).fract() * period;

        let mut seeds: Vec<Tri> = Vec::with_capacity(cols * 2);
        for col in 0..cols {
            let x0 = col as f32 * rw;
            // Two complementary 1:2 triangles sharing the diagonal.
            seeds.push([[x0, 0.0], [x0, rh], [x0 + rw, 0.0]]);
            seeds.push([[x0 + rw, rh], [x0 + rw, 0.0], [x0, rh]]);
        }

        // Draw a triangle at the drifting offset (twice, so the wrap-around
        // seam at the right edge is filled by the copy from the left).
        let draw_tri = |grid: &mut BrailleGrid, tri: &Tri| {
            for wrap in [0.0f32, -period] {
                let pts: Vec<(i32, i32)> = tri
                    .iter()
                    .map(|v| ((v[0] + drift + wrap).round() as i32, (v[1]).round() as i32))
                    .collect();
                if pts.iter().all(|&(x, _)| x < 0) || pts.iter().all(|&(x, _)| x >= dw as i32) {
                    continue;
                }
                draw_poly(grid, &pts);
            }
        };

        // The seed lattice is always visible; deflation generations bloom
        // seed by seed as eased advances (two full passes: depth 1, then 2).
        for tri in &seeds {
            draw_tri(grid, tri);
        }
        let n_seeds = seeds.len();
        let phase = ctx.eased * 2.0;
        for (i, seed) in seeds.iter().enumerate() {
            let gen1_on = phase * n_seeds as f32 >= (i + 1) as f32;
            let gen2_on = (phase - 1.0) * n_seeds as f32 >= (i + 1) as f32;
            if !gen1_on {
                continue;
            }
            let children = pinwheel_deflate(vec![*seed]);
            for tri in &children {
                draw_tri(grid, tri);
            }
            if gen2_on {
                for tri in &pinwheel_deflate(children) {
                    draw_tri(grid, tri);
                }
            }
        }
        Ok(())
    }
}

type Tri2 = [[f32; 2]; 3];

fn pinwheel_deflate(tris: Vec<Tri2>) -> Vec<Tri2> {
    let mut out = Vec::with_capacity(tris.len() * 5);
    for tri in tris {
        let [p, q, r] = tri;
        // Pinwheel substitution: 5 children.
        // Standard pinwheel: right angle at P, short leg PQ (len 1), long leg PR (len 2).
        // Sub-triangle hypotenuse = 1/√5 of parent hypotenuse.
        //
        // Key points (per Conway-Radin):
        //   M  = midpoint(P, R)           (midpoint of long leg)
        //   A  = P + (1/5)*(Q-P)          (1/5 along PQ from P)
        //   B  = P + (2/5)*(Q-P)          (2/5 along PQ from P)
        //   C  = midpoint(Q, R)
        //   D  = M + (1/2)*(Q - M)        ~ midpoint(M, Q)
        //
        // 5 children (approximate but structurally correct):
        let m = lerp2(p, r, 0.5);
        let a = lerp2(p, q, 0.2);
        let b = lerp2(p, q, 0.4);
        let c = lerp2(q, r, 0.5);
        let d = lerp2(m, q, 0.5);
        out.push([p, a, m]);
        out.push([a, b, d]);
        out.push([b, q, c]);
        out.push([d, c, m]);
        out.push([m, c, r]);
    }
    out
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Truchet-Quasi — Truchet arcs on a quasiperiodic rhombus lattice
// ────────────────────────────────────────────────────────────────────────────
//
// Classic Truchet tiles place quarter-circle arcs in squares, two orientations.
// We place arcs inside the rhombi of a Penrose-like rhombus grid (fat and thin),
// with orientation chosen by a deterministic pseudo-random rule seeded from
// the rhombus index so the pattern never repeats.
//
// Since we don't have a full deflation here, we generate a flat grid of
// rhombi using a pentagrid projection approach (approximate) and draw arcs
// inside each.

struct TruchetQuasi;
impl ProgressStyle for TruchetQuasi {
    fn name(&self) -> &str {
        "truchet-quasi"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Truchet-quasi: Truchet-style arcs on a Penrose rhomb tiling — every tile \
         carries an arc round its corner, and the arcs join across tile edges into \
         unbroken curves that never repeat; tiles are laid left to right with \
         progress, edges faint, and the tiling drifts slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = (cx * cx + cy * cy).sqrt() + 4.0;
        let mut tris = robinson_wheel();
        for _ in 0..4 {
            tris = robinson_deflate(tris, false);
        }
        let rot = ctx.time * 0.04;
        let front = 6.0 + ctx.eased.clamp(0.0, 1.0) * (dw as f32 + 2.0) - 3.0;
        let (sr, cr) = rot.sin_cos();
        let to_f = |v: [f32; 2]| -> (f32, f32) {
            (
                cx + (v[0] * cr - v[1] * sr) * scale,
                cy - (v[0] * sr + v[1] * cr) * scale,
            )
        };
        for &(_red, a, b, c) in &tris {
            let (pa, pb, pc) = (to_f(a), to_f(b), to_f(c));
            let mid_x = (pa.0 + pb.0 + pc.0) / 3.0;
            if mid_x > front || mid_x < -8.0 || mid_x > dw as f32 + 8.0 {
                continue;
            }
            // Faint tile edges: every third dot of the two outer edges.
            for (p, q) in [(pa, pb), (pa, pc)] {
                let n = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as usize).max(1);
                for i in (0..=n).step_by(3) {
                    let g = i as f32 / n as f32;
                    draw::dot_i(
                        grid,
                        (p.0 + (q.0 - p.0) * g).round() as i32,
                        (p.1 + (q.1 - p.1) * g).round() as i32,
                    );
                }
            }
            // Arc round the apex `a`, radius half the edge, from edge a–b to
            // edge a–c; the two halves of each rhomb complete one arc.
            let rad = 0.5 * ((pb.0 - pa.0).powi(2) + (pb.1 - pa.1).powi(2)).sqrt();
            let t0 = (pb.1 - pa.1).atan2(pb.0 - pa.0);
            let t1 = (pc.1 - pa.1).atan2(pc.0 - pa.0);
            let mut sweep = t1 - t0;
            // atan2 differences lie in (−2π, 2π): one wrap picks the short arc.
            if sweep > PI {
                sweep -= 2.0 * PI;
            } else if sweep < -PI {
                sweep += 2.0 * PI;
            }
            let steps = ((rad * sweep.abs() * 1.5).ceil() as usize).max(2);
            for i in 0..=steps {
                let t = t0 + sweep * i as f32 / steps as f32;
                draw::dot_i(
                    grid,
                    (pa.0 + rad * t.cos()).round() as i32,
                    (pa.1 + rad * t.sin()).round() as i32,
                );
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
    let styles = progress::styles::penrose::styles();
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
