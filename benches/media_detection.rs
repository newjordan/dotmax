//! Benchmarks for media format detection.
//!
//! Story 9.1 AC: #7 - Format detection must complete in <5ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Write;
use tempfile::NamedTempFile;

/// Benchmark magic byte detection from byte arrays.
///
/// This tests the core detection algorithm without file I/O.
fn bench_detect_format_from_bytes(c: &mut Criterion) {
    use dotmax::media::detect_format_from_bytes;

    // PNG magic bytes
    let png_bytes = [0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    // JPEG magic bytes
    let jpeg_bytes = [0xFFu8, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];

    // GIF magic bytes
    let gif_bytes = [0x47u8, 0x49, 0x46, 0x38, 0x39, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    // Unknown bytes (worst case - checks all patterns)
    let unknown_bytes = [0x00u8; 16];

    c.bench_function("detect_format_from_bytes/png", |b| {
        b.iter(|| detect_format_from_bytes(black_box(&png_bytes)))
    });

    c.bench_function("detect_format_from_bytes/jpeg", |b| {
        b.iter(|| detect_format_from_bytes(black_box(&jpeg_bytes)))
    });

    c.bench_function("detect_format_from_bytes/gif", |b| {
        b.iter(|| detect_format_from_bytes(black_box(&gif_bytes)))
    });

    c.bench_function("detect_format_from_bytes/unknown", |b| {
        b.iter(|| detect_format_from_bytes(black_box(&unknown_bytes)))
    });
}

/// Benchmark format detection from files.
///
/// This tests the full detection path including file I/O.
/// AC: #7 - Must complete in <5ms regardless of file size.
fn bench_detect_format_file(c: &mut Criterion) {
    use dotmax::media::detect_format;

    // Create a temp file with PNG magic bytes
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    // Write PNG header followed by some padding
    let png_header = [0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    temp_file.write_all(&png_header).expect("Failed to write header");

    // Add 1MB of padding to simulate a larger file
    // Detection should still be fast since we only read first 16 bytes
    let padding = vec![0u8; 1024 * 1024];
    temp_file.write_all(&padding).expect("Failed to write padding");
    temp_file.flush().expect("Failed to flush");

    let path = temp_file.path();

    c.bench_function("detect_format/1mb_png_file", |b| {
        b.iter(|| detect_format(black_box(path)))
    });
}

/// Benchmark extension fallback.
fn bench_extension_fallback(c: &mut Criterion) {
    use dotmax::media::detect_format;

    // Create a temp file with unknown magic bytes but known extension
    let mut temp_file = NamedTempFile::with_suffix(".png").expect("Failed to create temp file");

    // Write unknown bytes
    temp_file.write_all(&[0x00u8; 16]).expect("Failed to write");
    temp_file.flush().expect("Failed to flush");

    let path = temp_file.path();

    c.bench_function("detect_format/extension_fallback", |b| {
        b.iter(|| detect_format(black_box(path)))
    });
}

criterion_group!(
    benches,
    bench_detect_format_from_bytes,
    bench_detect_format_file,
    bench_extension_fallback
);

criterion_main!(benches);
