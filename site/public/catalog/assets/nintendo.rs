//! `nintendo` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O nintendo.rs && ./nintendo [style-name]
//! ```

const DEFAULT_STYLE: &str = "mario-run";

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
    pub mod nintendo {
//! NES / Nintendo-classics themed progress bars.
//!
//! Each style is mechanically distinct — geometry, algorithm, motion, and
//! symbol choice all differ. Colour alone is never the only differentiator.
//!
//! Styles:
//! - `mario-run`      — Mario runs rightward; at 100% the flagpole drops.
//! - `zelda-hearts`   — Heart-container row fills one half-heart at a time.
//! - `metroid-etanks` — Energy-tank segments charge one at a time.
//! - `tetris-well`    — A well fills upward with a settling tetromino stack.
//! - `duck-hunt`      — Ducks arc across the sky; scored ducks show progress.
//! - `excitebike`     — A bike races right with a turbo-heat gauge on row 2.
//! - `punchout-stars` — A stamina bar that depletes then refills as star-power.
//! - `contra-spread`  — Spread-shot bullets fan out rightward.
//! - `megaman-weapon` — Segmented Mega Man weapon-energy bar charges top-down.
//! - `iceclimber-up`  — Platforms ascend; climber rises with `eased`.
//! - `donkey-barrel`  — Barrels roll down girders; Mario climbs a ladder.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — cartridge red into NES blue. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(232, 64, 52);
const TINT_END: Color = Color::rgb(72, 124, 255);

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

/// All styles in the `nintendo` theme.
///
/// Returns one boxed implementor per NES game mechanic.  Every style is
/// a stateless unit struct — no heap allocation beyond the `Box`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(MarioRun)),
        Box::new(Tinted(ZeldaHearts)),
        Box::new(Tinted(MetroidETanks)),
        Box::new(Tinted(TetrisWell)),
        Box::new(Tinted(DuckHunt)),
        Box::new(Excitebike),
        Box::new(PunchOutStars),
        Box::new(Tinted(ContraSpread)),
        Box::new(MegaManWeapon),
        Box::new(Tinted(IceClimberUp)),
        Box::new(Tinted(DonkeyBarrel)),
    ]
}

// ─── 1. Mario Run ────────────────────────────────────────────────────────────

/// Mario runs rightward with animated legs; at 100% a flagpole appears on the
/// right and Mario "slides" (collapses to a dot at the base).
struct MarioRun;
impl ProgressStyle for MarioRun {
    fn name(&self) -> &str {
        "mario-run"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Mario sprints right in braille dots; flagpole drops at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Ground line across the bottom.
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        let mario_x =
            ((ctx.eased * (dw.saturating_sub(4)) as f32) as usize).min(dw.saturating_sub(1));
        let ground_y = dh.saturating_sub(1);

        if ctx.progress >= 0.999 {
            // Flagpole on the far right.
            let pole_x = dw.saturating_sub(2);
            draw::vline(grid, pole_x, 0, ground_y);
            // Flag (small filled square near top).
            if dh >= 3 {
                draw::fill_rect(grid, pole_x + 1, 0, 1, (dh / 3).max(1));
            }
            // Mario at the base of the pole (just a dot cluster).
            draw::dot(grid, pole_x, ground_y);
            if pole_x > 0 {
                draw::dot(grid, pole_x.saturating_sub(1), ground_y);
            }
        } else {
            // Mario body: 2-wide, 3-tall (scaled by dh).
            let body_h = ((dh as f32 * 0.6) as usize).max(1);
            let body_y = ground_y.saturating_sub(body_h);
            draw::fill_rect(
                grid,
                mario_x,
                body_y,
                2.min(dw.saturating_sub(mario_x)),
                body_h,
            );

            // Animated legs — alternate two dot patterns driven by time.
            let step = ((ctx.time * 8.0) as usize) % 2;
            if dh >= 2 {
                let lx = mario_x + step;
                draw::dot(grid, lx.min(dw.saturating_sub(1)), ground_y);
                if mario_x + 1 < dw {
                    draw::dot(grid, (mario_x + 1).saturating_sub(step), ground_y);
                }
            }
        }

        // Coin trail to the left of Mario (shows how far he's come).
        let coins = (ctx.eased * (mario_x / 4).max(1) as f32) as usize;
        for i in 0..coins {
            let cx = i * 4;
            if cx + 1 < mario_x && cx < dw {
                draw::dot(grid, cx, ground_y.saturating_sub(dh / 3));
            }
        }

        Ok(())
    }
}

// ─── 2. Zelda Hearts ─────────────────────────────────────────────────────────

/// Zelda HUD heart containers: full hearts are dense-filled, empty hearts are
/// outlines, and partial is a half-full heart.  Progress maps to half-hearts.
struct ZeldaHearts;
impl ProgressStyle for ZeldaHearts {
    fn name(&self) -> &str {
        "zelda-hearts"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Zelda heart-container row — fills one half-heart at a time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Pack as many 3-cell-wide hearts as fit, with a 1-cell gap.
        let heart_cell_w = 3usize;
        let gap = 1usize;
        let slot_w = heart_cell_w + gap;
        let n_hearts = (cw / slot_w).max(1);
        let half_hearts_total = n_hearts * 2;
        let half_filled = (ctx.eased * half_hearts_total as f32).round() as usize;

        let row = ch / 2; // draw in vertical middle

        for i in 0..n_hearts {
            let x0 = i * slot_w;
            let full_halves = (half_filled).saturating_sub(i * 2);
            // full_halves for this heart: 0 = empty, 1 = half, 2 = full
            let state = full_halves.min(2);

            // Heart outline (3 dots wide, 2 dots tall in dot-space):
            //  cols x0*2, x0*2+1, x0*2+2, x0*2+3, x0*2+4, x0*2+5
            let dx = x0 * 2;
            let (dw, dh) = draw::dot_dims(grid);
            let dy = (row * 4).min(dh.saturating_sub(3));
            // Two bumps on top row.
            if dx + 1 < dw && dy < dh {
                draw::dot(grid, dx + 1, dy);
            }
            if dx + 4 < dw && dy < dh {
                draw::dot(grid, dx + 4, dy);
            }
            // Wider middle: outline edges only, so the fill state reads.
            if dy + 1 < dh {
                if dx < dw {
                    draw::dot(grid, dx, dy + 1);
                }
                if dx + 5 < dw {
                    draw::dot(grid, dx + 5, dy + 1);
                }
            }
            // Taper bottom: outline edges only.
            if dy + 2 < dh {
                if dx + 1 < dw {
                    draw::dot(grid, dx + 1, dy + 2);
                }
                if dx + 4 < dw {
                    draw::dot(grid, dx + 4, dy + 2);
                }
            }
            if dx + 2 < dw && dy + 3 < dh {
                draw::dot(grid, dx + 2, dy + 3);
            }
            if dx + 3 < dw && dy + 3 < dh {
                draw::dot(grid, dx + 3, dy + 3);
            }

            // Fill interior based on state.
            if state == 2 {
                // Full: fill centre of the heart.
                for xx in (dx + 1)..(dx + 5).min(dw) {
                    if dy + 1 < dh {
                        draw::dot(grid, xx, dy + 1);
                    }
                }
                for xx in (dx + 1)..(dx + 5).min(dw) {
                    if dy + 2 < dh {
                        draw::dot(grid, xx, dy + 2);
                    }
                }
            } else if state == 1 {
                // Half: fill left side only.
                for xx in (dx + 1)..(dx + 3).min(dw) {
                    if dy + 1 < dh {
                        draw::dot(grid, xx, dy + 1);
                    }
                    if dy + 2 < dh {
                        draw::dot(grid, xx, dy + 2);
                    }
                }
            }
            // state == 0: outline only (already drawn above)
        }

        Ok(())
    }
}

// ─── 3. Metroid Energy Tanks ──────────────────────────────────────────────────

/// Metroid HUD: a row of rectangular energy-tank segments that charge one at a
/// time.  Each tank is a bordered rectangle; the active one shows a partial fill.
struct MetroidETanks;
impl ProgressStyle for MetroidETanks {
    fn name(&self) -> &str {
        "metroid-etanks"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Metroid energy-tank segments — each tank charges then the next activates"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let tank_dot_w = ((dw / 6).max(3)).min(dw);
        let gap_dots = 1usize;
        let n_tanks = (dw / (tank_dot_w + gap_dots)).max(1);

        let filled_f = ctx.eased * n_tanks as f32;
        let full_tanks = filled_f as usize;
        let partial_f = filled_f.fract();

        for i in 0..n_tanks {
            let x0 = i * (tank_dot_w + gap_dots);
            if x0 >= dw {
                break;
            }
            let actual_w = tank_dot_w.min(dw.saturating_sub(x0));
            // Border.
            draw::rect_outline(grid, x0, 0, actual_w, dh);
            // Fill.
            if i < full_tanks {
                // Fully charged: fill interior.
                let iw = actual_w.saturating_sub(2);
                let ih = dh.saturating_sub(2);
                if iw > 0 && ih > 0 {
                    draw::fill_rect(grid, x0 + 1, 1, iw, ih);
                }
            } else if i == full_tanks && partial_f > 0.001 {
                // Partially charged: fill bottom portion of interior.
                let iw = actual_w.saturating_sub(2);
                let ih = dh.saturating_sub(2);
                if iw > 0 && ih > 0 {
                    let charge_h = ((partial_f * ih as f32).round() as usize).max(1).min(ih);
                    let y0 = ih + 1 - charge_h; // fills bottom-up
                    draw::fill_rect(grid, x0 + 1, y0, iw, charge_h);
                }
            }
        }

        Ok(())
    }
}

// ─── 4. Tetris Well ──────────────────────────────────────────────────────────

/// A vertical well (left and right walls, open top) fills from the bottom as
/// tetrominoes settle.  The stack height is driven by `eased`; the topmost
/// layer shows the current "falling" piece as a blinking row.
struct TetrisWell;
impl ProgressStyle for TetrisWell {
    fn name(&self) -> &str {
        "tetris-well"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Tetris well fills upward with a settling block stack"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Well walls.
        draw::vline(grid, 0, 0, dh.saturating_sub(1));
        draw::vline(grid, dw.saturating_sub(1), 0, dh.saturating_sub(1));
        // Floor.
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        let inner_w = dw.saturating_sub(2);
        let inner_h = dh.saturating_sub(1);
        if inner_w == 0 || inner_h == 0 {
            return Ok(());
        }

        // Stack: fills from the bottom.
        let stack_dots = (ctx.eased * inner_h as f32) as usize;
        let stack_dots = stack_dots.min(inner_h);
        let stack_y0 = inner_h.saturating_sub(stack_dots);

        // Draw stacked rows with slight texture (every 4 dots a gap line).
        for y in stack_y0..inner_h {
            let row_mod = (inner_h - 1 - y) % 4;
            if row_mod == 3 {
                // "Mortar" gap — sparse dots.
                for x in (1..inner_w + 1).step_by(3) {
                    draw::dot(grid, x, y);
                }
            } else {
                draw::hline(grid, 1, inner_w, y);
            }
        }

        // Falling piece: a blinking 4-wide row just above the stack.
        if stack_dots < inner_h {
            let piece_y = stack_y0.saturating_sub(1);
            let blink = ((ctx.time * 4.0) as usize) % 2 == 0;
            if blink && piece_y < inner_h {
                // Randomise piece shape by time bucket (cycles through shapes).
                let shape = ((ctx.time * 0.5) as usize) % 5;
                let pw = (inner_w.min(8)).max(1);
                let px0 = 1 + (inner_w.saturating_sub(pw)) / 2;
                match shape {
                    0 => draw::hline(grid, px0, (px0 + 3).min(dw.saturating_sub(1)), piece_y), // I
                    1 => {
                        // L
                        draw::hline(grid, px0, (px0 + 2).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::dot(grid, px0 + 2, piece_y.saturating_sub(1));
                        }
                    }
                    2 => {
                        // S
                        draw::hline(grid, px0 + 1, (px0 + 3).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::hline(
                                grid,
                                px0,
                                (px0 + 2).min(dw.saturating_sub(1)),
                                piece_y.saturating_sub(1),
                            );
                        }
                    }
                    3 => {
                        // T
                        draw::hline(grid, px0, (px0 + 2).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::dot(grid, px0 + 1, piece_y.saturating_sub(1));
                        }
                    }
                    _ => {
                        // O (square)
                        draw::hline(grid, px0, (px0 + 1).min(dw.saturating_sub(1)), piece_y);
                        if piece_y >= 1 {
                            draw::hline(
                                grid,
                                px0,
                                (px0 + 1).min(dw.saturating_sub(1)),
                                piece_y.saturating_sub(1),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// ─── 5. Duck Hunt ─────────────────────────────────────────────────────────────

/// Ducks arc across the sky using sinusoidal paths; the number of "shot" ducks
/// equals `round(progress * total_ducks)`.  Downed ducks fall to the ground.
struct DuckHunt;
impl ProgressStyle for DuckHunt {
    fn name(&self) -> &str {
        "duck-hunt"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Ducks arc across the sky; progress = ducks shot, complete ducks fall"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Ground.
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        let n_ducks = 5usize;
        let shot_count = (ctx.eased * n_ducks as f32).round() as usize;

        for i in 0..n_ducks {
            let phase = i as f32 * 0.7 + ctx.time * 0.8;
            let t_norm = (phase * 0.4).fract(); // 0..1 left-to-right

            if i < shot_count {
                // Shot duck: falls toward the bottom.
                let fall = ((ctx.time - (i as f32 * 0.3)).max(0.0) * 12.0) as i32;
                let dx = (t_norm * dw as f32) as i32;
                let ground = dh as i32 - 2;
                let dy = (fall).min(ground);
                // Tumbling duck (just dots in a small cluster).
                draw::dot_i(grid, dx, dy);
                draw::dot_i(grid, dx + 1, dy);
                draw::dot_i(grid, dx, dy + 1);
            } else {
                // Live duck: sinusoidal arc across sky.
                let dx = (t_norm * dw as f32) as i32;
                let dy = ((PI * t_norm).sin() * (dh.saturating_sub(3)) as f32 * 0.6 + 1.0) as i32;
                // 2×2 body.
                draw::dot_i(grid, dx, dy);
                draw::dot_i(grid, dx + 1, dy);
                draw::dot_i(grid, dx, dy - 1);
                // Wing flap (alternate row).
                let flap = (((ctx.time * 6.0 + i as f32) as i32) % 2) as i32;
                draw::dot_i(grid, dx + 2, dy - flap);
            }
        }

        Ok(())
    }
}

// ─── 6. Excitebike ───────────────────────────────────────────────────────────

/// The bike position on row 1 tracks `eased` (progress = distance covered).
/// Row 2 is a TURBO HEAT gauge: it oscillates — if it overheats the bike icon
/// stutters (shows a "flame" glyph and the bar pulses red via tinting).
struct Excitebike;
impl ProgressStyle for Excitebike {
    fn name(&self) -> &str {
        "excitebike"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Bike races right; row 2 is a turbo-heat gauge that must not overheat"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // ── Track (ground line) ──
        draw::hline(grid, 0, dw.saturating_sub(1), dh.saturating_sub(1));

        // ── Bike position ──
        let bike_x =
            ((ctx.eased * (dw.saturating_sub(4)) as f32) as usize).min(dw.saturating_sub(1));
        let wheel_y = dh.saturating_sub(2);

        // Front and rear wheels (single dots).
        draw::dot(grid, bike_x, wheel_y);
        if bike_x >= 3 {
            draw::dot(grid, bike_x.saturating_sub(3), wheel_y);
        }

        // Frame (diagonal line from rear-wheel to handlebars).
        for i in 0..3usize {
            draw::dot_i(grid, bike_x as i32 - i as i32, wheel_y as i32 - i as i32);
        }
        // Rider head.
        if wheel_y >= 3 {
            draw::dot(grid, bike_x, wheel_y.saturating_sub(3));
        }

        // ── Turbo heat gauge on row 2 (cell row 1 if height > 1) ──
        if ch >= 2 && cw > 0 {
            // Heat oscillates with time; at high progress it climbs.
            let heat = (ctx.time * 1.2).sin() * 0.3 + ctx.eased * 0.7;
            let heat = heat.clamp(0.0, 1.0);
            let overheat = heat > 0.85;

            // Use hbar in the second-to-last cell row for the gauge.
            let gauge_row = ch.saturating_sub(2).min(ch.saturating_sub(1));
            // Draw gauge fill via glyph calls on the heat-gauge row.
            let heat_cells = (heat * cw as f32) as usize;
            for cx in 0..heat_cells.min(cw) {
                let shade = if overheat { 4usize } else { 2 };
                draw::shade(grid, cx, gauge_row, shade);
            }

            // Tint gauge row: red if overheating, yellow otherwise.
            if overheat {
                let hot_color = crate::Color::rgb(255, 40, 0);
                draw::tint_row(grid, gauge_row, 0, cw.saturating_sub(1), hot_color);
            } else {
                let warm_color = crate::Color::rgb(255, 200, 0);
                draw::tint_row(
                    grid,
                    gauge_row,
                    0,
                    heat_cells.min(cw.saturating_sub(1)),
                    warm_color,
                );
            }
        }

        Ok(())
    }
}

// ─── 7. Punch-Out Stars ──────────────────────────────────────────────────────

/// A segmented stamina bar that depletes left-to-right as `progress` rises,
/// then at 75%+ refills from right-to-left as a "star-power" burst (bright
/// glyphs).  Uses `vblock` for segment columns.
struct PunchOutStars;
impl ProgressStyle for PunchOutStars {
    fn name(&self) -> &str {
        "punchout-stars"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Punch-Out stamina bar depletes then surges back as star-power at 75%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Stamina: depletes until 75%, then star-power refills.
        let star_power = ctx.progress > 0.75;
        let bar_frac = if star_power {
            // Refill from 0 → 1 over the range 0.75..1.0.
            (ctx.progress - 0.75) / 0.25
        } else {
            // Deplete from 1 → 0 over the range 0..0.75.
            1.0 - ctx.progress / 0.75
        };
        let bar_frac = bar_frac.clamp(0.0, 1.0);

        let filled_cells = (bar_frac * cw as f32).round() as usize;

        for cx in 0..cw {
            for cy in 0..ch {
                let level = if cx < filled_cells { 8usize } else { 0 };
                if level > 0 {
                    if star_power {
                        // Star-power: alternate between full and heavy-shade.
                        let pulse = ((ctx.time * 8.0 + cx as f32 * 0.5) as usize) % 2;
                        draw::shade(grid, cx, cy, if pulse == 0 { 4 } else { 3 });
                    } else {
                        draw::vblock(grid, cx, cy, level);
                    }
                } else {
                    draw::shade(grid, cx, cy, 1);
                }
            }
        }

        // Tint: yellow stamina, white star-power.
        let color = if star_power {
            crate::Color::rgb(255, 255, 180)
        } else {
            crate::Color::rgb(255, 220, 0)
        };
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, filled_cells.min(cw).saturating_sub(1), color);
        }

        Ok(())
    }
}

// ─── 8. Contra Spread ────────────────────────────────────────────────────────

/// Contra spread-shot: multiple bullets fan out from the left edge in a
/// symmetric arc.  Progress controls how far the fan has travelled rightward.
/// Each bullet trail is a dotted line along its angle.
struct ContraSpread;
impl ProgressStyle for ContraSpread {
    fn name(&self) -> &str {
        "contra-spread"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Contra spread-shot bullets fan out rightward; progress = bullet travel"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx_f = 0.0f32; // fire from left edge
        let cy_f = (dh as f32 - 1.0) / 2.0; // vertical centre
        let n_rays = 5usize;
        let reach = (ctx.eased * dw as f32) as usize;

        // Also draw a muzzle flash at the origin.
        let flash = ((ctx.time * 12.0) as usize) % 2 == 0;
        if flash {
            draw::dot(grid, 0, cy_f as usize);
        }

        for i in 0..n_rays {
            // Angles: 0° (straight), ±22.5°, ±45°.
            let angle_idx = i as i32 - (n_rays as i32 / 2);
            let angle = angle_idx as f32 * PI / 8.0; // 22.5° steps
            let dx_f = angle.cos();
            let dy_f = angle.sin();

            // Draw the bullet trail up to reach dots.
            let steps = reach.min(dw);
            for s in (0..steps).step_by(2) {
                let bx = (cx_f + s as f32 * dx_f) as i32;
                let by = (cy_f + s as f32 * dy_f) as i32;
                draw::dot_i(grid, bx, by);
            }
            // Bullet head at the tip.
            if reach > 0 {
                let bx = (cx_f + reach as f32 * dx_f) as i32;
                let by = (cy_f + reach as f32 * dy_f) as i32;
                draw::dot_i(grid, bx, by);
                draw::dot_i(grid, bx + 1, by);
            }
        }

        Ok(())
    }
}

// ─── 9. Mega Man Weapon ───────────────────────────────────────────────────────

/// Mega Man weapon-energy: a tall narrow bar divided into discrete segments
/// (like the in-game weapon panel).  Segments light up from the bottom; the
/// active segment shows sub-segment precision via `vblock`.
struct MegaManWeapon;
impl ProgressStyle for MegaManWeapon {
    fn name(&self) -> &str {
        "megaman-weapon"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Mega Man weapon-energy bar — segmented column charges from bottom up"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Number of segments = cell height; each segment is one cell row.
        let n_segs = ch;
        let seg_f = ctx.eased * n_segs as f32;
        let full_segs = seg_f as usize;
        let partial = seg_f.fract();

        // The weapon bar occupies columns 0..=1 (or full width if narrow).
        let bar_w = cw.min(2);

        for seg in 0..n_segs {
            // Segments fill bottom-up: seg 0 = bottom (ch-1 in cell coords).
            let cell_y = n_segs.saturating_sub(1).saturating_sub(seg);

            if seg < full_segs {
                // Full segment: solid fill.
                for cx in 0..bar_w {
                    draw::glyph(grid, cx, cell_y, '█');
                }
            } else if seg == full_segs && partial > 0.001 {
                // Partial segment using vblock precision.
                let level = (partial * 8.0).round() as usize;
                for cx in 0..bar_w {
                    draw::vblock(grid, cx, cell_y, level);
                }
            } else {
                // Empty segment — light border using shade 1.
                for cx in 0..bar_w {
                    draw::shade(grid, cx, cell_y, 1);
                }
            }
        }

        // Remainder columns: empty track.
        for cx in bar_w..cw {
            for cy in 0..ch {
                draw::shade(grid, cx, cy, 0);
            }
        }

        // Tint the filled column.
        let bar_color = ctx.palette.sample(ctx.eased);
        for seg in 0..full_segs.min(ch) {
            let cell_y = ch.saturating_sub(1).saturating_sub(seg);
            draw::tint_row(grid, cell_y, 0, bar_w.saturating_sub(1), bar_color);
        }

        Ok(())
    }
}

// ─── 10. Ice Climber Up ───────────────────────────────────────────────────────

/// Platforms scroll upward: `eased` determines how high the climber has risen.
/// The bottom of the grid shows platforms drawn as horizontal dot-lines at
/// staggered heights; the climber is a small dot cluster that moves up with
/// `eased`.
struct IceClimberUp;
impl ProgressStyle for IceClimberUp {
    fn name(&self) -> &str {
        "iceclimber-up"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "Ice Climber ascends upward platform by platform driven by eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Platforms: spaced every dh/4 dots, alternating left-right alignment,
        // scrolling upward with time so the world moves past the climber.
        let scroll_offset = (ctx.time * 2.0) as usize % (dh.max(1));
        let plat_spacing = (dh / 4).max(2);
        let plat_count = (dh / plat_spacing) + 2;

        for p in 0..plat_count {
            let base_y = p * plat_spacing;
            let y = (base_y + scroll_offset) % (dh + plat_spacing);
            if y >= dh {
                continue;
            }

            let is_left = p % 2 == 0;
            let pw = dw * 3 / 4;
            let px0 = if is_left { 0 } else { dw.saturating_sub(pw) };
            draw::hline(
                grid,
                px0,
                (px0 + pw).saturating_sub(1).min(dw.saturating_sub(1)),
                y,
            );
            // Small snow pillar at platform edges.
            if y + 1 < dh {
                draw::dot(grid, px0, y + 1);
                let right = (px0 + pw).saturating_sub(1).min(dw.saturating_sub(1));
                draw::dot(grid, right, y + 1);
            }
        }

        // Climber: rises from bottom (eased=0) to top (eased=1).
        let climber_y = (dh.saturating_sub(3) as f32 * (1.0 - ctx.eased)) as usize;
        let climber_x = dw / 2;
        // Head.
        draw::dot(grid, climber_x, climber_y);
        draw::dot(grid, climber_x + 1, climber_y);
        // Body.
        if climber_y + 1 < dh {
            draw::dot(grid, climber_x, climber_y + 1);
            draw::dot(grid, climber_x + 1, climber_y + 1);
        }
        // Legs alternate with time.
        if climber_y + 2 < dh {
            let leg_step = ((ctx.time * 6.0) as usize) % 2;
            draw::dot(grid, climber_x + leg_step, climber_y + 2);
        }

        Ok(())
    }
}

// ─── 11. Donkey Kong Barrel ───────────────────────────────────────────────────

/// Donkey Kong: girders drawn as horizontal dot-lines at staggered rows; barrels
/// are circles that roll down the girders from right to left driven by time.
/// Mario is a small dot cluster at the bottom-left that climbs a ladder (moves
/// up) as `eased` increases.
struct DonkeyBarrel;
impl ProgressStyle for DonkeyBarrel {
    fn name(&self) -> &str {
        "donkey-barrel"
    }
    fn theme(&self) -> &str {
        "nintendo"
    }
    fn describe(&self) -> &str {
        "DK barrels roll down girders; Mario climbs a ladder as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // ── Girders: horizontal bands ──
        let n_girders = ((dh / 3).max(1)).min(4);
        let girder_gap = dh / (n_girders + 1).max(1);

        for g in 0..n_girders {
            let gy = dh.saturating_sub(1).saturating_sub((g + 1) * girder_gap);
            draw::hline(grid, 0, dw.saturating_sub(1), gy);
        }

        // ── Ladder: narrow vertical line at the far left ──
        draw::vline(grid, 1, 0, dh.saturating_sub(1));
        // Ladder rungs.
        let rung_spacing = 3usize;
        let mut ry = 0usize;
        while ry < dh {
            draw::dot(grid, 0, ry);
            draw::dot(grid, 2, ry);
            ry += rung_spacing;
        }

        // ── Barrels: roll along each girder, one per girder ──
        for g in 0..n_girders {
            let gy = dh.saturating_sub(1).saturating_sub((g + 1) * girder_gap);
            // Each barrel on a different girder has offset phase.
            let phase = g as f32 * 0.4 + ctx.time * 0.7;
            let barrel_t = 1.0 - (phase * 0.3).fract(); // travels right→left
            let bx = (barrel_t * (dw.saturating_sub(3)) as f32) as usize;

            // Barrel is a small circle: 3 dots wide.
            if bx < dw && gy >= 1 {
                draw::dot(grid, bx, gy.saturating_sub(1)); // top
                draw::dot(grid, bx.saturating_sub(1), gy); // err: was gy but actually the girder row
                                                           // Place barrel one row above girder so it sits on top.
                let barrel_y = gy.saturating_sub(1);
                draw::dot(grid, bx, barrel_y);
                if bx + 1 < dw {
                    draw::dot(grid, bx + 1, barrel_y);
                }
                // Rolling indicator: alternating side dot.
                let roll = ((ctx.time * 5.0 + g as f32) as usize) % 2;
                let side_dot = if roll == 0 {
                    bx.saturating_sub(1)
                } else {
                    (bx + 2).min(dw.saturating_sub(1))
                };
                if barrel_y >= 1 {
                    draw::dot(grid, side_dot, barrel_y);
                }
            }
        }

        // ── Mario: climbs the ladder from bottom to top ──
        let mario_y = (dh.saturating_sub(3) as f32 * (1.0 - ctx.eased)) as usize;
        // Mario is at x=1 (on the ladder).
        draw::dot(grid, 1, mario_y);
        draw::dot(grid, 2, mario_y);
        if mario_y + 1 < dh {
            draw::dot(grid, 1, mario_y + 1);
            draw::dot(grid, 2, mario_y + 1);
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
    let styles = progress::styles::nintendo::styles();
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
