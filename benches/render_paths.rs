//! Criterion benches for the three rendering paths: image, animation, and text.
//!
//! ```text
//! cargo bench --bench render_paths --features image -- --sample-size 20
//! ```

use criterion::{criterion_group, criterion_main, Criterion};
use dotmax::animation::{FrameBuffer, PrerenderedAnimation};
use dotmax::density::DensitySet;
use dotmax::image::{
    apply_dithering, apply_threshold, pixels_to_braille, resize_to_dimensions, to_grayscale,
    DitheringMethod,
};
use dotmax::BrailleGrid;
use image::{DynamicImage, RgbImage};
use std::hint::black_box;

fn synthetic_image(width: u32, height: u32) -> DynamicImage {
    let img = RgbImage::from_fn(width, height, |x, y| {
        let r = (x.wrapping_mul(17).wrapping_add(y.wrapping_mul(9))) as u8;
        let g = (y.wrapping_mul(13)) as u8;
        let b = (x.wrapping_mul(3).wrapping_add(80)) as u8;
        image::Rgb([r, g, b])
    });
    DynamicImage::ImageRgb8(img)
}

fn bench_image_pipeline(c: &mut Criterion) {
    let image = synthetic_image(160, 96);
    c.bench_function("image_pipeline_160x96", |b| {
        b.iter(|| {
            let resized = resize_to_dimensions(black_box(&image), 80, 48, false).expect("resize");
            let gray = to_grayscale(&resized);
            let binary = apply_dithering(&gray, DitheringMethod::FloydSteinberg).expect("dither");
            let grid = pixels_to_braille(&binary, 40, 12).expect("map");
            black_box(grid)
        });
    });

    c.bench_function("image_threshold_80x48", |b| {
        let resized = resize_to_dimensions(&image, 80, 48, false).expect("resize");
        let gray = to_grayscale(&resized);
        b.iter(|| {
            let binary = apply_threshold(black_box(&gray), 128);
            let grid = pixels_to_braille(&binary, 40, 12).expect("map");
            black_box(grid)
        });
    });
}

fn draw_frame(frame: usize, grid: &mut BrailleGrid) {
    grid.clear();
    let x = (frame * 3) % grid.dot_width();
    let y = grid.dot_height() / 2;
    let _ = grid.set_dot(x, y);
    let _ = grid.set_dot(x.saturating_add(1), y);
}

fn bench_animation(c: &mut Criterion) {
    c.bench_function("animation_frame_12x4", |b| {
        let mut grid = BrailleGrid::new(12, 4).expect("grid");
        let mut frame = 0usize;
        b.iter(|| {
            draw_frame(frame, &mut grid);
            frame = frame.wrapping_add(1);
            black_box(grid.get_char(0, 0));
        });
    });

    c.bench_function("animation_buffer_swap_40x12", |b| {
        let mut buffer = FrameBuffer::new(40, 12);
        b.iter(|| {
            draw_frame(3, buffer.get_back_buffer());
            buffer.swap_buffers();
            black_box(buffer.get_front_buffer().get_char(1, 1));
        });
    });

    c.bench_function("animation_prerender_build_16", |b| {
        b.iter(|| {
            let mut animation = PrerenderedAnimation::new(30);
            for frame in 0..16 {
                let mut grid = BrailleGrid::new(12, 4).expect("grid");
                draw_frame(frame, &mut grid);
                animation.add_frame(grid);
            }
            black_box(animation.frame_count())
        });
    });
}

fn bench_text(c: &mut Criterion) {
    let density = DensitySet::simple();
    let width = 80usize;
    let height = 24usize;
    let intensities: Vec<f32> = (0..width * height)
        .map(|i| (i % width) as f32 / (width - 1) as f32)
        .collect();

    c.bench_function("text_density_80x24", |b| {
        let mut grid = BrailleGrid::new(width, height).expect("grid");
        b.iter(|| {
            grid.render_density(black_box(&intensities), black_box(&density))
                .expect("density");
            black_box(grid.get_char(10, 2));
        });
    });

    let message = "dotmax renders text as terminal cells";
    c.bench_function("text_set_char_line", |b| {
        let mut grid = BrailleGrid::new(message.chars().count(), 1).expect("grid");
        b.iter(|| {
            for (x, ch) in message.chars().enumerate() {
                grid.set_char(x, 0, ch).expect("char");
            }
            black_box(grid.get_char(0, 0));
        });
    });
}

criterion_group!(paths, bench_image_pipeline, bench_animation, bench_text);
criterion_main!(paths);
