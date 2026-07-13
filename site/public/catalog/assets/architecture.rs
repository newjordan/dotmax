//! `architecture` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O architecture.rs && ./architecture [style-name]
//! ```

const DEFAULT_STYLE: &str = "skyscraper";

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
    pub mod architecture {
//! Architecture / construction progress bars.
//!
//! Twelve structurally distinct styles, each animating a different building
//! form or construction activity:
//!
//! - `skyscraper`       — floors stack floor-by-floor, crane swings with time
//! - `suspension-bridge`— towers rise, catenary cable spans, suspenders drop
//! - `gothic-arch`      — pointed arches spring upward, rose window blooms
//! - `brick-wall`       — running-bond masonry laid course by course, trowel sweeps
//! - `pyramid`          — stepped stone courses narrow toward the apex
//! - `scaffolding`      — horizontal planks + vertical poles erect around a form
//! - `geodesic-dome`    — triangulated hemisphere wireframe assembles by arc
//! - `spiral-staircase` — perspective helix climbs with progress
//! - `roman-aqueduct`   — round arches march across from left to right
//! - `classical-column` — fluted shaft grows, capital blooms at the top
//! - `blueprint`        — a drafting pen sweeps lines onto graph paper
//! - `tower-crane`      — mast rises, jib extends, load swings on a cable

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — sandstone into slate. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(214, 184, 142);
const TINT_END: Color = Color::rgb(122, 144, 176);

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

/// All styles in the `architecture` theme.
///
/// Returns twelve `Box<dyn ProgressStyle>` values, each implementing a
/// structurally distinct architecture/construction scene in braille dot space.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Skyscraper),
        Box::new(Tinted(SuspensionBridge)),
        Box::new(Tinted(GothicArch)),
        Box::new(BrickWall),
        Box::new(Pyramid),
        Box::new(Tinted(Scaffolding)),
        Box::new(Tinted(GeodesicDome)),
        Box::new(Tinted(SpiralStaircase)),
        Box::new(Tinted(RomanAqueduct)),
        Box::new(Tinted(ClassicalColumn)),
        Box::new(Blueprint),
        Box::new(TowerCrane),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Bresenham-style line between two dot-space points using `draw::dot_i`.
fn line_dots(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a quarter-circle arc (quadrant qx,qy in {-1,+1}) centred at (cx,cy),
/// radius r in dot units, stepped every `step` degrees.
fn arc_quarter(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, angle_start: f32, angle_end: f32) {
    if r < 1.0 {
        return;
    }
    let steps = ((r * 2.0) as usize).max(8);
    let mut prev: Option<(i32, i32)> = None;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = angle_start + (angle_end - angle_start) * t;
        let x = (cx + r * angle.cos()).round() as i32;
        let y = (cy - r * angle.sin()).round() as i32; // screen y flipped
        if let Some((px, py)) = prev {
            line_dots(grid, px, py, x, y);
        }
        prev = Some((x, y));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Skyscraper
// ─────────────────────────────────────────────────────────────────────────────

/// Floors stack from the ground up as progress rises; a crane jib swings with time.
struct Skyscraper;
impl ProgressStyle for Skyscraper {
    fn name(&self) -> &str {
        "skyscraper"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Skyscraper rises floor by floor; rooftop crane jib swings with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Building occupies center third of the width.
        let bld_w = (dw / 3).max(2);
        let bld_x0 = (dw / 2).saturating_sub(bld_w / 2);
        let bld_x1 = (bld_x0 + bld_w).min(dw - 1);

        // Number of floors to draw.
        let max_floors = dh.saturating_sub(2).max(1);
        let floors = (ctx.eased * max_floors as f32).round() as usize;

        // Draw floors from bottom up.
        for f in 0..floors {
            let y = dh.saturating_sub(1 + f);
            draw::hline(grid, bld_x0, bld_x1, y);
            // Side walls every other floor line.
            if f % 2 == 0 {
                draw::dot(grid, bld_x0, y);
                draw::dot(grid, bld_x1, y);
            }
        }

        // Foundation baseline.
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        // Crane: vertical mast above the building top.
        if floors >= max_floors.saturating_sub(1) || floors > 0 {
            let mast_top_y = dh.saturating_sub(floors + 2).max(0) as i32;
            let mast_x = (bld_x1 as i32).min(dw as i32 - 1);
            let building_top_y = dh.saturating_sub(floors + 1).max(0) as i32;
            // Mast (vertical post).
            for y in mast_top_y..=building_top_y {
                draw::dot_i(grid, mast_x, y);
            }
            // Jib swings: angle oscillates with time.
            let jib_len = ((dw as i32 - mast_x).max(2)).min(dw as i32 / 3);
            let swing = (ctx.time * 0.7).sin() * 0.3; // radians, small arc
            let jib_angle = PI * 0.5 + swing; // mostly horizontal left
            let jib_x1 = mast_x - (jib_len as f32 * jib_angle.cos()).round() as i32;
            let jib_y1 = mast_top_y + (jib_len as f32 * jib_angle.sin() * 0.25).round() as i32;
            line_dots(grid, mast_x, mast_top_y, jib_x1, jib_y1);
            // Counter-jib goes right.
            let cj_x = mast_x + (jib_len as i32 / 3);
            line_dots(grid, mast_x, mast_top_y, cj_x, mast_top_y);
            // Hook cable hangs from jib tip.
            let cable_len = (dh as i32 / 4).max(1);
            let load_y = (jib_y1 + cable_len).min(dh as i32 - 1);
            line_dots(grid, jib_x1, jib_y1, jib_x1, load_y);
            // Load block at cable bottom.
            draw::dot_i(grid, jib_x1 - 1, load_y);
            draw::dot_i(grid, jib_x1 + 1, load_y);
        }

        // Tint floors.
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx2 in 0..filled_cells.min(cw) {
            let t = if cw <= 1 { 0.5 } else { cx2 as f32 / cw as f32 };
            let color = ctx.palette.sample(t);
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Suspension Bridge
// ─────────────────────────────────────────────────────────────────────────────

/// Two towers rise, a catenary main cable spans between them, suspender cables drop.
struct SuspensionBridge;
impl ProgressStyle for SuspensionBridge {
    fn name(&self) -> &str {
        "suspension-bridge"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Twin towers rise, catenary cable spans, vertical suspenders hang down"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 2 {
            return Ok(());
        }

        let road_y = dh.saturating_sub(2);
        let tower_h = ((dh as f32 * 0.75) as usize).max(2);
        let tower_top_y = road_y.saturating_sub(tower_h);

        // Towers appear when progress passes 0.15 and 0.85.
        let left_tower_x = dw / 5;
        let right_tower_x = dw - dw / 5;

        // Roadway deck.
        draw::hline(grid, 0, dw.saturating_sub(1), road_y);

        // Left tower.
        if ctx.eased > 0.05 {
            let p = ((ctx.eased - 0.05) / 0.3).min(1.0);
            let h = (p * tower_h as f32).round() as usize;
            let ty0 = road_y.saturating_sub(h);
            draw::vline(grid, left_tower_x, ty0, road_y);
            draw::vline(grid, left_tower_x + 1, ty0, road_y);
            // Tower top crossbar.
            if h >= tower_h {
                draw::hline(
                    grid,
                    left_tower_x.saturating_sub(1),
                    left_tower_x + 2,
                    tower_top_y,
                );
            }
        }

        // Right tower.
        if ctx.eased > 0.5 {
            let p = ((ctx.eased - 0.5) / 0.3).min(1.0);
            let h = (p * tower_h as f32).round() as usize;
            let ty0 = road_y.saturating_sub(h);
            draw::vline(grid, right_tower_x, ty0, road_y);
            draw::vline(grid, right_tower_x + 1, ty0, road_y);
            if h >= tower_h {
                draw::hline(
                    grid,
                    right_tower_x.saturating_sub(1),
                    right_tower_x + 2,
                    tower_top_y,
                );
            }
        }

        // Main catenary cable: drawn when both towers are substantially up.
        if ctx.eased > 0.7 {
            let p = ((ctx.eased - 0.7) / 0.2).min(1.0);
            let sag = (tower_h as f32 * 0.4).max(1.0); // catenary dip at midspan
            let x0 = left_tower_x as f32;
            let x1 = right_tower_x as f32;
            let span_dots = ((x1 - x0) * p).max(1.0) as usize;
            for i in 0..=span_dots {
                let t = i as f32 / span_dots.max(1) as f32;
                let cx = x0 + t * (x1 - x0) * p;
                // Catenary shape: y = sag * cosh(k*(t-0.5)) / cosh(0.5k)
                // Approximated with parabola for simplicity.
                let cy_offset = sag * (4.0 * t * (1.0 - t));
                let cy = tower_top_y as f32 + cy_offset;
                draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

                // Suspender every ~6 dots.
                if i % 6 == 0 {
                    let sx = cx.round() as i32;
                    let sy_top = cy.round() as i32;
                    let sy_bot = road_y as i32;
                    for sy in sy_top..=sy_bot {
                        draw::dot_i(grid, sx, sy);
                    }
                }
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Gothic Arch / Cathedral
// ─────────────────────────────────────────────────────────────────────────────

/// Pointed Gothic arches spring upward; a rose window blooms at the top.
struct GothicArch;
impl ProgressStyle for GothicArch {
    fn name(&self) -> &str {
        "gothic-arch"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Pointed Gothic arches spring upward; rose window blooms at the apex"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }

        let base_y = dh.saturating_sub(1) as i32;
        // How many arches to draw based on width.
        let arch_count = ((dw / 10).max(1)).min(5);
        let arch_slot_w = dw / arch_count;

        for a in 0..arch_count {
            let ax_center = (a * arch_slot_w + arch_slot_w / 2) as f32;
            let arch_w_half = (arch_slot_w / 2).saturating_sub(1) as f32;
            let arch_top_y = (dh as f32 * (1.0 - ctx.eased) + 1.0) as i32;

            // Gothic pointed arch: two circular arcs meeting at a point.
            // Left arc: center offset right, arc from base-left to apex.
            // Right arc: center offset left, arc from base-right to apex.
            let foot_l = ax_center - arch_w_half;
            let foot_r = ax_center + arch_w_half;
            let apex_y = arch_top_y.max(0);

            // Arc radius roughly equals the arch width.
            let r = arch_w_half.max(1.0);

            // Left half-arch: sweep from foot_l up-right to apex.
            let steps = (r * 4.0) as usize + 4;
            let mut prev: Option<(i32, i32)> = None;
            for i in 0..=steps {
                let t = i as f32 / steps.max(1) as f32;
                // Interpolate from foot left to apex.
                let px = foot_l + t * (ax_center - foot_l);
                // Pointed arch Y: use sine for the springy curve.
                let raw_y = base_y as f32 - (t.sin() * (base_y - apex_y).abs() as f32 * t.sqrt());
                let py = raw_y.round() as i32;
                if let Some((ppx, ppy)) = prev {
                    line_dots(grid, ppx, ppy, px.round() as i32, py);
                }
                prev = Some((px.round() as i32, py));
            }
            // Right half-arch.
            prev = None;
            for i in 0..=steps {
                let t = i as f32 / steps.max(1) as f32;
                let px = foot_r - t * (foot_r - ax_center);
                let raw_y = base_y as f32 - (t.sin() * (base_y - apex_y).abs() as f32 * t.sqrt());
                let py = raw_y.round() as i32;
                if let Some((ppx, ppy)) = prev {
                    line_dots(grid, ppx, ppy, px.round() as i32, py);
                }
                prev = Some((px.round() as i32, py));
            }

            // Column shafts on left and right feet.
            draw::dot_i(grid, foot_l as i32, base_y);
            draw::dot_i(grid, foot_r as i32, base_y);
        }

        // Rose window: a small circle at the top center when near complete.
        if ctx.eased > 0.7 && dw >= 6 {
            let rose_p = ((ctx.eased - 0.7) / 0.3).min(1.0);
            let cx = (dw / 2) as f32;
            let cy = (dh as f32 * 0.15).max(2.0);
            let r = (dw as f32 * 0.07 * rose_p).max(1.0);
            // Outer ring.
            let spokes = 8usize;
            for i in 0..spokes {
                let angle = 2.0 * PI * i as f32 / spokes as f32 + ctx.time * 0.3;
                let x = (cx + r * angle.cos()).round() as i32;
                let y = (cy + r * angle.sin()).round() as i32;
                draw::dot_i(grid, x, y);
                // Spoke.
                line_dots(grid, cx.round() as i32, cy.round() as i32, x, y);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Brick Wall
// ─────────────────────────────────────────────────────────────────────────────

/// Running-bond masonry laid course by course; trowel sweeps along each course.
struct BrickWall;
impl ProgressStyle for BrickWall {
    fn name(&self) -> &str {
        "brick-wall"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Running-bond brick courses laid row by row; trowel sweeps each course"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Each cell row is one brick course. Courses fill bottom-up.
        let total_courses = ch;
        let courses_done_f = ctx.eased * total_courses as f32;
        let courses_done = courses_done_f.floor() as usize;
        // Fractional course in progress.
        let partial_frac = courses_done_f.fract();

        // Brick width in cells: ~4 wide, 1 tall (one row = one brick height).
        let brick_w = (cw / 4).max(1);

        for course in 0..courses_done.min(total_courses) {
            let cell_y = total_courses.saturating_sub(1 + course);
            // Running bond: odd courses offset by half brick.
            let offset = if course % 2 == 1 { brick_w / 2 } else { 0 };

            let mut x = 0usize;
            while x < cw {
                // Mortar gap is just the cell boundary — use block glyphs.
                let bx = x;
                let inner = brick_w.saturating_sub(1);
                for bxi in 0..inner {
                    if bx + bxi < cw {
                        draw::glyph(grid, (bx + bxi + offset) % cw, cell_y, '█');
                    }
                }
                x += brick_w;
            }

            // Mortar line (top of each course): thin hline at top dot of cell.
            // The glyph rows cover it; mark with shade for mortar texture.
            if course > 0 {
                let mortar_y = total_courses.saturating_sub(1 + course);
                // Place ░ for mortar joints between bricks (at the vertical seam dots).
                let mut mx = offset;
                while mx < cw {
                    if mx > 0 {
                        draw::glyph(grid, mx % cw, mortar_y, '▏');
                    }
                    mx += brick_w;
                }
            }
        }

        // Partial course: trowel sweeps.
        if courses_done < total_courses {
            let cell_y = total_courses.saturating_sub(1 + courses_done);
            let trowel_x = (partial_frac * cw as f32) as usize;
            for bxi in 0..trowel_x.min(cw) {
                draw::shade(grid, bxi, cell_y, 2); // ▒ for freshly-laid course
            }
            // Trowel leading edge marker.
            if trowel_x < cw {
                draw::glyph(grid, trowel_x, cell_y, '▌');
            }
        }

        // Tint: warm brick orange-red gradient.
        let brick_start = crate::Color::rgb(200, 80, 40);
        let brick_end = crate::Color::rgb(230, 140, 80);
        for cx2 in 0..cw {
            let t = if cw <= 1 {
                0.5
            } else {
                cx2 as f32 / (cw - 1) as f32
            };
            let color = {
                let r =
                    (brick_start.r as f32 + (brick_end.r as f32 - brick_start.r as f32) * t) as u8;
                let g =
                    (brick_start.g as f32 + (brick_end.g as f32 - brick_start.g as f32) * t) as u8;
                let b =
                    (brick_start.b as f32 + (brick_end.b as f32 - brick_start.b as f32) * t) as u8;
                crate::Color::rgb(r, g, b)
            };
            let max_course_y = total_courses.saturating_sub(courses_done.min(total_courses));
            for cy2 in max_course_y..ch {
                draw::tint_row(grid, cy2, cx2, cx2, color);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pyramid
// ─────────────────────────────────────────────────────────────────────────────

/// Stepped stone courses narrow upward; the pyramid grows from base to apex.
struct Pyramid;
impl ProgressStyle for Pyramid {
    fn name(&self) -> &str {
        "pyramid"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Pyramid rises course by course; each step narrower than the last"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 1 {
            return Ok(());
        }

        let max_courses = dh;
        let courses = (ctx.eased * max_courses as f32).ceil() as usize;
        let cx = dw / 2;

        for c in 0..courses.min(max_courses) {
            let y = dh.saturating_sub(1 + c);
            // Width tapers linearly: full at base, 1 dot at apex.
            let half_w = ((max_courses - c) * (dw / 2)) / max_courses.max(1);
            let x0 = cx.saturating_sub(half_w);
            let x1 = (cx + half_w).min(dw.saturating_sub(1));

            // Course line (step tread).
            draw::hline(grid, x0, x1, y);
            // Side walls (step risers).
            if c > 0 {
                draw::dot(grid, x0, y);
                draw::dot(grid, x1, y);
            }
        }

        // Tint from base (dark) to apex (bright).
        let (cw, ch) = grid.dimensions();
        for cy2 in 0..ch {
            let t = cy2 as f32 / ch.max(1) as f32;
            let color = ctx.palette.sample(1.0 - t);
            for cx2 in 0..cw {
                draw::tint_row(grid, cy2, cx2, cx2, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Scaffolding
// ─────────────────────────────────────────────────────────────────────────────

/// Vertical poles and horizontal planks erect around a central building form.
struct Scaffolding;
impl ProgressStyle for Scaffolding {
    fn name(&self) -> &str {
        "scaffolding"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Scaffold poles and planks erect around a building; cross-braces appear last"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 2 {
            return Ok(());
        }

        // Central building outline (always present, faint — just the rect).
        let bld_x0 = dw / 4;
        let bld_x1 = dw - dw / 4;
        let bld_y0 = dh / 4;
        let bld_y1 = dh.saturating_sub(1);
        draw::rect_outline(
            grid,
            bld_x0,
            bld_y0,
            bld_x1.saturating_sub(bld_x0),
            bld_y1.saturating_sub(bld_y0),
        );

        // Scaffold poles: progress reveals them left-to-right, inside-out.
        // Two poles at left and right of scaffold.
        let pole_left = bld_x0.saturating_sub(2);
        let pole_right = (bld_x1 + 2).min(dw.saturating_sub(1));

        // Pole heights grow with progress (bottom up).
        let pole_top_y = (dh as f32 * (1.0 - ctx.eased)).round() as usize;

        if ctx.eased > 0.0 {
            draw::vline(grid, pole_left, pole_top_y, dh.saturating_sub(1));
            draw::vline(grid, pole_right, pole_top_y, dh.saturating_sub(1));
        }

        // Horizontal planks: appear at intervals as progress advances.
        let plank_spacing = (dh / 4).max(2);
        let plank_count = dh / plank_spacing;
        let planks_shown = (ctx.eased * plank_count as f32).round() as usize;
        for p in 0..planks_shown.min(plank_count) {
            let py = dh.saturating_sub(1 + p * plank_spacing);
            draw::hline(grid, pole_left, pole_right, py);
        }

        // Cross-braces appear in the last 30% of progress.
        if ctx.eased > 0.7 {
            let brace_p = (ctx.eased - 0.7) / 0.3;
            let brace_top = (dh as f32 * (1.0 - brace_p)).round() as i32;
            // Left brace: ╲ from pole top-left to mid-right.
            let mid_x = ((pole_left + pole_right) / 2) as i32;
            let bot_y = dh as i32 - 1;
            line_dots(grid, pole_left as i32, brace_top, mid_x, bot_y);
            line_dots(grid, pole_right as i32, brace_top, mid_x, bot_y);
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Geodesic Dome
// ─────────────────────────────────────────────────────────────────────────────

/// Triangulated hemisphere wireframe assembles arc by arc as progress rises.
struct GeodesicDome;
impl ProgressStyle for GeodesicDome {
    fn name(&self) -> &str {
        "geodesic-dome"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Geodesic dome hemisphere assembles; triangulated arcs appear with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 2 {
            return Ok(());
        }

        let cx = dw as f32 / 2.0;
        let cy = dh as f32 - 1.0; // dome sits on the baseline
        let r = ((dw as f32 / 2.0) - 1.0).min((dh as f32) - 1.0).max(1.0);

        // Baseline.
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        // Main hemisphere arc (always drawn first).
        arc_quarter(grid, cx, cy, r, 0.0, PI);

        // Latitude rings: horizontal arcs at fractional heights.
        let lat_count = 4usize;
        let lats_shown = (ctx.eased * lat_count as f32).round() as usize;
        for li in 0..lats_shown.min(lat_count) {
            let frac = (li + 1) as f32 / (lat_count + 1) as f32;
            let lat_y_offset = r * frac; // screen offset from baseline upward
            let lat_r = (r * r - lat_y_offset * lat_y_offset).sqrt().max(0.5);
            let lat_cy = cy - lat_y_offset;
            // Draw the full horizontal arc at this latitude.
            arc_quarter(grid, cx, lat_cy, lat_r, 0.0, PI);
        }

        // Longitude meridian lines: spokes from base to apex.
        let meridian_count = 8usize;
        let meridians_shown = (ctx.eased * meridian_count as f32).round() as usize;
        for mi in 0..meridians_shown.min(meridian_count) {
            let angle = PI * mi as f32 / meridian_count as f32;
            let x0 = (cx + r * angle.cos()).round() as i32;
            let y0 = cy.round() as i32;
            let apex_x = cx.round() as i32;
            let apex_y = (cy - r).round() as i32;
            line_dots(grid, x0, y0, apex_x, apex_y);
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Spiral Staircase
// ─────────────────────────────────────────────────────────────────────────────

/// A perspective helix climbs with progress; treads project at diminishing depth.
struct SpiralStaircase;
impl ProgressStyle for SpiralStaircase {
    fn name(&self) -> &str {
        "spiral-staircase"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Perspective spiral staircase climbs; treads shrink with height"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 4 {
            return Ok(());
        }

        let cx = dw as f32 / 2.0;
        // Number of turns in the helix.
        let turns = 2.0f32;
        let total_steps = (turns * 12.0) as usize;
        let steps_shown = (ctx.eased * total_steps as f32).round() as usize;

        // Animate rotation with time.
        let rot = ctx.time * 0.4;

        for s in 0..steps_shown.min(total_steps) {
            let t = s as f32 / total_steps as f32;
            let angle = 2.0 * PI * turns * t + rot;
            // Vertical position: top at t=1, bottom at t=0.
            let y_dot = (dh as f32 * (1.0 - t * 0.9)) as i32;
            // Horizontal radius shrinks slightly with height (perspective).
            let radius = (dw as f32 * 0.35) * (1.0 - t * 0.3);
            let x_dot = (cx + radius * angle.cos()).round() as i32;

            // Tread: short horizontal line at this step's position.
            let tread_len = (radius * 0.6).round() as i32;
            for tx in -tread_len..=tread_len {
                draw::dot_i(grid, x_dot + tx, y_dot);
            }
            // Riser dot.
            draw::dot_i(grid, x_dot, y_dot);
            draw::dot_i(grid, x_dot, y_dot + 1);

            // Central newel post.
            draw::dot_i(grid, cx.round() as i32, y_dot);
        }

        // Central post full height.
        for y in 0..dh {
            draw::dot(grid, dw / 2, y);
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Roman Aqueduct
// ─────────────────────────────────────────────────────────────────────────────

/// Round arches march left to right; the water channel deck appears last.
struct RomanAqueduct;
impl ProgressStyle for RomanAqueduct {
    fn name(&self) -> &str {
        "roman-aqueduct"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Roman aqueduct arches march across; water channel appears at completion"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 3 {
            return Ok(());
        }

        let base_y = dh.saturating_sub(1);
        let arch_h = (dh as f32 * 0.6).round() as usize;
        let arch_w = (dw / 5).max(4); // width of each arch opening in dots
        let pier_w = (arch_w / 4).max(1);
        let arch_count = dw / (arch_w + pier_w);

        // How many arches revealed.
        let arches_shown = (ctx.eased * arch_count as f32).ceil() as usize;

        let mut x = 0usize;
        for a in 0..arches_shown.min(arch_count) {
            let _ = a;
            // Pier on left.
            for py in (base_y.saturating_sub(arch_h))..=base_y {
                draw::dot(grid, x, py);
                if pier_w > 1 {
                    draw::dot(grid, x + pier_w.saturating_sub(1), py);
                }
            }
            x += pier_w;

            // Semicircular arch.
            let arch_cx = (x + arch_w / 2) as f32;
            let arch_cy = base_y.saturating_sub(arch_h / 2) as f32;
            let arch_r = (arch_h / 2).max(1) as f32;
            arc_quarter(grid, arch_cx, arch_cy, arch_r, 0.0, PI);

            x += arch_w;
        }

        // Rightmost pier.
        if arches_shown > 0 && x < dw {
            for py in (base_y.saturating_sub(arch_h))..=base_y {
                draw::dot(grid, x.min(dw.saturating_sub(1)), py);
            }
        }

        // Water channel deck on top when complete.
        if ctx.eased > 0.9 {
            let deck_y = base_y.saturating_sub(arch_h);
            draw::hline(grid, 0, dw.saturating_sub(1), deck_y);
            // Water shimmer (animated dots).
            let shimmer_x = ((ctx.time * 3.0).sin() * dw as f32 * 0.5 + dw as f32 * 0.5) as usize;
            draw::dot(
                grid,
                shimmer_x.min(dw.saturating_sub(1)),
                deck_y.saturating_sub(1),
            );
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Classical Column
// ─────────────────────────────────────────────────────────────────────────────

/// A fluted Doric column shaft grows upward; the capital blooms at the top.
struct ClassicalColumn;
impl ProgressStyle for ClassicalColumn {
    fn name(&self) -> &str {
        "classical-column"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Classical column shaft rises with flutes; capital and entablature bloom last"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 2 {
            return Ok(());
        }

        let cx = dw / 2;
        let shaft_half = (dw / 6).max(1);
        let x0 = cx.saturating_sub(shaft_half);
        let x1 = (cx + shaft_half).min(dw.saturating_sub(1));
        let base_y = dh.saturating_sub(1);
        let shaft_h = ((dh as f32 * 0.75) as usize).max(2);
        let shaft_top_y = base_y.saturating_sub(shaft_h);

        // Base / stylobate (always present).
        draw::hline(grid, 0, dw.saturating_sub(1), base_y);
        draw::hline(
            grid,
            x0.saturating_sub(1),
            (x1 + 1).min(dw.saturating_sub(1)),
            base_y.saturating_sub(1),
        );

        // Shaft grows upward.
        let shaft_drawn = (ctx.eased * shaft_h as f32).round() as usize;
        let shaft_drawn_y0 = base_y.saturating_sub(shaft_drawn);

        // Shaft outline: two vertical sides.
        draw::vline(grid, x0, shaft_drawn_y0, base_y.saturating_sub(1));
        draw::vline(grid, x1, shaft_drawn_y0, base_y.saturating_sub(1));

        // Flutes: vertical lines inside the shaft.
        let flute_count = ((x1 - x0) / 2).max(1);
        for f in 0..flute_count {
            let fx = x0 + 1 + f * 2;
            if fx < x1 {
                // Flute is dotted every 2 to suggest engraving.
                let mut fy = shaft_drawn_y0;
                while fy <= base_y.saturating_sub(1) {
                    draw::dot(grid, fx, fy);
                    fy += 2;
                }
            }
        }

        // Capital: only appears in the top 25% of progress.
        if ctx.eased > 0.75 {
            let cap_p = (ctx.eased - 0.75) / 0.25;
            let capital_top = shaft_top_y.saturating_sub((dh as f32 * 0.15 * cap_p) as usize);
            // Echinus (curved moulding).
            let spread = (shaft_half as f32 * cap_p * 0.5) as usize;
            let cap_x0 = x0.saturating_sub(spread);
            let cap_x1 = (x1 + spread).min(dw.saturating_sub(1));
            draw::hline(grid, cap_x0, cap_x1, shaft_top_y);
            // Abacus (flat slab).
            draw::hline(
                grid,
                cap_x0.saturating_sub(1),
                (cap_x1 + 1).min(dw.saturating_sub(1)),
                capital_top,
            );
            // Entablature (beam).
            if cap_p > 0.8 {
                draw::hline(grid, 0, dw.saturating_sub(1), capital_top.saturating_sub(1));
                draw::hline(grid, 0, dw.saturating_sub(1), capital_top.saturating_sub(2));
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Blueprint
// ─────────────────────────────────────────────────────────────────────────────

/// A drafting pen sweeps technical lines across graph paper; grid appears first.
struct Blueprint;
impl ProgressStyle for Blueprint {
    fn name(&self) -> &str {
        "blueprint"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Blueprint drafted: graph grid first, then walls, windows, and dimensions"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 4 {
            return Ok(());
        }

        // Phase 1 (0–0.2): faint grid lines.
        if ctx.eased > 0.0 {
            let grid_spacing = 4usize;
            let grid_p = (ctx.eased / 0.2).min(1.0);
            let grid_cols = ((dw / grid_spacing) as f32 * grid_p) as usize;
            let grid_rows = ((dh / grid_spacing) as f32 * grid_p) as usize;
            for gx in 0..=grid_cols {
                let x = gx * grid_spacing;
                // Every other dot to keep it light.
                let mut y = 0usize;
                while y < dh {
                    draw::dot(grid, x.min(dw.saturating_sub(1)), y);
                    y += 2;
                }
            }
            for gy in 0..=grid_rows {
                let y = gy * grid_spacing;
                let mut x = 0usize;
                while x < dw {
                    draw::dot(grid, x, y.min(dh.saturating_sub(1)));
                    x += 2;
                }
            }
        }

        // Phase 2 (0.2–0.6): floor plan walls sweep in (outer rectangle).
        if ctx.eased > 0.2 {
            let wall_p = ((ctx.eased - 0.2) / 0.4).min(1.0);
            let x0 = dw / 8;
            let x1 = dw - dw / 8;
            let y0 = dh / 8;
            let y1 = dh - dh / 8;
            let perim = 2 * ((x1 - x0) + (y1 - y0));
            let drawn = (wall_p * perim as f32) as usize;
            // Walk the perimeter: top, right, bottom, left.
            let top_len = x1 - x0;
            let right_len = y1 - y0;
            let bot_len = x1 - x0;
            let left_len = y1 - y0;
            let mut rem = drawn;
            // Top.
            let seg = rem.min(top_len);
            draw::hline(grid, x0, x0 + seg, y0);
            rem = rem.saturating_sub(seg);
            // Right.
            let seg = rem.min(right_len);
            draw::vline(grid, x1, y0, y0 + seg);
            rem = rem.saturating_sub(seg);
            // Bottom (reversed).
            let seg = rem.min(bot_len);
            draw::hline(grid, x1.saturating_sub(seg), x1, y1);
            rem = rem.saturating_sub(seg);
            // Left (reversed).
            let seg = rem.min(left_len);
            draw::vline(grid, x0, y1.saturating_sub(seg), y1);
        }

        // Phase 3 (0.6–0.85): interior walls and door opening.
        if ctx.eased > 0.6 {
            let int_p = ((ctx.eased - 0.6) / 0.25).min(1.0);
            let mid_x = dw / 2;
            let y0 = dh / 8;
            let y1 = dh - dh / 8;
            let wall_len = (y1 - y0).saturating_sub(4); // door gap
            let drawn = (int_p * wall_len as f32) as usize;
            draw::vline(grid, mid_x, y0, y0 + drawn);
        }

        // Phase 4 (0.85–1.0): dimension lines + pen cursor.
        if ctx.eased > 0.85 {
            let dim_p = (ctx.eased - 0.85) / 0.15;
            // Dimension line along the top.
            let y_dim = (dh / 8).saturating_sub(2).max(0);
            let x_end = (dim_p * dw as f32) as usize;
            draw::hline(grid, 0, x_end.min(dw.saturating_sub(1)), y_dim);
            // Arrow heads.
            draw::dot(grid, 0, y_dim);
            let tail = x_end.min(dw.saturating_sub(1));
            draw::dot(grid, tail, y_dim);
        }

        // Pen cursor: a moving dot at the current draw frontier.
        let pen_x = (ctx.eased * dw as f32).round() as usize;
        let pen_y = ((ctx.time * 1.5).sin() * dh as f32 * 0.1 + dh as f32 * 0.5) as usize;
        draw::dot(
            grid,
            pen_x.min(dw.saturating_sub(1)),
            pen_y.min(dh.saturating_sub(1)),
        );

        // Tint: blueprint blue.
        let (cw, ch) = grid.dimensions();
        let blue = crate::Color::rgb(30, 80, 200);
        let light = crate::Color::rgb(100, 160, 255);
        for cx2 in 0..cw {
            for cy2 in 0..ch {
                let t = cx2 as f32 / cw.max(1) as f32;
                let color = {
                    let r = (blue.r as f32 + (light.r as f32 - blue.r as f32) * t) as u8;
                    let g = (blue.g as f32 + (light.g as f32 - blue.g as f32) * t) as u8;
                    let b = (blue.b as f32 + (light.b as f32 - blue.b as f32) * t) as u8;
                    crate::Color::rgb(r, g, b)
                };
                draw::tint_row(grid, cy2, cx2, cx2, color);
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Tower Crane
// ─────────────────────────────────────────────────────────────────────────────

/// Mast rises, horizontal jib extends, a load hangs from the trolley and swings.
struct TowerCrane;
impl ProgressStyle for TowerCrane {
    fn name(&self) -> &str {
        "tower-crane"
    }
    fn theme(&self) -> &str {
        "architecture"
    }
    fn describe(&self) -> &str {
        "Tower crane mast rises, jib extends, trolley rolls, load swings on a cable"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 4 || dh < 4 {
            return Ok(());
        }

        let base_y = dh.saturating_sub(1);
        // Mast anchored at 1/4 width.
        let mast_x = dw / 4;

        // Mast grows to full height at progress 0.4.
        let mast_p = (ctx.eased / 0.4).min(1.0);
        let full_mast_h = dh.saturating_sub(2);
        let mast_h = (mast_p * full_mast_h as f32).round() as usize;
        let mast_top = base_y.saturating_sub(mast_h);

        draw::vline(grid, mast_x, mast_top, base_y);
        // Mast cross-bracing pattern (diagonal dots).
        let brace_spacing = (dh / 4).max(2);
        let mut by = mast_top;
        while by + brace_spacing <= base_y {
            line_dots(
                grid,
                mast_x as i32 - 1,
                by as i32,
                mast_x as i32 + 1,
                (by + brace_spacing) as i32,
            );
            line_dots(
                grid,
                mast_x as i32 + 1,
                by as i32,
                mast_x as i32 - 1,
                (by + brace_spacing) as i32,
            );
            by += brace_spacing;
        }

        // Jib extends right from mast top at progress 0.3–0.7.
        if ctx.eased > 0.3 {
            let jib_p = ((ctx.eased - 0.3) / 0.4).min(1.0);
            let max_jib = dw.saturating_sub(mast_x + 2);
            let jib_len = (jib_p * max_jib as f32).round() as usize;
            let jib_tip = (mast_x + jib_len).min(dw.saturating_sub(1));
            draw::hline(grid, mast_x, jib_tip, mast_top);

            // Counter-jib extends left.
            let cj_len = (jib_len / 3).max(1);
            let cj_start = mast_x.saturating_sub(cj_len);
            draw::hline(grid, cj_start, mast_x, mast_top);

            // Stays: angled cables from mast top to jib tip and counter-jib end.
            if jib_len > 2 {
                let stay_y = (mast_top + 1).min(base_y);
                line_dots(
                    grid,
                    mast_x as i32,
                    stay_y as i32,
                    jib_tip as i32,
                    mast_top as i32,
                );
                line_dots(
                    grid,
                    mast_x as i32,
                    stay_y as i32,
                    cj_start as i32,
                    mast_top as i32,
                );
            }

            // Trolley rolls along jib with time.
            if ctx.eased > 0.6 && jib_len > 1 {
                let trolley_norm = ((ctx.time * 0.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                let trolley_x = mast_x + (trolley_norm * jib_len as f32) as usize;
                let trolley_x = trolley_x.min(jib_tip);
                // Trolley body.
                draw::dot(grid, trolley_x, mast_top);
                draw::dot(grid, trolley_x, mast_top.saturating_add(1));

                // Load cable hangs from trolley.
                let cable_len = ((dh / 3) as f32 * ctx.eased) as usize;
                let load_y = (mast_top + 2 + cable_len).min(base_y);
                draw::vline(grid, trolley_x, mast_top + 2, load_y);

                // Load block.
                let lx = trolley_x as i32;
                let ly = load_y as i32;
                draw::dot_i(grid, lx - 1, ly);
                draw::dot_i(grid, lx, ly);
                draw::dot_i(grid, lx + 1, ly);
                draw::dot_i(grid, lx - 1, ly + 1);
                draw::dot_i(grid, lx, ly + 1);
                draw::dot_i(grid, lx + 1, ly + 1);
            }
        }

        // Ground baseline.
        draw::hline(grid, 0, dw.saturating_sub(1), base_y);

        // Tint: steel-grey to orange for the crane.
        let (cw, ch) = grid.dimensions();
        let steel = crate::Color::rgb(120, 130, 145);
        let orange = crate::Color::rgb(255, 140, 0);
        for cy2 in 0..ch {
            let t = cy2 as f32 / ch.max(1) as f32;
            let color = {
                let r = (orange.r as f32 + (steel.r as f32 - orange.r as f32) * t) as u8;
                let g = (orange.g as f32 + (steel.g as f32 - orange.g as f32) * t) as u8;
                let b = (orange.b as f32 + (steel.b as f32 - orange.b as f32) * t) as u8;
                crate::Color::rgb(r, g, b)
            };
            for cx2 in 0..cw {
                draw::tint_row(grid, cy2, cx2, cx2, color);
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
    let styles = progress::styles::architecture::styles();
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
