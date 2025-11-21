//! Font quality demonstration for SVG text rendering
//!
//! This example demonstrates best practices for SVG text rendering in dotmax.
//! It loads test SVGs with various font configurations and shows how font
//! fallback works when specific fonts are unavailable.
//!
//! Run with: cargo run --example svg_font_quality --features svg,image

use dotmax::image::load_svg_from_path;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see font loading details
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== SVG Font Quality Demonstration ===\n");
    println!("This example shows how dotmax handles SVG text rendering with");
    println!("automatic system font loading and fallback behavior.\n");

    // Test 1: Simple text with common fonts
    println!("1. Simple Text (Arial/Helvetica with sans-serif fallback)");
    println!("   - Requests: Arial, Helvetica");
    println!("   - Fallback: System sans-serif (DejaVu Sans on Linux)");
    let img1 = load_svg_from_path(
        Path::new("tests/test_assets/svg_font_tests/simple_text.svg"),
        400,
        300,
    )?;
    println!("   ✓ Loaded: {}×{} pixels\n", img1.width(), img1.height());

    // Test 2: Mixed font families
    println!("2. Mixed Fonts (Arial, Georgia, Courier, Ubuntu)");
    println!("   - Tests: Sans-serif, serif, monospace, and Linux-specific fonts");
    println!("   - Demonstrates: Font family preferences and fallback chains");
    let img2 = load_svg_from_path(
        Path::new("tests/test_assets/svg_font_tests/mixed_fonts.svg"),
        500,
        300,
    )?;
    println!("   ✓ Loaded: {}×{} pixels\n", img2.width(), img2.height());

    // Test 3: Small text sizes
    println!("3. Small Text (8px to 36px)");
    println!("   - Tests: Font sizes 8, 12, 16, 24, 36 pixels");
    println!("   - Note: Sizes < 12px may be hard to read in braille output");
    println!("   - Recommendation: Use ≥14pt for best braille legibility");
    let img3 = load_svg_from_path(
        Path::new("tests/test_assets/svg_font_tests/small_text.svg"),
        400,
        300,
    )?;
    println!("   ✓ Loaded: {}×{} pixels\n", img3.width(), img3.height());

    // Test 4: Fallback font behavior
    println!("4. Fallback Font (Requesting non-existent font)");
    println!("   - Requests: NonExistentFancyFont123");
    println!("   - Fallback chain: Comic Sans MS → sans-serif");
    println!("   - Result: Uses system sans-serif (DejaVu Sans on Linux)");
    println!("   - Important: No error/panic, graceful fallback");
    let img4 = load_svg_from_path(
        Path::new("tests/test_assets/svg_font_tests/fallback_font.svg"),
        500,
        200,
    )?;
    println!("   ✓ Loaded: {}×{} pixels\n", img4.width(), img4.height());

    // Test 5: Bold and italic font variants
    println!("5. Font Styles (Normal, Bold, Italic, Bold Italic)");
    println!("   - Tests: Font-weight and font-style attributes");
    println!("   - Demonstrates: System fonts support for weight/style variants");
    let img5 = load_svg_from_path(
        Path::new("tests/test_assets/svg_font_tests/bold_italic.svg"),
        500,
        300,
    )?;
    println!("   ✓ Loaded: {}×{} pixels\n", img5.width(), img5.height());

    println!("=== All SVG Font Tests Completed Successfully! ===\n");

    println!("Key Takeaways:");
    println!("- System fonts loaded automatically (no configuration needed)");
    println!("- Missing fonts fall back gracefully to generic families");
    println!("- Font loading adds ~8-10ms overhead (acceptable performance)");
    println!("- Use common fonts + fallback chains for best cross-platform support");
    println!("- Avoid very small text (< 12pt) for braille rendering");

    Ok(())
}
