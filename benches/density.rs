//! Density rendering performance benchmarks
//!
//! Measures the performance of density set mapping and grid rendering operations.
//!
//! Run with: `cargo bench --bench density`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dotmax::density::DensitySet;
use dotmax::BrailleGrid;

/// Benchmark single intensity mapping for all predefined density sets
fn bench_density_mapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("density_mapping");

    let density_sets = vec![
        ("ASCII", DensitySet::ascii()),
        ("Simple", DensitySet::simple()),
        ("Blocks", DensitySet::blocks()),
        ("Braille", DensitySet::braille()),
    ];

    for (name, density) in density_sets {
        group.bench_with_input(BenchmarkId::new("single_map", name), &density, |b, d| {
            b.iter(|| {
                black_box(d.map(black_box(0.5)));
            });
        });
    }

    group.finish();
}

/// Benchmark mapping multiple intensity values
fn bench_density_batch_mapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("density_batch_mapping");

    // Test different batch sizes
    let batch_sizes = vec![10, 100, 1_000, 10_000];

    for size in batch_sizes {
        let intensities: Vec<f32> = (0..size).map(|i| i as f32 / (size - 1) as f32).collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &intensities,
            |b, ints| {
                let density = DensitySet::ascii();
                b.iter(|| {
                    for &intensity in ints {
                        black_box(density.map(black_box(intensity)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full grid density rendering for various grid sizes
fn bench_grid_density_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_density_rendering");

    // Test different grid sizes
    let grid_sizes = vec![
        ("small", 10, 10),    // 100 cells
        ("medium", 40, 24),   // 960 cells
        ("terminal", 80, 24), // 1920 cells (typical terminal)
        ("large", 160, 48),   // 7680 cells (double terminal)
    ];

    for (name, width, height) in grid_sizes {
        let size = width * height;
        let gradient: Vec<f32> = (0..size).map(|i| i as f32 / (size - 1) as f32).collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("render", name),
            &(width, height, gradient),
            |b, (w, h, grad)| {
                let mut grid = BrailleGrid::new(*w, *h).unwrap();
                let density = DensitySet::ascii();
                b.iter(|| {
                    black_box(
                        grid.render_density(black_box(grad), black_box(&density))
                            .unwrap(),
                    );
                });
            },
        );
    }

    group.finish();
}

/// Benchmark density rendering with different density sets
fn bench_density_sets_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("density_sets_comparison");

    // Use typical terminal size for comparison
    let width = 80;
    let height = 24;
    let size = width * height;
    let gradient: Vec<f32> = (0..size).map(|i| i as f32 / (size - 1) as f32).collect();

    let density_sets = vec![
        ("ASCII_69", DensitySet::ascii()),
        ("Simple_10", DensitySet::simple()),
        ("Blocks_5", DensitySet::blocks()),
        ("Braille_9", DensitySet::braille()),
    ];

    group.throughput(Throughput::Elements(size as u64));

    for (name, density) in density_sets {
        group.bench_with_input(BenchmarkId::from_parameter(name), &density, |b, d| {
            let mut grid = BrailleGrid::new(width, height).unwrap();
            b.iter(|| {
                black_box(
                    grid.render_density(black_box(&gradient), black_box(d))
                        .unwrap(),
                );
            });
        });
    }

    group.finish();
}

/// Benchmark density rendering with various gradient patterns
fn bench_gradient_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("gradient_patterns");

    let width = 80;
    let height = 24;
    let size = width * height;

    // Generate different gradient patterns
    let gradients = vec![
        (
            "horizontal",
            (0..size)
                .map(|i| (i % width) as f32 / (width - 1) as f32)
                .collect::<Vec<f32>>(),
        ),
        (
            "vertical",
            (0..size)
                .map(|i| (i / width) as f32 / (height - 1) as f32)
                .collect::<Vec<f32>>(),
        ),
        (
            "linear",
            (0..size)
                .map(|i| i as f32 / (size - 1) as f32)
                .collect::<Vec<f32>>(),
        ),
    ];

    group.throughput(Throughput::Elements(size as u64));

    for (name, gradient) in gradients {
        group.bench_with_input(BenchmarkId::from_parameter(name), &gradient, |b, grad| {
            let mut grid = BrailleGrid::new(width, height).unwrap();
            let density = DensitySet::ascii();
            b.iter(|| {
                black_box(
                    grid.render_density(black_box(grad), black_box(&density))
                        .unwrap(),
                );
            });
        });
    }

    group.finish();
}

/// Benchmark custom density set creation
fn bench_custom_density_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_density_creation");

    // Test different density set sizes
    let sizes = vec![5, 10, 50, 100, 256];

    for size in sizes {
        let chars: Vec<char> = (0..size).map(|i| (i as u8 as char)).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &chars,
            |b, characters| {
                b.iter(|| {
                    black_box(
                        DensitySet::new(
                            black_box(format!("Custom{}", characters.len())),
                            black_box(characters.clone()),
                        )
                        .unwrap(),
                    );
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_density_mapping,
    bench_density_batch_mapping,
    bench_grid_density_rendering,
    bench_density_sets_comparison,
    bench_gradient_patterns,
    bench_custom_density_creation,
);
criterion_main!(benches);
