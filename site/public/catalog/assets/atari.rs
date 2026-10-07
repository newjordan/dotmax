//! `atari` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O atari.rs && ./atari [style-name]
//! ```

const DEFAULT_STYLE: &str = "pong";

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
    pub mod atari {
//! ATARI 2600 / early-arcade themed progress bars.
//!
//! Each style draws the game's own iconic screen in braille dot-art: Pong's
//! paddles, net and blocky score, Breakout's rainbow brick wall, Asteroids'
//! vector rocks, Missile Command's cities under falling warheads, Centipede's
//! segmented bug in a mushroom field, Adventure's castle, hero and dragon,
//! Pitfall Harry on his vine, Combat's treaded tanks, Kaboom's Mad Bomber and
//! buckets, the Yar and the Qotile's shield, and Lunar Lander's descent.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — warm CRT phosphor orange.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(228, 84, 38);
const TINT_END: Color = Color::rgb(255, 184, 28);

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

// ---------------------------------------------------------------------------
// Sprite helpers — tiny dot-art stamps shared by the pictorial styles.
// ---------------------------------------------------------------------------

/// Stamp a sprite: every `#` in `rows` sets the dot at `(x + col, y + row)`.
/// Off-grid dots are dropped, so sprites may hang off any edge.
fn sprite(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + c as i32, y + r as i32);
            }
        }
    }
}

/// [`sprite`] mirrored left-to-right (for the facing-left twin of a sprite).
fn sprite_flip(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        let n = line.len() as i32;
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + n - 1 - c as i32, y + r as i32);
            }
        }
    }
}

/// Solid block of dots from signed coordinates (clipped to the grid).
fn block(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32) {
    for yy in y..y + h {
        for xx in x..x + w {
            draw::dot_i(grid, xx, yy);
        }
    }
}

/// Blocky 3×5 score digits, as the 2600 drew them with playfield pixels.
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

/// Draw digit `d` at `(x, y)`; each font pixel is `sx` dots wide.
fn digit(grid: &mut BrailleGrid, x: i32, y: i32, d: usize, sx: i32) {
    for (r, line) in DIGITS[d % 10].iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                block(grid, x + c as i32 * sx, y + r as i32, sx, 1);
            }
        }
    }
}

/// Triangle wave `0 → 1 → 0` with period 1 — a ball bouncing between walls.
fn tri(t: f32) -> f32 {
    1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs()
}

/// Deterministic hash of `i` into `0.0..1.0` (stable scatter, no RNG state).
fn hash01(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65536.0
}

/// All styles in the `atari` theme.
///
/// Returns eleven structurally distinct bars, each referencing a different
/// Atari-era game mechanic — from Pong's paddle rally to Lunar Lander's
/// vector descent. No two styles share the same geometry or algorithm.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Pong)),
        Box::new(Breakout),
        Box::new(Tinted(Asteroids)),
        Box::new(Tinted(MissileCommand)),
        Box::new(Tinted(Centipede)),
        Box::new(Tinted(Adventure)),
        Box::new(Tinted(Pitfall)),
        Box::new(Tinted(Combat)),
        Box::new(Tinted(Kaboom)),
        Box::new(Tinted(YarsRevenge)),
        Box::new(Tinted(LunarLander)),
    ]
}

// ---------------------------------------------------------------------------
// 1. Pong — two paddles rally a square ball over a dashed net; score = progress
// ---------------------------------------------------------------------------
struct Pong;
impl ProgressStyle for Pong {
    fn name(&self) -> &str {
        "pong"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Pong: two paddles rally a square ball over a dashed net; blocky score digits count up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Dashed centre net.
        let mid = wi / 2;
        for y in (0..hi).filter(|y| y % 4 < 2) {
            draw::dot_i(grid, mid, y);
        }

        // Scoreboard: player score climbs 0→9 with progress, CPU lags behind.
        let score = ((ctx.progress * 9.0).round() as usize).min(9);
        digit(grid, mid - 12, 0, score, 2);
        digit(grid, mid + 6, 0, score / 3, 2);

        // Square ball crosses paddle-to-paddle and bounces off top/bottom.
        let span_y = (hi - 2).max(0) as f32;
        let ball_y = |t: f32| (tri(t / 1.9 + 0.3) * span_y) as i32;
        let bx = 3 + (tri(ctx.time / 2.8) * (wi - 8).max(0) as f32) as i32;
        let by = ball_y(ctx.time);
        block(grid, bx, by, 2, 2);

        // Paddles (2 dots wide) track the ball; the CPU side reacts late.
        let ph = (hi / 3 + 1).clamp(1, 6);
        let paddle = |lag: f32| (ball_y(ctx.time - lag) + 1 - ph / 2).clamp(0, (hi - ph).max(0));
        block(grid, 1, paddle(0.0), 2, ph);
        block(grid, wi - 3, paddle(0.2), 2, ph);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Breakout — rainbow brick rows, a paddle and a ball; bricks cleared = eased
// ---------------------------------------------------------------------------
struct Breakout;

/// Classic Breakout row colours, top row first.
const BRICK_ROWS: [Color; 4] = [
    Color::rgb(222, 64, 52),
    Color::rgb(240, 140, 40),
    Color::rgb(230, 206, 60),
    Color::rgb(80, 190, 90),
];

impl ProgressStyle for Breakout {
    fn name(&self) -> &str {
        "breakout"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Breakout: red/orange brick rows above a paddle and bouncing ball; bricks knocked out = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);

        // One brick row per cell row (3 dots + mortar) over the top half.
        // Lower rows break first, with a hashed scatter so holes look punched.
        let rows = (ch / 2).max(1);
        let pitch = 8;
        let cols = (wi / pitch).max(1);
        let span = rows.saturating_sub(1).max(1) as f32;
        for r in 0..rows {
            for c in 0..cols {
                let i = (r as i32 * cols + c) as u32;
                let order = 0.98 * (0.65 * hash01(i) + 0.35 * (1.0 - r as f32 / span));
                if order >= ctx.eased {
                    block(grid, c * pitch, r as i32 * 4, pitch - 1, 3);
                }
            }
        }

        // Ball bounces around the open court below the wall.
        let top = rows as i32 * 4 + 1;
        let bx = (tri(ctx.time / 1.6) * (wi - 2).max(0) as f32) as i32;
        let by = top + (tri(ctx.time / 1.1) * (hi - top - 3).max(0) as f32) as i32;
        block(grid, bx, by, 2, 2);

        // Paddle on the bottom row follows the ball.
        let pw = 10.min(wi);
        let px = (bx - pw / 2 + 1).clamp(0, (wi - pw).max(0));
        block(grid, px, hi - 1, pw, 1);

        // Colour: rainbow brick rows, phosphor tint for the court.
        for r in 0..ch {
            let color = if r < rows {
                BRICK_ROWS[r % BRICK_ROWS.len()]
            } else {
                sample_tint(0.8)
            };
            draw::tint_row(grid, r, 0, cw - 1, color);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Asteroids — wireframe vector polygons shatter as progress rises
// ---------------------------------------------------------------------------
struct Asteroids;
impl ProgressStyle for Asteroids {
    fn name(&self) -> &str {
        "asteroids"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Asteroids: vector-wireframe rocks shatter as progress rises; a ship remains"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Draw a vector polygon (n-gon) centred at (cx, cy) with given radius.
        let draw_ngon = |grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32, n: usize, rot: f32| {
            if n < 2 {
                return;
            }
            for i in 0..n {
                let a0 = rot + 2.0 * PI * i as f32 / n as f32;
                let a1 = rot + 2.0 * PI * (i + 1) as f32 / n as f32;
                let x0 = cx + (r as f32 * a0.cos()) as i32;
                let y0 = cy + (r as f32 * a0.sin()) as i32;
                let x1 = cx + (r as f32 * a1.cos()) as i32;
                let y1 = cy + (r as f32 * a1.sin()) as i32;
                // Bresenham line via dot_i.
                let mut sx = x0;
                let mut sy = y0;
                let dx = (x1 - x0).abs();
                let dy = (y1 - y0).abs();
                let step_x: i32 = if x1 > x0 { 1 } else { -1 };
                let step_y: i32 = if y1 > y0 { 1 } else { -1 };
                let mut err = dx - dy;
                for _ in 0..(dx + dy + 1).min(256) {
                    draw::dot_i(grid, sx, sy);
                    if sx == x1 && sy == y1 {
                        break;
                    }
                    let e2 = 2 * err;
                    if e2 > -dy {
                        err -= dy;
                        sx += step_x;
                    }
                    if e2 < dx {
                        err += dx;
                        sy += step_y;
                    }
                }
            }
        };

        // Twinkling starfield backdrop (integer 4/s slots: seamless 4 s loop).
        let star_hash = |n: u32| -> u32 {
            let mut x = n.wrapping_mul(2_654_435_761);
            x ^= x >> 15;
            x.wrapping_mul(2_246_822_519)
        };
        let blink = ((ctx.time * 4.0) as u32).rem_euclid(8);
        for i in 0..24u32 {
            if (star_hash(i * 5 + 3) + blink) % 8 == 0 {
                continue; // this star is blinked off right now
            }
            let sx = (star_hash(i * 2 + 1) % w as u32) as i32;
            let sy = (star_hash(i * 7 + 5) % h as u32) as i32;
            draw::dot_i(grid, sx, sy);
        }

        // Asteroid field: 7 full-size rocks at fixed positions, blasted one by
        // one in a scattered order as progress rises. Each intact rock keeps
        // its full wireframe (progress = rocks destroyed, not rocks shrunk).
        let asteroid_data: [(f32, f32, f32, f32, usize); 7] = [
            (0.08, 0.32, 0.36, 0.0, 6),
            (0.22, 0.70, 0.30, 0.5, 5),
            (0.36, 0.26, 0.40, 1.1, 7),
            (0.52, 0.68, 0.34, 0.8, 6),
            (0.66, 0.28, 0.28, 0.3, 5),
            (0.81, 0.68, 0.40, 1.4, 7),
            (0.94, 0.30, 0.28, 0.9, 5),
        ];
        // Destruction order: scattered across the field, not left-to-right.
        let kill_rank: [f32; 7] = [3.0, 0.0, 5.0, 1.0, 6.0, 2.0, 4.0];
        let destroyed = ctx.eased * 7.0;
        for (i, &(fx, fy, fr, rot_off, sides)) in asteroid_data.iter().enumerate() {
            let cx = (fx * w as f32) as i32;
            let cy = (fy * h as f32) as i32;
            let r = ((h as f32 * fr) as i32).max(2);
            // Rotate by whole symmetry steps per 4 s loop → seamless.
            let step = 2.0 * PI / sides as f32;
            let spin = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rot = rot_off + ctx.time * spin * step * 0.25 * (1 + i % 2) as f32;
            let frac = destroyed - kill_rank[i];
            if frac >= 1.0 {
                // Rock already blasted: a few dust specks linger where it was.
                for j in 0..3u32 {
                    let dx = (star_hash(i as u32 * 13 + j * 3 + 1) % (r as u32 * 2 + 1)) as i32 - r;
                    let dy = (star_hash(i as u32 * 17 + j * 5 + 2) % (r as u32 + 1)) as i32 - r / 2;
                    draw::dot_i(grid, cx + dx, cy + dy);
                }
                continue;
            }
            if frac > 0.0 {
                // Mid-shatter: shards fly outward from the rock's position.
                for j in 0..sides {
                    let a = rot + 2.0 * PI * j as f32 / sides as f32;
                    let d = r as f32 * (0.4 + frac * 2.2);
                    let px = cx + (d * a.cos()) as i32;
                    let py = cy + (d * a.sin()) as i32;
                    draw::dot_i(grid, px, py);
                    draw::dot_i(grid, px + 1, py);
                }
                continue;
            }
            draw_ngon(grid, cx, cy, r, sides, rot);
            // A crater speck inside the wireframe sells the rock.
            draw::dot_i(grid, cx - r / 3, cy + r / 4);
            draw::dot_i(grid, cx + r / 3, cy - r / 4);
        }

        // Player ship: vector triangle near bottom-centre, thrust flickering
        // on an 8/s slot (seamless over the 4 s loop).
        let ship_cx = (w / 2) as i32;
        let ship_cy = (h * 7 / 10) as i32;
        let ship_r = ((h as f32 * 0.22).max(2.0)) as i32;
        draw_ngon(grid, ship_cx, ship_cy, ship_r, 3, -PI / 2.0);
        if ((ctx.time * 8.0) as i32).rem_euclid(2) == 0 {
            draw::dot_i(grid, ship_cx, ship_cy + ship_r + 1);
            draw::dot_i(grid, ship_cx - 1, ship_cy + ship_r + 2);
            draw::dot_i(grid, ship_cx + 1, ship_cy + ship_r + 2);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Missile Command — warheads fall on six cities; counter-missile fireballs
// ---------------------------------------------------------------------------
struct MissileCommand;

/// Dotted straight trail from `(x0, y0)` to `(x1, y1)`; `every` = dot spacing.
fn trail(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, every: i32) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for i in (0..=steps).filter(|i| i % every.max(1) == 0) {
        let t = i as f32 / steps as f32;
        draw::dot_i(
            grid,
            x0 + ((x1 - x0) as f32 * t).round() as i32,
            y0 + ((y1 - y0) as f32 * t).round() as i32,
        );
    }
}

impl ProgressStyle for MissileCommand {
    fn name(&self) -> &str {
        "missile-command"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Missile Command: enemy warheads streak down toward six cities while the central base's counter-missiles burst into fireballs; incoming missiles destroyed = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Ground, six cities and the central missile base.
        draw::hline(grid, 0, w - 1, h - 1);
        let city_x = [0.06, 0.18, 0.30, 0.62, 0.74, 0.86].map(|f: f32| (f * wi as f32) as i32);
        for &cx in &city_x {
            sprite(grid, cx, hi - 4, &[".#..#..", "##.###.", "#######"]);
        }
        let base = (wi / 2, hi - 4);
        sprite(
            grid,
            base.0 - 5,
            base.1,
            &["...###....", ".#######..", "##########"],
        );

        // Six incoming warheads; each one destroyed becomes a fireball,
        // linked to the base by its counter-missile's trail.
        let killed = ctx.eased * 6.0;
        for (k, &city) in city_x.iter().enumerate() {
            let sx = (hash01(k as u32 * 3 + 5) * wi as f32) as i32;
            let (tx, ty) = (city + 3, hi - 5);
            if (k as f32) < killed {
                let fx = sx + (tx - sx) * (2 + (k as i32 % 3)) / 8;
                let fy = 2 + (k as i32 % 3) * 2;
                trail(grid, base.0, base.1, fx, fy, 3);
                let r = 1.0 + 2.5 * tri(ctx.time * 0.9 + k as f32 * 0.37);
                for s in 0..12 {
                    let a = s as f32 * PI / 6.0;
                    draw::dot_i(
                        grid,
                        fx + (a.cos() * r * 1.2).round() as i32,
                        fy + (a.sin() * r).round() as i32,
                    );
                }
            } else {
                let u = (ctx.time * 0.3 + k as f32 * 0.21).fract();
                let hx = sx + ((tx - sx) as f32 * u) as i32;
                let hy = (ty as f32 * u) as i32;
                trail(grid, sx, 0, hx, hy, 1);
                block(grid, hx, hy, 2, 2);
            }
        }
        Ok(())
    }
}
// ---------------------------------------------------------------------------
// 5. Centipede — a segmented bug snakes through a mushroom field; shot = eased
// ---------------------------------------------------------------------------
struct Centipede;
impl ProgressStyle for Centipede {
    fn name(&self) -> &str {
        "centipede"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Centipede: a segmented bug with antennae wiggles through a mushroom field while the gnome shoots it down; segments shot = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Mushroom field: a fixed scatter of caps on stems.
        const MUSHROOM: [&str; 3] = ["###", "###", ".#."];
        let field = (hi - 8).max(1) as f32;
        for k in 0..(wi / 11).max(1) as u32 {
            let mx = (hash01(k * 2 + 1) * (wi - 4).max(1) as f32) as i32;
            let my = 1 + (hash01(k * 2 + 2) * field) as i32;
            sprite(grid, mx, my, &MUSHROOM);
        }

        // The centipede: round segments trailing a head along a wiggling path.
        // Ten segments to start; each 10% of progress shoots one off the tail.
        const SEGMENT: [&str; 4] = [".##.", "####", "####", ".##."];
        const HEAD: [&str; 5] = ["#..#", ".##.", "####", "####", ".##."];
        let alive = ((1.0 - ctx.eased) * 10.0).ceil() as i32;
        let lane = (hi / 3).max(1) as f32;
        let path = |s: f32| {
            let x = s.rem_euclid((wi + 48) as f32) as i32 - 8;
            let y = (lane + (s * 0.35).sin() * lane * 0.8) as i32;
            (x, y)
        };
        let head_s = ctx.time * 14.0;
        for i in (1..alive).rev() {
            let (x, y) = path(head_s - i as f32 * 5.0);
            sprite(grid, x, y, &SEGMENT);
        }
        if alive > 0 {
            let (x, y) = path(head_s);
            sprite(grid, x, y - 1, &HEAD);
        }

        // The gnome shooter at the bottom, firing a dashed laser upward.
        let gx = (wi / 2) + ((ctx.time * 1.3).sin() * wi as f32 * 0.3) as i32;
        sprite(grid, gx - 1, hi - 3, &[".#.", "###", "#.#"]);
        let shot = hi - 5 - ((ctx.time * 40.0) as i32).rem_euclid((hi - 4).max(1));
        draw::dot_i(grid, gx, shot);
        draw::dot_i(grid, gx, shot + 1);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Adventure — the square hero carries the sword to the castle; dragon chases
// ---------------------------------------------------------------------------
struct Adventure;
impl ProgressStyle for Adventure {
    fn name(&self) -> &str {
        "adventure"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Adventure: the square hero carries the arrow-sword toward a crenellated castle, a duck-like dragon on his heels; the portcullis rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // The castle: two crenellated towers, a battlemented wall, a gate.
        const CASTLE: [&str; 12] = [
            "#.#.#.....#.#.#",
            "#####.....#####",
            "#####.....#####",
            "#####.#.#.#####",
            "###############",
            "###############",
            "###############",
            "#####.....#####",
            "#####.....#####",
            "#####.....#####",
            "#####.....#####",
            "#####.....#####",
        ];
        let cx = wi - 16;
        let cy = hi - CASTLE.len() as i32;
        sprite(grid, cx, cy, &CASTLE);

        // Portcullis bars, lifting as the hero nears the gate.
        let gate_rows = 5;
        let lowered = gate_rows - (ctx.eased * gate_rows as f32).round() as i32;
        for r in 0..lowered {
            for gx in [cx + 6, cx + 8] {
                draw::dot_i(grid, gx, hi - gate_rows + r);
            }
        }

        // The hero (a square) carrying the arrow-shaped sword.
        let ground = hi - 4;
        let travel = (cx - 12).max(0) as f32;
        let hx = 2 + (ctx.eased * travel) as i32;
        block(grid, hx, ground, 3, 3);
        sprite(grid, hx + 4, ground, &["..#.", "####", "..#."]);

        // Yorgle the dragon waddles behind, jaw snapping with time.
        let jaw = if (ctx.time * 3.0).fract() < 0.5 {
            "##...."
        } else {
            "####.."
        };
        let dragon = [
            ".##...", "###...", jaw, ".##...", ".###..", "#####.", "######", ".####.", ".#..#.",
        ];
        let bob = ((ctx.time * 6.0).sin() * 1.5) as i32;
        sprite_flip(grid, hx - 14 + bob, hi - 10, &dragon);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Pitfall — Harry swings a vine over a tar pit; jungle scrolls with eased
// ---------------------------------------------------------------------------
struct Pitfall;
impl ProgressStyle for Pitfall {
    fn name(&self) -> &str {
        "pitfall"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Pitfall!: Harry swings on a vine over a tar pit under the jungle canopy; trees scroll by and the underground tunnel fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 4;

        // Leafy canopy along the top.
        for x in 0..wi {
            if hash01(x as u32 + 7) > 0.25 {
                draw::dot_i(grid, x, 0);
            }
            if hash01(x as u32 + 301) > 0.55 {
                draw::dot_i(grid, x, 1);
            }
        }

        // Tree trunks drifting past as Harry advances screen by screen.
        let pit = (wi / 2 - 13, wi / 2 + 13);
        let scroll = (ctx.eased * 64.0) as i32;
        let pitch = 22;
        for k in 0..=(wi / pitch + 1) {
            let tx = (k * pitch - scroll).rem_euclid(wi.max(1) + pitch) - 4;
            if tx + 2 < pit.0 || tx > pit.1 {
                block(grid, tx, 2, 2, (ground - 2).max(0));
            }
        }

        // Jungle floor (two dots thick) broken by the tar pit.
        for x in 0..wi {
            if x < pit.0 || x > pit.1 {
                draw::dot_i(grid, x, ground);
                draw::dot_i(grid, x, ground + 1);
            } else if (x + (ctx.time * 6.0) as i32) % 3 != 0 {
                // Tar shimmer.
                draw::dot_i(grid, x, ground + 1);
            }
        }

        // Underground tunnel: the progress fill along the bottom row.
        let fill = (ctx.eased * w as f32) as i32;
        if fill > 0 {
            block(grid, 0, hi - 1, fill, 1);
        }

        // The vine, pivoting from above the canopy.
        let pivot = (wi / 2, -6);
        let len = (ground - 5 - pivot.1).max(1);
        let theta = 0.6 * (ctx.time * 2.2).sin();
        let (s, c) = theta.sin_cos();
        for k in 0..=len {
            draw::dot_i(
                grid,
                pivot.0 + (k as f32 * s).round() as i32,
                pivot.1 + (k as f32 * c).round() as i32,
            );
        }

        // Harry hangs from the end: arms up, hat, torso, running legs.
        let end_x = pivot.0 + (len as f32 * s).round() as i32;
        let end_y = pivot.1 + (len as f32 * c).round() as i32;
        sprite(grid, end_x - 1, end_y, &["#.#", "###", ".#.", "###", "#.#"]);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Combat — two top-down tanks duel past barrier walls; hits = progress
// ---------------------------------------------------------------------------
struct Combat;
impl ProgressStyle for Combat {
    fn name(&self) -> &str {
        "combat"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Combat: two treaded tanks face off across barrier walls trading shells; the score digits count hits as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Top-down tank: treads, hull and a long barrel pointing right.
        const TANK: [&str; 5] = [
            "#####....",
            ".###.....",
            ".#########",
            ".###.....",
            "#####....",
        ];
        let ty = (hi - 5) / 2 + 2;
        let roll = ((ctx.time * 1.7).sin() * 1.2) as i32;
        let lx = 2 + roll;
        let rx = wi - 12 - roll;
        sprite(grid, lx, ty, &TANK);
        sprite_flip(grid, rx, ty, &TANK);

        // Barrier walls above and below the firing lane.
        let mid = wi / 2;
        block(grid, mid - 1, 6, 3, (ty - 6).max(0));
        block(grid, mid - 1, ty + 5, 3, (hi - ty - 5).max(0));

        // Shell: flies muzzle to muzzle, bursting on the far tank.
        let phase = (ctx.time / 1.2).fract();
        let (from, to) = (lx + 10, rx - 1);
        let shell_y = ty + 2;
        if phase < 0.75 {
            let sx = from + ((phase / 0.75) * (to - from) as f32) as i32;
            block(grid, sx, shell_y, 2, 1);
        } else {
            let r = 1 + ((phase - 0.75) * 12.0) as i32;
            for k in 0..8 {
                let a = k as f32 * PI / 4.0;
                draw::dot_i(
                    grid,
                    rx + 4 + (a.cos() * r as f32 * 1.5) as i32,
                    shell_y + (a.sin() * r as f32) as i32,
                );
            }
        }

        // Scores along the top: hits landed climb with progress.
        let hits = ((ctx.progress * 9.0).round() as usize).min(9);
        digit(grid, wi / 4, 0, hits, 2);
        digit(grid, wi * 3 / 4 - 6, 0, hits / 4, 2);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Kaboom — the Mad Bomber drops lit bombs; stacked buckets catch them
// ---------------------------------------------------------------------------
struct Kaboom;
impl ProgressStyle for Kaboom {
    fn name(&self) -> &str {
        "kaboom"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Kaboom!: the grinning Mad Bomber paces the wall dropping lit bombs into two stacked water buckets; the catch counter = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // The Mad Bomber paces along the top wall.
        let bomber_x = |t: f32| wi / 2 + 6 + ((t * 1.4).sin() * wi as f32 * 0.28) as i32;
        const BOMBER: [&str; 5] = [".###.", "#####", "#.#.#", ".###.", "#####"];
        let bx = bomber_x(ctx.time);
        sprite(grid, bx - 2, 0, &BOMBER);
        draw::hline(grid, 0, w - 1, 5);

        // Lit bombs, one released every 0.45 s from wherever he stood.
        let drop = 0.3;
        let speed = 11.0;
        let now = (ctx.time / drop).floor() as i32;
        let mut lowest: Option<(i32, i32)> = None;
        for k in 0..4 {
            let released = (now - k) as f32 * drop;
            let y = 6 + ((ctx.time - released) * speed) as i32;
            if y > hi - 5 {
                continue;
            }
            let x = bomber_x(released);
            sprite(grid, x - 1, y, &[".#.", "###", ".#."]);
            if (ctx.time * 8.0 + k as f32).fract() < 0.5 {
                draw::dot_i(grid, x + 1, y - 1); // fuse spark
            }
            match lowest {
                Some((_, ly)) if ly >= y => {}
                _ => lowest = Some((x, y)),
            }
        }

        // Two stacked water buckets slide under the lowest bomb.
        let target = lowest.map_or_else(|| bomber_x(ctx.time), |(x, _)| x);
        let ux = (target - 4).clamp(0, (wi - 8).max(0));
        for b in 0..2 {
            let y = hi - 4 + b * 2;
            sprite(grid, ux, y, &["#......#", ".######."]);
        }

        // Catch counter, top-left: 00 → 99 with progress.
        let caught = ((ctx.progress * 99.0).round() as usize).min(99);
        digit(grid, 0, 0, caught / 10, 1);
        digit(grid, 4, 0, caught % 10, 1);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Yars' Revenge — the Yar nibbles the Qotile's shield; erosion = progress
// ---------------------------------------------------------------------------
struct YarsRevenge;
impl ProgressStyle for YarsRevenge {
    fn name(&self) -> &str {
        "yars-revenge"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Yars' Revenge: the winged Yar hovers by the flickering neutral zone while the Qotile's cell shield erodes; erosion = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Shield: a bulging wall of 2×2 cells, eroded in a hashed order.
        let right = wi - 6;
        for row in 0..(hi / 2) {
            let bulge = 7 - ((row * 2 + 1 - hi / 2).abs() / 2).min(6);
            for col in 0..bulge {
                let i = (row * 16 + col) as u32;
                if hash01(i + 911) >= ctx.eased {
                    block(grid, right - col * 3, row * 2, 2, 2);
                }
            }
        }

        // The Qotile: a pulsing diamond behind the shield.
        let qy = hi / 2 - 2;
        let pulse = (ctx.time * 4.0).fract() < 0.5;
        let core: [&str; 4] = if pulse {
            [".#.", "###", "###", ".#."]
        } else {
            ["...", ".#.", ".#.", "..."]
        };
        sprite(grid, wi - 3, qy, &core);

        // Neutral zone: a flickering band of static.
        let frame = (ctx.time * 12.0) as u32;
        let zx = wi * 2 / 5;
        for y in 0..hi {
            for x in zx..zx + 6 {
                if hash01((y * 64 + x) as u32 ^ frame.wrapping_mul(7919)) > 0.62 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // The Yar: a fly with flapping wings, drifting toward the shield.
        let wings: [&str; 2] = if (ctx.time * 6.0).fract() < 0.5 {
            ["#...#", "##.##"]
        } else {
            ["....#", ".#.##"]
        };
        let yx = 4 + (ctx.eased * (zx - 12).max(0) as f32) as i32;
        let yy = (hi / 2 - 3) + ((ctx.time * 2.0).sin() * (hi as f32 / 4.0)) as i32;
        sprite(grid, yx, yy, &wings);
        sprite(grid, yx, yy + 2, &[".###.", "#####", ".###.", "..#.."]);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Lunar Lander — vector lander descends; fuel/altitude gauge on the side
// ---------------------------------------------------------------------------
struct LunarLander;
impl ProgressStyle for LunarLander {
    fn name(&self) -> &str {
        "lunar-lander"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Lunar Lander: a vector wireframe lander descends; altitude/fuel = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Altitude gauge: vertical bar on the far right (2 dots wide).
        let gauge_x = w.saturating_sub(2);
        let gauge_h = h;
        let fuel_h = (ctx.eased * gauge_h as f32) as usize;
        // Empty gauge outline.
        draw::vline(grid, gauge_x, 0, gauge_h.saturating_sub(1));
        // Fuel fill from bottom.
        for gy in (gauge_h.saturating_sub(fuel_h))..gauge_h {
            draw::dot(grid, gauge_x + 1, gy);
        }

        // Lunar surface: irregular terrain at the bottom.
        let surface_y = h.saturating_sub(2);
        draw::hline(grid, 0, gauge_x.saturating_sub(1), surface_y);
        // Jagged peaks.
        let peak_xs = [w / 6, w / 3, w / 2, 2 * w / 3];
        for &px in &peak_xs {
            if px < gauge_x {
                draw::vline(grid, px, surface_y.saturating_sub(2), surface_y);
            }
        }

        // Lander: descends from top to surface as eased rises.
        // Wireframe: body (rectangle) + legs (two diagonal lines) + thruster dot.
        let lander_col_centre = (w / 2) as i32;
        let descent_range = surface_y.saturating_sub(6);
        let lander_y = (ctx.eased * descent_range as f32) as i32;

        let bw: i32 = (w as i32 / 8).clamp(2, 5);
        let bh: i32 = 2.max((h as i32 / 8).min(3));

        // Body rectangle.
        let bx0 = lander_col_centre - bw / 2;
        let bx1 = lander_col_centre + bw / 2;
        let by0 = lander_y;
        let by1 = lander_y + bh;
        // Top and bottom of body.
        for x in bx0..=bx1 {
            draw::dot_i(grid, x, by0);
            draw::dot_i(grid, x, by1);
        }
        // Sides.
        for y in by0..=by1 {
            draw::dot_i(grid, bx0, y);
            draw::dot_i(grid, bx1, y);
        }
        // Landing legs: diagonal lines down from body corners.
        let leg_len: i32 = bh.max(2);
        draw::dot_i(grid, bx0 - leg_len, by1 + leg_len);
        draw::dot_i(grid, bx0 - leg_len + 1, by1 + leg_len - 1);
        draw::dot_i(grid, bx0 - leg_len + 2, by1 + leg_len - 2);
        draw::dot_i(grid, bx1 + leg_len, by1 + leg_len);
        draw::dot_i(grid, bx1 + leg_len - 1, by1 + leg_len - 1);
        draw::dot_i(grid, bx1 + leg_len - 2, by1 + leg_len - 2);

        // Thruster exhaust: a pulsing dot below the body (only when descending).
        if ctx.eased < 0.95 {
            let pulse = ((ctx.time * 8.0).sin() > 0.0) as i32;
            draw::dot_i(grid, lander_col_centre, by1 + 1 + pulse);
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
    let styles = progress::styles::atari::styles();
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
