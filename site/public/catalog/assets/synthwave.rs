//! `synthwave` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O synthwave.rs && ./synthwave [style-name]
//! ```

const DEFAULT_STYLE: &str = "sunrise";

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
    pub mod synthwave {
//! Synthwave progress bars — outrun sunsets, neon grids, chrome and VHS.
//!
//! Every style is a little 1985-that-never-was: a striped sun climbs as
//! progress rises, perspective grids scroll toward the viewer, neon tubes
//! flicker on, chrome gleams sweep past. Palette is hot pink / violet /
//! electric cyan around a sunset core. Deterministic in `(progress, time)`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::TAU;

// ─── deterministic hash ─────────────────────────────────────────────────────

/// Fast integer hash → `[0, 1)`.
#[inline]
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) % 1000) as f32 / 1000.0
}

/// 3-D variant: hash `(x, y, z_int)` for time-slotted flicker.
#[inline]
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash2(x ^ z.wrapping_mul(1_234_567), y ^ z.wrapping_mul(7_654_321))
}

// ─── theme colors — sunset neon ─────────────────────────────────────────────

/// Hot pink, the signature neon.
const SW_PINK: Color = Color::rgb(255, 64, 168);
/// Electric cyan for horizons and highlights.
const SW_CYAN: Color = Color::rgb(64, 230, 255);
/// Deep violet for dark structure.
const SW_VIOLET: Color = Color::rgb(122, 74, 226);
/// Dusk purple for unlit track and far grid.
const SW_DUSK: Color = Color::rgb(88, 44, 128);
/// Sunset orange, the middle of the sun ramp.
const SW_ORANGE: Color = Color::rgb(255, 138, 66);
/// Sun-core yellow.
const SW_YELLOW: Color = Color::rgb(255, 216, 102);
/// White-hot sparkle.
const SW_WHITE: Color = Color::rgb(255, 244, 248);

/// Blend two colors at `t` in `0.0..=1.0`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let l = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(l(a.r, b.r), l(a.g, b.g), l(a.b, b.b))
}

/// Sun ramp: yellow at the top, orange through the middle, pink at the base.
fn sun_ramp(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        mix(SW_YELLOW, SW_ORANGE, t * 2.0)
    } else {
        mix(SW_ORANGE, SW_PINK, (t - 0.5) * 2.0)
    }
}

/// All styles in the `synthwave` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Sunrise),
        Box::new(GridRun),
        Box::new(NeonSign),
        Box::new(ChromeFade),
        Box::new(VhsTracking),
        Box::new(RetroEq),
        Box::new(LaserHorizon),
        Box::new(Starfall),
        Box::new(Outrun),
        Box::new(NeonWave),
    ]
}

/// The striped outrun sun climbs above a scrolling grid as progress rises.
struct Sunrise;
impl ProgressStyle for Sunrise {
    fn name(&self) -> &str {
        "sunrise"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Striped sun rising over a scrolling neon grid"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let horizon = h as i32 - 5;
        let cx = w as i32 / 2;
        let r = 7i32;
        // Sun center climbs from just-peeking to fully risen; a linear blend
        // keeps the first percent visible instead of hiding in the ease-in.
        let rise = 0.5 * ctx.progress + 0.5 * ctx.eased;
        let cy = horizon + r - 1 - (rise * (horizon + r - 5) as f32).round() as i32;
        let stripe_shift = (ctx.time * 4.0) as i32;
        for dy in -r..=r {
            let y = cy + dy;
            if y < 0 || y >= horizon {
                continue;
            }
            // Lower half of the disc carries the classic scan-gap stripes.
            if dy > 0 && (y + stripe_shift).rem_euclid(3) == 0 {
                continue;
            }
            let half = ((r * r - dy * dy) as f32).sqrt() as i32;
            for x in (cx - half)..=(cx + half) {
                draw::dot_i(grid, x, y);
            }
            let ramp = (dy + r) as f32 / (2 * r) as f32;
            let c0 = ((cx - half) / 2).max(0) as usize;
            let c1 = ((cx + half) / 2).max(0) as usize;
            draw::tint_row(grid, (y / 4) as usize, c0, c1, sun_ramp(ramp));
        }
        // Horizon line, lit outward from center with progress.
        let lit = (ctx.eased * cx as f32).round() as i32;
        draw::hline(grid, 0, w - 1, horizon as usize);
        draw::tint_row(grid, (horizon / 4) as usize, 0, w / 2, SW_DUSK);
        if lit > 0 {
            let c0 = ((cx - lit) / 2).max(0) as usize;
            let c1 = ((cx + lit) / 2) as usize;
            draw::tint_row(grid, (horizon / 4) as usize, c0, c1, SW_CYAN);
        }
        // Perspective floor: converging verticals plus rolling horizontals.
        let floor_cell = (horizon as usize / 4 + 1).min(ctx.height - 1);
        for k in -6..=6i32 {
            let bx = cx + k * 9;
            let steps = h as i32 - 1 - horizon;
            for s in 1..=steps {
                let x = cx + (bx - cx) * s / steps.max(1);
                draw::dot_i(grid, x, horizon + s);
            }
        }
        for i in 0..3 {
            let f = (ctx.time * 0.75 + i as f32 / 3.0).fract();
            let y = horizon + 1 + (f * f * (h as i32 - 2 - horizon) as f32) as i32;
            draw::hline(grid, 0, w - 1, y as usize);
        }
        for cy2 in floor_cell..ctx.height {
            draw::tint_row(grid, cy2, 0, ctx.width - 1, SW_VIOLET);
        }
        Ok(())
    }
}

/// A wall of light sweeps down a perspective grid toward the viewer.
struct GridRun;
impl ProgressStyle for GridRun {
    fn name(&self) -> &str {
        "gridrun"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Light wall racing down an endless neon grid"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let vp_y = 3i32;
        let cx = w as i32 / 2;
        // Twinkling stars above the vanishing point.
        let slot = (ctx.time * 2.0) as i32;
        for i in 0..10 {
            let sx = (hash2(i, 11) * w as f32) as i32;
            let sy = (hash2(i, 23) * vp_y as f32) as i32;
            if hash3(i, 0, slot) > 0.35 {
                draw::dot_i(grid, sx, sy);
            }
        }
        // The light wall: progress pushes it from the horizon to the viewer.
        let wall_y = vp_y + (ctx.eased * (h as i32 - 1 - vp_y) as f32).round() as i32;
        // Converging verticals, dotted, with a clear band above the wall so
        // the wall stays crisp against the lattice.
        for k in -8..=8i32 {
            let bx = cx + k * 10;
            let steps = h as i32 - 1 - vp_y;
            for s in (0..=steps).step_by(2) {
                let y = vp_y + s;
                if y < wall_y - 2 || y > wall_y + 2 {
                    let x = cx + (bx - cx) * s / steps.max(1);
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Rolling horizontals, accelerating as they near the viewer.
        for i in 0..4 {
            let f = (ctx.time * 0.5 + i as f32 * 0.25).fract();
            let y = vp_y + (f * f * (h as i32 - 1 - vp_y) as f32) as i32;
            if y < wall_y - 2 || y > wall_y + 2 {
                draw::hline(grid, 0, w - 1, y as usize);
            }
        }
        for dy in 0..2 {
            draw::hline(grid, 0, w - 1, (wall_y + dy).min(h as i32 - 1) as usize);
        }
        // Color: dim violet grid, pink glow below the wall, cyan wall crest.
        for cy in 0..ctx.height {
            let row_mid = cy as i32 * 4 + 2;
            let c = if row_mid < wall_y {
                SW_DUSK
            } else if row_mid < wall_y + 4 {
                SW_CYAN
            } else {
                SW_PINK
            };
            draw::tint_row(grid, cy, 0, ctx.width - 1, c);
        }
        Ok(())
    }
}

/// A neon tube border lights up clockwise; the readout buzzes in the middle.
struct NeonSign;
impl ProgressStyle for NeonSign {
    fn name(&self) -> &str {
        "neon-sign"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Neon tube border flickering on, percent in lights"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (x0, y0, x1, y1) = (1i32, 1i32, w as i32 - 2, h as i32 - 2);
        // Walk the tube perimeter clockwise from the top-left corner.
        let top = x1 - x0;
        let right = y1 - y0;
        let perim = 2 * (top + right);
        let lit = (ctx.eased * perim as f32).round() as i32;
        let slot = (ctx.time * 4.0) as i32;
        for s in 0..perim {
            let (x, y) = if s < top {
                (x0 + s, y0)
            } else if s < top + right {
                (x1, y0 + (s - top))
            } else if s < 2 * top + right {
                (x1 - (s - top - right), y1)
            } else {
                (x0, y1 - (s - 2 * top - right))
            };
            let on = s < lit;
            // Fresh tube segments flicker as they warm up; old ones buzz rarely.
            let seg = s / 6;
            let fresh = on && lit - s < perim / 8;
            let buzz = hash3(seg, 3, slot);
            let bright = on && !(fresh && buzz < 0.4) && buzz >= 0.05;
            // Unlit tube is sparse glass; lit tube is a solid run of light,
            // so the fill reads even without color — and flickering segments
            // actually drop out of the tube, not just dim.
            if !on && s % 3 != 0 {
                continue;
            }
            if on && !bright && s % 2 != 0 {
                continue;
            }
            draw::dot_i(grid, x, y);
            let cell = ((x / 2) as usize, (y / 4) as usize);
            let _ = grid.set_cell_color(
                cell.0,
                cell.1,
                if bright {
                    SW_PINK
                } else if on {
                    SW_VIOLET
                } else {
                    SW_DUSK
                },
            );
        }
        // Percent readout in cyan lights, center stage.
        if let Some(label) = &ctx.label {
            let chars: Vec<char> = label.chars().collect();
            let cw = ctx.width;
            let cx0 = cw.saturating_sub(chars.len()) / 2;
            let cy = ctx.height / 2;
            for (i, c) in chars.iter().enumerate() {
                draw::glyph(grid, cx0 + i, cy, *c);
                let flick = hash3(i as i32, 9, slot) > 0.08;
                let _ = grid.set_cell_color(cx0 + i, cy, if flick { SW_CYAN } else { SW_DUSK });
            }
        }
        Ok(())
    }
}

/// A chrome bar with mirror banding and a gleam that sweeps past.
struct ChromeFade;
impl ProgressStyle for ChromeFade {
    fn name(&self) -> &str {
        "chrome-fade"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Mirror-chrome fill with a sweeping gleam"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let top = (h / 4).max(1);
        let bot = h.saturating_sub(h / 4 + 1).max(top);
        let filled = (ctx.eased * w as f32).round() as usize;
        // Track rails.
        draw::hline(grid, 0, w - 1, top.saturating_sub(2));
        draw::hline(grid, 0, w - 1, (bot + 2).min(h - 1));
        for cy in 0..ctx.height {
            draw::tint_row(grid, cy, 0, ctx.width - 1, SW_DUSK);
        }
        // Chrome body.
        for x in 0..filled {
            for y in top..=bot {
                draw::dot(grid, x, y);
            }
        }
        // Vertical leading edge.
        if filled > 0 && filled < w {
            draw::vline(grid, filled, top.saturating_sub(1), bot + 1);
        }
        // Banding: sky above the seam, sunset metal below. The seam bows
        // gently with time like a horizon reflected in curved chrome.
        let gleam = ((ctx.time * 0.25).fract() * (w as f32 + 24.0)) as i32 - 12;
        for cy in (top / 4)..=(bot / 4) {
            let row_mid = cy as f32 * 4.0 + 2.0;
            let seam = (h as f32 / 2.0) + (TAU * 0.25 * ctx.time).sin() * 1.5;
            let c = if row_mid < seam - 2.0 {
                mix(SW_CYAN, SW_WHITE, 0.55)
            } else if row_mid < seam {
                SW_WHITE
            } else if row_mid < seam + 3.0 {
                SW_ORANGE
            } else {
                SW_PINK
            };
            let hi_cell = (filled / 2).min(ctx.width.saturating_sub(1));
            draw::tint_row(grid, cy, 0, hi_cell, c);
            // Gleam: a diagonal white flash sliding along the filled chrome.
            for gx in 0..3i32 {
                let x = gleam + gx - (cy as i32 * 2);
                if x >= 0 && (x as usize) < filled {
                    let _ = grid.set_cell_color((x / 2) as usize, cy, SW_WHITE);
                }
            }
        }
        // Sparkle at the leading edge.
        if filled > 1 && filled < w - 1 {
            let sx = filled as i32;
            let sy = (top + bot) as i32 / 2;
            let pulse = ((ctx.time * TAU * 0.5).sin() * 2.0) as i32 + 2;
            draw::hline(
                grid,
                (sx - pulse).max(0) as usize,
                (sx + pulse) as usize,
                sy as usize,
            );
            draw::vline(
                grid,
                sx as usize,
                (sy - pulse).max(0) as usize,
                (sy + pulse) as usize,
            );
            let _ = grid.set_cell_color((sx / 2) as usize, (sy / 4) as usize, SW_WHITE);
        }
        Ok(())
    }
}

/// Tape fill with tracking jitter that settles as the signal locks in.
struct VhsTracking;
impl ProgressStyle for VhsTracking {
    fn name(&self) -> &str {
        "vhs-tracking"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "VHS fill stabilizing as tracking locks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32) as i32;
        let slot = (ctx.time * 6.0) as i32;
        let unstable = 1.0 - ctx.progress;
        // The picture: full-height fill, rows shoved sideways in bands of two
        // dots. Jitter is worst near the leading edge and calms with progress.
        for y in 0..h as i32 {
            let band = y / 2;
            let j = ((hash3(band, 1, slot) - 0.5) * 7.0 * unstable) as i32;
            let x1 = (filled + j).clamp(0, w as i32);
            for x in 0..x1 {
                draw::dot_i(grid, x, y);
            }
        }
        // A head-switch noise band rolls up the frame.
        let band_y = (((1.0 - (ctx.time * 0.5).fract()) * h as f32) as i32).min(h as i32 - 2);
        for y in band_y..(band_y + 2).min(h as i32) {
            for x in 0..w as i32 {
                if hash3(x, y, slot) > 0.45 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Washed-tape tint with chroma-fringe rows near the noise band.
        for cy in 0..ctx.height {
            let row_mid = cy as i32 * 4 + 2;
            let c = if (row_mid - band_y).abs() < 3 {
                SW_WHITE
            } else if (row_mid - band_y).abs() < 6 {
                if cy % 2 == 0 {
                    SW_PINK
                } else {
                    SW_CYAN
                }
            } else {
                mix(SW_CYAN, SW_WHITE, 0.35)
            };
            draw::tint_row(grid, cy, 0, ctx.width - 1, c);
        }
        Ok(())
    }
}

/// Bouncing equalizer columns wake up left to right with progress.
struct RetroEq;
impl ProgressStyle for RetroEq {
    fn name(&self) -> &str {
        "retro-eq"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Neon equalizer columns waking up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let bar_w = 4usize;
        let gap = 2usize;
        let n = w / (bar_w + gap);
        let margin = (w - n * (bar_w + gap) + gap) / 2;
        let active = (ctx.eased * n as f32).round() as usize;
        for i in 0..n {
            let x0 = i * (bar_w + gap) + margin;
            let t = i as f32 / n.max(1) as f32;
            let height = if i < active {
                // Rates quantized to quarter-hertz so the 4s loop is seamless.
                let rate = 0.5 + 0.25 * (hash2(i as i32, 5) * 4.0).floor();
                let bounce = (TAU * (ctx.time * rate + hash2(i as i32, 9))).sin().abs();
                (2.0 + bounce * (h as f32 - 3.0)) as usize
            } else {
                1
            };
            for y in (h - height.min(h))..h {
                for x in x0..(x0 + bar_w).min(w) {
                    draw::dot(grid, x, y);
                }
            }
            // Column color ramps pink→cyan; the cap cell burns white.
            let color = mix(SW_PINK, SW_CYAN, t);
            let top_cell = (h - height.min(h)) / 4;
            for cy in top_cell..ctx.height {
                let c0 = x0 / 2;
                let c1 = (x0 + bar_w - 1) / 2;
                let c = if cy == top_cell && i < active {
                    SW_WHITE
                } else {
                    color
                };
                draw::tint_row(grid, cy, c0, c1, c);
            }
        }
        Ok(())
    }
}

/// A fan of sky lasers switches on beam by beam over a dark grid.
struct LaserHorizon;
impl ProgressStyle for LaserHorizon {
    fn name(&self) -> &str {
        "laser-horizon"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Sky lasers fanning on over the horizon"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let horizon = h as i32 - 4;
        let cx = w as i32 / 2;
        // Dark floor grid.
        for k in -5..=5i32 {
            let bx = cx + k * 11;
            let steps = h as i32 - 1 - horizon;
            for s in 1..=steps {
                draw::dot_i(grid, cx + (bx - cx) * s / steps.max(1), horizon + s);
            }
        }
        draw::hline(grid, 0, w - 1, horizon as usize);
        // Beams fan out over 180°, switching on with progress; the whole fan
        // breathes side to side with time.
        let beams = 9;
        let lit = (ctx.eased * beams as f32).round() as i32;
        let sway = (TAU * 0.25 * ctx.time).sin() * 0.18;
        let mut pulse_cells: Vec<(usize, usize)> = Vec::new();
        for b in 0..beams {
            if b >= lit {
                continue;
            }
            let ang = std::f32::consts::PI * (0.08 + 0.84 * b as f32 / (beams - 1) as f32) + sway;
            let (dx, dy) = (ang.cos(), -ang.sin());
            let mut i = 0f32;
            loop {
                let x = cx as f32 + dx * i;
                let y = horizon as f32 + dy * i;
                if x < 0.0 || x >= w as f32 || y < 0.0 {
                    break;
                }
                // Solid beam with a bright pulse racing outward.
                draw::dot_i(grid, x as i32, y as i32);
                if ((ctx.time * 1.0 + b as f32 * 0.25).fract() * 60.0 - i).abs() < 3.0 {
                    pulse_cells.push((x as usize / 2, y as usize / 4));
                }
                i += 1.0;
            }
        }
        // Tint: pink beams in the sky, cyan horizon, violet floor.
        for cy in 0..ctx.height {
            let row_mid = cy as i32 * 4 + 2;
            if (row_mid - horizon).abs() <= 2 {
                draw::tint_row(grid, cy, 0, ctx.width - 1, SW_CYAN);
            } else if row_mid > horizon {
                draw::tint_row(grid, cy, 0, ctx.width - 1, SW_VIOLET);
            }
        }
        // Sky rows: paint only cells the beams touched, then re-flash pulses.
        let sky_cells = (horizon as usize) / 4;
        for cy in 0..sky_cells {
            for cxl in 0..ctx.width {
                let ch = grid.get_char(cxl, cy);
                if ch != '\u{2800}' && ch != ' ' {
                    let _ = grid.set_cell_color(cxl, cy, SW_PINK);
                }
            }
        }
        for (px, py) in pulse_cells {
            let _ = grid.set_cell_color(px, py, SW_WHITE);
        }
        Ok(())
    }
}

/// Meteors streak across a twinkling sky while the ground bar fills.
struct Starfall;
impl ProgressStyle for Starfall {
    fn name(&self) -> &str {
        "starfall"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Shooting stars over a filling neon skyline"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let ground = h - 3;
        let slot = (ctx.time * 2.0) as i32;
        // Star field.
        for i in 0..26 {
            let sx = (hash2(i, 31) * w as f32) as i32;
            let sy = (hash2(i, 47) * (ground as f32 - 2.0)) as i32;
            if hash3(i, 1, slot) > 0.3 {
                draw::dot_i(grid, sx, sy);
            }
        }
        // Shooting stars: heads on quarter-hertz cycles with six-dot tails.
        let mut head_cells: Vec<(usize, usize)> = Vec::new();
        for m in 0..3i32 {
            let f = (ctx.time * 0.25 + m as f32 / 3.0).fract();
            let sx = hash2(m, 77) * w as f32 * 0.7;
            let head_x = sx + f * w as f32 * 0.9;
            let head_y = f * (ground as f32 - 2.0);
            for k in 0..6i32 {
                let x = head_x as i32 - k * 2;
                let y = head_y as i32 - k;
                draw::dot_i(grid, x, y);
                if k == 0 && x >= 0 && y >= 0 {
                    head_cells.push((x as usize / 2, y as usize / 4));
                }
            }
        }
        // Sky tint: violet field, then white-hot meteor heads on top.
        for cy in 0..(ground / 4) {
            for cxl in 0..ctx.width {
                let ch = grid.get_char(cxl, cy);
                if ch != '\u{2800}' && ch != ' ' {
                    let _ = grid.set_cell_color(cxl, cy, SW_VIOLET);
                }
            }
        }
        for (hx, hy) in head_cells {
            let _ = grid.set_cell_color(hx, hy, SW_WHITE);
        }
        // Ground bar: solid pink fill over a sparse dotted track, so the
        // read survives in monochrome too.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, ground + 1);
        }
        for y in ground..h {
            for x in 0..filled {
                draw::dot(grid, x, y);
            }
        }
        let gcell = ground / 4;
        draw::tint_row(grid, gcell, 0, ctx.width - 1, SW_DUSK);
        if filled > 0 {
            draw::tint_row(grid, gcell, 0, filled / 2, SW_PINK);
            let _ = grid.set_cell_color((filled / 2).min(ctx.width - 1), gcell, SW_CYAN);
        }
        Ok(())
    }
}

/// A little coupe drives the bar toward the sun, dashes streaming past.
struct Outrun;
impl ProgressStyle for Outrun {
    fn name(&self) -> &str {
        "outrun"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Coupe cruising the bar toward the sun"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let road = h as i32 - 4;
        // Destination sun, low on the right.
        let (sun_x, sun_r) = (w as i32 - 7, 5i32);
        let stripe_shift = (ctx.time * 3.0) as i32;
        for dy in -sun_r..=0 {
            let y = road - 2 + dy;
            if y < 0 || (dy > -2 && (y + stripe_shift).rem_euclid(2) == 0) {
                continue;
            }
            let half = ((sun_r * sun_r - dy * dy) as f32).sqrt() as i32;
            for x in (sun_x - half)..=(sun_x + half) {
                draw::dot_i(grid, x, y);
            }
            draw::tint_row(
                grid,
                (y / 4) as usize,
                ((sun_x - half) / 2).max(0) as usize,
                ((sun_x + half) / 2) as usize,
                sun_ramp((dy + sun_r) as f32 / sun_r as f32),
            );
        }
        // Road with center dashes streaming left as the car "moves".
        draw::hline(grid, 0, w - 1, road as usize);
        let dash_off = ((ctx.time * 8.0) as i32) % 8;
        let mut x = -dash_off;
        while x < w as i32 {
            for k in 0..4 {
                draw::dot_i(grid, x + k, road + 2);
            }
            x += 8;
        }
        // The coupe: an 8×3 wedge with a spoiler, riding at eased.
        let car_x = (ctx.eased * (w as f32 - 12.0)) as i32 + 1;
        let car_y = road - 3;
        for k in 2..8 {
            draw::dot_i(grid, car_x + k, car_y);
        }
        for k in 0..9 {
            draw::dot_i(grid, car_x + k, car_y + 1);
            draw::dot_i(grid, car_x + k, car_y + 2);
        }
        // Wheel bumps.
        draw::dot_i(grid, car_x + 2, car_y + 3);
        draw::dot_i(grid, car_x + 6, car_y + 3);
        // Exhaust puffs trailing off behind.
        let slot = (ctx.time * 4.0) as i32;
        for p in 1..5i32 {
            let px = car_x - p * 3 - ((ctx.time * 8.0) as i32 % 3);
            if px >= 0 && hash3(p, 2, slot) > 0.35 {
                draw::dot_i(grid, px, car_y + 1 + (hash3(p, 5, slot) * 2.0) as i32);
            }
        }
        // Tints: pink car, cyan road, violet dashes.
        for cxl in 0..ctx.width {
            draw::tint_row(grid, (road / 4) as usize, cxl, cxl, SW_CYAN);
        }
        let car_cell_y = (car_y / 4).max(0) as usize;
        for cxl in (car_x / 2).max(0)..=((car_x + 9) / 2).min(ctx.width as i32 - 1) {
            let _ = grid.set_cell_color(cxl as usize, car_cell_y, SW_PINK);
        }
        Ok(())
    }
}

/// Twin neon sine ribbons snake across the bar as far as progress allows.
struct NeonWave;
impl ProgressStyle for NeonWave {
    fn name(&self) -> &str {
        "neon-wave"
    }
    fn theme(&self) -> &str {
        "synthwave"
    }
    fn describe(&self) -> &str {
        "Pink and cyan sine ribbons racing to the edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h as f32 / 2.0;
        let filled = (ctx.eased * w as f32).round() as usize;
        // Dim track line so the remaining path stays visible.
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, mid as usize);
        }
        for cy in 0..ctx.height {
            draw::tint_row(grid, cy, 0, ctx.width - 1, SW_DUSK);
        }
        // Two ribbons, opposite phase velocities, meeting glow in white.
        for x in 0..filled {
            let xf = x as f32;
            let y1 = mid + (xf * 0.12 + TAU * 0.5 * ctx.time).sin() * (mid - 2.0);
            let y2 = mid + (xf * 0.155 - TAU * 0.5 * ctx.time + 1.2).sin() * (mid - 2.0);
            for (y, c) in [(y1, SW_PINK), (y2, SW_CYAN)] {
                let yi = y as i32;
                draw::dot_i(grid, x as i32, yi);
                draw::dot_i(grid, x as i32, yi + 1);
                let cell = (x / 2, (yi.max(0) as usize) / 4);
                let close = (y1 - y2).abs() < 2.5;
                let _ = grid.set_cell_color(
                    cell.0,
                    cell.1.min(ctx.height - 1),
                    if close { SW_WHITE } else { c },
                );
            }
        }
        // Leading spark.
        if filled > 0 && filled < w {
            let xf = filled as f32;
            let y = mid + (xf * 0.12 + TAU * 0.5 * ctx.time).sin() * (mid - 2.0);
            for d in 0..3i32 {
                draw::dot_i(grid, filled as i32 + d, y as i32);
            }
            let _ = grid.set_cell_color(
                (filled / 2).min(ctx.width - 1),
                ((y as usize) / 4).min(ctx.height - 1),
                SW_WHITE,
            );
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
    let styles = progress::styles::synthwave::styles();
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
