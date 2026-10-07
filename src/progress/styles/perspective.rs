//! Fake-3D-perspective progress bars — the **perspective** theme.
//!
//! Every style uses converging lines, vanishing-point geometry, and scrolling
//! depth cues as its *primary* communication mechanism.  `ctx.eased` encodes
//! *distance traveled into the scene* (0 = at the entrance, 1 = deep inside);
//! `ctx.time` drives continuous forward-motion animation (segments scroll
//! toward the viewer regardless of progress).
//!
//! # Style catalogue
//! | name | geometry |
//! |---|---|
//! | `vp-tunnel`        | Concentric rectangles → you fly in; rings scale and exit |
//! | `road-horizon`     | Two rails converging at a horizon; dashed center line scrolls |
//! | `starfield-dive`   | 3-D stars stream from the VP; destination planet grows with progress |
//! | `wire-corridor`    | Floor + ceiling grid lines converging to vanishing point |
//! | `checker-floor`    | Receding perspective checkerboard scrolling toward viewer |
//! | `infinite-hallway` | Doorframe rectangles at increasing depth; pass one per step |
//! | `mine-shaft`       | Descending shaft with horizontal floor markers |
//! | `depth-brackets`   | `[ [ [ > ] ] ]` nesting zooms to communicate depth |
//! | `approach-gate`    | A ring grows from the vanishing point until it fills the screen |
//! | `pipe-wormhole`    | Pipe rings rush out of the VP past you; exit mouth opens with progress |
//! | `parallax-layers`  | Mountains / hills / pines scroll at 3 speeds; moon crosses with progress |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// Integer Bresenham line rasteriser.  OOB dots are silently dropped.
/// Step count is bounded by `|dx|+|dy|+2` so no infinite loop is possible.
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let max_steps = (dx + dy.abs() + 2) as usize;
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

/// Deterministic hash → float in [0, 1).
#[inline]
fn hf(n: u32) -> f32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);
    (x % 1000) as f32 / 1000.0
}

// ────────────────────────────────────────────────────────────────────────────
// Registry
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — synthwave horizon.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(96, 162, 255);
const TINT_END: Color = Color::rgb(224, 96, 204);

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

/// All styles in the `perspective` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per fake-3D-perspective bar.  Each
/// style is geometrically distinct — they differ in their perspective
/// geometry, not merely in colour.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(VpTunnel)),
        Box::new(Tinted(RoadHorizon)),
        Box::new(Tinted(StarfieldDive)),
        Box::new(Tinted(WireCorridor)),
        Box::new(Tinted(CheckerFloor)),
        Box::new(Tinted(InfiniteHallway)),
        Box::new(Tinted(MineShaft)),
        Box::new(Tinted(DepthBrackets)),
        Box::new(Tinted(ApproachGate)),
        Box::new(Tinted(PipeWormhole)),
        Box::new(ParallaxLayers),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Vanishing-point tunnel
// ────────────────────────────────────────────────────────────────────────────
//
// Concentric rectangles shrink toward the center vanishing point.
// `ctx.eased` = how deep you are → the innermost rings are drawn first,
// outermost last, so a full bar fills the screen edge-to-edge.
// `ctx.time` pushes the ring phases forward so they appear to fly past.

struct VpTunnel;
impl ProgressStyle for VpTunnel {
    fn name(&self) -> &str {
        "vp-tunnel"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Concentric rectangles converge to a center vanishing point; rings fly \
         toward the viewer — depth filled = eased, forward rush = time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = (dw / 2) as f32;
        let cy = (dh / 2) as f32;

        // How many rings to draw — more at larger grids.
        let n_rings: usize = 8.max(dw.min(dh) / 2);
        // Scroll phase: time moves rings toward viewer (outward).
        let scroll = (ctx.time * 0.4).fract();

        // Only draw rings that are within the depth the user has "reached".
        let depth_rings = ((ctx.eased * n_rings as f32).ceil() as usize)
            .min(n_rings)
            .max(1);

        for i in 0..depth_rings {
            // Ring parameter 0..1 where 0 = tiny center, 1 = full-screen edge.
            // Scroll moves rings outward over time.
            let raw = (i as f32 + scroll) / n_rings as f32;
            let t = raw.fract(); // wrap so rings scroll continuously

            // Scale: at t=0 a ring sits at the vanishing point; at t=1 it fills the screen.
            // Use non-linear spacing so far rings are denser.
            let s = t * t;
            let half_w = (cx * s).max(0.5) as i32;
            let half_h = (cy * s).max(0.5) as i32;
            if half_w <= 0 || half_h <= 0 {
                continue;
            }

            let x0 = (cx as i32 - half_w).max(0);
            let y0 = (cy as i32 - half_h).max(0);
            let x1 = (cx as i32 + half_w).min(dw as i32 - 1);
            let y1 = (cy as i32 + half_h).min(dh as i32 - 1);

            // Draw the four edges of the rectangle.
            line(grid, x0, y0, x1, y0);
            line(grid, x0, y1, x1, y1);
            line(grid, x0, y0, x0, y1);
            line(grid, x1, y0, x1, y1);

            // Corner diagonals to the vanishing point — the converging lines.
            line(grid, x0, y0, cx as i32, cy as i32);
            line(grid, x1, y0, cx as i32, cy as i32);
            line(grid, x0, y1, cx as i32, cy as i32);
            line(grid, x1, y1, cx as i32, cy as i32);
        }

        // Vanishing-point dot.
        draw::dot_i(grid, cx as i32, cy as i32);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Road to horizon
// ────────────────────────────────────────────────────────────────────────────
//
// Two rails converge at a horizon point on the vertical center.
// Dashed center line segments scroll toward the viewer (time).
// Progress = how far down the road the "you are here" marker has traveled.

struct RoadHorizon;
impl ProgressStyle for RoadHorizon {
    fn name(&self) -> &str {
        "road-horizon"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Two rails converge at a horizon point; dashed center line scrolls toward \
         the viewer — how far you've driven = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Horizon sits at the top third of the grid.
        let horizon_y = (dh / 3).max(1) as i32;
        let vp_x = (dw / 2) as i32; // vanishing point x

        // Road width at the bottom of the screen.
        let road_half = (dw as i32 / 4).max(2);

        // Left rail: from (vp_x, horizon_y) to (vp_x - road_half, dh-1).
        let left_bot = (vp_x - road_half).max(0);
        let right_bot = (vp_x + road_half).min(dw as i32 - 1);

        line(grid, vp_x, horizon_y, left_bot, dh as i32 - 1);
        line(grid, vp_x, horizon_y, right_bot, dh as i32 - 1);

        // Horizon line.
        draw::hline(grid, 0, dw - 1, horizon_y as usize);

        // Dashed center line: segments between the rails, scrolling toward viewer.
        // We sample several y positions below the horizon.
        let seg_count = 8usize;
        let scroll_phase = (ctx.time * 0.5).fract();

        for s in 0..seg_count {
            // Map segment index to a depth t in (0, 1], 0=horizon, 1=bottom.
            let t_base = (s as f32 + scroll_phase) / seg_count as f32;
            let t = t_base.fract();

            // y position in dot space (below horizon).
            let seg_y = (horizon_y as f32 + (dh as f32 - horizon_y as f32) * t) as i32;
            // x position: interpolate between vp and screen bottom x.
            let half_w = (road_half as f32 * t * 0.5) as i32;

            // Dash: draw only every other segment.
            if s % 2 == 0 {
                draw::dot_i(grid, vp_x, seg_y);
                if half_w > 1 {
                    draw::dot_i(grid, vp_x, seg_y - 1);
                    draw::dot_i(grid, vp_x, seg_y + 1);
                }
            }
        }

        // "You are here" marker: a horizontal bar across the road at the eased depth.
        // t = 0 → horizon, t = 1 → bottom of screen.
        let marker_t = ctx.eased;
        let marker_y = (horizon_y as f32 + (dh as f32 - horizon_y as f32) * marker_t) as i32;
        let marker_half = (road_half as f32 * marker_t) as i32;
        let mx0 = (vp_x - marker_half).max(0);
        let mx1 = (vp_x + marker_half).min(dw as i32 - 1);
        if marker_y >= 0 && marker_y < dh as i32 {
            line(grid, mx0, marker_y, mx1, marker_y);
        }

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Starfield dive
// ────────────────────────────────────────────────────────────────────────────
//
// True 3-D starfield: each star has a fixed (x, y) and a depth z that falls
// toward the camera with time, so it is projected as `x / z` — born at the
// vanishing point, accelerating outward, leaving a streak back toward the
// centre.  Progress is the destination planet growing at the vanishing point
// (you are arriving) and the warp speed / streak length rising.

struct StarfieldDive;
impl ProgressStyle for StarfieldDive {
    fn name(&self) -> &str {
        "starfield-dive"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Warp-speed starfield: stars stream out of the vanishing point and \
         streak past the edges while the destination planet swells ahead"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // Focal length: a star at z = 1 sits ~quarter-height from the centre.
        let f = (dh as f32 * 0.25).max(1.0);
        let aspect = (dw as f32 / dh as f32).max(1.0);

        // Destination planet at the vanishing point — radius grows with eased.
        let pr = 0.6 + ctx.eased * dh as f32 * 0.42;
        let ri = pr.ceil() as i32;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let d2 = (dx * dx + dy * dy) as f32;
                if d2 <= pr * pr {
                    // Terminator: the right-hand crescent stays dark.
                    let sx = dx as f32 - pr * 0.45;
                    let lit = sx * sx + (dy * dy) as f32 > pr * pr * 0.55 || pr < 2.0;
                    if lit || d2 > (pr - 1.0) * (pr - 1.0) {
                        draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
                    }
                }
            }
        }

        let speed = 0.35 + ctx.eased * 0.9;
        let trail = 0.06 + ctx.eased * 0.22;
        for i in 0..64u32 {
            let x = (hf(i) * 2.0 - 1.0) * aspect;
            let y = hf(i + 500) * 2.0 - 1.0;
            if x.abs() < 0.08 && y.abs() < 0.08 {
                continue;
            }
            let z = (1.0 - (hf(i + 900) + ctx.time * speed).fract()).max(0.06);
            let z_tail = z + trail;
            let hx = cx + x * f / z;
            let hy = cy + y * f / z;
            let tx = cx + x * f / z_tail;
            let ty = cy + y * f / z_tail;
            // Stars behind the planet's disc are hidden.
            let (ox, oy) = (tx - cx, ty - cy);
            if ox * ox + oy * oy <= pr * pr {
                continue;
            }
            line(grid, tx as i32, ty as i32, hx as i32, hy as i32);
            // Near stars get a fat head.
            if z < 0.3 {
                draw::dot_i(grid, hx as i32 + 1, hy as i32);
                draw::dot_i(grid, hx as i32, hy as i32 + 1);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Wireframe corridor
// ────────────────────────────────────────────────────────────────────────────
//
// Floor and ceiling grid lines converge to a single vanishing point at the
// center of the screen.  Vertical wall stripes are equally spaced on screen
// and converge to the VP.  Floor lines scroll toward viewer via time.
// Eased controls how many vertical stripes / floor rows are visible.

struct WireCorridor;
impl ProgressStyle for WireCorridor {
    fn name(&self) -> &str {
        "wire-corridor"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Wireframe corridor: floor, ceiling, and wall grid lines converge to a \
         central vanishing point; floor lines scroll forward with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let vpx = (dw / 2) as i32;
        let vpy = (dh / 2) as i32;

        // Horizon line (corridor midline).
        draw::hline(grid, 0, dw - 1, vpy as usize);

        // Left and right wall edges.
        line(grid, 0, 0, vpx, vpy);
        line(grid, dw as i32 - 1, 0, vpx, vpy);
        line(grid, 0, dh as i32 - 1, vpx, vpy);
        line(grid, dw as i32 - 1, dh as i32 - 1, vpx, vpy);

        // Vertical wall stripes: several lines from the top screen edge to the
        // bottom edge, all converging to the VP.
        let stripe_count = 6usize;
        let lit_stripes = ((ctx.eased * stripe_count as f32).ceil() as usize)
            .min(stripe_count)
            .max(1);
        for s in 0..lit_stripes {
            // Evenly spaced x positions across the screen.
            let x = (s + 1) as i32 * dw as i32 / (stripe_count + 1) as i32;
            line(grid, x, 0, vpx, vpy);
            line(grid, x, dh as i32 - 1, vpx, vpy);
        }

        // Floor lines: horizontal segments between the two floor-edge lines
        // at various depths, scrolling toward the viewer.
        let floor_count = 8usize;
        let scroll = (ctx.time * 0.45).fract();
        for f in 0..floor_count {
            let t = ((f as f32 + scroll) / floor_count as f32).fract();
            // Map t → y below the horizon: 0=horizon, 1=bottom
            let fy = (vpy as f32 + (dh as f32 - vpy as f32) * t * t) as i32;
            if fy < 0 || fy >= dh as i32 {
                continue;
            }
            // Width of the floor line at this depth (wider near viewer).
            let fx_half = (dw as f32 / 2.0 * t).max(0.5) as i32;
            let fx0 = (vpx - fx_half).max(0);
            let fx1 = (vpx + fx_half).min(dw as i32 - 1);
            line(grid, fx0, fy, fx1, fy);
        }

        // Ceiling lines (mirror of floor above horizon).
        for f in 0..floor_count {
            let t = ((f as f32 + scroll) / floor_count as f32).fract();
            let fy = (vpy as f32 - (vpy as f32) * t * t) as i32;
            if fy < 0 || fy >= dh as i32 {
                continue;
            }
            let fx_half = (dw as f32 / 2.0 * t).max(0.5) as i32;
            let fx0 = (vpx - fx_half).max(0);
            let fx1 = (vpx + fx_half).min(dw as i32 - 1);
            line(grid, fx0, fy, fx1, fy);
        }

        // Vanishing point dot.
        draw::dot_i(grid, vpx, vpy);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Checkerboard floor
// ────────────────────────────────────────────────────────────────────────────
//
// A receding perspective-projected checkerboard occupies the lower half of the
// screen.  Row heights compress toward the horizon (1/depth spacing).  Time
// scrolls the checker pattern toward the viewer.  Eased controls how many
// rows (depth levels) are visible.

struct CheckerFloor;
impl ProgressStyle for CheckerFloor {
    fn name(&self) -> &str {
        "checker-floor"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Perspective checkerboard floor: rows compress toward the horizon; \
         pattern scrolls toward the viewer with time; depth = eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let horizon_y = (dh / 3).max(1);
        // Draw the horizon.
        draw::hline(grid, 0, dw - 1, horizon_y);

        let vpx = dw / 2;
        let floor_h = dh - horizon_y; // dots below horizon

        // Number of depth rows to render (controlled by eased).
        let max_rows = 10usize;
        let visible_rows = ((ctx.eased * max_rows as f32).ceil() as usize)
            .min(max_rows)
            .max(1);

        // Scroll phase shifts checker rows forward over time.
        let scroll = (ctx.time * 0.35).fract();

        for row in 0..visible_rows {
            // depth t: 0 = nearest, 1 = at horizon.
            // We invert so row 0 is closest (bottom of screen).
            let t_raw = (row as f32 + scroll) / max_rows as f32;
            let t = t_raw.fract();
            // Inverse-depth: y approaches horizon_y as t → 1.
            let y = (horizon_y as f32 + floor_h as f32 * (1.0 - t * t)) as usize;
            if y >= dh {
                continue;
            }

            // Width of the floor at this depth.
            let half_w = (vpx as f32 * (1.0 - t * 0.9)).max(0.0) as usize;
            let x0 = vpx.saturating_sub(half_w);
            let x1 = (vpx + half_w).min(dw - 1);

            // Column count at this depth: fewer columns near horizon.
            let col_count = (half_w * 2 / 4).max(1);
            for col in 0..col_count {
                let cx0 = x0 + col * (x1 - x0 + 1) / col_count;
                let cx1 = x0 + (col + 1) * (x1 - x0 + 1) / col_count;
                // Checker: light only even columns on even depth rows, odd on odd.
                let checker = (row + col) % 2;
                if checker == 0 {
                    draw::hline(grid, cx0, cx1.min(dw - 1), y);
                }
            }
        }

        // Converging left and right floor edges.
        line(grid, 0, dh as i32 - 1, vpx as i32, horizon_y as i32);
        line(
            grid,
            dw as i32 - 1,
            dh as i32 - 1,
            vpx as i32,
            horizon_y as i32,
        );

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Infinite hallway
// ────────────────────────────────────────────────────────────────────────────
//
// A series of nested doorframe rectangles at increasing depth.  The viewer
// "passes through" one doorway per unit of progress.  Eased = how many
// doorways have been crossed; time shifts doorway phases so you continuously
// step through them.

struct InfiniteHallway;
impl ProgressStyle for InfiniteHallway {
    fn name(&self) -> &str {
        "infinite-hallway"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Nested doorframe rectangles at increasing depth — each door crossed = \
         one step of eased; doors scroll toward viewer with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;

        // Total doorways to draw.
        let n_doors = 8usize;
        // Time scrolls the phases forward (door indices advance).
        let scroll = (ctx.time * 0.4).fract();
        // Eased controls how many doorways are visible / how deep the view extends.
        let visible = ((ctx.eased * n_doors as f32).ceil() as usize)
            .min(n_doors)
            .max(1);

        for i in 0..visible {
            // t = 0 → smallest/farthest, t = 1 → largest/nearest.
            let raw = (i as f32 + scroll) / n_doors as f32;
            let t = raw.fract();
            // Non-linear scale: quadratic so far doors are densely packed.
            let s = t * t;

            let hw = ((dw as f32 / 2.0) * s).max(1.0) as i32;
            let hh = ((dh as f32 / 2.0) * s).max(1.0) as i32;

            let x0 = (cx - hw).max(0);
            let y0 = (cy - hh).max(0);
            let x1 = (cx + hw).min(dw as i32 - 1);
            let y1 = (cy + hh).min(dh as i32 - 1);

            // Draw the doorframe: top, left, right edges only (open at bottom = floor).
            line(grid, x0, y0, x1, y0); // top
            line(grid, x0, y0, x0, y1); // left
            line(grid, x1, y0, x1, y1); // right

            // Corner diagonals to the VP (depth cue).
            line(grid, x0, y0, cx, cy);
            line(grid, x1, y0, cx, cy);
        }

        // Vanishing-point marker.
        draw::dot_i(grid, cx, cy);
        draw::dot_i(grid, cx - 1, cy);
        draw::dot_i(grid, cx + 1, cy);
        draw::dot_i(grid, cx, cy - 1);
        draw::dot_i(grid, cx, cy + 1);

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Mine shaft / elevator descent
// ────────────────────────────────────────────────────────────────────────────
//
// Vertical shaft with horizontal floor markers that scroll upward as you
// descend.  Eased = depth gauge on the right side (how far down you are).
// Time drives the upward scroll of floor markers.

struct MineShaft;
impl ProgressStyle for MineShaft {
    fn name(&self) -> &str {
        "mine-shaft"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Descending shaft: floor markers scroll upward with time; depth gauge \
         on the right fills with eased; you're falling down and in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // Shaft walls: converge from wide at top to narrow near the bottom-center.
        let shaft_top_half = (dw as i32 / 3).max(2);
        let shaft_bot_half = (dw as i32 / 6).max(1);
        let vpx = (dw / 2) as i32;

        // Left and right shaft walls (converging vertical lines).
        line(
            grid,
            vpx - shaft_top_half,
            0,
            vpx - shaft_bot_half,
            dh as i32 - 1,
        );
        line(
            grid,
            vpx + shaft_top_half,
            0,
            vpx + shaft_bot_half,
            dh as i32 - 1,
        );

        // Horizontal floor markers scrolling upward (time).
        let marker_count = 10usize;
        let scroll = (ctx.time * 0.55).fract();

        for m in 0..marker_count {
            let t = ((m as f32 + scroll) / marker_count as f32).fract();
            // y: 0=top, 1=bottom (markers scroll from bottom to top as you descend).
            let y = (dh as f32 * (1.0 - t)) as i32;
            if y < 0 || y >= dh as i32 {
                continue;
            }

            // Shaft width at this y (interpolate between top and bottom half).
            let frac_y = y as f32 / dh as f32;
            let half_w =
                (shaft_top_half as f32 * (1.0 - frac_y) + shaft_bot_half as f32 * frac_y) as i32;
            let x0 = (vpx - half_w).max(0);
            let x1 = (vpx + half_w).min(dw as i32 - 1);
            line(grid, x0, y, x1, y);

            // Tick marks on the sides.
            draw::dot_i(grid, x0 - 1, y);
            draw::dot_i(grid, x1 + 1, y);
        }

        // Depth gauge: a filled column on the far right, filled proportional to eased.
        let gauge_x = (dw as i32 - 2).max(0);
        let filled_h = (ctx.eased * dh as f32).round() as i32;
        for y in 0..filled_h.min(dh as i32) {
            draw::dot_i(grid, gauge_x, y);
            draw::dot_i(grid, gauge_x + 1, y);
        }
        // Gauge outline.
        draw::vline(grid, gauge_x as usize, 0, dh - 1);
        draw::dot_i(grid, gauge_x, 0);
        draw::dot_i(grid, gauge_x, dh as i32 - 1);

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Depth brackets
// ────────────────────────────────────────────────────────────────────────────
//
// `[ [ [ > ] ] ]` — nested bracket pairs that zoom in, each pair one depth
// level deeper.  The number of visible bracket pairs = eased * max_depth.
// A `>` arrow (or a line of dots) shows the current position.
// Time causes the brackets to pulse / breathe slightly.

struct DepthBrackets;
impl ProgressStyle for DepthBrackets {
    fn name(&self) -> &str {
        "depth-brackets"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Nested [ [ [ > ] ] ] bracket pairs zoom inward; visible depth = \
         eased; inner marker pulses with time — pure typographic perspective"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;

        let max_depth = 6usize;
        let depth = ((ctx.eased * max_depth as f32).ceil() as usize)
            .min(max_depth)
            .max(1);

        // Subtle breathing animation via time.
        let breath = (ctx.time * 1.5).sin() * 0.04 + 1.0;

        for d in 0..depth {
            // The outermost bracket uses the full width; each inner one shrinks.
            let frac = (max_depth - d) as f32 / max_depth as f32;
            let hw = ((dw as f32 / 2.0) * frac * breath).max(1.0) as i32;
            let hh = ((dh as f32 / 2.0) * frac * breath).max(1.0) as i32;

            let x0 = (cx - hw).max(0);
            let y0 = (cy - hh).max(0);
            let x1 = (cx + hw).min(dw as i32 - 1);
            let y1 = (cy + hh).min(dh as i32 - 1);

            // Left bracket: vertical bar + horizontal serifs at top and bottom.
            draw::dot_i(grid, x0, y0);
            draw::dot_i(grid, x0, y1);
            draw::vline(grid, x0 as usize, y0 as usize, y1 as usize);
            // Short horizontal serifs.
            draw::dot_i(grid, x0 + 1, y0);
            draw::dot_i(grid, x0 + 1, y1);

            // Right bracket mirror.
            draw::dot_i(grid, x1, y0);
            draw::dot_i(grid, x1, y1);
            draw::vline(grid, x1 as usize, y0 as usize, y1 as usize);
            draw::dot_i(grid, x1 - 1, y0);
            draw::dot_i(grid, x1 - 1, y1);
        }

        // Center `>` marker: three dots forming an arrowhead.
        let pulse = ((ctx.time * 3.0).sin() * 1.5) as i32;
        draw::dot_i(grid, cx + pulse, cy);
        draw::dot_i(grid, cx + pulse - 1, cy - 1);
        draw::dot_i(grid, cx + pulse - 1, cy + 1);
        draw::dot_i(grid, cx + pulse - 2, cy - 2);
        draw::dot_i(grid, cx + pulse - 2, cy + 2);

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Approaching gate / portal
// ────────────────────────────────────────────────────────────────────────────
//
// A single elliptical ring starts tiny at the vanishing point and grows until
// it fills (and exits) the screen at progress=1.  Multiple rings at different
// phases stream toward the viewer; eased = how close the lead ring is.

struct ApproachGate;
impl ProgressStyle for ApproachGate {
    fn name(&self) -> &str {
        "approach-gate"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "A portal ring grows from the vanishing point until it fills the screen \
         at 100%; multiple rings stream toward the viewer continuously"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as f32;
        let cy = (dh / 2) as f32;
        let max_rx = cx;
        let max_ry = cy;

        // Stream of rings scrolling toward viewer.
        let n_rings = 5usize;
        let scroll = (ctx.time * 0.5).fract();

        // The lead ring's phase is clamped to eased (can't go further than progress).
        for i in 0..n_rings {
            let raw = (i as f32 + scroll) / n_rings as f32;
            let phase = raw.fract();
            // The lead ring (i=0 after scroll) is bounded by eased.
            // Others trail behind it.
            let t = if i == 0 {
                // Lead ring: its size is driven by eased, animated by scroll.
                // Blend: use the larger of the scroll-driven phase and eased.
                phase.max(ctx.eased * phase)
            } else {
                phase * ctx.eased
            };

            // Scale: small = far, large = near.
            let rx = (max_rx * t * t).max(0.5);
            let ry = (max_ry * t * t).max(0.5);

            // Draw ellipse via parametric steps.
            let steps = ((rx + ry) * 4.0).max(8.0) as usize;
            let mut prev: Option<(i32, i32)> = None;
            for s in 0..=steps {
                let angle = s as f32 / steps as f32 * 2.0 * PI;
                let px = (cx + rx * angle.cos()) as i32;
                let py = (cy + ry * angle.sin()) as i32;
                if let Some((ppx, ppy)) = prev {
                    line(grid, ppx, ppy, px, py);
                }
                prev = Some((px, py));
            }
        }

        // Vanishing-point dot.
        draw::dot_i(grid, cx as i32, cy as i32);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Pipe / wormhole
// ────────────────────────────────────────────────────────────────────────────
//
// Head-on view down a circular pipe.  Ring k sits at depth z = fract(k/N −
// t·v) and is projected with radius ∝ 1/z, so rings are born as a pin-prick
// at the vanishing point and accelerate outward until they rush past the
// frame.  Longitudinal seams twist slowly.  Progress is the exit mouth at
// the far end, opening wider the closer you get.

struct PipeWormhole;
impl ProgressStyle for PipeWormhole {
    fn name(&self) -> &str {
        "pipe-wormhole"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Flying down a circular pipe: concentric rings rush out of the \
         vanishing point past you along twisting seams; the exit opens with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cx = dw as f32 / 2.0;
        let cy = dh as f32 / 2.0;
        // Ellipse stretch so the pipe fills a wide bar; r0 = radius at z = 1.
        let stretch = (dw as f32 / dh as f32).clamp(1.0, 2.6);
        let r0 = (dh as f32 * 0.22).max(0.5);
        let z_near = 0.12;

        // Exit mouth at the far end: a filled glow, radius grows with eased.
        let er = 0.5 + ctx.eased * dh as f32 * 0.3;
        let eri = (er * stretch).ceil() as i32;
        let erj = er.ceil() as i32;
        for dy in -erj..=erj {
            for dx in -eri..=eri {
                let nx = dx as f32 / stretch;
                if nx * nx + (dy * dy) as f32 <= er * er {
                    draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
                }
            }
        }
        let exit_r = er + 1.5;

        // Rings flying toward the camera.
        let n_rings = 7usize;
        let speed = 0.35 + ctx.eased * 0.5;
        for k in 0..n_rings {
            let z = 1.0 - (k as f32 / n_rings as f32 + ctx.time * speed).fract();
            let r = r0 / z.max(z_near);
            if r < exit_r {
                continue;
            }
            let steps = ((r * stretch * 4.0) as usize).clamp(12, 400);
            let mut prev: Option<(i32, i32)> = None;
            for s in 0..=steps {
                let a = s as f32 / steps as f32 * 2.0 * PI;
                let p = (
                    (cx + r * stretch * a.cos()) as i32,
                    (cy + r * a.sin()) as i32,
                );
                if let Some(q) = prev {
                    line(grid, q.0, q.1, p.0, p.1);
                }
                prev = Some(p);
            }
        }

        // Twisting seams along the pipe wall, dotted at even depth steps.
        let twist = ctx.time * 0.6;
        for s in 0..8 {
            let a = twist + s as f32 * PI / 4.0;
            // 0.86^14 ≈ z_near: fifteen depth steps from z = 1 to the far end.
            for n in 0..15 {
                let r = r0 / 0.86f32.powi(n);
                if r >= exit_r {
                    draw::dot_i(
                        grid,
                        (cx + r * stretch * a.cos()) as i32,
                        (cy + r * a.sin()) as i32,
                    );
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Parallax layers
// ────────────────────────────────────────────────────────────────────────────
//
// A side-scrolling landscape in three depth planes: a far mountain ridge
// (outline + snow caps, slowest), rolling dithered hills (medium) and a row
// of solid pine trees on the ground (fastest).  Each plane gets its own
// colour, frontmost layer winning a cell.  Progress is the moon crossing the
// sky from left to right above the ridge.

struct ParallaxLayers;

/// Mountain ridge height (dots above the bottom) at world x.
fn ridge(x: f32, dh: f32) -> f32 {
    let a = (x * 0.11).sin() * 0.5 + (x * 0.047 + 1.3).sin() * 0.35 + (x * 0.23).sin() * 0.15;
    // Fold into peaks: |sin| gives sharp valleys turned upside down.
    dh * (0.48 + 0.28 * a)
}

impl ProgressStyle for ParallaxLayers {
    fn name(&self) -> &str {
        "parallax-layers"
    }
    fn theme(&self) -> &str {
        "perspective"
    }
    fn describe(&self) -> &str {
        "Side-scrolling landscape: far mountains, rolling hills and near pine \
         trees glide at three speeds while the moon crosses the sky with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (cw, ch) = grid.dimensions();
        // Layer owning each cell: 0 none, 1 moon, 2 mountains, 3 hills, 4 trees.
        let mut owner = vec![0u8; cw * ch];
        let mark = |owner: &mut [u8], x: i32, y: i32, layer: u8| {
            if x >= 0 && y >= 0 && (x as usize) < dw && (y as usize) < dh {
                let i = (y as usize / 4) * cw + x as usize / 2;
                if layer > owner[i] {
                    owner[i] = layer;
                }
            }
        };
        let hf_ = dh as f32;
        let t = ctx.time;

        // Far mountains: ridge outline with snow on the high peaks.
        let off_m = t * 2.0;
        let ridge_y: Vec<i32> = (0..dw)
            .map(|x| (hf_ - ridge(x as f32 + off_m, hf_)).round() as i32)
            .collect();

        // Moon: crosses the sky with progress, hidden behind the ridge.
        let mr = (hf_ * 0.16).clamp(1.0, 4.0);
        let mx = mr + 1.0 + ctx.eased * (dw as f32 - 2.0 * mr - 2.0).max(0.0);
        let my = mr + 0.5;
        let mri = mr.ceil() as i32;
        for dy in -mri..=mri {
            for dx in -mri..=mri {
                if (dx * dx + dy * dy) as f32 <= mr * mr {
                    let (x, y) = (mx as i32 + dx, my as i32 + dy);
                    let behind = x >= 0 && (x as usize) < dw && y >= ridge_y[x as usize];
                    if !behind {
                        draw::dot_i(grid, x, y);
                        mark(&mut owner, x, y, 1);
                    }
                }
            }
        }

        for (x, &ry) in ridge_y.iter().enumerate() {
            let x = x as i32;
            draw::dot_i(grid, x, ry);
            mark(&mut owner, x, ry, 2);
            if (ry as f32) < hf_ * 0.32 {
                // Snow cap: fill a little under the high ridge.
                draw::dot_i(grid, x, ry + 1);
                mark(&mut owner, x, ry + 1, 2);
            }
        }

        // Mid hills: smooth swells, sparse stipple fill.
        let off_h = t * 6.0;
        for x in 0..dw as i32 {
            let wx = x as f32 + off_h;
            let top = hf_ * (0.72 - 0.16 * (wx * 0.07).sin() - 0.06 * (wx * 0.19).sin());
            let top = top.round() as i32;
            draw::dot_i(grid, x, top);
            mark(&mut owner, x, top, 3);
            for y in top + 1..dh as i32 {
                if (x + 2 * y) % 5 == 0 {
                    draw::dot_i(grid, x, y);
                    mark(&mut owner, x, y, 3);
                }
            }
        }

        // Near trees: solid pines on a ground line, fastest.
        let gy = dh as i32 - 1;
        let off_t = (t * 14.0) as i32;
        let spacing = 19i32;
        let tree_h = (hf_ * 0.7).max(3.0) as i32;
        for x in 0..dw as i32 {
            draw::dot_i(grid, x, gy);
            mark(&mut owner, x, gy, 4);
        }
        let first = -(off_t.rem_euclid(spacing));
        let mut k = 0i32;
        let mut tx = first;
        while tx < dw as i32 + spacing {
            // Vary height per tree from its world index.
            let idx = (off_t.div_euclid(spacing) + k) as u32;
            let th = tree_h - (hf(idx) * hf_ * 0.25) as i32;
            let top = gy - th;
            for y in top..gy {
                // Canopy widens downward; short trunk at the base.
                let depth = y - top;
                let half = if y >= gy - 2 {
                    0
                } else {
                    (depth * 2 / 5).min(4)
                };
                for dx in -half..=half {
                    draw::dot_i(grid, tx + dx, y);
                    mark(&mut owner, tx + dx, y, 4);
                }
            }
            tx += spacing;
            k += 1;
        }

        grid.enable_color_support();
        for (i, &o) in owner.iter().enumerate() {
            let c = match o {
                1 => Color::rgb(250, 236, 180),
                2 => Color::rgb(110, 128, 200),
                3 => Color::rgb(176, 104, 214),
                4 => Color::rgb(236, 92, 170),
                _ => continue,
            };
            let _ = grid.set_cell_color(i % cw, i / cw, c);
        }
        Ok(())
    }
}
