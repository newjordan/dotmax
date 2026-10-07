//! Mythology-themed progress bars — fourteen animated braille styles drawn from
//! Greek, Norse, and world mythology.
//!
//! Each bar is structurally unique: distinct geometry, motion, and symbol
//! approach. Color alone is never the only differentiator. Every bar animates
//! with `ctx.time` and advances with `ctx.eased` progress.
//!
//! Styles:
//! 1. **phoenix-rising**   — raised-wing firebird climbs from embers, flame tail flickers
//! 2. **dragon-flight**    — horned bat-winged dragon flaps across breathing fire
//! 3. **hydra-heads**      — body sprouts swaying necks, up to nine snapping heads
//! 4. **kraken-depths**    — goggle-eyed squid surfaces and swims toward a ship
//! 5. **pegasus-gallop**   — white winged horse gallops over a stardust trail
//! 6. **medusa-gaze**      — snake-haired face whose gaze petrifies a line of warriors
//! 7. **minotaur-labyrinth** — red thread winds through a walled maze to the Minotaur
//! 8. **icarus-sun**       — winged figure climbs toward a blazing sun, shedding feathers
//! 9. **cerberus-watch**   — three-headed hound on a chain, heads bobbing and snapping
//! 10. **valkyrie-ride**   — winged-helmed rider on a galloping steed, spear levelled
//! 11. **yggdrasil-grows** — world tree grows: trunk, forked branches, crown, roots
//! 12. **ouroboros**       — scaled serpent coils a rotating ring until it bites its tail
//! 13. **trojan-horse**    — wheeled wooden horse rolls up to the gates of Troy
//! 14. **mjolnir-strike**  — Thor's hammer slams a growing rune road, lightning flies

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `mythology` theme.
///
/// Returns fourteen boxed mythology-themed bars. Each is structurally distinct
/// (geometry, motion, and symbol); color alone never separates them.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PhoenixRising),
        Box::new(DragonFlight),
        Box::new(HydraHeads),
        Box::new(KrakenDepths),
        Box::new(PegasusGallop),
        Box::new(MedusaGaze),
        Box::new(MinotaurLabyrinth),
        Box::new(IcarusSun),
        Box::new(CerberusWatch),
        Box::new(ValkyriRide),
        Box::new(YggdrasilGrows),
        Box::new(Ouroboros),
        Box::new(TrojanHorse),
        Box::new(MjolnirStrike),
    ]
}

// ---------------------------------------------------------------------------
// Shared sprite helpers
// ---------------------------------------------------------------------------
// The pictorial bars below are drawn from small ASCII sprites (`#` = dot) so
// each creature keeps a recognisable silhouette at 88x16 dots. All writes go
// through `draw`, so off-grid sprite dots are silently clipped.

/// Stamp an ASCII sprite with its top-left corner at dot `(x, y)`.
fn stamp(grid: &mut BrailleGrid, rows: &[&str], x: i32, y: i32) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            if c == b'#' {
                draw::dot_i(grid, x + dx as i32, y + dy as i32);
            }
        }
    }
}

/// Straight dot line from `(x0, y0)` to `(x1, y1)` (inclusive).
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for s in 0..=steps {
        let x = x0 + (x1 - x0) * s / steps;
        let y = y0 + (y1 - y0) * s / steps;
        draw::dot_i(grid, x, y);
    }
}

/// Tint every cell overlapping the dot rectangle `[x, x+w) × [y, y+h)`.
fn tint_box(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32, color: Color) {
    let (x1, y1) = (x + w - 1, y + h - 1);
    if w <= 0 || h <= 0 || x1 < 0 || y1 < 0 {
        return;
    }
    let (cx0, cx1) = ((x.max(0) / 2) as usize, (x1 / 2) as usize);
    for cy in (y.max(0) / 4)..=(y1 / 4) {
        draw::tint_row(grid, cy as usize, cx0, cx1, color);
    }
}

/// Fallback for grids too small to hold a sprite: a plain fill line.
fn tiny_fill(grid: &mut BrailleGrid, ctx: &BarContext, w: usize, h: usize) {
    if w == 0 || h == 0 {
        return;
    }
    let filled = (ctx.eased * w as f32).round() as usize;
    if filled > 0 {
        draw::hline(grid, 0, filled.min(w) - 1, h / 2);
    }
}

/// Side-on horse facing right, legs stretched (gallop frame A).
const HORSE_A: [&str; 13] = [
    "..................#.#...",
    "..................####..",
    ".................#######",
    "................####.###",
    "...............####...##",
    "..##..........####......",
    ".####.###########.......",
    "##..#############.......",
    "#...#############.......",
    "....#############.......",
    "....##.........##.......",
    "...##...........##......",
    "..##.............##.....",
];

/// The same horse with legs gathered under the body (gallop frame B).
const HORSE_B: [&str; 13] = [
    "..................#.#...",
    "..................####..",
    ".................#######",
    "................####.###",
    "...............####...##",
    "..##..........####......",
    ".####.###########.......",
    "#...#############.......",
    "#...#############.......",
    "....#############.......",
    ".....#.#......#.#.......",
    ".....#..#.....#..#......",
    "......#..#....#...#.....",
];

// ---------------------------------------------------------------------------
// 1. Phoenix Rising
// ---------------------------------------------------------------------------
// A heraldic firebird — raised wings, crested head, fanned tail — climbs from
// the left on a rising diagonal as progress advances. Its wings beat between
// two frames, flame licks off its tail, and a bed of embers glows beneath the
// ground it has already crossed.

const PHOENIX_UP: [&str; 12] = [
    "#....................#",
    "##........##........##",
    ".###......##......###.",
    ".####....####....####.",
    "..#####..####..#####..",
    "...######.##.######...",
    "....##############....",
    "......##########......",
    "........######........",
    ".......#.####.#.......",
    "......#..#..#..#......",
    ".....#..#....#..#.....",
];

const PHOENIX_DOWN: [&str; 12] = [
    "......................",
    "..........##..........",
    "..........##..........",
    ".........####.........",
    "##.......####.......##",
    ".#####....##....#####.",
    "...##############.....",
    "......##########......",
    "........######........",
    ".......#.####.#.......",
    "......#..#..#..#......",
    ".....#..#....#..#.....",
];

struct PhoenixRising;
impl ProgressStyle for PhoenixRising {
    fn name(&self) -> &str {
        "phoenix-rising"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A crested firebird beats its raised wings and climbs from the embers, trailing a flickering flame tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 14 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;

        // Bird climbs a diagonal: bottom-left at 0 %, upper-right at 100 %.
        let bx = (p * (wi - 22) as f32).round() as i32;
        let lift = (p * (hi - 14) as f32).round() as i32;
        let by = (hi - 14 - lift).max(0);
        let beat = ((ctx.time * 3.0) as i32).rem_euclid(2) == 0;
        stamp(grid, if beat { &PHOENIX_UP } else { &PHOENIX_DOWN }, bx, by);

        // Flame tail: flickering tongues dropping from the tail fan.
        let tail_len = 2 + ((ctx.time * 9.0).sin() * 1.5).round() as i32;
        for k in 0..5 {
            let fx = bx + 7 + k * 2;
            let len = tail_len + (k % 2) + ((ctx.time * 11.0 + k as f32).sin() > 0.0) as i32;
            for d in 0..len {
                draw::dot_i(grid, fx + ((d + k) % 2), by + 12 + d);
            }
        }

        // Ember bed along the ground the bird has crossed.
        let ground = hi - 1;
        for x in 0..(bx + 11).min(wi) {
            let flick = ((x as f32 * 1.7 + ctx.time * 10.0).sin() * 1.3).round() as i32;
            draw::dot_i(grid, x, ground);
            if flick > 0 {
                draw::dot_i(grid, x, ground - flick);
            }
        }

        // Colour: embers red, body orange, wing tips and crest gold.
        tint_box(grid, 0, hi - 4, wi, 4, Color::rgb(200, 50, 20));
        tint_box(grid, bx, by + 8, 22, 8, Color::rgb(255, 90, 20));
        tint_box(grid, bx, by + 4, 22, 4, Color::rgb(255, 150, 30));
        tint_box(grid, bx, by, 22, 4, Color::rgb(255, 215, 80));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Dragon Flight
// ---------------------------------------------------------------------------
// A side-on dragon — horned head, long neck, bat wing, clawed legs, whip
// tail — flies left→right with progress. The wing flaps between two frames,
// a cone of fire flickers from its jaws, and the ground it has passed over is
// left scorched.

const DRAGON: [&str; 12] = [
    "............................#....",
    "...........................###...",
    "..........................######.",
    ".........................####.###",
    "........................####.....",
    "........###############.####.....",
    "##....##################.........",
    ".##..##################..........",
    "..#######################........",
    "........#####....#####...........",
    "........#.#.#....#.#.#...........",
    ".......#..#..#..#..#..#..........",
];

/// Scalloped bat wing, raised (three finger spines along the top edge).
const BAT_WING_UP: [&str; 8] = [
    "#.....#.....#.........",
    "##....##....##........",
    ".##..####..####.......",
    ".###.#####.#####......",
    "..###############.....",
    "...###############....",
    ".....##############...",
    "........############..",
];

/// The same wing on the downstroke: flattened and swept back.
const BAT_WING_DOWN: [&str; 5] = [
    "#.....#.....#.........",
    "###..###...####.......",
    ".######..##########...",
    "..##################..",
    "....#################.",
];

struct DragonFlight;
impl ProgressStyle for DragonFlight {
    fn name(&self) -> &str {
        "dragon-flight"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A horned, bat-winged dragon flaps across breathing a flickering cone of fire over scorched ground"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 41 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Body travels with progress, leaving room for the fire cone.
        let dx = (ctx.eased * (wi - 41).max(0) as f32).round() as i32;
        let up = ((ctx.time * 3.0) as i32).rem_euclid(2) == 0;
        let dy = if up { 3 } else { 2 };
        stamp(grid, &DRAGON, dx, dy);
        if up {
            stamp(grid, &BAT_WING_UP, dx + 4, dy - 3);
        } else {
            stamp(grid, &BAT_WING_DOWN, dx + 4, dy + 1);
        }

        // Fire cone from the jaws: widens with distance, dots flicker.
        let (mx, my) = (dx + 33, dy + 3);
        let tick = (ctx.time * 12.0) as i32;
        for d in 0..8 {
            let spread = d / 2;
            for s in -spread..=spread {
                if (d * 7 + s * 13 + tick).rem_euclid(3) != 0 {
                    draw::dot_i(grid, mx + d, my + s);
                }
            }
        }

        // Scorched ground behind the dragon: a smouldering, flickering line.
        let ground = hi - 1;
        for x in 0..(dx + 16).min(wi) {
            if (x * 5 + tick).rem_euclid(7) != 0 {
                draw::dot_i(grid, x, ground);
            }
        }

        // Colour: body from the palette, fire gold-orange, scorch red.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.2));
        tint_box(grid, 0, hi - 2, dx + 16, 2, Color::rgb(190, 50, 20));
        tint_box(grid, mx, my - 4, 8, 9, Color::rgb(255, 150, 30));
        tint_box(grid, mx + 4, my - 4, 4, 9, Color::rgb(255, 220, 90));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Hydra Heads
// ---------------------------------------------------------------------------
// A many-headed serpent: a scaly body mound sits on the ground and a new
// neck sprouts for each step of progress (one head at 0 %, nine at 100 %).
// Each neck sways with time and ends in a horned dragon head whose jaws snap
// open and shut.

const HYDRA_HEAD_OPEN: [&str; 4] = [".##..", "#####", "##...", ".####"];
const HYDRA_HEAD_SHUT: [&str; 4] = [".##..", "#####", "#####", "....."];

struct HydraHeads;
impl ProgressStyle for HydraHeads {
    fn name(&self) -> &str {
        "hydra-heads"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A serpent body sprouts a new swaying neck with progress, up to nine snapping dragon heads"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 20 || h < 12 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let heads = 1 + (ctx.eased * 8.0).round() as i32;

        // Body: a long mound along the ground under all the necks.
        let body_w = 8 + (heads - 1) * (wi - 14) / 8;
        let bx = 2;
        for x in 0..body_w {
            let t = x as f32 / body_w.max(1) as f32;
            let hump = ((t * PI).sin() * 3.0).round() as i32 + 1;
            for y in 0..=hump {
                draw::dot_i(grid, bx + x, hi - 1 - y);
            }
        }

        // Necks: one per head, rising from the body and swaying with time.
        let gap = (body_w - 6).max(1) / (heads - 1).max(1);
        for i in 0..heads {
            let root = bx + 3 + i * gap;
            let neck_top = 3 + (i % 3);
            let mut end = (root, hi - 4);
            for y in (neck_top..=hi - 4).rev() {
                let d = (hi - 4 - y) as f32;
                let sway = (ctx.time * 2.5 + i as f32 * 1.3 + d * 0.3).sin() * d * 0.25;
                let x = root + sway.round() as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
                end = (x, y);
            }
            let open = ((ctx.time * 3.0 + i as f32 * 0.7) as i32).rem_euclid(2) == 0;
            let head = if open {
                &HYDRA_HEAD_OPEN
            } else {
                &HYDRA_HEAD_SHUT
            };
            stamp(grid, head, end.0, end.1 - 3);
        }

        // Colour: body and necks in the palette, heads a venomous green.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.3));
        tint_box(grid, 0, 0, wi, 8, Color::rgb(110, 220, 110));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Kraken Depths
// ---------------------------------------------------------------------------
// A giant squid surfaces and swims toward a ship at the far right as progress
// advances: a domed mantle with two staring eyes breaks the rolling waterline
// while a fan of tentacles undulates beneath it. At 100 % it reaches the hull.

const SHIP: [&str; 8] = [
    ".....#....",
    ".....##...",
    ".....###..",
    ".....####.",
    ".....#....",
    "##########",
    ".########.",
    "..######..",
];

struct KrakenDepths;
impl ProgressStyle for KrakenDepths {
    fn name(&self) -> &str {
        "kraken-depths"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A goggle-eyed kraken surfaces and swims toward a ship, tentacles writhing below the rolling waterline"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 36 || h < 15 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = 6;

        // Ship riding the waterline at the right edge.
        let ship_x = wi - 10;
        stamp(grid, &SHIP, ship_x, sea - 5);

        // Kraken position: centre of the mantle travels toward the ship.
        let kx = 7 + (ctx.eased * (wi - 26) as f32).round() as i32;
        let bob = ((ctx.time * 2.0).sin() * 0.8).round() as i32;
        let top = 1 + bob;

        // Mantle: a dome 13 dots wide, with two eye holes left unlit.
        for dy in 0..7i32 {
            let half = match dy {
                0 => 2,
                1 => 4,
                2 => 5,
                _ => 6,
            };
            for dx in -half..=half {
                let eye = dy == 4 && (dx == -3 || dx == 3);
                if !eye {
                    draw::dot_i(grid, kx + dx, top + dy);
                }
            }
        }

        // Tentacles: eight curling ribbons hanging from the mantle.
        for i in 0..8i32 {
            let base = kx - 6 + i * 12 / 7;
            for d in 0..(hi - top - 7) {
                let sway = (ctx.time * 3.0 + d as f32 * 0.5 + i as f32 * 0.9).sin();
                let x = base + (sway * d as f32 * 0.25).round() as i32 - (d / 3);
                draw::dot_i(grid, x, top + 7 + d);
            }
        }

        // Rolling waterline across the whole width.
        for x in 0..wi {
            let crest = ((x as f32 * 0.5 + ctx.time * 4.0).sin() * 0.9).round() as i32;
            draw::dot_i(grid, x, sea + crest);
        }

        // Colour: sea from the palette, kraken crimson, ship timber.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.0));
        tint_box(grid, kx - 9, top + 7, 20, hi, Color::rgb(170, 40, 90));
        tint_box(grid, kx - 6, top, 13, 7, Color::rgb(210, 60, 100));
        tint_box(grid, ship_x, 0, 10, sea + 3, Color::rgb(200, 150, 90));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Pegasus Gallop
// ---------------------------------------------------------------------------
// A white winged horse gallops through the sky left→right with progress. Its
// legs alternate between stretched and gathered frames, its feathered wing
// beats up and down, and a twinkling stardust trail marks the distance run.

const WING_UP: [&str; 8] = [
    "#.............",
    "###...........",
    ".####.........",
    "..#####.......",
    "...#######....",
    "....########..",
    ".....#########",
    "......########",
];

const WING_DOWN: [&str; 4] = [
    "######........",
    ".##########...",
    "...###########",
    ".....########.",
];

struct PegasusGallop;
impl ProgressStyle for PegasusGallop {
    fn name(&self) -> &str {
        "pegasus-gallop"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A white winged horse gallops across the sky, legs and feathered wings beating, over a stardust trail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 26 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        let x = (ctx.eased * (wi - 25) as f32).round() as i32;
        let stride = ((ctx.time * 6.0) as i32).rem_euclid(2) == 0;
        let y = if stride { 3 } else { 2 };
        stamp(grid, if stride { &HORSE_A } else { &HORSE_B }, x, y);
        // The wing roots at the withers and beats between two frames.
        if ((ctx.time * 3.0) as i32).rem_euclid(2) == 0 {
            stamp(grid, &WING_UP, x + 1, y - 3);
        } else {
            stamp(grid, &WING_DOWN, x, y + 3);
        }

        // Stardust trail: twinkling dots strewn behind the hooves.
        let tick = (ctx.time * 8.0) as i32;
        for tx in 0..x.min(wi) {
            let row = hi - 2 - (tx * 7).rem_euclid(4);
            if (tx * 3 + tick).rem_euclid(5) == 0 || tx % 2 == 0 && row == hi - 2 {
                draw::dot_i(grid, tx, row);
            }
        }

        // Colour: trail from the palette, the horse bright white, wing silver.
        tint_box(grid, 0, hi - 6, wi, 6, ctx.palette.sample(0.4));
        tint_box(grid, x, y, 25, 13, Color::rgb(240, 245, 255));
        tint_box(grid, x, y - 3, 12, 6, Color::rgb(190, 215, 255));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Medusa Gaze
// ---------------------------------------------------------------------------
// Medusa's face sits at the left, crowned by writhing snakes. A golden gaze
// beam shoots from her eyes and lengthens with progress; every spear-carrying
// warrior it has passed is turned to grey stone, while those still ahead of
// it march in place, bronze and alive.

const WARRIOR: [&str; 7] = [
    ".##.#", ".##.#", "#####", ".##.#", ".##.#", ".#.##", "#...#",
];

struct MedusaGaze;
impl ProgressStyle for MedusaGaze {
    fn name(&self) -> &str {
        "medusa-gaze"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Snake-haired Medusa stares down a line of warriors; her lengthening gaze turns each one to grey stone"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 30 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let (fx, fy) = (9, 7);

        // Face: an oval with two eye holes and a mouth slit left unlit.
        for dy in -5..=5i32 {
            for dx in -4..=4i32 {
                let eye = dy == -1 && (dx == -2 || dx == 2);
                let mouth = dy == 3 && dx.abs() <= 1;
                if dx * dx * 25 + dy * dy * 16 <= 400 && !eye && !mouth {
                    draw::dot_i(grid, fx + dx, fy + dy);
                }
            }
        }

        // Snake hair: seven serpents radiating from the crown, each swaying.
        for i in 0..7 {
            let base = -PI - 0.3 + i as f32 * (PI + 0.6) / 6.0;
            let len = 6 + (i % 2) * 2;
            let mut end = (fx, fy);
            for d in 0..len {
                let r = 5.0 + d as f32;
                let a = base + (ctx.time * 4.0 + d as f32 * 0.8 + i as f32).sin() * 0.22;
                let x = fx + (a.cos() * r * 1.4).round() as i32;
                let y = fy + (a.sin() * r * 0.9).round() as i32;
                draw::dot_i(grid, x, y);
                end = (x, y);
            }
            draw::dot_i(grid, end.0 + 1, end.1);
            draw::dot_i(grid, end.0, end.1 - 1);
        }

        // Gaze beam: two scrolling rays from the eyes, length = progress.
        let start = fx + 4;
        let tip = start + (ctx.eased * (wi - start) as f32).round() as i32;
        let scroll = (ctx.time * 24.0) as i32;
        for x in start..tip.min(wi) {
            if (x - scroll).rem_euclid(4) != 0 {
                draw::dot_i(grid, x, fy - 2);
            }
            if (x - scroll + 2).rem_euclid(4) != 0 {
                draw::dot_i(grid, x, fy - 1);
            }
        }

        // Warriors line up to the right; the living ones march in place.
        let mut wx = 24;
        let step = ((ctx.time * 4.0) as i32).rem_euclid(2);
        while wx + 5 <= wi {
            let stone = wx + 2 < tip;
            let lift = if stone { 0 } else { step };
            stamp(grid, &WARRIOR, wx, hi - 7 - lift);
            let color = if stone {
                Color::rgb(140, 140, 150)
            } else {
                Color::rgb(215, 150, 70)
            };
            tint_box(grid, wx, hi - 8, 5, 8, color);
            wx += 11;
        }

        // Colour: hair dark green, face pale green, gaze gold.
        tint_box(grid, 0, 0, 22, hi, Color::rgb(60, 170, 80));
        tint_box(grid, fx - 4, fy - 4, 9, 9, Color::rgb(150, 220, 150));
        tint_box(
            grid,
            start,
            fy - 2,
            tip - start,
            2,
            Color::rgb(255, 215, 80),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Minotaur Labyrinth
// ---------------------------------------------------------------------------
// A walled serpentine labyrinth: vertical walls with gaps alternating top and
// bottom. Ariadne's red thread unwinds from the entrance, snaking through
// each gap as progress advances, toward the horned, bull-headed Minotaur that
// paces in the inner chamber at the far end.

const MINOTAUR: [&str; 14] = [
    "#...........#",
    "##.........##",
    ".##.......##.",
    "..##.###.##..",
    "...#######...",
    "....#.#.#....",
    "....#####....",
    ".....###.....",
    "...#######...",
    "..#########..",
    "..##.###.##..",
    "..##.###.##..",
    "....##.##....",
    "....#...#....",
];

struct MinotaurLabyrinth;
impl ProgressStyle for MinotaurLabyrinth {
    fn name(&self) -> &str {
        "minotaur-labyrinth"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Ariadne's red thread winds through a walled labyrinth toward the horned Minotaur in its inner chamber"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 36 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = hi / 2;

        // Outer walls, with the entrance gap on the left.
        draw::hline(grid, 0, w - 1, 0);
        draw::hline(grid, 0, w - 1, h - 1);
        draw::vline(grid, w - 1, 0, h - 1);
        draw::vline(grid, 0, 0, (mid - 3) as usize);
        draw::vline(grid, 0, (mid + 3) as usize, h - 1);

        // Inner walls every 8 dots, gaps alternating top / bottom.
        let walls = (wi - 18) / 8;
        for k in 1..=walls {
            let x = (k * 8) as usize;
            if k % 2 == 1 {
                draw::vline(grid, x, 5, h - 1);
            } else {
                draw::vline(grid, x, 0, h - 6);
            }
        }

        // Ariadne's thread: a square-wave path through every gap, ending at
        // the chamber. Walk its polyline and light the first `eased` share.
        let mut pts = vec![(0, mid), (4, mid)];
        for k in 1..=walls {
            let gy = if k % 2 == 1 { 2 } else { hi - 3 };
            pts.push((k * 8 - 4, gy));
            pts.push((k * 8 + 4, gy));
        }
        pts.push((walls * 8 + 4, mid));
        pts.push((wi - 16, mid));
        let total: i32 = pts
            .windows(2)
            .map(|s| (s[1].0 - s[0].0).abs() + (s[1].1 - s[0].1).abs())
            .sum();
        let mut left = (ctx.eased * total as f32).round() as i32;
        let mut lit = Vec::new();
        for s in pts.windows(2) {
            let ((x0, y0), (x1, y1)) = (s[0], s[1]);
            let len = (x1 - x0).abs() + (y1 - y0).abs();
            let take = left.min(len);
            for d in 0..=take {
                let p = (x0 + (x1 - x0).signum() * d, y0 + (y1 - y0).signum() * d);
                draw::dot_i(grid, p.0, p.1);
                lit.push(p);
            }
            left -= take;
            if left <= 0 {
                break;
            }
        }
        // Theseus' torch: a flickering spark at the thread's tip.
        let tip = lit.last().copied().unwrap_or(pts[0]);
        if ((ctx.time * 6.0) as i32).rem_euclid(2) == 0 {
            draw::dot_i(grid, tip.0 + 1, tip.1 - 1);
            draw::dot_i(grid, tip.0 - 1, tip.1 - 1);
        }

        // The Minotaur paces in the chamber, head bobbing.
        let pace = ((ctx.time * 1.5).sin() * 2.0).round() as i32;
        let bob = ((ctx.time * 3.0) as i32).rem_euclid(2);
        let mx = wi - 15 + pace;
        stamp(grid, &MINOTAUR, mx, 1 + bob);

        // Colour: walls in the palette, thread red, Minotaur bronze, horns ivory.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.25));
        for &(x, y) in &lit {
            tint_box(grid, x, y, 1, 1, Color::rgb(235, 60, 70));
        }
        tint_box(grid, mx, 4, 13, hi - 4, Color::rgb(200, 130, 70));
        tint_box(grid, mx, 0, 13, 4, Color::rgb(240, 230, 200));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Icarus Sun
// ---------------------------------------------------------------------------
// A blazing sun with turning rays waits at the right edge. Icarus — a small
// figure with outstretched feathered wings — climbs from the sea toward it as
// progress advances. The closer he gets, the more wax melts: loose feathers
// tumble down toward the rolling waves below.

const ICARUS_UP: [&str; 8] = [
    "#...............#",
    "##......#......##",
    ".###...###...###.",
    "..####..#..####..",
    "...###########...",
    ".......###.......",
    ".......###.......",
    "......#...#......",
];

const ICARUS_DOWN: [&str; 8] = [
    ".................",
    "........#........",
    ".......###.......",
    "........#........",
    "#################",
    ".####..###..####.",
    "..##...###...##..",
    "......#...#......",
];

struct IcarusSun;
impl ProgressStyle for IcarusSun {
    fn name(&self) -> &str {
        "icarus-sun"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Winged Icarus climbs from the waves toward a blazing sun, shedding more melted feathers the closer he flies"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 40 || h < 14 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;

        // Sun: filled disc with eight rays that slowly turn.
        let (sx, sy) = (wi - 9, 6);
        for dy in -4..=4i32 {
            for dx in -5..=5i32 {
                if dx * dx * 16 + dy * dy * 25 <= 400 {
                    draw::dot_i(grid, sx + dx, sy + dy);
                }
            }
        }
        for r in 0..8 {
            let a = r as f32 * PI / 4.0 + ctx.time * 0.8;
            for d in 7..9 {
                let rx = sx + (a.cos() * d as f32 * 1.2).round() as i32;
                let ry = sy + (a.sin() * d as f32 * 0.7).round() as i32;
                draw::dot_i(grid, rx, ry);
            }
        }

        // Icarus climbs from low-left toward the sun.
        let ix = (p * (wi - 38) as f32).round() as i32;
        let iy = (hi - 10) - (p * (hi - 10) as f32).round() as i32;
        let flap = ((ctx.time * 4.0) as i32).rem_euclid(2) == 0;
        stamp(grid, if flap { &ICARUS_UP } else { &ICARUS_DOWN }, ix, iy);

        // Melting feathers: more fall as he nears the sun, drifting down-left.
        let feathers = (p * 7.0).round() as i32;
        for k in 0..feathers {
            let fall = (ctx.time * 5.0 + k as f32 * 2.3) % (hi as f32);
            let fx = ix + 2 + (k * 5) % 14 - (fall * 0.4) as i32;
            let fy = iy + 4 + fall as i32;
            if fy < hi - 2 {
                draw::dot_i(grid, fx, fy);
                draw::dot_i(grid, fx + 1, fy);
            }
        }

        // The sea: a rolling wave line along the bottom.
        for x in 0..wi {
            let wave = ((x as f32 * 0.45 - ctx.time * 3.0).sin() + 1.0).round() as i32;
            draw::dot_i(grid, x, hi - 1 - wave.min(1));
        }

        // Colour: sea blue, Icarus pale, feathers wax-gold, sun hot.
        tint_box(grid, 0, 0, wi, hi, Color::rgb(235, 215, 170));
        tint_box(grid, 0, hi - 4, wi, 4, Color::rgb(40, 120, 220));
        tint_box(grid, sx - 10, 0, 20, 16, Color::rgb(255, 200, 40));
        tint_box(grid, sx - 4, sy - 2, 8, 5, Color::rgb(255, 240, 150));
        tint_box(grid, ix, iy, 17, 8, Color::rgb(245, 235, 220));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Cerberus Watch
// ---------------------------------------------------------------------------
// The three-headed hound of the underworld pads left→right with progress on a
// chain staked at the left edge. Three pointy-eared heads on separate necks
// bob independently as they keep watch, each snapping its jaws in turn, while
// its legs alternate stride frames and its tail curls up behind.

const HOUND_A: [&str; 12] = [
    "#...................",
    ".#..................",
    ".#..................",
    "..#################.",
    "..#################.",
    "...################.",
    "...##.#.......##.#..",
    "..#....#.....#....#.",
    ".#.....#....#.....#.",
    ".#......#...#......#",
    "##......##.##......##",
    "....................",
];

const HOUND_B: [&str; 12] = [
    "#...................",
    ".#..................",
    ".#..................",
    "..#################.",
    "..#################.",
    "...################.",
    "....##.#......##.#..",
    "....#..#......#..#..",
    "....#..#......#..#..",
    "....#..#......#..#..",
    "...##.##.....##.##..",
    "....................",
];

const DOG_HEAD_SHUT: [&str; 6] = [
    ".#..#....",
    "##.##....",
    "########.",
    "##.######",
    "#########",
    ".###.....",
];
const DOG_HEAD_OPEN: [&str; 6] = [
    ".#..#....",
    "##.##....",
    "########.",
    "##.######",
    "#####....",
    ".########",
];

struct CerberusWatch;
impl ProgressStyle for CerberusWatch {
    fn name(&self) -> &str {
        "cerberus-watch"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Three-headed Cerberus pads along on its chain, each pointy-eared head bobbing and snapping as it keeps watch"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 40 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Hound body travels with progress; legs alternate stride frames.
        let x = 4 + (ctx.eased * (wi - 42) as f32).round() as i32;
        let stride = ((ctx.time * 5.0) as i32).rem_euclid(2) == 0;
        stamp(grid, if stride { &HOUND_A } else { &HOUND_B }, x, hi - 11);

        // Three necks fan out from the shoulders: one head reared high, one
        // thrust forward, one held low.
        let heads = [(16, 0), (28, 4), (22, 10)];
        let roots = [(17, 8), (18, 8), (18, 10)];
        for (i, (&(hx, hy), &(rx, ry))) in heads.iter().zip(roots.iter()).enumerate() {
            let bob =
                ((ctx.time * (2.0 + i as f32 * 0.6) + i as f32 * 2.0).sin() * 0.6).round() as i32;
            let (hx, hy) = (x + hx, hy + bob);
            line(grid, x + rx, ry, hx + 1, hy + 4);
            let open = ((ctx.time * 3.0 + i as f32 * 0.66) as i32).rem_euclid(3) == 0;
            stamp(
                grid,
                if open { &DOG_HEAD_OPEN } else { &DOG_HEAD_SHUT },
                hx,
                hy,
            );
        }

        // Stake at the left and a sagging chain to the hound's harness.
        draw::vline(grid, 0, 7, h - 1);
        draw::vline(grid, 1, 7, h - 1);
        let (cx1, cy1) = (x + 3, 9);
        let span = (cx1 - 2).max(1);
        for cx in 2..cx1 {
            let t = (cx - 2) as f32 / span as f32;
            let cy = cy1 + ((t * PI).sin() * 3.0).round() as i32;
            match cx.rem_euclid(4) {
                0 | 2 => draw::dot_i(grid, cx, cy),
                1 => {
                    draw::dot_i(grid, cx, cy - 1);
                    draw::dot_i(grid, cx, cy + 1);
                }
                _ => {}
            }
        }

        // Colour: chain steel, hound ash-brown, heads a darker brown.
        tint_box(grid, 0, 0, wi, hi, Color::rgb(170, 170, 185));
        tint_box(grid, 0, 6, 2, 10, Color::rgb(150, 100, 60));
        tint_box(grid, x, 0, 36, hi, Color::rgb(160, 110, 90));
        // Each head its own shade, so the three read apart.
        let shades = [
            Color::rgb(120, 78, 64),
            Color::rgb(205, 150, 120),
            Color::rgb(140, 92, 74),
        ];
        for (&(hx, hy), &shade) in heads.iter().zip(shades.iter()) {
            tint_box(grid, x + hx, hy, 9, 6, shade);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Valkyrie Ride
// ---------------------------------------------------------------------------
// A valkyrie in a winged helmet rides a galloping horse across the sky with
// her spear levelled ahead of her. The horse's legs alternate stride frames,
// and streaming speed lines behind her mark the distance ridden.

const VALKYRIE: [&str; 8] = [
    "#.....#", "##.#.##", "..###..", "...#...", ".#####.", "#.###.#", "..###..", "..#.#..",
];

struct ValkyriRide;
impl ProgressStyle for ValkyriRide {
    fn name(&self) -> &str {
        "valkyrie-ride"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A winged-helmed valkyrie gallops across the sky on her steed, spear levelled, trailing streaming speed lines"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 36 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        let x = (ctx.eased * (wi - 35) as f32).round() as i32;
        let stride = ((ctx.time * 6.0) as i32).rem_euclid(2) == 0;
        let y = if stride { 3 } else { 2 };
        stamp(grid, if stride { &HORSE_A } else { &HORSE_B }, x, y);

        // Rider on the horse's back, spear couched forward past its head.
        let (rx, ry) = (x + 7, y - 2);
        stamp(grid, &VALKYRIE, rx, ry);
        line(grid, rx + 6, ry + 5, x + 31, ry);
        stamp(grid, &["#..", "###", "#.."], x + 31, ry - 1);

        // Speed lines streaming behind her, scrolling with time.
        let scroll = (ctx.time * 30.0) as i32;
        for (row, phase) in [(y + 5, 0), (y + 8, 7), (y + 11, 3)] {
            for tx in 0..x.min(wi) {
                if (tx + scroll + phase).rem_euclid(9) < 5 {
                    draw::dot_i(grid, tx, row);
                }
            }
        }

        // Colour: lines from the palette, steed slate, rider gold, spear steel.
        tint_box(grid, 0, 0, wi, hi, ctx.palette.sample(0.6));
        tint_box(grid, x, y, 25, 13, Color::rgb(150, 160, 190));
        tint_box(grid, rx, ry, 7, 6, Color::rgb(250, 200, 70));
        tint_box(grid, rx + 8, ry - 1, 27, 4, Color::rgb(225, 230, 240));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Yggdrasil Grows
// ---------------------------------------------------------------------------
// The world tree grows from a sapling into a vast ash as progress advances: a
// thick trunk rises from the ground line, forked branches reach out, a broad
// dome of leaves widens over them (rustling with time), and roots spread
// down into the earth below.

struct YggdrasilGrows;
impl ProgressStyle for YggdrasilGrows {
    fn name(&self) -> &str {
        "yggdrasil-grows"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "The world tree grows from sapling to vast ash: trunk rises, branches fork, a rustling crown widens, roots spread"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 12 || h < 12 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let p = ctx.eased;
        let cx = wi / 2;
        let ground = hi - 4;

        // Trunk: grows taller and thicker with progress.
        let trunk_h = 2 + (p * (ground - 5) as f32).round() as i32;
        let top = ground - trunk_h;
        let half = if p > 0.5 { 2 } else { 1 };
        for y in top..=ground {
            for dx in -half..=half {
                draw::dot_i(grid, cx + dx, y);
            }
        }

        // Crown: a dome of leaves whose half-width grows with progress.
        let rx = 3.0 + p * (wi as f32 / 2.0 - 5.0);
        let ry = 1.5 + p * (top as f32 - 0.5).max(0.0);
        let tick = (ctx.time * 6.0) as i32;
        let crown_y = top as f32;
        for y in 0..=top {
            for x in 0..wi {
                let nx = (x - cx) as f32 / rx;
                let ny = (y as f32 - crown_y) / ry.max(1.0);
                if nx * nx + ny * ny <= 1.0 && (x * 3 + y * 5 + tick).rem_euclid(5) != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Branches: forks from the trunk top out toward the crown's edge.
        for side in [-1, 1] {
            let reach = (rx * 0.8) as i32;
            line(
                grid,
                cx,
                top + 2,
                cx + side * reach,
                top - (ry * 0.5) as i32,
            );
            line(grid, cx, top + 1, cx + side * reach / 2, top - ry as i32);
        }

        // Ground line and roots spreading below it.
        draw::hline(grid, 0, w - 1, ground as usize);
        let root_reach = (p * wi as f32 * 0.4) as i32 + 2;
        for (k, side) in [(0, -1), (1, 1), (2, -1), (3, 1)] {
            let len = root_reach - k * root_reach / 5;
            line(grid, cx, ground, cx + side * len, hi - 1 - (k % 2));
        }

        // Colour: leaves green, trunk and roots bark brown, ground earthy.
        tint_box(grid, 0, 0, wi, ground, Color::rgb(70, 200, 90));
        tint_box(grid, cx - 3, top + 1, 6, trunk_h, Color::rgb(170, 110, 60));
        tint_box(grid, 0, ground, wi, 4, Color::rgb(140, 90, 50));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Ouroboros
// ---------------------------------------------------------------------------
// A thick-bodied serpent coils around an oval ring. Its body grows head-first
// around the loop with progress — scaled with a diamond pattern, tapering to
// a pointed tail — and at 100 % its open jaws close on its own tail. The whole
// ring slowly rotates with time.

struct Ouroboros;
impl ProgressStyle for Ouroboros {
    fn name(&self) -> &str {
        "ouroboros"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "A scaled serpent coils around a rotating ring, its open jaws closing on its own tail at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 16 || h < 8 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0 - 0.5);
        let rx = w as f32 / 2.0 - 4.0;
        let ry = h as f32 / 2.0 - 2.0;

        // Body spans from the tail angle to the head angle; a gap remains
        // until progress completes. Rotation keeps it moving.
        let arc = 0.5 + ctx.eased * (2.0 * PI - 0.8);
        let rot = ctx.time * 0.5;
        let steps = (arc * rx * 1.4) as usize + 2;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let a = rot + t * arc;
            // Thickness tapers from tail (1) to body (3 rings of dots).
            let thick = if t < 0.08 {
                1
            } else if t < 0.2 {
                2
            } else {
                3
            };
            for layer in 0..thick {
                let k = layer as f32 - (thick - 1) as f32 / 2.0;
                // Diamond scale pattern: skip a lattice of dots on the body.
                if thick == 3 && layer == 1 && s % 6 == 3 {
                    continue;
                }
                let x = cx + a.cos() * (rx + k * 1.6);
                let y = cy + a.sin() * (ry + k);
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
        }

        // Head: a wedge at the leading end with open jaws and an eye.
        let ha = rot + arc;
        let (hx, hy) = (
            (cx + ha.cos() * rx).round() as i32,
            (cy + ha.sin() * ry).round() as i32,
        );
        // Tangent direction (where the head points) in dot space.
        let (tx, ty) = (-ha.sin() * rx, ha.cos() * ry);
        let len = (tx * tx + ty * ty).sqrt().max(0.01);
        let (ux, uy) = (tx / len, ty / len);
        let gape = if ctx.eased >= 1.0 {
            0.0
        } else {
            1.5 + (ctx.time * 5.0).sin()
        };
        for d in 0..5 {
            let df = d as f32;
            for side in [-1.0f32, 1.0] {
                let spread = 1.4 + df * gape * 0.3;
                let jx = hx as f32 + ux * df - uy * side * spread;
                let jy = hy as f32 + uy * df + ux * side * spread * 0.6;
                draw::dot_i(grid, jx.round() as i32, jy.round() as i32);
            }
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                draw::dot_i(grid, hx + dx, hy + dy);
            }
        }

        // Colour: body along the palette, head gold.
        let (cells_w, _) = grid.dimensions();
        for cx_c in 0..cells_w {
            let t = cx_c as f32 / cells_w.max(1) as f32;
            tint_box(grid, cx_c as i32 * 2, 0, 2, h as i32, ctx.palette.sample(t));
        }
        tint_box(grid, hx - 5, hy - 4, 11, 9, Color::rgb(255, 210, 70));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 13. Trojan Horse
// ---------------------------------------------------------------------------
// The great wooden horse, stood on a wheeled platform, is rolled left→right
// toward the walls of Troy — a crenellated wall between two towers with an
// arched gate. Its wheels turn as it moves; at 100 % it reaches the gate.

const TROY_WALL: [&str; 14] = [
    "###.#.###.......###.#.###",
    "#########.......#########",
    "#########.#.#.#.#########",
    "#########################",
    "#########################",
    "#########################",
    "#########################",
    "#########.......#########",
    "########.........########",
    "########.........########",
    "########.........########",
    "########.........########",
    "########.........########",
    "########.........########",
];

struct TrojanHorse;
impl ProgressStyle for TrojanHorse {
    fn name(&self) -> &str {
        "trojan-horse"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "The wooden horse rolls on turning wheels toward the towered walls and arched gate of Troy"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 52 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // The walls of Troy stand at the right edge.
        let troy = wi - 25;
        stamp(grid, &TROY_WALL, troy, hi - 14);

        // Horse on its platform, rolled toward the gate with progress.
        let x = (ctx.eased * (troy - 25) as f32).round() as i32;
        stamp(grid, &HORSE_B, x, 0);
        draw::hline(grid, x.max(0) as usize, (x + 22).max(0) as usize, 12);

        // Wheels: rings of six dots with a gap that turns as the horse rolls.
        let spin = (x / 2).rem_euclid(3);
        let ring = [(-2, 0), (-1, -1), (1, -1), (2, 0), (1, 1), (-1, 1)];
        for wx in [x + 4, x + 18] {
            draw::dot_i(grid, wx, 14);
            for (i, &(dx, dy)) in ring.iter().enumerate() {
                if (i as i32 + spin) % 3 != 0 {
                    draw::dot_i(grid, wx + dx, 14 + dy);
                }
            }
        }

        // Dust kicked up behind the platform, drifting with time.
        let tick = (ctx.time * 8.0) as i32;
        for d in 0..x.min(wi) {
            if (d * 5 + tick).rem_euclid(11) == 0 {
                draw::dot_i(grid, d, hi - 1 - (d + tick).rem_euclid(2));
            }
        }

        // Colour: horse timber, platform dark wood, Troy sandstone.
        tint_box(grid, 0, 0, wi, hi, Color::rgb(150, 120, 90));
        tint_box(grid, x, 0, 24, 13, Color::rgb(200, 140, 70));
        tint_box(grid, troy, 0, 25, hi, Color::rgb(220, 205, 160));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 14. Mjolnir Strike
// ---------------------------------------------------------------------------
// Thor's hammer — a broad steel head on a short leather-wrapped handle with
// a pommel loop — hovers over the leading edge of a glowing rune-stone road.
// It rises and slams down in a loop; each impact throws jagged lightning out
// to both sides. The road lengthens with progress, carrying the hammer.

const MJOLNIR: [&str; 12] = [
    "......##......",
    ".....#..#.....",
    "......##......",
    "......##......",
    "......##......",
    "......##......",
    "##############",
    "##############",
    "##.########.##",
    "##.#......#.##",
    "##############",
    "##############",
];

struct MjolnirStrike;
impl ProgressStyle for MjolnirStrike {
    fn name(&self) -> &str {
        "mjolnir-strike"
    }
    fn theme(&self) -> &str {
        "mythology"
    }
    fn describe(&self) -> &str {
        "Thor's hammer slams down at the edge of a growing rune-stone road, throwing jagged lightning on impact"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 24 || h < 16 {
            tiny_fill(grid, ctx, w, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Rune road: a solid course with carved marks above, up to the edge.
        let edge = (ctx.eased * wi as f32).round() as i32;
        for x in 0..edge.min(wi) {
            draw::dot_i(grid, x, hi - 1);
            if x % 4 != 3 && (x / 4) % 2 == 0 || x % 4 == 1 {
                draw::dot_i(grid, x, hi - 2);
            }
        }

        // Hammer: slow rise, fast slam, short rest on the road.
        let phase = (ctx.time * 1.2).rem_euclid(1.0);
        let lift = if phase < 0.6 {
            phase / 0.6 * 2.0
        } else if phase < 0.7 {
            2.0 - (phase - 0.6) / 0.1 * 2.0
        } else {
            0.0
        };
        let hx = (edge - 7).clamp(0, wi - 14);
        let hy = hi - 14 - lift.round() as i32;
        stamp(grid, &MJOLNIR, hx, hy);

        // Lightning: two jagged bolts from the head on impact.
        let impact = phase >= 0.7;
        if impact {
            let jag = ((ctx.time * 20.0) as i32).rem_euclid(2);
            for side in [-1, 1] {
                let mut bx = if side < 0 { hx - 1 } else { hx + 14 };
                let mut by = hy + 7;
                for seg in 0..7 {
                    let dy = if (seg + jag) % 2 == 0 { -1 } else { 2 };
                    for _ in 0..3 {
                        draw::dot_i(grid, bx, by);
                        bx += side;
                    }
                    by = (by + dy).clamp(0, hi - 3);
                }
            }
        }

        // Colour: road in the palette, steel head, leather handle, bolts.
        tint_box(grid, 0, hi - 4, wi, 4, ctx.palette.sample(0.1));
        if impact {
            tint_box(grid, hx - 22, hy + 4, 58, 9, Color::rgb(200, 235, 255));
        }
        tint_box(grid, hx, hy, 14, 6, Color::rgb(170, 110, 60));
        tint_box(grid, hx, hy + 6, 14, 6, Color::rgb(200, 205, 220));
        Ok(())
    }
}
