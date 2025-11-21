//! Benchmarks for extreme aspect ratio and large image pipeline performance
//!
//! This benchmark suite profiles the complete image-to-braille pipeline with:
//! - Extreme wide images (10000×4000 - viper_ultra_wide.png)
//! - Extreme tall images (4000×10000 - viper_ultra_tall.png)
//! - Very large square images (4000×4000 - viper_4k.png)
//!
//! Goal: Identify bottlenecks causing 20+ second load times (Story 3.5.5)
//! Target: <5 seconds for extreme images after optimization
//!
//! Run with:
//! ```sh
//! cargo bench --features image --bench extreme_image_pipeline
//! ```

#![cfg(feature = "image")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dotmax::image::{
    apply_dithering, auto_threshold, load_from_path, pixels_to_braille, resize_to_terminal,
    to_grayscale, DitheringMethod,
};
use std::hint::black_box;
use std::path::Path;
use std::time::Duration;

/// Benchmark loading extreme wide image (10000×4000)
fn bench_load_extreme_wide(c: &mut Criterion) {
    let path = Path::new("tests/fixtures/images/viper_ultra_wide.png");

    c.bench_function("load_extreme_wide_10000x4000", |b| {
        b.iter(|| {
            let img = load_from_path(black_box(path)).unwrap();
            black_box(img);
        });
    });
}

/// Benchmark loading extreme tall image (4000×10000)
fn bench_load_extreme_tall(c: &mut Criterion) {
    let path = Path::new("tests/fixtures/images/viper_ultra_tall.png");

    c.bench_function("load_extreme_tall_4000x10000", |b| {
        b.iter(|| {
            let img = load_from_path(black_box(path)).unwrap();
            black_box(img);
        });
    });
}

/// Benchmark loading very large square image (4000×4000)
fn bench_load_large_square(c: &mut Criterion) {
    let path = Path::new("tests/fixtures/images/viper_4k.png");

    c.bench_function("load_large_square_4000x4000", |b| {
        b.iter(|| {
            let img = load_from_path(black_box(path)).unwrap();
            black_box(img);
        });
    });
}

/// Benchmark resize stage for extreme wide image (EXPECTED BOTTLENECK)
fn bench_resize_extreme_wide(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");

    c.bench_function("resize_extreme_wide_10000x4000_to_terminal", |b| {
        b.iter(|| {
            let resized = resize_to_terminal(black_box(&img), 80, 24).unwrap();
            black_box(resized);
        });
    });
}

/// Benchmark resize stage for extreme tall image (EXPECTED BOTTLENECK)
fn bench_resize_extreme_tall(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_tall.png"))
        .expect("Failed to load viper_ultra_tall.png");

    c.bench_function("resize_extreme_tall_4000x10000_to_terminal", |b| {
        b.iter(|| {
            let resized = resize_to_terminal(black_box(&img), 80, 24).unwrap();
            black_box(resized);
        });
    });
}

/// Benchmark resize stage for large square image
fn bench_resize_large_square(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_4k.png"))
        .expect("Failed to load viper_4k.png");

    c.bench_function("resize_large_square_4000x4000_to_terminal", |b| {
        b.iter(|| {
            let resized = resize_to_terminal(black_box(&img), 80, 24).unwrap();
            black_box(resized);
        });
    });
}

/// Benchmark grayscale conversion stage for extreme wide
fn bench_grayscale_extreme_wide(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");
    let resized = resize_to_terminal(&img, 80, 24).unwrap();

    c.bench_function("grayscale_extreme_wide_after_resize", |b| {
        b.iter(|| {
            let gray = to_grayscale(black_box(&resized));
            black_box(gray);
        });
    });
}

/// Benchmark dithering stage for extreme wide (Floyd-Steinberg)
fn bench_dither_extreme_wide(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");
    let resized = resize_to_terminal(&img, 80, 24).unwrap();
    let gray = to_grayscale(&resized);

    c.bench_function("dither_floyd_steinberg_extreme_wide", |b| {
        b.iter(|| {
            let binary =
                apply_dithering(black_box(&gray), DitheringMethod::FloydSteinberg).unwrap();
            black_box(binary);
        });
    });
}

/// Benchmark threshold stage for extreme wide (Otsu)
fn bench_threshold_extreme_wide(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");
    let resized = resize_to_terminal(&img, 80, 24).unwrap();

    c.bench_function("threshold_otsu_extreme_wide", |b| {
        b.iter(|| {
            let binary = auto_threshold(black_box(&resized));
            black_box(binary);
        });
    });
}

/// Benchmark braille mapping stage for extreme wide
fn bench_map_to_braille_extreme_wide(c: &mut Criterion) {
    let img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");
    let resized = resize_to_terminal(&img, 80, 24).unwrap();
    let binary = auto_threshold(&resized);

    c.bench_function("map_to_braille_extreme_wide", |b| {
        b.iter(|| {
            let grid = pixels_to_braille(black_box(&binary), 80, 24).unwrap();
            black_box(grid);
        });
    });
}

/// Benchmark FULL pipeline for extreme wide image (load → resize → convert → threshold → map)
fn bench_full_pipeline_extreme_wide(c: &mut Criterion) {
    let path = Path::new("tests/fixtures/images/viper_ultra_wide.png");

    let mut group = c.benchmark_group("full_pipeline");
    group.measurement_time(Duration::from_secs(30)); // Allow up to 30s measurements
    group.sample_size(10); // Fewer samples for slow benchmarks

    group.bench_function("extreme_wide_10000x4000_full_pipeline", |b| {
        b.iter(|| {
            let img = load_from_path(black_box(path)).unwrap();
            let resized = resize_to_terminal(&img, 80, 24).unwrap();
            let binary = auto_threshold(&resized);
            let grid = pixels_to_braille(&binary, 80, 24).unwrap();
            black_box(grid);
        });
    });

    group.finish();
}

/// Benchmark FULL pipeline for extreme tall image
fn bench_full_pipeline_extreme_tall(c: &mut Criterion) {
    let path = Path::new("tests/fixtures/images/viper_ultra_tall.png");

    let mut group = c.benchmark_group("full_pipeline");
    group.measurement_time(Duration::from_secs(30)); // Allow up to 30s measurements
    group.sample_size(10); // Fewer samples for slow benchmarks

    group.bench_function("extreme_tall_4000x10000_full_pipeline", |b| {
        b.iter(|| {
            let img = load_from_path(black_box(path)).unwrap();
            let resized = resize_to_terminal(&img, 80, 24).unwrap();
            let binary = auto_threshold(&resized);
            let grid = pixels_to_braille(&binary, 80, 24).unwrap();
            black_box(grid);
        });
    });

    group.finish();
}

/// Benchmark FULL pipeline for large square image
fn bench_full_pipeline_large_square(c: &mut Criterion) {
    let path = Path::new("tests/fixtures/images/viper_4k.png");

    let mut group = c.benchmark_group("full_pipeline");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    group.bench_function("large_square_4000x4000_full_pipeline", |b| {
        b.iter(|| {
            let img = load_from_path(black_box(path)).unwrap();
            let resized = resize_to_terminal(&img, 80, 24).unwrap();
            let binary = auto_threshold(&resized);
            let grid = pixels_to_braille(&binary, 80, 24).unwrap();
            black_box(grid);
        });
    });

    group.finish();
}

/// Compare normal image vs extreme image resize performance
fn bench_resize_comparison(c: &mut Criterion) {
    let normal_img = load_from_path(Path::new("tests/fixtures/images/sample.png"))
        .expect("Failed to load sample.png");
    let extreme_img = load_from_path(Path::new("tests/fixtures/images/viper_ultra_wide.png"))
        .expect("Failed to load viper_ultra_wide.png");

    let mut group = c.benchmark_group("resize_comparison");

    group.bench_with_input(
        BenchmarkId::new("resize", "normal_800x600"),
        &normal_img,
        |b, img| {
            b.iter(|| {
                let resized = resize_to_terminal(black_box(img), 80, 24).unwrap();
                black_box(resized);
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("resize", "extreme_10000x4000"),
        &extreme_img,
        |b, img| {
            b.iter(|| {
                let resized = resize_to_terminal(black_box(img), 80, 24).unwrap();
                black_box(resized);
            });
        },
    );

    group.finish();
}

criterion_group!(
    load_benches,
    bench_load_extreme_wide,
    bench_load_extreme_tall,
    bench_load_large_square,
);

criterion_group!(
    resize_benches,
    bench_resize_extreme_wide,
    bench_resize_extreme_tall,
    bench_resize_large_square,
    bench_resize_comparison,
);

criterion_group!(
    stage_benches,
    bench_grayscale_extreme_wide,
    bench_dither_extreme_wide,
    bench_threshold_extreme_wide,
    bench_map_to_braille_extreme_wide,
);

criterion_group!(
    full_pipeline_benches,
    bench_full_pipeline_extreme_wide,
    bench_full_pipeline_extreme_tall,
    bench_full_pipeline_large_square,
);

criterion_main!(
    load_benches,
    resize_benches,
    stage_benches,
    full_pipeline_benches
);
