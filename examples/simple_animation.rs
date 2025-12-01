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
//! Simple bouncing dot animation demonstrating `AnimationLoop`.
//! Run with: `cargo run --example simple_animation`

use dotmax::animation::AnimationLoop;

fn main() -> Result<(), dotmax::DotmaxError> {
    // Create animation: 80×24 cells = 160×96 dots
    AnimationLoop::new(80, 24)
        .fps(30) // 30 FPS for smooth motion
        .on_frame(|frame, buffer| {
            // Bouncing dot: horizontal sweep, vertical bounce
            #[allow(clippy::cast_possible_truncation)]
            let x = ((frame % 160) * 3 % 160) as usize;
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let y = 48 + ((frame as f32 * 0.1).sin() * 40.0).abs() as usize;
            buffer.set_dot(x, y.min(95))?;
            // Draw a trail
            for i in 1..5 {
                let tx = x.saturating_sub(i * 3);
                buffer.set_dot(tx, y.min(95))?;
            }
            Ok(true) // Continue forever (Ctrl+C to exit)
        })
        .run()
}
