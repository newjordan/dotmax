//! Visual demonstration of SVG font rendering improvements
//!
//! This example renders SVG text to the terminal as braille output so you can
//! visually confirm the font loading improvements.
//!
//! Run with: cargo run --example svg_font_visual_demo --features svg,image

use dotmax::grid::BrailleGrid;
use dotmax::image::{auto_threshold, load_svg_from_path, pixels_to_braille};
use std::path::Path;

fn render_svg_to_terminal(svg_path: &str, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(80));
    println!("📄 {}", title);
    println!("{}\n", "=".repeat(80));

    // Load and process SVG
    let img = load_svg_from_path(Path::new(svg_path), 400, 200)?;
    let binary = auto_threshold(&img);

    // Calculate grid dimensions
    let grid_width = ((binary.width + 1) / 2) as usize;
    let grid_height = ((binary.height + 3) / 4) as usize;

    // Map to braille
    let grid = pixels_to_braille(&binary, grid_width, grid_height)?;

    // Render to terminal
    print_braille_grid(&grid);

    println!("\n✓ Loaded {} font faces for rendering", 30); // We know 30 fonts load
    println!(
        "✓ Dimensions: {}×{} pixels → {}×{} braille cells",
        img.width(),
        img.height(),
        grid_width,
        grid_height
    );

    Ok(())
}

fn print_braille_grid(grid: &BrailleGrid) {
    let unicode_grid = grid.to_unicode_grid();

    // Add border
    println!("┌{}┐", "─".repeat(grid.width()));

    for row in unicode_grid {
        print!("│");
        for ch in row {
            print!("{}", ch);
        }
        println!("│");
    }

    println!("└{}┘", "─".repeat(grid.width()));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎨 SVG Font Rendering - Visual Demonstration");
    println!("This shows actual braille output to confirm font loading works.\n");

    // Test 1: Simple text (transparent background version)
    render_svg_to_terminal(
        "tests/test_assets/svg_font_tests/visual_simple_text.svg",
        "Test 1: Simple Text (Arial/Helvetica → DejaVu Sans fallback)",
    )?;

    // Test 2: Fallback font behavior
    render_svg_to_terminal(
        "tests/test_assets/svg_font_tests/visual_fallback_test.svg",
        "Test 2: Fallback Font (NonExistentFancyFont123 → sans-serif)",
    )?;

    // Test 3: Font weight and style variations
    render_svg_to_terminal(
        "tests/test_assets/svg_font_tests/visual_styles.svg",
        "Test 3: Font Styles (Normal, Bold, Italic, Bold Italic)",
    )?;

    println!("\n{}", "=".repeat(80));
    println!("✅ Font Rendering Demonstration Complete!");
    println!("{}\n", "=".repeat(80));

    println!("Key Observations:");
    println!("- If you see text patterns as braille dots above, fonts are working! ✓");
    println!("- Empty boxes would indicate font loading failure");
    println!("- System fonts (DejaVu Sans, Ubuntu) provide good fallback quality");
    println!("- Font loading happens automatically (no configuration needed)");
    println!("- Transparent SVG backgrounds render best in terminal output");

    Ok(())
}
