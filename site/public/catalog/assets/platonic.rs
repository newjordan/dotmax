//! `platonic` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O platonic.rs && ./platonic [style-name]
//! ```

const DEFAULT_STYLE: &str = "tetrahedron";

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
    pub mod platonic {
//! Platonic solids and sacred-3D-form progress bars.
//!
//! Each bar is a rotating 3-D wireframe of a genuine geometric solid, drawn via
//! orthographic projection onto the dot lattice.  `ctx.time` drives the spin;
//! `ctx.eased` (0 → 1) reveals edges one-by-one so the solid assembles itself
//! as progress advances.
//!
//! A single `project` helper (identical signature to the topology module's) and
//! a bounded Bresenham edge-drawer are shared by every style.  Scale is
//! auto-fitted so the circumscribed sphere just fills `min(w/2, h)` dot-radii.
//!
//! # Styles
//!
//! | Name | Geometry |
//! |---|---|
//! | `tetrahedron`          | 4 vertices, 6 edges — the fire solid |
//! | `cube`                 | 8 vertices, 12 edges — the earth solid |
//! | `octahedron`           | 6 vertices, 12 edges — the air solid |
//! | `dodecahedron`         | 20 vertices, 30 edges — golden-ratio faces |
//! | `icosahedron`          | 12 vertices, 30 edges — dual of dodecahedron |
//! | `merkaba`              | Two counter-rotating tetrahedra (star-tetrahedron) |
//! | `star-octangulum`      | Stella octangula — two interlocked tetrahedra (octahedron dual) |
//! | `cuboctahedron`        | Vector equilibrium / Metatron's core, 12 vertices 24 edges |
//! | `nested-solids`        | Cube inside its dual octahedron, both spinning |
//! | `stellated-dodecahedron` | Great stellated dodecahedron — spiky star |
//! | `unfolding-net`        | Cube net folds up into 3-D as eased → 1 |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};

// ── Shared 3-D helpers ────────────────────────────────────────────────────────

/// Rotate `(x, y, z)` about X by `ax` then Y by `ay` (extrinsic Euler XY),
/// then orthographically project onto the dot lattice centred at `(cx, cy)`.
///
/// Returns `(screen_x, screen_y)` as `i32` for `draw::dot_i`.
#[inline]
fn project(x: f32, y: f32, z: f32, ax: f32, ay: f32, cx: i32, cy: i32, scale: f32) -> (i32, i32) {
    let (sax, cax) = ax.sin_cos();
    let y1 = y * cax - z * sax;
    let z1 = y * sax + z * cax;
    let (say, cay) = ay.sin_cos();
    let x2 = x * cay + z1 * say;
    let y2 = y1;
    let sx = cx + (x2 * scale).round() as i32;
    let sy = cy - (y2 * scale).round() as i32;
    (sx, sy)
}

/// Draw a Bresenham line between two projected points.  Step count is capped
/// at 400 so wide grids stay snappy; `draw::dot_i` silently ignores OOB.
#[inline]
fn draw_edge(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1i32 };
    let sy = if y0 < y1 { 1i32 } else { -1i32 };
    let mut x = x0;
    let mut y = y0;
    let mut err = dx - dy;
    let max_steps = (dx + dy + 2).min(400);
    for _ in 0..max_steps {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Compute grid centre in dot coords and a uniform scale so a unit-radius
/// solid fits `min(w/2, h)` dot-radii.  The `shrink` factor lets a style
/// reduce the scale further (e.g. 0.85 for dodecahedron which has circumradius > 1).
fn grid_centre_scale(grid: &BrailleGrid, shrink: f32) -> (i32, i32, f32) {
    let (dw, dh) = draw::dot_dims(grid);
    let cx = (dw / 2) as i32;
    let cy = (dh / 2) as i32;
    // Fit to the smaller of half-width and full-height (dots), with margin.
    let r = (dw / 2).min(dh) as f32;
    let scale = (r * 0.82 * shrink).max(1.0);
    (cx, cy, scale)
}

/// Project a slice of 3-D vertices with given rotation angles, returning
/// screen-space `(i32, i32)` for each.
fn project_verts(
    verts: &[[f32; 3]],
    ax: f32,
    ay: f32,
    cx: i32,
    cy: i32,
    scale: f32,
) -> Vec<(i32, i32)> {
    verts
        .iter()
        .map(|&[x, y, z]| project(x, y, z, ax, ay, cx, cy, scale))
        .collect()
}

/// Draw `n_show` edges from `edges`, using pre-projected `pts`.
fn draw_edges_partial(
    grid: &mut BrailleGrid,
    pts: &[(i32, i32)],
    edges: &[(usize, usize)],
    n_show: usize,
) {
    for &(a, b) in edges.iter().take(n_show) {
        if a < pts.len() && b < pts.len() {
            let (x0, y0) = pts[a];
            let (x1, y1) = pts[b];
            draw_edge(grid, x0, y0, x1, y1);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Tetrahedron — fire solid (4 vertices, 6 edges)
// ─────────────────────────────────────────────────────────────────────────────

/// Tetrahedron inscribed in the unit sphere.
const TETRA_VERTS: [[f32; 3]; 4] = [
    [0.0, 1.0, 0.0],                   // top
    [0.942809, -0.333333, 0.0],        // front-right
    [-0.471405, -0.333333, 0.816497],  // back-left
    [-0.471405, -0.333333, -0.816497], // back-right
];
const TETRA_EDGES: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

struct Tetrahedron;
impl ProgressStyle for Tetrahedron {
    fn name(&self) -> &str {
        "tetrahedron"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Tetrahedron — the fire solid: 4 vertices and 6 edges assembling as progress grows, spinning on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 1.0);
        let ax = ctx.time * 0.41;
        let ay = ctx.time * 0.63;
        let pts = project_verts(&TETRA_VERTS, ax, ay, cx, cy, scale);
        let n_show = (ctx.eased * TETRA_EDGES.len() as f32).ceil() as usize;
        draw_edges_partial(grid, &pts, &TETRA_EDGES, n_show);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Cube / hexahedron — earth solid (8 vertices, 12 edges)
// ─────────────────────────────────────────────────────────────────────────────

/// Cube with vertices at (±1/√3, ±1/√3, ±1/√3) — unit circumsphere.
const S3: f32 = 0.577_350_3; // 1/√3

const CUBE_VERTS: [[f32; 3]; 8] = [
    [-S3, -S3, -S3],
    [S3, -S3, -S3],
    [S3, S3, -S3],
    [-S3, S3, -S3],
    [-S3, -S3, S3],
    [S3, -S3, S3],
    [S3, S3, S3],
    [-S3, S3, S3],
];
const CUBE_EDGES: [(usize, usize); 12] = [
    // Bottom face.
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    // Top face.
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    // Verticals.
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

struct Cube;
impl ProgressStyle for Cube {
    fn name(&self) -> &str {
        "cube"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Hexahedron — the earth solid: 8-vertex cube with 12 edges revealed face-by-face as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 1.0);
        let ax = ctx.time * 0.37;
        let ay = ctx.time * 0.51;
        let pts = project_verts(&CUBE_VERTS, ax, ay, cx, cy, scale);
        let n_show = (ctx.eased * CUBE_EDGES.len() as f32).ceil() as usize;
        draw_edges_partial(grid, &pts, &CUBE_EDGES, n_show);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Octahedron — air solid (6 vertices, 12 edges)
// ─────────────────────────────────────────────────────────────────────────────

const OCTA_VERTS: [[f32; 3]; 6] = [
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];
const OCTA_EDGES: [(usize, usize); 12] = [
    (0, 2),
    (0, 3),
    (0, 4),
    (0, 5),
    (1, 2),
    (1, 3),
    (1, 4),
    (1, 5),
    (2, 4),
    (2, 5),
    (3, 4),
    (3, 5),
];

struct Octahedron;
impl ProgressStyle for Octahedron {
    fn name(&self) -> &str {
        "octahedron"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Octahedron — the air solid: 6-vertex double-pyramid with 12 equilateral-triangle edges spinning on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 1.0);
        let ax = ctx.time * 0.44;
        let ay = ctx.time * 0.59;
        let pts = project_verts(&OCTA_VERTS, ax, ay, cx, cy, scale);
        let n_show = (ctx.eased * OCTA_EDGES.len() as f32).ceil() as usize;
        draw_edges_partial(grid, &pts, &OCTA_EDGES, n_show);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Dodecahedron — aether solid (20 vertices, 30 edges)
//    Circumradius = √3 ⟹ shrink by 1/√3.
// ─────────────────────────────────────────────────────────────────────────────

/// Golden ratio φ = (1+√5)/2.
const PHI: f32 = 1.618_033_9;
/// 1/φ.
const INV_PHI: f32 = 0.618_033_9;
/// Circumradius of the dodecahedron built from these coordinates is √3.
const DODECA_SCALE: f32 = 1.0 / 1.732_050_8; // 1/√3

const DODECA_VERTS: [[f32; 3]; 20] = [
    // ±1 permutations (8 vertices).
    [1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    // (0, ±1/φ, ±φ) cyclic permutations (12 vertices).
    [0.0, INV_PHI, PHI],
    [0.0, INV_PHI, -PHI],
    [0.0, -INV_PHI, PHI],
    [0.0, -INV_PHI, -PHI],
    [INV_PHI, PHI, 0.0],
    [INV_PHI, -PHI, 0.0],
    [-INV_PHI, PHI, 0.0],
    [-INV_PHI, -PHI, 0.0],
    [PHI, 0.0, INV_PHI],
    [PHI, 0.0, -INV_PHI],
    [-PHI, 0.0, INV_PHI],
    [-PHI, 0.0, -INV_PHI],
];

/// Edges: the 30 edges of the dodecahedron.
/// Each vertex has degree 3.  Edge list derived from the known vertex adjacency.
const DODECA_EDGES: [(usize, usize); 30] = [
    // Vertex 0: (1,1,1) connects to vertices 8,12,16.
    (0, 8),
    (0, 12),
    (0, 16),
    // Vertex 1: (1,1,-1) connects to 9,12,17.
    (1, 9),
    (1, 12),
    (1, 17),
    // Vertex 2: (1,-1,1) connects to 10,13,16.
    (2, 10),
    (2, 13),
    (2, 16),
    // Vertex 3: (1,-1,-1) connects to 11,13,17.
    (3, 11),
    (3, 13),
    (3, 17),
    // Vertex 4: (-1,1,1) connects to 8,14,18.
    (4, 8),
    (4, 14),
    (4, 18),
    // Vertex 5: (-1,1,-1) connects to 9,14,19.
    (5, 9),
    (5, 14),
    (5, 19),
    // Vertex 6: (-1,-1,1) connects to 10,15,18.
    (6, 10),
    (6, 15),
    (6, 18),
    // Vertex 7: (-1,-1,-1) connects to 11,15,19.
    (7, 11),
    (7, 15),
    (7, 19),
    // (0,±INV_PHI,±PHI) ring connections (vertices 8-11).
    (8, 10),
    (9, 11),
    // (±INV_PHI,±PHI,0) ring connections (vertices 12-15).
    (12, 14),
    (13, 15),
    // (±PHI,0,±INV_PHI) ring connections (vertices 16-19).
    (16, 17),
    (18, 19),
];

struct Dodecahedron;
impl ProgressStyle for Dodecahedron {
    fn name(&self) -> &str {
        "dodecahedron"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Dodecahedron — the aether solid: 20 golden-ratio vertices and 30 pentagonal edges spinning and assembling"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, DODECA_SCALE);
        let ax = ctx.time * 0.29;
        let ay = ctx.time * 0.47;
        let pts = project_verts(&DODECA_VERTS, ax, ay, cx, cy, scale);
        let n_show = (ctx.eased * DODECA_EDGES.len() as f32).ceil() as usize;
        draw_edges_partial(grid, &pts, &DODECA_EDGES, n_show);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Icosahedron — water solid (12 vertices, 30 edges)
//    Vertices from (0, ±1, ±φ) cyclic permutations; circumradius = √(1+φ²).
// ─────────────────────────────────────────────────────────────────────────────

/// Circumradius of icosahedron with these coords = √(1+φ²) ≈ 1.902.
const ICOSA_SCALE: f32 = 1.0 / 1.902_113_0;

const ICOSA_VERTS: [[f32; 3]; 12] = [
    // (0, ±1, ±φ).
    [0.0, 1.0, PHI],
    [0.0, 1.0, -PHI],
    [0.0, -1.0, PHI],
    [0.0, -1.0, -PHI],
    // (±1, ±φ, 0).
    [1.0, PHI, 0.0],
    [1.0, -PHI, 0.0],
    [-1.0, PHI, 0.0],
    [-1.0, -PHI, 0.0],
    // (±φ, 0, ±1).
    [PHI, 0.0, 1.0],
    [PHI, 0.0, -1.0],
    [-PHI, 0.0, 1.0],
    [-PHI, 0.0, -1.0],
];

/// 30 edges of the icosahedron (each vertex connects to 5 neighbours at distance 2).
const ICOSA_EDGES: [(usize, usize); 30] = [
    // Top cap (0).
    (0, 2),
    (0, 4),
    (0, 6),
    (0, 8),
    (0, 10),
    // Upper ring.
    (4, 8),
    (8, 2),
    (2, 10),
    (10, 6),
    (6, 4),
    // Bottom cap (3).
    (3, 1),
    (3, 5),
    (3, 7),
    (3, 9),
    (3, 11),
    // Lower ring.
    (1, 9),
    (9, 5),
    (5, 7),
    (7, 11),
    (11, 1),
    // Equatorial connectors.
    (4, 1),
    (1, 6),
    (6, 11),
    (11, 10),
    (10, 7),
    (7, 5),
    (5, 2),
    (2, 8),
    (8, 9),
    (9, 3),
];

struct Icosahedron;
impl ProgressStyle for Icosahedron {
    fn name(&self) -> &str {
        "icosahedron"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Icosahedron — the water solid: 12 golden-ratio vertices forming 30 equilateral-triangle edges, spinning on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, ICOSA_SCALE);
        let ax = ctx.time * 0.33;
        let ay = ctx.time * 0.54;
        let pts = project_verts(&ICOSA_VERTS, ax, ay, cx, cy, scale);
        let n_show = (ctx.eased * ICOSA_EDGES.len() as f32).ceil() as usize;
        draw_edges_partial(grid, &pts, &ICOSA_EDGES, n_show);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Merkaba — star tetrahedron: two counter-rotating tetrahedra
// ─────────────────────────────────────────────────────────────────────────────

/// The merkaba has two interlocked tetrahedra.  One uses the canonical upward
/// tetrahedron; the other is its inversion (downward — dual).  They counter-rotate
/// with time so the Merkaba field animates distinctly even without edge-reveal.

struct Merkaba;
impl ProgressStyle for Merkaba {
    fn name(&self) -> &str {
        "merkaba"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Merkaba (star-tetrahedron): two interlocked tetrahedra counter-rotating — the light-body vehicle of sacred geometry"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 1.0);

        // Rotation for the upward tetrahedron.
        let ax_up = ctx.time * 0.39;
        let ay_up = ctx.time * 0.57;
        // Counter-rotation for the downward tetrahedron.
        let ax_dn = ctx.time * 0.39;
        let ay_dn = -ctx.time * 0.57;

        // Upward tetrahedron (same as TETRA_VERTS).
        let pts_up = project_verts(&TETRA_VERTS, ax_up, ay_up, cx, cy, scale);
        // Downward tetrahedron (invert y).
        let tetra_down: [[f32; 3]; 4] = TETRA_VERTS.map(|[x, y, z]| [x, -y, z]);
        let pts_dn = project_verts(&tetra_down, ax_dn, ay_dn, cx, cy, scale);

        // Reveal first tetrahedron on eased 0→0.5, second on 0.5→1.
        let total_edges = TETRA_EDGES.len() * 2;
        let n_show = (ctx.eased * total_edges as f32).ceil() as usize;
        let n_up = n_show.min(TETRA_EDGES.len());
        let n_dn = n_show.saturating_sub(TETRA_EDGES.len());

        draw_edges_partial(grid, &pts_up, &TETRA_EDGES, n_up);
        draw_edges_partial(grid, &pts_dn, &TETRA_EDGES, n_dn);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Star octangulum (stella octangula) — two interlocked tetrahedra as octahedron dual
//    Structurally different from merkaba: both tetrahedra share the SAME rotation
//    axis, offset by 180°, and the stella octangula has 8 spike tips + central octahedron
//    All 12 edges rendered in sequence of 3 groups: top tet, bottom tet, connectors.
// ─────────────────────────────────────────────────────────────────────────────

struct StarOctangulum;
impl ProgressStyle for StarOctangulum {
    fn name(&self) -> &str {
        "star-octangulum"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Stella octangula: two tetrahedra dual to each other inscribed in an octahedron — eight triangular spikes, one rotation axis"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 1.0);
        // Both tetrahedra share the same rotation (co-rotating, not counter-rotating).
        let ax = ctx.time * 0.35;
        let ay = ctx.time * 0.52;

        // Upward tet — scaled to circumradius 1.
        let pts_up = project_verts(&TETRA_VERTS, ax, ay, cx, cy, scale);
        // Downward tet — invert all three axes for the dual orientation.
        let tetra_dual: [[f32; 3]; 4] = TETRA_VERTS.map(|[x, y, z]| [-x, -y, -z]);
        let pts_dn = project_verts(&tetra_dual, ax, ay, cx, cy, scale);

        // Also draw the inner octahedron formed by the intersection.
        // Octahedron vertices are midpoints of the stella's edges (unit sphere).
        let pts_oct = project_verts(&OCTA_VERTS, ax, ay, cx, cy, scale * 0.577_350_3);

        let total = TETRA_EDGES.len() * 2 + OCTA_EDGES.len();
        let n_show = (ctx.eased * total as f32).ceil() as usize;
        let n_up = n_show.min(TETRA_EDGES.len());
        let n_dn = n_show
            .saturating_sub(TETRA_EDGES.len())
            .min(TETRA_EDGES.len());
        let n_oct = n_show.saturating_sub(TETRA_EDGES.len() * 2);

        draw_edges_partial(grid, &pts_up, &TETRA_EDGES, n_up);
        draw_edges_partial(grid, &pts_dn, &TETRA_EDGES, n_dn);
        draw_edges_partial(grid, &pts_oct, &OCTA_EDGES, n_oct);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Cuboctahedron — vector equilibrium / Metatron's core
//    12 vertices: (±1, ±1, 0) and all cyclic permutations. 24 edges.
// ─────────────────────────────────────────────────────────────────────────────

const CUBOCTA_VERTS: [[f32; 3]; 12] = [
    // (±1, ±1, 0).
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [-1.0, -1.0, 0.0],
    // (±1, 0, ±1).
    [1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, 1.0],
    [-1.0, 0.0, -1.0],
    // (0, ±1, ±1).
    [0.0, 1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, 1.0],
    [0.0, -1.0, -1.0],
];

/// Cuboctahedron circumradius = √2, so shrink.
const CUBOCTA_SCALE: f32 = 1.0 / 1.414_213_6;

/// 24 edges of the cuboctahedron.  Each vertex has degree 4.
/// Two vertices are adjacent iff their distance = √2 (= edge length here).
const CUBOCTA_EDGES: [(usize, usize); 24] = [
    // Vertex 0 ( 1, 1, 0): neighbours 4,5,8,9.
    (0, 4),
    (0, 5),
    (0, 8),
    (0, 9),
    // Vertex 1 ( 1,-1, 0): neighbours 4,5,10,11.
    (1, 4),
    (1, 5),
    (1, 10),
    (1, 11),
    // Vertex 2 (-1, 1, 0): neighbours 6,7,8,9.
    (2, 6),
    (2, 7),
    (2, 8),
    (2, 9),
    // Vertex 3 (-1,-1, 0): neighbours 6,7,10,11.
    (3, 6),
    (3, 7),
    (3, 10),
    (3, 11),
    // Cross-ring edges (between the three "belt" squares).
    (4, 8),
    (4, 10),
    (5, 9),
    (5, 11),
    (6, 8),
    (6, 10),
    (7, 9),
    (7, 11),
];

struct Cuboctahedron;
impl ProgressStyle for Cuboctahedron {
    fn name(&self) -> &str {
        "cuboctahedron"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Cuboctahedron — vector equilibrium / Metatron's core: 12 vertices at edge-midpoints of a cube, 8 triangles and 6 squares"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, CUBOCTA_SCALE);
        let ax = ctx.time * 0.31;
        let ay = ctx.time * 0.49;
        let pts = project_verts(&CUBOCTA_VERTS, ax, ay, cx, cy, scale);
        let n_show = (ctx.eased * CUBOCTA_EDGES.len() as f32).ceil() as usize;
        draw_edges_partial(grid, &pts, &CUBOCTA_EDGES, n_show);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Nested solids — cube inside its dual octahedron, both spinning
//    Structurally unique: two DIFFERENT solids rendered simultaneously at
//    different scales and independent rotation speeds.
// ─────────────────────────────────────────────────────────────────────────────

struct NestedSolids;
impl ProgressStyle for NestedSolids {
    fn name(&self) -> &str {
        "nested-solids"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Nested duals: a cube spinning inside its dual octahedron — both rotating at different rates, revealed in two waves"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 1.0);

        // Outer octahedron — slower rotation, revealed in first half of eased.
        let ax_oct = ctx.time * 0.27;
        let ay_oct = ctx.time * 0.41;
        let pts_oct = project_verts(&OCTA_VERTS, ax_oct, ay_oct, cx, cy, scale);

        // Inner cube — faster rotation, revealed in second half, scaled to inradius.
        // Inradius of octahedron = 1/√3 ≈ 0.577, so cube circumradius ≈ 0.577.
        let cube_inner_scale = scale * 0.577_350_3; // fit cube inside octahedron
        let ax_cube = ctx.time * 0.55;
        let ay_cube = ctx.time * 0.71;
        let pts_cube = project_verts(&CUBE_VERTS, ax_cube, ay_cube, cx, cy, cube_inner_scale);

        let n_show = (ctx.eased * (OCTA_EDGES.len() + CUBE_EDGES.len()) as f32).ceil() as usize;
        let n_oct = n_show.min(OCTA_EDGES.len());
        let n_cube = n_show.saturating_sub(OCTA_EDGES.len());

        draw_edges_partial(grid, &pts_oct, &OCTA_EDGES, n_oct);
        draw_edges_partial(grid, &pts_cube, &CUBE_EDGES, n_cube);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Stellated dodecahedron — great stellated dodecahedron (spiky star)
//     Built by erecting a pentagonal pyramid on each of the 12 dodecahedron
//     faces.  Approximated here with the 20 original dodecahedron vertices
//     PLUS 12 spike apices, one beyond each face centre, 60 spoke edges.
// ─────────────────────────────────────────────────────────────────────────────

/// Face centres of the dodecahedron (approximate — used as spike bases).
/// The dodecahedron has 12 pentagonal faces; their centres lie along the 12
/// icosahedron vertices (the dual relationship).  We scale icosahedron verts
/// to get spike tips.
struct StellatedDodecahedron;
impl ProgressStyle for StellatedDodecahedron {
    fn name(&self) -> &str {
        "stellated-dodecahedron"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Great stellated dodecahedron: dodecahedron with 12 pentagonal star-pyramids erupting from each face — a cosmic sea-urchin"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, DODECA_SCALE * 0.72);
        let ax = ctx.time * 0.25;
        let ay = ctx.time * 0.43;

        // Draw the dodecahedron base skeleton first.
        let pts_dodeca = project_verts(&DODECA_VERTS, ax, ay, cx, cy, scale);

        // The 12 spike tips — icosahedron vertices scaled outward by φ.
        let spike_scale = scale * PHI * ICOSA_SCALE;
        let pts_spikes = project_verts(&ICOSA_VERTS, ax, ay, cx, cy, spike_scale);

        // Each spike tip connects to the 5 nearest dodecahedron vertices.
        // For simplicity: each icosahedron vertex (face centre) connects to the
        // 5 dodecahedron vertices closest to it.  We store these as a fixed table.
        // (Derived from geometry: each face of the dodecahedron is a pentagon with 5 vertices.)
        let spike_fans: [(usize, [usize; 5]); 12] = [
            (0, [0, 8, 10, 4, 2]),   // +z face centre → vertices around top face
            (1, [1, 9, 11, 5, 3]),   // -z face
            (2, [0, 12, 14, 4, 16]), // +y-adjacent face
            (3, [1, 12, 14, 5, 17]), // another
            (4, [0, 8, 12, 16, 2]),
            (5, [6, 10, 15, 7, 18]),
            (6, [3, 11, 13, 15, 7]),
            (7, [3, 13, 17, 11, 19]),
            (8, [4, 14, 5, 19, 18]),
            (9, [2, 16, 17, 3, 13]),
            (10, [6, 18, 19, 7, 15]),
            (11, [1, 9, 11, 19, 5]),
        ];

        let n_base = DODECA_EDGES.len();
        let n_spokes = 12 * 5; // 60 spoke edges
        let total = n_base + n_spokes;
        let n_show = (ctx.eased * total as f32).ceil() as usize;
        let n_d = n_show.min(n_base);
        let n_s = n_show.saturating_sub(n_base);

        draw_edges_partial(grid, &pts_dodeca, &DODECA_EDGES, n_d);

        // Draw spokes (spike tip → dodecahedron vertex).
        let mut spoke_drawn = 0usize;
        'outer: for &(si, verts_around) in &spike_fans {
            for &di in &verts_around {
                if spoke_drawn >= n_s {
                    break 'outer;
                }
                if si < pts_spikes.len() && di < pts_dodeca.len() {
                    let (x0, y0) = pts_spikes[si];
                    let (x1, y1) = pts_dodeca[di];
                    draw_edge(grid, x0, y0, x1, y1);
                }
                spoke_drawn += 1;
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Unfolding net — a cube's net folds up into 3-D as eased → 1
//     At eased=0, a cross-shaped net lies flat (all faces in the XY plane).
//     At eased=1, the 6 faces have folded up into a cube.
//     Each face is drawn as a square outline (4 edges), folded by lerping
//     the face's normal direction from flat (z=0) to its final orientation.
// ─────────────────────────────────────────────────────────────────────────────

struct UnfoldingNet;
impl ProgressStyle for UnfoldingNet {
    fn name(&self) -> &str {
        "unfolding-net"
    }
    fn theme(&self) -> &str {
        "platonic"
    }
    fn describe(&self) -> &str {
        "Cube net: a cross-shaped flat net folds progressively into a 3-D cube as progress advances — geometry becoming solid"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_centre_scale(grid, 0.9);
        let ax = ctx.time * 0.30;
        let ay = ctx.time * 0.48;
        let t = ctx.eased; // 0 = flat net, 1 = cube

        // A standard cube cross-net has 6 faces.  We define each face by its
        // 4 corners in *net* space (flat) and in *cube* space (3-D).
        // We interpolate corners between flat and folded at parameter t.

        // Face corners in net space (centered at origin, side length 1).
        // Faces: bottom, front, right, left, back, top — standard cross layout.
        // Net layout (2D, z=0), face side = 1:
        //   [4]           top
        //   [2][0][1][3]  left, front, right, back
        //   [5]           bottom

        let half = 0.5_f32;

        // Net positions (flat, z=0) — each face's 4 corners (bl, br, tr, tl).
        let net: [[[f32; 3]; 4]; 6] = [
            // 0: front face (centre of cross, at (0,0)).
            [
                [-half, -half, 0.0],
                [half, -half, 0.0],
                [half, half, 0.0],
                [-half, half, 0.0],
            ],
            // 1: right face (at (1,0)).
            [
                [half, -half, 0.0],
                [3.0 * half, -half, 0.0],
                [3.0 * half, half, 0.0],
                [half, half, 0.0],
            ],
            // 2: left face (at (-1,0)).
            [
                [-3.0 * half, -half, 0.0],
                [-half, -half, 0.0],
                [-half, half, 0.0],
                [-3.0 * half, half, 0.0],
            ],
            // 3: back face (at (2,0)).
            [
                [3.0 * half, -half, 0.0],
                [5.0 * half, -half, 0.0],
                [5.0 * half, half, 0.0],
                [3.0 * half, half, 0.0],
            ],
            // 4: top face (at (0,1)).
            [
                [-half, half, 0.0],
                [half, half, 0.0],
                [half, 3.0 * half, 0.0],
                [-half, 3.0 * half, 0.0],
            ],
            // 5: bottom face (at (0,-1)).
            [
                [-half, -3.0 * half, 0.0],
                [half, -3.0 * half, 0.0],
                [half, -half, 0.0],
                [-half, -half, 0.0],
            ],
        ];

        // Cube corner positions for each face (fully folded).
        let cube: [[[f32; 3]; 4]; 6] = [
            // 0: front face z=+half (normal +z).
            [
                [-half, -half, half],
                [half, -half, half],
                [half, half, half],
                [-half, half, half],
            ],
            // 1: right face x=+half.
            [
                [half, -half, half],
                [half, -half, -half],
                [half, half, -half],
                [half, half, half],
            ],
            // 2: left face x=-half.
            [
                [-half, -half, -half],
                [-half, -half, half],
                [-half, half, half],
                [-half, half, -half],
            ],
            // 3: back face z=-half.
            [
                [half, -half, -half],
                [-half, -half, -half],
                [-half, half, -half],
                [half, half, -half],
            ],
            // 4: top face y=+half.
            [
                [-half, half, half],
                [half, half, half],
                [half, half, -half],
                [-half, half, -half],
            ],
            // 5: bottom face y=-half.
            [
                [-half, -half, -half],
                [half, -half, -half],
                [half, -half, half],
                [-half, -half, half],
            ],
        ];

        // Scale net so it fits the same display window as the folded cube.
        // Net spans from x=-1.5 to x=2.5 (4 units wide), y=-1.5 to y=1.5 (3 tall).
        // Cube spans ±0.5.  Map net scale to roughly 0.4 → 1.0 of display scale.
        let net_s = scale / 3.0;
        let cube_s = scale;

        for fi in 0..6usize {
            // Interpolate each corner between net and cube.
            let corners: Vec<(i32, i32)> = (0..4)
                .map(|ci| {
                    let [nx, ny, nz] = net[fi][ci];
                    let [cx2, cy2, cz] = cube[fi][ci];
                    let x = nx + (cx2 - nx) * t;
                    let y = ny + (cy2 - ny) * t;
                    let z = nz + (cz - nz) * t;
                    // Scale: lerp from net_s to cube_s.
                    let s = net_s + (cube_s - net_s) * t;
                    project(x, y, z, ax, ay, cx, cy, s)
                })
                .collect();

            // Draw the 4 edges of the face square.
            for ei in 0..4usize {
                let (x0, y0) = corners[ei];
                let (x1, y1) = corners[(ei + 1) % 4];
                draw_edge(grid, x0, y0, x1, y1);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — crystal blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(206, 224, 255);
const TINT_END: Color = Color::rgb(92, 142, 232);

/// Sample the theme gradient at `t` in `0.0..=1.0`.
fn sample_tint(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(
        lerp(TINT_START.r, TINT_END.r),
        lerp(TINT_START.g, TINT_END.g),
        lerp(TINT_START.b, TINT_END.b),
    )
}

/// Applies the theme's signature gradient to every cell the inner style drew,
/// drifting slowly with `time`. Styles stay monochrome-safe underneath: drop
/// the wrapper in [`styles`] for uncolored output.
struct Tinted<S>(S);

impl<S: ProgressStyle> ProgressStyle for Tinted<S> {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn theme(&self) -> &str {
        self.0.theme()
    }
    fn describe(&self) -> &str {
        self.0.describe()
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        self.0.render(grid, ctx)?;
        grid.enable_color_support();
        let (w, h) = grid.dimensions();
        for y in 0..h {
            for x in 0..w {
                let ch = grid.get_char(x, y);
                if ch != '\u{2800}' && ch != ' ' {
                    let t = (x as f32 / w.max(1) as f32 + ctx.time * 0.05).fract();
                    let tri = 1.0 - (2.0 * t - 1.0).abs();
                    let _ = grid.set_cell_color(x, y, sample_tint(tri));
                }
            }
        }
        Ok(())
    }
}

/// All styles in the `platonic` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per solid, in the order they appear in
/// the source: tetrahedron, cube, octahedron, dodecahedron, icosahedron,
/// merkaba, star-octangulum, cuboctahedron, nested-solids,
/// stellated-dodecahedron, unfolding-net.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Tetrahedron)),
        Box::new(Tinted(Cube)),
        Box::new(Tinted(Octahedron)),
        Box::new(Tinted(Dodecahedron)),
        Box::new(Tinted(Icosahedron)),
        Box::new(Tinted(Merkaba)),
        Box::new(Tinted(StarOctangulum)),
        Box::new(Tinted(Cuboctahedron)),
        Box::new(Tinted(NestedSolids)),
        Box::new(Tinted(StellatedDodecahedron)),
        Box::new(Tinted(UnfoldingNet)),
    ]
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::platonic::styles();
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
