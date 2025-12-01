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
//! Bouncing Ball Animation Demo
//!
//! Demonstrates the double-buffering workflow from Story 6.1:
//! 1. Clear the back buffer
//! 2. Draw the next frame to the back buffer
//! 3. Swap buffers (instant O(1) operation)
//! 4. Render the front buffer to the terminal
//! 5. Repeat with frame timing for 60 fps
//!
//! Run with: `cargo run --example animation_buffer`
//!
//! Press Ctrl+C to exit gracefully.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use dotmax::animation::FrameBuffer;
use dotmax::TerminalRenderer;
use std::time::{Duration, Instant};

/// Ball state for physics simulation
struct Ball {
    /// X position in dot coordinates
    x: f64,
    /// Y position in dot coordinates
    y: f64,
    /// X velocity (dots per frame)
    vx: f64,
    /// Y velocity (dots per frame)
    vy: f64,
    /// Ball radius in dots
    radius: f64,
}

impl Ball {
    const fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 3.0,     // Initial horizontal velocity
            vy: 2.0,     // Initial vertical velocity
            radius: 4.0, // Ball radius (2 braille cells wide)
        }
    }

    /// Update ball position with boundary bouncing
    fn update(&mut self, dot_width: f64, dot_height: f64) {
        // Update position
        self.x += self.vx;
        self.y += self.vy;

        // Bounce off left/right walls
        if self.x - self.radius < 0.0 {
            self.x = self.radius;
            self.vx = -self.vx;
        } else if self.x + self.radius >= dot_width {
            self.x = dot_width - self.radius - 1.0;
            self.vx = -self.vx;
        }

        // Bounce off top/bottom walls
        if self.y - self.radius < 0.0 {
            self.y = self.radius;
            self.vy = -self.vy;
        } else if self.y + self.radius >= dot_height {
            self.y = dot_height - self.radius - 1.0;
            self.vy = -self.vy;
        }
    }

    /// Draw the ball as a filled circle on the grid
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Ball coordinates are within terminal bounds, truncation is intentional"
    )]
    fn draw(&self, grid: &mut dotmax::BrailleGrid) {
        let cx = self.x as isize;
        let cy = self.y as isize;
        let r = self.radius as isize;

        // Draw filled circle using Bresenham-like approach
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && py >= 0 {
                        let _ = grid.set_dot(px as usize, py as usize);
                    }
                }
            }
        }
    }
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

    println!("Terminal size: {width}x{height}");
    println!("Dot resolution: {}x{}", width * 2, height * 4);
    println!("Press Ctrl+C to exit...\n");

    // Create double-buffered frame system
    let mut buffer = FrameBuffer::new(width, height);

    // Calculate dot dimensions for physics
    let dot_width = (width * 2) as f64;
    let dot_height = (height * 4) as f64;

    // Initialize ball in center of screen
    let mut ball = Ball::new(dot_width / 2.0, dot_height / 2.0);

    // Frame timing for 60 fps
    let target_frame_time = Duration::from_millis(16); // ~60 fps
    let mut frame_count: u64 = 0;
    let start_time = Instant::now();

    // Animation loop
    loop {
        let frame_start = Instant::now();

        // Check for Ctrl+C or 'q' to exit
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                if code == KeyCode::Char('q') || code == KeyCode::Esc {
                    break;
                }
            }
        }

        // ================================================================
        // DOUBLE BUFFERING WORKFLOW (Story 6.1 demonstration)
        // ================================================================

        // Step 1: Clear the back buffer
        buffer.get_back_buffer().clear();

        // Step 2: Update physics
        ball.update(dot_width, dot_height);

        // Step 3: Draw the next frame to the back buffer
        ball.draw(buffer.get_back_buffer());

        // Draw FPS counter (simple text approximation using dots)
        // We'll just draw a small indicator in the corner
        draw_frame_indicator(buffer.get_back_buffer(), frame_count);

        // Step 4: Swap buffers (instant O(1) operation)
        buffer.swap_buffers();

        // Step 5: Render the front buffer to terminal
        buffer.render(&mut renderer)?;

        // Frame timing
        frame_count += 1;
        let frame_elapsed = frame_start.elapsed();

        // Sleep to maintain target frame rate
        if frame_elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - frame_elapsed);
        }

        // Print FPS to stderr every second (doesn't interfere with terminal graphics)
        let total_elapsed = start_time.elapsed().as_secs();
        if total_elapsed > 0 && frame_count % 60 == 0 {
            let fps = frame_count as f64 / start_time.elapsed().as_secs_f64();
            eprintln!("\rFPS: {fps:.1}  Frame: {frame_count}  ");
        }
    }

    // Clean up terminal
    renderer.cleanup()?;

    // Print final stats
    let total_time = start_time.elapsed();
    let avg_fps = frame_count as f64 / total_time.as_secs_f64();
    println!("\n\nAnimation complete!");
    println!("Total frames: {frame_count}");
    println!("Total time: {:.2}s", total_time.as_secs_f64());
    println!("Average FPS: {avg_fps:.1}");

    Ok(())
}

/// Draw a simple frame indicator in the top-left corner
fn draw_frame_indicator(grid: &mut dotmax::BrailleGrid, frame: u64) {
    // Draw a small pulsing dot pattern based on frame number
    let pattern = (frame / 10) % 4;
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
