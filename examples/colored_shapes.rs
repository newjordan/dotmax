//! Colored shapes example demonstrating Story 4.5 color support for drawing primitives.
//!
//! This example draws multiple colored shapes on the braille grid:
//! - Red circle at center
//! - Green rectangle as border
//! - Blue diagonal line
//! - Yellow polygon (triangle)
//!
//! Run with: `cargo run --example colored_shapes --all-features`

use dotmax::primitives::{
    draw_circle_colored, draw_line_colored, draw_polygon_colored, draw_rectangle_colored,
};
use dotmax::{BrailleGrid, Color, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create grid and enable color support
    let mut grid = BrailleGrid::new(80, 24)?;
    grid.enable_color_support();

    // Calculate grid dimensions in dots
    let dot_width = grid.dot_width() as i32;
    let dot_height = grid.dot_height() as i32;
    let center_x = dot_width / 2;
    let center_y = dot_height / 2;

    // 1. Draw green rectangle as border
    let green = Color::rgb(0, 255, 0);
    draw_rectangle_colored(
        &mut grid,
        5,
        5,
        (dot_width - 10) as u32,
        (dot_height - 10) as u32,
        green,
        false,
    )?;

    // 2. Draw red circle at center
    let red = Color::rgb(255, 0, 0);
    draw_circle_colored(&mut grid, center_x, center_y, 15, red, false)?;

    // 3. Draw blue diagonal line (top-left to bottom-right)
    let blue = Color::rgb(0, 0, 255);
    draw_line_colored(
        &mut grid,
        10,
        10,
        dot_width - 10,
        dot_height - 10,
        blue,
        None,
    )?;

    // 4. Draw yellow triangle (polygon)
    let yellow = Color::rgb(255, 255, 0);
    let triangle = [
        (center_x - 20, center_y - 15),
        (center_x + 20, center_y - 15),
        (center_x, center_y + 15),
    ];
    draw_polygon_colored(&mut grid, &triangle, yellow, true)?;

    // 5. Draw cyan thick line (horizontal)
    let cyan = Color::rgb(0, 255, 255);
    draw_line_colored(
        &mut grid,
        20,
        center_y,
        dot_width - 20,
        center_y,
        cyan,
        Some(3),
    )?;

    // 6. Draw magenta filled small circle
    let magenta = Color::rgb(255, 0, 255);
    draw_circle_colored(&mut grid, center_x - 30, center_y - 20, 8, magenta, true)?;

    // 7. Draw white filled rectangle
    let white = Color::rgb(255, 255, 255);
    draw_rectangle_colored(&mut grid, center_x + 20, center_y + 10, 15, 10, white, true)?;

    // Render to terminal
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    println!("\nColored Shapes Demo (Story 4.5)");
    println!("================================");
    println!("✓ Green border rectangle");
    println!("✓ Red circle (outline)");
    println!("✓ Blue diagonal line");
    println!("✓ Yellow triangle (polygon)");
    println!("✓ Cyan thick horizontal line");
    println!("✓ Magenta filled circle");
    println!("✓ White filled rectangle");
    println!("\nAll colored primitive functions demonstrated!");

    Ok(())
}
