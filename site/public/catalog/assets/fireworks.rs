//! `fireworks` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O fireworks.rs && ./fireworks [style-name]
//! ```

const DEFAULT_STYLE: &str = "launch";

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
    pub mod fireworks {
//! Fireworks progress bars — launches, bursts, sparklers, grand finales.
//!
//! Progress reads as a show building: rockets go up one per ten percent,
//! bursts bloom wider, salvos thicken, and everything pays off at one
//! hundred percent. Colors are a night-show palette — gold, coral, cyan,
//! violet — around white-hot flashes. Deterministic in `(progress, time)`.

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

// ─── theme colors — night show ──────────────────────────────────────────────

/// Gold, the classic shell.
const FW_GOLD: Color = Color::rgb(255, 203, 94);
/// Coral red.
const FW_CORAL: Color = Color::rgb(240, 110, 90);
/// Electric cyan.
const FW_CYAN: Color = Color::rgb(92, 220, 240);
/// Violet.
const FW_VIOLET: Color = Color::rgb(176, 140, 250);
/// White-hot flash.
const FW_FLASH: Color = Color::rgb(255, 248, 231);

/// Pick a show color for particle index `i`.
fn fw_color(i: i32) -> Color {
    match (hash2(i, 999) * 4.0) as u32 {
        0 => FW_GOLD,
        1 => FW_CORAL,
        2 => FW_CYAN,
        _ => FW_VIOLET,
    }
}

/// All styles in the `fireworks` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Launch),
        Box::new(Peony),
        Box::new(Chrysanthemum),
        Box::new(Crackle),
        Box::new(RomanCandle),
        Box::new(SparklerFill),
        Box::new(Salvo),
        Box::new(Willow),
        Box::new(StrobePop),
        Box::new(GrandFinale),
    ]
}

/// Rockets launch one per ten percent, each leaving a star at apex.
struct Launch;
impl ProgressStyle for Launch {
    fn name(&self) -> &str {
        "launch"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Rockets launching one per ten percent"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let count = 10usize;
        // Ground line for composition.
        for x in (0..w).step_by(2) {
            draw::dot(grid, x, h - 1);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, Color::rgb(96, 92, 104));
        }
        let apex = 2usize;
        let span = (h - 1 - apex) as f32;
        for k in 0..count {
            let x = ((k as f32 + 0.5) / count as f32 * w as f32) as usize;
            if x >= w {
                continue;
            }
            let window = (ctx.eased * count as f32) - k as f32;
            if window >= 1.0 {
                // Arrived: a twinkling star at apex.
                if hash3(k as i32, 5, (ctx.time * 4.0) as i32) > 0.25 {
                    draw::dot(grid, x, apex);
                    draw::dot(grid, x.saturating_sub(1), apex + 1);
                    draw::dot(grid, (x + 1).min(w - 1), apex + 1);
                    let _ = grid.set_cell_color(x / 2, apex / 4, fw_color(k as i32));
                }
            } else if window > 0.0 {
                // In flight: rocket climbs with a short flame trail.
                let y = (h as f32 - 1.0 - window * span) as usize;
                draw::dot(grid, x, y);
                for t in 1..4usize {
                    let ty = y + t;
                    if ty < h && hash3(k as i32, t as i32, (ctx.time * 12.0) as i32) > 0.3 {
                        draw::dot(grid, x, ty);
                    }
                }
                let _ = grid.set_cell_color(x / 2, y / 4, FW_FLASH);
            }
        }
        Ok(())
    }
}

/// One great shell blooms from the center as progress grows.
struct Peony;
impl ProgressStyle for Peony {
    fn name(&self) -> &str {
        "peony"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A shell blooming wider with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let max_r = w as f32 / 2.0 - 1.0;
        let r = ctx.eased * max_r;
        let rays = 18;
        for ray in 0..rays {
            let ang = ray as f32 * TAU / rays as f32 + ctx.time * TAU * 0.25 * 0.2;
            let color = fw_color(ray);
            // Dots along the ray, denser near the tip.
            let steps = (r / 2.0) as i32;
            for s in 0..=steps {
                let dist = r * (s as f32 / steps.max(1) as f32);
                if hash3(ray, s, 0) < 0.35 && s < steps - 1 {
                    continue;
                }
                let px = cx + ang.cos() * dist;
                let py = cy + ang.sin() * dist * 0.45; // squash to the bar
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if s == steps { FW_FLASH } else { color };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        // Twinkle at the core.
        if hash3(0, 0, (ctx.time * 6.0) as i32) > 0.4 {
            draw::dot(grid, cx as usize, cy as usize);
            let _ = grid.set_cell_color(cx as usize / 2, cy as usize / 4, FW_FLASH);
        }
        Ok(())
    }
}

/// A double shell: a dense core disc inside a ring of falling petals.
struct Chrysanthemum;
impl ProgressStyle for Chrysanthemum {
    fn name(&self) -> &str {
        "chrysanthemum"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A double shell with drooping petals"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0;
        let cy = h as f32 * 0.4;
        let max_r = w as f32 / 2.0 - 1.0;
        let r = ctx.eased * max_r;
        // Core disc: dithered fill.
        let core = r * 0.35;
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx;
                let dy = (y as f32 - cy) * 2.2;
                if dx * dx + dy * dy <= core * core && hash2(x as i32, y as i32) < 0.6 {
                    draw::dot(grid, x, y);
                    let _ = grid.set_cell_color(x / 2, y / 4, FW_GOLD);
                }
            }
        }
        // Outer petals: ring points sagging with age.
        let petals = 22;
        for p in 0..petals {
            let ang = p as f32 * TAU / petals as f32;
            let sag = (r / max_r.max(1.0)).powi(2) * h as f32 * 0.3;
            let px = cx + ang.cos() * r;
            let py = cy + ang.sin() * r * 0.4 + sag;
            draw::dot_i(grid, px as i32, py as i32);
            // A short trailing spark above each petal.
            draw::dot_i(grid, px as i32, py as i32 - 2);
            if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, fw_color(p));
            }
        }
        Ok(())
    }
}

/// A steady bar that crackles with popping sparks as it fills.
struct Crackle;
impl ProgressStyle for Crackle {
    fn name(&self) -> &str {
        "crackle"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A bar crackling with popping sparks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let mid = h / 2;
        // The base bar: a chunky four-row band.
        for x in 0..filled {
            for dy in 0..4usize {
                draw::dot(grid, x, (mid + dy).saturating_sub(2).min(h - 1));
            }
            let _ = grid.set_cell_color(x / 2, mid / 4, FW_GOLD);
        }
        // Crackle pops above and below, densest near the leading edge.
        let slot = (ctx.time * 12.0) as i32;
        for x in 0..filled {
            let near = 1.0 - (filled - x) as f32 / w.max(1) as f32 * 1.8;
            if near <= 0.0 {
                continue;
            }
            if hash3(x as i32, 1, slot) < near * 0.7 {
                let up = 1 + (hash3(x as i32, 2, slot) * (mid as f32 - 2.0)) as usize;
                let y = if hash3(x as i32, 3, slot) > 0.5 {
                    mid.saturating_sub(2 + up)
                } else {
                    (mid + 2 + up).min(h - 1)
                };
                draw::dot(grid, x, y);
                if hash3(x as i32, 4, slot) > 0.5 {
                    draw::dot(grid, (x + 1).min(w - 1), y);
                }
                let _ = grid.set_cell_color(x / 2, y / 4, FW_FLASH);
            }
        }
        Ok(())
    }
}

/// A candle at the left lobs shots that burst at the progress point.
struct RomanCandle;
impl ProgressStyle for RomanCandle {
    fn name(&self) -> &str {
        "roman-candle"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Arcing shots bursting at the progress point"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        // The candle tube at the left edge.
        for y in (h * 2 / 3)..h {
            draw::dot(grid, 0, y);
            draw::dot(grid, 1.min(w - 1), y);
        }
        let _ = grid.set_cell_color(0, (h - 1) / 4, Color::rgb(120, 104, 86));
        let target = (ctx.eased * (w as f32 - 3.0)) + 2.0;
        // Milestone embers where earlier shots landed: little glow mounds.
        for k in 1..=10 {
            let mx = (k as f32 / 10.0 * (w as f32 - 3.0) + 2.0) as usize;
            if (mx as f32) < target && hash3(k, 8, (ctx.time * 4.0) as i32) > 0.25 {
                draw::dot(grid, mx, h - 1);
                draw::dot(grid, (mx + 1).min(w - 1), h - 1);
                draw::dot(grid, mx, h - 2);
                let _ = grid.set_cell_color(mx / 2, (h - 2) / 4, fw_color(k));
            }
        }
        // Target marker: a blinking tick where the next shot lands.
        if (ctx.time * 3.0) as i32 % 2 == 0 {
            let tx = (target as usize).min(w.saturating_sub(1));
            draw::vline(grid, tx, h.saturating_sub(4), h - 1);
            let _ = grid.set_cell_color(tx / 2, (h - 1) / 4, FW_FLASH);
        }
        // The shot in flight: a fat spark on a parabolic arc, with a trail.
        let phase = (ctx.time * 0.75).fract();
        let sx = 1.0 + phase * (target - 1.0);
        let ft = sx / w.max(1) as f32;
        let arc = (phase * (1.0 - phase)) * 4.0; // 0..1 peak mid-flight
        let sy = (h as f32 - 2.0) - arc * (h as f32 - 4.0) * (0.5 + ft * 0.5);
        for k in 0..4i32 {
            let tp = (phase - k as f32 * 0.03).max(0.0);
            let tx = 1.0 + tp * (target - 1.0);
            let tarc = (tp * (1.0 - tp)) * 4.0;
            let ty = (h as f32 - 2.0) - tarc * (h as f32 - 4.0) * (0.5 + ft * 0.5);
            draw::dot_i(grid, tx as i32, ty as i32);
        }
        draw::dot_i(grid, sx as i32, sy as i32 - 1);
        draw::dot_i(grid, sx as i32 + 1, sy as i32);
        if sx >= 0.0 && sy >= 0.0 && (sx as usize) < w && (sy as usize) < h {
            let _ = grid.set_cell_color(sx as usize / 2, sy as usize / 4, FW_FLASH);
        }
        // Burst bloom when the shot arrives.
        if phase > 0.9 {
            let bloom = (phase - 0.9) * 10.0;
            for ray in 0..8 {
                let ang = ray as f32 * TAU / 8.0;
                let px = target + ang.cos() * bloom * 5.0;
                let py = (h as f32 - 3.0) + ang.sin() * bloom * 3.0;
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, fw_color(ray));
                }
            }
        }
        Ok(())
    }
}

/// A sparkler point burns along a wire, spraying rays as it goes.
struct SparklerFill;
impl ProgressStyle for SparklerFill {
    fn name(&self) -> &str {
        "sparkler-fill"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A sparkler burning along a wire"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h / 2;
        let head = (ctx.eased * w as f32) as usize;
        // Burnt wire behind: a thin steady line.
        for x in 0..head {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, Color::rgb(150, 130, 110));
        }
        // Unburnt wire ahead: sparse dots.
        for x in (head..w).step_by(3) {
            draw::dot(grid, x, mid);
            let _ = grid.set_cell_color(x / 2, mid / 4, Color::rgb(90, 86, 96));
        }
        // The sparkler head: dense random short rays, re-rolled each frame.
        let slot = (ctx.time * 12.0) as i32;
        for s in 0..16i32 {
            let ang = hash3(s, 1, slot) * TAU;
            let len = 1.5 + hash3(s, 2, slot) * 5.0;
            for k in 0..len as i32 {
                let px = head as f32 + ang.cos() * k as f32;
                let py = mid as f32 + ang.sin() * k as f32 * 0.7;
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if k < 2 { FW_FLASH } else { FW_GOLD };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// Small shells burst all along the filled stretch, thicker as it grows.
struct Salvo;
impl ProgressStyle for Salvo {
    fn name(&self) -> &str {
        "salvo"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Shell bursts thickening along the fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // Thin baseline keeps the reading unambiguous.
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, FW_GOLD);
        }
        // Bursts: each has a home position and its own life cycle.
        let shells = 12;
        for s in 0..shells {
            let bx = (hash2(s, 51) * w as f32) as usize;
            if bx >= filled {
                continue;
            }
            let by = 2.0 + hash2(s, 52) * (h as f32 * 0.5);
            let age = (ctx.time * 0.5 + hash2(s, 53)).fract();
            let radius = age * 6.0;
            let fade = 1.0 - age;
            let points = 8;
            for p in 0..points {
                if hash3(s, p, 7) > fade + 0.3 {
                    continue;
                }
                let ang = p as f32 * TAU / points as f32 + hash2(s, 54) * TAU;
                let px = bx as f32 + ang.cos() * radius;
                let py = by + ang.sin() * radius * 0.6 + age * age * 3.0;
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if age < 0.15 {
                        FW_FLASH
                    } else {
                        fw_color(s * 31 + p)
                    };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// One golden willow: long drooping trails that stretch with progress.
struct Willow;
impl ProgressStyle for Willow {
    fn name(&self) -> &str {
        "willow"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Golden trails drooping wider with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0;
        let cy = 1.5;
        let trails = 16;
        let reach = ctx.eased * (w as f32 / 2.0);
        for t in 0..trails {
            let spread = (t as f32 / (trails - 1).max(1) as f32) * 2.0 - 1.0; // -1..1
            let dir = spread * reach;
            let steps = (reach * 0.9) as i32;
            for s in 0..steps {
                let frac = s as f32 / steps.max(1) as f32;
                let px = cx + dir * frac;
                let py = cy + frac * frac * (h as f32 - 2.5) + (hash3(t, s, 3) - 0.5);
                // Trails shimmer out near the tips.
                if frac > 0.6 && hash3(t, s, (ctx.time * 6.0) as i32) < frac - 0.5 {
                    continue;
                }
                draw::dot_i(grid, px as i32, py as i32);
                if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                    let c = if frac < 0.2 { FW_FLASH } else { FW_GOLD };
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                }
            }
        }
        Ok(())
    }
}

/// Single-frame flashes pop across the fill, leaving afterglow rings.
struct StrobePop;
impl ProgressStyle for StrobePop {
    fn name(&self) -> &str {
        "strobe-pop"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "Strobe flashes with afterglow rings"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        // Baseline bar, two rows.
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
            draw::dot(grid, x, h.saturating_sub(2));
            let _ = grid.set_cell_color(x / 2, (h - 1) / 4, FW_VIOLET);
        }
        // Pops: each strobe lives two slots — a bright cross, then a ring.
        let slot = (ctx.time * 6.0) as i32;
        let pops = 2 + (ctx.progress * 6.0) as i32;
        for p in 0..pops {
            let s = slot - p; // stagger pops across recent slots
            let px = (hash3(p, 1, s) * filled.max(1) as f32) as i32;
            let py = 1 + (hash3(p, 2, s) * (h as f32 - 6.0)) as i32;
            if px as usize >= filled {
                continue;
            }
            if p % 2 == 0 {
                // Flash: a bright cross.
                draw::dot_i(grid, px, py);
                draw::dot_i(grid, px - 1, py);
                draw::dot_i(grid, px + 1, py);
                draw::dot_i(grid, px - 2, py);
                draw::dot_i(grid, px + 2, py);
                draw::dot_i(grid, px, py - 1);
                draw::dot_i(grid, px, py + 1);
                if px >= 0 && py >= 0 {
                    let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, FW_FLASH);
                }
            } else {
                // Afterglow: a denser ring.
                for r in 0..8 {
                    let ang = r as f32 * TAU / 8.0;
                    let rx = px + (ang.cos() * 3.5) as i32;
                    let ry = py + (ang.sin() * 2.2) as i32;
                    if hash3(p, r, s) > 0.25 {
                        draw::dot_i(grid, rx, ry);
                        if rx >= 0 && ry >= 0 && (rx as usize) < w && (ry as usize) < h {
                            let _ = grid.set_cell_color(
                                rx as usize / 2,
                                ry as usize / 4,
                                fw_color(p * 7 + r),
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// The whole show at once: a gauge below, a sky that fills with bursts.
struct GrandFinale;
impl ProgressStyle for GrandFinale {
    fn name(&self) -> &str {
        "grand-finale"
    }
    fn theme(&self) -> &str {
        "fireworks"
    }
    fn describe(&self) -> &str {
        "A sky filling with bursts toward the finale"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (cw, ch) = grid.dimensions();
        // The gauge: a crisp eighth-block bar on the bottom cell row.
        if ch > 0 {
            draw::hbar(grid, ch - 1, ctx.eased);
            for cx in 0..cw {
                let _ = grid.set_cell_color(cx, ch - 1, FW_GOLD);
            }
        }
        // The sky above fills with simultaneous bursts as progress climbs.
        let sky_h = h.saturating_sub(4);
        if sky_h < 3 {
            return Ok(());
        }
        let bursts = 1 + (ctx.eased * 5.0) as i32;
        for b in 0..bursts {
            let age = (ctx.time * 0.5 + hash2(b, 61)).fract();
            let bx = (hash3(b, 62, (ctx.time * 0.5 + hash2(b, 61)) as i32) * w as f32) as f32;
            let by = 1.0 + hash2(b, 63) * (sky_h as f32 * 0.6);
            let radius = age * 7.0;
            let rays = 10;
            for r in 0..rays {
                let ang = r as f32 * TAU / rays as f32 + hash2(b, 64) * TAU;
                let px = bx + ang.cos() * radius;
                let py = by + ang.sin() * radius * 0.5 + age * age * 2.5;
                if py >= sky_h as f32 {
                    continue;
                }
                if hash3(b, r, 9) > age {
                    draw::dot_i(grid, px as i32, py as i32);
                    if px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h {
                        let c = if age < 0.12 {
                            FW_FLASH
                        } else {
                            fw_color(b * 13 + r)
                        };
                        let _ = grid.set_cell_color(px as usize / 2, py as usize / 4, c);
                    }
                }
            }
        }
        // The hundred-percent moment: a full-sky strobe.
        if ctx.progress > 0.97 && (ctx.time * 6.0) as i32 % 2 == 0 {
            for x in (0..w).step_by(2) {
                draw::dot(grid, x, 0);
                let _ = grid.set_cell_color(x / 2, 0, FW_FLASH);
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
    let styles = progress::styles::fireworks::styles();
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
