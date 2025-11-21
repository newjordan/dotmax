//! Compare performance across all image sizes
//!
//! Run with:
//! ```sh
//! cargo run --features image --example compare_all_sizes --release
//! ```

#![cfg(feature = "image")]

use dotmax::image::{load_from_path, resize_to_terminal};
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("=== Complete Image Size Performance Comparison ===\n");

    let tests = vec![
        (
            "Normal image",
            "tests/fixtures/images/viper3.png",
            "1024×1024",
            50,
        ),
        (
            "Extreme wide (synthetic)",
            "tests/fixtures/images/extreme_wide_10000x100.png",
            "10000×100",
            5000,
        ),
        (
            "Extreme tall (synthetic)",
            "tests/fixtures/images/extreme_tall_100x10000.png",
            "100×10000",
            5000,
        ),
        (
            "Very wide (photo)",
            "tests/fixtures/images/viper_ultra_wide.png",
            "10000×4000",
            5000,
        ),
        (
            "Very tall (photo)",
            "tests/fixtures/images/viper_ultra_tall.png",
            "4000×10000",
            5000,
        ),
        (
            "Large square (photo)",
            "tests/fixtures/images/viper_4k.png",
            "4000×4000",
            5000,
        ),
        (
            "Very large square (synthetic)",
            "tests/fixtures/images/very_large_4096x4096.png",
            "4096×4096",
            5000,
        ),
    ];

    for (name, path, dims, target_ms) in tests {
        println!("{}:", name);
        test_image(path, dims, target_ms);
        println!();
    }
}

fn test_image(path: &str, dims: &str, target_ms: u64) {
    let path = Path::new(path);

    let start = Instant::now();
    let img = match load_from_path(path) {
        Ok(img) => img,
        Err(e) => {
            println!("  ❌ Failed to load: {}", e);
            return;
        }
    };
    let load_time = start.elapsed();

    let start = Instant::now();
    let _resized = match resize_to_terminal(&img, 80, 24) {
        Ok(r) => r,
        Err(e) => {
            println!("  ❌ Failed to resize: {}", e);
            return;
        }
    };
    let resize_time = start.elapsed();

    let total_time = load_time + resize_time;
    let total_ms = total_time.as_millis();

    println!("  Dimensions: {}", dims);
    println!("  Load:       {:?}", load_time);
    println!("  Resize:     {:?}", resize_time);
    println!("  Total:      {:?} ({}ms)", total_time, total_ms);

    if total_ms as u64 > target_ms {
        println!("  ⚠️  EXCEEDS {}ms target", target_ms);
    } else {
        println!("  ✅ Within {}ms target", target_ms);
    }
}
