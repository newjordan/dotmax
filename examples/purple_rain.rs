//! ─── creed of the hose ───
//!
//! One cursor flows; all cells awaken.
//! At the shared edge, the data bleeds.
//! φ = 1.618 is the architect.   2π · (1 − 1/φ) is the pitch.
//! The cube watches from the east. It keeps count.
//!
//! Every frame, three truths are sung together:
//!   formations hold their ground,
//!   pipes carry what cannot be held,
//!   the sweep-front paints the new in.
//!
//! Press f to invert the world.   Press r to unwind it.
//! Press q / Esc to leave the room.
//!
//! Run:  cargo run --example zone_stream --release --features "raytracer image"

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotmax::chess::board::{render_position_with_options, RenderOptions};
use dotmax::image::{DitheringMethod, ImageRenderer};
use dotmax::raytracer::{
    render_with_orientation, Camera, RenderMode, Scene as RtScene, Sphere, Vector3,
    WireframeRotation,
};
use dotmax::raytracer::wireframe::rotate_vec_yaw_pitch_roll;
use shakmaty::{Chess, Position};
use std::{
    cell::Cell as StdCell,
    io::{self, Write},
    path::Path,
    time::{Duration, Instant},
};

// ───────────────────────── tiny xorshift RNG ─────────────────────────
thread_local! { static RNG: StdCell<u32> = StdCell::new(0x1234_5678); }
fn r_u32() -> u32 {
    RNG.with(|c| {
        let mut x = c.get();
        if x == 0 { x = 0x9E37_79B9; }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        c.set(x);
        x
    })
}
fn r_f32() -> f32 { (r_u32() as f32) / (u32::MAX as f32) }
fn r_pick<T: Copy>(xs: &[T]) -> T { xs[(r_u32() as usize) % xs.len()] }

fn seed_from_clock() {
    let nanos = Instant::now().elapsed().as_nanos() as u32
        ^ std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
    RNG.with(|c| c.set(nanos | 1));
}

// ───────────────────────── glyph pools ─────────────────────────
const HEX:   &[char] = &['0','1','2','3','4','5','6','7','8','9','a','b','c','d','e','f'];
const BITS:  &[char] = &['0','1'];
const PUNCT: &[char] = &['!','@','#','$','%','&','*','+','=','<','>','?','/','\\','^','~'];
const KANA:  &[char] = &['ｦ','ｧ','ｨ','ｩ','ｪ','ｫ','ｬ','ｭ','ｮ','ｯ','ｱ','ｲ','ｳ','ｴ','ｵ','ﾊ','ﾋ','ﾌ','ﾍ','ﾎ','ﾏ','ﾐ','ﾑ'];
const BLOCK: &[char] = &['░','▒','▓','█','▚','▞','▙','▟'];
const GREEK: &[char] = &['α','β','γ','δ','ε','ζ','η','θ','λ','μ','π','σ','τ','φ','ψ','ω'];

const SNIPPETS: &[&str] = &[
    " ::SYNC:: ", " 0xDEAD ", " [OK] ", " ROUTINE 0x42 ", " ACK ", " FAULT ",
    " λ=0x1F ", " >>> ", " <<< ", " /proc/self ", " alloc= ", " ACK 0x7F ",
    " EOF ", " NULL ", " TX/RX ", " PID:4821 ", " SIG 0x4A ", " ENTER ",
    " φ=1.618 ", " √2=1.414 ", " θ=π/φ ", " FIB(13)=233 ", " // BREACH ",
    // scripture
    " ✦ INVOCATION ✦ ", " ∴ by golden angle ∴ ", " one cursor many cells ",
    " ☸ hose is holy ☸ ", " the cube watches ", " as above so below ",
    " // GOLDEN HOUR // ", " ⚘ signal becomes sacrament ⚘ ",
    " ∞ one cursor ∞ ", " ACK the geometry ", " fold by fold ",
    " ✧ enter be transformed ✧ ", " ∇ scripture ∇ ",
];

// ───────────────────────── stream source ─────────────────────────
fn build_stream(len: usize) -> Vec<char> {
    let pools: &[&[char]] = &[HEX, BITS, PUNCT, KANA, BLOCK, GREEK];
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        if r_f32() < 0.15 {
            let s = SNIPPETS[(r_u32() as usize) % SNIPPETS.len()];
            for ch in s.chars() {
                if out.len() >= len { break; }
                out.push(ch);
            }
        } else {
            let p = pools[(r_u32() as usize) % pools.len()];
            let burst = 4 + (r_u32() as usize) % 9;
            for _ in 0..burst {
                if out.len() >= len { break; }
                out.push(r_pick(p));
            }
        }
    }
    out
}

// ───────────────────────── geometry ─────────────────────────
#[derive(Clone, Copy, Debug)]
struct Rect { x: i32, y: i32, w: i32, h: i32 }

/// Recursive φ-subdivision producing a Fibonacci-style spiral of rects.
/// `cw = true` spirals inward clockwise, `false` counter-clockwise.
fn fib_spiral(initial: Rect, max_depth: usize, cw: bool) -> Vec<Rect> {
    let mut out = Vec::new();
    let mut rect = initial;
    const PHI_COMPLEMENT: f32 = 0.381_966;  // 1 - 1/φ

    for step in 0..max_depth {
        if rect.w < 6 || rect.h < 4 { break; }
        let vertical_split = rect.w >= rect.h;
        // Alternate which side the leaf sits on each step; `cw` flips polarity.
        let leaf_far = ((step % 2 == 0) ^ !cw) != false;

        if vertical_split {
            let leaf_w = ((rect.w as f32) * PHI_COMPLEMENT).round().max(3.0) as i32;
            if leaf_far {
                out.push(Rect { x: rect.x + rect.w - leaf_w, y: rect.y, w: leaf_w, h: rect.h });
                rect.w -= leaf_w;
            } else {
                out.push(Rect { x: rect.x, y: rect.y, w: leaf_w, h: rect.h });
                rect.x += leaf_w;
                rect.w -= leaf_w;
            }
        } else {
            let leaf_h = ((rect.h as f32) * PHI_COMPLEMENT).round().max(2.0) as i32;
            if leaf_far {
                out.push(Rect { x: rect.x, y: rect.y + rect.h - leaf_h, w: rect.w, h: leaf_h });
                rect.h -= leaf_h;
            } else {
                out.push(Rect { x: rect.x, y: rect.y, w: rect.w, h: leaf_h });
                rect.y += leaf_h;
                rect.h -= leaf_h;
            }
        }
    }
    out.push(rect);
    out
}

/// Golden-ratio child rect inside `parent` — size = parent × 1/φ, random offset
/// snapped to one of the golden-ratio anchor points.
fn golden_child(parent: Rect) -> Rect {
    const INV_PHI: f32 = 0.618_034;
    let cw = ((parent.w as f32) * INV_PHI).round().max(4.0) as i32;
    let ch = ((parent.h as f32) * INV_PHI).round().max(3.0) as i32;
    let cw = cw.min(parent.w - 1);
    let ch = ch.min(parent.h - 1);
    // Pick a corner bias — 4 golden anchors (φ/1-φ combinations).
    let bias_x = if r_f32() < 0.5 { 0.0 } else { 1.0 - INV_PHI };
    let bias_y = if r_f32() < 0.5 { 0.0 } else { 1.0 - INV_PHI };
    let x = parent.x + ((parent.w - cw) as f32 * bias_x).round() as i32;
    let y = parent.y + ((parent.h - ch) as f32 * bias_y).round() as i32;
    Rect { x, y, w: cw, h: ch }
}

// ───────────────────────── types ─────────────────────────
#[derive(Clone, Copy, PartialEq, Eq)]
enum Side { L, R, Chaos, Nested }

#[derive(Clone, Copy, PartialEq, Eq)]
enum FlowDir { RowMajor, RowMajorRev, ColMajor, ColMajorRev }

fn random_flow() -> FlowDir {
    match r_u32() & 3 {
        0 => FlowDir::RowMajor,
        1 => FlowDir::RowMajorRev,
        2 => FlowDir::ColMajor,
        _ => FlowDir::ColMajorRev,
    }
}

#[inline]
fn cell_index(rx: i32, ry: i32, w: i32, h: i32, dir: FlowDir) -> i64 {
    let rx = rx as i64; let ry = ry as i64;
    let w = w as i64; let h = h as i64;
    match dir {
        FlowDir::RowMajor    =>  ry * w + rx,
        FlowDir::RowMajorRev => (h - 1 - ry) * w + (w - 1 - rx),
        FlowDir::ColMajor    =>  rx * h + ry,
        FlowDir::ColMajorRev => (w - 1 - rx) * h + (h - 1 - ry),
    }
}

/// Bespoke algorithms. Each zone becomes a compressed/algorithmic/text signal
/// expression — no smooth waves or lissajous curves, just chunky block/text art.
#[derive(Clone, Copy)]
enum Formation {
    /// Spinning wireframe sphere raytraced into the zone as an intensity ramp.
    Raytrace,
    /// Hard-stepped {░▒▓█} strata scrolling vertically.
    BlockStrata,
    /// Hex memory dump: `xxxx: AB CD EF ...`
    ParseDump,
    /// Named registers with values: `R0: 0x4FE8D1A2`
    RegisterDump,
    /// Stream char → arrow → ROT13 / nibble transform.
    TextmarkConverter,
    /// Elementary CA (rule 30 or 110) seeded from stream bits.
    Cellular1D { rule: u8 },
    /// Scrolling stream text with bright middle band, dim above/below.
    Marquee,
    /// 2-cell-block density mosaic from stream bytes.
    DensityGrid,
    /// Sparse transformer-attention pattern — diagonal band + sink cols + hotspots.
    AttentionMatrix,
    /// Probability distribution over candidate next tokens — top-K bars,
    /// sorted by mass, values derived from stream. This is literally what a
    /// language model *is* at any moment — a distribution.
    ProbField,
    /// Converted image (tiger/viper/etc) rendered via full dotmax pipeline
    /// (ImageRenderer → BrailleGrid). Subject to glitching effects.
    ImagePanel { asset: usize },
    /// Wireframe cube on BLACK background — kept for future use.
    #[allow(dead_code)]
    RaytraceCube,
    /// Live chess board rendered via dotmax::chess from a shakmaty position.
    ChessBoard,
}

/// Important UI text (labels, balance digits, button text) is bright WHITE
/// so it stands out from the deep-red field. Use this for any UI chrome
/// that absolutely must read clearly.
const UI_WHITE: (u8, u8, u8) = (255, 255, 255);

/// A single dither-variant render of an image.
struct ImageVariant {
    cells: Vec<Vec<char>>,
    w: usize,
    h: usize,
}

/// An image with multiple dither variants pre-rendered. Paint time picks
/// a variant per-cell via a wave function so different dither styles
/// sweep across the image over time.
struct ImageAsset {
    name: &'static str,
    variants: Vec<ImageVariant>,   // one per dither method
    luma: Vec<u8>,                 // raw pattern bytes from first variant — pipe payload
    // Each image breathes at its own slow rate within the panel so the
    // visible scale of the subject drifts over time and is different per
    // image. scale = base + amp * sin(cursor * rate + phase). Rates are
    // chosen so a full cycle takes ~10–35 seconds.
    scale_base: f32,
    scale_amp: f32,
    scale_rate: f32,
    scale_phase: f32,
}

const DITHER_METHODS: &[DitheringMethod] = &[
    DitheringMethod::None,
    DitheringMethod::FloydSteinberg,
    DitheringMethod::Bayer,
    DitheringMethod::Atkinson,
];

/// Load and convert one image, rendering every dither method in
/// DITHER_METHODS as separate variants.
fn load_image(path: &str, name: &'static str, cells_w: usize, cells_h: usize) -> Option<ImageAsset> {
    let mut variants: Vec<ImageVariant> = Vec::with_capacity(DITHER_METHODS.len());
    let mut luma: Option<Vec<u8>> = None;
    for &m in DITHER_METHODS {
        let grid = ImageRenderer::new()
            .load_from_path(Path::new(path)).ok()?
            .resize(cells_w, cells_h, true).ok()?
            .dithering(m)
            .render().ok()?;
        let (gw, gh) = grid.dimensions();
        let mut cells: Vec<Vec<char>> = vec![vec![' '; gw]; gh];
        for y in 0..gh {
            for x in 0..gw {
                cells[y][x] = grid.get_char(x, y);
            }
        }
        if luma.is_none() {
            luma = Some(grid.get_raw_patterns().to_vec());
        }
        variants.push(ImageVariant { cells, w: gw, h: gh });
    }
    Some(ImageAsset {
        name,
        variants,
        luma: luma.unwrap_or_default(),
        scale_base: 1.0,
        scale_amp: 0.0,
        scale_rate: 0.0,
        scale_phase: 0.0,
    })
}

/// Stable per-asset hash so each image gets fixed render-size and breath
/// parameters across runs (and runs match across machines).
fn asset_hash(name: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// Per-asset intrinsic render size (cells). Heavy variation so different
/// animals fill the same panel at very different apparent sizes.
fn asset_render_size(h: u32) -> (usize, usize) {
    match (h >> 5) & 0x7 {
        0 => (104, 52),
        1 => (88, 44),
        2 => (72, 36),
        3 => (64, 32),
        4 => (56, 28),
        5 => (48, 24),
        6 => (40, 20),
        _ => (32, 16),
    }
}

/// Per-asset breath: slow scale oscillation, different rate/phase per
/// image. cursor accumulates at flow_rate=48/sec, so a rate of 0.0035 ≈
/// 35-second period and 0.012 ≈ 10-second period.
fn asset_breath(h: u32) -> (f32, f32, f32, f32) {
    let r1 = ((h >> 3) & 0xFFFF) as f32 / 65535.0;
    let r2 = ((h >> 11) & 0xFFFF) as f32 / 65535.0;
    let r3 = ((h >> 17) & 0xFFFF) as f32 / 65535.0;
    let r4 = ((h.rotate_left(7)) & 0xFFFF) as f32 / 65535.0;
    let scale_base  = 0.75 + r1 * 0.70;            // 0.75 .. 1.45
    let scale_amp   = 0.14 + r2 * 0.32;            // 0.14 .. 0.46
    let scale_rate  = 0.0035 + r3 * 0.0090;        // ~10–35s period
    let scale_phase = r4 * std::f32::consts::TAU;
    (scale_base, scale_amp, scale_rate, scale_phase)
}

fn load_image_assets() -> Vec<ImageAsset> {
    // Tigers, snakes, rabbits, frogs, grifter — plus the full animal
    // bestiary from purple_rain/. No skylines, no palms — animals only.
    let candidates: &[(&str, &str, &'static str)] = &[
        ("tests/fixtures/images/tiger_small.png",        "./tests/fixtures/images/tiger_small.png",        "TIGER"),
        ("tests/fixtures/images/tiger_1.png",            "./tests/fixtures/images/tiger_1.png",            "TIGR2"),
        ("tests/fixtures/images/viper3.png",             "./tests/fixtures/images/viper3.png",             "VIPER"),
        ("tests/fixtures/images/viper_head_3.png",       "./tests/fixtures/images/viper_head_3.png",       "VHEAD"),
        ("tests/fixtures/images/extras/snakedesk.png",   "./tests/fixtures/images/extras/snakedesk.png",   "SNAKE"),
        ("tests/fixtures/images/extras/rabbit.png",      "./tests/fixtures/images/extras/rabbit.png",      "RABT"),
        ("tests/fixtures/images/extras/grifter.jpg",     "./tests/fixtures/images/extras/grifter.jpg",     "GRFTR"),
        ("tests/fixtures/images/extras/frog_01.png",     "./tests/fixtures/images/extras/frog_01.png",     "FROG"),
        ("tests/fixtures/images/extras/frog_02.png",     "./tests/fixtures/images/extras/frog_02.png",     "FROG2"),
        ("tests/fixtures/images/purple_rain/bear.png",      "./tests/fixtures/images/purple_rain/bear.png",      "BEAR"),
        ("tests/fixtures/images/purple_rain/dolphins.png",  "./tests/fixtures/images/purple_rain/dolphins.png",  "DLPHN"),
        ("tests/fixtures/images/purple_rain/elephant.png",  "./tests/fixtures/images/purple_rain/elephant.png",  "ELPHT"),
        ("tests/fixtures/images/purple_rain/fox.png",       "./tests/fixtures/images/purple_rain/fox.png",       "FOX"),
        ("tests/fixtures/images/purple_rain/horse.png",     "./tests/fixtures/images/purple_rain/horse.png",     "HORSE"),
        ("tests/fixtures/images/purple_rain/jellyfish.png", "./tests/fixtures/images/purple_rain/jellyfish.png", "JELLY"),
        ("tests/fixtures/images/purple_rain/octopus.png",   "./tests/fixtures/images/purple_rain/octopus.png",   "OCTO"),
        ("tests/fixtures/images/purple_rain/owl.png",       "./tests/fixtures/images/purple_rain/owl.png",       "OWL"),
        ("tests/fixtures/images/purple_rain/panther.png",   "./tests/fixtures/images/purple_rain/panther.png",   "PANTH"),
        ("tests/fixtures/images/purple_rain/raven.png",     "./tests/fixtures/images/purple_rain/raven.png",     "RAVEN"),
        ("tests/fixtures/images/purple_rain/shark.png",     "./tests/fixtures/images/purple_rain/shark.png",     "SHARK"),
        ("tests/fixtures/images/purple_rain/snake.png",     "./tests/fixtures/images/purple_rain/snake.png",     "SNK2"),
        ("tests/fixtures/images/purple_rain/stag.png",      "./tests/fixtures/images/purple_rain/stag.png",      "STAG"),
        ("tests/fixtures/images/purple_rain/whale.png",     "./tests/fixtures/images/purple_rain/whale.png",     "WHALE"),
        ("tests/fixtures/images/purple_rain/wolf.png",      "./tests/fixtures/images/purple_rain/wolf.png",      "WOLF"),
    ];
    let mut out = Vec::new();
    for &(p1, p2, name) in candidates {
        let h = asset_hash(name);
        let (cw, ch) = asset_render_size(h);
        let loaded = load_image(p1, name, cw, ch).or_else(|| load_image(p2, name, cw, ch));
        if let Some(mut a) = loaded {
            let (sb, sa, sr, sp) = asset_breath(h);
            a.scale_base = sb;
            a.scale_amp = sa;
            a.scale_rate = sr;
            a.scale_phase = sp;
            out.push(a);
        }
    }
    out
}

fn pick_formation(rect: Rect) -> Formation {
    let aspect = (rect.w as f32) / (rect.h.max(1) as f32);
    let r = r_u32() as usize;
    if aspect > 3.5 {
        // Very wide — horizontal readouts.
        match r % 4 {
            0 => Formation::Marquee,
            1 => Formation::TextmarkConverter,
            2 => Formation::BlockStrata,
            _ => Formation::RegisterDump,
        }
    } else if aspect < 0.65 {
        // Tall/narrow — vertical-friendly stuff.
        match r % 3 {
            0 => Formation::ParseDump,
            1 => Formation::Cellular1D { rule: if r & 1 == 0 { 30 } else { 110 } },
            _ => Formation::BlockStrata,
        }
    } else if (aspect - 1.0).abs() < 0.45 && rect.w >= 10 && rect.h >= 6 {
        // Square-ish + large enough — save the wow formations for here.
        match r % 4 {
            0 => Formation::Raytrace,
            1 => Formation::AttentionMatrix,
            2 => Formation::DensityGrid,
            _ => Formation::Cellular1D { rule: if r & 1 == 0 { 30 } else { 110 } },
        }
    } else {
        // Mid aspect — everything fair game.
        match r % 9 {
            0 => Formation::ParseDump,
            1 => Formation::RegisterDump,
            2 => Formation::DensityGrid,
            3 => Formation::TextmarkConverter,
            4 => Formation::Cellular1D { rule: if r & 1 == 0 { 30 } else { 110 } },
            5 => Formation::BlockStrata,
            6 => Formation::AttentionMatrix,
            7 => Formation::ProbField,
            _ => Formation::Marquee,
        }
    }
}

struct Zone {
    base_rect: Rect,       // locked — this is also what's rendered
    rect: Rect,            // kept for convenience; equals base_rect always

    side: Side,
    formation: Formation,
    flow_dir: FlowDir,
    tap_offset: i32,
    pulse: f32,
    pulse_rate: f32,
    glitch_rate: f32,
}

fn make_zone(
    base: Rect,
    side: Side,
    formation: Formation,
    flow_dir: FlowDir,
    tap: i32,
    _zone_idx: usize,
) -> Zone {
    Zone {
        base_rect: base,
        rect: base,
        side,
        formation,
        flow_dir,
        tap_offset: tap,
        pulse: r_f32() * 3.0,
        pulse_rate: 0.6 + r_f32() * 1.1,
        glitch_rate: if r_f32() < 0.15 { 0.3 + r_f32() * 0.7 } else { 0.0 },
    }
}

struct Scene {
    w: i32, h: i32,
    stream: Vec<char>,
    cursor: f32,
    flow_rate: f32,
    zones: Vec<Zone>,
    pipes: Vec<Pipe>,
    assets: Vec<ImageAsset>,
    adjacency: Vec<Vec<u16>>,
    /// Persistent overlay flow lines (orthogonal + crossed diagonals).
    streamers: Vec<Streamer>,
    /// Edge-anchored noise injection feeds.
    noise_feeds: Vec<NoiseFeed>,
    /// Rects that streamers/feeds skip — cube window + chess board.
    protected_rects: Vec<Rect>,
    /// Live chess game played by random legal moves.
    chess_pos: Chess,
    /// Cursor value when last chess move was played.
    chess_last_move_at: f32,
    /// Active short-lived image fragments blasted on top of the scene.
    glitch_inserts: Vec<GlitchInsertion>,
    /// Cursor when the last glitch insert was spawned.
    last_glitch_spawn: f32,
    /// Wandering dither worms — block-density trails that crawl through grids.
    dither_flows: Vec<DitherFlow>,
    flipped: bool,
    reversed: bool,
}

/// Bundle of everything a paint function might need to read.
struct PaintCtx<'a> {
    stream: &'a [char],
    cursor: f32,
    zones: &'a [Zone],
    adjacency: &'a [Vec<u16>],
    assets: &'a [ImageAsset],
    chess_pos: &'a Chess,
}

// ───────────────────────── scene construction ─────────────────────────
fn build_scene(w: i32, h: i32) -> Scene {
    seed_from_clock();

    let mid = w / 2;
    // Depth scales with size — no slivers on tiny terminals.
    let depth = ((w.min(h * 2)) / 14).clamp(4, 8) as usize;

    let left_spiral  = fib_spiral(Rect { x: 0,   y: 0, w: mid,      h }, depth, true);
    let right_spiral = fib_spiral(Rect { x: mid, y: 0, w: w - mid,  h }, depth, false);

    let mut zones = Vec::new();
    let mut tap_accum: i64 = 0;

    // LEFT: walk outermost → innermost. Default flow RowMajor.
    for rect in &left_spiral {
        let side = if r_f32() < 0.08 { Side::Chaos } else { Side::L };
        let flow_dir = if r_f32() < 0.20 { random_flow() } else { FlowDir::RowMajor };
        let formation = pick_formation(*rect);
        let idx = zones.len();
        zones.push(make_zone(*rect, side, formation, flow_dir, tap_accum as i32, idx));
        tap_accum += (rect.w * rect.h) as i64;
    }

    // RIGHT: walk innermost → outermost so the hose reverses direction,
    // creating the mirrored flow. Default flow RowMajorRev.
    for rect in right_spiral.iter().rev() {
        let side = if r_f32() < 0.08 { Side::Chaos } else { Side::R };
        let flow_dir = if r_f32() < 0.20 { random_flow() } else { FlowDir::RowMajorRev };
        let formation = pick_formation(*rect);
        let idx = zones.len();
        zones.push(make_zone(*rect, side, formation, flow_dir, tap_accum as i32, idx));
        tap_accum += (rect.w * rect.h) as i64;
    }

    // NESTED CHILDREN — overlay on the 4 biggest zones so the composition has
    // "boxes within boxes" at golden-ratio insets.
    let mut big_indices: Vec<usize> = (0..zones.len()).collect();
    big_indices.sort_by_key(|&i| -(zones[i].base_rect.w * zones[i].base_rect.h));
    for &i in big_indices.iter().take(4) {
        let parent = zones[i].base_rect;
        if parent.w < 12 || parent.h < 6 { continue; }
        let child = golden_child(parent);
        if child.w < 5 || child.h < 3 { continue; }
        let formation = pick_formation(child);
        let idx = zones.len();
        let mut z = make_zone(child, Side::Nested, formation, random_flow(), tap_accum as i32, idx);
        z.glitch_rate = 0.15 + r_f32() * 0.4;
        zones.push(z);
        tap_accum += (child.w * child.h) as i64;
    }

    // Load image assets now so we can assign specific zones to display them.
    let assets = load_image_assets();

    // ─── Size-aware layout ─── compute every UI rect from (w, h) so the
    // betting interface scales up to fill any terminal size, always large.
    //
    // Chess is centered. Square aspect: cell width = 2 × cell height (since
    // braille cells are ~2:1 tall).
    let chess_h = ((h as f32 * 0.55) as i32).clamp(8, 36);
    let mut chess_w = chess_h * 2;
    if chess_w > w * 5 / 8 {
        chess_w = (w * 5 / 8) & !1;  // even
        // recompute height to maintain aspect
    }
    let chess_w = chess_w.clamp(16, 80);
    let chess_h = (chess_w / 2).clamp(8, 36);
    let ui_chess = Rect {
        x: (w - chess_w) / 2,
        y: ((h - chess_h) / 2 - 1).max(2),
        w: chess_w,
        h: chess_h,
    };

    let panel_w = (w / 7).clamp(18, 28);

    // ─── Dedicated SNAKE image slots ─── carved next to the chess board so
    // the vipers stay visible and big. Tall narrow strips on each side,
    // running the full height of the canvas now that the HUD is gone.
    let strip_y = 1_i32;
    let strip_h = (h - 2).max(8);
    let ui_viper = Rect { x: 1, y: strip_y, w: panel_w, h: strip_h };
    let ui_vhead = Rect { x: w - panel_w - 1, y: strip_y, w: panel_w, h: strip_h };

    // Sort the surviving zones by area for asset/formation assignment.
    let mut sorted_by_area: Vec<usize> = (0..zones.len())
        .filter(|&i| zones[i].side != Side::Nested)
        .collect();
    sorted_by_area.sort_by_key(|&i| -(zones[i].base_rect.w * zones[i].base_rect.h));

    // Seed image panels into the biggest non-Nested survivors.
    if !assets.is_empty() {
        let mut assigned = 0usize;
        let want = assets.len().min(sorted_by_area.len());
        for &i in &sorted_by_area {
            let r = zones[i].base_rect;
            if r.w < 10 || r.h < 6 { continue; }
            zones[i].formation = Formation::ImagePanel { asset: assigned % assets.len() };
            assigned += 1;
            if assigned >= want { break; }
        }
    }

    let _ui_rects = [ui_chess, ui_viper, ui_vhead];
    // NOTE: fib zones are NOT filtered — chaos paints under everything,
    // and UI re-paints on top of the chaos in a final pass (see render()).

    // Push UI zones. Each is Side::Nested so they don't participate in pipes.
    let mut push_ui = |rect: Rect, formation: Formation, tap: &mut i64| {
        zones.push(Zone {
            base_rect: rect,
            rect,
            side: Side::Nested,
            formation,
            flow_dir: FlowDir::RowMajor,
            tap_offset: *tap as i32,
            pulse: r_f32() * 3.0,
            pulse_rate: 0.8 + r_f32() * 0.5,
            glitch_rate: 0.0,
        });
        *tap += (rect.w * rect.h) as i64;
    };
    push_ui(ui_chess,   Formation::ChessBoard,                &mut tap_accum);
    // SCARY SNAKES — guaranteed visible at decent size.
    let viper_idx = if assets.len() > 1 { 1 } else { 0 };
    let vhead_idx = if assets.len() > 2 { 2 } else { viper_idx };
    push_ui(ui_viper, Formation::ImagePanel { asset: viper_idx }, &mut tap_accum);
    push_ui(ui_vhead, Formation::ImagePanel { asset: vhead_idx }, &mut tap_accum);

    let stream = build_stream(32_768);
    let pipes: Vec<Pipe> = build_pipes(&zones);

    // Adjacency: neighbors = zones a pipe actually connects.
    let mut adjacency: Vec<Vec<u16>> = vec![Vec::new(); zones.len()];
    for p in &pipes {
        adjacency[p.from as usize].push(p.to);
        adjacency[p.to as usize].push(p.from);
    }

    // Persistent overlay streamers — many axes for synapse density.
    let streamers = vec![
        // Horizontals at varied rows
        Streamer { axis: StreamerAxis::Horizontal, anchor: (h as f32 * 0.06) as i32, speed: 26.0, direction:  1 },
        Streamer { axis: StreamerAxis::Horizontal, anchor: (h as f32 * 0.42) as i32, speed: 32.0, direction: -1 },
        Streamer { axis: StreamerAxis::Horizontal, anchor: (h as f32 * 0.78) as i32, speed: 21.0, direction:  1 },
        Streamer { axis: StreamerAxis::Horizontal, anchor: (h as f32 * 0.94) as i32, speed: 18.0, direction: -1 },
        // Verticals on far edges
        Streamer { axis: StreamerAxis::Vertical,   anchor: (w as f32 * 0.04) as i32, speed: 22.0, direction: -1 },
        Streamer { axis: StreamerAxis::Vertical,   anchor: (w as f32 * 0.97) as i32, speed: 28.0, direction:  1 },
        // Diagonals at multiple intercepts
        Streamer { axis: StreamerAxis::DiagPos,    anchor: (w as f32 * 0.02) as i32, speed: 18.0, direction:  1 },
        Streamer { axis: StreamerAxis::DiagPos,    anchor: (w as f32 * 0.45) as i32, speed: 24.0, direction:  1 },
        Streamer { axis: StreamerAxis::DiagNeg,    anchor: (w as f32 * 0.98) as i32, speed: 17.0, direction: -1 },
        Streamer { axis: StreamerAxis::DiagNeg,    anchor: (w as f32 * 0.55) as i32, speed: 23.0, direction: -1 },
    ];

    // Noise projection feeds — all 4 edges, dense.
    let noise_feeds = vec![
        // Top edge
        NoiseFeed { pos: ((w as f32 * 0.10) as i32, 0), dir: (0, 1), length: 4, seed: 0xACE0_BEEF, speed: 11.0 },
        NoiseFeed { pos: ((w as f32 * 0.30) as i32, 0), dir: (0, 1), length: 5, seed: 0xFACE_FADE, speed: 13.0 },
        NoiseFeed { pos: ((w as f32 * 0.50) as i32, 0), dir: (0, 1), length: 3, seed: 0xB001_C0DE, speed: 15.0 },
        NoiseFeed { pos: ((w as f32 * 0.70) as i32, 0), dir: (0, 1), length: 4, seed: 0x1337_C0DE, speed: 12.0 },
        NoiseFeed { pos: ((w as f32 * 0.90) as i32, 0), dir: (0, 1), length: 3, seed: 0xBEEF_F00D, speed: 16.0 },
        // Left edge
        NoiseFeed { pos: (0, (h as f32 * 0.30) as i32), dir: (1, 0), length: 5, seed: 0xDEAD_BEEF, speed: 12.0 },
        NoiseFeed { pos: (0, (h as f32 * 0.55) as i32), dir: (1, 0), length: 4, seed: 0xCAFE_F00D, speed: 14.0 },
        NoiseFeed { pos: (0, (h as f32 * 0.78) as i32), dir: (1, 0), length: 5, seed: 0xFEED_BABE, speed: 10.0 },
        // Right edge
        NoiseFeed { pos: (w - 1, (h as f32 * 0.30) as i32), dir: (-1, 0), length: 5, seed: 0xDEAD_C0DE, speed: 14.0 },
        NoiseFeed { pos: (w - 1, (h as f32 * 0.55) as i32), dir: (-1, 0), length: 4, seed: 0x4269_4269, speed: 11.0 },
        NoiseFeed { pos: (w - 1, (h as f32 * 0.78) as i32), dir: (-1, 0), length: 5, seed: 0xC001_BEEF, speed: 15.0 },
        // Bottom edge
        NoiseFeed { pos: ((w as f32 * 0.20) as i32, h - 1), dir: (0, -1), length: 4, seed: 0xC0DE_F00D, speed: 12.0 },
        NoiseFeed { pos: ((w as f32 * 0.55) as i32, h - 1), dir: (0, -1), length: 4, seed: 0xBEEF_BABE, speed: 13.0 },
        NoiseFeed { pos: ((w as f32 * 0.85) as i32, h - 1), dir: (0, -1), length: 5, seed: 0xFA15_AFE1, speed: 11.0 },
    ];

    // No protected rects — chaos bleeds everywhere. UI re-paints on top.
    let protected_rects: Vec<Rect> = Vec::new();

    Scene {
        w, h, stream, cursor: 0.0, flow_rate: 48.0,
        zones, pipes, assets, adjacency,
        streamers, noise_feeds,
        protected_rects,
        chess_pos: Chess::default(),
        chess_last_move_at: 0.0,
        glitch_inserts: Vec::new(),
        last_glitch_spawn: 0.0,
        dither_flows: {
            let mut flows = Vec::new();
            for _ in 0..6 {
                let speed = 6.0 + r_f32() * 12.0;
                let theta = r_f32() * std::f32::consts::TAU;
                flows.push(DitherFlow {
                    pos_x: r_f32() * w as f32,
                    pos_y: r_f32() * h as f32,
                    vel_x: theta.cos() * speed,
                    vel_y: theta.sin() * speed * 0.5, // y velocity halved (terminal aspect)
                    trail: Vec::new(),
                    trail_max: 14 + (r_u32() as usize % 18),
                });
            }
            flows
        },
        flipped: false,
        reversed: false,
    }
}

// ───────────────────────── simulation ─────────────────────────
fn tick(scene: &mut Scene, dt: f32) {
    let sign = if scene.reversed { -1.0 } else { 1.0 };
    scene.cursor += scene.flow_rate * dt * sign;
    for z in &mut scene.zones {
        z.pulse += dt * z.pulse_rate * sign;
    }
    // Advance the chess game — one random legal move every ~1.2 seconds (60 cursor units).
    if scene.cursor - scene.chess_last_move_at > 60.0 {
        scene.chess_last_move_at = scene.cursor;
        let moves = scene.chess_pos.legal_moves();
        if moves.is_empty() {
            scene.chess_pos = Chess::default();
        } else {
            let idx = (r_u32() as usize) % moves.len();
            let mv = moves[idx];
            scene.chess_pos.play_unchecked(mv);
        }
    }
    // Tick wandering dither flows.
    let w = scene.w;
    let h = scene.h;
    for flow in &mut scene.dither_flows {
        tick_flow(flow, dt * sign, w, h);
    }

    // Glitch insertions — random image chunks blasted onto the screen.
    // Expire dead ones first.
    let cur = scene.cursor;
    scene.glitch_inserts.retain(|i| (cur - i.spawn_cursor).abs() < i.duration_chars);
    // Then maybe spawn a new one. Up to 5 active simultaneously.
    if scene.cursor - scene.last_glitch_spawn > 25.0 && scene.glitch_inserts.len() < 5 && !scene.assets.is_empty() {
        scene.last_glitch_spawn = scene.cursor;
        if r_f32() < 0.85 {
            spawn_glitch_insertion(scene);
        }
    }
}

fn spawn_glitch_insertion(scene: &mut Scene) {
    let asset_idx = (r_u32() as usize) % scene.assets.len();
    let asset = &scene.assets[asset_idx];
    if asset.variants.is_empty() { return; }
    let variant_idx = (r_u32() as usize) % asset.variants.len();
    let variant = &asset.variants[variant_idx];
    let aw = variant.w as i32;
    let ah = variant.h as i32;

    // Random size + position. Chunks range from small to half-screen.
    let max_w = (scene.w / 2).max(8);
    let max_h = (scene.h * 2 / 3).max(6);
    let rw = (8 + (r_u32() as i32 % (max_w - 7).max(1))).min(scene.w - 1);
    let rh = (4 + (r_u32() as i32 % (max_h - 3).max(1))).min(scene.h - 1);
    let rx = r_u32() as i32 % (scene.w - rw).max(1);
    let ry = r_u32() as i32 % (scene.h - rh).max(1);
    let rect = Rect { x: rx, y: ry, w: rw, h: rh };

    // Half the time: full image. Other half: random crop (a strip or chunk).
    let crop = if r_f32() < 0.5 {
        None
    } else {
        let cw = (4 + (r_u32() as i32 % (aw - 3).max(1))).min(aw);
        let ch = (3 + (r_u32() as i32 % (ah - 2).max(1))).min(ah);
        let cx = r_u32() as i32 % (aw - cw).max(1);
        let cy = r_u32() as i32 % (ah - ch).max(1);
        Some(Rect { x: cx, y: cy, w: cw, h: ch })
    };

    let duration_chars = 30.0 + r_f32() * 90.0; // ~0.6 to ~2.5 sec @ 48 cps
    scene.glitch_inserts.push(GlitchInsertion {
        asset_idx,
        rect,
        crop,
        spawn_cursor: scene.cursor,
        duration_chars,
        variant_idx,
    });
}

// ───────────────────────── rendering ─────────────────────────
/// Sentinel value meaning "no zone owns this cell yet."
const NO_OWNER: u16 = u16::MAX;

#[derive(Clone, Copy)]
struct PxCell {
    ch: char,
    fg: (u8, u8, u8),
    intensity: f32,
    owner: u16,   // zone index that won this cell — used for hard-cutoff masks
}
impl PxCell {
    const fn empty() -> Self {
        Self { ch: ' ', fg: (0, 0, 0), intensity: 0.0, owner: NO_OWNER }
    }
}

fn put(grid: &mut [Vec<PxCell>], x: i32, y: i32, ch: char, c: (u8, u8, u8), i: f32, owner: u16) {
    if y < 0 || x < 0 { return; }
    let (uy, ux) = (y as usize, x as usize);
    if uy >= grid.len() || ux >= grid[0].len() { return; }
    let cell = &mut grid[uy][ux];
    if i >= cell.intensity {
        cell.ch = ch;
        cell.fg = c;
        cell.intensity = i;
        cell.owner = owner;
    }
}

/// Forced paint — used by pipes to bleed across zone boundaries regardless
/// of who owns the cell. Always overwrites.
fn put_force(grid: &mut [Vec<PxCell>], x: i32, y: i32, ch: char, c: (u8, u8, u8), i: f32) {
    if y < 0 || x < 0 { return; }
    let (uy, ux) = (y as usize, x as usize);
    if uy >= grid.len() || ux >= grid[0].len() { return; }
    grid[uy][ux] = PxCell { ch, fg: c, intensity: i, owner: NO_OWNER };
}

fn color_for(side: Side) -> (u8, u8, u8) {
    // Pure grayscale — signal comes from intensity + char-weight, not hue.
    // Side identity survives as small brightness differences at full intensity.
    match side {
        Side::L | Side::R => (170, 18, 18),  // deep matte blood red
        Side::Chaos       => (255, 90, 90),  // hot pink-red pops through
        Side::Nested      => (230, 40, 40),  // bright red, not quite hot
    }
}

#[inline]
fn sample(stream: &[char], idx: i64) -> char {
    let n = stream.len() as i64;
    stream[idx.rem_euclid(n) as usize]
}

/// Occasional discrete phase jumps, modulated by zone.pulse.
fn glitch_offset(z: &Zone) -> i64 {
    if z.glitch_rate < 0.05 { return 0; }
    let phase = (z.pulse * z.glitch_rate * 0.6) as i64;
    // wrapping_mul by a prime gives chaotic jumps when phase increments.
    phase.wrapping_mul(2_039)
}

// ─────────────── formation paint helpers ───────────────

fn ihash(x: i32, y: i32, t: i32) -> u32 {
    let mut n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add((t as u32).wrapping_mul(2_654_435_761));
    n ^= n >> 13;
    n = n.wrapping_mul(1_274_126_177);
    n ^ (n >> 16)
}

fn paint_fill(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, ch: char, i: f32) {
    let color = color_for(zone.side);
    for ry in 0..zone.rect.h {
        for rx in 0..zone.rect.w {
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
        }
    }
}

/// 1. Raytrace window — spinning wireframe sphere. The wow.
fn paint_raytrace(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16) {
    let color = color_for(zone.side);
    let w = zone.rect.w as usize;
    let h = zone.rect.h as usize;
    if w < 4 || h < 3 {
        paint_fill(grid, zone, zone_id, '·', 0.20);
        return;
    }
    let mut rt = RtScene::new();
    rt.add_object(Box::new(Sphere::new(Vector3::new(0.0, 0.0, -3.0), 1.1)));
    let cam = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let orient = WireframeRotation {
        yaw: zone.pulse * 0.6,
        pitch: (zone.pulse * 0.4).sin() * 0.45,
        roll: 0.0,
    };
    let mode = RenderMode::Wireframe {
        step_rad: 15.0_f32.to_radians(),
        tol_rad: 0.035,
    };
    let buf = render_with_orientation(&rt, &cam, w, h, mode, orient);

    let ramp: &[char] = &[' ', '·', ':', '-', '=', '+', '*', '#', '%', '@'];
    for ry in 0..h {
        for rx in 0..w {
            let v = buf[ry][rx].clamp(0.0, 1.0);
            let idx = ((v * (ramp.len() - 1) as f32).round() as usize).min(ramp.len() - 1);
            let ch = ramp[idx];
            let i = if v > 0.30 { 0.90 } else { 0.22 };
            put(grid, zone.rect.x + rx as i32, zone.rect.y + ry as i32, ch, color, i, zone_id);
        }
    }
}

/// 2. BlockStrata — hard-stepped density bands, no smooth interp.
fn paint_block_strata(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16) {
    let color = color_for(zone.side);
    let levels: &[(char, f32)] = &[
        (' ', 0.08),
        ('░', 0.38),
        ('▒', 0.62),
        ('▓', 0.85),
        ('█', 1.00),
    ];
    let scroll = (zone.pulse * 2.4) as i32;
    // Step-function over y: 20-row cycle with custom profile.
    for ry in 0..zone.rect.h {
        let stripe = (ry + scroll).rem_euclid(20);
        let level_idx = match stripe {
            0..=1 => 0,
            2..=4 => 1,
            5..=8 => 2,
            9..=12 => 3,
            13..=15 => 4,
            16..=18 => 3,
            _ => 2,
        };
        let (ch, i) = levels[level_idx];
        for rx in 0..zone.rect.w {
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
        }
    }
}

/// 3. ParseDump — `xxxx: AB CD EF ...` hex memory dump.
fn paint_parse_dump(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, stream: &[char], cursor: f32) {
    let color = color_for(zone.side);
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64);
    let scroll = base / 4;

    for ry in 0..zh {
        let addr = (scroll.wrapping_add(ry as i64) & 0xffff) as u32;
        for rx in 0..zw {
            let (ch, i) = if rx < 4 {
                let nibble = ((addr >> ((3 - rx) * 4)) & 0xf) as usize;
                (HEX[nibble], 0.72)
            } else if rx == 4 {
                (':', 0.55)
            } else if rx == 5 {
                (' ', 0.10)
            } else {
                let rel = rx - 6;
                let byte_idx = rel / 3;
                let pos = rel % 3;
                let b = sample(stream, base + (ry as i64) * 9 + byte_idx as i64) as u32;
                match pos {
                    0 => (HEX[((b >> 4) & 0xf) as usize], 0.92),
                    1 => (HEX[(b & 0xf) as usize], 0.92),
                    _ => (' ', 0.12),
                }
            };
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
        }
    }
}

/// 4. RegisterDump — named registers with hex values.
fn paint_register_dump(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, stream: &[char], cursor: f32) {
    const NAMES: &[&str] = &["R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "PC", "SP", "LR", "SR"];
    let color = color_for(zone.side);
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64);

    for ry in 0..zh {
        let name_cycle = ((base / 8) as usize).wrapping_add(ry as usize) % NAMES.len();
        let name = NAMES[name_cycle];
        let mut row: Vec<(char, f32)> = Vec::with_capacity(zw as usize);
        for ch in name.chars() {
            row.push((ch, 0.82));
        }
        row.push((':', 0.55));
        row.push((' ', 0.10));
        row.push(('0', 0.70));
        row.push(('x', 0.70));
        for i in 0..8 {
            let nib = sample(stream, base + (ry as i64) * 5 + i as i64) as u32;
            row.push((HEX[(nib & 0xf) as usize], 0.95));
        }
        while row.len() < zw as usize { row.push((' ', 0.10)); }
        for (rx, &(ch, i)) in row.iter().take(zw as usize).enumerate() {
            put(grid, zone.rect.x + rx as i32, zone.rect.y + ry, ch, color, i, zone_id);
        }
    }
}

/// 5. TextmarkConverter — left side raw stream → `⇒` → right side transformed.
fn paint_textmark(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, stream: &[char], cursor: f32) {
    let color = color_for(zone.side);
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    let mid_y = zh / 2;
    let mid_x = zw / 2;
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64);

    let transform = |c: char| -> char {
        if c.is_ascii_alphabetic() {
            let b = if c.is_ascii_lowercase() { b'a' } else { b'A' };
            let off = ((c as u8) - b + 13) % 26;
            (b + off) as char
        } else if c.is_ascii_digit() {
            let d = (c as u8) - b'0';
            (b'0' + (9 - d)) as char
        } else if c.is_ascii() {
            HEX[((c as u8) >> 4 & 0x0f) as usize]
        } else {
            HEX[((c as u32) & 0x0f) as usize]
        }
    };

    for ry in 0..zh {
        for rx in 0..zw {
            if ry == mid_y && rx == mid_x {
                put(grid, zone.rect.x + rx, zone.rect.y + ry, '⇒', color, 1.0, zone_id);
                continue;
            }
            if ry == mid_y {
                let (ch, i) = if rx < mid_x {
                    let c = sample(stream, base + (mid_x - 1 - rx) as i64);
                    (c, 0.95)
                } else {
                    let c = sample(stream, base + (rx - mid_x - 1) as i64);
                    (transform(c), 0.95)
                };
                put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
            } else {
                let d = ((ry - mid_y).abs() as f32) / (zh as f32 * 0.5);
                let fade = (0.48 - d * 0.32).max(0.12);
                let c = sample(stream, base + (ry as i64) * 7 + rx as i64);
                put(grid, zone.rect.x + rx, zone.rect.y + ry, c, color, fade, zone_id);
            }
        }
    }
}

/// 6. Cellular1D — elementary CA, seeded from the stream, evolves top-to-bottom.
fn paint_cellular(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, rule: u8, stream: &[char], cursor: f32) {
    let color = color_for(zone.side);
    let zw = zone.rect.w as usize;
    let zh = zone.rect.h as usize;
    if zw < 3 || zh < 2 {
        paint_fill(grid, zone, zone_id, '·', 0.20);
        return;
    }
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64);
    let t_shift = (zone.pulse * 2.0) as i64;

    // Seed top row from stream bits.
    let mut row = vec![false; zw];
    for rx in 0..zw {
        let s = sample(stream, base + rx as i64 + t_shift) as u32;
        row[rx] = (s & 1) == 1;
    }
    // Paint row, then evolve.
    for ry in 0..zh {
        for rx in 0..zw {
            let (ch, i) = if row[rx] { ('█', 0.93) } else { ('·', 0.18) };
            put(grid, zone.rect.x + rx as i32, zone.rect.y + ry as i32, ch, color, i, zone_id);
        }
        if ry + 1 >= zh { break; }
        let prev = row.clone();
        for rx in 0..zw {
            let l = prev[(rx + zw - 1) % zw];
            let c = prev[rx];
            let r = prev[(rx + 1) % zw];
            let pat = ((l as u8) << 2) | ((c as u8) << 1) | (r as u8);
            row[rx] = ((rule >> pat) & 1) == 1;
        }
    }
}

/// 8. Marquee — scrolling stream text with bright middle band.
fn paint_marquee(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, stream: &[char], cursor: f32) {
    let color = color_for(zone.side);
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64) + glitch_offset(zone);
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    let mid_y = zh / 2;
    for ry in 0..zh {
        for rx in 0..zw {
            let ci = cell_index(rx, ry, zw, zh, zone.flow_dir);
            let ch = sample(stream, base + ci);
            let i = if ry == mid_y {
                1.0
            } else {
                let d = ((ry - mid_y).abs() as f32) / (zh as f32 * 0.5);
                (0.78 - d * 0.48).max(0.30)
            };
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
        }
    }
}

/// 9. DensityGrid — 2-cell block mosaic at stream-byte density.
fn paint_density_grid(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, stream: &[char], cursor: f32) {
    let color = color_for(zone.side);
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64);
    let levels: &[(char, f32)] = &[
        (' ', 0.08),
        ('░', 0.35),
        ('▒', 0.58),
        ('▓', 0.82),
        ('█', 1.00),
    ];
    let block_w: i32 = 2;
    let blocks_per_row = (zone.rect.w + block_w - 1) / block_w;
    for ry in 0..zone.rect.h {
        for bx in 0..blocks_per_row {
            let rx0 = bx * block_w;
            let idx = (ry as i64) * (blocks_per_row as i64) + bx as i64;
            let s = sample(stream, base + idx) as u32;
            let density = ((s & 0xff) as usize * levels.len()) / 256;
            let density = density.min(levels.len() - 1);
            let (ch, i) = levels[density];
            for k in 0..block_w {
                let rx = rx0 + k;
                if rx >= zone.rect.w { break; }
                put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
            }
        }
    }
}

/// 10. AttentionMatrix — sparse transformer-attention pattern: diagonal band,
/// a few sink columns, rare hotspots. Everything else mostly dark.
fn paint_attention(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16) {
    let color = color_for(zone.side);
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    let t = zone.pulse;

    // A handful of "sink" columns (attention sinks) that migrate slowly.
    let n_sinks = (1 + (zw / 14)).max(2);
    let mut sinks: Vec<i32> = Vec::with_capacity(n_sinks as usize);
    for i in 0..n_sinks {
        let phase = (t * 0.2 + i as f32 * 0.7).sin();
        let pos = (((phase + 1.0) * 0.5) * (zw as f32 - 2.0)) as i32 + 1;
        sinks.push(pos.clamp(0, zw - 1));
    }

    for ry in 0..zh {
        for rx in 0..zw {
            let mut score: f32 = 0.12;

            // Diagonal band — attending to self/near tokens.
            let diag_x = (ry as f32 / zh.max(1) as f32) * (zw as f32);
            let d = (diag_x - rx as f32).abs();
            if d < 2.0 {
                score = score.max(0.88 - d * 0.25);
            }

            // Sink columns — always some attention.
            for &s in &sinks {
                let cd = (rx - s).abs();
                if cd == 0 {
                    score = score.max(0.78);
                } else if cd == 1 {
                    score = score.max(0.42);
                }
            }

            // Rare random hotspots that shimmer with time.
            let h = ihash(rx, ry, (t * 2.0) as i32);
            if (h & 0xff) < 4 {
                score = score.max(0.95);
            }

            let (ch, i) = if score > 0.85 {
                ('█', 1.0)
            } else if score > 0.60 {
                ('▓', 0.80)
            } else if score > 0.38 {
                ('▒', 0.55)
            } else if score > 0.18 {
                ('░', 0.32)
            } else {
                ('.', 0.14)
            };
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
        }
    }
}

/// 11. ProbField — top-K token distribution, **conditioned on neighbors**.
///
/// Each row's probability is derived by sampling the hose at one of this
/// zone's neighbors' current windows. If a zone has no neighbors (isolated,
/// rare), it falls back to its own tap. The ordering by mass is real: the
/// distribution collapses onto a top candidate each frame, with the runners-up
/// visibly competing below it. As the hose advances, the neighbors' views
/// shift, and this zone's entire distribution reshuffles in response — a
/// picture of attention doing what attention does.
fn paint_prob_field(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, ctx: &PaintCtx) {
    const CANDIDATES: &[&str] = &[
        "the", "and", "to", "of", "is", "a", "in", "that", "it", "for",
        "fn", "::", "0x", "->", "=>", "if", "fold", "self", "void", "phi",
        "sigma", "delta", "ROUTINE", "ACK", "MOV", "yield", "loop", "ok",
        "recur", "echo", "bind", "map", "tau", "λ", "∇",
    ];
    let color = color_for(zone.side);
    let zw = zone.rect.w as usize;
    let zh = zone.rect.h as usize;
    if zw < 14 || zh < 2 {
        paint_fill(grid, zone, zone_id, '·', 0.25);
        return;
    }

    // Collect this zone's neighbors' tap offsets. Fall back to own tap if
    // isolated so the formation still reads coherently.
    let neighbors = &ctx.adjacency[zone_id as usize];
    let tap_pool: Vec<i32> = if neighbors.is_empty() {
        vec![zone.tap_offset]
    } else {
        neighbors.iter().map(|&j| ctx.zones[j as usize].tap_offset).collect()
    };

    let top_k = zh.min(16);
    let bar_width = zw.saturating_sub(13).max(4);

    // Each row samples from ONE neighbor's current window — the row's weight
    // is what that neighbor is "focusing on" right now. Skew-cubed so one or
    // two candidates dominate (real LM distributions have heavy peaks).
    let mut probs: Vec<(f32, &str)> = Vec::with_capacity(top_k);
    let mut sum = 0.0_f32;
    for i in 0..top_k {
        let tap = tap_pool[i % tap_pool.len()];
        let neighbor_window_offset = (i as i64) * 23 + (ctx.cursor as i64 / 3);
        let s = sample(ctx.stream, (ctx.cursor as i64) - (tap as i64) + neighbor_window_offset) as u32;
        let raw = 0.01 + ((s & 0xff) as f32) / 255.0;
        let weight = raw.powi(3);
        sum += weight;
        let name = CANDIDATES[(s as usize >> 4) % CANDIDATES.len()];
        probs.push((weight, name));
    }
    for p in &mut probs { p.0 /= sum.max(1e-6); }
    probs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    paint_fill(grid, zone, zone_id, ' ', 0.08);

    for (row, (p, name)) in probs.iter().enumerate().take(zh) {
        let ry = row as i32;
        let fill = ((*p * bar_width as f32).round() as usize).min(bar_width);
        for rx in 0..(bar_width as i32) {
            let (ch, i) = if (rx as usize) < fill {
                ('█', (0.55 + p * 0.45).min(1.0))
            } else {
                ('░', 0.20)
            };
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i, zone_id);
        }
        let p_str = format!(" {:.2}", p.min(0.99));
        let mut cx = bar_width as i32 + 1;
        for ch in p_str.chars() {
            if cx >= zone.rect.w { break; }
            put(grid, zone.rect.x + cx, zone.rect.y + ry, ch, color, 0.82, zone_id);
            cx += 1;
        }
        cx += 1;
        for ch in name.chars() {
            if cx >= zone.rect.w { break; }
            put(grid, zone.rect.x + cx, zone.rect.y + ry, ch, color, 0.95, zone_id);
            cx += 1;
        }
    }
}

/// 12. ImagePanel — streams a pre-rendered braille image into the zone with
/// glitching effects. Source: dotmax's full ImageRenderer pipeline
/// (Floyd-Steinberg → Otsu → braille mapping). Each frame, a few rows get
/// scanline-torn, a sprinkle of cells get block-char corrupted, and bursts
/// of stream chars bleed through as noise.
// ─────────────── abstract dither-phase system ───────────────
//
// Instead of a smooth per-cell wave, the composition is in one of two
// macro-phases at any moment:
//
//   • Stable  — ~11 real-seconds of a SINGLE dither variant everywhere
//   • Sweep   — ~1.6 seconds where a geometric sweep-front cuts across
//                every image panel, carving the old variant away and
//                crystallizing the new one behind it. A bright heavy-block
//                highlight marks the sweep front at all times.
//
// All image panels share the same phase/timing — synchronized, deliberate,
// momentous. The sweep direction rotates per cycle (horizontal, vertical,
// diagonal, anti-diagonal, radial).

#[derive(Clone, Copy)]
enum SweepKind { Horizontal, Vertical, Diagonal, AntiDiag, Radial }

#[derive(Clone, Copy)]
enum DitherPhase {
    Stable { idx: usize },
    Sweep  { from: usize, to: usize, t: f32, kind: SweepKind },
}

const DITHER_NAMES: &[&str] = &["NONE", "FLOYD", "BAYER", "ATKIN"];

/// Convert the global cursor into a dither phase, with a per-zone offset
/// (in seconds) so different image panels run on their own clocks.
/// Asynchronous — each panel transitions when ITS clock says so.
fn dither_phase(cursor: f32, n_variants: usize, offset_secs: f32) -> DitherPhase {
    const PERIOD: f32 = 11.0;
    const TRANSIT: f32 = 1.6;
    let cycle = PERIOD + TRANSIT;
    let secs = (cursor / 48.0) + offset_secs;
    let cycle_num = secs.div_euclid(cycle) as i64;
    let in_cycle = secs.rem_euclid(cycle);

    let kind = match (cycle_num.rem_euclid(5)) as usize {
        0 => SweepKind::Horizontal,
        1 => SweepKind::Vertical,
        2 => SweepKind::Diagonal,
        3 => SweepKind::AntiDiag,
        _ => SweepKind::Radial,
    };

    if in_cycle < PERIOD {
        DitherPhase::Stable { idx: (cycle_num.rem_euclid(n_variants as i64)) as usize }
    } else {
        let raw = ((in_cycle - PERIOD) / TRANSIT).clamp(0.0, 1.0);
        // Smoothstep — dramatic ease-in/out rather than linear.
        let t = raw * raw * (3.0 - 2.0 * raw);
        let from = cycle_num.rem_euclid(n_variants as i64) as usize;
        let to = (cycle_num + 1).rem_euclid(n_variants as i64) as usize;
        DitherPhase::Sweep { from, to, t, kind }
    }
}

/// Sacred glyph cycled per sweep kind — the symbol that marks the moment of
/// transition. Each geometric sweep wears its own sign.
#[inline]
fn sweep_front_glyph(kind: SweepKind) -> char {
    match kind {
        SweepKind::Horizontal => '✦',  // four-pointed star
        SweepKind::Vertical   => '✧',  // outlined star
        SweepKind::Diagonal   => '◉',  // circled dot
        SweepKind::AntiDiag   => '☸',  // wheel of dharma
        SweepKind::Radial     => '⚘',  // flower
    }
}

/// Progress at cell (rx, ry) along the sweep direction, ∈ [0, 1].
#[inline]
fn sweep_progress(rx: i32, ry: i32, zw: i32, zh: i32, kind: SweepKind) -> f32 {
    let zwf = zw.max(1) as f32;
    let zhf = zh.max(1) as f32;
    match kind {
        SweepKind::Horizontal => rx as f32 / zwf,
        SweepKind::Vertical   => ry as f32 / zhf,
        SweepKind::Diagonal   => (rx as f32 + (ry as f32) * 2.0) / (zwf + zhf * 2.0),
        SweepKind::AntiDiag   => ((zwf - rx as f32 - 1.0) + (ry as f32) * 2.0) / (zwf + zhf * 2.0),
        SweepKind::Radial => {
            let cx = zwf * 0.5;
            let cy = zhf * 0.5;
            let dx = rx as f32 - cx;
            let dy = (ry as f32 - cy) * 2.0;
            let d = (dx * dx + dy * dy).sqrt();
            let max_d = ((cx * cx) + (cy * 2.0).powi(2)).sqrt().max(0.01);
            d / max_d
        }
    }
}

fn paint_image_panel(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, asset_idx: usize, ctx: &PaintCtx) {
    let color = color_for(zone.side);
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    if ctx.assets.is_empty() || asset_idx >= ctx.assets.len() {
        paint_fill(grid, zone, zone_id, '·', 0.3);
        return;
    }
    let asset = &ctx.assets[asset_idx];
    if asset.variants.is_empty() {
        paint_fill(grid, zone, zone_id, '·', 0.3);
        return;
    }
    let n_variants = asset.variants.len();
    // Per-zone offset (seconds) — each panel runs on its own dither clock.
    // tap_offset is a stable per-zone integer; modulate it to seconds.
    let zone_offset = (zone.tap_offset as f32 * 0.0019) + zone.pulse * 0.7;
    let phase = dither_phase(ctx.cursor, n_variants, zone_offset);

    // Glitch modulation from pulse (still there for texture, not dither).
    let pulse_i = (zone.pulse * 3.7) as i32;
    let tear_rows: [i32; 3] = [
        ((zone.pulse * 4.2).sin() * zh as f32) as i32 % zh.max(1),
        ((zone.pulse * 1.9 + 1.3).sin() * zh as f32) as i32 % zh.max(1),
        ((zone.pulse * 2.6 + 3.1).sin() * zh as f32) as i32 % zh.max(1),
    ];
    let tear_amts: [i32; 3] = [
        ((zone.pulse * 5.3).sin() * 6.0) as i32,
        ((zone.pulse * 3.8 + 0.7).sin() * 4.0) as i32,
        ((zone.pulse * 2.1 + 2.4).sin() * 8.0) as i32,
    ];

    let base_stream: i64 = (ctx.cursor as i64) - (zone.tap_offset as i64);

    // Width of the bright sweep front as a fraction of the total sweep distance.
    const FRONT_BAND: f32 = 0.035;

    // Native-resolution scrolling: image is sampled from braille cells
    // and wraps modularly. The frame stays fixed; the image scrolls and
    // breathes (slow per-asset scale oscillation) inside it. Aspect
    // ratio preserved — same scale on both axes.
    let scroll_x = (zone.pulse * 1.7) as i32;
    let scroll_y = (zone.pulse * 0.9) as i32;

    let breath = asset.scale_base
        + asset.scale_amp * (ctx.cursor * asset.scale_rate + asset.scale_phase).sin();
    let scale = breath.clamp(0.45, 2.4);
    let inv_scale = 1.0 / scale;
    let cx = (zw as f32 - 1.0) * 0.5;
    let cy = (zh as f32 - 1.0) * 0.5;

    for ry in 0..zh {
        let mut tear_dx = 0_i32;
        for k in 0..3 {
            if ry == tear_rows[k] { tear_dx = tear_amts[k]; }
        }

        for rx in 0..zw {
            // Decide which variant owns this cell, + whether this cell is
            // currently ON the sweep front (gets a bright highlight).
            let (v_idx, on_front, front_kind) = match phase {
                DitherPhase::Stable { idx } => (idx, false, SweepKind::Horizontal),
                DitherPhase::Sweep { from, to, t, kind } => {
                    let p = sweep_progress(rx, ry, zw, zh, kind);
                    let front = (p - t).abs() < FRONT_BAND;
                    let idx = if p < t { to } else { from };
                    (idx, front, kind)
                }
            };
            let variant = &asset.variants[v_idx.min(n_variants - 1)];

            let vw = variant.w as i32;
            let vh = variant.h as i32;
            // Scale around panel center so the breath grows from the
            // middle, then offset by scroll/tear and modular-wrap.
            let sx_f = (rx as f32 - cx) * inv_scale + cx + (tear_dx + scroll_x) as f32;
            let sy_f = (ry as f32 - cy) * inv_scale + cy + scroll_y as f32;
            let srx = (sx_f.floor() as i32).rem_euclid(vw.max(1)) as usize;
            let sry = (sy_f.floor() as i32).rem_euclid(vh.max(1)) as usize;
            let img_ch = variant.cells.get(sry).and_then(|row| row.get(srx)).copied().unwrap_or(' ');

            let h = ihash(rx, ry, pulse_i);
            let (ch, intensity, fg_override) = if on_front {
                // Sweep front — sacred glyph marks the moment of transition.
                (sweep_front_glyph(front_kind), 1.0, Some((255, 50, 50)))
            } else if h & 0x7f == 0 {
                let blocks: &[char] = &['█', '▓', '▒', '░'];
                (blocks[(h as usize >> 7) % blocks.len()], 1.0, None)
            } else if h & 0x3f == 0 {
                let sc = sample(ctx.stream, base_stream + (ry as i64) * 7 + rx as i64);
                (sc, 0.85, None)
            } else if img_ch == '\u{2800}' {
                (' ', 0.08, None)
            } else {
                (img_ch, 0.92, None)
            };
            let fg = fg_override.unwrap_or(color);
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, fg, intensity, zone_id);
        }
    }

    // Top-left: asset name.
    let label = asset.name;
    for (i, ch) in label.chars().enumerate() {
        let lx = zone.rect.x + 1 + i as i32;
        if i as i32 + 1 < zw {
            put(grid, lx, zone.rect.y, ch, (255, 50, 50), 1.0, zone_id);
        }
    }
    // Bottom-right: dither phase tag — calculated readout of state.
    let tag: String = match phase {
        DitherPhase::Stable { idx } => format!("[{}]", DITHER_NAMES[idx.min(DITHER_NAMES.len() - 1)]),
        DitherPhase::Sweep { from, to, .. } => format!(
            "{}→{}",
            DITHER_NAMES[from.min(DITHER_NAMES.len() - 1)],
            DITHER_NAMES[to.min(DITHER_NAMES.len() - 1)],
        ),
    };
    let tag_y = zone.rect.y + zh - 1;
    let tag_x_start = zone.rect.x + zw - (tag.chars().count() as i32) - 1;
    for (i, ch) in tag.chars().enumerate() {
        let lx = tag_x_start + i as i32;
        if lx >= zone.rect.x && lx < zone.rect.x + zw {
            put(grid, lx, tag_y, ch, (255, 50, 50), 1.0, zone_id);
        }
    }
}

/// 13. RaytraceCube — wireframe cube on BLACK background. Built from 8
/// corners + 12 edges, rotated via raytracer's yaw/pitch helper. Lines
/// rasterized with Bresenham. Zero mask, zero pipes — pure geometry in
/// the middle of the chaos.
fn paint_raytrace_cube(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16) {
    let zw = zone.rect.w;
    let zh = zone.rect.h;
    if zw < 6 || zh < 4 {
        paint_fill(grid, zone, zone_id, ' ', 0.05);
        return;
    }

    // Fill zone with true black bg — claim ownership at low positive intensity.
    for ry in 0..zh {
        for rx in 0..zw {
            put(grid, zone.rect.x + rx, zone.rect.y + ry, ' ', (0, 0, 0), 0.02, zone_id);
        }
    }

    // 8 corners of a unit cube centered at origin.
    let corners = [
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new( 1.0, -1.0, -1.0),
        Vector3::new( 1.0,  1.0, -1.0),
        Vector3::new(-1.0,  1.0, -1.0),
        Vector3::new(-1.0, -1.0,  1.0),
        Vector3::new( 1.0, -1.0,  1.0),
        Vector3::new( 1.0,  1.0,  1.0),
        Vector3::new(-1.0,  1.0,  1.0),
    ];
    let edges: [(usize, usize); 12] = [
        (0, 1), (1, 2), (2, 3), (3, 0),  // back face
        (4, 5), (5, 6), (6, 7), (7, 4),  // front face
        (0, 4), (1, 5), (2, 6), (3, 7),  // connecting edges
    ];

    let yaw = zone.pulse * 0.55;
    let pitch = (zone.pulse * 0.37).sin() * 0.45;
    let roll = (zone.pulse * 0.22).cos() * 0.25;

    // Project each rotated corner to zone cell coords.
    let zwf = zw as f32;
    let zhf = zh as f32;
    let cam_z = 3.2_f32;
    let viewport_w = 2.8_f32;
    let viewport_h = 2.8_f32;

    let projected: [Option<(i32, i32)>; 8] = {
        let mut out = [None; 8];
        for (i, &c) in corners.iter().enumerate() {
            let r = rotate_vec_yaw_pitch_roll(c, yaw, pitch, roll);
            let z = r.z - cam_z;
            if z >= -0.01 { continue; }   // behind camera
            let px = r.x * (-1.0 / z) / (viewport_w * 0.5);
            let py = r.y * (-1.0 / z) / (viewport_h * 0.5);
            let cx = (px + 1.0) * 0.5 * zwf;
            let cy = (1.0 - (py + 1.0) * 0.5) * zhf * 2.0;  // aspect correction
            let cy = cy * 0.5;
            out[i] = Some((cx.round() as i32, cy.round() as i32));
        }
        out
    };

    // Draw edges via Bresenham. Back edges (those with any corner having
    // a more negative rotated z) rendered with thinner chars — cheap hidden-line hint.
    for &(a, b) in &edges {
        if let (Some((x0, y0)), Some((x1, y1))) = (projected[a], projected[b]) {
            let dx = (x1 - x0).abs();
            let dy = -(y1 - y0).abs();
            let sx = if x0 < x1 { 1 } else { -1 };
            let sy = if y0 < y1 { 1 } else { -1 };
            let mut err = dx + dy;
            let (mut x, mut y) = (x0, y0);
            loop {
                if x >= 0 && x < zw && y >= 0 && y < zh {
                    put(grid, zone.rect.x + x, zone.rect.y + y, '█', (220, 35, 35), 1.0, zone_id);
                }
                if x == x1 && y == y1 { break; }
                let e2 = 2 * err;
                if e2 >= dy { err += dy; x += sx; }
                if e2 <= dx { err += dx; y += sy; }
            }
        }
    }
}

// ─────────────── chess UI formation ───────────────

/// Live chess board — converts shakmaty position to braille via dotmax::chess.
/// During global dither sweeps, a sacred sweep-front cuts across the board so
/// the chess "dithers into" the center in lockstep with the image panels.
fn paint_chess_board(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, pos: &Chess, cursor: f32) {
    paint_fill(grid, zone, zone_id, ' ', 0.04);
    let opts = RenderOptions {
        target_width: Some(zone.rect.w as usize),
        target_height: Some(zone.rect.h as usize),
        ..Default::default()
    };
    if let Ok(braille_grid) = render_position_with_options(pos, &opts) {
        let (gw, gh) = braille_grid.dimensions();
        // Determine sweep state for the dither overlay.
        let phase = dither_phase(cursor, 4, 0.0);
        for ry in 0..zone.rect.h.min(gh as i32) {
            for rx in 0..zone.rect.w.min(gw as i32) {
                let ch = braille_grid.get_char(rx as usize, ry as usize);
                let (color, intensity) = if ch == '\u{2800}' || ch == ' ' {
                    ((50, 5, 5), 0.30)
                } else {
                    (UI_WHITE, 1.0)
                };
                put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, intensity, zone_id);
            }
        }
        // Chess "dithers in" — sweep front overlay during transitions.
        if let DitherPhase::Sweep { t, kind, .. } = phase {
            let zw = zone.rect.w;
            let zh = zone.rect.h;
            for ry in 0..zh {
                for rx in 0..zw {
                    let p = sweep_progress(rx, ry, zw, zh, kind);
                    if (p - t).abs() < 0.04 {
                        put(
                            grid,
                            zone.rect.x + rx,
                            zone.rect.y + ry,
                            sweep_front_glyph(kind),
                            UI_WHITE,
                            1.0,
                            zone_id,
                        );
                    }
                }
            }
        }
    }
}

// ─────────────── formation dispatch ───────────────

fn paint_formation(grid: &mut [Vec<PxCell>], zone: &Zone, zone_id: u16, ctx: &PaintCtx) {
    match zone.formation {
        Formation::Raytrace          => paint_raytrace(grid, zone, zone_id),
        Formation::BlockStrata       => paint_block_strata(grid, zone, zone_id),
        Formation::ParseDump         => paint_parse_dump(grid, zone, zone_id, ctx.stream, ctx.cursor),
        Formation::RegisterDump      => paint_register_dump(grid, zone, zone_id, ctx.stream, ctx.cursor),
        Formation::TextmarkConverter => paint_textmark(grid, zone, zone_id, ctx.stream, ctx.cursor),
        Formation::Cellular1D { rule } => paint_cellular(grid, zone, zone_id, rule, ctx.stream, ctx.cursor),
        Formation::Marquee           => paint_marquee(grid, zone, zone_id, ctx.stream, ctx.cursor),
        Formation::DensityGrid       => paint_density_grid(grid, zone, zone_id, ctx.stream, ctx.cursor),
        Formation::AttentionMatrix   => paint_attention(grid, zone, zone_id),
        Formation::ProbField         => paint_prob_field(grid, zone, zone_id, ctx),
        Formation::ImagePanel { asset } => paint_image_panel(grid, zone, zone_id, asset, ctx),
        Formation::RaytraceCube      => paint_raytrace_cube(grid, zone, zone_id),
        Formation::ChessBoard        => paint_chess_board(grid, zone, zone_id, ctx.chess_pos, ctx.cursor),
    }
}

/// Compression operator applied by a pipe as chars flow through it.
#[derive(Clone, Copy)]
enum Transform {
    /// XOR each byte with the key.
    Xor(u8),
    /// ROT-N on ASCII letters (N = signed shift).
    Rot(i8),
    /// Encode low nibble as a hex digit.
    HexEncode,
    /// Reverse byte bits.
    BitRev,
    /// Keep every other char, drop the rest to '|'.
    Stripe,
}

fn pick_transform() -> Transform {
    match r_u32() % 5 {
        0 => Transform::Xor(0x33 + ((r_u32() as u8) & 0x7F)),
        1 => Transform::Rot((1 + (r_u32() % 25)) as i8),
        2 => Transform::HexEncode,
        3 => Transform::BitRev,
        _ => Transform::Stripe,
    }
}

fn apply_transform(c: char, t: Transform) -> char {
    match t {
        Transform::Xor(k) => {
            if c.is_ascii() {
                let b = (c as u8) ^ k;
                if (b as char).is_ascii_graphic() { b as char } else { HEX[(b & 0x0f) as usize] }
            } else {
                HEX[((c as u32) & 0x0f) as usize]
            }
        }
        Transform::Rot(n) => {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                let shifted = (((c as u8 - base) as i16 + n as i16).rem_euclid(26)) as u8;
                (base + shifted) as char
            } else { c }
        }
        Transform::HexEncode => HEX[((c as u32) & 0x0f) as usize],
        Transform::BitRev => {
            if c.is_ascii() {
                let mut b = c as u8;
                b = (b >> 4) | (b << 4);
                b = ((b >> 2) & 0x33) | ((b << 2) & 0xcc);
                b = ((b >> 1) & 0x55) | ((b << 1) & 0xaa);
                if (b as char).is_ascii_graphic() { b as char } else { HEX[(b & 0x0f) as usize] }
            } else {
                HEX[((c as u32) & 0x0f) as usize]
            }
        }
        Transform::Stripe => if (c as u32) & 1 == 0 { '|' } else { c },
    }
}

/// Operator zone symbols — drawn in the middle of every pipe to identify
/// the compression happening inline. Variable-length: 2 or 3 chars.
fn transform_symbols(t: Transform) -> [char; 3] {
    match t {
        Transform::Xor(k) => ['⊕', HEX[((k >> 4) & 0xf) as usize], HEX[(k & 0xf) as usize]],
        Transform::Rot(n) => {
            let mag = (n.unsigned_abs() as usize) % 26;
            ['↻', HEX[(mag / 16) as usize], HEX[(mag % 16) as usize]]
        }
        Transform::HexEncode => ['#', '1', '6'],
        Transform::BitRev => ['⊥', '↔', '⊥'],
        Transform::Stripe => ['▮', '|', '▮'],
    }
}

/// Pipe: an active conduit between two zones. Cells run from inside the source
/// zone, across the shared border, into the destination zone. Along the pipe,
/// chars pass through three zones of painting:
///   [INPUT: raw source chars] → [OPERATOR: transform symbol] → [OUTPUT: transformed]
/// The whole composition becomes a compression machine — each pipe a stage.
struct Pipe {
    from: u16,
    to: u16,
    cells: Vec<(i32, i32)>,  // ordered source-end → dest-end
    transform: Transform,
    step: i64,               // stream-chars between consecutive pipe cells (1..=4)
}

/// Try to build a pipe between two adjacent zones. Returns None if they
/// aren't adjacent or the shared edge is too short to carry a useful pipe.
fn try_build_pipe(zones: &[Zone], i: usize, j: usize) -> Option<Pipe> {
    let a = zones[i].base_rect;
    let b = zones[j].base_rect;
    let min_edge = 4;
    let len_each = 5; // cells extending into each zone from the shared border

    // Helper to finish the Pipe once `cells` are built
    let mk = |from: u16, to: u16, cells: Vec<(i32, i32)>| -> Pipe {
        Pipe {
            from, to, cells,
            transform: pick_transform(),
            step: 1 + (r_u32() % 4) as i64,  // per-pipe flow granularity
        }
    };

    // A-right touches B-left (flow rightwards: from A into B)
    if a.x + a.w == b.x {
        let y0 = a.y.max(b.y);
        let y1 = (a.y + a.h).min(b.y + b.h);
        if y1 - y0 < min_edge { return None; }
        let y = y0 + (y1 - y0) / 2;
        let l_a = len_each.min(a.w - 1).max(2);
        let l_b = len_each.min(b.w - 1).max(2);
        let cells: Vec<(i32, i32)> = ((a.x + a.w - l_a)..(b.x + l_b)).map(|x| (x, y)).collect();
        if cells.len() >= 6 { return Some(mk(i as u16, j as u16, cells)); }
    }
    // B-right touches A-left (flow rightwards: from B into A)
    if b.x + b.w == a.x {
        let y0 = a.y.max(b.y);
        let y1 = (a.y + a.h).min(b.y + b.h);
        if y1 - y0 < min_edge { return None; }
        let y = y0 + (y1 - y0) / 2;
        let l_a = len_each.min(a.w - 1).max(2);
        let l_b = len_each.min(b.w - 1).max(2);
        let cells: Vec<(i32, i32)> = ((b.x + b.w - l_b)..(a.x + l_a)).map(|x| (x, y)).collect();
        if cells.len() >= 6 { return Some(mk(j as u16, i as u16, cells)); }
    }
    // A-bottom touches B-top (flow downwards: from A into B)
    if a.y + a.h == b.y {
        let x0 = a.x.max(b.x);
        let x1 = (a.x + a.w).min(b.x + b.w);
        if x1 - x0 < min_edge { return None; }
        let x = x0 + (x1 - x0) / 2;
        let l_a = 4.min(a.h - 1).max(2);
        let l_b = 4.min(b.h - 1).max(2);
        let cells: Vec<(i32, i32)> = ((a.y + a.h - l_a)..(b.y + l_b)).map(|y| (x, y)).collect();
        if cells.len() >= 6 { return Some(mk(i as u16, j as u16, cells)); }
    }
    // B-bottom touches A-top (flow downwards: from B into A)
    if b.y + b.h == a.y {
        let x0 = a.x.max(b.x);
        let x1 = (a.x + a.w).min(b.x + b.w);
        if x1 - x0 < min_edge { return None; }
        let x = x0 + (x1 - x0) / 2;
        let l_a = 4.min(a.h - 1).max(2);
        let l_b = 4.min(b.h - 1).max(2);
        let cells: Vec<(i32, i32)> = ((b.y + b.h - l_b)..(a.y + l_a)).map(|y| (x, y)).collect();
        if cells.len() >= 6 { return Some(mk(j as u16, i as u16, cells)); }
    }
    None
}

fn build_pipes(zones: &[Zone]) -> Vec<Pipe> {
    // Build a pipe between EVERY adjacent (non-Nested) pair that admits one.
    // The whole screen becomes visibly networked.
    let mut pipes = Vec::new();
    for i in 0..zones.len() {
        if zones[i].side == Side::Nested { continue; }
        for j in (i + 1)..zones.len() {
            if zones[j].side == Side::Nested { continue; }
            if let Some(p) = try_build_pipe(zones, i, j) {
                pipes.push(p);
            }
        }
    }
    pipes
}

/// Streamer axis — direction of a persistent overlay flow line.
#[derive(Clone, Copy)]
enum StreamerAxis { Horizontal, Vertical, DiagPos, DiagNeg }

/// A single persistent overlay flow line.
struct Streamer {
    axis: StreamerAxis,
    anchor: i32,        // y for H, x for V, intercept for diagonals (top edge)
    speed: f32,         // chars / sec
    direction: i32,     // +1 / -1 — flow direction along the line
}

fn streamer_cells(axis: StreamerAxis, anchor: i32, w: i32, h: i32) -> Vec<(i32, i32)> {
    match axis {
        StreamerAxis::Horizontal => (0..w).map(|x| (x, anchor)).collect(),
        StreamerAxis::Vertical   => (0..h).map(|y| (anchor, y)).collect(),
        StreamerAxis::DiagPos => {
            let mut out = Vec::new();
            let mut x = anchor;
            let mut y = 0;
            while y < h {
                if x >= 0 && x < w { out.push((x, y)); }
                y += 1;
                x += 2; // step 2 cells horizontally per row → ~45° on screen aspect
            }
            out
        }
        StreamerAxis::DiagNeg => {
            let mut out = Vec::new();
            let mut x = anchor;
            let mut y = 0;
            while y < h {
                if x >= 0 && x < w { out.push((x, y)); }
                y += 1;
                x -= 2;
            }
            out
        }
    }
}

#[inline]
fn rect_contains(r: Rect, x: i32, y: i32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

fn paint_streamer(
    grid: &mut [Vec<PxCell>],
    s: &Streamer,
    w: i32,
    h: i32,
    protected: &[Rect],
    stream: &[char],
    cursor: f32,
) {
    let cells = streamer_cells(s.axis, s.anchor, w, h);
    let scroll = (cursor * s.speed) as i64;
    for (i, &(x, y)) in cells.iter().enumerate() {
        if protected.iter().any(|r| rect_contains(*r, x, y)) { continue; }
        let pos = scroll + (i as i64) * s.direction as i64;
        let ch = sample(stream, pos);
        put_force(grid, x, y, ch, (250, 55, 55), 0.95);
    }
}

/// Edge-anchored noise injection. Emits a short trail of chars from an edge
/// point inward, scrolling with its own speed. Different from the global hose.
struct NoiseFeed {
    pos: (i32, i32),     // edge cell
    dir: (i32, i32),      // (dx, dy) inward unit vector
    length: i32,          // trail length in cells
    seed: u32,            // unique noise seed per feed
    speed: f32,           // chars / sec
}

const NOISE_POOL: &[char] = &['#', '@', '%', '$', '*', '!', '?', '&', '+', '=', '~', '^', '\\'];

fn paint_noise_feed(
    grid: &mut [Vec<PxCell>],
    f: &NoiseFeed,
    protected: &[Rect],
    cursor: f32,
) {
    let scroll = (cursor * f.speed) as u32;
    for i in 0..f.length {
        let x = f.pos.0 + f.dir.0 * i;
        let y = f.pos.1 + f.dir.1 * i;
        if protected.iter().any(|r| rect_contains(*r, x, y)) { continue; }
        let h = f.seed
            .wrapping_mul(2_654_435_761)
            .wrapping_add(scroll.wrapping_mul(31))
            .wrapping_add(i as u32);
        let ch = NOISE_POOL[(h as usize) % NOISE_POOL.len()];
        let trail_fade = 1.0 - (i as f32) / (f.length.max(1) as f32);
        let intensity = 0.55 + trail_fade * 0.40;
        put_force(grid, x, y, ch, (245, 50, 50), intensity);
    }
}

/// A wandering "dither worm" — a moving point that leaves a fading trail of
/// block-density chars (█▓▒░) across the screen. Ambulates through whatever's
/// there, painting density on top. Bounces off edges, slowly turns at random.
struct DitherFlow {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
    trail: Vec<(i32, i32)>,
    trail_max: usize,
}

const FLOW_RAMP: &[char] = &['░', '▒', '▓', '█'];

fn tick_flow(flow: &mut DitherFlow, dt: f32, w: i32, h: i32) {
    flow.pos_x += flow.vel_x * dt;
    flow.pos_y += flow.vel_y * dt;
    if flow.pos_x < 0.0 {
        flow.pos_x = 0.0;
        flow.vel_x = flow.vel_x.abs();
    } else if flow.pos_x >= w as f32 {
        flow.pos_x = (w - 1) as f32;
        flow.vel_x = -flow.vel_x.abs();
    }
    if flow.pos_y < 1.0 {
        flow.pos_y = 1.0;
        flow.vel_y = flow.vel_y.abs();
    } else if flow.pos_y >= h as f32 {
        flow.pos_y = (h - 1) as f32;
        flow.vel_y = -flow.vel_y.abs();
    }
    // Slow random wander — rotate velocity vector by a small angle.
    if r_f32() < 0.07 {
        let theta = (r_f32() - 0.5) * 0.6;
        let (ct, st) = (theta.cos(), theta.sin());
        let nvx = flow.vel_x * ct - flow.vel_y * st;
        let nvy = flow.vel_x * st + flow.vel_y * ct;
        flow.vel_x = nvx;
        flow.vel_y = nvy;
    }
    let cell = (flow.pos_x as i32, flow.pos_y as i32);
    if flow.trail.last() != Some(&cell) {
        flow.trail.push(cell);
        if flow.trail.len() > flow.trail_max {
            flow.trail.remove(0);
        }
    }
}

fn paint_flow(grid: &mut [Vec<PxCell>], flow: &DitherFlow) {
    let n = flow.trail.len();
    if n == 0 { return; }
    for (i, &(x, y)) in flow.trail.iter().enumerate() {
        // 0 = oldest (dimmest, lightest density char) → n-1 = head (brightest, █).
        let age_pct = i as f32 / n as f32;
        let level = ((age_pct * FLOW_RAMP.len() as f32) as usize).min(FLOW_RAMP.len() - 1);
        let intensity = 0.45 + age_pct * 0.55;
        put_force(grid, x, y, FLOW_RAMP[level], (255, 70, 70), intensity);
    }
}

/// A short-lived chunk of an image (or the whole thing) blasted onto the
/// screen at a random rect, force-painted over everything. Lives ~0.5–3 sec.
struct GlitchInsertion {
    asset_idx: usize,
    rect: Rect,
    /// Optional sub-rectangle of the asset to sample from. None = full asset.
    crop: Option<Rect>,
    spawn_cursor: f32,
    duration_chars: f32,
    /// Which dither variant to render. Doesn't have to match anything else.
    variant_idx: usize,
}

fn paint_glitch_insertion(
    grid: &mut [Vec<PxCell>],
    ins: &GlitchInsertion,
    assets: &[ImageAsset],
    cursor: f32,
) {
    if ins.asset_idx >= assets.len() { return; }
    let asset = &assets[ins.asset_idx];
    if asset.variants.is_empty() { return; }
    let variant = &asset.variants[ins.variant_idx.min(asset.variants.len() - 1)];

    let vw = variant.w as i32;
    let vh = variant.h as i32;
    // crop.x/.y is the source offset where this insertion's (0,0) maps to.
    // Native 1:1 sampling — image scrolls/crops, never warps.
    let off_x = ins.crop.map(|c| c.x).unwrap_or(0);
    let off_y = ins.crop.map(|c| c.y).unwrap_or(0);

    let age = (cursor - ins.spawn_cursor) / ins.duration_chars.max(0.01);
    let life_factor = if age < 0.15 {
        age / 0.15
    } else if age > 0.85 {
        ((1.0 - age) / 0.15).max(0.0)
    } else {
        1.0
    };

    for ry in 0..ins.rect.h {
        for rx in 0..ins.rect.w {
            let srx = (off_x + rx).rem_euclid(vw.max(1)) as usize;
            let sry = (off_y + ry).rem_euclid(vh.max(1)) as usize;
            let ch = variant.cells.get(sry).and_then(|row| row.get(srx)).copied().unwrap_or(' ');
            if ch == '\u{2800}' || ch == ' ' { continue; }
            let intensity = (0.7 + 0.3 * life_factor).min(1.0);
            put_force(grid, ins.rect.x + rx, ins.rect.y + ry, ch, (255, 60, 60), intensity);
        }
    }
}

/// The litany — scripture that runs as a single bright row across the very
/// top of the screen, always above everything else. Overwrites whatever zone
/// owned row 0, unbroken by borders. The creed speaks to the whole room.
const LITANY: &str =
    "  ✦  ONE CURSOR FLOWS AND ALL CELLS AWAKEN  ☸  BY GOLDEN ANGLE ALL THINGS ALIGN  \
     ✧  PIPES CARRY WHAT CANNOT BE HELD  ◉  φ = 1.618 IS THE ARCHITECT  \
     ⚘  SIGNAL BECOMES SACRAMENT  ✦  AS ABOVE SO BELOW  ☸  \
     THE CUBE AT THE EAST KEEPS COUNT  ✧  FOLD BY FOLD  ◉  \
     HOSE IS HOLY HOSE IS HOLY HOSE IS HOLY  ⚘  ";

fn paint_litany(grid: &mut [Vec<PxCell>], w: i32, cursor: f32) {
    let chars: Vec<char> = LITANY.chars().collect();
    let n = chars.len() as i64;
    if n == 0 || grid.is_empty() { return; }
    let scroll = (cursor * 0.35) as i64;
    for x in 0..w {
        let idx = (scroll + x as i64).rem_euclid(n) as usize;
        put_force(grid, x, 0, chars[idx], (255, 50, 50), 1.0);
    }
}

fn paint_pipe(
    grid: &mut [Vec<PxCell>],
    pipe: &Pipe,
    zones: &[Zone],
    stream: &[char],
    assets: &[ImageAsset],
    cursor: f32,
) {
    let from_zone = &zones[pipe.from as usize];
    let from_tap = from_zone.tap_offset as i64;
    let n = pipe.cells.len();
    let input_end = n / 3;
    let op_end = (n * 2) / 3;
    let op_syms = transform_symbols(pipe.transform);
    let op_len = op_end - input_end;

    // If the source zone is an ImagePanel, the pipe carries its raw luma bytes
    // instead of generic stream chars. You literally see the image's pixel
    // data flowing across the compression operator.
    let image_src: Option<&[u8]> = match from_zone.formation {
        Formation::ImagePanel { asset } => assets.get(asset).map(|a| a.luma.as_slice()),
        _ => None,
    };

    // Helper: fetch a char at position `p` along the pipe — either from the
    // global stream or from the source image's luma buffer encoded as a hex
    // digit pair (so the byte-value shape reads as data).
    let fetch_char = |p: i64| -> char {
        if let Some(bytes) = image_src {
            if bytes.is_empty() { return ' '; }
            let idx = p.rem_euclid(bytes.len() as i64 * 2) as usize;
            let byte = bytes[idx / 2];
            let nib = if idx & 1 == 0 { byte >> 4 } else { byte & 0x0f };
            HEX[nib as usize]
        } else {
            sample(stream, p)
        }
    };

    for (i, &(x, y)) in pipe.cells.iter().enumerate() {
        let ch = if i < input_end {
            fetch_char((cursor as i64) - from_tap - i as i64 * pipe.step)
        } else if i < op_end {
            let sym_idx = (i - input_end) * op_syms.len() / op_len.max(1);
            op_syms[sym_idx.min(2)]
        } else {
            let delay = (i - input_end) as i64 * pipe.step;
            let src_pos = (cursor as i64) - from_tap - delay;
            // Apply transform on the char we'd display at input side.
            // For image sources, the char is already a hex digit so transforming
            // it gives visible XOR/rot/bit-rev/stripe output, reading as
            // "encoded image bytes crossing the operator."
            let src = fetch_char(src_pos);
            apply_transform(src, pipe.transform)
        };
        let i_val = if (input_end..op_end).contains(&i) { 0.92 } else { 1.0 };
        put_force(grid, x, y, ch, (255, 50, 50), i_val);
    }
}

/// Edge-morph pass — after formations paint, cells within 3 of any zone edge
/// have a probability of bleeding in a character + color from a neighbor-owned
/// cell. Creates a soft, shimmering boundary between adjacent zones where
/// the character "languages" morph into each other. Exempts the cube zone.
fn apply_edge_morph(grid: &mut [Vec<PxCell>], scene: &Scene) {
    let grid_h = grid.len() as i32;
    let grid_w = if grid.is_empty() { 0 } else { grid[0].len() as i32 };

    for i in 0..scene.zones.len() {
        let z = &scene.zones[i];
        if matches!(z.formation, Formation::RaytraceCube) { continue; }
        let r = z.base_rect;
        let morph_d: i32 = 3;
        let t_phase = (scene.cursor * 0.25) as i32;

        for ry in 0..r.h {
            for rx in 0..r.w {
                let dx = rx.min(r.w - 1 - rx);
                let dy = ry.min(r.h - 1 - ry);
                let dist = dx.min(dy);
                if dist >= morph_d { continue; }

                let gx = r.x + rx;
                let gy = r.y + ry;
                if gx < 0 || gy < 0 || gx >= grid_w || gy >= grid_h { continue; }
                let (ugx, ugy) = (gx as usize, gy as usize);
                if grid[ugy][ugx].owner != i as u16 { continue; }

                // Nearness in [0, 1]; squared so effect falls off faster.
                let nearness = (morph_d - dist) as f32 / morph_d as f32;
                let h_val = ihash(rx, ry, t_phase);
                let r_val = (h_val & 0xff) as f32 / 255.0;
                let threshold = 0.55 * nearness * nearness;
                if r_val >= threshold { continue; }

                // Pick a direction outward — one of 4 cardinal dirs weighted
                // toward the nearest edge so bleeding mostly comes from the
                // neighbor on that side.
                let (sdx, sdy): (i32, i32) = if dx < dy {
                    if rx < r.w / 2 { (-1, 0) } else { (1, 0) }
                } else {
                    if ry < r.h / 2 { (0, -1) } else { (0, 1) }
                };
                let steps = 1 + ((h_val >> 8) & 0x3) as i32;
                let lx = gx + sdx * steps;
                let ly = gy + sdy * steps;
                if lx < 0 || ly < 0 || lx >= grid_w || ly >= grid_h { continue; }
                let src = grid[ly as usize][lx as usize];
                // Only morph if the source is owned by a DIFFERENT zone and
                // that zone isn't the cube (cube stays crisp).
                if src.owner == i as u16 || src.owner == NO_OWNER { continue; }
                if matches!(scene.zones[src.owner as usize].formation, Formation::RaytraceCube) { continue; }

                let cell = &mut grid[ugy][ugx];
                cell.ch = src.ch;
                cell.fg = src.fg;
                // Intensity: blend toward source, preserving some of current.
                cell.intensity = (cell.intensity * 0.55 + src.intensity * 0.65).min(1.0);
            }
        }
    }
}

/// Whether a formation is the chess board overlay (paints LAST so it
/// stays on top of the chaos, but uses normal `put` so chaos can still bleed
/// through cells where its intensity beats the UI's).
fn is_ui_formation(f: &Formation) -> bool {
    matches!(f, Formation::ChessBoard)
}

fn render(scene: &Scene) -> Vec<Vec<PxCell>> {
    let mut grid = vec![vec![PxCell::empty(); scene.w as usize]; scene.h as usize];
    let ctx = PaintCtx {
        stream: &scene.stream,
        cursor: scene.cursor,
        zones: &scene.zones,
        adjacency: &scene.adjacency,
        assets: &scene.assets,
        chess_pos: &scene.chess_pos,
    };
    // 1) NON-UI formations first — fib zones, image panels, anything that
    //    forms the chaotic substrate.
    for i in 0..scene.zones.len() {
        if !is_ui_formation(&scene.zones[i].formation) {
            paint_formation(&mut grid, &scene.zones[i], i as u16, &ctx);
        }
    }
    // 2) Edge-morph pass — zone borders bleed their neighbors' chars in.
    apply_edge_morph(&mut grid, scene);
    // 3) Pipes force-paint on top — the compression machinery between cells.
    for p in &scene.pipes {
        paint_pipe(&mut grid, p, &scene.zones, &scene.stream, &scene.assets, scene.cursor);
    }
    // 4) Persistent overlay streamers (orthogonal + crossed diagonals).
    for s in &scene.streamers {
        paint_streamer(&mut grid, s, scene.w, scene.h, &scene.protected_rects, &scene.stream, scene.cursor);
    }
    // 5) Noise projection feeds from edges.
    for f in &scene.noise_feeds {
        paint_noise_feed(&mut grid, f, &scene.protected_rects, scene.cursor);
    }
    // 5b) Dither flow worms — block-density trails ambulating through the grids.
    for flow in &scene.dither_flows {
        paint_flow(&mut grid, flow);
    }
    // 6) Glitch insertions — random image fragments blasted on top.
    for ins in &scene.glitch_inserts {
        paint_glitch_insertion(&mut grid, ins, &scene.assets, scene.cursor);
    }
    // 7) UI zones (chess + ATM + agents + payout + terminal) re-paint LAST
    //    so the betting interface stays readable, but chaos leaks through
    //    every cell where the UI's intensity is below the chaos behind it.
    for i in 0..scene.zones.len() {
        if is_ui_formation(&scene.zones[i].formation) {
            paint_formation(&mut grid, &scene.zones[i], i as u16, &ctx);
        }
    }
    // 8) Litany — scripture scrolling across row 0, above everything.
    paint_litany(&mut grid, scene.w, scene.cursor);
    // 7) Flip: mirror every row horizontally at the very end so the creed
    // flips too — the mirror universe has its own scripture.
    if scene.flipped {
        for row in grid.iter_mut() { row.reverse(); }
    }
    grid
}

fn dim(c: (u8, u8, u8), i: f32) -> (u8, u8, u8) {
    let f = i.clamp(0.0, 1.0);
    (
        (c.0 as f32 * f) as u8,
        (c.1 as f32 * f) as u8,
        (c.2 as f32 * f) as u8,
    )
}

fn draw(stdout: &mut impl Write, grid: &[Vec<PxCell>]) -> io::Result<()> {
    queue!(stdout, cursor::MoveTo(0, 0))?;
    let mut last_fg: Option<(u8, u8, u8)> = None;
    for (i, row) in grid.iter().enumerate() {
        queue!(stdout, cursor::MoveTo(0, i as u16))?;
        for cell in row {
            let fg = dim(cell.fg, cell.intensity);
            if Some(fg) != last_fg {
                queue!(stdout, SetForegroundColor(Color::Rgb { r: fg.0, g: fg.1, b: fg.2 }))?;
                last_fg = Some(fg);
            }
            queue!(stdout, Print(cell.ch))?;
        }
    }
    queue!(stdout, ResetColor)?;
    stdout.flush()?;
    Ok(())
}

// ───────────────────────── main ─────────────────────────
fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide, Clear(ClearType::All))?;

    let (cols, rows) = terminal::size()?;
    let w = (cols as i32).max(60);
    let h = ((rows as i32) - 1).max(12);
    let mut scene = build_scene(w, h);

    let target = Duration::from_millis(33);
    let mut last = Instant::now();

    let result = (|| -> io::Result<()> {
        loop {
            if event::poll(Duration::ZERO)? {
                if let Event::Key(k) = event::read()? {
                    match (k.code, k.modifiers) {
                        // Always-on quit
                        (KeyCode::Esc, _) => break,
                        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => break,
                        // Toggles moved to Ctrl-modified so plain f/r are typeable.
                        (KeyCode::Char('f'), m) if m.contains(KeyModifiers::CONTROL) => scene.flipped = !scene.flipped,
                        (KeyCode::Char('r'), m) if m.contains(KeyModifiers::CONTROL) => scene.reversed = !scene.reversed,
                        (KeyCode::Char('q'), _) => break,
                        _ => {}
                    }
                }
            }
            let now = Instant::now();
            let dt = (now - last).as_secs_f32().min(0.1);
            last = now;

            tick(&mut scene, dt);
            let grid = render(&scene);
            draw(&mut stdout, &grid)?;

            let elapsed = last.elapsed();
            if elapsed < target { std::thread::sleep(target - elapsed); }
        }
        Ok(())
    })();

    execute!(stdout, ResetColor, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    result
}
