//! `ocean` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O ocean.rs && ./ocean [style-name]
//! ```

const DEFAULT_STYLE: &str = "rising-tide";

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
    pub mod ocean {
//! Ocean / aquatic progress bars.
//!
//! Ten animated braille bars inspired by the sea: tides, waves, bubbles,
//! fish, sonar, jellyfish, coral, seaweed, ripple interference, and a
//! deep-ocean depth gauge. Every bar uses `ctx.time` for continuous animation
//! and `ctx.eased` for fill/advance driven by progress.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `ocean` theme.
///
/// Returns ten boxed ocean-themed bars ready to be mixed into any registry
/// or rendered directly via [`ProgressStyle::render`].
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(RisingTide),
        Box::new(BubblesRising),
        Box::new(FishSwim),
        Box::new(WaveCrest),
        Box::new(SonarPing),
        Box::new(DepthGauge),
        Box::new(JellyfishPulse),
        Box::new(CoralReef),
        Box::new(Seaweed),
        Box::new(RippleInterference),
    ]
}

// ---------------------------------------------------------------------------
// 1. Rising Tide
// ---------------------------------------------------------------------------
// Water level climbs with eased progress; the surface is a live sine wave.

struct RisingTide;
impl ProgressStyle for RisingTide {
    fn name(&self) -> &str {
        "rising-tide"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Water level rises with eased progress; animated sine-wave surface"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Water level: 0 = empty (bottom), h = full (top). eased controls it.
        let water_h = (ctx.eased * h as f32).round() as usize;
        let water_top = h.saturating_sub(water_h); // dot-y of the surface line

        // Fill body below the surface.
        if water_h > 1 {
            draw::fill_rect(grid, 0, water_top + 1, w, h.saturating_sub(water_top + 1));
        }

        // Animated surface wave — one row of sine-displaced dots.
        let amp = (h as f32 * 0.08).max(1.0);
        for x in 0..w {
            let phase = ctx.time * 2.5 + x as f32 * 0.35;
            let dy = (phase.sin() * amp).round() as i32;
            let sy = water_top as i32 + dy;
            draw::dot_i(grid, x as i32, sy);
        }

        // Tint: deep ocean blue at bottom, sea-foam cyan at surface.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let frac = 1.0 - cy as f32 / cells_h.max(1) as f32; // 0 at top, 1 at bottom
            let color = ctx.palette.sample(frac);
            let dot_y_of_cell = cy * 4;
            if dot_y_of_cell >= water_top {
                draw::tint_row(grid, cy, 0, ctx.width.saturating_sub(1), color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Bubbles Rising
// ---------------------------------------------------------------------------
// Bubbles float upward with sinusoidal lateral wobble; spawn density ∝ progress.

struct BubblesRising;
impl ProgressStyle for BubblesRising {
    fn name(&self) -> &str {
        "bubbles-rising"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Bubbles float upward with wobble; spawn density scales with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of bubbles tied to progress (minimum 1 while progress > 0).
        let n_bubbles =
            ((ctx.eased * 14.0).round() as usize).max(if ctx.progress > 0.0 { 1 } else { 0 });

        for i in 0..n_bubbles {
            // Each bubble has a fixed column origin spread across the width.
            let base_x = if n_bubbles == 1 {
                w / 2
            } else {
                (i * w) / n_bubbles + w / (n_bubbles * 2).max(1)
            };
            // Independent speed and phase per bubble via prime-ish offsets.
            let speed = 0.5 + (i as f32 * 0.17) % 0.8;
            let phase_offset = i as f32 * 1.3;
            // y travels 0 (top) to h (bottom) and wraps.
            let travel = (ctx.time * speed + phase_offset) % 1.0;
            let y = h.saturating_sub(1) - (travel * h as f32) as usize;
            // Lateral sine wobble.
            let wobble = ((ctx.time * 1.8 + i as f32 * 0.9).sin() * 2.0).round() as i32;
            let bx = base_x as i32 + wobble;
            // Draw a tiny 2×2 bubble circle (four corners).
            draw::dot_i(grid, bx, y as i32);
            draw::dot_i(grid, bx + 1, y as i32);
            draw::dot_i(grid, bx, y as i32 + 1);
            draw::dot_i(grid, bx + 1, y as i32 + 1);
        }

        // Tint the whole grid with a soft deep-ocean gradient.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            let color = ctx.palette.sample(1.0 - t * 0.6);
            draw::tint_row(grid, cy, 0, ctx.width.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Fish Swimming
// ---------------------------------------------------------------------------
// A fish advances with eased progress; its tail flicks with time.

struct FishSwim;
impl ProgressStyle for FishSwim {
    fn name(&self) -> &str {
        "fish-swim"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A fish swims across with a flickering tail driven by time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = (h / 2) as i32;
        // Fish nose position.
        let head_x = (ctx.eased * w as f32).round() as i32;
        // Body length proportional to grid width, capped.
        let body_len = ((w as f32 * 0.25) as i32).max(4).min(w as i32 / 2);

        // Body: an ellipse-ish blob of dots (tall center, narrow ends).
        for bx in 0..body_len {
            let frac = bx as f32 / body_len.max(1) as f32;
            // Elliptical half-height: peaks at 0.5, tapers to 0 at ends.
            let half_h = ((frac * PI).sin() * (h as f32 * 0.35)).round() as i32;
            let dx = head_x - bx;
            draw::vline(
                grid,
                dx.max(0) as usize,
                (mid - half_h).max(0) as usize,
                (mid + half_h).min(h as i32 - 1) as usize,
            );
        }

        // Eye: single dot near the nose.
        draw::dot_i(grid, head_x - 1, mid - 1);

        // Tail: two angled lines that flick with time.
        let flick = (ctx.time * 6.0).sin() * (h as f32 * 0.25);
        let tail_x = head_x - body_len;
        let tail_tip_up = (mid - flick.round() as i32).clamp(0, h as i32 - 1);
        let tail_tip_dn = (mid + flick.round() as i32).clamp(0, h as i32 - 1);
        draw::vline(
            grid,
            tail_x.max(0) as usize,
            tail_tip_up as usize,
            mid as usize,
        );
        draw::vline(
            grid,
            tail_x.max(0) as usize,
            mid as usize,
            tail_tip_dn as usize,
        );

        // Wake bubbles trailing behind.
        if tail_x > 2 {
            for i in 0..3usize {
                let wt = (ctx.time * 3.0 + i as f32 * 1.1).fract();
                let wx = tail_x - 2 - (wt * 4.0) as i32;
                let wy = mid + ((ctx.time * 4.0 + i as f32).sin() * 1.5).round() as i32;
                draw::dot_i(grid, wx, wy);
            }
        }

        // Tint with palette across cell rows.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                cy,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Wave Crest
// ---------------------------------------------------------------------------
// A tall sine wave rides across the grid; its amplitude and position track
// eased progress, with additional time-driven ripple.

struct WaveCrest;
impl ProgressStyle for WaveCrest {
    fn name(&self) -> &str {
        "wave-crest"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A rolling sine-wave crest advances with progress; fill trails behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = h as f32 / 2.0;
        let amp = h as f32 * 0.4;
        // The crest center (in dot-x) tracks eased progress.
        let crest_center = (ctx.eased * w as f32) as i32;

        for x in 0..w {
            // Phase: combination of crest position and time scroll.
            let phase = (x as f32 - crest_center as f32) * 0.28 - ctx.time * 2.8;
            let envelope = {
                // Gaussian-ish envelope peaking at the crest, fading in front.
                let dist = (x as f32 - crest_center as f32) / w as f32 * 4.0;
                (-dist * dist).exp()
            };
            let wave_y = mid + phase.sin() * amp * envelope;
            let iy = wave_y.round() as i32;

            // Draw a vertical smear around the wave height for thickness.
            for dy in -1i32..=1 {
                draw::dot_i(grid, x as i32, iy + dy);
            }

            // Fill water below the wave for x < crest.
            if x < crest_center.max(0) as usize {
                let floor = (wave_y.round() as usize).min(h - 1);
                draw::vline(grid, x, floor, h - 1);
            }
        }

        // Palette tint: columns behind crest get deeper hue.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let filled_cx = (ctx.eased * ctx.width as f32) as usize;
            if filled_cx > 0 {
                let t = cy as f32 / cells_h.max(1) as f32;
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    filled_cx.saturating_sub(1),
                    ctx.palette.sample(t),
                );
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Sonar Ping
// ---------------------------------------------------------------------------
// Concentric arcs expand from the left edge; arc count tracks eased progress.

struct SonarPing;
impl ProgressStyle for SonarPing {
    fn name(&self) -> &str {
        "sonar-ping"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Expanding sonar arcs; ring count scales with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = 0i32;
        let cy = (h / 2) as i32;
        // Number of active rings — progress controls how many appear.
        let max_rings = 6usize;
        let active_rings = ((ctx.eased * max_rings as f32).round() as usize).max(1);

        for ring_idx in 0..active_rings {
            // Each ring has an independent oscillating radius driven by time.
            let speed = 1.5 + ring_idx as f32 * 0.4;
            let phase = ctx.time * speed + ring_idx as f32 * 0.9;
            let radius = ((phase % (2.0 * PI)) / (2.0 * PI) * w as f32).round() as i32;

            // Draw a quarter-circle arc (right-facing semicircle from the origin).
            let steps = (radius * 3).max(8) as usize;
            for s in 0..steps {
                let angle = (s as f32 / steps as f32) * PI - PI / 2.0;
                let ax = cx + (angle.cos() * radius as f32).round() as i32;
                let ay = cy + (angle.sin() * radius as f32).round() as i32;
                draw::dot_i(grid, ax, ay);
            }
        }

        // Origin blip.
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx + 1, cy);

        // Palette tint across rows.
        let (_, cells_h) = grid.dimensions();
        for cy_cell in 0..cells_h {
            let t = cy_cell as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t * 0.7),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Depth Gauge
// ---------------------------------------------------------------------------
// A vertical depth gauge on the left fills downward with eased progress;
// tick marks label depth levels.

struct DepthGauge;
impl ProgressStyle for DepthGauge {
    fn name(&self) -> &str {
        "depth-gauge"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Submarine depth gauge: fills downward with depth markers and tint"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Left vertical spine.
        draw::vline(grid, 1, 0, h - 1);

        // Fill downward: submarine goes deeper with progress.
        let filled_h = (ctx.eased * h as f32).round() as usize;
        draw::fill_rect(grid, 2, 0, (w / 3).max(1), filled_h);

        // Tick marks every ~20% down the spine.
        for i in 0..=5usize {
            let tick_y = (i as f32 / 5.0 * (h - 1) as f32).round() as usize;
            let tick_len = if i % 5 == 0 { 5usize } else { 3 };
            draw::hline(grid, 0, tick_len.min(w - 1), tick_y);
        }

        // Animated pressure bubbles racing up the right side.
        for i in 0..4usize {
            let bspeed = 0.6 + i as f32 * 0.15;
            let bt = (ctx.time * bspeed + i as f32 * 0.7) % 1.0;
            let by = h.saturating_sub(1) - (bt * h as f32) as usize;
            let bx = (w as i32 - 3) + (((ctx.time * 2.0 + i as f32).sin()) * 1.5) as i32;
            draw::dot_i(grid, bx, by as i32);
            draw::dot_i(grid, bx + 1, by as i32);
        }

        // Depth-gradient tint: lighter near surface, darkening with depth.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, ctx.width.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Jellyfish Pulse
// ---------------------------------------------------------------------------
// A jellyfish bell expands and contracts via time; it drifts upward with
// eased progress. Trailing tentacles sway with sine.

struct JellyfishPulse;
impl ProgressStyle for JellyfishPulse {
    fn name(&self) -> &str {
        "jellyfish-pulse"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A pulsing jellyfish bell drifts upward; tentacles sway in the current"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        // Bell center y: starts near bottom, rises with progress.
        let bell_y = (h as f32 * (1.0 - ctx.eased * 0.85)).round() as i32;
        // Bell radius pulses with time.
        let base_r = (w.min(h) as f32 * 0.22).max(3.0);
        let pulse = 1.0 + 0.25 * (ctx.time * 3.5).sin();
        let r = (base_r * pulse).round() as i32;
        let r_half = (r as f32 * 0.55).round() as i32;

        // Draw the dome (upper semicircle arc).
        let steps = (r * 6).max(12) as usize;
        for s in 0..=steps {
            let angle = s as f32 / steps as f32 * PI; // 0..=π (top half)
            let ax = cx + (angle.cos() * r as f32).round() as i32;
            let ay = bell_y - (angle.sin() * r_half as f32).round() as i32;
            draw::dot_i(grid, ax, ay);
        }
        // Flat base of bell.
        draw::hline(
            grid,
            (cx - r).max(0) as usize,
            (cx + r).min(w as i32 - 1) as usize,
            bell_y as usize,
        );

        // Interior pulsing fill (inner arc a bit smaller).
        let r2 = (r as f32 * 0.6).round() as i32;
        for s in 0..=steps {
            let angle = s as f32 / steps as f32 * PI;
            let ax = cx + (angle.cos() * r2 as f32).round() as i32;
            let ay = bell_y - (angle.sin() * (r_half as f32 * 0.6)).round() as i32;
            draw::dot_i(grid, ax, ay);
        }

        // Tentacles hanging below the bell base.
        let n_tent = 5usize;
        for t_idx in 0..n_tent {
            let tx_base = cx - r + (t_idx as i32 * (r * 2) / n_tent.max(1) as i32);
            let sway_amp = 2.0f32;
            let sway_freq = 1.8 + t_idx as f32 * 0.3;
            let tent_len = (h as f32 * 0.35).round() as i32;
            for seg in 0..tent_len {
                let sway =
                    (ctx.time * sway_freq + seg as f32 * 0.3 + t_idx as f32).sin() * sway_amp;
                let tx = tx_base + sway.round() as i32;
                let ty = bell_y + 1 + seg;
                draw::dot_i(grid, tx, ty);
            }
        }

        // Soft bioluminescence tint — shifting through the palette with time.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = ((cy as f32 / cells_h.max(1) as f32) + ctx.time * 0.05) % 1.0;
            draw::tint_row(
                grid,
                cy,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Coral Reef
// ---------------------------------------------------------------------------
// Branching coral grows upward from the seafloor; branch count ∝ progress.
// Fronds sway gently with time.

struct CoralReef;
impl ProgressStyle for CoralReef {
    fn name(&self) -> &str {
        "coral-reef"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Coral branches grow from the seafloor; sway and growth track progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Seafloor baseline.
        draw::hline(grid, 0, w - 1, h - 1);

        // Number of coral stalks from 1 up to ~10 based on progress.
        let n_stalks = ((ctx.eased * 10.0).round() as usize).max(1);

        for s in 0..n_stalks {
            let sx = (s * w) / n_stalks + w / (n_stalks * 2).max(1);
            // Each stalk height proportional to progress and staggered.
            let height_frac = ctx.eased * (0.5 + (s as f32 * 0.23) % 0.5);
            let stalk_h = (height_frac * h as f32 * 0.85).round() as usize;
            let stalk_base = h - 1;
            let stalk_top = stalk_base.saturating_sub(stalk_h);

            // Sway: gentle lateral sine on the whole stalk.
            let sway_phase = ctx.time * 1.2 + s as f32 * 0.8;

            for seg in 0..stalk_h {
                let seg_y = stalk_base - seg;
                let sway =
                    ((sway_phase + seg as f32 * 0.15).sin() * seg as f32 * 0.06).round() as i32;
                draw::dot_i(grid, sx as i32 + sway, seg_y as i32);
            }

            // Branch fronds at 1/3 and 2/3 of stalk height.
            for &branch_frac in &[0.33f32, 0.66] {
                let bseg = (stalk_h as f32 * branch_frac).round() as usize;
                if bseg == 0 {
                    continue;
                }
                let by = stalk_base.saturating_sub(bseg);
                let bsway =
                    ((sway_phase + bseg as f32 * 0.15).sin() * bseg as f32 * 0.06).round() as i32;
                let bx = sx as i32 + bsway;
                let branch_len = (stalk_h as f32 * 0.25).round() as i32;
                // Left and right branches.
                for bl in 1..=branch_len {
                    draw::dot_i(grid, bx - bl, by as i32 - bl / 2);
                    draw::dot_i(grid, bx + bl, by as i32 - bl / 2);
                }
            }

            // Tip dot.
            let sway_tip =
                ((sway_phase + stalk_h as f32 * 0.15).sin() * stalk_h as f32 * 0.06).round() as i32;
            draw::dot_i(grid, sx as i32 + sway_tip, stalk_top as i32);
        }

        // Warm reef tint.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = 1.0 - cy as f32 / cells_h.max(1) as f32; // deeper = more saturated
            draw::tint_row(
                grid,
                cy,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t * 0.8),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Seaweed
// ---------------------------------------------------------------------------
// Vertical fronds grow from the seafloor, swaying with time; filled width
// tracks eased progress.

struct Seaweed;
impl ProgressStyle for Seaweed {
    fn name(&self) -> &str {
        "seaweed"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Seaweed fronds grow and sway; column fill tracks eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of fronds filling from left.
        let filled_w = (ctx.eased * w as f32).round() as usize;
        let n_fronds = (filled_w / 2).max(if ctx.progress > 0.0 { 1 } else { 0 });

        for fi in 0..n_fronds {
            let fx = (fi * filled_w) / n_fronds.max(1);
            let sway_freq = 1.0 + (fi as f32 * 0.31) % 0.8;
            let height_frac = 0.5 + (fi as f32 * 0.19) % 0.5;
            let frond_h = (height_frac * h as f32).round() as usize;

            for seg in 0..frond_h {
                let seg_y = h - 1 - seg;
                // Sway increases toward tip.
                let sway_amp = seg as f32 / frond_h.max(1) as f32 * 3.0;
                let sway = ((ctx.time * sway_freq + fi as f32 * 0.7 + seg as f32 * 0.2).sin()
                    * sway_amp)
                    .round() as i32;
                draw::dot_i(grid, fx as i32 + sway, seg_y as i32);

                // Alternating side leaflets every 4 segments.
                if seg % 4 == 2 {
                    let leaf_side: i32 = if seg % 8 < 4 { 1 } else { -1 };
                    draw::dot_i(grid, fx as i32 + sway + leaf_side, seg_y as i32 - 1);
                }
            }
        }

        // Seafloor.
        draw::hline(grid, 0, filled_w.min(w - 1), h - 1);

        // Tint only the filled portion.
        let (_, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * ctx.width as f32).round() as usize;
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            if filled_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    filled_cells.saturating_sub(1),
                    ctx.palette.sample(0.3 + t * 0.5),
                );
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Ripple Interference
// ---------------------------------------------------------------------------
// Two sinusoidal wave sources produce an interference pattern; source
// separation tracks eased progress.

struct RippleInterference;
impl ProgressStyle for RippleInterference {
    fn name(&self) -> &str {
        "ripple-interference"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Two point sources create animated ripple interference patterns"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Source positions: start centered, spread apart with progress.
        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let spread = ctx.eased * (w as f32 * 0.35);
        let s1x = cx - spread;
        let s2x = cx + spread;

        for py in 0..h {
            for px in 0..w {
                let pxf = px as f32;
                let pyf = py as f32;
                // Distance from each source.
                let d1 = ((pxf - s1x).powi(2) + (pyf - cy).powi(2)).sqrt();
                let d2 = ((pxf - s2x).powi(2) + (pyf - cy).powi(2)).sqrt();
                // Superposition of two circular waves.
                let wave = (d1 * 0.6 - ctx.time * 4.0).sin() + (d2 * 0.6 - ctx.time * 4.0).sin();
                // Dot when combined amplitude exceeds threshold.
                if wave > 1.2 {
                    draw::dot(grid, px, py);
                }
            }
        }

        // Tint smoothly across both axes.
        let (_, cells_h) = grid.dimensions();
        for cy_cell in 0..cells_h {
            let t = cy_cell as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t),
            );
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
    let styles = progress::styles::ocean::styles();
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
