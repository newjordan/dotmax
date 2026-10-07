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

    pub fn clear(&mut self) {
        self.patterns.fill(0);
        self.colors.fill(None);
    }

    pub fn clear_region(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), DotmaxError> {
        let (end_x, end_y) = (x.saturating_add(width), y.saturating_add(height));
        if end_x > self.width || end_y > self.height {
            return Err(DotmaxError::OutOfBounds {
                x: end_x.saturating_sub(1),
                y: end_y.saturating_sub(1),
                width: self.width,
                height: self.height,
            });
        }
        for row in y..end_y {
            for col in x..end_x {
                self.patterns[row * self.width + col] = 0;
                self.colors[row * self.width + col] = None;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn get_raw_patterns(&self) -> &[u8] {
        &self.patterns
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
//! Wildlife-themed progress bars — eleven recognisable creatures rendered
//! in braille dots, each coloured per cell.
//!
//! Each style draws a distinct animal silhouette:
//! - `galloping-horse`    — chestnut horse in profile with a three-pose gallop
//! - `butterfly`          — monarch with flapping upper/lower wings on a flight path
//! - `spider-web`         — eight-legged spider spinning spokes, then rings
//! - `beehive`            — honeycomb filling with amber honey, bees buzzing
//! - `peacock-fan`        — crested peacock spreading an eye-spotted train
//! - `leaping-frog`       — bug-eyed frog croaking, then leaping pad to pad
//! - `octopus`            — domed octopus with six waving tentacles
//! - `owl`                — tufted, ring-eyed owl sidling along a branch, blinking
//! - `murmuration`        — many tiny dots flowing in a starling-like wave
//! - `elephant`           — grey elephant with ear, tusk, swinging trunk, plodding legs
//! - `bat-swoop`          — big-eared bat beating its wings past a full moon

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `wildlife` theme.
///
/// Returns one boxed [`ProgressStyle`] per creature variant. Every style is
/// stateless — all animation comes from `ctx.time` and `ctx.eased`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(GallopingHorse),
        Box::new(Butterfly),
        Box::new(SpiderWeb),
        Box::new(Beehive),
        Box::new(PeacockFan),
        Box::new(LeapingFrog),
        Box::new(Octopus),
        Box::new(Owl),
        Box::new(Murmuration),
        Box::new(Elephant),
        Box::new(BatSwoop),
    ]
}

// ── Sprite helpers ───────────────────────────────────────────────────────────

/// Stamp an ASCII sprite (`#` = dot) with its top-left corner at `(x, y)`.
fn stamp(grid: &mut BrailleGrid, rows: &[&str], x: i32, y: i32) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            if c == b'#' {
                draw::dot_i(grid, x + dx as i32, y + dy as i32);
            }
        }
    }
}

/// Colour every non-empty cell overlapping the dot rectangle `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    grid.enable_color_support();
    let (cw, ch) = grid.dimensions();
    let (cx1, cy1) = ((x1 / 2).min(cw as i32 - 1), (y1 / 4).min(ch as i32 - 1));
    for cy in (y0.max(0) / 4)..=cy1 {
        for cx in (x0.max(0) / 2)..=cx1 {
            let c = grid.get_char(cx as usize, cy as usize);
            if c != '\u{2800}' && c != ' ' {
                let _ = grid.set_cell_color(cx as usize, cy as usize, color);
            }
        }
    }
}

/// Inside test for an axis-aligned ellipse.
fn in_ellipse(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> bool {
    let (dx, dy) = ((x - cx) / rx.max(0.01), (y - cy) / ry.max(0.01));
    dx * dx + dy * dy <= 1.0
}

// ── Galloping Horse ──────────────────────────────────────────────────────────

/// Horse body facing right: ears, head and muzzle, arched neck, back, tail.
const HORSE: [&str; 9] = [
    "              # #     ",
    "             #####    ",
    "            ## #####  ",
    "           ########## ",
    "  #       ####   ###  ",
    " # ############       ",
    "#  #############      ",
    "   #############      ",
    "    ###########       ",
];

/// Gallop leg poses: stretched, gathered, passing.
const HORSE_LEGS: [[&str; 4]; 3] = [
    [
        "   ##       ##     ",
        "  ##         ##    ",
        " ##           ##   ",
        "##             ##  ",
    ],
    [
        "    ##     ##      ",
        "     ##   ##       ",
        "    ##   ##        ",
        "   ##     ##       ",
    ],
    [
        "    ##      ##     ",
        "    ##      ##     ",
        "   ##      ##      ",
        "   ##      ##      ",
    ],
];

struct GallopingHorse;
impl ProgressStyle for GallopingHorse {
    fn name(&self) -> &str {
        "galloping-horse"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Chestnut horse in side profile galloping right — ears, muzzle, flying tail, legs cycling stretch-gather and kicking up dust"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let hx = (ctx.eased * (wi - 22).max(0) as f32) as i32;
        // Gallop: three poses at 9 fps; the body lifts when legs gather.
        let pose = ((ctx.time * 9.0) as i32).rem_euclid(3) as usize;
        let lift = i32::from(pose == 1);
        let top = ground - 12 - lift;
        stamp(grid, &HORSE, hx, top);
        stamp(grid, &HORSE_LEGS[pose], hx, top + 9);
        paint(grid, hx, top, hx + 21, ground, Color::rgb(200, 125, 65));
        // Scrolling turf dashes and a dust puff behind the hooves.
        let scroll = (ctx.time * 24.0) as i32;
        for x in (0..wi).filter(|x| (x + scroll).rem_euclid(6) < 2) {
            draw::dot_i(grid, x, ground);
        }
        for k in 1..=3i32 {
            draw::dot_i(grid, hx - 2 * k, ground - 1 - (k + pose as i32) % 3);
        }
        paint(grid, hx, ground, hx + 21, ground, Color::rgb(200, 125, 65));
        paint(grid, hx + 22, ground, wi, ground, Color::rgb(110, 160, 70));
        paint(
            grid,
            0,
            ground - 3,
            hx - 1,
            ground,
            Color::rgb(160, 140, 110),
        );
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
        "Monarch butterfly with big upper and lower wings flapping open and shut as it flutters right over a dotted flight path"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bx = 10 + (ctx.eased * (wi - 20).max(0) as f32) as i32;
        let flutter = (ctx.time * PI).sin() * 1.5;
        let by = (hi as f32 / 2.0 + flutter).round() as i32;
        // Wing spread: two beats per second, never fully closed.
        let f = 0.3 + 0.7 * (ctx.time * 2.0 * PI).cos().abs();
        let (bxf, byf) = (bx as f32, by as f32);
        for y in (by - 7)..=(by + 6) {
            for x in (bx - 10)..=(bx + 10) {
                let (xf, yf) = (x as f32, y as f32);
                let side = if x < bx { -1.0 } else { 1.0 };
                let upper = in_ellipse(xf, yf, bxf + side * 4.5 * f, byf - 3.0, 4.5 * f + 0.6, 3.6);
                let lower = in_ellipse(xf, yf, bxf + side * 3.0 * f, byf + 3.0, 3.0 * f + 0.6, 2.6);
                if (upper || lower) && x != bx {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        paint(
            grid,
            bx - 11,
            by - 8,
            bx + 11,
            by - 1,
            Color::rgb(255, 140, 40),
        );
        paint(grid, bx - 11, by, bx + 11, by + 7, Color::rgb(255, 200, 70));
        // Body and curled antennae.
        draw::vline(
            grid,
            bx.max(0) as usize,
            (by - 4).max(0) as usize,
            (by + 5).max(0) as usize,
        );
        draw::dot_i(grid, bx - 1, by - 5);
        draw::dot_i(grid, bx - 2, by - 6);
        draw::dot_i(grid, bx + 1, by - 5);
        draw::dot_i(grid, bx + 2, by - 6);
        paint(grid, bx, by - 6, bx, by + 5, Color::rgb(90, 60, 50));
        // Looping dotted flight path behind it.
        for x in (0..(bx - 12)).step_by(3) {
            let y = (hi as f32 / 2.0 + ((x as f32 * 0.25) - ctx.time * PI).sin() * 3.0) as i32;
            draw::dot_i(grid, x, y);
        }
        paint(grid, 0, 0, bx - 12, hi, Color::rgb(200, 170, 230));
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
        "Eight-legged spider spinning a silver orb web — it lays the spokes first, then walks the rings round as progress fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as i32;
        let cy = (h / 2) as i32;
        let max_r = (w as f32 / 3.6).min(h as f32).max(1.0);

        // Number of radial spokes.
        let n_spokes = 12usize;
        // Spokes appear one by one in the first 40% of progress.
        let spoke_progress = (ctx.eased * n_spokes as f32 / 0.4).min(n_spokes as f32);
        let full_spokes = spoke_progress as usize;

        // Draw radial spokes.
        for s in 0..full_spokes.min(n_spokes) {
            let angle = s as f32 * (2.0 * PI / n_spokes as f32);
            let r_max = max_r as i32;
            for r in 0..=r_max {
                let dx = (angle.cos() * r as f32 * 1.8).round() as i32;
                let dy = (angle.sin() * r as f32 * 0.5).round() as i32;
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
                let dx = (angle.cos() * r as f32 * 1.8).round() as i32;
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
                let dx = (angle.cos() * r as f32 * 1.8).round() as i32;
                let dy = (angle.sin() * r as f32 * 0.5).round() as i32;
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        paint(grid, 0, 0, w as i32, h as i32, Color::rgb(190, 200, 225));
        // Spider sits where the thread is being spun — the tip of the newest
        // spoke, then the leading end of the ring in progress — legs twitching.
        let (ang, rad) = if full_spokes < n_spokes {
            let s = full_spokes.saturating_sub(1) as f32;
            (s * (2.0 * PI / n_spokes as f32), max_r)
        } else {
            let r = max_r * (full_rings.min(n_rings - 1) + 1) as f32 / n_rings as f32;
            (partial_ring_frac * 2.0 * PI, r)
        };
        let sx = cx + (ang.cos() * rad * 1.8).round() as i32 - 4;
        let sy = (cy + (ang.sin() * rad * 0.5).round() as i32 - 3).clamp(0, (h as i32 - 7).max(0));
        let twitch = ((ctx.time * 6.0) as i32).rem_euclid(2) as usize;
        stamp(grid, &SPIDER[twitch], sx, sy);
        paint(grid, sx, sy, sx + 8, sy + 6, Color::rgb(240, 110, 60));
        Ok(())
    }
}

/// Spider seen from above: two body segments and eight bent legs, two poses.
const SPIDER: [[&str; 7]; 2] = [
    [
        "#       #",
        " #  #  # ",
        "## ### ##",
        "  #####  ",
        "## ### ##",
        " #  #  # ",
        "#       #",
    ],
    [
        " #     # ",
        "#   #   #",
        " # ### # ",
        "# ##### #",
        " # ### # ",
        "#   #   #",
        " #     # ",
    ],
];

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
        "Hexagonal honeycomb filling with amber honey cell by cell while two striped bees buzz loops around the front"
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
            let col = idx / rows;
            let row = idx % rows;
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

        // Honey-filled cells glow amber; the rest stay pale wax.
        let (cells_w, cells_h) = grid.dimensions();
        let filled_cx = (ctx.eased * cells_w as f32) as usize;
        for cy_c in 0..cells_h {
            for cx_c in 0..cells_w {
                let c = if cx_c < filled_cx {
                    let g = 105 + ((cx_c * 7 + cy_c * 13) % 5) as u8 * 10;
                    Color::rgb(235, g, 20)
                } else {
                    Color::rgb(205, 190, 150)
                };
                draw::tint_row(grid, cy_c, cx_c, cx_c, c);
            }
        }
        // Two striped bees buzz in loops around the filling front, each in a
        // cleared pocket so its silhouette stays readable over the comb.
        let front = (ctx.eased * w as f32) as i32;
        for b in 0..2i32 {
            let ph = ctx.time * PI + b as f32 * PI;
            let bx = front - 5 + (ph.sin() * 7.0) as i32 + b * 4;
            let by = (h as i32 / 2 - 3) + ((ph * 2.0).sin() * 4.0) as i32;
            let (cx0, cy0) = ((bx.max(0) / 2) as usize, (by.max(0) / 4) as usize);
            let cx1 = (((bx + 11) / 2).max(0) as usize).min(cells_w.saturating_sub(1));
            let cy1 = (((by + 5) / 4).max(0) as usize).min(cells_h.saturating_sub(1));
            if cx0 <= cx1 && cy0 <= cy1 {
                let _ = grid.clear_region(cx0, cy0, cx1 - cx0 + 1, cy1 - cy0 + 1);
            }
            let flap = ((ctx.time * 10.0) as i32 + b).rem_euclid(2) as usize;
            stamp(grid, &BEE[flap], bx, by);
            paint(grid, bx, by, bx + 10, by + 5, Color::rgb(255, 245, 120));
        }
        Ok(())
    }
}

/// Bee facing right — wings up/down, striped abdomen, stinger, head.
const BEE: [[&str; 6]; 2] = [
    [
        "    ## ##  ",
        "    ## ##  ",
        "  # # # ## ",
        "## # # ####",
        "  # # # ## ",
        "           ",
    ],
    [
        "           ",
        "    ## ##  ",
        "  # # # ## ",
        "## # # ####",
        "  # # # ## ",
        "    ## ##  ",
    ],
];

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
        "Blue peacock with a crested head raising its train — eye-spotted feathers fan wider and fuller as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        // Pivot at the bird's rump, bottom-centre.
        let (px, py) = (wi / 2, hi - 1);
        let flen = (hi - 3).max(1) as f32;
        // Progress spreads the train: more feathers, wider fan.
        let half_fan = 0.15 + ctx.eased * PI * 0.37;
        let n_feathers = 13usize;
        let shown = ((ctx.eased * n_feathers as f32).ceil() as usize).clamp(3, n_feathers);
        for f in 0..shown {
            let t = if shown <= 1 {
                0.5
            } else {
                f as f32 / (shown - 1) as f32
            };
            // Feathers shimmer: each sways on its own phase.
            let sway = (ctx.time * PI + f as f32).sin() * 0.03;
            let angle = -PI / 2.0 + (t - 0.5) * 2.0 * half_fan + sway;
            let (ca, sa) = (angle.cos() * 2.6, angle.sin());
            for r in 3..(flen as i32 - 1) {
                draw::dot_i(
                    grid,
                    px + (ca * r as f32).round() as i32,
                    py + (sa * r as f32).round() as i32,
                );
            }
            // Eye spot: a ring with a centre dot at the feather tip.
            let (ex, ey) = (
                px + (ca * flen).round() as i32,
                py + (sa * flen).round() as i32,
            );
            for (dx, dy) in [
                (-2, 0),
                (2, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (0, 0),
            ] {
                draw::dot_i(grid, ex + dx, ey + dy);
            }
        }
        paint(grid, 0, 0, wi, hi, Color::rgb(60, 180, 110));
        for f in 0..shown {
            let t = if shown <= 1 {
                0.5
            } else {
                f as f32 / (shown - 1) as f32
            };
            let sway = (ctx.time * PI + f as f32).sin() * 0.03;
            let angle = -PI / 2.0 + (t - 0.5) * 2.0 * half_fan + sway;
            let (ex, ey) = (
                px + (angle.cos() * 2.6 * flen).round() as i32,
                py + (angle.sin() * flen).round() as i32,
            );
            paint(grid, ex - 1, ey - 1, ex + 1, ey + 1, ctx.palette.sample(t));
        }
        // Body: royal-blue breast, slim neck, head with beak and crest.
        for y in (py - 6)..=py {
            for x in (px - 3)..=(px + 3) {
                if in_ellipse(x as f32, y as f32, px as f32, (py - 3) as f32, 2.6, 3.6) {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        draw::vline(
            grid,
            (px + 1).max(0) as usize,
            (py - 10).max(0) as usize,
            (py - 6).max(0) as usize,
        );
        for (dx, dy) in [
            (1, -11),
            (2, -11),
            (2, -10),
            (3, -10),
            (1, -13),
            (2, -14),
            (3, -13),
        ] {
            draw::dot_i(grid, px + dx, py + dy);
        }
        paint(grid, px - 3, py - 14, px + 3, py, Color::rgb(50, 110, 240));
        Ok(())
    }
}

// ── Leaping Frog ─────────────────────────────────────────────────────────────

/// Frog sitting on a pad: bulging eyes on top, folded back legs.
const FROG_SIT: [&str; 7] = [
    "  ##   ##  ",
    " #### #### ",
    " ######### ",
    "###########",
    " ######### ",
    "##  ###  ##",
    "###     ###",
];

/// Frog mid-leap: body stretched forward, back legs trailing straight.
const FROG_LEAP: [&str; 6] = [
    "           ## ",
    "        ######",
    "    ######### ",
    "########  ##  ",
    "##        #   ",
    "#             ",
];

struct LeapingFrog;
impl ProgressStyle for LeapingFrog {
    fn name(&self) -> &str {
        "leaping-frog"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Bug-eyed frog croaking on a lily pad, then leaping pad to pad across the pond with legs outstretched"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let water = hi - 1;
        // Rippling water line.
        for x in 0..wi {
            if ((x as f32 * 0.6 + ctx.time * 2.0 * PI).sin()) > 0.2 {
                draw::dot_i(grid, x, water);
            }
        }
        paint(grid, 0, water, wi, water, Color::rgb(70, 130, 220));
        // Lily pads, evenly spaced.
        let n = 4i32;
        let span = (wi - 14).max(1) as f32;
        let pad_x = |k: i32| 2 + (k as f32 / (n - 1) as f32 * span) as i32;
        for k in 0..n {
            let px = pad_x(k);
            draw::hline(
                grid,
                px.max(0) as usize,
                (px + 10).max(0) as usize,
                (water - 1).max(0) as usize,
            );
            draw::hline(
                grid,
                (px + 1).max(0) as usize,
                (px + 4).max(0) as usize,
                (water - 2).max(0) as usize,
            );
            draw::hline(
                grid,
                (px + 6).max(0) as usize,
                (px + 9).max(0) as usize,
                (water - 2).max(0) as usize,
            );
        }
        paint(grid, 0, water - 2, wi, water - 1, Color::rgb(60, 160, 70));
        // Each segment: sit for 60% (croaking), leap for the last 40%.
        let seg = ctx.eased * (n - 1) as f32;
        let k = (seg as i32).min(n - 2).max(0);
        let frac = (seg - k as f32).clamp(0.0, 1.0);
        let frog = Color::rgb(160, 225, 80);
        if frac < 0.6 || ctx.eased >= 1.0 {
            let at = if ctx.eased >= 1.0 { n - 1 } else { k };
            let (fx, fy) = (pad_x(at), water - 9);
            stamp(grid, &FROG_SIT, fx, fy);
            paint(grid, fx, fy, fx + 10, fy + 6, frog);
            // Throat sac swells on the croak beat.
            if (ctx.time * 2.0).fract() < 0.5 {
                draw::hline(
                    grid,
                    (fx + 8).max(0) as usize,
                    (fx + 11).max(0) as usize,
                    (fy + 4).max(0) as usize,
                );
                draw::dot_i(grid, fx + 11, fy + 3);
                paint(
                    grid,
                    fx + 11,
                    fy + 3,
                    fx + 11,
                    fy + 4,
                    Color::rgb(250, 230, 150),
                );
            }
        } else {
            let t = (frac - 0.6) / 0.4;
            let x = pad_x(k) as f32 + t * (pad_x(k + 1) - pad_x(k)) as f32;
            let arc = 4.0 * (hi - 8).max(0) as f32 * t * (1.0 - t);
            let (fx, fy) = (x as i32, water - 8 - arc as i32);
            stamp(grid, &FROG_LEAP, fx, fy);
            paint(grid, fx, fy, fx + 13, fy + 5, frog);
        }
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
        "Domed octopus with two eyes and six curling tentacles waving below as it jets right, bubbles trailing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ox = 8 + (ctx.eased * (wi - 18).max(0) as f32) as i32;
        let bob = ((ctx.time * PI).sin() * 0.8).round() as i32;
        let (oxf, top) = (ox as f32, bob);
        // Mantle: tall dome with two open eyes.
        for y in top..top + 8 {
            for x in (ox - 6)..=(ox + 6) {
                let eye = (y == top + 5) && (x == ox - 2 || x == ox + 2);
                if !eye && in_ellipse(x as f32, y as f32, oxf, (top + 4) as f32, 5.5, 4.3) {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Six tentacles, each a wave that spreads out and curls at the tip.
        for i in 0..6i32 {
            let root = ox - 5 + i * 2;
            let spread = (i as f32 - 2.5) * 0.35;
            for d in 0..(hi - top - 8).max(0) {
                let sway = ((d as f32 * 0.7) - ctx.time * 2.0 * PI + i as f32).sin() * 1.3;
                let x = root as f32 + spread * d as f32 + sway;
                draw::dot_i(grid, x.round() as i32, top + 8 + d);
            }
        }
        paint(grid, ox - 10, 0, ox + 10, hi, Color::rgb(230, 100, 170));
        // Bubbles rising in its wake.
        for k in 0..4i32 {
            let bx = ox - 10 - k * 6;
            let by = hi - 1 - ((ctx.time * 5.0 + k as f32 * 4.0) as i32).rem_euclid(hi.max(1));
            if bx >= 0 {
                draw::dot_i(grid, bx, by);
                paint(grid, bx, by, bx, by, Color::rgb(150, 210, 255));
            }
        }
        Ok(())
    }
}

// ── Owl ──────────────────────────────────────────────────────────────────────

/// Perched owl: ear tufts, ringed eyes with pupils, beak, speckled breast.
const OWL: [&str; 15] = [
    "##         ##",
    "###       ###",
    "#############",
    "##   ###   ##",
    "#     #     #",
    "#  #  #  #  #",
    "#     #     #",
    "##   ###   ##",
    "##### # #####",
    "#############",
    "## ####### ##",
    "## # # # # ##",
    " ## ##### ## ",
    "  #########  ",
    "   ##   ##   ",
];

/// Closed-eye rows that replace rows 3..=7 during a blink.
const OWL_BLINK: [&str; 5] = [
    "#############",
    "#############",
    "#  ###  ### #",
    "#############",
    "##   ###   ##",
];

struct Owl;
impl ProgressStyle for Owl {
    fn name(&self) -> &str {
        "owl"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Tufted owl with big ringed amber eyes and a beak, sidling along a branch and blinking"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let branch = hi - 1;
        // Branch with a few leaves.
        draw::hline(grid, 0, w - 1, branch as usize);
        for k in 0..=(wi / 14) {
            let lx = k * 14 + 6;
            draw::dot_i(grid, lx, branch - 1);
            draw::dot_i(grid, lx + 1, branch - 1);
            draw::dot_i(grid, lx + 2, branch - 2);
        }
        paint(grid, 0, branch - 2, wi, branch, Color::rgb(130, 95, 60));
        let ox = (ctx.eased * (wi - 13).max(0) as f32) as i32;
        let oy = branch - 15;
        // Blink for a fifth of a second every two seconds.
        if (ctx.time * 0.5).fract() > 0.9 {
            stamp(grid, &OWL[..3], ox, oy);
            stamp(grid, &OWL_BLINK, ox, oy + 3);
            stamp(grid, &OWL[8..], ox, oy + 8);
        } else {
            stamp(grid, &OWL, ox, oy);
        }
        paint(grid, ox, oy, ox + 12, branch - 1, Color::rgb(175, 120, 70));
        paint(grid, ox, oy + 4, ox + 12, oy + 7, Color::rgb(245, 190, 70));
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
        "Starling murmuration — a dense, shape-shifting cloud of birds that swells, twists and pinches as it sweeps right"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wf, hf) = (w as f32, h as f32);
        // The flock's centre travels with progress; its outline morphs with
        // time (half-hertz terms, so the 4 s loop is seamless).
        let cx = 14.0 + ctx.eased * (wf - 28.0).max(0.0);
        let cy = hf / 2.0;
        let ph = ctx.time * PI;
        for i in 0..160u32 {
            // Stable pseudo-random seat inside a unit disc for each bird.
            let a = (i.wrapping_mul(2_654_435_761) >> 8) as f32 / 16_777_216.0 * 2.0 * PI;
            let r = ((i.wrapping_mul(40_503) % 1000) as f32 / 1000.0).sqrt();
            let (u, v) = (a.cos() * r, a.sin() * r);
            // Stretch, twist and pinch the cloud.
            let rx = 13.0 * (1.0 + 0.35 * (ph + v * 2.0).sin());
            let ry = (hf / 2.0 - 1.0) * (0.75 + 0.25 * (ph * 0.5 + u * 3.0).cos());
            let twist = (ph * 0.5 + v * 2.5).sin() * 5.0;
            let x = cx + u * rx + twist;
            let y = cy + v * ry;
            draw::dot_i(grid, x.round() as i32, y.round() as i32);
        }
        // Colour: dusky starlings against the evening, denser toward the core.
        grid.enable_color_support();
        let (cw, ch) = grid.dimensions();
        for y in 0..ch {
            for x in 0..cw {
                let d = ((x as f32 * 2.0 - cx).abs() / 18.0).min(1.0);
                let _ = grid.set_cell_color(x, y, ctx.palette.sample(d));
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
        "Grey elephant plodding right — domed back, flapping ear, ivory tusk, swinging trunk and four pillar legs"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let ex = (ctx.eased * (wi - 26).max(0) as f32) as i32 + 2;
        let top = ground - 15;
        let (exf, topf) = (ex as f32, top as f32);
        // Body and head as overlapping ellipses; the ear is a carved outline
        // that flaps on the time clock; one blank dot is the eye.
        let flap = (ctx.time * PI).sin() * 0.8;
        for y in top..=top + 11 {
            for x in ex..ex + 22 {
                let (xf, yf) = (x as f32, y as f32);
                let body = in_ellipse(xf, yf, exf + 8.0, topf + 6.5, 9.0, 5.5);
                let head = in_ellipse(xf, yf, exf + 17.0, topf + 4.5, 4.5, 4.5);
                let ear_out = in_ellipse(xf, yf, exf + 14.0 - flap, topf + 5.5, 3.2, 4.2);
                let ear_in = in_ellipse(xf, yf, exf + 14.0 - flap, topf + 5.5, 2.2, 3.2);
                let ear_edge = ear_out && !ear_in;
                let eye = x == ex + 19 && y == top + 3;
                if (body || head) && !ear_edge && !eye {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Trunk hangs from the brow and swings, curling forward at the tip.
        let swing = (ctx.time * PI).sin() * 1.5;
        for d in 0..9i32 {
            let x = (exf + 21.0 + swing * d as f32 / 8.0).round() as i32;
            draw::dot_i(grid, x, top + 6 + d);
            draw::dot_i(grid, x - 1, top + 6 + d);
        }
        draw::dot_i(grid, (exf + 22.0 + swing).round() as i32, top + 13);
        // Plodding legs: diagonal pairs lift alternately.
        let step = ((ctx.time * 3.0) as i32).rem_euclid(2);
        for (i, lx) in [1i32, 5, 11, 15].iter().enumerate() {
            let lift = i32::from((i as i32 + step) % 2 == 0);
            for y in (top + 11)..=(ground - lift) {
                draw::hline(
                    grid,
                    (ex + lx).max(0) as usize,
                    (ex + lx + 2).max(0) as usize,
                    y.max(0) as usize,
                );
            }
        }
        // Tail.
        draw::dot_i(grid, ex - 1, top + 5);
        draw::dot_i(grid, ex - 2, top + 6);
        draw::dot_i(grid, ex - 2, top + 7);
        paint(
            grid,
            ex - 2,
            top,
            ex + 24,
            ground,
            Color::rgb(170, 172, 190),
        );
        // Tusk.
        draw::hline(
            grid,
            (ex + 18).max(0) as usize,
            (ex + 19).max(0) as usize,
            (top + 9).max(0) as usize,
        );
        draw::dot_i(grid, ex + 20, top + 8);
        paint(
            grid,
            ex + 18,
            top + 8,
            ex + 20,
            top + 9,
            Color::rgb(250, 245, 225),
        );
        Ok(())
    }
}

// ── Bat Swoop ────────────────────────────────────────────────────────────────

/// Bat wing-beat frames: pointed ears, scalloped wings raised then lowered.
const BAT: [[&str; 6]; 2] = [
    [
        "#     # #     #",
        "##    ###    ##",
        "###  #####  ###",
        " ############# ",
        "  ### ### ###  ",
        "   #   #   #   ",
    ],
    [
        "      # #      ",
        "      ###      ",
        "  ## ##### ##  ",
        " ############# ",
        "##  # ### #  ##",
        "#      #      #",
    ],
];

struct BatSwoop;
impl ProgressStyle for BatSwoop {
    fn name(&self) -> &str {
        "bat-swoop"
    }
    fn theme(&self) -> &str {
        "wildlife"
    }
    fn describe(&self) -> &str {
        "Big-eared bat beating its scalloped wings as it swoops right past a full moon under twinkling stars"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        // Twinkling stars and a full moon high on the right.
        for k in 0..8i32 {
            let (sx, sy) = ((k * 37 + 5) % wi.max(1), (k * 5 + 1) % hi.max(1));
            if ((ctx.time * 2.0) as i32 + k).rem_euclid(3) != 0 {
                draw::dot_i(grid, sx, sy);
            }
        }
        paint(grid, 0, 0, wi, hi, Color::rgb(200, 200, 150));
        let (mx, my) = (wi * 3 / 4, (hi / 2).min(6));
        for y in (my - 6)..=(my + 6) {
            for x in (mx - 6)..=(mx + 6) {
                if in_ellipse(x as f32, y as f32, mx as f32, my as f32, 5.5, 5.5) {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        paint(
            grid,
            mx - 6,
            my - 6,
            mx + 6,
            my + 6,
            Color::rgb(250, 235, 170),
        );
        // The bat swoops along a dipping path, clearing a pocket so it stays
        // a crisp silhouette even in front of the moon.
        let bx = (ctx.eased * (wi - 15).max(0) as f32) as i32;
        let swoop = ((ctx.time * PI).sin() * (hi as f32 / 2.0 - 4.0).max(0.0)) as i32;
        let by = (hi / 2 - 3 + swoop).clamp(0, (hi - 6).max(0));
        let (cw, ch) = grid.dimensions();
        let (cx0, cy0) = ((bx.max(0) / 2) as usize, (by.max(0) / 4) as usize);
        let cx1 = (((bx + 14) / 2).max(0) as usize).min(cw.saturating_sub(1));
        let cy1 = (((by + 5) / 4).max(0) as usize).min(ch.saturating_sub(1));
        if cx0 <= cx1 && cy0 <= cy1 {
            let _ = grid.clear_region(cx0, cy0, cx1 - cx0 + 1, cy1 - cy0 + 1);
        }
        let beat = ((ctx.time * 6.0) as i32).rem_euclid(2) as usize;
        stamp(grid, &BAT[beat], bx, by);
        paint(grid, bx, by, bx + 14, by + 5, Color::rgb(170, 120, 230));
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
