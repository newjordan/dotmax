//! `blocks` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O blocks.rs && ./blocks [style-name]
//! ```

const DEFAULT_STYLE: &str = "smooth-hbar";

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
    pub mod blocks {
//! Unicode block-element progress bars — the **blocky ↔ smooth** spectrum.
//!
//! Every style in this theme showcases a distinct structural form built from
//! Unicode block, shade, or box-drawing glyphs. Color is secondary; the
//! *shape* is the point. Styles span the full contrast axis from
//! `draw::hbar`'s eighth-precise smoothness down to coarse single-cell
//! snap, with dithering, stacking, masonry, and spectrum animation in between.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — cool block blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(96, 202, 255);
const TINT_END: Color = Color::rgb(52, 88, 222);

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

/// All styles in the `blocks` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per structural form, covering the
/// full range of block-element technique: smooth sub-cell hbar, coarse snap,
/// segmented LED, equalizer columns, thermometer, shade dither, brick masonry,
/// Bayer ordered dither, stacked rows, waterfall, double-ended, and a shade
/// back-fill gradient meter.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(SmoothHbar)),
        Box::new(Tinted(BlockyHbar)),
        Box::new(Tinted(SegmentedLed)),
        Box::new(Tinted(Equalizer)),
        Box::new(Tinted(Thermometer)),
        Box::new(Tinted(DitherRamp)),
        Box::new(Tinted(BrickWall)),
        Box::new(Tinted(BayerDither)),
        Box::new(Tinted(StackedBars)),
        Box::new(Tinted(Waterfall)),
        Box::new(Tinted(DoubleEnded)),
        Box::new(Tinted(GradientMeter)),
    ]
}

// ---------------------------------------------------------------------------
// 1. smooth-hbar — eighth-precise sub-character bar via draw::hbar
// ---------------------------------------------------------------------------

struct SmoothHbar;
impl ProgressStyle for SmoothHbar {
    fn name(&self) -> &str {
        "smooth-hbar"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Eighth-precise sub-cell smooth bar — the gold standard crisp fill using draw::hbar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (_, cells_h) = grid.dimensions();
        // Fill every row with the same smooth bar so tall grids are solid.
        for cy in 0..cells_h {
            draw::hbar(grid, cy, ctx.eased);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. blocky-hbar — same fill SNAPPED to whole █ cells only (coarse)
// ---------------------------------------------------------------------------

struct BlockyHbar;
impl ProgressStyle for BlockyHbar {
    fn name(&self) -> &str {
        "blocky-hbar"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Full-cell █ snap only — explicit coarse counterpart to smooth-hbar, no sub-cell precision"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Snap to whole cells — no partial edge glyph.
        let filled = (ctx.eased * cells_w as f32).floor() as usize;
        let filled = filled.min(cells_w);
        for cy in 0..cells_h {
            for cx in 0..filled {
                draw::glyph(grid, cx, cy, '█');
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. segmented-led — lit █ cells with one-cell gaps between segments
// ---------------------------------------------------------------------------

struct SegmentedLed;
impl ProgressStyle for SegmentedLed {
    fn name(&self) -> &str {
        "segmented-led"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Discrete lit █ segments separated by single-cell gaps, like a VU meter"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Each segment: 2 wide, 1 gap. Minimum segment width = 1 with no gap if tiny.
        let seg_w = 2usize;
        let gap_w = 1usize;
        let stride = (seg_w + gap_w).max(1);
        let n_segs = (cells_w / stride).max(1);
        let lit = (ctx.eased * n_segs as f32).round() as usize;
        let lit = lit.min(n_segs);
        for s in 0..lit {
            let x0 = s * stride;
            for cx in x0..(x0 + seg_w).min(cells_w) {
                for cy in 0..cells_h {
                    draw::glyph(grid, cx, cy, '█');
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. equalizer — animated spectrum columns of vblock heights
// ---------------------------------------------------------------------------

struct Equalizer;
impl ProgressStyle for Equalizer {
    fn name(&self) -> &str {
        "equalizer"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Animated spectrum equalizer: vblock columns with sinusoidal heights, lit count gated by eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        let lit_cols = (ctx.eased * cells_w as f32).round() as usize;
        let lit_cols = lit_cols.min(cells_w);
        let cells_h_f = cells_h as f32;

        for cx in 0..lit_cols {
            // Synthetic frequency per column, animated by time.
            let freq = 1.0 + (cx as f32 / cells_w.max(1) as f32) * 3.0;
            let phase = ctx.time * 2.5 + cx as f32 * 0.4;
            let raw = (phase * freq).sin() * 0.5 + 0.5; // 0..1
            let col_height_f = raw * cells_h_f;
            let full_cells = col_height_f.floor() as usize;
            let frac_eighth = ((col_height_f.fract() * 8.0).round() as usize).min(8);

            // Draw from the bottom up.
            for row in 0..full_cells.min(cells_h) {
                let cy = cells_h.saturating_sub(1).saturating_sub(row);
                draw::vblock(grid, cx, cy, 8);
            }
            // Partial top cell.
            if full_cells < cells_h && frac_eighth > 0 {
                let cy = cells_h.saturating_sub(1).saturating_sub(full_cells);
                draw::vblock(grid, cx, cy, frac_eighth);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. thermometer — single column filling bottom-to-top via vblock
// ---------------------------------------------------------------------------

struct Thermometer;
impl ProgressStyle for Thermometer {
    fn name(&self) -> &str {
        "thermometer"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Vertical column filling bottom-to-top with vblock eighths, like a mercury thermometer"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }
        // Centre column (or all columns for wider grids, each a replica).
        let col_x = cells_w / 2;
        let total_eighths = (ctx.eased * cells_h as f32 * 8.0).round() as usize;
        let full_cells = (total_eighths / 8).min(cells_h);
        let rem_eighths = total_eighths % 8;

        // Full cells from bottom.
        for row in 0..full_cells {
            let cy = cells_h.saturating_sub(1).saturating_sub(row);
            draw::vblock(grid, col_x, cy, 8);
        }
        // Partial top cell.
        if full_cells < cells_h && rem_eighths > 0 {
            let cy = cells_h.saturating_sub(1).saturating_sub(full_cells);
            draw::vblock(grid, col_x, cy, rem_eighths);
        }

        // For wider grids add a second, mirrored column for symmetry.
        if cells_w >= 3 {
            let col_x2 = cells_w - 1 - col_x;
            if col_x2 != col_x {
                for row in 0..full_cells {
                    let cy = cells_h.saturating_sub(1).saturating_sub(row);
                    draw::vblock(grid, col_x2, cy, 8);
                }
                if full_cells < cells_h && rem_eighths > 0 {
                    let cy = cells_h.saturating_sub(1).saturating_sub(full_cells);
                    draw::vblock(grid, col_x2, cy, rem_eighths);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. dither-ramp — left-to-right SHADES gradient advancing with eased
// ---------------------------------------------------------------------------

struct DitherRamp;
impl ProgressStyle for DitherRamp {
    fn name(&self) -> &str {
        "dither-ramp"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Left-to-right shade ramp ░▒▓█ whose density front advances with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // The fill frontier in fractional cells.
        let frontier = ctx.eased * cells_w as f32;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let cell_f = cx as f32;
                // Distance behind the frontier determines density.
                let behind = frontier - cell_f;
                let level = if behind <= 0.0 {
                    0 // unfilled
                } else if behind < 1.0 {
                    // Transitional cell: interpolate shade level.
                    (behind * 4.0).ceil() as usize
                } else {
                    // Fully behind frontier.
                    let t = (cell_f / frontier.max(1.0)).clamp(0.0, 1.0);
                    // Gradient: cells near the start are lighter.
                    (t * 4.0).round() as usize
                };
                draw::shade(grid, cx, cy, level.min(4));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. brick-wall — masonry texture (offset rows) filling with eased
// ---------------------------------------------------------------------------

struct BrickWall;
impl ProgressStyle for BrickWall {
    fn name(&self) -> &str {
        "brick-wall"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Running-bond brick masonry pattern (alternating offset rows of █) filling with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Each brick: 3 wide, mortar gap is the blank between bricks.
        let brick_w: usize = 3;
        let frontier = (ctx.eased * cells_w as f32).round() as usize;
        let frontier = frontier.min(cells_w);

        for cy in 0..cells_h {
            // Alternate rows offset by half a brick.
            let offset = if cy % 2 == 1 { brick_w / 2 } else { 0 };
            let mut cx = 0usize;
            while cx < frontier {
                // Shift start by row offset, wrapping within the brick cycle.
                let brick_start = if cx == 0 && offset > 0 {
                    // First partial brick at left edge.
                    0
                } else {
                    cx
                };
                // Place solid block glyphs for `brick_w - 1` cells (1 mortar gap).
                let body = (brick_w.saturating_sub(1)).max(1);
                for bx in 0..body {
                    let target = brick_start + bx;
                    if target < frontier && target < cells_w {
                        // Mortar gap every `brick_w` cell boundary (relative to offset).
                        let col_in_cycle = (target + cells_w - offset) % brick_w;
                        if col_in_cycle < brick_w.saturating_sub(1) {
                            draw::glyph(grid, target, cy, '█');
                        }
                    }
                }
                cx += brick_w;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. bayer-dither — 4×4 ordered Bayer matrix reveal
// ---------------------------------------------------------------------------

struct BayerDither;

/// 4×4 Bayer matrix values in 0..16.
const BAYER4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

impl ProgressStyle for BayerDither {
    fn name(&self) -> &str {
        "bayer-dither"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "4×4 Bayer ordered-dither reveal — threshold rises with eased, producing a stippled fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Threshold rises 0..=16 with progress.
        let threshold = (ctx.eased * 16.0).round() as u8;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let bx = cx % 4;
                let by = cy % 4;
                let bval = BAYER4[by][bx];
                if bval < threshold {
                    draw::glyph(grid, cx, cy, '█');
                } else if bval == threshold && threshold > 0 {
                    draw::shade(grid, cx, cy, 2); // ▒ at the transition band
                }
                // else: leave blank (space)
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. stacked-bars — multi-row stacked bar chart filling
// ---------------------------------------------------------------------------

struct StackedBars;
impl ProgressStyle for StackedBars {
    fn name(&self) -> &str {
        "stacked-bars"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Multi-row stacked bar chart: each row a differently-shaded series filling with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // We use shade levels cycling per row to give each series a distinct density.
        // Series allocation: each row gets a fraction of the total progress.
        let n_series = cells_h.max(1);
        for cy in 0..cells_h {
            // Each series has slightly different progress fractions (staggered).
            let series_offset = (cy as f32 / n_series as f32) * 0.2;
            let series_frac = (ctx.eased - series_offset).clamp(0.0, 1.0);
            let filled = (series_frac * cells_w as f32).round() as usize;
            let filled = filled.min(cells_w);
            // Shade level cycles across rows: row 0 = █, row 1 = ▓, row 2 = ▒, …
            let shade_level = 4 - (cy % 4);
            for cx in 0..filled {
                draw::shade(grid, cx, cy, shade_level);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. waterfall — shade rows scrolling downward, intensity from a wave
// ---------------------------------------------------------------------------

struct Waterfall;
impl ProgressStyle for Waterfall {
    fn name(&self) -> &str {
        "waterfall"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Shade glyphs cascading downward (driven by time) with intensity from a horizontal wave"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        let frontier = ctx.eased * cells_w as f32;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                // Only draw within the progress frontier.
                if cx as f32 >= frontier {
                    continue;
                }
                // Wave: horizontal sine shifted by time and vertical position.
                let wave = ((cx as f32 * 0.5 - ctx.time * 3.0 + cy as f32 * 0.8) * PI * 0.5).sin();
                let intensity = (wave * 0.5 + 0.5).clamp(0.0, 1.0);
                let level = (intensity * 4.0).round() as usize;
                draw::shade(grid, cx, cy, level.min(4));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. double-ended — fills from both ends toward the centre
// ---------------------------------------------------------------------------

struct DoubleEnded;
impl ProgressStyle for DoubleEnded {
    fn name(&self) -> &str {
        "double-ended"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Bar fills simultaneously from both ends toward the centre using smooth hbar logic"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }
        // Each side fills half the width.
        let half = cells_w as f32 / 2.0;
        // Eighths of total half-width filled.
        let eighths_total = (ctx.eased * half * 8.0).round() as usize;
        let full_cells = (eighths_total / 8).min(cells_w / 2);
        let rem_eighths = eighths_total % 8;

        for cy in 0..cells_h {
            // Left side: fill from left edge inward.
            for cx in 0..full_cells.min(cells_w) {
                draw::glyph(grid, cx, cy, '█');
            }
            // Left partial edge cell.
            if rem_eighths > 0 && full_cells < cells_w {
                draw::glyph(grid, full_cells, cy, draw::H_BLOCKS[rem_eighths]);
            }

            // Right side: fill from right edge inward (mirror).
            let r_start = cells_w.saturating_sub(full_cells);
            for cx in r_start..cells_w {
                draw::glyph(grid, cx, cy, '█');
            }
            // Right partial edge cell (mirrored — use reverse H_BLOCK or full block).
            if rem_eighths > 0 && r_start > 0 {
                // Mirror: the partial cell is at r_start - 1.
                let pcx = r_start.saturating_sub(1);
                // Ensure no overlap with left fill.
                if pcx > full_cells {
                    draw::glyph(grid, pcx, cy, draw::H_BLOCKS[rem_eighths]);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. gradient-meter — smooth hbar over a shade backdrop for the track
// ---------------------------------------------------------------------------

struct GradientMeter;
impl ProgressStyle for GradientMeter {
    fn name(&self) -> &str {
        "gradient-meter"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Smooth hbar fill combined with a ░ shade backdrop on the unfilled track"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // First, lay down the track shade in every cell.
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                draw::shade(grid, cx, cy, 1); // ░ track
            }
        }
        // Then overwrite the filled portion with the smooth hbar (all rows).
        // We replicate hbar manually so it overwrites shade with full/partial blocks.
        let eighths_total = (ctx.eased * cells_w as f32 * 8.0).round() as usize;
        let full_cells = (eighths_total / 8).min(cells_w);
        let rem_eighths = eighths_total % 8;
        for cy in 0..cells_h {
            for cx in 0..full_cells {
                draw::glyph(grid, cx, cy, '█');
            }
            if rem_eighths > 0 && full_cells < cells_w {
                draw::glyph(grid, full_cells, cy, draw::H_BLOCKS[rem_eighths]);
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
    let styles = progress::styles::blocks::styles();
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
