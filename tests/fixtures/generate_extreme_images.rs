//! Helper program to generate extreme aspect ratio test images
//! Run with: cargo run --release --example generate_extreme_images --features image

use image::{ImageBuffer, Rgb};

fn main() {
    // Create extreme wide image: 10000×100
    println!("Generating extreme_wide_10000x100.png...");
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
    println!("Generating extreme_tall_100x10000.png...");
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
    println!("Generating very_large_4096x4096.png...");
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
}
