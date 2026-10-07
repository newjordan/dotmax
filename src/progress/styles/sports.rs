//! Sports-themed progress bars — eleven distinct athletic spectacles in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (perpetual animation)
//! and `ctx.eased` / `ctx.progress` (progress-driven advancement). All writes go
//! through `draw::` helpers so every pixel is silently bounds-safe.
//!
//! Styles in this file:
//! - `sprint-100m`      — sprinter strides down a lined track, tape breaks at the chequered finish
//! - `basketball-arc`   — spinning ball arcs into a backboard, rim and net; swish at 100%
//! - `soccer-goal`      — football rolls into a goal of posts and netting; net billows on score
//! - `swimming-laps`    — freestyle swimmer windmills down a roped lane to the wall
//! - `archery`          — arrow flies from a bow into a gold/red/blue ringed target
//! - `bowling`          — ball rolls down a wooden lane into a rack of pins; strike at 100%
//! - `darts`            — three darts land in turn on a segmented dartboard
//! - `high-jump`        — run-up, Fosbury flop over the crossbar, landing on the mat
//! - `weightlifting`    — lifter hoists a plate-loaded barbell from floor to overhead
//! - `tennis-rally`     — two rackets trade a bouncing ball over the net
//! - `cycling-peloton`  — three road cyclists pedal toward the finish, wheels spinning

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `sports` theme.
///
/// Returns one boxed [`ProgressStyle`] per sport variant, ready to be mixed
/// into a gallery or driven individually.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Sprint100m),
        Box::new(BasketballArc),
        Box::new(SoccerGoal),
        Box::new(SwimmingLaps),
        Box::new(Archery),
        Box::new(Bowling),
        Box::new(Darts),
        Box::new(HighJump),
        Box::new(Weightlifting),
        Box::new(TennisRally),
        Box::new(CyclingPeloton),
    ]
}

// ── Sprite helpers ────────────────────────────────────────────────────────────

/// Stamp an ASCII sprite: every `#` in `rows` becomes a dot at
/// `(x0 + col, y0 + row)`. Out-of-bounds dots are dropped by `draw::dot_i`.
fn sprite(grid: &mut BrailleGrid, x0: i32, y0: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x0 + c as i32, y0 + r as i32);
            }
        }
    }
}

/// Filled disc of dots centred on `(cx, cy)` with radius `r` (dot units).
fn disc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32) {
    let r2 = r * r + 0.35;
    for y in (cy - r).floor() as i32..=(cy + r).ceil() as i32 {
        for x in (cx - r).floor() as i32..=(cx + r).ceil() as i32 {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            if dx * dx + dy * dy <= r2 {
                draw::dot_i(grid, x, y);
            }
        }
    }
}

/// Colour every cell overlapping the dot box `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    if x1 < 0 || y1 < 0 || x1 < x0 || y1 < y0 {
        return;
    }
    let (cx0, cx1) = ((x0.max(0) / 2) as usize, (x1 / 2) as usize);
    let (cy0, cy1) = ((y0.max(0) / 4) as usize, (y1 / 4) as usize);
    for cy in cy0..=cy1 {
        draw::tint_row(grid, cy, cx0, cx1, color);
    }
}

// ── Sprint 100m ───────────────────────────────────────────────────────────────

/// Runner sprite, two stride frames (facing right, 8 wide × 11 tall).
const RUNNER: [[&str; 11]; 2] = [
    [
        ".....##.", ".....##.", "...####.", "..#.##.#", ".#..##..", "....##..", "...#..#.",
        "..#....#", ".#......", "#.......", "........",
    ],
    [
        ".....##.", ".....##.", "....###.", "...###..", "...#.##.", "....##..", "....##..",
        "....#.#.", "...#..#.", "...#...#", "........",
    ],
];

struct Sprint100m;
impl ProgressStyle for Sprint100m {
    fn name(&self) -> &str {
        "sprint-100m"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Sprinter strides down a lined track toward a chequered finish; the tape breaks at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let track = Color::rgb(200, 84, 60);
        let chalk = Color::rgb(235, 235, 235);

        // Lane lines: solid outer line, dashed distance marks that stream past.
        draw::hline(grid, 0, w - 1, base as usize);
        let scroll = (ctx.time * 30.0) as i32;
        for x in 0..wi {
            if (x + scroll).rem_euclid(10) < 4 {
                draw::dot_i(grid, x, 1);
            }
        }
        paint(grid, 0, 0, wi - 1, hi - 1, track);

        // Chequered finish line on the right.
        let fx = wi - 4;
        for y in 0..hi {
            for x in fx..fx + 4 {
                if ((x - fx) / 2 + y / 2) % 2 == 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        paint(grid, fx, 0, wi - 1, hi - 1, chalk);

        // Runner position follows progress; stride frame follows time.
        let travel = (fx - 10).max(0) as f32;
        let rx = (ctx.eased * travel) as i32;
        let ry = (base - 11).max(0);
        let frame = ((ctx.time * 8.0) as usize) % 2;
        sprite(grid, rx, ry, &RUNNER[frame]);
        // Speed lines trailing the runner.
        for k in 0..3 {
            let len = 3 + ((ctx.time * 10.0 + k as f32) as i32).rem_euclid(3);
            let y = ry + 3 + k * 2;
            draw::hline(
                grid,
                (rx - 2 - len).max(0) as usize,
                (rx - 2).max(0) as usize,
                y as usize,
            );
        }
        paint(grid, rx - 6, ry, rx + 7, ry + 9, Color::rgb(255, 214, 64));

        // Finish tape: taut across the line, snapped and fluttering at 100%.
        let tape_y = ry + 4;
        if ctx.progress < 0.999 {
            draw::hline(grid, (fx - 1) as usize, (wi - 1) as usize, tape_y as usize);
        } else {
            for k in 0..4 {
                let wave = ((ctx.time * 9.0 + k as f32).sin() * 1.5) as i32;
                draw::dot_i(grid, fx - 1 - k, tape_y + k / 2 + wave);
                draw::dot_i(grid, wi - 1, tape_y + k + wave);
            }
        }
        paint(
            grid,
            fx - 2,
            tape_y,
            fx + 1,
            tape_y,
            Color::rgb(255, 60, 60),
        );
        Ok(())
    }
}

// ── Basketball Arc ────────────────────────────────────────────────────────────

struct BasketballArc;
impl ProgressStyle for BasketballArc {
    fn name(&self) -> &str {
        "basketball-arc"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Spinning basketball arcs toward a backboard, rim and net; it swishes through at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let s = (hi as f32 / 16.0).max(0.25);

        // Hoop: pole, backboard, rim and a swaying net.
        let board_x = wi - 3;
        let rim_y = (5.0 * s) as i32;
        let rim_x0 = board_x - 12;
        draw::vline(grid, (wi - 1) as usize, 0, (hi - 1) as usize);
        draw::vline(grid, board_x as usize, 0, (rim_y + 3) as usize);
        draw::hline(
            grid,
            board_x as usize,
            (wi - 1) as usize,
            (rim_y - 1).max(0) as usize,
        );
        paint(grid, board_x, 0, wi - 1, hi - 1, Color::rgb(190, 200, 215));
        draw::hline(
            grid,
            rim_x0 as usize,
            (board_x - 1) as usize,
            rim_y as usize,
        );
        paint(
            grid,
            rim_x0,
            rim_y,
            board_x - 1,
            rim_y,
            Color::rgb(255, 80, 40),
        );
        let swish = if ctx.progress >= 0.999 {
            ((ctx.time * 10.0).sin() * 1.5) as i32
        } else {
            0
        };
        let net_h = (6.0 * s) as i32;
        for row in 1..=net_h {
            let inset = row / 2;
            for x in (rim_x0 + inset..board_x - inset).step_by(2) {
                let x = x + (row % 2) + if row > 2 { swish } else { 0 };
                draw::dot_i(grid, x, rim_y + row);
            }
        }
        paint(
            grid,
            rim_x0,
            rim_y + 1,
            board_x - 1,
            rim_y + net_h,
            Color::rgb(235, 235, 235),
        );

        // Court floor.
        draw::hline(grid, 0, (rim_x0 - 2).max(0) as usize, (hi - 1) as usize);
        paint(
            grid,
            0,
            hi - 1,
            rim_x0 - 2,
            hi - 1,
            Color::rgb(170, 110, 60),
        );

        // Ball flight: parabola from the shooter's hands to the rim.
        let p = ctx.eased;
        let (x0, y0) = (4.0, hi as f32 - 5.0 * s);
        let (x1, y1) = ((rim_x0 + board_x) as f32 / 2.0, rim_y as f32 + 2.0 * s);
        let apex = 6.0 * s;
        let pos = |q: f32| {
            (
                x0 + (x1 - x0) * q,
                y0 + (y1 - y0) * q - apex * 4.0 * q * (1.0 - q),
            )
        };
        // Dotted trail of where the ball has been.
        let trail = ((p - 0.06) / 0.035).ceil().max(0.0) as i32;
        for i in 0..trail {
            let (tx, ty) = pos(i as f32 * 0.035);
            draw::dot_i(grid, tx as i32, ty as i32);
            paint(
                grid,
                tx as i32,
                ty as i32,
                tx as i32,
                ty as i32,
                Color::rgb(150, 90, 40),
            );
        }
        let (bx, by) = pos(p);
        let by = if ctx.progress >= 0.999 {
            by + 3.0 * s
        } else {
            by
        };
        let r = 2.6 * s;
        // Ball outline with a seam that spins with time.
        for k in 0..20 {
            let a = k as f32 / 20.0 * 2.0 * PI;
            draw::dot_i(
                grid,
                (bx + a.cos() * r).round() as i32,
                (by + a.sin() * r).round() as i32,
            );
        }
        let spin = ctx.time * 6.0;
        for k in -2..=2 {
            let d = k as f32 * r / 2.5;
            draw::dot_i(
                grid,
                (bx + spin.cos() * d).round() as i32,
                (by + spin.sin() * d).round() as i32,
            );
        }
        paint(
            grid,
            (bx - r) as i32,
            (by - r) as i32,
            (bx + r) as i32,
            (by + r) as i32,
            Color::rgb(255, 140, 30),
        );
        Ok(())
    }
}

// ── Soccer Goal ───────────────────────────────────────────────────────────────

struct SoccerGoal;
impl ProgressStyle for SoccerGoal {
    fn name(&self) -> &str {
        "soccer-goal"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Football rolls across the pitch into a goal of posts, crossbar and net; the net billows on the score"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let scored = ctx.progress >= 0.999;

        // Goal (side-on): front post, crossbar, back stanchion and the net.
        let gx = wi - 16;
        let bar_y = (hi / 8).max(0);
        draw::vline(grid, gx as usize, bar_y as usize, base as usize);
        draw::hline(grid, gx as usize, (wi - 6).max(gx) as usize, bar_y as usize);
        draw::vline(grid, (wi - 1) as usize, (bar_y + 4) as usize, base as usize);
        let billow = if scored {
            ((ctx.time * 6.0).sin().abs() * 2.0) as i32
        } else {
            0
        };
        // Pitch with mown tufts that drift slowly.
        draw::hline(grid, 0, (w - 1) as usize, base as usize);
        let drift = (ctx.time * 4.0) as i32;
        for x in 0..gx {
            if (x + drift).rem_euclid(7) == 0 {
                draw::dot_i(grid, x, base - 1);
            }
        }
        paint(grid, 0, 0, wi - 1, base, Color::rgb(60, 200, 90));
        // Net mesh: a square lattice, pushed out on a goal.
        for y in bar_y + 1..base {
            for x in gx + 1..wi - 1 {
                let col = (x - gx) % 3 == 0 && y % 2 == 0;
                let row = (y - bar_y) % 3 == 0 && x % 2 == 0;
                if col || row {
                    let push = if y > bar_y + 2 && y < base - 2 {
                        billow
                    } else {
                        0
                    };
                    draw::dot_i(grid, (x + push).min(wi - 2), y);
                }
            }
        }
        paint(grid, gx, 0, wi - 1, base, Color::rgb(235, 235, 240));

        // Ball rolls toward the goal mouth; panels rotate with time.
        let r = (hi as f32 / 5.0).clamp(1.0, 3.2);
        let travel = (gx + 4) as f32 - r - 2.0;
        let bx = 2.0 + r + ctx.eased * (travel - 2.0 - r).max(0.0);
        let by = base as f32 - 1.0 - r;
        for k in 0..20 {
            let a = k as f32 / 20.0 * 2.0 * PI;
            draw::dot_i(
                grid,
                (bx + a.cos() * r).round() as i32,
                (by + a.sin() * r).round() as i32,
            );
        }
        let roll = ctx.time * 5.0 + ctx.eased * 12.0;
        draw::dot_i(grid, bx.round() as i32, by.round() as i32);
        for k in 0..3 {
            let a = roll + k as f32 * 2.0 * PI / 3.0;
            let d = r * 0.55;
            draw::dot_i(
                grid,
                (bx + a.cos() * d).round() as i32,
                (by + a.sin() * d).round() as i32,
            );
        }
        paint(
            grid,
            (bx - r) as i32,
            (by - r) as i32,
            (bx + r) as i32,
            (by + r) as i32,
            Color::rgb(255, 255, 255),
        );
        Ok(())
    }
}

// ── Swimming Laps ─────────────────────────────────────────────────────────────

struct SwimmingLaps;
impl ProgressStyle for SwimmingLaps {
    fn name(&self) -> &str {
        "swimming-laps"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Freestyle swimmer windmills down a rippling lane under rope floats toward the touchpad wall"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;
        let surface = hi / 2 - 1;

        // Lane rope: chunky floats, alternating red and white cells.
        for x in 0..wi {
            if x % 4 != 3 {
                draw::dot_i(grid, x, 1);
                draw::dot_i(grid, x, 2);
            }
        }
        let (cw, _) = grid.dimensions();
        for cx in 0..cw {
            let c = if (cx / 2) % 2 == 0 {
                Color::rgb(255, 70, 70)
            } else {
                Color::rgb(240, 240, 240)
            };
            draw::tint_row(grid, 0, cx, cx, c);
        }

        // Rippling water surface, a floor stripe and the wall with touchpad.
        for x in 0..wi - 3 {
            let y = surface + ((x as f32 * 0.5 - t * 4.0).sin() * 0.8).round() as i32 + 1;
            if x % 2 == 0 {
                draw::dot_i(grid, x, y);
            }
        }
        draw::hline(grid, 0, (wi - 4) as usize, (hi - 1) as usize);
        draw::hline(grid, 4, (wi - 8) as usize, (hi - 3) as usize);
        draw::vline(grid, (wi - 2) as usize, 1, (hi - 1) as usize);
        draw::vline(grid, (wi - 1) as usize, 1, (hi - 1) as usize);
        paint(grid, 0, 4, wi - 1, hi - 1, Color::rgb(60, 160, 255));
        paint(grid, wi - 2, 0, wi - 1, hi - 1, Color::rgb(250, 220, 60));

        // Swimmer: body along the surface, capped head, windmilling arm, kicks.
        let len = 14;
        let sx = len + (ctx.eased * (wi - len - 6) as f32) as i32;
        let by = surface + 1;
        draw::hline(grid, (sx - len).max(0) as usize, sx as usize, by as usize);
        draw::hline(
            grid,
            (sx - len + 3).max(0) as usize,
            (sx - 1) as usize,
            (by + 1) as usize,
        );
        disc(grid, (sx + 2) as f32, (by - 1) as f32, 1.3);
        let a = t * 7.0;
        for k in 1..=6 {
            let k = k as f32;
            draw::dot_i(
                grid,
                (sx as f32 - 2.0 + a.cos() * k) as i32,
                (by as f32 + a.sin() * k * 0.7) as i32,
            );
        }
        for k in 0..3 {
            let splash = ((t * 12.0 + k as f32 * 2.1).sin().abs() * 3.0) as i32;
            draw::dot_i(grid, sx - len - 1 - k * 2, by - splash);
        }
        paint(
            grid,
            sx - len - 6,
            by - 3,
            sx + 4,
            by + 1,
            Color::rgb(255, 200, 150),
        );
        paint(
            grid,
            sx + 1,
            by - 3,
            sx + 4,
            by - 1,
            Color::rgb(255, 80, 80),
        );
        Ok(())
    }
}

// ── Archery ───────────────────────────────────────────────────────────────────

struct Archery;
impl ProgressStyle for Archery {
    fn name(&self) -> &str {
        "archery"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Arrow flies from a strung bow toward a gold-red-blue ringed target; it thuds into the bullseye at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cy = (hi - 1) as f32 / 2.0;
        let r = cy.min(w as f32 / 6.0).max(1.0);
        let tx = wi as f32 - r - 1.0;

        // Target face: solid gold bullseye, then red and blue rings.
        disc(grid, tx, cy, r * 0.3);
        for (k, frac) in [0.55_f32, 0.8, 1.0].iter().enumerate() {
            let n = 36 + k * 8;
            for i in 0..n {
                let a = i as f32 / n as f32 * 2.0 * PI;
                draw::dot_i(
                    grid,
                    (tx + a.cos() * r * frac).round() as i32,
                    (cy + a.sin() * r * frac).round() as i32,
                );
            }
        }
        // Colour each target cell by its distance from the centre.
        let (cw, ch) = grid.dimensions();
        for cyc in 0..ch {
            for cx in 0..cw {
                let dx = (cx * 2) as f32 + 1.0 - tx;
                let dy = (cyc * 4) as f32 + 1.5 - cy;
                let d = (dx * dx + dy * dy).sqrt() / r;
                let c = match d {
                    d if d < 0.4 => Color::rgb(255, 215, 0),
                    d if d < 0.68 => Color::rgb(240, 50, 50),
                    d if d < 0.92 => Color::rgb(60, 140, 255),
                    d if d < 1.2 => Color::rgb(230, 230, 230),
                    _ => continue,
                };
                draw::tint_row(grid, cyc, cx, cx, c);
            }
        }

        // Bow on the left: a curved limb and its string.
        let bow_h = (hi - 2).max(1);
        for y in 0..=bow_h {
            let u = y as f32 / bow_h as f32 * 2.0 - 1.0;
            let bx = 1 + (3.0 * (1.0 - u * u)) as i32;
            draw::dot_i(grid, bx, y + 1);
        }
        draw::vline(grid, 1, 1, (bow_h + 1) as usize);
        paint(grid, 0, 0, 4, hi - 1, Color::rgb(170, 110, 60));

        // Arrow: shaft, head and fletching; flies to the bullseye with progress.
        let len = 12;
        let tip0 = 6.0;
        let tip1 = tx;
        let tip = (tip0 + (tip1 - tip0) * ctx.eased) as i32;
        let ay = cy.round() as i32;
        let wobble = if ctx.progress >= 0.999 {
            0
        } else {
            ((ctx.time * 14.0).sin() * 0.8).round() as i32
        };
        draw::hline(
            grid,
            (tip - len).max(0) as usize,
            tip.max(0) as usize,
            ay as usize,
        );
        for k in 1..3 {
            draw::dot_i(grid, tip - k, ay - k);
            draw::dot_i(grid, tip - k, ay + k);
        }
        for k in 0..4 {
            let spread = 2 - k / 2;
            draw::dot_i(grid, tip - len + k, ay - spread + wobble);
            draw::dot_i(grid, tip - len + k, ay + spread - wobble);
        }
        paint(
            grid,
            tip - len,
            ay - 2,
            tip - 3,
            ay + 2,
            Color::rgb(240, 240, 240),
        );
        Ok(())
    }
}

// ── Bowling ───────────────────────────────────────────────────────────────────

/// Standing pin silhouette (4 wide × 11 tall).
const PIN: [&str; 11] = [
    ".##.", "####", "####", ".##.", ".##.", "####", "####", "####", "####", "####", ".##.",
];
/// Toppled pin lying on its side (11 wide × 4 tall).
const PIN_DOWN: [&str; 4] = [".##.....##.", "###########", "###########", ".#####..##."];

struct Bowling;
impl ProgressStyle for Bowling {
    fn name(&self) -> &str {
        "bowling"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Bowling ball rolls down a wooden lane toward a rack of white pins; strike scatters them at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;

        // Lane boards with drifting board seams.
        draw::hline(grid, 0, w - 1, base as usize);
        for x in (0..wi).step_by(6) {
            draw::dot_i(grid, x + ((ctx.time * 2.0) as i32 % 2), base - 1);
        }
        paint(grid, 0, base - 1, wi - 1, base, Color::rgb(200, 150, 90));

        // Pins: four in a staggered rack; knocked flat on a strike.
        let rack_x = wi - 20;
        let strike = ctx.progress >= 0.999;
        for k in 0..4 {
            let px = rack_x + k * 5;
            if strike {
                let hop = ((ctx.time * 7.0 + k as f32 * 1.7).sin().abs() * 3.0) as i32;
                sprite(grid, px - 3 + k, base - 4 - hop - (k % 2) * 3, &PIN_DOWN);
            } else {
                sprite(grid, px, base - 11, &PIN);
            }
        }
        paint(
            grid,
            rack_x - 4,
            0,
            wi - 1,
            base - 2,
            Color::rgb(245, 245, 245),
        );

        // Ball: a solid sphere trailing motion streaks.
        let r = ((hi as f32 - 2.0) / 3.0).clamp(1.0, 4.0);
        let stop = (rack_x - 2) as f32 - r;
        let bx = r + 1.0 + ctx.eased * (stop - r - 1.0).max(0.0);
        let by = base as f32 - 1.0 - r;
        disc(grid, bx, by, r);
        paint(
            grid,
            (bx - r) as i32,
            (by - r) as i32,
            (bx + r) as i32,
            (by + r) as i32,
            Color::rgb(90, 70, 220),
        );
        // Motion streaks behind the ball.
        let streak = ((ctx.time * 12.0) as i32).rem_euclid(3);
        for k in 0..2 {
            let y = by as i32 - 1 + k * 2;
            let x1 = (bx - r) as i32 - 2;
            draw::hline(
                grid,
                (x1 - 3 - streak).max(0) as usize,
                x1.max(0) as usize,
                y as usize,
            );
        }
        Ok(())
    }
}

// ── Darts ─────────────────────────────────────────────────────────────────────

/// Draw one dart with its point at `(tip, y)`: needle, barrel and flights.
fn dart(grid: &mut BrailleGrid, tip: i32, y: i32) {
    draw::hline(
        grid,
        (tip - 2).max(0) as usize,
        tip.max(0) as usize,
        y.max(0) as usize,
    );
    for k in 3..7 {
        draw::dot_i(grid, tip - k, y - 1);
        draw::dot_i(grid, tip - k, y);
        draw::dot_i(grid, tip - k, y + 1);
    }
    draw::hline(
        grid,
        (tip - 10).max(0) as usize,
        (tip - 7).max(0) as usize,
        y.max(0) as usize,
    );
    for k in 0..3 {
        draw::dot_i(grid, tip - 9 - k, y - 1 - k);
        draw::dot_i(grid, tip - 9 - k, y + 1 + k);
    }
}

struct Darts;
impl ProgressStyle for Darts {
    fn name(&self) -> &str {
        "darts"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Three darts fly in turn at a segmented dartboard with red bull and doubles ring; each third of progress lands one"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cy = (hi - 1) as f32 / 2.0;
        let r = cy.min(w as f32 / 6.0).max(1.0);
        let bx = wi as f32 - r - 1.0;

        // Board: alternating wedge sectors, treble and double rings, bull.
        let ri = r.ceil() as i32;
        for y in -ri..=ri {
            for x in -ri..=ri {
                let d = ((x * x + y * y) as f32).sqrt() / r;
                if d > 1.0 {
                    continue;
                }
                let sector = ((y as f32).atan2(x as f32) / (2.0 * PI) * 20.0 + 20.5) as i32;
                let ringed = (0.86..=1.0).contains(&d) || (0.5..=0.62).contains(&d);
                if d < 0.16 || ringed || sector % 2 == 0 {
                    draw::dot_i(grid, bx as i32 + x, cy.round() as i32 + y);
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        for cyc in 0..ch {
            for cx in 0..cw {
                let dx = (cx * 2) as f32 + 0.5 - bx;
                let dy = (cyc * 4) as f32 + 1.5 - cy;
                let d = (dx * dx + dy * dy).sqrt() / r;
                let c = match d {
                    d if d < 0.3 => Color::rgb(240, 50, 50),
                    d if d < 0.85 => Color::rgb(240, 225, 190),
                    d if d < 1.25 => Color::rgb(40, 190, 90),
                    _ => continue,
                };
                draw::tint_row(grid, cyc, cx, cx, c);
            }
        }

        // Three throws: each third of progress carries one dart to the board.
        let hits = [(-1.0_f32, -2.0_f32), (0.5, 2.0), (0.0, 0.0)];
        let p = ctx.eased * 3.0;
        for (k, &(ox, oy)) in hits.iter().enumerate() {
            let u = (p - k as f32).clamp(0.0, 1.0);
            if u <= 0.0 && k > 0 {
                break;
            }
            let target_x = bx + ox - r * 0.1;
            let target_y = cy + oy;
            let tip = 10.0 + (target_x - 10.0) * u;
            let arc = 4.0 * u * (1.0 - u) * 3.0;
            let wobble = if u < 1.0 {
                (ctx.time * 15.0).sin() * 0.5
            } else {
                0.0
            };
            let y = (target_y - arc + wobble).round() as i32;
            dart(grid, tip as i32, y);
            paint(
                grid,
                tip as i32 - 12,
                y - 3,
                tip as i32 - 3,
                y + 3,
                Color::rgb(90, 200, 255),
            );
        }
        Ok(())
    }
}

// ── High Jump ─────────────────────────────────────────────────────────────────

/// Fosbury-flop body arched over the bar, head leading (12 wide × 5 tall).
const FLOP: [&str; 5] = [
    "....######..",
    "..##......##",
    ".#.........#",
    "#.........##",
    "#.........##",
];

struct HighJump;
impl ProgressStyle for HighJump {
    fn name(&self) -> &str {
        "high-jump"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Jumper sprints the run-up, arches over a crossbar between two standards in a flop and lands on the mat"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;

        // Standards and crossbar near the right; landing mat beyond.
        let sx = wi - 26;
        let bar_y = base - 9;
        draw::vline(grid, sx as usize, bar_y as usize, base as usize);
        draw::vline(grid, (sx + 4) as usize, bar_y as usize, base as usize);
        draw::hline(grid, (sx - 1) as usize, (sx + 5) as usize, bar_y as usize);
        paint(grid, sx - 1, bar_y, sx + 5, base, Color::rgb(255, 230, 80));
        draw::fill_rect(
            grid,
            (sx + 7) as usize,
            (base - 3) as usize,
            18.min(w - (sx + 7) as usize),
            4,
        );
        paint(
            grid,
            sx + 7,
            base - 3,
            wi - 1,
            base,
            Color::rgb(60, 120, 255),
        );
        draw::hline(grid, 0, (sx - 2) as usize, base as usize);
        paint(grid, 0, base, sx - 2, base, Color::rgb(200, 84, 60));

        // Run-up for the first 60%, then a flop arc over the bar onto the mat.
        let p = ctx.eased;
        let jumper = Color::rgb(255, 255, 255);
        if p < 0.6 {
            let rx = (p / 0.6 * (sx - 16) as f32) as i32;
            let frame = ((ctx.time * 8.0) as usize) % 2;
            sprite(grid, rx, base - 11, &RUNNER[frame]);
            paint(grid, rx, base - 11, rx + 7, base - 1, jumper);
        } else {
            let u = (p - 0.6) / 0.4;
            let x0 = (sx - 14) as f32;
            let x1 = (sx + 10) as f32;
            let x = x0 + (x1 - x0) * u;
            let ground = (base - 8) as f32;
            let y = ground - 4.0 * u * (1.0 - u) * 6.0 + u * 4.0;
            sprite(grid, x as i32, y.round() as i32, &FLOP);
            disc(grid, x + 12.0, y + 4.0, 1.2);
            paint(
                grid,
                x as i32,
                y as i32,
                x as i32 + 13,
                y as i32 + 5,
                jumper,
            );
        }
        Ok(())
    }
}

// ── Weightlifting ─────────────────────────────────────────────────────────────

/// Straight dot line between two points (simple DDA, bounds-safe).
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for i in 0..=n {
        let x = x0 + (x1 - x0) * i / n;
        let y = y0 + (y1 - y0) * i / n;
        draw::dot_i(grid, x, y);
    }
}

struct Weightlifting;
impl ProgressStyle for Weightlifting {
    fn name(&self) -> &str {
        "weightlifting"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Lifter hoists a plate-loaded barbell from the floor to locked-out overhead; plates load up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let cx = wi / 2;
        let p = ctx.eased;

        // Platform.
        draw::hline(
            grid,
            (cx - 30).max(0) as usize,
            (cx + 30).min(wi - 1) as usize,
            base as usize,
        );
        paint(grid, cx - 30, base, cx + 30, base, Color::rgb(150, 110, 70));

        // Bar height: floor (p = 0) to overhead (p = 1), straining with time.
        let strain = if p > 0.97 {
            ((ctx.time * 12.0).sin() * 0.6).round() as i32
        } else {
            0
        };
        let bar_y = (base as f32 - 4.0 - p * (base as f32 - 5.0)).round() as i32 + strain;

        // Lifter: head, torso, legs bending deeper while the bar is low.
        let squat = ((1.0 - p) * 3.0) as i32;
        let hip_y = base - 5 + squat / 2;
        let sh_y = hip_y - 5;
        disc(grid, cx as f32, (sh_y - 2) as f32, 1.3);
        draw::vline(grid, cx as usize, sh_y as usize, hip_y as usize);
        draw::vline(grid, (cx - 1) as usize, sh_y as usize, hip_y as usize);
        line(grid, cx - 1, hip_y, cx - 4 - squat, hip_y + 2 + squat / 2);
        line(grid, cx - 4 - squat, hip_y + 2 + squat / 2, cx - 4, base);
        line(grid, cx, hip_y, cx + 3 + squat, hip_y + 2 + squat / 2);
        line(grid, cx + 3 + squat, hip_y + 2 + squat / 2, cx + 3, base);
        // Arms reach from the shoulders to the grip on the bar.
        line(grid, cx - 2, sh_y, cx - 7, bar_y);
        line(grid, cx + 1, sh_y, cx + 6, bar_y);
        paint(grid, cx - 7, 0, cx + 7, base - 1, Color::rgb(255, 205, 150));

        // Barbell: shaft plus bumper plates, more loaded as progress climbs.
        let half = 20.min(cx - 1);
        draw::hline(
            grid,
            (cx - half).max(0) as usize,
            (cx + half) as usize,
            bar_y.max(0) as usize,
        );
        let plates = 1 + (p * 3.0).min(2.0) as i32;
        let colors = [
            Color::rgb(230, 50, 50),
            Color::rgb(50, 110, 240),
            Color::rgb(250, 210, 40),
        ];
        for k in 0..plates {
            let ph = 4 - k;
            for side in [-1, 1] {
                let px = cx + side * (half - 5 + k * 2);
                for dx in 0..2 {
                    draw::vline(
                        grid,
                        (px + dx).max(0) as usize,
                        (bar_y - ph).max(0) as usize,
                        (bar_y + ph).max(0) as usize,
                    );
                }
                paint(grid, px, bar_y - ph, px + 1, bar_y + ph, colors[k as usize]);
            }
        }
        Ok(())
    }
}

// ── Tennis Rally ──────────────────────────────────────────────────────────────

/// Racket sprite (5 wide × 11 tall): strung oval head on a handle.
const RACKET: [&str; 11] = [
    ".###.", "#.#.#", "#####", "#.#.#", "#####", "#.#.#", ".###.", "..#..", "..#..", "..#..",
    "..#..",
];

struct TennisRally;
impl ProgressStyle for TennisRally {
    fn name(&self) -> &str {
        "tennis-rally"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "Two rackets trade a bouncing ball over the net; the court line paints in as the rally runs"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;
        let mid = wi / 2;

        // Court: solid green surface up to progress, chalk dots beyond.
        let done = (ctx.eased * w as f32) as i32;
        for x in 0..wi {
            if x < done || x % 3 == 0 {
                draw::dot_i(grid, x, base);
            }
        }
        paint(grid, 0, base, done - 1, base, Color::rgb(70, 200, 110));
        paint(grid, done, base, wi - 1, base, Color::rgb(110, 110, 120));

        // Net: post plus a chequered mesh.
        let net_top = base - (hi / 2).max(2);
        draw::vline(grid, mid as usize, net_top as usize, base as usize);
        for y in net_top..base {
            for x in mid - 1..=mid + 1 {
                if (x + y) % 2 == 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        paint(
            grid,
            mid - 1,
            net_top,
            mid + 1,
            base - 1,
            Color::rgb(240, 240, 240),
        );

        // Rackets at either baseline, rocking as they swing.
        let phase = (ctx.time * 0.7).fract() * 2.0;
        let swing = |near: f32| ((1.0 - near.min(1.0)) * 2.0) as i32;
        let lx = 1 + swing((phase - 0.0).abs().min((phase - 2.0).abs()) * 4.0);
        let rx = wi - 6 - swing((phase - 1.0).abs() * 4.0);
        sprite(grid, lx, 1, &RACKET);
        sprite(grid, rx, 1, &RACKET);
        paint(grid, lx, 0, lx + 4, 11, Color::rgb(90, 160, 255));
        paint(grid, rx, 0, rx + 4, 11, Color::rgb(255, 90, 90));

        // Ball: struck at head height, bounces once on the far side.
        let (x0, x1) = ((lx + 6) as f32, (rx - 2) as f32);
        let (u, dir) = if phase < 1.0 {
            (phase, 1.0)
        } else {
            (phase - 1.0, -1.0)
        };
        let bx = if dir > 0.0 {
            x0 + (x1 - x0) * u
        } else {
            x1 - (x1 - x0) * u
        };
        let ground = (base - 2) as f32;
        let by = if u < 0.75 {
            let q = u / 0.75;
            3.0 + (ground - 3.0) * q - 6.0 * q * (1.0 - q) * 1.5
        } else {
            let q = (u - 0.75) / 0.25;
            ground + (3.0 - ground) * q - 4.0 * q * (1.0 - q)
        };
        draw::fill_rect(grid, bx.max(0.0) as usize, by.max(0.0) as usize, 2, 2);
        paint(
            grid,
            bx as i32,
            by as i32,
            bx as i32 + 1,
            by as i32 + 1,
            Color::rgb(220, 255, 60),
        );
        Ok(())
    }
}

// ── Cycling Peloton ───────────────────────────────────────────────────────────

/// One road bike with rider, rear hub at `(x0, hub_y)`; pedals and spokes turn with `spin`.
fn cyclist(grid: &mut BrailleGrid, x0: i32, hub_y: i32, spin: f32) {
    let r = 3.2;
    let fx = x0 + 11;
    for hx in [x0, fx] {
        for k in 0..16 {
            let a = k as f32 / 16.0 * 2.0 * PI;
            draw::dot_i(
                grid,
                (hx as f32 + a.cos() * r).round() as i32,
                (hub_y as f32 + a.sin() * r).round() as i32,
            );
        }
        let (sx, sy) = (
            (spin.cos() * 2.0).round() as i32,
            (spin.sin() * 2.0).round() as i32,
        );
        line(grid, hx - sx, hub_y - sy, hx + sx, hub_y + sy);
    }
    // Frame: rear triangle, top tube and fork.
    let (crank_x, seat_x, bar_x, top_y) = (x0 + 5, x0 + 3, x0 + 10, hub_y - 6);
    line(grid, x0, hub_y, crank_x, hub_y);
    line(grid, crank_x, hub_y, seat_x, top_y);
    line(grid, x0, hub_y, seat_x, top_y);
    line(grid, seat_x, top_y, bar_x, top_y);
    line(grid, bar_x, top_y, fx, hub_y);
    // Rider: tucked torso, head, arm to the bars, leg to the turning pedal.
    let (sh_x, sh_y) = (x0 + 8, top_y - 4);
    line(grid, seat_x, top_y - 1, sh_x, sh_y);
    disc(grid, (sh_x + 2) as f32, (sh_y - 1) as f32, 1.2);
    line(grid, sh_x, sh_y, bar_x, top_y);
    let (pdx, pdy) = (
        (spin.cos() * 2.0).round() as i32,
        (spin.sin() * 2.0).round() as i32,
    );
    let knee = (seat_x + 4, top_y + 1 + pdy / 2);
    line(grid, seat_x, top_y - 1, knee.0, knee.1);
    line(grid, knee.0, knee.1, crank_x + pdx, hub_y + pdy);
}

struct CyclingPeloton;
impl ProgressStyle for CyclingPeloton {
    fn name(&self) -> &str {
        "cycling-peloton"
    }
    fn theme(&self) -> &str {
        "sports"
    }
    fn describe(&self) -> &str {
        "A tight pack of three road cyclists, wheels spinning and legs pedalling, rides the road to the finish"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 48 || h < 12 {
            draw::hline(
                grid,
                0,
                (ctx.eased * w as f32) as usize,
                h.saturating_sub(1),
            );
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let base = hi - 1;

        // Road with centre dashes streaming past.
        draw::hline(grid, 0, w - 1, base as usize);
        let scroll = (ctx.time * 24.0) as i32;
        paint(grid, 0, base, wi - 1, base, Color::rgb(120, 120, 130));

        // Pack of three; leader in yellow.
        let pack = 56;
        let x = 4 + (ctx.eased * (wi - pack - 4).max(0) as f32) as i32;
        let hub_y = base - 4;
        let jerseys = [
            Color::rgb(60, 140, 255),
            Color::rgb(255, 80, 80),
            Color::rgb(255, 215, 0),
        ];
        for (k, jersey) in jerseys.iter().enumerate() {
            let rx = x + k as i32 * 19;
            let spin = ctx.time * 9.0 + k as f32 * 1.3;
            cyclist(grid, rx, hub_y, spin);
            paint(grid, rx - 3, 0, rx + 14, hub_y - 6, *jersey);
            paint(
                grid,
                rx - 3,
                hub_y - 5,
                rx + 14,
                base - 1,
                Color::rgb(200, 200, 210),
            );
        }
        // Wind lines ahead of the pack's wake.
        for k in 0..3 {
            let y = 3 + k * 3;
            let x0 = (x - 4 - (scroll + k * 5).rem_euclid(8)).max(0);
            draw::hline(grid, (x0 - 3).max(0) as usize, x0 as usize, y as usize);
        }
        Ok(())
    }
}
