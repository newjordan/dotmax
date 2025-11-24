//! Simple circle drawing demonstration (prints to stdout)
//!
//! This is a validation-friendly version that prints grid content to stdout
//! instead of using TerminalRenderer (which requires a TTY).
//!
//! Run with: cargo run --example circles_demo_simple

use dotmax::primitives::{draw_circle, draw_circle_filled};
use dotmax::BrailleGrid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("Circle Drawing Demo - Visual Validation");
    println!("═══════════════════════════════════════════════════════\n");

    // Demo 1: Small circles
    println!("Demo 1: Small circles (radius 5, 10, 15)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_circle(&mut grid, 15, 24, 5)?;
    draw_circle(&mut grid, 40, 24, 10)?;
    draw_circle(&mut grid, 75, 24, 15)?;
    print_grid(&grid);
    println!();

    // Demo 2: Concentric circles
    println!("Demo 2: Concentric circles pattern");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    let center_x = 60;
    let center_y = 24;
    for radius in [8, 12, 16, 20] {
        draw_circle(&mut grid, center_x, center_y, radius)?;
    }
    print_grid(&grid);
    println!();

    // Demo 3: Filled circles
    println!("Demo 3: Filled circles (solid fill)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_circle_filled(&mut grid, 30, 24, 12)?;
    draw_circle_filled(&mut grid, 90, 24, 16)?;
    print_grid(&grid);
    println!();

    // Demo 4: Filled vs outline comparison
    println!("Demo 4: Filled vs outline (side by side)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_circle(&mut grid, 40, 24, 18)?; // Outline
    draw_circle_filled(&mut grid, 80, 24, 18)?; // Filled
    print_grid(&grid);
    println!();

    // Demo 5: Overlapping circles (Venn diagram)
    println!("Demo 5: Overlapping circles");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_circle(&mut grid, 45, 24, 15)?;
    draw_circle(&mut grid, 75, 24, 15)?;
    draw_circle(&mut grid, 60, 36, 15)?;
    print_grid(&grid);
    println!();

    println!("═══════════════════════════════════════════════════════");
    println!("✓ All circle demos rendered successfully!");
    println!("✓ Visual inspection: Circles should be smooth and round");
    println!("✓ Check: No gaps in perimeters, proper symmetry");
    println!("═══════════════════════════════════════════════════════");

    Ok(())
}

/// Helper function to print grid contents to stdout
fn print_grid(grid: &BrailleGrid) {
    let (width, height) = grid.dimensions();
    for y in 0..height {
        for x in 0..width {
            let c = grid.get_char(x, y);
            print!("{}", c);
        }
        println!();
    }
}
