//! Topology / parametric-surface / 3D-projection progress bars.
//!
//! Most bars show one solid-looking object that slides along a wire through
//! the bar's middle — the wire solid behind it (done) and dotted ahead (to
//! go) — so `ctx.eased` is its position, while `ctx.time` turns it in 3-D.
//! A small perspective camera plus a per-dot depth buffer gives hidden-line
//! surfaces, silhouettes and true over/under knot crossings.  The helix grows
//! along the bar instead; the last three styles still use the older
//! orthographic reveal helper.
//!
//! All surfaces are rendered in **dot space** via `draw::dot_i`, so out-of-
//! bounds plots are silently discarded. Per-frame point counts are bounded to
//! keep even wide grids snappy.
//!
//! # Styles
//!
//! | Name | Surface |
//! |---|---|
//! | `mobius-strip` | Half-twisted Möbius band, hidden lines removed, slides along a wire |
//! | `torus-wireframe` | Perspective doughnut, hidden-line mesh, spins as it slides |
//! | `torus-knot` | (2,5) torus knot with over/under crossings |
//! | `trefoil-knot` | Trefoil knot with over/under crossings, rocking |
//! | `klein-bottle` | Glass Klein bottle, neck through the wall (x-ray dotted) |
//! | `hopf-fibers` | Three mutually linked Hopf fibers (Villarceau circles) |
//! | `sphere-inflate` | Spinning lat/long globe inflating as it rolls along |
//! | `helix-climb` | DNA double helix with rungs, growing along the bar |
//! | `saddle-surface` | Hyperbolic paraboloid (saddle) wireframe |
//! | `tesseract-spin` | Rotating 4-D hypercube projected to 2-D |
//! | `roman-surface` | Steiner's Roman surface parametric |
//! | `seifert-ramp` | Spiral Seifert-surface ramp |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── Shared 3-D projection helper ─────────────────────────────────────────────

/// Euler angles `(ax, ay)`, dot-space centre `(cx, cy)` and uniform `scale` in
/// dots-per-unit — the camera every projected point is pushed through.
#[derive(Clone, Copy)]
struct View {
    ax: f32,
    ay: f32,
    cx: i32,
    cy: i32,
    scale: f32,
}

/// Rotate `(x, y, z)` about the X-axis by `view.ax` radians then the Y-axis by
/// `view.ay` radians (standard Euler XY extrinsic), then orthographically project
/// the result onto the dot lattice centred at `(view.cx, view.cy)` with uniform
/// `view.scale` dots-per-unit.
///
/// Returns `(sx, sy)` as `i32` — suitable for `draw::dot_i`.
#[inline]
fn project(x: f32, y: f32, z: f32, view: &View) -> (i32, i32) {
    // Rotate about X axis.
    let (sax, cax) = view.ax.sin_cos();
    let y1 = y * cax - z * sax;
    let z1 = y * sax + z * cax;
    // Rotate about Y axis.
    let (say, cay) = view.ay.sin_cos();
    let x2 = x * cay + z1 * say;
    let y2 = y1;
    // Orthographic projection (drop z2, flip y for screen coords).
    let sx = view.cx + (x2 * view.scale).round() as i32;
    let sy = view.cy - (y2 * view.scale).round() as i32;
    (sx, sy)
}

/// Plot a parametric curve segment from `t0` to `t1` (in `[0, 2π]` or
/// `[0, 1]`), sampling `steps` evenly-spaced points, using `f(t) -> (x,y,z)`.
/// Each point is rotated and projected through `view`.
#[inline]
fn plot_curve<F>(grid: &mut BrailleGrid, t0: f32, t1: f32, steps: usize, view: &View, f: F)
where
    F: Fn(f32) -> (f32, f32, f32),
{
    if steps == 0 {
        return;
    }
    for i in 0..=steps {
        let t = t0 + (t1 - t0) * (i as f32 / steps as f32);
        let (x, y, z) = f(t);
        let (sx, sy) = project(x, y, z, view);
        draw::dot_i(grid, sx, sy);
    }
}

// ── Shared geometry: grid centre + uniform scale ──────────────────────────────

/// Return `(cx_i32, cy_i32, scale)` for a grid: centre in dot coords, and
/// scale chosen so a unit sphere just fits the smaller axis.
fn grid_cxys(grid: &BrailleGrid) -> (i32, i32, f32) {
    let (dw, dh) = draw::dot_dims(grid);
    let cx = (dw / 2) as i32;
    let cy = (dh / 2) as i32;
    let scale = (dw.min(dh * 2) as f32 * 0.42).max(1.0);
    (cx, cy, scale)
}

// ── Depth-tested perspective engine (styles 1–6) ─────────────────────────────

/// Camera for the solid-looking styles: spin `az` about the object's own Z
/// axis, tilt `ax` about X, turn `ay` about Y, then a mild perspective divide.
/// `stretch` widens the screen x-axis so small objects use the wide bar.
#[derive(Clone, Copy)]
struct Cam {
    ax: f32,
    ay: f32,
    az: f32,
    cx: f32,
    cy: f32,
    scale: f32,
    stretch: f32,
}

impl Cam {
    /// Project to `(sx, sy, depth)` — larger depth is nearer the viewer.
    fn proj(&self, p: (f32, f32, f32)) -> (i32, i32, f32) {
        let (x, y, z) = self.rot(p);
        let k = 6.0 / (6.0 - z.clamp(-3.0, 3.0));
        (
            (self.cx + x * self.scale * k * self.stretch).round() as i32,
            (self.cy - y * self.scale * k).round() as i32,
            z,
        )
    }
    /// Rotate a point / direction into view space (no projection).
    fn rot(&self, p: (f32, f32, f32)) -> (f32, f32, f32) {
        let (x, y, z) = p;
        let (s, c) = self.az.sin_cos();
        let (x, y) = (x * c - y * s, x * s + y * c);
        let (s, c) = self.ax.sin_cos();
        let (y, z) = (y * c - z * s, y * s + z * c);
        let (s, c) = self.ay.sin_cos();
        let (x, z) = (x * c + z * s, -x * s + z * c);
        (x, y, z)
    }
}

/// Rasterise surface `f(u, v)` over `[0, u.0] × [0, v.0]` (with `u.1 × v.1`
/// samples) into a depth buffer, and draw its visible silhouette — the points
/// where the surface turns edge-on to the viewer.  Returns the buffer so the
/// caller can hidden-line its own wires against it.
fn surface<F>(
    grid: &mut BrailleGrid,
    cam: &Cam,
    u: (f32, usize),
    v: (f32, usize),
    eps: f32,
    f: F,
) -> ZBuf
where
    F: Fn(f32, f32) -> (f32, f32, f32),
{
    let mut zb = ZBuf::new(grid);
    let (du, dv) = (u.0 / u.1.max(1) as f32, v.0 / v.1.max(1) as f32);
    let mut rim = Vec::new();
    for i in 0..=u.1 {
        for j in 0..=v.1 {
            let (uu, vv) = (i as f32 * du, j as f32 * dv);
            let p = f(uu, vv);
            let (x, y, z) = cam.proj(p);
            zb.splat(x, y, z, 1);
            // Normal by finite differences, rotated into view space.
            let a = f(uu + du * 0.5, vv);
            let b = f(uu, vv + dv * 0.5);
            let a = cam.rot((a.0 - p.0, a.1 - p.1, a.2 - p.2));
            let b = cam.rot((b.0 - p.0, b.1 - p.1, b.2 - p.2));
            let n = (
                a.1 * b.2 - a.2 * b.1,
                a.2 * b.0 - a.0 * b.2,
                a.0 * b.1 - a.1 * b.0,
            );
            let len = (n.0 * n.0 + n.1 * n.1 + n.2 * n.2).sqrt();
            if len > 1e-9 && (n.2 / len).abs() < 0.22 {
                rim.push((x, y, z));
            }
        }
    }
    for (x, y, z) in rim {
        if zb.front(x, y, z, eps) {
            draw::dot_i(grid, x, y);
        }
    }
    zb
}

/// Per-dot depth buffer (nearest z wins) for hidden-line removal.
struct ZBuf {
    w: i32,
    h: i32,
    z: Vec<f32>,
}

impl ZBuf {
    fn new(grid: &BrailleGrid) -> Self {
        let (dw, dh) = draw::dot_dims(grid);
        Self {
            w: dw as i32,
            h: dh as i32,
            z: vec![f32::NEG_INFINITY; dw * dh],
        }
    }
    /// Record depth `z` at `(x, y)` and its `r`-dot neighbourhood.
    fn splat(&mut self, x: i32, y: i32, z: f32, r: i32) {
        for yy in y - r..=y + r {
            for xx in x - r..=x + r {
                if xx >= 0 && yy >= 0 && xx < self.w && yy < self.h {
                    let i = (yy * self.w + xx) as usize;
                    if z > self.z[i] {
                        self.z[i] = z;
                    }
                }
            }
        }
    }
    /// `true` if a point at depth `z` is not behind anything (within `eps`).
    fn front(&self, x: i32, y: i32, z: f32, eps: f32) -> bool {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return false;
        }
        z + eps >= self.z[(y * self.w + x) as usize]
    }
}

/// Sample closed curve `f` over `[0, 2π)` into projected points.
fn sample_loop<F>(cam: &Cam, steps: usize, f: F) -> Vec<(i32, i32, f32)>
where
    F: Fn(f32) -> (f32, f32, f32),
{
    let n = steps.max(1);
    (0..n)
        .map(|i| cam.proj(f(i as f32 / n as f32 * 2.0 * PI)))
        .collect()
}

/// Draw projected curve points as a knot diagram: where two strands cross on
/// screen the farther one is broken, giving true over/under crossings.
fn draw_crossings(grid: &mut BrailleGrid, pts: &[(i32, i32, f32)], eps: f32) {
    let mut zb = ZBuf::new(grid);
    for &(x, y, z) in pts {
        zb.splat(x, y, z, 1);
    }
    for &(x, y, z) in pts {
        if zb.front(x, y, z, eps) {
            draw::dot_i(grid, x, y);
        }
    }
}

/// Lay out an object that slides along a wire through the bar's middle:
/// solid behind it (done), dotted ahead (to go).  `ex`/`ey` are its projected
/// half-width / half-height in object units (before `stretch`).  Returns the
/// camera centre and scale.
fn travel(grid: &mut BrailleGrid, ctx: &BarContext, ex: f32, ey: f32) -> (f32, f32, f32) {
    let (dw, dh) = draw::dot_dims(grid);
    let scale = ((dh as f32 - 2.0) / (2.0 * ey * 1.12)).max(0.3);
    let half = (ex * scale * 1.12).min(dw as f32 / 2.0);
    let cx = half + ctx.eased * (dw as f32 - 2.0 * half).max(0.0);
    let cy = (dh / 2) as f32;
    let y = dh / 2;
    for x in 0..dw {
        let fx = x as f32;
        if fx < cx - half - 1.0 || (fx > cx + half + 1.0 && x % 3 == 0) {
            draw::dot(grid, x, y);
        }
    }
    (cx, cy, scale)
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Möbius strip
// ─────────────────────────────────────────────────────────────────────────────

struct MobiusStrip;

impl ProgressStyle for MobiusStrip {
    fn name(&self) -> &str {
        "mobius-strip"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A one-sided Möbius band with its half-twist, hidden lines removed, cross-rulings flowing round it as it slides along the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // Band centreline radius 1, half-width 0.7, x stretched ×1.7.
        let (cx, cy, scale) = travel(grid, ctx, 1.7 * 1.7, 1.5);
        let cam = Cam {
            ax: 0.95,
            ay: 0.0,
            az: 0.2 * (ctx.time * 0.8).sin(),
            cx,
            cy,
            scale,
            stretch: 1.0,
        };
        let band = |u: f32, v: f32| {
            let r = 1.0 + 0.7 * v * (u / 2.0).cos();
            (1.7 * r * u.cos(), r * u.sin(), 0.7 * v * (u / 2.0).sin())
        };
        let eps = 1.5 / scale;
        let zb = surface(grid, &cam, (2.0 * PI, 120), (2.0, 8), eps, |u, v| {
            band(u, v - 1.0)
        });
        let mut plot = |p: (i32, i32, f32)| {
            if zb.front(p.0, p.1, p.2, eps) {
                draw::dot_i(grid, p.0, p.1);
            }
        };
        // The single boundary edge (v = ±1 is one curve going round twice).
        for i in 0..240 {
            let u = i as f32 / 240.0 * 4.0 * PI;
            plot(cam.proj(band(u, 1.0)));
        }
        // Rulings across the band, drifting round with time.
        for k in 0..10 {
            let u = (k as f32 + ctx.time * 0.8) / 10.0 * 2.0 * PI;
            for j in 0..=6 {
                plot(cam.proj(band(u, -1.0 + j as f32 / 3.0)));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Torus wireframe
// ─────────────────────────────────────────────────────────────────────────────

struct TorusWireframe;

impl ProgressStyle for TorusWireframe {
    fn name(&self) -> &str {
        "torus-wireframe"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A doughnut torus in perspective, hidden-line wireframe spinning about its axis as it rolls along the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (big, small) = (1.0f32, 0.45f32);
        let tilt = 0.8f32;
        let ey = (big + small) * tilt.cos() + small * tilt.sin();
        let (cx, cy, scale) = travel(grid, ctx, (big + small) * 1.3, ey);
        let cam = Cam {
            ax: tilt,
            ay: 0.15 * (ctx.time * 0.7).sin(),
            az: ctx.time * 0.9,
            cx,
            cy,
            scale,
            stretch: 1.3,
        };
        let torus = |u: f32, v: f32| {
            let r = big + small * v.cos();
            (r * u.cos(), r * u.sin(), small * v.sin())
        };
        let eps = 1.5 / scale;
        let _ = surface(grid, &cam, (2.0 * PI, 90), (2.0 * PI, 36), eps, torus);
        // Wires are back-face culled with the analytic surface normal.
        let mut plot = |u: f32, v: f32| {
            let n = cam.rot((v.cos() * u.cos(), v.cos() * u.sin(), v.sin()));
            if n.2 > -0.05 {
                let p = cam.proj(torus(u, v));
                draw::dot_i(grid, p.0, p.1);
            }
        };
        // Meridians (tube cross-sections); the silhouette gives the outline.
        for k in 0..8 {
            let u = k as f32 / 8.0 * 2.0 * PI;
            for j in 0..32 {
                plot(u, j as f32 / 32.0 * 2.0 * PI);
            }
        }
        // Outer equator and the inner rim of the hole.
        for &v in &[0.0, PI] {
            for i in 0..140 {
                plot(i as f32 / 140.0 * 2.0 * PI, v);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Torus knot (2,5)
// ─────────────────────────────────────────────────────────────────────────────

struct TorusKnot;

impl ProgressStyle for TorusKnot {
    fn name(&self) -> &str {
        "torus-knot"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A five-lobed (2,5) torus knot with true over/under crossings, turning slowly as it slides along the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = travel(grid, ctx, 3.0 * 1.6, 3.0);
        let cam = Cam {
            ax: 0.35 * (ctx.time * 0.6).sin(),
            ay: 0.3 * (ctx.time * 0.45).cos(),
            az: ctx.time * 0.5,
            cx,
            cy,
            scale,
            stretch: 1.6,
        };
        let pts = sample_loop(&cam, 520, |t| {
            let r = 2.0 + (5.0 * t).cos();
            (r * (2.0 * t).cos(), r * (2.0 * t).sin(), -(5.0 * t).sin())
        });
        draw_crossings(grid, &pts, 1.5 / scale);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Trefoil knot
// ─────────────────────────────────────────────────────────────────────────────

struct TrefoilKnot;

impl ProgressStyle for TrefoilKnot {
    fn name(&self) -> &str {
        "trefoil-knot"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "The three-lobed trefoil knot with alternating over/under crossings, rocking in 3-D as it slides along the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = travel(grid, ctx, 3.0 * 1.6, 3.0);
        let cam = Cam {
            ax: 0.4 * (ctx.time * 0.7).sin(),
            ay: 0.4 * (ctx.time * 0.5).cos(),
            az: ctx.time * 0.4,
            cx,
            cy,
            scale,
            stretch: 1.6,
        };
        let pts = sample_loop(&cam, 420, |t| {
            (
                t.sin() + 2.0 * (2.0 * t).sin(),
                t.cos() - 2.0 * (2.0 * t).cos(),
                -(3.0 * t).sin(),
            )
        });
        draw_crossings(grid, &pts, 1.5 / scale);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Klein bottle (bottle immersion)
// ─────────────────────────────────────────────────────────────────────────────

struct KleinBottle;

/// The classic glass-bottle immersion of the Klein bottle, `u ∈ [0, π]`
/// along the bottle, `v ∈ [0, 2π]` round the tube.
fn klein(u: f32, v: f32) -> (f32, f32, f32) {
    let (su, cu) = u.sin_cos();
    let (sv, cv) = v.sin_cos();
    let c2 = cu * cu;
    let c4 = c2 * c2;
    let c6 = c4 * c2;
    let x = -2.0 / 15.0
        * cu
        * (3.0 * cv - 30.0 * su + 90.0 * c4 * su - 60.0 * c6 * su + 5.0 * cu * cv * su);
    let y = -1.0 / 15.0
        * su
        * (3.0 * cv - 3.0 * c2 * cv - 48.0 * c4 * cv + 48.0 * c6 * cv - 60.0 * su
            + 5.0 * cu * cv * su
            - 5.0 * c2 * cu * cv * su
            - 80.0 * c4 * cu * cv * su
            + 80.0 * c6 * cu * cv * su);
    let z = 2.0 / 15.0 * (3.0 + 5.0 * cu * su) * sv;
    (x, y, z)
}

impl ProgressStyle for KleinBottle {
    fn name(&self) -> &str {
        "klein-bottle"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A glass Klein bottle lying on its side: the neck bends back and passes through the wall (hidden lines dotted) as it turns and slides along"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // The immersion spans roughly x ∈ [-1.6, 1.6], y ∈ [0, 4]; lay it on its
        // side (y → screen x) centred on the origin.
        let lie = |p: (f32, f32, f32)| (p.1 - 2.0, -p.0, p.2);
        let (cx, cy, scale) = travel(grid, ctx, 2.1 * 1.3, 1.6);
        let cam = Cam {
            ax: ctx.time * 0.7,
            ay: 0.2,
            az: 0.0,
            cx,
            cy,
            scale,
            stretch: 1.3,
        };
        // Swap rotation so the bottle spins about its own long (x) axis.
        let pr = |p: (f32, f32, f32)| {
            let (x, y, z) = lie(p);
            cam.proj((x, y, z))
        };
        let eps = 1.5 / scale;
        let zb = surface(grid, &cam, (PI, 72), (2.0 * PI, 24), eps, |u, v| {
            lie(klein(u, v))
        });
        let mut n = 0u32;
        let mut plot = |p: (i32, i32, f32)| {
            n = n.wrapping_add(1);
            // Visible wire solid; wire seen *through* the glass dotted.
            if zb.front(p.0, p.1, p.2, eps) || n % 5 == 0 {
                draw::dot_i(grid, p.0, p.1);
            }
        };
        // Cross-section rings along the bottle.
        for k in 0..8 {
            let u = (k as f32 + 0.5) / 8.0 * PI;
            for j in 0..28 {
                plot(pr(klein(u, j as f32 / 28.0 * 2.0 * PI)));
            }
        }
        // Long seams down the bottle.
        for k in 0..4 {
            let v = k as f32 / 4.0 * 2.0 * PI;
            for i in 0..90 {
                plot(pr(klein(i as f32 / 89.0 * PI, v)));
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Hopf fibration (linked fibers)
// ─────────────────────────────────────────────────────────────────────────────

struct HopfFibers;

impl ProgressStyle for HopfFibers {
    fn name(&self) -> &str {
        "hopf-fibers"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Three Hopf fibers — circles that each link every other — stereographically projected, interlocked with over/under crossings and turning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = travel(grid, ctx, 2.0 * 1.5, 1.9);
        let cam = Cam {
            ax: 1.0,
            ay: 0.2 * (ctx.time * 0.6).sin(),
            az: ctx.time * 0.5,
            cx,
            cy,
            scale,
            stretch: 1.5,
        };
        let eta = PI / 4.0;
        let (se, ce) = eta.sin_cos();
        let mut pts = Vec::with_capacity(3 * 160);
        for k in 0..3 {
            let a = k as f32 / 3.0 * 2.0 * PI;
            // Fiber over a base point: (cos η·e^{i(a+t)}, sin η·e^{it}) ⊂ S³,
            // projected stereographically from (0,0,0,1).
            pts.extend(sample_loop(&cam, 160, |t| {
                let (x1, x2) = (ce * (a + t).cos(), ce * (a + t).sin());
                let (x3, x4) = (se * t.cos(), se * t.sin());
                let d = (1.0 - x4).max(0.05);
                (x1 / d * 0.8, x2 / d * 0.8, x3 / d * 0.8)
            }));
        }
        draw_crossings(grid, &pts, 1.5 / scale);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Sphere wireframe (inflating)
// ─────────────────────────────────────────────────────────────────────────────

struct SphereInflate;

impl ProgressStyle for SphereInflate {
    fn name(&self) -> &str {
        "sphere-inflate"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A spinning latitude/longitude globe, back half hidden, inflating from a pea to full size as it rolls along the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = travel(grid, ctx, 1.0, 1.0);
        let rad = 0.4 + 0.6 * ctx.eased;
        let cam = Cam {
            ax: 1.2,
            ay: 0.0,
            az: ctx.time * 0.8,
            cx,
            cy,
            scale,
            stretch: 1.0,
        };
        // Only wire on the hemisphere facing the viewer is drawn.
        let mut plot = |lon: f32, lat: f32| {
            let n = (lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin());
            if cam.rot(n).2 > -0.05 {
                let p = cam.proj((n.0 * rad, n.1 * rad, n.2 * rad));
                draw::dot_i(grid, p.0, p.1);
            }
        };
        // Meridians.
        for k in 0..8 {
            let lon = k as f32 / 8.0 * 2.0 * PI;
            for j in 0..=40 {
                plot(lon, -PI / 2.0 + j as f32 / 40.0 * PI);
            }
        }
        // Parallels, equator included.
        for &lat in &[-0.9f32, -0.45, 0.0, 0.45, 0.9] {
            for i in 0..100 {
                plot(i as f32 / 100.0 * 2.0 * PI, lat);
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Helix / double helix climbing
// ─────────────────────────────────────────────────────────────────────────────

struct HelixClimb;

impl ProgressStyle for HelixClimb {
    fn name(&self) -> &str {
        "helix-climb"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "A DNA-style double helix screwing along the bar: two strands in opposite phase with base-pair rungs, the back strand broken where it passes behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let cy = (dh as f32 - 1.0) / 2.0;
        let amp = (dh as f32 - 1.0) / 2.0;
        // The helix grows from the left; a dotted axis shows what is to come.
        let reveal = (8.0 + ctx.eased * dw as f32).min(dw as f32) as usize;
        let spin = ctx.time * 3.0;
        for x in 0..dw {
            if x >= reveal {
                if x % 3 == 0 {
                    draw::dot(grid, x, dh / 2);
                }
                continue;
            }
            let ph = x as f32 * 0.28 - spin;
            let (s, c) = ph.sin_cos();
            // Strand A at +s, strand B at −s; A is in front when cos > 0.
            let ya = cy + amp * s;
            let yb = cy - amp * s;
            let (front, back) = if c > 0.0 { (ya, yb) } else { (yb, ya) };
            draw::dot_i(grid, x as i32, front.round() as i32);
            // The back strand breaks where it crosses behind the front one.
            if (front - back).abs() > 2.5 {
                draw::dot_i(grid, x as i32, back.round() as i32);
            }
            // Base-pair rungs every few dots, between the strands.
            if x % 5 == 0 && (front - back).abs() > 3.0 {
                let (lo, hi) = if ya < yb { (ya, yb) } else { (yb, ya) };
                let (lo, hi) = (lo.round() as i32 + 2, hi.round() as i32 - 2);
                for y in lo..=hi {
                    draw::dot_i(grid, x as i32, y);
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Saddle surface (hyperbolic paraboloid)
// ─────────────────────────────────────────────────────────────────────────────

struct SaddleSurface;

impl ProgressStyle for SaddleSurface {
    fn name(&self) -> &str {
        "saddle-surface"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Hyperbolic paraboloid z = x²−y² wireframe — the canonical saddle point, revealed as a grid of iso-lines, rotating with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.22 + 0.5;
        let ay = ctx.time * 0.48;
        let view = View {
            ax,
            ay,
            cx,
            cy,
            scale,
        };
        let n_lines = 12usize;
        let revealed = (ctx.eased * n_lines as f32 * 2.0).round() as usize;
        let steps = 60usize;
        // x-parallel iso-lines (vary y, constant x).
        for i in 0..revealed.min(n_lines) {
            let x = -1.0 + 2.0 * i as f32 / (n_lines - 1).max(1) as f32;
            plot_curve(grid, -1.0, 1.0, steps, &view, |y| (x, y, x * x - y * y));
        }
        // y-parallel iso-lines (vary x, constant y).
        let extra = revealed.saturating_sub(n_lines);
        for i in 0..extra.min(n_lines) {
            let y = -1.0 + 2.0 * i as f32 / (n_lines - 1).max(1) as f32;
            plot_curve(grid, -1.0, 1.0, steps, &view, |x| (x, y, x * x - y * y));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Tesseract (4-D hypercube) double projection
// ─────────────────────────────────────────────────────────────────────────────

struct TesseractSpin;

impl ProgressStyle for TesseractSpin {
    fn name(&self) -> &str {
        "tesseract-spin"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "4-D hypercube (tesseract) rotated in the XW and YZ planes then perspective-projected to 2-D — the full inner/outer cube wireframe turning inside out as it slides along the bar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        // The whole hypercube slides along the bar; its projection spans ±~0.7.
        let (fcx, fcy, scale) = travel(grid, ctx, 0.7 * 1.5, 0.7);
        let (cx, cy) = (fcx as i32, fcy as i32);

        // The 16 vertices of a unit tesseract in (x,y,z,w) ∈ {-1,+1}^4.
        let verts: [[f32; 4]; 16] = {
            let mut v = [[0.0f32; 4]; 16];
            for (i, vert) in v.iter_mut().enumerate() {
                vert[0] = if i & 1 != 0 { 1.0 } else { -1.0 };
                vert[1] = if i & 2 != 0 { 1.0 } else { -1.0 };
                vert[2] = if i & 4 != 0 { 1.0 } else { -1.0 };
                vert[3] = if i & 8 != 0 { 1.0 } else { -1.0 };
            }
            v
        };
        // The 32 edges: pairs that differ in exactly one bit.
        let mut edges: Vec<(usize, usize)> = Vec::with_capacity(32);
        for i in 0..16usize {
            for j in (i + 1)..16usize {
                if (i ^ j).is_power_of_two() {
                    edges.push((i, j));
                }
            }
        }

        // 4-D rotation angles: XW plane (time), YZ plane (time/2).
        let a_xw = ctx.time * 0.48;
        let a_yz = ctx.time * 0.31;

        // Rotate all 16 vertices.
        let project4 = |vert: [f32; 4]| -> (i32, i32) {
            let [x0, y0, z0, w0] = vert;
            // XW rotation.
            let (sxw, cxw) = a_xw.sin_cos();
            let x1 = x0 * cxw - w0 * sxw;
            let w1 = x0 * sxw + w0 * cxw;
            // YZ rotation.
            let (syz, cyz) = a_yz.sin_cos();
            let y1 = y0 * cyz - z0 * syz;
            let z1 = y0 * syz + z0 * cyz;
            // 3-D XY rotation from ctx.time (slow tumble).
            let ax = ctx.time * 0.22;
            let ay = ctx.time * 0.37;
            let (sax, cax) = ax.sin_cos();
            let y2 = y1 * cax - z1 * sax;
            let z2 = y1 * sax + z1 * cax;
            let (say, cay) = ay.sin_cos();
            let x3 = x1 * cay + z2 * say;
            let y3 = y2;
            // Perspective from w-depth.
            let w_dist = 2.5 - w1 * 0.5;
            let inv = if w_dist.abs() < 0.01 {
                0.0
            } else {
                1.0 / w_dist
            };
            let sx = cx + (x3 * inv * scale * 1.5) as i32;
            let sy = cy - (y3 * inv * scale) as i32;
            (sx, sy)
        };

        let projected: Vec<(i32, i32)> = verts.iter().map(|&v| project4(v)).collect();

        for &(a, b) in &edges {
            let (x0, y0) = projected[a];
            let (x1, y1) = projected[b];
            // Bresenham line between projected endpoints.
            let dx = (x1 - x0).abs();
            let dy = (y1 - y0).abs();
            let sx = if x0 < x1 { 1i32 } else { -1i32 };
            let sy = if y0 < y1 { 1i32 } else { -1i32 };
            let mut x = x0;
            let mut y = y0;
            let mut err = dx - dy;
            let max_steps = (dx + dy + 1).min(300);
            for _ in 0..max_steps {
                draw::dot_i(grid, x, y);
                if x == x1 && y == y1 {
                    break;
                }
                let e2 = 2 * err;
                if e2 > -dy {
                    err -= dy;
                    x += sx;
                }
                if e2 < dx {
                    err += dx;
                    y += sy;
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Roman surface (Steiner's surface)
// ─────────────────────────────────────────────────────────────────────────────

struct RomanSurface;

impl ProgressStyle for RomanSurface {
    fn name(&self) -> &str {
        "roman-surface"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Steiner's Roman surface — one of the most beautiful immersions of ℝP² into ℝ³, with six Whitney umbrella singularities, revealed petal by petal"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.27 + 0.4;
        let ay = ctx.time * 0.43;
        // Parametrisation: u ∈ [0,π], v ∈ [0,π].
        // Reveal u-strips up to eased·π.
        let u_max = ctx.eased * PI;
        let u_lines = 18usize;
        let v_steps = 60usize;
        let s = scale * 0.6;
        let view = View {
            ax,
            ay,
            cx,
            cy,
            scale: s,
        };
        for ui in 0..u_lines {
            let u = u_max * ui as f32 / u_lines.max(1) as f32;
            let (su, cu) = u.sin_cos();
            let s2u = (2.0 * u).sin();
            plot_curve(grid, 0.0, PI, v_steps, &view, |v| {
                let (sv, _cv) = v.sin_cos();
                let s2v = (2.0 * v).sin();
                // Roman surface: x=sin²u·sin 2v, y=sin 2u·sin²v, z=sin 2u·sin 2v / 2.
                let x = su * su * s2v;
                let y = s2u * sv * sv;
                let z = 0.5 * s2u * s2v;
                let _ = cu;
                (x, y, z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Seifert surface spiral ramp
// ─────────────────────────────────────────────────────────────────────────────

struct SeifertRamp;

impl ProgressStyle for SeifertRamp {
    fn name(&self) -> &str {
        "seifert-ramp"
    }
    fn theme(&self) -> &str {
        "topology"
    }
    fn describe(&self) -> &str {
        "Seifert-surface-inspired spiral ramp spanning a trefoil knot boundary — a twisted disk that rises through a full turn as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cx, cy, scale) = grid_cxys(grid);
        let ax = ctx.time * 0.30 + 0.5;
        let ay = ctx.time * 0.50;
        let view = View {
            ax,
            ay,
            cx,
            cy,
            scale,
        };
        // Seifert surface for trefoil: parametrised by (r, theta) in a disk,
        // lifted to 3-D via the trefoil fibration.
        // Approximation: r ∈ [0,1], theta ∈ [0, 2π].
        // Point: rotate by theta/3 (fibration angle), scale radially.
        // x = r·cos(theta), y = r·sin(theta), z = r·sin(theta/3)·(1-r).
        let theta_max = ctx.eased * 2.0 * PI;
        let n_radii = 8usize;
        let steps = 80usize;
        // Concentric rings.
        for ri in 1..=n_radii {
            let r = ri as f32 / n_radii as f32;
            plot_curve(grid, 0.0, theta_max, steps, &view, |theta| {
                let x = r * theta.cos();
                let y = r * theta.sin();
                let z = r * (theta / 3.0).sin() * (1.0 - r * 0.5);
                (x, y, z)
            });
        }
        // Radial spokes.
        let n_spokes = 12usize;
        for si in 0..n_spokes {
            let theta = theta_max * si as f32 / n_spokes.max(1) as f32;
            plot_curve(grid, 0.0, 1.0, 20, &view, |r| {
                let x = r * theta.cos();
                let y = r * theta.sin();
                let z = r * (theta / 3.0).sin() * (1.0 - r * 0.5);
                (x, y, z)
            });
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — green into blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(134, 232, 184);
const TINT_END: Color = Color::rgb(64, 104, 224);

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

/// All styles in the `topology` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per surface type, in the order they
/// appear in the source: Möbius, torus wireframe, torus knot, trefoil knot,
/// Klein bottle, Hopf fibers, sphere inflate, helix climb, saddle surface,
/// tesseract spin, Roman surface, Seifert ramp.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(MobiusStrip)),
        Box::new(Tinted(TorusWireframe)),
        Box::new(Tinted(TorusKnot)),
        Box::new(Tinted(TrefoilKnot)),
        Box::new(Tinted(KleinBottle)),
        Box::new(Tinted(HopfFibers)),
        Box::new(Tinted(SphereInflate)),
        Box::new(Tinted(HelixClimb)),
        Box::new(Tinted(SaddleSurface)),
        Box::new(Tinted(TesseractSpin)),
        Box::new(Tinted(RomanSurface)),
        Box::new(Tinted(SeifertRamp)),
    ]
}
