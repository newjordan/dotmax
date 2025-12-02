//! Generator for APNG test fixtures
//!
//! Run this to generate test APNG files:
//! ```bash
//! cargo run --features image --example generate_apng_fixtures
//! ```

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    let fixtures_dir = Path::new("tests/fixtures/media");
    std::fs::create_dir_all(fixtures_dir).unwrap();

    // Generate animated.png - 3 frames, infinite loop
    generate_animated_apng(
        &fixtures_dir.join("animated.png"),
        10,
        10,
        3,
        0, // infinite loop
    );
    println!("Generated: animated.png (3 frames, infinite loop)");

    // Generate static.png - single frame (not APNG)
    generate_static_png(&fixtures_dir.join("static_png.png"), 10, 10);
    println!("Generated: static_png.png (single frame, no animation)");

    // Generate loop_twice.png - 2 frames, loop 2 times
    generate_animated_apng(&fixtures_dir.join("loop_twice.png"), 10, 10, 2, 2);
    println!("Generated: loop_twice.png (2 frames, loop 2x)");

    println!("\nAll APNG fixtures generated successfully!");
    println!("Files are in: tests/fixtures/media/");
}

fn generate_animated_apng(path: &Path, width: u32, height: u32, num_frames: u32, num_plays: u32) {
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    // Set animation parameters
    encoder.set_animated(num_frames, num_plays).unwrap();
    encoder.set_frame_delay(10, 100).unwrap(); // 10/100 = 0.1s = 100ms

    let mut writer = encoder.write_header().unwrap();

    // Generate frames with different colors
    let colors: [(u8, u8, u8); 3] = [
        (255, 0, 0),   // Red
        (0, 255, 0),   // Green
        (0, 0, 255),   // Blue
    ];

    for frame_idx in 0..num_frames {
        let color = colors[frame_idx as usize % colors.len()];
        let frame_data: Vec<u8> = (0..width * height)
            .flat_map(|_| [color.0, color.1, color.2, 255])
            .collect();

        writer.write_image_data(&frame_data).unwrap();
    }

    writer.finish().unwrap();
}

fn generate_static_png(path: &Path, width: u32, height: u32) {
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    // Single gray frame
    let frame_data: Vec<u8> = (0..width * height)
        .flat_map(|_| [128, 128, 128, 255])
        .collect();

    writer.write_image_data(&frame_data).unwrap();
    writer.finish().unwrap();
}
