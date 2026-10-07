//! ATARI 2600 / early-arcade themed progress bars.
//!
//! Each style draws the game's own iconic screen in braille dot-art: Pong's
//! paddles, net and blocky score, Breakout's rainbow brick wall, Asteroids'
//! vector rocks, Missile Command's cities under falling warheads, Centipede's
//! segmented bug in a mushroom field, Adventure's castle, hero and dragon,
//! Pitfall Harry on his vine, Combat's treaded tanks, Kaboom's Mad Bomber and
//! buckets, the Yar and the Qotile's shield, and Lunar Lander's descent.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — warm CRT phosphor orange.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(228, 84, 38);
const TINT_END: Color = Color::rgb(255, 184, 28);

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

// ---------------------------------------------------------------------------
// Sprite helpers — tiny dot-art stamps shared by the pictorial styles.
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

/// [`sprite`] mirrored left-to-right (for the facing-left twin of a sprite).
fn sprite_flip(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        let n = line.len() as i32;
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + n - 1 - c as i32, y + r as i32);
            }
        }
    }
}

/// Solid block of dots from signed coordinates (clipped to the grid).
fn block(grid: &mut BrailleGrid, x: i32, y: i32, w: i32, h: i32) {
    for yy in y..y + h {
        for xx in x..x + w {
            draw::dot_i(grid, xx, yy);
        }
    }
}

/// Blocky 3×5 score digits, as the 2600 drew them with playfield pixels.
const DIGITS: [[&str; 5]; 10] = [
    ["###", "#.#", "#.#", "#.#", "###"],
    [".#.", "##.", ".#.", ".#.", "###"],
    ["###", "..#", "###", "#..", "###"],
    ["###", "..#", "###", "..#", "###"],
    ["#.#", "#.#", "###", "..#", "..#"],
    ["###", "#..", "###", "..#", "###"],
    ["###", "#..", "###", "#.#", "###"],
    ["###", "..#", "..#", "..#", "..#"],
    ["###", "#.#", "###", "#.#", "###"],
    ["###", "#.#", "###", "..#", "###"],
];

/// Draw digit `d` at `(x, y)`; each font pixel is `sx` dots wide.
fn digit(grid: &mut BrailleGrid, x: i32, y: i32, d: usize, sx: i32) {
    for (r, line) in DIGITS[d % 10].iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                block(grid, x + c as i32 * sx, y + r as i32, sx, 1);
            }
        }
    }
}

/// Triangle wave `0 → 1 → 0` with period 1 — a ball bouncing between walls.
fn tri(t: f32) -> f32 {
    1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs()
}

/// Deterministic hash of `i` into `0.0..1.0` (stable scatter, no RNG state).
fn hash01(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65536.0
}

/// All styles in the `atari` theme.
///
/// Returns eleven structurally distinct bars, each referencing a different
/// Atari-era game mechanic — from Pong's paddle rally to Lunar Lander's
/// vector descent. No two styles share the same geometry or algorithm.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Pong)),
        Box::new(Breakout),
        Box::new(Tinted(Asteroids)),
        Box::new(Tinted(MissileCommand)),
        Box::new(Tinted(Centipede)),
        Box::new(Tinted(Adventure)),
        Box::new(Tinted(Pitfall)),
        Box::new(Tinted(Combat)),
        Box::new(Tinted(Kaboom)),
        Box::new(Tinted(YarsRevenge)),
        Box::new(Tinted(LunarLander)),
    ]
}

// ---------------------------------------------------------------------------
// 1. Pong — two paddles rally a square ball over a dashed net; score = progress
// ---------------------------------------------------------------------------
struct Pong;
impl ProgressStyle for Pong {
    fn name(&self) -> &str {
        "pong"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Pong: two paddles rally a square ball over a dashed net; blocky score digits count up with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Dashed centre net.
        let mid = wi / 2;
        for y in (0..hi).filter(|y| y % 4 < 2) {
            draw::dot_i(grid, mid, y);
        }

        // Scoreboard: player score climbs 0→9 with progress, CPU lags behind.
        let score = ((ctx.progress * 9.0).round() as usize).min(9);
        digit(grid, mid - 12, 0, score, 2);
        digit(grid, mid + 6, 0, score / 3, 2);

        // Square ball crosses paddle-to-paddle and bounces off top/bottom.
        let span_y = (hi - 2).max(0) as f32;
        let ball_y = |t: f32| (tri(t / 1.9 + 0.3) * span_y) as i32;
        let bx = 3 + (tri(ctx.time / 2.8) * (wi - 8).max(0) as f32) as i32;
        let by = ball_y(ctx.time);
        block(grid, bx, by, 2, 2);

        // Paddles (2 dots wide) track the ball; the CPU side reacts late.
        let ph = (hi / 3 + 1).clamp(1, 6);
        let paddle = |lag: f32| (ball_y(ctx.time - lag) + 1 - ph / 2).clamp(0, (hi - ph).max(0));
        block(grid, 1, paddle(0.0), 2, ph);
        block(grid, wi - 3, paddle(0.2), 2, ph);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Breakout — rainbow brick rows, a paddle and a ball; bricks cleared = eased
// ---------------------------------------------------------------------------
struct Breakout;

/// Classic Breakout row colours, top row first.
const BRICK_ROWS: [Color; 4] = [
    Color::rgb(222, 64, 52),
    Color::rgb(240, 140, 40),
    Color::rgb(230, 206, 60),
    Color::rgb(80, 190, 90),
];

impl ProgressStyle for Breakout {
    fn name(&self) -> &str {
        "breakout"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Breakout: red/orange brick rows above a paddle and bouncing ball; bricks knocked out = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cw, ch) = grid.dimensions();
        if cw == 0 || ch == 0 {
            return Ok(());
        }
        let (w, h) = draw::dot_dims(grid);
        let (wi, hi) = (w as i32, h as i32);

        // One brick row per cell row (3 dots + mortar) over the top half.
        // Lower rows break first, with a hashed scatter so holes look punched.
        let rows = (ch / 2).max(1);
        let pitch = 8;
        let cols = (wi / pitch).max(1);
        let span = rows.saturating_sub(1).max(1) as f32;
        for r in 0..rows {
            for c in 0..cols {
                let i = (r as i32 * cols + c) as u32;
                let order = 0.98 * (0.65 * hash01(i) + 0.35 * (1.0 - r as f32 / span));
                if order >= ctx.eased {
                    block(grid, c * pitch, r as i32 * 4, pitch - 1, 3);
                }
            }
        }

        // Ball bounces around the open court below the wall.
        let top = rows as i32 * 4 + 1;
        let bx = (tri(ctx.time / 1.6) * (wi - 2).max(0) as f32) as i32;
        let by = top + (tri(ctx.time / 1.1) * (hi - top - 3).max(0) as f32) as i32;
        block(grid, bx, by, 2, 2);

        // Paddle on the bottom row follows the ball.
        let pw = 10.min(wi);
        let px = (bx - pw / 2 + 1).clamp(0, (wi - pw).max(0));
        block(grid, px, hi - 1, pw, 1);

        // Colour: rainbow brick rows, phosphor tint for the court.
        for r in 0..ch {
            let color = if r < rows {
                BRICK_ROWS[r % BRICK_ROWS.len()]
            } else {
                sample_tint(0.8)
            };
            draw::tint_row(grid, r, 0, cw - 1, color);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Asteroids — wireframe vector polygons shatter as progress rises
// ---------------------------------------------------------------------------
struct Asteroids;
impl ProgressStyle for Asteroids {
    fn name(&self) -> &str {
        "asteroids"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Asteroids: vector-wireframe rocks shatter as progress rises; a ship remains"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Draw a vector polygon (n-gon) centred at (cx, cy) with given radius.
        let draw_ngon = |grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32, n: usize, rot: f32| {
            if n < 2 {
                return;
            }
            for i in 0..n {
                let a0 = rot + 2.0 * PI * i as f32 / n as f32;
                let a1 = rot + 2.0 * PI * (i + 1) as f32 / n as f32;
                let x0 = cx + (r as f32 * a0.cos()) as i32;
                let y0 = cy + (r as f32 * a0.sin()) as i32;
                let x1 = cx + (r as f32 * a1.cos()) as i32;
                let y1 = cy + (r as f32 * a1.sin()) as i32;
                // Bresenham line via dot_i.
                let mut sx = x0;
                let mut sy = y0;
                let dx = (x1 - x0).abs();
                let dy = (y1 - y0).abs();
                let step_x: i32 = if x1 > x0 { 1 } else { -1 };
                let step_y: i32 = if y1 > y0 { 1 } else { -1 };
                let mut err = dx - dy;
                for _ in 0..(dx + dy + 1).min(256) {
                    draw::dot_i(grid, sx, sy);
                    if sx == x1 && sy == y1 {
                        break;
                    }
                    let e2 = 2 * err;
                    if e2 > -dy {
                        err -= dy;
                        sx += step_x;
                    }
                    if e2 < dx {
                        err += dx;
                        sy += step_y;
                    }
                }
            }
        };

        // Twinkling starfield backdrop (integer 4/s slots: seamless 4 s loop).
        let star_hash = |n: u32| -> u32 {
            let mut x = n.wrapping_mul(2_654_435_761);
            x ^= x >> 15;
            x.wrapping_mul(2_246_822_519)
        };
        let blink = ((ctx.time * 4.0) as u32).rem_euclid(8);
        for i in 0..24u32 {
            if (star_hash(i * 5 + 3) + blink) % 8 == 0 {
                continue; // this star is blinked off right now
            }
            let sx = (star_hash(i * 2 + 1) % w as u32) as i32;
            let sy = (star_hash(i * 7 + 5) % h as u32) as i32;
            draw::dot_i(grid, sx, sy);
        }

        // Asteroid field: 7 full-size rocks at fixed positions, blasted one by
        // one in a scattered order as progress rises. Each intact rock keeps
        // its full wireframe (progress = rocks destroyed, not rocks shrunk).
        let asteroid_data: [(f32, f32, f32, f32, usize); 7] = [
            (0.08, 0.32, 0.36, 0.0, 6),
            (0.22, 0.70, 0.30, 0.5, 5),
            (0.36, 0.26, 0.40, 1.1, 7),
            (0.52, 0.68, 0.34, 0.8, 6),
            (0.66, 0.28, 0.28, 0.3, 5),
            (0.81, 0.68, 0.40, 1.4, 7),
            (0.94, 0.30, 0.28, 0.9, 5),
        ];
        // Destruction order: scattered across the field, not left-to-right.
        let kill_rank: [f32; 7] = [3.0, 0.0, 5.0, 1.0, 6.0, 2.0, 4.0];
        let destroyed = ctx.eased * 7.0;
        for (i, &(fx, fy, fr, rot_off, sides)) in asteroid_data.iter().enumerate() {
            let cx = (fx * w as f32) as i32;
            let cy = (fy * h as f32) as i32;
            let r = ((h as f32 * fr) as i32).max(2);
            // Rotate by whole symmetry steps per 4 s loop → seamless.
            let step = 2.0 * PI / sides as f32;
            let spin = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rot = rot_off + ctx.time * spin * step * 0.25 * (1 + i % 2) as f32;
            let frac = destroyed - kill_rank[i];
            if frac >= 1.0 {
                // Rock already blasted: a few dust specks linger where it was.
                for j in 0..3u32 {
                    let dx = (star_hash(i as u32 * 13 + j * 3 + 1) % (r as u32 * 2 + 1)) as i32 - r;
                    let dy = (star_hash(i as u32 * 17 + j * 5 + 2) % (r as u32 + 1)) as i32 - r / 2;
                    draw::dot_i(grid, cx + dx, cy + dy);
                }
                continue;
            }
            if frac > 0.0 {
                // Mid-shatter: shards fly outward from the rock's position.
                for j in 0..sides {
                    let a = rot + 2.0 * PI * j as f32 / sides as f32;
                    let d = r as f32 * (0.4 + frac * 2.2);
                    let px = cx + (d * a.cos()) as i32;
                    let py = cy + (d * a.sin()) as i32;
                    draw::dot_i(grid, px, py);
                    draw::dot_i(grid, px + 1, py);
                }
                continue;
            }
            draw_ngon(grid, cx, cy, r, sides, rot);
            // A crater speck inside the wireframe sells the rock.
            draw::dot_i(grid, cx - r / 3, cy + r / 4);
            draw::dot_i(grid, cx + r / 3, cy - r / 4);
        }

        // Player ship: vector triangle near bottom-centre, thrust flickering
        // on an 8/s slot (seamless over the 4 s loop).
        let ship_cx = (w / 2) as i32;
        let ship_cy = (h * 7 / 10) as i32;
        let ship_r = ((h as f32 * 0.22).max(2.0)) as i32;
        draw_ngon(grid, ship_cx, ship_cy, ship_r, 3, -PI / 2.0);
        if ((ctx.time * 8.0) as i32).rem_euclid(2) == 0 {
            draw::dot_i(grid, ship_cx, ship_cy + ship_r + 1);
            draw::dot_i(grid, ship_cx - 1, ship_cy + ship_r + 2);
            draw::dot_i(grid, ship_cx + 1, ship_cy + ship_r + 2);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Missile Command — warheads fall on six cities; counter-missile fireballs
// ---------------------------------------------------------------------------
struct MissileCommand;

/// Dotted straight trail from `(x0, y0)` to `(x1, y1)`; `every` = dot spacing.
fn trail(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, every: i32) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for i in (0..=steps).filter(|i| i % every.max(1) == 0) {
        let t = i as f32 / steps as f32;
        draw::dot_i(
            grid,
            x0 + ((x1 - x0) as f32 * t).round() as i32,
            y0 + ((y1 - y0) as f32 * t).round() as i32,
        );
    }
}

impl ProgressStyle for MissileCommand {
    fn name(&self) -> &str {
        "missile-command"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Missile Command: enemy warheads streak down toward six cities while the central base's counter-missiles burst into fireballs; incoming missiles destroyed = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Ground, six cities and the central missile base.
        draw::hline(grid, 0, w - 1, h - 1);
        let city_x = [0.06, 0.18, 0.30, 0.62, 0.74, 0.86].map(|f: f32| (f * wi as f32) as i32);
        for &cx in &city_x {
            sprite(grid, cx, hi - 4, &[".#..#..", "##.###.", "#######"]);
        }
        let base = (wi / 2, hi - 4);
        sprite(
            grid,
            base.0 - 5,
            base.1,
            &["...###....", ".#######..", "##########"],
        );

        // Six incoming warheads; each one destroyed becomes a fireball,
        // linked to the base by its counter-missile's trail.
        let killed = ctx.eased * 6.0;
        for (k, &city) in city_x.iter().enumerate() {
            let sx = (hash01(k as u32 * 3 + 5) * wi as f32) as i32;
            let (tx, ty) = (city + 3, hi - 5);
            if (k as f32) < killed {
                let fx = sx + (tx - sx) * (2 + (k as i32 % 3)) / 8;
                let fy = 2 + (k as i32 % 3) * 2;
                trail(grid, base.0, base.1, fx, fy, 3);
                let r = 1.0 + 2.5 * tri(ctx.time * 0.9 + k as f32 * 0.37);
                for s in 0..12 {
                    let a = s as f32 * PI / 6.0;
                    draw::dot_i(
                        grid,
                        fx + (a.cos() * r * 1.2).round() as i32,
                        fy + (a.sin() * r).round() as i32,
                    );
                }
            } else {
                let u = (ctx.time * 0.3 + k as f32 * 0.21).fract();
                let hx = sx + ((tx - sx) as f32 * u) as i32;
                let hy = (ty as f32 * u) as i32;
                trail(grid, sx, 0, hx, hy, 1);
                block(grid, hx, hy, 2, 2);
            }
        }
        Ok(())
    }
}
// ---------------------------------------------------------------------------
// 5. Centipede — a segmented bug snakes through a mushroom field; shot = eased
// ---------------------------------------------------------------------------
struct Centipede;
impl ProgressStyle for Centipede {
    fn name(&self) -> &str {
        "centipede"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Centipede: a segmented bug with antennae wiggles through a mushroom field while the gnome shoots it down; segments shot = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Mushroom field: a fixed scatter of caps on stems.
        const MUSHROOM: [&str; 3] = ["###", "###", ".#."];
        let field = (hi - 8).max(1) as f32;
        for k in 0..(wi / 11).max(1) as u32 {
            let mx = (hash01(k * 2 + 1) * (wi - 4).max(1) as f32) as i32;
            let my = 1 + (hash01(k * 2 + 2) * field) as i32;
            sprite(grid, mx, my, &MUSHROOM);
        }

        // The centipede: round segments trailing a head along a wiggling path.
        // Ten segments to start; each 10% of progress shoots one off the tail.
        const SEGMENT: [&str; 4] = [".##.", "####", "####", ".##."];
        const HEAD: [&str; 5] = ["#..#", ".##.", "####", "####", ".##."];
        let alive = ((1.0 - ctx.eased) * 10.0).ceil() as i32;
        let lane = (hi / 3).max(1) as f32;
        let path = |s: f32| {
            let x = s.rem_euclid((wi + 48) as f32) as i32 - 8;
            let y = (lane + (s * 0.35).sin() * lane * 0.8) as i32;
            (x, y)
        };
        let head_s = ctx.time * 14.0;
        for i in (1..alive).rev() {
            let (x, y) = path(head_s - i as f32 * 5.0);
            sprite(grid, x, y, &SEGMENT);
        }
        if alive > 0 {
            let (x, y) = path(head_s);
            sprite(grid, x, y - 1, &HEAD);
        }

        // The gnome shooter at the bottom, firing a dashed laser upward.
        let gx = (wi / 2) + ((ctx.time * 1.3).sin() * wi as f32 * 0.3) as i32;
        sprite(grid, gx - 1, hi - 3, &[".#.", "###", "#.#"]);
        let shot = hi - 5 - ((ctx.time * 40.0) as i32).rem_euclid((hi - 4).max(1));
        draw::dot_i(grid, gx, shot);
        draw::dot_i(grid, gx, shot + 1);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Adventure — the square hero carries the sword to the castle; dragon chases
// ---------------------------------------------------------------------------
struct Adventure;
impl ProgressStyle for Adventure {
    fn name(&self) -> &str {
        "adventure"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Adventure: the square hero carries the arrow-sword toward a crenellated castle, a duck-like dragon on his heels; the portcullis rises with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // The castle: two crenellated towers, a battlemented wall, a gate.
        const CASTLE: [&str; 12] = [
            "#.#.#.....#.#.#",
            "#####.....#####",
            "#####.....#####",
            "#####.#.#.#####",
            "###############",
            "###############",
            "###############",
            "#####.....#####",
            "#####.....#####",
            "#####.....#####",
            "#####.....#####",
            "#####.....#####",
        ];
        let cx = wi - 16;
        let cy = hi - CASTLE.len() as i32;
        sprite(grid, cx, cy, &CASTLE);

        // Portcullis bars, lifting as the hero nears the gate.
        let gate_rows = 5;
        let lowered = gate_rows - (ctx.eased * gate_rows as f32).round() as i32;
        for r in 0..lowered {
            for gx in [cx + 6, cx + 8] {
                draw::dot_i(grid, gx, hi - gate_rows + r);
            }
        }

        // The hero (a square) carrying the arrow-shaped sword.
        let ground = hi - 4;
        let travel = (cx - 12).max(0) as f32;
        let hx = 2 + (ctx.eased * travel) as i32;
        block(grid, hx, ground, 3, 3);
        sprite(grid, hx + 4, ground, &["..#.", "####", "..#."]);

        // Yorgle the dragon waddles behind, jaw snapping with time.
        let jaw = if (ctx.time * 3.0).fract() < 0.5 {
            "##...."
        } else {
            "####.."
        };
        let dragon = [
            ".##...", "###...", jaw, ".##...", ".###..", "#####.", "######", ".####.", ".#..#.",
        ];
        let bob = ((ctx.time * 6.0).sin() * 1.5) as i32;
        sprite_flip(grid, hx - 14 + bob, hi - 10, &dragon);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Pitfall — Harry swings a vine over a tar pit; jungle scrolls with eased
// ---------------------------------------------------------------------------
struct Pitfall;
impl ProgressStyle for Pitfall {
    fn name(&self) -> &str {
        "pitfall"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Pitfall!: Harry swings on a vine over a tar pit under the jungle canopy; trees scroll by and the underground tunnel fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 4;

        // Leafy canopy along the top.
        for x in 0..wi {
            if hash01(x as u32 + 7) > 0.25 {
                draw::dot_i(grid, x, 0);
            }
            if hash01(x as u32 + 301) > 0.55 {
                draw::dot_i(grid, x, 1);
            }
        }

        // Tree trunks drifting past as Harry advances screen by screen.
        let pit = (wi / 2 - 13, wi / 2 + 13);
        let scroll = (ctx.eased * 64.0) as i32;
        let pitch = 22;
        for k in 0..=(wi / pitch + 1) {
            let tx = (k * pitch - scroll).rem_euclid(wi.max(1) + pitch) - 4;
            if tx + 2 < pit.0 || tx > pit.1 {
                block(grid, tx, 2, 2, (ground - 2).max(0));
            }
        }

        // Jungle floor (two dots thick) broken by the tar pit.
        for x in 0..wi {
            if x < pit.0 || x > pit.1 {
                draw::dot_i(grid, x, ground);
                draw::dot_i(grid, x, ground + 1);
            } else if (x + (ctx.time * 6.0) as i32) % 3 != 0 {
                // Tar shimmer.
                draw::dot_i(grid, x, ground + 1);
            }
        }

        // Underground tunnel: the progress fill along the bottom row.
        let fill = (ctx.eased * w as f32) as i32;
        if fill > 0 {
            block(grid, 0, hi - 1, fill, 1);
        }

        // The vine, pivoting from above the canopy.
        let pivot = (wi / 2, -6);
        let len = (ground - 5 - pivot.1).max(1);
        let theta = 0.6 * (ctx.time * 2.2).sin();
        let (s, c) = theta.sin_cos();
        for k in 0..=len {
            draw::dot_i(
                grid,
                pivot.0 + (k as f32 * s).round() as i32,
                pivot.1 + (k as f32 * c).round() as i32,
            );
        }

        // Harry hangs from the end: arms up, hat, torso, running legs.
        let end_x = pivot.0 + (len as f32 * s).round() as i32;
        let end_y = pivot.1 + (len as f32 * c).round() as i32;
        sprite(grid, end_x - 1, end_y, &["#.#", "###", ".#.", "###", "#.#"]);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Combat — two top-down tanks duel past barrier walls; hits = progress
// ---------------------------------------------------------------------------
struct Combat;
impl ProgressStyle for Combat {
    fn name(&self) -> &str {
        "combat"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Combat: two treaded tanks face off across barrier walls trading shells; the score digits count hits as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Top-down tank: treads, hull and a long barrel pointing right.
        const TANK: [&str; 5] = [
            "#####....",
            ".###.....",
            ".#########",
            ".###.....",
            "#####....",
        ];
        let ty = (hi - 5) / 2 + 2;
        let roll = ((ctx.time * 1.7).sin() * 1.2) as i32;
        let lx = 2 + roll;
        let rx = wi - 12 - roll;
        sprite(grid, lx, ty, &TANK);
        sprite_flip(grid, rx, ty, &TANK);

        // Barrier walls above and below the firing lane.
        let mid = wi / 2;
        block(grid, mid - 1, 6, 3, (ty - 6).max(0));
        block(grid, mid - 1, ty + 5, 3, (hi - ty - 5).max(0));

        // Shell: flies muzzle to muzzle, bursting on the far tank.
        let phase = (ctx.time / 1.2).fract();
        let (from, to) = (lx + 10, rx - 1);
        let shell_y = ty + 2;
        if phase < 0.75 {
            let sx = from + ((phase / 0.75) * (to - from) as f32) as i32;
            block(grid, sx, shell_y, 2, 1);
        } else {
            let r = 1 + ((phase - 0.75) * 12.0) as i32;
            for k in 0..8 {
                let a = k as f32 * PI / 4.0;
                draw::dot_i(
                    grid,
                    rx + 4 + (a.cos() * r as f32 * 1.5) as i32,
                    shell_y + (a.sin() * r as f32) as i32,
                );
            }
        }

        // Scores along the top: hits landed climb with progress.
        let hits = ((ctx.progress * 9.0).round() as usize).min(9);
        digit(grid, wi / 4, 0, hits, 2);
        digit(grid, wi * 3 / 4 - 6, 0, hits / 4, 2);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Kaboom — the Mad Bomber drops lit bombs; stacked buckets catch them
// ---------------------------------------------------------------------------
struct Kaboom;
impl ProgressStyle for Kaboom {
    fn name(&self) -> &str {
        "kaboom"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Kaboom!: the grinning Mad Bomber paces the wall dropping lit bombs into two stacked water buckets; the catch counter = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // The Mad Bomber paces along the top wall.
        let bomber_x = |t: f32| wi / 2 + 6 + ((t * 1.4).sin() * wi as f32 * 0.28) as i32;
        const BOMBER: [&str; 5] = [".###.", "#####", "#.#.#", ".###.", "#####"];
        let bx = bomber_x(ctx.time);
        sprite(grid, bx - 2, 0, &BOMBER);
        draw::hline(grid, 0, w - 1, 5);

        // Lit bombs, one released every 0.45 s from wherever he stood.
        let drop = 0.3;
        let speed = 11.0;
        let now = (ctx.time / drop).floor() as i32;
        let mut lowest: Option<(i32, i32)> = None;
        for k in 0..4 {
            let released = (now - k) as f32 * drop;
            let y = 6 + ((ctx.time - released) * speed) as i32;
            if y > hi - 5 {
                continue;
            }
            let x = bomber_x(released);
            sprite(grid, x - 1, y, &[".#.", "###", ".#."]);
            if (ctx.time * 8.0 + k as f32).fract() < 0.5 {
                draw::dot_i(grid, x + 1, y - 1); // fuse spark
            }
            match lowest {
                Some((_, ly)) if ly >= y => {}
                _ => lowest = Some((x, y)),
            }
        }

        // Two stacked water buckets slide under the lowest bomb.
        let target = lowest.map_or_else(|| bomber_x(ctx.time), |(x, _)| x);
        let ux = (target - 4).clamp(0, (wi - 8).max(0));
        for b in 0..2 {
            let y = hi - 4 + b * 2;
            sprite(grid, ux, y, &["#......#", ".######."]);
        }

        // Catch counter, top-left: 00 → 99 with progress.
        let caught = ((ctx.progress * 99.0).round() as usize).min(99);
        digit(grid, 0, 0, caught / 10, 1);
        digit(grid, 4, 0, caught % 10, 1);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Yars' Revenge — the Yar nibbles the Qotile's shield; erosion = progress
// ---------------------------------------------------------------------------
struct YarsRevenge;
impl ProgressStyle for YarsRevenge {
    fn name(&self) -> &str {
        "yars-revenge"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Yars' Revenge: the winged Yar hovers by the flickering neutral zone while the Qotile's cell shield erodes; erosion = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);

        // Shield: a bulging wall of 2×2 cells, eroded in a hashed order.
        let right = wi - 6;
        for row in 0..(hi / 2) {
            let bulge = 7 - ((row * 2 + 1 - hi / 2).abs() / 2).min(6);
            for col in 0..bulge {
                let i = (row * 16 + col) as u32;
                if hash01(i + 911) >= ctx.eased {
                    block(grid, right - col * 3, row * 2, 2, 2);
                }
            }
        }

        // The Qotile: a pulsing diamond behind the shield.
        let qy = hi / 2 - 2;
        let pulse = (ctx.time * 4.0).fract() < 0.5;
        let core: [&str; 4] = if pulse {
            [".#.", "###", "###", ".#."]
        } else {
            ["...", ".#.", ".#.", "..."]
        };
        sprite(grid, wi - 3, qy, &core);

        // Neutral zone: a flickering band of static.
        let frame = (ctx.time * 12.0) as u32;
        let zx = wi * 2 / 5;
        for y in 0..hi {
            for x in zx..zx + 6 {
                if hash01((y * 64 + x) as u32 ^ frame.wrapping_mul(7919)) > 0.62 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // The Yar: a fly with flapping wings, drifting toward the shield.
        let wings: [&str; 2] = if (ctx.time * 6.0).fract() < 0.5 {
            ["#...#", "##.##"]
        } else {
            ["....#", ".#.##"]
        };
        let yx = 4 + (ctx.eased * (zx - 12).max(0) as f32) as i32;
        let yy = (hi / 2 - 3) + ((ctx.time * 2.0).sin() * (hi as f32 / 4.0)) as i32;
        sprite(grid, yx, yy, &wings);
        sprite(grid, yx, yy + 2, &[".###.", "#####", ".###.", "..#.."]);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Lunar Lander — vector lander descends; fuel/altitude gauge on the side
// ---------------------------------------------------------------------------
struct LunarLander;
impl ProgressStyle for LunarLander {
    fn name(&self) -> &str {
        "lunar-lander"
    }
    fn theme(&self) -> &str {
        "atari"
    }
    fn describe(&self) -> &str {
        "Lunar Lander: a vector wireframe lander descends; altitude/fuel = progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Altitude gauge: vertical bar on the far right (2 dots wide).
        let gauge_x = w.saturating_sub(2);
        let gauge_h = h;
        let fuel_h = (ctx.eased * gauge_h as f32) as usize;
        // Empty gauge outline.
        draw::vline(grid, gauge_x, 0, gauge_h.saturating_sub(1));
        // Fuel fill from bottom.
        for gy in (gauge_h.saturating_sub(fuel_h))..gauge_h {
            draw::dot(grid, gauge_x + 1, gy);
        }

        // Lunar surface: irregular terrain at the bottom.
        let surface_y = h.saturating_sub(2);
        draw::hline(grid, 0, gauge_x.saturating_sub(1), surface_y);
        // Jagged peaks.
        let peak_xs = [w / 6, w / 3, w / 2, 2 * w / 3];
        for &px in &peak_xs {
            if px < gauge_x {
                draw::vline(grid, px, surface_y.saturating_sub(2), surface_y);
            }
        }

        // Lander: descends from top to surface as eased rises.
        // Wireframe: body (rectangle) + legs (two diagonal lines) + thruster dot.
        let lander_col_centre = (w / 2) as i32;
        let descent_range = surface_y.saturating_sub(6);
        let lander_y = (ctx.eased * descent_range as f32) as i32;

        let bw: i32 = (w as i32 / 8).clamp(2, 5);
        let bh: i32 = 2.max((h as i32 / 8).min(3));

        // Body rectangle.
        let bx0 = lander_col_centre - bw / 2;
        let bx1 = lander_col_centre + bw / 2;
        let by0 = lander_y;
        let by1 = lander_y + bh;
        // Top and bottom of body.
        for x in bx0..=bx1 {
            draw::dot_i(grid, x, by0);
            draw::dot_i(grid, x, by1);
        }
        // Sides.
        for y in by0..=by1 {
            draw::dot_i(grid, bx0, y);
            draw::dot_i(grid, bx1, y);
        }
        // Landing legs: diagonal lines down from body corners.
        let leg_len: i32 = bh.max(2);
        draw::dot_i(grid, bx0 - leg_len, by1 + leg_len);
        draw::dot_i(grid, bx0 - leg_len + 1, by1 + leg_len - 1);
        draw::dot_i(grid, bx0 - leg_len + 2, by1 + leg_len - 2);
        draw::dot_i(grid, bx1 + leg_len, by1 + leg_len);
        draw::dot_i(grid, bx1 + leg_len - 1, by1 + leg_len - 1);
        draw::dot_i(grid, bx1 + leg_len - 2, by1 + leg_len - 2);

        // Thruster exhaust: a pulsing dot below the body (only when descending).
        if ctx.eased < 0.95 {
            let pulse = ((ctx.time * 8.0).sin() > 0.0) as i32;
            draw::dot_i(grid, lander_col_centre, by1 + 1 + pulse);
        }

        Ok(())
    }
}
