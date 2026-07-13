//! `nature` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O nature.rs && ./nature [style-name]
//! ```

const DEFAULT_STYLE: &str = "growing-vine";

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
    pub mod nature {
//! Nature / weather / seasons progress bars.
//!
//! Ten animated styles built entirely from `draw::` helpers. Every bar reads
//! `ctx.eased` for its fill/growth amount and `ctx.time` for looping
//! animation, so they look alive even when progress is held constant.
//! Palette tinting is applied via `draw::tint_row`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── registry ─────────────────────────────────────────────────────────────────

/// All styles in the `nature` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(GrowingVine),
        Box::new(GrassBlade),
        Box::new(Sunrise),
        Box::new(RainGauge),
        Box::new(TreeRings),
        Box::new(MountainSnow),
        Box::new(FourSeasons),
        Box::new(FlowerBloom),
        Box::new(FallingLeaves),
        Box::new(LightningBolt),
    ]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Integer sine rounded to i32 — keeps per-bar code terse.
#[inline]
fn isin(angle: f32, amplitude: f32) -> i32 {
    (angle.sin() * amplitude).round() as i32
}

// ── 1. Growing vine ──────────────────────────────────────────────────────────

struct GrowingVine;
impl ProgressStyle for GrowingVine {
    fn name(&self) -> &str {
        "growing-vine"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A vine climbs left-to-right; leaf crosses sprout at every 10% milestone"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = (h / 2) as i32;
        let reach = (ctx.eased * w as f32).round() as usize;

        // Sine-swaying stem.
        for x in 0..reach {
            let sway = isin(x as f32 * 0.3 + ctx.time * 2.0, (h as f32 * 0.2).max(1.0));
            draw::dot_i(grid, x as i32, mid + sway);
        }

        // Leaf crosses at each completed 10% threshold.
        for step in 1..=10usize {
            let threshold = step as f32 * 0.1;
            if ctx.eased < threshold {
                break;
            }
            let lx = (threshold * w as f32).round() as i32;
            let sway = isin(lx as f32 * 0.3 + ctx.time * 2.0, (h as f32 * 0.2).max(1.0));
            let ly = mid + sway;
            // Small cross: horizontal + vertical arms of length 2.
            for dx in -2i32..=2 {
                draw::dot_i(grid, lx + dx, ly);
            }
            for dy in -2i32..=2 {
                draw::dot_i(grid, lx, ly + dy);
            }
        }

        // Tint the filled span green.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        let color = ctx.palette.sample(ctx.eased);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, filled_cells.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 2. Grass blades ──────────────────────────────────────────────────────────

struct GrassBlade;
impl ProgressStyle for GrassBlade {
    fn name(&self) -> &str {
        "grass-blades"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Individual grass blades grow taller with progress and sway in the wind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let blade_spacing = 3usize.max(1);
        let base_y = h.saturating_sub(1);
        let max_growth = h.saturating_sub(1);

        let blade_count = w / blade_spacing;
        for b in 0..blade_count {
            let bx = b * blade_spacing + blade_spacing / 2;
            // Each blade grows to a slightly different height (pseudo-random via hash).
            let variety = ((b as f32 * 7.3).sin() * 0.5 + 0.5) * 0.4 + 0.6; // 0.6..1.0
            let height = (ctx.eased * max_growth as f32 * variety).round() as usize;
            if height == 0 {
                continue;
            }

            // Sway: low-frequency sine, each blade phase-shifted by position.
            let sway = isin(ctx.time * 1.8 + bx as f32 * 0.5, (h as f32 * 0.12).max(1.0));
            let tip_x = (bx as i32 + sway).max(0) as usize;

            let top_y = base_y.saturating_sub(height);
            // Stem.
            for y in top_y..=base_y {
                let frac = (base_y - y) as f32 / height.max(1) as f32;
                let sx = (bx as f32 + sway as f32 * frac).round() as usize;
                draw::dot(grid, sx.min(w.saturating_sub(1)), y);
            }
            // Tip dot — slightly to the side.
            draw::dot(grid, tip_x.min(w.saturating_sub(1)), top_y);
        }

        // Gradient tint.
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

// ── 3. Sunrise ───────────────────────────────────────────────────────────────

struct Sunrise;
impl ProgressStyle for Sunrise {
    fn name(&self) -> &str {
        "sunrise"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A sun disc rises along an arc; the horizon glows as it climbs"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Horizon line at the bottom row.
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        // Sun arc: progress 0 = right horizon, progress 1 = zenith/left horizon apex.
        // We map eased in [0, 1] → angle in [0, PI], where:
        //   angle=0   → cos=1,  sin=0 → sun at right horizon
        //   angle=PI/2 → cos=0, sin=1 → sun at zenith (top centre)
        //   angle=PI  → cos=-1, sin=0 → sun at left horizon
        // To keep the sun HIGH at full progress we clamp the arc to [0, PI/2]
        // (dawn to noon) so t=1 means the sun is at its peak, not setting again.
        let cx = (w / 2) as i32;
        let base_y = h.saturating_sub(2) as i32;
        let arc_rx = (w as f32 * 0.38) as i32;
        let arc_ry = h.saturating_sub(2).max(1) as i32;

        // Map progress 0→1 to angle PI→0 (right horizon rises to zenith and beyond).
        // Clamp to [0, PI] so at t=1 the sun rests at the top-left.
        let angle = (1.0 - ctx.eased) * PI;
        let sun_x = cx + (arc_rx as f32 * angle.cos()) as i32;
        // sin is always ≥ 0 for angle in [0, PI], so sun is always above or at horizon.
        let sun_y = base_y - (arc_ry as f32 * angle.sin()) as i32;

        // Draw sun as a small filled disc (radius 2 in dot-space).
        let r = 2i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r + 1 {
                    draw::dot_i(grid, sun_x + dx, sun_y + dy);
                }
            }
        }

        // Rays: 8 short lines radiating from sun center, animated with slow rotation.
        let ray_len = 4i32;
        for k in 0..8 {
            let ray_angle = (k as f32 / 8.0) * 2.0 * PI + ctx.time * 0.4;
            let rx = (ray_angle.cos() * (r + ray_len) as f32) as i32;
            let ry = (ray_angle.sin() * (r + ray_len) as f32) as i32;
            draw::dot_i(grid, sun_x + rx, sun_y + ry);
            draw::dot_i(grid, sun_x + rx / 2, sun_y + ry / 2);
        }

        // Sky tint: palette from bottom to top across all cells.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let sky_t = 1.0 - cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let tint_t = ctx.eased * sky_t;
            let color = ctx.palette.sample(tint_t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 4. Rain gauge ────────────────────────────────────────────────────────────

struct RainGauge;
impl ProgressStyle for RainGauge {
    fn name(&self) -> &str {
        "rain-gauge"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Raindrops fall from a cloud; the water level rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Gauge outline.
        draw::rect_outline(grid, 0, 0, w, h);

        // Water fill from the bottom up.
        let fill_h = (ctx.eased * (h.saturating_sub(2)) as f32).round() as usize;
        let water_top = h.saturating_sub(1 + fill_h);
        if fill_h > 0 {
            draw::fill_rect(grid, 1, water_top, w.saturating_sub(2).max(1), fill_h);
        }

        // Animated raindrops — 5 drops cycling at different rates.
        let unfilled_h = water_top;
        if unfilled_h > 1 {
            for d in 0..5usize {
                let phase = d as f32 / 5.0;
                let drop_cycle = (ctx.time * 1.5 + phase).fract();
                let drop_y = (drop_cycle * unfilled_h as f32) as usize;
                let drop_x =
                    (((d as f32 * 1.618 + 0.5) % 1.0) * (w.saturating_sub(2)) as f32) as usize + 1;
                // Draw as a 1-dot pip.
                draw::dot(
                    grid,
                    drop_x.min(w.saturating_sub(2)),
                    drop_y.min(unfilled_h.saturating_sub(1)),
                );
            }
        }

        // Tint water blue via palette.
        let (cw, ch) = grid.dimensions();
        let water_top_cell = (water_top / 4).min(ch.saturating_sub(1));
        for cy in water_top_cell..ch {
            let t = (cy - water_top_cell) as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 5. Tree rings ────────────────────────────────────────────────────────────

struct TreeRings;
impl ProgressStyle for TreeRings {
    fn name(&self) -> &str {
        "tree-rings"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Concentric growth rings expand outward from the center as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_rings = 6usize;
        let max_rx = (w / 2).saturating_sub(1) as f32;
        let max_ry = (h / 2).saturating_sub(1) as f32;

        let rings_visible = (ctx.eased * max_rings as f32).ceil() as usize;

        for ring in 1..=rings_visible.min(max_rings) {
            // Partial reveal on the outermost ring.
            let ring_frac = if ring < rings_visible {
                1.0f32
            } else {
                let base = (ring - 1) as f32 / max_rings as f32;
                let span = 1.0 / max_rings as f32;
                ((ctx.eased - base) / span).clamp(0.0, 1.0)
            };

            let rx = (max_rx * ring as f32 / max_rings as f32) as i32;
            let ry = (max_ry * ring as f32 / max_rings as f32) as i32;
            let rx = rx.max(1);
            let ry = ry.max(1);

            // Parametric ellipse, draw fraction based on ring_frac.
            let steps = 120usize;
            let arc_steps = (steps as f32 * ring_frac).round() as usize;
            for s in 0..arc_steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                let ex = cx + (rx as f32 * a.cos()).round() as i32;
                let ey = cy + (ry as f32 * a.sin()).round() as i32;
                draw::dot_i(grid, ex, ey);
            }
        }

        // Center dot, always present.
        draw::dot_i(grid, cx, cy);

        // Radial tint.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 6. Mountain snow ─────────────────────────────────────────────────────────

struct MountainSnow;
impl ProgressStyle for MountainSnow {
    fn name(&self) -> &str {
        "mountain-snow"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A mountain silhouette fills with snow creeping down from the peaks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Mountain profile: triangle with roughened ridgeline.
        let peak_x = w / 2;
        let base_y = h.saturating_sub(1);

        // Render silhouette: for each x, calculate mountain height.
        let mountain_h = |x: usize| -> usize {
            let dist = x.abs_diff(peak_x);
            let slope = 1.0 - dist as f32 / peak_x.max(1) as f32;
            // Add ridge roughness.
            let roughness = ((x as f32 * 0.4).sin() * 0.08 + (x as f32 * 0.7).cos() * 0.05) * slope;
            let mh = (slope + roughness).clamp(0.0, 1.0);
            (mh * h as f32).round() as usize
        };

        // Rock body: crisp ridgeline with a dithered interior, so the solid
        // snow cap below reads against it even in monochrome.
        for x in 0..w {
            let mh = mountain_h(x);
            if mh == 0 {
                continue;
            }
            let top_y = base_y.saturating_sub(mh);
            draw::dot(grid, x, top_y);
            for y in (top_y + 1)..=base_y {
                if (x + y * 2) % 3 == 0 {
                    draw::dot(grid, x, y);
                }
            }
        }

        // Snow: solid fill creeping down from the ridgeline by eased fraction.
        for x in 0..w {
            let mh = mountain_h(x);
            if mh == 0 {
                continue;
            }
            let snow_h = (ctx.eased * mh as f32).round() as usize;
            if snow_h == 0 {
                continue;
            }
            // Snowline starts at the top and descends.
            let top_y = base_y.saturating_sub(mh);
            let snow_bottom = (top_y + snow_h).min(base_y);
            draw::vline(grid, x, top_y, snow_bottom);
        }

        // Snowfall: flakes drift down the open sky above the ridgeline. Fall
        // rates are multiples of 0.25 Hz so the 4-second loop stays seamless.
        for k in 0..10usize {
            let seed = (k as f32 * 0.618_034).fract();
            let fx0 = (seed * w as f32) as i32;
            let rate = 0.25 * (1 + k % 2) as f32;
            let fall = (ctx.time * rate + seed).fract();
            let fy = (fall * h as f32) as i32;
            let sway = isin(2.0 * PI * 0.25 * ctx.time + k as f32, 1.5);
            let fx = fx0 + sway;
            if fx >= 0 && (fx as usize) < w {
                let sky_floor = base_y.saturating_sub(mountain_h(fx as usize)) as i32;
                if fy < sky_floor {
                    draw::dot_i(grid, fx, fy);
                }
            }
        }

        // Snow-cap tint via palette (cool blue-white at top, warm at base).
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let height_frac = 1.0 - cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let t = height_frac * ctx.eased;
            let color = ctx.palette.sample(1.0 - t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 7. Four seasons ──────────────────────────────────────────────────────────

struct FourSeasons;
impl ProgressStyle for FourSeasons {
    fn name(&self) -> &str {
        "four-seasons"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "The fill sweeps through spring, summer, autumn, and winter colour bands"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let filled = (ctx.eased * w as f32).round() as usize;

        // Solid fill dots first.
        draw::fill_rect(grid, 0, 0, filled, h);

        // Season tinting: each quarter of the fill is a distinct season colour.
        // Spring: green (0–0.25), Summer: gold (0.25–0.5),
        // Autumn: burnt orange (0.5–0.75), Winter: icy blue (0.75–1.0).
        use crate::Color;
        let season_colors: [(Color, Color); 4] = [
            (Color::rgb(34, 139, 34), Color::rgb(144, 238, 144)), // spring
            (Color::rgb(218, 165, 32), Color::rgb(255, 215, 0)),  // summer
            (Color::rgb(160, 82, 45), Color::rgb(205, 133, 63)),  // autumn
            (Color::rgb(70, 130, 180), Color::rgb(173, 216, 230)), // winter
        ];

        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let frac = cx as f32 / cw.saturating_sub(1).max(1) as f32;
            // Which season are we in?
            let season_idx = ((frac * 4.0) as usize).min(3);
            let season_t = (frac * 4.0).fract();
            // Animate a subtle brightness pulse within each season.
            let pulse = (ctx.time * 1.2 + frac * 2.0 * PI).sin() * 0.08 + 0.92; // 0.84..1.0
            let (s_col, e_col) = season_colors[season_idx];
            let r = super::super::lerp(s_col.r as f32, e_col.r as f32, season_t) * pulse;
            let g = super::super::lerp(s_col.g as f32, e_col.g as f32, season_t) * pulse;
            let b = super::super::lerp(s_col.b as f32, e_col.b as f32, season_t) * pulse;
            let color = Color::rgb(
                r.clamp(0.0, 255.0) as u8,
                g.clamp(0.0, 255.0) as u8,
                b.clamp(0.0, 255.0) as u8,
            );
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        // Track outline.
        draw::hline(grid, 0, w.saturating_sub(1), 0);
        draw::hline(grid, 0, w.saturating_sub(1), h.saturating_sub(1));

        Ok(())
    }
}

// ── 8. Flower bloom ──────────────────────────────────────────────────────────

struct FlowerBloom;
impl ProgressStyle for FlowerBloom {
    fn name(&self) -> &str {
        "flower-bloom"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Petals extend radially from the centre as the flower blooms with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let petals = 8usize;
        // Max petal length — limited by whichever dimension is smaller.
        let max_len = (w.min(h) / 2).saturating_sub(1) as f32;

        for p in 0..petals {
            let base_angle = (p as f32 / petals as f32) * 2.0 * PI;
            // Petals slowly rotate with time.
            let angle = base_angle + ctx.time * 0.3;
            let petal_len = (ctx.eased * max_len).max(0.0);

            // Draw petal as a line from centre outward.
            let steps = petal_len.round() as usize;
            for s in 0..=steps {
                let r = s as f32;
                let px = cx + (angle.cos() * r).round() as i32;
                let py = cy + (angle.sin() * r).round() as i32;
                draw::dot_i(grid, px, py);
            }

            // Small bulge at the petal tip (two side dots).
            if petal_len >= 2.0 {
                let side_angle = angle + PI / 2.0;
                let tip_x = cx + (angle.cos() * petal_len).round() as i32;
                let tip_y = cy + (angle.sin() * petal_len).round() as i32;
                draw::dot_i(
                    grid,
                    tip_x + side_angle.cos().round() as i32,
                    tip_y + side_angle.sin().round() as i32,
                );
                draw::dot_i(
                    grid,
                    tip_x - side_angle.cos().round() as i32,
                    tip_y - side_angle.sin().round() as i32,
                );
            }
        }

        // Centre dot always present.
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx + 1, cy);
        draw::dot_i(grid, cx, cy + 1);

        // Tint radially: warm centre → cool petals.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 9. Falling leaves ────────────────────────────────────────────────────────

struct FallingLeaves;
impl ProgressStyle for FallingLeaves {
    fn name(&self) -> &str {
        "falling-leaves"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "Autumn leaves drift downward with parabolic + sine sway, density grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of leaves grows with progress (min 2, max 12).
        let leaf_count = (2.0 + ctx.eased * 10.0).round() as usize;

        for leaf in 0..leaf_count {
            // Each leaf has a fixed "lane" x and an individual fall phase.
            let phase = leaf as f32 / leaf_count as f32;
            let cycle = (ctx.time * 0.7 + phase).fract();

            // Horizontal position: sine drift across the width.
            let origin_x = (phase * (w.saturating_sub(4)) as f32) as i32 + 2;
            let sway = isin(
                cycle * 2.0 * PI * 1.5 + phase * 3.0,
                (w as f32 * 0.07).max(2.0),
            );
            let lx = (origin_x + sway).clamp(0, w.saturating_sub(1) as i32);

            // Vertical: linear fall, wraps.
            let ly = (cycle * h as f32) as i32;

            // Leaf shape: a small cross (2-wide).
            draw::dot_i(grid, lx, ly);
            draw::dot_i(grid, lx + 1, ly);
            draw::dot_i(grid, lx, ly + 1);
            draw::dot_i(grid, lx - 1, ly);
        }

        // Autumn gradient tint via palette.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t * ctx.eased + (1.0 - ctx.eased) * 0.5);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 10. Lightning bolt ───────────────────────────────────────────────────────

struct LightningBolt;
impl ProgressStyle for LightningBolt {
    fn name(&self) -> &str {
        "lightning-bolt"
    }
    fn theme(&self) -> &str {
        "nature"
    }
    fn describe(&self) -> &str {
        "A jagged lightning bolt zigzags across the bar; intensity pulses with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // How far the bolt extends is controlled by progress.
        let reach = (ctx.eased * w as f32).round() as usize;
        if reach == 0 {
            return Ok(());
        }

        // Brightness pulse: modulates the bolt's drawn thickness.
        let pulse = (ctx.time * 4.0 * PI).sin() * 0.5 + 0.5; // 0..1
        let thickness = if pulse > 0.6 { 2i32 } else { 1i32 };

        // Bolt path: zigzag with fixed "joints" spaced every w/5 dots.
        let segments = 5usize;
        let seg_w = (reach / segments.max(1)).max(1);

        // Seed the zigzag using time (slow drift, so it flickers).
        let mut prev_y = (h / 2) as i32;

        for seg in 0..segments {
            let x0 = (seg * seg_w).min(reach.saturating_sub(1));
            let x1 = ((seg + 1) * seg_w).min(reach.saturating_sub(1));
            if x1 <= x0 {
                break;
            }

            // Next joint y chosen via time-seeded sine (different per segment).
            let angle = ctx.time * 3.0 + seg as f32 * 1.9;
            let next_y = (h as f32 / 2.0 + angle.sin() * h as f32 * 0.38) as i32;

            // Draw a straight line between joints via DDA.
            let dx = (x1 - x0) as i32;
            let dy = next_y - prev_y;
            let steps = dx.max(dy.abs()).max(1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let bx = x0 as i32 + (t * dx as f32).round() as i32;
                let by = prev_y + (t * dy as f32).round() as i32;
                for tk in -thickness / 2..=thickness / 2 {
                    draw::dot_i(grid, bx, by + tk);
                }
            }

            prev_y = next_y;
        }

        // Electric tint: palette sampled at progress, brighter at the tip.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if filled_cells <= 1 {
                1.0
            } else {
                cx as f32 / (filled_cells - 1) as f32
            };
            let color = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, color);
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
    let styles = progress::styles::nature::styles();
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
