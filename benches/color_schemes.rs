//! Benchmarks for color scheme performance.
//!
//! Verifies AC #2: sample() performance <100ns per call.
//!
//! Run with:
//!   cargo bench --bench color_schemes

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dotmax::color::schemes::{
    blue_purple, cyan_magenta, get_scheme, grayscale, green_yellow, heat_map, list_schemes,
    monochrome, rainbow, ColorScheme,
};

/// Benchmark sample() for a single scheme
fn bench_sample_single(c: &mut Criterion) {
    let scheme = rainbow();

    c.bench_function("sample_single_intensity", |b| {
        b.iter(|| black_box(scheme.sample(black_box(0.5))))
    });
}

/// Benchmark sample() across all intensity values (0.0 to 1.0)
fn bench_sample_gradient(c: &mut Criterion) {
    let scheme = rainbow();

    c.bench_function("sample_gradient_100_points", |b| {
        b.iter(|| {
            for i in 0..100 {
                let intensity = i as f32 / 99.0;
                black_box(scheme.sample(black_box(intensity)));
            }
        })
    });
}

/// Benchmark sample() for each predefined scheme
fn bench_sample_all_schemes(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_per_scheme");

    let schemes = [
        ("rainbow", rainbow()),
        ("heat_map", heat_map()),
        ("blue_purple", blue_purple()),
        ("green_yellow", green_yellow()),
        ("cyan_magenta", cyan_magenta()),
        ("grayscale", grayscale()),
        ("monochrome", monochrome()),
    ];

    for (name, scheme) in schemes {
        group.bench_with_input(BenchmarkId::new("sample", name), &scheme, |b, scheme| {
            b.iter(|| black_box(scheme.sample(black_box(0.5))))
        });
    }

    group.finish();
}

/// Benchmark scheme creation (cold path)
fn bench_scheme_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheme_creation");

    group.bench_function("rainbow", |b| b.iter(|| black_box(rainbow())));

    group.bench_function("heat_map", |b| b.iter(|| black_box(heat_map())));

    group.bench_function("grayscale", |b| b.iter(|| black_box(grayscale())));

    group.finish();
}

/// Benchmark discovery API
fn bench_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("discovery");

    group.bench_function("list_schemes", |b| b.iter(|| black_box(list_schemes())));

    group.bench_function("get_scheme_hit", |b| {
        b.iter(|| black_box(get_scheme(black_box("rainbow"))))
    });

    group.bench_function("get_scheme_miss", |b| {
        b.iter(|| black_box(get_scheme(black_box("nonexistent"))))
    });

    group.finish();
}

/// Benchmark custom scheme creation
fn bench_custom_scheme(c: &mut Criterion) {
    use dotmax::Color;

    c.bench_function("custom_scheme_3_colors", |b| {
        b.iter(|| {
            let colors = vec![
                Color::rgb(255, 0, 0),
                Color::rgb(0, 255, 0),
                Color::rgb(0, 0, 255),
            ];
            black_box(ColorScheme::new("custom", colors))
        })
    });
}

/// Benchmark boundary conditions
fn bench_boundary_conditions(c: &mut Criterion) {
    let scheme = rainbow();
    let mut group = c.benchmark_group("boundary_conditions");

    group.bench_function("sample_0.0", |b| {
        b.iter(|| black_box(scheme.sample(black_box(0.0))))
    });

    group.bench_function("sample_1.0", |b| {
        b.iter(|| black_box(scheme.sample(black_box(1.0))))
    });

    group.bench_function("sample_negative_clamped", |b| {
        b.iter(|| black_box(scheme.sample(black_box(-0.5))))
    });

    group.bench_function("sample_above_1_clamped", |b| {
        b.iter(|| black_box(scheme.sample(black_box(1.5))))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sample_single,
    bench_sample_gradient,
    bench_sample_all_schemes,
    bench_scheme_creation,
    bench_discovery,
    bench_custom_scheme,
    bench_boundary_conditions,
);

criterion_main!(benches);
