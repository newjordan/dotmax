//! Animals-themed progress bars — eleven distinct creatures drawn in braille dots.
//!
//! Every bar is stateless: all motion comes from `ctx.time` (for perpetual
//! animation) and `ctx.eased` (for progress-driven advancement). The bars use
//! only the `draw::` helpers so all writes are silently bounds-safe.
//!
//! Styles in this file:
//! - `caterpillar`  — round-segmented crawler munching the leaves along a twig
//! - `snail`        — spiral-shelled snail with eye stalks on a slime trail
//! - `inchworm`     — looping arch that stretches flat along a ruler
//! - `fish-school`  — forked-tail fish swimming in formation, trailing bubbles
//! - `snake`        — banded S-curve body with a head and flicking tongue
//! - `rabbit-hops`  — parabolic jump arcs across discrete hop chunks
//! - `paw-prints`   — four-toed prints stepping left-right one by one
//! - `bird-flock`   — V-formation of flapping gulls
//! - `turtle`       — plate-shelled turtle plodding over sand
//! - `ant-march`    — six-legged ants hauling crumbs toward an anthill
//! - `cat-prowl`    — tabby with pointed ears and curled tail chasing yarn

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — meadow green into honey. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(124, 204, 92);
const TINT_END: Color = Color::rgb(240, 184, 64);

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

/// All styles in the `animals` theme.
///
/// Returns one boxed [`ProgressStyle`] per animal variant, ready to be mixed
/// into a gallery or driven individually.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Caterpillar),
        Box::new(Snail),
        Box::new(Inchworm),
        Box::new(FishSchool),
        Box::new(Snake),
        Box::new(Tinted(RabbitHops)),
        Box::new(PawPrints),
        Box::new(BirdFlock),
        Box::new(Turtle),
        Box::new(AntMarch),
        Box::new(CatProwl),
    ]
}

// ── Sprite helpers ───────────────────────────────────────────────────────────

/// Stamp an ASCII sprite (`#` = dot) with its top-left corner at `(x, y)`.
fn stamp(grid: &mut BrailleGrid, rows: &[&str], x: i32, y: i32) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            if c == b'#' {
                draw::dot_i(grid, x + dx as i32, y + dy as i32);
            }
        }
    }
}

/// Filled disc of dots centred on `(cx, cy)`.
fn disc(grid: &mut BrailleGrid, cx: i32, cy: i32, r: f32) {
    let ri = r.ceil() as i32;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            if (dx * dx + dy * dy) as f32 <= r * r {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }
    }
}

/// Colour every non-empty cell overlapping the dot rectangle `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    grid.enable_color_support();
    let (cw, ch) = grid.dimensions();
    let (cx1, cy1) = ((x1 / 2).min(cw as i32 - 1), (y1 / 4).min(ch as i32 - 1));
    for cy in (y0.max(0) / 4)..=cy1 {
        for cx in (x0.max(0) / 2)..=cx1 {
            let c = grid.get_char(cx as usize, cy as usize);
            if c != '\u{2800}' && c != ' ' {
                let _ = grid.set_cell_color(cx as usize, cy as usize, color);
            }
        }
    }
}

// ── Caterpillar ──────────────────────────────────────────────────────────────

struct Caterpillar;
impl ProgressStyle for Caterpillar {
    fn name(&self) -> &str {
        "caterpillar"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Round-segmented green caterpillar with a red head and antennae, rippling along a twig and munching the leaves ahead"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let twig = hi - 1;
        draw::hline(grid, 0, w - 1, twig as usize);
        // Head position: the caterpillar eats its way from left to right.
        let head_x = 26 + (ctx.eased * (wi - 30).max(0) as f32) as i32;
        // Leaves still standing ahead of the head; eaten ones leave a stub.
        for k in 0..=(wi / 10) {
            let lx = k * 10 + 5;
            if lx > head_x + 3 {
                stamp(
                    grid,
                    &[" ### ", "#####", "#####", " ### ", "  #  "],
                    lx - 2,
                    twig - 5,
                );
            } else {
                draw::dot_i(grid, lx, twig - 1);
            }
        }
        paint(grid, 0, twig - 6, wi, twig, Color::rgb(70, 160, 70));
        paint(grid, 0, twig, wi, twig, Color::rgb(150, 105, 60));
        // Body: round segments; a hump ripples tail→head (peristaltic crawl).
        let base = twig - 4;
        for i in (1..=6i32).rev() {
            let wave = ((i as f32 * 0.9 - ctx.time * 2.0 * PI * 0.75).sin()).max(0.0);
            let sy = base - (wave * 2.5).round() as i32;
            let sx = head_x - i * 5 + 1;
            disc(grid, sx, sy, 2.4);
            draw::dot_i(grid, sx, sy + 3);
        }
        paint(
            grid,
            head_x - 34,
            0,
            head_x - 3,
            twig - 1,
            Color::rgb(130, 220, 70),
        );
        // Head with antennae that twitch.
        let twitch = ((ctx.time * 3.0) as i32).rem_euclid(2);
        disc(grid, head_x + 1, base - 1, 3.0);
        draw::dot_i(grid, head_x + 2, base - 5);
        draw::dot_i(grid, head_x + 3 + twitch, base - 6);
        draw::dot_i(grid, head_x - 1, base - 5);
        draw::dot_i(grid, head_x - 1 + twitch, base - 6);
        paint(
            grid,
            head_x - 2,
            0,
            head_x + 5,
            twig - 1,
            Color::rgb(235, 80, 60),
        );
        Ok(())
    }
}

// ── Snail ─────────────────────────────────────────────────────────────────────

/// Snail facing right: spiral shell riding on a foot that tapers to the head.
const SNAIL: [&str; 10] = [
    "    #####          ",
    "  ##     ##        ",
    " #  #####  #       ",
    "#  #     #  #      ",
    "# #  ##   # #   ## ",
    "# #  # #  # #  ####",
    "#  #  ## #  # #####",
    " #  ####   # ######",
    "  ##     ## #######",
    "####################",
];

struct Snail;
impl ProgressStyle for Snail {
    fn name(&self) -> &str {
        "snail"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Spiral-shelled snail with waving eye stalks gliding right on a glistening slime trail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let sx = (ctx.eased * (wi - 22).max(0) as f32) as i32;
        let top = ground - 9;
        // Slime trail: glints shimmer along the path already travelled.
        let shimmer = (ctx.time * 6.0) as i32;
        for x in 0..sx.max(0) + 2 {
            if (x + shimmer) % 5 != 0 {
                draw::dot_i(grid, x, ground);
            }
        }
        paint(grid, 0, ground, sx + 1, ground, Color::rgb(150, 190, 230));
        stamp(grid, &SNAIL, sx, top);
        // Eye stalks sway on the time clock.
        let sway = ((ctx.time * PI).sin() * 1.4).round() as i32;
        for j in 1..=4i32 {
            let lean = (sway * j) / 4;
            draw::dot_i(grid, sx + 16 + lean, top + 4 - j);
            draw::dot_i(grid, sx + 18 + lean + j / 2, top + 4 - j);
        }
        disc(grid, sx + 16 + sway, top - 1, 0.9);
        disc(grid, sx + 20 + sway, top - 1, 0.9);
        paint(
            grid,
            sx + 12,
            top - 2,
            sx + 22,
            ground,
            Color::rgb(190, 210, 150),
        );
        paint(grid, sx, top, sx + 11, ground - 1, Color::rgb(220, 140, 70));
        Ok(())
    }
}

// ── Inchworm ─────────────────────────────────────────────────────────────────

struct Inchworm;
impl ProgressStyle for Inchworm {
    fn name(&self) -> &str {
        "inchworm"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Inchworm looping into an arch and stretching flat as it measures its way along a ruler"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ruler = hi - 1;
        // Ruler with inch ticks; the measured part is lit.
        draw::hline(grid, 0, w - 1, ruler as usize);
        for x in (0..wi).step_by(4) {
            let tall = if x % 16 == 0 { 3 } else { 1 };
            for d in 1..=tall {
                draw::dot_i(grid, x, ruler - d);
            }
        }
        // Gait: three loops per 4 s. First half the head reaches forward
        // (arch flattens); second half the tail catches up (arch rises).
        let cyc = (ctx.time * 0.75).fract();
        let (min_len, max_len) = (8.0f32, 22.0f32);
        let len = if cyc < 0.5 {
            min_len + (max_len - min_len) * (cyc * 2.0)
        } else {
            max_len - (max_len - min_len) * ((cyc - 0.5) * 2.0)
        };
        let base = ctx.eased * (wi as f32 - max_len - 4.0).max(0.0);
        let tail = if cyc < 0.5 {
            base
        } else {
            base + max_len - len
        };
        let arch = ((max_len - len) * 0.62 + 1.0).min((hi - 4).max(1) as f32);
        let foot = ruler - 2;
        paint(
            grid,
            0,
            ruler - 3,
            tail as i32 + 2,
            ruler,
            Color::rgb(240, 200, 90),
        );
        paint(
            grid,
            tail as i32 + 3,
            ruler - 3,
            wi,
            ruler,
            Color::rgb(120, 110, 90),
        );
        // Body: a thick half-sine arch from tail to head.
        let steps = (len * 2.0) as i32;
        for s in 0..=steps {
            let f = s as f32 / steps.max(1) as f32;
            let x = (tail + f * len).round() as i32;
            let y = (foot as f32 - (f * PI).sin() * arch).round() as i32;
            draw::dot_i(grid, x, y);
            draw::dot_i(grid, x, y - 1);
        }
        // Head with a stubby antenna at the leading end; clasping feet.
        let hx = (tail + len).round() as i32;
        disc(grid, hx + 1, foot - 1, 1.5);
        draw::dot_i(grid, hx + 3, foot - 3);
        draw::dot_i(grid, tail as i32 - 1, foot);
        paint(
            grid,
            tail as i32 - 2,
            0,
            hx + 4,
            foot,
            Color::rgb(140, 230, 80),
        );
        Ok(())
    }
}

// ── Fish School ───────────────────────────────────────────────────────────────

/// Fish facing right: forked tail, oval body, open eye.
const FISH: [&str; 7] = [
    "     #####   ",
    "#  ########  ",
    "## ###### ## ",
    " ############",
    "## ##########",
    "#  ######### ",
    "     #####   ",
];

struct FishSchool;
impl ProgressStyle for FishSchool {
    fn name(&self) -> &str {
        "fish-school"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "School of forked-tail fish swimming right in formation, bobbing and trailing bubbles"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let lead = 14 + (ctx.eased * (wi - 28).max(0) as f32) as i32;
        let colors = [
            Color::rgb(255, 150, 60),
            Color::rgb(255, 210, 80),
            Color::rgb(90, 200, 255),
        ];
        // Formation: leader plus two wingmen behind, each on its own bob phase.
        let slots = [(0i32, 4i32), (-15, 0), (-15, 9)];
        for (i, &(dx, dy)) in slots.iter().enumerate() {
            let bob = ((ctx.time * 2.0 * PI * 0.5 + i as f32 * 2.1).sin() * 1.2).round() as i32;
            let (fx, fy) = (lead + dx, (dy + bob).clamp(0, (hi - 7).max(0)));
            stamp(grid, &FISH, fx, fy);
            paint(grid, fx, fy, fx + 12, fy + 6, colors[i % colors.len()]);
        }
        // Bubbles drift up from the wake on the time clock.
        for k in 0..6i32 {
            let bx = lead - 20 - k * 7;
            if bx < 0 {
                break;
            }
            let by = hi - 1 - ((ctx.time * 6.0 + k as f32 * 5.0) as i32).rem_euclid(hi.max(1));
            draw::dot_i(grid, bx, by);
            draw::dot_i(grid, bx + 1, by - 1);
            paint(grid, bx, by - 1, bx + 1, by, Color::rgb(170, 220, 255));
        }
        Ok(())
    }
}

// ── Snake ─────────────────────────────────────────────────────────────────────

/// Snake head facing right, with an open eye.
const SNAKE_HEAD: [&str; 5] = [" ####  ", "## ### ", "#######", "###### ", " ####  "];

struct Snake;
impl ProgressStyle for Snake {
    fn name(&self) -> &str {
        "snake"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Banded snake slithering in S-curves, tapered tail to a wide head with a flicking forked tongue"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = hi as f32 / 2.0 - 0.5;
        let amp = (hi as f32 / 2.0 - 3.5).clamp(0.0, 4.0);
        let head_x = 8 + (ctx.eased * (wi - 18).max(0) as f32) as i32;
        // Body: a travelling sine, thickening from a thin tail to the neck.
        let y_at = |x: i32| mid + ((x as f32 * 0.3) - ctx.time * 2.0 * PI * 0.5).sin() * amp;
        for x in 0..head_x {
            let thick = 2 + (2 * x) / head_x.max(1);
            let y = y_at(x).round() as i32;
            for d in 0..thick {
                draw::dot_i(grid, x, y - thick / 2 + d);
            }
        }
        // Alternating bands along the body.
        for cx in 0..=(head_x / 2) {
            let band = if (cx / 2) % 2 == 0 {
                Color::rgb(110, 200, 80)
            } else {
                Color::rgb(230, 190, 60)
            };
            paint(grid, cx * 2, 0, cx * 2 + 1, hi, band);
        }
        // Head and forked tongue, flicking out twice a second.
        let hy = y_at(head_x).round() as i32 - 2;
        stamp(grid, &SNAKE_HEAD, head_x, hy);
        paint(
            grid,
            head_x,
            hy,
            head_x + 6,
            hy + 4,
            Color::rgb(110, 200, 80),
        );
        if (ctx.time * 2.0).fract() < 0.5 {
            draw::hline(
                grid,
                (head_x + 7).max(0) as usize,
                (head_x + 8).max(0) as usize,
                (hy + 2).max(0) as usize,
            );
            draw::dot_i(grid, head_x + 9, hy + 1);
            draw::dot_i(grid, head_x + 9, hy + 3);
            paint(
                grid,
                head_x + 7,
                hy + 1,
                head_x + 9,
                hy + 3,
                Color::rgb(240, 70, 70),
            );
        }
        Ok(())
    }
}

// ── Rabbit Hops ───────────────────────────────────────────────────────────────

struct RabbitHops;
impl ProgressStyle for RabbitHops {
    fn name(&self) -> &str {
        "rabbit-hops"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Rabbit hopping across in parabolic arcs — each hop a discrete progress chunk"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wi = w as i32;
        let hi = h as i32;
        let ground = hi - 1;

        // Meadow: dotted ground line across the full width, with grass tufts.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, ground);
        }
        for k in 0..=(wi / 9) {
            let gx = k * 9 + 4;
            draw::dot_i(grid, gx, ground - 1);
            draw::dot_i(grid, gx + 1, ground - 2);
        }

        // Hop geometry: the rabbit crosses in discrete parabolic chunks.
        let n_hops = 4i32;
        let x_min = 7.0f32;
        let x_max = (wi as f32 - 9.0).max(x_min + 1.0);
        let hop_index_f = ctx.eased * n_hops as f32;
        let current_hop = (hop_index_f as i32).min(n_hops - 1);
        let hop_frac = (hop_index_f - current_hop as f32).clamp(0.0, 1.0);
        let hop_x = |hop: i32, t: f32| -> f32 {
            let s = (hop as f32 + t) / n_hops as f32;
            x_min + s * (x_max - x_min)
        };
        let peak = (hi as f32 * 0.19).max(2.0);
        let arc_y = |t: f32| -> f32 { ground as f32 - 2.0 + 4.0 * peak * t * (t - 1.0) };

        // Paw prints where earlier hops landed.
        for hop in 0..current_hop {
            let px = hop_x(hop, 1.0) as i32;
            draw::dot_i(grid, px, ground - 1);
            draw::dot_i(grid, px + 1, ground - 1);
            draw::dot_i(grid, px + 3, ground - 2);
            draw::dot_i(grid, px + 4, ground - 2);
        }

        // Dotted trajectory trail behind the rabbit within the current hop.
        // The dash pattern crawls with time (rate 1.5/s: +6 slots per 4 s loop).
        let dash = ((ctx.time * 1.5) as i32).rem_euclid(3);
        let trail_steps = 24i32;
        for i in 0..trail_steps {
            let t = i as f32 / trail_steps as f32;
            if t >= hop_frac {
                break;
            }
            if (i + dash) % 3 == 0 {
                draw::dot_i(grid, hop_x(current_hop, t) as i32, arc_y(t) as i32);
            }
        }

        // Rabbit anchor: bottom-centre of the body, riding the arc plus a
        // gentle time-driven bounce (0.25 Hz-multiple: seamless 4 s loop).
        let bounce = (ctx.time * PI).sin().abs() * 1.2;
        let rx = hop_x(current_hop, hop_frac).round() as i32;
        let ry = (arc_y(hop_frac) - bounce).round() as i32;

        // Body: filled ellipse (10 wide, 5 tall), facing right.
        for dy in -5i32..=0 {
            let fy = (dy as f32 + 2.5) / 3.0;
            let span = ((1.0 - fy * fy).max(0.0)).sqrt() * 4.6;
            let lo = (-span).round() as i32;
            let hi_x = span.round() as i32;
            for dx in lo..=hi_x {
                draw::dot_i(grid, rx + dx, ry + dy);
            }
        }
        // Head: filled disc ahead of the body, one blank dot left as the eye.
        for dy in -7i32..=-4 {
            for dx in 4i32..=8 {
                let ex = dx as f32 - 6.0;
                let ey = dy as f32 + 5.5;
                if ex * ex / 5.5 + ey * ey / 3.2 <= 1.0 && !(dx == 7 && dy == -6) {
                    draw::dot_i(grid, rx + dx, ry + dy);
                }
            }
        }
        // Ears: two slanted lines; the back ear flicks on a 2/s time slot.
        let flick = ((ctx.time * 2.0) as i32).rem_euclid(2);
        for j in 1..=3i32 {
            draw::dot_i(grid, rx + 4, ry - 7 - j);
            draw::dot_i(grid, rx + 6 + (j > 2) as i32 * flick, ry - 7 - j);
        }
        // Tail puff and airborne feet.
        draw::dot_i(grid, rx - 5, ry - 4);
        draw::dot_i(grid, rx - 6, ry - 3);
        draw::dot_i(grid, rx - 3, ry + 1);
        draw::dot_i(grid, rx - 4, ry + 1);
        draw::dot_i(grid, rx + 3, ry + 1);
        Ok(())
    }
}

// ── Paw Prints ────────────────────────────────────────────────────────────────

/// One paw print, toes pointing right: heel pad plus four toe beans.
const PAW: [&str; 9] = [
    "    ##  ", "    ##  ", " ##   ##", "####  ##", "#####   ", "####  ##", " ##   ##", "    ##  ",
    "    ##  ",
];

struct PawPrints;
impl ProgressStyle for PawPrints {
    fn name(&self) -> &str {
        "paw-prints"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Four-toed paw prints stepping left-right across the bar, the freshest print pressing in"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let stride = 11i32;
        let count = ((wi - 8) / stride + 1).max(1);
        let shown = (ctx.eased * count as f32).ceil() as i32;
        for k in 0..shown.min(count) {
            let x = 1 + k * stride;
            let y = if k % 2 == 0 { (hi - 9).max(0) } else { 0 };
            let newest = k == shown - 1 && ctx.eased < 1.0;
            // The newest print flickers as it is pressed into the ground.
            if newest && (ctx.time * 4.0).fract() < 0.3 {
                stamp(grid, &PAW[2..7], x, y + 2);
            } else {
                stamp(grid, &PAW, x, y);
            }
            let age = (shown - 1 - k) as f32 / count as f32;
            let color = ctx.palette.sample(1.0 - age);
            paint(grid, x, y, x + 7, y + 8, color);
        }
        Ok(())
    }
}

// ── Bird Flock ────────────────────────────────────────────────────────────────

/// Gull wing-beat frames: wings up, gliding "m", wings down.
const GULL: [[&str; 5]; 3] = [
    [
        "##         ##",
        "  ##     ##  ",
        "    ## ##    ",
        "     ###     ",
        "      #      ",
    ],
    [
        "             ",
        "  ###   ###  ",
        " #   # #   # ",
        "#     #     #",
        "             ",
    ],
    [
        "             ",
        "     ###     ",
        "   ## # ##   ",
        " ##       ## ",
        "#           #",
    ],
];

struct BirdFlock;
impl ProgressStyle for BirdFlock {
    fn name(&self) -> &str {
        "bird-flock"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "V-formation of flapping gulls flying right, each on its own wing-beat"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let lead = 30 + (ctx.eased * (wi - 43).max(0) as f32) as i32;
        let mid = (hi / 2 - 3).max(0);
        let slots = [(0i32, 0i32), (-15, -5), (-15, 5), (-30, -6), (-30, 6)];
        for (i, &(dx, dy)) in slots.iter().enumerate() {
            // Beat cycle up → glide → down → glide, staggered per bird.
            let beat = ((ctx.time * 6.0) as i32 + i as i32).rem_euclid(4);
            let frame = [0usize, 1, 2, 1][beat as usize];
            let (bx, by) = (lead + dx, (mid + dy).clamp(0, (hi - 5).max(0)));
            stamp(grid, &GULL[frame], bx, by);
            let color = ctx.palette.sample(i as f32 / 4.0);
            paint(grid, bx, by, bx + 12, by + 4, color);
        }
        Ok(())
    }
}

// ── Turtle ────────────────────────────────────────────────────────────────────

/// Turtle facing right: domed shell with plate seams, head with an eye, tail.
const TURTLE: [&str; 7] = [
    "      #######          ",
    "    ###########        ",
    "   ###  ###  ###   ### ",
    "  #### ##### ####  # ##",
    " ####  #####  #########",
    "#######################",
    "  ##################   ",
];

/// Two walking poses for the four stubby legs.
const TURTLE_LEGS: [[&str; 2]; 2] = [
    ["  ###          ###    ", "  ##            ##    "],
    ["   ###       ###      ", "    ##       ##       "],
];

struct Turtle;
impl ProgressStyle for Turtle {
    fn name(&self) -> &str {
        "turtle"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Plate-shelled turtle plodding right on stubby legs, head bobbing, leaving a footprint trail in the sand"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let tx = (ctx.eased * (wi - 24).max(0) as f32) as i32;
        let pose = ((ctx.time * 2.0) as i32).rem_euclid(2) as usize;
        let top = ground - 9;
        // Sand, with footprints along the stretch already walked.
        draw::hline(grid, 0, w - 1, ground as usize);
        for x in (2..tx.max(0)).step_by(5) {
            draw::dot_i(grid, x, ground - 1);
        }
        paint(grid, 0, ground - 1, wi, ground, Color::rgb(210, 180, 120));
        stamp(grid, &TURTLE, tx, top);
        stamp(grid, &TURTLE_LEGS[pose], tx, top + 7);
        // Head bobs a dot forward on each step.
        draw::dot_i(grid, tx + 23, top + 3 + pose as i32);
        paint(
            grid,
            tx + 18,
            top + 2,
            tx + 23,
            ground - 1,
            Color::rgb(170, 215, 120),
        );
        paint(
            grid,
            tx,
            top + 7,
            tx + 18,
            ground - 1,
            Color::rgb(170, 215, 120),
        );
        paint(grid, tx, top, tx + 17, top + 6, Color::rgb(70, 150, 80));
        Ok(())
    }
}

// ── Ant March ─────────────────────────────────────────────────────────────────

/// Ant facing right — abdomen, waist, thorax, head, antenna — in two leg poses.
const ANT: [[&str; 6]; 2] = [
    [
        "          # ",
        " ###     #  ",
        "##### #####",
        "###########",
        " ###  # # # ",
        "     #  #  #",
    ],
    [
        "         #  ",
        " ###     #  ",
        "##### #####",
        "###########",
        " ### # # #  ",
        "    #  # #  ",
    ],
];

struct AntMarch;
impl ProgressStyle for AntMarch {
    fn name(&self) -> &str {
        "ant-march"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Single file of six-legged ants marching toward an anthill, every other one hauling a crumb"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        // Anthill on the right edge.
        for r in 0..6i32 {
            draw::hline(
                grid,
                (wi - 12 + r).max(0) as usize,
                (wi - 1).max(0) as usize,
                (ground - 5 + r).max(0) as usize,
            );
        }
        paint(
            grid,
            wi - 12,
            ground - 6,
            wi,
            ground,
            Color::rgb(170, 120, 70),
        );
        // The column reaches further as progress grows; ants scroll forward one
        // spacing per 4 s loop so the march is seamless.
        let front = (ctx.eased * (wi - 14).max(0) as f32) as i32;
        let spacing = 16i32;
        let shift = ((ctx.time * 4.0) as i32).rem_euclid(spacing);
        let mut k = 0i32;
        while k * spacing <= front + spacing {
            let ax = front - 12 - k * spacing + shift;
            if ax + 11 >= 0 && ax <= front {
                let pose = ((ctx.time * 8.0) as i32 + k).rem_euclid(2) as usize;
                let ay = ground - 5;
                stamp(grid, &ANT[pose], ax, ay);
                paint(grid, ax, ay, ax + 11, ground, Color::rgb(220, 90, 60));
                if k % 2 == 0 {
                    draw::fill_rect(
                        grid,
                        (ax + 6).max(0) as usize,
                        (ay - 3).max(0) as usize,
                        3,
                        3,
                    );
                    paint(
                        grid,
                        ax + 6,
                        ay - 3,
                        ax + 8,
                        ay - 2,
                        Color::rgb(250, 230, 160),
                    );
                }
            }
            k += 1;
        }
        Ok(())
    }
}

// ── Cat Prowl ─────────────────────────────────────────────────────────────────

/// Cat facing right: pointed ears, eyes, arched back and a question-mark tail.
const CAT: [&str; 9] = [
    "               #   #  ",
    "               ## ##  ",
    " #             #####  ",
    "#              # # #  ",
    "#              ###### ",
    "#  ############ ###   ",
    " # ##############     ",
    "  ###############     ",
    "   ##############     ",
];

/// Trotting leg poses.
const CAT_LEGS: [[&str; 3]; 2] = [
    [
        "   ##        ##   ",
        "  ##          ##  ",
        "  #            #  ",
    ],
    [
        "    ##      ##    ",
        "    ##      ##    ",
        "    #       #     ",
    ],
];

struct CatProwl;
impl ProgressStyle for CatProwl {
    fn name(&self) -> &str {
        "cat-prowl"
    }
    fn theme(&self) -> &str {
        "animals"
    }
    fn describe(&self) -> &str {
        "Orange tabby with pointed ears and a curled tail trotting right after a rolling ball of yarn"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let ground = hi - 1;
        let cx = (ctx.eased * (wi - 34).max(0) as f32) as i32;
        let pose = ((ctx.time * 4.0) as i32).rem_euclid(2) as usize;
        let top = ground - 12 + pose as i32;
        stamp(grid, &CAT, cx, top);
        stamp(grid, &CAT_LEGS[pose], cx, top + 9);
        // Tail tip flicks.
        if (ctx.time * 1.5).fract() < 0.5 {
            draw::dot_i(grid, cx + 2, top + 1);
        }
        paint(grid, cx - 1, top, cx + 21, ground, Color::rgb(245, 160, 70));
        // Yarn ball rolling ahead, its wrap lines turning with time.
        let (bx, by) = (cx + 28, ground - 3);
        disc(grid, bx, by, 3.0);
        let spin = (ctx.time * 4.0) as i32;
        for d in -2..=2i32 {
            if (d + spin).rem_euclid(2) == 0 {
                draw::dot_i(grid, bx + d, by - d);
            }
        }
        // A loose thread trails back toward the cat.
        for x in (cx + 22)..(bx - 3) {
            draw::dot_i(grid, x, ground - (x + spin).rem_euclid(3) / 2);
        }
        paint(
            grid,
            cx + 22,
            by - 3,
            bx + 3,
            ground,
            Color::rgb(240, 110, 170),
        );
        Ok(())
    }
}
