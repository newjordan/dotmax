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
use dotmax::media::{MediaPlayer, WebcamPlayer};
use dotmax::BrailleGrid;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
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
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> dotmax::Result<()> {
    println!("Starting webcam tuner...");
    println!("Press 'h' or '?' for help, 'q' to quit.");
    println!();

    // Brief pause for user to read
    std::thread::sleep(Duration::from_millis(500));

    run_webcam_tuner()
}

/// Main tuner loop.
fn run_webcam_tuner() -> dotmax::Result<()> {
    // Open webcam with default settings
    let mut player = WebcamPlayer::new()?;

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
            while event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    match handle_key_event(key, &mut state) {
                        KeyAction::Continue => {
                            // Apply updated settings to player
                            state.apply_to_player(&mut player);
                        }
                        KeyAction::Quit => return Ok(()),
                        KeyAction::None => {}
                    }
                } else if let Event::Resize(w, h) = event::read()? {
                    player.handle_resize(w as usize, h as usize);
                }
            }

            // Get next frame
            match player.next_frame() {
                Some(Ok((grid, delay))) => {
                    // Calculate FPS
                    let frame_elapsed = last_frame_time.elapsed();
                    last_frame_time = Instant::now();
                    let instant_fps = 1.0 / frame_elapsed.as_secs_f64();
                    avg_fps = avg_fps.mul_add(0.9, instant_fps * 0.1);

                    // Render frame and HUD
                    render_frame(&mut stdout, &grid, &state, avg_fps)?;

                    // Wait for frame timing (but respect user input)
                    let render_time = last_frame_time.elapsed();
                    if render_time < delay {
                        // Sleep for remaining time, checking for events periodically
                        let sleep_time = delay - render_time;
                        let sleep_deadline = Instant::now() + sleep_time;
                        while Instant::now() < sleep_deadline {
                            if event::poll(Duration::from_millis(5))? {
                                break; // Process event on next iteration
                            }
                        }
                    }
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

    // Move cursor to top
    execute!(stdout, cursor::MoveTo(0, 0))?;

    // Render grid lines
    let grid_lines = grid.height().min(max_grid_lines);
    for y in 0..grid_lines {
        render_grid_line(stdout, grid, y, term_width as usize)?;
    }

    // Clear any remaining lines between grid and HUD
    let clear_lines = max_grid_lines.saturating_sub(grid_lines);
    for _ in 0..clear_lines {
        write!(stdout, "{}\r\n", " ".repeat(term_width as usize))?;
    }

    // Render HUD
    render_hud(stdout, state, fps, term_width)?;

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

    // Reset color and pad line
    output.push_str("\x1b[0m");
    if width < max_width {
        output.push_str(&" ".repeat(max_width - width));
    }

    write!(stdout, "{}\r\n", output)?;
    Ok(())
}

/// Renders the status line HUD at the bottom of the screen.
fn render_hud(
    stdout: &mut impl Write,
    state: &TunerState,
    fps: f64,
    term_width: u16,
) -> dotmax::Result<()> {
    let inv_on = "\x1b[7m"; // Inverse video
    let inv_off = "\x1b[0m";
    let width = term_width as usize;

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

        for line in &help_lines {
            write!(stdout, "{}{}{}\r\n", inv_on, pad(line, width), inv_off)?;
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

        write!(stdout, "{}{}{}\r\n", inv_on, pad(&line1, width), inv_off)?;
        write!(stdout, "{}{}{}\r\n", inv_on, pad(&line2, width), inv_off)?;
        write!(stdout, "{}{}{}\r\n", inv_on, pad(line3, width), inv_off)?;
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
