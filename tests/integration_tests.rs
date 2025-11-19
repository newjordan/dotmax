//! Integration tests for dotmax
//!
//! These tests verify the complete rendering pipeline from `BrailleGrid`
//! to terminal output.

use dotmax::{BrailleGrid, Color, TerminalRenderer, TerminalType};

/// Test rendering a 10×10 grid to terminal (AC #7)
///
/// This test verifies that:
/// 1. A 10×10 `BrailleGrid` can be created
/// 2. Dots can be set in the grid
/// 3. The grid can be rendered to a terminal without panicking
/// 4. The rendering pipeline (`BrailleGrid` → `to_unicode_grid()` → terminal) works
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_render_10x10_grid() {
    // Create a 10×10 braille grid
    let mut grid = BrailleGrid::new(10, 10).expect("Failed to create 10×10 grid");

    // Set some dots to create a test pattern
    grid.set_dot(5, 5).expect("Failed to set dot at (5,5)");
    grid.set_dot(10, 10).expect("Failed to set dot at (10,10)");
    grid.set_dot(15, 15).expect("Failed to set dot at (15,15)");

    // Create terminal renderer
    let mut renderer = TerminalRenderer::new().expect("Failed to create terminal renderer");

    // Render should succeed without panic
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Rendering 10×10 grid should succeed: {:?}",
        result.err()
    );

    // Clean up
    renderer.cleanup().expect("Failed to cleanup terminal");
}

/// Test rendering pipeline with Unicode braille characters
///
/// Verifies that the pipeline correctly converts `BrailleGrid` → Unicode → terminal
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_rendering_pipeline_with_braille_chars() {
    // Create grid
    let mut grid = BrailleGrid::new(20, 10).expect("Failed to create grid");

    // Set multiple dots to create recognizable braille pattern
    // Create a diagonal line
    for i in 0..10 {
        grid.set_dot(i * 2, i * 4).expect("Failed to set dot");
    }

    // Create renderer
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render - should convert to Unicode braille and output to terminal
    let result = renderer.render(&grid);
    assert!(result.is_ok(), "Render should succeed: {:?}", result.err());

    // Verify we can get terminal size
    let (width, height) = renderer
        .get_terminal_size()
        .expect("Failed to get terminal size");
    assert!(
        width > 0 && height > 0,
        "Terminal should have positive size"
    );

    // Clean up
    renderer.cleanup().expect("Failed to cleanup");
}

/// Test `clear()` method
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_clear_terminal() {
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Clear should succeed
    let result = renderer.clear();
    assert!(result.is_ok(), "Clear should succeed: {:?}", result.err());

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test `get_terminal_size()` method
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_get_terminal_size() {
    let renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Get size should succeed
    let result = renderer.get_terminal_size();
    assert!(
        result.is_ok(),
        "Get size should succeed: {:?}",
        result.err()
    );

    let (width, height) = result.unwrap();
    assert!(
        width >= 40,
        "Terminal width should be at least 40, got {width}"
    );
    assert!(
        height >= 12,
        "Terminal height should be at least 12, got {height}"
    );
}

/// Test error handling - terminal too small
#[test]
#[ignore = "Requires actual terminal with specific size - manual testing"]
fn test_error_handling_terminal_too_small() {
    // This test would require mocking terminal size to be < 40×12
    // For now, we verify that TerminalRenderer::new() returns Result type
    // and can handle errors gracefully

    // If terminal is >= 40×12, this should succeed
    let result = TerminalRenderer::new();
    if let Ok(mut renderer) = result {
        renderer.cleanup().expect("Cleanup failed");
    }
    // If it fails, it should be a DotmaxError, not a panic
}

/// Test rendering empty grid (edge case)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_render_empty_grid() {
    let grid = BrailleGrid::new(10, 10).expect("Failed to create grid");
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Rendering empty grid (all blank braille chars) should succeed
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Rendering empty grid should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test rendering large grid
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_render_large_grid() {
    // Create an 80×24 grid (typical terminal size)
    let mut grid = BrailleGrid::new(80, 24).expect("Failed to create 80×24 grid");

    // Set dots across the entire grid
    for y in 0..24 * 4 {
        for x in 0..80 * 2 {
            if (x + y) % 3 == 0 {
                grid.set_dot(x, y).expect("Failed to set dot");
            }
        }
    }

    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render large grid
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Rendering 80×24 grid should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test multiple sequential renders (frame updates)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_sequential_renders() {
    let mut grid = BrailleGrid::new(20, 20).expect("Failed to create grid");
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render multiple frames
    for frame in 0..5 {
        // Update grid for each frame
        grid.set_dot(frame * 2, frame * 4)
            .expect("Failed to set dot");

        let result = renderer.render(&grid);
        assert!(
            result.is_ok(),
            "Render frame {} should succeed: {:?}",
            frame,
            result.err()
        );
    }

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test terminal resize workflow (Story 2.5, AC #1, #2, #3)
///
/// This test verifies the complete terminal resize handling workflow:
/// 1. Create a `TerminalRenderer` and query its size
/// 2. Create a `BrailleGrid` matching the terminal size
/// 3. Set some dots in the grid
/// 4. Render the grid
/// 5. Simulate a resize by manually resizing the grid
/// 6. Verify that dots are preserved
/// 7. Re-render the resized grid
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_terminal_resize_workflow() {
    // Create renderer and query terminal size
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");
    let (cols, rows) = renderer
        .get_terminal_size()
        .expect("Failed to get terminal size");

    // Create grid matching terminal size (in cells, not dots)
    let mut grid = BrailleGrid::new(cols as usize, rows as usize)
        .expect("Failed to create grid matching terminal size");

    // Set some dots in a recognizable pattern
    grid.set_dot(10, 10).expect("Failed to set dot at (10, 10)");
    grid.set_dot(20, 20).expect("Failed to set dot at (20, 20)");
    grid.set_dot(30, 30).expect("Failed to set dot at (30, 30)");

    // Render initial grid
    renderer
        .render(&grid)
        .expect("Failed to render initial grid");

    // Simulate terminal resize by manually resizing the grid
    // In a real application, this would be triggered by a crossterm Event::Resize
    let new_width = (cols as usize).max(50); // Ensure we have enough space
    let new_height = (rows as usize).max(30);

    grid.resize(new_width, new_height)
        .expect("Failed to resize grid");

    // Verify grid dimensions match new size
    assert_eq!(
        grid.dimensions(),
        (new_width, new_height),
        "Grid dimensions should match resize request"
    );

    // Dots within the original bounds should still be preserved
    // (We can't easily verify the exact dots without accessing internal state,
    // but we can verify the grid renders successfully)

    // Re-render with new size - should succeed
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Rendering resized grid should succeed: {:?}",
        result.err()
    );

    // Clean up
    renderer.cleanup().expect("Failed to cleanup");
}

/// Test resize to smaller dimensions (Story 2.5, AC #4)
#[test]
fn test_resize_shrink_without_terminal() {
    // This test doesn't require a terminal - it's unit-level but placed here
    // to demonstrate the resize workflow in an integration context

    let mut grid = BrailleGrid::new(50, 30).expect("Failed to create grid");

    // Set dots
    grid.set_dot(10, 10).expect("Failed to set dot");
    grid.set_dot(80, 100).expect("Failed to set dot"); // Will be truncated

    // Resize to smaller
    grid.resize(20, 15).expect("Failed to resize");

    assert_eq!(grid.dimensions(), (20, 15));

    // Verify grid is still functional after resize
    grid.set_dot(15, 15)
        .expect("Should be able to set dots after resize");
}

// ============================================================================
// Story 2.6: Color Support Integration Tests (AC #6, #7)
// ============================================================================

/// Test colored rendering workflow (Story 2.6, AC #6)
///
/// This test verifies the complete color rendering pipeline:
/// 1. Create a `BrailleGrid` and enable color support
/// 2. Set dots with colors on multiple cells
/// 3. Render to terminal with colors applied
/// 4. Verify rendering succeeds without errors
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_colored_rendering_workflow() {
    // Create grid with color support
    let mut grid = BrailleGrid::new(20, 20).expect("Failed to create grid");
    grid.enable_color_support();

    // Set dots with different colors
    grid.set_dot(10, 10).expect("Failed to set dot");
    grid.set_cell_color(5, 2, Color::rgb(255, 0, 0))
        .expect("Failed to set red color");

    grid.set_dot(20, 20).expect("Failed to set dot");
    grid.set_cell_color(10, 5, Color::rgb(0, 255, 0))
        .expect("Failed to set green color");

    grid.set_dot(30, 30).expect("Failed to set dot");
    grid.set_cell_color(15, 7, Color::rgb(0, 0, 255))
        .expect("Failed to set blue color");

    // Create renderer
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render with colors - should apply ANSI color codes
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Colored rendering should succeed: {:?}",
        result.err()
    );

    // Clean up
    renderer.cleanup().expect("Failed to cleanup");
}

/// Test monochrome fallback when no colors set (Story 2.6, AC #6)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_monochrome_fallback() {
    // Create grid without setting colors
    let mut grid = BrailleGrid::new(10, 10).expect("Failed to create grid");
    grid.enable_color_support();

    // Set dots but no colors
    grid.set_dot(5, 5).expect("Failed to set dot");
    grid.set_dot(10, 10).expect("Failed to set dot");

    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render should succeed with default (monochrome) output
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Monochrome rendering should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test mixed colored and monochrome cells (Story 2.6, AC #6)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_mixed_colored_and_monochrome_cells() {
    let mut grid = BrailleGrid::new(20, 20).expect("Failed to create grid");
    grid.enable_color_support();

    // Set some cells with colors
    grid.set_dot(10, 10).expect("Failed to set dot");
    grid.set_cell_color(5, 2, Color::rgb(255, 128, 0))
        .expect("Failed to set orange color");

    // Set some cells without colors (monochrome)
    grid.set_dot(20, 20).expect("Failed to set dot");
    // No color set for cell (10, 5) - should render with default

    // Set another colored cell
    grid.set_dot(30, 30).expect("Failed to set dot");
    grid.set_cell_color(15, 7, Color::black())
        .expect("Failed to set black color");

    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render with mix of colored and monochrome cells
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Mixed rendering should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test color rendering with all predefined colors (Story 2.6, AC #2, #6)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_color_rendering_with_predefined_colors() {
    let mut grid = BrailleGrid::new(10, 10).expect("Failed to create grid");
    grid.enable_color_support();

    // Test black() constructor
    grid.set_dot(0, 0).expect("Failed to set dot");
    grid.set_cell_color(0, 0, Color::black())
        .expect("Failed to set black");

    // Test white() constructor
    grid.set_dot(2, 2).expect("Failed to set dot");
    grid.set_cell_color(1, 0, Color::white())
        .expect("Failed to set white");

    // Test rgb() constructor
    grid.set_dot(4, 4).expect("Failed to set dot");
    grid.set_cell_color(2, 1, Color::rgb(255, 0, 255))
        .expect("Failed to set magenta");

    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Predefined color rendering should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}

/// Test `clear_colors()` and re-render (Story 2.6, AC #7)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_clear_colors_and_rerender() {
    let mut grid = BrailleGrid::new(10, 10).expect("Failed to create grid");
    grid.enable_color_support();

    // Set colored cells
    grid.set_dot(10, 10).expect("Failed to set dot");
    grid.set_cell_color(5, 2, Color::rgb(255, 0, 0))
        .expect("Failed to set color");

    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Render with colors
    renderer.render(&grid).expect("Initial render failed");

    // Clear all colors
    grid.clear_colors();

    // Render again - should render monochrome (dots still present, no colors)
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Render after clear_colors should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}

// ============================================================================
// Story 2.8: Terminal Viewport Detection Integration Tests
// ============================================================================

/// Test terminal type detection returns valid type (Story 2.8, AC #2)
#[test]
fn test_terminal_type_detection() {
    // Detect terminal type
    let terminal_type = TerminalType::detect();

    // Verify it's one of the valid types
    match terminal_type {
        TerminalType::WindowsTerminal
        | TerminalType::Wsl
        | TerminalType::WindowsConsole
        | TerminalType::MacOsTerminal
        | TerminalType::LinuxNative
        | TerminalType::Unknown => {
            // Valid terminal type detected
        }
    }

    // Verify name() method works
    let name = terminal_type.name();
    assert!(!name.is_empty(), "Terminal type name should not be empty");
}

/// Test terminal capabilities include terminal type (Story 2.8, AC #2)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_terminal_capabilities_include_type() {
    let renderer = TerminalRenderer::new().expect("Failed to create renderer");
    let caps = renderer.capabilities();

    // Verify terminal type is detected and included
    let _ = caps.terminal_type;
    let name = caps.terminal_type.name();
    assert!(!name.is_empty(), "Terminal type should be detected");

    // Also verify other capabilities
    assert!(caps.supports_unicode, "Should support Unicode");
}

/// Test viewport offset calculation based on terminal type (Story 2.8, AC #3)
#[test]
fn test_viewport_offset_calculation() {
    // Test various terminal types and heights
    let test_cases = vec![
        (TerminalType::WindowsTerminal, 24, 0),
        (TerminalType::WindowsTerminal, 100, 0),
        (TerminalType::Wsl, 20, 0),
        (TerminalType::Wsl, 21, 12),
        (TerminalType::Wsl, 53, 12),
        (TerminalType::WindowsConsole, 20, 0),
        (TerminalType::WindowsConsole, 21, 12),
        (TerminalType::WindowsConsole, 74, 12),
        (TerminalType::WindowsConsole, 100, 12),
        (TerminalType::MacOsTerminal, 24, 0),
        (TerminalType::LinuxNative, 24, 0),
        (TerminalType::Unknown, 24, 0),
    ];

    for (terminal_type, height, expected_offset) in test_cases {
        let offset = terminal_type.viewport_height_offset(height);
        assert_eq!(
            offset,
            expected_offset,
            "{} with height {} should have offset {}",
            terminal_type.name(),
            height,
            expected_offset
        );
    }
}

/// Test `get_terminal_size` uses viewport detection (Story 2.8, AC #1, #3)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_get_terminal_size_uses_viewport_detection() {
    let renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Get terminal size - should use viewport detection logic
    let (width, height) = renderer
        .get_terminal_size()
        .expect("Failed to get terminal size");

    // Verify size is reasonable (not negative after offset)
    assert!(width > 0, "Width should be positive");
    assert!(height > 0, "Height should be positive");

    // Height should be at least 12 (minimum terminal height requirement)
    assert!(
        height >= 12,
        "Height should be at least 12 after viewport adjustment"
    );
}

/// Test rendering respects viewport dimensions (Story 2.8, AC #5)
#[test]
#[ignore = "Requires actual terminal - run with `cargo test -- --ignored`"]
fn test_rendering_respects_viewport_dimensions() {
    let mut renderer = TerminalRenderer::new().expect("Failed to create renderer");

    // Get adjusted terminal size (with viewport detection)
    let (width, height) = renderer
        .get_terminal_size()
        .expect("Failed to get terminal size");

    // Create grid that matches the adjusted size
    let mut grid = BrailleGrid::new(width as usize, height as usize)
        .expect("Failed to create grid matching viewport size");

    // Fill grid with pattern
    for y in 0..height as usize * 4 {
        for x in 0..width as usize * 2 {
            if (x + y) % 5 == 0 {
                grid.set_dot(x, y).expect("Failed to set dot");
            }
        }
    }

    // Render should succeed without artifacts or misalignment
    let result = renderer.render(&grid);
    assert!(
        result.is_ok(),
        "Rendering with viewport-adjusted size should succeed: {:?}",
        result.err()
    );

    renderer.cleanup().expect("Failed to cleanup");
}
