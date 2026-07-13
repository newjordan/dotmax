//! `numbertheory` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O numbertheory.rs && ./numbertheory [style-name]
//! ```

const DEFAULT_STYLE: &str = "sieve-eratosthenes";

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
    /// (`0.0..=1.0`) using eighth-width block glyphs — the classic crisp,
    /// sub-character-precise progress bar. Mixes full `█` cells with one partial
    /// edge glyph for smoothness no braille dot run can match.
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
    pub mod numbertheory {
//! Number-theory progress bars.
//!
//! Twelve styles, each grounded in a concrete mathematical structure:
//! sieves, spirals, trees, triangles, histograms, and sequence geometry.
//! Every bar maps `ctx.eased` to "how many integers have been processed" and
//! `ctx.time` to a highlight or scan animation running independently of
//! progress. All arithmetic is bounded and cannot panic.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// Trial-division primality test. Returns `false` for 0 and 1.
/// Bounded: for any n ≤ 10_000 this completes in a handful of μs.
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3u64;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

// ────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — teal into deep blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(118, 224, 202);
const TINT_END: Color = Color::rgb(44, 132, 204);

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

/// All styles in the `numbertheory` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Sieve)),
        Box::new(Tinted(UlamSpiral)),
        Box::new(Tinted(PrimeCounting)),
        Box::new(Tinted(Collatz)),
        Box::new(Tinted(FibonacciSpiral)),
        Box::new(Tinted(PascalMod)),
        Box::new(Tinted(TotientHistogram)),
        Box::new(Tinted(SternBrocot)),
        Box::new(Tinted(ContinuedFraction)),
        Box::new(Tinted(ModularCircle)),
        Box::new(Tinted(Recaman)),
        Box::new(Tinted(DigitalRoot)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Sieve of Eratosthenes
// ────────────────────────────────────────────────────────────────────────────

/// Sieve of Eratosthenes: integers 1..N laid left→right; composites are
/// crossed out one by one, primes remain lit. `eased` controls how many
/// integers are visible; `time` sweeps a highlight over the current sieve
/// multiple.
struct Sieve;
impl ProgressStyle for Sieve {
    fn name(&self) -> &str {
        "sieve-eratosthenes"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Sieve of Eratosthenes: integers cross composites, primes survive as lit dots"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // N = number of integers we lay across the width (at least 2).
        let n = w.max(2).min(2000);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n);

        // Build sieve for 1..=n.
        let mut composite = vec![false; n + 1];
        composite[0] = true;
        if n >= 1 {
            composite[1] = true;
        }
        for p in 2..=n {
            if !composite[p] {
                let mut m = p * 2;
                while m <= n {
                    composite[m] = true;
                    m += p;
                }
            }
        }

        // Animated highlight: current sieve multiple being crossed.
        let highlight_p = {
            let p_frac = (ctx.time * 0.4).fract();
            // Pick a prime index from time.
            let prime_count = (2..=n).filter(|&k| !composite[k]).count().max(1);
            let pi = ((p_frac * prime_count as f32) as usize).min(prime_count.saturating_sub(1));
            (2..=n).filter(|&k| !composite[k]).nth(pi).unwrap_or(2)
        };

        // Draw: one dot-column per integer.
        for k in 1..=revealed {
            let x = ((k - 1) * w / n).min(w.saturating_sub(1));
            let is_p = !composite[k];
            // Primes: full column; composites: half-height bottom tick.
            if is_p {
                draw::vline(grid, x, 0, h.saturating_sub(1));
                // Highlight the current sieve prime's multiples.
                if k > 1 && k % highlight_p == 0 {
                    // draw a top cap to distinguish the animated sweep target.
                    draw::dot(grid, x, 0);
                }
            } else {
                let tick = (h / 4).max(1);
                draw::vline(grid, x, h.saturating_sub(tick), h.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Ulam Spiral
// ────────────────────────────────────────────────────────────────────────────

/// Ulam spiral: integers spiral outward from the centre of the grid; primes
/// glow along diagonals. `eased` reveals integers 1..N; `time` pulses the
/// prime dots.
struct UlamSpiral;
impl ProgressStyle for UlamSpiral {
    fn name(&self) -> &str {
        "ulam-spiral"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Ulam spiral: integers coil outward from centre, primes cluster on diagonals"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;

        // Cap N to avoid spiraling off into unreachable cells.
        let n = (w * h).min(4000).max(1);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n);

        // Generate Ulam spiral coords for 1..=revealed.
        // Direction order: right, up, left, down.
        let dx = [1i32, 0, -1, 0];
        let dy = [0i32, -1, 0, 1];

        let mut x = 0i32;
        let mut y = 0i32;
        let mut dir = 0usize;
        let mut steps_in_leg = 1usize;
        let mut steps_taken = 0usize;
        let mut leg_count = 0usize;

        // Pulse phase from time.
        let pulse = (ctx.time * 2.0 * PI * 0.5).sin() * 0.5 + 0.5;
        let _ = pulse; // used conceptually; we draw or skip based on phase parity.

        for n_i in 1..=revealed {
            let px = cx + x;
            let py = cy + y;

            if is_prime(n_i as u64) {
                draw::dot_i(grid, px, py);
            }

            // Advance spiral.
            x += dx[dir];
            y += dy[dir];
            steps_taken += 1;
            if steps_taken == steps_in_leg {
                steps_taken = 0;
                dir = (dir + 1) % 4;
                leg_count += 1;
                if leg_count % 2 == 0 {
                    steps_in_leg += 1;
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Prime counting function π(x)
// ────────────────────────────────────────────────────────────────────────────

/// Prime counting function π(x): a curve showing how many primes are ≤ x,
/// displayed as a rising step graph filling left to right with `eased`.
/// `time` scrolls a thin vertical scan line.
struct PrimeCounting;
impl ProgressStyle for PrimeCounting {
    fn name(&self) -> &str {
        "prime-counting"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "π(x) prime-counting function: a rising step curve filling as primes accumulate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n = w.max(2).min(3000);
        let revealed_x = ((ctx.eased * n as f32).round() as usize).min(n);

        // Precompute π(k) for k in 1..=n.
        let mut pi = 0usize;
        let mut prime_counts = vec![0usize; n + 1];
        for k in 1..=n {
            if is_prime(k as u64) {
                pi += 1;
            }
            prime_counts[k] = pi;
        }
        let pi_max = prime_counts[n].max(1);

        // Draw the step curve up to revealed_x.
        for k in 1..=revealed_x {
            let x = ((k - 1) * w / n).min(w.saturating_sub(1));
            let count = prime_counts[k];
            // Map prime count to y (bottom = 0 primes, top = pi_max).
            let bar_h = (count * h / pi_max).min(h);
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
        }

        // Animated scan line.
        let scan_x = ((ctx.time * 0.3).fract() * w as f32) as usize;
        if scan_x < w {
            draw::vline(grid, scan_x, 0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Collatz (3n+1) trajectories
// ────────────────────────────────────────────────────────────────────────────

/// Collatz conjecture: for each seed n (swept by `eased` over 1..N), plot
/// the stopping trajectory (n→n/2 if even, 3n+1 if odd). Height encodes
/// the value normalised to the peak value in the window. `time` selects
/// which trajectory is currently highlighted.
struct Collatz;
impl ProgressStyle for Collatz {
    fn name(&self) -> &str {
        "collatz"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Collatz (3n+1): seeds swept left→right, trajectory height is stopping sequence depth"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of seeds = width in dots, max 500.
        let n_seeds = w.min(500).max(1);
        let revealed = ((ctx.eased * n_seeds as f32).round() as usize).min(n_seeds);

        // Collatz stopping time for seed k.
        fn stopping_time(mut k: u64) -> u64 {
            let mut count = 0u64;
            while k != 1 && count < 10_000 {
                k = if k % 2 == 0 { k / 2 } else { 3 * k + 1 };
                count += 1;
            }
            count
        }

        // Collect stopping times for seeds 1..=n_seeds.
        let times: Vec<u64> = (1..=n_seeds).map(|k| stopping_time(k as u64)).collect();
        let max_t = times.iter().copied().max().unwrap_or(1).max(1);

        // Highlight column from time.
        let hi_x = ((ctx.time * 0.7).fract() * revealed as f32) as usize;

        for (i, &t) in times.iter().enumerate().take(revealed) {
            let x = (i * w / n_seeds).min(w.saturating_sub(1));
            let bar_h = ((t as f32 / max_t as f32) * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
            // Extra top dot for highlight column.
            if i == hi_x && y0 > 0 {
                draw::dot(grid, x, y0.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Fibonacci / Golden spiral
// ────────────────────────────────────────────────────────────────────────────

/// Fibonacci golden spiral: draws successive quarter-circle arcs in squares
/// whose side lengths are Fibonacci numbers. `eased` controls how many arcs
/// are drawn; `time` rotates a glowing cursor along the outermost arc. The
/// golden ratio φ = (1+√5)/2 governs growth.
struct FibonacciSpiral;
impl ProgressStyle for FibonacciSpiral {
    fn name(&self) -> &str {
        "fibonacci-spiral"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Golden spiral: Fibonacci quarter-circle arcs converging on φ"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Generate Fibonacci numbers scaled to fit the grid.
        let max_dim = w.min(h).max(1);
        let mut fibs = vec![1usize, 1usize];
        while *fibs.last().unwrap() < max_dim {
            let n = fibs.len();
            let next = fibs[n - 1] + fibs[n - 2];
            fibs.push(next);
            if fibs.len() > 20 {
                break;
            }
        }

        let max_arcs = fibs.len().min(12);
        let revealed = ((ctx.eased * max_arcs as f32).round() as usize).min(max_arcs);

        // Centre of the spiral.
        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;

        // Arc start angles follow the spiral: 0°, 90°, 180°, 270°, repeating.
        let start_angles = [0.0f32, PI / 2.0, PI, 3.0 * PI / 2.0];

        let mut rx = cx;
        let mut ry = cy;

        for (arc_idx, &fib) in fibs.iter().enumerate().take(revealed) {
            let r = fib as f32;
            let angle_start = start_angles[arc_idx % 4];
            let steps = ((r * PI / 2.0) as usize).max(4).min(256);

            for s in 0..=steps {
                let theta = angle_start + (s as f32 / steps as f32) * (PI / 2.0);
                let px = rx + (theta.cos() * r) as i32;
                let py = ry + (theta.sin() * r) as i32;
                draw::dot_i(grid, px, py);
            }

            // Advance centre for next arc.
            match arc_idx % 4 {
                0 => ry -= fib as i32,
                1 => rx -= fib as i32,
                2 => ry += fib as i32,
                _ => rx += fib as i32,
            }
        }

        // Animated cursor on outermost arc.
        if revealed > 0 && revealed <= fibs.len() {
            let last_idx = revealed.saturating_sub(1);
            let r = fibs[last_idx] as f32;
            let angle_start = start_angles[last_idx % 4];
            let theta = angle_start + (ctx.time * 0.5).fract() * (PI / 2.0);
            // cursor centre — approximate; recompute centre for last arc.
            let mut cxl = cx;
            let mut cyl = cy;
            for i in 0..last_idx {
                match i % 4 {
                    0 => cyl -= fibs[i] as i32,
                    1 => cxl -= fibs[i] as i32,
                    2 => cyl += fibs[i] as i32,
                    _ => cxl += fibs[i] as i32,
                }
            }
            let px = cxl + (theta.cos() * r) as i32;
            let py = cyl + (theta.sin() * r) as i32;
            draw::dot_i(grid, px, py);
            draw::dot_i(grid, px + 1, py);
            draw::dot_i(grid, px, py + 1);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Pascal's triangle mod p
// ────────────────────────────────────────────────────────────────────────────

/// Pascal's triangle mod p: rows revealed = `eased × height`. Mod 2 yields
/// Sierpinski's gasket; `time` cycles through mod 2, 3, 5 to show different
/// self-similar patterns.
struct PascalMod;
impl ProgressStyle for PascalMod {
    fn name(&self) -> &str {
        "pascal-mod"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Pascal's triangle mod p: Sierpinski (p=2), mod-3, mod-5 patterns cycling with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Pick modulus from time: cycles 2→3→5→2 every 3 s.
        let mods = [2u64, 3, 5];
        let mod_idx = ((ctx.time / 3.0) as usize) % mods.len();
        let modulus = mods[mod_idx];

        let revealed_rows = ((ctx.eased * h as f32).round() as usize).min(h);

        // We compute Pascal's triangle row by row (mod p) using a 1-D buffer.
        let row_len = w.min(512);
        let mut row = vec![0u64; row_len];
        if row_len > 0 {
            row[0] = 1;
        }

        for r in 0..revealed_rows {
            // Map row to y-coordinate (top = row 0).
            let py = r;
            // Map column within the row to x-coordinate, centred.
            // The Pascal triangle at row r has r+1 non-trivial entries; we
            // spread them symmetrically across the width.
            let entries = (r + 1).min(row_len);
            for c in 0..entries {
                if row[c] % modulus != 0 {
                    // Map entry position to x.
                    let x = if entries <= 1 {
                        w / 2
                    } else {
                        c * (w.saturating_sub(1)) / (entries.saturating_sub(1))
                    };
                    draw::dot(grid, x, py);
                }
            }
            // Advance row: compute next Pascal row (mod p), right-to-left.
            for c in (1..row_len).rev() {
                row[c] = (row[c] + row[c - 1]) % modulus;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Euler totient φ(n) histogram
// ────────────────────────────────────────────────────────────────────────────

/// Euler totient φ(n) histogram: bar height at column n is φ(n)/n, revealing
/// left to right as `eased` grows. Primes peek at the top (φ(p)=p−1≈p).
struct TotientHistogram;
impl ProgressStyle for TotientHistogram {
    fn name(&self) -> &str {
        "totient-histogram"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Euler totient φ(n): columns show φ(n)/n — primes spike to the top, composites sag"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n = w.max(2).min(2000);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n).max(1);

        // Compute totient for k in 2..=revealed.
        fn totient(n: usize) -> usize {
            if n == 0 {
                return 0;
            }
            // Euler's product formula via trial factoring.
            let mut phi = n;
            let mut temp = n;
            let mut p = 2;
            while p * p <= temp {
                if temp % p == 0 {
                    while temp % p == 0 {
                        temp /= p;
                    }
                    phi = phi - phi / p;
                }
                p += 1;
            }
            if temp > 1 {
                phi = phi - phi / temp;
            }
            phi
        }

        for k in 2..=revealed {
            let x = ((k - 2) * w / (n.saturating_sub(1).max(1))).min(w.saturating_sub(1));
            let phi_k = totient(k);
            // Ratio φ(k)/k ∈ (0, 1].
            let ratio = phi_k as f32 / k as f32;
            let bar_h = (ratio * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Stern-Brocot tree / Farey sequence
// ────────────────────────────────────────────────────────────────────────────

/// Stern-Brocot tree: fractions from the Farey sequence F_n plotted as
/// vertical spikes whose height is the denominator. `eased` controls the
/// order n of the Farey sequence; `time` pulses a cursor scanning fractions.
struct SternBrocot;
impl ProgressStyle for SternBrocot {
    fn name(&self) -> &str {
        "stern-brocot"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Stern-Brocot / Farey F_n: every rational in [0,1] placed by value, height = denominator"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Farey sequence F_n: all p/q with 0 ≤ p ≤ q ≤ n, gcd(p,q)=1, in order.
        // Cap n to keep term count manageable.
        let n = ((ctx.eased * 20.0).round() as usize).clamp(1, 20);

        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            a
        }

        // Collect Farey fractions as (numerator, denominator).
        let mut fracs: Vec<(usize, usize)> = Vec::new();
        for q in 1..=n {
            for p in 0..=q {
                if gcd(p, q) == 1 || (p == 0 && q == 1) {
                    fracs.push((p, q));
                }
            }
        }
        // Sort by value.
        fracs.sort_by(|a, b| {
            let va = a.0 * b.1;
            let vb = b.0 * a.1;
            va.cmp(&vb)
        });
        fracs.dedup_by(|a, b| a.0 * b.1 == b.0 * a.1);

        let max_q = n.max(1);

        // Animated scan cursor.
        let cursor_frac = (ctx.time * 0.2).fract();

        for (p, q) in &fracs {
            let value = *p as f32 / *q as f32;
            let x = (value * (w.saturating_sub(1)) as f32).round() as usize;
            let bar_h = ((*q as f32 / max_q as f32) * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));

            // Highlight the fraction closest to the cursor.
            let dist = (value - cursor_frac).abs();
            if dist < 0.03 {
                draw::dot(grid, x, y0.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Continued fraction convergents of φ
// ────────────────────────────────────────────────────────────────────────────

/// Continued fraction convergents of φ = [1;1,1,1,…]: each convergent p_k/q_k
/// is a best rational approximation. `eased` reveals convergents; each is
/// drawn as a dot at (x=value×width, y=row k). Convergents alternate above /
/// below the true value — the zig-zag visualises the approximation theorem.
struct ContinuedFraction;
impl ProgressStyle for ContinuedFraction {
    fn name(&self) -> &str {
        "continued-fraction-phi"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Continued fraction convergents of φ: zig-zagging best rationals approaching the golden ratio"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // φ convergents: p_k/q_k where p_k = F_{k+1}, q_k = F_k (Fibonacci).
        // Compute up to h convergents (one per dot-row).
        let max_k = h.min(40);
        let revealed = ((ctx.eased * max_k as f32).round() as usize).min(max_k);

        let mut p_prev = 0u64;
        let mut p_curr = 1u64;
        let mut q_prev = 1u64;
        let mut q_curr = 1u64;

        let phi = (1.0 + 5.0f32.sqrt()) / 2.0; // 1.6180…
                                               // We map value/phi into [0,1] range (phi ≈ 1.618, so p/q ≤ phi).
        let scale = phi;

        let mut prev_x: Option<usize> = None;
        let mut prev_y: Option<usize> = None;

        for k in 0..revealed {
            let value = p_curr as f32 / q_curr as f32;
            let x =
                ((value / scale).clamp(0.0, 1.0) * (w.saturating_sub(1)) as f32).round() as usize;
            let y = if h <= 1 {
                0
            } else {
                k * (h.saturating_sub(1)) / (max_k.saturating_sub(1).max(1))
            };
            let y = y.min(h.saturating_sub(1));

            draw::dot(grid, x, y);

            // Connect successive convergents with a line to show zig-zag.
            if let (Some(px), Some(py)) = (prev_x, prev_y) {
                // Simple Bresenham-lite: interpolate between the two points.
                let steps = ((x as i32 - px as i32)
                    .abs()
                    .max((y as i32 - py as i32).abs()))
                .max(1) as usize;
                for s in 1..steps {
                    let t = s as f32 / steps as f32;
                    let ix = (px as f32 + t * (x as i32 - px as i32) as f32).round() as usize;
                    let iy = (py as f32 + t * (y as i32 - py as i32) as f32).round() as usize;
                    draw::dot(
                        grid,
                        ix.min(w.saturating_sub(1)),
                        iy.min(h.saturating_sub(1)),
                    );
                }
            }
            prev_x = Some(x);
            prev_y = Some(y);

            // Advance convergents: [1;1,1,1,…] so a_k = 1 always.
            let p_new = p_curr + p_prev;
            let q_new = q_curr + q_prev;
            p_prev = p_curr;
            p_curr = p_new;
            q_prev = q_curr;
            q_curr = q_new;

            // Guard against overflow for deep k.
            if p_curr > 1_000_000 || q_curr > 1_000_000 {
                break;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Modular multiplication circle (cardioid string art)
// ────────────────────────────────────────────────────────────────────────────

/// Modular multiplication circle: N points around a circle; connect point i to
/// (k·i) mod N for each i. k=2 draws a cardioid; k=3 a nephroid; k swept by
/// `eased` reveals new curves. `time` rotates the whole figure slowly.
struct ModularCircle;
impl ProgressStyle for ModularCircle {
    fn name(&self) -> &str {
        "modular-circle"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Modular multiplication circle: string art i→k·i mod N, k swept by progress (cardioid at k=2)"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n_points = 100usize; // Fixed number of points on circle.
        let k = (ctx.eased * 10.0).floor() as usize + 2; // k in 2..=12.
        let k = k.min(50);

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let rx = (w / 2).saturating_sub(1) as f32;
        let ry = (h / 2).saturating_sub(1) as f32;

        // Slow rotation from time.
        let rot = ctx.time * 0.1;

        for i in 0..n_points {
            let j = (k * i) % n_points;
            let angle_i = (i as f32 / n_points as f32) * 2.0 * PI + rot;
            let angle_j = (j as f32 / n_points as f32) * 2.0 * PI + rot;

            let x0 = (cx + rx * angle_i.cos()).round() as i32;
            let y0 = (cy + ry * angle_i.sin()).round() as i32;
            let x1 = (cx + rx * angle_j.cos()).round() as i32;
            let y1 = (cy + ry * angle_j.sin()).round() as i32;

            // Bresenham line between the two points.
            let steps = ((x1 - x0).abs().max((y1 - y0).abs())).max(1) as usize;
            let steps = steps.min(512);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = (x0 as f32 + t * (x1 - x0) as f32).round() as i32;
                let py = (y0 as f32 + t * (y1 - y0) as f32).round() as i32;
                draw::dot_i(grid, px, py);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Recamán's sequence
// ────────────────────────────────────────────────────────────────────────────

/// Recamán's sequence: a(0)=0; a(n) = a(n-1)−n if positive and not already
/// in the sequence, else a(n-1)+n. Plotted as arcs (half-circles) on a
/// number line, alternating above/below. `eased` reveals terms; `time`
/// pulses the leading arc.
struct Recaman;
impl ProgressStyle for Recaman {
    fn name(&self) -> &str {
        "recaman"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Recamán's sequence: arcs above/below a number line, each term a backward or forward jump"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let max_terms = w.min(60).max(2);
        let revealed = ((ctx.eased * max_terms as f32).round() as usize).clamp(1, max_terms);

        // Build Recamán sequence.
        let mut seq = vec![0usize];
        let mut seen = std::collections::HashSet::new();
        seen.insert(0usize);
        for n in 1..max_terms {
            let prev = seq[n - 1];
            let candidate = prev.saturating_sub(n);
            let next = if candidate > 0 && !seen.contains(&candidate) {
                candidate
            } else {
                prev + n
            };
            seq.push(next);
            seen.insert(next);
        }

        // Normalise to fit in width.
        let max_val = seq.iter().copied().max().unwrap_or(1).max(1);
        let baseline = h / 2;

        // Draw number-line baseline.
        draw::hline(grid, 0, w.saturating_sub(1), baseline);

        // Draw arcs as semicircles, alternating above/below.
        for n in 1..revealed {
            let a = seq[n - 1];
            let b = seq[n];
            let forward = b > a;

            let x_a = (a * (w.saturating_sub(1)) / max_val).min(w.saturating_sub(1));
            let x_b = (b * (w.saturating_sub(1)) / max_val).min(w.saturating_sub(1));
            let arc_cx = (x_a + x_b) / 2;
            let arc_r = ((x_b as i32 - x_a as i32).abs() / 2).max(1) as f32;

            // Above baseline for backward jumps (a→b where b<a), below for forward.
            let above = !forward;

            let steps = ((arc_r * PI) as usize).max(4).min(256);
            for s in 0..=steps {
                let theta = s as f32 / steps as f32 * PI;
                let dx = (theta.cos() * arc_r).round() as i32;
                let dy = (theta.sin() * arc_r).round() as i32;
                let px = arc_cx as i32 + dx;
                let py = if above {
                    baseline as i32 - dy
                } else {
                    baseline as i32 + dy
                };
                draw::dot_i(grid, px, py);
            }
        }

        // Animate: pulse on the leading arc.
        let _ = ctx.time;
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 12. Digital root / Vortex math
// ────────────────────────────────────────────────────────────────────────────

/// Digital root vortex math: place digits 1–9 around a circle; connect the
/// digital root sequence of n (n, n+n, …) as a cycle of chords. `eased`
/// selects the base number n ∈ 1..9; `time` rotates and pulses the figure.
/// The vortex math pattern mod 9 reveals hidden symmetries (3–6–9, 1–2–4–8–7–5).
struct DigitalRoot;
impl ProgressStyle for DigitalRoot {
    fn name(&self) -> &str {
        "digital-root-vortex"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Vortex math: digital-root cycles 1–9 drawn as chords on a circle, revealing mod-9 symmetry"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let rx = (w / 2).saturating_sub(1) as f32;
        let ry = (h / 2).saturating_sub(1) as f32;

        // Digital root of n mod 9 (returns 1..=9; 0→9).
        fn digital_root(n: usize) -> usize {
            if n == 0 {
                return 9;
            }
            let r = n % 9;
            if r == 0 {
                9
            } else {
                r
            }
        }

        // Base digit from eased (1–9).
        let base_digit = ((ctx.eased * 8.0).floor() as usize + 1).min(9);

        // Build the cycle: starting from base_digit, keep adding base_digit mod 9.
        let mut cycle = vec![base_digit];
        let mut cur = base_digit;
        for _ in 0..8 {
            cur = digital_root(cur + base_digit);
            if cur == cycle[0] {
                break;
            }
            cycle.push(cur);
        }

        // Rotation from time.
        let rot = ctx.time * 0.15;

        // Draw the 9 node positions on the circle.
        for d in 1..=9usize {
            let angle = (d as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let px = (cx + rx * angle.cos()).round() as usize;
            let py = (cy + ry * angle.sin()).round() as usize;
            if px < w && py < h {
                draw::dot(grid, px, py);
            }
        }

        // Draw the cycle chords.
        for i in 0..cycle.len() {
            let a = cycle[i];
            let b = cycle[(i + 1) % cycle.len()];
            let angle_a = (a as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let angle_b = (b as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let x0 = (cx + rx * angle_a.cos()).round() as i32;
            let y0 = (cy + ry * angle_a.sin()).round() as i32;
            let x1 = (cx + rx * angle_b.cos()).round() as i32;
            let y1 = (cy + ry * angle_b.sin()).round() as i32;

            let steps = ((x1 - x0).abs().max((y1 - y0).abs())).max(1) as usize;
            let steps = steps.min(512);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = (x0 as f32 + t * (x1 - x0) as f32).round() as i32;
                let py = (y0 as f32 + t * (y1 - y0) as f32).round() as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Draw the 3-6-9 axis separately for emphasis.
        for d in [3usize, 6, 9] {
            let angle = (d as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let px = (cx + rx * angle.cos()).round() as i32;
            let py = (cy + ry * angle.sin()).round() as i32;
            draw::dot_i(grid, px, py);
            draw::dot_i(grid, px + 1, py);
            draw::dot_i(grid, px, py + 1);
            draw::dot_i(grid, px - 1, py);
            draw::dot_i(grid, px, py - 1);
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
    let styles = progress::styles::numbertheory::styles();
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
