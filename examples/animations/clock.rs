//! Analog Clock Animation - Story 6.6 AC5
//!
//! Demonstrates a real-time analog clock face with moving hands.
//! Uses circle and line drawing primitives from Epic 4.
//!
//! # Features
//! - Real-time clock using system time
//! - Hour, minute, and second hands with different lengths/colors
//! - Clock face circle with hour markers
//! - Updates at 1 FPS for efficiency
//!
//! # Usage
//! ```bash
//! cargo run --example clock
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
use dotmax::primitives::{draw_circle, draw_line_colored};
use dotmax::{BrailleGrid, Color, TerminalRenderer};
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::Duration;

/// Terminal dimensions in cells
const WIDTH: usize = 50;
const HEIGHT: usize = 25;

/// Clock configuration
const CLOCK_RADIUS: u32 = 40; // Clock face radius in dots
const HOUR_HAND_LENGTH: f64 = 20.0; // Short hour hand
const MINUTE_HAND_LENGTH: f64 = 32.0; // Medium minute hand
const SECOND_HAND_LENGTH: f64 = 38.0; // Long second hand

/// Colors for clock elements
const HOUR_HAND_COLOR: Color = Color { r: 255, g: 255, b: 255 }; // White
const MINUTE_HAND_COLOR: Color = Color { r: 200, g: 200, b: 255 }; // Light blue
const SECOND_HAND_COLOR: Color = Color { r: 255, g: 100, b: 100 }; // Red
const MARKER_COLOR: Color = Color { r: 255, g: 255, b: 200 }; // Light yellow

/// Get current system time as (hours, minutes, seconds)
fn get_current_time() -> (u32, u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let total_secs = now.as_secs();

    // Get seconds since midnight (approximate, ignoring timezones for simplicity)
    // Real applications should use chrono or time crate
    let secs_in_day = total_secs % (24 * 60 * 60);

    let hours = ((secs_in_day / 3600) % 12) as u32; // 12-hour format
    let minutes = ((secs_in_day % 3600) / 60) as u32;
    let seconds = (secs_in_day % 60) as u32;

    (hours, minutes, seconds)
}

/// Calculate angle for clock hand
/// 12 o'clock = -PI/2 (pointing up)
/// 3 o'clock = 0
/// 6 o'clock = PI/2
/// 9 o'clock = PI
fn time_to_angle(value: u32, max_value: u32) -> f64 {
    let fraction = f64::from(value) / f64::from(max_value);
    // Start at 12 o'clock (-PI/2) and go clockwise
    (fraction * 2.0).mul_add(PI, -PI / 2.0)
}

/// Draw a clock hand from center at given angle and length
fn draw_hand(
    grid: &mut BrailleGrid,
    center_x: i32,
    center_y: i32,
    angle: f64,
    length: f64,
    color: Color,
) -> Result<(), dotmax::DotmaxError> {
    let end_x = center_x + length.mul_add(angle.cos(), 0.0) as i32;
    let end_y = center_y + length.mul_add(angle.sin(), 0.0) as i32;

    draw_line_colored(grid, center_x, center_y, end_x, end_y, color, None)?;

    Ok(())
}

/// Draw hour markers around the clock face
fn draw_hour_markers(
    grid: &mut BrailleGrid,
    center_x: i32,
    center_y: i32,
    radius: u32,
) -> Result<(), dotmax::DotmaxError> {
    let radius_f64 = f64::from(radius);
    for hour in 0..12 {
        let angle = time_to_angle(hour, 12);

        // Draw marker at outer edge
        let outer_r = radius_f64 - 2.0;
        let inner_r = radius_f64 - 6.0;

        let outer_x = center_x + outer_r.mul_add(angle.cos(), 0.0) as i32;
        let outer_y = center_y + outer_r.mul_add(angle.sin(), 0.0) as i32;
        let inner_x = center_x + inner_r.mul_add(angle.cos(), 0.0) as i32;
        let inner_y = center_y + inner_r.mul_add(angle.sin(), 0.0) as i32;

        // Draw short line for marker
        draw_line_colored(grid, inner_x, inner_y, outer_x, outer_y, MARKER_COLOR, None)?;
    }

    Ok(())
}

/// Draw center dot
fn draw_center(
    grid: &mut BrailleGrid,
    center_x: i32,
    center_y: i32,
) -> Result<(), dotmax::DotmaxError> {
    // Draw small filled area at center
    for dx in -1..=1 {
        for dy in -1..=1 {
            let x = (center_x + dx) as usize;
            let y = (center_y + dy) as usize;
            let (w, h) = grid.dimensions();
            if x < w * 2 && y < h * 4 {
                grid.set_dot(x, y)?;
            }
        }
    }
    // Color the center cell
    let cell_x = (center_x as usize) / 2;
    let cell_y = (center_y as usize) / 4;
    grid.set_cell_color(cell_x, cell_y, HOUR_HAND_COLOR)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), Hide, MoveTo(0, 0))?;

    // Create renderer and timer
    let mut renderer = TerminalRenderer::new()?;
    let mut timer = FrameTimer::new(1); // 1 FPS is sufficient for a clock

    // Calculate center in dot coordinates
    let center_x = (WIDTH * 2 / 2) as i32;
    let center_y = (HEIGHT * 4 / 2) as i32;

    // Main animation loop
    loop {
        // Check for exit key (poll for 100ms to reduce CPU usage)
        if event::poll(Duration::from_millis(100))? {
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

        // Get current time
        let (hours, minutes, seconds) = get_current_time();

        // Create fresh grid each frame
        let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;

        // Draw clock face (circle outline)
        draw_circle(&mut grid, center_x, center_y, CLOCK_RADIUS)?;

        // Draw hour markers
        draw_hour_markers(&mut grid, center_x, center_y, CLOCK_RADIUS)?;

        // Calculate hand angles
        // Hour hand moves with both hours and minutes for smooth motion
        let hour_angle = time_to_angle(hours * 5 + minutes / 12, 60);
        let minute_angle = time_to_angle(minutes, 60);
        let second_angle = time_to_angle(seconds, 60);

        // Draw hands (order: hour, minute, second so second is on top)
        draw_hand(
            &mut grid,
            center_x,
            center_y,
            hour_angle,
            HOUR_HAND_LENGTH,
            HOUR_HAND_COLOR,
        )?;
        draw_hand(
            &mut grid,
            center_x,
            center_y,
            minute_angle,
            MINUTE_HAND_LENGTH,
            MINUTE_HAND_COLOR,
        )?;
        draw_hand(
            &mut grid,
            center_x,
            center_y,
            second_angle,
            SECOND_HAND_LENGTH,
            SECOND_HAND_COLOR,
        )?;

        // Draw center pivot
        draw_center(&mut grid, center_x, center_y)?;

        // Render to terminal
        renderer.render(&grid)?;

        // Display time text below the clock
        execute!(
            stdout,
            MoveTo(0, HEIGHT as u16 + 1),
            Print(format!(
                "  Analog Clock | {hours:02}:{minutes:02}:{seconds:02} | [q]uit                    "
            ))
        )?;
        stdout.flush()?;

        timer.wait_for_next_frame();
    }

    // Cleanup
    execute!(stdout, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    renderer.cleanup()?;
    disable_raw_mode()?;

    println!("Analog Clock Demo Complete!");
    println!("\nFeatures demonstrated:");
    println!("- Circle drawing (clock face)");
    println!("- Line drawing (clock hands)");
    println!("- Color support (different hand colors)");
    println!("- Real-time system time");
    println!("- Low FPS mode (1 FPS for efficiency)");

    Ok(())
}
