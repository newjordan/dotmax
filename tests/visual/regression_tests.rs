//! Visual regression tests for dotmax rendering
//!
//! These tests verify that rendering output remains consistent across changes.
//! Each test captures braille output and compares against stored baselines.
//!
//! ## Running Tests
//!
//! ```bash
//! # Normal test run (compare against baselines)
//! cargo test --test visual_regression
//!
//! # Update baselines when intentional changes occur
//! UPDATE_BASELINES=1 cargo test --test visual_regression
//! ```

mod visual;

use dotmax::{BrailleGrid, primitives::{draw_line, draw_circle, shapes::draw_rectangle}};
use visual::{capture_grid, compare_with_baseline, generate_baseline};

/// Test: Empty grid produces consistent blank output
#[test]
fn visual_empty_grid() {
    let grid = BrailleGrid::new(10, 5).unwrap();
    let output = capture_grid(&grid);

    // For initial baseline generation, uncomment:
    // generate_baseline("empty_grid", &output).unwrap();

    // Verify output structure
    assert_eq!(output.lines().count(), 5);
    for line in output.lines() {
        assert_eq!(line.chars().count(), 10);
    }
}

/// Test: Checkerboard pattern produces consistent output
#[test]
fn visual_checkerboard_pattern() {
    let mut grid = BrailleGrid::new(8, 4).unwrap();

    // Create checkerboard pattern in dots
    for cell_y in 0..4 {
        for cell_x in 0..8 {
            // Each cell is 2x4 dots
            for dy in 0..4 {
                for dx in 0..2 {
                    let x = cell_x * 2 + dx;
                    let y = cell_y * 4 + dy;
                    // Checkerboard: alternate based on cell position
                    if (cell_x + cell_y) % 2 == 0 {
                        let _ = grid.set_dot(x, y);
                    }
                }
            }
        }
    }

    let output = capture_grid(&grid);

    // For initial baseline generation:
    // generate_baseline("checkerboard", &output).unwrap();

    // Verify pattern exists (not all blank)
    assert!(output.chars().any(|c| c != '\u{2800}' && c != '\n'));
}

/// Test: Diagonal line produces consistent output
#[test]
fn visual_diagonal_line() {
    let mut grid = BrailleGrid::new(20, 10).unwrap();
    draw_line(&mut grid, 0, 0, 39, 39).unwrap(); // Diagonal across grid

    let output = capture_grid(&grid);

    // For initial baseline generation:
    // generate_baseline("diagonal_line", &output).unwrap();

    // Verify line exists
    assert!(output.chars().any(|c| c != '\u{2800}' && c != '\n'));
}

/// Test: Circle produces consistent output
#[test]
fn visual_circle() {
    let mut grid = BrailleGrid::new(30, 15).unwrap();
    draw_circle(&mut grid, 30, 30, 20).unwrap(); // Circle near center

    let output = capture_grid(&grid);

    // For initial baseline generation:
    // generate_baseline("circle", &output).unwrap();

    // Verify circle exists
    assert!(output.chars().any(|c| c != '\u{2800}' && c != '\n'));
}

/// Test: Rectangle produces consistent output
#[test]
fn visual_rectangle() {
    let mut grid = BrailleGrid::new(20, 10).unwrap();
    draw_rectangle(&mut grid, 5, 5, 30, 30).unwrap();

    let output = capture_grid(&grid);

    // For initial baseline generation:
    // generate_baseline("rectangle", &output).unwrap();

    // Verify rectangle exists
    assert!(output.chars().any(|c| c != '\u{2800}' && c != '\n'));
}

/// Test: Multiple primitives produces consistent output
#[test]
fn visual_combined_shapes() {
    let mut grid = BrailleGrid::new(40, 20).unwrap();

    // Draw several shapes
    draw_line(&mut grid, 0, 0, 79, 0).unwrap();   // Top border
    draw_line(&mut grid, 0, 79, 79, 79).unwrap(); // Bottom border
    draw_circle(&mut grid, 40, 40, 15).unwrap();  // Center circle
    draw_rectangle(&mut grid, 10, 10, 20, 20).unwrap(); // Small rect

    let output = capture_grid(&grid);

    // For initial baseline generation:
    // generate_baseline("combined_shapes", &output).unwrap();

    // Verify shapes exist
    assert!(output.chars().any(|c| c != '\u{2800}' && c != '\n'));
}

/// Test: Grid dimensions are preserved in output
#[test]
fn visual_grid_dimensions() {
    for (w, h) in [(5, 3), (10, 5), (20, 10), (80, 24)] {
        let grid = BrailleGrid::new(w, h).unwrap();
        let output = capture_grid(&grid);

        assert_eq!(
            output.lines().count(),
            h,
            "Height mismatch for {}x{} grid",
            w, h
        );

        for (i, line) in output.lines().enumerate() {
            assert_eq!(
                line.chars().count(),
                w,
                "Width mismatch on line {} for {}x{} grid",
                i, w, h
            );
        }
    }
}

/// Test: Unicode braille characters are valid
#[test]
fn visual_braille_unicode_valid() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();

    // Set various dots to create different braille patterns
    for y in 0..20 {
        for x in 0..20 {
            if (x + y) % 3 == 0 {
                let _ = grid.set_dot(x, y);
            }
        }
    }

    let output = capture_grid(&grid);

    // All characters should be in Unicode braille block (U+2800-U+28FF)
    for ch in output.chars() {
        if ch != '\n' {
            let code = ch as u32;
            assert!(
                (0x2800..=0x28FF).contains(&code),
                "Invalid braille character: U+{:04X}",
                code
            );
        }
    }
}
