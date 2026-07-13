//! `cars` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O cars.rs && ./cars [style-name]
//! ```

const DEFAULT_STYLE: &str = "drag-tree";

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
    pub mod cars {
//! Cars / Automotive / Racing progress bars — the **cars** theme.
//!
//! Every bar is **structurally** distinct: each uses a different geometric
//! primitive, spatial layout, or temporal mechanic.  Color alone never
//! separates two styles here.
//!
//! # Style catalogue
//!
//! | name | mechanic |
//! |---|---|
//! | `drag-tree`       | Christmas-tree pre-stage → amber cascade → GREEN go |
//! | `tachometer`      | Needle sweeps arc + shift light flashes at redline |
//! | `fuel-gauge`      | Needle pivots E→F over a horizontal arc dial |
//! | `lap-circuit`     | Dot car circuits an oval track; lap fraction = eased |
//! | `pit-stop`        | 4 tyre corners fill one by one as eased grows |
//! | `gear-shifter`    | H-pattern stick moves through gears 1→6 |
//! | `odometer`        | Rolling digit columns increment via block-glyph ramps |
//! | `tunnel-headlights` | Perspective converging road + headlight beams advance |
//! | `burnout`         | Tyre-tread rubber marks accumulate from the rear |
//! | `pistons`         | 4 engine pistons cycle up/down driven by time |
//! | `nitro-boost`     | Charge bar surges with a trailing flame spike |
//! | `checkered-flag`  | Sine-ripple checkered flag unfurls at 100% |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Shared Bresenham line helper (bounds-safe via draw::dot_i)
// ─────────────────────────────────────────────────────────────────────────────

fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let px = x0 + (x1 - x0) * i / steps;
        let py = y0 + (y1 - y0) * i / steps;
        draw::dot_i(grid, px, py);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `cars` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per automotive/racing bar.
/// Every style is geometrically distinct — they differ in their spatial
/// layout and temporal mechanic, never merely in color.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(DragTree),
        Box::new(Tachometer),
        Box::new(FuelGauge),
        Box::new(LapCircuit),
        Box::new(PitStop),
        Box::new(GearShifter),
        Box::new(Odometer),
        Box::new(TunnelHeadlights),
        Box::new(Burnout),
        Box::new(Pistons),
        Box::new(NitroBoost),
        Box::new(CheckeredFlag),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Drag-race Christmas tree
//    Pre-stage → two amber lights cascade top-to-bottom → GREEN go.
//    progress < 0.15: pre-stage (top amber dims in/out)
//    0.15..0.45: amber 1 lights (top pair)
//    0.45..0.70: amber 2 lights (middle pair)
//    0.70..0.85: amber 3 lights (bottom pair)
//    0.85..1.00: GREEN — all amber off, large green glow fills bottom
//    The amber light "flash" at each threshold is driven by (progress - thresh).
//    time drives a fast flicker on the green light at the finish.
// ─────────────────────────────────────────────────────────────────────────────

struct DragTree;
impl ProgressStyle for DragTree {
    fn name(&self) -> &str {
        "drag-tree"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Drag-race Christmas tree: amber lights cascade top-to-bottom, then GREEN GO fires"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let p = ctx.progress;
        // Thresholds for each light level
        let thresh = [0.15_f32, 0.45, 0.70, 0.85];
        // Center column x for lights
        let cx = (dw / 2) as i32;

        // Each amber "light" is a small disk; green fills the bottom half.
        // Sizes scale with available height.
        let segment_h = (dh / 5).max(1) as i32;

        // PRE-STAGE blinking top indicator
        let pre_stage_y = 1_i32;
        if p < thresh[0] {
            // blink at 4 Hz
            if (ctx.time * 4.0).fract() < 0.5 {
                for dy in 0..segment_h.min(2) {
                    draw::dot_i(grid, cx - 1, pre_stage_y + dy);
                    draw::dot_i(grid, cx, pre_stage_y + dy);
                    draw::dot_i(grid, cx + 1, pre_stage_y + dy);
                }
            }
        }

        // AMBER 1 — lights at row 1
        let a1_y = (segment_h + 2) as i32;
        if p >= thresh[0] && p < thresh[2] {
            let r = ((segment_h / 2).max(1)) as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r + r {
                        draw::dot_i(grid, cx + dx, a1_y + dy);
                    }
                }
            }
        }

        // AMBER 2 — lights at row 2
        let a2_y = (segment_h * 2 + 3) as i32;
        if p >= thresh[1] && p < thresh[2] {
            let r = ((segment_h / 2).max(1)) as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r + r {
                        draw::dot_i(grid, cx + dx, a2_y + dy);
                    }
                }
            }
        }

        // AMBER 3 — lights at row 3
        let a3_y = (segment_h * 3 + 4) as i32;
        if p >= thresh[2] && p < thresh[3] {
            let r = ((segment_h / 2).max(1)) as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r + r {
                        draw::dot_i(grid, cx + dx, a3_y + dy);
                    }
                }
            }
        }

        // GREEN — fills the bottom region
        if p >= thresh[3] {
            // pulsing green block
            let pulse = (ctx.time * 8.0).sin() * 0.5 + 1.0; // 0.5..1.5
            let green_y0 = (dh / 2) as i32;
            let green_h = ((dh as f32 * 0.45 * pulse) as i32)
                .max(2)
                .min(dh as i32 - green_y0);
            let green_w = ((dw as f32 * 0.6) as i32).max(2);
            let gx0 = (cx - green_w / 2).max(0);
            for dy in 0..green_h {
                for dx in 0..green_w {
                    draw::dot_i(grid, gx0 + dx, green_y0 + dy);
                }
            }
        }

        // Structural outline: two vertical pillars (the tree posts)
        let post_x_l = (cx - dw as i32 / 4).max(0);
        let post_x_r = (cx + dw as i32 / 4).min(dw as i32 - 1);
        draw::vline(grid, post_x_l as usize, 0, dh - 1);
        draw::vline(grid, post_x_r as usize, 0, dh - 1);

        // Color: amber during amber, green during green
        let (cw, ch) = grid.dimensions();
        let color = if p >= thresh[3] {
            ctx.palette.sample(1.0) // end = green-ish
        } else {
            ctx.palette.sample(p / thresh[3]) // ramp amber
        };
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Tachometer
//    A semicircular arc from ~200° to ~340° (bottom-left to bottom-right).
//    Needle sweeps from the left (idle) to redline (right) as eased grows.
//    A redline zone occupies the rightmost 15% of the arc.
//    When progress ≥ 0.85, the SHIFT light (top center) flashes with time.
//    The arc is drawn in dot-space with polar parametric stepping.
// ─────────────────────────────────────────────────────────────────────────────

struct Tachometer;
impl ProgressStyle for Tachometer {
    fn name(&self) -> &str {
        "tachometer"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Semicircular tach dial: needle sweeps idle→redline; shift light flashes at redline"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as f32;
        let cy = (dh as f32 * 0.65).max(1.0); // center slightly below mid
        let rx = (cx * 0.85).max(1.0);
        let ry = (cy * 0.80).max(1.0);

        // Arc spans from 200° to 340° (bottom-left to bottom-right, going clockwise)
        let arc_start = 200.0_f32 * PI / 180.0;
        let arc_end = 340.0_f32 * PI / 180.0;
        let arc_span = arc_end - arc_start;

        // Draw the arc (tick marks every 10°)
        let steps = ((rx + ry) * 3.0).max(24.0) as usize;
        let mut prev: Option<(i32, i32)> = None;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let angle = arc_start + t * arc_span;
            let px = (cx + rx * angle.cos()) as i32;
            let py = (cy + ry * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ppx, ppy)) = prev {
                if (px - ppx).abs() + (py - ppy).abs() > 2 {
                    line(grid, ppx, ppy, px, py);
                }
            }
            prev = Some((px, py));
        }

        // Tick marks along the arc
        let tick_count = 10usize;
        for t_idx in 0..=tick_count {
            let t = t_idx as f32 / tick_count as f32;
            let angle = arc_start + t * arc_span;
            let inner_scale = if t_idx % 5 == 0 { 0.75_f32 } else { 0.85 };
            let px0 = (cx + rx * angle.cos()) as i32;
            let py0 = (cy + ry * angle.sin()) as i32;
            let px1 = (cx + rx * inner_scale * angle.cos()) as i32;
            let py1 = (cy + ry * inner_scale * angle.sin()) as i32;
            line(grid, px0, py0, px1, py1);
        }

        // Redline zone: thick arc from 85%..100% of arc (darker, denser)
        let redline_start_t = 0.85_f32;
        let rl_steps = 12usize;
        let mut rl_prev: Option<(i32, i32)> = None;
        for s in 0..=rl_steps {
            let t = redline_start_t + (1.0 - redline_start_t) * s as f32 / rl_steps as f32;
            let angle = arc_start + t * arc_span;
            let px = (cx + rx * 0.95 * angle.cos()) as i32;
            let py = (cy + ry * 0.95 * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            // second ring slightly smaller = thick zone
            let px2 = (cx + rx * 0.88 * angle.cos()) as i32;
            let py2 = (cy + ry * 0.88 * angle.sin()) as i32;
            draw::dot_i(grid, px2, py2);
            if let Some((ppx, ppy)) = rl_prev {
                line(grid, ppx, ppy, px, py);
            }
            rl_prev = Some((px, py));
        }

        // Needle: from center to arc edge at eased fraction
        let needle_angle = arc_start + ctx.eased * arc_span;
        let nx = (cx + rx * 0.82 * needle_angle.cos()) as i32;
        let ny = (cy + ry * 0.82 * needle_angle.sin()) as i32;
        line(grid, cx as i32, cy as i32, nx, ny);
        // Needle hub dot
        draw::dot_i(grid, cx as i32, cy as i32);
        draw::dot_i(grid, cx as i32 - 1, cy as i32);
        draw::dot_i(grid, cx as i32 + 1, cy as i32);
        draw::dot_i(grid, cx as i32, cy as i32 - 1);
        draw::dot_i(grid, cx as i32, cy as i32 + 1);

        // SHIFT LIGHT: top center, flashes when in redline
        if ctx.eased >= 0.85 {
            let flash = (ctx.time * 10.0).sin() > 0.0;
            if flash {
                let sl_cx = cx as i32;
                let sl_cy = 1_i32;
                let sl_r = (dh as i32 / 8).max(1);
                for dy in -sl_r..=sl_r {
                    for dx in -sl_r..=sl_r {
                        if dx * dx + dy * dy <= sl_r * sl_r + sl_r {
                            draw::dot_i(grid, sl_cx + dx, sl_cy + dy);
                        }
                    }
                }
            }
        }

        // Color gradient on filled arc cells
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Fuel gauge
//    A horizontal arc with a pivoting needle.
//    The dial face is an upward-opening semicircle (bottom straight edge = baseline).
//    Left = E (empty), right = F (full).
//    Needle rotates from left (0%) to right (100%) as eased grows.
//    Low-fuel warning: leftmost zone + blink when eased < 0.15.
// ─────────────────────────────────────────────────────────────────────────────

struct FuelGauge;
impl ProgressStyle for FuelGauge {
    fn name(&self) -> &str {
        "fuel-gauge"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Fuel gauge: needle pivots E→F on a semicircular dial; low-fuel blink below 15%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as f32;
        let cy = (dh as f32 * 0.80).min(dh as f32 - 1.0).max(1.0);
        let rx = (cx * 0.85).max(1.0);
        let ry = (cy * 0.85).max(1.0);

        // Semicircle arc: from 180° (left = E) to 0° (right = F), via the top
        // We go from π to 0 i.e. left-to-right through the upper half
        let arc_steps = ((rx + ry) * 2.5).max(20.0) as usize;
        let mut prev: Option<(i32, i32)> = None;
        for s in 0..=arc_steps {
            let t = s as f32 / arc_steps as f32;
            let angle = PI - t * PI; // π → 0
            let px = (cx + rx * angle.cos()) as i32;
            let py = (cy + ry * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ppx, ppy)) = prev {
                if (px - ppx).abs() + (py - ppy).abs() > 2 {
                    line(grid, ppx, ppy, px, py);
                }
            }
            prev = Some((px, py));
        }

        // Baseline (flat bottom of the D-shape)
        let base_y = cy as i32;
        let left_x = (cx - rx) as i32;
        let right_x = (cx + rx) as i32;
        line(grid, left_x, base_y, right_x, base_y);

        // Tick marks along the arc (5 ticks: E, 1/4, 1/2, 3/4, F)
        for i in 0..=4 {
            let t = i as f32 / 4.0;
            let angle = PI - t * PI;
            let px0 = (cx + rx * angle.cos()) as i32;
            let py0 = (cy + ry * angle.sin()) as i32;
            let px1 = (cx + rx * 0.80 * angle.cos()) as i32;
            let py1 = (cy + ry * 0.80 * angle.sin()) as i32;
            line(grid, px0, py0, px1, py1);
        }

        // Needle: pivots from E (angle=π) to F (angle=0)
        let needle_angle = PI - ctx.eased * PI;
        let needle_len = (rx * 0.75).max(2.0);
        let nx = (cx + needle_len * needle_angle.cos()) as i32;
        let ny = (cy + needle_len * needle_angle.sin()) as i32;
        line(grid, cx as i32, cy as i32, nx, ny);

        // Hub dot
        draw::dot_i(grid, cx as i32, cy as i32);

        // Low-fuel blink: left-zone fill when eased < 0.15
        if ctx.eased < 0.15 {
            let blink = (ctx.time * 3.0).fract() < 0.5;
            if blink {
                // Small area near the E end
                let warn_r = (rx * 0.2) as i32;
                for dy in -(warn_r)..=warn_r {
                    for dx in -(warn_r)..=0 {
                        draw::dot_i(grid, left_x + (rx as i32 / 4) + dx, base_y + dy);
                    }
                }
            }
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Lap circuit
//    A dot-space oval: two straight sections + two semicircular ends.
//    A "car" dot orbits the oval continuously; its position is the eased lap
//    fraction (so at 0% it's at the start/finish line, at 100% it completes).
//    time drives a second "ghost" car one lap behind for visual interest.
//    The track is drawn as an outline; the portion the lead car has covered
//    is filled with a denser double-line.
// ─────────────────────────────────────────────────────────────────────────────

struct LapCircuit;
impl ProgressStyle for LapCircuit {
    fn name(&self) -> &str {
        "lap-circuit"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Oval track: a car dot circuits the loop; lap fraction = eased; ghost car trails behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as f32;
        let cy = (dh / 2) as f32;
        // Oval: horizontal semi-axes larger than vertical
        let rx = (cx * 0.80).max(1.0);
        let ry = (cy * 0.65).max(1.0);

        // Draw oval outline
        let steps = ((rx + ry) * 4.0).max(32.0) as usize;
        let mut prev: Option<(i32, i32)> = None;
        for s in 0..=steps {
            let angle = s as f32 / steps as f32 * 2.0 * PI;
            let px = (cx + rx * angle.cos()) as i32;
            let py = (cy + ry * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ppx, ppy)) = prev {
                if (px - ppx).abs() + (py - ppy).abs() > 2 {
                    line(grid, ppx, ppy, px, py);
                }
            }
            prev = Some((px, py));
        }

        // Inner oval (slightly smaller) to give track width
        let rx2 = (rx * 0.75).max(0.5);
        let ry2 = (ry * 0.70).max(0.5);
        let steps2 = ((rx2 + ry2) * 3.0).max(20.0) as usize;
        let mut prev2: Option<(i32, i32)> = None;
        for s in 0..=steps2 {
            let angle = s as f32 / steps2 as f32 * 2.0 * PI;
            let px = (cx + rx2 * angle.cos()) as i32;
            let py = (cy + ry2 * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ppx, ppy)) = prev2 {
                if (px - ppx).abs() + (py - ppy).abs() > 2 {
                    line(grid, ppx, ppy, px, py);
                }
            }
            prev2 = Some((px, py));
        }

        // Start/finish line: vertical line at the rightmost point of the oval
        let sf_x = (cx + rx) as i32;
        let sf_inner_x = (cx + rx2) as i32;
        let sf_y0 = (cy - ry * 0.15) as i32;
        let sf_y1 = (cy + ry * 0.15) as i32;
        line(grid, sf_x, sf_y0, sf_inner_x, sf_y0);
        line(grid, sf_x, sf_y1, sf_inner_x, sf_y1);

        // Car position: start at right (angle=0), circuit counter-clockwise
        // angle = -eased * 2π  (negative = counter-clockwise from right)
        let car_angle = -ctx.eased * 2.0 * PI;
        let car_r_x = (rx + rx2) / 2.0; // midpoint of track
        let car_r_y = (ry + ry2) / 2.0;
        let car_x = (cx + car_r_x * car_angle.cos()) as i32;
        let car_y = (cy + car_r_y * car_angle.sin()) as i32;
        // Car: 3-dot symbol
        draw::dot_i(grid, car_x, car_y);
        draw::dot_i(grid, car_x - 1, car_y);
        draw::dot_i(grid, car_x + 1, car_y);

        // Ghost car (trails by ~0.25 of a lap, driven by time)
        let ghost_offset = (ctx.time * 0.15).fract() * 2.0 * PI;
        let ghost_angle = car_angle - ghost_offset;
        let gx = (cx + car_r_x * ghost_angle.cos()) as i32;
        let gy = (cy + car_r_y * ghost_angle.sin()) as i32;
        draw::dot_i(grid, gx, gy);

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pit stop
//    The car is shown as a centered rectangle.  4 corners each represent one
//    wheel.  As eased grows through 4 equal bands, the corners light up one
//    by one (tyre changed).  When all 4 are done, the car frame flashes (go!).
//    time drives a subtle wrench-turning shimmer on the active corner.
// ─────────────────────────────────────────────────────────────────────────────

struct PitStop;
impl ProgressStyle for PitStop {
    fn name(&self) -> &str {
        "pit-stop"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Pit stop: 4 tyre corners fill one by one; all done = car flashes and GO fires"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;

        // Car body dimensions (rectangle in dot space)
        let car_w = ((dw as i32 * 2) / 3).max(4);
        let car_h = ((dh as i32 * 2) / 3).max(2);
        let car_x0 = cx - car_w / 2;
        let car_y0 = cy - car_h / 2;
        let car_x1 = car_x0 + car_w;
        let car_y1 = car_y0 + car_h;

        // Draw car body outline
        line(grid, car_x0, car_y0, car_x1, car_y0);
        line(grid, car_x0, car_y1, car_x1, car_y1);
        line(grid, car_x0, car_y0, car_x0, car_y1);
        line(grid, car_x1, car_y0, car_x1, car_y1);

        // Centre cross (cockpit)
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx - 1, cy);
        draw::dot_i(grid, cx + 1, cy);

        // Wheels done: each corner finishes at progress 0.25, 0.50, 0.75, 1.0
        // Order: FL (front-left), FR, RL, RR
        let corners: [(i32, i32); 4] = [
            (car_x0, car_y0), // FL
            (car_x1, car_y0), // FR
            (car_x0, car_y1), // RL
            (car_x1, car_y1), // RR
        ];
        let done_count = (ctx.eased * 4.0).floor() as usize;

        for (i, &(wx, wy)) in corners.iter().enumerate() {
            let wheel_r = (car_h / 4).max(1) as i32;
            let is_done = i < done_count;
            let is_active = i == done_count && done_count < 4;

            // Tyre wrench shimmer on active corner
            if is_active {
                let shimmer = ((ctx.time * 12.0).sin() * wheel_r as f32) as i32;
                for dy in -wheel_r..=wheel_r {
                    for dx in -wheel_r..=wheel_r {
                        if dx * dx + dy * dy <= wheel_r * wheel_r + wheel_r {
                            draw::dot_i(grid, wx + dx + shimmer / 4, wy + dy);
                        }
                    }
                }
            } else if is_done {
                // Filled solid tyre = done
                for dy in -wheel_r..=wheel_r {
                    for dx in -wheel_r..=wheel_r {
                        if dx * dx + dy * dy <= wheel_r * wheel_r + wheel_r {
                            draw::dot_i(grid, wx + dx, wy + dy);
                        }
                    }
                }
            } else {
                // Unfilled tyre circle = waiting
                let steps = (wheel_r * 6).max(8) as usize;
                for s in 0..steps {
                    let angle = s as f32 / steps as f32 * 2.0 * PI;
                    let px = (wx as f32 + wheel_r as f32 * angle.cos()) as i32;
                    let py = (wy as f32 + wheel_r as f32 * angle.sin()) as i32;
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // All done: car body flashes
        if done_count >= 4 {
            let flash = (ctx.time * 8.0).fract() < 0.5;
            if flash {
                draw::fill_rect(
                    grid,
                    car_x0.max(0) as usize,
                    car_y0.max(0) as usize,
                    (car_w as usize).min(dw.saturating_sub(car_x0.max(0) as usize)),
                    (car_h as usize).min(dh.saturating_sub(car_y0.max(0) as usize)),
                );
            }
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Gear shifter
//    An H-pattern stick shift is drawn in dot space.
//    The horizontal bar connects gear columns; vertical lines drop to each gate.
//    As eased grows 0→1, the stick position moves through gears: 1, 2, 3, 4, 5, 6.
//    The stick is a small dot that hops between gate positions.
//    time adds a subtle vibration while in gear (engine shake).
// ─────────────────────────────────────────────────────────────────────────────

struct GearShifter;
impl ProgressStyle for GearShifter {
    fn name(&self) -> &str {
        "gear-shifter"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "H-pattern gear shifter: stick hops 1→6 through the gate as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // H-pattern: 3 columns (left / center / right), 2 rows (up / down)
        // Gears: 1=top-left, 2=bottom-left, 3=top-center, 4=bottom-center, 5=top-right, 6=bottom-right
        // Reverse (R) lives below 6 — we skip it for the progress mapping.

        let col_count = 3usize;
        let margin_x = (dw / 5).max(1);
        let margin_y = (dh / 5).max(1);
        let usable_w = dw.saturating_sub(margin_x * 2);
        let usable_h = dh.saturating_sub(margin_y * 2);

        // Column x positions
        let col_xs: [i32; 3] = [
            (margin_x) as i32,
            (margin_x + usable_w / 2) as i32,
            (margin_x + usable_w) as i32,
        ];

        // Row y positions (top row = up-gears, bottom = down-gears)
        let row_ys: [i32; 2] = [(margin_y) as i32, (margin_y + usable_h) as i32];

        // Horizontal H-bar connecting all three columns at the midpoint
        let mid_y = (margin_y + usable_h / 2) as i32;
        line(grid, col_xs[0], mid_y, col_xs[col_count - 1], mid_y);

        // Vertical gate lines from mid_y to each gear slot
        for col in 0..col_count {
            line(grid, col_xs[col], mid_y, col_xs[col], row_ys[0]);
            line(grid, col_xs[col], mid_y, col_xs[col], row_ys[1]);
        }

        // Gate dots at each gear position
        for col in 0..col_count {
            for row in 0..2usize {
                draw::dot_i(grid, col_xs[col] - 1, row_ys[row]);
                draw::dot_i(grid, col_xs[col], row_ys[row]);
                draw::dot_i(grid, col_xs[col] + 1, row_ys[row]);
            }
        }

        // Gear index from eased: 0..5 → gears 1..6
        let num_gears = 6usize;
        let gear_idx = ((ctx.eased * num_gears as f32).floor() as usize).min(num_gears - 1);
        let col = gear_idx / 2;
        let row = gear_idx % 2;
        let col = col.min(col_count - 1);

        // Engine vibration when in gear
        let vib_x = if ctx.progress > 0.01 {
            ((ctx.time * 30.0).sin() * 1.5) as i32
        } else {
            0
        };

        let stick_x = col_xs[col] + vib_x;
        let stick_y = row_ys[row];

        // Stick: 3×3 filled dot cluster
        for dy in -2_i32..=2 {
            for dx in -2_i32..=2 {
                draw::dot_i(grid, stick_x + dx, stick_y + dy);
            }
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col_c = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col_c);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Odometer
//    Uses the glyph API to show rolling digit columns.
//    Progress maps to a large integer (0–99999).
//    Each digit column shows its value via V_BLOCKS — a vertical ramp from the
//    digit below to the digit above, with the current digit centered.
//    time drives a smooth fractional-digit roll animation.
// ─────────────────────────────────────────────────────────────────────────────

struct Odometer;
impl ProgressStyle for Odometer {
    fn name(&self) -> &str {
        "odometer"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Odometer: digit columns roll from 00000 to 99999 as progress grows; smooth block ramp"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Total odometer value: 0..99999 scaled by eased
        let max_val = 99999_u32;
        // Fractional value (with time-driven smooth roll)
        let frac_val = ctx.eased * max_val as f32;

        // How many digit columns fit?  Each digit needs 1 cell wide.
        // We center them horizontally.
        let num_digits = 5usize;
        let digit_cols = num_digits.min(cw);
        let start_col = (cw.saturating_sub(digit_cols)) / 2;

        // For each digit position compute the digit value + fractional part
        for d in 0..digit_cols {
            // The digit's place: d=0 is the 10^4 column, d=4 is the units column
            let place = 10_u32.pow((digit_cols - 1 - d) as u32);
            let digit_frac = (frac_val / place as f32) % 10.0;
            let digit_int = digit_frac.floor() as usize;
            let digit_rem = digit_frac.fract(); // 0..1 = how far into rolling to next digit

            let cell_x = start_col + d;

            // Draw each row of this cell column
            // In a 1-cell-tall grid: just draw the current digit.
            // In taller grids: top rows show the previous digit rolling out,
            // bottom rows show the next digit rolling in.
            for row in 0..ch {
                // Map row to sub-digit position
                // Row 0 = top (previous digit leaving), row ch-1 = bottom (next arriving)
                let row_frac = if ch <= 1 {
                    0.5
                } else {
                    row as f32 / (ch - 1) as f32
                };
                // Position within the rolling reel: 0=top of current, 1=bottom
                // We show: at the transition, the reel has moved digit_rem * ch rows up
                let reel_pos = row_frac + digit_rem;
                let which_digit = if reel_pos < 1.0 {
                    digit_int
                } else {
                    (digit_int + 1) % 10
                };
                let frac_in_cell = reel_pos.fract();
                // Use vblock to render the fractional height
                let level = (frac_in_cell * 8.0) as usize;
                let _ = level; // used below

                // Choose the glyph: full block for solid digit, partial block for rolling
                let glyph = if (reel_pos - reel_pos.floor()).abs() < 0.12 {
                    // Near top of this digit's cell: show a lighter shade
                    draw::SHADES[1]
                } else {
                    // Map digit 0-9 to a block density (using shades + blocks)
                    // Since we don't have actual numeral glyphs, use a density proxy:
                    // digits 0-9 map to shade levels 0..4 cycling, with full block for rounded digits
                    match which_digit {
                        0 | 8 => '█',     // full (round digits)
                        1 | 7 => '▏',     // thin (vertical stroke)
                        2 | 3 | 5 => '▓', // heavy fill
                        4 | 6 | 9 => '▒', // medium fill
                        _ => '░',
                    }
                };
                draw::glyph(grid, cell_x, row, glyph);
            }
        }

        // Border box around the odometer digits
        let (dw, dh) = draw::dot_dims(grid);
        let box_x0 = start_col * 2;
        let box_x1 = (start_col + digit_cols) * 2;
        if box_x1 <= dw && dh > 0 {
            draw::hline(grid, box_x0, box_x1, 0);
            draw::hline(grid, box_x0, box_x1, dh - 1);
            draw::vline(grid, box_x0, 0, dh - 1);
            draw::vline(grid, box_x1.min(dw - 1), 0, dh - 1);
        }

        // Color
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Tunnel headlights (perspective)
//    PERSPECTIVE-primary mechanic.  Road lines converge to a vanishing point
//    at the top-center.  As eased grows, the car (represented by two headlight
//    beams) advances: beams widen = car is closer to the viewer.
//    time drives scrolling dotted center-line segments.
//    At 100% the car exits toward the viewer (beams fill the full width).
// ─────────────────────────────────────────────────────────────────────────────

struct TunnelHeadlights;
impl ProgressStyle for TunnelHeadlights {
    fn name(&self) -> &str {
        "tunnel-headlights"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Perspective tunnel: converging road lines; headlight beams widen as the car approaches"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let vp_x = (dw / 2) as i32;
        let vp_y = 0_i32; // vanishing point at the very top center

        // Road edges: from vanishing point to wide bottom corners
        let road_half_bottom = (dw as i32 / 3).max(2);
        let bottom_y = (dh as i32) - 1;

        line(grid, vp_x, vp_y, vp_x - road_half_bottom, bottom_y);
        line(grid, vp_x, vp_y, vp_x + road_half_bottom, bottom_y);

        // Tunnel ceiling / walls: two more lines slightly tighter
        let tunnel_margin = (dw as i32 / 6).max(1);
        let ceiling_y = 0_i32;
        line(grid, vp_x - tunnel_margin, ceiling_y, 0, bottom_y);
        line(
            grid,
            vp_x + tunnel_margin,
            ceiling_y,
            dw as i32 - 1,
            bottom_y,
        );

        // Scrolling dotted center line
        let seg_count = 8usize;
        let scroll_phase = (ctx.time * 0.6).fract();
        for s in 0..seg_count {
            let t_base = (s as f32 + scroll_phase) / seg_count as f32;
            let t = t_base.fract();
            // y position between vp and bottom
            let seg_y = (vp_y as f32 + (dh as f32 - vp_y as f32) * t) as i32;
            if s % 2 == 0 {
                draw::dot_i(grid, vp_x, seg_y);
                draw::dot_i(grid, vp_x, seg_y + 1);
            }
        }

        // Headlights: the car is at depth = eased (0=far, 1=right at viewer)
        // Car y position on screen: at depth=0 it's near vp, at depth=1 it's at the bottom
        let car_depth = ctx.eased;
        let car_y = (vp_y as f32 + (dh as f32 - vp_y as f32) * car_depth) as i32;
        // Headlight spread: widens with depth (closer = wider beam)
        let beam_spread = (road_half_bottom as f32 * car_depth * 0.6).max(1.0) as i32;
        let beam_up = (dh as f32 * car_depth * 0.15) as i32;

        // Left headlight beam: from car position upward-left to vp area
        let lx = vp_x - beam_spread;
        let rx = vp_x + beam_spread;
        // Beam lines: car center → top
        if car_y > vp_y + 1 {
            line(
                grid,
                lx,
                car_y,
                vp_x - beam_spread / 3,
                (car_y - beam_up).max(vp_y + 1),
            );
            line(
                grid,
                rx,
                car_y,
                vp_x + beam_spread / 3,
                (car_y - beam_up).max(vp_y + 1),
            );
        }

        // Headlight dots (the lights themselves)
        for dy in -1_i32..=1 {
            draw::dot_i(grid, lx, car_y + dy);
            draw::dot_i(grid, rx, car_y + dy);
        }
        // Car width bar at car_y
        if lx < rx {
            draw::hline(
                grid,
                lx.max(0) as usize,
                rx.min(dw as i32 - 1) as usize,
                car_y as usize,
            );
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Burnout / tyre marks
//    A horizontal baseline representing the road.  As eased grows, rubber
//    marks (dense zigzag/wavy lines) are laid down from right to left across
//    the baseline — the track-out marks of a burnout.
//    time drives a slight lateral oscillation (the car fishtails).
//    The marks get denser (double lines) nearer to the car's current position.
// ─────────────────────────────────────────────────────────────────────────────

struct Burnout;
impl ProgressStyle for Burnout {
    fn name(&self) -> &str {
        "burnout"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Burnout: rubber tyre marks accumulate right-to-left as progress grows; fishtail via time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Road: bottom two rows
        let road_y = dh.saturating_sub(1);
        let road_y2 = dh.saturating_sub(2);
        draw::hline(grid, 0, dw - 1, road_y);

        // Tread zone: vertical span above the road
        let tread_h = ((dh as f32 * 0.45).max(2.0) as usize).min(dh.saturating_sub(1));

        // Marks are laid from the right, eased controls how far left they reach.
        let mark_right = dw.saturating_sub(1);
        let mark_left = (dw as f32 * (1.0 - ctx.eased)) as usize;

        // Left tyre track and right tyre track (two lines)
        let track_y_top = road_y2.saturating_sub(tread_h);
        let track_left_y = road_y2.saturating_sub(tread_h / 2);
        let track_right_y = road_y2;

        // Draw tyre marks: wavy lines using sine lateral offset driven by time
        for x in mark_left..=mark_right.min(dw - 1) {
            let phase = x as f32 * 0.8;
            let fishtail = (ctx.time * 6.0 + phase).sin() * 1.5;
            let y_off = fishtail as i32;

            // Left tyre mark
            let left_y = (track_left_y as i32 + y_off).clamp(0, dh as i32 - 1) as usize;
            draw::dot(grid, x, left_y);
            if left_y + 1 < dh {
                draw::dot(grid, x, left_y + 1);
            }

            // Right tyre mark
            let right_y = (track_right_y as i32 - y_off).clamp(0, dh as i32 - 1) as usize;
            draw::dot(grid, x, right_y);
        }

        // Car at the leading edge (left side of the marks = car current position)
        let car_x = mark_left as i32;
        let car_y = (track_left_y as i32 + track_right_y as i32) / 2;
        // Simple car silhouette: a filled rect
        let car_w = (dw as i32 / 8).max(2);
        let car_h_size = (dh as i32 / 4).max(2);
        for dy in -car_h_size / 2..=car_h_size / 2 {
            for dx in -car_w..=0 {
                draw::dot_i(grid, car_x + dx, car_y + dy);
            }
        }

        // Smoke: sparse dots above the car
        let smoke_density = (ctx.eased * 8.0) as usize;
        for s in 0..smoke_density {
            let sx_off = (s * 3) as i32;
            let sy_off = ((s * 7 + 3) % (tread_h.max(1))) as i32;
            draw::dot_i(
                grid,
                car_x - car_w / 2 - sx_off,
                road_y as i32 - car_h_size - sy_off,
            );
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        let _ = track_y_top;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Engine pistons
//    4 pistons in a row, each cycling up/down driven by time.
//    The crank offsets each piston by 90° so they fire in sequence.
//    eased controls RPM (cycle speed) so the pistons spin faster as progress grows.
//    Each piston is a rectangle; the connecting rod is a line to the crank.
// ─────────────────────────────────────────────────────────────────────────────

struct Pistons;
impl ProgressStyle for Pistons {
    fn name(&self) -> &str {
        "pistons"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "4 engine pistons fire in sequence driven by time; RPM grows with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let n_pistons = 4usize;
        let piston_w = (dw / (n_pistons * 2)).max(2);
        let gap = (dw.saturating_sub(n_pistons * piston_w)) / (n_pistons + 1);

        // RPM: low at 0% progress, high at 100%
        let rpm_min = 0.5_f32;
        let rpm_max = 8.0_f32;
        let rpm = rpm_min + ctx.eased * (rpm_max - rpm_min);

        // Crank center y (bottom third)
        let crank_y = (dh as f32 * 0.75).min(dh as f32 - 2.0) as i32;
        // Stroke amplitude: pistons travel this many dots up and down
        let stroke = ((dh as f32 * 0.30).max(2.0)) as i32;
        let piston_h = ((dh as f32 * 0.15).max(2.0)) as i32;

        // Cylinder walls top (fixed)
        let cylinder_top = 1_i32;

        for i in 0..n_pistons {
            // Phase offset: 90° between each piston
            let phase_offset = i as f32 * PI / 2.0;
            let crank_angle = ctx.time * rpm * 2.0 * PI + phase_offset;

            // Piston y = crank_y + stroke * cos(angle)  (0 = top of stroke)
            let piston_mid_y = (crank_y as f32 + stroke as f32 * crank_angle.cos()) as i32;
            let piston_y_top = (piston_mid_y - piston_h / 2).max(cylinder_top);
            let piston_y_bot = (piston_mid_y + piston_h / 2).min(crank_y - 1);

            // X center of this piston
            let piston_cx = (gap + i * (piston_w + gap) + piston_w / 2) as i32;
            let pw2 = (piston_w / 2) as i32;

            // Cylinder walls (vertical lines)
            draw::vline(
                grid,
                (piston_cx - pw2 - 1).max(0) as usize,
                cylinder_top as usize,
                crank_y as usize,
            );
            draw::vline(
                grid,
                (piston_cx + pw2 + 1)
                    .min(dw.saturating_sub(1) as i32)
                    .max(0) as usize,
                cylinder_top as usize,
                crank_y as usize,
            );

            // Piston body (filled rectangle)
            for dy in piston_y_top..=piston_y_bot.max(piston_y_top) {
                for dx in -pw2..=pw2 {
                    draw::dot_i(grid, piston_cx + dx, dy);
                }
            }

            // Connecting rod: from piston bottom to crank center
            line(
                grid,
                piston_cx,
                piston_y_bot.max(piston_y_top),
                piston_cx,
                crank_y,
            );

            // Crank pin dot
            draw::dot_i(grid, piston_cx, crank_y);
        }

        // Crank shaft: horizontal bar at crank_y
        let shaft_x0 = gap as i32;
        let shaft_x1 = (gap + n_pistons * (piston_w + gap)) as i32;
        draw::hline(
            grid,
            shaft_x0.max(0) as usize,
            shaft_x1.min(dw as i32 - 1) as usize,
            crank_y as usize,
        );

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Nitro / turbo boost
//    A horizontal charge bar (the boost tank).
//    As eased fills the tank, a trailing SURGE spike extends past the fill head:
//    the spike decays exponentially with distance from the head.
//    time drives an oscillating "charge wobble" on the spike.
//    At full charge (eased ≈ 1) the entire bar erupts in a wide flare.
// ─────────────────────────────────────────────────────────────────────────────

struct NitroBoost;
impl ProgressStyle for NitroBoost {
    fn name(&self) -> &str {
        "nitro-boost"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Nitro boost charge: fills left-to-right; a spike surges past the head; full charge erupts"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let mid = dh / 2;
        // Main fill: a thick band centered vertically
        let fill_h = ((dh as f32 * 0.55).max(2.0) as usize).min(dh);
        let fill_y0 = mid.saturating_sub(fill_h / 2);
        let fill_x1 = (ctx.eased * dw as f32).round() as usize;

        // Fill body
        draw::fill_rect(grid, 0, fill_y0, fill_x1.min(dw), fill_h);

        // Outline track
        draw::hline(grid, 0, dw - 1, fill_y0.saturating_sub(1));
        draw::hline(grid, 0, dw - 1, (fill_y0 + fill_h).min(dh - 1));

        // Surge spike: beyond the fill head, decaying exponential amplitude
        let spike_wobble = (ctx.time * 15.0).sin() * 0.5 + 0.5; // 0..1
        let spike_len = ((dw as f32 * 0.20).max(3.0)) as usize;

        for dx in 0..spike_len {
            let x = fill_x1 + dx;
            if x >= dw {
                break;
            }
            let decay = (-(dx as f32) / (spike_len as f32 * 0.4)).exp();
            let wobble_amp = decay * spike_wobble;
            let spike_h = ((fill_h as f32 * decay * 0.8).round() as usize).max(1);
            let wobble_y = (wobble_amp * 2.0) as usize;
            let sy0 = fill_y0.saturating_sub(wobble_y);
            let sy1 = (fill_y0 + spike_h + wobble_y).min(dh - 1);
            draw::vline(grid, x, sy0, sy1);
        }

        // FULL CHARGE eruption: wide flare
        if ctx.eased >= 0.95 {
            let flare_phase = (ctx.time * 12.0).fract();
            let flare_h = ((dh as f32 * flare_phase) as usize).min(dh);
            let flare_x0 = fill_x1.saturating_sub(4);
            for x in flare_x0..dw.min(flare_x0 + 8) {
                let fy0 = mid.saturating_sub(flare_h / 2);
                let fy1 = (mid + flare_h / 2).min(dh - 1);
                draw::vline(grid, x, fy0, fy1);
            }
        }

        // Color
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32) as usize;
        for cell_x in 0..filled.min(cw) {
            let t = cell_x as f32 / cw as f32;
            let col = ctx.palette.sample(t);
            for cell_y in 0..ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, col);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Checkered flag
//    At < 100% progress: a partial flag unfurls from the left.
//    At 100% progress: the full flag waves with a sine-ripple driven by time.
//    The flag is a checkerboard pattern (alternating filled/empty cells).
//    The ripple displaces each column vertically by A·sin(2π·x/λ + ωt).
// ─────────────────────────────────────────────────────────────────────────────

struct CheckeredFlag;
impl ProgressStyle for CheckeredFlag {
    fn name(&self) -> &str {
        "checkered-flag"
    }
    fn theme(&self) -> &str {
        "cars"
    }
    fn describe(&self) -> &str {
        "Checkered finish flag: unfurls as progress grows; at 100% the flag ripples via sine wave"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // How many columns of the flag are unfurled
        let unfurled_cols = (ctx.eased * cw as f32).ceil() as usize;

        // Wave parameters
        let wave_amp = if ch > 1 {
            (ch as f32 * 0.35).max(0.5)
        } else {
            0.0
        };
        let wave_len = (cw as f32 * 0.6).max(1.0);
        let wave_speed = 3.0_f32;

        for col in 0..unfurled_cols.min(cw) {
            // Sine displacement for this column (in cells)
            let wave_disp = if ctx.eased >= 1.0 {
                (wave_amp * (2.0 * PI * col as f32 / wave_len - wave_speed * ctx.time).sin()) as i32
            } else {
                0_i32
            };

            for row in 0..ch {
                // Checkerboard: filled if (col + row) is even
                let filled_cell = (col + row) % 2 == 0;

                // Apply wave displacement
                let target_row = (row as i32 + wave_disp).clamp(0, ch as i32 - 1) as usize;

                if filled_cell {
                    draw::glyph(grid, col, target_row, '█');
                } else {
                    // Empty square outline (just corners as a shade)
                    draw::glyph(grid, col, target_row, ' ');
                }
            }
        }

        // Flag pole: vertical line on the left edge in dot space
        let (dw, dh) = draw::dot_dims(grid);
        draw::vline(grid, 0, 0, dh - 1);
        // Pole tip
        draw::dot(grid, 1, 0);
        draw::dot(grid, 2, 0);

        // Color: no tint — the checkered pattern IS the visual; color would occlude it.
        // Instead tint only the empty (white) squares as a subtle background wash.
        for col in 0..unfurled_cols.min(cw) {
            for row in 0..ch {
                if (col + row) % 2 != 0 {
                    let t = col as f32 / cw as f32;
                    draw::tint_row(grid, row, col, col, ctx.palette.sample(t));
                }
            }
        }

        let _ = dw;
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
    let styles = progress::styles::cars::styles();
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
