//! Golden-ratio sacred-geometry progress bars.
//!
//! Every style is built from φ (the golden ratio) and its consequences:
//! Fibonacci tiling, the logarithmic spiral, pentagonal symmetry, the golden
//! gnomon, Kepler's triangle, and the 137.5° golden angle. All shapes are
//! constructed with exact φ-based mathematics and revealed by `ctx.eased`
//! while `ctx.time` drives continuous rotation or animation.
//!
//! Style catalogue:
//! - `golden-spiral`        — quarter-circle arcs in Fibonacci squares forming the logarithmic spiral
//! - `golden-rectangle`     — recursive whirling-rectangles φ-subdivision
//! - `fibonacci-squares`    — Fibonacci squares growing outward, the view zooming to frame them
//! - `pentagram`            — upright {5/2} star in its pentagon, the next star nested by 1/φ²
//! - `pentagon-nest`        — concentric pentagons scaled by 1/φ², rotating
//! - `golden-gnomon`        — golden triangles cut into gnomons, whirling inward in a spiral
//! - `phi-phyllotaxis-pent` — five-fold seed arrangement at golden angle
//! - `nautilus`             — chambered φ-logarithmic shell traced from the lip inward, septa every 45°
//! - `golden-angle-rays`    — rays at successive 137.5° increments, length growing
//! - `dodecagram`           — frieze of {12/5} twelve-pointed stars traced chord by chord
//! - `kepler-triangle`      — right triangle 1 : √φ : φ with the squares on its sides

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// The golden ratio φ = (1 + √5) / 2.
const PHI: f32 = 1.618_033_9;

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — burnished gold.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 204, 92);
const TINT_END: Color = Color::rgb(196, 120, 28);

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

/// All styles in the `goldenratio` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per golden-ratio / φ-geometry bar.
/// Styles are ordered from the most recognisable (golden spiral) to the most
/// exotic (Kepler triangle fan), but are fully independent.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(GoldenSpiral)),
        Box::new(Tinted(GoldenRectangle)),
        Box::new(Tinted(FibonacciSquares)),
        Box::new(Tinted(Pentagram)),
        Box::new(Tinted(PentagonNest)),
        Box::new(Tinted(GoldenGnomon)),
        Box::new(Tinted(PhiPhyllotaxisPent)),
        Box::new(Tinted(Nautilus)),
        Box::new(Tinted(GoldenAngleRays)),
        Box::new(Tinted(Dodecagram)),
        Box::new(Tinted(KeplerTriangle)),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Grid center in dot-space (floating-point).
#[inline]
fn center(dw: usize, dh: usize) -> (f32, f32) {
    (dw as f32 / 2.0, dh as f32 / 2.0)
}

/// Uniform scale so a unit-radius figure fits the grid with a small margin.
#[inline]
fn fit_scale(dw: usize, dh: usize) -> f32 {
    let hw = (dw as f32 / 2.0 - 1.0).max(1.0);
    let hh = (dh as f32 / 2.0 - 1.0).max(1.0);
    hw.min(hh)
}

/// Integer Bresenham line between two signed dot-space points.
/// Out-of-bounds dots are silently discarded by `draw::dot_i`.
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

/// Draw a circular arc from angle `a0` to `a1` (radians), radius `r` dots,
/// centered at dot-space `(cx, cy)`.  Step count is proportional to arc length.
fn arc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r < 0.5 {
        return;
    }
    let arc_len = (a1 - a0).abs() * r;
    let steps = (arc_len * 2.0).round() as usize;
    let steps = steps.max(2);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = a0 + t * (a1 - a0);
        let px = (cx + r * angle.cos()).round() as i32;
        let py = (cy - r * angle.sin()).round() as i32;
        draw::dot_i(grid, px, py);
    }
}

/// Draw a regular N-gon in dot-space, centered at (cx, cy), radius r dots,
/// base rotation `rot` radians. Returns the vertex coordinates.
fn ngon_vertices(n: usize, cx: f32, cy: f32, r: f32, rot: f32) -> Vec<(i32, i32)> {
    (0..n)
        .map(|i| {
            let angle = rot + 2.0 * PI * i as f32 / n as f32;
            (
                (cx + r * angle.cos()).round() as i32,
                (cy - r * angle.sin()).round() as i32,
            )
        })
        .collect()
}

/// Draw the outline of a regular N-gon.
fn ngon_outline(grid: &mut BrailleGrid, n: usize, cx: f32, cy: f32, r: f32, rot: f32) {
    let verts = ngon_vertices(n, cx, cy, r, rot);
    for i in 0..n {
        let (x0, y0) = verts[i];
        let (x1, y1) = verts[(i + 1) % n];
        bresenham(grid, x0, y0, x1, y1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Golden Spiral — quarter-circle arcs inscribed in Fibonacci squares
// ─────────────────────────────────────────────────────────────────────────────
//
// The canonical 21×13 Fibonacci rectangle: squares 13, 8, 5, 3, 2, 1, 1 cut
// off in the whirling order left → top → right → bottom, each hosting a
// quarter-circle arc of its own side length.  The arcs join into the true
// golden spiral.  `ctx.eased` traces the spiral inward from the big square
// (each square's outline appears as the arc enters it) while the figure rides
// a dotted rail across the strip; a bead runs the spiral on `ctx.time`.

/// Fibonacci side lengths, outermost first; the bounding box is 21 × 13.
const FIB_SIDES: [f32; 7] = [13.0, 8.0, 5.0, 3.0, 2.0, 1.0, 1.0];

/// One square of the whirling layout: top-left `(x, y)`, side `s`, and the arc
/// centre `(acx, acy)` with start angle `a0` (the arc sweeps −90° from `a0`).
#[derive(Clone, Copy)]
struct FibSquare {
    x: f32,
    y: f32,
    s: f32,
    acx: f32,
    acy: f32,
    a0: f32,
}

/// Lay out the seven squares in unit space (y down), outermost first.
fn fib_layout() -> [FibSquare; 7] {
    let mut out = [FibSquare {
        x: 0.0,
        y: 0.0,
        s: 0.0,
        acx: 0.0,
        acy: 0.0,
        a0: 0.0,
    }; 7];
    // Remaining rectangle (x, y, w, h).
    let (mut rx, mut ry, mut rw, mut rh) = (0.0_f32, 0.0_f32, 21.0_f32, 13.0_f32);
    for (i, &s) in FIB_SIDES.iter().enumerate() {
        let sq = match i % 4 {
            // Left: arc centred bottom-right, sweeping 180° → 90°.
            0 => {
                let q = (rx, ry);
                rx += s;
                rw -= s;
                (q.0, q.1, q.0 + s, q.1 + s, PI)
            }
            // Top: arc centred bottom-left, 90° → 0°.
            1 => {
                let q = (rx, ry);
                ry += s;
                rh -= s;
                (q.0, q.1, q.0, q.1 + s, PI / 2.0)
            }
            // Right: arc centred top-left, 0° → −90°.
            2 => {
                let q = (rx + rw - s, ry);
                rw -= s;
                (q.0, q.1, q.0, q.1, 0.0)
            }
            // Bottom: arc centred top-right, −90° → −180°.
            _ => {
                let q = (rx, ry + rh - s);
                rh -= s;
                (q.0, q.1, q.0 + s, q.1, -PI / 2.0)
            }
        };
        out[i] = FibSquare {
            x: sq.0,
            y: sq.1,
            s,
            acx: sq.2,
            acy: sq.3,
            a0: sq.4,
        };
    }
    out
}

struct GoldenSpiral;
impl ProgressStyle for GoldenSpiral {
    fn name(&self) -> &str {
        "golden-spiral"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "The true golden spiral: quarter-circle arcs through the 13-8-5-3-2-1-1 \
         Fibonacci squares, traced inward with progress as the rectangle rides \
         a rail and a bead runs the curve"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        // 13 units span the full height; the 21-unit width follows at φ.
        let k = (dh as f32 - 1.0) / 13.0;
        let fw = 21.0 * k;
        let travel = (dw as f32 - 1.0 - fw).max(0.0);
        let ox = (ctx.eased.clamp(0.0, 1.0) * travel).round();
        let oy = 0.0;

        // Dotted rail behind the figure, through the spiral's eye.
        let rail_y = (oy + 9.0 * k).round() as i32;
        let mut x = 0;
        let rail_end = (ox - 1.0).ceil() as i32;
        while x < rail_end {
            draw::dot_i(grid, x, rail_y);
            x += 2;
        }

        // Fraction of total arc length traced (outermost first).
        let total: f32 = FIB_SIDES.iter().sum();
        let traced = (0.3 + ctx.eased * 1.4).min(1.0) * total;
        let squares = fib_layout();
        let mut acc = 0.0;
        for sq in &squares {
            if traced <= acc {
                break;
            }
            let x0 = (ox + sq.x * k).round() as i32;
            let y0 = (oy + sq.y * k).round() as i32;
            let x1 = (ox + (sq.x + sq.s) * k).round() as i32;
            let y1 = (oy + (sq.y + sq.s) * k).round() as i32;
            bresenham(grid, x0, y0, x1, y0);
            bresenham(grid, x1, y0, x1, y1);
            bresenham(grid, x1, y1, x0, y1);
            bresenham(grid, x0, y1, x0, y0);
            let frac = ((traced - acc) / sq.s).min(1.0);
            arc(
                grid,
                ox + sq.acx * k,
                oy + sq.acy * k,
                sq.s * k,
                sq.a0,
                sq.a0 - frac * PI / 2.0,
            );
            acc += sq.s;
        }

        // Bead running the drawn spiral inward, looping every ~2.5 s.
        let run = (ctx.time * 0.4).fract() * traced;
        let mut acc = 0.0;
        for sq in &squares {
            if run < acc + sq.s {
                let a = sq.a0 - (run - acc) / sq.s * PI / 2.0;
                let bx = (ox + sq.acx * k + sq.s * k * a.cos()).round() as i32;
                let by = (oy + sq.acy * k - sq.s * k * a.sin()).round() as i32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)] {
                    draw::dot_i(grid, bx + dx, by + dy);
                }
                break;
            }
            acc += sq.s;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Golden Rectangle — recursive whirling-rectangles φ-subdivision
// ─────────────────────────────────────────────────────────────────────────────
//
// Start with a φ-rectangle. Cut a square from the long side → leaves a smaller
// φ-rectangle. Recurse. The squares + remaining rectangles are the "whirling
// rectangles". `ctx.eased` controls recursion depth.

struct GoldenRectangle;
impl ProgressStyle for GoldenRectangle {
    fn name(&self) -> &str {
        "golden-rectangle"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Recursive φ-rectangle subdivision into a square + smaller φ-rectangle — \
         the whirling-rectangles diagram, with each subdivision revealed as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);

        // Fit the initial φ-rectangle (width = PHI, height = 1) into the grid.
        // Scale so width fits within dw, height within dh.
        let scale_w = (dw as f32 - 2.0) / PHI;
        let scale_h = (dh as f32 - 2.0) / 1.0;
        let unit = scale_w.min(scale_h).max(1.0);

        // Initial rectangle corners in dot-space (top-left origin).
        let rect_w = PHI * unit;
        let rect_h = 1.0 * unit;
        let ox = ((dw as f32 - rect_w) / 2.0).max(0.0);
        let oy = ((dh as f32 - rect_h) / 2.0).max(0.0);

        let max_depth: usize = 10;
        let depth = ((ctx.eased * max_depth as f32).round() as usize)
            .min(max_depth)
            .max(1);

        // Recursive subdivision: (x0, y0, w, h, horizontal_cut)
        // horizontal_cut=true means we cut a square from the left side (width = h).
        let rects: Vec<(f32, f32, f32, f32, bool)> = vec![(ox, oy, rect_w, rect_h, true)];
        let mut drawn = 0usize;

        // Draw outer rectangle always.
        {
            let (x0, y0, w, h, _) = rects[0];
            draw::rect_outline(
                grid,
                x0.round() as usize,
                y0.round() as usize,
                w.round().max(1.0) as usize,
                h.round().max(1.0) as usize,
            );
        }
        drawn += 1;

        let mut next_rects: Vec<(f32, f32, f32, f32, bool)> = Vec::new();
        let mut current = rects;

        for _d in 0..depth.saturating_sub(1) {
            next_rects.clear();
            for &(x0, y0, w, h, horiz) in &current {
                if horiz {
                    // Cut a square of side h from the left.
                    let sq = h;
                    if sq >= 1.0 && w - sq >= 1.0 {
                        // Draw the dividing line between square and remainder.
                        let lx = (x0 + sq).round() as usize;
                        let ly0 = y0.round() as usize;
                        let ly1 = (y0 + h - 1.0).round() as usize;
                        if lx < dw {
                            draw::vline(
                                grid,
                                lx,
                                ly0.min(dh.saturating_sub(1)),
                                ly1.min(dh.saturating_sub(1)),
                            );
                        }
                        // The remaining rectangle is vertical (h > w).
                        next_rects.push((x0 + sq, y0, w - sq, h, false));
                        drawn += 1;
                    }
                } else {
                    // Cut a square of side w from the top.
                    let sq = w;
                    if sq >= 1.0 && h - sq >= 1.0 {
                        let ly = (y0 + sq).round() as usize;
                        let lx0 = x0.round() as usize;
                        let lx1 = (x0 + w - 1.0).round() as usize;
                        if ly < dh {
                            draw::hline(
                                grid,
                                lx0.min(dw.saturating_sub(1)),
                                lx1.min(dw.saturating_sub(1)),
                                ly,
                            );
                        }
                        next_rects.push((x0, y0 + sq, w, h - sq, true));
                        drawn += 1;
                    }
                }
                if drawn >= depth * 2 {
                    break;
                }
            }
            if next_rects.is_empty() {
                break;
            }
            current.clone_from(&next_rects);
        }
        let _ = drawn;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Fibonacci Squares — outward tiling of F₁,F₁,F₂,F₃,F₅,F₈,…
// ─────────────────────────────────────────────────────────────────────────────
//
// Draws only the square outlines (not the arcs) so the tiling geometry is the
// hero.  Each new square appears as `ctx.eased` steps forward.  The tiling is
// centered and scaled to fit any grid.

struct FibonacciSquares;
impl ProgressStyle for FibonacciSquares {
    fn name(&self) -> &str {
        "fibonacci-squares"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Fibonacci squares 1, 1, 2, 3, 5, 8, 13 growing outward in the whirling \
         layout, the view zooming out to frame each new square as it is inked"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let squares = fib_layout();
        let n = squares.len();
        // Squares shown innermost first; the newest is still being inked.
        let steps = ctx.eased.clamp(0.0, 1.0) * (n as f32 - 1.0) + 1.0;
        let shown = (steps.floor() as usize).clamp(1, n);
        let ink = if shown == n { 1.0 } else { steps.fract() };
        let first = n - shown;
        let newest = first.saturating_sub(1);
        let lo = if ink > 0.0 { newest } else { first };

        // Zoom so the squares on show fill the height (and fit the width).
        let (mut bx0, mut by0, mut bx1, mut by1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for sq in &squares[lo..] {
            bx0 = bx0.min(sq.x);
            by0 = by0.min(sq.y);
            bx1 = bx1.max(sq.x + sq.s);
            by1 = by1.max(sq.y + sq.s);
        }
        let k = ((dh as f32 - 1.0) / (by1 - by0))
            .min((dw as f32 - 1.0) / (bx1 - bx0))
            .max(0.1);
        let ox = dw as f32 / 2.0 - (bx0 + bx1) / 2.0 * k;
        let oy = (dh as f32 - 1.0) / 2.0 - (by0 + by1) / 2.0 * k;
        let corners = |sq: &FibSquare| {
            let x0 = (ox + sq.x * k).round() as i32;
            let y0 = (oy + sq.y * k).round() as i32;
            let x1 = (ox + (sq.x + sq.s) * k).round() as i32;
            let y1 = (oy + (sq.y + sq.s) * k).round() as i32;
            [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
        };
        for sq in &squares[first..] {
            let c = corners(sq);
            for e in 0..4 {
                bresenham(grid, c[e].0, c[e].1, c[(e + 1) % 4].0, c[(e + 1) % 4].1);
            }
        }
        // The newest square is inked side by side, its pen point marching.
        if ink > 0.0 && first > 0 {
            let c = corners(&squares[newest]);
            let sides = ink * 4.0;
            for e in 0..4 {
                let f = (sides - e as f32).clamp(0.0, 1.0);
                if f <= 0.0 {
                    break;
                }
                let (ax, ay) = c[e];
                let (bx, by) = c[(e + 1) % 4];
                let ex = ax + ((bx - ax) as f32 * f).round() as i32;
                let ey = ay + ((by - ay) as f32 * f).round() as i32;
                bresenham(grid, ax, ay, ex, ey);
                if f < 1.0 && (ctx.time * 6.0).sin() > 0.0 {
                    draw::dot_i(grid, ex + 1, ey);
                    draw::dot_i(grid, ex, ey + 1);
                    draw::dot_i(grid, ex + 1, ey + 1);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Pentagram — nested pentagram-in-pentagon (the infinite pentagram)
// ─────────────────────────────────────────────────────────────────────────────
//
// Each diagonal of a regular pentagon is φ times the side.  Inside any
// pentagram lives a smaller regular pentagon (scaled by 1/φ²), which in turn
// holds another pentagram. `ctx.eased` controls nesting depth; `ctx.time`
// rotates the stack.

struct Pentagram;
impl ProgressStyle for Pentagram {
    fn name(&self) -> &str {
        "pentagram"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Upright pentagram {5/2} in its pentagon (the next star nests in the inner \
         pentagon, scaled 1/φ², once there is room), riding a rail with progress while a glint \
         runs the star's single stroke"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        // Full-height star; the pentagon's bottom edge sits at r·cos 36°.
        let r = (dh as f32 / 2.0 - 0.5).max(1.0);
        let cy = r + 0.5;
        let span = (dw as f32 - 2.0 * r - 2.0).max(0.0);
        let cx = r + 1.0 + ctx.eased.clamp(0.0, 1.0) * span;
        let rail_y = (cy + r * 0.3).round() as i32;
        let mut x = 0;
        let rail_end = (cx - r - 1.0).ceil() as i32;
        while x < rail_end {
            draw::dot_i(grid, x, rail_y);
            x += 2;
        }

        let up = PI / 2.0;
        let verts = ngon_vertices(5, cx, cy, r, up);
        let star_order = [0usize, 2, 4, 1, 3, 0];
        for k in 0..5 {
            let (x0, y0) = verts[star_order[k]];
            let (x1, y1) = verts[star_order[k + 1]];
            bresenham(grid, x0, y0, x1, y1);
        }
        ngon_outline(grid, 5, cx, cy, r, up);
        // The star's inner pentagon points down; the next star nests in it.
        let inner = r / (PHI * PHI);
        if inner >= 4.0 {
            let iv = ngon_vertices(5, cx, cy, inner, -up);
            for k in 0..5 {
                let (x0, y0) = iv[star_order[k]];
                let (x1, y1) = iv[star_order[k + 1]];
                bresenham(grid, x0, y0, x1, y1);
            }
        }

        // Glint running the single unbroken stroke of the outer star.
        let run = (ctx.time * 0.5).fract() * 5.0;
        let k = (run as usize).min(4);
        let f = run - k as f32;
        let (x0, y0) = verts[star_order[k]];
        let (x1, y1) = verts[star_order[k + 1]];
        let gx = x0 + ((x1 - x0) as f32 * f).round() as i32;
        let gy = y0 + ((y1 - y0) as f32 * f).round() as i32;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            draw::dot_i(grid, gx + dx, gy + dy);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Pentagon Nest — concentric pentagons each scaled by 1/φ², rotating
// ─────────────────────────────────────────────────────────────────────────────
//
// Purely the concentric pentagons without the star chords.  Each ring slowly
// counter-rotates against `ctx.time`, and the nest grows as `ctx.eased` rises.

struct PentagonNest;
impl ProgressStyle for PentagonNest {
    fn name(&self) -> &str {
        "pentagon-nest"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Concentric pentagons each scaled 1/φ² smaller, counter-rotating with time — \
         a hypnotic tunnel of five-fold symmetry"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let base_r = fit_scale(dw, dh) * 0.95;

        let max_rings: usize = 8;
        let rings = ((ctx.eased * max_rings as f32).ceil() as usize)
            .min(max_rings)
            .max(1);
        let inner_scale = 1.0 / (PHI * PHI);

        let mut r = base_r;
        for i in 0..rings {
            if r < 1.0 {
                break;
            }
            // Alternate rotation direction per ring.
            let sign = if i % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
            let rot = ctx.time * 0.15 * sign - PI / 2.0 + i as f32 * 0.1;
            ngon_outline(grid, 5, cx, cy, r, rot);
            r *= inner_scale;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Golden Gnomon — 36-72-72 golden triangle recursive subdivision
// ─────────────────────────────────────────────────────────────────────────────
//
// The golden gnomon is the 36-72-72 isoceles triangle where the ratio of the
// long side to the short side equals φ. Subdividing the base triangle with a
// bisector from the apex yields a smaller golden gnomon + a golden triangle
// (36-36-108). Revealed with `ctx.eased` depth.

struct GoldenGnomon;
impl ProgressStyle for GoldenGnomon {
    fn name(&self) -> &str {
        "golden-gnomon"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Golden triangle and gnomon: a frieze of 36-72-72 golden triangles, each cut \
         into a 36-36-108 gnomon and a smaller golden triangle, which is cut again \
         and again so the pieces whirl inward in a spiral; turns added with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        // A frieze of gnomons: full-height 36-72-72 triangles shoulder to
        // shoulder across the whole width, with inverted copies between their
        // apexes (each is the reflection of its neighbour, so the strip tiles
        // exactly). Every triangle is recursively bisected to `eased` depth.
        let sin36 = 36.0_f32.to_radians().sin();
        let cos36 = 36.0_f32.to_radians().cos();
        let hf = (dh as f32 - 1.0).max(1.0);
        let side = hf / cos36; // long side spans the full canvas height
        let bw = 2.0 * sin36 * side; // gnomon base width

        // Breathing: the frieze pulses about the canvas centre (0.25 Hz —
        // seamless over the 4 s loop).
        let pulse = 1.0 + 0.035 * (ctx.time * PI * 0.5).sin();
        let (mx, my) = (dw as f32 / 2.0, dh as f32 / 2.0);

        let max_depth: usize = 5;
        let depth = (2 + (ctx.eased.clamp(0.0, 1.0) * (max_depth - 2) as f32).round() as usize)
            .min(max_depth);

        type Pt = (f32, f32);
        fn lerp_pt(a: Pt, b: Pt, t: f32) -> Pt {
            (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
        }
        let draw_tri = |grid: &mut BrailleGrid, a: Pt, b: Pt, c: Pt| {
            let to_dot = |p: Pt| {
                (
                    (mx + (p.0 - mx) * pulse).round() as i32,
                    (my + (p.1 - my) * pulse).round() as i32,
                )
            };
            let (ax, ay) = to_dot(a);
            let (bx, by) = to_dot(b);
            let (ccx, ccy) = to_dot(c);
            bresenham(grid, ax, ay, bx, by);
            bresenham(grid, bx, by, ccx, ccy);
            bresenham(grid, ccx, ccy, ax, ay);
        };

        // Root frieze: alternating upright / inverted gnomons.
        // is_gnomon = true: 36-72-72 gnomon; false: golden triangle (36-36-108).
        let n_up = ((dw as f32 / bw).ceil() as usize + 1).min(16);
        let mut tris: Vec<(Pt, Pt, Pt, bool)> = Vec::new();
        // Leading inverted gnomon so the top-left corner is tiled too.
        tris.push(((0.0, hf), (-bw / 2.0, 0.0), (bw / 2.0, 0.0), true));
        for k in 0..n_up {
            let x0 = k as f32 * bw;
            // Upright: apex on the top edge, base along the bottom.
            tris.push(((x0 + bw / 2.0, 0.0), (x0, hf), (x0 + bw, hf), true));
            // Inverted: apex on the bottom edge between neighbouring apexes.
            tris.push((
                (x0 + bw, hf),
                (x0 + bw / 2.0, 0.0),
                (x0 + 1.5 * bw, 0.0),
                true,
            ));
        }
        for &(a, b, c, _) in &tris {
            draw_tri(grid, a, b, c);
        }

        // Subdivision:
        //   Gnomon (A=apex 36°, B, C base): place P on AB at AP = BC = 1/PHI * AB.
        //     → gnomon(P, A, C) [rotated/reflected] + golden-tri(B, P, C).
        //   Golden-tri (A=apex 108°, B, C base): place P on AB at AP = 1/PHI.
        //     → gnomon(C, P, A) + golden-tri(B, P, C).
        // Only the upright triangles carry a spiral (the inverted ones stay
        // whole so neighbouring whirls don't run together).
        let mut tris: Vec<(Pt, Pt, Pt, bool)> = tris
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 1)
            .map(|(_, t)| t)
            .collect();
        // Only the newest 36-72-72 piece is cut again at each step, so each
        // upright triangle fills with a whirl of ever-smaller triangles
        // spiralling in towards its pole instead of a uniform mesh.
        for _d in 0..depth.saturating_sub(1) {
            let mut next: Vec<(Pt, Pt, Pt, bool)> = Vec::new();
            for &(a, b, c, _) in &tris {
                // P on AB with AP = |AB|/φ = |BC|: triangle A-P-C is the
                // 36-36-108 gnomon (left whole) and C-B-P is a smaller
                // 36-72-72 golden triangle with its apex at C, cut next.
                let p = lerp_pt(a, b, 1.0 / PHI);
                draw_tri(grid, a, p, c);
                draw_tri(grid, c, b, p);
                next.push((c, b, p, true));
            }
            tris = next;
            if tris.len() > 512 {
                break;
            } // cap for degenerate/large grids
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. φ-Phyllotaxis (pentagonal) — five-fold seed arrangement
// ─────────────────────────────────────────────────────────────────────────────
//
// Standard phyllotaxis but with a modular constraint: only seeds whose index
// is congruent to 0, 1, 2, 3, 4 (mod 5) are drawn in five distinct angular
// sectors separated by 72°.  This produces five spiral arms and five-fold
// symmetry, unlike the usual all-seeds sunflower.

struct PhiPhyllotaxisPent;
impl ProgressStyle for PhiPhyllotaxisPent {
    fn name(&self) -> &str {
        "phi-phyllotaxis-pent"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Five-fold phyllotaxis: seeds placed at the golden angle but grouped into five \
         symmetric spiral arms, creating pentagonal lattice symmetry with φ spacing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        // Golden angle in radians.
        let golden_angle = 2.0 * PI / (PHI * PHI); // ≈ 137.508°
                                                   // ~55 seeds keeps each one a distinct dot at 16 dots tall.
        let n_max: usize = 55;
        let n_plot = ((ctx.eased * n_max as f32).round() as usize).min(n_max);
        let c = scale / (n_max as f32).sqrt();
        let rot = ctx.time * 0.12;

        // Five sectors: seed n belongs to sector (n % 5).
        // Draw seeds in each sector at their natural positions.
        // Connect adjacent seeds within each arm with short lines for extra
        // visual texture (unlike geometry.rs plain phyllotaxis which just plots dots).
        let mut sector_prev: [Option<(i32, i32)>; 5] = [None; 5];

        for n in 0..n_plot {
            let angle = n as f32 * golden_angle + rot;
            let r = c * (n as f32).sqrt();
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy - r * angle.sin()).round() as i32;
            let sector = n % 5;
            // Connect to previous point in the same sector with a short chord.
            if let Some((lx, ly)) = sector_prev[sector] {
                bresenham(grid, lx, ly, px, py);
            } else {
                draw::dot_i(grid, px, py);
            }
            sector_prev[sector] = Some((px, py));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Nautilus — chambered logarithmic spiral with cross-walls
// ─────────────────────────────────────────────────────────────────────────────
//
// True φ-logarithmic spiral r = a·φ^(θ·2/π) (one φ-factor per quarter turn).
// Cross-walls ("septa") divide the spiral into chambers every π/2 radians,
// drawn as radial line segments from the inner to the outer wall.

struct Nautilus;
impl ProgressStyle for Nautilus {
    fn name(&self) -> &str {
        "nautilus"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "φ-logarithmic nautilus shell: three whorls, each ~2× the last, traced from the \
         outer lip inward with progress, chamber walls every 45° behind the open body chamber"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let (cx, cy) = center(dw, dh);
        let cy = cy - 0.5;
        // Braille dots are square: no squash, the outer whorl fills the height.
        let scale = (dh as f32 / 2.0 - 0.5).max(2.0);

        // Spiral: r(θ) = φ^(k·(θ−θ_total)) — three turns, each whorl ~2× the
        // last.  The shell is traced from the outer lip inward as progress
        // rises, so even early on the big body chamber reads as a shell.
        let theta_total = 6.0 * PI;
        let theta_min = theta_total * (1.0 - (0.3 + 0.7 * ctx.eased.clamp(0.0, 1.0)));
        let rot = ctx.time * 0.15;
        let k = 0.75 / PI;
        let at = |theta: f32| -> (i32, i32) {
            let r = PHI.powf(k * (theta - theta_total)) * scale;
            let angle = theta + rot;
            (
                (cx + r * angle.cos()).round() as i32,
                (cy - r * angle.sin()).round() as i32,
            )
        };

        let steps = (((theta_total - theta_min) / (2.0 * PI) * 90.0).round() as usize).max(2);
        for i in 0..=steps {
            let theta = theta_min + i as f32 / steps as f32 * (theta_total - theta_min);
            let (px, py) = at(theta);
            draw::dot_i(grid, px, py);
        }

        // Septa (chamber walls) every 45°, from each whorl to the one inside
        // it; the newest chamber (the body chamber at the lip) stays open.
        for s in 0..24 {
            let theta_wall = s as f32 * PI / 4.0;
            if theta_wall < theta_min.max(2.0 * PI) || theta_wall > theta_total - PI / 2.0 {
                continue;
            }
            let (xo, yo) = at(theta_wall);
            let (xi, yi) = at(theta_wall - 2.0 * PI);
            bresenham(grid, xi, yi, xo, yo);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Golden-Angle Rays — successive rays at 137.5° increments
// ─────────────────────────────────────────────────────────────────────────────
//
// Rays emanate from the center at successive multiples of the golden angle
// (137.508° ≈ 2π/φ²). Each ray is longer than the last by factor φ^(1/n),
// so the whole pattern never repeats and fills space evenly.

struct GoldenAngleRays;
impl ProgressStyle for GoldenAngleRays {
    fn name(&self) -> &str {
        "golden-angle-rays"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "21 rays at successive 137.5° golden-angle increments around a hub, each \
         growing longer — the irrational spacing that prevents clustering and produces \
         optimal coverage of the disk"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cx, cy) = center(dw, dh);
        let scale = fit_scale(dw, dh);

        let golden_angle = 2.0 * PI / (PHI * PHI); // ≈ 137.508°
                                                   // 21 = F₈: at 16 dots tall more rays merge into a solid disc.
        let n_max: usize = 21;
        let n_rays = ((ctx.eased * n_max as f32).round() as usize).min(n_max);
        let rot = ctx.time * 0.1;

        for i in 0..n_rays {
            let angle = i as f32 * golden_angle + rot;
            // Ray length grows with index, bounded by scale.
            let t = (i + 1) as f32 / n_max as f32;
            let len = scale * t; // linear growth for clarity
            let x_end = (cx + len * angle.cos()).round() as i32;
            let y_end = (cy - len * angle.sin()).round() as i32;
            // Rays start off a small hub so the centre stays open.
            let hub = scale * 0.18;
            let x0 = (cx + hub * angle.cos()).round() as i32;
            let y0 = (cy - hub * angle.sin()).round() as i32;
            bresenham(grid, x0, y0, x_end, y_end);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Dodecagram — 12-pointed φ-star via string-art on a 12-gon
// ─────────────────────────────────────────────────────────────────────────────
//
// Connect vertex i to vertex (i + 5) mod 12 on a regular 12-gon.  This
// produces a {12/5} dodecagram.  The ratio of chord to side of the 12-gon
// contains φ.  Chords appear one by one with progress.

struct Dodecagram;
impl ProgressStyle for Dodecagram {
    fn name(&self) -> &str {
        "dodecagram"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Dodecagram {12/5}: a frieze of twelve-pointed stars (every 5th vertex of a \
         12-gon joined, drawn as the star's silhouette); each star is traced point by \
         point along the strip with progress and turns slowly, alternate stars \
         counter-rotating"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let n: usize = 12;
        let step: usize = 5; // {12/5} star polygon
        let panels = (dw / 18).max(1);
        let pw = dw as f32 / panels as f32;
        let r = (dh as f32 * 0.5 - 0.5).min(pw * 0.5 - 0.5).max(1.0);
        let cy = (dh as f32 - 1.0) * 0.5;
        // Always show a little of the first star.
        let total = (0.3 + ctx.eased.clamp(0.0, 1.0) * (panels as f32 - 0.3)).min(panels as f32);
        for k in 0..panels {
            let cx = pw * (k as f32 + 0.5) - 0.5;
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                // Unbuilt star: its twelve vertices only.
                for (x, y) in ngon_vertices(n, cx, cy, r, PI / 2.0) {
                    draw::dot_i(grid, x, y);
                }
                continue;
            }
            let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
            let rot = PI / 2.0 + dir * ctx.time * 0.25;
            // Silhouette of {12/5}: tips on the circumcircle, notches where
            // neighbouring chords cross, at r·cos(75°)/cos(60°) ≈ 0.518 r.
            let nf = n as f32;
            let inner = r * (step as f32 * PI / nf).cos() / ((step - 1) as f32 * PI / nf).cos();
            let outline: Vec<(f32, f32)> = (0..2 * n)
                .map(|i| {
                    let t = rot + i as f32 * PI / n as f32;
                    let rr = if i % 2 == 0 { r } else { inner };
                    (cx + rr * t.cos(), cy - rr * t.sin())
                })
                .collect();
            partial_poly(grid, &outline, local);
            // Inner ring of the star: the 12-gon the notches sit on.
            if local >= 1.0 {
                let core = inner * 0.55 * (1.0 + 0.15 * (ctx.time * 2.0 + k as f32).sin());
                ngon_outline(grid, n, cx, cy, core, rot);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Kepler Triangle — right triangle with sides 1 : √φ : φ
// ─────────────────────────────────────────────────────────────────────────────
//
// The Kepler triangle has sides in geometric progression 1, √φ, φ, so the
// squares erected on its sides have areas 1, φ and φ² — and Pythagoras'
// a² + b² = c² becomes φ² = φ + 1.  Each panel draws the triangle with the
// three squares on its sides (the classic Pythagoras windmill figure); panels
// are built left to right with progress, each one drawing the triangle, then
// the squares on the short leg, the long leg and finally the hypotenuse.  A
// bead runs round each finished triangle on `ctx.time`.

/// Draw a fraction `f` (0..=1) of the closed polygon `pts` (dot-space floats).
fn partial_poly(grid: &mut BrailleGrid, pts: &[(f32, f32)], f: f32) {
    let n = pts.len();
    if n < 2 || f <= 0.0 {
        return;
    }
    let f = f.min(1.0);
    let lens: Vec<f32> = (0..n)
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
        })
        .collect();
    let total: f32 = lens.iter().sum();
    let mut left = total * f;
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let g = if lens[i] <= 0.0 {
            1.0
        } else {
            (left / lens[i]).min(1.0)
        };
        let e = (a.0 + (b.0 - a.0) * g, a.1 + (b.1 - a.1) * g);
        bresenham(
            grid,
            a.0.round() as i32,
            a.1.round() as i32,
            e.0.round() as i32,
            e.1.round() as i32,
        );
        left -= lens[i];
        if left <= 0.0 {
            break;
        }
    }
}

struct KeplerTriangle;
impl ProgressStyle for KeplerTriangle {
    fn name(&self) -> &str {
        "kepler-triangle"
    }
    fn theme(&self) -> &str {
        "goldenratio"
    }
    fn describe(&self) -> &str {
        "Kepler triangle: the right triangle with sides 1:√φ:φ and the squares on its \
         sides (areas 1, φ, φ² — Pythagoras becomes φ² = φ + 1), built panel by panel \
         along the strip"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let a = 1.0f32; // short leg (vertical)
        let b = PHI.sqrt(); // long leg (horizontal)
        let panels = (dw / 20).max(1);
        let pw = dw as f32 / panels as f32;
        // Figure extents in units: x ∈ [-a, a+b], y ∈ [-b, a+b].
        let u = ((dh as f32 - 1.0) / (a + 2.0 * b))
            .min((pw - 2.0) / (2.0 * a + b))
            .max(0.5);
        let oy = (dh as f32 - 1.0) * 0.5 + (a * 0.5) * u;
        let total = (0.3 + ctx.eased.clamp(0.0, 1.0) * (panels as f32 - 0.3)).min(panels as f32);
        for k in 0..panels {
            let local = (total - k as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let ox = pw * (k as f32 + 0.5) - 0.5 - (b * 0.5) * u;
            let m = |x: f32, y: f32| (ox + x * u, oy - y * u);
            let tri = [m(0.0, 0.0), m(b, 0.0), m(0.0, a)];
            let sq_a = [m(0.0, 0.0), m(0.0, a), m(-a, a), m(-a, 0.0)];
            let sq_b = [m(0.0, 0.0), m(0.0, -b), m(b, -b), m(b, 0.0)];
            let sq_c = [m(b, 0.0), m(b + a, b), m(a, a + b), m(0.0, a)];
            let seg = |i: usize| ((local * 4.0) - i as f32).clamp(0.0, 1.0);
            partial_poly(grid, &tri, seg(0));
            partial_poly(grid, &sq_a, seg(1));
            partial_poly(grid, &sq_b, seg(2));
            partial_poly(grid, &sq_c, seg(3));
            if local >= 1.0 {
                // Bead circling the triangle.
                let s = (ctx.time * 0.5 + k as f32 * 0.37).fract();
                let lens = [b, PHI, a];
                let mut d = s * (a + b + PHI);
                for i in 0..3 {
                    if d <= lens[i] || i == 2 {
                        let (p, q) = (tri[i], tri[(i + 1) % 3]);
                        let g = (d / lens[i]).clamp(0.0, 1.0);
                        let x = (p.0 + (q.0 - p.0) * g).round() as i32;
                        let y = (p.1 + (q.1 - p.1) * g).round() as i32;
                        for (ddx, ddy) in [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)] {
                            draw::dot_i(grid, x + ddx, y + ddy);
                        }
                        break;
                    }
                    d -= lens[i];
                }
            }
        }
        Ok(())
    }
}
