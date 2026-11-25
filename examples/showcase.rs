//! Print braille for the text, animation, and image paths.
//!
//! No terminal UI: the frames are written to stdout, so this runs in CI and
//! in a pipe.
//!
//! ```text
//! cargo run --example showcase
//! cargo run --example showcase --features image -- --gif
//! ```
//!
//! `--gif` (image feature) writes `docs/examples/bouncing_dot.gif`.

use dotmax::density::DensitySet;
use dotmax::primitives::{draw_circle, draw_line};
use dotmax::BrailleGrid;

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

fn bouncing_frame(frame: usize) -> BrailleGrid {
    let mut grid = BrailleGrid::new(16, 5).expect("grid");
    let travel = grid.dot_width() - 1;
    let period = travel * 2;
    let t = frame % period;
    let x = if t <= travel { t } else { period - t };
    let y = 8;
    let _ = grid.set_dot(x, y);
    let _ = grid.set_dot(x, y + 1);
    if x > 1 {
        let _ = grid.set_dot(x - 2, y + 1);
    }
    grid
}

fn text_sample() -> String {
    let width = 24;
    let mut grid = BrailleGrid::new(width, 2).expect("grid");
    let intensities: Vec<f32> = (0..width * 2)
        .map(|i| (i % width) as f32 / (width - 1) as f32)
        .collect();
    grid.render_density(&intensities, &DensitySet::simple())
        .expect("density");
    let mut label = BrailleGrid::new(11, 1).expect("label");
    for (x, ch) in "DOTMAX TEXT".chars().enumerate() {
        label.set_char(x, 0, ch).expect("char");
    }
    format!("{}\n{}", grid_text(&label), grid_text(&grid))
}

fn shape_sample() -> String {
    let mut grid = BrailleGrid::new(18, 6).expect("grid");
    draw_line(&mut grid, 0, 2, 35, 20).expect("line");
    draw_circle(&mut grid, 24, 12, 8).expect("circle");
    grid_text(&grid)
}

#[cfg(feature = "image")]
fn image_sample() -> (String, BrailleGrid) {
    use dotmax::image::{apply_threshold, pixels_to_braille, to_grayscale};
    use image::{DynamicImage, RgbImage};

    let img = RgbImage::from_fn(24, 12, |x, y| {
        let dx = x as i32 - 12;
        let dy = (y as i32 - 6) * 2;
        if dx * dx + dy * dy < 36 {
            image::Rgb([16, 16, 20])
        } else if x > 16 {
            image::Rgb([210, 90, 40])
        } else {
            image::Rgb([230, 230, 235])
        }
    });
    let gray = to_grayscale(&DynamicImage::ImageRgb8(img));
    let binary = apply_threshold(&gray, 140);
    let grid = pixels_to_braille(&binary, 12, 3).expect("map");
    (grid_text(&grid), grid)
}

#[cfg(feature = "image")]
fn write_gif(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use image::codecs::gif::{GifEncoder, Repeat};
    use image::{Delay, Frame, Rgba, RgbaImage};
    use std::fs::File;

    let scale = 6u32;
    let frames: Vec<BrailleGrid> = (0..24).map(bouncing_frame).collect();
    let (cells_w, cells_h) = frames[0].dimensions();
    let width = (cells_w as u32) * 2 * scale;
    let height = (cells_h as u32) * 4 * scale;

    let file = File::create(path)?;
    let mut encoder = GifEncoder::new(file);
    encoder.set_repeat(Repeat::Infinite)?;

    for grid in &frames {
        let mut img = RgbaImage::from_pixel(width, height, Rgba([12, 14, 20, 255]));
        for dot_y in 0..grid.dot_height() {
            for dot_x in 0..grid.dot_width() {
                let cell_x = dot_x / 2;
                let cell_y = dot_y / 4;
                let pattern = {
                    let ch = grid.get_char(cell_x, cell_y);
                    // Blank braille is U+2800. Any other braille char has bits set.
                    let bits = (ch as u32).wrapping_sub(0x2800);
                    if (0x2800..=0x28FF).contains(&(ch as u32)) {
                        bits
                    } else {
                        0
                    }
                };
                let local_x = dot_x % 2;
                let local_y = dot_y % 4;
                let bit = match (local_x, local_y) {
                    (0, 0) => 0x01,
                    (0, 1) => 0x02,
                    (0, 2) => 0x04,
                    (0, 3) => 0x40,
                    (1, 0) => 0x08,
                    (1, 1) => 0x10,
                    (1, 2) => 0x20,
                    (1, 3) => 0x80,
                    _ => 0,
                };
                if pattern & bit == 0 {
                    continue;
                }
                let px = (dot_x as u32) * scale;
                let py = (dot_y as u32) * scale;
                for oy in 1..scale - 1 {
                    for ox in 1..scale - 1 {
                        img.put_pixel(px + ox, py + oy, Rgba([186, 230, 253, 255]));
                    }
                }
            }
        }
        encoder.encode_frame(Frame::from_parts(
            img,
            0,
            0,
            Delay::from_numer_denom_ms(100, 1),
        ))?;
    }
    Ok(())
}

fn main() {
    println!("=== text ===");
    print!("{}", text_sample());
    println!("=== shapes ===");
    print!("{}", shape_sample());
    println!("=== animation ===");
    for frame in [0usize, 4, 8, 16] {
        println!("-- frame {frame} --");
        print!("{}", grid_text(&bouncing_frame(frame)));
    }

    #[cfg(feature = "image")]
    {
        println!("=== image ===");
        let (text, _) = image_sample();
        print!("{text}");
        if std::env::args().any(|arg| arg == "--gif") {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("docs/examples/bouncing_dot.gif");
            write_gif(&path).expect("write gif");
            println!("wrote {}", path.display());
        }
    }
}
