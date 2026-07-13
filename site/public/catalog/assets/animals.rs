//! `animals` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O animals.rs && ./animals [style-name]
//! ```

const DEFAULT_STYLE: &str = "caterpillar";

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
    pub mod animals {
//! Animals-themed progress bars — ten distinct creatures drawn in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (for perpetual
//! animation) and `ctx.eased` (for progress-driven advancement). The bars use
//! only the `draw::` helpers so all writes are silently bounds-safe.
//!
//! Styles in this file:
//! - `caterpillar`  — segmented body that undulates across the bar
//! - `snail`        — shell advancing over a slime trail
//! - `inchworm`     — bunching and stretching segments via eased spacing
//! - `fish-school`  — sine-bobbing dots swimming in formation
//! - `snake`        — sine-wave lateral slither advancing with progress
//! - `rabbit-hops`  — parabolic jump arcs across discrete hop chunks
//! - `paw-prints`   — alternating offset prints appearing one by one
//! - `bird-flock`   — V-formation sweeping across the bar
//! - `turtle`       — dome shell that fills from the bottom up
//! - `ant-march`    — ants carrying the load in single file

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — meadow green into honey. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(124, 204, 92);
const TINT_END: Color = Color::rgb(240, 184, 64);

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

/// All styles in the `animals` theme.
///
/// Returns one boxed [`ProgressStyle`] per animal variant, ready to be mixed
/// into a gallery or driven individually.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Caterpillar)),
        Box::new(Tinted(Snail)),
        Box::new(Tinted(Inchworm)),
        Box::new(FishSchool),
        Box::new(Tinted(Snake)),
        Box::new(Tinted(RabbitHops)),
        Box::new(PawPrints),
        Box::new(BirdFlock),
        Box::new(Tinted(Turtle)),
        Box::new(AntMarch),
    ]
}

// ── Caterpillar ──────────────────────────────────────────────────────────────

struct Caterpillar;
impl ProgressStyle for Caterpillar {
    fn name(&self) -> &str {
        "caterpillar"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Segmented caterpillar body undulating across the bar as it advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = h / 2;
        // How far the head has advanced (in dots).
        let head_x = (ctx.eased * w as f32) as usize;
        let seg_spacing = 4usize;
        // Phase offset scrolls with time for continuous crawl motion.
        let phase_offset = ctx.time * 6.0;
        // Draw body segments from tail to head.
        let mut x = 0usize;
        while x < head_x {
            // Sine-wave vertical displacement — body undulates.
            let wave = ((x as f32 * 0.4 + phase_offset) * 1.0).sin();
            let amp = ((h / 2).saturating_sub(1).max(1)) as f32 * 0.6;
            let y = (mid as f32 + wave * amp).round() as usize;
            let y = y.min(h - 1);
            // Segment body dot (slightly wider near center).
            draw::dot(grid, x, y);
            if x + 1 < head_x {
                draw::dot(grid, x + 1, y);
            }
            // Every seg_spacing dots, draw a leg nub above and below.
            if (x / seg_spacing) % 2 == 0 {
                if y + 1 < h {
                    draw::dot(grid, x, y + 1);
                }
                if y >= 1 {
                    draw::dot(grid, x, y - 1);
                }
            }
            x += seg_spacing;
        }
        // Head: two dots at the leading edge with antennae.
        if head_x > 0 {
            let hx = head_x.min(w - 1);
            let hy = mid.min(h - 1);
            draw::dot(grid, hx, hy);
            // Antenna: two dots diagonally up.
            draw::dot_i(grid, hx as i32 + 1, hy as i32 - 1);
            draw::dot_i(grid, hx as i32 + 1, hy as i32 - 2);
            draw::dot_i(grid, hx as i32 + 2, hy as i32 - 1);
        }
        Ok(())
    }
}

// ── Snail ─────────────────────────────────────────────────────────────────────

struct Snail;
impl ProgressStyle for Snail {
    fn name(&self) -> &str {
        "snail"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Snail leaving a slime trail — shell advances, trail fills behind it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = (h - 1).min(h.saturating_sub(1));
        let head_x = (ctx.eased * w as f32) as usize;
        // Slime trail: a single dot line along the base behind the snail.
        if head_x > 0 {
            draw::hline(grid, 0, head_x.saturating_sub(1).min(w - 1), base);
            // Slime shimmer — occasional dots one row above the trail.
            let shimmer_period = 8usize;
            let mut sx = 0usize;
            while sx < head_x.saturating_sub(2) {
                draw::dot(grid, sx, base.saturating_sub(1));
                sx += shimmer_period;
            }
        }
        // Shell: a small dome at head_x.
        let sx = head_x.min(w.saturating_sub(1));
        let shell_h = (h / 2).max(2);
        let shell_w = (shell_h).min(w.saturating_sub(sx));
        // Dome outline: arc of dots.
        for i in 0..=shell_w {
            let t = if shell_w == 0 {
                0.5
            } else {
                i as f32 / shell_w as f32
            };
            let arc_y = (base as f32 - (1.0 - (t * PI).sin() * (shell_h as f32 - 1.0)).max(0.0))
                .round() as usize;
            let arc_y = arc_y.max(base.saturating_sub(shell_h));
            draw::dot(grid, (sx + i).min(w - 1), arc_y.min(h - 1));
        }
        // Shell spiral (time-animated bob).
        let bob = ((ctx.time * 3.0).sin() * 0.4).round() as i32;
        let inner_x = sx as i32 + shell_w as i32 / 2;
        let inner_y = base as i32 - shell_h as i32 / 2 + bob;
        draw::dot_i(grid, inner_x, inner_y);
        // Head: eyestalk.
        let eye_x = sx as i32 + shell_w as i32 + 1;
        draw::dot_i(grid, eye_x, base as i32);
        draw::dot_i(grid, eye_x, base as i32 - 1);
        draw::dot_i(grid, eye_x + 1, base as i32 - 2);
        Ok(())
    }
}

// ── Inchworm ─────────────────────────────────────────────────────────────────

struct Inchworm;
impl ProgressStyle for Inchworm {
    fn name(&self) -> &str {
        "inchworm"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Inchworm bunching and stretching — eased segment spacing gives organic motion"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = h / 2;
        // The inchworm body spans from 0 to head_x, split into N segments.
        let head_x = (ctx.eased * w as f32) as usize;
        let n_segs = 6usize;
        // Bunch/stretch phase: oscillates with time. 0 = fully bunched, 1 = stretched.
        let stretch_phase = ((ctx.time * 2.0).sin() * 0.5 + 0.5) as f32;
        // Map segment index → x position using an eased distribution.
        for seg in 0..=n_segs {
            let raw_t = if n_segs == 0 {
                0.5
            } else {
                seg as f32 / n_segs as f32
            };
            // Interpolate between bunched (quadratic cluster at head) and stretched (linear).
            let bunched = raw_t * raw_t; // cluster toward head
            let t = bunched * (1.0 - stretch_phase) + raw_t * stretch_phase;
            let sx = (t * head_x as f32) as usize;
            let sx = sx.min(w - 1);
            // Arch height: tallest in the middle of the body.
            let arch = (PI * raw_t).sin();
            let arch_h = (arch * (h / 2) as f32).round() as usize;
            let sy = mid.saturating_sub(arch_h).min(h - 1);
            draw::dot(grid, sx, sy);
            // Connect adjacent segments with a line.
            if seg > 0 {
                let prev_t_raw = (seg - 1) as f32 / n_segs as f32;
                let prev_bunched = prev_t_raw * prev_t_raw;
                let prev_t = prev_bunched * (1.0 - stretch_phase) + prev_t_raw * stretch_phase;
                let prev_x = (prev_t * head_x as f32) as usize;
                let prev_arch = (PI * prev_t_raw).sin();
                let prev_arch_h = (prev_arch * (h / 2) as f32).round() as usize;
                let prev_y = mid.saturating_sub(prev_arch_h).min(h - 1);
                // Draw midpoint bridging dot.
                let bx = (prev_x + sx) / 2;
                let by = (prev_y + sy) / 2;
                draw::dot(grid, bx.min(w - 1), by.min(h - 1));
            }
        }
        // Head features.
        if head_x > 0 {
            let hx = head_x.min(w.saturating_sub(1));
            draw::dot_i(grid, hx as i32 + 1, mid as i32);
            draw::dot_i(grid, hx as i32 + 1, mid as i32 - 1);
        }
        Ok(())
    }
}

// ── Fish School ───────────────────────────────────────────────────────────────

struct FishSchool;
impl ProgressStyle for FishSchool {
    fn name(&self) -> &str {
        "fish-school"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "School of fish bobbing in sine-wave formation, swimming with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let front_x = (ctx.eased * w as f32) as usize;
        // Number of fish scales with bar width.
        let n_fish = (w / 8).max(2).min(10);
        let amp = (h / 2).saturating_sub(1).max(1) as f32 * 0.8;
        let mid = h as f32 / 2.0;
        for i in 0..n_fish {
            // Each fish is offset behind the leader proportionally.
            let lag = i as f32 / n_fish as f32;
            // Fish x position: lead fish at front_x, others trail behind.
            let fish_x = (front_x as f32 * (1.0 - lag * 0.35)) as usize;
            let fish_x = fish_x.min(w.saturating_sub(2));
            // Vertical bobbing: each fish has a phase offset.
            let phase = i as f32 * (PI * 2.0 / n_fish as f32);
            let bob_y = mid + amp * (ctx.time * 4.0 + phase).sin();
            let fy = (bob_y.round() as usize).min(h - 1);
            // Fish body: two dots.
            draw::dot(grid, fish_x, fy);
            if fish_x + 1 < w {
                draw::dot(grid, fish_x + 1, fy);
            }
            // Fish tail: a dot forked behind.
            if fish_x >= 1 {
                draw::dot(grid, fish_x - 1, fy.saturating_sub(1).min(h - 1));
                draw::dot(grid, fish_x - 1, (fy + 1).min(h - 1));
            }
        }
        // Color: gradient across the school.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            for cx in 0..filled_cells.min(cells_w) {
                let t = if filled_cells <= 1 {
                    0.5
                } else {
                    cx as f32 / filled_cells as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Snake ─────────────────────────────────────────────────────────────────────

struct Snake;
impl ProgressStyle for Snake {
    fn name(&self) -> &str {
        "snake"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Snake slithering — sine-wave lateral offset along a horizontal advance"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = h as f32 / 2.0;
        let amp = (h as f32 / 2.0 - 1.0).max(0.5);
        let head_x = (ctx.eased * w as f32) as usize;
        // Phase scrolls with time for continuous slither.
        let phase = ctx.time * 5.0;
        // Draw snake body from tail to head.
        for x in 0..head_x.min(w) {
            // Lateral sine offset; frequency increases toward the head (tighter wriggle).
            let freq = 0.25 + (x as f32 / w.max(1) as f32) * 0.25;
            let y = mid + amp * (x as f32 * freq + phase).sin();
            let y = (y.round() as usize).min(h - 1);
            draw::dot(grid, x, y);
            // Slight body thickness — extra dot perpendicular.
            if x % 3 == 0 {
                let y2 = (y + 1).min(h - 1);
                draw::dot(grid, x, y2);
            }
        }
        // Head: tongue flick (time-based).
        if head_x > 0 {
            let hx = head_x.min(w - 1);
            let hy_f = mid + amp * (hx as f32 * 0.5 + phase).sin();
            let hy = (hy_f.round() as usize).min(h - 1);
            // Tongue: two dots above the head at a tongue-flick interval.
            let tongue = ((ctx.time * 3.0).sin() > 0.4) as usize;
            if tongue > 0 {
                draw::dot_i(grid, hx as i32 + 1, hy as i32 - 1);
                draw::dot_i(grid, hx as i32 + 2, hy as i32 - 2);
                draw::dot_i(grid, hx as i32 + 2, hy as i32);
            } else {
                draw::dot_i(grid, hx as i32 + 1, hy as i32);
            }
            // Eye.
            draw::dot(grid, hx, hy.saturating_sub(1));
        }
        Ok(())
    }
}

// ── Rabbit Hops ───────────────────────────────────────────────────────────────

struct RabbitHops;
impl ProgressStyle for RabbitHops {
    fn name(&self) -> &str {
        "rabbit-hops"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Rabbit hopping across in parabolic arcs — each hop a discrete progress chunk"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;
        let ground = hi - 1;

        // Meadow: dotted ground line across the full width, with grass tufts.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, ground);
        }
        for k in 0..(wi / 9 + 1) {
            let gx = k * 9 + 4;
            draw::dot_i(grid, gx, ground - 1);
            draw::dot_i(grid, gx + 1, ground - 2);
        }

        // Hop geometry: the rabbit crosses in discrete parabolic chunks.
        let n_hops = 4i32;
        let x_min = 7.0f32;
        let x_max = (wi as f32 - 9.0).max(x_min + 1.0);
        let hop_index_f = ctx.eased * n_hops as f32;
        let current_hop = (hop_index_f as i32).min(n_hops - 1);
        let hop_frac = (hop_index_f - current_hop as f32).clamp(0.0, 1.0);
        let hop_x = |hop: i32, t: f32| -> f32 {
            let s = (hop as f32 + t) / n_hops as f32;
            x_min + s * (x_max - x_min)
        };
        let peak = (hi as f32 * 0.19).max(2.0);
        let arc_y = |t: f32| -> f32 { ground as f32 - 2.0 + 4.0 * peak * t * (t - 1.0) };

        // Paw prints where earlier hops landed.
        for hop in 0..current_hop {
            let px = hop_x(hop, 1.0) as i32;
            draw::dot_i(grid, px, ground - 1);
            draw::dot_i(grid, px + 1, ground - 1);
            draw::dot_i(grid, px + 3, ground - 2);
            draw::dot_i(grid, px + 4, ground - 2);
        }

        // Dotted trajectory trail behind the rabbit within the current hop.
        // The dash pattern crawls with time (rate 1.5/s: +6 slots per 4 s loop).
        let dash = ((ctx.time * 1.5) as i32).rem_euclid(3);
        let trail_steps = 24i32;
        for i in 0..trail_steps {
            let t = i as f32 / trail_steps as f32;
            if t >= hop_frac {
                break;
            }
            if (i + dash) % 3 == 0 {
                draw::dot_i(grid, hop_x(current_hop, t) as i32, arc_y(t) as i32);
            }
        }

        // Rabbit anchor: bottom-centre of the body, riding the arc plus a
        // gentle time-driven bounce (0.25 Hz-multiple: seamless 4 s loop).
        let bounce = (ctx.time * PI).sin().abs() * 1.2;
        let rx = hop_x(current_hop, hop_frac).round() as i32;
        let ry = (arc_y(hop_frac) - bounce).round() as i32;

        // Body: filled ellipse (10 wide, 5 tall), facing right.
        for dy in -5i32..=0 {
            let fy = (dy as f32 + 2.5) / 3.0;
            let span = ((1.0 - fy * fy).max(0.0)).sqrt() * 4.6;
            let lo = (-span).round() as i32;
            let hi_x = span.round() as i32;
            for dx in lo..=hi_x {
                draw::dot_i(grid, rx + dx, ry + dy);
            }
        }
        // Head: filled disc ahead of the body, one blank dot left as the eye.
        for dy in -7i32..=-4 {
            for dx in 4i32..=8 {
                let ex = dx as f32 - 6.0;
                let ey = dy as f32 + 5.5;
                if ex * ex / 5.5 + ey * ey / 3.2 <= 1.0 && !(dx == 7 && dy == -6) {
                    draw::dot_i(grid, rx + dx, ry + dy);
                }
            }
        }
        // Ears: two slanted lines; the back ear flicks on a 2/s time slot.
        let flick = ((ctx.time * 2.0) as i32).rem_euclid(2);
        for j in 1..=3i32 {
            draw::dot_i(grid, rx + 4, ry - 7 - j);
            draw::dot_i(grid, rx + 6 + (j > 2) as i32 * flick, ry - 7 - j);
        }
        // Tail puff and airborne feet.
        draw::dot_i(grid, rx - 5, ry - 4);
        draw::dot_i(grid, rx - 6, ry - 3);
        draw::dot_i(grid, rx - 3, ry + 1);
        draw::dot_i(grid, rx - 4, ry + 1);
        draw::dot_i(grid, rx + 3, ry + 1);
        Ok(())
    }
}

// ── Paw Prints ────────────────────────────────────────────────────────────────

struct PawPrints;
impl ProgressStyle for PawPrints {
    fn name(&self) -> &str {
        "paw-prints"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Paw prints appearing one by one, alternating left/right offset across the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let n_prints = 8usize;
        let visible = (ctx.eased * n_prints as f32).ceil() as usize;
        let mid = h / 2;
        let step_w = w / n_prints.max(1);
        let offset = (h / 4).max(1);
        for i in 0..visible.min(n_prints) {
            let px = i * step_w + step_w / 2;
            let px = px.min(w.saturating_sub(2));
            // Alternate left/right offset.
            let (py, small_off) = if i % 2 == 0 {
                (mid.saturating_sub(offset), 1usize)
            } else {
                ((mid + offset).min(h.saturating_sub(2)), 0usize)
            };
            // Pad: a 2×2 cluster.
            draw::dot(grid, px, py);
            draw::dot(grid, (px + 1).min(w - 1), py);
            draw::dot(grid, px, (py + 1).min(h - 1));
            draw::dot(grid, (px + 1).min(w - 1), (py + 1).min(h - 1));
            // Three toe dots above the pad.
            let toe_y = py.saturating_sub(1);
            draw::dot(grid, px.saturating_sub(1), toe_y);
            draw::dot(grid, px, toe_y.saturating_sub(small_off));
            draw::dot(grid, (px + 1).min(w - 1), toe_y);
            // Tint completed prints with palette color.
            let t = if n_prints <= 1 {
                1.0
            } else {
                i as f32 / (n_prints - 1) as f32
            };
            let color = ctx.palette.sample(t);
            let (cells_w, cells_h) = grid.dimensions();
            let cell_x = (px / 2).min(cells_w.saturating_sub(1));
            for cy in 0..cells_h {
                draw::tint_row(
                    grid,
                    cy,
                    cell_x,
                    (cell_x + 1).min(cells_w.saturating_sub(1)),
                    color,
                );
            }
        }
        Ok(())
    }
}

// ── Bird Flock ────────────────────────────────────────────────────────────────

struct BirdFlock;
impl ProgressStyle for BirdFlock {
    fn name(&self) -> &str {
        "bird-flock"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "V-formation bird flock sweeping across the bar, wings flapping with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = h / 2;
        let leader_x = (ctx.eased * w as f32) as usize;
        // Wing flap: oscillate between up and down positions.
        let flap = (ctx.time * 5.0).sin();
        let wing_spread = ((h / 4).max(1)) as f32;
        // Birds in a V: index 0 = leader, positive/negative index = left/right wing.
        let n_side = 3usize; // birds on each side of the V
        let v_step_x = 3usize; // horizontal spacing between birds in the V
        let v_step_y = 1usize; // vertical depth per step back in the V
                               // Draw leader.
        let lx = leader_x.min(w.saturating_sub(1));
        let ly = mid.min(h - 1);
        draw_bird(grid, lx, ly, flap, wing_spread, w, h);
        // Draw left and right wings.
        for side in 0..n_side {
            let offset_x = (side + 1) * v_step_x;
            let offset_y = (side + 1) * v_step_y;
            // Phase lag per bird — trailing birds flap slightly later.
            let lag = side as f32 * 0.4;
            let flap_bird = (ctx.time * 5.0 - lag).sin();
            // Left wing bird (above mid).
            let lx_left = leader_x.saturating_sub(offset_x);
            let ly_left = mid.saturating_sub(offset_y).min(h - 1);
            draw_bird(
                grid,
                lx_left.min(w.saturating_sub(1)),
                ly_left,
                flap_bird,
                wing_spread * 0.7,
                w,
                h,
            );
            // Right wing bird (below mid).
            let lx_right = leader_x.saturating_sub(offset_x);
            let ly_right = (mid + offset_y).min(h - 1);
            draw_bird(
                grid,
                lx_right.min(w.saturating_sub(1)),
                ly_right,
                flap_bird,
                wing_spread * 0.7,
                w,
                h,
            );
        }
        // Gradient tint on the swept region.
        let (cells_w, cells_h) = grid.dimensions();
        let swept = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            for cx in 0..swept.min(cells_w) {
                let t = if swept <= 1 {
                    0.0
                } else {
                    cx as f32 / (swept - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

/// Draw a single bird glyph at (bx, by) with a wing flap value in [-1, 1].
fn draw_bird(
    grid: &mut BrailleGrid,
    bx: usize,
    by: usize,
    flap: f32,
    wing_spread: f32,
    _w: usize,
    h: usize,
) {
    let wing_h = (flap * wing_spread * 0.5).round() as i32;
    // Body dot.
    draw::dot_i(grid, bx as i32, by as i32);
    // Left wing.
    draw::dot_i(grid, bx as i32 - 1, by as i32 + wing_h);
    draw::dot_i(
        grid,
        bx as i32 - 2,
        by as i32 + wing_h.abs().min((h / 2) as i32),
    );
    // Right wing.
    draw::dot_i(grid, bx as i32 + 1, by as i32 + wing_h);
    draw::dot_i(
        grid,
        bx as i32 + 2,
        by as i32 + wing_h.abs().min((h / 2) as i32),
    );
}

// ── Turtle ────────────────────────────────────────────────────────────────────

struct Turtle;
impl ProgressStyle for Turtle {
    fn name(&self) -> &str {
        "turtle"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Turtle shell dome that fills from bottom up as progress increases"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        // Shell dome: centered, width = 80% of bar, height = full bar.
        let shell_w = (w * 4 / 5).max(4).min(w);
        let x0 = w.saturating_sub(shell_w) / 2;
        let base = h - 1;
        // Draw dome outline using semi-ellipse.
        for xi in 0..=shell_w {
            let t = if shell_w == 0 {
                0.5
            } else {
                xi as f32 / shell_w as f32
            };
            // Ellipse: y = h * sqrt(1 - (2t-1)^2).
            let norm = 2.0 * t - 1.0;
            let ellipse_y = (h as f32 * (1.0 - norm * norm).sqrt()).min(h as f32 - 1.0);
            let top_y = base.saturating_sub(ellipse_y.round() as usize);
            draw::dot(grid, (x0 + xi).min(w - 1), top_y.min(h - 1));
        }
        // Bottom base line.
        draw::hline(grid, x0, (x0 + shell_w).min(w - 1), base);
        // Fill from base up to progress fraction of dome height.
        let fill_height = (ctx.eased * h as f32).round() as usize;
        for xi in 0..=shell_w {
            let t = if shell_w == 0 {
                0.5
            } else {
                xi as f32 / shell_w as f32
            };
            let norm = 2.0 * t - 1.0;
            let dome_height = (h as f32 * (1.0 - norm * norm).sqrt()).round() as usize;
            let col_fill = fill_height.min(dome_height);
            if col_fill > 0 {
                let y_top = base.saturating_sub(col_fill);
                draw::vline(grid, (x0 + xi).min(w - 1), y_top, base);
            }
        }
        // Shell pattern: diamond grid overlay (scute pattern), animated bobble.
        let bob = ((ctx.time * 1.5).sin() * 0.5) as i32;
        for row in (0..h).step_by(3) {
            for col in (x0..x0 + shell_w).step_by(4) {
                draw::dot_i(grid, col as i32, row as i32 + bob);
            }
        }
        // Head and tail (only partially visible, emerge at sides).
        draw::dot_i(grid, x0 as i32 - 1, base as i32);
        draw::dot_i(grid, x0 as i32 - 2, base as i32);
        draw::dot_i(grid, (x0 + shell_w) as i32 + 1, base as i32);
        // Legs.
        draw::dot_i(grid, x0 as i32 + 1, base as i32 + 1);
        draw::dot_i(grid, (x0 + shell_w) as i32 - 1, base as i32 + 1);
        Ok(())
    }
}

// ── Ant March ─────────────────────────────────────────────────────────────────

struct AntMarch;
impl ProgressStyle for AntMarch {
    fn name(&self) -> &str {
        "ant-march"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Ants marching in single file, carrying the load — legs animate with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let n_ants = (w / 7).max(1).min(8);
        let base = (h - 1).min(h.saturating_sub(1));
        let head_x = (ctx.eased * w as f32) as usize;
        // Ant spacing: packed in the filled region.
        let spacing = if n_ants <= 1 {
            head_x.max(1)
        } else {
            head_x / n_ants.max(1)
        };
        // Leg animation: alternating sets swing with time.
        let leg_phase = ctx.time * 8.0;
        for i in 0..n_ants {
            let ant_x = if spacing == 0 { 0 } else { i * spacing };
            if ant_x >= head_x.max(1) {
                break;
            }
            let ant_x = ant_x.min(w.saturating_sub(3));
            // Alternate leg sets: even ants phase-A, odd ants phase-B.
            let ant_leg_phase = leg_phase + i as f32 * PI / n_ants as f32;
            let leg_up = (ant_leg_phase.sin() > 0.0) as usize;
            // Ant: head (1 dot), thorax (1 dot), abdomen (1 dot).
            // Head.
            draw::dot(grid, (ant_x + 2).min(w - 1), base.saturating_sub(2));
            // Antennae — flicker with leg_up.
            draw::dot_i(grid, ant_x as i32 + 1, base as i32 - 3);
            draw::dot_i(grid, ant_x as i32 + 3, base as i32 - 3 + leg_up as i32);
            // Thorax.
            draw::dot(grid, (ant_x + 2).min(w - 1), base.saturating_sub(1));
            // Abdomen.
            draw::dot(grid, (ant_x + 2).min(w - 1), base);
            // Legs (3 pairs): alternate up/down with phase.
            for leg in 0..3usize {
                let leg_y_off = if (leg + leg_up) % 2 == 0 { 0i32 } else { 1i32 };
                // Left leg.
                draw::dot_i(
                    grid,
                    ant_x as i32 + 1,
                    base as i32 - leg as i32 + leg_y_off - 1,
                );
                // Right leg.
                draw::dot_i(
                    grid,
                    ant_x as i32 + 3,
                    base as i32 - leg as i32 + leg_y_off - 1,
                );
            }
        }
        // Gradient tint on the ant column.
        let (cells_w, cells_h) = grid.dimensions();
        let ant_cells = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            for cx in 0..ant_cells.min(cells_w) {
                let t = if ant_cells <= 1 {
                    0.0
                } else {
                    cx as f32 / (ant_cells - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
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
    let styles = progress::styles::animals::styles();
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
