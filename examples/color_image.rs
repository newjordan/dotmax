//! Color image rendering example
//!
//! Demonstrates the three color modes for image rendering:
//! - Monochrome: Black/white only (default, backward compatible)
//! - Grayscale: 256 shades using ANSI 256-color palette
//! - `TrueColor`: Full RGB color per braille cell (24-bit)
//!
//! Usage:
//! ```bash
//! cargo run --example color_image --features image

#![allow(clippy::uninlined_format_args, clippy::too_many_lines)]
//! ```
//!
//! This example loads a sample image and renders it in all three modes,
//! demonstrating the visual differences between color rendering strategies.

use dotmax::image::{load_from_path, render_image_with_color, ColorMode, DitheringMethod};
use dotmax::TerminalRenderer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Color Mode Image Rendering Demo ===\n");

    // Load sample image
    let img_path = Path::new("tests/fixtures/images/sample.png");
    println!("Loading image: {}", img_path.display());

    let img = load_from_path(img_path)?;
    println!(
        "Image loaded: {}×{} pixels\n",
        img.width(),
        img.height()
    );

    // Initialize terminal renderer
    let mut renderer = TerminalRenderer::new()?;

    // =========================================================================
    // Mode 1: Monochrome (Default)
    // =========================================================================
    println!("---\n");
    println!("1. MONOCHROME MODE (Black/White Only)");
    println!("   - Default mode, backward compatible");
    println!("   - No color information stored");
    println!("   - Fastest rendering (no color extraction overhead)\n");

    let grid_mono = render_image_with_color(
        &img, ColorMode::Monochrome, 80, 24,
        DitheringMethod::FloydSteinberg, None, 1.0, 1.0, 1.0,
    )?;
    println!(
        "   Rendered {}×{} braille cells",
        grid_mono.width(),
        grid_mono.height()
    );

    // Render to terminal
    renderer.render(&grid_mono)?;
    println!("\n");

    // =========================================================================
    // Mode 2: Grayscale (ANSI 256-color)
    // =========================================================================
    println!("---\n");
    println!("2. GRAYSCALE MODE (256 Shades)");
    println!("   - Uses ANSI 256-color palette");
    println!("   - Converts RGB → luminance (BT.709 formula)");
    println!("   - Maps intensity to grayscale ramp (ANSI 232-255)");
    println!("   - Wide terminal compatibility (95%+)\n");

    let grid_gray = render_image_with_color(
        &img, ColorMode::Grayscale, 80, 24,
        DitheringMethod::FloydSteinberg, None, 1.0, 1.0, 1.0,
    )?;
    println!(
        "   Rendered {}×{} braille cells with grayscale",
        grid_gray.width(),
        grid_gray.height()
    );

    // Count cells with color
    let mut color_count = 0;
    for y in 0..grid_gray.height() {
        for x in 0..grid_gray.width() {
            if grid_gray.get_color(x, y).is_some() {
                color_count += 1;
            }
        }
    }
    println!("   {} cells have color data", color_count);

    // Render to terminal
    renderer.render(&grid_gray)?;
    println!("\n");

    // =========================================================================
    // Mode 3: TrueColor (24-bit RGB)
    // =========================================================================
    println!("---\n");
    println!("3. TRUECOLOR MODE (Full RGB)");
    println!("   - Preserves full RGB values per cell");
    println!("   - Renders with ANSI 24-bit color codes");
    println!("   - High-fidelity color reproduction");
    println!("   - Requires modern terminal (80%+ support)\n");

    let grid_true = render_image_with_color(
        &img, ColorMode::TrueColor, 80, 24,
        DitheringMethod::FloydSteinberg, None, 1.0, 1.0, 1.0,
    )?;
    println!(
        "   Rendered {}×{} braille cells with truecolor",
        grid_true.width(),
        grid_true.height()
    );

    // Verify every cell has color in TrueColor mode
    let mut color_count_true = 0;
    for y in 0..grid_true.height() {
        for x in 0..grid_true.width() {
            if grid_true.get_color(x, y).is_some() {
                color_count_true += 1;
            }
        }
    }
    let total_cells = grid_true.width() * grid_true.height();
    println!(
        "   {} / {} cells have color data ({}%)",
        color_count_true,
        total_cells,
        (color_count_true * 100) / total_cells
    );

    // Render to terminal
    renderer.render(&grid_true)?;
    println!("\n");

    // =========================================================================
    // Comparison Summary
    // =========================================================================
    println!("---\n");
    println!("=== COMPARISON SUMMARY ===\n");
    println!(
        "Grid Dimensions: {}×{} cells (all modes consistent)",
        grid_mono.width(),
        grid_mono.height()
    );
    println!("\nColor Mode         | Color Data | Terminal Compatibility");
    println!("-------------------|------------|----------------------");
    println!("Monochrome         | No         | 100% (universal)");
    println!("Grayscale (256)    | Yes (24)   | 95%+ (modern)");
    println!("TrueColor (24-bit) | Yes (RGB)  | 80%+ (latest)");
    println!("\n=== Demo Complete ===\n");

    Ok(())
}
