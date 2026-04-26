//! Generate a braille watermark from an image — tuned for thin lines.

use dotmax::image::{DitheringMethod, ImageRenderer};
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or("usage: generate_watermark <image> <out_dir>")?;
    let out_dir: PathBuf = args
        .next()
        .ok_or("usage: generate_watermark <image> <out_dir>")?
        .into();

    fs::create_dir_all(&out_dir)?;

    // (name, width, height)
    let sizes: &[(&str, usize, usize)] = &[
        ("small", 16, 10),
        ("medium", 24, 14),
        ("large", 36, 20),
        ("xlarge", 52, 30),
    ];

    // (suffix, threshold, brightness, contrast)
    let variants: &[(&str, u8, f32, f32)] = &[
        ("t160", 160, 1.0, 1.0),
        ("t180", 180, 1.0, 1.0),
        ("t200", 200, 1.0, 1.0),
        ("t220", 220, 1.0, 1.0),
        ("c15_t180", 180, 1.0, 1.5),
        ("c20_t200", 200, 1.0, 2.0),
        ("b08_t180", 180, 0.8, 1.2),
    ];

    for (size_name, w, h) in sizes {
        for (variant, thresh, brightness, contrast) in variants {
            let mut builder = ImageRenderer::new()
                .load_from_path(Path::new(&image_path))?
                .resize(*w, *h, true)?
                .dithering(DitheringMethod::None)
                .brightness(*brightness)?
                .contrast(*contrast)?
                .threshold(*thresh);
            let grid = builder.render()?;

            let unicode = grid.to_unicode_grid();
            let mut text = String::with_capacity((grid.width() + 1) * grid.height());
            for row in unicode {
                for ch in row {
                    text.push(ch);
                }
                text.push('\n');
            }

            let out_path = out_dir.join(format!("thin_{}_{}.txt", size_name, variant));
            fs::write(&out_path, &text)?;
        }
    }

    println!("done — files in {}", out_dir.display());
    Ok(())
}
