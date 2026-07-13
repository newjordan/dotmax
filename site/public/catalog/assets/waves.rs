//! `waves` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O waves.rs && ./waves [style-name]
//! ```

const DEFAULT_STYLE: &str = "fourier-square";

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
    pub mod waves {
//! Fourier / wave-synthesis progress bars for dotmax.
//!
//! Every bar in this theme is driven by real harmonic mathematics — no faked
//! wiggle, no cheap sine trickery. `ctx.eased` controls how many harmonics are
//! revealed, what amplitude is reached, or how far along a series we sit.
//! `ctx.time` advances the phase so every bar animates even at a fixed progress
//! value.
//!
//! ## Styles (11 total)
//!
//! | Name | Math |
//! |------|------|
//! | `fourier-square` | Gibbs-ringing square via `Σ sin((2k-1)x)/(2k-1)` |
//! | `fourier-sawtooth` | Sawtooth via `Σ (-1)^(k+1) sin(kx)/k` |
//! | `fourier-triangle` | Triangle via `Σ (-1)^k sin((2k+1)x)/(2k+1)²` |
//! | `epicycle` | Epicycle chain, vectors + traced path |
//! | `lissajous` | Lissajous figure, δ swept by `eased` |
//! | `standing-wave` | Standing wave, mode = 1+floor(eased·6) |
//! | `interference` | Two-source interference, constructive/destructive bands |
//! | `chladni` | Chladni nodal lines `cos(nπx)cos(mπy)−cos(mπx)cos(nπy)=0` |
//! | `beat-frequency` | Beat envelope `sin(f1·x)+sin(f2·x)`, Δf driven by eased |
//! | `wave-packet` | Gaussian-modulated travelling sinusoid |
//! | `spectrum` | Synthetic Fourier spectrum that fills with eased |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `waves` theme.
///
/// Returns 11 distinct bars, each implementing real harmonic mathematics and
/// safe to render from a 1×1 cell grid up to 80×8 or larger.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(FourierSquare),
        Box::new(FourierSawtooth),
        Box::new(FourierTriangle),
        Box::new(Epicycle),
        Box::new(Lissajous),
        Box::new(StandingWave),
        Box::new(Interference),
        Box::new(Chladni),
        Box::new(BeatFrequency),
        Box::new(WavePacket),
        Box::new(Spectrum),
    ]
}

// ---------------------------------------------------------------------------
// Helper: clamp a float y-coordinate into dot rows [0, h)
// ---------------------------------------------------------------------------
#[inline]
fn y_to_dot(norm: f32, h: usize) -> i32 {
    // norm in [-1, 1] → dot row in [0, h)
    let row = ((1.0 - (norm * 0.5 + 0.5)) * h as f32) as i32;
    row.clamp(0, h as i32 - 1)
}

// ---------------------------------------------------------------------------
// 1. Fourier square wave
//    y = (4/π) Σ_{k=1}^{N} sin((2k-1)·θ) / (2k-1)
//    N = 1 + floor(eased · 12)
//    Watch the Gibbs ear appear at N≥3 and sharpen toward ~9% overshoot.
// ---------------------------------------------------------------------------
struct FourierSquare;
impl ProgressStyle for FourierSquare {
    fn name(&self) -> &str {
        "fourier-square"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Fourier square-wave synthesis: Gibbs ringing grows visible as harmonics unlock with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let harmonics = 1 + (ctx.eased * 12.0).floor() as usize;
        let phase = ctx.time * 2.0 * PI * 0.3; // slow rightward travel

        // Draw a thin baseline
        let base = h / 2;
        draw::hline(grid, 0, w.saturating_sub(1), base);

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let theta = (xi as f32 / w as f32) * 4.0 * PI + phase;
            let mut val: f32 = 0.0;
            for k in 1..=harmonics {
                let n = (2 * k - 1) as f32;
                val += (n * theta).sin() / n;
            }
            val *= 4.0 / PI;
            // val ∈ roughly [-1.18, 1.18] due to Gibbs — clamp gently for vis
            let val_n = val.clamp(-1.0, 1.0);
            let dy = y_to_dot(val_n, h);
            draw::dot_i(grid, xi as i32, dy);
            // Connect consecutive dots vertically to avoid gaps
            if let Some(py) = prev_y {
                let lo = py.min(dy);
                let hi = py.max(dy);
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Tint: colour ramps from start to end across the filled region
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if filled_cells <= 1 {
                0.5
            } else {
                cx as f32 / (filled_cells - 1) as f32
            };
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Fourier sawtooth
//    y = (2/π) Σ_{k=1}^{N} (-1)^{k+1} sin(k·θ) / k
// ---------------------------------------------------------------------------
struct FourierSawtooth;
impl ProgressStyle for FourierSawtooth {
    fn name(&self) -> &str {
        "fourier-sawtooth"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Fourier sawtooth synthesis: each harmonic adds a finer diagonal ramp until the teeth appear"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let harmonics = 1 + (ctx.eased * 11.0).floor() as usize;
        let phase = ctx.time * 2.0 * PI * 0.25;

        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.45).max(1.0);

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let theta = (xi as f32 / w as f32) * 4.0 * PI + phase;
            let mut val: f32 = 0.0;
            for k in 1..=harmonics {
                let kf = k as f32;
                let sign = if k % 2 == 1 { 1.0_f32 } else { -1.0_f32 };
                val += sign * (kf * theta).sin() / kf;
            }
            val *= 2.0 / PI;
            let dy = (mid - (val * amp) as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let (lo, hi) = (py.min(dy), py.max(dy));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Subtle leading-edge colour
        let (cw, ch) = grid.dimensions();
        let head_cell = (ctx.eased * cw as f32).round() as usize;
        for cy in 0..ch {
            let col = ctx.palette.sample(ctx.eased);
            draw::tint_row(grid, cy, 0, head_cell.min(cw.saturating_sub(1)), col);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Fourier triangle wave
//    y = (8/π²) Σ_{k=0}^{N} (-1)^k sin((2k+1)·θ) / (2k+1)²
// ---------------------------------------------------------------------------
struct FourierTriangle;
impl ProgressStyle for FourierTriangle {
    fn name(&self) -> &str {
        "fourier-triangle"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Fourier triangle-wave: smoother convergence than square/sawtooth — peaks sharpen gradually"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let harmonics = 1 + (ctx.eased * 10.0).floor() as usize;
        let phase = ctx.time * 2.0 * PI * 0.2;

        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let theta = (xi as f32 / w as f32) * 4.0 * PI + phase;
            let mut val: f32 = 0.0;
            for k in 0..harmonics {
                let kf = k as f32;
                let n = 2.0 * kf + 1.0;
                let sign = if k % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
                val += sign * (n * theta).sin() / (n * n);
            }
            val *= 8.0 / (PI * PI);
            let dy = (mid - (val * amp) as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let (lo, hi) = (py.min(dy), py.max(dy));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Epicycle Fourier drawing
//    Chain of N rotating vectors (epicycles); each vector k has radius 1/k and
//    angular velocity k·ω. Their tip traces a path that converges to a square
//    wave as N grows. eased controls N = 1..=8, time drives ω.
// ---------------------------------------------------------------------------
struct Epicycle;
impl ProgressStyle for Epicycle {
    fn name(&self) -> &str {
        "epicycle"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Epicycle chain: N rotating vectors sum to trace a square-wave path; watch the circles spin"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let n_vec = 1 + (ctx.eased * 7.0).floor() as usize; // 1..=8
        let omega = ctx.time * 1.8; // base angular speed (rad/s)

        // Origin at left-centre; epicycles stack rightward conceptually but
        // we render the tip path across the full width.
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let r_base = (h as f32 * 0.35).max(1.0);

        // Draw the path traced by the tip across many phase samples
        let steps = w * 2;
        let mut prev: Option<(i32, i32)> = None;
        for si in 0..steps {
            let t = si as f32 / steps as f32; // [0, 1)
            let phase_offset = t * 2.0 * PI;
            let mut tip_x = cx;
            let mut tip_y = cy;
            for k in 1..=n_vec {
                let kf = k as f32;
                let radius = r_base / kf;
                let angle = (2 * k - 1) as f32 * (omega + phase_offset);
                tip_x += radius * angle.cos();
                tip_y += radius * angle.sin();
            }
            // Map tip_x across bar width (re-map from cx±r_base to [0,w))
            let px = ((tip_x / w as f32) * w as f32) as i32;
            let py = tip_y as i32;
            draw::dot_i(grid, px, py);
            if let Some((lx, ly)) = prev {
                // Connect with Bresenham-style horizontal smear
                let dx = (px - lx).abs();
                if dx > 0 {
                    for step in 0..=dx {
                        let ix = lx + (px - lx) * step / dx.max(1);
                        let iy = ly + (py - ly) * step / dx.max(1);
                        draw::dot_i(grid, ix, iy);
                    }
                }
                draw::dot_i(grid, px, py);
            }
            prev = Some((px, py));
        }

        // Draw the outermost circle outline (largest epicycle) at current time
        let angle0 = omega;
        let r0 = r_base;
        let circle_pts = 48usize;
        for i in 0..circle_pts {
            let a = angle0 + (i as f32 / circle_pts as f32) * 2.0 * PI;
            let px = (cx + r0 * a.cos()) as i32;
            let py = (cy + r0 * a.sin()) as i32;
            draw::dot_i(grid, px, py);
        }

        // Draw the arm from centre to tip at current phase (static snapshot)
        {
            let mut tip_x = cx;
            let mut tip_y = cy;
            for k in 1..=n_vec {
                let kf = k as f32;
                let radius = r_base / kf;
                let angle = (2 * k - 1) as f32 * omega;
                let nx = tip_x + radius * angle.cos();
                let ny = tip_y + radius * angle.sin();
                // Draw arm segment
                let steps2 = (radius.max(1.0) as usize).max(2);
                for s in 0..=steps2 {
                    let frac = s as f32 / steps2 as f32;
                    let ax = (tip_x + (nx - tip_x) * frac) as i32;
                    let ay = (tip_y + (ny - tip_y) * frac) as i32;
                    draw::dot_i(grid, ax, ay);
                }
                tip_x = nx;
                tip_y = ny;
            }
        }

        // Colour the bar by eased
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx2 in 0..filled.min(cw) {
            let t = cx2 as f32 / cw as f32;
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Lissajous figure
//    x = sin(a·τ + δ),  y = sin(b·τ)
//    a=3, b=2 (3:2 ratio);  δ = eased·2π so phase sweep reveals the knot.
//    τ animated by time.
// ---------------------------------------------------------------------------
struct Lissajous;
impl ProgressStyle for Lissajous {
    fn name(&self) -> &str {
        "lissajous"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Lissajous figure 3:2 — phase delta sweeps with progress, time rotates the knot in place"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let cx = (w as f32 - 1.0) / 2.0;
        let cy = (h as f32 - 1.0) / 2.0;
        let rx = cx * 0.92;
        let ry = cy * 0.92;

        let a = 3.0_f32;
        let b = 2.0_f32;
        let delta = ctx.eased * 2.0 * PI;
        let tau_off = ctx.time * 0.5; // slow rotation of the whole figure

        let steps = (w * h * 2).max(256);
        let period = 2.0 * PI; // one full Lissajous period in τ
        let mut prev: Option<(i32, i32)> = None;
        for si in 0..steps {
            let tau = (si as f32 / steps as f32) * period + tau_off;
            let lx = rx * (a * tau + delta).sin();
            let ly = ry * (b * tau).sin();
            let px = (cx + lx) as i32;
            let py = (cy + ly) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ox, oy)) = prev {
                // Fill gaps with an interpolating walk
                let steps2 = (((px - ox).abs() + (py - oy).abs()) as usize).max(1);
                for s in 1..steps2 {
                    let f = s as f32 / steps2 as f32;
                    let ix = (ox as f32 + (px - ox) as f32 * f) as i32;
                    let iy = (oy as f32 + (py - oy) as f32 * f) as i32;
                    draw::dot_i(grid, ix, iy);
                }
            }
            prev = Some((px, py));
        }

        // Tint: gradient across the figure, keyed by eased
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx2 in 0..filled.min(cw) {
            let t = cx2 as f32 / cw as f32;
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Standing wave
//    y(x, t) = 2A · sin(k·x) · cos(ω·t)
//    mode number n = 1 + floor(eased·6), so k = n·π/L
//    Nodes are fixed; antinodes breathe with cos(ωt).
// ---------------------------------------------------------------------------
struct StandingWave;
impl ProgressStyle for StandingWave {
    fn name(&self) -> &str {
        "standing-wave"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Standing wave: fixed nodes, breathing antinodes — mode unlocks as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mode = 1 + (ctx.eased * 6.0).floor() as usize; // 1..=7
        let k = mode as f32 * PI / w as f32; // wavenumber
        let omega = 2.5 * PI; // angular freq (vis only)
        let amp = h as f32 * 0.40;
        let mid = (h / 2) as i32;

        // Draw baseline
        draw::hline(grid, 0, w.saturating_sub(1), mid as usize);

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let xf = xi as f32;
            let val = 2.0 * amp * (k * xf).sin() * (omega * ctx.time).cos();
            let dy = (mid - val as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let (lo, hi) = (py.min(dy), py.max(dy));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Mark node positions with tiny vertical strokes
        for n in 0..=mode {
            let node_x = (n as f32 / mode as f32 * w as f32) as usize;
            if node_x < w {
                let tick = (h / 8).max(1);
                let y0 = mid as usize;
                draw::vline(
                    grid,
                    node_x,
                    y0.saturating_sub(tick),
                    (y0 + tick).min(h - 1),
                );
            }
        }

        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Two-source interference
//    Two point sources at x=0 and x=L produce circular waves.
//    Intensity ∝ (sin(k·r1 − ω·t) + sin(k·r2 − ω·t))²
//    Lit if intensity > threshold → constructive/destructive bands.
// ---------------------------------------------------------------------------
struct Interference;
impl ProgressStyle for Interference {
    fn name(&self) -> &str {
        "interference"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Two-source interference: constructive/destructive fringe bands sweep with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        // Sources: left-centre and right-centre, separation driven by eased
        let sep = (ctx.eased * wf * 0.6 + wf * 0.1).min(wf - 1.0);
        let src1_x = (wf / 2.0 - sep / 2.0).max(0.0);
        let src2_x = (wf / 2.0 + sep / 2.0).min(wf - 1.0);
        let src_y = hf / 2.0;

        let lambda = wf / 4.0; // wavelength in dots
        let k = 2.0 * PI / lambda.max(1.0);
        let omega = 2.0 * PI * 1.2;
        let threshold = 0.5_f32;

        for yi in 0..h {
            let yf = yi as f32;
            for xi in 0..w {
                let xf = xi as f32;
                let r1 = ((xf - src1_x).powi(2) + (yf - src_y).powi(2)).sqrt();
                let r2 = ((xf - src2_x).powi(2) + (yf - src_y).powi(2)).sqrt();
                let s1 = (k * r1 - omega * ctx.time).sin();
                let s2 = (k * r2 - omega * ctx.time).sin();
                let intensity = (s1 + s2) * 0.5; // ∈ [-1, 1]
                if intensity.abs() > threshold {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Chladni plate pattern
//    Nodal lines defined by: cos(n·π·x)·cos(m·π·y) − cos(m·π·x)·cos(n·π·y) ≈ 0
//    (n, m) chosen by eased: eased selects among (1,2),(1,3),(2,3),(2,5),(3,4).
//    Points near the nodal lines are lit.
// ---------------------------------------------------------------------------
struct Chladni;
impl ProgressStyle for Chladni {
    fn name(&self) -> &str {
        "chladni"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Chladni nodal lines: sand settles on vibration nodes — pattern changes with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Mode table: (n, m) pairs
        let modes: [(f32, f32); 5] = [(1.0, 2.0), (1.0, 3.0), (2.0, 3.0), (2.0, 5.0), (3.0, 4.0)];
        let idx = (ctx.eased * 4.999) as usize;
        let (n, m) = modes[idx.min(4)];

        // Animate: slow rotation of effective n/m by blending adjacent modes
        let blend = (ctx.time * 0.15).sin() * 0.08;
        let n = n + blend;
        let m = m - blend;

        let threshold = 0.25_f32;

        for yi in 0..h {
            let yf = yi as f32 / h as f32; // [0,1]
            for xi in 0..w {
                let xf = xi as f32 / w as f32; // [0,1]
                let val = (n * PI * xf).cos() * (m * PI * yf).cos()
                    - (m * PI * xf).cos() * (n * PI * yf).cos();
                if val.abs() < threshold {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Beat frequency
//    y(x) = sin(f1·x) + sin(f2·x)
//    f1 = base freq, f2 = f1 + Δf where Δf = eased · 0.2
//    Envelope: |2·cos(Δf·x/2)| sculpts the familiar beat "lumps".
//    time shifts both sinusoids together (travelling beats).
// ---------------------------------------------------------------------------
struct BeatFrequency;
impl ProgressStyle for BeatFrequency {
    fn name(&self) -> &str {
        "beat-frequency"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Beat frequency: two close sinusoids interfere to create a slowly pulsing envelope"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);

        let f1 = 8.0_f32; // cycles across bar
        let df = ctx.eased * 2.0 + 0.05; // beat frequency (eased → 0..2.05 extra cycles)
        let f2 = f1 + df;
        let phase_shift = ctx.time * 2.0 * PI * 0.4;

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let xn = xi as f32 / w as f32; // [0, 1]
            let theta = xn * 2.0 * PI + phase_shift;
            let val = (f1 * theta).sin() + (f2 * theta).sin();
            // val ∈ [-2, 2] nominally
            let val_n = val * 0.5; // normalise to [-1, 1]
            let dy = (mid - (val_n * amp) as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let (lo, hi) = (py.min(dy), py.max(dy));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Draw the envelope: ±2·|cos(Δf·θ/2)|
        for xi in 0..w {
            let xn = xi as f32 / w as f32;
            let theta = xn * 2.0 * PI + phase_shift;
            let env = 2.0 * ((df * theta / 2.0).cos()).abs() * 0.5 * amp;
            let top_y = (mid - env as i32).clamp(0, h as i32 - 1);
            let bot_y = (mid + env as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, top_y);
            draw::dot_i(grid, xi as i32, bot_y);
        }

        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Wave packet
//     A Gaussian-modulated sinusoid: y = A·exp(-((x-x0)/σ)²) · sin(k·(x-x0)+φ)
//     x0 = progress × L (packet position), σ = L/6, k = 12π/L.
//     time provides the carrier phase φ = ω·t so the inner fringes oscillate.
// ---------------------------------------------------------------------------
struct WavePacket;
impl ProgressStyle for WavePacket {
    fn name(&self) -> &str {
        "wave-packet"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Gaussian wave packet: a quantum-style probability envelope travels across the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);

        let x0 = ctx.eased * wf;
        let sigma = (wf / 5.0).max(1.0);
        let k = 12.0 * PI / wf;
        let phi = ctx.time * 4.0 * PI; // carrier phase

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let xf = xi as f32;
            let dx = xf - x0;
            let envelope = (-(dx * dx) / (2.0 * sigma * sigma)).exp();
            let carrier = (k * dx + phi).sin();
            let val = envelope * carrier;
            let dy = (mid - (val * amp) as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let (lo, hi) = (py.min(dy), py.max(dy));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Draw the Gaussian envelope outline above and below mid
        let mut prev_env: Option<(i32, i32)> = None;
        for xi in 0..w {
            let xf = xi as f32;
            let dx = xf - x0;
            let envelope = (-(dx * dx) / (2.0 * sigma * sigma)).exp();
            let ey = (envelope * amp) as i32;
            let top = (mid - ey).clamp(0, h as i32 - 1);
            let bot = (mid + ey).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, top);
            draw::dot_i(grid, xi as i32, bot);
            if let Some((pt, pb)) = prev_env {
                // Connect envelope curve
                let (lt, ht) = (pt.min(top), pt.max(top));
                let (lb, hb) = (pb.min(bot), pb.max(bot));
                for yy in lt..=ht {
                    draw::dot_i(grid, xi as i32, yy);
                }
                for yy in lb..=hb {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_env = Some((top, bot));
        }

        let (cw, ch) = grid.dimensions();
        // Tint concentrated around the packet centre
        let centre_cell = (ctx.eased * cw as f32).round() as usize;
        let sigma_cells = (cw / 5).max(1);
        for cx in 0..cw {
            let dist = if cx > centre_cell {
                (cx - centre_cell) as f32
            } else {
                (centre_cell - cx) as f32
            };
            let env_c = (-(dist * dist) / (2.0 * sigma_cells as f32 * sigma_cells as f32)).exp();
            if env_c > 0.05 {
                let col = ctx.palette.sample(cx as f32 / cw as f32);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, cx, cx, col);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Spectrum / equalizer bars
//     Synthetic Fourier-domain spectrum: N frequency bins, each amplitude
//     = A_k · (progress fill) + noise driven by sin(k·time).
//     eased controls how many bins are "filled" (left-to-right reveal).
//     Animated by time so each column pulses at its own frequency.
// ---------------------------------------------------------------------------
struct Spectrum;
impl ProgressStyle for Spectrum {
    fn name(&self) -> &str {
        "spectrum"
    }
    fn theme(&self) -> &str {
        "waves"
    }
    fn describe(&self) -> &str {
        "Fourier spectrum equalizer: frequency bins light up left-to-right as progress fills them"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n_bins = w;
        // How many bins are "active" — left n_filled are in the revealed region
        let n_filled = (ctx.eased * n_bins as f32).round() as usize;

        // Draw baseline
        if h >= 1 {
            draw::hline(grid, 0, w.saturating_sub(1), h - 1);
        }

        for bin in 0..n_bins {
            // Spectral envelope: 1/f-like roll-off from DC with a few harmonics louder
            let bin_f = (bin + 1) as f32;
            let spectral = (1.0 / bin_f.sqrt())
                * (1.0 + 0.4 * (bin_f * 0.7).sin())
                * (1.0 + 0.2 * (bin_f * 1.3 + ctx.time * 2.5).sin().abs());

            // Amplitude: spectral shape × whether this bin is "filled"
            let amplitude = if bin < n_filled {
                spectral
            } else {
                spectral * 0.08
            };
            let bar_h = (amplitude * h as f32 * 0.85).round() as usize;
            let bar_h = bar_h.clamp(0, h);

            let y0 = h.saturating_sub(bar_h);
            for y in y0..h {
                draw::dot(grid, bin, y);
            }
        }

        // Colour: gradient across filled bins
        let (cw, ch) = grid.dimensions();
        // Map dot-bins to cells (each cell = 2 dot-columns)
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
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
    let styles = progress::styles::waves::styles();
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
