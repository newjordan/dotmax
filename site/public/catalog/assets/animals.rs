//! `animals` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O animals.rs && ./animals [style-name]
//! ```

const DEFAULT_STYLE: &str = "caterpillar";

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
    pub mod animals {
//! Animals-themed progress bars — eleven distinct creatures drawn in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (for perpetual
//! animation) and `ctx.eased` (for progress-driven advancement). The bars use
//! only the `draw::` helpers so all writes are silently bounds-safe.
//!
//! Styles in this file:
//! - `caterpillar`  — round-segmented crawler munching the leaves along a twig
//! - `snail`        — spiral-shelled snail with eye stalks on a slime trail
//! - `inchworm`     — looping arch that stretches flat along a ruler
//! - `fish-school`  — forked-tail fish swimming in formation, trailing bubbles
//! - `snake`        — banded S-curve body with a head and flicking tongue
//! - `rabbit-hops`  — parabolic jump arcs across discrete hop chunks
//! - `paw-prints`   — four-toed prints stepping left-right one by one
//! - `bird-flock`   — V-formation of flapping gulls
//! - `turtle`       — plate-shelled turtle plodding over sand
//! - `ant-march`    — six-legged ants hauling crumbs toward an anthill
//! - `cat-prowl`    — tabby with pointed ears and curled tail chasing yarn

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — meadow green into honey. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(124, 204, 92);
const TINT_END: Color = Color::rgb(240, 184, 64);

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

/// All styles in the `animals` theme.
///
/// Returns one boxed [`ProgressStyle`] per animal variant, ready to be mixed
/// into a gallery or driven individually.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Caterpillar),
        Box::new(Snail),
        Box::new(Inchworm),
        Box::new(FishSchool),
        Box::new(Snake),
        Box::new(Tinted(RabbitHops)),
        Box::new(PawPrints),
        Box::new(BirdFlock),
        Box::new(Turtle),
        Box::new(AntMarch),
        Box::new(CatProwl),
    ]
}

// ── Sprite helpers ───────────────────────────────────────────────────────────

/// Stamp an ASCII sprite (`#` = dot) with its top-left corner at `(x, y)`.
fn stamp(grid: &mut BrailleGrid, rows: &[&str], x: i32, y: i32) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            if c == b'#' {
                draw::dot_i(grid, x + dx as i32, y + dy as i32);
            }
        }
    }
}

/// Filled disc of dots centred on `(cx, cy)`.
fn disc(grid: &mut BrailleGrid, cx: i32, cy: i32, r: f32) {
    let ri = r.ceil() as i32;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            if (dx * dx + dy * dy) as f32 <= r * r {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }
    }
}

/// Colour every non-empty cell overlapping the dot rectangle `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    grid.enable_color_support();
    let (cw, ch) = grid.dimensions();
    let (cx1, cy1) = ((x1 / 2).min(cw as i32 - 1), (y1 / 4).min(ch as i32 - 1));
    for cy in (y0.max(0) / 4)..=cy1 {
        for cx in (x0.max(0) / 2)..=cx1 {
            let c = grid.get_char(cx as usize, cy as usize);
            if c != '\u{2800}' && c != ' ' {
                let _ = grid.set_cell_color(cx as usize, cy as usize, color);
            }
        }
    }
}

// ── Caterpillar ──────────────────────────────────────────────────────────────

struct Caterpillar;
impl ProgressStyle for Caterpillar {
    fn name(&self) -> &str {
        "caterpillar"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Round-segmented green caterpillar with a red head and antennae, rippling along a twig and munching the leaves ahead"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let twig = hi - 1;
        draw::hline(grid, 0, w - 1, twig as usize);
        // Head position: the caterpillar eats its way from left to right.
        let head_x = 26 + (ctx.eased * (wi - 30).max(0) as f32) as i32;
        // Leaves still standing ahead of the head; eaten ones leave a stub.
        for k in 0..=(wi / 10) {
            let lx = k * 10 + 5;
            if lx > head_x + 3 {
                stamp(
                    grid,
                    &[" ### ", "#####", "#####", " ### ", "  #  "],
                    lx - 2,
                    twig - 5,
                );
            } else {
                draw::dot_i(grid, lx, twig - 1);
            }
        }
        paint(grid, 0, twig - 6, wi, twig, Color::rgb(70, 160, 70));
        paint(grid, 0, twig, wi, twig, Color::rgb(150, 105, 60));
        // Body: round segments; a hump ripples tail→head (peristaltic crawl).
        let base = twig - 4;
        for i in (1..=6i32).rev() {
            let wave = ((i as f32 * 0.9 - ctx.time * 2.0 * PI * 0.75).sin()).max(0.0);
            let sy = base - (wave * 2.5).round() as i32;
            let sx = head_x - i * 5 + 1;
            disc(grid, sx, sy, 2.4);
            draw::dot_i(grid, sx, sy + 3);
        }
        paint(
            grid,
            head_x - 34,
            0,
            head_x - 3,
            twig - 1,
            Color::rgb(130, 220, 70),
        );
        // Head with antennae that twitch.
        let twitch = ((ctx.time * 3.0) as i32).rem_euclid(2);
        disc(grid, head_x + 1, base - 1, 3.0);
        draw::dot_i(grid, head_x + 2, base - 5);
        draw::dot_i(grid, head_x + 3 + twitch, base - 6);
        draw::dot_i(grid, head_x - 1, base - 5);
        draw::dot_i(grid, head_x - 1 + twitch, base - 6);
        paint(
            grid,
            head_x - 2,
            0,
            head_x + 5,
            twig - 1,
            Color::rgb(235, 80, 60),
        );
        Ok(())
    }
}

// ── Snail ─────────────────────────────────────────────────────────────────────

/// Snail facing right: spiral shell riding on a foot that tapers to the head.
const SNAIL: [&str; 10] = [
    "    #####          ",
    "  ##     ##        ",
    " #  #####  #       ",
    "#  #     #  #      ",
    "# #  ##   # #   ## ",
    "# #  # #  # #  ####",
    "#  #  ## #  # #####",
    " #  ####   # ######",
    "  ##     ## #######",
    "####################",
];

struct Snail;
impl ProgressStyle for Snail {
    fn name(&self) -> &str {
        "snail"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Spiral-shelled snail with waving eye stalks gliding right on a glistening slime trail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let sx = (ctx.eased * (wi - 22).max(0) as f32) as i32;
        let top = ground - 9;
        // Slime trail: glints shimmer along the path already travelled.
        let shimmer = (ctx.time * 6.0) as i32;
        for x in 0..sx.max(0) + 2 {
            if (x + shimmer) % 5 != 0 {
                draw::dot_i(grid, x, ground);
            }
        }
        paint(grid, 0, ground, sx + 1, ground, Color::rgb(150, 190, 230));
        stamp(grid, &SNAIL, sx, top);
        // Eye stalks sway on the time clock.
        let sway = ((ctx.time * PI).sin() * 1.4).round() as i32;
        for j in 1..=4i32 {
            let lean = (sway * j) / 4;
            draw::dot_i(grid, sx + 16 + lean, top + 4 - j);
            draw::dot_i(grid, sx + 18 + lean + j / 2, top + 4 - j);
        }
        disc(grid, sx + 16 + sway, top - 1, 0.9);
        disc(grid, sx + 20 + sway, top - 1, 0.9);
        paint(
            grid,
            sx + 12,
            top - 2,
            sx + 22,
            ground,
            Color::rgb(190, 210, 150),
        );
        paint(grid, sx, top, sx + 11, ground - 1, Color::rgb(220, 140, 70));
        Ok(())
    }
}

// ── Inchworm ─────────────────────────────────────────────────────────────────

struct Inchworm;
impl ProgressStyle for Inchworm {
    fn name(&self) -> &str {
        "inchworm"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Inchworm looping into an arch and stretching flat as it measures its way along a ruler"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ruler = hi - 1;
        // Ruler with inch ticks; the measured part is lit.
        draw::hline(grid, 0, w - 1, ruler as usize);
        for x in (0..wi).step_by(4) {
            let tall = if x % 16 == 0 { 3 } else { 1 };
            for d in 1..=tall {
                draw::dot_i(grid, x, ruler - d);
            }
        }
        // Gait: three loops per 4 s. First half the head reaches forward
        // (arch flattens); second half the tail catches up (arch rises).
        let cyc = (ctx.time * 0.75).fract();
        let (min_len, max_len) = (8.0f32, 22.0f32);
        let len = if cyc < 0.5 {
            min_len + (max_len - min_len) * (cyc * 2.0)
        } else {
            max_len - (max_len - min_len) * ((cyc - 0.5) * 2.0)
        };
        let base = ctx.eased * (wi as f32 - max_len - 4.0).max(0.0);
        let tail = if cyc < 0.5 {
            base
        } else {
            base + max_len - len
        };
        let arch = ((max_len - len) * 0.62 + 1.0).min((hi - 4).max(1) as f32);
        let foot = ruler - 2;
        paint(
            grid,
            0,
            ruler - 3,
            tail as i32 + 2,
            ruler,
            Color::rgb(240, 200, 90),
        );
        paint(
            grid,
            tail as i32 + 3,
            ruler - 3,
            wi,
            ruler,
            Color::rgb(120, 110, 90),
        );
        // Body: a thick half-sine arch from tail to head.
        let steps = (len * 2.0) as i32;
        for s in 0..=steps {
            let f = s as f32 / steps.max(1) as f32;
            let x = (tail + f * len).round() as i32;
            let y = (foot as f32 - (f * PI).sin() * arch).round() as i32;
            draw::dot_i(grid, x, y);
            draw::dot_i(grid, x, y - 1);
        }
        // Head with a stubby antenna at the leading end; clasping feet.
        let hx = (tail + len).round() as i32;
        disc(grid, hx + 1, foot - 1, 1.5);
        draw::dot_i(grid, hx + 3, foot - 3);
        draw::dot_i(grid, tail as i32 - 1, foot);
        paint(
            grid,
            tail as i32 - 2,
            0,
            hx + 4,
            foot,
            Color::rgb(140, 230, 80),
        );
        Ok(())
    }
}

// ── Fish School ───────────────────────────────────────────────────────────────

/// Fish facing right: forked tail, oval body, open eye.
const FISH: [&str; 7] = [
    "     #####   ",
    "#  ########  ",
    "## ###### ## ",
    " ############",
    "## ##########",
    "#  ######### ",
    "     #####   ",
];

struct FishSchool;
impl ProgressStyle for FishSchool {
    fn name(&self) -> &str {
        "fish-school"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "School of forked-tail fish swimming right in formation, bobbing and trailing bubbles"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let lead = 14 + (ctx.eased * (wi - 28).max(0) as f32) as i32;
        let colors = [
            Color::rgb(255, 150, 60),
            Color::rgb(255, 210, 80),
            Color::rgb(90, 200, 255),
        ];
        // Formation: leader plus two wingmen behind, each on its own bob phase.
        let slots = [(0i32, 4i32), (-15, 0), (-15, 9)];
        for (i, &(dx, dy)) in slots.iter().enumerate() {
            let bob = ((ctx.time * 2.0 * PI * 0.5 + i as f32 * 2.1).sin() * 1.2).round() as i32;
            let (fx, fy) = (lead + dx, (dy + bob).clamp(0, (hi - 7).max(0)));
            stamp(grid, &FISH, fx, fy);
            paint(grid, fx, fy, fx + 12, fy + 6, colors[i % colors.len()]);
        }
        // Bubbles drift up from the wake on the time clock.
        for k in 0..6i32 {
            let bx = lead - 20 - k * 7;
            if bx < 0 {
                break;
            }
            let by = hi - 1 - ((ctx.time * 6.0 + k as f32 * 5.0) as i32).rem_euclid(hi.max(1));
            draw::dot_i(grid, bx, by);
            draw::dot_i(grid, bx + 1, by - 1);
            paint(grid, bx, by - 1, bx + 1, by, Color::rgb(170, 220, 255));
        }
        Ok(())
    }
}

// ── Snake ─────────────────────────────────────────────────────────────────────

/// Snake head facing right, with an open eye.
const SNAKE_HEAD: [&str; 5] = [" ####  ", "## ### ", "#######", "###### ", " ####  "];

struct Snake;
impl ProgressStyle for Snake {
    fn name(&self) -> &str {
        "snake"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Banded snake slithering in S-curves, tapered tail to a wide head with a flicking forked tongue"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = hi as f32 / 2.0 - 0.5;
        let amp = (hi as f32 / 2.0 - 3.5).clamp(0.0, 4.0);
        let head_x = 8 + (ctx.eased * (wi - 18).max(0) as f32) as i32;
        // Body: a travelling sine, thickening from a thin tail to the neck.
        let y_at = |x: i32| mid + ((x as f32 * 0.3) - ctx.time * 2.0 * PI * 0.5).sin() * amp;
        for x in 0..head_x {
            let thick = 2 + (2 * x) / head_x.max(1);
            let y = y_at(x).round() as i32;
            for d in 0..thick {
                draw::dot_i(grid, x, y - thick / 2 + d);
            }
        }
        // Alternating bands along the body.
        for cx in 0..=(head_x / 2) {
            let band = if (cx / 2) % 2 == 0 {
                Color::rgb(110, 200, 80)
            } else {
                Color::rgb(230, 190, 60)
            };
            paint(grid, cx * 2, 0, cx * 2 + 1, hi, band);
        }
        // Head and forked tongue, flicking out twice a second.
        let hy = y_at(head_x).round() as i32 - 2;
        stamp(grid, &SNAKE_HEAD, head_x, hy);
        paint(
            grid,
            head_x,
            hy,
            head_x + 6,
            hy + 4,
            Color::rgb(110, 200, 80),
        );
        if (ctx.time * 2.0).fract() < 0.5 {
            draw::hline(
                grid,
                (head_x + 7).max(0) as usize,
                (head_x + 8).max(0) as usize,
                (hy + 2).max(0) as usize,
            );
            draw::dot_i(grid, head_x + 9, hy + 1);
            draw::dot_i(grid, head_x + 9, hy + 3);
            paint(
                grid,
                head_x + 7,
                hy + 1,
                head_x + 9,
                hy + 3,
                Color::rgb(240, 70, 70),
            );
        }
        Ok(())
    }
}

// ── Rabbit Hops ───────────────────────────────────────────────────────────────

struct RabbitHops;
impl ProgressStyle for RabbitHops {
    fn name(&self) -> &str {
        "rabbit-hops"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Rabbit hopping across in parabolic arcs — each hop a discrete progress chunk"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;
        let ground = hi - 1;

        // Meadow: dotted ground line across the full width, with grass tufts.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, ground);
        }
        for k in 0..=(wi / 9) {
            let gx = k * 9 + 4;
            draw::dot_i(grid, gx, ground - 1);
            draw::dot_i(grid, gx + 1, ground - 2);
        }

        // Hop geometry: the rabbit crosses in discrete parabolic chunks.
        let n_hops = 4i32;
        let x_min = 7.0f32;
        let x_max = (wi as f32 - 9.0).max(x_min + 1.0);
        let hop_index_f = ctx.eased * n_hops as f32;
        let current_hop = (hop_index_f as i32).min(n_hops - 1);
        let hop_frac = (hop_index_f - current_hop as f32).clamp(0.0, 1.0);
        let hop_x = |hop: i32, t: f32| -> f32 {
            let s = (hop as f32 + t) / n_hops as f32;
            x_min + s * (x_max - x_min)
        };
        let peak = (hi as f32 * 0.19).max(2.0);
        let arc_y = |t: f32| -> f32 { ground as f32 - 2.0 + 4.0 * peak * t * (t - 1.0) };

        // Paw prints where earlier hops landed.
        for hop in 0..current_hop {
            let px = hop_x(hop, 1.0) as i32;
            draw::dot_i(grid, px, ground - 1);
            draw::dot_i(grid, px + 1, ground - 1);
            draw::dot_i(grid, px + 3, ground - 2);
            draw::dot_i(grid, px + 4, ground - 2);
        }

        // Dotted trajectory trail behind the rabbit within the current hop.
        // The dash pattern crawls with time (rate 1.5/s: +6 slots per 4 s loop).
        let dash = ((ctx.time * 1.5) as i32).rem_euclid(3);
        let trail_steps = 24i32;
        for i in 0..trail_steps {
            let t = i as f32 / trail_steps as f32;
            if t >= hop_frac {
                break;
            }
            if (i + dash) % 3 == 0 {
                draw::dot_i(grid, hop_x(current_hop, t) as i32, arc_y(t) as i32);
            }
        }

        // Rabbit anchor: bottom-centre of the body, riding the arc plus a
        // gentle time-driven bounce (0.25 Hz-multiple: seamless 4 s loop).
        let bounce = (ctx.time * PI).sin().abs() * 1.2;
        let rx = hop_x(current_hop, hop_frac).round() as i32;
        let ry = (arc_y(hop_frac) - bounce).round() as i32;

        // Body: filled ellipse (10 wide, 5 tall), facing right.
        for dy in -5i32..=0 {
            let fy = (dy as f32 + 2.5) / 3.0;
            let span = ((1.0 - fy * fy).max(0.0)).sqrt() * 4.6;
            let lo = (-span).round() as i32;
            let hi_x = span.round() as i32;
            for dx in lo..=hi_x {
                draw::dot_i(grid, rx + dx, ry + dy);
            }
        }
        // Head: filled disc ahead of the body, one blank dot left as the eye.
        for dy in -7i32..=-4 {
            for dx in 4i32..=8 {
                let ex = dx as f32 - 6.0;
                let ey = dy as f32 + 5.5;
                if ex * ex / 5.5 + ey * ey / 3.2 <= 1.0 && !(dx == 7 && dy == -6) {
                    draw::dot_i(grid, rx + dx, ry + dy);
                }
            }
        }
        // Ears: two slanted lines; the back ear flicks on a 2/s time slot.
        let flick = ((ctx.time * 2.0) as i32).rem_euclid(2);
        for j in 1..=3i32 {
            draw::dot_i(grid, rx + 4, ry - 7 - j);
            draw::dot_i(grid, rx + 6 + (j > 2) as i32 * flick, ry - 7 - j);
        }
        // Tail puff and airborne feet.
        draw::dot_i(grid, rx - 5, ry - 4);
        draw::dot_i(grid, rx - 6, ry - 3);
        draw::dot_i(grid, rx - 3, ry + 1);
        draw::dot_i(grid, rx - 4, ry + 1);
        draw::dot_i(grid, rx + 3, ry + 1);
        Ok(())
    }
}

// ── Paw Prints ────────────────────────────────────────────────────────────────

/// One paw print, toes pointing right: heel pad plus four toe beans.
const PAW: [&str; 9] = [
    "    ##  ", "    ##  ", " ##   ##", "####  ##", "#####   ", "####  ##", " ##   ##", "    ##  ",
    "    ##  ",
];

struct PawPrints;
impl ProgressStyle for PawPrints {
    fn name(&self) -> &str {
        "paw-prints"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Four-toed paw prints stepping left-right across the bar, the freshest print pressing in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let stride = 11i32;
        let count = ((wi - 8) / stride + 1).max(1);
        let shown = (ctx.eased * count as f32).ceil() as i32;
        for k in 0..shown.min(count) {
            let x = 1 + k * stride;
            let y = if k % 2 == 0 { (hi - 9).max(0) } else { 0 };
            let newest = k == shown - 1 && ctx.eased < 1.0;
            // The newest print flickers as it is pressed into the ground.
            if newest && (ctx.time * 4.0).fract() < 0.3 {
                stamp(grid, &PAW[2..7], x, y + 2);
            } else {
                stamp(grid, &PAW, x, y);
            }
            let age = (shown - 1 - k) as f32 / count as f32;
            let color = ctx.palette.sample(1.0 - age);
            paint(grid, x, y, x + 7, y + 8, color);
        }
        Ok(())
    }
}

// ── Bird Flock ────────────────────────────────────────────────────────────────

/// Gull wing-beat frames: wings up, gliding "m", wings down.
const GULL: [[&str; 5]; 3] = [
    [
        "##         ##",
        "  ##     ##  ",
        "    ## ##    ",
        "     ###     ",
        "      #      ",
    ],
    [
        "             ",
        "  ###   ###  ",
        " #   # #   # ",
        "#     #     #",
        "             ",
    ],
    [
        "             ",
        "     ###     ",
        "   ## # ##   ",
        " ##       ## ",
        "#           #",
    ],
];

struct BirdFlock;
impl ProgressStyle for BirdFlock {
    fn name(&self) -> &str {
        "bird-flock"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "V-formation of flapping gulls flying right, each on its own wing-beat"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let lead = 30 + (ctx.eased * (wi - 43).max(0) as f32) as i32;
        let mid = (hi / 2 - 3).max(0);
        let slots = [(0i32, 0i32), (-15, -5), (-15, 5), (-30, -6), (-30, 6)];
        for (i, &(dx, dy)) in slots.iter().enumerate() {
            // Beat cycle up → glide → down → glide, staggered per bird.
            let beat = ((ctx.time * 6.0) as i32 + i as i32).rem_euclid(4);
            let frame = [0usize, 1, 2, 1][beat as usize];
            let (bx, by) = (lead + dx, (mid + dy).clamp(0, (hi - 5).max(0)));
            stamp(grid, &GULL[frame], bx, by);
            let color = ctx.palette.sample(i as f32 / 4.0);
            paint(grid, bx, by, bx + 12, by + 4, color);
        }
        Ok(())
    }
}

// ── Turtle ────────────────────────────────────────────────────────────────────

/// Turtle facing right: domed shell with plate seams, head with an eye, tail.
const TURTLE: [&str; 7] = [
    "      #######          ",
    "    ###########        ",
    "   ###  ###  ###   ### ",
    "  #### ##### ####  # ##",
    " ####  #####  #########",
    "#######################",
    "  ##################   ",
];

/// Two walking poses for the four stubby legs.
const TURTLE_LEGS: [[&str; 2]; 2] = [
    ["  ###          ###    ", "  ##            ##    "],
    ["   ###       ###      ", "    ##       ##       "],
];

struct Turtle;
impl ProgressStyle for Turtle {
    fn name(&self) -> &str {
        "turtle"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Plate-shelled turtle plodding right on stubby legs, head bobbing, leaving a footprint trail in the sand"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let tx = (ctx.eased * (wi - 24).max(0) as f32) as i32;
        let pose = ((ctx.time * 2.0) as i32).rem_euclid(2) as usize;
        let top = ground - 9;
        // Sand, with footprints along the stretch already walked.
        draw::hline(grid, 0, w - 1, ground as usize);
        for x in (2..tx.max(0)).step_by(5) {
            draw::dot_i(grid, x, ground - 1);
        }
        paint(grid, 0, ground - 1, wi, ground, Color::rgb(210, 180, 120));
        stamp(grid, &TURTLE, tx, top);
        stamp(grid, &TURTLE_LEGS[pose], tx, top + 7);
        // Head bobs a dot forward on each step.
        draw::dot_i(grid, tx + 23, top + 3 + pose as i32);
        paint(
            grid,
            tx + 18,
            top + 2,
            tx + 23,
            ground - 1,
            Color::rgb(170, 215, 120),
        );
        paint(
            grid,
            tx,
            top + 7,
            tx + 18,
            ground - 1,
            Color::rgb(170, 215, 120),
        );
        paint(grid, tx, top, tx + 17, top + 6, Color::rgb(70, 150, 80));
        Ok(())
    }
}

// ── Ant March ─────────────────────────────────────────────────────────────────

/// Ant facing right — abdomen, waist, thorax, head, antenna — in two leg poses.
const ANT: [[&str; 6]; 2] = [
    [
        "          # ",
        " ###     #  ",
        "##### #####",
        "###########",
        " ###  # # # ",
        "     #  #  #",
    ],
    [
        "         #  ",
        " ###     #  ",
        "##### #####",
        "###########",
        " ### # # #  ",
        "    #  # #  ",
    ],
];

struct AntMarch;
impl ProgressStyle for AntMarch {
    fn name(&self) -> &str {
        "ant-march"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Single file of six-legged ants marching toward an anthill, every other one hauling a crumb"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        // Anthill on the right edge.
        for r in 0..6i32 {
            draw::hline(
                grid,
                (wi - 12 + r).max(0) as usize,
                (wi - 1).max(0) as usize,
                (ground - 5 + r).max(0) as usize,
            );
        }
        paint(
            grid,
            wi - 12,
            ground - 6,
            wi,
            ground,
            Color::rgb(170, 120, 70),
        );
        // The column reaches further as progress grows; ants scroll forward one
        // spacing per 4 s loop so the march is seamless.
        let front = (ctx.eased * (wi - 14).max(0) as f32) as i32;
        let spacing = 16i32;
        let shift = ((ctx.time * 4.0) as i32).rem_euclid(spacing);
        let mut k = 0i32;
        while k * spacing <= front + spacing {
            let ax = front - 12 - k * spacing + shift;
            if ax + 11 >= 0 && ax <= front {
                let pose = ((ctx.time * 8.0) as i32 + k).rem_euclid(2) as usize;
                let ay = ground - 5;
                stamp(grid, &ANT[pose], ax, ay);
                paint(grid, ax, ay, ax + 11, ground, Color::rgb(220, 90, 60));
                if k % 2 == 0 {
                    draw::fill_rect(
                        grid,
                        (ax + 6).max(0) as usize,
                        (ay - 3).max(0) as usize,
                        3,
                        3,
                    );
                    paint(
                        grid,
                        ax + 6,
                        ay - 3,
                        ax + 8,
                        ay - 2,
                        Color::rgb(250, 230, 160),
                    );
                }
            }
            k += 1;
        }
        Ok(())
    }
}

// ── Cat Prowl ─────────────────────────────────────────────────────────────────

/// Cat facing right: pointed ears, eyes, arched back and a question-mark tail.
const CAT: [&str; 9] = [
    "               #   #  ",
    "               ## ##  ",
    " #             #####  ",
    "#              # # #  ",
    "#              ###### ",
    "#  ############ ###   ",
    " # ##############     ",
    "  ###############     ",
    "   ##############     ",
];

/// Trotting leg poses.
const CAT_LEGS: [[&str; 3]; 2] = [
    [
        "   ##        ##   ",
        "  ##          ##  ",
        "  #            #  ",
    ],
    [
        "    ##      ##    ",
        "    ##      ##    ",
        "    #       #     ",
    ],
];

struct CatProwl;
impl ProgressStyle for CatProwl {
    fn name(&self) -> &str {
        "cat-prowl"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Orange tabby with pointed ears and a curled tail trotting right after a rolling ball of yarn"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let cx = (ctx.eased * (wi - 34).max(0) as f32) as i32;
        let pose = ((ctx.time * 4.0) as i32).rem_euclid(2) as usize;
        let top = ground - 12 + pose as i32;
        stamp(grid, &CAT, cx, top);
        stamp(grid, &CAT_LEGS[pose], cx, top + 9);
        // Tail tip flicks.
        if (ctx.time * 1.5).fract() < 0.5 {
            draw::dot_i(grid, cx + 2, top + 1);
        }
        paint(grid, cx - 1, top, cx + 21, ground, Color::rgb(245, 160, 70));
        // Yarn ball rolling ahead, its wrap lines turning with time.
        let (bx, by) = (cx + 28, ground - 3);
        disc(grid, bx, by, 3.0);
        let spin = (ctx.time * 4.0) as i32;
        for d in -2..=2i32 {
            if (d + spin).rem_euclid(2) == 0 {
                draw::dot_i(grid, bx + d, by - d);
            }
        }
        // A loose thread trails back toward the cat.
        for x in (cx + 22)..(bx - 3) {
            draw::dot_i(grid, x, ground - (x + spin).rem_euclid(3) / 2);
        }
        paint(
            grid,
            cx + 22,
            by - 3,
            bx + 3,
            ground,
            Color::rgb(240, 110, 170),
        );
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
    let styles = progress::styles::animals::styles();
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
