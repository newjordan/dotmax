//! Hello Braille - Simple viewport test
//!
//! Run with: RUST_LOG=dotmax=debug cargo run --example hello_braille

use dotmax::{BrailleGrid, TerminalRenderer};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== Simple Viewport Test ===\n");

    let mut renderer = TerminalRenderer::new()?;
    let (term_width, term_height) = renderer.get_terminal_size()?;

    println!("Terminal reports: {}×{}", term_width, term_height);

    // Create a grid that matches EXACTLY what terminal reports
    let mut grid = BrailleGrid::new(term_width as usize, term_height as usize)?;

    println!("Grid created: {}×{} cells", grid.width(), grid.height());

    let dot_width = grid.width() * 2;
    let dot_height = grid.height() * 4;

    // Draw THREE horizontal lines:
    // 1. At Y=0 (top)
    // 2. At Y=dot_height/2 (middle)
    // 3. At Y=dot_height-1 (bottom)

    println!("Drawing 3 horizontal lines:");
    println!("  - Top line at dot Y=0");
    println!("  - Middle line at dot Y={}", dot_height / 2);
    println!("  - Bottom line at dot Y={}", dot_height - 1);

    // TOP LINE
    for x in 0..dot_width {
        grid.set_dot(x, 0)?;
    }

    // MIDDLE LINE
    for x in 0..dot_width {
        grid.set_dot(x, dot_height / 2)?;
    }

    // BOTTOM LINE
    for x in 0..dot_width {
        grid.set_dot(x, dot_height - 1)?;
    }

    println!("\nRendering... (check debug logs for 'Rendering area')");

    renderer.render(&grid)?;

    println!("\n❓ What do you see?");
    println!("   Expected: 3 horizontal lines (top, middle, bottom)");
    println!("   If you only see 2 lines → the grid is bigger than viewport");

    thread::sleep(Duration::from_secs(15));

    renderer.cleanup()?;

    Ok(())
}
