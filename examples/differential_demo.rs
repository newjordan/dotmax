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
//! Differential Rendering Demo - Story 6.5
//!
//! Demonstrates the I/O savings of differential rendering compared to full frame rendering.
//! Animates a bouncing ball on a static border background.
//!
//! # Usage
//! ```bash
//! cargo run --example differential_demo
//! ```
//!
//! # Controls
//! - Press 'f' to toggle between differential and full rendering
//! - Press 'q' or Ctrl+C to exit
//!
//! # What to observe
//! - The "Changed cells" counter shows how many cells are updated each frame
//! - With differential rendering, only ~1-5% of cells update (the ball position)
//! - With full rendering, all 1920 cells (80x24) are redrawn every frame
//! - I/O reduction is typically 95%+ for this animation

// Allow certain clippy warnings that are acceptable in examples

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use dotmax::animation::{DifferentialRenderer, FrameTimer};
use dotmax::{BrailleGrid, TerminalRenderer};
use std::io::{stdout, Write};
use std::time::Duration;

const WIDTH: usize = 80;
const HEIGHT: usize = 24;
const TARGET_FPS: u32 = 60;

struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: usize,
}

impl Ball {
    const fn new() -> Self {
        Self {
            x: 40.0,
            y: 48.0,
            vx: 3.0,
            vy: 2.5,
            radius: 4,
        }
    }

    fn update(&mut self) {
        // Update position
        self.x += self.vx;
        self.y += self.vy;

        // Bounce off walls (in dot coordinates)
        // Grid is WIDTH*2 x HEIGHT*4 dots
        let max_x = (WIDTH * 2 - 1) as f64;
        let max_y = (HEIGHT * 4 - 1) as f64;

        if self.x - self.radius as f64 <= 1.0 || self.x + self.radius as f64 >= max_x - 1.0 {
            self.vx = -self.vx;
            self.x = self.x.clamp(self.radius as f64 + 1.0, max_x - self.radius as f64 - 1.0);
        }
        if self.y - self.radius as f64 <= 1.0 || self.y + self.radius as f64 >= max_y - 1.0 {
            self.vy = -self.vy;
            self.y = self.y.clamp(self.radius as f64 + 1.0, max_y - self.radius as f64 - 1.0);
        }
    }

    fn draw(&self, grid: &mut BrailleGrid) {
        // Draw filled circle
        let cx = self.x as isize;
        let cy = self.y as isize;
        let r = self.radius as isize;

        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let px = (cx + dx) as usize;
                    let py = (cy + dy) as usize;
                    let _ = grid.set_dot(px, py);
                }
            }
        }
    }
}

fn draw_border(grid: &mut BrailleGrid) {
    let dot_width = WIDTH * 2;
    let dot_height = HEIGHT * 4;

    // Draw top and bottom borders
    for x in 0..dot_width {
        let _ = grid.set_dot(x, 0);
        let _ = grid.set_dot(x, dot_height - 1);
    }

    // Draw left and right borders
    for y in 0..dot_height {
        let _ = grid.set_dot(0, y);
        let _ = grid.set_dot(dot_width - 1, y);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), Hide, MoveTo(0, 0))?;

    // Create renderer - we'll use differential by default
    let mut terminal = TerminalRenderer::new()?;
    let mut diff_renderer = DifferentialRenderer::new();
    let mut timer = FrameTimer::new(TARGET_FPS);

    // Animation state
    let mut ball = Ball::new();
    let mut frame_count = 0u64;
    let mut use_differential = true;
    let mut last_changed_cells: usize;

    // Previous frame for comparison
    let mut prev_frame: Option<BrailleGrid> = None;

    // Main animation loop
    loop {
        // Check for keyboard input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if event::poll(Duration::ZERO).is_ok() => break,
                    KeyCode::Char('f') => {
                        use_differential = !use_differential;
                        diff_renderer.invalidate(); // Force full render on mode change
                    }
                    _ => {}
                }
            }
        }

        // Create new frame
        let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;

        // Draw static content (border)
        draw_border(&mut grid);

        // Update and draw ball
        ball.update();
        ball.draw(&mut grid);

        // Calculate changed cells for display
        if let Some(ref prev) = prev_frame {
            last_changed_cells = diff_renderer.count_changed_cells(&grid, prev);
        } else {
            last_changed_cells = WIDTH * HEIGHT; // First frame = all cells
        }

        // Render
        if use_differential {
            diff_renderer.render_diff(&grid, &mut terminal)?;
        } else {
            terminal.render(&grid)?;
        }

        // Store current frame for next comparison
        prev_frame = Some(grid);

        // Display stats overlay (using direct terminal writes to avoid affecting grid)
        let total_cells = WIDTH * HEIGHT;
        let reduction = if last_changed_cells > 0 {
            ((total_cells - last_changed_cells) as f64 / total_cells as f64) * 100.0
        } else {
            100.0
        };

        // Move cursor to bottom and print stats
        execute!(
            stdout,
            MoveTo(0, HEIGHT as u16),
            Print(format!(
                "Frame: {:6} | FPS: {:5.1} | Mode: {:12} | Changed: {:4}/{:4} cells | I/O Reduction: {:5.1}% | [f]toggle [q]quit",
                frame_count,
                timer.actual_fps(),
                if use_differential { "Differential" } else { "Full" },
                last_changed_cells,
                total_cells,
                reduction
            ))
        )?;
        stdout.flush()?;

        frame_count += 1;
        timer.wait_for_next_frame();
    }

    // Cleanup
    execute!(stdout, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    terminal.cleanup()?;
    disable_raw_mode()?;

    // Print summary
    println!("Differential Rendering Demo Complete!");
    println!("=====================================");
    println!("Total frames rendered: {frame_count}");
    println!("Average FPS: {:.1}", timer.actual_fps());
    println!();
    println!("Key benefits of differential rendering:");
    println!("- Only changed cells are sent to terminal");
    println!("- For typical animations with small moving objects: 60-95% I/O reduction");
    println!("- Lower CPU usage and smoother animations at high frame rates");

    Ok(())
}
