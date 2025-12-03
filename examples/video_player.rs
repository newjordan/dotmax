//! Video playback example using FFmpeg.
//!
//! This example demonstrates video file playback in the terminal with
//! configurable render settings.
//!
//! # Usage
//!
//! ```bash
//! # Basic playback (uses defaults)
//! cargo run --example video_player --features video -- video.mp4
//!
//! # With custom render settings
//! cargo run --example video_player --features video -- video.mp4 --dither bayer --brightness 1.2
//! ```
//!
//! # Render Settings
//!
//! Use `render_tuner` to interactively discover optimal settings:
//! ```bash
//! cargo run --example render_tuner --features image,video -- video.mp4
//! ```
//!
//! # Options
//!
//! - `--dither <method>`: none, floyd (default), bayer, atkinson
//! - `--threshold <0-255>`: Manual threshold (omit for auto Otsu)
//! - `--brightness <float>`: Brightness multiplier (default: 1.0)
//! - `--contrast <float>`: Contrast multiplier (default: 1.0)
//! - `--gamma <float>`: Gamma correction (default: 1.0)
//! - `--loop`: Loop video playback
//!
//! # Requirements
//!
//! - The `video` feature must be enabled
//! - FFmpeg libraries must be installed on your system:
//!   - Linux: `sudo apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev`
//!   - macOS: `brew install ffmpeg`
//!   - Windows: FFmpeg binaries must be in PATH

use dotmax::image::DitheringMethod;
use dotmax::media::{MediaPlayer, VideoPlayer};
use dotmax::TerminalRenderer;
use std::env;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::io::stdout;

/// Parsed command line options
struct Options {
    video_path: String,
    dithering: DitheringMethod,
    threshold: Option<u8>,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    loop_playback: bool,
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Missing video path".to_string());
    }

    let mut opts = Options {
        video_path: String::new(),
        dithering: DitheringMethod::FloydSteinberg,
        threshold: None,
        brightness: 1.0,
        contrast: 1.0,
        gamma: 1.0,
        loop_playback: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dither" => {
                i += 1;
                if i >= args.len() {
                    return Err("--dither requires a value".to_string());
                }
                opts.dithering = match args[i].to_lowercase().as_str() {
                    "none" => DitheringMethod::None,
                    "floyd" | "floydsteinberg" | "floyd-steinberg" => DitheringMethod::FloydSteinberg,
                    "bayer" => DitheringMethod::Bayer,
                    "atkinson" => DitheringMethod::Atkinson,
                    other => return Err(format!("Unknown dither method: {}", other)),
                };
            }
            "--threshold" => {
                i += 1;
                if i >= args.len() {
                    return Err("--threshold requires a value".to_string());
                }
                let val: u8 = args[i]
                    .parse()
                    .map_err(|_| "Invalid threshold value (0-255)")?;
                opts.threshold = Some(val);
            }
            "--brightness" => {
                i += 1;
                if i >= args.len() {
                    return Err("--brightness requires a value".to_string());
                }
                opts.brightness = args[i]
                    .parse()
                    .map_err(|_| "Invalid brightness value")?;
            }
            "--contrast" => {
                i += 1;
                if i >= args.len() {
                    return Err("--contrast requires a value".to_string());
                }
                opts.contrast = args[i]
                    .parse()
                    .map_err(|_| "Invalid contrast value")?;
            }
            "--gamma" => {
                i += 1;
                if i >= args.len() {
                    return Err("--gamma requires a value".to_string());
                }
                opts.gamma = args[i]
                    .parse()
                    .map_err(|_| "Invalid gamma value")?;
            }
            "--loop" => {
                opts.loop_playback = true;
            }
            arg if arg.starts_with("--") => {
                return Err(format!("Unknown option: {}", arg));
            }
            _ => {
                if opts.video_path.is_empty() {
                    opts.video_path = args[i].clone();
                } else {
                    return Err(format!("Unexpected argument: {}", args[i]));
                }
            }
        }
        i += 1;
    }

    if opts.video_path.is_empty() {
        return Err("Missing video path".to_string());
    }

    Ok(opts)
}

fn print_usage() {
    eprintln!("Usage: video_player <video_file> [options]");
    eprintln!("\nVideo playback with configurable render settings.");
    eprintln!("\nSupported formats: MP4, MKV, AVI, WebM, MOV");
    eprintln!("\nOptions:");
    eprintln!("  --dither <method>     Dithering: none, floyd (default), bayer, atkinson");
    eprintln!("  --threshold <0-255>   Manual threshold (omit for auto Otsu)");
    eprintln!("  --brightness <float>  Brightness multiplier (default: 1.0)");
    eprintln!("  --contrast <float>    Contrast multiplier (default: 1.0)");
    eprintln!("  --gamma <float>       Gamma correction (default: 1.0)");
    eprintln!("  --loop                Loop video playback");
    eprintln!("\nTip: Use render_tuner to discover optimal settings interactively:");
    eprintln!("  cargo run --example render_tuner --features image,video -- video.mp4");
    eprintln!("\nExample:");
    eprintln!("  cargo run --example video_player --features video -- video.mp4 --dither bayer");
}

fn dithering_name(d: DitheringMethod) -> &'static str {
    match d {
        DitheringMethod::None => "None",
        DitheringMethod::FloydSteinberg => "FloydSteinberg",
        DitheringMethod::Bayer => "Bayer",
        DitheringMethod::Atkinson => "Atkinson",
    }
}

fn main() -> dotmax::Result<()> {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}\n", e);
            print_usage();
            std::process::exit(1);
        }
    };

    // Create video player with configured settings
    println!("Opening video: {}", opts.video_path);

    let mut player = VideoPlayer::new(&opts.video_path)?
        .dithering(opts.dithering)
        .brightness(opts.brightness)
        .contrast(opts.contrast)
        .gamma(opts.gamma);

    if let Some(t) = opts.threshold {
        player = player.threshold(Some(t));
    }

    // Print video info
    println!("Video info:");
    println!("  Resolution: {}x{}", player.width(), player.height());
    println!("  Frame rate: {:.2} fps", player.fps());
    if let Some(duration) = player.duration() {
        println!("  Duration: {:.2}s", duration.as_secs_f64());
    }
    if let Some(frames) = player.frame_count() {
        println!("  Estimated frames: {}", frames);
    }
    println!("\nRender settings:");
    println!("  Dithering:  {}", dithering_name(opts.dithering));
    println!(
        "  Threshold:  {}",
        opts.threshold
            .map(|t| t.to_string())
            .unwrap_or_else(|| "Auto (Otsu)".to_string())
    );
    println!("  Brightness: {:.2}", opts.brightness);
    println!("  Contrast:   {:.2}", opts.contrast);
    println!("  Gamma:      {:.2}", opts.gamma);
    println!("  Looping:    {}", if opts.loop_playback { "Yes" } else { "No" });
    println!("\nPress any key to stop playback...\n");

    // Small delay to let user read info
    std::thread::sleep(Duration::from_secs(1));

    // Enter raw mode and alternate screen
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let mut renderer = TerminalRenderer::new()?;
    let mut frame_count = 0u64;
    let mut loop_count = 0u32;
    let start_time = Instant::now();

    // Play frames
    let result = (|| -> dotmax::Result<()> {
        loop {
            match player.next_frame() {
                Some(Ok((grid, delay))) => {
                    frame_count += 1;

                    // Render frame
                    renderer.render(&grid)?;

                    // Wait for frame duration, checking for keypress
                    let deadline = Instant::now() + delay;
                    while Instant::now() < deadline {
                        if event::poll(Duration::from_millis(10))? {
                            if let Event::Key(key_event) = event::read()? {
                                if !matches!(key_event.code, KeyCode::Modifier(_)) {
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => return Err(e),
                None => {
                    // Video ended
                    if opts.loop_playback {
                        loop_count += 1;
                        player.reset();
                    } else {
                        return Ok(());
                    }
                }
            }
        }
    })();

    // Cleanup - always restore terminal state
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    // Print playback stats
    let elapsed = start_time.elapsed();
    println!("\nPlayback complete!");
    println!("  Frames rendered: {}", frame_count);
    println!("  Loops completed: {}", loop_count);
    println!("  Time elapsed: {:.2}s", elapsed.as_secs_f64());
    if elapsed.as_secs_f64() > 0.0 {
        println!(
            "  Average FPS: {:.2}",
            frame_count as f64 / elapsed.as_secs_f64()
        );
    }

    result
}
