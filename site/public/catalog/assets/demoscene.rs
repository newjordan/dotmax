//! `demoscene` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O demoscene.rs && ./demoscene [style-name]
//! ```

const DEFAULT_STYLE: &str = "copper-bars";

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
    pub mod demoscene {
//! Demoscene progress bars — copper bars, plasma, rotozoom, sine scrollers.
//!
//! A love letter to the Amiga and C64 crack-intro canon: every style is a
//! classic real-time effect reimagined as a loading indicator. Rainbow
//! palettes cycle, rasters sweep, checkerboards spin. All of it is a pure
//! function of `(progress, time)` and loops seamlessly every four seconds.

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

// ─── palette ────────────────────────────────────────────────────────────────

/// Cycle the classic full-saturation rainbow; `t` wraps at 1.0.
fn spectrum(t: f32) -> Color {
    let t = t.rem_euclid(1.0);
    let chan = |off: f32| {
        let v = 0.5 + 0.5 * (TAU * (t + off)).cos();
        (60.0 + 195.0 * v) as u8
    };
    Color::rgb(chan(0.0), chan(2.0 / 3.0), chan(1.0 / 3.0))
}

/// Blend two colors at `t` in `0.0..=1.0`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let l = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(l(a.r, b.r), l(a.g, b.g), l(a.b, b.b))
}

/// Dim track gray-blue, for unfilled remainders.
const DS_TRACK: Color = Color::rgb(70, 70, 96);
/// White for crisp readout edges.
const DS_WHITE: Color = Color::rgb(244, 244, 252);

/// All styles in the `demoscene` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(CopperBars),
        Box::new(Plasma),
        Box::new(Rotozoom),
        Box::new(SineScroller),
        Box::new(RasterBars),
        Box::new(Twister),
        Box::new(Metaballs),
        Box::new(Tunnel),
        Box::new(Moire),
        Box::new(BobParade),
    ]
}

/// Bouncing copper gradient bars — one more joins for every sixth of progress.
struct CopperBars;
impl ProgressStyle for CopperBars {
    fn name(&self) -> &str {
        "copper-bars"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Amiga copper bars joining the bounce one by one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        // Only four two-row bars fit a sixteen-dot canvas with air between
        // them — more and the bounce reads as a solid wall.
        let total = 4;
        let visible = 1 + (ctx.eased * (total as f32 - 0.01)) as i32;
        for i in 0..visible {
            // Quarter-hertz phases keep the 4s loop seamless. Bars bounce in
            // the band above the underline so the progress read stays clean.
            let phase = TAU * (ctx.time * 0.5 + i as f32 / 4.0);
            let y0 = (0.5 + 0.5 * phase.sin()) * (h as f32 - 6.0);
            let hue = i as f32 / total as f32;
            for dy in 0..2i32 {
                let y = y0 as i32 + dy;
                for x in 0..w as i32 {
                    draw::dot_i(grid, x, y);
                }
                // Bright top scanline gives the metallic sheen.
                let c = if dy == 0 {
                    mix(spectrum(hue), DS_WHITE, 0.55)
                } else {
                    spectrum(hue)
                };
                if y >= 0 && (y as usize) < h {
                    draw::tint_row(grid, y as usize / 4, 0, ctx.width - 1, c);
                }
            }
        }
        // Crisp progress underline so the read is exact: dotted track,
        // solid two-row fill.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 2);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 2);
            draw::dot(grid, x, h - 1);
        }
        draw::tint_row(grid, ctx.height - 1, 0, ctx.width - 1, DS_WHITE);
        Ok(())
    }
}

/// Old-school plasma revealed left to right, palette forever cycling.
struct Plasma;
impl ProgressStyle for Plasma {
    fn name(&self) -> &str {
        "plasma"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Palette-cycling plasma pouring in from the left"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let t = ctx.time;
        let field = |x: f32, y: f32| {
            (x * 0.24 + TAU * 0.25 * t).sin()
                + (y * 0.55 - TAU * 0.5 * t).sin()
                + ((x * 0.14 + y * 0.31) + TAU * 0.25 * t).sin()
        };
        for y in 0..h {
            for x in 0..filled {
                let v = field(x as f32, y as f32);
                if v > -0.9 {
                    draw::dot(grid, x, y);
                }
            }
        }
        // Sparse dim dots mark the unfilled remainder.
        for y in (0..h).step_by(4) {
            for x in (filled..w).step_by(4) {
                draw::dot(grid, x, y);
            }
        }
        // Color cells by the field at their center, palette slowly cycling.
        for cy in 0..ctx.height {
            for cx in 0..ctx.width {
                if cx * 2 < filled {
                    let v = field(cx as f32 * 2.0 + 1.0, cy as f32 * 4.0 + 2.0);
                    let c = spectrum(v / 6.0 + 0.5 + t * 0.25);
                    let _ = grid.set_cell_color(cx, cy, c);
                } else {
                    let _ = grid.set_cell_color(cx, cy, DS_TRACK);
                }
            }
        }
        Ok(())
    }
}

/// A spinning, breathing checkerboard opens out from the center like an iris.
struct Rotozoom;
impl ProgressStyle for Rotozoom {
    fn name(&self) -> &str {
        "rotozoom"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Rotozooming checkerboard revealed by a growing iris"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let max_r = (cx * cx + cy * cy).sqrt();
        // A floor keeps the iris visible from the very first percent.
        let radius = (0.08 + 0.92 * ctx.eased) * max_r;
        let ang = TAU * 0.25 * ctx.time;
        let zoom = 1.3 + 0.5 * (TAU * 0.25 * ctx.time).sin();
        let (sa, ca) = ang.sin_cos();
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx;
                let dy = (y as f32 - cy) * 2.0; // braille dots are taller than wide
                let r = (dx * dx + dy * dy).sqrt();
                if r > radius {
                    continue;
                }
                let u = (dx * ca - dy * sa) * zoom;
                let v = (dx * sa + dy * ca) * zoom;
                let tile = ((u / 7.0).floor() as i32 + (v / 7.0).floor() as i32).rem_euclid(2);
                if tile == 0 {
                    draw::dot(grid, x, y);
                }
                let hue = hash2((u / 7.0).floor() as i32, (v / 7.0).floor() as i32);
                let _ = grid.set_cell_color(x / 2, y / 4, spectrum(hue * 0.4 + ctx.time * 0.25));
            }
        }
        // Bright iris rim makes the progress edge crisp.
        if radius > 2.0 && ctx.eased < 1.0 {
            let steps = (radius * 4.0) as i32;
            for s in 0..steps {
                let a = TAU * s as f32 / steps.max(1) as f32;
                let x = cx + a.cos() * radius;
                let y = cy + a.sin() * radius * 0.5;
                draw::dot_i(grid, x as i32, y as i32);
                let _ = grid.set_cell_color(
                    (x as usize / 2).min(ctx.width - 1),
                    (y.max(0.0) as usize / 4).min(ctx.height - 1),
                    DS_WHITE,
                );
            }
        }
        Ok(())
    }
}

/// The classic rainbow sine scroller, riding above a crisp baseline bar.
struct SineScroller;
impl ProgressStyle for SineScroller {
    fn name(&self) -> &str {
        "sine-scroller"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Rainbow marquee text on a sine wave"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        grid.enable_color_support();
        let cw = ctx.width;
        // Build a marquee whose length divides the scroll distance so the
        // 4s loop wraps without a seam (24 cells at 6 cells/sec).
        let label = ctx.label.clone().unwrap_or_default();
        let mut text: Vec<char> = format!(" LOADING {label} · DOTMAX ·").chars().collect();
        text.truncate(24);
        while text.len() < 24 {
            text.push(' ');
        }
        let scroll = (ctx.time * 6.0) as i32;
        for sx in 0..cw as i32 {
            let idx = (sx + scroll).rem_euclid(24) as usize;
            let c = text[idx];
            if c == ' ' {
                continue;
            }
            let wave = (TAU * (sx as f32 * 0.028 + ctx.time * 0.5)).sin();
            let cy = ((1.0 + wave) * 0.5 * (ctx.height as f32 - 2.05)) as usize;
            draw::glyph(grid, sx as usize, cy, c);
            let hue = sx as f32 / cw as f32 + ctx.time * 0.5;
            let _ = grid.set_cell_color(sx as usize, cy, spectrum(hue));
        }
        // Baseline progress bar in dots along the bottom.
        let (w, h) = draw::dot_dims(grid);
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 2);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 2);
            draw::dot(grid, x, h - 1);
        }
        for cx in 0..cw {
            if grid.get_char(cx, ctx.height - 1) != '\u{2800}' {
                let c = if cx * 2 < filled {
                    spectrum(cx as f32 / cw as f32 + ctx.time * 0.5)
                } else {
                    DS_TRACK
                };
                let _ = grid.set_cell_color(cx, ctx.height - 1, c);
            }
        }
        Ok(())
    }
}

/// Rainbow rasters sweep behind a crisp framed progress bar.
struct RasterBars;
impl ProgressStyle for RasterBars {
    fn name(&self) -> &str {
        "raster-bars"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Raster interrupt rainbows behind a framed bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let top = (h / 2).saturating_sub(3);
        let bot = (h / 2 + 3).min(h - 1);
        // Framed bar first — it owns its cells.
        draw::rect_outline(grid, 0, top, w, bot - top + 1);
        let filled = ((ctx.eased * (w as f32 - 4.0)).max(0.0) as usize).min(w.saturating_sub(4));
        for y in (top + 2)..=bot.saturating_sub(2) {
            for x in 2..(2 + filled) {
                draw::dot(grid, x, y);
            }
        }
        let bar_cell_top = top / 4;
        let bar_cell_bot = bot / 4;
        // Raster bars sweep the rows above and below, painting only cells the
        // frame doesn't own; inside the frame they tint the fill hue instead.
        for i in 0..3i32 {
            let phase = TAU * (ctx.time * 0.25 + i as f32 / 3.0);
            let y0 = ((0.5 + 0.5 * phase.sin()) * (h as f32 - 3.0)) as usize;
            let hue = i as f32 / 3.0 + ctx.time * 0.25;
            for dy in 0..3usize {
                let y = (y0 + dy).min(h - 1);
                let cy = y / 4;
                if cy >= bar_cell_top && cy <= bar_cell_bot {
                    continue;
                }
                // Half-tone texture keeps rasters visually behind the solid bar.
                for x in ((y % 2)..w).step_by(2) {
                    draw::dot(grid, x, y);
                }
                let c = if dy == 1 {
                    mix(spectrum(hue), DS_WHITE, 0.5)
                } else {
                    spectrum(hue)
                };
                draw::tint_row(grid, cy, 0, ctx.width - 1, c);
            }
        }
        // Bar tint: white frame rows, cycling fill.
        for cy in bar_cell_top..=bar_cell_bot {
            draw::tint_row(grid, cy, 0, ctx.width - 1, DS_WHITE);
        }
        if filled > 0 {
            let mid_cell = (top + 2) / 4;
            for cx in 1..=((2 + filled) / 2).min(ctx.width.saturating_sub(2)) {
                let _ =
                    grid.set_cell_color(cx, mid_cell, spectrum(cx as f32 * 0.02 + ctx.time * 0.5));
            }
        }
        Ok(())
    }
}

/// A ribbon that twists around its own axis as it grows across the bar.
struct Twister;
impl ProgressStyle for Twister {
    fn name(&self) -> &str {
        "twister"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Twisting ribbon column growing with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let mid = h as f32 / 2.0;
        let amp = mid - 1.5;
        let filled = (ctx.eased * w as f32).round() as usize;
        // Dim center track for the remainder.
        for x in (filled..w).step_by(3) {
            draw::dot(grid, x, mid as usize);
        }
        for cy in 0..ctx.height {
            draw::tint_row(grid, cy, 0, ctx.width - 1, DS_TRACK);
        }
        for x in 0..filled {
            let twist = x as f32 * 0.085 + TAU * 0.5 * ctx.time;
            // Four ribbon edges; consecutive pairs that face us get filled.
            let mut edges = [0f32; 4];
            for (k, e) in edges.iter_mut().enumerate() {
                *e = mid + amp * (twist + k as f32 * TAU / 4.0).sin();
            }
            for k in 0..4 {
                let (a, b) = (edges[k], edges[(k + 1) % 4]);
                if a < b {
                    // Front face: fill between edges, hue by face index.
                    for y in (a as i32)..=(b as i32) {
                        draw::dot_i(grid, x as i32, y);
                    }
                    let hue = k as f32 / 4.0 + 0.1;
                    let shade = ((b - a) / (2.0 * amp)).clamp(0.15, 1.0);
                    let cell_y = (((a + b) / 2.0) as usize / 4).min(ctx.height - 1);
                    let _ = grid.set_cell_color(
                        x / 2,
                        cell_y,
                        mix(Color::rgb(20, 20, 30), spectrum(hue), 0.35 + 0.65 * shade),
                    );
                }
            }
            // Crisp rims top and bottom of the silhouette.
            let lo = edges.iter().fold(f32::MAX, |m, &v| m.min(v));
            let hi = edges.iter().fold(f32::MIN, |m, &v| m.max(v));
            draw::dot_i(grid, x as i32, lo as i32);
            draw::dot_i(grid, x as i32, hi as i32);
        }
        Ok(())
    }
}

/// Blobs that merge and split, roaming exactly as far as progress allows.
struct Metaballs;
impl ProgressStyle for Metaballs {
    fn name(&self) -> &str {
        "metaballs"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Metaballs roaming the filled region"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let reach = (ctx.eased * w as f32).max(10.0);
        let t = ctx.time;
        // Three blobs on closed Lissajous orbits inside [0, reach].
        let balls = [
            (
                reach * (0.5 + 0.38 * (TAU * (t * 0.25)).sin()),
                h as f32 * (0.45 + 0.22 * (TAU * (t * 0.5 + 0.25)).sin()),
                5.6f32,
            ),
            (
                reach * (0.5 + 0.4 * (TAU * (t * 0.5 + 0.6)).sin()),
                h as f32 * (0.45 + 0.24 * (TAU * (t * 0.25 + 0.5)).cos()),
                4.6f32,
            ),
            (
                reach * (0.5 + 0.3 * (TAU * (t * 0.75 + 0.35)).cos()),
                h as f32 * (0.45 + 0.2 * (TAU * (t * 0.5)).sin()),
                3.8f32,
            ),
        ];
        let field = |x: f32, y: f32| {
            balls
                .iter()
                .map(|&(bx, by, r)| {
                    let dx = x - bx;
                    let dy = (y - by) * 1.8;
                    r * r / (dx * dx + dy * dy + 0.6)
                })
                .sum::<f32>()
        };
        for y in 0..h {
            for x in 0..(reach as usize).min(w) {
                if field(x as f32, y as f32) > 0.85 {
                    draw::dot(grid, x, y);
                }
            }
        }
        // Slime tint: hot core → green rim by field strength.
        for cy in 0..ctx.height {
            for cx in 0..ctx.width {
                let v = field(cx as f32 * 2.0 + 1.0, cy as f32 * 4.0 + 2.0);
                if v > 0.85 {
                    let hot = ((v - 0.85) / 2.0).clamp(0.0, 1.0);
                    let _ = grid.set_cell_color(
                        cx,
                        cy,
                        mix(Color::rgb(60, 220, 130), Color::rgb(235, 255, 200), hot),
                    );
                } else {
                    let _ = grid.set_cell_color(cx, cy, DS_TRACK);
                }
            }
        }
        // Baseline read.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 1);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
        }
        Ok(())
    }
}

/// A checkered tunnel revealed by a clock sweep, forever rushing inward.
struct Tunnel;
impl ProgressStyle for Tunnel {
    fn name(&self) -> &str {
        "tunnel"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Rushing tunnel revealed by a radial sweep"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let cx = w as f32 / 2.0 + 5.0 * (TAU * 0.25 * ctx.time).sin();
        let cy = h as f32 / 2.0 + 2.0 * (TAU * 0.25 * ctx.time).cos();
        let sweep = ctx.eased * TAU;
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx;
                let dy = (y as f32 - cy) * 2.0;
                let ang = dy.atan2(dx).rem_euclid(TAU);
                if ang > sweep {
                    continue;
                }
                let r = (dx * dx + dy * dy).sqrt().max(1.5);
                let u = 26.0 / r + ctx.time * 1.0;
                // Pure rushing rings — angle drives only the color, which
                // keeps the monochrome read clean.
                if (u.floor() as i32).rem_euclid(2) == 0 {
                    draw::dot(grid, x, y);
                }
                // Deeper (smaller r) is darker; hue rides the ring index.
                let depth = (r / (w as f32 * 0.5)).clamp(0.0, 1.0);
                let c = mix(
                    Color::rgb(18, 18, 30),
                    spectrum(u.floor() * 0.11 + ctx.time * 0.25),
                    0.25 + 0.75 * depth,
                );
                let _ = grid.set_cell_color(x / 2, y / 4, c);
            }
        }
        // Bright sweep edge so the progress read stays crisp.
        if ctx.eased > 0.02 && ctx.eased < 0.995 {
            let steps = 40;
            for s in 0..steps {
                let rr = 2.0 + s as f32;
                let x = cx + sweep.cos() * rr;
                let y = cy + sweep.sin() * rr * 0.5;
                if x < 0.0 || x >= w as f32 || y < 0.0 || y >= h as f32 {
                    break;
                }
                draw::dot_i(grid, x as i32, y as i32);
                let _ = grid.set_cell_color(
                    (x as usize / 2).min(ctx.width - 1),
                    (y as usize / 4).min(ctx.height - 1),
                    DS_WHITE,
                );
            }
        }
        Ok(())
    }
}

/// Two expanding ring systems interfere; the fringes crawl as they separate.
struct Moire;
impl ProgressStyle for Moire {
    fn name(&self) -> &str {
        "moire"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Interference rings crawling in from the left"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let filled = (ctx.eased * w as f32).round() as usize;
        let mid = h as f32 / 2.0;
        let sep = 12.0 + 7.0 * (TAU * 0.25 * ctx.time).sin();
        let (ax, ay) = (w as f32 / 2.0 - sep, mid);
        let (bx, by) = (w as f32 / 2.0 + sep, mid);
        let grow = ctx.time * 4.0; // rings expand 4 dots/sec; 16 over the loop
        for y in 0..h {
            for x in 0..filled {
                let d1 = {
                    let dx = x as f32 - ax;
                    let dy = (y as f32 - ay) * 2.0;
                    (dx * dx + dy * dy).sqrt()
                };
                let d2 = {
                    let dx = x as f32 - bx;
                    let dy = (y as f32 - by) * 2.0;
                    (dx * dx + dy * dy).sqrt()
                };
                let ring = ((d1 - grow) / 4.0).floor() as i32 + ((d2 - grow) / 4.0).floor() as i32;
                if ring.rem_euclid(2) == 0 {
                    draw::dot(grid, x, y);
                }
                if x % 2 == 0 && y % 4 == 0 {
                    // Fringe hue follows the path difference.
                    let fringe = ((d1 - d2) / 10.0).rem_euclid(1.0);
                    let _ = grid.set_cell_color(x / 2, y / 4, spectrum(fringe + ctx.time * 0.25));
                }
            }
        }
        // Track dots ahead of the reveal.
        for x in (filled..w).step_by(4) {
            draw::dot(grid, x, mid as usize);
            let _ = grid.set_cell_color(x / 2, (mid as usize) / 4, DS_TRACK);
        }
        Ok(())
    }
}

/// A parade of glowing blitter bobs joins the loop one per tenth of progress.
struct BobParade;
impl ProgressStyle for BobParade {
    fn name(&self) -> &str {
        "bob-parade"
    }
    fn theme(&self) -> &str {
        "demoscene"
    }
    fn describe(&self) -> &str {
        "Blitter bobs joining a Lissajous parade"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        grid.enable_color_support();
        let total = 10;
        let visible = (1.0 + ctx.eased * (total as f32 - 1.0)).round() as i32;
        for i in (0..visible).rev() {
            // Trail follows the leader at fixed phase lag on a closed curve.
            let phi = TAU * (ctx.time * 0.25) - i as f32 * 0.22;
            let x = w as f32 * (0.5 + 0.42 * (2.0 * phi).sin());
            let y = h as f32 * (0.5 + 0.34 * (3.0 * phi + 1.3).sin());
            let r = if i == 0 { 3i32 } else { 2i32 };
            for dy in -r..=r {
                for dx in -(r * 2)..=(r * 2) {
                    if dx * dx + dy * dy * 4 <= r * r * 4 {
                        draw::dot_i(grid, x as i32 + dx, y as i32 + dy);
                    }
                }
            }
            let hue = i as f32 / total as f32 + ctx.time * 0.25;
            let c = if i == 0 {
                mix(spectrum(hue), DS_WHITE, 0.6)
            } else {
                spectrum(hue)
            };
            let cell = (
                (x as usize / 2).min(ctx.width - 1),
                (y as usize / 4).min(ctx.height - 1),
            );
            for dcx in -1..=1i32 {
                let cxp = cell.0 as i32 + dcx;
                if cxp >= 0 && (cxp as usize) < ctx.width {
                    let _ = grid.set_cell_color(cxp as usize, cell.1, c);
                }
            }
        }
        // Baseline read.
        let filled = (ctx.eased * w as f32).round() as usize;
        for x in (0..w).step_by(3) {
            draw::dot(grid, x, h - 1);
        }
        for x in 0..filled {
            draw::dot(grid, x, h - 1);
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
    let styles = progress::styles::demoscene::styles();
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
