//! `perspective` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O perspective.rs && ./perspective [style-name]
//! ```

const DEFAULT_STYLE: &str = "vp-tunnel";

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
    pub mod perspective {
//! Fake-3D-perspective progress bars — the **perspective** theme.
//!
//! Every style uses converging lines, vanishing-point geometry, and scrolling
//! depth cues as its *primary* communication mechanism.  `ctx.eased` encodes
//! *distance traveled into the scene* (0 = at the entrance, 1 = deep inside);
//! `ctx.time` drives continuous forward-motion animation (segments scroll
//! toward the viewer regardless of progress).
//!
//! # Style catalogue
//! | name | geometry |
//! |---|---|
//! | `vp-tunnel`        | Concentric rectangles → you fly in; rings scale and exit |
//! | `road-horizon`     | Two rails converging at a horizon; dashed center line scrolls |
//! | `starfield-dive`   | Stars streak from center outward; streak ∝ warp speed |
//! | `wire-corridor`    | Floor + ceiling grid lines converging to vanishing point |
//! | `checker-floor`    | Receding perspective checkerboard scrolling toward viewer |
//! | `infinite-hallway` | Doorframe rectangles at increasing depth; pass one per step |
//! | `mine-shaft`       | Descending shaft with horizontal floor markers |
//! | `depth-brackets`   | `[ [ [ > ] ] ]` nesting zooms to communicate depth |
//! | `approach-gate`    | A ring grows from the vanishing point until it fills the screen |
//! | `pipe-wormhole`    | Circular tunnel with radial spokes rotating as you travel |
//! | `parallax-layers`  | 3 horizontal line-layers scrolling at 3× different speeds |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// Integer Bresenham line rasteriser.  OOB dots are silently dropped.
/// Step count is bounded by `|dx|+|dy|+2` so no infinite loop is possible.
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx + dy.abs() + 2) as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Deterministic hash → float in [0, 1).
#[inline]
fn hf(n: u32) -> f32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);
    (x % 1000) as f32 / 1000.0
}

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — synthwave horizon.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(96, 162, 255);
const TINT_END: Color = Color::rgb(224, 96, 204);

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

/// All styles in the `perspective` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per fake-3D-perspective bar.  Each
/// style is geometrically distinct — they differ in their perspective
/// geometry, not merely in colour.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(VpTunnel)),
        Box::new(Tinted(RoadHorizon)),
        Box::new(Tinted(StarfieldDive)),
        Box::new(Tinted(WireCorridor)),
        Box::new(Tinted(CheckerFloor)),
        Box::new(Tinted(InfiniteHallway)),
        Box::new(Tinted(MineShaft)),
        Box::new(Tinted(DepthBrackets)),
        Box::new(Tinted(ApproachGate)),
        Box::new(Tinted(PipeWormhole)),
        Box::new(Tinted(ParallaxLayers)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Vanishing-point tunnel
// ────────────────────────────────────────────────────────────────────────────
//
// Concentric rectangles shrink toward the center vanishing point.
// `ctx.eased` = how deep you are → the innermost rings are drawn first,
// outermost last, so a full bar fills the screen edge-to-edge.
// `ctx.time` pushes the ring phases forward so they appear to fly past.

struct VpTunnel;
impl ProgressStyle for VpTunnel {
    fn name(&self) -> &str {
        "vp-tunnel"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Concentric rectangles converge to a center vanishing point; rings fly \
         toward the viewer — depth filled = eased, forward rush = time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = (dw / 2) as f32;
        let cy = (dh / 2) as f32;

        // How many rings to draw — more at larger grids.
        let n_rings: usize = 8.max(dw.min(dh) / 2);
        // Scroll phase: time moves rings toward viewer (outward).
        let scroll = (ctx.time * 0.4).fract();

        // Only draw rings that are within the depth the user has "reached".
        let depth_rings = ((ctx.eased * n_rings as f32).ceil() as usize)
            .min(n_rings)
            .max(1);

        for i in 0..depth_rings {
            // Ring parameter 0..1 where 0 = tiny center, 1 = full-screen edge.
            // Scroll moves rings outward over time.
            let raw = (i as f32 + scroll) / n_rings as f32;
            let t = raw.fract(); // wrap so rings scroll continuously

            // Scale: at t=0 a ring sits at the vanishing point; at t=1 it fills the screen.
            // Use non-linear spacing so far rings are denser.
            let s = t * t;
            let half_w = (cx * s).max(0.5) as i32;
            let half_h = (cy * s).max(0.5) as i32;
            if half_w <= 0 || half_h <= 0 {
                continue;
            }

            let x0 = (cx as i32 - half_w).max(0);
            let y0 = (cy as i32 - half_h).max(0);
            let x1 = (cx as i32 + half_w).min(dw as i32 - 1);
            let y1 = (cy as i32 + half_h).min(dh as i32 - 1);

            // Draw the four edges of the rectangle.
            line(grid, x0, y0, x1, y0);
            line(grid, x0, y1, x1, y1);
            line(grid, x0, y0, x0, y1);
            line(grid, x1, y0, x1, y1);

            // Corner diagonals to the vanishing point — the converging lines.
            line(grid, x0, y0, cx as i32, cy as i32);
            line(grid, x1, y0, cx as i32, cy as i32);
            line(grid, x0, y1, cx as i32, cy as i32);
            line(grid, x1, y1, cx as i32, cy as i32);
        }

        // Vanishing-point dot.
        draw::dot_i(grid, cx as i32, cy as i32);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Road to horizon
// ────────────────────────────────────────────────────────────────────────────
//
// Two rails converge at a horizon point on the vertical center.
// Dashed center line segments scroll toward the viewer (time).
// Progress = how far down the road the "you are here" marker has traveled.

struct RoadHorizon;
impl ProgressStyle for RoadHorizon {
    fn name(&self) -> &str {
        "road-horizon"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Two rails converge at a horizon point; dashed center line scrolls toward \
         the viewer — how far you've driven = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Horizon sits at the top third of the grid.
        let horizon_y = (dh / 3).max(1) as i32;
        let vp_x = (dw / 2) as i32; // vanishing point x

        // Road width at the bottom of the screen.
        let road_half = (dw as i32 / 4).max(2);

        // Left rail: from (vp_x, horizon_y) to (vp_x - road_half, dh-1).
        let left_bot = (vp_x - road_half).max(0);
        let right_bot = (vp_x + road_half).min(dw as i32 - 1);

        line(grid, vp_x, horizon_y, left_bot, dh as i32 - 1);
        line(grid, vp_x, horizon_y, right_bot, dh as i32 - 1);

        // Horizon line.
        draw::hline(grid, 0, dw - 1, horizon_y as usize);

        // Dashed center line: segments between the rails, scrolling toward viewer.
        // We sample several y positions below the horizon.
        let seg_count = 8usize;
        let scroll_phase = (ctx.time * 0.5).fract();

        for s in 0..seg_count {
            // Map segment index to a depth t in (0, 1], 0=horizon, 1=bottom.
            let t_base = (s as f32 + scroll_phase) / seg_count as f32;
            let t = t_base.fract();

            // y position in dot space (below horizon).
            let seg_y = (horizon_y as f32 + (dh as f32 - horizon_y as f32) * t) as i32;
            // x position: interpolate between vp and screen bottom x.
            let half_w = (road_half as f32 * t * 0.5) as i32;

            // Dash: draw only every other segment.
            if s % 2 == 0 {
                draw::dot_i(grid, vp_x, seg_y);
                if half_w > 1 {
                    draw::dot_i(grid, vp_x, seg_y - 1);
                    draw::dot_i(grid, vp_x, seg_y + 1);
                }
            }
        }

        // "You are here" marker: a horizontal bar across the road at the eased depth.
        // t = 0 → horizon, t = 1 → bottom of screen.
        let marker_t = ctx.eased;
        let marker_y = (horizon_y as f32 + (dh as f32 - horizon_y as f32) * marker_t) as i32;
        let marker_half = (road_half as f32 * marker_t) as i32;
        let mx0 = (vp_x - marker_half).max(0);
        let mx1 = (vp_x + marker_half).min(dw as i32 - 1);
        if marker_y >= 0 && marker_y < dh as i32 {
            line(grid, mx0, marker_y, mx1, marker_y);
        }

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Starfield dive
// ────────────────────────────────────────────────────────────────────────────
//
// Stars stream outward from the center; each star has a fixed angle and a
// radial position that scrolls with time.  Streak length grows with eased
// (= warp intensity).  Progress maps to how many stars are active.

struct StarfieldDive;
impl ProgressStyle for StarfieldDive {
    fn name(&self) -> &str {
        "starfield-dive"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Stars stream from the center vanishing point outward; streak length = \
         warp speed from eased; active-star count grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let max_r = cx.min(cy * 1.5).max(1.0);

        // Stars: fixed angle per index, radial position scrolls with time.
        let num_stars: u32 = 60;
        let active = ((ctx.eased * num_stars as f32).ceil() as u32).clamp(1, num_stars);
        // Warp intensity: longer streaks as progress grows.
        let streak_len = (ctx.eased * 12.0 + 1.0) as i32;
        let speed = 0.6 + ctx.eased * 1.8;

        for i in 0..active {
            let angle = hf(i) * 2.0 * PI;
            let phase = hf(i + 100);
            // Radial phase scrolls outward, wraps at 1.
            let r_frac = ((phase + ctx.time * speed * 0.07).fract()).clamp(0.0, 1.0);
            // Use quadratic growth for perspective (slow near center, fast at edge).
            let r = r_frac * r_frac * max_r;

            let px = cx + angle.cos() * r;
            // Vertical squish so stars don't look circular in wide terminals.
            let py = cy + angle.sin() * r * 0.55;

            // Streak: draw dots between the current and inward position.
            for s in 0..streak_len {
                let sr = (r - s as f32 * 0.8 * r_frac).max(0.0);
                let sx = cx + angle.cos() * sr;
                let sy = cy + angle.sin() * sr * 0.55;
                draw::dot_i(grid, sx as i32, sy as i32);
            }
            // Bright head.
            draw::dot_i(grid, px as i32, py as i32);
        }

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Wireframe corridor
// ────────────────────────────────────────────────────────────────────────────
//
// Floor and ceiling grid lines converge to a single vanishing point at the
// center of the screen.  Vertical wall stripes are equally spaced on screen
// and converge to the VP.  Floor lines scroll toward viewer via time.
// Eased controls how many vertical stripes / floor rows are visible.

struct WireCorridor;
impl ProgressStyle for WireCorridor {
    fn name(&self) -> &str {
        "wire-corridor"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Wireframe corridor: floor, ceiling, and wall grid lines converge to a \
         central vanishing point; floor lines scroll forward with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let vpx = (dw / 2) as i32;
        let vpy = (dh / 2) as i32;

        // Horizon line (corridor midline).
        draw::hline(grid, 0, dw - 1, vpy as usize);

        // Left and right wall edges.
        line(grid, 0, 0, vpx, vpy);
        line(grid, dw as i32 - 1, 0, vpx, vpy);
        line(grid, 0, dh as i32 - 1, vpx, vpy);
        line(grid, dw as i32 - 1, dh as i32 - 1, vpx, vpy);

        // Vertical wall stripes: several lines from the top screen edge to the
        // bottom edge, all converging to the VP.
        let stripe_count = 6usize;
        let lit_stripes = ((ctx.eased * stripe_count as f32).ceil() as usize)
            .min(stripe_count)
            .max(1);
        for s in 0..lit_stripes {
            // Evenly spaced x positions across the screen.
            let x = (s + 1) as i32 * dw as i32 / (stripe_count + 1) as i32;
            line(grid, x, 0, vpx, vpy);
            line(grid, x, dh as i32 - 1, vpx, vpy);
        }

        // Floor lines: horizontal segments between the two floor-edge lines
        // at various depths, scrolling toward the viewer.
        let floor_count = 8usize;
        let scroll = (ctx.time * 0.45).fract();
        for f in 0..floor_count {
            let t = ((f as f32 + scroll) / floor_count as f32).fract();
            // Map t → y below the horizon: 0=horizon, 1=bottom
            let fy = (vpy as f32 + (dh as f32 - vpy as f32) * t * t) as i32;
            if fy < 0 || fy >= dh as i32 {
                continue;
            }
            // Width of the floor line at this depth (wider near viewer).
            let fx_half = (dw as f32 / 2.0 * t).max(0.5) as i32;
            let fx0 = (vpx - fx_half).max(0);
            let fx1 = (vpx + fx_half).min(dw as i32 - 1);
            line(grid, fx0, fy, fx1, fy);
        }

        // Ceiling lines (mirror of floor above horizon).
        for f in 0..floor_count {
            let t = ((f as f32 + scroll) / floor_count as f32).fract();
            let fy = (vpy as f32 - (vpy as f32) * t * t) as i32;
            if fy < 0 || fy >= dh as i32 {
                continue;
            }
            let fx_half = (dw as f32 / 2.0 * t).max(0.5) as i32;
            let fx0 = (vpx - fx_half).max(0);
            let fx1 = (vpx + fx_half).min(dw as i32 - 1);
            line(grid, fx0, fy, fx1, fy);
        }

        // Vanishing point dot.
        draw::dot_i(grid, vpx, vpy);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Checkerboard floor
// ────────────────────────────────────────────────────────────────────────────
//
// A receding perspective-projected checkerboard occupies the lower half of the
// screen.  Row heights compress toward the horizon (1/depth spacing).  Time
// scrolls the checker pattern toward the viewer.  Eased controls how many
// rows (depth levels) are visible.

struct CheckerFloor;
impl ProgressStyle for CheckerFloor {
    fn name(&self) -> &str {
        "checker-floor"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Perspective checkerboard floor: rows compress toward the horizon; \
         pattern scrolls toward the viewer with time; depth = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let horizon_y = (dh / 3).max(1);
        // Draw the horizon.
        draw::hline(grid, 0, dw - 1, horizon_y);

        let vpx = dw / 2;
        let floor_h = dh - horizon_y; // dots below horizon

        // Number of depth rows to render (controlled by eased).
        let max_rows = 10usize;
        let visible_rows = ((ctx.eased * max_rows as f32).ceil() as usize)
            .min(max_rows)
            .max(1);

        // Scroll phase shifts checker rows forward over time.
        let scroll = (ctx.time * 0.35).fract();

        for row in 0..visible_rows {
            // depth t: 0 = nearest, 1 = at horizon.
            // We invert so row 0 is closest (bottom of screen).
            let t_raw = (row as f32 + scroll) / max_rows as f32;
            let t = t_raw.fract();
            // Inverse-depth: y approaches horizon_y as t → 1.
            let y = (horizon_y as f32 + floor_h as f32 * (1.0 - t * t)) as usize;
            if y >= dh {
                continue;
            }

            // Width of the floor at this depth.
            let half_w = (vpx as f32 * (1.0 - t * 0.9)).max(0.0) as usize;
            let x0 = vpx.saturating_sub(half_w);
            let x1 = (vpx + half_w).min(dw - 1);

            // Column count at this depth: fewer columns near horizon.
            let col_count = (half_w * 2 / 4).max(1);
            for col in 0..col_count {
                let cx0 = x0 + col * (x1 - x0 + 1) / col_count;
                let cx1 = x0 + (col + 1) * (x1 - x0 + 1) / col_count;
                // Checker: light only even columns on even depth rows, odd on odd.
                let checker = (row + col) % 2;
                if checker == 0 {
                    draw::hline(grid, cx0, cx1.min(dw - 1), y);
                }
            }
        }

        // Converging left and right floor edges.
        line(grid, 0, dh as i32 - 1, vpx as i32, horizon_y as i32);
        line(
            grid,
            dw as i32 - 1,
            dh as i32 - 1,
            vpx as i32,
            horizon_y as i32,
        );

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Infinite hallway
// ────────────────────────────────────────────────────────────────────────────
//
// A series of nested doorframe rectangles at increasing depth.  The viewer
// "passes through" one doorway per unit of progress.  Eased = how many
// doorways have been crossed; time shifts doorway phases so you continuously
// step through them.

struct InfiniteHallway;
impl ProgressStyle for InfiniteHallway {
    fn name(&self) -> &str {
        "infinite-hallway"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Nested doorframe rectangles at increasing depth — each door crossed = \
         one step of eased; doors scroll toward viewer with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;

        // Total doorways to draw.
        let n_doors = 8usize;
        // Time scrolls the phases forward (door indices advance).
        let scroll = (ctx.time * 0.4).fract();
        // Eased controls how many doorways are visible / how deep the view extends.
        let visible = ((ctx.eased * n_doors as f32).ceil() as usize)
            .min(n_doors)
            .max(1);

        for i in 0..visible {
            // t = 0 → smallest/farthest, t = 1 → largest/nearest.
            let raw = (i as f32 + scroll) / n_doors as f32;
            let t = raw.fract();
            // Non-linear scale: quadratic so far doors are densely packed.
            let s = t * t;

            let hw = ((dw as f32 / 2.0) * s).max(1.0) as i32;
            let hh = ((dh as f32 / 2.0) * s).max(1.0) as i32;

            let x0 = (cx - hw).max(0);
            let y0 = (cy - hh).max(0);
            let x1 = (cx + hw).min(dw as i32 - 1);
            let y1 = (cy + hh).min(dh as i32 - 1);

            // Draw the doorframe: top, left, right edges only (open at bottom = floor).
            line(grid, x0, y0, x1, y0); // top
            line(grid, x0, y0, x0, y1); // left
            line(grid, x1, y0, x1, y1); // right

            // Corner diagonals to the VP (depth cue).
            line(grid, x0, y0, cx, cy);
            line(grid, x1, y0, cx, cy);
        }

        // Vanishing-point marker.
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx - 1, cy);
        draw::dot_i(grid, cx + 1, cy);
        draw::dot_i(grid, cx, cy - 1);
        draw::dot_i(grid, cx, cy + 1);

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Mine shaft / elevator descent
// ────────────────────────────────────────────────────────────────────────────
//
// Vertical shaft with horizontal floor markers that scroll upward as you
// descend.  Eased = depth gauge on the right side (how far down you are).
// Time drives the upward scroll of floor markers.

struct MineShaft;
impl ProgressStyle for MineShaft {
    fn name(&self) -> &str {
        "mine-shaft"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Descending shaft: floor markers scroll upward with time; depth gauge \
         on the right fills with eased; you're falling down and in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Shaft walls: converge from wide at top to narrow near the bottom-center.
        let shaft_top_half = (dw as i32 / 3).max(2);
        let shaft_bot_half = (dw as i32 / 6).max(1);
        let vpx = (dw / 2) as i32;

        // Left and right shaft walls (converging vertical lines).
        line(
            grid,
            vpx - shaft_top_half,
            0,
            vpx - shaft_bot_half,
            dh as i32 - 1,
        );
        line(
            grid,
            vpx + shaft_top_half,
            0,
            vpx + shaft_bot_half,
            dh as i32 - 1,
        );

        // Horizontal floor markers scrolling upward (time).
        let marker_count = 10usize;
        let scroll = (ctx.time * 0.55).fract();

        for m in 0..marker_count {
            let t = ((m as f32 + scroll) / marker_count as f32).fract();
            // y: 0=top, 1=bottom (markers scroll from bottom to top as you descend).
            let y = (dh as f32 * (1.0 - t)) as i32;
            if y < 0 || y >= dh as i32 {
                continue;
            }

            // Shaft width at this y (interpolate between top and bottom half).
            let frac_y = y as f32 / dh as f32;
            let half_w =
                (shaft_top_half as f32 * (1.0 - frac_y) + shaft_bot_half as f32 * frac_y) as i32;
            let x0 = (vpx - half_w).max(0);
            let x1 = (vpx + half_w).min(dw as i32 - 1);
            line(grid, x0, y, x1, y);

            // Tick marks on the sides.
            draw::dot_i(grid, x0 - 1, y);
            draw::dot_i(grid, x1 + 1, y);
        }

        // Depth gauge: a filled column on the far right, filled proportional to eased.
        let gauge_x = (dw as i32 - 2).max(0);
        let filled_h = (ctx.eased * dh as f32).round() as i32;
        for y in 0..filled_h.min(dh as i32) {
            draw::dot_i(grid, gauge_x, y);
            draw::dot_i(grid, gauge_x + 1, y);
        }
        // Gauge outline.
        draw::vline(grid, gauge_x as usize, 0, dh - 1);
        draw::dot_i(grid, gauge_x, 0);
        draw::dot_i(grid, gauge_x, dh as i32 - 1);

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Depth brackets
// ────────────────────────────────────────────────────────────────────────────
//
// `[ [ [ > ] ] ]` — nested bracket pairs that zoom in, each pair one depth
// level deeper.  The number of visible bracket pairs = eased * max_depth.
// A `>` arrow (or a line of dots) shows the current position.
// Time causes the brackets to pulse / breathe slightly.

struct DepthBrackets;
impl ProgressStyle for DepthBrackets {
    fn name(&self) -> &str {
        "depth-brackets"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Nested [ [ [ > ] ] ] bracket pairs zoom inward; visible depth = \
         eased; inner marker pulses with time — pure typographic perspective"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;

        let max_depth = 6usize;
        let depth = ((ctx.eased * max_depth as f32).ceil() as usize)
            .min(max_depth)
            .max(1);

        // Subtle breathing animation via time.
        let breath = (ctx.time * 1.5).sin() * 0.04 + 1.0;

        for d in 0..depth {
            // The outermost bracket uses the full width; each inner one shrinks.
            let frac = (max_depth - d) as f32 / max_depth as f32;
            let hw = ((dw as f32 / 2.0) * frac * breath).max(1.0) as i32;
            let hh = ((dh as f32 / 2.0) * frac * breath).max(1.0) as i32;

            let x0 = (cx - hw).max(0);
            let y0 = (cy - hh).max(0);
            let x1 = (cx + hw).min(dw as i32 - 1);
            let y1 = (cy + hh).min(dh as i32 - 1);

            // Left bracket: vertical bar + horizontal serifs at top and bottom.
            draw::dot_i(grid, x0, y0);
            draw::dot_i(grid, x0, y1);
            draw::vline(grid, x0 as usize, y0 as usize, y1 as usize);
            // Short horizontal serifs.
            draw::dot_i(grid, x0 + 1, y0);
            draw::dot_i(grid, x0 + 1, y1);

            // Right bracket mirror.
            draw::dot_i(grid, x1, y0);
            draw::dot_i(grid, x1, y1);
            draw::vline(grid, x1 as usize, y0 as usize, y1 as usize);
            draw::dot_i(grid, x1 - 1, y0);
            draw::dot_i(grid, x1 - 1, y1);
        }

        // Center `>` marker: three dots forming an arrowhead.
        let pulse = ((ctx.time * 3.0).sin() * 1.5) as i32;
        draw::dot_i(grid, cx + pulse, cy);
        draw::dot_i(grid, cx + pulse - 1, cy - 1);
        draw::dot_i(grid, cx + pulse - 1, cy + 1);
        draw::dot_i(grid, cx + pulse - 2, cy - 2);
        draw::dot_i(grid, cx + pulse - 2, cy + 2);

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Approaching gate / portal
// ────────────────────────────────────────────────────────────────────────────
//
// A single elliptical ring starts tiny at the vanishing point and grows until
// it fills (and exits) the screen at progress=1.  Multiple rings at different
// phases stream toward the viewer; eased = how close the lead ring is.

struct ApproachGate;
impl ProgressStyle for ApproachGate {
    fn name(&self) -> &str {
        "approach-gate"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "A portal ring grows from the vanishing point until it fills the screen \
         at 100%; multiple rings stream toward the viewer continuously"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as f32;
        let cy = (dh / 2) as f32;
        let max_rx = cx;
        let max_ry = cy;

        // Stream of rings scrolling toward viewer.
        let n_rings = 5usize;
        let scroll = (ctx.time * 0.5).fract();

        // The lead ring's phase is clamped to eased (can't go further than progress).
        for i in 0..n_rings {
            let raw = (i as f32 + scroll) / n_rings as f32;
            let phase = raw.fract();
            // The lead ring (i=0 after scroll) is bounded by eased.
            // Others trail behind it.
            let t = if i == 0 {
                // Lead ring: its size is driven by eased, animated by scroll.
                // Blend: use the larger of the scroll-driven phase and eased.
                phase.max(ctx.eased * phase)
            } else {
                phase * ctx.eased
            };

            // Scale: small = far, large = near.
            let rx = (max_rx * t * t).max(0.5);
            let ry = (max_ry * t * t).max(0.5);

            // Draw ellipse via parametric steps.
            let steps = ((rx + ry) * 4.0).max(8.0) as usize;
            let mut prev: Option<(i32, i32)> = None;
            for s in 0..=steps {
                let angle = s as f32 / steps as f32 * 2.0 * PI;
                let px = (cx + rx * angle.cos()) as i32;
                let py = (cy + ry * angle.sin()) as i32;
                if let Some((ppx, ppy)) = prev {
                    line(grid, ppx, ppy, px, py);
                }
                prev = Some((px, py));
            }
        }

        // Vanishing-point dot.
        draw::dot_i(grid, cx as i32, cy as i32);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Pipe / wormhole
// ────────────────────────────────────────────────────────────────────────────
//
// A circular tunnel cross-section with radial spokes.  Spokes rotate with
// time (you're spinning through the pipe).  Multiple concentric circles give
// depth; eased controls how many depth rings are visible.

struct PipeWormhole;
impl ProgressStyle for PipeWormhole {
    fn name(&self) -> &str {
        "pipe-wormhole"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Circular pipe tunnel with radial spokes; spokes rotate with time as \
         you travel through — rings fill in with eased for depth"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let max_r = cx.min(cy).max(1.0);

        // Number of depth rings.
        let n_rings = 6usize;
        let visible = ((ctx.eased * n_rings as f32).ceil() as usize)
            .min(n_rings)
            .max(1);

        // Draw concentric circles from inside out.
        for r_idx in 0..visible {
            let t = (r_idx + 1) as f32 / n_rings as f32;
            let r = max_r * t;
            let steps = ((r * 2.0 * PI).max(8.0)) as usize;
            let mut prev: Option<(i32, i32)> = None;
            for s in 0..=steps {
                let angle = s as f32 / steps as f32 * 2.0 * PI;
                let px = (cx + r * angle.cos()) as i32;
                // Vertical squish so it looks more circular in tall-font terminals.
                let py = (cy + r * angle.sin() * 0.55) as i32;
                if let Some((ppx, ppy)) = prev {
                    line(grid, ppx, ppy, px, py);
                }
                prev = Some((px, py));
            }
        }

        // Radial spokes from center to outer ring, rotating with time.
        let spoke_count = 8usize;
        let rot = ctx.time * 0.8;
        let outer_r = max_r * (visible as f32 / n_rings as f32);

        for s in 0..spoke_count {
            let angle = rot + s as f32 * 2.0 * PI / spoke_count as f32;
            let x1 = (cx + outer_r * angle.cos()) as i32;
            let y1 = (cy + outer_r * angle.sin() * 0.55) as i32;
            line(grid, cx as i32, cy as i32, x1, y1);
        }

        // Center dot.
        draw::dot_i(grid, cx as i32, cy as i32);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Parallax layers
// ────────────────────────────────────────────────────────────────────────────
//
// Three distinct horizontal-line "planes" — foreground, midground, background
// — each scrolling horizontally at a different speed (3× ratio).  Broken
// lines give the impression of objects at different depths.  Eased controls
// how many planes are active; time drives all scroll speeds.

struct ParallaxLayers;
impl ProgressStyle for ParallaxLayers {
    fn name(&self) -> &str {
        "parallax-layers"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Three line-layers scroll at 3× speed ratios — background, midground, \
         foreground — giving pure parallax depth; layer count = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // How many layers are active.
        let n_layers = 3usize;
        let active = ((ctx.eased * n_layers as f32).ceil() as usize)
            .min(n_layers)
            .max(1);

        // Layer definitions: (y_fraction, speed_multiplier, dash_period_dots, dash_on_dots)
        // Background = slow, few dashes; foreground = fast, many dashes.
        let layers: [(f32, f32, usize, usize); 3] = [
            (0.25, 0.3, 8, 3), // background (top third)
            (0.55, 0.9, 6, 4), // midground
            (0.85, 2.2, 4, 3), // foreground (bottom)
        ];

        for (idx, &(y_frac, speed, period, on)) in layers.iter().enumerate() {
            if idx >= active {
                break;
            }

            let y = (dh as f32 * y_frac) as usize;
            if y >= dh {
                continue;
            }

            // Horizontal scroll offset in dots.
            let offset = ((ctx.time * speed * dw as f32 * 0.1) as usize) % period.max(1);

            for x in 0..dw {
                let phase = (x + offset) % period.max(1);
                if phase < on {
                    draw::dot(grid, x, y);
                    // Foreground layer gets a thicker line.
                    if idx == 2 && y + 1 < dh {
                        draw::dot(grid, x, y + 1);
                    }
                }
            }

            // Short vertical marks (parallax poles) at intervals — different
            // depths get different mark heights.
            let pole_h = match idx {
                0 => 1usize,
                1 => 2,
                _ => 3,
            };
            let pole_spacing = dw / (idx * 3 + 4).max(1);
            let pole_offset = ((ctx.time * speed * 0.5) as usize * pole_spacing / dw.max(1))
                % pole_spacing.max(1);
            let mut px = pole_offset;
            while px < dw {
                for py in 0..pole_h {
                    let dot_y = y + py + 1;
                    if dot_y < dh {
                        draw::dot(grid, px, dot_y);
                    }
                    if y >= py + 1 {
                        draw::dot(grid, px, y - py - 1);
                    }
                }
                px += pole_spacing.max(1);
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
    let styles = progress::styles::perspective::styles();
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
