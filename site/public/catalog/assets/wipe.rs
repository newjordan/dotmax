//! `wipe` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O wipe.rs && ./wipe [style-name]
//! ```

const DEFAULT_STYLE: &str = "linear-lr";

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
    pub mod wipe {
//! Screen-wipe / transition masks for full-screen reveals.
//!
//! Each style treats the grid as a transition mask: dots (the "filled" region)
//! represent the revealed or covered area at progress `t`.  At `t=0` the screen
//! is empty; at `t=1` it is fully filled.  A consumer can use the mask to
//! composite two frames; the gallery simply shows the wipe sweeping as progress
//! advances from 0 to 1.
//!
//! Styles in this module:
//!
//! | name              | mask geometry                                        |
//! |-------------------|------------------------------------------------------|
//! | `linear-lr`       | straight fill advancing left → right                |
//! | `linear-tb`       | straight fill advancing top → bottom                |
//! | `barn-door`       | two halves parting from the centre outward           |
//! | `iris`            | expanding filled circle from the centre              |
//! | `iris-diamond`    | expanding diamond (L1 / Manhattan distance)          |
//! | `diagonal`        | 45° diagonal wipe line sweeping across               |
//! | `checkerboard`    | checkerboard cells filling in threshold order        |
//! | `venetian-blinds` | horizontal slats that each grow from their midpoint  |
//! | `dissolve`        | ordered 4×4 Bayer-dither per-pixel threshold fade    |
//! | `clock-wipe`      | radial pie-slice sweeping 0 → 2π around the centre  |
//! | `spiral`          | Archimedean spiral mask filling outward              |
//! | `split`           | top and bottom edges advance inward to the midline   |
//! | `zigzag`          | wipe whose leading edge is a sine-wave sawtooth      |
//! | `pixelate`        | coarse blocks fill with increasing shade density     |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `wipe` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per transition-mask style.  Every style
/// fills the grid with braille dots representing the "revealed" region at the
/// current progress fraction.  They are structurally independent: each uses a
/// fundamentally different mask geometry.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(LinearLR),
        Box::new(LinearTB),
        Box::new(BarnDoor),
        Box::new(Iris),
        Box::new(IrisDiamond),
        Box::new(Diagonal),
        Box::new(Checkerboard),
        Box::new(VenetianBlinds),
        Box::new(Dissolve),
        Box::new(ClockWipe),
        Box::new(Spiral),
        Box::new(Split),
        Box::new(Zigzag),
        Box::new(Pixelate),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Linear left-to-right wipe
// ─────────────────────────────────────────────────────────────────────────────

struct LinearLR;
impl ProgressStyle for LinearLR {
    fn name(&self) -> &str {
        "linear-lr"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Hard-edge curtain sweeping left → right: a clean vertical wipe line \
         advances from the left edge to the right as progress rises 0→1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let filled = (ctx.eased * dw as f32).round() as usize;
        draw::fill_rect(grid, 0, 0, filled.min(dw), dh);
        // Tint the filled region.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if cw <= 1 {
                0.5
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

// ─────────────────────────────────────────────────────────────────────────────
// 2. Linear top-to-bottom wipe
// ─────────────────────────────────────────────────────────────────────────────

struct LinearTB;
impl ProgressStyle for LinearTB {
    fn name(&self) -> &str {
        "linear-tb"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Horizontal curtain dropping top → bottom: the revealed band grows \
         downward one dot-row at a time as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let filled_rows = (ctx.eased * dh as f32).round() as usize;
        draw::fill_rect(grid, 0, 0, dw, filled_rows.min(dh));
        // Tint per cell-row.
        let (cw, ch) = grid.dimensions();
        let filled_cell_rows = (ctx.eased * ch as f32).round() as usize;
        for cy in 0..filled_cell_rows.min(ch) {
            let t = if ch <= 1 {
                0.5
            } else {
                cy as f32 / (ch - 1) as f32
            };
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Barn-door: two halves part from the centre
// ─────────────────────────────────────────────────────────────────────────────

struct BarnDoor;
impl ProgressStyle for BarnDoor {
    fn name(&self) -> &str {
        "barn-door"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Barn-door wipe: the left half slides left and the right half slides \
         right, parting from the vertical centre seam as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let half = dw / 2;
        // Each door panel grows outward from the centre by `reach` dots.
        let reach = (ctx.eased * half as f32).round() as usize;
        // Left panel: fills from (half - reach) to half.
        let lx = half.saturating_sub(reach);
        draw::fill_rect(grid, lx, 0, half.saturating_sub(lx), dh);
        // Right panel: fills from half to (half + reach).
        let rx_end = (half + reach).min(dw);
        if rx_end > half {
            draw::fill_rect(grid, half, 0, rx_end - half, dh);
        }
        // Tint the two panels with opposite ends of the palette.
        let (cw, ch) = grid.dimensions();
        let half_c = cw / 2;
        let reach_c = (ctx.eased * half_c as f32).round() as usize;
        let lx_c = half_c.saturating_sub(reach_c);
        let rx_end_c = (half_c + reach_c).min(cw);
        for cy in 0..ch {
            let color_l = ctx.palette.sample(0.2);
            let color_r = ctx.palette.sample(0.8);
            if lx_c < half_c {
                draw::tint_row(grid, cy, lx_c, half_c.saturating_sub(1), color_l);
            }
            if rx_end_c > half_c {
                draw::tint_row(grid, cy, half_c, rx_end_c.saturating_sub(1), color_r);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Iris: expanding filled circle from the centre
// ─────────────────────────────────────────────────────────────────────────────

struct Iris;
impl ProgressStyle for Iris {
    fn name(&self) -> &str {
        "iris"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Iris wipe: a filled circle expands from the grid's centre, its radius \
         growing from zero to cover the full screen as progress reaches 1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // max_r must reach the far corner.
        let max_r = ((cx * cx + cy * cy) as f32).sqrt() + 1.0;
        let r = ctx.eased * max_r;
        let r2 = r * r;
        // Rasterise: any dot whose centre is within radius r is lit.
        for dy in 0..dh {
            let vy = dy as f32 + 0.5 - cy;
            for dx in 0..dw {
                let vx = dx as f32 + 0.5 - cx;
                if vx * vx + vy * vy <= r2 {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint radially.
        let (cw, ch) = grid.dimensions();
        let ccx = cw as f32 / 2.0;
        let ccy = ch as f32 / 2.0;
        let max_rc = ((ccx * ccx + ccy * ccy) as f32).sqrt().max(1.0);
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = cx_idx as f32 + 0.5 - ccx;
                let vy = cy_idx as f32 + 0.5 - ccy;
                let dist = (vx * vx + vy * vy).sqrt();
                if dist / max_rc <= ctx.eased {
                    let t = (dist / max_rc).clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Iris-diamond: expanding diamond (L1/Manhattan distance)
// ─────────────────────────────────────────────────────────────────────────────

struct IrisDiamond;
impl ProgressStyle for IrisDiamond {
    fn name(&self) -> &str {
        "iris-diamond"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Diamond iris wipe: a filled rhombus expands from the centre using \
         Manhattan (L1) distance — produces sharp 45° diamond edges instead of \
         the circular iris's smooth curve"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // max Manhattan radius to reach the corner.
        let max_r = cx + cy + 1.0;
        let r = ctx.eased * max_r;
        for dy in 0..dh {
            let vy = (dy as f32 + 0.5 - cy).abs();
            for dx in 0..dw {
                let vx = (dx as f32 + 0.5 - cx).abs();
                if vx + vy <= r {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint with L1-distance gradient.
        let (cw, ch) = grid.dimensions();
        let ccx = cw as f32 / 2.0;
        let ccy = ch as f32 / 2.0;
        let max_rc = (ccx + ccy).max(1.0);
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = (cx_idx as f32 + 0.5 - ccx).abs();
                let vy = (cy_idx as f32 + 0.5 - ccy).abs();
                let dist = vx + vy;
                if dist / max_rc <= ctx.eased {
                    let t = (dist / max_rc).clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Diagonal: 45° diagonal wipe
// ─────────────────────────────────────────────────────────────────────────────

struct Diagonal;
impl ProgressStyle for Diagonal {
    fn name(&self) -> &str {
        "diagonal"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Diagonal wipe: a 45° slanted edge sweeps from the top-left corner to \
         the bottom-right, revealing the screen along the anti-diagonal axis"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // The wipe front is the line: x + y = threshold.
        // At progress=0 threshold=0 (nothing shown); at 1 threshold=dw+dh-2 (all shown).
        let total = (dw + dh) as f32;
        let threshold = ctx.eased * total;
        for dy in 0..dh {
            for dx in 0..dw {
                if (dx as f32 + dy as f32) < threshold {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint along the diagonal axis.
        let (cw, ch) = grid.dimensions();
        let total_c = (cw + ch) as f32;
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let diag = (cx_idx + cy_idx) as f32;
                if diag / total_c <= ctx.eased {
                    let t = (diag / total_c).clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Checkerboard: cells fill in checkerboard order
// ─────────────────────────────────────────────────────────────────────────────

struct Checkerboard;
impl ProgressStyle for Checkerboard {
    fn name(&self) -> &str {
        "checkerboard"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Checkerboard wipe: the screen is split into an alternating tile grid; \
         even tiles fill first and odd tiles follow, creating a two-pass \
         chequerboard dissolve as progress crosses 0.5"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // Tile size: 4 dots wide × 4 dots tall (a 2×1 cell block — distinct from
        // the Bayer-dither which operates per dot with a 4×4 matrix).
        let tile_w = 4usize.max(1);
        let tile_h = 4usize.max(1);
        // Each tile has a phase in [0, 1): even tiles have phase 0, odd 0.5.
        // A tile is revealed when ctx.eased > phase.
        for ty in 0.. {
            let y0 = ty * tile_h;
            if y0 >= dh {
                break;
            }
            for tx in 0.. {
                let x0 = tx * tile_w;
                if x0 >= dw {
                    break;
                }
                let phase = if (tx + ty) % 2 == 0 { 0.0f32 } else { 0.5 };
                if ctx.eased > phase {
                    let tw = tile_w.min(dw.saturating_sub(x0));
                    let th = tile_h.min(dh.saturating_sub(y0));
                    draw::fill_rect(grid, x0, y0, tw, th);
                }
            }
        }
        // Tint by tile parity.
        let (cw, ch) = grid.dimensions();
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let tx = cx_idx / 2;
                let ty = cy_idx;
                let phase = if (tx + ty) % 2 == 0 { 0.0f32 } else { 0.5 };
                if ctx.eased > phase {
                    let t = if (tx + ty) % 2 == 0 { 0.3 } else { 0.7 };
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Venetian blinds: horizontal slats grow from their midpoints
// ─────────────────────────────────────────────────────────────────────────────

struct VenetianBlinds;
impl ProgressStyle for VenetianBlinds {
    fn name(&self) -> &str {
        "venetian-blinds"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Venetian-blinds wipe: the screen is divided into horizontal slats; each \
         slat opens symmetrically from its own centreline, all in unison, so the \
         full height is revealed in parallel stripes"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // Slat height in dots — 8 dots = 2 cell rows.
        let slat_h = 8usize.max(1);
        let num_slats = (dh + slat_h - 1) / slat_h;
        for s in 0..num_slats {
            let top = s * slat_h;
            let slat_actual = slat_h.min(dh.saturating_sub(top));
            let open_half = (ctx.eased * slat_actual as f32 / 2.0).round() as usize;
            let mid = top + slat_actual / 2;
            let y0 = mid.saturating_sub(open_half);
            let y1 = (mid + open_half).min(top + slat_actual);
            if y1 > y0 {
                draw::fill_rect(grid, 0, y0, dw, y1 - y0);
            }
        }
        // Tint alternating slats.
        let (cw, ch) = grid.dimensions();
        let slat_c = 2usize.max(1);
        for cy_idx in 0..ch {
            let slat_idx = cy_idx / slat_c;
            let t = if slat_idx % 2 == 0 { 0.25 } else { 0.75 };
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Dissolve: ordered Bayer-dither per-pixel threshold
// ─────────────────────────────────────────────────────────────────────────────

// 4×4 Bayer matrix values in [0, 15] mapped to [0, 1).
const BAYER: [[f32; 4]; 4] = [
    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
];

struct Dissolve;
impl ProgressStyle for Dissolve {
    fn name(&self) -> &str {
        "dissolve"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Ordered Bayer-dither dissolve: each dot has a threshold from a 4×4 \
         matrix; it lights up when progress exceeds its threshold, producing a \
         structured stipple that expands uniformly across the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        for dy in 0..dh {
            for dx in 0..dw {
                let bx = dx % 4;
                let by = dy % 4;
                if BAYER[by][bx] < ctx.eased {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint with a uniform midpoint colour — the dither pattern provides
        // the structural variety; a flat colour avoids misleading gradients.
        let (cw, ch) = grid.dimensions();
        let color = ctx.palette.sample(0.5);
        for cy_idx in 0..ch {
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Clock-wipe: radial pie-slice sweep 0 → 2π
// ─────────────────────────────────────────────────────────────────────────────

struct ClockWipe;
impl ProgressStyle for ClockWipe {
    fn name(&self) -> &str {
        "clock-wipe"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Clock wipe: a radial pie slice rotates clockwise from 12 o'clock, \
         sweeping the filled region around the centre like a clock hand until \
         the full circle is revealed at progress 1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // Sweep angle: 0 → 2π, starting from -π/2 (12 o'clock), clockwise.
        let sweep = ctx.eased * 2.0 * PI;
        let start = -PI / 2.0; // 12 o'clock in standard coords
        for dy in 0..dh {
            let vy = dy as f32 + 0.5 - cy;
            for dx in 0..dw {
                let vx = dx as f32 + 0.5 - cx;
                // atan2 in standard coords; convert to clockwise angle from 12.
                let angle = vx.atan2(-vy); // atan2(x, -y) gives CW from 12
                                           // Normalise to [0, 2π).
                let norm_angle = if angle < start {
                    angle - start + 2.0 * PI
                } else {
                    angle - start
                };
                if norm_angle < sweep {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint by angular position.
        let (cw, ch) = grid.dimensions();
        let ccx = cw as f32 / 2.0;
        let ccy = ch as f32 / 2.0;
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = cx_idx as f32 + 0.5 - ccx;
                let vy = cy_idx as f32 + 0.5 - ccy;
                let angle = vx.atan2(-vy);
                let norm_angle = if angle < start {
                    angle - start + 2.0 * PI
                } else {
                    angle - start
                };
                if norm_angle < sweep {
                    let t = (norm_angle / (2.0 * PI)).clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Spiral: Archimedean spiral mask filling outward
// ─────────────────────────────────────────────────────────────────────────────

struct Spiral;
impl ProgressStyle for Spiral {
    fn name(&self) -> &str {
        "spiral"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Archimedean spiral wipe: each dot is revealed according to its spiral \
         parameter θ = √(r/r_max)·N_turns·2π, so the filled region uncoils \
         outward from the centre in a continuous tight helix"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let max_r = ((cx * cx + cy * cy) as f32).sqrt().max(1.0);
        // Number of spiral arms / turns.
        let n_turns = 3.0f32;
        let two_pi = 2.0 * PI;
        // For each dot compute its canonical spiral phase in [0, 1).
        // Phase = (angle_from_12_normalised + radial_fraction * n_turns) / n_turns,
        // clamped to [0, 1].  A dot is revealed when phase ≤ eased.
        for dy in 0..dh {
            let vy = dy as f32 + 0.5 - cy;
            for dx in 0..dw {
                let vx = dx as f32 + 0.5 - cx;
                let r = (vx * vx + vy * vy).sqrt();
                let r_frac = (r / max_r).clamp(0.0, 1.0);
                // Angle clockwise from 12 o'clock, in [0, 2π).
                let raw_angle = vx.atan2(-vy); // CW from 12
                let angle = if raw_angle < 0.0 {
                    raw_angle + two_pi
                } else {
                    raw_angle
                };
                // Combine angle and radius into a spiral parameter.
                let phase = (angle / two_pi + r_frac * n_turns) / n_turns;
                if phase <= ctx.eased {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint by radial fraction.
        let (cw, ch) = grid.dimensions();
        let ccx = cw as f32 / 2.0;
        let ccy = ch as f32 / 2.0;
        let max_rc = ((ccx * ccx + ccy * ccy) as f32).sqrt().max(1.0);
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = cx_idx as f32 + 0.5 - ccx;
                let vy = cy_idx as f32 + 0.5 - ccy;
                let r = (vx * vx + vy * vy).sqrt();
                let r_frac = r / max_rc;
                let raw_angle = vx.atan2(-vy);
                let angle = if raw_angle < 0.0 {
                    raw_angle + two_pi
                } else {
                    raw_angle
                };
                let phase = (angle / two_pi + r_frac * n_turns) / n_turns;
                if phase <= ctx.eased {
                    let t = r_frac.clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Split: top and bottom edges advance inward to the midline
// ─────────────────────────────────────────────────────────────────────────────

struct Split;
impl ProgressStyle for Split {
    fn name(&self) -> &str {
        "split"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Split wipe: two horizontal bands advance from the top edge and bottom \
         edge simultaneously, meeting at the vertical midline when progress = 1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let half_h = dh / 2;
        let reach = (ctx.eased * half_h as f32).round() as usize;
        // Top band: grows downward from row 0.
        draw::fill_rect(grid, 0, 0, dw, reach.min(dh));
        // Bottom band: grows upward from row dh-1.
        let bot_start = dh.saturating_sub(reach);
        if bot_start < dh {
            draw::fill_rect(grid, 0, bot_start, dw, dh - bot_start);
        }
        // Tint top half with palette start, bottom with palette end.
        let (cw, ch) = grid.dimensions();
        let half_c = ch / 2;
        let reach_c = (ctx.eased * half_c as f32).round() as usize;
        for cy_idx in 0..reach_c.min(ch) {
            let color = ctx.palette.sample(0.15);
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        let bot_start_c = ch.saturating_sub(reach_c);
        for cy_idx in bot_start_c..ch {
            let color = ctx.palette.sample(0.85);
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 13. Zigzag: wipe with a sine-wave leading edge
// ─────────────────────────────────────────────────────────────────────────────

struct Zigzag;
impl ProgressStyle for Zigzag {
    fn name(&self) -> &str {
        "zigzag"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Zigzag wipe: a left-to-right curtain whose leading edge is a sine wave, \
         producing a rippling serrated boundary that sweeps across the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // The mean position of the wipe front.
        let mean_x = ctx.eased * dw as f32;
        // Amplitude of the sine wave on the leading edge (in dots).
        let amp = (dh as f32 * 0.35).max(1.0);
        // Frequency: one full wave per screen height.
        let freq = 2.0 * PI / dh.max(1) as f32;
        // Phase shift driven by time for animated ripple.
        let phase = ctx.time * 1.2;
        for dy in 0..dh {
            // The wipe boundary x for this row.
            let boundary = mean_x + amp * (freq * dy as f32 + phase).sin();
            let col_x = boundary.round() as i32;
            // Fill from 0 to col_x (clamped).
            let fill_end = col_x.max(0) as usize;
            for dx in 0..fill_end.min(dw) {
                draw::dot(grid, dx, dy);
            }
        }
        // Tint column-by-column.
        let (cw, ch) = grid.dimensions();
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let t = if cw <= 1 {
                    0.5
                } else {
                    cx_idx as f32 / (cw - 1) as f32
                };
                let color = ctx.palette.sample(t);
                // Only tint cells that are within the zigzag boundary.
                let dy_mid = cy_idx * 4 + 2;
                let boundary = mean_x + amp * (freq * dy_mid as f32 + phase).sin();
                if (cx_idx * 2) as f32 + 1.0 < boundary {
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 14. Pixelate: coarse block fill with increasing shade density
// ─────────────────────────────────────────────────────────────────────────────

struct Pixelate;
impl ProgressStyle for Pixelate {
    fn name(&self) -> &str {
        "pixelate"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Pixelate wipe: the screen is divided into coarse 2×1-cell blocks, each \
         ramp from empty → ░ → ▒ → ▓ → █ as progress rises through four density \
         thresholds — a block-density mosaic that fills by region, not by edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        // Block size in cells (2 wide × 1 tall — avoids conflating with Bayer).
        let block_w = 2usize.max(1);
        let block_h = 1usize.max(1);
        // Each block has a pseudorandom order based on its grid position so that
        // blocks light up in a spatial-hash order rather than simple scan order.
        // We use a cheap integer hash of (block_col, block_row).
        let num_bx = (cw + block_w - 1) / block_w;
        let num_by = (ch + block_h - 1) / block_h;
        let total_blocks = (num_bx * num_by).max(1);
        for bby in 0..num_by {
            for bbx in 0..num_bx {
                // Hash to a threshold in [0, 1).
                let hash = hash2(bbx as u32, bby as u32);
                let threshold = hash as f32 / u32::MAX as f32;
                // Local progress relative to this block's threshold.
                // Shade level 0..=4 based on how far past the threshold we are.
                let local = if ctx.eased <= threshold {
                    0.0
                } else {
                    ((ctx.eased - threshold) / (1.0 - threshold + 1e-6)).clamp(0.0, 1.0)
                };
                // Map to 0..=4 shade levels from SHADES.
                let level = (local * 4.0).floor() as usize;
                // Block cell coordinates.
                let cx0 = bbx * block_w;
                let cy0 = bby * block_h;
                // Each block covers block_w × block_h cells.
                for dy in 0..block_h {
                    for dx in 0..block_w {
                        let cell_x = cx0 + dx;
                        let cell_y = cy0 + dy;
                        if cell_x < cw && cell_y < ch {
                            draw::shade(grid, cell_x, cell_y, level);
                        }
                    }
                }
                // Tint filled blocks.
                if level > 0 {
                    let t = threshold;
                    let color = ctx.palette.sample(t);
                    for dy in 0..block_h {
                        let cell_y = cy0 + dy;
                        if cell_y < ch {
                            let cx1 = (cx0 + block_w - 1).min(cw.saturating_sub(1));
                            draw::tint_row(grid, cell_y, cx0.min(cw.saturating_sub(1)), cx1, color);
                        }
                    }
                }
                let _ = total_blocks; // suppress unused warning
            }
        }
        Ok(())
    }
}

/// Cheap integer hash of two u32 values → u32, used for block ordering.
/// Based on the Wang hash.
#[inline]
fn hash2(x: u32, y: u32) -> u32 {
    let mut h = x
        .wrapping_mul(2654435761)
        .wrapping_add(y.wrapping_mul(2246822519));
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::wipe::styles();
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
