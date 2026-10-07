//! Surf / surfing-culture progress bars.
//!
//! Twelve animated braille bars built around the surfing world: barrels,
//! swell lines marching from the horizon, surfer carves, wipeouts, aerials,
//! cutbacks, noserides, offshore spray, wave sets, tide lines advancing up
//! the beach, a palm tree at sunset and a surf van on the coast road. Every bar is
//! structurally distinct — different geometry, layout, and animation logic,
//! not merely a recolor of another.
//!
//! `ctx.eased` = how far along the ride / how much of the wave is done.
//! `ctx.time`  = continuous wave-cycle driver (sine + scroll).

use super::super::draw;
use super::super::{lerp, BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `surf` theme.
///
/// Returns twelve boxed surf-themed bars ready to be mixed into any registry
/// or rendered directly via [`ProgressStyle::render`].
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(BarrelTube),
        Box::new(SwellLines),
        Box::new(SurferCarve),
        Box::new(Wipeout),
        Box::new(Cutback),
        Box::new(OffshoreSpray),
        Box::new(NoseRide),
        Box::new(AerialLaunch),
        Box::new(TideLine),
        Box::new(PalmSunset),
        Box::new(WaveSets),
        Box::new(SurfVan),
    ]
}

// ---------------------------------------------------------------------------
// 1. Barrel / Tube
// ---------------------------------------------------------------------------
// Side view of a hollow wave: a thick lip pitches over from the back wall and
// curls down toward the water, leaving an open tube with a crouched surfer
// inside. `eased` slides the barrel to the right; behind it the wave has
// already broken into churning whitewater. `time` churns the foam and wobbles
// the lip.

struct BarrelTube;
impl ProgressStyle for BarrelTube {
    fn name(&self) -> &str {
        "barrel-tube"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A hollow wave pitches its lip over a crouched surfer; the barrel rides right with progress, whitewater behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;

        // Barrel geometry: hollow ellipse (rx, ry) inside a lip ~3 dots thick.
        let rx = (h as f32 * 0.6).max(3.0);
        let ry = (h as f32 * 0.42).max(2.0);
        let travel = (w as f32 - 2.0 * rx - 12.0).max(0.0);
        let cx = 4.0 + rx + ctx.eased * travel;
        let cy = sea as f32 - ry * 0.55;
        let wobble = (ctx.time * 4.0).sin() * 0.4;
        let foam_end = (cx - rx - 3.0) as i32;

        for x in 0..wi {
            let fx = x as f32;
            let surf = sea as f32 + (fx * 0.25 - ctx.time * 3.0).sin() * 0.7;
            for y in 0..hi {
                let fy = y as f32;
                let dx = fx - cx;
                let dy = fy - cy;
                let inner = (dx / rx).powi(2) + (dy / ry).powi(2);
                let outer = (dx / (rx + 3.0)).powi(2) + (dy / (ry + 2.5 + wobble)).powi(2);
                // The lip: the ring between the ellipses, open at the lower
                // left where the tip hangs just above the water.
                let lip = outer <= 1.0 && inner > 1.0 && !(dx < 0.0 && dy > ry * 0.3);
                // The back of the wave slopes down to the flat sea on the right.
                let back = dx > 0.0 && inner > 1.0 && fy >= cy - ry - 2.0 + dx * 0.55;
                let sea_dot = fy >= surf;
                if (lip || back || sea_dot) && !(inner <= 1.0 && fy < sea as f32) {
                    draw::dot_i(grid, x, y);
                }
            }
            // Whitewater: churning foam stacked on the sea behind the barrel.
            if x < foam_end {
                let churn = ((x as f32 * 1.7 + ctx.time * 9.0).sin() + 1.0) * 1.5;
                let top = sea - 2 - churn as i32;
                for y in top..sea {
                    if (x + y + (ctx.time * 8.0) as i32) % 3 != 0 {
                        draw::dot_i(grid, x, y);
                    }
                }
            }
        }

        // Crouched surfer and board deep in the tube.
        let sx = cx as i32 - 3;
        stamp(grid, sx, sea - 8, &CROUCH, false, false);
        board(grid, sx - 2, sea - 1, 11, false);

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(
                    grid,
                    r,
                    c,
                    c,
                    ctx.palette
                        .sample(t * 0.6 + 0.4 * r as f32 / cells_h as f32),
                );
            }
        }
        paint(grid, 0, 0, foam_end, sea - 1, FOAM);
        paint(
            grid,
            (cx - rx - 3.0) as i32,
            0,
            (cx + rx) as i32,
            (cy - ry - 1.0) as i32,
            FOAM,
        );
        paint(grid, sx, sea - 8, sx + 7, sea - 1, SKIN);
        paint(grid, sx - 2, sea - 1, sx - 1, sea - 1, BOARD);
        paint(grid, sx + 8, sea - 2, sx + 9, sea - 1, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Swell Lines
// ---------------------------------------------------------------------------
// Looking out to sea: corduroy lines of swell roll in from a dotted horizon
// at the top toward the viewer, spreading apart and thickening as they get
// closer (perspective). `time` marches the lines in; `eased` = how far across
// the bar the swell has filled in — beyond it the sea is still glassy.

struct SwellLines;
impl ProgressStyle for SwellLines {
    fn name(&self) -> &str {
        "swell-lines"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "Corduroy swell lines roll in from the horizon in perspective, filling across the sea with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let edge = (ctx.eased * w as f32).round() as i32;

        // Horizon: a dotted line across the full width.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, 0);
        }

        // Lines at perspective depths: `d` in 0..1 maps to y via d², so far
        // lines bunch up near the horizon and near ones spread out.
        let n = 6;
        let march = (ctx.time * 0.35).fract();
        for k in 0..n {
            let d = (k as f32 + march) / n as f32;
            let y = 1 + (d * d * (hi - 2) as f32) as i32;
            let thick = 1 + (d * 1.5) as i32;
            for x in 0..edge {
                let bend = ((x as f32 * 0.08 + k as f32).sin() * d * 1.5).round() as i32;
                for t in 0..thick {
                    draw::dot_i(grid, x, y + bend + t);
                }
            }
        }
        // Glassy sea ahead of the swell: sparse glints.
        for x in (edge.max(0)..wi).step_by(5) {
            draw::dot_i(grid, x, hi / 2 + ((x * 7) % (hi / 2).max(1)));
        }

        let (cells_w, cells_h) = grid.dimensions();
        for r in 0..cells_h {
            let t = r as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                r,
                0,
                cells_w.saturating_sub(1),
                ctx.palette.sample(1.0 - t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Surfer Carving the Face
// ---------------------------------------------------------------------------
// A surfer in a wide stance, arms out, rides a board across the water. The
// board's carved S-line is cut into the water behind them as a hollow wake
// and a fan of spray kicks up off the tail. `eased` = how far along the ride;
// `time` bobs the rider and flings the spray.

struct SurferCarve;
impl ProgressStyle for SurferCarve {
    fn name(&self) -> &str {
        "surfer-carve"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A surfer on a board rides right with progress, carving an S-shaped wake and throwing spray off the tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let sx = 2 + (ctx.eased * (wi - 16).max(0) as f32) as i32;

        // Water, with the carve line left hollow behind the board.
        for x in 0..wi {
            let surf = sea as f32 + (x as f32 * 0.22 - ctx.time * 3.0).sin() * 0.8;
            let carve = sea + 1 + ((x as f32 * 0.3).sin() + 1.0).round() as i32;
            for y in (surf.round() as i32).max(0)..hi {
                if !(x < sx && y == carve) {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Rider: bobs on the swell.
        let bob = ((ctx.time * 2.5).sin() * 1.2).round() as i32;
        let by = sea - 1 + bob.min(0);
        stamp(grid, sx + 2, by - 10, &SURFER, false, false);
        board(grid, sx, by, 14, false);

        // Spray fan thrown up behind the tail.
        for k in 0..8 {
            let ph = (ctx.time * 1.8 + k as f32 * 0.125).fract();
            let px = sx - 1 - (ph * 10.0) as i32;
            let py = by - ((ph * PI).sin() * 6.0) as i32;
            draw::dot_i(grid, px, py);
        }

        let (cells_w, _) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            draw::tint_row(grid, (sea / 4) as usize, c, c, ctx.palette.sample(t));
            draw::tint_row(
                grid,
                (hi / 4 - 1).max(0) as usize,
                c,
                c,
                ctx.palette.sample(t),
            );
        }
        paint(grid, 0, 0, sx - 1, sea - 2, FOAM);
        paint(grid, sx + 2, by - 10, sx + 11, by, SKIN);
        paint(grid, sx, by, sx + 1, by, BOARD);
        paint(grid, sx + 12, by - 1, sx + 14, by, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Wipeout
// ---------------------------------------------------------------------------
// Whitewater rolls in from the left as progress; at its leading edge a
// surfer has gone over the falls — upside down, legs kicking out of the
// foam — while the board cartwheels through the air above. `time` spins the
// board, kicks the legs and churns the splash.

struct Wipeout;
impl ProgressStyle for Wipeout {
    fn name(&self) -> &str {
        "wipeout"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A surfer goes over the falls: legs up in the whitewater, board cartwheeling overhead, foam advancing with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let fx = 6 + (ctx.eased * (wi - 20).max(0) as f32) as i32;

        // Sea ahead, heaped churning whitewater behind the crash.
        for x in 0..wi {
            let surf = sea as f32 + (x as f32 * 0.22 - ctx.time * 3.0).sin() * 0.7;
            let top = if x <= fx + 3 {
                let churn = ((x as f32 * 1.3 + ctx.time * 7.0).sin() + 1.0) * 1.2;
                sea - 2 - churn as i32
            } else {
                surf.round() as i32
            };
            for y in top.max(0)..hi {
                if x > fx + 3 || y >= sea || (x + y + (ctx.time * 6.0) as i32) % 3 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Surfer upside down: head under, legs kicking out of the foam.
        let kick = (ctx.time * 6.0).sin() > 0.0;
        stamp(grid, fx - 4, sea - 7, &SURFER, kick, true);

        // Board cartwheeling overhead.
        let a = ctx.time * 5.0;
        let (bx, by) = (fx + 9, 3 + ((ctx.time * 2.0).sin() * 1.0) as i32);
        for i in -5..=5 {
            let px = bx + (a.cos() * i as f32) as i32;
            let py = by + (a.sin() * i as f32 * 0.5) as i32;
            draw::dot_i(grid, px, py);
        }

        // Splash droplets bursting from the impact.
        for k in 0..10 {
            let ph = (ctx.time * 1.5 + k as f32 * 0.1).fract();
            let dir = if k % 2 == 0 { 1.0 } else { -1.0 };
            let px = fx + (dir * ph * (4.0 + k as f32)) as i32;
            let py = sea - 2 - ((ph * PI).sin() * 7.0) as i32;
            draw::dot_i(grid, px, py);
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, 0, 0, fx + 3, sea - 1, FOAM);
        paint(grid, fx - 4, sea - 7, fx + 5, sea - 5, SKIN);
        paint(grid, bx - 5, by - 3, bx + 5, by + 3, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Cutback
// ---------------------------------------------------------------------------
// The surfer has ridden out onto the shoulder and whips the board back
// toward the breaking curl: rider facing back left, a carved white track
// hooking round behind them and a rooster tail of spray thrown high off the
// turn. `eased` = how far out the turn happens; `time` flings the spray.

struct Cutback;
impl ProgressStyle for Cutback {
    fn name(&self) -> &str {
        "cutback"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A surfer whips back toward the curl, throwing a rooster tail of spray; the turn moves right with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let turn = 14 + (ctx.eased * (wi - 30).max(0) as f32) as i32;

        water(grid, w, h, sea as f32, 0.7, ctx.time);

        // The curl breaking at the far left.
        for y in 0..sea {
            let k = (sea - y) as f32 / sea as f32;
            let lip = ((1.0 - k) * 4.0 + (ctx.time * 3.0).sin()) as i32;
            if y > 2 {
                draw::hline(grid, 0, lip.max(0) as usize, y as usize);
            }
        }
        for x in 0..6 {
            draw::dot_i(grid, x, 2 + (x - 3).abs() / 2);
        }

        // Carved track: out along the face, then hooking back round the turn.
        for x in 6..turn {
            if x % 2 == 0 {
                draw::dot_i(grid, x, sea - 1);
            }
        }
        for a in 0..10 {
            let th = a as f32 / 10.0 * PI;
            draw::dot_i(
                grid,
                turn + (th.sin() * 4.0) as i32,
                sea - 1 - (th.cos() * 2.0 + 2.0) as i32,
            );
        }

        // Rider facing back toward the curl.
        stamp(grid, turn - 9, sea - 11, &SURFER, true, false);
        board(grid, turn - 11, sea - 1, 13, true);

        // Rooster tail: spray arcing up and out off the turn.
        for k in 0..14 {
            let ph = (ctx.time * 1.4 + k as f32 / 14.0).fract();
            let px = turn + 2 + (ph * 12.0) as i32;
            let py = sea - 2 - ((ph * PI).sin() * (sea - 2) as f32) as i32;
            draw::dot_i(grid, px, py);
            if k % 2 == 0 {
                draw::dot_i(grid, px + 1, py + 1);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, 0, 0, 5, 3, FOAM);
        paint(grid, turn + 2, 0, turn + 15, sea - 1, FOAM);
        paint(grid, turn - 9, sea - 11, turn, sea - 1, SKIN);
        paint(grid, turn - 12, sea - 2, turn - 10, sea - 1, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Offshore Spray
// ---------------------------------------------------------------------------
// A steep wave rolls toward the beach (rightward) while an offshore wind
// tears a feathery plume of spray off its crest and blows it back over the
// top. Wind streaks race across the sky. `eased` carries the wave along;
// `time` streams the spray and the wind lines.

struct OffshoreSpray;
impl ProgressStyle for OffshoreSpray {
    fn name(&self) -> &str {
        "offshore-spray"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "Offshore wind feathers a plume of spray back off a steep wave's crest as it rolls right with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi - 3;
        let peak = 22 + (ctx.eased * (wi - 30).max(0) as f32) as i32;
        let height = (hi as f32 * 0.55) as i32;

        // Sea plus the wave: a long back slope and a steep front face.
        for x in 0..wi {
            let surf = sea as f32 + (x as f32 * 0.25 - ctx.time * 3.0).sin() * 0.6;
            let d = (x - peak) as f32;
            let rise = if d <= 0.0 {
                (1.0 + d / 20.0).max(0.0)
            } else {
                (1.0 - d / 4.0).max(0.0)
            };
            let top = (surf - rise * height as f32).round() as i32;
            draw::vline(grid, x as usize, top.max(0) as usize, h - 1);
        }

        // Spray plume: streams of droplets blown back up off the crest.
        let crest = sea - height;
        for s in 0..4 {
            for k in 0..9 {
                let ph = (ctx.time * 1.6 + k as f32 / 9.0 + s as f32 * 0.29).fract();
                let px = peak - 1 - (ph * (14.0 + s as f32 * 3.0)) as i32;
                let py = crest - 1 - (ph.sqrt() * (crest - s).max(1) as f32) as i32 + s;
                draw::dot_i(grid, px, py);
            }
        }

        // Wind streaks across the sky ahead of the wave.
        for k in 0..3 {
            let len = 5;
            let x0 = wi - ((ctx.time * 30.0 + k as f32 * 29.0) as i32).rem_euclid(wi + len);
            let y = 1 + k * 2;
            for i in 0..len {
                if x0 + i > peak + 3 {
                    draw::dot_i(grid, x0 + i, y);
                }
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, peak - 20, 0, peak + 1, crest - 1, FOAM);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Longboard Noseride
// ---------------------------------------------------------------------------
// A long, flat longboard trims across the water with the rider standing
// tall on the very nose, toes over the tip (hanging ten). `eased` carries the
// board along; `time` rocks the board and streams a wake from the tail.

struct NoseRide;
impl ProgressStyle for NoseRide {
    fn name(&self) -> &str {
        "noseride"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A longboarder hangs ten on the nose of a long board, trimming right with progress and leaving a wake"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi * 3 / 4;
        let len = (wi / 3).clamp(10, 30);
        let bx = (ctx.eased * (wi - len - 2).max(0) as f32) as i32;
        let rock = ((ctx.time * 2.0).sin() * 0.8).round() as i32;
        let by = sea - 1 + rock.min(0);

        water(grid, w, h, sea as f32, 0.7, ctx.time);

        // Long board with a nose rocker.
        board(grid, bx, by, len, false);
        draw::dot_i(grid, bx, by - 1);

        // Rider upright on the nose, toes on the tip.
        stamp(grid, bx + len - 5, by - 10, &NOSE_RIDER, false, false);

        // Wake: a dashed foam line streaming off the tail.
        for x in 0..bx {
            if (x + (ctx.time * 10.0) as i32) % 4 != 0 {
                draw::dot_i(grid, x, sea - 1);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, 0, sea - 1, bx - 1, sea - 1, FOAM);
        paint(grid, bx + len - 5, by - 10, bx + len, by - 1, SKIN);
        paint(grid, bx, by, bx + len - 6, by, BOARD);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Aerial Launch
// ---------------------------------------------------------------------------
// A crouched surfer hits the lip of a wave and launches into the air on a
// looping hop — board tilted nose-up on the way up, nose-down on the way
// down — with spray peeling off the lip. `eased` moves the launch point
// along the bar; `time` drives the hop cycle.

struct AerialLaunch;
impl ProgressStyle for AerialLaunch {
    fn name(&self) -> &str {
        "aerial-launch"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A crouched surfer boosts off a wave lip into the air and lands, launching further along with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi - 3;
        let lx = 6 + (ctx.eased * (wi - 30).max(0) as f32) as i32;

        // Sea with a peaking lip to launch from.
        for x in 0..wi {
            let d = (x - lx) as f32 / 4.0;
            let peak = 5.0 * (-d * d).exp();
            let surf = sea as f32 - peak + (x as f32 * 0.25 - ctx.time * 3.0).sin() * 0.6;
            draw::vline(grid, x as usize, (surf.round().max(0.0)) as usize, h - 1);
        }

        // Hop cycle: airborne for the first 70% of each loop.
        let ph = (ctx.time * 0.6).fract();
        let air = if ph < 0.7 { (ph / 0.7 * PI).sin() } else { 0.0 };
        let lift = (air * (sea - 9).max(0) as f32) as i32;
        let sx = lx + 3 + (ph.min(0.7) * 10.0) as i32;
        let by = sea - 1 - lift;
        // Nose up on the way up, nose down on the way down, flat on landing.
        let tilt = match ph {
            p if p < 0.35 => -1,
            p if p < 0.7 => 1,
            _ => 0,
        };
        stamp(grid, sx + 1, by - 8, &CROUCH, false, false);
        for i in 0..11 {
            draw::dot_i(grid, sx + i - 1, by + tilt * (i - 5) / 4);
        }

        // Spray peeling off the lip behind the launch.
        for k in 0..8 {
            let p = (ctx.time * 2.0 + k as f32 * 0.125).fract();
            draw::dot_i(
                grid,
                lx - (p * 8.0) as i32,
                sea - 5 - ((p * PI).sin() * 3.0) as i32,
            );
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }
        paint(grid, lx - 8, 0, lx, sea - 3, FOAM);
        paint(grid, sx + 1, by - 8, sx + 8, by - 2, SKIN);
        if by / 4 < sea / 4 {
            paint(grid, sx - 1, by, sx + 10, by, BOARD);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Tide Line
// ---------------------------------------------------------------------------
// A sandy beach slopes up to the right. The sea climbs it as the tide comes
// in with progress — the waterline creeps up the sand — while a lacy line of
// swash foam runs up and slides back with `time`. A wet-sand band marks the
// last high-water line.

struct TideLine;
impl ProgressStyle for TideLine {
    fn name(&self) -> &str {
        "tide-line"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "The tide floods up a sloping sand beach with progress while lacy swash foam runs up and slides back"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        // Beach surface rises from the bottom-left to mid-height on the right.
        let sand_top = |x: i32| hi - 2 - (x as f32 / wi as f32 * (hi as f32 * 0.6)) as i32;
        // Waterline: where the sea surface meets the sand, moving right.
        let shore = 4 + (ctx.eased * (wi as f32 * 0.72)) as i32;
        let level = sand_top(shore);
        let swash = ((ctx.time * 1.6).sin() * 0.5 + 0.5) * 6.0;
        let reach = shore + swash as i32;

        for x in 0..wi {
            let st = sand_top(x);
            // Sand, speckled so it reads as grains.
            for y in st..hi {
                if x < reach || (x + y * 3) % 4 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
            // Sea over the sand up to the waterline, with a rolling surface.
            if x < shore {
                let s = level + ((x as f32 * 0.3 - ctx.time * 3.0).sin() * 0.7).round() as i32;
                for y in s.max(0)..st {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Lacy swash foam running up the slope.
        for x in shore..=reach {
            if (x + (ctx.time * 6.0) as i32) % 3 != 0 {
                draw::dot_i(grid, x, sand_top(x) - 1);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(r as f32 / cells_h as f32));
            }
        }
        for x in (shore + 8..wi).step_by(2) {
            paint(grid, x, sand_top(x), x, hi - 1, SAND);
        }
        paint(grid, shore, level - 2, reach + 1, sand_top(shore) + 1, FOAM);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Palm Tree and Sunset
// ---------------------------------------------------------------------------
// A curved palm trunk rises from a little sand spit on the left, its crown
// of drooping fronds swaying in the breeze. Out on the horizon a big striped
// sun sinks into the sea as progress grows, laying a shimmering reflection
// on the water beneath it.

struct PalmSunset;
impl ProgressStyle for PalmSunset {
    fn name(&self) -> &str {
        "palm-sunset"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A swaying palm tree on a sand spit while a striped sun sinks into the sea with progress, reflection shimmering"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let horizon = hi * 3 / 4;

        // Sun: a striped disc sinking through the horizon.
        let r = (h as f32 * 0.38).max(2.0);
        let sun_x = w as f32 * 0.62;
        let sun_y = lerp(r * 0.9, horizon as f32 + r, ctx.eased);
        for y in 0..horizon {
            let dy = y as f32 - sun_y;
            if dy.abs() > r {
                continue;
            }
            // Retro bands cut through the lower half of the disc.
            if dy > 0.0 && (y + 1) % 3 == 0 {
                continue;
            }
            let half = (r * r - dy * dy).max(0.0).sqrt() * 1.5;
            draw::hline(
                grid,
                (sun_x - half).max(0.0) as usize,
                (sun_x + half).max(0.0) as usize,
                y as usize,
            );
        }

        // Sea: horizon line plus shimmering reflection strokes under the sun.
        draw::hline(grid, 0, w - 1, horizon as usize);
        let glow = (1.0 - ctx.eased * 0.7) * r * 1.4;
        for y in (horizon + 2..hi).step_by(2) {
            let jitter = ((ctx.time * 3.0 + y as f32).sin() * 3.0) as i32;
            let half = (glow * (1.0 - (y - horizon) as f32 / (hi - horizon) as f32 * 0.5)) as i32;
            for x in (sun_x as i32 - half + jitter)..=(sun_x as i32 + half + jitter) {
                if (x + y) % 5 != 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Sand spit at the palm's foot.
        let tx = (wi / 8).max(3);
        for x in 0..tx * 2 + 4 {
            let mound = (2.0 * (1.0 - ((x - tx) as f32 / (tx + 3) as f32).powi(2))).max(0.0);
            draw::vline(
                grid,
                x as usize,
                (horizon - mound as i32).max(0) as usize,
                horizon as usize,
            );
        }

        // Curved trunk, two dots thick, leaning right as it rises.
        let crown_y = 3;
        let sway = (ctx.time * 1.2).sin();
        let mut crown_x = tx;
        for y in crown_y..=horizon {
            let k = (horizon - y) as f32 / (horizon - crown_y).max(1) as f32;
            let x = tx + (k * k * (4.0 + sway)) as i32;
            draw::dot_i(grid, x, y);
            draw::dot_i(grid, x + 1, y);
            if y == crown_y {
                crown_x = x;
            }
        }

        // Fronds: long arcs that rise off the crown and droop at the tips,
        // with leaflets hanging beneath like a comb.
        for (i, &dir) in [-1.0f32, -0.6, 0.6, 1.0].iter().enumerate() {
            let len = if dir.abs() > 0.9 { 12 } else { 9 };
            let flap = (ctx.time * 2.0 + i as f32).sin() * 0.8;
            for s in 0..len {
                let f = s as f32 / len as f32;
                let x = crown_x as f32 + dir * s as f32 * 1.2 + sway;
                let y = crown_y as f32 - 2.5 * (f * PI * 0.8).sin() + f * f * (6.0 + flap);
                draw::dot_i(grid, x as i32, y as i32);
                if s > 1 && s % 2 == 0 {
                    draw::dot_i(grid, x as i32, y as i32 + 1);
                    draw::dot_i(grid, x as i32, y as i32 + 2);
                }
            }
        }
        // Coconuts.
        draw::dot_i(grid, crown_x, crown_y + 2);
        draw::dot_i(grid, crown_x + 2, crown_y + 2);

        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(r as f32 / cells_h as f32));
            }
        }
        paint(grid, 0, 0, wi - 1, horizon - 1, SKY);
        paint(
            grid,
            (sun_x - r * 1.5) as i32,
            (sun_y - r) as i32,
            (sun_x + r * 1.5) as i32,
            horizon - 1,
            SUN,
        );
        paint(
            grid,
            (sun_x - glow) as i32 - 3,
            horizon + 1,
            (sun_x + glow) as i32 + 3,
            hi - 1,
            BOARD,
        );
        paint(grid, 0, horizon - 2, tx * 2 + 4, horizon, SAND);
        paint(grid, tx, crown_y + 4, tx + 6, horizon - 3, TRUNK);
        paint(grid, crown_x - 15, 0, crown_x + 16, crown_y + 3, LEAF);
        paint(
            grid,
            crown_x - 15,
            crown_y + 4,
            crown_x - 4,
            crown_y + 7,
            LEAF,
        );
        paint(
            grid,
            crown_x + 5,
            crown_y + 4,
            crown_x + 16,
            crown_y + 7,
            LEAF,
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Wave Sets
// ---------------------------------------------------------------------------
// Surf arrives in "sets": groups of three breaking waves, biggest first.
// Each wave is a steep-faced hump whose white lip curls over the front.
// `eased` = how many sets have rolled in (left to right); `time` pitches the
// lips and churns the foam at their feet.

struct WaveSets;
impl ProgressStyle for WaveSets {
    fn name(&self) -> &str {
        "wave-sets"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "Sets of three curling breakers, biggest first, roll in across the bar as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let sea = hi - 2;
        // Flat sea between sets.
        for x in 0..wi {
            draw::vline(grid, x as usize, sea as usize, h - 1);
        }
        let (cells_w, cells_h) = grid.dimensions();
        for c in 0..cells_w {
            let t = c as f32 / cells_w.max(1) as f32;
            for r in 0..cells_h {
                draw::tint_row(grid, r, c, c, ctx.palette.sample(t));
            }
        }

        let set_w = 30;
        let sets = (wi / set_w).max(1);
        let arrived = ctx.eased * sets as f32;
        for s in 0..sets {
            let g = (arrived - s as f32).clamp(0.0, 1.0);
            for k in 0..3 {
                // Each wave rises in turn as its set arrives.
                let rise = (g * 3.0 - k as f32).clamp(0.0, 1.0);
                if rise <= 0.0 {
                    continue;
                }
                let size = [1.0f32, 0.8, 0.6][k as usize];
                let hgt = ((hi - 4) as f32 * size * rise) as i32;
                let wid = (10.0 * size) as i32 + 2;
                let x0 = s * set_w + k * 10 + 1;
                let pitch = ((ctx.time * 3.0 + k as f32 + s as f32).sin() * 1.0).round() as i32;
                // Hump with a steep front (left) and a long back slope.
                for dx in 0..wid {
                    let u = dx as f32 / wid as f32;
                    let f = if u < 0.3 {
                        u / 0.3
                    } else {
                        1.0 - (u - 0.3) / 0.7
                    };
                    let top = sea - (f * hgt as f32) as i32;
                    for y in top..sea {
                        draw::dot_i(grid, x0 + dx, y);
                    }
                }
                // Curling lip hooking over the front.
                let cx = x0 + (wid as f32 * 0.3) as i32;
                let ct = sea - hgt;
                for (lx, ly) in [(0, -1), (-1, -1), (-2, 0), (-3, 1), (-3, 2)] {
                    draw::dot_i(grid, cx + lx + pitch.min(0), ct + ly);
                }
                // Foam at the foot.
                for dx in -3..2 {
                    if (dx + (ctx.time * 8.0) as i32) % 2 == 0 {
                        draw::dot_i(grid, x0 + dx, sea - 1);
                    }
                }
                if hgt > 2 {
                    paint(grid, cx - 4, ct - 1, cx + 1, ct + 1, FOAM);
                }
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Surf Van
// ---------------------------------------------------------------------------
// The classic two-tone split-window surf bus drives down the coast road with
// a longboard strapped to its roof rack. Windows and wheel arches are cut as
// negative space; the wheels spin, the road dashes scroll by and exhaust
// puffs trail behind. `eased` = how far down the road the van has got.

struct SurfVan;
impl ProgressStyle for SurfVan {
    fn name(&self) -> &str {
        "surf-van"
    }
    fn theme(&self) -> &str {
        "surf"
    }
    fn describe(&self) -> &str {
        "A two-tone surf bus with a longboard on the roof rack drives right with progress, wheels spinning"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 16 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let wi = w as i32;
        let base = h as i32 - 16; // van art is 16 dots tall, sat on the bottom
        let len = 30;
        let bx = 4 + (ctx.eased * (wi - len - 6).max(0) as f32) as i32;
        let wheels = [bx + 6, bx + 23];
        let wy = base + 12;

        // Road dashes scrolling past.
        for x in 0..wi {
            if (x + (ctx.time * 24.0) as i32).rem_euclid(8) < 4 {
                draw::dot_i(grid, x, base + 15);
            }
        }

        // Longboard on the roof rack.
        draw::hline(
            grid,
            (bx + 3).max(0) as usize,
            (bx + 26).max(0) as usize,
            base.max(0) as usize,
        );
        draw::hline(
            grid,
            (bx + 1).max(0) as usize,
            (bx + 28).max(0) as usize,
            (base + 1) as usize,
        );
        for rx in [bx + 6, bx + 22] {
            draw::vline(
                grid,
                rx.max(0) as usize,
                (base + 2) as usize,
                (base + 3) as usize,
            );
        }

        // Body with rounded corners, a sloped nose, windows and wheel arches.
        for x in 0..len {
            for y in 4..=12 {
                let corner = (x == 0 || x == len - 1) && (y == 4 || y == 12);
                let nose = x >= len - 3 && y < 4 + (x - (len - 4)) * 2;
                let window = (5..=7).contains(&y)
                    && ((2..=7).contains(&x)
                        || (10..=15).contains(&x)
                        || (18..=22).contains(&x)
                        || (25..=27).contains(&x));
                let arch = wheels.iter().any(|&c| {
                    let (dx, dy) = ((bx + x - c) as f32, (y - 12) as f32);
                    dx * dx + dy * dy < 12.0
                });
                let belt = y == 8 && x % 2 == 0;
                if !(corner || nose || window || arch || belt) {
                    draw::dot_i(grid, bx + x, base + y);
                }
            }
        }

        // Wheels: rims with a spinning spoke.
        let spin = ctx.time * 9.0;
        for &c in &wheels {
            for s in 0..12 {
                let a = s as f32 / 12.0 * 2.0 * PI;
                draw::dot_i(
                    grid,
                    c + (a.cos() * 2.4).round() as i32,
                    wy + (a.sin() * 2.4).round() as i32,
                );
            }
            draw::dot_i(grid, c, wy);
            draw::dot_i(
                grid,
                c + spin.cos().round() as i32,
                wy + spin.sin().round() as i32,
            );
        }

        // Exhaust puffs drifting back.
        for k in 0..3 {
            let ph = (ctx.time * 1.2 + k as f32 / 3.0).fract();
            draw::dot_i(
                grid,
                bx - 1 - (ph * 10.0) as i32,
                base + 11 - (ph * 3.0) as i32,
            );
        }

        let (cells_w, cells_h) = grid.dimensions();
        for r in 0..cells_h {
            draw::tint_row(
                grid,
                r,
                0,
                cells_w.saturating_sub(1),
                ctx.palette.sample(r as f32 / cells_h as f32),
            );
        }
        paint(grid, bx, base, bx + len, base + 3, BOARD);
        paint(grid, bx, base + 4, bx + len, base + 7, FOAM);
        paint(grid, bx, base + 8, bx + len, base + 11, VAN);
        paint(grid, bx, base + 12, bx + len, base + 15, TYRE);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shared sprites and helpers
// ---------------------------------------------------------------------------

/// Cell colours for the pictorial bars.
const SKIN: Color = Color::rgb(250, 228, 196);
const BOARD: Color = Color::rgb(255, 168, 60);
const FOAM: Color = Color::rgb(225, 240, 255);
const VAN: Color = Color::rgb(60, 200, 190);
const TYRE: Color = Color::rgb(150, 150, 165);
const SUN: Color = Color::rgb(255, 120, 60);
const SKY: Color = Color::rgb(150, 90, 200);
const SAND: Color = Color::rgb(230, 200, 130);
const TRUNK: Color = Color::rgb(170, 110, 60);
const LEAF: Color = Color::rgb(80, 200, 90);

/// Surfer in a wide stance, arms out, facing right (10×10 dots).
const SURFER: [&str; 10] = [
    "....##....",
    "....##....",
    "##.####.##",
    "..#.##.#..",
    "....##....",
    "....##....",
    "...####...",
    "..##..##..",
    ".##....##.",
    ".#......#.",
];

/// Surfer crouched low, arms forward (8×7 dots) — for tubes and airs.
const CROUCH: [&str; 7] = [
    "....##..", "....##..", "#..####.", ".#.##..#", "..####..", ".##..##.", ".#....#.",
];

/// Longboarder standing tall and still, arms loose (6×10 dots).
const NOSE_RIDER: [&str; 10] = [
    "..##..", "..##..", ".####.", "#.##.#", "#.##.#", "..##..", "..##..", ".#..#.", ".#..#.",
    ".#..#.",
];

/// Stamp an ASCII sprite (`#` = dot) with its top-left at `(x, y)`.
/// `mirror` flips it left↔right; `upside` flips it top↔bottom.
fn stamp(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str], mirror: bool, upside: bool) {
    let n = rows.len() as i32;
    for (ry, row) in rows.iter().enumerate() {
        let len = row.len() as i32;
        for (rx, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                let dx = if mirror {
                    len - 1 - rx as i32
                } else {
                    rx as i32
                };
                let dy = if upside { n - 1 - ry as i32 } else { ry as i32 };
                draw::dot_i(grid, x + dx, y + dy);
            }
        }
    }
}

/// Surfboard `len` dots long with its tail at `(x, y)`; the nose kicks up.
fn board(grid: &mut BrailleGrid, x: i32, y: i32, len: i32, mirror: bool) {
    let len = len.max(2);
    for i in 0..len {
        draw::dot_i(grid, x + i, y);
    }
    let nose = if mirror { x - 1 } else { x + len };
    draw::dot_i(grid, nose, y - 1);
}

/// Fill water below a sine surface centred on row `base`.
fn water(grid: &mut BrailleGrid, w: usize, h: usize, base: f32, amp: f32, time: f32) {
    for x in 0..w {
        let s = base + (x as f32 * 0.22 - time * 3.0).sin() * amp;
        let top = (s.round().max(0.0) as usize).min(h.saturating_sub(1));
        draw::vline(grid, x, top, h.saturating_sub(1));
    }
}

/// Colour every cell overlapping the dot rectangle `[x0, x1] × [y0, y1]`.
fn paint(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    grid.enable_color_support();
    let (cw, ch) = grid.dimensions();
    if cw == 0 || ch == 0 || x1 < 0 || y1 < 0 || x1 < x0 || y1 < y0 {
        return;
    }
    let (cx0, cy0) = ((x0.max(0) / 2) as usize, (y0.max(0) / 4) as usize);
    let (cx1, cy1) = (
        ((x1 / 2) as usize).min(cw - 1),
        ((y1 / 4) as usize).min(ch - 1),
    );
    for cy in cy0..=cy1 {
        for cx in cx0..=cx1 {
            let _ = grid.set_cell_color(cx, cy, color);
        }
    }
}
