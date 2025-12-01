#![allow(
    clippy::uninlined_format_args,
    clippy::doc_markdown,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]
//! FPS Control Demo - Frame Timing Demonstration
//!
//! Demonstrates the `FrameTimer` API from Story 6.2:
//! - Precise frame timing control (30fps, 60fps)
//! - Real-time FPS display
//! - Frame drop detection
//! - Toggle between frame rates
//!
//! Run with: `cargo run --example fps_control`
//!
//! Controls:
//! - Press 'f' to toggle between 30fps and 60fps
//! - Press 'r' to reset timer statistics
//! - Press 'q' or Esc to exit
//! - Press Ctrl+C for graceful exit

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use dotmax::animation::{FrameBuffer, FrameTimer};
use dotmax::TerminalRenderer;
use std::io::{self, Write};

/// Simple counter animation state
struct Counter {
    value: u64,
    x_offset: usize,
}

impl Counter {
    const fn new() -> Self {
        Self {
            value: 0,
            x_offset: 0,
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "Counter value modulo screen width fits in usize"
    )]
    fn update(&mut self, width: usize) {
        self.value = self.value.wrapping_add(1);
        // Move the indicator dot across the screen
        self.x_offset = (self.value as usize / 4) % (width.saturating_sub(4));
    }

    /// Draw a moving dot indicator and frame counter pattern
    fn draw(&self, grid: &mut dotmax::BrailleGrid, frame_num: u64) {
        // Draw a moving horizontal bar (progress indicator)
        let y_pos = 8;
        for dx in 0..8 {
            let x = (self.x_offset * 2) + dx;
            if x < grid.dimensions().0 {
                let _ = grid.set_dot(x, y_pos);
                let _ = grid.set_dot(x, y_pos + 1);
            }
        }

        // Draw pulsing corner indicator based on frame
        let pattern = (frame_num / 15) % 4;
        match pattern {
            0 => {
                let _ = grid.set_dot(0, 0);
            }
            1 => {
                let _ = grid.set_dot(0, 0);
                let _ = grid.set_dot(1, 0);
            }
            2 => {
                let _ = grid.set_dot(0, 0);
                let _ = grid.set_dot(1, 0);
                let _ = grid.set_dot(0, 1);
            }
            3 => {
                let _ = grid.set_dot(0, 0);
                let _ = grid.set_dot(1, 0);
                let _ = grid.set_dot(0, 1);
                let _ = grid.set_dot(1, 1);
            }
            _ => {}
        }
    }
}

/// Display status information via stderr (doesn't interfere with terminal graphics)
fn display_status(
    timer: &FrameTimer,
    frame_count: u64,
    target_fps: u32,
) {
    let actual_fps = timer.actual_fps();
    let frame_time_ms = timer.frame_time().as_secs_f64() * 1000.0;
    let target_ms = timer.target_frame_time().as_secs_f64() * 1000.0;

    // Use carriage return to update in place
    eprint!(
        "\rFrame: {frame_count:6} | Target: {target_fps}fps ({target_ms:.1}ms) | Actual: {actual_fps:.1}fps ({frame_time_ms:.2}ms)  "
    );
    let _ = io::stderr().flush();
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Terminal dimensions fit in f64 mantissa, precision loss is negligible"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal renderer
    let mut renderer = TerminalRenderer::new()?;

    // Get terminal size and create appropriately sized buffer
    let (term_width, term_height) = renderer.get_terminal_size()?;
    let width = term_width as usize;
    let height = term_height as usize;

    // Display startup information
    eprintln!("FPS Control Demo - Story 6.2: Frame Timing and Rate Control");
    eprintln!("Terminal size: {width}x{height}");
    eprintln!("Controls: [f] Toggle 30/60fps  [r] Reset timer  [q/Esc] Exit\n");

    // Create double-buffered frame system (Story 6.1)
    let mut buffer = FrameBuffer::new(width, height);

    // Create frame timer targeting 60fps (Story 6.2)
    let mut current_fps: u32 = 60;
    let mut timer = FrameTimer::new(current_fps);

    // Animation state
    let mut counter = Counter::new();
    let mut frame_count: u64 = 0;

    // Animation loop
    loop {
        // Check for keyboard input
        if event::poll(std::time::Duration::from_millis(0))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    // Ctrl+C exit
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    // 'q' or Esc to exit
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    // 'f' to toggle FPS
                    KeyCode::Char('f') => {
                        current_fps = if current_fps == 60 { 30 } else { 60 };
                        timer = FrameTimer::new(current_fps);
                        eprintln!("\n>>> Switched to {current_fps}fps");
                    }
                    // 'r' to reset timer statistics
                    KeyCode::Char('r') => {
                        timer.reset();
                        eprintln!("\n>>> Timer reset");
                    }
                    _ => {}
                }
            }
        }

        // ================================================================
        // FRAME TIMING WORKFLOW (Story 6.2 demonstration)
        // ================================================================

        // Step 1: Clear the back buffer
        buffer.get_back_buffer().clear();

        // Step 2: Update animation state
        counter.update(width);

        // Step 3: Draw to back buffer
        counter.draw(buffer.get_back_buffer(), frame_count);

        // Step 4: Swap buffers (instant O(1) operation from Story 6.1)
        buffer.swap_buffers();

        // Step 5: Render the front buffer to terminal
        buffer.render(&mut renderer)?;

        // Step 6: Wait for next frame using FrameTimer (Story 6.2)
        // This blocks until the target frame time has elapsed
        timer.wait_for_next_frame();

        // Update frame counter and display status
        frame_count += 1;

        // Display FPS info every frame (stderr doesn't interfere with terminal graphics)
        if frame_count % 5 == 0 {
            display_status(&timer, frame_count, current_fps);
        }
    }

    // Clean up terminal
    renderer.cleanup()?;

    // Print final statistics
    eprintln!("\n\n--- Final Statistics ---");
    eprintln!("Total frames rendered: {frame_count}");
    eprintln!("Final FPS measurement: {:.1}", timer.actual_fps());
    eprintln!("Target frame time: {:.2}ms", timer.target_frame_time().as_secs_f64() * 1000.0);
    eprintln!("Last frame time: {:.2}ms", timer.frame_time().as_secs_f64() * 1000.0);

    Ok(())
}
