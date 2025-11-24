//! Simple colored shapes demonstration (prints to stdout)
//!
//! This is a validation-friendly version that prints grid content to stdout
//! instead of using TerminalRenderer (which requires a TTY).
//!
//! Note: Colors will show ANSI escape codes in this simple version.
//! For proper visual rendering, use colored_shapes.rs in an interactive terminal.
//!
//! Run with: cargo run --example colored_shapes_simple --all-features

use dotmax::primitives::{
    draw_circle_colored, draw_line_colored, draw_polygon_colored, draw_rectangle_colored,
};
use dotmax::{BrailleGrid, Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("Colored Shapes Demo - Visual Validation");
    println!("═══════════════════════════════════════════════════════\n");

    // Create grid and enable color support
    let mut grid = BrailleGrid::new(60, 16)?;
    grid.enable_color_support();

    let dot_width = grid.dot_width() as i32;
    let dot_height = grid.dot_height() as i32;
    let center_x = dot_width / 2;
    let center_y = dot_height / 2;

    // 1. Red circle
    let red = Color::rgb(255, 0, 0);
    draw_circle_colored(&mut grid, center_x, center_y, 15, red, false)?;

    // 2. Green rectangle border
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

    // 3. Blue diagonal line
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

    // 4. Yellow triangle
    let yellow = Color::rgb(255, 255, 0);
    let triangle = [
        (center_x - 20, center_y - 12),
        (center_x + 20, center_y - 12),
        (center_x, center_y + 12),
    ];
    draw_polygon_colored(&mut grid, &triangle, yellow, true)?;

    // 5. Cyan horizontal line
    let cyan = Color::rgb(0, 255, 255);
    draw_line_colored(
        &mut grid,
        20,
        center_y,
        dot_width - 20,
        center_y,
        cyan,
        Some(2),
    )?;

    // 6. Magenta filled small circle
    let magenta = Color::rgb(255, 0, 255);
    draw_circle_colored(&mut grid, center_x - 30, center_y - 15, 6, magenta, true)?;

    // 7. White filled rectangle
    let white = Color::rgb(255, 255, 255);
    draw_rectangle_colored(&mut grid, center_x + 20, center_y + 8, 12, 8, white, true)?;

    println!("Demo: All colored primitives");
    println!("───────────────────────────────────────────────────────");
    print_grid_with_color(&grid);
    println!();

    println!("═══════════════════════════════════════════════════════");
    println!("✓ All colored shape functions executed successfully!");
    println!("✓ Primitives demonstrated:");
    println!("  - draw_line_colored (blue diagonal, cyan horizontal)");
    println!("  - draw_circle_colored (red outline, magenta filled)");
    println!("  - draw_rectangle_colored (green border, white filled)");
    println!("  - draw_polygon_colored (yellow triangle)");
    println!();
    println!("⚠️  Note: This simple demo shows ANSI codes in output.");
    println!("   For proper visual display, run in an interactive terminal:");
    println!("   cargo run --example colored_shapes --all-features");
    println!("═══════════════════════════════════════════════════════");

    Ok(())
}

/// Helper function to print grid contents with color info
fn print_grid_with_color(grid: &BrailleGrid) {
    let (width, height) = grid.dimensions();
    for y in 0..height {
        for x in 0..width {
            let c = grid.get_char(x, y);
            // Get color if available
            if let Some(color) = grid.get_color(x, y) {
                // Print with ANSI color code
                print!("\x1b[38;2;{};{};{}m{}\x1b[0m", color.r, color.g, color.b, c);
            } else {
                print!("{}", c);
            }
        }
        println!();
    }
}
