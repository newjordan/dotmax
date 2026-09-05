//! `cellular` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O cellular.rs && ./cellular [style-name]
//! ```

const DEFAULT_STYLE: &str = "ca-rule30";

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
    pub mod cellular {
//! Cellular-automata progress bars.
//!
//! Every style in this module is a pure function of `(ctx.eased, ctx.time)` —
//! no mutable state is stored between frames. The CA state at each frame is
//! recomputed from a fixed initial condition, with the number of generations
//! evolved being a deterministic function of `eased` (for progress) and/or
//! `time` (for animation).
//!
//! Included algorithms:
//! - Wolfram elementary 1D CA (Rule 30, Rule 90, Rule 110, Rule 184, Rule 54, Rule 150)
//! - Conway's Game of Life (glider / r-pentomino seeds)
//! - Brian's Brain (3-state excitable CA)
//! - Langton's Ant
//! - Gray-Scott reaction-diffusion (approximated)
//! - Cyclic / rock-paper-scissors CA
//! - Wireworld electron loop
//! - Forest fire model

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};

// ---------------------------------------------------------------------------
// Deterministic hash (no external crates)
// ---------------------------------------------------------------------------

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hash_f(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

/// Mix two seeds into a hash (useful for 2D coordinates).
#[inline]
fn hash2(x: u32, y: u32) -> u32 {
    hash(x.wrapping_mul(1_000_003).wrapping_add(y))
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Theme tint — petri-dish green.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(96, 226, 130);
const TINT_END: Color = Color::rgb(24, 128, 88);

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

/// All styles in the `cellular` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per distinct cellular-automaton style.
/// Every style's `theme()` returns `"cellular"` and every `name()` is unique
/// within this theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(WolframRule { rule: 30 })),
        Box::new(Tinted(WolframRule { rule: 90 })),
        Box::new(Tinted(WolframRule { rule: 110 })),
        Box::new(Tinted(WolframRule { rule: 184 })),
        Box::new(Tinted(WolframRule { rule: 54 })),
        Box::new(Tinted(WolframRule { rule: 150 })),
        Box::new(Tinted(GameOfLife {
            seed: GoLSeed::Glider,
        })),
        Box::new(Tinted(GameOfLife {
            seed: GoLSeed::RPentomino,
        })),
        Box::new(Tinted(BriansBrain)),
        Box::new(Tinted(LangtonsAnt)),
        Box::new(Tinted(CyclicCA)),
        Box::new(Tinted(Wireworld)),
        Box::new(Tinted(GrayScott)),
        Box::new(Tinted(ForestFire)),
    ]
}

// ===========================================================================
// 1–6: Wolfram Elementary 1D CA (Rules 30, 90, 110, 184, 54, 150)
// ===========================================================================
//
// Layout: the dot grid is treated as a 1D tape (width) evolving over time
// (height). Row 0 is the initial condition (single centre-cell alive). Each
// subsequent row is one generation. We reveal rows top-to-bottom up to
// `ceil(eased * h)` rows. Time shifts a horizontal scroll offset so the
// pattern slowly drifts when the bar is held at a fixed progress.

struct WolframRule {
    rule: u8,
}

impl WolframRule {
    /// Apply one generation of the elementary CA rule.
    /// `row` must be length `w`; returns a new row of the same length.
    fn step(row: &[u8], rule: u8) -> Vec<u8> {
        let w = row.len();
        (0..w)
            .map(|i| {
                let l = if i == 0 { row[w - 1] } else { row[i - 1] };
                let c = row[i];
                let r = if i + 1 == w { row[0] } else { row[i + 1] };
                let pattern = (l << 2) | (c << 1) | r;
                (rule >> pattern) & 1
            })
            .collect()
    }
}

impl ProgressStyle for WolframRule {
    fn name(&self) -> &str {
        match self.rule {
            30 => "ca-rule30",
            90 => "ca-rule90",
            110 => "ca-rule110",
            184 => "ca-rule184",
            54 => "ca-rule54",
            150 => "ca-rule150",
            _ => "ca-wolfram",
        }
    }

    fn theme(&self) -> &str {
        "cellular"
    }

    fn describe(&self) -> &str {
        match self.rule {
            30 => "Rule 30: chaotic braille texture — Wolfram's fractal entropy engine",
            90 => "Rule 90: Sierpinski triangle — XOR diffusion fills the bar row by row",
            110 => "Rule 110: Turing-complete edge of chaos — complex local structures emerge",
            184 => "Rule 184: traffic flow CA — particles drift rightward with eased density",
            54 => "Rule 54: nested gliders — quasi-periodic wave fronts cascade down the bar",
            150 => "Rule 150: additive XOR diffusion — Pascal's triangle mod 2 in braille",
            _ => "Wolfram elementary CA",
        }
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Rows to reveal: 0 = empty, h = fully revealed.
        let reveal = (ctx.eased * h as f32).ceil() as usize;
        // Slow horizontal drift driven by time (wraps around tape). Rule 184
        // laps the tape exactly once per 4 s loop so the traffic flows
        // seamlessly; the other rules keep their subtle drift.
        let scroll = if self.rule == 184 {
            ((ctx.time * 0.25).fract() * w as f32) as usize % w
        } else {
            (ctx.time * 0.3) as usize % w.max(1)
        };

        // Seed: single live cell at the centre — except Rule 184 (traffic
        // flow), where a lone car is invisible: seed a whole rush hour whose
        // density rises with eased, so jams condense as the bar fills.
        let mut row = vec![0u8; w];
        if self.rule == 184 {
            let density = 0.30 + 0.45 * ctx.eased;
            for (x, cell) in row.iter_mut().enumerate() {
                if hash_f(x as u32 * 3 + 11) < density {
                    *cell = 1;
                }
            }
        } else {
            row[w / 2] = 1;
        }

        for gen in 0..reveal.min(h) {
            // Draw this generation into dot row `gen`.
            for x in 0..w {
                let sx = (x + scroll) % w;
                if row[sx] == 1 {
                    draw::dot(grid, x, gen);
                }
            }
            row = Self::step(&row, self.rule);
        }

        Ok(())
    }
}

// ===========================================================================
// 7–8: Conway's Game of Life
// ===========================================================================
//
// We run GoL on a cell grid of `w × h` dot-pixels (not braille cells).
// Seed is placed at a canonical position. Generations = floor(time * speed).
// `eased` gates how much of the grid is rendered (left-to-right reveal mask).

#[derive(Clone, Copy)]
enum GoLSeed {
    Glider,
    RPentomino,
}

struct GameOfLife {
    seed: GoLSeed,
}

impl GameOfLife {
    fn make_board(w: usize, h: usize, seed: GoLSeed) -> Vec<Vec<bool>> {
        let mut board = vec![vec![false; w]; h];
        match seed {
            GoLSeed::Glider => {
                // Standard glider pattern, placed top-left.
                // . X .
                // . . X
                // X X X
                let patterns: &[(usize, usize)] = &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
                for &(px, py) in patterns {
                    if px < w && py < h {
                        board[py][px] = true;
                    }
                }
            }
            GoLSeed::RPentomino => {
                // r-pentomino: . X X / X X . / . X .  — centred.
                let cx = w / 2;
                let cy = h / 2;
                let patterns: &[(i32, i32)] = &[(1, -1), (2, -1), (0, 0), (1, 0), (1, 1)];
                for &(dx, dy) in patterns {
                    let px = cx as i32 + dx;
                    let py = cy as i32 + dy;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        board[py as usize][px as usize] = true;
                    }
                }
            }
        }
        board
    }

    fn step(board: &[Vec<bool>]) -> Vec<Vec<bool>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let mut next = vec![vec![false; w]; h];
        for y in 0..h {
            for x in 0..w {
                let mut n = 0u8;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                        let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                        if board[ny][nx] {
                            n += 1;
                        }
                    }
                }
                next[y][x] = if board[y][x] {
                    n == 2 || n == 3
                } else {
                    n == 3
                };
            }
        }
        next
    }
}

impl ProgressStyle for GameOfLife {
    fn name(&self) -> &str {
        match self.seed {
            GoLSeed::Glider => "ca-gol-glider",
            GoLSeed::RPentomino => "ca-gol-rpentomino",
        }
    }

    fn theme(&self) -> &str {
        "cellular"
    }

    fn describe(&self) -> &str {
        match self.seed {
            GoLSeed::Glider =>
                "Game of Life glider — a diagonal spaceship animates across the bar as progress fills",
            GoLSeed::RPentomino =>
                "Game of Life r-pentomino — explosive chaotic growth, progress gates the reveal column",
        }
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Speed: glider moves ~1 cell/5 gens; r-pentomino expands faster.
        let speed = match self.seed {
            GoLSeed::Glider => 4.0f32,
            GoLSeed::RPentomino => 8.0f32,
        };
        let gens = (ctx.time * speed) as usize;
        // Cap at a reasonable bound to prevent runaway cost.
        let gens = gens.min(500);

        let mut board = Self::make_board(w, h, self.seed);
        for _ in 0..gens {
            board = Self::step(&board);
        }

        // eased controls a left-to-right reveal column.
        let reveal_x = (ctx.eased * w as f32).round() as usize;

        for (y, row) in board.iter().enumerate().take(h) {
            for x in 0..reveal_x.min(w) {
                if x < row.len() && row[x] {
                    draw::dot(grid, x, y);
                }
            }
        }

        // A sparse live population can leave the bar unreadable, so a dotted
        // baseline plus an edge tick carries the progress reading.
        for x in (0..reveal_x.min(w)).step_by(3) {
            draw::dot(grid, x, h - 1);
        }
        let edge = reveal_x.min(w.saturating_sub(1));
        draw::vline(grid, edge, h.saturating_sub(3), h - 1);

        Ok(())
    }
}

// ===========================================================================
// 9: Brian's Brain
// ===========================================================================
//
// 3-state excitable CA: OFF(0), FIRING(1), REFRACTORY(2).
// A cell fires if exactly 2 FIRING neighbours; fires→refractory→off.
// Seeded with a deterministic hash pattern. Generations driven by time.

struct BriansBrain;

impl BriansBrain {
    fn initial(w: usize, h: usize) -> Vec<Vec<u8>> {
        let mut board = vec![vec![0u8; w]; h];
        // Scatter some firing seeds using the hash function.
        for (y, row) in board.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let v = hash2(x as u32, y as u32) % 5;
                *cell = u8::from(v == 0);
            }
        }
        board
    }

    fn step(board: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let mut next = vec![vec![0u8; w]; h];
        for y in 0..h {
            for x in 0..w {
                let state = board[y][x];
                next[y][x] = match state {
                    1 => 2, // firing → refractory
                    2 => 0, // refractory → off
                    _ => {
                        // off → firing if exactly 2 firing neighbours
                        let mut n = 0u8;
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                                let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                                if board[ny][nx] == 1 {
                                    n += 1;
                                }
                            }
                        }
                        u8::from(n == 2)
                    }
                };
            }
        }
        next
    }
}

impl ProgressStyle for BriansBrain {
    fn name(&self) -> &str {
        "ca-brians-brain"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Brian's Brain: 3-state excitable CA — firing waves pulse and swirl as progress advances"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        let gens = ((ctx.time * 6.0) as usize).min(300);
        let mut board = Self::initial(w, h);
        for _ in 0..gens {
            board = Self::step(&board);
        }

        // progress controls a diagonal reveal: cells with (x+y)/max_sum <= eased are shown.
        let max_sum = (w + h).saturating_sub(2).max(1);
        for (y, row) in board.iter().enumerate().take(h) {
            for x in 0..w {
                let reveal_frac = (x + y) as f32 / max_sum as f32;
                if reveal_frac <= ctx.eased && x < row.len() && row[x] == 1 {
                    draw::dot(grid, x, y);
                }
            }
        }

        Ok(())
    }
}

// ===========================================================================
// 10: Langton's Ant
// ===========================================================================
//
// Ant turns right on white (0), left on black (1), flips cell, moves forward.
// We run `eased * MAX_STEPS` steps from a blank grid, then render the trail.
// `time` adds a small periodic "wander" offset to produce animation.

struct LangtonsAnt;

impl LangtonsAnt {
    const MAX_STEPS: usize = 2000;
}

impl ProgressStyle for LangtonsAnt {
    fn name(&self) -> &str {
        "ca-langtons-ant"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Langton's Ant: deterministic trail grows with progress — the highway emerges near 10,000 steps"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        let steps = (ctx.eased * Self::MAX_STEPS as f32) as usize;
        // Time animates a slow starting offset (cycles through small offsets).
        let time_offset = (ctx.time * 0.5) as usize % 8;
        let steps = steps.saturating_add(time_offset).min(Self::MAX_STEPS + 8);

        let mut cells = vec![0u8; w * h];
        // Ant starts at centre.
        let mut ax = (w / 2) as i32;
        let mut ay = (h / 2) as i32;
        // Direction: 0=up, 1=right, 2=down, 3=left.
        let mut dir = 0i32;

        for _ in 0..steps {
            // Clamp ant to grid (toroidal wrap).
            ax = ax.rem_euclid(w as i32);
            ay = ay.rem_euclid(h as i32);
            let idx = ay as usize * w + ax as usize;
            let c = cells[idx];
            if c == 0 {
                dir = (dir + 1).rem_euclid(4); // turn right
                cells[idx] = 1;
            } else {
                dir = (dir - 1).rem_euclid(4); // turn left
                cells[idx] = 0;
            }
            // Move forward.
            match dir {
                0 => ay -= 1,
                1 => ax += 1,
                2 => ay += 1,
                _ => ax -= 1,
            }
        }

        for y in 0..h {
            for x in 0..w {
                if cells[y * w + x] == 1 {
                    draw::dot(grid, x, y);
                }
            }
        }

        Ok(())
    }
}

// ===========================================================================
// 11: Cyclic / Rock-Paper-Scissors CA
// ===========================================================================
//
// Each cell holds a state in [0, N). A cell advances (+1 mod N) if at least
// one of its 8 neighbours is in the next state. Spiral waves emerge.
// N=8 states. Driven by time for animation, eased for density of initial fill.

struct CyclicCA;

impl CyclicCA {
    const N_STATES: u8 = 8;
    const THRESHOLD: usize = 3; // neighbours needed to advance

    fn initial(w: usize, h: usize, density_seed: u32) -> Vec<Vec<u8>> {
        let n = Self::N_STATES;
        (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| {
                        // Use density_seed to vary initial state distribution.
                        let v = hash2(x as u32 ^ density_seed, y as u32);
                        (v % n as u32) as u8
                    })
                    .collect()
            })
            .collect()
    }

    fn step(board: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let n = CyclicCA::N_STATES;
        let thresh = CyclicCA::THRESHOLD;
        let mut next = board.to_vec();
        for y in 0..h {
            for x in 0..w {
                let cur = board[y][x];
                let target = (cur + 1) % n;
                let mut count = 0usize;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                        let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                        if board[ny][nx] == target {
                            count += 1;
                        }
                    }
                }
                if count >= thresh {
                    next[y][x] = target;
                }
            }
        }
        next
    }
}

impl ProgressStyle for CyclicCA {
    fn name(&self) -> &str {
        "ca-cyclic"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Cyclic CA (rock-paper-scissors): spiral waves of 8 states — progress advances the generation count"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Wave density is roughly constant at any generation count, so the
        // generation count alone cannot carry the progress reading. Run the
        // waves on time and gate the field behind a left-to-right reveal.
        let gens = 8 + (ctx.time * 3.0) as usize;
        let seed = 7u32;
        let mut board = Self::initial(w, h, seed);
        for _ in 0..gens.min(80) {
            board = Self::step(&board);
        }

        let reveal_x = (ctx.eased * w as f32).round() as usize;

        // Draw cells that are in "high" states (upper half of N_STATES range).
        let half = Self::N_STATES / 2;
        for (y, row) in board.iter().enumerate().take(h) {
            for x in 0..reveal_x.min(w) {
                if x < row.len() && row[x] >= half {
                    draw::dot(grid, x, y);
                }
            }
        }
        let edge = reveal_x.min(w.saturating_sub(1));
        draw::vline(grid, edge, 0, h - 1);

        Ok(())
    }
}

// ===========================================================================
// 12: Wireworld
// ===========================================================================
//
// 4 states: EMPTY(0), CONDUCTOR(1), ELECTRON_HEAD(2), ELECTRON_TAIL(3).
// Rules:
//   EMPTY       → EMPTY
//   CONDUCTOR   → ELECTRON_HEAD if 1 or 2 electron heads in neighbourhood, else CONDUCTOR
//   ELECTRON_HEAD → ELECTRON_TAIL
//   ELECTRON_TAIL → CONDUCTOR
//
// We draw a loop of conductor around the perimeter of the dot grid, inject
// an electron, and let it chase itself. `eased` controls how much of the
// perimeter the conductor covers; `time` drives the electron position.

struct Wireworld;

impl Wireworld {
    const EMPTY: u8 = 0;
    const CONDUCTOR: u8 = 1;
    const HEAD: u8 = 2;
    const TAIL: u8 = 3;

    fn build_grid(w: usize, h: usize, fill_frac: f32) -> Vec<Vec<u8>> {
        let mut board = vec![vec![Self::EMPTY; w]; h];
        // Perimeter loop: top, right, bottom (reversed), left (reversed).
        let perimeter: Vec<(usize, usize)> = {
            let mut p = Vec::new();
            // Top row left→right.
            for x in 0..w {
                p.push((x, 0));
            }
            // Right column top→bottom (skip corners already added).
            for y in 1..h {
                p.push((w.saturating_sub(1), y));
            }
            // Bottom row right→left (skip corner).
            if h > 1 {
                let y = h - 1;
                for x in (0..w.saturating_sub(1)).rev() {
                    p.push((x, y));
                }
            }
            // Left column bottom→top (skip corners).
            if w > 1 && h > 1 {
                for y in (1..h.saturating_sub(1)).rev() {
                    p.push((0, y));
                }
            }
            p
        };
        let total = perimeter.len().max(1);
        let lit = (fill_frac * total as f32) as usize;

        for &(x, y) in perimeter.iter().take(lit.min(total)) {
            board[y][x] = Self::CONDUCTOR;
        }
        board
    }

    fn inject_electron(board: &mut [Vec<u8>], electron_pos: usize) {
        // Walk the perimeter to find the conductor cell at position `electron_pos`.
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let mut idx = 0usize;
        // Top row.
        if let Some(top) = board.first_mut() {
            for cell in top {
                if *cell == Self::CONDUCTOR {
                    if idx == electron_pos {
                        *cell = Self::HEAD;
                        return;
                    }
                    idx += 1;
                }
            }
        }
        // Right column.
        let rx = w.saturating_sub(1);
        for row in board.iter_mut().skip(1) {
            if row[rx] == Self::CONDUCTOR {
                if idx == electron_pos {
                    row[rx] = Self::HEAD;
                    return;
                }
                idx += 1;
            }
        }
        // Bottom row reversed.
        if h > 1 {
            let by = h - 1;
            for x in (0..w.saturating_sub(1)).rev() {
                if board[by][x] == Self::CONDUCTOR {
                    if idx == electron_pos {
                        board[by][x] = Self::HEAD;
                        return;
                    }
                    idx += 1;
                }
            }
        }
        // Left column reversed.
        if w > 1 && h > 1 {
            for y in (1..h.saturating_sub(1)).rev() {
                if board[y][0] == Self::CONDUCTOR {
                    if idx == electron_pos {
                        board[y][0] = Self::HEAD;
                        return;
                    }
                    idx += 1;
                }
            }
        }
    }

    fn step(board: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let mut next = board.to_vec();
        for y in 0..h {
            for x in 0..w {
                next[y][x] = match board[y][x] {
                    Self::HEAD => Self::TAIL,
                    Self::TAIL => Self::CONDUCTOR,
                    Self::CONDUCTOR => {
                        let mut heads = 0u8;
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                                let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                                if board[ny][nx] == Self::HEAD {
                                    heads += 1;
                                }
                            }
                        }
                        if heads == 1 || heads == 2 {
                            Self::HEAD
                        } else {
                            Self::CONDUCTOR
                        }
                    }
                    _ => Self::EMPTY,
                };
            }
        }
        next
    }
}

impl ProgressStyle for Wireworld {
    fn name(&self) -> &str {
        "ca-wireworld"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Wireworld: an electron chases itself around a conductor perimeter — conductor grows with progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Build conductor loop sized by eased.
        let mut board = Self::build_grid(w, h, ctx.eased);

        // Perimeter length (approximate) — used to position electron.
        let perimeter_len = (2 * (w + h)).saturating_sub(4).max(1);

        // Place HEAD at position driven by time.
        let head_pos = (ctx.time * 8.0) as usize % perimeter_len;
        let tail_pos = head_pos.saturating_sub(1) % perimeter_len;
        Self::inject_electron(&mut board, head_pos);
        Self::inject_electron(&mut board, tail_pos);

        // Step a few generations so the electron is properly propagated.
        for _ in 0..2 {
            board = Self::step(&board);
        }

        for (y, row) in board.iter().enumerate().take(h) {
            for x in 0..w {
                if x < row.len() {
                    match row[x] {
                        Self::CONDUCTOR => draw::dot(grid, x, y),
                        Self::HEAD => {
                            // Draw HEAD brighter by also dotting adjacent positions.
                            draw::dot(grid, x, y);
                            if x + 1 < w {
                                draw::dot(grid, x + 1, y);
                            }
                            if x > 0 {
                                draw::dot(grid, x.saturating_sub(1), y);
                            }
                        }
                        // TAIL is invisible (just went dark), as is EMPTY.
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

// ===========================================================================
// 13: Gray-Scott reaction-diffusion
// ===========================================================================
//
// Two chemicals u, v diffuse and react: u + 2v -> 3v, with feed rate F and
// kill rate k. Seeded with a central blob and integrated for a number of
// steps that grows with `eased` — coral / mitosis labyrinths emerge. Stateless:
// the whole simulation is recomputed from the same seed every frame.

struct GrayScott;
impl ProgressStyle for GrayScott {
    fn name(&self) -> &str {
        "ca-gray-scott"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Gray-Scott reaction-diffusion: coral/mitosis labyrinths grow as progress feeds the reaction"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);
        let n = w * h;
        let mut u = vec![1.0f32; n];
        let mut v = vec![0.0f32; n];
        // Seed a small central square of v.
        let cx = w / 2;
        let cy = h / 2;
        for dy in 0..3 {
            for dx in 0..3 {
                let x = (cx + dx).saturating_sub(1);
                let y = (cy + dy).saturating_sub(1);
                if x < w && y < h {
                    v[y * w + x] = 1.0;
                    u[y * w + x] = 0.5;
                }
            }
        }
        const F: f32 = 0.055;
        const K: f32 = 0.062;
        const DU: f32 = 0.16;
        const DV: f32 = 0.08;
        let iters = ((ctx.eased * 60.0) as usize + 1).min(90);
        let mut un = u.clone();
        let mut vn = v.clone();
        for _ in 0..iters {
            for y in 0..h {
                let ym = if y == 0 { h - 1 } else { y - 1 };
                let yp = if y + 1 == h { 0 } else { y + 1 };
                for x in 0..w {
                    let xm = if x == 0 { w - 1 } else { x - 1 };
                    let xp = if x + 1 == w { 0 } else { x + 1 };
                    let i = y * w + x;
                    let lap_u =
                        u[y * w + xm] + u[y * w + xp] + u[ym * w + x] + u[yp * w + x] - 4.0 * u[i];
                    let lap_v =
                        v[y * w + xm] + v[y * w + xp] + v[ym * w + x] + v[yp * w + x] - 4.0 * v[i];
                    let uvv = u[i] * v[i] * v[i];
                    un[i] = (u[i] + DU * lap_u - uvv + F * (1.0 - u[i])).clamp(0.0, 1.0);
                    vn[i] = (v[i] + DV * lap_v + uvv - (F + K) * v[i]).clamp(0.0, 1.0);
                }
            }
            std::mem::swap(&mut u, &mut un);
            std::mem::swap(&mut v, &mut vn);
        }
        for y in 0..h {
            for x in 0..w {
                if v[y * w + x] > 0.25 {
                    draw::dot(grid, x, y);
                }
            }
        }
        Ok(())
    }
}

// ===========================================================================
// 14: Forest-fire model
// ===========================================================================
//
// Trees are scattered by a fixed hash; a fire front sweeps left-to-right with
// `eased` (the burn extent = progress). Trees ahead of the front stand;
// trees at the front flicker as flames (animated by `time`); behind it is ash.

struct ForestFire;
impl ProgressStyle for ForestFire {
    fn name(&self) -> &str {
        "ca-forest-fire"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Forest-fire CA: a flame front burns through scattered trees as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);
        let front = (ctx.eased * w as f32) as i32;
        for y in 0..h {
            for x in 0..w {
                // Tree if the hash for this cell clears the density threshold.
                if hash_f((y * w + x) as u32 ^ 0x9e37) >= 0.55 {
                    continue;
                }
                let d = x as i32 - front;
                if d > 1 {
                    // Standing tree ahead of the fire.
                    draw::dot(grid, x, y);
                    if y > 0 && hash_f((x * 7 + y) as u32) > 0.5 {
                        draw::dot(grid, x, y - 1);
                    }
                } else if d >= -2 {
                    // Flame front: flicker upward with time.
                    let flick = ((ctx.time * 12.0 + (x + y) as f32).sin() * 2.0) as i32;
                    draw::dot_i(grid, x as i32, y as i32 - flick.abs());
                    draw::dot(grid, x, y);
                }
                // Behind the front (d < -2): ash, drawn empty.
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
    let styles = progress::styles::cellular::styles();
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
