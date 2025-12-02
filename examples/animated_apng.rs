//! Animated PNG (APNG) playback demonstration.
//!
//! This example shows how to use dotmax's APNG playback capabilities:
//!
//! 1. Using quick API: `quick::show_file("animation.png")`
//! 2. Using ApngPlayer directly for custom control
//! 3. Using MediaPlayer trait for polymorphic playback
//!
//! # Usage
//!
//! ```bash
//! # Play any APNG file
//! cargo run --features image --example animated_apng -- path/to/animation.png
//!
//! # Or use the test fixture
//! cargo run --features image --example animated_apng
//! ```
//!
//! Press any key to stop playback.

use dotmax::media::{detect_format, ApngPlayer, MediaContent, MediaFormat, MediaPlayer};
use dotmax::quick;
use dotmax::Result;
use std::env;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize logging (optional)
    let _ = tracing_subscriber::fmt::try_init();

    // Get file path from args or use test fixture
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        // Use test fixture
        "tests/fixtures/media/animated.png".to_string()
    };

    println!("dotmax APNG Playback Demo");
    println!("=========================\n");

    // Check if file exists
    if !std::path::Path::new(&path).exists() {
        eprintln!("File not found: {}", path);
        eprintln!("\nGenerate test fixtures first:");
        eprintln!("  cargo run --features image --example generate_apng_fixtures");
        return Ok(());
    }

    // Demonstrate format detection
    println!("1. Format Detection");
    println!("-------------------");
    let format = detect_format(&path)?;
    println!("   File: {}", path);
    println!("   Format: {}", format);

    match format {
        MediaFormat::AnimatedPng => {
            println!("   ✓ Detected as animated PNG (APNG)\n");
        }
        MediaFormat::StaticImage(_) => {
            println!("   ✓ Detected as static image\n");
            // For static images, just display once
            println!("   Displaying static image...\n");
            quick::show_file(&path)?;
            return Ok(());
        }
        _ => {
            println!("   ✗ Not an image file\n");
            return Ok(());
        }
    }

    // Demonstrate ApngPlayer API
    println!("2. ApngPlayer API Demo");
    println!("----------------------");
    let player = ApngPlayer::new(&path)?;
    println!("   Canvas size: {}x{}", player.canvas_width(), player.canvas_height());
    println!("   Frame count: {:?}", player.frame_count());
    println!(
        "   Loop count: {:?} (0 = infinite)",
        player.loop_count()
    );
    println!();

    // Demonstrate MediaContent API (polymorphic)
    println!("3. MediaContent API (Polymorphic)");
    println!("---------------------------------");
    let content = quick::load_file(&path)?;
    match &content {
        MediaContent::Static(_) => println!("   Loaded as: Static image"),
        MediaContent::Animated(player) => {
            println!("   Loaded as: Animated content");
            println!("   Frame count: {:?}", player.frame_count());
        }
    }
    println!();

    // Play the animation with manual timing display
    println!("4. Playing Animation");
    println!("--------------------");
    println!("   Press any key to stop playback.\n");

    // Small delay to let user read
    std::thread::sleep(Duration::from_secs(1));

    // Use quick::show_file for actual playback
    // This handles terminal setup/teardown and keypress detection
    quick::show_file(&path)?;

    println!("\n✓ Playback complete!");

    Ok(())
}
