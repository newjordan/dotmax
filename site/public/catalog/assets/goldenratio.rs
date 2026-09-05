//! `goldenratio` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O goldenratio.rs && ./goldenratio [style-name]
//! ```

const DEFAULT_STYLE: &str = "golden-spiral";

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
    pub mod goldenratio {
//! Golden-ratio sacred-geometry progress bars.
//!
//! Every style is built from φ (the golden ratio) and its consequences:
//! Fibonacci tiling, the logarithmic spiral, pentagonal symmetry, the golden
//! gnomon, Kepler's triangle, and the 137.5° golden angle. All shapes are
//! constructed with exact φ-based mathematics and revealed by `ctx.eased`
//! while `ctx.time` drives continuous rotation or animation.
//!
//! Style catalogue:
//! - `golden-spiral`        — quarter-circle arcs in Fibonacci squares forming the logarithmic spiral
//! - `golden-rectangle`     — recursive whirling-rectangles φ-subdivision
//! - `fibonacci-squares`    — outward tiling of Fibonacci-sized squares
//! - `pentagram`            — nested pentagram-in-pentagon revealing φ self-similarity
//! - `pentagon-nest`        — concentric pentagons scaled by 1/φ², rotating
//! - `golden-gnomon`        — 36-72-72 golden triangle recursive subdivision
//! - `phi-phyllotaxis-pent` — five-fold seed arrangement at golden angle
//! - `nautilus`             — chambered logarithmic spiral r=a·φ^(θ·2/π) with cross-walls
//! - `golden-angle-rays`    — rays at successive 137.5° increments, length growing
//! - `dodecagram`           — 12-pointed φ-star (string-art on a 12-gon)
//! - `kepler-triangle`      — right triangle with sides 1, √φ, φ (recursive fan)

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// The golden ratio φ = (1 + √5) / 2.
const PHI: f32 = 1.618_033_9;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — burnished gold.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 204, 92);
const TINT_END: Color = Color::rgb(196, 120, 28);

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

/// All styles in the `goldenratio` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per golden-ratio / φ-geometry bar.
/// Styles are ordered from the most recognisable (golden spiral) to the most
/// exotic (Kepler triangle fan), but are fully independent.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(GoldenSpiral)),
        Box::new(Tinted(GoldenRectangle)),
        Box::new(Tinted(FibonacciSquares)),
        Box::new(Tinted(Pentagram)),
        Box::new(Tinted(PentagonNest)),
        Box::new(Tinted(GoldenGnomon)),
        Box::new(Tinted(PhiPhyllotaxisPent)),
        Box::new(Tinted(Nautilus)),
        Box::new(Tinted(GoldenAngleRays)),
        Box::new(Tinted(Dodecagram)),
        Box::new(Tinted(KeplerTriangle)),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Grid center in dot-space (floating-point).
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

/// Uniform scale so a unit-radius figure fits the grid with a small margin.
#[inline]
fn fit_scale(dw: usize, dh: usize) -> f32 {
    let hw = (dw as f32 / 2.0 - 1.0).max(1.0);
    let hh = (dh as f32 / 2.0 - 1.0).max(1.0);
    hw.min(hh)
}

/// Integer Bresenham line between two signed dot-space points.
/// Out-of-bounds dots are silently discarded by `draw::dot_i`.
fn bresenham(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a circular arc from angle `a0` to `a1` (radians), radius `r` dots,
/// centered at dot-space `(cx, cy)`.  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r < 0.5 {
        return;
    }
    let arc_len = (a1 - a0).abs() * r;
    let steps = (arc_len * 2.0).round() as usize;
    let steps = steps.max(2);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = a0 + t * (a1 - a0);
        let px = (cx + r * angle.cos()).round() as i32;
        let py = (cy - r * angle.sin()).round() as i32;
        draw::dot_i(grid, px, py);
    }
}

/// Draw a regular N-gon in dot-space, centered at (cx, cy), radius r dots,
/// base rotation `rot` radians. Returns the vertex coordinates.
fn ngon_vertices(n: usize, cx: f32, cy: f32, r: f32, rot: f32) -> Vec<(i32, i32)> {
    (0..n)
        .map(|i| {
            let angle = rot + 2.0 * PI * i as f32 / n as f32;
            (
                (cx + r * angle.cos()).round() as i32,
                (cy - r * angle.sin()).round() as i32,
            )
        })
        .collect()
}

/// Draw the outline of a regular N-gon.
fn ngon_outline(grid: &mut BrailleGrid, n: usize, cx: f32, cy: f32, r: f32, rot: f32) {
    let verts = ngon_vertices(n, cx, cy, r, rot);
    for i in 0..n {
        let (x0, y0) = verts[i];
        let (x1, y1) = verts[(i + 1) % n];
        bresenham(grid, x0, y0, x1, y1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Golden Spiral — quarter-circle arcs inscribed in Fibonacci squares
// ─────────────────────────────────────────────────────────────────────────────
//
// Each Fibonacci square hosts a quarter-circle arc whose radius equals the
// square's side length.  The squares tile outward in the canonical F1,F1,F2,
// F3,F5,F8… pattern; `ctx.eased` gates how many squares are revealed.

struct GoldenSpiral;
impl ProgressStyle for GoldenSpiral {
    fn name(&self) -> &str {
        "golden-spiral"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Fibonacci-square tiling with quarter-circle arcs forming the golden logarithmic \
         spiral; squares and arcs revealed one step per eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let scale = fit_scale(dw, dh);

        // Pre-compute up to 9 Fibonacci numbers (normalised so the two seeds = 1).
        // We store the relative sizes: 1,1,2,3,5,8,13,21,34.
        // The total rectangle after N squares has dimensions F[N] × F[N+1].
        // We normalise by the largest F so the whole thing fits in `scale` dots.
        let fibs: [f32; 9] = [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0];
        let n_total: usize = 9;
        let n_show = ((ctx.eased * n_total as f32).round() as usize).min(n_total);

        // The bounding rectangle after n_total squares has long side = fibs[n_total-1].
        let long = fibs[n_total - 1];
        // Dot-space unit = scale / long * 2 (×2 for comfortable fit).
        let unit = (scale * 1.8 / long).max(0.5);

        // We build the tiling by tracking the current "pivot corner" in
        // normalised Fibonacci units, then convert to dot-space.
        // The spiral grows: right, up, left, down, right, ... starting from
        // the two unit squares sharing the top-left corner.
        //
        // Pivot meaning: the arc sweeps from the corner diagonally opposite
        // the previous square. We track in (u, v) normalised coords, then
        // map to dot-space centered on the grid.

        // Center of the first 1×1 square in normalised units (left square).
        // We'll accumulate the pivot in normalised space, offset to center later.
        // Row,col offsets in normalised Fibonacci units relative to origin.

        // Direction cycle: right, up, left, down.
        // After placing square of side s in direction d, arc sweeps from the
        // far corner of the previous square.
        //
        // Track the "arc pivot" (normalised) and the arc start angle.
        // We start at origin, facing right.
        // Direction cycle: right, down, left, up (norm-space: y positive = down).
        let dirs: [(f32, f32); 4] = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
        // Arc start angles — one quarter-circle sweep per square, clockwise.
        let arc_starts: [f32; 4] = [PI, 3.0 * PI / 2.0, 0.0, PI / 2.0];

        // Pivot = arc center in normalised units (starts at origin = grid center).
        let mut px_n: f32 = 0.0;
        let mut py_n: f32 = 0.0;
        let (dcx, dcy) = center(dw, dh);

        for (i, &side) in fibs.iter().take(n_show).enumerate() {
            let a_start = arc_starts[i % 4];
            let a_end = a_start - PI / 2.0; // quarter circle, clockwise in screen

            // Convert normalised pivot to dot-space.
            let arc_cx = dcx + px_n * unit;
            let arc_cy = dcy + py_n * unit; // norm y positive = down = dot-space y positive

            // Draw the outline of the square.
            let (ddx, ddy) = dirs[i % 4];
            // The square occupies from (px_n, py_n) extending in the direction
            // perpendicular to current movement.  Just draw the arc; suppress
            // the square outline for cleaner look (arc alone reads well).
            // But on first reveal draw the square too.
            {
                // Square corners relative to pivot in norm units depend on dir.
                // For direction 0 (right): square is to the right, corners at
                // (px_n, py_n), (px_n+s, py_n), (px_n+s, py_n-s), (px_n, py_n-s).
                // For simplicity, derive from dir and perp.
                let perp = match i % 4 {
                    0 => (0.0, -1.0), // right-moving: square extends up (norm)
                    1 => (1.0, 0.0),  // down-moving:  square extends right
                    2 => (0.0, 1.0),  // left-moving:  square extends down
                    _ => (-1.0, 0.0), // up-moving:    square extends left
                };
                let s = side;
                let c0 = (px_n, py_n);
                let c1 = (px_n + ddx * s, py_n + ddy * s);
                let c2 = (px_n + ddx * s + perp.0 * s, py_n + ddy * s + perp.1 * s);
                let c3 = (px_n + perp.0 * s, py_n + perp.1 * s);
                let corners = [c0, c1, c2, c3];
                for k in 0..4 {
                    let (ax, ay) = corners[k];
                    let (bx, by) = corners[(k + 1) % 4];
                    let p0x = (dcx + ax * unit).round() as i32;
                    let p0y = (dcy + ay * unit).round() as i32;
                    let p1x = (dcx + bx * unit).round() as i32;
                    let p1y = (dcy + by * unit).round() as i32;
                    bresenham(grid, p0x, p0y, p1x, p1y);
                }
            }

            // Draw the arc: radius = side * unit dots.
            arc(grid, arc_cx, arc_cy, side * unit, a_start, a_end);

            // Advance pivot.
            let (adx, ady) = match i % 4 {
                0 => (side, 0.0),  // moved right → pivot goes right by side
                1 => (0.0, side),  // moved down  → pivot goes down by side
                2 => (-side, 0.0), // moved left  → pivot goes left by side
                _ => (0.0, -side), // moved up    → pivot goes up by side
            };
            // Arc pivot for the NEXT step is at the far corner of the square just drawn.
            // That corner is pivot + current_dir*side (the arc center was at start pivot).
            px_n += adx;
            py_n += ady;
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Golden Rectangle — recursive whirling-rectangles φ-subdivision
// ─────────────────────────────────────────────────────────────────────────────
//
// Start with a φ-rectangle. Cut a square from the long side → leaves a smaller
// φ-rectangle. Recurse. The squares + remaining rectangles are the "whirling
// rectangles". `ctx.eased` controls recursion depth.

struct GoldenRectangle;
impl ProgressStyle for GoldenRectangle {
    fn name(&self) -> &str {
        "golden-rectangle"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Recursive φ-rectangle subdivision into a square + smaller φ-rectangle — \
         the whirling-rectangles diagram, with each subdivision revealed as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);

        // Fit the initial φ-rectangle (width = PHI, height = 1) into the grid.
        // Scale so width fits within dw, height within dh.
        let scale_w = (dw as f32 - 2.0) / PHI;
        let scale_h = (dh as f32 - 2.0) / 1.0;
        let unit = scale_w.min(scale_h).max(1.0);

        // Initial rectangle corners in dot-space (top-left origin).
        let rect_w = PHI * unit;
        let rect_h = 1.0 * unit;
        let ox = ((dw as f32 - rect_w) / 2.0).max(0.0);
        let oy = ((dh as f32 - rect_h) / 2.0).max(0.0);

        let max_depth: usize = 10;
        let depth = ((ctx.eased * max_depth as f32).round() as usize)
            .min(max_depth)
            .max(1);

        // Recursive subdivision: (x0, y0, w, h, horizontal_cut)
        // horizontal_cut=true means we cut a square from the left side (width = h).
        let rects: Vec<(f32, f32, f32, f32, bool)> = vec![(ox, oy, rect_w, rect_h, true)];
        let mut drawn = 0usize;

        // Draw outer rectangle always.
        {
            let (x0, y0, w, h, _) = rects[0];
            draw::rect_outline(
                grid,
                x0.round() as usize,
                y0.round() as usize,
                w.round().max(1.0) as usize,
                h.round().max(1.0) as usize,
            );
        }
        drawn += 1;

        let mut next_rects: Vec<(f32, f32, f32, f32, bool)> = Vec::new();
        let mut current = rects;

        for _d in 0..depth.saturating_sub(1) {
            next_rects.clear();
            for &(x0, y0, w, h, horiz) in &current {
                if horiz {
                    // Cut a square of side h from the left.
                    let sq = h;
                    if sq >= 1.0 && w - sq >= 1.0 {
                        // Draw the dividing line between square and remainder.
                        let lx = (x0 + sq).round() as usize;
                        let ly0 = y0.round() as usize;
                        let ly1 = (y0 + h - 1.0).round() as usize;
                        if lx < dw {
                            draw::vline(
                                grid,
                                lx,
                                ly0.min(dh.saturating_sub(1)),
                                ly1.min(dh.saturating_sub(1)),
                            );
                        }
                        // The remaining rectangle is vertical (h > w).
                        next_rects.push((x0 + sq, y0, w - sq, h, false));
                        drawn += 1;
                    }
                } else {
                    // Cut a square of side w from the top.
                    let sq = w;
                    if sq >= 1.0 && h - sq >= 1.0 {
                        let ly = (y0 + sq).round() as usize;
                        let lx0 = x0.round() as usize;
                        let lx1 = (x0 + w - 1.0).round() as usize;
                        if ly < dh {
                            draw::hline(
                                grid,
                                lx0.min(dw.saturating_sub(1)),
                                lx1.min(dw.saturating_sub(1)),
                                ly,
                            );
                        }
                        next_rects.push((x0, y0 + sq, w, h - sq, true));
                        drawn += 1;
                    }
                }
                if drawn >= depth * 2 {
                    break;
                }
            }
            if next_rects.is_empty() {
                break;
            }
            current.clone_from(&next_rects);
        }
        let _ = drawn;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Fibonacci Squares — outward tiling of F₁,F₁,F₂,F₃,F₅,F₈,…
// ─────────────────────────────────────────────────────────────────────────────
//
// Draws only the square outlines (not the arcs) so the tiling geometry is the
// hero.  Each new square appears as `ctx.eased` steps forward.  The tiling is
// centered and scaled to fit any grid.

struct FibonacciSquares;
impl ProgressStyle for FibonacciSquares {
    fn name(&self) -> &str {
        "fibonacci-squares"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Outward Fibonacci-square tiling (1,1,2,3,5,8,13,21,34…): each square \
         outline appears as progress advances, building the canonical φ-rectangle mosaic"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (dcx, dcy) = center(dw, dh);

        let fibs: [f32; 10] = [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0];
        let n_total: usize = 10;
        let n_show = ((ctx.eased * n_total as f32).round() as usize).min(n_total);

        // Scale: the long side of the 10-square rectangle = 55 units.
        let long = fibs[n_total - 1];
        let unit = ((dw.min(dh) as f32 - 2.0) / long * 1.6).max(0.5);

        // Pivot tracking: same as golden-spiral but only rectangles.
        let dirs: [(f32, f32); 4] = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
        let perps: [(f32, f32); 4] = [(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)];

        let mut px_n: f32 = 0.0;
        let mut py_n: f32 = 0.0;

        for (i, &side) in fibs.iter().take(n_show).enumerate() {
            let (ddx, ddy) = dirs[i % 4];
            let (ppx, ppy) = perps[i % 4];
            let s = side;
            // Four corners in norm units.
            let c0 = (px_n, py_n);
            let c1 = (px_n + ddx * s, py_n + ddy * s);
            let c2 = (px_n + ddx * s + ppx * s, py_n + ddy * s + ppy * s);
            let c3 = (px_n + ppx * s, py_n + ppy * s);
            let corners = [c0, c1, c2, c3];
            for k in 0..4 {
                let (ax, ay) = corners[k];
                let (bx, by) = corners[(k + 1) % 4];
                let p0x = (dcx + ax * unit).round() as i32;
                let p0y = (dcy + ay * unit).round() as i32;
                let p1x = (dcx + bx * unit).round() as i32;
                let p1y = (dcy + by * unit).round() as i32;
                bresenham(grid, p0x, p0y, p1x, p1y);
            }
            // Advance pivot.
            let step = match i % 4 {
                0 => (side, 0.0),
                1 => (0.0, side),
                2 => (-side, 0.0),
                _ => (0.0, -side),
            };
            px_n += step.0;
            py_n += step.1;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Pentagram — nested pentagram-in-pentagon (the infinite pentagram)
// ─────────────────────────────────────────────────────────────────────────────
//
// Each diagonal of a regular pentagon is φ times the side.  Inside any
// pentagram lives a smaller regular pentagon (scaled by 1/φ²), which in turn
// holds another pentagram. `ctx.eased` controls nesting depth; `ctx.time`
// rotates the stack.

struct Pentagram;
impl ProgressStyle for Pentagram {
    fn name(&self) -> &str {
        "pentagram"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Nested pentagram-in-pentagon: each star's inner pentagon contains a smaller \
         pentagram scaled by 1/φ², revealing infinite φ self-similarity as progress deepens"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let base_r = fit_scale(dw, dh) * 0.95;

        let max_depth: usize = 6;
        let depth = ((ctx.eased * max_depth as f32).ceil() as usize)
            .min(max_depth)
            .max(1);
        let rot0 = ctx.time * 0.2 - PI / 2.0; // point upward, rotate with time

        // Draw from outermost to innermost.
        let inner_scale = 1.0 / (PHI * PHI); // each level shrinks by 1/φ²

        let mut r = base_r;
        let mut rot = rot0;

        for _ in 0..depth {
            if r < 1.0 {
                break;
            }
            // Draw pentagram: connect vertices 0-2-4-1-3-0.
            let verts = ngon_vertices(5, cx, cy, r, rot);
            let star_order = [0usize, 2, 4, 1, 3, 0];
            for k in 0..5 {
                let (x0, y0) = verts[star_order[k]];
                let (x1, y1) = verts[star_order[k + 1]];
                bresenham(grid, x0, y0, x1, y1);
            }
            // Draw the enclosing pentagon.
            ngon_outline(grid, 5, cx, cy, r, rot);

            r *= inner_scale;
            rot += PI / 5.0; // each inner pentagon is rotated by 36°
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pentagon Nest — concentric pentagons each scaled by 1/φ², rotating
// ─────────────────────────────────────────────────────────────────────────────
//
// Purely the concentric pentagons without the star chords.  Each ring slowly
// counter-rotates against `ctx.time`, and the nest grows as `ctx.eased` rises.

struct PentagonNest;
impl ProgressStyle for PentagonNest {
    fn name(&self) -> &str {
        "pentagon-nest"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Concentric pentagons each scaled 1/φ² smaller, counter-rotating with time — \
         a hypnotic tunnel of five-fold symmetry"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let base_r = fit_scale(dw, dh) * 0.95;

        let max_rings: usize = 8;
        let rings = ((ctx.eased * max_rings as f32).ceil() as usize)
            .min(max_rings)
            .max(1);
        let inner_scale = 1.0 / (PHI * PHI);

        let mut r = base_r;
        for i in 0..rings {
            if r < 1.0 {
                break;
            }
            // Alternate rotation direction per ring.
            let sign = if i % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
            let rot = ctx.time * 0.15 * sign - PI / 2.0 + i as f32 * 0.1;
            ngon_outline(grid, 5, cx, cy, r, rot);
            r *= inner_scale;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Golden Gnomon — 36-72-72 golden triangle recursive subdivision
// ─────────────────────────────────────────────────────────────────────────────
//
// The golden gnomon is the 36-72-72 isoceles triangle where the ratio of the
// long side to the short side equals φ. Subdividing the base triangle with a
// bisector from the apex yields a smaller golden gnomon + a golden triangle
// (36-36-108). Revealed with `ctx.eased` depth.

struct GoldenGnomon;
impl ProgressStyle for GoldenGnomon {
    fn name(&self) -> &str {
        "golden-gnomon"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "36-72-72 golden gnomon recursively bisected into smaller golden triangles — \
         the geometric basis of Penrose tilings, depth revealed with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // A frieze of gnomons: full-height 36-72-72 triangles shoulder to
        // shoulder across the whole width, with inverted copies between their
        // apexes (each is the reflection of its neighbour, so the strip tiles
        // exactly). Every triangle is recursively bisected to `eased` depth.
        let sin36 = 36.0_f32.to_radians().sin();
        let cos36 = 36.0_f32.to_radians().cos();
        let hf = (dh as f32 - 1.0).max(1.0);
        let side = hf / cos36; // long side spans the full canvas height
        let bw = 2.0 * sin36 * side; // gnomon base width

        // Breathing: the frieze pulses about the canvas centre (0.25 Hz —
        // seamless over the 4 s loop).
        let pulse = 1.0 + 0.035 * (ctx.time * PI * 0.5).sin();
        let (mx, my) = (dw as f32 / 2.0, dh as f32 / 2.0);

        let max_depth: usize = 4;
        let depth = ((ctx.eased * max_depth as f32).ceil() as usize).clamp(1, max_depth);

        type Pt = (f32, f32);
        fn lerp_pt(a: Pt, b: Pt, t: f32) -> Pt {
            (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
        }
        let draw_tri = |grid: &mut BrailleGrid, a: Pt, b: Pt, c: Pt| {
            let to_dot = |p: Pt| {
                (
                    (mx + (p.0 - mx) * pulse).round() as i32,
                    (my + (p.1 - my) * pulse).round() as i32,
                )
            };
            let (ax, ay) = to_dot(a);
            let (bx, by) = to_dot(b);
            let (ccx, ccy) = to_dot(c);
            bresenham(grid, ax, ay, bx, by);
            bresenham(grid, bx, by, ccx, ccy);
            bresenham(grid, ccx, ccy, ax, ay);
        };

        // Root frieze: alternating upright / inverted gnomons.
        // is_gnomon = true: 36-72-72 gnomon; false: golden triangle (36-36-108).
        let n_up = ((dw as f32 / bw).ceil() as usize + 1).min(16);
        let mut tris: Vec<(Pt, Pt, Pt, bool)> = Vec::new();
        // Leading inverted gnomon so the top-left corner is tiled too.
        tris.push(((0.0, hf), (-bw / 2.0, 0.0), (bw / 2.0, 0.0), true));
        for k in 0..n_up {
            let x0 = k as f32 * bw;
            // Upright: apex on the top edge, base along the bottom.
            tris.push(((x0 + bw / 2.0, 0.0), (x0, hf), (x0 + bw, hf), true));
            // Inverted: apex on the bottom edge between neighbouring apexes.
            tris.push((
                (x0 + bw, hf),
                (x0 + bw / 2.0, 0.0),
                (x0 + 1.5 * bw, 0.0),
                true,
            ));
        }
        for &(a, b, c, _) in &tris {
            draw_tri(grid, a, b, c);
        }

        // Subdivision:
        //   Gnomon (A=apex 36°, B, C base): place P on AB at AP = BC = 1/PHI * AB.
        //     → gnomon(P, A, C) [rotated/reflected] + golden-tri(B, P, C).
        //   Golden-tri (A=apex 108°, B, C base): place P on AB at AP = 1/PHI.
        //     → gnomon(C, P, A) + golden-tri(B, P, C).
        for _d in 0..depth.saturating_sub(1) {
            let mut next: Vec<(Pt, Pt, Pt, bool)> = Vec::new();
            for &(a, b, c, is_gnomon) in &tris {
                // P on AB s.t. AP = 1/PHI * |AB|, for both subdivisions.
                let p = lerp_pt(a, b, 1.0 / PHI);
                if is_gnomon {
                    next.push((p, a, c, true)); // smaller gnomon
                } else {
                    next.push((c, p, a, true)); // gnomon
                }
                next.push((b, p, c, false)); // golden triangle
            }
            // Draw all triangles at this depth.
            for &(a, b, c, _) in &next {
                draw_tri(grid, a, b, c);
            }
            tris = next;
            if tris.len() > 512 {
                break;
            } // cap for degenerate/large grids
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. φ-Phyllotaxis (pentagonal) — five-fold seed arrangement
// ─────────────────────────────────────────────────────────────────────────────
//
// Standard phyllotaxis but with a modular constraint: only seeds whose index
// is congruent to 0, 1, 2, 3, 4 (mod 5) are drawn in five distinct angular
// sectors separated by 72°.  This produces five spiral arms and five-fold
// symmetry, unlike the usual all-seeds sunflower.

struct PhiPhyllotaxisPent;
impl ProgressStyle for PhiPhyllotaxisPent {
    fn name(&self) -> &str {
        "phi-phyllotaxis-pent"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Five-fold phyllotaxis: seeds placed at the golden angle but grouped into five \
         symmetric spiral arms, creating pentagonal lattice symmetry with φ spacing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // Golden angle in radians.
        let golden_angle = 2.0 * PI / (PHI * PHI); // ≈ 137.508°
        let n_max: usize = 300;
        let n_plot = ((ctx.eased * n_max as f32).round() as usize).min(n_max);
        let c = scale / (n_max as f32).sqrt();
        let rot = ctx.time * 0.12;

        // Five sectors: seed n belongs to sector (n % 5).
        // Draw seeds in each sector at their natural positions.
        // Connect adjacent seeds within each arm with short lines for extra
        // visual texture (unlike geometry.rs plain phyllotaxis which just plots dots).
        let mut sector_prev: [Option<(i32, i32)>; 5] = [None; 5];

        for n in 0..n_plot {
            let angle = n as f32 * golden_angle + rot;
            let r = c * (n as f32).sqrt();
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy - r * angle.sin()).round() as i32;
            let sector = n % 5;
            // Connect to previous point in the same sector with a short chord.
            if let Some((lx, ly)) = sector_prev[sector] {
                bresenham(grid, lx, ly, px, py);
            } else {
                draw::dot_i(grid, px, py);
            }
            sector_prev[sector] = Some((px, py));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Nautilus — chambered logarithmic spiral with cross-walls
// ─────────────────────────────────────────────────────────────────────────────
//
// True φ-logarithmic spiral r = a·φ^(θ·2/π) (one φ-factor per quarter turn).
// Cross-walls ("septa") divide the spiral into chambers every π/2 radians,
// drawn as radial line segments from the inner to the outer wall.

struct Nautilus;
impl ProgressStyle for Nautilus {
    fn name(&self) -> &str {
        "nautilus"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "φ-logarithmic nautilus spiral r=a·φ^(θ·2/π) with chambered septa every \
         quarter turn — the classic cephalopod shell built from pure golden-ratio growth"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        // The 0.55 vertical squash below means height limits the shell at
        // dh/2/0.55, not dh/2 — use the larger budget so the shell fills.
        let scale = (dh as f32 / 2.0 / 0.55 - 1.0)
            .min(dw as f32 / 2.0 - 1.0)
            .max(2.0);

        // Spiral: r(θ) = φ^(k·(θ−θ_total)) — three turns, exponent compressed
        // so the innermost whorl is still ~a tenth of the outer radius.
        // (The raw φ^(θ·2/π) over four turns spans a 2000:1 radius range and
        // renders sub-dot for three of the four turns.)
        let theta_total = 6.0 * PI;
        let theta_max = ctx.eased * theta_total;
        let rot = ctx.time * 0.15;
        let k = 0.75 / PI;

        let steps = ((theta_max / (2.0 * PI) * 140.0).round() as usize).max(2);

        for i in 0..=steps {
            let theta = i as f32 / steps as f32 * theta_max;
            let r = PHI.powf(k * (theta - theta_total)) * scale;
            let angle = theta + rot;
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy - r * angle.sin() * 0.55).round() as i32;
            draw::dot_i(grid, px, py);
            draw::dot_i(grid, px + 1, py);
        }

        // Draw septa (cross-walls) every π/2 radians, from inner to outer wall.
        let n_septa = (theta_max / (PI / 2.0)).floor() as usize;
        for s in 0..=n_septa {
            let theta_wall = s as f32 * PI / 2.0;
            if theta_wall > theta_max {
                break;
            }
            // Inner radius: one turn back (θ - 2π), or 0 if s < 4.
            let r_outer = PHI.powf(k * (theta_wall - theta_total)) * scale;
            let theta_inner = theta_wall - 2.0 * PI;
            let r_inner = if theta_inner > 0.0 {
                PHI.powf(k * (theta_inner - theta_total)) * scale
            } else {
                0.0
            };
            let angle = theta_wall + rot;
            let x_outer = (cx + r_outer * angle.cos()).round() as i32;
            let y_outer = (cy - r_outer * angle.sin() * 0.55).round() as i32;
            let x_inner = (cx + r_inner * angle.cos()).round() as i32;
            let y_inner = (cy - r_inner * angle.sin() * 0.55).round() as i32;
            bresenham(grid, x_inner, y_inner, x_outer, y_outer);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Golden-Angle Rays — successive rays at 137.5° increments
// ─────────────────────────────────────────────────────────────────────────────
//
// Rays emanate from the center at successive multiples of the golden angle
// (137.508° ≈ 2π/φ²). Each ray is longer than the last by factor φ^(1/n),
// so the whole pattern never repeats and fills space evenly.

struct GoldenAngleRays;
impl ProgressStyle for GoldenAngleRays {
    fn name(&self) -> &str {
        "golden-angle-rays"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Rays at successive 137.5° golden-angle increments from the center, each \
         growing longer — the irrational spacing that prevents clustering and produces \
         optimal coverage of the disk"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let golden_angle = 2.0 * PI / (PHI * PHI); // ≈ 137.508°
        let n_max: usize = 55; // 55 = F₁₀, natural Fibonacci count
        let n_rays = ((ctx.eased * n_max as f32).round() as usize).min(n_max);
        let rot = ctx.time * 0.1;

        for i in 0..n_rays {
            let angle = i as f32 * golden_angle + rot;
            // Ray length grows with index, bounded by scale.
            let t = (i + 1) as f32 / n_max as f32;
            let len = scale * t; // linear growth for clarity
            let x_end = (cx + len * angle.cos()).round() as i32;
            let y_end = (cy - len * angle.sin()).round() as i32;
            let cx_i = cx.round() as i32;
            let cy_i = cy.round() as i32;
            bresenham(grid, cx_i, cy_i, x_end, y_end);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Dodecagram — 12-pointed φ-star via string-art on a 12-gon
// ─────────────────────────────────────────────────────────────────────────────
//
// Connect vertex i to vertex (i + 5) mod 12 on a regular 12-gon.  This
// produces a {12/5} dodecagram.  The ratio of chord to side of the 12-gon
// contains φ.  Chords appear one by one with progress.

struct Dodecagram;
impl ProgressStyle for Dodecagram {
    fn name(&self) -> &str {
        "dodecagram"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Dodecagram {12/5}: connecting every 5th vertex of a regular 12-gon produces \
         a twelve-pointed φ-star; chords materialise progressively while the star rotates"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh) * 0.92;

        let n: usize = 12;
        let step: usize = 5; // {12/5} star polygon
        let rot = ctx.time * 0.14;
        let n_chords = ((ctx.eased * n as f32).round() as usize).min(n);

        // Draw the outer 12-gon outline always.
        ngon_outline(grid, n, cx, cy, scale, rot);

        // Draw the star chords.
        let verts = ngon_vertices(n, cx, cy, scale, rot);
        for i in 0..n_chords {
            let j = (i + step) % n;
            let (x0, y0) = verts[i];
            let (x1, y1) = verts[j];
            bresenham(grid, x0, y0, x1, y1);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Kepler Triangle — right triangle with sides 1 : √φ : φ
// ─────────────────────────────────────────────────────────────────────────────
//
// The Kepler triangle has the unique property that its sides form a geometric
// progression 1, √φ, φ, so the hypotenuse-to-leg ratio embeds φ.
// We render a fan of Kepler triangles tiled around the center, each rotated by
// the golden angle, revealing more triangles as progress rises.

struct KeplerTriangle;
impl ProgressStyle for KeplerTriangle {
    fn name(&self) -> &str {
        "kepler-triangle"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Kepler triangle fan: right triangles with sides 1:√φ:φ tiled around the \
         center, each rotated by the golden angle — where Pythagoras meets φ"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // Kepler triangle sides in proportion: a=1, b=√φ, c=φ.
        // We scale so the hypotenuse (φ) = scale.
        let a = scale / PHI; // short leg
        let b = scale / PHI.sqrt(); // long leg (= √φ * a/1 * scale/φ simplified)
                                    // Verify: a² + b² = (scale/PHI)² + (scale/PHI.sqrt())²
                                    //       = scale²(1/φ² + 1/φ) = scale²((1+φ)/φ²) = scale²(φ²/φ²) = scale² ✓

        let golden_angle = 2.0 * PI / (PHI * PHI);
        let n_max: usize = 34; // F₉
        let n_tris = ((ctx.eased * n_max as f32).round() as usize).min(n_max);
        let rot_base = ctx.time * 0.1;

        for i in 0..n_tris {
            let rot = i as f32 * golden_angle + rot_base;
            // Right-angle vertex at center.
            // Leg a along rot direction, leg b perpendicular (rot + π/2).
            let tip_a = (cx + a * rot.cos(), cy - a * rot.sin());
            let tip_b = (
                cx + b * (rot + PI / 2.0).cos(),
                cy - b * (rot + PI / 2.0).sin(),
            );
            let origin = (cx.round() as i32, cy.round() as i32);
            let (ax, ay) = (tip_a.0.round() as i32, tip_a.1.round() as i32);
            let (bx, by) = (tip_b.0.round() as i32, tip_b.1.round() as i32);
            // Draw the three sides of the Kepler triangle.
            bresenham(grid, origin.0, origin.1, ax, ay); // short leg
            bresenham(grid, origin.0, origin.1, bx, by); // long leg
            bresenham(grid, ax, ay, bx, by); // hypotenuse
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
    let styles = progress::styles::goldenratio::styles();
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
