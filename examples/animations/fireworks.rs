//! Fireworks Particle System Animation - Story 6.6 AC4
//!
//! Demonstrates a basic particle system with firework explosions.
//! Shows multiple animated particles with color fading effects.
//!
//! # Features
//! - Particle system with position, velocity, color, and lifetime
//! - Gravity and drag physics on particles
//! - Color fading over particle lifetime
//! - Random burst patterns at random positions
//!
//! # Usage
//! ```bash
//! cargo run --example fireworks
//! ```
//!
//! # Controls
//! - Press 'q' or Ctrl+C to exit gracefully

// Allow certain clippy warnings that are acceptable in examples
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use dotmax::animation::FrameTimer;
use dotmax::color::schemes::{get_scheme, rainbow};
use dotmax::{BrailleGrid, Color, ColorScheme, TerminalRenderer};
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::Duration;

/// Terminal dimensions in cells
const WIDTH: usize = 80;
const HEIGHT: usize = 24;

/// Physics constants
const GRAVITY: f64 = 0.15; // Gravity acceleration
const DRAG: f64 = 0.98; // Velocity damping (air resistance)
const SPAWN_INTERVAL: u64 = 60; // Frames between new fireworks
const PARTICLES_PER_EXPLOSION: usize = 30; // Particles in each burst

/// Target frame rate
const TARGET_FPS: u32 = 60;

/// A single particle in the firework system
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    lifetime: u32, // Frames remaining
    max_lifetime: u32,
    color: Color,
}

impl Particle {
    /// Update particle physics for one frame
    fn update(&mut self) {
        // Apply gravity
        self.vy += GRAVITY;

        // Apply drag (air resistance)
        self.vx *= DRAG;
        self.vy *= DRAG;

        // Update position
        self.x += self.vx;
        self.y += self.vy;

        // Decrease lifetime
        if self.lifetime > 0 {
            self.lifetime -= 1;
        }
    }

    /// Check if particle is still alive
    const fn is_alive(&self) -> bool {
        self.lifetime > 0
    }

    /// Get current color with fading based on remaining lifetime
    fn faded_color(&self) -> Color {
        let life_ratio = f64::from(self.lifetime) / f64::from(self.max_lifetime);

        // Fade color by reducing RGB values
        let fade = life_ratio.clamp(0.0, 1.0);
        Color::rgb(
            (f64::from(self.color.r) * fade) as u8,
            (f64::from(self.color.g) * fade) as u8,
            (f64::from(self.color.b) * fade) as u8,
        )
    }

    /// Check if particle is within visible bounds
    fn is_visible(&self, dot_width: usize, dot_height: usize) -> bool {
        self.x >= 0.0
            && self.x < dot_width as f64
            && self.y >= 0.0
            && self.y < dot_height as f64
    }
}

/// Simple random number generator (Linear Congruential Generator)
/// Using this to avoid external dependencies
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate next random u64
    fn next_u64(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        self.state
    }

    /// Generate random f64 in range [0.0, 1.0)
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generate random f64 in range [min, max)
    fn range(&mut self, min: f64, max: f64) -> f64 {
        (max - min).mul_add(self.next_f64(), min)
    }
}

/// Create a burst of particles at the given position
fn create_explosion(
    x: f64,
    y: f64,
    rng: &mut SimpleRng,
    scheme: &ColorScheme,
) -> Vec<Particle> {
    let mut particles = Vec::with_capacity(PARTICLES_PER_EXPLOSION);

    for _ in 0..PARTICLES_PER_EXPLOSION {
        // Random angle and speed for radial burst
        let angle = rng.range(0.0, 2.0 * PI);
        let speed = rng.range(1.5, 4.0);

        // Random lifetime for variation
        let lifetime = rng.range(30.0, 80.0) as u32;

        // Get color from scheme based on angle (creates color variety)
        let color_intensity = (angle / (2.0 * PI)) as f32;
        let color = scheme.sample(color_intensity);

        particles.push(Particle {
            x,
            y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            lifetime,
            max_lifetime: lifetime,
            color,
        });
    }

    particles
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), Hide, MoveTo(0, 0))?;

    // Create renderer and timer
    let mut renderer = TerminalRenderer::new()?;
    let mut timer = FrameTimer::new(TARGET_FPS);

    // Particle system state
    let mut particles: Vec<Particle> = Vec::new();
    let mut rng = SimpleRng::new(42); // Deterministic seed for reproducibility

    // Color schemes to cycle through
    let schemes = [
        rainbow(),
        get_scheme("heat_map").unwrap(),
        get_scheme("cyan_magenta").unwrap(),
        get_scheme("blue_purple").unwrap(),
    ];
    let mut current_scheme = 0;

    // Animation state
    let mut frame: u64 = 0;
    let dot_width = WIDTH * 2;
    let dot_height = HEIGHT * 4;

    // Main animation loop
    loop {
        // Check for exit key
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Esc
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    break;
                }
            }
        }

        // Spawn new firework periodically
        if frame % SPAWN_INTERVAL == 0 {
            // Random position in upper half of screen
            let spawn_x = rng.range(20.0, (dot_width - 20) as f64);
            let spawn_y = rng.range(10.0, (dot_height / 2) as f64);

            // Create explosion with current color scheme
            let mut new_particles = create_explosion(
                spawn_x,
                spawn_y,
                &mut rng,
                &schemes[current_scheme],
            );
            particles.append(&mut new_particles);

            // Cycle to next color scheme
            current_scheme = (current_scheme + 1) % schemes.len();
        }

        // Update all particles
        for particle in &mut particles {
            particle.update();
        }

        // Remove dead or off-screen particles
        particles.retain(|p| p.is_alive() && p.is_visible(dot_width, dot_height));

        // Create fresh grid each frame
        let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;

        // Draw all particles
        for particle in &particles {
            let x = particle.x as usize;
            let y = particle.y as usize;

            if x < dot_width && y < dot_height {
                let color = particle.faded_color();
                // Set the dot first, then color the cell
                grid.set_dot(x, y)?;
                // Color is per-cell, so convert dot coords to cell coords
                let cell_x = x / 2;
                let cell_y = y / 4;
                grid.set_cell_color(cell_x, cell_y, color)?;
            }
        }

        // Render to terminal
        renderer.render(&grid)?;

        // Display status text below the braille grid
        execute!(
            stdout,
            MoveTo(0, HEIGHT as u16 + 1),
            Print(format!(
                "Fireworks | Particles: {:4} | Frame: {:5} | FPS: {:5.1} | [q]uit     ",
                particles.len(),
                frame,
                timer.actual_fps()
            ))
        )?;
        stdout.flush()?;

        frame += 1;
        timer.wait_for_next_frame();
    }

    // Cleanup
    execute!(stdout, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    renderer.cleanup()?;
    disable_raw_mode()?;

    println!("Fireworks Demo Complete!");
    println!("Rendered {frame} frames of particle effects.");
    println!("\nFeatures demonstrated:");
    println!("- Particle system (position, velocity, lifetime)");
    println!("- Gravity and drag physics");
    println!("- Color fading over lifetime");
    println!("- Multiple color schemes cycling");
    println!("- Random burst patterns");

    Ok(())
}
