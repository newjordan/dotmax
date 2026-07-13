//! `geometry` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O geometry.rs && ./geometry [style-name]
//! ```

const DEFAULT_STYLE: &str = "rose";

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
    pub mod geometry {
//! Sacred-geometry / parametric-curve progress bars.
//!
//! Every style maps `ctx.eased` onto the swept parameter range of a classical
//! curve so the shape **draws itself** as progress rises, while `ctx.time`
//! drives continuous rotation / animation.  Eleven distinct styles, each
//! implementing the real mathematical equations in dot space.
//!
//! Curve catalogue:
//! - `rose`          — Rhodonea / rose curve r = cos(k·θ)
//! - `spirograph`    — Hypotrochoid spirograph
//! - `epitrochoid`   — Epicycloid variant spirograph
//! - `superformula`  — Gielis superformula with animated exponents
//! - `phyllotaxis`   — Sunflower / golden-angle spiral
//! - `cardioid`      — String-art cardioid (multiplication table on a circle)
//! - `astroid`       — Astroid (4-cusp hypocycloid)
//! - `lemniscate`    — Lemniscate of Bernoulli (figure-eight)
//! - `maurer-rose`   — Maurer rose (chord-connected rose points)
//! - `mystic-rose`   — Mystic rose (all chords of an n-gon)
//! - `fermat-spiral` — Fermat's / parabolic spiral revealed by progress

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — cyan into violet.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(70, 200, 255);
const TINT_END: Color = Color::rgb(164, 96, 255);

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

/// All styles in the `geometry` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per parametric-curve bar.  The vector
/// is ordered from simplest (rose) to most elaborate (mystic rose), but all
/// styles are independent and can be used in any order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Rose)),
        Box::new(Tinted(Spirograph)),
        Box::new(Tinted(Epitrochoid)),
        Box::new(Tinted(Superformula)),
        Box::new(Tinted(Phyllotaxis)),
        Box::new(Tinted(Cardioid)),
        Box::new(Tinted(Astroid)),
        Box::new(Tinted(Lemniscate)),
        Box::new(Tinted(MaurerRose)),
        Box::new(Tinted(MysticRose)),
        Box::new(Tinted(FermatSpiral)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// Shared geometry helpers
// ────────────────────────────────────────────────────────────────────────────

/// Convert polar (r, theta) + grid center to dot-space (x, y), scale by
/// `scale` dots.  Returns signed ints so `draw::dot_i` can bounds-clip them.
#[inline]
fn polar_to_dot(cx: f32, cy: f32, r: f32, theta: f32, scale: f32) -> (i32, i32) {
    let x = cx + r * theta.cos() * scale;
    let y = cy - r * theta.sin() * scale; // y-axis flipped in screen space
    (x.round() as i32, y.round() as i32)
}

/// Compute a sensible uniform scale so a unit-radius curve fits the grid with
/// a small margin.  Uses the minimum half-dimension minus 1 dot of padding.
#[inline]
fn fit_scale(dw: usize, dh: usize) -> f32 {
    let hw = (dw as f32 / 2.0 - 1.0).max(1.0);
    let hh = (dh as f32 / 2.0 - 1.0).max(1.0);
    hw.min(hh)
}

/// Grid center in dot-space (floating-point).
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Rose curve (Rhodonea) — r = cos(k·θ)
// ────────────────────────────────────────────────────────────────────────────

struct Rose;
impl ProgressStyle for Rose {
    fn name(&self) -> &str {
        "rose"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Rhodonea rose r=cos(k·θ): petals unfurl from the center as progress rises, \
         the whole bloom slowly rotating with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // k=5 gives 5 petals (odd k → k petals, even k → 2k petals).
        let k: f32 = 5.0;
        // Full rose requires θ ∈ [0, π] for odd k; sweep up to eased·π.
        let theta_max = ctx.eased * PI;
        // Rotate the whole rose slowly with time.
        let rot = ctx.time * 0.3;

        let steps = (512.0 * ctx.eased).max(4.0) as usize;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let r = (k * theta).cos();
            let (x, y) = polar_to_dot(cx, cy, r, theta + rot, scale);
            draw::dot_i(grid, x, y);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Spirograph / hypotrochoid
// ────────────────────────────────────────────────────────────────────────────
//
//   x = (R − r)·cos θ + d·cos((R−r)/r · θ)
//   y = (R − r)·sin θ − d·sin((R−r)/r · θ)
//
// R=5, r=3, d=5 gives the classic "petals inside a ring" pattern.

struct Spirograph;
impl ProgressStyle for Spirograph {
    fn name(&self) -> &str {
        "spirograph"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Hypotrochoid spirograph: the inner-wheel trace of a circle rolling inside \
         a larger circle, drawing an intricate looping flower as progress sweeps θ"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let big_r: f32 = 5.0;
        let small_r: f32 = 3.0;
        let d: f32 = 5.0;
        let ratio = (big_r - small_r) / small_r;
        // Full period requires lcm(R,r)/r full rotations of θ.
        let theta_max = ctx.eased * 2.0 * PI * small_r; // ≈ 6π for (5,3)
        let rot = ctx.time * 0.2;

        let steps = (800.0 * ctx.eased).max(4.0) as usize;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let xf = (big_r - small_r) * theta.cos() + d * (ratio * theta).cos();
            let yf = (big_r - small_r) * theta.sin() - d * (ratio * theta).sin();
            // Normalise by the bounding radius so it fills the grid.
            let norm = big_r + d;
            let sx = cx + (xf / norm) * scale * (rot.cos());
            let sy = cy - (yf / norm) * scale;
            // Apply rotation in dot-space.
            let dx = xf / norm * scale;
            let dy = yf / norm * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            let _ = (sx, sy); // suppress unused warning
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Epitrochoid (epicycloid variant)
// ────────────────────────────────────────────────────────────────────────────
//
//   x = (R + r)·cos θ − d·cos((R+r)/r · θ)
//   y = (R + r)·sin θ − d·sin((R+r)/r · θ)

struct Epitrochoid;
impl ProgressStyle for Epitrochoid {
    fn name(&self) -> &str {
        "epitrochoid"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Epitrochoid: a small circle rolling *outside* a fixed circle traces \
         a multi-lobed crown; progress reveals the full pattern chord by chord"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // R=3, r=1, d=2.5 → 3-lobed limaçon-like shape.
        let big_r: f32 = 3.0;
        let small_r: f32 = 1.0;
        let d: f32 = 2.5;
        let ratio = (big_r + small_r) / small_r;
        let theta_max = ctx.eased * 2.0 * PI;
        let rot = ctx.time * 0.25;

        let steps = (600.0 * ctx.eased).max(4.0) as usize;
        let norm = big_r + small_r + d;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let xf = (big_r + small_r) * theta.cos() - d * (ratio * theta).cos();
            let yf = (big_r + small_r) * theta.sin() - d * (ratio * theta).sin();
            let dx = xf / norm * scale;
            let dy = yf / norm * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Gielis Superformula
// ────────────────────────────────────────────────────────────────────────────
//
//   r(φ) = [ |cos(m·φ/4)/a|^n2 + |sin(m·φ/4)/b|^n3 ]^(−1/n1)
//
// Animate n1 with time to morph between circle, star, and amoeba shapes.

struct Superformula;
impl ProgressStyle for Superformula {
    fn name(&self) -> &str {
        "superformula"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Gielis superformula: a single equation spanning circles, stars, flowers, \
         and alien blobs — n-exponents morph continuously with time as the curve \
         draws itself with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let m: f32 = 6.0; // symmetry order
        let a: f32 = 1.0;
        let b: f32 = 1.0;
        // Animate the exponents to morph the shape.
        let n1 = 1.5 + 2.5 * (ctx.time * 0.4).sin().abs();
        let n2 = 2.0 + 1.5 * (ctx.time * 0.31).cos();
        let n3 = 2.0 + 1.5 * (ctx.time * 0.27).sin();
        let rot = ctx.time * 0.18;

        let theta_max = ctx.eased * 2.0 * PI;
        let steps = (512.0 * ctx.eased).max(4.0) as usize;

        // Track max radius over the full sweep for normalization.
        // We pre-compute with the current n-values.
        let mut r_max: f32 = 0.001;
        for i in 0..=256 {
            let phi = i as f32 / 256.0 * 2.0 * PI;
            let t1 = ((m * phi / 4.0).cos() / a).abs().powf(n2);
            let t2 = ((m * phi / 4.0).sin() / b).abs().powf(n3);
            let sum = t1 + t2;
            if sum > 1e-6 {
                let r = sum.powf(-1.0 / n1);
                if r > r_max {
                    r_max = r;
                }
            }
        }

        for i in 0..=steps {
            let phi = i as f32 / steps as f32 * theta_max;
            let t1 = ((m * phi / 4.0).cos() / a).abs().powf(n2);
            let t2 = ((m * phi / 4.0).sin() / b).abs().powf(n3);
            let sum = t1 + t2;
            if sum < 1e-6 {
                continue;
            }
            let r = sum.powf(-1.0 / n1) / r_max;
            let dx = r * phi.cos() * scale;
            let dy = r * phi.sin() * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Phyllotaxis / sunflower (golden-angle spiral)
// ────────────────────────────────────────────────────────────────────────────
//
//   angle_n = n · 137.5°
//   radius_n = c · √n
//
// Plot the first eased·N points; N grows so the spiral fills as progress rises.

struct Phyllotaxis;
impl ProgressStyle for Phyllotaxis {
    fn name(&self) -> &str {
        "phyllotaxis"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Sunflower phyllotaxis: seeds appear at the golden angle 137.5°, \
         radius ∝ √n, producing the mesmerising Fibonacci spiral pattern of \
         real sunflowers and pinecones"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let golden_angle: f32 = 2.0 * PI * (1.0 - 1.0 / 1.618_033_9); // ≈ 137.508°
        let n_max: usize = 400;
        let n_plot = (ctx.eased * n_max as f32).round() as usize;
        // c chosen so the outermost seed lands near the grid edge.
        let c = scale / (n_max as f32).sqrt();
        let rot = ctx.time * 0.15;

        for n in 0..n_plot {
            let angle = n as f32 * golden_angle + rot;
            let r = c * (n as f32).sqrt();
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy - r * angle.sin()).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Cardioid via string-art (multiplication table on a circle)
// ────────────────────────────────────────────────────────────────────────────
//
//   Place N points evenly around a unit circle.
//   Draw chord from point i to point (2·i mod N).
//   The envelope of these chords is the cardioid r = a(1 − cos θ).

struct Cardioid;
impl ProgressStyle for Cardioid {
    fn name(&self) -> &str {
        "cardioid"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Cardioid string-art: connecting point i to (2·i mod N) on a circle \
         traces the cardioid r=a(1−cosθ) as an emergent envelope — chords appear \
         one by one as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh) * 0.9;

        let n_total: usize = 180; // total points on the circle
        let n_chords = (ctx.eased * n_total as f32).round() as usize;
        let rot = ctx.time * 0.2;

        for i in 0..n_chords {
            let j = (2 * i) % n_total;
            let a_i = 2.0 * PI * i as f32 / n_total as f32 + rot;
            let a_j = 2.0 * PI * j as f32 / n_total as f32 + rot;

            let x0 = (cx + a_i.cos() * scale).round() as i32;
            let y0 = (cy - a_i.sin() * scale).round() as i32;
            let x1 = (cx + a_j.cos() * scale).round() as i32;
            let y1 = (cy - a_j.sin() * scale).round() as i32;

            // Bresenham line between the two circle points.
            bresenham(grid, x0, y0, x1, y1);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Astroid (4-cusp hypocycloid)
// ────────────────────────────────────────────────────────────────────────────
//
//   x = R·cos³θ,  y = R·sin³θ   (hypocycloid with k=4, so r = R/4)

struct Astroid;
impl ProgressStyle for Astroid {
    fn name(&self) -> &str {
        "astroid"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Astroid (4-cusp hypocycloid): x=R·cos³θ, y=R·sin³θ — a star-shaped \
         curve with four sharp cusps that sweeps closed as progress completes a \
         full 2π revolution"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let theta_max = ctx.eased * 2.0 * PI;
        let rot = ctx.time * 0.22;
        let steps = (512.0 * ctx.eased).max(4.0) as usize;

        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let xf = theta.cos().powi(3);
            let yf = theta.sin().powi(3);
            let dx = xf * scale;
            let dy = yf * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Lemniscate of Bernoulli — r² = a²·cos(2θ)
// ────────────────────────────────────────────────────────────────────────────
//
//   Cartesian form: (x²+y²)² = a²(x²−y²)
//   Parametric:     x = a·cos(t)/(1+sin²t),  y = a·sin(t)·cos(t)/(1+sin²t)

struct Lemniscate;
impl ProgressStyle for Lemniscate {
    fn name(&self) -> &str {
        "lemniscate"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Lemniscate of Bernoulli: the ∞ figure-eight (x²+y²)²=a²(x²−y²), \
         traced with the rational parametric form — both lobes materialise \
         symmetrically as progress sweeps 0→2π"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let theta_max = ctx.eased * 2.0 * PI;
        let rot = ctx.time * 0.18;
        let steps = (512.0 * ctx.eased).max(4.0) as usize;

        for i in 0..=steps {
            let t = i as f32 / steps as f32 * theta_max;
            let denom = 1.0 + t.sin().powi(2);
            if denom.abs() < 1e-6 {
                continue;
            }
            // Rational parametric lemniscate (a = 1).
            let xf = t.cos() / denom;
            let yf = t.sin() * t.cos() / denom;
            let dx = xf * scale;
            let dy = yf * scale;
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            let px = (cx + rx).round() as i32;
            let py = (cy - ry).round() as i32;
            draw::dot_i(grid, px, py);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Maurer Rose
// ────────────────────────────────────────────────────────────────────────────
//
//   The rose curve r = cos(k·θ) evaluated at evenly spaced integer multiples
//   of d degrees, connected with straight chords.  d=71°, k=5 is classic.

struct MaurerRose;
impl ProgressStyle for MaurerRose {
    fn name(&self) -> &str {
        "maurer-rose"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Maurer rose: rose curve points at d-degree integer steps connected by \
         straight chords, producing a densely interlaced star web — d=71°, k=5 \
         gives the canonical intricate design"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let k: f32 = 5.0;
        let d_deg: f32 = 71.0; // step in degrees
        let d_rad = d_deg * PI / 180.0;
        let n_total: usize = 361; // one full revolution in d-degree steps
        let n_chords = (ctx.eased * n_total as f32).round() as usize;
        let rot = ctx.time * 0.15;

        // Compute successive chord endpoints.
        let point = |n: usize| -> (i32, i32) {
            let theta = n as f32 * d_rad + rot;
            let r = (k * theta).cos();
            let x = (cx + r * theta.cos() * scale).round() as i32;
            let y = (cy - r * theta.sin() * scale).round() as i32;
            (x, y)
        };

        for n in 0..n_chords.saturating_sub(1) {
            let (x0, y0) = point(n);
            let (x1, y1) = point(n + 1);
            bresenham(grid, x0, y0, x1, y1);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Mystic Rose — all chords of a regular n-gon
// ────────────────────────────────────────────────────────────────────────────
//
//   Place N points equally spaced on a circle and draw every chord.
//   Total chords = N·(N−1)/2.  Progress reveals them in order.

struct MysticRose;
impl ProgressStyle for MysticRose {
    fn name(&self) -> &str {
        "mystic-rose"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Mystic rose: every pair of vertices in a regular 24-gon connected by a \
         chord — 276 chords in total, appearing progressively to build a densely \
         woven stained-glass wheel"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let n: usize = 24; // 24-gon → 276 chords
        let total_chords = n * (n - 1) / 2;
        let chords_to_draw = (ctx.eased * total_chords as f32).round() as usize;
        let rot = ctx.time * 0.12;

        let vertex = |i: usize| -> (i32, i32) {
            let angle = 2.0 * PI * i as f32 / n as f32 + rot;
            let px = (cx + angle.cos() * scale).round() as i32;
            let py = (cy - angle.sin() * scale).round() as i32;
            (px, py)
        };

        let mut drawn = 0usize;
        'outer: for i in 0..n {
            for j in (i + 1)..n {
                if drawn >= chords_to_draw {
                    break 'outer;
                }
                let (x0, y0) = vertex(i);
                let (x1, y1) = vertex(j);
                bresenham(grid, x0, y0, x1, y1);
                drawn += 1;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Fermat's Spiral (parabolic spiral)
// ────────────────────────────────────────────────────────────────────────────
//
//   r = ±a·√θ,  θ ∈ [0, eased·θ_max]
//
// Both arms (±r) give the symmetric double-armed spiral.

struct FermatSpiral;
impl ProgressStyle for FermatSpiral {
    fn name(&self) -> &str {
        "fermat-spiral"
    }
    fn theme(&self) -> &str {
        "geometry"
    }
    fn describe(&self) -> &str {
        "Fermat's (parabolic) spiral r=±a√θ: both symmetric arms uncoil from the \
         origin as progress sweeps outward, the pair slowly rotating with time to \
         reveal a balanced double helix in braille dots"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // How many full revolutions to uncoil (6π ≈ 3 turns).
        let theta_max = ctx.eased * 6.0 * PI;
        let rot = ctx.time * 0.17;
        // Normalise so the outermost point lands near the grid edge.
        // At θ_max the radius = a·√θ_max; we want that ≈ 1.0 in unit coords.
        let a = 1.0 / (6.0 * PI).sqrt(); // ensures r ≤ 1 at full sweep

        let steps = (600.0 * ctx.eased).max(4.0) as usize;
        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let r = a * theta.sqrt();
            // Positive arm.
            let angle = theta + rot;
            let px = (cx + r * angle.cos() * scale).round() as i32;
            let py = (cy - r * angle.sin() * scale).round() as i32;
            draw::dot_i(grid, px, py);
            // Negative arm (π offset = opposite side).
            let qx = (cx - r * angle.cos() * scale).round() as i32;
            let qy = (cy + r * angle.sin() * scale).round() as i32;
            draw::dot_i(grid, qx, qy);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Internal: integer Bresenham line rasteriser
// ────────────────────────────────────────────────────────────────────────────

/// Draw a straight line between two signed dot-space points using Bresenham's
/// algorithm.  Out-of-bounds dots are silently discarded by `draw::dot_i`.
fn bresenham(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    // Bound the number of steps to avoid infinite loops on degenerate input.
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;

    loop {
        draw::dot_i(grid, x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::geometry::styles();
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
