//! Quick module demonstration (Story 8.2)
//!
//! This example demonstrates the one-liner convenience functions in the `quick` module.
//! The quick module provides the simplest possible API for common dotmax tasks.
//!
//! # Running this example
//!
//! Without image support (demonstrates grid/show):
//! ```bash
//! cargo run --example quick_demo
//! ```
//!
//! With image support (demonstrates all functions):
//! ```bash
//! cargo run --example quick_demo --features image
//! ```
//!
//! # What this example shows
//!
//! 1. `quick::grid()` - Create a terminal-sized grid
//! 2. `quick::grid_sized()` - Create a grid with specific dimensions
//! 3. `quick::show()` - Display a grid and wait for keypress
//! 4. `quick::load_image()` - Load an image into a grid (image feature)
//! 5. `quick::show_image()` - One-line image display (image feature)

use dotmax::primitives::{draw_circle, draw_line, draw_rectangle};
use dotmax::quick;

fn main() -> Result<(), dotmax::DotmaxError> {
    println!("=== dotmax quick module demo ===\n");

    // Example 1: Create a terminal-sized grid and draw on it
    println!("1. Creating terminal-sized grid with quick::grid()");
    let grid = quick::grid()?;
    println!(
        "   Grid created: {}x{} cells ({}x{} dots)",
        grid.width(),
        grid.height(),
        grid.dot_width(),
        grid.dot_height()
    );

    // Example 2: Create a grid with specific dimensions
    println!("\n2. Creating 40x20 grid with quick::grid_sized(40, 20)");
    let mut grid = quick::grid_sized(40, 20)?;
    println!("   Grid created: {}x{} cells", grid.width(), grid.height());

    // Draw some shapes on the grid
    println!("   Drawing shapes...");

    // Get dimensions for drawing
    let dot_w = grid.dot_width() as u32;
    let dot_h = grid.dot_height() as u32;

    // Draw a border (draw_rectangle takes i32 for position, u32 for size)
    draw_rectangle(&mut grid, 0, 0, dot_w, dot_h)?;

    // Draw diagonal lines (draw_line takes i32 for all coordinates)
    let w = dot_w as i32 - 1;
    let h = dot_h as i32 - 1;
    draw_line(&mut grid, 0, 0, w, h)?;
    draw_line(&mut grid, w, 0, 0, h)?;

    // Draw a circle in the center (draw_circle takes i32 for position)
    let center_x = w / 2;
    let center_y = h / 2;
    draw_circle(&mut grid, center_x, center_y, 15)?;

    // Display the grid
    println!("\n3. Displaying grid with quick::show()");
    println!("   Press any key to continue after viewing...\n");
    quick::show(&grid)?;
    println!("   Grid displayed and dismissed.");

    // Image examples (only with image feature)
    #[cfg(feature = "image")]
    {
        use std::path::Path;

        // Try to find a sample image
        let sample_paths = [
            "tests/fixtures/images/sample.png",
            "examples/sample.png",
            "sample.png",
        ];

        let sample_image = sample_paths.iter().find(|p| Path::new(p).exists());

        if let Some(image_path) = sample_image {
            println!("\n4. Loading image with quick::load_image()");
            let image_grid = quick::load_image(image_path)?;
            println!(
                "   Loaded {} into {}x{} grid",
                image_path,
                image_grid.width(),
                image_grid.height()
            );

            println!("\n5. Displaying image with quick::show_image() - one line!");
            println!("   Press any key after viewing...\n");
            quick::show_image(image_path)?;
            println!("   Image displayed and dismissed.");
        } else {
            println!("\n4-5. Skipping image demos (no sample image found)");
            println!("   Try placing a sample.png in the examples/ directory");
        }
    }

    #[cfg(not(feature = "image"))]
    {
        println!("\n4-5. Image demos require --features image");
        println!("   Run: cargo run --example quick_demo --features image");
    }

    println!("\n=== Demo complete ===");
    Ok(())
}
