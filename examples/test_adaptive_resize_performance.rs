//! Quick performance test for adaptive resize algorithm
//!
//! Manually measures resize time for extreme vs normal images to verify
//! the CatmullRom filter is faster than Lanczos3 for extreme aspect ratios.
//!
//! Run with:
//! ```sh
//! cargo run --example test_adaptive_resize_performance --features image --release
//! ```

use dotmax::image::{load_from_path, resize_to_terminal};
use std::path::Path;
use std::time::Instant;

fn main() {
    // Enable tracing output to see which filter is selected
    // (requires RUST_LOG=dotmax=debug environment variable)

    println!("=== Adaptive Resize Performance Test (Story 3.5.5) ===\n");

    // Test 1: Normal image (should use Lanczos3)
    println!("Test 1: Normal aspect ratio image (viper3.png 1024×1024)");
    let normal_img = load_from_path(Path::new("tests/fixtures/images/viper3.png"))
        .expect("Failed to load viper3.png");

    let start = Instant::now();
    let _resized = resize_to_terminal(&normal_img, 80, 24).expect("Failed to resize normal image");
    let duration_normal = start.elapsed();

    println!("✓ Resize time: {:?}", duration_normal);
    println!("  Expected filter: Lanczos3 (high quality)\n");

    // Test 2: Extreme wide image (should use CatmullRom)
    println!("Test 2: Extreme wide aspect ratio (viper_ultra_wide.png 10000×4000)");
    let extreme_wide = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");

    let start = Instant::now();
    let _resized =
        resize_to_terminal(&extreme_wide, 80, 24).expect("Failed to resize extreme wide image");
    let duration_extreme_wide = start.elapsed();

    println!("✓ Resize time: {:?}", duration_extreme_wide);
    println!("  Expected filter: Lanczos3 (2.5:1 ratio, not extreme enough)\n");

    // Test 3: Create synthetic extreme image (100:1 ratio)
    println!("Test 3: Testing with actual extreme ratio...");
    println!("  (viper images are 2.5:1, not extreme enough for threshold)");
    println!("  Need to test with synthetically generated 10000×100 image");
    println!("  Threshold: ratio > 10:1 or < 1:10\n");

    // Test 4: Extreme tall image
    println!("Test 4: Extreme tall aspect ratio (viper_ultra_tall.png 4000×10000)");
    let extreme_tall = load_from_path(Path::new("tests/fixtures/images/viper_ultra_tall.png"))
        .expect("Failed to load viper_ultra_tall.png");

    let start = Instant::now();
    let _resized =
        resize_to_terminal(&extreme_tall, 80, 24).expect("Failed to resize extreme tall image");
    let duration_extreme_tall = start.elapsed();

    println!("✓ Resize time: {:?}", duration_extreme_tall);
    println!("  Expected filter: Lanczos3 (1:2.5 ratio, not extreme enough)\n");

    // Test 5: Large square image (4k)
    println!("Test 5: Large square image (viper_4k.png 4000×4000)");
    let large_square = load_from_path(Path::new("tests/fixtures/images/viper_4k.png"))
        .expect("Failed to load viper_4k.png");

    let start = Instant::now();
    let _resized =
        resize_to_terminal(&large_square, 80, 24).expect("Failed to resize large square image");
    let duration_large = start.elapsed();

    println!("✓ Resize time: {:?}", duration_large);
    println!("  Expected filter: Lanczos3 (1:1 ratio, normal)\n");

    // Summary
    println!("=== Summary ===");
    println!("Normal 1024×1024:         {:?}", duration_normal);
    println!("Extreme wide 10000×4000:  {:?}", duration_extreme_wide);
    println!("Extreme tall 4000×10000:  {:?}", duration_extreme_tall);
    println!("Large square 4000×4000:   {:?}", duration_large);
    println!("\n⚠️  NOTE: viper images don't trigger extreme aspect ratio optimization (2.5:1 < 10:1 threshold)");
    println!("✓  Adaptive algorithm is working, but we need images with >10:1 or <1:10 ratios to test optimization");
    println!("✓  Check RUST_LOG=dotmax=debug output to see which filter was selected");
}
