//! Example: Image Resizing with Aspect Ratio Preservation
//!
//! Demonstrates how to use dotmax's image resizing functions:
//! 1. Automatic terminal-based resizing with `resize_to_terminal()`
//! 2. Manual resizing with `resize_to_dimensions()` and aspect ratio control
//!
//! Run with: cargo run --example `resize_image` --features image

#![cfg(feature = "image")]
#![allow(
    clippy::uninlined_format_args,
    clippy::too_many_lines,
    clippy::unnecessary_debug_formatting,
    clippy::cast_precision_loss
)]

use dotmax::image::{load_from_path, resize_to_dimensions, resize_to_terminal};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Image Resizing Example ===\n");

    // Load a sample image
    let image_path = Path::new("examples/tiger_1.png");

    if !image_path.exists() {
        eprintln!("Error: Image not found at {:?}", image_path);
        eprintln!("Please ensure the example image exists.");
        eprintln!("\nAlternatively, modify this example to point to your own image.");
        return Err("Image file not found".into());
    }

    println!("Loading image from: {:?}", image_path);
    let img = load_from_path(image_path)?;
    println!(
        "✓ Loaded successfully: {}×{} pixels\n",
        img.width(),
        img.height()
    );

    // Example 1: Resize to standard terminal dimensions (80×24)
    println!("--- Example 1: Resize to Standard Terminal (80×24) ---");
    let terminal_resized = resize_to_terminal(&img, 80, 24)?;
    println!(
        "Terminal dimensions: 80×24 cells → {}×{} pixels",
        80 * 2,
        24 * 4
    );
    println!(
        "Resized image: {}×{} pixels (aspect ratio preserved)",
        terminal_resized.width(),
        terminal_resized.height()
    );

    let aspect_terminal = terminal_resized.width() as f32 / terminal_resized.height() as f32;
    let aspect_original = img.width() as f32 / img.height() as f32;
    println!(
        "Aspect ratio: {:.3} (original: {:.3})\n",
        aspect_terminal, aspect_original
    );

    // Example 2: Resize to large terminal (200×50)
    println!("--- Example 2: Resize to Large Terminal (200×50) ---");
    let large_terminal = resize_to_terminal(&img, 200, 50)?;
    println!(
        "Terminal dimensions: 200×50 cells → {}×{} pixels",
        200 * 2,
        50 * 4
    );
    println!(
        "Resized image: {}×{} pixels",
        large_terminal.width(),
        large_terminal.height()
    );
    println!("Note: Upscale prevention may limit size to preserve quality\n");

    // Example 3: Manual resize with aspect ratio preservation
    println!("--- Example 3: Manual Resize (200×100, preserve aspect) ---");
    let manual_preserve = resize_to_dimensions(&img, 200, 100, true)?;
    println!(
        "Resized to: {}×{} pixels (fits within 200×100)",
        manual_preserve.width(),
        manual_preserve.height()
    );

    let aspect_manual = manual_preserve.width() as f32 / manual_preserve.height() as f32;
    println!(
        "Aspect ratio preserved: {:.3} (original: {:.3})\n",
        aspect_manual, aspect_original
    );

    // Example 4: Manual resize WITHOUT aspect ratio preservation (stretch)
    println!("--- Example 4: Manual Resize (200×100, stretch to fit) ---");
    let manual_stretch = resize_to_dimensions(&img, 200, 100, false)?;
    println!(
        "Resized to: {}×{} pixels (exact dimensions, may be distorted)",
        manual_stretch.width(),
        manual_stretch.height()
    );

    let aspect_stretch = manual_stretch.width() as f32 / manual_stretch.height() as f32;
    println!(
        "Aspect ratio: {:.3} (original: {:.3})",
        aspect_stretch, aspect_original
    );
    println!("Note: Image may appear distorted due to aspect ratio change\n");

    // Example 5: Resize very small image (demonstrates upscale prevention)
    println!("--- Example 5: Upscale Prevention ---");
    let small_img_path = Path::new("tests/fixtures/images/sample.png");
    if small_img_path.exists() {
        let small_img = load_from_path(small_img_path)?;
        println!(
            "Small image: {}×{} pixels",
            small_img.width(),
            small_img.height()
        );

        let upscaled = resize_to_terminal(&small_img, 80, 24)?;
        println!(
            "After resize to terminal: {}×{} pixels",
            upscaled.width(),
            upscaled.height()
        );

        if upscaled.width() == small_img.width() && upscaled.height() == small_img.height() {
            println!("✓ Upscale prevention activated (would exceed 2x quality threshold)");
        } else {
            println!("✓ Image resized (within acceptable upscale limits)");
        }
    } else {
        println!("(Small test image not found, skipping upscale prevention demo)");
    }

    println!("\n=== Resize Examples Complete ===");
    println!("\nKey Features:");
    println!("• Automatic terminal-based resizing accounts for braille cell dimensions (2×4 dots)");
    println!("• Aspect ratio preservation prevents distortion (can be disabled)");
    println!("• Upscale prevention maintains quality (max 2x upscaling)");
    println!("• Lanczos3 filter provides high-quality results");

    Ok(())
}
