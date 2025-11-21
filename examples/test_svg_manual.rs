//! Manual test for SVG rendering - displays SVG in terminal for human verification

#![allow(clippy::unnecessary_debug_formatting, clippy::uninlined_format_args)]

use dotmax::image::{auto_threshold, load_svg_from_path, pixels_to_braille};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_path = Path::new("tests/fixtures/svg/dark_bg_light_content.svg");

    println!("=== SVG Manual Test ===");
    println!("File: {}", svg_path.display());
    println!();

    // Load and rasterize SVG
    let img = load_svg_from_path(svg_path, 160, 96)?;

    // Convert to binary
    let binary = auto_threshold(&img);

    // Map to braille
    let grid = pixels_to_braille(&binary, 80, 24)?;

    println!("Rendered output:");
    // Print grid directly
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            print!("{}", grid.get_char(x, y));
        }
        println!();
    }
    println!();

    // Count filled cells
    let mut filled = 0;
    let mut empty = 0;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let ch = grid.get_char(x, y);
            if ch != ' ' && ch != '\u{2800}' {
                filled += 1;
            } else {
                empty += 1;
            }
        }
    }

    let total = filled + empty;
    let filled_pct = (filled * 100) / total;

    println!("Stats: {} filled, {} empty ({}% filled)", filled, empty, filled_pct);
    println!();
    println!("Can you see the content clearly? (The text 'Omni-Booth' should be visible)");

    Ok(())
}
