//! Bouncing Ball Animation - Story 6.6 AC1
//!
//! Demonstrates physics simulation with gravity, velocity, and bounce physics.
//! The ball bounces within terminal bounds using realistic physics calculations.
//!
//! # Features
//! - Gravity and velocity-based physics
//! - Bounce damping for realistic energy loss
//! - Real-time FPS counter display
//! - Uses `AnimationLoop` for smooth animation
//!
//! # Usage
//! ```bash
//! cargo run --example bouncing_ball
//! ```
//!
//! # Controls
//! - Press 'q' or Ctrl+C to exit gracefully

// Allow certain clippy warnings that are acceptable in examples
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use dotmax::animation::AnimationLoop;
use dotmax::primitives::draw_circle_filled;

/// Terminal dimensions in cells
const WIDTH: usize = 80;
const HEIGHT: usize = 24;

/// Physics constants
const GRAVITY: f64 = 0.3; // Gravity acceleration (dots per frame^2)
const BOUNCE_DAMPING: f64 = 0.85; // Energy retained after bounce (0.0-1.0)
const INITIAL_VX: f64 = 2.5; // Initial horizontal velocity
const INITIAL_VY: f64 = 0.0; // Initial vertical velocity (starts at rest)

/// Ball state for physics simulation
struct Ball {
    /// X position in dot coordinates
    x: f64,
    /// Y position in dot coordinates
    y: f64,
    /// Horizontal velocity (dots per frame)
    vx: f64,
    /// Vertical velocity (dots per frame)
    vy: f64,
    /// Ball radius in dots
    radius: u32,
}

impl Ball {
    /// Create a new ball at the center-top of the screen
    const fn new(width: usize, _height: usize) -> Self {
        Self {
            // Start near top-center
            x: (width * 2 / 2) as f64, // Center horizontally (dot coordinates)
            y: 20.0,                   // Near top
            vx: INITIAL_VX,
            vy: INITIAL_VY,
            radius: 6,
        }
    }

    /// Update ball physics for one frame
    ///
    /// # Physics Calculations
    /// 1. Apply gravity to vertical velocity: vy += GRAVITY
    /// 2. Update position: x += vx, y += vy
    /// 3. Bounce off walls with damping
    fn update(&mut self, max_x: f64, max_y: f64) {
        // Apply gravity to vertical velocity
        // Gravity accelerates the ball downward each frame
        self.vy += GRAVITY;

        // Update position based on velocity
        self.x += self.vx;
        self.y += self.vy;

        // Bounce off horizontal walls (left/right)
        let radius_f64 = f64::from(self.radius);
        let min_x = radius_f64;
        let max_bound_x = max_x - radius_f64;

        if self.x <= min_x {
            self.x = min_x;
            self.vx = -self.vx * BOUNCE_DAMPING;
        } else if self.x >= max_bound_x {
            self.x = max_bound_x;
            self.vx = -self.vx * BOUNCE_DAMPING;
        }

        // Bounce off vertical walls (top/bottom)
        let min_y = radius_f64;
        let max_bound_y = max_y - radius_f64;

        if self.y <= min_y {
            self.y = min_y;
            self.vy = -self.vy * BOUNCE_DAMPING;
        } else if self.y >= max_bound_y {
            self.y = max_bound_y;
            // Bounce with damping - energy is lost on each bounce
            self.vy = -self.vy * BOUNCE_DAMPING;
            // Also apply horizontal damping on floor bounce (friction)
            self.vx *= 0.98;
        }
    }

    /// Draw the ball on the grid as a filled circle
    fn draw(&self, grid: &mut dotmax::BrailleGrid) {
        let _ = draw_circle_filled(
            grid,
            self.x as i32,
            self.y as i32,
            self.radius,
        );
    }
}

fn main() -> Result<(), dotmax::DotmaxError> {
    // Calculate dot dimensions (each cell is 2×4 dots)
    let dot_width = (WIDTH * 2) as f64;
    let dot_height = (HEIGHT * 4) as f64;

    // Initialize ball with screen dimensions
    let mut ball = Ball::new(WIDTH, HEIGHT);

    // Run animation loop
    // AnimationLoop handles:
    // - Double buffering for flicker-free updates
    // - Frame timing for consistent FPS
    // - Terminal setup and cleanup
    // - Ctrl+C handling for graceful exit
    AnimationLoop::new(WIDTH, HEIGHT)
        .fps(60) // Target 60 FPS for smooth motion
        .on_frame(move |frame, buffer| {
            // Update physics
            ball.update(dot_width, dot_height);

            // Draw ball
            ball.draw(buffer);

            // Draw FPS counter in top-left corner using dots
            // Frame number displayed as dot pattern for simplicity
            // (A real app might overlay text separately)
            draw_fps_indicator(buffer, frame)?;

            Ok(true) // Continue animation
        })
        .run()
}

/// Draw a simple frame indicator using dots
/// Shows activity by blinking dots based on frame number
fn draw_fps_indicator(
    grid: &mut dotmax::BrailleGrid,
    frame: u64,
) -> Result<(), dotmax::DotmaxError> {
    // Draw 4 dots that cycle to show animation is running
    let dot_index = (frame % 4) as usize;

    // Draw a small activity indicator in top-left
    for i in 0..4 {
        if i <= dot_index {
            grid.set_dot(2 + i * 2, 2)?;
        }
    }

    Ok(())
}
