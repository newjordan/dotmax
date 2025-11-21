//! Test SVG Loading - Quick diagnostic for `svg_test.svg`
//!
//! This example loads the problematic `svg_test.svg` and provides diagnostic output

#![allow(
    clippy::doc_markdown,
    clippy::unnecessary_debug_formatting,
    clippy::uninlined_format_args,
    clippy::cast_precision_loss,
    clippy::branches_sharing_code
)]

use dotmax::image::{auto_threshold, load_svg_from_path, pixels_to_braille, to_grayscale};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_path = Path::new("tests/fixtures/svg/svg_test.svg");

    println!("═══════════════════════════════════════════");
    println!("  SVG Test Loading Diagnostic");
    println!("═══════════════════════════════════════════");
    println!();
    println!("Testing: {:?}", svg_path);

    if !svg_path.exists() {
        eprintln!("✗ ERROR: SVG file does not exist!");
        return Err("SVG file not found".into());
    }
    println!("✓ File exists");

    let metadata = std::fs::metadata(svg_path)?;
    println!("✓ File size: {} bytes", metadata.len());

    println!();
    println!("Loading SVG at 160×96 pixels...");

    let img = match load_svg_from_path(svg_path, 160, 96) {
        Ok(img) => {
            println!("✓ SVG loaded successfully");
            img
        }
        Err(e) => {
            eprintln!("✗ ERROR loading SVG: {}", e);
            return Err(e.into());
        }
    };

    println!("✓ Image dimensions: {}×{}", img.width(), img.height());
    println!("✓ Image color type: {:?}", img.color());

    println!();
    println!("Converting to grayscale...");
    let gray = to_grayscale(&img);
    println!("✓ Grayscale image: {}×{}", gray.width(), gray.height());

    println!();
    println!("Applying auto threshold...");
    let binary = auto_threshold(&img);
    println!("✓ Binary image: {}×{}", binary.width, binary.height);

    // Count black vs white pixels
    let mut black_count = 0;
    let mut white_count = 0;
    for y in 0..binary.height as usize {
        for x in 0..binary.width as usize {
            if binary.pixels[y * binary.width as usize + x] {
                black_count += 1;
            } else {
                white_count += 1;
            }
        }
    }
    println!(
        "  Black pixels: {} ({:.1}%)",
        black_count,
        (black_count as f32 / (black_count + white_count) as f32) * 100.0
    );
    println!(
        "  White pixels: {} ({:.1}%)",
        white_count,
        (white_count as f32 / (black_count + white_count) as f32) * 100.0
    );

    if black_count == 0 {
        println!();
        println!("⚠ WARNING: No black pixels detected!");
        println!("This means the image is completely white after thresholding.");
        println!("Possible causes:");
        println!("  1. SVG has transparent background that becomes white");
        println!("  2. SVG content is very light colored");
        println!("  3. Auto-threshold algorithm is not detecting content");
    }

    println!();
    println!("Mapping to braille grid (80×24)...");
    let grid = pixels_to_braille(&binary, 80, 24)?;
    println!("✓ Braille grid: {}×{}", grid.width(), grid.height());

    // Count non-space braille characters
    let mut braille_count = 0;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let ch = grid.get_char(x, y);
            if ch != ' ' && ch != '\u{2800}' {
                // Space and empty braille
                braille_count += 1;
            }
        }
    }
    println!(
        "  Non-empty braille cells: {} out of {} ({:.1}%)",
        braille_count,
        grid.width() * grid.height(),
        (braille_count as f32 / (grid.width() * grid.height()) as f32) * 100.0
    );

    if braille_count == 0 {
        println!();
        println!("✗ PROBLEM IDENTIFIED: No braille characters generated!");
        println!("The SVG loaded but produced no visible output.");
    } else {
        println!();
        println!("✓ SUCCESS: Braille output generated");
        println!();
        println!("First 10 lines of output:");
        println!("─────────────────────────────");
        for y in 0..10.min(grid.height()) {
            for x in 0..grid.width() {
                print!("{}", grid.get_char(x, y));
            }
            println!();
        }
    }

    Ok(())
}
