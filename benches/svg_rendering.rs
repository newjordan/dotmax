//! Benchmarks for SVG rasterization and rendering pipeline
//!
//! Run with: `cargo bench --features svg svg_rendering`

#![cfg(all(feature = "svg", feature = "image"))]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dotmax::image::{
    apply_dithering, auto_threshold, load_svg_from_path, pixels_to_braille, to_grayscale,
    DitheringMethod,
};
use std::path::Path;

/// Benchmark small SVG rasterization (<5KB icon)
/// Target: <50ms
fn bench_small_svg_rasterization(c: &mut Criterion) {
    let svg_path = Path::new("tests/fixtures/svg/simple_circle.svg");

    c.bench_function("svg_rasterize_small_icon", |b| {
        b.iter(|| {
            let img = load_svg_from_path(black_box(svg_path), black_box(100), black_box(100))
                .expect("Failed to load SVG");
            black_box(img);
        });
    });
}

/// Benchmark medium SVG rasterization (10-50KB logo)
/// Target: <100ms
fn bench_medium_svg_rasterization(c: &mut Criterion) {
    let svg_path = Path::new("tests/fixtures/svg/logo.svg");

    c.bench_function("svg_rasterize_medium_logo", |b| {
        b.iter(|| {
            let img = load_svg_from_path(black_box(svg_path), black_box(200), black_box(200))
                .expect("Failed to load SVG");
            black_box(img);
        });
    });
}

/// Benchmark SVG with gradient (tests complex rendering)
fn bench_svg_with_gradient(c: &mut Criterion) {
    let svg_path = Path::new("tests/fixtures/svg/gradient.svg");

    c.bench_function("svg_rasterize_gradient", |b| {
        b.iter(|| {
            let img = load_svg_from_path(black_box(svg_path), black_box(256), black_box(256))
                .expect("Failed to load SVG");
            black_box(img);
        });
    });
}

/// Benchmark full SVG → braille pipeline
/// Measures: rasterization + grayscale + dithering + braille mapping
fn bench_svg_full_pipeline_to_braille(c: &mut Criterion) {
    let svg_path = Path::new("tests/fixtures/svg/logo.svg");

    c.bench_function("svg_full_pipeline_floyd_steinberg", |b| {
        b.iter(|| {
            // Load and rasterize SVG
            let img = load_svg_from_path(black_box(svg_path), black_box(160), black_box(96))
                .expect("Failed to load SVG");

            // Convert to grayscale
            let gray = to_grayscale(&img);

            // Apply Floyd-Steinberg dithering
            let binary =
                apply_dithering(&gray, DitheringMethod::FloydSteinberg).expect("Failed to dither");

            // Map to braille
            let grid = pixels_to_braille(&binary, 80, 24).expect("Failed to map to braille");

            black_box(grid);
        });
    });
}

/// Benchmark SVG → threshold pipeline (simpler, faster than dithering)
fn bench_svg_pipeline_with_threshold(c: &mut Criterion) {
    let svg_path = Path::new("tests/fixtures/svg/simple_circle.svg");

    c.bench_function("svg_pipeline_otsu_threshold", |b| {
        b.iter(|| {
            // Load and rasterize SVG
            let img = load_svg_from_path(black_box(svg_path), black_box(160), black_box(96))
                .expect("Failed to load SVG");

            // Auto threshold (includes grayscale conversion)
            let binary = auto_threshold(&img);

            // Map to braille
            let grid = pixels_to_braille(&binary, 80, 24).expect("Failed to map to braille");

            black_box(grid);
        });
    });
}

/// Benchmark SVG text rendering (tests font fallback)
fn bench_svg_text_heavy(c: &mut Criterion) {
    let svg_path = Path::new("tests/fixtures/svg/text_heavy.svg");

    c.bench_function("svg_rasterize_text_heavy", |b| {
        b.iter(|| {
            let img = load_svg_from_path(black_box(svg_path), black_box(300), black_box(100))
                .expect("Failed to load text SVG");
            black_box(img);
        });
    });
}

criterion_group!(
    svg_benches,
    bench_small_svg_rasterization,
    bench_medium_svg_rasterization,
    bench_svg_with_gradient,
    bench_svg_full_pipeline_to_braille,
    bench_svg_pipeline_with_threshold,
    bench_svg_text_heavy
);

criterion_main!(svg_benches);
