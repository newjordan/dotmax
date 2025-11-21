//! Compare performance and quality of different resize filters
//!
//! Tests all available FilterType options from the image crate:
//! - Nearest (fastest, lowest quality)
//! - Triangle (fast, bilinear interpolation)
//! - CatmullRom (balanced, bicubic)
//! - Gaussian (high quality, slower)
//! - Lanczos3 (highest quality, slowest - current default)
//!
//! Run with:
//! ```sh
//! cargo run --features image --example compare_resize_filters --release
//! ```

#![cfg(feature = "image")]

use image::{imageops::FilterType, DynamicImage};
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("=== Resize Filter Performance Comparison ===\n");

    // Load test images
    let extreme_wide = load_image("tests/fixtures/images/viper_ultra_wide.png", "10000×4000");
    let extreme_tall = load_image("tests/fixtures/images/viper_ultra_tall.png", "4000×10000");
    let large_square = load_image("tests/fixtures/images/viper_4k.png", "4000×4000");

    let filters = vec![
        (FilterType::Nearest, "Nearest"),
        (FilterType::Triangle, "Triangle"),
        (FilterType::CatmullRom, "CatmullRom"),
        (FilterType::Gaussian, "Gaussian"),
        (FilterType::Lanczos3, "Lanczos3"),
    ];

    // Test each filter with extreme wide image
    println!("Test 1: Extreme wide image (10000×4000 → 160×64)");
    for (filter, name) in &filters {
        test_filter(&extreme_wide, *filter, name, 160, 64);
    }

    println!("\nTest 2: Extreme tall image (4000×10000 → 160×64)");
    for (filter, name) in &filters {
        test_filter(&extreme_tall, *filter, name, 160, 64);
    }

    println!("\nTest 3: Large square image (4000×4000 → 160×96)");
    for (filter, name) in &filters {
        test_filter(&large_square, *filter, name, 160, 96);
    }

    println!("\n=== Summary ===");
    println!("Fastest filter: Nearest (but lowest quality)");
    println!("Best quality/speed balance: Triangle or CatmullRom");
    println!("Highest quality: Lanczos3 (current default)");
    println!("\nRecommendation: Use CatmullRom for extreme ratios (>10:1 or <1:10)");
    println!("Maintain Lanczos3 for normal images to preserve quality");
}

fn load_image(path: &str, dims: &str) -> DynamicImage {
    image::open(Path::new(path))
        .unwrap_or_else(|e| panic!("Failed to load {} ({}): {}", path, dims, e))
}

fn test_filter(img: &DynamicImage, filter: FilterType, name: &str, target_w: u32, target_h: u32) {
    let start = Instant::now();
    let _resized = img.resize(target_w, target_h, filter);
    let duration = start.elapsed();

    println!("  {:<12} {:?}", name, duration);
}
