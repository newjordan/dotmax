//! `nintendo` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O nintendo.rs && ./nintendo [style-name]
//! ```

const DEFAULT_STYLE: &str = "mario-run";

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
    pub mod nintendo {
//! NES / Nintendo-classics themed progress bars.
//!
//! Each style is mechanically distinct — geometry, algorithm, motion, and
//! symbol choice all differ. Colour alone is never the only differentiator.
//!
//! Styles:
//! - `mario-run`      — Mario runs a brick road, hops a pipe, reaches the flagpole.
//! - `zelda-hearts`   — Heart-container row fills one half-heart at a time.
//! - `metroid-etanks` — Energy-tank segments charge one at a time.
//! - `tetris-well`    — A well fills upward with a settling tetromino stack.
//! - `duck-hunt`      — A mallard flaps over the grass; the HIT counter fills.
//! - `excitebike`     — A motocross rider races and jumps a ramp; TEMP gauge.
//! - `punchout-stars` — Little Mac vs. a foe in the ring; stars earned fill.
//! - `contra-spread`  — Spread-shot bullets fan out rightward.
//! - `megaman-weapon` — Mega Man fires pellets beside his tick energy meter.
//! - `iceclimber-up`  — Popo hammers through ice-brick floors; mountain gauge shows altitude.
//! - `donkey-barrel`  — DK throws barrels down girders; Mario climbs a ladder.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — cartridge red into NES blue. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(232, 64, 52);
const TINT_END: Color = Color::rgb(72, 124, 255);

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
// Sprite helpers — dot-art stamps for the pictorial styles.
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

/// Solid block of dots from signed coordinates (clipped to the grid).
fn block(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32) {
    for yy in y..y + h {
        for xx in x..x + w {
            draw::dot_i(grid, xx, yy);
        }
    }
}

/// Triangle wave `0 → 1 → 0` with period 1.
fn tri(t: f32) -> f32 {
    1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs()
}

/// All styles in the `nintendo` theme.
///
/// Returns one boxed implementor per NES game mechanic.  Every style is
/// a stateless unit struct — no heap allocation beyond the `Box`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(MarioRun),
        Box::new(Tinted(ZeldaHearts)),
        Box::new(Tinted(MetroidETanks)),
        Box::new(Tinted(TetrisWell)),
        Box::new(DuckHunt),
        Box::new(Excitebike),
        Box::new(PunchOutStars),
        Box::new(Tinted(ContraSpread)),
        Box::new(MegaManWeapon),
        Box::new(IceClimberUp),
        Box::new(DonkeyBarrel),
    ]
}

// ─── 1. Mario Run ────────────────────────────────────────────────────────────

/// Mario (capped, overalled, two-frame run cycle) runs along a brick ground,
/// hops a green warp pipe and heads for the flagpole; the flag slides down as
/// he reaches it. Each scene element gets its own NES colour.
struct MarioRun;

/// Small-Mario run frames: cap, face, shirt, overalls, legs.
const MARIO_A: [&str; 10] = [
    "..####..", ".#######", ".##.#...", ".######.", ".######.", "########", "##.##.##", ".######.",
    ".##..##.", "##....##",
];
const MARIO_B: [&str; 10] = [
    "..####..", ".#######", ".##.#...", ".######.", ".######.", "########", "##.##.##", ".######.",
    "..####..", ".###.##.",
];

impl ProgressStyle for MarioRun {
    fn name(&self) -> &str {
        "mario-run"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Mario in red cap and blue overalls runs a brick road, hops a green pipe and reaches the flagpole as progress fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 2;

        // Brick ground: top edge plus staggered mortar joints.
        draw::hline(grid, 0, w - 1, ground as usize);
        for x in (0..wi).filter(|x| x % 4 == 0) {
            draw::dot_i(grid, x + (x / 4 % 2) * 2, ground + 1);
        }

        // Green warp pipe mid-course: rim over a narrower body.
        let pipe_x = wi / 2 - 4;
        block(grid, pipe_x, ground - 6, 8, 2);
        block(grid, pipe_x + 1, ground - 4, 6, 4);

        // Flagpole: ball on top, pole, pennant sliding down at the finish.
        let pole_x = wi - 3;
        block(grid, pole_x - 1, 0, 3, 2);
        draw::vline(
            grid,
            pole_x.max(0) as usize,
            2,
            (ground - 1).max(2) as usize,
        );
        let lower = ((ctx.eased - 0.9) / 0.1).clamp(0.0, 1.0);
        let fy = 2 + (lower * (ground - 6).max(0) as f32) as i32;
        sprite(grid, pole_x - 5, fy, &["#####", ".####", "...##"]);

        // Mario: runs to the pole, hopping in an arc over the pipe.
        let mx = (ctx.eased * (pole_x - 10).max(0) as f32) as i32;
        let dx = (mx + 4 - (pipe_x + 4)) as f32 / 10.0;
        let hop = if dx.abs() < 1.0 {
            ((1.0 - dx * dx) * 7.0) as i32
        } else {
            0
        };
        let my = ground - 10 - hop;
        let frame = if (ctx.time * 8.0).fract() < 0.5 {
            &MARIO_A
        } else {
            &MARIO_B
        };
        sprite(grid, mx, my, frame);

        // Colour each cell by the scene element under its centre.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let color = if (mx..mx + 8).contains(&px) && (my..my + 10).contains(&py) {
                    if py < my + 5 {
                        Color::rgb(228, 52, 40) // cap and face
                    } else {
                        Color::rgb(64, 110, 240) // overalls
                    }
                } else if (pipe_x..pipe_x + 8).contains(&px) && py < ground {
                    Color::rgb(60, 200, 70)
                } else if px >= pole_x - 6 && py < ground {
                    Color::rgb(230, 240, 230)
                } else {
                    Color::rgb(196, 110, 50) // bricks
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 2. Zelda Hearts ─────────────────────────────────────────────────────────

/// Zelda HUD heart containers: full hearts are dense-filled, empty hearts are
/// outlines, and partial is a half-full heart.  Progress maps to half-hearts.
struct ZeldaHearts;
impl ProgressStyle for ZeldaHearts {
    fn name(&self) -> &str {
        "zelda-hearts"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Zelda heart-container row — fills one half-heart at a time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Pack as many 3-cell-wide hearts as fit, with a 1-cell gap.
        let heart_cell_w = 3usize;
        let gap = 1usize;
        let slot_w = heart_cell_w + gap;
        let n_hearts = (cw / slot_w).max(1);
        let half_hearts_total = n_hearts * 2;
        let half_filled = (ctx.eased * half_hearts_total as f32).round() as usize;

        let row = ch / 2; // draw in vertical middle

        for i in 0..n_hearts {
            let x0 = i * slot_w;
            let full_halves = (half_filled).saturating_sub(i * 2);
            // full_halves for this heart: 0 = empty, 1 = half, 2 = full
            let state = full_halves.min(2);

            // Heart outline (3 dots wide, 2 dots tall in dot-space):
            //  cols x0*2, x0*2+1, x0*2+2, x0*2+3, x0*2+4, x0*2+5
            let dx = x0 * 2;
            let (dw, dh) = draw::dot_dims(grid);
            let dy = (row * 4).min(dh.saturating_sub(3));
            // Two bumps on top row.
            if dx + 1 < dw && dy < dh {
                draw::dot(grid, dx + 1, dy);
            }
            if dx + 4 < dw && dy < dh {
                draw::dot(grid, dx + 4, dy);
            }
            // Wider middle: outline edges only, so the fill state reads.
            if dy + 1 < dh {
                if dx < dw {
                    draw::dot(grid, dx, dy + 1);
                }
                if dx + 5 < dw {
                    draw::dot(grid, dx + 5, dy + 1);
                }
            }
            // Taper bottom: outline edges only.
            if dy + 2 < dh {
                if dx + 1 < dw {
                    draw::dot(grid, dx + 1, dy + 2);
                }
                if dx + 4 < dw {
                    draw::dot(grid, dx + 4, dy + 2);
                }
            }
            if dx + 2 < dw && dy + 3 < dh {
                draw::dot(grid, dx + 2, dy + 3);
            }
            if dx + 3 < dw && dy + 3 < dh {
                draw::dot(grid, dx + 3, dy + 3);
            }

            // Fill interior based on state.
            if state == 2 {
                // Full: fill centre of the heart.
                for xx in (dx + 1)..(dx + 5).min(dw) {
                    if dy + 1 < dh {
                        draw::dot(grid, xx, dy + 1);
                    }
                }
                for xx in (dx + 1)..(dx + 5).min(dw) {
                    if dy + 2 < dh {
                        draw::dot(grid, xx, dy + 2);
                    }
                }
            } else if state == 1 {
                // Half: fill left side only.
                for xx in (dx + 1)..(dx + 3).min(dw) {
                    if dy + 1 < dh {
                        draw::dot(grid, xx, dy + 1);
                    }
                    if dy + 2 < dh {
                        draw::dot(grid, xx, dy + 2);
                    }
                }
            }
            // state == 0: outline only (already drawn above)
        }

        Ok(())
    }
}

// ─── 3. Metroid Energy Tanks ──────────────────────────────────────────────────

/// Metroid HUD: a row of rectangular energy-tank segments that charge one at a
/// time.  Each tank is a bordered rectangle; the active one shows a partial fill.
struct MetroidETanks;
impl ProgressStyle for MetroidETanks {
    fn name(&self) -> &str {
        "metroid-etanks"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Metroid energy-tank segments — each tank charges then the next activates"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let tank_dot_w = ((dw / 6).max(3)).min(dw);
        let gap_dots = 1usize;
        let n_tanks = (dw / (tank_dot_w + gap_dots)).max(1);

        let filled_f = ctx.eased * n_tanks as f32;
        let full_tanks = filled_f as usize;
        let partial_f = filled_f.fract();

        for i in 0..n_tanks {
            let x0 = i * (tank_dot_w + gap_dots);
            if x0 >= dw {
                break;
            }
            let actual_w = tank_dot_w.min(dw.saturating_sub(x0));
            // Border.
            draw::rect_outline(grid, x0, 0, actual_w, dh);
            // Fill.
            if i < full_tanks {
                // Fully charged: fill interior.
                let iw = actual_w.saturating_sub(2);
                let ih = dh.saturating_sub(2);
                if iw > 0 && ih > 0 {
                    draw::fill_rect(grid, x0 + 1, 1, iw, ih);
                }
            } else if i == full_tanks && partial_f > 0.001 {
                // Partially charged: fill bottom portion of interior.
                let iw = actual_w.saturating_sub(2);
                let ih = dh.saturating_sub(2);
                if iw > 0 && ih > 0 {
                    let charge_h = ((partial_f * ih as f32).round() as usize).max(1).min(ih);
                    let y0 = ih + 1 - charge_h; // fills bottom-up
                    draw::fill_rect(grid, x0 + 1, y0, iw, charge_h);
                }
            }
        }

        Ok(())
    }
}

// ─── 4. Tetris Well ──────────────────────────────────────────────────────────

/// A vertical well (left and right walls, open top) fills from the bottom as
/// tetrominoes settle.  The stack height is driven by `eased`; the topmost
/// layer shows the current "falling" piece as a blinking row.
struct TetrisWell;
impl ProgressStyle for TetrisWell {
    fn name(&self) -> &str {
        "tetris-well"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Tetris well fills upward with a settling block stack"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Well walls.
        draw::vline(grid, 0, 0, dh.saturating_sub(1));
        draw::vline(grid, dw.saturating_sub(1), 0, dh.saturating_sub(1));
        // Floor.
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        let inner_w = dw.saturating_sub(2);
        let inner_h = dh.saturating_sub(1);
        if inner_w == 0 || inner_h == 0 {
            return Ok(());
        }

        // Stack: fills from the bottom.
        let stack_dots = (ctx.eased * inner_h as f32) as usize;
        let stack_dots = stack_dots.min(inner_h);
        let stack_y0 = inner_h.saturating_sub(stack_dots);

        // Draw stacked rows with slight texture (every 4 dots a gap line).
        for y in stack_y0..inner_h {
            let row_mod = (inner_h - 1 - y) % 4;
            if row_mod == 3 {
                // "Mortar" gap — sparse dots.
                for x in (1..=inner_w).step_by(3) {
                    draw::dot(grid, x, y);
                }
            } else {
                draw::hline(grid, 1, inner_w, y);
            }
        }

        // Falling piece: a blinking 4-wide row just above the stack.
        if stack_dots < inner_h {
            let piece_y = stack_y0.saturating_sub(1);
            let blink = ((ctx.time * 4.0) as usize) % 2 == 0;
            if blink && piece_y < inner_h {
                // Randomise piece shape by time bucket (cycles through shapes).
                let shape = ((ctx.time * 0.5) as usize) % 5;
                let pw = inner_w.clamp(1, 8);
                let px0 = 1 + (inner_w.saturating_sub(pw)) / 2;
                match shape {
                    0 => draw::hline(grid, px0, (px0 + 3).min(dw.saturating_sub(1)), piece_y), // I
                    1 => {
                        // L
                        draw::hline(grid, px0, (px0 + 2).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::dot(grid, px0 + 2, piece_y.saturating_sub(1));
                        }
                    }
                    2 => {
                        // S
                        draw::hline(grid, px0 + 1, (px0 + 3).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::hline(
                                grid,
                                px0,
                                (px0 + 2).min(dw.saturating_sub(1)),
                                piece_y.saturating_sub(1),
                            );
                        }
                    }
                    3 => {
                        // T
                        draw::hline(grid, px0, (px0 + 2).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::dot(grid, px0 + 1, piece_y.saturating_sub(1));
                        }
                    }
                    _ => {
                        // O (square)
                        draw::hline(grid, px0, (px0 + 1).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::hline(
                                grid,
                                px0,
                                (px0 + 1).min(dw.saturating_sub(1)),
                                piece_y.saturating_sub(1),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// ─── 5. Duck Hunt ─────────────────────────────────────────────────────────────

/// A mallard flaps across the sky past a tree and the tall grass while the
/// HIT counter along the bottom fills duck by duck with progress.
struct DuckHunt;
impl ProgressStyle for DuckHunt {
    fn name(&self) -> &str {
        "duck-hunt"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Duck Hunt: a flapping mallard crosses the sky above a tree and tall grass; the HIT counter's ten duck marks turn red as progress = ducks shot"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);
        let grass = hi - 6;

        // Tree on the left: leafy crown on a trunk.
        sprite(
            grid,
            1,
            0,
            &[
                ".####.", "######", "######", ".####.", "..##..", "..##..", "..##..",
            ],
        );

        // Tall grass: a row of tufts with a solid base.
        for x in 0..wi {
            let tuft = [1, 0, 2, 0, 1, 2, 0][(x % 7) as usize];
            for t in 0..=tuft {
                draw::dot_i(grid, x, grass - t + 1);
            }
        }
        draw::hline(grid, 0, w - 1, (grass + 1).max(0) as usize);

        // The duck: wings flap between two frames as it zigzags right.
        const UP: [&str; 4] = ["...##....", "....##.##", ".######.#", "..#####.."];
        const DOWN: [&str; 4] = [".......##", ".######.#", "..####...", "...##...."];
        let dx = (ctx.time * 22.0).rem_euclid((wi + 20) as f32) as i32 - 10;
        let dy = 1 + (tri(ctx.time * 0.6) * (grass - 6).max(0) as f32) as i32;
        let wings = if (ctx.time * 5.0).fract() < 0.5 {
            &UP
        } else {
            &DOWN
        };
        sprite(grid, dx, dy, wings);

        // HIT counter: ten duck marks along the bottom row of cells.
        let hits = (ctx.progress * 10.0).round() as i32;
        let slots = 10.min(cw as i32 / 2);
        let x0 = (cw as i32 / 2 - slots) * 2;
        for i in 0..slots {
            let x = x0 + i * 4;
            if i < hits {
                block(grid, x, hi - 3, 3, 3);
            } else {
                block(grid, x + 1, hi - 2, 1, 1);
            }
        }

        // Colour: green tree and grass, brown-green duck, red marks for hits.
        grid.enable_color_support();
        let last = ch - 1;
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let color = if cy == last && ch > 1 {
                    let i = (px - x0).div_euclid(4);
                    if (0..hits).contains(&i) {
                        Color::rgb(232, 60, 48)
                    } else {
                        Color::rgb(220, 220, 220)
                    }
                } else if px + 1 >= dx && px - 1 < dx + 9 && py + 2 >= dy && py - 2 < dy + 4 {
                    if px >= dx + 6 {
                        Color::rgb(40, 170, 80) // green head
                    } else {
                        Color::rgb(160, 110, 60)
                    }
                } else if py >= grass - 2 || px < 8 {
                    Color::rgb(70, 180, 60)
                } else {
                    Color::rgb(110, 170, 255)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}
// ─── 6. Excitebike ───────────────────────────────────────────────────────────

/// A motocross rider (helmet, hunched body, two spoked wheels) races along the
/// dirt track, launching off a ramp mid-course; distance covered is `eased`.
/// The TEMP gauge top-right throbs with turbo heat and flashes near overheat.
struct Excitebike;
impl ProgressStyle for Excitebike {
    fn name(&self) -> &str {
        "excitebike"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Excitebike: a motocross rider races the dirt track and jumps a ramp as progress = distance, with a turbo TEMP gauge throbbing top-right"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;

        // Track: ground line, dashed lane marks, and a ramp mid-course.
        draw::hline(grid, 0, w - 1, ground.max(0) as usize);
        for x in (0..wi).filter(|x| x % 6 < 3) {
            draw::dot_i(grid, x - (ctx.time * 30.0) as i32 % 6, ground - 5);
        }
        let ramp_x = wi / 2;
        let ramp = |x: i32| -> i32 {
            let d = x - (ramp_x - 12);
            if (0..12).contains(&d) {
                d / 2
            } else {
                0
            }
        };
        for x in ramp_x - 12..ramp_x {
            draw::vline(
                grid,
                x.max(0) as usize,
                (ground - ramp(x)).max(0) as usize,
                ground.max(0) as usize,
            );
        }

        // Rider and bike; airborne in an arc for a stretch after the ramp lip.
        const BIKE: [[&str; 8]; 2] = [
            [
                ".....##.....",
                "....###.....",
                "...####.##..",
                "..#######...",
                ".##.###.##..",
                "#..#...#..#.",
                "#.##...#.##.",
                ".##.....##..",
            ],
            [
                ".....##.....",
                "....###.....",
                "...####.##..",
                "..#######...",
                ".##.###.##..",
                "#..#...#..#.",
                "##.#...##.#.",
                ".##.....##..",
            ],
        ];
        let bx = (ctx.eased * (wi - 12).max(0) as f32) as i32;
        let wheel_x = bx + 9;
        let air = {
            let d = (wheel_x - ramp_x) as f32 / 14.0;
            if (0.0..1.0).contains(&d) {
                (6.0 * (1.0 - (2.0 * d - 1.0).powi(2)) + 6.0 * (1.0 - d)) as i32
            } else {
                ramp(wheel_x)
            }
        };
        let by = ground - 8 - air;
        let spin = ((ctx.time * 10.0) as usize) % 2;
        sprite(grid, bx, by, &BIKE[spin]);

        // Turbo TEMP gauge: throbs with time; flashes when nearly overheated.
        let heat = 0.55 + 0.4 * (ctx.time * 1.7).sin().abs();
        let gw = 20.min(wi / 3).max(4);
        let gx = (wi - gw - 1).max(0);
        draw::rect_outline(grid, gx as usize, 0, gw as usize, 4);
        let fill = (heat * (gw - 2) as f32) as usize;
        if heat < 0.9 || (ctx.time * 8.0).fract() < 0.5 {
            draw::fill_rect(grid, gx as usize + 1, 1, fill, 2);
        }

        // Colour: red bike, orange-red gauge, brown dirt.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2, cy as i32 * 4);
                let color = if px + 1 >= bx && px <= bx + 11 && py + 3 >= by && py <= by + 7 {
                    Color::rgb(232, 60, 52)
                } else if py < 4 && px + 1 >= gx {
                    Color::rgb(255, 140, 40)
                } else {
                    Color::rgb(190, 130, 70)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 7. Punch-Out Stars ──────────────────────────────────────────────────────

/// The classic over-the-shoulder Punch-Out view: ring ropes and corner posts,
/// a gloves-up opponent swaying centre-ring, and Little Mac jabbing from the
/// foreground. Star punches earned (top-left) fill with progress while the
/// opponent's stamina bar (top-right) drains.
struct PunchOutStars;

const STAR_FULL: [&str; 5] = ["..#..", ".###.", "#####", ".###.", ".#.#."];
const STAR_EMPTY: [&str; 5] = ["..#..", ".#.#.", "#...#", ".#.#.", ".#.#."];

impl ProgressStyle for PunchOutStars {
    fn name(&self) -> &str {
        "punchout-stars"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Punch-Out!!: Little Mac jabs at a gloves-up opponent inside the ring ropes; star punches earned fill with progress as the opponent's stamina drains"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);
        let mid = wi / 2;

        // Ropes (kept clear of the fighters) and corner posts.
        for y in [6, 9, 12] {
            for x in 2..wi - 2 {
                if (x - mid).abs() > 10 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        draw::vline(grid, 1, 5, h - 1);
        draw::vline(grid, w.saturating_sub(2), 5, h - 1);

        // Opponent: head, gloves up, trunks; sways with time.
        const FOE: [&str; 12] = [
            "....#####....",
            "...#######...",
            "...#.###.#...",
            "...#######...",
            "##..#####..##",
            "###.......###",
            "###.#####.###",
            ".#.#######.#.",
            "...#######...",
            "...###.###...",
            "...##...##...",
            "...##...##...",
        ];
        let sway = ((ctx.time * 2.5).sin() * 2.0) as i32;
        let fx = mid - 6 + sway;
        sprite(grid, fx, 0, &FOE);

        // Little Mac, seen from behind, throwing alternate jabs.
        let jab = (ctx.time * 3.0).fract() < 0.3;
        let left = (ctx.time * 1.5).fract() < 0.5;
        let mac: [&str; 7] = if !jab {
            [
                "...###...",
                "...###...",
                "#.#####.#",
                "##.###.##",
                "...###...",
                "..#...#..",
                "..#...#..",
            ]
        } else if left {
            [
                "##.###...",
                ".#.###...",
                "..#####.#",
                "...###.##",
                "...###...",
                "..#...#..",
                "..#...#..",
            ]
        } else {
            [
                "...###.##",
                "...###.#.",
                "#.#####..",
                "##.###...",
                "...###...",
                "..#...#..",
                "..#...#..",
            ]
        };
        let mx = mid - 4;
        sprite(grid, mx, hi - 7, &mac);

        // Star punches earned, top-left.
        let stars = (ctx.eased * 3.0).round() as i32;
        for k in 0..3 {
            let art = if k < stars { &STAR_FULL } else { &STAR_EMPTY };
            sprite(grid, 4 + k * 7, 0, art);
        }

        // Opponent stamina bar, top-right, draining with progress.
        let bw = 22.min(wi / 4).max(4);
        let bx = (wi - bw - 3).max(0);
        draw::rect_outline(grid, bx as usize, 0, bw as usize, 4);
        let left_hp = ((1.0 - ctx.eased) * (bw - 2) as f32).round() as usize;
        if left_hp > 0 {
            draw::fill_rect(grid, bx as usize + 1, 1, left_hp, 2);
        }

        // Colour: yellow stars, pink stamina, skin-tone foe, green Mac, red ropes.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let color = if py < 4 && px < 26 {
                    Color::rgb(250, 214, 60)
                } else if py < 4 && px >= bx {
                    Color::rgb(240, 110, 150)
                } else if (mx..mx + 9).contains(&px) && py >= hi - 7 {
                    Color::rgb(90, 210, 100)
                } else if (fx..fx + 13).contains(&px) {
                    Color::rgb(236, 176, 120)
                } else {
                    Color::rgb(220, 64, 64)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 8. Contra Spread ────────────────────────────────────────────────────────

/// Contra spread-shot: multiple bullets fan out from the left edge in a
/// symmetric arc.  Progress controls how far the fan has travelled rightward.
/// Each bullet trail is a dotted line along its angle.
struct ContraSpread;
impl ProgressStyle for ContraSpread {
    fn name(&self) -> &str {
        "contra-spread"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Contra spread-shot bullets fan out rightward; progress = bullet travel"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx_f = 0.0f32; // fire from left edge
        let cy_f = (dh as f32 - 1.0) / 2.0; // vertical centre
        let n_rays = 5usize;
        let reach = (ctx.eased * dw as f32) as usize;

        // Also draw a muzzle flash at the origin.
        let flash = ((ctx.time * 12.0) as usize) % 2 == 0;
        if flash {
            draw::dot(grid, 0, cy_f as usize);
        }

        for i in 0..n_rays {
            // Angles: 0° (straight), ±22.5°, ±45°.
            let angle_idx = i as i32 - (n_rays as i32 / 2);
            let angle = angle_idx as f32 * PI / 8.0; // 22.5° steps
            let dx_f = angle.cos();
            let dy_f = angle.sin();

            // Draw the bullet trail up to reach dots.
            let steps = reach.min(dw);
            for s in (0..steps).step_by(2) {
                let bx = (cx_f + s as f32 * dx_f) as i32;
                let by = (cy_f + s as f32 * dy_f) as i32;
                draw::dot_i(grid, bx, by);
            }
            // Bullet head at the tip.
            if reach > 0 {
                let bx = (cx_f + reach as f32 * dx_f) as i32;
                let by = (cy_f + reach as f32 * dy_f) as i32;
                draw::dot_i(grid, bx, by);
                draw::dot_i(grid, bx + 1, by);
            }
        }

        Ok(())
    }
}

// ─── 9. Mega Man Weapon ───────────────────────────────────────────────────────

/// Mega Man, arm cannon raised, fires lemon pellets to the right beside his
/// classic tick-segmented weapon-energy meter, which charges bottom-up with
/// `eased`.
struct MegaManWeapon;
impl ProgressStyle for MegaManWeapon {
    fn name(&self) -> &str {
        "megaman-weapon"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Mega Man fires lemon pellets from his arm cannon beside the tick-segmented weapon-energy meter, which charges bottom-up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);

        // Energy meter: a framed column of horizontal ticks, filled bottom-up.
        draw::rect_outline(grid, 0, 0, 7, h);
        let ticks = ((hi - 2) / 2).max(1);
        let lit = (ctx.eased * ticks as f32).round() as i32;
        for k in 0..lit {
            block(grid, 2, hi - 3 - k * 2, 3, 1);
        }

        // Mega Man: round helmet, face, chunky body, buster arm forward.
        const MEGAMAN: [&str; 12] = [
            "....####...",
            "...######..",
            "..##..####.",
            "..#.#.###..",
            "..######...",
            "...####....",
            ".#######...",
            "###########",
            "#.######.##",
            "..######...",
            ".###..###..",
            "####..####.",
        ];
        let my = hi - 12;
        sprite(grid, 10, my, &MEGAMAN);

        // Lemon pellets streaming out of the buster.
        let lane = (wi - 22).max(1) as f32;
        for k in 0..3 {
            let x = 22 + (ctx.time * 46.0 + k as f32 * lane / 3.0).rem_euclid(lane) as i32;
            sprite(grid, x, my + 6, &[".##.", "####", ".##."]);
        }

        // Colour: pale-yellow meter and pellets, Mega Man blue.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let px = cx as i32 * 2 + 1;
                let color = if (9..21).contains(&px) {
                    Color::rgb(64, 140, 255)
                } else {
                    Color::rgb(250, 226, 130)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 10. Ice Climber Up ───────────────────────────────────────────────────────

/// Popo in his hooded parka swings his mallet up, smashes the ice-brick floor
/// overhead (chunks fly), jumps through the gap and the mountain scrolls down
/// a floor. A mountain gauge on the right shows his altitude = `eased`, with
/// the summit flag waiting at the peak.
struct IceClimberUp;

/// Popo, facing right: hood with face opening, parka, arms, boots.
const POPO: [&str; 7] = [
    ".####.", "##..##", "##..##", ".####.", "######", ".####.", ".#..#.",
];

impl ProgressStyle for IceClimberUp {
    fn name(&self) -> &str {
        "iceclimber-up"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Popo in his parka smashes the ice-brick floor overhead with his mallet and jumps up through it; a mountain gauge shows altitude"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (dw as i32, dh as i32);
        let t = ctx.time;

        // Scene on the left, mountain gauge on the right.
        let gauge_w = 26.min(wi / 3).max(0);
        let sw = (wi - gauge_w - 2).max(8);
        let gap = 9; // floor spacing in dots
        let u = (t * 0.8).fract();
        let scroll = if u > 0.85 {
            ((u - 0.85) / 0.15 * gap as f32) as i32
        } else {
            0
        };
        let px = sw / 2 - 6;
        let hole = (px + 6, px + 12);
        let base = hi - 2 + scroll; // top row of the floor Popo stands on
        let broken = u >= 0.4;
        for k in -1..3 {
            let fy = base - k * gap;
            for x in 0..sw {
                let in_hole = x >= hole.0 && x < hole.1;
                // Floor 0 already has its hole; floor 1 breaks at u = 0.4.
                if in_hole && (k == 0 || (k == 1 && broken)) {
                    continue;
                }
                draw::dot_i(grid, x, fy);
                if (x + k * 2).rem_euclid(4) != 0 {
                    draw::dot_i(grid, x, fy + 1);
                }
            }
        }
        // Flying ice chunks after the smash.
        if (0.4..0.75).contains(&u) {
            let s = (u - 0.4) / 0.35;
            let fy = base - gap;
            for (i, dir) in [-1.0f32, -0.4, 0.4, 1.0].iter().enumerate() {
                let x = hole.0 + 3 + (dir * s * 10.0) as i32;
                let y = fy - (s * (6.0 - i as f32)) as i32 + (s * s * 10.0) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
        }

        // Popo: stand, wind up, smash, jump up through the gap and land left.
        let (mut ox, mut oy) = (0, 0);
        if (0.5..=0.85).contains(&u) {
            let s = (u - 0.5) / 0.35;
            ox = ((s * PI).sin() * 7.0) as i32;
            oy = -(s * gap as f32 + (s * PI).sin() * 4.0) as i32;
        } else if u > 0.85 {
            oy = -gap;
        }
        let (x, y) = (px + ox, base - 7 + oy + if u > 0.85 { scroll } else { 0 });
        // u > 0.85 already moved with the floor via `base`, so undo double shift.
        let y = if u > 0.85 { y - scroll } else { y };
        sprite(grid, x, y, &POPO);
        // Mallet: resting on the shoulder, raised overhead for the smash.
        if (0.2..0.5).contains(&u) {
            draw::dot_i(grid, x + 5, y + 2);
            draw::dot_i(grid, x + 6, y + 1);
            draw::dot_i(grid, x + 7, y);
            block(grid, x + 7, y - 3, 3, 3);
        } else {
            draw::dot_i(grid, x + 6, y + 3);
            draw::dot_i(grid, x + 7, y + 2);
            block(grid, x + 7, y, 3, 2);
        }

        // Mountain gauge: snowy peak with a climber marker and summit flag.
        let g0 = wi - gauge_w;
        let half = gauge_w / 2;
        if gauge_w > 4 {
            for i in 0..=half {
                let yy = hi - 1 - (i * (hi - 4)) / half.max(1);
                draw::dot_i(grid, g0 + i, yy);
                draw::dot_i(grid, wi - 1 - i, yy);
            }
            let peak = (g0 + half, 3);
            sprite(grid, peak.0, 0, &["###", "#..", "#.."]);
            let m = ctx.eased.clamp(0.0, 1.0);
            let mx = g0 + (m * half as f32) as i32;
            let my = hi - 1 - (m * (hi - 4) as f32) as i32;
            block(grid, mx - 1, my - 3, 2, 3);
            let _ = peak;
        }

        // Colour: blue parka, wooden mallet, icy bricks, white peak, red flag.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (qx, qy) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let color = if qx >= g0 - 1 {
                    if qy < 4 && qx > g0 + half {
                        Color::rgb(236, 60, 60) // summit flag
                    } else if (g0..g0 + half).contains(&qx)
                        && (ctx.eased * half as f32) as i32 + g0 + 2 >= qx
                        && qy < hi - 2
                    {
                        Color::rgb(90, 140, 255) // climber marker
                    } else {
                        Color::rgb(230, 240, 255)
                    }
                } else if (x - 1..x + 6).contains(&qx) && (y - 1..y + 8).contains(&qy) {
                    Color::rgb(80, 120, 255) // Popo's parka
                } else if (x + 6..x + 11).contains(&qx) && (y - 4..y + 4).contains(&qy) {
                    Color::rgb(214, 150, 80) // mallet
                } else {
                    Color::rgb(130, 210, 255) // ice bricks
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 11. Donkey Kong Barrel ───────────────────────────────────────────────────

/// Donkey Kong squats on the top girder hurling barrels that roll along the
/// riveted girders while Mario climbs the ladder between them; his height on
/// the ladder is `eased`.
struct DonkeyBarrel;
impl ProgressStyle for DonkeyBarrel {
    fn name(&self) -> &str {
        "donkey-barrel"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Donkey Kong: DK squats on the sloped top girder throwing rolling barrels while Mario climbs the cyan ladder; his height on it = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);

        // Riveted girders: a sloped top deck and the flat floor.
        let deck_end = (wi - 10).max(1);
        let top_y = |x: i32| 6 + x * 2 / wi.max(1);
        for x in 0..deck_end {
            draw::dot_i(grid, x, top_y(x));
            if x % 3 == 0 {
                draw::dot_i(grid, x, top_y(x) + 1);
            }
        }
        for x in 0..wi {
            draw::dot_i(grid, x, hi - 2);
            if x % 3 == 0 {
                draw::dot_i(grid, x, hi - 1);
            }
        }

        // Donkey Kong, top-left, arms out.
        const DK: [&str; 6] = [
            "....####....",
            "...#.##.#...",
            "..########..",
            "############",
            "##.######.##",
            "##..#..#..##",
        ];
        let dk_y = top_y(0) - 6 + i32::from((ctx.time * 2.0).fract() >= 0.5);
        sprite(grid, 0, dk_y, &DK);

        // Ladder joining the girders.
        let lx = wi * 11 / 20;
        let (l_top, l_bot) = (top_y(lx) + 2, hi - 3);
        draw::vline(
            grid,
            lx.max(0) as usize,
            l_top.max(0) as usize,
            l_bot.max(0) as usize,
        );
        draw::vline(
            grid,
            (lx + 5).max(0) as usize,
            l_top.max(0) as usize,
            l_bot.max(0) as usize,
        );
        for y in (l_top..=l_bot).filter(|y| y % 2 == 0) {
            draw::hline(
                grid,
                lx.max(0) as usize,
                (lx + 5).max(0) as usize,
                y.max(0) as usize,
            );
        }

        // Barrels: rolling right along the deck, then left along the floor.
        const BARREL: [[&str; 4]; 2] = [
            [".##.", "#..#", "#..#", ".##."],
            [".##.", "####", "####", ".##."],
        ];
        for k in 0..2 {
            let x = 14
                + (ctx.time * 16.0 + k as f32 * 31.0).rem_euclid((deck_end - 14).max(1) as f32)
                    as i32;
            sprite(grid, x, top_y(x) - 4, &BARREL[(x / 2 % 2) as usize]);
            let fx = wi - 4 - (ctx.time * 16.0 + k as f32 * 44.0).rem_euclid(wi as f32) as i32;
            sprite(
                grid,
                fx,
                hi - 6,
                &BARREL[(fx / 2 % 2).unsigned_abs() as usize],
            );
        }

        // Mario on the ladder: feet climb from the floor to the top deck.
        let feet = l_bot - (ctx.eased * (l_bot - l_top + 2) as f32) as i32;
        let legs = if (ctx.time * 6.0).fract() < 0.5 {
            "#...#"
        } else {
            ".#.#."
        };
        sprite(
            grid,
            lx,
            feet - 5,
            &[".###.", "#####", ".##..", "#####", ".###.", legs],
        );

        // Colour: red Mario and girders, brown DK and barrels, cyan ladder.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2, cy as i32 * 4);
                let mario = px + 1 >= lx && px <= lx + 4 && py + 3 >= feet - 5 && py <= feet;
                let color = if mario {
                    Color::rgb(232, 56, 48)
                } else if px < 12 && py + 3 < top_y(0) {
                    Color::rgb(170, 96, 40)
                } else if (lx..=lx + 5).contains(&px) || (lx..=lx + 5).contains(&(px + 1)) {
                    Color::rgb(80, 210, 230)
                } else if cy == ch - 1
                    || py + 3 >= top_y(px) && py <= top_y(px) + 1 && px < deck_end
                {
                    Color::rgb(226, 70, 96)
                } else {
                    Color::rgb(206, 140, 60)
                };
                let _ = grid.set_cell_color(cx, cy, color);
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
    let styles = progress::styles::nintendo::styles();
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
