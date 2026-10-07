//! Penrose / aperiodic-tiling sacred-geometry progress styles.
//!
//! Ten structurally distinct styles, each built on a real aperiodic or
//! quasi-periodic mathematical construction:
//!
//! - `penrose-p3`            — Fat & thin rhomb tiling via Robinson-triangle subdivision
//! - `penrose-p2`            — Kite & dart tiling via Robinson-triangle subdivision
//! - `sun-pattern`           — The canonical Penrose "sun" (5 kites) seed cluster
//! - `girih-tiles`           — frieze of girih decagons, each with its ten-pointed strapwork star
//! - `ammann-bars`           — five families of Ammann bars, Fibonacci L/S spacing, over rhomb vertices
//! - `debruijn-pentagrid`    — de Bruijn pentagrid: 5 families of parallel lines at 72°
//! - `decagon-fractal`       — frieze of decagons, each holding a ring of ten and a 1/φ² copy
//! - `quasicrystal-diffraction` — 10-fold Bragg-spot diffraction patterns, rings in φ ratio
//! - `pinwheel-tiling`       — Pinwheel 1:2 right-triangle substitution tiling
//! - `truchet-quasi`         — Truchet arcs on a Penrose rhomb tiling, joining into endless curves

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

const PHI: f32 = 1.618_034;

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — sunset tiling.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 146, 94);
const TINT_END: Color = Color::rgb(146, 86, 255);

/// Sample the theme gradient at `t` in `0.0..=1.0`.
fn sample_tint(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
    Color::rgb(
        lerp(TINT_START.r, TINT_END.r),
        lerp(TINT_START.g, TINT_END.g),
        lerp(TINT_START.b, TINT_END.b),
    )
}

/// Applies the theme's signature gradient to every cell the inner style drew,
/// drifting slowly with `time`. Styles stay monochrome-safe underneath: drop
/// the wrapper in [`styles`] for uncolored output.
struct Tinted<S>(S);

impl<S: ProgressStyle> ProgressStyle for Tinted<S> {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn theme(&self) -> &str {
        self.0.theme()
    }
    fn describe(&self) -> &str {
        self.0.describe()
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        self.0.render(grid, ctx)?;
        grid.enable_color_support();
        let (w, h) = grid.dimensions();
        for y in 0..h {
            for x in 0..w {
                let ch = grid.get_char(x, y);
                if ch != '\u{2800}' && ch != ' ' {
                    let t = (x as f32 / w.max(1) as f32 + ctx.time * 0.05).fract();
                    let tri = 1.0 - (2.0 * t - 1.0).abs();
                    let _ = grid.set_cell_color(x, y, sample_tint(tri));
                }
            }
        }
        Ok(())
    }
}

/// All styles in the `penrose` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per aperiodic-tiling bar, ordered from
/// most iconic (P3 rhombus) to most exotic (Truchet-quasi).  All styles are
/// independent and can be used in any order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(PenroseP3)),
        Box::new(Tinted(PenroseP2)),
        Box::new(Tinted(SunPattern)),
        Box::new(Tinted(GirihTiles)),
        Box::new(Tinted(AmmannBars)),
        Box::new(Tinted(DeBruijnPentagrid)),
        Box::new(Tinted(DecagonFractal)),
        Box::new(Tinted(QuasicrystalDiffraction)),
        Box::new(Tinted(PinwheelTiling)),
        Box::new(Tinted(TruchetQuasi)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// A triangle tagged with its kind: `(kind, p, q, r)`, vertices in unit space.
type MarkedTri = (bool, [f32; 2], [f32; 2], [f32; 2]);

/// Grid center in dot-space.
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 * 0.5, dh as f32 * 0.5)
}

/// Bresenham line rasteriser. Out-of-bounds dots are silently discarded.
fn bresenham(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx.abs() + dy.abs() + 2) as usize;
    let mut steps = 0usize;
    loop {
        draw::dot_i(grid, x, y);
        if x == x1 && y == y1 {
            break;
        }
        steps += 1;
        if steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a polygon defined by a slice of (x,y) dot-space points (closed).
fn draw_poly(grid: &mut BrailleGrid, pts: &[(i32, i32)]) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    for i in 0..n {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % n];
        bresenham(grid, x0, y0, x1, y1);
    }
}

/// Map a unit-space point (scale ~1.0) to dot-space with center + scale.
#[inline]
fn to_dot(cx: f32, cy: f32, scale: f32, ux: f32, uy: f32, rot: f32) -> (i32, i32) {
    let rx = ux * rot.cos() - uy * rot.sin();
    let ry = ux * rot.sin() + uy * rot.cos();
    (
        (cx + rx * scale).round() as i32,
        (cy - ry * scale).round() as i32,
    )
}

// ────────────────────────────────────────────────────────────────────────────
// Shared Robinson-triangle tiling (P2 and P3)
// ────────────────────────────────────────────────────────────────────────────
//
// Both Penrose tilings come from the same seed — a decagonal wheel of ten
// 36°-apex Robinson triangles — and differ only in the subdivision rule.
// Each triangle is `(red, a, b, c)` with apex `a`; two triangles of one tile
// share an edge that is never inked, so the outlines show whole rhombs (P3)
// or whole kites and darts (P2).
//
//   P3:  red  (thin-rhomb half):  P = a + (b−a)/φ → red(c,P,b) + blue(P,c,a)
//        blue (fat-rhomb half):   Q = b + (a−b)/φ, R = b + (c−b)/φ
//                                 → blue(R,c,a) + blue(Q,R,b) + red(R,Q,a)
//   P2:  red  (kite half):        Q = a + (b−a)/φ, R = b + (c−b)/φ
//                                 → blue(R,Q,b) + red(Q,a,R) + red(c,a,R)
//        blue (dart half):        P = c + (a−c)/φ → blue(b,P,a) + red(P,c,b)

/// Seed wheel of ten red triangles, every second one mirrored.
fn robinson_wheel() -> Vec<MarkedTri> {
    (0..10)
        .map(|i| {
            let ab = (2.0 * i as f32 - 1.0) * PI / 10.0;
            let ac = (2.0 * i as f32 + 1.0) * PI / 10.0;
            let mut bv = [ab.cos(), ab.sin()];
            let mut cv = [ac.cos(), ac.sin()];
            if i % 2 == 0 {
                std::mem::swap(&mut bv, &mut cv);
            }
            (true, [0.0, 0.0], bv, cv)
        })
        .collect()
}

/// One subdivision step for the rhomb (P3) or kite-and-dart (P2) rule set.
fn robinson_deflate(tris: Vec<MarkedTri>, kites: bool) -> Vec<MarkedTri> {
    let inv = 1.0 / PHI;
    let mut out = Vec::with_capacity(tris.len() * 3);
    for (red, a, b, c) in tris {
        match (kites, red) {
            (false, true) => {
                let p = lerp2(a, b, inv);
                out.push((true, c, p, b));
                out.push((false, p, c, a));
            }
            (false, false) => {
                let q = lerp2(b, a, inv);
                let r = lerp2(b, c, inv);
                out.push((false, r, c, a));
                out.push((false, q, r, b));
                out.push((true, r, q, a));
            }
            (true, true) => {
                let q = lerp2(a, b, inv);
                let r = lerp2(b, c, inv);
                out.push((false, r, q, b));
                out.push((true, q, a, r));
                out.push((true, c, a, r));
            }
            (true, false) => {
                let p = lerp2(c, a, inv);
                out.push((false, b, p, a));
                out.push((true, p, c, b));
            }
        }
    }
    out
}

/// Draw a Penrose tiling across the whole strip: tile outlines (two edges
/// of every triangle, so the shared half-edge stays blank), with the tiles of
/// one kind (`stipple_red`) given a sparse dot texture so the two shapes read
/// apart in monochrome.  Tiles appear left → right up to `ctx.eased`; the
/// wheel turns very slowly on `ctx.time` so the tiling drifts.
fn render_robinson(grid: &mut BrailleGrid, ctx: &BarContext, kites: bool, stipple_red: bool) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw == 0 || dh == 0 {
        return;
    }
    let (cx, cy) = center(dw, dh);
    // Wheel radius covers the strip's half-diagonal; four subdivisions (three
    // for the larger kites) leave tile edges ~7-11 dots long on an 88-dot strip.
    let scale = (cx * cx + cy * cy).sqrt() + 4.0;
    let mut tris = robinson_wheel();
    for _ in 0..if kites { 3 } else { 4 } {
        tris = robinson_deflate(tris, kites);
    }
    let rot = ctx.time * 0.04;
    let front = ctx.eased.clamp(0.0, 1.0) * (dw as f32 + 6.0) - 3.0;
    let (sr, cr) = rot.sin_cos();
    let to_f = |v: [f32; 2]| -> (f32, f32) {
        (
            cx + (v[0] * cr - v[1] * sr) * scale,
            cy - (v[0] * sr + v[1] * cr) * scale,
        )
    };
    for &(red, a, b, c) in &tris {
        let (pa, pb, pc) = (to_f(a), to_f(b), to_f(c));
        let mid_x = (pa.0 + pb.0 + pc.0) / 3.0;
        if mid_x > front {
            continue;
        }
        let ri = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
        let (ia, ib, ic) = (ri(pa), ri(pb), ri(pc));
        // The two halves of a tile meet along b–c, the edge left un-inked.
        bresenham(grid, ic.0, ic.1, ia.0, ia.1);
        bresenham(grid, ia.0, ia.1, ib.0, ib.1);
        if red == stipple_red {
            // Sparse texture inside the triangle (dots on a 2×2 lattice).
            let x0 = pa.0.min(pb.0).min(pc.0).floor().max(0.0) as i32;
            let x1 = pa.0.max(pb.0).max(pc.0).ceil().min(dw as f32) as i32;
            let y0 = pa.1.min(pb.1).min(pc.1).floor().max(0.0) as i32;
            let y1 = pa.1.max(pb.1).max(pc.1).ceil().min(dh as f32) as i32;
            let edge = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
                (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0)
            };
            for y in (y0..y1).filter(|y| y % 2 == 1) {
                for x in (x0..x1).filter(|x| x % 2 == 1) {
                    let (fx, fy) = (x as f32, y as f32);
                    let d1 = edge(pa, pb, fx, fy);
                    let d2 = edge(pb, pc, fx, fy);
                    let d3 = edge(pc, pa, fx, fy);
                    let inside =
                        (d1 > 0.5 && d2 > 0.5 && d3 > 0.5) || (d1 < -0.5 && d2 < -0.5 && d3 < -0.5);
                    if inside {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
        }
    }
}

#[inline]
fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Penrose P3 — rhombus tiling
// ────────────────────────────────────────────────────────────────────────────

struct PenroseP3;
impl ProgressStyle for PenroseP3 {
    fn name(&self) -> &str {
        "penrose-p3"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose P3 rhombus tiling built by Robinson-triangle subdivision: fat \
         (72°) rhombs in outline, thin (36°) rhombs stippled, laid left to right \
         across the strip as the tiling slowly drifts"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        render_robinson(grid, ctx, false, true);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Penrose P2 — kite & dart tiling
// ────────────────────────────────────────────────────────────────────────────

struct PenroseP2;
impl ProgressStyle for PenroseP2 {
    fn name(&self) -> &str {
        "penrose-p2"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose P2 kite & dart tiling built by Robinson-triangle subdivision: \
         kites in outline, darts stippled, laid left to right across the strip \
         as the tiling slowly drifts"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        render_robinson(grid, ctx, true, false);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Sun Pattern — 5 kites around a center, expanding rings
// ────────────────────────────────────────────────────────────────────────────
//
// The canonical Penrose "sun" seed: five kites sharing a vertex at the origin.
// Progress reveals rings of tiles growing outward from the center.

struct SunPattern;
impl ProgressStyle for SunPattern {
    fn name(&self) -> &str {
        "sun-pattern"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Penrose sun: five kites (72-72-72-144°) meeting tip-to-tip at one vertex, \
         laid kite by kite and slowly turning between two progress paths"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        let scale = (dh as f32 / 2.0 - 0.5).max(1.0);
        let e = ctx.eased.clamp(0.0, 1.0);
        let rot = ctx.time * 0.2 + PI / 2.0;

        // Progress path either side of the sun: solid when done, dotted ahead.
        let lit = (e * dw as f32) as i32;
        let half = scale.ceil() as i32 + 2;
        let phase = (ctx.time * 6.0) as i32;
        for x in 0..dw as i32 {
            if (x - cx as i32).abs() <= half {
                continue;
            }
            if x < lit || (x + phase).rem_euclid(3) == 0 {
                draw::dot_i(grid, x, cy.round() as i32);
            }
        }

        // Kite k: tip at the centre, axis at 72k°, wings at ±36° — every
        // outer vertex sits on the unit circle, so the five close into a
        // decagon split by five spokes (the shared wing edges).
        let kites = ((0.2 + e * 1.6).min(1.0) * 5.0).ceil() as usize;
        for k in 0..kites.min(5) {
            let th = k as f32 * 2.0 * PI / 5.0;
            let pts = [
                to_dot(cx, cy, scale, 0.0, 0.0, rot),
                to_dot(
                    cx,
                    cy,
                    scale,
                    (th - PI / 5.0).cos(),
                    (th - PI / 5.0).sin(),
                    rot,
                ),
                to_dot(cx, cy, scale, th.cos(), th.sin(), rot),
                to_dot(
                    cx,
                    cy,
                    scale,
                    (th + PI / 5.0).cos(),
                    (th + PI / 5.0).sin(),
                    rot,
                ),
            ];
            draw_poly(grid, &pts);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Girih Tiles — Islamic decagon strapwork
// ────────────────────────────────────────────────────────────────────────────
//
// Girih tiles are five polygons (decagon, pentagon, hexagon, bowtie, rhombus)
// whose edges all have the same length and whose angles are multiples of 36°.
// We draw a central decagon, then surround it with pentagons and bowties,
// revealing the strapwork interior lines as progress rises.

struct GirihTiles;
impl ProgressStyle for GirihTiles {
    fn name(&self) -> &str {
        "girih-tiles"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Islamic girih frieze: a row of edge-sharing decagon tiles, each carrying the \
         ten-pointed strapwork star whose points touch every tile edge at its midpoint, \
         meeting the neighbouring star there; tiles are laid left to right with \
         progress and a spark runs round each finished star"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Decagon with vertical left/right edges: vertices at 18° + 36°k.
        let r = (dh as f32 * 0.5 - 0.5).max(1.0);
        let pitch = (2.0 * r * (PI / 10.0).cos()).max(2.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        let n_tiles = ((dw as f32 / pitch).ceil() as usize).max(1);
        let total = 0.35 + ctx.eased.clamp(0.0, 1.0) * (n_tiles as f32 - 0.35);
        let x0 = pitch * 0.5;
        for k in 0..n_tiles {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                break;
            }
            let cx = x0 + k as f32 * pitch;
            let off = PI / 10.0;
            // Tile outline (drawn as a dotted line once the straps are in).
            let verts: Vec<(f32, f32)> = (0..10)
                .map(|i| {
                    let t = off + i as f32 * PI / 5.0;
                    (cx + r * t.cos(), cy - r * t.sin())
                })
                .collect();
            let outline = (local * 2.0).min(1.0);
            let n_edges = (outline * 10.0).ceil() as usize;
            for i in 0..n_edges.min(10) {
                let (p, q) = (verts[i], verts[(i + 1) % 10]);
                let steps = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as usize).max(1);
                for s in 0..=steps {
                    if s % 3 == 0 {
                        let g = s as f32 / steps as f32;
                        draw::dot_i(
                            grid,
                            (p.0 + (q.0 - p.0) * g).round() as i32,
                            (p.1 + (q.1 - p.1) * g).round() as i32,
                        );
                    }
                }
            }
            // Strapwork: the ten-pointed star whose points sit on the edge
            // midpoints (where the straps cross into the neighbouring tile).
            let rm = r * (PI / 10.0).cos();
            let star: Vec<(f32, f32)> = (0..20)
                .map(|i| {
                    let t = off + PI / 10.0 + i as f32 * PI / 10.0;
                    let rr = if i % 2 == 0 { rm } else { rm * 0.5 };
                    (cx + rr * t.cos(), cy - rr * t.sin())
                })
                .collect();
            let straps = ((local - 0.3) / 0.7).clamp(0.0, 1.0) * 20.0;
            let full = straps.floor() as usize;
            for i in 0..20usize {
                let (p, q) = (star[i], star[(i + 1) % 20]);
                let g = match i.cmp(&full) {
                    std::cmp::Ordering::Less => 1.0,
                    std::cmp::Ordering::Equal => straps - full as f32,
                    std::cmp::Ordering::Greater => break,
                };
                bresenham(
                    grid,
                    p.0.round() as i32,
                    p.1.round() as i32,
                    (p.0 + (q.0 - p.0) * g).round() as i32,
                    (p.1 + (q.1 - p.1) * g).round() as i32,
                );
            }
            // A spark runs round the strap star once the tile is complete.
            if local >= 1.0 {
                let u = (ctx.time * 0.4 + k as f32 * 0.31).fract() * 20.0;
                let i = (u.floor() as usize).min(19);
                let g = u - i as f32;
                let (p, q) = (star[i], star[(i + 1) % 20]);
                let (x, y) = (
                    (p.0 + (q.0 - p.0) * g).round() as i32,
                    (p.1 + (q.1 - p.1) * g).round() as i32,
                );
                for (ddx, ddy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                    draw::dot_i(grid, x + ddx, y + ddy);
                }
            }
        }
        Ok(())
    }
}

/// Compute vertices of a regular n-gon centered at (cx,cy) in dot-space.
fn regular_ngon(n: usize, cx: f32, cy: f32, radius: f32, offset: f32) -> Vec<(i32, i32)> {
    (0..n)
        .map(|k| {
            let a = 2.0 * PI * k as f32 / n as f32 + offset;
            (
                (cx + radius * a.cos()).round() as i32,
                (cy - radius * a.sin()).round() as i32,
            )
        })
        .collect()
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Ammann Bars — the hidden quasiperiodic grid on a P3 rhombus tiling
// ────────────────────────────────────────────────────────────────────────────
//
// Every Penrose P3 tiling carries an "Ammann bar" decoration: each rhombus
// gets one stripe across it, and the stripes form 5 families of parallel lines
// at angles 0°, 72°, 144°, 216°, 288°.  The spacings within each family are
// quasiperiodic (long L and short S with L/S = PHI).
//
// We approximate this by drawing 5 families of parallel quasiperiodic lines.
// Each family k has angle k*36° and lines at positions …, 0, L, L+S, 2L+S, 2L+2S, …
// where L = PHI and S = 1.

struct AmmannBars;
impl ProgressStyle for AmmannBars {
    fn name(&self) -> &str {
        "ammann-bars"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Ammann bars: the five families of parallel bars that run through every \
         Penrose rhomb tiling, at 72° to one another, each spaced in the \
         never-repeating Fibonacci order of long and short gaps (L/S = φ); a faint \
         rhomb tiling lies beneath, and the families sweep in one after another \
         with progress, sliding slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        // Faint rhomb tiling: vertices of the P3 tiling only.
        let scale = (cx * cx + cy * cy).sqrt() + 4.0;
        let mut tris = robinson_wheel();
        for _ in 0..4 {
            tris = robinson_deflate(tris, false);
        }
        for &(_red, a, _b, _c) in &tris {
            draw::dot_i(
                grid,
                (cx + a[0] * scale).round() as i32,
                (cy - a[1] * scale).round() as i32,
            );
        }
        // Fibonacci word of long/short gaps.
        let s_gap = 6.0f32;
        let l_gap = s_gap * PHI;
        let half = (dw as f32 + dh as f32) * 0.5 + l_gap;
        let fams = 0.3 + ctx.eased.clamp(0.0, 1.0) * 4.7;
        for fam in 0..5usize {
            let reveal = (fams - fam as f32).clamp(0.0, 1.0);
            if reveal <= 0.0 {
                break;
            }
            let th = PI / 2.0 + fam as f32 * 2.0 * PI / 5.0;
            let (nx, ny) = (th.cos(), -th.sin()); // normal (screen space)
            let (lx, ly) = (-ny, nx); // bar direction
            let x_lim = reveal * dw as f32;
            // Each family slides slowly along its normal; the gap sequence is
            // anchored to the first bar so the Fibonacci word travels intact.
            let slide = (ctx.time * 1.5 + fam as f32 * 1.3) % (l_gap + s_gap);
            let mut pos = -half + slide;
            for n in 1..=200usize {
                if pos >= half {
                    break;
                }
                let base = (cx + nx * pos, cy + ny * pos);
                let reach = half * 1.5;
                let steps = (reach * 2.0) as usize;
                for i in 0..=steps {
                    let t = -reach + i as f32;
                    let x = base.0 + lx * t;
                    let y = base.1 + ly * t;
                    if x >= 0.0 && x <= x_lim && y >= 0.0 && y < dh as f32 {
                        draw::dot_i(grid, x.round() as i32, y.round() as i32);
                    }
                }
                // n-th letter of the Fibonacci word: L when ⌊(n+1)/φ⌋ − ⌊n/φ⌋ = 1.
                let is_l = ((n + 1) as f32 / PHI).floor() - (n as f32 / PHI).floor() >= 1.0;
                pos += if is_l { l_gap } else { s_gap };
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. De Bruijn Pentagrid — 5 families of parallel lines → dual Penrose tiling
// ────────────────────────────────────────────────────────────────────────────
//
// de Bruijn (1981): a Penrose P3 tiling is dual to a pentagrid — 5 families of
// equidistant parallel lines at angles 72°·k, each with offset γ_k ≈ arbitrary.
//
// We draw the five families, then animate the intersection points to reveal
// the dual rhombus vertices.  The dual step is: each intersection of family j
// and family k at crossing index (m,n) maps to a rhombus vertex.
//
// For progress we simply reveal the five line families one at a time, and
// overlay the computed dual vertices with `ctx.eased`.

struct DeBruijnPentagrid;
impl ProgressStyle for DeBruijnPentagrid {
    fn name(&self) -> &str {
        "debruijn-pentagrid"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "de Bruijn pentagrid: five families of evenly spaced parallel lines at 72° to \
         one another, each with its own offset, whose dual is a Penrose tiling; the \
         families are laid one after another across the strip with progress and the \
         grid drifts slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let spacing = 9.0f32;
        // Shifts γ_k (sum 0, generic) as in de Bruijn's construction.
        let gammas: [f32; 5] = [0.2, -0.35, 0.1, 0.4, -0.35];
        // Families reveal one after another; the first one is always partly in.
        let fams = 0.25 + ctx.eased.clamp(0.0, 1.0) * 4.75;
        let drift = ctx.time * 0.25;
        let cx = dw as f32 * 0.5;
        let cy = dh as f32 * 0.5;
        for (fam, &gamma) in gammas.iter().enumerate() {
            let reveal = (fams - fam as f32).clamp(0.0, 1.0);
            if reveal <= 0.0 {
                break;
            }
            // Family k: lines x·cos θ + y·sin θ = (n + γ_k)·spacing, θ = 90° + 72°k.
            let th = PI / 2.0 + fam as f32 * 2.0 * PI / 5.0;
            let (c, sn) = (th.cos(), th.sin());
            let x_lim = reveal * dw as f32;
            for y in 0..dh {
                for x in 0..dw {
                    if x as f32 > x_lim {
                        break;
                    }
                    let d = ((x as f32 - cx) * c + (y as f32 - cy) * sn) / spacing - gamma - drift;
                    let f = d - d.round();
                    // ~half-dot line width in dot units, scaled by the slope.
                    if f.abs() * spacing < 0.5 {
                        draw::dot(grid, x, y);
                    }
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Decagon Fractal — recursive decagon infill
// ────────────────────────────────────────────────────────────────────────────
//
// A decagon subdivides into 10 smaller decagons + connecting pentagons (roughly).
// We approximate with: one large decagon, then ring of 10 smaller at radius
// 1+sin(18°) * sub_r, then sub-sub-decagons, up to depth 3.
// Depth capped at 3 for cost safety.

struct DecagonFractal;
impl ProgressStyle for DecagonFractal {
    fn name(&self) -> &str {
        "decagon-fractal"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Decagon fractal: a frieze of self-similar decagons — each holds a ring of ten \
         satellites and a 1/φ²-scaled copy of itself at its heart, which repeats the \
         rule; flakes unfold left to right with progress, rings counter-turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let panels = (dw / 18).max(1);
        let pw = dw as f32 / panels as f32;
        let big = (dh as f32 * 0.5 - 0.5).min(pw * 0.5 - 0.5).max(1.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        let total = 0.3 + ctx.eased.clamp(0.0, 1.0) * (panels as f32 - 0.3);
        let shrink = 1.0 / (1.0 + PHI); // 1/φ²
        for k in 0..panels {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                break;
            }
            let cx = pw * (k as f32 + 0.5) - 0.5;
            let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
            let rot = PI / 2.0;
            // Self-similar rule: a decagon = outline + ring of ten satellites
            // near its rim + a 1/φ²-scaled copy at its heart, and the copy
            // obeys the same rule.
            let spin = rot + dir * ctx.time * 0.5;
            let mut level_r = big;
            let mut budget = local * 2.0;
            for level in 0..2 {
                if budget <= 0.0 || level_r < 1.0 {
                    break;
                }
                let part = budget.min(1.0);
                budget -= 1.0;
                draw_poly(grid, &regular_ngon(10, cx, cy, level_r, rot));
                let child = level_r * 0.16;
                let dist = level_r * 0.72;
                let n = (part * 10.0).ceil() as usize;
                let lspin = if level % 2 == 0 {
                    spin
                } else {
                    2.0 * rot - spin
                };
                for j in 0..n.min(10) {
                    let t = lspin + j as f32 * PI / 5.0;
                    let (px, py) = (cx + dist * t.cos(), cy - dist * t.sin());
                    if child >= 1.2 {
                        draw_poly(grid, &regular_ngon(10, px, py, child, rot));
                    } else {
                        draw::dot_i(grid, px.round() as i32, py.round() as i32);
                    }
                }
                level_r *= shrink;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Quasicrystal Diffraction — 5 plane waves at 72° → 10-fold interference
// ────────────────────────────────────────────────────────────────────────────
//
// The quasicrystal interference pattern is computed as:
//   f(x,y) = Σ_{k=0}^{4} cos(2π · (x·cos(k·72°) + y·sin(k·72°)) · freq)
// Normalise to [0,1] and threshold at eased to reveal the bright regions.
// Braille dot space is sampled directly — very fast per dot.

struct QuasicrystalDiffraction;
impl ProgressStyle for QuasicrystalDiffraction {
    fn name(&self) -> &str {
        "quasicrystal-diffraction"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Quasicrystal diffraction: a row of 10-fold Bragg-spot patterns (bright centre, \
         two rings of ten spots at radii in φ ratio, the outer ring turned 18°) — the \
         forbidden symmetry that won Shechtman the Nobel prize; patterns expose \
         left to right with progress and their spots twinkle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let panels = (dw / 18).max(1);
        let pw = dw as f32 / panels as f32;
        let r_max = (dh as f32 * 0.5 - 0.5).min(pw * 0.5 - 0.5).max(1.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        // Ring radii in φ ratio, outermost touching the strip edge.
        let phi = 1.618_034_f32;
        let radii = [r_max / phi, r_max];
        let total = 0.3 + ctx.eased.clamp(0.0, 1.0) * (panels as f32 - 0.3);
        let blob = |grid: &mut BrailleGrid, x: f32, y: f32, big: bool| {
            let (xi, yi) = (x.round() as i32, y.round() as i32);
            draw::dot_i(grid, xi, yi);
            if big {
                for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    draw::dot_i(grid, xi + ddx, yi + ddy);
                }
            }
        };
        for k in 0..panels {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                break;
            }
            let cx = pw * (k as f32 + 0.5) - 0.5;
            // Central (000) beam — always strong.
            blob(grid, cx, cy, true);
            for (ri, &rr) in radii.iter().enumerate() {
                let need = ri as f32 * 0.5;
                if local <= need {
                    break;
                }
                let frac = ((local - need) * 2.0).min(1.0);
                let n_spots = (frac * 10.0).ceil() as usize;
                let off = if ri % 2 == 0 {
                    PI / 2.0
                } else {
                    PI / 2.0 + PI / 10.0
                };
                for j in 0..n_spots.min(10) {
                    let t = off + j as f32 * PI / 5.0;
                    let x = cx + rr * t.cos();
                    let y = cy - rr * t.sin();
                    // Inner rings are brighter; spots twinkle on time.
                    let tw = (ctx.time * 3.0 + j as f32 * 1.7 + ri as f32 * 2.3 + k as f32).sin();
                    let big = ri == 1 && tw > -0.5;
                    blob(grid, x, y, big);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Pinwheel Tiling — 1:2 right-triangle substitution
// ────────────────────────────────────────────────────────────────────────────
//
// The pinwheel tiling (Conway & Radin 1994) is based on a right triangle
// with legs 1 and 2 and hypotenuse √5.  One triangle substitutes into 5,
// and with each generation the triangles appear at every possible rotation
// (dense in SO(2)), making it rotationally aperiodic.
//
// Deflation: a right triangle (legs a=1, b=2) splits into 5 smaller copies.
// Vertices: P (right angle), Q (on short side), R (far end of long side).
// We cap at depth 4 and cost 2000 triangles.

struct PinwheelTiling;
impl ProgressStyle for PinwheelTiling {
    fn name(&self) -> &str {
        "pinwheel-tiling"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Pinwheel tiling: a 1:2 right triangle substitutes into 5 smaller copies \
         at irrational rotations, filling the plane with triangles at every angle; \
         deflation generations bloom outward as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // The whole canvas is tiled with 1:2 right triangles: a lattice of
        // 2h×h rectangles, each split along its diagonal into two seeds.
        // As progress rises every seed deflates into 5, then 25 copies —
        // exactly the pinwheel substitution — sweeping left to right.
        type Tri = [[f32; 2]; 3]; // [P (right angle), Q (short leg), R (long leg)]
        let rh = (dh as f32 - 1.0).max(2.0); // rectangle height (short leg)
        let rw = rh * 2.0; // rectangle width (long leg)
        let cols = ((dw as f32 / rw).ceil() as usize + 1).min(64);
        let period = cols as f32 * rw;

        // The tiling drifts one full lattice period per 4 s loop (seamless).
        let drift = (ctx.time * 0.25).fract() * period;

        let mut seeds: Vec<Tri> = Vec::with_capacity(cols * 2);
        for col in 0..cols {
            let x0 = col as f32 * rw;
            // Two complementary 1:2 triangles sharing the diagonal.
            seeds.push([[x0, 0.0], [x0, rh], [x0 + rw, 0.0]]);
            seeds.push([[x0 + rw, rh], [x0 + rw, 0.0], [x0, rh]]);
        }

        // Draw a triangle at the drifting offset (twice, so the wrap-around
        // seam at the right edge is filled by the copy from the left).
        let draw_tri = |grid: &mut BrailleGrid, tri: &Tri| {
            for wrap in [0.0f32, -period] {
                let pts: Vec<(i32, i32)> = tri
                    .iter()
                    .map(|v| ((v[0] + drift + wrap).round() as i32, (v[1]).round() as i32))
                    .collect();
                if pts.iter().all(|&(x, _)| x < 0) || pts.iter().all(|&(x, _)| x >= dw as i32) {
                    continue;
                }
                draw_poly(grid, &pts);
            }
        };

        // The seed lattice is always visible; deflation generations bloom
        // seed by seed as eased advances (two full passes: depth 1, then 2).
        for tri in &seeds {
            draw_tri(grid, tri);
        }
        let n_seeds = seeds.len();
        let phase = ctx.eased * 2.0;
        for (i, seed) in seeds.iter().enumerate() {
            let gen1_on = phase * n_seeds as f32 >= (i + 1) as f32;
            let gen2_on = (phase - 1.0) * n_seeds as f32 >= (i + 1) as f32;
            if !gen1_on {
                continue;
            }
            let children = pinwheel_deflate(vec![*seed]);
            for tri in &children {
                draw_tri(grid, tri);
            }
            if gen2_on {
                for tri in &pinwheel_deflate(children) {
                    draw_tri(grid, tri);
                }
            }
        }
        Ok(())
    }
}

type Tri2 = [[f32; 2]; 3];

fn pinwheel_deflate(tris: Vec<Tri2>) -> Vec<Tri2> {
    let mut out = Vec::with_capacity(tris.len() * 5);
    for tri in tris {
        let [p, q, r] = tri;
        // Pinwheel substitution: 5 children.
        // Standard pinwheel: right angle at P, short leg PQ (len 1), long leg PR (len 2).
        // Sub-triangle hypotenuse = 1/√5 of parent hypotenuse.
        //
        // Key points (per Conway-Radin):
        //   M  = midpoint(P, R)           (midpoint of long leg)
        //   A  = P + (1/5)*(Q-P)          (1/5 along PQ from P)
        //   B  = P + (2/5)*(Q-P)          (2/5 along PQ from P)
        //   C  = midpoint(Q, R)
        //   D  = M + (1/2)*(Q - M)        ~ midpoint(M, Q)
        //
        // 5 children (approximate but structurally correct):
        let m = lerp2(p, r, 0.5);
        let a = lerp2(p, q, 0.2);
        let b = lerp2(p, q, 0.4);
        let c = lerp2(q, r, 0.5);
        let d = lerp2(m, q, 0.5);
        out.push([p, a, m]);
        out.push([a, b, d]);
        out.push([b, q, c]);
        out.push([d, c, m]);
        out.push([m, c, r]);
    }
    out
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Truchet-Quasi — Truchet arcs on a quasiperiodic rhombus lattice
// ────────────────────────────────────────────────────────────────────────────
//
// Classic Truchet tiles place quarter-circle arcs in squares, two orientations.
// We place arcs inside the rhombi of a Penrose-like rhombus grid (fat and thin),
// with orientation chosen by a deterministic pseudo-random rule seeded from
// the rhombus index so the pattern never repeats.
//
// Since we don't have a full deflation here, we generate a flat grid of
// rhombi using a pentagrid projection approach (approximate) and draw arcs
// inside each.

struct TruchetQuasi;
impl ProgressStyle for TruchetQuasi {
    fn name(&self) -> &str {
        "truchet-quasi"
    }
    fn theme(&self) -> &str {
        "penrose"
    }
    fn describe(&self) -> &str {
        "Truchet-quasi: Truchet-style arcs on a Penrose rhomb tiling — every tile \
         carries an arc round its corner, and the arcs join across tile edges into \
         unbroken curves that never repeat; tiles are laid left to right with \
         progress, edges faint, and the tiling drifts slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let scale = (cx * cx + cy * cy).sqrt() + 4.0;
        let mut tris = robinson_wheel();
        for _ in 0..4 {
            tris = robinson_deflate(tris, false);
        }
        let rot = ctx.time * 0.04;
        let front = 6.0 + ctx.eased.clamp(0.0, 1.0) * (dw as f32 + 2.0) - 3.0;
        let (sr, cr) = rot.sin_cos();
        let to_f = |v: [f32; 2]| -> (f32, f32) {
            (
                cx + (v[0] * cr - v[1] * sr) * scale,
                cy - (v[0] * sr + v[1] * cr) * scale,
            )
        };
        for &(_red, a, b, c) in &tris {
            let (pa, pb, pc) = (to_f(a), to_f(b), to_f(c));
            let mid_x = (pa.0 + pb.0 + pc.0) / 3.0;
            if mid_x > front || mid_x < -8.0 || mid_x > dw as f32 + 8.0 {
                continue;
            }
            // Faint tile edges: every third dot of the two outer edges.
            for (p, q) in [(pa, pb), (pa, pc)] {
                let n = ((q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil() as usize).max(1);
                for i in (0..=n).step_by(3) {
                    let g = i as f32 / n as f32;
                    draw::dot_i(
                        grid,
                        (p.0 + (q.0 - p.0) * g).round() as i32,
                        (p.1 + (q.1 - p.1) * g).round() as i32,
                    );
                }
            }
            // Arc round the apex `a`, radius half the edge, from edge a–b to
            // edge a–c; the two halves of each rhomb complete one arc.
            let rad = 0.5 * ((pb.0 - pa.0).powi(2) + (pb.1 - pa.1).powi(2)).sqrt();
            let t0 = (pb.1 - pa.1).atan2(pb.0 - pa.0);
            let t1 = (pc.1 - pa.1).atan2(pc.0 - pa.0);
            let mut sweep = t1 - t0;
            // atan2 differences lie in (−2π, 2π): one wrap picks the short arc.
            if sweep > PI {
                sweep -= 2.0 * PI;
            } else if sweep < -PI {
                sweep += 2.0 * PI;
            }
            let steps = ((rad * sweep.abs() * 1.5).ceil() as usize).max(2);
            for i in 0..=steps {
                let t = t0 + sweep * i as f32 / steps as f32;
                draw::dot_i(
                    grid,
                    (pa.0 + rad * t.cos()).round() as i32,
                    (pa.1 + rad * t.sin()).round() as i32,
                );
            }
        }
        Ok(())
    }
}
