//! `fractal` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O fractal.rs && ./fractal [style-name]
//! ```

const DEFAULT_STYLE: &str = "mandelbrot-escape";

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
    pub mod fractal {
//! Fractal-themed progress bars — hard mathematics rendered in braille dots.
//!
//! Each style implements a distinct fractal or iterated-function-system formula,
//! mapping `ctx.eased` to a reveal / iteration parameter and `ctx.time` to
//! continuous animation (zoom, rotation, c-parameter drift). All coordinate
//! arithmetic is done in `f32`/`i32` and committed via `draw::dot_i` so that
//! negative values from fractal math never cause panics at small grid sizes.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic hash (no external crates)
// ---------------------------------------------------------------------------

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `fractal` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per style, in display order. Each
/// style maps `ctx.eased` to a mathematical reveal parameter (iteration depth,
/// point count, recursion level) and `ctx.time` to continuous animation.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(MandelbrotEscape),
        Box::new(JuliaSet),
        Box::new(SierpinskiTriangle),
        Box::new(KochCurve),
        Box::new(BarnsleyFern),
        Box::new(DragonCurve),
        Box::new(SierpinskiCarpet),
        Box::new(BurningShip),
        Box::new(PythagorasTree),
        Box::new(NewtonFractal),
        Box::new(CantorDust),
        Box::new(LyapunovBar),
    ]
}

// ---------------------------------------------------------------------------
// 1. Mandelbrot escape-time
// ---------------------------------------------------------------------------

struct MandelbrotEscape;
impl ProgressStyle for MandelbrotEscape {
    fn name(&self) -> &str {
        "mandelbrot-escape"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Mandelbrot set rendered as braille: escape-time threshold rises with progress, \
         viewport pans and zooms with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Max iterations scales with eased progress (2..=48).
        let max_iter = (2.0 + ctx.eased * 46.0) as u32;

        // Animate: gentle zoom and pan into the Seahorse Valley area.
        let zoom = 2.5 / (1.0 + ctx.time * 0.05);
        let cx_center = -0.7269 - ctx.time * 0.002;
        let cy_center = 0.1889;

        let aspect = dw as f32 / dh as f32;

        for py in 0..dh {
            for px in 0..dw {
                // Map dot to complex plane.
                let cr = cx_center + (px as f32 / dw as f32 - 0.5) * zoom * aspect;
                let ci = cy_center + (py as f32 / dh as f32 - 0.5) * zoom;

                let mut zr = 0.0f32;
                let mut zi = 0.0f32;
                let mut escaped = false;
                for _ in 0..max_iter {
                    let zr2 = zr * zr;
                    let zi2 = zi * zi;
                    if zr2 + zi2 > 4.0 {
                        escaped = true;
                        break;
                    }
                    let new_zr = zr2 - zi2 + cr;
                    zi = 2.0 * zr * zi + ci;
                    zr = new_zr;
                }

                // Points that escape → lit; interior stays dark (classic look).
                if escaped {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Tint the filled portion by progress.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            for cx in 0..filled_cells.min(cw) {
                let t = if cw <= 1 {
                    0.5
                } else {
                    cx as f32 / (cw - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Julia set  z → z² + c,  c = 0.7885·e^(i·θ)
// ---------------------------------------------------------------------------

struct JuliaSet;
impl ProgressStyle for JuliaSet {
    fn name(&self) -> &str {
        "julia-set"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Julia set with c=0.7885·e^(i·θ), θ animated by time; resolution and iteration \
         depth grow with progress."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let theta = ctx.time * 0.4;
        let cr = 0.7885 * theta.cos();
        let ci = 0.7885 * theta.sin();

        let max_iter = (4.0 + ctx.eased * 44.0) as u32;
        let scale = 1.5;

        for py in 0..dh {
            for px in 0..dw {
                let mut zr = (px as f32 / dw as f32 - 0.5) * scale * 2.0;
                let mut zi = (py as f32 / dh as f32 - 0.5) * scale * 2.0;

                let mut iters = 0u32;
                while iters < max_iter && zr * zr + zi * zi <= 4.0 {
                    let new_zr = zr * zr - zi * zi + cr;
                    zi = 2.0 * zr * zi + ci;
                    zr = new_zr;
                    iters += 1;
                }

                if iters < max_iter {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Palette tint: columns cycle through hue based on progress.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            for cx in 0..cw {
                let t = (cx as f32 / cw as f32 + ctx.time * 0.1).fract();
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Sierpinski triangle — chaos-game IFS
// ---------------------------------------------------------------------------

struct SierpinskiTriangle;
impl ProgressStyle for SierpinskiTriangle {
    fn name(&self) -> &str {
        "sierpinski-triangle"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Sierpinski triangle via the chaos game (random midpoint IFS); point count \
         grows with progress, triangle rotates with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let n_points = (10.0 + ctx.eased * 2990.0) as u32;

        // Three vertices of the triangle, rotated by time.
        let rot = ctx.time * 0.3;
        let vertices: [(f32, f32); 3] = [
            (0.5 + 0.45 * (rot).cos(), 0.5 + 0.45 * (rot).sin()),
            (
                0.5 + 0.45 * (rot + 2.094).cos(),
                0.5 + 0.45 * (rot + 2.094).sin(),
            ),
            (
                0.5 + 0.45 * (rot + 4.189).cos(),
                0.5 + 0.45 * (rot + 4.189).sin(),
            ),
        ];

        // Seed — iterate once without drawing to warm up.
        let mut px = 0.5f32;
        let mut py = 0.5f32;
        for i in 0..n_points + 20 {
            let v = (hash(i.wrapping_mul(1_000_003)) % 3) as usize;
            px = (px + vertices[v].0) * 0.5;
            py = (py + vertices[v].1) * 0.5;
            if i >= 20 {
                let dx = (px * dw as f32) as i32;
                let dy = (py * dh as f32) as i32;
                draw::dot_i(grid, dx, dy);
            }
        }

        // Tint by row.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Koch snowflake curve — L-system recursion
// ---------------------------------------------------------------------------

struct KochCurve;
impl ProgressStyle for KochCurve {
    fn name(&self) -> &str {
        "koch-curve"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Koch snowflake drawn as a dot polyline; L-system recursion depth grows \
         with progress (0..=4), rotated and scaled by time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let depth = (ctx.eased * 4.0).floor() as u32;

        // Collect Koch curve segment endpoints via recursive subdivision.
        fn koch_points(ax: f32, ay: f32, bx: f32, by: f32, depth: u32, pts: &mut Vec<(f32, f32)>) {
            if depth == 0 {
                pts.push((bx, by));
                return;
            }
            let dx = bx - ax;
            let dy = by - ay;
            // p1 = 1/3 of the way
            let p1x = ax + dx / 3.0;
            let p1y = ay + dy / 3.0;
            // p3 = 2/3 of the way
            let p3x = ax + 2.0 * dx / 3.0;
            let p3y = ay + 2.0 * dy / 3.0;
            // p2 = peak of the outward triangle (60° rotation of the 1/3 segment)
            let p2x = p1x + (dx / 3.0) * 0.5 - (dy / 3.0) * 0.866;
            let p2y = p1y + (dy / 3.0) * 0.5 + (dx / 3.0) * 0.866;
            koch_points(ax, ay, p1x, p1y, depth - 1, pts);
            koch_points(p1x, p1y, p2x, p2y, depth - 1, pts);
            koch_points(p2x, p2y, p3x, p3y, depth - 1, pts);
            koch_points(p3x, p3y, bx, by, depth - 1, pts);
        }

        // Three sides of a snowflake, centred in the dot grid.
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (dw.min(dh * 2) as f32 * 0.42).max(1.0);
        let rot = ctx.time * 0.2;

        let tri: Vec<(f32, f32)> = (0..3)
            .map(|i| {
                let a = rot + i as f32 * 2.0 * PI / 3.0 - PI / 2.0;
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect();

        for side in 0..3 {
            let (ax, ay) = tri[side];
            let (bx, by) = tri[(side + 1) % 3];
            let mut pts = vec![(ax, ay)];
            koch_points(ax, ay, bx, by, depth, &mut pts);

            // Draw line segments between consecutive points.
            for w in pts.windows(2) {
                let (x0, y0) = w[0];
                let (x1, y1) = w[1];
                // Bresenham-style plot via parametric steps.
                let steps = ((x1 - x0).abs() + (y1 - y0).abs()).ceil() as u32 + 1;
                let steps = steps.max(1);
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let px = x0 + t * (x1 - x0);
                    let py = y0 + t * (y1 - y0);
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Tint.
        let (cw, ch) = grid.dimensions();
        for cy_cell in 0..ch {
            let t = cy_cell as f32 / ch.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                cw.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Barnsley fern — IFS with 4 affine maps
// ---------------------------------------------------------------------------

struct BarnsleyFern;
impl ProgressStyle for BarnsleyFern {
    fn name(&self) -> &str {
        "barnsley-fern"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Barnsley fern IFS (4 affine maps, exact original coefficients); point count \
         and detail scale with progress; sways gently with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let n_pts = (20.0 + ctx.eased * 2980.0) as u32;

        // Classic Barnsley fern IFS coefficients:
        //  f1: stem        probability 0.01
        //  f2: large leaf  probability 0.85
        //  f3: left leaflet  probability 0.07
        //  f4: right leaflet probability 0.07
        // Each: [a, b, c, d, e, f] for x' = ax+by+e, y' = cx+dy+f
        let maps: [[f32; 6]; 4] = [
            [0.0, 0.0, 0.0, 0.16, 0.0, 0.0],      // stem
            [0.85, 0.04, -0.04, 0.85, 0.0, 1.6],  // large leaf
            [0.20, -0.26, 0.23, 0.22, 0.0, 1.6],  // left leaflet
            [-0.15, 0.28, 0.26, 0.24, 0.0, 0.44], // right leaflet
        ];
        // Cumulative probability thresholds (×1000 for integer hash).
        let thresholds = [10u32, 860, 930, 1000];

        // Gentle sway: perturb e coefficient of map 2 and 3.
        let sway = (ctx.time * 0.5).sin() * 0.04;

        let mut x = 0.0f32;
        let mut y = 0.0f32;

        for i in 0..n_pts + 20 {
            let r = hash(i.wrapping_mul(999_983)) % 1000;
            let map_idx = if r < thresholds[0] {
                0
            } else if r < thresholds[1] {
                1
            } else if r < thresholds[2] {
                2
            } else {
                3
            };

            let m = maps[map_idx];
            let new_x = m[0] * x + m[1] * y + m[4] + if map_idx >= 2 { sway } else { 0.0 };
            let new_y = m[2] * x + m[3] * y + m[5];
            x = new_x;
            y = new_y;

            if i >= 20 {
                // Fern lives in x ∈ [-2.182, 2.6558], y ∈ [0, 9.9983].
                let px = ((x + 2.182) / 4.8378 * dw as f32) as i32;
                let py = ((1.0 - y / 9.9983) * dh as f32) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Green-ish tint via palette.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(
                grid,
                cy,
                0,
                cw.saturating_sub(1),
                ctx.palette.sample(1.0 - t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Dragon curve — paper-folding L-system
// ---------------------------------------------------------------------------

struct DragonCurve;
impl ProgressStyle for DragonCurve {
    fn name(&self) -> &str {
        "dragon-curve"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Dragon curve drawn by paper-folding construction; reveal length grows with \
         progress (up to 12 folds), rotates with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Number of folds determines curve length = 2^n segments.
        let max_folds: u32 = 12;
        let folds = (ctx.eased * max_folds as f32).floor() as u32;
        let folds = folds.min(max_folds);
        let n_segs = 1u32 << folds; // 2^folds

        // Generate turn sequence: for segment k, turn = fold_direction(k).
        // The k-th turn of the dragon curve (0-indexed):
        // T(k) = bit at position (highest power of 2 dividing k+1) shifted odd/even.
        // Equivalently: let k' = (k+1). Turn right if ((k' >> trailing_zeros(k')) >> 1) & 1 == 0.
        fn dragon_turn(k: u32) -> bool {
            // k is 1-indexed segment.
            let tz = k.trailing_zeros();
            ((k >> tz) >> 1) & 1 == 0
        }

        // Walk the curve.
        // Direction: 0=right, 1=up, 2=left, 3=down.
        let rot_offset = (ctx.time * 0.25) as i32; // integer quarter-turns
        let start_dir: i32 = rot_offset.rem_euclid(4);

        // Collect all segment start points to find bounding box for centering.
        let step = 1i32;
        let dx = [step, 0, -step, 0];
        let dy = [0, -step, 0, step];

        let mut cx_i: i32 = 0;
        let mut cy_i: i32 = 0;
        let mut dir: i32 = start_dir;
        let mut pts: Vec<(i32, i32)> = Vec::with_capacity((n_segs + 1) as usize);
        pts.push((cx_i, cy_i));

        for k in 1..=n_segs {
            cx_i += dx[dir as usize];
            cy_i += dy[dir as usize];
            pts.push((cx_i, cy_i));
            if k < n_segs {
                let turn = dragon_turn(k);
                dir = if turn {
                    (dir + 1).rem_euclid(4)
                } else {
                    (dir + 3).rem_euclid(4)
                };
            }
        }

        // Bounding box for centering.
        let min_x = pts.iter().map(|p| p.0).min().unwrap_or(0);
        let max_x = pts.iter().map(|p| p.0).max().unwrap_or(0);
        let min_y = pts.iter().map(|p| p.1).min().unwrap_or(0);
        let max_y = pts.iter().map(|p| p.1).max().unwrap_or(0);
        let span_x = (max_x - min_x).max(1);
        let span_y = (max_y - min_y).max(1);

        for p in &pts {
            let px = (p.0 - min_x) as f32 / span_x as f32 * (dw as f32 - 1.0);
            let py = (p.1 - min_y) as f32 / span_y as f32 * (dh as f32 - 1.0);
            draw::dot_i(grid, px as i32, py as i32);
        }

        // Tint horizontally.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Sierpinski carpet — recursive square removal
// ---------------------------------------------------------------------------

struct SierpinskiCarpet;
impl ProgressStyle for SierpinskiCarpet {
    fn name(&self) -> &str {
        "sierpinski-carpet"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Sierpinski carpet: each dot is tested via the 9-ary digit rule (x&y in base 3 \
         for any digit == 1 → hole); depth driven by progress."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Depth controls which power-of-3 we map to.
        // At depth d we check d digits in base 3.
        let max_depth = (ctx.eased * 5.0).ceil() as u32;
        let max_depth = max_depth.max(1);

        // Scale: use a 3^max_depth × 3^max_depth logical grid.
        let pow3 = 3u32.pow(max_depth);

        // Animate: slow drift in mapping origin.
        let off_x = (ctx.time * 0.4).sin() * 0.08;
        let off_y = (ctx.time * 0.3).cos() * 0.08;

        for py in 0..dh {
            for px in 0..dw {
                // Map dot to carpet coordinates in [0, pow3).
                let fx = ((px as f32 / dw as f32 + off_x).rem_euclid(1.0) * pow3 as f32) as u32;
                let fy = ((py as f32 / dh as f32 + off_y).rem_euclid(1.0) * pow3 as f32) as u32;

                // Sierpinski carpet rule: if any base-3 digit of x AND y both == 1 → hole.
                let mut in_hole = false;
                let mut rx = fx;
                let mut ry = fy;
                for _ in 0..max_depth {
                    if rx % 3 == 1 && ry % 3 == 1 {
                        in_hole = true;
                        break;
                    }
                    rx /= 3;
                    ry /= 3;
                }

                if !in_hole {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Tint.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            for cx in 0..filled_cells.min(cw) {
                let t = cx as f32 / cw.max(1) as f32;
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Burning Ship fractal  — |Re(z)|, |Im(z)| before squaring
// ---------------------------------------------------------------------------

struct BurningShip;
impl ProgressStyle for BurningShip {
    fn name(&self) -> &str {
        "burning-ship"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Burning Ship fractal (z → (|Re z|+i|Im z|)² + c); its jagged, ship-like \
         silhouette burns across the bar as progress rises."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let max_iter = (3.0 + ctx.eased * 45.0) as u32;

        // Classic Burning Ship viewport: Re ∈ [-2.5, 1.5], Im ∈ [-2, 0.5].
        // Animate by slowly drifting.
        let drift = ctx.time * 0.015;
        let re_min = -2.5 + drift.sin() * 0.2;
        let re_max = 1.5 + drift.cos() * 0.1;
        let im_min = -2.0 + (drift * 0.7).sin() * 0.15;
        let im_max = 0.5 + (drift * 0.5).cos() * 0.1;

        for py in 0..dh {
            for px in 0..dw {
                let cr = re_min + (px as f32 / dw as f32) * (re_max - re_min);
                let ci = im_min + (py as f32 / dh as f32) * (im_max - im_min);

                let mut zr = 0.0f32;
                let mut zi = 0.0f32;
                let mut escaped = false;
                for _ in 0..max_iter {
                    let zr2 = zr * zr;
                    let zi2 = zi * zi;
                    if zr2 + zi2 > 4.0 {
                        escaped = true;
                        break;
                    }
                    // Key difference: absolute values before squaring.
                    let new_zr = zr2 - zi2 + cr;
                    zi = 2.0 * zr.abs() * zi.abs() + ci;
                    zr = new_zr;
                }

                if escaped {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Pythagoras tree — recursive binary squares
// ---------------------------------------------------------------------------

struct PythagorasTree;
impl ProgressStyle for PythagorasTree {
    fn name(&self) -> &str {
        "pythagoras-tree"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Pythagoras tree: a square sprouts two smaller squares at a time-animated \
         split angle; branch count grows with progress (depth 0..=7)."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let max_depth = (ctx.eased * 7.0).floor() as u32;
        // Animate split angle α between 20° and 70°.
        let alpha = 0.349 + (ctx.time * 0.2).sin().abs() * 0.524; // 20°..50° in radians

        // Draw a square given its bottom-left corner vector and direction.
        // We represent the square by two corners of its base edge: (x1,y1)-(x2,y2),
        // growing upward (away from the viewer, i.e. toward smaller y in screen coords).
        fn draw_square(grid: &mut BrailleGrid, x1: f32, y1: f32, x2: f32, y2: f32) {
            // Edge vector.
            let ex = x2 - x1;
            let ey = y2 - y1;
            // Perpendicular pointing up on screen (y grows downward): (ey, -ex).
            let x3 = x2 + ey;
            let y3 = y2 - ex;
            let x4 = x1 + ey;
            let y4 = y1 - ex;

            // Draw 4 edges via parametric steps.
            let corners = [(x1, y1), (x2, y2), (x3, y3), (x4, y4), (x1, y1)];
            for w in corners.windows(2) {
                let (ax, ay) = w[0];
                let (bx, by) = w[1];
                let steps = ((bx - ax).abs() + (by - ay).abs()).ceil() as i32 + 1;
                let steps = steps.max(1);
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let px = ax + t * (bx - ax);
                    let py = ay + t * (by - ay);
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Recursively build the tree.
        fn tree(
            grid: &mut BrailleGrid,
            x1: f32,
            y1: f32,
            x2: f32,
            y2: f32,
            alpha: f32,
            depth: u32,
        ) {
            if depth == 0 {
                return;
            }
            draw_square(grid, x1, y1, x2, y2);

            // Top edge of the square (same screen-up perpendicular as above).
            let ex = x2 - x1;
            let ey = y2 - y1;
            let tx1 = x1 + ey;
            let ty1 = y1 - ex;
            let tx2 = x2 + ey;
            let ty2 = y2 - ex;

            // Split point on the top edge at angle alpha.
            let ca = alpha.cos();
            let sa = alpha.sin();
            let ex2 = tx2 - tx1;
            let ey2 = ty2 - ty1;
            let len = (ex2 * ex2 + ey2 * ey2).sqrt().max(1e-6);
            let ux = ex2 / len;
            let uy = ey2 / len;
            // Left branch base: tx1 to apex, rotated by -alpha so the apex
            // bulges above the top edge (screen-up) instead of folding down.
            let left_size = len * ca;
            let apex_x = tx1 + ux * left_size * ca + uy * left_size * sa;
            let apex_y = ty1 + uy * left_size * ca - ux * left_size * sa;

            // Left child square.
            tree(grid, tx1, ty1, apex_x, apex_y, alpha, depth - 1);
            // Right child square.
            tree(grid, apex_x, apex_y, tx2, ty2, alpha, depth - 1);
        }

        // Root square: centred at the bottom, capped by grid height so the
        // crown (roughly 3x the root side) still fits short bar grids.
        let sq_w = (dw as f32 * 0.28).min(dh as f32 * 0.34).max(2.0);
        let base_y = dh as f32 - 1.0;
        let base_x = dw as f32 / 2.0 - sq_w / 2.0;
        tree(
            grid,
            base_x,
            base_y,
            base_x + sq_w,
            base_y,
            alpha,
            max_depth + 1,
        );

        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.max(1) as f32; // brighter at top (crown)
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Newton fractal — basins of attraction for z³ - 1 = 0
// ---------------------------------------------------------------------------

struct NewtonFractal;
impl ProgressStyle for NewtonFractal {
    fn name(&self) -> &str {
        "newton-fractal"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Newton fractal for z³−1=0: three root basins painted with palette colors; \
         iteration count threshold rises with progress, domain rotates with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let max_iter = (2.0 + ctx.eased * 30.0) as u32;
        let tol2 = 1e-4f32;

        // Three cube roots of unity.
        let roots = [(1.0f32, 0.0f32), (-0.5, 0.866_025_4), (-0.5, -0.866_025_4)];

        let scale = 2.0f32 / (1.0 + ctx.time * 0.04);
        let theta = ctx.time * 0.15;
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        for py in 0..dh {
            for px in 0..dw {
                // Map to complex plane, then rotate.
                let raw_r = (px as f32 / dw as f32 - 0.5) * scale * 2.0;
                let raw_i = (py as f32 / dh as f32 - 0.5) * scale * 2.0;
                let mut zr = raw_r * cos_t - raw_i * sin_t;
                let mut zi = raw_r * sin_t + raw_i * cos_t;

                // Newton iteration: z ← z - f(z)/f'(z) = z - (z³-1)/(3z²)
                let mut root_id = 0usize;
                let mut converged = false;
                for _ in 0..max_iter {
                    let zr2 = zr * zr;
                    let zi2 = zi * zi;
                    // z³ = z² * z
                    let z3r = (zr2 - zi2) * zr - 2.0 * zr * zi * zi;
                    let z3i = (zr2 - zi2) * zi + 2.0 * zr * zi * zr;
                    // 3z²
                    let d3r = 3.0 * (zr2 - zi2);
                    let d3i = 6.0 * zr * zi;
                    let d_mag2 = d3r * d3r + d3i * d3i;
                    if d_mag2 < 1e-10 {
                        break;
                    }
                    // (z³-1) / (3z²): numerator real and imag.
                    let nr = z3r - 1.0;
                    let ni = z3i;
                    // Complex division (nr + i·ni) / (d3r + i·d3i).
                    let qr = (nr * d3r + ni * d3i) / d_mag2;
                    let qi = (ni * d3r - nr * d3i) / d_mag2;
                    zr -= qr;
                    zi -= qi;

                    // Check convergence to each root.
                    for (rid, &(rr, ri)) in roots.iter().enumerate() {
                        let dr = zr - rr;
                        let di = zi - ri;
                        if dr * dr + di * di < tol2 {
                            root_id = rid;
                            converged = true;
                            break;
                        }
                    }
                    if converged {
                        break;
                    }
                }

                if converged {
                    // Color by which root basin we landed in.
                    let t = root_id as f32 / 2.0;
                    let (cw, ch) = grid.dimensions();
                    let cell_x = (px / 2).min(cw.saturating_sub(1));
                    let cell_y = (py / 4).min(ch.saturating_sub(1));
                    draw::dot_i(grid, px as i32, py as i32);
                    draw::tint_row(grid, cell_y, cell_x, cell_x, ctx.palette.sample(t));
                }
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Cantor dust — iterated middle-third removal cascade
// ---------------------------------------------------------------------------

struct CantorDust;
impl ProgressStyle for CantorDust {
    fn name(&self) -> &str {
        "cantor-dust"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Cantor dust: 2-D middle-third removal across rows; each row is a deeper \
         level of the Cantor set, revealing from top to bottom with progress."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // How many rows to reveal.
        let revealed_rows = ((ctx.eased * dh as f32).ceil() as usize).min(dh);

        for py in 0..revealed_rows {
            // Row depth: maps row to a Cantor iteration level (0..=7).
            let level = (py as f32 / dh as f32 * 7.0).floor() as u32;
            let pow3 = 3u32.pow(level.min(7));

            // Animate: slow horizontal drift.
            let drift = (ctx.time * 0.3 + py as f32 * 0.05).sin() * 0.1;

            for px in 0..dw {
                // Map x to [0, pow3) with drift.
                let fx = ((px as f32 / dw as f32 + drift).rem_euclid(1.0) * pow3 as f32) as u32;
                // Cantor: point is IN the set iff no base-3 digit == 1.
                let mut in_cantor = true;
                let mut rx = fx;
                for _ in 0..level {
                    if rx % 3 == 1 {
                        in_cantor = false;
                        break;
                    }
                    rx /= 3;
                }
                if in_cantor {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Tint rows.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Lyapunov bar — stability exponent rendered as fill intensity
// ---------------------------------------------------------------------------

struct LyapunovBar;
impl ProgressStyle for LyapunovBar {
    fn name(&self) -> &str {
        "lyapunov-bar"
    }
    fn theme(&self) -> &str {
        "fractal"
    }
    fn describe(&self) -> &str {
        "Lyapunov exponent landscape for the logistic map sequence AABB…; each column \
         is a parameter r swept across [2.5, 4.0], lit where the exponent is negative \
         (stable). Progress raises the iteration count; time scrolls the AB sequence phase."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let n_iter = (8.0 + ctx.eased * 92.0) as u32;
        let warmup = 20u32;

        // Sequence AABB (phase scrolled by time).
        // We use a 4-symbol repeating sequence; time shifts which symbol we start on.
        let phase = (ctx.time * 0.5) as u32;

        for px in 0..dw {
            // Map column to parameter a ∈ [2.5, 4.0] and b ∈ [2.5, 4.0] (diagonal sweep).
            let t_col = px as f32 / dw as f32;
            let ra = 2.5 + t_col * 1.5;
            let rb = 2.5 + (1.0 - t_col) * 1.5;

            // Compute Lyapunov exponent.
            let mut x = 0.5f32;
            let mut lam = 0.0f32;
            let seq_len = 4u32;

            // Warmup.
            for i in 0..warmup {
                let r = if (i + phase) % seq_len < 2 { ra } else { rb };
                x = r * x * (1.0 - x);
            }
            // Measure.
            for i in 0..n_iter {
                let r = if (i + phase) % seq_len < 2 { ra } else { rb };
                x = r * x * (1.0 - x);
                let deriv = (r * (1.0 - 2.0 * x)).abs().max(1e-10);
                lam += deriv.ln();
            }
            lam /= n_iter as f32;

            // Negative exponent → stable (chaos-free) → draw column.
            // Positive exponent → chaotic → height proportional to |λ|.
            let col_fill_frac = if lam < 0.0 {
                1.0f32 // fully lit: stable region
            } else {
                (1.0 - (lam / 2.0).min(1.0)).max(0.0) // partial: chaotic
            };

            let col_h = (col_fill_frac * dh as f32).round() as usize;
            let y0 = dh.saturating_sub(col_h);
            for py in y0..dh {
                draw::dot_i(grid, px as i32, py as i32);
            }
        }

        // Tint: palette maps column to hue.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            for cx in 0..cw {
                let t = cx as f32 / cw.max(1) as f32;
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
    let styles = progress::styles::fractal::styles();
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
