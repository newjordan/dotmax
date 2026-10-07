//! `physics` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O physics.rs && ./physics [style-name]
//! ```

const DEFAULT_STYLE: &str = "projectile";

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
    pub mod physics {
//! Physics / mechanics progress bars for dotmax.
//!
//! Every bar in this theme is driven by real physical equations — no faked
//! motion. `ctx.eased` is the canonical "how far through" parameter; `ctx.time`
//! advances animation so bars live independently of progress.
//!
//! ## Styles (11 total)
//!
//! | Name | Physics | Key equation |
//! |------|---------|-------------|
//! | `projectile` | Projectile motion | y = v·sinθ·t − ½g·t²,  x = v·cosθ·t |
//! | `pendulum` | Simple pendulum SHM | θ(t) = θ₀·cos(ωt), pivot rides the beam |
//! | `mass-spring` | Mass-spring SHM | x = A·cos(ωt),  spring coil drawn |
//! | `newtons-cradle` | Newton's cradle | touching balls, end-ball momentum swap |
//! | `orbital` | Kepler orbit | E − e·sin E = M, star at a focus |
//! | `terminal-velocity` | Drag / terminal vel. | v = v_t·(1 − e^(−t/τ)) |
//! | `elastic-collision` | 1D elastic collision | two carts exchange momentum |
//! | `gravity-well` | 1/r potential well | grid-sheet funnel, spiral in-fall |
//! | `light-cone` | Special relativity | past/future cones, timelike worldline |
//! | `double-slit` | Wave interference | I(y) ∝ cos²(πy/Δ), fringes accumulate |
//! | `doppler` | Doppler effect | compressed / expanded wavefronts |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `physics` theme.
///
/// Returns 11 bars each implementing a distinct branch of classical or modern
/// physics, safe to render from 1×1 cells to 80×8 or larger.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Projectile),
        Box::new(Pendulum),
        Box::new(MassSpring),
        Box::new(NewtonsCradle),
        Box::new(Orbital),
        Box::new(TerminalVelocity),
        Box::new(ElasticCollision),
        Box::new(GravityWell),
        Box::new(LightCone),
        Box::new(DoubleSlit),
        Box::new(Doppler),
    ]
}

// ---------------------------------------------------------------------------
// Helper: draw a line between two dot-space points (Bresenham)
// ---------------------------------------------------------------------------
#[inline]
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let px = x0 + (x1 - x0) * i / steps;
        let py = y0 + (y1 - y0) * i / steps;
        draw::dot_i(grid, px, py);
    }
}

/// Filled round blob of radius `r` dots centred on `(cx, cy)`.
fn disc(grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r + r {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }
    }
}

/// Tint every cell touched by the square of radius `r` dots around `(x, y)`.
fn tint_around(grid: &mut BrailleGrid, x: i32, y: i32, r: i32, col: crate::Color) {
    let lo = (x - r).max(0) as usize / 2;
    let hi = (x + r).max(0) as usize / 2;
    for yy in (y - r).max(0)..=(y + r).max(0) {
        draw::tint_row(grid, yy as usize / 4, lo, hi, col);
    }
}

// ---------------------------------------------------------------------------
// 1. Projectile motion
//    x = v·cosθ·τ,  y = v·sinθ·τ − ½g·τ²
//    eased sweeps τ from 0 → T_land (time of flight = 2v·sinθ/g).
//    The full parabolic arc draws out dot by dot; a filled baseline tracks range.
// ---------------------------------------------------------------------------
struct Projectile;
impl ProgressStyle for Projectile {
    fn name(&self) -> &str {
        "projectile"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Projectile motion: a cannonball fired from a cannon traces its parabola toward a target flag as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        // Physics params (arbitrary units scaled to dot space)
        let theta: f32 = PI / 4.0; // 45° launch angle
        let g: f32 = 1.0; // gravitational acc (normalised)
        let v: f32 = 1.0; // launch speed (normalised)
        let t_flight = 2.0 * v * theta.sin() / g; // total flight time
        let x_range = v * theta.cos() * t_flight; // total horizontal range

        // Number of arc-steps visible = eased fraction of flight
        let arc_steps = ((w * 4).max(32)) as usize;
        let t_now = ctx.eased * t_flight; // current simulation time

        // Ground baseline: dotted ahead of the range already covered.
        let ground = h.saturating_sub(1);
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, ground);
        }

        // Draw the arc up to t_now
        let mut prev: Option<(i32, i32)> = None;
        for step in 0..=arc_steps {
            let frac = step as f32 / arc_steps as f32;
            let t = frac * t_now;
            // x and y in physics units → dot space
            let px = (v * theta.cos() * t / x_range.max(1e-6) * wf) as i32;
            // y is up in physics, down in screen, origin at ground
            let py_phys = v * theta.sin() * t - 0.5 * g * t * t;
            let max_y = 0.5 * v * v * theta.sin() * theta.sin() / g;
            let py = (ground as f32 - py_phys / max_y.max(1e-6) * (hf - 2.0)) as i32;
            let py = py.clamp(0, h as i32 - 1);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, px, py);
            }
            draw::dot_i(grid, px, py);
            prev = Some((px, py));
        }

        // Filled range bar at ground: how far the projectile has travelled
        let fill_x = (ctx.eased * wf) as usize;
        draw::hline(grid, 0, fill_x.min(w.saturating_sub(1)), ground);

        // Cannonball at the head of the arc (spins a notch with time).
        if let Some((bx, by)) = prev {
            disc(grid, bx, by.min(ground as i32 - 2), 1);
            if (ctx.time * 8.0) as i32 % 2 == 0 {
                draw::dot_i(grid, bx + 2, by.min(ground as i32 - 2));
            }
        }

        // Cannon at the launch point: wheel plus a 45° barrel.
        let gi = ground as i32;
        if h >= 8 {
            disc(grid, 2, gi - 2, 1);
            for k in 0..5 {
                draw::dot_i(grid, 2 + k, gi - 3 - k);
                draw::dot_i(grid, 3 + k, gi - 3 - k);
            }
        }

        // Target flag at the landing point.
        let fx = w.saturating_sub(2);
        draw::vline(grid, fx, ground.saturating_sub(7), ground);
        for k in 0..3usize {
            draw::hline(
                grid,
                fx.saturating_sub(4 - k),
                fx,
                ground.saturating_sub(7) + k,
            );
        }

        // Tint the filled arc region
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Pendulum (simple SHM)
//    θ(t) = θ₀·cos(ω·t)   where ω = √(g/L)
//    eased sets θ₀ (max swing amplitude, 0 → π/3).
//    time drives ω·t: the bob swings; its trail is drawn.
//    Pivot is at top-centre; rod and bob rendered each frame.
// ---------------------------------------------------------------------------
struct Pendulum;
impl ProgressStyle for Pendulum {
    fn name(&self) -> &str {
        "pendulum"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Pendulum: pivot rides a ceiling beam to progress; rod and solid bob swing θ₀·cos(ωt) beneath"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;

        // Rod length leaves room for the bob radius below it.
        let bob_r = if hi >= 12 { 2 } else { 1 };
        let rod_len = (hi - bob_r - 2).max(1) as f32;

        // Fixed visual amplitude θ₀ = 35°; ω scaled up so it reads as motion.
        let theta0 = 0.61_f32;
        let omega = 2.6_f32;
        let theta = theta0 * (omega * ctx.time).cos();

        // Pivot carriage travels along the beam with progress, kept far
        // enough from the walls that the full swing stays on screen.
        let margin = (rod_len * theta0.sin()) as i32 + bob_r + 1;
        let span = (wi - 2 * margin).max(0) as f32;
        let pivot_x = margin.min(wi - 1) + (ctx.eased * span) as i32;
        let pivot_y = 0_i32;

        // Ceiling beam: solid behind the carriage, dotted ahead.
        for x in 0..wi {
            if x <= pivot_x || x % 3 == 0 {
                draw::dot_i(grid, x, pivot_y);
            }
        }
        // Carriage block on the beam.
        for dx in -2_i32..=2 {
            draw::dot_i(grid, pivot_x + dx, pivot_y + 1);
        }

        // Faint swing arc traced by the bob.
        let steps = ((2.0 * rod_len * theta0) as i32).max(2);
        for s in 0..=steps {
            if s % 2 == 1 {
                continue;
            }
            let th = theta0 * ((s as f32 / steps as f32) * 2.0 - 1.0);
            let ax = pivot_x + (rod_len * th.sin()).round() as i32;
            let ay = pivot_y + (rod_len * th.cos()).round() as i32;
            draw::dot_i(grid, ax, ay);
        }

        // Rod and solid round bob.
        let bob_x = pivot_x + (rod_len * theta.sin()).round() as i32;
        let bob_y = pivot_y + (rod_len * theta.cos()).round() as i32;
        line(grid, pivot_x, pivot_y + 1, bob_x, bob_y);
        disc(grid, bob_x, bob_y, bob_r);

        // Tint: beam gradient up to the carriage, bob/rod in the leading colour.
        let (cw, ch) = grid.dimensions();
        let pivot_cell = (pivot_x.max(0) as usize) / 2;
        for cx in 0..=pivot_cell.min(cw.saturating_sub(1)) {
            let t = cx as f32 / cw.max(1) as f32;
            draw::tint_row(grid, 0, cx, cx, ctx.palette.sample(t));
        }
        let lead = ctx.palette.sample(ctx.eased);
        let (bx0, bx1) = (
            (bob_x - bob_r).min(pivot_x).max(0) as usize / 2,
            (bob_x + bob_r).max(pivot_x).max(0) as usize / 2,
        );
        for cy in 1..ch {
            draw::tint_row(grid, cy, bx0, bx1, lead);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Mass-spring SHM
//    x(t) = A·cos(ωt + φ)   where ω = √(k/m)
//    eased sets A (amplitude, 0 → half-bar).  time drives ωt.
//    The spring coil is drawn as a zigzag between the wall and the mass.
// ---------------------------------------------------------------------------
struct MassSpring;
impl ProgressStyle for MassSpring {
    fn name(&self) -> &str {
        "mass-spring"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Mass-spring SHM: x=A·cos(ωt) — coil compresses/extends, amplitude grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        let omega = 3.0_f32; // angular frequency (visual)
        let amp_max = wf * 0.35;
        let amp = ctx.eased * amp_max;
        let equilibrium_x = wf * 0.55; // equilibrium position (dot space)
        let mass_x = equilibrium_x + amp * (omega * ctx.time).cos();
        let clamp_lo = 2.0_f32;
        let clamp_hi = (wf - 3.0).max(clamp_lo);
        let mass_x = mass_x.clamp(clamp_lo, clamp_hi);
        let mid_y = (hf / 2.0) as i32;

        // Wall on the left (2 dots wide)
        draw::vline(grid, 0, 0, h.saturating_sub(1));
        draw::vline(grid, 1, 0, h.saturating_sub(1));

        // Spring coil: zigzag from x=2 to mass_x-3
        let spring_start = 2_i32;
        let spring_end = (mass_x as i32 - 3).max(spring_start + 1);
        let spring_len = (spring_end - spring_start).max(1);
        let coil_count = 6_i32;
        let half_amp = (hf * 0.25).max(1.0) as i32;

        for c in 0..coil_count {
            let x0 = spring_start + spring_len * c / coil_count;
            let x1 = spring_start + spring_len * (c + 1) / coil_count;
            let y0 = mid_y + if c % 2 == 0 { half_amp } else { -half_amp };
            let y1 = mid_y + if c % 2 == 0 { -half_amp } else { half_amp };
            line(grid, x0, y0, x1, y1);
        }

        // Mass block (3×3 dot rectangle)
        let mx = mass_x as i32;
        for dy in -2_i32..=2 {
            for dx in 0_i32..=3 {
                draw::dot_i(grid, mx + dx, mid_y + dy);
            }
        }

        // Equilibrium marker (dashed vertical line at equilibrium_x)
        let eq_x = equilibrium_x as usize;
        for y in (0..h).step_by(2) {
            draw::dot(grid, eq_x.min(w.saturating_sub(1)), y);
        }

        // Tint: left of mass = compressed, right = extended
        let (cw, ch) = grid.dimensions();
        let mass_cell = (mass_x / 2.0) as usize;
        let eq_cell = (equilibrium_x / 2.0) as usize;
        let (lo, hi) = if mass_cell < eq_cell {
            (mass_cell, eq_cell)
        } else {
            (eq_cell, mass_cell)
        };
        for cx in lo..hi.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        // Also tint filled portion from left by eased
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t * 0.5); // blend, don't double-tint same colour
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Newton's cradle
//    5 balls on strings; only the end balls swing (elastic collision).
//    Left ball swings in with angle θ_L(t), right swings out θ_R(t).
//    Period T = 2π√(L/g); eased controls swing amplitude θ₀.
//    Momentum is conserved: when left ball hits, right ball leaves at same speed.
// ---------------------------------------------------------------------------
struct NewtonsCradle;
impl ProgressStyle for NewtonsCradle {
    fn name(&self) -> &str {
        "newtons-cradle"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Newton's cradle: five touching balls in a frame; the end balls trade swings, base fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        let wi = w as i32;
        let hi = h as i32;
        let n_balls: i32 = 5;

        // Five touching steel balls hung from a frame: ball size from height.
        let ball_r = ((hf * 0.16) as i32).max(1);
        let pitch = 2 * ball_r + 2;
        let rod_top = 0_i32;
        let rod_len = (hf - ball_r as f32 - 3.0).max(2.0);
        let row_w = pitch * (n_balls - 1);
        let x_first = (wf / 2.0) as i32 - row_w / 2;

        // Swing amplitude: always visible, grows with progress.
        let omega = 3.5_f32;
        let theta0 = 0.35 + ctx.eased * 0.35;
        // Only the end balls move: the left one swings in, strikes, and the
        // momentum pops the right one out, then back again.
        let phase = (omega * ctx.time).cos();
        let left_theta = if phase >= 0.0 { -theta0 * phase } else { 0.0 };
        let right_theta = if phase < 0.0 { -theta0 * phase } else { 0.0 };

        // Frame: top bar, two posts and a base.
        let reach = (rod_len * theta0.sin()) as i32 + ball_r + 4;
        let post_l = (x_first - reach).max(0);
        let post_r = (x_first + row_w + reach).min(wi - 1);
        let base_y = hi - 1;
        for x in post_l..=post_r {
            draw::dot_i(grid, x, rod_top);
        }
        line(grid, post_l, rod_top, post_l, base_y);
        line(grid, post_r, rod_top, post_r, base_y);

        // Base doubles as the progress track: solid to eased, dotted beyond.
        let filled = (ctx.eased * wf) as i32;
        for x in 0..wi {
            if x < filled || x % 3 == 0 {
                draw::dot_i(grid, x, base_y);
            }
        }

        for i in 0..n_balls {
            let pivot_x = x_first + i * pitch;
            let theta = if i == 0 {
                left_theta
            } else if i == n_balls - 1 {
                right_theta
            } else {
                0.0
            };
            let bob_x = pivot_x + (rod_len * theta.sin()).round() as i32;
            let bob_y = rod_top + (rod_len * theta.cos()).round() as i32;
            line(grid, pivot_x, rod_top, bob_x, bob_y);
            disc(grid, bob_x, bob_y, ball_r);
        }

        // Tint: base gradient to progress; balls in the leading colour.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (filled.max(0) as usize + 1) / 2;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            draw::tint_row(grid, ch.saturating_sub(1), cx, cx, ctx.palette.sample(t));
        }
        let lo = ((x_first - reach).max(0) as usize) / 2 + 1;
        let hi_c = ((x_first + row_w + reach).max(0) as usize) / 2;
        for cy in 0..ch.saturating_sub(1) {
            draw::tint_row(
                grid,
                cy,
                lo,
                hi_c.saturating_sub(1),
                ctx.palette.sample(ctx.eased),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Orbital (Kepler's laws)
//    Elliptical orbit: x = a·cos(E), y = b·sin(E)
//    where a = semi-major, b = a·√(1−e²), eccentricity e = 0.6.
//    Eccentric anomaly E(t) solved via Kepler's equation: E − e·sin(E) = M
//    Mean anomaly M = 2π·(time mod T)/T.
//    eased draws fraction of the orbit; planet sweeps equal areas in equal times.
// ---------------------------------------------------------------------------
struct Orbital;
impl ProgressStyle for Orbital {
    fn name(&self) -> &str {
        "orbital"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Kepler orbit: planet with a circling moon sweeps an elliptical path around a pulsing star; orbit traced = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        // Ellipse centre, semi-axes and eccentricity (star sits at a focus).
        let cx = wf / 2.0;
        let cy = (hf - 1.0) / 2.0;
        let a = (wf / 2.0 - 4.0).max(1.0);
        let b = (hf / 2.0 - 2.0).max(0.5);
        let e = 0.3_f32;
        let star_x = cx + a * e;

        // Kepler's equation E − e·sin(E) = M, solved by Newton iteration.
        // Mean anomaly M runs 0 → 2π with progress, starting at aphelion.
        let kepler = |m: f32| -> f32 {
            let mut ea = m;
            for _ in 0..6 {
                ea -= (ea - e * ea.sin() - m) / (1.0 - e * ea.cos());
            }
            ea
        };
        let pos = |ea: f32| -> (i32, i32) {
            // E measured from aphelion (left), planet travels clockwise.
            let ang = ea + PI;
            (
                (cx + a * ang.cos()).round() as i32,
                (cy - b * ang.sin()).round() as i32,
            )
        };

        // Orbit path: solid where already swept, dotted ahead.
        let m_now = ctx.eased * 2.0 * PI;
        let e_now = kepler(m_now);
        let steps = (w * 2).max(8);
        for s in 0..=steps {
            let ea = s as f32 / steps as f32 * 2.0 * PI;
            if ea <= e_now || s % 4 == 0 {
                let (x, y) = pos(ea);
                draw::dot_i(grid, x, y);
            }
        }

        // Star at the focus: solid core plus short rays that flicker.
        let sx = star_x.round() as i32;
        let sy = cy.round() as i32;
        let star_r = if h >= 12 { 2 } else { 1 };
        disc(grid, sx, sy, star_r);
        let ray = star_r + 1 + ((ctx.time * 6.0).sin() > 0.0) as i32;
        for &(dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            draw::dot_i(grid, sx + dx * (ray + 1), sy + dy * ray);
        }

        // Planet (solid) with a moon circling it over time.
        let (px, py) = pos(e_now);
        disc(grid, px, py, 1);
        let moon_a = ctx.time * 5.0;
        draw::dot_i(
            grid,
            px + (4.0 * moon_a.cos()).round() as i32,
            py + (2.5 * moon_a.sin()).round() as i32,
        );

        // Tint: swept orbit gradient, star and planet in the end colours.
        let (cw, ch) = grid.dimensions();
        for cx_ in 0..cw {
            let t = cx_ as f32 / cw.max(1) as f32;
            for cy_ in 0..ch {
                draw::tint_row(grid, cy_, cx_, cx_, ctx.palette.sample(t * ctx.eased));
            }
        }
        tint_around(grid, sx, sy, star_r + 2, ctx.palette.end);
        tint_around(grid, px, py, 2, ctx.palette.start);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Terminal velocity
//    An object falling under gravity with drag: F = mg − k·v
//    Solution: v(t) = v_t·(1 − e^(−t/τ))  where v_t = mg/k, τ = m/k
//    Position:  y(t) = v_t·(t − τ·(1 − e^(−t/τ)))
//    eased controls normalised time (0 → T_max = 4τ).
//    Bar fills from the top as the object speeds up, then plateaus.
//    The velocity curve y=v(t) is plotted as a line graph.
// ---------------------------------------------------------------------------
struct TerminalVelocity;
impl ProgressStyle for TerminalVelocity {
    fn name(&self) -> &str {
        "terminal-velocity"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Terminal velocity: v=v_t·(1−e^(−t/τ)) plots toward its asymptote while a falling body's drag arrow grows to match its weight"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Physics: τ = 1 (normalised), v_t = 1
        let tau = 1.0_f32;
        let t_max = 4.0 * tau; // x-axis spans 0..4τ

        // Draw axes
        let ax_x = 1_usize; // y-axis (left edge)
        let ax_y = h.saturating_sub(2); // x-axis (bottom)
        draw::vline(grid, ax_x, 0, ax_y);
        draw::hline(grid, ax_x, w.saturating_sub(1), ax_y);

        // Draw the asymptote v = v_t (top of the graph area)
        let top_y = 1_usize;
        for xi in (ax_x..w).step_by(3) {
            if xi + 1 < w {
                draw::dot(grid, xi, top_y);
            }
        }

        // Draw v(t) curve up to eased fraction of t_max
        let t_now = ctx.eased * t_max;
        let graph_h = (ax_y as f32 - top_y as f32).max(1.0);
        // Leave room on the right for the falling body's force diagram.
        let fbd_w = if w >= 40 { 16 } else { 0 };
        let graph_w = w.saturating_sub(ax_x + 1 + fbd_w) as f32;

        let mut prev: Option<(i32, i32)> = None;
        let steps = w.max(2);
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let t = frac * t_now;
            let v = 1.0 - (-t / tau).exp(); // v/v_t ∈ [0,1)
            let px = (ax_x as f32 + frac * graph_w * (t_now / t_max)) as i32;
            let py = (ax_y as f32 - v * graph_h) as i32;
            let py = py.clamp(top_y as i32, ax_y as i32);
            draw::dot_i(grid, px, py);
            if let Some((lx, ly)) = prev {
                line(grid, lx, ly, px, py);
            }
            prev = Some((px, py));
        }

        // Falling body with its free-body diagram: weight (down, fixed) and
        // drag (up, ∝ v) — the arrows match at terminal velocity. Air streaks
        // rush upward past it faster as it speeds up.
        if fbd_w > 0 {
            let v_now = 1.0 - (-t_now / tau).exp();
            let bx = w as i32 - (fbd_w as i32) / 2;
            let by = (ax_y as i32) / 2;
            disc(grid, bx, by, 2);
            let arrow = |grid: &mut BrailleGrid, y0: i32, len: i32, dir: i32| {
                if len <= 0 {
                    return;
                }
                let tip = y0 + dir * len;
                line(grid, bx, y0, bx, tip);
                draw::dot_i(grid, bx - 1, tip - dir);
                draw::dot_i(grid, bx + 1, tip - dir);
            };
            let max_len = (by - 3).max(1);
            arrow(grid, by + 3, max_len, 1);
            arrow(grid, by - 3, (v_now * max_len as f32).round() as i32, -1);
            // Air streaks either side, scrolling upward with time.
            let streaks = (v_now * 4.0).round() as i32;
            let scroll = (ctx.time * (4.0 + 20.0 * v_now)) as i32;
            for k in 0..streaks {
                for side in [-5_i32, 5] {
                    let y = (ax_y as i32
                        - 1
                        - (scroll + k * 4 + side).rem_euclid((ax_y as i32).max(1)))
                    .max(0);
                    draw::dot_i(grid, bx + side, y);
                    draw::dot_i(grid, bx + side, y + 1);
                }
            }
        }

        // Tint
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Elastic collision (1D)
//    Two equal-mass carts on a frictionless track.
//    Before collision: cart A moves right at v, cart B is stationary.
//    After: A stops, B moves right at v.  (Elastic: Δp = 0, ΔKE = 0)
//    eased divides the bar into three phases:
//       [0, 0.4)  → approach
//       [0.4, 0.6) → collision (overlap + rebound flash)
//       [0.6, 1.0] → separation
//    time provides subtle vibration at the collision instant.
// ---------------------------------------------------------------------------
struct ElasticCollision;
impl ProgressStyle for ElasticCollision {
    fn name(&self) -> &str {
        "elastic-collision"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "1D elastic collision: cart A→B, momentum swaps — three phases: approach, impact, separation"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        let mid_y = (hf / 2.0) as i32;
        let cart_w = (wf / 8.0).max(3.0) as i32;
        let cart_h = (hf / 3.0).max(2.0) as i32;
        let track_y = (mid_y + cart_h / 2 + 1).min(h as i32 - 1);

        // Track
        draw::hline(grid, 0, w.saturating_sub(1), track_y as usize);

        // Phase logic
        let phase = ctx.eased;
        let center_x = (wf / 2.0) as i32;

        let (ax, bx) = if phase < 0.4 {
            // Approach: A moves from left, B is stationary at center+cart_w/2
            let a_frac = phase / 0.4;
            let a_x = (a_frac * (wf * 0.4)) as i32;
            let b_x = center_x;
            (a_x, b_x)
        } else if phase < 0.6 {
            // Collision: both at center (slight overlap, vibrate)
            let vib = ((ctx.time * 20.0).sin() * 2.0) as i32;
            (center_x + vib, center_x + vib)
        } else {
            // Separation: A stops at center, B moves right
            let b_frac = (phase - 0.6) / 0.4;
            let a_x = center_x;
            let b_x = center_x + (b_frac * wf * 0.4) as i32;
            (a_x, b_x)
        };

        // Draw cart A (left cart)
        let a_left = ax;
        let a_top = mid_y - cart_h / 2;
        for dy in 0..cart_h {
            for dx in 0..cart_w {
                draw::dot_i(grid, a_left + dx, a_top + dy);
            }
        }

        // Hollow interior of A
        if cart_w > 2 && cart_h > 2 {
            for dy in 1..cart_h - 1 {
                for dx in 1..cart_w - 1 {
                    // don't draw inside — use outline only
                    let _ = (a_left + dx, a_top + dy); // already filled above; we want solid
                }
            }
        }

        // Draw cart B (right cart)
        let b_left = bx;
        let b_top = mid_y - cart_h / 2;
        for dy in 0..cart_h {
            draw::dot_i(grid, b_left, b_top + dy);
            draw::dot_i(grid, b_left + cart_w - 1, b_top + dy);
        }
        for dx in 0..cart_w {
            draw::dot_i(grid, b_left + dx, b_top);
            draw::dot_i(grid, b_left + dx, b_top + cart_h - 1);
        }

        // Velocity arrows: draw rightward arrows above carts
        let arrow_y = (a_top - 2).max(0);
        // Cart A arrow (before collision it moves; after it stops)
        if phase < 0.4 {
            let tail = a_left;
            let head = (a_left + cart_w + 2).min(w as i32 - 1);
            line(grid, tail, arrow_y, head, arrow_y);
            draw::dot_i(grid, head - 1, arrow_y - 1);
            draw::dot_i(grid, head - 1, arrow_y + 1);
        }
        // Cart B arrow (after collision it moves)
        if phase > 0.6 {
            let tail = b_left + cart_w;
            let head = (b_left + cart_w + 4).min(w as i32 - 1);
            line(grid, tail, arrow_y, head, arrow_y);
            draw::dot_i(grid, head - 1, arrow_y - 1);
            draw::dot_i(grid, head - 1, arrow_y + 1);
        }

        // Tint the bar by progress
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Gravity well (1/r potential)
//    Funnel lines: at radius r from centre, dot-row = y_max·(1 − r_min/r)
//    Plotted as a V-shaped funnel (side profile of a potential well).
//    A test particle spirals inward: r decreases with eased; azimuth from time.
// ---------------------------------------------------------------------------
struct GravityWell;
impl ProgressStyle for GravityWell {
    fn name(&self) -> &str {
        "gravity-well"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Gravity well: spacetime grid sheet dips into a funnel around a mass; a test particle spirals in with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;
        let cx = wf / 2.0;
        let bottom = (hf - 3.0).max(0.0);

        // Sheet depth profile: a softened 1/r dip, g(0) = 1, g(∞) → 0.
        let s = (wf / 10.0).max(1.0);
        let dip = |x: f32| -> f32 {
            let u = (x - cx) / s;
            1.0 / (1.0 + u * u)
        };
        // Surface height of grid line `base` at column x.
        let surf = |base: f32, x: f32| -> f32 { base + (bottom - base) * dip(x) };

        // Three receding grid lines ("latitudes") that all funnel to the mass.
        let bases = [1.0_f32, hf * 0.30, hf * 0.55];
        for &base in &bases {
            let mut prev: Option<(i32, i32)> = None;
            for x in 0..w {
                let y = surf(base, x as f32).round() as i32;
                if let Some((lx, ly)) = prev {
                    line(grid, lx, ly, x as i32, y);
                }
                prev = Some((x as i32, y));
            }
        }
        // "Longitude" struts joining the grid lines, bending into the well.
        let nstrut = 4;
        for k in 1..=nstrut {
            for side in [-1.0_f32, 1.0] {
                let x = cx + side * (k as f32) * (wf / 2.0) / (nstrut as f32 + 0.5);
                let y0 = surf(bases[0], x).round() as i32;
                let y1 = surf(bases[2], x).round() as i32;
                let xi = x.round() as i32;
                line(grid, xi, y0, xi - side as i32, y1);
            }
        }

        // Central mass at the bottom of the funnel.
        let mass_y = bottom.round() as i32;
        disc(grid, cx.round() as i32, mass_y, if h >= 12 { 2 } else { 1 });

        // Test particle: orbit radius shrinks with progress, angular speed
        // rises as it falls (∝ r^-1.5, Kepler), drawn on the sheet surface.
        let r_max = wf / 2.0 - 2.0;
        let r_now = (r_max * (1.0 - ctx.eased)).max(3.0);
        let omega = 1.5 * (r_max / r_now).powf(1.5).min(12.0);
        let phi = ctx.time * omega;
        let at = |r: f32, ph: f32| -> (i32, i32) {
            let x = cx + r * ph.cos();
            let depth = surf(bases[1], cx + r) + 2.0 * ph.sin();
            (x.round() as i32, depth.round() as i32)
        };
        // Spiral wake trailing the particle.
        for i in 1..18 {
            let back = i as f32 * 0.16;
            let r = (r_now + back * 2.5).min(r_max);
            let (x, y) = at(r, phi - back);
            if i % 2 == 0 {
                draw::dot_i(grid, x, y);
            }
        }
        let (px, py) = at(r_now, phi);
        disc(grid, px, py, 1);

        // Tint: sheet in a depth gradient, mass and particle in the end colour.
        let (cw, ch) = grid.dimensions();
        for cy_ in 0..ch {
            let t = (cy_ as f32 + 0.5) / ch.max(1) as f32;
            draw::tint_row(grid, cy_, 0, cw, ctx.palette.sample(t));
        }
        tint_around(grid, cx.round() as i32, mass_y, 2, ctx.palette.end);
        tint_around(grid, px, py, 1, ctx.palette.start);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Relativity light cone
//    The textbook 3-D spacetime diagram: an event (here-now) sits at the
//    centre of a wide "plane of the present"; the past cone opens downward
//    to an elliptical rim, the future cone opens upward. Light spreads at c,
//    so the future cone's height and rim grow with eased; a timelike particle
//    worldline threads both cones and climbs with it.
// ---------------------------------------------------------------------------
struct LightCone;
impl ProgressStyle for LightCone {
    fn name(&self) -> &str {
        "light-cone"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Light cone: hourglass of past and future cones through the here-now plane; future cone and worldline grow with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hi = h as i32;
        let cx = (wf / 2.0) as i32;
        let ey = (hi - 1) / 2;
        // Horizontal stretch so the cones read at bar aspect ratios.
        let slope = 1.7_f32;

        let ellipse = |grid: &mut BrailleGrid, ecx: i32, ecy: i32, a: f32, b: f32, dotted: bool| {
            let steps = ((a * 4.0) as i32).max(8);
            let shift = (ctx.time * 6.0) as i32;
            for s in 0..steps {
                if dotted && (s + shift) % 3 != 0 {
                    continue;
                }
                let t = s as f32 / steps as f32 * 2.0 * PI;
                draw::dot_i(
                    grid,
                    ecx + (a * t.cos()).round() as i32,
                    ecy + (b * t.sin()).round() as i32,
                );
            }
        };

        // Plane of the present: wide dotted ellipse through the event.
        ellipse(grid, cx, ey, wf / 2.0 - 2.0, (hi / 4) as f32, true);

        // Past cone: fully formed, down to a rim near the bottom.
        let past_d = (hi - 2 - ey).max(1);
        let past_a = past_d as f32 * slope;
        line(grid, cx, ey, cx - past_a as i32, ey + past_d);
        line(grid, cx, ey, cx + past_a as i32, ey + past_d);
        ellipse(grid, cx, ey + past_d, past_a, 1.0, false);

        // Future cone: light has reached height `fut_d` so far.
        let fut_max = ey.max(1);
        let fut_d = (1.0 + ctx.eased * (fut_max - 1) as f32).round() as i32;
        let fut_a = fut_d as f32 * slope;
        line(grid, cx, ey, cx - fut_a as i32, ey - fut_d);
        line(grid, cx, ey, cx + fut_a as i32, ey - fut_d);
        ellipse(grid, cx, ey - fut_d, fut_a, 1.0, false);

        // Timelike worldline: wanders (|dx/dy| < 1) up through the event.
        let top = ey - fut_d + 1;
        for y in top..hi {
            let dy = (y - ey) as f32;
            let x = cx as f32 + (dy * 0.45 + ctx.time * 1.5).sin() * 0.8 * dy.abs().min(4.0);
            draw::dot_i(grid, x.round() as i32, y);
        }

        // The event itself.
        for d in -1i32..=1 {
            draw::dot_i(grid, cx + d, ey);
            draw::dot_i(grid, cx, ey + d);
        }

        // Tint: future cone in the end colour, past in the start colour.
        let (cw, ch) = grid.dimensions();
        let ey_cell = (ey.max(0) as usize) / 4;
        for cy in 0..ch {
            let col = match cy.cmp(&ey_cell) {
                std::cmp::Ordering::Less => ctx.palette.sample(0.5 + 0.5 * ctx.eased),
                std::cmp::Ordering::Equal => ctx.palette.sample(0.5),
                std::cmp::Ordering::Greater => ctx.palette.start,
            };
            draw::tint_row(grid, cy, 0, cw, col);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Double-slit interference (wave-particle duality)
//     Two slits at y = h/3 and y = 2h/3.  For each screen position x,
//     intensity I(x) ∝ cos²(π·d·sin(θ)/λ) where sin(θ) ≈ (x−x_screen)/L.
//     eased controls how many particles have accumulated: fringe density
//     scales with eased.  time animates which particles are "in flight".
// ---------------------------------------------------------------------------
struct DoubleSlit;
impl ProgressStyle for DoubleSlit {
    fn name(&self) -> &str {
        "double-slit"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Double slit: plane waves hit a two-slit barrier, ripples interfere, and the screen's cos² fringes build with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;
        let cy = (hi - 1) as f32 / 2.0;

        // Layout: source | plane waves | barrier | ripples | fringe histogram | screen.
        let barrier_x = (wi * 3 / 10).max(1);
        let screen_x = wi - 1;
        let hist_len = (wi / 4).max(1);
        let ripple_end = screen_x - hist_len - 1;

        // Two slits symmetric about the centre line.
        let slit_off = (hi / 5).max(1);
        let slits = [
            (cy - slit_off as f32).round() as i32,
            (cy + slit_off as f32).round() as i32,
        ];
        for y in 0..hi {
            if slits.iter().all(|&sy| (y - sy).abs() > 1) {
                draw::dot_i(grid, barrier_x, y);
                draw::dot_i(grid, barrier_x + 1, y);
            }
        }

        // Source block and incoming plane wavefronts marching right.
        let lambda = 5.0_f32;
        let phase = (ctx.time * 9.0) % lambda;
        for y in (cy as i32 - 2)..=(cy as i32 + 2) {
            draw::dot_i(grid, 0, y);
            draw::dot_i(grid, 1, y);
        }
        let mut x = 3.0 + phase;
        while (x as i32) < barrier_x - 1 {
            for y in 0..hi {
                if y % 2 == 0 {
                    draw::dot_i(grid, x as i32, y);
                }
            }
            x += lambda;
        }

        // Circular ripples spreading from each slit (half-arcs to the right).
        let max_r = (ripple_end - barrier_x).max(1) as f32;
        for ring in 0.. {
            let r = phase + 1.0 + ring as f32 * lambda;
            if r >= max_r {
                break;
            }
            let steps = (r * 3.0) as i32 + 4;
            for &sy in &slits {
                for i in 0..=steps {
                    if i % 2 == 1 {
                        continue;
                    }
                    let a = -PI / 2.0 + PI * i as f32 / steps as f32;
                    let px = barrier_x + 2 + (r * a.cos()).round() as i32;
                    let py = sy + (r * a.sin()).round() as i32;
                    if px <= ripple_end {
                        draw::dot_i(grid, px, py);
                    }
                }
            }
        }

        // Screen, and the accumulated hit histogram growing left from it:
        // I(y) = cos²(π·y/fringe) · envelope, length ∝ progress.
        for y in 0..hi {
            draw::dot_i(grid, screen_x, y);
        }
        let fringe = (hi as f32 / 3.2).max(1.0);
        for y in 0..hi {
            let dy = y as f32 - cy;
            let env = (1.0 - (dy.abs() / (cy + 2.0))).max(0.0);
            let i = (PI * dy / fringe).cos().powi(2) * env;
            let len = (i * ctx.eased * hist_len as f32).round() as i32;
            for k in 0..len {
                draw::dot_i(grid, screen_x - 1 - k, y);
            }
        }

        // Tint: waves in the start colour, fringes in a gradient by progress.
        let (cw, ch) = grid.dimensions();
        let bcell = barrier_x.max(0) as usize / 2;
        let hcell = (ripple_end + 1).max(0) as usize / 2;
        for cy_ in 0..ch {
            draw::tint_row(grid, cy_, 0, bcell.saturating_sub(1), ctx.palette.start);
            draw::tint_row(grid, cy_, bcell + 1, hcell, ctx.palette.sample(0.4));
            draw::tint_row(
                grid,
                cy_,
                hcell + 1,
                cw,
                ctx.palette.sample(0.5 + ctx.eased * 0.5),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Doppler effect
//     A source moves rightward at speed v_s < c_sound.
//     Wavefronts are circles emitted at intervals T = λ/c.
//     The source position when the nth wavefront was emitted:
//       x_n = v_s · n · T  (moves right over time)
//     Wavefront n at current time has radius r_n = c · (t − n·T).
//     eased → source speed (0 = stationary, 1 = Mach 0.9).
//     time advances so the wavefronts expand visually.
// ---------------------------------------------------------------------------
struct Doppler;
impl ProgressStyle for Doppler {
    fn name(&self) -> &str {
        "doppler"
    }
    fn theme(&self) -> &str {
        "physics"
    }
    fn describe(&self) -> &str {
        "Doppler effect: a source drives across with progress; its circular wavefronts bunch up ahead and stretch out behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        let c_sound = 24.0_f32; // wave speed in dots/sec
        let v_src = c_sound * 0.6; // source speed: Mach 0.6
        let period = 0.26_f32; // emission period
        let n_waves = 8_usize; // number of wavefronts shown

        // Source position is the progress: it drives left → right across
        // the bar while emitting continuously (wavefronts left in its wake).
        let margin = 4.0_f32;
        let src_x = margin + ctx.eased * (wf - 2.0 * margin).max(0.0);
        let src_y = (hf - 1.0) / 2.0;
        let t_phase = ctx.time % period;

        // Draw each wavefront as a circle (dot approximation)
        for n in 0..n_waves {
            let age = t_phase + n as f32 * period; // time since emission
            let radius = c_sound * age; // wavefront radius in dots
            let emit_x = src_x - v_src * age; // source was here at emission

            if radius < 0.5 {
                continue;
            }

            // Draw circle outline (dot approximation)
            let circ_steps = ((radius * PI * 2.0) as usize + 8).clamp(8, 128);
            let mut prev_c: Option<(i32, i32)> = None;
            for s in 0..=circ_steps {
                let angle = s as f32 / circ_steps as f32 * 2.0 * PI;
                let cx = (emit_x + radius * angle.cos()) as i32;
                let cy = (src_y + radius * angle.sin()) as i32;
                draw::dot_i(grid, cx, cy);
                if let Some((lx, ly)) = prev_c {
                    if (cx - lx).abs() + (cy - ly).abs() > 2 {
                        line(grid, lx, ly, cx, cy);
                    }
                }
                prev_c = Some((cx, cy));
            }
        }

        // Draw source: a solid emitter.
        let src_ix = src_x as i32;
        let src_iy = src_y.round() as i32;
        disc(grid, src_ix, src_iy, 2);

        // Direction arrow above source
        let arr_y = (src_iy - 5).max(1);
        let arr_len = (v_src / c_sound * 6.0 + 1.0) as i32;
        line(grid, src_ix, arr_y, src_ix + arr_len, arr_y);
        draw::dot_i(grid, src_ix + arr_len - 1, arr_y - 1);
        draw::dot_i(grid, src_ix + arr_len - 1, arr_y + 1);

        // Tint
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
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
    let styles = progress::styles::physics::styles();
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
