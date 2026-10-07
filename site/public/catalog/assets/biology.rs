//! `biology` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O biology.rs && ./biology [style-name]
//! ```

const DEFAULT_STYLE: &str = "dna-helix";

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
    pub mod biology {
//! Biology-themed progress bars for dotmax.
//!
//! Twelve animated braille bars each rooted in a distinct biological concept:
//! DNA replication, cell division, neural firing, logistic growth, cardiac
//! rhythm, protein folding, photosynthesis, phylogeny, hemodynamics, enzyme
//! kinetics, viral spread, and ion-channel pumping. Every bar is structurally
//! different — color alone is never the only distinguishing feature.
//!
//! ## Styles
//!
//! | Name                  | Concept                                              |
//! |-----------------------|------------------------------------------------------|
//! | `dna-helix`           | Double helix with rotating strands and base-pair rungs |
//! | `mitosis`             | Chromosomes align, split to poles; cell pinches in two |
//! | `action-potential`    | PQRST-like spike travelling along an axon            |
//! | `logistic-growth`     | S-curve fill with carrying-capacity ceiling          |
//! | `ecg-heartbeat`       | Scrolling ECG waveform; beats counted = progress     |
//! | `protein-folding`     | Bead chain folds from strand into a β-sheet          |
//! | `photosynthesis`      | Rayed sun beams photons; veined leaf fills with colour |
//! | `phylogenetic-tree`   | Bifurcating tree grows outward with eased            |
//! | `blood-flow`          | Biconcave red cells tumble through a pulsing vessel  |
//! | `enzyme-kinetics`     | Wedge substrate docks in active site; products stack |
//! | `virus-spread`        | Nucleated cells turn solid; virions bud at the front |
//! | `ion-channels`        | Lipid bilayer + protein pores; ions pile up inside   |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// All styles in the `biology` theme.
///
/// Returns 12 boxed bars, each embodying a structurally distinct biological
/// process. Safe to render from a 1×1 cell grid up to 80×8 or larger; every
/// function uses saturating arithmetic and `draw::dot_i` for bounds safety.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(DnaHelix),
        Box::new(Mitosis),
        Box::new(ActionPotential),
        Box::new(LogisticGrowth),
        Box::new(EcgHeartbeat),
        Box::new(ProteinFolding),
        Box::new(Photosynthesis),
        Box::new(PhylogeneticTree),
        Box::new(BloodFlow),
        Box::new(EnzymeKinetics),
        Box::new(VirusSpread),
        Box::new(IonChannels),
    ]
}

// ---------------------------------------------------------------------------
// Shared helper: Bresenham-style line between two signed dot coordinates
// ---------------------------------------------------------------------------
fn line_i(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let x = x0 + (x1 - x0) * i / steps;
        let y = y0 + (y1 - y0) * i / steps;
        draw::dot_i(grid, x, y);
    }
}

// ---------------------------------------------------------------------------
// 1. DNA Double Helix
//    Two sine strands π out of phase; horizontal base-pair rungs connect them.
//    Rotates via time (simulates 3-D twist); unzipping revealed by eased.
// ---------------------------------------------------------------------------
struct DnaHelix;
impl ProgressStyle for DnaHelix {
    fn name(&self) -> &str {
        "dna-helix"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "DNA double helix: two counter-rotating sine strands with base-pair rungs; unzips with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = (h / 2) as f32;
        let amp = (h as f32 * 0.42).max(1.0);
        // One full helix period spans the bar; time advances phase for 3-D rotation feel.
        let phase_offset = ctx.time * 1.4;
        // How far the helix is "unzipped" — eased controls it left-to-right.
        let unzip_x = (ctx.eased * w as f32) as usize;

        let mut prev_a: Option<i32> = None;
        let mut prev_b: Option<i32> = None;

        for xi in 0..w {
            let xf = xi as f32 / w as f32;
            let angle = xf * 4.0 * PI + phase_offset;

            let ya = (mid + amp * angle.sin()) as i32;
            let yb = (mid + amp * (angle + PI).sin()) as i32;

            // Draw strand A (continuous curve)
            draw::dot_i(grid, xi as i32, ya);
            if let Some(py) = prev_a {
                let (lo, hi) = (py.min(ya), py.max(ya));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_a = Some(ya);

            // Draw strand B (phase-flipped)
            draw::dot_i(grid, xi as i32, yb);
            if let Some(py) = prev_b {
                let (lo, hi) = (py.min(yb), py.max(yb));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_b = Some(yb);

            // Base-pair rungs every ~8 dots, only in the zipped region
            if xi >= unzip_x && xi % 8 == 0 {
                let rung_lo = ya.min(yb).max(0) as usize;
                let rung_hi = ya.max(yb).min(h as i32 - 1) as usize;
                draw::vline(grid, xi, rung_lo, rung_hi);
            }
        }

        // Tint: filled region gets palette gradient
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
// 2. Mitosis
//    One cell runs through the phases of division as eased advances:
//    prophase (nucleus envelope, condensing X-shaped chromosomes) →
//    metaphase (chromosomes line up on the equator, spindle fibres reach in
//    from both poles) → anaphase (sister chromatids are pulled apart to the
//    poles while the cell elongates) → telophase / cytokinesis (the membrane
//    pinches into a furrow and two daughter cells with fresh nuclei
//    separate). The membrane is drawn as the union of two ellipses whose
//    centres slide apart; time makes it breathe and the chromosomes jiggle.
// ---------------------------------------------------------------------------
struct Mitosis;
impl ProgressStyle for Mitosis {
    fn name(&self) -> &str {
        "mitosis"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Cell mitosis: chromosomes line up, spindles pull them apart, and the cell pinches into two daughter cells"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (fw, fh) = (w as f32, h as f32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let cx = fw / 2.0;
        let cy = (fh - 1.0) / 2.0;
        let r = (fh / 2.0 - 0.5).min(fw / 8.0).max(1.0);
        // Horizontal stretch: the cell is an ellipse, wider than tall.
        let k = 1.5_f32;
        let membrane = ctx.palette.sample(0.0);
        let dna = ctx.palette.sample(1.0);

        // Half-separation of the two daughter centres: a slow elongation
        // through anaphase, then the split.
        let half = if p < 0.3 {
            0.0
        } else if p < 0.65 {
            (p - 0.3) / 0.35 * r * k * 0.45
        } else {
            r * k * 0.45 + (p - 0.65) / 0.35 * (r * k * 0.9 + 3.0)
        };
        let (lc, rc) = (cx - half, cx + half);

        // --- Membrane: outline of the union of two circles ---
        let ri = (r + 2.0) as i32;
        let rxi = (r * k + 2.0) as i32;
        for &c in &[lc, rc] {
            for dy in -ri..=ri {
                for dx in -rxi..=rxi {
                    let (x, y) = (c.round() as i32 + dx, cy.round() as i32 + dy);
                    let (fx, fy) = (x as f32, y as f32);
                    let ang = (fy - cy).atan2((fx - c) / k);
                    let rr = r + 0.35 * (ang * 3.0 + ctx.time * 2.0).sin();
                    let d = (((fx - c) / k).powi(2) + (fy - cy).powi(2)).sqrt();
                    let other = if c < cx { rc } else { lc };
                    let d_other = (((fx - other) / k).powi(2) + (fy - cy).powi(2)).sqrt();
                    if (d - rr).abs() < 0.6 && (half <= 0.0 || d_other > rr - 0.3) {
                        draw::dot_i(grid, x, y);
                        if x >= 0 && y >= 0 {
                            let c2 = (x / 2) as usize;
                            draw::tint_row(grid, y as usize / 4, c2, c2, membrane);
                        }
                    }
                }
            }
        }

        // --- Chromosomes / chromatids ---
        let n = 4;
        let pitch = (fh / 5.0).max(1.0);
        let mark = |grid: &mut BrailleGrid, x: f32, y: f32, shape: &[(i32, i32)]| {
            let (xi, yi) = (x.round() as i32, y.round() as i32);
            for &(dx, dy) in shape {
                draw::dot_i(grid, xi + dx, yi + dy);
            }
            if xi >= 0 && yi >= 0 {
                let c2 = (xi / 2) as usize;
                draw::tint_row(grid, yi as usize / 4, c2, c2, dna);
            }
        };
        let x_shape = [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)];
        let to_left = [(1, -1), (0, 0), (1, 1)];
        let to_right = [(-1, -1), (0, 0), (-1, 1)];
        if p < 0.65 {
            // Poles for the spindle fibres (metaphase onward).
            let pole_l = lc - r * k * 0.85;
            let pole_r = rc + r * k * 0.85;
            for i in 0..n {
                let jig = (ctx.time * 3.0 + i as f32 * 1.7).sin() * 0.6;
                let ey = cy + (i as f32 - (n as f32 - 1.0) / 2.0) * pitch;
                // Prophase: scattered; metaphase: aligned on the equator.
                let settle = (p / 0.3).min(1.0);
                let sx = cx + ((i as f32 * 2.3).sin() * r * k * 0.4) * (1.0 - settle) + jig;
                let sy = ey + ((i as f32 * 1.3).cos() * 1.5) * (1.0 - settle);
                if p < 0.4 {
                    mark(grid, sx, sy, &x_shape);
                } else {
                    // Anaphase: sister chromatids pulled to opposite poles.
                    let pull = (p - 0.4) / 0.25 * (r * k * 0.75);
                    mark(grid, sx - 1.0 - pull, sy, &to_left);
                    mark(grid, sx + 1.0 + pull, sy, &to_right);
                }
                if p >= 0.2 {
                    let off = if p < 0.4 {
                        2.0
                    } else {
                        (p - 0.4) / 0.25 * r * k * 0.75 + 2.0
                    };
                    line_i(
                        grid,
                        pole_l.round() as i32,
                        cy.round() as i32,
                        (sx - off).round() as i32,
                        sy.round() as i32,
                    );
                    line_i(
                        grid,
                        pole_r.round() as i32,
                        cy.round() as i32,
                        (sx + off).round() as i32,
                        sy.round() as i32,
                    );
                }
            }
            // Nuclear envelope during prophase.
            if p < 0.15 {
                let nr = r * 0.6;
                for seg in 0..24 {
                    if seg % 2 == 0 {
                        let a = seg as f32 / 24.0 * PI * 2.0;
                        draw::dot_i(
                            grid,
                            (cx + a.cos() * nr * k).round() as i32,
                            (cy + a.sin() * nr).round() as i32,
                        );
                    }
                }
            }
        } else {
            // Telophase: each daughter re-forms a solid nucleus.
            let nr = r * 0.4;
            for &c in &[lc, rc] {
                let ni = nr.ceil() as i32;
                for dy in -ni..=ni {
                    for dx in -ni..=ni {
                        if (dx * dx + dy * dy) as f32 <= nr * nr + 0.5 {
                            mark(grid, c + dx as f32, cy + dy as f32, &[(0, 0)]);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Neuron Action Potential
//    An axon runs as a horizontal baseline. A travelling spike (−70→+40→hyperpolarise)
//    scrolls left via time; spike count accumulated = eased.
// ---------------------------------------------------------------------------
struct ActionPotential;
impl ProgressStyle for ActionPotential {
    fn name(&self) -> &str {
        "action-potential"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Neuron action potential: a travelling voltage spike scrolls along the axon; spikes fired = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let baseline_y = (h as f32 * 0.72) as usize; // resting potential sits low
        let spike_amp = (h as f32 * 0.70).max(1.0);

        // Axon baseline
        draw::hline(grid, 0, w.saturating_sub(1), baseline_y);

        // Soma bump on the left
        let soma_r = (h as f32 * 0.25).max(1.0) as i32;
        for dy in -soma_r..=soma_r {
            let dist = (dy.abs() as f32 / soma_r as f32).powi(2);
            let dx = ((1.0 - dist).max(0.0).sqrt() * soma_r as f32) as i32;
            for ddx in -dx..=dx {
                draw::dot_i(grid, ddx, baseline_y as i32 + dy);
            }
        }

        // Wave shape: resting(−70)=0, depol(+40)=+1, repol(0)=0, hyperpol(−90)=−0.2, return=0
        // Parameterised over local phase [0, 1)
        let spike_shape = |phase: f32| -> f32 {
            if phase < 0.15 {
                // Rising depolarisation
                (phase / 0.15) * 1.0
            } else if phase < 0.30 {
                // Falling repolarisation
                1.0 - ((phase - 0.15) / 0.15) * 1.2
            } else if phase < 0.45 {
                // Hyperpolarisation undershoot
                -0.2 * (1.0 - ((phase - 0.30) / 0.15))
            } else {
                0.0
            }
        };

        // Spike repetition period in seconds: fewer spikes at low progress
        let spike_period = 1.2_f32 - ctx.eased * 0.7; // 1.2s → 0.5s at full
        let spike_period = spike_period.max(0.4);

        // How wide a single spike is in dots
        let spike_w = (w as f32 * 0.22).max(6.0);

        // Render spikes scrolling left; phase offset driven by time
        let scroll = (ctx.time / spike_period).fract(); // 0..1 scroll within period
                                                        // Draw up to 5 spike instances tiled across the bar
        for pass in 0..5i32 {
            // Position of this spike's leading edge (in dots from right, scrolling left)
            let spike_center = w as f32 - (scroll + pass as f32) * (w as f32 * 0.5);

            for xi in 0..w {
                let xf = xi as f32;
                let dist = xf - spike_center;
                if dist.abs() < spike_w {
                    let local_phase = (dist / spike_w * 0.5 + 0.5).clamp(0.0, 1.0);
                    let v = spike_shape(local_phase);
                    let dy = (v * spike_amp) as i32;
                    let sy = baseline_y as i32 - dy;
                    draw::dot_i(grid, xi as i32, sy.clamp(0, h as i32 - 1));
                }
            }
        }

        // Filled region (spikes fired) tinted with palette
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw.max(1) as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Logistic Population Growth
//    N(t) = K / (1 + exp(−r·(t − t_mid)))  S-curve fills with eased.
//    A horizontal carrying-capacity ceiling is drawn at the top.
// ---------------------------------------------------------------------------
struct LogisticGrowth;
impl ProgressStyle for LogisticGrowth {
    fn name(&self) -> &str {
        "logistic-growth"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Logistic population growth: S-curve fills the bar with carrying-capacity ceiling"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Carrying-capacity line (dashed): 2 dots from top
        let k_y = 2usize;
        for x in (0..w).step_by(4) {
            draw::hline(grid, x, (x + 2).min(w - 1), k_y);
        }

        // Logistic curve: x axis = progress (0..w), y axis = population
        // r controls steepness; midpoint at x=half
        let r = 8.0_f32; // steepness
                         // eased shifts the midpoint of the visible S-curve along x
        let midpoint = (1.0 - ctx.eased) * w as f32; // at eased=1 midpoint exits left

        let mut prev_pop_y: Option<i32> = None;
        for xi in 0..w {
            let xf = xi as f32;
            let t = (xf - midpoint) * r / w as f32;
            let n = 1.0 / (1.0 + (-t).exp()); // 0..1
                                              // Map to dot rows: n=1 → k_y+1, n=0 → h−1
            let pop_y = (k_y + 1 + ((1.0 - n) * (h - k_y - 2) as f32) as usize).min(h - 1);
            let py = pop_y as i32;

            draw::dot_i(grid, xi as i32, py);
            if let Some(prev) = prev_pop_y {
                let (lo, hi) = (prev.min(py), prev.max(py));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_pop_y = Some(py);

            // Shade column under the curve
            for y in (pop_y + 1)..h {
                // Dotted fill — every other row only to keep it readable
                if y % 2 == xi % 2 {
                    draw::dot(grid, xi, y);
                }
            }
        }

        // Population label ticks at 25%, 50%, 75%
        for &frac in &[0.25_f32, 0.5, 0.75] {
            let ty = (k_y + 1 + ((1.0 - frac) * (h - k_y - 2) as f32) as usize).min(h - 1);
            draw::hline(grid, 0, 3.min(w - 1), ty);
        }

        // Tint: gradient across the filled portion
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw.max(1) as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. ECG / Heartbeat
//    PQRST waveform scrolls leftward driven by time.
//    Number of beats since start = floor(eased × max_beats) — shown as
//    filled-fraction of the baseline.
// ---------------------------------------------------------------------------
struct EcgHeartbeat;
impl ProgressStyle for EcgHeartbeat {
    fn name(&self) -> &str {
        "ecg-heartbeat"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Scrolling ECG / PQRST heartbeat waveform; beats counted = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let baseline_y = (h as f32 * 0.65) as usize;
        draw::hline(grid, 0, w.saturating_sub(1), baseline_y);

        // PQRST shape: parameterised over [0,1) within one beat period
        // Returns vertical offset relative to baseline (positive = up)
        let pqrst = |phase: f32| -> f32 {
            // P wave: small bump 0.05..0.15
            let p = if (0.05..0.20).contains(&phase) {
                ((phase - 0.05) / 0.075 * PI).sin() * 0.15
            } else {
                0.0
            };
            // Q: slight dip 0.25..0.30
            let q = if (0.25..0.30).contains(&phase) {
                -((phase - 0.25) / 0.05 * PI).sin() * 0.12
            } else {
                0.0
            };
            // R: tall spike 0.30..0.40
            let r = if (0.30..0.40).contains(&phase) {
                ((phase - 0.30) / 0.05 * PI).sin() * 1.0
            } else {
                0.0
            };
            // S: dip immediately after 0.40..0.46
            let s = if (0.40..0.46).contains(&phase) {
                -((phase - 0.40) / 0.03 * PI).sin() * 0.25
            } else {
                0.0
            };
            // T: broad bump 0.50..0.70
            let t = if (0.50..0.70).contains(&phase) {
                ((phase - 0.50) / 0.10 * PI).sin() * 0.30
            } else {
                0.0
            };
            p + q + r + s + t
        };

        // Heart rate: ~60 BPM at low progress, faster (up to 120 BPM) at full
        let bpm = 60.0 + ctx.eased * 60.0;
        let period = 60.0 / bpm; // seconds per beat
                                 // Time scroll position
        let scroll_phase = (ctx.time / period) % 1.0;

        let amp = (h as f32 * 0.55).max(1.0);
        let mut prev_y: Option<i32> = None;

        for xi in 0..w {
            // Each dot column maps to a phase that scrolls left
            let phase = ((xi as f32 / w as f32) + scroll_phase) % 1.0;
            let v = pqrst(phase);
            let dy = baseline_y as i32 - (v * amp) as i32;
            let dy = dy.clamp(0, h as i32 - 1);
            draw::dot_i(grid, xi as i32, dy);
            if let Some(py) = prev_y {
                let (lo, hi) = (py.min(dy), py.max(dy));
                for yy in lo..=hi {
                    draw::dot_i(grid, xi as i32, yy);
                }
            }
            prev_y = Some(dy);
        }

        // Tint: filled fraction of the bar = beats counted
        let (cw, ch) = grid.dimensions();
        let filled = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled.min(cw) {
            let t = cx as f32 / cw.max(1) as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Protein Folding
//    A bead-on-a-string amino-acid chain: residues are solid beads joined by
//    backbone bonds. At eased=0 the chain lies extended across the bar and
//    jiggles thermally; as eased→1 every residue slides to its slot in a
//    compact snake-folded β-sheet (rows joined by hairpin turns), the jiggle
//    calms, and dotted hydrogen bonds snap in between neighbouring strands.
//    Hydrophobic residues (every third) take the end colour.
// ---------------------------------------------------------------------------
struct ProteinFolding;
impl ProgressStyle for ProteinFolding {
    fn name(&self) -> &str {
        "protein-folding"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Protein folding: a bead-on-string residue chain folds from an extended strand into a compact hydrogen-bonded β-sheet"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;
        let e = ctx.eased;

        // Folded sheet layout: `rows` strands joined by hairpins.
        let rows = (h / 5).clamp(1, 3);
        let per_row = 5usize;
        let n = rows * per_row;
        let pitch = 6.0_f32;
        let row_gap = (hf / rows as f32).max(1.0);
        let block_w = pitch * (per_row - 1) as f32;
        let fx0 = wf / 2.0 - block_w / 2.0;
        let mid_y = (hf - 1.0) / 2.0;

        // Thermal jiggle fades as the chain settles.
        let jig = (1.0 - e) * 1.5;

        let mut pts: Vec<(i32, i32)> = Vec::with_capacity(n);
        for i in 0..n {
            let frac = i as f32 / (n - 1).max(1) as f32;
            // Extended strand across the bar with a gentle wave.
            let ex = 2.0 + frac * (wf - 5.0).max(0.0);
            let ey = mid_y + (i as f32 * 0.9 + ctx.time * 2.0).sin() * jig * 1.5;
            // Folded slot: boustrophedon rows.
            let r = i / per_row;
            let c = i % per_row;
            let col = if r % 2 == 0 { c } else { per_row - 1 - c };
            let fx = fx0 + col as f32 * pitch;
            let fy = row_gap * (r as f32 + 0.5);
            let jx = (ctx.time * 7.0 + i as f32 * 2.3).sin() * jig;
            let x = ex + (fx - ex) * e + jx;
            let y = ey + (fy - ey) * e;
            pts.push((x.round() as i32, y.round() as i32));
        }

        // Backbone bonds, then beads on top.
        for pair in pts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            line_i(grid, a.0, a.1, b.0, b.1);
        }
        for &(x, y) in &pts {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    draw::dot_i(grid, x + dx, y + dy);
                }
            }
        }

        // Hydrogen bonds between neighbouring strands once nearly folded.
        if e > 0.85 && rows > 1 {
            for i in 0..n {
                let r = i / per_row;
                if r + 1 >= rows {
                    continue;
                }
                let c = i % per_row;
                // Partner directly below in the next strand.
                let j = (r + 1) * per_row + (per_row - 1 - c);
                if j < n && c % 2 == 0 {
                    let (a, b) = (pts[i], pts[j]);
                    let mut y = a.1.min(b.1) + 2;
                    while y < a.1.max(b.1) - 1 {
                        draw::dot_i(grid, a.0, y);
                        y += 2;
                    }
                }
            }
        }

        // Tint: chain in the start colour, hydrophobic beads in the end colour.
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            draw::tint_row(grid, cy, 0, cw, ctx.palette.start);
        }
        for (i, &(x, y)) in pts.iter().enumerate() {
            if i % 3 == 0 && x >= 0 && y >= 0 {
                draw::tint_row(
                    grid,
                    y as usize / 4,
                    x as usize / 2,
                    x as usize / 2,
                    ctx.palette.end,
                );
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Photosynthesis
//    A rayed sun on the left beams photons (dots travelling along rays) onto
//    a large leaf: stalk, almond blade with a pointed tip, midrib and
//    side veins left as negative space. The blade greens up — filling solid
//    from the stalk toward the tip — with eased progress, while the sun's
//    rays slowly rotate.
// ---------------------------------------------------------------------------
struct Photosynthesis;
impl ProgressStyle for Photosynthesis {
    fn name(&self) -> &str {
        "photosynthesis"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Photosynthesis: a rayed sun beams photons onto a veined leaf that fills with colour from stalk to tip"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (fw, fh) = (w as f32, h as f32);
        let mid = (fh - 1.0) / 2.0;

        // --- Sun: solid disc + rotating rays ---
        let sr = (fh * 0.2).min(fw * 0.05).max(1.0);
        let sx = sr + 3.0;
        let ri = sr.ceil() as i32;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                if (dx * dx + dy * dy) as f32 <= sr * sr + 0.5 {
                    draw::dot_i(grid, sx as i32 + dx, mid.round() as i32 + dy);
                }
            }
        }
        for k in 0..8 {
            let a = k as f32 * PI / 4.0 + ctx.time * 0.6;
            let (c, s) = (a.cos(), a.sin());
            line_i(
                grid,
                (sx + c * (sr + 1.5)).round() as i32,
                (mid + s * (sr + 1.5)).round() as i32,
                (sx + c * (sr + 3.0)).round() as i32,
                (mid + s * (sr + 3.0)).round() as i32,
            );
        }

        // --- Leaf geometry ---
        let stalk0 = sx + sr + 6.0;
        let l0 = stalk0 + (fw * 0.06).max(2.0);
        let l1 = fw - 3.0;
        let span = (l1 - l0).max(1.0);
        let big_h = (fh - 2.0) / 2.0;
        let half = |x: f32| {
            let u = ((x - l0) / span).clamp(0.0, 1.0);
            big_h * (PI * u.powf(0.6)).sin()
        };

        // Photons travel from the sun toward the leaf's upper and lower faces.
        for i in 0..4 {
            let ph = (ctx.time * 0.8 + i as f32 * 0.25).fract();
            let tx = l0 + span * (0.2 + 0.15 * i as f32);
            let ty = if i % 2 == 0 { 0.0 } else { fh - 1.0 };
            let px = sx + (tx - sx) * ph;
            let py = mid + (ty - mid) * ph * 0.9;
            draw::dot_i(grid, px.round() as i32, py.round() as i32);
        }

        // Stalk.
        draw::hline(grid, stalk0 as usize, l0 as usize, mid.round() as usize);

        // Blade outline + progressive solid fill with negative-space veins.
        let fill_x = l0 + span * ctx.eased;
        let mut prev: Option<(i32, i32)> = None;
        let xs = l0.round() as i32;
        let xe = l1.round() as i32;
        for x in xs..=xe {
            let hw = half(x as f32);
            let top = (mid - hw).round() as i32;
            let bot = (mid + hw).round() as i32;
            match prev {
                Some((pt, pb)) => {
                    line_i(grid, x - 1, pt, x, top);
                    line_i(grid, x - 1, pb, x, bot);
                }
                None => line_i(grid, x, top, x, bot),
            }
            prev = Some((top, bot));
            if (x as f32) <= fill_x {
                let my = mid.round() as i32;
                for y in (top + 1)..bot {
                    let off = (y - my).abs();
                    let vein = (x - xs - off * 2).rem_euclid(9) == 0;
                    if y != my && !vein {
                        draw::dot_i(grid, x, y);
                    }
                }
                if x >= 0 {
                    let u = (x as f32 - l0) / span;
                    let c = (x / 2) as usize;
                    let (_, ch) = grid.dimensions();
                    for cy in 0..ch {
                        draw::tint_row(grid, cy, c, c, ctx.palette.sample(u));
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Phylogenetic Tree
//    Branches bifurcate outward from a root on the left; depth / branch count
//    driven by eased. Branches splay up and down. Time animates tip flutter.
// ---------------------------------------------------------------------------
struct PhylogeneticTree;
impl ProgressStyle for PhylogeneticTree {
    fn name(&self) -> &str {
        "phylogenetic-tree"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Phylogenetic tree: branches bifurcate outward; depth unlocks with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Max depth controlled by eased: 0→1 = 1..=5 levels
        let max_depth = 1 + (ctx.eased * 4.0).floor() as usize; // 1..=5

        // Recursively draw branches using a stack (avoid actual recursion)
        // Each entry: (x, y, angle_rad, length, depth)
        let root_x = 2i32;
        let root_y = (h / 2) as i32;
        let root_len = (w as f32 * 0.24).max(4.0);

        struct Branch {
            x: i32,
            y: i32,
            angle: f32,
            len: f32,
            depth: usize,
        }
        let mut stack: Vec<Branch> = vec![Branch {
            x: root_x,
            y: root_y,
            angle: 0.0,
            len: root_len,
            depth: 0,
        }];

        // Gentle tip flutter via time
        let flutter_amp = 0.05_f32;
        let flutter = (ctx.time * 2.5).sin() * flutter_amp;

        while let Some(b) = stack.pop() {
            if b.depth > max_depth {
                continue;
            }
            // Squash the vertical component so deep branches stay on the
            // short canvas instead of shooting off the top and bottom.
            let end_x = b.x + (b.angle.cos() * b.len) as i32;
            let end_y = b.y + (b.angle.sin() * b.len * 0.35) as i32;
            line_i(grid, b.x, b.y, end_x, end_y);

            if b.depth < max_depth {
                let new_len = b.len * 0.62;
                if new_len < 1.5 {
                    continue;
                }
                let spread = PI * 0.38 + flutter * (b.depth as f32 + 1.0);
                stack.push(Branch {
                    x: end_x,
                    y: end_y,
                    angle: b.angle - spread,
                    len: new_len,
                    depth: b.depth + 1,
                });
                stack.push(Branch {
                    x: end_x,
                    y: end_y,
                    angle: b.angle + spread,
                    len: new_len,
                    depth: b.depth + 1,
                });
            }
        }

        // Root dot
        draw::dot_i(grid, root_x, root_y);

        // Tint gradient left→right
        let (cw, ch) = grid.dimensions();
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            if t <= ctx.eased {
                let col = ctx.palette.sample(t);
                for cy in 0..ch {
                    draw::tint_row(grid, cy, cx, cx, col);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Blood Flow
//    A vessel (two parallel lines) runs horizontally. Red blood cell dots
//    stream along it at pulsatile speed (time-modulated). Filled fraction of
//    the vessel = eased (vessel "perfused").
// ---------------------------------------------------------------------------
struct BloodFlow;
impl ProgressStyle for BloodFlow {
    fn name(&self) -> &str {
        "blood-flow"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Blood flow: biconcave red cells tumble through a walled vessel in pulsing surges; perfused fraction = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = h / 2;
        let vessel_r = (h as f32 * 0.45).max(1.0) as usize;

        // Vessel walls: double lines (endothelium + outer wall)
        let top_wall = mid.saturating_sub(vessel_r);
        let bot_wall = (mid + vessel_r).min(h - 1);
        draw::hline(grid, 0, w.saturating_sub(1), top_wall);
        draw::hline(grid, 0, w.saturating_sub(1), bot_wall);
        if h >= 16 {
            draw::hline(grid, 0, w.saturating_sub(1), top_wall + 1);
            draw::hline(grid, 0, w.saturating_sub(1), bot_wall - 1);
        }

        // Perfused segment fills from left with eased
        let perfused_w = (ctx.eased * w as f32) as usize;

        // Pulsatile flow: steady drift plus a systolic surge each beat
        // (displacement stays monotonic, so cells never run backwards).
        let cardiac_freq = 1.1_f32; // Hz
        let travel = ctx.time * 10.0 + 1.2 * (ctx.time * cardiac_freq * 2.0 * PI).sin();

        // Red blood cells: biconcave discs (a ring with a pale centre) in two
        // lanes, tumbling between face-on and narrower three-quarter views.
        let face: [&[u8]; 4] = [b".####.", b"##..##", b"##..##", b".####."];
        let turned: [&[u8]; 4] = [b".##.", b"#..#", b"#..#", b".##."];
        let lanes = [
            mid as i32 - (vessel_r as i32) / 2 - 1,
            mid as i32 + (vessel_r as i32) / 2 - 2,
        ];
        let n_cells = (w / 9).max(1);
        for ci in 0..n_cells {
            let lane = ci % 2;
            let base_x = (ci as f32 / n_cells as f32) * w as f32 + lane as f32 * 3.0;
            let scroll = (travel + base_x).rem_euclid(w as f32);
            if (scroll as usize) >= perfused_w {
                continue;
            }
            let tumble = (ctx.time * 1.3 + ci as f32 * 0.9).sin() > 0.2;
            let shape: &[&[u8]; 4] = if tumble { &turned } else { &face };
            let (x0, y0) = (scroll as i32 - 3, lanes[lane]);
            for (dy, row) in shape.iter().enumerate() {
                for (dx, &c) in row.iter().enumerate() {
                    if c == b'#' {
                        draw::dot_i(grid, x0 + dx as i32, y0 + dy as i32);
                    }
                }
            }
        }

        // Tint perfused section
        let (cw, ch) = grid.dimensions();
        let perfused_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..perfused_cells.min(cw) {
            let t = cx as f32 / cw.max(1) as f32;
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Enzyme Kinetics (Michaelis-Menten)
//     Lock and key: a large solid enzyme on the left with a wedge-shaped
//     active site opening to the right. Each catalytic cycle (driven by time)
//     a matching wedge-shaped substrate slides in along the axis, docks
//     (enzyme-substrate complex), then splits into two product halves that
//     drop away. Finished products stack in a tally along the floor; the
//     tally length = eased.
// ---------------------------------------------------------------------------
struct EnzymeKinetics;
impl ProgressStyle for EnzymeKinetics {
    fn name(&self) -> &str {
        "enzyme-kinetics"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Enzyme kinetics: a wedge substrate docks in the enzyme's active site and splits into products that stack up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (fw, fh) = (w as f32, h as f32);
        let ey = (fh - 1.0) / 2.0;
        let ry = (fh / 2.0 - 0.5).max(1.0);
        let rx = (ry * 1.6).min(fw * 0.2);
        let ex = rx + 1.0;
        let enzyme = ctx.palette.sample(0.0);
        let substrate = ctx.palette.sample(1.0);
        // Active-site wedge: apex inside the enzyme, opening to the right.
        let apex = ex + rx * 0.15;
        let slope = 0.7_f32;
        let in_site = |x: f32, y: f32| x > apex && (y - ey).abs() < (x - apex) * slope;

        // --- Enzyme body ---
        let (rxi, ryi) = (rx.ceil() as i32, ry.ceil() as i32);
        for dy in -ryi..=ryi {
            for dx in -rxi..=rxi {
                let (x, y) = (ex + dx as f32, ey.round() + dy as f32);
                let (u, v) = ((x - ex) / rx, (y - ey) / ry);
                if u * u + v * v <= 1.0 && !in_site(x, y) {
                    draw::dot_i(grid, x as i32, y as i32);
                    if x >= 0.0 && y >= 0.0 {
                        let c = x as usize / 2;
                        draw::tint_row(grid, y as usize / 4, c, c, enzyme);
                    }
                }
            }
        }

        // --- One catalytic cycle: approach → dock → split ---
        let ph = (ctx.time * 0.7).fract();
        let len = (rx * 0.85).max(2.0);
        let docked_x = apex + 1.0;
        let wedge = |grid: &mut BrailleGrid, tip_x: f32, tip_y: f32, half: i32| {
            // half: 0 = whole wedge, -1 = upper half, 1 = lower half.
            for i in 0..=(len as i32) {
                let x = tip_x + i as f32;
                let spread = (i as f32 * slope - 0.8).max(0.0);
                let (lo, hi) = match half {
                    -1 => (-spread, 0.0),
                    1 => (1.0, spread),
                    _ => (-spread, spread),
                };
                let y_lo = (tip_y + lo).round() as i32;
                let y_hi = (tip_y + hi).floor() as i32;
                for y in y_lo..=y_hi {
                    draw::dot_i(grid, x.round() as i32, y);
                    if x >= 0.0 && y >= 0 {
                        let c = x as usize / 2;
                        draw::tint_row(grid, y as usize / 4, c, c, substrate);
                    }
                }
            }
        };
        let far = fw * 0.62;
        if ph < 0.45 {
            let t = ph / 0.45;
            let x = far + (docked_x - far) * (1.0 - (1.0 - t) * (1.0 - t));
            wedge(grid, x, ey, 0);
        } else if ph < 0.65 {
            wedge(grid, docked_x, ey, 0);
        } else {
            let t = (ph - 0.65) / 0.35;
            let x = docked_x + t * rx * 1.2;
            wedge(grid, x, ey - t * ry * 0.6, -1);
            wedge(grid, x, ey + t * ry * 0.6, 1);
        }

        // --- Product tally along the floor ---
        let x0 = (ex + rx + 4.0) as usize;
        let slots = w.saturating_sub(x0 + 1) / 4;
        let done = (ctx.eased * slots as f32).round() as usize;
        let y = h - 2;
        for i in 0..done {
            let x = x0 + i * 4;
            draw::fill_rect(grid, x, y, 2, 2);
            draw::tint_row(grid, y / 4, x / 2, x / 2, substrate);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Virus Spread
//    A tissue of round-cornered cells, each with a nucleus. Infection starts
//    at the left edge and sweeps right with a ragged front: infected cells
//    turn solid and coloured, and at the frontier spiky virions bud off and
//    drift into the next healthy cell. Infected fraction = eased.
// ---------------------------------------------------------------------------
struct VirusSpread;
impl ProgressStyle for VirusSpread {
    fn name(&self) -> &str {
        "virus-spread"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Viral spread: a tissue of nucleated cells turns solid as infection sweeps right; spiky virions bud into healthy neighbours"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let px = 8usize;
        let py = if h >= 8 { 8usize } else { h };
        let cols = w / px;
        let rows = (h / py).max(1);
        let ox = (w - cols * px) / 2;
        let reach = ctx.eased * (cols as f32 + 1.5);
        let infected = |c: usize, r: usize| {
            let k = (c * 7 + r * 13) as u32;
            let jitter = (k.wrapping_mul(2654435761) >> 16) % 100;
            c as f32 + jitter as f32 / 100.0 * 1.5 < reach
        };
        let sick = ctx.palette.sample(1.0);
        let (s, e) = (px - 1, py - 1);

        for r in 0..rows {
            for c in 0..cols {
                let (x, y) = (ox + c * px, r * py);
                // Membrane with cut corners.
                draw::hline(grid, x + 1, x + s - 2, y);
                draw::hline(grid, x + 1, x + s - 2, y + e - 1);
                draw::vline(grid, x, y + 1, y + e.saturating_sub(2));
                draw::vline(grid, x + s - 1, y + 1, y + e.saturating_sub(2));
                if infected(c, r) {
                    draw::fill_rect(grid, x + 1, y + 1, s - 2, e.saturating_sub(2));
                    for cy in (y / 4)..=((y + e - 1) / 4) {
                        draw::tint_row(grid, cy, x / 2, (x + s - 1) / 2, sick);
                    }
                    // Virions bud toward a healthy right-hand neighbour.
                    if c + 1 < cols && !infected(c + 1, r) {
                        let ph = (ctx.time * 0.9 + (c + r) as f32 * 0.31).fract();
                        let vx = (x + s) as f32 + ph * px as f32 * 0.8;
                        let vy = y as f32 + e as f32 * 0.5 + (ph * PI * 2.0).sin();
                        let (vx, vy) = (vx.round() as i32, vy.round() as i32 - 1);
                        draw::dot_i(grid, vx, vy);
                        for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1), (0, -2), (0, 2)] {
                            draw::dot_i(grid, vx + dx, vy + dy);
                        }
                    }
                } else {
                    // Nucleus.
                    draw::fill_rect(grid, x + s / 2 - 1, y + e / 2 - 1, 2, 2);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Ion Channels (Cell Membrane)
//    A phospholipid bilayer spans the bar: two leaflets of round heads with
//    paired tails pointing inward. Transmembrane channel proteins (solid
//    columns flanking an open pore) punctuate it, and ions (+ signs) stream
//    through each pore from outside to inside, driven by time. The gradient
//    builds with eased progress: ions pile up along the inside face from the
//    left while the outside reservoir drains.
// ---------------------------------------------------------------------------
struct IonChannels;
impl ProgressStyle for IonChannels {
    fn name(&self) -> &str {
        "ion-channels"
    }
    fn theme(&self) -> &str {
        "biology"
    }
    fn describe(&self) -> &str {
        "Ion channels: a phospholipid bilayer with protein pores; + ions stream through and pile up inside as progress builds"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let fh = h as f32;
        let row = |f: f32| (f * fh / 16.0).round() as usize;
        let (head_top, head_bot) = (row(4.0), row(10.0));
        let (ion_top, ion_bot) = (row(1.0), row(14.0));
        let lipid = ctx.palette.sample(0.0);
        let protein = ctx.palette.sample(1.0);

        // Channel pore centres.
        let n = (w / 30).clamp(1, 5);
        let pores: Vec<usize> = (0..n).map(|i| w * (2 * i + 1) / (2 * n)).collect();
        let near_pore = |x: usize| pores.iter().any(|&p| x + 6 > p && x < p + 6);

        // --- Bilayer: heads (2×2) with tails pointing into the core ---
        let mut x = 1;
        while x + 1 < w {
            if !near_pore(x) && !near_pore(x + 1) {
                draw::fill_rect(grid, x, head_top, 2, 2);
                draw::fill_rect(grid, x, head_bot, 2, 2);
                draw::vline(grid, x, head_top + 2, head_top + 3);
                draw::vline(grid, x + 1, head_bot.saturating_sub(2), head_bot - 1);
                draw::tint_row(grid, head_top / 4, x / 2, x / 2, lipid);
                draw::tint_row(grid, head_bot / 4, x / 2, x / 2, lipid);
            }
            x += 4;
        }

        // --- Channel proteins and ions passing through ---
        let (p0, p1) = (row(2.0), row(13.0));
        for (i, &p) in pores.iter().enumerate() {
            for side in [p.saturating_sub(4), p + 2] {
                draw::fill_rect(grid, side, p0 + 1, 3, p1.saturating_sub(p0 + 1));
                draw::hline(grid, side + 1, side + 1, p0);
                draw::hline(grid, side + 1, side + 1, p1);
                for cy in (p0 / 4)..=(p1 / 4) {
                    draw::tint_row(grid, cy, side / 2, (side + 2) / 2, protein);
                }
            }
            let ph = (ctx.time * 0.7 + i as f32 * 0.37).fract();
            let iy = ion_top as f32 + ph * (ion_bot - ion_top) as f32;
            plus(grid, p as i32, iy.round() as i32);
        }

        // --- Gradient: inside fills from the left, outside drains ---
        let split = (ctx.eased * w as f32) as usize;
        let mut x = 2;
        while x + 1 < w {
            if !near_pore(x) {
                if x < split {
                    plus(grid, x as i32, ion_bot as i32);
                } else {
                    plus(grid, x as i32, ion_top as i32);
                }
            }
            x += 6;
        }
        Ok(())
    }
}

/// Small `+` ion marker centred at `(x, y)`.
fn plus(grid: &mut BrailleGrid, x: i32, y: i32) {
    for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        draw::dot_i(grid, x + dx, y + dy);
    }
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::biology::styles();
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
