//! `spinner` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O spinner.rs && ./spinner [style-name]
//! ```

const DEFAULT_STYLE: &str = "braille-spin";

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
    pub mod spinner {
//! Indeterminate spinner / busy-indicator styles.
//!
//! Every spinner here is **time-driven**: the animation runs purely from
//! `ctx.time` (seconds elapsed) and looks alive at any fixed progress value.
//! `ctx.progress` is used only as a subtle modifier (e.g. spin-rate scaling)
//! where noted. All spinners are centered in whatever grid they are given and
//! are safe at 1×1 cells.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─── registry ────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — cool spinner cyan. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(84, 222, 255);
const TINT_END: Color = Color::rgb(128, 144, 255);

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

/// All styles in the `spinner` theme.
///
/// Returns 14 structurally distinct spinners in a canonical order. Each entry
/// is a heap-allocated [`ProgressStyle`] suitable for direct use with
/// [`crate::progress::render_lines`] or a [`crate::TerminalRenderer`].
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(BrailleSpinner)),
        Box::new(DotRing),
        Box::new(Tinted(ArcSweep)),
        Box::new(Tinted(DualArc)),
        Box::new(Bounce),
        Box::new(Pulse),
        Box::new(Orbit),
        Box::new(Tinted(ClockHand)),
        Box::new(Tinted(Radar)),
        Box::new(Ellipsis),
        Box::new(GrowingArc),
        Box::new(SquareRunner),
        Box::new(Tinted(SpinnerBars)),
        Box::new(HourglassFlip),
    ]
}

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Return the dot-space center of the grid as `(cx, cy)` floats.
#[inline]
fn dot_center(grid: &BrailleGrid) -> (f32, f32) {
    let (dw, dh) = draw::dot_dims(grid);
    (dw as f32 * 0.5 - 0.5, dh as f32 * 0.5 - 0.5)
}

/// Draw a single dot from a polar angle `theta` and radius `r`, centered.
#[inline]
fn dot_polar(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, theta: f32) {
    let x = cx + r * theta.cos();
    let y = cy + r * theta.sin();
    draw::dot_i(grid, x.round() as i32, y.round() as i32);
}

/// Draw an arc from `theta_start` to `theta_end` with `steps` sample points.
fn draw_arc(
    grid: &mut BrailleGrid,
    cx: f32,
    cy: f32,
    r: f32,
    theta_start: f32,
    theta_end: f32,
    steps: u32,
) {
    if steps == 0 {
        return;
    }
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let theta = theta_start + t * (theta_end - theta_start);
        dot_polar(grid, cx, cy, r, theta);
    }
}

/// Draw a radial line from `(cx,cy)` toward angle `theta`, length `len` dots.
fn draw_radial(grid: &mut BrailleGrid, cx: f32, cy: f32, len: f32, theta: f32) {
    let steps = (len.ceil() as u32).max(1);
    for i in 0..=steps {
        let r = i as f32 * len / steps as f32;
        dot_polar(grid, cx, cy, r, theta);
    }
}

// ─── 1. BrailleSpinner ───────────────────────────────────────────────────────

/// Classic 8-frame braille spinner glyph cycling at the grid center.
struct BrailleSpinner;

const BRAILLE_FRAMES: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

impl ProgressStyle for BrailleSpinner {
    fn name(&self) -> &str {
        "braille-spin"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Classic ⠋⠙⠹⠸⠼⠴⠦⠧ single-cell braille glyph cycling at the center"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        // Speed subtly scales with progress (faster when more progress).
        let rate = 10.0 + ctx.progress * 6.0;
        let frame = (ctx.time * rate) as usize % BRAILLE_FRAMES.len();
        let cx = cw / 2;
        let cy = ch / 2;
        draw::glyph(grid, cx, cy, BRAILLE_FRAMES[frame]);
        Ok(())
    }
}

// ─── 2. DotRing ──────────────────────────────────────────────────────────────

/// N dots on a circle; one bright head rotates, leaving a dimming comet tail.
struct DotRing;

impl ProgressStyle for DotRing {
    fn name(&self) -> &str {
        "dot-ring"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Rotating comet head on a dot ring — bright lead, fading tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let r = (dw.min(dh) as f32 * 0.35).max(1.0);
        let n: usize = 12;
        let tail_len: usize = 5;

        // Head angle advances clockwise.
        let head_angle = ctx.time * 2.5 * 2.0 * PI;
        let head_idx = (head_angle / (2.0 * PI) * n as f32) as usize % n;

        for i in 0..n {
            // Is this dot within tail_len steps behind the head?
            let behind = (head_idx + n - i) % n;
            if behind < tail_len {
                // tail_len=0 is head (full), tail_len-1 is faintest
                let theta = 2.0 * PI * i as f32 / n as f32;
                // Draw the dot using dot_polar — head always drawn, tail drawn
                // with decreasing density (we just draw every dot; shade varies
                // by skipping for far tail members).
                // Simplest: draw head, draw first half of tail as full dots.
                if behind == 0 || behind < tail_len / 2 + 1 {
                    dot_polar(grid, cx, cy, r, theta);
                } else {
                    // Outer tail: only draw if even index (creates spacing = dimming)
                    if i % 2 == 0 {
                        dot_polar(grid, cx, cy, r, theta);
                    }
                }
            }
            // Non-tail dots: draw a faint ring placeholder using every 3rd dot
            else if i % 3 == 0 {
                let theta = 2.0 * PI * i as f32 / n as f32;
                dot_polar(grid, cx, cy, r, theta);
            }
        }

        // Tint the head bright if color available.
        let head_theta = 2.0 * PI * head_idx as f32 / n as f32;
        let hx = (cx + r * head_theta.cos()).round() as i32;
        let hy = (cy + r * head_theta.sin()).round() as i32;
        if hx >= 0 && hy >= 0 {
            let cell_x = (hx as usize) / 2;
            let cell_y = (hy as usize) / 4;
            let color = ctx.palette.sample(1.0);
            let (cw, ch) = grid.dimensions();
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }
        Ok(())
    }
}

// ─── 3. ArcSweep ─────────────────────────────────────────────────────────────

/// A quarter-arc rotating around the center — single sweeping arc.
struct ArcSweep;

impl ProgressStyle for ArcSweep {
    fn name(&self) -> &str {
        "arc-sweep"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "A quarter-arc sweeping clockwise around the center"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let r = (dw.min(dh) as f32 * 0.38).max(1.0);
        let arc_span = PI * 0.5; // quarter turn
        let head = ctx.time * 2.0 * PI * 0.8;
        let steps = ((r * arc_span).ceil() as u32).max(2).min(32);
        draw_arc(grid, cx, cy, r, head, head + arc_span, steps);
        Ok(())
    }
}

// ─── 4. DualArc ──────────────────────────────────────────────────────────────

/// Two counter-rotating half-arcs on the same circle.
struct DualArc;

impl ProgressStyle for DualArc {
    fn name(&self) -> &str {
        "dual-arc"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Two half-arcs spinning in opposite directions on the same ring"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let r = (dw.min(dh) as f32 * 0.38).max(1.0);
        let arc_span = PI * 0.55;
        let t = ctx.time * 2.0 * PI * 0.7;
        let steps = ((r * arc_span).ceil() as u32).max(2).min(32);
        // Forward arc.
        draw_arc(grid, cx, cy, r, t, t + arc_span, steps);
        // Counter-rotating arc (offset by PI).
        draw_arc(grid, cx, cy, r, -t + PI, -t + PI + arc_span, steps);
        Ok(())
    }
}

// ─── 5. Bounce ───────────────────────────────────────────────────────────────

/// A dot bouncing left-right (Cylon/Knight Rider scanner) along the center row.
struct Bounce;

impl ProgressStyle for Bounce {
    fn name(&self) -> &str {
        "bounce"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Knight Rider: a dot bouncing left-right along the center with a fading trail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cy = dh / 2;
        // Sine oscillates -1..1, map to 0..dw-1.
        let phase = (ctx.time * PI * 0.9).sin(); // -1..1
        let head_x = ((phase * 0.5 + 0.5) * (dw.saturating_sub(1)) as f32).round() as usize;
        // Trail: draw 4 dots fading toward head.
        let trail_len: i32 = 5;
        let direction_sign: i32 = if phase >= 0.0 { 1 } else { -1 };
        for i in 0..trail_len {
            let tx = head_x as i32 - direction_sign * i;
            if i == 0 {
                draw::dot_i(grid, tx, cy as i32);
            } else if i < trail_len / 2 + 1 {
                draw::dot_i(grid, tx, cy as i32);
            } else if i % 2 == 0 {
                draw::dot_i(grid, tx, cy as i32);
            }
        }
        // Tint head cell.
        let cell_x = head_x / 2;
        let cell_y = cy / 4;
        let (cw, ch) = grid.dimensions();
        if cell_x < cw && cell_y < ch {
            let color = ctx.palette.sample(head_x as f32 / dw.max(1) as f32);
            draw::tint_row(grid, cell_y, cell_x, cell_x, color);
        }
        Ok(())
    }
}

// ─── 6. Pulse ────────────────────────────────────────────────────────────────

/// A circle that expands and contracts (breathing) with time.
struct Pulse;

impl ProgressStyle for Pulse {
    fn name(&self) -> &str {
        "pulse"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Breathing circle that expands and contracts with a sine rhythm"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let max_r = (dw.min(dh) as f32 * 0.40).max(1.0);
        // Breathe: r oscillates between 30% and 100% of max_r.
        let breath = (ctx.time * PI * 0.7).sin() * 0.5 + 0.5; // 0..1
        let r = max_r * (0.3 + breath * 0.7);
        let steps = ((2.0 * PI * r).ceil() as u32).max(4).min(64);
        draw_arc(grid, cx, cy, r, 0.0, 2.0 * PI, steps);
        // Tint proportionally to pulse phase.
        let (cw, ch) = grid.dimensions();
        let mid_cy = ch / 2;
        if ch > 0 && cw > 0 {
            let color = ctx.palette.sample(breath);
            draw::tint_row(grid, mid_cy, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─── 7. Orbit ────────────────────────────────────────────────────────────────

/// A small moon orbiting a fixed center dot.
struct Orbit;

impl ProgressStyle for Orbit {
    fn name(&self) -> &str {
        "orbit"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Moon orbiting a stationary center planet"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        // Center dot (planet).
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

        let r = (dw.min(dh) as f32 * 0.35).max(1.5);
        let theta = ctx.time * 2.0 * PI * 0.6;
        // Moon.
        dot_polar(grid, cx, cy, r, theta);
        // Short trail behind moon.
        for i in 1..3usize {
            let trail_theta = theta - i as f32 * (PI / 8.0);
            if i == 1 {
                dot_polar(grid, cx, cy, r, trail_theta);
            }
        }
        // Tint moon cell.
        let mx = (cx + r * theta.cos()).round() as i32;
        let my = (cy + r * theta.sin()).round() as i32;
        if mx >= 0 && my >= 0 {
            let cell_x = (mx as usize) / 2;
            let cell_y = (my as usize) / 4;
            let (cw, ch) = grid.dimensions();
            if cell_x < cw && cell_y < ch {
                let color = ctx.palette.sample(0.8);
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }
        Ok(())
    }
}

// ─── 8. ClockHand ────────────────────────────────────────────────────────────

/// A single radial line sweeping like a clock hand.
struct ClockHand;

impl ProgressStyle for ClockHand {
    fn name(&self) -> &str {
        "clock-hand"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Clock-hand: a single radial spoke sweeping 360° with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let len = (dw.min(dh) as f32 * 0.40).max(1.0);
        // Clockwise from 12 o'clock (−π/2).
        let theta = ctx.time * 2.0 * PI * 0.5 - PI / 2.0;
        draw_radial(grid, cx, cy, len, theta);
        // Center hub dot.
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
        Ok(())
    }
}

// ─── 9. Radar ────────────────────────────────────────────────────────────────

/// Sweeping radar line leaving a fading wedge behind it.
struct Radar;

impl ProgressStyle for Radar {
    fn name(&self) -> &str {
        "radar"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Radar sweep: rotating spoke with a fading wedge afterglow"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let r = (dw.min(dh) as f32 * 0.40).max(1.0);
        let head = ctx.time * 2.0 * PI * 0.55;

        // Fading wedge: 4 ghost spokes behind the head.
        for ghost in 1..=4usize {
            let fade_angle = head - ghost as f32 * (PI / 10.0);
            // Only draw every-other dot on ghosts to imply fading.
            let steps = ((r * 0.8).ceil() as u32).max(1).min(20);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let gr = t * r * 0.85;
                // Skip interior dots for farthest ghosts.
                if ghost > 2 && i % 2 != 0 {
                    continue;
                }
                let gx = cx + gr * fade_angle.cos();
                let gy = cy + gr * fade_angle.sin();
                draw::dot_i(grid, gx.round() as i32, gy.round() as i32);
            }
        }

        // Solid head spoke.
        let steps = (r.ceil() as u32).max(1).min(24);
        draw_radial(grid, cx, cy, r, head);
        let _ = steps; // used above

        // Outer ring (full circle, sparse).
        let n = 24u32;
        for i in 0..n {
            if i % 4 == 0 {
                let theta = 2.0 * PI * i as f32 / n as f32;
                dot_polar(grid, cx, cy, r, theta);
            }
        }

        // Center dot.
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
        Ok(())
    }
}

// ─── 10. Ellipsis ────────────────────────────────────────────────────────────

/// Three dots that light in sequence (· ·· ···) — the "Loading..." cycle.
struct Ellipsis;

impl ProgressStyle for Ellipsis {
    fn name(&self) -> &str {
        "ellipsis"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Three-dot ellipsis cycling · ·· ··· — the classic Loading... indicator"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        // How many dots are lit: 0, 1, 2, 3, repeating every ~1.2 s.
        let phase = (ctx.time / 1.2).fract(); // 0..1
        let lit = (phase * 4.0) as usize; // 0,1,2,3

        let dot_count = 3usize;
        // Center the three glyph cells.
        let start_x = cw.saturating_sub(dot_count) / 2;
        let mid_y = ch / 2;

        // Braille ellipsis frames for each slot.
        const DARK: char = '⠂'; // faint placeholder dot
        const BRIGHT: char = '⣿'; // full braille block

        for i in 0..dot_count {
            let cx = (start_x + i).min(cw.saturating_sub(1));
            if i < lit.min(dot_count) {
                draw::glyph(grid, cx, mid_y, BRIGHT);
                let color = ctx.palette.sample(i as f32 / dot_count as f32);
                draw::tint_row(grid, mid_y, cx, cx, color);
            } else {
                draw::glyph(grid, cx, mid_y, DARK);
            }
        }
        Ok(())
    }
}

// ─── 11. GrowingArc ──────────────────────────────────────────────────────────

/// An arc that grows then shrinks as it rotates (Material / Android spinner).
struct GrowingArc;

impl ProgressStyle for GrowingArc {
    fn name(&self) -> &str {
        "growing-arc"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Material-style arc that grows then shrinks as it rotates around the ring"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let r = (dw.min(dh) as f32 * 0.38).max(1.0);

        // Period = 2 s: first half arc grows 10°→270°, second half it shrinks.
        let cycle = (ctx.time * 0.5).fract(); // 0..1 per 2 s
        let arc_frac = if cycle < 0.5 {
            // growing: 0 → full
            cycle * 2.0
        } else {
            // shrinking: full → 0
            1.0 - (cycle - 0.5) * 2.0
        };
        let arc_span = (PI / 18.0) + arc_frac * (PI * 1.4); // 10° to ~252°

        // Head rotates continuously.
        let head = ctx.time * 2.0 * PI * 0.4;
        let steps = ((r * arc_span).ceil() as u32).max(2).min(48);
        draw_arc(grid, cx, cy, r, head, head + arc_span, steps);

        // Tint the head end.
        let tip_theta = head + arc_span;
        let tx = (cx + r * tip_theta.cos()).round() as i32;
        let ty = (cy + r * tip_theta.sin()).round() as i32;
        if tx >= 0 && ty >= 0 {
            let cell_x = (tx as usize) / 2;
            let cell_y = (ty as usize) / 4;
            let (cw, ch) = grid.dimensions();
            if cell_x < cw && cell_y < ch {
                let color = ctx.palette.sample(1.0);
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }
        Ok(())
    }
}

// ─── 12. SquareRunner ────────────────────────────────────────────────────────

/// A dot running clockwise around the perimeter of a centered square.
struct SquareRunner;

impl ProgressStyle for SquareRunner {
    fn name(&self) -> &str {
        "square-runner"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "A dot chasing around the perimeter of a centered square — with outline"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Square side in dots (fit inside grid).
        let side = dw.min(dh).saturating_sub(2).max(1);
        let half = side / 2;
        let (cx, cy) = dot_center(grid);
        let x0 = (cx - half as f32).round().max(0.0) as usize;
        let y0 = (cy - half as f32).round().max(0.0) as usize;

        // Draw outline.
        draw::rect_outline(grid, x0, y0, side, side);

        // Perimeter length.
        let perim = (side.saturating_sub(1)) * 4;
        if perim == 0 {
            return Ok(());
        }

        // Position along perimeter 0..perim.
        let pos = (ctx.time * (perim as f32) * 0.5).rem_euclid(perim as f32) as usize;

        // Map pos to (x, y) — clockwise: top, right, bottom (reversed), left (reversed).
        let seg = side.saturating_sub(1).max(1);
        let (rx, ry) = if pos < seg {
            // top edge left→right
            (x0 + pos, y0)
        } else if pos < 2 * seg {
            // right edge top→bottom
            (x0 + side.saturating_sub(1), y0 + (pos - seg))
        } else if pos < 3 * seg {
            // bottom edge right→left
            (
                x0 + side.saturating_sub(1).saturating_sub(pos - 2 * seg),
                y0 + side.saturating_sub(1),
            )
        } else {
            // left edge bottom→top
            (
                x0,
                y0 + side.saturating_sub(1).saturating_sub(pos - 3 * seg),
            )
        };

        // Draw runner dot (on top of outline).
        draw::dot(grid, rx, ry);
        // Tint runner cell.
        let cell_x = rx / 2;
        let cell_y = ry / 4;
        let (cw, ch) = grid.dimensions();
        if cell_x < cw && cell_y < ch {
            let t = pos as f32 / perim.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cell_y, cell_x, cell_x, color);
        }
        Ok(())
    }
}

// ─── 13. SpinnerBars ─────────────────────────────────────────────────────────

/// A ring of short radial spokes fading in sequence — the classic throbber.
struct SpinnerBars;

impl ProgressStyle for SpinnerBars {
    fn name(&self) -> &str {
        "spinner-bars"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Ring of 8 radial spokes fading in sequence — the classic macOS throbber"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = dot_center(grid);
        let outer_r = (dw.min(dh) as f32 * 0.40).max(2.0);
        let inner_r = (outer_r * 0.45).max(1.0);
        let n: usize = 8;

        // Which spoke is the bright head.
        let head_idx = (ctx.time * n as f32 * 1.5).rem_euclid(n as f32) as usize % n;

        for i in 0..n {
            let theta = 2.0 * PI * i as f32 / n as f32 - PI / 2.0; // from 12 o'clock
                                                                   // Distance behind head (0 = head, n-1 = farthest).
            let behind = (head_idx + n - i) % n;
            // Only draw spokes within the "lit" arc (0..n/2+1).
            if behind > n / 2 {
                continue; // dark half — skip spoke
            }
            // Spokes closer to head are longer; farther ones are shorter.
            let len_frac = 1.0 - behind as f32 / (n / 2 + 1) as f32;
            let spoke_outer = inner_r + (outer_r - inner_r) * len_frac;

            // Draw spoke as dots from inner_r to spoke_outer.
            let steps = ((spoke_outer - inner_r).ceil() as u32).max(1).min(8);
            for s in 0..=steps {
                let r = inner_r + (spoke_outer - inner_r) * s as f32 / steps as f32;
                dot_polar(grid, cx, cy, r, theta);
            }
        }
        Ok(())
    }
}

// ─── 14. HourglassFlip ───────────────────────────────────────────────────────

/// An hourglass glyph that flips between ⧗ and ⧖ every ~0.8 s.
struct HourglassFlip;

const HOURGLASS_FRAMES: [char; 4] = ['⧗', '⧗', '⧖', '⧖'];

impl ProgressStyle for HourglassFlip {
    fn name(&self) -> &str {
        "hourglass-flip"
    }
    fn theme(&self) -> &str {
        "spinner"
    }
    fn describe(&self) -> &str {
        "Hourglass glyph flipping ⧗↔⧖ with a braille-dot shimmer around it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        let frame = (ctx.time * 1.25) as usize % HOURGLASS_FRAMES.len();
        let cx = cw / 2;
        let cy = ch / 2;

        // Place hourglass glyph at center.
        draw::glyph(grid, cx, cy, HOURGLASS_FRAMES[frame]);

        // Tint center cell.
        let color = ctx.palette.sample(if frame < 2 { 0.2 } else { 0.8 });
        draw::tint_row(grid, cy, cx, cx, color);

        // Braille shimmer: rotating single-dot around the glyph cell (dot space).
        let (dw, dh) = draw::dot_dims(grid);
        let dcx = dw as f32 * 0.5;
        let dcy = dh as f32 * 0.5;
        let r = 2.5_f32.max(1.0);
        let theta = ctx.time * 2.0 * PI * 1.2;
        let sx = (dcx + r * theta.cos()).round() as i32;
        let sy = (dcy + r * theta.sin()).round() as i32;
        // Only draw shimmer dot if it doesn't land on the center glyph cell.
        let shimmer_cell_x = (sx.max(0) as usize) / 2;
        let shimmer_cell_y = (sy.max(0) as usize) / 4;
        if shimmer_cell_x != cx || shimmer_cell_y != cy {
            draw::dot_i(grid, sx, sy);
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
    let styles = progress::styles::spinner::styles();
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
