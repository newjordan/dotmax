//! Streaming zones — sacred-geometry hose, bilateral mirror, nested boxes, chaos accents.
//!
//! Composition:
//!   • Whole screen is split at the vertical midline — bilateral mirror axis.
//!   • Each half is recursively subdivided by the golden ratio (φ) into a
//!     Fibonacci spiral of rectangles (clockwise left, counter-clockwise right).
//!   • A single shared char stream snakes through every canvas (same hose).
//!     Left half walks the spiral outermost→innermost; right half reverses it
//!     so the two spirals visibly flow in opposing directions.
//!   • 3–4 biggest canvases get **nested children** at golden-ratio insets,
//!     rendered on top (boxes within boxes).
//!   • Chaos accents: ~8% of tiles get magenta color, ~20% get a randomized
//!     internal flow direction, ~15% have a glitch-offset that jumps them
//!     to a different phase of the stream on a slow cadence.
//!
//! Quit: q, Esc, or Ctrl-C.
//! Run:  cargo run --example zone_stream --release

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    cell::Cell as StdCell,
    io::{self, Write},
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

enum Formation {
    Raw,
    HexDump,
    BitGrid,
    Wave { freq: f32 },
    Marquee,
}

fn pick_formation(rect: Rect, chaos: f32) -> Formation {
    if r_f32() < chaos {
        return match r_u32() % 5 {
            0 => Formation::Raw,
            1 => Formation::HexDump,
            2 => Formation::BitGrid,
            3 => Formation::Wave { freq: 0.28 + r_f32() * 0.40 },
            _ => Formation::Marquee,
        };
    }
    let aspect = (rect.w as f32) / (rect.h.max(1) as f32);
    if aspect > 3.2       { Formation::Marquee }
    else if aspect < 0.65 { Formation::BitGrid }
    else if (aspect - 1.0).abs() < 0.35 { Formation::Wave { freq: 0.30 + r_f32() * 0.40 } }
    else                  { Formation::HexDump }
}

/// Golden angle, the canonical phyllotaxis spacing: 2π × (1 − 1/φ) radians.
/// Seeds each zone's breath phase so no two neighbors ever peak together.
const GOLDEN_ANGLE: f32 = 2.399_963_2;

struct Zone {
    base_rect: Rect,       // anchor rect from the fib spiral (never changes)
    rect: Rect,            // live animated rect — rendered each frame

    // Grow/fold animation
    grow_phase: f32,       // radians, advances with dt
    grow_rate: f32,        // rad/sec — breath speed
    grow_amp_w: f32,       // 0..~0.4, fraction of base width the zone can swell by
    grow_amp_h: f32,
    anchor_x: f32,         // 0 = grow right, 0.5 = grow both ways, 1 = grow left
    anchor_y: f32,

    side: Side,
    formation: Formation,
    flow_dir: FlowDir,
    tap_offset: i32,
    pulse: f32,
    pulse_rate: f32,
    glitch_rate: f32,   // 0 = steady, higher = more phase jumps
}

fn make_zone(
    base: Rect,
    side: Side,
    formation: Formation,
    flow_dir: FlowDir,
    tap: i32,
    zone_idx: usize,
) -> Zone {
    // Smaller tiles get bigger amplitude so the inner spiral folds more dramatically.
    let area = (base.w * base.h) as f32;
    let size_fraction = (area / 1800.0).min(1.0);
    let amp_base = 0.09 + (1.0 - size_fraction.sqrt()) * 0.22;  // 0.09..0.31

    Zone {
        base_rect: base,
        rect: base,
        grow_phase:  (zone_idx as f32) * GOLDEN_ANGLE + r_f32() * 0.4,
        grow_rate:   0.55 + r_f32() * 0.95,    // ~4–11s full breath
        grow_amp_w:  amp_base * (0.7 + r_f32() * 0.6),
        grow_amp_h:  amp_base * (0.7 + r_f32() * 0.6),
        anchor_x:    r_f32(),
        anchor_y:    r_f32(),

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
        let formation = pick_formation(*rect, 0.25);
        let idx = zones.len();
        zones.push(make_zone(*rect, side, formation, flow_dir, tap_accum as i32, idx));
        tap_accum += (rect.w * rect.h) as i64;
    }

    // RIGHT: walk innermost → outermost so the hose reverses direction,
    // creating the mirrored flow. Default flow RowMajorRev.
    for rect in right_spiral.iter().rev() {
        let side = if r_f32() < 0.08 { Side::Chaos } else { Side::R };
        let flow_dir = if r_f32() < 0.20 { random_flow() } else { FlowDir::RowMajorRev };
        let formation = pick_formation(*rect, 0.25);
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
        let formation = pick_formation(child, 0.5); // children lean chaotic
        let idx = zones.len();
        let mut z = make_zone(child, Side::Nested, formation, random_flow(), tap_accum as i32, idx);
        // Children breathe faster + bigger to pulse over their parent like a heart.
        z.grow_rate *= 1.35;
        z.grow_amp_w *= 1.4;
        z.grow_amp_h *= 1.4;
        z.glitch_rate = 0.15 + r_f32() * 0.4;
        zones.push(z);
        tap_accum += (child.w * child.h) as i64;
    }

    let stream = build_stream(32_768);
    Scene { w, h, stream, cursor: 0.0, flow_rate: 48.0, zones }
}

// ───────────────────────── simulation ─────────────────────────
fn tick(scene: &mut Scene, dt: f32) {
    scene.cursor += scene.flow_rate * dt;
    for z in &mut scene.zones {
        z.pulse += dt * z.pulse_rate;
        z.grow_phase += dt * z.grow_rate;

        // Breath signal ∈ [0, 1] — always non-negative so zones never shrink
        // below base (no gaps in the tiling; all motion is positive overlap).
        // H and W breathe on slightly offset phases for organic, non-square motion.
        let bw = (z.grow_phase.sin() + 1.0) * 0.5;
        let bh = ((z.grow_phase + 0.73).sin() + 1.0) * 0.5;

        let aw = ((z.base_rect.w as f32) * (1.0 + bw * z.grow_amp_w)).round() as i32;
        let ah = ((z.base_rect.h as f32) * (1.0 + bh * z.grow_amp_h)).round() as i32;
        let dw = aw - z.base_rect.w;
        let dh = ah - z.base_rect.h;

        z.rect = Rect {
            x: z.base_rect.x - ((dw as f32) * z.anchor_x).round() as i32,
            y: z.base_rect.y - ((dh as f32) * z.anchor_y).round() as i32,
            w: aw.max(2),
            h: ah.max(2),
        };
    }
}

// ───────────────────────── rendering ─────────────────────────
#[derive(Clone, Copy)]
struct PxCell { ch: char, fg: (u8, u8, u8), intensity: f32 }
impl PxCell {
    const fn empty() -> Self { Self { ch: ' ', fg: (0, 0, 0), intensity: 0.0 } }
}

fn put(grid: &mut [Vec<PxCell>], x: i32, y: i32, ch: char, c: (u8, u8, u8), i: f32) {
    if y < 0 || x < 0 { return; }
    let (uy, ux) = (y as usize, x as usize);
    if uy >= grid.len() || ux >= grid[0].len() { return; }
    let cell = &mut grid[uy][ux];
    if i >= cell.intensity {
        cell.ch = ch;
        cell.fg = c;
        cell.intensity = i;
    }
}

fn color_for(side: Side) -> (u8, u8, u8) {
    // Pure grayscale — signal comes from intensity + char-weight, not hue.
    // Side identity survives as small brightness differences at full intensity.
    match side {
        Side::L | Side::R => (218, 218, 218),  // matte white
        Side::Chaos       => (255, 255, 255),  // hot white pops through
        Side::Nested      => (242, 242, 242),  // bright white, not quite hot
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

fn paint_formation(grid: &mut [Vec<PxCell>], zone: &Zone, stream: &[char], cursor: f32) {
    let color = color_for(zone.side);
    let base: i64 = (cursor as i64) - (zone.tap_offset as i64) + glitch_offset(zone);
    let zw = zone.rect.w;
    let zh = zone.rect.h;

    match zone.formation {
        Formation::Raw => {
            for ry in 0..zh {
                for rx in 0..zw {
                    let ci = cell_index(rx, ry, zw, zh, zone.flow_dir);
                    let ch = sample(stream, base + ci);
                    put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, 0.88);
                }
            }
        }
        Formation::HexDump => {
            for ry in 0..zh {
                for rx in 0..zw {
                    let ci = cell_index(rx, ry, zw, zh, zone.flow_dir);
                    let s = sample(stream, base + ci / 2) as u32;
                    let nibble = if ci & 1 == 0 { s >> 4 } else { s };
                    let ch = HEX[(nibble & 0x0f) as usize];
                    let jitter = ((s >> 8) & 0x7) as f32 * 0.022;
                    put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, 0.92 - jitter);
                }
            }
        }
        Formation::BitGrid => {
            for ry in 0..zh {
                for rx in 0..zw {
                    let ci = cell_index(rx, ry, zw, zh, zone.flow_dir);
                    let s = sample(stream, base + ci / 8) as u32;
                    let bit = (s >> (ci & 7)) & 1;
                    let ch = if bit == 1 { '1' } else { '0' };
                    let i = if bit == 1 { 0.95 } else { 0.52 };
                    put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i);
                }
            }
        }
        Formation::Wave { freq } => {
            for ry in 0..zh {
                for rx in 0..zw {
                    let ci = cell_index(rx, ry, zw, zh, zone.flow_dir);
                    let ch = sample(stream, base + ci);
                    put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, 0.33);
                }
            }
            let yc = (zh as f32 - 1.0) * 0.5;
            let amp_max = (zh as f32 * 0.5) - 1.0;
            for rx in 0..zw {
                let s = sample(stream, base + rx as i64) as u32;
                let mod_amp = 0.55 + 0.45 * (((s & 0x1f) as f32) / 31.0);
                let ang = rx as f32 * freq + zone.pulse * 4.6;
                let y = yc + ang.sin() * amp_max * mod_amp;
                let yi = y.round() as i32;
                if yi >= 0 && yi < zh {
                    put(grid, zone.rect.x + rx, zone.rect.y + yi, '~', color, 1.0);
                    if yi - 1 >= 0 { put(grid, zone.rect.x + rx, zone.rect.y + yi - 1, '`', color, 0.70); }
                    if yi + 1 < zh { put(grid, zone.rect.x + rx, zone.rect.y + yi + 1, ',', color, 0.70); }
                }
            }
        }
        Formation::Marquee => {
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
                    put(grid, zone.rect.x + rx, zone.rect.y + ry, ch, color, i);
                }
            }
        }
    }
}

/// Meta-scale pattern field ∈ [0, 1]. Two counter-rotating spiral arms + a
/// horizontal sweep, phase driven by the shared `cursor` so this pattern
/// moves in lockstep with the hose — the meta-waves literally ride the data.
fn meta_mask(x: i32, y: i32, cursor: f32, w: i32, h: i32) -> f32 {
    let cx = (w as f32) * 0.5;
    let cy = (h as f32) * 0.5;
    // Terminal cells are ~2:1 tall — correct aspect so spirals look circular.
    let xf = (x as f32) - cx;
    let yf = ((y as f32) - cy) * 2.0;
    let t = cursor * 0.009;

    let r = (xf * xf + yf * yf).sqrt() + 0.01;
    let theta = yf.atan2(xf);

    // Two counter-rotating spiral arms (like a galaxy).
    let arm_a = ((theta * 2.0 + r * 0.18 - t * 4.2).sin() + 1.0) * 0.5;
    let arm_b = ((-theta * 3.0 + r * 0.12 + t * 3.1).sin() + 1.0) * 0.5;
    // Horizontal sweep band to break the radial symmetry.
    let sweep = ((xf * 0.11 + t * 2.7).sin() + 1.0) * 0.5;

    arm_a.max(arm_b).max(sweep * 0.78)
}

/// Walk the grid and apply the meta-mask: promote cells under mask peaks to
/// heavy block chars (█/▓), dim cells under troughs, leaving the middle
/// band alone. Creates visible meta-scale bands rolling across all zones.
fn apply_meta_mask(grid: &mut [Vec<PxCell>], cursor: f32, w: i32, h: i32) {
    for y in 0..h as usize {
        for x in 0..w as usize {
            let m = meta_mask(x as i32, y as i32, cursor, w, h);
            let cell = &mut grid[y][x];

            // Baseline intensity modulation — troughs dim by up to 45%.
            cell.intensity *= 0.55 + 0.48 * m;

            // Heavy overlays at the peaks carve the meta-pattern onto the data.
            if m > 0.94 {
                cell.ch = '█';
                cell.fg = (255, 255, 255);
                cell.intensity = 1.0;
            } else if m > 0.85 {
                cell.ch = '▓';
                cell.intensity = cell.intensity.max(0.85);
            } else if m > 0.76 {
                // Keep formation char but ensure it reads brightly.
                cell.intensity = (cell.intensity + 0.08).min(1.0);
            }
        }
    }
}

fn render(scene: &Scene) -> Vec<Vec<PxCell>> {
    let mut grid = vec![vec![PxCell::empty(); scene.w as usize]; scene.h as usize];
    for z in &scene.zones {
        paint_formation(&mut grid, z, &scene.stream, scene.cursor);
    }
    apply_meta_mask(&mut grid, scene.cursor, scene.w, scene.h);
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
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => break,
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
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
