//! Flower-of-Life sacred geometry progress styles.
//!
//! Eleven structurally distinct styles drawn from the Flower-of-Life family:
//! overlapping-circle constructions on a hexagonal lattice, their intersection
//! graphs (Metatron's Cube), the Kabbalistic Tree of Life, a 64-tetrahedron
//! triangular projection, and a time-rotating torus-flower. Figures fill the
//! strip's height, most repeated as a frieze filled panel by panel. Every style
//! reveals itself as `ctx.eased` rises from 0 → 1 and breathes or rotates
//! via `ctx.time`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ── shared geometry helpers ────────────────────────────────────────────────

/// Plot a circle (parametric) in dot-space.
/// `cx,cy` are the centre in dots (f32); `r` is the radius in dots.
/// Steps are bounded to avoid overdraw on tiny grids.
fn plot_circle(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    if r < 0.5 {
        draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
        return;
    }
    // Circumference ≈ 2πr; sample at ≥2 dots per step so we don't miss any dot.
    let steps = ((2.0 * PI * r).ceil() as usize * 2).clamp(8, 2048);
    for i in 0..steps {
        let angle = 2.0 * PI * i as f32 / steps as f32;
        let px = cx + r * angle.cos();
        let py = cy + r * angle.sin();
        draw::dot_i(grid, px.round() as i32, py.round() as i32);
    }
}

/// Draw a line between two dot-space points using Bresenham-style stepping.
fn plot_line(grid: &mut BrailleGrid, x0: f32, y0: f32, x1: f32, y1: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()).ceil() as usize).clamp(1, 4096);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let px = x0 + dx * t;
        let py = y0 + dy * t;
        draw::dot_i(grid, px.round() as i32, py.round() as i32);
    }
}

/// Hex-lattice unit vectors (6 directions, 60° apart).
fn hex_offset(ring_angle_index: usize, radius: f32) -> (f32, f32) {
    let a = ring_angle_index as f32 * PI / 3.0;
    (radius * a.cos(), radius * a.sin())
}

/// 13 Fruit-of-Life centres: centre + 6 at r + 6 at 2r (same hex axes).
fn fruit_centres(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    let mut v: Vec<(f32, f32)> = Vec::with_capacity(13);
    v.push((cx, cy));
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, r);
        v.push((cx + dx, cy + dy));
    }
    for i in 0..6 {
        let (dx, dy) = hex_offset(i, 2.0 * r);
        v.push((cx + dx, cy + dy));
    }
    v
}

/// Lays a frieze of full-height panels across the strip, filled left to right
/// with progress (the first already part-drawn at the start), each tinted
/// with its own palette stop. The callback gets `(grid, cx, cy, r,
/// local_reveal, index)`; `local_reveal` is 0 for a panel not yet reached.
/// `wf` is the panel pitch in radii, `gap` extra dots between panels.
fn frieze(
    grid: &mut BrailleGrid,
    ctx: &BarContext,
    wf: f32,
    gap: f32,
    mut panel: impl FnMut(&mut BrailleGrid, f32, f32, f32, f32, usize),
) {
    let (dw, dh) = draw::dot_dims(grid);
    if dw < 2 || dh < 2 {
        return;
    }
    let r = (dw.min(dh) as f32 / 2.0 - 0.5).max(1.0);
    let pitch = (r * wf + gap).max(2.0);
    let n = ((dw as f32 / pitch) as usize).clamp(1, 64);
    let off = (dw as f32 - n as f32 * pitch) / 2.0;
    let cy = dh as f32 / 2.0 - 0.5;
    let fill = (0.08 + ctx.eased.clamp(0.0, 1.0) * 0.92) * n as f32;
    let (cw, ch) = grid.dimensions();
    for i in 0..n {
        let cx = off + pitch * (i as f32 + 0.5) - 0.5;
        let lr = (fill - i as f32).clamp(0.0, 1.0);
        panel(grid, cx, cy, r, lr, i);
        let color = ctx.palette.sample(i as f32 / (n - 1).max(1) as f32);
        let x0 = ((cx - pitch / 2.0).max(0.0) / 2.0) as usize;
        let x1 = (((cx + pitch / 2.0).max(0.0) / 2.0) as usize).min(cw.saturating_sub(1));
        if cw > 0 && x0 <= x1 {
            for y in 0..ch {
                draw::tint_row(grid, y, x0, x1, color);
            }
        }
    }
}

/// Faint dotted ring: the placeholder for a frieze panel not yet reached.
fn dotted_ring(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let n = ((r * 2.0) as usize).clamp(4, 64);
    for k in 0..n {
        let a = 2.0 * PI * k as f32 / n as f32;
        draw::dot_i(
            grid,
            (cx + r * a.cos()).round() as i32,
            (cy + r * a.sin()).round() as i32,
        );
    }
}

/// Plot only the part of circle (`ox`,`oy`,`r`) lying within distance `lim`
/// of (`cx`,`cy`).
fn clipped_circle(grid: &mut BrailleGrid, ox: f32, oy: f32, r: f32, cx: f32, cy: f32, lim: f32) {
    let steps = ((2.0 * PI * r).ceil() as usize * 2).clamp(8, 512);
    for i in 0..steps {
        let a = 2.0 * PI * i as f32 / steps as f32;
        let (px, py) = (ox + r * a.cos(), oy + r * a.sin());
        if (px - cx).hypot(py - cy) <= lim {
            draw::dot_i(grid, px.round() as i32, py.round() as i32);
        }
    }
}

// ── All styles ─────────────────────────────────────────────────────────────

/// All styles in the `floweroflife` theme.
///
/// Returns eleven structurally distinct sacred-geometry progress styles,
/// each revealing itself as `eased` rises and animating via `time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(VesicaPiscis),
        Box::new(SeedOfLife),
        Box::new(FlowerOfLife),
        Box::new(EggOfLife),
        Box::new(FruitOfLife),
        Box::new(MetatronsCube),
        Box::new(GermOfLife),
        Box::new(TripodOfLife),
        Box::new(TreeOfLife),
        Box::new(TetraGrid),
        Box::new(TorusFlower),
    ]
}

// ── 1. Vesica Piscis ───────────────────────────────────────────────────────

/// Two overlapping circles; the shared lens (vesica) forms as `eased` rises.
struct VesicaPiscis;
impl ProgressStyle for VesicaPiscis {
    fn name(&self) -> &str {
        "vesica-piscis"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Two circles overlap to reveal the sacred lens; arc segments appear with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let r = (cx.min(cy) * 0.45).max(1.0);

        // Separation between the two circle centres, grows with eased.
        // At eased=0: coincident. At eased=1: separated by r (classic vesica).
        let sep = r * ctx.eased;
        let lx = cx - sep / 2.0;
        let rx = cx + sep / 2.0;

        // Draw the two circles, breathing via time.
        let breathe = 1.0 + 0.04 * (ctx.time * 1.2).sin();
        let rr = r * breathe;
        plot_circle(grid, lx, cy, rr);
        if ctx.eased > 0.05 {
            plot_circle(grid, rx, cy, rr);
        }

        // Color: tint the left half one shade, right half another.
        let (cw, ch) = grid.dimensions();
        let mid_cell = cw / 2;
        for cy_cell in 0..ch {
            if mid_cell > 0 {
                draw::tint_row(
                    grid,
                    cy_cell,
                    0,
                    mid_cell.saturating_sub(1),
                    ctx.palette.start,
                );
                draw::tint_row(
                    grid,
                    cy_cell,
                    mid_cell,
                    cw.saturating_sub(1),
                    ctx.palette.end,
                );
            }
        }
        Ok(())
    }
}

// ── 2. Seed of Life ────────────────────────────────────────────────────────

/// Seed of Life: seven equal circles, six around one, each through the centre.
struct SeedOfLife;
impl ProgressStyle for SeedOfLife {
    fn name(&self) -> &str {
        "seed-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Seed of Life: a row of seeds, each seven equal circles — one centre \
         and six around it, every one passing through the centre — set circle \
         by circle and seed by seed as progress rises, slowly turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Outer circles reach 2·rho: the seed fills the strip's height.
            let rho = (r / 2.0).max(0.8);
            let rot = time * 0.25 * if i % 2 == 0 { 1.0 } else { -1.0 };
            let n = (lr * 7.0).ceil() as usize;
            for k in 0..n.min(7) {
                if k == 0 {
                    plot_circle(g, cx, cy, rho);
                } else {
                    let a = rot + (k - 1) as f32 * PI / 3.0;
                    plot_circle(g, cx + rho * a.cos(), cy + rho * a.sin(), rho);
                }
            }
        });
        Ok(())
    }
}

// ── 3. Flower of Life ─────────────────────────────────────────────────────

/// The Flower of Life as its true, unbounded form: equal circles on a
/// hexagonal lattice, each passing through its six neighbours' centres.  At
/// 16 dots tall the 19-circle medallion shrinks to a smudge, so the lattice
/// runs as a frieze across the strip instead — circles bloom left → right
/// with progress (the newest column growing from its centre) while the
/// whole weave drifts slowly on `time`.
struct FlowerOfLife;
impl ProgressStyle for FlowerOfLife {
    fn name(&self) -> &str {
        "flower-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Flower of Life lattice: equal circles on a hexagonal grid, each through \
         its neighbours' centres, blooming left to right as a drifting frieze"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cy = dh as f32 / 2.0 - 0.5;
        // Circle radius = lattice spacing; just over half the height keeps
        // the petals open enough to read.
        let r = (dh as f32 * 0.56).max(1.5);
        let row_h = r * 3f32.sqrt() / 2.0;
        let drift = (ctx.time * 1.5).rem_euclid(r);
        let front = ctx.eased.clamp(0.0, 1.0) * (dw as f32 + r);

        let rows = (cy / row_h).ceil() as i32 + 1;
        let cols = (dw as f32 / r).ceil() as i32 + 2;
        for j in -rows..=rows {
            let py = cy + j as f32 * row_h;
            let shift = if j.rem_euclid(2) == 1 { r / 2.0 } else { 0.0 };
            for i in -1..cols {
                let px = i as f32 * r + shift + drift - r;
                if px > front {
                    continue;
                }
                // Circles just behind the front are still opening.
                let grow = ((front - px) / r).clamp(0.0, 1.0);
                plot_circle(grid, px, py, r * grow);
            }
        }

        // Palette across the bloomed region, the leading cells brightest.
        let (cw, ch) = grid.dimensions();
        let lit = ((front / 2.0).ceil() as usize).min(cw);
        for cx_cell in 0..lit {
            let t = cx_cell as f32 / cw.max(1) as f32;
            let color = if cx_cell + 2 >= lit {
                ctx.palette.end
            } else {
                ctx.palette.sample(t * 0.8)
            };
            for cy_cell in 0..ch {
                draw::tint_row(grid, cy_cell, cx_cell, cx_cell, color);
            }
        }
        Ok(())
    }
}

// ── 4. Egg of Life ────────────────────────────────────────────────────────

/// Egg of Life: seven touching spheres (one centre, six around) swaying in 3-D.
struct EggOfLife;
impl ProgressStyle for EggOfLife {
    fn name(&self) -> &str {
        "egg-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Egg of Life: a row of seed clusters, each seven touching spheres — \
         one centre, six around, highlighted and swaying in 3-D — set sphere \
         by sphere and cluster by cluster as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            let rho = (r / 3.0).max(0.8);
            let d = 2.0 * rho;
            let sway = 0.45 * (time * 1.2 + i as f32).sin();
            let n = (lr * 7.0).ceil() as usize;
            for k in 0..n.min(7) {
                let (ox, oy, z) = if k == 0 {
                    (0.0, 0.0, 0.0)
                } else {
                    let a = (k - 1) as f32 * PI / 3.0 + PI / 6.0;
                    let (x, y) = (d * a.cos(), d * a.sin());
                    (x * sway.cos(), y, x * sway.sin())
                };
                let cr = rho * (1.0 + 0.15 * z / d);
                let (px, py) = (cx + ox, cy + oy);
                plot_circle(g, px, py, cr);
                // Specular glint: each circle reads as a sphere.
                let gx = (px - cr * 0.4).round() as i32;
                let gy = (py - cr * 0.4).round() as i32;
                draw::dot_i(g, gx, gy);
            }
        });
        Ok(())
    }
}

// ── 5. Fruit of Life ──────────────────────────────────────────────────────

/// Fruit of Life: 13 disjoint circles on the hexagonal Flower-of-Life axes.
struct FruitOfLife;
impl ProgressStyle for FruitOfLife {
    fn name(&self) -> &str {
        "fruit-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Fruit of Life: a row of thirteen-circle fruits — a centre, six \
         around and six beyond along the hexagonal axes, never touching — \
         each set ring by ring as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.3, 3.0, |g, cx, cy, r, lr, _| {
            let rho = (r / 4.46).max(0.6);
            let centres = fruit_centres(cx, cy, 2.0 * rho);
            if lr <= 0.0 {
                for &(px, py) in &centres {
                    draw::dot_i(g, px.round() as i32, py.round() as i32);
                }
                return;
            }
            let n = (lr * centres.len() as f32).ceil() as usize;
            let pulse = 1.0 + 0.08 * (time * 2.0).sin();
            for &(px, py) in centres.iter().take(n) {
                plot_circle(g, px, py, rho * pulse);
            }
        });
        Ok(())
    }
}

// ── 6. Metatron's Cube ────────────────────────────────────────────────────

/// Metatron's Cube: the 13 Fruit-of-Life nodes joined by their key lines.
struct MetatronsCube;
impl ProgressStyle for MetatronsCube {
    fn name(&self) -> &str {
        "metatrons-cube"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Metatron's Cube: a row of cubes, each the 13 Fruit-of-Life node \
         circles joined into outer hexagon and great hexagram, drawn \
         family by family and cube by cube as progress rises, slowly turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // At 16 dots the full 78-line web is a solid blot, so the two
        // families that carry the figure are drawn in turn — outer hexagon,
        // then the great hexagram — over the 13 node circles.
        let mut lines: Vec<(usize, usize)> = Vec::with_capacity(12);
        for i in 0..6 {
            lines.push((7 + i, 7 + (i + 1) % 6));
        }
        for i in 0..6 {
            lines.push((7 + i, 7 + (i + 2) % 6));
        }
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Outer ring of nodes sits at 2·s: the cube fills the height.
            let nr = 1.0f32;
            let s = ((r - nr) / 2.0).max(0.8);
            let rot = time * 0.15 + i as f32 * PI / 6.0;
            let nodes: Vec<(f32, f32)> = fruit_centres(cx, cy, s)
                .iter()
                .map(|&(px, py)| {
                    let (ox, oy) = (px - cx, py - cy);
                    (
                        ox * rot.cos() - oy * rot.sin() + cx,
                        ox * rot.sin() + oy * rot.cos() + cy,
                    )
                })
                .collect();
            let n = (lr * 1.25).min(1.0) * lines.len() as f32;
            for &(a, b) in lines.iter().take(n.ceil() as usize) {
                let (ax, ay) = nodes[a];
                let (bx, by) = nodes[b];
                plot_line(g, ax, ay, bx, by);
            }
            for &(px, py) in &nodes {
                plot_circle(g, px, py, nr);
            }
        });
        Ok(())
    }
}

// ── 7. Germ of Life ───────────────────────────────────────────────────────

/// First 7 circles of the Flower of Life, inner petals traced bold.
struct GermOfLife;
impl ProgressStyle for GermOfLife {
    fn name(&self) -> &str {
        "germ-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Germ of Life: a row of seven-circle germs — the six-petal flower cut \
         cut by six circles centred on a ring, a seed at its heart and the \
         circle centres marked — petal by petal as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Six circles centred on a ring, each through the centre: only
            // their arcs inside the ring are traced, cutting the six-petal
            // germ around a small seed circle.
            let rho = r;
            let rot = time * 0.2 * if i % 2 == 0 { 1.0 } else { -1.0 };
            let n = (lr * 6.0).ceil() as usize;
            for k in 0..n.min(6) {
                let a = rot + k as f32 * PI / 3.0;
                let (ox, oy) = (cx + rho * a.cos(), cy + rho * a.sin());
                clipped_circle(g, ox, oy, rho, cx, cy, rho - 0.4);
                draw::dot_i(g, ox.round() as i32, oy.round() as i32);
            }
            if lr >= 1.0 {
                plot_circle(g, cx, cy, (r * 0.2).max(1.0));
            }
        });
        Ok(())
    }
}

// ── 8. Tripod of Life ─────────────────────────────────────────────────────

/// Tripod of Life: three mutually overlapping circles, a 3-arm spoke between.
struct TripodOfLife;
impl ProgressStyle for TripodOfLife {
    fn name(&self) -> &str {
        "tripod-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Tripod of Life: a row of three mutually overlapping circles turning \
         about their shared centre, a three-arm spoke joining their centres — \
         circle by circle and tripod by tripod as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            let rho = r * 0.6;
            let d = r - rho;
            let rot = time * 0.3 + i as f32 * PI / 3.0;
            let n = ((lr * 1.5).min(1.0) * 3.0).ceil() as usize;
            for k in 0..3 {
                let a = rot + k as f32 * 2.0 * PI / 3.0 - PI / 2.0;
                let (ox, oy) = (cx + d * a.cos(), cy + d * a.sin());
                if k < n {
                    plot_circle(g, ox, oy, rho);
                }
                if lr > 0.66 {
                    plot_line(g, cx, cy, ox, oy);
                }
            }
            draw::dot_i(g, cx.round() as i32, cy.round() as i32);
        });
        Ok(())
    }
}

// ── 9. Tree of Life ───────────────────────────────────────────────────────

/// Kabbalistic Tree of Life: 10 sephirot nodes + 22 connecting paths.
/// Paths drawn progressively with eased; nodes always visible.
struct TreeOfLife;
impl ProgressStyle for TreeOfLife {
    fn name(&self) -> &str {
        "tree-of-life"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Kabbalistic Tree of Life laid on its side: 10 sephirot on three pillars, the \
         22 paths drawn crown to kingdom with progress while the lightning flash \
         runs the sephirot in order"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;

        // The tree lies on its side to use the wide strip: its 6-unit trunk
        // runs left (Kether) to right (Malkuth), the pillars stacked ±1 unit.
        let sx = ((dw as f32 - 6.0) / 6.0).max(1.0);
        let sy = (dh as f32 / 2.0 - 2.0).max(1.0);

        // Sephirot positions (normalized, y increasing downward).
        // Classical layout:  1=Kether(top), 10=Malkuth(bottom).
        let nodes: [(f32, f32); 10] = [
            (0.0, -3.0),  // 0 Kether
            (-1.0, -2.0), // 1 Chokmah
            (1.0, -2.0),  // 2 Binah
            (-1.0, -1.0), // 3 Chesed
            (1.0, -1.0),  // 4 Geburah
            (0.0, 0.0),   // 5 Tiphareth
            (-1.0, 1.0),  // 6 Netzach
            (1.0, 1.0),   // 7 Hod
            (0.0, 2.0),   // 8 Yesod
            (0.0, 3.0),   // 9 Malkuth
        ];

        // Convert to dot-space.
        let dot_nodes: Vec<(f32, f32)> = nodes
            .iter()
            .map(|&(x, y)| (cx + y * sx, cy - 0.5 + x * sy))
            .collect();

        // 22 paths of the Tree of Life (traditional Sefer Yetzirah connections).
        let paths: [(usize, usize); 22] = [
            (0, 1),
            (0, 2),
            (0, 5), // Kether connections
            (1, 2),
            (1, 3),
            (1, 5), // Chokmah
            (2, 4),
            (2, 5), // Binah
            (3, 4),
            (3, 5),
            (3, 6), // Chesed
            (4, 5),
            (4, 7), // Geburah
            (5, 6),
            (5, 7),
            (5, 8), // Tiphareth
            (6, 7),
            (6, 8), // Netzach
            (7, 8), // Hod
            (8, 9), // Yesod-Malkuth
            (6, 9),
            (7, 9), // Netzach / Hod to Malkuth
        ];

        let reveal = (ctx.eased * paths.len() as f32).round() as usize;
        for (i, &(a, b)) in paths.iter().enumerate().take(reveal) {
            let (ax, ay) = dot_nodes[a];
            let (bx, by) = dot_nodes[b];
            plot_line(grid, ax, ay, bx, by);
            let t = i as f32 / (paths.len() - 1).max(1) as f32;
            let _ = t;
        }

        // Draw sephirot nodes as small circles.
        let nr = (sy * 0.25).clamp(1.0, 1.5);
        for (i, &(px, py)) in dot_nodes.iter().enumerate() {
            plot_circle(grid, px, py, nr);
            let t = i as f32 / 9.0;
            let color = ctx.palette.sample(t);
            let (cw, ch) = grid.dimensions();
            let cell_x = (px / 2.0).round() as usize;
            let cell_y = (py / 4.0).round() as usize;
            if cell_x < cw && cell_y < ch {
                draw::tint_row(grid, cell_y, cell_x, cell_x, color);
            }
        }

        // The lightning flash: a spark running the sephirot 1 → 10 in order.
        let run = (ctx.time * 1.5).rem_euclid(9.0);
        let k = (run as usize).min(8);
        let f = run - k as f32;
        let (ax, ay) = dot_nodes[k];
        let (bx, by) = dot_nodes[k + 1];
        let (fx, fy) = (ax + (bx - ax) * f, ay + (by - ay) * f);
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            draw::dot_i(grid, fx.round() as i32 + dx, fy.round() as i32 + dy);
        }
        Ok(())
    }
}

// ── 10. 64-Tetrahedron Grid ───────────────────────────────────────────────

/// 64-tetrahedron grid seen down its axis: star tetrahedra in hexagonal tiles.
struct TetraGrid;
impl ProgressStyle for TetraGrid {
    fn name(&self) -> &str {
        "64-tetra-grid"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "64-tetrahedron grid: the lattice seen down its axis as a row of \
         edge-sharing hexagons, each filled with a star tetrahedron (two \
         interlocking triangles), its axes and core — tile by tile with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 3f32.sqrt(), 0.0, |g, cx, cy, r, lr, _| {
            let v: Vec<(f32, f32)> = (0..6)
                .map(|k| {
                    let a = -PI / 2.0 + k as f32 * PI / 3.0;
                    (cx + r * a.cos(), cy + r * a.sin())
                })
                .collect();
            for k in 0..6 {
                let (a, b) = (v[k], v[(k + 1) % 6]);
                plot_line(g, a.0, a.1, b.0, b.1);
            }
            if lr <= 0.0 {
                return;
            }
            // Upward tetrahedron, then axes, then the downward one.
            for k in [0usize, 2, 4] {
                let (a, b) = (v[k], v[(k + 2) % 6]);
                plot_line(g, a.0, a.1, b.0, b.1);
            }
            if lr > 0.33 {
                for k in 0..3 {
                    let (a, b) = (v[k], v[k + 3]);
                    plot_line(g, a.0, a.1, b.0, b.1);
                }
            }
            if lr > 0.66 {
                for k in [1usize, 3, 5] {
                    let (a, b) = (v[k], v[(k + 2) % 6]);
                    plot_line(g, a.0, a.1, b.0, b.1);
                }
            }
            if lr >= 1.0 && (time * 3.0).sin() > -0.6 {
                plot_circle(g, cx, cy, (r * 0.2).max(1.0));
            }
        });
        Ok(())
    }
}

// ── 11. Torus Flower ──────────────────────────────────────────────────────

/// Torus flower: circles rotated about a centre, sweeping out a torus.
struct TorusFlower;
impl ProgressStyle for TorusFlower {
    fn name(&self) -> &str {
        "torus-flower"
    }
    fn theme(&self) -> &str {
        "floweroflife"
    }
    fn describe(&self) -> &str {
        "Torus flower: a row of tori, each eight tube circles rotated about \
         an open centre into a ring, turning with time — \
         circle by circle and torus by torus as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let time = ctx.time;
        frieze(grid, ctx, 2.0, 3.0, |g, cx, cy, r, lr, i| {
            if lr <= 0.0 {
                dotted_ring(g, cx, cy, r);
                return;
            }
            // Tube circles ringed round an open hole: a donut of circles.
            let rho = r * 0.36;
            let d = r - rho;
            let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rot = dir * time * 0.35;
            let n = (lr * 8.0).ceil() as usize;
            for k in 0..n.min(8) {
                let a = rot + k as f32 * PI / 4.0;
                plot_circle(g, cx + d * a.cos(), cy + d * a.sin(), rho);
            }
        });
        Ok(())
    }
}
