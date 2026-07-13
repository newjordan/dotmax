//! `gadgets` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O gadgets.rs && ./gadgets [style-name]
//! ```

const DEFAULT_STYLE: &str = "phone-battery";

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
    pub mod gadgets {
//! Gadgets / consumer-tech progress bars — device UIs in braille.
//!
//! Every style mimics a familiar consumer device interaction: phone charging,
//! WiFi connecting, Bluetooth pairing, disk defragging, CRT power-on, dial-up
//! modem handshake, vinyl spinning up, drone propellers, an activity ring,
//! USB file transfer, a gear train, and an e-ink page refresh. All are
//! stateless — animation is driven entirely by `ctx.time` and `ctx.eased`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ─── tiny deterministic hash ─────────────────────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

/// All styles in the `gadgets` theme.
///
/// Returns 12 structurally distinct consumer-device progress bar
/// implementations, each stateless and animatable via `ctx.time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PhoneBattery),
        Box::new(WifiConnect),
        Box::new(BluetoothPair),
        Box::new(DiskDefrag),
        Box::new(CrtPowerOn),
        Box::new(DialUpModem),
        Box::new(VinylSpinUp),
        Box::new(DroneProps),
        Box::new(ActivityRing),
        Box::new(UsbTransfer),
        Box::new(GearTrain),
        Box::new(EinkRefresh),
    ]
}

// ─── 1. Smartphone battery charging ──────────────────────────────────────────

/// Battery outline that fills with lightning bolt and rising charge bubbles.
struct PhoneBattery;
impl ProgressStyle for PhoneBattery {
    fn name(&self) -> &str {
        "phone-battery"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Smartphone battery outline fills with charge; lightning bolt pulses"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Battery body outline — leave a 2-dot border.
        // "Terminal nub" on the right: a 2-dot-wide bump.
        let nub_w = (w / 16).max(1);
        let body_w = w.saturating_sub(nub_w + 2);
        let body_h = h.saturating_sub(2).max(1);
        let bx = 0usize;
        let by = 1usize;

        draw::rect_outline(grid, bx, by, body_w.max(2), body_h.max(2));

        // Nub
        let nub_y0 = by + body_h / 4;
        let nub_h = body_h / 2;
        draw::fill_rect(grid, bx + body_w, nub_y0, nub_w, nub_h.max(1));

        // Fill level inside the battery.
        let inner_w = body_w.saturating_sub(4).max(1);
        let inner_h = body_h.saturating_sub(4).max(1);
        let filled_w = ((ctx.eased * inner_w as f32) as usize).min(inner_w);
        if filled_w > 0 {
            draw::fill_rect(grid, bx + 2, by + 2, filled_w, inner_h);
        }

        // Lightning bolt: drawn with dots in the center when filling > 0.
        // Shape: top-right diagonal, then bottom-left diagonal.
        let bolt_x = (bx + body_w / 2) as i32;
        let bolt_mid = (by + body_h / 2) as i32;
        let bolt_h = inner_h as i32;
        let blink = (ctx.time * 2.0).fract() > 0.3 || ctx.eased > 0.5;
        if blink {
            // Top half of bolt: slants right.
            for dy in 0..bolt_h / 2 {
                let y = bolt_mid - dy as i32;
                let x = bolt_x + (dy as f32 * 0.6) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
            // Bottom half: slants left.
            for dy in 0..=bolt_h / 2 {
                let y = bolt_mid + dy as i32;
                let x = bolt_x - (dy as f32 * 0.6) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
        }

        // Rising charge bubbles — dots that travel upward inside the fill.
        let n_bubbles = 4usize;
        for b in 0..n_bubbles {
            let phase = b as f32 / n_bubbles as f32;
            let t = (ctx.time * 0.5 + phase).fract();
            let bub_x = bx + 2 + (hashf(b as u32 * 7 + 3) * inner_w as f32) as usize;
            let bub_x = bub_x.min(bx + body_w.saturating_sub(3));
            let bub_y = by
                + 2
                + inner_h
                    .saturating_sub(1)
                    .saturating_sub((t * inner_h as f32) as usize);
            // Only show bubble in the filled zone.
            let fill_right = bx + 2 + filled_w;
            if bub_x < fill_right {
                draw::dot(grid, bub_x, bub_y);
                if bub_x + 1 < fill_right {
                    draw::dot(grid, bub_x + 1, bub_y);
                }
            }
        }

        // Tint filled region green→yellow by charge level.
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        for cx in 0..filled_cells {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 2. WiFi connecting ───────────────────────────────────────────────────────

/// Concentric arcs pulse outward, then lock solid once connected.
struct WifiConnect;
impl ProgressStyle for WifiConnect {
    fn name(&self) -> &str {
        "wifi-connect"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "WiFi arcs pulse outward while connecting, then lock solid at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Origin: bottom-center, like a standard WiFi symbol.
        let cx_dot = w / 2;
        let cy_dot = h.saturating_sub(1);

        // Number of arcs lit by progress (0–4).
        let n_arcs = 4usize;
        let arcs_lit = (ctx.eased * n_arcs as f32).floor() as usize;
        // Connected: all 4 arcs lit.
        let connected = arcs_lit >= n_arcs;

        // Draw the base dot (the WiFi "device" dot).
        draw::dot(grid, cx_dot, cy_dot);
        if cx_dot + 1 < w {
            draw::dot(grid, cx_dot + 1, cy_dot);
        }

        for arc in 0..n_arcs {
            let radius = (arc + 1) as f32 * (h as f32 / (n_arcs + 1) as f32);
            let lit = arc < arcs_lit;

            // Pulsing alpha for unlocked arcs: sine pulse that propagates outward.
            let pulse_phase = (ctx.time * 1.5 - arc as f32 * 0.4).fract();
            let pulse = (pulse_phase * PI * 2.0).sin() * 0.5 + 0.5;
            let draw_it = lit || (!connected && pulse > 0.5);

            if draw_it {
                // Draw a semicircular arc (top half only — like WiFi symbol).
                // Sweep 180° (from left to right, convex upward).
                let steps = ((PI * radius) as usize).max(8);
                for step in 0..=steps {
                    let angle = PI * (step as f32 / steps as f32);
                    // angle 0 = right, PI = left; we want arcs opening downward.
                    let dx = (angle.cos() * radius) as i32;
                    let dy = -(angle.sin() * radius * 0.6) as i32; // flatten vertically
                    draw::dot_i(grid, cx_dot as i32 + dx, cy_dot as i32 + dy);
                    if !lit {
                        // Dashed: only draw alternating steps for the pulsing arcs.
                        // Already drawn above; skip extra for pulsing effect.
                    }
                }

                // Color: lit arcs get palette gradient.
                if lit {
                    let t = arc as f32 / n_arcs.max(1) as f32;
                    let color = ctx.palette.sample(t);
                    let cell_cx = cx_dot / 2;
                    let r_cells = (radius as usize / 2).max(1);
                    let cx0 = cell_cx.saturating_sub(r_cells);
                    let cx1 = (cell_cx + r_cells).min(cells_w.saturating_sub(1));
                    let cy_cell = cy_dot / 4;
                    let r_cell_h = (radius as usize / 4).max(1);
                    let cy0 = cy_cell.saturating_sub(r_cell_h);
                    for cy in cy0..cells_h {
                        draw::tint_row(grid, cy, cx0, cx1, color);
                    }
                }
            }
        }

        // "Locked" indicator: horizontal bar at bottom when connected.
        if connected {
            let blink_off = (ctx.time * 3.0).fract() < 0.2;
            if !blink_off {
                draw::hline(
                    grid,
                    cx_dot.saturating_sub(2),
                    (cx_dot + 2).min(w - 1),
                    cy_dot,
                );
                let full_color = ctx.palette.sample(1.0);
                draw::tint_row(
                    grid,
                    cells_h.saturating_sub(1),
                    (cells_w / 2).saturating_sub(1),
                    (cells_w / 2 + 1).min(cells_w - 1),
                    full_color,
                );
            }
        }

        Ok(())
    }
}

// ─── 3. Bluetooth pairing ─────────────────────────────────────────────────────

/// The ᛒ-like Bluetooth glyph with handshake blips radiating outward.
struct BluetoothPair;
impl ProgressStyle for BluetoothPair {
    fn name(&self) -> &str {
        "bluetooth-pair"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Bluetooth rune glyph at center; handshake blips pulse outward while pairing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let cx = w / 2;
        let cy = h / 2;

        // ── Bluetooth glyph in dot space (approximated) ──
        // Vertical spine.
        let spine_h = (h * 3 / 4).max(2);
        let y0 = cy.saturating_sub(spine_h / 2);
        let y1 = (cy + spine_h / 2).min(h.saturating_sub(1));
        draw::vline(grid, cx, y0, y1);

        // Upper-right arm and return (forms the top lobe of ᛒ).
        let arm = (h / 5).max(1);
        // Upper-right diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (y0 + arm) as i32 - i as i32);
        }
        // Upper-right return diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (y0 + arm) as i32 + i as i32);
        }
        // Lower-right diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, cy as i32 + i as i32);
        }
        // Lower-right return diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (cy + arm) as i32 - i as i32);
        }

        // ── Handshake blips ──
        // Blips travel horizontally outward from the glyph.
        let n_blips = 5usize;
        let paired = ctx.eased >= 1.0;
        for b in 0..n_blips {
            let phase = b as f32 / n_blips as f32;
            let t = if paired {
                // Frozen in place when paired.
                1.0f32
            } else {
                (ctx.time * 0.8 + phase).fract()
            };
            let reach = (t * (w / 2) as f32) as usize;

            // Left blip.
            let lx = cx.saturating_sub(reach);
            draw::dot(grid, lx, cy);

            // Right blip.
            let rx = (cx + reach).min(w.saturating_sub(1));
            draw::dot(grid, rx, cy);
        }

        // Tint: progress drives gradient, center stays brightest.
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        let mid_cell = cells_w / 2;
        let reach_cells = (filled_cells / 2).max(1);
        let cx0 = mid_cell.saturating_sub(reach_cells);
        let cx1 = (mid_cell + reach_cells).min(cells_w.saturating_sub(1));
        for cx_c in cx0..=cx1 {
            let t = (cx_c.saturating_sub(cx0)) as f32 / (cx1.saturating_sub(cx0) + 1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 4. Disk defragmenter ────────────────────────────────────────────────────

/// A grid of shade-glyph blocks reorganize from chaotic to solid.
struct DiskDefrag;
impl ProgressStyle for DiskDefrag {
    fn name(&self) -> &str {
        "disk-defrag"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Disk defrag: chaotic shade blocks consolidate into solid filled region"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }

        let total_cells = cells_w * cells_h;
        // How many cells are "defragged" (consolidated) vs still fragmented.
        let defragged = (ctx.eased * total_cells as f32) as usize;

        // Cells are laid out in a zigzag (boustrophedon) scan order.
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let linear = cy * cells_w + cx;
                let color = ctx
                    .palette
                    .sample(linear as f32 / total_cells.max(1) as f32);

                if linear < defragged {
                    // Defragged: solid block.
                    draw::glyph(grid, cx, cy, '█');
                    draw::tint_row(grid, cy, cx, cx, color);
                } else {
                    // Fragmented: random shade glyph determined by hash + time.
                    // The fragmented blocks "shuffle" — they animate in place.
                    let epoch = (ctx.time * 4.0) as u32;
                    let h_val = hash(linear as u32 * 31 + epoch * 7 + 3);
                    let shade_level = (h_val % 4) as usize; // 0..3 → ' ░▒▓'
                    if shade_level > 0 {
                        draw::shade(grid, cx, cy, shade_level);
                        // Dim tint for fragmented cells.
                        let frag_color = ctx.palette.sample(0.15);
                        draw::tint_row(grid, cy, cx, cx, frag_color);
                    }
                }
            }
        }
        Ok(())
    }
}

// ─── 5. CRT TV power-on ──────────────────────────────────────────────────────

/// A bright horizontal line at center expands vertically to fill the screen.
struct CrtPowerOn;
impl ProgressStyle for CrtPowerOn {
    fn name(&self) -> &str {
        "crt-power-on"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "CRT power-on: bright center line expands vertically to fill the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = h / 2;
        // The beam starts as a single scanline and grows to full height.
        let beam_h = ((ctx.eased * h as f32) as usize).max(1).min(h);
        let y0 = mid.saturating_sub(beam_h / 2);
        let y1 = (y0 + beam_h).min(h);

        // Fill the beam area — horizontal scanlines.
        for y in y0..y1 {
            draw::hline(grid, 0, w.saturating_sub(1), y);
        }

        // Bright edge lines (phosphor glow at the expanding boundary).
        // The leading edges are extra dense (draw them twice — one line extra).
        if y0 > 0 {
            draw::hline(grid, 0, w.saturating_sub(1), y0.saturating_sub(1));
        }
        if y1 < h {
            draw::hline(grid, 0, w.saturating_sub(1), y1);
        }

        // Horizontal scanline "noise" inside the beam: a few dotted lines fade in.
        if beam_h > 4 {
            let noise_lines = beam_h / 4;
            for nl in 0..noise_lines {
                let y_nl = y0 + 2 + nl * 4;
                if y_nl >= y1 {
                    break;
                }
                // Sparse noise dots derived from hash.
                for x in 0..w {
                    let on = (hash((x as u32).wrapping_add(y_nl as u32 * 17)) % 5) == 0;
                    if on {
                        draw::dot(grid, x, y_nl);
                    }
                }
            }
        }

        // Tint: white-ish center → palette edge glow.
        let cy_center = cells_h / 2;
        let beam_cells = (beam_h / 4).max(1);
        let cy0 = cy_center.saturating_sub(beam_cells / 2);
        let cy1 = (cy0 + beam_cells).min(cells_h.saturating_sub(1));
        for cy in cy0..=cy1 {
            let t = (cy.saturating_sub(cy0)) as f32 / (cy1.saturating_sub(cy0) + 1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─── 6. Dial-up modem handshake ──────────────────────────────────────────────

/// Noisy oscillation across the bar gradually resolves to a steady carrier tone.
struct DialUpModem;
impl ProgressStyle for DialUpModem {
    fn name(&self) -> &str {
        "dialup-modem"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Dial-up modem: chaotic noise waveform settles to a clean carrier tone as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = (h / 2) as i32;
        let amp = (h / 2).saturating_sub(1) as f32;

        for x in 0..w {
            // "Clean" carrier: a smooth sine at this column.
            let carrier_phase = x as f32 / w as f32 * 4.0 * PI + ctx.time * 3.0;
            let clean_y = mid + (carrier_phase.sin() * amp * 0.8) as i32;

            // "Noisy" signal: carrier + random hash jitter.
            let epoch = (ctx.time * 8.0) as u32;
            let noise = hashf(x as u32 * 13 + epoch * 37) * 2.0 - 1.0;
            let noisy_y = mid + ((carrier_phase.sin() * 0.8 + noise * 1.5) * amp) as i32;

            // Blend: lerp from noisy (progress=0) → clean (progress=1).
            let y = (noisy_y as f32 * (1.0 - ctx.eased) + clean_y as f32 * ctx.eased) as i32;
            draw::dot_i(grid, x as i32, y);

            // Draw a second dot one above/below to fatten the trace.
            draw::dot_i(grid, x as i32, y + 1);
        }

        // Tint: chaotic region dim, settled region bright (left→right by progress).
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.max(1) as f32;
            let is_settled = cx < filled_cells;
            let color = if is_settled {
                ctx.palette.sample(t)
            } else {
                ctx.palette.sample(0.05)
            };
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 7. CD / Vinyl spinning up ────────────────────────────────────────────────

/// A disc outline with an index mark rotates; RPM increases with progress.
struct VinylSpinUp;
impl ProgressStyle for VinylSpinUp {
    fn name(&self) -> &str {
        "vinyl-spin"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Vinyl/CD disc outline spins up; rotation speed scales with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let cx = w / 2;
        let cy = h / 2;
        // Radius: constrained to fit in the shorter axis, slightly inset.
        let r_max = (w.min(h * 2) / 2).saturating_sub(1).max(1);

        // Angular speed: 0 rpm at progress=0 → 360 deg/s at progress=1.
        let omega = ctx.eased * 2.0 * PI * 2.0; // 2 full turns/sec at full progress.
        let angle = ctx.time * omega;

        // Outer disc ring.
        let steps = (2.0 * PI * r_max as f32) as usize + 4;
        for i in 0..steps {
            let theta = i as f32 / steps as f32 * 2.0 * PI;
            // Squish Y by 0.5 (dots are taller than wide).
            let dx = (theta.cos() * r_max as f32) as i32;
            let dy = (theta.sin() * r_max as f32 * 0.5) as i32;
            draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
        }

        // Inner hole (label ring) — about 30% of r_max.
        let r_inner = (r_max * 3 / 10).max(1);
        let inner_steps = (2.0 * PI * r_inner as f32) as usize + 4;
        for i in 0..inner_steps {
            let theta = i as f32 / inner_steps as f32 * 2.0 * PI;
            let dx = (theta.cos() * r_inner as f32) as i32;
            let dy = (theta.sin() * r_inner as f32 * 0.5) as i32;
            draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
        }

        // Center spindle dot.
        draw::dot(grid, cx, cy);

        // Index mark: a short radial line from inner to outer, rotating.
        let mark_steps = (r_max - r_inner).max(1);
        for step in 0..=mark_steps {
            let r = r_inner + step;
            let dx = (angle.cos() * r as f32) as i32;
            let dy = (angle.sin() * r as f32 * 0.5) as i32;
            draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
        }

        // Groove lines (concentric rings at intermediate radii).
        let n_grooves = 3usize;
        for g in 1..=n_grooves {
            let r_g = r_inner + (r_max - r_inner) * g / (n_grooves + 1);
            if r_g == 0 {
                continue;
            }
            let g_steps = (2.0 * PI * r_g as f32) as usize + 4;
            for i in (0..g_steps).step_by(3) {
                // Sparse for groove look.
                let theta = i as f32 / g_steps as f32 * 2.0 * PI;
                let dx = (theta.cos() * r_g as f32) as i32;
                let dy = (theta.sin() * r_g as f32 * 0.5) as i32;
                draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
            }
        }

        // Tint the disc area with palette gradient (left→right).
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(ctx.eased * t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 8. Drone quad-propellers ────────────────────────────────────────────────

/// Four rotating arc-pairs in quadrants; RPM rises to full with progress.
struct DroneProps;
impl ProgressStyle for DroneProps {
    fn name(&self) -> &str {
        "drone-props"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Quad-drone: four propeller arcs spin up as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Propeller hubs at the four quadrant centers.
        let qw = w / 2;
        let qh = h / 2;
        // Arc radius: fills the quadrant.
        let r = (qw.min(qh * 2) / 2).max(1);

        // Spin rate: 0 → 3 rev/sec.
        let omega = ctx.eased * 2.0 * PI * 3.0;
        let angle = ctx.time * omega;

        // Four hub positions: (cx, cy) in dot space.
        let hubs = [
            (qw / 2, qh / 2),
            (w - qw / 2, qh / 2),
            (qw / 2, h - qh / 2),
            (w - qw / 2, h - qh / 2),
        ];

        // Draw a cross-frame connector (drone body).
        let body_cx = w / 2;
        let body_cy = h / 2;
        draw::dot(grid, body_cx, body_cy);
        // Arms from center to each hub (sparse dots).
        for (hx, hy) in &hubs {
            let dx = *hx as i32 - body_cx as i32;
            let dy = *hy as i32 - body_cy as i32;
            let steps = dx.abs().max(dy.abs()).max(1);
            for step in 0..steps {
                let t = step as f32 / steps as f32;
                let px = body_cx as i32 + (dx as f32 * t) as i32;
                let py = body_cy as i32 + (dy as f32 * t) as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // For each hub, draw two blade arcs (180° apart).
        for (prop_idx, (hx, hy)) in hubs.iter().enumerate() {
            let hub_angle = angle + prop_idx as f32 * PI * 0.5; // Each prop offset.
                                                                // Two blades per prop, 180° apart.
            for blade in 0..2usize {
                let blade_angle = hub_angle + blade as f32 * PI;
                // Arc: 60° sweep.
                let arc_sweep = PI / 3.0;
                let arc_steps = (arc_sweep * r as f32) as usize + 4;
                for step in 0..arc_steps {
                    let theta = blade_angle - arc_sweep / 2.0
                        + step as f32 / arc_steps.max(1) as f32 * arc_sweep;
                    let dx = (theta.cos() * r as f32) as i32;
                    let dy = (theta.sin() * r as f32 * 0.5) as i32;
                    draw::dot_i(grid, *hx as i32 + dx, *hy as i32 + dy);
                }
            }
            // Hub dot.
            draw::dot(grid, *hx, *hy);
        }

        // Tint by quadrant.
        for cx_c in 0..cells_w {
            for cy_c in 0..cells_h {
                let t = cx_c as f32 / cells_w.max(1) as f32;
                let color = ctx.palette.sample(t * ctx.eased);
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 9. Smartwatch activity ring ─────────────────────────────────────────────

/// A circular ring closes clockwise as progress increases.
struct ActivityRing;
impl ProgressStyle for ActivityRing {
    fn name(&self) -> &str {
        "activity-ring"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Smartwatch activity ring closes clockwise as progress fills it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let cx = w / 2;
        let cy = h / 2;
        let r_outer = (w.min(h * 2) / 2).saturating_sub(1).max(2);
        let r_inner = r_outer.saturating_sub((r_outer / 4).max(1));

        // Total arc swept: progress × 2π, starting at top (−π/2).
        let start_angle = -PI / 2.0;
        let sweep = ctx.eased * 2.0 * PI;

        // Draw thick arc from r_inner to r_outer.
        let steps = ((2.0 * PI * r_outer as f32) as usize + 4).max(16);
        for i in 0..=steps {
            let frac = i as f32 / steps as f32;
            let theta = frac * 2.0 * PI;
            // Only draw dots within the swept angle.
            let rel = (theta - (start_angle + PI * 2.0)).rem_euclid(2.0 * PI);
            if rel > sweep {
                continue;
            }

            for r in r_inner..=r_outer {
                let dx = (theta.cos() * r as f32) as i32;
                let dy = (theta.sin() * r as f32 * 0.5) as i32;
                draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
            }
        }

        // "Track" ring (unfilled portion) — sparse dots.
        for i in 0..=steps {
            let frac = i as f32 / steps as f32;
            let theta = frac * 2.0 * PI;
            let rel = (theta - (start_angle + PI * 2.0)).rem_euclid(2.0 * PI);
            if rel <= sweep {
                continue;
            }
            if i % 3 != 0 {
                continue;
            } // Sparse.
            let r = (r_inner + r_outer) / 2;
            let dx = (theta.cos() * r as f32) as i32;
            let dy = (theta.sin() * r as f32 * 0.5) as i32;
            draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
        }

        // Leading-edge glow: extra dots at the arc tip.
        let tip_theta = start_angle + sweep;
        for r in r_inner..=r_outer {
            let dx = (tip_theta.cos() * r as f32) as i32;
            let dy = (tip_theta.sin() * r as f32 * 0.5) as i32;
            draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
            draw::dot_i(grid, cx as i32 + dx + 1, cy as i32 + dy);
        }

        // Tint filled cells.
        let mid_c = cells_w / 2;
        let r_cells = (r_outer / 2).max(1);
        let cx0 = mid_c.saturating_sub(r_cells);
        let cx1 = (mid_c + r_cells).min(cells_w.saturating_sub(1));
        for cx_c in cx0..=cx1 {
            let t = (cx_c.saturating_sub(cx0)) as f32 / (cx1.saturating_sub(cx0) + 1).max(1) as f32;
            let color = ctx.palette.sample(ctx.eased * t + ctx.eased * 0.2);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 10. USB file transfer ────────────────────────────────────────────────────

/// File glyphs fly from a left device to a right device; count matches eased%.
struct UsbTransfer;
impl ProgressStyle for UsbTransfer {
    fn name(&self) -> &str {
        "usb-transfer"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "USB transfer: file packets fly left→right; count and speed scale with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = h / 2;

        // ── Device icons: left (source) and right (destination) ──
        // Left block: a 2-dot-wide, full-height rect.
        let dev_w = (w / 16).max(2);
        draw::fill_rect(grid, 0, 0, dev_w, h);
        // Right block.
        draw::fill_rect(grid, w.saturating_sub(dev_w), 0, dev_w, h);

        // ── USB cable track line ──
        draw::hline(grid, dev_w, w.saturating_sub(dev_w + 1), mid);

        // ── Flying file packets ──
        // Number of active packets: 1 at 0%, up to 6 at 100%.
        let max_packets = 6usize;
        let active = ((ctx.eased * max_packets as f32).ceil() as usize).min(max_packets);
        let track_w = w.saturating_sub(dev_w * 2 + 2);

        for p in 0..active {
            let phase = p as f32 / max_packets as f32;
            // Speed: faster at higher progress.
            let speed = 0.4 + ctx.eased * 0.6;
            let t = (ctx.time * speed + phase).fract();
            let x0 = dev_w + 1 + (t * track_w as f32) as usize;
            let x0 = x0.min(w.saturating_sub(dev_w + 3));

            // Packet shape: small rectangle, 3-dot wide, 2-dot tall.
            let pkt_w = (w / 20).max(2).min(4);
            let pkt_h = 2usize.min(h);
            let py0 = mid.saturating_sub(pkt_h / 2);
            draw::fill_rect(grid, x0, py0, pkt_w, pkt_h);

            // Tiny "file" notch on the packet (top-right corner void).
            if x0 + pkt_w < w && py0 < h {
                // Just leave that corner unset (draw::fill_rect already did it,
                // but this gives the packet a recognisable dog-ear by drawing the
                // outline of the corner explicitly with dots).
            }

            // Tint.
            let t_color = p as f32 / max_packets.max(1) as f32;
            let color = ctx.palette.sample(t_color);
            let cx0 = (x0 / 2).min(cells_w.saturating_sub(1));
            let cx1 = ((x0 + pkt_w) / 2).min(cells_w.saturating_sub(1));
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx0, cx1, color);
            }
        }

        // Tint device icons.
        let dev_cells = (dev_w / 2).max(1);
        let src_color = ctx.palette.sample(0.0);
        let dst_color = ctx.palette.sample(1.0);
        for cy in 0..cells_h {
            draw::tint_row(grid, cy, 0, dev_cells.saturating_sub(1), src_color);
            draw::tint_row(
                grid,
                cy,
                cells_w.saturating_sub(dev_cells),
                cells_w.saturating_sub(1),
                dst_color,
            );
        }
        Ok(())
    }
}

// ─── 11. Gear train ──────────────────────────────────────────────────────────

/// Interlocking rotating gears; large left, smaller right, alternate directions.
struct GearTrain;
impl ProgressStyle for GearTrain {
    fn name(&self) -> &str {
        "gear-train"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Interlocking gear train: large and small gears rotate in opposite directions"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Draw a gear: circle with N teeth (short radial protrusions).
        let draw_gear = |grid: &mut BrailleGrid,
                         cx: i32,
                         cy: i32,
                         r: usize,
                         n_teeth: usize,
                         angle_offset: f32| {
            if r == 0 {
                return;
            }
            let tooth_len = (r / 4).max(1) as f32;
            let r_f = r as f32;
            let steps = (2.0 * PI * r_f) as usize + 8;
            for i in 0..=steps {
                let theta = i as f32 / steps as f32 * 2.0 * PI + angle_offset;
                // Is this angle at a tooth?
                let tooth_phase = (theta * n_teeth as f32 / (2.0 * PI)).fract();
                let tooth_bump = if tooth_phase < 0.25 || tooth_phase > 0.75 {
                    tooth_len
                } else {
                    0.0
                };
                let r_here = r_f + tooth_bump;
                let dx = (theta.cos() * r_here) as i32;
                let dy = (theta.sin() * r_here * 0.5) as i32;
                draw::dot_i(grid, cx + dx, cy + dy);
            }
            // Hub.
            draw::dot_i(grid, cx, cy);
        };

        // Gear dimensions: big gear on the left, smaller on the right.
        let big_r = (h * 3 / 8).max(2);
        let small_r = (big_r / 2).max(1);
        let big_cx = (big_r + 2) as i32;
        let big_cy = (h / 2) as i32;
        // Small gear meshes with the big gear; positioned to its right.
        let mesh_gap = 1i32; // gap between teeth
        let small_cx = big_cx + big_r as i32 + small_r as i32 + mesh_gap;
        let small_cy = big_cy;

        // Rotation: speed 1.0 rad/s at full progress.
        let omega_big = ctx.eased * 1.5;
        let big_angle = ctx.time * omega_big;
        // Small gear rotates inversely proportional to size ratio.
        let gear_ratio = big_r as f32 / small_r.max(1) as f32;
        let small_angle = -ctx.time * omega_big * gear_ratio;

        let n_teeth_big = (big_r / 2).max(4).min(16);
        let n_teeth_small = (small_r / 2).max(3).min(8);

        draw_gear(grid, big_cx, big_cy, big_r, n_teeth_big, big_angle);
        // Only draw small gear if it fits within the grid.
        if small_cx + small_r as i32 + 2 < w as i32 {
            draw_gear(
                grid,
                small_cx,
                small_cy,
                small_r,
                n_teeth_small,
                small_angle,
            );
        }

        // Optional third (tiny) gear further right.
        let tiny_r = (small_r / 2).max(1);
        let tiny_cx = small_cx + small_r as i32 + tiny_r as i32 + mesh_gap;
        let tiny_angle = -small_angle * (small_r as f32 / tiny_r.max(1) as f32);
        if tiny_cx + tiny_r as i32 + 2 < w as i32 {
            draw_gear(
                grid,
                tiny_cx,
                small_cy,
                tiny_r,
                (tiny_r / 2).max(3).min(6),
                tiny_angle,
            );
        }

        // Tint: gradient left to right.
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 12. E-ink page refresh ───────────────────────────────────────────────────

/// Screen flashes to black, then content fills in cell-by-cell, row by row.
struct EinkRefresh;
impl ProgressStyle for EinkRefresh {
    fn name(&self) -> &str {
        "eink-refresh"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "E-ink refresh: flashes full black then redraws content row-by-row via shade blocks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);

        // Phase 0 (progress 0–0.15): flash to full black.
        // Phase 1 (0.15–1.0): content draws in row by row, left to right.
        let flash_phase = 0.15f32;

        if ctx.eased < flash_phase {
            // Full black (invert flash) — fill everything.
            draw::fill_rect(grid, 0, 0, w, h);
            let color = ctx.palette.sample(0.0);
            draw::tint_row(grid, 0, 0, cells_w.saturating_sub(1), color);
        } else {
            // Normalise to the draw-in phase.
            let draw_frac = (ctx.eased - flash_phase) / (1.0 - flash_phase);
            let total_cells = cells_w * cells_h;
            let cells_drawn = ((draw_frac * total_cells as f32) as usize).min(total_cells);

            // Rows: complete rows first, then a partial last row.
            let full_rows = cells_drawn / cells_w.max(1);
            let partial_cols = cells_drawn % cells_w.max(1);

            for cy in 0..full_rows.min(cells_h) {
                for cx in 0..cells_w {
                    // Alternating shade density based on a hash pattern —
                    // simulates "newspaper" e-ink content.
                    let linear = cy * cells_w + cx;
                    let h_val = hash(linear as u32 * 13 + 7);
                    let shade_level = match h_val % 5 {
                        0 => 1,     // ░
                        1 => 2,     // ▒
                        2 => 3,     // ▓
                        3 | 4 => 4, // █
                        _ => 4,
                    };
                    draw::shade(grid, cx, cy, shade_level);
                    let t = cx as f32 / cells_w.max(1) as f32;
                    let color = ctx.palette.sample(t * 0.6 + 0.2);
                    draw::tint_row(grid, cy, cx, cx, color);
                }
            }
            // Partial row.
            let partial_row = full_rows.min(cells_h.saturating_sub(1));
            if full_rows < cells_h {
                for cx in 0..partial_cols.min(cells_w) {
                    let linear = partial_row * cells_w + cx;
                    let h_val = hash(linear as u32 * 13 + 7);
                    let shade_level = (h_val % 4 + 1) as usize;
                    draw::shade(grid, cx, partial_row, shade_level);
                    let t = cx as f32 / cells_w.max(1) as f32;
                    let color = ctx.palette.sample(t * 0.5 + 0.3);
                    draw::tint_row(grid, partial_row, cx, cx, color);
                }
                // Cursor line: a blinking single-dot underline at the draw-in edge.
                if partial_cols < cells_w {
                    let blink = (ctx.time * 4.0).fract() > 0.5;
                    if blink {
                        let (_, dot_h) = draw::dot_dims(grid);
                        let row_y = (partial_row * 4 + 3).min(dot_h.saturating_sub(1));
                        draw::dot(grid, partial_cols * 2, row_y);
                    }
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
    let styles = progress::styles::gadgets::styles();
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
