//! Fruits-themed progress bars — eleven distinct fruit styles drawn in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (for perpetual
//! animation) and `ctx.eased` (for progress-driven advancement). All writes
//! go through `draw::` helpers so every coordinate is silently bounds-safe.
//!
//! Styles in this file:
//! - `citrus-wheel`    — orange wheels filling segment by segment
//! - `apple-bite`      — row of red apples eaten bite by bite down to cores
//! - `banana-peel`     — curved banana peeled from the tip, flaps dangling
//! - `grape-bunch`     — swaying grape bunches ripening purple grape by grape
//! - `watermelon-slice`— half-moon slices with rind and seeded red flesh
//! - `strawberry-seeds`— leafy-topped strawberries ripening red from the tip up
//! - `pineapple-lattice`— crosshatched pineapple ripening under a leaf crown
//! - `cherry-swing`    — swinging cherry pairs ripening red pair by pair
//! - `juice-squeeze`   — lemon half squeezed flat over a filling glass
//! - `berry-pop`       — crowned blueberries popping into a row with a bounce
//! - `kiwi-rings`      — kiwi halves with seed rings, cut open one by one

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// Set dot `(x, y)` and tint its cell `c`. Colour is one per cell (2×4 dots),
/// so the last object drawn over a cell decides its colour.
fn px(grid: &mut BrailleGrid, x: i32, y: i32, c: Color) {
    if x >= 0 && y >= 0 {
        draw::dot_i(grid, x, y);
        let (cx, cy) = (x as usize / 2, y as usize / 4);
        draw::tint_row(grid, cy, cx, cx, c);
    }
}

/// All styles in the `fruits` theme.
///
/// Returns eleven [`ProgressStyle`] implementations, each using a distinct
/// geometric or motion strategy so no two differ by color alone.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(CitrusWheel),
        Box::new(AppleBite),
        Box::new(BananaPeel),
        Box::new(GrapeBunch),
        Box::new(WatermelonSlice),
        Box::new(StrawberrySeeds),
        Box::new(PineappleLattice),
        Box::new(CherrySwing),
        Box::new(JuiceSqueeze),
        Box::new(BerryPop),
        Box::new(KiwiRings),
    ]
}

// ── CitrusWheel ───────────────────────────────────────────────────────────────
// A row of orange wheels seen face-on: an orange rind ring and eight juicy
// segments split by gap membranes radiating from the centre. Segments
// fill in clockwise, wheel after wheel, with eased; the wheels slowly turn.

struct CitrusWheel;
impl ProgressStyle for CitrusWheel {
    fn name(&self) -> &str {
        "citrus-wheel"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of orange wheels — rind and eight juicy segments — filling segment by segment"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 1) / 2).clamp(1, 7);
        let cy = hi / 2;
        let n = (wi / (2 * r + 6)).max(1);
        let pitch = wi / n;
        let rind = Color::rgb(255, 125, 0);
        let pulp = Color::rgb(255, 205, 90);
        let rf = r as f32;
        let segs = ctx.eased * (n * 8) as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let spin = ctx.time * 0.5 + i as f32;
            let filled = (segs - (i * 8) as f32).clamp(0.0, 8.0);
            for dy in -r..=r {
                for dx in -r..=r {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d > rf + 0.3 {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if d > rf - 1.0 {
                        px(grid, x, y, rind);
                        continue;
                    }
                    // Clockwise angle from the top, in segment units 0..8.
                    let a = ((dx as f32).atan2(-dy as f32) + spin).rem_euclid(2.0 * PI);
                    let s = a * 8.0 / (2.0 * PI);
                    let membrane = s.fract() < 0.25 || d < 1.5 || d > rf - 1.6;
                    if s < filled && !membrane {
                        px(grid, x, y, pulp);
                    }
                }
            }
        }
        Ok(())
    }
}

// ── AppleBite ────────────────────────────────────────────────────────────────
// A row of red apples (two overlapping lobes, brown stem, green leaf) eaten
// left to right. The apple being eaten loses up to three bites and wobbles;
// finished apples are left as cream cores with their stems.

struct AppleBite;
impl ProgressStyle for AppleBite {
    fn name(&self) -> &str {
        "apple-bite"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of red apples with stems and leaves, eaten bite by bite down to their cores"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 4) / 2).clamp(1, 6);
        let cy = hi - 1 - r;
        let n = (wi / (2 * r + 8)).max(1);
        let pitch = wi / n;
        let red = Color::rgb(225, 40, 50);
        let flesh = Color::rgb(240, 225, 170);
        let stem = Color::rgb(150, 95, 45);
        let leaf = Color::rgb(80, 200, 70);
        let eaten = ctx.eased * n as f32;
        let r2 = r * r + r / 2;
        let rb = r / 2 + 1;
        for i in 0..n {
            let cx = (pitch * i + pitch / 2) & !1;
            let f = (eaten - i as f32).clamp(0.0, 1.0);
            if f >= 1.0 {
                // Core: narrow waist with flared ends and two seed gaps.
                for dy in -r..=r {
                    let hw = if dy.abs() >= r - 1 {
                        3
                    } else if dy.abs() >= r - 2 {
                        2
                    } else {
                        1
                    };
                    for dx in -hw..=hw {
                        if dx == 0 && dy.abs() == 1 {
                            continue;
                        }
                        px(grid, cx + dx, cy + dy, flesh);
                    }
                }
            } else {
                let wob = if f > 0.0 {
                    ((ctx.time * 9.0).sin() * 0.9).round() as i32
                } else {
                    0
                };
                // Bites appear at f > 0, 1/3, 2/3: right, left, right-low.
                let bites = [
                    (cx + r + 2, cy - r / 3, 0.0),
                    (cx - r - 2, cy + r / 4, 0.34),
                    (cx + r + 1, cy + r / 2 + 1, 0.67),
                ];
                for dy in -r..=r {
                    for dx in -(r + 2)..=(r + 2) {
                        let lobe = (dx + 2) * (dx + 2) + dy * dy <= r2
                            || (dx - 2) * (dx - 2) + dy * dy <= r2;
                        if !lobe {
                            continue;
                        }
                        let (x, y) = (cx + dx + wob, cy + dy);
                        let mut c = red;
                        let mut gone = false;
                        for &(bx, by, at) in &bites {
                            if f > at {
                                let d = (x - bx - wob) * (x - bx - wob) + (y - by) * (y - by);
                                if d <= rb * rb {
                                    gone = true;
                                } else if d <= (rb + 2) * (rb + 2) {
                                    c = flesh;
                                }
                            }
                        }
                        if !gone {
                            px(grid, x, y, c);
                        }
                    }
                }
                // Leaf on the right of the stem.
                for &(lx, ly) in &[(1, -2), (2, -3), (3, -3), (2, -2), (3, -2), (4, -3)] {
                    px(grid, cx + lx + wob, cy - r + ly, leaf);
                }
            }
            // Stem sits on every apple and core.
            draw::vline(
                grid,
                cx.max(0) as usize,
                (cy - r - 2).max(0) as usize,
                (cy - r).max(0) as usize,
            );
            draw::tint_row(
                grid,
                ((cy - r - 2).max(0) / 4) as usize,
                (cx / 2) as usize,
                (cx / 2) as usize,
                stem,
            );
        }
        Ok(())
    }
}

// ── BananaPeel ───────────────────────────────────────────────────────────────
// One long banana lying in a smile-shaped curve across the bar: yellow skin,
// brown stem and tip. Progress peels it from the right: the peeled part shows
// cream fruit, and three peel flaps hang from the peel line, swaying with time.

struct BananaPeel;
impl ProgressStyle for BananaPeel {
    fn name(&self) -> &str {
        "banana-peel"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Curved yellow banana peeled from the tip as progress grows; peel flaps dangle and sway"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let skin = Color::rgb(250, 215, 50);
        let fruit = Color::rgb(250, 240, 200);
        let brown = Color::rgb(140, 90, 40);
        let mid = (wi - 1) as f32 / 2.0;
        let half = (wi as f32 / 2.0 - 2.0).max(1.0);
        let hf = hi as f32;
        // Centreline sags in the middle; thickness tapers toward both ends.
        let centre = |x: i32| {
            let u = (x as f32 - mid) / half;
            (hf * 0.62 - hf * 0.42 * u * u, u)
        };
        let peel_x = wi as f32 - 2.0 - ctx.eased * (wi as f32 - 6.0);
        for x in 0..wi {
            let (yc, u) = centre(x);
            if u.abs() > 1.0 {
                continue;
            }
            let thick = (hf * 0.36 * (1.0 - u * u).max(0.0).sqrt()).max(1.0);
            let peeled = x as f32 > peel_x && u < 0.93;
            let ys = (yc - thick / 2.0).round() as i32;
            let ye = (yc + thick / 2.0).round() as i32;
            for y in ys..=ye {
                let c = if u.abs() > 0.93 {
                    brown
                } else if peeled {
                    fruit
                } else {
                    skin
                };
                // A ridge line of negative space runs along the skin.
                if !peeled && y == ys + 1 && thick > 3.0 && x % 3 != 0 {
                    continue;
                }
                if peeled && (y == ys || y == ye) && thick > 2.5 {
                    continue;
                }
                px(grid, x, y, c);
            }
        }
        // Peel flaps hang from the peel line once peeling has started.
        if ctx.eased > 0.02 {
            let x = peel_x.round() as i32;
            let (yc, _) = centre(x);
            for k in -1..=1 {
                let sway = (ctx.time * 2.5 + k as f32).sin() * 1.5;
                let y0 = yc.round() as i32;
                for y in y0..hi {
                    let t = (y - y0) as f32 / hf;
                    let fx = x + (k as f32 * (2.0 + t * 8.0) + sway * t * 2.0).round() as i32;
                    px(grid, fx, y, skin);
                    if y + 1 == hi {
                        px(grid, fx, y, brown);
                    }
                }
            }
        }
        Ok(())
    }
}

// ── GrapeBunch ───────────────────────────────────────────────────────────────
// Hanging grape bunches: rows of 3-2-1 round grapes under a brown stem and a
// green leaf. Grapes ripen (fill purple, with a glossy highlight) one at a
// time with eased; unripe grapes are dim rings. Bunches sway with time, the
// lower rows swinging further than the top.

struct GrapeBunch;
impl ProgressStyle for GrapeBunch {
    fn name(&self) -> &str {
        "grape-bunch"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Swaying grape bunches on stems; grapes ripen purple one by one as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let purple = [Color::rgb(130, 60, 200), Color::rgb(165, 85, 225)];
        let dim = Color::rgb(85, 85, 110);
        let stem = Color::rgb(150, 100, 50);
        let leaf = Color::rgb(70, 190, 70);
        let total = n * 6;
        let ripe = ctx.eased * total as f32;
        let mut g = 0;
        for i in 0..n {
            let bx = pitch * i + pitch / 2;
            let sway = (ctx.time * 1.8 + i as f32 * 1.3).sin();
            // Collect grape centres; later (lower) grapes overlap earlier ones,
            // and a one-dot gap ring around each keeps the grapes distinct.
            let mut grapes = [(0i32, 0i32, false); 6];
            let mut m = 0;
            for (j, count) in [3, 2, 1].iter().enumerate() {
                let gy = 5 + j as i32 * 4;
                let off = (sway * (j as f32 + 1.0) * 0.5).round() as i32;
                for k in 0..*count {
                    let gx = bx - (count - 1) * 3 + k * 6 + off;
                    grapes[m] = (gx, gy, (g as f32) < ripe.floor());
                    m += 1;
                    g += 1;
                }
            }
            for y in 2..hi {
                for x in (bx - 9)..=(bx + 9) {
                    let d = |q: usize| {
                        let (gx, gy, _) = grapes[q];
                        (x - gx) * (x - gx) + (y - gy) * (y - gy)
                    };
                    let Some(o) = (0..6).rev().find(|&q| d(q) <= 5) else {
                        continue;
                    };
                    if (o + 1..6).any(|q| d(q) <= 8) {
                        continue;
                    }
                    let (gx, gy, full) = grapes[o];
                    if full && !(x - gx == -1 && y - gy == -1) {
                        px(grid, x, y, purple[o % 2]);
                    } else if !full && d(o) >= 4 {
                        px(grid, x, y, dim);
                    }
                }
            }
            // Stem and leaf, drawn last so their cells keep their colours.
            for y in 0..3 {
                px(grid, bx, y, stem);
            }
            for &(lx, ly) in &[(2, 0), (3, 0), (4, 0), (5, 1), (3, 1), (4, 1), (2, 1)] {
                px(grid, bx + lx, ly, leaf);
            }
        }
        Ok(())
    }
}

// ── WatermelonSlice ───────────────────────────────────────────────────────────
// A row of half-moon watermelon slices, flat side up: green rind around the
// curve, a pale band, then red flesh with black-seed gaps. Each slice fills
// with flesh from the rind upward in turn; the juicy surface ripples with time.

struct WatermelonSlice;
impl ProgressStyle for WatermelonSlice {
    fn name(&self) -> &str {
        "watermelon-slice"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Half-moon watermelon slices with green rind and seeded red flesh, filling one by one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = (hi - 3).clamp(2, 12);
        let y0 = hi - 1 - r;
        let n = (wi / (2 * r + 2)).max(1);
        let pitch = wi / n;
        let rind = Color::rgb(40, 170, 70);
        let flesh = Color::rgb(240, 50, 70);
        let pale = Color::rgb(210, 240, 190);
        let r2 = r * r;
        let inner = (r - 2) * (r - 2);
        let flesh_r = (r - 3) * (r - 3);
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            for dy in 0..=r {
                // Surface level (rows from the rind upward), rippling.
                let ripple = (ctx.time * 4.0 + dy as f32).sin() * 0.6;
                for dx in -r..=r {
                    let d = dx * dx + dy * dy;
                    if d > r2 {
                        continue;
                    }
                    let (x, y) = (cx + dx, y0 + dy);
                    if d > inner {
                        px(grid, x, y, rind);
                    } else if d > flesh_r {
                        if dy == 0 {
                            px(grid, x, y, pale);
                        }
                    } else {
                        let depth = r as f32 - (d as f32).sqrt();
                        let filled = depth <= f * (r as f32 - 1.0) + ripple;
                        let seed = (dy % 4 == 2) && (dx + dy).rem_euclid(5) == 0 && dy < r - 4;
                        if f > 0.0 && filled && !seed {
                            px(grid, x, y, flesh);
                        } else if dy == 0 {
                            px(grid, x, y, pale);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ── StrawberrySeeds ───────────────────────────────────────────────────────────
// A row of strawberries: heart-shouldered bodies tapering to a point, with a
// spiky green calyx and stem. Each berry ripens from its tip upward with
// eased, turning from a pale outline into solid red dotted with seed gaps.
// Calyx tips flutter with time.

struct StrawberrySeeds;
impl ProgressStyle for StrawberrySeeds {
    fn name(&self) -> &str {
        "strawberry-seeds"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of strawberries with leafy green tops, ripening red and seeded from tip to shoulder"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bh = (hi - 4).clamp(2, 12);
        let y0 = hi - bh;
        let n = (wi / 18).max(1);
        let pitch = wi / n;
        let red = Color::rgb(230, 35, 60);
        let pale = Color::rgb(195, 225, 165);
        let leaf = Color::rgb(70, 185, 60);
        let ripe = ctx.eased * n as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let f = (ripe - i as f32).clamp(0.0, 1.0);
            for row in 0..bh {
                let t = row as f32 / bh as f32;
                let mut hw = 6.5 * (1.0 - t).sqrt();
                if row == 0 {
                    hw -= 1.5;
                } else if row == 1 {
                    hw -= 0.5;
                }
                let hw = hw.round().max(0.0) as i32;
                let y = y0 + row;
                let ripe_row = t >= 1.0 - f;
                for dx in -hw..=hw {
                    let edge = dx.abs() == hw || row == 0 || row == bh - 1;
                    if ripe_row {
                        let seed = row % 3 == 1 && (dx + (row / 3 % 2) * 2).rem_euclid(4) == 0;
                        if edge || !seed {
                            px(grid, cx + dx, y, red);
                        }
                    } else if edge {
                        px(grid, cx + dx, y, pale);
                    }
                }
            }
            // Calyx: a band of leaves with fluttering tips, plus a short stem.
            let fl = ((ctx.time * 3.0 + i as f32).sin() * 1.2).round() as i32;
            draw::hline(
                grid,
                (cx - 4).max(0) as usize,
                (cx + 4).max(0) as usize,
                (y0 - 1).max(0) as usize,
            );
            for &tx in &[-5, -2, 2, 5] {
                px(grid, cx + tx + fl * tx.signum(), y0 - 2, leaf);
            }
            px(grid, cx, y0 - 2, leaf);
            px(grid, cx, y0 - 3, leaf);
            px(grid, cx + 1, y0 - 4, leaf);
            draw::tint_row(
                grid,
                ((y0 - 1).max(0) / 4) as usize,
                ((cx - 5).max(0) / 2) as usize,
                ((cx + 5).max(0) / 2) as usize,
                leaf,
            );
        }
        Ok(())
    }
}

// ── PineappleLattice ──────────────────────────────────────────────────────────
// A pineapple lying on its side: an oval body with a leaf crown fanning out to
// the right. Ripe (filled) rind is solid gold cut by diagonal grooves, giving
// the crosshatched diamond skin; unripe rind is an outline with lattice nodes.
// The ripe region sweeps left to right with eased; the crown sways with time.

struct PineappleLattice;
impl ProgressStyle for PineappleLattice {
    fn name(&self) -> &str {
        "pineapple-lattice"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Pineapple on its side: crosshatched gold skin ripens left to right under a swaying leaf crown"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let crown_len = (wi / 4).clamp(2, 22);
        let x0 = 1;
        let x1 = (wi - crown_len - 1).max(x0 + 2);
        let cx = (x0 + x1) as f32 / 2.0;
        let rx = ((x1 - x0) as f32 / 2.0).max(1.0);
        let cy = (hi - 1) as f32 / 2.0;
        let ry = (hi as f32 / 2.0 - 0.5).max(1.0);
        let inside = |x: i32, y: i32| {
            let nx = (x as f32 - cx) / rx;
            let ny = (y as f32 - cy) / ry;
            nx * nx + ny * ny <= 1.0
        };
        let fill_x = x0 as f32 + ctx.eased * (x1 - x0 + 1) as f32;
        let gold = Color::rgb(245, 185, 40);
        let green = Color::rgb(120, 150, 60);
        let leaf = Color::rgb(60, 195, 80);
        for y in 0..hi {
            for x in x0..=x1 {
                if !inside(x, y) {
                    continue;
                }
                let edge = !inside(x - 1, y)
                    || !inside(x + 1, y)
                    || !inside(x, y - 1)
                    || !inside(x, y + 1);
                let a = (x + y).rem_euclid(6);
                let b = (x - y).rem_euclid(6);
                if (x as f32) < fill_x {
                    // Ripe: solid skin, diagonal grooves left as negative space.
                    if edge || (a != 0 && b != 0) {
                        px(grid, x, y, gold);
                    }
                } else if edge || (a == 0 && b == 0) {
                    px(grid, x, y, green);
                }
            }
        }
        // Leaf crown: five blades fanning from the stem end, each swaying.
        let base_y = cy.round() as i32;
        for k in 0..5 {
            let a = (k as f32 - 2.0) * 0.42 + (ctx.time * 2.2 + k as f32).sin() * 0.1;
            let len = crown_len as f32 * (1.0 - (k as f32 - 2.0).abs() * 0.18);
            let steps = len.max(1.0) as i32;
            for s in 0..=steps {
                let lx = x1 - 1 + (s as f32 * a.cos()).round() as i32;
                let ly = base_y + (s as f32 * a.sin()).round() as i32;
                px(grid, lx, ly, leaf);
                if s < steps / 2 {
                    px(grid, lx, ly + 1, leaf);
                }
            }
        }
        Ok(())
    }
}

// ── CherrySwing ───────────────────────────────────────────────────────────────
// A row of cherry pairs: two round cherries on stems that meet at a leafy top,
// each pair swinging like a pendulum with time. Pairs ripen left to right with
// eased: solid red with a glint once ripe, pale-green rings before.

struct CherrySwing;
impl ProgressStyle for CherrySwing {
    fn name(&self) -> &str {
        "cherry-swing"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of swinging cherry pairs on joined stems, ripening red one pair at a time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 6) / 2).clamp(1, 4);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let red = Color::rgb(215, 20, 50);
        let unripe = Color::rgb(170, 215, 120);
        let stem = Color::rgb(90, 160, 60);
        let leaf = Color::rgb(60, 200, 80);
        let ripe = ctx.eased * n as f32;
        let cy = hi - 1 - r;
        for i in 0..n {
            let top_x = pitch * i + pitch / 2;
            let swing = (ctx.time * 2.0 + i as f32 * 0.9).sin() * 2.0;
            let f = (ripe - i as f32).clamp(0.0, 1.0);
            for side in [-1, 1] {
                let cx = top_x + side * (r + 1) + swing.round() as i32;
                // Stem: a curve from the shared top to the cherry.
                let steps = cy - r;
                for s in 0..steps {
                    let t = s as f32 / steps.max(1) as f32;
                    let sx = top_x as f32 + (cx - top_x) as f32 * t.sqrt();
                    px(grid, sx.round() as i32, s, stem);
                }
                for dy in -r..=r {
                    for dx in -r..=r {
                        let d = dx * dx + dy * dy;
                        if d > r * r + 1 {
                            continue;
                        }
                        let (x, y) = (cx + dx, cy + dy);
                        if f >= 1.0 || (f > 0.0 && side < 0) {
                            if !(dx == -r / 2 && dy == -r / 2) {
                                px(grid, x, y, red);
                            }
                        } else if d > (r - 1) * (r - 1) {
                            px(grid, x, y, unripe);
                        }
                    }
                }
            }
            for &(lx, ly) in &[(1, 0), (2, 0), (3, 1), (2, 1), (4, 1)] {
                px(grid, top_x + lx, ly, leaf);
            }
        }
        Ok(())
    }
}

// ── JuiceSqueeze ──────────────────────────────────────────────────────────────
// A lemon half is squeezed over a glass: the lemon flattens as progress rises,
// juice drips fall from it with time, and the glass fills with yellow juice.
// Spent lemon rinds pile up beside the glass as the squeezing goes on.

struct JuiceSqueeze;
impl ProgressStyle for JuiceSqueeze {
    fn name(&self) -> &str {
        "juice-squeeze"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Lemon half squeezed flat over a glass that fills with juice; drips fall and spent rinds pile up"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let lemon = Color::rgb(240, 235, 60);
        let pith = Color::rgb(255, 250, 205);
        let juice = Color::rgb(255, 170, 30);
        let glass = Color::rgb(175, 205, 225);
        let p = ctx.eased;
        // Glass in the middle: walls and base; juice rises with progress.
        let gw = 22;
        let x0 = (wi / 2 - gw / 2) | 1;
        let x1 = x0 + gw + 1;
        let gtop = hi - 9;
        for y in gtop..hi {
            px(grid, x0, y, glass);
            px(grid, x1, y, glass);
        }
        for x in x0..=x1 {
            px(grid, x, hi - 1, glass);
        }
        let level = (p * (hi - 1 - gtop) as f32).round() as i32;
        for y in (hi - 1 - level..hi - 1).rev() {
            for x in x0 + 1..x1 {
                px(grid, x, y, juice);
            }
        }
        // Lemon half, cut face down, flattening as it is squeezed.
        let cx = (x0 + x1) / 2;
        let lh = (5.0 - p * 3.0).round() as i32;
        let lw = 9 + (p * 3.0) as i32;
        let base = gtop - 2;
        for dy in 0..=lh {
            let t = dy as f32 / lh.max(1) as f32;
            let hw = (lw as f32 * (1.0 - t * t).sqrt()).round() as i32;
            let c = if dy == 0 { pith } else { lemon };
            for dx in -hw..=hw {
                px(grid, cx + dx, base - dy, c);
            }
        }
        // Drips falling from the cut face into the glass.
        if p > 0.0 && p < 1.0 {
            let surface = hi - 2 - level;
            for k in 0..2 {
                let ph = (ctx.time * 1.8 + k as f32 * 0.5).fract();
                let dy = (ph * (surface - base).max(1) as f32) as i32;
                px(grid, cx - 2 + k * 4, base + 1 + dy, juice);
            }
        }
        // Spent rinds pile up on the left: small flattened domes.
        let spent = (p * 4.0) as i32;
        for k in 0..spent {
            let rx = x0 - 8 - k * 9;
            for dy in 0..2 {
                for dx in -3 + dy..=3 - dy {
                    px(grid, rx + dx, hi - 1 - dy, lemon);
                }
            }
        }
        Ok(())
    }
}

// ── BerryPop ──────────────────────────────────────────────────────────────────
// Blueberries pop into a row one by one with eased: each is a round blue berry
// with the star-shaped crown gap on top and a pale glint. The newest berry
// drops in and bounces with time; empty spots wait as dim rings.

struct BerryPop;
impl ProgressStyle for BerryPop {
    fn name(&self) -> &str {
        "berry-pop"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Crowned blueberries pop into a row one by one, the newest bouncing as it lands"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 2) / 2).clamp(1, 5);
        let n = (wi / (2 * r + 2)).max(1);
        let pitch = wi / n;
        let blue = Color::rgb(70, 95, 215);
        let crown = Color::rgb(45, 55, 130);
        let dim = Color::rgb(70, 75, 100);
        let popped = ctx.eased * n as f32;
        let rf = r as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let f = (popped - i as f32).clamp(0.0, 1.0);
            let newest = f > 0.0 && (popped - i as f32) < 1.5;
            // The newest berry bounces; others rest on the floor.
            let lift = if newest {
                ((ctx.time * 5.0 + i as f32).sin().abs() * (hi - 2 * r - 1) as f32) as i32
            } else {
                0
            };
            let cy = hi - 1 - r - lift;
            for dy in -r..=r {
                for dx in -r..=r {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d > rf + 0.3 {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if f <= 0.0 {
                        if d > rf - 0.8 {
                            px(grid, x, y, dim);
                        }
                        continue;
                    }
                    // Crown: an X-shaped gap just below the top of the berry.
                    let star = dy <= -r + 3 && dy > -r && (dx.abs() == (dy + r - 2).abs());
                    let glint = dx == r / 2 && dy == 0;
                    if star || glint {
                        continue;
                    }
                    let c = if dy <= -r + 3 && dx.abs() <= 2 {
                        crown
                    } else {
                        blue
                    };
                    px(grid, x, y, c);
                }
            }
        }
        Ok(())
    }
}

// ── KiwiRings ────────────────────────────────────────────────────────────────
// A row of kiwi halves seen face-on: fuzzy brown skin, green flesh, a ring of
// black seeds (gaps) around a cream core. Halves are cut one by one with
// eased (uncut ones show only the skin outline); the seed ring slowly turns.

struct KiwiRings;
impl ProgressStyle for KiwiRings {
    fn name(&self) -> &str {
        "kiwi-rings"
    }
    fn theme(&self) -> &str {
        "fruits"
    }
    fn describe(&self) -> &str {
        "Row of kiwi halves — brown skin, green flesh, seed ring and pale core — cut open one by one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi - 1) / 2).clamp(1, 7);
        let cy = hi / 2;
        let n = (wi / (2 * r + 4)).max(1);
        let pitch = wi / n;
        let skin = Color::rgb(140, 95, 50);
        let flesh = Color::rgb(120, 200, 60);
        let core = Color::rgb(235, 240, 200);
        let rf = r as f32;
        let cut = ctx.eased * n as f32;
        for i in 0..n {
            let cx = pitch * i + pitch / 2;
            let open = cut > i as f32 + 0.5;
            let spin = ctx.time * 0.6 + i as f32;
            for (pass, dy) in (0..2).flat_map(|p| (-r..=r).map(move |dy| (p, dy))) {
                for dx in -r..=r {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d > rf + 0.3 {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if d > rf - 1.0 {
                        // Skin is drawn in a first pass so flesh owns shared cells.
                        if pass == 0 {
                            px(grid, x, y, skin);
                        }
                    } else if !open || pass == 0 {
                        // Uncut halves show only their skin.
                    } else if d < rf * 0.3 {
                        px(grid, x, y, core);
                    } else {
                        // Seed ring: gaps at regular angles around mid-radius.
                        let a = (dy as f32).atan2(dx as f32) + spin;
                        let ring = (d - rf * 0.5).abs() < 0.9;
                        let seed = ring && (a * 12.0 / (2.0 * PI)).rem_euclid(1.0) < 0.5;
                        if !seed {
                            px(grid, x, y, flesh);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
