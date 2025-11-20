//! Visual comparison of dithering algorithms
//!
//! This example demonstrates all three dithering methods side-by-side,
//! showing the quality and performance trade-offs of each algorithm.
//!
//! Usage:
//!   cargo run --example dither_comparison --features image

#![cfg(feature = "image")]

use dotmax::image::{
    apply_dithering, load_from_path, resize_to_terminal, to_grayscale, DitheringMethod,
};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Dotmax Dithering Algorithm Comparison ===\n");

    // Load and prepare image
    let image_path = Path::new("tests/fixtures/images/sample.png");
    println!("Loading image from: {:?}", image_path);

    let img = load_from_path(image_path)?;
    println!("Loaded: {}×{} pixels\n", img.width(), img.height());

    // Resize to terminal size
    println!("Resizing to terminal (80×24 cells → 160×96 pixels)...");
    let resized = resize_to_terminal(&img, 80, 24)?;
    println!(
        "Resized to: {}×{} pixels\n",
        resized.width(),
        resized.height()
    );

    // Convert to grayscale
    println!("Converting to grayscale...");
    let gray = to_grayscale(&resized);
    println!("Grayscale: {}×{} pixels\n", gray.width(), gray.height());

    // Test all dithering methods
    let methods = [
        (DitheringMethod::None, "None (Direct Threshold)"),
        (DitheringMethod::FloydSteinberg, "Floyd-Steinberg"),
        (DitheringMethod::Bayer, "Bayer Ordered"),
        (DitheringMethod::Atkinson, "Atkinson"),
    ];

    println!("=== Dithering Results ===\n");

    for (method, name) in &methods {
        println!("Testing: {}", name);

        // Time the operation
        let start = Instant::now();
        let binary = apply_dithering(&gray, *method)?;
        let duration = start.elapsed();

        // Count black/white pixels
        let total = binary.pixel_count();
        let black = binary.pixels.iter().filter(|&&p| p).count();
        let white = total - black;

        println!("  Time:         {:.2}ms", duration.as_secs_f64() * 1000.0);
        println!(
            "  Dimensions:   {}×{} ({} pixels)",
            binary.width, binary.height, total
        );
        println!(
            "  Black pixels: {} ({:.1}%)",
            black,
            100.0 * black as f64 / total as f64
        );
        println!(
            "  White pixels: {} ({:.1}%)",
            white,
            100.0 * white as f64 / total as f64
        );
        println!();
    }

    // Summary comparison
    println!("=== Algorithm Characteristics ===\n");
    println!("┌─────────────────┬──────────┬─────────┬────────────────────────┐");
    println!("│ Algorithm       │ Speed    │ Quality │ Best For               │");
    println!("├─────────────────┼──────────┼─────────┼────────────────────────┤");
    println!("│ Floyd-Steinberg │ Slower   │ Highest │ Photos, complex images │");
    println!("│ Bayer           │ Fastest  │ Good    │ Gradients, real-time   │");
    println!("│ Atkinson        │ Moderate │ Artistic│ Line art, illustrations│");
    println!("│ None (Threshold)│ Fast     │ Basic   │ Simple binary needs    │");
    println!("└─────────────────┴──────────┴─────────┴────────────────────────┘");

    println!("\n=== Quality Comparison ===\n");
    println!("Floyd-Steinberg:");
    println!("  ✓ Best for photographs and complex tonal ranges");
    println!("  ✓ Minimal visual artifacts");
    println!("  ✗ Slowest due to error diffusion to 4 neighbors\n");

    println!("Bayer Ordered:");
    println!("  ✓ Fastest (stateless, no error propagation)");
    println!("  ✓ Good for gradients and smooth transitions");
    println!("  ✗ Visible 8×8 pattern on uniform areas\n");

    println!("Atkinson:");
    println!("  ✓ Artistic, softer than Floyd-Steinberg");
    println!("  ✓ Original algorithm from Apple MacPaint (1984)");
    println!("  ✗ Diffuses only 75% of error (6/8), may lose some detail\n");

    println!("None (Direct Threshold):");
    println!("  ✓ Very fast");
    println!("  ✗ Poor quality on gradients and complex images");
    println!("  ✗ Use only when dithering is not needed\n");

    println!("=== Recommendations ===\n");
    println!("• For terminal image viewers: FloydSteinberg (best quality)");
    println!("• For real-time animations: Bayer (fastest)");
    println!("• For artistic/retro style: Atkinson (unique aesthetic)");
    println!("• For simple binary graphics: None (direct threshold)");

    println!("\n✓ Comparison complete!");
    println!("\nNext steps:");
    println!("  1. Run benchmarks: cargo bench --features image --bench dithering");
    println!(
        "  2. Try with your own images: cargo run --example dither_comparison --features image"
    );
    println!("  3. Experiment with different algorithms in your code");

    Ok(())
}
