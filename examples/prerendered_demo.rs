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
//! Prerendered Animation Demo
//!
//! This example demonstrates the `PrerenderedAnimation` struct for creating
//! smooth animations with zero computation during playback.
//!
//! # Pre-rendering Advantage
//!
//! Unlike `AnimationLoop` which computes each frame on-the-fly, pre-rendered
//! animations front-load all computation:
//!
//! 1. **Pre-render phase**: All frames are generated and stored in memory
//! 2. **Playback phase**: Frames are displayed with zero computation
//!
//! This is ideal for:
//! - Loading spinners that repeat indefinitely
//! - Intro animations shown at app startup
//! - Complex graphics that would cause frame drops if computed in real-time
//!
//! # Usage
//!
//! ```bash
//! cargo run --example prerendered_demo
//! ```
//!
//! Press Ctrl+C to exit the looping animation.

use dotmax::animation::PrerenderedAnimation;
use dotmax::primitives::draw_circle;
use dotmax::{BrailleGrid, DotmaxError, TerminalRenderer};
use std::f64::consts::PI;
use std::time::Instant;

fn main() -> Result<(), DotmaxError> {
    println!("Prerendered Animation Demo");
    println!("==========================\n");

    // Grid dimensions
    let width: usize = 40;
    let height: usize = 20;
    let dot_width = width * 2; // 80 dots
    let dot_height = height * 4; // 80 dots

    // Animation parameters
    let frame_count: u32 = 60;
    let fps: u32 = 30;

    // ========================================================================
    // Phase 1: Pre-render all frames
    // ========================================================================
    println!("Phase 1: Pre-rendering {frame_count} frames...");
    let pre_render_start = Instant::now();

    let mut animation = PrerenderedAnimation::new(fps);

    // Create a spinning line animation using i32 for coordinates
    let center_x = (dot_width / 2) as i32;
    let center_y = (dot_height / 2) as i32;
    let radius = center_y.min(center_x) - 2;

    for frame in 0..frame_count {
        let mut grid = BrailleGrid::new(width, height)?;

        // Calculate angle for this frame (full rotation over all frames)
        let angle = f64::from(frame) / f64::from(frame_count) * 2.0 * PI;

        // Draw a spinning line from center using mul_add for better precision
        let end_x = angle.cos().mul_add(f64::from(radius), f64::from(center_x));
        let end_y = angle.sin().mul_add(f64::from(radius), f64::from(center_y));

        // Draw line from center to calculated endpoint
        for t in 0..=radius {
            let progress = f64::from(t) / f64::from(radius);
            let x = f64::from(center_x) + progress * (end_x - f64::from(center_x));
            let y = f64::from(center_y) + progress * (end_y - f64::from(center_y));

            // Convert to usize with bounds check
            if x >= 0.0 && y >= 0.0 {
                let x_usize = x as usize;
                let y_usize = y as usize;
                if x_usize < dot_width && y_usize < dot_height {
                    let _ = grid.set_dot(x_usize, y_usize);
                }
            }
        }

        // Draw a circle around the center
        #[allow(clippy::cast_sign_loss)]
        draw_circle(&mut grid, center_x, center_y, radius as u32)?;

        // Draw center dot
        #[allow(clippy::cast_sign_loss)]
        let _ = grid.set_dot(center_x as usize, center_y as usize);

        animation.add_frame(grid);
    }

    let pre_render_duration = pre_render_start.elapsed();
    println!(
        "Pre-render complete: {} frames in {:.2}ms ({:.1} frames/sec)",
        animation.frame_count(),
        pre_render_duration.as_secs_f64() * 1000.0,
        f64::from(frame_count) / pre_render_duration.as_secs_f64()
    );

    // Memory usage estimate
    let frame_bytes = width * height;
    let total_bytes = frame_bytes * frame_count as usize;
    println!(
        "Memory usage: ~{:.1}KB ({frame_bytes} bytes per frame)",
        total_bytes as f64 / 1024.0
    );

    println!("\nPhase 2: Playback (press Ctrl+C to stop)");
    println!("=========================================\n");
    println!("Notice: Zero computation during playback - just displaying pre-rendered frames.\n");

    // ========================================================================
    // Phase 2: Playback (zero computation)
    // ========================================================================
    let mut renderer = TerminalRenderer::new()?;

    // Play the animation in a loop (stops on Ctrl+C)
    animation.play_loop(&mut renderer)?;

    // Cleanup
    renderer.cleanup()?;

    println!("\n\nAnimation stopped. Goodbye!");

    Ok(())
}
