//! Custom image rendering example - demonstrates full customization
//!
//! This example shows how to use the `ImageRenderer` builder pattern to fully
//! customize the image rendering pipeline with brightness adjustments, custom
//! dithering algorithms, and color modes.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example custom_image --features image
//! ```

use dotmax::image::{ColorMode, DitheringMethod, ImageRenderer};
use dotmax::TerminalRenderer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut renderer = TerminalRenderer::new()?;

    println!("=== Custom Image Rendering Example ===\n");

    // Example 1: Manual dimensions with brightness/contrast adjustments
    println!("1. Custom dimensions (60×30) with adjustments:");
    let mut builder1 = ImageRenderer::new()
        .load_from_path(Path::new("tests/fixtures/images/sample.png"))?
        .resize(60, 30, true)? // 60×30 cells, preserve aspect
        .brightness(1.2)? // 20% brighter
        .contrast(1.1)? // 10% more contrast
        .gamma(0.9)?; // Slight darkening via gamma
    let grid1 = builder1.render()?;
    renderer.render(&grid1)?;
    println!();

    // Example 2: Different dithering algorithm
    println!("2. Atkinson dithering (softer appearance):");
    let mut builder2 = ImageRenderer::new()
        .load_from_path(Path::new("tests/fixtures/images/sample.png"))?
        .resize_to_terminal()? // Auto terminal size
        .dithering(DitheringMethod::Atkinson);
    let grid2 = builder2.render()?;
    renderer.render(&grid2)?;
    println!();

    // Example 3: Manual threshold (no dithering)
    println!("3. Manual threshold at 128 (mid-point):");
    let mut builder3 = ImageRenderer::new()
        .load_from_path(Path::new("tests/fixtures/images/sample.png"))?
        .resize(50, 25, true)?
        .threshold(128); // Manual threshold, no dithering
    let grid3 = builder3.render()?;
    renderer.render(&grid3)?;
    println!();

    // Example 4: Color mode (TrueColor)
    println!("4. TrueColor mode (full RGB per braille cell):");
    let mut builder4 = ImageRenderer::new()
        .load_from_path(Path::new("tests/fixtures/images/sample.png"))?
        .resize(40, 20, true)?
        .color_mode(ColorMode::TrueColor);
    let grid4 = builder4.render()?;
    renderer.render(&grid4)?;
    println!();

    println!("=== All examples completed! ===");

    Ok(())
}
