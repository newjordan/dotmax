//! Simple image rendering example - demonstrates <10 line usage
//!
//! This example shows the minimal code needed to render an image to braille
//! using dotmax's one-liner convenience function.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example simple_image --features image
//! ```

use dotmax::image::render_image_simple;
use dotmax::TerminalRenderer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One-liner: load, resize to terminal, render with defaults
    // Uses Floyd-Steinberg dithering, Monochrome mode, auto Otsu threshold
    let grid = render_image_simple(Path::new("tests/fixtures/images/sample.png"))?;

    // Display in terminal
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    Ok(())
}
