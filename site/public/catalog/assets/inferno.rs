//! `inferno` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O inferno.rs && ./inferno [style-name]
//! ```

const DEFAULT_STYLE: &str = "wildfire";

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
    pub mod inferno {
//! Fire progress bars — spreading flames, rising embers, white-hot heads.
//!
//! Everything burns on a char-red → orange → gold → white heat ramp.
//! Progress reads as fire spreading along the bar (or candles lighting,
//! or a torch cutting through) while `time` keeps the flames flickering.
//! All motion is deterministic in `(progress, time)`.

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

/// 3-D variant: hash `(x, y, z_int)` for time-slotted flicker.
#[inline]
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash2(x ^ z.wrapping_mul(1_234_567), y ^ z.wrapping_mul(7_654_321))
}

// ─── theme tint — heat ramp ─────────────────────────────────────────────────

/// Charred deep red at the cold end.
const HEAT_CHAR: Color = Color::rgb(143, 29, 18);
/// Flame orange in the body.
const HEAT_ORANGE: Color = Color::rgb(255, 122, 47);
/// Molten gold near the core.
const HEAT_GOLD: Color = Color::rgb(255, 200, 74);
/// White-hot center.
const HEAT_WHITE: Color = Color::rgb(255, 243, 214);

/// Sample the char → orange → gold → white heat ramp at `t` in `0.0..=1.0`.
fn sample_tint(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8, k: f32| (f32::from(a) + (f32::from(b) - f32::from(a)) * k) as u8;
    let (from, to, k) = if t < 0.45 {
        (HEAT_CHAR, HEAT_ORANGE, t / 0.45)
    } else if t < 0.8 {
        (HEAT_ORANGE, HEAT_GOLD, (t - 0.45) / 0.35)
    } else {
        (HEAT_GOLD, HEAT_WHITE, (t - 0.8) / 0.2)
    };
    Color::rgb(
        lerp(from.r, to.r, k),
        lerp(from.g, to.g, k),
        lerp(from.b, to.b, k),
    )
}

/// Applies the heat ramp to every cell the inner style drew: hotter toward
/// the bottom (where the fire lives) with a fast per-cell flicker.
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
        let slot = (ctx.time * 8.0) as i32;
        for y in 0..h {
            for x in 0..w {
                let ch = grid.get_char(x, y);
                if ch != '\u{2800}' && ch != ' ' {
                    let depth = (y as f32 + 0.5) / h.max(1) as f32;
                    let flicker = 0.2 * hash3(x as i32, y as i32, slot);
                    let _ = grid.set_cell_color(x, y, sample_tint(0.25 + 0.5 * depth + flicker));
                }
            }
        }
        Ok(())
    }
}

/// All styles in the `inferno` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Wildfire),
        Box::new(FlameFill),
        Box::new(EmberRise),
        Box::new(Blowtorch),
        Box::new(Candle),
        Box::new(Tinted(Flashover)),
        Box::new(Charline),
        Box::new(Tinted(Backdraft)),
        Box::new(SolarFlare),
        Box::new(Phoenix),
    ]
}

/// Fire spreads left to right: ember bed behind, flames at the front.
struct Wildfire;
impl ProgressStyle for Wildfire {
    fn name(&self) -> &str {
        "wildfire"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A fire line spreading across the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let front = (ctx.eased * w as f32).round() as usize;
        let slot = (ctx.time * 8.0) as i32;
        // Unburnt fuel ahead: a thin ground line.
        for x in front..w {
            draw::dot(grid, x, h - 1);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, Color::rgb(92, 74, 52));
        }
        // Ember bed behind the front: lower half, glowing unevenly.
        for x in 0..front {
            let bed = h / 2 + (hash2(x as i32, 3) * 2.0) as usize;
            for y in bed.min(h - 1)..h {
                draw::dot(grid, x, y);
            }
            let glow = 0.25 + 0.25 * hash3(x as i32, 0, slot / 2);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, sample_tint(glow));
        }
        // Burning zone at the front: tall flame tongues licking upward.
        let zone = 12usize;
        for k in 0..zone {
            let x = front.saturating_sub(zone) + k;
            if x >= w {
                break;
            }
            let near = k as f32 / zone as f32;
            let flick = hash3(x as i32, 1, slot);
            let tongue = ((0.45 + 0.55 * near) * (0.65 + 0.35 * flick) * h as f32) as usize;
            for y in h.saturating_sub(tongue.max(2))..h {
                draw::dot(grid, x, y);
            }
            // Sparks leaping off the tallest tongues.
            if flick > 0.6 {
                let sy = h.saturating_sub(tongue + 2);
                draw::dot(grid, x, sy);
            }
            let heat = 0.5 + 0.5 * near;
            for cy in 0..grid.dimensions().1 {
                draw::tint_row(grid, cy, x / 2, x / 2, sample_tint(heat - cy as f32 * 0.12));
            }
        }
        Ok(())
    }
}

/// The whole filled region is living flame, tips flickering.
struct FlameFill;
impl ProgressStyle for FlameFill {
    fn name(&self) -> &str {
        "flame-fill"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A bar of living flame"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let slot = (ctx.time * 8.0) as i32;
        for x in 0..filled {
            let sway = (x as f32 * 0.4 + ctx.time * TAU * 0.5).sin();
            let flick = hash3(x as i32, 0, slot);
            let height = ((0.55 + 0.2 * sway + 0.25 * flick) * h as f32) as usize;
            let top = h.saturating_sub(height.max(2));
            for y in top..h {
                draw::dot(grid, x, y);
            }
            // Heat rises: white at the base, red at the tips.
            let (_, ch_cells) = grid.dimensions();
            for cy in 0..ch_cells {
                let frac = 1.0 - cy as f32 / ch_cells.max(1) as f32;
                draw::tint_row(grid, cy, x / 2, x / 2, sample_tint(1.0 - frac * 0.75));
            }
        }
        Ok(())
    }
}

/// A glowing bed extends along the bar; embers rise and drift above it.
struct EmberRise;
impl ProgressStyle for EmberRise {
    fn name(&self) -> &str {
        "ember-rise"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "Embers drifting up from a glowing bed"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // The bed: two glowing dot rows.
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
            draw::dot(grid, x, h.saturating_sub(2));
            let glow = 0.35 + 0.3 * hash3(x as i32, 0, (ctx.time * 6.0) as i32);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, sample_tint(glow));
        }
        // Embers rise from the lit bed, drifting sideways as they climb.
        let head = h.saturating_sub(2) as f32;
        if head > 1.0 {
            for e in 0..(w / 3) {
                let ex = (hash2(e as i32, 5) * filled.max(1) as f32) as usize;
                if ex >= filled {
                    continue;
                }
                let rate = 0.25 + ((hash2(e as i32, 6) * 3.0).round()) * 0.25;
                let climb = (ctx.time * rate + hash2(e as i32, 7)).fract();
                let y = head - climb * head;
                let drift = ((climb * TAU).sin() * 2.5) as i32;
                let px = ex as i32 + drift;
                draw::dot_i(grid, px, y as i32);
                if px >= 0 {
                    let heat = 1.0 - climb;
                    let _ = grid.set_cell_color(
                        px as usize / 2,
                        y as usize / 4,
                        sample_tint(0.4 + 0.6 * heat),
                    );
                }
            }
        }
        Ok(())
    }
}

/// A torch head cuts rightward, leaving a cooling molten line.
struct Blowtorch;
impl ProgressStyle for Blowtorch {
    fn name(&self) -> &str {
        "blowtorch"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A torch cutting a molten line"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let head = (ctx.eased * w as f32) as usize;
        let mid = h / 2;
        // The cooling cut: brightness decays with distance behind the head.
        for x in 0..head.min(w) {
            draw::dot(grid, x, mid);
            draw::dot(grid, x, mid.saturating_sub(1));
            let cool = (head - x) as f32 / w.max(1) as f32;
            let _ = grid.set_cell_color(x / 2, mid / 4, sample_tint(1.0 - cool * 1.6));
        }
        // The torch cone: a bright wedge of dots at the head.
        let slot = (ctx.time * 12.0) as i32;
        for dx in 0..5i32 {
            let spread = dx;
            for dy in -spread..=spread {
                let x = head as i32 + dx;
                let y = mid as i32 + dy;
                if hash3(x, y, slot) < 0.75 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        if head < w {
            let _ = grid.set_cell_color(head / 2, mid / 4, HEAT_WHITE);
            let _ = grid.set_cell_color(((head + 4).min(w - 1)) / 2, mid / 4, HEAT_GOLD);
        }
        // Sparks spraying off the cut point.
        for s in 0..6i32 {
            if hash3(s, 9, slot) < 0.5 {
                let sx = head as i32 + (hash3(s, 10, slot) * 8.0) as i32 - 2;
                let sy = mid as i32 + (hash3(s, 11, slot) * 8.0) as i32 - 4;
                draw::dot_i(grid, sx, sy);
            }
        }
        Ok(())
    }
}

/// Ten candles light up one by one, flames swaying.
struct Candle;
impl ProgressStyle for Candle {
    fn name(&self) -> &str {
        "candle"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "Candles lighting one per ten percent"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let count = 10usize;
        let lit = (ctx.progress * count as f32).round() as usize;
        for c in 0..count {
            let x = ((c as f32 + 0.5) / count as f32 * w as f32) as usize;
            if x >= w {
                continue;
            }
            // Candle body: a short two-wide column from the floor.
            let body_top = h.saturating_sub(h / 3);
            for y in body_top..h {
                draw::dot(grid, x, y);
                draw::dot(grid, (x + 1).min(w - 1), y);
            }
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, Color::rgb(120, 104, 86));
            // Flame above lit candles, swaying on its own phase.
            if c < lit {
                let sway = ((ctx.time * TAU * 0.5 + c as f32 * 1.3).sin() * 1.5) as i32;
                let fx = x as i32 + sway;
                let fy = body_top as i32;
                draw::dot_i(grid, fx, fy - 2);
                draw::dot_i(grid, fx, fy - 3);
                draw::dot_i(grid, fx + 1, fy - 2);
                draw::dot_i(grid, fx + 1, fy - 3);
                draw::dot_i(grid, fx, fy - 4);
                draw::dot_i(grid, fx, fy - 5);
                if fx >= 0 {
                    let _ = grid.set_cell_color(
                        fx as usize / 2,
                        (fy as usize).saturating_sub(3) / 4,
                        HEAT_GOLD,
                    );
                    let _ = grid.set_cell_color(
                        fx as usize / 2,
                        (fy as usize).saturating_sub(1) / 4,
                        HEAT_WHITE,
                    );
                }
            }
        }
        Ok(())
    }
}

/// The whole bar heats through shades until it rolls with fire.
struct Flashover;
impl ProgressStyle for Flashover {
    fn name(&self) -> &str {
        "flashover"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "Rising heat shading into full flashover"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        for y in 0..ch {
            for x in 0..cw {
                let fx = x as f32 / cw.max(1) as f32;
                let roll = 0.15 * (fx * TAU + ctx.time * TAU * 0.5).sin();
                let heat = ctx.eased * 1.25 - fx * 0.35 + roll;
                let level = (heat.clamp(0.0, 1.0) * 4.4) as usize;
                if level > 0 {
                    draw::shade(grid, x, y, level.min(4));
                }
            }
        }
        Ok(())
    }
}

/// A fuse burns along the middle: sparks at the front, smoke behind.
struct Charline;
impl ProgressStyle for Charline {
    fn name(&self) -> &str {
        "charline"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A burning fuse with sparks and smoke"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let burn = (ctx.eased * w as f32) as usize;
        let mid = h / 2;
        // Unburnt fuse ahead.
        for x in burn..w {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, Color::rgb(122, 104, 76));
        }
        // Char line behind: the consumed fuse stays as a glowing broken line,
        // so the bar keeps reading at high progress.
        for x in 0..burn {
            if hash2(x as i32, 12) < 0.8 {
                draw::dot(grid, x, mid);
                draw::dot(grid, x, (mid + 1).min(h - 1));
                let glow = 0.1 + 0.25 * hash3(x as i32, 13, (ctx.time * 4.0) as i32);
                let _ = grid.set_cell_color(x / 2, mid / 4, sample_tint(glow));
            }
        }
        // Spark cluster at the burn point.
        let slot = (ctx.time * 12.0) as i32;
        for s in 0..10i32 {
            if hash3(s, 1, slot) < 0.6 {
                let sx = burn as i32 + (hash3(s, 2, slot) * 7.0) as i32 - 3;
                let sy = mid as i32 + (hash3(s, 3, slot) * 7.0) as i32 - 3;
                draw::dot_i(grid, sx, sy);
                if sx >= 0 && sy >= 0 {
                    let _ = grid.set_cell_color(sx as usize / 2, sy as usize / 4, HEAT_GOLD);
                }
            }
        }
        if burn < w {
            let _ = grid.set_cell_color(burn / 2, mid / 4, HEAT_WHITE);
        }
        // Smoke wisps rising over the burnt stretch.
        for x in (0..burn).step_by(4) {
            let age = (burn - x) as f32 / w.max(1) as f32;
            let lift = (age * 8.0 + (ctx.time * TAU * 0.25 + x as f32 * 0.4).sin() * 1.5) as usize;
            let y = mid.saturating_sub(1 + lift.min(mid));
            if hash2(x as i32, 8) < 0.6 {
                draw::dot(grid, x, y);
                let gray = (150.0 - 90.0 * age) as u8;
                let _ = grid.set_cell_color(x / 2, y / 4, Color::rgb(gray, gray, gray));
            }
        }
        Ok(())
    }
}

/// Flame surges past the fill line and retracts, breathing like a backdraft.
struct Backdraft;
impl ProgressStyle for Backdraft {
    fn name(&self) -> &str {
        "backdraft"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A fill that surges and retracts like a backdraft"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let base = ctx.eased * w as f32;
        // The surge: a breathing overshoot ahead of the true fill.
        let surge = (0.5 + 0.5 * (ctx.time * TAU * 0.5).sin()) * w as f32 * 0.08;
        let slot = (ctx.time * 10.0) as i32;
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32;
                if fx < base {
                    draw::dot(grid, x, y);
                } else if fx < base + surge {
                    // Surge zone: ragged, flickering flame.
                    let chance = 1.0 - (fx - base) / surge.max(0.5);
                    if hash3(x as i32, y as i32, slot) < chance * 0.8 {
                        draw::dot(grid, x, y);
                    }
                }
            }
        }
        Ok(())
    }
}

/// A blazing orb rides the bar, corona spikes wheeling around it.
struct SolarFlare;
impl ProgressStyle for SolarFlare {
    fn name(&self) -> &str {
        "solar-flare"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "A blazing orb with a plasma trail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h as f32 / 2.0;
        let cx = ctx.eased * (w as f32 - 4.0) + 2.0;
        // Plasma trail back to the start.
        for x in 0..(cx as usize) {
            let fade = 1.0 - (cx - x as f32) / w.max(1) as f32 * 1.6;
            if fade > 0.0 {
                draw::dot(grid, x, mid as usize);
                if fade > 0.45 {
                    draw::dot(grid, x, (mid as usize).saturating_sub(1));
                    draw::dot(grid, x, (mid as usize + 1).min(h - 1));
                }
                let _ = grid.set_cell_color(x / 2, mid as usize / 4, sample_tint(fade.max(0.05)));
            }
        }
        // The orb: a filled disc.
        let r = (h as f32 / 3.2).max(1.5);
        let (x0, x1) = ((cx - r) as i32, (cx + r) as i32);
        for x in x0..=x1 {
            for y in 0..h as i32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - mid;
                if dx * dx + dy * dy <= r * r {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        if cx >= 0.0 {
            let _ = grid.set_cell_color(cx as usize / 2, mid as usize / 4, HEAT_WHITE);
        }
        // Corona spikes wheeling with time.
        for s in 0..6 {
            let ang = ctx.time * TAU * 0.25 + s as f32 * TAU / 6.0;
            let sx = cx + ang.cos() * (r + 2.0);
            let sy = mid + ang.sin() * (r + 1.0);
            draw::dot_i(grid, sx as i32, sy as i32);
            if sx >= 0.0 && sy >= 0.0 && (sx as usize) < w && (sy as usize) < h {
                let _ = grid.set_cell_color(sx as usize / 2, sy as usize / 4, HEAT_GOLD);
            }
        }
        Ok(())
    }
}

/// Feathered wing layers build the bar; at full it bursts into flame.
struct Phoenix;
impl ProgressStyle for Phoenix {
    fn name(&self) -> &str {
        "phoenix"
    }
    fn theme(&self) -> &str {
        "inferno"
    }
    fn describe(&self) -> &str {
        "Feathered fire that bursts at one hundred percent"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // Feathered body: overlapping arcs rising toward the leading edge.
        for x in 0..filled {
            let t = x as f32 / w.max(1) as f32;
            let feather = (x as f32 * 0.5 + ctx.time * TAU * 0.25).sin() * 1.5;
            let height = (h as f32 * (0.35 + 0.4 * t) + feather) as usize;
            for y in h.saturating_sub(height.max(1))..h {
                if (y + x / 6) % 4 != 3 {
                    draw::dot(grid, x, y);
                }
            }
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, sample_tint(0.3 + 0.6 * t));
        }
        // The rebirth burst at (nearly) full.
        if ctx.progress > 0.94 {
            let strength = (ctx.progress - 0.94) / 0.06;
            let cx = filled.min(w.saturating_sub(1)) as f32;
            let cy = h as f32 / 2.0;
            for ray in 0..14 {
                let ang = ray as f32 * TAU / 14.0 + ctx.time * TAU * 0.25;
                let len = strength * h as f32 * (0.6 + 0.4 * hash2(ray, 3));
                for k in 0..len as i32 {
                    let px = cx + ang.cos() * k as f32 * 1.6;
                    let py = cy + ang.sin() * k as f32 * 0.8;
                    draw::dot_i(grid, px as i32, py as i32);
                    if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                        let _ = grid.set_cell_color(
                            px as usize / 2,
                            py as usize / 4,
                            sample_tint(1.0 - k as f32 / len.max(1.0)),
                        );
                    }
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
    let styles = progress::styles::inferno::styles();
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
