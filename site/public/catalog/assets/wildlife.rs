//! `wildlife` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O wildlife.rs && ./wildlife [style-name]
//! ```

const DEFAULT_STYLE: &str = "galloping-horse";

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
    pub mod wildlife {
//! Wildlife-themed progress bars — ten visually distinct creatures rendered
//! in braille dots and block glyphs.
//!
//! Each style uses a fundamentally different drawing algorithm:
//! - `galloping-horse`    — running silhouette with cycling leg phases
//! - `butterfly`          — figure advancing with wings opening/closing
//! - `spider-web`         — radial + spiral web threads appear as progress fills
//! - `beehive`            — hexagonal comb cells fill one by one
//! - `peacock-fan`        — tail feathers fan out radially with eased spread
//! - `leaping-frog`       — parabolic arcs in two-phase jumps
//! - `octopus`            — central blob with undulating sine tentacles
//! - `owl`                — fixed head with blinking eyes and slow head-turn
//! - `murmuration`        — many tiny dots flowing in a starling-like wave
//! - `elephant`           — walking silhouette with swinging trunk

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — forest canopy green. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(144, 222, 104);
const TINT_END: Color = Color::rgb(44, 144, 92);

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

/// All styles in the `wildlife` theme.
///
/// Returns one boxed [`ProgressStyle`] per creature variant. Every style is
/// stateless — all animation comes from `ctx.time` and `ctx.eased`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(GallopingHorse),
        Box::new(Butterfly),
        Box::new(Tinted(SpiderWeb)),
        Box::new(Beehive),
        Box::new(PeacockFan),
        Box::new(Tinted(LeapingFrog)),
        Box::new(Octopus),
        Box::new(Tinted(Owl)),
        Box::new(Murmuration),
        Box::new(Elephant),
    ]
}

// ── Galloping Horse ──────────────────────────────────────────────────────────

struct GallopingHorse;
impl ProgressStyle for GallopingHorse {
    fn name(&self) -> &str {
        "galloping-horse"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Horse galloping rightward — body advances with progress, four legs cycle through a gait"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = h.saturating_sub(1);
        let mid = h / 2;

        // Head of horse advances with progress.
        let head_x = (ctx.eased * w as f32) as usize;
        let head_x = head_x.min(w.saturating_sub(1));

        // Gait phase: four-beat gallop cycle driven by time.
        let gait = ctx.time * 8.0;

        // Draw hoofprints (dust trail) every 12 dots behind the horse.
        let step = 12usize;
        let mut tx = 0usize;
        while tx + step < head_x {
            let mark_x = tx + step / 2;
            draw::dot(grid, mark_x.min(w - 1), base);
            draw::dot(grid, (mark_x + 1).min(w - 1), base);
            tx += step;
        }

        // --- Horse body silhouette centered at head_x - 4 ---
        let bx = head_x.saturating_sub(4);

        // Body: a horizontal blob (3 dots wide, 2 tall at mid).
        for dx in 0..4usize {
            let px = bx.saturating_sub(dx);
            if px < w {
                draw::dot(grid, px, mid);
                if mid + 1 < h {
                    draw::dot(grid, px, mid + 1);
                }
            }
        }

        // Neck & head — above and ahead of body.
        let neck_x = (bx + 1).min(w.saturating_sub(1));
        draw::dot_i(grid, neck_x as i32 + 1, mid as i32 - 1);
        draw::dot_i(grid, neck_x as i32 + 2, mid as i32 - 2);
        draw::dot_i(grid, neck_x as i32 + 3, mid as i32 - 2); // muzzle
                                                              // Mane dot.
        draw::dot_i(grid, neck_x as i32 + 1, mid as i32 - 2);

        // Tail — behind the body.
        let tail_x = bx.saturating_sub(4) as i32;
        let tail_wave = ((gait * 0.5).sin() * 1.5).round() as i32;
        draw::dot_i(grid, tail_x, mid as i32 + tail_wave);
        draw::dot_i(grid, tail_x - 1, mid as i32 + tail_wave + 1);

        // Four legs: two pairs, each alternating up/down.
        // Leg offsets from bx: front pair at bx+1/bx+2, rear at bx-1/bx-2.
        let leg_positions: [i32; 4] = [bx as i32 + 2, bx as i32 + 1, bx as i32 - 1, bx as i32 - 2];
        // Four-beat gait: each leg offset by quarter cycle.
        let quarter = PI / 2.0;
        for (i, &lx) in leg_positions.iter().enumerate() {
            let phase = gait + i as f32 * quarter;
            // Hoof height: 0 = fully raised, 1 = on ground.
            let lift = (phase.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let foot_y = mid as i32 + 1 + (lift * (base as f32 - mid as f32 - 1.0)).round() as i32;
            let knee_y = mid as i32 + 1;
            // Upper leg.
            draw::dot_i(grid, lx, knee_y);
            // Lower leg + hoof.
            draw::dot_i(grid, lx, foot_y);
            if foot_y < base as i32 {
                draw::dot_i(grid, lx, foot_y + 1); // hoof ground touch
            }
        }

        // Palette tint: sweep up to head.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cells = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            for cx in 0..filled_cells.min(cells_w) {
                let t = if filled_cells <= 1 {
                    0.5
                } else {
                    cx as f32 / (filled_cells - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Butterfly ────────────────────────────────────────────────────────────────

struct Butterfly;
impl ProgressStyle for Butterfly {
    fn name(&self) -> &str {
        "butterfly"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Butterfly advancing with wings that open and close on a time-driven flap cycle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mid = (h / 2) as i32;

        // Body advances with progress.
        let body_x = (ctx.eased * w as f32) as usize;
        let body_x = body_x.min(w.saturating_sub(1)) as i32;

        // Wing flap: cos so wings are fully open at flap=1, closed at flap=0.
        let flap_t = (ctx.time * 6.0).cos().abs(); // 0..1, bounces
        let wing_span = ((w / 4) as f32 * flap_t).round() as i32;
        let wing_h = ((h / 2) as f32 * flap_t).round() as i32;

        // Body: three dots vertically.
        draw::dot_i(grid, body_x, mid - 1);
        draw::dot_i(grid, body_x, mid);
        draw::dot_i(grid, body_x, mid + 1);

        // Upper wings: fan of dots spanning left and right above mid.
        for s in 1..=wing_span.max(1) {
            let s_frac = if wing_span <= 1 {
                1.0
            } else {
                s as f32 / wing_span as f32
            };
            let wy = mid - (s_frac * wing_h as f32).round() as i32;
            draw::dot_i(grid, body_x - s, wy);
            draw::dot_i(grid, body_x + s, wy);
            // Wing edge — outermost dot.
            if s == wing_span {
                draw::dot_i(grid, body_x - s, wy + 1);
                draw::dot_i(grid, body_x + s, wy + 1);
            }
        }

        // Lower wings: smaller, below mid.
        let lower_span = (wing_span * 2 / 3).max(0);
        let lower_h = (wing_h / 2).max(0);
        for s in 1..=lower_span.max(1) {
            let s_frac = if lower_span <= 1 {
                1.0
            } else {
                s as f32 / lower_span as f32
            };
            let wy = mid + 1 + (s_frac * lower_h as f32).round() as i32;
            draw::dot_i(grid, body_x - s, wy);
            draw::dot_i(grid, body_x + s, wy);
        }

        // Antennae (always visible, slight wave).
        let ant_wave = ((ctx.time * 4.0).sin() * 0.5).round() as i32;
        draw::dot_i(grid, body_x - 1, mid - 2 + ant_wave);
        draw::dot_i(grid, body_x + 1, mid - 2 - ant_wave);
        draw::dot_i(grid, body_x - 2, mid - 3 + ant_wave);
        draw::dot_i(grid, body_x + 2, mid - 3 - ant_wave);

        // Tint only the wing cells with palette color.
        let (cells_w, cells_h) = grid.dimensions();
        let center_cx = (body_x.max(0) as usize / 2).min(cells_w.saturating_sub(1));
        let wing_cells = (wing_span.max(0) as usize / 2 + 1).min(cells_w);
        let cx0 = center_cx.saturating_sub(wing_cells);
        let cx1 = (center_cx + wing_cells).min(cells_w.saturating_sub(1));
        for cy in 0..cells_h {
            for cx in cx0..=cx1 {
                let t = if cx1 == cx0 {
                    0.5
                } else {
                    (cx - cx0) as f32 / (cx1 - cx0) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Spider Web ───────────────────────────────────────────────────────────────

struct SpiderWeb;
impl ProgressStyle for SpiderWeb {
    fn name(&self) -> &str {
        "spider-web"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Spider weaving a radial web — spoke threads appear first, then concentric spiral rings fill with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = (w.min(h * 2) / 2).max(1) as f32;

        // Number of radial spokes.
        let n_spokes = 8usize;
        // Spokes appear one by one in the first 40% of progress.
        let spoke_progress = (ctx.eased * n_spokes as f32 / 0.4).min(n_spokes as f32);
        let full_spokes = spoke_progress as usize;

        // Draw radial spokes.
        for s in 0..full_spokes.min(n_spokes) {
            let angle = s as f32 * (2.0 * PI / n_spokes as f32);
            let r_max = max_r as i32;
            for r in 0..=r_max {
                let dx = (angle.cos() * r as f32).round() as i32;
                let dy = (angle.sin() * r as f32).round() as i32;
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Concentric ring threads fill in the remaining 60% of progress.
        let n_rings = 5usize;
        let ring_progress = ((ctx.eased - 0.4) / 0.6 * n_rings as f32).max(0.0);
        let full_rings = ring_progress as usize;
        let partial_ring_frac = ring_progress.fract();

        for ring in 0..full_rings.min(n_rings) {
            let r = (max_r * (ring + 1) as f32 / n_rings as f32).round() as i32;
            // Full ring: draw 64 points around circumference.
            for step in 0..64usize {
                let angle = step as f32 * (2.0 * PI / 64.0);
                let dx = (angle.cos() * r as f32).round() as i32;
                let dy = (angle.sin() * r as f32 * 0.5).round() as i32; // squish vertically
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Partial ring for the currently-being-woven ring.
        if full_rings < n_rings {
            let r = (max_r * (full_rings + 1) as f32 / n_rings as f32).round() as i32;
            let steps = (partial_ring_frac * 64.0) as usize;
            for step in 0..steps {
                let angle = step as f32 * (2.0 * PI / 64.0);
                let dx = (angle.cos() * r as f32).round() as i32;
                let dy = (angle.sin() * r as f32 * 0.5).round() as i32;
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Spider: a small dot near the center, moving outward on one spoke with time.
        let spider_spoke_angle = (ctx.time * 0.8).rem_euclid(2.0 * PI);
        let spider_r = (max_r * 0.5 * (0.5 + 0.5 * (ctx.time * 0.7).sin())) as i32;
        let sx = cx + (spider_spoke_angle.cos() * spider_r as f32).round() as i32;
        let sy = cy + (spider_spoke_angle.sin() * spider_r as f32 * 0.5).round() as i32;
        draw::dot_i(grid, sx, sy);
        draw::dot_i(grid, sx + 1, sy);
        draw::dot_i(grid, sx, sy + 1);
        Ok(())
    }
}

// ── Beehive ──────────────────────────────────────────────────────────────────

struct Beehive;
impl ProgressStyle for Beehive {
    fn name(&self) -> &str {
        "beehive"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Honeycomb cells filling one by one — each hexagonal cell appears as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Hexagonal cell size in dots: width=6, height=4 (pointy-top hex approximation).
        let cell_w = 6usize;
        let cell_h = 4usize;
        let cols = (w / cell_w).max(1);
        let rows = (h / cell_h).max(1);
        let total_cells = cols * rows;

        // How many cells are lit?
        let lit = (ctx.eased * total_cells as f32).round() as usize;

        for idx in 0..lit.min(total_cells) {
            let col = idx % cols;
            let row = idx / cols;
            // Offset odd rows by half cell width for hex stagger.
            let x_off = if row % 2 == 1 { cell_w / 2 } else { 0 };
            let x0 = col * cell_w + x_off;
            let y0 = row * cell_h;

            // Draw a simple hex outline (6-sided) in dot space.
            // Points: use flat-top hex with half-width horizontal edges.
            let hw = (cell_w / 2) as i32;
            let hh = (cell_h / 2) as i32;
            let cx = (x0 + cell_w / 2) as i32;
            let cy = (y0 + cell_h / 2) as i32;

            // Top & bottom horizontal edges.
            for dx in -hw + 1..hw {
                draw::dot_i(grid, cx + dx, cy - hh);
                draw::dot_i(grid, cx + dx, cy + hh);
            }
            // Left & right diagonal sides.
            for dy in -hh..=hh {
                let frac = dy.abs() as f32 / hh.max(1) as f32;
                let x_indent = (frac * 1.0).round() as i32;
                draw::dot_i(grid, cx - hw + x_indent, cy + dy);
                draw::dot_i(grid, cx + hw - x_indent, cy + dy);
            }

            // Fill interior with progress-gated shimmer.
            let fill_frac = ((idx + 1) as f32 / total_cells.max(1) as f32).min(1.0);
            let pulse = (ctx.time * 4.0 + fill_frac * PI * 2.0).sin() * 0.3 + 0.7;
            if pulse > 0.6 {
                // Solid fill.
                for dy in -hh + 1..hh {
                    let indent = (dy.abs() as f32 / hh.max(1) as f32).round() as i32;
                    for dx in -hw + 1 + indent..hw - indent {
                        draw::dot_i(grid, cx + dx, cy + dy);
                    }
                }
            }
        }

        // Palette tint: left to right across the bar.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cx = (ctx.eased * cells_w as f32) as usize;
        for cy_c in 0..cells_h {
            for cx_c in 0..filled_cx.min(cells_w) {
                let t = if filled_cx <= 1 {
                    0.5
                } else {
                    cx_c as f32 / (filled_cx - 1) as f32
                };
                draw::tint_row(grid, cy_c, cx_c, cx_c, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Peacock Fan ──────────────────────────────────────────────────────────────

struct PeacockFan;
impl ProgressStyle for PeacockFan {
    fn name(&self) -> &str {
        "peacock-fan"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Peacock fanning its tail — radial feathers spread from a pivot point as progress increases"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Pivot at bottom-center.
        let px = (w / 2) as i32;
        let py = (h - 1) as i32;
        let max_feather_len = ((h as f32 * 0.9) as i32).max(1);

        // Fan arc: spans from angle_min to angle_max (in radians from vertical).
        // Progress controls how wide the fan is spread.
        let half_fan = ctx.eased * PI * 0.7; // up to 126° each side
        let n_feathers = 12usize;

        for f in 0..n_feathers {
            let t = if n_feathers <= 1 {
                0.5
            } else {
                f as f32 / (n_feathers - 1) as f32
            };
            // Angle from straight-up (-PI/2), distributed across the fan.
            let angle = -PI / 2.0 + (t - 0.5) * 2.0 * half_fan;

            // Feather only appears once enough progress to include it.
            if t > ctx.eased + 0.05 {
                continue;
            }

            // Feather length: outermost feathers slightly shorter.
            let edge = 1.0 - (2.0 * t - 1.0).abs() * 0.25;
            let flen = (max_feather_len as f32 * edge) as i32;

            // Draw feather shaft.
            for r in 0..=flen {
                let dx = (angle.cos() * r as f32 * 1.5).round() as i32; // stretch horizontally
                let dy = (angle.sin() * r as f32).round() as i32;
                draw::dot_i(grid, px + dx, py + dy);
            }

            // Eye spot at tip: oscillates color/dot with time.
            let eye_r = flen;
            let edx = (angle.cos() * eye_r as f32 * 1.5).round() as i32;
            let edy = (angle.sin() * eye_r as f32).round() as i32;
            let pulse = ((ctx.time * 3.0 + t * PI * 2.0).sin() * 0.5 + 0.5) > 0.4;
            if pulse {
                draw::dot_i(grid, px + edx + 1, py + edy);
                draw::dot_i(grid, px + edx - 1, py + edy);
                draw::dot_i(grid, px + edx, py + edy - 1);
            }
        }

        // Peacock body: small vertical column at pivot.
        for dy in 0..=(h / 4) as i32 {
            draw::dot_i(grid, px, py - dy);
        }
        // Head.
        draw::dot_i(grid, px, py - (h / 4) as i32 - 1);
        draw::dot_i(grid, px + 1, py - (h / 4) as i32 - 2); // crest

        // Palette tint across full bar width.
        let (cells_w, cells_h) = grid.dimensions();
        for cy_c in 0..cells_h {
            for cx_c in 0..cells_w {
                let t = if cells_w <= 1 {
                    0.5
                } else {
                    cx_c as f32 / (cells_w - 1) as f32
                };
                let color = ctx.palette.sample(t * ctx.eased);
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ── Leaping Frog ─────────────────────────────────────────────────────────────

struct LeapingFrog;
impl ProgressStyle for LeapingFrog {
    fn name(&self) -> &str {
        "leaping-frog"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Frog leaping in two-phase parabolic arcs — each jump leaves a ripple splash on landing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let base = (h - 1) as i32;

        // Number of lily-pad stops (jump targets).
        let n_jumps = 4usize;
        let jump_w = w / n_jumps.max(1);

        // Which jump are we in?
        let jump_idx_f = ctx.eased * n_jumps as f32;
        let current_jump = (jump_idx_f as usize).min(n_jumps.saturating_sub(1));
        let jump_frac = jump_idx_f.fract();

        // Draw lily pads at each destination.
        for j in 0..=current_jump {
            let pad_x = (j * jump_w + jump_w / 2) as i32;
            // Lily pad: small oval on base line.
            draw::dot_i(grid, pad_x - 1, base);
            draw::dot_i(grid, pad_x, base);
            draw::dot_i(grid, pad_x + 1, base);
            draw::dot_i(grid, pad_x, base - 1);

            // Splash ripple on landing: fades over time.
            if j < current_jump {
                // Already landed — static rings.
                draw::dot_i(grid, pad_x - 2, base);
                draw::dot_i(grid, pad_x + 2, base);
                draw::dot_i(grid, pad_x, base - 2);
            }
        }

        // Frog arc position during current jump.
        let start_x = (current_jump * jump_w) as i32;
        let end_x = ((current_jump + 1) * jump_w) as i32;
        let frog_x = start_x + ((jump_frac * (end_x - start_x) as f32) as i32);

        // Parabolic height: peak at jump_frac = 0.5.
        let peak_h = (h as f32 * 0.65) as i32;
        let arc_h = (4.0 * peak_h as f32 * jump_frac * (1.0 - jump_frac)) as i32;
        let frog_y = base - arc_h;
        let frog_y = frog_y.max(0);

        // Frog body.
        draw::dot_i(grid, frog_x, frog_y);
        draw::dot_i(grid, frog_x + 1, frog_y);
        draw::dot_i(grid, frog_x, frog_y + 1);
        draw::dot_i(grid, frog_x + 1, frog_y + 1);

        // Eyes on top.
        draw::dot_i(grid, frog_x - 1, frog_y - 1);
        draw::dot_i(grid, frog_x + 2, frog_y - 1);

        // Legs: extend outward during jump, tucked when crouching.
        let leg_ext = (arc_h as f32 / peak_h.max(1) as f32 * 2.0).min(2.0) as i32;
        // Front legs.
        draw::dot_i(grid, frog_x - 1 - leg_ext, frog_y + 1);
        draw::dot_i(grid, frog_x + 2 + leg_ext, frog_y + 1);
        // Rear legs.
        draw::dot_i(grid, frog_x - 1, frog_y + 2 + leg_ext);
        draw::dot_i(grid, frog_x + 2, frog_y + 2 + leg_ext);

        Ok(())
    }
}

// ── Octopus ───────────────────────────────────────────────────────────────────

struct Octopus;
impl ProgressStyle for Octopus {
    fn name(&self) -> &str {
        "octopus"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Octopus with eight undulating tentacles — the filled region reveals more arms as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Mantle (body) centered horizontally at progress position.
        let cx = (ctx.eased * w as f32) as i32;
        let cx = cx.clamp(0, w as i32 - 1);
        let cy = (h / 3) as i32;

        // Mantle: oval blob.
        let mw = ((w / 8).max(2) as i32).min(6);
        let mh = ((h / 4).max(1) as i32).min(4);
        for dy in -mh..=mh {
            let x_extent =
                (mw as f32 * (1.0 - (dy as f32 / mh.max(1) as f32).powi(2)).sqrt()).round() as i32;
            for dx in -x_extent..=x_extent {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Eyes.
        draw::dot_i(grid, cx - mw / 2, cy - mh / 2);
        draw::dot_i(grid, cx + mw / 2, cy - mh / 2);

        // Eight tentacles: angle them downward, sine-undulate with time.
        let n_arms = 8usize;
        // Number of arms visible = scaled with progress.
        let visible_arms = (ctx.eased * n_arms as f32).round() as usize;
        let arm_len = ((h as f32 * 0.7) as usize).max(2);

        for a in 0..visible_arms.min(n_arms) {
            // Spread arms evenly across the bottom semicircle.
            let t = if n_arms <= 1 {
                0.5
            } else {
                a as f32 / (n_arms - 1) as f32
            };
            let base_angle = PI * t; // 0 → PI (left to right below mantle)
            let arm_phase = ctx.time * 2.5 + a as f32 * 0.8;

            // Draw arm segment by segment, applying sine wave lateral offset.
            let arm_x0 = cx + ((t * 2.0 - 1.0) * mw as f32).round() as i32;
            let arm_y0 = cy + mh;

            for step in 0..arm_len {
                let frac = step as f32 / arm_len.max(1) as f32;
                // Direction angle: fans outward then curves back down.
                let dir_angle = base_angle + (0.5 - t).abs() * PI * 0.3 * frac;
                let lateral = (arm_phase + frac * PI * 2.0).sin() * frac * 2.0;
                let dx = (dir_angle.cos() * frac * mw as f32 * 1.5 + lateral).round() as i32;
                let dy =
                    (frac * arm_len as f32 * 0.8 + (1.0 - frac) * dir_angle.sin()).round() as i32;
                draw::dot_i(grid, arm_x0 + dx, arm_y0 + dy);
                // Sucker: every 3 steps, a dot perpendicular.
                if step % 3 == 0 {
                    draw::dot_i(grid, arm_x0 + dx + 1, arm_y0 + dy);
                }
            }
        }

        // Palette tint.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cx = (ctx.eased * cells_w as f32) as usize;
        for cy_c in 0..cells_h {
            for cx_c in 0..filled_cx.min(cells_w) {
                let t = if filled_cx <= 1 {
                    0.5
                } else {
                    cx_c as f32 / (filled_cx - 1) as f32
                };
                draw::tint_row(grid, cy_c, cx_c, cx_c, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Owl ──────────────────────────────────────────────────────────────────────

struct Owl;
impl ProgressStyle for Owl {
    fn name(&self) -> &str {
        "owl"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Owl with blinking eyes and a slow head-turn — tracks progress by rotating to face rightward"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Owl body centered at progress position.
        let body_cx = (ctx.eased * w as f32) as i32;
        let body_cx = body_cx.clamp(0, w as i32 - 1);
        let body_cy = (h / 2) as i32;

        // Body: tall oval.
        let bw = ((w / 10).max(1) as i32).min(4);
        let bh = ((h / 2).max(2) as i32).min(6);
        for dy in -bh..=bh {
            let x_ext =
                (bw as f32 * (1.0 - (dy as f32 / bh.max(1) as f32).powi(2)).sqrt()).round() as i32;
            for dx in -x_ext..=x_ext {
                draw::dot_i(grid, body_cx + dx, body_cy + dy);
            }
        }

        // Head: slightly smaller oval above body.
        let head_cy = body_cy - bh - 1;
        let hw = (bw - 1).max(1);
        let hh = (bh / 2).max(1);
        for dy in -hh..=hh {
            let x_ext =
                (hw as f32 * (1.0 - (dy as f32 / hh.max(1) as f32).powi(2)).sqrt()).round() as i32;
            for dx in -x_ext..=x_ext {
                draw::dot_i(grid, body_cx + dx, head_cy + dy);
            }
        }

        // Ear tufts.
        draw::dot_i(grid, body_cx - hw, head_cy - hh - 1);
        draw::dot_i(grid, body_cx + hw, head_cy - hh - 1);

        // Blink: eyes appear for most of the time, close briefly every ~3s.
        let blink_cycle = (ctx.time * 1.2).fract();
        let eyes_open = blink_cycle > 0.08; // closed 8% of cycle

        // Head-turn: gaze direction follows progress — look right as bar fills.
        let gaze_x_off = ((ctx.eased * 2.0 - 1.0) * hw as f32 * 0.5).round() as i32;

        if eyes_open {
            // Eyes: two dots with offset based on gaze.
            draw::dot_i(grid, body_cx - hw / 2 + gaze_x_off, head_cy);
            draw::dot_i(grid, body_cx + hw / 2 + gaze_x_off, head_cy);
            // Pupil highlight.
            draw::dot_i(grid, body_cx - hw / 2 + gaze_x_off + 1, head_cy - 1);
            draw::dot_i(grid, body_cx + hw / 2 + gaze_x_off + 1, head_cy - 1);
        } else {
            // Closed eyes: a horizontal line.
            draw::dot_i(grid, body_cx - hw / 2, head_cy);
            draw::dot_i(grid, body_cx - hw / 2 + 1, head_cy);
            draw::dot_i(grid, body_cx + hw / 2 - 1, head_cy);
            draw::dot_i(grid, body_cx + hw / 2, head_cy);
        }

        // Beak.
        draw::dot_i(grid, body_cx + gaze_x_off, head_cy + 1);

        // Wings: outstretched slightly when progress > 0.5.
        if ctx.eased > 0.5 {
            let wing_ext = ((ctx.eased - 0.5) * 2.0 * bw as f32 * 2.0).round() as i32;
            draw::dot_i(grid, body_cx - bw - wing_ext, body_cy);
            draw::dot_i(grid, body_cx + bw + wing_ext, body_cy);
            draw::dot_i(grid, body_cx - bw - wing_ext + 1, body_cy + 1);
            draw::dot_i(grid, body_cx + bw + wing_ext - 1, body_cy + 1);
        }

        // Talons at base.
        draw::dot_i(grid, body_cx - 1, body_cy + bh + 1);
        draw::dot_i(grid, body_cx, body_cy + bh + 1);
        draw::dot_i(grid, body_cx + 1, body_cy + bh + 1);

        Ok(())
    }
}

// ── Murmuration ───────────────────────────────────────────────────────────────

struct Murmuration;
impl ProgressStyle for Murmuration {
    fn name(&self) -> &str {
        "murmuration"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Starling murmuration — a dense wave of dots that rolls across the bar in flowing sine sheets"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Front of the flock advances with progress.
        let front_x = (ctx.eased * w as f32) as usize;
        let flock_depth = (w / 2).max(4);

        // Flock: a cloud of dots with per-dot sine offsets.
        // We generate a deterministic but chaotic-looking distribution.
        let n_birds = (front_x / 3).clamp(1, 60);

        for b in 0..n_birds {
            // Each bird has a unique phase and speed.
            let seed = b as f32;
            let x_offset = (seed * 7.3 + ctx.time * (1.5 + (seed * 0.13).fract() * 2.0))
                .rem_euclid(flock_depth as f32);
            let bx = front_x.saturating_sub(x_offset as usize);
            if bx >= w {
                continue;
            }

            // Y: three-layer sine (simulates turbulent flocking).
            let y_phase = seed * 1.7 + ctx.time * (2.0 + (seed * 0.07).fract());
            let y_frac = 0.5 + 0.35 * (y_phase).sin() + 0.15 * (y_phase * 2.3 + seed * 0.9).sin();
            let by = (y_frac * (h - 1) as f32).round() as usize;
            let by = by.min(h - 1);

            draw::dot(grid, bx, by);

            // Wing tip dots (brief v-shape).
            let wing_beat = (ctx.time * 6.0 + seed * 0.5).sin();
            if wing_beat > 0.2 {
                draw::dot_i(grid, bx as i32 - 1, by as i32 - 1);
                draw::dot_i(grid, bx as i32 + 1, by as i32 - 1);
            }
        }

        // Faint trailing density: shade the track with palette.
        let (cells_w, cells_h) = grid.dimensions();
        let front_cell = (ctx.eased * cells_w as f32) as usize;
        for cy in 0..cells_h {
            for cx in 0..front_cell.min(cells_w) {
                let t = if front_cell <= 1 {
                    0.5
                } else {
                    cx as f32 / (front_cell - 1) as f32
                };
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ── Elephant ─────────────────────────────────────────────────────────────────

struct Elephant;
impl ProgressStyle for Elephant {
    fn name(&self) -> &str {
        "elephant"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Elephant walking rightward — four legs cycle in a slow plod, trunk swings with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let base = (h - 1) as i32;
        // Elephant is larger — body is proportionally big.
        let body_h = ((h * 2 / 3).max(2) as i32).min(8);
        let body_w = ((w / 6).max(3) as i32).min(10);

        // Position: leading edge (trunk tip) at progress × width.
        let trunk_tip_x = (ctx.eased * w as f32) as i32;
        let trunk_tip_x = trunk_tip_x.clamp(0, w as i32 - 1);

        // Body center is behind the trunk.
        let body_cx = (trunk_tip_x - body_w / 2 - 2).max(0);
        let body_top = base - body_h;

        // Body rectangle (filled solid so it's clearly an elephant silhouette).
        for dy in 0..=body_h {
            let x_w = if dy == 0 || dy == body_h {
                body_w * 2 / 3
            } else {
                body_w
            };
            for dx in -x_w..=x_w {
                draw::dot_i(grid, body_cx + dx, body_top + dy);
            }
        }

        // Head: forward of body, rounded.
        let head_cx = body_cx + body_w + 1;
        let head_r = (body_h / 3).max(1);
        for dy in -head_r..=head_r {
            let x_ext = (head_r as f32 * (1.0 - (dy as f32 / head_r.max(1) as f32).powi(2)).sqrt())
                .round() as i32;
            for dx in -x_ext..=x_ext {
                draw::dot_i(grid, head_cx + dx, base - body_h / 2 + dy);
            }
        }

        // Eye.
        draw::dot_i(grid, head_cx + head_r / 2, base - body_h / 2 - head_r / 2);

        // Ear: large floppy disc behind and above head.
        let ear_cx = head_cx - head_r;
        let ear_cy = base - body_h / 2 - head_r / 2;
        for dy in -head_r - 1..=head_r {
            let x_ext = ((head_r + 1) as f32
                * (1.0 - (dy as f32 / (head_r + 1).max(1) as f32).powi(2)).sqrt())
            .round() as i32;
            draw::dot_i(grid, ear_cx - x_ext / 2, ear_cy + dy);
        }

        // Trunk: hangs from head, swings side-to-side with time.
        let trunk_swing = (ctx.time * 1.8).sin();
        let trunk_len = (body_h / 2 + 2).max(2) as i32;
        let trunk_base_x = head_cx + head_r;
        let trunk_base_y = base - body_h / 2 + head_r / 2;
        for step in 0..=trunk_len {
            let frac = step as f32 / trunk_len.max(1) as f32;
            let swing_x = (trunk_swing * frac * 2.0 * head_r as f32).round() as i32;
            let tx = trunk_base_x + swing_x;
            let ty = trunk_base_y + step;
            draw::dot_i(grid, tx, ty);
        }
        // Trunk tip curl.
        let tip_x = trunk_base_x + (trunk_swing * 2.0 * head_r as f32).round() as i32;
        let tip_y = trunk_base_y + trunk_len;
        draw::dot_i(grid, tip_x + 1, tip_y - 1);
        draw::dot_i(grid, tip_x + 1, tip_y);

        // Tail: short stub at back.
        let tail_x = body_cx - body_w;
        let tail_wave = ((ctx.time * 2.5).sin() * 1.0).round() as i32;
        draw::dot_i(grid, tail_x - 1, base - body_h + tail_wave);
        draw::dot_i(grid, tail_x - 2, base - body_h + tail_wave + 1);

        // Four legs: plodding walk cycle.
        let plod = ctx.time * 2.5;
        let half = PI;
        let leg_positions: [i32; 4] = [
            body_cx + body_w / 2,
            body_cx + body_w / 4,
            body_cx - body_w / 4,
            body_cx - body_w / 2,
        ];
        let leg_len = (base - (base - body_h + body_h / 3)).max(1);
        for (i, &lx) in leg_positions.iter().enumerate() {
            let phase = plod + i as f32 * half / 2.0;
            // Lift = 0 (ground) to 1 (raised).
            let lift = (phase.sin().max(0.0) * 0.6) as i32;
            let leg_top = base - body_h + body_h / 3;
            let leg_bot = base - lift;
            draw::dot_i(grid, lx, leg_top);
            draw::dot_i(grid, lx, (leg_top + leg_len / 2).min(leg_bot));
            draw::dot_i(grid, lx, leg_bot.max(leg_top));
            // Hoof: wider at bottom.
            draw::dot_i(grid, lx - 1, leg_bot.max(leg_top));
            draw::dot_i(grid, lx + 1, leg_bot.max(leg_top));
        }

        // Palette tint behind the elephant.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cx = (ctx.eased * cells_w as f32) as usize;
        for cy_c in 0..cells_h {
            for cx_c in 0..filled_cx.min(cells_w) {
                let t = if filled_cx <= 1 {
                    0.5
                } else {
                    cx_c as f32 / (filled_cx - 1) as f32
                };
                draw::tint_row(grid, cy_c, cx_c, cx_c, ctx.palette.sample(t));
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
    let styles = progress::styles::wildlife::styles();
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
