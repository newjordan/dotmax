//! `food` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O food.rs && ./food [style-name]
//! ```

const DEFAULT_STYLE: &str = "beer-glass";

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
    pub mod food {
//! Food / kitchen progress bars — a full set of animated, braille-rendered
//! loading styles themed around cooking, eating, and the kitchen.
//!
//! Every bar is stateless: it is a pure function of `(ctx.eased, ctx.time)`.
//! Bubbles rise, steam drifts, conveyor plates slide, and popcorn pops — all
//! driven by `ctx.time` so the bars stay alive at any fixed progress value.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic hash — gives pseudo-random f32 in [0, 1) from any u32 seed.
// ---------------------------------------------------------------------------
#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1_000) as f32 / 1_000.0
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Beer glass filling with a foam head and rising bubbles.
struct BeerGlass;
impl ProgressStyle for BeerGlass {
    fn name(&self) -> &str {
        "beer-glass"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Beer glass fills with amber liquid; foam head and rising bubbles animate over time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        // Glass outline — tapered slightly: wider at top, narrower at base.
        let left = 1usize;
        let right = w.saturating_sub(2);
        // Left and right walls
        draw::vline(grid, left, 0, h - 1);
        draw::vline(grid, right, 0, h - 1);
        // Base
        draw::hline(grid, left, right, h - 1);

        // Liquid level from the bottom, driven by eased progress.
        let liquid_h = (ctx.eased * (h - 2) as f32).round() as usize;
        let liquid_top = h.saturating_sub(1).saturating_sub(liquid_h);
        if liquid_h > 0 {
            draw::fill_rect(
                grid,
                left + 1,
                liquid_top,
                right.saturating_sub(left + 1).max(1),
                liquid_h,
            );
        }

        // Foam head: 1–2 dot rows above the liquid.
        let foam_rows = ((ctx.eased * 2.0).round() as usize).min(2);
        for fr in 0..foam_rows {
            let fy = liquid_top.saturating_sub(fr + 1);
            // Bumpy foam: dots spaced every 2.
            let mut fx = left + 1;
            while fx < right {
                draw::dot(grid, fx, fy);
                fx += 2;
            }
        }

        // Bubbles: small dots that rise from the bottom of the liquid.
        if liquid_h > 1 {
            let bubble_count = 4usize;
            for i in 0..bubble_count {
                let bx = left
                    + 1
                    + (hashf(i as u32) * (right.saturating_sub(left + 2)).max(1) as f32) as usize;
                // Period varies per bubble so they don't all move in sync.
                let period = 1.5 + hashf(i as u32 + 100) * 2.0;
                let phase = hashf(i as u32 + 200);
                let t = (ctx.time / period + phase).fract();
                // Only show bubble while it's inside the liquid column.
                let by_raw = liquid_top + ((1.0 - t) * liquid_h as f32) as usize;
                if by_raw < h.saturating_sub(1) {
                    draw::dot(grid, bx.min(right.saturating_sub(1)), by_raw);
                }
            }
        }

        // Palette tint — amber/gold over the liquid rows.
        let (cw, ch) = grid.dimensions();
        let liq_cell_top = liquid_top / 4;
        for cy in liq_cell_top..ch {
            let t = if liquid_h == 0 {
                0.5
            } else {
                (cy.saturating_sub(liq_cell_top)) as f32 / ch.max(1) as f32
            };
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Coffee cup filling with steam wisps drifting upward.
struct CoffeePour;
impl ProgressStyle for CoffeePour {
    fn name(&self) -> &str {
        "coffee-pour"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Coffee cup fills with dark brew; sinusoidal steam wisps drift upward as it warms"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 6 || h < 4 {
            return Ok(());
        }

        // Cup outline.
        let left = w / 6;
        let right = w.saturating_sub(w / 6 + 1);
        draw::hline(grid, left, right, h - 1); // base
        draw::vline(grid, left, h / 3, h - 1); // left wall
        draw::vline(grid, right, h / 3, h - 1); // right wall
        draw::hline(grid, left, right, h / 3); // rim

        // Handle: a small arc on the right side.
        let hx = right + 1;
        let hmid = (h / 3 + h - 1) / 2;
        draw::dot(grid, hx.min(w - 1), hmid.saturating_sub(1));
        draw::dot(grid, hx.min(w - 1), hmid);
        draw::dot(grid, hx.min(w - 1), hmid + 1);

        // Coffee fill from bottom of cup up.
        let cup_inner_h = h.saturating_sub(h / 3 + 2);
        let fill_h = (ctx.eased * cup_inner_h as f32).round() as usize;
        if fill_h > 0 {
            let fill_top = h.saturating_sub(1).saturating_sub(fill_h);
            draw::fill_rect(
                grid,
                left + 1,
                fill_top,
                right.saturating_sub(left + 1).max(1),
                fill_h,
            );
        }

        // Steam wisps above the cup: sine waves drifting up with time.
        let wisp_count = 3usize;
        let steam_amount = ctx.eased;
        for i in 0..wisp_count {
            if steam_amount < (i as f32 * 0.3) {
                continue;
            }
            let base_x =
                left + 1 + i * ((right.saturating_sub(left + 1)).max(3) / (wisp_count).max(1));
            let phase = (i as f32) * 2.0 * PI / wisp_count as f32;
            let rise_speed = 0.8 + i as f32 * 0.3;
            for dot_y in 0..h / 3 {
                // Phase advances with time to make the wisp drift upward.
                let t = dot_y as f32 / (h / 3).max(1) as f32;
                let drift_t = (ctx.time * rise_speed + t * 3.0 + phase).sin() * 2.0;
                let sx = base_x as i32 + drift_t as i32;
                // Fade out near top (only draw some dots for wispiness).
                let density = ((1.0 - t) * 3.0) as usize;
                if dot_y % (density.max(1)) == 0 {
                    draw::dot_i(grid, sx, dot_y as i32);
                }
            }
        }

        // Palette tint over the fill area.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let color = ctx.palette.sample(cy as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pizza being eaten: slices disappear as progress rises.
struct PizzaSlices;
impl ProgressStyle for PizzaSlices {
    fn name(&self) -> &str {
        "pizza-slices"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Pizza loses slices as progress climbs — radial wedges removed from 8-slice pie"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        let cx = w / 2;
        let cy = h / 2;
        let r = (w.min(h) / 2).saturating_sub(1).max(1);

        // How many slices remain (out of 8). At progress=0 → 8 slices, at 1 → 0.
        let total_slices = 8usize;
        let eaten = (ctx.eased * total_slices as f32).round() as usize;
        let remaining = total_slices.saturating_sub(eaten);

        // Draw filled wedges for remaining slices.
        let slice_angle = 2.0 * PI / total_slices as f32;
        // Rotate so we start at the top.
        let start_offset = -PI / 2.0;

        for s in 0..remaining {
            let a0 = start_offset + s as f32 * slice_angle;
            let a1 = a0 + slice_angle;
            // Rasterise the wedge by scanning all dots in bounding box.
            for dy in 0..h {
                for dx in 0..w {
                    let fx = dx as f32 - cx as f32;
                    let fy = dy as f32 - cy as f32;
                    let dist = (fx * fx + fy * fy).sqrt();
                    if dist > r as f32 {
                        continue;
                    }
                    let mut angle = fy.atan2(fx);
                    // Normalise angle into [a0, a1] range.
                    // atan2 ∈ [-π, π] and a0 < 3π/2, so this runs at most twice.
                    #[allow(clippy::while_float)]
                    while angle < a0 {
                        angle += 2.0 * PI;
                    }
                    if angle <= a1 {
                        draw::dot(grid, dx, dy);
                    }
                }
            }
        }

        // Crust ring.
        for dot_y in 0..h {
            for dot_x in 0..w {
                let fx = dot_x as f32 - cx as f32;
                let fy = dot_y as f32 - cy as f32;
                let dist = (fx * fx + fy * fy).sqrt();
                if dist >= r as f32 && dist <= r as f32 + 1.0 {
                    draw::dot(grid, dot_x, dot_y);
                }
            }
        }

        // Palette tint.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let color = ctx.palette.sample(cy_c as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Boiling pot: liquid heat-fills from bottom, bubbles pop at the surface.
struct BoilingPot;
impl ProgressStyle for BoilingPot {
    fn name(&self) -> &str {
        "boiling-pot"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Pot fills with boiling liquid; bubbles pop vigorously at the surface driven by time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        // Pot outline: rounded bottom, straight walls, rim with handles.
        let left = 2usize;
        let right = w.saturating_sub(3);
        let rim = 1usize;
        let base = h - 1;

        // Pot walls.
        draw::vline(grid, left, rim, base);
        draw::vline(grid, right, rim, base);
        // Pot base.
        draw::hline(grid, left + 1, right.saturating_sub(1), base);
        // Rim.
        draw::hline(grid, left, right, rim);
        // Handles.
        draw::dot(grid, left.saturating_sub(1), rim);
        draw::dot(grid, left.saturating_sub(1), rim + 1);
        draw::dot(grid, right + 1, rim);
        draw::dot(grid, right + 1, rim + 1);

        // Fill: liquid from base upward.
        let inner_w = right.saturating_sub(left + 1).max(1);
        let inner_h = base.saturating_sub(rim + 1).max(1);
        let fill_h = (ctx.eased * inner_h as f32).round() as usize;
        if fill_h > 0 {
            let fill_top = base.saturating_sub(fill_h);
            draw::fill_rect(grid, left + 1, fill_top, inner_w, fill_h);

            // Bubbles at the surface: time-driven pop timing.
            let surface_y = fill_top;
            let bubble_count = 6usize;
            for i in 0..bubble_count {
                let bx_frac = hashf(i as u32 + 77);
                let bx = left + 1 + (bx_frac * (inner_w.saturating_sub(1)) as f32) as usize;
                // Each bubble pops on its own period.
                let period = 0.4 + hashf(i as u32 + 200) * 0.8;
                let t = (ctx.time / period + hashf(i as u32 + 300)).fract();
                // Pop arc: rises a few dots above the surface then disappears.
                if t < 0.5 {
                    let rise = (t * 2.0 * PI).sin(); // 0..0 parabola via sine half-wave
                    let bubble_y = surface_y as i32 - (rise * 3.0) as i32;
                    draw::dot_i(grid, bx as i32, bubble_y);
                    // Small splat ring at apex.
                    if t > 0.3 && t < 0.45 {
                        draw::dot_i(grid, bx as i32 - 1, bubble_y);
                        draw::dot_i(grid, bx as i32 + 1, bubble_y);
                    }
                }
            }
        }

        // Palette tint.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let color = ctx.palette.sample(cy as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Sushi conveyor: plates slide right, count served = eased.
struct SushiConveyor;
impl ProgressStyle for SushiConveyor {
    fn name(&self) -> &str {
        "sushi-conveyor"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Sushi plates slide along a conveyor belt; plates served scales with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 6 || h < 3 {
            return Ok(());
        }

        // Two belt rails.
        let rail_top = 0usize;
        let rail_bot = h.saturating_sub(1);
        draw::hline(grid, 0, w - 1, rail_top);
        draw::hline(grid, 0, w - 1, rail_bot);

        // Belt notches (static texture).
        let notch_gap = 4usize;
        let mut nx = 0;
        while nx < w {
            draw::dot(grid, nx, rail_top + 1);
            draw::dot(grid, nx, rail_bot.saturating_sub(1));
            nx += notch_gap;
        }

        // Total plates to show on the belt at full progress.
        let max_plates = (w / 10).max(2);
        let served = (ctx.eased * max_plates as f32).round() as usize;

        // Plate width/height in dots.
        let pw = 6usize;
        let ph = h.saturating_sub(2).max(2);
        let plate_gap = (w / max_plates.max(1)).max(pw + 2);
        let belt_speed = 8.0_f32; // dots per second

        for i in 0..max_plates {
            // Shift each plate with time so they slide right.
            let base_x = i * plate_gap;
            let scroll = (ctx.time * belt_speed) as usize % (w + pw);
            let plate_x = (base_x + scroll) % (w + pw);

            // Only draw plates that have been "served" (within eased count).
            if i >= served {
                continue;
            }

            let px0 = plate_x.min(w.saturating_sub(1));
            // Plate oval: fill a rounded rect.
            let ph_inner = ph.saturating_sub(2).max(1);
            draw::fill_rect(grid, px0, 1 + 1, pw.min(w.saturating_sub(px0)), ph_inner);
            // Plate border top/bottom.
            draw::hline(grid, px0, (px0 + pw).min(w - 1), 1);
            draw::hline(grid, px0, (px0 + pw).min(w - 1), 1 + ph_inner + 1);

            // Nigiri topping: a small bump in the center of the plate.
            let top_x = px0 + pw / 2;
            if top_x < w {
                draw::dot(grid, top_x, 2);
                draw::dot(grid, top_x, 3);
            }
        }

        // Palette tint on belt rows.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Popcorn kernels launching on parabolic arcs, filling a box.
struct PopcornPopping;
impl ProgressStyle for PopcornPopping {
    fn name(&self) -> &str {
        "popcorn-popping"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Kernels pop on random parabolic arcs; the box fills with fluffy popcorn as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        // Box outline.
        draw::rect_outline(grid, 0, 0, w, h);

        // Settled popcorn pile: fill from the bottom up, scaled by eased.
        let pile_h = (ctx.eased * (h - 2) as f32).round() as usize;
        if pile_h > 0 {
            // Wavy top edge with small sine bumps.
            let pile_top = h.saturating_sub(1).saturating_sub(pile_h);
            draw::fill_rect(
                grid,
                1,
                pile_top + 1,
                w.saturating_sub(2).max(1),
                pile_h.saturating_sub(1),
            );
            // Wavy surface: alternate dots on top row.
            for px in (1..w.saturating_sub(1)).step_by(2) {
                let wave = ((px as f32 * 0.7 + ctx.time * 0.5).sin() * 1.0) as i32;
                draw::dot_i(grid, px as i32, pile_top as i32 + wave);
            }
        }

        // Airborne popcorn: each kernel follows a parabolic arc.
        let kernel_count = 8usize;
        for i in 0..kernel_count {
            // Each kernel has a random launch time offset.
            let offset = hashf(i as u32) * 2.0;
            let period = 0.6 + hashf(i as u32 + 50) * 0.7;
            let t = ((ctx.time * 1.2 + offset) / period).fract();

            // Only in the air during rising phase (t < 0.7).
            if t > 0.7 {
                continue;
            }
            if ctx.eased < hashf(i as u32 + 10) * 0.9 {
                continue;
            } // threshold before launch

            // Horizontal: random x near center spread.
            let lx = 2 + (hashf(i as u32 + 20) * (w.saturating_sub(4)) as f32) as usize;
            // Vertical: parabola — peaks in the middle of the phase.
            let up = -((t / 0.35 - 1.0).powi(2) - 1.0); // 0→1→0 arc
            let ky = (h as f32 - 2.0) * (1.0 - up * 0.85) - 1.0;
            let ky = ky as i32;

            draw::dot_i(grid, lx as i32, ky);
            draw::dot_i(grid, lx as i32 + 1, ky);
            draw::dot_i(grid, lx as i32, ky - 1);
        }

        // Palette tint.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let color = ctx.palette.sample(cy as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Egg hourglass: sand empties from top chamber, fills bottom chamber.
struct EggTimer;
impl ProgressStyle for EggTimer {
    fn name(&self) -> &str {
        "egg-timer"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Egg-timer hourglass drains sand from the top into the bottom as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 6 {
            return Ok(());
        }

        let mid_y = h / 2;
        let cx = w / 2;

        // Outer hourglass silhouette — two triangles meeting at the waist.
        for dy in 0..mid_y {
            let spread = ((1.0 - dy as f32 / mid_y as f32) * (cx as f32 - 1.0)).round() as usize;
            let x0 = cx.saturating_sub(spread);
            let x1 = (cx + spread).min(w - 1);
            draw::dot(grid, x0, dy);
            draw::dot(grid, x1, dy);
        }
        for dy in mid_y..h {
            let spread =
                (((dy - mid_y) as f32 / mid_y.max(1) as f32) * (cx as f32 - 1.0)).round() as usize;
            let x0 = cx.saturating_sub(spread);
            let x1 = (cx + spread).min(w - 1);
            draw::dot(grid, x0, dy);
            draw::dot(grid, x1, dy);
        }
        // Waist pinch (single dot at center for each side).
        draw::dot(grid, cx.saturating_sub(1), mid_y);
        draw::dot(grid, (cx + 1).min(w - 1), mid_y);

        // Top sand (draining): fills from top downward, decreases as progress rises.
        let top_fill = ((1.0 - ctx.eased) * (mid_y as f32 - 1.0)).round() as usize;
        for row in 0..top_fill {
            let spread =
                ((1.0 - row as f32 / mid_y.max(1) as f32) * (cx as f32 - 2.0)).round() as usize;
            let x0 = cx.saturating_sub(spread);
            let x1 = (cx + spread).min(w - 1);
            if x1 > x0 {
                draw::hline(grid, x0 + 1, x1.saturating_sub(1), row);
            }
        }

        // Bottom sand (accumulating): fills from bottom upward as progress rises.
        let bot_fill = (ctx.eased * (mid_y as f32 - 1.0)).round() as usize;
        for row in 0..bot_fill {
            let abs_row = h.saturating_sub(1).saturating_sub(row);
            let spread_t = row as f32 / mid_y.max(1) as f32;
            let spread = (spread_t * (cx as f32 - 2.0)).round() as usize;
            let x0 = cx.saturating_sub(spread);
            let x1 = (cx + spread).min(w - 1);
            if x1 > x0 {
                draw::hline(grid, x0 + 1, x1.saturating_sub(1), abs_row);
            }
        }

        // Falling sand particle: a dot that travels from waist to mid when in-flight.
        if ctx.eased > 0.0 && ctx.eased < 1.0 {
            let fall_t = (ctx.time * 3.0).fract();
            let particle_y = mid_y + (fall_t * mid_y as f32) as usize;
            draw::dot(grid, cx, particle_y.min(h - 1));
        }

        // Palette tint.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let color = ctx.palette.sample(cy as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pancake stack growing one pancake at a time.
struct PancakeStack;
impl ProgressStyle for PancakeStack {
    fn name(&self) -> &str {
        "pancake-stack"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "A stack of pancakes grows taller with each progress increment; syrup drips animate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 3 {
            return Ok(());
        }

        let max_cakes = 6usize;
        let cake_h = (h / max_cakes).max(2);
        let count = (ctx.eased * max_cakes as f32).ceil() as usize;

        for i in 0..count.min(max_cakes) {
            let y0 = h.saturating_sub((i + 1) * cake_h);
            let y1 = h.saturating_sub(i * cake_h + 1);

            // Pancake ellipse: taller in center, tapers on sides.
            for dy in y0..=y1 {
                let t = if y1 <= y0 {
                    0.5
                } else {
                    (dy - y0) as f32 / (y1 - y0).max(1) as f32
                };
                let rel = 1.0 - (t * 2.0 - 1.0).powi(2); // 0→1→0 bulge
                let half = ((w as f32 / 2.0) * (0.6 + rel * 0.4)).round() as usize;
                let cx = w / 2;
                let px0 = cx.saturating_sub(half);
                let px1 = (cx + half).min(w - 1);
                draw::hline(grid, px0, px1, dy);
            }

            // Syrup drip: wiggly vertical line on right side, time-animated.
            let drip_x = w * 3 / 4;
            let drip_start = y0;
            let drip_end = y1 + 1;
            for dy in drip_start..drip_end {
                let sine = (ctx.time * 4.0 + dy as f32 * 0.5).sin() * 1.5;
                let dx = drip_x as i32 + sine as i32;
                draw::dot_i(grid, dx, dy as i32);
            }
        }

        // Top animation: a last pancake floating down if mid-progress.
        let frac = (ctx.eased * max_cakes as f32).fract();
        if frac > 0.0 && frac < 1.0 && count > 0 {
            let stack_top = h.saturating_sub(count * cake_h);
            let fall_offset = ((1.0 - frac) * (cake_h as f32 * 2.0)) as usize;
            let landing_y = stack_top.saturating_sub(fall_offset.min(h));
            let half = w / 3;
            let cx = w / 2;
            draw::hline(
                grid,
                cx.saturating_sub(half),
                (cx + half).min(w - 1),
                landing_y,
            );
        }

        // Palette tint.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let color = ctx.palette.sample(cy as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Candy jar filling with jellybeans.
struct CandyJar;
impl ProgressStyle for CandyJar {
    fn name(&self) -> &str {
        "candy-jar"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "A glass candy jar fills with jelly beans whose colors shift across the palette"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        // Jar outline: wide body, narrow lid.
        let left = 1usize;
        let right = w.saturating_sub(2);
        let lid_h = (h / 5).max(1);
        let body_h = h.saturating_sub(lid_h);

        // Lid.
        draw::hline(grid, left + 1, right.saturating_sub(1), 0);
        draw::hline(grid, left, right, lid_h);
        draw::vline(grid, left + 1, 0, lid_h);
        draw::vline(grid, right.saturating_sub(1), 0, lid_h);

        // Jar body.
        draw::vline(grid, left, lid_h, h - 1);
        draw::vline(grid, right, lid_h, h - 1);
        draw::hline(grid, left, right, h - 1);

        // Fill beans: stack from the bottom.
        let inner_h = body_h.saturating_sub(2).max(1);
        let _inner_w = right.saturating_sub(left + 1).max(1);
        let fill_h = (ctx.eased * inner_h as f32).round() as usize;

        if fill_h > 0 {
            let fill_top = h.saturating_sub(1).saturating_sub(fill_h);
            // Individual beans: 2-dot oval shapes in a pseudo-random grid.
            let bean_rows = fill_h / 3;
            for row in 0..bean_rows {
                let by = fill_top + row * 3;
                let row_offset = if row % 2 == 0 { 0usize } else { 2 };
                let mut bx = left + 1 + row_offset;
                let mut bi = 0u32;
                while bx + 2 < right {
                    // Each bean is a 2-wide dot pair.
                    let jitter = (hashf(bi + row as u32 * 37) * 1.0) as i32;
                    draw::dot_i(grid, bx as i32, by as i32 + jitter);
                    draw::dot_i(grid, bx as i32 + 1, by as i32 + jitter);
                    draw::dot_i(grid, bx as i32, by as i32 + jitter + 1);
                    bx += 4;
                    bi += 1;
                }
            }
        }

        // Palette tint for color variety on the beans.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.max(1) as f32; // invert so colors change row by row
            let color = ctx.palette.sample((t + ctx.time * 0.05).fract());
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Noodle being slurped: a wiggly spaghetti strand shortens from right to left.
struct NoodleSlurp;
impl ProgressStyle for NoodleSlurp {
    fn name(&self) -> &str {
        "noodle-slurp"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "A spaghetti strand wiggles and shortens from the right as it is slurped up"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 2 {
            return Ok(());
        }

        let mid_y = h / 2;
        // Remaining noodle length shrinks as eased rises.
        let noodle_len = ((1.0 - ctx.eased) * w as f32).round() as usize;

        if noodle_len > 0 {
            // Draw noodle from left edge to noodle_len with time-driven sine wiggle.
            let strand_count = 3usize;
            for s in 0..strand_count {
                let y_base = mid_y.saturating_sub(s);
                for px in 0..noodle_len.min(w) {
                    // Phase and frequency per strand for variety.
                    let freq = 0.4 + s as f32 * 0.15;
                    let speed = 3.0 + s as f32 * 1.5;
                    let phase = s as f32 * PI / strand_count as f32;
                    let wave = ((px as f32 * freq + ctx.time * speed + phase).sin()
                        * (h as f32 * 0.2)) as i32;
                    draw::dot_i(grid, px as i32, y_base as i32 + wave);
                }
            }

            // "Mouth" at the leading edge: a small vertical bite marker.
            let mouth_x = noodle_len.min(w.saturating_sub(1));
            draw::vline(
                grid,
                mouth_x,
                mid_y.saturating_sub(1),
                (mid_y + 1).min(h - 1),
            );
        }

        // Sauce splatter near origin: dots scattered around x=0 with time jitter.
        for i in 0..4u32 {
            let period = 0.5 + hashf(i + 500) * 0.5;
            let t = (ctx.time / period + hashf(i + 600)).fract();
            if t < 0.15 {
                let sx = (hashf(i + 700) * 5.0) as i32;
                let sy = mid_y as i32 + (hashf(i + 800) * 4.0) as i32 - 2;
                draw::dot_i(grid, sx, sy);
            }
        }

        // Palette tint.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let color = ctx.palette.sample(cy as f32 / ch.max(1) as f32);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Toast browning in a toaster: color darkens from golden to dark brown via palette.
struct ToastBrowning;
impl ProgressStyle for ToastBrowning {
    fn name(&self) -> &str {
        "toast-browning"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Toast darkens from pale gold to deep brown inside a toaster; the toast pops up at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        // Toaster body: outer rectangle.
        draw::rect_outline(grid, 0, h / 3, w, h.saturating_sub(h / 3));

        // Slots in the toaster top.
        let slot_w = (w / 3).max(2);
        let slot_x1 = w / 4;
        let slot_x2 = w * 3 / 4;
        draw::hline(
            grid,
            slot_x1,
            slot_x1 + slot_w.min(w.saturating_sub(slot_x1)),
            h / 3,
        );
        draw::hline(
            grid,
            slot_x2.saturating_sub(slot_w / 2),
            slot_x2 + slot_w / 2,
            h / 3,
        );

        // Toast rectangles: two slices popping up.
        let pop_h = if ctx.eased >= 0.99 {
            // Full pop: toast rises above the toaster.
            h / 3 + 1
        } else {
            // Toast is inside: visible height = eased * slot depth.
            let slot_depth = h / 3;
            (ctx.eased * slot_depth as f32).round() as usize
        };

        // Left slice.
        let tx1 = slot_x1 + 1;
        let tw1 = slot_w.saturating_sub(2).max(1);
        if pop_h > 0 {
            let toast_top = h / 3 + 1;
            let toast_bot = (toast_top + tw1).min(h - 1);
            // Draw toast shape: rectangle.
            draw::fill_rect(
                grid,
                tx1,
                toast_top.saturating_sub(pop_h),
                tw1,
                pop_h.min(toast_bot.saturating_sub(toast_top) + 1),
            );
        }

        // Right slice (mirrored).
        let tx2 = slot_x2.saturating_sub(slot_w / 2) + 1;
        if pop_h > 0 {
            let toast_top = h / 3 + 1;
            draw::fill_rect(
                grid,
                tx2,
                toast_top.saturating_sub(pop_h),
                slot_w.saturating_sub(2).max(1),
                pop_h,
            );
        }

        // Palette tint darkens as progress increases — lighter at start, darker at end.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            // Color the toast darker based on progress.
            let brown_t = ctx.eased;
            let color = ctx.palette.sample(brown_t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pie chart / progress: radial sweep fills a circular pie from 0→2π.
struct PieSweep;
impl ProgressStyle for PieSweep {
    fn name(&self) -> &str {
        "pie-sweep"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "A pie dish fills clockwise from 0° to 360°; a pulsing crust ring surrounds the fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }

        let cx = w / 2;
        let cy = h / 2;
        let r = (w.min(h) / 2).saturating_sub(1).max(1);

        let filled_angle = ctx.eased * 2.0 * PI;

        for dy in 0..h {
            for dx in 0..w {
                let fx = dx as f32 - cx as f32;
                let fy = dy as f32 - cy as f32;
                let dist = (fx * fx + fy * fy).sqrt();
                if dist > r as f32 {
                    continue;
                }

                // Angle measured clockwise from top (12-o'clock).
                let mut angle = fy.atan2(fx) + PI / 2.0;
                if angle < 0.0 {
                    angle += 2.0 * PI;
                }

                if angle <= filled_angle {
                    draw::dot(grid, dx, dy);
                }
            }
        }

        // Crust: pulsing ring that shimmers with time.
        let pulse = 0.5 + 0.5 * (ctx.time * 2.0 * PI * 0.5).sin();
        let crust_r = r as f32 + 0.5 + pulse * 0.5;
        for dy in 0..h {
            for dx in 0..w {
                let fx = dx as f32 - cx as f32;
                let fy = dy as f32 - cy as f32;
                let dist = (fx * fx + fy * fy).sqrt();
                if (dist - crust_r).abs() < 1.0 {
                    draw::dot(grid, dx, dy);
                }
            }
        }

        // Palette tint radiating from center.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// All styles in the `food` theme.
///
/// Returns one boxed [`ProgressStyle`] per food/kitchen bar. Call this to get
/// the full set for display in a gallery or picker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(BeerGlass),
        Box::new(CoffeePour),
        Box::new(PizzaSlices),
        Box::new(BoilingPot),
        Box::new(SushiConveyor),
        Box::new(PopcornPopping),
        Box::new(EggTimer),
        Box::new(PancakeStack),
        Box::new(CandyJar),
        Box::new(NoodleSlurp),
        Box::new(ToastBrowning),
        Box::new(PieSweep),
    ]
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::food::styles();
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
