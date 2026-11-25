//! Unit and snapshot tests for the image, animation, and text rendering paths.
//!
//! Snapshots are plain UTF-8 captures of what a terminal would print. Set
//! `UPDATE_SNAPSHOTS=1` to rewrite them after an intentional renderer change.

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use dotmax::animation::{FrameBuffer, PrerenderedAnimation};
use dotmax::density::DensitySet;
use dotmax::primitives::{draw_circle, draw_line};
use dotmax::BrailleGrid;

fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/render_paths")
}

fn grid_text(grid: &BrailleGrid) -> String {
    let (width, height) = grid.dimensions();
    let mut out = String::new();
    for y in 0..height {
        for x in 0..width {
            out.push(grid.get_char(x, y));
        }
        out.push('\n');
    }
    out
}

fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_dir().join(name);
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() || !path.exists() {
        fs::create_dir_all(path.parent().expect("snapshot parent")).expect("create snapshot dir");
        fs::write(&path, actual).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    }
    let expected =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    assert_eq!(
        expected, actual,
        "snapshot {name} changed; set UPDATE_SNAPSHOTS=1 to accept it"
    );
}

fn bouncing_frame(frame: usize) -> BrailleGrid {
    let mut grid = BrailleGrid::new(12, 4).expect("grid");
    let travel = grid.dot_width().saturating_sub(1);
    let x = if travel == 0 {
        0
    } else {
        let period = travel * 2;
        let t = frame % period;
        if t <= travel {
            t
        } else {
            period - t
        }
    };
    let y = 6 + ((frame % 5) as i32 - 2).unsigned_abs() as usize;
    grid.set_dot(x, y).expect("dot");
    if x > 0 {
        let _ = grid.set_dot(x - 1, y);
    }
    grid
}

#[test]
fn animation_frames_match_snapshot() {
    let mut capture = String::new();
    for frame in 0..6 {
        let _ = writeln!(capture, "-- frame {frame} --");
        capture.push_str(&grid_text(&bouncing_frame(frame)));
    }
    assert_snapshot("animation_bounce.txt", &capture);
}

#[test]
fn animation_buffer_swap_publishes_the_back_buffer() {
    let mut buffer = FrameBuffer::new(8, 2);
    buffer.get_back_buffer().set_dot(1, 1).expect("dot");
    assert_eq!(
        buffer.get_front_buffer().get_char(0, 0),
        '⠀',
        "front stays blank until swap"
    );
    buffer.swap_buffers();
    assert_ne!(
        buffer.get_front_buffer().get_char(0, 0),
        '⠀',
        "swap publishes the drawn dot"
    );
    assert_snapshot(
        "animation_swapped_frame.txt",
        &grid_text(buffer.get_front_buffer()),
    );
}

#[test]
fn animation_prerender_roundtrip_preserves_frames() {
    let mut animation = PrerenderedAnimation::new(24);
    for frame in 0..4 {
        animation.add_frame(bouncing_frame(frame));
    }
    assert_eq!(animation.frame_count(), 4);

    let path = std::env::temp_dir().join("dotmax-prerender-snapshot.dmax");
    animation.save_to_file(&path).expect("save");
    let loaded = PrerenderedAnimation::load_from_file(&path).expect("load");
    let _ = fs::remove_file(&path);
    assert_eq!(loaded.frame_count(), 4);
}

#[test]
fn text_density_gradient_matches_snapshot() {
    let width = 20;
    let height = 3;
    let mut grid = BrailleGrid::new(width, height).expect("grid");
    let intensities: Vec<f32> = (0..width * height)
        .map(|i| (i % width) as f32 / (width - 1) as f32)
        .collect();
    grid.render_density(&intensities, &DensitySet::simple())
        .expect("density");
    assert_snapshot("text_density_gradient.txt", &grid_text(&grid));
    assert_eq!(grid.get_char(0, 0), ' ');
    assert_eq!(grid.get_char(width - 1, 0), '@');
}

#[test]
fn text_set_char_overrides_braille_dots() {
    let mut grid = BrailleGrid::new(13, 1).expect("grid");
    grid.set_dot(0, 0).expect("dot");
    for (x, ch) in "HELLO BRAILLE".chars().enumerate() {
        grid.set_char(x, 0, ch).expect("char");
    }
    assert_snapshot("text_hello.txt", &grid_text(&grid));
    assert_eq!(grid.get_char(0, 0), 'H');
    grid.clear_characters();
    assert_ne!(grid.get_char(0, 0), 'H');
}

#[test]
fn text_render_density_rejects_a_short_buffer() {
    let mut grid = BrailleGrid::new(4, 2).expect("grid");
    let err = grid
        .render_density(&[0.0, 1.0], &DensitySet::blocks())
        .expect_err("short buffer");
    let message = err.to_string();
    assert!(
        message.contains('8') || message.to_lowercase().contains("buffer"),
        "unexpected error: {message}"
    );
}

#[test]
fn drawing_primitives_match_snapshot() {
    let mut grid = BrailleGrid::new(16, 6).expect("grid");
    draw_line(&mut grid, 0, 0, 31, 23).expect("line");
    draw_circle(&mut grid, 20, 12, 8).expect("circle");
    assert_snapshot("primitives_line_circle.txt", &grid_text(&grid));
}

#[cfg(feature = "image")]
mod image_path {
    use super::{assert_snapshot, grid_text};
    use dotmax::image::{
        apply_dithering, apply_threshold, pixels_to_braille, resize_to_dimensions, to_grayscale,
        DitheringMethod,
    };
    use image::{DynamicImage, RgbImage};

    fn picture(width: u32, height: u32) -> DynamicImage {
        let img = RgbImage::from_fn(width, height, |x, y| {
            let on = ((x / 2) + (y / 2)) % 2 == 0;
            if on {
                image::Rgb([20, 20, 24])
            } else {
                image::Rgb([235, 235, 240])
            }
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn threshold_image_matches_snapshot() {
        let image = picture(16, 8);
        let gray = to_grayscale(&image);
        let binary = apply_threshold(&gray, 128);
        let grid = pixels_to_braille(&binary, 8, 2).expect("map");
        assert_eq!((grid.width(), grid.height()), (8, 2));
        assert_snapshot("image_threshold_checker.txt", &grid_text(&grid));
        assert!(
            grid_text(&grid).chars().any(|ch| ch != '⠀' && ch != '\n'),
            "checkerboard should set some dots"
        );
    }

    #[test]
    fn dithered_resize_pipeline_matches_snapshot() {
        let image = picture(32, 16);
        let resized = resize_to_dimensions(&image, 16, 8, false).expect("resize");
        let gray = to_grayscale(&resized);
        let binary = apply_dithering(&gray, DitheringMethod::Bayer).expect("dither");
        let grid = pixels_to_braille(&binary, 8, 2).expect("map");
        assert_snapshot("image_bayer_pipeline.txt", &grid_text(&grid));
    }

    #[test]
    fn empty_image_is_rejected() {
        let binary = dotmax::image::BinaryImage::new(0, 4);
        let err = pixels_to_braille(&binary, 1, 1).expect_err("empty");
        assert!(
            err.to_string().to_lowercase().contains("dimension") || err.to_string().contains('0')
        );
    }
}

#[cfg(feature = "chess")]
#[test]
fn chess_pgn_renders_the_starting_position() {
    let board = dotmax::chess::render_pgn("1. e4 *", Some(0)).expect("pgn");
    assert!(board.lines().count() > 4, "board:\n{board}");
    assert_snapshot("chess_start.txt", &board);
}
