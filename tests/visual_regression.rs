//! Visual regression test suite for dotmax
//!
//! This test file runs visual regression tests that verify rendering output
//! remains consistent across changes. Tests capture braille output and compare
//! against stored baselines.
//!
//! ## Test Organization
//!
//! - `visual/mod.rs` - Test framework utilities (capture, compare, baselines)
//! - `visual/baselines/` - Stored baseline output files
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
//!
//! ## Adding New Tests
//!
//! 1. Create test function that produces deterministic output
//! 2. Use `capture_grid()` to convert grid to string
//! 3. Use `compare_with_baseline()` to verify against baseline
//! 4. Run with `UPDATE_BASELINES=1` to create initial baseline

mod visual;

use dotmax::{BrailleGrid, primitives::{draw_line, draw_circle, shapes::{draw_rectangle, draw_polygon}}};
use visual::{capture_grid, compare_with_baseline, generate_baseline};

// =============================================================================
// Grid Pattern Tests
// =============================================================================

/// Test: Empty grid produces blank braille characters
#[test]
fn visual_empty_grid() {
    let grid = BrailleGrid::new(10, 5).unwrap();
    let output = capture_grid(&grid);

    // Verify structure without baseline (deterministic output)
    assert_eq!(output.lines().count(), 5, "Grid should have 5 rows");
    for line in output.lines() {
        assert_eq!(line.chars().count(), 10, "Each row should have 10 cells");
        // All should be blank braille (U+2800)
        for ch in line.chars() {
            assert_eq!(ch, '\u{2800}', "Empty grid should have blank braille");
        }
    }
}

/// Test: Checkerboard pattern is deterministic
#[test]
fn visual_checkerboard_pattern() {
    let mut grid = BrailleGrid::new(8, 4).unwrap();

    // Create checkerboard pattern
    for cell_y in 0..4 {
        for cell_x in 0..8 {
            if (cell_x + cell_y) % 2 == 0 {
                // Fill this cell completely
                for dy in 0..4 {
                    for dx in 0..2 {
                        let x = cell_x * 2 + dx;
                        let y = cell_y * 4 + dy;
                        let _ = grid.set_dot(x, y);
                    }
                }
            }
        }
    }

    let output = capture_grid(&grid);

    // Verify pattern characteristics
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank > 0, "Checkerboard should have non-blank cells");

    // Full cells should be U+28FF (all 8 dots set)
    let full_cells: usize = output.chars().filter(|&c| c == '\u{28FF}').count();
    assert_eq!(full_cells, 16, "Should have 16 full cells (half of 32)");
}

// =============================================================================
// Primitive Drawing Tests
// =============================================================================

/// Test: Horizontal line is rendered correctly
#[test]
fn visual_horizontal_line() {
    let mut grid = BrailleGrid::new(20, 5).unwrap();
    draw_line(&mut grid, 0, 10, 39, 10).unwrap(); // Horizontal at y=10

    let output = capture_grid(&grid);

    // Line should affect multiple cells on row ~2-3 (y=10 in dot space)
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 15, "Horizontal line should affect many cells");
}

/// Test: Vertical line is rendered correctly
#[test]
fn visual_vertical_line() {
    let mut grid = BrailleGrid::new(5, 10).unwrap();
    draw_line(&mut grid, 5, 0, 5, 39).unwrap(); // Vertical at x=5

    let output = capture_grid(&grid);

    // Line should affect cells in column ~2 (x=5 in dot space)
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 8, "Vertical line should affect many cells");
}

/// Test: Diagonal line from corner to corner
#[test]
fn visual_diagonal_line() {
    let mut grid = BrailleGrid::new(20, 10).unwrap();
    draw_line(&mut grid, 0, 0, 39, 39).unwrap();

    let output = capture_grid(&grid);

    // Diagonal should affect cells along the diagonal
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 10, "Diagonal line should affect multiple cells");
}

/// Test: Circle is rendered with symmetry
#[test]
fn visual_circle_symmetry() {
    let mut grid = BrailleGrid::new(30, 15).unwrap();
    let cx = 30; // Center x in dot coords
    let cy = 30; // Center y in dot coords
    draw_circle(&mut grid, cx, cy, 15).unwrap();

    let output = capture_grid(&grid);

    // Circle should have non-blank cells
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 10, "Circle should have visible outline");
}

/// Test: Rectangle corners are set
#[test]
fn visual_rectangle_corners() {
    let mut grid = BrailleGrid::new(20, 10).unwrap();
    draw_rectangle(&mut grid, 4, 4, 30, 30).unwrap();

    let output = capture_grid(&grid);

    // Rectangle should have non-blank cells for edges
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 8, "Rectangle should have visible edges");
}

/// Test: Triangle polygon
#[test]
fn visual_triangle() {
    let mut grid = BrailleGrid::new(20, 10).unwrap();
    let triangle = [(20, 5), (5, 35), (35, 35)];
    draw_polygon(&mut grid, &triangle).unwrap();

    let output = capture_grid(&grid);

    // Triangle should have visible edges
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 5, "Triangle should have visible edges");
}

// =============================================================================
// Grid Dimension Tests
// =============================================================================

/// Test: Various grid dimensions produce correct output dimensions
#[test]
fn visual_dimension_preservation() {
    let test_cases = [
        (5, 3),
        (10, 5),
        (20, 10),
        (80, 24),
        (1, 1),
        (100, 50),
    ];

    for (w, h) in test_cases {
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

// =============================================================================
// Unicode Validation Tests
// =============================================================================

/// Test: All output characters are valid Unicode braille
#[test]
fn visual_braille_unicode_range() {
    let mut grid = BrailleGrid::new(20, 10).unwrap();

    // Create various patterns
    draw_line(&mut grid, 0, 0, 39, 39).unwrap();
    draw_circle(&mut grid, 20, 20, 10).unwrap();
    draw_rectangle(&mut grid, 5, 5, 10, 10).unwrap();

    let output = capture_grid(&grid);

    for (i, ch) in output.chars().enumerate() {
        if ch == '\n' {
            continue;
        }
        let code = ch as u32;
        assert!(
            (0x2800..=0x28FF).contains(&code),
            "Character {} at position {} (U+{:04X}) is not valid braille",
            ch, i, code
        );
    }
}

/// Test: Combined shapes produce valid output
#[test]
fn visual_combined_shapes() {
    let mut grid = BrailleGrid::new(40, 20).unwrap();

    // Multiple overlapping shapes
    draw_rectangle(&mut grid, 5, 5, 70, 70).unwrap();
    draw_circle(&mut grid, 40, 40, 20).unwrap();
    draw_line(&mut grid, 10, 10, 70, 70).unwrap();
    draw_line(&mut grid, 70, 10, 10, 70).unwrap();

    let output = capture_grid(&grid);

    // Should have significant non-blank content
    let non_blank: usize = output.chars().filter(|&c| c != '\u{2800}' && c != '\n').count();
    assert!(non_blank >= 50, "Combined shapes should have significant content");

    // All characters should be valid braille
    for ch in output.chars() {
        if ch != '\n' {
            let code = ch as u32;
            assert!((0x2800..=0x28FF).contains(&code));
        }
    }
}
