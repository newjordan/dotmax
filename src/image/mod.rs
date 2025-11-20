//! Image rendering module (feature-gated)
//!
//! This module provides functionality to load and render images to braille grids.
//! All image loading code is behind the `image` feature flag to keep the core
//! library lightweight.
//!
//! # Feature Gate
//!
//! To use this module, enable the `image` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! dotmax = { version = "0.1", features = ["image"] }
//! ```
//!
//! # Supported Formats
//!
//! The following image formats are supported via the `image` crate:
//! - PNG (Portable Network Graphics)
//! - JPEG/JPG (Joint Photographic Experts Group)
//! - GIF (Graphics Interchange Format)
//! - BMP (Windows Bitmap)
//! - WebP (Google WebP)
//! - TIFF (Tagged Image File Format)
//!
//! Format detection is automatic based on file magic bytes.
//!
//! # SVG Support
//!
//! SVG vector graphics are supported via the `svg` feature flag (separate from raster images).
//! Enable with:
//!
//! ```toml
//! [dependencies]
//! dotmax = { version = "0.1", features = ["svg"] }
//! ```
//!
//! SVG files are rasterized to pixel buffers before braille mapping. See the `svg` module for details.
//!
//! # Examples
//!
//! ## Loading from file path
//!
//! ```no_run
//! use dotmax::image::load_from_path;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = load_from_path(Path::new("image.png"))?;
//! println!("Loaded image: {}×{}", img.width(), img.height());
//! # Ok(())
//! # }
//! ```
//!
//! ## Loading from byte buffer
//!
//! ```no_run
//! use dotmax::image::load_from_bytes;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let bytes = include_bytes!("../../tests/fixtures/images/sample.png");
//! let img = load_from_bytes(bytes)?;
//! println!("Loaded image from bytes: {}×{}", img.width(), img.height());
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! Image loading targets <5ms for typical images (cached after first load).
//! The primary bottleneck is disk I/O for file-based loading.
//!
//! # Image Processing Pipeline
//!
//! The typical image-to-braille pipeline:
//! 1. Load image with [`load_from_path`] or [`load_from_bytes`]
//! 2. Resize to terminal dimensions with [`resize_to_terminal`]
//! 3. Convert to grayscale with [`convert::to_grayscale`]
//! 4. (Optional) Adjust brightness/contrast/gamma with [`threshold`] module functions
//! 5. (Optional) Apply dithering with [`dither::apply_dithering`] for improved quality
//! 6. Apply thresholding with [`threshold::auto_threshold`] or [`threshold::apply_threshold`]
//! 7. Map to braille grid (Story 3.5)

pub mod color_mode;
pub mod convert;
pub mod dither;
pub mod loader;
pub mod mapper;
pub mod resize;
#[cfg(feature = "svg")]
pub mod svg;
pub mod threshold;

// Re-export public functions for convenience
pub use color_mode::{render_image_with_color, ColorMode, ColorSamplingStrategy};
pub use convert::to_grayscale;
pub use dither::{apply_dithering, DitheringMethod};
pub use loader::{load_from_bytes, load_from_path, supported_formats};
pub use mapper::pixels_to_braille;
pub use resize::{resize_to_dimensions, resize_to_terminal};
#[cfg(feature = "svg")]
pub use svg::{load_svg_from_bytes, load_svg_from_path};
pub use threshold::{
    adjust_brightness, adjust_contrast, adjust_gamma, apply_threshold, auto_threshold,
    otsu_threshold, BinaryImage,
};
