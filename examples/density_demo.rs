//! Density Rendering Demonstration
//!
//! This example demonstrates character density-based rendering using various
//! predefined density sets. It shows how to create gradients and render intensity
//! buffers as ASCII-art style visualizations.
//!
//! Run with: `cargo run --example density_demo`

use dotmax::density::DensitySet;
use dotmax::BrailleGrid;

fn main() -> Result<(), dotmax::DotmaxError> {
    println!("=== Dotmax Density Rendering Demo ===\n");

    // Create a grid for demonstrations (40×10 cells for visual clarity)
    let width = 40;
    let height = 10;

    println!(
        "Grid dimensions: {}×{} cells = {} total cells\n",
        width,
        height,
        width * height
    );

    // Demo 1: Horizontal gradient with ASCII density
    println!("--- Demo 1: Horizontal Gradient (ASCII Density) ---");
    demo_gradient(
        "ASCII",
        DensitySet::ascii(),
        width,
        height,
        GradientType::Horizontal,
    )?;
    println!();

    // Demo 2: Horizontal gradient with Simple density
    println!("--- Demo 2: Horizontal Gradient (Simple Density) ---");
    demo_gradient(
        "Simple",
        DensitySet::simple(),
        width,
        height,
        GradientType::Horizontal,
    )?;
    println!();

    // Demo 3: Horizontal gradient with Blocks density
    println!("--- Demo 3: Horizontal Gradient (Blocks Density) ---");
    demo_gradient(
        "Blocks",
        DensitySet::blocks(),
        width,
        height,
        GradientType::Horizontal,
    )?;
    println!();

    // Demo 4: Horizontal gradient with Braille density
    println!("--- Demo 4: Horizontal Gradient (Braille Density) ---");
    demo_gradient(
        "Braille",
        DensitySet::braille(),
        width,
        height,
        GradientType::Horizontal,
    )?;
    println!();

    // Demo 5: Vertical gradient
    println!("--- Demo 5: Vertical Gradient (ASCII Density) ---");
    demo_gradient(
        "ASCII",
        DensitySet::ascii(),
        width,
        height,
        GradientType::Vertical,
    )?;
    println!();

    // Demo 6: Radial gradient
    println!("--- Demo 6: Radial Gradient (ASCII Density) ---");
    demo_gradient(
        "ASCII",
        DensitySet::ascii(),
        width,
        height,
        GradientType::Radial,
    )?;
    println!();

    // Demo 7: Custom density set
    println!("--- Demo 7: Custom Density Set (Emoji) ---");
    let custom = DensitySet::new(
        "Emoji".to_string(),
        vec!['⚫', '🌑', '🌒', '🌓', '🌔', '🌕', '⚪'],
    )?;
    demo_gradient("Emoji", custom, width, height, GradientType::Horizontal)?;
    println!();

    // Demo 8: Intensity mapping demonstration
    println!("--- Demo 8: Intensity Mapping Test ---");
    demo_intensity_mapping();
    println!();

    println!("=== Demo Complete ===");
    Ok(())
}

/// Gradient type for generation
enum GradientType {
    Horizontal,
    Vertical,
    Radial,
}

/// Generate and render a gradient using the specified density set
fn demo_gradient(
    name: &str,
    density_set: DensitySet,
    width: usize,
    height: usize,
    gradient_type: GradientType,
) -> Result<(), dotmax::DotmaxError> {
    // Generate intensity buffer based on gradient type
    let intensities: Vec<f32> = match gradient_type {
        GradientType::Horizontal => {
            // Horizontal gradient: intensity increases left to right
            (0..height * width)
                .map(|i| {
                    let x = i % width;
                    x as f32 / (width - 1) as f32
                })
                .collect()
        }
        GradientType::Vertical => {
            // Vertical gradient: intensity increases top to bottom
            (0..height * width)
                .map(|i| {
                    let y = i / width;
                    y as f32 / (height - 1) as f32
                })
                .collect()
        }
        GradientType::Radial => {
            // Radial gradient: intensity increases from center outward
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let max_radius = ((center_x * center_x) + (center_y * center_y)).sqrt();

            (0..height * width)
                .map(|i| {
                    let x = (i % width) as f32;
                    let y = (i / width) as f32;
                    let dx = x - center_x;
                    let dy = y - center_y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    (distance / max_radius).min(1.0)
                })
                .collect()
        }
    };

    // Create grid and render density
    let mut grid = BrailleGrid::new(width, height)?;
    grid.render_density(&intensities, &density_set)?;

    // Display character mapping
    println!(
        "Density set: {} ({} characters)",
        name,
        density_set.characters.len()
    );
    println!(
        "Character progression: {} → {}",
        density_set.characters.first().unwrap(),
        density_set.characters.last().unwrap()
    );

    // TODO: Once render_density properly renders characters to grid,
    // we can use TerminalRenderer to display the result:
    // let mut renderer = TerminalRenderer::new()?;
    // renderer.render(&grid)?;

    // For now, manually display the density mapping
    println!("Visual preview:");
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let intensity = intensities[idx];
            let char = density_set.map(intensity);
            print!("{}", char);
        }
        println!();
    }

    Ok(())
}

/// Demonstrate intensity-to-character mapping at various intensity levels
fn demo_intensity_mapping() {
    let density_sets = vec![
        ("ASCII", DensitySet::ascii()),
        ("Simple", DensitySet::simple()),
        ("Blocks", DensitySet::blocks()),
        ("Braille", DensitySet::braille()),
    ];

    println!("Intensity mapping comparison:\n");
    println!("Intensity | ASCII | Simple | Blocks | Braille");
    println!("----------|-------|--------|--------|--------");

    let test_intensities = vec![0.0, 0.25, 0.5, 0.75, 1.0];

    for &intensity in &test_intensities {
        print!("  {:5.2}   |", intensity);
        for (_, density) in &density_sets {
            let char = density.map(intensity);
            print!("   {}   |", char);
        }
        println!();
    }

    println!("\nEdge case testing (out-of-range intensities):");
    println!("Intensity | Clamped Result");
    println!("----------|---------------");

    let edge_cases = vec![
        ("   -0.5", -0.5_f32),
        ("    1.5", 1.5_f32),
        ("  100.0", 100.0_f32),
        (" -100.0", -100.0_f32),
    ];

    for (label, intensity) in edge_cases {
        let char = DensitySet::ascii().map(intensity);
        println!(
            "{} | '{}' (clamped to {})",
            label,
            char,
            intensity.clamp(0.0, 1.0)
        );
    }
}
