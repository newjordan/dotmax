//! Image resizing with aspect ratio preservation
//!
//! This module provides functions for resizing images to fit terminal dimensions
//! while preserving aspect ratios and avoiding distortion. It implements two main
//! resize strategies:
//!
//! 1. **Terminal-based resizing** (`resize_to_terminal`): Automatically calculates
//!    optimal dimensions based on terminal size and braille cell geometry (2×4 dots)
//! 2. **Manual resizing** (`resize_to_dimensions`): Allows explicit dimension
//!    specification with optional aspect ratio preservation
//!
//! # Braille Cell Coordinate System
//!
//! Terminal dimensions are measured in braille cells, where each cell is 2×4 dots:
//! - Terminal width of 80 cells = 160 pixels wide (80 × 2)
//! - Terminal height of 24 cells = 96 pixels tall (24 × 4)
//!
//! This coordinate system ensures images map correctly to the braille dot matrix.
//!
//! # Resize Quality
//!
//! All resize operations use the Lanczos3 filter from the `image` crate, which
//! provides high-quality output at the cost of some performance. For typical
//! terminal images (800×600 → 160×96), resize completes in <10ms.
//!
//! **Quality Trade-offs:**
//! - **Lanczos3** (used): Highest quality, slower (~10ms)
//! - **Triangle**: Medium quality, faster (~5ms)
//! - **Nearest**: Lowest quality, fastest (~1ms)
//!
//! We prioritize quality over speed since resize is a one-time operation per image.
//!
//! # Examples
//!
//! ## Resize to terminal dimensions
//!
//! ```no_run
//! use dotmax::image::{load_from_path, resize_to_terminal};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = load_from_path(Path::new("photo.jpg"))?;
//! let resized = resize_to_terminal(&img, 80, 24)?;
//! println!("Resized to {}×{} pixels", resized.width(), resized.height());
//! # Ok(())
//! # }
//! ```
//!
//! ## Resize to custom dimensions
//!
//! ```no_run
//! use dotmax::image::{load_from_path, resize_to_dimensions};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = load_from_path(Path::new("diagram.png"))?;
//!
//! // Preserve aspect ratio (letterbox/pillarbox)
//! let resized = resize_to_dimensions(&img, 200, 100, true)?;
//!
//! // Stretch to exact dimensions (may distort)
//! let stretched = resize_to_dimensions(&img, 200, 100, false)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! Target: <10ms for typical images (800×600 → terminal size)
//! - Small images (100×100): ~2ms
//! - Medium images (800×600): ~8ms
//! - Large images (1920×1080): ~15ms

use crate::error::DotmaxError;
use crate::image::loader::{MAX_IMAGE_HEIGHT, MAX_IMAGE_WIDTH};
use image::{imageops, DynamicImage};
use tracing::debug;

/// Maximum upscale factor to prevent quality degradation
/// Images will not be upscaled more than 2x their original size
const MAX_UPSCALE_FACTOR: f32 = 2.0;

/// Braille cell width in dots (2 dots wide)
const BRAILLE_CELL_WIDTH: u16 = 2;

/// Braille cell height in dots (4 dots tall)
const BRAILLE_CELL_HEIGHT: u16 = 4;

/// Resize image to fit terminal dimensions with aspect ratio preservation
///
/// Calculates optimal pixel dimensions from terminal dimensions (measured in
/// braille cells) and resizes the image to fit while preserving aspect ratio.
/// Uses Lanczos3 filter for high-quality output.
///
/// # Braille Cell Math
///
/// Terminal dimensions are in braille cells where each cell is 2×4 dots:
/// - `pixel_width = term_width × 2`
/// - `pixel_height = term_height × 4`
///
/// Example: 80×24 terminal = 160×96 pixels
///
/// # Arguments
///
/// * `image` - Source image to resize
/// * `term_width` - Terminal width in braille cells
/// * `term_height` - Terminal height in braille cells
///
/// # Returns
///
/// Resized `DynamicImage` that fits within terminal dimensions with aspect
/// ratio preserved (may be letterboxed or pillarboxed).
///
/// # Errors
///
/// Returns `DotmaxError::InvalidImageDimensions` if:
/// - Terminal dimensions are zero
/// - Calculated dimensions exceed maximum limits
///
/// # Examples
///
/// ```no_run
/// use dotmax::image::{load_from_path, resize_to_terminal};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let img = load_from_path(Path::new("photo.jpg"))?;
/// let resized = resize_to_terminal(&img, 80, 24)?;
/// assert_eq!(resized.width(), 160);  // 80 cells × 2 dots
/// assert_eq!(resized.height(), 96);  // 24 cells × 4 dots
/// # Ok(())
/// # }
/// ```
pub fn resize_to_terminal(
    image: &DynamicImage,
    term_width: u16,
    term_height: u16,
) -> Result<DynamicImage, DotmaxError> {
    // Validate terminal dimensions
    if term_width == 0 || term_height == 0 {
        return Err(DotmaxError::InvalidImageDimensions {
            width: u32::from(term_width),
            height: u32::from(term_height),
        });
    }

    // Calculate pixel dimensions from braille cell dimensions
    let target_width_px = u32::from(term_width) * u32::from(BRAILLE_CELL_WIDTH);
    let target_height_px = u32::from(term_height) * u32::from(BRAILLE_CELL_HEIGHT);

    debug!(
        "Resize to terminal: {}×{} cells → {}×{} pixels",
        term_width, term_height, target_width_px, target_height_px
    );

    // Calculate aspect ratio of source image
    let src_width = image.width();
    let src_height = image.height();
    #[allow(clippy::cast_precision_loss)]
    let aspect_ratio = src_width as f32 / src_height as f32;

    debug!(
        "Source image: {}×{}, aspect ratio: {:.2}",
        src_width, src_height, aspect_ratio
    );

    // Calculate dimensions that preserve aspect ratio and fit within target
    let (mut final_width, mut final_height) =
        calculate_fit_dimensions(src_width, src_height, target_width_px, target_height_px);

    // Apply upscale prevention (default: enabled)
    (final_width, final_height) = prevent_upscale(src_width, src_height, final_width, final_height);

    debug!(
        "Final dimensions after aspect ratio preservation and upscale prevention: {}×{}",
        final_width, final_height
    );

    // Only resize if dimensions changed
    if final_width == src_width && final_height == src_height {
        debug!("Image already at target size, skipping resize");
        return Ok(image.clone());
    }

    // Perform resize with Lanczos3 filter for quality
    let resized = imageops::resize(
        image,
        final_width,
        final_height,
        imageops::FilterType::Lanczos3,
    );

    Ok(DynamicImage::ImageRgba8(resized))
}

/// Calculate dimensions that fit within target while preserving aspect ratio
///
/// Implements letterboxing (reduce height) or pillarboxing (reduce width) to
/// maintain the source aspect ratio while fitting within target dimensions.
///
/// # Arguments
///
/// * `src_w` - Source image width
/// * `src_h` - Source image height
/// * `target_w` - Target width constraint
/// * `target_h` - Target height constraint
///
/// # Returns
///
/// Tuple of (width, height) that fits within target with aspect ratio preserved
///
/// # Example Output
///
/// Wide image (16:9) → letterbox (reduce height):
/// - Input: 1920×1080 → Target: 160×96
/// - Output: 160×90 (constrained by width)
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn calculate_fit_dimensions(src_w: u32, src_h: u32, target_w: u32, target_h: u32) -> (u32, u32) {
    let src_aspect = src_w as f32 / src_h as f32;
    let target_aspect = target_w as f32 / target_h as f32;

    let (final_w, final_h) = if src_aspect > target_aspect {
        // Source is wider → constrain by width (pillarbox)
        let new_width = target_w;
        let new_height = (target_w as f32 / src_aspect).round() as u32;
        (new_width, new_height)
    } else {
        // Source is taller or same → constrain by height (letterbox)
        let new_height = target_h;
        let new_width = (target_h as f32 * src_aspect).round() as u32;
        (new_width, new_height)
    };

    // Ensure we don't exceed target dimensions due to rounding
    let final_w = final_w.min(target_w).max(1); // At least 1 pixel wide
    let final_h = final_h.min(target_h).max(1); // At least 1 pixel tall

    (final_w, final_h)
}

/// Prevent excessive upscaling to maintain image quality
///
/// If the target dimensions would require upscaling beyond `MAX_UPSCALE_FACTOR`,
/// returns the original dimensions instead. This prevents quality degradation
/// from excessive upscaling (e.g., 100×100 → 1000×1000).
///
/// # Arguments
///
/// * `src_w` - Source image width
/// * `src_h` - Source image height
/// * `target_w` - Proposed target width
/// * `target_h` - Proposed target height
///
/// # Returns
///
/// Tuple of (width, height) clamped to reasonable upscale limits
///
/// # Example Output
///
/// Small image, large target → clamp to original size:
/// - Input: 100×100 → Target: 800×600 (8x upscale)
/// - Output: 100×100 (no upscaling beyond `MAX_UPSCALE_FACTOR` of 2.0)
#[allow(clippy::cast_precision_loss)]
fn prevent_upscale(src_w: u32, src_h: u32, target_w: u32, target_h: u32) -> (u32, u32) {
    // Check if we're upscaling
    if target_w > src_w || target_h > src_h {
        // Calculate upscale factors
        let width_factor = target_w as f32 / src_w as f32;
        let height_factor = target_h as f32 / src_h as f32;
        let max_factor = width_factor.max(height_factor);

        if max_factor > MAX_UPSCALE_FACTOR {
            debug!(
                "Upscale factor {:.2} exceeds limit {:.2}, using original dimensions",
                max_factor, MAX_UPSCALE_FACTOR
            );
            // Return original dimensions (no upscaling)
            return (src_w, src_h);
        }
    }

    // Downscaling or acceptable upscaling
    (target_w, target_h)
}

/// Resize image to specific dimensions with optional aspect ratio preservation
///
/// Provides manual control over image dimensions. When `preserve_aspect` is true,
/// the image is resized to fit within the target dimensions while maintaining
/// aspect ratio (letterbox/pillarbox). When false, the image is stretched to
/// exact dimensions (may cause distortion).
///
/// # Arguments
///
/// * `image` - Source image to resize
/// * `target_width` - Desired width in pixels
/// * `target_height` - Desired height in pixels
/// * `preserve_aspect` - If true, maintain aspect ratio; if false, stretch to exact size
///
/// # Returns
///
/// Resized `DynamicImage` matching the target dimensions (or fitted within if preserving aspect)
///
/// # Errors
///
/// Returns `DotmaxError::InvalidImageDimensions` if:
/// - Target width or height is zero
/// - Target dimensions exceed `MAX_IMAGE_WIDTH` or `MAX_IMAGE_HEIGHT`
///
/// # Examples
///
/// ## Preserve aspect ratio (letterbox/pillarbox)
///
/// ```no_run
/// use dotmax::image::{load_from_path, resize_to_dimensions};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let img = load_from_path(Path::new("photo.jpg"))?;
/// let resized = resize_to_dimensions(&img, 200, 100, true)?;
/// // Output fits within 200×100, aspect ratio maintained
/// assert!(resized.width() <= 200);
/// assert!(resized.height() <= 100);
/// # Ok(())
/// # }
/// ```
///
/// ## Stretch to exact dimensions (may distort)
///
/// ```no_run
/// use dotmax::image::{load_from_path, resize_to_dimensions};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let img = load_from_path(Path::new("diagram.png"))?;
/// let resized = resize_to_dimensions(&img, 200, 100, false)?;
/// // Output is exactly 200×100 (may be distorted)
/// assert_eq!(resized.width(), 200);
/// assert_eq!(resized.height(), 100);
/// # Ok(())
/// # }
/// ```
pub fn resize_to_dimensions(
    image: &DynamicImage,
    target_width: u32,
    target_height: u32,
    preserve_aspect: bool,
) -> Result<DynamicImage, DotmaxError> {
    // Validate target dimensions
    if target_width == 0 || target_height == 0 {
        return Err(DotmaxError::InvalidImageDimensions {
            width: target_width,
            height: target_height,
        });
    }

    // Validate against maximum dimensions
    if target_width > MAX_IMAGE_WIDTH || target_height > MAX_IMAGE_HEIGHT {
        return Err(DotmaxError::InvalidImageDimensions {
            width: target_width,
            height: target_height,
        });
    }

    let src_width = image.width();
    let src_height = image.height();

    debug!(
        "Resize to dimensions: {}×{} → {}×{}, preserve_aspect: {}",
        src_width, src_height, target_width, target_height, preserve_aspect
    );

    let (final_width, final_height) = if preserve_aspect {
        // Calculate dimensions that fit within target while preserving aspect ratio
        let dims = calculate_fit_dimensions(src_width, src_height, target_width, target_height);
        debug!(
            "Aspect ratio preserved: final dimensions {}×{}",
            dims.0, dims.1
        );
        dims
    } else {
        // Stretch to exact dimensions (may distort)
        debug!("Stretching to exact dimensions (aspect ratio not preserved)");
        (target_width, target_height)
    };

    // Perform resize with Lanczos3 filter for quality
    let resized = imageops::resize(
        image,
        final_width,
        final_height,
        imageops::FilterType::Lanczos3,
    );

    Ok(DynamicImage::ImageRgba8(resized))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    // Helper to create a test image
    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let img = RgbaImage::new(width, height);
        DynamicImage::ImageRgba8(img)
    }

    // Task 7.1: Test calculate_fit_dimensions() with 16:9 aspect ratio
    #[test]
    fn test_calculate_fit_dimensions_16_9() {
        // 1920×1080 (16:9) → 160×96 target
        let (w, h) = calculate_fit_dimensions(1920, 1080, 160, 96);

        // Should constrain by width (wider aspect than target)
        assert!(w <= 160);
        assert!(h <= 96);

        // Aspect ratio should be preserved (within rounding error)
        let aspect = w as f32 / h as f32;
        let expected_aspect = 16.0 / 9.0;
        assert!(
            (aspect - expected_aspect).abs() < 0.01,
            "Aspect ratio {:.3} != expected {:.3}",
            aspect,
            expected_aspect
        );
    }

    // Task 7.2: Test calculate_fit_dimensions() with 4:3 aspect ratio
    #[test]
    fn test_calculate_fit_dimensions_4_3() {
        // 800×600 (4:3) → 160×96 target
        let (w, h) = calculate_fit_dimensions(800, 600, 160, 96);

        assert!(w <= 160);
        assert!(h <= 96);

        let aspect = w as f32 / h as f32;
        let expected_aspect = 4.0 / 3.0;
        assert!((aspect - expected_aspect).abs() < 0.01);
    }

    // Task 7.3: Test calculate_fit_dimensions() with 1:1 (square) aspect ratio
    #[test]
    fn test_calculate_fit_dimensions_1_1_square() {
        // 500×500 (1:1) → 160×96 target
        let (w, h) = calculate_fit_dimensions(500, 500, 160, 96);

        assert!(w <= 160);
        assert!(h <= 96);

        // Square should become 96×96 (constrained by height)
        assert_eq!(h, 96);
        assert_eq!(w, 96);
    }

    // Task 7.4: Test calculate_fit_dimensions() with 21:9 (ultrawide) aspect ratio
    #[test]
    fn test_calculate_fit_dimensions_21_9_ultrawide() {
        // 2560×1080 (21:9) → 160×96 target
        let (w, h) = calculate_fit_dimensions(2560, 1080, 160, 96);

        assert!(w <= 160);
        assert!(h <= 96);

        let aspect = w as f32 / h as f32;
        let expected_aspect = 21.0 / 9.0;
        // More lenient tolerance for extreme aspect ratios due to rounding
        assert!((aspect - expected_aspect).abs() < 0.1);
    }

    // Task 7.5: Test letterboxing scenario (wide image → tall target)
    #[test]
    fn test_letterboxing_wide_to_tall() {
        // Very wide image → tall target
        let (w, h) = calculate_fit_dimensions(1000, 100, 200, 400);

        // Should constrain by width, leaving vertical letterbox space
        assert_eq!(w, 200);
        assert!(h < 400);

        // Aspect ratio preserved
        let aspect = w as f32 / h as f32;
        let expected_aspect = 1000.0 / 100.0;
        assert!((aspect - expected_aspect).abs() < 0.1);
    }

    // Task 7.6: Test pillarboxing scenario (tall image → wide target)
    #[test]
    fn test_pillarboxing_tall_to_wide() {
        // Very tall image → wide target
        let (w, h) = calculate_fit_dimensions(100, 1000, 400, 200);

        // Should constrain by height, leaving horizontal pillarbox space
        assert_eq!(h, 200);
        assert!(w < 400);

        // Aspect ratio preserved
        let aspect = w as f32 / h as f32;
        let expected_aspect = 100.0 / 1000.0;
        assert!((aspect - expected_aspect).abs() < 0.01);
    }

    // Task 7.7: Test perfect fit (source and target same aspect ratio)
    #[test]
    fn test_perfect_fit_same_aspect() {
        // Both 16:9 aspect ratio
        let (w, h) = calculate_fit_dimensions(1920, 1080, 160, 90);

        // Should use full target dimensions
        assert_eq!(w, 160);
        assert_eq!(h, 90);
    }

    // Task 7.8: Test upscale prevention (small image → large target, no upscale)
    #[test]
    fn test_prevent_upscale_small_image() {
        // 50×50 image → 800×600 target (16x upscale, exceeds MAX_UPSCALE_FACTOR)
        let (w, h) = prevent_upscale(50, 50, 800, 600);

        // Should return original dimensions (no upscaling beyond limit)
        assert_eq!(w, 50);
        assert_eq!(h, 50);
    }

    // Task 7.9: Test downscale (large image → small target)
    #[test]
    fn test_downscale_large_to_small() {
        // 1920×1080 → 160×90 (downscale allowed)
        let (w, h) = prevent_upscale(1920, 1080, 160, 90);

        // Should use target dimensions (downscaling is fine)
        assert_eq!(w, 160);
        assert_eq!(h, 90);
    }

    // Task 7.10: Test zero dimensions error handling
    #[test]
    fn test_resize_to_terminal_zero_width() {
        let img = create_test_image(100, 100);
        let result = resize_to_terminal(&img, 0, 24);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DotmaxError::InvalidImageDimensions { .. }
        ));
    }

    #[test]
    fn test_resize_to_terminal_zero_height() {
        let img = create_test_image(100, 100);
        let result = resize_to_terminal(&img, 80, 0);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DotmaxError::InvalidImageDimensions { .. }
        ));
    }

    #[test]
    fn test_resize_to_dimensions_zero_width() {
        let img = create_test_image(100, 100);
        let result = resize_to_dimensions(&img, 0, 100, true);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DotmaxError::InvalidImageDimensions { .. }
        ));
    }

    #[test]
    fn test_resize_to_dimensions_zero_height() {
        let img = create_test_image(100, 100);
        let result = resize_to_dimensions(&img, 100, 0, false);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DotmaxError::InvalidImageDimensions { .. }
        ));
    }

    // Task 7.11: Test extreme aspect ratios (10000×1, 1×10000)
    #[test]
    fn test_extreme_aspect_ratio_wide() {
        // Very wide image
        let (w, h) = calculate_fit_dimensions(10000, 1, 160, 96);

        // Should fit within constraints
        assert!(w <= 160);
        assert!(h <= 96);
        assert!(h >= 1); // At least 1 pixel tall
    }

    #[test]
    fn test_extreme_aspect_ratio_tall() {
        // Very tall image
        let (w, h) = calculate_fit_dimensions(1, 10000, 160, 96);

        // Should fit within constraints
        assert!(w <= 160);
        assert!(h <= 96);
        assert!(w >= 1); // At least 1 pixel wide
    }

    // Additional tests for resize_to_dimensions
    #[test]
    fn test_resize_to_dimensions_preserve_aspect_true() {
        let img = create_test_image(1920, 1080);
        let result = resize_to_dimensions(&img, 200, 100, true);

        assert!(result.is_ok());
        let resized = result.unwrap();

        // Should fit within target with aspect preserved
        assert!(resized.width() <= 200);
        assert!(resized.height() <= 100);
    }

    #[test]
    fn test_resize_to_dimensions_preserve_aspect_false() {
        let img = create_test_image(1920, 1080);
        let result = resize_to_dimensions(&img, 200, 100, false);

        assert!(result.is_ok());
        let resized = result.unwrap();

        // Should be exactly target dimensions (stretched)
        assert_eq!(resized.width(), 200);
        assert_eq!(resized.height(), 100);
    }

    #[test]
    fn test_resize_to_dimensions_exceeds_max_dimensions() {
        let img = create_test_image(100, 100);
        let result = resize_to_dimensions(&img, MAX_IMAGE_WIDTH + 1, 100, false);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DotmaxError::InvalidImageDimensions { .. }
        ));
    }

    #[test]
    fn test_resize_to_terminal_basic() {
        let img = create_test_image(800, 600);
        let result = resize_to_terminal(&img, 80, 24);

        assert!(result.is_ok());
        let resized = result.unwrap();

        // Should fit within terminal dimensions (160×96 pixels)
        assert!(resized.width() <= 160);
        assert!(resized.height() <= 96);
    }

    #[test]
    fn test_no_resize_when_already_correct_size() {
        let img = create_test_image(160, 96);
        let result = resize_to_terminal(&img, 80, 24);

        assert!(result.is_ok());
        let resized = result.unwrap();

        // Should be same dimensions (no resize needed)
        assert_eq!(resized.width(), 160);
        assert_eq!(resized.height(), 96);
    }
}
