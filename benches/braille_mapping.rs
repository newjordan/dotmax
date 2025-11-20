//! Benchmarks for braille mapping operations (Story 3.5)
//!
//! These benchmarks measure the performance of the pixels-to-braille mapping stage,
//! which converts binary pixel data to BrailleGrid using 2×4 pixel blocks.
//!
//! Performance targets:
//! - 160×96 pixels (80×24 cells): <10ms
//! - 400×200 pixels (200×50 cells): <25ms

#![cfg(feature = "image")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dotmax::image::{
    auto_threshold, load_from_path, pixels_to_braille, resize_to_dimensions, to_grayscale,
};
use std::path::Path;

/// Benchmark braille mapping for standard terminal size (160×96 pixels = 80×24 cells)
fn bench_pixels_to_braille_standard(c: &mut Criterion) {
    // Prepare binary image at terminal size
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 160, 96, true);
    let gray = to_grayscale(&resized);
    let binary = auto_threshold(&gray);

    c.bench_function("pixels_to_braille_160x96", |b| {
        b.iter(|| {
            let grid = pixels_to_braille(black_box(&binary), 80, 24).unwrap();
            black_box(grid);
        });
    });
}

/// Benchmark braille mapping for large terminal (400×200 pixels = 200×50 cells)
fn bench_pixels_to_braille_large(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 400, 200, true);
    let gray = to_grayscale(&resized);
    let binary = auto_threshold(&gray);

    c.bench_function("pixels_to_braille_400x200", |b| {
        b.iter(|| {
            let grid = pixels_to_braille(black_box(&binary), 200, 50).unwrap();
            black_box(grid);
        });
    });
}

/// Benchmark braille mapping for small image (40×24 pixels = 20×6 cells)
fn bench_pixels_to_braille_small(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 40, 24, true);
    let gray = to_grayscale(&resized);
    let binary = auto_threshold(&gray);

    c.bench_function("pixels_to_braille_40x24", |b| {
        b.iter(|| {
            let grid = pixels_to_braille(black_box(&binary), 20, 6).unwrap();
            black_box(grid);
        });
    });
}

/// Benchmark full pipeline: threshold → braille mapping (standard size)
fn bench_threshold_to_braille_pipeline(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 160, 96, true);
    let gray = to_grayscale(&resized);

    c.bench_function("threshold_to_braille_pipeline_160x96", |b| {
        b.iter(|| {
            let binary = auto_threshold(black_box(&gray));
            let grid = pixels_to_braille(&binary, 80, 24).unwrap();
            black_box(grid);
        });
    });
}

/// Benchmark full end-to-end pipeline (load → resize → grayscale → threshold → braille)
fn bench_full_image_to_braille_pipeline(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");

    c.bench_function("full_image_to_braille_pipeline", |b| {
        b.iter(|| {
            let resized = resize_to_dimensions(black_box(&img), 160, 96, true);
            let gray = to_grayscale(&resized);
            let binary = auto_threshold(&gray);
            let grid = pixels_to_braille(&binary, 80, 24).unwrap();
            black_box(grid);
        });
    });
}

criterion_group!(
    benches,
    bench_pixels_to_braille_standard,
    bench_pixels_to_braille_large,
    bench_pixels_to_braille_small,
    bench_threshold_to_braille_pipeline,
    bench_full_image_to_braille_pipeline,
);

criterion_main!(benches);
