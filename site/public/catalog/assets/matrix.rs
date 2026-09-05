//! `matrix` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O matrix.rs && ./matrix [style-name]
//! ```

const DEFAULT_STYLE: &str = "rain-fill";

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
    pub mod matrix {
//! Digital-rain progress bars — cascading code, phosphor decay, boot logs.
//!
//! The signature dotmax theme: everything is green-on-black terminal light.
//! Styles range from classic falling-glyph rain that settles into a solid
//! bar, to CRT raster scans, packet streams, and a self-typing boot log.
//! All motion is deterministic in `(ctx.progress, ctx.time)`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::TAU;

// ─── deterministic hash ─────────────────────────────────────────────────────

/// Fast integer hash → `[0, 1)`.
#[inline]
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) % 1000) as f32 / 1000.0
}

/// 3-D variant: hash `(x, y, z_int)` — useful for animating with a time slot.
#[inline]
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash2(x ^ z.wrapping_mul(1_234_567), y ^ z.wrapping_mul(7_654_321))
}

// ─── theme tint — phosphor green ────────────────────────────────────────────

/// Deep phosphor green, the unlit end of the ramp.
const TINT_DARK: Color = Color::rgb(14, 92, 55);
/// Bright terminal green, the lit end of the ramp.
const TINT_BRIGHT: Color = Color::rgb(110, 231, 160);
/// White-hot head color for leading edges.
const TINT_HOT: Color = Color::rgb(214, 255, 226);

/// Sample the dark→bright green ramp at `t` in `0.0..=1.0`.
fn sample_tint(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(
        lerp(TINT_DARK.r, TINT_BRIGHT.r),
        lerp(TINT_DARK.g, TINT_BRIGHT.g),
        lerp(TINT_DARK.b, TINT_BRIGHT.b),
    )
}

/// Applies the phosphor-green tint to every cell the inner style drew, with a
/// per-cell shimmer so the field of glyphs never sits still.
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
        let slot = (ctx.time * 3.0) as i32;
        for y in 0..h {
            for x in 0..w {
                let ch = grid.get_char(x, y);
                if ch != '\u{2800}' && ch != ' ' {
                    let ramp = 0.25 + 0.45 * (x as f32 / w.max(1) as f32);
                    let shimmer = 0.3 * hash3(x as i32, y as i32, slot);
                    let _ = grid.set_cell_color(x, y, sample_tint(ramp + shimmer));
                }
            }
        }
        Ok(())
    }
}

/// All styles in the `matrix` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(RainFill),
        Box::new(Tinted(Decode)),
        Box::new(Trace),
        Box::new(Tinted(CascadeWipe)),
        Box::new(Tinted(CodeColumn)),
        Box::new(PhosphorTrail),
        Box::new(Downlink),
        Box::new(PacketBurst),
        Box::new(GreenNoiseFill),
        Box::new(TerminalBoot),
    ]
}

/// Falling code streams settle left-to-right into a solid bar.
struct RainFill;
impl ProgressStyle for RainFill {
    fn name(&self) -> &str {
        "rain-fill"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Digital rain settling into a solid bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // Settled region: solid columns with a ragged, hash-torn top edge.
        for x in 0..filled {
            let torn = (hash2(x as i32, 7) * 2.5) as usize;
            for y in torn..h {
                draw::dot(grid, x, y);
            }
            let t = 0.35 + 0.45 * (x as f32 / w.max(1) as f32);
            draw::tint_row(grid, 0, x / 2, x / 2, sample_tint(t));
            for cy in 1..grid.dimensions().1 {
                draw::tint_row(grid, cy, x / 2, x / 2, sample_tint(t));
            }
        }
        // Active region: streams of rain with bright heads and fading tails.
        let cycle = (h * 2) as f32;
        for x in filled..w {
            let speed = 6.0 + hash2(x as i32, 1) * 8.0;
            // 0.25-multiple rates keep the 4 s capture loop seamless.
            let speed = (speed * 4.0).round() / 4.0;
            let head = ((ctx.time * speed + hash2(x as i32, 2) * cycle) % cycle) as i32;
            let tail = 3 + (hash2(x as i32, 3) * 4.0) as i32;
            for k in 0..=tail {
                let y = head - k;
                if y >= 0 && (y as usize) < h {
                    draw::dot(grid, x, y as usize);
                    let bright = if k == 0 {
                        TINT_HOT
                    } else {
                        sample_tint(0.8 - 0.2 * k as f32)
                    };
                    let _ = grid.set_cell_color(x / 2, y as usize / 4, bright);
                }
            }
        }
        Ok(())
    }
}

/// Scrambled code characters resolve into solid blocks at the leading edge.
struct Decode;
impl ProgressStyle for Decode {
    fn name(&self) -> &str {
        "decode"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Scrambled glyphs resolving into a solid bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        const CODE: [char; 12] = ['0', '1', '<', '>', '/', '#', '$', '+', '=', '*', '?', ';'];
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        let slot = (ctx.time * 10.0) as i32;
        for y in 0..ch {
            for x in 0..cw {
                if x + 2 < filled {
                    draw::glyph(grid, x, y, '█');
                } else if x < filled + 4 {
                    // Decode band: glyphs flip fast, density fades rightward.
                    let fade = 1.0 - (x as f32 - filled as f32 + 2.0) / 6.0;
                    if hash3(x as i32, y as i32, slot / 3) < fade {
                        let pick = (hash3(x as i32, y as i32, slot) * CODE.len() as f32) as usize;
                        draw::glyph(grid, x, y, CODE[pick.min(CODE.len() - 1)]);
                    }
                } else if hash3(x as i32, y as i32, slot / 4) < 0.05 {
                    // Sparse idle static far right of the edge.
                    draw::dot(grid, x * 2, y * 4 + 2);
                }
            }
        }
        Ok(())
    }
}

/// Light packets race along a phosphor track toward the leading edge.
struct Trace;
impl ProgressStyle for Trace {
    fn name(&self) -> &str {
        "trace"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Packets racing along a phosphor track"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h / 2;
        let filled = (ctx.eased * w as f32).round() as usize;
        // Dim dotted track across the whole width.
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, TINT_DARK);
        }
        // Completed run: a solid double line.
        for x in 0..filled {
            draw::dot(grid, x, mid.saturating_sub(1));
            draw::dot(grid, x, mid);
            let t = 0.3 + 0.3 * (x as f32 / w.max(1) as f32);
            let _ = grid.set_cell_color(x / 2, mid / 4, sample_tint(t));
        }
        // Packets: bright dashes streaming through the completed run.
        if filled > 2 {
            for i in 0..3 {
                let phase = (ctx.time * 0.5 + i as f32 / 3.0).fract();
                let head = (phase * filled as f32) as usize;
                for k in 0..6 {
                    if head >= k {
                        let x = head - k;
                        draw::dot(grid, x, mid.saturating_sub(1));
                        draw::dot(grid, x, mid);
                        // The packet head bulges above and below the track so
                        // the stream reads even without color.
                        if k < 2 {
                            draw::dot(grid, x, mid.saturating_sub(2));
                            draw::dot(grid, x, (mid + 1).min(h - 1));
                        }
                        let c = if k < 2 {
                            TINT_HOT
                        } else {
                            sample_tint(0.9 - 0.15 * k as f32)
                        };
                        let _ = grid.set_cell_color(x / 2, mid / 4, c);
                    }
                }
            }
        }
        // Blinking terminal tick at the leading edge.
        if (ctx.time * 2.0) as i32 % 2 == 0 {
            let x = filled.min(w.saturating_sub(1));
            draw::vline(grid, x, mid.saturating_sub(3), (mid + 3).min(h - 1));
            let _ = grid.set_cell_color(x / 2, mid / 4, TINT_HOT);
        }
        Ok(())
    }
}

/// A glyph waterfall wipes across, dissolving from solid to mist.
struct CascadeWipe;
impl ProgressStyle for CascadeWipe {
    fn name(&self) -> &str {
        "cascade-wipe"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Waterfall wipe dissolving from solid to mist"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        let front = ctx.eased * (cw as f32 + 6.0);
        for y in 0..ch {
            for x in 0..cw {
                let rag = hash2(x as i32, y as i32) * 3.0;
                let drip = if hash2(x as i32, 40) < 0.25 {
                    y as f32 * 0.8
                } else {
                    0.0
                };
                let shimmer = 0.4 * (ctx.time * TAU * 0.5 + x as f32 * 0.7).sin();
                let depth = front - x as f32 - rag + drip + shimmer;
                let level = if depth >= 3.0 {
                    4
                } else if depth >= 2.0 {
                    3
                } else if depth >= 1.0 {
                    2
                } else {
                    usize::from(depth >= 0.0)
                };
                if level > 0 {
                    draw::shade(grid, x, y, level);
                }
            }
        }
        Ok(())
    }
}

/// Lines of code type themselves out, one blinking cursor at a time.
struct CodeColumn;
impl ProgressStyle for CodeColumn {
    fn name(&self) -> &str {
        "code-column"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Code lines typing in with a blinking cursor"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        let total = cw * ch;
        let typed = (ctx.eased * total as f32).round() as usize;
        for y in 0..ch {
            for x in 0..cw {
                let idx = y * cw + x;
                if idx >= typed {
                    continue;
                }
                // Pseudo source line: hash-chosen indent, chunks, and gaps.
                let indent = (hash2(y as i32, 0) * 5.0) as usize;
                if x < indent {
                    continue;
                }
                let chunk = hash2((x / 4) as i32, y as i32 + 31);
                if chunk < 0.72 && hash2(x as i32, y as i32 + 17) < 0.9 {
                    // The freshest line of code still settles: its glyphs
                    // flicker between weights as if mid-compile.
                    let fresh = typed - idx < cw;
                    let slot = (ctx.time * 4.0) as i32;
                    let heavy = if fresh && hash3(x as i32, y as i32, slot) < 0.35 {
                        chunk >= 0.2
                    } else {
                        chunk < 0.2
                    };
                    draw::glyph(grid, x, y, if heavy { '▓' } else { '█' });
                }
            }
        }
        // Blinking cursor at the caret.
        if typed < total && (ctx.time * 2.5) as i32 % 2 == 0 {
            draw::glyph(grid, typed % cw, typed / cw, '▌');
        }
        Ok(())
    }
}

/// A CRT raster scan lights the bar dot by dot, glow decaying behind it.
struct PhosphorTrail;
impl ProgressStyle for PhosphorTrail {
    fn name(&self) -> &str {
        "phosphor-trail"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "CRT raster scan with decaying phosphor glow"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let total = w * h;
        let lit = (ctx.eased * total as f32).round() as usize;
        let full_rows = lit / w.max(1);
        let partial = lit % w.max(1);
        // A CRT hum bar sweeps down the lit raster once per loop.
        let hum = ((ctx.time * 0.25).fract() * h as f32) as usize;
        for y in 0..full_rows.min(h) {
            if y != hum {
                draw::hline(grid, 0, w - 1, y);
            }
        }
        if full_rows < h {
            for x in 0..partial {
                draw::dot(grid, x, full_rows);
            }
            // Scan-head flare.
            draw::vline(
                grid,
                partial.min(w.saturating_sub(1)),
                full_rows.saturating_sub(1),
                (full_rows + 1).min(h - 1),
            );
        }
        // Phosphor decay: brightness falls with raster distance behind the head.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            for cx in 0..cw {
                if grid.get_char(cx, cy) == '\u{2800}' {
                    continue;
                }
                let cell_order = (cy * 4 + 2) * w + cx * 2;
                let behind = lit.saturating_sub(cell_order) as f32 / total.max(1) as f32;
                let flicker = 0.08 * hash3(cx as i32, cy as i32, (ctx.time * 6.0) as i32);
                let _ = grid.set_cell_color(cx, cy, sample_tint(1.0 - behind * 1.4 + flicker));
            }
        }
        // White-hot head cell.
        if full_rows < h {
            let _ = grid.set_cell_color(
                (partial.min(w.saturating_sub(1))) / 2,
                full_rows / 4,
                TINT_HOT,
            );
        }
        Ok(())
    }
}

/// Data packets fall from above into a pool that rises with progress.
struct Downlink;
impl ProgressStyle for Downlink {
    fn name(&self) -> &str {
        "downlink"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Falling packets filling a rising pool"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let pool = (ctx.eased * h as f32).round() as usize;
        let surface = h.saturating_sub(pool);
        // Pool body with a rippling surface line.
        for y in surface..h {
            for x in 0..w {
                let ripple = (x as f32 * 0.5 + ctx.time * TAU * 0.25).sin();
                if y == surface && ripple < -0.2 {
                    continue;
                }
                draw::dot(grid, x, y);
                let depth = (y - surface) as f32 / pool.max(1) as f32;
                let _ = grid.set_cell_color(x / 2, y / 4, sample_tint(0.75 - depth * 0.5));
            }
        }
        // Falling packets above the surface.
        if surface > 1 {
            for x in 0..w {
                if hash2(x as i32, 3) > 0.45 {
                    continue;
                }
                let speed = 4.0 + (hash2(x as i32, 4) * 8.0 * 4.0).round() / 4.0;
                let y = ((ctx.time * speed + hash2(x as i32, 5) * surface as f32) % surface as f32)
                    as usize;
                for k in 0..3usize {
                    if y >= k {
                        draw::dot(grid, x, y - k);
                    }
                }
                let c = if y + 2 >= surface {
                    TINT_HOT
                } else {
                    sample_tint(0.9)
                };
                let _ = grid.set_cell_color(x / 2, y / 4, c);
            }
        }
        Ok(())
    }
}

/// Packets stream home through staggered lanes and stack into the bar.
struct PacketBurst;
impl ProgressStyle for PacketBurst {
    fn name(&self) -> &str {
        "packet-burst"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Lane packets arriving into a growing stack"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let lanes = (h / 4).max(1);
        let filled = (ctx.eased * w as f32).round() as usize;
        for lane in 0..lanes {
            let y0 = lane * 4 + 1;
            let rag = (hash2(lane as i32, 9) * 4.0) as usize;
            let lane_fill = filled.saturating_sub(rag);
            // The stacked, completed run.
            for x in 0..lane_fill {
                draw::dot(grid, x, y0);
                draw::dot(grid, x, y0 + 1);
                let t = 0.3 + 0.4 * (x as f32 / w.max(1) as f32);
                let _ = grid.set_cell_color(x / 2, y0 / 4, sample_tint(t));
            }
            // Inbound packet: travels right-to-left into the stack.
            if lane_fill + 4 < w {
                let rate = 0.25 + 0.25 * (lane % 3) as f32;
                let span = (w - lane_fill) as f32;
                let pos = w as f32 - (ctx.time * rate + hash2(lane as i32, 11)).fract() * span;
                let px = pos as usize;
                for k in 0..5 {
                    let x = px + k;
                    if x < w {
                        draw::dot(grid, x, y0);
                        draw::dot(grid, x, y0 + 1);
                        let _ = grid.set_cell_color(x / 2, y0 / 4, TINT_HOT);
                    }
                }
                // Arrival flash at the stack edge.
                if pos < lane_fill as f32 + 4.0 {
                    draw::vline(grid, lane_fill, y0.saturating_sub(1), (y0 + 2).min(h - 1));
                    let _ = grid.set_cell_color(lane_fill / 2, y0 / 4, TINT_HOT);
                }
            }
        }
        Ok(())
    }
}

/// Dithered static condenses into solid signal behind a fizzing edge.
struct GreenNoiseFill;
impl ProgressStyle for GreenNoiseFill {
    fn name(&self) -> &str {
        "green-noise-fill"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Static condensing into solid signal"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let edge = ctx.eased * w as f32;
        let band = 12.0;
        let slot = (ctx.time * 8.0) as i32;
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32;
                let lit = if fx < edge - band {
                    // Solid signal with drifting pinprick holes.
                    hash3(x as i32, y as i32, (ctx.time * 3.0) as i32) < 0.96
                } else if fx < edge {
                    let t = (edge - fx) / band;
                    hash3(x as i32, y as i32, slot) < 0.05 + 0.9 * t
                } else {
                    hash3(x as i32, y as i32, slot) < 0.04
                };
                if lit {
                    draw::dot(grid, x, y);
                    let c = if fx >= edge - band && fx < edge {
                        sample_tint(0.85)
                    } else if fx < edge {
                        sample_tint(0.3 + 0.35 * (fx / w.max(1) as f32))
                    } else {
                        TINT_DARK
                    };
                    let _ = grid.set_cell_color(x / 2, y / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// A terminal boot log types itself in, percent readout on the last line.
struct TerminalBoot;
impl ProgressStyle for TerminalBoot {
    fn name(&self) -> &str {
        "terminal-boot"
    }
    fn theme(&self) -> &str {
        "matrix"
    }
    fn describe(&self) -> &str {
        "Boot log typing in with a live percent readout"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        const TEXT: [char; 4] = ['▄', '▀', '█', '▐'];
        let (cw, ch) = grid.dimensions();
        grid.enable_color_support();
        let total = cw * ch;
        let typed = (ctx.eased * total as f32).round() as usize;
        for y in 0..ch {
            for x in 0..cw {
                if y * cw + x >= typed {
                    continue;
                }
                if x == 0 {
                    draw::glyph(grid, x, y, '>');
                    let _ = grid.set_cell_color(x, y, TINT_BRIGHT);
                    continue;
                }
                // Word-like chunks separated by hash-chosen gaps.
                let line_len = 3 + (hash2(y as i32, 1) * (cw as f32 * 0.85)) as usize;
                if x > line_len || hash2((x / 3) as i32, y as i32 + 5) < 0.25 {
                    continue;
                }
                // The newest line of the log still churns as it prints.
                let idx = y * cw + x;
                let pick = if typed - idx < cw {
                    (hash3(x as i32, y as i32, (ctx.time * 4.0) as i32) * TEXT.len() as f32)
                        as usize
                } else {
                    (hash2(x as i32, y as i32) * TEXT.len() as f32) as usize
                };
                draw::glyph(grid, x, y, TEXT[pick.min(TEXT.len() - 1)]);
                let shimmer = 0.3 * hash3(x as i32, y as i32, (ctx.time * 2.0) as i32);
                let _ = grid.set_cell_color(x, y, sample_tint(0.45 + shimmer));
            }
        }
        // Blinking cursor at the caret.
        if typed < total && (ctx.time * 2.5) as i32 % 2 == 0 {
            draw::glyph(grid, typed % cw, typed / cw, '▌');
            let _ = grid.set_cell_color(typed % cw, typed / cw, TINT_HOT);
        }
        // Percent readout, bottom-right, in real text.
        if let Some(label) = &ctx.label {
            let chars: Vec<char> = label.chars().collect();
            if cw > chars.len() + 1 && ch > 0 {
                let x0 = cw - chars.len() - 1;
                for (i, c) in chars.iter().enumerate() {
                    draw::glyph(grid, x0 + i, ch - 1, *c);
                    let _ = grid.set_cell_color(x0 + i, ch - 1, TINT_HOT);
                }
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
    let styles = progress::styles::matrix::styles();
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
