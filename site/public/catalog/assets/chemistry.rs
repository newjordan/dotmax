//! `chemistry` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O chemistry.rs && ./chemistry [style-name]
//! ```

const DEFAULT_STYLE: &str = "periodic-table";

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
    pub mod chemistry {
//! Chemistry-themed progress bars for dotmax.
//!
//! Each bar visualises a distinct chemical phenomenon using real physical
//! behaviour — no style differs merely by palette. Braille dots represent
//! particle clouds and kinetic curves; block glyphs represent periodic table
//! cells, pH gradients, and histogram columns.
//!
//! ## Styles (11 total)
//!
//! | Name | Chemical concept |
//! |------|-----------------|
//! | `periodic-table` | Element cells nucleate one-by-one across a grid |
//! | `electron-orbitals` | s/p/d orbital dot-clouds grow with eased progress |
//! | `titration-curve` | Sigmoid pH–volume curve draws out; drops fall via time |
//! | `reaction-kinetics` | [A] exponential decay + [B] sigmoid rise to equilibrium |
//! | `crystallization` | Lattice nucleates at a seed and grows outward |
//! | `ph-scale` | Indicator dot slides along a shaded acid→base track |
//! | `gas-diffusion` | Particles random-walk and spread; spread = eased |
//! | `distillation` | Vapor rises, condenses, drips into receiver that fills |
//! | `bond-vibration` | Two atoms oscillate on a spring; bond length = time |
//! | `boltzmann-distribution` | Velocity histogram fills to Maxwell-Boltzmann shape |
//! | `flame-spectral-lines` | Discrete emission lines appear as progress advances |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `chemistry` theme.
///
/// Returns 11 distinct bars, each encoding a real chemical process.
/// Every bar is safe to render from a 1×1 cell grid up to 80×8 or larger.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PeriodicTable),
        Box::new(ElectronOrbitals),
        Box::new(TitrationCurve),
        Box::new(ReactionKinetics),
        Box::new(Crystallization),
        Box::new(PhScale),
        Box::new(GasDiffusion),
        Box::new(Distillation),
        Box::new(BondVibration),
        Box::new(BoltzmannDistribution),
        Box::new(FlameSpectralLines),
    ]
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Cheap deterministic hash for seeding pseudo-randomness without rand crate.
#[inline]
fn hash2(a: usize, b: usize) -> u32 {
    let mut x = (a as u32)
        .wrapping_mul(2654435761)
        .wrapping_add(b as u32)
        .wrapping_mul(2246822519);
    x ^= x >> 15;
    x = x.wrapping_mul(2246822519);
    x ^= x >> 13;
    x
}

// ---------------------------------------------------------------------------
// 1. Periodic table
//    Element cells fill one-by-one reading left-to-right, top-to-bottom
//    using vertical-block glyphs. eased controls how many cells are filled.
//    Each filled cell shows a solid █; partial progress uses V_BLOCKS.
// ---------------------------------------------------------------------------
struct PeriodicTable;
impl ProgressStyle for PeriodicTable {
    fn name(&self) -> &str {
        "periodic-table"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Periodic table: element cells fill one-by-one left-to-right, top-to-bottom as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        let total_cells = cw * ch;
        // How many full cells are lit by eased progress
        let lit_f = ctx.eased * total_cells as f32;
        let full = lit_f.floor() as usize;
        let frac = lit_f - full as f32; // 0..1 partial

        for idx in 0..total_cells {
            let cx = idx % cw;
            let cy = idx / cw;
            if idx < full {
                // Fully filled — solid block glyph
                draw::glyph(grid, cx, cy, '█');
            } else if idx == full && frac > 0.0 {
                // Partial cell: pick a vertical-block level 1..8
                let level = (frac * 8.0).round() as usize;
                draw::vblock(grid, cx, cy, level.max(1));
            }
            // else: empty cell — leave blank
        }

        // Tint filled region with palette gradient
        for idx in 0..full.min(total_cells) {
            let cx = idx % cw;
            let cy = idx / cw;
            let t = idx as f32 / total_cells as f32;
            draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Electron orbitals
//    s orbital (sphere), p orbital (dumbbell), d orbital (cloverleaf) —
//    shape index cycles through s→p→d as eased increases (thirds).
//    Dot density builds with eased within each shape.
// ---------------------------------------------------------------------------
struct ElectronOrbitals;
impl ProgressStyle for ElectronOrbitals {
    fn name(&self) -> &str {
        "electron-orbitals"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Electron orbitals: s-sphere, p-dumbbell, d-cloverleaf shapes grow as progress fills the subshell"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Shape index: 0=s, 1=p, 2=d  (advances with eased)
        let shape = ((ctx.eased * 3.0).floor() as usize).min(2);
        // Within each third, local progress 0..1 drives cloud density
        let local = (ctx.eased * 3.0).fract();

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let rx = (w as f32 * 0.48).max(1.0);
        let ry = (h as f32 * 0.48).max(1.0);

        // Animate orientation with time
        let phase = ctx.time * 0.4;

        // Threshold: only draw dots whose orbital density exceeds (1-local)
        // giving a cloud that fills in as local → 1
        let threshold = 1.0 - local;

        for yi in 0..h {
            for xi in 0..w {
                let nx = (xi as f32 - cx) / rx; // normalised [-1, 1]
                let ny = (yi as f32 - cy) / ry;
                let r = (nx * nx + ny * ny).sqrt();
                if r > 1.1 {
                    continue;
                }

                let density = match shape {
                    // s: spherical — density = Gaussian on r
                    0 => {
                        let sigma = 0.5_f32;
                        (-(r * r) / (2.0 * sigma * sigma)).exp()
                    }
                    // p: dumbbell along rotated axis
                    // ψ_p ∝ cos(θ) where θ is angle from bond axis
                    1 => {
                        let angle = ny.atan2(nx) - phase;
                        let lobe = angle.cos().powi(2);
                        let radial = (-(r * r) / 0.4).exp();
                        lobe * radial
                    }
                    // d: cloverleaf — 4-lobe pattern  ψ_d ∝ cos(2θ)
                    _ => {
                        let angle = ny.atan2(nx) - phase * 0.5;
                        let lobe = (2.0 * angle).cos().powi(2);
                        let radial = (-(r * r) / 0.35).exp();
                        lobe * radial
                    }
                };

                if density > threshold {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        // Tint whole bar by orbital shape colour
        let (cw, ch) = grid.dimensions();
        for cx2 in 0..cw {
            let t = ctx.eased;
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Titration curve
//    Sigmoid pH-vs-volume curve draws out left-to-right as eased advances.
//    ctx.time drives "drip" marks that fall down at regular intervals —
//    each drip is a single dot column descending from y=0.
//    Equivalence point is marked with a tick at pH 7.
// ---------------------------------------------------------------------------
struct TitrationCurve;
impl ProgressStyle for TitrationCurve {
    fn name(&self) -> &str {
        "titration-curve"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Titration: a sigmoid pH–volume curve draws out while drips fall and the equivalence point appears"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // --- Draw the sigmoid pH curve ---
        // Map progress [0,1] to volume axis [0, w).
        // pH(v) = 7 + 4·tanh(10·(v - 0.5))  — steep at equivalence
        let drawn_x = (ctx.eased * w as f32).round() as usize;

        let mut prev_y: Option<i32> = None;
        for xi in 0..drawn_x.min(w) {
            let v = xi as f32 / w as f32; // volume fraction [0,1]
            let ph = 7.0 + 4.0 * (10.0 * (v - 0.5)).tanh();
            // Map pH [3, 11] to dot row [h-1, 0]
            let norm = (ph - 3.0) / 8.0; // [0,1] low→high pH
            let row = ((1.0 - norm) * (h - 1) as f32).round() as i32;
            let row = row.clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, row);
            if let Some(py) = prev_y {
                let lo = py.min(row);
                let hi = py.max(row);
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(row);
        }

        // --- Equivalence point tick at pH 7 (v = 0.5) ---
        let eq_x = (w / 2) as i32;
        let eq_ph = 7.0_f32;
        let eq_norm = (eq_ph - 3.0) / 8.0;
        let eq_y = ((1.0 - eq_norm) * (h - 1) as f32).round() as i32;
        // Vertical tick through equivalence point
        let tick = (h / 6).max(1) as i32;
        for yy in (eq_y - tick).max(0)..=(eq_y + tick).min(h as i32 - 1) {
            draw::dot_i(grid, eq_x, yy);
        }

        // --- Drips: falling dots driven by time ---
        // One drip per 0.4 s interval; drop falls at speed 4 dots/s
        let drip_interval = 0.4_f32;
        let drip_speed = 6.0_f32; // dots per second
                                  // Only drip left of the drawn curve
        let n_drips = 5usize;
        for di in 0..n_drips {
            let t_offset = di as f32 * drip_interval;
            let t_local = (ctx.time - t_offset).rem_euclid(drip_interval * n_drips as f32);
            if t_local < 0.0 {
                continue;
            }
            let drop_y = (t_local * drip_speed) as i32;
            // x position: evenly spaced across drawn region
            let drip_x = if drawn_x > 0 {
                (di * drawn_x / n_drips.max(1)) as i32
            } else {
                0
            };
            if drop_y < h as i32 {
                draw::dot_i(grid, drip_x, drop_y);
            }
        }

        // Tint by eased
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Reaction kinetics
//    [A] = A₀·e^(−k·eased·3) — reactant decays (drawn as descending curve).
//    [B] = 1 − [A]           — product rises (drawn as ascending curve).
//    Two distinct dot-curves fill simultaneously; below A-curve is shaded
//    lightly; below B-curve filled solidly.
// ---------------------------------------------------------------------------
struct ReactionKinetics;
impl ProgressStyle for ReactionKinetics {
    fn name(&self) -> &str {
        "reaction-kinetics"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Reaction kinetics: [A] decays exponentially while [B] rises to equilibrium — both curves fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // How far along the time axis we've drawn (eased = 0..1 maps to full timeline)
        let drawn_x = (ctx.eased * w as f32).round() as usize;
        let k = 3.0_f32; // rate constant (visual scale)

        let mut prev_a: Option<i32> = None;
        let mut prev_b: Option<i32> = None;

        for xi in 0..drawn_x.min(w) {
            let t_norm = xi as f32 / w as f32; // 0..1
            let a = (-k * t_norm).exp(); // [A] in [1, 0]
            let b = 1.0 - a; // [B] in [0, 1]

            // A-curve: high concentration = near top (row 0), low = near bottom
            let row_a = ((1.0 - a) * (h - 1) as f32).round() as i32;
            let row_a = row_a.clamp(0, h as i32 - 1);
            // B-curve: low → high, inverse mapping
            let row_b = ((1.0 - b) * (h - 1) as f32).round() as i32;
            let row_b = row_b.clamp(0, h as i32 - 1);

            draw::dot_i(grid, xi as i32, row_a);
            draw::dot_i(grid, xi as i32, row_b);

            if let Some(pa) = prev_a {
                let (lo, hi) = (pa.min(row_a), pa.max(row_a));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            if let Some(pb) = prev_b {
                let (lo, hi) = (pb.min(row_b), pb.max(row_b));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }

            // Fill below the B-curve (product accumulation region)
            for y in (row_b as usize)..h {
                // Every other dot for a lighter fill texture
                if (xi + y) % 2 == 0 {
                    draw::dot(grid, xi, y);
                }
            }

            prev_a = Some(row_a);
            prev_b = Some(row_b);
        }

        // Colour: start = reactant (red-ish via palette start), end = product
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Crystallization
//    A seed appears at the centre; the crystal lattice grows outward Manhattan-
//    distance-first. Cells within radius r = eased × max_r are drawn as dots
//    only if they sit on lattice positions (x and y both multiples of spacing).
//    Off-lattice cells that are very close show shade characters.
// ---------------------------------------------------------------------------
struct Crystallization;
impl ProgressStyle for Crystallization {
    fn name(&self) -> &str {
        "crystallization"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Crystallization: a lattice nucleates at the seed and grows outward — atoms snap onto grid positions"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;

        // Lattice spacing in dots — tighter for wider grids
        let spacing = (w / 10).max(2) as i32;

        // Maximum growth radius = diagonal of the half-grid
        let max_r = (((cx * cx + cy * cy) as f32).sqrt()).max(1.0);
        let r = ctx.eased * max_r;

        // Draw lattice nodes within radius r
        // Step across dot-space in lattice units
        let mut lx = -(cx / spacing) * spacing - spacing;
        while lx <= cx + cx {
            let mut ly = -(cy / spacing) * spacing - spacing;
            while ly <= cy + cy {
                let dx = lx as f32;
                let dy = ly as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= r {
                    let px = cx + lx;
                    let py = cy + ly;
                    // Node dot
                    draw::dot_i(grid, px, py);
                    // Bond lines to right and down neighbours (only if both in radius)
                    let nx = lx + spacing;
                    let ny = ly + spacing;
                    let dist_r = ((nx as f32).powi(2) + (dy).powi(2)).sqrt();
                    let dist_d = ((dx).powi(2) + (ny as f32).powi(2)).sqrt();
                    if dist_r <= r && px >= 0 && (cx + nx) < w as i32 {
                        draw::hline(
                            grid,
                            px.max(0) as usize,
                            (cx + nx).max(0).min(w as i32 - 1) as usize,
                            py.max(0).min(h as i32 - 1) as usize,
                        );
                    }
                    if dist_d <= r && py >= 0 && (cy + ny) < h as i32 {
                        draw::vline(
                            grid,
                            px.max(0).min(w as i32 - 1) as usize,
                            py.max(0) as usize,
                            (cy + ny).max(0).min(h as i32 - 1) as usize,
                        );
                    }
                }
                ly += spacing;
            }
            lx += spacing;
        }

        // Shade cells near the growth frontier
        let (cw, ch) = grid.dimensions();
        for cx2 in 0..cw {
            for cy2 in 0..ch {
                let dx = cx2 as i32 - cw as i32 / 2;
                let dy = cy2 as i32 - ch as i32 / 2;
                // dot distance from seed
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                // shade level: full inside crystal, trail near frontier
                let r_cells = r / 2.0; // cell units (approx)
                if d <= r_cells {
                    let t = d / r_cells.max(1.0);
                    draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(1.0 - t * 0.5));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. pH scale
//    A shaded acid→base gradient track spans the full width.
//    The measured pH = eased × 14 positions a sliding indicator dot-column
//    that moves left (acid) to right (base). Structural novelty: it is a
//    SLIDING MARKER, not just changing colour.
// ---------------------------------------------------------------------------
struct PhScale;
impl ProgressStyle for PhScale {
    fn name(&self) -> &str {
        "ph-scale"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "pH scale: a marker slides left (acid) to right (base) along a shaded gradient track"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w == 0 || h == 0 || cw == 0 || ch == 0 {
            return Ok(());
        }

        // --- Gradient track using shade glyphs ---
        // Shade 0 (acid, left) → shade 4 (base, right) per cell
        for cx in 0..cw {
            let t = cx as f32 / cw.saturating_sub(1).max(1) as f32;
            // shade level: 0-1 at acid end, 3-4 at base end
            let level = (t * 4.0).round() as usize;
            for cy in 0..ch {
                // Middle rows get the full shade; outer rows get one step lighter
                let adj = if ch > 2 && (cy == 0 || cy == ch - 1) {
                    level.saturating_sub(1)
                } else {
                    level
                };
                draw::shade(grid, cx, cy, adj.min(4));
            }
            // Tint: red at acid end → purple/blue at base end
            // We use palette.sample across the full track
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }

        // --- Sliding indicator: a bright full-height block column ---
        // Must be a glyph: every track cell already holds a shade glyph, and
        // glyphs override braille dots, so a dot-drawn marker would be
        // invisible. pH = eased × 14, position = eased × (cw-1).
        let icx = (ctx.eased * cw.saturating_sub(1) as f32).round() as usize;
        for cy in 0..ch {
            draw::glyph(grid, icx, cy, '█');
            draw::tint_row(grid, cy, icx, icx, crate::Color::rgb(255, 250, 240));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Gas diffusion
//    Particles start at a release point (left centre) and random-walk
//    outward; the spread radius = eased × half_width.  Particle positions
//    are deterministic (seeded from particle index) and the cloud density
//    decays as a Gaussian so it looks like real diffusion.
// ---------------------------------------------------------------------------
struct GasDiffusion;
impl ProgressStyle for GasDiffusion {
    fn name(&self) -> &str {
        "gas-diffusion"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Gas diffusion: molecules spread from a point source — cloud density is a Gaussian growing with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Diffusion sigma in dot units grows with eased
        let sigma_x = (ctx.eased * w as f32 * 0.5).max(0.5);
        let sigma_y = (ctx.eased * h as f32 * 0.5).max(0.5);

        // Source at left-centre
        let src_x = 0.0_f32;
        let src_y = (h as f32) / 2.0;

        // Animate: the centre of mass drifts rightward with time
        let drift = (ctx.time * 2.0).min(w as f32 * 0.5);
        let cx = src_x + drift;

        // Draw the Gaussian density cloud using dots
        // To avoid O(w*h) being too slow for tiny grids, iterate all dots
        for yi in 0..h {
            for xi in 0..w {
                let dx = xi as f32 - cx;
                let dy = yi as f32 - src_y;
                let exponent =
                    (dx * dx) / (2.0 * sigma_x * sigma_x) + (dy * dy) / (2.0 * sigma_y * sigma_y);
                let density = (-exponent).exp(); // [0, 1]

                // Threshold density against a hash value for stipple effect
                let h_val = hash2(xi, yi) as f32 / u32::MAX as f32;
                if density > h_val * 0.9 + 0.05 {
                    draw::dot(grid, xi, yi);
                }
            }
        }

        // Shade the spreading region via tint — colour gradient from source
        let (cw, ch) = grid.dimensions();
        let spread_cells = (ctx.eased * cw as f32).round() as usize;
        for cx2 in 0..spread_cells.min(cw) {
            let t = cx2 as f32 / cw as f32;
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Distillation
//    A distillation column: vapour (dots) rises on the left half, condenses
//    at the top, and drips down the right side into a receiver flask that
//    fills with eased progress. Vapour density animated by time; receiver
//    fill drawn as a rising fill_rect.
// ---------------------------------------------------------------------------
struct Distillation;
impl ProgressStyle for Distillation {
    fn name(&self) -> &str {
        "distillation"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Distillation: vapor rises in a column, condenses at the top, drips down into a filling receiver"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid_x = w / 2;

        // --- Column walls ---
        // Left (boiler) and right (receiver) vertical lines
        if mid_x > 0 {
            draw::vline(grid, 0, 0, h - 1);
        }
        draw::vline(grid, mid_x, 0, h - 1);
        if w > 1 {
            draw::vline(grid, w - 1, 0, h - 1);
        }
        // Condenser at the top
        draw::hline(grid, 0, w - 1, 0);

        // --- Vapour rising in left column (animated dots) ---
        // Vapour particles rise from the bottom; position = time-based
        let n_vapour = (mid_x / 2).max(1);
        for vi in 0..n_vapour {
            let speed = 0.7 + (vi as f32 * 0.13).sin().abs() * 0.4;
            let phase = vi as f32 * 0.6;
            // y position: starts at bottom, rises to top, loops
            let y_frac = 1.0 - ((ctx.time * speed + phase).rem_euclid(1.8) / 1.8);
            let y = (y_frac * (h - 1) as f32).round() as usize;
            // x: slight lateral wobble inside left column
            let x_wobble = (((ctx.time * 1.1 + vi as f32 * 0.4) * PI).sin() * 1.5) as i32;
            let vx = (mid_x / 4) as i32 + x_wobble;
            draw::dot_i(grid, vx.max(1).min(mid_x as i32 - 1), y as i32);
        }

        // --- Drip line on the right of the column ---
        // One drop falling per period, animated by time
        let drip_period = 0.6_f32;
        let drip_y = ((ctx.time.rem_euclid(drip_period) / drip_period) * h as f32).round() as usize;
        let drip_x = mid_x + (w - mid_x) / 2;
        if drip_x < w && drip_y < h {
            draw::dot(grid, drip_x.min(w - 2), drip_y);
        }

        // --- Receiver fill: liquid rising from bottom, height = eased ---
        let receiver_x0 = mid_x + 1;
        let receiver_w = w.saturating_sub(receiver_x0 + 1);
        if receiver_w > 0 {
            let fill_h = (ctx.eased * (h - 2) as f32).round() as usize;
            if fill_h > 0 {
                let fill_y0 = (h - 1).saturating_sub(fill_h);
                draw::fill_rect(grid, receiver_x0, fill_y0, receiver_w, fill_h);
            }
        }

        // Tint the receiver fill with palette
        let (cw, ch) = grid.dimensions();
        let recv_cx0 = cw / 2 + 1;
        let recv_filled = (ctx.eased * ch as f32).round() as usize;
        for cy in (ch.saturating_sub(recv_filled))..ch {
            let t = 1.0 - cy as f32 / ch as f32;
            draw::tint_row(
                grid,
                cy,
                recv_cx0,
                cw.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Bond vibration
//    Two atoms (solid dots) sit at fixed centres; a spring-bond connects them.
//    The internuclear distance oscillates: d(t) = d₀ + A·cos(ω·t).
//    Bond length = progress-controlled d₀; amplitude A fixed.
//    The spring is drawn as a zigzag line between the atoms.
// ---------------------------------------------------------------------------
struct BondVibration;
impl ProgressStyle for BondVibration {
    fn name(&self) -> &str {
        "bond-vibration"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Bond vibration: two atoms oscillate on a spring — frequency and bond length vary with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid_y = (h / 2) as i32;

        // Equilibrium bond length: progress controls separation
        let d0 = ctx.eased * (w as f32 * 0.6) + w as f32 * 0.1;
        // Vibrational amplitude
        let amp = (h as f32 * 0.15).max(1.0);
        // Vibrational frequency — higher at small d0 (Morse-like: stiffer at compressed bond)
        let omega = 4.0 * PI * (1.0 + (1.0 - ctx.eased) * 2.0);
        // Instantaneous displacement
        let disp = amp * (omega * ctx.time).cos();

        let cx = (w / 2) as f32;
        // Atom A: left, Atom B: right — each displaced ±disp/2 from cx ± d0/2
        let ax = (cx - d0 / 2.0 - disp / 2.0).round() as i32;
        let bx = (cx + d0 / 2.0 + disp / 2.0).round() as i32;

        // --- Draw atoms as small filled circles (3×3 dot clusters) ---
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx * dx + dy * dy <= 1 {
                    draw::dot_i(grid, ax + dx, mid_y + dy);
                    draw::dot_i(grid, bx + dx, mid_y + dy);
                }
            }
        }

        // --- Draw zigzag spring between atoms ---
        let left_x = ax.max(0).min(bx);
        let right_x = bx.max(0).max(ax);
        let bond_len = (right_x - left_x).max(1);
        let n_teeth = ((bond_len / 3).max(2) as usize).min(20);
        let tooth_h = (h as i32 / 4).max(1);

        for ti in 0..n_teeth {
            let x0 = left_x + (ti as i32 * bond_len) / n_teeth as i32;
            let x1 = left_x + ((ti + 1) as i32 * bond_len) / n_teeth as i32;
            let y0 = mid_y + if ti % 2 == 0 { -tooth_h } else { tooth_h };
            let y1 = mid_y + if ti % 2 == 0 { tooth_h } else { -tooth_h };
            // Draw a line from (x0,y0) to (x1,y1) via interpolation
            let steps = ((x1 - x0).abs() + (y1 - y0).abs()).max(1) as usize;
            for s in 0..=steps {
                let f = s as f32 / steps as f32;
                let ix = (x0 as f32 + (x1 - x0) as f32 * f).round() as i32;
                let iy = (y0 as f32 + (y1 - y0) as f32 * f).round() as i32;
                draw::dot_i(grid, ix, iy);
            }
        }

        // Tint by eased
        let (cw, ch) = grid.dimensions();
        for cx2 in 0..cw {
            let t = cx2 as f32 / cw as f32;
            for cy2 in 0..ch {
                draw::tint_row(grid, cy2, cx2, cx2, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Boltzmann distribution
//     Velocity histogram filling up to the Maxwell-Boltzmann shape.
//     f(v) ∝ v²·exp(−v²/(2kT)) — temperature T = 1 + eased·3.
//     Each column (velocity bin) is a vblock glyph filled to the MB height;
//     eased controls how "hot" the distribution is (broader + shifted peak).
//     time animates a slight shimmer on bar heights.
// ---------------------------------------------------------------------------
struct BoltzmannDistribution;
impl ProgressStyle for BoltzmannDistribution {
    fn name(&self) -> &str {
        "boltzmann-distribution"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Maxwell-Boltzmann: velocity histogram builds up — temperature rises with progress, broadening the peak"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Temperature: low at start (sharp tall peak), high at end (broad flat)
        let temp = 0.3 + ctx.eased * 2.7; // T in [0.3, 3.0]

        // Maxwell-Boltzmann: f(v) = C · v² · exp(−v²/(2T))
        // v ranges from 0 to v_max; we pick v_max = 4·sqrt(T) for full coverage
        let v_max = 4.0 * temp.sqrt();

        // Compute raw heights
        let mut heights: Vec<f32> = Vec::with_capacity(cw);
        let mut max_h = 0.0_f32;
        for col in 0..cw {
            let v = (col as f32 + 0.5) / cw as f32 * v_max;
            let fv = v * v * (-(v * v) / (2.0 * temp)).exp();
            heights.push(fv);
            if fv > max_h {
                max_h = fv;
            }
        }

        // Animate: slight shimmer on bar heights (time-driven ripple)
        for (col, h_val) in heights.iter_mut().enumerate() {
            let ripple = 1.0 + 0.05 * (ctx.time * 3.0 + col as f32 * 0.3).sin();
            *h_val *= ripple;
        }

        // Re-check max after shimmer
        let max_h = heights.iter().copied().fold(0.0_f32, f32::max).max(0.001);

        // Draw each column using vblock glyphs from the bottom up
        for (col, &height) in heights.iter().enumerate() {
            let norm = height / max_h; // [0,1]
                                       // Each column spans ch cells vertically; fill bottom-up
            let total_eighths = (norm * ch as f32 * 8.0).round() as usize;
            let full_cells = total_eighths / 8;
            let partial = total_eighths % 8;

            // Full cells: filled from bottom
            for row in 0..full_cells.min(ch) {
                let cy = ch - 1 - row;
                draw::vblock(grid, col, cy, 8);
            }
            // Partial cell on top
            if partial > 0 && full_cells < ch {
                let cy = ch - 1 - full_cells;
                draw::vblock(grid, col, cy, partial);
            }

            // Colour: hot bins (near peak) get end-palette colour
            let t = height / max_h;
            for cy in 0..ch {
                draw::tint_row(grid, cy, col, col, ctx.palette.sample(t));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Flame test / spectral lines
//     Discrete emission lines appear left-to-right as progress advances.
//     Each line is a narrow bright vertical column of dots at a specific
//     wavelength position. time drives a flicker in intensity (flame effect).
//     Lines are drawn as vlines of varying height (intensity); some are
//     doublets (two adjacent dots wide).
// ---------------------------------------------------------------------------
struct FlameSpectralLines;
impl ProgressStyle for FlameSpectralLines {
    fn name(&self) -> &str {
        "flame-spectral-lines"
    }
    fn theme(&self) -> &str {
        "chemistry"
    }
    fn describe(&self) -> &str {
        "Flame test: discrete emission lines flicker into existence as progress reveals each spectral line"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Emission line wavelength positions (as fractions of bar width)
        // Based loosely on real alkali-metal / hydrogen lines:
        // H: 656nm(red), 486nm(cyan), 434nm(violet), 410nm(deep violet)
        // Na: 589nm(yellow doublet)
        // K:  766nm(IR), 404nm(violet)
        // Ca: 423nm, 442nm, 616nm, 646nm
        let lines: &[(f32, f32, bool)] = &[
            // (position_frac, relative_intensity, is_doublet)
            (0.08, 0.60, false), // K 766 nm (deep red)
            (0.20, 0.95, false), // H 656 nm (red Hα — strong)
            (0.38, 0.60, true),  // Na 589 nm (yellow doublet)
            (0.44, 0.45, false), // Ca 558 nm
            (0.52, 0.75, false), // H 486 nm (cyan Hβ)
            (0.60, 0.50, false), // Ca 442 nm
            (0.65, 0.40, false), // Ca 423 nm
            (0.72, 0.65, false), // H 434 nm (violet Hγ)
            (0.80, 0.35, false), // K  404 nm
            (0.88, 0.50, false), // H 410 nm (deep violet Hδ)
            (0.94, 0.30, false), // Ca 393 nm (UV edge)
        ];

        let n_lines = lines.len();

        // Dark background: draw a thin horizontal baseline
        draw::hline(grid, 0, w - 1, h - 1);

        // How many lines are visible: eased × n_lines
        let visible = (ctx.eased * n_lines as f32).round() as usize;

        for (li, &(pos_frac, intensity, doublet)) in lines.iter().enumerate() {
            if li >= visible {
                break;
            }

            // Flicker: multiply intensity by a time-driven noise factor
            let flicker = 0.7 + 0.3 * (ctx.time * (3.0 + li as f32 * 0.7)).sin().abs();
            let final_intensity = (intensity * flicker).min(1.0);

            // Column position in dots
            let xd = (pos_frac * (w - 1) as f32).round() as usize;

            // Height of the line proportional to intensity
            let line_h = (final_intensity * (h - 2) as f32).round() as usize;
            let y0 = (h - 1).saturating_sub(line_h);

            // Draw the emission line
            draw::vline(grid, xd.min(w - 1), y0, h - 2);

            // Doublet: draw an adjacent column
            if doublet && xd + 1 < w {
                let doublet_h = (line_h as f32 * 0.85).round() as usize;
                let dy0 = (h - 1).saturating_sub(doublet_h);
                draw::vline(grid, xd + 1, dy0, h - 2);
            }

            // Tint each line with a colour from the palette keyed to its position
            let (cw, ch) = grid.dimensions();
            let cx = (pos_frac * (cw - 1) as f32).round() as usize;
            let col = ctx.palette.sample(pos_frac);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx.min(cw - 1), cx.min(cw - 1), col);
            }
            if doublet {
                let cx2 = (cx + 1).min(cw - 1);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, cx2, cx2, ctx.palette.sample(pos_frac + 0.01));
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
    let styles = progress::styles::chemistry::styles();
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
