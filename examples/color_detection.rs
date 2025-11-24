//! Example: Terminal Color Capability Detection
//!
//! This example demonstrates how dotmax detects terminal color capabilities
//! automatically using environment variables.
//!
//! Run with: `cargo run --example color_detection`

use dotmax::{detect_color_capability, ColorCapability};
use std::env;

fn main() {
    println!("=== Terminal Color Capability Detection ===\n");

    // Detect capability (result is cached after first call)
    let capability = detect_color_capability();

    // Display detected capability
    println!("Detected Capability: {:?}", capability);
    println!("Display Name: {}", capability);
    println!();

    // Show support levels
    println!("Support Levels:");
    println!("  Supports Color: {}", capability.supports_color());
    println!(
        "  Supports True Color (24-bit): {}",
        capability.supports_truecolor()
    );
    println!();

    // Show environment variables used for detection
    println!("Environment Variables:");
    println!(
        "  COLORTERM: {}",
        env::var("COLORTERM").unwrap_or_else(|_| "(not set)".to_string())
    );
    println!(
        "  TERM: {}",
        env::var("TERM").unwrap_or_else(|_| "(not set)".to_string())
    );
    println!();

    // Explain what this means for the user
    println!("What This Means:");
    match capability {
        ColorCapability::Monochrome => {
            println!("  Your terminal does not support colors.");
            println!("  Output will be monochrome (black and white).");
            println!("  This is rare in modern terminals.");
        }
        ColorCapability::Ansi16 => {
            println!("  Your terminal supports 16 basic ANSI colors.");
            println!("  This includes 8 standard colors plus 8 bright variants.");
            println!("  RGB colors will be mapped to the closest of these 16 colors.");
        }
        ColorCapability::Ansi256 => {
            println!("  Your terminal supports 256 colors (extended ANSI).");
            println!("  This is the most common capability in modern terminals.");
            println!("  RGB colors will be mapped to the 256-color palette.");
        }
        ColorCapability::TrueColor => {
            println!("  Your terminal supports 24-bit true color!");
            println!("  This allows 16 million colors (full RGB).");
            println!("  RGB colors will be rendered exactly as specified.");
        }
    }
    println!();

    // Demonstrate the detect() alias
    println!("Alternative API:");
    let via_alias = ColorCapability::detect();
    println!("  ColorCapability::detect() returns: {:?}", via_alias);
    println!(
        "  Same as detect_color_capability(): {}",
        via_alias == capability
    );
    println!();

    // Show caching behavior
    println!("Caching Behavior:");
    println!("  Detection is cached using OnceLock.");
    println!("  First call: <1ms (environment variable reads)");
    println!("  Subsequent calls: <1ns (cached result)");

    // Show color output demonstration if supported
    if capability.supports_color() {
        println!();
        println!("Color Demo (if your terminal supports it):");
        if capability.supports_truecolor() {
            // True color demo using RGB escape codes
            println!(
                "  \x1b[38;2;255;0;0mRed\x1b[0m \
                 \x1b[38;2;0;255;0mGreen\x1b[0m \
                 \x1b[38;2;0;0;255mBlue\x1b[0m \
                 \x1b[38;2;255;255;0mYellow\x1b[0m \
                 \x1b[38;2;255;0;255mMagenta\x1b[0m \
                 \x1b[38;2;0;255;255mCyan\x1b[0m"
            );
            println!("  (Using 24-bit RGB escape codes)");
        } else {
            // Basic ANSI color demo
            println!(
                "  \x1b[31mRed\x1b[0m \
                 \x1b[32mGreen\x1b[0m \
                 \x1b[34mBlue\x1b[0m \
                 \x1b[33mYellow\x1b[0m \
                 \x1b[35mMagenta\x1b[0m \
                 \x1b[36mCyan\x1b[0m"
            );
            println!("  (Using basic ANSI escape codes)");
        }
    }
}
