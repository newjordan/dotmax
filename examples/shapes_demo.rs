//! Shapes Demo - Rectangle and Polygon Drawing Examples
//!
//! This example demonstrates the rectangle and polygon drawing capabilities of dotmax.
//! Run with: cargo run --example shapes_demo

use dotmax::primitives::{
    draw_polygon, draw_polygon_filled, draw_rectangle, draw_rectangle_filled, draw_rectangle_thick,
};
use dotmax::{BrailleGrid, TerminalRenderer};

fn main() -> Result<(), dotmax::DotmaxError> {
    println!("dotmax - Shapes Demo");
    println!("====================\n");

    // Initialize grid (80×24 cells = 160×96 dots)
    let mut grid = BrailleGrid::new(80, 24)?;

    // Demo 1: Small Rectangle Outline
    println!("Demo 1: Small Rectangle Outline (20×15 dots)");
    draw_rectangle(&mut grid, 10, 5, 20, 15)?;

    // Demo 2: Medium Rectangle Outline
    println!("Demo 2: Medium Rectangle Outline (30×20 dots)");
    draw_rectangle(&mut grid, 40, 5, 30, 20)?;

    // Demo 3: Large Rectangle Outline
    println!("Demo 3: Large Rectangle Outline (50×25 dots)");
    draw_rectangle(&mut grid, 80, 5, 50, 25)?;

    // Demo 4: Filled Rectangle (Small Panel)
    println!("Demo 4: Filled Rectangle (15×10 dots solid)");
    draw_rectangle_filled(&mut grid, 10, 30, 15, 10)?;

    // Demo 5: Filled Rectangle (Medium Background)
    println!("Demo 5: Filled Rectangle (25×15 dots background)");
    draw_rectangle_filled(&mut grid, 35, 30, 25, 15)?;

    // Demo 6: Thick Rectangle (Thickness 2)
    println!("Demo 6: Thick Rectangle Border (thickness 2)");
    draw_rectangle_thick(&mut grid, 70, 30, 30, 20, 2)?;

    // Demo 7: Thick Rectangle (Thickness 3)
    println!("Demo 7: Thick Rectangle Border (thickness 3)");
    draw_rectangle_thick(&mut grid, 110, 30, 35, 25, 3)?;

    // Demo 8: Thick Rectangle (Thickness 5)
    println!("Demo 8: Thick Rectangle Border (thickness 5)");
    draw_rectangle_thick(&mut grid, 10, 55, 40, 30, 5)?;

    // Demo 9: Triangle (Upright)
    println!("Demo 9: Triangle Outline (upright orientation)");
    let triangle_upright = [(75, 55), (55, 80), (95, 80)];
    draw_polygon(&mut grid, &triangle_upright)?;

    // Demo 10: Triangle (Inverted)
    println!("Demo 10: Triangle Outline (inverted orientation)");
    let triangle_inverted = [(75, 85), (55, 60), (95, 60)];
    draw_polygon(&mut grid, &triangle_inverted)?;

    // Demo 11: Square (via polygon with 4 vertices)
    println!("Demo 11: Square via Polygon (20×20 dots)");
    let square = [(105, 55), (125, 55), (125, 75), (105, 75)];
    draw_polygon(&mut grid, &square)?;

    // Demo 12: Pentagon (5 vertices, regular)
    println!("Demo 12: Pentagon Outline (5 vertices)");
    let pentagon = [(135, 55), (145, 65), (140, 80), (130, 80), (125, 65)];
    draw_polygon(&mut grid, &pentagon)?;

    // Demo 13: Hexagon (6 vertices)
    println!("Demo 13: Hexagon Outline (6 vertices)");
    let hexagon = [
        (20, 90),
        (30, 85),
        (40, 90),
        (40, 100),
        (30, 105),
        (20, 100),
    ];
    draw_polygon(&mut grid, &hexagon)?;

    // Demo 14: Octagon (8 vertices)
    println!("Demo 14: Octagon Outline (8 vertices)");
    let octagon = [
        (65, 88),
        (73, 86),
        (78, 92),
        (78, 100),
        (73, 106),
        (65, 108),
        (57, 106),
        (52, 100),
    ];
    draw_polygon(&mut grid, &octagon)?;

    // Demo 15: Filled Triangle (Solid)
    println!("Demo 15: Filled Triangle (solid fill)");
    let filled_triangle = [(100, 85), (85, 105), (115, 105)];
    draw_polygon_filled(&mut grid, &filled_triangle)?;

    // Demo 16: Filled Hexagon (Solid)
    println!("Demo 16: Filled Hexagon (solid fill)");
    let filled_hexagon = [
        (135, 85),
        (143, 90),
        (143, 100),
        (135, 105),
        (127, 100),
        (127, 90),
    ];
    draw_polygon_filled(&mut grid, &filled_hexagon)?;

    // Demo 17: Irregular Polygon (Asymmetric Shape)
    println!("Demo 17: Irregular Polygon Outline (asymmetric)");
    let irregular = [(55, 115), (70, 112), (85, 118), (80, 128), (60, 130)];
    draw_polygon(&mut grid, &irregular)?;

    // Demo 18: Complex Polygon (Many Vertices - Star Pattern)
    println!("Demo 18: Star Polygon (self-intersecting)");
    let star = [(110, 115), (115, 130), (105, 120), (120, 120), (110, 135)];
    draw_polygon(&mut grid, &star)?;

    // Demo 19: Rectangle Partially Off-Grid (Clipping Demo)
    println!("Demo 19: Rectangle Clipping (partially off-grid, top-left)");
    draw_rectangle(&mut grid, -5, -5, 25, 25)?;

    // Demo 20: Rectangle Partially Off-Grid (Clipping Demo, bottom-right)
    println!("Demo 20: Rectangle Clipping (partially off-grid, bottom-right)");
    draw_rectangle(&mut grid, 145, 80, 30, 30)?;

    // Demo 21: Nested Rectangles (Concentric)
    println!("Demo 21: Nested Rectangles (concentric pattern)");
    for i in 0..5 {
        let offset = (i * 3) as i32;
        let size = (40 - (i * 6)) as u32;
        draw_rectangle(&mut grid, 130 + offset, offset, size, size)?;
    }

    // Render to terminal
    println!("\n=== Rendered Output ===\n");
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    println!("\n=== Demo Complete ===");
    println!("Demonstrated:");
    println!("  - Rectangle outlines (small, medium, large)");
    println!("  - Filled rectangles (solid backgrounds, panels)");
    println!("  - Thick rectangle borders (thickness 2, 3, 5)");
    println!("  - Triangles (upright, inverted)");
    println!("  - Squares (special case of rectangle/polygon)");
    println!("  - Regular polygons (pentagon, hexagon, octagon)");
    println!("  - Filled polygons (solid triangles, hexagons)");
    println!("  - Irregular polygons (asymmetric, complex shapes)");
    println!("  - Star patterns (self-intersecting polygons)");
    println!("  - Clipping (shapes partially off-grid)");
    println!("  - Nested patterns (concentric rectangles)");

    Ok(())
}
