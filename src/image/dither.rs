//! Dithering algorithms for binary image conversion.
//!
//! This module provides three industry-standard dithering algorithms to convert
//! grayscale images to binary (black/white) images while preserving visual quality
//! through optimal error diffusion or ordered dithering patterns.
//!
//! # Algorithms
//!
//! ## Floyd-Steinberg (1976) - Error Diffusion
//!
//! The Floyd-Steinberg algorithm diffuses quantization error to neighboring pixels
//! using a carefully designed coefficient pattern. This produces the highest quality
//! results for photographs and complex images.
//!
//! **Reference:** Floyd, R. W.; Steinberg, L. (1976). "An Adaptive Algorithm for
//! Spatial Grey Scale". Proceedings of the Society of Information Display. 17: 75–77.
//!
//! **Characteristics:**
//! - Best quality (minimal visual artifacts)
//! - Slowest (error diffusion to 4 neighbors per pixel)
//! - Target performance: <15ms for 160×96 images
//!
//! ## Bayer Ordered Dithering - Threshold Matrix
//!
//! Bayer dithering uses an 8×8 threshold matrix to make binary decisions without
//! error propagation. This makes it stateless, parallelizable, and fast.
//!
//! **Reference:** Bayer, B. E. (1973). "An optimum method for two-level rendition
//! of continuous-tone pictures". IEEE International Conference on Communications.
//!
//! **Characteristics:**
//! - Good quality (visible pattern on uniform areas)
//! - Fastest (no error propagation, stateless)
//! - Target performance: <10ms for 160×96 images
//!
//! ## Atkinson (1984) - Partial Error Diffusion
//!
//! The Atkinson algorithm, developed by Bill Atkinson for Apple MacPaint, diffuses
//! only 75% of the quantization error (6/8), discarding 25%. This produces softer,
//! more artistic results compared to Floyd-Steinberg.
//!
//! **Reference:** Bill Atkinson, Apple Computer (1984). Algorithm used in MacPaint
//! and HyperCard.
//!
//! **Characteristics:**
//! - Artistic quality (softer than Floyd-Steinberg)
//! - Moderate speed (error diffusion to 6 neighbors)
//! - Target performance: <12ms for 160×96 images
//!
//! # Examples
//!
//! ## Using the unified API
//!
//! ```no_run
//! use dotmax::image::{to_grayscale, load_from_path};
//! use dotmax::image::dither::{apply_dithering, DitheringMethod};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = load_from_path(Path::new("photo.jpg"))?;
//! let gray = to_grayscale(&img);
//! let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg)?;
//! println!("Dithered to {}×{} binary image", binary.width, binary.height);
//! # Ok(())
//! # }
//! ```
//!
//! ## Comparing algorithms
//!
//! ```no_run
//! # use dotmax::image::{to_grayscale, load_from_path};
//! # use dotmax::image::dither::{apply_dithering, DitheringMethod};
//! # use std::path::Path;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let img = load_from_path(Path::new("photo.jpg"))?;
//! # let gray = to_grayscale(&img);
//! // Try all three methods
//! let floyd = apply_dithering(&gray, DitheringMethod::FloydSteinberg)?;
//! let bayer = apply_dithering(&gray, DitheringMethod::Bayer)?;
//! let atkinson = apply_dithering(&gray, DitheringMethod::Atkinson)?;
//!
//! // Or skip dithering and use direct threshold
//! let direct = apply_dithering(&gray, DitheringMethod::None)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Trade-offs
//!
//! | Algorithm | Speed | Quality | Best For |
//! |-----------|-------|---------|----------|
//! | Floyd-Steinberg | Slower | Highest | Photos, complex images |
//! | Bayer | Fastest | Good | Gradients, simple images |
//! | Atkinson | Moderate | Artistic | Line art, artistic renders |
//! | None (direct threshold) | Fast | Basic | When dithering not needed |

use image::GrayImage;
use tracing::debug;

use crate::error::DotmaxError;
use crate::image::threshold::{auto_threshold, BinaryImage};

/// Dithering algorithm selection.
///
/// This enum allows users to choose between different dithering algorithms,
/// each with different performance and quality characteristics.
///
/// # Examples
///
/// ```
/// use dotmax::image::dither::DitheringMethod;
///
/// // For best quality (photographs)
/// let method = DitheringMethod::FloydSteinberg;
///
/// // For fastest rendering (real-time)
/// let method = DitheringMethod::Bayer;
///
/// // For artistic output
/// let method = DitheringMethod::Atkinson;
///
/// // For no dithering (direct threshold)
/// let method = DitheringMethod::None;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitheringMethod {
    /// Skip dithering and use direct Otsu threshold.
    ///
    /// This is the fastest option but produces the lowest quality for images
    /// with gradients or complex tonal ranges.
    None,

    /// Floyd-Steinberg error diffusion (1976).
    ///
    /// Diffuses quantization error to 4 neighbors with coefficients:
    /// - Right pixel (x+1, y): 7/16
    /// - Bottom-left (x-1, y+1): 3/16
    /// - Bottom (x, y+1): 5/16
    /// - Bottom-right (x+1, y+1): 1/16
    ///
    /// **Best for:** Photographs, complex images with many tonal variations.
    ///
    /// **Performance:** ~15ms for 160×96 images (slowest, highest quality).
    FloydSteinberg,

    /// Bayer ordered dithering with 8×8 threshold matrix.
    ///
    /// Uses a stateless ordered dithering pattern. No error propagation,
    /// making it parallelizable and fast.
    ///
    /// **Best for:** Gradients, simple images, real-time rendering.
    ///
    /// **Performance:** ~10ms for 160×96 images (fastest).
    Bayer,

    /// Atkinson error diffusion (1984, Apple MacPaint).
    ///
    /// Diffuses only 6/8 of quantization error to 6 neighbors (1/8 each),
    /// discarding 2/8. This produces softer, more artistic output compared
    /// to Floyd-Steinberg.
    ///
    /// **Best for:** Line art, diagrams, artistic rendering.
    ///
    /// **Performance:** ~12ms for 160×96 images (moderate).
    Atkinson,
}

/// Standard 8×8 Bayer threshold matrix.
///
/// Values range from 0-63 and are used to determine threshold points
/// in the ordered dithering pattern. The matrix is normalized to 0.0-1.0
/// range during threshold comparison.
///
/// Reference: Bayer, B. E. (1973). "An optimum method for two-level rendition
/// of continuous-tone pictures".
const BAYER_MATRIX_8X8: [[u8; 8]; 8] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

/// Threshold value for binary decision (middle gray).
///
/// Pixels with value >= THRESHOLD become white (true), pixels < THRESHOLD become black (false).
const THRESHOLD: u8 = 127;

/// Apply dithering to a grayscale image using the specified method.
///
/// This is the primary entry point for dithering. It dispatches to the appropriate
/// algorithm based on the `method` parameter.
///
/// # Arguments
///
/// * `gray` - Input grayscale image (8-bit per pixel)
/// * `method` - Dithering algorithm to use
///
/// # Returns
///
/// A binary image (boolean pixels) where `true` = black dot, `false` = white/empty.
///
/// # Errors
///
/// Returns [`DotmaxError::InvalidParameter`] if:
/// - Image width or height is 0
/// - Image dimensions would cause overflow
///
/// # Examples
///
/// ```no_run
/// use dotmax::image::{to_grayscale, load_from_path};
/// use dotmax::image::dither::{apply_dithering, DitheringMethod};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let img = load_from_path(Path::new("photo.jpg"))?;
/// let gray = to_grayscale(&img);
///
/// // High quality dithering
/// let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg)?;
/// # Ok(())
/// # }
/// ```
pub fn apply_dithering(
    gray: &GrayImage,
    method: DitheringMethod,
) -> Result<BinaryImage, DotmaxError> {
    debug!("Applying {:?} dithering to {}×{} image", method, gray.width(), gray.height());

    match method {
        DitheringMethod::None => {
            // Use auto_threshold from threshold module (Otsu + binary conversion)
            let binary = auto_threshold(&image::DynamicImage::ImageLuma8(gray.clone()));
            Ok(binary)
        }
        DitheringMethod::FloydSteinberg => floyd_steinberg(gray),
        DitheringMethod::Bayer => bayer(gray),
        DitheringMethod::Atkinson => atkinson(gray),
    }
}

/// Floyd-Steinberg error diffusion dithering.
///
/// Implements the classic Floyd-Steinberg algorithm (1976) which diffuses
/// quantization error to 4 neighboring pixels. This produces high-quality
/// results but is the slowest of the three methods.
///
/// # Algorithm
///
/// For each pixel (left to right, top to bottom):
/// 1. Calculate `new_value = old_value + accumulated_error`
/// 2. Apply threshold: `output = if new_value >= 127 { 255 } else { 0 }`
/// 3. Calculate error: `error = new_value - output`
/// 4. Diffuse error to neighbors:
///    - Right (x+1, y): `error * 7/16`
///    - Bottom-left (x-1, y+1): `error * 3/16`
///    - Bottom (x, y+1): `error * 5/16`
///    - Bottom-right (x+1, y+1): `error * 1/16`
///
/// # Performance
///
/// Target: <15ms for 160×96 images
///
/// # Errors
///
/// Returns [`DotmaxError::InvalidParameter`] if image dimensions are 0.
///
/// # Examples
///
/// ```no_run
/// # use dotmax::image::{to_grayscale, load_from_path};
/// # use dotmax::image::dither::floyd_steinberg;
/// # use std::path::Path;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let img = load_from_path(Path::new("photo.jpg"))?;
/// # let gray = to_grayscale(&img);
/// let binary = floyd_steinberg(&gray)?;
/// # Ok(())
/// # }
/// ```
pub fn floyd_steinberg(gray: &GrayImage) -> Result<BinaryImage, DotmaxError> {
    let width = gray.width() as usize;
    let height = gray.height() as usize;

    if width == 0 || height == 0 {
        return Err(DotmaxError::InvalidParameter {
            parameter_name: "image dimensions".to_string(),
            value: format!("{}×{}", width, height),
            min: "1×1".to_string(),
            max: "unlimited".to_string(),
        });
    }

    debug!("Floyd-Steinberg dithering {}×{} image", width, height);

    // Create error buffer (accumulated errors from previous pixels)
    // Using f32 for precision with fractional coefficients
    let mut errors = vec![0.0f32; width * height];
    let mut binary = BinaryImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let pixel_idx = y * width + x;
            let old_pixel = gray.get_pixel(x as u32, y as u32)[0] as f32;
            let new_pixel = old_pixel + errors[pixel_idx];

            // Apply threshold
            let output_value = if new_pixel >= THRESHOLD as f32 {
                255.0
            } else {
                0.0
            };
            binary.set_pixel(x as u32, y as u32, output_value == 255.0);

            // Calculate quantization error
            let quant_error = new_pixel - output_value;

            // Diffuse error to neighbors (with boundary checks)
            // Right pixel (x+1, y): 7/16
            if x + 1 < width {
                errors[pixel_idx + 1] += quant_error * 7.0 / 16.0;
            }

            // Bottom row neighbors (y+1)
            if y + 1 < height {
                let next_row_idx = (y + 1) * width;

                // Bottom-left (x-1, y+1): 3/16
                if x > 0 {
                    errors[next_row_idx + x - 1] += quant_error * 3.0 / 16.0;
                }

                // Bottom (x, y+1): 5/16
                errors[next_row_idx + x] += quant_error * 5.0 / 16.0;

                // Bottom-right (x+1, y+1): 1/16
                if x + 1 < width {
                    errors[next_row_idx + x + 1] += quant_error * 1.0 / 16.0;
                }
            }
        }
    }

    Ok(binary)
}

/// Bayer ordered dithering with 8×8 threshold matrix.
///
/// Implements stateless ordered dithering using a standard 8×8 Bayer matrix.
/// Each pixel's binary value is determined independently based on its position
/// in the matrix pattern, making this algorithm fast and parallelizable.
///
/// # Algorithm
///
/// For each pixel at (x, y):
/// 1. Get Bayer threshold: `bayer_matrix[y % 8][x % 8] / 64.0`
/// 2. Compare: `if (pixel_value / 255.0) > bayer_threshold { white } else { black }`
///
/// # Performance
///
/// Target: <10ms for 160×96 images (fastest algorithm)
///
/// # Errors
///
/// Returns [`DotmaxError::InvalidParameter`] if image dimensions are 0.
///
/// # Examples
///
/// ```no_run
/// # use dotmax::image::{to_grayscale, load_from_path};
/// # use dotmax::image::dither::bayer;
/// # use std::path::Path;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let img = load_from_path(Path::new("gradient.png"))?;
/// # let gray = to_grayscale(&img);
/// let binary = bayer(&gray)?;
/// # Ok(())
/// # }
/// ```
pub fn bayer(gray: &GrayImage) -> Result<BinaryImage, DotmaxError> {
    let width = gray.width() as usize;
    let height = gray.height() as usize;

    if width == 0 || height == 0 {
        return Err(DotmaxError::InvalidParameter {
            parameter_name: "image dimensions".to_string(),
            value: format!("{}×{}", width, height),
            min: "1×1".to_string(),
            max: "unlimited".to_string(),
        });
    }

    debug!("Bayer dithering {}×{} image", width, height);

    let mut binary = BinaryImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let pixel_value = gray.get_pixel(x as u32, y as u32)[0];

            // Get Bayer threshold for this position (normalized to 0.0-1.0)
            let bayer_threshold = BAYER_MATRIX_8X8[y % 8][x % 8] as f32 / 64.0;

            // Compare normalized pixel value to Bayer threshold
            let normalized_pixel = pixel_value as f32 / 255.0;
            let output = normalized_pixel > bayer_threshold;

            binary.set_pixel(x as u32, y as u32, output);
        }
    }

    Ok(binary)
}

/// Atkinson error diffusion dithering.
///
/// Implements the Atkinson algorithm developed by Bill Atkinson for Apple MacPaint (1984).
/// This algorithm diffuses only 6/8 of the quantization error to 6 neighbors (1/8 each),
/// discarding 2/8. This produces softer, more artistic results than Floyd-Steinberg.
///
/// # Algorithm
///
/// For each pixel (left to right, top to bottom):
/// 1. Calculate `new_value = old_value + accumulated_error`
/// 2. Apply threshold: `output = if new_value >= 127 { 255 } else { 0 }`
/// 3. Calculate error: `error = new_value - output`
/// 4. Diffuse 1/8 of error to each of 6 neighbors:
///    - Right (x+1, y)
///    - Two-right (x+2, y)
///    - Bottom-left (x-1, y+1)
///    - Bottom (x, y+1)
///    - Bottom-right (x+1, y+1)
///    - Two-down (x, y+2)
/// 5. Discard 2/8 of error (Atkinson's signature)
///
/// # Performance
///
/// Target: <12ms for 160×96 images (moderate speed)
///
/// # Errors
///
/// Returns [`DotmaxError::InvalidParameter`] if image dimensions are 0.
///
/// # Examples
///
/// ```no_run
/// # use dotmax::image::{to_grayscale, load_from_path};
/// # use dotmax::image::dither::atkinson;
/// # use std::path::Path;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let img = load_from_path(Path::new("lineart.png"))?;
/// # let gray = to_grayscale(&img);
/// let binary = atkinson(&gray)?;
/// # Ok(())
/// # }
/// ```
pub fn atkinson(gray: &GrayImage) -> Result<BinaryImage, DotmaxError> {
    let width = gray.width() as usize;
    let height = gray.height() as usize;

    if width == 0 || height == 0 {
        return Err(DotmaxError::InvalidParameter {
            parameter_name: "image dimensions".to_string(),
            value: format!("{}×{}", width, height),
            min: "1×1".to_string(),
            max: "unlimited".to_string(),
        });
    }

    debug!("Atkinson dithering {}×{} image", width, height);

    // Create error buffer (accumulated errors from previous pixels)
    let mut errors = vec![0.0f32; width * height];
    let mut binary = BinaryImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let pixel_idx = y * width + x;
            let old_pixel = gray.get_pixel(x as u32, y as u32)[0] as f32;
            let new_pixel = old_pixel + errors[pixel_idx];

            // Apply threshold
            let output_value = if new_pixel >= THRESHOLD as f32 {
                255.0
            } else {
                0.0
            };
            binary.set_pixel(x as u32, y as u32, output_value == 255.0);

            // Calculate quantization error
            let quant_error = new_pixel - output_value;

            // Diffuse 1/8 of error to 6 neighbors (total: 6/8, discard 2/8)
            // Right pixel (x+1, y): 1/8
            if x + 1 < width {
                errors[pixel_idx + 1] += quant_error / 8.0;
            }

            // Two-right pixel (x+2, y): 1/8
            if x + 2 < width {
                errors[pixel_idx + 2] += quant_error / 8.0;
            }

            // Bottom row neighbors (y+1)
            if y + 1 < height {
                let next_row_idx = (y + 1) * width;

                // Bottom-left (x-1, y+1): 1/8
                if x > 0 {
                    errors[next_row_idx + x - 1] += quant_error / 8.0;
                }

                // Bottom (x, y+1): 1/8
                errors[next_row_idx + x] += quant_error / 8.0;

                // Bottom-right (x+1, y+1): 1/8
                if x + 1 < width {
                    errors[next_row_idx + x + 1] += quant_error / 8.0;
                }
            }

            // Two-down pixel (x, y+2): 1/8
            if y + 2 < height {
                let two_rows_idx = (y + 2) * width;
                errors[two_rows_idx + x] += quant_error / 8.0;
            }
        }
    }

    Ok(binary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    /// Helper: Create a uniform gray image (all pixels same value)
    fn create_uniform_image(width: u32, height: u32, value: u8) -> GrayImage {
        GrayImage::from_fn(width, height, |_, _| Luma([value]))
    }

    /// Helper: Create a gradient image (0 to 255 smooth gradient left to right)
    fn create_gradient_image(width: u32, height: u32) -> GrayImage {
        GrayImage::from_fn(width, height, |x, _| {
            let value = (x as f32 / width as f32 * 255.0) as u8;
            Luma([value])
        })
    }

    // ===== DitheringMethod Enum Tests =====

    #[test]
    fn test_dithering_method_enum() {
        // Verify all variants exist
        let _ = DitheringMethod::None;
        let _ = DitheringMethod::FloydSteinberg;
        let _ = DitheringMethod::Bayer;
        let _ = DitheringMethod::Atkinson;
    }

    #[test]
    fn test_dithering_method_derives() {
        let method1 = DitheringMethod::FloydSteinberg;
        let method2 = DitheringMethod::FloydSteinberg;
        let method3 = DitheringMethod::Bayer;

        // Test Debug
        assert_eq!(format!("{:?}", method1), "FloydSteinberg");

        // Test Clone
        let method1_clone = method1.clone();
        assert_eq!(method1, method1_clone);

        // Test Copy
        let method1_copy = method1;
        assert_eq!(method1, method1_copy);

        // Test PartialEq
        assert_eq!(method1, method2);
        assert_ne!(method1, method3);

        // Test Eq (no separate test needed, derives from PartialEq)
    }

    // ===== Floyd-Steinberg Tests =====

    #[test]
    fn test_floyd_steinberg_uniform_gray() {
        let gray = create_uniform_image(10, 10, 128);
        let binary = floyd_steinberg(&gray).unwrap();

        // For uniform gray (128), expect roughly 50% black/white distribution
        let black_count = binary.pixels.iter().filter(|&&p| p).count();
        let total = binary.pixels.len();

        // Should be roughly balanced (within 30% tolerance for small image)
        assert!(
            (black_count as f32 / total as f32) > 0.3 && (black_count as f32 / total as f32) < 0.7,
            "Expected balanced black/white for uniform gray, got {} black out of {}",
            black_count,
            total
        );
    }

    #[test]
    fn test_floyd_steinberg_all_black() {
        let gray = create_uniform_image(10, 10, 0);
        let binary = floyd_steinberg(&gray).unwrap();

        // All pixels should be black (false)
        assert!(
            binary.pixels.iter().all(|&p| !p),
            "Expected all black for input value 0"
        );
    }

    #[test]
    fn test_floyd_steinberg_all_white() {
        let gray = create_uniform_image(10, 10, 255);
        let binary = floyd_steinberg(&gray).unwrap();

        // All pixels should be white (true)
        assert!(
            binary.pixels.iter().all(|&p| p),
            "Expected all white for input value 255"
        );
    }

    #[test]
    fn test_floyd_steinberg_gradient() {
        let gray = create_gradient_image(100, 10);
        let binary = floyd_steinberg(&gray).unwrap();

        // Left side (dark) should have more black, right side (bright) more white
        let left_quarter = &binary.pixels[0..250]; // First 25% of pixels
        let right_quarter = &binary.pixels[750..1000]; // Last 25% of pixels

        let left_black = left_quarter.iter().filter(|&&p| !p).count();
        let right_black = right_quarter.iter().filter(|&&p| !p).count();

        assert!(
            left_black > right_black,
            "Expected more black on left (dark) side, got left={} right={}",
            left_black,
            right_black
        );
    }

    #[test]
    fn test_floyd_steinberg_small_image() {
        // Test 1×1 image (edge case)
        let gray = create_uniform_image(1, 1, 128);
        let binary = floyd_steinberg(&gray).unwrap();
        assert_eq!(binary.width, 1);
        assert_eq!(binary.height, 1);

        // Test 2×2 image
        let gray = create_uniform_image(2, 2, 128);
        let binary = floyd_steinberg(&gray).unwrap();
        assert_eq!(binary.width, 2);
        assert_eq!(binary.height, 2);

        // Test 3×3 image
        let gray = create_uniform_image(3, 3, 128);
        let binary = floyd_steinberg(&gray).unwrap();
        assert_eq!(binary.width, 3);
        assert_eq!(binary.height, 3);
    }

    #[test]
    fn test_floyd_steinberg_zero_dimensions() {
        // Zero width
        let gray = GrayImage::new(0, 10);
        assert!(floyd_steinberg(&gray).is_err());

        // Zero height
        let gray = GrayImage::new(10, 0);
        assert!(floyd_steinberg(&gray).is_err());

        // Both zero
        let gray = GrayImage::new(0, 0);
        assert!(floyd_steinberg(&gray).is_err());
    }

    // ===== Bayer Tests =====

    #[test]
    fn test_bayer_uniform_gray() {
        let gray = create_uniform_image(16, 16, 128);
        let binary = bayer(&gray).unwrap();

        // For uniform gray (128 = 50%), expect Bayer pattern visible
        let black_count = binary.pixels.iter().filter(|&&p| p).count();
        let total = binary.pixels.len();

        // Should be roughly balanced (within 30% tolerance)
        assert!(
            (black_count as f32 / total as f32) > 0.3 && (black_count as f32 / total as f32) < 0.7,
            "Expected balanced black/white for uniform gray, got {} black out of {}",
            black_count,
            total
        );
    }

    #[test]
    fn test_bayer_all_black() {
        let gray = create_uniform_image(16, 16, 0);
        let binary = bayer(&gray).unwrap();

        // All pixels should be black (false)
        assert!(
            binary.pixels.iter().all(|&p| !p),
            "Expected all black for input value 0"
        );
    }

    #[test]
    fn test_bayer_all_white() {
        let gray = create_uniform_image(16, 16, 255);
        let binary = bayer(&gray).unwrap();

        // All pixels should be white (true)
        assert!(
            binary.pixels.iter().all(|&p| p),
            "Expected all white for input value 255"
        );
    }

    #[test]
    fn test_bayer_deterministic() {
        // Same input should always produce same output
        let gray = create_uniform_image(10, 10, 128);
        let binary1 = bayer(&gray).unwrap();
        let binary2 = bayer(&gray).unwrap();

        assert_eq!(
            binary1.pixels, binary2.pixels,
            "Bayer should be deterministic"
        );
    }

    #[test]
    fn test_bayer_pattern_applied() {
        // Verify that Bayer matrix is being applied (pattern should be visible)
        let gray = create_uniform_image(8, 8, 128);
        let binary = bayer(&gray).unwrap();

        // For an 8×8 image with uniform gray, the Bayer pattern should create
        // a specific pattern. We can't predict exact output, but we can verify
        // it's not all black or all white.
        let black_count = binary.pixels.iter().filter(|&&p| p).count();
        assert!(
            black_count > 0 && black_count < binary.pixels.len(),
            "Bayer pattern should produce mixed output for uniform gray"
        );
    }

    #[test]
    fn test_bayer_small_image() {
        // Test 1×1 image
        let gray = create_uniform_image(1, 1, 128);
        let binary = bayer(&gray).unwrap();
        assert_eq!(binary.width, 1);
        assert_eq!(binary.height, 1);

        // Test 2×2 image
        let gray = create_uniform_image(2, 2, 128);
        let binary = bayer(&gray).unwrap();
        assert_eq!(binary.width, 2);
        assert_eq!(binary.height, 2);
    }

    #[test]
    fn test_bayer_zero_dimensions() {
        // Zero width
        let gray = GrayImage::new(0, 10);
        assert!(bayer(&gray).is_err());

        // Zero height
        let gray = GrayImage::new(10, 0);
        assert!(bayer(&gray).is_err());
    }

    // ===== Atkinson Tests =====

    #[test]
    fn test_atkinson_uniform_gray() {
        let gray = create_uniform_image(10, 10, 128);
        let binary = atkinson(&gray).unwrap();

        // For uniform gray (128), expect mixed output
        let black_count = binary.pixels.iter().filter(|&&p| p).count();
        let total = binary.pixels.len();

        // Should have some variation (within 30% tolerance)
        assert!(
            (black_count as f32 / total as f32) > 0.3 && (black_count as f32 / total as f32) < 0.7,
            "Expected balanced black/white for uniform gray, got {} black out of {}",
            black_count,
            total
        );
    }

    #[test]
    fn test_atkinson_all_black() {
        let gray = create_uniform_image(10, 10, 0);
        let binary = atkinson(&gray).unwrap();

        // All pixels should be black (false)
        assert!(
            binary.pixels.iter().all(|&p| !p),
            "Expected all black for input value 0"
        );
    }

    #[test]
    fn test_atkinson_all_white() {
        let gray = create_uniform_image(10, 10, 255);
        let binary = atkinson(&gray).unwrap();

        // All pixels should be white (true)
        assert!(
            binary.pixels.iter().all(|&p| p),
            "Expected all white for input value 255"
        );
    }

    #[test]
    fn test_atkinson_gradient() {
        let gray = create_gradient_image(100, 10);
        let binary = atkinson(&gray).unwrap();

        // Left side (dark) should have more black, right side (bright) more white
        let left_quarter = &binary.pixels[0..250]; // First 25% of pixels
        let right_quarter = &binary.pixels[750..1000]; // Last 25% of pixels

        let left_black = left_quarter.iter().filter(|&&p| !p).count();
        let right_black = right_quarter.iter().filter(|&&p| !p).count();

        assert!(
            left_black > right_black,
            "Expected more black on left (dark) side, got left={} right={}",
            left_black,
            right_black
        );
    }

    #[test]
    fn test_atkinson_small_image() {
        // Test 1×1 image
        let gray = create_uniform_image(1, 1, 128);
        let binary = atkinson(&gray).unwrap();
        assert_eq!(binary.width, 1);
        assert_eq!(binary.height, 1);

        // Test 2×2 image
        let gray = create_uniform_image(2, 2, 128);
        let binary = atkinson(&gray).unwrap();
        assert_eq!(binary.width, 2);
        assert_eq!(binary.height, 2);
    }

    #[test]
    fn test_atkinson_zero_dimensions() {
        // Zero width
        let gray = GrayImage::new(0, 10);
        assert!(atkinson(&gray).is_err());

        // Zero height
        let gray = GrayImage::new(10, 0);
        assert!(atkinson(&gray).is_err());
    }

    // ===== apply_dithering() Tests =====

    #[test]
    fn test_apply_dithering_none() {
        let gray = create_uniform_image(10, 10, 128);
        let binary = apply_dithering(&gray, DitheringMethod::None).unwrap();

        // Should use auto_threshold fallback
        assert_eq!(binary.width, 10);
        assert_eq!(binary.height, 10);
    }

    #[test]
    fn test_apply_dithering_floyd_steinberg() {
        let gray = create_uniform_image(10, 10, 128);
        let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg).unwrap();

        assert_eq!(binary.width, 10);
        assert_eq!(binary.height, 10);
    }

    #[test]
    fn test_apply_dithering_bayer() {
        let gray = create_uniform_image(10, 10, 128);
        let binary = apply_dithering(&gray, DitheringMethod::Bayer).unwrap();

        assert_eq!(binary.width, 10);
        assert_eq!(binary.height, 10);
    }

    #[test]
    fn test_apply_dithering_atkinson() {
        let gray = create_uniform_image(10, 10, 128);
        let binary = apply_dithering(&gray, DitheringMethod::Atkinson).unwrap();

        assert_eq!(binary.width, 10);
        assert_eq!(binary.height, 10);
    }

    #[test]
    fn test_all_algorithms_same_dimensions() {
        let gray = create_uniform_image(20, 15, 128);

        let none = apply_dithering(&gray, DitheringMethod::None).unwrap();
        let floyd = apply_dithering(&gray, DitheringMethod::FloydSteinberg).unwrap();
        let bayer = apply_dithering(&gray, DitheringMethod::Bayer).unwrap();
        let atkinson = apply_dithering(&gray, DitheringMethod::Atkinson).unwrap();

        // All should preserve dimensions
        assert_eq!(none.width, 20);
        assert_eq!(none.height, 15);
        assert_eq!(floyd.width, 20);
        assert_eq!(floyd.height, 15);
        assert_eq!(bayer.width, 20);
        assert_eq!(bayer.height, 15);
        assert_eq!(atkinson.width, 20);
        assert_eq!(atkinson.height, 15);
    }

    #[test]
    fn test_algorithms_produce_different_output() {
        // All three algorithms should produce different patterns for same input
        let gray = create_uniform_image(20, 20, 128);

        let floyd = apply_dithering(&gray, DitheringMethod::FloydSteinberg).unwrap();
        let bayer = apply_dithering(&gray, DitheringMethod::Bayer).unwrap();
        let atkinson = apply_dithering(&gray, DitheringMethod::Atkinson).unwrap();

        // They should not all be identical
        let all_same = floyd.pixels == bayer.pixels && floyd.pixels == atkinson.pixels;
        assert!(
            !all_same,
            "Expected different patterns from different algorithms"
        );
    }

    // ===== Integration Tests =====

    #[test]
    fn test_large_image_performance_check() {
        // Test with 160×96 image (standard terminal size in pixels)
        let gray = create_uniform_image(160, 96, 128);

        // All algorithms should complete without panic
        let _ = floyd_steinberg(&gray).unwrap();
        let _ = bayer(&gray).unwrap();
        let _ = atkinson(&gray).unwrap();
    }

    #[test]
    fn test_extreme_dimensions() {
        // Very wide image
        let gray = create_uniform_image(1000, 1, 128);
        assert!(floyd_steinberg(&gray).is_ok());
        assert!(bayer(&gray).is_ok());
        assert!(atkinson(&gray).is_ok());

        // Very tall image
        let gray = create_uniform_image(1, 1000, 128);
        assert!(floyd_steinberg(&gray).is_ok());
        assert!(bayer(&gray).is_ok());
        assert!(atkinson(&gray).is_ok());
    }
}
