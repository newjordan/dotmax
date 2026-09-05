//! `sinewave` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O sinewave.rs && ./sinewave [style-name]
//! ```

const DEFAULT_STYLE: &str = "sw-traveling";

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
    pub mod sinewave {
//! Sine-wave progress bars for dotmax — a full family spanning smooth↔blocky.
//!
//! Every style in this theme is driven by sine-wave mathematics, but each
//! exploits a *structurally distinct* form of sinusoidal behavior:
//!
//! | Name | Waveform / Structure |
//! |------|----------------------|
//! | `sw-traveling` | y = A·sin(kx − ωt) scrolling; area below filled |
//! | `sw-sine-scroll` | demoscene bobbing markers on a rippling baseline |
//! | `sw-am` | amplitude-modulated: envelope·carrier, A(x)·sin(ωx) |
//! | `sw-chirp` | frequency chirp/sweep: freq rises across x, eased sets max |
//! | `sw-wave-packet` | Gaussian-windowed sinusoid traveling with time |
//! | `sw-harmonics` | N-harmonic superposition; N = 1 + eased·8 |
//! | `sw-barber-pole` | phase-gradient columns → diagonal stripes scrolling in time |
//! | `sw-area-fill` | region under |sin| fills up to eased (smooth braille) |
//! | `sw-blocky-eq` | same sine rendered as vblock columns (blocky counterpart) |
//! | `sw-rectified` | |sin| hard-clipped to threshold — rectified ripple |
//! | `sw-damped` | e^(−γx)·sin(ωx) ring-down from left edge |
//! | `sw-standing` | 2A·sin(kx)·cos(ωt) with nodes fixed in space |
//! | `sw-density` | dot density per column ∝ |sin| — dithered shading texture |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `sinewave` theme.
///
/// Returns 13 structurally distinct sine-wave bars. Each style exploits a
/// different mathematical form: traveling waves, AM envelopes, chirps,
/// Gaussian packets, harmonic superposition, phase gradients, area fills,
/// blocky quantization, rectification, ring-down damping, standing waves,
/// and density textures. Color does not substitute for structural variety.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Traveling),
        Box::new(SineScroll),
        Box::new(AmplitudeMod),
        Box::new(FreqChirp),
        Box::new(WavePacket),
        Box::new(Harmonics),
        Box::new(BarberPole),
        Box::new(AreaFill),
        Box::new(BlockyEq),
        Box::new(Rectified),
        Box::new(Damped),
        Box::new(StandingEnvelope),
        Box::new(Density),
    ]
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Draw a connected curve: for each x column compute y, connect vertically to
/// prior column to prevent single-dot gaps.
#[inline]
fn draw_curve<F>(grid: &mut BrailleGrid, w: usize, h: usize, f: F)
where
    F: Fn(usize) -> i32,
{
    let mut prev: Option<i32> = None;
    for xi in 0..w {
        let dy = f(xi).clamp(0, h as i32 - 1);
        draw::dot_i(grid, xi as i32, dy);
        if let Some(py) = prev {
            let lo = py.min(dy);
            let hi = py.max(dy);
            for yy in lo..=hi {
                draw::dot_i(grid, xi as i32, yy);
            }
        }
        prev = Some(dy);
    }
}

/// Tint the filled-cells region with a gradient.
#[inline]
fn tint_filled(grid: &mut BrailleGrid, ctx: &BarContext) {
    let (cw, ch) = grid.dimensions();
    let filled = (ctx.eased * cw as f32).round() as usize;
    for cx in 0..filled.min(cw) {
        let t = if filled <= 1 {
            0.5
        } else {
            cx as f32 / (filled - 1) as f32
        };
        let col = ctx.palette.sample(t);
        for cy in 0..ch {
            draw::tint_row(grid, cy, cx, cx, col);
        }
    }
}

// ---------------------------------------------------------------------------
// 1. Traveling wave: y = A·sin(kx − ωt)
//    The wave scrolls rightward as time increases.
//    Progress controls how much of the bar is "filled" below the wave.
// ---------------------------------------------------------------------------
struct Traveling;
impl ProgressStyle for Traveling {
    fn name(&self) -> &str {
        "sw-traveling"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Traveling wave y=A·sin(kx−ωt): scrolls rightward; progress fills the swept area below"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let amp = h as f32 * 0.38;
        let k = 2.0 * PI * 3.0 / w as f32; // 3 full cycles across the bar
        let omega = 2.0 * PI * 1.2; // rad/s
        let mid = (h / 2) as i32;
        let fill_x = (ctx.eased * w as f32).round() as usize; // rightward fill boundary

        // Filled region: solid dots from baseline to the wave surface
        for xi in 0..fill_x.min(w) {
            let theta = k * xi as f32 - omega * ctx.time;
            let wave_y = (mid - (amp * theta.sin()) as i32).clamp(0, h as i32 - 1);
            // Fill from wave surface down to bottom
            let top = wave_y.min(mid).max(0) as usize;
            let bot = wave_y.max(mid).min(h as i32 - 1) as usize;
            for y in top..=bot {
                draw::dot(grid, xi, y);
            }
        }

        // Unfilled region: just the wave outline
        draw_curve(grid, w, h, |xi| {
            let theta = k * xi as f32 - omega * ctx.time;
            (mid - (amp * theta.sin()) as i32).clamp(0, h as i32 - 1)
        });

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Sine-scroll (demoscene): a row of markers bob on a sine baseline.
//    The baseline itself ripples — sine of a sine. Markers are vertical
//    strokes at regular x positions, their y driven by sin(x + t).
// ---------------------------------------------------------------------------
struct SineScroll;
impl ProgressStyle for SineScroll {
    fn name(&self) -> &str {
        "sw-sine-scroll"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Demoscene sine-scroller: markers bob on a rippling baseline — classic 8-bit demo effect"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let amp = (h as f32 * 0.30).max(1.0);
        let mid = h as f32 / 2.0;
        let speed = 3.0_f32;
        let k1 = 2.0 * PI * 2.5 / w as f32; // baseline ripple freq
        let k2 = 2.0 * PI * 5.0 / w as f32; // marker bobbing freq (faster)

        // Rippling baseline
        draw_curve(grid, w, h, |xi| {
            let base_y = mid + amp * 0.4 * (k1 * xi as f32 - speed * ctx.time).sin();
            base_y as i32
        });

        // Marker positions: evenly spaced, limited by progress
        let n_markers = ((w / 4).max(1)).min(w);
        let markers_shown = (ctx.eased * n_markers as f32).ceil() as usize;
        for m in 0..markers_shown.min(n_markers) {
            let xi = (m * w / n_markers.max(1)).min(w - 1);
            let bob_y = mid + amp * (k2 * xi as f32 - speed * 1.3 * ctx.time).sin();
            let bob_y = bob_y.clamp(0.0, (h - 1) as f32) as usize;
            // Vertical stroke marker (3 dots tall)
            let top = bob_y.saturating_sub(1);
            let bot = (bob_y + 1).min(h - 1);
            draw::vline(grid, xi, top, bot);
            // Horizontal crossbar
            if xi > 0 {
                draw::dot(grid, xi - 1, bob_y);
            }
            if xi + 1 < w {
                draw::dot(grid, xi + 1, bob_y);
            }
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Amplitude modulation: A(x) · sin(ωx + φt)
//    Envelope A(x) is a sine arch over [0, eased·L] — trapezoidal AM shape.
//    Carrier frequency is constant; the envelope reveals with progress.
// ---------------------------------------------------------------------------
struct AmplitudeMod;
impl ProgressStyle for AmplitudeMod {
    fn name(&self) -> &str {
        "sw-am"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Amplitude modulation: arch-shaped envelope × carrier sine; envelope width grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;
        let max_amp = (h as f32 * 0.44).max(1.0);
        let omega = 2.0 * PI * 6.0 / w as f32; // carrier: 6 cycles
        let phi = ctx.time * 2.0 * PI * 0.8; // slow phase drift
                                             // Envelope boundary: progress sets how far the AM arch extends
        let env_end = ctx.eased * w as f32;

        draw_curve(grid, w, h, |xi| {
            let xf = xi as f32;
            // Envelope: sin arch over [0, env_end], 0 outside
            let env = if xf <= env_end && env_end > 0.0 {
                (PI * xf / env_end).sin().max(0.0)
            } else {
                0.0
            };
            let carrier = (omega * xf + phi).sin();
            let val = env * carrier;
            (mid - (max_amp * val) as i32).clamp(0, h as i32 - 1)
        });

        // Draw envelope outline (upper and lower silhouette)
        for xi in 0..w {
            let xf = xi as f32;
            let env = if xf <= env_end && env_end > 0.0 {
                (PI * xf / env_end).sin().max(0.0)
            } else {
                0.0
            };
            let e = (max_amp * env) as i32;
            draw::dot_i(grid, xi as i32, (mid - e).clamp(0, h as i32 - 1));
            draw::dot_i(grid, xi as i32, (mid + e).clamp(0, h as i32 - 1));
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Frequency chirp: sin(k(x)·x)  where k(x) = k_min + (k_max−k_min)·x/L
//    Frequency increases linearly from left to right. eased sets k_max.
//    Time shifts the instantaneous phase: chirp "sweeps" as a whole.
// ---------------------------------------------------------------------------
struct FreqChirp;
impl ProgressStyle for FreqChirp {
    fn name(&self) -> &str {
        "sw-chirp"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Frequency chirp: low→high sweep, compression visible from left; progress raises max frequency"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);
        let k_min = 2.0 * PI * 1.0 / w as f32; // 1 cycle at left
        let k_max = 2.0 * PI * (1.0 + ctx.eased * 9.0) / w as f32; // up to 10 cycles at right
        let phase = ctx.time * 2.0 * PI * 0.5; // global phase offset

        draw_curve(grid, w, h, |xi| {
            let xf = xi as f32;
            // Instantaneous phase: integral of k(x') dx' from 0 to x = k_min·x + (k_max−k_min)·x²/(2L)
            let inst_phase =
                k_min * xf + (k_max - k_min) * xf * xf / (2.0 * w.max(1) as f32) + phase;
            let val = inst_phase.sin();
            (mid - (amp * val) as i32).clamp(0, h as i32 - 1)
        });

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Wave packet: Gaussian-windowed sinusoid.
//    A(x) = exp(−((x − x₀)/σ)²) · sin(k·(x − x₀) + φ)
//    x₀ = eased × L (packet centre), σ = L/5, φ = ωt (carrier oscillates).
//    Distinct from waves::WavePacket: here the Gaussian envelope is drawn as
//    solid fill under the packet, not just a traced curve.
// ---------------------------------------------------------------------------
struct WavePacket;
impl ProgressStyle for WavePacket {
    fn name(&self) -> &str {
        "sw-wave-packet"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Gaussian wave packet: solid filled envelope travels with progress; carrier fringes oscillate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);
        let x0 = ctx.eased * wf;
        let sigma = (wf / 5.0).max(1.0);
        let k = 10.0 * PI / wf;
        let phi = ctx.time * 3.0 * PI;

        // Solid fill: between midline and wave surface, weighted by envelope
        for xi in 0..w {
            let xf = xi as f32;
            let dx = xf - x0;
            let env = (-(dx * dx) / (2.0 * sigma * sigma)).exp();
            if env < 0.01 {
                continue;
            }
            let wave_y = (mid - (amp * env * (k * dx + phi).sin()) as i32).clamp(0, h as i32 - 1);
            let top = wave_y.min(mid).max(0) as usize;
            let bot = wave_y.max(mid).min(h as i32 - 1) as usize;
            for y in top..=bot {
                draw::dot(grid, xi, y);
            }
        }

        // Gaussian envelope outline
        for xi in 0..w {
            let xf = xi as f32;
            let dx = xf - x0;
            let env = (-(dx * dx) / (2.0 * sigma * sigma)).exp();
            let ey = (amp * env) as i32;
            draw::dot_i(grid, xi as i32, (mid - ey).clamp(0, h as i32 - 1));
            draw::dot_i(grid, xi as i32, (mid + ey).clamp(0, h as i32 - 1));
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Harmonic superposition: y = Σ_{n=1}^{N} sin(n·θ) / n
//    N = 1 + floor(eased·8), so progress "adds" harmonics one by one.
//    Time provides a common phase offset so the whole composite wave scrolls.
//    At N=1 it's a clean sine; at N=9 it approximates a sawtooth-ish shape.
// ---------------------------------------------------------------------------
struct Harmonics;
impl ProgressStyle for Harmonics {
    fn name(&self) -> &str {
        "sw-harmonics"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Harmonic superposition: each harmonic unlocks with progress, morphing sine toward a complex wave"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);
        let n_harm = (1 + (ctx.eased * 8.0).floor() as usize).min(9);
        let phase_off = ctx.time * 2.0 * PI * 0.4; // scroll speed

        // Normalization: max theoretical amplitude = Σ 1/n
        let norm: f32 = (1..=n_harm).map(|n| 1.0 / n as f32).sum::<f32>().max(1.0);

        draw_curve(grid, w, h, |xi| {
            let theta = (xi as f32 / w.max(1) as f32) * 2.0 * PI * 3.0 + phase_off;
            let val: f32 = (1..=n_harm)
                .map(|n| (n as f32 * theta).sin() / n as f32)
                .sum::<f32>()
                / norm;
            (mid - (amp * val) as i32).clamp(0, h as i32 - 1)
        });

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Phase-gradient barber-pole: each column x has phase φ(x) = 2π·x/L·M
//    producing M diagonal stripe repeats. Time offsets all phases → stripes
//    scroll diagonally. Cells are lit if sin(φ(x) − ωt + 2π·y/h·M) > thresh.
// ---------------------------------------------------------------------------
struct BarberPole;
impl ProgressStyle for BarberPole {
    fn name(&self) -> &str {
        "sw-barber-pole"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Phase-gradient barber-pole: diagonal stripes scroll as time flows — each column a shifted sine"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w.max(1) as f32;
        let hf = h.max(1) as f32;
        let stripes = 4.0_f32; // number of diagonal stripe repeats
        let omega = 2.0 * PI * 1.5; // scroll speed
        let thresh = 0.3_f32;
        // Fill gate: only dots in [0, fill_x) are lit
        let fill_x = (ctx.eased * wf).round() as usize;

        for xi in 0..fill_x.min(w) {
            for yi in 0..h {
                let xn = xi as f32 / wf;
                let yn = yi as f32 / hf;
                // Phase gradient across x + y gives diagonal stripes
                let phase = 2.0 * PI * stripes * (xn + yn) - omega * ctx.time;
                if phase.sin() > thresh {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        // Unfilled region: dim ghost stripes (only column outline)
        for xi in fill_x.min(w)..w {
            for yi in 0..h {
                let xn = xi as f32 / wf;
                let yn = yi as f32 / hf;
                let phase = 2.0 * PI * stripes * (xn + yn) - omega * ctx.time;
                // Only light the bright peak in unfilled area
                if phase.sin() > 0.85 {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Sine area fill (smooth braille): the region 0 ≤ y ≤ |sin(kx)| · h · eased
//    is filled with braille dots. Pure smooth rendering — no block glyphs.
//    Time animates the phase so the filled ripple rolls.
// ---------------------------------------------------------------------------
struct AreaFill;
impl ProgressStyle for AreaFill {
    fn name(&self) -> &str {
        "sw-area-fill"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Sine area fill: |sin| landscape fills smoothly with braille dots up to eased height"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hf = h as f32;
        let k = 2.0 * PI * 4.0 / w.max(1) as f32; // 4 bumps across bar
        let phase = ctx.time * 2.0 * PI * 0.6;

        for xi in 0..w {
            let xf = xi as f32;
            let raw = (k * xf + phase).sin().abs(); // ∈ [0, 1]
                                                    // scale by eased to fill progressively
            let fill_h = (raw * ctx.eased * hf).round() as usize;
            let fill_h = fill_h.min(h);
            let y0 = h.saturating_sub(fill_h);
            for y in y0..h {
                draw::dot(grid, xi, y);
            }
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Blocky equalizer (explicit blocky counterpart of AreaFill):
//    The same |sin| amplitude computed per CELL column, then quantized to
//    block-eighths via draw::vblock — character-cell rendering only, no dots.
// ---------------------------------------------------------------------------
struct BlockyEq;
impl ProgressStyle for BlockyEq {
    fn name(&self) -> &str {
        "sw-blocky-eq"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Blocky sine equalizer: |sin| quantized to ▁▂▃▄▅▆▇█ columns — character-cell only, zero dots"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let k = 2.0 * PI * 4.0 / cw.max(1) as f32;
        let phase = ctx.time * 2.0 * PI * 0.6;
        let fill_cells = (ctx.eased * cw as f32).round() as usize;

        for cx in 0..cw {
            let xf = cx as f32;
            let amp = if cx < fill_cells {
                (k * xf + phase).sin().abs() // ∈ [0, 1]
            } else {
                // Ghost: dim un-filled columns
                (k * xf + phase).sin().abs() * 0.25
            };
            // Map to eighths per cell row
            let total_eighths = (amp * (ch * 8) as f32).round() as usize;
            let full_rows = total_eighths / 8;
            let rem_eighths = total_eighths % 8;

            // Fill from bottom up: full rows + partial top row
            for row_from_bot in 0..full_rows.min(ch) {
                let cy = ch.saturating_sub(1 + row_from_bot);
                draw::vblock(grid, cx, cy, 8);
            }
            if rem_eighths > 0 {
                let cy = ch.saturating_sub(1 + full_rows.min(ch));
                if full_rows < ch {
                    draw::vblock(grid, cx, cy, rem_eighths);
                }
            }
        }

        // Tint filled cells
        let (_, ch2) = grid.dimensions();
        for cx in 0..fill_cells.min(cw) {
            let t = if fill_cells <= 1 {
                0.5
            } else {
                cx as f32 / (fill_cells - 1) as f32
            };
            let col = ctx.palette.sample(t);
            for cy in 0..ch2 {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Rectified / clipped sine: y = clip(|sin(kx − ωt)|, threshold)
//     The waveform is always ≥ 0 (rectified); a hard top-clip flattens peaks.
//     Structurally distinct: no zero-crossings, flat tops, scalloped pattern.
//     eased sets the clip ceiling: low eased → shaved flat; high → full humps.
// ---------------------------------------------------------------------------
struct Rectified;
impl ProgressStyle for Rectified {
    fn name(&self) -> &str {
        "sw-rectified"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Rectified & clipped sine: |sin| with a hard ceiling — scalloped humps, no zero-crossings"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hf = h as f32;
        let k = 2.0 * PI * 5.0 / w.max(1) as f32; // 5 humps
        let omega = 2.0 * PI * 1.1;
        // Clip ceiling: rises with eased (more of the hump is shown)
        let ceiling = (0.2 + ctx.eased * 0.8).min(1.0);

        for xi in 0..w {
            let xf = xi as f32;
            // Rectified and clipped in [0, ceiling]
            let val = (k * xf - omega * ctx.time).sin().abs().min(ceiling);
            let fill_h = (val / ceiling.max(0.001) * hf).round() as usize;
            let fill_h = fill_h.min(h);
            let y0 = h.saturating_sub(fill_h);
            for y in y0..h {
                draw::dot(grid, xi, y);
            }
        }

        // Draw the clip line across the top as a visible ceiling
        let clip_y = (hf * (1.0 - ceiling)).round() as usize;
        if clip_y < h {
            draw::hline(grid, 0, w.saturating_sub(1), clip_y);
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Damped oscillation: y = e^(−γ·x) · sin(ω·x + φt)
//     Ring-down from the left edge. eased controls decay rate γ:
//     low eased → slow decay (many visible oscillations); high → fast ring-down.
//     Time provides a phase shift so the carrier ripples continuously.
// ---------------------------------------------------------------------------
struct Damped;
impl ProgressStyle for Damped {
    fn name(&self) -> &str {
        "sw-damped"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Damped oscillation e^(−γx)·sin(ωx): ring-down from left; decay rate rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);
        let gamma = 0.5 + ctx.eased * 4.0; // decay rate: 0.5..4.5
        let omega = 2.0 * PI * 6.0 / w.max(1) as f32; // 6 cycles at zero decay
        let phi = ctx.time * 2.0 * PI * 0.7; // carrier phase drift

        draw_curve(grid, w, h, |xi| {
            let xn = xi as f32 / w.max(1) as f32; // [0, 1]
            let env = (-gamma * xn).exp();
            let val = env * (omega * xi as f32 + phi).sin();
            (mid - (amp * val) as i32).clamp(0, h as i32 - 1)
        });

        // Draw the decaying envelope outline
        for xi in 0..w {
            let xn = xi as f32 / w.max(1) as f32;
            let env = (-gamma * xn).exp();
            let ey = (amp * env) as i32;
            draw::dot_i(grid, xi as i32, (mid - ey).clamp(0, h as i32 - 1));
            draw::dot_i(grid, xi as i32, (mid + ey).clamp(0, h as i32 - 1));
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Standing wave envelope: 2A·sin(kx)·cos(ωt)
//     Nodes are fixed in space; antinodes breathe in time.
//     eased selects mode n = 1 + floor(eased·5); each mode adds one more node.
//     Fills the area between the upper and lower envelope with dots.
// ---------------------------------------------------------------------------
struct StandingEnvelope;
impl ProgressStyle for StandingEnvelope {
    fn name(&self) -> &str {
        "sw-standing"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Standing wave: fixed-node antinodes breathe with time; mode count rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;
        let amp = (h as f32 * 0.44).max(1.0);
        let mode = (1 + (ctx.eased * 5.0).floor() as usize).min(6);
        let k = mode as f32 * PI / w.max(1) as f32;
        let omega = 2.5 * PI;
        let breath = (omega * ctx.time).cos(); // ∈ [-1, 1]

        // Fill between upper and lower envelope
        for xi in 0..w {
            let spatial = (k * xi as f32).sin(); // spatial factor
            let upper_y = (mid - (amp * spatial * breath.abs()) as i32).clamp(0, h as i32 - 1);
            let lower_y = (mid + (amp * spatial * breath.abs()) as i32).clamp(0, h as i32 - 1);
            let top = upper_y.min(lower_y) as usize;
            let bot = upper_y.max(lower_y) as usize;
            for y in top..=bot {
                draw::dot(grid, xi, y);
            }
        }

        // Draw the traveling wave itself (sign preserved for direction)
        draw_curve(grid, w, h, |xi| {
            let val = 2.0 * (k * xi as f32).sin() * breath;
            (mid - (amp * val) as i32).clamp(0, h as i32 - 1)
        });

        // Mark nodes with tiny vertical ticks
        for n in 0..=mode {
            let node_x = (n as f32 / mode as f32 * w.max(1) as f32) as usize;
            if node_x < w {
                let tick = (h / 8).max(1);
                draw::vline(
                    grid,
                    node_x,
                    (mid as usize).saturating_sub(tick),
                    ((mid as usize) + tick).min(h - 1),
                );
            }
        }

        tint_filled(grid, ctx);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 13. Sine-density texture: dot density in each column ∝ |sin(kx − ωt)|.
//     Each column gets a random-looking dither pattern with density set by the
//     sine value. Uses a row-hash to scatter dots vertically — no block glyphs.
//     Progress gates which columns participate: unfilled cols are near-empty.
// ---------------------------------------------------------------------------
struct Density;
impl ProgressStyle for Density {
    fn name(&self) -> &str {
        "sw-density"
    }
    fn theme(&self) -> &str {
        "sinewave"
    }
    fn describe(&self) -> &str {
        "Sine-density texture: dot density per column ∝ |sin| — dithered shading, no curves drawn"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let k = 2.0 * PI * 5.0 / w.max(1) as f32;
        let omega = 2.0 * PI * 0.9;
        let fill_x = (ctx.eased * w as f32).round() as usize;

        for xi in 0..w {
            let density = if xi < fill_x {
                (k * xi as f32 - omega * ctx.time).sin().abs() // ∈ [0, 1]
            } else {
                // Dim ghost in unfilled region
                (k * xi as f32 - omega * ctx.time).sin().abs() * 0.15
            };
            let lit_rows = (density * h as f32).round() as usize;
            // Scatter dots using a deterministic hash: row is lit if
            // (xi * 7 + yi * 13) mod prime < lit_rows * prime / h
            // This avoids vertical run artifacts.
            let prime = 31usize;
            let thresh = lit_rows * prime / h.max(1);
            for yi in 0..h {
                let hash = (xi.wrapping_mul(7).wrapping_add(yi.wrapping_mul(13))) % prime;
                if hash < thresh {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        tint_filled(grid, ctx);
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
    let styles = progress::styles::sinewave::styles();
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
