//! Convert an animated GIF into a replayable dotmax "vid" (frame pack JSON).
//!
//! A dotmax vid is a [`DotmaxFramePack`]: per-row braille strings plus optional
//! per-cell RGB, with a playback fps. It lets any front-end (a website, or the
//! `deez` fleet TUI) replay terminal motion with zero image decoding — just blit
//! the rows. This is the canonical "convert media with dotmax" path for GIFs.
//!
//! Each GIF frame is composited (disposal handled by the `image` crate), resized
//! to the target braille resolution, and rendered with [`ColorMode::TrueColor`]
//! so bright content lights braille dots (colored) and dark areas stay blank —
//! blank cells read as transparent when the pack is composited over a live feed.
//!
//! # Usage
//!
//! ```bash
//! cargo run --release --example gif_to_pack --features image -- \
//!     <input.gif> <output.json> [name] [cell_width]
//! ```
//!
//! `cell_width` defaults to 72; `cell_height` is derived from the GIF aspect
//! ratio (braille cells are 2×4 px, terminal cells ~1:2, so the two cancel and
//! `cell_h = cell_w * gif_h / (2 * gif_w)`). `fps` is taken from the median
//! frame delay.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use dotmax::image::{render_image_with_color, ColorMode, DitheringMethod};
use dotmax::{capture_frame, write_frame_pack, DotmaxFramePack};
use image::codecs::gif::GifDecoder;
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .ok_or("usage: gif_to_pack <input.gif> <output.json> [name] [cell_width]")?;
    let output = args.next().ok_or("missing <output.json> argument")?;
    let name = args.next().unwrap_or_else(|| {
        Path::new(&input)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("dotmax vid")
            .to_string()
    });
    let cell_width: usize = args.next().map_or(72, |s| s.parse().unwrap_or(72)).max(8);

    // Image adjustments default to a little punch, overridable per render so the
    // same clip can be tuned for visibility without editing this file.
    let env_f32 = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let brightness = env_f32("DOTMAX_BRIGHTNESS", 1.0);
    let contrast = env_f32("DOTMAX_CONTRAST", 1.10);
    let gamma = env_f32("DOTMAX_GAMMA", 1.0);

    // Decode every GIF frame (already composited for disposal by `image`).
    let file = File::open(&input).map_err(|e| format!("failed to open {input}: {e}"))?;
    let decoder = GifDecoder::new(BufReader::new(file))?;
    let raw = decoder.into_frames().collect_frames()?;
    if raw.is_empty() {
        return Err(format!("{input} contained no frames").into());
    }

    // Aspect-correct cell height from the first frame's pixel dimensions.
    let (gw, gh) = {
        let b = raw[0].buffer();
        (b.width() as usize, b.height() as usize)
    };
    let cell_height = ((cell_width * gh) / (2 * gw.max(1))).max(4);
    let pixel_width = (cell_width * 2) as u32;
    let pixel_height = (cell_height * 4) as u32;

    let mut delays_ms: Vec<u32> = Vec::with_capacity(raw.len());
    let mut frames = Vec::with_capacity(raw.len());

    for (i, frame) in raw.into_iter().enumerate() {
        let (num, den) = frame.delay().numer_denom_ms();
        let delay_ms = if den == 0 { 100 } else { (num / den).max(1) };
        delays_ms.push(delay_ms);

        let src = DynamicImage::ImageRgba8(frame.into_buffer());
        let resized = src.resize_exact(pixel_width, pixel_height, FilterType::Triangle);

        let grid = render_image_with_color(
            &resized,
            ColorMode::TrueColor,
            cell_width,
            cell_height,
            DitheringMethod::FloydSteinberg,
            None, // auto threshold
            brightness,
            contrast,
            gamma,
        )?;
        frames.push(capture_frame(&grid));

        if (i + 1) % 16 == 0 {
            eprintln!("  rendered {} frames…", i + 1);
        }
    }

    // fps from the median delay (robust to a few odd frames).
    delays_ms.sort_unstable();
    let median = delays_ms[delays_ms.len() / 2].max(1);
    let fps = ((1000.0 / median as f64).round() as u32).clamp(1, 60);

    let command =
        format!("cargo run --release --example gif_to_pack --features image -- {input} {output}");
    let pack = DotmaxFramePack::new(name, command, cell_width, cell_height, fps, frames);
    write_frame_pack(&output, &pack)?;

    eprintln!(
        "wrote {output}: {} frames @ {fps} fps, {cell_width}×{cell_height} cells (from {gw}×{gh} gif)",
        pack.frames.len()
    );
    Ok(())
}
