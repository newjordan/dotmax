//! `plants` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O plants.rs && ./plants [style-name]
//! ```

const DEFAULT_STYLE: &str = "seedling";

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
    pub mod plants {
//! Plants / flora progress bars — living, growing, breathing.
//!
//! Ten structurally distinct styles, each driven by `ctx.eased` for growth
//! stage and `ctx.time` for sway / breeze animation. Every style uses a
//! completely different rendering algorithm: no two share the same structural
//! approach, differing only in palette.
//!
//! | Name              | Mechanism                                              |
//! |-------------------|--------------------------------------------------------|
//! | `seedling`        | Discrete seed→sprout→stem→leaf→flower growth stages   |
//! | `fern-fiddlehead` | Logarithmic spiral unrolling (fiddlehead uncurling)    |
//! | `bamboo-shoot`    | Vertical segments shooting up one by one               |
//! | `ivy-trellis`     | Vine climbing a grid trellis with curling tendrils     |
//! | `cactus-arms`     | Trunk grows then arms branch out at thresholds         |
//! | `sunflower-seeds` | Disc fills with golden-angle phyllotaxis seed dots     |
//! | `mushroom-cap`    | Cap rises and expands as stalk grows upward            |
//! | `succulent`       | Radial rosette: leaves open from centre outward        |
//! | `root-system`     | Fractal root branches growing downward                 |
//! | `bonsai`          | Trunk → recursive branches → dot canopy               |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── registry ─────────────────────────────────────────────────────────────────

/// All styles in the `plants` theme.
///
/// Returns ten structurally distinct plant-growth progress bars, each
/// mapping `ctx.eased` to visible growth and `ctx.time` to living motion.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Seedling),
        Box::new(FernFiddlehead),
        Box::new(BambooShoot),
        Box::new(IvyTrellis),
        Box::new(CactusArms),
        Box::new(SunflowerSeeds),
        Box::new(MushroomCap),
        Box::new(Succulent),
        Box::new(RootSystem),
        Box::new(Bonsai),
    ]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Integer sine — keeps per-bar code terse.
#[inline]
fn isin(angle: f32, amplitude: f32) -> i32 {
    (angle.sin() * amplitude).round() as i32
}

// ── 1. Seedling — discrete growth stages ─────────────────────────────────────
//
// eased maps to five visible stages:
//   0.00–0.20  seed dot (underground bulge)
//   0.20–0.40  sprout curl emerging from soil
//   0.40–0.60  single stem rising
//   0.60–0.80  two side leaves appear
//   0.80–1.00  flower petals open at the tip

struct Seedling;
impl ProgressStyle for Seedling {
    fn name(&self) -> &str {
        "seedling"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Five discrete growth stages: seed → sprout → stem → leaves → flower, driven by progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let stage = ctx.eased;
        let cx = (w / 2) as i32;
        let ground = h.saturating_sub(2) as i32;
        // Sway all above-ground parts with a gentle breeze.
        let sway = isin(ctx.time * 1.4, (w as f32 * 0.04).max(1.0));

        // Stage 1: seed — a small oval underground.
        if stage >= 0.0 {
            // Seed sits at ground level.
            draw::dot_i(grid, cx, ground);
            draw::dot_i(grid, cx + 1, ground);
            draw::dot_i(grid, cx, ground + 1);
            draw::dot_i(grid, cx + 1, ground + 1);
        }

        // Stage 2: sprout hook emerging above ground (eased 0.20+).
        if stage >= 0.20 {
            // Hook: a short upward line that curves over.
            let sprout_h = ((stage - 0.20) / 0.20 * h as f32 * 0.25).round() as i32;
            for dy in 0..=sprout_h {
                draw::dot_i(grid, cx + sway / 2, ground - dy);
            }
            // Curved tip — the cotyledon loop.
            draw::dot_i(grid, cx + sway / 2 + 1, ground - sprout_h);
            draw::dot_i(grid, cx + sway / 2 + 1, ground - sprout_h + 1);
        }

        // Stage 3: full stem (eased 0.40+).
        if stage >= 0.40 {
            let stem_h = ((stage - 0.40) / 0.20 * h as f32 * 0.50).round() as i32;
            let stem_h = stem_h.min(ground);
            for dy in 0..=stem_h {
                let lean = isin(ctx.time * 1.4 + dy as f32 * 0.2, (w as f32 * 0.04).max(1.0));
                draw::dot_i(grid, cx + lean, ground - dy);
            }
        }

        // Stage 4: two leaves at mid-stem (eased 0.60+).
        if stage >= 0.60 {
            let stem_h = (h as f32 * 0.50).round() as i32;
            let leaf_y = ground - stem_h / 2;
            let leaf_reach = ((stage - 0.60) / 0.20 * w as f32 * 0.20).round() as i32;
            for lx in 1..=leaf_reach {
                // Left leaf curves up-left, right leaf up-right.
                let curve = (lx as f32 / leaf_reach.max(1) as f32 * PI * 0.5).sin();
                let dy = -(curve * 2.0).round() as i32;
                draw::dot_i(grid, cx + sway - lx, leaf_y + dy);
                draw::dot_i(grid, cx + sway + lx, leaf_y + dy);
            }
        }

        // Stage 5: flower petals (eased 0.80+).
        if stage >= 0.80 {
            let stem_h = (h as f32 * 0.50).round() as i32;
            let tip_y = ground - stem_h;
            let petals = 6usize;
            let petal_len = ((stage - 0.80) / 0.20 * (w as f32 * 0.12).max(2.0)).max(0.0);
            for p in 0..petals {
                let angle = (p as f32 / petals as f32) * 2.0 * PI + ctx.time * 0.4;
                let steps = petal_len.round() as usize;
                for s in 1..=steps {
                    let r = s as f32;
                    draw::dot_i(
                        grid,
                        cx + sway + (angle.cos() * r).round() as i32,
                        tip_y + (angle.sin() * r).round() as i32,
                    );
                }
            }
            // Centre of flower.
            draw::dot_i(grid, cx + sway, tip_y);
        }

        // Tint: soil brown at bottom, green in the middle, gold at tip.
        let (cw, ch) = grid.dimensions();
        use crate::Color;
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.saturating_sub(1).max(1) as f32; // 0 = top, 1 = bottom
            let color = if t > 0.85 {
                Color::rgb(101, 67, 33) // soil
            } else {
                ctx.palette.sample(1.0 - t)
            };
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 2. Fern Fiddlehead — logarithmic spiral unrolling ────────────────────────
//
// A fiddlehead starts as a tight coil (eased=0) and progressively unrolls
// into an open frond (eased=1). The spiral uses polar coordinates.
// ctx.time adds a gentle oscillation so the frond breathes.

struct FernFiddlehead;
impl ProgressStyle for FernFiddlehead {
    fn name(&self) -> &str {
        "fern-fiddlehead"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A fern fiddlehead unrolls from a tight coil into an open frond as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;

        // Logarithmic spiral: r = a * e^(b * theta).
        // At eased=0 draw only the tight coil; at eased=1 draw full frond.
        let a = 1.0f32;
        let b = 0.22f32;

        // Total angular span of the frond: 0 (coil only) → 4*PI (full unroll).
        let max_turns = 4.0 * PI;
        let unroll = ctx.eased * max_turns;

        // Coil starts wound tightly (small theta) and unwinds.
        // We draw from theta = 0 (innermost) outward.
        let max_r = (w.min(h) / 2).saturating_sub(1) as f32;
        let scale = max_r / (a * (b * max_turns).exp());

        // Breathe: the coil rotates slightly with time.
        let breath_offset = (ctx.time * 0.5).sin() * 0.15;

        let steps = 200usize;
        for s in 0..=steps {
            let theta = s as f32 / steps as f32 * unroll;
            if theta > unroll {
                break;
            }
            let r = a * (b * theta).exp() * scale;
            // Rotate so fiddlehead starts pointing down (PI/2) and uncurls upward.
            let angle = theta + PI / 2.0 + breath_offset;
            let px = cx + (r * angle.cos()).round() as i32;
            let py = cy + (r * angle.sin()).round() as i32;
            draw::dot_i(grid, px, py);

            // Pinnae (side leaflets) every half-turn on the outer half of the spiral.
            if theta > max_turns / 2.0 && s % 20 == 0 {
                let pinnae_len = (r * 0.4).max(1.0);
                let pinnae_angle = angle + PI / 2.0;
                for pd in 1..=(pinnae_len.round() as i32) {
                    draw::dot_i(
                        grid,
                        px + (pinnae_angle.cos() * pd as f32).round() as i32,
                        py + (pinnae_angle.sin() * pd as f32).round() as i32,
                    );
                    draw::dot_i(
                        grid,
                        px - (pinnae_angle.cos() * pd as f32).round() as i32,
                        py - (pinnae_angle.sin() * pd as f32).round() as i32,
                    );
                }
            }
        }

        // Tint: deep green shifting to pale frond tip.
        let (cw, ch) = grid.dimensions();
        for row in 0..ch {
            let t = row as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased);
            draw::tint_row(grid, row, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 3. Bamboo Shoot — vertical segments shooting up ──────────────────────────
//
// Each segment is a cell-glyph column block. Segments appear one by one from
// the bottom as eased advances. Each joint has a horizontal line (node).
// ctx.time causes segments to sway slightly left-right as a column.

struct BambooShoot;
impl ProgressStyle for BambooShoot {
    fn name(&self) -> &str {
        "bamboo-shoot"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Bamboo culm grows segment by segment upward; joints appear at each node as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let (cw, ch) = grid.dimensions();
        // Bamboo column sits in the centre third.
        let stem_cx = cw / 2;
        let seg_height_cells = 2usize.max(1); // each segment is 2 cell rows tall
        let max_segs = (ch / seg_height_cells).max(1);
        let segs_grown = (ctx.eased * max_segs as f32).round() as usize;

        // Sway: the culm leans left/right.
        let sway_cells = (isin(ctx.time * 1.0, (cw as f32 * 0.06).max(1.0))) as i32;

        for seg in 0..segs_grown.min(max_segs) {
            // Segments grow from bottom up.
            let base_cell_y = ch.saturating_sub(1 + seg * seg_height_cells);
            let sx = (stem_cx as i32 + sway_cells).clamp(0, cw.saturating_sub(1) as i32) as usize;

            // Draw segment body using vblock glyphs (full column in both cell rows).
            for dy in 0..seg_height_cells {
                let cy = base_cell_y.saturating_sub(dy);
                draw::vblock(grid, sx.min(cw.saturating_sub(1)), cy, 8);
                // Thin border dots on both sides (in dot space).
                let dx = sx * 2;
                let dy_dot = cy * 4;
                draw::vline(grid, dx.saturating_sub(1), dy_dot, dy_dot + 3);
                draw::vline(grid, (dx + 2).min(w.saturating_sub(1)), dy_dot, dy_dot + 3);
            }

            // Node (joint) line at the base of each segment.
            let node_y = base_cell_y;
            let node_dot_y = node_y * 4 + 3;
            let left = sx.saturating_sub(1) * 2;
            let right = (sx + 2).min(w / 2) * 2;
            draw::hline(grid, left, right, node_dot_y.min(h.saturating_sub(1)));

            // Leaf pair at every other node (alternating sides).
            if seg % 2 == 0 && segs_grown > 1 {
                let leaf_y = node_y as i32;
                let leaf_x_base = sx as i32 * 2;
                let side = if seg % 4 == 0 { 1i32 } else { -1i32 };
                // Three-dot leaf sweeping out.
                for ld in 1..=3i32 {
                    draw::dot_i(grid, leaf_x_base + side * ld, (leaf_y * 4) as i32 - ld);
                }
            }
        }

        // Gradient tint: deep green at base, yellow-green at tip.
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 4. Ivy Trellis — vine climbing a grid trellis with tendrils ──────────────
//
// The trellis is a fixed dot-grid. The vine advances along the trellis
// left→right (progress). At intervals, curling tendrils spiral off the vine.
// ctx.time animates the tendril curl continuously.

struct IvyTrellis;
impl ProgressStyle for IvyTrellis {
    fn name(&self) -> &str {
        "ivy-trellis"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Ivy climbs a dot-grid trellis left-to-right; curling tendrils spiral off the vine"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Draw trellis: vertical posts every 8 dots, two horizontal rails.
        let post_spacing = 8usize.max(1);
        let rail1 = h / 4;
        let rail2 = 3 * h / 4;
        // Top rail.
        draw::hline(grid, 0, w.saturating_sub(1), rail1);
        // Bottom rail.
        draw::hline(grid, 0, w.saturating_sub(1), rail2);
        // Vertical posts.
        let posts = w / post_spacing;
        for p in 0..=posts {
            let px = p * post_spacing;
            draw::vline(grid, px.min(w.saturating_sub(1)), rail1, rail2);
        }

        // Vine: travels along the bottom rail, then climbs up each post, then along top rail.
        // The vine path is parameterized in [0,1] → (x,y) dot coordinates.
        // Phase 0–0.5: bottom rail left→right
        // Phase 0.5–1.0: top rail left→right (climbs posts along the way)
        let reach = ctx.eased;
        let vine_end_dot = (reach * w as f32) as usize;

        // Bottom rail vine.
        let bottom_reach = (reach * 2.0).min(1.0);
        let brd = (bottom_reach * w as f32) as usize;
        for x in 0..brd.min(w) {
            let sway = isin(x as f32 * 0.5 + ctx.time * 1.2, 1.0);
            draw::dot_i(grid, x as i32, rail2 as i32 + sway);
        }

        // Top rail vine only appears in second half of progress.
        if reach > 0.5 {
            let top_reach = (reach - 0.5) * 2.0;
            let trd = (top_reach * w as f32) as usize;
            for x in 0..trd.min(w) {
                let sway = isin(x as f32 * 0.5 + ctx.time * 1.2, 1.0);
                draw::dot_i(grid, x as i32, rail1 as i32 + sway);
            }
        }

        // Tendrils: small clockwise spirals hanging off the bottom vine.
        let tendril_spacing = 12usize.max(1);
        let tendril_count = vine_end_dot / tendril_spacing;
        for t_idx in 0..tendril_count {
            let tx = (t_idx * tendril_spacing + tendril_spacing / 2).min(w.saturating_sub(1));
            // Tendril curls below the rail.
            let curl_turns = 1.5f32;
            let tendril_r = (h as f32 * 0.08).max(2.0);
            let phase_offset = ctx.time * 1.5 + t_idx as f32 * 0.8;
            let curl_steps = 24usize;
            for s in 0..=curl_steps {
                let theta = s as f32 / curl_steps as f32 * curl_turns * 2.0 * PI + phase_offset;
                let r = tendril_r * (1.0 - s as f32 / curl_steps.max(1) as f32 * 0.7);
                let px = tx as i32 + (theta.cos() * r).round() as i32;
                let py = rail2 as i32 + 2 + (theta.sin() * r).round() as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Tint the filled region.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if cw <= 1 {
                0.0
            } else {
                cx as f32 / (cw - 1) as f32
            };
            let color = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ── 5. Cactus Arms — trunk grows then lateral arms branch out ────────────────
//
// The trunk (a solid vline) grows from the bottom up as eased → 0.5.
// At eased = 0.5, left arm appears; at eased = 0.75, right arm.
// At eased = 1.0, arms grow spines (small perpendicular dots).
// ctx.time gives a slow pulse (very slight trunk width change).

struct CactusArms;
impl ProgressStyle for CactusArms {
    fn name(&self) -> &str {
        "cactus-arms"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A cactus trunk rises then sprouts a left arm, right arm, and finally spines at full growth"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let base_y = h.saturating_sub(1) as i32;

        // Trunk grows from base upward — first 50% of eased.
        let trunk_frac = (ctx.eased / 0.5).min(1.0);
        let trunk_h = (trunk_frac * h as f32 * 0.80).round() as i32;
        let trunk_top = base_y - trunk_h;

        // Draw trunk as 2-wide column.
        for dy in 0..=trunk_h {
            let y = base_y - dy;
            draw::dot_i(grid, cx, y);
            draw::dot_i(grid, cx + 1, y);
        }

        // Slow pulse width (purely cosmetic, time-driven).
        let pulse = ((ctx.time * 0.8).sin() * 0.5 + 0.5) > 0.5;
        if pulse && trunk_h > 4 {
            for dy in 2..=trunk_h - 2 {
                draw::dot_i(grid, cx - 1, base_y - dy);
            }
        }

        // Left arm appears at eased 0.50–0.75.
        if ctx.eased >= 0.50 {
            let arm_frac = ((ctx.eased - 0.50) / 0.25).min(1.0);
            let arm_attach_y = trunk_top + trunk_h / 3;
            let arm_w = (arm_frac * w as f32 * 0.25).round() as i32;
            // Horizontal run out, then up.
            for dx in 0..=arm_w {
                draw::dot_i(grid, cx - dx, arm_attach_y);
            }
            let arm_up = (arm_frac * h as f32 * 0.15).round() as i32;
            for dy in 0..=arm_up {
                draw::dot_i(grid, cx - arm_w, arm_attach_y - dy);
            }
        }

        // Right arm appears at eased 0.75–1.00.
        if ctx.eased >= 0.75 {
            let arm_frac = ((ctx.eased - 0.75) / 0.25).min(1.0);
            let arm_attach_y = trunk_top + trunk_h / 5;
            let arm_w = (arm_frac * w as f32 * 0.22).round() as i32;
            for dx in 0..=arm_w {
                draw::dot_i(grid, cx + 1 + dx, arm_attach_y);
            }
            let arm_up = (arm_frac * h as f32 * 0.12).round() as i32;
            for dy in 0..=arm_up {
                draw::dot_i(grid, cx + 1 + arm_w, arm_attach_y - dy);
            }
        }

        // Spines: small perpendicular dots along trunk at full growth.
        if ctx.eased >= 0.95 {
            let spine_spacing = 4i32;
            let mut y = base_y - spine_spacing;
            while y > trunk_top {
                // Alternating left/right spines.
                let side = if (y / spine_spacing) % 2 == 0 {
                    3i32
                } else {
                    -3i32
                };
                draw::dot_i(grid, cx + side, y);
                draw::dot_i(grid, cx + side / 2, y - 1);
                y -= spine_spacing;
            }
        }

        // Cactus green tint.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(1.0 - t * 0.5);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 6. Sunflower Seeds — golden-angle phyllotaxis ────────────────────────────
//
// The disc is filled with seeds placed at golden-angle increments.
// eased controls how many seeds are visible (from centre outward).
// ctx.time causes the whole disc to rotate slowly.

struct SunflowerSeeds;
impl ProgressStyle for SunflowerSeeds {
    fn name(&self) -> &str {
        "sunflower-seeds"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Sunflower disc fills with golden-angle phyllotaxis seeds radiating from the centre"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = (w.min(h) / 2).saturating_sub(1) as f32;

        // Total seeds scales with area; use a fixed cap for fast rendering.
        let max_seeds = 200usize;
        let visible = (ctx.eased * max_seeds as f32).round() as usize;

        // Golden angle in radians.
        let golden_angle = PI * (3.0 - 5.0_f32.sqrt());
        let rotation_offset = ctx.time * 0.2;

        for n in 0..visible.min(max_seeds) {
            let r = max_r * (n as f32 / max_seeds as f32).sqrt();
            let theta = n as f32 * golden_angle + rotation_offset;
            let sx = cx + (r * theta.cos()).round() as i32;
            let sy = cy + (r * theta.sin()).round() as i32;
            draw::dot_i(grid, sx, sy);
            // Slightly thicker seeds in the outer half.
            if r > max_r * 0.5 {
                draw::dot_i(grid, sx + 1, sy);
            }
        }

        // Outer ring (flower receptacle edge).
        if ctx.eased > 0.1 {
            let rim_r = (max_r * ctx.eased.sqrt()).round() as i32;
            let rim_r = rim_r.max(1);
            let steps = (2.0 * PI * rim_r as f32).round() as usize + 4;
            for s in 0..steps {
                let angle = s as f32 / steps as f32 * 2.0 * PI;
                draw::dot_i(
                    grid,
                    cx + (rim_r as f32 * angle.cos()).round() as i32,
                    cy + (rim_r as f32 * angle.sin()).round() as i32,
                );
            }
        }

        // Warm yellow-brown centre tint.
        let (cw, ch) = grid.dimensions();
        for row in 0..ch {
            let t = row as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased);
            draw::tint_row(grid, row, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 7. Mushroom Cap — stalk rises, cap expands ───────────────────────────────
//
// eased 0–0.5: stalk grows upward from the ground (vline, 2-wide).
// eased 0.5–1.0: dome cap arc widens from a dot to a full hemisphere.
// ctx.time causes subtle cap wobble and spore dots fall below the cap.

struct MushroomCap;
impl ProgressStyle for MushroomCap {
    fn name(&self) -> &str {
        "mushroom-cap"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A mushroom stalk rises then its dome cap expands; spore dots drift downward"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let ground = h.saturating_sub(1) as i32;

        // Stalk grows from ground up during first 50%.
        let stalk_frac = (ctx.eased / 0.5).min(1.0);
        let stalk_h = (stalk_frac * h as f32 * 0.55).round() as i32;
        for dy in 0..=stalk_h {
            let y = ground - dy;
            draw::dot_i(grid, cx, y);
            draw::dot_i(grid, cx + 1, y);
        }

        // Stalk top position.
        let stalk_top = ground - stalk_h;

        // Cap: dome arc (upper semi-ellipse) centred above the stalk.
        if ctx.eased >= 0.5 {
            let cap_frac = ((ctx.eased - 0.5) / 0.5).min(1.0);
            // Wobble with time.
            let wobble = (ctx.time * 2.3).sin() * cap_frac * 1.0;
            let cap_rx = ((cap_frac * w as f32 * 0.40) + wobble).max(1.0) as i32;
            let cap_ry = (cap_frac * h as f32 * 0.35).max(1.0) as i32;
            let cap_cy = stalk_top;

            // Draw filled upper semi-ellipse.
            for dy in 0..=cap_ry {
                // Horizontal half-width at this row (ellipse formula).
                let row_w = if cap_ry == 0 {
                    0
                } else {
                    (cap_rx as f32 * (1.0 - (dy as f32 / cap_ry as f32).powi(2)).sqrt()).round()
                        as i32
                };
                let y = cap_cy - dy;
                for dx in -row_w..=row_w {
                    draw::dot_i(grid, cx + dx, y);
                }
            }

            // Spots on the cap (Amanita-style) — fixed polar positions.
            let spots = [(0.3f32, 0.4f32), (-0.35, 0.55), (0.55, 0.65), (-0.1, 0.75)];
            for &(sx_frac, sy_frac) in &spots {
                let sx = cx + (sx_frac * cap_rx as f32).round() as i32;
                let sy = cap_cy - (sy_frac * cap_ry as f32).round() as i32;
                draw::dot_i(grid, sx, sy);
                draw::dot_i(grid, sx + 1, sy);
            }

            // Spore fall: dots below the cap edge drift downward.
            let spore_count = 5usize;
            for s in 0..spore_count {
                let phase = s as f32 / spore_count as f32;
                let spore_x = cx + ((phase - 0.5) * cap_rx as f32 * 1.8).round() as i32;
                let drop = ((ctx.time * 0.8 + phase).fract() * (ground - stalk_top) as f32) as i32;
                let spore_y = stalk_top + drop;
                draw::dot_i(grid, spore_x, spore_y);
            }
        }

        // Tint: earthy warm for stalk, red/orange cap above.
        let (cw, ch) = grid.dimensions();
        let stalk_cell = (stalk_top.max(0) as usize / 4).min(ch.saturating_sub(1));
        for cy in 0..ch {
            let t = if cy < stalk_cell {
                ctx.palette.sample(0.9)
            } else {
                ctx.palette
                    .sample(0.3 + cy as f32 / ch.saturating_sub(1).max(1) as f32 * 0.6)
            };
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), t);
        }

        Ok(())
    }
}

// ── 8. Succulent Rosette — radial leaves open from centre ────────────────────
//
// eased controls how many leaves are visible AND their opening angle.
// Leaves are rendered as filled wedge arcs growing outward from the centre.
// ctx.time rotates the whole rosette slowly (succulents track the sun).

struct Succulent;
impl ProgressStyle for Succulent {
    fn name(&self) -> &str {
        "succulent-rosette"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A succulent rosette opens radially; leaves widen and extend as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = (w.min(h) / 2).saturating_sub(1) as f32;

        let total_leaves = 12usize;
        // Leaves appear in order as progress increases.
        let leaves_open = (ctx.eased * total_leaves as f32).ceil() as usize;

        // Rotate slowly with time.
        let rotation = ctx.time * 0.15;

        for leaf in 0..leaves_open.min(total_leaves) {
            let base_angle = (leaf as f32 / total_leaves as f32) * 2.0 * PI + rotation;

            // This leaf's opening fraction (newest leaf partially open, others full).
            let leaf_frac = if leaf + 1 < leaves_open {
                1.0f32
            } else {
                let base = leaf as f32 / total_leaves as f32;
                ((ctx.eased - base) * total_leaves as f32).clamp(0.0, 1.0)
            };

            let leaf_r = (leaf_frac * max_r * 0.85).max(0.0);
            // Half-angular width of each leaf (wedge).
            let half_w = leaf_frac * PI / (total_leaves as f32) * 2.5;

            // Fill the wedge arc with dot samples.
            let arc_steps = (leaf_r * 4.0).round() as usize + 4;
            let r_steps = (leaf_r * 0.5).round() as usize + 2;
            for rs in 1..=r_steps {
                let r = leaf_r * rs as f32 / r_steps as f32;
                for a in 0..=arc_steps {
                    let angle_off = (a as f32 / arc_steps as f32 - 0.5) * 2.0 * half_w;
                    let angle = base_angle + angle_off;
                    let px = cx + (r * angle.cos()).round() as i32;
                    let py = cy + (r * angle.sin()).round() as i32;
                    draw::dot_i(grid, px, py);
                }
            }

            // Midrib line (leaf vein).
            let vein_steps = (leaf_r * 0.9).round() as usize;
            for s in 0..=vein_steps {
                let r = leaf_r * s as f32 / vein_steps.max(1) as f32;
                draw::dot_i(
                    grid,
                    cx + (base_angle.cos() * r).round() as i32,
                    cy + (base_angle.sin() * r).round() as i32,
                );
            }
        }

        // Centre dome.
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx + 1, cy);
        draw::dot_i(grid, cx, cy + 1);

        // Tint concentric rings.
        let (cw, ch) = grid.dimensions();
        for row in 0..ch {
            let t = row as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, row, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 9. Root System — fractal branching downward ───────────────────────────────
//
// Grows roots downward from a horizontal taproot line at the top.
// eased controls how deep / how many branch generations appear.
// ctx.time causes slight lateral tremor (root seeking moisture).

struct RootSystem;
impl ProgressStyle for RootSystem {
    fn name(&self) -> &str {
        "root-system"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Fractal root branches grow downward from a taproot; depth and density increase with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Taproot horizontal line at top.
        let troot_y = 1usize;
        draw::hline(grid, 0, w.saturating_sub(1), troot_y);

        // Number of branch generations shown.
        let max_depth = 5usize;
        let depth = (ctx.eased * max_depth as f32).ceil() as usize;

        // Recursive branch drawer (iterative via stack to avoid recursion limits).
        // Stack: (x, y, angle, length, generation)
        let mut stack: Vec<(i32, i32, f32, f32, usize)> = Vec::new();

        // Seed with 3 primary roots evenly spaced.
        let num_primaries = 3usize;
        for p in 0..num_primaries {
            let px = (p as f32 / (num_primaries - 1).max(1) as f32 * (w.saturating_sub(1)) as f32)
                as i32;
            stack.push((px, troot_y as i32, PI / 2.0, h as f32 * 0.35, 0));
        }

        while let Some((x0, y0, angle, length, gen)) = stack.pop() {
            if gen >= depth {
                continue;
            }
            if length < 1.5 {
                continue;
            }

            let tremor = (ctx.time * 2.0 + gen as f32 * 1.3 + x0 as f32 * 0.1).sin() * 0.08;
            let actual_angle = angle + tremor;

            let x1 = x0 + (actual_angle.cos() * length).round() as i32;
            let y1 = y0 + (actual_angle.sin() * length).round() as i32;

            // Draw this segment.
            let steps = length.round() as usize + 1;
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let bx = x0 + ((x1 - x0) as f32 * t).round() as i32;
                let by = y0 + ((y1 - y0) as f32 * t).round() as i32;
                draw::dot_i(grid, bx, by);
            }

            // Branch into two children, spread by ~30 degrees.
            let child_len = length * 0.62;
            let spread = PI / 6.0;
            stack.push((x1, y1, actual_angle - spread, child_len, gen + 1));
            stack.push((x1, y1, actual_angle + spread, child_len, gen + 1));
        }

        // Earthy tint: dark at top roots, lighter at fine root tips below.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 10. Bonsai — trunk → recursive branches → dot canopy ─────────────────────
//
// eased 0–0.40: trunk rises from bottom, slightly tapered.
// eased 0.40–0.70: two main branches fork from near the top.
// eased 0.70–0.90: secondary branches fork again.
// eased 0.90–1.00: canopy dots fill in around the branch tips.
// ctx.time: gentle breeze sways the canopy.

struct Bonsai;
impl ProgressStyle for Bonsai {
    fn name(&self) -> &str {
        "bonsai"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A bonsai forms: trunk rises, branches fork in tiers, then canopy foliage fills in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let ground = h.saturating_sub(1) as i32;

        // ── Trunk ──────────────────────────────────────────────────────────
        let trunk_frac = (ctx.eased / 0.40).min(1.0);
        let trunk_h = (trunk_frac * h as f32 * 0.55).round() as i32;
        for dy in 0..=trunk_h {
            let y = ground - dy;
            // Taper: 3 wide at base, 1 wide at top.
            let width = if dy < trunk_h / 3 {
                3i32
            } else if dy < 2 * trunk_h / 3 {
                2i32
            } else {
                1i32
            };
            for dx in 0..width {
                draw::dot_i(grid, cx - width / 2 + dx, y);
            }
        }

        let fork_y = ground - trunk_h; // where branches begin

        // ── Primary branches ───────────────────────────────────────────────
        if ctx.eased >= 0.40 {
            let br_frac = ((ctx.eased - 0.40) / 0.30).min(1.0);
            let br_len = (br_frac * w as f32 * 0.28).round() as i32;
            let br_rise = (br_frac * h as f32 * 0.20).round() as i32;
            // Left branch.
            for s in 0..=br_len {
                let t = s as f32 / br_len.max(1) as f32;
                let bx = cx - s;
                let by = fork_y - (t * br_rise as f32).round() as i32;
                draw::dot_i(grid, bx, by);
            }
            // Right branch.
            for s in 0..=br_len {
                let t = s as f32 / br_len.max(1) as f32;
                let bx = cx + s;
                let by = fork_y - (t * br_rise as f32 * 0.8).round() as i32;
                draw::dot_i(grid, bx, by);
            }

            // ── Secondary branches ─────────────────────────────────────────
            if ctx.eased >= 0.70 {
                let sb_frac = ((ctx.eased - 0.70) / 0.20).min(1.0);
                let sb_len = (sb_frac * w as f32 * 0.15).round() as i32;
                let sb_rise = (sb_frac * h as f32 * 0.12).round() as i32;
                // Tips of primary branches.
                let left_tip_x = cx - br_len;
                let left_tip_y = fork_y - br_rise;
                let right_tip_x = cx + br_len;
                let right_tip_y = fork_y - (br_rise as f32 * 0.8).round() as i32;

                for s in 0..=sb_len {
                    let t = s as f32 / sb_len.max(1) as f32;
                    // From left tip: branch further left and up.
                    draw::dot_i(
                        grid,
                        left_tip_x - s,
                        left_tip_y - (t * sb_rise as f32).round() as i32,
                    );
                    // From left tip: branch up-right.
                    draw::dot_i(
                        grid,
                        left_tip_x + s / 2,
                        left_tip_y - (t * sb_rise as f32 * 1.2).round() as i32,
                    );
                    // From right tip: branch right and up.
                    draw::dot_i(
                        grid,
                        right_tip_x + s,
                        right_tip_y - (t * sb_rise as f32).round() as i32,
                    );
                    // From right tip: branch up-left.
                    draw::dot_i(
                        grid,
                        right_tip_x - s / 2,
                        right_tip_y - (t * sb_rise as f32 * 1.1).round() as i32,
                    );
                }

                // ── Canopy foliage dots ────────────────────────────────────
                if ctx.eased >= 0.90 {
                    let canopy_frac = ((ctx.eased - 0.90) / 0.10).min(1.0);
                    // Scatter foliage dots around branch tips using time-driven noise.
                    let dot_count = (canopy_frac * 40.0).round() as usize;
                    let canopy_cx = cx;
                    let canopy_cy = (fork_y - br_rise - sb_rise).max(0);
                    let canopy_rx = (br_len + sb_len).max(1) as f32;
                    let canopy_ry = (br_rise + sb_rise).max(1) as f32;

                    for d in 0..dot_count {
                        // Deterministic scatter via golden angle + time sway.
                        let theta = d as f32 * 2.399 + ctx.time * 0.5; // 2.399 ≈ golden angle
                        let r_frac = (d as f32 / dot_count.max(1) as f32).sqrt();
                        let sway = (ctx.time * 1.3 + d as f32 * 0.3).sin() * 1.5;
                        let dx = (theta.cos() * canopy_rx * r_frac + sway).round() as i32;
                        let dy = (theta.sin() * canopy_ry * r_frac).round() as i32;
                        draw::dot_i(grid, canopy_cx + dx, canopy_cy + dy);
                    }
                }
            }
        }

        // Soil line at very bottom.
        draw::hline(grid, 0, w.saturating_sub(1), ground as usize);

        // Tint: brown trunk base → green foliage top.
        let (cw, ch) = grid.dimensions();
        let trunk_cell_top = (fork_y.max(0) as usize / 4).min(ch.saturating_sub(1));
        for cy in 0..ch {
            let color = if cy >= trunk_cell_top {
                ctx.palette.sample(0.85) // trunk: warm brown-ish
            } else {
                ctx.palette
                    .sample(cy as f32 / trunk_cell_top.max(1) as f32 * 0.7)
            };
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
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
    let styles = progress::styles::plants::styles();
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
