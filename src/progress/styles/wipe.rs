//! Screen-wipe / transition masks for full-screen reveals.
//!
//! Each style treats the grid as a transition mask: dots (the "filled" region)
//! represent the revealed or covered area at progress `t`.  At `t=0` the screen
//! is empty; at `t=1` it is fully filled.  A consumer can use the mask to
//! composite two frames; the gallery simply shows the wipe sweeping as progress
//! advances from 0 to 1.
//!
//! Styles in this module:
//!
//! | name              | mask geometry                                        |
//! |-------------------|------------------------------------------------------|
//! | `linear-lr`       | straight fill advancing left → right                |
//! | `linear-tb`       | straight fill advancing top → bottom                |
//! | `barn-door`       | two halves parting from the centre outward           |
//! | `iris`            | round aperture opening from the centre, white rim    |
//! | `iris-diamond`    | expanding diamond (L1 / Manhattan distance)          |
//! | `diagonal`        | 45° diagonal wipe line sweeping across               |
//! | `checkerboard`    | checkerboard cells filling in threshold order        |
//! | `venetian-blinds` | horizontal slats that each grow from their midpoint  |
//! | `dissolve`        | ordered 4×4 Bayer-dither per-pixel threshold fade    |
//! | `clock-wipe`      | pie slice swept clockwise from 12 by a white hand    |
//! | `spiral`          | Archimedean coiled tape unrolling outward, grooved   |
//! | `split`           | top and bottom edges advance inward to the midline   |
//! | `zigzag`          | wipe whose leading edge is a sine-wave sawtooth      |
//! | `pixelate`        | coarse blocks fill with increasing shade density     |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `wipe` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per transition-mask style.  Every style
/// fills the grid with braille dots representing the "revealed" region at the
/// current progress fraction.  They are structurally independent: each uses a
/// fundamentally different mask geometry.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(LinearLR),
        Box::new(LinearTB),
        Box::new(BarnDoor),
        Box::new(Iris),
        Box::new(IrisDiamond),
        Box::new(Diagonal),
        Box::new(Checkerboard),
        Box::new(VenetianBlinds),
        Box::new(Dissolve),
        Box::new(ClockWipe),
        Box::new(Spiral),
        Box::new(Split),
        Box::new(Zigzag),
        Box::new(Pixelate),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Linear left-to-right wipe
// ─────────────────────────────────────────────────────────────────────────────

struct LinearLR;
impl ProgressStyle for LinearLR {
    fn name(&self) -> &str {
        "linear-lr"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Hard-edge curtain sweeping left → right: a clean vertical wipe line \
         advances from the left edge to the right as progress rises 0→1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let filled = (ctx.eased * dw as f32).round() as usize;
        draw::fill_rect(grid, 0, 0, filled.min(dw), dh);
        // Tint the filled region.
        let (cw, ch) = grid.dimensions();
        let filled_cells = (ctx.eased * cw as f32).round() as usize;
        for cx in 0..filled_cells.min(cw) {
            let t = if cw <= 1 {
                0.5
            } else {
                cx as f32 / (cw - 1) as f32
            };
            let color = ctx.palette.sample(t);
            for cy in 0..ch {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Linear top-to-bottom wipe
// ─────────────────────────────────────────────────────────────────────────────

struct LinearTB;
impl ProgressStyle for LinearTB {
    fn name(&self) -> &str {
        "linear-tb"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Horizontal curtain dropping top → bottom: the revealed band grows \
         downward one dot-row at a time as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let filled_rows = (ctx.eased * dh as f32).round() as usize;
        draw::fill_rect(grid, 0, 0, dw, filled_rows.min(dh));
        // Tint per cell-row.
        let (cw, ch) = grid.dimensions();
        let filled_cell_rows = (ctx.eased * ch as f32).round() as usize;
        for cy in 0..filled_cell_rows.min(ch) {
            let t = if ch <= 1 {
                0.5
            } else {
                cy as f32 / (ch - 1) as f32
            };
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Barn-door: two halves part from the centre
// ─────────────────────────────────────────────────────────────────────────────

struct BarnDoor;
impl ProgressStyle for BarnDoor {
    fn name(&self) -> &str {
        "barn-door"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Barn-door wipe: the left half slides left and the right half slides \
         right, parting from the vertical centre seam as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let half = dw / 2;
        // Each door panel grows outward from the centre by `reach` dots.
        let reach = (ctx.eased * half as f32).round() as usize;
        // Left panel: fills from (half - reach) to half.
        let lx = half.saturating_sub(reach);
        draw::fill_rect(grid, lx, 0, half.saturating_sub(lx), dh);
        // Right panel: fills from half to (half + reach).
        let rx_end = (half + reach).min(dw);
        if rx_end > half {
            draw::fill_rect(grid, half, 0, rx_end - half, dh);
        }
        // Tint the two panels with opposite ends of the palette.
        let (cw, ch) = grid.dimensions();
        let half_c = cw / 2;
        let reach_c = (ctx.eased * half_c as f32).round() as usize;
        let lx_c = half_c.saturating_sub(reach_c);
        let rx_end_c = (half_c + reach_c).min(cw);
        for cy in 0..ch {
            let color_l = ctx.palette.sample(0.2);
            let color_r = ctx.palette.sample(0.8);
            if lx_c < half_c {
                draw::tint_row(grid, cy, lx_c, half_c.saturating_sub(1), color_l);
            }
            if rx_end_c > half_c {
                draw::tint_row(grid, cy, half_c, rx_end_c.saturating_sub(1), color_r);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Iris: round aperture opening from the centre, white rim
// ─────────────────────────────────────────────────────────────────────────────

struct Iris;
impl ProgressStyle for Iris {
    fn name(&self) -> &str {
        "iris"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Iris wipe: a round aperture opens from the centre with a bright white \
         rim; it is only gently widened on wide strips so it stays round, and \
         grows by revealed area so progress reads evenly to the corners"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // At most a 2:1 oval — round enough to read as an aperture, wide
        // enough that its curved sides survive on a short strip.
        let sx = (cx / cy.max(0.5)).clamp(1.0, 2.0);
        let dist = |x: f32, y: f32| ((x - cx) / sx).hypot(y - cy);

        // Radius at which the revealed area equals `eased` of the frame.
        let mut all: Vec<f32> = Vec::with_capacity(dw * dh);
        for dy in 0..dh {
            for dx in 0..dw {
                all.push(dist(dx as f32 + 0.5, dy as f32 + 0.5));
            }
        }
        all.sort_by(f32::total_cmp);
        let e = ctx.eased.clamp(0.0, 1.0);
        let r = if all.is_empty() || e <= 0.0 {
            -1.0
        } else {
            all[((e * all.len() as f32) as usize).clamp(1, all.len()) - 1]
        };

        for dy in 0..dh {
            for dx in 0..dw {
                if dist(dx as f32 + 0.5, dy as f32 + 0.5) <= r {
                    draw::dot(grid, dx, dy);
                }
            }
        }

        // Tint cells fully inside the aperture by radius; cells straddling the
        // edge are left untinted so the rim reads as a bright white ring.
        let (cw, ch) = grid.dimensions();
        let r_max = all.last().copied().unwrap_or(1.0).max(1.0);
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let mut inside = 0;
                for oy in 0..4 {
                    for ox in 0..2 {
                        let (x, y) = (cx_idx * 2 + ox, cy_idx * 4 + oy);
                        if dist(x as f32 + 0.5, y as f32 + 0.5) <= r {
                            inside += 1;
                        }
                    }
                }
                if inside == 8 {
                    let d = dist(cx_idx as f32 * 2.0 + 1.0, cy_idx as f32 * 4.0 + 2.0);
                    let color = ctx.palette.sample((d / r_max).clamp(0.0, 1.0));
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Iris-diamond: expanding diamond (L1/Manhattan distance)
// ─────────────────────────────────────────────────────────────────────────────

struct IrisDiamond;
impl ProgressStyle for IrisDiamond {
    fn name(&self) -> &str {
        "iris-diamond"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Diamond iris wipe: a filled rhombus expands from the centre using \
         Manhattan (L1) distance — produces sharp 45° diamond edges instead of \
         the circular iris's smooth curve"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // max Manhattan radius to reach the corner.
        let max_r = cx + cy + 1.0;
        let r = ctx.eased * max_r;
        for dy in 0..dh {
            let vy = (dy as f32 + 0.5 - cy).abs();
            for dx in 0..dw {
                let vx = (dx as f32 + 0.5 - cx).abs();
                if vx + vy <= r {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint with L1-distance gradient.
        let (cw, ch) = grid.dimensions();
        let ccx = cw as f32 / 2.0;
        let ccy = ch as f32 / 2.0;
        let max_rc = (ccx + ccy).max(1.0);
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = (cx_idx as f32 + 0.5 - ccx).abs();
                let vy = (cy_idx as f32 + 0.5 - ccy).abs();
                let dist = vx + vy;
                if dist / max_rc <= ctx.eased {
                    let t = (dist / max_rc).clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Diagonal: 45° diagonal wipe
// ─────────────────────────────────────────────────────────────────────────────

struct Diagonal;
impl ProgressStyle for Diagonal {
    fn name(&self) -> &str {
        "diagonal"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Diagonal wipe: a 45° slanted edge sweeps from the top-left corner to \
         the bottom-right, revealing the screen along the anti-diagonal axis"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // The wipe front is the line: x + y = threshold.
        // At progress=0 threshold=0 (nothing shown); at 1 threshold=dw+dh-2 (all shown).
        let total = (dw + dh) as f32;
        let threshold = ctx.eased * total;
        for dy in 0..dh {
            for dx in 0..dw {
                if (dx as f32 + dy as f32) < threshold {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint along the diagonal axis.
        let (cw, ch) = grid.dimensions();
        let total_c = (cw + ch) as f32;
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let diag = (cx_idx + cy_idx) as f32;
                if diag / total_c <= ctx.eased {
                    let t = (diag / total_c).clamp(0.0, 1.0);
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Checkerboard: cells fill in checkerboard order
// ─────────────────────────────────────────────────────────────────────────────

struct Checkerboard;
impl ProgressStyle for Checkerboard {
    fn name(&self) -> &str {
        "checkerboard"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Checkerboard wipe: the screen is split into an alternating tile grid; \
         even tiles fill first and odd tiles follow, creating a two-pass \
         chequerboard dissolve as progress crosses 0.5"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // Tile size: 4 dots wide × 4 dots tall (a 2×1 cell block — distinct from
        // the Bayer-dither which operates per dot with a 4×4 matrix).
        let tile_w = 4usize;
        let tile_h = 4usize;
        // Each tile has a phase in [0, 1): even tiles have phase 0, odd 0.5.
        // A tile is revealed when ctx.eased > phase.
        for ty in 0.. {
            let y0 = ty * tile_h;
            if y0 >= dh {
                break;
            }
            for tx in 0.. {
                let x0 = tx * tile_w;
                if x0 >= dw {
                    break;
                }
                let phase = if (tx + ty) % 2 == 0 { 0.0f32 } else { 0.5 };
                if ctx.eased > phase {
                    let tw = tile_w.min(dw.saturating_sub(x0));
                    let th = tile_h.min(dh.saturating_sub(y0));
                    draw::fill_rect(grid, x0, y0, tw, th);
                }
            }
        }
        // Tint by tile parity.
        let (cw, ch) = grid.dimensions();
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let tx = cx_idx / 2;
                let ty = cy_idx;
                let phase = if (tx + ty) % 2 == 0 { 0.0f32 } else { 0.5 };
                if ctx.eased > phase {
                    let t = if (tx + ty) % 2 == 0 { 0.3 } else { 0.7 };
                    let color = ctx.palette.sample(t);
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Venetian blinds: horizontal slats grow from their midpoints
// ─────────────────────────────────────────────────────────────────────────────

struct VenetianBlinds;
impl ProgressStyle for VenetianBlinds {
    fn name(&self) -> &str {
        "venetian-blinds"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Venetian-blinds wipe: the screen is divided into horizontal slats; each \
         slat opens symmetrically from its own centreline, all in unison, so the \
         full height is revealed in parallel stripes"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // Slat height in dots — 8 dots = 2 cell rows.
        let slat_h = 8usize;
        let num_slats = (dh + slat_h - 1) / slat_h;
        for s in 0..num_slats {
            let top = s * slat_h;
            let slat_actual = slat_h.min(dh.saturating_sub(top));
            let open_half = (ctx.eased * slat_actual as f32 / 2.0).round() as usize;
            let mid = top + slat_actual / 2;
            let y0 = mid.saturating_sub(open_half);
            let y1 = (mid + open_half).min(top + slat_actual);
            if y1 > y0 {
                draw::fill_rect(grid, 0, y0, dw, y1 - y0);
            }
        }
        // Tint alternating slats.
        let (cw, ch) = grid.dimensions();
        let slat_c = 2usize;
        for cy_idx in 0..ch {
            let slat_idx = cy_idx / slat_c;
            let t = if slat_idx % 2 == 0 { 0.25 } else { 0.75 };
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Dissolve: ordered Bayer-dither per-pixel threshold
// ─────────────────────────────────────────────────────────────────────────────

// 4×4 Bayer matrix values in [0, 15] mapped to [0, 1).
const BAYER: [[f32; 4]; 4] = [
    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
];

struct Dissolve;
impl ProgressStyle for Dissolve {
    fn name(&self) -> &str {
        "dissolve"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Ordered Bayer-dither dissolve: each dot has a threshold from a 4×4 \
         matrix; it lights up when progress exceeds its threshold, producing a \
         structured stipple that expands uniformly across the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        for dy in 0..dh {
            for dx in 0..dw {
                let bx = dx % 4;
                let by = dy % 4;
                if BAYER[by][bx] < ctx.eased {
                    draw::dot(grid, dx, dy);
                }
            }
        }
        // Tint with a uniform midpoint colour — the dither pattern provides
        // the structural variety; a flat colour avoids misleading gradients.
        let (cw, ch) = grid.dimensions();
        let color = ctx.palette.sample(0.5);
        for cy_idx in 0..ch {
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Clock-wipe: pie slice swept clockwise from 12 by a hand
// ─────────────────────────────────────────────────────────────────────────────

struct ClockWipe;
impl ProgressStyle for ClockWipe {
    fn name(&self) -> &str {
        "clock-wipe"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Clock wipe: a pie slice sweeps clockwise from 12 o'clock around the \
         centre, led by a bright clock hand with a hub dot, until the full \
         frame is revealed at progress 1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let two_pi = 2.0 * PI;
        // Sweep angle: 0 → 2π, measured clockwise from 12 o'clock.
        let sweep = ctx.eased.clamp(0.0, 1.0) * two_pi;
        // Angles are taken in frame-normalised space (the frame is treated as
        // a stretched square) so the swept area grows evenly with progress.
        // atan2(x, -y) is "clockwise from 12" in screen coords.
        let (nx, ny) = (cx.max(0.5), cy.max(0.5));
        let cw_angle = |vx: f32, vy: f32| (vx / nx).atan2(-vy / ny).rem_euclid(two_pi);
        for dy in 0..dh {
            let vy = dy as f32 + 0.5 - cy;
            for dx in 0..dw {
                let vx = dx as f32 + 0.5 - cx;
                if cw_angle(vx, vy) < sweep {
                    draw::dot(grid, dx, dy);
                }
            }
        }

        // The hand: a line from the hub along the sweep front, plus a fixed
        // 12 o'clock mark so the start of the dial is always readable.
        let (cw, ch) = grid.dimensions();
        let mut hand_cells = vec![false; cw * ch];
        let (hx, hy) = (sweep.sin() * nx, -sweep.cos() * ny);
        let steps = ((dw + dh) * 2) as i32;
        for i in 0..=steps {
            let d = 1.5 * i as f32 / steps.max(1) as f32;
            let (px, py) = (cx + hx * d, cy + hy * d);
            draw::dot_i(grid, px as i32, py as i32);
            if px >= 0.0 && py >= 0.0 {
                let (ux, uy) = (px as usize / 2, py as usize / 4);
                if ux < cw && uy < ch {
                    hand_cells[uy * cw + ux] = true;
                }
            }
        }
        let top = (cy as usize).saturating_sub(1).min(3);
        draw::vline(grid, cx as usize, 0, top);
        draw::fill_rect(
            grid,
            (cx as usize).saturating_sub(1),
            (cy as usize).saturating_sub(1),
            3,
            3,
        );

        // Tint swept cells by angle (cell centres measured in dot space);
        // cells along the hand are left untinted so the hand reads white.
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = cx_idx as f32 * 2.0 + 1.0 - cx;
                let vy = cy_idx as f32 * 4.0 + 2.0 - cy;
                let a = cw_angle(vx, vy);
                if a < sweep && !hand_cells[cy_idx * cw + cx_idx] {
                    let color = ctx.palette.sample((a / two_pi).clamp(0.0, 1.0));
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Spiral: grooved Archimedean tape unrolling outward
// ─────────────────────────────────────────────────────────────────────────────

struct Spiral;
impl ProgressStyle for Spiral {
    fn name(&self) -> &str {
        "spiral"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Spiral wipe: a coiled tape unrolls outward from the centre along an \
         Archimedean spiral; an empty groove between the coils keeps the spiral \
         visible and the colour runs along the tape from first coil to last"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        let two_pi = 2.0 * PI;
        // Squash x so the coils stay round-ish on a wide strip but still reach
        // the side edges; the outer coil is allowed to run off top/bottom.
        let sx = (cx / cy.max(1.0)).clamp(1.0, 2.5);
        let r_max = ((cx / sx).powi(2) + cy * cy).sqrt().max(1.0);
        let n_turns = (r_max / 4.0).clamp(1.0, 5.0);
        let e = ctx.eased.clamp(0.0, 1.0);

        // Spiral parameter of a point: which coil it is on plus how far round.
        // Returns (s in 0..1 along the tape, fractional position across coil).
        let param = |vx: f32, vy: f32| {
            let r = ((vx / sx).powi(2) + vy * vy).sqrt() / r_max;
            let a = vx.atan2(-vy).rem_euclid(two_pi) / two_pi;
            let turn = r * n_turns - a;
            let coil = turn.floor();
            ((coil + a) / n_turns, turn - coil)
        };

        // The outer coils mostly run off a wide strip, so reveal by on-screen
        // area: the threshold is the `e`-quantile of every dot's tape position.
        let mut all: Vec<f32> = Vec::with_capacity(dw * dh);
        for dy in 0..dh {
            for dx in 0..dw {
                all.push(param(dx as f32 + 0.5 - cx, dy as f32 + 0.5 - cy).0);
            }
        }
        all.sort_by(f32::total_cmp);
        let e = if all.is_empty() || e <= 0.0 {
            f32::NEG_INFINITY
        } else {
            all[((e * all.len() as f32) as usize).clamp(1, all.len()) - 1]
        };

        for dy in 0..dh {
            let vy = dy as f32 + 0.5 - cy;
            for dx in 0..dw {
                let vx = dx as f32 + 0.5 - cx;
                let (s, across) = param(vx, vy);
                // Groove: a thin empty band between neighbouring coils.
                if s <= e && across > 0.22 {
                    draw::dot(grid, dx, dy);
                }
            }
        }

        // Tint along the tape; the cells at the unrolling tip stay white.
        let (cw, ch) = grid.dimensions();
        let tip = 0.6 / (n_turns * 4.0).max(1.0);
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let vx = cx_idx as f32 * 2.0 + 1.0 - cx;
                let vy = cy_idx as f32 * 4.0 + 2.0 - cy;
                let (s, _) = param(vx, vy);
                if s <= e && e - s > tip {
                    let color = ctx.palette.sample(s.clamp(0.0, 1.0));
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Split: top and bottom edges advance inward to the midline
// ─────────────────────────────────────────────────────────────────────────────

struct Split;
impl ProgressStyle for Split {
    fn name(&self) -> &str {
        "split"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Split wipe: two horizontal bands advance from the top edge and bottom \
         edge simultaneously, meeting at the vertical midline when progress = 1"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let half_h = dh / 2;
        let reach = (ctx.eased * half_h as f32).round() as usize;
        // Top band: grows downward from row 0.
        draw::fill_rect(grid, 0, 0, dw, reach.min(dh));
        // Bottom band: grows upward from row dh-1.
        let bot_start = dh.saturating_sub(reach);
        if bot_start < dh {
            draw::fill_rect(grid, 0, bot_start, dw, dh - bot_start);
        }
        // Tint top half with palette start, bottom with palette end.
        let (cw, ch) = grid.dimensions();
        let half_c = ch / 2;
        let reach_c = (ctx.eased * half_c as f32).round() as usize;
        for cy_idx in 0..reach_c.min(ch) {
            let color = ctx.palette.sample(0.15);
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        let bot_start_c = ch.saturating_sub(reach_c);
        for cy_idx in bot_start_c..ch {
            let color = ctx.palette.sample(0.85);
            draw::tint_row(grid, cy_idx, 0, cw.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 13. Zigzag: wipe with a sine-wave leading edge
// ─────────────────────────────────────────────────────────────────────────────

struct Zigzag;
impl ProgressStyle for Zigzag {
    fn name(&self) -> &str {
        "zigzag"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Zigzag wipe: a left-to-right curtain whose leading edge is a sine wave, \
         producing a rippling serrated boundary that sweeps across the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        // The mean position of the wipe front.
        let mean_x = ctx.eased * dw as f32;
        // Amplitude of the sine wave on the leading edge (in dots).
        let amp = (dh as f32 * 0.35).max(1.0);
        // Frequency: one full wave per screen height.
        let freq = 2.0 * PI / dh.max(1) as f32;
        // Phase shift driven by time for animated ripple.
        let phase = ctx.time * 1.2;
        for dy in 0..dh {
            // The wipe boundary x for this row.
            let boundary = mean_x + amp * (freq * dy as f32 + phase).sin();
            let col_x = boundary.round() as i32;
            // Fill from 0 to col_x (clamped).
            let fill_end = col_x.max(0) as usize;
            for dx in 0..fill_end.min(dw) {
                draw::dot(grid, dx, dy);
            }
        }
        // Tint column-by-column.
        let (cw, ch) = grid.dimensions();
        for cy_idx in 0..ch {
            for cx_idx in 0..cw {
                let t = if cw <= 1 {
                    0.5
                } else {
                    cx_idx as f32 / (cw - 1) as f32
                };
                let color = ctx.palette.sample(t);
                // Only tint cells that are within the zigzag boundary.
                let dy_mid = cy_idx * 4 + 2;
                let boundary = mean_x + amp * (freq * dy_mid as f32 + phase).sin();
                if (cx_idx * 2) as f32 + 1.0 < boundary {
                    draw::tint_row(grid, cy_idx, cx_idx, cx_idx, color);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 14. Pixelate: coarse block fill with increasing shade density
// ─────────────────────────────────────────────────────────────────────────────

struct Pixelate;
impl ProgressStyle for Pixelate {
    fn name(&self) -> &str {
        "pixelate"
    }
    fn theme(&self) -> &str {
        "wipe"
    }
    fn describe(&self) -> &str {
        "Pixelate wipe: the screen is divided into coarse 2×1-cell blocks, each \
         ramp from empty → ░ → ▒ → ▓ → █ as progress rises through four density \
         thresholds — a block-density mosaic that fills by region, not by edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        // Block size in cells (2 wide × 1 tall — avoids conflating with Bayer).
        let block_w = 2usize;
        let block_h = 1;
        // Each block has a pseudorandom order based on its grid position so that
        // blocks light up in a spatial-hash order rather than simple scan order.
        // We use a cheap integer hash of (block_col, block_row).
        let num_bx = (cw + block_w - 1) / block_w;
        let num_by = (ch + block_h - 1) / block_h;
        let total_blocks = (num_bx * num_by).max(1);
        for bby in 0..num_by {
            for bbx in 0..num_bx {
                // Hash to a threshold in [0, 1).
                let hash = hash2(bbx as u32, bby as u32);
                let threshold = hash as f32 / u32::MAX as f32;
                // Local progress relative to this block's threshold.
                // Shade level 0..=4 based on how far past the threshold we are.
                let local = if ctx.eased <= threshold {
                    0.0
                } else {
                    ((ctx.eased - threshold) / (1.0 - threshold + 1e-6)).clamp(0.0, 1.0)
                };
                // Map to 0..=4 shade levels from SHADES.
                let level = (local * 4.0).floor() as usize;
                // Block cell coordinates.
                let cx0 = bbx * block_w;
                let cy0 = bby * block_h;
                // Each block covers block_w × block_h cells.
                for dy in 0..block_h {
                    for dx in 0..block_w {
                        let cell_x = cx0 + dx;
                        let cell_y = cy0 + dy;
                        if cell_x < cw && cell_y < ch {
                            draw::shade(grid, cell_x, cell_y, level);
                        }
                    }
                }
                // Tint filled blocks.
                if level > 0 {
                    let t = threshold;
                    let color = ctx.palette.sample(t);
                    for dy in 0..block_h {
                        let cell_y = cy0 + dy;
                        if cell_y < ch {
                            let cx1 = (cx0 + block_w - 1).min(cw.saturating_sub(1));
                            draw::tint_row(grid, cell_y, cx0.min(cw.saturating_sub(1)), cx1, color);
                        }
                    }
                }
                let _ = total_blocks; // suppress unused warning
            }
        }
        Ok(())
    }
}

/// Cheap integer hash of two u32 values → u32, used for block ordering.
/// Based on the Wang hash.
#[inline]
fn hash2(x: u32, y: u32) -> u32 {
    let mut h = x
        .wrapping_mul(2654435761)
        .wrapping_add(y.wrapping_mul(2246822519));
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}
