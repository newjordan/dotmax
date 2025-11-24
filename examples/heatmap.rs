//! Heatmap Visualization Example (Story 5.5)
//!
//! Demonstrates applying color schemes to intensity data to create
//! heatmap-style visualizations in the terminal.
//!
//! Run with: `cargo run --example heatmap`
//!
//! This example shows:
//! - Creating intensity data programmatically
//! - Applying different color schemes
//! - Interactive scheme switching with keyboard

use crossterm::event::{self, Event, KeyCode};
use dotmax::color::schemes::{
    blue_purple, cyan_magenta, grayscale, green_yellow, heat_map, rainbow,
};
use dotmax::{BrailleGrid, ColorScheme, TerminalRenderer};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut renderer = TerminalRenderer::new()?;
    let (width, height) = renderer.get_terminal_size()?;

    println!("Terminal size: {width}×{height}");
    println!("Heatmap demo - Press 1-6 to switch schemes, Q to quit");

    // Create grid matching terminal size
    let mut grid = BrailleGrid::new(width as usize, height as usize)?;

    // Available color schemes
    let schemes: Vec<(&str, ColorScheme)> = vec![
        ("Heat Map (Black→Red→Yellow→White)", heat_map()),
        ("Rainbow (Spectral)", rainbow()),
        ("Blue Purple", blue_purple()),
        ("Green Yellow", green_yellow()),
        ("Cyan Magenta", cyan_magenta()),
        ("Grayscale", grayscale()),
    ];
    let mut current_scheme = 0;

    // Generate intensity data (simulated heatmap data)
    let intensities = generate_intensity_data(width as usize, height as usize);

    // Initial render
    render_heatmap(&mut grid, &mut renderer, &intensities, &schemes[current_scheme])?;

    // Event loop
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('1') => current_scheme = 0,
                    KeyCode::Char('2') => current_scheme = 1,
                    KeyCode::Char('3') => current_scheme = 2,
                    KeyCode::Char('4') => current_scheme = 3,
                    KeyCode::Char('5') => current_scheme = 4,
                    KeyCode::Char('6') => current_scheme = 5,
                    _ => continue,
                }

                // Re-render with new scheme
                render_heatmap(&mut grid, &mut renderer, &intensities, &schemes[current_scheme])?;
            }
        }
    }

    // Cleanup
    renderer.cleanup()?;
    Ok(())
}

/// Generate sample intensity data representing a 2D heatmap
fn generate_intensity_data(width: usize, height: usize) -> Vec<f32> {
    let mut data = Vec::with_capacity(width * height);

    // Create a few "hot spots" at different positions
    let hotspots = [
        (0.3, 0.3, 0.4),  // x, y, radius
        (0.7, 0.6, 0.35),
        (0.5, 0.8, 0.3),
        (0.2, 0.7, 0.25),
        (0.8, 0.2, 0.3),
    ];

    for y in 0..height {
        for x in 0..width {
            #[allow(clippy::cast_precision_loss)]
            let nx = x as f32 / width as f32;
            #[allow(clippy::cast_precision_loss)]
            let ny = y as f32 / height as f32;

            // Base gradient (diagonal)
            let mut intensity = (nx + ny) / 4.0;

            // Add hotspot contributions
            for &(hx, hy, radius) in &hotspots {
                let dx = nx - hx;
                let dy = ny - hy;
                let dist = dx.hypot(dy);

                // Gaussian falloff from hotspot center
                if dist < radius {
                    let contribution = ((radius - dist) / radius).powi(2) * 0.8;
                    intensity += contribution;
                }
            }

            // Add some "wave" pattern
            intensity += (nx * 10.0).sin().mul_add((ny * 10.0).cos(), 1.0) * 0.1;

            // Clamp to valid range
            data.push(intensity.clamp(0.0, 1.0));
        }
    }

    data
}

/// Render the heatmap with a specific color scheme
fn render_heatmap(
    grid: &mut BrailleGrid,
    renderer: &mut TerminalRenderer,
    intensities: &[f32],
    scheme: &(&str, ColorScheme),
) -> Result<(), Box<dyn std::error::Error>> {
    // Clear grid and apply color scheme
    grid.clear();
    grid.apply_color_scheme(intensities, &scheme.1)?;

    // Set dots for all cells to make colors visible
    // (Without dots, colors wouldn't show in braille rendering)
    let (width, height) = grid.dimensions();
    for y in 0..height {
        for x in 0..width {
            // Set all 8 dots in each cell for full color coverage
            for dy in 0..4 {
                for dx in 0..2 {
                    let _ = grid.set_dot(x * 2 + dx, y * 4 + dy);
                }
            }
        }
    }

    // Render
    renderer.render(grid)?;

    Ok(())
}
