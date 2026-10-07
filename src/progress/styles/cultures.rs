//! World-cultures / ornament-themed progress bars.
//!
//! Eleven structurally distinct styles, each drawing itself as `ctx.eased` rises
//! and animating via `ctx.time`.  Every style draws a recognisable cultural
//! motif or ornament in braille dot-space:
//!
//! - `mandala`        — turning lotus mandala; rosette border unrolls outward
//! - `celtic-knot`    — two outlined ribbons twisting with over/under breaks
//! - `aztec-fret`     — xicalcoliuhqui staircase-and-spiral units, left to right
//! - `islamic-star`   — chain of eight-pointed {8/3} stars blooming in turn
//! - `greek-key`      — one unbroken squared meander line drawn in from the left
//! - `seigaiha`       — Japanese overlapping fans of concentric wave arcs
//! - `totem-pole`     — carved poles with thunderbird / bear crests raised in turn
//! - `runes`          — Elder Futhark runes chiselled stroke by stroke into stone
//! - `paisley-swirl`  — boteh teardrops with curled tips traced one by one
//! - `kente-weave`    — gold / green / red kente strips woven one by one
//! - `paper-lanterns` — string of red paper lanterns lit one after another

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Public registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `cultures` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per ornament style, in display order.
/// Every style is structurally distinct — variety is in shape, not colour.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Mandala),
        Box::new(CelticKnot),
        Box::new(AztecFret),
        Box::new(IslamicStar),
        Box::new(GreekKey),
        Box::new(Seigaiha),
        Box::new(TotemPole),
        Box::new(Runes),
        Box::new(PaisleySwirl),
        Box::new(KenteWeave),
        Box::new(PaperLanterns),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Integer Bresenham line rasteriser. Step-bounded; `draw::dot_i` clips OOB.
fn bres(grid: &mut BrailleGrid, mut x0: i32, mut y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx - dy + 2).unsigned_abs() as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps + 2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Draw a circle arc (dots) from angle `a0` to `a1` (radians) around `(cx,cy)`
/// with dot-space radius `r`.  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r < 0.5 {
        return;
    }
    let arc_len = (a1 - a0).abs() * r;
    let steps = (arc_len.ceil() as usize + 2).max(4);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = a0 + t * (a1 - a0);
        let px = (cx + r * a.cos()).round() as i32;
        let py = (cy + r * a.sin()).round() as i32;
        draw::dot_i(grid, px, py);
    }
}

/// Tint every cell in the grid with a horizontal palette gradient.
fn palette_tint(grid: &mut BrailleGrid, ctx: &BarContext) {
    let (cw, ch) = grid.dimensions();
    for cy in 0..ch {
        for cx in 0..cw {
            let t = cx as f32 / cw.max(1) as f32;
            draw::tint_row(grid, cy, cx, cx, ctx.palette.sample(t));
        }
    }
}

/// Blend colour `a` toward `b` by `t` in `[0, 1]`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    Color::rgb(f(a.r, b.r), f(a.g, b.g), f(a.b, b.b))
}

/// Stamp a `#`-art sprite: every `#` becomes a `block × block` square of dots
/// whose top-left is `origin + (col, row) * block`. `flip_v` mirrors the rows.
/// Only dots with `x < clip.0` and `y >= clip.1` are set, so sprites can be
/// revealed left-to-right or bottom-up.
fn stamp(
    grid: &mut BrailleGrid,
    rows: &[&str],
    origin: (i32, i32),
    block: i32,
    flip_v: bool,
    clip: (i32, i32),
) {
    let n = rows.len();
    for (r, line) in rows.iter().enumerate() {
        let r = if flip_v { n - 1 - r } else { r } as i32;
        for (c, ch) in line.bytes().enumerate() {
            if ch != b'#' {
                continue;
            }
            for dy in 0..block {
                for dx in 0..block {
                    let x = origin.0 + c as i32 * block + dx;
                    let y = origin.1 + r * block + dy;
                    if x < clip.0 && y >= clip.1 {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Mandala — a rotating lotus mandala with a rosette border unrolling outward
// ─────────────────────────────────────────────────────────────────────────────

struct Mandala;
impl ProgressStyle for Mandala {
    fn name(&self) -> &str {
        "mandala"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Lotus mandala: a ringed six-petal lotus turns slowly at the centre while \
         a border of rosettes and diamonds unrolls symmetrically outward as progress rises."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw as f32 - 1.0) / 2.0;
        let cy = (dh as f32 - 1.0) / 2.0;
        let r = cy.min(cx).max(1.0);
        let rot = ctx.time * 0.6;

        // Core: outer ring, six lens-shaped petals around an inner ring, and a seed.
        arc(grid, cx, cy, r, 0.0, 2.0 * PI);
        let n_petals = 6usize;
        let steps = (r * 2.0) as usize + 4;
        for k in 0..n_petals {
            let a = k as f32 * 2.0 * PI / n_petals as f32 + rot;
            let (ca, sa) = (a.cos(), a.sin());
            for s in 0..=steps {
                let u = s as f32 / steps as f32;
                let rr = r * (0.42 + 0.58 * u);
                let half = (PI * u).sin() * r * 0.22;
                for side in [-1.0f32, 1.0] {
                    let px = cx + rr * ca - side * half * sa;
                    let py = cy + rr * sa + side * half * ca;
                    draw::dot_i(grid, px.round() as i32, py.round() as i32);
                }
            }
        }
        arc(grid, cx, cy, r * 0.3, 0.0, 2.0 * PI);
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);

        // Border: rosettes (ring + seed + four turning spokes) alternating with
        // small diamonds, mirrored left and right, revealed out to `reach`.
        let reach = ctx.eased * (cx - r - 1.0).max(0.0);
        let rose_r = (r * 0.4).max(1.0);
        let pitch = (rose_r * 3.4).max(4.0);
        let mut d = r + 2.0 + rose_r;
        while d + rose_r <= r + 1.0 + reach + 0.5 && d < cx + pitch {
            for side in [-1.0f32, 1.0] {
                let rx = cx + side * d;
                arc(grid, rx, cy, rose_r, 0.0, 2.0 * PI);
                draw::dot_i(grid, rx.round() as i32, cy.round() as i32);
                for q in 0..4 {
                    let a = q as f32 * PI / 2.0 - side * rot;
                    let px = rx + (rose_r + 2.0) * a.cos();
                    let py = cy + (rose_r + 2.0) * a.sin();
                    draw::dot_i(grid, px.round() as i32, py.round() as i32);
                }
                // Diamond halfway to the next rosette, once it is reached too.
                let dd = d + pitch / 2.0;
                if dd + 2.0 <= r + 1.0 + reach + 0.5 {
                    let (mx, my) = ((cx + side * dd).round() as i32, cy.round() as i32);
                    bres(grid, mx - 2, my, mx, my - 2);
                    bres(grid, mx, my - 2, mx + 2, my);
                    bres(grid, mx + 2, my, mx, my + 2);
                    bres(grid, mx, my + 2, mx - 2, my);
                }
            }
            d += pitch;
        }

        // Colour radiates from the centre: start hue at the core, end at the rim.
        let (cw, ch) = grid.dimensions();
        for cx_c in 0..cw {
            let t = ((cx_c as f32 * 2.0 + 1.0) - cx).abs() / cx.max(1.0);
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, ctx.palette.sample(t));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Celtic Knot — two outlined ribbons twisting with over/under breaks
// ─────────────────────────────────────────────────────────────────────────────

struct CelticKnot;
impl ProgressStyle for CelticKnot {
    fn name(&self) -> &str {
        "celtic-knot"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Celtic interlace: two outlined ribbons twist between border rules, each \
         passing over then under the other — the lower ribbon breaks at every crossing; \
         the knotwork extends rightward as progress rises and slowly flows with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let filled_w = (ctx.eased * dw as f32).round() as usize;
        if filled_w == 0 {
            return Ok(());
        }

        // Border rules frame the band.
        let last = filled_w.min(dw) - 1;
        draw::hline(grid, 0, last, 0);
        draw::hline(grid, 0, last, dh - 1);

        let cy = (dh as f32 - 1.0) / 2.0;
        // Strand amplitude leaves room above and below for the rules.
        let amp = (dh as f32 / 2.0 - 4.0).max(0.5);
        let period = (dh as f32 * 2.5).max(8.0);
        let phase = ctx.time * 1.2;
        // Ribbon half-width, and the separation below which the under-strand
        // is cut away so the over-strand reads as passing on top.
        let half = if dh >= 12 { 2 } else { 1 };
        let gap = (dh as f32 * 0.3).max(1.5);
        let strand = |x: i32, sign: f32| {
            let theta = 2.0 * PI * x as f32 / period + phase;
            (cy + sign * amp * theta.sin()).round() as i32
        };

        for x in 0..filled_w.min(dw) as i32 {
            let theta = 2.0 * PI * x as f32 / period + phase;
            let sep = (2.0 * amp * theta.sin()).abs();
            // Crossings sit at multiples of π; alternate which strand is on top.
            let a_on_top = ((theta / PI).round() as i64).rem_euclid(2) == 0;
            for (sign, top) in [(1.0f32, a_on_top), (-1.0, !a_on_top)] {
                if sep < gap && !top {
                    continue;
                }
                // Outlined ribbon: both edges, joined to the previous column.
                let (y0, y1) = (strand(x - 1, sign), strand(x, sign));
                for e in [-half, half] {
                    bres(grid, x - 1, y0 + e, x, y1 + e);
                }
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Aztec Fret — xicalcoliuhqui: staircase + spiral hook, tiled left to right
// ─────────────────────────────────────────────────────────────────────────────

/// One xicalcoliuhqui unit (2×2-dot blocks): a staircase climbing into a
/// squared spiral hook.
const FRET_UNIT: [&str; 6] = [
    "....######",
    "....#....#",
    "...##.##.#",
    "..##..#..#",
    ".##...####",
    "##........",
];

struct AztecFret;
impl ProgressStyle for AztecFret {
    fn name(&self) -> &str {
        "aztec-fret"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Aztec/Maya stepped fret (xicalcoliuhqui): bold staircase-and-spiral units, \
         alternately mirrored, are laid between border bands from left to right as \
         progress rises; a glint of light sweeps across the finished stone."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let fill_x = (ctx.eased * dw as f32).round() as i32;
        if fill_x <= 0 {
            return Ok(());
        }

        // Border bands top and bottom.
        let last = (fill_x as usize).min(dw) - 1;
        draw::hline(grid, 0, last, 0);
        draw::hline(grid, 0, last, dh - 1);

        // Units are 20×12 dots, centred vertically, alternately flipped so the
        // staircases interlock into a zigzag.
        let y0 = (dh as i32 - 12) / 2;
        let pitch = 20i32;
        let n_units = dw as i32 / pitch + 1;
        for u in 0..n_units {
            stamp(
                grid,
                &FRET_UNIT,
                (u * pitch, y0),
                2,
                u % 2 == 1,
                (fill_x, i32::MIN),
            );
        }

        // Stone colour with a moving glint over the laid section.
        let (cw, ch) = grid.dimensions();
        let glint = (ctx.time * 0.35).fract() * (cw as f32 + 8.0) - 4.0;
        let white = Color::rgb(255, 255, 255);
        for cx_c in 0..cw {
            let base = ctx.palette.sample(cx_c as f32 / cw.max(1) as f32);
            let near = 1.0 - ((cx_c as f32 - glint).abs() / 2.5).min(1.0);
            let col = mix(base, white, near * 0.6);
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Islamic Star — a chain of eight-pointed stars (two interlaced squares)
// ─────────────────────────────────────────────────────────────────────────────

struct IslamicStar;
impl ProgressStyle for IslamicStar {
    fn name(&self) -> &str {
        "islamic-star"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Islamic geometric: eight-pointed interlaced stars ({8/3} star polygons \
         around a ring) bloom into place one after another along a linked chain as progress \
         rises; the rings breathe and the stars glimmer with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Star circumradius fills the height; stars are linked across a gap.
        let s = ((dh as f32 - 1.0) / 2.0).max(1.0);
        let link = 3.0f32;
        let pitch = 2.0 * s + link;
        let n = (((dw as f32 + link) / pitch) as usize).max(1);
        let margin = (dw as f32 - n as f32 * pitch + link) / 2.0;
        let cy = (dh as f32 - 1.0) / 2.0;
        let shown = ctx.eased * n as f32;

        for i in 0..n {
            let f = (shown - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            // The newest star blooms outward from its centre.
            let k = 1.0 - (1.0 - f) * (1.0 - f);
            let cx = margin + i as f32 * pitch + s;
            let r = s * k;
            let p = |x: f32, y: f32| ((cx + x).round() as i32, (cy + y).round() as i32);
            // {8/3} star polygon: eight points, each joined to the third along.
            let pts: Vec<(i32, i32)> = (0..8)
                .map(|j| {
                    let a = j as f32 * PI / 4.0 + PI / 8.0;
                    p(r * a.cos(), r * a.sin())
                })
                .collect();
            for j in 0..8 {
                let (a, b) = (pts[j], pts[(j + 3) % 8]);
                bres(grid, a.0, a.1, b.0, b.1);
            }
            // Breathing inner ring.
            let ring_r = s * 0.32 * k * (0.85 + 0.15 * (ctx.time * 3.0 + i as f32).sin());
            arc(grid, cx, cy, ring_r, 0.0, 2.0 * PI);
            // Link to the previous star.
            if i > 0 && f >= 1.0 {
                let (lx0, ly) = p(-s - link, 0.0);
                let (lx1, _) = p(-s, 0.0);
                if lx0 >= 0 && ly >= 0 {
                    draw::hline(grid, lx0 as usize, lx1.max(0) as usize, ly as usize);
                }
            }
        }

        // Gradient with a glimmer wandering from star to star.
        let (cw, ch) = grid.dimensions();
        let white = Color::rgb(255, 255, 255);
        for cx_c in 0..cw {
            let star = ((cx_c as f32 * 2.0 - margin) / pitch).floor();
            let glow = ((ctx.time * 2.0 - star * 0.9).sin() * 0.5 + 0.5).powi(4);
            let col = mix(
                ctx.palette.sample(cx_c as f32 / cw.max(1) as f32),
                white,
                glow * 0.5,
            );
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Greek Key — a continuous single-line meander drawn in from the left
// ─────────────────────────────────────────────────────────────────────────────

/// One meander period (2×2-dot blocks). Tiled edge to edge, the bottom rows
/// join into one unbroken line that climbs and spirals in every period.
const MEANDER_UNIT: [&str; 6] = ["#####.", "#...#.", "#.#.#.", "#.###.", "#.....", "######"];

struct GreekKey;
impl ProgressStyle for GreekKey {
    fn name(&self) -> &str {
        "greek-key"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Greek key / meander: one unbroken squared line climbs and spirals inward \
         again and again between two border rules; it is drawn in from the left as \
         progress rises and slowly travels along the band with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let fill_x = (ctx.eased * dw as f32).round() as i32;
        if fill_x <= 0 {
            return Ok(());
        }

        // Border rules.
        let last = (fill_x as usize).min(dw) - 1;
        draw::hline(grid, 0, last, 0);
        draw::hline(grid, 0, last, dh - 1);

        // Meander band, 12 dots tall, travelling one period every few seconds.
        let period = 12i32;
        let y0 = (dh as i32 - 12) / 2;
        let shift = ((ctx.time * 3.0) as i32).rem_euclid(period);
        let n_units = dw as i32 / period + 2;
        for u in -1..n_units {
            stamp(
                grid,
                &MEANDER_UNIT,
                (u * period + shift, y0),
                2,
                false,
                (fill_x, i32::MIN),
            );
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Seigaiha — Japanese overlapping wave-scales of concentric arcs
// ─────────────────────────────────────────────────────────────────────────────

struct Seigaiha;
impl ProgressStyle for Seigaiha {
    fn name(&self) -> &str {
        "seigaiha"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Japanese seigaiha (blue ocean waves): fans of concentric arcs overlap like \
         fish scales, each row tucked behind the one below; scales are laid in from \
         the left as progress rises and the sea drifts gently with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Scale radius, horizontal pitch (one diameter) and row spacing.
        let r = (dh as f32 / 2.0).clamp(2.0, 8.0);
        let sx = 2.0 * r;
        let vy = r * 0.7;
        let drift = (ctx.time * 2.5).rem_euclid(sx);
        let reveal_x = ctx.eased * (dw as f32 + r);

        for y in 0..dh {
            let yf = y as f32;
            // Rows whose circles can reach this dot, front (lowest) first.
            let k_hi = ((yf + r) / vy).floor() as i32;
            let k_lo = ((yf - r) / vy).ceil() as i32;
            for x in 0..dw {
                let xf = x as f32;
                let mut k = k_hi;
                while k >= k_lo {
                    let ccy = k as f32 * vy;
                    let off = if k.rem_euclid(2) == 1 { r } else { 0.0 } + drift;
                    let ccx = ((xf - off) / sx).round() * sx + off;
                    let d = ((xf - ccx).powi(2) + (yf - ccy).powi(2)).sqrt();
                    if ccx <= reveal_x && d < r {
                        // The front-most scale owns this dot: concentric rings.
                        if (r - d).rem_euclid(r / 3.0) < 1.0 && d > 1.5 {
                            draw::dot(grid, x, y);
                        }
                        break;
                    }
                    k -= 1;
                }
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Totem Pole — a row of carved poles raised one by one
// ─────────────────────────────────────────────────────────────────────────────

/// Thunderbird crest, wings raised (11 dots wide, 4 tall).
const BIRD_UP: [&str; 4] = ["#...###...#", "##..#.#..##", ".#########.", "...#####..."];
/// Thunderbird crest, wings lowered.
const BIRD_DOWN: [&str; 4] = ["....###....", "....#.#....", ".#########.", "##.#####.##"];
/// Bear-head crest with rounded ears.
const BEAR_HEAD: [&str; 4] = ["..##...##..", "..#######..", "..#.###.#..", "..#######.."];
/// The carved shaft below the crest: two stacked faces with hollow eyes and
/// mouths (12 dots tall).
const POLE_BODY: [&str; 12] = [
    "..#######..",
    "..#..#..#..",
    "..#..#..#..",
    "..#######..",
    "..##...##..",
    "..#######..",
    ".#########.",
    "..#.###.#..",
    "..#######..",
    "..###.###..",
    "..#.....#..",
    "..#######..",
];

struct TotemPole;
impl ProgressStyle for TotemPole {
    fn name(&self) -> &str {
        "totem-pole"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Totem poles: a row of carved poles — stacked faces with hollow eyes, crowned \
         by a thunderbird or a bear — rises one by one from the ground as progress \
         climbs; the thunderbirds beat their wings with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Poles are 11×16 dots on a 12-dot pitch, centred, standing on the floor.
        let pitch = 12i32;
        let n = ((dw as i32 + 1) / pitch).max(1);
        let margin = (dw as i32 - n * pitch + 1) / 2;
        let y0 = dh as i32 - 16;
        let built = ctx.eased * n as f32;

        for i in 0..n {
            let f = (built - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            // The pole being raised is revealed from its base upward.
            let rows = (f * 16.0).ceil() as i32;
            let clip = (i32::MAX, y0 + 16 - rows);
            let x0 = margin + i * pitch;
            let flap = ((ctx.time * 4.0) as i64 + i64::from(i)).rem_euclid(2) == 0;
            let crest: &[&str] = match (i % 2, flap) {
                (0, true) => &BIRD_UP,
                (0, false) => &BIRD_DOWN,
                _ => &BEAR_HEAD,
            };
            stamp(grid, crest, (x0, y0), 1, false, clip);
            stamp(grid, &POLE_BODY, (x0, y0 + 4), 1, false, clip);
        }

        // Shaft in the palette gradient; crests picked out in a brighter accent.
        palette_tint(grid, ctx);
        let (cw, _) = grid.dimensions();
        if y0 >= 0 {
            let crest = mix(ctx.palette.end, Color::rgb(255, 255, 255), 0.35);
            draw::tint_row(grid, y0 as usize / 4, 0, cw.saturating_sub(1), crest);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Runes — Elder Futhark runes chiselled stroke by stroke into a stone band
// ─────────────────────────────────────────────────────────────────────────────

/// Elder Futhark runes as stroke lists `(x0, y0, x1, y1)` in a 7×12 box,
/// in futhark order: fehu, uruz, thurisaz, ansuz, raidho, kaunan, gebo,
/// hagalaz, naudiz, isa, algiz, sowilo, tiwaz, berkanan, mannaz, othala.
const RUNE_STROKES: [&[(i32, i32, i32, i32)]; 16] = [
    &[(1, 0, 1, 11), (1, 4, 5, 0), (1, 7, 5, 3)],
    &[(1, 0, 1, 11), (1, 0, 5, 4), (5, 4, 5, 11)],
    &[(1, 0, 1, 11), (1, 3, 5, 6), (5, 6, 1, 9)],
    &[(1, 0, 1, 11), (1, 0, 5, 4), (1, 4, 5, 8)],
    &[(1, 0, 1, 11), (1, 0, 5, 3), (5, 3, 1, 6), (1, 6, 5, 11)],
    &[(5, 2, 1, 6), (1, 6, 5, 10)],
    &[(0, 1, 6, 10), (6, 1, 0, 10)],
    &[(0, 0, 0, 11), (5, 0, 5, 11), (0, 3, 5, 7)],
    &[(3, 0, 3, 11), (0, 4, 6, 8)],
    &[(3, 0, 3, 11)],
    &[(3, 0, 3, 11), (3, 5, 0, 1), (3, 5, 6, 1)],
    &[(5, 0, 1, 4), (1, 4, 5, 7), (5, 7, 1, 11)],
    &[(3, 0, 3, 11), (3, 0, 0, 4), (3, 0, 6, 4)],
    &[
        (1, 0, 1, 11),
        (1, 0, 5, 3),
        (5, 3, 1, 5),
        (1, 5, 5, 8),
        (5, 8, 1, 11),
    ],
    &[(0, 0, 0, 11), (5, 0, 5, 11), (0, 0, 5, 6), (5, 0, 0, 6)],
    &[(3, 0, 6, 3), (3, 0, 0, 3), (6, 3, 0, 10), (0, 3, 6, 10)],
];

struct Runes;
impl ProgressStyle for Runes {
    fn name(&self) -> &str {
        "runes"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Norse Elder Futhark runes: angular rune-staves are chiselled stroke by stroke \
         into a stone band, in futhark order, as progress rises; sparks flicker at \
         the chisel point."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Stone band rules top and bottom.
        draw::hline(grid, 0, dw - 1, 0);
        draw::hline(grid, 0, dw - 1, dh - 1);

        // Rune slots on a 10-dot pitch; strokes are two dots thick.
        let pitch = 10i32;
        let n = ((dw as i32 - 2) / pitch).max(1);
        let margin = (dw as i32 - n * pitch + 2) / 2;
        let oy = if dh >= 8 { 2 } else { 0 };
        let sy = ((dh as f32 - 1.0 - 2.0 * oy as f32) / 11.0).max(0.0);
        let carved = ctx.eased * n as f32;
        let spark_on = ((ctx.time * 10.0) as i64).rem_euclid(2) == 0;

        for i in 0..n {
            let f = (carved - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let strokes = RUNE_STROKES[i as usize % RUNE_STROKES.len()];
            let x0 = margin + i * pitch;
            let map = |x: i32, y: i32| (x0 + x, oy + (y as f32 * sy).round() as i32);
            for (s, &(ax, ay, bx, by)) in strokes.iter().enumerate() {
                let g = (f * strokes.len() as f32 - s as f32).clamp(0.0, 1.0);
                if g <= 0.0 {
                    break;
                }
                let (px0, py0) = map(ax, ay);
                let (px1, py1) = map(bx, by);
                let ex = (px0 as f32 + (px1 - px0) as f32 * g).round() as i32;
                let ey = (py0 as f32 + (py1 - py0) as f32 * g).round() as i32;
                bres(grid, px0, py0, ex, ey);
                bres(grid, px0 + 1, py0, ex + 1, ey);
                // Chisel sparks at the stroke being cut.
                if g < 1.0 && spark_on {
                    for (dx, dy) in [(-2, -2), (3, -2), (-2, 2), (3, 2)] {
                        draw::dot_i(grid, ex + dx, ey + dy);
                    }
                }
            }
        }

        // Carved section in the palette gradient; uncut stone stays dim.
        let (cw, ch) = grid.dimensions();
        let cut_cell = ((margin as f32 + carved * pitch as f32) / 2.0).max(0.0) as usize;
        let stone = mix(ctx.palette.track, ctx.palette.start, 0.3);
        for cx_c in 0..cw {
            let col = if cx_c <= cut_cell {
                ctx.palette.sample(cx_c as f32 / cw.max(1) as f32)
            } else {
                stone
            };
            for cy_c in 0..ch {
                draw::tint_row(grid, cy_c, cx_c, cx_c, col);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Paisley Swirl — boteh teardrops traced one after another
// ─────────────────────────────────────────────────────────────────────────────

/// Append a quadratic Bézier `p0 → p2` (control `c`) to `pts`.
fn quad(pts: &mut Vec<(f32, f32)>, p0: (f32, f32), c: (f32, f32), p2: (f32, f32)) {
    let len = ((c.0 - p0.0).hypot(c.1 - p0.1) + (p2.0 - c.0).hypot(p2.1 - c.1)).max(1.0);
    let n = (len * 2.0).ceil() as usize;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let u = 1.0 - t;
        pts.push((
            u * u * p0.0 + 2.0 * u * t * c.0 + t * t * p2.0,
            u * u * p0.1 + 2.0 * u * t * c.1 + t * t * p2.1,
        ));
    }
}

/// Append a circular arc around `c` from angle `a0` to `a1` to `pts`.
fn ring(pts: &mut Vec<(f32, f32)>, c: (f32, f32), r: f32, a0: f32, a1: f32) {
    let n = ((a1 - a0).abs() * r * 2.0).ceil().max(4.0) as usize;
    for i in 0..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        pts.push((c.0 + r * a.cos(), c.1 + r * a.sin()));
    }
}

/// Trace order of one paisley (boteh) of bowl radius `r`, in y-up local
/// coordinates centred on the bowl: rounded bowl, left flank up to the tip,
/// the curled hook, the right flank back down, then the inner eye.
fn paisley_points(r: f32) -> Vec<(f32, f32)> {
    let mut pts = Vec::new();
    ring(&mut pts, (0.0, 0.0), r, 0.0, -PI);
    let tip = (0.9 * r, 2.0 * r);
    quad(&mut pts, (-r, 0.0), (-r, 1.8 * r), tip);
    let cr = 0.4 * r;
    let cc = (tip.0, tip.1 - cr);
    ring(&mut pts, cc, cr, PI / 2.0, -PI);
    quad(&mut pts, (cc.0 + cr, cc.1), (1.25 * r, 0.7 * r), (r, 0.0));
    ring(&mut pts, (0.0, 0.15 * r), 0.4 * r, 0.0, 2.0 * PI);
    pts.push((0.0, 0.15 * r));
    pts
}

struct PaisleySwirl;
impl ProgressStyle for PaisleySwirl {
    fn name(&self) -> &str {
        "paisley-swirl"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Paisley (boteh): teardrop motifs with a curled tip and an inner eye are \
         traced one after another, alternately mirrored, as progress rises; each \
         sways gently on its base with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Bowl radius from the height: the motif stands 3r tall, 2.3r wide.
        let r = ((dh as f32 - 1.0) / 3.0).clamp(0.5, 8.0);
        let pitch = (r * 3.2).max(2.0);
        let n = ((dw as f32 / pitch) as usize).max(1);
        let margin = (dw as f32 - n as f32 * pitch) / 2.0 + 0.2 * pitch;
        let pts = paisley_points(r);
        let traced = ctx.eased * n as f32;

        for i in 0..n {
            let f = (traced - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let shown = ((f * pts.len() as f32).ceil() as usize).min(pts.len());
            let bx = margin + i as f32 * pitch + r;
            let by = dh as f32 - 1.0 - r;
            let mirror = i % 2 == 1;
            let sway = 0.12 * (ctx.time * 1.8 + i as f32).sin();
            let (cs, sn) = (sway.cos(), sway.sin());
            for &(x, y) in &pts[..shown] {
                let x = if mirror { 0.3 * r - x } else { x };
                let px = bx + x * cs - y * sn;
                let py = by - (x * sn + y * cs);
                draw::dot_i(grid, px.round() as i32, py.round() as i32);
            }
        }

        palette_tint(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Kente Weave — narrow kente strips woven one by one, side by side
// ─────────────────────────────────────────────────────────────────────────────

/// Kente gold, green and red — the cloth's signature colours (black is the
/// unlit gaps between strips).
const KENTE_GOLD: Color = Color::rgb(240, 180, 30);
const KENTE_GREEN: Color = Color::rgb(30, 160, 70);
const KENTE_RED: Color = Color::rgb(210, 45, 40);

struct KenteWeave;
impl ProgressStyle for KenteWeave {
    fn name(&self) -> &str {
        "kente-weave"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Kente cloth: narrow strips of gold, green and red blocks — weft bars, warp \
         stripes, checks and solid panels — are woven one by one and laid side by \
         side as progress rises; a shuttle darts across the strip on the loom."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Strips are 7 dots of cloth plus a 1-dot seam, one cell-row per block.
        let strip = 8usize;
        let n = (dw / strip).max(1);
        let woven = ctx.eased * n as f32;
        let (cw, ch) = grid.dimensions();

        for i in 0..n {
            let f = (woven - i as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let x0 = i * strip;
            // The strip on the loom is woven top-down.
            let rows = ((f * dh as f32).ceil() as usize).min(dh);
            for y in 0..rows {
                let block = y / 4;
                let motif = (i * 3 + block) % 4;
                for dx in 0..strip - 1 {
                    let (lx, ly) = (dx, y % 4);
                    let on = match motif {
                        0 => ly % 2 == 0,                // weft bars
                        1 => true,                       // solid panel
                        2 => lx % 2 == 0,                // warp stripes
                        _ => (lx / 2 + ly / 2) % 2 == 0, // checks
                    };
                    if on {
                        draw::dot(grid, x0 + dx, y);
                    }
                }
            }

            // Colour each block cell: gold dominates, green / red accents.
            for block in 0..ch {
                let col = match (i + block) % 4 {
                    0 | 2 => KENTE_GOLD,
                    1 => KENTE_GREEN,
                    _ => KENTE_RED,
                };
                let c0 = x0 / 2;
                draw::tint_row(
                    grid,
                    block,
                    c0,
                    (c0 + strip / 2 - 1).min(cw.saturating_sub(1)),
                    col,
                );
            }

            // Shuttle darting across the weaving line.
            if f < 1.0 {
                let sx = x0 as f32 + (0.5 + 0.5 * (ctx.time * 9.0).sin()) * (strip - 2) as f32;
                let sy = rows.min(dh - 1);
                draw::fill_rect(grid, sx as usize, sy, 2, 1);
                draw::tint_row(
                    grid,
                    sy / 4,
                    sx as usize / 2,
                    sx as usize / 2,
                    Color::rgb(255, 255, 255),
                );
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Paper Lanterns — a string of round lanterns lit one after another
// ─────────────────────────────────────────────────────────────────────────────

/// Unlit lantern: paper outline, caps and tassel cord (9×12 dots).
const LANTERN_DARK: [&str; 12] = [
    "...###...",
    ".##...##.",
    "#.......#",
    "#.......#",
    "#.......#",
    "#.......#",
    "#.......#",
    ".##...##.",
    "...###...",
    "....#....",
    "....#....",
    ".........",
];
/// Lit lantern: glowing body with two dark ribs, caps and a full tassel.
const LANTERN_LIT: [&str; 12] = [
    "...###...",
    ".#######.",
    "##.###.##",
    "##.###.##",
    "##.###.##",
    "##.###.##",
    "##.###.##",
    ".#######.",
    "...###...",
    "....#....",
    "...###...",
    "...###...",
];

struct PaperLanterns;
impl ProgressStyle for PaperLanterns {
    fn name(&self) -> &str {
        "paper-lanterns"
    }
    fn theme(&self) -> &str {
        "cultures"
    }
    fn describe(&self) -> &str {
        "Paper lanterns: round red lanterns with gold caps and tassels hang from a \
         sagging festival string; each glows alight from the bottom up in turn as \
         progress rises, and they sway and flicker with time."
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Festival string: a shallow sag across the full width.
        let sag = |x: f32| {
            let u = x / dw.max(1) as f32 * 2.0 - 1.0;
            (1.5 * (1.0 - u * u)).round() as i32
        };
        for x in 0..dw {
            draw::dot_i(grid, x as i32, sag(x as f32));
        }

        // Lanterns: 9 dots wide on a 12-dot pitch, hung 2 dots below the string.
        let pitch = 12i32;
        let n = ((dw as i32 + 3) / pitch).max(1);
        let margin = (dw as i32 - n * pitch + 3) / 2;
        let lit = ctx.eased * n as f32;
        let (cw, ch) = grid.dimensions();
        let red = Color::rgb(225, 40, 35);
        let gold = Color::rgb(245, 185, 45);
        let dim = mix(ctx.palette.track, Color::rgb(160, 160, 170), 0.35);
        for cy_c in 0..ch {
            draw::tint_row(grid, cy_c, 0, cw.saturating_sub(1), mix(gold, dim, 0.4));
        }

        for i in 0..n {
            let sway = (1.2 * (ctx.time * 2.0 + i as f32 * 0.7).sin()).round() as i32;
            let x0 = margin + i * pitch + sway;
            let mid = x0 + 4;
            let y0 = 3;
            for y in sag(mid as f32)..y0 {
                draw::dot_i(grid, mid, y);
            }
            // Fraction of this lantern alight, glowing from the bottom up.
            let f = (lit - i as f32).clamp(0.0, 1.0);
            let glow_y = y0 + 12 - (f * 12.0).ceil() as i32;
            stamp(
                grid,
                &LANTERN_DARK,
                (x0, y0),
                1,
                false,
                (i32::MAX, i32::MIN),
            );
            stamp(grid, &LANTERN_LIT, (x0, y0), 1, false, (i32::MAX, glow_y));

            // Lit cells are red with gold cap / tassel rows; the rest stay dim.
            let flicker = 0.12 * (ctx.time * 7.0 + i as f32 * 1.7).sin().max(0.0);
            let c0 = (x0.max(0) / 2) as usize;
            let c1 = ((x0 + 8).max(0) / 2) as usize;
            for cy_c in 0..ch {
                let lit_cell = f > 0.0 && (cy_c as i32 * 4 + 3) >= glow_y;
                let col = if !lit_cell {
                    dim
                } else if cy_c == 0 || cy_c * 4 >= (y0 + 8) as usize {
                    gold
                } else {
                    mix(red, gold, flicker)
                };
                draw::tint_row(grid, cy_c, c0, c1.min(cw.saturating_sub(1)), col);
            }
        }
        Ok(())
    }
}
