//! `floweroflife` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O floweroflife.rs && ./floweroflife [style-name]
//! ```

const DEFAULT_STYLE: &str = "vesica-piscis";

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
    pub mod floweroflife {
//! Flower-of-Life sacred geometry progress styles.
//!
//! Eleven structurally distinct styles drawn from the Flower-of-Life family:
//! overlapping-circle constructions on a hexagonal lattice, their intersection
//! graphs (Metatron's Cube), the Kabbalistic Tree of Life, a 64-tetrahedron
//! triangular projection, and a time-rotating torus-flower. Figures fill the
//! strip's height, most repeated as a frieze filled panel by panel. Every style
//! reveals itself as `ctx.eased` rises from 0 → 1 and breathes or rotates
//! via `ctx.time`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── shared geometry helpers ────────────────────────────────────────────────

/// Plot a circle (parametric) in dot-space.
/// `cx,cy` are the centre in dots (f32); `r` is the radius in dots.
/// Steps are bounded to avoid overdraw on tiny grids.
fn plot_circle(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    if r < 0.5 {
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
        return;
    }
    // Circumference ≈ 2πr; sample at ≥2 dots per step so we don't miss any dot.
    let steps = ((2.0 * PI * r).ceil() as usize * 2).clamp(8, 2048);
    for i in 0..steps {
        let angle = 2.0 * PI * i as f32 / steps as f32;
        let px = cx + r * angle.cos();
        let py = cy + r * angle.sin();
        draw::dot_i(grid, px.round() as i32, py.round() as i32);
    }
}

/// Draw a line between two dot-space points using Bresenham-style stepping.
fn plot_line(grid: &mut BrailleGrid, x0: f32, y0: f32, x1: f32, y1: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()).ceil() as usize).clamp(1, 4096);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let px = x0 + dx * t;
        let py = y0 + dy * t;
        draw::dot_i(grid, px.round() as i32, py.round() as i32);
    }
}

/// Hex-lattice unit vectors (6 directions, 60° apart).
fn hex_offset(ring_angle_index: usize, radius: f32) -> (f32, f32) {
    let a = ring_angle_index as f32 * PI / 3.0;
    (radius * a.cos(), radius * a.sin())
}

/// 13 Fruit-of-Life centres: centre + 6 at r + 6 at 2r (same hex axes).
fn fruit_centres(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    let mut v: Vec<(f32, f32)> = Vec::with_capacity(13);
    v.push((cx, cy));
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, r);
        v.push((cx + dx, cy + dy));
    }
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, 2.0 * r);
        v.push((cx + dx, cy + dy));
    }
    v
}

/// Lays a frieze of full-height panels across the strip, filled left to right
/// with progress (the first already part-drawn at the start), each tinted
/// with its own palette stop. The callback gets `(grid, cx, cy, r,
/// local_reveal, index)`; `local_reveal` is 0 for a panel not yet reached.
/// `wf` is the panel pitch in radii, `gap` extra dots between panels.
fn frieze(
    grid: &mut BrailleGrid,
    ctx: &BarContext,
    wf: f32,
    gap: f32,
    mut panel: impl FnMut(&mut BrailleGrid, f32, f32, f32, f32, usize),
) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw < 2 || dh < 2 {
        return;
    }
    let r = (dw.min(dh) as f32 / 2.0 - 0.5).max(1.0);
    let pitch = (r * wf + gap).max(2.0);
    let n = ((dw as f32 / pitch) as usize).clamp(1, 64);
    let off = (dw as f32 - n as f32 * pitch) / 2.0;
    let cy = dh as f32 / 2.0 - 0.5;
    let fill = (0.08 + ctx.eased.clamp(0.0, 1.0) * 0.92) * n as f32;
    let (cw, ch) = grid.dimensions();
    for i in 0..n {
        let cx = off + pitch * (i as f32 + 0.5) - 0.5;
        let lr = (fill - i as f32).clamp(0.0, 1.0);
        panel(grid, cx, cy, r, lr, i);
        let color = ctx.palette.sample(i as f32 / (n - 1).max(1) as f32);
        let x0 = ((cx - pitch / 2.0).max(0.0) / 2.0) as usize;
        let x1 = (((cx + pitch / 2.0).max(0.0) / 2.0) as usize).min(cw.saturating_sub(1));
        if cw > 0 && x0 <= x1 {
            for y in 0..ch {
                draw::tint_row(grid, y, x0, x1, color);
            }
        }
    }
}

/// Faint dotted ring: the placeholder for a frieze panel not yet reached.
fn dotted_ring(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let n = ((r * 2.0) as usize).clamp(4, 64);
    for k in 0..n {
        let a = 2.0 * PI * k as f32 / n as f32;
        draw::dot_i(
            grid,
            (cx + r * a.cos()).round() as i32,
            (cy + r * a.sin()).round() as i32,
        );
    }
}

/// Plot only the part of circle (`ox`,`oy`,`r`) lying within distance `lim`
/// of (`cx`,`cy`).
fn clipped_circle(grid: &mut BrailleGrid, ox: f32, oy: f32, r: f32, cx: f32, cy: f32, lim: f32) {
    let steps = ((2.0 * PI * r).ceil() as usize * 2).clamp(8, 512);
    for i in 0..steps {
        let a = 2.0 * PI * i as f32 / steps as f32;
        let (px, py) = (ox + r * a.cos(), oy + r * a.sin());
        if (px - cx).hypot(py - cy) <= lim {
            draw::dot_i(grid, px.round() as i32, py.round() as i32);
        }
    }
}

// ── All styles ─────────────────────────────────────────────────────────────

/// All styles in the `floweroflife` theme.
///
/// Returns eleven structurally distinct sacred-geometry progress styles,
/// each revealing itself as `eased` rises and animating via `time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(VesicaPiscis),
        Box::new(SeedOfLife),
        Box::new(FlowerOfLife),
        Box::new(EggOfLife),
        Box::new(FruitOfLife),
        Box::new(MetatronsCube),
        Box::new(GermOfLife),
        Box::new(TripodOfLife),
        Box::new(TreeOfLife),
        Box::new(TetraGrid),
        Box::new(TorusFlower),
    ]
}

// ── 1. Vesica Piscis ───────────────────────────────────────────────────────

/// Two overlapping circles; the shared lens (vesica) forms as `eased` rises.
struct VesicaPiscis;
impl ProgressStyle for VesicaPiscis {
    fn name(&self) -> &str {
        "vesica-piscis"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Two circles overlap to reveal the sacred lens; arc segments appear with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) * 0.45).max(1.0);

        // Separation between the two circle centres, grows with eased.
        // At eased=0: coincident. At eased=1: separated by r (classic vesica).
        let sep = r * ctx.eased;
        let lx = cx - sep / 2.0;
        let rx = cx + sep / 2.0;

        // Draw the two circles, breathing via time.
        let breathe = 1.0 + 0.04 * (ctx.time * 1.2).sin();
        let rr = r * breathe;
        plot_circle(grid, lx, cy, rr);
        if ctx.eased > 0.05 {
            plot_circle(grid, rx, cy, rr);
        }

        // Color: tint the left half one shade, right half another.
        let (cw, ch) = grid.dimensions();
        let mid_cell = cw / 2;
        for cy_cell in 0..ch {
            if mid_cell > 0 {
                draw::tint_row(
                    grid,
                    cy_cell,
                    0,
                    mid_cell.saturating_sub(1),
                    ctx.palette.start,
                );
                draw::tint_row(
                    grid,
                    cy_cell,
                    mid_cell,
                    cw.saturating_sub(1),
                    ctx.palette.end,
                );
            }
        }
        Ok(())
    }
}

// ── 2. Seed of Life ────────────────────────────────────────────────────────

/// Seed of Life: seven equal circles, six around one, each through the centre.
struct SeedOfLife;
impl ProgressStyle for SeedOfLife {
    fn name(&self) -> &str {
        "seed-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Seed of Life: a row of seeds, each seven equal circles — one centre \
         and six around it, every one passing through the centre — set circle \
         by circle and seed by seed as progress rises, slowly turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Outer circles reach 2·rho: the seed fills the strip's height.
            let rho = (r / 2.0).max(0.8);
            let rot = time * 0.25 * if i % 2 == 0 { 1.0 } else { -1.0 };
            let n = (lr * 7.0).ceil() as usize;
            for k in 0..n.min(7) {
                if k == 0 {
                    plot_circle(g, cx, cy, rho);
                } else {
                    let a = rot + (k - 1) as f32 * PI / 3.0;
                    plot_circle(g, cx + rho * a.cos(), cy + rho * a.sin(), rho);
                }
            }
        });
        Ok(())
    }
}

// ── 3. Flower of Life ─────────────────────────────────────────────────────

/// The Flower of Life as its true, unbounded form: equal circles on a
/// hexagonal lattice, each passing through its six neighbours' centres.  At
/// 16 dots tall the 19-circle medallion shrinks to a smudge, so the lattice
/// runs as a frieze across the strip instead — circles bloom left → right
/// with progress (the newest column growing from its centre) while the
/// whole weave drifts slowly on `time`.
struct FlowerOfLife;
impl ProgressStyle for FlowerOfLife {
    fn name(&self) -> &str {
        "flower-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Flower of Life lattice: equal circles on a hexagonal grid, each through \
         its neighbours' centres, blooming left to right as a drifting frieze"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cy = dh as f32 / 2.0 - 0.5;
        // Circle radius = lattice spacing; just over half the height keeps
        // the petals open enough to read.
        let r = (dh as f32 * 0.56).max(1.5);
        let row_h = r * 3f32.sqrt() / 2.0;
        let drift = (ctx.time * 1.5).rem_euclid(r);
        let front = ctx.eased.clamp(0.0, 1.0) * (dw as f32 + r);

        let rows = (cy / row_h).ceil() as i32 + 1;
        let cols = (dw as f32 / r).ceil() as i32 + 2;
        for j in -rows..=rows {
            let py = cy + j as f32 * row_h;
            let shift = if j.rem_euclid(2) == 1 { r / 2.0 } else { 0.0 };
            for i in -1..cols {
                let px = i as f32 * r + shift + drift - r;
                if px > front {
                    continue;
                }
                // Circles just behind the front are still opening.
                let grow = ((front - px) / r).clamp(0.0, 1.0);
                plot_circle(grid, px, py, r * grow);
            }
        }

        // Palette across the bloomed region, the leading cells brightest.
        let (cw, ch) = grid.dimensions();
        let lit = ((front / 2.0).ceil() as usize).min(cw);
        for cx_cell in 0..lit {
            let t = cx_cell as f32 / cw.max(1) as f32;
            let color = if cx_cell + 2 >= lit {
                ctx.palette.end
            } else {
                ctx.palette.sample(t * 0.8)
            };
            for cy_cell in 0..ch {
                draw::tint_row(grid, cy_cell, cx_cell, cx_cell, color);
            }
        }
        Ok(())
    }
}

// ── 4. Egg of Life ────────────────────────────────────────────────────────

/// Egg of Life: seven touching spheres (one centre, six around) swaying in 3-D.
struct EggOfLife;
impl ProgressStyle for EggOfLife {
    fn name(&self) -> &str {
        "egg-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Egg of Life: a row of seed clusters, each seven touching spheres — \
         one centre, six around, highlighted and swaying in 3-D — set sphere \
         by sphere and cluster by cluster as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            let rho = (r / 3.0).max(0.8);
            let d = 2.0 * rho;
            let sway = 0.45 * (time * 1.2 + i as f32).sin();
            let n = (lr * 7.0).ceil() as usize;
            for k in 0..n.min(7) {
                let (ox, oy, z) = if k == 0 {
                    (0.0, 0.0, 0.0)
                } else {
                    let a = (k - 1) as f32 * PI / 3.0 + PI / 6.0;
                    let (x, y) = (d * a.cos(), d * a.sin());
                    (x * sway.cos(), y, x * sway.sin())
                };
                let cr = rho * (1.0 + 0.15 * z / d);
                let (px, py) = (cx + ox, cy + oy);
                plot_circle(g, px, py, cr);
                // Specular glint: each circle reads as a sphere.
                let gx = (px - cr * 0.4).round() as i32;
                let gy = (py - cr * 0.4).round() as i32;
                draw::dot_i(g, gx, gy);
            }
        });
        Ok(())
    }
}

// ── 5. Fruit of Life ──────────────────────────────────────────────────────

/// Fruit of Life: 13 disjoint circles on the hexagonal Flower-of-Life axes.
struct FruitOfLife;
impl ProgressStyle for FruitOfLife {
    fn name(&self) -> &str {
        "fruit-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Fruit of Life: a row of thirteen-circle fruits — a centre, six \
         around and six beyond along the hexagonal axes, never touching — \
         each set ring by ring as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.3, 3.0, |g, cx, cy, r, lr, _| {
            let rho = (r / 4.46).max(0.6);
            let centres = fruit_centres(cx, cy, 2.0 * rho);
            if lr <= 0.0 {
                for &(px, py) in &centres {
                    draw::dot_i(g, px.round() as i32, py.round() as i32);
                }
                return;
            }
            let n = (lr * centres.len() as f32).ceil() as usize;
            let pulse = 1.0 + 0.08 * (time * 2.0).sin();
            for &(px, py) in centres.iter().take(n) {
                plot_circle(g, px, py, rho * pulse);
            }
        });
        Ok(())
    }
}

// ── 6. Metatron's Cube ────────────────────────────────────────────────────

/// Metatron's Cube: the 13 Fruit-of-Life nodes joined by their key lines.
struct MetatronsCube;
impl ProgressStyle for MetatronsCube {
    fn name(&self) -> &str {
        "metatrons-cube"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Metatron's Cube: a row of cubes, each the 13 Fruit-of-Life node \
         circles joined into outer hexagon and great hexagram, drawn \
         family by family and cube by cube as progress rises, slowly turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // At 16 dots the full 78-line web is a solid blot, so the two
        // families that carry the figure are drawn in turn — outer hexagon,
        // then the great hexagram — over the 13 node circles.
        let mut lines: Vec<(usize, usize)> = Vec::with_capacity(12);
        for i in 0..6 {
            lines.push((7 + i, 7 + (i + 1) % 6));
        }
        for i in 0..6 {
            lines.push((7 + i, 7 + (i + 2) % 6));
        }
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Outer ring of nodes sits at 2·s: the cube fills the height.
            let nr = 1.0f32;
            let s = ((r - nr) / 2.0).max(0.8);
            let rot = time * 0.15 + i as f32 * PI / 6.0;
            let nodes: Vec<(f32, f32)> = fruit_centres(cx, cy, s)
                .iter()
                .map(|&(px, py)| {
                    let (ox, oy) = (px - cx, py - cy);
                    (
                        ox * rot.cos() - oy * rot.sin() + cx,
                        ox * rot.sin() + oy * rot.cos() + cy,
                    )
                })
                .collect();
            let n = (lr * 1.25).min(1.0) * lines.len() as f32;
            for &(a, b) in lines.iter().take(n.ceil() as usize) {
                let (ax, ay) = nodes[a];
                let (bx, by) = nodes[b];
                plot_line(g, ax, ay, bx, by);
            }
            for &(px, py) in &nodes {
                plot_circle(g, px, py, nr);
            }
        });
        Ok(())
    }
}

// ── 7. Germ of Life ───────────────────────────────────────────────────────

/// First 7 circles of the Flower of Life, inner petals traced bold.
struct GermOfLife;
impl ProgressStyle for GermOfLife {
    fn name(&self) -> &str {
        "germ-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Germ of Life: a row of seven-circle germs — the six-petal flower cut \
         cut by six circles centred on a ring, a seed at its heart and the \
         circle centres marked — petal by petal as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Six circles centred on a ring, each through the centre: only
            // their arcs inside the ring are traced, cutting the six-petal
            // germ around a small seed circle.
            let rho = r;
            let rot = time * 0.2 * if i % 2 == 0 { 1.0 } else { -1.0 };
            let n = (lr * 6.0).ceil() as usize;
            for k in 0..n.min(6) {
                let a = rot + k as f32 * PI / 3.0;
                let (ox, oy) = (cx + rho * a.cos(), cy + rho * a.sin());
                clipped_circle(g, ox, oy, rho, cx, cy, rho - 0.4);
                draw::dot_i(g, ox.round() as i32, oy.round() as i32);
            }
            if lr >= 1.0 {
                plot_circle(g, cx, cy, (r * 0.2).max(1.0));
            }
        });
        Ok(())
    }
}

// ── 8. Tripod of Life ─────────────────────────────────────────────────────

/// Tripod of Life: three mutually overlapping circles, a 3-arm spoke between.
struct TripodOfLife;
impl ProgressStyle for TripodOfLife {
    fn name(&self) -> &str {
        "tripod-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Tripod of Life: a row of three mutually overlapping circles turning \
         about their shared centre, a three-arm spoke joining their centres — \
         circle by circle and tripod by tripod as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            let rho = r * 0.6;
            let d = r - rho;
            let rot = time * 0.3 + i as f32 * PI / 3.0;
            let n = ((lr * 1.5).min(1.0) * 3.0).ceil() as usize;
            for k in 0..3 {
                let a = rot + k as f32 * 2.0 * PI / 3.0 - PI / 2.0;
                let (ox, oy) = (cx + d * a.cos(), cy + d * a.sin());
                if k < n {
                    plot_circle(g, ox, oy, rho);
                }
                if lr > 0.66 {
                    plot_line(g, cx, cy, ox, oy);
                }
            }
            draw::dot_i(g, cx.round() as i32, cy.round() as i32);
        });
        Ok(())
    }
}

// ── 9. Tree of Life ───────────────────────────────────────────────────────

/// Kabbalistic Tree of Life: 10 sephirot nodes + 22 connecting paths.
/// Paths drawn progressively with eased; nodes always visible.
struct TreeOfLife;
impl ProgressStyle for TreeOfLife {
    fn name(&self) -> &str {
        "tree-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Kabbalistic Tree of Life laid on its side: 10 sephirot on three pillars, the \
         22 paths drawn crown to kingdom with progress while the lightning flash \
         runs the sephirot in order"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;

        // The tree lies on its side to use the wide strip: its 6-unit trunk
        // runs left (Kether) to right (Malkuth), the pillars stacked ±1 unit.
        let sx = ((dw as f32 - 6.0) / 6.0).max(1.0);
        let sy = (dh as f32 / 2.0 - 2.0).max(1.0);

        // Sephirot positions (normalized, y increasing downward).
        // Classical layout:  1=Kether(top), 10=Malkuth(bottom).
        let nodes: [(f32, f32); 10] = [
            (0.0, -3.0),  // 0 Kether
            (-1.0, -2.0), // 1 Chokmah
            (1.0, -2.0),  // 2 Binah
            (-1.0, -1.0), // 3 Chesed
            (1.0, -1.0),  // 4 Geburah
            (0.0, 0.0),   // 5 Tiphareth
            (-1.0, 1.0),  // 6 Netzach
            (1.0, 1.0),   // 7 Hod
            (0.0, 2.0),   // 8 Yesod
            (0.0, 3.0),   // 9 Malkuth
        ];

        // Convert to dot-space.
        let dot_nodes: Vec<(f32, f32)> = nodes
            .iter()
            .map(|&(x, y)| (cx + y * sx, cy - 0.5 + x * sy))
            .collect();

        // 22 paths of the Tree of Life (traditional Sefer Yetzirah connections).
        let paths: [(usize, usize); 22] = [
            (0, 1),
            (0, 2),
            (0, 5), // Kether connections
            (1, 2),
            (1, 3),
            (1, 5), // Chokmah
            (2, 4),
            (2, 5), // Binah
            (3, 4),
            (3, 5),
            (3, 6), // Chesed
            (4, 5),
            (4, 7), // Geburah
            (5, 6),
            (5, 7),
            (5, 8), // Tiphareth
            (6, 7),
            (6, 8), // Netzach
            (7, 8), // Hod
            (8, 9), // Yesod-Malkuth
            (6, 9),
            (7, 9), // Netzach / Hod to Malkuth
        ];

        let reveal = (ctx.eased * paths.len() as f32).round() as usize;
        for (i, &(a, b)) in paths.iter().enumerate().take(reveal) {
            let (ax, ay) = dot_nodes[a];
            let (bx, by) = dot_nodes[b];
            plot_line(grid, ax, ay, bx, by);
            let t = i as f32 / (paths.len() - 1).max(1) as f32;
            let _ = t;
        }

        // Draw sephirot nodes as small circles.
        let nr = (sy * 0.25).clamp(1.0, 1.5);
        for (i, &(px, py)) in dot_nodes.iter().enumerate() {
            plot_circle(grid, px, py, nr);
            let t = i as f32 / 9.0;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (px / 2.0).round() as usize;
            let cell_y = (py / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }

        // The lightning flash: a spark running the sephirot 1 → 10 in order.
        let run = (ctx.time * 1.5).rem_euclid(9.0);
        let k = (run as usize).min(8);
        let f = run - k as f32;
        let (ax, ay) = dot_nodes[k];
        let (bx, by) = dot_nodes[k + 1];
        let (fx, fy) = (ax + (bx - ax) * f, ay + (by - ay) * f);
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            draw::dot_i(grid, fx.round() as i32 + dx, fy.round() as i32 + dy);
        }
        Ok(())
    }
}

// ── 10. 64-Tetrahedron Grid ───────────────────────────────────────────────

/// 64-tetrahedron grid seen down its axis: star tetrahedra in hexagonal tiles.
struct TetraGrid;
impl ProgressStyle for TetraGrid {
    fn name(&self) -> &str {
        "64-tetra-grid"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "64-tetrahedron grid: the lattice seen down its axis as a row of \
         edge-sharing hexagons, each filled with a star tetrahedron (two \
         interlocking triangles), its axes and core — tile by tile with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 3f32.sqrt(), 0.0, |g, cx, cy, r, lr, _| {
            let v: Vec<(f32, f32)> = (0..6)
                .map(|k| {
                    let a = -PI / 2.0 + k as f32 * PI / 3.0;
                    (cx + r * a.cos(), cy + r * a.sin())
                })
                .collect();
            for k in 0..6 {
                let (a, b) = (v[k], v[(k + 1) % 6]);
                plot_line(g, a.0, a.1, b.0, b.1);
            }
            if lr <= 0.0 {
                return;
            }
            // Upward tetrahedron, then axes, then the downward one.
            for k in [0usize, 2, 4] {
                let (a, b) = (v[k], v[(k + 2) % 6]);
                plot_line(g, a.0, a.1, b.0, b.1);
            }
            if lr > 0.33 {
                for k in 0..3 {
                    let (a, b) = (v[k], v[k + 3]);
                    plot_line(g, a.0, a.1, b.0, b.1);
                }
            }
            if lr > 0.66 {
                for k in [1usize, 3, 5] {
                    let (a, b) = (v[k], v[(k + 2) % 6]);
                    plot_line(g, a.0, a.1, b.0, b.1);
                }
            }
            if lr >= 1.0 && (time * 3.0).sin() > -0.6 {
                plot_circle(g, cx, cy, (r * 0.2).max(1.0));
            }
        });
        Ok(())
    }
}

// ── 11. Torus Flower ──────────────────────────────────────────────────────

/// Torus flower: circles rotated about a centre, sweeping out a torus.
struct TorusFlower;
impl ProgressStyle for TorusFlower {
    fn name(&self) -> &str {
        "torus-flower"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Torus flower: a row of tori, each eight tube circles rotated about \
         an open centre into a ring, turning with time — \
         circle by circle and torus by torus as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Tube circles ringed round an open hole: a donut of circles.
            let rho = r * 0.36;
            let d = r - rho;
            let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rot = dir * time * 0.35;
            let n = (lr * 8.0).ceil() as usize;
            for k in 0..n.min(8) {
                let a = rot + k as f32 * PI / 4.0;
                plot_circle(g, cx + d * a.cos(), cy + d * a.sin(), rho);
            }
        });
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
    let styles = progress::styles::floweroflife::styles();
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
