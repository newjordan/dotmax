//! Logging demonstration example
//!
//! This example demonstrates how to enable and use tracing logging with dotmax.
//!
//! Run with:
//!   cargo run --example `logging_demo`
//!
//! To see different log levels:
//!   `RUST_LOG=dotmax=trace` cargo run --example `logging_demo`
//!   `RUST_LOG=dotmax=debug` cargo run --example `logging_demo`
//!   `RUST_LOG=dotmax=info` cargo run --example `logging_demo`

use dotmax::{BrailleGrid, Color, TerminalRenderer};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing subscriber
    // This is required for dotmax logging to work
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    println!("=== Dotmax Logging Demonstration ===\n");
    println!("This example shows tracing logs from dotmax operations.");
    println!("Look for logs with target 'dotmax::grid' and 'dotmax::render'\n");
    println!("Starting demo in 2 seconds...\n");

    thread::sleep(Duration::from_secs(2));

    // Step 1: Create a BrailleGrid
    // This will emit INFO log: "Creating BrailleGrid"
    println!("\n[Step 1] Creating BrailleGrid (80×24)...");
    let mut grid = BrailleGrid::new(80, 24)?;
    println!("✓ Grid created successfully\n");

    thread::sleep(Duration::from_secs(1));

    // Step 2: Set some dots
    // Hot path operations (set_dot) do NOT log at DEBUG level per AC #4
    println!("[Step 2] Setting dots (no debug logs from set_dot - hot path)...");
    for y in 0..20 {
        for x in 0..40 {
            if (x + y) % 3 == 0 {
                grid.set_dot(x, y)?;
            }
        }
    }
    println!("✓ Set 200+ dots (no performance impact from logging)\n");

    thread::sleep(Duration::from_secs(1));

    // Step 3: Enable color support
    // This will emit DEBUG log: "Enabling color support"
    println!("[Step 3] Enabling color support...");
    grid.enable_color_support();
    println!("✓ Color support enabled\n");

    thread::sleep(Duration::from_secs(1));

    // Step 4: Set cell colors
    println!("[Step 4] Setting cell colors...");
    grid.set_cell_color(5, 5, Color::rgb(255, 0, 0))?; // Red
    grid.set_cell_color(10, 10, Color::rgb(0, 255, 0))?; // Green
    grid.set_cell_color(15, 15, Color::rgb(0, 0, 255))?; // Blue
    println!("✓ Colors set for 3 cells\n");

    thread::sleep(Duration::from_secs(1));

    // Step 5: Resize grid
    // This will emit DEBUG log: "Resizing BrailleGrid"
    println!("[Step 5] Resizing grid from 80×24 to 100×30...");
    grid.resize(100, 30)?;
    println!("✓ Grid resized successfully\n");

    thread::sleep(Duration::from_secs(1));

    // Step 6: Clear grid
    // This will emit DEBUG log: "Clearing all dots in grid"
    println!("[Step 6] Clearing grid...");
    grid.clear();
    println!("✓ Grid cleared\n");

    thread::sleep(Duration::from_secs(1));

    // Step 7: Initialize terminal renderer
    // This will emit INFO log: "Terminal renderer initialized successfully"
    println!("[Step 7] Initializing TerminalRenderer...");
    let mut renderer = TerminalRenderer::new()?;
    println!("✓ Terminal renderer initialized\n");

    thread::sleep(Duration::from_secs(1));

    // Step 8: Query terminal size
    // This will emit DEBUG log: "Queried terminal size"
    println!("[Step 8] Querying terminal size...");
    let (width, height) = renderer.get_terminal_size()?;
    println!("✓ Terminal size: {width}×{height}\n");

    thread::sleep(Duration::from_secs(1));

    // Step 9: Set more dots for rendering
    println!("[Step 9] Drawing pattern for rendering...");
    for y in 0..40 {
        for x in 0..80 {
            if (x / 4 + y / 4) % 2 == 0 {
                grid.set_dot(x, y)?;
            }
        }
    }
    println!("✓ Pattern drawn\n");

    thread::sleep(Duration::from_secs(1));

    // Step 10: Render to terminal
    // This will emit DEBUG log: "Rendering BrailleGrid to terminal"
    println!("[Step 10] Rendering grid to terminal...");
    renderer.render(&grid)?;
    println!("\n✓ Rendered to terminal (check above for braille output)\n");

    thread::sleep(Duration::from_secs(2));

    // Step 11: Demonstrate error logging
    println!("[Step 11] Demonstrating error logging with invalid operations...\n");

    // Invalid dimensions - will emit ERROR log
    println!("  Attempting to create grid with zero dimensions...");
    #[allow(clippy::ignored_unit_patterns)]
    match BrailleGrid::new(0, 0) {
        Ok(_) => println!("  ✗ Should have failed"),
        Err(e) => println!("  ✓ Got expected error: {e}\n"),
    }

    thread::sleep(Duration::from_secs(1));

    // Out of bounds access - will emit ERROR log
    println!("  Attempting out-of-bounds dot access...");
    #[allow(clippy::ignored_unit_patterns)]
    match grid.set_dot(10000, 10000) {
        Ok(_) => println!("  ✗ Should have failed"),
        Err(e) => println!("  ✓ Got expected error: {e}\n"),
    }

    thread::sleep(Duration::from_secs(1));

    // Out of bounds color assignment - will emit ERROR log
    println!("  Attempting out-of-bounds color assignment...");
    #[allow(clippy::ignored_unit_patterns)]
    match grid.set_cell_color(10000, 10000, Color::rgb(255, 255, 255)) {
        Ok(_) => println!("  ✗ Should have failed"),
        Err(e) => println!("  ✓ Got expected error: {e}\n"),
    }

    thread::sleep(Duration::from_secs(2));

    // Cleanup
    println!("[Cleanup] Restoring terminal...");
    renderer.cleanup()?;
    println!("✓ Terminal restored\n");

    println!("=== Demo Complete ===");
    println!("\nKey observations:");
    println!("1. INFO logs for major operations (grid creation, renderer init)");
    println!("2. DEBUG logs for detailed flow (resize, clear, render)");
    println!("3. ERROR logs for failures (invalid dimensions, out-of-bounds)");
    println!("4. NO DEBUG logs from hot paths (set_dot) - zero performance impact");
    println!("\nTry running with different log levels:");
    println!("  RUST_LOG=dotmax=info cargo run --example logging_demo");
    println!("  RUST_LOG=dotmax=debug cargo run --example logging_demo");
    println!("  RUST_LOG=dotmax=trace cargo run --example logging_demo\n");

    Ok(())
}
