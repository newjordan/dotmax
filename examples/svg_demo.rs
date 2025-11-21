//! SVG rendering demonstration
//!
//! This example demonstrates the full SVG → braille pipeline:
//! 1. Load SVG from file path
//! 2. Rasterize to target dimensions

#![allow(clippy::uninlined_format_args)]
//! 3. Convert to grayscale
//! 4. Apply dithering for quality
//! 5. Map to braille grid
//! 6. Render to terminal
//!
//! Run with: `cargo run --example svg_demo --features image,svg`

#![cfg(all(feature = "image", feature = "svg"))]

use dotmax::image::{
    apply_dithering, load_svg_from_path, pixels_to_braille, to_grayscale, DitheringMethod,
};
use dotmax::BrailleGrid;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════╗");
    println!("║   dotmax SVG → Braille Demo             ║");
    println!("╚══════════════════════════════════════════╝\n");

    // Load and render simple circle SVG
    println!("1. Simple Circle SVG (simple_circle.svg):");
    println!("   Demonstrates basic shape rasterization\n");

    let circle_path = Path::new("tests/fixtures/svg/simple_circle.svg");
    render_svg_to_terminal(circle_path, 40, 20, DitheringMethod::None)?;

    println!("\n{}", "─".repeat(50));

    // Load and render logo SVG with Floyd-Steinberg dithering
    println!("\n2. Logo SVG with Gradients (logo.svg):");
    println!("   Demonstrates complex SVG with gradients and paths");
    println!("   Using Floyd-Steinberg dithering for quality\n");

    let logo_path = Path::new("tests/fixtures/svg/logo.svg");
    render_svg_to_terminal(logo_path, 50, 30, DitheringMethod::FloydSteinberg)?;

    println!("\n{}", "─".repeat(50));

    // Load and render text-heavy SVG
    println!("\n3. Text-Heavy SVG (text_heavy.svg):");
    println!("   Demonstrates text rendering with font fallback\n");

    let text_path = Path::new("tests/fixtures/svg/text_heavy.svg");
    render_svg_to_terminal(text_path, 75, 20, DitheringMethod::Bayer)?;

    println!("\n{}", "─".repeat(50));

    // Load and render gradient SVG
    println!("\n4. Gradient SVG (gradient.svg):");
    println!("   Demonstrates gradient rasterization");
    println!("   Using Atkinson dithering for smooth gradients\n");

    let gradient_path = Path::new("tests/fixtures/svg/gradient.svg");
    render_svg_to_terminal(gradient_path, 64, 32, DitheringMethod::Atkinson)?;

    println!("\n{}", "═".repeat(50));
    println!("\n✅ SVG rendering demo complete!");
    println!("\nKey Features Demonstrated:");
    println!("  • SVG loading and rasterization");
    println!("  • Aspect ratio preservation");
    println!("  • Gradient rendering");
    println!("  • Text with font fallback");
    println!("  • Multiple dithering algorithms");
    println!("  • Transparent background handling");

    Ok(())
}

/// Helper function to render SVG to terminal using braille
fn render_svg_to_terminal(
    svg_path: &Path,
    width: u32,
    height: u32,
    dither_method: DitheringMethod,
) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Load and rasterize SVG
    let img = load_svg_from_path(svg_path, width, height)?;
    println!(
        "   ✓ Loaded SVG: {} ({}×{} pixels)",
        svg_path.display(),
        width,
        height
    );

    // Step 2: Convert to grayscale
    let gray = to_grayscale(&img);
    println!("   ✓ Converted to grayscale");

    // Step 3: Apply dithering (if specified)
    let binary = if dither_method == DitheringMethod::None {
        // Use simple Otsu thresholding
        dotmax::image::auto_threshold(&img)
    } else {
        apply_dithering(&gray, dither_method)?
    };
    println!("   ✓ Applied {:?}", dither_method);

    // Step 4: Map to braille grid
    // Calculate grid dimensions from pixel dimensions
    let grid_width = ((binary.width + 1) / 2) as usize;
    let grid_height = ((binary.height + 3) / 4) as usize;

    let grid = pixels_to_braille(&binary, grid_width, grid_height)?;
    println!(
        "   ✓ Mapped to {}×{} braille grid\n",
        grid_width, grid_height
    );

    // Step 5: Render to terminal
    print_braille_grid(&grid);

    Ok(())
}

/// Print braille grid to terminal (simple output without terminal backend)
fn print_braille_grid(grid: &BrailleGrid) {
    let unicode_grid = grid.to_unicode_grid();

    for row in unicode_grid {
        for ch in row {
            print!("{}", ch);
        }
        println!();
    }
}
