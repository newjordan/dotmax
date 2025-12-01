//! Core rendering benchmarks for Story 7.2 (AC1)
//!
//! Benchmark groups:
//! - `grid_creation` - BrailleGrid::new() for various sizes
//! - `dot_operations` - set_dot/clear operations
//! - `unicode_conversion` - to_char() conversion performance
//!
//! Performance targets (from architecture.md):
//! - Grid creation: <1ms
//! - Unicode conversion: <5ms for full 80x24 grid

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dotmax::BrailleGrid;
use std::hint::black_box;

// ============================================================================
// Grid Creation Benchmarks (AC1: grid_creation group)
// ============================================================================

/// Benchmark grid creation for various terminal sizes
///
/// Tests sizes: 40x12 (small), 80x24 (standard), 160x48 (large), 200x50 (max)
fn bench_grid_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_creation");

    // (width, height, label)
    let sizes = [
        (40, 12, "40x12_small"),
        (80, 24, "80x24_standard"),
        (160, 48, "160x48_large"),
        (200, 50, "200x50_max"),
    ];

    for (width, height, label) in sizes {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("new", label), &(width, height), |b, &(w, h)| {
            b.iter(|| black_box(BrailleGrid::new(w, h).unwrap()));
        });
    }

    group.finish();
}

/// Benchmark grid clear operation for various sizes
fn bench_grid_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_clear");

    let sizes = [
        (40, 12, "40x12_small"),
        (80, 24, "80x24_standard"),
        (160, 48, "160x48_large"),
        (200, 50, "200x50_max"),
    ];

    for (width, height, label) in sizes {
        let mut grid = BrailleGrid::new(width, height).unwrap();
        // Pre-populate with some dots
        for y in 0..(height * 4).min(100) {
            for x in 0..(width * 2).min(100) {
                let _ = grid.set_dot(x, y);
            }
        }

        group.bench_with_input(BenchmarkId::new("clear", label), &(), |b, _| {
            b.iter(|| {
                grid.clear();
                black_box(&grid);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Dot Operations Benchmarks (AC1: dot_operations group)
// ============================================================================

/// Benchmark set_dot operations at various scales
///
/// Tests: 1000 ops, 10000 ops to measure throughput
fn bench_dot_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_operations");

    // Benchmark 1000 set_dot operations
    group.throughput(Throughput::Elements(1000));
    group.bench_function("set_dot_1000_ops", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();
        b.iter(|| {
            for i in 0..1000 {
                let x = (i * 7) % 160; // 80 cells * 2 dots = 160 dot columns
                let y = (i * 13) % 96; // 24 cells * 4 dots = 96 dot rows
                let _ = grid.set_dot(x, y);
            }
            black_box(&grid);
        });
    });

    // Benchmark 10000 set_dot operations
    group.throughput(Throughput::Elements(10000));
    group.bench_function("set_dot_10000_ops", |b| {
        let mut grid = BrailleGrid::new(200, 50).unwrap(); // Larger grid for 10k ops
        b.iter(|| {
            for i in 0..10000 {
                let x = (i * 7) % 400; // 200 cells * 2 dots
                let y = (i * 13) % 200; // 50 cells * 4 dots
                let _ = grid.set_dot(x, y);
            }
            black_box(&grid);
        });
    });

    // Benchmark mixed set/get operations
    group.throughput(Throughput::Elements(1000));
    group.bench_function("mixed_set_get_1000_ops", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            for i in 0..500 {
                let x = (i * 7) % 160;
                let y = (i * 13) % 96;
                let _ = grid.set_dot(x, y);
            }
            // Read back (simulating rendering)
            for i in 0..500 {
                let cell_x = (i * 7) % 80;
                let cell_y = (i * 13) % 24;
                black_box(grid.cell_to_braille_char(cell_x, cell_y).unwrap());
            }
        });
    });

    group.finish();
}

// ============================================================================
// Unicode Conversion Benchmarks (AC1: unicode_conversion group)
// ============================================================================

/// Benchmark unicode conversion performance
///
/// Target: <1μs per cell (1920 cells at <1μs = <2ms total)
fn bench_unicode_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode_conversion");

    // Single cell conversion
    group.bench_function("cell_to_braille_char_single", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();
        // Set some dots for realistic data
        let _ = grid.set_dot(0, 0);
        let _ = grid.set_dot(1, 1);
        let _ = grid.set_dot(0, 2);

        b.iter(|| black_box(grid.cell_to_braille_char(0, 0).unwrap()));
    });

    // Full grid conversion (80x24 = 1920 cells)
    group.throughput(Throughput::Elements(1920)); // 80 * 24 cells
    group.bench_function("to_char_80x24_full_grid", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();
        // Set dots for realistic non-empty pattern
        for y in 0..24 {
            for x in 0..80 {
                let _ = grid.set_dot(x * 2, y * 4);
            }
        }

        b.iter(|| {
            for y in 0..24 {
                for x in 0..80 {
                    black_box(grid.cell_to_braille_char(x, y).unwrap());
                }
            }
        });
    });

    // to_unicode_grid batch conversion
    group.bench_function("to_unicode_grid_80x24", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();
        for y in 0..24 {
            for x in 0..80 {
                let _ = grid.set_dot(x * 2, y * 4);
            }
        }

        b.iter(|| black_box(grid.to_unicode_grid()));
    });

    // Large grid (200x50 = 10000 cells)
    group.throughput(Throughput::Elements(10000)); // 200 * 50 cells
    group.bench_function("to_unicode_grid_200x50", |b| {
        let mut grid = BrailleGrid::new(200, 50).unwrap();
        for y in 0..50 {
            for x in 0..200 {
                let _ = grid.set_dot(x * 2, y * 4);
            }
        }

        b.iter(|| black_box(grid.to_unicode_grid()));
    });

    group.finish();
}

/// Benchmark combined operations (realistic usage pattern)
fn bench_combined_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_operations");

    // Typical frame: create, draw, convert
    group.bench_function("create_draw_convert_80x24", |b| {
        b.iter(|| {
            let mut grid = BrailleGrid::new(80, 24).unwrap();

            // Draw a simple pattern (simulating a small shape)
            for dy in 0..8 {
                for dx in 0..4 {
                    let _ = grid.set_dot(80 + dx, 48 + dy);
                }
            }

            // Convert to unicode
            let chars = grid.to_unicode_grid();
            black_box(chars);
        });
    });

    // Frame update: clear, redraw, convert
    group.bench_function("clear_redraw_convert_80x24", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            // Clear previous frame
            grid.clear();

            // Draw new pattern (simulating animation frame)
            for dy in 0..8 {
                for dx in 0..4 {
                    let _ = grid.set_dot(82 + dx, 50 + dy); // Slightly moved
                }
            }

            // Convert to unicode
            let chars = grid.to_unicode_grid();
            black_box(chars);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_grid_creation,
    bench_grid_clear,
    bench_dot_operations,
    bench_unicode_conversion,
    bench_combined_operations
);
criterion_main!(benches);
