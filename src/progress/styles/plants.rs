//! Plants / flora progress bars — living, growing, breathing.
//!
//! Ten structurally distinct styles, each driven by `ctx.eased` for growth
//! stage and `ctx.time` for sway / breeze animation. Every style uses a
//! completely different rendering algorithm: no two share the same structural
//! approach. Each paints its own natural colours (greens, earth, petals)
//! cell by cell rather than using the bar palette.
//!
//! | Name                | Mechanism                                              |
//! |---------------------|--------------------------------------------------------|
//! | `seedling`          | Seed tray sprouting: hooked shoot → two seed leaves    |
//! | `fern-fiddlehead`   | Logarithmic spiral unrolling (fiddlehead uncurling)    |
//! | `bamboo-shoot`      | Vertical segments shooting up one by one               |
//! | `ivy-trellis`       | Ivy with three-lobed leaves winding over a lattice     |
//! | `cactus-arms`       | Saguaros grow in turn, sprout arms and a crown flower  |
//! | `sunflower-seeds`   | Sunflowers open: petal ring round a phyllotaxis disc   |
//! | `mushroom-cap`      | Red spotted toadstools pop up one after another        |
//! | `succulent-rosette` | Potted echeveria: plump leaves fan outward in pairs    |
//! | `root-system`       | Soil cutaway: roots snake right, forking rootlets      |
//! | `bonsai`            | Potted S-trunk → branches → swelling cloud-pad foliage |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ── registry ─────────────────────────────────────────────────────────────────

/// All styles in the `plants` theme.
///
/// Returns ten structurally distinct plant-growth progress bars, each
/// mapping `ctx.eased` to visible growth and `ctx.time` to living motion.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Seedling),
        Box::new(FernFiddlehead),
        Box::new(BambooShoot),
        Box::new(IvyTrellis),
        Box::new(CactusArms),
        Box::new(SunflowerSeeds),
        Box::new(MushroomCap),
        Box::new(Succulent),
        Box::new(RootSystem),
        Box::new(Bonsai),
    ]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Integer sine — keeps per-bar code terse.
#[inline]
fn isin(angle: f32, amplitude: f32) -> i32 {
    (angle.sin() * amplitude).round() as i32
}

/// Tint the cell containing dot `(x, y)`; out-of-range dots are ignored.
fn tint_at(grid: &mut BrailleGrid, x: i32, y: i32, color: Color) {
    if x >= 0 && y >= 0 {
        let (cx, cy) = (x as usize / 2, y as usize / 4);
        draw::tint_row(grid, cy, cx, cx, color);
    }
}

/// Leaf green ramp: `0.0` deep forest green → `1.0` fresh yellow-green.
fn leafy(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::rgb(
        (40.0 + 130.0 * t) as u8,
        (130.0 + 90.0 * t) as u8,
        (50.0 + 30.0 * t) as u8,
    )
}

/// Set a dot and tint its cell in one go.
fn paint(grid: &mut BrailleGrid, x: i32, y: i32, color: Color) {
    draw::dot_i(grid, x, y);
    tint_at(grid, x, y, color);
}

// ── 1. Seedling — a seed tray sprouting in turn ──────────────────────────────
//
// A row of seeds in a soil bed germinate left to right. Each one swells,
// pushes up a hooked shoot, straightens, and opens two rounded seed leaves
// (cotyledons) that sway in the breeze.

struct Seedling;
impl ProgressStyle for Seedling {
    fn name(&self) -> &str {
        "seedling"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Seeds in a soil bed sprout left to right: a hooked shoot straightens and opens two swaying seed leaves"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let soil = Color::rgb(120, 80, 45);
        let seed_c = Color::rgb(215, 180, 110);
        let stem_c = Color::rgb(150, 215, 90);
        let leaf_c = Color::rgb(70, 185, 70);
        let ground = h as i32 - 2;

        // Soil bed: a solid crust line with crumbs underneath.
        for x in 0..w as i32 {
            paint(grid, x, ground, soil);
            if x % 3 == 0 {
                paint(grid, x, ground + 1, soil);
            }
        }

        let n = (w / 14).max(1);
        let spacing = w as f32 / n as f32;
        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            let x = (spacing * (i as f32 + 0.5)) as i32;
            if local < 0.15 {
                // A dormant seed resting on the soil.
                paint(grid, x, ground - 1, seed_c);
                paint(grid, x + 1, ground - 1, seed_c);
                continue;
            }
            let sway = isin(ctx.time * 1.5 + i as f32, 1.0);
            let rise = ((local - 0.15) / 0.45).min(1.0);
            let height = (1.0 + (h as f32 * 0.4) * rise) as i32;
            let top = ground - height;
            for y in top..ground {
                let dx = if y < ground - height / 2 { sway } else { 0 };
                paint(grid, x + dx, y, stem_c);
            }
            if local < 0.6 {
                // Hooked shoot: the tip still bent over.
                paint(grid, x + sway + 1, top, stem_c);
                paint(grid, x + sway + 2, top + 1, stem_c);
                continue;
            }
            // Two seed leaves opening into rounded ovals.
            let open = ((local - 0.6) / 0.4).min(1.0);
            let rx = 1.0 + 1.2 * open;
            let ry = 0.7 + 0.5 * open;
            for side in [-1.0f32, 1.0] {
                let lcx = (x + sway) as f32 + side * (rx + 0.5);
                let lcy = top as f32 - open;
                for yy in (lcy - ry - 2.0) as i32..=(lcy + ry + 2.0) as i32 {
                    for xx in (lcx - rx - 1.0) as i32..=(lcx + rx + 1.0) as i32 {
                        let fx = (xx as f32 - lcx) / rx;
                        // Leaves tilt upward away from the stem.
                        let fy = (yy as f32 - lcy + fx * side * 1.6) / ry;
                        if fx * fx + fy * fy <= 1.0 {
                            paint(grid, xx, yy, leaf_c);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// ── 2. Fern Fiddlehead — logarithmic spiral unrolling ────────────────────────
//
// A fiddlehead starts as a tight coil (eased=0) and progressively unrolls
// into an open frond (eased=1). The spiral uses polar coordinates.
// ctx.time adds a gentle oscillation so the frond breathes.

struct FernFiddlehead;
impl ProgressStyle for FernFiddlehead {
    fn name(&self) -> &str {
        "fern-fiddlehead"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A fern fiddlehead unrolls from a tight coil into an open frond as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let wf = w as f32;
        let hf = h as f32;

        // The frond unrolls left→right across the whole canvas: the rachis
        // (main stem) extends with eased, pinnae unfurl behind the advancing
        // tip, and the tip itself carries the uncurling fiddlehead coil.
        let x0 = 3.0f32;
        let y0 = hf - 2.0;
        let full_len = (wf - 12.0).max(2.0);
        let rise = hf * 0.45;

        // Breeze: a slow wave travels along the frond (0.25 Hz — seamless).
        let breeze = |s: f32| (ctx.time * PI * 0.5 + s * 3.0).sin() * 1.1 * s;

        // Rachis point at parameter s in 0..=1.
        let rachis = |s: f32| -> (f32, f32) {
            let x = x0 + s * full_len;
            let y = y0 - rise * s.powf(1.25) + breeze(s);
            (x, y)
        };

        // Draw the rachis up to the growth front, thickened near the base.
        let grown = ctx.eased.clamp(0.0, 1.0);
        let steps = (full_len * 1.5) as i32;
        for i in 0..=steps {
            let s = i as f32 / steps.max(1) as f32 * grown;
            if s > grown {
                break;
            }
            let (x, y) = rachis(s);
            draw::dot_i(grid, x.round() as i32, y.round() as i32);
            if s < 0.6 {
                draw::dot_i(grid, x.round() as i32, y.round() as i32 + 1);
            }
        }

        // Pinnae: paired leaflets sprouting along the grown rachis. Each pair
        // matures (lengthens) once the tip has passed it.
        let n_pinnae = 16i32;
        for k in 1..=n_pinnae {
            let sk = k as f32 / (n_pinnae + 1) as f32;
            if sk >= grown {
                break;
            }
            let maturity = ((grown - sk) / 0.12).clamp(0.0, 1.0);
            let len = (hf * 0.38) * (1.0 - 0.55 * sk) * maturity;
            let (px, py) = rachis(sk);
            for j in 1..=(len.round() as i32) {
                let jf = j as f32;
                // Upward leaflet sweeps back; downward one is shorter.
                draw::dot_i(
                    grid,
                    (px - jf * 0.45).round() as i32,
                    (py - jf * 0.9).round() as i32,
                );
                if jf < len * 0.7 {
                    draw::dot_i(
                        grid,
                        (px - jf * 0.35).round() as i32,
                        (py + jf * 0.8).round() as i32,
                    );
                }
            }
        }

        // Fiddlehead coil at the growth tip: a tight Archimedean curl that
        // unwinds as progress advances until only a small hook remains.
        let (tx, ty) = rachis(grown);
        let coil_turns = 0.4 + 2.4 * (1.0 - grown);
        let coil_r = 2.2 + 4.0 * (1.0 - grown);
        let coil_cx = tx + coil_r * 0.4;
        let coil_cy = ty - coil_r * 0.9;
        let coil_steps = (coil_turns * 24.0) as i32;
        for i in 0..=coil_steps {
            let t = i as f32 / coil_steps.max(1) as f32;
            let theta = t * coil_turns * 2.0 * PI + PI * 0.5;
            let r = coil_r * (1.0 - t * 0.85);
            draw::dot_i(
                grid,
                (coil_cx + r * theta.cos()).round() as i32,
                (coil_cy + r * theta.sin()).round() as i32,
            );
        }

        // Tint: deep green shifting to pale frond tip.
        let (cw, ch) = grid.dimensions();
        for row in 0..ch {
            let t = row as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = leafy(1.0 - t);
            draw::tint_row(grid, row, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 3. Bamboo Shoot — vertical segments shooting up ──────────────────────────
//
// Each segment is a cell-glyph column block. Segments appear one by one from
// the bottom as eased advances. Each joint has a horizontal line (node).
// ctx.time causes segments to sway slightly left-right as a column.

struct BambooShoot;
impl ProgressStyle for BambooShoot {
    fn name(&self) -> &str {
        "bamboo-shoot"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Bamboo culm grows segment by segment upward; joints appear at each node as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let (cw, ch) = grid.dimensions();
        let wi = w as i32;
        let hi = h as i32;
        let ground = hi - 1;

        // Dotted soil line the grove springs from.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, ground);
        }

        // A grove of culms revealed left→right: culm i shoots up while
        // `eased * n` passes through [i, i+1], each rising segment by segment.
        let n_culms = 6i32;
        let culm_max = (hi - 3).max(1) as f32;
        for i in 0..n_culms {
            let grow = (ctx.eased * n_culms as f32 - i as f32).clamp(0.0, 1.0);
            if grow <= 0.0 {
                break;
            }
            let cx = wi * (2 * i + 1) / (2 * n_culms);
            let culm_h = (grow * culm_max).round() as i32;
            // Sway: each culm leans on its own phase (0.25 Hz — seamless).
            let lean = (ctx.time * PI * 0.5 + i as f32 * 1.1).sin() * 1.6;

            for dy in 0..=culm_h {
                let xoff = (lean * dy as f32 / culm_max).round() as i32;
                let y = ground - 1 - dy;
                // Twin walls form the hollow culm.
                draw::dot_i(grid, cx - 1 + xoff, y);
                draw::dot_i(grid, cx + 1 + xoff, y);
                // Node (joint) line closes each segment.
                if dy % 4 == 3 {
                    draw::dot_i(grid, cx - 2 + xoff, y);
                    draw::dot_i(grid, cx + xoff, y);
                    draw::dot_i(grid, cx + 2 + xoff, y);
                }
            }
            // Tapered growing tip while the culm is still shooting.
            let tip_off = (lean * culm_h as f32 / culm_max).round() as i32;
            if grow < 1.0 {
                draw::dot_i(grid, cx + tip_off, ground - 2 - culm_h);
            }
            // Leaves at the upper nodes once tall enough, alternating sides.
            if culm_h >= 7 {
                let side = if i % 2 == 0 { 1i32 } else { -1 };
                for ld in 1..=4i32 {
                    draw::dot_i(
                        grid,
                        cx + tip_off + side * ld,
                        ground - 1 - culm_h + 4 - (ld + 1) / 2,
                    );
                    if culm_h >= 11 {
                        draw::dot_i(
                            grid,
                            cx + tip_off - side * ld,
                            ground - 1 - culm_h + 8 - (ld + 1) / 2,
                        );
                    }
                }
            }
        }

        // Gradient tint: deep green at base, yellow-green at tip.
        for cy in 0..ch {
            let t = 1.0 - cy as f32 / ch.saturating_sub(1).max(1) as f32;
            let color = leafy(t);
            draw::tint_row(grid, cy, 0, cw.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ── 4. Ivy Trellis — ivy climbing a diamond-lattice garden trellis ──────────
//
// A wooden lattice of crossed diagonal slats spans the bar. An ivy vine
// winds along it left→right with progress, sprouting three-lobed leaves on
// alternate sides; the leaves flutter between two poses with ctx.time.

struct IvyTrellis;
impl ProgressStyle for IvyTrellis {
    fn name(&self) -> &str {
        "ivy-trellis"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Ivy winds left to right across a wooden diamond-lattice trellis, sprouting fluttering three-lobed leaves"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let wood = Color::rgb(160, 115, 70);
        let vine = leafy(0.2);
        let hh = h as i32;

        // Lattice: two families of diagonal slats crossing into diamonds.
        for y in 0..hh {
            for x in 0..w as i32 {
                if (x + y).rem_euclid(10) == 0 || (x - y).rem_euclid(10) == 0 {
                    paint(grid, x, y, wood);
                }
            }
        }

        // Vine: a winding stem out to `reach`.
        let reach = (ctx.eased * w as f32) as i32;
        let mid = hh as f32 / 2.0;
        let stem_y = |x: i32| (mid + (x as f32 * 0.2).sin() * (hh as f32 * 0.22)) as i32;
        for x in 0..reach {
            paint(grid, x, stem_y(x), vine);
        }

        // Leaves on alternate sides; the newest ones near the tip are small.
        let mut i = 0;
        let mut x = 3;
        while x < reach {
            let up = i % 2 == 0;
            let sy = stem_y(x);
            let big = reach - x > 4;
            let flap = (ctx.time * 3.0 + i as f32 * 1.3).sin() > 0.0;
            let dir = if up { -1 } else { 1 };
            let c = leafy(if i % 3 == 0 { 0.5 } else { 0.25 });
            // Petiole from the stem to the leaf.
            paint(grid, x, sy + dir, c);
            let ly = sy + dir * 2;
            if big {
                // Three-lobed ivy leaf: a broad body with two side lobes.
                let lobe = if flap { 0 } else { dir };
                for dx in -2..=2 {
                    paint(grid, x + dx, ly, c);
                }
                for dx in -1..=1 {
                    paint(grid, x + dx, ly + dir, c);
                }
                paint(grid, x, ly + dir * 2, c);
                paint(grid, x - 2, ly - dir + lobe, c);
                paint(grid, x + 2, ly - dir + lobe, c);
            } else {
                paint(grid, x, ly, c);
                paint(grid, x + 1, ly, c);
            }
            i += 1;
            x += 5;
        }

        Ok(())
    }
}

// ── 5. Cactus Arms — saguaros grow, arms branch, a flower crowns them ───────
//
// Three saguaros on a sandy floor grow in turn. Each trunk rises with a
// rounded top, then its elbowed arms sprout at thresholds, and a pink flower
// opens on the crown once it is fully grown. A tumbleweed rolls past on time.

struct CactusArms;
impl ProgressStyle for CactusArms {
    fn name(&self) -> &str {
        "cactus-arms"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Three saguaro cacti rise in turn, sprout elbowed arms and a pink crown flower while a tumbleweed rolls by"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let green = Color::rgb(60, 170, 80);
        let sand = Color::rgb(210, 170, 100);
        let pink = Color::rgb(255, 110, 170);
        let weed = Color::rgb(150, 110, 60);
        let ground = h as i32 - 1;

        // Sandy floor: a dotted dune line.
        for x in 0..w as i32 {
            if x % 3 != 1 {
                paint(grid, x, ground, sand);
            }
        }

        // Tumbleweed: a little scribbled wheel rolling and bouncing (drawn
        // first so the cacti stay green when it rolls behind them).
        let span = w as f32 + 8.0;
        let tx = (ctx.time * 9.0).rem_euclid(span) - 4.0;
        let ty = ground as f32 - 2.0 - (ctx.time * 6.0).sin().abs() * 1.5;
        for k in 0..8 {
            let a = k as f32 / 8.0 * 2.0 * PI + ctx.time * 5.0;
            let rr = if k % 2 == 0 { 2.0 } else { 1.2 };
            paint(
                grid,
                (tx + a.cos() * rr * 1.2).round() as i32,
                (ty + a.sin() * rr).round() as i32,
                weed,
            );
        }

        let n = 3usize;
        let full_h = (h as f32 - 2.0).max(1.0);
        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let cx = (w as f32 * (0.18 + 0.32 * i as f32)) as i32;
            let height = (full_h * (local / 0.55).min(1.0)).max(1.0) as i32;
            let top = ground - height;
            // Trunk: 4 dots wide with a rounded crown and a rib groove.
            for y in top..ground {
                let crown = y == top;
                for dx in -2..=1 {
                    if crown && (dx == -2 || dx == 1) {
                        continue;
                    }
                    if dx == 0 && y > top + 1 && (y + i as i32) % 3 == 0 {
                        continue;
                    }
                    paint(grid, cx + dx, y, green);
                }
            }
            // Arms: out sideways, then up — left at 60%, right at 80%.
            let arms = [(0.6, -1, 0.55), (0.8, 1, 0.4)];
            for (k, &(at, side, frac)) in arms.iter().enumerate() {
                if local < at {
                    continue;
                }
                let grow = ((local - at) / 0.15).min(1.0);
                let elbow_y = ground - (full_h * frac) as i32;
                let reach = 3;
                let x_out = if side < 0 {
                    cx - 2 - reach
                } else {
                    cx + 1 + reach
                };
                for s in 1..=reach {
                    let x = if side < 0 { cx - 2 - s } else { cx + 1 + s };
                    paint(grid, x, elbow_y, green);
                    paint(grid, x, elbow_y - 1, green);
                }
                let up = ((3.0 + k as f32) * grow).round() as i32;
                for y in (elbow_y - 1 - up)..elbow_y {
                    paint(grid, x_out, y, green);
                    paint(grid, x_out - side, y, green);
                }
            }
            // Flower crown once fully grown.
            if local >= 0.98 {
                paint(grid, cx - 1, top - 1, pink);
                paint(grid, cx, top - 1, pink);
                paint(grid, cx - 2, top - 2, pink);
                paint(grid, cx + 1, top - 2, pink);
            }
        }

        Ok(())
    }
}

// ── 6. Sunflower Seeds — a field of sunflowers opening in turn ──────────────
//
// Each sunflower grows its stem, then opens: a ring of yellow petals spreads
// round a brown disc that fills with golden-angle (phyllotaxis) seeds.
// Flowers open left to right with progress; heads nod in the breeze.

struct SunflowerSeeds;
impl ProgressStyle for SunflowerSeeds {
    fn name(&self) -> &str {
        "sunflower-seeds"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A row of sunflowers opens left to right: yellow petals ring a brown disc packed with spiral seeds"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let stem = Color::rgb(70, 170, 60);
        let petal = Color::rgb(255, 205, 30);
        let seed = Color::rgb(130, 75, 30);
        let ground = h as i32 - 1;
        let n = (w / 15).max(1);
        let spacing = w as f32 / n as f32;
        let hy = h as f32 * 0.36;
        let r_out = (h as f32 * 0.34).min(spacing * 0.4).max(1.0);
        let r_disc = (r_out * 0.45).max(1.0);
        const GOLDEN: f32 = 2.399_963;

        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            let base_x = spacing * (i as f32 + 0.5);
            if local <= 0.0 {
                // Unsprouted: a seed in the soil.
                paint(grid, base_x as i32, ground, seed);
                continue;
            }
            let nod = (ctx.time * 1.3 + i as f32 * 1.1).sin() * 0.8;
            let hx = base_x + nod;
            let grow = (local * 2.0).min(1.0);
            let bloom = ((local - 0.5) * 2.0).clamp(0.0, 1.0);
            let top = ground as f32 - (ground as f32 - hy) * grow;
            for y in top as i32..=ground {
                paint(grid, base_x.round() as i32, y, stem);
            }
            // Two leaves on the stem.
            if grow > 0.6 {
                let ly = ground - 3;
                paint(grid, base_x as i32 - 1, ly, stem);
                paint(grid, base_x as i32 - 2, ly - 1, stem);
                paint(grid, base_x as i32 + 1, ly + 1, stem);
                paint(grid, base_x as i32 + 2, ly, stem);
            }
            if grow < 1.0 {
                continue;
            }
            // Petals: 12 rays that lengthen as the head opens, turning slowly.
            let len = r_disc + 0.5 + (r_out - r_disc) * (0.3 + 0.7 * bloom);
            for k in 0..12 {
                let a = k as f32 / 12.0 * 2.0 * PI + ctx.time * 0.2;
                for step in 0..=((len - r_disc) * 2.0) as i32 {
                    let d = r_disc + step as f32 * 0.5;
                    paint(
                        grid,
                        (hx + a.cos() * d * 1.25).round() as i32,
                        (hy + a.sin() * d).round() as i32,
                        petal,
                    );
                }
            }
            // Seed disc: phyllotaxis spiral fills in as the head matures.
            let seeds = 6 + (bloom * 40.0) as i32;
            for k in 0..seeds {
                let rr = r_disc * (k as f32 / 46.0).sqrt();
                let a = k as f32 * GOLDEN;
                paint(
                    grid,
                    (hx + a.cos() * rr * 1.25).round() as i32,
                    (hy + a.sin() * rr).round() as i32,
                    seed,
                );
            }
        }

        Ok(())
    }
}

// ── 7. Mushroom Cap — toadstools pop up one after another ────────────────────
//
// A row of fly-agaric toadstools: each pushes up a cream stalk, then its red
// domed cap swells, punched with white spots (negative space). Toadstools
// appear left to right with progress; the newest one breathes and puffs spores.

struct MushroomCap;
impl ProgressStyle for MushroomCap {
    fn name(&self) -> &str {
        "mushroom-cap"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Red spotted toadstools pop up left to right: a cream stalk rises, then the domed cap swells and puffs spores"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cap_c = Color::rgb(225, 45, 45);
        let stalk_c = Color::rgb(240, 225, 195);
        let spore_c = Color::rgb(200, 190, 160);
        let ground = h as i32 - 1;
        let n = (w / 18).max(1);
        let spacing = w as f32 / n as f32;
        let cap_base = (h as f32 * 0.7) as i32;

        for i in 0..n {
            let local = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if local <= 0.0 {
                continue;
            }
            let cx = (spacing * (i as f32 + 0.5)) as i32;
            // Stalk rises in the first 30%, cap swells for the rest.
            let rise = (local / 0.3).min(1.0);
            let swell = ((local - 0.2) / 0.8).clamp(0.0, 1.0);
            let breathe = if local < 1.0 || i + 1 == n {
                (ctx.time * 3.0 + i as f32).sin() * 0.3
            } else {
                0.0
            };
            let base_y = ground - ((ground - cap_base) as f32 * rise) as i32;
            for y in base_y..=ground {
                for dx in -1..=1 {
                    paint(grid, cx + dx, y, stalk_c);
                }
            }
            // Skirt ring under the cap.
            if swell > 0.3 {
                paint(grid, cx - 2, base_y + 1, stalk_c);
                paint(grid, cx + 2, base_y + 1, stalk_c);
            }

            let rx = (1.5 + (spacing * 0.36 - 1.5).max(0.0) * swell + breathe).max(1.0);
            let ry = (1.0 + (h as f32 * 0.5 - 1.0) * swell + breathe).max(1.0);
            let cyf = base_y as f32;
            // White spots, as fractions of the cap radii.
            let spots = [(-0.5, -0.45), (0.15, -0.75), (0.55, -0.35), (-0.1, -0.25)];
            for y in (cyf - ry).floor() as i32..=base_y {
                for x in (cx as f32 - rx).floor() as i32..=(cx as f32 + rx).ceil() as i32 {
                    let fx = (x - cx) as f32 / rx;
                    let fy = (y as f32 - cyf) / ry;
                    if fx * fx + fy * fy > 1.0 {
                        continue;
                    }
                    let spot = swell > 0.5
                        && spots.iter().any(|&(sx, sy)| {
                            ((fx - sx) * rx).abs() < 1.0 && ((fy - sy) * ry).abs() < 1.0
                        });
                    if !spot {
                        paint(grid, x, y, cap_c);
                    }
                }
            }

            // Spores drift up off the newest cap.
            if i + 1 == ((ctx.eased * n as f32).ceil() as usize).min(n) && swell > 0.5 {
                for k in 0..3 {
                    let t = (ctx.time * 0.8 + k as f32 / 3.0).fract();
                    let sx = cx + isin(ctx.time * 2.0 + k as f32 * 2.0, 3.0) + (k - 1) * 3;
                    let sy = (cyf - ry - 1.0 - t * 4.0) as i32;
                    paint(grid, sx, sy, spore_c);
                }
            }
        }

        Ok(())
    }
}

// ── 8. Succulent Rosette — fleshy leaves fan open from a potted rosette ─────
//
// Side view of an echeveria in a terracotta pot. Upright inner leaves appear
// first; with progress the rosette fans outward, each new leaf longer and
// lower, until the outermost leaves spread almost flat. Pink-blushed tips,
// and the whole rosette breathes gently with time.

struct Succulent;
impl ProgressStyle for Succulent {
    fn name(&self) -> &str {
        "succulent-rosette"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A potted echeveria fans open: plump sage-green leaves with pink tips spread outward from the centre"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let pot = Color::rgb(190, 95, 60);
        let leaf_c = Color::rgb(120, 190, 150);
        let tip_c = Color::rgb(235, 120, 160);
        let cx = w as f32 / 2.0;
        let ground = h as i32 - 1;
        let base_y = (h as f32 - 4.0).max(0.0);

        // Pot: rim plus a tapering body.
        let half = (w as f32 * 0.12).max(2.0) as i32;
        for y in (ground - 2)..=ground {
            let inset = (y - ground + 2) * 2;
            for x in (cx as i32 - half + inset)..=(cx as i32 + half - inset) {
                paint(grid, x, y, pot);
            }
        }

        // Leaves pair up symmetrically: ring 0 is upright, ring 4 nearly flat.
        let rings = 5;
        let shown = ctx.eased * rings as f32;
        let breathe = (ctx.time * 1.8).sin() * 0.3;
        for ring in (0..rings).rev() {
            let grow = (shown - ring as f32).clamp(0.0, 1.0);
            if grow <= 0.0 {
                continue;
            }
            let tilt = 0.15 + ring as f32 * 0.32; // radians from vertical
            let len = (h as f32 * (0.55 + ring as f32 * 0.12) * grow + breathe).max(1.0);
            for side in [-1.0f32, 1.0] {
                let a = tilt * side;
                let (dx, dy) = (a.sin() * 2.2, -a.cos());
                for step in 0..=(len * 2.0) as i32 {
                    let s = step as f32 * 0.5;
                    let f = s / len;
                    // Plump leaf: widest a third of the way out, pointed tip.
                    let half_w = (1.1 * (1.0 - (f - 0.3).abs() * 1.3)).max(0.25);
                    let (px, py) = (cx + dx * s, base_y + dy * s);
                    let c = if f > 0.85 { tip_c } else { leaf_c };
                    for kk in 0..=(half_w * 4.0) as i32 {
                        let k = -half_w + kk as f32 * 0.5;
                        // Offset across the leaf, perpendicular to its axis.
                        paint(
                            grid,
                            (px - dy * k * 1.3).round() as i32,
                            (py + a.sin() * k).round() as i32,
                            c,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

// ── 9. Root System — a cutaway of roots spreading through the soil ──────────
//
// A sprout stands on the soil surface at the left. Below ground, in a
// speckled cutaway, its taproot dives down and a main root snakes right with
// progress, forking rootlets up and down whose fine tips wriggle with time.

struct RootSystem;
impl ProgressStyle for RootSystem {
    fn name(&self) -> &str {
        "root-system"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "Soil cutaway under a sprout: cream roots snake right through speckled earth, forking rootlets with wriggling tips"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let soil = Color::rgb(120, 80, 45);
        let root = Color::rgb(235, 210, 160);
        let hh = h as i32;
        let surface = (hh / 4).max(1) - 1;
        let px = 6i32;

        // Soil: surface crust plus scattered speckles and pebbles.
        for x in 0..w as i32 {
            paint(grid, x, surface + 1, soil);
            for y in surface + 2..hh {
                if (x.wrapping_mul(7919) ^ y.wrapping_mul(104_729)).rem_euclid(23) == 0 {
                    paint(grid, x, y, soil);
                }
            }
        }

        // The sprout above ground: a stem with two leaves that sway.
        let sway = isin(ctx.time * 1.5, 1.0);
        for y in 0..=surface {
            paint(grid, px, y, leafy(0.6));
        }
        paint(grid, px - 1 + sway, 0, leafy(0.8));
        paint(grid, px - 2 + sway, 0, leafy(0.8));
        paint(grid, px + 1 + sway, 0, leafy(0.8));
        paint(grid, px + 2 + sway, 0, leafy(0.8));

        // Taproot down, then the main root snaking right out to `reach`.
        let depth = surface + 6;
        let reach = px + (ctx.eased * (w as f32 - px as f32 - 1.0)) as i32;
        for y in surface + 2..=depth {
            paint(grid, px, y, root);
            paint(grid, px + 1, y, root);
        }
        let main_y = |x: i32| depth + ((x as f32 * 0.18).sin() * 1.2) as i32;
        for x in px..=reach {
            paint(grid, x, main_y(x), root);
            // Thicker near the plant, thinning towards the tip.
            if x < px + (reach - px) / 2 {
                paint(grid, x, main_y(x) + 1, root);
            }
        }

        // Rootlets: fork downward every few dots, alternating steep and shallow.
        let mut k = 0;
        let mut x = px + 5;
        while x < reach - 1 {
            let steep = k % 2 == 0;
            let len = ((reach - x) / 3).clamp(1, 6);
            let y0 = main_y(x);
            let (mut rx, mut ry) = (x, y0);
            for s in 1..=len {
                rx = x + s;
                ry = if steep { y0 + s } else { y0 + (s + 1) / 2 };
                if ry >= hh {
                    break;
                }
                paint(grid, rx, ry, root);
            }
            // Fine root hair at the tip, wriggling.
            let wig = isin(ctx.time * 4.0 + x as f32, 1.0);
            paint(grid, rx + 1, ry + wig, root);
            k += 1;
            x += 6;
        }

        Ok(())
    }
}

// ── 10. Bonsai — potted tree: trunk, branches, cloud-pad canopy ──────────────
//
// A shallow terracotta pot holds a twisting trunk. Progress grows the trunk
// first, then reaches out the branches, then swells the flat cloud-like
// foliage pads at each branch tip. Pads drift slightly in the breeze.

struct Bonsai;
impl ProgressStyle for Bonsai {
    fn name(&self) -> &str {
        "bonsai"
    }
    fn theme(&self) -> &str {
        "plants"
    }
    fn describe(&self) -> &str {
        "A bonsai in a terracotta pot: the twisting trunk grows, branches reach out and cloud-pad foliage swells"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let pot = Color::rgb(190, 90, 55);
        let bark = Color::rgb(140, 95, 55);
        let leaf = Color::rgb(70, 175, 75);
        let leaf_hi = Color::rgb(140, 215, 90);
        let p = ctx.eased;
        let cx = (w / 2) as i32;
        let hh = h as i32;
        let ground = hh - 1;

        // Pot: a wide rim over a tapering body.
        let pot_top = ground - 2;
        let half = (w as i32 / 7).max(2);
        for y in pot_top..=ground {
            let inset = (y - pot_top) * 2;
            for x in (cx - half - 2 + inset)..=(cx + half + 2 - inset) {
                if y != pot_top || (x - cx).abs() <= half + 2 {
                    paint(grid, x, y, pot);
                }
            }
        }

        // Trunk: an S-curve from the soil up to the crown, 2 dots thick.
        let trunk_h = (hh - 7).max(1);
        let grown = ((p / 0.35).min(1.0) * trunk_h as f32) as i32;
        let trunk_x = |y: i32| -> i32 {
            let f = (pot_top - y) as f32 / trunk_h as f32;
            cx + ((f * PI * 1.4).sin() * (hh as f32 * 0.22)) as i32
        };
        for y in (pot_top - grown)..pot_top {
            let x = trunk_x(y);
            let thick = if y > pot_top - trunk_h / 2 { 3 } else { 2 };
            for k in 0..thick {
                paint(grid, x + k - 1, y, bark);
            }
        }
        if p < 0.3 {
            return Ok(());
        }

        // Branches and their pads: (height fraction, reach in dots, pad size).
        let unit = (w as f32 / 12.0).max(1.0);
        let branches = [
            (0.45, -2.6, 1.0),
            (0.62, 2.4, 0.9),
            (0.85, -1.2, 0.8),
            (1.0, 1.0, 1.1),
        ];
        let reach = ((p - 0.3) / 0.25).clamp(0.0, 1.0);
        let swell = ((p - 0.45) / 0.55).clamp(0.0, 1.0);
        for (k, &(fh, dir, size)) in branches.iter().enumerate() {
            let by = pot_top - (trunk_h as f32 * fh) as i32;
            let bx = trunk_x(by);
            let ex = bx + (dir * unit * reach) as i32;
            let ey = by - 1;
            let steps = (ex - bx).abs().max(1);
            for s in 0..=steps {
                let x = bx + (ex - bx) * s / steps;
                let y = by - s / 4;
                paint(grid, x, y, bark);
            }
            if swell <= 0.0 {
                continue;
            }
            // Cloud pad: a flat ellipse, wider than tall, drifting gently.
            let drift = (ctx.time * 1.2 + k as f32).sin() * 0.6;
            let rx = (unit * 1.3 * size * swell).max(1.0);
            let ry = (hh as f32 * 0.15 * size * swell).max(0.8);
            let pcx = ex as f32 + drift;
            let pcy = ey as f32 - ry * 0.4;
            for y in (pcy - ry).floor() as i32..=(pcy + ry).ceil() as i32 {
                for x in (pcx - rx).floor() as i32..=(pcx + rx).ceil() as i32 {
                    let fx = (x as f32 - pcx) / rx;
                    let fy = (y as f32 - pcy) / ry;
                    if fx * fx + fy * fy <= 1.0 {
                        let c = if fy < -0.3 { leaf_hi } else { leaf };
                        paint(grid, x, y, c);
                    }
                }
            }
        }

        Ok(())
    }
}
