//! Simple shapes demonstration (prints to stdout)
//!
//! This is a validation-friendly version that prints grid content to stdout
//! instead of using TerminalRenderer (which requires a TTY).
//!
//! Run with: cargo run --example shapes_demo_simple

use dotmax::primitives::{
    draw_polygon, draw_polygon_filled, draw_rectangle, draw_rectangle_filled,
};
use dotmax::BrailleGrid;

fn main() -> Result<(), dotmax::DotmaxError> {
    println!("═══════════════════════════════════════════════════════");
    println!("Shapes Demo - Visual Validation");
    println!("═══════════════════════════════════════════════════════\n");

    // Demo 1: Rectangle outlines
    println!("Demo 1: Rectangle outlines (various sizes)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_rectangle(&mut grid, 5, 5, 15, 10)?;
    draw_rectangle(&mut grid, 30, 5, 25, 15)?;
    draw_rectangle(&mut grid, 65, 5, 40, 30)?;
    print_grid(&grid);
    println!();

    // Demo 2: Filled rectangles
    println!("Demo 2: Filled rectangles (solid fill)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_rectangle_filled(&mut grid, 10, 5, 20, 12)?;
    draw_rectangle_filled(&mut grid, 40, 8, 25, 18)?;
    draw_rectangle_filled(&mut grid, 75, 10, 30, 25)?;
    print_grid(&grid);
    println!();

    // Demo 3: Triangles (outline)
    println!("Demo 3: Triangles (outline)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    let triangle1 = [(30, 5), (15, 35), (45, 35)];
    let triangle2 = [(90, 35), (75, 5), (105, 5)];
    draw_polygon(&mut grid, &triangle1)?;
    draw_polygon(&mut grid, &triangle2)?;
    print_grid(&grid);
    println!();

    // Demo 4: Filled triangles
    println!("Demo 4: Filled triangles (solid fill)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    let triangle = [(60, 8), (40, 40), (80, 40)];
    draw_polygon_filled(&mut grid, &triangle)?;
    print_grid(&grid);
    println!();

    // Demo 5: Regular polygons
    println!("Demo 5: Regular polygons (pentagon, hexagon)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;

    // Pentagon
    let pentagon = [(30, 8), (40, 15), (35, 30), (25, 30), (20, 15)];
    draw_polygon(&mut grid, &pentagon)?;

    // Hexagon
    let hexagon = [(70, 15), (80, 10), (90, 15), (90, 25), (80, 30), (70, 25)];
    draw_polygon(&mut grid, &hexagon)?;

    print_grid(&grid);
    println!();

    // Demo 6: Filled polygon (hexagon)
    println!("Demo 6: Filled hexagon (solid fill)");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    let hexagon = [(50, 10), (65, 8), (75, 18), (70, 32), (55, 34), (45, 24)];
    draw_polygon_filled(&mut grid, &hexagon)?;
    print_grid(&grid);
    println!();

    // Demo 7: Mixed - rectangles and polygons
    println!("Demo 7: Mixed shapes");
    println!("───────────────────────────────────────────────────────");
    let mut grid = BrailleGrid::new(60, 12)?;
    draw_rectangle(&mut grid, 5, 5, 30, 25)?;
    let triangle = [(60, 8), (45, 32), (75, 32)];
    draw_polygon(&mut grid, &triangle)?;
    draw_rectangle_filled(&mut grid, 90, 15, 20, 18)?;
    print_grid(&grid);
    println!();

    println!("═══════════════════════════════════════════════════════");
    println!("✓ All shape demos rendered successfully!");
    println!("✓ Visual inspection: Check edges are continuous");
    println!("✓ Check: Filled shapes have solid interiors");
    println!("✓ Check: Polygon vertices connect properly");
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
