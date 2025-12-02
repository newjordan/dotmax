//! Universal Media Display Example
//!
//! Story 9.1: Demonstrates `show_file()` and `load_file()` functions
//! that automatically detect and handle any supported media format.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example universal_media --features image -- <path>
//! cargo run --example universal_media --features image,svg -- diagram.svg
//! ```
//!
//! # Format Detection
//!
//! The media module uses magic byte detection to identify file formats:
//! - PNG: 89 50 4E 47 (0x89PNG)
//! - JPEG: FF D8 FF
//! - GIF: 47 49 46 38 (GIF8)
//! - BMP: 42 4D (BM)
//! - WebP: RIFF....WEBP
//! - TIFF: II*\0 (little-endian) or MM\0* (big-endian)
//! - SVG: <?xml or <svg
//! - MP4: ftyp at offset 4
//! - MKV/WebM: 1A 45 DF A3 (EBML)
//! - AVI: RIFF....AVI
//!
//! If magic bytes are inconclusive, the file extension is used as a fallback.

use dotmax::media::{detect_format, detect_format_from_bytes, MediaContent, MediaFormat};
use dotmax::quick;
use std::env;
use std::io::Read;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        demonstrate_format_detection();
        return Ok(());
    }

    let path = Path::new(&args[1]);

    // Demonstrate format detection first
    println!("Format Detection Demo");
    println!("=====================\n");

    // 1. Detect format from path (reads magic bytes)
    println!("Detecting format for: {}", path.display());
    let format = detect_format(path)?;
    println!("Detected format: {}\n", format);

    // 2. Show format details
    match &format {
        MediaFormat::StaticImage(img_fmt) => {
            println!("Category: Static Image");
            println!("Image format: {}", img_fmt);
            println!("Renderer: ImageRenderer (existing pipeline)");
        }
        MediaFormat::Svg => {
            println!("Category: Vector Graphics");
            println!("Renderer: SVG rasterizer (requires 'svg' feature)");
        }
        MediaFormat::AnimatedGif => {
            println!("Category: Animated Image");
            println!("Renderer: GifPlayer (coming in Story 9.2)");
        }
        MediaFormat::AnimatedPng => {
            println!("Category: Animated Image");
            println!("Renderer: ApngPlayer (coming in Story 9.3)");
        }
        MediaFormat::Video(codec) => {
            println!("Category: Video");
            println!("Codec: {}", codec);
            println!("Renderer: VideoPlayer (coming in Story 9.4)");
        }
        MediaFormat::Unknown => {
            println!("Category: Unknown");
            println!("This format is not supported.");
        }
    }

    println!();

    // 3. Load file into MediaContent
    println!("Loading file into MediaContent...");
    match quick::load_file(path) {
        Ok(content) => {
            match content {
                MediaContent::Static(grid) => {
                    println!("Loaded as Static content: {}x{} grid", grid.width(), grid.height());
                    println!("\nPress any key to display...");

                    // Read a keypress
                    let mut stdin = std::io::stdin();
                    let _ = stdin.read(&mut [0u8]).unwrap();

                    // Display it
                    quick::show(&grid)?;
                }
                MediaContent::Animated(player) => {
                    println!(
                        "Loaded as Animated content: {:?} frames",
                        player.frame_count()
                    );
                    println!("Animated playback coming in Stories 9.2-9.4");
                }
            }
        }
        Err(e) => {
            println!("Could not load file: {}", e);
            println!("\nThis is expected for formats not yet implemented.");
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Universal Media Display Example");
    println!("================================\n");
    println!("Usage: cargo run --example universal_media --features image -- <path>\n");
    println!("Supported formats:");
    println!("  Static images: PNG, JPEG, GIF, BMP, WebP, TIFF");
    println!("  Vector: SVG (requires --features image,svg)");
    println!("  Animated: GIF, APNG (coming soon)");
    println!("  Video: MP4, MKV, AVI, WebM (coming soon)\n");
}

fn demonstrate_format_detection() {
    println!("Format Detection Demo (no file provided)");
    println!("========================================\n");
    println!("The detect_format_from_bytes() function identifies formats from magic bytes:\n");

    // Demonstrate magic byte detection
    let test_cases = [
        (
            "PNG",
            vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        ),
        ("JPEG", vec![0xFF, 0xD8, 0xFF, 0xE0]),
        ("GIF89a", vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61]),
        ("BMP", vec![0x42, 0x4D]),
        (
            "WebP",
            vec![
                0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
            ],
        ),
        ("TIFF (LE)", vec![0x49, 0x49, 0x2A, 0x00]),
        ("TIFF (BE)", vec![0x4D, 0x4D, 0x00, 0x2A]),
        (
            "MP4",
            vec![
                0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x6D,
            ],
        ),
        ("MKV/WebM", vec![0x1A, 0x45, 0xDF, 0xA3]),
        (
            "AVI",
            vec![
                0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x41, 0x56, 0x49, 0x20,
            ],
        ),
        ("SVG (xml)", b"<?xml version".to_vec()),
        ("SVG (direct)", b"<svg xmlns".to_vec()),
        ("Unknown", vec![0x00, 0x00, 0x00, 0x00]),
    ];

    for (name, bytes) in test_cases {
        let format = detect_format_from_bytes(&bytes);
        let hex: String = bytes.iter().take(8).map(|b| format!("{:02X} ", b)).collect();
        println!("{:12} | {} | -> {}", name, hex.trim(), format);
    }

    println!("\n(Provide a file path to see full detection and display)");
}
