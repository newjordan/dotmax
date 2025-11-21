//! Helper program to generate extreme aspect ratio test images for benchmarking
//!
//! Creates synthetic test images with extreme dimensions:
//! - 10000×100 (extreme wide)
//! - 100×10000 (extreme tall)
//! - 4096×4096 (very large square)
//!
//! Run with:
//! ```sh
//! cargo run --example generate_extreme_test_images --features image
//! ```

use image::{ImageBuffer, Rgb};
use std::fs;

fn main() {
    // Ensure output directory exists
    fs::create_dir_all("tests/fixtures/images").expect("Failed to create fixtures directory");

    // Create extreme wide image: 10000×100
    println!("Generating extreme_wide_10000x100.png (10000×100 = 1MB image)...");
    let img_wide = ImageBuffer::from_fn(10000, 100, |x, y| {
        // Create a gradient pattern for visual interest
        let r = ((x as f32 / 10000.0) * 255.0) as u8;
        let g = ((y as f32 / 100.0) * 255.0) as u8;
        let b = 128;
        Rgb([r, g, b])
    });
    img_wide
        .save("tests/fixtures/images/extreme_wide_10000x100.png")
        .expect("Failed to save extreme wide image");
    println!("✓ Created tests/fixtures/images/extreme_wide_10000x100.png");

    // Create extreme tall image: 100×10000
    println!("Generating extreme_tall_100x10000.png (100×10000 = 1MB image)...");
    let img_tall = ImageBuffer::from_fn(100, 10000, |x, y| {
        let r = ((x as f32 / 100.0) * 255.0) as u8;
        let g = ((y as f32 / 10000.0) * 255.0) as u8;
        let b = 128;
        Rgb([r, g, b])
    });
    img_tall
        .save("tests/fixtures/images/extreme_tall_100x10000.png")
        .expect("Failed to save extreme tall image");
    println!("✓ Created tests/fixtures/images/extreme_tall_100x10000.png");

    // Create very large square image: 4096×4096
    println!("Generating very_large_4096x4096.png (4096×4096 = 16MB image)...");
    let img_large = ImageBuffer::from_fn(4096, 4096, |x, y| {
        let r = ((x as f32 / 4096.0) * 255.0) as u8;
        let g = ((y as f32 / 4096.0) * 255.0) as u8;
        let b = 128;
        Rgb([r, g, b])
    });
    img_large
        .save("tests/fixtures/images/very_large_4096x4096.png")
        .expect("Failed to save large square image");
    println!("✓ Created tests/fixtures/images/very_large_4096x4096.png");

    println!("\n✅ All test images generated successfully!");
    println!("\nImages saved to tests/fixtures/images/:");
    println!("  - extreme_wide_10000x100.png (10000×100 pixels, ~300KB)");
    println!("  - extreme_tall_100x10000.png (100×10000 pixels, ~300KB)");
    println!("  - very_large_4096x4096.png (4096×4096 pixels, ~50MB)");
}
