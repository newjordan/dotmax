//! Example: Creating Custom Color Schemes with ColorSchemeBuilder
//!
//! This example demonstrates how to create custom color schemes using the
//! builder pattern introduced in Story 5.4. It shows:
//!
//! 1. Building a custom scheme with intensity-based color stops
//! 2. Using the `from_colors()` convenience constructor
//! 3. Rendering gradient bars to visualize the schemes
//!
//! # Usage
//!
//! ```bash
//! cargo run --example custom_scheme
//! ```

use dotmax::color::scheme_builder::ColorSchemeBuilder;
use dotmax::color::schemes::ColorScheme;
use dotmax::Color;

/// Render a horizontal gradient bar for a color scheme
fn render_gradient_bar(scheme: &ColorScheme, width: usize) {
    print!("  ");
    for i in 0..width {
        let intensity = i as f32 / (width - 1) as f32;
        let color = scheme.sample(intensity);

        // Use truecolor ANSI escape codes for background color
        print!("\x1b[48;2;{};{};{}m \x1b[0m", color.r, color.g, color.b);
    }
    println!();
}

/// Print a section header
fn print_header(title: &str) {
    println!("\n\x1b[1;36m{}\x1b[0m", title);
    println!("{}", "─".repeat(60));
}

fn main() -> Result<(), dotmax::DotmaxError> {
    println!("\x1b[1;33m╔══════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;33m║     Custom Color Scheme Builder Demo (Story 5.4)             ║\x1b[0m");
    println!("\x1b[1;33m╚══════════════════════════════════════════════════════════════╝\x1b[0m");

    // ========================================================================
    // Example 1: Fire/Heat Gradient
    // ========================================================================
    print_header("🔥 Fire Gradient (Custom Color Stops)");
    println!("  A heat-based gradient from black through red, orange, yellow to white");
    println!("  Colors placed at specific intensity positions for realistic fire effect");
    println!();

    let fire_scheme = ColorSchemeBuilder::new("fire")
        // Black at 0% intensity (cold/dark)
        .add_color(0.0, Color::rgb(0, 0, 0))
        // Dark red at 20% (ember glow)
        .add_color(0.2, Color::rgb(128, 0, 0))
        // Bright red at 40% (flame base)
        .add_color(0.4, Color::rgb(255, 0, 0))
        // Orange at 60% (flame core)
        .add_color(0.6, Color::rgb(255, 128, 0))
        // Yellow at 80% (hot center)
        .add_color(0.8, Color::rgb(255, 255, 0))
        // White at 100% (intense heat)
        .add_color(1.0, Color::rgb(255, 255, 255))
        .build()?;

    println!(
        "  Scheme: {} ({} color stops)",
        fire_scheme.name(),
        fire_scheme.colors().len()
    );
    render_gradient_bar(&fire_scheme, 60);

    // ========================================================================
    // Example 2: Ocean Gradient
    // ========================================================================
    print_header("🌊 Ocean Gradient (Deep to Surface)");
    println!("  Simulates ocean depth: dark blue depths → turquoise surface → white foam");
    println!();

    let ocean_scheme = ColorSchemeBuilder::new("ocean")
        // Deep ocean (dark blue)
        .add_color(0.0, Color::rgb(0, 0, 50))
        // Mid-depth (blue)
        .add_color(0.3, Color::rgb(0, 50, 150))
        // Shallow water (cyan-blue)
        .add_color(0.6, Color::rgb(0, 150, 200))
        // Near surface (turquoise)
        .add_color(0.8, Color::rgb(64, 224, 208))
        // Surface foam (white)
        .add_color(1.0, Color::rgb(240, 255, 255))
        .build()?;

    println!(
        "  Scheme: {} ({} color stops)",
        ocean_scheme.name(),
        ocean_scheme.colors().len()
    );
    render_gradient_bar(&ocean_scheme, 60);

    // ========================================================================
    // Example 3: Brand Colors (Corporate Palette)
    // ========================================================================
    print_header("🏢 Brand Colors (Corporate Gradient)");
    println!("  Example corporate palette: dark navy → brand blue → accent teal → white");
    println!("  Perfect for data visualizations matching company branding");
    println!();

    let brand_scheme = ColorSchemeBuilder::new("acme_corp")
        // Primary dark (navy)
        .add_color(0.0, Color::rgb(0, 32, 64))
        // Brand blue
        .add_color(0.4, Color::rgb(0, 102, 204))
        // Accent teal
        .add_color(0.7, Color::rgb(0, 180, 180))
        // Light accent
        .add_color(1.0, Color::rgb(200, 240, 255))
        .build()?;

    println!(
        "  Scheme: {} ({} color stops)",
        brand_scheme.name(),
        brand_scheme.colors().len()
    );
    render_gradient_bar(&brand_scheme, 60);

    // ========================================================================
    // Example 4: Using from_colors() Convenience Constructor
    // ========================================================================
    print_header("🎨 Using from_colors() Convenience Constructor");
    println!("  Evenly-spaced colors: just provide the colors, positions auto-calculated");
    println!("  4 colors → positions at 0.0, 0.33, 0.67, 1.0");
    println!();

    let sunset_scheme = ColorScheme::from_colors(
        "sunset",
        vec![
            Color::rgb(25, 25, 112),   // Midnight blue
            Color::rgb(255, 69, 0),    // Red-orange
            Color::rgb(255, 215, 0),   // Gold
            Color::rgb(255, 250, 205), // Lemon chiffon
        ],
    )?;

    println!(
        "  Scheme: {} ({} color stops)",
        sunset_scheme.name(),
        sunset_scheme.colors().len()
    );
    render_gradient_bar(&sunset_scheme, 60);

    // ========================================================================
    // Example 5: Neon Cyberpunk Gradient
    // ========================================================================
    print_header("💜 Neon Cyberpunk Gradient");
    println!("  Vibrant neon colors with sharp transitions for that retro-future aesthetic");
    println!();

    let neon_scheme = ColorSchemeBuilder::new("cyberpunk")
        // Deep purple/black
        .add_color(0.0, Color::rgb(20, 0, 40))
        // Electric purple
        .add_color(0.25, Color::rgb(138, 43, 226))
        // Hot pink
        .add_color(0.5, Color::rgb(255, 20, 147))
        // Cyan
        .add_color(0.75, Color::rgb(0, 255, 255))
        // Electric blue
        .add_color(1.0, Color::rgb(0, 191, 255))
        .build()?;

    println!(
        "  Scheme: {} ({} color stops)",
        neon_scheme.name(),
        neon_scheme.colors().len()
    );
    render_gradient_bar(&neon_scheme, 60);

    // ========================================================================
    // Example 6: Earth Tones
    // ========================================================================
    print_header("🌍 Earth Tones (Natural Gradient)");
    println!("  Warm, natural colors: deep brown → terracotta → sand → cream");
    println!();

    let earth_scheme = ColorScheme::from_colors(
        "earth_tones",
        vec![
            Color::rgb(59, 36, 27),    // Deep brown
            Color::rgb(139, 69, 19),   // Saddle brown
            Color::rgb(205, 133, 63),  // Peru/terracotta
            Color::rgb(222, 184, 135), // Burlywood
            Color::rgb(255, 248, 220), // Cornsilk
        ],
    )?;

    println!(
        "  Scheme: {} ({} color stops)",
        earth_scheme.name(),
        earth_scheme.colors().len()
    );
    render_gradient_bar(&earth_scheme, 60);

    // ========================================================================
    // Sampling Demonstration
    // ========================================================================
    print_header("📊 Sampling the Fire Scheme at Different Intensities");
    println!();

    let intensities = [0.0, 0.25, 0.5, 0.75, 1.0];
    for &intensity in &intensities {
        let color = fire_scheme.sample(intensity);
        print!(
            "  intensity={:.2} → \x1b[48;2;{};{};{}m    \x1b[0m RGB({:3}, {:3}, {:3})\n",
            intensity, color.r, color.g, color.b, color.r, color.g, color.b
        );
    }

    // ========================================================================
    // Summary
    // ========================================================================
    print_header("✅ Summary");
    println!("  • ColorSchemeBuilder allows precise color stop placement");
    println!("  • from_colors() provides a simple API for evenly-spaced gradients");
    println!("  • Both methods produce ColorScheme instances compatible with sample()");
    println!("  • Custom schemes enable brand-specific and artistic visualizations");
    println!();

    Ok(())
}
