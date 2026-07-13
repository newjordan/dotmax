//! `lasers` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O lasers.rs && ./lasers [style-name]
//! ```

const DEFAULT_STYLE: &str = "charge-and-fire";

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
    pub mod lasers {
//! Laser / light-beam progress bars — eleven structurally distinct beam styles.
//!
//! Every style is stateless: all animation comes from `ctx.time` and all
//! charge / fill comes from `ctx.eased`. Lines are the medium — beams, sweeps,
//! fans, grids, and reflections. Color tints are additive; structure drives the
//! variety.
//!
//! Styles in this module:
//! 1. `charge-and-fire`     — core charges, then a beam lances the full width
//! 2. `scanning-line`       — a vertical sweep beam travels back and forth
//! 3. `security-grid`       — criss-crossing beams; tripped beams flicker
//! 4. `prism-dispersion`    — one beam fans into a spectrum of angled lines
//! 5. `laser-light-show`    — Lissajous beams crossing, time-animated
//! 6. `range-finder`        — rotating sweep with a target lock at eased position
//! 7. `mirror-bounce`       — beam reflects off walls; path length = eased
//! 8. `fiber-pulse`         — light pulses travel along curved fiber lines
//! 9. `plasma-bolt`         — jagged lightning-like beam jittered by time
//! 10. `particle-accelerator` — dots racing along a track, speed = eased
//! 11. `disco-fan`          — radial beams sweeping from a corner origin

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ─── deterministic hash helpers ─────────────────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

// ─── step-bounded Bresenham line ─────────────────────────────────────────────

/// Draw a straight line from `(x0,y0)` to `(x1,y1)` in dot-space using
/// integer Bresenham. At most `max_steps` pixels are emitted so that a 1×1
/// grid can never loop a million times. `draw::dot_i` ignores OOB writes.
fn beam_line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, max_steps: usize) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x0;
    let mut cy = y0;
    let steps = (dx + dy + 2) as usize;
    let limit = steps.min(max_steps);
    for _ in 0..limit {
        draw::dot_i(grid, cx, cy);
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

// ─── public registry ─────────────────────────────────────────────────────────

/// All styles in the `lasers` theme.
///
/// Returns eleven laser / light-beam bars, each with a structurally distinct
/// beam topology. Progress (`ctx.eased`) drives charge, fill, or target
/// position; time (`ctx.time`) drives sweep, animation, and flicker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(ChargeAndFire),
        Box::new(ScanningLine),
        Box::new(SecurityGrid),
        Box::new(PrismDispersion),
        Box::new(LaserLightShow),
        Box::new(RangeFinder),
        Box::new(MirrorBounce),
        Box::new(FiberPulse),
        Box::new(PlasmaBolt),
        Box::new(ParticleAccelerator),
        Box::new(DiscoFan),
    ]
}

// ─── 1. Charge and fire ──────────────────────────────────────────────────────

/// Core charges visibly as eased grows; at 100% a full-width beam fires.
struct ChargeAndFire;
impl ProgressStyle for ChargeAndFire {
    fn name(&self) -> &str {
        "charge-and-fire"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Core charges with eased; at full power a beam lances across the entire bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = h / 2;
        let core_x = w / 6;

        // Phase: 0..0.85 = charging, 0.85..1.0 = fired
        let fired = ctx.eased >= 0.85;
        let charge = (ctx.eased / 0.85).clamp(0.0, 1.0);

        if fired {
            // Full horizontal beam across entire bar.
            for y in mid.saturating_sub(1)..=(mid + 1).min(h.saturating_sub(1)) {
                draw::hline(grid, 0, w.saturating_sub(1), y);
            }
            // Bright centre stripe.
            draw::hline(grid, 0, w.saturating_sub(1), mid);

            // Tint: intense palette end color across all cells.
            let color = ctx.palette.sample(1.0);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
            }
        } else {
            // Charging core: concentric rings growing outward from core_x.
            let rings = ((charge * 6.0) as usize).max(1).min(6);
            for r in 0..rings {
                let radius = r + 1;
                // Horizontal arms.
                for dr in 0..radius {
                    draw::dot_i(grid, core_x as i32 + dr as i32, mid as i32);
                    if core_x >= dr {
                        draw::dot_i(grid, core_x as i32 - dr as i32, mid as i32);
                    }
                }
                // Vertical arms.
                for dv in 0..radius {
                    draw::dot_i(grid, core_x as i32, mid as i32 + dv as i32);
                    draw::dot_i(grid, core_x as i32, mid as i32 - dv as i32);
                }
            }

            // Pulsing ring: flicker via time.
            let pulse_r = (charge * 4.0) as i32;
            if pulse_r > 0 {
                let steps = 32usize;
                for s in 0..steps {
                    let angle = s as f32 / steps as f32 * 2.0 * PI + ctx.time * 4.0;
                    // Squish vertically (braille dots are taller than wide).
                    let px = core_x as i32 + (angle.cos() * pulse_r as f32 * 1.5) as i32;
                    let py = mid as i32 + (angle.sin() * pulse_r as f32 * 0.7) as i32;
                    draw::dot_i(grid, px, py);
                }
            }

            // Charge beam lead: partial horizontal line toward the right.
            let beam_reach = ((charge * w as f32) as usize).min(w.saturating_sub(1));
            if beam_reach > core_x {
                draw::hline(grid, core_x, beam_reach, mid);
            }

            // Tint proportional to charge.
            let filled_cells = (charge * cells_w as f32) as usize;
            for cx in 0..filled_cells.min(cells_w) {
                let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
                let color = ctx.palette.sample(t);
                for cy in 0..cells_h {
                    draw::tint_row(grid, cy, cx, cx, color);
                }
            }
        }

        Ok(())
    }
}

// ─── 2. Scanning line ────────────────────────────────────────────────────────

/// A bright vertical beam sweeps left↔right; the swept region fills with eased.
struct ScanningLine;
impl ProgressStyle for ScanningLine {
    fn name(&self) -> &str {
        "scanning-line"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Vertical beam sweeps back and forth; swept fraction fills to eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Sweep: triangle wave in time.
        let period = 3.0f32;
        let t_norm = (ctx.time % period) / period; // 0..1
        let sweep_frac = if t_norm < 0.5 {
            t_norm * 2.0
        } else {
            2.0 - t_norm * 2.0
        };
        let sweep_x = (sweep_frac * w as f32) as usize;
        let sweep_x = sweep_x.min(w.saturating_sub(1));

        // Filled region up to eased progress: sparse horizontal lines.
        let filled = ((ctx.eased * w as f32) as usize).min(w);
        let mid = h / 2;
        // Three horizontal rails.
        draw::hline(
            grid,
            0,
            filled.saturating_sub(1),
            mid.saturating_sub(1).min(h - 1),
        );
        draw::hline(grid, 0, filled.saturating_sub(1), mid);
        draw::hline(
            grid,
            0,
            filled.saturating_sub(1),
            (mid + 1).min(h.saturating_sub(1)),
        );

        // Sweep beam: full-height vertical line with a narrow bright core.
        draw::vline(grid, sweep_x, 0, h.saturating_sub(1));
        // One-dot wings on each side for width.
        if sweep_x > 0 {
            draw::vline(grid, sweep_x - 1, h / 4, h * 3 / 4);
        }
        if sweep_x + 1 < w {
            draw::vline(grid, sweep_x + 1, h / 4, h * 3 / 4);
        }

        // Tint: filled region gets gradient; beam cell gets bright end.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        let beam_cell = (sweep_x / 2).min(cells_w.saturating_sub(1));
        let bright = ctx.palette.sample(1.0);
        for cy in 0..cells_h {
            draw::tint_row(grid, cy, beam_cell, beam_cell, bright);
        }

        Ok(())
    }
}

// ─── 3. Security grid ────────────────────────────────────────────────────────

/// Horizontal + vertical beams form a grid; beams tripped by the sweep flicker.
struct SecurityGrid;
impl ProgressStyle for SecurityGrid {
    fn name(&self) -> &str {
        "security-grid"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Criss-crossing security beams; tripped beams flicker dangerously via time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Grid spacing: number of beams scales with eased.
        let h_beams = ((ctx.eased * 5.0 + 1.0) as usize).min(8).max(1);
        let v_beams = ((ctx.eased * 3.0 + 1.0) as usize).min(6).max(1);

        // Horizontal beams.
        for i in 0..h_beams {
            let y = if h_beams <= 1 {
                h / 2
            } else {
                i * h.saturating_sub(1) / (h_beams - 1)
            };
            // Trip: a "tripwire" sweeper crosses at ctx.time.
            let trip_x = ((ctx.time * 0.4).fract() * w as f32) as usize;
            let tripped = trip_x < w / 2; // left half crossed = tripped
            let flicker_on = (ctx.time * 12.0 + i as f32 * 1.7).sin() > 0.0;

            if tripped && flicker_on {
                // Flicker: draw the beam in two broken halves.
                if trip_x > 0 {
                    draw::hline(grid, 0, trip_x.saturating_sub(1), y);
                }
                if trip_x + 2 < w {
                    draw::hline(grid, trip_x + 2, w.saturating_sub(1), y);
                }
            } else {
                draw::hline(grid, 0, w.saturating_sub(1), y);
            }
        }

        // Vertical beams.
        for j in 0..v_beams {
            let x = if v_beams <= 1 {
                w / 2
            } else {
                j * w.saturating_sub(1) / (v_beams - 1)
            };
            let on = (ctx.time * 8.0 + j as f32 * 2.3).cos() > -0.3;
            if on {
                draw::vline(grid, x, 0, h.saturating_sub(1));
            }
        }

        // Tint: map each cell column to palette gradient.
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ─── 4. Prism dispersion ────────────────────────────────────────────────────

/// One incoming beam from the left fans into multiple angled beams via a prism.
struct PrismDispersion;
impl ProgressStyle for PrismDispersion {
    fn name(&self) -> &str {
        "prism-dispersion"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "One white beam enters a prism and fans into a spectrum of angled beams"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let prism_x = (w / 4) as i32;
        let mid = (h / 2) as i32;

        // Input beam: horizontal from left to prism.
        beam_line(grid, 0, mid, prism_x, mid, w + h);

        // Prism triangle outline.
        let tri_h = (h / 3).max(1) as i32;
        beam_line(
            grid,
            prism_x,
            mid - tri_h,
            prism_x + tri_h,
            mid + tri_h,
            w + h,
        );
        beam_line(grid, prism_x, mid - tri_h, prism_x, mid + tri_h, w + h);
        beam_line(
            grid,
            prism_x,
            mid + tri_h,
            prism_x + tri_h,
            mid + tri_h,
            w + h,
        );

        // Fanned output beams: spread angle increases with eased.
        let n_beams = ((ctx.eased * 7.0 + 1.0) as usize).max(1).min(8);
        let fan_origin_x = prism_x + tri_h;
        let fan_origin_y = mid;
        let spread = (ctx.eased * PI * 0.7).max(0.05);

        for b in 0..n_beams {
            // Angle: fan from -spread/2 to +spread/2.
            let angle = if n_beams == 1 {
                0.0f32
            } else {
                -spread / 2.0 + b as f32 / (n_beams - 1) as f32 * spread
            };

            // Length of each beam: reaches the right edge.
            let remain_w = (w as i32 - fan_origin_x).max(1);
            let end_x = fan_origin_x + remain_w;
            let end_y = fan_origin_y + (angle.tan() * remain_w as f32) as i32;

            beam_line(grid, fan_origin_x, fan_origin_y, end_x, end_y, (w + h) * 2);

            // Tint each beam with a palette sample.
            let t = b as f32 / n_beams.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            // Approximate the cell columns this beam passes through.
            let cx0 = (fan_origin_x / 2).max(0) as usize;
            let cx1 = (end_x / 2).clamp(0, cells_w as i32 - 1) as usize;
            for cy in 0..cells_h {
                draw::tint_row(
                    grid,
                    cy,
                    cx0.min(cx1),
                    cx0.max(cx1).min(cells_w.saturating_sub(1)),
                    color,
                );
            }
        }

        Ok(())
    }
}

// ─── 5. Laser light show ────────────────────────────────────────────────────

/// Multiple Lissajous beams cross; each traces a 1-D scan at different frequencies.
struct LaserLightShow;
impl ProgressStyle for LaserLightShow {
    fn name(&self) -> &str {
        "laser-light-show"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Multiple Lissajous sweep beams crossing at different frequencies; count = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Number of beams active = eased * max.
        let n_active = ((ctx.eased * 6.0 + 1.0) as usize).min(7);

        // Each beam is a Lissajous-style scan: x scans uniformly, y = A*sin(f*x + phase).
        let freq_pairs: [(f32, f32); 7] = [
            (1.0, 2.0),
            (2.0, 3.0),
            (3.0, 4.0),
            (3.0, 5.0),
            (5.0, 6.0),
            (4.0, 7.0),
            (7.0, 8.0),
        ];

        let amp = (h as f32 / 2.0 - 1.0).max(0.5);
        let cy_mid = h as f32 / 2.0;

        for b in 0..n_active {
            let (fx, fy) = freq_pairs[b % freq_pairs.len()];
            let phase = hashf(b as u32 * 17) * 2.0 * PI + ctx.time * (0.5 + b as f32 * 0.15);

            // Sample beam as a sequence of dots along x.
            let steps = w * 2;
            for s in 0..=steps {
                let frac = s as f32 / steps as f32;
                let px = (frac * w as f32) as i32;
                let theta_x = frac * fx * PI * 2.0;
                let theta_y = frac * fy * PI * 2.0 + phase;
                let py = (cy_mid + amp * theta_y.sin() * (theta_x.cos() * 0.3 + 0.7)) as i32;
                draw::dot_i(grid, px, py);
            }

            // Tint with palette.
            let t = b as f32 / n_active.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
            }
        }

        Ok(())
    }
}

// ─── 6. Range finder ────────────────────────────────────────────────────────

/// A rotating radial sweep beam; target lock reticle at the eased radius.
struct RangeFinder;
impl ProgressStyle for RangeFinder {
    fn name(&self) -> &str {
        "range-finder"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Rotating sweep beam radiates from center; target lock reticle at eased radius"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = (w.min(h * 2) / 2) as f32;

        // Radar-style range rings (3 concentric).
        for ring in 1..=3usize {
            let ring_r = (ring as f32 / 3.0 * max_r) as i32;
            if ring_r == 0 {
                continue;
            }
            let steps = (ring_r * 8).max(16) as usize;
            for s in 0..steps {
                let angle = s as f32 / steps as f32 * 2.0 * PI;
                // Squish vertically: braille cells are ~2× taller than wide.
                let px = cx + (angle.cos() * ring_r as f32) as i32;
                let py = cy + (angle.sin() * ring_r as f32 * 0.5) as i32;
                // Sparse: only every 3rd dot.
                if s % 3 == 0 {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Rotating sweep beam.
        let sweep_angle = ctx.time * 1.8; // ~1 revolution per 3.5 s
        let sweep_dx = sweep_angle.cos();
        let sweep_dy = sweep_angle.sin() * 0.5;
        let beam_end_x = cx + (sweep_dx * max_r) as i32;
        let beam_end_y = cy + (sweep_dy * max_r) as i32;
        beam_line(grid, cx, cy, beam_end_x, beam_end_y, w + h);

        // Ghost trail (slightly behind sweep).
        for ghost in 1..=4usize {
            let ga = sweep_angle - ghost as f32 * 0.12;
            let gx = cx + (ga.cos() * max_r) as i32;
            let gy = cy + (ga.sin() * 0.5 * max_r) as i32;
            // Step only every 2nd pixel.
            let dx = (gx - cx).abs();
            let dy = (gy - cy).abs();
            let steps = dx.max(dy).max(1) as usize;
            for s in (0..steps).step_by(2) {
                let t = s as f32 / steps as f32;
                let px = cx + ((gx - cx) as f32 * t) as i32;
                let py = cy + ((gy - cy) as f32 * t) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Target lock: cross-hair at eased radius along the sweep beam.
        let lock_r = ctx.eased * max_r;
        let lock_x = cx + (sweep_angle.cos() * lock_r) as i32;
        let lock_y = cy + (sweep_angle.sin() * 0.5 * lock_r) as i32;
        // Reticle: 4-dot cross.
        for d in 0..3i32 {
            draw::dot_i(grid, lock_x + d, lock_y);
            draw::dot_i(grid, lock_x - d, lock_y);
            draw::dot_i(grid, lock_x, lock_y + d);
            draw::dot_i(grid, lock_x, lock_y - d);
        }

        // Tint sweep sector with bright color.
        let color = ctx.palette.sample(0.8);
        let dim = ctx.palette.sample(0.2);
        for cx_c in 0..cells_w {
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, dim);
            }
        }
        let beam_cell = (beam_end_x / 2).clamp(0, cells_w as i32 - 1) as usize;
        let center_cell = (cx / 2).clamp(0, cells_w as i32 - 1) as usize;
        for cy_c in 0..cells_h {
            let lo = center_cell.min(beam_cell);
            let hi = center_cell.max(beam_cell);
            draw::tint_row(grid, cy_c, lo, hi, color);
        }

        Ok(())
    }
}

// ─── 7. Mirror bounce ───────────────────────────────────────────────────────

/// A beam enters from the left and bounces off walls; path length = eased.
struct MirrorBounce;
impl ProgressStyle for MirrorBounce {
    fn name(&self) -> &str {
        "mirror-bounce"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Beam enters left and reflects off top/bottom walls; total path length = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Beam starts at left-middle, travels at a diagonal.
        // Angle modulated by time so the bounce pattern shifts.
        let angle_base = PI / 4.0 + (ctx.time * 0.2).sin() * PI / 8.0;
        // Float accumulators: an i32 position re-truncated each step would
        // never accumulate sub-dot motion and the beam would stand still.
        let mut fx = 0.0f32;
        let mut fy = h as f32 / 2.0;
        let dx = angle_base.cos();
        let mut dy = angle_base.sin();

        // Total dots to emit = eased * (w + several bounces worth).
        let total_dots = ((ctx.eased * (w * 6) as f32) as usize).max(1);
        let step = 1.0f32;

        for _ in 0..total_dots {
            fx += dx * step;
            fy += dy * step;

            // Reflect off top/bottom walls.
            if fy < 0.0 {
                fy = -fy;
                dy = -dy;
            } else if fy >= h as f32 {
                fy = 2.0 * h as f32 - fy - 2.0;
                dy = -dy;
            }
            // Wrap x so long beams keep bouncing across the bar.
            if fx >= w as f32 {
                fx -= w as f32;
            }

            let (bx, by) = (fx as i32, fy as i32);
            draw::dot_i(grid, bx, by);

            // Mark bounce point with a triple dot when direction reverses.
            if (dy > 0.0 && by <= 1) || (dy < 0.0 && by >= h as i32 - 2) {
                draw::dot_i(grid, bx - 1, by);
                draw::dot_i(grid, bx + 1, by);
            }
        }

        // Tint: gradient left to right.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ─── 8. Fiber pulse ─────────────────────────────────────────────────────────

/// Light pulses travel down multiple sinusoidal fiber lines; speed = eased.
struct FiberPulse;
impl ProgressStyle for FiberPulse {
    fn name(&self) -> &str {
        "fiber-pulse"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Pulses race down curved fiber-optic lines; pulse speed and fiber count = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let n_fibers = ((ctx.eased * 6.0 + 1.0) as usize).max(1).min(7);
        let pulse_speed = 0.3 + ctx.eased * 2.5;

        for f in 0..n_fibers {
            // Each fiber is a sine wave with unique amplitude and frequency.
            let fiber_amp = (h as f32 / (n_fibers as f32 * 1.2 + 1.0)).max(0.5);
            let freq = 1.5 + f as f32 * 0.4;
            let phase_off = f as f32 * PI * 0.6;
            // Vertical offset so fibers don't all overlap.
            let v_off = if n_fibers == 1 {
                h as f32 / 2.0
            } else {
                (f as f32 + 0.5) * h as f32 / n_fibers as f32
            };

            // Draw the fiber path.
            for px in 0..w {
                let x_frac = px as f32 / w.saturating_sub(1).max(1) as f32;
                let py = v_off + fiber_amp * (x_frac * freq * 2.0 * PI + phase_off).sin();
                draw::dot_i(grid, px as i32, py as i32);
            }

            // Pulse position: travels left→right, wraps.
            let pulse_phase = (ctx.time * pulse_speed + f as f32 * 0.37).fract();
            let pulse_x = (pulse_phase * w as f32) as usize;

            // Draw a 7-dot wide bright pulse on the fiber at pulse_x.
            let pulse_half = 4usize;
            for dp in 0..=pulse_half * 2 {
                let ppx = pulse_x.saturating_sub(pulse_half) + dp;
                if ppx >= w {
                    break;
                }
                let x_frac = ppx as f32 / w.saturating_sub(1).max(1) as f32;
                let py = v_off + fiber_amp * (x_frac * freq * 2.0 * PI + phase_off).sin();
                // Intensity falls off from centre of pulse.
                let dist = (dp as i32 - pulse_half as i32).abs();
                if dist <= 2 {
                    // Core dot.
                    draw::dot_i(grid, ppx as i32, py as i32);
                    draw::dot_i(grid, ppx as i32, py as i32 - 1);
                    draw::dot_i(grid, ppx as i32, py as i32 + 1);
                } else {
                    draw::dot_i(grid, ppx as i32, py as i32);
                }
            }

            // Tint per fiber.
            let t = f as f32 / n_fibers.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
            }
        }

        Ok(())
    }
}

// ─── 9. Plasma bolt ─────────────────────────────────────────────────────────

/// A jagged lightning beam from left to right; jitter driven by time.
struct PlasmaBolt;
impl ProgressStyle for PlasmaBolt {
    fn name(&self) -> &str {
        "plasma-bolt"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Jagged plasma lightning beam jitters frame-by-frame via time; length = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let bolt_len = ((ctx.eased * w as f32) as usize).max(1).min(w);
        let mid = h / 2;
        let jitter_epoch = (ctx.time * 20.0) as u32; // changes 20×/sec

        // Main bolt: walk left-to-right, each column's y is jittered.
        let mut prev_y = mid as i32;
        let max_jitter = ((h / 2) as i32).max(1);

        for px in 0..bolt_len {
            // Jitter: unique per (x, epoch).
            let jitter_raw = hashf(px as u32 * 7 + jitter_epoch * 31);
            let jitter = ((jitter_raw * 2.0 - 1.0) * max_jitter as f32) as i32;
            let mut cy = mid as i32 + jitter;
            cy = cy.clamp(0, h as i32 - 1);

            // Draw vertical segment between prev_y and cy (ensures continuity).
            let y_lo = prev_y.min(cy).clamp(0, h as i32 - 1) as usize;
            let y_hi = prev_y.max(cy).clamp(0, h as i32 - 1) as usize;
            draw::vline(grid, px, y_lo, y_hi);

            prev_y = cy;
        }

        // Secondary thinner bolt slightly offset in time.
        let jitter_epoch2 = jitter_epoch.wrapping_add(7);
        let mut prev_y2 = mid as i32;
        for px in 0..bolt_len {
            let jitter_raw = hashf(px as u32 * 13 + jitter_epoch2 * 41);
            let jitter = ((jitter_raw * 2.0 - 1.0) * (max_jitter / 2) as f32) as i32;
            let mut cy = mid as i32 + jitter;
            cy = cy.clamp(0, h as i32 - 1);
            draw::dot_i(grid, px as i32, cy);
            // Thinner: no vline, just the dot.
            prev_y2 = cy;
        }
        let _ = prev_y2;

        // Tint the bolt region.
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ─── 10. Particle accelerator ───────────────────────────────────────────────

/// Dots race along a track; spacing and speed scale with eased.
struct ParticleAccelerator;
impl ProgressStyle for ParticleAccelerator {
    fn name(&self) -> &str {
        "particle-accelerator"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Charged particles accelerate along a beam track; speed and density = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = h / 2;

        // Track: double-rail lines.
        let rail_above = mid.saturating_sub(1);
        let rail_below = (mid + 1).min(h.saturating_sub(1));
        draw::hline(grid, 0, w.saturating_sub(1), rail_above);
        draw::hline(grid, 0, w.saturating_sub(1), rail_below);

        // Cross-ties every few dots.
        let tie_spacing = 5usize;
        let mut tx = 0;
        while tx < w {
            draw::vline(grid, tx, rail_above, rail_below);
            tx += tie_spacing;
        }

        // Particles: n_particles travel left→right at speed = eased.
        let speed = 0.4 + ctx.eased * 4.0;
        let n_particles = ((ctx.eased * 8.0 + 1.0) as usize).max(1).min(10);
        let particle_gap = w / n_particles.max(1);

        for p in 0..n_particles {
            let phase_off = p as f32 / n_particles as f32;
            // Position: wraps continuously.
            let pos = ((ctx.time * speed + phase_off).fract() * w as f32) as usize;
            let pos = pos.min(w.saturating_sub(1));

            // Particle core: 3-dot cluster on the centreline.
            draw::dot_i(grid, pos as i32, mid as i32);
            draw::dot_i(grid, pos as i32 + 1, mid as i32);
            if pos > 0 {
                draw::dot_i(grid, pos as i32 - 1, mid as i32);
            }

            // Wake: decreasing density behind the particle.
            let wake_len = (particle_gap / 2).max(2).min(w);
            for w_step in 1..wake_len {
                // Probability falls off with distance.
                if w_step * 3 < wake_len * 2 {
                    let wx = if pos >= w_step { pos - w_step } else { 0 };
                    draw::dot_i(grid, wx as i32, mid as i32);
                }
            }
        }

        // Tint: speed gradient.
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased + (1.0 - ctx.eased) * 0.1);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ─── 11. Disco fan ──────────────────────────────────────────────────────────

/// Radial beams sweep from a corner origin; sweep angle grows with eased + time.
struct DiscoFan;
impl ProgressStyle for DiscoFan {
    fn name(&self) -> &str {
        "disco-fan"
    }
    fn theme(&self) -> &str {
        "lasers"
    }
    fn describe(&self) -> &str {
        "Radial fan of beams sweeps from a corner; beam count and arc = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Fan origin: bottom-left corner.
        let ox = 0i32;
        let oy = h as i32 - 1;

        // Number of beams.
        let n_beams = ((ctx.eased * 9.0 + 1.0) as usize).max(1).min(10);

        // The fan sweeps continuously via time, occupying an arc that grows with eased.
        let arc = ctx.eased * PI * 0.9 + 0.1; // arc in radians, 0.1..~π·0.9
        let sweep_center = PI / 2.0 * (0.4 + 0.6 * ctx.eased) - (ctx.time * 0.6).sin() * arc * 0.3; // centre oscillates

        for b in 0..n_beams {
            let angle_frac = if n_beams == 1 {
                0.5f32
            } else {
                b as f32 / (n_beams - 1) as f32
            };
            let angle = sweep_center - arc / 2.0 + angle_frac * arc;

            // Beam extends to the far edge of the grid.
            let beam_len = ((w as f32).hypot(h as f32) as i32 + 2).max(2);
            let end_x = ox + (angle.cos() * beam_len as f32) as i32;
            let end_y = oy - (angle.sin() * beam_len as f32) as i32;

            beam_line(grid, ox, oy, end_x, end_y, (w + h) * 2);

            // Tint each beam column.
            let t = b as f32 / n_beams.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            // Paint a stripe along the beam.
            let x_end = end_x.clamp(0, cells_w as i32 - 1) as usize;
            let x_start = 0usize;
            let (lo, hi) = if x_start <= x_end {
                (x_start, x_end)
            } else {
                (x_end, x_start)
            };
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, lo, hi.min(cells_w.saturating_sub(1)), color);
            }
        }

        // Bright origin point.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, ox + dx, oy + dy);
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
    let styles = progress::styles::lasers::styles();
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
