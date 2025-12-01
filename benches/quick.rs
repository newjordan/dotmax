//! Benchmarks for the quick module (Story 8.2, AC9)
//!
//! Validates that quick functions add < 5ms overhead vs manual approach.
//!
//! These benchmarks compare:
//! - quick::grid() vs BrailleGrid::new()
//! - quick::load_image() vs manual ImageRenderer pipeline
//!
//! Performance target: < 5ms overhead for convenience functions

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

// ============================================================================
// Grid Creation Overhead (AC9)
// ============================================================================

/// Compare quick::grid() vs BrailleGrid::new() overhead
///
/// Measures the overhead of terminal size detection in quick::grid()
fn bench_grid_overhead(c: &mut Criterion) {
    use dotmax::{quick, BrailleGrid};

    let mut group = c.benchmark_group("quick_overhead");

    // Benchmark direct BrailleGrid::new (baseline)
    group.bench_function("BrailleGrid::new_80x24", |b| {
        b.iter(|| black_box(BrailleGrid::new(80, 24).unwrap()));
    });

    // Benchmark quick::grid() (includes terminal size detection)
    group.bench_function("quick::grid", |b| {
        b.iter(|| black_box(quick::grid().unwrap()));
    });

    // Benchmark quick::grid_sized() (no terminal detection)
    group.bench_function("quick::grid_sized_80x24", |b| {
        b.iter(|| black_box(quick::grid_sized(80, 24).unwrap()));
    });

    group.finish();
}

// ============================================================================
// Image Loading Overhead (AC9) - requires image feature
// ============================================================================

#[cfg(feature = "image")]
mod image_benchmarks {
    use super::*;
    use dotmax::image::ImageRenderer;
    use dotmax::quick;
    use std::path::Path;

    /// Compare quick::load_image() vs manual ImageRenderer pipeline
    ///
    /// This measures the overhead of the convenience function vs manual setup.
    /// Both should produce identical results, but quick::load_image() does
    /// terminal size detection automatically.
    pub fn bench_load_image_overhead(c: &mut Criterion) {
        let mut group = c.benchmark_group("quick_image_overhead");

        // Use test fixture if available, skip otherwise
        let test_image = Path::new("tests/fixtures/images/sample.png");
        if !test_image.exists() {
            println!("Skipping image benchmarks: test image not found");
            return;
        }

        // Benchmark manual ImageRenderer pipeline (baseline)
        group.bench_function("manual_ImageRenderer_80x24", |b| {
            b.iter(|| {
                black_box(
                    ImageRenderer::new()
                        .load_from_path(test_image)
                        .unwrap()
                        .resize(80, 24, true)
                        .unwrap()
                        .render()
                        .unwrap(),
                )
            });
        });

        // Benchmark quick::load_image_sized() (convenience function)
        group.bench_function("quick::load_image_sized_80x24", |b| {
            b.iter(|| black_box(quick::load_image_sized(test_image, 80, 24).unwrap()));
        });

        // Benchmark quick::load_image() (includes terminal detection)
        // Note: Terminal detection adds minimal overhead (~microseconds)
        group.bench_function("quick::load_image_auto", |b| {
            b.iter(|| black_box(quick::load_image(test_image).unwrap()));
        });

        group.finish();
    }
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(quick_benches, bench_grid_overhead);

#[cfg(feature = "image")]
criterion_group!(quick_image_benches, image_benchmarks::bench_load_image_overhead);

#[cfg(feature = "image")]
criterion_main!(quick_benches, quick_image_benches);

#[cfg(not(feature = "image"))]
criterion_main!(quick_benches);
