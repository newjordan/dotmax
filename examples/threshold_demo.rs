//! Demonstrates grayscale conversion and thresholding operations
//!
//! This example shows:
//! - Grayscale conversion from color images
//! - Automatic threshold calculation using Otsu's method
//! - Manual threshold application
//! - Brightness/contrast/gamma adjustments
//!
//! Usage:
//!   cargo run --example `threshold_demo` --features image

#![cfg(feature = "image")]
#![allow(
    clippy::uninlined_format_args,
    clippy::too_many_lines,
    clippy::unnecessary_debug_formatting,
    clippy::cast_precision_loss
)]

use dotmax::image::{
    adjust_brightness, adjust_contrast, adjust_gamma, apply_threshold, auto_threshold,
    load_from_path, otsu_threshold, resize_to_terminal, to_grayscale,
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Dotmax Threshold Demo ===\n");

    // Load sample image
    let image_path = Path::new("tests/fixtures/images/sample.png");
    println!("Loading image from: {:?}", image_path);

    let img = load_from_path(image_path)?;
    println!("Loaded image: {}×{} pixels\n", img.width(), img.height());

    // Resize to terminal size (80×24 cells → 160×96 pixels)
    println!("Resizing to terminal dimensions (80×24 cells)...");
    let resized = resize_to_terminal(&img, 80, 24)?;
    println!(
        "Resized to: {}×{} pixels\n",
        resized.width(),
        resized.height()
    );

    // Convert to grayscale
    println!("Converting to grayscale...");
    let gray = to_grayscale(&resized);
    println!("Grayscale conversion complete\n");

    // Calculate Otsu threshold
    println!("Calculating optimal threshold using Otsu's method...");
    let otsu = otsu_threshold(&gray);
    println!("Otsu threshold: {}\n", otsu);

    // Apply threshold manually
    println!("Applying manual threshold (128)...");
    let binary_manual = apply_threshold(&gray, 128);
    println!(
        "Manual threshold result: {}×{} binary image ({} pixels)\n",
        binary_manual.width,
        binary_manual.height,
        binary_manual.pixel_count()
    );

    // Count black/white pixels
    let black_count = binary_manual.pixels.iter().filter(|&&p| p).count();
    let white_count = binary_manual.pixel_count() - black_count;
    println!(
        "  Black pixels: {} ({:.1}%)",
        black_count,
        100.0 * black_count as f64 / binary_manual.pixel_count() as f64
    );
    println!(
        "  White pixels: {} ({:.1}%)\n",
        white_count,
        100.0 * white_count as f64 / binary_manual.pixel_count() as f64
    );

    // Apply automatic thresholding
    println!("Applying automatic threshold (Otsu)...");
    let binary_auto = auto_threshold(&resized);
    let black_count_auto = binary_auto.pixels.iter().filter(|&&p| p).count();
    let white_count_auto = binary_auto.pixel_count() - black_count_auto;
    println!(
        "  Black pixels: {} ({:.1}%)",
        black_count_auto,
        100.0 * black_count_auto as f64 / binary_auto.pixel_count() as f64
    );
    println!(
        "  White pixels: {} ({:.1}%)\n",
        white_count_auto,
        100.0 * white_count_auto as f64 / binary_auto.pixel_count() as f64
    );

    // Demonstrate brightness adjustment
    println!("=== Image Adjustments ===\n");

    println!("Adjusting brightness (factor: 1.2)...");
    let bright = adjust_brightness(&gray, 1.2)?;
    let binary_bright = apply_threshold(&bright, 128);
    let black_bright = binary_bright.pixels.iter().filter(|&&p| p).count();
    println!(
        "  Black pixels after brightening: {} ({:.1}%)\n",
        black_bright,
        100.0 * black_bright as f64 / binary_bright.pixel_count() as f64
    );

    println!("Adjusting contrast (factor: 1.3)...");
    let contrasted = adjust_contrast(&gray, 1.3)?;
    let binary_contrast = apply_threshold(&contrasted, 128);
    let black_contrast = binary_contrast.pixels.iter().filter(|&&p| p).count();
    println!(
        "  Black pixels after contrast adjustment: {} ({:.1}%)\n",
        black_contrast,
        100.0 * black_contrast as f64 / binary_contrast.pixel_count() as f64
    );

    println!("Applying gamma correction (gamma: 0.8)...");
    let gamma_corrected = adjust_gamma(&gray, 0.8)?;
    let binary_gamma = apply_threshold(&gamma_corrected, 128);
    let black_gamma = binary_gamma.pixels.iter().filter(|&&p| p).count();
    println!(
        "  Black pixels after gamma correction: {} ({:.1}%)\n",
        black_gamma,
        100.0 * black_gamma as f64 / binary_gamma.pixel_count() as f64
    );

    // Demonstrate chained adjustments
    println!("=== Chained Adjustments ===\n");
    println!("Applying brightness + contrast + gamma...");

    let chained = adjust_brightness(&gray, 1.1)
        .and_then(|img| adjust_contrast(&img, 1.2))
        .and_then(|img| adjust_gamma(&img, 0.9))?;

    let binary_chained = apply_threshold(&chained, 128);
    let black_chained = binary_chained.pixels.iter().filter(|&&p| p).count();
    println!(
        "  Black pixels after chained adjustments: {} ({:.1}%)\n",
        black_chained,
        100.0 * black_chained as f64 / binary_chained.pixel_count() as f64
    );

    println!("=== Summary ===\n");
    println!("Comparison of threshold results:");
    println!(
        "  Manual (128):      {}% black",
        100.0 * black_count as f64 / binary_manual.pixel_count() as f64
    );
    println!(
        "  Automatic (Otsu):  {}% black",
        100.0 * black_count_auto as f64 / binary_auto.pixel_count() as f64
    );
    println!(
        "  With brightness:   {}% black",
        100.0 * black_bright as f64 / binary_bright.pixel_count() as f64
    );
    println!(
        "  With contrast:     {}% black",
        100.0 * black_contrast as f64 / binary_contrast.pixel_count() as f64
    );
    println!(
        "  With gamma:        {}% black",
        100.0 * black_gamma as f64 / binary_gamma.pixel_count() as f64
    );
    println!(
        "  Chained:           {}% black",
        100.0 * black_chained as f64 / binary_chained.pixel_count() as f64
    );

    println!("\n✓ Demo complete!");

    Ok(())
}
