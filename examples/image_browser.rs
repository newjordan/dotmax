//! Interactive Image Browser - Test UI for Image Rendering
//!
//! This example provides an interactive terminal UI for testing image rendering
//! with different settings and browsing through test images.
//!
//! # Controls
//!
//! - **Left/Right Arrow**: Previous/Next image
//! - **C**: Cycle color mode (Monochrome → Grayscale → `TrueColor`)
//! - **D**: Cycle dithering algorithm (Floyd-Steinberg → Bayer → Atkinson → None)
//! - **b/B**: Increase/Decrease brightness by 0.05 (lowercase = up, uppercase = down)
//! - **t/T**: Increase/Decrease contrast by 0.05 (lowercase = up, uppercase = down)
//! - **g/G**: Increase/Decrease gamma by 0.05 (lowercase = up, uppercase = down)
//! - **R**: Reset all adjustments to defaults
//! - **Q or Esc**: Quit

#![allow(clippy::uninlined_format_args, clippy::cast_lossless, clippy::unnecessary_wraps, clippy::needless_pass_by_ref_mut, clippy::missing_const_for_fn, clippy::items_after_statements, clippy::map_unwrap_or)]
//!
//! # Usage
//!
//! ```bash
//! cargo run --example image_browser --features image,svg
//! ```

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use dotmax::image::{ColorMode, DitheringMethod, ImageRenderer};
use dotmax::TerminalRenderer;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
struct RenderSettings {
    color_mode: ColorMode,
    dithering: DitheringMethod,
    brightness: f32,
    contrast: f32,
    gamma: f32,
}

impl RenderSettings {
    fn new() -> Self {
        Self {
            color_mode: ColorMode::Monochrome,
            dithering: DitheringMethod::FloydSteinberg,
            brightness: 1.0,
            contrast: 1.0,
            gamma: 1.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn cycle_color_mode(&mut self) {
        self.color_mode = match self.color_mode {
            ColorMode::Monochrome => ColorMode::Grayscale,
            ColorMode::Grayscale => ColorMode::TrueColor,
            ColorMode::TrueColor => ColorMode::Monochrome,
        };
    }

    fn cycle_dithering(&mut self) {
        self.dithering = match self.dithering {
            DitheringMethod::FloydSteinberg => DitheringMethod::Bayer,
            DitheringMethod::Bayer => DitheringMethod::Atkinson,
            DitheringMethod::Atkinson => DitheringMethod::None,
            DitheringMethod::None => DitheringMethod::FloydSteinberg,
        };
    }

    fn adjust_brightness(&mut self, delta: f32) {
        self.brightness = (self.brightness + delta).clamp(0.0, 2.0);
        // Round to 2 decimal places for cleaner display
        self.brightness = (self.brightness * 100.0).round() / 100.0;
    }

    fn adjust_contrast(&mut self, delta: f32) {
        self.contrast = (self.contrast + delta).clamp(0.0, 2.0);
        // Round to 2 decimal places for cleaner display
        self.contrast = (self.contrast * 100.0).round() / 100.0;
    }

    fn adjust_gamma(&mut self, delta: f32) {
        self.gamma = (self.gamma + delta).clamp(0.1, 3.0);
        // Round to 2 decimal places for cleaner display
        self.gamma = (self.gamma * 100.0).round() / 100.0;
    }

    fn display_string(&self) -> String {
        format!(
            "Color: {:?} | Dither: {:?} | Brightness: {:.1} | Contrast: {:.1} | Gamma: {:.1}",
            self.color_mode, self.dithering, self.brightness, self.contrast, self.gamma
        )
    }
}

struct ImageBrowser {
    images: Vec<PathBuf>,
    current_index: usize,
    settings: RenderSettings,
    renderer: TerminalRenderer,
}

impl ImageBrowser {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let images = Self::discover_images()?;

        if images.is_empty() {
            return Err("No images found in tests/fixtures/images or tests/fixtures/svg".into());
        }

        Ok(Self {
            images,
            current_index: 0,
            settings: RenderSettings::new(),
            renderer: TerminalRenderer::new()?,
        })
    }

    fn discover_images() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut images = Vec::new();

        // Scan tests/fixtures/images directory
        let images_dir = PathBuf::from("tests/fixtures/images");
        if images_dir.exists() {
            Self::scan_directory(&images_dir, &mut images)?;
        }

        // Scan tests/fixtures/svg directory (if svg feature enabled)
        #[cfg(feature = "svg")]
        {
            let svg_dir = PathBuf::from("tests/fixtures/svg");
            if svg_dir.exists() {
                Self::scan_directory(&svg_dir, &mut images)?;
            }
        }

        // Filter out known test files that are intentionally corrupted
        images.retain(|path| {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            !filename.contains("corrupted") && !filename.contains("malformed")
        });

        images.sort();
        Ok(images)
    }

    fn scan_directory(dir: &PathBuf, images: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tiff" | "svg") {
                        images.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    fn current_image(&self) -> &PathBuf {
        &self.images[self.current_index]
    }

    fn next_image(&mut self) {
        self.current_index = (self.current_index + 1) % self.images.len();
    }

    fn prev_image(&mut self) {
        self.current_index = if self.current_index == 0 {
            self.images.len() - 1
        } else {
            self.current_index - 1
        };
    }

    fn render_current(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear screen
        execute!(io::stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        let path = self.current_image().clone();
        let is_svg = path.extension().map(|e| e == "svg").unwrap_or(false);

        // Try to render image with current settings
        match self.try_render_image(is_svg) {
            Ok(grid) => {
                // Render to terminal
                self.renderer.render(&grid)?;
            }
            Err(e) => {
                // Display error message
                println!("\n\n");
                println!("╔════════════════════════════════════════════════════════════════════╗");
                println!("║                          ERROR LOADING IMAGE                       ║");
                println!("╠════════════════════════════════════════════════════════════════════╣");
                println!("║ Image: {:60} ║", path.display().to_string());
                println!("║ Error: {:60} ║", format!("{}", e));
                println!("╠════════════════════════════════════════════════════════════════════╣");
                println!("║ Press ← or → to try another image, or Q to quit                   ║");
                println!("╚════════════════════════════════════════════════════════════════════╝");
            }
        }

        // Display UI footer
        self.display_footer()?;

        // Flush output
        use std::io::Write;
        io::stdout().flush()?;

        Ok(())
    }

    fn try_render_image(&mut self, is_svg: bool) -> Result<dotmax::BrailleGrid, Box<dyn std::error::Error>> {
        let path = self.current_image();

        // Render image with current settings
        let mut builder = ImageRenderer::new()
            .dithering(self.settings.dithering)
            .color_mode(self.settings.color_mode);

        // Apply adjustments if not default
        if (self.settings.brightness - 1.0).abs() > 0.001 {
            builder = builder.brightness(self.settings.brightness)?;
        }
        if (self.settings.contrast - 1.0).abs() > 0.001 {
            builder = builder.contrast(self.settings.contrast)?;
        }
        if (self.settings.gamma - 1.0).abs() > 0.001 {
            builder = builder.gamma(self.settings.gamma)?;
        }

        // Load image (SVG or raster)
        #[cfg(feature = "svg")]
        let builder = if is_svg {
            let (width, height) = self.renderer.get_terminal_size()?;
            builder.load_svg_from_path(path, width as u32 * 2, height as u32 * 4)?
        } else {
            builder.load_from_path(path)?
        };

        #[cfg(not(feature = "svg"))]
        let builder = builder.load_from_path(path)?;

        let mut builder = builder.resize_to_terminal()?;
        let grid = builder.render()?;

        Ok(grid)
    }

    fn display_footer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (_width, height) = self.renderer.get_terminal_size()?;

        // Move cursor to bottom area (leave some space)
        if height > 10 {
            execute!(io::stdout(), cursor::MoveTo(0, height.saturating_sub(8)))?;
        } else {
            println!("\n");
        }

        // Display current image info
        println!("\n┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│ Image: {}/{} - {}",
            self.current_index + 1,
            self.images.len(),
            self.current_image().display()
        );
        println!("│ {}", self.settings.display_string());
        println!("│");
        println!("│ Controls: ← → (prev/next) | C (color) | D (dither) | R (reset) | Q (quit)");
        println!("│ Adjust:   b/B (+/- brightness) | t/T (+/- contrast) | g/G (+/- gamma)");
        println!("└─────────────────────────────────────────────────────────────────────────────┘");

        Ok(())
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Enable raw mode for key input
        terminal::enable_raw_mode()?;

        // Initial render
        self.render_current()?;

        loop {
            // Poll for events with timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.handle_key(key)? {
                        ControlFlow::Continue => {
                            self.render_current()?;
                        }
                        ControlFlow::Skip => {
                            // No render needed
                        }
                        ControlFlow::Quit => break,
                    }
                }
            }
        }

        // Cleanup
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        println!("Image browser closed.");

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<ControlFlow, Box<dyn std::error::Error>> {
        // Only process key press events, ignore release
        // On some systems, we get Press + Release, on others we might get Repeat
        // We want to ignore Release events specifically (skip re-render)
        if matches!(key.kind, KeyEventKind::Release) {
            return Ok(ControlFlow::Skip);
        }

        match key.code {
            // Navigation
            KeyCode::Left => {
                self.prev_image();
                Ok(ControlFlow::Continue)
            }
            KeyCode::Right => {
                self.next_image();
                Ok(ControlFlow::Continue)
            }

            // Settings - cycle through options
            KeyCode::Char('c' | 'C') => {
                self.settings.cycle_color_mode();
                Ok(ControlFlow::Continue)
            }
            KeyCode::Char('d' | 'D') => {
                self.settings.cycle_dithering();
                Ok(ControlFlow::Continue)
            }

            // Brightness (±0.05 for finer control)
            KeyCode::Char('b') => {
                self.settings.adjust_brightness(0.05);
                Ok(ControlFlow::Continue)
            }
            KeyCode::Char('B') => {
                self.settings.adjust_brightness(-0.05);
                Ok(ControlFlow::Continue)
            }

            // Contrast (±0.05 for finer control)
            KeyCode::Char('t') => {
                self.settings.adjust_contrast(0.05);
                Ok(ControlFlow::Continue)
            }
            KeyCode::Char('T') => {
                self.settings.adjust_contrast(-0.05);
                Ok(ControlFlow::Continue)
            }

            // Gamma (±0.05 for finer control)
            KeyCode::Char('g') => {
                self.settings.adjust_gamma(0.05);
                Ok(ControlFlow::Continue)
            }
            KeyCode::Char('G') => {
                self.settings.adjust_gamma(-0.05);
                Ok(ControlFlow::Continue)
            }

            // Reset
            KeyCode::Char('r' | 'R') => {
                self.settings.reset();
                Ok(ControlFlow::Continue)
            }

            // Quit
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => {
                Ok(ControlFlow::Quit)
            }

            // Unhandled keys - no re-render needed
            _ => Ok(ControlFlow::Skip),
        }
    }
}

enum ControlFlow {
    Continue, // Re-render needed
    Skip,     // No re-render needed
    Quit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Interactive Image Browser ===");
    println!("Scanning for images in tests/fixtures/...");

    let mut browser = ImageBrowser::new()?;

    println!("Found {} images", browser.images.len());
    println!("Starting browser... (Press Q to quit)");

    std::thread::sleep(Duration::from_secs(1));

    browser.run()?;

    Ok(())
}
