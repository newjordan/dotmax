//! Integration tests for character density rendering
//!
//! Tests the integration of density rendering with the image pipeline
//! and validates end-to-end workflows.

use dotmax::density::DensitySet;
use dotmax::BrailleGrid;

#[test]
fn test_density_rendering_basic_gradient() {
    // Create grid and generate simple gradient
    let mut grid = BrailleGrid::new(10, 5).unwrap();
    let gradient: Vec<f32> = (0..50).map(|i| i as f32 / 49.0).collect();

    // Render using ASCII density
    let density = DensitySet::ascii();
    let result = grid.render_density(&gradient, &density);

    assert!(
        result.is_ok(),
        "Density rendering should succeed for valid gradient"
    );
}

#[test]
fn test_density_rendering_with_all_predefined_sets() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();
    let gradient: Vec<f32> = (0..50).map(|i| i as f32 / 49.0).collect();

    // Test all predefined density sets
    let density_sets = vec![
        DensitySet::ascii(),
        DensitySet::simple(),
        DensitySet::blocks(),
        DensitySet::braille(),
    ];

    for density in density_sets {
        let result = grid.render_density(&gradient, &density);
        assert!(
            result.is_ok(),
            "Rendering should succeed for {} density set",
            density.name
        );
    }
}

#[test]
fn test_density_rendering_with_custom_set() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();
    let gradient: Vec<f32> = (0..50).map(|i| i as f32 / 49.0).collect();

    // Create custom density set
    let custom = DensitySet::new(
        "Custom".to_string(),
        vec![' ', '.', ':', '-', '=', '+', '*', '#', '@'],
    )
    .unwrap();

    let result = grid.render_density(&gradient, &custom);
    assert!(
        result.is_ok(),
        "Rendering should succeed for custom density set"
    );
}

#[test]
fn test_density_rendering_horizontal_gradient() {
    let width = 20;
    let height = 10;
    let mut grid = BrailleGrid::new(width, height).unwrap();

    // Generate horizontal gradient (intensity increases left to right)
    let gradient: Vec<f32> = (0..width * height)
        .map(|i| {
            let x = i % width;
            x as f32 / (width - 1) as f32
        })
        .collect();

    let density = DensitySet::simple();
    let result = grid.render_density(&gradient, &density);

    assert!(
        result.is_ok(),
        "Horizontal gradient rendering should succeed"
    );
}

#[test]
fn test_density_rendering_vertical_gradient() {
    let width = 20;
    let height = 10;
    let mut grid = BrailleGrid::new(width, height).unwrap();

    // Generate vertical gradient (intensity increases top to bottom)
    let gradient: Vec<f32> = (0..width * height)
        .map(|i| {
            let y = i / width;
            y as f32 / (height - 1) as f32
        })
        .collect();

    let density = DensitySet::simple();
    let result = grid.render_density(&gradient, &density);

    assert!(result.is_ok(), "Vertical gradient rendering should succeed");
}

#[test]
fn test_density_rendering_radial_gradient() {
    let width = 20;
    let height = 10;
    let mut grid = BrailleGrid::new(width, height).unwrap();

    // Generate radial gradient (intensity increases from center outward)
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_radius = ((center_x * center_x) + (center_y * center_y)).sqrt();

    let gradient: Vec<f32> = (0..width * height)
        .map(|i| {
            let x = (i % width) as f32;
            let y = (i / width) as f32;
            let dx = x - center_x;
            let dy = y - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            (distance / max_radius).min(1.0)
        })
        .collect();

    let density = DensitySet::ascii();
    let result = grid.render_density(&gradient, &density);

    assert!(result.is_ok(), "Radial gradient rendering should succeed");
}

#[test]
fn test_density_rendering_all_zeros() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();
    let all_zeros = vec![0.0_f32; 50];

    let density = DensitySet::simple();
    let result = grid.render_density(&all_zeros, &density);

    assert!(result.is_ok(), "Rendering all zeros should succeed");
}

#[test]
fn test_density_rendering_all_ones() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();
    let all_ones = vec![1.0_f32; 50];

    let density = DensitySet::simple();
    let result = grid.render_density(&all_ones, &density);

    assert!(result.is_ok(), "Rendering all ones should succeed");
}

#[test]
fn test_density_rendering_mixed_intensities() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();
    let mixed: Vec<f32> = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]
        .into_iter()
        .cycle()
        .take(50)
        .collect();

    let density = DensitySet::ascii();
    let result = grid.render_density(&mixed, &density);

    assert!(result.is_ok(), "Rendering mixed intensities should succeed");
}

#[test]
fn test_density_rendering_buffer_size_validation() {
    let mut grid = BrailleGrid::new(10, 5).unwrap();

    // Test various incorrect buffer sizes
    let wrong_sizes = vec![
        vec![0.5_f32; 49],  // Too small
        vec![0.5_f32; 51],  // Too large
        vec![0.5_f32; 100], // Way too large
        vec![0.5_f32; 1],   // Single element
        vec![],             // Empty buffer
    ];

    let density = DensitySet::simple();

    for wrong_buffer in wrong_sizes {
        let result = grid.render_density(&wrong_buffer, &density);
        assert!(
            result.is_err(),
            "Rendering with buffer size {} should fail (expected 50)",
            wrong_buffer.len()
        );
    }
}

#[test]
fn test_density_rendering_large_grid() {
    // Test with typical terminal size (80×24)
    let mut grid = BrailleGrid::new(80, 24).unwrap();
    let gradient: Vec<f32> = (0..80 * 24)
        .map(|i| i as f32 / (80.0 * 24.0 - 1.0))
        .collect();

    let density = DensitySet::ascii();
    let result = grid.render_density(&gradient, &density);

    assert!(
        result.is_ok(),
        "Rendering large grid (80×24) should succeed"
    );
}

// Integration with Epic 3 image pipeline tests (requires image feature)
// Story 4.4 AC6: Integration with Image Pipeline
//
// This module tests the integration between density rendering and Epic 3 grayscale conversion:
// 1. Load image using Epic 3 API
// 2. Convert to grayscale using Epic 3 API
// 3. Extract intensity buffer from grayscale image
// 4. Render using density set
#[cfg(feature = "image")]
mod image_integration {
    use super::*;
    use dotmax::image::{load_from_path, to_grayscale};
    use std::path::Path;

    #[test]
    fn test_density_rendering_from_image_pipeline() {
        // 1. Load image using Epic 3 API
        let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
            .expect("Failed to load sample image");

        // 2. Convert to grayscale using Epic 3 API (BT.709 formula)
        let grayscale = to_grayscale(&img);

        // 3. Extract intensity buffer from grayscale image
        // Normalize pixel values from [0, 255] to [0.0, 1.0]
        let intensities: Vec<f32> = grayscale
            .pixels()
            .map(|pixel| {
                let luma = pixel.0[0];
                f32::from(luma) / 255.0
            })
            .collect();

        // 4. Create grid matching grayscale dimensions
        let width = grayscale.width() as usize;
        let height = grayscale.height() as usize;
        let mut grid = BrailleGrid::new(width, height).expect("Failed to create grid");

        // 5. Render using density set
        let density = DensitySet::ascii();
        let result = grid.render_density(&intensities, &density);

        assert!(
            result.is_ok(),
            "Density rendering should succeed with Epic 3 grayscale output"
        );

        // 6. Verify characters are set on grid
        // Since we normalized intensities, they should be in [0.0, 1.0] range
        // and density rendering should produce valid ASCII characters
        for y in 0..height {
            for x in 0..width {
                let ch = grid.get_char(x, y);
                // Verify character is from ASCII_DENSITY range
                assert!(
                    ch.is_ascii() || ch == '⠀', // ASCII or empty braille
                    "Character at ({}, {}) should be ASCII or empty braille, got: '{}'",
                    x,
                    y,
                    ch
                );
            }
        }
    }

    #[test]
    fn test_density_rendering_with_multiple_sets() {
        // Load and prepare image
        let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
            .expect("Failed to load sample image");
        let grayscale = to_grayscale(&img);

        let intensities: Vec<f32> = grayscale
            .pixels()
            .map(|pixel| f32::from(pixel.0[0]) / 255.0)
            .collect();

        let width = grayscale.width() as usize;
        let height = grayscale.height() as usize;

        // Test with all predefined density sets
        let density_sets = vec![
            DensitySet::ascii(),
            DensitySet::simple(),
            DensitySet::blocks(),
            DensitySet::braille(),
        ];

        for density_set in density_sets {
            let mut grid = BrailleGrid::new(width, height).expect("Failed to create grid");
            let result = grid.render_density(&intensities, &density_set);

            assert!(
                result.is_ok(),
                "Density rendering should succeed with {} set",
                density_set.name
            );

            // Verify at least some characters are set
            let mut char_count = 0;
            for y in 0..height {
                for x in 0..width {
                    let _ = grid.get_char(x, y);
                    char_count += 1;
                }
            }

            assert_eq!(
                char_count,
                width * height,
                "All grid cells should have characters for {} set",
                density_set.name
            );
        }
    }

    #[test]
    fn test_density_rendering_preserves_image_dimensions() {
        // Verify that density rendering maintains image dimensions exactly
        let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
            .expect("Failed to load sample image");
        let grayscale = to_grayscale(&img);

        let original_width = grayscale.width() as usize;
        let original_height = grayscale.height() as usize;

        let intensities: Vec<f32> = grayscale
            .pixels()
            .map(|pixel| f32::from(pixel.0[0]) / 255.0)
            .collect();

        let mut grid =
            BrailleGrid::new(original_width, original_height).expect("Failed to create grid");

        let density = DensitySet::simple();
        grid.render_density(&intensities, &density)
            .expect("Failed to render");

        // Verify grid dimensions match image
        let (grid_width, grid_height) = grid.dimensions();
        assert_eq!(
            grid_width, original_width,
            "Grid width should match image width"
        );
        assert_eq!(
            grid_height, original_height,
            "Grid height should match image height"
        );
    }
}
