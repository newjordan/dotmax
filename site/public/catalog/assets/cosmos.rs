//! `cosmos` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O cosmos.rs && ./cosmos [style-name]
//! ```

const DEFAULT_STYLE: &str = "supernova";

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
    pub mod cosmos {
//! Cosmos / deep-space-phenomena progress bars.
//!
//! Ten structurally distinct styles, each modelling a different astrophysical
//! phenomenon: supernova shockwave, pulsar lighthouse, nebula condensation,
//! big-bang expansion, solar flare arc, aurora curtains, meteor shower,
//! total eclipse corona, gravitational-lens Einstein ring, cosmic-web
//! filaments, redshift wavefronts, and a quasar relativistic jet.
//!
//! All bars return `"cosmos"` from `theme()`. `ctx.eased` drives the
//! intensity / progress of the phenomenon; `ctx.time` drives animation.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic pseudo-random helpers — no external crates.
// ---------------------------------------------------------------------------

/// Cheap integer hash (Knuth multiplicative + avalanche).
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

/// Float in [0, 1) from index `n`.
fn hash_f(n: u32) -> f32 {
    (hash(n) % 10_000) as f32 / 10_000.0
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `cosmos` theme, in display order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Supernova),
        Box::new(Pulsar),
        Box::new(NebulaClouds),
        Box::new(BigBang),
        Box::new(SolarFlare),
        Box::new(AuroraCurtains),
        Box::new(MeteorShower),
        Box::new(TotalEclipse),
        Box::new(GravitationalLens),
        Box::new(CosmicWeb),
        Box::new(Redshift),
        Box::new(QuasarJet),
    ]
}

// ---------------------------------------------------------------------------
// 1 — Supernova
// ---------------------------------------------------------------------------
// Structural idea: an expanding shell of dots whose radius = eased * max_r.
// The shell itself is a thin ring (drawn at radius ± 1). A dense core shrinks
// as the shell grows, and a sparse debris halo scatters outside the shell.
// No other style here uses a growing circular shell as its primary element.

/// Supernova: shockwave shell radiates outward; radius tracks eased, debris halos outside.
struct Supernova;
impl ProgressStyle for Supernova {
    fn name(&self) -> &str {
        "supernova"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Supernova shockwave shell explodes outward; radius = eased * max; debris halos outside"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = cx.min(cy * 2).max(1) as f32;
        let shell_r = (ctx.eased * max_r) as i32;

        // Collapsing stellar core — solid disc that shrinks as shell expands.
        let core_r = ((1.0 - ctx.eased) * max_r * 0.25) as i32;
        for dy in -core_r..=core_r {
            let dx_max = ((core_r * core_r - dy * dy).max(0) as f32).sqrt() as i32;
            for dx in -dx_max..=dx_max {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Shell ring — draw dots at radius shell_r ± 1.
        if shell_r > 0 {
            let steps = (2.0 * PI * shell_r as f32 * 1.5) as usize;
            let steps = steps.max(8);
            for s in 0..steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                for dr in [-1i32, 0, 1] {
                    let r = (shell_r + dr).max(0);
                    let px = cx + (r as f32 * a.cos()) as i32;
                    let py = cy + (r as f32 * a.sin() * 0.5) as i32;
                    // Occasional gap for ring texture
                    if hash((s as u32).wrapping_add(dr.unsigned_abs() * 997)) % 5 != 0 {
                        draw::dot_i(grid, px, py);
                    }
                }
            }
        }

        // Debris: sparse dots between core and shell, animated via time.
        let debris_count = 40u32;
        for i in 0..debris_count {
            let angle = hash_f(i) * 2.0 * PI;
            let r_frac = hash_f(i + 1000);
            let r = (r_frac * shell_r as f32 * 0.9) as i32;
            if r <= core_r {
                continue;
            }
            // Drift outward at speed proportional to position
            let drift = (ctx.time * 0.4 * r_frac) as i32;
            let r_d = (r + drift).min(shell_r);
            let px = cx + (r_d as f32 * angle.cos()) as i32;
            let py = cy + (r_d as f32 * angle.sin() * 0.5) as i32;
            if hash(i.wrapping_add((ctx.time * 3.0) as u32)) % 3 != 0 {
                draw::dot_i(grid, px, py);
            }
        }

        // Tint: hot white core → violet shell
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0).max(1.0);
            let t = 1.0 - dist;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2 — Pulsar
// ---------------------------------------------------------------------------
// Structural idea: two opposed lighthouse beams sweeping around a central dot
// at angular speed driven by time; beam count (blips) accumulates via eased.
// No other style here uses sweeping angular beams as the fill mechanism.

/// Pulsar: two lighthouse beams sweep around a magnetar; blip count = eased * max.
struct Pulsar;
impl ProgressStyle for Pulsar {
    fn name(&self) -> &str {
        "pulsar"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Pulsar lighthouse beams sweep via time; blip tally counts up to eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = cx.min(cy * 2).max(2) as f32;

        // Spin rate accelerates with eased (faster pulsar = more progress).
        let spin_rate = 2.0 + ctx.eased * 6.0;
        let beam_angle = ctx.time * spin_rate;

        // Two opposed beams (180° apart).
        for beam in 0u32..2 {
            let base_a = beam_angle + beam as f32 * PI;
            // Each beam has angular half-width that narrows as progress grows
            // (pulsar emission cone sharpens at higher spin).
            let half_width = 0.18 - ctx.eased * 0.08;
            let half_width = half_width.max(0.04);

            let beam_steps = (max_r * 1.5) as usize;
            let beam_steps = beam_steps.max(4);
            for s in 0..beam_steps {
                let r = s as f32;
                if r > max_r {
                    break;
                }
                // Fan: sweep a few angles within half_width
                let fan_n = 5usize;
                for f in 0..fan_n {
                    let da =
                        (f as f32 / fan_n.saturating_sub(1).max(1) as f32 - 0.5) * half_width * 2.0;
                    let a = base_a + da;
                    let px = cx + (r * a.cos()) as i32;
                    let py = cy + (r * a.sin() * 0.5) as i32;
                    // Beam fades with distance
                    let fade_thresh = (1.0 - r / max_r) * 3.0;
                    if hash(
                        (s as u32)
                            .wrapping_mul(7)
                            .wrapping_add(f as u32 * 13)
                            .wrapping_add(beam * 997),
                    ) % 4
                        < (fade_thresh * 3.5) as u32 + 1
                    {
                        draw::dot_i(grid, px, py);
                    }
                }
            }
        }

        // Neutron-star core: dense 3×3 cluster
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Blip tally bar: small accumulator along the bottom row of dots.
        // Each blip = one lit dot; max blips = width.
        let max_blips = w.saturating_sub(2);
        let blips = (ctx.eased * max_blips as f32) as usize;
        for b in 0..blips.min(max_blips) {
            draw::dot(grid, b + 1, h.saturating_sub(1));
        }

        // Tint beams with palette
        let (cells_w, cells_h) = grid.dimensions();
        for cy_c in 0..cells_h {
            let t = cy_c as f32 / cells_h.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cells_w.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3 — Nebula cloud condensing
// ---------------------------------------------------------------------------
// Structural idea: the entire canvas is subdivided into shaded cells; cell
// shade level grows with eased (gas condenses into denser material).
// Pure shade-glyph approach — no dot-drawing at all in this style.

/// Nebula: gas cloud condenses across the canvas; cell shade = eased density.
struct NebulaClouds;
impl ProgressStyle for NebulaClouds {
    fn name(&self) -> &str {
        "nebula-clouds"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Nebula gas cloud condenses via shade glyphs; density = eased; ripples with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }

        // Each cell gets a noise-like density offset from hash + time ripple.
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let idx = (cy * cells_w + cx) as u32;

                // Static spatial noise [0,1).
                let spatial = hash_f(idx);

                // Time ripple: slow outward pulse from centre.
                let dcx = cx as f32 - cells_w as f32 / 2.0;
                let dcy = (cy as f32 - cells_h as f32 / 2.0) * 2.0;
                let dist = (dcx * dcx + dcy * dcy).sqrt();
                let ripple = (dist * 0.4 - ctx.time * 0.7).sin() * 0.15;

                // Combined density: eased base + spatial variation + ripple.
                let density = (ctx.eased + spatial * 0.35 - 0.175 + ripple).clamp(0.0, 1.0);

                // Map density → shade level 0..4.
                let level = (density * 4.0) as usize;
                draw::shade(grid, cx, cy, level);
            }
        }

        // Tint with palette
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4 — Big Bang expansion
// ---------------------------------------------------------------------------
// Structural idea: all dots radiate OUTWARD from a single origin point as
// eased increases. At progress=0 everything is at the singularity; at
// progress=1 dots fill the whole canvas. Radial particle positions scale
// linearly with eased, animated with a slow drift via time.

/// Big Bang: particles radiate from singularity; position = eased × final_coords.
struct BigBang;
impl ProgressStyle for BigBang {
    fn name(&self) -> &str {
        "big-bang"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Big-bang singularity explodes: all particles radiate outward, position scales with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;

        // Singularity flash at t=0: bright core when eased is small.
        if ctx.eased < 0.05 {
            draw::dot_i(grid, cx as i32, cy as i32);
            return Ok(());
        }

        let particle_count = 120u32;
        for i in 0..particle_count {
            // Each particle has a stable random target position on the canvas edge.
            let target_angle = hash_f(i) * 2.0 * PI;
            // Target distance: between half-way and the edge.
            let target_r_frac = 0.5 + hash_f(i + 500) * 0.5;
            let max_half = cx.min(cy * 1.8);
            let target_r = target_r_frac * max_half;

            // Current position = origin + eased * (target - origin)
            let r = ctx.eased * target_r;

            // Slow thermal drift via time.
            let drift_a = target_angle + (ctx.time * 0.05 * hash_f(i + 200) * 2.0 - 0.05);
            let px = cx + r * drift_a.cos();
            let py = cy + r * drift_a.sin() * 0.55;

            // Draw particle; brighter near leading edge.
            draw::dot_i(grid, px as i32, py as i32);
            if hash(i.wrapping_add(17)) % 4 == 0 {
                draw::dot_i(grid, px as i32 + 1, py as i32);
            }
        }

        // Tint: hot centre → cool edge
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0).max(1.0);
            let t = 1.0 - dist * 0.9;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5 — Solar flare arc
// ---------------------------------------------------------------------------
// Structural idea: a parabolic arc of dots loops OFF a stellar limb.
// The arc rises and falls (parametric in angle 0→π), its peak height driven
// by eased; time animates a slow rotation of the arc base on the stellar disc.
// No other style draws a single parametric arc off a limb.

/// Solar flare: a looping arc erupts off the stellar limb; height = eased.
struct SolarFlare;
impl ProgressStyle for SolarFlare {
    fn name(&self) -> &str {
        "solar-flare"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Solar flare arc loops off a stellar limb; peak height = eased; base rotates with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let star_r = (cy * 0.6).max(1.0);

        // Stellar disc.
        let disc_steps = 64usize;
        for s in 0..disc_steps {
            let a = s as f32 / disc_steps as f32 * 2.0 * PI;
            for dr in 0..=(star_r as i32) {
                let frac = dr as f32 / star_r;
                if hash(
                    (s as u32)
                        .wrapping_mul(13)
                        .wrapping_add(dr.unsigned_abs() * 7),
                ) % 10
                    < (frac * 9.0 + 1.0) as u32
                {
                    let px = cx + dr as f32 * a.cos();
                    let py = cy + dr as f32 * a.sin() * 0.55;
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Flare arc: parametric, base rotates with time.
        let base_angle = ctx.time * 0.4; // slow rotation
        let arc_half_span = 0.4f32; // angular half-width of the base on the limb (radians)

        // Arc from base_angle - arc_half_span → base_angle + arc_half_span via apex.
        // Apex is above the limb, height = eased * extra.
        let apex_r = star_r + ctx.eased * (cy * 1.2).max(1.0);

        let arc_steps = 50usize;
        for s in 0..arc_steps {
            let t = s as f32 / arc_steps.saturating_sub(1).max(1) as f32;
            // Parametric: angle sweeps from left base to right base;
            // radius peaks at apex_r at t=0.5, equals star_r at t=0 and t=1.
            let angle = (base_angle - arc_half_span) + t * 2.0 * arc_half_span;
            let r = star_r + (apex_r - star_r) * (PI * t).sin();

            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin() * 0.55;
            draw::dot_i(grid, px as i32, py as i32);

            // Glow: an extra dot offset slightly outward
            if s % 3 == 0 {
                let px2 = cx + (r + 1.5) * angle.cos();
                let py2 = cy + (r + 1.5) * angle.sin() * 0.55;
                draw::dot_i(grid, px2 as i32, py2 as i32);
            }
        }

        // Tint: star warm orange, flare hot white
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6 — Aurora curtains
// ---------------------------------------------------------------------------
// Structural idea: vertical columns of dots whose height is modulated by a
// sine-wave, creating "curtains" that ripple horizontally via time.
// eased controls how many columns are lit (curtain extent).
// The ripple phase varies per column — distinct from any horizontal bar style.

/// Aurora: vertical curtain columns ripple via sine; extent = eased columns.
struct AuroraCurtains;
impl ProgressStyle for AuroraCurtains {
    fn name(&self) -> &str {
        "aurora-curtains"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Aurora curtains: vertical sine-sheet columns ripple via time; extent = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let lit_cols = ((ctx.eased * w as f32) as usize).min(w);

        for x in 0..lit_cols {
            // Each column has a phase offset based on position.
            let col_phase = x as f32 * 0.3;
            // Height envelope: sine wave scrolling over time.
            let envelope = 0.5 + 0.5 * (ctx.time * 1.8 + col_phase).sin();
            // Secondary ripple for width texture.
            let secondary = 0.2 * (ctx.time * 3.1 + col_phase * 1.7).sin();
            let col_h_frac = (envelope + secondary).clamp(0.1, 1.0);
            let col_h = (col_h_frac * h as f32) as usize;

            // Curtain hangs from the top.
            let y0 = 0usize;
            let y1 = col_h.min(h).saturating_sub(1);
            for y in y0..=y1 {
                // Density fades toward the bottom of each curtain.
                let fade = 1.0 - (y as f32 / col_h.max(1) as f32);
                if hash(
                    (x as u32)
                        .wrapping_mul(17)
                        .wrapping_add(y as u32 * 31)
                        .wrapping_add((ctx.time * 8.0) as u32),
                ) % 8
                    < (fade * 7.5 + 0.5) as u32
                {
                    draw::dot(grid, x, y);
                }
            }
        }

        // Tint: green → blue → violet across width (aurora spectrum).
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7 — Meteor shower
// ---------------------------------------------------------------------------
// Structural idea: diagonal streaks rain from top-right to bottom-left.
// Active meteor count = eased * max_meteors; each meteor has a stable entry
// point on the top/right edge and trails behind it with decreasing dot density.
// Direction and density-from-count are structurally unlike all other styles.

/// Meteor shower: diagonal streaks rain across the canvas; count = eased * max.
struct MeteorShower;
impl ProgressStyle for MeteorShower {
    fn name(&self) -> &str {
        "meteor-shower"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Meteor shower: diagonal streaks rain top-right → bottom-left; count scales with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let max_meteors = 20u32;
        let active = (ctx.eased * max_meteors as f32) as u32;

        for m in 0..active.min(max_meteors) {
            // Stable entry x along top edge + right edge combined.
            let entry_frac = hash_f(m);
            let speed = 0.6 + hash_f(m + 300) * 1.2;
            let trail_len = (4.0 + hash_f(m + 600) * 8.0) as usize;

            // Entry x (spread across full width plus some from right edge).
            let entry_x = (entry_frac * (w + h) as f32) as i32 - h as i32;
            // Head position: travels diagonally at rate (speed, speed/2).
            let travel = (ctx.time * speed * 8.0) as i32;
            // Wrap travel so shower continues indefinitely.
            let period = (w + h) as i32;
            let travel = travel % period.max(1);

            let head_x = entry_x - travel;
            let head_y = travel / 2;

            // Trail dots from head backward along the streak direction (+1, -0.5 per dot).
            for t in 0..trail_len {
                let tx = head_x + t as i32;
                let ty = head_y - t as i32 / 2;
                // Density drops with distance from head.
                let frac = 1.0 - t as f32 / trail_len as f32;
                if hash(
                    m.wrapping_mul(31)
                        .wrapping_add(t as u32 * 7)
                        .wrapping_add((ctx.time * 4.0) as u32),
                ) % 10
                    < (frac * frac * 9.5 + 0.5) as u32
                {
                    draw::dot_i(grid, tx, ty);
                }
            }
            // Bright head: 2-dot cluster.
            draw::dot_i(grid, head_x, head_y);
            draw::dot_i(grid, head_x - 1, head_y);
        }

        // Background star field (stable, sparse).
        for i in 0u32..20 {
            let sx = (hash_f(i + 9000) * w as f32) as i32;
            let sy = (hash_f(i + 9100) * h as f32) as i32;
            if hash(i.wrapping_add((ctx.time * 2.0) as u32)) % 4 != 0 {
                draw::dot_i(grid, sx, sy);
            }
        }

        // Tint: cool blue at top, warmer orange at bottom (heat from entry).
        let (cells_w, cells_h) = grid.dimensions();
        for cy_c in 0..cells_h {
            let t = 1.0 - cy_c as f32 / cells_h.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cells_w.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8 — Total eclipse
// ---------------------------------------------------------------------------
// Structural idea: TWO discs — a star (bright, fixed left-of-center) and
// an occluder moon (moving right-to-left across the star as eased goes 0→1).
// At mid-eclipse a corona ring of dots surrounds the star's limb where the
// moon occluder covers it. No other style has two interacting geometric bodies.

/// Total eclipse: moon disc occluder crosses star limb; corona ring appears at totality.
struct TotalEclipse;
impl ProgressStyle for TotalEclipse {
    fn name(&self) -> &str {
        "total-eclipse"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Moon occluder crosses star limb from right; corona ring bursts at mid-eclipse"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let star_cx = (w / 2) as i32;
        let star_cy = (h / 2) as i32;
        let star_r = ((h / 2).saturating_sub(1).max(2)) as i32;

        // Moon starts at the right edge, crosses the star, ends at the left
        // edge. At eased=0.5 the centres align (totality). Travel is capped
        // at the grid width so the moon limb stays visible at both extremes
        // and the render always reflects progress.
        let travel = w as i32;
        let moon_cx = star_cx + travel / 2 - (ctx.eased * travel as f32) as i32;
        let moon_cy = star_cy;
        let moon_r = (star_r as f32 * 0.92) as i32;

        // Draw stellar disc (skip pixels covered by moon).
        let steps = 72usize;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            for dr in 0..=star_r {
                let px = star_cx + (dr as f32 * a.cos()) as i32;
                let py = star_cy + (dr as f32 * a.sin() * 0.5) as i32;
                // Check if this dot is inside the moon disc.
                let ddx = px - moon_cx;
                let ddy = (py - moon_cy) * 2; // undo vertical squeeze
                let in_moon = ddx * ddx + ddy * ddy <= moon_r * moon_r;
                if !in_moon {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Moon disc (always drawn, covers star).
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            for dr in 0..=moon_r {
                let px = moon_cx + (dr as f32 * a.cos()) as i32;
                let py = moon_cy + (dr as f32 * a.sin() * 0.5) as i32;
                // We draw only the outline to leave the moon "dark" (no dots inside).
                if dr == moon_r || dr == moon_r - 1 {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Corona ring: visible only near totality (eased ≈ 0.5).
        let totality_closeness = 1.0 - (ctx.eased * 2.0 - 1.0).abs(); // peaks at 0.5
        let corona_steps = 48usize;
        // Breathe at 0.25 Hz so the 4-second loop stays seamless.
        let corona_r = star_r + 2 + (ctx.time * 0.5 * PI).sin().round() as i32;
        let corona_r = corona_r.max(star_r + 1);
        for s in 0..corona_steps {
            let a = s as f32 / corona_steps as f32 * 2.0 * PI;
            let px = star_cx + (corona_r as f32 * a.cos()) as i32;
            let py = star_cy + (corona_r as f32 * a.sin() * 0.5) as i32;
            // Only appear when near totality; flicker via time.
            let threshold = (totality_closeness * 10.0) as u32;
            if hash((s as u32).wrapping_add((ctx.time * 5.0) as u32)) % 10 < threshold {
                draw::dot_i(grid, px, py);
            }
        }

        // Eclipse track: a thin baseline along the bottom edge fills
        // left-to-right with eased, keeping progress legible in monochrome.
        let track_x = ((ctx.eased * (w - 1) as f32) as usize).min(w - 1);
        if track_x > 0 {
            draw::hline(grid, 0, track_x, h - 1);
        }

        // Tint palette
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9 — Gravitational lensing
// ---------------------------------------------------------------------------
// Structural idea: dots from a "background" grid are displaced radially
// outward from a central lens mass, creating an Einstein-ring arc pattern.
// The ring radius grows with eased. Pure dot-displacement geometry — unlike
// any other style which has no concept of optical warping.

/// Gravitational lens: background light bends around a mass; Einstein ring at eased radius.
struct GravitationalLens;
impl ProgressStyle for GravitationalLens {
    fn name(&self) -> &str {
        "grav-lens"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Einstein ring arc: background dots deflect around a central mass; ring radius = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let max_r = cx.min(cy * 1.8);
        // Einstein ring radius grows with eased.
        let ring_r = ctx.eased * max_r * 0.85;

        // Draw the Einstein ring arc (full ring at 100%, partial arc otherwise).
        let ring_steps = 80usize;
        let arc_frac = ctx.eased.min(1.0);
        let lit_steps = (arc_frac * ring_steps as f32) as usize;
        for s in 0..lit_steps.min(ring_steps) {
            let a = s as f32 / ring_steps as f32 * 2.0 * PI;
            // Ring thickness: 2 dots.
            for dr in [-1i32, 0, 1] {
                let r = (ring_r as i32 + dr).max(0) as f32;
                let px = cx + r * a.cos();
                let py = cy + r * a.sin() * 0.5;
                if hash((s as u32).wrapping_add(dr.unsigned_abs() * 199)) % 3 != 0 {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Lensing smear arcs — shorter arcs between ring and lens mass showing
        // deflected background sources.
        let source_count = 6u32;
        for src in 0..source_count {
            let src_angle = hash_f(src) * 2.0 * PI;
            let src_r = max_r * (0.5 + hash_f(src + 100) * 0.45);
            // Source position (outside ring).
            let _sx = cx + src_r * src_angle.cos();
            let _sy = cy + src_r * src_angle.sin() * 0.5;

            // Arc of deflected image: a short curve at ring_r, on the far side.
            let arc_center_a = src_angle + PI; // opposite side from source
            let arc_half = 0.25;
            let arc_n = 12usize;
            for k in 0..arc_n {
                let t = k as f32 / arc_n.saturating_sub(1).max(1) as f32;
                let a = (arc_center_a - arc_half) + t * 2.0 * arc_half;
                let r = ring_r * (0.9 + 0.15 * (PI * t).sin());
                let px = cx + r * a.cos();
                let py = cy + r * a.sin() * 0.5;
                if ctx.eased > hash_f(src + 200) * 0.5 {
                    draw::dot_i(grid, px as i32, py as i32);
                }
            }
        }

        // Central lens mass: compact dot cluster.
        for dy in -1i32..=1 {
            draw::dot_i(grid, cx as i32, cy as i32 + dy);
        }
        draw::dot_i(grid, cx as i32 - 1, cy as i32);
        draw::dot_i(grid, cx as i32 + 1, cy as i32);

        // Tint
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0).max(1.0);
            let t = 1.0 - dist * 0.6;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10 — Cosmic web
// ---------------------------------------------------------------------------
// Structural idea: a set of stable node positions connected by line filaments.
// Filaments appear one by one as eased grows (like galaxy filaments forming).
// Nodes pulse in brightness via time. Structurally: graph edges, not rings,
// not shells, not columns, not sweeping beams.

/// Cosmic web: galaxy nodes connected by filaments that appear as eased grows.
struct CosmicWeb;
impl ProgressStyle for CosmicWeb {
    fn name(&self) -> &str {
        "cosmic-web"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Cosmic web: node galaxies connected by filaments appearing as eased grows; nodes pulse"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        const NUM_NODES: u32 = 10;
        // Stable node positions (hash-seeded, small margin from edges).
        let nodes: Vec<(i32, i32)> = (0..NUM_NODES)
            .map(|i| {
                let nx = (hash_f(i) * (w.saturating_sub(4)) as f32 + 2.0) as i32;
                let ny = (hash_f(i + 200) * (h.saturating_sub(2)) as f32 + 1.0) as i32;
                (nx, ny)
            })
            .collect();

        // Filaments: connect each node to its two nearest by index
        // (i → i+1, i → i+2 wrapping) — gives a sparse web structure.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for i in 0..NUM_NODES as usize {
            edges.push((i, (i + 1) % NUM_NODES as usize));
            edges.push((i, (i + 2) % NUM_NODES as usize));
            // One cross-brace to make it feel 3-D.
            edges.push((i, (i + NUM_NODES as usize / 2) % NUM_NODES as usize));
        }
        // Remove exact duplicates (normalise so a < b).
        edges.iter_mut().for_each(|(a, b)| {
            if *a > *b {
                std::mem::swap(a, b);
            }
        });
        edges.sort_unstable();
        edges.dedup();

        let lit_edges = (ctx.eased * edges.len() as f32) as usize;

        // Draw revealed filaments.
        for &(a, b) in edges.iter().take(lit_edges) {
            let (ax, ay) = nodes[a];
            let (bx, by) = nodes[b];
            let dx = (bx - ax).abs();
            let dy = (by - ay).abs();
            let steps = dx.max(dy).max(1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = ax + ((bx - ax) as f32 * t) as i32;
                let py = ay + ((by - ay) as f32 * t) as i32;
                // Filament has gaps — sparse, like gas strands.
                if hash(
                    (s as u32)
                        .wrapping_mul(a as u32 * 13 + b as u32 * 7 + 1)
                        .wrapping_add((ctx.time * 2.0) as u32),
                ) % 4
                    != 0
                {
                    draw::dot_i(grid, px, py);
                }
            }
        }

        // Node dots — bright cluster, pulse via time.
        for (i, &(nx, ny)) in nodes.iter().enumerate() {
            let pulse = (ctx.time * 1.5 + i as f32 * 0.9).sin() * 0.5 + 0.5;
            // Cross cluster; size modulated by pulse.
            let size = if pulse > 0.5 { 2i32 } else { 1i32 };
            for dy in -size..=size {
                for dx in -size..=size {
                    if dx.abs() + dy.abs() <= size {
                        draw::dot_i(grid, nx + dx, ny + dy);
                    }
                }
            }
        }

        // Tint with palette across width.
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11 — Redshift wavefronts
// ---------------------------------------------------------------------------
// Structural idea: concentric horizontal wavefronts (hlines) expanding from a
// central source. Their spacing grows with eased (wavelength stretches = redshift).
// No circles — purely horizontal bands whose spacing is the progress indicator.

/// Redshift: wavefronts stretch horizontally from centre; spacing = eased wavelength.
struct Redshift;
impl ProgressStyle for Redshift {
    fn name(&self) -> &str {
        "redshift"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Redshift wavefronts: horizontal bands from source; spacing stretches with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Source at vertical centre.
        let cy = (h / 2) as f32;

        // Wavelength stretches: at eased=0 waves are packed (short λ, blue);
        // at eased=1 they are widely spaced (long λ, red).
        let min_lambda = 2.0f32;
        let max_lambda = (h as f32 * 0.45).max(min_lambda + 0.5);
        let lambda = min_lambda + ctx.eased * (max_lambda - min_lambda);

        // Emit wave fronts from cy; they scroll outward via time.
        // Offset = time * wave_speed mod lambda.
        let wave_speed = 3.0f32;
        let phase_offset = (ctx.time * wave_speed) % lambda.max(0.01);

        // Draw wavefronts above and below the source.
        let mut y_up = cy - phase_offset;
        let mut y_dn = cy + phase_offset;

        let max_waves = 20usize;
        for _wv in 0..max_waves {
            // Draw a horizontal line at this y position.
            let iy_up = y_up.round() as i32;
            let iy_dn = y_dn.round() as i32;
            if iy_up >= 0 {
                // Partial width: wavefront amplitude decays with distance from source.
                let dist_frac = (cy - y_up) / cy.max(1.0);
                let width_frac = (1.0 - dist_frac * 0.4).clamp(0.1, 1.0);
                let x_margin = ((1.0 - width_frac) * w as f32 * 0.5) as usize;
                draw::hline(
                    grid,
                    x_margin,
                    w.saturating_sub(x_margin + 1),
                    iy_up as usize,
                );
            }
            if iy_dn < h as i32 && iy_dn != iy_up {
                let dist_frac = (y_dn - cy) / (h as f32 - cy).max(1.0);
                let width_frac = (1.0 - dist_frac * 0.4).clamp(0.1, 1.0);
                let x_margin = ((1.0 - width_frac) * w as f32 * 0.5) as usize;
                draw::hline(
                    grid,
                    x_margin,
                    w.saturating_sub(x_margin + 1),
                    iy_dn as usize,
                );
            }

            y_up -= lambda;
            y_dn += lambda;

            if y_up < 0.0 && y_dn >= h as f32 {
                break;
            }
        }

        // Source dot at centre.
        draw::dot_i(grid, (w / 2) as i32, cy as i32);

        // Tint: blue at eased=0, red at eased=1 (redshift colour shift).
        let (cells_w, cells_h) = grid.dimensions();
        for cy_c in 0..cells_h {
            let t = ctx.eased; // fixed tint driven by eased, not position
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_c, 0, cells_w.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12 — Quasar jet
// ---------------------------------------------------------------------------
// Structural idea: two opposed straight-line relativistic jets fire along the
// vertical axis from a central nucleus; jet length = eased * half-height.
// Knots (bright blobs) travel along the jet at speed driven by time.
// A small accretion torus ring is drawn perpendicular to the jets.
// The structural element is a VLINE-based bidirectional jet — unique.

/// Quasar jet: twin relativistic jets fire along vertical axis; length = eased.
struct QuasarJet;
impl ProgressStyle for QuasarJet {
    fn name(&self) -> &str {
        "quasar-jet"
    }
    fn theme(&self) -> &str {
        "cosmos"
    }
    fn describe(&self) -> &str {
        "Quasar twin jets fire vertically from nucleus; length = eased; knots travel via time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;

        // Jet length grows with eased.
        let jet_len = (ctx.eased * cy as f32) as i32;

        // Draw primary jet spine (3 dots wide for visibility).
        for jet_dir in [-1i32, 1i32] {
            for offset in [-1i32, 0, 1] {
                let col = (cx + offset).max(0);
                let y_end = cy + jet_dir * jet_len;
                let y0 = cy.min(y_end).max(0) as usize;
                let y1 = cy.max(y_end).min(h as i32 - 1) as usize;
                draw::vline(grid, col as usize, y0, y1);
            }
        }

        // Bright knots travelling outward along each jet.
        let knot_count = 4usize;
        let knot_speed = 5.0f32;
        for jet_dir in [-1i32, 1i32] {
            for k in 0..knot_count {
                let phase = k as f32 / knot_count as f32;
                let knot_pos = ((ctx.time * knot_speed + phase * jet_len as f32)
                    % jet_len.max(1) as f32) as i32;
                let ky = cy + jet_dir * knot_pos;
                // Cross-shaped bright knot.
                for dy in -1i32..=1 {
                    for dx in -2i32..=2 {
                        if dx.abs() + dy.abs() <= 2 {
                            draw::dot_i(grid, cx + dx, ky + dy);
                        }
                    }
                }
            }
        }

        // Accretion torus: small horizontal ellipse around the nucleus.
        let torus_rx = (w / 6).max(2) as i32;
        let torus_ry = 1i32;
        let torus_steps = 32usize;
        for s in 0..torus_steps {
            let a = s as f32 / torus_steps as f32 * 2.0 * PI;
            let px = cx + (torus_rx as f32 * a.cos()) as i32;
            let py = cy + (torus_ry as f32 * a.sin()) as i32;
            draw::dot_i(grid, px, py);
        }

        // Nucleus.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Tint: jet colour from palette top→bottom.
        let (cells_w, cells_h) = grid.dimensions();
        for cy_c in 0..cells_h {
            let t = cy_c as f32 / cells_h.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(1.0 - t);
            draw::tint_row(grid, cy_c, 0, cells_w.saturating_sub(1), color);
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
    let styles = progress::styles::cosmos::styles();
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
