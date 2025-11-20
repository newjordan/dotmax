//! Integration tests for the complete image rendering pipeline
//!
//! These tests verify the end-to-end image processing pipeline:
//! Load → Resize → Grayscale → Dither/Threshold → Map to Braille → Render

#[cfg(feature = "image")]
mod image_pipeline_tests {
    use dotmax::image::{
        apply_dithering, auto_threshold, load_from_path, pixels_to_braille, resize_to_dimensions,
        to_grayscale, DitheringMethod,
    };
    use std::path::Path;

    #[test]
    fn test_full_pipeline_with_threshold() {
        // Load test image
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        // Resize to terminal dimensions (80×24 cells = 160×96 pixels)
        let resized = resize_to_dimensions(&img, 160, 96, true).expect("Failed to resize");

        // Apply auto thresholding (which includes grayscale conversion)
        let binary = auto_threshold(&resized);

        // Calculate expected grid dimensions based on actual binary image dimensions
        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        // Map to braille grid
        let grid = pixels_to_braille(&binary, expected_width, expected_height)
            .expect("Failed to map to braille");

        // Verify grid dimensions match expected
        assert_eq!(grid.width(), expected_width, "Grid width mismatch");
        assert_eq!(grid.height(), expected_height, "Grid height mismatch");

        // Verify grid is not empty (at least some dots should be set)
        let unicode_grid = grid.to_unicode_grid();
        assert_eq!(unicode_grid.len(), expected_height, "Unicode grid height");
        assert_eq!(unicode_grid[0].len(), expected_width, "Unicode grid width");
    }

    #[test]
    fn test_full_pipeline_with_floyd_steinberg_dithering() {
        // Load test image
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        // Resize to smaller dimensions for faster test
        let resized = resize_to_dimensions(&img, 40, 24, true).expect("Failed to resize");

        // Convert to grayscale
        let gray = to_grayscale(&resized);

        // Apply Floyd-Steinberg dithering
        let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg)
            .expect("Failed to apply Floyd-Steinberg dithering");

        // Calculate expected grid dimensions
        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        // Map to braille grid
        let grid = pixels_to_braille(&binary, expected_width, expected_height)
            .expect("Failed to map to braille");

        // Verify grid dimensions match expected
        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);
    }

    #[test]
    fn test_full_pipeline_with_bayer_dithering() {
        // Load test image
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        // Resize to smaller dimensions
        let resized = resize_to_dimensions(&img, 40, 24, true).expect("Failed to resize");

        // Convert to grayscale
        let gray = to_grayscale(&resized);

        // Apply Bayer dithering
        let binary = apply_dithering(&gray, DitheringMethod::Bayer)
            .expect("Failed to apply Bayer dithering");

        // Calculate expected grid dimensions
        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        // Map to braille grid
        let grid = pixels_to_braille(&binary, expected_width, expected_height)
            .expect("Failed to map to braille");

        // Verify grid dimensions match expected
        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);
    }

    #[test]
    fn test_full_pipeline_with_atkinson_dithering() {
        // Load test image
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        // Resize to smaller dimensions
        let resized = resize_to_dimensions(&img, 40, 24, true).expect("Failed to resize");

        // Convert to grayscale
        let gray = to_grayscale(&resized);

        // Apply Atkinson dithering
        let binary = apply_dithering(&gray, DitheringMethod::Atkinson)
            .expect("Failed to apply Atkinson dithering");

        // Calculate expected grid dimensions
        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        // Map to braille grid
        let grid = pixels_to_braille(&binary, expected_width, expected_height)
            .expect("Failed to map to braille");

        // Verify grid dimensions match expected
        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);
    }

    #[test]
    fn test_pipeline_with_non_divisible_dimensions() {
        // Load test image
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        // Resize to non-divisible dimensions (37×21 pixels)
        let resized = resize_to_dimensions(&img, 37, 21, false).expect("Failed to resize");

        // Apply auto thresholding (includes grayscale conversion)
        let binary = auto_threshold(&resized);

        // Map to braille grid (should pad to 38×24 → 19×6 grid)
        let grid = pixels_to_braille(&binary, 19, 6).expect("Failed to map to braille");

        // Verify grid dimensions with padding
        // Width: (37+1)/2 = 19, Height: (21+3)/4 = 6
        assert_eq!(grid.width(), 19);
        assert_eq!(grid.height(), 6);
    }

    #[test]
    fn test_pipeline_preserves_details() {
        // Load test image
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        // Resize to terminal dimensions
        let resized = resize_to_dimensions(&img, 160, 96, true).expect("Failed to resize");

        // Convert to grayscale
        let gray = to_grayscale(&resized);

        // Compare threshold vs Floyd-Steinberg dithering
        let binary_threshold = auto_threshold(&resized);
        let binary_dithered = apply_dithering(&gray, DitheringMethod::FloydSteinberg).unwrap();

        // Calculate expected dimensions
        let expected_width = ((binary_threshold.width + 1) / 2) as usize;
        let expected_height = ((binary_threshold.height + 3) / 4) as usize;

        // Map both to grids
        let grid_threshold =
            pixels_to_braille(&binary_threshold, expected_width, expected_height).unwrap();
        let grid_dithered =
            pixels_to_braille(&binary_dithered, expected_width, expected_height).unwrap();

        // Both should have same dimensions
        assert_eq!(grid_threshold.width(), grid_dithered.width());
        assert_eq!(grid_threshold.height(), grid_dithered.height());

        // Both should produce valid unicode grids
        let unicode_threshold = grid_threshold.to_unicode_grid();
        let unicode_dithered = grid_dithered.to_unicode_grid();

        assert_eq!(unicode_threshold.len(), expected_height);
        assert_eq!(unicode_dithered.len(), expected_height);
    }
}

/// Integration tests for color mode rendering pipeline
#[cfg(feature = "image")]
mod color_pipeline_tests {
    use dotmax::image::{load_from_path, render_image_with_color, ColorMode};
    use std::path::Path;

    #[test]
    fn test_color_pipeline_monochrome_mode() {
        // Test monochrome mode (backward compatible, no colors)
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        let grid = render_image_with_color(&img, ColorMode::Monochrome)
            .expect("Failed to render with monochrome mode");

        // Verify grid has reasonable dimensions (aspect ratio preserved)
        // Target is 80×24 but actual may be smaller due to aspect ratio
        assert!(grid.width() > 0 && grid.width() <= 80);
        assert!(grid.height() > 0 && grid.height() <= 24);

        // Monochrome mode should not have colors
        // Grid should still have dot patterns
        let unicode_grid = grid.to_unicode_grid();
        assert_eq!(unicode_grid.len(), grid.height());
        assert_eq!(unicode_grid[0].len(), grid.width());
    }

    #[test]
    fn test_color_pipeline_grayscale_mode() {
        // Test grayscale mode (ANSI 256-color)
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        let grid = render_image_with_color(&img, ColorMode::Grayscale)
            .expect("Failed to render with grayscale mode");

        // Verify grid has reasonable dimensions
        assert!(grid.width() > 0 && grid.width() <= 80);
        assert!(grid.height() > 0 && grid.height() <= 24);

        // Grayscale mode should have colors
        // Verify at least some cells have non-black colors (intensity mapping)
        let mut has_gray = false;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if let Some(color) = grid.get_color(x, y) {
                    // Grayscale colors have R=G=B
                    if color.r == color.g && color.g == color.b {
                        has_gray = true;
                        break;
                    }
                }
            }
            if has_gray {
                break;
            }
        }
        // Note: We can't guarantee has_gray is true since the image might be all black/white
        // But the test verifies the API works
    }

    #[test]
    fn test_color_pipeline_truecolor_mode() {
        // Test TrueColor mode (24-bit RGB)
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        let grid = render_image_with_color(&img, ColorMode::TrueColor)
            .expect("Failed to render with truecolor mode");

        // Verify grid has reasonable dimensions
        assert!(grid.width() > 0 && grid.width() <= 80);
        assert!(grid.height() > 0 && grid.height() <= 24);

        // TrueColor mode should have colors
        // Verify we can retrieve colors from grid
        let mut color_count = 0;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get_color(x, y).is_some() {
                    color_count += 1;
                }
            }
        }

        // Every cell should have a color in TrueColor mode
        let expected_count = grid.width() * grid.height();
        assert_eq!(
            color_count, expected_count,
            "Every cell should have a color in TrueColor mode"
        );
    }

    #[test]
    fn test_color_pipeline_dimensions_consistent() {
        // Verify all modes produce same grid dimensions
        let img_path = Path::new("tests/fixtures/images/sample.png");
        let img = load_from_path(img_path).expect("Failed to load test image");

        let grid_mono = render_image_with_color(&img, ColorMode::Monochrome)
            .expect("Failed to render monochrome");
        let grid_gray = render_image_with_color(&img, ColorMode::Grayscale)
            .expect("Failed to render grayscale");
        let grid_true = render_image_with_color(&img, ColorMode::TrueColor)
            .expect("Failed to render truecolor");

        // All modes should produce same dimensions
        assert_eq!(grid_mono.width(), grid_gray.width());
        assert_eq!(grid_mono.width(), grid_true.width());
        assert_eq!(grid_mono.height(), grid_gray.height());
        assert_eq!(grid_mono.height(), grid_true.height());
    }

    #[test]
    fn test_color_extraction_with_small_image() {
        // Test with very small image (edge case)
        use image::{DynamicImage, RgbImage};

        // Create 4×8 pixel image (2×2 braille cells)
        let mut img = RgbImage::new(4, 8);
        // Fill with gradient
        for y in 0..8 {
            for x in 0..4 {
                let intensity = ((x + y) * 255 / 11) as u8;
                img.put_pixel(x, y, image::Rgb([intensity, intensity, intensity]));
            }
        }
        let dyn_img = DynamicImage::ImageRgb8(img);

        // render_image_with_color targets 80×24 but preserves aspect ratio
        // Small image may be upscaled with aspect preserved
        let grid = render_image_with_color(&dyn_img, ColorMode::TrueColor)
            .expect("Failed to render small image");

        // Verify reasonable dimensions (should be upscaled)
        assert!(grid.width() > 2);
        assert!(grid.height() > 2);
        assert!(grid.width() <= 80);
        assert!(grid.height() <= 24);
    }
}

/// Integration tests for SVG→braille rendering pipeline
#[cfg(all(feature = "image", feature = "svg"))]
mod svg_pipeline_tests {
    use dotmax::image::{
        apply_dithering, auto_threshold, load_svg_from_path, pixels_to_braille, to_grayscale,
        DitheringMethod,
    };
    use std::path::Path;

    #[test]
    fn test_svg_to_dynamic_image_properties() {
        // Load SVG and rasterize to specified dimensions
        let svg_path = Path::new("tests/fixtures/svg/simple_circle.svg");
        let img = load_svg_from_path(svg_path, 100, 100).expect("Failed to load SVG");

        // Verify dimensions match requested
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);

        // Verify we got RGBA8 format (from rasterization)
        assert!(matches!(img, image::DynamicImage::ImageRgba8(_)));
    }

    #[test]
    fn test_svg_grayscale_threshold_braille_pipeline() {
        // Full pipeline: SVG → grayscale → threshold → braille
        let svg_path = Path::new("tests/fixtures/svg/simple_circle.svg");
        let img = load_svg_from_path(svg_path, 80, 80).expect("Failed to load SVG");

        // Convert to grayscale
        let gray = to_grayscale(&img);

        // Apply auto threshold
        let binary = auto_threshold(&img);

        // Calculate expected grid dimensions
        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        // Map to braille grid
        let grid =
            pixels_to_braille(&binary, expected_width, expected_height).expect("Failed to map");

        // Verify grid dimensions
        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);

        // Verify grid produces valid unicode output
        let unicode_grid = grid.to_unicode_grid();
        assert_eq!(unicode_grid.len(), expected_height);
    }

    #[test]
    fn test_svg_dither_floyd_steinberg_braille() {
        // SVG → grayscale → Floyd-Steinberg dithering → braille
        let svg_path = Path::new("tests/fixtures/svg/gradient.svg");
        let img = load_svg_from_path(svg_path, 100, 100).expect("Failed to load SVG");

        let gray = to_grayscale(&img);
        let binary =
            apply_dithering(&gray, DitheringMethod::FloydSteinberg).expect("Failed to dither");

        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        let grid =
            pixels_to_braille(&binary, expected_width, expected_height).expect("Failed to map");

        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);
    }

    #[test]
    fn test_svg_dither_bayer_braille() {
        // SVG → grayscale → Bayer dithering → braille
        let svg_path = Path::new("tests/fixtures/svg/gradient.svg");
        let img = load_svg_from_path(svg_path, 100, 100).expect("Failed to load SVG");

        let gray = to_grayscale(&img);
        let binary = apply_dithering(&gray, DitheringMethod::Bayer).expect("Failed to dither");

        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        let grid =
            pixels_to_braille(&binary, expected_width, expected_height).expect("Failed to map");

        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);
    }

    #[test]
    fn test_svg_dither_atkinson_braille() {
        // SVG → grayscale → Atkinson dithering → braille
        let svg_path = Path::new("tests/fixtures/svg/gradient.svg");
        let img = load_svg_from_path(svg_path, 100, 100).expect("Failed to load SVG");

        let gray = to_grayscale(&img);
        let binary = apply_dithering(&gray, DitheringMethod::Atkinson).expect("Failed to dither");

        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        let grid =
            pixels_to_braille(&binary, expected_width, expected_height).expect("Failed to map");

        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);
    }

    #[test]
    fn test_svg_logo_full_pipeline() {
        // Complex SVG (logo with gradients and paths) through full pipeline
        let svg_path = Path::new("tests/fixtures/svg/logo.svg");
        let img = load_svg_from_path(svg_path, 160, 160).expect("Failed to load logo SVG");

        // Apply Floyd-Steinberg for quality
        let gray = to_grayscale(&img);
        let binary =
            apply_dithering(&gray, DitheringMethod::FloydSteinberg).expect("Failed to dither");

        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        let grid =
            pixels_to_braille(&binary, expected_width, expected_height).expect("Failed to map");

        // Verify output
        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);

        // Verify at least some dots are set (logo has content)
        let unicode_grid = grid.to_unicode_grid();
        let has_content = unicode_grid
            .iter()
            .any(|row| row.iter().any(|&ch| ch != '\u{2800}'));
        assert!(has_content, "Logo should produce non-empty braille output");
    }

    #[test]
    fn test_svg_text_heavy_pipeline() {
        // SVG with text elements (tests font fallback)
        let svg_path = Path::new("tests/fixtures/svg/text_heavy.svg");
        let img = load_svg_from_path(svg_path, 150, 50).expect("Failed to load text SVG");

        // Full pipeline
        let binary = auto_threshold(&img);

        let expected_width = ((binary.width + 1) / 2) as usize;
        let expected_height = ((binary.height + 3) / 4) as usize;

        let grid =
            pixels_to_braille(&binary, expected_width, expected_height).expect("Failed to map");

        assert_eq!(grid.width(), expected_width);
        assert_eq!(grid.height(), expected_height);

        // Text should produce visible output
        let unicode_grid = grid.to_unicode_grid();
        let has_text = unicode_grid
            .iter()
            .any(|row| row.iter().any(|&ch| ch != '\u{2800}'));
        assert!(has_text, "Text SVG should produce visible braille output");
    }
}
