//! Benchmarks for drawing primitives (lines, circles, shapes).
//!
//! Performance targets (from Story 4.1 AC5 and Story 4.2 AC6):
//! - 1000-pixel line: <1ms
//! - Thick line (thickness=5, 1000px): <5ms
//! - Circle outline (radius 100): <2ms
//! - Filled circle (radius 100): <10ms
//! - Thick circle (thickness 5, radius 100): <10ms
//!
//! Run with: `cargo bench primitives`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dotmax::{
    primitives::{
        draw_circle, draw_circle_filled, draw_circle_thick, draw_line, draw_line_thick,
        draw_polygon, draw_polygon_filled, draw_rectangle, draw_rectangle_filled,
        draw_rectangle_thick,
    },
    BrailleGrid,
};
use std::hint::black_box;

/// Benchmark draw_line for various line lengths
fn bench_line_drawing(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_line");

    // Test different line lengths: small (10px), medium (100px), large (1000px)
    for length in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(length), length, |b, &length| {
            // Grid large enough to hold the line
            let mut grid =
                BrailleGrid::new(((length / 2) + 10) as usize, ((length / 4) + 10) as usize)
                    .unwrap();

            b.iter(|| {
                // Draw diagonal line of given length
                draw_line(
                    black_box(&mut grid),
                    black_box(0),
                    black_box(0),
                    black_box(length),
                    black_box(length),
                )
                .unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark draw_line for different octants (angles)
fn bench_line_octants(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_line_octants");

    // Grid: 100×100 cells = 200×400 dots
    let mut grid = BrailleGrid::new(100, 100).unwrap();

    let test_cases = [
        ("horizontal", 0, 100, 199, 100),
        ("vertical", 100, 0, 100, 399),
        ("diagonal_45deg", 0, 0, 199, 199),
        ("steep_positive", 100, 0, 120, 199),
        ("shallow_positive", 0, 100, 199, 120),
    ];

    for (name, x0, y0, x1, y1) in test_cases.iter() {
        group.bench_function(*name, |b| {
            b.iter(|| {
                draw_line(
                    black_box(&mut grid),
                    black_box(*x0),
                    black_box(*y0),
                    black_box(*x1),
                    black_box(*y1),
                )
                .unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark draw_line_thick for various thicknesses
fn bench_thick_line_drawing(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_line_thick");

    // Grid: 100×100 cells = 200×400 dots
    let mut grid = BrailleGrid::new(100, 100).unwrap();

    // Test different thicknesses on 1000px line
    for thickness in [1, 3, 5, 7, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("thickness_{}", thickness)),
            thickness,
            |b, &thickness| {
                b.iter(|| {
                    draw_line_thick(
                        black_box(&mut grid),
                        black_box(0),
                        black_box(0),
                        black_box(199),
                        black_box(199),
                        black_box(thickness),
                    )
                    .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark realistic use case: drawing multiple lines (grid pattern)
fn bench_grid_pattern(c: &mut Criterion) {
    c.bench_function("grid_pattern_10x10", |b| {
        // Grid: 80×24 cells = 160×96 dots (standard terminal)
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            // Draw 10 vertical lines
            for x in (0i32..=160).step_by(16) {
                draw_line(
                    black_box(&mut grid),
                    black_box(x),
                    black_box(0),
                    black_box(x),
                    black_box(95),
                )
                .unwrap();
            }

            // Draw 10 horizontal lines
            for y in (0i32..=96).step_by(9) {
                draw_line(
                    black_box(&mut grid),
                    black_box(0),
                    black_box(y),
                    black_box(159),
                    black_box(y),
                )
                .unwrap();
            }
        });
    });
}

/// Benchmark worst case: line with boundary clipping
fn bench_boundary_clipping(c: &mut Criterion) {
    c.bench_function("line_with_clipping", |b| {
        // Small grid: 20×10 cells = 40×40 dots
        let mut grid = BrailleGrid::new(20, 10).unwrap();

        b.iter(|| {
            // Line extends far beyond grid bounds (tests clipping performance)
            draw_line(
                black_box(&mut grid),
                black_box(-1000),
                black_box(-1000),
                black_box(1000),
                black_box(1000),
            )
            .unwrap();
        });
    });
}

/// Benchmark draw_circle for various radii
fn bench_circle_drawing(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_circle");

    // Test different radii: small (10), medium (50), large (100), very large (500)
    for radius in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("radius_{}", radius)),
            radius,
            |b, &radius| {
                // Grid large enough to hold the circle
                let grid_size = ((radius * 2 + 20) / 2) as usize;
                let mut grid = BrailleGrid::new(grid_size, (grid_size * 2) / 4).unwrap();
                let center_x = (grid_size) as i32;
                let center_y = (grid_size * 2) as i32;

                b.iter(|| {
                    draw_circle(
                        black_box(&mut grid),
                        black_box(center_x),
                        black_box(center_y),
                        black_box(radius),
                    )
                    .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark draw_circle_filled for various radii
fn bench_circle_filled(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_circle_filled");

    // Test different radii: small (10), medium (50), large (100)
    for radius in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("radius_{}", radius)),
            radius,
            |b, &radius| {
                // Grid large enough to hold the circle
                let grid_size = ((radius * 2 + 20) / 2) as usize;
                let mut grid = BrailleGrid::new(grid_size, (grid_size * 2) / 4).unwrap();
                let center_x = (grid_size) as i32;
                let center_y = (grid_size * 2) as i32;

                b.iter(|| {
                    draw_circle_filled(
                        black_box(&mut grid),
                        black_box(center_x),
                        black_box(center_y),
                        black_box(radius),
                    )
                    .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark draw_circle_thick for various thicknesses
fn bench_circle_thick(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_circle_thick");

    let radius = 100;
    // Test different thicknesses
    for thickness in [1, 3, 5, 7, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("thickness_{}", thickness)),
            thickness,
            |b, &thickness| {
                // Grid large enough to hold the circle
                let grid_size = ((radius * 2 + 20) / 2) as usize;
                let mut grid = BrailleGrid::new(grid_size, (grid_size * 2) / 4).unwrap();
                let center_x = (grid_size) as i32;
                let center_y = (grid_size * 2) as i32;

                b.iter(|| {
                    draw_circle_thick(
                        black_box(&mut grid),
                        black_box(center_x),
                        black_box(center_y),
                        black_box(radius),
                        black_box(thickness),
                    )
                    .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark realistic use case: concentric circles pattern
fn bench_concentric_circles(c: &mut Criterion) {
    c.bench_function("concentric_circles_10", |b| {
        // Grid: 80×24 cells = 160×96 dots (standard terminal)
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            // Draw 10 concentric circles with radii 5, 10, 15, ..., 50
            for i in 1..=10 {
                let radius = i * 5;
                draw_circle(
                    black_box(&mut grid),
                    black_box(80),
                    black_box(48),
                    black_box(radius),
                )
                .unwrap();
            }
        });
    });
}

/// Benchmark circle with boundary clipping
fn bench_circle_boundary_clipping(c: &mut Criterion) {
    c.bench_function("circle_with_clipping", |b| {
        // Small grid: 20×10 cells = 40×40 dots
        let mut grid = BrailleGrid::new(20, 10).unwrap();

        b.iter(|| {
            // Circle extends beyond grid bounds (tests clipping performance)
            draw_circle(
                black_box(&mut grid),
                black_box(-50),
                black_box(-50),
                black_box(200),
            )
            .unwrap();
        });
    });
}

/// Benchmark rectangle outline drawing
fn bench_rectangle_drawing(c: &mut Criterion) {
    c.bench_function("rectangle_outline_100x50", |b| {
        // Grid: 80×24 cells = 160×96 dots
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            // Rectangle 100×50 dots (target: <1ms)
            draw_rectangle(
                black_box(&mut grid),
                black_box(30),
                black_box(23),
                black_box(100),
                black_box(50),
            )
            .unwrap();
        });
    });
}

/// Benchmark filled rectangle drawing
fn bench_rectangle_filled(c: &mut Criterion) {
    c.bench_function("rectangle_filled_100x50", |b| {
        // Grid: 80×24 cells = 160×96 dots
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            // Filled rectangle 100×50 dots (target: <5ms)
            draw_rectangle_filled(
                black_box(&mut grid),
                black_box(30),
                black_box(23),
                black_box(100),
                black_box(50),
            )
            .unwrap();
        });
    });
}

/// Benchmark thick rectangle drawing
fn bench_rectangle_thick(c: &mut Criterion) {
    c.bench_function("rectangle_thick_thickness5_100x50", |b| {
        // Grid: 80×24 cells = 160×96 dots
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            // Thick rectangle thickness=5, 100×50 dots (target: <5ms)
            draw_rectangle_thick(
                black_box(&mut grid),
                black_box(30),
                black_box(23),
                black_box(100),
                black_box(50),
                black_box(5),
            )
            .unwrap();
        });
    });
}

/// Benchmark polygon outline drawing
fn bench_polygon_drawing(c: &mut Criterion) {
    c.bench_function("polygon_outline_10_vertices", |b| {
        // Grid: 80×24 cells = 160×96 dots
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        // 10-vertex polygon
        let vertices: Vec<(i32, i32)> = vec![
            (50, 20),
            (60, 25),
            (70, 30),
            (75, 40),
            (70, 50),
            (60, 55),
            (50, 60),
            (40, 55),
            (30, 50),
            (30, 40),
        ];

        b.iter(|| {
            // Polygon with 10 vertices (target: <2ms)
            draw_polygon(black_box(&mut grid), black_box(&vertices)).unwrap();
        });
    });
}

/// Benchmark filled polygon drawing
fn bench_polygon_filled(c: &mut Criterion) {
    c.bench_function("polygon_filled_10_vertices", |b| {
        // Grid: 80×24 cells = 160×96 dots
        let mut grid = BrailleGrid::new(80, 24).unwrap();

        // 10-vertex polygon
        let vertices: Vec<(i32, i32)> = vec![
            (50, 20),
            (60, 25),
            (70, 30),
            (75, 40),
            (70, 50),
            (60, 55),
            (50, 60),
            (40, 55),
            (30, 50),
            (30, 40),
        ];

        b.iter(|| {
            // Filled polygon with 10 vertices (target: <10ms)
            draw_polygon_filled(black_box(&mut grid), black_box(&vertices)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_line_drawing,
    bench_line_octants,
    bench_thick_line_drawing,
    bench_grid_pattern,
    bench_boundary_clipping,
    bench_circle_drawing,
    bench_circle_filled,
    bench_circle_thick,
    bench_concentric_circles,
    bench_circle_boundary_clipping,
    bench_rectangle_drawing,
    bench_rectangle_filled,
    bench_rectangle_thick,
    bench_polygon_drawing,
    bench_polygon_filled
);

criterion_main!(benches);
