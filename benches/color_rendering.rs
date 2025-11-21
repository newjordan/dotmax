//! Benchmarks for color mode image rendering
//!
//! These benchmarks measure the performance overhead of color extraction and rendering:
//! - Monochrome mode (baseline, no color overhead)
//! - Grayscale mode (RGB → intensity mapping)
//! - TrueColor mode (full RGB preservation)
//! - Color extraction strategies (average, dominant, center pixel)
//!
//! Performance targets (for 80×24 terminal = 160×96 pixels):
//! - Monochrome mode: <45ms (existing pipeline)
//! - Grayscale mode: <48ms (+3ms color extraction)
//! - TrueColor mode: <50ms (+5ms color extraction)

#![cfg(feature = "image")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dotmax::image::color_mode::extract_cell_colors;
use dotmax::image::{
    load_from_path, render_image_with_color, resize_to_dimensions, ColorMode, ColorSamplingStrategy, DitheringMethod,
};
use std::path::Path;

/// Benchmark monochrome mode (baseline, no color overhead)
fn bench_render_monochrome(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");

    c.bench_function("render_image_monochrome_80x24", |b| {
        b.iter(|| {
            let grid = render_image_with_color(
                black_box(&img),
                ColorMode::Monochrome,
                80,
                24,
                DitheringMethod::FloydSteinberg,
                None,
                1.0,
                1.0,
                1.0,
            )
            .expect("Failed to render");
            black_box(grid);
        });
    });
}

/// Benchmark grayscale mode (RGB → intensity + ANSI 256-color)
fn bench_render_grayscale(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");

    c.bench_function("render_image_grayscale_80x24", |b| {
        b.iter(|| {
            let grid = render_image_with_color(
                black_box(&img),
                ColorMode::Grayscale,
                80,
                24,
                DitheringMethod::FloydSteinberg,
                None,
                1.0,
                1.0,
                1.0,
            )
            .expect("Failed to render");
            black_box(grid);
        });
    });
}

/// Benchmark TrueColor mode (full RGB preservation)
fn bench_render_truecolor(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");

    c.bench_function("render_image_truecolor_80x24", |b| {
        b.iter(|| {
            let grid = render_image_with_color(
                black_box(&img),
                ColorMode::TrueColor,
                80,
                24,
                DitheringMethod::FloydSteinberg,
                None,
                1.0,
                1.0,
                1.0,
            )
            .expect("Failed to render");
            black_box(grid);
        });
    });
}

/// Benchmark color extraction with average strategy
fn bench_color_extraction_average(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 160, 96, true).expect("Failed to resize");

    // Calculate actual cell dimensions after resize (aspect ratio preserved)
    let cell_width = (resized.width() as usize + 1) / 2;
    let cell_height = (resized.height() as usize + 3) / 4;

    c.bench_function("color_extraction_average_80x24", |b| {
        b.iter(|| {
            let colors = extract_cell_colors(
                black_box(&resized),
                cell_width,
                cell_height,
                ColorSamplingStrategy::Average,
            );
            black_box(colors);
        });
    });
}

/// Benchmark color extraction with dominant strategy
fn bench_color_extraction_dominant(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 160, 96, true).expect("Failed to resize");

    let cell_width = (resized.width() as usize + 1) / 2;
    let cell_height = (resized.height() as usize + 3) / 4;

    c.bench_function("color_extraction_dominant_80x24", |b| {
        b.iter(|| {
            let colors = extract_cell_colors(
                black_box(&resized),
                cell_width,
                cell_height,
                ColorSamplingStrategy::Dominant,
            );
            black_box(colors);
        });
    });
}

/// Benchmark color extraction with center pixel strategy
fn bench_color_extraction_center(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");
    let resized = resize_to_dimensions(&img, 160, 96, true).expect("Failed to resize");

    let cell_width = (resized.width() as usize + 1) / 2;
    let cell_height = (resized.height() as usize + 3) / 4;

    c.bench_function("color_extraction_center_80x24", |b| {
        b.iter(|| {
            let colors = extract_cell_colors(
                black_box(&resized),
                cell_width,
                cell_height,
                ColorSamplingStrategy::CenterPixel,
            );
            black_box(colors);
        });
    });
}

/// Benchmark large terminal (200×50 cells = 400×200 pixels)
fn bench_render_large_terminal(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample image");

    // Resize to large terminal size for stress test
    let resized = resize_to_dimensions(&img, 400, 200, true).expect("Failed to resize");

    c.bench_function("render_image_truecolor_200x50", |b| {
        b.iter(|| {
            let grid = render_image_with_color(
                black_box(&resized),
                ColorMode::TrueColor,
                200,
                50,
                DitheringMethod::FloydSteinberg,
                None,
                1.0,
                1.0,
                1.0,
            )
            .expect("Failed to render");
            black_box(grid);
        });
    });
}

criterion_group!(
    color_mode_benches,
    bench_render_monochrome,
    bench_render_grayscale,
    bench_render_truecolor,
    bench_color_extraction_average,
    bench_color_extraction_dominant,
    bench_color_extraction_center,
    bench_render_large_terminal
);

criterion_main!(color_mode_benches);
