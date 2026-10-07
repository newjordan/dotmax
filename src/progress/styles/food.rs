//! Food / kitchen progress bars — a full set of animated, braille-rendered
//! loading styles themed around cooking, eating, and the kitchen.
//!
//! Every bar is stateless: it is a pure function of `(ctx.eased, ctx.time)`.
//! Bubbles rise, steam drifts, conveyor plates slide, and popcorn pops — all
//! driven by `ctx.time` so the bars stay alive at any fixed progress value.

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

// ---------------------------------------------------------------------------
// Deterministic hash — gives a pseudo-random u32 from any u32 seed.
// ---------------------------------------------------------------------------
#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Glass beer mugs with handles, filling amber one after another under a
/// white foam head; bubbles rise through the beer over time.
struct BeerGlass;
impl ProgressStyle for BeerGlass {
    fn name(&self) -> &str {
        "beer-glass"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Row of glass beer mugs with handles filling amber in turn; foam heads and rising bubbles"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 6 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 28).max(1);
        let pitch = wi / n;
        let bw = ((pitch - 10).clamp(4, 18)) & !1;
        let glass = Color::rgb(175, 200, 220);
        let beer = Color::rgb(240, 165, 30);
        let foam = Color::rgb(250, 248, 235);
        let ih = hi - 2;
        for i in 0..n {
            let x0 = (pitch * i + (pitch - bw - 6) / 2) | 1;
            let x1 = x0 + bw + 1;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            // Glass: walls, base, and a D-shaped handle on the right.
            for y in 1..hi {
                px(grid, x0, y, glass);
                px(grid, x1, y, glass);
            }
            for x in x0..=x1 {
                px(grid, x, hi - 1, glass);
            }
            let (hy0, hy1) = (4.min(hi - 2), (hi - 4).max(4));
            for x in x1..=x1 + 4 {
                px(grid, x, hy0, glass);
                px(grid, x, hy1, glass);
            }
            for y in hy0..=hy1 {
                px(grid, x1 + 4, y, glass);
            }
            // Beer from the bottom up; the top two rows are foam.
            let level = (f * ih as f32).round() as i32;
            if level <= 0 {
                continue;
            }
            let top = hi - 1 - level;
            // Bottom-up, so the foam owns the colour of the cell it shares.
            for y in (top..hi - 1).rev() {
                for x in x0 + 1..x1 {
                    let foamy = y < top + 2 && level > 2;
                    // Bubbles: negative-space dots rising on fixed columns.
                    let col = (x - x0 - 1) % 5 == (i % 5) + 1;
                    let rise = ((ctx.time * 5.0) as i32 + x * 3) % level.max(1);
                    if !foamy && col && y == hi - 2 - rise {
                        continue;
                    }
                    px(grid, x, y, if foamy { foam } else { beer });
                }
            }
            // A full mug's head domes up over the rim.
            if f >= 1.0 {
                for x in x0..=x1 {
                    if (x + (ctx.time * 4.0) as i32) % 3 != 0 {
                        px(grid, x, 0, foam);
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// A coffee pot travels along a row of mugs, pouring a stream into each in
/// turn; filled mugs steam.
struct CoffeePour;
impl ProgressStyle for CoffeePour {
    fn name(&self) -> &str {
        "coffee-pour"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Coffee pot pours a wavering stream into a row of mugs one by one; full mugs steam"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let mug = Color::rgb(235, 235, 240);
        let coffee = Color::rgb(130, 75, 35);
        let crema = Color::rgb(205, 150, 90);
        let pot = Color::rgb(150, 160, 175);
        let steam = Color::rgb(170, 170, 185);
        let top = hi - 8;
        let pour = ctx.eased * n as f32;
        let current = (pour as i32).min(n - 1);
        for i in 0..n {
            let x0 = (pitch * i + (pitch - 16) / 2) | 1;
            let x1 = x0 + 11;
            let f = (pour - i as f32).clamp(0.0, 1.0);
            // Mug: walls, base, and a handle on the right.
            for y in top..hi {
                px(grid, x0, y, mug);
                px(grid, x1, y, mug);
            }
            for x in x0..=x1 {
                px(grid, x, hi - 1, mug);
            }
            for &(dx, dy) in &[(1, 1), (2, 1), (3, 2), (3, 3), (3, 4), (2, 5), (1, 5)] {
                px(grid, x1 + dx, top + dy, mug);
            }
            // Coffee level, crema on top.
            let level = (f * (hi - 1 - top) as f32).round() as i32;
            for y in (hi - 1 - level..hi - 1).rev() {
                for x in x0 + 1..x1 {
                    let c = if y == hi - 1 - level { crema } else { coffee };
                    px(grid, x, y, c);
                }
            }
            // Full mugs give off two curling wisps of steam.
            if f >= 1.0 && i != current {
                for k in 0..2 {
                    for y in 1..top - 1 {
                        let sx = x0
                            + 3
                            + k * 5
                            + ((ctx.time * 3.0 + y as f32 * 0.9 + k as f32 * 2.0).sin() * 1.3)
                                .round() as i32;
                        if (y + (ctx.time * 6.0) as i32 + k) % 3 != 0 {
                            px(grid, sx, y, steam);
                        }
                    }
                }
            }
        }
        // The pot hovers above the mug being filled, tipped toward its spout.
        if ctx.eased < 1.0 {
            let x0 = (pitch * current + (pitch - 16) / 2) | 1;
            let sx = x0 + 6;
            let bx = sx - 8;
            for y in 0..4 {
                for x in bx + y / 2..bx + 7 - y / 2 {
                    px(grid, x, y, pot);
                }
            }
            px(grid, bx - 1, 1, pot);
            px(grid, bx - 1, 2, pot);
            px(grid, bx + 7, 2, pot);
            px(grid, sx - 1, 3, pot);
            // Wavering stream from the spout down to the coffee surface.
            let f = (pour - current as f32).clamp(0.0, 1.0);
            let surface = hi - 1 - (f * (hi - 1 - top) as f32).round() as i32;
            for y in 4..surface {
                let wob = ((ctx.time * 12.0 + y as f32).sin() * 0.6).round() as i32;
                px(grid, sx + wob, y, coffee);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pizza slices served one by one: crust along the top, cheese tapering to a
/// point, pepperoni; the slice being served drips a stretchy cheese string.
struct PizzaSlices;
impl ProgressStyle for PizzaSlices {
    fn name(&self) -> &str {
        "pizza-slices"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Row of pepperoni pizza slices served in turn; the newest one stretches a swaying cheese string"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 6 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 18).max(1);
        let pitch = wi / n;
        let crust = Color::rgb(200, 130, 60);
        let cheese = Color::rgb(255, 210, 80);
        let pep_c = Color::rgb(210, 40, 40);
        let dim = Color::rgb(90, 85, 80);
        let sh = (hi - 2).max(3);
        let served = ctx.eased * n as f32;
        for i in 0..n {
            let cx = (pitch * i + pitch / 2) & !1;
            let f = (served - i as f32).clamp(0.0, 1.0);
            // Rows filled from the crust down for the slice being served.
            let rows = (f * sh as f32).ceil() as i32;
            for row in 0..sh {
                let hw = ((sh - row) as f32 * 7.0 / sh as f32).round() as i32;
                let y = row;
                for dx in -hw..=hw {
                    let x = cx + dx;
                    let edge = dx.abs() == hw;
                    if row < rows {
                        // Crust fills the first cell row; two cell-aligned
                        // pepperoni rounds sit in the cheese with a gap rim.
                        let pep = matches!((dx, row), (-4..=-3, 5..=6) | (2..=3, 9..=10));
                        let rim = matches!((dx, row), (-4..=-3, 4 | 7) | (2..=3, 8 | 11));
                        let c = if row < 4 {
                            crust
                        } else if pep {
                            pep_c
                        } else {
                            cheese
                        };
                        if !rim && (row != 0 || (dx + 30) % 3 != 0) {
                            px(grid, x, y, c);
                        }
                    } else if edge || row == 0 {
                        px(grid, x, y, dim);
                    }
                }
            }
            // Cheese string below the slice being served.
            if f > 0.0 && f < 1.0 {
                let sway = (ctx.time * 3.0 + i as f32).sin();
                let y0 = rows.min(sh);
                for y in y0..hi {
                    let t = (y - y0) as f32 / (hi - y0).max(1) as f32;
                    px(grid, cx + (sway * t * 3.0).round() as i32, y, cheese);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Boiling pot: a steel stock pot with a lipped rim and side handles; the
/// water rises with progress, bubbles stream up through it and burst at the
/// surface, and steam curls over the rim once it is near the boil.
struct BoilingPot;
impl ProgressStyle for BoilingPot {
    fn name(&self) -> &str {
        "boiling-pot"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Steel pot with handles fills with bubbling water; bubbles burst at the surface and steam rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 12 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let steel = Color::rgb(160, 170, 185);
        let water = Color::rgb(60, 145, 235);
        let foam = Color::rgb(210, 235, 255);
        let steam = Color::rgb(175, 175, 190);
        let pw = (wi * 6 / 10).max(6);
        let x0 = (wi - pw) / 2;
        let x1 = x0 + pw;
        let rim = (hi / 3).max(2);
        // Pot: walls, base, lipped rim, loop handles.
        for y in rim..hi {
            px(grid, x0, y, steel);
            px(grid, x1, y, steel);
        }
        for x in x0 - 2..=x1 + 2 {
            px(grid, x, rim, steel);
        }
        for x in x0..=x1 {
            px(grid, x, hi - 1, steel);
        }
        for &(dx, dy) in &[(1, 2), (2, 2), (3, 2), (3, 3), (3, 4), (2, 4), (1, 4)] {
            px(grid, x0 - dx, rim + dy, steel);
            px(grid, x1 + dx, rim + dy, steel);
        }
        // Water, with bubble gaps streaming upward on fixed columns.
        let ih = hi - 2 - rim;
        let level = (ctx.eased * ih as f32).round() as i32;
        let surface = hi - 1 - level;
        for y in surface..hi - 1 {
            for x in x0 + 1..x1 {
                let col = (x - x0) % 6 == 3;
                let rise = ((ctx.time * 7.0) as i32 + x * 5) % (level + 2);
                if col && y == hi - 2 - rise {
                    continue;
                }
                px(grid, x, y, water);
            }
        }
        if level > 0 {
            // Bursting bubbles dance just above the surface.
            for x in x0 + 1..x1 {
                let pop = ((ctx.time * 9.0) as i32 + x * 7) % 9;
                if pop == 0 || (pop == 1 && ctx.eased > 0.5) {
                    px(grid, x, surface - 1, foam);
                }
            }
        }
        // Steam over the rim once the pot is past half full.
        if ctx.eased > 0.5 {
            for k in 0..3 {
                let sx = x0 + pw * (k + 1) / 4;
                for y in 0..rim - 1 {
                    let wob = ((ctx.time * 3.0 + y as f32 * 0.8 + k as f32 * 2.0).sin() * 1.5)
                        .round() as i32;
                    if (y + (ctx.time * 6.0) as i32 + k) % 3 != 0 {
                        px(grid, sx + wob, y, steam);
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Sushi conveyor belt: salmon and tamago nigiri ride a moving belt and
/// fill it from the left as progress grows.
struct SushiConveyor;
impl ProgressStyle for SushiConveyor {
    fn name(&self) -> &str {
        "sushi-conveyor"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Salmon and tamago nigiri glide along a moving conveyor belt that fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let belt = Color::rgb(125, 125, 140);
        let rice = Color::rgb(245, 245, 240);
        let salmon = Color::rgb(250, 120, 70);
        let nori = Color::rgb(40, 90, 60);
        let egg = Color::rgb(250, 215, 70);
        // Belt: rails plus tread marks that scroll with time.
        let shift = (ctx.time * 8.0) as i32;
        for x in 0..wi {
            px(grid, x, hi - 3, belt);
            px(grid, x, hi - 1, belt);
            if (x - shift).rem_euclid(4) == 0 {
                px(grid, x, hi - 2, belt);
            }
        }
        // Dishes ride the belt; only those inside the filled span are shown.
        let pitch = 16;
        let travel = ctx.time * 8.0;
        let off = (travel as i32).rem_euclid(pitch);
        let base = (travel as i32).div_euclid(pitch);
        let fill_x = ctx.eased * wi as f32;
        let slots = wi / pitch + 2;
        for k in 0..slots {
            let x = (k - 1) * pitch + off + 2;
            if (x + 6) as f32 > fill_x {
                continue;
            }
            if (k - base).rem_euclid(2) == 0 {
                // Nigiri: rice block topped by a striped salmon slab.
                for y in hi - 7..hi - 3 {
                    for dx in 0..12 {
                        let corner = (dx == 0 || dx == 11) && y == hi - 7;
                        if !corner {
                            px(grid, x + dx, y, rice);
                        }
                    }
                }
                for y in hi - 10..hi - 7 {
                    for dx in -1..13 {
                        let corner = (dx == -1 || dx == 12) && y == hi - 10;
                        if !corner && (dx + y).rem_euclid(4) != 0 {
                            px(grid, x + dx, y, salmon);
                        }
                    }
                }
            } else {
                // Tamago: rice block, yellow omelette slab, nori belt. Drawn
                // bottom-up so the omelette owns the cell it shares with rice.
                for y in (hi - 10..hi - 3).rev() {
                    for dx in 0..12 {
                        let c = if (5..=6).contains(&dx) {
                            nori
                        } else if y < hi - 7 {
                            egg
                        } else {
                            rice
                        };
                        let corner = (dx == 0 || dx == 11) && (y == hi - 10 || y == hi - 7);
                        if !corner {
                            px(grid, x + dx, y, c);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Red-and-white striped popcorn boxes heaping up with fluffy popped kernels,
/// one box after another; a fresh kernel pops out of the filling box.
struct PopcornPopping;
impl ProgressStyle for PopcornPopping {
    fn name(&self) -> &str {
        "popcorn-popping"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Striped popcorn boxes heap up with fluffy kernels in turn while fresh ones pop into the air"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let n = (wi / 22).max(1);
        let pitch = wi / n;
        let red = Color::rgb(225, 40, 45);
        let white = Color::rgb(245, 240, 230);
        let corn = Color::rgb(255, 235, 150);
        let top = hi - 10;
        // Kernel heap layout above the box mouth, filled in this order.
        let heap = [
            (-5, 1),
            (0, 1),
            (5, 1),
            (-3, -2),
            (3, -2),
            (0, -4),
            (-7, -1),
            (7, -1),
        ];
        let puff = |grid: &mut BrailleGrid, kx: i32, ky: i32| {
            for &(dx, dy) in &[
                (0, -1),
                (1, -1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
            ] {
                px(grid, kx + dx, ky + dy, corn);
            }
        };
        for i in 0..n {
            let bx = pitch * i + pitch / 2;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            // Box: tapers from 16 dots at the mouth to 12 at the base, with
            // vertical red stripes on white.
            for y in top..hi {
                let hw = 8 - (y - top) * 2 / 9;
                for dx in -hw..=hw {
                    let stripe = (dx + 16).rem_euclid(4) < 2;
                    let edge = dx.abs() == hw || y == top || y == hi - 1;
                    if stripe {
                        px(grid, bx + dx, y, red);
                    } else if edge {
                        px(grid, bx + dx, y, white);
                    }
                }
            }
            // Heap of popped kernels grows with this box's share of progress.
            let count = (f * heap.len() as f32).round() as usize;
            for &(hx, hy) in heap.iter().take(count) {
                puff(grid, bx + hx, top - 2 + hy);
            }
            // The box being filled pops a kernel on a little arc.
            if f > 0.0 && f < 1.0 {
                let ph = (ctx.time * 1.6 + i as f32).fract();
                let kx = bx + ((ph - 0.5) * 14.0) as i32;
                let ky = top - 2 - ((ph * PI).sin() * (top - 2) as f32) as i32;
                puff(grid, kx, ky);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Egg timers: a row of upright hourglasses that run one after another.
struct EggTimer;
impl ProgressStyle for EggTimer {
    fn name(&self) -> &str {
        "egg-timer"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Row of wooden-capped hourglass egg timers — each drains a trickle of sand from its top bulb into a growing heap, one after another"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let glass = Color::rgb(170, 200, 230);
        let sand = Color::rgb(240, 200, 110);
        let wood = Color::rgb(170, 110, 60);
        let n = ((w as i32 + 9) / 22).max(1);
        // Half-width of the glass at row `r` (1..=14): two bulbs pinched at 7/8.
        let half = |r: i32| -> i32 {
            let k = if r <= 7 { r } else { 15 - r };
            (5.0 - (k - 1) as f32 * 0.75).round().max(1.0) as i32
        };
        for i in 0..n {
            let x0 = i * 22 + 4;
            let cx = x0 + 6;
            let f = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            // Glass walls.
            for r in 1..=14 {
                px(grid, cx - half(r), r, glass);
                px(grid, cx + half(r), r, glass);
            }
            // Top bulb: sand sits in the funnel and sinks as it drains.
            let top_rows = ((1.0 - f) * 5.0).round() as i32;
            for r in (8 - top_rows)..=7 {
                for x in (cx - half(r) + 1)..=(cx + half(r) - 1).max(cx) {
                    px(grid, x, r, sand);
                }
            }
            // Bottom bulb: a heap that grows and narrows toward its peak.
            let heap = (f * 5.0).round() as i32;
            for k in 0..heap {
                let r = 14 - k;
                let ww = (half(r) - 1 - k / 2).max(0);
                for x in (cx - ww)..=(cx + ww) {
                    px(grid, x, r, sand);
                }
            }
            // Falling trickle through the neck while the timer runs.
            if f > 0.0 && f < 1.0 {
                let flick = (ctx.time * 10.0) as i32;
                for r in 8..=(14 - heap) {
                    if (r + flick) % 2 == 0 {
                        px(grid, cx, r, sand);
                    }
                }
            }
            // Wooden end caps and corner posts.
            for x in x0..=x0 + 12 {
                px(grid, x, 0, wood);
                px(grid, x, 15, wood);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Pancake stack on a plate, growing one golden pancake at a time; a butter
/// pat sits on top and maple syrup drips down the side over time.
struct PancakeStack;
impl ProgressStyle for PancakeStack {
    fn name(&self) -> &str {
        "pancake-stack"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Golden pancakes stack up on a plate one by one, topped with butter and dripping syrup"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 8 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let plate = Color::rgb(225, 230, 240);
        let cake = Color::rgb(230, 165, 70);
        let butter = Color::rgb(255, 235, 120);
        let syrup = Color::rgb(150, 70, 20);
        let cx = wi / 2;
        let pw = (wi * 2 / 5).max(8);
        // Plate: a wide rim under the stack.
        for x in cx - pw / 2 - 6..=cx + pw / 2 + 6 {
            px(grid, x, hi - 1, plate);
        }
        // Pancakes: 3 dots tall each, rounded ends, slightly varied widths.
        let max = ((hi - 4) / 3).max(1);
        let count = (ctx.eased * max as f32).ceil() as i32;
        let mut top = hi - 1;
        for k in 0..count {
            let hw = pw / 2 - (hash(k as u32 + 11) % 4) as i32;
            let y0 = hi - 2 - k * 3;
            for dy in 0..3 {
                // Bottom row is a seam gap (only the rounded rims show), so
                // each pancake reads separately from the one below.
                let inset = i32::from(dy != 1);
                for x in cx - hw + inset..=cx + hw - inset {
                    if dy != 0 || (x - cx).abs() >= hw - 2 {
                        px(grid, x, y0 - dy, cake);
                    }
                }
            }
            top = y0 - 2;
        }
        if count > 0 {
            // Butter pat and a syrup drip running down the right side.
            for y in top - 2..top {
                for x in cx - 2..cx + 2 {
                    px(grid, x, y, butter);
                }
            }
            let drip_x = cx + pw / 2 - 4;
            let len = (hi - 2 - top).max(1);
            let reach = ((ctx.time * 3.0).fract() * len as f32) as i32;
            for y in top..=top + reach {
                px(grid, drip_x, y, syrup);
                px(grid, drip_x + 1, y, syrup);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Candy jar: a lidded glass jar fills from the bottom with brightly coloured
/// round sweets, one per cell, while the next sweet drops in from the top.
struct CandyJar;
impl ProgressStyle for CandyJar {
    fn name(&self) -> &str {
        "candy-jar"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Lidded glass jar fills with multicoloured round sweets from the bottom up as one more drops in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 12 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let glass = Color::rgb(175, 205, 225);
        let lid = Color::rgb(220, 60, 120);
        let sweets = [
            Color::rgb(240, 60, 60),
            Color::rgb(250, 210, 50),
            Color::rgb(80, 210, 90),
            Color::rgb(70, 150, 245),
            Color::rgb(245, 120, 210),
            Color::rgb(255, 150, 40),
        ];
        // Jar walls sit on odd/even dot columns so they own their cells.
        let jw = ((wi * 6 / 10) & !1).max(8);
        let x0 = ((wi - jw) / 2) | 1;
        let x1 = x0 + jw + 1;
        // Lid with a knob, then the glass: shoulders, walls, base.
        for x in x0 + 2..=x1 - 2 {
            px(grid, x, 1, lid);
            px(grid, x, 2, lid);
        }
        let mid = (x0 + x1) / 2;
        px(grid, mid, 0, lid);
        px(grid, mid + 1, 0, lid);
        for y in 4..hi {
            px(grid, x0, y, glass);
            px(grid, x1, y, glass);
        }
        px(grid, x0 + 1, 3, glass);
        px(grid, x1 - 1, 3, glass);
        for x in x0..=x1 {
            px(grid, x, hi - 1, glass);
        }
        // Sweets fill whole cells inside the jar, bottom row first.
        let (c0, c1) = ((x0 + 1) / 2, x1 / 2 - 1);
        let rows = hi / 4 - 1;
        if c1 < c0 || rows < 1 {
            return Ok(());
        }
        let per_row = c1 - c0 + 1;
        let total = per_row * rows;
        let count = (ctx.eased * total as f32).round() as i32;
        let sweet = |grid: &mut BrailleGrid, x: i32, y: i32, c: Color| {
            for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1), (0, -1), (1, 2)] {
                px(grid, x + dx, y + dy, c);
            }
        };
        for k in 0..count {
            let (cx, cy) = (c0 + k % per_row, hi / 4 - 1 - k / per_row);
            let c = sweets[(hash(k as u32 * 7 + 3) % 6) as usize];
            sweet(grid, cx * 2, cy * 4 + 1, c);
        }
        // The next sweet falls in from under the lid.
        if count < total {
            let (cx, cy) = (c0 + count % per_row, hi / 4 - 1 - count / per_row);
            let ty = cy * 4 + 1;
            let fy = 4 + ((ctx.time * 1.5).fract() * (ty - 4).max(0) as f32) as i32;
            sweet(
                grid,
                cx * 2,
                fy,
                sweets[(hash(count as u32 * 7 + 3) % 6) as usize],
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Noodle bowl: a red ramen bowl with chopsticks lifting a long wiggly
/// noodle that is slurped shorter and shorter as progress grows.
struct NoodleSlurp;
impl ProgressStyle for NoodleSlurp {
    fn name(&self) -> &str {
        "noodle-slurp"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Chopsticks lift a wiggly noodle from a red ramen bowl; it is slurped shorter as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bowl = Color::rgb(210, 45, 45);
        let rim_c = Color::rgb(240, 240, 235);
        let noodle = Color::rgb(250, 220, 120);
        let stick = Color::rgb(160, 110, 60);
        // Bowl: lower half-ellipse on the left with a pale rim.
        let bhw = (wi / 6).clamp(6, 15);
        let bx = bhw + 1;
        let rim = hi - 9;
        for y in rim..hi {
            let t = (y - rim) as f32 / (hi - rim) as f32;
            let hw = (bhw as f32 * (1.0 - t * t).sqrt()).round() as i32;
            for x in bx - hw..=bx + hw {
                px(grid, x, y, if y == rim { rim_c } else { bowl });
            }
        }
        // The lifted noodle: a wiggling strand out of the bowl to the right,
        // getting shorter as it is slurped.
        let start = bx + bhw / 2;
        let full = wi - 2 - start;
        let end = start + ((1.0 - ctx.eased) * full as f32) as i32;
        let mid = (rim / 2).max(2) as f32;
        for x in start..=end {
            let t = (x - start) as f32;
            let y = mid + (t * 0.45 - ctx.time * 7.0).sin() * (mid - 1.0) * (t / 10.0).min(1.0);
            px(grid, x, y.round() as i32, noodle);
            px(grid, x, y.round() as i32 + 1, noodle);
        }
        // Chopsticks pinch the noodle's far end.
        if end > start + 2 {
            let t = (end - start) as f32;
            let ey = (mid + (t * 0.45 - ctx.time * 7.0).sin() * (mid - 1.0) * (t / 10.0).min(1.0))
                .round() as i32;
            for k in 0..6 {
                px(grid, end + 1 + k, ey - 1 - k / 2, stick);
                px(grid, end + 1 + k, ey + 2 + k / 2, stick);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Toaster: two slices of bread sink into a chrome toaster as the lever goes
/// down, browning from pale to golden to dark, then pop up when done; heat
/// shimmers over the slots while toasting.
struct ToastBrowning;
impl ProgressStyle for ToastBrowning {
    fn name(&self) -> &str {
        "toast-browning"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Two bread slices sink into a chrome toaster and brown with progress, popping up at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let chrome = Color::rgb(185, 195, 210);
        let p = ctx.eased;
        let lerp = |a: f32, b: f32, t: f32| (a + (b - a) * t) as u8;
        // Bread colour: pale -> golden over the first half, golden -> brown after.
        let toast = if p < 0.5 {
            let t = p * 2.0;
            Color::rgb(
                lerp(245.0, 225.0, t),
                lerp(225.0, 160.0, t),
                lerp(170.0, 60.0, t),
            )
        } else {
            let t = (p - 0.5) * 2.0;
            Color::rgb(
                lerp(225.0, 140.0, t),
                lerp(160.0, 80.0, t),
                lerp(60.0, 30.0, t),
            )
        };
        let tw = (wi / 3).clamp(10, 36);
        let x0 = (wi - tw) / 2;
        let x1 = x0 + tw;
        let top = hi / 2;
        // Bread sinks while toasting and pops up (with a bounce) when done.
        let sink = if p >= 1.0 {
            -(((ctx.time * 6.0).sin().abs()) * 2.0) as i32
        } else {
            (p.min(0.15) / 0.15 * 3.0) as i32
        };
        let sw = (tw - 6) / 2;
        for s in 0..2 {
            let sx = x0 + 2 + s * (sw + 2);
            let sy = top - 6 + sink;
            for y in sy..top {
                for x in sx..sx + sw {
                    // Rounded shoulders on the top of each slice.
                    let corner = y == sy && (x == sx || x == sx + sw - 1);
                    if !corner {
                        px(grid, x, y, toast);
                    }
                }
            }
        }
        // Toaster body: rounded box with slots on top and a side lever.
        for y in top..hi {
            for x in x0..=x1 {
                let corner = (y == top || y == hi - 1) && (x == x0 || x == x1);
                let edge = x == x0 || x == x1 || y == top || y == hi - 1;
                let stripe = y == top + 3;
                if !corner && (edge || stripe) {
                    px(grid, x, y, chrome);
                }
            }
        }
        let lever_y = if p >= 1.0 {
            top + 1
        } else {
            top + 1 + (p * (hi - top - 3) as f32) as i32
        };
        for x in x1 + 1..=x1 + 3 {
            px(grid, x, lever_y, chrome);
        }
        // Heat shimmer above the slots while toasting.
        if p > 0.0 && p < 1.0 {
            for k in 0..3 {
                let hx = x0 + tw * (k + 1) / 4;
                for y in 0..top - 7 + sink {
                    let wob = ((ctx.time * 5.0 + y as f32 + k as f32).sin() * 1.2).round() as i32;
                    if (y + (ctx.time * 8.0) as i32) % 2 == 0 {
                        px(grid, hx + wob, y, Color::rgb(255, 140, 60));
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// Lattice pie seen from the side: a tapered tin, crimped rim, and a domed
/// top whose woven lattice crust sweeps across with progress; steam rises
/// from the baked part.
struct PieSweep;
impl ProgressStyle for PieSweep {
    fn name(&self) -> &str {
        "pie-sweep"
    }
    fn theme(&self) -> &str {
        "food"
    }
    fn describe(&self) -> &str {
        "Domed lattice pie in a tin: the woven golden crust sweeps across as progress grows, steaming"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 12 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let tin = Color::rgb(170, 178, 190);
        let crust = Color::rgb(230, 165, 70);
        let raw = Color::rgb(120, 105, 85);
        let steam = Color::rgb(175, 175, 190);
        let pw = (wi * 7 / 10).max(12);
        let x0 = (wi - pw) / 2;
        let x1 = x0 + pw;
        let rim = hi - 6;
        // Tin: tapered sides and base.
        for y in rim + 1..hi {
            let inset = (y - rim) / 2;
            px(grid, x0 + inset, y, tin);
            px(grid, x1 - inset, y, tin);
        }
        for x in x0 + 2..=x1 - 2 {
            px(grid, x, hi - 1, tin);
        }
        let sweep = x0 as f32 + ctx.eased * pw as f32;
        // Crimped rim: a wavy band of crust along the top of the tin.
        for x in x0 - 1..=x1 + 1 {
            let c = if (x as f32) < sweep { crust } else { raw };
            px(grid, x, rim, c);
            px(grid, x, rim + 1 + (x % 2), c);
        }
        // Domed top: woven lattice where baked, outline where not.
        let cx = (x0 + x1) as f32 / 2.0;
        let rx = pw as f32 / 2.0;
        let ry = (rim - 2) as f32;
        let dome = |x: i32, y: i32| {
            let nx = (x as f32 - cx) / rx;
            let ny = (y - rim) as f32 / ry;
            nx * nx + ny * ny <= 1.0 && y < rim
        };
        for y in 0..rim {
            for x in x0..=x1 {
                if !dome(x, y) {
                    continue;
                }
                let edge = !dome(x, y - 1) || !dome(x - 1, y) || !dome(x + 1, y);
                if (x as f32) < sweep {
                    let lattice = (x + y).rem_euclid(5) < 2 || (x - y).rem_euclid(5) < 2;
                    if edge || lattice {
                        px(grid, x, y, crust);
                    }
                } else if edge {
                    px(grid, x, y, raw);
                }
            }
        }
        // Steam curls up from the baked side of the dome.
        if ctx.eased > 0.1 {
            let sx = ((x0 as f32 + sweep) / 2.0) as i32;
            for y in 0..3 {
                let wob = ((ctx.time * 4.0 + y as f32).sin() * 1.5).round() as i32;
                if (y + (ctx.time * 6.0) as i32) % 3 != 0 {
                    px(grid, sx + wob, y, steam);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------

/// All styles in the `food` theme.
///
/// Returns one boxed [`ProgressStyle`] per food/kitchen bar. Call this to get
/// the full set for display in a gallery or picker.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(BeerGlass),
        Box::new(CoffeePour),
        Box::new(PizzaSlices),
        Box::new(BoilingPot),
        Box::new(SushiConveyor),
        Box::new(PopcornPopping),
        Box::new(EggTimer),
        Box::new(PancakeStack),
        Box::new(CandyJar),
        Box::new(NoodleSlurp),
        Box::new(ToastBrowning),
        Box::new(PieSweep),
    ]
}
