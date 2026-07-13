//! Simple video decode test - no terminal rendering
//!
//! This example tests video decoding without requiring a terminal.
//! Useful for verifying FFmpeg integration works correctly.
//!
//! Usage:
//! ```bash
//! cargo run --example test_video_decode --features video -- path/to/video.mp4
//! ```

use dotmax::media::{MediaPlayer, VideoPlayer};
use std::time::Instant;

fn main() -> dotmax::Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: test_video_decode <video_file>");

    println!("Opening video: {}", path);
    let mut player = VideoPlayer::new(&path)?;

    println!("Video info:");
    println!("  Resolution: {}x{}", player.width(), player.height());
    println!("  Frame rate: {:.2} fps", player.fps());
    if let Some(duration) = player.duration() {
        println!("  Duration: {:.2}s", duration.as_secs_f64());
    }
    if let Some(frames) = player.frame_count() {
        println!("  Estimated frames: {}", frames);
    }

    println!("\nDecoding frames...");
    let start = Instant::now();
    let mut frame_count = 0;

    while let Some(result) = player.next_frame() {
        let (grid, _delay) = result?;
        frame_count += 1;
        if frame_count % 10 == 0 {
            println!(
                "  Frame {}: grid {}x{}",
                frame_count,
                grid.width(),
                grid.height()
            );
        }
    }

    let elapsed = start.elapsed();
    println!("\nDecode complete!");
    println!("  Total frames: {}", frame_count);
    println!("  Time: {:.2}s", elapsed.as_secs_f64());
    println!(
        "  Decode FPS: {:.2}",
        frame_count as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}
