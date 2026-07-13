//! `topology` — dotmax progress styles as a standalone, dependency-free program.
//!
//! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).
//! The style code below is the crate's verbatim source; only the small
//! grid runtime at the top replaces the dotmax crate.
//!
//! Build and run (no cargo needed):
//!
//! ```sh
//! rustc -O topology.rs && ./topology [style-name]
//! ```

const DEFAULT_STYLE: &str = "mobius-strip";

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
    pub mod topology {
//! Topology / parametric-surface / 3D-projection progress bars.
//!
//! Each bar maps `ctx.eased` to the extent of a surface reveal or curve trace,
//! and `ctx.time` to a continuous 3D rotation. A shared helper rotates a point
//! by Euler angles (ax, ay) and orthographically projects it onto the dot
//! lattice, centred on the grid and scaled to fit both axes.
//!
//! All surfaces are rendered in **dot space** via `draw::dot_i`, so out-of-
//! bounds plots are silently discarded. Per-frame point counts are bounded to
//! keep even wide grids snappy.
//!
//! # Styles
//!
//! | Name | Surface |
//! |---|---|
//! | `mobius-strip` | Möbius strip, u revealed by eased |
//! | `torus-wireframe` | (R,r)-torus latitude/longitude wire mesh |
//! | `torus-knot` | (2,3) torus knot trace |
//! | `trefoil-knot` | Trefoil knot, reveals and spins |
//! | `klein-bottle` | Figure-8 Klein bottle immersion |
//! | `hopf-fibers` | Three Hopf fibration circles |
//! | `sphere-inflate` | Wireframe sphere inflating with eased |
//! | `helix-climb` | Double helix climbing with eased |
//! | `saddle-surface` | Hyperbolic paraboloid (saddle) wireframe |
//! | `tesseract-spin` | Rotating 4-D hypercube projected to 2-D |
//! | `roman-surface` | Steiner's Roman surface parametric |
//! | `seifert-ramp` | Spiral Seifert-surface ramp |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── Shared 3-D projection helper ─────────────────────────────────────────────

/// Rotate `(x, y, z)` about the X-axis by `ax` radians then the Y-axis by
/// `ay` radians (standard Euler XY extrinsic), then orthographically project
/// the result onto the dot lattice centred at `(cx, cy)` with uniform `scale`
/// dots-per-unit.
///
/// Returns `(sx, sy)` as `i32` — suitable for `draw::dot_i`.
#[inline]
fn project(x: f32, y: f32, z: f32, ax: f32, ay: f32, cx: i32, cy: i32, scale: f32) -> (i32, i32) {
    // Rotate about X axis.
    let (sax, cax) = ax.sin_cos();
    let y1 = y * cax - z * sax;
    let z1 = y * sax + z * cax;
    // Rotate about Y axis.
    let (say, cay) = ay.sin_cos();
    let x2 = x * cay + z1 * say;
    let y2 = y1;
    // Orthographic projection (drop z2, flip y for screen coords).
    let sx = cx + (x2 * scale).round() as i32;
    let sy = cy - (y2 * scale).round() as i32;
    (sx, sy)
}

/// Plot a parametric curve segment from `t0` to `t1` (in `[0, 2π]` or
/// `[0, 1]`), sampling `steps` evenly-spaced points, using `f(t) -> (x,y,z)`.
/// Each point is rotated and projected with the given angles / centre / scale.
#[inline]
fn plot_curve<F>(
    grid: &mut BrailleGrid,
    t0: f32,
    t1: f32,
    steps: usize,
    ax: f32,
    ay: f32,
    cx: i32,
    cy: i32,
    scale: f32,
    f: F,
) where
    F: Fn(f32) -> (f32, f32, f32),
{
    if steps == 0 {
        return;
    }
    for i in 0..=steps {
        let t = t0 + (t1 - t0) * (i as f32 / steps as f32);
        let (x, y, z) = f(t);
        let (sx, sy) = project(x, y, z, ax, ay, cx, cy, scale);
        draw::dot_i(grid, sx, sy);
    }
}

// ── Shared geometry: grid centre + uniform scale ──────────────────────────────

/// Return `(cx_i32, cy_i32, scale)` for a grid: centre in dot coords, and
/// scale chosen so a unit sphere just fits the smaller axis.
fn grid_cxys(grid: &BrailleGrid) -> (i32, i32, f32) {
    let (dw, dh) = draw::dot_dims(grid);
    let cx = (dw / 2) as i32;
    let cy = (dh / 2) as i32;
    let scale = (dw.min(dh * 2) as f32 * 0.42).max(1.0);
    (cx, cy, scale)
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Möbius strip
// ─────────────────────────────────────────────────────────────────────────────

struct MobiusStrip;

impl ProgressStyle for MobiusStrip {
    fn name(&self) -> &str {
        "mobius-strip"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A one-sided Möbius strip unfurling along its parametric u-axis as progress advances, rotating lazily on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.37;
        let ay = ctx.time * 0.53;
        // Reveal u from 0 to eased·2π across ~20 stripes in v.
        let u_max = ctx.eased * 2.0 * PI;
        let u_steps = 120usize;
        let v_lines = 9usize; // v ∈ [-1, 1] in v_lines steps
        for vi in 0..v_lines {
            let v = -1.0 + 2.0 * vi as f32 / (v_lines - 1).max(1) as f32;
            plot_curve(grid, 0.0, u_max, u_steps, ax, ay, cx, cy, scale, |u| {
                let half = 0.5 * v * (u / 2.0).cos();
                let x = (1.0 + half) * u.cos();
                let y = (1.0 + half) * u.sin();
                let z = 0.5 * v * (u / 2.0).sin();
                (x, y, z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Torus wireframe
// ─────────────────────────────────────────────────────────────────────────────

struct TorusWireframe;

impl ProgressStyle for TorusWireframe {
    fn name(&self) -> &str {
        "torus-wireframe"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Latitude/longitude wireframe of a (R=1, r=0.38) torus, circles revealed row-by-row as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.4 + 0.4;
        let ay = ctx.time * 0.6;
        let r_big = 1.0_f32;
        let r_small = 0.38_f32;
        let n_lat = 14usize; // latitude circles (constant v)
        let n_lon = 10usize; // longitude circles (constant u)
        let u_steps = 60usize;

        // Latitude circles: reveal the first `n_lit` of them.
        let n_lit = (ctx.eased * (n_lat + n_lon) as f32).round() as usize;

        for i in 0..n_lit.min(n_lat) {
            let v = 2.0 * PI * i as f32 / n_lat as f32;
            plot_curve(grid, 0.0, 2.0 * PI, u_steps, ax, ay, cx, cy, scale, |u| {
                let x = (r_big + r_small * v.cos()) * u.cos();
                let y = (r_big + r_small * v.cos()) * u.sin();
                let z = r_small * v.sin();
                (x, y, z)
            });
        }
        // Longitude circles.
        let n_lon_lit = n_lit.saturating_sub(n_lat).min(n_lon);
        for i in 0..n_lon_lit {
            let u = 2.0 * PI * i as f32 / n_lon as f32;
            plot_curve(grid, 0.0, 2.0 * PI, u_steps, ax, ay, cx, cy, scale, |v| {
                let x = (r_big + r_small * v.cos()) * u.cos();
                let y = (r_big + r_small * v.cos()) * u.sin();
                let z = r_small * v.sin();
                (x, y, z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Torus knot (2,3)
// ─────────────────────────────────────────────────────────────────────────────

struct TorusKnot;

impl ProgressStyle for TorusKnot {
    fn name(&self) -> &str {
        "torus-knot"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "(2,3) torus knot traced on a torus surface — a trefoil path that wraps twice around the tube for every three loops of the core"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.29;
        let ay = ctx.time * 0.47;
        let p = 2_i32;
        let q = 3_i32;
        let r_big = 1.0_f32;
        let r_small = 0.35_f32;
        // Parametric: t ∈ [0, 2π·lcm(p,q)] but one pass = 2π suffices for (2,3).
        let t_max = ctx.eased * 2.0 * PI;
        let steps = 200usize;
        plot_curve(grid, 0.0, t_max, steps, ax, ay, cx, cy, scale, |t| {
            let u = p as f32 * t;
            let v = q as f32 * t;
            let x = (r_big + r_small * v.cos()) * u.cos();
            let y = (r_big + r_small * v.cos()) * u.sin();
            let z = r_small * v.sin();
            (x, y, z)
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Trefoil knot
// ─────────────────────────────────────────────────────────────────────────────

struct TrefoilKnot;

impl ProgressStyle for TrefoilKnot {
    fn name(&self) -> &str {
        "trefoil-knot"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Trefoil knot: x=sin t+2sin 2t, y=cos t−2cos 2t, z=−sin 3t — the simplest non-trivial knot, spinning on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.33;
        let ay = ctx.time * 0.55;
        let t_max = ctx.eased * 2.0 * PI;
        let steps = 200usize;
        // Normalise scale: trefoil radius ≈ 3, so shrink by 1/3.
        let s = scale / 3.0;
        plot_curve(grid, 0.0, t_max, steps, ax, ay, cx, cy, s, |t| {
            let x = t.sin() + 2.0 * (2.0 * t).sin();
            let y = t.cos() - 2.0 * (2.0 * t).cos();
            let z = -(3.0 * t).sin();
            (x, y, z)
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Klein bottle (figure-8 immersion)
// ─────────────────────────────────────────────────────────────────────────────

struct KleinBottle;

impl ProgressStyle for KleinBottle {
    fn name(&self) -> &str {
        "klein-bottle"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Figure-8 immersion of the Klein bottle in ℝ³ — a non-orientable surface with no inside revealed petal by petal"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.28 + 0.6;
        let ay = ctx.time * 0.44;
        // Figure-8 (Lawson) immersion: u ∈ [0,π], v ∈ [0,2π].
        // Reveal u-strips up to eased·π.
        let u_max = ctx.eased * PI;
        let u_lines = 20usize;
        let v_steps = 60usize;
        let s = scale / 2.5;
        for ui in 0..u_lines {
            let u = u_max * ui as f32 / u_lines.max(1) as f32;
            plot_curve(grid, 0.0, 2.0 * PI, v_steps, ax, ay, cx, cy, s, |v| {
                // Standard figure-8 Klein bottle parametrisation.
                let cu = u.cos();
                let su = u.sin();
                let cv = v.cos();
                let sv = v.sin();
                let a = 2.5;
                let x = (a + cu * (cv + 1.0)) * (2.0 * u).cos() - su * (2.0 * u).cos() * cv;
                let y = (a + cu * (cv + 1.0)) * (2.0 * u).sin() - su * (2.0 * u).sin() * cv;
                let z = su * (cv + 1.0) + cu * sv;
                (x * 0.3, y * 0.3, z * 0.5)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Hopf fibration (three fibers)
// ─────────────────────────────────────────────────────────────────────────────

struct HopfFibers;

impl ProgressStyle for HopfFibers {
    fn name(&self) -> &str {
        "hopf-fibers"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Three Hopf fibration circles stereographically projected from S³ — every fiber is a great circle linking every other"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.31;
        let ay = ctx.time * 0.52;
        // Base points on S²: we choose 3 latitudes on the sphere, evenly spread.
        // For base point (θ,φ) on S², the Hopf fiber is the circle in S³
        // parameterised by t, and we stereographically project S³→ℝ³.
        // Base points: three points on S² at latitudes 0°, 60°, −60°.
        let bases: [(f32, f32); 3] = [
            (0.0, 0.0),
            (PI / 3.0, 2.0 * PI / 3.0),
            (-PI / 3.0, 4.0 * PI / 3.0),
        ];
        let t_max = ctx.eased * 2.0 * PI;
        let steps = 100usize;
        let s = scale * 0.55;
        for &(theta, phi) in &bases {
            // Hopf fiber: quaternion (cos α, sin α · p) where p ∈ S²,
            // stereographically projected from S³ \ {north pole} to ℝ³.
            let (st, ct) = theta.sin_cos();
            let (sp, cp) = phi.sin_cos();
            // Base point on S² as quaternion component: p = (ct·cp, ct·sp, st).
            // Fiber in S³: (cos t · 1  +  sin t · base-quat-pair).
            // We use a clean parameterisation:
            //   q(t) = (cos t · cos(θ/2),  cos t · sin(θ/2)·e^{iφ},
            //           sin t · cos(θ/2),  sin t · sin(θ/2)·e^{iφ})  [simplified]
            let hth = theta / 2.0;
            let (shth, chth) = hth.sin_cos();
            plot_curve(grid, 0.0, t_max, steps, ax, ay, cx, cy, s, |t| {
                // q = (q0,q1,q2,q3) ∈ S³.
                let q0 = t.cos() * chth;
                let q1 = t.cos() * shth * cp;
                let q2 = t.sin() * chth;
                let q3 = t.sin() * shth * cp;
                // Stereographic from north pole (1,0,0,0):
                // Project: (x,y,z) = (q1,q2,q3)/(1-q0).
                // Guard: if q0 ≈ 1 skip point.
                let denom = 1.0 - q0;
                if denom.abs() < 1e-4 {
                    (0.0, 0.0, 0.0)
                } else {
                    let inv = 1.0 / denom;
                    // Blend in phi so fibers spread out.
                    let _ = (st, sp, ct); // suppress unused
                    (q1 * inv, q2 * inv, q3 * inv)
                }
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Sphere wireframe (inflating)
// ─────────────────────────────────────────────────────────────────────────────

struct SphereInflate;

impl ProgressStyle for SphereInflate {
    fn name(&self) -> &str {
        "sphere-inflate"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Unit sphere wireframe expanding from a point to full radius as progress grows, spinning on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.35;
        let ay = ctx.time * 0.62;
        let r = ctx.eased; // radius grows with eased, 0 → 1
        let n_lat = 7usize;
        let n_lon = 8usize;
        let steps = 50usize;
        // Latitude circles (constant polar angle θ).
        for i in 0..n_lat {
            let theta = PI * (i + 1) as f32 / (n_lat + 1) as f32;
            let ring_r = r * theta.sin();
            let z0 = r * theta.cos();
            plot_curve(grid, 0.0, 2.0 * PI, steps, ax, ay, cx, cy, scale, |phi| {
                (ring_r * phi.cos(), ring_r * phi.sin(), z0)
            });
        }
        // Longitude arcs (constant azimuth φ).
        for i in 0..n_lon {
            let phi = 2.0 * PI * i as f32 / n_lon as f32;
            let (sp, cp) = phi.sin_cos();
            plot_curve(grid, 0.0, PI, steps, ax, ay, cx, cy, scale, |theta| {
                (r * theta.sin() * cp, r * theta.sin() * sp, r * theta.cos())
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Helix / double helix climbing
// ─────────────────────────────────────────────────────────────────────────────

struct HelixClimb;

impl ProgressStyle for HelixClimb {
    fn name(&self) -> &str {
        "helix-climb"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Double helix on a cylinder — two strands climb in opposite phase as progress advances, rotating with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.25 + 0.3;
        let ay = ctx.time * 0.58;
        let turns = 4.0_f32;
        let t_max = ctx.eased * turns * 2.0 * PI;
        let steps = 200usize;
        let r = 0.6_f32;
        let height = 2.0_f32; // total height -1 to +1
                              // Strand A.
        plot_curve(grid, 0.0, t_max, steps, ax, ay, cx, cy, scale, |t| {
            let z = -1.0 + height * t / (turns * 2.0 * PI);
            (r * t.cos(), r * t.sin(), z)
        });
        // Strand B (π offset in phase).
        plot_curve(grid, 0.0, t_max, steps, ax, ay, cx, cy, scale, |t| {
            let z = -1.0 + height * t / (turns * 2.0 * PI);
            (r * (t + PI).cos(), r * (t + PI).sin(), z)
        });
        // Cross-links every half turn.
        let n_links = (ctx.eased * turns * 2.0) as usize;
        for i in 0..n_links {
            let t_link = i as f32 * PI;
            let z = -1.0 + height * t_link / (turns * 2.0 * PI);
            plot_curve(grid, 0.0, 1.0, 4, ax, ay, cx, cy, scale, |s| {
                let angle_a = t_link;
                let angle_b = t_link + PI;
                let xa = r * angle_a.cos();
                let ya = r * angle_a.sin();
                let xb = r * angle_b.cos();
                let yb = r * angle_b.sin();
                (xa + s * (xb - xa), ya + s * (yb - ya), z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Saddle surface (hyperbolic paraboloid)
// ─────────────────────────────────────────────────────────────────────────────

struct SaddleSurface;

impl ProgressStyle for SaddleSurface {
    fn name(&self) -> &str {
        "saddle-surface"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Hyperbolic paraboloid z = x²−y² wireframe — the canonical saddle point, revealed as a grid of iso-lines, rotating with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.22 + 0.5;
        let ay = ctx.time * 0.48;
        let n_lines = 12usize;
        let revealed = (ctx.eased * n_lines as f32 * 2.0).round() as usize;
        let steps = 60usize;
        // x-parallel iso-lines (vary y, constant x).
        for i in 0..revealed.min(n_lines) {
            let x = -1.0 + 2.0 * i as f32 / (n_lines - 1).max(1) as f32;
            plot_curve(grid, -1.0, 1.0, steps, ax, ay, cx, cy, scale, |y| {
                (x, y, x * x - y * y)
            });
        }
        // y-parallel iso-lines (vary x, constant y).
        let extra = revealed.saturating_sub(n_lines);
        for i in 0..extra.min(n_lines) {
            let y = -1.0 + 2.0 * i as f32 / (n_lines - 1).max(1) as f32;
            plot_curve(grid, -1.0, 1.0, steps, ax, ay, cx, cy, scale, |x| {
                (x, y, x * x - y * y)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Tesseract (4-D hypercube) double projection
// ─────────────────────────────────────────────────────────────────────────────

struct TesseractSpin;

impl ProgressStyle for TesseractSpin {
    fn name(&self) -> &str {
        "tesseract-spin"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "4-D hypercube (tesseract) rotated in the XW and YZ planes then perspective-projected to 2-D — edges revealed by progress, spinning on time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;
        let scale = (dw.min(dh * 2) as f32 * 0.35).max(1.0);

        // The 16 vertices of a unit tesseract in (x,y,z,w) ∈ {-1,+1}^4.
        let verts: [[f32; 4]; 16] = {
            let mut v = [[0.0f32; 4]; 16];
            for i in 0..16usize {
                v[i][0] = if i & 1 != 0 { 1.0 } else { -1.0 };
                v[i][1] = if i & 2 != 0 { 1.0 } else { -1.0 };
                v[i][2] = if i & 4 != 0 { 1.0 } else { -1.0 };
                v[i][3] = if i & 8 != 0 { 1.0 } else { -1.0 };
            }
            v
        };
        // The 32 edges: pairs that differ in exactly one bit.
        let mut edges: Vec<(usize, usize)> = Vec::with_capacity(32);
        for i in 0..16usize {
            for j in (i + 1)..16usize {
                if (i ^ j).count_ones() == 1 {
                    edges.push((i, j));
                }
            }
        }

        // Reveal edges up to progress.
        let n_show = (ctx.eased * edges.len() as f32).round() as usize;

        // 4-D rotation angles: XW plane (time), YZ plane (time/2).
        let a_xw = ctx.time * 0.48;
        let a_yz = ctx.time * 0.31;

        // Rotate all 16 vertices.
        let project4 = |vert: [f32; 4]| -> (i32, i32) {
            let [x0, y0, z0, w0] = vert;
            // XW rotation.
            let (sxw, cxw) = a_xw.sin_cos();
            let x1 = x0 * cxw - w0 * sxw;
            let w1 = x0 * sxw + w0 * cxw;
            // YZ rotation.
            let (syz, cyz) = a_yz.sin_cos();
            let y1 = y0 * cyz - z0 * syz;
            let z1 = y0 * syz + z0 * cyz;
            // 3-D XY rotation from ctx.time (slow tumble).
            let ax = ctx.time * 0.22;
            let ay = ctx.time * 0.37;
            let (sax, cax) = ax.sin_cos();
            let y2 = y1 * cax - z1 * sax;
            let z2 = y1 * sax + z1 * cax;
            let (say, cay) = ay.sin_cos();
            let x3 = x1 * cay + z2 * say;
            let y3 = y2;
            // Perspective from w-depth.
            let w_dist = 2.5 - w1 * 0.5;
            let inv = if w_dist.abs() < 0.01 {
                0.0
            } else {
                1.0 / w_dist
            };
            let sx = cx + (x3 * inv * scale) as i32;
            let sy = cy - (y3 * inv * scale) as i32;
            (sx, sy)
        };

        let projected: Vec<(i32, i32)> = verts.iter().map(|&v| project4(v)).collect();

        for &(a, b) in edges.iter().take(n_show) {
            let (x0, y0) = projected[a];
            let (x1, y1) = projected[b];
            // Bresenham line between projected endpoints.
            let dx = (x1 - x0).abs();
            let dy = (y1 - y0).abs();
            let sx = if x0 < x1 { 1i32 } else { -1i32 };
            let sy = if y0 < y1 { 1i32 } else { -1i32 };
            let mut x = x0;
            let mut y = y0;
            let mut err = dx - dy;
            let max_steps = (dx + dy + 1).min(300);
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
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Roman surface (Steiner's surface)
// ─────────────────────────────────────────────────────────────────────────────

struct RomanSurface;

impl ProgressStyle for RomanSurface {
    fn name(&self) -> &str {
        "roman-surface"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Steiner's Roman surface — one of the most beautiful immersions of ℝP² into ℝ³, with six Whitney umbrella singularities, revealed petal by petal"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.27 + 0.4;
        let ay = ctx.time * 0.43;
        // Parametrisation: u ∈ [0,π], v ∈ [0,π].
        // Reveal u-strips up to eased·π.
        let u_max = ctx.eased * PI;
        let u_lines = 18usize;
        let v_steps = 60usize;
        let s = scale * 0.6;
        for ui in 0..u_lines {
            let u = u_max * ui as f32 / u_lines.max(1) as f32;
            let (su, cu) = u.sin_cos();
            let s2u = (2.0 * u).sin();
            plot_curve(grid, 0.0, PI, v_steps, ax, ay, cx, cy, s, |v| {
                let (sv, _cv) = v.sin_cos();
                let s2v = (2.0 * v).sin();
                // Roman surface: x=sin²u·sin 2v, y=sin 2u·sin²v, z=sin 2u·sin 2v / 2.
                let x = su * su * s2v;
                let y = s2u * sv * sv;
                let z = 0.5 * s2u * s2v;
                let _ = cu;
                (x, y, z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Seifert surface spiral ramp
// ─────────────────────────────────────────────────────────────────────────────

struct SeifertRamp;

impl ProgressStyle for SeifertRamp {
    fn name(&self) -> &str {
        "seifert-ramp"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Seifert-surface-inspired spiral ramp spanning a trefoil knot boundary — a twisted disk that rises through a full turn as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.30 + 0.5;
        let ay = ctx.time * 0.50;
        // Seifert surface for trefoil: parametrised by (r, theta) in a disk,
        // lifted to 3-D via the trefoil fibration.
        // Approximation: r ∈ [0,1], theta ∈ [0, 2π].
        // Point: rotate by theta/3 (fibration angle), scale radially.
        // x = r·cos(theta), y = r·sin(theta), z = r·sin(theta/3)·(1-r).
        let theta_max = ctx.eased * 2.0 * PI;
        let n_radii = 8usize;
        let steps = 80usize;
        // Concentric rings.
        for ri in 1..=n_radii {
            let r = ri as f32 / n_radii as f32;
            plot_curve(
                grid,
                0.0,
                theta_max,
                steps,
                ax,
                ay,
                cx,
                cy,
                scale,
                |theta| {
                    let x = r * theta.cos();
                    let y = r * theta.sin();
                    let z = r * (theta / 3.0).sin() * (1.0 - r * 0.5);
                    (x, y, z)
                },
            );
        }
        // Radial spokes.
        let n_spokes = 12usize;
        for si in 0..n_spokes {
            let theta = theta_max * si as f32 / n_spokes.max(1) as f32;
            plot_curve(grid, 0.0, 1.0, 20, ax, ay, cx, cy, scale, |r| {
                let x = r * theta.cos();
                let y = r * theta.sin();
                let z = r * (theta / 3.0).sin() * (1.0 - r * 0.5);
                (x, y, z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — green into blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(134, 232, 184);
const TINT_END: Color = Color::rgb(64, 104, 224);

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

/// All styles in the `topology` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per surface type, in the order they
/// appear in the source: Möbius, torus wireframe, torus knot, trefoil knot,
/// Klein bottle, Hopf fibers, sphere inflate, helix climb, saddle surface,
/// tesseract spin, Roman surface, Seifert ramp.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(MobiusStrip)),
        Box::new(Tinted(TorusWireframe)),
        Box::new(Tinted(TorusKnot)),
        Box::new(Tinted(TrefoilKnot)),
        Box::new(Tinted(KleinBottle)),
        Box::new(Tinted(HopfFibers)),
        Box::new(Tinted(SphereInflate)),
        Box::new(Tinted(HelixClimb)),
        Box::new(Tinted(SaddleSurface)),
        Box::new(Tinted(TesseractSpin)),
        Box::new(Tinted(RomanSurface)),
        Box::new(Tinted(SeifertRamp)),
    ]
}

    }
}





}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::topology::styles();
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
