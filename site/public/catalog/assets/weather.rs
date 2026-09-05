//! `weather` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O weather.rs && ./weather [style-name]
//! ```

const DEFAULT_STYLE: &str = "hurricane";

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
    pub mod weather {
//! Weather / meteorology progress bars.
//!
//! Twelve animated styles built entirely from `draw::` helpers. Each bar reads
//! `ctx.eased` for its fill/intensity amount and `ctx.time` for looping
//! animation, so they stay alive even when progress is held constant. Every
//! style targets a distinct atmospheric phenomenon — hurricane spirals, tornado
//! funnels, accumulating snowdrifts, hailstreaks, rainbow arcs, fog density,
//! barometer needles, windsock gusts, blizzard whiteout, cumulonimbus
//! convection, frost crystal propagation, and a mercury thermometer column.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── registry ──────────────────────────────────────────────────────────────────

/// All styles in the `weather` theme.
///
/// Each style is a structurally distinct weather phenomenon — spiral arms,
/// funnel geometry, accumulation physics, optical arcs, density fields,
/// analog instruments, fluid flow, and crystal growth. Color is not the
/// distinguishing axis; the drawn geometry is.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Hurricane),
        Box::new(Tornado),
        Box::new(SnowAccumulation),
        Box::new(Hailstorm),
        Box::new(RainbowArc),
        Box::new(FogRollIn),
        Box::new(Barometer),
        Box::new(WindSock),
        Box::new(BlizzardWhiteout),
        Box::new(Cumulonimbus),
        Box::new(FrostCrystals),
        Box::new(Thermometer),
    ]
}

// ── deterministic hash ────────────────────────────────────────────────────────

/// Fast integer hash used for stable per-particle position seeds.
#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ── 1. Hurricane ─────────────────────────────────────────────────────────────

/// Rotating spiral arms with a calm eye, intensity scales with `eased`.
struct Hurricane;
impl ProgressStyle for Hurricane {
    fn name(&self) -> &str {
        "hurricane"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Spiral arms rotate around a calm eye; intensity and radius grow with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        // Max radius shrinks slightly from the edge.
        let max_rx = (w / 2).saturating_sub(1) as f32;
        let max_ry = (h / 2).saturating_sub(1) as f32;
        let intensity = ctx.eased;

        // Eye: a small empty circle, radius grows slightly with progress.
        let eye_r = (1.0 + intensity * 2.0) as i32;

        // Four spiral arms, each offset by PI/2.
        let arms = 4usize;
        let arm_turns = 1.5_f32;
        let steps = 80usize;
        for arm in 0..arms {
            let arm_offset = arm as f32 * 2.0 * PI / arms as f32;
            for s in 0..steps {
                let t = s as f32 / steps as f32;
                let r_frac = t * intensity; // arm length scales with eased
                let angle = arm_offset + t * arm_turns * 2.0 * PI + ctx.time * 1.8;
                let rx = max_rx * r_frac;
                let ry = max_ry * r_frac;
                let px = cx + (angle.cos() * rx) as i32;
                let py = cy + (angle.sin() * ry) as i32;
                // Skip eye region.
                let dist2 = (px - cx) * (px - cx) + (py - cy) * (py - cy);
                if dist2 > eye_r * eye_r {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Eye wall: ring at eye radius.
        let eye_steps = 32usize;
        for s in 0..eye_steps {
            let a = s as f32 / eye_steps as f32 * 2.0 * PI;
            let ex = cx + (a.cos() * eye_r as f32) as i32;
            let ey = cy + (a.sin() * eye_r as f32) as i32;
            draw::dot_i(grid, ex, ey);
        }

        // Color: palette across rows.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * intensity);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 2. Tornado ───────────────────────────────────────────────────────────────

/// Tapering funnel that widens toward the ground; debris vortices swirl.
struct Tornado;
impl ProgressStyle for Tornado {
    fn name(&self) -> &str {
        "tornado"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Tapering funnel widens toward the ground; debris particles orbit the vortex"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Funnel centre x drifts a little with time — the wobble.
        let base_cx = (w / 2) as f32;
        let wobble = (ctx.time * 1.3).sin() * (w as f32 * 0.06).max(1.0);
        let cx = (base_cx + wobble) as i32;

        // Funnel occupies `eased` fraction of height from the top.
        let funnel_bottom = (ctx.eased * h as f32) as usize;

        // At each row y, funnel half-width grows linearly: 1 at top, max at bottom.
        let max_hw = (w as f32 * 0.35).max(2.0);

        for y in 0..funnel_bottom.min(h) {
            let frac = if funnel_bottom <= 1 {
                1.0f32
            } else {
                y as f32 / (funnel_bottom - 1) as f32
            };
            let hw = (frac * max_hw) as i32;
            // Left and right edges of the funnel.
            draw::dot_i(grid, cx - hw, y as i32);
            draw::dot_i(grid, cx + hw, y as i32);
            // Ground touchdown: fill the bottom ring solid.
            if y + 1 >= funnel_bottom {
                for dx in -hw..=hw {
                    draw::dot_i(grid, cx + dx, y as i32);
                }
            }
        }

        // Debris: 8 particles orbiting at different radii and speeds.
        let debris_count = 8usize;
        for d in 0..debris_count {
            let h_val = hash(d as u32);
            let orbit_r_frac = 0.3 + (h_val & 0xFF) as f32 / 255.0 * 0.5; // 0.3..0.8
            let orbit_rx = max_hw * orbit_r_frac * 1.4;
            let orbit_ry = (h as f32 * 0.18).max(1.0) * orbit_r_frac;
            let h2 = hash(h_val);
            let orbit_y_frac = (h2 & 0xFF) as f32 / 255.0;
            let orbit_cy = (orbit_y_frac * funnel_bottom.max(1) as f32) as i32;
            let speed = 2.0 + (h_val >> 8 & 0xFF) as f32 / 255.0 * 4.0;
            let phase = d as f32 / debris_count as f32 * 2.0 * PI;
            let angle = ctx.time * speed + phase;
            let dx = (angle.cos() * orbit_rx) as i32 + cx;
            let dy = orbit_cy + (angle.sin() * orbit_ry) as i32;
            draw::dot_i(grid, dx, dy);
        }

        // Tint: palette from top (pale) to ground (dark).
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 3. Snow accumulation ─────────────────────────────────────────────────────

/// Flakes drift down; a drift pile rises from the bottom with `eased`.
struct SnowAccumulation;
impl ProgressStyle for SnowAccumulation {
    fn name(&self) -> &str {
        "snow-accumulation"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Snowflakes drift downward while a snowdrift pile rises from the bottom with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Drift pile: solid fill rising from the bottom.
        let drift_h = (ctx.eased * h as f32).round() as usize;
        let drift_top = h.saturating_sub(drift_h);
        if drift_h > 0 {
            // Curved top surface: sine bump.
            for x in 0..w {
                let bump =
                    ((x as f32 / w.max(1) as f32 * PI * 2.0).sin() * (h as f32 * 0.06)) as i32;
                let top = (drift_top as i32 - bump).max(0) as usize;
                draw::vline(grid, x, top, h.saturating_sub(1));
            }
        }

        // Falling flakes: 12 flakes, each with own horizontal lane & phase.
        let flake_count = 12usize;
        for f in 0..flake_count {
            let h_val = hash(f as u32);
            let lane_x = (h_val & 0xFFFF) as f32 / 65535.0 * (w.saturating_sub(1)) as f32;
            let speed = 0.6 + (h_val >> 16 & 0xFF) as f32 / 255.0 * 0.8;
            let phase = f as f32 / flake_count as f32;
            // Drift stops above the pile.
            let fall_space = drift_top.saturating_sub(1);
            if fall_space == 0 {
                continue;
            }
            let cycle = (ctx.time * speed + phase).fract();
            let fy = (cycle * fall_space as f32) as usize;
            // Horizontal sway.
            let sway = ((ctx.time * 1.2 + phase * 7.3).sin() * (w as f32 * 0.04)).round() as i32;
            let fx = (lane_x as i32 + sway).clamp(0, w.saturating_sub(1) as i32);
            draw::dot_i(grid, fx, fy as i32);
        }

        // Tint: cool blue for drift, lighter above.
        let (cw, ch) = grid.dimensions();
        let drift_top_cell = drift_top / 4;
        for cy_c in 0..ch {
            let t = if cy_c >= drift_top_cell {
                (cy_c - drift_top_cell) as f32 / ch.saturating_sub(1).max(1) as f32
            } else {
                0.0
            };
            let color = ctx.palette.sample(t * 0.6 + 0.4 * ctx.eased);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 4. Hailstorm ─────────────────────────────────────────────────────────────

/// Fast vertical streaks that hit the ground and bounce.
struct Hailstorm;
impl ProgressStyle for Hailstorm {
    fn name(&self) -> &str {
        "hailstorm"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Dense hailstones streak straight down and bounce on impact; density ramps with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of active stones grows with progress.
        let stone_count = (2.0 + ctx.eased * 20.0).round() as usize;

        for s in 0..stone_count {
            let h_val = hash(s as u32);
            // Horizontal position: fixed per stone.
            let sx = (h_val & 0xFFFF) as f32 / 65535.0 * (w.saturating_sub(1)) as f32;
            let speed = 1.8 + (h_val >> 16 & 0xFF) as f32 / 255.0 * 2.5;
            let phase = s as f32 / stone_count.max(1) as f32;

            // Vertical: linear fall + bounce near bottom.
            let cycle = (ctx.time * speed + phase).fract();
            // Bounce: reflect off the bottom — fold the cycle around 0.5.
            let bounce_t = if cycle < 0.85 {
                cycle / 0.85
            } else {
                // Short bounce-up.
                1.0 - (cycle - 0.85) / 0.15 * 0.25
            };
            let sy = (bounce_t * (h.saturating_sub(1)) as f32) as i32;
            let sx_i = sx as i32;

            // Stone: a short vertical streak (2-3 dots).
            draw::dot_i(grid, sx_i, sy);
            draw::dot_i(grid, sx_i, sy - 1);

            // Impact flash at the very bottom when stone lands.
            if bounce_t > 0.95 {
                let flash_w = 3i32;
                for dx in -flash_w..=flash_w {
                    draw::dot_i(grid, sx_i + dx, (h.saturating_sub(1)) as i32);
                }
            }
        }

        // Ground line.
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        // Cold-grey tint.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(1.0 - t * 0.5);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 5. Rainbow arc ───────────────────────────────────────────────────────────

/// Concentric colour arcs drawn band by band as progress increases.
struct RainbowArc;
impl ProgressStyle for RainbowArc {
    fn name(&self) -> &str {
        "rainbow-arc"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Concentric rainbow arcs appear band by band from the horizon as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Arc centred at the bottom-centre.
        let cx = (w / 2) as i32;
        let base_y = h as i32;

        // Seven bands of the rainbow; each appears when progress passes its threshold.
        let bands = 7usize;
        let max_ry = (h.saturating_sub(1)).max(1) as f32;
        let max_rx = (w / 2).saturating_sub(0) as f32;

        use crate::Color;
        let band_colors: [Color; 7] = [
            Color::rgb(148, 0, 211), // violet
            Color::rgb(75, 0, 130),  // indigo
            Color::rgb(0, 0, 255),   // blue
            Color::rgb(0, 128, 0),   // green
            Color::rgb(255, 255, 0), // yellow
            Color::rgb(255, 127, 0), // orange
            Color::rgb(255, 0, 0),   // red (outermost)
        ];

        for (band, &color) in band_colors.iter().enumerate() {
            // Band `band` becomes visible when progress crosses its threshold.
            let threshold = band as f32 / bands as f32;
            if ctx.eased < threshold {
                continue;
            }

            // Partial reveal on the current outermost visible band.
            let band_frac = ((ctx.eased - threshold) * bands as f32).clamp(0.0, 1.0);

            // Radius fraction for this band: innermost (band 0) is smallest.
            let r_frac = (band + 1) as f32 / bands as f32;
            let rx = (max_rx * r_frac) as i32;
            let ry = (max_ry * r_frac) as i32;

            // Draw a semicircle arc (angles PI to 0, i.e. left horizon to right).
            let steps = (rx.max(ry) * 4).max(32) as usize;
            let arc_steps = (steps as f32 * band_frac).round() as usize;
            for s in 0..arc_steps {
                // Angle from PI (left) to 0 (right) so it draws left→right.
                let a = PI - s as f32 / steps.max(1) as f32 * PI;
                let px = cx + (a.cos() * rx as f32) as i32;
                let py = base_y - (a.sin() * ry as f32) as i32;
                draw::dot_i(grid, px, py);
            }

            // Apply band color.
            let (cw, ch) = grid.dimensions();
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
            }
        }

        // Re-tint with palette sampled at eased to blend the per-band colors.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(1.0 - t);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 6. Fog rolling in ────────────────────────────────────────────────────────

/// Shade-glyph density sweeps left-to-right; denser at the leading edge.
struct FogRollIn;
impl ProgressStyle for FogRollIn {
    fn name(&self) -> &str {
        "fog-roll-in"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Dense fog rolls in from the left; the leading edge thickens with a shade-glyph gradient"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        let fog_front = (ctx.eased * cw as f32) as usize;

        for cx in 0..cw {
            // How deeply into the fog are we? 0=clear, 1=deep fog.
            if cx >= fog_front {
                break;
            }
            let depth = (fog_front - cx) as f32 / fog_front.max(1) as f32;
            // Add a slow undulation from time.
            let wave = ((cx as f32 * 0.5 + ctx.time * 1.5).sin() * 0.12 + 1.0).clamp(0.5, 1.5);
            let density = (depth * wave).clamp(0.0, 1.0);

            // Map density [0..1] → shade level [0..4].
            let level = (density * 4.0).round() as usize;

            for cy in 0..ch {
                // Row-vary the fog slightly — ground-level is denser.
                let row_extra = cy as f32 / ch.saturating_sub(1).max(1) as f32 * 0.5;
                let row_level = ((density + row_extra).clamp(0.0, 1.0) * 4.0).round() as usize;
                draw::shade(grid, cx, cy, row_level.min(4));
                let _ = level; // used above for clarity
            }
        }

        // Tint the fog grey-blue via palette.
        let (cw2, ch2) = grid.dimensions();
        for cy_c in 0..ch2 {
            let color = ctx.palette.sample(ctx.eased * 0.5);
            draw::tint_row(grid, cy_c, 0, fog_front.min(cw2).saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 7. Barometer ─────────────────────────────────────────────────────────────

/// An analog pressure dial with a rotating needle driven by `eased`.
struct Barometer;
impl ProgressStyle for Barometer {
    fn name(&self) -> &str {
        "barometer"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Analog pressure gauge: a needle sweeps from STORMY to FAIR as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = (w.min(h * 2) / 2).saturating_sub(2) as i32;

        // Arc: semicircle from left (STORMY) to right (FAIR), opening downward.
        // Angles: PI (left) down to 0 (right) sweeping through 0..PI (top half).
        let arc_steps = 60usize;
        for s in 0..=arc_steps {
            let a = PI - s as f32 / arc_steps as f32 * PI;
            let px = cx + (a.cos() * max_r as f32) as i32;
            let py = cy - (a.sin() * max_r as f32) as i32; // minus: up
            draw::dot_i(grid, px, py);
        }

        // Tick marks at 0%, 25%, 50%, 75%, 100% of the arc.
        for tick in 0..=4 {
            let a = PI - tick as f32 / 4.0 * PI;
            let inner = (max_r - 3).max(1);
            let outer = max_r;
            for r in inner..=outer {
                let px = cx + (a.cos() * r as f32) as i32;
                let py = cy - (a.sin() * r as f32) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Needle: points from centre toward the arc, angle driven by eased.
        // eased=0 → left (PI), eased=1 → right (0).
        let needle_angle = PI - ctx.eased * PI;
        // Slow pressure tremor.
        let tremor = (ctx.time * 6.0).sin() * 0.03;
        let needle_angle = needle_angle + tremor;
        let needle_len = (max_r - 2).max(1) as f32;
        let steps = needle_len.round() as usize;
        for s in 0..=steps {
            let r = s as f32;
            let px = cx + (needle_angle.cos() * r) as i32;
            let py = cy - (needle_angle.sin() * r) as i32;
            draw::dot_i(grid, px, py);
        }

        // Centre pivot.
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx + 1, cy);

        // Tint: warm high-pressure orange at right, stormy blue at left.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let color = ctx.palette.sample(ctx.eased);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 8. Wind sock ─────────────────────────────────────────────────────────────

/// A conical sock extends right; gust streak lines sweep across behind it.
struct WindSock;
impl ProgressStyle for WindSock {
    fn name(&self) -> &str {
        "wind-sock"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "A windsock cone extends as gusts blow; streak lines visualise airspeed"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = (h / 2) as i32;
        // Sock extends from left pole to eased * width.
        let sock_len = (ctx.eased * (w.saturating_sub(2)) as f32).round() as usize;
        let pole_x = 0i32;

        // Pole: vertical line at x=0.
        draw::vline(grid, 0, 0, h.saturating_sub(1));

        // Horizontal attachment rod (top half of pole).
        let attach_y = (h / 4) as i32;
        draw::dot_i(grid, pole_x, attach_y);

        // Sock: tapered cone — wide at pole (opening_hw), narrow at tip (1).
        let opening_hw = (h as f32 * 0.38).max(1.0) as i32;
        for x in 0..sock_len.min(w.saturating_sub(1)) {
            let frac = if sock_len <= 1 {
                0.0
            } else {
                x as f32 / (sock_len - 1) as f32
            };
            // Wind flutter: opening oscillates slightly with time.
            let flutter = (ctx.time * 4.0 + x as f32 * 0.2).sin() * (opening_hw as f32 * 0.12);
            let hw = ((opening_hw as f32 * (1.0 - frac) + 1.0 + flutter) as i32).max(1);
            draw::dot_i(grid, pole_x + x as i32 + 1, mid - hw);
            draw::dot_i(grid, pole_x + x as i32 + 1, mid + hw);
        }
        // Tip dot.
        if sock_len > 0 {
            draw::dot_i(grid, pole_x + sock_len as i32, mid);
        }

        // Gust streaks: horizontal lines scrolling left-to-right.
        let gust_count = 5usize;
        for g in 0..gust_count {
            let h_val = hash(g as u32);
            let gust_y = (h_val & 0xFF) as f32 / 255.0 * h as f32;
            let speed = 1.2 + (h_val >> 8 & 0xFF) as f32 / 255.0 * 2.0;
            let phase = g as f32 / gust_count as f32;
            // Streaks scroll from left, wrap around.
            let gust_x_frac = (ctx.time * speed + phase).fract();
            let gust_x = (gust_x_frac * w as f32) as usize;
            let gust_len = (w / 6).max(2);
            let x0 = gust_x;
            let x1 = (gust_x + gust_len).min(w.saturating_sub(1));
            draw::hline(grid, x0, x1, gust_y as usize);
        }

        // Tint.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased + ctx.eased * 0.3);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 9. Blizzard whiteout ─────────────────────────────────────────────────────

/// Shade density ramps toward full whiteout as progress approaches 1.
struct BlizzardWhiteout;
impl ProgressStyle for BlizzardWhiteout {
    fn name(&self) -> &str {
        "blizzard-whiteout"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Visibility collapses into whiteout: shade density ramps to full as progress reaches 1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Base whiteout shade level driven by eased.
        let base_density = ctx.eased;

        for cx in 0..cw {
            for cy_c in 0..ch {
                // Add spatial noise and time flicker to break the uniformity.
                let h_val = hash((cx as u32).wrapping_mul(31).wrapping_add(cy_c as u32 * 97));
                let spatial_noise = (h_val & 0xFF) as f32 / 255.0 * 0.3;
                let flicker = (ctx.time * 8.0 + cx as f32 * 0.7 + cy_c as f32 * 1.3).sin() * 0.08;
                let density =
                    (base_density + spatial_noise * base_density + flicker).clamp(0.0, 1.0);
                let level = (density * 4.0).round() as usize;
                draw::shade(grid, cx, cy_c, level.min(4));
            }
        }

        // Particles: bright dots that blow horizontally through the whiteout.
        let (w, h) = draw::dot_dims(grid);
        let particle_count = (4.0 + ctx.eased * 16.0).round() as usize;
        for p in 0..particle_count {
            let h_val = hash(p as u32 + 500);
            let row_frac = (h_val & 0xFFFF) as f32 / 65535.0;
            let py = (row_frac * h.saturating_sub(1) as f32) as usize;
            let speed = 1.0 + (h_val >> 16 & 0xFF) as f32 / 255.0 * 3.0;
            let phase = p as f32 / particle_count.max(1) as f32;
            let px = ((ctx.time * speed + phase).fract() * w as f32) as usize;
            draw::dot(grid, px.min(w.saturating_sub(1)), py);
        }

        // Tint: cold white-blue.
        for cy_c in 0..ch {
            let color = ctx.palette.sample(ctx.eased * 0.3);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 10. Cumulonimbus ─────────────────────────────────────────────────────────

/// Cloud builds vertically into a thunderhead, then rain falls beneath.
struct Cumulonimbus;
impl ProgressStyle for Cumulonimbus {
    fn name(&self) -> &str {
        "cumulonimbus"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "A cloud tower builds upward into a thunderhead, then rain streaks fall below"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // The bar is divided: top 60% = cloud building, bottom 40% = rain.
        let cloud_zone = (h as f32 * 0.6) as usize;
        let rain_zone_top = cloud_zone;

        // Cloud anvil: top half fills with irregular lobes as progress grows.
        // Each lobe is an ellipse centred along the width at a fixed height.
        let lobe_count = 5usize;
        let cloud_height = (ctx.eased * cloud_zone as f32).round() as usize;

        for lobe in 0..lobe_count {
            let lobe_cx = (lobe as f32 + 0.5) / lobe_count as f32 * w as f32;
            // Lobes vary in width.
            let h_val = hash(lobe as u32 + 200);
            let lobe_rx = (w as f32 / lobe_count as f32 * 0.6
                + (h_val & 0xFF) as f32 / 255.0 * w as f32 * 0.08)
                .max(2.0);
            let lobe_ry = cloud_height as f32 * (0.5 + (h_val >> 8 & 0xFF) as f32 / 255.0 * 0.5);
            let lobe_cy = (cloud_zone.saturating_sub(1)) as f32;

            // Animate lobe tops with a slow boil.
            let boil = (ctx.time * 0.8 + lobe as f32 * 1.4).sin() * cloud_height as f32 * 0.04;

            let steps = 40usize;
            for s in 0..=steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                let ex = (lobe_cx + a.cos() * lobe_rx + boil).round() as i32;
                let ey = (lobe_cy - (a.sin().abs() * lobe_ry)).round() as i32;
                // Only the top hemisphere (a.sin() >= 0 draws the fluffy top).
                if a.sin() >= 0.0 {
                    draw::dot_i(grid, ex, ey);
                }
                // Fill interior.
                if ey >= 0 && ey < cloud_zone as i32 {
                    let ey_fill = ey;
                    draw::dot_i(grid, ex, ey_fill);
                    draw::dot_i(grid, ex, lobe_cy as i32);
                    // Vertical fill from ey to base.
                    if ey < lobe_cy as i32 {
                        draw::vline(
                            grid,
                            ex.max(0) as usize,
                            ey.max(0) as usize,
                            lobe_cy as usize,
                        );
                    }
                }
            }
        }

        // Rain streaks below cloud zone — appear only after cloud is 50% built.
        if ctx.eased > 0.5 {
            let rain_intensity = ((ctx.eased - 0.5) * 2.0).clamp(0.0, 1.0);
            let rain_count = (2.0 + rain_intensity * 14.0).round() as usize;
            let rain_h = h.saturating_sub(rain_zone_top);
            if rain_h > 0 {
                for r in 0..rain_count {
                    let h_val = hash(r as u32 + 400);
                    let rx = (h_val & 0xFFFF) as f32 / 65535.0 * (w.saturating_sub(1)) as f32;
                    let speed = 1.5 + (h_val >> 16 & 0xFF) as f32 / 255.0 * 2.0;
                    let phase = r as f32 / rain_count.max(1) as f32;
                    let cycle = (ctx.time * speed + phase).fract();
                    let ry = rain_zone_top + (cycle * rain_h as f32) as usize;
                    draw::dot(grid, rx as usize, ry.min(h.saturating_sub(1)));
                    draw::dot(grid, rx as usize, (ry + 1).min(h.saturating_sub(1)));
                }
            }
        }

        // Tint: dark grey cloud, light blue rain.
        let (cw, ch) = grid.dimensions();
        let cloud_top_cell = 0;
        let rain_cell = (rain_zone_top / 4).min(ch.saturating_sub(1));
        for cy_c in cloud_top_cell..rain_cell.min(ch) {
            let color = ctx.palette.sample(0.3 * ctx.eased);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }
        for cy_c in rain_cell..ch {
            let color = ctx.palette.sample(ctx.eased * 0.7 + 0.3);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 11. Frost crystals ────────────────────────────────────────────────────────

/// Dendritic frost spreads inward from all four edges.
struct FrostCrystals;
impl ProgressStyle for FrostCrystals {
    fn name(&self) -> &str {
        "frost-crystals"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Dendritic frost crystals propagate inward from the edges; branching depth grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Frost depth in dots from each edge.
        let max_depth = ((w.min(h) / 2).saturating_sub(1)) as f32;
        let depth = (ctx.eased * max_depth).round() as usize;

        if depth == 0 {
            return Ok(());
        }

        // Grow dendritic crystals from left, right, top, bottom edges.
        // Each edge has a set of roots spaced every 4 dots along the edge.
        // From each root, a main spine grows inward; then side branches sprout
        // every few dots, each in the perpendicular direction.
        let branch_spacing = 4usize;
        let branch_len_frac = 0.4_f32;

        // Left edge.
        let mut root = 0usize;
        while root < h {
            let h_val = hash(root as u32 + 1000);
            // Slight y variation.
            let ry = root;
            for d in 0..depth.min(w) {
                draw::dot(grid, d, ry.min(h.saturating_sub(1)));
                // Side branches perpendicular (up/down).
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(h / 2) {
                        draw::dot(grid, d, (ry + b).min(h.saturating_sub(1)));
                        if ry >= b {
                            draw::dot(grid, d, ry - b);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Right edge.
        root = 0;
        while root < h {
            let h_val = hash(root as u32 + 2000);
            let ry = root;
            for d in 0..depth.min(w) {
                let x = w.saturating_sub(1 + d);
                draw::dot(grid, x, ry.min(h.saturating_sub(1)));
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(h / 2) {
                        draw::dot(grid, x, (ry + b).min(h.saturating_sub(1)));
                        if ry >= b {
                            draw::dot(grid, x, ry - b);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Top edge.
        root = 0;
        while root < w {
            let h_val = hash(root as u32 + 3000);
            let rx = root;
            for d in 0..depth.min(h) {
                draw::dot(grid, rx.min(w.saturating_sub(1)), d);
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(w / 2) {
                        draw::dot(grid, (rx + b).min(w.saturating_sub(1)), d);
                        if rx >= b {
                            draw::dot(grid, rx - b, d);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Bottom edge.
        root = 0;
        while root < w {
            let h_val = hash(root as u32 + 4000);
            let rx = root;
            for d in 0..depth.min(h) {
                let y = h.saturating_sub(1 + d);
                draw::dot(grid, rx.min(w.saturating_sub(1)), y);
                if d > 0 && d % branch_spacing == 0 {
                    let blen =
                        (depth as f32 * branch_len_frac * (h_val & 0xFF) as f32 / 255.0) as usize;
                    for b in 1..blen.min(w / 2) {
                        draw::dot(grid, (rx + b).min(w.saturating_sub(1)), y);
                        if rx >= b {
                            draw::dot(grid, rx - b, y);
                        }
                    }
                }
            }
            root += branch_spacing + (h_val >> 8 & 3) as usize;
        }

        // Ice-blue tint.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let color = ctx.palette.sample(0.2 + ctx.eased * 0.6);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 12. Thermometer ──────────────────────────────────────────────────────────

/// A mercury column rises in a vertical thermometer tube.
struct Thermometer;
impl ProgressStyle for Thermometer {
    fn name(&self) -> &str {
        "thermometer"
    }
    fn theme(&self) -> &str {
        "weather"
    }
    fn describe(&self) -> &str {
        "Mercury column rises in a thermometer tube; bulb glows at bottom, tick marks show scale"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Thermometer shaft: centred horizontally, tube width = 4 dots.
        let cx = (w / 2) as i32;
        let tube_w = 4i32; // inner width
        let tube_left = cx - tube_w / 2;
        let tube_right = cx + tube_w / 2;

        // Tube occupies top 85% of the height; bulb is the bottom 15%.
        let bulb_top_y = (h as f32 * 0.82) as usize;
        let tube_top = 1usize;

        // Tube outline.
        for y in tube_top..bulb_top_y {
            draw::dot_i(grid, tube_left - 1, y as i32);
            draw::dot_i(grid, tube_right + 1, y as i32);
        }
        draw::hline(
            grid,
            tube_left.max(0) as usize,
            (tube_right + 1) as usize,
            tube_top,
        );

        // Bulb: filled circle at the bottom.
        let bulb_cx = cx;
        let bulb_cy = (bulb_top_y + h.saturating_sub(1)) as i32 / 2;
        let bulb_r = ((h.saturating_sub(bulb_top_y)) / 2).max(2) as i32;
        for dy in -bulb_r..=bulb_r {
            for dx in -bulb_r..=bulb_r {
                if dx * dx + dy * dy <= bulb_r * bulb_r + bulb_r {
                    draw::dot_i(grid, bulb_cx + dx, bulb_cy + dy);
                }
            }
        }

        // Mercury column: fills the tube from the bulb upward.
        let tube_h = bulb_top_y.saturating_sub(tube_top);
        let mercury_h = (ctx.eased * tube_h as f32).round() as usize;
        let mercury_top = bulb_top_y.saturating_sub(mercury_h);
        if mercury_h > 0 {
            for y in mercury_top..bulb_top_y {
                for x in tube_left..=tube_right {
                    draw::dot_i(grid, x, y as i32);
                }
            }
        }

        // Tick marks on the right side of the tube, every 25%.
        let tick_positions = [0.0, 0.25, 0.5, 0.75, 1.0];
        for &tp in &tick_positions {
            let ty = bulb_top_y.saturating_sub((tp * tube_h as f32).round() as usize);
            let tick_len = if (tp * 4.0).round() as usize % 2 == 0 {
                4i32
            } else {
                2i32
            };
            for dx in 0..tick_len {
                draw::dot_i(grid, tube_right + 2 + dx, ty as i32);
            }
        }

        // Mercury tint: cool blue at empty → hot red at full via palette.
        let (cw, ch) = grid.dimensions();
        let mercury_top_cell = (mercury_top / 4).min(ch.saturating_sub(1));
        let bulb_top_cell = (bulb_top_y / 4).min(ch.saturating_sub(1));
        for cy_c in mercury_top_cell..=bulb_top_cell.min(ch.saturating_sub(1)) {
            let color = ctx.palette.sample(ctx.eased);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
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
    let styles = progress::styles::weather::styles();
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
