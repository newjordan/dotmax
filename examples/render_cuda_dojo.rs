//! Render cuda_dojo.png at 5 progressive sizes.
//!
//! Settings are tunable via environment variables — see render_cuda_dojo.sh.

use dotmax::image::{DitheringMethod, ImageRenderer};
use std::fs;
use std::path::PathBuf;

fn env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_u8(key: &str, default: u8) -> Option<u8> {
    match std::env::var(key).ok().as_deref() {
        Some("auto") | Some("AUTO") | Some("") => None,
        Some(s) => s.parse().ok(),
        None => Some(default),
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key).ok().as_deref() {
        Some("1" | "true" | "TRUE" | "yes") => true,
        Some("0" | "false" | "FALSE" | "no") => false,
        _ => default,
    }
}

fn env_dither(key: &str) -> DitheringMethod {
    match std::env::var(key)
        .ok()
        .as_deref()
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("floyd") | Some("floydsteinberg") => DitheringMethod::FloydSteinberg,
        Some("bayer") => DitheringMethod::Bayer,
        Some("atkinson") => DitheringMethod::Atkinson,
        _ => DitheringMethod::None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or("usage: render_cuda_dojo <image> <out_dir>")?;
    let out_dir: PathBuf = args
        .next()
        .ok_or("usage: render_cuda_dojo <image> <out_dir>")?
        .into();

    fs::create_dir_all(&out_dir)?;

    let dithering = env_dither("DITHER");
    let threshold = env_u8("THRESHOLD", 128);
    let brightness = env_f32("BRIGHTNESS", 1.0);
    let contrast = env_f32("CONTRAST", 1.3);
    let gamma = env_f32("GAMMA", 1.0);
    let invert = env_bool("INVERT", true);
    let font_px = env_f32("FONT_PX", 16.0);

    println!(
        "settings: dither={:?} threshold={:?} brightness={} contrast={} gamma={} invert={}",
        dithering, threshold, brightness, contrast, gamma, invert
    );

    // Load source; optionally invert so dark figures become "on" (dotmax maps bright→filled).
    let src = image::open(&image_path)?;
    let mut rgba = src.to_rgba8();
    if invert {
        for px in rgba.pixels_mut() {
            px.0[0] = 255 - px.0[0];
            px.0[1] = 255 - px.0[1];
            px.0[2] = 255 - px.0[2];
        }
    }

    // Source is 1536x1024 (aspect 1.5). Braille chars are 2 dots wide × 4 tall,
    // so visual-preserving character ratio is 3:1 (width:height).
    let sizes: &[(&str, usize, usize)] = &[
        ("01_xs_30x10", 30, 10),
        ("02_sm_60x20", 60, 20),
        ("03_md_96x32", 96, 32),
        ("04_lg_144x48", 144, 48),
        ("05_xl_210x70", 210, 70),
    ];

    for (name, w, h) in sizes {
        let mut builder = ImageRenderer::new()
            .load_from_rgba(rgba.clone())
            .resize(*w, *h, true)?
            .dithering(dithering);

        if (brightness - 1.0).abs() > f32::EPSILON {
            builder = builder.brightness(brightness)?;
        }
        if (contrast - 1.0).abs() > f32::EPSILON {
            builder = builder.contrast(contrast)?;
        }
        if (gamma - 1.0).abs() > f32::EPSILON {
            builder = builder.gamma(gamma)?;
        }
        if let Some(t) = threshold {
            builder = builder.threshold(t);
        }

        let grid = builder.render()?;

        let unicode = grid.to_unicode_grid();
        let mut text = String::with_capacity((grid.width() + 1) * grid.height());
        for row in &unicode {
            for ch in row {
                text.push(*ch);
            }
            text.push('\n');
        }

        let txt_path = out_dir.join(format!("{}.txt", name));
        fs::write(&txt_path, &text)?;

        // Render to SVG for README embedding. Each row is a <text> element with its
        // own explicit y, so there is zero line-height gap (unlike markdown code blocks).
        // Char cell: 2:4 dot aspect; monospace fonts sit around 0.55:1 width:height,
        // so using font_px for row stride and 0.55*font_px for char stride keeps the
        // visual proportions close to the source image.
        let char_w = font_px * 0.55;
        let row_h = font_px;
        let svg_w = grid.width() as f32 * char_w;
        let svg_h = grid.height() as f32 * row_h;

        let mut svg = String::with_capacity(4096);
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {:.2} {:.2}\" width=\"{:.0}\" height=\"{:.0}\" font-family=\"ui-monospace,Menlo,Consolas,monospace\" font-size=\"{}\">\n",
            svg_w, svg_h, svg_w, svg_h, font_px
        ));
        svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n");
        for (row_idx, row) in unicode.iter().enumerate() {
            let line: String = row.iter().collect();
            // baseline ~80% down the row box keeps glyphs inside the viewBox
            let y = (row_idx as f32 + 0.8) * row_h;
            let escaped = line
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            svg.push_str(&format!(
                "<text x=\"0\" y=\"{:.2}\" xml:space=\"preserve\" textLength=\"{:.2}\" lengthAdjust=\"spacingAndGlyphs\">{}</text>\n",
                y, svg_w, escaped
            ));
        }
        svg.push_str("</svg>\n");

        let svg_path = out_dir.join(format!("{}.svg", name));
        fs::write(&svg_path, svg)?;

        println!(
            "wrote {} + {}.svg ({}x{} chars)",
            txt_path.display(),
            name,
            grid.width(),
            grid.height()
        );
    }

    println!("done — files in {}", out_dir.display());
    Ok(())
}
