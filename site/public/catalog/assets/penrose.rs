//! `penrose` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O penrose.rs && ./penrose [style-name]
//! ```

const DEFAULT_STYLE: &str = "penrose-p3";

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
    pub mod penrose {
//! Penrose / aperiodic-tiling sacred-geometry progress styles.
//!
//! Ten structurally distinct styles, each built on a real aperiodic or
//! quasi-periodic mathematical construction:
//!
//! - `penrose-p3`            — Rhombus tiling via Robinson-triangle deflation
//! - `penrose-p2`            — Kite & dart tiling via gnomon/triangle subdivision
//! - `sun-pattern`           — The canonical Penrose "sun" (5 kites) seed cluster
//! - `girih-tiles`           — Islamic girih decagon strapwork
//! - `ammann-bars`           — Ammann quasiperiodic line overlay on rhombi
//! - `debruijn-pentagrid`    — de Bruijn pentagrid dual: 5 families at 72°
//! - `decagon-fractal`       — Decagon recursively filled with smaller decagons
//! - `quasicrystal-diffraction` — Sum of 5 plane waves → 10-fold interference pattern
//! - `pinwheel-tiling`       — Pinwheel 1:2 right-triangle substitution tiling
//! - `truchet-quasi`         — Truchet arcs on a quasiperiodic rhombus lattice

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

const PHI: f32 = 1.6180339887;

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — sunset tiling.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 146, 94);
const TINT_END: Color = Color::rgb(146, 86, 255);

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

/// All styles in the `penrose` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per aperiodic-tiling bar, ordered from
/// most iconic (P3 rhombus) to most exotic (Truchet-quasi).  All styles are
/// independent and can be used in any order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(PenroseP3)),
        Box::new(Tinted(PenroseP2)),
        Box::new(Tinted(SunPattern)),
        Box::new(Tinted(GirihTiles)),
        Box::new(Tinted(AmmannBars)),
        Box::new(Tinted(DeBruijnPentagrid)),
        Box::new(Tinted(DecagonFractal)),
        Box::new(Tinted(QuasicrystalDiffraction)),
        Box::new(Tinted(PinwheelTiling)),
        Box::new(Tinted(TruchetQuasi)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// Grid center in dot-space.
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 * 0.5, dh as f32 * 0.5)
}

/// Uniform scale to fit a unit-radius object in the grid with padding.
#[inline]
fn fit_scale(dw: usize, dh: usize) -> f32 {
    let hw = (dw as f32 * 0.5 - 1.0).max(1.0);
    let hh = (dh as f32 * 0.5 - 1.0).max(1.0);
    hw.min(hh)
}

/// Bresenham line rasteriser. Out-of-bounds dots are silently discarded.
fn bresenham(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a polygon defined by a slice of (x,y) dot-space points (closed).
fn draw_poly(grid: &mut BrailleGrid, pts: &[(i32, i32)]) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    for i in 0..n {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % n];
        bresenham(grid, x0, y0, x1, y1);
    }
}

/// Map a unit-space point (scale ~1.0) to dot-space with center + scale.
#[inline]
fn to_dot(cx: f32, cy: f32, scale: f32, ux: f32, uy: f32, rot: f32) -> (i32, i32) {
    let rx = ux * rot.cos() - uy * rot.sin();
    let ry = ux * rot.sin() + uy * rot.cos();
    (
        (cx + rx * scale).round() as i32,
        (cy - ry * scale).round() as i32,
    )
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Penrose P3 — rhombus tiling via Robinson-triangle deflation
// ────────────────────────────────────────────────────────────────────────────
//
// A Penrose P3 tiling decomposes into two Robinson triangles:
//   • "fat" rhombus → interior angle 72° (composed of two "acute" triangles)
//   • "thin" rhombus → interior angle 36° (composed of two "obtuse" triangles)
//
// Deflation rule (one step doubles the number of triangles):
//   Acute triangle  (A) with vertices P, Q, R:
//     split at S on PR where PS = 1/PHI · PR → A(P,S,Q)  + B(R,S,Q)
//   Obtuse triangle (B) with vertices P, Q, R:
//     split at S on QP where QS = 1/PHI · QP → A(R,S,P)  + B(R,Q,S)
//
// We seed with 10 acute triangles forming a "sun" and deflate up to 5 times,
// capping depth with `ctx.eased`.

struct PenroseP3;
impl ProgressStyle for PenroseP3 {
    fn name(&self) -> &str {
        "penrose-p3"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose P3 rhombus tiling: Robinson-triangle deflation reveals fat (72°) \
         and thin (36°) rhombi generation by generation as progress rises; the \
         whole pattern rotates slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);
        let rot = ctx.time * 0.25;

        // Depth from eased: 0 → depth 0, 1 → depth 4 (max 4 for cost safety).
        let depth = (ctx.eased * 4.0).floor() as usize;
        // Reveal fraction within the current depth level.
        let reveal_frac = (ctx.eased * 4.0).fract();

        // Seed: 10 acute Robinson triangles arranged in a "sun" ring.
        // Each triangle: type=Acute, vertices (p,q,r) in unit space.
        // Acute triangle: two short sides length 1, long side PHI.
        //   p = center, q & r on the circle at angles (k±36°)*π/180
        let mut tris: Vec<(bool, [f32; 2], [f32; 2], [f32; 2])> = Vec::new();
        for k in 0..10usize {
            let a1 = (k as f32 * 36.0) * PI / 180.0;
            let a2 = (k as f32 * 36.0 + 36.0) * PI / 180.0;
            let p = [0.0f32, 0.0];
            let q = [a1.cos(), a1.sin()];
            let r = [a2.cos(), a2.sin()];
            // Alternate acute/obtuse for perfect sun seeding.
            let is_acute = k % 2 == 0;
            tris.push((is_acute, p, q, r));
        }

        // Deflate `depth` times.
        for _step in 0..depth.min(4) {
            tris = deflate_p3(tris);
            if tris.len() > 3000 {
                break;
            } // cost cap
        }

        // Draw revealed triangles (edges only for aesthetic clarity).
        let n_total = tris.len();
        let n_draw = if depth < 4 {
            (reveal_frac * n_total as f32).round() as usize
        } else {
            n_total
        };

        for tri in tris.iter().take(n_draw) {
            let (_is_acute, p, q, r) = tri;
            let pd = to_dot(cx, cy, scale, p[0], p[1], rot);
            let qd = to_dot(cx, cy, scale, q[0], q[1], rot);
            let rd = to_dot(cx, cy, scale, r[0], r[1], rot);
            draw_poly(grid, &[pd, qd, rd]);
        }
        Ok(())
    }
}

/// One deflation step for P3 Robinson triangles.
/// is_acute=true → "acute" (fat-rhombus) triangle, false → "obtuse" (thin-rhombus).
fn deflate_p3(
    tris: Vec<(bool, [f32; 2], [f32; 2], [f32; 2])>,
) -> Vec<(bool, [f32; 2], [f32; 2], [f32; 2])> {
    let mut out = Vec::with_capacity(tris.len() * 2);
    for (is_acute, p, q, r) in tris {
        if is_acute {
            // Acute: P is apex. Split PQ at S where PS = 1/PHI * PQ.
            let s = lerp2(p, q, 1.0 / PHI);
            // Produce: acute(Q,S,R) + obtuse(P,S,R)   [classic Penrose deflation]
            out.push((true, q, s, r));
            out.push((false, p, s, r));
        } else {
            // Obtuse: R is apex opposite the long side. Split RP at S where RS = 1/PHI * RP.
            let s = lerp2(r, p, 1.0 / PHI);
            // Produce: obtuse(Q,S,P) + acute(Q,R,S)
            out.push((false, q, s, p));
            out.push((true, q, r, s));
        }
    }
    out
}

#[inline]
fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Penrose P2 — kite & dart tiling via golden gnomon subdivision
// ────────────────────────────────────────────────────────────────────────────
//
// Kite: a quadrilateral with interior angles 72°,72°,72°,144°.
// Dart: a quadrilateral with interior angles 36°,36°,36°,252°.
//
// We represent each as a pair of "golden triangles":
//   Golden triangle (GT): isoceles, apex 36°, base angles 72° each.
//   Golden gnomon  (GG): isoceles, apex 108°, base angles 36° each.
//
// Deflation (each step roughly multiplies count by PHI²):
//   GT (apex P, base QR): split at S on PR s.t. PS=PQ/PHI → GT(P,S,Q) + GG(S,Q,R)
//   GG (apex P, base QR): split at S on QP s.t. QS=QR/PHI → GT(R,Q,S) + GG(P,R,S)

struct PenroseP2;
impl ProgressStyle for PenroseP2 {
    fn name(&self) -> &str {
        "penrose-p2"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose P2 kite & dart tiling: golden-triangle / golden-gnomon subdivision \
         reveals the kite-and-dart mosaic generation by generation; time animates \
         a gentle shimmer across revealed tiles"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);
        let rot = ctx.time * 0.18;

        let depth = (ctx.eased * 4.0).floor() as usize;
        let reveal_frac = (ctx.eased * 4.0).fract();

        // Seed: 5 golden triangles forming a "star" at the origin.
        let mut tris: Vec<(bool, [f32; 2], [f32; 2], [f32; 2])> = Vec::new();
        for k in 0..5usize {
            let a_mid = (k as f32 * 72.0 + 90.0) * PI / 180.0;
            let a_lo = (k as f32 * 72.0 + 90.0 - 36.0) * PI / 180.0;
            let a_hi = (k as f32 * 72.0 + 90.0 + 36.0) * PI / 180.0;
            let p = [0.0f32, 0.0];
            let q = [a_lo.cos(), a_lo.sin()];
            let r = [a_hi.cos(), a_hi.sin()];
            let _ = a_mid;
            tris.push((true, p, q, r)); // golden triangle, apex at center
        }

        for _step in 0..depth.min(4) {
            tris = deflate_p2(tris);
            if tris.len() > 3000 {
                break;
            }
        }

        let n_total = tris.len();
        let n_draw = if depth < 4 {
            (reveal_frac * n_total as f32).round() as usize
        } else {
            n_total
        };

        for tri in tris.iter().take(n_draw) {
            let (_gt, p, q, r) = tri;
            let pd = to_dot(cx, cy, scale, p[0], p[1], rot);
            let qd = to_dot(cx, cy, scale, q[0], q[1], rot);
            let rd = to_dot(cx, cy, scale, r[0], r[1], rot);
            draw_poly(grid, &[pd, qd, rd]);
        }
        Ok(())
    }
}

fn deflate_p2(
    tris: Vec<(bool, [f32; 2], [f32; 2], [f32; 2])>,
) -> Vec<(bool, [f32; 2], [f32; 2], [f32; 2])> {
    let mut out = Vec::with_capacity(tris.len() * 2);
    for (is_gt, p, q, r) in tris {
        if is_gt {
            // Golden triangle: P is apex (36°), Q and R are base.
            // S on PR such that PS = PQ / PHI (both equal 1 in unit space, S divides at 1/PHI).
            let s = lerp2(p, r, 1.0 / PHI);
            out.push((true, p, q, s));
            out.push((false, s, q, r));
        } else {
            // Golden gnomon: P is apex (108°). S on QP such that QS = QR / PHI.
            let s = lerp2(q, p, 1.0 / PHI);
            out.push((true, r, q, s));
            out.push((false, p, r, s));
        }
    }
    out
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Sun Pattern — 5 kites around a center, expanding rings
// ────────────────────────────────────────────────────────────────────────────
//
// The canonical Penrose "sun" seed: five kites sharing a vertex at the origin.
// Progress reveals rings of tiles growing outward from the center.

struct SunPattern;
impl ProgressStyle for SunPattern {
    fn name(&self) -> &str {
        "sun-pattern"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose sun: five kites sharing the central vertex, then the surrounding \
         dart ring, then outer kite rings — each concentric generation appears as \
         progress rises, pulsing gently with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);
        let rot = ctx.time * 0.15;

        // Kite: apex at center (36° angle), two arms of length 1/PHI, two arms of length 1.
        // Vertices in unit space (apex at 0,0, tip at distance 1 along bisector):
        //   center(0,0), left-wing, outer-tip, right-wing
        // For kite k: bisector angle = k*72°
        //   outer_tip  = (cos(a), sin(a))         distance 1
        //   left_wing  = (cos(a+36°), sin(a+36°)) * (1/PHI)
        //   right_wing = (cos(a-36°), sin(a-36°)) * (1/PHI)

        // Number of concentric "rings" to draw (1-3 based on eased).
        let rings = ((ctx.eased * 3.0) as usize).max(1).min(3);

        for ring in 0..rings {
            let ring_scale = scale / (1.0 + ring as f32 * 0.6);
            // Pulse: inner ring blinks faster.
            let pulse = (ctx.time * (1.0 + ring as f32 * 0.5)).sin() * 0.5 + 0.5;
            if ring > 0 && pulse < 0.3 {
                continue;
            }

            for k in 0..5usize {
                let a = k as f32 * 72.0 * PI / 180.0 + rot + ring as f32 * 36.0 * PI / 180.0;
                let tip = [a.cos(), a.sin()];
                let lw = [(a + PI / 5.0).cos() / PHI, (a + PI / 5.0).sin() / PHI];
                let rw = [(a - PI / 5.0).cos() / PHI, (a - PI / 5.0).sin() / PHI];
                let ctr = [0.0f32, 0.0];

                let pd = to_dot(cx, cy, ring_scale, ctr[0], ctr[1], 0.0);
                let qd = to_dot(cx, cy, ring_scale, lw[0], lw[1], 0.0);
                let rd = to_dot(cx, cy, ring_scale, tip[0], tip[1], 0.0);
                let sd = to_dot(cx, cy, ring_scale, rw[0], rw[1], 0.0);
                draw_poly(grid, &[pd, qd, rd, sd]);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Girih Tiles — Islamic decagon strapwork
// ────────────────────────────────────────────────────────────────────────────
//
// Girih tiles are five polygons (decagon, pentagon, hexagon, bowtie, rhombus)
// whose edges all have the same length and whose angles are multiples of 36°.
// We draw a central decagon, then surround it with pentagons and bowties,
// revealing the strapwork interior lines as progress rises.

struct GirihTiles;
impl ProgressStyle for GirihTiles {
    fn name(&self) -> &str {
        "girih-tiles"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Islamic girih tile strapwork: a central 10-gon surrounded by pentagons \
         and bowties; interior strap lines reveal with progress, the whole pattern \
         rotating with time like a medieval mosque decoration"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh) * 0.45;
        let rot = ctx.time * 0.12;

        // Central decagon.
        let dec = regular_ngon(10, cx, cy, scale, rot);
        draw_poly(grid, &dec);

        // Interior strapwork: connect every other vertex of the decagon.
        let n_straps = (ctx.eased * 10.0).round() as usize;
        for i in 0..n_straps.min(10) {
            let j = (i + 2) % 10;
            bresenham(grid, dec[i].0, dec[i].1, dec[j].0, dec[j].1);
        }

        // Surrounding 10 pentagons at each edge midpoint.
        let n_pent = (ctx.eased * 10.0 * 2.0 - 10.0).round() as usize; // second half of progress
        for k in 0..n_pent.min(10) {
            let a = k as f32 * 36.0 * PI / 180.0 + rot + 18.0 * PI / 180.0;
            let dist = scale * (1.0 + 1.0 / (2.0 * (PI / 10.0).tan()));
            let pcx = cx + dist * a.cos();
            let pcy = cy - dist * a.sin();
            let side = scale * 2.0 * (PI / 10.0).sin();
            let pent = regular_ngon(5, pcx, pcy, side * 0.5, rot + a);
            draw_poly(grid, &pent);
        }
        Ok(())
    }
}

/// Compute vertices of a regular n-gon centered at (cx,cy) in dot-space.
fn regular_ngon(n: usize, cx: f32, cy: f32, radius: f32, offset: f32) -> Vec<(i32, i32)> {
    (0..n)
        .map(|k| {
            let a = 2.0 * PI * k as f32 / n as f32 + offset;
            (
                (cx + radius * a.cos()).round() as i32,
                (cy - radius * a.sin()).round() as i32,
            )
        })
        .collect()
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Ammann Bars — the hidden quasiperiodic grid on a P3 rhombus tiling
// ────────────────────────────────────────────────────────────────────────────
//
// Every Penrose P3 tiling carries an "Ammann bar" decoration: each rhombus
// gets one stripe across it, and the stripes form 5 families of parallel lines
// at angles 0°, 72°, 144°, 216°, 288°.  The spacings within each family are
// quasiperiodic (long L and short S with L/S = PHI).
//
// We approximate this by drawing 5 families of parallel quasiperiodic lines.
// Each family k has angle k*36° and lines at positions …, 0, L, L+S, 2L+S, 2L+2S, …
// where L = PHI and S = 1.

struct AmmannBars;
impl ProgressStyle for AmmannBars {
    fn name(&self) -> &str {
        "ammann-bars"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Ammann bars: the five quasiperiodic stripe families that decorate every \
         Penrose P3 rhombus tiling, with long-L and short-S spacings in ratio PHI; \
         families reveal one by one as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);
        let rot = ctx.time * 0.08;

        // How many of the 5 families to draw (reveal with eased).
        let n_families = (ctx.eased * 5.0).ceil() as usize;
        let family_frac = (ctx.eased * 5.0).fract(); // partial last family

        for fam in 0..n_families.min(5) {
            let angle = fam as f32 * 36.0 * PI / 180.0 + rot;
            // Direction perpendicular to the stripe family.
            let perp_x = angle.cos();
            let perp_y = -angle.sin();
            // Line direction.
            let line_x = -angle.sin();
            let line_y = -angle.cos();

            // Generate quasiperiodic offsets: L, S, L, L, S, L, S, …
            // The Fibonacci word gives the sequence of L/S.
            let max_lines = 20usize;
            let mut offsets: Vec<f32> = Vec::with_capacity(max_lines * 2 + 1);
            offsets.push(0.0);
            let mut pos = 0.0f32;
            let l_step = scale * PHI / (PHI + 1.0);
            let s_step = scale * 1.0 / (PHI + 1.0);
            let mut fib_a = 1usize;
            let mut fib_b = 1usize;
            for _i in 0..max_lines {
                // Use the Fibonacci word: if fib ratio decides L or S.
                let use_long = fib_a > fib_b;
                let step = if use_long { l_step } else { s_step };
                // Update Fibonacci-like counter (Beatty sequence approximation).
                let old_a = fib_a;
                fib_a = fib_a + fib_b;
                fib_b = old_a;
                let fib_a_c = fib_a;
                let fib_b_c = fib_b;
                let _ = (fib_a_c, fib_b_c);
                pos += step;
                offsets.push(pos);
                offsets.push(-pos);
            }

            // Last family: only draw partial set.
            let n_lines = if fam + 1 == n_families {
                (family_frac * offsets.len() as f32) as usize
            } else {
                offsets.len()
            };

            for &off in offsets.iter().take(n_lines) {
                // Line at distance `off` from center, in direction `line_*`.
                let base_x = cx + perp_x * off;
                let base_y = cy + perp_y * off;
                let half = scale * 1.5;
                let x0 = (base_x + line_x * half).round() as i32;
                let y0 = (base_y + line_y * half).round() as i32;
                let x1 = (base_x - line_x * half).round() as i32;
                let y1 = (base_y - line_y * half).round() as i32;
                bresenham(grid, x0, y0, x1, y1);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. De Bruijn Pentagrid — 5 families of parallel lines → dual Penrose tiling
// ────────────────────────────────────────────────────────────────────────────
//
// de Bruijn (1981): a Penrose P3 tiling is dual to a pentagrid — 5 families of
// equidistant parallel lines at angles 72°·k, each with offset γ_k ≈ arbitrary.
//
// We draw the five families, then animate the intersection points to reveal
// the dual rhombus vertices.  The dual step is: each intersection of family j
// and family k at crossing index (m,n) maps to a rhombus vertex.
//
// For progress we simply reveal the five line families one at a time, and
// overlay the computed dual vertices with `ctx.eased`.

struct DeBruijnPentagrid;
impl ProgressStyle for DeBruijnPentagrid {
    fn name(&self) -> &str {
        "debruijn-pentagrid"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "de Bruijn pentagrid: five families of quasiperiodic parallel lines at 72° \
         intervals whose dual gives a Penrose tiling; the grid weaves in with \
         progress and the dual rhombus vertices twinkle with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);
        let rot = ctx.time * 0.07;

        // Five-direction angles.
        let gammas: [f32; 5] = [0.1, 0.2, -0.15, 0.05, -0.08]; // irrational offsets
        let n_families = (ctx.eased * 5.0).ceil() as usize;

        for fam in 0..n_families.min(5) {
            let angle = fam as f32 * 72.0 * PI / 180.0 + rot;
            let perp_x = angle.cos();
            let perp_y = -angle.sin();
            let line_x = -angle.sin();
            let line_y = -angle.cos();
            let gamma = gammas[fam];

            // Draw ~9 parallel lines (4 on each side of center).
            let lines = 9i32;
            for m in -lines / 2..=lines / 2 {
                let off = (m as f32 + gamma) * scale * 0.4;
                let base_x = cx + perp_x * off;
                let base_y = cy + perp_y * off;
                let half = scale * 1.5;
                let x0 = (base_x + line_x * half).round() as i32;
                let y0 = (base_y + line_y * half).round() as i32;
                let x1 = (base_x - line_x * half).round() as i32;
                let y1 = (base_y - line_y * half).round() as i32;
                bresenham(grid, x0, y0, x1, y1);
            }
        }

        // Dual rhombus vertices: intersections of each pair of families.
        // Only draw if eased > 0.7.
        if ctx.eased > 0.7 {
            let dual_frac = (ctx.eased - 0.7) / 0.3;
            let step = scale * 0.4;
            let mut pts: Vec<(i32, i32)> = Vec::new();
            for fj in 0..5usize {
                for fk in (fj + 1)..5usize {
                    let aj = fj as f32 * 72.0 * PI / 180.0 + rot;
                    let ak = fk as f32 * 72.0 * PI / 180.0 + rot;
                    for mj in -4i32..=4 {
                        for mk in -4i32..=4 {
                            // Intersection of line mj in family j and mk in family k.
                            let bj_x = (mj as f32 + gammas[fj]) * step * aj.cos();
                            let bj_y = (mj as f32 + gammas[fj]) * step * (-aj.sin());
                            let bk_x = (mk as f32 + gammas[fk]) * step * ak.cos();
                            let bk_y = (mk as f32 + gammas[fk]) * step * (-ak.sin());
                            let dj_x = -aj.sin();
                            let dj_y = -aj.cos();
                            let dk_x = -ak.sin();
                            let dk_y = -ak.cos();
                            // Solve: (bj + t*dj) = (bk + s*dk)
                            let det = dj_x * dk_y - dj_y * dk_x;
                            if det.abs() < 1e-6 {
                                continue;
                            }
                            let dx = bk_x - bj_x;
                            let dy = bk_y - bj_y;
                            let t = (dx * dk_y - dy * dk_x) / det;
                            let ix = cx + bj_x + t * dj_x;
                            let iy = cy + bj_y + t * dj_y;
                            pts.push((ix.round() as i32, iy.round() as i32));
                        }
                    }
                }
            }
            let n_pts = (dual_frac * pts.len() as f32).round() as usize;
            for &(px, py) in pts.iter().take(n_pts) {
                // Draw a tiny cross at each dual vertex.
                draw::dot_i(grid, px, py);
                draw::dot_i(grid, px + 1, py);
                draw::dot_i(grid, px - 1, py);
                draw::dot_i(grid, px, py + 1);
                draw::dot_i(grid, px, py - 1);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Decagon Fractal — recursive decagon infill
// ────────────────────────────────────────────────────────────────────────────
//
// A decagon subdivides into 10 smaller decagons + connecting pentagons (roughly).
// We approximate with: one large decagon, then ring of 10 smaller at radius
// 1+sin(18°) * sub_r, then sub-sub-decagons, up to depth 3.
// Depth capped at 3 for cost safety.

struct DecagonFractal;
impl ProgressStyle for DecagonFractal {
    fn name(&self) -> &str {
        "decagon-fractal"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Decagon fractal: a central 10-gon filled with rings of smaller 10-gons \
         at each recursion level, forming a self-similar quasicrystalline snowflake \
         that unfolds as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh) * 0.9;
        let rot = ctx.time * 0.1;

        let depth = (ctx.eased * 3.0).floor() as usize;
        let reveal_frac = (ctx.eased * 3.0).fract();

        // Seed: (cx, cy, radius, phase_offset)
        let mut decagons: Vec<(f32, f32, f32, f32)> = vec![(cx, cy, scale, rot)];

        for _d in 0..depth.min(3) {
            let mut next = Vec::new();
            for &(dx, dy, r, phase) in &decagons {
                // Child decagons: 10 of them at distance r + sub_r.
                let sub_r = r / (1.0 + PHI);
                let dist = r - sub_r + sub_r * 0.1; // slight overlap
                for k in 0..10usize {
                    let a = k as f32 * 36.0 * PI / 180.0 + phase;
                    let nx = dx + dist * a.cos();
                    let ny = dy - dist * a.sin();
                    next.push((nx, ny, sub_r, phase + a));
                }
            }
            // Draw the current level's decagons.
            for &(dx, dy, r, phase) in &decagons {
                let pts = regular_ngon(10, dx, dy, r, phase);
                draw_poly(grid, &pts);
            }
            decagons = next;
            if decagons.len() > 500 {
                break;
            }
        }

        // Partial reveal of the last depth ring.
        let n_draw = (reveal_frac * decagons.len() as f32).round() as usize;
        for &(dx, dy, r, phase) in decagons.iter().take(n_draw) {
            let pts = regular_ngon(10, dx, dy, r, phase);
            draw_poly(grid, &pts);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Quasicrystal Diffraction — 5 plane waves at 72° → 10-fold interference
// ────────────────────────────────────────────────────────────────────────────
//
// The quasicrystal interference pattern is computed as:
//   f(x,y) = Σ_{k=0}^{4} cos(2π · (x·cos(k·72°) + y·sin(k·72°)) · freq)
// Normalise to [0,1] and threshold at eased to reveal the bright regions.
// Braille dot space is sampled directly — very fast per dot.

struct QuasicrystalDiffraction;
impl ProgressStyle for QuasicrystalDiffraction {
    fn name(&self) -> &str {
        "quasicrystal-diffraction"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Quasicrystal interference: sum of 5 plane waves at 72° intervals produces \
         a 10-fold diffraction pattern; a moving threshold cuts through the field as \
         progress rises, the whole pattern rotating slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh).max(1.0);

        // Spatial frequency: higher → more rings visible.
        let freq = 3.0 / scale;
        // Phase drift with time — the whole pattern rotates.
        let phase = ctx.time * 0.4;

        // Threshold: eased controls which portion of intensity is lit.
        // f ranges from -5 to 5; threshold sweeps 5 → -5 as eased goes 0→1.
        let threshold = 5.0 - ctx.eased * 10.0;

        for dy in 0..dh {
            for dx in 0..dw {
                let ux = (dx as f32 - cx) / scale;
                let uy = (dy as f32 - cy) / scale;
                // Sum of 5 plane waves.
                let mut f = 0.0f32;
                for k in 0..5usize {
                    let angle = k as f32 * 72.0 * PI / 180.0 + phase;
                    f += (2.0 * PI * freq * (ux * angle.cos() + uy * angle.sin()) * scale).cos();
                }
                if f > threshold {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Pinwheel Tiling — 1:2 right-triangle substitution
// ────────────────────────────────────────────────────────────────────────────
//
// The pinwheel tiling (Conway & Radin 1994) is based on a right triangle
// with legs 1 and 2 and hypotenuse √5.  One triangle substitutes into 5,
// and with each generation the triangles appear at every possible rotation
// (dense in SO(2)), making it rotationally aperiodic.
//
// Deflation: a right triangle (legs a=1, b=2) splits into 5 smaller copies.
// Vertices: P (right angle), Q (on short side), R (far end of long side).
// We cap at depth 4 and cost 2000 triangles.

struct PinwheelTiling;
impl ProgressStyle for PinwheelTiling {
    fn name(&self) -> &str {
        "pinwheel-tiling"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Pinwheel tiling: a 1:2 right triangle substitutes into 5 smaller copies \
         at irrational rotations, filling the plane with triangles at every angle; \
         deflation generations bloom outward as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh) * 0.8;
        let rot = ctx.time * 0.12;

        let depth = (ctx.eased * 3.0).floor() as usize;
        let reveal_frac = (ctx.eased * 3.0).fract();

        // Seed: one 1:2 right triangle in unit space.
        // P=right angle=(0,0), Q=(1,0), R=(0,2), normalised to scale.
        let seed_scale = 1.0 / 5.0f32.sqrt(); // so hypotenuse=1 in unit space
        type Tri = [[f32; 2]; 3]; // [P, Q, R]

        let seed: Tri = [[0.0, 0.0], [seed_scale, 0.0], [0.0, seed_scale * 2.0]];
        let mut tris: Vec<Tri> = vec![seed];

        for _step in 0..depth.min(3) {
            tris = pinwheel_deflate(tris);
            if tris.len() > 2000 {
                break;
            }
        }

        let n_total = tris.len();
        let n_draw = if depth < 3 {
            (reveal_frac * n_total as f32).round() as usize
        } else {
            n_total
        };

        for tri in tris.iter().take(n_draw) {
            let pts: Vec<(i32, i32)> = tri
                .iter()
                .map(|v| to_dot(cx, cy, scale, v[0], v[1], rot))
                .collect();
            draw_poly(grid, &pts);
        }
        Ok(())
    }
}

type Tri2 = [[f32; 2]; 3];

fn pinwheel_deflate(tris: Vec<Tri2>) -> Vec<Tri2> {
    let mut out = Vec::with_capacity(tris.len() * 5);
    for tri in tris {
        let [p, q, r] = tri;
        // Pinwheel substitution: 5 children.
        // Standard pinwheel: right angle at P, short leg PQ (len 1), long leg PR (len 2).
        // Sub-triangle hypotenuse = 1/√5 of parent hypotenuse.
        //
        // Key points (per Conway-Radin):
        //   M  = midpoint(P, R)           (midpoint of long leg)
        //   A  = P + (1/5)*(Q-P)          (1/5 along PQ from P)
        //   B  = P + (2/5)*(Q-P)          (2/5 along PQ from P)
        //   C  = midpoint(Q, R)
        //   D  = M + (1/2)*(Q - M)        ~ midpoint(M, Q)
        //
        // 5 children (approximate but structurally correct):
        let m = lerp2(p, r, 0.5);
        let a = lerp2(p, q, 0.2);
        let b = lerp2(p, q, 0.4);
        let c = lerp2(q, r, 0.5);
        let d = lerp2(m, q, 0.5);
        out.push([p, a, m]);
        out.push([a, b, d]);
        out.push([b, q, c]);
        out.push([d, c, m]);
        out.push([m, c, r]);
    }
    out
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Truchet-Quasi — Truchet arcs on a quasiperiodic rhombus lattice
// ────────────────────────────────────────────────────────────────────────────
//
// Classic Truchet tiles place quarter-circle arcs in squares, two orientations.
// We place arcs inside the rhombi of a Penrose-like rhombus grid (fat and thin),
// with orientation chosen by a deterministic pseudo-random rule seeded from
// the rhombus index so the pattern never repeats.
//
// Since we don't have a full deflation here, we generate a flat grid of
// rhombi using a pentagrid projection approach (approximate) and draw arcs
// inside each.

struct TruchetQuasi;
impl ProgressStyle for TruchetQuasi {
    fn name(&self) -> &str {
        "truchet-quasi"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Truchet-quasi: quarter-circle arcs placed in the fat and thin rhombi of a \
         Penrose-like quasiperiodic lattice; orientation is deterministically varied \
         so the arc flow never repeats, revealed tile by tile with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);
        let rot = ctx.time * 0.09;

        // Generate rhombi from a simple pentagrid projection.
        // For a pentagrid with 5 families at 72°·k:
        //   Each pair of family lines (j,k) intersects, dual vertex = rhombus.
        //   Rhombus type = |j - k| mod 5: if 1 or 4 → fat, if 2 or 3 → thin.
        let spacing = scale * 0.35;
        let mut rhombi: Vec<(f32, f32, f32, bool, usize)> = Vec::new(); // (cx, cy, angle, is_fat, idx)

        let n_lines = 5i32;
        for fj in 0..5usize {
            for fk in (fj + 1)..5usize {
                let aj = fj as f32 * 72.0 * PI / 180.0 + rot;
                let ak = fk as f32 * 72.0 * PI / 180.0 + rot;
                for mj in -n_lines..=n_lines {
                    for mk in -n_lines..=n_lines {
                        let bj_x = mj as f32 * spacing * aj.cos();
                        let bj_y = mj as f32 * spacing * (-aj.sin());
                        let bk_x = mk as f32 * spacing * ak.cos();
                        let bk_y = mk as f32 * spacing * (-ak.sin());
                        let dj_x = -aj.sin();
                        let dj_y = -aj.cos();
                        let dk_x = -ak.sin();
                        let dk_y = -ak.cos();
                        let det = dj_x * dk_y - dj_y * dk_x;
                        if det.abs() < 1e-6 {
                            continue;
                        }
                        let dx = bk_x - bj_x;
                        let dy = bk_y - bj_y;
                        let t = (dx * dk_y - dy * dk_x) / det;
                        let rx = cx + bj_x + t * dj_x;
                        let ry = cy + bj_y + t * dj_y;
                        // Inside grid?
                        if rx < 0.0 || ry < 0.0 || rx >= dw as f32 || ry >= dh as f32 {
                            continue;
                        }
                        let diff = (fk - fj) % 5;
                        let is_fat = diff == 1 || diff == 4;
                        let idx = (mj.unsigned_abs() as usize * 7
                            + mk.unsigned_abs() as usize * 13
                            + fj * 3
                            + fk * 5)
                            & 1; // 0 or 1
                        rhombi.push((rx, ry, aj, is_fat, idx));
                    }
                }
            }
        }

        let n_total = rhombi.len();
        let n_draw = (ctx.eased * n_total as f32).round() as usize;

        for &(rx, ry, angle, _is_fat, flip) in rhombi.iter().take(n_draw) {
            // Draw a small quarter-arc inside the rhombus.
            // Arc radius = spacing * 0.4.
            let r = spacing * 0.4;
            let arc_steps = 8usize;
            // Choose two corners based on flip.
            let corner_a = angle + if flip == 0 { 0.0 } else { PI };
            let corner_b = corner_a + PI * 0.5;
            // Arc from corner_a to corner_b.
            let arc_cx = rx + r * corner_a.cos();
            let arc_cy = ry - r * corner_a.sin();
            for s in 0..=arc_steps {
                let a = corner_b + (corner_a - corner_b) * s as f32 / arc_steps as f32;
                let px = (arc_cx + r * a.cos()).round() as i32;
                let py = (arc_cy - r * a.sin()).round() as i32;
                draw::dot_i(grid, px, py);
            }
            // Second arc from the opposite corner.
            let corner_c = corner_a + PI;
            let corner_d = corner_c + PI * 0.5;
            let arc_cx2 = rx + r * corner_c.cos();
            let arc_cy2 = ry - r * corner_c.sin();
            for s in 0..=arc_steps {
                let a = corner_d + (corner_c - corner_d) * s as f32 / arc_steps as f32;
                let px = (arc_cx2 + r * a.cos()).round() as i32;
                let py = (arc_cy2 - r * a.sin()).round() as i32;
                draw::dot_i(grid, px, py);
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
    let styles = progress::styles::penrose::styles();
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
