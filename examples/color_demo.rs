//! Color Demo - Story 2.6 Example
//!
//! Demonstrates color support for braille cells:
//! - Creating a grid with color support
//! - Assigning RGB colors to individual cells
//! - Rendering colored braille patterns to the terminal
//!
//! Run with: `cargo run --example color_demo`

use dotmax::{BrailleGrid, Color, TerminalRenderer};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Color Demo - Story 2.6");
    println!("======================\n");
    println!("Creating a colorful braille pattern...\n");

    // Create terminal renderer
    let mut renderer = TerminalRenderer::new()?;

    // Create a 40×20 braille grid
    let mut grid = BrailleGrid::new(40, 20)?;

    // Enable color support
    grid.enable_color_support();

    // Demo 1: Rainbow gradient
    println!("Rendering rainbow gradient...");
    thread::sleep(Duration::from_secs(1));

    for x in 0..40 {
        for y in 0..20 {
            // Set dots in a diagonal pattern
            if (x + y) % 3 == 0 {
                grid.set_dot(x * 2, y * 4)?;
            }

            // Create rainbow gradient across the width
            #[allow(clippy::cast_precision_loss)]
            let hue = (x as f32 / 40.0) * 360.0;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            grid.set_cell_color(x, y, Color::rgb(r, g, b))?;
        }
    }

    renderer.render(&grid)?;
    thread::sleep(Duration::from_secs(2));

    // Demo 2: Red, Green, Blue blocks
    grid.clear();
    println!("\nRendering RGB color blocks...");
    thread::sleep(Duration::from_secs(1));

    // Red block (left third)
    for x in 0..13 {
        for y in 0..20 {
            grid.set_dot(x * 2, y * 4)?;
            grid.set_cell_color(x, y, Color::rgb(255, 0, 0))?;
        }
    }

    // Green block (middle third)
    for x in 13..27 {
        for y in 0..20 {
            grid.set_dot(x * 2 + 1, y * 4 + 1)?;
            grid.set_cell_color(x, y, Color::rgb(0, 255, 0))?;
        }
    }

    // Blue block (right third)
    for x in 27..40 {
        for y in 0..20 {
            grid.set_dot(x * 2, y * 4 + 2)?;
            grid.set_cell_color(x, y, Color::rgb(0, 0, 255))?;
        }
    }

    renderer.render(&grid)?;
    thread::sleep(Duration::from_secs(2));

    // Demo 3: Predefined colors (black and white)
    grid.clear();
    println!("\nRendering checkerboard pattern (black/white)...");
    thread::sleep(Duration::from_secs(1));

    for x in 0..40 {
        for y in 0..20 {
            // Set all dots
            for dx in 0..2 {
                for dy in 0..4 {
                    grid.set_dot(x * 2 + dx, y * 4 + dy)?;
                }
            }

            // Checkerboard color pattern
            if (x + y) % 2 == 0 {
                grid.set_cell_color(x, y, Color::white())?;
            } else {
                grid.set_cell_color(x, y, Color::black())?;
            }
        }
    }

    renderer.render(&grid)?;
    thread::sleep(Duration::from_secs(2));

    // Demo 4: Clear colors (monochrome fallback)
    println!("\nClearing colors (monochrome)...");
    thread::sleep(Duration::from_secs(1));

    grid.clear_colors();
    renderer.render(&grid)?;
    thread::sleep(Duration::from_secs(2));

    // Clean up
    renderer.cleanup()?;

    println!("\n\nDemo complete! Color support verified.");
    println!("\nKey features demonstrated:");
    println!("  ✓ RGB color assignment per cell");
    println!("  ✓ Predefined colors (black, white, rgb)");
    println!("  ✓ Color rendering via TerminalRenderer");
    println!("  ✓ clear_colors() for monochrome fallback");

    Ok(())
}

/// Convert HSV to RGB
/// H: 0-360, S: 0-1, V: 0-1
/// Returns: (r, g, b) in 0-255 range
#[allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
