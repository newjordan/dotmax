//! Demonstration of RGB to ANSI color conversion.
//!
//! Run with: `cargo run --example color_conversion_demo`

use dotmax::color::convert::{
    ansi256_fg_escape, color_reset, rgb_to_ansi16, rgb_to_ansi256, rgb_to_terminal_color,
    rgb_to_truecolor_escape,
};
use dotmax::ColorCapability;

fn main() {
    println!("=== dotmax Color Conversion Demo ===\n");

    // Demo 1: Primary colors in ANSI 256
    println!("--- RGB to ANSI 256 Conversion ---");
    let colors = [
        ("Red", (255, 0, 0)),
        ("Green", (0, 255, 0)),
        ("Blue", (0, 0, 255)),
        ("Yellow", (255, 255, 0)),
        ("Cyan", (0, 255, 255)),
        ("Magenta", (255, 0, 255)),
        ("White", (255, 255, 255)),
        ("Black", (0, 0, 0)),
        ("Gray", (128, 128, 128)),
    ];

    for (name, (r, g, b)) in colors {
        let ansi256 = rgb_to_ansi256(r, g, b);
        let escape = ansi256_fg_escape(ansi256);
        println!(
            "{escape}■{reset} {name}: RGB({r}, {g}, {b}) → ANSI 256 index {ansi256}",
            reset = color_reset()
        );
    }

    println!("\n--- RGB to ANSI 16 Conversion ---");
    for (name, (r, g, b)) in colors {
        let ansi16 = rgb_to_ansi16(r, g, b);
        println!("{name}: RGB({r}, {g}, {b}) → ANSI 16 index {ansi16}");
    }

    // Demo 2: True color gradient
    println!("\n--- True Color Gradient (if supported) ---");
    print!("Gradient: ");
    for i in 0..32 {
        let r = ((i as f32 / 31.0) * 255.0) as u8;
        let g = 0;
        let b = 255 - r;
        let escape = rgb_to_truecolor_escape(r, g, b);
        print!("{escape}█{}", color_reset());
    }
    println!();

    // Demo 3: Smart conversion based on capability
    println!("\n--- Smart Conversion (rgb_to_terminal_color) ---");
    let test_color = (255, 128, 64); // Orange

    println!(
        "Orange RGB({}, {}, {}):",
        test_color.0, test_color.1, test_color.2
    );

    let capabilities = [
        ("TrueColor", ColorCapability::TrueColor),
        ("ANSI 256", ColorCapability::Ansi256),
        ("ANSI 16", ColorCapability::Ansi16),
        ("Monochrome", ColorCapability::Monochrome),
    ];

    for (name, cap) in capabilities {
        let escape = rgb_to_terminal_color(test_color.0, test_color.1, test_color.2, cap);
        if escape.is_empty() {
            println!("  {name}: (no color output)");
        } else {
            println!("  {name}: {escape}Sample{}", color_reset());
        }
    }

    // Demo 4: Grayscale ramp
    println!("\n--- Grayscale Ramp (ANSI 256 indices 232-255) ---");
    print!("Dark to Light: ");
    for gray in (0..=255).step_by(11) {
        let ansi = rgb_to_ansi256(gray, gray, gray);
        let escape = ansi256_fg_escape(ansi);
        print!("{escape}█{}", color_reset());
    }
    println!();

    println!("\n=== Demo Complete ===");
}
