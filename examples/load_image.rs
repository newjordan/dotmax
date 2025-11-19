//! Example: Load images from file paths and byte buffers
//!
//! This example demonstrates dotmax's image loading capabilities.
//!
//! Run with:
//! ```bash
//! cargo run --example load_image --features image
//! ```

use dotmax::image::{load_from_bytes, load_from_path, supported_formats};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to see debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("═══════════════════════════════════════════════");
    println!("  dotmax - Image Loading Example");
    println!("═══════════════════════════════════════════════\n");

    // Display supported formats
    println!("📋 Supported Image Formats:");
    let formats = supported_formats();
    for (i, format) in formats.iter().enumerate() {
        print!("  {}", format.to_uppercase());
        if i < formats.len() - 1 {
            print!(", ");
        }
    }
    println!("\n");

    // Example 1: Load from file path
    println!("📂 Example 1: Loading from file path");
    println!("   Path: tests/fixtures/images/sample.png");

    let path = Path::new("tests/fixtures/images/sample.png");
    match load_from_path(path) {
        Ok(img) => {
            println!("   ✓ Successfully loaded image!");
            println!("   - Dimensions: {}×{} pixels", img.width(), img.height());
            println!("   - Color type: {:?}", img.color());
        }
        Err(e) => {
            println!("   ✗ Failed to load: {}", e);
        }
    }

    println!();

    // Example 2: Load from byte buffer
    println!("🗂️  Example 2: Loading from byte buffer");
    println!("   (Reading file into memory first)");

    match std::fs::read(path) {
        Ok(bytes) => {
            println!("   - Read {} bytes from disk", bytes.len());

            match load_from_bytes(&bytes) {
                Ok(img) => {
                    println!("   ✓ Successfully loaded from bytes!");
                    println!("   - Dimensions: {}×{} pixels", img.width(), img.height());
                }
                Err(e) => {
                    println!("   ✗ Failed to load from bytes: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ✗ Failed to read file: {}", e);
        }
    }

    println!();

    // Example 3: Error handling
    println!("⚠️  Example 3: Error handling");
    println!("   Path: nonexistent.png (does not exist)");

    match load_from_path(Path::new("nonexistent.png")) {
        Ok(_) => {
            println!("   ✗ Unexpected success!");
        }
        Err(e) => {
            println!("   ✓ Error handled gracefully:");
            println!("   - Error: {}", e);
        }
    }

    println!();

    // Example 4: Corrupted file handling
    println!("🛡️  Example 4: Corrupted file handling");
    println!("   Path: tests/fixtures/images/corrupted.png");

    match load_from_path(Path::new("tests/fixtures/images/corrupted.png")) {
        Ok(_) => {
            println!("   ✗ Unexpected success on corrupted file!");
        }
        Err(e) => {
            println!("   ✓ Corrupted file detected:");
            println!("   - Error: {}", e);
        }
    }

    println!("\n═══════════════════════════════════════════════");
    println!("  Example completed successfully!");
    println!("═══════════════════════════════════════════════");

    Ok(())
}
