//! `cultures` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O cultures.rs && ./cultures [style-name]
//! ```

const DEFAULT_STYLE: &str = "mandala";

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
    pub mod cultures {
//! World-cultures / ornament-themed progress bars.
//!
//! Eleven structurally distinct styles, each drawing itself as `ctx.eased` rises
//! and animating via `ctx.time`.  Every style draws a recognisable cultural
//! motif or ornament in braille dot-space:
//!
//! - `mandala`        — turning lotus mandala; rosette border unrolls outward
//! - `celtic-knot`    — two outlined ribbons twisting with over/under breaks
//! - `aztec-fret`     — xicalcoliuhqui staircase-and-spiral units, left to right
//! - `islamic-star`   — chain of eight-pointed {8/3} stars blooming in turn
//! - `greek-key`      — one unbroken squared meander line drawn in from the left
//! - `seigaiha`       — Japanese overlapping fans of concentric wave arcs
//! - `totem-pole`     — carved poles with thunderbird / bear crests raised in turn
//! - `runes`          — Elder Futhark runes chiselled stroke by stroke into stone
//! - `paisley-swirl`  — boteh teardrops with curled tips traced one by one
//! - `kente-weave`    — gold / green / red kente strips woven one by one
//! - `paper-lanterns` — string of red paper lanterns lit one after another

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Public registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `cultures` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per ornament style, in display order.
/// Every style is structurally distinct — variety is in shape, not colour.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Mandala),
        Box::new(CelticKnot),
        Box::new(AztecFret),
        Box::new(IslamicStar),
        Box::new(GreekKey),
        Box::new(Seigaiha),
        Box::new(TotemPole),
        Box::new(Runes),
        Box::new(PaisleySwirl),
        Box::new(KenteWeave),
        Box::new(PaperLanterns),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Integer Bresenham line rasteriser. Step-bounded; `draw::dot_i` clips OOB.
fn bres(grid: &mut BrailleGrid, mut x0: i32, mut y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx - dy + 2).unsigned_abs() as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps + 2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Draw a circle arc (dots) from angle `a0` to `a1` (radians) around `(cx,cy)`
/// with dot-space radius `r`.  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r < 0.5 {
        return;
    }
    let arc_len = (a1 - a0).abs() * r;
    let steps = (arc_len.ceil() as usize + 2).max(4);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = a0 + t * (a1 - a0);
        let px = (cx + r * a.cos()).round() as i32;
        let py = (cy + r * a.sin()).round() as i32;
        draw::dot_i(grid, px, py);
    }
}

/// Tint every cell in the grid with a horizontal palette gradient.
fn palette_tint(grid: &mut BrailleGrid, ctx: &BarContext) {
    let (cw, ch) = grid.dimensions();
    for cy in 0..ch {
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
        }
    }
}

/// Blend colour `a` toward `b` by `t` in `[0, 1]`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    Color::rgb(f(a.r, b.r), f(a.g, b.g), f(a.b, b.b))
}

/// Stamp a `#`-art sprite: every `#` becomes a `block × block` square of dots
/// whose top-left is `origin + (col, row) * block`. `flip_v` mirrors the rows.
/// Only dots with `x < clip.0` and `y >= clip.1` are set, so sprites can be
/// revealed left-to-right or bottom-up.
fn stamp(
    grid: &mut BrailleGrid,
    rows: &[&str],
    origin: (i32, i32),
    block: i32,
    flip_v: bool,
    clip: (i32, i32),
) {
    let n = rows.len();
    for (r, line) in rows.iter().enumerate() {
        let r = if flip_v { n - 1 - r } else { r } as i32;
        for (c, ch) in line.bytes().enumerate() {
            if ch != b'#' {
                continue;
            }
            for dy in 0..block {
                for dx in 0..block {
                    let x = origin.0 + c as i32 * block + dx;
                    let y = origin.1 + r * block + dy;
                    if x < clip.0 && y >= clip.1 {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Mandala — a rotating lotus mandala with a rosette border unrolling outward
// ─────────────────────────────────────────────────────────────────────────────

struct Mandala;
impl ProgressStyle for Mandala {
    fn name(&self) -> &str {
        "mandala"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Lotus mandala: a ringed six-petal lotus turns slowly at the centre while \
         a border of rosettes and diamonds unrolls symmetrically outward as progress rises."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw as f32 - 1.0) / 2.0;
        let cy = (dh as f32 - 1.0) / 2.0;
        let r = cy.min(cx).max(1.0);
        let rot = ctx.time * 0.6;

        // Core: outer ring, six lens-shaped petals around an inner ring, and a seed.
        arc(grid, cx, cy, r, 0.0, 2.0 * PI);
        let n_petals = 6usize;
        let steps = (r * 2.0) as usize + 4;
        for k in 0..n_petals {
            let a = k as f32 * 2.0 * PI / n_petals as f32 + rot;
            let (ca, sa) = (a.cos(), a.sin());
            for s in 0..=steps {
                let u = s as f32 / steps as f32;
                let rr = r * (0.42 + 0.58 * u);
                let half = (PI * u).sin() * r * 0.22;
                for side in [-1.0f32, 1.0] {
                    let px = cx + rr * ca - side * half * sa;
                    let py = cy + rr * sa + side * half * ca;
                    draw::dot_i(grid, px.round() as i32, py.round() as i32);
                }
            }
        }
        arc(grid, cx, cy, r * 0.3, 0.0, 2.0 * PI);
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

        // Border: rosettes (ring + seed + four turning spokes) alternating with
        // small diamonds, mirrored left and right, revealed out to `reach`.
        let reach = ctx.eased * (cx - r - 1.0).max(0.0);
        let rose_r = (r * 0.4).max(1.0);
        let pitch = (rose_r * 3.4).max(4.0);
        let mut d = r + 2.0 + rose_r;
        while d + rose_r <= r + 1.0 + reach + 0.5 && d < cx + pitch {
            for side in [-1.0f32, 1.0] {
                let rx = cx + side * d;
                arc(grid, rx, cy, rose_r, 0.0, 2.0 * PI);
                draw::dot_i(grid, rx.round() as i32, cy.round() as i32);
                for q in 0..4 {
                    let a = q as f32 * PI / 2.0 - side * rot;
                    let px = rx + (rose_r + 2.0) * a.cos();
                    let py = cy + (rose_r + 2.0) * a.sin();
                    draw::dot_i(grid, px.round() as i32, py.round() as i32);
                }
                // Diamond halfway to the next rosette, once it is reached too.
                let dd = d + pitch / 2.0;
                if dd + 2.0 <= r + 1.0 + reach + 0.5 {
                    let (mx, my) = ((cx + side * dd).round() as i32, cy.round() as i32);
                    bres(grid, mx - 2, my, mx, my - 2);
                    bres(grid, mx, my - 2, mx + 2, my);
                    bres(grid, mx + 2, my, mx, my + 2);
                    bres(grid, mx, my + 2, mx - 2, my);
                }
            }
            d += pitch;
        }

        // Colour radiates from the centre: start hue at the core, end at the rim.
        let (cw, ch) = grid.dimensions();
        for cx_c in 0..cw {
            let t = ((cx_c as f32 * 2.0 + 1.0) - cx).abs() / cx.max(1.0);
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Celtic Knot — two outlined ribbons twisting with over/under breaks
// ─────────────────────────────────────────────────────────────────────────────

struct CelticKnot;
impl ProgressStyle for CelticKnot {
    fn name(&self) -> &str {
        "celtic-knot"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Celtic interlace: two outlined ribbons twist between border rules, each \
         passing over then under the other — the lower ribbon breaks at every crossing; \
         the knotwork extends rightward as progress rises and slowly flows with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let filled_w = (ctx.eased * dw as f32).round() as usize;
        if filled_w == 0 {
            return Ok(());
        }

        // Border rules frame the band.
        let last = filled_w.min(dw) - 1;
        draw::hline(grid, 0, last, 0);
        draw::hline(grid, 0, last, dh - 1);

        let cy = (dh as f32 - 1.0) / 2.0;
        // Strand amplitude leaves room above and below for the rules.
        let amp = (dh as f32 / 2.0 - 4.0).max(0.5);
        let period = (dh as f32 * 2.5).max(8.0);
        let phase = ctx.time * 1.2;
        // Ribbon half-width, and the separation below which the under-strand
        // is cut away so the over-strand reads as passing on top.
        let half = if dh >= 12 { 2 } else { 1 };
        let gap = (dh as f32 * 0.3).max(1.5);
        let strand = |x: i32, sign: f32| {
            let theta = 2.0 * PI * x as f32 / period + phase;
            (cy + sign * amp * theta.sin()).round() as i32
        };

        for x in 0..filled_w.min(dw) as i32 {
            let theta = 2.0 * PI * x as f32 / period + phase;
            let sep = (2.0 * amp * theta.sin()).abs();
            // Crossings sit at multiples of π; alternate which strand is on top.
            let a_on_top = ((theta / PI).round() as i64).rem_euclid(2) == 0;
            for (sign, top) in [(1.0f32, a_on_top), (-1.0, !a_on_top)] {
                if sep < gap && !top {
                    continue;
                }
                // Outlined ribbon: both edges, joined to the previous column.
                let (y0, y1) = (strand(x - 1, sign), strand(x, sign));
                for e in [-half, half] {
                    bres(grid, x - 1, y0 + e, x, y1 + e);
                }
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Aztec Fret — xicalcoliuhqui: staircase + spiral hook, tiled left to right
// ─────────────────────────────────────────────────────────────────────────────

/// One xicalcoliuhqui unit (2×2-dot blocks): a staircase climbing into a
/// squared spiral hook.
const FRET_UNIT: [&str; 6] = [
    "....######",
    "....#....#",
    "...##.##.#",
    "..##..#..#",
    ".##...####",
    "##........",
];

struct AztecFret;
impl ProgressStyle for AztecFret {
    fn name(&self) -> &str {
        "aztec-fret"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Aztec/Maya stepped fret (xicalcoliuhqui): bold staircase-and-spiral units, \
         alternately mirrored, are laid between border bands from left to right as \
         progress rises; a glint of light sweeps across the finished stone."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let fill_x = (ctx.eased * dw as f32).round() as i32;
        if fill_x <= 0 {
            return Ok(());
        }

        // Border bands top and bottom.
        let last = (fill_x as usize).min(dw) - 1;
        draw::hline(grid, 0, last, 0);
        draw::hline(grid, 0, last, dh - 1);

        // Units are 20×12 dots, centred vertically, alternately flipped so the
        // staircases interlock into a zigzag.
        let y0 = (dh as i32 - 12) / 2;
        let pitch = 20i32;
        let n_units = dw as i32 / pitch + 1;
        for u in 0..n_units {
            stamp(
                grid,
                &FRET_UNIT,
                (u * pitch, y0),
                2,
                u % 2 == 1,
                (fill_x, i32::MIN),
            );
        }

        // Stone colour with a moving glint over the laid section.
        let (cw, ch) = grid.dimensions();
        let glint = (ctx.time * 0.35).fract() * (cw as f32 + 8.0) - 4.0;
        let white = Color::rgb(255, 255, 255);
        for cx_c in 0..cw {
            let base = ctx.palette.sample(cx_c as f32 / cw.max(1) as f32);
            let near = 1.0 - ((cx_c as f32 - glint).abs() / 2.5).min(1.0);
            let col = mix(base, white, near * 0.6);
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Islamic Star — a chain of eight-pointed stars (two interlaced squares)
// ─────────────────────────────────────────────────────────────────────────────

struct IslamicStar;
impl ProgressStyle for IslamicStar {
    fn name(&self) -> &str {
        "islamic-star"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Islamic geometric: eight-pointed interlaced stars ({8/3} star polygons \
         around a ring) bloom into place one after another along a linked chain as progress \
         rises; the rings breathe and the stars glimmer with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Star circumradius fills the height; stars are linked across a gap.
        let s = ((dh as f32 - 1.0) / 2.0).max(1.0);
        let link = 3.0f32;
        let pitch = 2.0 * s + link;
        let n = (((dw as f32 + link) / pitch) as usize).max(1);
        let margin = (dw as f32 - n as f32 * pitch + link) / 2.0;
        let cy = (dh as f32 - 1.0) / 2.0;
        let shown = ctx.eased * n as f32;

        for i in 0..n {
            let f = (shown - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            // The newest star blooms outward from its centre.
            let k = 1.0 - (1.0 - f) * (1.0 - f);
            let cx = margin + i as f32 * pitch + s;
            let r = s * k;
            let p = |x: f32, y: f32| ((cx + x).round() as i32, (cy + y).round() as i32);
            // {8/3} star polygon: eight points, each joined to the third along.
            let pts: Vec<(i32, i32)> = (0..8)
                .map(|j| {
                    let a = j as f32 * PI / 4.0 + PI / 8.0;
                    p(r * a.cos(), r * a.sin())
                })
                .collect();
            for j in 0..8 {
                let (a, b) = (pts[j], pts[(j + 3) % 8]);
                bres(grid, a.0, a.1, b.0, b.1);
            }
            // Breathing inner ring.
            let ring_r = s * 0.32 * k * (0.85 + 0.15 * (ctx.time * 3.0 + i as f32).sin());
            arc(grid, cx, cy, ring_r, 0.0, 2.0 * PI);
            // Link to the previous star.
            if i > 0 && f >= 1.0 {
                let (lx0, ly) = p(-s - link, 0.0);
                let (lx1, _) = p(-s, 0.0);
                if lx0 >= 0 && ly >= 0 {
                    draw::hline(grid, lx0 as usize, lx1.max(0) as usize, ly as usize);
                }
            }
        }

        // Gradient with a glimmer wandering from star to star.
        let (cw, ch) = grid.dimensions();
        let white = Color::rgb(255, 255, 255);
        for cx_c in 0..cw {
            let star = ((cx_c as f32 * 2.0 - margin) / pitch).floor();
            let glow = ((ctx.time * 2.0 - star * 0.9).sin() * 0.5 + 0.5).powi(4);
            let col = mix(
                ctx.palette.sample(cx_c as f32 / cw.max(1) as f32),
                white,
                glow * 0.5,
            );
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Greek Key — a continuous single-line meander drawn in from the left
// ─────────────────────────────────────────────────────────────────────────────

/// One meander period (2×2-dot blocks). Tiled edge to edge, the bottom rows
/// join into one unbroken line that climbs and spirals in every period.
const MEANDER_UNIT: [&str; 6] = ["#####.", "#...#.", "#.#.#.", "#.###.", "#.....", "######"];

struct GreekKey;
impl ProgressStyle for GreekKey {
    fn name(&self) -> &str {
        "greek-key"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Greek key / meander: one unbroken squared line climbs and spirals inward \
         again and again between two border rules; it is drawn in from the left as \
         progress rises and slowly travels along the band with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let fill_x = (ctx.eased * dw as f32).round() as i32;
        if fill_x <= 0 {
            return Ok(());
        }

        // Border rules.
        let last = (fill_x as usize).min(dw) - 1;
        draw::hline(grid, 0, last, 0);
        draw::hline(grid, 0, last, dh - 1);

        // Meander band, 12 dots tall, travelling one period every few seconds.
        let period = 12i32;
        let y0 = (dh as i32 - 12) / 2;
        let shift = ((ctx.time * 3.0) as i32).rem_euclid(period);
        let n_units = dw as i32 / period + 2;
        for u in -1..n_units {
            stamp(
                grid,
                &MEANDER_UNIT,
                (u * period + shift, y0),
                2,
                false,
                (fill_x, i32::MIN),
            );
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Seigaiha — Japanese overlapping wave-scales of concentric arcs
// ─────────────────────────────────────────────────────────────────────────────

struct Seigaiha;
impl ProgressStyle for Seigaiha {
    fn name(&self) -> &str {
        "seigaiha"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Japanese seigaiha (blue ocean waves): fans of concentric arcs overlap like \
         fish scales, each row tucked behind the one below; scales are laid in from \
         the left as progress rises and the sea drifts gently with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Scale radius, horizontal pitch (one diameter) and row spacing.
        let r = (dh as f32 / 2.0).clamp(2.0, 8.0);
        let sx = 2.0 * r;
        let vy = r * 0.7;
        let drift = (ctx.time * 2.5).rem_euclid(sx);
        let reveal_x = ctx.eased * (dw as f32 + r);

        for y in 0..dh {
            let yf = y as f32;
            // Rows whose circles can reach this dot, front (lowest) first.
            let k_hi = ((yf + r) / vy).floor() as i32;
            let k_lo = ((yf - r) / vy).ceil() as i32;
            for x in 0..dw {
                let xf = x as f32;
                let mut k = k_hi;
                while k >= k_lo {
                    let ccy = k as f32 * vy;
                    let off = if k.rem_euclid(2) == 1 { r } else { 0.0 } + drift;
                    let ccx = ((xf - off) / sx).round() * sx + off;
                    let d = ((xf - ccx).powi(2) + (yf - ccy).powi(2)).sqrt();
                    if ccx <= reveal_x && d < r {
                        // The front-most scale owns this dot: concentric rings.
                        if (r - d).rem_euclid(r / 3.0) < 1.0 && d > 1.5 {
                            draw::dot(grid, x, y);
                        }
                        break;
                    }
                    k -= 1;
                }
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Totem Pole — a row of carved poles raised one by one
// ─────────────────────────────────────────────────────────────────────────────

/// Thunderbird crest, wings raised (11 dots wide, 4 tall).
const BIRD_UP: [&str; 4] = ["#...###...#", "##..#.#..##", ".#########.", "...#####..."];
/// Thunderbird crest, wings lowered.
const BIRD_DOWN: [&str; 4] = ["....###....", "....#.#....", ".#########.", "##.#####.##"];
/// Bear-head crest with rounded ears.
const BEAR_HEAD: [&str; 4] = ["..##...##..", "..#######..", "..#.###.#..", "..#######.."];
/// The carved shaft below the crest: two stacked faces with hollow eyes and
/// mouths (12 dots tall).
const POLE_BODY: [&str; 12] = [
    "..#######..",
    "..#..#..#..",
    "..#..#..#..",
    "..#######..",
    "..##...##..",
    "..#######..",
    ".#########.",
    "..#.###.#..",
    "..#######..",
    "..###.###..",
    "..#.....#..",
    "..#######..",
];

struct TotemPole;
impl ProgressStyle for TotemPole {
    fn name(&self) -> &str {
        "totem-pole"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Totem poles: a row of carved poles — stacked faces with hollow eyes, crowned \
         by a thunderbird or a bear — rises one by one from the ground as progress \
         climbs; the thunderbirds beat their wings with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Poles are 11×16 dots on a 12-dot pitch, centred, standing on the floor.
        let pitch = 12i32;
        let n = ((dw as i32 + 1) / pitch).max(1);
        let margin = (dw as i32 - n * pitch + 1) / 2;
        let y0 = dh as i32 - 16;
        let built = ctx.eased * n as f32;

        for i in 0..n {
            let f = (built - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            // The pole being raised is revealed from its base upward.
            let rows = (f * 16.0).ceil() as i32;
            let clip = (i32::MAX, y0 + 16 - rows);
            let x0 = margin + i * pitch;
            let flap = ((ctx.time * 4.0) as i64 + i64::from(i)).rem_euclid(2) == 0;
            let crest: &[&str] = match (i % 2, flap) {
                (0, true) => &BIRD_UP,
                (0, false) => &BIRD_DOWN,
                _ => &BEAR_HEAD,
            };
            stamp(grid, crest, (x0, y0), 1, false, clip);
            stamp(grid, &POLE_BODY, (x0, y0 + 4), 1, false, clip);
        }

        // Shaft in the palette gradient; crests picked out in a brighter accent.
        palette_tint(grid, ctx);
        let (cw, _) = grid.dimensions();
        if y0 >= 0 {
            let crest = mix(ctx.palette.end, Color::rgb(255, 255, 255), 0.35);
            draw::tint_row(grid, y0 as usize / 4, 0, cw.saturating_sub(1), crest);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Runes — Elder Futhark runes chiselled stroke by stroke into a stone band
// ─────────────────────────────────────────────────────────────────────────────

/// Elder Futhark runes as stroke lists `(x0, y0, x1, y1)` in a 7×12 box,
/// in futhark order: fehu, uruz, thurisaz, ansuz, raidho, kaunan, gebo,
/// hagalaz, naudiz, isa, algiz, sowilo, tiwaz, berkanan, mannaz, othala.
const RUNE_STROKES: [&[(i32, i32, i32, i32)]; 16] = [
    &[(1, 0, 1, 11), (1, 4, 5, 0), (1, 7, 5, 3)],
    &[(1, 0, 1, 11), (1, 0, 5, 4), (5, 4, 5, 11)],
    &[(1, 0, 1, 11), (1, 3, 5, 6), (5, 6, 1, 9)],
    &[(1, 0, 1, 11), (1, 0, 5, 4), (1, 4, 5, 8)],
    &[(1, 0, 1, 11), (1, 0, 5, 3), (5, 3, 1, 6), (1, 6, 5, 11)],
    &[(5, 2, 1, 6), (1, 6, 5, 10)],
    &[(0, 1, 6, 10), (6, 1, 0, 10)],
    &[(0, 0, 0, 11), (5, 0, 5, 11), (0, 3, 5, 7)],
    &[(3, 0, 3, 11), (0, 4, 6, 8)],
    &[(3, 0, 3, 11)],
    &[(3, 0, 3, 11), (3, 5, 0, 1), (3, 5, 6, 1)],
    &[(5, 0, 1, 4), (1, 4, 5, 7), (5, 7, 1, 11)],
    &[(3, 0, 3, 11), (3, 0, 0, 4), (3, 0, 6, 4)],
    &[
        (1, 0, 1, 11),
        (1, 0, 5, 3),
        (5, 3, 1, 5),
        (1, 5, 5, 8),
        (5, 8, 1, 11),
    ],
    &[(0, 0, 0, 11), (5, 0, 5, 11), (0, 0, 5, 6), (5, 0, 0, 6)],
    &[(3, 0, 6, 3), (3, 0, 0, 3), (6, 3, 0, 10), (0, 3, 6, 10)],
];

struct Runes;
impl ProgressStyle for Runes {
    fn name(&self) -> &str {
        "runes"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Norse Elder Futhark runes: angular rune-staves are chiselled stroke by stroke \
         into a stone band, in futhark order, as progress rises; sparks flicker at \
         the chisel point."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Stone band rules top and bottom.
        draw::hline(grid, 0, dw - 1, 0);
        draw::hline(grid, 0, dw - 1, dh - 1);

        // Rune slots on a 10-dot pitch; strokes are two dots thick.
        let pitch = 10i32;
        let n = ((dw as i32 - 2) / pitch).max(1);
        let margin = (dw as i32 - n * pitch + 2) / 2;
        let oy = if dh >= 8 { 2 } else { 0 };
        let sy = ((dh as f32 - 1.0 - 2.0 * oy as f32) / 11.0).max(0.0);
        let carved = ctx.eased * n as f32;
        let spark_on = ((ctx.time * 10.0) as i64).rem_euclid(2) == 0;

        for i in 0..n {
            let f = (carved - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let strokes = RUNE_STROKES[i as usize % RUNE_STROKES.len()];
            let x0 = margin + i * pitch;
            let map = |x: i32, y: i32| (x0 + x, oy + (y as f32 * sy).round() as i32);
            for (s, &(ax, ay, bx, by)) in strokes.iter().enumerate() {
                let g = (f * strokes.len() as f32 - s as f32).clamp(0.0, 1.0);
                if g <= 0.0 {
                    break;
                }
                let (px0, py0) = map(ax, ay);
                let (px1, py1) = map(bx, by);
                let ex = (px0 as f32 + (px1 - px0) as f32 * g).round() as i32;
                let ey = (py0 as f32 + (py1 - py0) as f32 * g).round() as i32;
                bres(grid, px0, py0, ex, ey);
                bres(grid, px0 + 1, py0, ex + 1, ey);
                // Chisel sparks at the stroke being cut.
                if g < 1.0 && spark_on {
                    for (dx, dy) in [(-2, -2), (3, -2), (-2, 2), (3, 2)] {
                        draw::dot_i(grid, ex + dx, ey + dy);
                    }
                }
            }
        }

        // Carved section in the palette gradient; uncut stone stays dim.
        let (cw, ch) = grid.dimensions();
        let cut_cell = ((margin as f32 + carved * pitch as f32) / 2.0).max(0.0) as usize;
        let stone = mix(ctx.palette.track, ctx.palette.start, 0.3);
        for cx_c in 0..cw {
            let col = if cx_c <= cut_cell {
                ctx.palette.sample(cx_c as f32 / cw.max(1) as f32)
            } else {
                stone
            };
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Paisley Swirl — boteh teardrops traced one after another
// ─────────────────────────────────────────────────────────────────────────────

/// Append a quadratic Bézier `p0 → p2` (control `c`) to `pts`.
fn quad(pts: &mut Vec<(f32, f32)>, p0: (f32, f32), c: (f32, f32), p2: (f32, f32)) {
    let len = ((c.0 - p0.0).hypot(c.1 - p0.1) + (p2.0 - c.0).hypot(p2.1 - c.1)).max(1.0);
    let n = (len * 2.0).ceil() as usize;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let u = 1.0 - t;
        pts.push((
            u * u * p0.0 + 2.0 * u * t * c.0 + t * t * p2.0,
            u * u * p0.1 + 2.0 * u * t * c.1 + t * t * p2.1,
        ));
    }
}

/// Append a circular arc around `c` from angle `a0` to `a1` to `pts`.
fn ring(pts: &mut Vec<(f32, f32)>, c: (f32, f32), r: f32, a0: f32, a1: f32) {
    let n = ((a1 - a0).abs() * r * 2.0).ceil().max(4.0) as usize;
    for i in 0..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        pts.push((c.0 + r * a.cos(), c.1 + r * a.sin()));
    }
}

/// Trace order of one paisley (boteh) of bowl radius `r`, in y-up local
/// coordinates centred on the bowl: rounded bowl, left flank up to the tip,
/// the curled hook, the right flank back down, then the inner eye.
fn paisley_points(r: f32) -> Vec<(f32, f32)> {
    let mut pts = Vec::new();
    ring(&mut pts, (0.0, 0.0), r, 0.0, -PI);
    let tip = (0.9 * r, 2.0 * r);
    quad(&mut pts, (-r, 0.0), (-r, 1.8 * r), tip);
    let cr = 0.4 * r;
    let cc = (tip.0, tip.1 - cr);
    ring(&mut pts, cc, cr, PI / 2.0, -PI);
    quad(&mut pts, (cc.0 + cr, cc.1), (1.25 * r, 0.7 * r), (r, 0.0));
    ring(&mut pts, (0.0, 0.15 * r), 0.4 * r, 0.0, 2.0 * PI);
    pts.push((0.0, 0.15 * r));
    pts
}

struct PaisleySwirl;
impl ProgressStyle for PaisleySwirl {
    fn name(&self) -> &str {
        "paisley-swirl"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Paisley (boteh): teardrop motifs with a curled tip and an inner eye are \
         traced one after another, alternately mirrored, as progress rises; each \
         sways gently on its base with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Bowl radius from the height: the motif stands 3r tall, 2.3r wide.
        let r = ((dh as f32 - 1.0) / 3.0).clamp(0.5, 8.0);
        let pitch = (r * 3.2).max(2.0);
        let n = ((dw as f32 / pitch) as usize).max(1);
        let margin = (dw as f32 - n as f32 * pitch) / 2.0 + 0.2 * pitch;
        let pts = paisley_points(r);
        let traced = ctx.eased * n as f32;

        for i in 0..n {
            let f = (traced - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let shown = ((f * pts.len() as f32).ceil() as usize).min(pts.len());
            let bx = margin + i as f32 * pitch + r;
            let by = dh as f32 - 1.0 - r;
            let mirror = i % 2 == 1;
            let sway = 0.12 * (ctx.time * 1.8 + i as f32).sin();
            let (cs, sn) = (sway.cos(), sway.sin());
            for &(x, y) in &pts[..shown] {
                let x = if mirror { 0.3 * r - x } else { x };
                let px = bx + x * cs - y * sn;
                let py = by - (x * sn + y * cs);
                draw::dot_i(grid, px.round() as i32, py.round() as i32);
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Kente Weave — narrow kente strips woven one by one, side by side
// ─────────────────────────────────────────────────────────────────────────────

/// Kente gold, green and red — the cloth's signature colours (black is the
/// unlit gaps between strips).
const KENTE_GOLD: Color = Color::rgb(240, 180, 30);
const KENTE_GREEN: Color = Color::rgb(30, 160, 70);
const KENTE_RED: Color = Color::rgb(210, 45, 40);

struct KenteWeave;
impl ProgressStyle for KenteWeave {
    fn name(&self) -> &str {
        "kente-weave"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Kente cloth: narrow strips of gold, green and red blocks — weft bars, warp \
         stripes, checks and solid panels — are woven one by one and laid side by \
         side as progress rises; a shuttle darts across the strip on the loom."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Strips are 7 dots of cloth plus a 1-dot seam, one cell-row per block.
        let strip = 8usize;
        let n = (dw / strip).max(1);
        let woven = ctx.eased * n as f32;
        let (cw, ch) = grid.dimensions();

        for i in 0..n {
            let f = (woven - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let x0 = i * strip;
            // The strip on the loom is woven top-down.
            let rows = ((f * dh as f32).ceil() as usize).min(dh);
            for y in 0..rows {
                let block = y / 4;
                let motif = (i * 3 + block) % 4;
                for dx in 0..strip - 1 {
                    let (lx, ly) = (dx, y % 4);
                    let on = match motif {
                        0 => ly % 2 == 0,                // weft bars
                        1 => true,                       // solid panel
                        2 => lx % 2 == 0,                // warp stripes
                        _ => (lx / 2 + ly / 2) % 2 == 0, // checks
                    };
                    if on {
                        draw::dot(grid, x0 + dx, y);
                    }
                }
            }

            // Colour each block cell: gold dominates, green / red accents.
            for block in 0..ch {
                let col = match (i + block) % 4 {
                    0 | 2 => KENTE_GOLD,
                    1 => KENTE_GREEN,
                    _ => KENTE_RED,
                };
                let c0 = x0 / 2;
                draw::tint_row(
                    grid,
                    block,
                    c0,
                    (c0 + strip / 2 - 1).min(cw.saturating_sub(1)),
                    col,
                );
            }

            // Shuttle darting across the weaving line.
            if f < 1.0 {
                let sx = x0 as f32 + (0.5 + 0.5 * (ctx.time * 9.0).sin()) * (strip - 2) as f32;
                let sy = rows.min(dh - 1);
                draw::fill_rect(grid, sx as usize, sy, 2, 1);
                draw::tint_row(
                    grid,
                    sy / 4,
                    sx as usize / 2,
                    sx as usize / 2,
                    Color::rgb(255, 255, 255),
                );
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Paper Lanterns — a string of round lanterns lit one after another
// ─────────────────────────────────────────────────────────────────────────────

/// Unlit lantern: paper outline, caps and tassel cord (9×12 dots).
const LANTERN_DARK: [&str; 12] = [
    "...###...",
    ".##...##.",
    "#.......#",
    "#.......#",
    "#.......#",
    "#.......#",
    "#.......#",
    ".##...##.",
    "...###...",
    "....#....",
    "....#....",
    ".........",
];
/// Lit lantern: glowing body with two dark ribs, caps and a full tassel.
const LANTERN_LIT: [&str; 12] = [
    "...###...",
    ".#######.",
    "##.###.##",
    "##.###.##",
    "##.###.##",
    "##.###.##",
    "##.###.##",
    ".#######.",
    "...###...",
    "....#....",
    "...###...",
    "...###...",
];

struct PaperLanterns;
impl ProgressStyle for PaperLanterns {
    fn name(&self) -> &str {
        "paper-lanterns"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Paper lanterns: round red lanterns with gold caps and tassels hang from a \
         sagging festival string; each glows alight from the bottom up in turn as \
         progress rises, and they sway and flicker with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Festival string: a shallow sag across the full width.
        let sag = |x: f32| {
            let u = x / dw.max(1) as f32 * 2.0 - 1.0;
            (1.5 * (1.0 - u * u)).round() as i32
        };
        for x in 0..dw {
            draw::dot_i(grid, x as i32, sag(x as f32));
        }

        // Lanterns: 9 dots wide on a 12-dot pitch, hung 2 dots below the string.
        let pitch = 12i32;
        let n = ((dw as i32 + 3) / pitch).max(1);
        let margin = (dw as i32 - n * pitch + 3) / 2;
        let lit = ctx.eased * n as f32;
        let (cw, ch) = grid.dimensions();
        let red = Color::rgb(225, 40, 35);
        let gold = Color::rgb(245, 185, 45);
        let dim = mix(ctx.palette.track, Color::rgb(160, 160, 170), 0.35);
        for cy_c in 0..ch {
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), mix(gold, dim, 0.4));
        }

        for i in 0..n {
            let sway = (1.2 * (ctx.time * 2.0 + i as f32 * 0.7).sin()).round() as i32;
            let x0 = margin + i * pitch + sway;
            let mid = x0 + 4;
            let y0 = 3;
            for y in sag(mid as f32)..y0 {
                draw::dot_i(grid, mid, y);
            }
            // Fraction of this lantern alight, glowing from the bottom up.
            let f = (lit - i as f32).clamp(0.0, 1.0);
            let glow_y = y0 + 12 - (f * 12.0).ceil() as i32;
            stamp(
                grid,
                &LANTERN_DARK,
                (x0, y0),
                1,
                false,
                (i32::MAX, i32::MIN),
            );
            stamp(grid, &LANTERN_LIT, (x0, y0), 1, false, (i32::MAX, glow_y));

            // Lit cells are red with gold cap / tassel rows; the rest stay dim.
            let flicker = 0.12 * (ctx.time * 7.0 + i as f32 * 1.7).sin().max(0.0);
            let c0 = (x0.max(0) / 2) as usize;
            let c1 = ((x0 + 8).max(0) / 2) as usize;
            for cy_c in 0..ch {
                let lit_cell = f > 0.0 && (cy_c as i32 * 4 + 3) >= glow_y;
                let col = if !lit_cell {
                    dim
                } else if cy_c == 0 || cy_c * 4 >= (y0 + 8) as usize {
                    gold
                } else {
                    mix(red, gold, flicker)
                };
                draw::tint_row(grid, cy_c, c0, c1.min(cw.saturating_sub(1)), col);
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
    let styles = progress::styles::cultures::styles();
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
