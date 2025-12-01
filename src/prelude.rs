//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from dotmax,
//! allowing you to import everything you need with a single statement:
//!
//! ```
//! use dotmax::prelude::*;
//! ```
//!
//! This follows the standard Rust convention used by libraries like
//! `std::prelude`, `tokio::prelude`, and `serde::prelude`.
//!
//! # What's Included
//!
//! ## Core Types
//!
//! - [`BrailleGrid`]: The fundamental braille rendering buffer
//! - [`Color`]: RGB color for cells and drawing operations
//! - [`TerminalRenderer`]: Handles terminal output and cursor management
//! - [`TerminalBackend`]: Trait for custom terminal backends
//! - [`TerminalCapabilities`]: Terminal feature detection
//! - [`DotmaxError`]: Error type for all operations
//! - [`Result`]: Convenience type alias (`Result<T, DotmaxError>`)
//!
//! ## Drawing Primitives
//!
//! - [`draw_line`], [`draw_line_colored`]: Line drawing (Bresenham algorithm)
//! - [`draw_circle`], [`draw_circle_colored`]: Circle drawing
//! - [`draw_rectangle`], [`draw_rectangle_colored`]: Rectangle drawing
//! - [`draw_polygon`], [`draw_polygon_colored`]: Polygon drawing
//!
//! ## Animation
//!
//! - [`AnimationLoop`], [`AnimationLoopBuilder`]: Animation loop management
//! - [`FrameBuffer`]: Double-buffered frame storage
//! - [`FrameTimer`]: Frame rate control and timing
//! - [`DifferentialRenderer`]: Efficient partial screen updates
//! - [`PrerenderedAnimation`]: Pre-computed animation frames
//!
//! ## Color System
//!
//! - [`ColorScheme`], [`ColorSchemeBuilder`]: Color scheme types
//! - [`ColorCapability`]: Terminal color capability detection
//! - [`detect_color_capability`]: Auto-detect terminal color support
//! - [`apply_color_scheme`], [`apply_colors_to_grid`]: Apply colors to grids
//! - Built-in schemes: [`heat_map`], [`rainbow`], [`grayscale`], [`monochrome`],
//!   [`blue_purple`], [`cyan_magenta`], [`green_yellow`]
//!
//! # Feature-Gated Exports
//!
//! With the `image` feature enabled, also includes:
//!
//! - `ImageRenderer`: High-level image-to-braille rendering
//! - `DitheringMethod`: Dithering algorithm selection
//!
//! # Note on `Result`
//!
//! This prelude exports `dotmax::Result<T>` which is an alias for
//! `std::result::Result<T, DotmaxError>`. If you need to use a different
//! error type (like `Box<dyn std::error::Error>`), use the fully qualified
//! `std::result::Result` instead.
//!
//! # Examples
//!
//! ## Basic Grid Usage
//!
//! ```
//! use dotmax::prelude::*;
//!
//! // Create a grid and draw some shapes
//! let mut grid = BrailleGrid::new(80, 24)?;
//! draw_line(&mut grid, 0, 0, 100, 50)?;
//! draw_circle(&mut grid, 80, 48, 30)?;
//! # Ok::<(), DotmaxError>(())
//! ```
//!
//! ## Animation Loop
//!
//! ```no_run
//! use dotmax::prelude::*;
//!
//! // AnimationLoop::new(width, height) returns an AnimationLoopBuilder
//! let animation = AnimationLoop::new(80, 24)
//!     .fps(30);
//! ```
//!
//! ## Color Schemes
//!
//! ```
//! use dotmax::prelude::*;
//!
//! let scheme = heat_map();
//! let capability = detect_color_capability();
//! # Ok::<(), DotmaxError>(())
//! ```
//!
//! ## Image Rendering (with `image` feature)
//!
//! ```ignore
//! use dotmax::prelude::*;
//! use std::path::Path;
//!
//! let grid = ImageRenderer::new()
//!     .load_from_path(Path::new("image.png"))?
//!     .resize_to_terminal()?
//!     .dithering(DitheringMethod::FloydSteinberg)
//!     .render()?;
//! ```

// ============================================================================
// Core Types (AC: #2)
// ============================================================================

pub use crate::{
    BrailleGrid, Color, DotmaxError, Result, TerminalBackend, TerminalCapabilities,
    TerminalRenderer,
};

// ============================================================================
// Drawing Primitives (AC: #3)
// ============================================================================

pub use crate::primitives::{
    draw_circle, draw_circle_colored, draw_line, draw_line_colored, draw_polygon,
    draw_polygon_colored, draw_rectangle, draw_rectangle_colored,
};

// ============================================================================
// Animation Types (AC: #4)
// ============================================================================

pub use crate::{
    AnimationLoop, AnimationLoopBuilder, DifferentialRenderer, FrameBuffer, FrameTimer,
    PrerenderedAnimation,
};

// ============================================================================
// Color Types (AC: #5)
// ============================================================================

pub use crate::{
    apply_color_scheme, apply_colors_to_grid, blue_purple, cyan_magenta, detect_color_capability,
    grayscale, green_yellow, heat_map, monochrome, rainbow, ColorCapability, ColorScheme,
    ColorSchemeBuilder,
};

// ============================================================================
// Image Types - Feature-Gated (AC: #6)
// ============================================================================

#[cfg(feature = "image")]
pub use crate::image::{DitheringMethod, ImageRenderer};

// ============================================================================
// Quick Functions (Story 8.2)
// ============================================================================

pub use crate::quick::{grid, grid_sized, show};

#[cfg(feature = "image")]
pub use crate::quick::{load_image, load_image_sized, show_image};

// ============================================================================
// Tests (AC: #7)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_types_accessible() {
        // Test BrailleGrid
        let grid = BrailleGrid::new(10, 5).unwrap();
        assert_eq!(grid.width(), 10);

        // Test Color
        let color = Color::rgb(255, 128, 0);
        assert_eq!(color.r, 255);

        // Test Result type alias compiles
        fn returns_result() -> Result<()> {
            Ok(())
        }
        returns_result().unwrap();
    }

    #[test]
    fn test_primitives_accessible() {
        let mut grid = BrailleGrid::new(20, 10).unwrap();

        // All primitives should be callable
        draw_line(&mut grid, 0, 0, 10, 10).unwrap();
        draw_circle(&mut grid, 20, 20, 5).unwrap();
        draw_rectangle(&mut grid, 0, 0, 10, 10).unwrap();
        draw_polygon(&mut grid, &[(0, 0), (10, 0), (5, 10)]).unwrap();

        // Colored variants (with proper arguments)
        let color = Color::rgb(255, 0, 0);
        draw_line_colored(&mut grid, 0, 0, 5, 5, color, None).unwrap();
        draw_circle_colored(&mut grid, 15, 15, 3, color, false).unwrap();
        draw_rectangle_colored(&mut grid, 5, 5, 10, 10, color, false).unwrap();
        draw_polygon_colored(&mut grid, &[(0, 0), (5, 0), (2, 5)], color, true).unwrap();
    }

    #[test]
    fn test_animation_types_accessible() {
        // Test FrameBuffer
        let fb = FrameBuffer::new(10, 5);
        assert_eq!(fb.width(), 10);

        // Test FrameTimer
        let timer = FrameTimer::new(30);
        assert_eq!(timer.target_fps(), 30);

        // Test AnimationLoopBuilder (created via AnimationLoop::new(w, h))
        let _builder = AnimationLoop::new(80, 24).fps(60);

        // Test DifferentialRenderer (takes no args)
        let _diff = DifferentialRenderer::new();

        // Test PrerenderedAnimation (takes frame_rate)
        let mut prerendered = PrerenderedAnimation::new(30);
        prerendered.add_frame(BrailleGrid::new(10, 5).unwrap());
        assert_eq!(prerendered.frame_rate(), 30);
    }

    #[test]
    fn test_color_types_accessible() {
        // Test ColorCapability
        let _cap = ColorCapability::TrueColor;

        // Test detect_color_capability function
        let _detected = detect_color_capability();

        // Test ColorScheme via built-in functions
        let _scheme1 = heat_map();
        let _scheme2 = rainbow();
        let _scheme3 = grayscale();
        let _scheme4 = monochrome();
        let _scheme5 = blue_purple();
        let _scheme6 = cyan_magenta();
        let _scheme7 = green_yellow();

        // Test ColorSchemeBuilder
        let _builder = ColorSchemeBuilder::new("test");
    }

    #[test]
    #[cfg(feature = "image")]
    fn test_image_types_accessible() {
        // Test ImageRenderer
        let _renderer = ImageRenderer::new();

        // Test DitheringMethod
        let _method = DitheringMethod::FloydSteinberg;
    }

    #[test]
    fn test_no_naming_conflicts() {
        // This test verifies that all re-exported items can be used together
        // without naming conflicts. If this compiles, there are no conflicts.

        let mut grid = BrailleGrid::new(20, 10).unwrap();
        let color = Color::rgb(128, 128, 128);
        let scheme = grayscale();
        let _cap = detect_color_capability();

        draw_line(&mut grid, 0, 0, 10, 10).unwrap();
        draw_line_colored(&mut grid, 5, 5, 15, 15, color, None).unwrap();

        let fb = FrameBuffer::new(20, 10);
        let _timer = FrameTimer::new(30);

        // If this function compiles, there are no conflicts
        fn _use_result_alias() -> Result<()> {
            Ok(())
        }

        // Verify scheme methods work (sample takes f32 intensity 0.0-1.0)
        let _sampled_color = scheme.sample(0.5);

        // Verify we can access buffer dimensions
        assert_eq!(fb.width(), 20);
        assert_eq!(fb.height(), 10);
    }
}
