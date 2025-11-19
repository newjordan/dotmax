//! Benchmarks for image grayscale conversion and thresholding operations
//!
//! These benchmarks measure the performance of the image-to-binary pipeline:
//! - Grayscale conversion (color → grayscale)
//! - Otsu threshold calculation
//! - Binary conversion (grayscale → binary with threshold)
//! - Brightness/contrast/gamma adjustments
//!
//! Performance targets (for 160×96 terminal-sized images):
//! - Grayscale conversion: <2ms
//! - Otsu threshold calculation: <5ms
//! - Binary conversion: <2ms
//! - Total auto_threshold pipeline: <10ms

#![cfg(feature = "image")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dotmax::image::{
    adjust_brightness, adjust_contrast, adjust_gamma, apply_threshold, auto_threshold,
    load_from_path, otsu_threshold, resize_to_terminal, to_grayscale,
};
use std::path::Path;

/// Benchmark grayscale conversion for terminal-sized image (160×96)
fn bench_grayscale_conversion(c: &mut Criterion) {
    // Load and resize to terminal size (80×24 → 160×96 pixels)
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");

    c.bench_function("grayscale_conversion_160x96", |b| {
        b.iter(|| {
            let gray = to_grayscale(black_box(&resized));
            black_box(gray);
        });
    });
}

/// Benchmark Otsu threshold calculation for 160×96 image
fn bench_otsu_threshold(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");
    let gray = to_grayscale(&resized);

    c.bench_function("otsu_threshold_160x96", |b| {
        b.iter(|| {
            let threshold = otsu_threshold(black_box(&gray));
            black_box(threshold);
        });
    });
}

/// Benchmark binary conversion (apply threshold) for 160×96 image
fn bench_apply_threshold(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");
    let gray = to_grayscale(&resized);

    c.bench_function("apply_threshold_160x96", |b| {
        b.iter(|| {
            let binary = apply_threshold(black_box(&gray), 128);
            black_box(binary);
        });
    });
}

/// Benchmark full auto_threshold pipeline (grayscale → otsu → binary)
fn bench_auto_threshold_pipeline(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");

    c.bench_function("auto_threshold_pipeline_160x96", |b| {
        b.iter(|| {
            let binary = auto_threshold(black_box(&resized));
            black_box(binary);
        });
    });
}

/// Benchmark brightness adjustment
fn bench_adjust_brightness(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");
    let gray = to_grayscale(&resized);

    c.bench_function("adjust_brightness_160x96", |b| {
        b.iter(|| {
            let adjusted = adjust_brightness(black_box(&gray), 1.2).unwrap();
            black_box(adjusted);
        });
    });
}

/// Benchmark contrast adjustment
fn bench_adjust_contrast(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");
    let gray = to_grayscale(&resized);

    c.bench_function("adjust_contrast_160x96", |b| {
        b.iter(|| {
            let adjusted = adjust_contrast(black_box(&gray), 1.3).unwrap();
            black_box(adjusted);
        });
    });
}

/// Benchmark gamma correction
fn bench_adjust_gamma(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_terminal(&img, 80, 24).expect("Failed to resize");
    let gray = to_grayscale(&resized);

    c.bench_function("adjust_gamma_160x96", |b| {
        b.iter(|| {
            let adjusted = adjust_gamma(black_box(&gray), 0.9).unwrap();
            black_box(adjusted);
        });
    });
}

/// Benchmark complete pipeline with adjustments (load → resize → grayscale → adjust → threshold)
fn bench_complete_pipeline_with_adjustments(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");

    c.bench_function("complete_pipeline_with_adjustments", |b| {
        b.iter(|| {
            let resized = resize_to_terminal(black_box(&img), 80, 24).unwrap();
            let gray = to_grayscale(&resized);
            let adjusted = adjust_brightness(&gray, 1.1)
                .and_then(|img| adjust_contrast(&img, 1.2))
                .and_then(|img| adjust_gamma(&img, 0.95))
                .unwrap();
            let binary = auto_threshold(&image::DynamicImage::ImageLuma8(adjusted));
            black_box(binary);
        });
    });
}

criterion_group!(
    benches,
    bench_grayscale_conversion,
    bench_otsu_threshold,
    bench_apply_threshold,
    bench_auto_threshold_pipeline,
    bench_adjust_brightness,
    bench_adjust_contrast,
    bench_adjust_gamma,
    bench_complete_pipeline_with_adjustments,
);

criterion_main!(benches);
