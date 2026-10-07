//! `surf` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O surf.rs && ./surf [style-name]
//! ```

const DEFAULT_STYLE: &str = "barrel-tube";

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
    pub mod surf {
//! Surf / surfing-culture progress bars.
//!
//! Twelve animated braille bars built around the surfing world: barrels,
//! swell lines marching from the horizon, surfer carves, wipeouts, aerials,
//! cutbacks, noserides, offshore spray, wave sets, tide lines advancing up
//! the beach, a palm tree at sunset and a surf van on the coast road. Every bar is
//! structurally distinct — different geometry, layout, and animation logic,
//! not merely a recolor of another.
//!
//! `ctx.eased` = how far along the ride / how much of the wave is done.
//! `ctx.time`  = continuous wave-cycle driver (sine + scroll).

use super::super::draw;
use super::super::{lerp, BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `surf` theme.
///
/// Returns twelve boxed surf-themed bars ready to be mixed into any registry
/// or rendered directly via [`ProgressStyle::render`].
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(BarrelTube),
        Box::new(SwellLines),
        Box::new(SurferCarve),
        Box::new(Wipeout),
        Box::new(Cutback),
        Box::new(OffshoreSpray),
        Box::new(NoseRide),
        Box::new(AerialLaunch),
        Box::new(TideLine),
        Box::new(PalmSunset),
        Box::new(WaveSets),
        Box::new(SurfVan),
    ]
}

// ---------------------------------------------------------------------------
// 1. Barrel / Tube
// ---------------------------------------------------------------------------
// Side view of a hollow wave: a thick lip pitches over from the back wall and
// curls down toward the water, leaving an open tube with a crouched surfer
// inside. `eased` slides the barrel to the right; behind it the wave has
// already broken into churning whitewater. `time` churns the foam and wobbles
// the lip.

struct BarrelTube;
impl ProgressStyle for BarrelTube {
    fn name(&self) -> &str {
        "barrel-tube"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A hollow wave pitches its lip over a crouched surfer; the barrel rides right with progress, whitewater behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;

        // Barrel geometry: hollow ellipse (rx, ry) inside a lip ~3 dots thick.
        let rx = (h as f32 * 0.6).max(3.0);
        let ry = (h as f32 * 0.42).max(2.0);
        let travel = (w as f32 - 2.0 * rx - 12.0).max(0.0);
        let cx = 4.0 + rx + ctx.eased * travel;
        let cy = sea as f32 - ry * 0.55;
        let wobble = (ctx.time * 4.0).sin() * 0.4;
        let foam_end = (cx - rx - 3.0) as i32;

        for x in 0..wi {
            let fx = x as f32;
            let surf = sea as f32 + (fx * 0.25 - ctx.time * 3.0).sin() * 0.7;
            for y in 0..hi {
                let fy = y as f32;
                let dx = fx - cx;
                let dy = fy - cy;
                let inner = (dx / rx).powi(2) + (dy / ry).powi(2);
                let outer = (dx / (rx + 3.0)).powi(2) + (dy / (ry + 2.5 + wobble)).powi(2);
                // The lip: the ring between the ellipses, open at the lower
                // left where the tip hangs just above the water.
                let lip = outer <= 1.0 && inner > 1.0 && !(dx < 0.0 && dy > ry * 0.3);
                // The back of the wave slopes down to the flat sea on the right.
                let back = dx > 0.0 && inner > 1.0 && fy >= cy - ry - 2.0 + dx * 0.55;
                let sea_dot = fy >= surf;
                if (lip || back || sea_dot) && !(inner <= 1.0 && fy < sea as f32) {
                    draw::dot_i(grid, x, y);
                }
            }
            // Whitewater: churning foam stacked on the sea behind the barrel.
            if x < foam_end {
                let churn = ((x as f32 * 1.7 + ctx.time * 9.0).sin() + 1.0) * 1.5;
                let top = sea - 2 - churn as i32;
                for y in top..sea {
                    if (x + y + (ctx.time * 8.0) as i32) % 3 != 0 {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
        }

        // Crouched surfer and board deep in the tube.
        let sx = cx as i32 - 3;
        stamp(grid, sx, sea - 8, &CROUCH, false, false);
        board(grid, sx - 2, sea - 1, 11, false);

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(
                    grid,
                    r,
                    c,
                    c,
                    ctx.palette
                        .sample(t * 0.6 + 0.4 * r as f32 / cells_h as f32),
                );
            }
        }
        paint(grid, 0, 0, foam_end, sea - 1, FOAM);
        paint(
            grid,
            (cx - rx - 3.0) as i32,
            0,
            (cx + rx) as i32,
            (cy - ry - 1.0) as i32,
            FOAM,
        );
        paint(grid, sx, sea - 8, sx + 7, sea - 1, SKIN);
        paint(grid, sx - 2, sea - 1, sx - 1, sea - 1, BOARD);
        paint(grid, sx + 8, sea - 2, sx + 9, sea - 1, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Swell Lines
// ---------------------------------------------------------------------------
// Looking out to sea: corduroy lines of swell roll in from a dotted horizon
// at the top toward the viewer, spreading apart and thickening as they get
// closer (perspective). `time` marches the lines in; `eased` = how far across
// the bar the swell has filled in — beyond it the sea is still glassy.

struct SwellLines;
impl ProgressStyle for SwellLines {
    fn name(&self) -> &str {
        "swell-lines"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "Corduroy swell lines roll in from the horizon in perspective, filling across the sea with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let edge = (ctx.eased * w as f32).round() as i32;

        // Horizon: a dotted line across the full width.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, 0);
        }

        // Lines at perspective depths: `d` in 0..1 maps to y via d², so far
        // lines bunch up near the horizon and near ones spread out.
        let n = 6;
        let march = (ctx.time * 0.35).fract();
        for k in 0..n {
            let d = (k as f32 + march) / n as f32;
            let y = 1 + (d * d * (hi - 2) as f32) as i32;
            let thick = 1 + (d * 1.5) as i32;
            for x in 0..edge {
                let bend = ((x as f32 * 0.08 + k as f32).sin() * d * 1.5).round() as i32;
                for t in 0..thick {
                    draw::dot_i(grid, x, y + bend + t);
                }
            }
        }
        // Glassy sea ahead of the swell: sparse glints.
        for x in (edge.max(0)..wi).step_by(5) {
            draw::dot_i(grid, x, hi / 2 + ((x * 7) % (hi / 2).max(1)));
        }

        let (cells_w, cells_h) = grid.dimensions();
        for r in 0..cells_h {
            let t = r as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                r,
                0,
                cells_w.saturating_sub(1),
                ctx.palette.sample(1.0 - t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Surfer Carving the Face
// ---------------------------------------------------------------------------
// A surfer in a wide stance, arms out, rides a board across the water. The
// board's carved S-line is cut into the water behind them as a hollow wake
// and a fan of spray kicks up off the tail. `eased` = how far along the ride;
// `time` bobs the rider and flings the spray.

struct SurferCarve;
impl ProgressStyle for SurferCarve {
    fn name(&self) -> &str {
        "surfer-carve"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A surfer on a board rides right with progress, carving an S-shaped wake and throwing spray off the tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let sx = 2 + (ctx.eased * (wi - 16).max(0) as f32) as i32;

        // Water, with the carve line left hollow behind the board.
        for x in 0..wi {
            let surf = sea as f32 + (x as f32 * 0.22 - ctx.time * 3.0).sin() * 0.8;
            let carve = sea + 1 + ((x as f32 * 0.3).sin() + 1.0).round() as i32;
            for y in (surf.round() as i32).max(0)..hi {
                if !(x < sx && y == carve) {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Rider: bobs on the swell.
        let bob = ((ctx.time * 2.5).sin() * 1.2).round() as i32;
        let by = sea - 1 + bob.min(0);
        stamp(grid, sx + 2, by - 10, &SURFER, false, false);
        board(grid, sx, by, 14, false);

        // Spray fan thrown up behind the tail.
        for k in 0..8 {
            let ph = (ctx.time * 1.8 + k as f32 * 0.125).fract();
            let px = sx - 1 - (ph * 10.0) as i32;
            let py = by - ((ph * PI).sin() * 6.0) as i32;
            draw::dot_i(grid, px, py);
        }

        let (cells_w, _) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            draw::tint_row(grid, (sea / 4) as usize, c, c, ctx.palette.sample(t));
            draw::tint_row(
                grid,
                (hi / 4 - 1).max(0) as usize,
                c,
                c,
                ctx.palette.sample(t),
            );
        }
        paint(grid, 0, 0, sx - 1, sea - 2, FOAM);
        paint(grid, sx + 2, by - 10, sx + 11, by, SKIN);
        paint(grid, sx, by, sx + 1, by, BOARD);
        paint(grid, sx + 12, by - 1, sx + 14, by, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Wipeout
// ---------------------------------------------------------------------------
// Whitewater rolls in from the left as progress; at its leading edge a
// surfer has gone over the falls — upside down, legs kicking out of the
// foam — while the board cartwheels through the air above. `time` spins the
// board, kicks the legs and churns the splash.

struct Wipeout;
impl ProgressStyle for Wipeout {
    fn name(&self) -> &str {
        "wipeout"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A surfer goes over the falls: legs up in the whitewater, board cartwheeling overhead, foam advancing with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let fx = 6 + (ctx.eased * (wi - 20).max(0) as f32) as i32;

        // Sea ahead, heaped churning whitewater behind the crash.
        for x in 0..wi {
            let surf = sea as f32 + (x as f32 * 0.22 - ctx.time * 3.0).sin() * 0.7;
            let top = if x <= fx + 3 {
                let churn = ((x as f32 * 1.3 + ctx.time * 7.0).sin() + 1.0) * 1.2;
                sea - 2 - churn as i32
            } else {
                surf.round() as i32
            };
            for y in top.max(0)..hi {
                if x > fx + 3 || y >= sea || (x + y + (ctx.time * 6.0) as i32) % 3 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Surfer upside down: head under, legs kicking out of the foam.
        let kick = (ctx.time * 6.0).sin() > 0.0;
        stamp(grid, fx - 4, sea - 7, &SURFER, kick, true);

        // Board cartwheeling overhead.
        let a = ctx.time * 5.0;
        let (bx, by) = (fx + 9, 3 + ((ctx.time * 2.0).sin() * 1.0) as i32);
        for i in -5..=5 {
            let px = bx + (a.cos() * i as f32) as i32;
            let py = by + (a.sin() * i as f32 * 0.5) as i32;
            draw::dot_i(grid, px, py);
        }

        // Splash droplets bursting from the impact.
        for k in 0..10 {
            let ph = (ctx.time * 1.5 + k as f32 * 0.1).fract();
            let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
            let px = fx + (dir * ph * (4.0 + k as f32)) as i32;
            let py = sea - 2 - ((ph * PI).sin() * 7.0) as i32;
            draw::dot_i(grid, px, py);
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, 0, 0, fx + 3, sea - 1, FOAM);
        paint(grid, fx - 4, sea - 7, fx + 5, sea - 5, SKIN);
        paint(grid, bx - 5, by - 3, bx + 5, by + 3, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Cutback
// ---------------------------------------------------------------------------
// The surfer has ridden out onto the shoulder and whips the board back
// toward the breaking curl: rider facing back left, a carved white track
// hooking round behind them and a rooster tail of spray thrown high off the
// turn. `eased` = how far out the turn happens; `time` flings the spray.

struct Cutback;
impl ProgressStyle for Cutback {
    fn name(&self) -> &str {
        "cutback"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A surfer whips back toward the curl, throwing a rooster tail of spray; the turn moves right with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let turn = 14 + (ctx.eased * (wi - 30).max(0) as f32) as i32;

        water(grid, w, h, sea as f32, 0.7, ctx.time);

        // The curl breaking at the far left.
        for y in 0..sea {
            let k = (sea - y) as f32 / sea as f32;
            let lip = ((1.0 - k) * 4.0 + (ctx.time * 3.0).sin()) as i32;
            if y > 2 {
                draw::hline(grid, 0, lip.max(0) as usize, y as usize);
            }
        }
        for x in 0..6 {
            draw::dot_i(grid, x, 2 + (x - 3).abs() / 2);
        }

        // Carved track: out along the face, then hooking back round the turn.
        for x in 6..turn {
            if x % 2 == 0 {
                draw::dot_i(grid, x, sea - 1);
            }
        }
        for a in 0..10 {
            let th = a as f32 / 10.0 * PI;
            draw::dot_i(
                grid,
                turn + (th.sin() * 4.0) as i32,
                sea - 1 - (th.cos() * 2.0 + 2.0) as i32,
            );
        }

        // Rider facing back toward the curl.
        stamp(grid, turn - 9, sea - 11, &SURFER, true, false);
        board(grid, turn - 11, sea - 1, 13, true);

        // Rooster tail: spray arcing up and out off the turn.
        for k in 0..14 {
            let ph = (ctx.time * 1.4 + k as f32 / 14.0).fract();
            let px = turn + 2 + (ph * 12.0) as i32;
            let py = sea - 2 - ((ph * PI).sin() * (sea - 2) as f32) as i32;
            draw::dot_i(grid, px, py);
            if k % 2 == 0 {
                draw::dot_i(grid, px + 1, py + 1);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, 0, 0, 5, 3, FOAM);
        paint(grid, turn + 2, 0, turn + 15, sea - 1, FOAM);
        paint(grid, turn - 9, sea - 11, turn, sea - 1, SKIN);
        paint(grid, turn - 12, sea - 2, turn - 10, sea - 1, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Offshore Spray
// ---------------------------------------------------------------------------
// A steep wave rolls toward the beach (rightward) while an offshore wind
// tears a feathery plume of spray off its crest and blows it back over the
// top. Wind streaks race across the sky. `eased` carries the wave along;
// `time` streams the spray and the wind lines.

struct OffshoreSpray;
impl ProgressStyle for OffshoreSpray {
    fn name(&self) -> &str {
        "offshore-spray"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "Offshore wind feathers a plume of spray back off a steep wave's crest as it rolls right with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi - 3;
        let peak = 22 + (ctx.eased * (wi - 30).max(0) as f32) as i32;
        let height = (hi as f32 * 0.55) as i32;

        // Sea plus the wave: a long back slope and a steep front face.
        for x in 0..wi {
            let surf = sea as f32 + (x as f32 * 0.25 - ctx.time * 3.0).sin() * 0.6;
            let d = (x - peak) as f32;
            let rise = if d <= 0.0 {
                (1.0 + d / 20.0).max(0.0)
            } else {
                (1.0 - d / 4.0).max(0.0)
            };
            let top = (surf - rise * height as f32).round() as i32;
            draw::vline(grid, x as usize, top.max(0) as usize, h - 1);
        }

        // Spray plume: streams of droplets blown back up off the crest.
        let crest = sea - height;
        for s in 0..4 {
            for k in 0..9 {
                let ph = (ctx.time * 1.6 + k as f32 / 9.0 + s as f32 * 0.29).fract();
                let px = peak - 1 - (ph * (14.0 + s as f32 * 3.0)) as i32;
                let py = crest - 1 - (ph.sqrt() * (crest - s).max(1) as f32) as i32 + s;
                draw::dot_i(grid, px, py);
            }
        }

        // Wind streaks across the sky ahead of the wave.
        for k in 0..3 {
            let len = 5;
            let x0 = wi - ((ctx.time * 30.0 + k as f32 * 29.0) as i32).rem_euclid(wi + len);
            let y = 1 + k * 2;
            for i in 0..len {
                if x0 + i > peak + 3 {
                    draw::dot_i(grid, x0 + i, y);
                }
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, peak - 20, 0, peak + 1, crest - 1, FOAM);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Longboard Noseride
// ---------------------------------------------------------------------------
// A long, flat longboard trims across the water with the rider standing
// tall on the very nose, toes over the tip (hanging ten). `eased` carries the
// board along; `time` rocks the board and streams a wake from the tail.

struct NoseRide;
impl ProgressStyle for NoseRide {
    fn name(&self) -> &str {
        "noseride"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A longboarder hangs ten on the nose of a long board, trimming right with progress and leaving a wake"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let len = (wi / 3).clamp(10, 30);
        let bx = (ctx.eased * (wi - len - 2).max(0) as f32) as i32;
        let rock = ((ctx.time * 2.0).sin() * 0.8).round() as i32;
        let by = sea - 1 + rock.min(0);

        water(grid, w, h, sea as f32, 0.7, ctx.time);

        // Long board with a nose rocker.
        board(grid, bx, by, len, false);
        draw::dot_i(grid, bx, by - 1);

        // Rider upright on the nose, toes on the tip.
        stamp(grid, bx + len - 5, by - 10, &NOSE_RIDER, false, false);

        // Wake: a dashed foam line streaming off the tail.
        for x in 0..bx {
            if (x + (ctx.time * 10.0) as i32) % 4 != 0 {
                draw::dot_i(grid, x, sea - 1);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, 0, sea - 1, bx - 1, sea - 1, FOAM);
        paint(grid, bx + len - 5, by - 10, bx + len, by - 1, SKIN);
        paint(grid, bx, by, bx + len - 6, by, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Aerial Launch
// ---------------------------------------------------------------------------
// A crouched surfer hits the lip of a wave and launches into the air on a
// looping hop — board tilted nose-up on the way up, nose-down on the way
// down — with spray peeling off the lip. `eased` moves the launch point
// along the bar; `time` drives the hop cycle.

struct AerialLaunch;
impl ProgressStyle for AerialLaunch {
    fn name(&self) -> &str {
        "aerial-launch"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A crouched surfer boosts off a wave lip into the air and lands, launching further along with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi - 3;
        let lx = 6 + (ctx.eased * (wi - 30).max(0) as f32) as i32;

        // Sea with a peaking lip to launch from.
        for x in 0..wi {
            let d = (x - lx) as f32 / 4.0;
            let peak = 5.0 * (-d * d).exp();
            let surf = sea as f32 - peak + (x as f32 * 0.25 - ctx.time * 3.0).sin() * 0.6;
            draw::vline(grid, x as usize, (surf.round().max(0.0)) as usize, h - 1);
        }

        // Hop cycle: airborne for the first 70% of each loop.
        let ph = (ctx.time * 0.6).fract();
        let air = if ph < 0.7 { (ph / 0.7 * PI).sin() } else { 0.0 };
        let lift = (air * (sea - 9).max(0) as f32) as i32;
        let sx = lx + 3 + (ph.min(0.7) * 10.0) as i32;
        let by = sea - 1 - lift;
        // Nose up on the way up, nose down on the way down, flat on landing.
        let tilt = match ph {
            p if p < 0.35 => -1,
            p if p < 0.7 => 1,
            _ => 0,
        };
        stamp(grid, sx + 1, by - 8, &CROUCH, false, false);
        for i in 0..11 {
            draw::dot_i(grid, sx + i - 1, by + tilt * (i - 5) / 4);
        }

        // Spray peeling off the lip behind the launch.
        for k in 0..8 {
            let p = (ctx.time * 2.0 + k as f32 * 0.125).fract();
            draw::dot_i(
                grid,
                lx - (p * 8.0) as i32,
                sea - 5 - ((p * PI).sin() * 3.0) as i32,
            );
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, lx - 8, 0, lx, sea - 3, FOAM);
        paint(grid, sx + 1, by - 8, sx + 8, by - 2, SKIN);
        if by / 4 < sea / 4 {
            paint(grid, sx - 1, by, sx + 10, by, BOARD);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Tide Line
// ---------------------------------------------------------------------------
// A sandy beach slopes up to the right. The sea climbs it as the tide comes
// in with progress — the waterline creeps up the sand — while a lacy line of
// swash foam runs up and slides back with `time`. A wet-sand band marks the
// last high-water line.

struct TideLine;
impl ProgressStyle for TideLine {
    fn name(&self) -> &str {
        "tide-line"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "The tide floods up a sloping sand beach with progress while lacy swash foam runs up and slides back"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        // Beach surface rises from the bottom-left to mid-height on the right.
        let sand_top = |x: i32| hi - 2 - (x as f32 / wi as f32 * (hi as f32 * 0.6)) as i32;
        // Waterline: where the sea surface meets the sand, moving right.
        let shore = 4 + (ctx.eased * (wi as f32 * 0.72)) as i32;
        let level = sand_top(shore);
        let swash = ((ctx.time * 1.6).sin() * 0.5 + 0.5) * 6.0;
        let reach = shore + swash as i32;

        for x in 0..wi {
            let st = sand_top(x);
            // Sand, speckled so it reads as grains.
            for y in st..hi {
                if x < reach || (x + y * 3) % 4 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
            // Sea over the sand up to the waterline, with a rolling surface.
            if x < shore {
                let s = level + ((x as f32 * 0.3 - ctx.time * 3.0).sin() * 0.7).round() as i32;
                for y in s.max(0)..st {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Lacy swash foam running up the slope.
        for x in shore..=reach {
            if (x + (ctx.time * 6.0) as i32) % 3 != 0 {
                draw::dot_i(grid, x, sand_top(x) - 1);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(r as f32 / cells_h as f32));
            }
        }
        for x in (shore + 8..wi).step_by(2) {
            paint(grid, x, sand_top(x), x, hi - 1, SAND);
        }
        paint(grid, shore, level - 2, reach + 1, sand_top(shore) + 1, FOAM);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Palm Tree and Sunset
// ---------------------------------------------------------------------------
// A curved palm trunk rises from a little sand spit on the left, its crown
// of drooping fronds swaying in the breeze. Out on the horizon a big striped
// sun sinks into the sea as progress grows, laying a shimmering reflection
// on the water beneath it.

struct PalmSunset;
impl ProgressStyle for PalmSunset {
    fn name(&self) -> &str {
        "palm-sunset"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A swaying palm tree on a sand spit while a striped sun sinks into the sea with progress, reflection shimmering"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let horizon = hi * 3 / 4;

        // Sun: a striped disc sinking through the horizon.
        let r = (h as f32 * 0.38).max(2.0);
        let sun_x = w as f32 * 0.62;
        let sun_y = lerp(r * 0.9, horizon as f32 + r, ctx.eased);
        for y in 0..horizon {
            let dy = y as f32 - sun_y;
            if dy.abs() > r {
                continue;
            }
            // Retro bands cut through the lower half of the disc.
            if dy > 0.0 && (y + 1) % 3 == 0 {
                continue;
            }
            let half = (r * r - dy * dy).max(0.0).sqrt() * 1.5;
            draw::hline(
                grid,
                (sun_x - half).max(0.0) as usize,
                (sun_x + half).max(0.0) as usize,
                y as usize,
            );
        }

        // Sea: horizon line plus shimmering reflection strokes under the sun.
        draw::hline(grid, 0, w - 1, horizon as usize);
        let glow = (1.0 - ctx.eased * 0.7) * r * 1.4;
        for y in (horizon + 2..hi).step_by(2) {
            let jitter = ((ctx.time * 3.0 + y as f32).sin() * 3.0) as i32;
            let half = (glow * (1.0 - (y - horizon) as f32 / (hi - horizon) as f32 * 0.5)) as i32;
            for x in (sun_x as i32 - half + jitter)..=(sun_x as i32 + half + jitter) {
                if (x + y) % 5 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Sand spit at the palm's foot.
        let tx = (wi / 8).max(3);
        for x in 0..tx * 2 + 4 {
            let mound = (2.0 * (1.0 - ((x - tx) as f32 / (tx + 3) as f32).powi(2))).max(0.0);
            draw::vline(
                grid,
                x as usize,
                (horizon - mound as i32).max(0) as usize,
                horizon as usize,
            );
        }

        // Curved trunk, two dots thick, leaning right as it rises.
        let crown_y = 3;
        let sway = (ctx.time * 1.2).sin();
        let mut crown_x = tx;
        for y in crown_y..=horizon {
            let k = (horizon - y) as f32 / (horizon - crown_y).max(1) as f32;
            let x = tx + (k * k * (4.0 + sway)) as i32;
            draw::dot_i(grid, x, y);
            draw::dot_i(grid, x + 1, y);
            if y == crown_y {
                crown_x = x;
            }
        }

        // Fronds: long arcs that rise off the crown and droop at the tips,
        // with leaflets hanging beneath like a comb.
        for (i, &dir) in [-1.0f32, -0.6, 0.6, 1.0].iter().enumerate() {
            let len = if dir.abs() > 0.9 { 12 } else { 9 };
            let flap = (ctx.time * 2.0 + i as f32).sin() * 0.8;
            for s in 0..len {
                let f = s as f32 / len as f32;
                let x = crown_x as f32 + dir * s as f32 * 1.2 + sway;
                let y = crown_y as f32 - 2.5 * (f * PI * 0.8).sin() + f * f * (6.0 + flap);
                draw::dot_i(grid, x as i32, y as i32);
                if s > 1 && s % 2 == 0 {
                    draw::dot_i(grid, x as i32, y as i32 + 1);
                    draw::dot_i(grid, x as i32, y as i32 + 2);
                }
            }
        }
        // Coconuts.
        draw::dot_i(grid, crown_x, crown_y + 2);
        draw::dot_i(grid, crown_x + 2, crown_y + 2);

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(r as f32 / cells_h as f32));
            }
        }
        paint(grid, 0, 0, wi - 1, horizon - 1, SKY);
        paint(
            grid,
            (sun_x - r * 1.5) as i32,
            (sun_y - r) as i32,
            (sun_x + r * 1.5) as i32,
            horizon - 1,
            SUN,
        );
        paint(
            grid,
            (sun_x - glow) as i32 - 3,
            horizon + 1,
            (sun_x + glow) as i32 + 3,
            hi - 1,
            BOARD,
        );
        paint(grid, 0, horizon - 2, tx * 2 + 4, horizon, SAND);
        paint(grid, tx, crown_y + 4, tx + 6, horizon - 3, TRUNK);
        paint(grid, crown_x - 15, 0, crown_x + 16, crown_y + 3, LEAF);
        paint(
            grid,
            crown_x - 15,
            crown_y + 4,
            crown_x - 4,
            crown_y + 7,
            LEAF,
        );
        paint(
            grid,
            crown_x + 5,
            crown_y + 4,
            crown_x + 16,
            crown_y + 7,
            LEAF,
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Wave Sets
// ---------------------------------------------------------------------------
// Surf arrives in "sets": groups of three breaking waves, biggest first.
// Each wave is a steep-faced hump whose white lip curls over the front.
// `eased` = how many sets have rolled in (left to right); `time` pitches the
// lips and churns the foam at their feet.

struct WaveSets;
impl ProgressStyle for WaveSets {
    fn name(&self) -> &str {
        "wave-sets"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "Sets of three curling breakers, biggest first, roll in across the bar as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi - 2;
        // Flat sea between sets.
        for x in 0..wi {
            draw::vline(grid, x as usize, sea as usize, h - 1);
        }
        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }

        let set_w = 30;
        let sets = (wi / set_w).max(1);
        let arrived = ctx.eased * sets as f32;
        for s in 0..sets {
            let g = (arrived - s as f32).clamp(0.0, 1.0);
            for k in 0..3 {
                // Each wave rises in turn as its set arrives.
                let rise = (g * 3.0 - k as f32).clamp(0.0, 1.0);
                if rise <= 0.0 {
                    continue;
                }
                let size = [1.0f32, 0.8, 0.6][k as usize];
                let hgt = ((hi - 4) as f32 * size * rise) as i32;
                let wid = (10.0 * size) as i32 + 2;
                let x0 = s * set_w + k * 10 + 1;
                let pitch = ((ctx.time * 3.0 + k as f32 + s as f32).sin() * 1.0).round() as i32;
                // Hump with a steep front (left) and a long back slope.
                for dx in 0..wid {
                    let u = dx as f32 / wid as f32;
                    let f = if u < 0.3 {
                        u / 0.3
                    } else {
                        1.0 - (u - 0.3) / 0.7
                    };
                    let top = sea - (f * hgt as f32) as i32;
                    for y in top..sea {
                        draw::dot_i(grid, x0 + dx, y);
                    }
                }
                // Curling lip hooking over the front.
                let cx = x0 + (wid as f32 * 0.3) as i32;
                let ct = sea - hgt;
                for (lx, ly) in [(0, -1), (-1, -1), (-2, 0), (-3, 1), (-3, 2)] {
                    draw::dot_i(grid, cx + lx + pitch.min(0), ct + ly);
                }
                // Foam at the foot.
                for dx in -3..2 {
                    if (dx + (ctx.time * 8.0) as i32) % 2 == 0 {
                        draw::dot_i(grid, x0 + dx, sea - 1);
                    }
                }
                if hgt > 2 {
                    paint(grid, cx - 4, ct - 1, cx + 1, ct + 1, FOAM);
                }
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Surf Van
// ---------------------------------------------------------------------------
// The classic two-tone split-window surf bus drives down the coast road with
// a longboard strapped to its roof rack. Windows and wheel arches are cut as
// negative space; the wheels spin, the road dashes scroll by and exhaust
// puffs trail behind. `eased` = how far down the road the van has got.

struct SurfVan;
impl ProgressStyle for SurfVan {
    fn name(&self) -> &str {
        "surf-van"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A two-tone surf bus with a longboard on the roof rack drives right with progress, wheels spinning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 16 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let wi = w as i32;
        let base = h as i32 - 16; // van art is 16 dots tall, sat on the bottom
        let len = 30;
        let bx = 4 + (ctx.eased * (wi - len - 6).max(0) as f32) as i32;
        let wheels = [bx + 6, bx + 23];
        let wy = base + 12;

        // Road dashes scrolling past.
        for x in 0..wi {
            if (x + (ctx.time * 24.0) as i32).rem_euclid(8) < 4 {
                draw::dot_i(grid, x, base + 15);
            }
        }

        // Longboard on the roof rack.
        draw::hline(
            grid,
            (bx + 3).max(0) as usize,
            (bx + 26).max(0) as usize,
            base.max(0) as usize,
        );
        draw::hline(
            grid,
            (bx + 1).max(0) as usize,
            (bx + 28).max(0) as usize,
            (base + 1) as usize,
        );
        for rx in [bx + 6, bx + 22] {
            draw::vline(
                grid,
                rx.max(0) as usize,
                (base + 2) as usize,
                (base + 3) as usize,
            );
        }

        // Body with rounded corners, a sloped nose, windows and wheel arches.
        for x in 0..len {
            for y in 4..=12 {
                let corner = (x == 0 || x == len - 1) && (y == 4 || y == 12);
                let nose = x >= len - 3 && y < 4 + (x - (len - 4)) * 2;
                let window = (5..=7).contains(&y)
                    && ((2..=7).contains(&x)
                        || (10..=15).contains(&x)
                        || (18..=22).contains(&x)
                        || (25..=27).contains(&x));
                let arch = wheels.iter().any(|&c| {
                    let (dx, dy) = ((bx + x - c) as f32, (y - 12) as f32);
                    dx * dx + dy * dy < 12.0
                });
                let belt = y == 8 && x % 2 == 0;
                if !(corner || nose || window || arch || belt) {
                    draw::dot_i(grid, bx + x, base + y);
                }
            }
        }

        // Wheels: rims with a spinning spoke.
        let spin = ctx.time * 9.0;
        for &c in &wheels {
            for s in 0..12 {
                let a = s as f32 / 12.0 * 2.0 * PI;
                draw::dot_i(
                    grid,
                    c + (a.cos() * 2.4).round() as i32,
                    wy + (a.sin() * 2.4).round() as i32,
                );
            }
            draw::dot_i(grid, c, wy);
            draw::dot_i(
                grid,
                c + spin.cos().round() as i32,
                wy + spin.sin().round() as i32,
            );
        }

        // Exhaust puffs drifting back.
        for k in 0..3 {
            let ph = (ctx.time * 1.2 + k as f32 / 3.0).fract();
            draw::dot_i(
                grid,
                bx - 1 - (ph * 10.0) as i32,
                base + 11 - (ph * 3.0) as i32,
            );
        }

        let (cells_w, cells_h) = grid.dimensions();
        for r in 0..cells_h {
            draw::tint_row(
                grid,
                r,
                0,
                cells_w.saturating_sub(1),
                ctx.palette.sample(r as f32 / cells_h as f32),
            );
        }
        paint(grid, bx, base, bx + len, base + 3, BOARD);
        paint(grid, bx, base + 4, bx + len, base + 7, FOAM);
        paint(grid, bx, base + 8, bx + len, base + 11, VAN);
        paint(grid, bx, base + 12, bx + len, base + 15, TYRE);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shared sprites and helpers
// ---------------------------------------------------------------------------

/// Cell colours for the pictorial bars.
const SKIN: Color = Color::rgb(250, 228, 196);
const BOARD: Color = Color::rgb(255, 168, 60);
const FOAM: Color = Color::rgb(225, 240, 255);
const VAN: Color = Color::rgb(60, 200, 190);
const TYRE: Color = Color::rgb(150, 150, 165);
const SUN: Color = Color::rgb(255, 120, 60);
const SKY: Color = Color::rgb(150, 90, 200);
const SAND: Color = Color::rgb(230, 200, 130);
const TRUNK: Color = Color::rgb(170, 110, 60);
const LEAF: Color = Color::rgb(80, 200, 90);

/// Surfer in a wide stance, arms out, facing right (10×10 dots).
const SURFER: [&str; 10] = [
    "....##....",
    "....##....",
    "##.####.##",
    "..#.##.#..",
    "....##....",
    "....##....",
    "...####...",
    "..##..##..",
    ".##....##.",
    ".#......#.",
];

/// Surfer crouched low, arms forward (8×7 dots) — for tubes and airs.
const CROUCH: [&str; 7] = [
    "....##..", "....##..", "#..####.", ".#.##..#", "..####..", ".##..##.", ".#....#.",
];

/// Longboarder standing tall and still, arms loose (6×10 dots).
const NOSE_RIDER: [&str; 10] = [
    "..##..", "..##..", ".####.", "#.##.#", "#.##.#", "..##..", "..##..", ".#..#.", ".#..#.",
    ".#..#.",
];

/// Stamp an ASCII sprite (`#` = dot) with its top-left at `(x, y)`.
/// `mirror` flips it left↔right; `upside` flips it top↔bottom.
fn stamp(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str], mirror: bool, upside: bool) {
    let n = rows.len() as i32;
    for (ry, row) in rows.iter().enumerate() {
        let len = row.len() as i32;
        for (rx, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                let dx = if mirror {
                    len - 1 - rx as i32
                } else {
                    rx as i32
                };
                let dy = if upside { n - 1 - ry as i32 } else { ry as i32 };
                draw::dot_i(grid, x + dx, y + dy);
            }
        }
    }
}

/// Surfboard `len` dots long with its tail at `(x, y)`; the nose kicks up.
fn board(grid: &mut BrailleGrid, x: i32, y: i32, len: i32, mirror: bool) {
    let len = len.max(2);
    for i in 0..len {
        draw::dot_i(grid, x + i, y);
    }
    let nose = if mirror { x - 1 } else { x + len };
    draw::dot_i(grid, nose, y - 1);
}

/// Fill water below a sine surface centred on row `base`.
fn water(grid: &mut BrailleGrid, w: usize, h: usize, base: f32, amp: f32, time: f32) {
    for x in 0..w {
        let s = base + (x as f32 * 0.22 - time * 3.0).sin() * amp;
        let top = (s.round().max(0.0) as usize).min(h.saturating_sub(1));
        draw::vline(grid, x, top, h.saturating_sub(1));
    }
}

/// Colour every cell overlapping the dot rectangle `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    grid.enable_color_support();
    let (cw, ch) = grid.dimensions();
    if cw == 0 || ch == 0 || x1 < 0 || y1 < 0 || x1 < x0 || y1 < y0 {
        return;
    }
    let (cx0, cy0) = ((x0.max(0) / 2) as usize, (y0.max(0) / 4) as usize);
    let (cx1, cy1) = (
        ((x1 / 2) as usize).min(cw - 1),
        ((y1 / 4) as usize).min(ch - 1),
    );
    for cy in cy0..=cy1 {
        for cx in cx0..=cx1 {
            let _ = grid.set_cell_color(cx, cy, color);
        }
    }
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::surf::styles();
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
