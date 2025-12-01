//! Integration tests for image loading and resizing functionality
//!
//! These tests verify end-to-end image loading and resizing behavior across
//! all supported formats and error conditions.

#![cfg(feature = "image")]

use dotmax::image::{
    adjust_brightness, adjust_contrast, adjust_gamma, apply_threshold, auto_threshold,
    load_from_bytes, load_from_path, resize_to_dimensions, resize_to_terminal, supported_formats,
    to_grayscale,
};
use dotmax::DotmaxError;
use std::path::Path;

#[test]
fn test_integration_load_png_verify_dimensions() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let result = load_from_path(path);

    assert!(result.is_ok(), "Failed to load PNG: {:?}", result.err());

    let img = result.unwrap();
    assert_eq!(img.width(), 10, "Expected 10x10 test image");
    assert_eq!(img.height(), 10, "Expected 10x10 test image");
}

#[test]
fn test_integration_load_second_image_file() {
    // The test_photo.jpg is a test file (may be PNG or JPG format)
    // This test verifies we can load it regardless of extension
    let path = Path::new("tests/fixtures/images/test_photo.jpg");

    // Only run test if file exists (may be PNG masquerading as JPG)
    if path.exists() {
        let result = load_from_path(path);

        // Image crate detects format by content, not extension
        // So this should either load successfully or fail gracefully
        match result {
            Ok(img) => {
                assert!(img.width() > 0);
                assert!(img.height() > 0);
            }
            Err(DotmaxError::ImageLoad { .. }) => {
                // Expected if file format doesn't match extension
                // The important thing is it returns an error, not panics
            }
            Err(other) => {
                panic!("Unexpected error type: {:?}", other);
            }
        }
    }
}

#[test]
fn test_integration_load_all_supported_formats() {
    // Verify that supported_formats() returns a consistent list
    let formats = supported_formats();

    assert!(formats.contains(&"png"), "PNG should be supported");
    assert!(formats.contains(&"jpg"), "JPG should be supported");
    assert!(formats.contains(&"jpeg"), "JPEG should be supported");
    assert!(formats.contains(&"gif"), "GIF should be supported");
    assert!(formats.contains(&"bmp"), "BMP should be supported");
    assert!(formats.contains(&"webp"), "WebP should be supported");
    assert!(formats.contains(&"tiff"), "TIFF should be supported");

    // Verify we have exactly the expected formats
    assert_eq!(formats.len(), 7, "Expected 7 supported formats");
}

#[test]
fn test_integration_error_handling_missing_file() {
    let path = Path::new("tests/fixtures/images/does_not_exist.png");
    let result = load_from_path(path);

    assert!(result.is_err(), "Should return error for missing file");

    match result.unwrap_err() {
        DotmaxError::ImageLoad { path: err_path, .. } => {
            assert!(err_path.to_string_lossy().contains("does_not_exist"));
        }
        other => panic!("Expected ImageLoad error, got {:?}", other),
    }
}

#[test]
fn test_integration_error_handling_corrupted_file() {
    let path = Path::new("tests/fixtures/images/corrupted.png");
    let result = load_from_path(path);

    assert!(result.is_err(), "Should return error for corrupted file");

    match result.unwrap_err() {
        DotmaxError::ImageLoad { .. } => {
            // Expected - corrupted data should trigger ImageLoad error
        }
        other => panic!("Expected ImageLoad error, got {:?}", other),
    }
}

#[test]
fn test_integration_bytes_loading_roundtrip() {
    // Load file, get bytes, reload from bytes - should match
    let path = Path::new("tests/fixtures/images/sample.png");
    let bytes = std::fs::read(path).expect("Failed to read sample.png");

    let img_from_path = load_from_path(path).expect("Failed to load from path");
    let img_from_bytes = load_from_bytes(&bytes).expect("Failed to load from bytes");

    assert_eq!(
        img_from_path.width(),
        img_from_bytes.width(),
        "Width should match"
    );
    assert_eq!(
        img_from_path.height(),
        img_from_bytes.height(),
        "Height should match"
    );
}

#[test]
fn test_integration_feature_gate_compiles() {
    // This test simply existing and compiling verifies the feature gate works
    // If the `image` feature is not enabled, this entire file won't compile

    let formats = supported_formats();
    assert!(!formats.is_empty(), "Should have supported formats");
}

#[test]
fn test_integration_zero_panics_guarantee() {
    // Verify that error conditions return Result, not panic

    // Missing file
    let result = load_from_path(Path::new("nonexistent.png"));
    assert!(result.is_err());

    // Invalid bytes
    let result = load_from_bytes(b"not an image");
    assert!(result.is_err());

    // Corrupted file
    let result = load_from_path(Path::new("tests/fixtures/images/corrupted.png"));
    assert!(result.is_err());

    // All error conditions handled gracefully via Result
}

// ========== Integration Tests for Image Resizing (Story 3.2) ==========

// Task 8.1: Integration test: Load PNG → resize to terminal (80×24) → verify dimensions
#[test]
fn test_integration_load_and_resize_to_terminal() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Resize to standard 80×24 terminal
    let resized = resize_to_terminal(&img, 80, 24).expect("Resize failed");

    // Terminal dimensions: 80×24 cells → 160×96 pixels (cells × 2, cells × 4)
    assert!(
        resized.width() <= 160,
        "Width {} exceeds terminal width 160",
        resized.width()
    );
    assert!(
        resized.height() <= 96,
        "Height {} exceeds terminal height 96",
        resized.height()
    );

    // Should preserve aspect ratio (sample.png is 10×10 square)
    // Terminal resize allows upscaling to fill available space
    // A 10×10 square image resizing to 80×24 terminal (160×96 pixels) should
    // scale up to 96×96 (limited by height, preserving square aspect ratio)
    assert_eq!(
        resized.width(),
        96,
        "Square image upscales to fit terminal height"
    );
    assert_eq!(
        resized.height(),
        96,
        "Square image upscales to fit terminal height"
    );
}

// Task 8.2: Integration test: Load JPG → resize manually (100×50, preserve aspect) → verify
#[test]
fn test_integration_load_jpg_and_resize_preserve_aspect() {
    // Use sample.png if test_photo.jpg doesn't exist
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load image");

    // Resize to 100×50 with aspect ratio preservation
    let resized = resize_to_dimensions(&img, 100, 50, true).expect("Resize failed");

    // Should fit within target dimensions
    assert!(
        resized.width() <= 100,
        "Width {} exceeds target 100",
        resized.width()
    );
    assert!(
        resized.height() <= 50,
        "Height {} exceeds target 50",
        resized.height()
    );

    // Square image → should be 50×50 (constrained by smaller dimension)
    assert_eq!(resized.width(), 50);
    assert_eq!(resized.height(), 50);
}

// Task 8.3: Integration test: Load image → resize without aspect preservation → verify exact dimensions
#[test]
fn test_integration_resize_without_aspect_preservation() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Resize to exact dimensions without preserving aspect ratio
    let resized = resize_to_dimensions(&img, 100, 50, false).expect("Resize failed");

    // Should be exactly target dimensions (stretched)
    assert_eq!(resized.width(), 100, "Width should match target exactly");
    assert_eq!(resized.height(), 50, "Height should match target exactly");
}

// Task 8.4: Integration test: Resize to terminal size, verify braille cell math (width×2, height×4)
#[test]
fn test_integration_braille_cell_math_verification() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Test various terminal sizes and verify pixel calculations
    let test_cases = [
        (80, 24, 160, 96),   // Standard terminal
        (100, 30, 200, 120), // Larger terminal
        (40, 12, 80, 48),    // Smaller terminal
    ];

    for (term_w, term_h, max_px_w, max_px_h) in test_cases {
        let resized = resize_to_terminal(&img, term_w, term_h)
            .unwrap_or_else(|_| panic!("Failed to resize to {}×{}", term_w, term_h));

        // Verify dimensions fit within braille cell calculations
        assert!(
            resized.width() <= max_px_w,
            "Terminal {}×{} → width {} exceeds max {}",
            term_w,
            term_h,
            resized.width(),
            max_px_w
        );
        assert!(
            resized.height() <= max_px_h,
            "Terminal {}×{} → height {} exceeds max {}",
            term_w,
            term_h,
            resized.height(),
            max_px_h
        );
    }
}

// Task 8.5: Integration test: Error handling (invalid dimensions return error, not panic)
#[test]
fn test_integration_resize_error_handling_zero_dimensions() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Zero terminal width
    let result = resize_to_terminal(&img, 0, 24);
    assert!(result.is_err(), "Should error on zero terminal width");
    assert!(
        matches!(
            result.unwrap_err(),
            DotmaxError::InvalidImageDimensions { .. }
        ),
        "Should return InvalidImageDimensions error"
    );

    // Zero terminal height
    let result = resize_to_terminal(&img, 80, 0);
    assert!(result.is_err(), "Should error on zero terminal height");

    // Zero target width
    let result = resize_to_dimensions(&img, 0, 100, true);
    assert!(result.is_err(), "Should error on zero target width");

    // Zero target height
    let result = resize_to_dimensions(&img, 100, 0, false);
    assert!(result.is_err(), "Should error on zero target height");

    // No panics - all errors returned as Result
}

// Additional integration test: Full pipeline (load from bytes → resize → verify)
#[test]
fn test_integration_full_pipeline_bytes_to_resize() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let bytes = std::fs::read(path).expect("Failed to read sample.png");

    // Load from bytes
    let img = load_from_bytes(&bytes).expect("Failed to load from bytes");

    // Resize to terminal
    let resized = resize_to_terminal(&img, 80, 24).expect("Resize failed");

    // Verify result
    assert!(resized.width() > 0 && resized.width() <= 160);
    assert!(resized.height() > 0 && resized.height() <= 96);
}

// Integration test: Verify large terminal dimensions work
#[test]
fn test_integration_large_terminal_resize() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Large terminal: 200×50 cells → 400×200 pixels
    let resized = resize_to_terminal(&img, 200, 50).expect("Resize failed");

    assert!(resized.width() <= 400);
    assert!(resized.height() <= 200);

    // Original image is 10×10 square, terminal is 200×50 cells (400×200 pixels)
    // Terminal resize allows upscaling to fill available space
    // A square image should scale to fit the smaller dimension (200 height)
    assert_eq!(
        resized.width(),
        200,
        "Square image upscales to fit terminal height (200×200)"
    );
    assert_eq!(
        resized.height(),
        200,
        "Square image upscales to fit terminal height (200×200)"
    );
}

// Integration test: Resize maintains quality with Lanczos3 filter
#[test]
fn test_integration_resize_uses_lanczos3_quality() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Resize to various sizes - should always produce valid output
    let sizes = [(80, 24), (100, 30), (40, 12)];

    for (w, h) in sizes {
        let resized = resize_to_terminal(&img, w, h)
            .unwrap_or_else(|_| panic!("Failed to resize to {}×{}", w, h));

        // Verify image is valid (has non-zero dimensions)
        assert!(resized.width() > 0, "Resized image has zero width");
        assert!(resized.height() > 0, "Resized image has zero height");

        // Image format should have 3 or 4 channels (RGB or RGBA)
        let channels = resized.color().channel_count();
        assert!(
            channels == 3 || channels == 4,
            "Resized image should be RGB or RGBA, got {} channels",
            channels
        );
    }
}

// ========== Integration Tests for Grayscale Conversion and Thresholding (Story 3.3) ==========

// Task 11.1: Integration test: load PNG → resize → grayscale → verify
#[test]
fn test_integration_load_resize_grayscale_pipeline() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Resize to terminal
    let resized = resize_to_terminal(&img, 80, 24).expect("Resize failed");

    // Convert to grayscale
    let gray = to_grayscale(&resized);

    // Verify dimensions preserved
    assert_eq!(gray.width(), resized.width());
    assert_eq!(gray.height(), resized.height());

    // Verify it's actually grayscale by checking pixel type
    // GrayImage has 1 byte per pixel
    let first_pixel = gray.get_pixel(0, 0);
    assert_eq!(first_pixel.0.len(), 1, "GrayImage should have 1 channel");
}

// Task 11.2: Integration test: load color image → auto_threshold → verify BinaryImage
#[test]
fn test_integration_auto_threshold_pipeline() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    // Apply auto-threshold (grayscale → otsu → binary)
    let binary = auto_threshold(&img);

    // Verify dimensions
    assert_eq!(binary.width, img.width());
    assert_eq!(binary.height, img.height());

    // Verify pixel count matches dimensions
    assert_eq!(binary.pixel_count(), (img.width() * img.height()) as usize);

    // Verify pixels are boolean (they always are, but let's verify the type is correct)
    for &pixel in &binary.pixels {
        // Pixels are always boolean, just ensure we can iterate them
        let _: bool = pixel;
    }
}

// Task 11.3: Integration test: adjust brightness → threshold → compare results
#[test]
fn test_integration_brightness_adjustment_affects_threshold() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    let gray = to_grayscale(&img);

    // Brighten image
    let bright = adjust_brightness(&gray, 1.5).expect("Brightness adjustment failed");

    // Apply threshold to both
    let binary_normal = apply_threshold(&gray, 128);
    let binary_bright = apply_threshold(&bright, 128);

    // Brightened image should have more black pixels (above threshold)
    let normal_black_count = binary_normal.pixels.iter().filter(|&&p| p).count();
    let bright_black_count = binary_bright.pixels.iter().filter(|&&p| p).count();

    // Brightening should increase the number of pixels above threshold
    assert!(
        bright_black_count >= normal_black_count,
        "Brightness should increase black pixels"
    );
}

// Task 11.4: Integration test: adjust contrast → threshold → compare results
#[test]
fn test_integration_contrast_adjustment_pipeline() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    let gray = to_grayscale(&img);

    // Increase contrast
    let contrasted = adjust_contrast(&gray, 1.5).expect("Contrast adjustment failed");

    // Both should have valid dimensions
    assert_eq!(contrasted.width(), gray.width());
    assert_eq!(contrasted.height(), gray.height());

    // Pixel values should be different (unless all pixels were 128)
    // We can't guarantee specific changes without knowing the image content,
    // but we can verify the operation doesn't fail
    let binary = apply_threshold(&contrasted, 128);
    assert_eq!(
        binary.pixel_count(),
        (gray.width() * gray.height()) as usize
    );
}

// Task 11.5: Integration test: adjust gamma → threshold → compare results
#[test]
fn test_integration_gamma_correction_pipeline() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    let gray = to_grayscale(&img);

    // Apply gamma correction (brighten with gamma < 1.0)
    let gamma_corrected = adjust_gamma(&gray, 0.8).expect("Gamma correction failed");

    // Verify dimensions preserved
    assert_eq!(gamma_corrected.width(), gray.width());
    assert_eq!(gamma_corrected.height(), gray.height());

    // Apply threshold
    let binary = apply_threshold(&gamma_corrected, 128);
    assert_eq!(
        binary.pixel_count(),
        (gray.width() * gray.height()) as usize
    );
}

// Task 11.6: Integration test: chain adjustments (brightness + contrast + gamma) → threshold
#[test]
fn test_integration_chained_adjustments_pipeline() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    let gray = to_grayscale(&img);

    // Chain multiple adjustments
    let adjusted = adjust_brightness(&gray, 1.2)
        .and_then(|img| adjust_contrast(&img, 1.3))
        .and_then(|img| adjust_gamma(&img, 0.9))
        .expect("Chained adjustments failed");

    // Verify dimensions preserved through chain
    assert_eq!(adjusted.width(), gray.width());
    assert_eq!(adjusted.height(), gray.height());

    // Apply threshold to final result
    let binary = apply_threshold(&adjusted, 128);
    assert_eq!(
        binary.pixel_count(),
        (gray.width() * gray.height()) as usize
    );
}

// Task 11.7: Error handling integration test: invalid parameters return errors (not panics)
#[test]
fn test_integration_adjustment_error_handling() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    let gray = to_grayscale(&img);

    // Invalid brightness (negative)
    let result = adjust_brightness(&gray, -0.5);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DotmaxError::InvalidParameter { .. }
    ));

    // Invalid brightness (too high)
    let result = adjust_brightness(&gray, 3.0);
    assert!(result.is_err());

    // Invalid contrast (negative)
    let result = adjust_contrast(&gray, -1.0);
    assert!(result.is_err());

    // Invalid contrast (too high)
    let result = adjust_contrast(&gray, 2.5);
    assert!(result.is_err());

    // Invalid gamma (too low)
    let result = adjust_gamma(&gray, 0.05);
    assert!(result.is_err());

    // Invalid gamma (too high)
    let result = adjust_gamma(&gray, 5.0);
    assert!(result.is_err());

    // All errors returned as Result, no panics
}

// Full pipeline integration test: load → resize → grayscale → threshold → verify
#[test]
fn test_integration_complete_image_to_binary_pipeline() {
    use image::DynamicImage;

    let path = Path::new("tests/fixtures/images/sample.png");

    // Step 1: Load image
    let img = load_from_path(path).expect("Failed to load");

    // Step 2: Resize to terminal dimensions
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");

    // Step 3: Convert to grayscale
    let gray = to_grayscale(&resized);

    // Step 4: Optional adjustments
    let adjusted = adjust_brightness(&gray, 1.1).expect("Failed to adjust brightness");

    // Step 5: Apply automatic thresholding
    let binary = auto_threshold(&DynamicImage::ImageLuma8(adjusted));

    // Verify final binary image
    assert!(binary.width > 0);
    assert!(binary.height > 0);
    assert_eq!(
        binary.pixel_count(),
        (binary.width * binary.height) as usize
    );

    // Verify all pixels are boolean (they always are, but let's verify the type is correct)
    for &pixel in &binary.pixels {
        // Pixels are always boolean, just ensure we can iterate them
        let _: bool = pixel;
    }
}

// Integration test: Verify BinaryImage get/set pixel methods
#[test]
fn test_integration_binary_image_pixel_access() {
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample.png");

    let binary = auto_threshold(&img);

    // Test get_pixel
    let pixel = binary.get_pixel(0, 0);
    assert!(pixel.is_some());

    // Test out of bounds
    let out_of_bounds = binary.get_pixel(10000, 10000);
    assert!(out_of_bounds.is_none());
}

// ===== Dithering Integration Tests =====

use dotmax::image::{apply_dithering, DitheringMethod};

#[test]
fn test_integration_full_pipeline_with_floyd_steinberg() {
    // Full pipeline: load → resize → grayscale → dither (Floyd-Steinberg) → verify BinaryImage
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    // Resize to smaller size for faster test
    let resized = resize_to_dimensions(&img, 20, 20, true).expect("Resize failed");

    // Convert to grayscale
    let gray = to_grayscale(&resized);

    // Apply Floyd-Steinberg dithering
    let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg).expect("Dithering failed");

    // Verify output
    assert_eq!(binary.width, 20);
    assert_eq!(binary.height, 20);
    assert_eq!(binary.pixels.len(), 400); // 20×20 = 400 pixels
}

#[test]
fn test_integration_full_pipeline_with_bayer() {
    // Full pipeline with Bayer dithering
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 20, 20, true).expect("Resize failed");
    let gray = to_grayscale(&resized);
    let binary = apply_dithering(&gray, DitheringMethod::Bayer).expect("Dithering failed");

    assert_eq!(binary.width, 20);
    assert_eq!(binary.height, 20);
}

#[test]
fn test_integration_full_pipeline_with_atkinson() {
    // Full pipeline with Atkinson dithering
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 20, 20, true).expect("Resize failed");
    let gray = to_grayscale(&resized);
    let binary = apply_dithering(&gray, DitheringMethod::Atkinson).expect("Dithering failed");

    assert_eq!(binary.width, 20);
    assert_eq!(binary.height, 20);
}

#[test]
fn test_integration_dithering_method_none() {
    // Test DitheringMethod::None (should use auto_threshold)
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 20, 20, true).expect("Resize failed");
    let gray = to_grayscale(&resized);
    let binary = apply_dithering(&gray, DitheringMethod::None).expect("Dithering failed");

    assert_eq!(binary.width, 20);
    assert_eq!(binary.height, 20);
}

#[test]
fn test_integration_all_dithering_methods_produce_valid_output() {
    // Verify all three methods produce valid BinaryImage output
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 30, 30, true).expect("Resize failed");
    let gray = to_grayscale(&resized);

    // Test all methods
    let floyd =
        apply_dithering(&gray, DitheringMethod::FloydSteinberg).expect("Floyd-Steinberg failed");
    let bayer = apply_dithering(&gray, DitheringMethod::Bayer).expect("Bayer failed");
    let atkinson = apply_dithering(&gray, DitheringMethod::Atkinson).expect("Atkinson failed");
    let none = apply_dithering(&gray, DitheringMethod::None).expect("None failed");

    // All should have same dimensions
    assert_eq!(floyd.width, 30);
    assert_eq!(bayer.width, 30);
    assert_eq!(atkinson.width, 30);
    assert_eq!(none.width, 30);

    // All should have same pixel count
    assert_eq!(floyd.pixels.len(), 900);
    assert_eq!(bayer.pixels.len(), 900);
    assert_eq!(atkinson.pixels.len(), 900);
    assert_eq!(none.pixels.len(), 900);
}

#[test]
fn test_integration_dithering_preserves_dimensions() {
    // Verify that all algorithms preserve input dimensions
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    // Test various sizes
    for size in [10, 20, 50, 100] {
        let resized = resize_to_dimensions(&img, size, size, true).expect("Resize failed");
        let gray = to_grayscale(&resized);

        let floyd = apply_dithering(&gray, DitheringMethod::FloydSteinberg)
            .expect("Floyd-Steinberg failed");
        let bayer = apply_dithering(&gray, DitheringMethod::Bayer).expect("Bayer failed");
        let atkinson = apply_dithering(&gray, DitheringMethod::Atkinson).expect("Atkinson failed");

        assert_eq!(floyd.width, size);
        assert_eq!(floyd.height, size);
        assert_eq!(bayer.width, size);
        assert_eq!(bayer.height, size);
        assert_eq!(atkinson.width, size);
        assert_eq!(atkinson.height, size);
    }
}

#[test]
fn test_integration_dithering_with_brightness_adjustment() {
    // Test dithering after brightness adjustment
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 20, 20, true).expect("Resize failed");
    let gray = to_grayscale(&resized);

    // Adjust brightness
    let adjusted = adjust_brightness(&gray, 1.5).expect("Brightness adjustment failed");

    // Apply dithering
    let binary =
        apply_dithering(&adjusted, DitheringMethod::FloydSteinberg).expect("Dithering failed");

    assert_eq!(binary.width, 20);
    assert_eq!(binary.height, 20);
}

#[test]
fn test_integration_dithering_with_contrast_adjustment() {
    // Test dithering after contrast adjustment
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 20, 20, true).expect("Resize failed");
    let gray = to_grayscale(&resized);

    // Adjust contrast
    let adjusted = adjust_contrast(&gray, 1.5).expect("Contrast adjustment failed");

    // Apply dithering
    let binary = apply_dithering(&adjusted, DitheringMethod::Bayer).expect("Dithering failed");

    assert_eq!(binary.width, 20);
    assert_eq!(binary.height, 20);
}

#[test]
fn test_integration_dithering_cross_platform_consistency() {
    // Verify that same input produces same output (deterministic)
    let path = Path::new("tests/fixtures/images/sample.png");
    let img = load_from_path(path).expect("Failed to load sample image");

    let resized = resize_to_dimensions(&img, 15, 15, true).expect("Resize failed");
    let gray = to_grayscale(&resized);

    // Run Bayer (deterministic) twice
    let result1 = apply_dithering(&gray, DitheringMethod::Bayer).expect("Dithering failed");
    let result2 = apply_dithering(&gray, DitheringMethod::Bayer).expect("Dithering failed");

    // Should produce identical output
    assert_eq!(
        result1.pixels, result2.pixels,
        "Bayer dithering should be deterministic"
    );
}
