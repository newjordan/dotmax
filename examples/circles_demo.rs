//! Circle drawing demonstration
//!
//! This example demonstrates the circle drawing capabilities of dotmax:
//! - Small, medium, and large circles
//! - Concentric circles patterns
//! - Filled vs outline circles
//! - Thick circles with various thickness values
//! - Boundary clipping behavior
//!
//! Run with: cargo run --example circles_demo

use dotmax::primitives::{draw_circle, draw_circle_filled, draw_circle_thick};
use dotmax::{BrailleGrid, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize grid (80×24 cells = 160×96 dots)
    let mut grid = BrailleGrid::new(80, 24)?;

    println!("Circle Drawing Demo");
    println!("===================\n");

    // Demo 1: Small circles (radius 5, 10)
    println!("1. Small circles (radius 5, 10)");
    draw_circle(&mut grid, 10, 10, 5)?;
    draw_circle(&mut grid, 30, 10, 10)?;

    // Demo 2: Medium circles (radius 20, 30)
    println!("2. Medium circles (radius 20, 30, 40)");
    draw_circle(&mut grid, 70, 20, 20)?;
    draw_circle(&mut grid, 120, 40, 30)?;

    // Demo 3: Large circle (radius 45)
    println!("3. Large circle (radius 45)");
    draw_circle(&mut grid, 80, 70, 45)?;

    // Render the grid
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 4: Concentric circles pattern
    println!("4. Concentric circles pattern (radii 10, 20, 30, 40)");
    let center_x = 80;
    let center_y = 48;
    for radius in [10, 20, 30, 40] {
        draw_circle(&mut grid, center_x, center_y, radius)?;
    }
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 5: Filled circles
    println!("5. Filled circles (radius 15, 20, 25)");
    draw_circle_filled(&mut grid, 30, 24, 15)?;
    draw_circle_filled(&mut grid, 80, 48, 20)?;
    draw_circle_filled(&mut grid, 130, 72, 25)?;
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 6: Filled vs outline comparison
    println!("6. Filled vs outline comparison");
    draw_circle(&mut grid, 40, 48, 30)?; // Outline
    draw_circle_filled(&mut grid, 120, 48, 30)?; // Filled
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 7: Thick circles (thickness 3, 5)
    println!("7. Thick circles (thickness 3, 5)");
    draw_circle_thick(&mut grid, 40, 40, 25, 3)?;
    draw_circle_thick(&mut grid, 120, 56, 30, 5)?;
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 8: Varying thickness comparison
    println!("8. Varying thickness (1, 2, 3, 4, 5)");
    let base_x = 30;
    for (i, thickness) in [1, 2, 3, 4, 5].iter().enumerate() {
        let x = base_x + (i as i32 * 28);
        draw_circle_thick(&mut grid, x, 48, 20, *thickness)?;
    }
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 9: Boundary clipping - circle partially off-grid
    println!("9. Boundary clipping (circles partially off-grid)");
    draw_circle(&mut grid, -10, 10, 25)?; // Top-left corner
    draw_circle(&mut grid, 170, 10, 25)?; // Top-right corner
    draw_circle(&mut grid, -10, 86, 25)?; // Bottom-left corner
    draw_circle(&mut grid, 170, 86, 25)?; // Bottom-right corner
    draw_circle(&mut grid, 80, -15, 30)?; // Top edge
    draw_circle(&mut grid, 80, 110, 30)?; // Bottom edge
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 10: Artistic pattern - spiral of circles
    println!("10. Artistic pattern - circular arrangement");
    let center_x = 80;
    let center_y = 48;
    let orbit_radius = 35;
    for angle in 0..12 {
        let theta = (angle as f32) * (2.0 * std::f32::consts::PI / 12.0);
        let x = center_x + (orbit_radius as f32 * theta.cos()) as i32;
        let y = center_y + (orbit_radius as f32 * theta.sin()) as i32;
        draw_circle(&mut grid, x, y, 8)?;
    }
    // Center circle
    draw_circle_filled(&mut grid, center_x, center_y, 12)?;
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 11: Overlapping circles
    println!("11. Overlapping circles (Venn diagram style)");
    draw_circle(&mut grid, 60, 48, 30)?;
    draw_circle(&mut grid, 100, 48, 30)?;
    draw_circle(&mut grid, 80, 68, 30)?;
    renderer.render(&grid)?;

    println!("\n---\n");

    // Clear grid for next demo
    grid = BrailleGrid::new(80, 24)?;

    // Demo 12: Mixed - circles and filled circles
    println!("12. Mixed patterns - outline and filled");
    // Outer ring
    for radius in [40, 42, 44] {
        draw_circle(&mut grid, 80, 48, radius)?;
    }
    // Inner filled circles
    draw_circle_filled(&mut grid, 60, 48, 15)?;
    draw_circle_filled(&mut grid, 100, 48, 15)?;
    draw_circle_filled(&mut grid, 80, 28, 15)?;
    draw_circle_filled(&mut grid, 80, 68, 15)?;
    renderer.render(&grid)?;

    println!("\nAll circle demos complete!");

    Ok(())
}
