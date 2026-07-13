//! `cultures` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O cultures.rs && ./cultures [style-name]
//! ```

const DEFAULT_STYLE: &str = "mandala";

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
    pub mod cultures {
//! World-cultures / ornament-themed progress bars.
//!
//! Ten structurally distinct styles, each drawing itself as `ctx.eased` rises
//! and animating via `ctx.time`.  Every style encodes a real cultural ornament
//! pattern in braille dot-space:
//!
//! - `mandala`        — radial symmetry: petals and concentric rings unfurl
//! - `celtic-knot`    — interlaced over/under woven bands fill the bar
//! - `aztec-fret`     — stepped-fret meander tiles from left to right
//! - `islamic-star`   — 8-fold star tessellation tiles revealed by progress
//! - `greek-key`      — meander border scrolls in from the left
//! - `seigaiha`       — Japanese overlapping wave-scale arcs fill row by row
//! - `totem-pole`     — carved segments stack upward as progress rises
//! - `runes`          — Norse rune glyphs carved in one by one
//! - `paisley-swirl`  — henna swirl spirals unfurl from multiple seeds
//! - `kente-weave`    — kente / tartan warp-and-weft interlacing

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── optional easing import ────────────────────────────────────────────────────
use super::super::{ease, Easing};

// ─────────────────────────────────────────────────────────────────────────────
// Public registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `cultures` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per ornament style, in display order.
/// Every style is structurally distinct — variety is in shape, not colour.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Mandala),
        Box::new(CelticKnot),
        Box::new(AztecFret),
        Box::new(IslamicStar),
        Box::new(GreekKey),
        Box::new(Seigaiha),
        Box::new(TotemPole),
        Box::new(Runes),
        Box::new(PaisleySwirl),
        Box::new(KenteWeave),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Integer Bresenham line rasteriser. Step-bounded; `draw::dot_i` clips OOB.
fn bres(grid: &mut BrailleGrid, mut x0: i32, mut y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx - dy + 2).unsigned_abs() as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps + 2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Draw a circle arc (dots) from angle `a0` to `a1` (radians) around `(cx,cy)`
/// with dot-space radius `r`.  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r < 0.5 {
        return;
    }
    let arc_len = (a1 - a0).abs() * r;
    let steps = (arc_len.ceil() as usize + 2).max(4);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = a0 + t * (a1 - a0);
        let px = (cx + r * a.cos()).round() as i32;
        let py = (cy + r * a.sin()).round() as i32;
        draw::dot_i(grid, px, py);
    }
}

/// Tint every cell in the grid with a horizontal palette gradient.
fn palette_tint(grid: &mut BrailleGrid, ctx: &BarContext) {
    let (cw, ch) = grid.dimensions();
    for cy in 0..ch {
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Mandala — radial symmetry: concentric rings + petal spokes unfurl
// ─────────────────────────────────────────────────────────────────────────────

struct Mandala;
impl ProgressStyle for Mandala {
    fn name(&self) -> &str {
        "mandala"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Radial mandala: concentric rings and petal spokes bloom outward as progress \
         rises; the whole mandala rotates slowly with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let max_r = (dw.min(dh * 2) as f32 / 2.0 - 1.0).max(1.0);
        let rot = ctx.time * 0.25;

        // How many rings and petals are revealed.
        let n_rings = 4usize;
        let n_petals = 8usize;

        for ring in 0..n_rings {
            let ring_frac = (ring + 1) as f32 / n_rings as f32;
            // Each ring appears when eased > its threshold.
            if ctx.eased < ring_frac * 0.6 {
                continue;
            }
            let r = ring_frac * max_r;
            // Full ring circle.
            arc(grid, cx, cy, r, 0.0, 2.0 * PI);
        }

        // Petal spokes — Bézier-approximated with two symmetric arc pairs.
        let petal_reveal = (ctx.eased * 3.0 - 0.5).clamp(0.0, 1.0);
        for p in 0..n_petals {
            let base_angle = p as f32 * 2.0 * PI / n_petals as f32 + rot;
            let tip_r = max_r * petal_reveal;
            // Draw spoke line.
            let x1 = (cx + tip_r * base_angle.cos()).round() as i32;
            let y1 = (cy + tip_r * base_angle.sin()).round() as i32;
            bres(grid, cx as i32, cy as i32, x1, y1);
            // Petal arc: small bulge to either side.
            if tip_r > 3.0 {
                let bulge_r = tip_r * 0.35;
                let mid_r = tip_r * 0.55;
                let mid_x = cx + mid_r * base_angle.cos();
                let mid_y = cy + mid_r * base_angle.sin();
                let perp = base_angle + PI / 2.0;
                let side_x = mid_x + bulge_r * perp.cos();
                let side_y = mid_y + bulge_r * perp.sin();
                arc(
                    grid,
                    side_x,
                    side_y,
                    bulge_r,
                    base_angle + PI,
                    base_angle + 2.0 * PI,
                );
                let side2_x = mid_x - bulge_r * perp.cos();
                let side2_y = mid_y - bulge_r * perp.sin();
                arc(grid, side2_x, side2_y, bulge_r, base_angle, base_angle + PI);
            }
        }

        // Radial tint from center outward.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Celtic Knot — interlaced over/under woven bands
// ─────────────────────────────────────────────────────────────────────────────

struct CelticKnot;
impl ProgressStyle for CelticKnot {
    fn name(&self) -> &str {
        "celtic-knot"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Celtic interlace: two sinusoidal bands weave over and under each other; \
         the knot expands rightward as progress rises, undulating with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let filled_w = (ctx.eased * dw as f32).round() as usize;
        if filled_w == 0 {
            return Ok(());
        }

        let cy = dh as f32 / 2.0;
        // Amplitude of each strand — scale to half the grid height minus border.
        let amp = (dh as f32 / 2.0 - 1.0).max(0.5);
        // Wave period: 1 full cycle per ~20 dots.
        let period = (dw as f32 / 3.0).max(8.0);
        let phase = ctx.time * 1.5;
        // Band thickness in dots (1 or 2).
        let thick = (dh / 8).max(1);

        for x in 0..filled_w.min(dw) {
            let t = x as f32 / period;
            // Two strands: 180° apart, giving the over/under illusion via
            // a gap wherever they cross.
            let y_a = cy + amp * (2.0 * PI * t + phase).sin();
            let y_b = cy + amp * (2.0 * PI * t + phase + PI).sin();

            // Detect crossing region — suppress one strand near crossings.
            let dist = (y_a - y_b).abs();
            let crossing = dist < amp * 0.5;

            for dy in 0..thick {
                // Strand A (always drawn).
                let ya = y_a as i32 + dy as i32;
                draw::dot_i(grid, x as i32, ya);
                draw::dot_i(grid, x as i32, ya - 1);
                // Strand B suppressed near crossing for the "under" effect.
                if !crossing || (x / (thick + 1)) % 2 == 0 {
                    let yb = y_b as i32 + dy as i32;
                    draw::dot_i(grid, x as i32, yb);
                    draw::dot_i(grid, x as i32, yb - 1);
                }
            }
        }

        // Tint: gradient across the bar.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cy_c in 0..ch {
            let hi = filled_cells.saturating_sub(1).min(cw.saturating_sub(1));
            if filled_cells > 0 {
                draw::tint_row(grid, cy_c, 0, hi, ctx.palette.sample(0.6));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Aztec Fret — stepped-T meander pattern tiles left to right
// ─────────────────────────────────────────────────────────────────────────────

struct AztecFret;
impl ProgressStyle for AztecFret {
    fn name(&self) -> &str {
        "aztec-fret"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Aztec/Maya stepped-fret meander: interlocking T-shaped spirals tile the \
         bar from left to right as progress rises."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Each fret tile is `tile_w` dots wide and fills the full height.
        let tile_w = ((dw / 6).max(3)).min(dw);
        let filled_w = (ctx.eased * dw as f32).round() as usize;
        if filled_w == 0 {
            return Ok(());
        }

        // Outer border line.
        draw::hline(grid, 0, filled_w.min(dw).saturating_sub(1), 0);
        draw::hline(
            grid,
            0,
            filled_w.min(dw).saturating_sub(1),
            dh.saturating_sub(1),
        );

        let n_tiles = (filled_w + tile_w - 1) / tile_w;

        for tile in 0..n_tiles {
            let x0 = tile * tile_w;
            let x1 = (x0 + tile_w).min(filled_w).min(dw);
            if x0 >= x1 {
                continue;
            }

            // Even tiles: stepped fret going right-then-down.
            // Odd tiles: mirror image going left-then-up.
            let flip = tile % 2 == 1;

            // Vertical stem at the start of the tile.
            let stem_x = if flip { x1.saturating_sub(1) } else { x0 };
            draw::vline(
                grid,
                stem_x.min(dw.saturating_sub(1)),
                0,
                dh.saturating_sub(1),
            );

            // Stepped rungs: 3 horizontal bars at equal y-intervals.
            let n_steps = 3usize;
            for s in 1..=n_steps {
                let y = s * dh / (n_steps + 1);
                let rung_x0 = if flip {
                    stem_x.saturating_sub(tile_w / 2)
                } else {
                    stem_x
                };
                let rung_x1 = if flip { stem_x } else { stem_x + tile_w / 2 };
                let rx0 = rung_x0.min(dw.saturating_sub(1));
                let rx1 = rung_x1.min(dw.saturating_sub(1));
                draw::hline(grid, rx0, rx1, y.min(dh.saturating_sub(1)));
                // Short vertical drop at the rung end.
                let drop_x = if flip { rx0 } else { rx1 };
                let drop_top = y.saturating_sub(dh / (n_steps * 2 + 1) + 1);
                draw::vline(grid, drop_x, drop_top, y.min(dh.saturating_sub(1)));
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Islamic Star — 8-fold star tessellation
// ─────────────────────────────────────────────────────────────────────────────

struct IslamicStar;
impl ProgressStyle for IslamicStar {
    fn name(&self) -> &str {
        "islamic-star"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Islamic geometric: 8-fold star tiles tesselate the bar, each star's points \
         appearing as progress rises; the grid shimmers in slow rotation with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Tile the grid with 8-pointed stars (octagram).
        // Each star is centred on a regular grid, radius scales to cell size.
        let star_r = ((dh as f32 / 2.0).min(dw as f32 / 2.0) - 1.0).max(2.0);
        let cell_x = (star_r * 2.2).max(4.0);
        let cell_y = (star_r * 2.2).max(4.0);

        let n_cols = ((dw as f32 / cell_x).ceil() as usize + 1).max(1);
        let n_rows = ((dh as f32 / cell_y).ceil() as usize + 1).max(1);

        // Animation: slow rotation of star orientation.
        let rot = ctx.time * 0.18;

        // Draw each star up to the progress-gated total.
        let total_stars = n_cols * n_rows;
        let reveal = (ctx.eased * total_stars as f32).ceil() as usize;

        for idx in 0..reveal.min(total_stars) {
            let row = idx / n_cols;
            let col = idx % n_cols;

            // Hex-offset: odd rows shifted by half a cell.
            let off_x = if row % 2 == 1 { cell_x / 2.0 } else { 0.0 };
            let cx = col as f32 * cell_x + star_r + off_x;
            let cy = row as f32 * cell_y + star_r;

            // Draw an 8-pointed star: outer 8 points + inner 8 points.
            let n_pts = 8usize;
            let inner_r = star_r * 0.42;
            let mut outer: Vec<(i32, i32)> = Vec::with_capacity(n_pts);
            let mut inner: Vec<(i32, i32)> = Vec::with_capacity(n_pts);

            for i in 0..n_pts {
                let angle = i as f32 * 2.0 * PI / n_pts as f32 + rot;
                let half_angle = angle + PI / n_pts as f32;
                outer.push((
                    (cx + star_r * angle.cos()).round() as i32,
                    (cy + star_r * angle.sin()).round() as i32,
                ));
                inner.push((
                    (cx + inner_r * half_angle.cos()).round() as i32,
                    (cy + inner_r * half_angle.sin()).round() as i32,
                ));
            }

            // Connect outer[i] → inner[i] → outer[i+1] forming the star outline.
            for i in 0..n_pts {
                let next = (i + 1) % n_pts;
                bres(grid, outer[i].0, outer[i].1, inner[i].0, inner[i].1);
                bres(grid, inner[i].0, inner[i].1, outer[next].0, outer[next].1);
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Greek Key — meander border scrolling from the left
// ─────────────────────────────────────────────────────────────────────────────

struct GreekKey;
impl ProgressStyle for GreekKey {
    fn name(&self) -> &str {
        "greek-key"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Greek key / meander border: interlocking rectangular spirals scroll in \
         from the left edge as progress rises, one complete meander unit at a time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // One meander unit is a rectangular spiral: right, down, left, up, right.
        // Unit width in dots.
        let unit_w = ((dh * 2).max(6)).min(dw / 2 + 1);
        let unit_h = dh.saturating_sub(2).max(2);
        let step = (unit_w / 4).max(1);

        let filled_w = (ctx.eased * dw as f32).round() as usize;
        if filled_w == 0 {
            return Ok(());
        }

        let n_units = (filled_w + unit_w - 1) / unit_w;

        for u in 0..n_units {
            let x0 = u * unit_w;
            if x0 >= dw {
                break;
            }
            let x1 = (x0 + unit_w).min(filled_w).min(dw);
            let y0 = 0usize;
            let y1 = unit_h.min(dh.saturating_sub(1));

            // Outer border of the unit.
            draw::hline(grid, x0, x1.saturating_sub(1), y0);
            draw::hline(grid, x0, x1.saturating_sub(1), y1);
            draw::vline(grid, x0, y0, y1);
            draw::vline(grid, x1.saturating_sub(1).min(dw.saturating_sub(1)), y0, y1);

            // Inner key: a hook shape entering from the top.
            if step >= 1 && unit_w >= 4 && unit_h >= 4 {
                let ix = (x0 + step).min(dw.saturating_sub(1));
                let iy0 = y0 + step;
                let iy1 = y1.saturating_sub(step);
                // Vertical down segment.
                draw::vline(grid, ix, iy0, iy1);
                // Horizontal right at the bottom of the inner key.
                let inner_right = (ix + step + step)
                    .min(x1.saturating_sub(1))
                    .min(dw.saturating_sub(1));
                draw::hline(grid, ix, inner_right, iy1);
                // Short vertical up.
                let rise = (iy1.saturating_sub(step)).max(iy0);
                draw::vline(grid, inner_right, rise, iy1);
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Seigaiha — Japanese overlapping wave scales filling row by row
// ─────────────────────────────────────────────────────────────────────────────

struct Seigaiha;
impl ProgressStyle for Seigaiha {
    fn name(&self) -> &str {
        "seigaiha"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Japanese seigaiha (blue ocean waves): overlapping semicircular scales tile \
         the bar bottom-up as progress rises; time ripples a subtle phase shift."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Each scale is a semicircle.  Scales tile in a brick offset pattern.
        let scale_r = (dh as f32 / 2.0).max(2.0).min(dw as f32 / 2.0);
        let scale_w = (scale_r * 2.0) as usize;
        let scale_h = scale_r as usize + 1;

        // Reveal from bottom to top.
        let eased_e = ease(Easing::CubicInOut, ctx.eased);
        let revealed_h = (eased_e * dh as f32).round() as usize;
        let y_threshold = dh.saturating_sub(revealed_h);

        // Phase animation — gentle horizontal drift.
        let phase_offset = (ctx.time * 0.4).sin() * scale_r * 0.1;

        let n_rows = (dh / scale_h.max(1) + 2).max(1);
        let n_cols = (dw / scale_w.max(1) + 2).max(1);

        for row in 0..n_rows {
            let cy = (dh as i32 - row as i32 * scale_h as i32) as f32;
            if cy < -(scale_r) {
                break;
            }

            for col in 0..n_cols {
                // Brick offset: odd rows shift by half a scale width.
                let offset = if row % 2 == 1 {
                    scale_w as f32 / 2.0
                } else {
                    0.0
                };
                let cx = col as f32 * scale_w as f32 + offset + phase_offset;

                // Draw semicircle (upper half only — flat edge down).
                let r = scale_r;
                let steps = (PI * r).ceil() as usize + 4;
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let angle = PI + t * PI; // from π to 2π (top half of unit circle)
                    let px = (cx + r * angle.cos()).round() as i32;
                    let py = (cy + r * angle.sin()).round() as i32;
                    if py >= 0 && (py as usize) >= y_threshold {
                        draw::dot_i(grid, px, py);
                    }
                }
                // Flat baseline of the scale.
                let py_base = cy.round() as i32;
                let x_left = (cx - r).round() as i32;
                let x_right = (cx + r).round() as i32;
                if py_base >= 0 && (py_base as usize) >= y_threshold {
                    for px in x_left..=x_right {
                        draw::dot_i(grid, px, py_base);
                    }
                }
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Totem Pole — carved segments stacking upward with progress
// ─────────────────────────────────────────────────────────────────────────────

struct TotemPole;
impl ProgressStyle for TotemPole {
    fn name(&self) -> &str {
        "totem-pole"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Totem pole: vertically stacked carved segments (eyes, beak, wings) build \
         upward cell by cell as progress rises; columns shimmer with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Each totem column is `seg_w` dots wide; they stand side by side.
        let seg_w = (dw / 5).max(2).min(dw);
        let n_cols = (dw / seg_w).max(1);

        // How many dot rows are revealed (growing from the bottom).
        let revealed = (ctx.eased * dh as f32).round() as usize;
        let y0_visible = dh.saturating_sub(revealed);

        for col in 0..n_cols {
            let x0 = col * seg_w;
            let x1 = (x0 + seg_w).min(dw);
            let cx = (x0 + x1) / 2;
            let cw_col = x1.saturating_sub(x0);

            // Outer column border.
            draw::vline(grid, x0, y0_visible, dh.saturating_sub(1));
            draw::vline(
                grid,
                x1.saturating_sub(1).min(dw.saturating_sub(1)),
                y0_visible,
                dh.saturating_sub(1),
            );

            // Segment height: divide the column into 4 face segments.
            let n_segs = 4usize;
            let seg_h = (dh / n_segs).max(1);

            for seg in 0..n_segs {
                let sy0 = seg * seg_h;
                let sy1 = ((seg + 1) * seg_h).min(dh);
                if sy0 < y0_visible {
                    continue;
                }
                let scy = (sy0 + sy1) / 2;

                // Each segment gets a different carved motif.
                match seg % 4 {
                    0 => {
                        // Eyes: two dots side by side.
                        let eye_y = scy.min(dh.saturating_sub(1));
                        let eye_lx = cx.saturating_sub(cw_col / 4).min(dw.saturating_sub(1));
                        let eye_rx = (cx + cw_col / 4).min(dw.saturating_sub(1));
                        draw::dot(grid, eye_lx, eye_y);
                        draw::dot(grid, eye_rx, eye_y);
                        // Eyebrow lines.
                        let brow_y = scy.saturating_sub(1).min(dh.saturating_sub(1));
                        draw::hline(
                            grid,
                            x0,
                            x1.saturating_sub(1).min(dw.saturating_sub(1)),
                            brow_y,
                        );
                    }
                    1 => {
                        // Beak: V-shape pointing downward.
                        let beak_top_y = sy0.max(y0_visible);
                        let beak_bot_y = scy.min(dh.saturating_sub(1));
                        bres(
                            grid,
                            x0 as i32,
                            beak_top_y as i32,
                            cx as i32,
                            beak_bot_y as i32,
                        );
                        bres(
                            grid,
                            x1.saturating_sub(1) as i32,
                            beak_top_y as i32,
                            cx as i32,
                            beak_bot_y as i32,
                        );
                    }
                    2 => {
                        // Wings: horizontal bars spreading from centre.
                        let wing_y = scy.min(dh.saturating_sub(1));
                        draw::hline(
                            grid,
                            x0,
                            x1.saturating_sub(1).min(dw.saturating_sub(1)),
                            wing_y,
                        );
                        // Wing tips — vlines up and down.
                        let tip_h = (seg_h / 3).max(1);
                        draw::vline(
                            grid,
                            x0,
                            wing_y.saturating_sub(tip_h),
                            (wing_y + tip_h).min(dh.saturating_sub(1)),
                        );
                        draw::vline(
                            grid,
                            x1.saturating_sub(1).min(dw.saturating_sub(1)),
                            wing_y.saturating_sub(tip_h),
                            (wing_y + tip_h).min(dh.saturating_sub(1)),
                        );
                    }
                    _ => {
                        // Base plaque: full width filled rectangle.
                        let plaque_y = sy1.saturating_sub(2).max(sy0).min(dh.saturating_sub(1));
                        draw::hline(
                            grid,
                            x0,
                            x1.saturating_sub(1).min(dw.saturating_sub(1)),
                            plaque_y,
                        );
                        draw::hline(
                            grid,
                            x0,
                            x1.saturating_sub(1).min(dw.saturating_sub(1)),
                            sy1.saturating_sub(1).min(dh.saturating_sub(1)),
                        );
                    }
                }

                // Segment divider.
                draw::hline(
                    grid,
                    x0,
                    x1.saturating_sub(1).min(dw.saturating_sub(1)),
                    sy0.min(dh.saturating_sub(1)),
                );
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Runes — Norse rune glyphs carved in one by one via draw::glyph
// ─────────────────────────────────────────────────────────────────────────────

struct Runes;
impl ProgressStyle for Runes {
    fn name(&self) -> &str {
        "runes"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Norse elder futhark runes: glyphs are carved into stone cells one by one \
         as progress rises; a flicker-shimmer via shade density pulses with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }

        // Elder Futhark runes (Unicode block U+16A0...).
        let runes: &[char] = &[
            'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ', 'ᛏ',
            'ᛒ', 'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛞ', 'ᛟ',
        ];

        let total_cells = cw * ch;
        let revealed = (ctx.eased * total_cells as f32).round() as usize;

        // Time-driven flicker: a shimmer wave travels left to right.
        let shimmer_x = ((ctx.time * 0.7).fract() * cw as f32) as usize;

        for idx in 0..revealed.min(total_cells) {
            let cx = idx % cw;
            let cy = idx / cw;
            let rune = runes[idx % runes.len()];

            // Near the shimmer wave: show a lighter shade character instead,
            // giving the impression of light reflecting off carved stone.
            let dist = (cx as i32 - shimmer_x as i32).unsigned_abs() as usize;
            if dist <= 1 {
                draw::shade(grid, cx, cy, 2); // ▒ — lit stone
            } else {
                draw::glyph(grid, cx, cy, rune);
            }
        }

        // Unfilled cells: bare stone background (light shade).
        for idx in revealed..total_cells {
            let cx = idx % cw;
            let cy = idx / cw;
            draw::shade(grid, cx, cy, 1); // ░
        }

        // Tint by column.
        for cy in 0..ch {
            for cx_c in 0..cw {
                let t = cx_c as f32 / cw.max(1) as f32;
                draw::tint_row(grid, cy, cx_c, cx_c, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Paisley Swirl — henna spirals unfurling from multiple seeds
// ─────────────────────────────────────────────────────────────────────────────

struct PaisleySwirl;
impl ProgressStyle for PaisleySwirl {
    fn name(&self) -> &str {
        "paisley-swirl"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Henna / paisley: teardrop spiral seeds unfurl across the bar as progress \
         rises; each swirl grows its own tight logarithmic coil, animated with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Number of paisley seeds tiling the bar.
        let n_seeds: usize = ((dw / (dh.max(1) * 2)).max(1)).min(12);
        let seed_w = dw / n_seeds.max(1);

        let eased_e = ease(Easing::QuadOut, ctx.eased);
        let max_turns = 2.5f32; // spiral coil turns at full progress.

        for s in 0..n_seeds {
            // Seed centre.
            let cx = s as f32 * seed_w as f32 + seed_w as f32 / 2.0;
            let cy = dh as f32 / 2.0;
            // Orientation alternates + time drift.
            let base_angle = if s % 2 == 0 { 0.0f32 } else { PI } + ctx.time * 0.3;

            // Max radius of this swirl — limited by seed cell height.
            let max_r = (dh as f32 / 2.0 - 1.0).max(1.0).min(seed_w as f32 / 2.0);

            // Teardrop outline: draw a small circle offset from the spiral tip.
            let tip_r = max_r * 0.25 * eased_e;
            let tip_cx = cx + (max_r * 0.6 * eased_e) * base_angle.cos();
            let tip_cy = cy + (max_r * 0.6 * eased_e) * base_angle.sin();
            if tip_r >= 1.0 {
                arc(grid, tip_cx, tip_cy, tip_r, 0.0, 2.0 * PI);
            }

            // Logarithmic spiral: r = a * e^(b*theta).
            // We parameterise so r goes from 0 to max_r over max_turns*2π.
            let theta_max = eased_e * max_turns * 2.0 * PI;
            let steps = (theta_max * max_r).ceil() as usize + 4;
            let a = max_r / (max_turns * 2.0 * PI).exp();
            let b = 1.0f32;

            for i in 0..=steps {
                let t = i as f32 / steps.max(1) as f32;
                let theta = t * theta_max;
                let r = a * (b * theta).exp();
                if r > max_r {
                    break;
                }
                let angle = theta + base_angle;
                let px = (cx + r * angle.cos()).round() as i32;
                let py = (cy + r * angle.sin()).round() as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Radial tint from the grid centre.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), ctx.palette.sample(t));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Kente Weave — warp-and-weft interlacing strips
// ─────────────────────────────────────────────────────────────────────────────

struct KenteWeave;
impl ProgressStyle for KenteWeave {
    fn name(&self) -> &str {
        "kente-weave"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Kente / tartan weave: vertical warp strips and horizontal weft strips \
         interlace in a floating-weave pattern; strips appear as progress rises and \
         the crossing pattern is animated by time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Strip width in dots (both warp and weft).
        let strip = ((dh / 4).max(1)).min(6).min(dw);
        let n_warp = (dw / (strip * 2).max(1)).max(1);
        let n_weft = (dh / (strip * 2).max(1)).max(1);

        let filled_w = (ctx.eased * dw as f32).round() as usize;
        let filled_h = (ctx.eased * dh as f32).round() as usize;

        // Time-driven phase shifts which strips appear to float over or under.
        let phase = (ctx.time * 0.5) as usize;

        // Draw warp (vertical) strips across the progress-filled width.
        for warp in 0..n_warp {
            let x0 = warp * strip * 2;
            if x0 >= filled_w {
                break;
            }
            let x1 = (x0 + strip).min(filled_w).min(dw);
            // Draw this vertical strip for its full height.
            for x in x0..x1 {
                for y in 0..dh {
                    // Leave gaps where weft floats over (cross-check with weft stripe).
                    let weft_idx = y / (strip * 2);
                    let in_weft = (y % (strip * 2)) < strip;
                    // Alternate which crosses on top using the phase.
                    let warp_on_top = (warp + weft_idx + phase) % 2 == 0;
                    if !in_weft || warp_on_top {
                        draw::dot(grid, x, y);
                    }
                }
            }
        }

        // Draw weft (horizontal) strips across the progress-filled height.
        for weft in 0..n_weft {
            let y0 = weft * strip * 2;
            if y0 >= filled_h {
                break;
            }
            let y1 = (y0 + strip).min(filled_h).min(dh);
            for y in y0..y1 {
                for x in 0..dw {
                    let warp_idx = x / (strip * 2);
                    let in_warp = (x % (strip * 2)) < strip;
                    let warp_on_top = (warp_idx + weft + phase) % 2 == 0;
                    // Weft draws only where it floats over warp, or in the gap.
                    if !in_warp || !warp_on_top {
                        draw::dot(grid, x, y);
                    }
                }
            }
        }

        // Tint warp and weft rows with contrasting palette samples.
        let (cw, ch) = grid.dimensions();
        for cy_c in 0..ch {
            let t = cy_c as f32 / ch.max(1) as f32;
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), ctx.palette.sample(t));
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
    let styles = progress::styles::cultures::styles();
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
