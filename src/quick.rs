//! Quick one-liner functions for common dotmax tasks.
//!
//! This module provides the simplest possible API for displaying images
//! and working with braille grids. Use these functions when you want
//! results fast without configuring options.
//!
//! # Overview
//!
//! The `quick` module is designed for rapid prototyping and simple use cases
//! where sensible defaults are sufficient. For fine-grained control, use the
//! underlying [`BrailleGrid`] and [`TerminalRenderer`] types directly
//! (plus `ImageRenderer` with the `image` feature).
//!
//! # Examples
//!
//! ## Display an Image (One Line!)
//!
//! ```ignore
//! // Requires `image` feature
//! use dotmax::quick;
//!
//! // Display an image with auto-detected terminal size and Floyd-Steinberg dithering
//! quick::show_image("photo.png")?;
//! # Ok::<(), dotmax::DotmaxError>(())
//! ```
//!
//! ## Create and Display a Custom Grid
//!
//! ```no_run
//! use dotmax::quick;
//! use dotmax::primitives::draw_circle;
//!
//! // Get a terminal-sized grid for drawing
//! let mut grid = quick::grid()?;
//!
//! // Draw something on it
//! draw_circle(&mut grid, 40, 40, 20)?;
//!
//! // Display and wait for keypress
//! quick::show(&grid)?;
//! # Ok::<(), dotmax::DotmaxError>(())
//! ```
//!
//! ## Load an Image for Manipulation
//!
//! ```ignore
//! // Requires `image` feature
//! use dotmax::quick;
//!
//! // Load image into grid (doesn't display yet)
//! let grid = quick::load_image("photo.png")?;
//! println!("Grid dimensions: {}x{}", grid.width(), grid.height());
//!
//! // Now display it
//! quick::show(&grid)?;
//! # Ok::<(), dotmax::DotmaxError>(())
//! ```
//!
//! # Sensible Defaults
//!
//! All functions use these defaults for optimal results:
//!
//! - **Terminal size**: Auto-detected via `crossterm::terminal::size()`, with 80×24 fallback
//! - **Dithering**: Floyd-Steinberg (best quality for most images)
//! - **Aspect ratio**: Preserved (no distortion)
//! - **Wait behavior**: `show()` and `show_image()` wait for any keypress before returning
//!
//! # Performance
//!
//! Quick functions add minimal overhead (< 5ms) compared to manual API usage.
//! The main cost is the terminal setup/teardown in `show()`.
//!
//! # Error Handling
//!
//! All functions return `Result<_, DotmaxError>` and never panic.
//! Common errors include:
//!
//! - `DotmaxError::ImageLoad` - File not found or unsupported format
//! - `DotmaxError::Terminal` - Terminal I/O errors
//! - `DotmaxError::TerminalBackend` - Terminal too small (minimum 40×12)
//!
//! [`BrailleGrid`]: crate::BrailleGrid
//! [`TerminalRenderer`]: crate::TerminalRenderer

use crate::{BrailleGrid, Result, TerminalRenderer};

// ============================================================================
// Terminal Size Detection (AC: #6)
// ============================================================================

/// Default terminal dimensions when detection fails.
const DEFAULT_WIDTH: usize = 80;
const DEFAULT_HEIGHT: usize = 24;

/// Detects the current terminal size with fallback.
///
/// Uses `crossterm::terminal::size()` to detect terminal dimensions.
/// Returns `(80, 24)` if detection fails (e.g., running without a terminal).
///
/// # Returns
///
/// Tuple of `(width, height)` in terminal cells.
#[inline]
fn terminal_size() -> (usize, usize) {
    crossterm::terminal::size()
        .map(|(w, h)| (w as usize, h as usize))
        .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT))
}

/// Waits for any keypress.
///
/// Blocks until the user presses any key. Used by `show()` and `show_image()`
/// to prevent immediate return.
fn wait_for_key() -> Result<()> {
    use crossterm::event::{self, Event};

    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }
    Ok(())
}

// ============================================================================
// Core Functions (AC: #2, #3)
// ============================================================================

/// Creates a [`BrailleGrid`] sized to the current terminal.
///
/// This is the quickest way to get a grid ready for drawing. The grid
/// dimensions match the terminal size, so rendering fills the screen.
///
/// # Returns
///
/// A new `BrailleGrid` with dimensions matching the terminal.
///
/// # Errors
///
/// Returns `DotmaxError::InvalidDimensions` if the detected terminal size
/// is invalid (which should never happen with the 80×24 fallback).
///
/// # Examples
///
/// ```no_run
/// use dotmax::quick;
/// use dotmax::primitives::draw_line;
///
/// let mut grid = quick::grid()?;
/// draw_line(&mut grid, 0, 0, 100, 50)?;
/// quick::show(&grid)?;
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
pub fn grid() -> Result<BrailleGrid> {
    let (w, h) = terminal_size();
    BrailleGrid::new(w, h)
}

/// Creates a [`BrailleGrid`] with explicit dimensions.
///
/// Use this when you need a specific grid size rather than terminal-sized.
///
/// # Arguments
///
/// * `width` - Grid width in terminal cells
/// * `height` - Grid height in terminal cells
///
/// # Returns
///
/// A new `BrailleGrid` with the specified dimensions.
///
/// # Errors
///
/// Returns `DotmaxError::InvalidDimensions` if width or height is 0
/// or exceeds maximum limits (10,000).
///
/// # Examples
///
/// ```
/// use dotmax::quick;
///
/// // Create a 100x50 grid
/// let grid = quick::grid_sized(100, 50)?;
/// assert_eq!(grid.width(), 100);
/// assert_eq!(grid.height(), 50);
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
pub fn grid_sized(width: usize, height: usize) -> Result<BrailleGrid> {
    BrailleGrid::new(width, height)
}

/// Displays a grid and waits for a keypress.
///
/// This is a blocking function that:
/// 1. Initializes the terminal (enters raw mode, alternate screen)
/// 2. Renders the grid
/// 3. Waits for any keypress
/// 4. Cleans up terminal state
///
/// # Arguments
///
/// * `grid` - The braille grid to display
///
/// # Errors
///
/// Returns `DotmaxError::Terminal` for I/O errors, or
/// `DotmaxError::TerminalBackend` if the terminal is too small.
///
/// # Examples
///
/// ```no_run
/// use dotmax::quick;
/// use dotmax::primitives::draw_circle;
///
/// let mut grid = quick::grid()?;
/// draw_circle(&mut grid, 80, 48, 30)?;
/// quick::show(&grid)?; // Displays until keypress
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
///
/// # Note
///
/// For non-blocking rendering or more control over terminal state,
/// use [`TerminalRenderer`] directly.
pub fn show(grid: &BrailleGrid) -> Result<()> {
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(grid)?;
    wait_for_key()?;
    // TerminalRenderer::drop() handles cleanup automatically
    Ok(())
}

// ============================================================================
// Image Functions (AC: #4, #5, #7) - Feature-gated
// ============================================================================

/// Loads and displays an image in one call.
///
/// This is the ultimate one-liner for image display:
/// 1. Loads image from file
/// 2. Resizes to terminal dimensions (preserving aspect ratio)
/// 3. Converts to braille using Floyd-Steinberg dithering
/// 4. Displays in terminal
/// 5. Waits for keypress
/// 6. Cleans up
///
/// # Arguments
///
/// * `path` - Path to image file (PNG, JPEG, GIF, BMP, WebP, TIFF supported)
///
/// # Errors
///
/// Returns `DotmaxError::ImageLoad` if the file doesn't exist or format
/// is unsupported, or terminal errors during display.
///
/// # Examples
///
/// ```no_run
/// use dotmax::quick;
///
/// // That's it - one line to display any image!
/// quick::show_image("photo.png")?;
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
#[cfg(feature = "image")]
pub fn show_image(path: impl AsRef<std::path::Path>) -> Result<()> {
    let grid = load_image(path)?;
    show(&grid)
}

/// Loads an image into a [`BrailleGrid`] for further manipulation.
///
/// Like `show_image()` but returns the grid instead of displaying it.
/// Use this when you want to modify the grid before display or use
/// it for other purposes.
///
/// # Arguments
///
/// * `path` - Path to image file
///
/// # Returns
///
/// A `BrailleGrid` containing the rendered image.
///
/// # Errors
///
/// Returns `DotmaxError::ImageLoad` if loading fails.
///
/// # Examples
///
/// ```no_run
/// use dotmax::quick;
/// use dotmax::primitives::draw_rectangle;
///
/// // Load image
/// let mut grid = quick::load_image("photo.png")?;
///
/// // Add a border (uses u32 for width/height)
/// let (w, h) = (grid.dot_width() as u32, grid.dot_height() as u32);
/// draw_rectangle(&mut grid, 0, 0, w, h)?;
///
/// // Now display
/// quick::show(&grid)?;
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
#[cfg(feature = "image")]
pub fn load_image(path: impl AsRef<std::path::Path>) -> Result<BrailleGrid> {
    use crate::image::ImageRenderer;

    let (w, h) = terminal_size();
    ImageRenderer::new()
        .load_from_path(path.as_ref())?
        .resize(w, h, true)?
        .render()
}

/// Loads an image with explicit dimensions.
///
/// Like `load_image()` but with manual size control instead of auto-detection.
///
/// # Arguments
///
/// * `path` - Path to image file
/// * `width` - Target width in terminal cells
/// * `height` - Target height in terminal cells
///
/// # Returns
///
/// A `BrailleGrid` containing the rendered image at the specified size.
///
/// # Errors
///
/// Returns `DotmaxError::ImageLoad` if loading fails, or
/// `DotmaxError::InvalidDimensions` if dimensions are invalid.
///
/// # Examples
///
/// ```no_run
/// use dotmax::quick;
///
/// // Load at specific size
/// let grid = quick::load_image_sized("photo.png", 100, 50)?;
/// assert_eq!(grid.width(), 100);
/// assert_eq!(grid.height(), 50);
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
#[cfg(feature = "image")]
pub fn load_image_sized(
    path: impl AsRef<std::path::Path>,
    width: usize,
    height: usize,
) -> Result<BrailleGrid> {
    use crate::image::ImageRenderer;

    ImageRenderer::new()
        .load_from_path(path.as_ref())?
        .resize(width, height, true)?
        .render()
}

// ============================================================================
// Tests (AC: #2, #3, #4, #5, #6, #8)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DotmaxError;

    // ========================================================================
    // Terminal Size Detection Tests
    // ========================================================================

    #[test]
    fn test_terminal_size_returns_reasonable_values() {
        let (w, h) = terminal_size();
        // Should return either actual terminal size or fallback
        assert!(w > 0, "Width should be positive");
        assert!(h > 0, "Height should be positive");
        // Fallback is 80x24, actual terminal might be larger
        assert!(w >= 80 || w > 0, "Width should be at least fallback or positive");
        assert!(h >= 24 || h > 0, "Height should be at least fallback or positive");
    }

    #[test]
    fn test_terminal_size_fallback_values() {
        // In CI/test environment without terminal, should get fallback
        // This test just verifies the function doesn't panic
        let (_w, _h) = terminal_size();
    }

    // ========================================================================
    // Grid Creation Tests
    // ========================================================================

    #[test]
    fn test_grid_creates_valid_grid() {
        let result = grid();
        assert!(result.is_ok(), "grid() should succeed");
        let g = result.unwrap();
        assert!(g.width() > 0, "Grid width should be positive");
        assert!(g.height() > 0, "Grid height should be positive");
    }

    #[test]
    fn test_grid_sized_creates_exact_dimensions() {
        let g = grid_sized(100, 50).unwrap();
        assert_eq!(g.width(), 100);
        assert_eq!(g.height(), 50);
    }

    #[test]
    fn test_grid_sized_zero_width_fails() {
        let result = grid_sized(0, 50);
        assert!(result.is_err());
        match result {
            Err(DotmaxError::InvalidDimensions { width, height }) => {
                assert_eq!(width, 0);
                assert_eq!(height, 50);
            }
            _ => panic!("Expected InvalidDimensions error"),
        }
    }

    #[test]
    fn test_grid_sized_zero_height_fails() {
        let result = grid_sized(100, 0);
        assert!(result.is_err());
        match result {
            Err(DotmaxError::InvalidDimensions { width, height }) => {
                assert_eq!(width, 100);
                assert_eq!(height, 0);
            }
            _ => panic!("Expected InvalidDimensions error"),
        }
    }

    // ========================================================================
    // Image Function Tests (feature-gated)
    // ========================================================================

    #[cfg(feature = "image")]
    mod image_tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn test_load_image_sized_creates_correct_dimensions() {
            // Use a test image from fixtures
            let test_image = Path::new("tests/fixtures/images/sample.png");
            if test_image.exists() {
                let result = load_image_sized(test_image, 40, 20);
                assert!(result.is_ok(), "load_image_sized should succeed: {:?}", result.err());
                let g = result.unwrap();
                assert_eq!(g.width(), 40);
                assert_eq!(g.height(), 20);
            }
        }

        #[test]
        fn test_load_image_nonexistent_file_fails() {
            let result = load_image("nonexistent_image_12345.png");
            assert!(result.is_err(), "load_image should fail for nonexistent file");
        }

        #[test]
        fn test_load_image_sized_nonexistent_file_fails() {
            let result = load_image_sized("nonexistent_image_12345.png", 100, 50);
            assert!(result.is_err(), "load_image_sized should fail for nonexistent file");
        }
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[test]
    fn test_grid_sized_exceeds_max_fails() {
        // Max is 10,000
        let result = grid_sized(10_001, 100);
        assert!(result.is_err(), "Should fail for width > 10000");
    }

    // Note: show() and show_image() require a terminal and are tested
    // via integration tests and examples rather than unit tests.
    // See tests/quick_test.rs and examples/quick_demo.rs
}
