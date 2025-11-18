//! Terminal rendering module
//!
//! Provides terminal rendering abstractions for braille grids using ratatui and crossterm.
//! Extracted and adapted from crabmusic/src/rendering/mod.rs
//!
//! # Examples
//!
//! ```no_run
//! use dotmax::{BrailleGrid, TerminalRenderer};
//!
//! let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
//! let mut grid = BrailleGrid::new(80, 24).expect("Failed to create grid");
//!
//! // Set some dots
//! grid.set_dot(10, 10).expect("Failed to set dot");
//!
//! // Render to terminal
//! renderer.render(&grid).expect("Failed to render");
//!
//! // Clean up
//! renderer.cleanup().expect("Failed to cleanup");
//! ```

use crate::error::DotmaxError;
use crate::grid::BrailleGrid;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::io::{self, Stdout};

// ============================================================================
// Error Handling - Extends DotmaxError for terminal operations
// ============================================================================

// DotmaxError::Terminal automatically converts from std::io::Error via #[from]
// in src/grid.rs:41

// ============================================================================
// Terminal Capabilities Detection
// ============================================================================

/// Terminal capabilities information
///
/// Provides information about what features the terminal supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCapabilities {
    /// Whether the terminal supports basic ANSI colors (16 colors)
    pub supports_color: bool,
    /// Whether the terminal supports true color (24-bit RGB)
    pub supports_truecolor: bool,
    /// Whether the terminal supports Unicode braille characters (U+2800-U+28FF)
    pub supports_unicode: bool,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        // Modern terminals typically support all these features
        // Crossterm handles platform differences automatically
        Self {
            supports_color: true,
            supports_truecolor: true,
            supports_unicode: true,
        }
    }
}

// ============================================================================
// Terminal Backend Trait - ADR 0004
// ============================================================================

/// Terminal backend abstraction
///
/// This trait abstracts terminal I/O operations to enable testing with mock
/// backends and reduce lock-in to specific terminal libraries.
///
/// Implements ADR 0004: Terminal Backend Abstraction via Trait
pub trait TerminalBackend {
    /// Get the current terminal size in character cells
    ///
    /// # Returns
    /// A tuple of (width, height) in characters
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if querying size fails
    fn size(&self) -> Result<(u16, u16), DotmaxError>;

    /// Render content to the terminal
    ///
    /// # Arguments
    /// * `content` - The content to render (typically braille characters)
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if rendering fails
    fn render(&mut self, content: &str) -> Result<(), DotmaxError>;

    /// Clear the terminal display
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if clearing fails
    fn clear(&mut self) -> Result<(), DotmaxError>;

    /// Get terminal capabilities
    ///
    /// # Returns
    /// Information about terminal features (color, unicode support)
    fn capabilities(&self) -> TerminalCapabilities;
}

// ============================================================================
// Terminal Renderer - Main Implementation
// ============================================================================

/// Terminal renderer for braille grids
///
/// Manages terminal state and renders `BrailleGrid` to the terminal display.
/// Uses ratatui and crossterm for cross-platform terminal manipulation.
///
/// Extracted and adapted from crabmusic/src/rendering/mod.rs:56-377
///
/// # Examples
///
/// ```no_run
/// use dotmax::{BrailleGrid, TerminalRenderer};
///
/// let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
/// let grid = BrailleGrid::new(80, 24).expect("Failed to create grid");
/// renderer.render(&grid).expect("Failed to render");
/// renderer.cleanup().expect("Failed to cleanup terminal");
/// ```
pub struct TerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    #[allow(dead_code)] // Reserved for future resize detection (Story 2.5)
    last_size: (u16, u16),
}

impl TerminalRenderer {
    /// Initialize a new terminal renderer
    ///
    /// Sets up the terminal in raw mode and alternate screen.
    /// Extracted from crabmusic/src/rendering/mod.rs:81-117
    ///
    /// # Returns
    /// A new `TerminalRenderer` instance
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if terminal setup fails
    /// Returns `DotmaxError::TerminalBackend` if terminal is too small (min 40×12)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dotmax::TerminalRenderer;
    ///
    /// let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
    /// ```
    pub fn new() -> Result<Self, DotmaxError> {
        let mut stdout = io::stdout();

        // Check terminal size (minimum 40×12 for basic functionality)
        // Extracted from crabmusic/src/rendering/mod.rs:85-93
        let (width, height) = crossterm::terminal::size()?;

        if width < 40 || height < 12 {
            return Err(DotmaxError::TerminalBackend(format!(
                "Terminal too small: {width}×{height} (minimum 40×12 required)"
            )));
        }

        // Enter raw mode
        // Extracted from crabmusic/src/rendering/mod.rs:95-96
        enable_raw_mode()?;

        // Enter alternate screen
        // Extracted from crabmusic/src/rendering/mod.rs:98-99
        execute!(stdout, EnterAlternateScreen)?;

        // Set up panic handler to restore terminal
        // Extracted from crabmusic/src/rendering/mod.rs:101-106
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }));

        // Create Ratatui terminal
        // Extracted from crabmusic/src/rendering/mod.rs:108-110
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            last_size: (width, height),
        })
    }

    /// Render a braille grid to the terminal
    ///
    /// Uses Ratatui's Frame API to efficiently render the grid.
    /// Ratatui handles differential rendering automatically.
    ///
    /// Extracted and adapted from crabmusic/src/rendering/mod.rs:202-254
    /// Adapted to use `BrailleGrid::to_unicode_grid()` from Story 2.2
    ///
    /// # Arguments
    /// * `grid` - The braille grid to render
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if rendering fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dotmax::{BrailleGrid, TerminalRenderer};
    ///
    /// let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
    /// let grid = BrailleGrid::new(80, 24).expect("Failed to create grid");
    /// renderer.render(&grid).expect("Failed to render");
    /// ```
    pub fn render(&mut self, grid: &BrailleGrid) -> Result<(), DotmaxError> {
        // Convert grid to Unicode characters using Story 2.2 functionality
        let unicode_grid = grid.to_unicode_grid();

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Convert Unicode grid to Ratatui Lines
            // Extracted from crabmusic/src/rendering/mod.rs:207-246
            // Simplified to remove color support (Story 2.6 will add colors)
            let lines: Vec<Line> = unicode_grid
                .iter()
                .map(|row| {
                    let text: String = row.iter().collect();
                    Line::from(Span::raw(text))
                })
                .collect();

            // Create paragraph widget
            let paragraph = Paragraph::new(lines);

            // Render to frame
            frame.render_widget(paragraph, area);
        })?;

        Ok(())
    }

    /// Clear the terminal display
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if clearing fails
    pub fn clear(&mut self) -> Result<(), DotmaxError> {
        self.terminal.clear()?;
        Ok(())
    }

    /// Get the current terminal dimensions
    ///
    /// Extracted from crabmusic/src/rendering/mod.rs:282-285
    ///
    /// # Returns
    /// A tuple of (width, height) in characters
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if querying terminal size fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dotmax::TerminalRenderer;
    ///
    /// let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
    /// let (width, height) = renderer.get_terminal_size().unwrap();
    /// assert!(width >= 40);
    /// assert!(height >= 12);
    /// ```
    pub fn get_terminal_size(&self) -> Result<(u16, u16), DotmaxError> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }

    /// Clean up and restore terminal state
    ///
    /// Should be called before the application exits to restore the terminal
    /// to its original state.
    ///
    /// Extracted from crabmusic/src/rendering/mod.rs:263-265
    ///
    /// # Errors
    /// Returns `DotmaxError::Terminal` if cleanup fails
    pub fn cleanup(&mut self) -> Result<(), DotmaxError> {
        Self::restore_terminal()
    }

    /// Restore terminal to original state (static for panic handler)
    ///
    /// Extracted from crabmusic/src/rendering/mod.rs:358-369
    fn restore_terminal() -> Result<(), DotmaxError> {
        let mut stdout = io::stdout();

        // Leave alternate screen
        execute!(stdout, LeaveAlternateScreen)?;

        // Disable raw mode
        disable_raw_mode()?;

        Ok(())
    }

    /// Get terminal capabilities
    ///
    /// Returns information about terminal features.
    /// Currently returns default capabilities (assumes modern terminal).
    ///
    /// # Returns
    /// Terminal capabilities information
    #[must_use]
    pub fn capabilities(&self) -> TerminalCapabilities {
        TerminalCapabilities::default()
    }
}

impl Drop for TerminalRenderer {
    /// Ensure terminal is cleaned up even if `cleanup()` wasn't called
    ///
    /// Extracted from crabmusic/src/rendering/mod.rs:372-377
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_capabilities_default() {
        let caps = TerminalCapabilities::default();
        assert!(caps.supports_color);
        assert!(caps.supports_truecolor);
        assert!(caps.supports_unicode);
    }

    #[test]
    #[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
    fn test_terminal_renderer_creation() {
        let renderer = TerminalRenderer::new();
        assert!(
            renderer.is_ok(),
            "Failed to create terminal renderer: {:?}",
            renderer.err()
        );
    }

    #[test]
    #[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
    fn test_terminal_dimensions() {
        let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
        let (width, height) = renderer.get_terminal_size().expect("Failed to get size");
        assert!(width >= 40, "Terminal width should be at least 40");
        assert!(height >= 12, "Terminal height should be at least 12");
    }

    #[test]
    #[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
    fn test_terminal_cleanup() {
        let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
        let result = renderer.cleanup();
        assert!(result.is_ok(), "Cleanup should succeed: {:?}", result.err());
    }

    #[test]
    #[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
    fn test_render_braille_grid() {
        let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
        let mut grid = BrailleGrid::new(10, 10).expect("Failed to create grid");

        // Set a test pattern
        grid.set_dot(5, 5).expect("Failed to set dot");
        grid.set_dot(6, 6).expect("Failed to set dot");

        // Render should succeed
        let result = renderer.render(&grid);
        assert!(result.is_ok(), "Render should succeed: {:?}", result.err());
    }

    #[test]
    #[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
    fn test_clear_terminal() {
        let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
        let result = renderer.clear();
        assert!(result.is_ok(), "Clear should succeed: {:?}", result.err());
    }

    #[test]
    #[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
    fn test_get_capabilities() {
        let renderer = TerminalRenderer::new().expect("Failed to initialize");
        let caps = renderer.capabilities();
        // Should return default capabilities
        assert!(caps.supports_unicode);
    }
}
