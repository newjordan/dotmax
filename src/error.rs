//! Error types for dotmax operations
//!
//! This module defines `DotmaxError`, the primary error type returned by all
//! public dotmax APIs. All errors include contextual information (coordinates,
//! dimensions, indices) to aid debugging.
//!
//! # Zero Panics Policy
//!
//! All public API methods return `Result<T, DotmaxError>` instead of panicking.
//! This ensures applications can gracefully handle all error conditions.
//!
//! # Examples
//!
//! ```
//! use dotmax::{BrailleGrid, DotmaxError};
//!
//! // Create grid with invalid dimensions
//! let result = BrailleGrid::new(0, 10);
//! match result {
//!     Err(DotmaxError::InvalidDimensions { width, height }) => {
//!         println!("Invalid dimensions: {}×{}", width, height);
//!     }
//!     _ => unreachable!(),
//! }
//!
//! // Access out-of-bounds coordinates
//! let mut grid = BrailleGrid::new(10, 10).unwrap();
//! let result = grid.set_dot(100, 50);
//! match result {
//!     Err(DotmaxError::OutOfBounds { x, y, width, height }) => {
//!         println!("({}, {}) is outside {}×{} grid", x, y, width, height);
//!     }
//!     _ => unreachable!(),
//! }
//! ```

use thiserror::Error;

/// Comprehensive error type for all dotmax operations
///
/// All variants include contextual information to aid debugging and provide
/// actionable error messages to end users.
#[derive(Error, Debug)]
pub enum DotmaxError {
    /// Grid dimensions are invalid (zero or exceeding maximum limits)
    ///
    /// Valid dimensions must satisfy:
    /// - `width > 0 && width <= 10,000`
    /// - `height > 0 && height <= 10,000`
    #[error("Invalid grid dimensions: width={width}, height={height}")]
    InvalidDimensions { width: usize, height: usize },

    /// Coordinate access is outside grid boundaries
    ///
    /// Valid coordinates must satisfy:
    /// - `x < width`
    /// - `y < height`
    #[error("Out of bounds access: ({x}, {y}) in grid of size ({width}, {height})")]
    OutOfBounds {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },

    /// Dot index is invalid (must be 0-7 for 2×4 braille cells)
    ///
    /// Valid dot indices:
    /// ```text
    /// 0 3    (positions in braille cell)
    /// 1 4
    /// 2 5
    /// 6 7
    /// ```
    #[error("Invalid dot index: {index} (must be 0-7)")]
    InvalidDotIndex { index: u8 },

    /// Terminal I/O error from underlying terminal backend
    ///
    /// This wraps `std::io::Error` using `#[from]` to preserve the error source
    /// chain for proper debugging and error context propagation.
    #[error("Terminal I/O error: {0}")]
    Terminal(#[from] std::io::Error),

    /// Terminal backend operation failed
    ///
    /// Used for terminal-specific errors that don't map to standard I/O errors
    /// (e.g., capability detection failures, initialization errors).
    #[error("Terminal backend error: {0}")]
    TerminalBackend(String),

    /// Unicode braille character conversion failed
    ///
    /// This should rarely occur as braille Unicode range (U+2800–U+28FF) is
    /// well-defined, but may happen if cell data becomes corrupted.
    #[error("Unicode conversion failed for cell ({x}, {y})")]
    UnicodeConversion { x: usize, y: usize },

    /// Image loading failed (file not found, decode error, etc.)
    ///
    /// This error wraps the underlying `image::ImageError` using `#[source]`
    /// to preserve the error chain for debugging.
    ///
    /// Common causes:
    /// - File does not exist or is not readable
    /// - File format is corrupted or unsupported
    /// - Memory allocation failure during decode
    #[cfg(feature = "image")]
    #[error("Failed to load image from {path:?}: {source}")]
    ImageLoad {
        path: std::path::PathBuf,
        #[source]
        source: image::ImageError,
    },

    /// Unsupported image format
    ///
    /// The provided file or byte buffer is not in a supported image format.
    /// See [`crate::image::supported_formats`] for the list of valid formats.
    #[cfg(feature = "image")]
    #[error("Unsupported image format: {format}")]
    UnsupportedFormat { format: String },

    /// Image dimensions exceed maximum limits
    ///
    /// Images larger than 10,000×10,000 pixels are rejected to prevent
    /// memory exhaustion attacks.
    #[cfg(feature = "image")]
    #[error("Invalid image dimensions: {width}×{height} exceeds maximum (10,000×10,000)")]
    InvalidImageDimensions { width: u32, height: u32 },

    /// Invalid parameter value provided to image processing function
    ///
    /// This error is returned when a function parameter (brightness, contrast,
    /// gamma, etc.) is outside its valid range.
    ///
    /// The error message includes:
    /// - Parameter name (e.g., "brightness factor")
    /// - Provided value
    /// - Valid range (min-max)
    #[cfg(feature = "image")]
    #[error("Invalid {parameter_name}: {value} (valid range: {min}-{max})")]
    InvalidParameter {
        parameter_name: String,
        value: String,
        min: String,
        max: String,
    },

    /// SVG rendering error (parsing or rasterization failure)
    ///
    /// This error is returned when SVG loading fails due to:
    /// - Malformed or invalid SVG syntax
    /// - Unsupported SVG features (complex filters, animations)
    /// - Rasterization failures (pixmap creation, rendering errors)
    /// - Font loading issues for text-heavy SVGs
    ///
    /// The error message includes descriptive context to aid debugging.
    #[cfg(feature = "svg")]
    #[error("SVG rendering error: {0}")]
    SvgError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_dimensions_message_includes_context() {
        let err = DotmaxError::InvalidDimensions {
            width: 0,
            height: 10,
        };
        let msg = format!("{err}");
        assert!(msg.contains('0'));
        assert!(msg.contains("10"));
        assert!(msg.contains("width"));
        assert!(msg.contains("height"));
    }

    #[test]
    fn test_out_of_bounds_message_includes_all_context() {
        let err = DotmaxError::OutOfBounds {
            x: 100,
            y: 50,
            width: 80,
            height: 24,
        };
        let msg = format!("{err}");
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.contains("80"));
        assert!(msg.contains("24"));
    }

    #[test]
    fn test_invalid_dot_index_message_includes_index() {
        let err = DotmaxError::InvalidDotIndex { index: 10 };
        let msg = format!("{err}");
        assert!(msg.contains("10"));
        assert!(msg.contains("0-7"));
    }

    #[test]
    fn test_unicode_conversion_message_includes_coordinates() {
        let err = DotmaxError::UnicodeConversion { x: 15, y: 20 };
        let msg = format!("{err}");
        assert!(msg.contains("15"));
        assert!(msg.contains("20"));
    }

    #[test]
    fn test_terminal_backend_message() {
        let err = DotmaxError::TerminalBackend("Test error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("Test error"));
        assert!(msg.contains("Terminal backend error"));
    }

    #[test]
    fn test_io_error_automatic_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test file");
        let dotmax_err: DotmaxError = io_err.into();
        assert!(matches!(dotmax_err, DotmaxError::Terminal(_)));
    }

    #[test]
    fn test_io_error_preserves_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let dotmax_err: DotmaxError = io_err.into();

        match dotmax_err {
            DotmaxError::Terminal(inner) => {
                assert_eq!(inner.kind(), std::io::ErrorKind::PermissionDenied);
                assert!(inner.to_string().contains("access denied"));
            }
            _ => panic!("Expected Terminal variant"),
        }
    }

    #[cfg(feature = "image")]
    #[test]
    fn test_image_load_error_includes_path_and_source() {
        use std::path::PathBuf;
        let err = DotmaxError::ImageLoad {
            path: PathBuf::from("/path/to/image.png"),
            source: image::ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            )),
        };
        let msg = format!("{err}");
        assert!(msg.contains("image.png"));
        assert!(msg.contains("Failed to load"));
    }

    #[cfg(feature = "image")]
    #[test]
    fn test_unsupported_format_error_includes_format() {
        let err = DotmaxError::UnsupportedFormat {
            format: "xyz".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("xyz"));
        assert!(msg.contains("Unsupported"));
    }

    #[cfg(feature = "image")]
    #[test]
    fn test_invalid_image_dimensions_includes_dimensions() {
        let err = DotmaxError::InvalidImageDimensions {
            width: 15_000,
            height: 20_000,
        };
        let msg = format!("{err}");
        assert!(msg.contains("15000") || msg.contains("15,000"));
        assert!(msg.contains("20000") || msg.contains("20,000"));
        assert!(msg.contains("10,000"));
    }

    #[cfg(feature = "image")]
    #[test]
    fn test_invalid_parameter_includes_all_context() {
        let err = DotmaxError::InvalidParameter {
            parameter_name: "brightness factor".to_string(),
            value: "3.5".to_string(),
            min: "0.0".to_string(),
            max: "2.0".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("brightness factor"));
        assert!(msg.contains("3.5"));
        assert!(msg.contains("0.0"));
        assert!(msg.contains("2.0"));
        assert!(msg.contains("Invalid"));
    }
}
