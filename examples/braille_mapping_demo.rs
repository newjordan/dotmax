//! Braille Mapping Demonstration
//!
//! This example demonstrates the complete image-to-braille rendering pipeline,
//! showing how images are processed and mapped to braille characters for terminal display.
//!
//! Usage:
//!   cargo run --example braille_mapping_demo --features image

#[cfg(feature = "image")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dotmax::image::{
        apply_dithering, auto_threshold, load_from_path, pixels_to_braille, resize_to_dimensions,
        to_grayscale, DitheringMethod,
    };
    use dotmax::render::TerminalRenderer;
    use std::path::Path;

    println!("=== Braille Mapping Demo ===\n");

    // Load the sample image
    let img_path = Path::new("tests/fixtures/images/sample.png");
    println!("Loading image from: {:?}", img_path);
    let img = load_from_path(img_path)?;
    println!(
        "Original dimensions: {}×{} pixels\n",
        img.width(),
        img.height()
    );

    // Resize to small terminal dimensions for demo (30×20 pixels = 15×5 cells)
    println!("Resizing to 30×20 pixels (15×5 braille cells)...");
    let resized = resize_to_dimensions(&img, 30, 20, true);

    // Convert to grayscale
    println!("Converting to grayscale...");
    let gray = to_grayscale(&resized);

    // Demo 1: Auto threshold (no dithering)
    println!("\n--- Rendering with Auto Threshold (No Dithering) ---");
    let binary_threshold = auto_threshold(&gray);
    let grid_threshold = pixels_to_braille(&binary_threshold, 15, 5)?;
    println!(
        "Grid dimensions: {}×{} cells\n",
        grid_threshold.width(),
        grid_threshold.height()
    );

    let renderer = TerminalRenderer::new()?;
    renderer.render(&grid_threshold)?;

    // Demo 2: Floyd-Steinberg dithering
    println!("\n--- Rendering with Floyd-Steinberg Dithering ---");
    let binary_fs = apply_dithering(&gray, DitheringMethod::FloydSteinberg)?;
    let grid_fs = pixels_to_braille(&binary_fs, 15, 5)?;
    renderer.render(&grid_fs)?;

    // Demo 3: Bayer dithering
    println!("\n--- Rendering with Bayer Dithering ---");
    let binary_bayer = apply_dithering(&gray, DitheringMethod::Bayer)?;
    let grid_bayer = pixels_to_braille(&binary_bayer, 15, 5)?;
    renderer.render(&grid_bayer)?;

    // Demo 4: Atkinson dithering
    println!("\n--- Rendering with Atkinson Dithering ---");
    let binary_atkinson = apply_dithering(&gray, DitheringMethod::Atkinson)?;
    let grid_atkinson = pixels_to_braille(&binary_atkinson, 15, 5)?;
    renderer.render(&grid_atkinson)?;

    println!("\n=== Pipeline Explanation ===");
    println!("1. Load image from file (PNG, JPG, GIF, etc.)");
    println!("2. Resize to terminal pixel dimensions (width×2, height×4)");
    println!("3. Convert to grayscale (if color)");
    println!("4. Apply dithering for better quality (optional)");
    println!("5. Map 2×4 pixel blocks to braille cells");
    println!("6. Render braille grid to terminal\n");

    println!("Each braille cell contains 8 dots arranged in a 2×4 matrix:");
    println!("┌─┬─┐");
    println!("│1│4│  Dots 1-8 map to Unicode braille");
    println!("├─┼─┤  characters U+2800 to U+28FF");
    println!("│2│5│");
    println!("├─┼─┤  Black pixel → dot ON");
    println!("│3│6│  White pixel → dot OFF");
    println!("├─┼─┤");
    println!("│7│8│");
    println!("└─┴─┘\n");

    Ok(())
}

#[cfg(not(feature = "image"))]
fn main() {
    eprintln!("This example requires the 'image' feature to be enabled.");
    eprintln!("Run with: cargo run --example braille_mapping_demo --features image");
    std::process::exit(1);
}
