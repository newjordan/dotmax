//! Interactive render settings tuner for discovering optimal parameters.
//!
//! **Requires the `image` feature. For video tuning, also requires `video` feature.**
//!
//! This tool helps you find the best render settings for your media by providing
//! real-time visual feedback as you adjust parameters. Once you've found settings
//! you like, it outputs the exact API code to reproduce them.
//!
//! # Usage
//!
//! ```bash
//! # For images
//! cargo run --example render_tuner --features image -- image.png
//!
//! # For video (plays in loop while tuning)
//! cargo run --example render_tuner --features image,video -- video.mp4
//! ```
//!
//! # Controls
//!
//! | Key       | Action                                      |
//! |-----------|---------------------------------------------|
//! | `D`       | Cycle dithering: None → Floyd → Bayer → Atkinson |
//! | `T`       | Toggle threshold mode: Auto ↔ Manual        |
//! | `↑`/`↓`   | Adjust threshold (±5, when manual)          |
//! | `B`/`b`   | Increase/decrease brightness (±0.1)         |
//! | `C`/`c`   | Increase/decrease contrast (±0.1)           |
//! | `G`/`g`   | Increase/decrease gamma (±0.1)              |
//! | `M`       | Cycle color mode: Mono → Gray → TrueColor   |
//! | `Space`   | Pause/Resume video playback                 |
//! | `R`       | Reset all settings to defaults              |
//! | `S`       | Show API snippet inline                     |
//! | `Q`/`Esc` | Quit and print API snippet                  |
//!
//! # Performance
//!
//! The tuner uses optimized rendering with:
//! - Pre-built line buffers (single write per line)
//! - Differential rendering (skip unchanged lines)
//! - Efficient color code batching

use dotmax::image::{ColorMode, DitheringMethod, ImageRenderer};
use dotmax::BrailleGrid;
use std::env;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::io::{stdout, Write};

/// Current render settings being tuned.
#[derive(Debug, Clone)]
struct TunerSettings {
    dithering: DitheringMethod,
    threshold: Option<u8>,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    color_mode: ColorMode,
    paused: bool,
    show_snippet: bool,
}

impl Default for TunerSettings {
    fn default() -> Self {
        Self {
            dithering: DitheringMethod::FloydSteinberg,
            threshold: None,
            brightness: 1.0,
            contrast: 1.0,
            gamma: 1.0,
            color_mode: ColorMode::Monochrome,
            paused: false,
            show_snippet: false,
        }
    }
}

/// Frame buffer for differential rendering.
/// Stores pre-rendered lines to enable skip-unchanged optimization.
struct FrameBuffer {
    /// Pre-rendered lines with ANSI codes baked in
    lines: Vec<String>,
    /// Terminal width used for this buffer
    width: usize,
    /// Terminal height used for this buffer
    height: usize,
}

impl FrameBuffer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// Render a grid to the frame buffer, returning which lines changed.
    /// Returns a Vec of (line_index, line_content) for lines that need redraw.
    fn update_from_grid(
        &mut self,
        grid: &BrailleGrid,
        term_width: usize,
        term_height: usize,
        hud_height: usize,
    ) -> Vec<(usize, String)> {
        let max_grid_lines = term_height.saturating_sub(hud_height);
        let grid_lines = grid.height().min(max_grid_lines);

        // Check if dimensions changed - full redraw needed
        let dimensions_changed = self.width != term_width || self.height != term_height;
        if dimensions_changed {
            self.width = term_width;
            self.height = term_height;
            self.lines.clear();
        }

        // Ensure we have enough line slots
        if self.lines.len() < grid_lines {
            self.lines.resize(grid_lines, String::new());
        }

        let mut changed_lines = Vec::new();

        for y in 0..grid_lines {
            let new_line = render_grid_line(grid, y, term_width);

            // Check if line changed (or first render)
            let line_changed = if y < self.lines.len() {
                self.lines[y] != new_line
            } else {
                true
            };

            if line_changed || dimensions_changed {
                if y < self.lines.len() {
                    self.lines[y].clone_from(&new_line);
                } else {
                    self.lines.push(new_line.clone());
                }
                changed_lines.push((y, new_line));
            }
        }

        changed_lines
    }
}

/// Render a single grid line to a string with ANSI color codes.
/// Batches consecutive same-color characters for efficiency.
fn render_grid_line(grid: &BrailleGrid, y: usize, max_width: usize) -> String {
    let width = grid.width().min(max_width);
    // Pre-allocate: worst case is color change every char (~20 bytes per color code)
    let mut output = String::with_capacity(width * 4);
    let mut last_color: Option<dotmax::Color> = None;
    let mut batch_chars: Vec<char> = Vec::with_capacity(width);

    for x in 0..width {
        let ch = grid.get_char(x, y);
        let color = grid.get_color(x, y);

        if color != last_color && !batch_chars.is_empty() {
            // Flush the batch
            flush_color_batch(&mut output, &batch_chars, last_color);
            batch_chars.clear();
        }

        batch_chars.push(ch);
        last_color = color;
    }

    // Flush remaining batch
    if !batch_chars.is_empty() {
        flush_color_batch(&mut output, &batch_chars, last_color);
    }

    // Reset color at end of line
    output.push_str("\x1b[0m");

    // Pad to full width to overwrite previous content
    let visible_chars = width;
    if visible_chars < max_width {
        output.push_str(&" ".repeat(max_width - visible_chars));
    }

    output
}

/// Flush a batch of same-colored characters to the output string.
#[inline]
fn flush_color_batch(output: &mut String, chars: &[char], color: Option<dotmax::Color>) {
    if let Some(c) = color {
        output.push_str(&format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b));
    }
    output.extend(chars.iter());
}

impl TunerSettings {
    fn dithering_name(&self) -> &'static str {
        match self.dithering {
            DitheringMethod::None => "None",
            DitheringMethod::FloydSteinberg => "FloydSteinberg",
            DitheringMethod::Bayer => "Bayer",
            DitheringMethod::Atkinson => "Atkinson",
        }
    }

    fn cycle_dithering(&mut self) {
        self.dithering = match self.dithering {
            DitheringMethod::None => DitheringMethod::FloydSteinberg,
            DitheringMethod::FloydSteinberg => DitheringMethod::Bayer,
            DitheringMethod::Bayer => DitheringMethod::Atkinson,
            DitheringMethod::Atkinson => DitheringMethod::None,
        };
    }

    fn toggle_threshold_mode(&mut self) {
        self.threshold = match self.threshold {
            None => Some(128),
            Some(_) => None,
        };
    }

    fn adjust_threshold(&mut self, delta: i16) {
        if let Some(ref mut t) = self.threshold {
            *t = (*t as i16 + delta).clamp(0, 255) as u8;
        }
    }

    fn adjust_brightness(&mut self, delta: f32) {
        // ImageRenderer accepts 0.0-2.0
        self.brightness = (self.brightness + delta).clamp(0.1, 2.0);
    }

    fn adjust_contrast(&mut self, delta: f32) {
        // ImageRenderer accepts 0.0-2.0
        self.contrast = (self.contrast + delta).clamp(0.1, 2.0);
    }

    fn adjust_gamma(&mut self, delta: f32) {
        // ImageRenderer accepts 0.1-3.0
        self.gamma = (self.gamma + delta).clamp(0.1, 3.0);
    }

    fn cycle_color_mode(&mut self) {
        self.color_mode = match self.color_mode {
            ColorMode::Monochrome => ColorMode::Grayscale,
            ColorMode::Grayscale => ColorMode::TrueColor,
            ColorMode::TrueColor => ColorMode::Monochrome,
        };
    }

    fn color_mode_name(&self) -> &'static str {
        match self.color_mode {
            ColorMode::Monochrome => "Mono",
            ColorMode::Grayscale => "Gray",
            ColorMode::TrueColor => "True",
        }
    }

    /// Generate the API snippet for ImageRenderer
    fn to_image_renderer_snippet(&self) -> String {
        let mut lines = vec!["ImageRenderer::new()".to_string()];

        // Only include non-default settings
        if self.dithering != DitheringMethod::FloydSteinberg {
            lines.push(format!(
                "    .dithering(DitheringMethod::{})",
                self.dithering_name()
            ));
        }

        if let Some(t) = self.threshold {
            lines.push(format!("    .threshold({})", t));
        }

        if (self.brightness - 1.0).abs() > 0.01 {
            lines.push(format!("    .brightness({:.1})?", self.brightness));
        }

        if (self.contrast - 1.0).abs() > 0.01 {
            lines.push(format!("    .contrast({:.1})?", self.contrast));
        }

        if (self.gamma - 1.0).abs() > 0.01 {
            lines.push(format!("    .gamma({:.1})?", self.gamma));
        }

        if self.color_mode != ColorMode::Monochrome {
            let mode_str = match self.color_mode {
                ColorMode::Monochrome => "Monochrome",
                ColorMode::Grayscale => "Grayscale",
                ColorMode::TrueColor => "TrueColor",
            };
            lines.push(format!("    .color_mode(ColorMode::{})", mode_str));
        }

        if lines.len() == 1 {
            "ImageRenderer::new()  // Using all defaults".to_string()
        } else {
            lines.join("\n")
        }
    }

    /// Generate the API snippet for VideoPlayer
    fn to_video_player_snippet(&self, path: &str) -> String {
        let mut lines = vec![format!("VideoPlayer::new(\"{}\")?", path)];

        if self.dithering != DitheringMethod::FloydSteinberg {
            lines.push(format!(
                "    .dithering(DitheringMethod::{})",
                self.dithering_name()
            ));
        }

        if let Some(t) = self.threshold {
            lines.push(format!("    .threshold(Some({}))", t));
        }

        if (self.brightness - 1.0).abs() > 0.01 {
            lines.push(format!("    .brightness({:.1})", self.brightness));
        }

        if (self.contrast - 1.0).abs() > 0.01 {
            lines.push(format!("    .contrast({:.1})", self.contrast));
        }

        if (self.gamma - 1.0).abs() > 0.01 {
            lines.push(format!("    .gamma({:.1})", self.gamma));
        }

        if self.color_mode != ColorMode::Monochrome {
            let mode_str = match self.color_mode {
                ColorMode::Monochrome => "Monochrome",
                ColorMode::Grayscale => "Grayscale",
                ColorMode::TrueColor => "TrueColor",
            };
            lines.push(format!("    .color_mode(ColorMode::{})", mode_str));
        }

        if lines.len() == 1 {
            format!("VideoPlayer::new(\"{}\")?  // Using all defaults", path)
        } else {
            lines.join("\n")
        }
    }
}

/// Media source for tuning
enum MediaSource {
    Image(image::DynamicImage),
    #[cfg(feature = "video")]
    Video(String), // Path to video file
}

fn main() -> dotmax::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <media_file>", args[0]);
        eprintln!("\nInteractive render settings tuner.");
        eprintln!("\nSupported formats:");
        eprintln!("  Images: PNG, JPEG, GIF, BMP, WebP");
        #[cfg(feature = "video")]
        eprintln!("  Video:  MP4, MKV, AVI, WebM (plays in loop)");
        eprintln!("\nControls:");
        eprintln!("  D       - Cycle dithering algorithm");
        eprintln!("  T       - Toggle threshold mode (auto/manual)");
        eprintln!("  Up/Down - Adjust manual threshold (±5)");
        eprintln!("  B/b     - Increase/decrease brightness");
        eprintln!("  C/c     - Increase/decrease contrast");
        eprintln!("  G/g     - Increase/decrease gamma");
        eprintln!("  Space   - Pause/Resume video");
        eprintln!("  R       - Reset all settings to defaults");
        eprintln!("  S       - Show API snippet");
        eprintln!("  Q/Esc   - Quit and print snippet");
        eprintln!("\nExample:");
        eprintln!("  cargo run --example render_tuner --features image -- photo.jpg");
        eprintln!("  cargo run --example render_tuner --features image,video -- video.mp4");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let path = Path::new(file_path);

    // Load media
    println!("Loading: {}", file_path);
    let (source, _is_video) = load_media(path)?;

    match source {
        MediaSource::Image(img) => {
            println!("Loaded image. Starting tuner...");
            std::thread::sleep(Duration::from_millis(300));
            run_image_tuner(img, file_path)
        }
        #[cfg(feature = "video")]
        MediaSource::Video(video_path) => {
            println!("Loaded video. Starting tuner with live playback...");
            std::thread::sleep(Duration::from_millis(300));
            run_video_tuner(&video_path)
        }
    }
}

fn load_media(path: &Path) -> dotmax::Result<(MediaSource, bool)> {
    #[allow(unused_variables)]
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Check if it's a video format
    #[cfg(feature = "video")]
    {
        if matches!(ext.as_str(), "mp4" | "mkv" | "avi" | "webm" | "mov") {
            return Ok((MediaSource::Video(path.to_string_lossy().to_string()), true));
        }
    }

    // Load as image
    let img = image::open(path).map_err(|e| dotmax::DotmaxError::ImageLoad {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok((MediaSource::Image(img), false))
}

// ============================================================================
// Image Tuner (static frame)
// ============================================================================

fn run_image_tuner(img: image::DynamicImage, file_path: &str) -> dotmax::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let mut settings = TunerSettings::default();
    let mut needs_redraw = true;

    let result = (|| -> dotmax::Result<()> {
        loop {
            if needs_redraw {
                let start = Instant::now();
                let grid = render_image_with_settings(&img, &settings)?;
                let render_time = start.elapsed();
                draw_image_frame(&mut stdout, &grid, &settings, render_time)?;
                needs_redraw = false;
            }

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            settings.cycle_dithering();
                            needs_redraw = true;
                        }
                        KeyCode::Char('t') | KeyCode::Char('T') => {
                            settings.toggle_threshold_mode();
                            needs_redraw = true;
                        }
                        KeyCode::Up => {
                            settings.adjust_threshold(5);
                            needs_redraw = true;
                        }
                        KeyCode::Down => {
                            settings.adjust_threshold(-5);
                            needs_redraw = true;
                        }
                        KeyCode::Char('B') => {
                            settings.adjust_brightness(0.1);
                            needs_redraw = true;
                        }
                        KeyCode::Char('b') => {
                            settings.adjust_brightness(-0.1);
                            needs_redraw = true;
                        }
                        KeyCode::Char('C') => {
                            settings.adjust_contrast(0.1);
                            needs_redraw = true;
                        }
                        KeyCode::Char('c') => {
                            settings.adjust_contrast(-0.1);
                            needs_redraw = true;
                        }
                        KeyCode::Char('G') => {
                            settings.adjust_gamma(0.1);
                            needs_redraw = true;
                        }
                        KeyCode::Char('g') => {
                            settings.adjust_gamma(-0.1);
                            needs_redraw = true;
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            settings.cycle_color_mode();
                            needs_redraw = true;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            settings = TunerSettings::default();
                            needs_redraw = true;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            settings.show_snippet = !settings.show_snippet;
                            needs_redraw = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    })();

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    print_final_snippet(&settings, file_path, false);
    result
}

fn render_image_with_settings(
    img: &image::DynamicImage,
    settings: &TunerSettings,
) -> dotmax::Result<BrailleGrid> {
    let (term_width, term_height) = terminal::size().unwrap_or((80, 24));
    let render_height = (term_height as usize).saturating_sub(5);

    let mut renderer = ImageRenderer::new()
        .load_from_rgba(img.to_rgba8())
        .resize(term_width as usize, render_height, true)?
        .dithering(settings.dithering)
        .color_mode(settings.color_mode);

    if let Some(t) = settings.threshold {
        renderer = renderer.threshold(t);
    }

    if (settings.brightness - 1.0).abs() > f32::EPSILON {
        renderer = renderer.brightness(settings.brightness)?;
    }
    if (settings.contrast - 1.0).abs() > f32::EPSILON {
        renderer = renderer.contrast(settings.contrast)?;
    }
    if (settings.gamma - 1.0).abs() > f32::EPSILON {
        renderer = renderer.gamma(settings.gamma)?;
    }

    renderer.render()
}

fn draw_image_frame(
    stdout: &mut impl Write,
    grid: &BrailleGrid,
    settings: &TunerSettings,
    render_time: Duration,
) -> dotmax::Result<()> {
    let (term_width, term_height) = terminal::size().unwrap_or((80, 24));

    execute!(stdout, cursor::MoveTo(0, 0))?;

    for y in 0..grid.height() {
        let line: String = (0..grid.width()).map(|x| grid.get_char(x, y)).collect();
        write!(stdout, "{}\r\n", line)?;
    }

    let remaining = (term_height as usize).saturating_sub(grid.height() + 5);
    for _ in 0..remaining {
        write!(stdout, "{}\r\n", " ".repeat(term_width as usize))?;
    }

    draw_hud(stdout, settings, render_time, term_width, 0.0, false)?;
    stdout.flush()?;
    Ok(())
}

// ============================================================================
// Video Tuner (live playback with loop)
// ============================================================================

#[cfg(feature = "video")]
fn run_video_tuner(video_path: &str) -> dotmax::Result<()> {
    use dotmax::media::MediaPlayer;

    // Set up terminal manually - don't use TerminalRenderer since we need
    // to mix frame rendering with HUD text, which conflicts with Ratatui's buffer
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let mut settings = TunerSettings::default();
    let mut last_frame_time = Instant::now();
    let mut avg_fps = 0.0f64;
    let mut render_fps = 0.0f64; // Actual render performance (excludes sleep)

    // Frame buffer for differential rendering
    let mut frame_buffer = FrameBuffer::new();

    // Create initial player
    let mut player = create_video_player(video_path, &settings)?;

    // Track if HUD needs full redraw (settings changed)
    let mut hud_dirty = true;

    let result = (|| -> dotmax::Result<()> {
        loop {
            // Handle input (non-blocking) - use 10ms poll for efficiency
            while event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char(' ') => {
                            settings.paused = !settings.paused;
                            hud_dirty = true;
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            settings.cycle_dithering();
                            player.set_dithering(settings.dithering);
                            hud_dirty = true;
                        }
                        KeyCode::Char('t') | KeyCode::Char('T') => {
                            settings.toggle_threshold_mode();
                            player.set_threshold(settings.threshold);
                            hud_dirty = true;
                        }
                        KeyCode::Up => {
                            settings.adjust_threshold(5);
                            player.set_threshold(settings.threshold);
                            hud_dirty = true;
                        }
                        KeyCode::Down => {
                            settings.adjust_threshold(-5);
                            player.set_threshold(settings.threshold);
                            hud_dirty = true;
                        }
                        KeyCode::Char('B') => {
                            settings.adjust_brightness(0.1);
                            player.set_brightness(settings.brightness);
                            hud_dirty = true;
                        }
                        KeyCode::Char('b') => {
                            settings.adjust_brightness(-0.1);
                            player.set_brightness(settings.brightness);
                            hud_dirty = true;
                        }
                        KeyCode::Char('C') => {
                            settings.adjust_contrast(0.1);
                            player.set_contrast(settings.contrast);
                            hud_dirty = true;
                        }
                        KeyCode::Char('c') => {
                            settings.adjust_contrast(-0.1);
                            player.set_contrast(settings.contrast);
                            hud_dirty = true;
                        }
                        KeyCode::Char('G') => {
                            settings.adjust_gamma(0.1);
                            player.set_gamma(settings.gamma);
                            hud_dirty = true;
                        }
                        KeyCode::Char('g') => {
                            settings.adjust_gamma(-0.1);
                            player.set_gamma(settings.gamma);
                            hud_dirty = true;
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            settings.cycle_color_mode();
                            player.set_color_mode(settings.color_mode);
                            hud_dirty = true;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            settings = TunerSettings::default();
                            player = create_video_player(video_path, &settings)?;
                            frame_buffer = FrameBuffer::new(); // Reset buffer on player reset
                            hud_dirty = true;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            settings.show_snippet = !settings.show_snippet;
                            hud_dirty = true;
                        }
                        _ => {}
                    }
                }
            }

            // Skip frame if paused
            if settings.paused {
                if hud_dirty {
                    draw_paused_hud(&mut stdout, &settings, avg_fps, render_fps)?;
                    hud_dirty = false;
                }
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            // Get next frame
            match player.next_frame() {
                Some(Ok((grid, delay))) => {
                    let render_start = Instant::now();

                    // Calculate total FPS (includes sleep from last frame)
                    let frame_elapsed = last_frame_time.elapsed();
                    last_frame_time = Instant::now();
                    let instant_fps = 1.0 / frame_elapsed.as_secs_f64();
                    avg_fps = avg_fps * 0.9 + instant_fps * 0.1;

                    // Render frame with differential updates
                    let (term_width, term_height) = terminal::size().unwrap_or((80, 24));
                    let hud_height = if settings.show_snippet { 5 } else { 4 };

                    let render_params = FrameRenderParams {
                        fps: avg_fps,
                        render_fps,
                        term_width: term_width as usize,
                        term_height: term_height as usize,
                        hud_height,
                        force_hud_redraw: hud_dirty,
                    };

                    draw_video_frame_optimized(
                        &mut stdout,
                        &grid,
                        &mut frame_buffer,
                        &settings,
                        &render_params,
                    )?;
                    hud_dirty = false;

                    // Calculate render FPS (excludes sleep)
                    let render_time = render_start.elapsed();
                    let instant_render_fps = 1.0 / render_time.as_secs_f64();
                    render_fps = render_fps * 0.9 + instant_render_fps * 0.1;

                    // Wait for frame timing
                    if render_time < delay {
                        std::thread::sleep(delay - render_time);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => {
                    // Loop video
                    player.reset();
                }
            }
        }
    })();

    // Clean up terminal state
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    print_final_snippet(&settings, video_path, true);
    result
}

#[cfg(feature = "video")]
fn create_video_player(
    path: &str,
    settings: &TunerSettings,
) -> dotmax::Result<dotmax::media::VideoPlayer> {
    use dotmax::media::VideoPlayer;

    let mut player = VideoPlayer::new(path)?
        .dithering(settings.dithering)
        .brightness(settings.brightness)
        .contrast(settings.contrast)
        .gamma(settings.gamma)
        .color_mode(settings.color_mode);

    if let Some(t) = settings.threshold {
        player = player.threshold(Some(t));
    }

    Ok(player)
}

/// Parameters for frame rendering to avoid clippy's too-many-arguments lint.
#[cfg(feature = "video")]
struct FrameRenderParams {
    fps: f64,
    render_fps: f64,
    term_width: usize,
    term_height: usize,
    hud_height: usize,
    force_hud_redraw: bool,
}

/// Optimized video frame rendering with differential updates.
/// Only redraws lines that have changed since last frame.
#[cfg(feature = "video")]
fn draw_video_frame_optimized(
    stdout: &mut impl Write,
    grid: &BrailleGrid,
    frame_buffer: &mut FrameBuffer,
    settings: &TunerSettings,
    params: &FrameRenderParams,
) -> dotmax::Result<()> {
    // Get changed lines from differential buffer
    let changed_lines = frame_buffer.update_from_grid(
        grid,
        params.term_width,
        params.term_height,
        params.hud_height,
    );

    // Build a single output buffer for all changed lines
    let mut output = String::with_capacity(changed_lines.len() * (params.term_width + 20));

    for (y, line_content) in changed_lines {
        // Move cursor to line position and write the pre-rendered line
        output.push_str(&format!("\x1b[{};1H{}", y + 1, line_content));
    }

    // Position cursor for HUD (always at bottom)
    let hud_start_row = params.term_height.saturating_sub(params.hud_height);
    output.push_str(&format!("\x1b[{};1H", hud_start_row + 1));

    // Write frame content in one syscall
    write!(stdout, "{}", output)?;

    // Draw HUD (always update FPS display)
    draw_hud_optimized(
        stdout,
        settings,
        params.fps,
        params.render_fps,
        params.term_width as u16,
        params.hud_height,
        params.force_hud_redraw,
    )?;

    stdout.flush()?;
    Ok(())
}

#[cfg(feature = "video")]
fn draw_paused_hud(
    stdout: &mut impl Write,
    settings: &TunerSettings,
    fps: f64,
    render_fps: f64,
) -> dotmax::Result<()> {
    let (term_width, term_height) = terminal::size().unwrap_or((80, 24));
    let hud_height = if settings.show_snippet { 5 } else { 4 };
    let hud_start_row = (term_height as usize).saturating_sub(hud_height);

    // Position cursor at HUD start
    execute!(stdout, cursor::MoveTo(0, hud_start_row as u16))?;

    draw_hud_optimized(stdout, settings, fps, render_fps, term_width, hud_height, true)?;
    stdout.flush()?;
    Ok(())
}

/// Optimized HUD drawing for video mode.
#[cfg(feature = "video")]
fn draw_hud_optimized(
    stdout: &mut impl Write,
    settings: &TunerSettings,
    fps: f64,
    render_fps: f64,
    term_width: u16,
    hud_height: usize,
    _force_redraw: bool, // Reserved for future selective HUD updates
) -> dotmax::Result<()> {
    let inv_on = "\x1b[7m";
    let inv_off = "\x1b[0m";

    let status = if settings.paused { "PAUSED " } else { "Playing" };

    // Line 1: Status and main settings
    let line1 = format!(
        " {} | Dither: {:12} | Thresh: {:4} | FPS: {:5.1} ({:5.1}) ",
        status,
        settings.dithering_name(),
        settings
            .threshold
            .map(|t| format!("{}", t))
            .unwrap_or_else(|| "Auto".to_string()),
        fps,
        render_fps, // Show render FPS in parentheses
    );

    // Line 2: Adjustments
    let line2 = format!(
        " Bright: {:4.2} | Contrast: {:4.2} | Gamma: {:4.2} | Color: {:5} ",
        settings.brightness, settings.contrast, settings.gamma, settings.color_mode_name()
    );

    // Line 3: Controls
    let line3 = " [D]ither [T]hresh [B]right [C]ontrast [G]amma [M]ode [Space]Pause [R]eset [S]nippet [Q]uit ";

    write!(stdout, "{}{}{}\r\n", inv_on, pad(&line1, term_width as usize), inv_off)?;
    write!(stdout, "{}{}{}\r\n", inv_on, pad(&line2, term_width as usize), inv_off)?;
    write!(stdout, "{}{}{}\r\n", inv_on, pad(line3, term_width as usize), inv_off)?;

    // Line 4: Snippet (optional)
    if settings.show_snippet && hud_height >= 4 {
        let snippet_line = format!(
            " {} ",
            settings
                .to_image_renderer_snippet()
                .replace('\n', " | ")
        );
        write!(stdout, "{}{}{}\r\n", inv_on, pad(&snippet_line, term_width as usize), inv_off)?;
    }

    Ok(())
}

// ============================================================================
// Shared UI
// ============================================================================

fn draw_hud(
    stdout: &mut impl Write,
    settings: &TunerSettings,
    render_time: Duration,
    term_width: u16,
    fps: f64,
    is_video: bool,
) -> dotmax::Result<()> {
    let inv_on = "\x1b[7m";
    let inv_off = "\x1b[0m";

    let status = if settings.paused { "PAUSED" } else { "Playing" };

    let line1 = if is_video {
        format!(
            " {} | Dither: {:15} | Thresh: {:8} | FPS: {:5.1} ",
            status,
            settings.dithering_name(),
            settings
                .threshold
                .map(|t| format!("{}", t))
                .unwrap_or_else(|| "Auto".to_string()),
            fps
        )
    } else {
        format!(
            " Dither: {:15} | Thresh: {:8} | Render: {:6.2}ms ",
            settings.dithering_name(),
            settings
                .threshold
                .map(|t| format!("{}", t))
                .unwrap_or_else(|| "Auto".to_string()),
            render_time.as_secs_f64() * 1000.0
        )
    };

    let line2 = format!(
        " Bright: {:4.2} | Contrast: {:4.2} | Gamma: {:4.2} | Color: {:5} ",
        settings.brightness, settings.contrast, settings.gamma, settings.color_mode_name()
    );

    let line3 = if is_video {
        " [D]ither [T]hresh [B]right [C]ontrast [G]amma [M]ode [Space]Pause [R]eset [S]nippet [Q]uit "
    } else {
        " [D]ither [T]hresh [B]right [C]ontrast [G]amma [M]ode [R]eset [S]nippet [Q]uit "
    };

    write!(
        stdout,
        "{}{}{}\r\n",
        inv_on,
        pad(&line1, term_width as usize),
        inv_off
    )?;
    write!(
        stdout,
        "{}{}{}\r\n",
        inv_on,
        pad(&line2, term_width as usize),
        inv_off
    )?;
    write!(
        stdout,
        "{}{}{}\r\n",
        inv_on,
        pad(line3, term_width as usize),
        inv_off
    )?;

    if settings.show_snippet {
        let snippet_line = format!(
            " {} ",
            settings
                .to_image_renderer_snippet()
                .replace('\n', " | ")
        );
        write!(
            stdout,
            "{}{}{}\r\n",
            inv_on,
            pad(&snippet_line, term_width as usize),
            inv_off
        )?;
    }

    Ok(())
}

fn pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

fn print_final_snippet(settings: &TunerSettings, file_path: &str, is_video: bool) {
    println!("\n{}", "=".repeat(60));
    println!("Your optimized render settings:");
    println!("{}", "=".repeat(60));
    println!();

    if is_video {
        println!("// For VideoPlayer:");
        println!("{}", settings.to_video_player_snippet(file_path));
        println!();
    }

    println!("// For ImageRenderer:");
    println!("{}", settings.to_image_renderer_snippet());
    println!();
    println!("{}", "=".repeat(60));
}
