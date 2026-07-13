//! `border` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O border.rs && ./border [style-name]
//! ```

const DEFAULT_STYLE: &str = "draw-on";

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
    pub mod border {
//! Animated border / frame progress styles.
//!
//! Every style in this theme draws progress as motion **around the perimeter**
//! of the grid region rather than as a horizontal fill. The twelve styles cover
//! fundamentally different visual strategies: sequential draw-on, oscillating
//! dashes, growing corner brackets, a chasing comet, nested frames, pulsing
//! glow, distinct dash patterns, chamfered corners, inward thickening, a ruler
//! with a marker, a window title strip, and two opposing runners.
//!
//! # Perimeter helpers
//!
//! All styles share a common concept: the perimeter is a 1-D sequence of dot
//! positions walked clockwise starting from the top-left corner. The helper
//! `perim_point(i, pw, ph)` converts a perimeter index to `(x, y)` in dot
//! space. `perim_len(pw, ph)` gives the total count.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── Perimeter helpers ────────────────────────────────────────────────────────

/// Total perimeter length in dots for a rectangle of dot-size `(pw, ph)`.
/// Minimum 1 so we never divide by zero.
#[inline]
fn perim_len(pw: usize, ph: usize) -> usize {
    if pw == 0 || ph == 0 {
        return 1;
    }
    if pw == 1 && ph == 1 {
        return 1;
    }
    if pw == 1 {
        return ph;
    }
    if ph == 1 {
        return pw;
    }
    2 * (pw - 1) + 2 * (ph - 1)
}

/// Map a perimeter index `i` (0-based, clockwise from top-left) to `(x, y)`
/// in dot space. Wraps modulo the perimeter length.
#[inline]
fn perim_point(i: usize, pw: usize, ph: usize) -> (usize, usize) {
    if pw == 0 || ph == 0 {
        return (0, 0);
    }
    if pw == 1 && ph == 1 {
        return (0, 0);
    }
    if pw == 1 {
        let i = i % ph;
        return (0, i);
    }
    if ph == 1 {
        let i = i % pw;
        return (i, 0);
    }
    let p = perim_len(pw, ph);
    let i = i % p;
    // top edge: left→right
    if i < pw {
        return (i, 0);
    }
    let i = i - (pw - 1);
    // right edge: top→bottom (skip top-right corner already counted)
    if i < ph {
        return (pw - 1, i);
    }
    let i = i - (ph - 1);
    // bottom edge: right→left
    if i < pw {
        return (pw - 1 - i, ph - 1);
    }
    let i = i - (pw - 1);
    // left edge: bottom→top (skip corners already counted)
    let i = i % (ph - 1).max(1); // guard against degenerate ph
    (0, ph - 1 - i)
}

// ── Public registry ──────────────────────────────────────────────────────────

/// All styles in the `border` theme.
///
/// Returns 12 structurally distinct styles, each communicating progress or
/// time through perimeter motion, geometry, or density — never only colour.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(DrawOn),
        Box::new(MarchingAnts),
        Box::new(CornerBrackets),
        Box::new(Runner),
        Box::new(DoubleFrame),
        Box::new(NeonPulse),
        Box::new(DashedBorder),
        Box::new(DottedBorder),
        Box::new(RoundedBorder),
        Box::new(FillFrame),
        Box::new(TickedFrame),
        Box::new(TwoRunners),
    ]
}

// ── Style 1: DrawOn ──────────────────────────────────────────────────────────

/// The border draws itself clockwise from the top-left; `eased` sets how far
/// around the perimeter the stroke has reached. At 1.0 the frame is complete.
struct DrawOn;
impl ProgressStyle for DrawOn {
    fn name(&self) -> &str {
        "draw-on"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Border draws clockwise from top-left; perimeter = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        let p = perim_len(pw, ph);
        let lit = (ctx.eased * p as f32).round() as usize;
        for i in 0..lit.min(p) {
            let (x, y) = perim_point(i, pw, ph);
            draw::dot(grid, x, y);
        }
        // Tint the lit portion with the palette gradient.
        let (cw, ch) = grid.dimensions();
        if lit > 0 {
            for cy in 0..ch {
                let t = cy as f32 / ch.max(1) as f32;
                draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Style 2: MarchingAnts ────────────────────────────────────────────────────

/// A dashed border whose dash pattern scrolls around the perimeter over time.
/// Dash length and gap are fixed; `time` drives the scroll offset.
struct MarchingAnts;
impl ProgressStyle for MarchingAnts {
    fn name(&self) -> &str {
        "marching-ants"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Dashes scroll clockwise around the frame; speed increases with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        let p = perim_len(pw, ph);
        let dash = 6usize;
        let gap = 4usize;
        let period = (dash + gap) as f32;
        // Speed scales up with progress so faster near completion.
        let speed = 8.0 + ctx.progress * 24.0;
        let offset = (ctx.time * speed) as usize;
        for i in 0..p {
            let phase = (i + offset) % (dash + gap);
            if phase < dash {
                let (x, y) = perim_point(i, pw, ph);
                draw::dot(grid, x, y);
            }
        }
        // Tint the whole border in start colour.
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(0.3);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        let _ = period; // suppress lint
        Ok(())
    }
}

// ── Style 3: CornerBrackets ──────────────────────────────────────────────────

/// Four L-shaped corner brackets grow inward along each edge proportionally
/// to `eased`. At 1.0 they meet in the middle of every edge.
struct CornerBrackets;
impl ProgressStyle for CornerBrackets {
    fn name(&self) -> &str {
        "corner-brackets"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Corner brackets extend toward midpoints of each edge as progress fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        if pw == 0 || ph == 0 {
            return Ok(());
        }
        let hreach = ((ctx.eased * (pw / 2) as f32).round() as usize).min(pw / 2);
        let vreach = ((ctx.eased * (ph / 2) as f32).round() as usize).min(ph / 2);
        let x1 = pw.saturating_sub(1);
        let y1 = ph.saturating_sub(1);
        // Top-left
        draw::hline(grid, 0, hreach.min(x1), 0);
        draw::vline(grid, 0, 0, vreach.min(y1));
        // Top-right
        draw::hline(grid, x1.saturating_sub(hreach), x1, 0);
        draw::vline(grid, x1, 0, vreach.min(y1));
        // Bottom-left
        draw::hline(grid, 0, hreach.min(x1), y1);
        draw::vline(grid, 0, y1.saturating_sub(vreach), y1);
        // Bottom-right
        draw::hline(grid, x1.saturating_sub(hreach), x1, y1);
        draw::vline(grid, x1, y1.saturating_sub(vreach), y1);
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(ctx.eased);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        Ok(())
    }
}

// ── Style 4: Runner ──────────────────────────────────────────────────────────

/// A single bright comet chases the perimeter with `time`; the tail fades
/// using dot density. A secondary dim outline shows the full frame.
struct Runner;
impl ProgressStyle for Runner {
    fn name(&self) -> &str {
        "runner"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Bright comet races around the full frame perimeter with a fading tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        let p = perim_len(pw, ph);
        // Dim outline — every other dot so it reads as a ghost frame.
        for i in (0..p).step_by(2) {
            let (x, y) = perim_point(i, pw, ph);
            draw::dot(grid, x, y);
        }
        // Completed laps: the track solidifies clockwise from the top-left
        // origin as progress advances, giving a monochrome progress read.
        let done = ((ctx.eased * p as f32) as usize).min(p);
        for i in 0..done {
            let (x, y) = perim_point(i, pw, ph);
            draw::dot(grid, x, y);
        }
        // Comet head position driven by time.
        let speed = 0.5; // laps per second
        let head = ((ctx.time * speed).fract() * p as f32) as usize;
        let tail = (p / 6).max(3);
        for k in 0..tail {
            let idx = (head + p - k) % p;
            let (x, y) = perim_point(idx, pw, ph);
            // Tail thins: draw every dot near head, every other farther out.
            if k == 0 || k % (1 + k / (tail / 3).max(1)) == 0 {
                draw::dot(grid, x, y);
            }
        }
        // Tint comet with palette end colour.
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(1.0);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        Ok(())
    }
}

// ── Style 5: DoubleFrame ─────────────────────────────────────────────────────

/// Two nested rectangle outlines. The outer is always drawn; the inner
/// animates with `eased` — it begins as a tiny dot cluster and expands
/// outward until it sits 2 dots inside the outer frame.
struct DoubleFrame;
impl ProgressStyle for DoubleFrame {
    fn name(&self) -> &str {
        "double-frame"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Outer frame always present; inner frame expands from centre with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        // Outer frame.
        draw::rect_outline(grid, 0, 0, pw, ph);
        // Inner frame: starts at centre, grows to 2 dots inside outer.
        let max_inset = 2usize;
        let inset = max_inset
            + (((1.0 - ctx.eased) * (pw.min(ph) / 2).saturating_sub(max_inset) as f32).round()
                as usize);
        let inset = inset.min(pw / 2).min(ph / 2);
        let ix0 = inset;
        let iy0 = inset;
        let iw = pw.saturating_sub(inset * 2);
        let ih = ph.saturating_sub(inset * 2);
        if iw >= 2 && ih >= 2 {
            draw::rect_outline(grid, ix0, iy0, iw, ih);
        }
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ── Style 6: NeonPulse ───────────────────────────────────────────────────────

/// The frame pulses in apparent brightness by varying the dot density of a
/// two-pixel-wide border band. More progress → base brightness; time drives
/// the oscillation on top.
struct NeonPulse;
impl ProgressStyle for NeonPulse {
    fn name(&self) -> &str {
        "neon-pulse"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Frame border pulses in dot-density (glow simulation); brighter with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        // Pulse: base brightness from progress, oscillation from time.
        let osc = (ctx.time * 2.5 * PI).sin() * 0.2;
        let density = (ctx.eased * 0.7 + 0.3 + osc).clamp(0.0, 1.0);
        let p = perim_len(pw, ph);
        // Outer ring.
        for i in 0..p {
            let (x, y) = perim_point(i, pw, ph);
            // Probabilistic density: use i modulo to approximate fraction.
            let period = (1.0 / density.max(0.01)).round() as usize;
            let period = period.max(1);
            if i % period == 0 {
                draw::dot(grid, x, y);
            }
        }
        // Second ring (1 dot inset) with lower density for glow halo.
        let inner_density = (density * 0.55).clamp(0.0, 1.0);
        if pw >= 4 && ph >= 4 {
            let ip = perim_len(pw - 2, ph - 2);
            let period2 = ((1.0 / inner_density.max(0.01)).round() as usize).max(1);
            for i in 0..ip {
                if i % period2 == 0 {
                    let (x, y) = perim_point(i, pw - 2, ph - 2);
                    draw::dot(grid, x + 1, y + 1);
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(ctx.eased);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        Ok(())
    }
}

// ── Style 7: DashedBorder ────────────────────────────────────────────────────

/// A static dashed frame (long dash, long gap – 3:2 ratio). `eased` controls
/// how many of the dashes are lit, walking clockwise — structurally a
/// draw-on of a dash pattern rather than a solid stroke.
struct DashedBorder;
impl ProgressStyle for DashedBorder {
    fn name(&self) -> &str {
        "dashed"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Dashed frame border; lit dashes accumulate clockwise with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        let p = perim_len(pw, ph);
        let dash = 5usize;
        let gap = 3usize;
        let period = dash + gap;
        let lit_perimeter = (ctx.eased * p as f32).round() as usize;
        for i in 0..lit_perimeter.min(p) {
            if i % period < dash {
                let (x, y) = perim_point(i, pw, ph);
                draw::dot(grid, x, y);
            }
        }
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(ctx.eased);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        Ok(())
    }
}

// ── Style 8: DottedBorder ────────────────────────────────────────────────────

/// A dotted (1:2 ratio – single dot, double gap) frame. Progress fills
/// clockwise. Visually sparser and more delicate than `dashed`.
struct DottedBorder;
impl ProgressStyle for DottedBorder {
    fn name(&self) -> &str {
        "dotted"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Sparse dotted frame (1:2 on/off); lit dots accumulate with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        let p = perim_len(pw, ph);
        let period = 3usize; // 1 dot, 2 gap
        let lit_perimeter = (ctx.eased * p as f32).round() as usize;
        for i in 0..lit_perimeter.min(p) {
            if i % period == 0 {
                let (x, y) = perim_point(i, pw, ph);
                draw::dot(grid, x, y);
            }
        }
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(0.6);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        Ok(())
    }
}

// ── Style 9: RoundedBorder ───────────────────────────────────────────────────

/// A frame with arc-approximated rounded corners: the four corner regions use
/// diagonal dot clusters to soften the right angles. The straight edges fill
/// with `eased` clockwise.
struct RoundedBorder;
impl ProgressStyle for RoundedBorder {
    fn name(&self) -> &str {
        "rounded"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Chamfered/rounded corners via arc dots; straight edges fill with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        if pw == 0 || ph == 0 {
            return Ok(());
        }
        let x1 = pw.saturating_sub(1);
        let y1 = ph.saturating_sub(1);
        // Corner arc radius in dots (capped so it doesn't consume the whole edge).
        let r = 3usize.min(pw / 4).min(ph / 4).max(1);
        // ── Straight edges (filled with eased) ──────────────────────────────
        // Top & bottom edges between corners.
        let top_x0 = r;
        let top_x1 = x1.saturating_sub(r);
        let bot_x0 = r;
        let bot_x1 = x1.saturating_sub(r);
        // Edge lengths.
        let h_len = top_x1.saturating_sub(top_x0) + 1;
        let v_len = y1.saturating_sub(r).saturating_sub(r) + 1;
        let total_edge = 2 * h_len + 2 * v_len;
        let lit_edge = (ctx.eased * total_edge as f32).round() as usize;
        // Clockwise: top, right, bottom, left.
        let mut rem = lit_edge;
        // Top
        if rem > 0 && top_x0 <= top_x1 {
            let seg = rem.min(h_len);
            draw::hline(grid, top_x0, top_x0 + seg.saturating_sub(1), 0);
            rem = rem.saturating_sub(seg);
        }
        // Right
        if rem > 0 {
            let r_y0 = r;
            let r_y1 = y1.saturating_sub(r);
            let seg = rem.min(v_len);
            if r_y0 <= r_y1 {
                draw::vline(grid, x1, r_y0, r_y0 + seg.saturating_sub(1));
            }
            rem = rem.saturating_sub(seg);
        }
        // Bottom
        if rem > 0 && bot_x0 <= bot_x1 {
            let seg = rem.min(h_len);
            draw::hline(
                grid,
                bot_x1.saturating_sub(seg.saturating_sub(1)),
                bot_x1,
                y1,
            );
            rem = rem.saturating_sub(seg);
        }
        // Left
        if rem > 0 {
            let l_y0 = r;
            let l_y1 = y1.saturating_sub(r);
            let seg = rem.min(v_len);
            if l_y0 <= l_y1 {
                draw::vline(grid, 0, l_y1.saturating_sub(seg.saturating_sub(1)), l_y1);
            }
        }
        // ── Corner arcs (always drawn) ───────────────────────────────────────
        // Approximate a quarter-circle in braille dots for each corner.
        let steps = (r * 4).max(4);
        for s in 0..steps {
            let angle = (s as f32 / steps as f32) * PI / 2.0;
            let ax = (angle.cos() * r as f32).round() as usize;
            let ay = (angle.sin() * r as f32).round() as usize;
            // Top-left (quarter from 180°→90°): arc maps to (r-ax, r-ay).
            draw::dot(grid, r.saturating_sub(ax), r.saturating_sub(ay));
            // Top-right: (x1 - r + ax, r - ay)
            if x1 >= r.saturating_sub(ax) {
                draw::dot(
                    grid,
                    x1.saturating_sub(r).saturating_add(ax).min(x1),
                    r.saturating_sub(ay),
                );
            }
            // Bottom-left: (r - ax, y1 - r + ay)
            draw::dot(
                grid,
                r.saturating_sub(ax),
                y1.saturating_sub(r).saturating_add(ay).min(y1),
            );
            // Bottom-right: (x1 - r + ax, y1 - r + ay)
            draw::dot(
                grid,
                x1.saturating_sub(r).saturating_add(ax).min(x1),
                y1.saturating_sub(r).saturating_add(ay).min(y1),
            );
        }
        let (cw, ch) = grid.dimensions();
        let c = ctx.palette.sample(ctx.eased);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), c);
        }
        Ok(())
    }
}

// ── Style 10: FillFrame ──────────────────────────────────────────────────────

/// The frame thickens inward proportionally to `eased`. At 0 it is a hairline;
/// at 1.0 it fills the entire grid. The result is a frame-shaped progress meter.
struct FillFrame;
impl ProgressStyle for FillFrame {
    fn name(&self) -> &str {
        "fill-frame"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Frame thickens inward with eased — a border-shaped progress meter"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        if pw == 0 || ph == 0 {
            return Ok(());
        }
        // Maximum thickness is half the shorter dimension.
        let max_thick = (pw.min(ph) / 2).max(1);
        let thick = (ctx.eased * max_thick as f32).round() as usize;
        let thick = thick.max(1);
        // Draw concentric outlines from outermost inward to `thick`.
        for t in 0..thick {
            let x0 = t;
            let y0 = t;
            let w = pw.saturating_sub(t * 2);
            let h = ph.saturating_sub(t * 2);
            if w < 1 || h < 1 {
                break;
            }
            draw::rect_outline(grid, x0, y0, w, h);
        }
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ── Style 11: TickedFrame ────────────────────────────────────────────────────

/// The top edge is a ruler with tick marks at every 10% of its length. A
/// traveling marker (solid vline pip) moves left-to-right with `eased`, and
/// the remaining three sides form a plain frame.
struct TickedFrame;
impl ProgressStyle for TickedFrame {
    fn name(&self) -> &str {
        "ticked-frame"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Ruler ticks along the top edge with a traveling marker; plain frame elsewhere"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        if pw == 0 || ph == 0 {
            return Ok(());
        }
        let x1 = pw.saturating_sub(1);
        let y1 = ph.saturating_sub(1);
        // Three sides (not top).
        draw::vline(grid, 0, 0, y1);
        draw::vline(grid, x1, 0, y1);
        draw::hline(grid, 0, x1, y1);
        // Top ruler line.
        draw::hline(grid, 0, x1, 0);
        // Tick marks: short ticks at 10% intervals, longer at 0/50/100.
        for i in 0..=10 {
            let x = (i as f32 / 10.0 * x1 as f32).round() as usize;
            let tick_h = if i == 0 || i == 5 || i == 10 {
                (ph / 3).max(1)
            } else {
                (ph / 6).max(1)
            };
            draw::vline(grid, x, 0, tick_h.min(y1));
        }
        // Traveling marker: a full-height vline at the progress position.
        let marker_x = (ctx.eased * x1 as f32).round() as usize;
        draw::vline(grid, marker_x.min(x1), 0, y1);
        // Tint marker column with end color, rest with start.
        let (cw, ch) = grid.dimensions();
        let base_c = ctx.palette.sample(0.2);
        let marker_c = ctx.palette.sample(1.0);
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), base_c);
        }
        // Marker cell column (cell x = marker_x / 2).
        let marker_cx = marker_x / 2;
        for cy in 0..ch {
            draw::tint_row(grid, cy, marker_cx, marker_cx, marker_c);
        }
        Ok(())
    }
}

// ── Style 12: TwoRunners ─────────────────────────────────────────────────────

/// Two bright segments race in opposite directions around the perimeter.
/// They start at top-left, one going clockwise, one counter-clockwise. They
/// meet at the point corresponding to `eased * P/2`. A dim full outline
/// shows the frame.
struct TwoRunners;
impl ProgressStyle for TwoRunners {
    fn name(&self) -> &str {
        "two-runners"
    }
    fn theme(&self) -> &str {
        "border"
    }
    fn describe(&self) -> &str {
        "Two comet segments race opposite ways; they meet at progress * half-perimeter"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (pw, ph) = draw::dot_dims(grid);
        let p = perim_len(pw, ph);
        // Ghost outline — every 3rd dot.
        for i in (0..p).step_by(3) {
            let (x, y) = perim_point(i, pw, ph);
            draw::dot(grid, x, y);
        }
        // Meeting point: each runner travels eased * p/2 from the origin.
        let half = p / 2;
        let reach = (ctx.eased * half as f32).round() as usize;
        let tail = (half / 5).max(2);
        // Clockwise runner.
        for k in 0..reach.min(half) {
            let idx = k % p;
            let (x, y) = perim_point(idx, pw, ph);
            draw::dot(grid, x, y);
        }
        // Counter-clockwise runner (start at same origin, go backwards).
        for k in 0..reach.min(half) {
            let idx = (p - (k % p)) % p;
            let (x, y) = perim_point(idx, pw, ph);
            draw::dot(grid, x, y);
        }
        // Animate time-based comet shimmer on the leading edges.
        let head_cw = reach % p;
        let head_ccw = (p - reach % p) % p;
        let shimmer_tail = tail;
        for k in 0..shimmer_tail {
            let step = (ctx.time * 8.0) as usize;
            if (k + step) % 2 == 0 {
                let idx_cw = (head_cw + p - k) % p;
                let idx_ccw = (head_ccw + k) % p;
                let (x, y) = perim_point(idx_cw, pw, ph);
                draw::dot(grid, x, y);
                let (x2, y2) = perim_point(idx_ccw, pw, ph);
                draw::dot(grid, x2, y2);
            }
        }
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            let t = cy as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), ctx.palette.sample(t));
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
    let styles = progress::styles::border::styles();
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
