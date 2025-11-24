//! Benchmark suite for color conversion functions.
//!
//! Performance targets from Story 5.2:
//! - `rgb_to_ansi256`: <100ns per conversion
//! - `rgb_to_ansi16`: <50ns per conversion
//! - `rgb_to_truecolor_escape`: <50ns per conversion
//! - `rgb_to_terminal_color`: <150ns per conversion (includes capability check)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dotmax::color::convert::{
    rgb_to_ansi16, rgb_to_ansi256, rgb_to_terminal_color, rgb_to_truecolor_escape,
};
use dotmax::ColorCapability;

/// Benchmark rgb_to_ansi256 conversion.
///
/// Target: <100ns per conversion
fn bench_rgb_to_ansi256(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgb_to_ansi256");

    // Test with different color types
    group.bench_function("pure_red", |b| {
        b.iter(|| rgb_to_ansi256(black_box(255), black_box(0), black_box(0)))
    });

    group.bench_function("pure_green", |b| {
        b.iter(|| rgb_to_ansi256(black_box(0), black_box(255), black_box(0)))
    });

    group.bench_function("pure_blue", |b| {
        b.iter(|| rgb_to_ansi256(black_box(0), black_box(0), black_box(255)))
    });

    group.bench_function("gray", |b| {
        b.iter(|| rgb_to_ansi256(black_box(128), black_box(128), black_box(128)))
    });

    group.bench_function("random_color", |b| {
        b.iter(|| rgb_to_ansi256(black_box(173), black_box(94), black_box(212)))
    });

    // Batch conversion benchmark (1000 colors)
    group.bench_function("batch_1000", |b| {
        b.iter(|| {
            for r in (0..=255).step_by(26) {
                for g in (0..=255).step_by(26) {
                    for b in (0..=255).step_by(85) {
                        black_box(rgb_to_ansi256(r, g, b));
                    }
                }
            }
        })
    });

    group.finish();
}

/// Benchmark rgb_to_ansi16 conversion.
///
/// Target: <50ns per conversion
fn bench_rgb_to_ansi16(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgb_to_ansi16");

    group.bench_function("pure_red", |b| {
        b.iter(|| rgb_to_ansi16(black_box(255), black_box(0), black_box(0)))
    });

    group.bench_function("pure_green", |b| {
        b.iter(|| rgb_to_ansi16(black_box(0), black_box(255), black_box(0)))
    });

    group.bench_function("gray", |b| {
        b.iter(|| rgb_to_ansi16(black_box(128), black_box(128), black_box(128)))
    });

    group.bench_function("random_color", |b| {
        b.iter(|| rgb_to_ansi16(black_box(173), black_box(94), black_box(212)))
    });

    // Batch conversion benchmark
    group.bench_function("batch_1000", |b| {
        b.iter(|| {
            for r in (0..=255).step_by(26) {
                for g in (0..=255).step_by(26) {
                    for b in (0..=255).step_by(85) {
                        black_box(rgb_to_ansi16(r, g, b));
                    }
                }
            }
        })
    });

    group.finish();
}

/// Benchmark rgb_to_truecolor_escape string generation.
///
/// Target: <50ns per escape code
fn bench_rgb_to_truecolor_escape(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgb_to_truecolor_escape");

    group.bench_function("typical", |b| {
        b.iter(|| rgb_to_truecolor_escape(black_box(255), black_box(128), black_box(0)))
    });

    group.bench_function("zeros", |b| {
        b.iter(|| rgb_to_truecolor_escape(black_box(0), black_box(0), black_box(0)))
    });

    group.bench_function("max_values", |b| {
        b.iter(|| rgb_to_truecolor_escape(black_box(255), black_box(255), black_box(255)))
    });

    group.finish();
}

/// Benchmark rgb_to_terminal_color smart conversion.
///
/// Target: <150ns per conversion (includes capability matching)
fn bench_rgb_to_terminal_color(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgb_to_terminal_color");

    // Benchmark each capability level
    for capability in [
        ColorCapability::TrueColor,
        ColorCapability::Ansi256,
        ColorCapability::Ansi16,
        ColorCapability::Monochrome,
    ] {
        group.bench_with_input(
            BenchmarkId::new("capability", format!("{:?}", capability)),
            &capability,
            |b, cap| {
                b.iter(|| rgb_to_terminal_color(black_box(255), black_box(128), black_box(0), *cap))
            },
        );
    }

    // Mixed capabilities batch benchmark
    group.bench_function("mixed_batch_1000", |b| {
        let caps = [
            ColorCapability::TrueColor,
            ColorCapability::Ansi256,
            ColorCapability::Ansi16,
            ColorCapability::Monochrome,
        ];
        b.iter(|| {
            for (i, r) in (0..=255).step_by(26).enumerate() {
                for g in (0..=255).step_by(51) {
                    let cap = caps[i % 4];
                    black_box(rgb_to_terminal_color(r, g, 128, cap));
                }
            }
        })
    });

    group.finish();
}

/// Throughput benchmark: conversions per second.
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Test 1M conversions
    group.sample_size(10);
    group.bench_function("ansi256_1m_conversions", |b| {
        b.iter(|| {
            let mut sum = 0u32;
            for _ in 0..1_000_000 {
                sum += u32::from(rgb_to_ansi256(
                    black_box(173),
                    black_box(94),
                    black_box(212),
                ));
            }
            black_box(sum)
        })
    });

    group.bench_function("ansi16_1m_conversions", |b| {
        b.iter(|| {
            let mut sum = 0u32;
            for _ in 0..1_000_000 {
                sum += u32::from(rgb_to_ansi16(black_box(173), black_box(94), black_box(212)));
            }
            black_box(sum)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rgb_to_ansi256,
    bench_rgb_to_ansi16,
    bench_rgb_to_truecolor_escape,
    bench_rgb_to_terminal_color,
    bench_throughput,
);

criterion_main!(benches);
