//! Benchmark suite for dithering algorithms
//!
//! Measures performance of Floyd-Steinberg, Bayer, and Atkinson dithering
//! algorithms against target performance budgets:
//! - Floyd-Steinberg: <15ms for 160×96 images
//! - Bayer: <10ms for 160×96 images
//! - Atkinson: <12ms for 160×96 images

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dotmax::image::{apply_dithering, to_grayscale, DitheringMethod};
use image::{DynamicImage, GrayImage, Luma};

/// Helper: Create a grayscale gradient image for consistent benchmarking
fn create_gradient_image(width: u32, height: u32) -> GrayImage {
    GrayImage::from_fn(width, height, |x, y| {
        let value = ((x + y) as f32 / (width + height) as f32 * 255.0) as u8;
        Luma([value])
    })
}

fn benchmark_floyd_steinberg(c: &mut Criterion) {
    let mut group = c.benchmark_group("floyd_steinberg");

    // Standard terminal size (160×96 pixels = 80×24 cells × 2×4 dots)
    group.bench_function("160x96_standard_terminal", |b| {
        let gray = create_gradient_image(160, 96);
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::FloydSteinberg)
                .expect("Floyd-Steinberg failed")
        });
    });

    // Small image (50×50)
    group.bench_function("50x50_small", |b| {
        let gray = create_gradient_image(50, 50);
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::FloydSteinberg)
                .expect("Floyd-Steinberg failed")
        });
    });

    // Large terminal (320×192 pixels = 160×48 cells)
    group.bench_function("320x192_large_terminal", |b| {
        let gray = create_gradient_image(320, 192);
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::FloydSteinberg)
                .expect("Floyd-Steinberg failed")
        });
    });

    group.finish();
}

fn benchmark_bayer(c: &mut Criterion) {
    let mut group = c.benchmark_group("bayer");

    // Standard terminal size (160×96 pixels)
    group.bench_function("160x96_standard_terminal", |b| {
        let gray = create_gradient_image(160, 96);
        b.iter(|| apply_dithering(black_box(&gray), DitheringMethod::Bayer).expect("Bayer failed"));
    });

    // Small image (50×50)
    group.bench_function("50x50_small", |b| {
        let gray = create_gradient_image(50, 50);
        b.iter(|| apply_dithering(black_box(&gray), DitheringMethod::Bayer).expect("Bayer failed"));
    });

    // Large terminal (320×192 pixels)
    group.bench_function("320x192_large_terminal", |b| {
        let gray = create_gradient_image(320, 192);
        b.iter(|| apply_dithering(black_box(&gray), DitheringMethod::Bayer).expect("Bayer failed"));
    });

    group.finish();
}

fn benchmark_atkinson(c: &mut Criterion) {
    let mut group = c.benchmark_group("atkinson");

    // Standard terminal size (160×96 pixels)
    group.bench_function("160x96_standard_terminal", |b| {
        let gray = create_gradient_image(160, 96);
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Atkinson).expect("Atkinson failed")
        });
    });

    // Small image (50×50)
    group.bench_function("50x50_small", |b| {
        let gray = create_gradient_image(50, 50);
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Atkinson).expect("Atkinson failed")
        });
    });

    // Large terminal (320×192 pixels)
    group.bench_function("320x192_large_terminal", |b| {
        let gray = create_gradient_image(320, 192);
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Atkinson).expect("Atkinson failed")
        });
    });

    group.finish();
}

fn benchmark_apply_dithering_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_dithering");

    let gray = create_gradient_image(160, 96);

    group.bench_function("floyd_steinberg_via_dispatch", |b| {
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::FloydSteinberg)
                .expect("Dithering failed")
        });
    });

    group.bench_function("bayer_via_dispatch", |b| {
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Bayer).expect("Dithering failed")
        });
    });

    group.bench_function("atkinson_via_dispatch", |b| {
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Atkinson).expect("Dithering failed")
        });
    });

    group.finish();
}

fn benchmark_all_algorithms_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithm_comparison");

    let gray = create_gradient_image(160, 96);

    group.bench_function("floyd_steinberg", |b| {
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::FloydSteinberg)
                .expect("Dithering failed")
        });
    });

    group.bench_function("bayer", |b| {
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Bayer).expect("Dithering failed")
        });
    });

    group.bench_function("atkinson", |b| {
        b.iter(|| {
            apply_dithering(black_box(&gray), DitheringMethod::Atkinson).expect("Dithering failed")
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_floyd_steinberg,
    benchmark_bayer,
    benchmark_atkinson,
    benchmark_apply_dithering_dispatch,
    benchmark_all_algorithms_comparison
);
criterion_main!(benches);
