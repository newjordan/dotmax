//! `fractal` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O fractal.rs && ./fractal [style-name]
//! ```

const DEFAULT_STYLE: &str = "mandelbrot-escape";

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
    pub mod fractal {
//! Fractal-themed progress bars — hard mathematics rendered in braille dots.
//!
//! Each style implements a distinct fractal or iterated-function-system formula,
//! mapping `ctx.eased` to a reveal / iteration parameter and `ctx.time` to
//! continuous animation (zoom, rotation, c-parameter drift). All coordinate
//! arithmetic is done in `f32`/`i32` and committed via `draw::dot_i` so that
//! negative values from fractal math never cause panics at small grid sizes.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic hash (no external crates)
// ---------------------------------------------------------------------------

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `fractal` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per style, in display order. Each
/// style maps `ctx.eased` to a mathematical reveal parameter (iteration depth,
/// point count, recursion level) and `ctx.time` to continuous animation.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(MandelbrotEscape),
        Box::new(JuliaSet),
        Box::new(SierpinskiTriangle),
        Box::new(KochCurve),
        Box::new(BarnsleyFern),
        Box::new(DragonCurve),
        Box::new(SierpinskiCarpet),
        Box::new(BurningShip),
        Box::new(PythagorasTree),
        Box::new(NewtonFractal),
        Box::new(CantorDust),
        Box::new(LyapunovBar),
    ]
}

// ---------------------------------------------------------------------------
// 1. Mandelbrot escape-time
// ---------------------------------------------------------------------------

struct MandelbrotEscape;
impl ProgressStyle for MandelbrotEscape {
    fn name(&self) -> &str {
        "mandelbrot-escape"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "The Mandelbrot set's cardioid-and-bulb silhouette at the bar's centre, with its \
         escape-time equipotential lines spreading outward to fill the bar as progress rises \
         and rippling with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Uniform scale: the set's body (|Im| ≤ 1.05) fits the bar height.
        let s = (dh as f32 / 2.1).max(1.0);
        let (ox, oy) = (dw as f32 * 0.5, dh as f32 * 0.5);
        let (cw, ch) = grid.dimensions();
        let mut inside = vec![0u8; cw * ch];
        // Smooth escape count μ per dot; NaN marks the set's interior.
        let mut field = vec![f32::NAN; dw * dh];

        for py in 0..dh {
            for px in 0..dw {
                let cr = -0.55 + (px as f32 + 0.5 - ox) / s;
                let ci = (py as f32 + 0.5 - oy) / s;
                let (mut zr, mut zi) = (0.0f32, 0.0f32);
                let mut n = 0u32;
                while n < 40 && zr * zr + zi * zi <= 256.0 {
                    let t = zr * zr - zi * zi + cr;
                    zi = 2.0 * zr * zi + ci;
                    zr = t;
                    n += 1;
                }
                if n >= 40 {
                    draw::dot(grid, px, py);
                    inside[(py / 4) * cw + px / 2] += 1;
                    continue;
                }
                let lz = (zr * zr + zi * zi).max(1.0001).ln() * 0.5;
                let mu = n as f32 + 1.0 - lz.max(1e-4).ln() / std::f32::consts::LN_2;
                field[py * dw + px] = mu;
            }
        }

        // Equipotential lines = where the banded μ steps between neighbours.
        // They spread from the set (high μ) outward (low μ) as progress rises.
        let mu_min = 7.0 * (1.0 - ctx.eased) - 0.5;
        let band = |mu: f32| ((mu + 1.0).ln() * 4.5 - ctx.time * 0.8).floor() as i32;
        for py in 0..dh {
            for px in 0..dw {
                let mu = field[py * dw + px];
                if mu.is_nan() || mu < mu_min {
                    continue;
                }
                let b = band(mu);
                let right = if px + 1 < dw {
                    field[py * dw + px + 1]
                } else {
                    mu
                };
                let down = if py + 1 < dh {
                    field[(py + 1) * dw + px]
                } else {
                    mu
                };
                let edge = |m: f32| !m.is_nan() && band(m) != b;
                if edge(right) || edge(down) {
                    draw::dot(grid, px, py);
                }
            }
        }

        // One colour per cell: the set itself glows, equipotentials fade outward.
        grid.enable_color_support();
        let half = cw as f32 * 0.5;
        for cy in 0..ch {
            for cx in 0..cw {
                let c = if inside[cy * cw + cx] >= 3 {
                    ctx.palette.start
                } else {
                    let d = ((cx as f32 + 0.5 - half).abs() / half.max(1.0)).min(1.0);
                    ctx.palette.sample(0.25 + d * 0.75)
                };
                let _ = grid.set_cell_color(cx, cy, c);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Julia sets  z → z² + c — basilica, rabbit, Siegel disk
// ---------------------------------------------------------------------------

struct JuliaSet;
impl ProgressStyle for JuliaSet {
    fn name(&self) -> &str {
        "julia-set"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Three filled Julia sets side by side — the basilica (c=−1), Douady's rabbit and the \
         spiralling Siegel disk — each sharpening from a round blob into its true shape as \
         progress raises its iteration depth; c trembles gently with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        // (c, half-width, half-height) of each set's bounding box.
        const SETS: [(f32, f32, f32, f32); 3] = [
            (-1.0, 0.0, 1.62, 0.78),
            (-0.123, 0.745, 1.25, 1.2),
            (-0.391, -0.587, 1.25, 1.2),
        ];
        let n = if dw >= dh * 4 { 3 } else { 1 };
        let pw = dw as f32 / n as f32;
        let wob = 0.008 * (ctx.time * 1.3).sin();
        let (cw, ch) = grid.dimensions();
        for (k, &(cr, ci, hw, hh)) in SETS.iter().take(n).enumerate() {
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let s = ((pw - 2.0) / (2.0 * hw)).min(dh as f32 / (2.0 * hh));
            let ox = pw * (k as f32 + 0.5);
            let oy = dh as f32 * 0.5;
            if local <= 0.0 {
                // Waiting panel: just its centre point, c's critical orbit start.
                draw::dot_i(grid, ox as i32, oy as i32);
                continue;
            }
            // Shallow depths give a blob; by ~30 iterations the set has its true outline.
            let max_iter = (2.0 + local * 30.0) as u32;
            let (cr, ci) = (cr + wob, ci + wob * 0.5);
            let x_lo = (ox - hw * s).max(0.0) as usize;
            let x_hi = ((ox + hw * s) as usize).min(dw);
            for py in 0..dh {
                for px in x_lo..x_hi {
                    let mut zr = (px as f32 + 0.5 - ox) / s;
                    let mut zi = (py as f32 + 0.5 - oy) / s;
                    let mut it = 0u32;
                    while it < max_iter && zr * zr + zi * zi <= 4.0 {
                        let t = zr * zr - zi * zi + cr;
                        zi = 2.0 * zr * zi + ci;
                        zr = t;
                        it += 1;
                    }
                    if it >= max_iter {
                        draw::dot(grid, px, py);
                    }
                }
            }
            // Panel colour: finished sets take the palette's end, growing ones its start.
            let c = ctx.palette.sample(0.2 + local * 0.8);
            let c0 = (pw * k as f32 / 2.0) as usize;
            let c1 = ((pw * (k + 1) as f32 / 2.0) as usize).min(cw);
            for cy in 0..ch {
                for cx in c0..c1 {
                    draw::tint_row(grid, cy, cx, cx, c);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Sierpinski triangle — Pascal's triangle mod 2
// ---------------------------------------------------------------------------

struct SierpinskiTriangle;
impl ProgressStyle for SierpinskiTriangle {
    fn name(&self) -> &str {
        "sierpinski-triangle"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "A row of Sierpinski triangles built from Pascal's triangle mod 2: each starts as a \
         solid triangle and subdivides level by level, left to right, as progress rises; a \
         chaos-game walker hops between vertices with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let rows = dh as i32;
        // Levels available at this height: 16 rows → 4 halvings.
        let dmax = 32 - (dh.max(2) as u32 - 1).leading_zeros();
        let base = 2 * rows - 1;
        let n = ((dw as i32 + base / 2) / base.max(1)).max(1);
        let pitch = dw as f32 / n as f32;
        let q = ctx.eased * n as f32;

        for i in 0..n {
            let apex = ((i as f32 + 0.5) * pitch) as i32;
            let local = (q - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                // Not started yet: just the outline of the triangle to come.
                for r in 0..rows {
                    draw::dot_i(grid, apex - r, r);
                    draw::dot_i(grid, apex + r, r);
                }
                for k in 0..rows {
                    draw::dot_i(grid, apex - rows + 1 + 2 * k, rows - 1);
                }
                continue;
            }
            // Depth 0 = solid triangle, dmax = full Pascal-mod-2 detail.
            let depth = ((local * (dmax as f32 + 0.999)) as u32).min(dmax);
            let sh = dmax - depth;
            for r in 0..rows {
                for k in 0..=r {
                    let (kb, rb) = ((k as u32) >> sh, (r as u32) >> sh);
                    if kb & (rb - kb) == 0 {
                        draw::dot_i(grid, apex - r + 2 * k, r);
                    }
                }
            }
        }

        // Chaos-game walker: midpoint hops toward random vertices of the active triangle.
        let active = (q.floor() as i32).clamp(0, n - 1);
        let apex = (active as f32 + 0.5) * pitch;
        let verts = [
            (apex, 0.0),
            (apex - (rows - 1) as f32, (rows - 1) as f32),
            (apex + (rows - 1) as f32, (rows - 1) as f32),
        ];
        let seed = (ctx.time * 10.0).max(0.0) as u32;
        let (mut wx, mut wy) = (apex, rows as f32 * 0.6);
        for j in 0..12u32 {
            let v = verts[(hash(seed.wrapping_add(j)) % 3) as usize];
            wx = (wx + v.0) * 0.5;
            wy = (wy + v.1) * 0.5;
        }

        grid.enable_color_support();
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            for cx in 0..cw {
                let x = cx as f32 * 2.0 + 1.0;
                let tri = ((x / pitch).floor() as i32).clamp(0, n - 1);
                let done = q - tri as f32;
                let c = if done <= 0.0 {
                    ctx.palette.track
                } else {
                    ctx.palette
                        .sample((cy as f32 / ch.max(1) as f32 + tri as f32 * 0.3).fract())
                };
                let _ = grid.set_cell_color(cx, cy, c);
            }
        }
        draw::dot_i(grid, wx as i32, wy as i32);
        draw::dot_i(grid, wx as i32 + 1, wy as i32);
        let _ = grid.set_cell_color(
            (wx.max(0.0) as usize) / 2,
            (wy.max(0.0) as usize) / 4,
            crate::Color::rgb(255, 255, 255),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Koch curve — recursive middle-third bumps along the bar
// ---------------------------------------------------------------------------

struct KochCurve;

/// Koch curve vertices from `a` to `b` at recursion `depth`, bump on the
/// screen-up side. Appends every vertex after `a`.
fn koch_points(a: (f32, f32), b: (f32, f32), depth: u32, pts: &mut Vec<(f32, f32)>) {
    if depth == 0 {
        pts.push(b);
        return;
    }
    let (dx, dy) = ((b.0 - a.0) / 3.0, (b.1 - a.1) / 3.0);
    let p1 = (a.0 + dx, a.1 + dy);
    let p3 = (a.0 + 2.0 * dx, a.1 + 2.0 * dy);
    // Rotate the middle third by −60° (screen y grows downward → bump goes up).
    let p2 = (
        p1.0 + dx * 0.5 + dy * 0.866_025,
        p1.1 + dy * 0.5 - dx * 0.866_025,
    );
    koch_points(a, p1, depth - 1, pts);
    koch_points(p1, p2, depth - 1, pts);
    koch_points(p2, p3, depth - 1, pts);
    koch_points(p3, b, depth - 1, pts);
}

impl ProgressStyle for KochCurve {
    fn name(&self) -> &str {
        "koch-curve"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Koch curve laid along the bar: each middle third grows a triangular bump, again and \
         again — progress rewrites the line left to right at each finer level (straight line → \
         one bump → 4 → 16 → 64), and a spark runs along the finished curve with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh == 0 {
            return Ok(());
        }
        // Each Koch span is √3/6 ≈ 0.29 as tall as it is long; use as many
        // side-by-side spans as needed to fit the bar's height.
        let max_len = ((dh as f32 - 1.0) / 0.289).max(2.0);
        let spans = ((dw as f32 / max_len).ceil() as usize).clamp(1, 16);
        let span = (dw as f32 - 1.0) / spans as f32;
        let base = dh as f32 - 1.0;

        let levels = 5.0f32;
        let stage = (ctx.eased.clamp(0.0, 1.0) * levels).min(levels - 1e-3);
        let depth = stage as u32;
        let split = stage.fract() * (dw as f32);

        let mut old = Vec::new();
        let mut new = Vec::new();
        for s in 0..spans {
            let a = (s as f32 * span, base);
            let b = ((s + 1) as f32 * span, base);
            old.push(a);
            koch_points(a, b, depth, &mut old);
            new.push(a);
            koch_points(a, b, (depth + 1).min(5), &mut new);
        }
        // Spark: a short gap that travels along the curve with time.
        let spark = ((ctx.time.max(0.0) * 0.25).fract() * dw as f32) as i32;
        let mut seg = |p: (f32, f32), q: (f32, f32)| {
            let steps = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as i32).clamp(1, 256);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let x = (p.0 + (q.0 - p.0) * t).round() as i32;
                let y = (p.1 + (q.1 - p.1) * t).round() as i32;
                if (x - spark).abs() > 1 {
                    draw::dot_i(grid, x, y);
                }
            }
        };
        for w in new.windows(2) {
            if w[0].0.max(w[1].0) <= split {
                seg(w[0], w[1]);
            }
        }
        for w in old.windows(2) {
            if w[0].0.min(w[1].0) >= split {
                seg(w[0], w[1]);
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy_cell in 0..ch {
            let t = cy_cell as f32 / ch.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                cw.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Barnsley fern — IFS with 4 affine maps
// ---------------------------------------------------------------------------

struct BarnsleyFern;
impl ProgressStyle for BarnsleyFern {
    fn name(&self) -> &str {
        "barnsley-fern"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Barnsley's four-map IFS fern laid along the bar: the frond unfurls from its stem \
         toward the tip as progress rises, with alternating leaflets, and sways with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        // Fern spans x ∈ [-2.2, 2.7], y ∈ [0, 10]; lay its axis (y) along the bar.
        let reach = 0.08 + ctx.eased * 0.92;
        let sway = (ctx.time * 0.9).sin() * 0.35;
        let mut hits = vec![0u8; dw * dh];
        let (mut x, mut y) = (0.0f32, 0.0f32);
        let count = 8_000u32;
        for i in 0..count {
            let r = hash(i.wrapping_mul(2_654_435_761) ^ 0x5bd1) % 100;
            (x, y) = match r {
                0 => (0.0, 0.16 * y),
                1..=85 => (0.85 * x + 0.04 * y, -0.04 * x + 0.85 * y + 1.6),
                86..=92 => (0.2 * x - 0.26 * y, 0.23 * x + 0.22 * y + 1.6),
                _ => (-0.15 * x + 0.28 * y, 0.26 * x + 0.24 * y + 0.44),
            };
            let along = y / 10.0;
            if i < 20 || along > reach {
                continue;
            }
            // Bend grows toward the tip so the frond sways, not slides.
            let bend = x + sway * along * along * 4.0;
            let px = (along * (dw - 1) as f32) as usize;
            let py = ((bend + 2.4) / 5.4 * (dh - 1) as f32).round();
            if py >= 0.0 && (py as usize) < dh && px < dw {
                let h = &mut hits[py as usize * dw + px];
                *h = h.saturating_add(1);
            }
        }
        // Threshold per column against that column's peak: the tip collects far
        // more points than the base, and this keeps leaflets distinct along the frond.
        for px in 0..dw {
            let peak = (0..dh).map(|py| hits[py * dw + px]).max().unwrap_or(0);
            let thr = (u16::from(peak) * 3 / 10).max(1);
            for py in 0..dh {
                if u16::from(hits[py * dw + px]) >= thr {
                    draw::dot(grid, px, py);
                }
            }
        }
        // Stem-to-tip green-ish gradient from the palette.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Dragon curve — paper-folding L-system
// ---------------------------------------------------------------------------

struct DragonCurve;

/// Straight dot line between two signed dot coordinates (bounds-safe).
fn line_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 512);
    for i in 0..=n {
        draw::dot_i(grid, x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n);
    }
}

impl ProgressStyle for DragonCurve {
    fn name(&self) -> &str {
        "dragon-curve"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Heighway dragon by paper folding: four panels show the strip folded 2, 4, 6 and 8 times, \
         each drawing itself in turn as progress rises; a walker retraces finished dragons with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        const ORDERS: [u32; 4] = [2, 4, 6, 8];
        let n = if dw >= dh * 4 { 4 } else { 1 };
        let pw = dw as f32 / n as f32;

        for k in 0..n {
            let order = ORDERS[(k + 4 - n) % 4];
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let segs = 1u32 << order;
            // Walk the fold sequence: turn left/right by the bit above the
            // lowest set bit of the 1-based segment index.
            let mut pts: Vec<(f32, f32)> = Vec::with_capacity(segs as usize + 1);
            let (mut x, mut y, mut dir) = (0i32, 0i32, 0i32);
            pts.push((0.0, 0.0));
            for i in 1..=segs {
                let (dx, dy) = [(1, 0), (0, -1), (-1, 0), (0, 1)][dir as usize];
                x += dx;
                y += dy;
                pts.push((x as f32, y as f32));
                let right = ((i >> i.trailing_zeros()) >> 1) & 1 == 0;
                dir = (dir + if right { 1 } else { 3 }) & 3;
            }
            // Centre on the bounding box and fit its radius to the panel.
            let (mut lx, mut hx, mut ly, mut hy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for &(px, py) in &pts {
                lx = lx.min(px);
                hx = hx.max(px);
                ly = ly.min(py);
                hy = hy.max(py);
            }
            let (mx, my) = ((lx + hx) * 0.5, (ly + hy) * 0.5);
            // Axis-aligned, uniform scale: right angles stay crisp in dots.
            let s = ((pw - 3.0) / (hx - lx).max(1.0)).min((dh as f32 - 1.0) / (hy - ly).max(1.0));
            let (ox, oy) = (pw * (k as f32 + 0.5), (dh as f32 - 1.0) * 0.5);
            let to_dot = |(px, py): (f32, f32)| {
                (
                    (ox + (px - mx) * s).round() as i32,
                    (oy + (py - my) * s).round() as i32,
                )
            };
            let shown = (local * segs as f32).ceil() as usize;
            if shown == 0 {
                // Waiting panel: the unfolded strip — a single short segment.
                let (a, b) = (to_dot((mx - 1.0, my)), to_dot((mx + 1.0, my)));
                line_i(grid, a.0, a.1, b.0, b.1);
                continue;
            }
            for w in pts.windows(2).take(shown) {
                let (a, b) = (to_dot(w[0]), to_dot(w[1]));
                line_i(grid, a.0, a.1, b.0, b.1);
            }
            // A walker retraces each finished dragon with time.
            if shown >= segs as usize {
                let at = to_dot(pts[(ctx.time.max(0.0) * 9.0) as usize % pts.len()]);
                draw::fill_rect(grid, at.0.max(0) as usize, at.1.max(0) as usize, 2, 2);
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Sierpinski carpet — recursive square removal
// ---------------------------------------------------------------------------

struct SierpinskiCarpet;
impl ProgressStyle for SierpinskiCarpet {
    fn name(&self) -> &str {
        "sierpinski-carpet"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "A row of square Sierpinski carpets: each starts solid, has its centre ninth punched \
         out, then the centre of every remaining ninth, filling in turn as progress rises while \
         a glint sweeps across them with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Square tiles as tall as the bar, separated by a two-dot gap.
        let side = dh;
        let n = ((dw + 2) / (side + 2)).max(1);
        let x_left = dw.saturating_sub(n * (side + 2) - 2) / 2;
        for k in 0..n {
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            let x0 = x_left + k * (side + 2);
            if local <= 0.0 {
                // Waiting tile: only its outline.
                draw::rect_outline(grid, x0, 0, side.min(dw), side);
                continue;
            }
            // Depth 0 = solid, 1 = centre removed, 2 = full level-2 carpet.
            let depth = ((local * 2.999) as u32).min(2);
            for py in 0..side {
                for px in 0..side {
                    // Map dots onto the 9×9 level-2 grid.
                    let (mut u, mut v) = (px * 9 / side, py * 9 / side);
                    let (mut hole, mut level) = (false, 0);
                    // Most significant base-3 digit first.
                    let mut scale = 3;
                    while level < depth {
                        if (u / scale) % 3 == 1 && (v / scale) % 3 == 1 {
                            hole = true;
                            break;
                        }
                        u %= scale;
                        v %= scale;
                        scale /= 3;
                        level += 1;
                        if scale == 0 {
                            break;
                        }
                    }
                    if !hole {
                        draw::dot(grid, x0 + px, py);
                    }
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        let glint = (ctx.time * 0.25).fract();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let near = 1.0 - ((t - glint).abs() * 8.0).min(1.0);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t * 0.6 + near * 0.4));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Burning Ship fractal  — |Re(z)|, |Im(z)| before squaring, zoomed on the ship
// ---------------------------------------------------------------------------

struct BurningShip;
impl ProgressStyle for BurningShip {
    fn name(&self) -> &str {
        "burning-ship"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Burning Ship fractal (z → (|Re z|+i|Im z|)² + c), zoomed onto the little ship near \
         c ≈ −1.76 that gives the set its name: hull on the waterline, masts and rigging \
         rising above — a convoy of them sails in as progress rises, flames flickering with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // The ship window, masts pointing up (−Im at the top).
        let (re0, re1) = (-1.802f32, -1.698f32);
        let (im0, im1) = (-0.088f32, 0.012f32);
        // Tile width keeps the ship roughly in proportion (stretched ≈1.4×).
        let tile = ((dh as f32 * 1.45).round() as usize).clamp(4, dw.max(4));
        let tiles = (dw / tile).max(1);
        let left = (dw - (tiles * tile).min(dw)) / 2;
        let shown = ctx.eased.clamp(0.0, 1.0) * tiles as f32;
        let max_iter = 60u32;
        // Flicker: the "hot" threshold breathes with time.
        let thresh = 20 + ((ctx.time * 5.0).sin() * 2.0).round() as i32;

        for k in 0..tiles {
            let local = (shown - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                // Waiting slot: a dotted waterline.
                let y = (dh as f32 * (0.0 - im0) / (im1 - im0)).round() as usize;
                for x in (left + k * tile..left + (k + 1) * tile).step_by(3) {
                    draw::dot(grid, x, y.min(dh - 1));
                }
                continue;
            }
            // The ship rises out of the water from the waterline up.
            let rise = (1.0 - local) * dh as f32;
            for py in 0..dh {
                if (py as f32) < rise {
                    continue;
                }
                let ci = im0 + (py as f32 + 0.5) / dh as f32 * (im1 - im0);
                for px in 0..tile {
                    let cr = re0 + (px as f32 + 0.5) / tile as f32 * (re1 - re0);
                    let (mut zr, mut zi) = (0.0f32, 0.0f32);
                    let mut n = 0u32;
                    while n < max_iter {
                        let (zr2, zi2) = (zr * zr, zi * zi);
                        if zr2 + zi2 > 4.0 {
                            break;
                        }
                        let nzr = zr2 - zi2 + cr;
                        zi = 2.0 * zr.abs() * zi.abs() + ci;
                        zr = nzr;
                        n += 1;
                    }
                    if n as i32 >= thresh {
                        draw::dot(grid, left + k * tile + px, py);
                    }
                }
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Pythagoras trees — recursive squares on right triangles, a row of them
// ---------------------------------------------------------------------------

struct PythagorasTree;

/// One Pythagoras-tree square on base edge `a→b` (growing to screen-up),
/// then its two children on the top edge split at angle `alpha`.
fn pyth_tree(
    grid: &mut BrailleGrid,
    a: (f32, f32),
    b: (f32, f32),
    alpha: f32,
    depth: u32,
    budget: &mut u32,
) {
    if depth == 0 || *budget == 0 {
        return;
    }
    *budget -= 1;
    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
    let side = (ex * ex + ey * ey).sqrt();
    // Top corners: perpendicular (ey, −ex) points up on screen.
    let c = (b.0 + ey, b.1 - ex);
    let d = (a.0 + ey, a.1 - ex);
    if side < 2.2 {
        // Too small to outline: fill it so the canopy reads as foliage.
        let (lo_x, hi_x) = (
            a.0.min(b.0).min(c.0).min(d.0),
            a.0.max(b.0).max(c.0).max(d.0),
        );
        let (lo_y, hi_y) = (
            a.1.min(b.1).min(c.1).min(d.1),
            a.1.max(b.1).max(c.1).max(d.1),
        );
        let (cx, cy) = ((lo_x + hi_x) * 0.5, (lo_y + hi_y) * 0.5);
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
    } else {
        for (p, q) in [(a, b), (b, c), (c, d), (d, a)] {
            let n = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as i32).clamp(1, 128);
            for i in 0..=n {
                let t = i as f32 / n as f32;
                draw::dot_i(
                    grid,
                    (p.0 + (q.0 - p.0) * t).round() as i32,
                    (p.1 + (q.1 - p.1) * t).round() as i32,
                );
            }
        }
    }
    // Apex of the right triangle sitting on the top edge d→c.
    let (ux, uy) = ((c.0 - d.0) / side.max(1e-6), (c.1 - d.1) / side.max(1e-6));
    let l = side * alpha.cos();
    let (ca, sa) = (alpha.cos(), alpha.sin());
    let apex = (d.0 + (ux * ca + uy * sa) * l, d.1 + (uy * ca - ux * sa) * l);
    pyth_tree(grid, d, apex, alpha, depth - 1, budget);
    pyth_tree(grid, apex, c, alpha, depth - 1, budget);
}

impl ProgressStyle for PythagorasTree {
    fn name(&self) -> &str {
        "pythagoras-tree"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Pythagoras trees: a trunk square carries a right triangle whose two legs carry two \
         smaller squares, and so on — a row of trees each branching level by level as progress \
         reaches it, their crowns swaying gently with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // A 45° tree fits a 6s × 4s box; size the trunk from the height.
        let side = ((dh as f32 - 1.0) / 4.0).max(1.0);
        let slot = (side * 6.4).max(2.0);
        let trees = ((dw as f32 / slot) as usize).clamp(1, 12);
        let pitch = dw as f32 / trees as f32;
        let total = ctx.eased.clamp(0.0, 1.0) * trees as f32;
        let max_depth = 7.0f32;
        for k in 0..trees {
            let local = (total - k as f32).clamp(0.0, 1.0);
            let depth = (local * max_depth).ceil() as u32;
            let cx = pitch * (k as f32 + 0.5);
            let base = dh as f32 - 1.0;
            if depth == 0 {
                // Seed: a dot where the trunk will stand.
                draw::dot_i(grid, cx.round() as i32, base as i32);
                continue;
            }
            let sway = 0.12 * (ctx.time * 1.1 + k as f32 * 1.7).sin();
            let alpha = PI / 4.0 + sway;
            let mut budget = 400u32;
            pyth_tree(
                grid,
                (cx - side * 0.5, base),
                (cx + side * 0.5, base),
                alpha,
                depth,
                &mut budget,
            );
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.max(1) as f32; // brighter at top (crown)
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Newton fractal — basins of attraction for z³ - 1 = 0
// ---------------------------------------------------------------------------

struct NewtonFractal;
impl ProgressStyle for NewtonFractal {
    fn name(&self) -> &str {
        "newton-fractal"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Newton's method on z³−1: each cell is coloured by the cube root it converges to and \
         dotted in convergence-speed bands, so the three basins meet in chains of beads; more \
         of the plane resolves as progress raises the step budget, and the plane turns with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        const ROOTS: [(f32, f32); 3] = [(1.0, 0.0), (-0.5, 0.866_025_4), (-0.5, -0.866_025_4)];
        let max_iter = (2.0 + ctx.eased * 18.0) as u32;
        let s = (dh as f32 / 2.6).max(0.5);
        let (ox, oy) = (dw as f32 * 0.5, dh as f32 * 0.5);
        let rot = ctx.time * 0.15;
        let (cr, sr) = (rot.cos(), rot.sin());
        let (cw, ch) = grid.dimensions();
        // Per-cell vote of which root its dots reach.
        let mut votes = vec![[0u8; 3]; cw * ch];

        for py in 0..dh {
            for px in 0..dw {
                let (u, v) = ((px as f32 + 0.5 - ox) / s, (py as f32 + 0.5 - oy) / s);
                let (mut zr, mut zi) = (u * cr - v * sr, u * sr + v * cr);
                let mut hit = None;
                for it in 0..max_iter {
                    // z ← z − (z³ − 1) / (3z²)
                    let (z2r, z2i) = (zr * zr - zi * zi, 2.0 * zr * zi);
                    let (z3r, z3i) = (z2r * zr - z2i * zi, z2r * zi + z2i * zr);
                    let (nr, ni) = (z3r - 1.0, z3i);
                    let (dr, di) = (3.0 * z2r, 3.0 * z2i);
                    let den = (dr * dr + di * di).max(1e-9);
                    zr -= (nr * dr + ni * di) / den;
                    zi -= (ni * dr - nr * di) / den;
                    if let Some(r) = ROOTS
                        .iter()
                        .position(|&(rr, ri)| (zr - rr).powi(2) + (zi - ri).powi(2) < 0.01)
                    {
                        hit = Some((r, it));
                        break;
                    }
                }
                if let Some((r, it)) = hit {
                    let cell = &mut votes[(py / 4) * cw + px / 2];
                    cell[r] = cell[r].saturating_add(1);
                    // Banded by step count: the fast-converging basin cores stay solid.
                    if it < 2 || it % 2 == 0 {
                        draw::dot(grid, px, py);
                    }
                }
            }
        }

        grid.enable_color_support();
        let shades = [
            ctx.palette.start,
            crate::Color::rgb(235, 235, 245),
            ctx.palette.end,
        ];
        for cy in 0..ch {
            for cx in 0..cw {
                let v = votes[cy * cw + cx];
                let best = (0..3).max_by_key(|&i| v[i]).unwrap_or(0);
                let c = if v[best] == 0 {
                    ctx.palette.track
                } else {
                    shades[best]
                };
                let _ = grid.set_cell_color(cx, cy, c);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Cantor dust — iterated middle-third removal cascade
// ---------------------------------------------------------------------------

struct CantorDust;
impl ProgressStyle for CantorDust {
    fn name(&self) -> &str {
        "cantor-dust"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "The Cantor set as its textbook ladder: a full bar, then thirds, ninths, 27ths and 81sts \
         stacked beneath it, each level drawing in as progress rises while a highlight sweeps \
         the surviving dust with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Widest power of three that fits, so every removed third is whole dots.
        let mut width = 1usize;
        let mut depth = 0u32;
        while width * 3 <= dw && depth < 4 {
            width *= 3;
            depth += 1;
        }
        let levels = depth as usize + 1;
        let rows = (dh / levels).max(1);
        let bar = rows.saturating_sub(1).max(1);
        let x0 = (dw - width) / 2;
        let shown = ctx.eased * levels as f32;
        for lvl in 0..levels {
            let reveal = (shown - lvl as f32).clamp(0.0, 1.0);
            if reveal <= 0.0 {
                break;
            }
            let unit = width / 3usize.pow(lvl as u32);
            let cols = (reveal * width as f32) as usize;
            for x in 0..cols {
                // Keep x only if none of its first `lvl` base-3 digits is 1.
                let mut u = x / unit.max(1);
                let mut keep = true;
                for _ in 0..lvl {
                    if u % 3 == 1 {
                        keep = false;
                        break;
                    }
                    u /= 3;
                }
                if keep {
                    for r in 0..bar {
                        draw::dot(grid, x0 + x, lvl * rows + r);
                    }
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        let sweep = (ctx.time * 0.3).fract();
        for cy in 0..ch {
            for cx in 0..cw {
                let t = cx as f32 / cw.max(1) as f32;
                let near = 1.0 - ((t - sweep).abs() * 6.0).min(1.0);
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t * 0.7 + near * 0.3));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Lyapunov bar — Markus–Lyapunov fractal for the sequence AB
// ---------------------------------------------------------------------------

struct LyapunovBar;
impl ProgressStyle for LyapunovBar {
    fn name(&self) -> &str {
        "lyapunov-bar"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Markus–Lyapunov fractal for the logistic map driven by the sequence AB (a across, b up, \
         both in [3, 4]): stable regions with negative exponent are lit, forming the classic \
         swallow-tail shapes, swept in left to right by progress as the window drifts with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cols = (ctx.eased * dw as f32).ceil() as usize;
        let drift = 0.08 * (ctx.time * 0.4).sin();
        let (cw, ch) = grid.dimensions();
        let mut stable = vec![0u8; cw * ch];
        for px in 0..cols.min(dw) {
            let a = 3.0 + (px as f32 + 0.5) / dw as f32;
            for py in 0..dh {
                let b = 4.0 - (py as f32 + 0.5) / dh as f32 + drift;
                let mut x = 0.5f32;
                let mut lam = 0.0f32;
                for n in 0..60u32 {
                    let r = if n % 2 == 0 { a } else { b };
                    x = r * x * (1.0 - x);
                    if n >= 20 {
                        lam += (r * (1.0 - 2.0 * x)).abs().max(1e-6).ln();
                    }
                }
                if lam < 0.0 {
                    draw::dot(grid, px, py);
                    stable[(py / 4) * cw + px / 2] += 1;
                }
            }
        }
        // Stable cells glow; the chaotic sea between them stays dark.
        for cy in 0..ch {
            for cx in 0..cw {
                let t = (stable[cy * cw + cx] as f32 / 8.0).min(1.0);
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(1.0 - t));
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
    let styles = progress::styles::fractal::styles();
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
