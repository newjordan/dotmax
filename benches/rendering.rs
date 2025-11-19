use criterion::{criterion_group, criterion_main, Criterion};
use dotmax::BrailleGrid;
use std::hint::black_box;

/// Benchmark: `BrailleGrid` creation (Story 2.1)
///
/// Allocates 80×24 grid (standard terminal size)
fn bench_braille_grid_creation(c: &mut Criterion) {
    c.bench_function("braille_grid_creation", |b| {
        b.iter(|| black_box(BrailleGrid::new(80, 24).unwrap()));
    });
}

/// Benchmark: Grid clearing (Story 2.1)
///
/// Clears all dots in 80×24 grid
fn bench_grid_clear(c: &mut Criterion) {
    c.bench_function("grid_clear", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();
        b.iter(|| {
            grid.clear();
            black_box(&grid);
        });
    });
}

/// Benchmark: Unicode conversion per cell (Story 2.2, AC #6)
///
/// **Target: <1μs per cell**
///
/// Measures `cell_to_braille_char()` for all cells in 80×24 grid (1,920 cells).
/// At <1μs/cell target: 1,920 cells × 1μs = 1.92ms total.
///
/// This benchmark validates AC #6: "Conversion <1μs per cell"
fn bench_unicode_conversion(c: &mut Criterion) {
    c.bench_function("convert_cell_to_braille_char", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        // Set some dots for realistic data (not all zeros)
        for y in 0..24 {
            for x in 0..80 {
                // Set dot 0 in each cell to create non-empty pattern
                grid.set_dot(x * 2, y * 4).unwrap();
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
}

/// Benchmark: Batch grid conversion (Story 2.2)
///
/// Measures `to_unicode_grid()` for entire 80×24 grid
fn bench_to_unicode_grid(c: &mut Criterion) {
    c.bench_function("to_unicode_grid_80x24", |b| {
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        // Set some dots for realistic data
        for y in 0..24 {
            for x in 0..80 {
                grid.set_dot(x * 2, y * 4).unwrap();
            }
        }

        b.iter(|| {
            black_box(grid.to_unicode_grid());
        });
    });
}

criterion_group!(
    benches,
    bench_braille_grid_creation,
    bench_grid_clear,
    bench_unicode_conversion,
    bench_to_unicode_grid
);
criterion_main!(benches);
