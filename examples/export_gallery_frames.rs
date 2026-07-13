//! Export compact static frame packs for the website mini-terminal gallery.
//!
//! Run with:
//!   cargo run --example export_gallery_frames

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use dotmax::color::schemes::{cyan_magenta, heat_map, rainbow};
use dotmax::primitives::{
    draw_circle, draw_line, draw_line_colored, draw_polygon, draw_rectangle, draw_rectangle_filled,
};
use dotmax::progress::{all_styles, render_lines, themes, BarContext, Easing};
use dotmax::{
    capture_frame, write_frame_pack, BrailleGrid, Color, DotmaxError, DotmaxFrame, DotmaxFramePack,
};
use serde::Serialize;
use std::f64::consts::PI;
use std::fs;
use std::fs::File;
use std::path::Path;

const WIDTH: usize = 36;
const HEIGHT: usize = 12;
const FRAMES: u64 = 24;
const LOADING_BAR_FRAMES: u64 = 48;
const LOADING_BAR_FPS: u32 = 4;
const BAR_CATALOG_WIDTH: usize = 44;
const BAR_CATALOG_HEIGHT: usize = 4;
const BAR_CATALOG_FPS: u32 = 4;
const BAR_CATALOG_FRAMES: usize = 32;

#[derive(Serialize)]
struct LoadingBarThemeSummary {
    name: String,
    count: usize,
}

#[derive(Serialize)]
struct LoadingBarStylePreview {
    id: String,
    theme: String,
    name: String,
    description: String,
    command: String,
    frames: Vec<Vec<String>>,
}

#[derive(Serialize)]
struct LoadingBarCatalog {
    total: usize,
    width: usize,
    height: usize,
    fps: u32,
    frames_per_style: usize,
    themes: Vec<LoadingBarThemeSummary>,
    styles: Vec<LoadingBarStylePreview>,
}

fn set_dot_color(
    grid: &mut BrailleGrid,
    x: usize,
    y: usize,
    color: Color,
) -> Result<(), DotmaxError> {
    if x < grid.dot_width() && y < grid.dot_height() {
        grid.set_dot(x, y)?;
        grid.set_cell_color(x / 2, y / 4, color)?;
    }
    Ok(())
}

fn write_text(
    grid: &mut BrailleGrid,
    x: usize,
    y: usize,
    text: &str,
    color: Color,
) -> Result<(), DotmaxError> {
    for (offset, character) in text.chars().enumerate() {
        let cell_x = x + offset;
        if cell_x < grid.width() && y < grid.height() {
            grid.set_char(cell_x, y, character)?;
            grid.set_cell_color(cell_x, y, color)?;
        }
    }
    Ok(())
}

fn simple_animation_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let dot_width = grid.dot_width();
    let dot_height = grid.dot_height();
    let x = ((frame * 5) % dot_width as u64) as usize;
    let y = ((dot_height as f64 / 2.0) + (frame as f64 * 0.35).sin() * 14.0) as usize;

    for trail in 0..7 {
        let tx = x.saturating_sub(trail * 3);
        let intensity = (255usize.saturating_sub(trail * 28)) as u8;
        set_dot_color(&mut grid, tx, y, Color::rgb(104, intensity, 138))?;
        if y + 1 < dot_height {
            set_dot_color(&mut grid, tx, y + 1, Color::rgb(40, intensity, 220))?;
        }
    }
    draw_line(&mut grid, 0, dot_height as i32 - 5, dot_width as i32, 5)?;
    write_text(
        &mut grid,
        1,
        1,
        "simple animation",
        Color::rgb(215, 255, 224),
    )?;

    Ok(capture_frame(&grid))
}

fn loading_spinner_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let center_x = grid.dot_width() / 2;
    let center_y = grid.dot_height() / 2;
    let angle = frame as f64 * 0.45;

    for step in 0..16 {
        let local = angle + f64::from(step) * PI / 8.0;
        let radius = 9.0 + f64::from(step % 4);
        let x = center_x as f64 + local.cos() * radius;
        let y = center_y as f64 + local.sin() * radius;
        let fade = 255u8.saturating_sub((step * 12) as u8);
        if x >= 0.0 && y >= 0.0 {
            set_dot_color(
                &mut grid,
                x as usize,
                y as usize,
                Color::rgb(104, fade, 180),
            )?;
        }
    }

    let dots = ".".repeat((frame as usize % 4) + 1);
    write_text(
        &mut grid,
        10,
        9,
        &format!("loading{dots:<4}"),
        Color::rgb(215, 255, 224),
    )?;

    Ok(capture_frame(&grid))
}

fn waveform_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let scheme = rainbow();
    let dot_width = grid.dot_width();
    let dot_height = grid.dot_height();
    let center_y = dot_height as f64 / 2.0;
    let phase = frame as f64 * 0.32;

    for y in [6usize, dot_height / 2, dot_height - 7] {
        for x in (0..dot_width).step_by(4) {
            set_dot_color(&mut grid, x, y, Color::rgb(42, 66, 76))?;
        }
    }

    for wave in 0..3 {
        let amplitude = 7.0 + f64::from(wave) * 4.5;
        let frequency = 0.10 + f64::from(wave) * 0.035;
        let color = scheme.sample(0.18 + wave as f32 * 0.26);
        let mut previous = None;

        for x in 0..dot_width {
            let y = center_y
                + (x as f64 * frequency + phase * (1.0 + wave as f64 * 0.4)).sin() * amplitude;
            let y = y.clamp(0.0, (dot_height - 1) as f64) as i32;
            if let Some((px, py)) = previous {
                draw_line_colored(&mut grid, px, py, x as i32, y, color, None)?;
            }
            previous = Some((x as i32, y));
        }
    }

    Ok(capture_frame(&grid))
}

fn fireworks_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let centers = [(18.0, 16.0), (50.0, 22.0), (34.0, 11.0)];
    let scheme = heat_map();

    for (burst, (cx, cy)) in centers.into_iter().enumerate() {
        let age = ((frame + burst as u64 * 8) % FRAMES) as f64;
        let radius = 1.0 + age * 0.62;
        for particle in 0..24 {
            let angle = (particle as f64 / 24.0) * 2.0 * PI + burst as f64 * 0.4;
            let wobble = ((frame + particle) as f64 * 0.3).sin() * 1.4;
            let x = cx + angle.cos() * (radius + wobble);
            let y = cy + angle.sin() * (radius * 0.62 + wobble) + age * 0.12;
            let fade = (1.0 - age / FRAMES as f64).clamp(0.15, 1.0) as f32;
            let color = scheme.sample(fade);
            if x >= 0.0 && y >= 0.0 {
                set_dot_color(&mut grid, x as usize, y as usize, color)?;
            }
        }
    }

    write_text(
        &mut grid,
        1,
        10,
        "fireworks particles",
        Color::rgb(215, 255, 224),
    )?;
    Ok(capture_frame(&grid))
}

fn shapes_demo_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let dot_width = grid.dot_width();
    let dot_height = grid.dot_height();
    draw_rectangle(&mut grid, 3, 4, 18, 14)?;
    draw_rectangle_filled(&mut grid, 27, 7, 14, 10)?;
    draw_circle(&mut grid, 55, 18, 9)?;
    draw_polygon(
        &mut grid,
        &[(8, 34), (20, 24), (33, 36), (25, 44), (12, 43)],
    )?;
    draw_polygon(
        &mut grid,
        &[(48, 32), (54, 22), (60, 32), (70, 35), (59, 39)],
    )?;

    let sweep = ((frame * 3) % dot_width as u64) as i32;
    draw_line_colored(
        &mut grid,
        sweep,
        0,
        dot_width as i32 - sweep,
        dot_height as i32 - 1,
        Color::rgb(0, 220, 255),
        None,
    )?;
    write_text(&mut grid, 1, 1, "primitives", Color::rgb(215, 255, 224))?;

    Ok(capture_frame(&grid))
}

fn color_schemes_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let schemes = [rainbow(), heat_map(), cyan_magenta()];

    for (row, scheme) in schemes.iter().enumerate() {
        let y = row * 3 + 2;
        for x in 1..(WIDTH - 1) {
            let shifted = ((x as f32 / (WIDTH - 2) as f32) + frame as f32 * 0.02).fract();
            let color = scheme.sample(shifted);
            grid.set_char(x, y, '█')?;
            grid.set_cell_color(x, y, color)?;
            grid.set_char(x, y + 1, '▄')?;
            grid.set_cell_color(x, y + 1, color)?;
        }
    }

    write_text(&mut grid, 1, 0, "color schemes", Color::rgb(215, 255, 224))?;
    Ok(capture_frame(&grid))
}

fn smooth_ping_pong_progress(frame: usize, frame_count: usize) -> f32 {
    let phase = frame as f32 / frame_count.max(1) as f32;
    0.5 - 0.5 * (phase * std::f32::consts::TAU).cos()
}

fn loading_bars_frame(frame: u64) -> Result<DotmaxFrame, DotmaxError> {
    let progress = smooth_ping_pong_progress(frame as usize, LOADING_BAR_FRAMES as usize);

    let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
    let scheme = cyan_magenta();
    let bar_left = 3usize;
    let bar_width = WIDTH - bar_left * 2;
    let filled = (bar_width as f32 * progress).round() as usize;

    for row in [3usize, 5, 7] {
        for x in 0..bar_width {
            let cell_x = bar_left + x;
            let active = x <= filled;
            let pulse = ((x as u64 + frame) % 10) as f32 / 10.0;
            let color = if active {
                scheme.sample((progress + pulse * 0.35).fract())
            } else {
                Color::rgb(34, 42, 48)
            };
            grid.set_char(cell_x, row, if active { '█' } else { '░' })?;
            grid.set_cell_color(cell_x, row, color)?;
        }
    }

    write_text(
        &mut grid,
        1,
        10,
        &format!("loading bars {:>3.0}%", progress * 100.0),
        Color::rgb(215, 255, 224),
    )?;
    Ok(capture_frame(&grid))
}

fn build_frames(
    render: fn(u64) -> Result<DotmaxFrame, DotmaxError>,
) -> Result<Vec<DotmaxFrame>, DotmaxError> {
    (0..FRAMES).map(render).collect()
}

fn build_frame_count(
    count: u64,
    render: fn(u64) -> Result<DotmaxFrame, DotmaxError>,
) -> Result<Vec<DotmaxFrame>, DotmaxError> {
    (0..count).map(render).collect()
}

fn write_pack(
    output_dir: &Path,
    file_name: &str,
    name: &str,
    command: &str,
    fps: u32,
    frames: Vec<DotmaxFrame>,
) -> Result<(), DotmaxError> {
    let pack = DotmaxFramePack::new(name, command, WIDTH, HEIGHT, fps, frames);
    write_frame_pack(output_dir.join(file_name), &pack)?;
    println!("wrote {file_name}");
    Ok(())
}

fn write_loading_bar_catalog(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let styles = all_styles();
    let mut previews = Vec::with_capacity(styles.len());
    let mut theme_counts = themes()
        .iter()
        .map(|theme| LoadingBarThemeSummary {
            name: (*theme).to_string(),
            count: 0,
        })
        .collect::<Vec<_>>();

    for style in &styles {
        if let Some(summary) = theme_counts
            .iter_mut()
            .find(|summary| summary.name == style.theme())
        {
            summary.count += 1;
        }

        let frames = (0..BAR_CATALOG_FRAMES)
            .map(|frame_index| {
                let progress = smooth_ping_pong_progress(frame_index, BAR_CATALOG_FRAMES);
                let time = frame_index as f32 / BAR_CATALOG_FPS as f32;
                let label = format!("{:.0}%", progress * 100.0);
                let context =
                    BarContext::new(progress, time, BAR_CATALOG_WIDTH, BAR_CATALOG_HEIGHT)
                        .with_easing(Easing::CubicInOut)
                        .with_label(label);
                render_lines(style.as_ref(), &context)
            })
            .collect::<Result<Vec<_>, _>>()?;

        previews.push(LoadingBarStylePreview {
            id: format!("{}/{}", style.theme(), style.name()),
            theme: style.theme().to_string(),
            name: style.name().to_string(),
            description: style.describe().to_string(),
            command: format!("cargo run --example loading_bars -- {}", style.name()),
            frames,
        });
    }

    let catalog = LoadingBarCatalog {
        total: previews.len(),
        width: BAR_CATALOG_WIDTH,
        height: BAR_CATALOG_HEIGHT,
        fps: BAR_CATALOG_FPS,
        frames_per_style: BAR_CATALOG_FRAMES,
        themes: theme_counts,
        styles: previews,
    };

    let file = File::create(output_dir.join("loading_bar_catalog.json"))?;
    serde_json::to_writer_pretty(file, &catalog)?;
    println!(
        "wrote loading_bar_catalog.json with {} styles",
        catalog.total
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new("site/public/examples");
    fs::create_dir_all(output_dir)?;

    write_pack(
        output_dir,
        "simple_animation.json",
        "Simple Animation",
        "cargo run --example simple_animation",
        18,
        build_frames(simple_animation_frame)?,
    )?;
    write_pack(
        output_dir,
        "loading_spinner.json",
        "Loading Spinner",
        "cargo run --example loading_spinner",
        14,
        build_frames(loading_spinner_frame)?,
    )?;
    write_pack(
        output_dir,
        "waveform.json",
        "Waveform",
        "cargo run --example waveform",
        18,
        build_frames(waveform_frame)?,
    )?;
    write_pack(
        output_dir,
        "fireworks.json",
        "Fireworks",
        "cargo run --example fireworks",
        18,
        build_frames(fireworks_frame)?,
    )?;
    write_pack(
        output_dir,
        "shapes_demo.json",
        "Shapes Demo",
        "cargo run --example shapes_demo",
        10,
        build_frames(shapes_demo_frame)?,
    )?;
    write_pack(
        output_dir,
        "color_schemes_demo.json",
        "Color Schemes Demo",
        "cargo run --example color_schemes_demo",
        10,
        build_frames(color_schemes_frame)?,
    )?;
    write_pack(
        output_dir,
        "loading_bars.json",
        "Loading Bars",
        "cargo run --example loading_bars",
        LOADING_BAR_FPS,
        build_frame_count(LOADING_BAR_FRAMES, loading_bars_frame)?,
    )?;
    write_loading_bar_catalog(output_dir)?;

    println!(
        "Wrote dotmax website frame packs to {}",
        output_dir.display()
    );
    Ok(())
}
