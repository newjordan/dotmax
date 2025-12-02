//! Generator for APNG test fixtures
//!
//! Run this to generate test APNG files:
//! ```
//! cargo run --example generate_apng_fixtures
//! ```
//!
//! This creates:
//! - animated.png - 3-frame animation (10x10), infinite loop, 100ms delay
//! - static.png - Single frame PNG (not an APNG)
//! - loop_twice.png - 2-frame animation with num_plays=2
//! - blend_modes.png - Test blend operations (Source/Over)
//! - dispose_modes.png - Test dispose operations (None/Background/Previous)

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
        100,
    );
    println!("Generated: animated.png");

    // Generate static.png - single frame (not APNG)
    generate_static_png(&fixtures_dir.join("static.png"), 10, 10);
    println!("Generated: static.png");

    // Generate loop_twice.png - 2 frames, loop 2 times
    generate_animated_apng(&fixtures_dir.join("loop_twice.png"), 10, 10, 2, 2, 100);
    println!("Generated: loop_twice.png");

    // Generate blend_modes.png - test BlendOp::Source and BlendOp::Over
    generate_blend_test(&fixtures_dir.join("blend_modes.png"));
    println!("Generated: blend_modes.png");

    // Generate dispose_modes.png - test DisposeOp
    generate_dispose_test(&fixtures_dir.join("dispose_modes.png"));
    println!("Generated: dispose_modes.png");

    println!("\nAll APNG fixtures generated successfully!");
}

fn generate_animated_apng(
    path: &Path,
    width: u32,
    height: u32,
    num_frames: u32,
    num_plays: u32,
    delay_ms: u16,
) {
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    // Set animation parameters
    encoder.set_animated(num_frames, num_plays).unwrap();
    encoder.set_frame_delay(delay_ms / 10, 100).unwrap(); // delay_num, delay_den

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
            .flat_map(|_| vec![color.0, color.1, color.2, 255])
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
        .flat_map(|_| vec![128, 128, 128, 255])
        .collect();

    writer.write_image_data(&frame_data).unwrap();
    writer.finish().unwrap();
}

fn generate_blend_test(path: &Path) {
    let width = 10u32;
    let height = 10u32;

    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_animated(2, 0).unwrap();
    encoder.set_frame_delay(1, 10).unwrap(); // 100ms

    let mut writer = encoder.write_header().unwrap();

    // Frame 1: Solid red
    let frame1: Vec<u8> = (0..width * height)
        .flat_map(|_| vec![255, 0, 0, 255])
        .collect();
    writer.write_image_data(&frame1).unwrap();

    // Frame 2: Semi-transparent blue (will blend with frame 1)
    let frame2: Vec<u8> = (0..width * height)
        .flat_map(|_| vec![0, 0, 255, 128]) // 50% alpha
        .collect();
    writer.write_image_data(&frame2).unwrap();

    writer.finish().unwrap();
}

fn generate_dispose_test(path: &Path) {
    let width = 10u32;
    let height = 10u32;

    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_animated(3, 0).unwrap();
    encoder.set_frame_delay(1, 10).unwrap(); // 100ms

    let mut writer = encoder.write_header().unwrap();

    // Frame 1: Red
    let frame1: Vec<u8> = (0..width * height)
        .flat_map(|_| vec![255, 0, 0, 255])
        .collect();
    writer.write_image_data(&frame1).unwrap();

    // Frame 2: Green
    let frame2: Vec<u8> = (0..width * height)
        .flat_map(|_| vec![0, 255, 0, 255])
        .collect();
    writer.write_image_data(&frame2).unwrap();

    // Frame 3: Blue
    let frame3: Vec<u8> = (0..width * height)
        .flat_map(|_| vec![0, 0, 255, 255])
        .collect();
    writer.write_image_data(&frame3).unwrap();

    writer.finish().unwrap();
}
