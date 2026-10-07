//! Medieval / Fantasy-RPG progress bars — fourteen pictorial scenes, each a
//! recognisable object whose state tracks completion: a sword drawn from its
//! scabbard, a longbow at full draw, a castle rising course by course, a
//! trebuchet lobbing boulders further downrange, heraldic shields filling with
//! colour, two knights charging down the lists, a drawbridge lowering over a
//! moat, a wall of torches lighting one by one, potion flasks filling, a
//! treasure chest opening, a scroll unrolling, a portcullis lifting, a
//! blacksmith hammering out a glowing blade on an anvil, and a heraldic
//! banner unfurling from its pole.
//!
//! Every bar animates via `ctx.time` and tracks completion via `ctx.eased`.
//! Objects are drawn in fixed material colours (stone, steel, wood, gold,
//! flame) with one colour per terminal cell. All are safe at 1×1, 2×1, and
//! 80×1 grids — every write goes through bounds-checked helpers.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic hash helpers (no external crates).
// ---------------------------------------------------------------------------

fn mhash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

fn mhash_f(n: u32) -> f32 {
    (mhash(n) % 1000) as f32 / 1000.0
}

// ---------------------------------------------------------------------------
// Material palette — fixed so the objects read as what they are.
// ---------------------------------------------------------------------------

const STONE: Color = Color::rgb(150, 150, 165);
const SAND: Color = Color::rgb(224, 198, 140);
const WOOD: Color = Color::rgb(160, 104, 54);
const WOOD_DK: Color = Color::rgb(108, 68, 36);
const LEATHER: Color = Color::rgb(126, 70, 38);
const IRON: Color = Color::rgb(118, 124, 142);
const STEEL: Color = Color::rgb(205, 215, 232);
const WHITE: Color = Color::rgb(250, 250, 255);
const GOLD: Color = Color::rgb(245, 196, 64);
const FLAME_Y: Color = Color::rgb(255, 218, 96);
const FLAME_O: Color = Color::rgb(255, 136, 40);
const FLAME_R: Color = Color::rgb(226, 64, 36);
const WATER: Color = Color::rgb(56, 116, 210);
const GRASS: Color = Color::rgb(84, 156, 70);
const EARTH: Color = Color::rgb(120, 86, 52);
const PARCH: Color = Color::rgb(234, 214, 168);
const PARCH_DK: Color = Color::rgb(196, 160, 104);
const GLASS: Color = Color::rgb(170, 205, 228);
const RED: Color = Color::rgb(214, 54, 54);
const BLUE: Color = Color::rgb(64, 104, 224);
const GREEN: Color = Color::rgb(72, 184, 92);
const PURPLE: Color = Color::rgb(164, 84, 214);
const DIM: Color = Color::rgb(96, 96, 112);

/// Heraldic tinctures / brew colours, cycled across repeated objects.
const TINCTURES: [Color; 5] = [RED, BLUE, GREEN, PURPLE, GOLD];

// ---------------------------------------------------------------------------
// Bounds-safe colour drawing helpers (dot space, signed coordinates).
// ---------------------------------------------------------------------------

fn mix(a: Color, b: Color, t: f32) -> Color {
    let l = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    Color::rgb(l(a.r, b.r), l(a.g, b.g), l(a.b, b.b))
}

/// Colour the cell that owns dot `(x, y)`.
fn tint(grid: &mut BrailleGrid, x: i32, y: i32, c: Color) {
    if x >= 0 && y >= 0 {
        let (cx, cy) = (x as usize / 2, y as usize / 4);
        draw::tint_row(grid, cy, cx, cx, c);
    }
}

/// Set one dot and colour its cell; out-of-bounds is ignored.
fn px(grid: &mut BrailleGrid, x: i32, y: i32, c: Color) {
    let (w, h) = draw::dot_dims(grid);
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        draw::dot_i(grid, x, y);
        tint(grid, x, y, c);
    }
}

/// Filled inclusive rectangle `[x0..=x1] × [y0..=y1]`, clipped to the grid.
fn block(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let (w, h) = draw::dot_dims(grid);
    let (xa, xb) = (x0.min(x1).max(0), x0.max(x1).min(w as i32 - 1));
    let (ya, yb) = (y0.min(y1).max(0), y0.max(y1).min(h as i32 - 1));
    for y in ya..=yb {
        for x in xa..=xb {
            px(grid, x, y, c);
        }
    }
}

/// Straight line between two dots.
fn seg(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 4096);
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = x0 as f32 + (x1 - x0) as f32 * t;
        let y = y0 as f32 + (y1 - y0) as f32 * t;
        px(grid, x.round() as i32, y.round() as i32, c);
    }
}

/// Stamp a character-art sprite; `col` maps each glyph to a colour (or none).
fn sprite(
    grid: &mut BrailleGrid,
    rows: &[&str],
    x0: i32,
    y0: i32,
    flip: bool,
    col: impl Fn(char) -> Option<Color>,
) {
    for (j, row) in rows.iter().enumerate() {
        let n = row.len() as i32;
        for (i, ch) in row.chars().enumerate() {
            if let Some(c) = col(ch) {
                let i = i as i32;
                let x = if flip { x0 + n - 1 - i } else { x0 + i };
                px(grid, x, y0 + j as i32, c);
            }
        }
    }
}

/// A small waving pennant flown from a pole whose top is at `(x, y)`.
fn pennant(grid: &mut BrailleGrid, x: i32, y: i32, pole: i32, t: f32, c: Color) {
    seg(grid, x, y, x, y + pole, WOOD_DK);
    for k in 0..4 {
        let lift = ((t * 7.0 - k as f32 * 0.9).sin() * 0.6).round() as i32;
        px(grid, x + 1 + k, y + lift, c);
        if k < 3 {
            px(grid, x + 1 + k, y + 1 + lift, c);
        }
    }
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `medieval` theme, in display order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(SwordDraw),
        Box::new(BowDraw),
        Box::new(CastleBuild),
        Box::new(TrebuchetLaunch),
        Box::new(ShieldCharge),
        Box::new(Jousting),
        Box::new(DrawbridgeLower),
        Box::new(TorchFlame),
        Box::new(PotionBrew),
        Box::new(TreasureChest),
        Box::new(ScrollUnroll),
        Box::new(PortcullisRaise),
        Box::new(AnvilForge),
        Box::new(BannerUnfurl),
    ]
}

// ---------------------------------------------------------------------------
// 1 — Sword draw: a longsword slides out of its scabbard, glint on the blade.
// ---------------------------------------------------------------------------

/// Hilt (pommel, grip, crossguard) pulls left; the blade emerges from a
/// leather scabbard with gold fittings. A glint runs along the bare steel.
struct SwordDraw;
impl ProgressStyle for SwordDraw {
    fn name(&self) -> &str {
        "sword-draw"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A longsword drawn from a leather scabbard: pommel, grip and gold crossguard pull clear as the steel blade emerges, a glint running its edge"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let cy = hi / 2;

        // Scabbard on the right: leather body, gold throat and chape.
        let mouth = (wi * 56 / 100).max(1);
        let end = wi - 2;
        block(grid, mouth, cy - 2, end - 1, cy + 2, LEATHER);
        block(grid, end, cy - 1, end + 1, cy + 1, GOLD);
        block(grid, mouth, cy - 2, mouth + 1, cy + 2, GOLD);

        // Blade: as long as the scabbard; only the drawn part is visible.
        let blade = (end - mouth).max(3);
        let guard = mouth - 1 - (p * blade as f32) as i32;
        let tip = guard + 1 + blade;
        let shown = (mouth - 1).min(tip);
        for x in guard + 1..=shown {
            match tip - x {
                0 | 1 => px(grid, x, cy, STEEL),
                2 | 3 => block(grid, x, cy - 1, x, cy, STEEL),
                _ => block(grid, x, cy - 1, x, cy + 1, STEEL),
            }
        }
        let bare = shown - guard;
        if bare > 3 {
            let gx = guard + 1 + ((ctx.time * 0.9).fract() * bare as f32) as i32;
            block(grid, gx, cy - 1, gx + 1, cy + 1, WHITE);
        }

        // Hilt: crossguard, wrapped grip, round pommel.
        block(grid, guard - 1, cy - 5, guard, cy + 5, GOLD);
        px(grid, guard - 2, cy - 5, GOLD);
        px(grid, guard - 2, cy + 5, GOLD);
        block(grid, guard - 7, cy - 1, guard - 2, cy + 1, LEATHER);
        block(grid, guard - 10, cy - 1, guard - 8, cy + 1, GOLD);
        block(grid, guard - 9, cy - 2, guard - 9, cy + 2, GOLD);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2 — Bow draw: the string is pulled back to full draw, then loosed.
// ---------------------------------------------------------------------------

/// A recurved longbow: progress pulls the nocked arrow back and bends the
/// stave; at 100% the arrow flies into the archery butt.
struct BowDraw;
impl ProgressStyle for BowDraw {
    fn name(&self) -> &str {
        "bow-draw"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A wooden longbow drawn back by progress: the stave bends, the string pulls to a point behind a fletched arrow aimed at a target; at 100% the arrow flies"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let cy = hi / 2;
        let released = ctx.progress >= 0.999;

        // Target (archery butt) on a tripod at the right edge.
        let tx = wi - 6;
        seg(grid, tx - 3, hi - 1, tx, cy, WOOD_DK);
        seg(grid, tx + 3, hi - 1, tx, cy, WOOD_DK);
        let ry = (cy as f32 - 0.5).max(1.0);
        for y in 0..hi {
            for x in tx - 4..=tx + 4 {
                let u = (x - tx) as f32 / 4.5;
                let v = (y - cy) as f32 / ry;
                let r = (u * u + v * v).sqrt();
                if r <= 1.0 && !(0.52..0.64).contains(&r) {
                    let c = if r < 0.34 {
                        GOLD
                    } else if r < 0.64 {
                        RED
                    } else {
                        WHITE
                    };
                    px(grid, x, y, c);
                }
            }
        }

        // Stave: a D-shaped curve bulging toward the target.
        let bx = wi * 36 / 100;
        let bend = if released { 3.0 } else { 3.0 + p * 4.0 };
        for y in 0..hi {
            let u = (y - cy) as f32 / cy.max(1) as f32;
            let off = (bend * (1.0 - u * u)).round() as i32;
            px(grid, bx + off, y, WOOD);
            if u.abs() < 0.7 {
                px(grid, bx + off + 1, y, WOOD);
            }
        }
        block(
            grid,
            bx + bend as i32,
            cy - 1,
            bx + bend as i32 + 1,
            cy + 1,
            LEATHER,
        );

        // String and arrow.
        let max_draw = (wi * 24 / 100).max(1);
        let alen = (wi * 26 / 100).max(4);
        let nock = if released {
            let f = (t * 0.8).fract();
            bx + (f * (tx - 2 - alen - bx) as f32) as i32
        } else {
            bx - (p * max_draw as f32) as i32
        };
        let string_x = if released { bx } else { nock };
        seg(grid, bx, 0, string_x, cy, Color::rgb(236, 230, 206));
        seg(grid, string_x, cy, bx, hi - 1, Color::rgb(236, 230, 206));
        seg(grid, nock, cy, nock + alen, cy, Color::rgb(222, 190, 140));
        let hx = nock + alen;
        let head = if (t * 1.3).fract() < 0.15 {
            WHITE
        } else {
            STEEL
        };
        block(grid, hx, cy - 2, hx, cy + 2, head);
        block(grid, hx + 1, cy - 1, hx + 2, cy + 1, head);
        px(grid, hx + 3, cy, head);
        // Fletching: two feather chevrons behind the nock.
        for k in [0, 3] {
            seg(grid, nock + k, cy - 2, nock + k + 2, cy, RED);
            seg(grid, nock + k, cy + 2, nock + k + 2, cy, RED);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3 — Castle build: towers, curtain walls and keep rise course by course.
// ---------------------------------------------------------------------------

/// Two crenellated towers, curtain walls and a gated keep rise from the ground
/// up; a stone drops onto the working course; banners fly when complete.
struct CastleBuild;
impl ProgressStyle for CastleBuild {
    fn name(&self) -> &str {
        "castle-build"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A stone castle — crenellated towers, curtain walls and a gated keep — rises course by course from the ground; blocks drop onto the fresh course and banners fly when complete"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let sx = |v: i32| v * wi / 88;
        let sy = |v: i32| v * hi / 16;
        let gy = hi - 1;
        let mh = sy(2).max(1);

        // Rows at or below `level` have been laid.
        let level = gy - (p * hi as f32) as i32;
        let hole = |x: i32, y: i32| {
            let gate = x >= sx(40) && x < sx(48) && y >= sy(10);
            let arch = (x == sx(40) || x == sx(48) - 1) && y == sy(10);
            let slit = (x == sx(10) || x == sx(77)) && y >= sy(7) && y <= sy(9);
            let win = (x == sx(43) || x == sx(44)) && y >= sy(5) && y <= sy(6);
            (gate && !arch) || slit || win
        };
        // (x0, x1, body top) for tower, wall, keep, wall, tower.
        let parts = [
            (4, 17, 5),
            (17, 36, 9),
            (36, 52, 3),
            (52, 71, 9),
            (71, 84, 5),
        ];
        for &(a, b, top) in &parts {
            let (x0, x1, top) = (sx(a), sx(b), sy(top));
            for x in x0..x1 {
                let merlon = (x - x0) % 4 < 2;
                let col_top = if merlon { top - mh } else { top };
                for y in col_top.max(level)..=gy {
                    if hole(x, y) {
                        continue;
                    }
                    let fresh = p < 1.0 && y / 4 == level / 4;
                    px(grid, x, y, if fresh { SAND } else { STONE });
                }
            }
        }

        if p < 1.0 && level > 3 {
            // A block being lowered onto the working course.
            let k = (t * 1.1) as u32;
            let f = (t * 1.1).fract();
            let bx = sx(6) + (mhash_f(k) * sx(76) as f32) as i32;
            let by = (f * (level - 2) as f32) as i32;
            block(grid, bx, by, bx + 2, by + 1, SAND);
        } else if p >= 1.0 {
            pennant(grid, sx(10), 0, sy(5) - mh - 1, t, ctx.palette.start);
            pennant(grid, sx(77), 0, sy(5) - mh - 1, t + 0.4, ctx.palette.end);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4 — Trebuchet: the arm swings and hurls a boulder further with progress.
// ---------------------------------------------------------------------------

/// A wooden trebuchet on its A-frame swings its throwing arm (counterweight
/// dropping) and lobs a boulder; progress pushes the landing point downrange.
struct TrebuchetLaunch;
impl ProgressStyle for TrebuchetLaunch {
    fn name(&self) -> &str {
        "trebuchet-launch"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A wooden trebuchet swings its arm as the iron counterweight drops, hurling a boulder along a dotted arc; progress pushes the target flag further downrange"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let sx = |v: i32| v * wi / 88;
        let gy = hi - 1;

        // Ground.
        for x in (0..wi).step_by(2) {
            px(grid, x, gy, GRASS);
        }

        // Frame: base beam, wheels, A-frame legs up to the pivot.
        let (pvx, pvy) = (sx(14), gy - hi * 8 / 16);
        block(grid, sx(5), gy - 1, sx(23), gy - 1, WOOD_DK);
        block(grid, sx(6), gy - 1, sx(8), gy, WOOD_DK);
        block(grid, sx(20), gy - 1, sx(22), gy, WOOD_DK);
        seg(grid, sx(7), gy - 1, pvx, pvy, WOOD);
        seg(grid, sx(21), gy - 1, pvx, pvy, WOOD);

        // Launch cycle: swing, flight, reload.
        let f = (ctx.time / 2.4).fract();
        let cock = 200.0_f32.to_radians();
        let fire = 52.0_f32.to_radians();
        let a = if f < 0.2 {
            cock + (fire - cock) * (f / 0.2).powi(2)
        } else if f < 0.85 {
            fire
        } else {
            fire + (cock - fire) * ((f - 0.85) / 0.15)
        };
        let long = (hi * 9 / 16).max(2) as f32;
        let short = long * 0.4;
        let (ca, sa) = (a.cos(), a.sin());
        let tip = (pvx + (ca * long) as i32, pvy - (sa * long) as i32);
        let cw = (pvx - (ca * short) as i32, pvy + (sa * short) as i32);
        seg(grid, cw.0, cw.1, tip.0, tip.1, WOOD);
        block(grid, cw.0 - 2, cw.1, cw.0 + 1, cw.1 + 3, IRON);

        // Trajectory toward the landing flag.
        let rel = (
            pvx + (fire.cos() * long) as i32,
            pvy - (fire.sin() * long) as i32,
        );
        let lx = sx(34) + (p * (wi - sx(34) - 4) as f32) as i32;
        let apex = (rel.1 as f32 - 1.0).max(0.0) + 2.0;
        let arc = |s: f32| {
            let x = rel.0 as f32 + (lx - rel.0) as f32 * s;
            let y = rel.1 as f32 + ((gy - 2) - rel.1) as f32 * s - apex * 4.0 * s * (1.0 - s);
            (x.round() as i32, y.round() as i32)
        };
        for k in (0..40).step_by(2) {
            let (x, y) = arc(k as f32 / 40.0);
            px(grid, x, y, DIM);
        }
        pennant(grid, lx, gy - 5, 5, ctx.time, RED);

        // Boulder: in the sling, in flight, or landed with a dust puff.
        let (bx, by) = if (0.2..0.85).contains(&f) {
            arc(((f - 0.2) / 0.5).min(1.0))
        } else {
            tip
        };
        block(grid, bx - 1, by - 1, bx + 1, by + 1, STONE);
        if (0.7..0.85).contains(&f) {
            for k in 0..6u32 {
                let dx = (mhash_f(k) * 8.0 - 4.0) as i32;
                let dy = (mhash_f(k + 9) * 3.0) as i32;
                px(grid, bx + dx, by - dy, SAND);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5 — Shield charge: a rack of heater shields fills with heraldic colour.
// ---------------------------------------------------------------------------

/// Heater shields hang in a row; each fills bottom-up with its tincture,
/// leaving a cross as the heraldic charge, then catches a passing glint.
struct ShieldCharge;
impl ProgressStyle for ShieldCharge {
    fn name(&self) -> &str {
        "shield-charge"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A row of heater shields, iron outlines filling one by one with heraldic colours around a cross charge; a glint sweeps across the finished shields"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let n = ((wi + 4) / 18).max(1);
        let slot = wi / n;
        let sw = (slot - 4).clamp(2, 14);
        let shoulder = hi * 55 / 100;
        let bar_y = hi * 30 / 100;
        let sweep = (ctx.time * 40.0) as i32 % (wi + 40);
        let half = |y: i32| {
            if y < shoulder {
                sw / 2
            } else {
                let u = (y - shoulder) as f32 / (hi - shoulder).max(1) as f32;
                ((sw / 2) as f32 * (1.0 - u * u)).round() as i32
            }
        };
        for i in 0..n {
            let cx = i * slot + slot / 2;
            let f = (p * n as f32 - i as f32).clamp(0.0, 1.0);
            let c = TINCTURES[i as usize % TINCTURES.len()];
            // Iron outline.
            for y in 0..hi {
                let hw = half(y).max(1);
                px(grid, cx - hw, y, IRON);
                px(grid, cx + hw - 1, y, IRON);
                if y == 0 || y == hi - 1 {
                    block(grid, cx - hw, y, cx + hw - 1, y, IRON);
                }
            }
            if f <= 0.0 {
                continue;
            }
            // Field rises from the point; the cross stays as negative space.
            let fill_top = hi as f32 - f * hi as f32;
            for y in 0..hi {
                if (y as f32) < fill_top {
                    continue;
                }
                let hw = half(y);
                for x in cx - hw..cx + hw {
                    let cross = x == cx - 1 || x == cx || y == bar_y || y == bar_y + 1;
                    if cross && y > 0 && y < hi - 2 && hw > 2 {
                        continue;
                    }
                    let glint = f >= 1.0 && (x + y - sweep + 20).abs() < 3;
                    px(grid, x, y, if glint { mix(c, WHITE, 0.6) } else { c });
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6 — Jousting: two mounted knights charge each other down the lists.
// ---------------------------------------------------------------------------

/// Knight on a caparisoned horse, facing right (legs drawn separately).
const KNIGHT: [&str; 12] = [
    "......pp................",
    "......hhh...............",
    "......hhh..........e....",
    ".....aaaa.........eee...",
    ".....aaaa........nnnnn..",
    ".....aaaa.......nnnnnnn.",
    ".....aaaa......nnnn..nnn",
    "..cccccccccccccnnn....nn",
    ".ccccccccccccccnn.......",
    "cccccccccccccccn........",
    "ccccccccccccccc.........",
    ".ccccccccccccc..........",
];

/// Blue and red knights gallop toward each other with couched lances; the
/// lances meet and splinter at 100%.
struct Jousting;
impl ProgressStyle for Jousting {
    fn name(&self) -> &str {
        "jousting"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "Two armoured knights on caparisoned horses gallop toward each other with striped lances couched; they close with progress and the lances splinter on impact at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let lance = (wi * 15 / 88).max(2);
        let start = -10;
        let meet = wi / 2 - 24 - lance;
        let xl = start + (p * (meet - start) as f32) as i32;

        for (side, team) in [(false, BLUE), (true, RED)] {
            let x0 = if side { wi - 24 - xl } else { xl };
            let stride = (t * 7.0 + if side { 1.3 } else { 0.0 }).sin();
            let bob = i32::from(stride > 0.0);
            // Couched lance with painted stripes (drawn first so the horse's
            // head keeps its colour where the lance crosses it).
            let (from, to) = if side {
                (x0 + 14, x0 - lance)
            } else {
                (x0 + 9, x0 + 23 + lance)
            };
            let ly = bob + 5;
            let steps = (to - from).abs();
            for k in 0..=steps {
                let x = from + if side { -k } else { k };
                let y = ly - k * 2 / steps.max(1);
                px(grid, x, y, if (x / 4) % 2 == 0 { WHITE } else { team });
            }
            sprite(grid, &KNIGHT, x0, bob, side, |ch| match ch {
                'p' | 'c' => Some(team),
                'h' | 'a' => Some(STEEL),
                'e' | 'n' => Some(WOOD),
                _ => None,
            });
            // Galloping legs: extended vs gathered.
            let spread = if stride > 0.0 { 2 } else { -1 };
            for (lx, dir) in [(1, -1), (3, -1), (12, 1), (14, 1)] {
                let (x, d) = if side {
                    (x0 + 23 - lx, -dir * spread)
                } else {
                    (x0 + lx, dir * spread)
                };
                seg(grid, x, bob + 12, x + d, hi - 1, WOOD_DK);
            }
        }

        // Impact: splinters burst where the lances meet.
        if ctx.progress >= 0.99 {
            let r = (t * 2.0).fract() * 7.0;
            for k in 0..8 {
                let a = k as f32 * PI / 4.0 + 0.3;
                let x = wi / 2 + (a.cos() * r * 1.4) as i32;
                let y = 3 + (a.sin() * r * 0.6) as i32;
                px(grid, x, y, if k % 2 == 0 { GOLD } else { WHITE });
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7 — Drawbridge: the bridge swings down on chains across the moat.
// ---------------------------------------------------------------------------

/// A crenellated gatehouse beside a moat; the drawbridge rotates from upright
/// (closed) to flat across the water (open) as progress grows.
struct DrawbridgeLower;
impl ProgressStyle for DrawbridgeLower {
    fn name(&self) -> &str {
        "drawbridge"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A wooden drawbridge lowers on iron chains from a crenellated gatehouse, swinging from upright to flat across a rippling moat to the far bank"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let gy = hi - 1;
        let tx = wi * 64 / 100;
        let top = hi * 5 / 16;
        let mh = (hi * 2 / 16).max(1);

        // Gatehouse with merlons and arrow slits.
        for x in tx..wi {
            let merlon = (x - tx) % 4 < 2;
            let y0 = if merlon { top - mh } else { top };
            for y in y0..=gy {
                let slit = (x == tx + 8 || x == tx + 20) && y >= top + 2 && y <= top + 4;
                if !slit {
                    px(grid, x, y, STONE);
                }
            }
        }
        pennant(
            grid,
            tx + (wi - tx) / 2,
            0,
            top - mh - 1,
            t,
            ctx.palette.end,
        );

        // Far bank and moat.
        let bl = (hi * 10 / 16).max(2);
        let hy = gy - 3;
        let mx0 = tx - bl - 1;
        block(grid, 0, hy + 2, mx0 - 1, gy, EARTH);
        block(grid, 0, hy + 1, mx0 - 1, hy + 1, GRASS);
        for x in mx0.max(0)..tx {
            let crest = ((x as f32 * 0.7 - t * 4.0).sin() > 0.4) as i32;
            for y in hy + 3 - crest..=gy {
                if (y + x) % 3 != 0 || y == hy + 3 - crest {
                    px(grid, x, y, WATER);
                }
            }
        }

        // Bridge: hinged at the gate's foot, rotating 90° toward the bank.
        let a = p * PI / 2.0;
        let ex = tx - 1 - (bl as f32 * a.sin()).round() as i32;
        let ey = hy - (bl as f32 * a.cos()).round() as i32;
        seg(grid, tx - 1, hy, ex, ey, WOOD);
        if a < PI / 4.0 {
            seg(grid, tx - 2, hy, ex - 1, ey, WOOD);
        } else {
            seg(grid, tx - 1, hy + 1, ex, ey + 1, WOOD);
        }
        // Chains from the gatehouse to the bridge tip (dotted links).
        let (cx0, cy0) = (tx, top + 1);
        let steps = (ex - cx0).abs().max((ey - cy0).abs()).max(1);
        for s in (0..=steps).step_by(2) {
            let x = cx0 + (ex - cx0) * s / steps;
            let y = cy0 + (ey - cy0) * s / steps;
            px(grid, x, y, IRON);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8 — Torch flame: wall torches light one after another.
// ---------------------------------------------------------------------------

/// A row of wall-mounted torches (wooden haft, iron bracket, pitch head);
/// progress lights them in turn, each flame flickering and throwing sparks.
struct TorchFlame;
impl ProgressStyle for TorchFlame {
    fn name(&self) -> &str {
        "torch-flame"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A row of wall torches in iron brackets lights one by one; each teardrop flame flickers yellow-orange-red and throws rising sparks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let n = (wi / 15).max(1);
        let slot = wi / n;
        let head_y = hi / 2;
        let lit = p * n as f32;
        for i in 0..n {
            let cx = i * slot + slot / 2;
            let fi = i as f32;
            // Haft, bracket and pitch-soaked head.
            block(grid, cx, head_y + 2, cx + 1, hi - 2, WOOD);
            px(grid, cx, hi - 1, WOOD);
            block(grid, cx - 2, hi * 11 / 16, cx + 3, hi * 11 / 16, IRON);
            block(grid, cx - 1, head_y, cx + 2, head_y + 1, WOOD_DK);

            let f = (lit - fi).clamp(0.0, 1.0);
            if f <= 0.0 {
                continue;
            }
            let flick = 0.85 + 0.15 * (t * 11.0 + fi * 1.7).sin() * (t * 7.3 + fi).sin();
            let fh = (head_y as f32 * (0.35 + 0.65 * f) * flick).max(1.0);
            for k in 0..fh as i32 {
                let u = k as f32 / fh;
                let hw = if u < 0.2 {
                    2.2
                } else {
                    2.8 * (1.0 - u).powf(0.8)
                };
                let sway = (t * 8.0 + fi * 2.0 + u * 3.0).sin() * u * 1.5;
                let c = if u < 0.35 {
                    FLAME_Y
                } else if u < 0.7 {
                    FLAME_O
                } else {
                    FLAME_R
                };
                let mid = cx as f32 + 0.5 + sway;
                let (xl, xr) = ((mid - hw).round() as i32, (mid + hw).round() as i32 - 1);
                block(grid, xl, head_y - 1 - k, xr.max(xl), head_y - 1 - k, c);
            }
            if f >= 1.0 {
                for s in 0..2 {
                    let ph = (t * 1.3 + fi * 0.37 + s as f32 * 0.5).fract();
                    let y = head_y - 1 - (ph * head_y as f32) as i32;
                    let x = cx + ((t * 3.0 + fi + s as f32 * 2.0).sin() * 3.0) as i32;
                    if y < head_y - 1 - fh as i32 {
                        px(grid, x, y, FLAME_O);
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9 — Potion brew: round-bottomed flasks fill with bubbling brews.
// ---------------------------------------------------------------------------

/// Corked round-bottom flasks fill one after another with coloured potions;
/// bubbles rise through the liquid and filled flasks glint.
struct PotionBrew;
impl ProgressStyle for PotionBrew {
    fn name(&self) -> &str {
        "potion-brew"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "Corked round-bottom glass flasks fill one by one with red, blue, green and violet potions; bubbles rise through the brew and full flasks glint"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let n = (wi / 18).max(1);
        let slot = wi / n;
        let r = ((hi - 4) / 2).min(slot / 2 - 2).max(1);
        let by = hi - 1 - r;
        let rf = r as f32;
        for i in 0..n {
            let cx = i * slot + slot / 2;
            let f = (p * n as f32 - i as f32).clamp(0.0, 1.0);
            let c = TINCTURES[i as usize % TINCTURES.len()];

            // Glass: round body, short neck, cork.
            for y in by - r - 1..=by + r + 1 {
                for x in cx - r - 1..=cx + r + 1 {
                    let (dx, dy) = ((x - cx) as f32, (y - by) as f32);
                    let d = (dx * dx + dy * dy).sqrt();
                    if (d - rf).abs() < 0.5 && !(dy < 0.0 && dx.abs() <= 1.0) {
                        px(grid, x, y, GLASS);
                    }
                }
            }
            seg(grid, cx - 2, 2, cx - 2, by - r, GLASS);
            seg(grid, cx + 2, 2, cx + 2, by - r, GLASS);
            block(grid, cx - 2, 0, cx + 2, 1, WOOD);
            if f <= 0.0 {
                continue;
            }

            // Liquid with a sloshing surface and bubbles left as gaps.
            let level = (by + r) as f32 - f * 2.0 * rf;
            let fi = i as f32;
            let bubbles: Vec<(i32, i32)> = (0..3)
                .map(|k| {
                    let ph = (t * 0.7 + k as f32 * 0.33 + fi * 0.21).fract();
                    let bx = cx + ((mhash_f(i as u32 * 7 + k) - 0.5) * rf) as i32;
                    let span = ((by + r) as f32 - level).max(0.0);
                    (bx, by + r - 1 - (ph * span) as i32)
                })
                .collect();
            for y in by - r..=by + r {
                for x in cx - r..=cx + r {
                    let (dx, dy) = ((x - cx) as f32, (y - by) as f32);
                    let wob = (t * 3.0 + fi + x as f32 * 0.6).sin() * 0.6;
                    if (dx * dx + dy * dy).sqrt() < rf - 0.8
                        && y as f32 >= level + wob
                        && !bubbles.contains(&(x, y))
                    {
                        px(grid, x, y, c);
                    }
                }
            }
            if f >= 1.0 && (t * 0.9 + fi * 0.3).fract() < 0.25 {
                px(grid, cx - r / 2, by - r / 2, WHITE);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10 — Treasure chest: the lid opens on a heap of glittering gold.
// ---------------------------------------------------------------------------

/// An iron-banded wooden chest with a gold lock plate; progress lifts the
/// domed lid to reveal a gold hoard; sparkles twinkle beside it.
struct TreasureChest;
impl ProgressStyle for TreasureChest {
    fn name(&self) -> &str {
        "treasure-chest"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "An iron-banded wooden treasure chest with a gold lock: the domed lid lifts with progress to reveal a glittering heap of gold, sparkles twinkling around it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let cw = (wi - 24).clamp(4, 52) / 2 * 2;
        let x0 = (wi - cw) / 2;
        let x1 = x0 + cw - 1;
        let xc = x0 + cw / 2;
        let by = hi / 2;
        let lid_h = (hi / 4).max(1);
        let lid_top = by - lid_h - (p * (by - lid_h) as f32) as i32;
        let lid_bot = by - 1 - (p * (lid_h + 1) as f32) as i32;

        // Domed lid.
        for y in lid_top..=lid_bot {
            let inset = if y == lid_top { 2 } else { 0 };
            block(grid, x0 + inset, y, x1 - inset, y, WOOD);
        }
        // Gold hoard visible between lid and body (drawn after the lid so a
        // shared cell glows gold).
        for x in x0 + 1..x1 {
            let u = (x - xc) as f32 / (cw / 2) as f32;
            let top = by - 1 - (4.0 * (1.0 - u * u)).round() as i32;
            for y in top.max(lid_bot + 1)..by {
                if y / 4 == lid_bot / 4 {
                    continue;
                }
                if mhash((x * 31 + y * 7) as u32) % 7 != 0 {
                    let tw = mhash((x * 13 + y) as u32 + (t * 6.0) as u32) % 23 == 0;
                    px(grid, x, y, if tw { WHITE } else { GOLD });
                }
            }
        }

        // Body with a plank seam.
        for y in by..hi {
            if y == by + 4 {
                continue;
            }
            block(grid, x0, y, x1, y, WOOD);
        }
        // Iron bands and gold lock plate with keyhole.
        for bx in [x0 + 4, x1 - 5] {
            block(grid, bx, lid_top, bx + 1, lid_bot, IRON);
            block(grid, bx, by, bx + 1, hi - 1, IRON);
        }
        block(grid, xc - 2, by, xc + 1, by + 3, GOLD);

        // Sparkles on either side once the chest is open.
        let cycle = (t * 2.0) as u32;
        for k in 0..6u32 {
            if mhash_f(k * 5 + cycle) > p {
                continue;
            }
            let left = k % 2 == 0;
            let span = (x0 - 4).max(1) as f32;
            let off = (mhash_f(k + cycle * 3) * span) as i32;
            let x = if left { 1 + off } else { x1 + 3 + off };
            let y = 2 + (mhash_f(k * 3 + cycle) * (hi - 5) as f32) as i32;
            px(grid, x, y, GOLD);
            px(grid, x - 1, y, GOLD);
            px(grid, x + 1, y, GOLD);
            px(grid, x, y - 1, GOLD);
            px(grid, x, y + 1, GOLD);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11 — Scroll unroll: parchment unrolls between two wooden-capped rolls.
// ---------------------------------------------------------------------------

/// The right-hand roll travels with progress, unfurling parchment covered in
/// lines of script; the moving roll turns as it goes.
struct ScrollUnroll;
impl ProgressStyle for ScrollUnroll {
    fn name(&self) -> &str {
        "scroll-unroll"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A parchment scroll unrolls between two rolls with wooden end-caps, revealing lines of handwritten script; the travelling roll turns as it goes"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let lx = 1;
        let rx = lx + 5 + (p * (wi - 11).max(0) as f32) as i32;

        // Parchment with script as negative-space words.
        let (top, bot) = (2, hi - 3);
        for y in top..=bot {
            let text_row = (y - 4) % 3 == 0 && y >= 4 && y <= bot - 2;
            let line = (y / 3) as u32;
            let mut word_end = lx + 6;
            let mut in_word = false;
            for x in lx + 4..rx {
                if text_row && x >= lx + 7 && x < rx - 2 {
                    if x >= word_end {
                        in_word = !in_word;
                        let len = if in_word {
                            2 + (mhash(line * 977 + x as u32) % 6) as i32
                        } else {
                            2
                        };
                        word_end = x + len;
                    }
                    if in_word {
                        continue;
                    }
                }
                px(grid, x, y, PARCH);
            }
        }

        // Rolls: parchment cylinders with wooden end-caps.
        let spin = (ctx.time * 3.0) as i32;
        for (i, x) in [lx, rx].into_iter().enumerate() {
            for xx in x..x + 4 {
                let seam = i == 1 && (xx - x) == spin.rem_euclid(4);
                for y in 1..hi - 1 {
                    if !seam || y % 2 == 0 {
                        px(grid, xx, y, PARCH_DK);
                    }
                }
            }
            block(grid, x - 1, 0, x + 4, 0, WOOD_DK);
            block(grid, x - 1, hi - 1, x + 4, hi - 1, WOOD_DK);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12 — Portcullis raise: the iron gate lifts out of a stone archway.
// ---------------------------------------------------------------------------

/// A crenellated gatehouse with an arched opening; the spiked iron
/// portcullis lattice rattles upward into the wall as progress grows.
struct PortcullisRaise;
impl ProgressStyle for PortcullisRaise {
    fn name(&self) -> &str {
        "portcullis"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A spiked iron portcullis lattice rattles up into a crenellated stone gatehouse, clearing the arched gateway; torches flicker either side"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;
        let gy = hi - 1;
        let gl = wi * 30 / 100;
        let gr = wi - gl;
        let cx = wi / 2;
        let half = ((gr - gl) / 2).max(1) as f32;
        let wall_top = (hi * 2 / 16).max(1);
        let arch_top = hi * 5 / 16;
        let open_top = |x: i32| {
            let u = ((x - cx) as f32 + 0.5) / half;
            arch_top + (4.0 * (1.0 - (1.0 - u * u).max(0.0).sqrt())).round() as i32
        };
        let torches = [gl - 5, gr + 4];
        let niche = |x: i32, y: i32| {
            torches
                .iter()
                .any(|&tx| (x - tx).abs() <= 2 && y >= hi * 5 / 16 && y <= hi * 10 / 16)
        };

        // Stone gatehouse with merlons, arched opening and torch niches.
        for x in 0..wi {
            for y in 0..=gy {
                if y < wall_top && x % 6 >= 3 {
                    continue;
                }
                if x >= gl && x < gr && y >= open_top(x) {
                    continue;
                }
                if niche(x, y) {
                    continue;
                }
                px(grid, x, y, STONE);
            }
        }
        for (k, &tx) in torches.iter().enumerate() {
            let fl = (t * 9.0 + k as f32 * 2.1).sin();
            block(grid, tx, hi * 8 / 16, tx, hi * 10 / 16, WOOD);
            let fh = 2 + i32::from(fl > 0.0);
            block(
                grid,
                tx - 1,
                hi * 8 / 16 - fh,
                tx + 1,
                hi * 8 / 16 - 1,
                FLAME_O,
            );
            px(grid, tx, hi * 8 / 16 - fh - 1, FLAME_Y);
        }

        // Portcullis: lattice bottom rises from the ground into the arch.
        let rattle = if p > 0.0 && p < 1.0 {
            ((t * 14.0).sin() * 0.6).round() as i32
        } else {
            0
        };
        let yb = gy - (p * (gy - arch_top + 2) as f32) as i32 + rattle;
        for x in gl + 1..gr - 1 {
            let top = open_top(x);
            let bar = (x - gl - 2).rem_euclid(4) == 0;
            for y in top..=yb {
                let rung = (yb - y).rem_euclid(4) == 2;
                if bar || rung {
                    px(grid, x, y, IRON);
                }
            }
            if bar && yb + 1 >= top {
                px(grid, x, yb + 1, STEEL);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 13 — Anvil forge: a hammer beats a glowing blade out across an anvil.
// ---------------------------------------------------------------------------

/// A blacksmith's anvil (horn, face, waist, flared base); a red-hot blade on
/// its face lengthens with progress while a hammer rises and strikes the
/// glowing tip, throwing sparks on every blow.
struct AnvilForge;
impl ProgressStyle for AnvilForge {
    fn name(&self) -> &str {
        "anvil-forge"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A blacksmith's hammer rises and strikes a red-hot blade on an iron anvil, sparks flying; the blade is beaten longer with progress, cooling to steel behind the glowing tip"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let sx = |v: i32| v * wi / 88;
        let sy = |v: i32| v * hi / 16;

        // Anvil: horn tapering left, flat face, narrow waist, flared feet.
        let face = sy(10);
        block(grid, sx(10), face, sx(40), face + 1, IRON);
        seg(grid, sx(2), face, sx(10), face, IRON);
        seg(grid, sx(5), face + 1, sx(10), face + 1, IRON);
        seg(grid, sx(8), face + 2, sx(14), face + 2, IRON);
        block(grid, sx(36), face + 2, sx(40), face + 2, IRON);
        block(grid, sx(18), face + 2, sx(32), sy(13), IRON);
        block(grid, sx(15), sy(14), sx(35), sy(14), IRON);
        block(grid, sx(12), sy(15), sx(38), sy(15), IRON);

        // Blade on the face, growing rightward; the tip is still glowing.
        let b0 = sx(12);
        let b1 = sx(26) + (p * (wi - sx(26) - 4).max(0) as f32) as i32;
        let by = face - 2;
        for x in b0..=b1 {
            let heat = b1 - x;
            let c = if heat < 4 {
                FLAME_Y
            } else if heat < 10 {
                FLAME_O
            } else if heat < 14 {
                FLAME_R
            } else {
                STEEL
            };
            if heat < 2 {
                px(grid, x, by + 1, c);
            } else {
                block(grid, x, by, x, by + 1, c);
            }
        }

        // Hammer: slow lift, fast blow onto the hot tip.
        let f = (ctx.time * 1.4).fract();
        let lift = if f < 0.75 {
            (f / 0.75).sqrt() * sy(4) as f32
        } else {
            (1.0 - (f - 0.75) / 0.25) * sy(4) as f32
        };
        let hx = b1 - 5;
        let hy = by - 1 - lift.round() as i32;
        seg(grid, hx + 2, hy - 1, hx + 10, hy - 3, WOOD);
        block(grid, hx - 2, hy - 2, hx + 2, hy, STEEL);

        // Sparks spray from the strike for a moment after each blow.
        if f < 0.18 {
            let r = f / 0.18 * 9.0;
            for k in 0..7 {
                let a = PI * (0.1 + 0.8 * k as f32 / 6.0);
                let x = hx + (a.cos() * r * 1.6) as i32;
                let y = by - (a.sin() * r * 0.7) as i32;
                px(grid, x, y, if k % 2 == 0 { FLAME_Y } else { FLAME_O });
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 14 — Banner unfurl: a swallow-tailed heraldic banner streams from its pole.
// ---------------------------------------------------------------------------

/// A tall pole with a gold finial; a red banner with a gold fess unfurls
/// rightward with progress, rippling in the wind, its end cut swallow-tail.
struct BannerUnfurl;
impl ProgressStyle for BannerUnfurl {
    fn name(&self) -> &str {
        "banner-unfurl"
    }
    fn theme(&self) -> &str {
        "medieval"
    }
    fn describe(&self) -> &str {
        "A red heraldic banner with a gold stripe unfurls from a tall pole with a gold finial, streaming further with progress and rippling in the wind, its end cut swallow-tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let t = ctx.time;

        // Pole and finial.
        block(grid, 2, 2, 3, hi - 1, WOOD_DK);
        block(grid, 1, 0, 4, 1, GOLD);

        // Cloth: rows [top, bot] relative to the pole, rippling more toward
        // the free end; the last few columns are cut into a swallow-tail.
        let (top, bot) = (2, (hi - 4).max(2));
        let mid = (top + bot) / 2;
        let len = 3 + (p * (wi - 9).max(0) as f32) as i32;
        let tail = (len / 5).clamp(1, 6);
        for i in 0..len {
            let x = 4 + i;
            let reach = i as f32 / (wi - 9).max(1) as f32;
            let wave = ((i as f32 * 0.22 - t * 5.0).sin() * (0.4 + 1.6 * reach)).round() as i32;
            let notch = (i - (len - tail)).max(0) * (bot - top) / (2 * tail);
            for v in top..=bot {
                if (v - mid).abs() < notch {
                    continue;
                }
                let fess = (v - mid).abs() <= (bot - top) / 6;
                px(grid, x, v + wave, if fess { GOLD } else { RED });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{BarContext, Easing, Palette};
    use super::*;
    use crate::BrailleGrid;

    #[test]
    fn medieval_styles_render_all_sizes() {
        let sizes = [(1usize, 1usize), (2, 1), (50, 4), (80, 1), (10, 8)];
        let progresses = [0.0f32, 0.001, 0.5, 0.999, 1.0];
        let times = [0.0f32, 1.5, 3.7, 100.0];
        for style in styles() {
            assert!(!style.name().is_empty(), "empty name");
            assert_eq!(
                style.theme(),
                "medieval",
                "wrong theme for {}",
                style.name()
            );
            for &(w, h) in &sizes {
                for &p in &progresses {
                    for &t in &times {
                        let mut grid = BrailleGrid::new(w, h).unwrap();
                        let ctx = BarContext::new(p, t, w, h)
                            .with_easing(Easing::CubicInOut)
                            .with_palette(Palette::default());
                        style.render(&mut grid, &ctx).unwrap_or_else(|e| {
                            panic!("{} failed at {w}x{h} p={p} t={t}: {e}", style.name())
                        });
                    }
                }
            }
        }
    }
}
