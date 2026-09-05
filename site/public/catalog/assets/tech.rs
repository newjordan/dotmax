//! `tech` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O tech.rs && ./tech [style-name]
//! ```

const DEFAULT_STYLE: &str = "matrix-rain";

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
    pub mod tech {
//! Tech / cyberpunk progress bars — digital rain, glitch, neon, and signal.
//!
//! Every style in this module is stateless: all animation derives from
//! `ctx.time` and `ctx.eased` with no mutable state. The deterministic
//! `hash` helper produces pseudo-random values for sparkle and glitch
//! effects without any external dependencies.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::cmp::Ordering;
use std::f32::consts::PI;

// ─── deterministic hash (no external crates) ────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

/// Map hash output to [0.0, 1.0).
#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

/// All styles in the `tech` theme.
///
/// Returns 11 distinct cyberpunk-themed progress bar implementations, each
/// stateless and animatable via `ctx.time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(MatrixRain),
        Box::new(NeonScanline),
        Box::new(DataPackets),
        Box::new(GlitchBar),
        Box::new(TerminalTyper),
        Box::new(HexFill),
        Box::new(SignalBars),
        Box::new(DownloadStream),
        Box::new(BinaryCounter),
        Box::new(Heartbeat),
        Box::new(CircuitTrace),
    ]
}

// ─── 1. Matrix digital rain ──────────────────────────────────────────────────

/// Matrix-style columns of falling dots whose density rises with progress.
struct MatrixRain;
impl ProgressStyle for MatrixRain {
    fn name(&self) -> &str {
        "matrix-rain"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Matrix digital rain: column density rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // How many columns are "active" depends on eased progress.
        let active_cols = ((ctx.eased * w as f32) as usize).min(w);

        for col in 0..active_cols {
            // Each column gets a unique phase offset from the hash.
            let phase = hashf(col as u32 * 7 + 1);
            let speed = 0.4 + 0.6 * hashf(col as u32 * 13 + 3);
            // Fall position: wraps top-to-bottom.
            let fall_t = (ctx.time * speed + phase).fract();
            let head = (fall_t * h as f32) as usize;

            // Draw a "raindrop" — head is bright, tail fades.
            let tail_len = (h / 3).max(2);
            for i in 0..tail_len {
                let y = if head >= i { head - i } else { h + head - i };
                if y < h {
                    draw::dot(grid, col, y);
                }
            }

            // Tint with palette: head bright end, tail dim start.
            let cell_x = col / 2;
            if cell_x < cells_w {
                let t = col as f32 / active_cols.max(1) as f32;
                let color = ctx.palette.sample(t);
                for cy in 0..cells_h {
                    draw::tint_row(grid, cy, cell_x, cell_x, color);
                }
            }
        }
        Ok(())
    }
}

// ─── 2. Neon scanline sweep ──────────────────────────────────────────────────

/// A neon vertical bar sweeps right; eased progress determines how far it
/// travels. The swept region glows with the palette gradient.
struct NeonScanline;
impl ProgressStyle for NeonScanline {
    fn name(&self) -> &str {
        "neon-scanline"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Neon vertical scanline with palette glow, eased to progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let head = ((ctx.eased * w as f32) as usize).min(w.saturating_sub(1));

        // Filled track up to head.
        // Draw a center baseline and a thick beam at the head.
        let mid = h / 2;
        draw::hline(grid, 0, head, mid);

        // Scanline beam: full height at head, half-height one behind, etc.
        let beam_w = (w / 20).max(1);
        for offset in 0..beam_w {
            if head < offset {
                break;
            }
            let x = head - offset;
            let reach = (h as f32 * (1.0 - offset as f32 / beam_w as f32)) as usize;
            let y0 = mid.saturating_sub(reach / 2);
            let y1 = (mid + reach / 2).min(h.saturating_sub(1));
            draw::vline(grid, x, y0, y1);
        }

        // Gradient tint from start to head.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = if filled_cells <= 1 {
                0.0
            } else {
                cx as f32 / (filled_cells - 1) as f32
            };
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 3. Data packets ─────────────────────────────────────────────────────────

/// Discrete data "packets" slide left→right; the number flowing increases with
/// progress. Uses cubic-out easing on each packet's internal travel.
struct DataPackets;
impl ProgressStyle for DataPackets {
    fn name(&self) -> &str {
        "data-packets"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Discrete data packets stream right; flow rate scales with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let packet_w = (w / 8).max(2);
        let gap = (w / 12).max(1);
        let stride = packet_w + gap;
        let n_packets = (w / stride).max(1);

        // How many packets are visible is gated by progress.
        let active = (ctx.eased * n_packets as f32).ceil() as usize;

        for p in 0..active.min(n_packets) {
            let phase = p as f32 / n_packets as f32;
            // Each packet loops: position travels 0..w in its own time slot.
            let t_raw = (ctx.time * 0.5 + phase).fract();
            // Cubic-out easing for a "snap then slow" feel.
            let t_eased = 1.0 - (1.0 - t_raw).powi(3);
            let x0 = (t_eased * (w + packet_w) as f32) as i32 - packet_w as i32;

            let row_h = (h / 2).max(1);
            let y0 = (h - row_h) / 2;
            for dy in 0..row_h {
                for dx in 0..packet_w as i32 {
                    draw::dot_i(grid, x0 + dx, (y0 + dy) as i32);
                }
            }

            // Tint the packet's cell span.
            let t_color = p as f32 / n_packets.max(1) as f32;
            let color = ctx.palette.sample(t_color);
            let cx0 = (x0 / 2).max(0) as usize;
            let cx1 = ((x0 + packet_w as i32) / 2).clamp(0, cells_w as i32 - 1) as usize;
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx0, cx1, color);
            }
        }
        Ok(())
    }
}

// ─── 4. Glitch bar ───────────────────────────────────────────────────────────

/// A solid fill that occasionally "glitches": horizontal strips are displaced
/// by random amounts derived from the hash function and snapped by time.
struct GlitchBar;
impl ProgressStyle for GlitchBar {
    fn name(&self) -> &str {
        "glitch"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Solid fill with periodic horizontal glitch displacements"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let filled = ((ctx.eased * w as f32) as usize).min(w);

        // Glitch "epoch" changes 4 times per second; drives which rows glitch.
        let epoch = (ctx.time * 4.0) as u32;

        for y in 0..h {
            // Decide if this scanline glitches this epoch.
            let row_hash = hash(y as u32 * 31 + epoch * 997);
            let glitch_prob = hashf(row_hash);
            let shift: i32 = if glitch_prob < 0.15 {
                // Displace by up to ±1/8 of width.
                let mag = (hashf(row_hash ^ 0xDEAD) * (w as f32 / 8.0)) as i32;
                if row_hash & 1 == 0 {
                    mag
                } else {
                    -mag
                }
            } else {
                0
            };

            for x in 0..filled {
                draw::dot_i(grid, x as i32 + shift, y as i32);
            }
        }

        // Gradient tint on the filled region.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 5. Terminal typer ───────────────────────────────────────────────────────

/// A cursor "types" across the bar, leaving a trail of filled dots; the cursor
/// blinks at 2 Hz using `ctx.time`. Progress controls how far it has typed.
struct TerminalTyper;
impl ProgressStyle for TerminalTyper {
    fn name(&self) -> &str {
        "terminal-typer"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Cursor types across the bar; cursor blinks, trail is filled"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let head = ((ctx.eased * w as f32) as usize).min(w.saturating_sub(1));

        // Trail: filled region before cursor.
        if head > 0 {
            draw::fill_rect(grid, 0, 0, head, h);
        }

        // Blinking cursor block: blinks at 2 Hz.
        let blink_on = (ctx.time * 2.0).fract() > 0.5;
        if blink_on && head < w {
            // Cursor is a full-height vertical block.
            draw::vline(grid, head, 0, h.saturating_sub(1));
            // Add one to the right for a fat cursor.
            if head + 1 < w {
                draw::vline(grid, head + 1, 0, h.saturating_sub(1));
            }
        }

        // Tint: trail in palette gradient, cursor in bright end color.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 6. Hex / binary block fill ──────────────────────────────────────────────

/// The bar is divided into fixed-width "hex cells". Each cell toggles on in
/// sequence as eased progress advances, giving a quantised, digital feel.
struct HexFill;
impl ProgressStyle for HexFill {
    fn name(&self) -> &str {
        "hex-fill"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Hex-cell blocks toggle on sequentially as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let n_cells = 16usize.min(w / 2);
        let cell_w = w / n_cells.max(1);
        let lit = (ctx.eased * n_cells as f32) as usize;

        for i in 0..n_cells {
            let x0 = i * cell_w;
            // Gap of 1 dot between cells.
            let bw = cell_w.saturating_sub(1).max(1);
            match i.cmp(&lit) {
                Ordering::Less => {
                    // Fully lit cell.
                    draw::fill_rect(grid, x0, 0, bw, h);
                }
                Ordering::Equal => {
                    // Partially lit cell — animates in with a sine flicker.
                    let flicker = ((ctx.time * 8.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                    let bh = (flicker * h as f32) as usize;
                    let y0 = h.saturating_sub(bh);
                    draw::fill_rect(grid, x0, y0, bw, bh);
                }
                Ordering::Greater => {
                    // Unlit: just outline.
                    draw::rect_outline(grid, x0, 0, bw.max(2), h.max(2));
                }
            }

            // Tint lit cells.
            if i <= lit {
                let t = i as f32 / n_cells.max(1) as f32;
                let color = ctx.palette.sample(t);
                let cx0 = (x0 / 2).min(cells_w.saturating_sub(1));
                let cx1 = ((x0 + bw) / 2).min(cells_w.saturating_sub(1));
                for cy in 0..cells_h {
                    draw::tint_row(grid, cy, cx0, cx1, color);
                }
            }
        }
        Ok(())
    }
}

// ─── 7. Signal / WiFi bars ───────────────────────────────────────────────────

/// Rising signal bars whose heights step up with progress, like a WiFi
/// strength indicator. Bars pulse slightly via a sine wave.
struct SignalBars;
impl ProgressStyle for SignalBars {
    fn name(&self) -> &str {
        "signal-bars"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Rising WiFi-style signal bars that fill with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let n_bars = 8usize;
        let bar_w = (w / n_bars / 2).max(1);
        let gap = (w / n_bars).saturating_sub(bar_w).max(1);
        let stride = bar_w + gap;

        let lit_count = (ctx.eased * n_bars as f32).round() as usize;

        for i in 0..n_bars {
            let x0 = i * stride;
            if x0 >= w {
                break;
            }

            // Each bar's target height: taller bars toward the right.
            let base_frac = (i + 1) as f32 / n_bars as f32;
            // Pulse: active bars breathe slightly.
            let pulse = if i < lit_count {
                1.0 + 0.05 * (ctx.time * 2.0 * PI + i as f32 * 0.5).sin()
            } else {
                1.0
            };
            let bar_h = (base_frac * h as f32 * pulse).round() as usize;
            let bar_h = bar_h.min(h);
            let y0 = h.saturating_sub(bar_h);

            if i < lit_count {
                draw::fill_rect(grid, x0, y0, bar_w, bar_h);
                let t = i as f32 / n_bars.max(1) as f32;
                let color = ctx.palette.sample(t);
                let cx = (x0 / 2).min(cells_w.saturating_sub(1));
                for cy in 0..cells_h {
                    draw::tint_row(grid, cy, cx, cx, color);
                }
            } else {
                // Dim outline for inactive bars.
                if bar_h >= 2 && bar_w >= 1 {
                    draw::rect_outline(grid, x0, y0, bar_w.max(2), bar_h.max(2));
                }
            }
        }
        Ok(())
    }
}

// ─── 8. Download stream ──────────────────────────────────────────────────────

/// A moving "buffer window" slides over a track of dots. The track density
/// behind the window matches eased progress; the window itself scrolls.
struct DownloadStream;
impl ProgressStyle for DownloadStream {
    fn name(&self) -> &str {
        "download-stream"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Moving buffer window slides over a download track"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let filled = ((ctx.eased * w as f32) as usize).min(w);

        // Background track: sparse dots every 3 columns in the filled zone.
        let mid = h / 2;
        for x in 0..filled {
            if x % 3 == 0 {
                draw::dot(grid, x, mid);
            }
        }

        // Buffer window: a bright solid block that scrolls through filled zone.
        let win_w = (w / 6).max(2);
        if filled > win_w {
            let travel = filled - win_w;
            let win_x = ((ctx.time * 0.8).fract() * travel as f32) as usize;
            let win_x = win_x.min(travel);
            let win_h = h;
            draw::fill_rect(grid, win_x, 0, win_w, win_h);
        }

        // Track baseline.
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        // Tint the filled region.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 9. Binary counter ───────────────────────────────────────────────────────

/// Progress is rendered as a binary number — each bit column toggles on/off as
/// it would in a rising counter, giving a flickering digital readout effect.
struct BinaryCounter;
impl ProgressStyle for BinaryCounter {
    fn name(&self) -> &str {
        "binary-counter"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Progress displayed as a live binary counter in dot columns"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Map progress to an integer value in [0, 2^n_bits).
        let n_bits = (w / 4).clamp(4, 16);
        let max_val = (1u32 << n_bits).saturating_sub(1);
        let val = (ctx.eased * max_val as f32).round() as u32;

        let bit_w = w / n_bits;
        let bit_w = bit_w.max(1);

        for bit in 0..n_bits {
            // MSB on the left.
            let bit_idx = n_bits - 1 - bit;
            let on = (val >> bit_idx) & 1 == 1;
            let x0 = bit * bit_w;
            let bw = bit_w.saturating_sub(1).max(1);

            if on {
                // Lit bit: full column.
                draw::fill_rect(grid, x0, 0, bw, h);
                // Tint.
                let t = bit as f32 / n_bits.max(1) as f32;
                let color = ctx.palette.sample(t);
                let cx = (x0 / 2).min(cells_w.saturating_sub(1));
                for cy in 0..cells_h {
                    draw::tint_row(grid, cy, cx, cx, color);
                }
            } else {
                // Unlit bit: just the bottom dot.
                draw::dot(grid, x0, h.saturating_sub(1));
            }
        }

        // Time-based flicker on the LSB to suggest counting.
        let lsb_x = (n_bits - 1) * bit_w;
        if (ctx.time * 8.0).fract() > 0.5 {
            draw::vline(grid, lsb_x, 0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ─── 10. Heartbeat / EKG ────────────────────────────────────────────────────

/// An EKG trace advances with progress. A sharp spike pulses once per second
/// driven by `ctx.time`; the baseline has filled up to `ctx.eased`.
struct Heartbeat;
impl ProgressStyle for Heartbeat {
    fn name(&self) -> &str {
        "heartbeat"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "EKG heartbeat line pulses in real time; trace advances with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let filled = ((ctx.eased * w as f32) as usize).min(w);
        let base = (h * 2 / 3).min(h.saturating_sub(1));

        // Draw baseline up to the filled point.
        draw::hline(grid, 0, filled.saturating_sub(1), base);

        // EKG spike: a repeating sharp bump that scrolls at ctx.time.
        // Spike width in dots.
        let spike_w = (w / 6).max(4);
        // Phase: spike repeats every 1.0 seconds.
        let phase = ctx.time.fract();
        // Spike head position (leading edge of the filled region, scrolling).
        let spike_center = if filled > spike_w {
            let travel = filled - spike_w;
            let scroll = (phase * travel as f32) as usize;
            scroll + spike_w / 2
        } else {
            filled / 2
        };

        // Draw the spike shape: /\_ with sharp peak.
        let peak_h = (h as f32 * 0.85) as usize;
        for dx in 0..spike_w {
            let x = spike_center.saturating_sub(spike_w / 2) + dx;
            if x >= w {
                break;
            }
            // Normalised position within spike [0,1].
            let t = dx as f32 / spike_w.max(1) as f32;
            let y_offset: i32 = if t < 0.25 {
                // Rising: base → peak.
                let rise = t / 0.25;
                -((rise * peak_h as f32) as i32)
            } else if t < 0.5 {
                // Falling from peak: peak → below base (the "S" dip).
                let fall = (t - 0.25) / 0.25;
                -(((1.0 - fall) * peak_h as f32) as i32)
            } else if t < 0.65 {
                // Small negative dip.
                let dip = (t - 0.5) / 0.15;
                (dip * h as f32 * 0.15) as i32
            } else {
                0
            };
            let y = (base as i32 + y_offset).clamp(0, h as i32 - 1);
            draw::dot(grid, x, y as usize);
        }

        // Tint.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 11. Circuit trace ───────────────────────────────────────────────────────

/// A circuit-board trace routes across the bar: horizontal runs punctuated by
/// 90-degree turns and junction dots. The lit portion grows with eased progress
/// and "current" pulses along the trace driven by `ctx.time`.
struct CircuitTrace;
impl ProgressStyle for CircuitTrace {
    fn name(&self) -> &str {
        "circuit-trace"
    }
    fn theme(&self) -> &str {
        "tech"
    }
    fn describe(&self) -> &str {
        "Circuit board trace with routing turns, lit by progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let filled = ((ctx.eased * w as f32) as usize).min(w);

        // Build a simple repeating trace: horizontal run, then a vertical
        // "via" jog, then continue. Segment length varies by column using hash.
        let seg_len = (w / 8).max(4);
        let mut x = 0usize;
        let mut y = h / 2;
        let mut up = true; // next jog direction.

        while x < filled {
            // Horizontal run.
            let run = seg_len + (hash(x as u32 * 3 + 17) % seg_len as u32) as usize;
            let run_end = (x + run).min(filled);
            draw::hline(grid, x, run_end.saturating_sub(1), y);
            x = run_end;
            if x >= filled {
                break;
            }

            // Vertical jog (via).
            let jog = (h / 3).max(1);
            let (y0, y1) = if up {
                let new_y = y.saturating_sub(jog);
                (new_y, y)
            } else {
                let new_y = (y + jog).min(h.saturating_sub(1));
                (y, new_y)
            };
            draw::vline(grid, x.saturating_sub(1), y0, y1);
            // Junction dot (pad).
            draw::dot(grid, x.saturating_sub(1), y0);
            draw::dot(grid, x.saturating_sub(1), y1);
            y = if up {
                y.saturating_sub(jog)
            } else {
                (y + jog).min(h.saturating_sub(1))
            };
            up = !up;
        }

        // "Current pulse": a bright dot riding the trace, driven by time.
        let pulse_x = ((ctx.time * 0.7).fract() * filled as f32) as usize;
        let pulse_x = pulse_x.min(filled.saturating_sub(1));
        // Draw a 3-dot flare around the pulse.
        for dx in 0..3usize {
            let px = pulse_x.saturating_sub(1) + dx;
            if px < w {
                draw::dot(grid, px, h / 2);
            }
        }

        // Tint the lit region.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
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
    let styles = progress::styles::tech::styles();
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
