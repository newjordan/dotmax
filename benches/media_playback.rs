//! Benchmarks for media player frame decoding.
//!
//! Story 9.5: Integration and polish - benchmarks for animated media playback.
//!
//! These benchmarks measure:
//! - GIF frame decode time (AC: should support 60fps)
//! - APNG frame decode time
//! - MediaPlayer trait overhead

#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::items_after_statements)]
#![allow(deprecated)]  // criterion::black_box is deprecated but still used in existing benches

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;

#[cfg(feature = "image")]
fn bench_gif_playback(c: &mut Criterion) {
    use dotmax::media::{GifPlayer, MediaPlayer};

    let path = Path::new("tests/fixtures/media/animated.gif");
    if !path.exists() {
        eprintln!("Skipping GIF benchmarks: test fixture not found");
        return;
    }

    c.bench_function("gif_player/new", |b| {
        b.iter(|| GifPlayer::new(black_box(path)))
    });

    // Measure frame decode time
    c.bench_function("gif_player/next_frame", |b| {
        let mut player = GifPlayer::new(path).expect("Failed to create player");

        b.iter(|| {
            // Get next frame
            let frame = player.next_frame();
            if frame.is_none() {
                // Reset if we've reached the end
                player.reset();
            }
            frame
        })
    });

    // Measure single frame decode
    c.bench_function("gif_player/single_frame_decode", |b| {
        b.iter_batched(
            || GifPlayer::new(path).expect("Failed to create player"),
            |mut player| player.next_frame(),
            criterion::BatchSize::SmallInput,
        )
    });
}

#[cfg(feature = "image")]
fn bench_apng_playback(c: &mut Criterion) {
    use dotmax::media::{ApngPlayer, MediaPlayer};

    let path = Path::new("tests/fixtures/media/animated.png");
    if !path.exists() {
        eprintln!("Skipping APNG benchmarks: test fixture not found");
        return;
    }

    c.bench_function("apng_player/new", |b| {
        b.iter(|| ApngPlayer::new(black_box(path)))
    });

    // Measure frame decode time
    c.bench_function("apng_player/next_frame", |b| {
        let mut player = ApngPlayer::new(path).expect("Failed to create player");

        b.iter(|| {
            let frame = player.next_frame();
            if frame.is_none() {
                player.reset();
            }
            frame
        })
    });

    // Measure single frame decode
    c.bench_function("apng_player/single_frame_decode", |b| {
        b.iter_batched(
            || ApngPlayer::new(path).expect("Failed to create player"),
            |mut player| player.next_frame(),
            criterion::BatchSize::SmallInput,
        )
    });
}

#[cfg(feature = "image")]
fn bench_media_content_load(c: &mut Criterion) {
    use dotmax::quick;

    let gif_path = Path::new("tests/fixtures/media/animated.gif");
    let png_path = Path::new("tests/fixtures/media/animated.png");
    let static_path = Path::new("tests/fixtures/media/static_png.png");

    if gif_path.exists() {
        c.bench_function("load_file/animated_gif", |b| {
            b.iter(|| quick::load_file(black_box(gif_path)))
        });
    }

    if png_path.exists() {
        c.bench_function("load_file/animated_png", |b| {
            b.iter(|| quick::load_file(black_box(png_path)))
        });
    }

    if static_path.exists() {
        c.bench_function("load_file/static_png", |b| {
            b.iter(|| quick::load_file(black_box(static_path)))
        });
    }
}

#[cfg(feature = "image")]
fn bench_resize_handling(c: &mut Criterion) {
    use dotmax::media::{GifPlayer, MediaPlayer};

    let path = Path::new("tests/fixtures/media/animated.gif");
    if !path.exists() {
        eprintln!("Skipping resize benchmarks: test fixture not found");
        return;
    }

    c.bench_function("gif_player/handle_resize", |b| {
        let mut player = GifPlayer::new(path).expect("Failed to create player");

        b.iter(|| {
            player.handle_resize(black_box(120), black_box(40));
        })
    });
}

#[cfg(feature = "image")]
criterion_group!(
    benches,
    bench_gif_playback,
    bench_apng_playback,
    bench_media_content_load,
    bench_resize_handling,
);

#[cfg(not(feature = "image"))]
fn bench_no_image_feature(_c: &mut Criterion) {
    eprintln!("Media playback benchmarks require the 'image' feature");
}

#[cfg(not(feature = "image"))]
criterion_group!(benches, bench_no_image_feature);

criterion_main!(benches);
