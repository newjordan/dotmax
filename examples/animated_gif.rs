//! Animated GIF playback example (Story 9.2)
//!
//! Demonstrates how to play animated GIFs in the terminal using dotmax.
//!
//! # Usage
//!
//! ```bash
//! # Play a GIF file (auto-detect and play)
//! cargo run --example animated_gif --features image -- path/to/animation.gif
//!
//! # Use test fixture
//! cargo run --example animated_gif --features image -- tests/fixtures/media/animated.gif
//! ```
//!
//! # Controls
//!
//! - **Any key**: Stop playback and exit
//!
//! # Features Demonstrated
//!
//! - `show_file()` for automatic format detection and playback
//! - `GifPlayer` for manual frame control
//! - `MediaPlayer` trait for polymorphic media handling

use dotmax::media::{detect_format, GifPlayer, MediaFormat, MediaPlayer};
use std::env;
use std::io::stdout;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get GIF path from command line
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("tests/fixtures/media/animated.gif");

    println!("Animated GIF Player - dotmax Story 9.2");
    println!("=====================================\n");

    // Check file exists
    if !std::path::Path::new(path).exists() {
        eprintln!("Error: File not found: {}", path);
        eprintln!("\nUsage: cargo run --example animated_gif --features image -- <gif_path>");
        std::process::exit(1);
    }

    // Detect format to confirm it's an animated GIF
    let format = detect_format(path)?;
    println!("Detected format: {}", format);

    match format {
        MediaFormat::AnimatedGif => {
            println!("✓ Animated GIF detected!\n");
            play_with_manual_control(path)?;
        }
        MediaFormat::StaticImage(_) => {
            println!("⚠ This is a static image (single frame)");
            println!("  Use show_file() for static images.");
        }
        _ => {
            println!("⚠ Not a GIF file: {}", format);
        }
    }

    Ok(())
}

/// Demonstrates manual frame control with GifPlayer
fn play_with_manual_control(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::{self, Event, KeyCode};
    use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
    use crossterm::{cursor, execute};

    let mut player = GifPlayer::new(path)?;

    println!("GIF Info:");
    println!("  Canvas: {}x{} pixels", player.canvas_width(), player.canvas_height());
    println!(
        "  Loop count: {}",
        match player.loop_count() {
            Some(0) => "infinite".to_string(),
            Some(n) => format!("{} times", n),
            None => "once".to_string(),
        }
    );
    println!("\nPress Enter to start playback (any key to stop)...");

    // Wait for Enter
    let _ = std::io::stdin().read_line(&mut String::new());

    // Enter alternate screen for clean playback
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    // Create renderer
    let mut renderer = dotmax::TerminalRenderer::new()?;

    // Play frames
    let mut frame_number = 0;
    let start_time = Instant::now();

    let result: Result<(), Box<dyn std::error::Error>> = (|| {
        loop {
            match player.next_frame() {
                Some(Ok((grid, delay))) => {
                    // Render frame
                    renderer.render(&grid)?;

                    frame_number += 1;

                    // Wait for frame duration, checking for keypress
                    let deadline = Instant::now() + delay;
                    while Instant::now() < deadline {
                        if event::poll(Duration::from_millis(10))? {
                            if let Event::Key(key) = event::read()? {
                                if !matches!(key.code, KeyCode::Modifier(_)) {
                                    // Exit on any key
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    // Log error and continue
                    eprintln!("Frame error: {:?}", e);
                }
                None => {
                    // Animation complete
                    break;
                }
            }
        }
        Ok(())
    })();

    // Restore terminal
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    // Print stats
    let elapsed = start_time.elapsed();
    println!("\nPlayback complete!");
    println!("  Frames displayed: {}", frame_number);
    println!("  Total time: {:.2?}", elapsed);
    if frame_number > 0 {
        println!(
            "  Average FPS: {:.1}",
            frame_number as f64 / elapsed.as_secs_f64()
        );
    }

    result?;
    Ok(())
}
