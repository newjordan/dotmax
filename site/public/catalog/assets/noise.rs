//! `noise` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O noise.rs && ./noise [style-name]
//! ```

const DEFAULT_STYLE: &str = "value-noise-fill";

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
    pub mod noise {
//! Procedural-noise and flow-field progress bars.
//!
//! Every style is driven by **real noise algorithms** written from scratch —
//! value noise, fBm, Perlin-style gradient noise, domain warping, Worley /
//! Voronoi cellular noise, DLA crystal growth, curl noise, topographic
//! contour bands, plasma, Brownian trails, and a flow-field particle streamer.
//!
//! All algorithms are deterministic given `(ctx.progress, ctx.time)`.
//! Per-frame cost is bounded to ≤ ~3 000 evaluations in the worst case so the
//! bars stay fast even on large grids.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─── deterministic hash ──────────────────────────────────────────────────────

/// Fast integer hash → `[0, 1)`.  Used by every noise algorithm below.
#[inline]
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) % 1000) as f32 / 1000.0
}

/// 3-D variant: hash `(x, y, z_int)` — useful for animating with a time slot.
#[inline]
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash2(x ^ z.wrapping_mul(1_234_567), y ^ z.wrapping_mul(7_654_321))
}

// ─── smoothstep ──────────────────────────────────────────────────────────────

#[inline]
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// ─── value noise ─────────────────────────────────────────────────────────────

/// Bilinear value noise at `(x, y)`.  Hash the four surrounding lattice
/// corners and interpolate with smoothstep.  Pure, no state. Kept as a
/// static building block for anyone extracting this file (the bundled styles
/// use the time-animated [`value_noise_t`] variant).
#[allow(dead_code)]
fn value_noise(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let fx = smoothstep(x - xi as f32);
    let fy = smoothstep(y - yi as f32);
    let v00 = hash2(xi, yi);
    let v10 = hash2(xi + 1, yi);
    let v01 = hash2(xi, yi + 1);
    let v11 = hash2(xi + 1, yi + 1);
    lerp(lerp(v00, v10, fx), lerp(v01, v11, fx), fy)
}

/// Animated value noise: treat `time` as a third dimension by linearly blending
/// between two lattice slabs `floor(t)` and `floor(t)+1`.
fn value_noise_t(x: f32, y: f32, t: f32) -> f32 {
    let ti = t.floor() as i32;
    let ft = smoothstep(t - ti as f32);

    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let fx = smoothstep(x - xi as f32);
    let fy = smoothstep(y - yi as f32);

    let slab = |tz: i32| -> f32 {
        let v00 = hash3(xi, yi, tz);
        let v10 = hash3(xi + 1, yi, tz);
        let v01 = hash3(xi, yi + 1, tz);
        let v11 = hash3(xi + 1, yi + 1, tz);
        lerp(lerp(v00, v10, fx), lerp(v01, v11, fx), fy)
    };
    lerp(slab(ti), slab(ti + 1), ft)
}

// ─── fBm (fractal Brownian motion) ───────────────────────────────────────────

/// Sum `octaves` layers of animated value noise with lacunarity 2 / gain 0.5.
fn fbm(x: f32, y: f32, t: f32, octaves: usize) -> f32 {
    let mut val = 0.0f32;
    let mut amp = 0.5f32;
    let mut freq = 1.0f32;
    let mut norm = 0.0f32;
    for _ in 0..octaves.max(1) {
        val += amp * value_noise_t(x * freq, y * freq, t * freq);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    val / norm
}

// ─── Perlin-style gradient noise ─────────────────────────────────────────────

/// Unit gradient vector from a hash, mapped to 8 cardinal / diagonal dirs.
#[inline]
fn grad_vec(ix: i32, iy: i32) -> (f32, f32) {
    let h = (hash2(ix, iy) * 8.0) as u32 % 8;
    match h {
        0 => (1.0, 0.0),
        1 => (-1.0, 0.0),
        2 => (0.0, 1.0),
        3 => (0.0, -1.0),
        4 => (0.707, 0.707),
        5 => (-0.707, 0.707),
        6 => (0.707, -0.707),
        _ => (-0.707, -0.707),
    }
}

/// Perlin-style gradient noise at `(x, y)`.
fn gradient_noise(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let fx = x - xi as f32;
    let fy = y - yi as f32;
    let ux = smoothstep(fx);
    let uy = smoothstep(fy);

    let dot = |ix: i32, iy: i32, dx: f32, dy: f32| -> f32 {
        let (gx, gy) = grad_vec(ix, iy);
        gx * dx + gy * dy
    };

    let n00 = dot(xi, yi, fx, fy);
    let n10 = dot(xi + 1, yi, fx - 1.0, fy);
    let n01 = dot(xi, yi + 1, fx, fy - 1.0);
    let n11 = dot(xi + 1, yi + 1, fx - 1.0, fy - 1.0);

    // Perlin output is [-0.7, 0.7] → remap to [0, 1].
    (lerp(lerp(n00, n10, ux), lerp(n01, n11, ux), uy) + 0.7) / 1.4
}

// ─── Worley / cellular noise ─────────────────────────────────────────────────

/// Distance to the nearest of several hash-placed feature points in the cell
/// `(xi, yi)` and its 8 neighbours.  Returns (F1, F2).
fn worley(x: f32, y: f32, seed_density: u32) -> (f32, f32) {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let mut f1 = f32::MAX;
    let mut f2 = f32::MAX;
    for cy in (yi - 1)..=(yi + 1) {
        for cx in (xi - 1)..=(xi + 1) {
            for k in 0..seed_density {
                let px = cx as f32 + hash2(cx * 7 + k as i32, cy * 13);
                let py = cy as f32 + hash2(cx * 11, cy * 17 + k as i32);
                let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                if d < f1 {
                    f2 = f1;
                    f1 = d;
                } else if d < f2 {
                    f2 = d;
                }
            }
        }
    }
    (f1, f2)
}

// ─── registry ────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — analog static blue-gray.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(188, 196, 214);
const TINT_END: Color = Color::rgb(96, 116, 156);

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

/// All styles in the `noise` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(ValueNoiseFill)),
        Box::new(Tinted(FbmContour)),
        Box::new(Tinted(GradientNoiseFill)),
        Box::new(Tinted(DomainWarp)),
        Box::new(Tinted(FlowField)),
        Box::new(Tinted(WorleyCell)),
        Box::new(Tinted(VoronoiDiagram)),
        Box::new(Tinted(DlaGrowth)),
        Box::new(Tinted(BrownianTrail)),
        Box::new(Tinted(CurlNoise)),
        Box::new(Tinted(TopoContour)),
        Box::new(Tinted(Plasma)),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. VALUE NOISE FILL
// ═══════════════════════════════════════════════════════════════════════════

/// Animated bilinear value noise thresholded by eased progress.
struct ValueNoiseFill;
impl ProgressStyle for ValueNoiseFill {
    fn name(&self) -> &str {
        "value-noise-fill"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Bilinear value noise lattice thresholded by progress — blobs of filled \
         dots that slowly churn and spread as progress grows."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Scale: zoom out enough that the field looks interesting even at 1-cell height.
        let scale_x = 4.0 / dw as f32;
        let scale_y = 4.0 / dh.max(1) as f32;
        let t = ctx.time * 0.3;
        let threshold = ctx.eased;
        // Hard threshold left of fill marker; animate everywhere.
        let fill_x = (ctx.eased * dw as f32) as usize;
        for dy in 0..dh {
            for dx in 0..dw {
                // Left of the fill boundary: always lit if noise < threshold.
                let n = value_noise_t(dx as f32 * scale_x, dy as f32 * scale_y, t);
                if dx < fill_x {
                    // Inside fill: draw where noise is above a moving wavefront.
                    if n > 0.3 - threshold * 0.2 {
                        draw::dot(grid, dx, dy);
                    }
                } else if dx == fill_x && n > 0.5 {
                    // Leading edge sparkle.
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Always draw full bottom edge as baseline.
        draw::hline(grid, 0, fill_x.saturating_sub(1), dh - 1);
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. fBm CONTOUR FILL
// ═══════════════════════════════════════════════════════════════════════════

/// fBm with octave count driven by progress — the detail level visibly grows.
struct FbmContour;
impl ProgressStyle for FbmContour {
    fn name(&self) -> &str {
        "fbm-contour"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Fractal Brownian motion fill: octave count rises with progress, so the \
         bar starts smooth and gains turbulent detail as it nears 100%."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Octave count: 1 at 0%, 6 at 100%.
        let octaves = (1 + (ctx.eased * 5.0).floor() as usize).min(6);
        let scale = 3.0 / dw as f32;
        let t = ctx.time * 0.25;
        let fill_x = (ctx.eased * dw as f32) as usize;
        // Bias threshold so that ≈50% of the noise field is lit at full progress.
        let threshold = 0.45 - ctx.eased * 0.1;
        for dy in 0..dh {
            for dx in 0..fill_x.min(dw) {
                let n = fbm(dx as f32 * scale, dy as f32 * scale, t, octaves);
                if n > threshold {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Solid baseline so the bar reads as a bar even at low progress.
        if dh > 0 {
            draw::hline(grid, 0, fill_x.saturating_sub(1), dh - 1);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. GRADIENT (PERLIN-STYLE) NOISE FILL
// ═══════════════════════════════════════════════════════════════════════════

/// Perlin-style gradient noise: smoother, more organic than value noise.
struct GradientNoiseFill;
impl ProgressStyle for GradientNoiseFill {
    fn name(&self) -> &str {
        "gradient-noise-fill"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Perlin-style gradient noise thresholded by progress — smoother, more \
         organic blobs than value noise, with a shimmering animated field."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let scale_x = 5.0 / dw as f32;
        let scale_y = 3.0 / dh.max(1) as f32;
        // Animate by slowly shifting the sample coordinates.
        let tx = ctx.time * 0.2;
        let ty = ctx.time * 0.15;
        let fill_x = (ctx.eased * dw as f32) as usize;
        for dy in 0..dh {
            for dx in 0..fill_x.min(dw) {
                let n = gradient_noise(dx as f32 * scale_x + tx, dy as f32 * scale_y + ty);
                if n > 0.35 {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        if dh > 0 {
            draw::hline(grid, 0, fill_x.saturating_sub(1), dh - 1);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. DOMAIN WARPING
// ═══════════════════════════════════════════════════════════════════════════

/// Domain warping: sample fBm at `(x + fbm(x,y), y + fbm(x+5,y+5))`.
/// Creates hypnotic swirls that evolve with time.
struct DomainWarp;
impl ProgressStyle for DomainWarp {
    fn name(&self) -> &str {
        "domain-warp"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Domain warping — sampling fBm at displaced coordinates — produces \
         hypnotic, swirling tendrils that churn and grow with progress."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let scale = 3.0 / dw as f32;
        let sy = 3.0 / dh.max(1) as f32;
        let t = ctx.time * 0.2;
        let fill_x = (ctx.eased * dw as f32) as usize;
        // Warp strength grows with progress.
        let warp = ctx.eased * 1.5;
        let octaves = 3;
        for dy in 0..dh {
            for dx in 0..fill_x.min(dw) {
                let sx = dx as f32 * scale;
                let sy2 = dy as f32 * sy;
                // First warp pass.
                let qx = fbm(sx, sy2, t, octaves);
                let qy = fbm(sx + 5.2, sy2 + 1.3, t, octaves);
                // Second warp pass with offset.
                let n = fbm(sx + warp * qx, sy2 + warp * qy, t, octaves);
                if n > 0.42 {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        if dh > 0 {
            draw::hline(grid, 0, fill_x.saturating_sub(1), dh - 1);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. FLOW FIELD PARTICLE STREAKS
// ═══════════════════════════════════════════════════════════════════════════

/// Flow field: angle = noise(x, y, t)·2π, particles are advected and drawn.
/// Number of active particles scales with progress.
struct FlowField;
impl ProgressStyle for FlowField {
    fn name(&self) -> &str {
        "flow-field"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Flow-field particles: each particle is advected along a noise-derived \
         angle field, leaving a streak. Particle count grows with progress."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let max_particles = 48usize;
        let n_particles = ((ctx.eased * max_particles as f32) as usize)
            .max(1)
            .min(max_particles);
        let streak_len = 12usize;
        let scale = 4.0 / dw as f32;
        let sy = 4.0 / dh.max(1) as f32;
        let t = ctx.time * 0.4;
        for p in 0..n_particles {
            // Deterministic start position from hash.
            let mut px = hash2(p as i32 * 7, 3) * dw as f32;
            let mut py = hash2(p as i32 * 11, 17) * dh as f32;
            // Offset start by time so particles continually respawn.
            let phase = hash2(p as i32, 99) * 10.0;
            let tp = (t + phase).fract();
            px = (px + tp * dw as f32 * 0.3) % dw as f32;
            for _ in 0..streak_len {
                draw::dot_i(grid, px as i32, py as i32);
                // Angle from noise field.
                let angle = value_noise_t(px * scale, py * sy, t) * 2.0 * PI;
                let step = 1.2f32;
                px += angle.cos() * step;
                py += angle.sin() * step * 0.5;
                // Wrap within grid.
                if px < 0.0 {
                    px += dw as f32;
                }
                if py < 0.0 {
                    py += dh as f32;
                }
                if px >= dw as f32 {
                    px -= dw as f32;
                }
                if py >= dh as f32 {
                    py -= dh as f32;
                }
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. WORLEY / CELLULAR NOISE
// ═══════════════════════════════════════════════════════════════════════════

/// Worley cellular noise: shade by F1 distance, reveal cells left of fill edge.
struct WorleyCell;
impl ProgressStyle for WorleyCell {
    fn name(&self) -> &str {
        "worley-cell"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Worley cellular noise — distance to nearest feature point — fills with \
         shimmering cell-membrane patterns, edges sharpening with progress."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let fill_x = (ctx.eased * dw as f32) as usize;
        let scale = 4.0 / dw as f32;
        let sy = 3.0 / dh.max(1) as f32;
        // Animate feature points slowly.
        let t_offset = ctx.time * 0.15;
        let seeds = 2u32;
        // Threshold: draw dot when F2-F1 is small (near cell edges) or F1 is small.
        for dy in 0..dh {
            for dx in 0..fill_x.min(dw) {
                let fx = dx as f32 * scale + t_offset * 0.1;
                let fy = dy as f32 * sy;
                let (f1, f2) = worley(fx, fy, seeds);
                // Cell edge (F2-F1 near 0) or inside small cells (F1 small).
                let edge = f2 - f1;
                if edge < 0.08 || f1 < 0.12 {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        if dh > 0 {
            draw::hline(grid, 0, fill_x.saturating_sub(1), dh - 1);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. VORONOI DIAGRAM
// ═══════════════════════════════════════════════════════════════════════════

/// Voronoi diagram: seeds grow with progress, draw nearest-seed boundaries.
struct VoronoiDiagram;
impl ProgressStyle for VoronoiDiagram {
    fn name(&self) -> &str {
        "voronoi-diagram"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Voronoi diagram with seed count growing with progress — cell boundaries \
         shatter and multiply, revealing a cracked-glass pattern."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let max_seeds = 24usize;
        let n_seeds = ((ctx.eased * max_seeds as f32) as usize)
            .max(1)
            .min(max_seeds);
        // Pre-compute seed positions in dot-space.
        let mut seeds: Vec<(f32, f32)> = Vec::with_capacity(n_seeds);
        let t_drift = ctx.time * 0.08;
        for i in 0..n_seeds {
            let bx = hash2(i as i32 * 3, 1) * dw as f32;
            let by = hash2(i as i32 * 7, 2) * dh as f32;
            // Seeds drift slowly.
            let dx2 = (hash2(i as i32, 42) - 0.5) * t_drift * dw as f32 * 0.1;
            let dy2 = (hash2(i as i32 + 100, 42) - 0.5) * t_drift * dh as f32 * 0.1;
            let sx = (bx + dx2).rem_euclid(dw as f32);
            let sy = (by + dy2).rem_euclid(dh.max(1) as f32);
            seeds.push((sx, sy));
        }
        // For each dot, find nearest seed index and second-nearest; draw edge if different.
        // Cost: dw*dh*n_seeds — cap to keep within budget.
        let budget_dw = dw.min(80);
        let budget_dh = dh.min(16);
        for dy in 0..budget_dh {
            for dx in 0..budget_dw {
                let mut d1 = f32::MAX;
                let mut d2 = f32::MAX;
                let mut id1 = 0usize;
                for (si, &(sx, sy)) in seeds.iter().enumerate() {
                    let d = ((dx as f32 - sx).powi(2) + (dy as f32 - sy).powi(2)).sqrt();
                    if d < d1 {
                        d2 = d1;
                        d1 = d;
                        id1 = si;
                    } else if d < d2 {
                        d2 = d;
                    }
                }
                // Boundary: d2-d1 very small → on an edge between cells.
                // Color by parity of nearest-seed id.
                let _ = id1; // used implicitly for the edge condition
                if d2 - d1 < 1.5 {
                    draw::dot(grid, dx, dy);
                } else if id1 % 2 == 0 && d1 < 3.0 {
                    // Dot near every other seed centre.
                    draw::dot(grid, dx, dy);
                }
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. DLA (DIFFUSION-LIMITED AGGREGATION) CRYSTAL
// ═══════════════════════════════════════════════════════════════════════════

/// Deterministic DLA approximation: grow a crystal cluster whose size is
/// controlled by `eased`.  Implemented as a space-filling hash walk so it's
/// O(N) with no random-number state.
struct DlaGrowth;
impl ProgressStyle for DlaGrowth {
    fn name(&self) -> &str {
        "dla-growth"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Deterministic DLA crystal: a branching, fern-like aggregate grows from \
         the centre outward as progress increases."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;
        let max_particles = 200usize;
        let n_alive = ((ctx.eased * max_particles as f32) as usize).min(max_particles);
        // Animate: time shifts particle walks so the crystal breathes.
        let t_int = (ctx.time * 1.5) as i32;
        // Each particle performs a short deterministic random walk from the
        // edge and "sticks" to the cluster approximated by a distance-from-centre
        // budget that grows with eased.
        let max_r = (ctx.eased * (dw.min(dh) as f32 * 0.45)).max(1.0);
        for p in 0..n_alive {
            // Deterministic start on the perimeter.
            let angle = hash2(p as i32, t_int) * 2.0 * PI;
            let mut px = cx + (angle.cos() * max_r * 1.2) as i32;
            let mut py = cy + (angle.sin() * max_r * 0.6) as i32;
            // Walk 20 steps toward centre with noise jitter.
            for step in 0..20i32 {
                let jitter_h = hash2(p as i32 * 31 + step, t_int ^ 0xABCD);
                let jitter_v = hash2(p as i32 * 17 + step + 1000, t_int ^ 0x1234);
                let dx2 = if jitter_h > 0.5 { 1i32 } else { -1 };
                let dy2 = if jitter_v > 0.5 { 1i32 } else { -1 };
                let bias_x = if px > cx { -1i32 } else { 1 };
                let bias_y = if py > cy { -1i32 } else { 1 };
                // Blend jitter with attraction.
                let blend_h = hash2(p as i32 + step * 3, 7777);
                let blend_v = hash2(p as i32 + step * 5, 8888);
                px += if blend_h > 0.4 { bias_x } else { dx2 };
                py += if blend_v > 0.4 { bias_y } else { dy2 };
                // Stick when close enough to centre.
                let r2 = ((px - cx).pow(2) + (py - cy).pow(2)) as f32;
                if r2 < max_r * max_r * 0.5 {
                    draw::dot_i(grid, px, py);
                    break;
                }
            }
            draw::dot_i(grid, px, py);
        }
        // Always show a seed dot at the centre.
        draw::dot_i(grid, cx, cy);
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. BROWNIAN TRAIL
// ═══════════════════════════════════════════════════════════════════════════

/// Hash-driven random walk trails, revealed left-to-right by eased progress.
struct BrownianTrail;
impl ProgressStyle for BrownianTrail {
    fn name(&self) -> &str {
        "brownian-trail"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Brownian random-walk trails: multiple walkers leave hash-driven paths \
         that are progressively revealed as the bar fills."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let n_walkers = 6usize;
        let steps_per_walker = (dw / n_walkers.max(1)).max(4);
        let reveal_x = (ctx.eased * dw as f32) as usize;
        // Animate: walkers shift with time.
        let t_seed = (ctx.time * 0.5) as i32;
        for w in 0..n_walkers {
            let mut x = (w * dw / n_walkers) as i32;
            let mut y = (dh / 2) as i32;
            for s in 0..steps_per_walker {
                let hx = hash2(w as i32 * 1000 + s as i32, t_seed);
                let hy = hash2(w as i32 * 2000 + s as i32, t_seed + 1);
                x += if hx > 0.5 { 1 } else { -1 };
                y += if hy > 0.66 {
                    1
                } else if hy < 0.33 {
                    -1
                } else {
                    0
                };
                if x >= 0 && (x as usize) < reveal_x {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Baseline.
        if dh > 0 {
            draw::hline(grid, 0, reveal_x.saturating_sub(1), dh - 1);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. CURL NOISE FIELD
// ═══════════════════════════════════════════════════════════════════════════

/// Curl noise: take the 2-D curl of a scalar noise potential to get a
/// divergence-free velocity field, then draw streamlines through it.
struct CurlNoise;
impl ProgressStyle for CurlNoise {
    fn name(&self) -> &str {
        "curl-noise"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Curl-noise streamlines: a divergence-free velocity field derived from \
         the gradient of value noise — swirling, never-crossing filaments."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let n_streams = 32usize;
        let steps = 20usize;
        let scale = 4.0 / dw as f32;
        let sy = 3.0 / dh.max(1) as f32;
        let t = ctx.time * 0.3;
        let reveal_x = (ctx.eased * dw as f32) as usize;
        let eps = 0.5f32;
        for s in 0..n_streams {
            // Seed point spread across the bar.
            let mut px = hash2(s as i32, 1) * dw as f32;
            let mut py = hash2(s as i32, 2) * dh as f32;
            for _ in 0..steps {
                // Numerical curl: d/dy(noise) in x, -d/dx(noise) in y.
                let n_y_plus = value_noise_t(px * scale, (py + eps) * sy, t);
                let n_y_minus = value_noise_t(px * scale, (py - eps) * sy, t);
                let n_x_plus = value_noise_t((px + eps) * scale, py * sy, t);
                let n_x_minus = value_noise_t((px - eps) * scale, py * sy, t);
                let curl_x = (n_y_plus - n_y_minus) / (2.0 * eps * sy);
                let curl_y = -(n_x_plus - n_x_minus) / (2.0 * eps * scale);
                // Normalise.
                let mag = (curl_x * curl_x + curl_y * curl_y).sqrt().max(0.001);
                let step = 1.5f32;
                px += curl_x / mag * step;
                py += curl_y / mag * step * 0.5;
                // Wrap.
                px = px.rem_euclid(dw as f32);
                py = py.rem_euclid(dh.max(1) as f32);
                if (px as usize) < reveal_x {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. TOPOGRAPHIC CONTOUR (fBm ISOLINES)
// ═══════════════════════════════════════════════════════════════════════════

/// Marching contour bands of fBm — looks like a topographic map.
/// Number of contour levels grows with progress.
struct TopoContour;
impl ProgressStyle for TopoContour {
    fn name(&self) -> &str {
        "topo-contour"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Topographic fBm isolines: contour bands of fractal Brownian motion \
         multiply with progress, building a richly detailed elevation map."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let n_levels = ((ctx.eased * 8.0) as usize).clamp(1, 8);
        let scale = 3.5 / dw as f32;
        let sy = 3.0 / dh.max(1) as f32;
        let t = ctx.time * 0.2;
        let octaves = 4;
        let fill_x = (ctx.eased * dw as f32) as usize;
        for dy in 0..dh {
            for dx in 0..fill_x.min(dw) {
                let n = fbm(
                    dx as f32 * scale,
                    dy as f32 * scale * sy / scale,
                    t,
                    octaves,
                );
                // Draw at contour bands: every 1/n_levels interval near an isoline.
                let band = (n * n_levels as f32).fract();
                // A dot is on the isoline if the band value is near 0 or 1.
                if !(0.12..=0.88).contains(&band) {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. PLASMA (CLASSIC DEMOSCENE)
// ═══════════════════════════════════════════════════════════════════════════

/// Classic demoscene plasma: sum of sines of x, y, distance, and time.
/// Threshold the result against eased progress to fill left-to-right.
struct Plasma;
impl ProgressStyle for Plasma {
    fn name(&self) -> &str {
        "plasma"
    }
    fn theme(&self) -> &str {
        "noise"
    }
    fn describe(&self) -> &str {
        "Classic demoscene plasma — superimposed sine waves of position, \
         distance, and time — thresholded into a pulsing psychedelic fill."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let t = ctx.time;
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let fill_x = (ctx.eased * dw as f32) as usize;
        // Threshold oscillates slightly with time for shimmer.
        let threshold = 0.4 + 0.08 * (t * 1.1).sin();
        for dy in 0..dh {
            for dx in 0..fill_x.min(dw) {
                let x = dx as f32 / dw as f32 * 8.0;
                let y = dy as f32 / dh.max(1) as f32 * 4.0;
                let dist = ((dx as f32 - cx).powi(2) + (dy as f32 - cy).powi(2)).sqrt();
                let v = 0.25 * (x + t).sin()
                    + 0.25 * (y + t * 0.7).sin()
                    + 0.25 * ((x + y) * 0.5 + t * 1.3).sin()
                    + 0.25 * (dist * 0.25 + t).sin();
                // v ∈ [-1, 1] → remap to [0, 1].
                let vn = (v + 1.0) * 0.5;
                if vn > threshold {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Baseline.
        if dh > 0 {
            draw::hline(grid, 0, fill_x.saturating_sub(1), dh - 1);
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
    let styles = progress::styles::noise::styles();
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
