//! Render any image to braille at 5 progressive sizes (TXT + SVG).
//! Settings via env vars — see render_braille.sh.
//!
//! Each size now emits FRAMES (default 8) variations driven by ambient
//! dithering: same input image, evolving binary output per frame, so the
//! preview HTML shows it "breathing" instead of locking to a single static
//! frame.

use dotmax::image::{DitheringMethod, ImageRenderer};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_u8(key: &str, default: u8) -> Option<u8> {
    match std::env::var(key).ok().as_deref() {
        Some("auto" | "AUTO" | "") => None,
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
        Some("floyd" | "floydsteinberg") => DitheringMethod::FloydSteinberg,
        Some("bayer") => DitheringMethod::Bayer,
        Some("atkinson") => DitheringMethod::Atkinson,
        _ => DitheringMethod::None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or("usage: render_braille <image> <out_dir>")?;
    let out_dir: PathBuf = args
        .next()
        .ok_or("usage: render_braille <image> <out_dir>")?
        .into();

    fs::create_dir_all(&out_dir)?;

    let dithering = env_dither("DITHER");
    let threshold = env_u8("THRESHOLD", 128);
    let brightness = env_f32("BRIGHTNESS", 1.0);
    let contrast = env_f32("CONTRAST", 1.3);
    let gamma = env_f32("GAMMA", 1.0);
    let invert = env_bool("INVERT", true);
    let invert_output = env_bool("INVERT_OUTPUT", false);
    let font_px = env_f32("FONT_PX", 16.0);
    // Ambient dithering controls — defaults make every render evolve.
    let frames = env_usize("FRAMES", 8).max(1);
    let ambient = env_f32("AMBIENT", 0.55).clamp(0.0, 1.0);
    // Optional pinned seed for reproducible animations. 0 → clock-driven.
    let ambient_seed: u64 = std::env::var("AMBIENT_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let frame_ms = env_usize("FRAME_MS", 120);

    let stem = Path::new(&image_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out")
        .to_string();

    println!(
        "settings: dither={:?} threshold={:?} brightness={} contrast={} gamma={} invert={} ambient={:.2} frames={} stem={}",
        dithering, threshold, brightness, contrast, gamma, invert, ambient, frames, stem
    );

    let src = image::open(&image_path)?;
    let (src_w, src_h) = (src.width() as f32, src.height() as f32);

    let mut rgba = src.to_rgba8();
    if invert {
        for px in rgba.pixels_mut() {
            px.0[0] = 255 - px.0[0];
            px.0[1] = 255 - px.0[1];
            px.0[2] = 255 - px.0[2];
        }
    }

    // Braille cell = 2 dots × 4 dots. For pixel-accurate aspect:
    //   width_chars / height_chars = 2 × (src_w / src_h)
    let char_aspect = 2.0 * src_w / src_h;
    let widths = [30usize, 60, 96, 144, 210];
    let sizes: Vec<(String, usize, usize)> = widths
        .iter()
        .enumerate()
        .map(|(i, &w)| {
            let h = ((w as f32 / char_aspect).round() as usize).max(4);
            let tag = match i {
                0 => "xs",
                1 => "sm",
                2 => "md",
                3 => "lg",
                _ => "xl",
            };
            (format!("{:02}_{}_{}x{}", i + 1, tag, w, h), w, h)
        })
        .collect();

    for (name, w, h) in &sizes {
        // One renderer per size — the frame counter inside it advances with
        // each render() call, so frames evolve naturally.
        let mut builder = ImageRenderer::new()
            .load_from_rgba(rgba.clone())
            .resize(*w, *h, true)?
            .dithering(dithering)
            .ambient(ambient);

        if ambient_seed != 0 {
            builder = builder.ambient_seed(ambient_seed);
        }

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

        let mut last_dims = (0u32, 0u32);
        for frame_idx in 0..frames {
            let grid = builder.render()?;
            last_dims = (grid.width() as u32, grid.height() as u32);

            let mut unicode = grid.to_unicode_grid();
            if invert_output {
                // Flip the 8 dot bits of each braille glyph (0x2800 + bits)
                // so filled ↔ empty — useful when dithering gave a solid
                // background.
                for row in &mut unicode {
                    for ch in row.iter_mut() {
                        let bits = (*ch as u32).saturating_sub(0x2800) as u8;
                        *ch = char::from_u32(0x2800 + (!bits as u32)).unwrap_or(*ch);
                    }
                }
            }
            let mut text = String::with_capacity((grid.width() + 1) * grid.height());
            for row in &unicode {
                for ch in row {
                    text.push(*ch);
                }
                text.push('\n');
            }

            let txt_path = out_dir.join(format!("{}_{}_f{:02}.txt", stem, name, frame_idx));
            fs::write(&txt_path, &text)?;

            // SVG with per-row explicit y: zero line-height gap when embedded via <img>.
            let char_w_svg = font_px * 0.55;
            let row_h = font_px;
            let svg_w = grid.width() as f32 * char_w_svg;
            let svg_h = grid.height() as f32 * row_h;

            let mut svg = String::with_capacity(4096);
            let _ = writeln!(
                svg,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {svg_w:.2} {svg_h:.2}\" width=\"{svg_w:.0}\" height=\"{svg_h:.0}\" font-family=\"ui-monospace,Menlo,Consolas,monospace\" font-size=\"{font_px}\">"
            );
            svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n");
            for (row_idx, row) in unicode.iter().enumerate() {
                let line: String = row.iter().collect();
                let y = (row_idx as f32 + 0.8) * row_h;
                let escaped = line
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                let _ = writeln!(
                    svg,
                    "<text x=\"0\" y=\"{y:.2}\" xml:space=\"preserve\" textLength=\"{svg_w:.2}\" lengthAdjust=\"spacingAndGlyphs\">{escaped}</text>"
                );
            }
            svg.push_str("</svg>\n");

            let svg_path = out_dir.join(format!("{}_{}_f{:02}.svg", stem, name, frame_idx));
            fs::write(&svg_path, svg)?;
        }

        println!(
            "  {}x{} chars × {} frames → {}_{}_f00..f{:02}.{{txt,svg}}",
            last_dims.0,
            last_dims.1,
            frames,
            stem,
            name,
            frames - 1
        );
    }

    // Preview HTML — for each size, cycle through ambient frames at FRAME_MS
    // so the image visibly evolves; then move to the next size; finally zoom
    // on the XL pyramid frames.
    fn build_seq(
        stem: &str,
        name: &str,
        frames: usize,
        frame_ms: usize,
        scale: f32,
        ox: &str,
        oy: &str,
    ) -> String {
        (0..frames)
            .map(|i| {
                format!(
                    "{{src:'{stem}_{name}_f{i:02}.svg',ms:{frame_ms},scale:{scale},ox:'{ox}',oy:'{oy}'}}"
                )
            })
            .collect::<Vec<_>>()
            .join(",\n    ")
    }

    let frames_js = sizes
        .iter()
        .map(|(name, _, _)| build_seq(&stem, name, frames, frame_ms, 1.0, "50%", "50%"))
        .collect::<Vec<_>>()
        .join(",\n    ");
    let xl_name = &sizes.last().unwrap().0;
    let zoom_seq = build_seq(&stem, xl_name, frames, frame_ms, 2.8, "54%", "28%");
    let html = format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>{stem} preview</title>
<style>
  html,body {{ margin:0; height:100%; background:#111; color:#ddd; font-family:ui-monospace,monospace; }}
  .stage {{ position:fixed; inset:0; display:grid; place-items:center; overflow:hidden; }}
  .stage img {{
    max-width:96vw; max-height:92vh;
    transition: transform 400ms ease, opacity 80ms ease;
    image-rendering: pixelated;
    background:#fff;
  }}
  .label {{ position:fixed; bottom:8px; left:12px; font-size:12px; opacity:0.6; }}
</style>
<div class="stage"><img id="f" alt=""></div>
<div class="label" id="l"></div>
<script>
  const frames = [
    {frames_js},
    {zoom_seq}
  ];
  const img = document.getElementById('f');
  const label = document.getElementById('l');
  let i = 0;
  function tick() {{
    const fr = frames[i];
    img.src = fr.src;
    img.style.transformOrigin = fr.ox + ' ' + fr.oy;
    img.style.transform = 'scale(' + fr.scale + ')';
    label.textContent = fr.src + (fr.scale > 1 ? '  (zoom ' + fr.scale + 'x)' : '');
    i = (i + 1) % frames.length;
    setTimeout(tick, fr.ms);
  }}
  tick();
</script>
"#,
        stem = stem,
        frames_js = frames_js,
        zoom_seq = zoom_seq
    );
    let html_path = out_dir.join(format!("{}_preview.html", stem));
    fs::write(&html_path, html)?;
    println!("preview: {}", html_path.display());

    println!("done — files in {}", out_dir.display());
    Ok(())
}
