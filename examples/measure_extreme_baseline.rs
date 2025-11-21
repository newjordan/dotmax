//! Measure baseline performance with truly extreme aspect ratio images
//!
//! This tests the 10000×100 images that trigger the 20+ second issue.
//!
//! Run with:
//! ```sh
//! cargo run --features image --example measure_extreme_baseline --release
//! ```

#![cfg(feature = "image")]

use dotmax::image::{load_from_path, resize_to_terminal};
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("=== Baseline Performance Test (Truly Extreme Ratios) ===\n");

    // Test 1: Extreme wide 10000×100 (100:1 ratio) - THE PROBLEM CASE
    println!("Test 1: Extreme wide 10000×100 (100:1 ratio) - Original problem case");
    test_image(
        "tests/fixtures/images/extreme_wide_10000x100.png",
        "10000×100",
    );

    // Test 2: Extreme tall 100×10000 (1:100 ratio)
    println!("\nTest 2: Extreme tall 100×10000 (1:100 ratio)");
    test_image(
        "tests/fixtures/images/extreme_tall_100x10000.png",
        "100×10000",
    );

    // Test 3: Very large square 4096×4096
    println!("\nTest 3: Very large square 4096×4096");
    test_image(
        "tests/fixtures/images/very_large_4096x4096.png",
        "4096×4096",
    );
}

fn test_image(path: &str, dimensions: &str) {
    let path = Path::new(path);

    // Load
    let start = Instant::now();
    let img = match load_from_path(path) {
        Ok(img) => img,
        Err(e) => {
            println!("  ❌ Failed to load: {}", e);
            return;
        }
    };
    let load_time = start.elapsed();

    // Resize (Lanczos3 - current implementation)
    let start = Instant::now();
    let _resized = match resize_to_terminal(&img, 80, 24) {
        Ok(r) => r,
        Err(e) => {
            println!("  ❌ Failed to resize: {}", e);
            return;
        }
    };
    let resize_time = start.elapsed();

    // Report
    println!("  Load:   {:?}", load_time);
    println!("  Resize: {:?} (Lanczos3)", resize_time);
    println!("  Total:  {:?}", load_time + resize_time);

    // Check against targets
    let total_secs = (load_time + resize_time).as_secs_f64();
    if dimensions.contains("10000") {
        if total_secs > 5.0 {
            println!("  ⚠️  EXCEEDS 5s target ({:.1}s)", total_secs);
        } else {
            println!("  ✅ Within 5s target");
        }
    }
}
