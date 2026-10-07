//! `gadgets` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O gadgets.rs && ./gadgets [style-name]
//! ```

const DEFAULT_STYLE: &str = "phone-battery";

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
    pub mod gadgets {
//! Gadgets / consumer-tech progress bars — device UIs in braille.
//!
//! Every style mimics a familiar consumer device interaction: phone charging,
//! WiFi connecting, Bluetooth pairing, disk defragging, CRT power-on, dial-up
//! modem handshake, a vinyl record spinning up, a quadcopter, an activity ring,
//! USB file transfer, a gear train, and an e-ink page refresh. All are
//! stateless — animation is driven entirely by `ctx.time` and `ctx.eased`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─── dot-art helpers ─────────────────────────────────────────────────────────

/// Stamp a sprite: every `#` in `rows` sets the dot at `(x + col, y + row)`.
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

// ─── tiny deterministic hash ─────────────────────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

/// All styles in the `gadgets` theme.
///
/// Returns 12 structurally distinct consumer-device progress bar
/// implementations, each stateless and animatable via `ctx.time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PhoneBattery),
        Box::new(WifiConnect),
        Box::new(BluetoothPair),
        Box::new(DiskDefrag),
        Box::new(CrtPowerOn),
        Box::new(DialUpModem),
        Box::new(VinylSpinUp),
        Box::new(DroneProps),
        Box::new(ActivityRing),
        Box::new(UsbTransfer),
        Box::new(GearTrain),
        Box::new(EinkRefresh),
    ]
}

// ─── 1. Smartphone battery charging ──────────────────────────────────────────

/// Battery outline that fills with lightning bolt and rising charge bubbles.
struct PhoneBattery;
impl ProgressStyle for PhoneBattery {
    fn name(&self) -> &str {
        "phone-battery"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Smartphone battery outline fills with charge; lightning bolt pulses"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Battery body outline — leave a 2-dot border.
        // "Terminal nub" on the right: a 2-dot-wide bump.
        let nub_w = (w / 16).max(1);
        let body_w = w.saturating_sub(nub_w + 2);
        let body_h = h.saturating_sub(2).max(1);
        let bx = 0usize;
        let by = 1usize;

        draw::rect_outline(grid, bx, by, body_w.max(2), body_h.max(2));

        // Nub
        let nub_y0 = by + body_h / 4;
        let nub_h = body_h / 2;
        draw::fill_rect(grid, bx + body_w, nub_y0, nub_w, nub_h.max(1));

        // Fill level inside the battery.
        let inner_w = body_w.saturating_sub(4).max(1);
        let inner_h = body_h.saturating_sub(4).max(1);
        let filled_w = ((ctx.eased * inner_w as f32) as usize).min(inner_w);
        if filled_w > 0 {
            draw::fill_rect(grid, bx + 2, by + 2, filled_w, inner_h);
        }

        // Lightning bolt: drawn with dots in the center when filling > 0.
        // Shape: top-right diagonal, then bottom-left diagonal.
        let bolt_x = (bx + body_w / 2) as i32;
        let bolt_mid = (by + body_h / 2) as i32;
        let bolt_h = inner_h as i32;
        let blink = (ctx.time * 2.0).fract() > 0.3 || ctx.eased > 0.5;
        if blink {
            // Top half of bolt: slants right.
            for dy in 0..bolt_h / 2 {
                let y = bolt_mid - dy as i32;
                let x = bolt_x + (dy as f32 * 0.6) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
            // Bottom half: slants left.
            for dy in 0..=bolt_h / 2 {
                let y = bolt_mid + dy as i32;
                let x = bolt_x - (dy as f32 * 0.6) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
        }

        // Rising charge bubbles — dots that travel upward inside the fill.
        let n_bubbles = 4usize;
        for b in 0..n_bubbles {
            let phase = b as f32 / n_bubbles as f32;
            let t = (ctx.time * 0.5 + phase).fract();
            let bub_x = bx + 2 + (hashf(b as u32 * 7 + 3) * inner_w as f32) as usize;
            let bub_x = bub_x.min(bx + body_w.saturating_sub(3));
            let bub_y = by
                + 2
                + inner_h
                    .saturating_sub(1)
                    .saturating_sub((t * inner_h as f32) as usize);
            // Only show bubble in the filled zone.
            let fill_right = bx + 2 + filled_w;
            if bub_x < fill_right {
                draw::dot(grid, bub_x, bub_y);
                if bub_x + 1 < fill_right {
                    draw::dot(grid, bub_x + 1, bub_y);
                }
            }
        }

        // Tint filled region green→yellow by charge level.
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        for cx in 0..filled_cells {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 2. WiFi connecting ───────────────────────────────────────────────────────

/// Concentric arcs pulse outward, then lock solid once connected.
struct WifiConnect;
impl ProgressStyle for WifiConnect {
    fn name(&self) -> &str {
        "wifi-connect"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "WiFi arcs pulse outward while connecting, then lock solid at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Origin: bottom-center, like a standard WiFi symbol.
        let cx_dot = w / 2;
        let cy_dot = h.saturating_sub(1);

        // Number of arcs lit by progress (0–4).
        let n_arcs = 4usize;
        let arcs_lit = (ctx.eased * n_arcs as f32).floor() as usize;
        // Connected: all 4 arcs lit.
        let connected = arcs_lit >= n_arcs;

        // Draw the base dot (the WiFi "device" dot).
        draw::dot(grid, cx_dot, cy_dot);
        if cx_dot + 1 < w {
            draw::dot(grid, cx_dot + 1, cy_dot);
        }

        for arc in 0..n_arcs {
            let radius = (arc + 1) as f32 * (h as f32 / (n_arcs + 1) as f32);
            let lit = arc < arcs_lit;

            // Pulsing alpha for unlocked arcs: sine pulse that propagates outward.
            let pulse_phase = (ctx.time * 1.5 - arc as f32 * 0.4).fract();
            let pulse = (pulse_phase * PI * 2.0).sin() * 0.5 + 0.5;
            let draw_it = lit || (!connected && pulse > 0.5);

            if draw_it {
                // Draw a semicircular arc (top half only — like WiFi symbol).
                // Sweep 180° (from left to right, convex upward).
                let steps = ((PI * radius) as usize).max(8);
                for step in 0..=steps {
                    let angle = PI * (step as f32 / steps as f32);
                    // angle 0 = right, PI = left; we want arcs opening downward.
                    let dx = (angle.cos() * radius) as i32;
                    let dy = -(angle.sin() * radius * 0.6) as i32; // flatten vertically
                    draw::dot_i(grid, cx_dot as i32 + dx, cy_dot as i32 + dy);
                    if !lit {
                        // Dashed: only draw alternating steps for the pulsing arcs.
                        // Already drawn above; skip extra for pulsing effect.
                    }
                }

                // Color: lit arcs get palette gradient.
                if lit {
                    let t = arc as f32 / n_arcs.max(1) as f32;
                    let color = ctx.palette.sample(t);
                    let cell_cx = cx_dot / 2;
                    let r_cells = (radius as usize / 2).max(1);
                    let cx0 = cell_cx.saturating_sub(r_cells);
                    let cx1 = (cell_cx + r_cells).min(cells_w.saturating_sub(1));
                    let cy_cell = cy_dot / 4;
                    let r_cell_h = (radius as usize / 4).max(1);
                    let cy0 = cy_cell.saturating_sub(r_cell_h);
                    for cy in cy0..cells_h {
                        draw::tint_row(grid, cy, cx0, cx1, color);
                    }
                }
            }
        }

        // "Locked" indicator: horizontal bar at bottom when connected.
        if connected {
            let blink_off = (ctx.time * 3.0).fract() < 0.2;
            if !blink_off {
                draw::hline(
                    grid,
                    cx_dot.saturating_sub(2),
                    (cx_dot + 2).min(w - 1),
                    cy_dot,
                );
                let full_color = ctx.palette.sample(1.0);
                draw::tint_row(
                    grid,
                    cells_h.saturating_sub(1),
                    (cells_w / 2).saturating_sub(1),
                    (cells_w / 2 + 1).min(cells_w - 1),
                    full_color,
                );
            }
        }

        Ok(())
    }
}

// ─── 3. Bluetooth pairing ─────────────────────────────────────────────────────

/// The ᛒ-like Bluetooth glyph with handshake blips radiating outward.
struct BluetoothPair;
impl ProgressStyle for BluetoothPair {
    fn name(&self) -> &str {
        "bluetooth-pair"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Bluetooth rune glyph at center; handshake blips pulse outward while pairing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let cx = w / 2;
        let cy = h / 2;

        // ── Bluetooth glyph in dot space (approximated) ──
        // Vertical spine.
        let spine_h = (h * 3 / 4).max(2);
        let y0 = cy.saturating_sub(spine_h / 2);
        let y1 = (cy + spine_h / 2).min(h.saturating_sub(1));
        draw::vline(grid, cx, y0, y1);

        // Upper-right arm and return (forms the top lobe of ᛒ).
        let arm = (h / 5).max(1);
        // Upper-right diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (y0 + arm) as i32 - i as i32);
        }
        // Upper-right return diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (y0 + arm) as i32 + i as i32);
        }
        // Lower-right diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, cy as i32 + i as i32);
        }
        // Lower-right return diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (cy + arm) as i32 - i as i32);
        }

        // ── Pairing signal arcs ──
        // Nested arcs flank the rune on both sides; the count grows with
        // progress (1 faint arc at the start, 5 when pairing is nearly done)
        // so completion stays legible in monochrome.
        let max_arcs = 5usize;
        let n_arcs = ((ctx.eased * max_arcs as f32).ceil() as usize).clamp(1, max_arcs);
        let arc_steps = 13usize;
        for k in 0..n_arcs {
            let r = (arm + 4 + k * 7) as f32;
            for s in 0..arc_steps {
                let a = (s as f32 / (arc_steps - 1) as f32 - 0.5) * 1.2; // ±0.6 rad
                let dx = (r * a.cos()).round() as i32;
                let dy = (r * 0.35 * a.sin()).round() as i32;
                draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
                draw::dot_i(grid, cx as i32 - dx, cy as i32 + dy);
            }
        }

        // ── Handshake blips ──
        // Blips travel horizontally outward from the glyph. The 0.75 Hz rate
        // is a multiple of 0.25 Hz so the 4-second loop stays seamless.
        let n_blips = 5usize;
        let paired = ctx.eased >= 1.0;
        for b in 0..n_blips {
            let phase = b as f32 / n_blips as f32;
            let t = if paired {
                // Frozen in place when paired.
                1.0f32
            } else {
                (ctx.time * 0.75 + phase).fract()
            };
            let reach = (t * (w / 2) as f32) as usize;

            // Left blip.
            let lx = cx.saturating_sub(reach);
            draw::dot(grid, lx, cy);

            // Right blip.
            let rx = (cx + reach).min(w.saturating_sub(1));
            draw::dot(grid, rx, cy);
        }

        // Tint: progress drives gradient, center stays brightest.
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        let mid_cell = cells_w / 2;
        let reach_cells = (filled_cells / 2).max(1);
        let cx0 = mid_cell.saturating_sub(reach_cells);
        let cx1 = (mid_cell + reach_cells).min(cells_w.saturating_sub(1));
        for cx_c in cx0..=cx1 {
            let t = (cx_c.saturating_sub(cx0)) as f32 / (cx1.saturating_sub(cx0) + 1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 4. Disk defragmenter ────────────────────────────────────────────────────

/// The classic defrag cluster map: a grid of small cluster blocks. Behind the
/// sweep the used clusters are packed solid and contiguous; ahead of it the
/// disk is still fragmented (scattered hollow clusters with gaps). A blinking
/// read/write head marks the cluster being moved.
struct DiskDefrag;
impl ProgressStyle for DiskDefrag {
    fn name(&self) -> &str {
        "disk-defrag"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Disk defrag cluster map: scattered hollow fragments ahead of the sweep, packed solid clusters behind it, with a blinking read/write head"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let cols = (w / 4).max(1);
        let rows = (h / 4).max(1);
        let total = cols * rows;
        // Column-major order so the sweep runs left to right across all rows.
        let done = (ctx.eased * total as f32) as usize;
        let head_on = (ctx.time * 5.0).fract() < 0.5;
        for i in 0..total {
            let (col, row) = (i / rows, i % rows);
            let (x, y) = (col * 4, row * 4);
            if i < done {
                // Defragmented: 70% of the swept area is packed data.
                if (i as f32) < done as f32 * 0.7 + 0.5 {
                    draw::fill_rect(grid, x, y, 3, 3);
                }
            } else if i == done {
                if head_on {
                    draw::fill_rect(grid, x, y, 3, 3);
                } else {
                    draw::rect_outline(grid, x, y, 3, 3);
                }
            } else if hashf(i as u32 + 17) < 0.6 {
                draw::rect_outline(grid, x, y, 3, 3);
            }
        }
        Ok(())
    }
}

// ─── 5. CRT TV power-on ──────────────────────────────────────────────────────

/// A bright horizontal line at center expands vertically to fill the screen.
struct CrtPowerOn;
impl ProgressStyle for CrtPowerOn {
    fn name(&self) -> &str {
        "crt-power-on"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "CRT power-on: bright center line expands vertically to fill the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = h / 2;
        // The beam starts as a single scanline and grows to full height.
        let beam_h = ((ctx.eased * h as f32) as usize).max(1).min(h);
        let y0 = mid.saturating_sub(beam_h / 2);
        let y1 = (y0 + beam_h).min(h);

        // Fill the beam area — horizontal scanlines.
        for y in y0..y1 {
            draw::hline(grid, 0, w.saturating_sub(1), y);
        }

        // Bright edge lines (phosphor glow at the expanding boundary).
        // The leading edges are extra dense (draw them twice — one line extra).
        if y0 > 0 {
            draw::hline(grid, 0, w.saturating_sub(1), y0.saturating_sub(1));
        }
        if y1 < h {
            draw::hline(grid, 0, w.saturating_sub(1), y1);
        }

        // Horizontal scanline "noise" inside the beam: a few dotted lines fade in.
        if beam_h > 4 {
            let noise_lines = beam_h / 4;
            for nl in 0..noise_lines {
                let y_nl = y0 + 2 + nl * 4;
                if y_nl >= y1 {
                    break;
                }
                // Sparse noise dots derived from hash.
                for x in 0..w {
                    let on = (hash((x as u32).wrapping_add(y_nl as u32 * 17)) % 5) == 0;
                    if on {
                        draw::dot(grid, x, y_nl);
                    }
                }
            }
        }

        // Tint: white-ish center → palette edge glow.
        let cy_center = cells_h / 2;
        let beam_cells = (beam_h / 4).max(1);
        let cy0 = cy_center.saturating_sub(beam_cells / 2);
        let cy1 = (cy0 + beam_cells).min(cells_h.saturating_sub(1));
        for cy in cy0..=cy1 {
            let t = (cy.saturating_sub(cy0)) as f32 / (cy1.saturating_sub(cy0) + 1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─── 6. Dial-up modem handshake ──────────────────────────────────────────────

/// Noisy oscillation across the bar gradually resolves to a steady carrier tone.
struct DialUpModem;
impl ProgressStyle for DialUpModem {
    fn name(&self) -> &str {
        "dialup-modem"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Dial-up modem: chaotic noise waveform settles to a clean carrier tone as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = (h / 2) as i32;
        let amp = (h / 2).saturating_sub(1) as f32;

        for x in 0..w {
            // "Clean" carrier: a smooth sine at this column.
            let carrier_phase = x as f32 / w as f32 * 4.0 * PI + ctx.time * 3.0;
            let clean_y = mid + (carrier_phase.sin() * amp * 0.8) as i32;

            // "Noisy" signal: carrier + random hash jitter.
            let epoch = (ctx.time * 8.0) as u32;
            let noise = hashf(x as u32 * 13 + epoch * 37) * 2.0 - 1.0;
            let noisy_y = mid + ((carrier_phase.sin() * 0.8 + noise * 1.5) * amp) as i32;

            // Blend: lerp from noisy (progress=0) → clean (progress=1).
            let y = (noisy_y as f32 * (1.0 - ctx.eased) + clean_y as f32 * ctx.eased) as i32;
            draw::dot_i(grid, x as i32, y);

            // Draw a second dot one above/below to fatten the trace.
            draw::dot_i(grid, x as i32, y + 1);
        }

        // Tint: chaotic region dim, settled region bright (left→right by progress).
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.max(1) as f32;
            let is_settled = cx < filled_cells;
            let color = if is_settled {
                ctx.palette.sample(t)
            } else {
                ctx.palette.sample(0.05)
            };
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 7. CD / Vinyl spinning up ────────────────────────────────────────────────

/// A round black record (grooves, paper label, spindle) spins on the platter,
/// its sheen gap and label mark turning faster as progress rises; the tonearm
/// tracks inward and a playback timeline beside it fills.
struct VinylSpinUp;
impl ProgressStyle for VinylSpinUp {
    fn name(&self) -> &str {
        "vinyl-spin"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Vinyl record: a grooved black disc with a paper label spins up on the turntable while the tonearm tracks inward and a playback timeline fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi as f32 - 1.0) / 2.0).clamp(0.5, 7.5);
        let (cx, cy) = (r + 1.0, (hi as f32 - 1.0) / 2.0);

        // The record: rim, concentric grooves broken by a turning sheen gap,
        // and a solid paper label with a spindle hole and one print mark.
        let spin = ctx.time * (1.5 + ctx.eased * 9.0);
        let (mx, my) = (cx + 1.5 * spin.cos(), cy + 1.5 * spin.sin());
        for y in 0..hi {
            for x in 0..(2.0 * r + 3.0) as i32 {
                let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                let d = (dx * dx + dy * dy).sqrt();
                let diff = (dy.atan2(dx) - spin).rem_euclid(PI);
                let gap = !(0.35..PI - 0.35).contains(&diff);
                let mark = (x as f32 - mx).abs() < 0.6 && (y as f32 - my).abs() < 0.6;
                let on = if d > r + 0.3 {
                    false
                } else if d > r - 0.7 {
                    true // rim
                } else if d < 0.7 {
                    false // spindle hole
                } else if d < 2.6 {
                    !mark // label
                } else {
                    let groove = (d - 2.6).rem_euclid(1.6) < 0.6;
                    groove && !gap
                };
                if on {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Tonearm: pivot block, arm swinging the stylus inward with progress.
        let px = (cx + r + 5.0) as i32;
        draw::fill_rect(grid, (px - 1).max(0) as usize, 0, 3, 3);
        let reach = r - 0.5 - ctx.eased * (r - 3.5).max(0.0);
        let (sx, sy) = ((cx + reach * 0.7) as i32, (cy + reach * 0.7) as i32);
        let steps = 16;
        for k in 0..=steps {
            let t = k as f32 / steps as f32;
            draw::dot_i(
                grid,
                px + ((sx - px) as f32 * t).round() as i32,
                1 + ((sy - 1) as f32 * t).round() as i32,
            );
        }

        // Playback timeline: track line, elapsed fill and a round playhead.
        let x0 = px + 5;
        let span = (wi - x0 - 3).max(0);
        let mid = hi / 2;
        for x in x0..x0 + span {
            draw::dot_i(grid, x, mid);
        }
        let head = x0 + (ctx.eased * span as f32) as i32;
        for x in x0..head {
            draw::dot_i(grid, x, mid - 1);
            draw::dot_i(grid, x, mid + 1);
        }
        for (ox, oy) in [
            (0, -2),
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (0, 2),
        ] {
            draw::dot_i(grid, head + ox, mid + oy);
        }
        Ok(())
    }
}

// ─── 8. Drone quad-propellers ────────────────────────────────────────────────

/// Top-down quadcopter: a central body with a blinking nose LED, four arms
/// and four rotor rings whose twin blades spin faster as progress rises,
/// while the drone flies left to right.
struct DroneProps;
impl ProgressStyle for DroneProps {
    fn name(&self) -> &str {
        "drone-props"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Quadcopter seen from above: body, four arms and rotor rings whose blades spin up with progress as the drone flies across"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cx = 14 + (ctx.eased * (wi - 29).max(0) as f32) as i32;
        let cy = hi / 2;
        let bob = i32::from((ctx.time * 2.0).fract() >= 0.5 && ctx.eased < 1.0);

        // Body: a rounded hull with a blinking nose LED on the leading edge.
        for (dx, dy) in (-3..=3_i32).flat_map(|x| (-2..=2_i32).map(move |y| (x, y))) {
            if dx.abs() + dy.abs() < 5 {
                draw::dot_i(grid, cx + dx, cy + dy + bob);
            }
        }
        if (ctx.time * 3.0).fract() < 0.5 {
            draw::dot_i(grid, cx + 5, cy + bob);
        }

        // Arms and rotors at the four corners.
        let spin = ctx.time * (3.0 + ctx.eased * 25.0);
        for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let (rx, ry) = (cx + sx * 10, cy + sy * 4 + bob);
            for k in 1..8 {
                draw::dot_i(grid, cx + sx * (k * 10 / 8), cy + sy * (k * 4 / 8) + bob);
            }
            // Rotor guard ring.
            for s in 0..16 {
                let a = s as f32 * PI / 8.0;
                draw::dot_i(
                    grid,
                    rx + (a.cos() * 3.5).round() as i32,
                    ry + (a.sin() * 3.5).round() as i32,
                );
            }
            // Twin blades, counter-rotating on diagonal pairs.
            let a = spin * (sx * sy) as f32;
            for k in -3..=3 {
                let t = k as f32;
                draw::dot_i(
                    grid,
                    rx + (a.cos() * t).round() as i32,
                    ry + (a.sin() * t * 0.9).round() as i32,
                );
            }
        }
        Ok(())
    }
}

// ─── 9. Smartwatch activity ring ─────────────────────────────────────────────

/// Three concentric round rings — red Move, green Exercise, blue Stand —
/// close clockwise from twelve o'clock over dim tracks, drawn as true circles
/// (braille dots are square). Matching summary bars sit to the right.
struct ActivityRing;

/// Ring colours: Move, Exercise, Stand.
const RING_COLORS: [Color; 3] = [
    Color::rgb(250, 40, 90),
    Color::rgb(150, 250, 40),
    Color::rgb(40, 220, 250),
];

impl ProgressStyle for ActivityRing {
    fn name(&self) -> &str {
        "activity-ring"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Smartwatch activity rings: concentric red, green and blue circles close clockwise as progress grows, with matching summary bars"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w == 0 || h == 0 {
            return Ok(());
        }
        let e = ctx.eased.clamp(0.0, 1.0);
        let vals = [e, e.powf(0.8), e.powf(1.3)];
        let r_out = (h.min(w) as f32 / 2.0).max(1.0);
        let (ccx, ccy) = (r_out, h as f32 / 2.0);
        let s = r_out / 8.0;
        let radii = [7.0 * s, 4.6 * s, 2.2 * s];
        let half = 0.95 * s;
        let box_w = (2.0 * r_out).ceil() as usize + 1;
        // Per-cell ring ownership: (ring, lit) with the strongest vote.
        let mut votes = vec![[0u8; 6]; cw * ch];
        for y in 0..h {
            for x in 0..box_w.min(w) {
                let dx = x as f32 + 0.5 - ccx;
                let dy = y as f32 + 0.5 - ccy;
                let d = (dx * dx + dy * dy).sqrt();
                for (k, &r) in radii.iter().enumerate() {
                    if (d - r).abs() > half {
                        continue;
                    }
                    let ang = dx.atan2(-dy).rem_euclid(2.0 * PI) / (2.0 * PI);
                    let lit = ang <= vals[k] && vals[k] > 0.0;
                    if lit || (x + y) % 2 == 0 {
                        draw::dot(grid, x, y);
                    }
                    let cell = (y / 4) * cw + x / 2;
                    if let Some(v) = votes.get_mut(cell) {
                        let slot = k * 2 + usize::from(lit);
                        v[slot] = v[slot].saturating_add(if lit { 2 } else { 1 });
                    }
                }
            }
        }
        // Summary bars to the right, one per ring.
        let bx0 = box_w + 3;
        let bar_w = w.saturating_sub(bx0 + 1);
        if bar_w > 2 {
            for (k, &v) in vals.iter().enumerate() {
                let y = (h * (2 * k + 1)) / 6;
                let fill = (v * bar_w as f32) as usize;
                for x in 0..bar_w {
                    if x < fill {
                        draw::dot(grid, bx0 + x, y);
                        if y + 1 < h {
                            draw::dot(grid, bx0 + x, y + 1);
                        }
                    } else if x % 3 == 0 {
                        draw::dot(grid, bx0 + x, y);
                    }
                }
            }
        }

        grid.enable_color_support();
        let dim = |c: Color| Color::rgb(c.r / 3, c.g / 3, c.b / 3);
        for cy in 0..ch {
            for cx in 0..cw {
                let px = cx * 2 + 1;
                let color = if px < box_w {
                    let v = votes[cy * cw + cx];
                    let (best, _) =
                        v.iter().enumerate().fold(
                            (0, 0u8),
                            |acc, (i, &n)| if n > acc.1 { (i, n) } else { acc },
                        );
                    let c = RING_COLORS[best / 2];
                    if best % 2 == 1 {
                        c
                    } else {
                        dim(c)
                    }
                } else if px >= bx0 && bar_w > 2 {
                    let k = ((cy * 4 + 2) * 3 / h.max(1)).min(2);
                    let fill = (vals[k] * bar_w as f32) as usize;
                    if px - bx0 < fill {
                        RING_COLORS[k]
                    } else {
                        dim(RING_COLORS[k])
                    }
                } else {
                    dim(RING_COLORS[0])
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 10. USB file transfer ────────────────────────────────────────────────────

/// The USB trident logo on the left points at a flash drive on the right;
/// little file packets (dog-eared pages) stream along into the drive's
/// metal plug, the drive's storage fills with progress and its LED blinks.
struct UsbTransfer;
impl ProgressStyle for UsbTransfer {
    fn name(&self) -> &str {
        "usb-transfer"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "USB transfer: file packets stream from the USB trident logo into a flash drive's plug; the drive fills with progress, LED blinking"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;
        let my = hi / 2;

        // USB trident logo: base circle, shaft, arrowhead, two branches.
        sprite(grid, 0, my - 2, &[".##.", "#..#", "#..#", ".##."]);
        for x in 4..20 {
            draw::dot_i(grid, x, my);
        }
        sprite(grid, 20, my - 2, &["#..", "##.", "###", "##.", "#.."]);
        for k in 0..4 {
            draw::dot_i(grid, 7 + k, my - 1 - k);
            draw::dot_i(grid, 10 + k, my + 1 + k);
        }
        for x in 11..14 {
            draw::dot_i(grid, x, my - 4);
        }
        sprite(grid, 14, my - 5, &[".##.", "#..#", ".##."]);
        for x in 14..16 {
            draw::dot_i(grid, x, my + 4);
        }
        sprite(grid, 16, my + 3, &["###", "###", "###"]);

        // Flash drive: plug (with two contact holes) + body.
        let body_w = 24.min(wi / 3).max(6);
        let bx = wi - body_w;
        let px = bx - 9;
        sprite(
            grid,
            px,
            my - 4,
            &[
                "#########",
                "#........",
                "#.##..##.",
                "#.##..##.",
                "#........",
                "#........",
                "#........",
                "#########",
            ],
        );
        draw::rect_outline(
            grid,
            bx.max(0) as usize,
            (my - 7).max(0) as usize,
            body_w.max(1) as usize - 1,
            (hi - 2).max(1) as usize,
        );
        // Stored data fills the body.
        let inner = (body_w - 6).max(0);
        let fill = (ctx.eased * inner as f32).round() as i32;
        for x in 0..fill {
            for y in my - 4..=my + 4 {
                if (x + y) % 2 == 0 || x % 3 == 0 {
                    draw::dot_i(grid, bx + 2 + x, y);
                }
            }
        }
        // Activity LED near the end.
        if (t * 6.0).fract() < 0.5 {
            block(grid, wi - 3, my - 1, 2, 2);
        }

        // File packets streaming from the arrowhead into the plug.
        let (x0, x1) = (25, px - 1);
        let span = (x1 - x0).max(1);
        for i in 0..3 {
            let s = (t * 0.9 + i as f32 / 3.0).fract();
            let x = x0 + (s * span as f32) as i32;
            if x + 4 > x1 {
                continue;
            }
            sprite(grid, x, my - 3, &["##.", "###", "###", "###"]);
        }
        for x in (x0..x1).step_by(3) {
            draw::dot_i(grid, x, my + 3);
        }

        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let qx = cx as i32 * 2 + 1;
                let color = if qx < 24 {
                    Color::rgb(220, 226, 236) // logo
                } else if qx < px - 1 {
                    Color::rgb(110, 250, 150) // packets
                } else if qx < bx {
                    Color::rgb(196, 200, 210) // metal plug
                } else if qx >= wi - 4 {
                    Color::rgb(80, 255, 120) // LED
                } else {
                    Color::rgb(60, 120, 250)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 11. Gear train ──────────────────────────────────────────────────────────

/// A row of round toothed gears, alternating large and small, each meshed with
/// its neighbour. Progress engages the train gear by gear: engaged gears turn
/// (neighbours counter-rotating, small ones faster), idle gears sit still.
struct GearTrain;
impl ProgressStyle for GearTrain {
    fn name(&self) -> &str {
        "gear-train"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Gear train: a row of meshed toothed gears, large and small; progress engages them one by one (idle gears ghosted) and engaged gears counter-rotate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let cy = (h as f32 - 1.0) / 2.0;
        let big = (cy - 1.2).clamp(1.0, 6.3);
        let small = (big * 0.6).max(1.0);
        let pitch = big + small + 1.6;
        let n = (((w as f32 - 2.0 * big - 2.0) / pitch) as usize + 1).max(1);
        let engaged = (ctx.eased * n as f32).ceil() as usize;

        for i in 0..n {
            let r = if i % 2 == 0 { big } else { small };
            let teeth = if i % 2 == 0 { 10.0 } else { 6.0 };
            let cx = big + 1.0 + i as f32 * pitch;
            // Meshed gears share surface speed: ω ∝ 1/r, alternating sign.
            let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rot = if i < engaged {
                ctx.time * 3.0 * dir / r
            } else {
                0.0
            };
            // Idle gears are ghosted (every third dot) so engagement reads.
            let samples = (r * 14.0) as usize;
            let stride = if i < engaged { 1 } else { 3 };
            for s in (0..samples).step_by(stride) {
                let a = s as f32 / samples as f32 * 2.0 * PI;
                let tooth = ((a - rot) * teeth / (2.0 * PI)).rem_euclid(1.0) < 0.5;
                let rr = if tooth { r + 1.2 } else { r };
                draw::dot_i(
                    grid,
                    (cx + a.cos() * rr).round() as i32,
                    (cy + a.sin() * rr).round() as i32,
                );
            }
            // Hub with a spoke mark so rotation reads.
            draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
            let (sx, sy) = (cx + rot.cos() * r * 0.55, cy + rot.sin() * r * 0.55);
            draw::dot_i(grid, sx.round() as i32, sy.round() as i32);
        }
        Ok(())
    }
}

// ─── 12. E-ink page refresh ───────────────────────────────────────────────────

/// An e-reader: bezel with page-turn buttons around a paper-white screen of
/// justified text lines. Every page turn the whole screen flashes black, then
/// blank, then the new page's text draws in. A location bar at the bottom of
/// the screen fills with progress.
struct EinkRefresh;
impl ProgressStyle for EinkRefresh {
    fn name(&self) -> &str {
        "eink-refresh"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "E-reader page turn: the screen flashes full black, then white, then a new page of text lines appears; the location bar fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        // Bezel, with page-turn buttons on the right.
        draw::rect_outline(grid, 0, 0, w, h);
        let sx1 = (wi - 6).max(2); // screen right edge (exclusive)
        for y in 1..hi - 1 {
            draw::dot_i(grid, sx1, y);
        }
        for y in [hi / 2 - 4, hi / 2 + 1] {
            for k in 0..3 {
                draw::dot_i(grid, sx1 + 2, y + k);
                draw::dot_i(grid, sx1 + 3, y + k);
            }
        }

        let period = 1.6;
        let page = (ctx.time / period) as u32;
        let u = (ctx.time / period).fract();
        let (x0, x1, y0, y1) = (2, sx1 - 1, 2, hi - 3);
        if u < 0.08 {
            // Full black flash.
            for y in 1..hi - 1 {
                for x in 1..sx1 {
                    draw::dot_i(grid, x, y);
                }
            }
        } else if u >= 0.16 {
            // Text lines revealed top to bottom.
            let reveal = ((u - 0.16) / 0.2).min(1.0);
            let rows = ((y1 - y0) / 2).max(0);
            let shown = (reveal * rows as f32).ceil() as i32;
            for r in 0..shown {
                let y = y0 + r * 2;
                if y >= y1 - 1 {
                    break;
                }
                // Words of varying length; last line of a paragraph short.
                let end = if r % 4 == 3 { x0 + (x1 - x0) / 2 } else { x1 };
                let mut x = x0 + if r % 4 == 0 { 3 } else { 0 };
                let mut k = 0u32;
                while x < end {
                    let len = 2 + (hash(page * 977 + r as u32 * 31 + k) % 6) as i32;
                    for d in 0..len.min(end - x) {
                        draw::dot_i(grid, x + d, y);
                    }
                    x += len + 2;
                    k += 1;
                }
            }
        }
        // Location bar along the bottom of the screen.
        let fill = x0 + (ctx.eased * (x1 - x0) as f32) as i32;
        for x in x0..x1 {
            if x <= fill || x % 2 == 0 {
                draw::dot_i(grid, x, hi - 2);
            }
        }

        grid.enable_color_support();
        let flash = u < 0.08;
        for cy in 0..ch {
            for cx in 0..cw {
                let qx = cx as i32 * 2 + 1;
                let color = if qx >= sx1 {
                    Color::rgb(130, 132, 140) // bezel
                } else if flash {
                    Color::rgb(70, 70, 74)
                } else {
                    Color::rgb(232, 228, 214) // paper white
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
    let styles = progress::styles::gadgets::styles();
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
