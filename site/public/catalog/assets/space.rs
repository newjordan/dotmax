//! `space` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O space.rs && ./space [style-name]
//! ```

const DEFAULT_STYLE: &str = "rocket-launch";

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
    pub mod space {
//! Space / cosmic progress bars — ten distinct styles from rocket launches
//! to black-hole accretion disks, all animated via `ctx.time` and driven by
//! `ctx.eased` for fill amount. Every style returns `"space"` from `theme()`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic pseudo-random helper (no external crates).
// Returns a stable float in [0, 1) for star index `n`.
// ---------------------------------------------------------------------------
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

fn hash_f(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `space` theme, in display order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(RocketLaunch),
        Box::new(StarfieldWarp),
        Box::new(PlanetOrbit),
        Box::new(CometTail),
        Box::new(MoonPhase),
        Box::new(GalaxySpiral),
        Box::new(SatelliteDish),
        Box::new(SaturnRings),
        Box::new(BlackHole),
        Box::new(Constellation),
    ]
}

// ---------------------------------------------------------------------------
// 1 — Rocket launch
// ---------------------------------------------------------------------------

/// Rocket rises from left to right with eased body and a flickering exhaust plume.
struct RocketLaunch;
impl ProgressStyle for RocketLaunch {
    fn name(&self) -> &str {
        "rocket-launch"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Rocket advances with eased fill; exhaust flame flickers via time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Fuel trail — thin exhaust baseline
        let head = ((ctx.eased * w as f32) as usize).min(w.saturating_sub(1));
        let mid = h / 2;

        // Body: solid bar from 0..head
        draw::hline(grid, 0, head, mid);
        if h >= 3 {
            draw::hline(grid, 0, head, mid.saturating_sub(1));
        }

        // Tint the trail with palette gradient
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..filled_cells.min(cells_w) {
            let t = if filled_cells <= 1 {
                0.0
            } else {
                cx as f32 / (filled_cells - 1) as f32
            };
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        // Rocket nose — vertical stripe at head position
        if head < w {
            draw::vline(
                grid,
                head,
                mid.saturating_sub(1),
                (mid + 1).min(h.saturating_sub(1)),
            );
        }

        // Exhaust plume behind the rocket: flicker via time + sine
        if head > 0 {
            let plume_len = (h / 2).max(1);
            for p in 0..plume_len {
                let flicker = (ctx.time * 18.0 + p as f32 * 1.3).sin();
                let offset = (flicker * 1.5) as i32;
                let px = head.saturating_sub(p + 1);
                draw::dot_i(grid, px as i32, (mid as i32) + offset);
                if p < plume_len / 2 {
                    draw::dot_i(grid, px as i32, (mid as i32) + offset + 1);
                    draw::dot_i(grid, px as i32, (mid as i32) + offset - 1);
                }
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2 — Starfield warp
// ---------------------------------------------------------------------------

/// Stars streak outward from center, speed proportional to progress; density uses hash.
struct StarfieldWarp;
impl ProgressStyle for StarfieldWarp {
    fn name(&self) -> &str {
        "starfield-warp"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Stars streak from center outward; speed and streak length ramp with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let speed = 0.3 + ctx.eased * 2.5;
        let max_streak = (ctx.eased * 8.0 + 1.0) as usize;
        let num_stars: u32 = 64;

        for i in 0..num_stars {
            // Stable angle and base radius per star
            let angle = hash_f(i) * 2.0 * PI;
            let base_r = hash_f(i + 1000) * 0.5 + 0.1; // fraction of half-width
            let phase = hash_f(i + 2000); // time offset

            // Star travels outward; wrap via fract
            let r_frac = ((base_r + ctx.time * speed * 0.05 + phase).fract()).clamp(0.0, 1.0);
            let max_r = cx.min(cy);
            let r = r_frac * max_r;

            let sx = cx + angle.cos() * r;
            let sy = cy + angle.sin() * r * 0.6; // slightly squish vertically

            // Streak toward center (streak grows with progress + radius)
            let streak = ((r_frac * max_streak as f32) as usize).max(1);
            for s in 0..streak {
                let sr = (r - s as f32).max(0.0);
                let px = (cx + angle.cos() * sr) as i32;
                let py = (cy + angle.sin() * sr * 0.6) as i32;
                draw::dot_i(grid, px, py);
            }
            // Bright head
            draw::dot_i(grid, sx as i32, sy as i32);
        }

        // Tint with palette
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
// 3 — Planet orbit
// ---------------------------------------------------------------------------

/// A planet traces an elliptical orbit; angle = eased * 2π with a trailing tail.
struct PlanetOrbit;
impl ProgressStyle for PlanetOrbit {
    fn name(&self) -> &str {
        "planet-orbit"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Planet travels an elliptical orbit; position = eased * 2π with orbital tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let rx = (w as f32 * 0.42).max(1.0);
        let ry = (h as f32 * 0.38).max(1.0);

        // Draw faint orbit ellipse (every 4th dot)
        let steps = 80usize;
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            let ox = (cx + rx * a.cos()) as i32;
            let oy = (cy + ry * a.sin()) as i32;
            if s % 4 == 0 {
                draw::dot_i(grid, ox, oy);
            }
        }

        // Star / sun at center
        draw::dot_i(grid, cx as i32, cy as i32);
        draw::dot_i(grid, cx as i32 - 1, cy as i32);
        draw::dot_i(grid, cx as i32 + 1, cy as i32);
        draw::dot_i(grid, cx as i32, cy as i32 - 1);
        draw::dot_i(grid, cx as i32, cy as i32 + 1);

        // Planet angle driven by time for continuous animation, progress sets lap fraction
        let angle = ctx.time * 0.9 + ctx.eased * 2.0 * PI;

        // Trailing tail (10 ghost dots fading behind)
        let tail_len = 12usize;
        for t in 0..tail_len {
            let frac = (tail_len - t) as f32 / tail_len as f32;
            let ta = angle - (t as f32 * 0.15);
            let tx = (cx + rx * ta.cos()) as i32;
            let ty = (cy + ry * ta.sin()) as i32;
            // Sparser toward end of tail
            if hash(t as u32 * 7 + (ctx.time * 10.0) as u32) % 100 < (frac * 90.0) as u32 {
                draw::dot_i(grid, tx, ty);
            }
        }

        // Planet body (3-dot cluster)
        let px = (cx + rx * angle.cos()) as i32;
        let py = (cy + ry * angle.sin()) as i32;
        draw::dot_i(grid, px, py);
        draw::dot_i(grid, px + 1, py);
        draw::dot_i(grid, px, py + 1);
        draw::dot_i(grid, px + 1, py + 1);

        // Tint palette across cells
        let (cells_w, cells_h) = grid.dimensions();
        let color_start = ctx.palette.sample(0.0);
        let color_end = ctx.palette.sample(1.0);
        for cy_c in 0..cells_h {
            draw::tint_row(grid, cy_c, 0, cells_w / 2, color_start);
            draw::tint_row(
                grid,
                cy_c,
                cells_w / 2,
                cells_w.saturating_sub(1),
                color_end,
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4 — Comet with tail
// ---------------------------------------------------------------------------

/// Comet head at eased position; tail trails behind with decreasing dot density.
struct CometTail;
impl ProgressStyle for CometTail {
    fn name(&self) -> &str {
        "comet-tail"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Comet head at eased position; tail fades behind via decreasing dot density"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = h / 2;
        let head_x = ((ctx.eased * w as f32) as usize).min(w.saturating_sub(1));

        // Background stars (stable via hash)
        for i in 0u32..30 {
            let sx = (hash_f(i) * w as f32) as usize;
            let sy = (hash_f(i + 50) * h as f32) as usize;
            // Twinkle: occasionally skip
            if (hash(i + (ctx.time * 3.0) as u32) % 5) != 0 {
                draw::dot(grid, sx, sy);
            }
        }

        // Tail: comet path from 0 to head, density drops off from head toward origin
        let tail_len = head_x;
        for t in 0..tail_len {
            let dist_from_head = tail_len.saturating_sub(t);
            let frac = 1.0 - (dist_from_head as f32 / tail_len.max(1) as f32);
            // Probability of lighting a dot increases near the head
            let threshold = (frac * frac * 900.0) as u32;
            let roll = hash(
                (t as u32)
                    .wrapping_mul(31)
                    .wrapping_add((ctx.time * 5.0) as u32),
            ) % 1000;
            if roll < threshold {
                let spread = (frac * (h as f32 / 2.0)) as i32;
                let wobble = ((ctx.time * 4.0 + t as f32 * 0.2).sin() * spread as f32) as i32;
                draw::dot_i(grid, t as i32, mid as i32 + wobble);
                if spread > 1 {
                    draw::dot_i(grid, t as i32, mid as i32 + wobble - 1);
                }
            }
        }

        // Comet core — bright 3-dot head
        if head_x < w {
            draw::vline(
                grid,
                head_x,
                mid.saturating_sub(1),
                (mid + 1).min(h.saturating_sub(1)),
            );
            if head_x + 1 < w {
                draw::dot(grid, head_x + 1, mid);
            }
        }

        // Tint: warm orange→blue across the sweep
        let (cells_w, cells_h) = grid.dimensions();
        let head_cell = (ctx.eased * cells_w as f32) as usize;
        for cx in 0..head_cell.min(cells_w) {
            let t = cx as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5 — Moon phase
// ---------------------------------------------------------------------------

/// Disc whose illuminated fraction equals eased; terminator sweeps like a moon phase.
struct MoonPhase;
impl ProgressStyle for MoonPhase {
    fn name(&self) -> &str {
        "moon-phase"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Circular disc illuminated by eased fraction — sweeps new→full moon"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let r = ((w.min(h) / 2).saturating_sub(1)).max(1) as i32;

        // Phase: 0 = new moon (none lit), 1 = full moon (all lit)
        // Terminator is a vertical line sweeping left→right across the disc.
        // At phase=0.5 it's at center (half-moon); phase=1 terminator is at +r (all lit).
        // Terminator x-offset: (eased * 2 - 1) * r  →  ranges -r..+r
        let term_x = ((ctx.eased * 2.0 - 1.0) * r as f32) as i32;

        for dy in -r..=r {
            // Half-chord at this row
            let dx_max_sq = r * r - dy * dy;
            if dx_max_sq < 0 {
                continue;
            }
            let dx_max = (dx_max_sq as f32).sqrt() as i32;
            for dx in -dx_max..=dx_max {
                // Lit side: right of terminator (dx >= term_x)
                if dx >= term_x {
                    draw::dot_i(grid, cx + dx, cy + dy);
                } else {
                    // Dark side: draw sparse dots for the disc outline only
                    if dx == -dx_max || dy == -r || dy == r {
                        draw::dot_i(grid, cx + dx, cy + dy);
                    }
                }
            }
        }

        // Tint lit portion with pale yellow→white from palette
        let (cells_w, cells_h) = grid.dimensions();
        let lit_cells = (ctx.eased * cells_w as f32) as usize;
        let start_cell = cells_w.saturating_sub(lit_cells);
        for cy_c in 0..cells_h {
            draw::tint_row(
                grid,
                cy_c,
                start_cell,
                cells_w.saturating_sub(1),
                ctx.palette.sample(0.85),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6 — Galaxy spiral
// ---------------------------------------------------------------------------

/// Logarithmic spiral arms; dots lit up to eased fraction, arms rotate with time.
struct GalaxySpiral;
impl ProgressStyle for GalaxySpiral {
    fn name(&self) -> &str {
        "galaxy-spiral"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Two-armed logarithmic galaxy spiral rotating in time; arms fill to eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let max_r = cx.min(cy * 1.4);

        // Logarithmic spiral: r = a * e^(b * theta)
        let a: f32 = 0.8;
        let b: f32 = 0.18;
        let num_arms = 2usize;
        let arm_steps = 120usize;
        let lit_steps = (ctx.eased * arm_steps as f32) as usize;
        let rot = ctx.time * 0.3; // slow rotation

        for arm in 0..num_arms {
            let arm_offset = arm as f32 * PI; // 180° apart
            for s in 0..lit_steps.min(arm_steps) {
                let theta = s as f32 / arm_steps as f32 * 4.0 * PI;
                let r = a * (b * theta).exp();
                if r > max_r {
                    break;
                }
                let angle = theta + arm_offset + rot;
                let px = (cx + r * angle.cos()) as i32;
                // Squish vertically to look more natural in wide terminals
                let py = (cy + r * angle.sin() * 0.55) as i32;
                draw::dot_i(grid, px, py);
                // Occasional scatter dot around arm
                if hash((s as u32).wrapping_add(arm as u32 * 500)) % 5 == 0 {
                    let scatter_a = angle + hash_f((s * 3 + arm * 999) as u32) * 0.4 - 0.2;
                    let scatter_r = r * (0.8 + hash_f((s * 7 + arm * 777) as u32) * 0.4);
                    let spx = (cx + scatter_r * scatter_a.cos()) as i32;
                    let spy = (cy + scatter_r * scatter_a.sin() * 0.55) as i32;
                    draw::dot_i(grid, spx, spy);
                }
            }
        }

        // Bright galactic core
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
            }
        }

        // Tint: deep blue→purple across all cells
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
// 7 — Satellite dish receiving signal
// ---------------------------------------------------------------------------

/// Parabolic dish with concentric arcs pulsing outward via time; fill = eased.
struct SatelliteDish;
impl ProgressStyle for SatelliteDish {
    fn name(&self) -> &str {
        "satellite-dish"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Parabolic dish with signal arcs pulsing outward; strength driven by eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Dish: left-anchored parabola
        let dish_cx = (w / 4) as i32;
        let dish_cy = (h / 2) as i32;
        let dish_r = ((h / 2).saturating_sub(1)).max(2) as i32;

        // Draw parabolic bowl: x = cx - (dy^2 / dish_r)
        for dy in -dish_r..=dish_r {
            let dx = (dy * dy) / dish_r.max(1);
            let px = dish_cx - dx;
            let py = dish_cy + dy;
            draw::dot_i(grid, px, py);
        }
        // Stem
        draw::vline(
            grid,
            dish_cx as usize,
            (dish_cy + dish_r) as usize,
            (dish_cy + dish_r + 2).min(h.saturating_sub(1) as i32) as usize,
        );

        // Signal arcs emanating from focal point to the right
        let focal_x = dish_cx + dish_r / 2 + 1;
        let focal_y = dish_cy;
        let num_arcs = (ctx.eased * 5.0 + 1.0) as usize;
        let phase = ctx.time * 2.5;

        for arc_idx in 0..num_arcs.min(5) {
            // Each arc scrolls outward; wrap with fract
            let arc_phase = (phase + arc_idx as f32 * 0.7).fract();
            let arc_r = (arc_phase * (w as f32 * 0.55)) as i32;
            let arc_r = arc_r.max(1);

            // Draw a quarter-circle arc opening to the right
            let arc_steps = 20usize;
            for s in 0..arc_steps {
                let a = (s as f32 / arc_steps as f32 - 0.5) * PI; // -π/2 .. +π/2
                let px = focal_x + (arc_r as f32 * a.cos()) as i32;
                let py = focal_y + (arc_r as f32 * a.sin()) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Tint signal region with palette
        let (cells_w, cells_h) = grid.dimensions();
        let signal_cells = (ctx.eased * cells_w as f32) as usize;
        let dish_cell = cells_w / 4;
        for cx_c in dish_cell..signal_cells.min(cells_w) {
            let t = (cx_c - dish_cell) as f32 / (cells_w - dish_cell).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8 — Saturn rings filling
// ---------------------------------------------------------------------------

/// Planet disc surrounded by rings that fill in from eased; ring tilt animated.
struct SaturnRings;
impl ProgressStyle for SaturnRings {
    fn name(&self) -> &str {
        "saturn-rings"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Saturn disc with concentric rings filling from inner to outer as eased grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let planet_r = ((h / 2).saturating_sub(2)).max(1) as i32;

        // Planet body
        for dy in -planet_r..=planet_r {
            let dx_max = ((planet_r * planet_r - dy * dy) as f32).sqrt() as i32;
            for dx in -dx_max..=dx_max {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Rings: ellipses with increasing radii, tilt oscillates via time
        let num_rings = 5usize;
        let tilt = 0.25 + 0.08 * (ctx.time * 0.4).sin(); // vertical compression
        let max_ring_r = (w / 2).saturating_sub(1) as i32;
        let rings_lit = (ctx.eased * num_rings as f32).ceil() as usize;

        for ring in 0..rings_lit.min(num_rings) {
            let frac = (ring + 1) as f32 / num_rings as f32;
            let ring_rx = (planet_r + 2 + (frac * (max_ring_r - planet_r - 2) as f32) as i32)
                .max(planet_r + 1);
            let ring_ry = (ring_rx as f32 * tilt) as i32;
            let ring_ry = ring_ry.max(1);

            // Partial ring: last ring fills to eased sub-fraction
            let lit_frac = if ring + 1 == rings_lit {
                let inner_frac = ctx.eased * num_rings as f32 - ring as f32;
                inner_frac.clamp(0.0, 1.0)
            } else {
                1.0
            };
            let ring_steps = 80usize;
            let lit_steps = (lit_frac * ring_steps as f32) as usize;

            for s in 0..lit_steps.min(ring_steps) {
                let a = s as f32 / ring_steps as f32 * 2.0 * PI;
                let px = cx + (ring_rx as f32 * a.cos()) as i32;
                let py = cy + (ring_ry as f32 * a.sin()) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Tint: rings in palette color
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
// 9 — Black hole accretion disk
// ---------------------------------------------------------------------------

/// Event horizon with swirling accretion disk; disk rotates with time, fills with eased.
struct BlackHole;
impl ProgressStyle for BlackHole {
    fn name(&self) -> &str {
        "black-hole"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Black-hole event horizon with swirling accretion disk rotating via time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let horizon_r = ((h / 2).saturating_sub(2)).max(1) as i32;

        // Event horizon: solid black circle (we draw its outline only — interior stays empty)
        let eh_steps = 64usize;
        for s in 0..eh_steps {
            let a = s as f32 / eh_steps as f32 * 2.0 * PI;
            let px = cx + (horizon_r as f32 * a.cos()) as i32;
            let py = cy + (horizon_r as f32 * a.sin() * 0.55) as i32;
            draw::dot_i(grid, px, py);
        }

        // Accretion disk: multiple rings outside the horizon
        let disk_rings = 4usize;
        let disk_lit = (ctx.eased * disk_rings as f32).ceil() as usize;
        let rot = ctx.time * 1.2;

        for ring in 0..disk_lit.min(disk_rings) {
            let frac = (ring + 1) as f32 / disk_rings as f32;
            let ring_rx = horizon_r + 2 + (frac * (w as f32 * 0.3)) as i32;
            let ring_ry = (ring_rx as f32 * 0.3).max(1.0) as i32;

            let lit_frac = if ring + 1 == disk_lit {
                (ctx.eased * disk_rings as f32 - ring as f32).clamp(0.0, 1.0)
            } else {
                1.0
            };

            let disk_steps = 72usize;
            let lit_steps = (lit_frac * disk_steps as f32) as usize;
            for s in 0..lit_steps.min(disk_steps) {
                // Spiral: angle offset by ring index to create swirl effect
                let a = s as f32 / disk_steps as f32 * 2.0 * PI + rot + ring as f32 * 0.4;
                // Wavy radius: breathing effect
                let r_vary = 1.0 + 0.12 * (a * 3.0 + ctx.time * 2.0).sin();
                let px = cx + (ring_rx as f32 * r_vary * a.cos()) as i32;
                let py = cy + (ring_ry as f32 * r_vary * a.sin()) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Tint: hot orange/red near hole, fading outward
        let (cells_w, cells_h) = grid.dimensions();
        for cx_c in 0..cells_w {
            // Map to distance from center
            let dist = (cx_c as f32 - cells_w as f32 / 2.0).abs() / (cells_w as f32 / 2.0);
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
// 10 — Constellation
// ---------------------------------------------------------------------------

/// Fixed star positions; edges connect one by one as eased * N edges grow.
struct Constellation;
impl ProgressStyle for Constellation {
    fn name(&self) -> &str {
        "constellation"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Star constellation: edges connect one by one as eased fraction grows; stars twinkle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // 12 stable star positions (hash-seeded, unit fractions)
        const NUM_STARS: u32 = 12;
        let stars: Vec<(i32, i32)> = (0..NUM_STARS)
            .map(|i| {
                let sx = (hash_f(i) * (w as f32 - 2.0) + 1.0) as i32;
                let sy = (hash_f(i + 100) * (h as f32 - 2.0) + 1.0) as i32;
                (sx, sy)
            })
            .collect();

        // Adjacency: connect star i to star (i+1) % N and (i+3) % N for visual variety
        let edges: Vec<(usize, usize)> = (0..NUM_STARS as usize)
            .flat_map(|i| {
                vec![
                    (i, (i + 1) % NUM_STARS as usize),
                    (i, (i + 3) % NUM_STARS as usize),
                ]
            })
            .collect();

        let edges_lit = (ctx.eased * edges.len() as f32) as usize;

        // Draw edges that are revealed so far
        for (a, b) in edges.iter().take(edges_lit) {
            let (ax, ay) = stars[*a];
            let (bx, by) = stars[*b];
            // Bresenham-lite: step along the longer axis
            let dx = (bx - ax).abs();
            let dy = (by - ay).abs();
            let steps = dx.max(dy).max(1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = ax + ((bx - ax) as f32 * t) as i32;
                let py = ay + ((by - ay) as f32 * t) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Draw stars with twinkle: occasionally skip a dot based on time+index
        for (i, (sx, sy)) in stars.iter().enumerate() {
            let twinkle = (hash((i as u32).wrapping_add((ctx.time * 4.0) as u32)) % 6) != 0;
            if twinkle {
                draw::dot_i(grid, *sx, *sy);
                // Larger star cross
                draw::dot_i(grid, sx + 1, *sy);
                draw::dot_i(grid, sx - 1, *sy);
                draw::dot_i(grid, *sx, sy + 1);
                draw::dot_i(grid, *sx, sy - 1);
            }
        }

        // Tint revealed edges with gradient
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cx_c in 0..filled_cells.min(cells_w) {
            let t = cx_c as f32 / cells_w.saturating_sub(1).max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
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
    let styles = progress::styles::space::styles();
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
