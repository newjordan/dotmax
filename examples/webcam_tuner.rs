//! Interactive webcam tuner for discovering optimal render settings.
//!
//! **Requires the `video` feature.**
//!
//! This tool helps you find the best render settings for your webcam feed by providing
//! real-time visual feedback as you adjust parameters. Demonstrates the WebcamPlayer
//! runtime settings API.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example webcam_tuner --features video
//! ```
//!
//! # Controls
//!
//! | Key       | Action                                      |
//! |-----------|---------------------------------------------|
//! | `D`/`d`   | Cycle dithering: Floyd → Bayer → Atkinson → None |
//! | `T`/`t`   | Toggle threshold mode: Auto (Otsu) ↔ Manual |
//! | `+`/`-`   | Adjust manual threshold (±10)               |
//! | `[`/`]`   | Fine adjust threshold (±1)                  |
//! | `B`/`b`   | Increase/decrease brightness (±0.1)         |
//! | `C`/`c`   | Increase/decrease contrast (±0.1)           |
//! | `G`/`g`   | Increase/decrease gamma (±0.1)              |
//! | `M`/`m`   | Toggle color mode: Mono ↔ TrueColor         |
//! | `R`/`r`   | Reset all settings to defaults              |
//! | `H`/`?`   | Toggle help overlay                         |
//! | `Q`/`Esc` | Quit                                        |

use dotmax::image::{ColorMode, DitheringMethod};
use dotmax::media::{list_webcams, MediaPlayer, WebcamPlayer};
use dotmax::BrailleGrid;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::fmt::Write as FmtWrite;
use std::io::{stdin, stdout, Write};
use std::time::{Duration, Instant};

// ============================================================================
// TunerState - Settings State Management (AC: #1, Task 2)
// ============================================================================

/// Current render settings being tuned.
#[derive(Debug, Clone)]
struct TunerState {
    dithering: DitheringMethod,
    use_otsu: bool,        // true = Otsu (auto), false = manual threshold
    manual_threshold: u8,  // Used when use_otsu is false
    brightness: f32,
    contrast: f32,
    gamma: f32,
    color_mode: ColorMode,
    show_help: bool,
}

impl Default for TunerState {
    fn default() -> Self {
        Self {
            dithering: DitheringMethod::FloydSteinberg,
            use_otsu: true,
            manual_threshold: 128,
            brightness: 1.0,
            contrast: 1.0,
            gamma: 1.0,
            color_mode: ColorMode::Monochrome,
            show_help: false,
        }
    }
}

impl TunerState {
    /// Returns the dithering method name for display.
    const fn dithering_name(&self) -> &'static str {
        match self.dithering {
            DitheringMethod::None => "None",
            DitheringMethod::FloydSteinberg => "FloydSteinberg",
            DitheringMethod::Bayer => "Bayer",
            DitheringMethod::Atkinson => "Atkinson",
        }
    }

    /// Cycles to the next dithering method.
    fn cycle_dithering(&mut self) {
        self.dithering = match self.dithering {
            DitheringMethod::FloydSteinberg => DitheringMethod::Bayer,
            DitheringMethod::Bayer => DitheringMethod::Atkinson,
            DitheringMethod::Atkinson => DitheringMethod::None,
            DitheringMethod::None => DitheringMethod::FloydSteinberg,
        };
    }

    /// Toggles between Otsu (auto) and manual threshold mode.
    fn toggle_threshold_mode(&mut self) {
        self.use_otsu = !self.use_otsu;
    }

    /// Adjusts the manual threshold value.
    fn adjust_threshold(&mut self, delta: i16) {
        let new_val = (self.manual_threshold as i16 + delta).clamp(0, 255);
        self.manual_threshold = new_val as u8;
    }

    /// Returns the threshold value for the player (None = Otsu).
    const fn threshold_value(&self) -> Option<u8> {
        if self.use_otsu {
            None
        } else {
            Some(self.manual_threshold)
        }
    }

    /// Returns threshold display string.
    fn threshold_display(&self) -> String {
        if self.use_otsu {
            "Auto (Otsu)".to_string()
        } else {
            format!("{}", self.manual_threshold)
        }
    }

    /// Adjusts brightness within valid range.
    fn adjust_brightness(&mut self, delta: f32) {
        self.brightness = (self.brightness + delta).clamp(0.1, 3.0);
    }

    /// Adjusts contrast within valid range.
    fn adjust_contrast(&mut self, delta: f32) {
        self.contrast = (self.contrast + delta).clamp(0.1, 3.0);
    }

    /// Adjusts gamma within valid range.
    fn adjust_gamma(&mut self, delta: f32) {
        self.gamma = (self.gamma + delta).clamp(0.1, 3.0);
    }

    /// Toggles between Monochrome and TrueColor modes.
    fn toggle_color_mode(&mut self) {
        self.color_mode = match self.color_mode {
            ColorMode::TrueColor => ColorMode::Monochrome,
            ColorMode::Monochrome | ColorMode::Grayscale => ColorMode::TrueColor,
        };
    }

    /// Returns the color mode name for display.
    const fn color_mode_name(&self) -> &'static str {
        match self.color_mode {
            ColorMode::Monochrome => "Mono",
            ColorMode::Grayscale => "Gray",
            ColorMode::TrueColor => "TrueColor",
        }
    }

    /// Applies current state to the WebcamPlayer.
    fn apply_to_player(&self, player: &mut WebcamPlayer) {
        player.set_dithering(self.dithering);
        player.set_threshold(self.threshold_value());
        player.set_brightness(self.brightness);
        player.set_contrast(self.contrast);
        player.set_gamma(self.gamma);
        player.set_color_mode(self.color_mode);
    }

    /// Prints the final settings as code that can be copy-pasted.
    fn print_final_settings(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║                    FINAL TUNER SETTINGS                      ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║                                                              ║");
        println!("║  Dithering:   {:15}                               ║", self.dithering_name());
        println!("║  Threshold:   {:15}                               ║", self.threshold_display());
        println!("║  Brightness:  {:15.2}                               ║", self.brightness);
        println!("║  Contrast:    {:15.2}                               ║", self.contrast);
        println!("║  Gamma:       {:15.2}                               ║", self.gamma);
        println!("║  Color Mode:  {:15}                               ║", self.color_mode_name());
        println!("║                                                              ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Copy-paste code:                                            ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("// WebcamPlayer settings:");
        println!("let player = WebcamPlayer::builder()");
        println!("    .dithering(DitheringMethod::{:?})", self.dithering);
        if self.use_otsu {
            println!("    .threshold(None)  // Auto (Otsu)");
        } else {
            println!("    .threshold(Some({}))", self.manual_threshold);
        }
        println!("    .brightness({:.2})", self.brightness);
        println!("    .contrast({:.2})", self.contrast);
        println!("    .gamma({:.2})", self.gamma);
        println!("    .color_mode(ColorMode::{:?})", self.color_mode);
        println!("    .build()?;");
        println!();
        println!("// Or apply to existing player:");
        println!("player.set_dithering(DitheringMethod::{:?});", self.dithering);
        if self.use_otsu {
            println!("player.set_threshold(None);");
        } else {
            println!("player.set_threshold(Some({}));", self.manual_threshold);
        }
        println!("player.set_brightness({:.2});", self.brightness);
        println!("player.set_contrast({:.2});", self.contrast);
        println!("player.set_gamma({:.2});", self.gamma);
        println!("player.set_color_mode(ColorMode::{:?});", self.color_mode);
        println!();
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> dotmax::Result<()> {
    println!("=== Webcam Tuner ===\n");


    // Select camera before starting
    let camera_index = select_camera()?;

    println!("\nStarting webcam tuner...");
    println!("Press 'h' or '?' for help, 'q' to quit.\n");

    // Brief pause for user to read
    std::thread::sleep(Duration::from_millis(500));

    run_webcam_tuner(camera_index)
}

/// Lists available cameras and prompts user to select one.
/// Returns the selected camera index.
fn select_camera() -> dotmax::Result<usize> {
    let cameras = list_webcams();

    if cameras.is_empty() {
        println!("No webcams detected on this system.");
        println!("\nTroubleshooting:");
        println!("  - Ensure a webcam is connected");
        println!("  - On Linux: check that /dev/video* devices exist");
        println!("  - On macOS: grant camera access in System Preferences");
        println!("  - On Windows: ensure camera drivers are installed");
        return Err(dotmax::DotmaxError::CameraNotFound {
            device: "any".to_string(),
            available: vec![],
        });
    }

    // If only one camera, use it automatically
    if cameras.len() == 1 {
        println!("Found camera: {}", cameras[0].name);
        return Ok(0);
    }

    // Display available cameras
    println!("Available webcams:\n");
    for (i, cam) in cameras.iter().enumerate() {
        println!("  [{i}] {}", cam.name);
        if !cam.description.is_empty() {
            println!("      {}", cam.description);
        }
    }
    println!();

    // Get user selection
    loop {
        print!("Select camera (0-{}): ", cameras.len() - 1);
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let input = input.trim();

        // Default to first camera on empty input
        if input.is_empty() {
            println!("Using camera 0: {}", cameras[0].name);
            return Ok(0);
        }

        match input.parse::<usize>() {
            Ok(idx) if idx < cameras.len() => {
                println!("Using camera {idx}: {}", cameras[idx].name);
                return Ok(idx);
            }
            Ok(idx) => {
                println!("Invalid selection: {idx}. Please enter 0-{}.", cameras.len() - 1);
            }
            Err(_) => {
                println!("Please enter a number.");
            }
        }
    }
}

/// Main tuner loop.
fn run_webcam_tuner(camera_index: usize) -> dotmax::Result<()> {
    // Open selected webcam by index (library handles platform-specific lookup)
    let mut player = WebcamPlayer::from_device(camera_index)?;

    // Print camera info before entering alternate screen
    println!(
        "Camera: {}x{} @ {:.1} fps (reported)",
        player.width(),
        player.height(),
        player.fps()
    );
    std::thread::sleep(Duration::from_millis(1000));

    // Enter raw mode and alternate screen
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    // Initialize state
    let mut state = TunerState::default();
    let mut last_frame_time = Instant::now();
    let mut avg_fps = 0.0f64;

    // Run main loop with proper cleanup
    let result = (|| -> dotmax::Result<()> {
        loop {
            // Handle input (non-blocking with short timeout)
            // Only process key Press events, not Release or Repeat to avoid multiple triggers
            while event::poll(Duration::from_millis(1))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match handle_key_event(key, &mut state) {
                            KeyAction::Continue => {
                                // Apply updated settings to player
                                state.apply_to_player(&mut player);
                            }
                            KeyAction::Quit => {
                                // Cleanup terminal before printing
                                execute!(stdout, LeaveAlternateScreen)?;
                                crossterm::terminal::disable_raw_mode()?;
                                // Print final settings
                                state.print_final_settings();
                                return Ok(());
                            }
                            KeyAction::None => {}
                        }
                    }
                    Event::Resize(w, h) => {
                        player.handle_resize(w as usize, h as usize);
                    }
                    _ => {} // Ignore key release/repeat events
                }
            }

            // Get next frame
            match player.next_frame() {
                Some(Ok((grid, _delay))) => {
                    // Calculate FPS
                    let frame_elapsed = last_frame_time.elapsed();
                    last_frame_time = Instant::now();
                    let instant_fps = 1.0 / frame_elapsed.as_secs_f64();
                    avg_fps = avg_fps.mul_add(0.9, instant_fps * 0.1);

                    // Render frame and HUD
                    render_frame(&mut stdout, &grid, &state, avg_fps)?;

                    // Poll briefly for user input
                    let _ = event::poll(Duration::from_millis(1));
                }
                Some(Err(e)) => return Err(e),
                None => {
                    // Webcam streams shouldn't end, but handle gracefully
                    break;
                }
            }
        }
        Ok(())
    })();

    // Cleanup - always restore terminal state
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;


    result
}

// ============================================================================
// Key Handling (AC: #2-9, Tasks 3-8)
// ============================================================================

/// Result of handling a key event.
enum KeyAction {
    Continue, // Settings changed, continue loop
    Quit,     // Exit the tuner
    None,     // No action needed
}

/// Handles a key event and updates state accordingly.
fn handle_key_event(key: crossterm::event::KeyEvent, state: &mut TunerState) -> KeyAction {
    // If help is showing, dismiss on any key (except modifier keys)
    if state.show_help {
        if !matches!(key.code, KeyCode::Modifier(_)) {
            state.show_help = false;
            return KeyAction::Continue;
        }
        return KeyAction::None;
    }

    match key.code {
        // Quit (AC: #8)
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => KeyAction::Quit,

        // Dithering cycle (AC: #2)
        KeyCode::Char('d' | 'D') => {
            state.cycle_dithering();
            KeyAction::Continue
        }

        // Threshold toggle (AC: #3)
        KeyCode::Char('t' | 'T') => {
            state.toggle_threshold_mode();
            KeyAction::Continue
        }

        // Threshold adjustment - coarse (AC: #3)
        KeyCode::Char('+' | '=') => {
            state.adjust_threshold(10);
            KeyAction::Continue
        }
        KeyCode::Char('-') => {
            state.adjust_threshold(-10);
            KeyAction::Continue
        }

        // Threshold adjustment - fine (AC: #3)
        KeyCode::Char('[') => {
            state.adjust_threshold(-1);
            KeyAction::Continue
        }
        KeyCode::Char(']') => {
            state.adjust_threshold(1);
            KeyAction::Continue
        }

        // Brightness (AC: #4)
        KeyCode::Char('B') => {
            state.adjust_brightness(0.1);
            KeyAction::Continue
        }
        KeyCode::Char('b') => {
            state.adjust_brightness(-0.1);
            KeyAction::Continue
        }

        // Contrast (AC: #5)
        KeyCode::Char('C') => {
            state.adjust_contrast(0.1);
            KeyAction::Continue
        }
        KeyCode::Char('c') => {
            state.adjust_contrast(-0.1);
            KeyAction::Continue
        }

        // Gamma (AC: #6)
        KeyCode::Char('G') => {
            state.adjust_gamma(0.1);
            KeyAction::Continue
        }
        KeyCode::Char('g') => {
            state.adjust_gamma(-0.1);
            KeyAction::Continue
        }

        // Color mode toggle (AC: #7)
        KeyCode::Char('m' | 'M') => {
            state.toggle_color_mode();
            KeyAction::Continue
        }

        // Reset (AC: #8)
        KeyCode::Char('r' | 'R') => {
            *state = TunerState::default();
            KeyAction::Continue
        }

        // Help toggle (AC: #9)
        KeyCode::Char('h' | 'H' | '?') => {
            state.show_help = !state.show_help;
            KeyAction::Continue
        }

        _ => KeyAction::None,
    }
}

// ============================================================================
// Rendering (AC: #1, Task 9)
// ============================================================================

/// Renders a frame with the status line HUD.
fn render_frame(
    stdout: &mut impl Write,
    grid: &BrailleGrid,
    state: &TunerState,
    fps: f64,
) -> dotmax::Result<()> {
    let (term_width, term_height) = terminal::size().unwrap_or((80, 24));
    let hud_height = if state.show_help { 8 } else { 3 };
    let max_grid_lines = (term_height as usize).saturating_sub(hud_height);

    // Move cursor to top-left and hide it during render
    execute!(stdout, cursor::MoveTo(0, 0))?;

    // Render grid lines - use exact positioning to avoid jitter
    let grid_lines = grid.height().min(max_grid_lines);
    for y in 0..grid_lines {
        // Move to exact line position to prevent drift
        execute!(stdout, cursor::MoveTo(0, y as u16))?;
        render_grid_line(stdout, grid, y, term_width as usize)?;
    }

    // Clear any remaining lines between grid and HUD
    for y in grid_lines..max_grid_lines {
        execute!(stdout, cursor::MoveTo(0, y as u16))?;
        write!(stdout, "{}", " ".repeat(term_width as usize))?;
    }

    // Render HUD at fixed position from bottom
    let hud_start = max_grid_lines as u16;
    render_hud(stdout, state, fps, term_width, hud_start)?;

    stdout.flush()?;
    Ok(())
}

/// Renders a single line of the grid with color support.
fn render_grid_line(
    stdout: &mut impl Write,
    grid: &BrailleGrid,
    y: usize,
    max_width: usize,
) -> dotmax::Result<()> {
    let width = grid.width().min(max_width);
    let mut output = String::with_capacity(width * 10);
    let mut last_color: Option<dotmax::Color> = None;

    for x in 0..width {
        let ch = grid.get_char(x, y);
        let color = grid.get_color(x, y);

        // Change color if different
        if color != last_color {
            if let Some(c) = color {
                let _ = write!(output, "\x1b[38;2;{};{};{}m", c.r, c.g, c.b);
            } else {
                output.push_str("\x1b[0m");
            }
            last_color = color;
        }
        output.push(ch);
    }

    // Reset color and pad line (no newline - cursor positioning handles that)
    output.push_str("\x1b[0m");
    if width < max_width {
        output.push_str(&" ".repeat(max_width - width));
    }

    write!(stdout, "{}", output)?;
    Ok(())
}

/// Renders the status line HUD at the bottom of the screen.
fn render_hud(
    stdout: &mut impl Write,
    state: &TunerState,
    fps: f64,
    term_width: u16,
    start_row: u16,
) -> dotmax::Result<()> {
    let inv_on = "\x1b[7m"; // Inverse video
    let inv_off = "\x1b[0m";
    let width = term_width as usize;

    execute!(stdout, cursor::MoveTo(0, start_row))?;

    if state.show_help {
        // Help overlay (AC: #9)
        let help_lines = [
            " Webcam Tuner Controls ",
            " [D] Cycle dithering    [T] Toggle threshold (Otsu/Manual) ",
            " [B/b] Brightness +/-   [C/c] Contrast +/-   [G/g] Gamma +/- ",
            " [+/-] Threshold +/-10  []/[] Threshold +/-1 (fine tune) ",
            " [M] Toggle color mode  [R] Reset all settings ",
            " [H/?] Toggle help      [Q/Esc] Quit ",
            "",
            " Press any key to dismiss this help ",
        ];

        for (i, line) in help_lines.iter().enumerate() {
            execute!(stdout, cursor::MoveToNextLine(1))?;
            if i == 0 {
                // First line, cursor already positioned
            }
            write!(stdout, "{}{}{}", inv_on, pad(line, width), inv_off)?;
        }
    } else {
        // Compact status line (AC: #1, #2-7)
        let line1 = format!(
            " [D] {}  [T] {}  [M] {}  FPS: {:.1} ",
            state.dithering_name(),
            state.threshold_display(),
            state.color_mode_name(),
            fps
        );

        let line2 = format!(
            " [B] {:.1}  [C] {:.1}  [G] {:.1}  [H]elp [R]eset [Q]uit ",
            state.brightness, state.contrast, state.gamma
        );

        // Settings changed indicator line
        let defaults = TunerState::default();
        let changed = state.dithering != defaults.dithering
            || state.use_otsu != defaults.use_otsu
            || (state.brightness - defaults.brightness).abs() > 0.01
            || (state.contrast - defaults.contrast).abs() > 0.01
            || (state.gamma - defaults.gamma).abs() > 0.01
            || state.color_mode != defaults.color_mode;

        let line3 = if changed {
            " * Settings modified - press [R] to reset "
        } else {
            " Using default settings "
        };

        write!(stdout, "{}{}{}", inv_on, pad(&line1, width), inv_off)?;
        execute!(stdout, cursor::MoveToNextLine(1))?;
        write!(stdout, "{}{}{}", inv_on, pad(&line2, width), inv_off)?;
        execute!(stdout, cursor::MoveToNextLine(1))?;
        write!(stdout, "{}{}{}", inv_on, pad(line3, width), inv_off)?;
    }

    Ok(())
}

/// Pads or truncates a string to exactly the given width.
fn pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}
