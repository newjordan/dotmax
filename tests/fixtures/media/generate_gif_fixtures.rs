//! Script to generate GIF test fixtures
//!
//! Run with: cargo run --example generate_gif_fixtures --features image

use gif::{Encoder, Frame, Repeat};
use std::borrow::Cow;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create fixtures directory if needed
    std::fs::create_dir_all("tests/fixtures/media")?;

    // Generate static GIF (single frame)
    create_static_gif()?;
    println!("✓ Created static.gif");

    // Generate animated GIF (infinite loop)
    create_animated_gif()?;
    println!("✓ Created animated.gif");

    // Generate finite loop GIF (loop=2)
    create_loop_twice_gif()?;
    println!("✓ Created loop_twice.gif");

    // Generate GIF with various disposal methods
    create_disposal_gif()?;
    println!("✓ Created disposal_modes.gif");

    println!("\nAll GIF fixtures created in tests/fixtures/media/");
    Ok(())
}

/// Creates a static single-frame GIF (10x10 red square)
fn create_static_gif() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/fixtures/media/static.gif";
    let file = File::create(path)?;

    let mut encoder = Encoder::new(file, 10, 10, &[])?;
    encoder.set_repeat(Repeat::Finite(0))?; // No looping

    // Create red frame
    let mut frame = Frame::default();
    frame.width = 10;
    frame.height = 10;

    // Create palette: [0] = red
    let palette = [255, 0, 0]; // RGB
    frame.palette = Some(palette.to_vec());

    // All pixels reference color 0 (red)
    frame.buffer = Cow::Owned(vec![0u8; 100]);

    encoder.write_frame(&frame)?;
    Ok(())
}

/// Creates an animated GIF with 4 frames (infinite loop, 100ms delay each)
fn create_animated_gif() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/fixtures/media/animated.gif";
    let file = File::create(path)?;

    // Global palette: red, green, blue, yellow
    let palette = [
        255, 0, 0, // red
        0, 255, 0, // green
        0, 0, 255, // blue
        255, 255, 0, // yellow
    ];

    let mut encoder = Encoder::new(file, 10, 10, &palette)?;
    encoder.set_repeat(Repeat::Infinite)?;

    // Create 4 frames, each with different color
    for color_idx in 0u8..4 {
        let mut frame = Frame::default();
        frame.width = 10;
        frame.height = 10;
        frame.delay = 10; // 100ms (delay is in 1/100th seconds)
        frame.buffer = Cow::Owned(vec![color_idx; 100]);
        encoder.write_frame(&frame)?;
    }

    Ok(())
}

/// Creates a GIF that loops exactly twice
fn create_loop_twice_gif() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/fixtures/media/loop_twice.gif";
    let file = File::create(path)?;

    // Global palette: white, black
    let palette = [255, 255, 255, 0, 0, 0];

    let mut encoder = Encoder::new(file, 10, 10, &palette)?;
    encoder.set_repeat(Repeat::Finite(2))?; // Loop 2 times

    // Create 2 frames: white then black
    for color_idx in 0u8..2 {
        let mut frame = Frame::default();
        frame.width = 10;
        frame.height = 10;
        frame.delay = 50; // 500ms
        frame.buffer = Cow::Owned(vec![color_idx; 100]);
        encoder.write_frame(&frame)?;
    }

    Ok(())
}

/// Creates a GIF demonstrating different disposal methods
fn create_disposal_gif() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/fixtures/media/disposal_modes.gif";
    let file = File::create(path)?;

    // Global palette: transparent, red, green, blue, white
    let palette = [
        0, 0, 0, // black (will be transparent)
        255, 0, 0, // red
        0, 255, 0, // green
        0, 0, 255, // blue
        255, 255, 255, // white
    ];

    let mut encoder = Encoder::new(file, 20, 20, &palette)?;
    encoder.set_repeat(Repeat::Infinite)?;

    // Frame 1: Full red frame, disposal=None
    let mut frame1 = Frame::default();
    frame1.width = 20;
    frame1.height = 20;
    frame1.delay = 50;
    frame1.dispose = gif::DisposalMethod::Keep;
    frame1.buffer = Cow::Owned(vec![1u8; 400]); // red
    encoder.write_frame(&frame1)?;

    // Frame 2: Small green square (partial), disposal=Background
    let mut frame2 = Frame::default();
    frame2.width = 10;
    frame2.height = 10;
    frame2.left = 5;
    frame2.top = 5;
    frame2.delay = 50;
    frame2.dispose = gif::DisposalMethod::Background;
    frame2.buffer = Cow::Owned(vec![2u8; 100]); // green
    encoder.write_frame(&frame2)?;

    // Frame 3: Blue square, disposal=Previous
    let mut frame3 = Frame::default();
    frame3.width = 10;
    frame3.height = 10;
    frame3.left = 5;
    frame3.top = 5;
    frame3.delay = 50;
    frame3.dispose = gif::DisposalMethod::Previous;
    frame3.buffer = Cow::Owned(vec![3u8; 100]); // blue
    encoder.write_frame(&frame3)?;

    Ok(())
}
