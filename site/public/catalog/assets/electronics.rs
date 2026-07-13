//! `electronics` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O electronics.rs && ./electronics [style-name]
//! ```

const DEFAULT_STYLE: &str = "rc-charge";

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
    pub mod electronics {
//! Electronics / signals themed progress bars for dotmax.
//!
//! Every style in this theme is grounded in a real electronics or signals
//! concept — not just a colour change. The visual vocabulary deliberately
//! maps to physical behaviour:
//!
//! - [`RcCharge`] — exponential V=V₀(1−e^(−t/RC)) capacitor fill
//! - [`Oscilloscope`] — graticule + live waveform sweep (sine/triangle/square)
//! - [`LogicGate`] — signal pulse propagating through AND→OR→XOR→NOT
//! - [`SevenSegment`] — numeric counter rendered with segment lines
//! - [`LedVuMeter`] — VU column meter with peak-hold dots
//! - [`BinaryBus`] — parallel bit-bus with bits shifting left→right
//! - [`SquareClock`] — clock signal with scrolling rising/falling edges
//! - [`PwmDuty`] — PWM pulse whose duty cycle widens with progress
//! - [`ResistorBands`] — colour-coded resistor bands revealing left→right
//! - [`SignalNoise`] — clean sine emerging from noise as SNR improves
//! - [`LissajousScope`] — Lissajous x=sin(at), y=sin(bt+δ) on a CRT grid

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `electronics` theme.
///
/// Returns 11 structurally distinct bars, each modelling a different concept
/// from analogue/digital electronics. Safe to render at any size from 1×1 up.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(RcCharge),
        Box::new(Oscilloscope),
        Box::new(LogicGate),
        Box::new(SevenSegment),
        Box::new(LedVuMeter),
        Box::new(BinaryBus),
        Box::new(SquareClock),
        Box::new(PwmDuty),
        Box::new(ResistorBands),
        Box::new(SignalNoise),
        Box::new(LissajousScope),
    ]
}

// ---------------------------------------------------------------------------
// 1. RC capacitor charging
//    V(t) = V₀ · (1 − e^(−eased / (1−eased+ε)))
//    The curve is drawn dot-by-dot from left to right; a capacitor symbol
//    (two vertical plates separated by a gap) appears at the right edge and
//    fills from bottom upward proportional to V.
// ---------------------------------------------------------------------------
struct RcCharge;
impl ProgressStyle for RcCharge {
    fn name(&self) -> &str {
        "rc-charge"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "RC capacitor charging: exponential V=V₀(1−e^(−t/RC)) curve draws out, capacitor fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Reserve the rightmost 4 dot-columns for the capacitor symbol.
        let cap_w = 4usize;
        let plot_w = w.saturating_sub(cap_w + 1);

        // Draw the exponential charging curve across plot_w columns.
        // We evaluate V at each x position relative to progress reaching that x.
        let mid_y = (h - 1) as i32; // baseline at the bottom
        let top_y = 0i32; // 100% charged = top dot row

        let mut prev_y: Option<i32> = None;
        for xi in 0..plot_w {
            // normalised x in [0, 1]
            let xn = if plot_w <= 1 {
                ctx.eased
            } else {
                xi as f32 / (plot_w - 1) as f32
            };
            // The curve is only drawn up to the current charge level (eased).
            let effective = xn.min(ctx.eased);
            // RC time constant: tau chosen so 5RC covers the bar.
            let tau = 0.2_f32;
            let v = 1.0 - (-effective / tau).exp();
            // v in [0, 1]; map to dot row (0 = top, h-1 = bottom)
            let dot_y = (mid_y - (v * h as f32 * 0.9) as i32).clamp(top_y, mid_y);
            draw::dot_i(grid, xi as i32, dot_y);
            // Connect consecutive dots vertically to avoid gaps.
            if let Some(py) = prev_y {
                let lo = py.min(dot_y);
                let hi = py.max(dot_y);
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dot_y);
        }

        // Draw baseline.
        draw::hline(grid, 0, plot_w.saturating_sub(1), h - 1);

        // Draw the capacitor symbol: two plates (vlines) with a gap between.
        if cap_w < w {
            let plate_x1 = w.saturating_sub(cap_w);
            let plate_x2 = w.saturating_sub(cap_w - 2);
            let plate_top = h / 4;
            let plate_bot = h.saturating_sub(h / 4).max(plate_top);
            draw::vline(grid, plate_x1, plate_top, plate_bot);
            draw::vline(grid, plate_x2, plate_top, plate_bot);
            // Wire into the plates from the curve endpoint and from right.
            let wire_y = h / 2;
            draw::hline(grid, plot_w, plate_x1, wire_y);
            draw::hline(grid, plate_x2, w - 1, wire_y);
            // Fill the capacitor from bottom upward proportional to charge.
            let fill_h = (ctx.eased * (plate_bot - plate_top + 1) as f32).round() as usize;
            let fill_y0 = plate_bot.saturating_sub(fill_h.saturating_sub(1));
            if fill_h > 0 {
                for fy in fill_y0..=plate_bot {
                    draw::dot(grid, plate_x1 + 1, fy);
                }
            }
        }

        // Tint the charged region.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = cx as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Oscilloscope
//    A graticule (light grid lines) with a live waveform trace.
//    Waveform shape cycles through sine → triangle → square as eased rises.
//    The trace sweeps rightward driven by time.
// ---------------------------------------------------------------------------
struct Oscilloscope;
impl ProgressStyle for Oscilloscope {
    fn name(&self) -> &str {
        "oscilloscope"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "CRT oscilloscope with graticule grid and waveform sweeping sine→triangle→square"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Draw graticule: sparse horizontal and vertical lines.
        let h_divs = 4usize.max(1);
        let v_divs = 8usize.max(1);
        for di in 0..=h_divs {
            let y = di * h / h_divs.max(1);
            let y = y.min(h - 1);
            // Dashed: every 4 dots draw 2, skip 2.
            for xi in (0..w).step_by(4) {
                draw::dot(grid, xi, y);
                if xi + 1 < w {
                    draw::dot(grid, xi + 1, y);
                }
            }
        }
        for di in 0..=v_divs {
            let x = di * w / v_divs.max(1);
            let x = x.min(w - 1);
            for yi in (0..h).step_by(4) {
                draw::dot(grid, x, yi);
                if yi + 1 < h {
                    draw::dot(grid, x, yi + 1);
                }
            }
        }

        // Choose waveform: sine (0..0.33), triangle (0.33..0.67), square (0.67..1).
        let shape = (ctx.eased * 3.0).floor() as usize; // 0,1,2
        let freq = 3.0_f32; // cycles across bar
        let phase = ctx.time * 2.0 * PI * 0.5;
        let amp = (h as f32 * 0.42).max(1.0);
        let mid = (h / 2) as i32;

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let theta = (xi as f32 / w as f32) * freq * 2.0 * PI + phase;
            let val: f32 = match shape {
                0 => theta.sin(),
                1 => {
                    // Triangle: 2/π · arcsin(sin(θ))
                    (2.0 / PI) * theta.sin().asin()
                }
                _ => {
                    // Square
                    if theta.sin() >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
            };
            let dy = (mid - (val * amp) as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let lo = py.min(dy);
                let hi = py.max(dy);
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Tint: full-width green-ish glow (use palette, skew toward end colour).
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t * ctx.eased);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Logic-gate cascade
//    A signal pulse propagates through four gates: AND → OR → XOR → NOT.
//    Gates are drawn as text symbols; the signal wire lights up between them
//    as progress crosses each threshold (0.25, 0.5, 0.75, 1.0).
//    The active gate flickers at ~4 Hz to indicate processing.
// ---------------------------------------------------------------------------
struct LogicGate;
impl ProgressStyle for LogicGate {
    fn name(&self) -> &str {
        "logic-gate"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "Signal pulse cascades through AND→OR→XOR→NOT gates; active gate flickers"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, _h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Gate labels; we render them as glyphs.
        let gates = ["&", "|", "^", "!"];
        let n_gates = gates.len();
        // Each gate occupies (cw / (n_gates+1)) cells; wires fill the gaps.
        let spacing = (cw / (n_gates + 1)).max(1);

        // Wire row: middle cell row.
        let wire_row = ch / 2;

        // Draw the full wire as a horizontal run of dots.
        let wire_dot_y = wire_row * 4 + 1; // a dot near the middle of the cell row
        let wire_dot_y = wire_dot_y.min(w.saturating_sub(1)); // reuse h bound check via dot()
                                                              // Horizontal baseline wire across the whole grid dot-width.
        draw::hline(
            grid,
            0,
            w.saturating_sub(1),
            wire_dot_y.min({
                let (_, dh) = draw::dot_dims(grid);
                dh.saturating_sub(1)
            }),
        );

        // Thicken the powered stretch of wire so progress reads without
        // color, and race a pulse packet along it.
        let (_, dh) = draw::dot_dims(grid);
        let powered = (ctx.eased * w as f32) as usize;
        let wire_y = (wire_row * 4 + 1).min(dh.saturating_sub(2));
        for x in 0..powered.min(w) {
            draw::dot(grid, x, wire_y + 1);
        }
        if powered > 4 {
            let pulse = ((ctx.time * 0.75).fract() * powered as f32) as usize;
            for k in 0..4usize {
                if pulse >= k {
                    draw::dot(grid, pulse - k, wire_y.saturating_sub(1));
                }
            }
        }

        // Draw each gate symbol and light up wires up to the active gate.
        let lit_gates = (ctx.eased * n_gates as f32).floor() as usize;
        let flicker_on = (ctx.time * 8.0) as usize % 2 == 0;

        for (gi, &label) in gates.iter().enumerate() {
            let gate_cell_x = (gi + 1) * spacing;
            if gate_cell_x >= cw {
                break;
            }

            // Draw the gate glyph at wire_row.
            let ch_sym = label.chars().next().unwrap_or('?');
            draw::glyph(grid, gate_cell_x, wire_row, ch_sym);

            // Tint the wire cells leading up to this gate if lit.
            if gi < lit_gates {
                let wire_start = if gi == 0 { 0 } else { gi * spacing };
                let wire_end = gate_cell_x.saturating_sub(1);
                for cx in wire_start..=wire_end.min(cw.saturating_sub(1)) {
                    let col = ctx.palette.sample(gi as f32 / n_gates as f32);
                    for cy in 0..ch {
                        draw::tint_row(grid, cy, cx, cx, col);
                    }
                }
                // Tint the gate cell itself.
                let col = ctx.palette.sample((gi + 1) as f32 / n_gates as f32);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, gate_cell_x, gate_cell_x, col);
                }
            } else if gi == lit_gates && flicker_on {
                // Active gate (being processed): flicker.
                let col = ctx.palette.sample(0.8);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, gate_cell_x, gate_cell_x, col);
                }
            }
        }

        // Tint the wire after the last lit gate.
        if lit_gates >= n_gates {
            let last_gate_x = n_gates * spacing;
            let trail_start = last_gate_x + 1;
            for cx in trail_start..cw {
                let col = ctx.palette.sample(1.0);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, cx, cx, col);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. 7-segment counter
//    Counts from 0 up to a value proportional to eased (max 99 for 2 digits).
//    Each digit is drawn using segment lines in dot space.
//    Segments: top, top-left, top-right, middle, bot-left, bot-right, bottom.
// ---------------------------------------------------------------------------

/// Segment on/off table for digits 0–9.
/// Order: [top, tl, tr, mid, bl, br, bot]
const SEG7: [[bool; 7]; 10] = [
    [true, true, true, false, true, true, true],     // 0
    [false, false, true, false, false, true, false], // 1
    [true, false, true, true, true, false, true],    // 2
    [true, false, true, true, false, true, true],    // 3
    [false, true, true, true, false, true, false],   // 4
    [true, true, false, true, false, true, true],    // 5
    [true, true, false, true, true, true, true],     // 6
    [true, false, true, false, false, true, false],  // 7
    [true, true, true, true, true, true, true],      // 8
    [true, true, true, true, false, true, true],     // 9
];

/// Draw a single 7-segment digit in dot space at (ox, oy) with given w×h.
fn draw_seg7_digit(
    grid: &mut BrailleGrid,
    digit: usize,
    ox: usize,
    oy: usize,
    sw: usize,
    sh: usize,
) {
    let d = digit.min(9);
    let segs = SEG7[d];
    let mid_y = oy + sh / 2;
    let bot_y = oy + sh.saturating_sub(1);
    let right_x = ox + sw.saturating_sub(1);

    // top
    if segs[0] {
        draw::hline(grid, ox, right_x, oy);
    }
    // top-left
    if segs[1] {
        draw::vline(grid, ox, oy, mid_y);
    }
    // top-right
    if segs[2] {
        draw::vline(grid, right_x, oy, mid_y);
    }
    // middle
    if segs[3] {
        draw::hline(grid, ox, right_x, mid_y);
    }
    // bot-left
    if segs[4] {
        draw::vline(grid, ox, mid_y, bot_y);
    }
    // bot-right
    if segs[5] {
        draw::vline(grid, right_x, mid_y, bot_y);
    }
    // bottom
    if segs[6] {
        draw::hline(grid, ox, right_x, bot_y);
    }
}

struct SevenSegment;
impl ProgressStyle for SevenSegment {
    fn name(&self) -> &str {
        "seven-segment"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "7-segment numeric counter: digits drawn with segment lines, counting up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Determine how many digits we can fit.
        // Each digit needs ~6 dot-columns wide and full height.
        let digit_w = (w / 4).max(3).min(12);
        let gap = 2usize;
        let n_digits = ((w + gap) / (digit_w + gap)).max(1).min(4);

        // Current count: 0..=10^n_digits - 1
        let max_val = 10usize.pow(n_digits as u32).saturating_sub(1);
        let count = (ctx.eased * max_val as f32).round() as usize;

        // Centre the digits horizontally.
        let total_w = n_digits * digit_w + (n_digits - 1) * gap;
        let ox = w.saturating_sub(total_w) / 2;

        for di in 0..n_digits {
            // Extract the di-th digit from count (leftmost = most significant).
            let place = 10usize.pow((n_digits - 1 - di) as u32);
            let digit = (count / place) % 10;
            let dx = ox + di * (digit_w + gap);
            if dx + digit_w <= w {
                draw_seg7_digit(grid, digit, dx, 0, digit_w, h);
            }
        }

        // Tint: gradient across the full width, brightness scales with eased.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t * ctx.eased + ctx.eased * 0.5);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. LED VU meter
//    N vertical bar columns; each column's level is driven by a synthetic
//    audio signal: A_k = |sin(k·time · f_k)| where f_k varies per column.
//    The filled region (left columns) react fully; right columns are dim.
//    Peak-hold: a single dot at the highest recent level per column.
// ---------------------------------------------------------------------------
struct LedVuMeter;
impl ProgressStyle for LedVuMeter {
    fn name(&self) -> &str {
        "led-vu-meter"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "LED VU bar graph: columns pulse to synthetic audio levels; peak-hold dot per column"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        let n_cols = cw;
        let lit_cols = (ctx.eased * n_cols as f32).round() as usize;

        for col in 0..n_cols {
            // Synthetic audio level per column: sum of two sinusoids.
            let kf = (col + 1) as f32;
            let freq1 = 0.7 + kf * 0.3;
            let freq2 = 1.3 + kf * 0.17;
            let raw = ((kf * ctx.time * freq1).sin().abs()
                + (kf * 0.5 * ctx.time * freq2 + 1.0).sin().abs())
                * 0.5;
            let level = if col < lit_cols { raw } else { raw * 0.12 };

            // Convert level [0,1] to vblock eighths per cell row.
            // Columns fill from the bottom upward.
            let total_eighths = (level * ch as f32 * 8.0).round() as usize;
            let full_cells = total_eighths / 8;
            let rem = total_eighths % 8;

            // Draw full cells from bottom upward.
            for row in 0..full_cells.min(ch) {
                let cell_y = ch.saturating_sub(1).saturating_sub(row);
                draw::vblock(grid, col, cell_y, 8);
            }
            // Partial cell above the full cells.
            if rem > 0 && full_cells < ch {
                let cell_y = ch.saturating_sub(1).saturating_sub(full_cells);
                draw::vblock(grid, col, cell_y, rem);
            }

            // Peak-hold dot: place a single vblock=1 at the top of the level.
            let peak_eighths = total_eighths.saturating_add(4).min(ch * 8);
            let peak_row = ch.saturating_sub(1).saturating_sub(peak_eighths / 8);
            if peak_row < ch && full_cells > 0 {
                draw::vblock(grid, col, peak_row, 1);
            }

            // Tint the column.
            if col < lit_cols {
                let t = col as f32 / n_cols.max(1) as f32;
                let col_color = ctx.palette.sample(t);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, col, col, col_color);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Binary data bus
//    8 parallel horizontal bit lines. Bits shift left→right over time.
//    The number of "ones" in the current word = floor(eased × 8).
//    Each line carries a different bit position of a pseudo-random byte stream.
// ---------------------------------------------------------------------------
struct BinaryBus;
impl ProgressStyle for BinaryBus {
    fn name(&self) -> &str {
        "binary-bus"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "8-bit data bus: parallel bit lines shift bits left→right; word density = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of bus lines — cap at 8, fill h evenly.
        let n_lines = 8usize.min(h);
        let ones_per_word = (ctx.eased * n_lines as f32).round() as usize;

        // Bit cell width in dots (each bit takes bit_w dots).
        let bit_w = (w / 16).max(2);
        let n_cells = w / bit_w;

        // Scroll offset: bits move left at 4 cells/sec.
        let scroll = (ctx.time * 4.0) as usize;

        for line in 0..n_lines {
            // y position for this line: evenly spaced.
            let y = if n_lines <= 1 {
                h / 2
            } else {
                line * (h - 1) / (n_lines - 1)
            };
            let y = y.min(h - 1);

            // Draw each bit cell.
            for ci in 0..n_cells {
                // Determine the bit value via a simple LFSR-like hash of (ci+scroll, line).
                let slot = ci.wrapping_add(scroll);
                // Hash: mix slot and line to produce a pseudo-random bit.
                let hash = slot
                    .wrapping_mul(2654435761)
                    .wrapping_add(line.wrapping_mul(40503));
                // Which bit of hash to use: cycle through n_lines bits.
                let bit_pos = line % 8;
                let raw_bit = (hash >> bit_pos) & 1;
                // bit is "1" only if it is in the active ones budget AND raw says so.
                let bit = raw_bit == 1 && line < ones_per_word;

                let x0 = ci * bit_w;
                let x1 = (x0 + bit_w).saturating_sub(2).max(x0);

                if bit {
                    // HIGH: draw a line at top of cell.
                    draw::hline(grid, x0, x1, y.saturating_sub(1).max(0));
                    draw::hline(grid, x0, x1, y);
                } else {
                    // LOW: draw a line at bottom (just one dot row).
                    draw::hline(grid, x0, x1, (y + 1).min(h - 1));
                }

                // Rising/falling edge connectors (vertical transitions).
                // Peek at next bit.
                if ci + 1 < n_cells {
                    let next_slot = ci + 1 + scroll;
                    let next_hash = next_slot
                        .wrapping_mul(2654435761)
                        .wrapping_add(line.wrapping_mul(40503));
                    let next_raw = (next_hash >> (line % 8)) & 1;
                    let next_bit = next_raw == 1 && line < ones_per_word;
                    if bit != next_bit && x1 + 1 < w {
                        // Draw a vertical edge connector at x1.
                        let y_lo = y.saturating_sub(1);
                        let y_hi = (y + 1).min(h - 1);
                        draw::vline(grid, x1 + 1, y_lo, y_hi);
                    }
                }
            }
        }

        // Tint active lines.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t * ctx.eased);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Square-wave clock signal
//    A digital clock trace scrolls from right to left.
//    Duty cycle is fixed at 50%; eased controls how many complete clock
//    cycles have passed (i.e. the fill level = cycles completed).
//    Rising and falling edges are sharp verticals.
// ---------------------------------------------------------------------------
struct SquareClock;
impl ProgressStyle for SquareClock {
    fn name(&self) -> &str {
        "square-clock"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "Digital clock signal scrolling left; rising/falling edges sharp; progress = cycles complete"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Clock period in dots; eased sets how many full cycles fit in the bar.
        let min_period = 8usize;
        let max_cycles = (w / min_period).max(1);
        let cycles = (ctx.eased * max_cycles as f32).ceil() as usize + 1;
        let period = (w / cycles.max(1)).max(4);

        // Scroll: the waveform shifts left over time.
        let scroll_dots = (ctx.time * 6.0) as usize % (period * 2).max(1);

        let hi_y = h / 4; // top rail (logic HIGH)
        let lo_y = h.saturating_sub(h / 4 + 1); // bottom rail (logic LOW)

        let lit_x = (ctx.eased * w as f32).round() as usize; // filled boundary

        let mut prev_level: Option<bool> = None;
        for xi in 0..w {
            // Phase position within the period, accounting for scroll.
            let phase = (xi + scroll_dots) % (period * 2).max(1);
            let high = phase < period;

            let y = if high { hi_y } else { lo_y };
            let y = y.min(h - 1);

            // Draw the horizontal rail dot.
            draw::dot(grid, xi, y);

            // Draw the vertical edge when the level changes.
            if let Some(prev) = prev_level {
                if prev != high {
                    draw::vline(grid, xi, hi_y, lo_y);
                }
            }

            prev_level = Some(high);

            // Dim dots beyond the progress boundary.
            if xi >= lit_x && xi < lit_x + 2 {
                // Draw a progress cursor marker.
                draw::vline(grid, xi, 0, h.saturating_sub(1));
            }
        }

        // Tint.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let col = ctx.palette.sample(cx as f32 / cw.max(1) as f32);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. PWM duty cycle
//    A repeating pulse whose duty cycle = eased (0%→100%).
//    Multiple pulses fill the bar; the on-time widens as progress increases.
//    Frequency stays constant (≈8 pulses across the bar).
// ---------------------------------------------------------------------------
struct PwmDuty;
impl ProgressStyle for PwmDuty {
    fn name(&self) -> &str {
        "pwm-duty"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "PWM signal: fixed-frequency pulses whose on-time duty cycle widens with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n_pulses = 8usize;
        let period = (w / n_pulses).max(2);
        let on_time = (ctx.eased * period as f32).round() as usize;

        let hi_y = h / 5;
        let lo_y = h.saturating_sub(h / 5 + 1).min(h - 1);

        // Scroll slightly with time for visual feedback.
        let scroll = (ctx.time * 3.0) as usize % period.max(1);

        for xi in 0..w {
            let phase = (xi + scroll) % period.max(1);
            let is_high = phase < on_time;
            let y = if is_high { hi_y } else { lo_y };
            draw::dot(grid, xi, y.min(h - 1));

            // Vertical edge.
            if xi > 0 {
                let prev_phase = (xi + scroll - 1) % period.max(1);
                let prev_high = prev_phase < on_time;
                if is_high != prev_high {
                    draw::vline(grid, xi, hi_y, lo_y);
                }
            }
        }

        // Shading inside the ON region using shade glyphs for visual texture.
        let (cw, ch) = grid.dimensions();
        for col in 0..cw {
            let xi_mid = col * 2 + 1;
            let phase = (xi_mid + scroll) % (period * 2 / 1).max(2); // dot phase
            let cell_period = period / 2; // cells per period (each cell = 2 dots)
            let cell_on = (on_time / 2).max(if on_time > 0 { 1 } else { 0 });
            let cell_phase = (col + scroll / 2) % cell_period.max(1);
            if cell_phase < cell_on && cell_period > 0 {
                // Use shade to indicate ON time.
                let density = 2usize + (ctx.eased * 2.0) as usize;
                for cy in 0..ch {
                    draw::shade(grid, col, cy, density.min(4));
                }
                let col_color = ctx.palette.sample(col as f32 / cw.max(1) as f32);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, col, col, col_color);
                }
            }
            let _ = phase; // suppress unused warning
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Resistor colour bands
//    A resistor body drawn with end caps. Colour bands reveal left→right
//    as progress increases. Each band is a distinct vertical stripe of
//    dots at a fixed width. The IEC resistor colour code:
//    0=black, 1=brown, 2=red, 3=orange, 4=yellow, 5=green, 6=blue,
//    7=violet, 8=grey, 9=white. Bands use palette lerp for terminal colour.
// ---------------------------------------------------------------------------

/// Band colour fractions within the start→end palette, indexed by digit 0–9.
const BAND_T: [f32; 10] = [
    0.0,  // 0 black   → start
    0.11, // 1 brown
    0.22, // 2 red
    0.33, // 3 orange
    0.44, // 4 yellow
    0.55, // 5 green
    0.66, // 6 blue
    0.77, // 7 violet
    0.88, // 8 grey
    1.0,  // 9 white   → end
];

struct ResistorBands;
impl ProgressStyle for ResistorBands {
    fn name(&self) -> &str {
        "resistor-bands"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "Resistor with IEC colour bands revealing left-to-right as progress increases"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w == 0 || h == 0 || cw == 0 || ch == 0 {
            return Ok(());
        }

        // Resistor body: a filled horizontal rectangle in the middle 60% of height.
        let body_top = h / 5;
        let body_bot = h.saturating_sub(h / 5).max(body_top);

        // End caps / lead wires (thin horizontal lines at mid-height).
        let lead_y = (body_top + body_bot) / 2;
        let body_x0 = w / 6;
        let body_x1 = w.saturating_sub(w / 6).max(body_x0 + 1);
        // Left lead.
        draw::hline(grid, 0, body_x0, lead_y);
        // Right lead.
        draw::hline(grid, body_x1, w.saturating_sub(1), lead_y);
        // Body outline.
        draw::rect_outline(
            grid,
            body_x0,
            body_top,
            body_x1 - body_x0,
            body_bot - body_top + 1,
        );
        // Body fill (sparse, to show bands over it).
        for y in body_top + 1..body_bot {
            for x in body_x0 + 1..body_x1 {
                if (x + y) % 3 == 0 {
                    draw::dot(grid, x, y);
                }
            }
        }

        // Bands: 4 bands (3 value + 1 tolerance) within the body.
        let n_bands = 4usize;
        let body_len = body_x1.saturating_sub(body_x0 + 2); // inner pixels
        let band_spacing = body_len / (n_bands + 1);
        let band_w = (band_spacing / 2).max(1);

        let lit_bands = (ctx.eased * n_bands as f32).ceil() as usize;

        for bi in 0..n_bands {
            if bi >= lit_bands {
                break;
            }
            // Band centre x in dot space.
            let cx_dot = body_x0 + 1 + (bi + 1) * band_spacing;
            let bx0 = cx_dot.saturating_sub(band_w / 2);
            let bx1 = (bx0 + band_w).min(body_x1.saturating_sub(1));

            // Digit for this band: use a fixed sequence 4,7,2,5 (a representative value).
            let digits = [4usize, 7, 2, 5];
            let digit = digits[bi % digits.len()];
            let band_t = BAND_T[digit];

            // Draw band as solid vertical stripe.
            for bx in bx0..=bx1 {
                draw::vline(
                    grid,
                    bx,
                    body_top + 1,
                    body_bot.saturating_sub(1).max(body_top + 1),
                );
            }

            // Tint the band cells.
            let band_cell_x0 = bx0 / 2;
            let band_cell_x1 = (bx1 / 2 + 1).min(cw.saturating_sub(1));
            let col = ctx.palette.sample(band_t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, band_cell_x0, band_cell_x1, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Signal through noise
//     A sinusoidal target signal mixed with Gaussian-like noise.
//     SNR = eased: at 0 the trace is pure noise; at 1 it is a clean sine.
//     Noise is generated from a deterministic hash of (x, time_bucket).
// ---------------------------------------------------------------------------

/// Fast deterministic noise in [-1, 1] from integer seed.
#[inline]
fn pseudo_noise(seed: u32) -> f32 {
    let h = seed
        .wrapping_mul(2246822519)
        .wrapping_add(seed.wrapping_mul(3266489917));
    let h = h ^ (h >> 13);
    let h = h.wrapping_mul(1274126177);
    let h = h ^ (h >> 16);
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

struct SignalNoise;
impl ProgressStyle for SignalNoise {
    fn name(&self) -> &str {
        "signal-noise"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "Sine signal rising from noise: SNR improves with progress until the clean wave emerges"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let snr = ctx.eased; // 0=all noise, 1=clean sine
        let noise_amp = 1.0 - snr * 0.95; // noise amplitude → 0 as SNR → 1
        let sig_amp = snr; // signal amplitude → 1

        let freq = 3.0_f32;
        let phase = ctx.time * PI * 0.6;

        // Quantise time into buckets (4 per second) so noise is stable between frames.
        let time_bucket = (ctx.time * 4.0) as u32;

        let mid = (h / 2) as i32;
        let half_h = (h as f32 * 0.45).max(1.0);

        let mut prev_y: Option<i32> = None;
        for xi in 0..w {
            let xn = xi as f32 / w as f32;
            let sine_val = (xn * freq * 2.0 * PI + phase).sin();
            let noise_val = pseudo_noise(xi as u32 ^ time_bucket.wrapping_mul(1013904223));
            let val = sig_amp * sine_val + noise_amp * noise_val;
            let dy = (mid - (val * half_h) as i32).clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let lo = py.min(dy);
                let hi = py.max(dy);
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Baseline.
        draw::hline(grid, 0, w.saturating_sub(1), (h / 2).min(h - 1));

        // Tint: gradient that sharpens (more saturated) as SNR increases.
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t * 0.5 + snr * 0.5);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Lissajous on an oscilloscope screen
//     x = sin(a·τ + δ),  y = sin(b·τ)
//     a and b selected by eased (ratio table), δ drifts with time.
//     A faint CRT-style circular border frames the display.
// ---------------------------------------------------------------------------

/// (a, b) ratio pairs for Lissajous figures, indexed by eased × N_RATIOS.
const LISSAJOUS_RATIOS: [(f32, f32); 6] = [
    (1.0, 1.0), // circle / ellipse
    (2.0, 1.0), // figure-8 horizontal
    (1.0, 2.0), // figure-8 vertical
    (3.0, 2.0), // trefoil-like
    (3.0, 4.0), // 5-lobe
    (5.0, 4.0), // complex knot
];

struct LissajousScope;
impl ProgressStyle for LissajousScope {
    fn name(&self) -> &str {
        "lissajous-scope"
    }
    fn theme(&self) -> &str {
        "electronics"
    }
    fn describe(&self) -> &str {
        "Lissajous figure on a CRT scope: ratio unlocks with progress, phase drifts with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n_ratios = LISSAJOUS_RATIOS.len();
        let idx = ((ctx.eased * n_ratios as f32) as usize).min(n_ratios - 1);
        let (a, b) = LISSAJOUS_RATIOS[idx];

        let delta = ctx.time * 0.4; // phase drift

        // Draw a faint circular CRT bezel (ellipse inscribed in the dot rect).
        let cx_f = (w as f32 - 1.0) / 2.0;
        let cy_f = (h as f32 - 1.0) / 2.0;
        let rx = cx_f * 0.97;
        let ry = cy_f * 0.97;
        let bezel_pts = (w + h) * 2;
        for i in 0..bezel_pts {
            let angle = (i as f32 / bezel_pts as f32) * 2.0 * PI;
            let px = (cx_f + rx * angle.cos()) as i32;
            let py = (cy_f + ry * angle.sin()) as i32;
            draw::dot_i(grid, px, py);
        }

        // Draw graticule cross-hairs (centre lines only).
        let mid_x = w / 2;
        let mid_y = h / 2;
        // Dashed hline.
        for x in (0..w).step_by(4) {
            draw::dot(grid, x.min(w - 1), mid_y);
            if x + 1 < w {
                draw::dot(grid, x + 1, mid_y);
            }
        }
        // Dashed vline.
        for y in (0..h).step_by(4) {
            draw::dot(grid, mid_x, y.min(h - 1));
            if y + 1 < h {
                draw::dot(grid, mid_x, y + 1);
            }
        }

        // Plot the Lissajous figure.
        let plot_rx = rx * 0.88;
        let plot_ry = ry * 0.88;
        let steps = (w * h).max(512);
        let period = 2.0 * PI;
        let mut prev: Option<(i32, i32)> = None;
        for si in 0..steps {
            let tau = (si as f32 / steps as f32) * period;
            let lx = plot_rx * (a * tau + delta).sin();
            let ly = plot_ry * (b * tau).sin();
            let px = (cx_f + lx) as i32;
            let py = (cy_f + ly) as i32;
            draw::dot_i(grid, px, py);
            if let Some((ox, oy)) = prev {
                let gap = (((px - ox).abs() + (py - oy).abs()) as usize).max(1);
                for s in 1..gap {
                    let f = s as f32 / gap as f32;
                    let ix = (ox as f32 + (px - ox) as f32 * f) as i32;
                    let iy = (oy as f32 + (py - oy) as f32 * f) as i32;
                    draw::dot_i(grid, ix, iy);
                }
            }
            prev = Some((px, py));
        }

        // Tint: full grid coloured by eased.
        let (cw, ch) = grid.dimensions();
        for cell_x in 0..cw {
            let t = cell_x as f32 / cw.max(1) as f32;
            let col = ctx.palette.sample(t * ctx.eased + (1.0 - ctx.eased) * 0.3);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cell_x, cell_x, col);
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
    let styles = progress::styles::electronics::styles();
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
