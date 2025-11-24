//! Benchmark suite for color scheme application (Story 5.5)
//!
//! Measures performance of applying color schemes to intensity buffers.
//!
//! Run with: `cargo bench --bench color_apply`

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use dotmax::color::apply::{apply_color_scheme, apply_colors_to_grid};
use dotmax::color::schemes::{grayscale, heat_map, rainbow};
use dotmax::{BrailleGrid, Color};

/// Benchmark `apply_color_scheme` with various grid sizes
fn bench_apply_color_scheme(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_color_scheme");

    let sizes = [
        (80, 24, "80x24_standard"),     // Standard terminal
        (120, 30, "120x30_modern"),     // Modern terminal
        (200, 50, "200x50_large"),      // Large terminal
        (320, 80, "320x80_xlarge"),     // Extra large
    ];

    for (width, height, name) in sizes {
        // Create intensity buffer
        let intensities: Vec<Vec<f32>> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| (x as f32 + y as f32) / (width + height) as f32)
                    .collect()
            })
            .collect();

        // Benchmark with heat_map (most common for visualizations)
        let scheme = heat_map();
        group.bench_with_input(
            BenchmarkId::new("heat_map", name),
            &intensities,
            |b, intensities| {
                b.iter(|| apply_color_scheme(black_box(intensities), black_box(&scheme)));
            },
        );

        // Benchmark with rainbow (more complex interpolation)
        let scheme = rainbow();
        group.bench_with_input(
            BenchmarkId::new("rainbow", name),
            &intensities,
            |b, intensities| {
                b.iter(|| apply_color_scheme(black_box(intensities), black_box(&scheme)));
            },
        );

        // Benchmark with grayscale (simplest)
        let scheme = grayscale();
        group.bench_with_input(
            BenchmarkId::new("grayscale", name),
            &intensities,
            |b, intensities| {
                b.iter(|| apply_color_scheme(black_box(intensities), black_box(&scheme)));
            },
        );
    }

    group.finish();
}

/// Benchmark `apply_colors_to_grid`
fn bench_apply_colors_to_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_colors_to_grid");

    let sizes = [
        (80, 24, "80x24_standard"),
        (200, 50, "200x50_large"),
    ];

    for (width, height, name) in sizes {
        // Pre-compute colors
        let colors: Vec<Vec<Color>> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let t = (x as f32 + y as f32) / (width + height) as f32;
                        Color::rgb((t * 255.0) as u8, 128, ((1.0 - t) * 255.0) as u8)
                    })
                    .collect()
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("apply", name), &colors, |b, colors| {
            b.iter_batched(
                || BrailleGrid::new(width, height).unwrap(),
                |mut grid| {
                    apply_colors_to_grid(black_box(&mut grid), black_box(colors)).unwrap();
                    grid
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark `BrailleGrid::apply_color_scheme` convenience method
fn bench_braille_grid_apply_color_scheme(c: &mut Criterion) {
    let mut group = c.benchmark_group("BrailleGrid_apply_color_scheme");

    let sizes = [
        (80, 24, "80x24_standard"),
        (200, 50, "200x50_large"),
    ];

    for (width, height, name) in sizes {
        // Create flattened intensity buffer
        let intensities: Vec<f32> = (0..(width * height))
            .map(|i| i as f32 / ((width * height) as f32 - 1.0))
            .collect();

        let scheme = heat_map();

        group.bench_with_input(
            BenchmarkId::new("heat_map", name),
            &intensities,
            |b, intensities| {
                b.iter_batched(
                    || BrailleGrid::new(width, height).unwrap(),
                    |mut grid| {
                        grid.apply_color_scheme(black_box(intensities), black_box(&scheme))
                            .unwrap();
                        grid
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// End-to-end benchmark: intensity buffer → color scheme → grid
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_colorization_pipeline");

    let sizes = [
        (80, 24, "80x24_standard"),
        (200, 50, "200x50_large"),
    ];

    for (width, height, name) in sizes {
        let intensities_2d: Vec<Vec<f32>> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| (x as f32 + y as f32) / (width + height) as f32)
                    .collect()
            })
            .collect();

        let scheme = heat_map();

        group.bench_with_input(
            BenchmarkId::new("full", name),
            &intensities_2d,
            |b, intensities| {
                b.iter_batched(
                    || BrailleGrid::new(width, height).unwrap(),
                    |mut grid| {
                        // Step 1: Apply color scheme to intensity buffer
                        let colors = apply_color_scheme(black_box(intensities), black_box(&scheme));
                        // Step 2: Apply colors to grid
                        apply_colors_to_grid(black_box(&mut grid), black_box(&colors)).unwrap();
                        grid
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_apply_color_scheme,
    bench_apply_colors_to_grid,
    bench_braille_grid_apply_color_scheme,
    bench_full_pipeline
);
criterion_main!(benches);
