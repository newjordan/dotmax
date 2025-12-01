//! Image processing benchmarks for Story 7.2 (AC2, AC5)
//!
//! Benchmark groups:
//! - `image_load` - Loading images from disk
//! - `image_resize` - Resizing to terminal dimensions
//! - `dither_algorithms` - Floyd-Steinberg, Bayer, Atkinson comparison
//! - `threshold` - Otsu thresholding performance
//! - `full_pipeline` - End-to-end image-to-braille conversion
//!
//! Performance target (AC5): 80x24 terminal render < 25ms

#![cfg(feature = "image")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dotmax::image::{
    apply_dithering, auto_threshold, load_from_path, pixels_to_braille, resize_to_terminal,
    to_grayscale, DitheringMethod,
};
use image::{DynamicImage, GrayImage, Luma, RgbaImage};
use std::hint::black_box;
use std::path::Path;
use std::time::Duration;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a synthetic test image of specified dimensions
fn create_test_image(width: u32, height: u32) -> DynamicImage {
    // Create gradient image for realistic dithering behavior
    let img = RgbaImage::from_fn(width, height, |x, y| {
        let r = ((x as f32 / width as f32) * 255.0) as u8;
        let g = ((y as f32 / height as f32) * 255.0) as u8;
        let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
        image::Rgba([r, g, b, 255])
    });
    DynamicImage::ImageRgba8(img)
}

/// Create a grayscale gradient image
fn create_gradient_gray(width: u32, height: u32) -> GrayImage {
    GrayImage::from_fn(width, height, |x, y| {
        let value = ((x + y) as f32 / (width + height) as f32 * 255.0) as u8;
        Luma([value])
    })
}

// ============================================================================
// Image Load Benchmarks (AC2: image_load group)
// ============================================================================

fn bench_image_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_load");

    // Test loading PNG
    let png_path = Path::new("tests/fixtures/images/sample.png");
    if png_path.exists() {
        group.bench_function("load_png_sample", |b| {
            b.iter(|| black_box(load_from_path(png_path).unwrap()));
        });
    }

    // Test loading from bytes (common pattern)
    if png_path.exists() {
        let bytes = std::fs::read(png_path).expect("Failed to read sample.png");
        group.bench_function("load_from_bytes_png", |b| {
            b.iter(|| {
                black_box(dotmax::image::load_from_bytes(&bytes).unwrap());
            });
        });
    }

    group.finish();
}

// ============================================================================
// Image Resize Benchmarks (AC2: image_resize group)
// ============================================================================

fn bench_image_resize(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_resize");

    // Test various source sizes to terminal dimensions
    let test_cases = [
        (100, 100, "100x100_small"),
        (800, 600, "800x600_typical"),
        (1920, 1080, "1920x1080_hd"),
    ];

    for (width, height, label) in test_cases {
        let img = create_test_image(width, height);

        // Resize to 80x24 terminal
        group.bench_with_input(
            BenchmarkId::new("to_80x24", label),
            &img,
            |b, img| {
                b.iter(|| black_box(resize_to_terminal(img, 80, 24).unwrap()));
            },
        );
    }

    // Resize to various terminal sizes from typical 800x600 source
    let source_img = create_test_image(800, 600);
    let terminal_sizes = [
        (40, 12, "40x12_small"),
        (80, 24, "80x24_standard"),
        (160, 48, "160x48_large"),
        (200, 50, "200x50_max"),
    ];

    for (tw, th, label) in terminal_sizes {
        group.bench_with_input(
            BenchmarkId::new("800x600_to", label),
            &(tw, th),
            |b, &(tw, th)| {
                b.iter(|| black_box(resize_to_terminal(&source_img, tw, th).unwrap()));
            },
        );
    }

    group.finish();
}

// ============================================================================
// Dithering Algorithm Benchmarks (AC2: dither_algorithms group)
// ============================================================================

fn bench_dither_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("dither_algorithms");

    // Standard terminal size: 160x96 pixels (80x24 cells * 2x4 dots)
    let gray = create_gradient_gray(160, 96);

    // Floyd-Steinberg
    group.bench_function("floyd_steinberg_160x96", |b| {
        b.iter(|| {
            black_box(apply_dithering(&gray, DitheringMethod::FloydSteinberg).unwrap());
        });
    });

    // Bayer
    group.bench_function("bayer_160x96", |b| {
        b.iter(|| {
            black_box(apply_dithering(&gray, DitheringMethod::Bayer).unwrap());
        });
    });

    // Atkinson
    group.bench_function("atkinson_160x96", |b| {
        b.iter(|| {
            black_box(apply_dithering(&gray, DitheringMethod::Atkinson).unwrap());
        });
    });

    // Comparison at different sizes
    let large_gray = create_gradient_gray(320, 192); // 160x48 terminal
    group.bench_function("floyd_steinberg_320x192_large", |b| {
        b.iter(|| {
            black_box(apply_dithering(&large_gray, DitheringMethod::FloydSteinberg).unwrap());
        });
    });

    group.finish();
}

// ============================================================================
// Threshold Benchmarks (AC2: threshold group)
// ============================================================================

fn bench_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("threshold");

    // Load and resize real image for realistic thresholding
    let test_cases = [
        (160, 96, "160x96_80x24_terminal"),
        (320, 192, "320x192_160x48_terminal"),
        (400, 200, "400x200_200x50_terminal"),
    ];

    for (width, height, label) in test_cases {
        let img = create_test_image(width, height);

        group.bench_with_input(BenchmarkId::new("otsu", label), &img, |b, img| {
            b.iter(|| black_box(auto_threshold(img)));
        });
    }

    group.finish();
}

// ============================================================================
// Full Pipeline Benchmarks (AC2, AC5: full_pipeline group)
// ============================================================================

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    // AC5 Target: 80x24 terminal < 25ms
    group.measurement_time(Duration::from_secs(10));

    // Test with synthetic image
    let source_img = create_test_image(800, 600);

    // Full pipeline: resize → threshold → braille (auto_threshold handles grayscale internally)
    group.bench_function("pipeline_80x24_from_800x600", |b| {
        b.iter(|| {
            let resized = resize_to_terminal(&source_img, 80, 24).unwrap();
            let binary = auto_threshold(&resized);
            let grid = pixels_to_braille(&binary, 80_usize, 24_usize).unwrap();
            black_box(grid);
        });
    });

    // Full pipeline with dithering (alternative path)
    group.bench_function("pipeline_80x24_dithered", |b| {
        b.iter(|| {
            let resized = resize_to_terminal(&source_img, 80, 24).unwrap();
            let gray = to_grayscale(&resized);
            let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg).unwrap();
            let grid = pixels_to_braille(&binary, 80_usize, 24_usize).unwrap();
            black_box(grid);
        });
    });

    // Various terminal sizes (using usize for pixels_to_braille)
    let terminal_sizes: [(u16, u16, &str); 4] = [
        (40, 12, "40x12_small"),
        (80, 24, "80x24_standard"),
        (160, 48, "160x48_large"),
        (200, 50, "200x50_max"),
    ];

    for (tw, th, label) in terminal_sizes {
        group.bench_with_input(
            BenchmarkId::new("pipeline", label),
            &(tw, th),
            |b, &(tw, th)| {
                b.iter(|| {
                    let resized = resize_to_terminal(&source_img, tw, th).unwrap();
                    let binary = auto_threshold(&resized);
                    let grid = pixels_to_braille(&binary, tw as usize, th as usize).unwrap();
                    black_box(grid);
                });
            },
        );
    }

    // Test with real image if available
    let sample_path = Path::new("tests/fixtures/images/sample.png");
    if sample_path.exists() {
        let real_img = load_from_path(sample_path).expect("Failed to load sample.png");

        group.bench_function("pipeline_80x24_real_image", |b| {
            b.iter(|| {
                let resized = resize_to_terminal(&real_img, 80, 24).unwrap();
                let binary = auto_threshold(&resized);
                let grid = pixels_to_braille(&binary, 80_usize, 24_usize).unwrap();
                black_box(grid);
            });
        });
    }

    group.finish();
}

/// Benchmark including file I/O (realistic end-to-end)
fn bench_full_pipeline_with_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline_with_load");
    group.measurement_time(Duration::from_secs(10));

    let sample_path = Path::new("tests/fixtures/images/sample.png");
    if sample_path.exists() {
        group.bench_function("load_and_render_80x24", |b| {
            b.iter(|| {
                let img = load_from_path(sample_path).unwrap();
                let resized = resize_to_terminal(&img, 80, 24).unwrap();
                let binary = auto_threshold(&resized);
                let grid = pixels_to_braille(&binary, 80_usize, 24_usize).unwrap();
                black_box(grid);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Grayscale Conversion Benchmarks (pipeline stage)
// ============================================================================

fn bench_grayscale_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("grayscale_conversion");

    let sizes: [(u32, u32, &str); 3] = [
        (160, 96, "160x96"),
        (320, 192, "320x192"),
        (400, 200, "400x200"),
    ];

    for (width, height, label) in sizes {
        let img = create_test_image(width, height);
        let tw = (width / 2) as u16;
        let th = (height / 4) as u16;
        let resized = resize_to_terminal(&img, tw, th).unwrap();

        group.bench_with_input(BenchmarkId::new("to_grayscale", label), &resized, |b, img| {
            b.iter(|| black_box(to_grayscale(img)));
        });
    }

    group.finish();
}

// ============================================================================
// Braille Mapping Benchmarks (pipeline stage)
// ============================================================================

fn bench_braille_mapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("braille_mapping");

    let sizes: [(u16, u16, &str); 3] = [
        (80, 24, "80x24_standard"),
        (160, 48, "160x48_large"),
        (200, 50, "200x50_max"),
    ];

    for (tw, th, label) in sizes {
        let img = create_test_image(800, 600);
        let resized = resize_to_terminal(&img, tw, th).unwrap();
        let binary = auto_threshold(&resized);

        group.bench_with_input(
            BenchmarkId::new("pixels_to_braille", label),
            &(tw, th),
            |b, &(tw, th)| {
                b.iter(|| black_box(pixels_to_braille(&binary, tw as usize, th as usize).unwrap()));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_image_load,
    bench_image_resize,
    bench_dither_algorithms,
    bench_threshold,
    bench_grayscale_conversion,
    bench_braille_mapping,
    bench_full_pipeline,
    bench_full_pipeline_with_load
);
criterion_main!(benches);
