//! `yantra` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O yantra.rs && ./yantra [style-name]
//! ```

const DEFAULT_STYLE: &str = "sri-yantra";

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
    pub mod yantra {
//! Yantra / Mandala sacred-geometry progress styles.
//!
//! Eleven structurally distinct radially symmetric styles drawn entirely with
//! braille dots via `draw::dot_i` and the Bresenham line helper defined below.
//! Each style has a unique construction principle:
//!
//! - `sri-yantra`            — 9 interlocking triangles + lotus + bhupura gates
//! - `lotus-mandala`         — concentric rings of opening lotus petals (arc pairs)
//! - `rose-window`           — Gothic cathedral radial tracery with foil cusps
//! - `bagua`                 — eight trigrams around a yin-yang center
//! - `enneagram`             — 9-point star (1-4-2-8-5-7 cycle + triangle)
//! - `rangoli`               — dot-grid kolam with loops around pulli points
//! - `chartres-labyrinth`    — 11-circuit circular labyrinth path winding inward
//! - `mandala-tessellation`  — n-fold rotational tiling growing outward with progress
//! - `dharma-wheel`          — 8-spoked wheel with hub, rim and decorative felloes
//! - `vesica-rosette`        — 6-petal rosette from overlapping vesica piscis arcs
//! - `star-of-david`         — Star of David hexagram nested with concentric circles

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — saffron into gold.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 126, 84);
const TINT_END: Color = Color::rgb(255, 204, 64);

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

/// All styles in the `yantra` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per sacred-geometry style. The eleven
/// styles span distinct construction methods — triangle systems, petal arcs,
/// tracery, trigrams, star polygons, kolam loops, labyrinth paths, rotational
/// tilings, spoked wheels, vesica arcs, and nested hexagrams — ensuring no two
/// styles are structurally alike.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(SriYantra)),
        Box::new(Tinted(LotusMandala)),
        Box::new(Tinted(RoseWindow)),
        Box::new(Tinted(Bagua)),
        Box::new(Tinted(Enneagram)),
        Box::new(Tinted(Rangoli)),
        Box::new(Tinted(ChartresLabyrinth)),
        Box::new(Tinted(MandalaTessellation)),
        Box::new(Tinted(DharmaWheel)),
        Box::new(Tinted(VesicaRosette)),
        Box::new(Tinted(StarOfDavid)),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared geometry helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Grid center in dot-space.
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

/// Largest radius that fits within the dot grid with 1-dot padding.
#[inline]
fn fit_radius(dw: usize, dh: usize) -> f32 {
    let hw = (dw as f32 / 2.0 - 1.0).max(1.0);
    let hh = (dh as f32 / 2.0 - 1.0).max(1.0);
    hw.min(hh)
}

/// Convert polar coordinates centered at (cx, cy) to dot-space integers.
#[inline]
fn polar(cx: f32, cy: f32, r: f32, angle: f32) -> (i32, i32) {
    let x = cx + r * angle.cos();
    let y = cy - r * angle.sin(); // screen y-axis is flipped
    (x.round() as i32, y.round() as i32)
}

/// Step-bounded Bresenham line between two signed dot-space points.
/// Out-of-bounds dots are silently discarded by `draw::dot_i`.
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x < x1 { 1 } else { -1 };
    let sy: i32 = if y < y1 { 1 } else { -1 };
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

/// Draw a circular arc from `a_start` to `a_end` (radians, CCW) at radius `r`
/// centered at (cx, cy).  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a_start: f32, a_end: f32) {
    if r < 0.5 {
        return;
    }
    let span = (a_end - a_start).abs();
    let steps = ((r * span).ceil() as usize).max(4).min(1024);
    let mut prev: Option<(i32, i32)> = None;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = a_start + (a_end - a_start) * t;
        let p = polar(cx, cy, r, a);
        if let Some(q) = prev {
            line(grid, q.0, q.1, p.0, p.1);
        }
        prev = Some(p);
    }
}

/// Draw a full circle at radius `r` centered at (cx, cy).
fn circle(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    arc(grid, cx, cy, r, 0.0, 2.0 * PI);
}

/// Draw a triangle through three dot-space points.
fn triangle(grid: &mut BrailleGrid, ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) {
    line(grid, ax, ay, bx, by);
    line(grid, bx, by, cx, cy);
    line(grid, cx, cy, ax, ay);
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Sri Yantra
// ─────────────────────────────────────────────────────────────────────────────

struct SriYantra;
impl ProgressStyle for SriYantra {
    fn name(&self) -> &str {
        "sri-yantra"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Sri Yantra: 9 interlocking triangles (4 upward Shiva + 5 downward Shakti) \
         forming 43 small triangles, with an outer lotus ring and square bhupura gates — \
         the triangles are revealed layer by layer as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh);
        let rot = ctx.time * 0.15;

        // 9 triangles defined as (apex_angle, base_angle_L, base_angle_R, radius_scale)
        // Upward triangles (Shiva) point up (apex at top = PI/2), alternating angles.
        // Downward triangles (Shakti) apex at bottom (= -PI/2).
        // We approximate the classic proportions with 5 size levels.
        let triangles: &[(f32, f32, f32)] = &[
            // (apex_angle from center, base half-angle_spread, radius_fraction)
            // 4 upward (apex = PI/2):
            (PI / 2.0, 0.95, 1.00), // T1 largest upward
            (PI / 2.0, 0.70, 0.75), // T2
            (PI / 2.0, 0.50, 0.55), // T3
            (PI / 2.0, 0.30, 0.35), // T4 smallest upward
            // 5 downward (apex = -PI/2):
            (-PI / 2.0, 0.88, 0.92), // T5 largest downward
            (-PI / 2.0, 0.65, 0.68), // T6
            (-PI / 2.0, 0.45, 0.48), // T7
            (-PI / 2.0, 0.28, 0.30), // T8
            (-PI / 2.0, 0.15, 0.16), // T9 smallest downward
        ];

        let total = triangles.len();
        let reveal = (ctx.eased * (total + 6) as f32).round() as usize; // +6 for lotus+bhupura

        // Draw bhupura (square gates) — three nested squares
        if reveal >= total + 3 {
            for k in 0..3usize {
                let s = r * (1.15 + k as f32 * 0.10);
                // Square corners
                let corners = [
                    (cx - s, cy - s * 0.5),
                    (cx + s, cy - s * 0.5),
                    (cx + s, cy + s * 0.5),
                    (cx - s, cy + s * 0.5),
                ];
                for i in 0..4 {
                    let (ax, ay) = corners[i];
                    let (bx, by) = corners[(i + 1) % 4];
                    line(
                        grid,
                        ax.round() as i32,
                        ay.round() as i32,
                        bx.round() as i32,
                        by.round() as i32,
                    );
                }
            }
        }

        // Draw outer lotus ring (8 petals as arcs)
        if reveal >= total + 1 {
            let petal_r = r * 0.18;
            let ring_r = r * 1.05;
            let n_petals = 8usize;
            for k in 0..n_petals {
                let angle = rot + 2.0 * PI * k as f32 / n_petals as f32;
                let pcx = cx + ring_r * angle.cos();
                let pcy = cy - ring_r * angle.sin();
                arc(grid, pcx, pcy, petal_r, angle + PI * 0.3, angle + PI * 1.7);
            }
        }

        // Enclosing circle
        if reveal >= total {
            circle(grid, cx, cy, r);
        }

        // Draw triangles (revealed one at a time with eased)
        for (idx, &(apex_a, half_spread, rfrac)) in triangles.iter().enumerate() {
            if idx >= reveal {
                break;
            }
            let tr = r * rfrac;
            let apex_a = apex_a + rot;
            let (ax, ay) = polar(cx, cy, tr, apex_a);
            let (bx, by) = polar(cx, cy, tr * 0.7, apex_a + PI - half_spread);
            let (dx, dy) = polar(cx, cy, tr * 0.7, apex_a + PI + half_spread);
            triangle(grid, ax, ay, bx, by, dx, dy);
        }

        // Bindu (central dot)
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Lotus Mandala
// ─────────────────────────────────────────────────────────────────────────────

struct LotusMandala;
impl ProgressStyle for LotusMandala {
    fn name(&self) -> &str {
        "lotus-mandala"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Lotus mandala: concentric rings of lotus petals drawn as paired arcs, \
         opening outward petal-by-petal as progress rises — inner buds swell \
         into full blooms, outer rings shimmer with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r_max = fit_radius(dw, dh);
        let rot = ctx.time * 0.12;

        // Concentric rings of petals: (n_petals, ring_frac, petal_open_frac)
        let rings: &[(usize, f32)] = &[
            (1, 0.0), // center bindu
            (8, 0.25),
            (12, 0.50),
            (16, 0.75),
            (24, 1.00),
        ];

        let total_rings = rings.len();
        let reveal_frac = ctx.eased;

        for (ri, &(n_petals, ring_frac)) in rings.iter().enumerate() {
            // Each ring appears after a fraction of progress
            let ring_threshold = ri as f32 / total_rings as f32;
            if reveal_frac < ring_threshold {
                break;
            }
            // How far this ring is "open"
            let ring_open = ((reveal_frac - ring_threshold) * total_rings as f32).min(1.0);

            if n_petals == 1 {
                // Bindu
                draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
                continue;
            }

            let ring_r = r_max * ring_frac;
            let petal_r = ring_r * 0.35;
            // Time shimmer offset per ring
            let shimmer = rot * (1.0 + ri as f32 * 0.3);

            for k in 0..n_petals {
                // Reveal petals one by one within the ring
                let petal_thresh = k as f32 / n_petals as f32;
                if ring_open < petal_thresh {
                    break;
                }
                let petal_open = ((ring_open - petal_thresh) * n_petals as f32).min(1.0);

                let base_angle = 2.0 * PI * k as f32 / n_petals as f32 + shimmer;
                let pcx = cx + ring_r * base_angle.cos();
                let pcy = cy - ring_r * base_angle.sin();

                // Two arcs per petal (left and right lobe) opening symmetrically
                let half_span = PI * 0.45 * petal_open;
                arc(
                    grid,
                    pcx,
                    pcy,
                    petal_r,
                    base_angle + PI - half_span,
                    base_angle + PI + half_span,
                );
                // Inner arc (cusp line back toward center)
                let inner_r = petal_r * 0.5 * petal_open;
                arc(
                    grid,
                    cx,
                    cy,
                    ring_r - inner_r,
                    base_angle - half_span * 0.4,
                    base_angle + half_span * 0.4,
                );
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Rose Window (Gothic cathedral tracery)
// ─────────────────────────────────────────────────────────────────────────────

struct RoseWindow;
impl ProgressStyle for RoseWindow {
    fn name(&self) -> &str {
        "rose-window"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Rose window: Gothic cathedral radial tracery — spokes divide the circle \
         into 12 lancets each filled with trefoil cusps, the tracery spun by \
         time while progress reveals ring after ring of foils"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh);
        let rot = ctx.time * 0.10;

        let n_spokes = 12usize;
        let reveal = ctx.eased;

        // Outer rim
        if reveal > 0.0 {
            circle(grid, cx, cy, r);
        }

        // Hub circle
        let hub_r = r * 0.12;
        if reveal > 0.05 {
            circle(grid, cx, cy, hub_r);
        }

        // Radial spokes
        let spoke_count = (reveal * n_spokes as f32).ceil() as usize;
        for k in 0..spoke_count.min(n_spokes) {
            let a = rot + 2.0 * PI * k as f32 / n_spokes as f32;
            let (x0, y0) = polar(cx, cy, hub_r, a);
            let (x1, y1) = polar(cx, cy, r, a);
            line(grid, x0, y0, x1, y1);
        }

        // Concentric rings of trefoil foils at 3 radii
        let foil_radii = [0.40, 0.65, 0.87];
        let foil_counts = [6usize, 12, 24];
        for (fi, (&fr, &fc)) in foil_radii.iter().zip(foil_counts.iter()).enumerate() {
            let ring_thresh = (fi as f32 + 1.0) / 4.0;
            if reveal < ring_thresh {
                continue;
            }
            let ring_r = r * fr;
            let foil_r = r * 0.10;
            let shimmer = rot * (1.0 + fi as f32 * 0.5);
            for k in 0..fc {
                let a = shimmer + 2.0 * PI * k as f32 / fc as f32;
                let fx = cx + ring_r * a.cos();
                let fy = cy - ring_r * a.sin();
                // Trefoil = three small circles at 120° offsets
                for leaf in 0..3usize {
                    let la = a + 2.0 * PI * leaf as f32 / 3.0;
                    let lx = fx + foil_r * 0.55 * la.cos();
                    let ly = fy - foil_r * 0.55 * la.sin();
                    circle(grid, lx, ly, foil_r * 0.45);
                }
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Bagua (Eight Trigrams)
// ─────────────────────────────────────────────────────────────────────────────

struct Bagua;
impl ProgressStyle for Bagua {
    fn name(&self) -> &str {
        "bagua"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Bagua: the eight trigrams of the I-Ching arranged around a yin-yang \
         center — each trigram is three parallel lines (broken or solid) appearing \
         as progress rises, slowly rotating with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh);
        let rot = ctx.time * 0.08;

        // Eight trigrams: bit pattern (3 bits, MSB = top line), 0=broken 1=solid.
        // King Wen arrangement: Qian(7) Dui(6) Li(5) Zhen(4) Xun(3) Kan(2) Gen(1) Kun(0)
        let trigrams: [u8; 8] = [7, 6, 5, 4, 3, 2, 1, 0];
        let reveal_count = (ctx.eased * 8.0).ceil() as usize;

        // Yin-yang circle (two arcs + S-curve)
        let yy_r = r * 0.18;
        if ctx.eased > 0.0 {
            circle(grid, cx, cy, yy_r);
            // Upper half: small filled circle
            circle(grid, cx, cy - yy_r * 0.5, yy_r * 0.25);
            // Lower half: small empty circle
            circle(grid, cx, cy + yy_r * 0.5, yy_r * 0.25);
            // S-curve: two arcs through center
            arc(grid, cx, cy - yy_r * 0.5, yy_r * 0.5, -PI / 2.0, PI / 2.0);
            arc(
                grid,
                cx,
                cy + yy_r * 0.5,
                yy_r * 0.5,
                PI / 2.0,
                3.0 * PI / 2.0,
            );
        }

        // Outer ring
        if ctx.eased > 0.05 {
            circle(grid, cx, cy, r * 0.85);
        }

        for (idx, &bits) in trigrams.iter().enumerate() {
            if idx >= reveal_count {
                break;
            }
            let angle = rot + 2.0 * PI * idx as f32 / 8.0 - PI / 2.0;

            // Position trigram at outer ring
            let tr_cx = cx + r * 0.68 * angle.cos();
            let tr_cy = cy - r * 0.68 * angle.sin();

            // Three lines per trigram, stacked perpendicular to the radial direction
            let perp = angle + PI / 2.0; // perpendicular direction
            let line_len = r * 0.10;
            let line_gap = r * 0.08;

            for line_idx in 0..3usize {
                let offset = (line_idx as f32 - 1.0) * line_gap;
                let lx = tr_cx + offset * angle.cos();
                let ly = tr_cy - offset * angle.sin();
                let solid = (bits >> (2 - line_idx)) & 1 == 1;

                let (x0, y0) = polar(lx, ly, line_len, perp);
                let (x1, y1) = polar(lx, ly, line_len, perp + PI);
                if solid {
                    line(grid, x0, y0, x1, y1);
                } else {
                    // Broken line: two halves with gap
                    let (xm0, ym0) = polar(lx, ly, line_len * 0.3, perp);
                    let (xm1, ym1) = polar(lx, ly, line_len * 0.3, perp + PI);
                    line(grid, x0, y0, xm0, ym0);
                    line(grid, x1, y1, xm1, ym1);
                }
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Enneagram
// ─────────────────────────────────────────────────────────────────────────────

struct Enneagram;
impl ProgressStyle for Enneagram {
    fn name(&self) -> &str {
        "enneagram"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Enneagram: 9 points on a circle connected by the 1-4-2-8-5-7 hexad \
         and a separate 3-6-9 triangle — two interlocked figures appear chord \
         by chord as progress rises, spinning slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh);
        let rot = ctx.time * 0.10 - PI / 2.0; // apex at top

        // 9 evenly spaced vertices
        let verts: Vec<(i32, i32)> = (0..9)
            .map(|k| polar(cx, cy, r, rot + 2.0 * PI * k as f32 / 9.0))
            .collect();

        // Enclosing circle
        if ctx.eased > 0.0 {
            circle(grid, cx, cy, r);
        }

        // Hexad sequence: 1→4→2→8→5→7→1 (0-indexed: 0→3→1→7→4→6→0)
        let hexad: &[usize] = &[0, 3, 1, 7, 4, 6, 0];
        let hexad_chords = hexad.len() - 1;

        // Triangle: 3-6-9 → 0-indexed 2,5,8
        let tri: &[usize] = &[2, 5, 8, 2];
        let tri_chords = tri.len() - 1;

        let total_chords = hexad_chords + tri_chords;
        let reveal = (ctx.eased * total_chords as f32).round() as usize;

        // Draw hexad chords
        for i in 0..hexad_chords.min(reveal) {
            let (ax, ay) = verts[hexad[i]];
            let (bx, by) = verts[hexad[i + 1]];
            line(grid, ax, ay, bx, by);
        }

        // Draw triangle chords
        let tri_drawn = reveal.saturating_sub(hexad_chords);
        for i in 0..tri_chords.min(tri_drawn) {
            let (ax, ay) = verts[tri[i]];
            let (bx, by) = verts[tri[i + 1]];
            line(grid, ax, ay, bx, by);
        }

        // Vertex dots
        let vert_reveal = reveal.min(9);
        for (vx, vy) in verts.iter().take(vert_reveal) {
            draw::dot_i(grid, *vx, *vy);
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Rangoli / Kolam
// ─────────────────────────────────────────────────────────────────────────────

struct Rangoli;
impl ProgressStyle for Rangoli {
    fn name(&self) -> &str {
        "rangoli"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Rangoli kolam: a grid of pulli dots with looping curves drawn around \
         them in the South Indian kolam tradition — loops spiral outward from \
         the center dot as progress rises, time rotates the outer loops"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r_max = fit_radius(dw, dh);
        let rot = ctx.time * 0.07;

        // Grid of pulli dots arranged in concentric diamond rings.
        // Each ring k has 4k dots at distance k*step from center.
        let step = (r_max / 4.0).max(2.0);
        let n_rings = ((r_max / step) as usize).max(1).min(4);
        let total_dots = 1 + (1..=n_rings).map(|k| 4 * k).sum::<usize>();
        let reveal = (ctx.eased * (total_dots + n_rings * 4) as f32).round() as usize;

        let mut dots_drawn = 0usize;
        let mut loops_drawn = 0usize;

        // Center dot
        if reveal > 0 {
            draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
            dots_drawn += 1;
        }

        // Loop around center dot (small circle)
        if reveal > 1 {
            let lrot = rot;
            arc(
                grid,
                cx + step * 0.5 * lrot.cos(),
                cy - step * 0.5 * lrot.sin(),
                step * 0.45,
                0.0,
                2.0 * PI,
            );
            loops_drawn += 1;
        }

        for ring in 1..=n_rings {
            let ring_dots = 4 * ring;
            for k in 0..ring_dots {
                if dots_drawn >= reveal {
                    break;
                }
                let angle = rot + 2.0 * PI * k as f32 / ring_dots as f32;
                let dr = step * ring as f32;
                let dx = cx + dr * angle.cos();
                let dy = cy - dr * angle.sin();
                draw::dot_i(grid, dx.round() as i32, dy.round() as i32);
                dots_drawn += 1;

                // Loop around this dot: figure-eight loops encircling adjacent pairs
                if dots_drawn + loops_drawn < reveal {
                    let next_a = rot + 2.0 * PI * (k + 1) as f32 / ring_dots as f32;
                    let nx = cx + dr * next_a.cos();
                    let ny = cy - dr * next_a.sin();
                    let mx = (dx + nx) / 2.0;
                    let my = (dy + ny) / 2.0;
                    let loop_r = ((dx - nx).hypot(dy - ny) * 0.35).max(1.0);
                    // Two arcs forming a loop around the midpoint
                    arc(grid, mx, my, loop_r, angle + PI * 0.2, angle + PI * 1.8);
                    loops_drawn += 1;
                }
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Chartres Labyrinth
// ─────────────────────────────────────────────────────────────────────────────

struct ChartresLabyrinth;
impl ProgressStyle for ChartresLabyrinth {
    fn name(&self) -> &str {
        "chartres-labyrinth"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Chartres labyrinth: the classical 11-circuit circular labyrinth path \
         winding inward — concentric arcs separated by quarter-turn gaps trace \
         the unicursal path as progress slowly fills each circuit"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r_max = fit_radius(dw, dh);

        // 11 circuits + center: 12 concentric arcs
        let n_circuits = 11usize;
        let reveal = (ctx.eased * (n_circuits + 1) as f32).ceil() as usize;

        // Slow rotation of the whole labyrinth with time
        let rot = ctx.time * 0.05;

        // Each circuit is a pair of arcs: the left half and the right half,
        // connected by cross-passages at the top (entrance) and at ±90° turns.
        // We simplify: each circuit = one near-full arc with a gap for the path entrance.
        for circuit in 0..reveal.min(n_circuits) {
            let frac = (n_circuits - circuit) as f32 / n_circuits as f32;
            let r = r_max * frac;

            // Gap angle: alternates left/right per circuit in the Chartres pattern
            let gap_side = if circuit % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
            let gap_center = rot + gap_side * PI / 2.0;
            let gap_half = PI * 0.12; // gap width in radians

            // Arc sweeping almost full circle, skipping the gap
            let a_start = gap_center + gap_half;
            let a_end = gap_center + 2.0 * PI - gap_half;
            arc(grid, cx, cy, r, a_start, a_end);

            // Cross-passage: a short radial line at the gap
            if circuit + 1 < reveal {
                let inner_r = r_max * (n_circuits - circuit - 1) as f32 / n_circuits as f32;
                let (x0, y0) = polar(cx, cy, inner_r, gap_center + gap_half);
                let (x1, y1) = polar(cx, cy, r, gap_center + gap_half);
                line(grid, x0, y0, x1, y1);
            }
        }

        // Center (goal)
        if reveal > n_circuits {
            circle(grid, cx, cy, r_max / n_circuits as f32 * 0.5);
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Mandala Tessellation
// ─────────────────────────────────────────────────────────────────────────────

struct MandalaTessellation;
impl ProgressStyle for MandalaTessellation {
    fn name(&self) -> &str {
        "mandala-tessellation"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Mandala tessellation: 6-fold rotational tiling of kite-and-dart units \
         growing outward from the center — each tile is a rhombus drawn with \
         Bresenham lines, revealing ring by ring as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r_max = fit_radius(dw, dh);
        let rot = ctx.time * 0.09;

        let n_fold = 6usize;
        let n_rings = 4usize;
        let total = n_fold * n_rings;
        let reveal = (ctx.eased * total as f32).ceil() as usize;

        let tile_r = r_max / n_rings as f32;

        let mut drawn = 0usize;
        'outer: for ring in 1..=n_rings {
            let tiles_per_ring = n_fold * ring; // grows with ring
            let inner_r = tile_r * (ring - 1) as f32;
            let outer_r = tile_r * ring as f32;

            for k in 0..tiles_per_ring {
                if drawn >= reveal {
                    break 'outer;
                }
                let a0 = rot + 2.0 * PI * k as f32 / tiles_per_ring as f32;
                let a1 = rot + 2.0 * PI * (k + 1) as f32 / tiles_per_ring as f32;
                let am = (a0 + a1) / 2.0;

                // Rhombus: four corners
                let (ax, ay) = polar(cx, cy, inner_r, a0);
                let (bx, by) = polar(cx, cy, outer_r, a0);
                let (ex, ey) = polar(cx, cy, outer_r, a1);
                let (fx, fy) = polar(cx, cy, inner_r, a1);
                let (mx, my) = polar(cx, cy, (inner_r + outer_r) / 2.0, am);

                // Draw the kite as two triangles sharing the midpoint apex
                line(grid, ax, ay, mx, my);
                line(grid, mx, my, ex, ey);
                line(grid, ex, ey, fx, fy);
                line(grid, fx, fy, ax, ay);
                // Internal decorative diagonal
                line(grid, bx, by, fx, fy);

                drawn += 1;
            }
        }

        // Center hub
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Dharma Wheel
// ─────────────────────────────────────────────────────────────────────────────

struct DharmaWheel;
impl ProgressStyle for DharmaWheel {
    fn name(&self) -> &str {
        "dharma-wheel"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Dharma wheel (Dharmachakra): 8 spokes radiating from a central hub \
         to an outer rim, with decorative felloe arcs between spoke tips — \
         the wheel spins continuously with time while spokes reveal with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh);
        let rot = ctx.time * 0.40; // continuous spin

        let n_spokes = 8usize;
        let hub_r = (r * 0.15).max(1.0);
        let rim_r = r;
        let felloe_r = r * 0.12; // arc radius between spoke tips

        // Outer rim
        circle(grid, cx, cy, rim_r);
        // Inner rim (double rim = traditional look)
        circle(grid, cx, cy, rim_r * 0.88);
        // Hub
        circle(grid, cx, cy, hub_r);
        circle(grid, cx, cy, hub_r * 0.5);

        let spoke_reveal = (ctx.eased * n_spokes as f32).ceil() as usize;
        for k in 0..spoke_reveal.min(n_spokes) {
            let a = rot + PI * k as f32 / n_spokes as f32; // 8 spokes = every 22.5°
            let (x0, y0) = polar(cx, cy, hub_r, a);
            let (x1, y1) = polar(cx, cy, rim_r * 0.88, a);
            line(grid, x0, y0, x1, y1);
            // Opposite spoke (each of 8 half-spokes creates a full diameter)
            let (x2, y2) = polar(cx, cy, hub_r, a + PI);
            let (x3, y3) = polar(cx, cy, rim_r * 0.88, a + PI);
            line(grid, x2, y2, x3, y3);
        }

        // Felloe arcs between spoke tips (decorative cusps)
        if ctx.eased > 0.5 {
            for k in 0..n_spokes {
                let a0 = rot + PI * k as f32 / n_spokes as f32;
                let a1 = rot + PI * (k + 1) as f32 / n_spokes as f32;
                let (tx0, ty0) = polar(cx, cy, rim_r * 0.88, a0);
                let (tx1, ty1) = polar(cx, cy, rim_r * 0.88, a1);
                let mx = (tx0 + tx1) as f32 / 2.0;
                let my = (ty0 + ty1) as f32 / 2.0;
                circle(grid, mx, my, felloe_r);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Vesica Rosette
// ─────────────────────────────────────────────────────────────────────────────

struct VesicaRosette;
impl ProgressStyle for VesicaRosette {
    fn name(&self) -> &str {
        "vesica-rosette"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Vesica rosette: a six-petal rosette built from six overlapping vesica \
         piscis arcs — each arc is a circle of the same radius passing through \
         the center, petals bloom one by one with progress as the rosette rotates"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh) * 0.55; // vesica circles have radius = r_outer
        let rot = ctx.time * 0.14;

        let n_petals = 6usize;
        let reveal = (ctx.eased * (n_petals + 2) as f32).round() as usize;

        // Outer enclosing circle
        if reveal >= n_petals + 1 {
            circle(grid, cx, cy, r * 1.00);
        }

        // Center bindu
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

        // Each of the 6 vesica circles is offset from center by r in one direction.
        // A vesica piscis is the intersection region of two equal circles, each
        // passing through the other's center.  We draw only the arc that lies within
        // the central enclosing circle.
        for k in 0..reveal.min(n_petals) {
            let a = rot + 2.0 * PI * k as f32 / n_petals as f32;
            let ocx = cx + r * a.cos();
            let ocy = cy - r * a.sin();
            // The arc of this circle that passes through the center region spans
            // approximately ±60° around the direction back toward the center.
            let back = a + PI;
            arc(grid, ocx, ocy, r, back - PI / 3.0, back + PI / 3.0);

            // Second arc on the same circle: forward-facing lobe
            // (gives the petal shape when two adjacent circles overlap)
            let fwd = a;
            arc(grid, ocx, ocy, r, fwd - PI / 3.0, fwd + PI / 3.0);
        }

        // Inner hexagon at the vertices of the petal intersections
        if reveal >= n_petals {
            for k in 0..n_petals {
                let a0 = rot + 2.0 * PI * k as f32 / n_petals as f32;
                let a1 = rot + 2.0 * PI * (k + 1) as f32 / n_petals as f32;
                let (x0, y0) = polar(cx, cy, r, a0);
                let (x1, y1) = polar(cx, cy, r, a1);
                line(grid, x0, y0, x1, y1);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Star of David / Hexagram
// ─────────────────────────────────────────────────────────────────────────────

struct StarOfDavid;
impl ProgressStyle for StarOfDavid {
    fn name(&self) -> &str {
        "star-of-david"
    }
    fn theme(&self) -> &str {
        "yantra"
    }
    fn describe(&self) -> &str {
        "Star of David hexagram: two interlocking equilateral triangles nested \
         inside concentric circles — the up-triangle and down-triangle appear \
         in sequence with progress while three harmonic circles spin with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let r = fit_radius(dw, dh);
        let rot = ctx.time * 0.12;

        let reveal = ctx.eased;

        // Three concentric circles (outer, mid, inner) spinning with time
        if reveal > 0.0 {
            circle(grid, cx, cy, r);
        }
        if reveal > 0.25 {
            circle(grid, cx, cy, r * 0.70);
        }
        if reveal > 0.50 {
            circle(grid, cx, cy, r * 0.35);
        }

        // Upward triangle (Star of David: apex at top, Δ)
        if reveal > 0.15 {
            let t_frac = ((reveal - 0.15) / 0.35).min(1.0);
            let tr = r * 0.85;
            let apex_a = PI / 2.0 + rot;
            let left_a = PI / 2.0 + 2.0 * PI / 3.0 + rot;
            let right_a = PI / 2.0 - 2.0 * PI / 3.0 + rot;
            let (ax, ay) = polar(cx, cy, tr, apex_a);
            let (lx, ly) = polar(cx, cy, tr, left_a);
            let (rx, ry) = polar(cx, cy, tr, right_a);

            // Reveal side by side progressively
            let sides = (t_frac * 3.0).ceil() as usize;
            if sides >= 1 {
                line(grid, ax, ay, lx, ly);
            }
            if sides >= 2 {
                line(grid, lx, ly, rx, ry);
            }
            if sides >= 3 {
                line(grid, rx, ry, ax, ay);
            }
        }

        // Downward triangle (inverted, ∇)
        if reveal > 0.50 {
            let t_frac = ((reveal - 0.50) / 0.35).min(1.0);
            let tr = r * 0.85;
            let apex_a = -PI / 2.0 + rot;
            let left_a = -PI / 2.0 + 2.0 * PI / 3.0 + rot;
            let right_a = -PI / 2.0 - 2.0 * PI / 3.0 + rot;
            let (ax, ay) = polar(cx, cy, tr, apex_a);
            let (lx, ly) = polar(cx, cy, tr, left_a);
            let (rx, ry) = polar(cx, cy, tr, right_a);

            let sides = (t_frac * 3.0).ceil() as usize;
            if sides >= 1 {
                line(grid, ax, ay, lx, ly);
            }
            if sides >= 2 {
                line(grid, lx, ly, rx, ry);
            }
            if sides >= 3 {
                line(grid, rx, ry, ax, ay);
            }
        }

        // Central hexagon (intersection of the two triangles)
        if reveal > 0.85 {
            let hex_r = r * 0.49;
            for k in 0..6usize {
                let a0 = rot + PI / 6.0 + 2.0 * PI * k as f32 / 6.0;
                let a1 = rot + PI / 6.0 + 2.0 * PI * (k + 1) as f32 / 6.0;
                let (x0, y0) = polar(cx, cy, hex_r, a0);
                let (x1, y1) = polar(cx, cy, hex_r, a1);
                line(grid, x0, y0, x1, y1);
            }
        }

        // Center bindu
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

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
    let styles = progress::styles::yantra::styles();
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
