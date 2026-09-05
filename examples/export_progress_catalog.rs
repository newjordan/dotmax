//! Export the progress-style catalog for the website — catalog v2.
//!
//! Replaces the single text-only `loading_bar_catalog.json` with:
//!
//! - `site/public/catalog/index.json` — metadata for every style, no frames.
//! - `site/public/catalog/themes/<theme>.json` — colored frame packs, one per
//!   theme, lazy-loaded by the site. Colors are palette-indexed (2 hex chars
//!   per cell, `..` = uncolored) to keep payloads small.
//! - `site/public/catalog/assets/<theme>.rs` — a standalone single-file Rust
//!   program per theme: the verbatim style sources wrapped around a minimal
//!   grid runtime. `rustc <theme>.rs && ./<theme> <style>` — no cargo, no deps.
//!
//! Run: `cargo run --example export_progress_catalog`

use dotmax::progress::{styles_for_theme, themes, BarContext, Easing, ProgressStyle};
use dotmax::BrailleGrid;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

/// An exact 8-bit RGB triple, used as the palette lookup key.
type Rgb = (u8, u8, u8);

/// A style's palette (hex strings, indexed) plus the exact-RGB -> index lookup.
type Palette = (Vec<String>, HashMap<Rgb, u8>);

const WIDTH: usize = 44;
const HEIGHT: usize = 4;
const FPS: u32 = 12;
const FRAMES: usize = 48;

/// Curated `(theme, style)` picks exported to `hero.json` for the site's
/// landing showcase, in rotation order. Every entry must match a registered
/// style or the export fails loudly.
const HERO_STYLES: &[(&str, &str)] = &[
    ("matrix", "rain-fill"),
    ("synthwave", "sunrise"),
    ("demoscene", "plasma"),
    ("aurora", "ribbon-flow"),
    ("glitch", "sync-lock"),
    ("inferno", "wildfire"),
    ("demoscene", "copper-bars"),
    ("synthwave", "neon-wave"),
    ("fireworks", "grand-finale"),
    ("fable", "phosphor-tide"),
    ("space", "black-hole"),
    ("fable", "murmuration"),
    ("cellular", "ca-brians-brain"),
    ("fable", "lantern-procession"),
    ("classic", "rocket"),
];

// ---------------------------------------------------------------------------
// JSON shapes
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CatalogIndex {
    version: u32,
    total: usize,
    width: usize,
    height: usize,
    fps: u32,
    frames_per_style: usize,
    themes: Vec<ThemeMeta>,
    styles: Vec<StyleMeta>,
}

#[derive(Serialize)]
struct ThemeMeta {
    name: String,
    count: usize,
    kind: &'static str,
}

#[derive(Serialize)]
struct StyleMeta {
    id: String,
    theme: String,
    name: String,
    kind: &'static str,
    description: String,
}

#[derive(Serialize)]
struct ThemePack {
    theme: String,
    width: usize,
    height: usize,
    fps: u32,
    frames_per_style: usize,
    styles: Vec<StylePack>,
}

#[derive(Serialize, Clone)]
struct StylePack {
    id: String,
    name: String,
    description: String,
    /// Hex colors (`#rrggbb`) indexed by the `c` strings in each frame.
    palette: Vec<String>,
    frames: Vec<FramePack>,
    /// The style's own Rust source (doc comment + struct + impl), for display.
    source: String,
}

#[derive(Serialize, Clone)]
struct FramePack {
    /// Text rows — one string of `width` chars per cell row.
    t: Vec<String>,
    /// Color rows — 2 hex chars per cell indexing `palette`, `..` = none.
    /// Omitted when the frame sets no colors.
    #[serde(skip_serializing_if = "Option::is_none")]
    c: Option<Vec<String>>,
}

/// Component kind a theme's styles belong to — bars are the default; the
/// border/wipe/spinner/meter themes are first-class non-bar families.
fn theme_kind(theme: &str) -> &'static str {
    match theme {
        "border" => "border",
        "spinner" => "spinner",
        "wipe" => "wipe",
        "meter" => "meter",
        _ => "bar",
    }
}

fn smooth_ping_pong_progress(frame: usize, frame_count: usize) -> f32 {
    let phase = frame as f32 / frame_count.max(1) as f32;
    0.5 - 0.5 * (phase * std::f32::consts::TAU).cos()
}

// ---------------------------------------------------------------------------
// Frame capture + palette-indexed encoding
// ---------------------------------------------------------------------------

struct RawFrame {
    rows: Vec<String>,
    colors: Vec<Vec<Option<(u8, u8, u8)>>>,
    has_color: bool,
}

fn capture_style_frames(style: &dyn ProgressStyle) -> Result<Vec<RawFrame>, dotmax::DotmaxError> {
    let mut frames = Vec::with_capacity(FRAMES);
    for frame_index in 0..FRAMES {
        let progress = smooth_ping_pong_progress(frame_index, FRAMES);
        let time = frame_index as f32 / FPS as f32;
        let ctx = BarContext::new(progress, time, WIDTH, HEIGHT)
            .with_easing(Easing::CubicInOut)
            .with_label(format!("{:.0}%", progress * 100.0));
        let mut grid = BrailleGrid::new(WIDTH, HEIGHT)?;
        style.render(&mut grid, &ctx)?;

        let mut rows = Vec::with_capacity(HEIGHT);
        let mut colors = Vec::with_capacity(HEIGHT);
        let mut has_color = false;
        for y in 0..HEIGHT {
            let mut row = String::with_capacity(WIDTH);
            let mut color_row = Vec::with_capacity(WIDTH);
            for x in 0..WIDTH {
                row.push(grid.get_char(x, y));
                let color = grid.get_color(x, y).map(|c| {
                    has_color = true;
                    (c.r, c.g, c.b)
                });
                color_row.push(color);
            }
            rows.push(row);
            colors.push(color_row);
        }
        frames.push(RawFrame {
            rows,
            colors,
            has_color,
        });
    }
    Ok(frames)
}

/// Build a ≤256-color palette for one style and a lookup from exact RGB to
/// palette index. Falls back to 5-bit channel quantization, then to
/// nearest-of-most-frequent if a style somehow exceeds 256 colors.
fn build_palette(frames: &[RawFrame]) -> Palette {
    let mut counts: HashMap<(u8, u8, u8), u32> = HashMap::new();
    let mut order: Vec<(u8, u8, u8)> = Vec::new();
    for frame in frames {
        for row in &frame.colors {
            for color in row.iter().flatten() {
                if !counts.contains_key(color) {
                    order.push(*color);
                }
                *counts.entry(*color).or_insert(0) += 1;
            }
        }
    }

    let quantize = |c: (u8, u8, u8)| (c.0 & 0xF8, c.1 & 0xF8, c.2 & 0xF8);

    // Level 0: exact colors fit.
    if order.len() <= 256 {
        let palette: Vec<String> = order
            .iter()
            .map(|c| format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2))
            .collect();
        let map = order
            .iter()
            .enumerate()
            .map(|(i, c)| (*c, i as u8))
            .collect();
        return (palette, map);
    }

    // Level 1: 5-bit quantization.
    let mut q_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();
    let mut q_order: Vec<(u8, u8, u8)> = Vec::new();
    for color in &order {
        let q = quantize(*color);
        if !q_counts.contains_key(&q) {
            q_order.push(q);
        }
        *q_counts.entry(q).or_insert(0) += counts[color];
    }

    // Level 2: keep the 256 most frequent quantized colors, map the rest to
    // their nearest survivor.
    if q_order.len() > 256 {
        q_order.sort_by_key(|q| std::cmp::Reverse(q_counts[q]));
        q_order.truncate(256);
    }

    let palette: Vec<String> = q_order
        .iter()
        .map(|c| format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2))
        .collect();
    let index_of: HashMap<(u8, u8, u8), u8> = q_order
        .iter()
        .enumerate()
        .map(|(i, c)| (*c, i as u8))
        .collect();
    let nearest = |c: (u8, u8, u8)| -> u8 {
        let mut best = 0u8;
        let mut best_d = u32::MAX;
        for (i, q) in q_order.iter().enumerate() {
            let d = (i32::from(c.0) - i32::from(q.0)).unsigned_abs().pow(2)
                + (i32::from(c.1) - i32::from(q.1)).unsigned_abs().pow(2)
                + (i32::from(c.2) - i32::from(q.2)).unsigned_abs().pow(2);
            if d < best_d {
                best_d = d;
                best = i as u8;
            }
        }
        best
    };
    let map = order
        .iter()
        .map(|c| {
            let q = quantize(*c);
            (*c, index_of.get(&q).copied().unwrap_or_else(|| nearest(q)))
        })
        .collect();
    (palette, map)
}

fn encode_frames(frames: &[RawFrame], map: &HashMap<(u8, u8, u8), u8>) -> Vec<FramePack> {
    frames
        .iter()
        .map(|frame| {
            let c = frame.has_color.then(|| {
                frame
                    .colors
                    .iter()
                    .map(|row| {
                        let mut encoded = String::with_capacity(row.len() * 2);
                        for color in row {
                            match color.and_then(|c| map.get(&c)) {
                                Some(index) => {
                                    let _ = write!(encoded, "{index:02x}");
                                }
                                None => encoded.push_str(".."),
                            }
                        }
                        encoded
                    })
                    .collect()
            });
            FramePack {
                t: frame.rows.clone(),
                c,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Source extraction (display source per style)
// ---------------------------------------------------------------------------

/// Find the end index of the brace block that starts at the first `{` at or
/// after `from`, skipping string/char literals and comments.
fn brace_block_end(src: &str, from: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = from + src[from..].find('{')?;
    let mut depth = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            b'"' => {
                // String literal: scan to unescaped closing quote.
                i += 1;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => i += 1,
                        b'"' => break,
                        _ => {}
                    }
                    i += 1;
                }
            }
            b'\'' => {
                // Char literal if it closes within a few bytes; lifetimes don't.
                let mut j = i + 1;
                if j < bytes.len() && bytes[j] == b'\\' {
                    j += 2;
                    while j < bytes.len() && bytes[j] != b'\'' {
                        j += 1;
                    }
                    i = j;
                } else {
                    // A (possibly multi-byte) char followed by a closing quote.
                    let rest = &src[i + 1..];
                    if let Some(ch) = rest.chars().next() {
                        let after = i + 1 + ch.len_utf8();
                        if after < bytes.len() && bytes[after] == b'\'' {
                            i = after;
                        }
                    }
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Find `needle` in `src` where the match is not followed by an identifier
/// character (so `Comet` does not match inside `CometTail`).
fn find_ident_bounded(src: &str, needle: &str, from: usize) -> Option<usize> {
    let mut start = from;
    while let Some(rel) = src[start..].find(needle) {
        let pos = start + rel;
        let end = pos + needle.len();
        let bounded = src.as_bytes().get(end).map_or(true, |b| !is_ident_char(*b));
        if bounded {
            return Some(pos);
        }
        start = end;
    }
    None
}

/// Struct names in registration order, parsed from `pub fn styles()`.
fn extract_struct_names(theme_src: &str) -> Vec<String> {
    let Some(fn_pos) = theme_src.find("pub fn styles()") else {
        return Vec::new();
    };
    let Some(end) = brace_block_end(theme_src, fn_pos) else {
        return Vec::new();
    };
    let body = &theme_src[fn_pos..end];
    let mut names = Vec::new();
    let mut at = 0;
    while let Some(rel) = body[at..].find("Box::new(") {
        let start = at + rel + "Box::new(".len();
        let mut name: String = body[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        // Unwrap one decorator level, e.g. `Box::new(Tinted(Rose))`.
        if body[start + name.len()..].starts_with('(') {
            let inner_start = start + name.len() + 1;
            name = body[inner_start..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
        }
        if !name.is_empty() {
            names.push(name);
        }
        at = start;
    }
    names
}

/// Extract the doc comment + struct declaration + `impl ProgressStyle` block
/// for one style struct.
fn extract_style_source(theme_src: &str, struct_name: &str) -> Option<String> {
    let decl_pos = find_ident_bounded(theme_src, &format!("struct {struct_name}"), 0)?;
    // Walk back over contiguous doc-comment / attribute lines.
    let mut start = theme_src[..decl_pos].rfind('\n').map_or(0, |p| p + 1);
    loop {
        let prev_line_start = theme_src[..start.saturating_sub(1)]
            .rfind('\n')
            .map_or(0, |p| p + 1);
        if prev_line_start >= start {
            break;
        }
        let line = theme_src[prev_line_start..start].trim_start();
        if line.starts_with("///") || line.starts_with("#[") {
            start = prev_line_start;
            if prev_line_start == 0 {
                break;
            }
        } else {
            break;
        }
    }
    let impl_pos = find_ident_bounded(
        theme_src,
        &format!("impl ProgressStyle for {struct_name}"),
        0,
    )?;
    let impl_end = brace_block_end(theme_src, impl_pos)?;
    Some(theme_src[start..impl_end].trim_end().to_string())
}

// ---------------------------------------------------------------------------
// Standalone single-file program assembly
// ---------------------------------------------------------------------------

/// Minimal grid runtime — API-compatible subset of `dotmax::BrailleGrid` with
/// identical braille bit mapping and `set_char` override semantics.
const RUNTIME_ROOT: &str = r"
// ===========================================================================
// Minimal runtime — a drop-in stand-in for the dotmax types the styles use.
// Identical braille dot mapping and glyph-override semantics to the crate.
// ===========================================================================

/// RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug)]
pub enum DotmaxError {
    OutOfBounds {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
}

/// A `width x height` cell canvas; every cell is 2x4 braille dots, an
/// optional glyph override, and an optional color.
pub struct BrailleGrid {
    width: usize,
    height: usize,
    patterns: Vec<u8>,
    characters: Vec<Option<char>>,
    colors: Vec<Option<Color>>,
}

impl BrailleGrid {
    pub fn new(width: usize, height: usize) -> Result<Self, DotmaxError> {
        let width = width.max(1);
        let height = height.max(1);
        Ok(Self {
            width,
            height,
            patterns: vec![0; width * height],
            characters: vec![None; width * height],
            colors: vec![None; width * height],
        })
    }

    #[must_use]
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_dot(&mut self, dot_x: usize, dot_y: usize) -> Result<(), DotmaxError> {
        if dot_x >= self.width * 2 || dot_y >= self.height * 4 {
            return Err(DotmaxError::OutOfBounds {
                x: dot_x,
                y: dot_y,
                width: self.width * 2,
                height: self.height * 4,
            });
        }
        let index = (dot_y / 4) * self.width + dot_x / 2;
        let bit = match (dot_x % 2, dot_y % 4) {
            (0, 0) => 0x01,
            (0, 1) => 0x02,
            (0, 2) => 0x04,
            (0, 3) => 0x40,
            (1, 0) => 0x08,
            (1, 1) => 0x10,
            (1, 2) => 0x20,
            _ => 0x80,
        };
        self.patterns[index] |= bit;
        Ok(())
    }

    pub fn set_char(&mut self, x: usize, y: usize, character: char) -> Result<(), DotmaxError> {
        self.check_cell(x, y)?;
        self.characters[y * self.width + x] = Some(character);
        Ok(())
    }

    pub fn set_cell_color(&mut self, x: usize, y: usize, color: Color) -> Result<(), DotmaxError> {
        self.check_cell(x, y)?;
        self.colors[y * self.width + x] = Some(color);
        Ok(())
    }

    pub fn enable_color_support(&mut self) {}

    #[must_use]
    pub fn get_char(&self, x: usize, y: usize) -> char {
        if x >= self.width || y >= self.height {
            return '\u{2800}';
        }
        let index = y * self.width + x;
        if let Some(ch) = self.characters[index] {
            return ch;
        }
        char::from_u32(0x2800 + u32::from(self.patterns[index])).unwrap_or('\u{2800}')
    }

    #[must_use]
    pub fn get_color(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.colors[y * self.width + x]
    }

    fn check_cell(&self, x: usize, y: usize) -> Result<(), DotmaxError> {
        if x >= self.width || y >= self.height {
            return Err(DotmaxError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        Ok(())
    }
}
";

fn build_main(theme: &str) -> String {
    format!(
        r#"
fn main() {{
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_STYLE.to_string());
    let styles = progress::styles::{theme}::styles();
    let Some(style) = styles.iter().find(|s| s.name() == name) else {{
        eprintln!("unknown style '{{name}}'. available in this file:");
        for s in &styles {{
            eprintln!("  {{:<18}} {{}}", s.name(), s.describe());
        }}
        std::process::exit(1);
    }};

    let (width, height) = (44usize, 4usize);
    let fps = 12u64;
    let loop_frames = 96u64;
    println!("{{}} - {{}}  (ctrl-c to quit)", style.name(), style.describe());
    let mut frame = 0u64;
    loop {{
        let phase = (frame % loop_frames) as f32 / loop_frames as f32;
        let progress = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
        let time = frame as f32 / fps as f32;
        let ctx = progress::BarContext::new(progress, time, width, height)
            .with_easing(progress::Easing::CubicInOut)
            .with_label(format!("{{:.0}}%", progress * 100.0));
        let mut grid = BrailleGrid::new(width, height).expect("grid");
        style.render(&mut grid, &ctx).expect("render");

        let mut out = String::new();
        for y in 0..height {{
            let mut current: Option<Color> = None;
            for x in 0..width {{
                let color = grid.get_color(x, y);
                if color != current {{
                    match color {{
                        Some(c) => out.push_str(&format!("\x1b[38;2;{{}};{{}};{{}}m", c.r, c.g, c.b)),
                        None => out.push_str("\x1b[0m"),
                    }}
                    current = color;
                }}
                out.push(grid.get_char(x, y));
            }}
            out.push_str("\x1b[0m\n");
        }}
        print!("{{out}}");
        use std::io::Write as _;
        std::io::stdout().flush().expect("flush");
        std::thread::sleep(std::time::Duration::from_millis(1000 / fps));
        print!("\x1b[{{height}}A");
        frame += 1;
    }}
}}
"#,
        theme = theme,
    )
}

/// Remove every `#[cfg(test)]` block (attribute through matching close brace).
fn strip_tests(src: &str) -> String {
    let mut out = src.to_string();
    while let Some(pos) = out.find("#[cfg(test)]") {
        let end = brace_block_end(&out, pos).unwrap_or(out.len());
        out.replace_range(pos..end, "");
    }
    out
}

/// Assemble one self-contained `.rs` program: runtime + the crate's verbatim
/// `progress` core (easing + BarContext/Palette/draw) + the theme's verbatim
/// style file + a `main` that animates a chosen style with ANSI color.
fn build_standalone(
    theme: &str,
    theme_src: &str,
    mod_src: &str,
    easing_src: &str,
    default_style: &str,
) -> Result<String, String> {
    // Surgery on src/progress/mod.rs: inline easing, inline the theme file in
    // place of the registry, drop the registry re-export, drop tests.
    let mut core = strip_tests(mod_src);
    let easing_src = strip_tests(easing_src);
    let theme_src = strip_tests(theme_src);

    let easing_inline = format!("pub mod easing {{\n{easing_src}\n}}");
    if !core.contains("pub mod easing;") {
        return Err("marker `pub mod easing;` not found in progress/mod.rs".into());
    }
    core = core.replace("pub mod easing;", &easing_inline);

    let styles_inline =
        format!("pub mod styles {{\n    pub mod {theme} {{\n{theme_src}\n    }}\n}}");
    if !core.contains("mod styles;") {
        return Err("marker `mod styles;` not found in progress/mod.rs".into());
    }
    core = core.replace("mod styles;", &styles_inline);
    core = core.replace(
        "pub use styles::{all_styles, styles_for_theme, themes};",
        "",
    );

    let mut out = String::new();
    let _ = write!(
        out,
        "//! `{theme}` — dotmax progress styles as a standalone, dependency-free program.\n\
         //!\n\
         //! Generated from https://github.com/newjordan/dotmax (MIT OR Apache-2.0).\n\
         //! The style code below is the crate's verbatim source; only the small\n\
         //! grid runtime at the top replaces the dotmax crate.\n\
         //!\n\
         //! Build and run (no cargo needed):\n\
         //!\n\
         //! ```sh\n\
         //! rustc -O {theme}.rs && ./{theme} [style-name]\n\
         //! ```\n\n"
    );
    let _ = writeln!(out, "const DEFAULT_STYLE: &str = \"{default_style}\";");
    out.push_str(RUNTIME_ROOT);
    out.push_str("\npub mod progress {\n");
    out.push_str(&core);
    out.push_str("\n}\n");
    out.push_str(&build_main(theme));
    Ok(out)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let styles_dir = manifest.join("src/progress/styles");
    let out_dir = manifest.join("site/public/catalog");
    fs::create_dir_all(out_dir.join("themes"))?;
    fs::create_dir_all(out_dir.join("assets"))?;

    let mod_src = fs::read_to_string(manifest.join("src/progress/mod.rs"))?;
    let easing_src = fs::read_to_string(manifest.join("src/progress/easing.rs"))?;

    let mut index_themes = Vec::new();
    let mut index_styles = Vec::new();
    let mut total = 0usize;
    let mut total_bytes = 0u64;
    let mut hero_packs: Vec<(usize, StylePack)> = Vec::new();

    for theme in themes() {
        let theme_styles = styles_for_theme(theme);
        let theme_src = fs::read_to_string(styles_dir.join(format!("{theme}.rs")))?;
        let struct_names = extract_struct_names(&theme_src);
        if struct_names.len() != theme_styles.len() {
            return Err(format!(
                "theme '{theme}': parsed {} struct names but registry has {} styles",
                struct_names.len(),
                theme_styles.len()
            )
            .into());
        }

        let mut packs = Vec::with_capacity(theme_styles.len());
        for (style, struct_name) in theme_styles.iter().zip(&struct_names) {
            let frames = capture_style_frames(style.as_ref())?;
            let (palette, map) = build_palette(&frames);
            let source = extract_style_source(&theme_src, struct_name).ok_or_else(|| {
                format!("theme '{theme}': could not extract source for struct {struct_name}")
            })?;
            packs.push(StylePack {
                id: format!("{theme}/{}", style.name()),
                name: style.name().to_string(),
                description: style.describe().to_string(),
                palette,
                frames: encode_frames(&frames, &map),
                source,
            });
            // Clone curated picks into the hero pack, source stripped —
            // the landing showcase never renders source; the dialog loads
            // the real theme pack for that.
            if let Some(pos) = HERO_STYLES
                .iter()
                .position(|&(t, n)| t == *theme && n == style.name())
            {
                let mut hero = packs.last().expect("just pushed").clone();
                hero.source = String::new();
                hero_packs.push((pos, hero));
            }
            index_styles.push(StyleMeta {
                id: format!("{theme}/{}", style.name()),
                theme: (*theme).to_string(),
                name: style.name().to_string(),
                kind: theme_kind(theme),
                description: style.describe().to_string(),
            });
        }

        let pack = ThemePack {
            theme: (*theme).to_string(),
            width: WIDTH,
            height: HEIGHT,
            fps: FPS,
            frames_per_style: FRAMES,
            styles: packs,
        };
        let pack_path = out_dir.join("themes").join(format!("{theme}.json"));
        fs::write(&pack_path, serde_json::to_string(&pack)?)?;
        total_bytes += fs::metadata(&pack_path)?.len();

        let default_style = theme_styles[0].name().to_string();
        let standalone =
            build_standalone(theme, &theme_src, &mod_src, &easing_src, &default_style)?;
        fs::write(
            out_dir.join("assets").join(format!("{theme}.rs")),
            standalone,
        )?;

        index_themes.push(ThemeMeta {
            name: (*theme).to_string(),
            count: theme_styles.len(),
            kind: theme_kind(theme),
        });
        total += theme_styles.len();
        println!(
            "{theme}: {} styles -> themes/{theme}.json ({} KB), assets/{theme}.rs",
            theme_styles.len(),
            fs::metadata(&pack_path)?.len() / 1024,
        );
    }

    let index = CatalogIndex {
        version: 2,
        total,
        width: WIDTH,
        height: HEIGHT,
        fps: FPS,
        frames_per_style: FRAMES,
        themes: index_themes,
        styles: index_styles,
    };
    let index_path = out_dir.join("index.json");
    fs::write(&index_path, serde_json::to_string(&index)?)?;

    // Hero pack: the curated landing-page showcase, in HERO_STYLES order.
    if hero_packs.len() != HERO_STYLES.len() {
        let found: Vec<&str> = hero_packs.iter().map(|(_, p)| p.id.as_str()).collect();
        return Err(format!(
            "hero pack: matched {}/{} curated styles ({found:?}) — check HERO_STYLES entries",
            hero_packs.len(),
            HERO_STYLES.len()
        )
        .into());
    }
    hero_packs.sort_by_key(|(pos, _)| *pos);
    let hero = ThemePack {
        theme: "hero".to_string(),
        width: WIDTH,
        height: HEIGHT,
        fps: FPS,
        frames_per_style: FRAMES,
        styles: hero_packs.into_iter().map(|(_, p)| p).collect(),
    };
    let hero_path = out_dir.join("hero.json");
    fs::write(&hero_path, serde_json::to_string(&hero)?)?;
    println!(
        "hero: {} curated styles -> hero.json ({} KB)",
        HERO_STYLES.len(),
        fs::metadata(&hero_path)?.len() / 1024,
    );

    println!(
        "\ncatalog v2: {total} styles, {} themes, {:.1} MB of theme packs, index.json {} KB",
        themes().len(),
        total_bytes as f64 / (1024.0 * 1024.0),
        fs::metadata(&index_path)?.len() / 1024,
    );
    Ok(())
}
