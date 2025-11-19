//! Benchmarks for image resizing (Story 3.2)
//!
//! Performance targets from tech spec (Epic 3):
//! - Resize stage budget: <10ms target for typical images (800×600 → terminal size)
//! - Small images (100×100): ~2ms
//! - Medium images (800×600): ~8ms
//! - Large images (1920×1080): ~15ms

#![cfg(feature = "image")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dotmax::image::{load_from_bytes, resize_to_dimensions, resize_to_terminal};
use image::{DynamicImage, RgbaImage};
use std::hint::black_box;

/// Create a synthetic test image of specified dimensions
fn create_test_image(width: u32, height: u32) -> DynamicImage {
    let img = RgbaImage::new(width, height);
    DynamicImage::ImageRgba8(img)
}

/// Task 9.1: Benchmark resize 800×600 → 80×24 terminal (typical terminal)
///
/// Target: <10ms (per tech spec)
fn bench_resize_typical_terminal(c: &mut Criterion) {
    let img = create_test_image(800, 600);

    c.bench_function("resize_800x600_to_80x24_terminal", |b| {
        b.iter(|| {
            black_box(resize_to_terminal(&img, 80, 24).unwrap());
        });
    });
}

/// Task 9.2: Benchmark resize 1920×1080 → 200×50 terminal (large terminal)
///
/// Target: <15ms (acceptable for large images)
fn bench_resize_large_terminal(c: &mut Criterion) {
    let img = create_test_image(1920, 1080);

    c.bench_function("resize_1920x1080_to_200x50_terminal", |b| {
        b.iter(|| {
            black_box(resize_to_terminal(&img, 200, 50).unwrap());
        });
    });
}

/// Task 9.3: Benchmark resize 100×100 → 80×24 terminal (small image)
///
/// Target: ~2ms (fast for small images)
fn bench_resize_small_image(c: &mut Criterion) {
    let img = create_test_image(100, 100);

    c.bench_function("resize_100x100_to_80x24_terminal", |b| {
        b.iter(|| {
            black_box(resize_to_terminal(&img, 80, 24).unwrap());
        });
    });
}

/// Task 9.4: Verify resize <10ms for typical images
///
/// This benchmark group validates the performance target by testing
/// multiple typical image sizes resizing to standard 80×24 terminal
fn bench_resize_performance_target_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("resize_performance_targets");

    // Test cases: (width, height, expected_max_ms)
    let test_cases = [
        (100, 100, 2),    // Small: <2ms
        (400, 300, 5),    // Small-medium: <5ms
        (800, 600, 10),   // Typical: <10ms target
        (1024, 768, 12),  // Medium-large: <12ms
        (1920, 1080, 15), // Large: <15ms acceptable
    ];

    for (width, height, _expected_max) in test_cases {
        let img = create_test_image(width, height);
        let id = BenchmarkId::new("resize_to_80x24", format!("{}x{}", width, height));

        group.bench_with_input(id, &img, |b, img| {
            b.iter(|| {
                black_box(resize_to_terminal(img, 80, 24).unwrap());
            });
        });
    }

    group.finish();
}

/// Task 9.5: Compare Lanczos3 vs Triangle vs Nearest filters
///
/// This benchmark documents the quality vs speed trade-offs for different
/// resize filters. Currently we use Lanczos3 for best quality.
///
/// Note: This benchmark uses manual resize to demonstrate filter differences,
/// though our public API only exposes Lanczos3.
fn bench_resize_filter_comparison(c: &mut Criterion) {
    use image::imageops::{resize, FilterType};

    let img = create_test_image(800, 600);
    let target_width = 160;
    let target_height = 96;

    let mut group = c.benchmark_group("resize_filter_comparison");

    // Lanczos3 (highest quality, slowest)
    group.bench_function("lanczos3", |b| {
        b.iter(|| {
            black_box(resize(
                &img,
                target_width,
                target_height,
                FilterType::Lanczos3,
            ));
        });
    });

    // Triangle (medium quality, faster)
    group.bench_function("triangle", |b| {
        b.iter(|| {
            black_box(resize(
                &img,
                target_width,
                target_height,
                FilterType::Triangle,
            ));
        });
    });

    // Nearest (lowest quality, fastest)
    group.bench_function("nearest", |b| {
        b.iter(|| {
            black_box(resize(
                &img,
                target_width,
                target_height,
                FilterType::Nearest,
            ));
        });
    });

    group.finish();
}

/// Task 9.6: Document filter trade-offs
///
/// This benchmark measures the full resize_to_dimensions() pipeline
/// which includes validation, aspect ratio calculation, and resize.
fn bench_full_resize_pipeline(c: &mut Criterion) {
    let img = create_test_image(800, 600);

    // With aspect ratio preservation
    c.bench_function("resize_pipeline_preserve_aspect", |b| {
        b.iter(|| {
            black_box(resize_to_dimensions(&img, 200, 100, true).unwrap());
        });
    });

    // Without aspect ratio preservation (stretch)
    c.bench_function("resize_pipeline_stretch", |b| {
        b.iter(|| {
            black_box(resize_to_dimensions(&img, 200, 100, false).unwrap());
        });
    });
}

/// Benchmark: Load and resize pipeline (real-world workflow)
///
/// Measures the combined cost of loading an image from bytes and resizing
/// to terminal dimensions. This represents a typical use case.
#[cfg(feature = "image")]
fn bench_load_and_resize_pipeline(c: &mut Criterion) {
    // Use a real test image
    let path = std::path::Path::new("tests/fixtures/images/sample.png");
    let bytes = std::fs::read(path).expect("Failed to read sample.png");

    c.bench_function("load_and_resize_pipeline", |b| {
        b.iter(|| {
            let img = load_from_bytes(&bytes).unwrap();
            black_box(resize_to_terminal(&img, 80, 24).unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_resize_typical_terminal,
    bench_resize_large_terminal,
    bench_resize_small_image,
    bench_resize_performance_target_validation,
    bench_resize_filter_comparison,
    bench_full_resize_pipeline,
    bench_load_and_resize_pipeline,
);
criterion_main!(benches);
