//! Game Boy / handheld LCD progress bars.
//!
//! Eleven styles evoking the washed-out dot-matrix LCD panels of classic
//! handheld gaming: Pokémon HP, Nokia Snake, Tetris GB, Dr. Mario, Kirby,
//! Tamagotchi, Game & Watch, Link's Awakening hearts, Mole Whack, Wario
//! treasure, and LCD Pinball.
//!
//! Each style draws its game's recognisable objects as monochrome dot-art,
//! like a reflective-LCD panel: a Poké Ball, bevelled tetrominoes, a pill
//! bottle, an egg-shaped Tamagotchi, heart containers, a treasure chest, an
//! LCD juggler and a pinball table. Animation is a pure function of
//! `ctx.time`; fill extent is `ctx.eased`. All writes go through `draw::`
//! helpers — out-of-bounds coordinates are silently discarded.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `gameboy` theme.
///
/// Returns one boxed implementor per style, in display order. Each struct is a
/// zero-size private unit type — no heap allocation beyond the `Box` itself.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PokemonHp),
        Box::new(NokiaSnake),
        Box::new(TetrisGb),
        Box::new(DrMario),
        Box::new(KirbyInhale),
        Box::new(Tamagotchi),
        Box::new(GameWatch),
        Box::new(HeartContainers),
        Box::new(MoleWhack),
        Box::new(WarioTreasure),
        Box::new(LcdPinball),
    ]
}

// ---------------------------------------------------------------------------
// Sprite helpers — dot-art stamps for the pictorial styles.
// ---------------------------------------------------------------------------

/// Stamp a sprite: every `#` in `rows` sets the dot at `(x + col, y + row)`.
/// Off-grid dots are dropped, so sprites may hang off any edge.
fn sprite(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + c as i32, y + r as i32);
            }
        }
    }
}

/// Triangle wave `0 → 1 → 0` with period 1.
fn tri(t: f32) -> f32 {
    1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs()
}

/// Solid block of dots from signed coordinates (clipped to the grid).
fn block(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32) {
    for yy in y..y + h {
        for xx in x..x + w {
            draw::dot_i(grid, xx, yy);
        }
    }
}

// ─── 1. Pokémon HP Bar ───────────────────────────────────────────────────────

/// A Poké Ball (solid top shell, centre band, ringed button) wobbles mid-
/// capture beside the battle HUD's `HP` bar; the wobble dies away as the bar
/// fills and the ball sits still at 100%.
struct PokemonHp;
impl ProgressStyle for PokemonHp {
    fn name(&self) -> &str {
        "pokemon-hp"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Pokémon battle HUD: a Poké Ball wobbles mid-capture beside an `HP` bar that fills with progress; the ball settles at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Poké Ball: rotate the sample point to wobble, then classify it.
        let r = ((hi - 1) as f32 / 2.0).min(7.5);
        let (cx, cy) = (r + 1.0, (hi - 1) as f32 / 2.0);
        let wobble = 0.45 * (ctx.time * 7.0).sin() * (1.0 - ctx.eased).max(0.0);
        let (s, c) = wobble.sin_cos();
        for y in 0..hi {
            for x in 0..(2.0 * r + 3.0) as i32 {
                let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                let (u, v) = (dx * c + dy * s, -dx * s + dy * c);
                let d = (u * u + v * v).sqrt();
                let on = if d > r + 0.3 {
                    false
                } else if d < 1.2 {
                    true // button
                } else if d < 2.3 {
                    false // button gap
                } else if d < 3.3 {
                    true // button ring
                } else {
                    d > r - 0.8 || v < -0.5 || v.abs() < 0.5
                };
                if on {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // "HP" label and the rounded bar outline.
        let lx = (2.0 * r + 5.0) as i32;
        let ly = hi / 2 - 3;
        sprite(
            grid,
            lx,
            ly,
            &["#.#.##.", "#.#.#.#", "###.##.", "#.#.#..", "#.#.#.."],
        );
        let bx = lx + 9;
        let bw = (wi - bx - 1).max(0);
        if bw > 4 {
            draw::hline(grid, (bx + 1) as usize, (bx + bw - 2) as usize, ly as usize);
            draw::hline(
                grid,
                (bx + 1) as usize,
                (bx + bw - 2) as usize,
                (ly + 4) as usize,
            );
            draw::vline(grid, bx as usize, (ly + 1) as usize, (ly + 3) as usize);
            draw::vline(
                grid,
                (bx + bw - 1) as usize,
                (ly + 1) as usize,
                (ly + 3) as usize,
            );
            let fill = (ctx.eased * (bw - 4) as f32).round() as i32;
            block(grid, bx + 2, ly + 2, fill, 1);
            if fill > 0 {
                block(grid, bx + 2, ly + 1, fill, 3);
            }
        }
        Ok(())
    }
}

// ─── 2. Nokia Snake ──────────────────────────────────────────────────────────

/// Snake grows segment by segment as progress rises, eating pellets on a dim LCD grid.
struct NokiaSnake;
impl ProgressStyle for NokiaSnake {
    fn name(&self) -> &str {
        "nokia-snake"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Nokia-style snake grows from left; segments = progress, pellets sparkle with time"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();

        // ── Snake body in dot space ──────────────────────────────────────────
        // Snake travels in a boustrophedon (row-alternating) path.
        let snake_dots = (ctx.eased * (w * h) as f32) as usize;
        let mid_y = h / 2;

        for seg in 0..snake_dots.min(w * h) {
            // Boustrophedon: even rows go right, odd rows go left.
            let row = seg / w;
            let col_in_row = seg % w;
            let x = if row % 2 == 0 {
                col_in_row
            } else {
                w.saturating_sub(1 + col_in_row)
            };
            let y = row;
            draw::dot(grid, x, y.min(h.saturating_sub(1)));
        }

        // ── Pellets: 2-dot squares at regular intervals, dim if eaten ────────
        let pellet_spacing = (w / 5).max(2);
        let pellet_count = w / pellet_spacing;
        let eaten = (ctx.eased * pellet_count as f32) as usize;
        for p in 0..pellet_count {
            let px = p * pellet_spacing + pellet_spacing / 2;
            if p >= eaten {
                // Uneaten pellet — blink with time
                let blink = ((ctx.time * 3.0 + p as f32 * 0.7).sin() > 0.0) as usize;
                let shade = 2 + blink; // ▒ or ▓
                draw::dot(grid, px.min(w.saturating_sub(1)), mid_y);
                draw::dot(grid, px.saturating_sub(1), mid_y);
                let _ = shade; // shade already expressed via dot presence/absence
            }
            // Eaten pellets leave no trace (already set by snake body or absent)
        }

        // ── Head: 3×3 block at the snake tip ─────────────────────────────────
        if snake_dots > 0 {
            let head_seg = snake_dots.saturating_sub(1);
            let row = head_seg / w;
            let col_in_row = head_seg % w;
            let hx = if row % 2 == 0 {
                col_in_row
            } else {
                w.saturating_sub(1 + col_in_row)
            };
            let hy = row.min(h.saturating_sub(1));
            for dy in 0..2i32 {
                for dx in 0..2i32 {
                    draw::dot_i(grid, hx as i32 + dx, hy as i32 + dy);
                }
            }
        }

        // ── Dim LCD background: only cells the snake left blank ─────────────
        // (a shade glyph overrides braille dots, so it must come last)
        for cy in 0..ch {
            for cx in 0..cw {
                if grid.get_char(cx, cy) == '\u{2800}' {
                    draw::shade(grid, cx, cy, 1);
                }
            }
        }

        // ── Palette tint on filled region ────────────────────────────────────
        let filled_cells = (ctx.eased * cw as f32) as usize;
        for cy in 0..ch {
            if filled_cells > 0 {
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    filled_cells.saturating_sub(1),
                    ctx.palette.sample(ctx.eased),
                );
            }
        }
        Ok(())
    }
}

// ─── 3. Tetris GB ────────────────────────────────────────────────────────────

/// Bevelled GB minos lock into a four-row strip one tetromino at a time —
/// I, O, O, L, J repeating so the strip tiles solid — while the next piece
/// drops in from the top.
struct TetrisGb;

/// One tetromino: mino cells as `(col, row)` within the 4-row strip.
type Piece = [(i32, i32); 4];

/// The repeating 5-column tiling, in lock order (bottom pieces first).
const TILING: [Piece; 5] = [
    [(0, 0), (0, 1), (0, 2), (0, 3)], // I
    [(1, 2), (2, 2), (1, 3), (2, 3)], // O (bottom)
    [(1, 0), (2, 0), (1, 1), (2, 1)], // O (top)
    [(4, 1), (4, 2), (4, 3), (3, 3)], // J
    [(3, 0), (3, 1), (3, 2), (4, 0)], // L
];

impl ProgressStyle for TetrisGb {
    fn name(&self) -> &str {
        "tetris-gb"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Game Boy Tetris: I, O, L and J tetrominoes, each with its own block texture, lock into a solid stack left-to-right while the next piece drops in; stack length = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cols = (wi / 4).max(1);
        let units = (cols + 4) / 5;
        let total = units * TILING.len() as i32;
        let placed = (ctx.eased * total as f32) as i32;

        // A mino: 3×3 on a 4-dot pitch, centred vertically. Like the GB game,
        // each piece type has its own block texture so pieces read apart.
        let top = (hi - 16).max(0) / 2;
        let mino = |grid: &mut BrailleGrid, kind: usize, col: i32, row: i32, dy: i32| {
            let art: [&str; 3] = match kind {
                0 => ["###", "###", "###"],     // I: solid
                1 | 2 => ["###", "#.#", "###"], // O: framed
                3 => ["#.#", ".#.", "#.#"],     // J: crossed
                _ => [".#.", "###", ".#."],     // L: studded
            };
            sprite(grid, col * 4, top + row * 4 + dy, &art);
        };
        for k in 0..total.min(placed + 1) {
            let piece = TILING[(k % 5) as usize];
            let base = (k / 5) * 5;
            // The next piece is still falling: it lifts off the top as time runs.
            let dy = if k == placed {
                -(((1.0 - (ctx.time * 1.5).fract()) * 16.0) as i32)
            } else {
                0
            };
            for (c, r) in piece {
                if base + c < cols {
                    mino(grid, (k % 5) as usize, base + c, r, dy);
                }
            }
        }
        Ok(())
    }
}

// ─── 4. Dr. Mario ────────────────────────────────────────────────────────────

/// The Dr. Mario bottle (neck, shoulders, glass body) full of wriggling
/// viruses that vanish as progress rises, with a two-tone capsule dropping in
/// through the neck; a row of capsules alongside fills with progress.
struct DrMario;

/// A capsule: two rounded halves with a seam.
const CAPSULE: [&str; 3] = [".##.##.", "###.###", ".##.##."];

impl ProgressStyle for DrMario {
    fn name(&self) -> &str {
        "dr-mario"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Dr. Mario: wriggling viruses vanish from the medicine bottle as a capsule drops through its neck; the capsule row beside it fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Bottle: neck on top, shoulders, then the glass body.
        let bw = 24.min(wi);
        draw::vline(grid, 8, 0, 3);
        draw::vline(grid, 15, 0, 3);
        draw::hline(grid, 0, 8, 3);
        draw::hline(grid, 15, (bw - 1).max(0) as usize, 3);
        draw::vline(grid, 0, 3, h - 1);
        draw::vline(grid, (bw - 1).max(0) as usize, 3, h - 1);
        draw::hline(grid, 0, (bw - 1).max(0) as usize, h - 1);

        // Viruses: a 5×2 grid of critters, cleared in a scattered order.
        let wiggle = (ctx.time * 4.0).fract() < 0.5;
        let virus: [&str; 3] = if wiggle {
            ["#.#", ".#.", "#.#"]
        } else {
            [".#.", "###", "#.#"]
        };
        for i in 0..10 {
            let (col, row) = (i % 5, i / 5);
            let order = ((i * 3 + 1) % 10) as f32 / 10.0;
            if order >= ctx.eased {
                sprite(grid, 2 + col * 4, hi - 9 + row * 4, &virus);
            }
        }

        // A capsule tumbling down the neck into the bottle.
        let fall = ((ctx.time * 0.8).fract() * (hi - 9).max(1) as f32) as i32;
        sprite(grid, 9, fall - 2, &[".##.", "####", "####", ".##."]);

        // Capsule row: one more capsule per step of progress.
        let slots = ((wi - bw - 4) / 8).max(0);
        let filled = (ctx.eased * slots as f32).round() as i32;
        for k in 0..slots {
            let x = bw + 4 + k * 8;
            if k < filled {
                sprite(grid, x, hi / 2 - 1, &CAPSULE);
            } else {
                draw::dot_i(grid, x + 3, hi / 2);
            }
        }
        Ok(())
    }
}

// ─── 5. Kirby Inhale ─────────────────────────────────────────────────────────

/// Pink round Kirby (red feet, rosy cheek, stubby arm) walks right with
/// progress, mouth gaping, inhaling a whirling stream of stars and Waddle
/// Dees along wind lines; the ground behind him is the track covered.
struct KirbyInhale;

/// Waddle Dee: round body, face patch with two eyes, little feet.
const WADDLE: [&str; 5] = [".###.", "#####", "#.#.#", "#####", "##.##"];
/// Five-point star.
const STAR5: [&str; 5] = ["..#..", "..#..", "#####", ".###.", "##.##"];

impl ProgressStyle for KirbyInhale {
    fn name(&self) -> &str {
        "kirby-inhale"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Pink Kirby with red feet and rosy cheeks walks right with progress, mouth wide, inhaling a stream of stars and Waddle Dees"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        // Kirby: body radius 6, feet on the bottom rows.
        let top = hi - 16;
        let travel = (wi - 34).max(0) as f32;
        let kx = 8 + (ctx.eased * travel).round() as i32;
        let bob = i32::from((t * 4.0).fract() >= 0.5);
        let cy = top + 6 + bob;
        let (mx, my) = (kx + 5, cy + 1); // mouth centre
        for dy in -6..=6 {
            for dx in -7..=6 {
                let inside = dx * dx + dy * dy <= 42;
                let arm = dx <= -6 && (0..=1).contains(&dy);
                if !(inside || arm) {
                    continue;
                }
                let (x, y) = (kx + dx, cy + dy);
                // Gaping mouth: an oval hole on the right side, with a lip.
                let (ox, oy) = (x - mx, y - my);
                let d = ox * ox + oy * oy;
                if d <= 10 && ox > -3 {
                    if d >= 7 && ox <= 0 {
                        draw::dot_i(grid, x, y);
                    }
                    continue;
                }
                // Eyes: tall ovals.
                if (dx == 0 || dx == 2) && (-4..=-2).contains(&dy) {
                    continue;
                }
                draw::dot_i(grid, x, y);
            }
        }
        // Red feet.
        sprite(grid, kx - 7, cy + 6, &[".#####.", "#######"]);
        sprite(grid, kx, cy + 6, &[".#####.", "#######"]);

        // Ground: solid behind Kirby, dashed ahead.
        let gy = hi - 1;
        for x in 0..wi {
            if x < kx - 7 || (x > kx + 7 && x % 4 == 0) {
                draw::dot_i(grid, x, gy);
            }
        }

        // Wind lines funnelling into the mouth.
        let far = wi - 1;
        let span = (far - mx).max(1) as f32;
        for i in 0..4 {
            let s = (t * 1.4 + i as f32 * 0.25).fract();
            let x = far - (s * span) as i32;
            let off = ((i % 2) * 2 - 1) as f32 * (1.0 - s) * 6.0;
            let y = my + off.round() as i32;
            for k in 0..4 {
                if x + k > mx + 2 {
                    draw::dot_i(grid, x + k, y);
                }
            }
        }
        // Stars and a Waddle Dee swirl in and vanish into the mouth.
        let mut dee_x = i32::MIN;
        for i in 0..3 {
            let s = (t * 0.6 + i as f32 / 3.0).fract();
            let x = far - (s * span) as i32;
            if x <= mx + 2 {
                continue;
            }
            let wob = ((t * 5.0 + i as f32 * 2.0).sin() * (1.0 - s) * 4.0).round() as i32;
            let y = my - 2 + wob;
            if i == 1 {
                sprite(grid, x, y, &WADDLE);
                dee_x = x;
            } else {
                sprite(grid, x, y, &STAR5);
            }
        }

        // Colour: pink body, rosy cheek, red feet, yellow stars, orange Dee.
        grid.enable_color_support();
        for cyy in 0..ch {
            for cxx in 0..cw {
                let (px, py) = (cxx as i32 * 2 + 1, cyy as i32 * 4 + 2);
                let color = if (kx - 8..=kx + 7).contains(&px) && py >= cy + 6 {
                    Color::rgb(232, 40, 70) // feet
                } else if (kx - 8..=kx + 7).contains(&px) {
                    if (kx - 3..=kx).contains(&px) && (cy..=cy + 3).contains(&py) {
                        Color::rgb(255, 80, 140) // blush
                    } else {
                        Color::rgb(255, 160, 205)
                    }
                } else if px < kx {
                    Color::rgb(120, 200, 110) // grass track
                } else if (dee_x - 1..=dee_x + 5).contains(&px) {
                    Color::rgb(255, 150, 60) // Waddle Dee
                } else {
                    Color::rgb(255, 230, 100) // stars and wind
                };
                let _ = grid.set_cell_color(cxx, cyy, color);
            }
        }
        Ok(())
    }
}

// ─── 6. Tamagotchi ───────────────────────────────────────────────────────────

/// The egg-shaped Tamagotchi with its LCD window and three buttons; the pet
/// inside grows from baby to adult with progress and toddles about, blinking,
/// while the hunger hearts beside it fill.
struct Tamagotchi;
impl ProgressStyle for Tamagotchi {
    fn name(&self) -> &str {
        "tamagotchi"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Tamagotchi: an egg-shaped toy with LCD window and three buttons; the pet grows baby→adult and toddles about while the hunger hearts fill with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;

        // Egg shell: an ellipse that narrows toward the top.
        let (cx, cy) = (11.0_f32, (hi - 1) as f32 / 2.0 + 0.5);
        let ry = (hi as f32 / 2.0).max(1.0);
        for y in 0..hi {
            for x in 0..23 {
                let v = (y as f32 - cy) / ry;
                let rx = 10.5 * (1.0 + 0.12 * v);
                let u = (x as f32 - cx) / rx;
                let d = (u * u + v * v).sqrt();
                if (0.86..1.0).contains(&d) {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // LCD window and the three buttons below it.
        draw::rect_outline(grid, 5, 3, 13, 8);
        for bx in [7, 11, 15] {
            draw::dot_i(grid, bx, 13);
        }

        // The pet: baby, child, then adult, toddling across the LCD.
        let stage: &[&str] = if ctx.eased < 0.34 {
            &[".##.", "#..#", ".##."]
        } else if ctx.eased < 0.67 {
            &[".###.", "#.#.#", "#####", ".#.#."]
        } else {
            &["#...#", ".###.", "#.#.#", "#####", ".#.#."]
        };
        let pw = stage[0].len() as i32;
        let px = 6 + (tri(ctx.time * 0.35) * (11 - pw).max(0) as f32) as i32;
        let py = 10 - stage.len() as i32;
        sprite(grid, px, py, stage);
        if (ctx.time * 0.7).fract() < 0.1 {
            // Blink: fill the eye row closed.
            draw::hline(
                grid,
                px.max(0) as usize,
                (px + pw - 1).max(0) as usize,
                (py + 1).max(0) as usize,
            );
        }

        // Hunger hearts: four of them, filling in half-heart steps.
        let halves = (ctx.eased * 8.0).round() as i32;
        for i in 0..4 {
            let left = halves - i * 2;
            let art = if left >= 2 {
                &HEART_FULL
            } else if left == 1 {
                &HEART_HALF
            } else {
                &HEART_EMPTY
            };
            sprite(grid, 28 + i * 10, hi / 2 - 3, art);
        }
        Ok(())
    }
}
// ─── 7. Game & Watch ─────────────────────────────────────────────────────────

/// Game & Watch *Ball*: the LCD juggler stands centre-stage with three nested
/// arcs of fixed ball segments overhead (unlit ones ghost faintly); balls hop
/// segment to segment and progress adds balls and lights the score ticks.
struct GameWatch;
impl ProgressStyle for GameWatch {
    fn name(&self) -> &str {
        "game-watch"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Game & Watch Ball: an LCD juggler under three arcs of ghosted ball segments; balls hop segment to segment, more join and score ticks light as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cx = wi / 2;

        // The juggler; his hands swap high/low with the throw rhythm.
        let beat = (ctx.time * 3.0).fract() < 0.5;
        let arms = if beat { "#..###...." } else { "....###..#" };
        let body = [
            "....###...",
            "....#.#...",
            "....###...",
            arms,
            ".#######..",
            "....###...",
            "....###...",
            "...#...#..",
            "..#.....#.",
        ];
        sprite(grid, cx - 5, hi - 9, &body);

        // Three nested arcs of LCD ball segments; lit balls are 2×2 blocks.
        let balls = 1 + (ctx.eased * 2.99) as i32;
        let slots = 7;
        for k in 0..3 {
            let (rx, ry) = (12.0 + k as f32 * 9.0, 5.0 + k as f32 * 1.5);
            let base = hi as f32 - 7.0;
            let tick = (ctx.time * 5.0) as i32 + k * 3;
            let span = (slots - 1) * 2;
            let pos = tick.rem_euclid(span);
            let lit = if pos < slots { pos } else { span - pos };
            for s in 0..slots {
                let a = PI * s as f32 / (slots - 1) as f32;
                let x = cx + (a.cos() * rx) as i32;
                let y = (base - a.sin() * ry) as i32;
                if k < balls && s == lit {
                    block(grid, x - 1, y - 1, 2, 2);
                } else {
                    draw::dot_i(grid, x, y); // ghost segment
                }
            }
        }

        // Score ticks along the top-left: ten LCD dashes.
        let ticks = (ctx.eased * 10.0).round() as i32;
        for i in 0..10 {
            if i < ticks {
                block(grid, i * 3, 0, 2, 1);
            } else {
                draw::dot_i(grid, i * 3, 0);
            }
        }
        Ok(())
    }
}

// ─── 8. Link's Awakening Hearts ──────────────────────────────────────────────

/// The Link's Awakening HUD: B/A item boxes (shield, sword) beside two rows
/// of seven heart containers that fill half a heart at a time; the next empty
/// heart blinks.
struct HeartContainers;

const HEART_FULL: [&str; 6] = [
    ".##.##.", "#######", "#######", ".#####.", "..###..", "...#...",
];
const HEART_HALF: [&str; 6] = [
    ".##.##.", "####..#", "####..#", ".###.#.", "..##...", "...#...",
];
const HEART_EMPTY: [&str; 6] = [
    ".##.##.", "#..#..#", "#.....#", ".#...#.", "..#.#..", "...#...",
];

impl ProgressStyle for HeartContainers {
    fn name(&self) -> &str {
        "hearts"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Link's Awakening HUD: B/A item boxes with shield and sword beside two rows of seven heart containers filling half a heart at a time; the next heart blinks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;

        // Item boxes: B (shield) and A (sword).
        if hi >= 14 && w >= 40 {
            draw::rect_outline(grid, 0, 1, 12, 14);
            draw::rect_outline(grid, 14, 1, 12, 14);
            sprite(
                grid,
                3,
                4,
                &["######", "#.##.#", "#.##.#", "######", ".####.", "..##.."],
            );
            sprite(
                grid,
                17,
                3,
                &[
                    "......#", ".....#.", "....#..", "#..#...", ".##....", ".##....", "#..#...",
                ],
            );
        }
        let x0 = if w >= 40 { 30 } else { 0 };

        // Fourteen containers, two rows of seven, in half-heart steps.
        let halves = (ctx.eased * 28.0).round() as i32;
        let blink = (ctx.time * 3.0).fract() < 0.5;
        for i in 0..14 {
            let (col, row) = (i % 7, i / 7);
            let x = x0 + col * 8;
            let y = if hi >= 14 { 1 + row * 8 } else { row * 8 };
            let left = halves - i * 2;
            let art = if left >= 2 {
                &HEART_FULL
            } else if left == 1 {
                &HEART_HALF
            } else if left == 0 && blink {
                continue; // the next heart to fill blinks out
            } else {
                &HEART_EMPTY
            };
            sprite(grid, x, y, art);
        }
        Ok(())
    }
}

// ─── 9. Mole Whack ───────────────────────────────────────────────────────────

/// A row of dirt holes; moles (round heads, beady eyes, snout) bob up and
/// down, and a red mallet winds up and slams the next one. Whacked moles stay
/// half-sunk with dizzy stars circling — the whacked count is the progress.
struct MoleWhack;

/// Mole head, top to bottom: crown, eyes, snout, shoulders.
const MOLE: [&str; 7] = [
    "..####..", ".######.", "##.##.##", "########", "###..###", "########", "########",
];
/// Mallet: barrel head with a handle sticking out to the right.
const MALLET: [&str; 5] = [
    ".######.........",
    "########........",
    "################",
    "########........",
    ".######.........",
];

impl ProgressStyle for MoleWhack {
    fn name(&self) -> &str {
        "mole-whack"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Whack-a-Mole: moles bob from a row of holes, a red mallet slams the next one; whacked moles see stars — whacked count = progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;

        let n = ((wi - 4) / 14).max(1);
        let spacing = (wi / n).max(1);
        let whacked = ((ctx.eased * n as f32) as i32).min(n);
        let rim = hi - 3;
        // Swing cycle for the mallet on the current target mole.
        let u = (t * 1.1).fract();
        let mut mallet = (i32::MIN / 2, i32::MIN / 2);

        for i in 0..n {
            let hx = i * spacing + spacing / 2;
            // Hole: a dirt lip arc the mole pops out of.
            sprite(
                grid,
                hx - 6,
                rim,
                &["##........##", ".##........##.", "..##########.."],
            );
            // Grass tufts between holes.
            sprite(grid, hx + spacing / 2 - 1, hi - 2, &["#.#", ".#."]);

            let rise = match i.cmp(&whacked) {
                std::cmp::Ordering::Less => 3,
                std::cmp::Ordering::Equal => {
                    if u < 0.45 {
                        (u / 0.45 * 7.0) as i32
                    } else if u < 0.6 {
                        7
                    } else {
                        (7.0 - (u - 0.6) / 0.4 * 7.0) as i32
                    }
                }
                std::cmp::Ordering::Greater => {
                    (tri(t * 0.7 + i as f32 * 0.37) * 10.0 - 3.0).clamp(0.0, 6.0) as i32
                }
            };
            // Mole, clipped so it emerges from inside the hole.
            let top = rim + 1 - rise;
            for (r, row) in MOLE.iter().enumerate() {
                let y = top + r as i32;
                if y > rim {
                    break;
                }
                for (c, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        draw::dot_i(grid, hx - 4 + c as i32, y);
                    }
                }
            }
            if i < whacked {
                // Dizzy stars orbiting the bonked mole.
                for k in 0..2 {
                    let a = t * 6.0 + k as f32 * PI + i as f32;
                    let sx = hx + (a.cos() * 5.0).round() as i32;
                    let sy = top - 3 + (a.sin() * 1.5).round() as i32;
                    sprite(grid, sx - 1, sy - 1, &[".#.", "###", ".#."]);
                }
            } else if i == whacked {
                // Mallet: wound up high and right, slams down at u = 0.6.
                let swing = if u < 0.5 {
                    1.0
                } else if u < 0.6 {
                    1.0 - (u - 0.5) / 0.1
                } else {
                    0.0
                };
                let my = rim - 11 - (swing * 2.0) as i32;
                let mx = hx - 4 + (swing * 7.0) as i32;
                sprite(grid, mx, my, &MALLET);
                mallet = (mx, my);
                if (0.6..0.75).contains(&u) {
                    // Impact burst.
                    for (dx, dy) in [(-7, -2), (-6, 1), (6, -2), (7, 1)] {
                        draw::dot_i(grid, hx + dx, my + 5 + dy);
                        draw::dot_i(grid, hx + dx + dx.signum(), my + 5 + dy);
                    }
                }
            }
        }

        // Colour: red mallet head, tan handle, brown moles, yellow stars,
        // green grass.
        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let (px, py) = (cx as i32 * 2 + 1, cy as i32 * 4 + 2);
                let local = (px + spacing * 4) % spacing - spacing / 2;
                let hole = (px / spacing).min(n - 1);
                let color = if (mallet.0 - 1..mallet.0 + 8).contains(&px)
                    && (mallet.1 - 1..mallet.1 + 6).contains(&py)
                {
                    Color::rgb(236, 52, 48)
                } else if (mallet.0 + 8..mallet.0 + 17).contains(&px)
                    && (mallet.1 - 1..mallet.1 + 6).contains(&py)
                {
                    Color::rgb(222, 176, 110)
                } else if local.abs() > 5 {
                    Color::rgb(110, 200, 90) // grass
                } else if hole < whacked && py < rim - 3 {
                    Color::rgb(255, 226, 80) // dizzy stars
                } else if py >= rim {
                    Color::rgb(126, 86, 48) // dirt rim
                } else {
                    Color::rgb(214, 168, 140) // mole
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 10. Wario Treasure ──────────────────────────────────────────────────────

/// Spinning gold coins arc into an open treasure chest on the right; the
/// coin stack beside it grows with progress and the chest overflows at 100%.
struct WarioTreasure;

/// Coin spin frames: face-on, three-quarter, edge-on.
const COIN: [[&str; 5]; 3] = [
    [".###.", "#...#", "#.#.#", "#...#", ".###."],
    ["..#..", ".#.#.", ".#.#.", ".#.#.", "..#.."],
    ["..#..", "..#..", "..#..", "..#..", "..#.."],
];

impl ProgressStyle for WarioTreasure {
    fn name(&self) -> &str {
        "wario-treasure"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "Wario Land treasure: spinning gold coins arc into an open treasure chest while the coin stacks beside it grow with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Treasure chest: a banded box with its lid thrown open behind.
        let chx = wi - 18;
        sprite(
            grid,
            chx + 1,
            hi - 13,
            &[".##############", "#..............#", "################"],
        );
        draw::rect_outline(grid, chx.max(0) as usize, (hi - 9).max(0) as usize, 18, 9);
        draw::hline(
            grid,
            chx.max(0) as usize,
            (chx + 17).max(0) as usize,
            (hi - 6).max(0) as usize,
        );
        block(grid, chx + 8, hi - 7, 2, 3); // lock

        // Coin stacks to the left, one coin per step of progress.
        let stacks = ((chx - 4) / 6).max(0);
        let coins = (ctx.eased * (stacks * 3) as f32).round() as i32;
        for i in 0..coins {
            let (col, lvl) = (i / 3, i % 3);
            sprite(grid, col * 6, hi - 3 - lvl * 3, &["####", "#..#", "####"]);
        }

        // Coins in flight, spinning as they arc into the chest.
        for k in 0..2 {
            let t = (ctx.time * 0.8 + k as f32 * 0.5).fract();
            let x0 = (stacks * 6).min(chx - 6) as f32;
            let x = x0 + t * (chx as f32 + 6.0 - x0);
            let y = (hi - 9) as f32 - (t * PI).sin() * (hi as f32 - 6.0);
            let frame = ((ctx.time * 8.0) as i32 + k).rem_euclid(4);
            let art = &COIN[[0, 1, 2, 1][frame as usize]];
            sprite(grid, x as i32 - 2, y as i32 - 2, art);
        }

        // At 100% the chest overflows with gold.
        if ctx.eased >= 1.0 {
            sprite(grid, chx + 3, hi - 11, &["#.#.#.#.#.#", ".#.#.#.#.#."]);
        }
        Ok(())
    }
}

// ─── 11. LCD Pinball ─────────────────────────────────────────────────────────

/// A sideways LCD pinball table: rails top and bottom, a coiled plunger on
/// the left, six round pop bumpers (lit ones solid) and a pair of flippers
/// guarding the drain on the right, with the ball ricocheting across.
struct LcdPinball;
impl ProgressStyle for LcdPinball {
    fn name(&self) -> &str {
        "lcd-pinball"
    }
    fn theme(&self) -> &str {
        "gameboy"
    }
    fn describe(&self) -> &str {
        "LCD pinball table: plunger, six round pop bumpers and twin flippers; bumpers light up solid as progress rises while the ball ricochets with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Rails and the plunger spring.
        draw::hline(grid, 0, w - 1, 0);
        draw::hline(grid, 0, w - 1, h - 1);
        for y in 3..hi - 3 {
            draw::dot_i(grid, 1 + (y % 2), y);
        }

        // Pop bumpers in two staggered rows; lit ones are solid discs.
        const RING: [&str; 5] = [".###.", "#...#", "#...#", "#...#", ".###."];
        const LIT: [&str; 5] = [".###.", "#####", "#####", "#####", ".###."];
        let lit = (ctx.eased * 6.0).round() as i32;
        let span = (wi - 26).max(6);
        for k in 0..6 {
            let x = 8 + k * span / 6;
            let y = if k % 2 == 0 { 2 } else { hi - 7 };
            sprite(grid, x, y, if k < lit { &LIT } else { &RING });
        }

        // Flippers: angled bats either side of the drain, flicking up.
        let flick = (ctx.time * 1.3).fract() < 0.18;
        let (fx0, fx1) = (wi - 2, wi - 10);
        let mid = hi / 2;
        let lift = if flick { 3 } else { 0 };
        for i in 0..=8 {
            let t = i as f32 / 8.0;
            let x = fx0 + ((fx1 - fx0) as f32 * t) as i32;
            draw::dot_i(grid, x, 1 + ((mid - 2 + lift - 1) as f32 * t) as i32);
            draw::dot_i(grid, x, hi - 2 - ((mid - 2 + lift - 1) as f32 * t) as i32);
        }

        // The ball ricochets around the table.
        let bx = 5 + (tri(ctx.time * 0.45) * (wi - 18).max(0) as f32) as i32;
        let by = 2 + (tri(ctx.time * 0.83 + 0.2) * (hi - 6).max(0) as f32) as i32;
        block(grid, bx, by, 2, 2);
        Ok(())
    }
}
