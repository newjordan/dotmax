//! dotmax - High-performance terminal braille rendering
//!
//! This library provides braille-based rendering capabilities for terminal applications,
//! enabling images, animations, and graphics in any terminal environment.
//!
//! # Getting Started
//!
//! Create a braille grid and manipulate individual dots:
//!
//! ```
//! use dotmax::BrailleGrid;
//!
//! // Create an 80×24 braille grid (typical terminal size)
//! let mut grid = BrailleGrid::new(80, 24).unwrap();
//!
//! // Set individual dots using pixel coordinates
//! // Grid is 80×24 cells = 160×96 dots (2×4 dots per cell)
//! grid.set_dot(0, 0).unwrap(); // Top-left dot of first cell
//! grid.set_dot(1, 0).unwrap(); // Top-right dot of first cell
//!
//! // Query dimensions
//! let (width, height) = grid.dimensions();
//! ```
//!
//! # Logging
//!
//! Dotmax uses the [`tracing`](https://docs.rs/tracing) crate for structured logging.
//! The library does **not** initialize a tracing subscriber - your application must
//! do this if you want to see log output.
//!
//! To enable logging in your application:
//!
//! ```no_run
//! use tracing_subscriber;
//!
//! // Initialize the tracing subscriber (do this once at startup)
//! tracing_subscriber::fmt()
//!     .with_max_level(tracing::Level::DEBUG)
//!     .init();
//!
//! // Now dotmax operations will emit trace events
//! use dotmax::BrailleGrid;
//! let grid = BrailleGrid::new(80, 24).unwrap();  // Logs: "Creating BrailleGrid: 80×24"
//! ```
//!
//! **Log Levels:**
//! - `ERROR`: Operation failures (e.g., out-of-bounds errors)
//! - `WARN`: Degraded operation (e.g., terminal lacks Unicode support)
//! - `INFO`: Major operations (grid creation, rendering)
//! - `DEBUG`: Detailed flow (resize, color changes)
//! - `TRACE`: Hot path internals (not used by default for performance)
//!
//! For more information, see the [tracing documentation](https://docs.rs/tracing).
//!
//! # License
//!
//! Licensed under either of:
//! - MIT license ([LICENSE-MIT](../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//! - Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//!
//! at your option.

// Core modules (Epic 2)
pub mod error;
pub mod grid;
pub mod render;

// Re-export public types for convenience
pub use error::DotmaxError;
pub use grid::{BrailleGrid, Color};
pub use render::{TerminalBackend, TerminalCapabilities, TerminalRenderer, TerminalType};

/// Convenience type alias for Results using `DotmaxError`
///
/// This allows writing `dotmax::Result<T>` instead of `Result<T, DotmaxError>`
/// in applications using this library.
pub type Result<T> = std::result::Result<T, DotmaxError>;

// Feature modules (Epic 3+): image, primitives, color, animation

#[cfg(test)]
mod tests {
    #[test]
    fn test_ci_basic() {
        assert_eq!(1 + 1, 2); // Intentional formatting issue
    }
}
