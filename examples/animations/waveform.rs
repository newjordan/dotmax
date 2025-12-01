//! Waveform Visualization Animation - Story 6.6 AC3
//!
//! Demonstrates animated sine wave visualization with scrolling animation.
//! Uses line drawing primitives from Epic 4 and color schemes from Epic 5.
//!
//! # Features
//! - Animated sine wave with scrolling phase shift
//! - Multiple overlapping waves with different frequencies
//! - Uses color schemes for visual appeal
//! - Demonstrates line drawing primitives
//!
//! # Usage
//! ```bash
//! cargo run --example waveform
//! ```
//!
//! # Controls
//! - Press 'q' or Ctrl+C to exit gracefully

// Allow certain clippy warnings that are acceptable in examples
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use dotmax::animation::FrameTimer;
use dotmax::color::schemes::rainbow;
use dotmax::primitives::draw_line_colored;
use dotmax::{BrailleGrid, Color, TerminalRenderer};
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::Duration;

/// Terminal dimensions in cells
const WIDTH: usize = 80;
const HEIGHT: usize = 24;

/// Wave configuration
const PRIMARY_AMPLITUDE: f64 = 35.0; // Primary wave amplitude in dots
const PRIMARY_FREQUENCY: f64 = 0.03; // Primary wave frequency
const SECONDARY_AMPLITUDE: f64 = 15.0; // Secondary wave amplitude
const SECONDARY_FREQUENCY: f64 = 0.07; // Secondary wave frequency (higher)
const SCROLL_SPEED: f64 = 0.15; // Phase shift per frame
const TARGET_FPS: u32 = 30;

/// Calculate wave Y position at given X with phase offset
fn wave_y(x: f64, phase: f64, amplitude: f64, frequency: f64, center_y: f64) -> f64 {
    amplitude.mul_add((x.mul_add(frequency, phase)).sin(), center_y)
}

/// Draw a smooth wave using connected line segments
fn draw_wave(
    grid: &mut BrailleGrid,
    phase: f64,
    amplitude: f64,
    frequency: f64,
    center_y: f64,
    color: Color,
) -> Result<(), dotmax::DotmaxError> {
    let dot_width = WIDTH * 2;

    // Draw wave as connected line segments for smoothness
    let mut prev_x: Option<i32> = None;
    let mut prev_y: Option<i32> = None;

    for x in 0..dot_width {
        let y = wave_y(x as f64, phase, amplitude, frequency, center_y);

        // Clamp Y to grid bounds
        let dot_height = HEIGHT * 4;
        let clamped_y = y.clamp(0.0, (dot_height - 1) as f64) as i32;

        // Draw line segment from previous point to current point
        if let (Some(px), Some(py)) = (prev_x, prev_y) {
            draw_line_colored(grid, px, py, x as i32, clamped_y, color, None)?;
        }

        prev_x = Some(x as i32);
        prev_y = Some(clamped_y);
    }

    Ok(())
}

/// Draw horizontal grid lines for reference
fn draw_grid_lines(grid: &mut BrailleGrid) -> Result<(), dotmax::DotmaxError> {
    let dot_width = WIDTH * 2;
    let dot_height = HEIGHT * 4;
    let center_y = dot_height / 2;

    // Draw center line (axis)
    for x in (0..dot_width).step_by(4) {
        grid.set_dot(x, center_y)?;
    }

    // Draw top and bottom reference lines
    for x in (0..dot_width).step_by(8) {
        grid.set_dot(x, 4)?;
        grid.set_dot(x, dot_height - 5)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), Hide, MoveTo(0, 0))?;

    // Create renderer and timer
    let mut renderer = TerminalRenderer::new()?;
    let mut timer = FrameTimer::new(TARGET_FPS);

    // Get color scheme for visual appeal
    let scheme = rainbow();

    // Animation state
    let mut frame: u64 = 0;
    let center_y = (HEIGHT * 4 / 2) as f64;

    // Main animation loop
    loop {
        // Check for exit key
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Esc
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    break;
                }
            }
        }

        // Create fresh grid each frame
        let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;

        // Calculate scrolling phase
        let phase = frame as f64 * SCROLL_SPEED;

        // Draw background grid lines
        draw_grid_lines(&mut grid)?;

        // Draw primary wave (blue-ish color from rainbow)
        let primary_color = scheme.sample(0.6); // Blue from rainbow
        draw_wave(
            &mut grid,
            phase,
            PRIMARY_AMPLITUDE,
            PRIMARY_FREQUENCY,
            center_y,
            primary_color,
        )?;

        // Draw secondary wave (overlapping, different frequency)
        // Use a contrasting color from the scheme
        let secondary_color = scheme.sample(0.0); // Red from rainbow
        draw_wave(
            &mut grid,
            phase * 1.5, // Slightly different phase speed
            SECONDARY_AMPLITUDE,
            SECONDARY_FREQUENCY,
            center_y,
            secondary_color,
        )?;

        // Draw third wave for visual interest (green)
        let tertiary_color = scheme.sample(0.33); // Green from rainbow
        draw_wave(
            &mut grid,
            phase * 0.7,
            PRIMARY_AMPLITUDE * 0.5,
            PRIMARY_FREQUENCY * 2.0,
            center_y,
            tertiary_color,
        )?;

        // Render to terminal
        renderer.render(&grid)?;

        // Display status text below the braille grid
        execute!(
            stdout,
            MoveTo(0, HEIGHT as u16 + 1),
            Print(format!(
                "Waveform Demo | Frame: {:5} | Phase: {:6.2} | FPS: {:5.1} | [q]uit     ",
                frame,
                phase % (2.0 * PI),
                timer.actual_fps()
            ))
        )?;
        stdout.flush()?;

        frame += 1;
        timer.wait_for_next_frame();
    }

    // Cleanup
    execute!(stdout, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    renderer.cleanup()?;
    disable_raw_mode()?;

    println!("Waveform Animation Complete!");
    println!("Rendered {frame} frames of scrolling waveforms.");
    println!("\nFeatures demonstrated:");
    println!("- Line drawing primitives (draw_line_colored)");
    println!("- Color schemes from Epic 5 (rainbow)");
    println!("- Sine wave physics with phase shifting");
    println!("- Multiple overlapping waves");

    Ok(())
}
