//! `sports` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O sports.rs && ./sports [style-name]
//! ```

const DEFAULT_STYLE: &str = "sprint-100m";

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
    pub mod sports {
//! Sports-themed progress bars — eleven distinct athletic spectacles in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (perpetual animation)
//! and `ctx.eased` / `ctx.progress` (progress-driven advancement). All writes go
//! through `draw::` helpers so every pixel is silently bounds-safe.
//!
//! Styles in this file:
//! - `sprint-100m`      — runner advances right, legs cycle, finish tape snaps at 100%
//! - `basketball-arc`   — ball traces a parabola into a hoop, swish flash at 100%
//! - `soccer-goal`      — ball curves into a net, net bulges on score
//! - `swimming-laps`    — swimmer bobs across a lane, lap counter via vblocks
//! - `archery`          — bow draws back with eased, arrow flies to a bullseye
//! - `bowling`          — ball rolls down lane, pins topple progressively at end
//! - `darts`            — dart flies toward concentric scoring rings of a dartboard
//! - `high-jump`        — athlete arcs over a bar that rises with progress
//! - `weightlifting`    — barbell overhead, plate stack height = eased
//! - `tennis-rally`     — ball bounces between baselines, rally count as hbar fills
//! - `cycling-peloton`  — wheels spin, riders packed tight, gap eaten by progress

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — court orange into jersey red. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 152, 56);
const TINT_END: Color = Color::rgb(255, 84, 84);

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

/// All styles in the `sports` theme.
///
/// Returns one boxed [`ProgressStyle`] per sport variant, ready to be mixed
/// into a gallery or driven individually.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Sprint100m),
        Box::new(Tinted(BasketballArc)),
        Box::new(SoccerGoal),
        Box::new(Tinted(SwimmingLaps)),
        Box::new(Tinted(Archery)),
        Box::new(Bowling),
        Box::new(Tinted(Darts)),
        Box::new(Tinted(HighJump)),
        Box::new(Tinted(Weightlifting)),
        Box::new(TennisRally),
        Box::new(CyclingPeloton),
    ]
}

// ── Sprint 100m ───────────────────────────────────────────────────────────────

struct Sprint100m;
impl ProgressStyle for Sprint100m {
    fn name(&self) -> &str {
        "sprint-100m"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "100m sprinter advances right with cycling legs; finish-line tape snaps at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);
        let mid = h / 2;

        // Ground line.
        draw::hline(grid, 0, w.saturating_sub(1), base);

        // Finish line — a vertical column of dots at the right edge.
        let finish_x = w.saturating_sub(2);
        draw::vline(grid, finish_x, 0, base);

        // Runner horizontal position driven by progress.
        let runner_x = (ctx.eased * finish_x.saturating_sub(4) as f32) as usize;
        let runner_x = runner_x.min(finish_x.saturating_sub(4));

        // Leg cycle phase from time.
        let phase = ctx.time * 8.0;
        // Forward leg (right).
        let fwd = (phase.sin() * 1.5).round() as i32;
        // Back leg (left) is opposite phase.
        let bk = (-(phase.sin()) * 1.5).round() as i32;

        // Body: torso leaning forward.
        let torso_bot = base as i32 - 1;
        let torso_top = torso_bot - (h as i32 / 3).max(1);
        draw::dot_i(grid, runner_x as i32 + 1, torso_bot);
        draw::dot_i(grid, runner_x as i32 + 1, torso_bot - 1);
        draw::dot_i(grid, runner_x as i32 + 1, torso_top);
        // Head.
        draw::dot_i(grid, runner_x as i32 + 2, torso_top - 1);

        // Arms (opposite to legs).
        let arm_phase = -phase;
        let arm_fwd = (arm_phase.sin() * 1.2).round() as i32;
        draw::dot_i(grid, runner_x as i32 + 2, torso_bot - 1 + arm_fwd);
        draw::dot_i(grid, runner_x as i32, torso_bot - 1 - arm_fwd);

        // Legs.
        draw::dot_i(grid, runner_x as i32 + 2, torso_bot + fwd);
        draw::dot_i(grid, runner_x as i32, torso_bot + bk);
        // Feet.
        draw::dot_i(grid, runner_x as i32 + 3, base as i32 + fwd.min(0));
        draw::dot_i(grid, runner_x as i32 - 1, base as i32 + bk.min(0));

        // Tape break flash when at 100%.
        if ctx.progress >= 0.999 {
            // Horizontal tape across mid of bar at the finish line.
            draw::hline(grid, finish_x.saturating_sub(1), w.saturating_sub(1), mid);
            // Torn fragments: a few scattered dots to the right.
            for k in 0..3usize {
                let fx = finish_x + k * 2;
                let fy = (mid as i32 + (k as i32 % 3) - 1).max(0) as usize;
                draw::dot(
                    grid,
                    fx.min(w.saturating_sub(1)),
                    fy.min(h.saturating_sub(1)),
                );
            }
        }

        // Tint the swept lane.
        let (cw, ch) = grid.dimensions();
        let swept = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            for cx in 0..swept.min(cw) {
                let t = if swept <= 1 {
                    0.0
                } else {
                    cx as f32 / (swept - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Basketball Arc ────────────────────────────────────────────────────────────

struct BasketballArc;
impl ProgressStyle for BasketballArc {
    fn name(&self) -> &str {
        "basketball-arc"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Basketball traces a parabolic arc toward a hoop; swish flash lights up at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);

        // Hoop: two dots at top-right, a small horizontal bar.
        let hoop_x = w.saturating_sub(3);
        let hoop_y = h / 4;
        draw::hline(grid, hoop_x, w.saturating_sub(1), hoop_y);
        // Net: vertical lines below hoop.
        let net_h = (h / 4).max(1);
        for nx in [hoop_x, w.saturating_sub(1)] {
            draw::vline(
                grid,
                nx.min(w.saturating_sub(1)),
                hoop_y,
                (hoop_y + net_h).min(base),
            );
        }
        // Net cross-strands (every 2 dots).
        for ny in (hoop_y..=(hoop_y + net_h).min(base)).step_by(2) {
            draw::hline(grid, hoop_x, w.saturating_sub(1), ny.min(base));
        }

        // Ball trajectory: parabola from bottom-left to hoop.
        // t drives the shot from 0 (ball in hand) → 1 (in hoop).
        // We animate t using progress; time adds a spin bobble at rest.
        let t = ctx.eased;

        // Parametric: x linear, y parabolic (peak at mid-arc).
        let bx = (t * hoop_x as f32) as i32;
        // Parabola that starts at base and ends at hoop_y.
        // y(t) = base - t*base + peak * 4t(1-t), where peak brings it above hoop.
        let start_y = base as f32;
        let end_y = hoop_y as f32;
        let peak_lift = (h as f32 * 0.6).min((base - hoop_y) as f32 + h as f32 * 0.4);
        let linear_y = start_y + t * (end_y - start_y);
        let arc_y = linear_y - peak_lift * 4.0 * t * (1.0 - t);
        let by = arc_y.round().clamp(0.0, base as f32) as i32;

        // Ball (2-dot body, slightly round via 4 dots).
        draw::dot_i(grid, bx, by);
        draw::dot_i(grid, bx + 1, by);
        draw::dot_i(grid, bx, by + 1);
        draw::dot_i(grid, bx + 1, by + 1);

        // Swish flash at 100%: fill net bright.
        if ctx.progress >= 0.999 {
            for ny in hoop_y..=(hoop_y + net_h).min(base) {
                draw::hline(grid, hoop_x, w.saturating_sub(1), ny);
            }
        }

        // Faint arc trail (dotted line of the parabola).
        for step in 0..20usize {
            let pt = step as f32 / 20.0;
            if pt >= t {
                break;
            }
            let tx = (pt * hoop_x as f32) as i32;
            let ty_lin = start_y + pt * (end_y - start_y);
            let ty = (ty_lin - peak_lift * 4.0 * pt * (1.0 - pt))
                .round()
                .clamp(0.0, base as f32) as i32;
            if step % 2 == 0 {
                draw::dot_i(grid, tx, ty);
            }
        }

        Ok(())
    }
}

// ── Soccer Goal ───────────────────────────────────────────────────────────────

struct SoccerGoal;
impl ProgressStyle for SoccerGoal {
    fn name(&self) -> &str {
        "soccer-goal"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Soccer ball curves into a goal net; net bulges on score"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);

        // Goal frame: right side of bar.
        let goal_left = w.saturating_sub((w / 5).max(3));
        let goal_top = 0usize;
        // Crossbar.
        draw::hline(grid, goal_left, w.saturating_sub(1), goal_top);
        // Posts.
        draw::vline(grid, goal_left, goal_top, base);
        draw::vline(grid, w.saturating_sub(1), goal_top, base);

        // Net: diagonal grid inside the goal.
        let net_bulge: i32 = if ctx.progress >= 0.999 {
            ((ctx.time * 10.0).sin() * 1.5).round() as i32
        } else {
            0
        };
        for ny in (goal_top..=base).step_by(2) {
            for nx in (goal_left..w).step_by(3) {
                let bx = nx as i32 + if ny % 4 == 0 { net_bulge } else { -net_bulge };
                draw::dot_i(grid, bx, ny as i32);
            }
        }

        // Ball curve: starts bottom-left, curves into goal mouth.
        let t = ctx.eased;
        // Horizontal: 0 → goal_left.
        let bx = (t * goal_left as f32) as i32;
        // Vertical: starts at mid, curves up then down to goal mid.
        let start_y = (h as f32 * 0.7) as i32;
        let end_y = (h / 2) as i32;
        let curve_peak = ((h as f32 * 0.25) * (PI * t).sin()) as i32;
        let by =
            (start_y + (t * (end_y - start_y) as f32) as i32 - curve_peak).clamp(0, base as i32);

        // Ball: pentagon-ish 3x3 pattern.
        draw::dot_i(grid, bx, by);
        draw::dot_i(grid, bx + 1, by);
        draw::dot_i(grid, bx, by + 1);
        draw::dot_i(grid, bx + 1, by + 1);
        // Pentagons: alternating black dots on ball.
        if (ctx.time * 5.0) as usize % 2 == 0 {
            draw::dot_i(grid, bx + 1, by - 1);
        } else {
            draw::dot_i(grid, bx - 1, by + 1);
        }

        // Field: grass line at base.
        for gx in (0..w).step_by(4) {
            draw::dot(grid, gx.min(w.saturating_sub(1)), base);
            if gx + 1 < w {
                draw::dot(grid, gx + 1, base.saturating_sub(1));
            }
        }

        // Tint the arc path.
        let (cw, ch) = grid.dimensions();
        let swept = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            for cx in 0..swept.min(cw) {
                let t2 = if swept <= 1 {
                    0.0
                } else {
                    cx as f32 / (swept - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t2));
            }
        }
        Ok(())
    }
}

// ── Swimming Laps ─────────────────────────────────────────────────────────────

struct SwimmingLaps;
impl ProgressStyle for SwimmingLaps {
    fn name(&self) -> &str {
        "swimming-laps"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Swimmer bobs across a lane; vblock lap counter fills on the right as laps complete"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cw, ch) = grid.dimensions();
        let mid = h / 2;

        // Lane boundaries: top and bottom dots.
        draw::hline(grid, 0, w.saturating_sub(1), 0);
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        // Lane rope: dashed middle.
        let rope_y = mid;
        for rx in (0..w).step_by(3) {
            draw::dot(grid, rx.min(w.saturating_sub(1)), rope_y);
        }

        // Swimmer position: goes left-to-right then right-to-left on lap 2.
        // Number of laps = 4 total. Each lap = 0.25 of progress.
        let n_laps = 4usize;
        let lap_frac_f = ctx.eased * n_laps as f32;
        let current_lap = (lap_frac_f as usize).min(n_laps.saturating_sub(1));
        let within_lap = lap_frac_f.fract();

        // Alternate direction each lap.
        let going_right = current_lap % 2 == 0;
        let swim_x = if going_right {
            (within_lap * w as f32) as usize
        } else {
            w.saturating_sub(1)
                .saturating_sub((within_lap * w as f32) as usize)
        };
        let swim_x = swim_x.min(w.saturating_sub(3));

        // Vertical bob from time + small correction for which half of lane.
        let bob_amp = ((h / 4).max(1)) as f32 * 0.5;
        let lane_y = if going_right {
            mid.saturating_sub(2)
        } else {
            (mid + 2).min(h.saturating_sub(2))
        };
        let bob = (ctx.time * 6.0).sin() * bob_amp;
        let sy = (lane_y as f32 + bob)
            .round()
            .clamp(1.0, h.saturating_sub(2) as f32) as usize;

        // Swimmer body.
        draw::dot(grid, swim_x, sy);
        draw::dot(grid, swim_x + 1, sy);
        // Head.
        let head_off: i32 = if going_right { 2 } else { -1 };
        draw::dot_i(grid, swim_x as i32 + head_off, sy as i32 - 1);

        // Arm stroke — alternating.
        let stroke = (ctx.time * 4.0).sin();
        let arm_x = if going_right {
            swim_x as i32 + 3
        } else {
            swim_x as i32 - 2
        };
        draw::dot_i(grid, arm_x, sy as i32 + (stroke * 1.5).round() as i32);
        // Kick: tiny tail dots.
        let kick_x = if going_right {
            swim_x as i32 - 1
        } else {
            swim_x as i32 + 2
        };
        let kick_off = (-stroke).round() as i32;
        draw::dot_i(grid, kick_x, sy as i32 + kick_off);
        draw::dot_i(grid, kick_x, sy as i32 + kick_off + 1);

        // Lap counter: vblocks on right columns, one per completed lap.
        let laps_done = current_lap.min(n_laps);
        let counter_cells = (n_laps).min(cw);
        let start_cell = cw.saturating_sub(counter_cells);
        for i in 0..counter_cells {
            let filled = if i < laps_done { 8 } else { 0 };
            for cy in 0..ch {
                draw::vblock(grid, start_cell + i, cy, filled);
            }
        }

        // Water ripples trailing the swimmer.
        let ripple_dir: i32 = if going_right { -1 } else { 1 };
        for k in 1..4usize {
            let rx = swim_x as i32 + ripple_dir * (k as i32 * 2);
            if rx >= 0 && (rx as usize) < w {
                let ry_off = (k as f32 * 0.7 * (ctx.time * 3.0 + k as f32).sin()) as i32;
                draw::dot_i(grid, rx, sy as i32 + ry_off);
            }
        }

        Ok(())
    }
}

// ── Archery ───────────────────────────────────────────────────────────────────

struct Archery;
impl ProgressStyle for Archery {
    fn name(&self) -> &str {
        "archery"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Bow draws back with eased progress; arrow flies across to a multi-ring bullseye"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = h / 2;

        // Bullseye target: concentric rings on the right.
        let target_cx = w.saturating_sub(3) as i32;
        let target_cy = mid as i32;
        let max_r = (h / 2).clamp(1, 3) as i32;
        for r in (1..=max_r).rev() {
            // Approximate circle with 8 cardinal dots.
            for &(dx, dy) in &[
                (r, 0),
                (-r, 0),
                (0, r),
                (0, -r),
                (r, r),
                (-r, r),
                (r, -r),
                (-r, -r),
            ] {
                draw::dot_i(grid, target_cx + dx, target_cy + dy);
            }
        }
        // Bull: center dot.
        draw::dot_i(grid, target_cx, target_cy);

        // Bow on the left side.
        let bow_x = 2i32;
        let bow_h = (h * 3 / 4).max(2) as i32;
        let bow_top = (mid as i32) - bow_h / 2;
        let bow_bot = bow_top + bow_h;
        // Bow limbs: arced using a few dots.
        for step in 0..=bow_h {
            let by = bow_top + step;
            // Arc bulge to the left (pulling back).
            let pull = ctx.eased; // 0 = no pull, 1 = full draw
            let arc = (PI * step as f32 / bow_h as f32).sin();
            let bx = bow_x - (arc * 2.0 * pull).round() as i32;
            draw::dot_i(grid, bx, by);
        }
        // Bowstring: straight line from top to bottom of bow.
        let string_x = bow_x + 1 - (ctx.eased * 1.5).round() as i32;
        draw::vline(
            grid,
            string_x.max(0) as usize,
            bow_top.max(0) as usize,
            bow_bot.min(h as i32 - 1) as usize,
        );

        // Arrow: travels from string to target when progress > 0.
        // Before release (eased < 0.8) the arrow is nocked and held back.
        // After release (eased >= 0.8) the arrow flies to the target.
        let t = ctx.eased;
        let arrow_head_x = if t < 0.8 {
            // Nocked: arrow tip just ahead of string.
            string_x + 1
        } else {
            // Flying: interpolate from nock to target.
            let flight_t = (t - 0.8) / 0.2;
            let nock_x = string_x + 1;
            (nock_x as f32 + flight_t * (target_cx - nock_x as i32) as f32).round() as i32
        };

        // Arrow shaft (horizontal dots from string to head).
        let shaft_start = (string_x + 1).max(0);
        let shaft_end = arrow_head_x.max(shaft_start);
        for ax in shaft_start..=shaft_end.min(target_cx) {
            draw::dot_i(grid, ax, mid as i32);
        }
        // Arrow head: small right-pointing tip.
        draw::dot_i(grid, arrow_head_x, mid as i32 - 1);
        draw::dot_i(grid, arrow_head_x + 1, mid as i32);
        draw::dot_i(grid, arrow_head_x, mid as i32 + 1);
        // Fletching: two dots at the tail.
        let tail_x = shaft_start - 1;
        draw::dot_i(grid, tail_x, mid as i32 - 1);
        draw::dot_i(grid, tail_x, mid as i32 + 1);

        Ok(())
    }
}

// ── Bowling ───────────────────────────────────────────────────────────────────

struct Bowling;
impl ProgressStyle for Bowling {
    fn name(&self) -> &str {
        "bowling"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Ball rolls down lane toward pins; pins scatter progressively at the end"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);
        let mid = h / 2;

        // Lane: two guide lines.
        let lane_top = mid.saturating_sub(1);
        let lane_bot = (mid + 1).min(base);
        let guide_step = (w / 8).max(1);
        for gx in (0..w).step_by(guide_step) {
            draw::dot(grid, gx.min(w.saturating_sub(1)), lane_top);
            draw::dot(grid, gx.min(w.saturating_sub(1)), lane_bot);
        }

        // Pin rack at the right end: triangle of 10 pins in rows 4-3-2-1.
        let pin_area_x = w.saturating_sub((w / 6).max(4));
        // Knock-down threshold: pins fall when progress > 0.8, staggered.
        let knock_t = ((ctx.eased - 0.8) / 0.2).clamp(0.0, 1.0);
        let rows = [4usize, 3, 2, 1];
        let mut pin_idx = 0usize;
        let total_pins = 10usize;
        for (row, &count) in rows.iter().enumerate() {
            for col in 0..count {
                let px = pin_area_x + col * 2 + row;
                let px = px.min(w.saturating_sub(1));
                let pin_frac = pin_idx as f32 / total_pins as f32;
                let knocked = knock_t > pin_frac;
                if knocked {
                    // Pin lying down: dot to the right.
                    let scatter = ((ctx.time * 3.0 + pin_idx as f32).sin() * 1.5).round() as i32;
                    draw::dot_i(grid, px as i32 + 1 + scatter, base as i32);
                } else {
                    // Pin standing: vertical stack of 2.
                    draw::dot(grid, px, mid);
                    draw::dot(grid, px, mid.saturating_sub(1));
                }
                pin_idx += 1;
            }
        }

        // Ball: rolls from left to pin area.
        let ball_x = (ctx.eased * pin_area_x as f32) as usize;
        let ball_x = ball_x.min(pin_area_x.saturating_sub(2));
        // Slight vertical wobble on roll.
        let wobble = ((ctx.time * 12.0).sin() * 0.4).round() as i32;
        let by = (mid as i32 + wobble).clamp(0, base as i32) as usize;
        // Ball (2x2 dots).
        draw::dot(grid, ball_x, by);
        draw::dot(grid, (ball_x + 1).min(w.saturating_sub(1)), by);
        draw::dot(grid, ball_x, (by + 1).min(base));
        draw::dot(
            grid,
            (ball_x + 1).min(w.saturating_sub(1)),
            (by + 1).min(base),
        );
        // Spin dot.
        let spin_angle = ctx.time * 10.0;
        let sdx = spin_angle.cos().round() as i32;
        let sdy = spin_angle.sin().round() as i32;
        draw::dot_i(grid, ball_x as i32 + sdx, by as i32 + sdy);

        // Color: lane gets warmer toward pins.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            for cx in 0..filled_cells.min(cw) {
                let t = if filled_cells <= 1 {
                    0.0
                } else {
                    cx as f32 / (filled_cells - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Darts ─────────────────────────────────────────────────────────────────────

struct Darts;
impl ProgressStyle for Darts {
    fn name(&self) -> &str {
        "darts"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Dart flies toward concentric scoring rings; board glows as the dart closes in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = h / 2;

        // Dartboard: concentric dot-rings on the right, centered.
        let board_cx = w.saturating_sub(2) as i32;
        let board_cy = mid as i32;
        let n_rings = (h / 2).clamp(1, 4);
        for r in 1..=n_rings {
            let radius = r as i32;
            // Draw partial ellipse (wider than tall) using parametric dots.
            let steps = (radius * 8).max(8) as usize;
            for s in 0..steps {
                let angle = 2.0 * PI * s as f32 / steps as f32;
                let ex = (board_cx + (angle.cos() * radius as f32 * 1.5).round() as i32).max(0);
                let ey = (board_cy + (angle.sin() * radius as f32 * 0.75).round() as i32).max(0);
                draw::dot_i(grid, ex, ey);
            }
        }
        // Bull.
        draw::dot_i(grid, board_cx, board_cy);

        // Dart: travels horizontally from the left.
        let dart_x = (ctx.eased * board_cx as f32).round() as i32;
        let dart_y = board_cy;

        // Slight arc drop: gravity dip over the flight.
        let drop = if dart_x < board_cx {
            let flight_t = dart_x as f32 / board_cx.max(1) as f32;
            (flight_t * (1.0 - flight_t) * 2.0 * h as f32 * 0.2).round() as i32
        } else {
            0
        };
        let dart_y_dropped = (dart_y + drop).clamp(0, h as i32 - 1);

        // Dart body: tip then shaft then flights.
        // Tip.
        draw::dot_i(grid, dart_x, dart_y_dropped);
        // Barrel (2 dots back).
        draw::dot_i(grid, dart_x - 1, dart_y_dropped);
        draw::dot_i(grid, dart_x - 2, dart_y_dropped);
        // Flights: angled fork.
        draw::dot_i(grid, dart_x - 3, dart_y_dropped - 1);
        draw::dot_i(grid, dart_x - 3, dart_y_dropped + 1);
        draw::dot_i(grid, dart_x - 4, dart_y_dropped - 2);
        draw::dot_i(grid, dart_x - 4, dart_y_dropped + 2);

        // Board glow: as dart gets close, fill innermost ring column.
        if ctx.eased > 0.8 {
            let glow_t = (ctx.eased - 0.8) / 0.2;
            let glow_r = (glow_t * n_rings as f32).round() as i32;
            for gr in 1..=glow_r.min(n_rings as i32) {
                draw::dot_i(grid, board_cx, board_cy - gr);
                draw::dot_i(grid, board_cx, board_cy + gr);
                draw::dot_i(grid, board_cx - gr, board_cy);
                draw::dot_i(grid, board_cx + gr, board_cy);
            }
        }

        Ok(())
    }
}

// ── High Jump ─────────────────────────────────────────────────────────────────

struct HighJump;
impl ProgressStyle for HighJump {
    fn name(&self) -> &str {
        "high-jump"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Athlete arcs over a bar that rises with progress; backflop pose at peak"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);

        // High-jump mat: fill at the right.
        let mat_x = w.saturating_sub((w / 5).max(2));
        draw::hline(grid, mat_x, w.saturating_sub(1), base);

        // Bar uprights: two vertical posts.
        let post_h = (ctx.eased * base as f32).round() as usize;
        let post_h = post_h.max(1);
        let lpost = mat_x.saturating_sub(2).min(w.saturating_sub(1));
        let rpost = (mat_x + 1).min(w.saturating_sub(1));
        draw::vline(grid, lpost, base.saturating_sub(post_h), base);
        draw::vline(grid, rpost, base.saturating_sub(post_h), base);
        // Crossbar at the top of the posts.
        let bar_y = base.saturating_sub(post_h);
        draw::hline(grid, lpost, rpost, bar_y);

        // Athlete: progresses from approach run to arc over bar.
        // phase: 0 = running in, 0.5 = peak arc, 1 = landing.
        let t = ctx.eased;
        // Athlete x: runs toward post then curves over and lands on mat.
        let approach_x = (t * lpost as f32) as usize;
        // Arc: highest at t=0.5, using a parabola shifted by the bar height.
        let arc_frac = 4.0 * t * (1.0 - t); // 0→1→0
        let arc_lift = (arc_frac * post_h as f32).round() as usize;
        let ath_base = base.saturating_sub(arc_lift);

        let ax = approach_x.min(w.saturating_sub(3));

        // Body shape changes with arc phase.
        if t < 0.4 {
            // Running: upright torso.
            draw::dot(grid, ax, ath_base);
            draw::dot(grid, ax + 1, ath_base);
            draw::dot(grid, ax, ath_base.saturating_sub(1));
            // Head.
            draw::dot_i(grid, ax as i32 + 1, ath_base as i32 - 2);
            // Legs.
            let leg_phase = (ctx.time * 8.0).sin();
            draw::dot_i(grid, ax as i32 + 1, ath_base as i32 + 1);
            draw::dot_i(
                grid,
                ax as i32 - 1 + (leg_phase * 1.0).round() as i32,
                ath_base as i32 + 1,
            );
        } else {
            // Arcing: horizontal body (Fosbury flop).
            let bod_y = ath_base;
            draw::dot(grid, ax, bod_y);
            draw::dot(grid, ax + 1, bod_y);
            draw::dot(grid, ax + 2, bod_y);
            // Arch the back: middle dot slightly higher.
            draw::dot_i(grid, ax as i32 + 1, bod_y as i32 - 1);
            // Head tilted back.
            draw::dot_i(grid, ax as i32 - 1, bod_y as i32 - 1);
            // Legs up.
            draw::dot_i(grid, ax as i32 + 3, bod_y as i32 - 1);
            draw::dot_i(grid, ax as i32 + 4, bod_y as i32 - 2);
        }

        // Approach runway: ground to the left.
        draw::hline(grid, 0, lpost.saturating_sub(1), base);

        Ok(())
    }
}

// ── Weightlifting ─────────────────────────────────────────────────────────────

struct Weightlifting;
impl ProgressStyle for Weightlifting {
    fn name(&self) -> &str {
        "weightlifting"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Barbell lifted overhead; plate stack height and arm angle track eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);
        let cx = w / 2;

        // Lift progress: 0 = bar on floor, 1 = fully overhead.
        // Use eased so the lift has inertia.
        let lift = ctx.eased;
        // Barbell y position: from near base to top.
        let bar_range = (base.saturating_sub(2)) as f32;
        let bar_y = base
            .saturating_sub(2)
            .saturating_sub((lift * bar_range) as usize);
        let bar_y = bar_y.min(base.saturating_sub(1));

        // Barbell shaft.
        let shaft_half = (w / 4).max(2);
        draw::hline(
            grid,
            cx.saturating_sub(shaft_half),
            (cx + shaft_half).min(w.saturating_sub(1)),
            bar_y,
        );

        // Plates: stack at each end proportional to progress.
        let max_plates = 4usize;
        let plates_on = (lift * max_plates as f32).ceil() as usize;
        for p in 0..plates_on.min(max_plates) {
            let plate_off = p + 1;
            // Left plate.
            let lpx = cx.saturating_sub(shaft_half).saturating_sub(plate_off);
            draw::vline(grid, lpx, bar_y.saturating_sub(1), (bar_y + 1).min(base));
            // Right plate.
            let rpx = (cx + shaft_half + plate_off).min(w.saturating_sub(1));
            draw::vline(grid, rpx, bar_y.saturating_sub(1), (bar_y + 1).min(base));
        }
        // Outer collar dots.
        let lcollar = cx.saturating_sub(shaft_half + plates_on + 1);
        let rcollar = (cx + shaft_half + plates_on + 1).min(w.saturating_sub(1));
        draw::dot(grid, lcollar, bar_y);
        draw::dot(grid, rcollar, bar_y);

        // Athlete: body adapts to lift phase.
        // Legs always at base, torso/arms angle with lift.
        let torso_top = bar_y.saturating_sub(1);
        let hip_y = (base.saturating_sub(1)).min(base);

        // Legs: two dots each side.
        let leg_spread = (w / 10).max(1);
        draw::vline(grid, cx.saturating_sub(leg_spread), hip_y, base);
        draw::vline(
            grid,
            (cx + leg_spread).min(w.saturating_sub(1)),
            hip_y,
            base,
        );

        // Torso.
        draw::vline(grid, cx, torso_top, hip_y);

        // Arms: angle from hips up to bar.
        // Simplified: diagonal from shoulder to bar ends.
        let shoulder_y = torso_top.saturating_sub(1);
        let lshoulder_x = cx.saturating_sub(1);
        let rshoulder_x = (cx + 1).min(w.saturating_sub(1));

        // Left arm: shoulder → left end of bar.
        let arm_steps = 3usize;
        for step in 0..=arm_steps {
            let at = step as f32 / arm_steps as f32;
            let ax = (lshoulder_x as f32 + at * (lcollar as i32 - lshoulder_x as i32) as f32)
                .round() as i32;
            let ay =
                (shoulder_y as f32 + at * (bar_y as i32 - shoulder_y as i32) as f32).round() as i32;
            draw::dot_i(grid, ax, ay);
        }
        // Right arm.
        for step in 0..=arm_steps {
            let at = step as f32 / arm_steps as f32;
            let ax = (rshoulder_x as f32 + at * (rcollar as i32 - rshoulder_x as i32) as f32)
                .round() as i32;
            let ay =
                (shoulder_y as f32 + at * (bar_y as i32 - shoulder_y as i32) as f32).round() as i32;
            draw::dot_i(grid, ax, ay);
        }

        // Head.
        draw::dot_i(grid, cx as i32, torso_top as i32 - 2);
        draw::dot_i(grid, cx as i32, torso_top as i32 - 1);

        // Strain shimmer at peak.
        if ctx.progress >= 0.9 {
            let shake = ((ctx.time * 20.0).sin() * 0.5).round() as i32;
            draw::dot_i(grid, cx as i32 + shake, bar_y as i32 - 1);
        }

        Ok(())
    }
}

// ── Tennis Rally ──────────────────────────────────────────────────────────────

struct TennisRally;
impl ProgressStyle for TennisRally {
    fn name(&self) -> &str {
        "tennis-rally"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Ball bounces between baselines; a smooth hbar fill tracks rally progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cw, ch) = grid.dimensions();

        // Bottom row: hbar fill showing overall progress.
        if ch > 0 {
            draw::hbar(grid, ch.saturating_sub(1), ctx.eased);
        }

        // Court area: rows above the hbar.
        let court_rows = ch.saturating_sub(1);
        if court_rows == 0 {
            return Ok(());
        }
        let court_h = court_rows * 4;
        let court_base = court_h.saturating_sub(1);

        // Court lines: baselines and net.
        draw::hline(grid, 0, w.saturating_sub(1), court_base);
        draw::hline(grid, 0, w.saturating_sub(1), 0);
        let net_x = w / 2;
        draw::vline(grid, net_x, 0, court_base);

        // Service line markers.
        let sl = w / 4;
        let sr = w * 3 / 4;
        for sy in (0..court_base).step_by(2) {
            draw::dot(grid, sl.min(w.saturating_sub(1)), sy);
            draw::dot(grid, sr.min(w.saturating_sub(1)), sy);
        }

        // Ball: bounces back and forth driven by time.
        // Horizontal: sinusoidal across width (rally speed increases with progress).
        let rally_speed = 1.5 + ctx.progress * 3.0;
        let bx_raw = ((ctx.time * rally_speed).sin() * 0.5 + 0.5) * (w - 1) as f32;
        let bx = bx_raw.round() as usize;
        let bx = bx.min(w.saturating_sub(1));

        // Vertical: parabolic bounce — abs(sin) gives repeated parabola shape.
        let bounce_freq = rally_speed * 2.0;
        let bounce = ((ctx.time * bounce_freq).sin()).abs();
        let by_raw = court_base as f32 - bounce * (court_base as f32 * 0.7);
        let by = by_raw.round().clamp(0.0, court_base as f32) as usize;

        // Ball dot.
        draw::dot(grid, bx, by);
        // Second dot for visibility (right or below).
        if bx + 1 < w {
            draw::dot(grid, bx + 1, by);
        }

        // Ball shadow on the court floor.
        draw::dot(grid, bx, court_base);

        // Players: simple stick figures at each end.
        // Left player (server side).
        let lp_x = 1usize;
        let lp_mid = court_h / 2;
        draw::dot(grid, lp_x, lp_mid.saturating_sub(1));
        draw::dot(grid, lp_x, lp_mid);
        draw::dot(grid, lp_x, (lp_mid + 1).min(court_base));
        // Right player.
        let rp_x = w.saturating_sub(2);
        draw::dot(grid, rp_x, lp_mid.saturating_sub(1));
        draw::dot(grid, rp_x, lp_mid);
        draw::dot(grid, rp_x, (lp_mid + 1).min(court_base));

        // Tint court.
        for cy in 0..court_rows {
            for cx in 0..cw {
                let t = if cw <= 1 {
                    0.5
                } else {
                    cx as f32 / (cw - 1) as f32
                };
                // Subtle tint — muted by applying only partially.
                let color = ctx.palette.sample(t);
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ── Cycling Peloton ───────────────────────────────────────────────────────────

struct CyclingPeloton;
impl ProgressStyle for CyclingPeloton {
    fn name(&self) -> &str {
        "cycling-peloton"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Tightly-packed peloton advances; each bike's wheels spin with time via dot rotation"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);
        let rider_h = (h / 2).max(2);
        let body_y = base.saturating_sub(rider_h);

        // Road: baseline.
        draw::hline(grid, 0, w.saturating_sub(1), base);
        // Road markings.
        let center_y = (base + body_y) / 2;
        for rx in (0..w).step_by(5) {
            draw::dot(grid, rx.min(w.saturating_sub(1)), center_y);
        }

        // Pack of riders: progress dictates how far the leading edge is.
        let lead_x = (ctx.eased * w as f32) as usize;
        let lead_x = lead_x.min(w.saturating_sub(1));

        // Number of riders: scales with bar width, minimum 2.
        let n_riders = (w / 10).clamp(2, 8);
        // Pack depth behind leader.
        let pack_depth = (n_riders as f32 * 5.0) as usize;

        // Wheel spin angle (radians) driven by time.
        let spin = ctx.time * 8.0;
        let wheel_r = ((h / 4).max(1)) as f32;

        for i in 0..n_riders {
            // Space riders behind the leader.
            let offset = i * (pack_depth / n_riders.max(1));
            let rx = lead_x.saturating_sub(offset);
            if rx == 0 && i > 0 {
                continue;
            }
            let rx = rx.min(w.saturating_sub(3));

            // Each rider: two wheels + frame + body.
            let front_wheel_x = (rx + 2).min(w.saturating_sub(1)) as i32;
            let rear_wheel_x = rx as i32;
            let wheel_y = (base - 1) as i32;

            // Wheel spokes (2 per wheel, cross-shaped, rotating).
            for spoke in 0..2usize {
                let angle = spin + spoke as f32 * PI;
                let sdx = (angle.cos() * wheel_r * 0.8).round() as i32;
                let sdy = (angle.sin() * wheel_r * 0.5).round() as i32;
                // Front wheel.
                draw::dot_i(grid, front_wheel_x + sdx, wheel_y + sdy);
                draw::dot_i(grid, front_wheel_x - sdx, wheel_y - sdy);
                // Rear wheel.
                draw::dot_i(grid, rear_wheel_x + sdx, wheel_y + sdy);
                draw::dot_i(grid, rear_wheel_x - sdx, wheel_y - sdy);
            }
            // Wheel rims (hub dots).
            draw::dot_i(grid, front_wheel_x, wheel_y);
            draw::dot_i(grid, rear_wheel_x, wheel_y);

            // Frame: diagonal from rear wheel to front wheel top.
            draw::dot_i(grid, rear_wheel_x + 1, wheel_y - 1);
            draw::dot_i(grid, front_wheel_x - 1, wheel_y - 1);

            // Rider body.
            let seat_x = rear_wheel_x + 1;
            let seat_y = wheel_y - 2;
            draw::dot_i(grid, seat_x, seat_y);
            // Torso leans forward.
            draw::dot_i(grid, seat_x + 1, seat_y - 1);
            draw::dot_i(grid, seat_x + 1, seat_y - 2);
            // Head.
            draw::dot_i(grid, front_wheel_x, seat_y - 2);
        }

        // Gradient tint of the swept portion.
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            for cx in 0..filled.min(cw) {
                let t = if filled <= 1 {
                    0.0
                } else {
                    cx as f32 / (filled - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
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
    let styles = progress::styles::sports::styles();
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
