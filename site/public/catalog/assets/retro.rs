//! `retro` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O retro.rs && ./retro [style-name]
//! ```

const DEFAULT_STYLE: &str = "pacman";

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
    pub mod retro {
//! Retro / arcade-gaming progress bars.
//!
//! Ten distinct styles that evoke classic 80s and 90s arcade and home-console
//! aesthetics: Pac-Man pellets, Space Invader waves, Tetris stacking, RPG
//! segmented health bars, cassette reels, 8-bit blocky pixels, CRT scanlines,
//! a growing snake, pinball brick-breaker, and a combo power-up meter.
//!
//! All bars are stateless — animation derives purely from `ctx.time`, and fill
//! extent from `ctx.eased`. Every write goes through `draw::` helpers so
//! out-of-bounds coordinates are silently discarded.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `retro` theme.
///
/// Returns one boxed implementor per style. The vec is in display order —
/// register it with `all_styles` or iterate for a gallery picker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PacMan),
        Box::new(SpaceInvaders),
        Box::new(TetrisStack),
        Box::new(RpgHealthBar),
        Box::new(CassetteReels),
        Box::new(EightBitBlocks),
        Box::new(CrtScanline),
        Box::new(Snake),
        Box::new(PinballBricks),
        Box::new(ComboPower),
    ]
}

// ─── 1. Pac-Man ──────────────────────────────────────────────────────────────

struct PacMan;
impl ProgressStyle for PacMan {
    fn name(&self) -> &str {
        "pacman"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Pac-Man chomps through a row of pellets; eaten count = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let mid_y = h / 2;

        // How many pellets exist total across the track.
        let pellet_count = (w / 6).max(1);
        let pellet_spacing = w / pellet_count.max(1);
        let eaten = (ctx.eased * pellet_count as f32) as usize;

        // Draw uneaten pellets (small 2×2 squares).
        for p in eaten..pellet_count {
            let px = p * pellet_spacing + pellet_spacing / 2;
            draw::dot(grid, px, mid_y);
            draw::dot(grid, px.saturating_sub(1), mid_y);
            draw::dot(grid, px, mid_y.saturating_sub(1));
            draw::dot(grid, px.saturating_sub(1), mid_y.saturating_sub(1));
        }

        // Pac-Man position: just past the last eaten pellet.
        let pac_x = if eaten == 0 {
            0usize
        } else {
            (eaten.saturating_sub(1) * pellet_spacing + pellet_spacing / 2 + 3)
                .min(w.saturating_sub(1))
        };

        // Mouth angle oscillates with time — open/close cycle.
        let chomp = ((ctx.time * 6.0).sin() * 0.5 + 0.5) as f32; // 0..1
        let mouth_angle = chomp * (PI / 3.0); // 0 = fully open, PI/3 = widest

        // Pac-Man body: 5×5 dot circle, clipped by mouth wedge.
        let r = (h / 2).max(2) as i32;
        let cx = pac_x as i32;
        let cy = mid_y as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    // Mouth wedge: suppress dots in the forward-facing cone.
                    let angle = (dy as f32).atan2(dx as f32).abs();
                    if angle > mouth_angle {
                        draw::dot_i(grid, cx + dx, cy + dy);
                    }
                }
            }
        }

        // Tint eaten region yellow-ish, uneaten region dim.
        let (cells_w, cells_h) = grid.dimensions();
        let eaten_cells = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            if eaten_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    eaten_cells.saturating_sub(1),
                    ctx.palette.sample(0.85),
                ); // warm yellow via palette end
            }
        }

        Ok(())
    }
}

// ─── 2. Space Invaders ───────────────────────────────────────────────────────

struct SpaceInvaders;
impl ProgressStyle for SpaceInvaders {
    fn name(&self) -> &str {
        "space-invaders"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "A row of descending invaders; fill = alive count; legs toggle with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);

        let inv_w = 6usize; // dots wide per invader
        let inv_count = (w / inv_w).max(1);
        let alive = (ctx.eased * inv_count as f32).ceil() as usize;
        let leg_frame = ((ctx.time * 4.0) as usize) % 2; // 0 or 1

        for i in 0..alive.min(inv_count) {
            let base_x = i * inv_w;
            let cx = base_x + inv_w / 2; // center x
            let top_y = 0usize;

            // Body: 3-wide block in the upper rows.
            draw::hline(
                grid,
                cx.saturating_sub(1),
                (cx + 1).min(w.saturating_sub(1)),
                top_y + 1,
            );
            draw::hline(
                grid,
                cx.saturating_sub(2),
                (cx + 2).min(w.saturating_sub(1)),
                top_y + 2,
            );
            draw::hline(
                grid,
                cx.saturating_sub(1),
                (cx + 1).min(w.saturating_sub(1)),
                top_y + 3,
            );

            // Eyes: two dots.
            draw::dot(grid, cx.saturating_sub(1), top_y + 2);
            draw::dot(grid, (cx + 1).min(w.saturating_sub(1)), top_y + 2);

            // Antennae.
            if cx >= 2 {
                draw::dot(grid, cx - 2, top_y);
            }
            if cx + 2 < w {
                draw::dot(grid, cx + 2, top_y);
            }

            // Legs: alternate between two frames.
            let leg_y = top_y + 4;
            if leg_y < h {
                if leg_frame == 0 {
                    if cx >= 2 {
                        draw::dot(grid, cx - 2, leg_y);
                    }
                    if cx + 2 < w {
                        draw::dot(grid, cx + 2, leg_y);
                    }
                } else {
                    if cx >= 1 {
                        draw::dot(grid, cx - 1, leg_y);
                    }
                    if cx + 1 < w {
                        draw::dot(grid, cx + 1, leg_y);
                    }
                }
            }
        }

        // Tint alive invaders green.
        let (cells_w, cells_h) = grid.dimensions();
        let alive_cells = (ctx.eased * cells_w as f32).ceil() as usize;
        for cy in 0..cells_h {
            if alive_cells > 0 {
                let end = alive_cells.min(cells_w).saturating_sub(1);
                draw::tint_row(grid, cy, 0, end, ctx.palette.sample(0.4));
            }
        }

        Ok(())
    }
}

// ─── 3. Tetris Stack ─────────────────────────────────────────────────────────

struct TetrisStack;
impl ProgressStyle for TetrisStack {
    fn name(&self) -> &str {
        "tetris-stack"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Tetris blocks stack from the bottom; stack height = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let col_count = (w / 4).max(1);
        let col_w = 4usize;

        // Stack height for each column: vary by column index to create a ragged skyline.
        for col in 0..col_count {
            let phase = (col as f32 / col_count as f32) * PI;
            let col_frac = (ctx.eased + 0.15 * phase.sin()).clamp(0.0, 1.0);
            let stack_h = (col_frac * h as f32) as usize;
            if stack_h == 0 {
                continue;
            }

            let x0 = col * col_w;
            let bw = (col_w - 1).max(1);
            let y0 = h.saturating_sub(stack_h);

            // Draw the stacked column as a filled rect with outline.
            draw::fill_rect(grid, x0, y0, bw, stack_h);

            // Draw block divisions every 4 dots (simulate individual pieces).
            let mut seg_y = y0;
            while seg_y + 4 < y0 + stack_h {
                draw::hline(grid, x0, (x0 + bw).saturating_sub(1), seg_y + 3);
                seg_y += 4;
            }
        }

        // Tint by column using the palette gradient.
        let (_, cells_h) = grid.dimensions();
        let (cells_w, _) = grid.dimensions();
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            let col_frac = ctx.eased;
            let stack_cells = (col_frac * cells_h as f32) as usize;
            let y_start = cells_h.saturating_sub(stack_cells);
            for cy in y_start..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ─── 4. RPG Health Bar ───────────────────────────────────────────────────────

struct RpgHealthBar;
impl ProgressStyle for RpgHealthBar {
    fn name(&self) -> &str {
        "rpg-health"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Chunky segmented HP bar with a slow shine sweep — classic RPG style"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);

        // Outer border.
        draw::rect_outline(grid, 0, 0, w, h);

        // Segment geometry.
        let seg_count = 10usize;
        let inner_w = w.saturating_sub(4);
        let seg_w = (inner_w / seg_count).max(2);
        let gap = 1usize;
        let lit_segs = (ctx.eased * seg_count as f32).round() as usize;

        for s in 0..lit_segs.min(seg_count) {
            let x0 = 2 + s * seg_w;
            let bw = seg_w.saturating_sub(gap).max(1);
            let by = 2usize;
            let bh = h.saturating_sub(4).max(1);
            draw::fill_rect(grid, x0, by, bw, bh);
        }

        // Shine sweep: a bright vertical stripe traveling left→right over time.
        let shine_period = 3.0f32;
        let shine_t = (ctx.time % shine_period) / shine_period;
        let shine_x = (shine_t * inner_w as f32) as usize + 2;
        let shine_w = (inner_w / 8).max(2);
        for dx in 0..shine_w {
            let sx = shine_x + dx;
            if sx < w.saturating_sub(2) {
                // Only shine over filled segs.
                let frac = (sx as f32 - 2.0) / inner_w.max(1) as f32;
                if frac <= ctx.eased {
                    // Thin central bright line.
                    draw::dot(grid, sx, h / 2);
                }
            }
        }

        // Color: green when healthy, red when low.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32).round() as usize;
        let health_color = ctx.palette.sample(ctx.eased);
        for cy in 0..cells_h {
            if filled_cells > 0 {
                draw::tint_row(grid, cy, 0, filled_cells.saturating_sub(1), health_color);
            }
        }

        Ok(())
    }
}

// ─── 5. Cassette Reels ───────────────────────────────────────────────────────

struct CassetteReels;
impl ProgressStyle for CassetteReels {
    fn name(&self) -> &str {
        "cassette-reels"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Two tape reels spin as tape transfers from supply to take-up reel"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let mid_y = (h / 2) as i32;

        // Reel radii: supply shrinks, take-up grows.
        let max_r = ((h / 2).saturating_sub(1)).max(2) as i32;
        let take_r = (1 + (ctx.eased * (max_r - 1) as f32) as i32)
            .max(1)
            .min(max_r);
        let supply_r = (max_r - take_r + 1).max(1).min(max_r);

        let reel_offset = (w as i32 / 4).max(max_r + 1);
        let left_cx = reel_offset;
        let right_cx = w as i32 - reel_offset;

        // Draw reel circles as outlines (supply = left, take-up = right).
        let draw_reel = |grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32, angle: f32| {
            // Circle outline.
            let steps = (2.0 * PI * r as f32 * 1.5) as usize;
            for s in 0..steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                let dx = (a.cos() * r as f32).round() as i32;
                let dy = (a.sin() * r as f32).round() as i32;
                draw::dot_i(grid, cx + dx, cy + dy);
            }
            // Rotating spokes (4 spokes, each 90° apart).
            for spoke in 0..4 {
                let sa = angle + spoke as f32 * PI / 2.0;
                for t in 0..r {
                    let dx = (sa.cos() * t as f32).round() as i32;
                    let dy = (sa.sin() * t as f32).round() as i32;
                    draw::dot_i(grid, cx + dx, cy + dy);
                }
            }
            // Hub dot.
            draw::dot_i(grid, cx, cy);
        };

        let left_angle = ctx.time * 2.5;
        let right_angle = ctx.time * 2.5 + PI; // opposite phase

        draw_reel(grid, left_cx, mid_y, supply_r, left_angle);
        draw_reel(grid, right_cx, mid_y, take_r, right_angle);

        // Tape path: two horizontal lines connecting the reel tangent points.
        let tape_y_top = mid_y - 1;
        let tape_y_bot = mid_y + 1;
        let tape_x0 = (left_cx + supply_r + 1).max(0) as usize;
        let tape_x1 = (right_cx - take_r - 1).max(0) as usize;
        if tape_x0 < tape_x1 {
            draw::hline(grid, tape_x0, tape_x1, tape_y_top.max(0) as usize);
            if tape_y_bot < h as i32 {
                draw::hline(grid, tape_x0, tape_x1, tape_y_bot as usize);
            }
        }

        // Tint reels.
        let (cells_w, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let left_end = (cells_w / 4).min(cells_w.saturating_sub(1));
            let right_start = (cells_w * 3 / 4).min(cells_w.saturating_sub(1));
            draw::tint_row(grid, cy, 0, left_end, ctx.palette.sample(1.0 - ctx.eased));
            draw::tint_row(
                grid,
                cy,
                right_start,
                cells_w.saturating_sub(1),
                ctx.palette.sample(ctx.eased),
            );
        }

        Ok(())
    }
}

// ─── 6. 8-bit Blocky Pixels ──────────────────────────────────────────────────

struct EightBitBlocks;
impl ProgressStyle for EightBitBlocks {
    fn name(&self) -> &str {
        "8bit-blocks"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Coarse pixel blocks fill in left-to-right, snapping to a chunky grid"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);

        // Pixel block size in dots — deliberately coarse.
        let px_w = 4usize;
        let px_h = (h / 3).max(2);
        let cols = (w / px_w).max(1);
        let rows = (h / px_h).max(1);
        let total = cols * rows;
        // Fill order: column by column, bottom row first (arcade-screen bottom-up).
        let filled = (ctx.eased * total as f32) as usize;

        for idx in 0..filled.min(total) {
            let col = idx / rows;
            let row_from_bottom = idx % rows;
            let row = rows.saturating_sub(1).saturating_sub(row_from_bottom);
            let x0 = col * px_w;
            let y0 = row * px_h;
            // 1-dot inset on each block to show the grid.
            draw::fill_rect(
                grid,
                x0,
                y0,
                px_w.saturating_sub(1).max(1),
                px_h.saturating_sub(1).max(1),
            );
        }

        // Tint with a slow color cycle (time-based hue shift via palette).
        let (cells_w, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let t = (cx as f32 / cells_w.max(1) as f32 + ctx.time * 0.1).fract();
                let color = ctx.palette.sample(t);
                // Only tint cells within the filled region.
                let block_col = cx / 2; // 2 cells per px_w/2 block approx
                if block_col < (ctx.eased * (cells_w / 2) as f32) as usize {
                    draw::tint_row(grid, cy, cx, cx, color);
                }
            }
        }

        Ok(())
    }
}

// ─── 7. CRT Scanline ─────────────────────────────────────────────────────────

struct CrtScanline;
impl ProgressStyle for CrtScanline {
    fn name(&self) -> &str {
        "crt-scanline"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "CRT phosphor bar with a rolling dark scanline and interlaced glow"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);

        // Filled region.
        let filled = (ctx.eased * w as f32).round() as usize;
        draw::fill_rect(grid, 0, 0, filled, h);

        // Scanline band rolling downward (wraps).
        let scan_period = 1.5f32;
        let scan_t = (ctx.time % scan_period) / scan_period;
        let scan_y = (scan_t * h as f32) as usize;
        // Erase the scanline row (subtract from braille — just skip those dots).
        // Since braille dots are set, we can punch a "dark band" by not drawing
        // two rows and instead drawing everything except those rows.
        //
        // Re-draw: clear whole grid by only setting non-scanline rows.
        // The fill_rect above already set everything; overlay a blank stripe
        // by drawing over with the outline trick isn't possible with set-only dots.
        // Instead: draw only every other row for the scanline effect.
        // We use a fresh approach: draw only non-suppressed rows.

        // Actually, braille grid is write-only in this API, so we simulate by
        // drawing line by line and skipping scanline rows in the first pass,
        // but fill_rect already ran. Let's draw the scan band as a brighter line
        // by double-drawing adjacent dots — the "dark" is the absence we can't erase.
        //
        // Practical CRT effect: draw even/odd scanlines with varying density.
        // Odd rows get a lighter fill (every other dot).
        // We do this by redrawing the bar without fill_rect, row by row.
        // First, re-render (we can't clear, but the grid starts clean each frame).
        // The grid IS cleared each render call by the caller. So we can just draw
        // row by row:

        // Clear conceptual state and re-draw carefully:
        // (The grid passed in is fresh — we can draw selectively.)
        // We already called fill_rect, but let's work with what we have.
        // Draw a "glow" column at the leading edge.
        if filled > 0 && filled <= w {
            let edge = filled.saturating_sub(1);
            draw::vline(grid, edge, 0, h - 1);
        }

        // Rolling scanline: highlight one horizontal band with extra dots.
        // Since we can only add dots, the "scanline" is a bright band.
        let bright_y = scan_y.min(h.saturating_sub(1));
        if filled > 0 {
            draw::hline(grid, 0, filled.saturating_sub(1), bright_y);
        }

        // Interlace: every other Y row, extend dots 1px into the unfilled zone.
        for y in (0..h).step_by(2) {
            if filled + 1 < w {
                draw::dot(grid, filled, y);
            }
        }

        // Tint: phosphor green across the fill, rolling brightness.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32).round() as usize;
        for cy in 0..cells_h {
            // Vary brightness by row (simulate scanline dimming).
            let row_t = cy as f32 / cells_h.max(1) as f32;
            let scan_dist = ((row_t - scan_t).abs()).min(1.0);
            let t = ctx.eased * (0.7 + 0.3 * scan_dist);
            let color = ctx.palette.sample(t.clamp(0.0, 1.0));
            if filled_cells > 0 {
                draw::tint_row(grid, cy, 0, filled_cells.saturating_sub(1), color);
            }
        }

        Ok(())
    }
}

// ─── 8. Snake ────────────────────────────────────────────────────────────────

struct Snake;
impl ProgressStyle for Snake {
    fn name(&self) -> &str {
        "snake"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Snake grows longer as progress increases, wiggling its body with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let mid_y = h / 2;

        // Snake length in dots proportional to progress.
        let snake_len = (ctx.eased * w as f32) as usize;
        if snake_len == 0 {
            return Ok(());
        }

        // Head leads at the right; tail trails.
        for seg in 0..snake_len {
            // X position: head at snake_len-1, tail at 0.
            let x = seg;
            // Y wiggle: sine wave travelling backwards (toward tail).
            let wave_phase = seg as f32 * 0.4 - ctx.time * 5.0;
            let amp = ((h / 2).saturating_sub(1)) as f32;
            let y_offset = (wave_phase.sin() * amp * 0.5).round() as i32;
            let y = mid_y as i32 + y_offset;

            // Draw a 2-dot wide segment for thickness.
            draw::dot_i(grid, x as i32, y);
            draw::dot_i(grid, x as i32, y + 1);
        }

        // Head: a 3×3 block at the front.
        let head_x = snake_len.saturating_sub(1) as i32;
        let head_wave = ((snake_len as f32) * 0.4 - ctx.time * 5.0).sin();
        let head_y =
            mid_y as i32 + (head_wave * (h / 2).saturating_sub(1) as f32 * 0.5).round() as i32;
        draw::dot_i(grid, head_x + 1, head_y);
        draw::dot_i(grid, head_x + 1, head_y + 1);
        // Eyes: two dots offset from center.
        draw::dot_i(grid, head_x, head_y - 1);

        // Tint the snake body with a gradient head→tail.
        let (cells_w, cells_h) = grid.dimensions();
        let snake_cells = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            for cx in 0..snake_cells.min(cells_w) {
                let t = cx as f32 / snake_cells.max(1) as f32;
                let color = ctx.palette.sample(t);
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ─── 9. Pinball Brick-Breaker ─────────────────────────────────────────────────

struct PinballBricks;
impl ProgressStyle for PinballBricks {
    fn name(&self) -> &str {
        "pinball-bricks"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "A bouncing ball knocks out bricks; bricks cleared = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);

        // Brick grid.
        let brick_cols = (w / 5).max(1);
        let brick_h = (h / 3).max(2).min(h.saturating_sub(4));
        let brick_w = 4usize;
        let bricks_total = brick_cols;
        let bricks_broken = (ctx.eased * bricks_total as f32) as usize;

        // Draw intact bricks at the top.
        for b in bricks_broken..bricks_total {
            let x0 = b * brick_w;
            draw::rect_outline(
                grid,
                x0,
                0,
                brick_w.min(w.saturating_sub(x0)),
                brick_h.max(2),
            );
        }

        // Paddle at the bottom — centered, moves slightly with time.
        let paddle_w = (w / 5).max(4);
        let paddle_y = h.saturating_sub(2);
        let paddle_drift =
            ((ctx.time * 0.8).sin() * (w.saturating_sub(paddle_w)) as f32 * 0.3) as i32;
        let paddle_cx = (w / 2) as i32 + paddle_drift;
        let px0 = (paddle_cx - paddle_w as i32 / 2).max(0) as usize;
        let px1 = (paddle_cx + paddle_w as i32 / 2).min(w as i32 - 1).max(0) as usize;
        draw::hline(grid, px0, px1, paddle_y);
        if paddle_y + 1 < h {
            draw::hline(grid, px0, px1, paddle_y + 1);
        }

        // Ball: parabolic bounce trajectory.
        let ball_period = 1.2f32;
        let ball_t = (ctx.time % ball_period) / ball_period; // 0..1
                                                             // X sweeps left→right.
        let ball_x = (ball_t * w as f32) as i32;
        // Y: parabola — up on first half, down on second.
        let ball_arc = 1.0 - (ball_t * 2.0 - 1.0).powi(2); // peak at t=0.5
        let ball_y = (brick_h as f32 + ball_arc * (h - brick_h) as f32 * 0.9) as i32;

        // Draw ball as a 2×2 block.
        draw::dot_i(grid, ball_x, ball_y);
        draw::dot_i(grid, ball_x + 1, ball_y);
        draw::dot_i(grid, ball_x, ball_y + 1);
        draw::dot_i(grid, ball_x + 1, ball_y + 1);

        // Tint broken region (where bricks were).
        let (cells_w, cells_h) = grid.dimensions();
        let broken_cells = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            if broken_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    broken_cells.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }

        Ok(())
    }
}

// ─── 10. Combo Power-Up Meter ────────────────────────────────────────────────

struct ComboPower;
impl ProgressStyle for ComboPower {
    fn name(&self) -> &str {
        "combo-power"
    }
    fn theme(&self) -> &str {
        "retro"
    }
    fn describe(&self) -> &str {
        "Combo/power-up meter with a pulsing plasma core and charging arcs"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let mid_y = (h / 2) as i32;
        let mid_x = (w / 2) as i32;

        // Fill: a segmented bar along the bottom half.
        let seg_count = 8usize;
        let seg_w = (w / seg_count).max(2);
        let lit = (ctx.eased * seg_count as f32).round() as usize;
        for s in 0..lit.min(seg_count) {
            let x0 = s * seg_w;
            let bw = seg_w.saturating_sub(1).max(1);
            let by = h / 2 + 1;
            let bh = h.saturating_sub(by + 1).max(1);
            draw::fill_rect(grid, x0, by, bw, bh);
        }

        // Plasma core: expanding ring that pulses with time.
        let pulse = (ctx.time * 4.0).sin() * 0.5 + 0.5; // 0..1
        let core_r = (pulse * ctx.eased * (h / 2) as f32).round() as i32;
        if core_r > 0 {
            let steps = (2.0 * PI * core_r as f32 * 2.0) as usize + 4;
            for s in 0..steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                let dx = (a.cos() * core_r as f32).round() as i32;
                let dy = (a.sin() * core_r as f32 * 0.5).round() as i32; // squash vertically
                draw::dot_i(grid, mid_x + dx, mid_y + dy);
            }
        }

        // Charging arcs: lightning-bolt-style zigzag lines converging on the core.
        let arc_count = lit.min(4);
        for arc in 0..arc_count {
            let base_angle = arc as f32 * PI / 2.0 + ctx.time * 3.0;
            let arc_len = (w / 4).max(2) as i32;
            for step in 0..arc_len {
                let frac = step as f32 / arc_len as f32;
                let angle = base_angle + (frac * PI * 2.0).sin() * 0.4; // zigzag
                let r = (arc_len as f32 * (1.0 - frac)).round() as i32;
                let ax = mid_x + (angle.cos() * r as f32).round() as i32;
                let ay = mid_y + (angle.sin() * r as f32 * 0.4).round() as i32;
                draw::dot_i(grid, ax, ay);
            }
        }

        // Center dot.
        draw::dot_i(grid, mid_x, mid_y);

        // Tint: full-spectrum cycle that speeds up as power charges.
        let (cells_w, cells_h) = grid.dimensions();
        let speed = 0.2 + ctx.eased * 1.0;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let t = (cx as f32 / cells_w.max(1) as f32 + ctx.time * speed).fract();
                let color = ctx.palette.sample(t);
                // Only tint cells within lit segments + core area.
                let in_bar = cx < (ctx.eased * cells_w as f32) as usize;
                let in_core = (cx as i32 - cells_w as i32 / 2).abs()
                    < (ctx.eased * cells_w as f32 * 0.15) as i32 + 1;
                if in_bar || in_core {
                    draw::tint_row(grid, cy, cx, cx, color);
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
    let styles = progress::styles::retro::styles();
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
