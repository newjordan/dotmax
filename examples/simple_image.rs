//! Simple image rendering example with automatic resize handling
//!
//! This example demonstrates how to render an image to braille with automatic
//! re-rendering when the terminal window is resized. The image will scale
//! appropriately to fit the new terminal dimensions while maintaining aspect ratio.
//!
//! # Controls
//!
//! - **Q or Esc**: Quit
//! - Terminal resize will automatically trigger re-render
//!
//! # Usage
//!
//! ```bash
//! cargo run --example simple_image --features image
//! ```

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use dotmax::image::render_image_simple;
use dotmax::TerminalRenderer;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the image we'll render
    let image_path = Path::new("tests/fixtures/images/sample.png");

    // Enable raw mode for event handling
    terminal::enable_raw_mode()?;

    // Initial render
    render_image(image_path)?;

    // Event loop for resize handling and quit
    loop {
        // Poll for events with 100ms timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // Handle terminal resize events
                Event::Resize(width, height) => {
                    tracing::info!("Terminal resized to {}x{}", width, height);
                    render_image(image_path)?;
                }
                // Handle quit keys (Q or Esc)
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q' | 'Q') | KeyCode::Esc,
                    ..
                }) => {
                    break;
                }
                // Ignore other events
                _ => {}
            }
        }
    }

    // Cleanup: disable raw mode and clear screen
    terminal::disable_raw_mode()?;
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    println!("Simple image viewer closed.");

    Ok(())
}

/// Render the image to the terminal
///
/// This function clears the screen, loads the image, resizes it to fit the
/// terminal dimensions, and renders it using the default settings
/// (Floyd-Steinberg dithering, Monochrome mode, auto Otsu threshold).
fn render_image(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Clear screen before rendering
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    // Load and render image with defaults (auto-resizes to terminal)
    let grid = render_image_simple(path)?;

    // Display in terminal
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    // Display help text at bottom
    println!("\nPress Q or Esc to quit. Resize window to see automatic re-render.");

    // Flush output to ensure everything is displayed
    io::stdout().flush()?;

    Ok(())
}
