//! `chaos` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O chaos.rs && ./chaos [style-name]
//! ```

const DEFAULT_STYLE: &str = "lorenz";

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
    pub mod chaos {
//! Chaos / strange-attractor progress bars.
//!
//! Each style implements a real dynamical system from the literature, iterating
//! from a fixed seed each frame (stateless) so the bar is a pure function of
//! `(progress, time)`. Attractor orbits are mapped into dot-space coordinates
//! and revealed progressively via `ctx.eased`. Parameter animation uses
//! `ctx.time` to drive slow rotations, phase-shifts, and bifurcations.
//!
//! Systems implemented:
//! - **Lorenz** — the canonical butterfly attractor (σ=10, ρ=28, β=8/3).
//! - **Rössler** — the tilted spiral band with its fold-over lobe (a=0.2, b=0.2, c=5.7).
//! - **Clifford** — three classic parameter sets, density-rendered side by side.
//! - **De Jong** — three classic parameter sets, density-rendered side by side.
//! - **Hénon map** — the folded boomerang attractor (a=1.4, b=0.3).
//! - **Logistic bifurcation** — bifurcation diagram swept across the bar.
//! - **Double pendulum** — two rods and bobs from a pivot riding the progress rail.
//! - **Standard map** — phase portrait whose KAM curves break up as K rises.
//! - **Gingerbreadman map** — the chaotic orbit from (−0.1, 0) filling its silhouette.
//! - **Duffing oscillator** — phase portrait of the forced anharmonic well.
//! - **Tinkerbell map** — the classic wand-trail attractor.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Theme tint — hot pink into violet.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 84, 130);
const TINT_END: Color = Color::rgb(128, 44, 212);

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

/// All styles in the `chaos` theme.
///
/// Returns 11 dynamical-system progress bars. Every style is stateless and
/// renders from a fixed seed each frame; no mutable state is held.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(LorenzAttractor)),
        Box::new(Tinted(RosslerAttractor)),
        Box::new(Tinted(CliffordAttractor)),
        Box::new(Tinted(DeJongAttractor)),
        Box::new(Tinted(HenonMap)),
        Box::new(Tinted(LogisticBifurcation)),
        Box::new(Tinted(DoublePendulum)),
        Box::new(Tinted(StandardMap)),
        Box::new(Tinted(Gingerbreadman)),
        Box::new(Tinted(DuffingOscillator)),
        Box::new(Tinted(TinkerbellMap)),
    ]
}

// ---------------------------------------------------------------------------
// 1. Lorenz Attractor
//    dx/dt = σ(y − x)        σ = 10
//    dy/dt = x(ρ − z) − y    ρ = 28
//    dz/dt = xy − βz         β = 8/3
//
//    Progress reveals orbit length. Time swings the view about the z axis.
// ---------------------------------------------------------------------------

struct LorenzAttractor;

/// Bresenham-style dot line between two signed dot coordinates.
fn seg(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 512);
    for i in 0..=n {
        let x = x0 + (x1 - x0) * i / n;
        let y = y0 + (y1 - y0) * i / n;
        draw::dot_i(grid, x, y);
    }
}

impl ProgressStyle for LorenzAttractor {
    fn name(&self) -> &str {
        "lorenz"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Lorenz butterfly attractor (σ=10 ρ=28 β=8/3) seen in the x–z plane: the orbit loops \
         round both wings as progress lengthens it, and the view swings about the z axis with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (sigma, rho, beta) = (10.0f32, 28.0f32, 8.0f32 / 3.0);
        let dt = 0.01f32;
        let (mut x, mut y, mut z) = (1.0f32, 1.0f32, 20.0f32);
        let step = |x: &mut f32, y: &mut f32, z: &mut f32| {
            // Midpoint (RK2) step — stable at dt = 0.01.
            let (dx, dy, dz) = (sigma * (*y - *x), *x * (rho - *z) - *y, *x * *y - beta * *z);
            let (mx, my, mz) = (*x + dx * dt * 0.5, *y + dy * dt * 0.5, *z + dz * dt * 0.5);
            *x += sigma * (my - mx) * dt;
            *y += (mx * (rho - mz) - my) * dt;
            *z += (mx * my - beta * mz) * dt;
        };
        // Burn in so the orbit starts on the attractor.
        for _ in 0..200 {
            step(&mut x, &mut y, &mut z);
        }
        // Gentle swing of the camera about the z axis.
        let th = (ctx.time * 0.6).sin() * 0.25;
        let (c, s) = (th.cos(), th.sin());
        // Wings span |x| ≲ 20, z ∈ [3, 48]; stretch x to use the wide bar.
        let sx = (dw as f32 * 0.3) / 22.0;
        let sz = (dh as f32 - 1.0) / 46.0;
        let cx = dw as f32 * 0.5;
        let proj = |x: f32, y: f32, z: f32| {
            let u = x * c + y * s;
            ((cx + u * sx) as i32, ((48.0 - z) * sz) as i32)
        };
        let steps = (80.0 + ctx.eased * 1100.0) as u32;
        let mut prev = proj(x, y, z);
        for _ in 0..steps {
            step(&mut x, &mut y, &mut z);
            let p = proj(x, y, z);
            seg(grid, prev.0, prev.1, p.0, p.1);
            prev = p;
        }
        // Bright head on the orbit's leading point.
        draw::fill_rect(grid, prev.0.max(0) as usize, prev.1.max(0) as usize, 2, 2);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Rössler Attractor
//    dx/dt = −y − z
//    dy/dt = x + ay          a = 0.2
//    dz/dt = b + z(x − c)   b = 0.2, c = 5.7
//
//    Oblique view: the x–y spiral band seen tilted, z lifting the fold-over
//    lobe above it. Progress reveals orbit length; time swings the view.
// ---------------------------------------------------------------------------

struct RosslerAttractor;

impl ProgressStyle for RosslerAttractor {
    fn name(&self) -> &str {
        "rossler"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Rössler attractor (a=b=0.2, c=5.7) seen from above and a little to the side: the \
         orbit winds outward round a tilted spiral band, then lifts into the fold-over lobe \
         and drops back to the centre; progress lengthens the orbit, time turns the view"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let a: f32 = 0.2;
        let b: f32 = 0.2;
        let c: f32 = 5.7;
        let dt: f32 = 0.03;

        let n_total = 1500usize;
        let n_draw = ((ctx.eased * n_total as f32) as usize).clamp(2, n_total);

        // View: swing a little about the z axis, tilt so the disc is an ellipse
        // and the z excursion (the fold) rises above it.
        let yaw = 0.35 * (ctx.time * 0.5).sin();
        let (cy_, sy_) = (yaw.cos(), yaw.sin());
        // Fit: horizontal extent ≈ 24 units, vertical ≈ disc depth + lobe.
        let hs = (dw as f32 - 2.0) / 24.0;
        let hs = hs.min(dh as f32 * 0.17).max(0.1);
        let vs = (dh as f32 - 2.0) / 22.0;
        let ox = dw as f32 * 0.5;
        let base = dh as f32 * 0.62;

        let (mut x, mut y, mut z) = (1.0f32, 0.0f32, 0.0f32);
        for _ in 0..300 {
            let (dx, dy, dz) = (-y - z, x + a * y, b + z * (x - c));
            x += dx * dt;
            y += dy * dt;
            z += dz * dt;
        }
        let project = |x: f32, y: f32, z: f32| -> (i32, i32) {
            let rx = x * cy_ - y * sy_;
            let ry = x * sy_ + y * cy_;
            let sx = ox + rx * hs;
            let sy = base - ry * vs * 0.55 - z * vs * 0.55;
            (sx.round() as i32, sy.round() as i32)
        };
        let mut prev = project(x, y, z);
        for _ in 0..n_draw {
            let (dx, dy, dz) = (-y - z, x + a * y, b + z * (x - c));
            x += dx * dt;
            y += dy * dt;
            z += dz * dt;
            let p = project(x, y, z);
            seg(grid, prev.0, prev.1, p.0, p.1);
            prev = p;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Clifford Attractor (IFS)
//    x_{n+1} = sin(a·y) + c·cos(a·x)
//    y_{n+1} = sin(b·x) + d·cos(b·y)
//
//    Three classic (a,b,c,d) sets; a breathes with time. Density-rendered.
// ---------------------------------------------------------------------------

struct CliffordAttractor;

/// Shared renderer for 2-D iterated-map attractors.
///
/// Lays out `panels` copies of the attractor across the bar (each its own
/// parameter set, chosen by `map`'s panel index), developing them left to
/// right: panel `k` iterates `300 + local·5000` points where `local` is its
/// share of progress. Each panel's bounding box is measured from a short
/// warm-up orbit, then hits accumulate in a density buffer and only cells
/// visited often enough are lit — so the attractor's filaments read as lines
/// rather than a solid blob. `aspect` caps a panel's width at `aspect × height`.
fn attractor_row(
    grid: &mut BrailleGrid,
    ctx: &BarContext,
    panels: usize,
    aspect: f32,
    seed: impl Fn(usize) -> (f32, f32),
    map: impl Fn(usize, f32, f32) -> (f32, f32),
) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw < 2 || dh < 2 {
        return;
    }
    let n = if dw >= dh * 4 { panels.max(1) } else { 1 };
    let pw = dw / n;
    let bw = ((dh as f32 * aspect) as usize)
        .min(pw.saturating_sub(2))
        .max(1);
    let mut hits = vec![0u8; bw * dh];
    for k in 0..n {
        let x0 = if n == 1 {
            // A lone figure travels along the bar with progress, leaving a dotted wake.
            let x0 = ((dw - bw) as f32 * ctx.eased) as usize;
            for x in (0..x0).step_by(3) {
                draw::dot(grid, x, dh - 1);
            }
            x0
        } else {
            k * pw + (pw - bw) / 2
        };
        let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
        if local <= 0.0 {
            // Waiting panel: a dotted baseline marks its slot.
            for x in (x0..x0 + bw).step_by(3) {
                draw::dot(grid, x, dh - 1);
            }
            continue;
        }
        let (mut x, mut y) = seed(k);
        let (mut lo_x, mut hi_x, mut lo_y, mut hi_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for i in 0..900 {
            (x, y) = map(k, x, y);
            if !x.is_finite() || !y.is_finite() || x.abs() > 1e4 || y.abs() > 1e4 {
                break;
            }
            if i >= 100 {
                lo_x = lo_x.min(x);
                hi_x = hi_x.max(x);
                lo_y = lo_y.min(y);
                hi_y = hi_y.max(y);
            }
        }
        if !(lo_x < hi_x && lo_y < hi_y) {
            continue;
        }
        let (sx, sy) = (
            (bw - 1) as f32 / (hi_x - lo_x).max(1e-3),
            (dh - 1) as f32 / (hi_y - lo_y).max(1e-3),
        );
        hits.fill(0);
        let count = (300.0 + local * 5000.0) as usize;
        for _ in 0..count {
            (x, y) = map(k, x, y);
            let px = ((x - lo_x) * sx).round();
            let py = ((hi_y - y) * sy).round();
            if px >= 0.0 && py >= 0.0 && (px as usize) < bw && (py as usize) < dh {
                let h = &mut hits[py as usize * bw + px as usize];
                *h = h.saturating_add(1);
            }
        }
        let thr = (1 + count / 1000) as u8;
        for py in 0..dh {
            for px in 0..bw {
                if hits[py * bw + px] >= thr {
                    draw::dot(grid, x0 + px, py);
                }
            }
        }
    }
}

impl ProgressStyle for CliffordAttractor {
    fn name(&self) -> &str {
        "clifford"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Three Clifford attractors (x′=sin ay+c·cos ax, y′=sin bx+d·cos by) from classic parameter \
         sets, each densifying from a sprinkle into its swirling filaments as progress reaches it; \
         parameters breathe with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        const SETS: [[f32; 4]; 3] = [
            [-1.4, 1.6, 1.0, 0.7],
            [1.7, 1.7, 0.6, 1.2],
            [-1.7, 1.3, -0.1, -1.2],
        ];
        let t = ctx.time;
        attractor_row(
            grid,
            ctx,
            3,
            1.7,
            |_| (0.1, 0.1),
            |k, x, y| {
                let [a, b, c, d] = SETS[k % 3];
                let a = a + 0.06 * (t * 0.4 + k as f32).sin();
                (
                    (a * y).sin() + c * (a * x).cos(),
                    (b * x).sin() + d * (b * y).cos(),
                )
            },
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. De Jong Attractor (IFS)
//    x_{n+1} = sin(a·y) − cos(b·x)
//    y_{n+1} = sin(c·x) − cos(d·y)
//
//    Three classic parameter sets; a breathes with time. Density-rendered.
// ---------------------------------------------------------------------------

struct DeJongAttractor;

impl ProgressStyle for DeJongAttractor {
    fn name(&self) -> &str {
        "de-jong"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Three Peter de Jong attractors (x′=sin ay−cos bx, y′=sin cx−cos dy) from classic \
         parameter sets, densifying into their folded ribbons one after another with progress; \
         parameters breathe with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        const SETS: [[f32; 4]; 3] = [
            [1.4, -2.3, 2.4, -2.1],
            [-2.7, -0.09, -0.86, -2.2],
            [-0.827, -1.637, 1.659, -0.943],
        ];
        let t = ctx.time;
        attractor_row(
            grid,
            ctx,
            3,
            1.7,
            |_| (0.1, 0.1),
            |k, x, y| {
                let [a, b, c, d] = SETS[k % 3];
                let a = a + 0.05 * (t * 0.35 + k as f32).sin();
                ((a * y).sin() - (b * x).cos(), (c * x).sin() - (d * y).cos())
            },
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Hénon Map
//    x_{n+1} = 1 − a·x² + y    a = 1.4
//    y_{n+1} = b·x              b = 0.3
//
//    Classic boomerang-shaped strange attractor. Progress = points iterated
//    and the figure's position along the bar; a breathes with time.
// ---------------------------------------------------------------------------

struct HenonMap;

impl ProgressStyle for HenonMap {
    fn name(&self) -> &str {
        "henon"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Hénon map (a=1.4, b=0.3): the boomerang-shaped strange attractor with its parallel folded \
         bands, densifying from scattered points as progress iterates it; a breathes with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let a = 1.4 - 0.01 * (1.0 + (ctx.time * 0.5).sin());
        attractor_row(
            grid,
            ctx,
            1,
            3.6,
            |_| (0.1, 0.1),
            |_, x, y| (1.0 - a * x * x + y, 0.3 * x),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Logistic Map Bifurcation Diagram
//    x_{n+1} = r · x · (1 − x),   x₀ = 0.5
//
//    The x-axis is the bar: r swept from 2.8 to 4.0 left→right.
//    ctx.eased gates how far right r has been revealed.
//    At each r column we iterate 400+ steps (discard) + 8 (plot), enough to
//    show every branch of periods 1, 2, 4, 8 and the period-3 window.
//    The y-axis is x (attractor values). Time shifts which stretch of the
//    orbit is sampled, so the chaotic band shimmers.
//
//    This is the most gorgeous bar — the entire period-doubling cascade
//    is drawn column by column as progress advances.
// ---------------------------------------------------------------------------

struct LogisticBifurcation;

impl ProgressStyle for LogisticBifurcation {
    fn name(&self) -> &str {
        "logistic-bifurcation"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Logistic-map bifurcation diagram: r runs 2.8→4.0 left to right, the long-run values \
         of x stacked up each column — one branch forks into 2, 4, 8 and dissolves into \
         chaos, with the period-3 window near r≈3.83; progress sweeps r, time stirs the chaos"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let r_min: f32 = 2.8;
        let r_max: f32 = 4.0;
        let reveal_cols = ((ctx.eased * dw as f32).round() as usize).min(dw);
        // Time picks which stretch of the long-run orbit is sampled, so the
        // chaotic band shimmers while periodic branches stay put.
        let skip = (ctx.time * 12.0).max(0.0) as usize % 64;
        // Sparse samples keep the chaos speckled so windows and forks read.
        // Eight samples cover every branch of periods 1, 2, 4, 8 and 3.
        let samples = 8usize;

        for col in 0..reveal_cols {
            let r = r_min + (col as f32 / dw.saturating_sub(1).max(1) as f32) * (r_max - r_min);
            let mut x = 0.5f32;
            for _ in 0..(400 + skip) {
                x = r * x * (1.0 - x);
            }
            for _ in 0..samples {
                x = r * x * (1.0 - x);
                let row = ((1.0 - x.clamp(0.0, 1.0)) * (dh - 1) as f32).round() as usize;
                draw::dot(grid, col, row);
            }
        }
        // Leading edge: the column being computed now.
        if reveal_cols > 0 && reveal_cols < dw && (ctx.time * 4.0).sin() > 0.0 {
            for y in (0..dh).step_by(3) {
                draw::dot(grid, reveal_cols, y);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Double Pendulum Trace
//    Lagrangian equations (equal masses m=1, equal lengths l=1, g=9.81):
//
//    θ̈₁ = [−g(2m)sinθ₁ − mg·sin(θ₁−2θ₂) − 2sin(θ₁−θ₂)·m(θ̇₂²l+θ̇₁²l·cos(θ₁−θ₂))]
//           / [l(2m − m·cos(2θ₁−2θ₂))]
//
//    θ̈₂ = [2sin(θ₁−θ₂)(θ̇₁²l·2m + g·2m·cosθ₁ + θ̇₂²l·m·cos(θ₁−θ₂))]
//           / [l(2m − m·cos(2θ₁−2θ₂))]
//
//    Semi-implicit Euler from rest, replayed each frame (24 s loop).
//    Progress moves the pivot along the rail; the lower bob leaves a trail.
// ---------------------------------------------------------------------------

struct DoublePendulum;

impl ProgressStyle for DoublePendulum {
    fn name(&self) -> &str {
        "double-pendulum"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "A real double pendulum (two rods, two bobs) swinging chaotically from a pivot that \
         rides along the top rail with progress, the lower bob trailing its recent path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Equal masses and rod lengths; full Lagrangian equations of motion.
        let g = 9.81f32;
        let dt = 0.004f32;
        let (mut a1, mut a2, mut w1, mut w2) = (1.35f32, 2.2f32, 0.0f32, 0.0f32);
        let accel = |a1: f32, a2: f32, w1: f32, w2: f32| {
            let d = a1 - a2;
            let den = 3.0 - (2.0 * d).cos();
            let n1 = -3.0 * g * a1.sin()
                - g * (a1 - 2.0 * a2).sin()
                - 2.0 * d.sin() * (w2 * w2 + w1 * w1 * d.cos());
            let n2 = 2.0 * d.sin() * (2.0 * w1 * w1 + 2.0 * g * a1.cos() + w2 * w2 * d.cos());
            (n1 / den, n2 / den)
        };
        // Stateless: replay from rest each frame; loop every 24 s to bound cost.
        let t = ctx.time.max(0.0) % 24.0;
        let total = (t / dt) as u32;
        let trail = 160u32;
        let rail = 1i32;
        let len = ((dh as f32 - 2.0) / 2.0).max(1.0);
        // Keep the swing on screen: the pivot travels between the margins.
        let margin = (len * 3.0).min(dw as f32 * 0.25);
        let px = (margin + ctx.eased * (dw as f32 - 1.0 - 2.0 * margin)) as i32;
        let tip = |a1: f32, a2: f32| {
            let (x1, y1) = (a1.sin() * len, a1.cos() * len);
            (x1 + a2.sin() * len, y1 + a2.cos() * len)
        };
        for k in 0..total {
            // Semi-implicit Euler: velocities first, then angles.
            let (e1, e2) = accel(a1, a2, w1, w2);
            w1 += e1 * dt;
            w2 += e2 * dt;
            a1 += w1 * dt;
            a2 += w2 * dt;
            if total - k <= trail && k % 2 == 0 {
                let (tx, ty) = tip(a1, a2);
                draw::dot_i(grid, px + (tx * 2.0) as i32, rail + ty as i32);
            }
        }
        // Top rail: solid up to the pivot (progress), dotted beyond.
        for x in 0..dw as i32 {
            if x <= px || x % 3 == 0 {
                draw::dot_i(grid, x, 0);
            }
        }
        // Rods and bobs. Horizontal is doubled so the swing reads on the wide bar.
        let (x1, y1) = (a1.sin() * len * 2.0, a1.cos() * len);
        let (tx, ty) = tip(a1, a2);
        let b1 = (px + x1 as i32, rail + y1 as i32);
        let b2 = (px + (tx * 2.0) as i32, rail + ty as i32);
        seg(grid, px, rail, b1.0, b1.1);
        seg(grid, b1.0, b1.1, b2.0, b2.1);
        for (bx, by) in [b1, b2] {
            draw::dot_i(grid, bx + 1, by);
            draw::dot_i(grid, bx, by + 1);
            draw::dot_i(grid, bx + 1, by + 1);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Standard Map (Chirikov–Taylor map)
//    p_{n+1} = p_n + K·sin(θ_n)   (mod 2π)
//    θ_{n+1} = θ_n + p_{n+1}      (mod 2π)
//
//    K = stochasticity parameter. Progress ramps K from 0.3 → 1.4.
//    Seven orbits launched across the momentum axis.
//    Time scrolls the portrait in θ.
// ---------------------------------------------------------------------------

struct StandardMap;

impl ProgressStyle for StandardMap {
    fn name(&self) -> &str {
        "standard-map"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Chirikov–Taylor standard map phase portrait (θ across, p up): a fan of orbits traces \
         wavy KAM curves and island chains, and as progress raises K from 0.3 to 1.4 the curves \
         break up into a chaotic sea; the portrait scrolls in θ with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let tau = 2.0 * PI;
        let k = 0.3 + ctx.eased * 1.1;
        let scroll = ctx.time * 0.25;
        let orbits = 7u32;
        for o in 0..orbits {
            // Seeds fan out across the momentum axis.
            let mut p = tau * (o as f32 + 0.5) / orbits as f32;
            let mut th = 0.6 + o as f32 * 0.41;
            for _ in 0..110 {
                p = (p + k * th.sin()).rem_euclid(tau);
                th = (th + p).rem_euclid(tau);
                let x = ((th + scroll).rem_euclid(tau) / tau * dw as f32) as usize;
                // Shift p by π so the main resonance island sits mid-bar.
                let y = ((1.0 - (p + PI).rem_euclid(tau) / tau) * dh as f32) as usize;
                draw::dot(grid, x.min(dw - 1), y.min(dh - 1));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Gingerbreadman Map
//    x_{n+1} = 1 − y + |x|
//    y_{n+1} = x
//
//    Chaotic orbit from (−0.1, 0) filling the gingerbread-man silhouette.
//    Progress iterates more points and moves the figure along the bar.
// ---------------------------------------------------------------------------

struct Gingerbreadman;

impl ProgressStyle for Gingerbreadman {
    fn name(&self) -> &str {
        "gingerbreadman"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Gingerbreadman map (x′=1−y+|x|, y′=x) from (−0.1, 0): the orbit fills the chunky \
         hexagonal gingerbread-man silhouette, densifying as progress iterates it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let wob = 0.004 * (ctx.time * 0.7).sin();
        attractor_row(
            grid,
            ctx,
            1,
            1.25,
            |_| (-0.1 + wob, 0.0),
            |_, x, y| (1.0 - y + x.abs(), x),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Duffing Oscillator Phase Portrait
//     ẋ = y
//     ẏ = −δy + αx − βx³ + γcos(ωt_sim)
//
//     δ=0.25, α=1.0, β=1.0, γ=0.3, ω=1.0 (chaotic two-well driving).
//     Plot (x, y) = (position, velocity) phase portrait.
//     Progress controls how many trajectory steps are drawn.
//     ctx.time offsets the driving phase, ctx.eased gates steps.
// ---------------------------------------------------------------------------

struct DuffingOscillator;

impl ProgressStyle for DuffingOscillator {
    fn name(&self) -> &str {
        "duffing"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "Duffing oscillator phase portrait (ẏ=−δy+αx−βx³+γcos ωt): the forced orbit loops round both potential wells, its trace lengthening with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let delta: f32 = 0.25;
        let alpha: f32 = 1.0;
        let beta: f32 = 1.0;
        let gamma: f32 = 0.3;
        let omega: f32 = 1.0;
        let dt: f32 = 0.02;

        let n_total: usize = 4000;
        let n_draw = (ctx.eased * n_total as f32).max(1.0) as usize;

        // Phase portrait fits in roughly x∈[−1.6,1.6], y∈[−1.1,1.1]; keep it
        // to the middle of the bar so the two wells read as two lobes.
        let x_range = 1.7_f32;
        let y_range = 1.15_f32;

        let mut x: f32 = 1.0;
        let mut y: f32 = 0.0;
        // Driving phase starts offset by ctx.time.
        let mut t_sim: f32 = ctx.time * 0.5;

        for _ in 0..n_draw.min(n_total) {
            let force = gamma * (omega * t_sim).cos();
            let ax = y;
            let ay = -delta * y + alpha * x - beta * x * x * x + force;

            // Semi-implicit Euler keeps the orbit from drifting off in energy.
            y += ay * dt;
            x += y * dt;
            let _ = ax;
            t_sim += dt;

            let norm_y = (y + y_range) / (2.0 * y_range);
            let px = (dw as f32 * (0.5 + 0.3 * (x / x_range).clamp(-1.0, 1.0))) as i32;
            let py = ((1.0 - norm_y.clamp(0.0, 1.0)) * (dh - 1) as f32) as i32;
            draw::dot_i(grid, px, py);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Tinkerbell Map
//     x_{n+1} = x² − y² + a·x + b·y
//     y_{n+1} = 2xy + c·x + d·y
//
//     a=0.9, b=−0.6013, c=2.0, d=0.5
//     Named for the fractal boundary of its basin of attraction.
//     Progress iterates more points and moves the figure along the bar.
// ---------------------------------------------------------------------------

struct TinkerbellMap;

impl ProgressStyle for TinkerbellMap {
    fn name(&self) -> &str {
        "tinkerbell"
    }
    fn theme(&self) -> &str {
        "chaos"
    }
    fn describe(&self) -> &str {
        "The Tinkerbell attractor (x′=x²−y²+ax+by, y′=2xy+cx+dy; a=0.9 b=−0.6013 c=2 d=0.5): \
         its looping wand-trail shape densifies as it travels along the bar with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let wob = 0.004 * (ctx.time * 0.6).sin();
        attractor_row(
            grid,
            ctx,
            1,
            2.2,
            |_| (-0.72, -0.64),
            |_, x, y| {
                let [a, b, c, d] = [0.9, -0.6013, 2.0, 0.5];
                (
                    x * x - y * y + a * x + b * y,
                    2.0 * x * y + c * x + d * y + wob,
                )
            },
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
    let styles = progress::styles::chaos::styles();
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
