//! Loading Spinner Animation - Story 6.6 AC2
//!
//! Demonstrates rotating loading indicators using braille dots.
//! Shows multiple spinner styles with consistent rotation speed.
//!
//! # Features
//! - Multiple spinner styles (dot, arc, full circle)
//! - Uses `FrameTimer` for consistent 10 FPS rotation
//! - Shows "Loading..." text effect with animated spinner
//! - Cycles through spinner styles every few seconds
//!
//! # Usage
//! ```bash
//! cargo run --example loading_spinner
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
use dotmax::{BrailleGrid, TerminalRenderer};
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::Duration;

/// Terminal dimensions in cells
const WIDTH: usize = 40;
const HEIGHT: usize = 12;

/// Spinner configuration
const SPINNER_RADIUS: f64 = 15.0;
const ROTATION_FPS: u32 = 10; // Rotation speed

/// Spinner style enumeration
#[derive(Clone, Copy)]
enum SpinnerStyle {
    /// Single rotating dot
    Dot,
    /// Rotating arc (quarter circle)
    Arc,
    /// Full rotating circle with trailing dots
    Circle,
}

impl SpinnerStyle {
    /// Get the next style in sequence
    const fn next(self) -> Self {
        match self {
            Self::Dot => Self::Arc,
            Self::Arc => Self::Circle,
            Self::Circle => Self::Dot,
        }
    }

    /// Get style name for display
    const fn name(self) -> &'static str {
        match self {
            Self::Dot => "Dot",
            Self::Arc => "Arc",
            Self::Circle => "Circle",
        }
    }
}

/// Draw a spinner at the given center with the specified style
fn draw_spinner(
    grid: &mut BrailleGrid,
    center_x: usize,
    center_y: usize,
    angle: f64,
    style: SpinnerStyle,
) -> Result<(), dotmax::DotmaxError> {
    match style {
        SpinnerStyle::Dot => {
            // Single rotating dot
            draw_dot_at_angle(grid, center_x, center_y, SPINNER_RADIUS, angle)?;
            // Add a small trail
            for i in 1..=3 {
                let trail_angle = f64::from(i).mul_add(-0.3, angle);
                let trail_radius = SPINNER_RADIUS - f64::from(i);
                if trail_radius > 0.0 {
                    draw_dot_at_angle(grid, center_x, center_y, trail_radius, trail_angle)?;
                }
            }
        }
        SpinnerStyle::Arc => {
            // Quarter circle arc that rotates
            for i in 0..8 {
                let arc_angle = f64::from(i).mul_add(PI / 16.0, angle);
                draw_dot_at_angle(grid, center_x, center_y, SPINNER_RADIUS, arc_angle)?;
            }
        }
        SpinnerStyle::Circle => {
            // Full circle with intensity gradient
            let num_dots: i32 = 16;
            for i in 0..num_dots {
                let dot_angle = (f64::from(i) * 2.0 * PI / f64::from(num_dots)) + angle;
                // Vary radius slightly for visual interest
                let r = SPINNER_RADIUS - (f64::from(i) * 0.3).min(5.0);
                if r > 0.0 {
                    draw_dot_at_angle(grid, center_x, center_y, r, dot_angle)?;
                }
            }
        }
    }
    Ok(())
}

/// Draw a single dot at a given angle and radius from center
fn draw_dot_at_angle(
    grid: &mut BrailleGrid,
    center_x: usize,
    center_y: usize,
    radius: f64,
    angle: f64,
) -> Result<(), dotmax::DotmaxError> {
    let x = radius.mul_add(angle.cos(), center_x as f64);
    let y = radius.mul_add(angle.sin(), center_y as f64);

    // Only draw if within bounds
    let (width, height) = grid.dimensions();
    let dot_width = width * 2;
    let dot_height = height * 4;

    if x >= 0.0 && x < dot_width as f64 && y >= 0.0 && y < dot_height as f64 {
        grid.set_dot(x as usize, y as usize)?;
    }
    Ok(())
}

/// Draw "Loading..." text pattern using braille dots
fn draw_loading_text(
    grid: &mut BrailleGrid,
    frame: u64,
) -> Result<(), dotmax::DotmaxError> {
    // Simple dot pattern at the bottom that pulses
    let (width, _height) = grid.dimensions();
    let dot_width = width * 2;
    let base_y = 44; // Near bottom

    // Draw dots that cycle to simulate "loading"
    let num_dots: usize = 6;
    let active_dots = ((frame / 3) % (num_dots as u64 + 1)) as usize;

    let start_x = dot_width / 2 - num_dots * 2;
    for i in 0..num_dots {
        if i < active_dots {
            grid.set_dot(start_x + i * 4, base_y)?;
            grid.set_dot(start_x + i * 4 + 1, base_y)?;
        }
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
    let mut timer = FrameTimer::new(ROTATION_FPS);

    // Animation state
    let mut frame: u64 = 0;
    let mut style = SpinnerStyle::Dot;
    let style_change_interval = 50; // Change style every 50 frames (5 seconds at 10 FPS)

    // Calculate center in dot coordinates
    let center_x = WIDTH * 2 / 2; // Center of grid in dots
    let center_y = HEIGHT * 4 / 2 - 4; // Slightly above center

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

        // Calculate rotation angle from frame number
        // 36 degrees per frame = full rotation in 10 frames
        let angle = (frame as f64 * 36.0).to_radians();

        // Draw spinner
        draw_spinner(&mut grid, center_x, center_y, angle, style)?;

        // Draw loading indicator
        draw_loading_text(&mut grid, frame)?;

        // Render to terminal
        renderer.render(&grid)?;

        // Display status text below the braille grid
        execute!(
            stdout,
            MoveTo(0, HEIGHT as u16 + 1),
            Print(format!(
                "Style: {:8} | Frame: {:4} | FPS: {:5.1} | [q]uit",
                style.name(),
                frame,
                timer.actual_fps()
            ))
        )?;
        stdout.flush()?;

        // Cycle through styles
        if frame > 0 && frame % style_change_interval == 0 {
            style = style.next();
        }

        frame += 1;
        timer.wait_for_next_frame();
    }

    // Cleanup
    execute!(stdout, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    renderer.cleanup()?;
    disable_raw_mode()?;

    println!("Loading Spinner Demo Complete!");
    println!("Demonstrated {frame} frames across all spinner styles.");

    Ok(())
}
