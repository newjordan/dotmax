//! `floweroflife` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O floweroflife.rs && ./floweroflife [style-name]
//! ```

const DEFAULT_STYLE: &str = "vesica-piscis";

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
    pub mod floweroflife {
//! Flower-of-Life sacred geometry progress styles.
//!
//! Eleven structurally distinct styles drawn from the Flower-of-Life family:
//! overlapping-circle constructions on a hexagonal lattice, their intersection
//! graphs (Metatron's Cube), the Kabbalistic Tree of Life, a 64-tetrahedron
//! triangular projection, and a time-rotating torus-flower. Every style
//! reveals itself as `ctx.eased` rises from 0 → 1 and breathes or rotates
//! via `ctx.time`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── shared geometry helpers ────────────────────────────────────────────────

/// Plot a circle (parametric) in dot-space.
/// `cx,cy` are the centre in dots (f32); `r` is the radius in dots.
/// Steps are bounded to avoid overdraw on tiny grids.
fn plot_circle(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    if r < 0.5 {
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
        return;
    }
    // Circumference ≈ 2πr; sample at ≥2 dots per step so we don't miss any dot.
    let steps = ((2.0 * PI * r).ceil() as usize * 2).max(8).min(2048);
    for i in 0..steps {
        let angle = 2.0 * PI * i as f32 / steps as f32;
        let px = cx + r * angle.cos();
        let py = cy + r * angle.sin();
        draw::dot_i(grid, px.round() as i32, py.round() as i32);
    }
}

/// Draw a line between two dot-space points using Bresenham-style stepping.
fn plot_line(grid: &mut BrailleGrid, x0: f32, y0: f32, x1: f32, y1: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()).ceil() as usize).max(1).min(4096);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let px = x0 + dx * t;
        let py = y0 + dy * t;
        draw::dot_i(grid, px.round() as i32, py.round() as i32);
    }
}

/// Hex-lattice unit vectors (6 directions, 60° apart).
fn hex_offset(ring_angle_index: usize, radius: f32) -> (f32, f32) {
    let a = ring_angle_index as f32 * PI / 3.0;
    (radius * a.cos(), radius * a.sin())
}

/// The 7 Seed-of-Life circle centres: centre + 6 around it at distance `r`.
/// Returned in order: [centre, 0°, 60°, 120°, 180°, 240°, 300°].
fn seed_centres(cx: f32, cy: f32, r: f32) -> [(f32, f32); 7] {
    let mut out = [(0f32, 0f32); 7];
    out[0] = (cx, cy);
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, r);
        out[i + 1] = (cx + dx, cy + dy);
    }
    out
}

/// All 19 Flower-of-Life centres: Seed (7) + outer ring (12).
fn flower_centres(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    let mut v: Vec<(f32, f32)> = Vec::with_capacity(19);
    // Inner 7
    for c in seed_centres(cx, cy, r) {
        v.push(c);
    }
    // Outer 12: two rings of 6 each at √3·r and 2·r, offset 30°
    for i in 0..6 {
        let a = i as f32 * PI / 3.0 + PI / 6.0;
        let d = 3f32.sqrt() * r;
        v.push((cx + d * a.cos(), cy + d * a.sin()));
    }
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, 2.0 * r);
        v.push((cx + dx, cy + dy));
    }
    v
}

/// 13 Fruit-of-Life centres: centre + 6 at r + 6 at 2r (same hex axes).
fn fruit_centres(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    let mut v: Vec<(f32, f32)> = Vec::with_capacity(13);
    v.push((cx, cy));
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, r);
        v.push((cx + dx, cy + dy));
    }
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, 2.0 * r);
        v.push((cx + dx, cy + dy));
    }
    v
}

// ── All styles ─────────────────────────────────────────────────────────────

/// All styles in the `floweroflife` theme.
///
/// Returns eleven structurally distinct sacred-geometry progress styles,
/// each revealing itself as `eased` rises and animating via `time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(VesicaPiscis),
        Box::new(SeedOfLife),
        Box::new(FlowerOfLife),
        Box::new(EggOfLife),
        Box::new(FruitOfLife),
        Box::new(MetatronsCube),
        Box::new(GermOfLife),
        Box::new(TripodOfLife),
        Box::new(TreeOfLife),
        Box::new(TetraGrid),
        Box::new(TorusFlower),
    ]
}

// ── 1. Vesica Piscis ───────────────────────────────────────────────────────

/// Two overlapping circles; the shared lens (vesica) forms as `eased` rises.
struct VesicaPiscis;
impl ProgressStyle for VesicaPiscis {
    fn name(&self) -> &str {
        "vesica-piscis"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Two circles overlap to reveal the sacred lens; arc segments appear with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) * 0.45).max(1.0);

        // Separation between the two circle centres, grows with eased.
        // At eased=0: coincident. At eased=1: separated by r (classic vesica).
        let sep = r * ctx.eased;
        let lx = cx - sep / 2.0;
        let rx = cx + sep / 2.0;

        // Draw the two circles, breathing via time.
        let breathe = 1.0 + 0.04 * (ctx.time * 1.2).sin();
        let rr = r * breathe;
        plot_circle(grid, lx, cy, rr);
        if ctx.eased > 0.05 {
            plot_circle(grid, rx, cy, rr);
        }

        // Color: tint the left half one shade, right half another.
        let (cw, ch) = grid.dimensions();
        let mid_cell = cw / 2;
        for cy_cell in 0..ch {
            if mid_cell > 0 {
                draw::tint_row(
                    grid,
                    cy_cell,
                    0,
                    mid_cell.saturating_sub(1),
                    ctx.palette.start,
                );
                draw::tint_row(
                    grid,
                    cy_cell,
                    mid_cell,
                    cw.saturating_sub(1),
                    ctx.palette.end,
                );
            }
        }
        Ok(())
    }
}

// ── 2. Seed of Life ────────────────────────────────────────────────────────

/// Central circle + 6 petals revealed one at a time; rotates with time.
struct SeedOfLife;
impl ProgressStyle for SeedOfLife {
    fn name(&self) -> &str {
        "seed-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Seven circles of the Seed of Life appear petal-by-petal as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) * 0.38).max(1.0);

        // Rotation offset from time.
        let rot = ctx.time * 0.25;
        // How many of the 7 circles to reveal (0 → 7).
        let reveal = (ctx.eased * 7.0).ceil() as usize;

        let centres = seed_centres(cx, cy, r);
        for (i, &(px, py)) in centres.iter().enumerate().take(reveal.min(7)) {
            // Rotate each petal centre around cx,cy.
            let (ox, oy) = (px - cx, py - cy);
            let rx_c = ox * rot.cos() - oy * rot.sin() + cx;
            let ry_c = ox * rot.sin() + oy * rot.cos() + cy;
            plot_circle(grid, rx_c, ry_c, r);

            // Tint by petal index.
            let t = i as f32 / 6.0;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (rx_c / 2.0).round() as usize;
            let cell_y = (ry_c / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }
        Ok(())
    }
}

// ── 3. Flower of Life ─────────────────────────────────────────────────────

/// Full 19-circle pattern with boundary ring; circles added as eased rises.
struct FlowerOfLife;
impl ProgressStyle for FlowerOfLife {
    fn name(&self) -> &str {
        "flower-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "All 19 overlapping circles of the Flower of Life bloom with progress, bounded by an outer ring"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // Fit: 19 circles span ≈3r radius from centre; outer ring at 2r.
        let r = (cx.min(cy) / 3.2).max(1.0);

        let centres = flower_centres(cx, cy, r);
        let total = centres.len(); // 19
        let reveal = (ctx.eased * total as f32).ceil() as usize;

        for (i, &(px, py)) in centres.iter().enumerate().take(reveal.min(total)) {
            plot_circle(grid, px, py, r);
            let t = i as f32 / (total - 1).max(1) as f32;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (px / 2.0).round() as usize;
            let cell_y = (py / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }

        // Outer boundary circle at 2r.
        if ctx.eased >= 0.95 {
            plot_circle(grid, cx, cy, 2.0 * r);
        }
        Ok(())
    }
}

// ── 4. Egg of Life ────────────────────────────────────────────────────────

/// 7 non-overlapping circles at alternate Flower-of-Life positions; pulses with time.
struct EggOfLife;
impl ProgressStyle for EggOfLife {
    fn name(&self) -> &str {
        "egg-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Seven separated circles in egg-of-life formation; radius pulses with time as progress fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // Egg of life: 7 circles placed at seed positions but non-overlapping (spacing=2r).
        let r = (cx.min(cy) * 0.22).max(1.0);
        let spacing = r * 2.2;

        let reveal = (ctx.eased * 7.0).ceil() as usize;
        let pulse = 1.0 + 0.06 * (ctx.time * 1.8).sin();

        let mut centres = [(0f32, 0f32); 7];
        centres[0] = (cx, cy);
        for i in 0..6 {
            let (dx, dy) = hex_offset(i, spacing);
            centres[i + 1] = (cx + dx, cy + dy);
        }

        for (i, &(px, py)) in centres.iter().enumerate().take(reveal.min(7)) {
            plot_circle(grid, px, py, r * pulse);
            // Fill inner dot for egg effect.
            draw::dot_i(grid, px.round() as i32, py.round() as i32);
            let t = i as f32 / 6.0;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (px / 2.0).round() as usize;
            let cell_y = (py / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }
        Ok(())
    }
}

// ── 5. Fruit of Life ──────────────────────────────────────────────────────

/// 13-circle pattern (the Fruit of Life); circles connected by lines as eased rises.
struct FruitOfLife;
impl ProgressStyle for FruitOfLife {
    fn name(&self) -> &str {
        "fruit-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "13 circles of the Fruit of Life; connecting lines drawn between all centres with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) / 2.5).max(1.0);
        let cr = (r * 0.18).max(1.0);

        let centres = fruit_centres(cx, cy, r);
        let total = centres.len(); // 13

        // Phase 1 (eased 0→0.5): reveal circles one by one.
        // Phase 2 (eased 0.5→1): draw connecting lines between all centres.
        let circle_frac = (ctx.eased * 2.0).min(1.0);
        let line_frac = ((ctx.eased - 0.5) * 2.0).max(0.0).min(1.0);

        let reveal_circles = (circle_frac * total as f32).ceil() as usize;
        for (i, &(px, py)) in centres.iter().enumerate().take(reveal_circles.min(total)) {
            plot_circle(grid, px, py, cr);
            let t = i as f32 / (total - 1).max(1) as f32;
            draw::tint_row(
                grid,
                (py / 4.0) as usize,
                (px / 2.0) as usize,
                (px / 2.0) as usize,
                ctx.palette.sample(t),
            );
        }

        // Connecting lines (all pairs = 78 edges).
        let all_pairs: Vec<(usize, usize)> = (0..total)
            .flat_map(|a| (a + 1..total).map(move |b| (a, b)))
            .collect();
        let reveal_lines = (line_frac * all_pairs.len() as f32).round() as usize;
        for &(a, b) in all_pairs.iter().take(reveal_lines) {
            let (ax, ay) = centres[a];
            let (bx, by) = centres[b];
            plot_line(grid, ax, ay, bx, by);
        }
        Ok(())
    }
}

// ── 6. Metatron's Cube ────────────────────────────────────────────────────

/// 13 Fruit-of-Life centres; ALL straight lines between them drawn with eased.
struct MetatronsCube;
impl ProgressStyle for MetatronsCube {
    fn name(&self) -> &str {
        "metatrons-cube"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Metatron's Cube: 13 nodes with all 78 connecting edges revealed progressively"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) / 2.4).max(1.0);
        // Slow rotation via time.
        let rot = ctx.time * 0.15;

        let raw = fruit_centres(cx, cy, r);
        // Rotate all centres around cx,cy.
        let centres: Vec<(f32, f32)> = raw
            .iter()
            .map(|&(px, py)| {
                let ox = px - cx;
                let oy = py - cy;
                (
                    ox * rot.cos() - oy * rot.sin() + cx,
                    ox * rot.sin() + oy * rot.cos() + cy,
                )
            })
            .collect();

        let total = centres.len();
        let all_pairs: Vec<(usize, usize)> = (0..total)
            .flat_map(|a| (a + 1..total).map(move |b| (a, b)))
            .collect();
        let reveal = (ctx.eased * all_pairs.len() as f32).round() as usize;

        for &(a, b) in all_pairs.iter().take(reveal) {
            let (ax, ay) = centres[a];
            let (bx, by) = centres[b];
            let t = a as f32 / (total - 1).max(1) as f32;
            let _ = t; // color applied per row via tint below
            plot_line(grid, ax, ay, bx, by);
        }

        // Draw node dots on top.
        let cr = 1.5f32.max(r * 0.12);
        for &(px, py) in &centres {
            plot_circle(grid, px, py, cr);
        }

        // Gradient tint across all columns.
        let (cw, ch) = grid.dimensions();
        for cy_cell in 0..ch {
            for cx_cell in 0..cw {
                let t = cx_cell as f32 / cw.max(1) as f32;
                draw::tint_row(grid, cy_cell, cx_cell, cx_cell, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── 7. Germ of Life ───────────────────────────────────────────────────────

/// First 7 circles of the Flower of Life with overlapping arcs shaded by eased.
/// Distinct from Seed-of-Life: draws ONLY the interior arcs (intersection petals),
/// not full circles — revealing the petal "germ" geometry.
struct GermOfLife;
impl ProgressStyle for GermOfLife {
    fn name(&self) -> &str {
        "germ-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Germ of Life: interior arc-petals of 7 overlapping circles fill in as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) * 0.38).max(1.0);
        let centres = seed_centres(cx, cy, r);

        // For each pair of adjacent outer circles (6 pairs), draw ONLY the arc
        // segment of circle i that lies within circle i+1. We approximate this
        // by plotting only angle ranges where the point is inside the neighbour.
        let reveal_petals = (ctx.eased * 6.0).ceil() as usize;

        // Draw the central circle fully.
        plot_circle(grid, cx, cy, r);

        for petal in 0..reveal_petals.min(6) {
            let (ox1, oy1) = (centres[petal + 1].0, centres[petal + 1].1);
            // Next petal (wrapping).
            let next = ((petal + 1) % 6) + 1;
            let (ox2, oy2) = (centres[next].0, centres[next].1);

            // Arc of circle at ox1,oy1 that passes through cx,cy region.
            let steps = 256usize;
            for i in 0..steps {
                let angle = 2.0 * PI * i as f32 / steps as f32;
                let px = ox1 + r * angle.cos();
                let py = oy1 + r * angle.sin();
                // Only draw if inside the central circle.
                let dc = (px - cx).hypot(py - cy);
                if dc <= r + 0.5 {
                    draw::dot_i(grid, px.round() as i32, py.round() as i32);
                }
            }

            // Also arc of ox2 inside ox1.
            for i in 0..steps {
                let angle = 2.0 * PI * i as f32 / steps as f32;
                let px = ox2 + r * angle.cos();
                let py = oy2 + r * angle.sin();
                let d1 = (px - ox1).hypot(py - oy1);
                if d1 <= r + 0.5 {
                    draw::dot_i(grid, px.round() as i32, py.round() as i32);
                }
            }

            let t = petal as f32 / 5.0;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let pcx = (ox1 / 2.0).round() as usize;
            let pcy = (oy1 / 4.0).round() as usize;
            if pcx < cw && pcy < ch {
                draw::tint_row(grid, pcy, pcx, pcx, color);
            }
        }
        Ok(())
    }
}

// ── 8. Tripod of Life ─────────────────────────────────────────────────────

/// Three-fold symmetry: 3 interlocking circles + 3-arm spoke structure.
/// Structurally distinct: uses 3-fold (not 6-fold) symmetry and builds a
/// branching tripod geometry inside the overlapping region.
struct TripodOfLife;
impl ProgressStyle for TripodOfLife {
    fn name(&self) -> &str {
        "tripod-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Tripod of Life: three interlocking circles with a central 3-arm spoke revealed by eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) * 0.42).max(1.0);

        let rot = ctx.time * 0.3;

        // 3 circle centres at 120° intervals.
        let mut circle_centres = [(0f32, 0f32); 3];
        for i in 0..3 {
            let a = rot + i as f32 * 2.0 * PI / 3.0;
            circle_centres[i] = (cx + r * 0.5 * a.cos(), cy + r * 0.5 * a.sin());
        }

        // Reveal circles (phase 1: eased 0→0.5), then spoke arms (phase 2: 0.5→1).
        let circle_frac = (ctx.eased * 2.0).min(1.0);
        let arm_frac = ((ctx.eased - 0.5) * 2.0).max(0.0).min(1.0);

        let reveal_circles = (circle_frac * 3.0).ceil() as usize;
        for i in 0..reveal_circles.min(3) {
            let (px, py) = circle_centres[i];
            plot_circle(grid, px, py, r);
            let color = ctx.palette.sample(i as f32 / 2.0);
            let (cw, ch) = grid.dimensions();
            let cell_x = (px / 2.0).round() as usize;
            let cell_y = (py / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }

        // Spoke arms from centre outward.
        for i in 0..3 {
            let a = rot + i as f32 * 2.0 * PI / 3.0;
            let arm_r = r * arm_frac;
            let ex = cx + arm_r * a.cos();
            let ey = cy + arm_r * a.sin();
            plot_line(grid, cx, cy, ex, ey);
        }

        // Central dot.
        if ctx.eased > 0.1 {
            draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
        }
        Ok(())
    }
}

// ── 9. Tree of Life ───────────────────────────────────────────────────────

/// Kabbalistic Tree of Life: 10 sephirot nodes + 22 connecting paths.
/// Paths drawn progressively with eased; nodes always visible.
struct TreeOfLife;
impl ProgressStyle for TreeOfLife {
    fn name(&self) -> &str {
        "tree-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Kabbalistic Tree of Life: 10 sephirot nodes with 22 paths revealed as eased rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;

        // Scale so the tree fits: spans 6 units tall, 4 wide.
        let sx = (dw as f32 / 4.5).min(dh as f32 / 6.5).max(1.0);
        let sy = sx;

        // Sephirot positions (normalized, y increasing downward).
        // Classical layout:  1=Kether(top), 10=Malkuth(bottom).
        let nodes: [(f32, f32); 10] = [
            (0.0, -3.0),  // 0 Kether
            (-1.0, -2.0), // 1 Chokmah
            (1.0, -2.0),  // 2 Binah
            (-1.0, -1.0), // 3 Chesed
            (1.0, -1.0),  // 4 Geburah
            (0.0, 0.0),   // 5 Tiphareth
            (-1.0, 1.0),  // 6 Netzach
            (1.0, 1.0),   // 7 Hod
            (0.0, 2.0),   // 8 Yesod
            (0.0, 3.0),   // 9 Malkuth
        ];

        // Convert to dot-space.
        let dot_nodes: Vec<(f32, f32)> = nodes
            .iter()
            .map(|&(x, y)| (cx + x * sx, cy + y * sy))
            .collect();

        // 22 paths of the Tree of Life (traditional Sefer Yetzirah connections).
        let paths: [(usize, usize); 22] = [
            (0, 1),
            (0, 2),
            (0, 5), // Kether connections
            (1, 2),
            (1, 3),
            (1, 5), // Chokmah
            (2, 4),
            (2, 5), // Binah
            (3, 4),
            (3, 5),
            (3, 6), // Chesed
            (4, 5),
            (4, 7), // Geburah
            (5, 6),
            (5, 7),
            (5, 8), // Tiphareth
            (6, 7),
            (6, 8), // Netzach
            (7, 8), // Hod
            (8, 9), // Yesod-Malkuth
            (1, 4),
            (2, 3), // cross-paths (Paroketh)
        ];

        let reveal = (ctx.eased * paths.len() as f32).round() as usize;
        for (i, &(a, b)) in paths.iter().enumerate().take(reveal) {
            let (ax, ay) = dot_nodes[a];
            let (bx, by) = dot_nodes[b];
            plot_line(grid, ax, ay, bx, by);
            let t = i as f32 / (paths.len() - 1).max(1) as f32;
            let _ = t;
        }

        // Draw sephirot nodes as small circles.
        let nr = (sx * 0.25).max(1.0);
        for (i, &(px, py)) in dot_nodes.iter().enumerate() {
            plot_circle(grid, px, py, nr);
            let t = i as f32 / 9.0;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (px / 2.0).round() as usize;
            let cell_y = (py / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }
        Ok(())
    }
}

// ── 10. 64-Tetrahedron Grid ───────────────────────────────────────────────

/// 2D projection of the 64-tetrahedron grid as a triangular lattice.
/// Structurally different: pure triangular grid, no circles.
struct TetraGrid;
impl ProgressStyle for TetraGrid {
    fn name(&self) -> &str {
        "64-tetra-grid"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "64-tetrahedron grid: equilateral triangular lattice revealed by progress, sweeping in from left"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;

        // Triangular lattice basis vectors (equilateral triangles).
        // a = (1, 0), b = (0.5, √3/2), scaled to fit.
        let cell_size = (dw.min(dh) as f32 / 8.0).max(2.0);
        let ax = cell_size;
        let ay = 0.0f32;
        let bx = cell_size * 0.5;
        let by = cell_size * (3f32.sqrt() / 2.0);

        // Generate lattice points in a grid range.
        let range = 6i32;
        let mut points: Vec<(f32, f32)> = Vec::new();
        for i in -range..=range {
            for j in -range..=range {
                let px = cx + i as f32 * ax + j as f32 * bx;
                let py = cy + i as f32 * ay + j as f32 * by;
                // Only keep if within dot bounds.
                if px >= 0.0 && px < dw as f32 && py >= 0.0 && py < dh as f32 {
                    points.push((px, py));
                }
            }
        }

        // Reveal point by point from left to right (x-sorted).
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let reveal = (ctx.eased * points.len() as f32).round() as usize;

        // Draw the triangular edges between adjacent lattice points.
        // For each revealed point, draw edges to its right-neighbours.
        for &(px, py) in points.iter().take(reveal) {
            // The six neighbour offsets in triangular lattice.
            let neighbors: [(f32, f32); 6] = [
                (ax, ay),
                (-ax, -ay),
                (bx, by),
                (-bx, -by),
                (ax - bx, ay - by),
                (-(ax - bx), -(ay - by)),
            ];
            draw::dot_i(grid, px.round() as i32, py.round() as i32);
            for (dx, dy) in neighbors {
                let nx = px + dx;
                let ny = py + dy;
                if nx >= 0.0 && nx < dw as f32 && ny >= 0.0 && ny < dh as f32 {
                    // Only draw if neighbour is also revealed.
                    let idx = points.partition_point(|&(qx, _)| qx < nx - 0.5);
                    let in_revealed = points[..idx.min(reveal)]
                        .iter()
                        .any(|&(qx, qy)| (qx - nx).abs() < 1.0 && (qy - ny).abs() < 1.0);
                    if in_revealed {
                        plot_line(grid, px, py, nx, ny);
                    }
                }
            }
        }

        // Color gradient left-to-right.
        let (cw, ch) = grid.dimensions();
        for cy_cell in 0..ch {
            for cx_cell in 0..cw {
                let t = cx_cell as f32 / cw.max(1) as f32;
                draw::tint_row(grid, cy_cell, cx_cell, cx_cell, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── 11. Torus Flower ──────────────────────────────────────────────────────

/// Flower-of-Life circles warped onto a toroidal projection, rotating with time.
/// Structurally distinct: each circle centre is displaced by a sinusoidal
/// torus-warp offset derived from its lattice angle, creating a rolling,
/// depth-suggesting motion entirely absent from the flat flower.
struct TorusFlower;
impl ProgressStyle for TorusFlower {
    fn name(&self) -> &str {
        "torus-flower"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Flower-of-Life circles warped onto a torus surface; the whole pattern rolls with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) / 3.2).max(1.0);

        // Torus parameters: major radius R (from centre to tube centre),
        // minor radius ru (tube radius).
        let big_r = cx.min(cy) * 0.55;
        let small_r = cx.min(cy) * 0.2;

        // Time drives toroidal rotation angle phi.
        let phi = ctx.time * 0.5;

        let raw = flower_centres(cx, cy, r);
        let total = raw.len();
        let reveal = (ctx.eased * total as f32).ceil() as usize;

        for (i, &(px, py)) in raw.iter().enumerate().take(reveal.min(total)) {
            // Map flat (px,py) → polar angle θ relative to centre.
            let theta = (py - cy).atan2(px - cx);
            let dist_frac = (px - cx).hypot(py - cy) / (2.0 * r).max(1.0);

            // Torus warp: displace each centre by a depth offset derived from
            // mapping theta → torus surface.
            let torus_x_offset = big_r
                * (theta + phi).cos()
                * (1.0 - small_r / big_r * (dist_frac * PI * 2.0 + phi).cos());
            let torus_y_offset = small_r * (dist_frac * PI * 2.0 + phi).sin();

            // Blend the flat position with the torus-warped position.
            let warp = 0.3;
            let wpx = (1.0 - warp) * px + warp * (cx + torus_x_offset * (r / big_r.max(1.0)));
            let wpy = (1.0 - warp) * py + warp * (cy + torus_y_offset * 2.0);

            // Scale circle radius by apparent depth (simulating perspective).
            let depth = 1.0 + 0.25 * (dist_frac * PI * 2.0 + phi).cos();
            let cr = (r * depth).max(1.0);

            plot_circle(grid, wpx, wpy, cr);

            let t = i as f32 / (total - 1).max(1) as f32;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (wpx / 2.0).round() as usize;
            let cell_y = (wpy / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
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
    let styles = progress::styles::floweroflife::styles();
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
