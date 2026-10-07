//! Transit / Transportation progress bars — the **transit** theme.
//!
//! Twelve transit scenes: trains, planes, boats, rides and mass-transit
//! systems. Vehicles are drawn side-on as recognisable silhouettes and move
//! through their scene as progress grows, while wheels, rotors, flames, steps
//! or waves animate with time. Cells already passed are tinted with the
//! palette gradient so progress also reads as a left-to-right sweep.
//!
//! # Style catalogue
//!
//! | name | mechanic |
//! |---|---|
//! | `steam-train`       | Locomotive (boiler, cab, stack, dome) + tender + coach roll right; rods drive the wheels, smoke trails back |
//! | `airplane-takeoff`  | Side-view airliner (swept wing, tail fin, windows, engine) taxis the runway, pitches up and climbs out with a contrail |
//! | `sailboat-tacking`  | Sloop (hull, mast, mainsail + jib) sails right over rolling waves; sails swing through each tack |
//! | `subway-map`        | Station dots along a route line; train marker advances between them |
//! | `hot-air-balloon`   | Teardrop envelope with gores, basket on ropes and a flickering burner drifts up and right |
//! | `ferris-wheel`      | Big spoked wheel on an A-frame turns, cabins hang level; festoon bulbs light up with progress |
//! | `cable-car`         | Gondola climbs a diagonal cable; support towers mark intervals |
//! | `escalator`         | Stepped truss with handrail between two floors; steps climb, a rider walks, rides up and steps off |
//! | `rocket-gantry`     | Propellant line fills the rocket by its gantry; arms swing clear, liftoff |
//! | `ferry-crossing`    | Ferry (hull, two window decks, funnel) crosses between piers, V wake and smoke streaming behind |
//! | `bicycle`           | Diamond-frame bicycle with saddle and handlebars rolls right; spokes and pedal cranks turn |
//! | `helicopter`        | Bubble cabin, tail boom, tail rotor and skids lift off a pad and climb right; main rotor spins |

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, DotmaxError};
use std::f32::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Bresenham line in dot-space, bounds-safe via `draw::dot_i`.
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let px = x0 + (x1 - x0) * i / steps;
        let py = y0 + (y1 - y0) * i / steps;
        draw::dot_i(grid, px, py);
    }
}

/// Draw a small filled circle (disk) in dot-space at (cx,cy) with radius r.
fn disk(grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32) {
    let r = r.max(0);
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r + r {
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }
    }
}

/// Draw a circle outline at (cx,cy) with radius r.
fn circle(grid: &mut BrailleGrid, cx: i32, cy: i32, r: i32) {
    let r = r.max(1);
    let steps = (r * 7).max(12) as usize;
    let mut prev: Option<(i32, i32)> = None;
    for s in 0..=steps {
        let angle = s as f32 / steps as f32 * 2.0 * PI;
        let px = cx + (r as f32 * angle.cos()).round() as i32;
        let py = cy + (r as f32 * angle.sin()).round() as i32;
        draw::dot_i(grid, px, py);
        if let Some((ppx, ppy)) = prev {
            if (px - ppx).abs() + (py - ppy).abs() > 2 {
                line(grid, ppx, ppy, px, py);
            }
        }
        prev = Some((px, py));
    }
}

/// Horizontal run in signed dot coordinates, clipped to the grid.
fn hl(grid: &mut BrailleGrid, x0: i32, x1: i32, y: i32) {
    let (dw, dh) = draw::dot_dims(grid);
    if y < 0 || y >= dh as i32 || dw == 0 {
        return;
    }
    let lo = x0.min(x1).max(0);
    let hi = x0.max(x1).min(dw as i32 - 1);
    for x in lo..=hi {
        draw::dot(grid, x as usize, y as usize);
    }
}

/// Vertical run in signed dot coordinates, clipped to the grid.
fn vl(grid: &mut BrailleGrid, x: i32, y0: i32, y1: i32) {
    let (dw, dh) = draw::dot_dims(grid);
    if x < 0 || x >= dw as i32 || dh == 0 {
        return;
    }
    let lo = y0.min(y1).max(0);
    let hi = y0.max(y1).min(dh as i32 - 1);
    for y in lo..=hi {
        draw::dot(grid, x as usize, y as usize);
    }
}

/// Stamp a dot-art sprite (`#` = dot) with its top-left corner at `(x, y)`.
///
/// `slope` shears the sprite vertically around column `pivot` (positive lifts
/// the right-hand side), which reads as a pitch rotation at braille scale.
fn stamp(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str], pivot: i32, slope: f32) {
    for (r, row) in rows.iter().enumerate() {
        for (c, b) in row.bytes().enumerate() {
            if b == b'#' {
                let c = c as i32;
                let lift = ((c - pivot) as f32 * slope).round() as i32;
                draw::dot_i(grid, x + c, y + r as i32 - lift);
            }
        }
    }
}

/// Map `t` in `0..=1` onto the integer range `a..=b`.
fn lerp_i(a: i32, b: i32, t: f32) -> i32 {
    a + ((b - a) as f32 * t.clamp(0.0, 1.0)).round() as i32
}

/// Tint every cell column left of the progress point with the palette
/// gradient; cells ahead keep the default colour. Shared by every style here.
fn tint_progress(grid: &mut BrailleGrid, ctx: &BarContext) {
    let (cw, ch) = grid.dimensions();
    let filled = ((ctx.eased.clamp(0.0, 1.0) * cw as f32) as usize).min(cw);
    for cx in 0..filled {
        let c = ctx.palette.sample(cx as f32 / cw.max(1) as f32);
        for cy in 0..ch {
            draw::tint_row(grid, cy, cx, cx, c);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// All styles in the `transit` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per transportation bar.  Every style is
/// geometrically distinct — they differ in spatial layout and temporal mechanic,
/// never merely in color.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(SteamTrain),
        Box::new(AirplaneTakeoff),
        Box::new(SailboatTacking),
        Box::new(SubwayMap),
        Box::new(HotAirBalloon),
        Box::new(FerrisWheel),
        Box::new(CableCar),
        Box::new(Escalator),
        Box::new(RocketGantry),
        Box::new(FerryCrossing),
        Box::new(Bicycle),
        Box::new(Helicopter),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Steam train
//    Side-view locomotive (cab, boiler, steam dome, flared smokestack,
//    cowcatcher) pulling a coal tender and a coach along a rail.  Progress =
//    how far the head of the train has rolled.  The connecting rod circles with
//    time and smoke puffs stream back from the stack.
// ─────────────────────────────────────────────────────────────────────────────

/// Locomotive body, facing right (wheels are drawn procedurally below row 7).
const LOCO: [&str; 11] = [
    ".....................####",
    "########..............##",
    ".##..##.......##......##",
    ".##..##......####.....##",
    ".#########################",
    ".##########################",
    ".#.#.#.####################",
    "############################",
    "........................####",
    ".........................####",
    "..........................####",
];
/// Coal tender (11 wide) — lumpy coal heap over a box.
const TENDER: [&str; 5] = [
    "..##.###.#.",
    ".#########.",
    "###########",
    "###########",
    "###########",
];
/// Passenger coach (18 wide) — roof, window band, frame.
const COACH: [&str; 6] = [
    ".################.",
    "##################",
    "#..#..#..#..#..#.#",
    "#..#..#..#..#..#.#",
    "##################",
    "##################",
];

struct SteamTrain;
impl ProgressStyle for SteamTrain {
    fn name(&self) -> &str {
        "steam-train"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Steam locomotive with cab, boiler, dome and flared stack pulls a tender and coach; connecting rods turn the wheels and smoke streams back"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let rail_y = dh - 1;
        let top = rail_y - 13; // sprite top row (row 7 = running board)

        // Rail + sleepers
        hl(grid, 0, dw - 1, rail_y);

        let head = lerp_i(27, dw + 1, ctx.eased);
        let lx = head - 27;
        let tx = lx - 13;
        let cx = tx - 20;

        // Locomotive
        stamp(grid, lx, top, &LOCO, 0, 0.0);
        let wy = top + 10;
        let ang = ctx.time * 6.0;
        let (pdx, pdy) = (
            (1.4 * ang.cos()).round() as i32,
            (1.4 * ang.sin()).round() as i32,
        );
        for wx in [6, 12, 18] {
            circle(grid, lx + wx, wy, 2);
        }
        // Connecting rod rides the crank pins of all three driving wheels
        line(grid, lx + 6 + pdx, wy + pdy, lx + 18 + pdx, wy + pdy);
        // Piston rod to the cylinder under the smokebox
        line(grid, lx + 18 + pdx, wy + pdy, lx + 22, top + 8);
        circle(grid, lx + 23, wy + 1, 1);

        // Tender + coach, coupled at running-board height
        stamp(grid, tx, top + 3, &TENDER, 0, 0.0);
        for wx in [2, 8] {
            circle(grid, tx + wx, wy + 1, 1);
        }
        hl(grid, tx + 11, lx - 1, top + 7);
        stamp(grid, cx, top + 2, &COACH, 0, 0.0);
        for wx in [3, 14] {
            circle(grid, cx + wx, wy + 1, 1);
        }
        hl(grid, cx + 18, tx - 1, top + 7);

        // Smoke puffs: born at the stack, drift back (left) and up, growing
        let sx = lx + 22;
        for i in 0..4 {
            let age = (ctx.time * 0.9 + i as f32 * 0.25).fract();
            let px = sx - (age * 18.0) as i32;
            let py = top - 1 - (age * 3.0) as i32;
            if age < 0.35 {
                disk(grid, px, py, 1);
            } else {
                circle(grid, px, py, 2);
            }
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Airplane takeoff
//    A side-view airliner — tail fin, window band, cockpit, swept wing and
//    under-wing engine — taxis along the runway for the first half of the bar,
//    then rotates nose-up and climbs out trailing a dashed contrail.  Runway
//    lights strobe in sequence and the tail beacon blinks with time.
// ─────────────────────────────────────────────────────────────────────────────

/// Airliner, nose to the right (32 x 10 dots).
const AIRLINER: [&str; 10] = [
    "##",
    "###",
    "####",
    "#####",
    "############################",
    "######.#.#.#.#.#.#.#.#.#.###..#",
    "..##############################",
    ".....#########################",
    "...........#######..######",
    ".........#####.......####",
];

struct AirplaneTakeoff;
impl ProgressStyle for AirplaneTakeoff {
    fn name(&self) -> &str {
        "airplane-takeoff"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Side-view airliner with swept wing, tail fin and window band taxis the runway, rotates nose-up and climbs out trailing a contrail; strobe lights and tail beacon blink"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let ground = dh - 1;
        let p = ctx.eased.clamp(0.0, 1.0);

        // Runway (solid) then grass verge (sparse) past the liftoff point
        let runway_end = dw * 3 / 4;
        hl(grid, 0, runway_end, ground);
        let mut x = runway_end + 3;
        while x < dw {
            draw::dot_i(grid, x, ground);
            x += 3;
        }
        // Sequenced strobe lights running toward the runway end
        let n_lights = 5;
        let lit = (ctx.time * 8.0) as i32 % n_lights;
        for i in 0..n_lights {
            let lx = runway_end - 4 - (n_lights - 1 - i) * 5;
            draw::dot_i(grid, lx, ground - 1);
            if i == lit {
                draw::dot_i(grid, lx, ground - 2);
                draw::dot_i(grid, lx - 1, ground - 1);
                draw::dot_i(grid, lx + 1, ground - 1);
            }
        }

        // Plane placement: taxi for p < 0.45, then climb
        let taxi_end = 0.45_f32;
        let nose = if p <= taxi_end {
            lerp_i(31, dw * 3 / 5, p / taxi_end)
        } else {
            lerp_i(dw * 3 / 5, dw - 2, (p - taxi_end) / (1.0 - taxi_end))
        };
        let climb = ((p - taxi_end) / (1.0 - taxi_end)).clamp(0.0, 1.0);
        let gear_rows = 1;
        let ground_top = ground - AIRLINER.len() as i32 - gear_rows;
        let alt = (climb.powf(1.3) * (ground_top + 1) as f32) as i32;
        let pitch = 0.1 * (climb * 4.0).min(1.0);
        let px = nose - 31;
        let py = ground_top - alt;
        stamp(grid, px, py, &AIRLINER, 16, pitch);

        if climb <= 0.0 {
            // Landing gear down: nose gear and main gear touching the runway
            vl(grid, px + 27, py + 8, ground - 1);
            for gx in [14, 15, 17] {
                draw::dot_i(grid, px + gx, ground - 1);
            }
            vl(grid, px + 15, py + 10, ground - 1);
        }

        // Anti-collision beacon blinking above the tail fin
        let tail_lift = ((-16) as f32 * pitch).round() as i32;
        if ((ctx.time * 3.0) as i32) % 2 == 0 {
            draw::dot_i(grid, px, py - 1 - tail_lift);
            draw::dot_i(grid, px + 1, py - 1 - tail_lift);
        }
        // Once airborne, a dashed contrail streams back from the tail
        if climb > 0.0 {
            let flow = (ctx.time * 12.0) as i32;
            for d in 2..(px + 2).clamp(2, 40) {
                if (d + flow).rem_euclid(5) < 3 {
                    let lift = ((-16 - d) as f32 * pitch).round() as i32;
                    draw::dot_i(grid, px - d, py + 5 - lift);
                }
            }
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Sailboat tacking
//    A sloop — hull with raised bow, mast with pennant, big mainsail and a
//    jib — sails rightward over rolling waves.  Each tack swings the sails
//    through (they narrow edge-on, then fill again).  The hull bobs and a
//    short wake trails behind with time.
// ─────────────────────────────────────────────────────────────────────────────

/// Sloop hull, bow to the right (20 x 4 dots).
const HULL: [&str; 4] = [
    "..................##",
    "####################",
    ".##################",
    "...##############",
];

struct SailboatTacking;
impl ProgressStyle for SailboatTacking {
    fn name(&self) -> &str {
        "sailboat-tacking"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Sloop with hull, mast, mainsail and jib sails right over rolling waves; the sails swing through each tack and the hull bobs"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let p = ctx.eased.clamp(0.0, 1.0);

        // Water: two rows of travelling wave crests
        let water = dh - 3;
        for row in 0..3 {
            let y = water + row;
            if row == 1 {
                continue;
            }
            let shift = (ctx.time * (6.0 + row as f32 * 3.0)) as i32 + row * 5;
            for x in 0..dw {
                if (x + shift).rem_euclid(10) < 3 + row / 2 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Boat placement + bob
        let bob = ((ctx.time * 2.2).sin() * 0.9).round() as i32;
        let bx = lerp_i(0, dw - 20, p);
        let hull_top = water - 3 + bob;
        stamp(grid, bx, hull_top - 1, &HULL, 0, 0.0);

        // Mast + pennant
        let mast_x = bx + 8;
        let mast_h = (hull_top - 1).clamp(2, 11);
        let mast_top = hull_top - mast_h;
        vl(grid, mast_x, mast_top, hull_top - 1);
        let flap = ((ctx.time * 7.0) as i32) % 2;
        draw::dot_i(grid, mast_x - 1, mast_top);
        draw::dot_i(grid, mast_x - 2, mast_top + flap);

        // Sails: width swings with the tack (edge-on mid-tack)
        let tack = (p * 3.0 * PI).cos().abs();
        let swing = 0.45 + 0.55 * tack;
        let main_w = (7.0 * swing).round() as i32;
        let jib_w = (8.0 * swing).round() as i32;
        let sail_rows = (mast_h - 2).max(1);
        for r in 0..sail_rows {
            let y = mast_top + 1 + r;
            let f = (r + 1) as f32 / sail_rows as f32;
            let mw = (main_w as f32 * f).round() as i32;
            if mw > 0 {
                hl(grid, mast_x - 1 - mw, mast_x - 1, y);
            }
            // Jib starts a little lower and keeps a gap from the mast
            let jf = ((r as f32 - 1.0) / sail_rows as f32).max(0.0);
            let jw = (jib_w as f32 * jf).round() as i32;
            if jw > 0 {
                hl(grid, mast_x + 2, mast_x + 1 + jw, y);
            }
        }

        // Wake: short dashes behind the stern
        for k in 0..3 {
            let wx = bx - 2 - k * 4 - ((ctx.time * 8.0) as i32 % 4);
            hl(grid, wx - 1, wx, hull_top + 2 + k % 2);
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Subway map
//    A horizontal line represents a transit route.  Station dots are placed at
//    equal intervals.  A "train" marker (filled disk) advances between stations
//    as eased grows.  Stations behind the train are shown as filled; ahead as
//    hollow circles.  The line itself is drawn as a thick rail.
// ─────────────────────────────────────────────────────────────────────────────

struct SubwayMap;
impl ProgressStyle for SubwayMap {
    fn name(&self) -> &str {
        "subway-map"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Subway route line: station dots fill as the train marker advances between stops"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let rail_y = dh / 2;

        // ── Rail (double line for visual weight) ──
        draw::hline(grid, 0, dw - 1, rail_y);
        if rail_y + 1 < dh {
            draw::hline(grid, 0, dw - 1, rail_y + 1);
        }

        // ── Stations ──
        let n_stations = (dw / 8).clamp(3, 16);
        let station_r = (dh as i32 / 6).max(1);

        for s in 0..n_stations {
            let sx = (s * dw) / (n_stations - 1).max(1);
            let t_station = sx as f32 / dw as f32;
            if t_station <= ctx.eased {
                // Visited: filled disk
                disk(grid, sx as i32, rail_y as i32, station_r);
            } else {
                // Ahead: hollow circle
                circle(grid, sx as i32, rail_y as i32, station_r);
            }
        }

        // ── Train marker ──
        let train_x = (ctx.eased * dw as f32) as i32;
        let train_r = (station_r * 2).max(2);
        disk(grid, train_x, rail_y as i32, train_r);
        // Halo / outline for distinction
        circle(grid, train_x, rail_y as i32, train_r + 1);

        // Station labels: small tick marks above each station
        for s in 0..n_stations {
            let sx = (s * dw) / (n_stations - 1).max(1);
            let tick_top = rail_y.saturating_sub(station_r as usize + 2);
            let tick_bot = rail_y.saturating_sub(station_r as usize + 1);
            draw::vline(grid, sx.min(dw - 1), tick_top, tick_bot);
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Hot-air balloon
//    A teardrop envelope (round crown tapering to the throat) divided into
//    gores, a wicker basket hung on ropes, and a burner flame that flickers in
//    the gap.  Progress lifts the balloon off the launch field and drifts it
//    rightward; clouds slide past and the balloon sways with time.
// ─────────────────────────────────────────────────────────────────────────────

struct HotAirBalloon;
impl ProgressStyle for HotAirBalloon {
    fn name(&self) -> &str {
        "hot-air-balloon"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Teardrop hot-air balloon with gored envelope, rope-hung basket and flickering burner lifts off and drifts up and right past clouds"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let ground = dh - 1;

        // Launch field: ground line with grass tufts that falls away below as
        // the balloon gains height.
        let ground = ground + (p * dh as f32 * 1.5) as i32;
        let mut gx = 0;
        while gx < dw {
            draw::dot_i(grid, gx, ground);
            if gx % 7 == 3 {
                draw::dot_i(grid, gx, ground - 1);
            }
            gx += 2;
        }

        // Clouds drifting left behind the balloon
        for i in 0..3 {
            let span = dw + 16;
            let cx = (dw + 8 - ((ctx.time * 3.0) as i32 + i * span / 3).rem_euclid(span)) as i32;
            let cy = 1 + (i * 3) % 5;
            hl(grid, cx - 3, cx + 3, cy + 1);
            hl(grid, cx - 1, cx + 1, cy);
        }

        // Envelope geometry (scaled to height)
        let rx = (dh * 6 / 16).max(2);
        let ry = (rx - 1).max(1);
        let taper = (rx * 2 / 3).max(1);
        let throat = (rx / 3).max(1);
        let env_h = ry + 1 + taper;
        let total = env_h + 1 + 2; // envelope + rope row + basket
        let sway = ((ctx.time * 1.6).sin() * 1.2).round() as i32;
        let bx = lerp_i(rx + 2, dw - rx - 3, p) + sway;
        let top = lerp_i(dh - total - 1, 0, p.powf(0.8));

        // Half-width of the envelope at row k (0 = crown)
        let half = |k: i32| -> f32 {
            if k <= ry {
                let d = (ry - k) as f32 / (ry as f32 + 0.6);
                rx as f32 * (1.0 - d * d).max(0.0).sqrt()
            } else {
                let f = (k - ry) as f32 / taper as f32;
                rx as f32 + (throat as f32 - rx as f32) * f
            }
        };
        for k in 0..env_h {
            let y = top + k;
            let w = half(k).round() as i32;
            let g = (half(k) * 0.45).round() as i32;
            for x in (bx - w)..=(bx + w) {
                // Gore seams: dark lines at +-45% of the width (not on the crown)
                let seam = k > 0 && w > 2 && ((x - bx).abs() == g);
                if !seam {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Throat mouth: open gap at the bottom centre
        let mouth_y = top + env_h - 1;

        // Ropes + basket
        let rope_y = mouth_y + 1;
        draw::dot_i(grid, bx - throat, rope_y);
        draw::dot_i(grid, bx + throat, rope_y);
        hl(grid, bx - throat, bx + throat, rope_y + 1);
        hl(grid, bx - throat + 1, bx + throat - 1, rope_y + 2);

        // Burner flame flickers between the ropes, licking into the mouth
        let flick = ((ctx.time * 11.0) as i32).rem_euclid(3);
        draw::dot_i(grid, bx, rope_y);
        if flick > 0 {
            draw::dot_i(grid, bx, mouth_y);
        }
        if flick == 2 {
            draw::dot_i(grid, bx - 1, rope_y);
            draw::dot_i(grid, bx + 1, rope_y);
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Ferris wheel
//    A big rimmed wheel with spokes turning about a hub on an A-frame, cabins
//    hanging level from the rim as it rotates.  Festoon light strings run from
//    lamp posts to the wheel; bulbs light up left-to-right with progress.
// ─────────────────────────────────────────────────────────────────────────────

struct FerrisWheel;
impl ProgressStyle for FerrisWheel {
    fn name(&self) -> &str {
        "ferris-wheel"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Big spoked Ferris wheel turns on an A-frame with cabins hanging level; festoon bulbs on the light strings switch on with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let ground = dh - 1;

        let r = ((dh - 2) / 2).max(2);
        let cx = dw / 2;
        let cy = ((dh - 1) / 2).max(1);
        let spin = ctx.time * 0.7;

        // Rim
        circle(grid, cx, cy, r);
        // Spokes from just outside the hub to the rim
        let n = 6;
        for s in 0..n {
            let a = spin + s as f32 / n as f32 * 2.0 * PI;
            let (c, si) = (a.cos(), a.sin());
            let x0 = cx + (2.0 * c).round() as i32;
            let y0 = cy + (2.0 * si).round() as i32;
            let x1 = cx + (r as f32 * c).round() as i32;
            let y1 = cy + (r as f32 * si).round() as i32;
            line(grid, x0, y0, x1, y1);
            // Cabin hangs level below its pivot on the rim
            hl(grid, x1 - 1, x1 + 1, y1 + 1);
            hl(grid, x1 - 1, x1 + 1, y1 + 2);
        }
        disk(grid, cx, cy, 1);

        // A-frame legs + base
        let leg = (r * 3 / 4).max(2);
        line(grid, cx, cy, cx - leg, ground);
        line(grid, cx, cy, cx + leg, ground);
        hl(grid, cx - leg - 2, cx + leg + 2, ground);

        // Festoon strings: lamp posts at the ends, sagging string to the wheel,
        // bulbs every 4 dots — lit bulbs (a 2x2 block) sweep in with progress.
        let post_top = 2.min(dh - 1);
        vl(grid, 1, post_top, ground);
        vl(grid, dw - 2, post_top, ground);
        let segs = [(1, cx - r - 3), (cx + r + 3, dw - 2)];
        let lit_x = (p * dw as f32) as i32;
        for (a, b) in segs {
            if b <= a {
                continue;
            }
            let span = (b - a) as f32;
            let sag = (dh as f32 * 0.22).max(1.0);
            let y_at = |x: i32| -> i32 {
                let u = (x - a) as f32 / span;
                post_top + (4.0 * sag * u * (1.0 - u)).round() as i32
            };
            for x in a..=b {
                if x % 2 == 0 {
                    draw::dot_i(grid, x, y_at(x));
                }
            }
            let mut x = a + 3;
            while x < b - 1 {
                let y = y_at(x) + 1;
                if x <= lit_x {
                    hl(grid, x, x + 1, y);
                    hl(grid, x, x + 1, y + 1);
                } else {
                    draw::dot_i(grid, x, y);
                }
                x += 4;
            }
        }
        // Leading bulb twinkles
        if ((ctx.time * 6.0) as i32) % 2 == 0 {
            draw::dot_i(grid, lit_x, dh - 3);
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Cable car / gondola
//    A diagonal cable runs from bottom-left to top-right.  A gondola box
//    (rectangle with a wheel hanger above) slides along the cable as eased
//    grows.  Support towers divide the cable into spans.
// ─────────────────────────────────────────────────────────────────────────────

struct CableCar;
impl ProgressStyle for CableCar {
    fn name(&self) -> &str {
        "cable-car"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Gondola climbs a diagonal cable from valley to peak; support towers mark intervals"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cable_x0 = 0_i32;
        let cable_y0 = (dh as i32) - 1;
        let cable_x1 = (dw as i32) - 1;
        let cable_y1 = 0_i32;

        // ── Cable ──
        line(grid, cable_x0, cable_y0, cable_x1, cable_y1);

        // ── Support towers (3 towers at 25%, 50%, 75%) ──
        let n_towers = 3usize;
        for t in 1..=n_towers {
            let frac = t as f32 / (n_towers + 1) as f32;
            let tx = (cable_x0 as f32 + (cable_x1 - cable_x0) as f32 * frac) as i32;
            let ty = (cable_y0 as f32 + (cable_y1 - cable_y0) as f32 * frac) as i32;
            // Tower = vertical line from cable attachment to ground
            draw::vline(
                grid,
                tx.max(0) as usize,
                ty.max(0) as usize,
                (cable_y0).min(dh as i32 - 1) as usize,
            );
            // Tower cap
            draw::dot_i(grid, tx - 1, ty);
            draw::dot_i(grid, tx + 1, ty);
        }

        // ── Gondola position along cable ──
        let g_frac = ctx.eased;
        let g_x = (cable_x0 as f32 + (cable_x1 - cable_x0) as f32 * g_frac) as i32;
        let g_y_cab = (cable_y0 as f32 + (cable_y1 - cable_y0) as f32 * g_frac) as i32;

        let car_w = ((dw as f32 * 0.07).max(3.0)) as i32;
        let car_h = ((dh as f32 * 0.20).max(2.0)) as i32;
        let hanger = 2_i32;

        // Wheel hanger: connects gondola to cable
        draw::vline(
            grid,
            g_x.max(0) as usize,
            g_y_cab.max(0) as usize,
            (g_y_cab + hanger).min(dh as i32 - 1).max(0) as usize,
        );
        draw::dot_i(grid, g_x - 1, g_y_cab);
        draw::dot_i(grid, g_x + 1, g_y_cab);

        // Gondola body (rectangle)
        let body_top = g_y_cab + hanger;
        let body_bot = (body_top + car_h).min(dh as i32 - 1);
        for y in body_top..=body_bot {
            draw::hline(
                grid,
                (g_x - car_w).max(0) as usize,
                (g_x + car_w).min(dw as i32 - 1).max(0) as usize,
                y.max(0) as usize,
            );
        }

        // Window
        let win_y = (body_top + car_h / 3).min(body_bot);
        draw::dot_i(grid, g_x, win_y);

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Escalator
//    Side view of an escalator between a lower and an upper floor: a stepped
//    staircase of treads and risers on a diagonal truss, with a handrail and
//    newel curves at both ends.  The steps climb continuously with time.  A
//    rider marks progress: walks along the lower floor, rides up the steps,
//    then walks off along the upper floor.
// ─────────────────────────────────────────────────────────────────────────────

/// Standing rider (3 x 5) and walking-stride variant.
const RIDER: [&str; 6] = [".##.", ".##.", "####", ".##.", ".##.", ".##."];
const RIDER_STRIDE: [&str; 6] = [".##.", ".##.", "####", ".##.", ".##.", "#..#"];

struct Escalator;
impl ProgressStyle for Escalator {
    fn name(&self) -> &str {
        "escalator"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Escalator between two floors: stepped truss and handrail with steps climbing over time; a rider walks on, rides up and steps off with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let p = ctx.eased.clamp(0.0, 1.0);

        let lo = dh - 1; // lower floor row
        let hi = (dh * 3 / 8).max(1).min(lo); // upper floor row
        let riser = 2;
        let tread = 4;
        let rise = (lo - hi).max(0);
        let run = rise / riser * tread;
        let xa = (dw - run) / 2; // foot of the incline
        let xb = xa + run; // head of the incline

        // Floors: lower floor, upper floor slab with its front edge
        hl(grid, 0, xa + 2, lo);
        hl(grid, xb, dw - 1, hi);
        hl(grid, xb + 2, dw - 1, hi + 1);
        vl(grid, xb, hi, hi + 3);

        // Rider: walk → ride → walk, mapped linearly on x.  A one-dot halo
        // around the rider keeps the handrail and steps from merging into it.
        let rx = lerp_i(3, dw - 5, p);
        let feet = if rx + 1 < xa {
            lo - 1
        } else if rx + 1 > xb {
            hi - 1
        } else {
            lo - 1 - (rx + 1 - xa) * riser / tread.max(1)
        };
        let halo = |x: i32, y: i32| x >= rx - 1 && x <= rx + 4 && y >= feet - 6 && y <= feet;

        // Steps: sawtooth profile marching up the incline with time
        let shift = ((ctx.time * 4.0) as i32).rem_euclid(tread);
        let mut prev_y = lo;
        for x in xa..=xb {
            let u = x - xa - shift;
            let k = u.div_euclid(tread) + 1;
            let y = (lo - k * riser - (shift * riser + tread / 2) / tread).clamp(hi, lo);
            let (y0, y1) = if x > xa {
                (y.min(prev_y), y.max(prev_y))
            } else {
                (y, y)
            };
            for yy in y0..=y1 {
                if !halo(x, yy) {
                    draw::dot_i(grid, x, yy);
                }
            }
            prev_y = y;
        }

        // Truss: solid side cladding band under the steps
        for x in (xa + 2)..=xb {
            let ny = lo - (x - xa) * riser / tread.max(1);
            vl(grid, x, ny + 3, (ny + 4).min(lo));
        }

        // Handrail parallel to the steps, flat runs + newel curves at the ends
        let hr = 5;
        for x in xa..=xb {
            let y = lo - hr - ((x - xa) * rise + run / 2) / run.max(1);
            if !halo(x, y) {
                draw::dot_i(grid, x, y);
            }
        }
        hl(grid, xa - 4, xa, lo - hr);
        vl(grid, xa - 5, lo - hr + 1, lo - hr + 2);
        hl(grid, xb, xb + 4, hi - hr);
        vl(grid, xb + 5, hi - hr + 1, hi - hr + 2);

        let on_floor = rx + 1 < xa || rx + 1 > xb;
        let stride = on_floor && ((ctx.time * 5.0) as i32) % 2 == 1;
        let sprite: &[&str] = if stride { &RIDER_STRIDE } else { &RIDER };
        stamp(grid, rx, feet - 5, sprite, 0, 0.0);

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Rocket gantry launch
//    A finned rocket stands on its pad beside a lattice gantry whose arms hold
//    it. Progress pumps propellant from a spherical tank along the ground line
//    to the pad, frosting the rocket's tanks from the bottom up. Near the end
//    the arms swing clear, the engines light, smoke rolls out of the flame
//    trench and the rocket climbs out of frame.
// ─────────────────────────────────────────────────────────────────────────────

/// Materials of the gantry scene, in rising priority. Each cell takes the
/// colour of the material most of its dots hold; rocket parts weigh more.
const G_STEEL: u8 = 1;
const G_SMOKE: u8 = 2;
const G_FUEL: u8 = 3;
const G_FLAME: u8 = 4;
const G_CORE: u8 = 5;
const G_HULL: u8 = 6;
const G_FROST: u8 = 7;
const G_TRIM: u8 = 8;

/// Fast integer hash → `[0, 1)` for smoke puffs.
fn gantry_hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    (x.wrapping_mul(2_246_822_519) % 1000) as f32 / 1000.0
}

/// A dot-space canvas of gantry-scene materials, flushed into a grid at once.
struct GantryCanvas {
    w: usize,
    h: usize,
    ink: Vec<u8>,
}

impl GantryCanvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            ink: vec![0; w * h],
        }
    }

    fn set(&mut self, x: i32, y: i32, ink: u8) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.ink[y as usize * self.w + x as usize] = ink;
        }
    }

    fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.ink[y as usize * self.w + x as usize]
        } else {
            0
        }
    }

    /// Paint an upright rocket: nozzle at `(cx, base)`, `len` dots tall, hull
    /// radius `r` — pointed nose cone, hull with a porthole, swept fins and a
    /// flickering two-tone flame `flame` dots long. Hull below `frost_y` is
    /// painted frosted (fuelled).
    #[allow(clippy::too_many_arguments)]
    fn rocket(&mut self, cx: f32, base: f32, len: f32, r: f32, flame: f32, frost_y: f32, t: f32) {
        let nose = len * 0.3;
        let nose_at = len - nose;
        let fin = len * 0.36;
        let flare = r * 0.9 + 0.8;
        let port_u = nose_at - r * 1.2;
        let reach = (r + flare + 1.0).ceil() as i32;
        for y in (base - len - 1.0) as i32..=(base + flame + 1.0) as i32 {
            for dx in -reach..=reach {
                // u climbs from the nozzle (0) to the nose tip (len).
                let u = base - (y as f32 + 0.5);
                let v = (dx as f32).abs();
                let x = cx as i32 + dx;
                let ink = if u > nose_at && u <= len {
                    let k = (u - nose_at) / nose;
                    (v <= r * (1.0 - k).powf(0.85) + 0.2).then_some(G_TRIM)
                } else if (0.0..=nose_at).contains(&u) {
                    let porthole = r >= 2.0 && (u - port_u).powi(2) + v * v < 1.1;
                    if v <= r && !porthole {
                        Some(if (y as f32) > frost_y {
                            G_FROST
                        } else {
                            G_HULL
                        })
                    } else if u < fin && v > r && v <= r + flare * (1.0 - u / fin) {
                        Some(G_TRIM)
                    } else {
                        None
                    }
                } else if (-1.5..0.0).contains(&u) && v > r * 0.75 && v <= r + flare {
                    // Fin tips sweep down past the nozzle.
                    Some(G_TRIM)
                } else if u < 0.0 && u >= -flame {
                    let k = -u / flame.max(0.1);
                    let jag = 0.75 + 0.25 * (t * 37.0 + v * 2.3 + u * 0.9).sin();
                    let width = r * 0.8 * (1.0 - k).powf(0.6) * jag + 0.3;
                    if v <= width {
                        Some(if k < 0.4 && v < width * 0.6 {
                            G_CORE
                        } else {
                            G_FLAME
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(ink) = ink {
                    self.set(x, y, ink);
                }
            }
        }
    }

    /// Set every inked dot, then colour each cell by its weighted majority.
    fn flush(&self, grid: &mut BrailleGrid) {
        grid.enable_color_support();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.ink[y * self.w + x] != 0 {
                    draw::dot(grid, x, y);
                }
            }
        }
        for cy in 0..(self.h + 3) / 4 {
            for cx in 0..(self.w + 1) / 2 {
                let mut counts = [0u8; 9];
                for y in cy * 4..(cy * 4 + 4).min(self.h) {
                    for x in cx * 2..(cx * 2 + 2).min(self.w) {
                        counts[self.ink[y * self.w + x] as usize] += 1;
                    }
                }
                let weight = |i: u8| {
                    counts[i as usize]
                        * match i {
                            G_STEEL | G_SMOKE => 1,
                            G_FUEL => 2,
                            _ => 3,
                        }
                };
                let best = (1..9u8).rev().max_by_key(|&i| weight(i)).unwrap_or(0);
                if counts[best as usize] > 0 {
                    let color = match best {
                        G_STEEL => crate::Color::rgb(130, 135, 150),
                        G_SMOKE => crate::Color::rgb(175, 175, 188),
                        G_FUEL => crate::Color::rgb(90, 200, 255),
                        G_FLAME => crate::Color::rgb(255, 140, 40),
                        G_CORE => crate::Color::rgb(255, 236, 150),
                        G_HULL => crate::Color::rgb(236, 238, 245),
                        G_FROST => crate::Color::rgb(185, 225, 255),
                        _ => crate::Color::rgb(235, 70, 60),
                    };
                    let _ = grid.set_cell_color(cx, cy, color);
                }
            }
        }
    }
}

struct RocketGantry;
impl ProgressStyle for RocketGantry {
    fn name(&self) -> &str {
        "rocket-gantry"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Rocket fuels beside its gantry, the arms swing clear, and it lifts off"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        let mut cv = GantryCanvas::new(w, h);
        let (wf, hf) = (w as f32, h as f32);
        let q = ctx.eased;
        let ground = h as i32 - 1;

        // Phases: fuelling to FUELLED, arms retract to IGNITE, then liftoff.
        const FUELLED: f32 = 0.78;
        const IGNITE: f32 = 0.86;
        let fuel = (q / FUELLED).min(1.0);
        let swing = ((q - FUELLED) / (IGNITE - FUELLED)).clamp(0.0, 1.0);
        let burn = ((q - IGNITE) / (1.0 - IGNITE)).clamp(0.0, 1.0);

        let len = (hf - 3.0).min(wf * 0.3);
        let r = (len / 6.2).max(0.8);
        let pad_x = wf * 0.74;
        // The climb ends with the rocket half out of frame on a long flame.
        let rise = burn * burn * (len * 0.55 + 2.0);
        let base = hf - 2.0 - rise;

        // Ground, and the spherical propellant tank on its legs.
        for x in 0..w as i32 {
            cv.set(x, ground, G_STEEL);
        }
        let tank_r = (hf * 0.28).min(wf * 0.08).max(1.0);
        let (tank_x, tank_y) = (tank_r + 2.0, hf - tank_r - 3.0);
        for y in (tank_y - tank_r) as i32..=(tank_y + tank_r) as i32 + 1 {
            for x in (tank_x - tank_r) as i32 - 1..=(tank_x + tank_r) as i32 + 1 {
                let d = (x as f32 + 0.5 - tank_x).hypot((y as f32 + 0.5 - tank_y) * 1.0);
                if d <= tank_r && d > tank_r - 1.2 {
                    cv.set(x, y, G_STEEL);
                } else if d <= tank_r - 1.2 && (y as f32) > tank_y + (fuel * 2.0 - 1.0) * tank_r {
                    // The tank drains as the rocket fills.
                    cv.set(x, y, G_FUEL);
                }
            }
        }
        for leg in [-0.7, 0.7] {
            let lx = (tank_x + leg * tank_r) as i32;
            for y in (tank_y + tank_r * 0.7) as i32..ground {
                cv.set(lx, y, G_STEEL);
            }
        }

        // The fuel line: an empty pipe that floods toward the pad with progress.
        let pipe0 = (tank_x + tank_r) as i32 + 1;
        let pipe1 = (pad_x - r - 1.0) as i32;
        let flow = pipe0 + ((pipe1 - pipe0) as f32 * fuel) as i32;
        let pulse = (ctx.time * 12.0) as i32;
        for x in pipe0..=pipe1 {
            cv.set(x, ground - 3, G_STEEL);
            if x < flow {
                cv.set(x, ground - 2, G_FUEL);
                if (x + pulse) % 5 != 0 {
                    cv.set(x, ground - 1, G_FUEL);
                }
            } else if x % 3 == 0 {
                cv.set(x, ground - 2, G_STEEL);
            }
        }

        // Lattice gantry tower to the left of the pad, X-braced.
        let tw = (r * 1.6).max(2.0) as i32;
        let tx1 = (pad_x - r - 3.0) as i32;
        let tx0 = tx1 - tw;
        let top = ((hf - 2.0 - len * 1.1) as i32).max(0);
        for y in top..ground {
            cv.set(tx0, y, G_STEEL);
            cv.set(tx1, y, G_STEEL);
            let k = (y - top).rem_euclid(tw * 2);
            let off = if k < tw { k } else { tw * 2 - k };
            cv.set(tx0 + off, y, G_STEEL);
        }
        // Swing arms: reach the hull until they retract toward the tower.
        let arm_reach = ((pad_x - r) as i32 - tx1) as f32 * (1.0 - swing);
        for arm_y in [hf - 2.0 - len * 0.45, hf - 2.0 - len * 0.8] {
            let ay = arm_y as i32;
            for x in tx1..=tx1 + arm_reach as i32 {
                cv.set(x, ay, G_STEEL);
            }
        }

        // Liftoff: smoke rolls out of the flame trench to both sides.
        if burn > 0.0 {
            let boil = (ctx.time * 5.0) as u32;
            let spread = 3.0 + burn.sqrt() * wf * 0.16;
            for i in 0..(spread * 0.9) as u32 {
                let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                let ox = side * gantry_hash(i * 29 + boil * 7) * spread;
                let rad = 1.0 + gantry_hash(i * 31 + boil * 3) * 2.2 * (1.0 - ox.abs() / spread);
                let (px, py) = (pad_x + ox, hf - 2.0 - gantry_hash(i * 37 + boil) * 2.0);
                for dy in -3..=1 {
                    for dx in -3..=3 {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d <= rad && (rad < 2.0 || d > rad - 1.3) {
                            cv.set(px as i32 + dx, py as i32 + dy, G_SMOKE);
                        }
                    }
                }
            }
        }

        let flame = if burn > 0.0 {
            r * (2.4 + 0.8 * (ctx.time * 23.0).sin().abs()) + rise
        } else {
            0.0
        };
        // Propellant level inside the rocket, rising from the nozzle.
        let frost_y = base - len * 0.7 * fuel;
        cv.rocket(pad_x, base, len, r, flame, frost_y, ctx.time);

        // Keep the arms in front of the hull while they still hold it.
        if swing < 1.0 && burn == 0.0 {
            for arm_y in [hf - 2.0 - len * 0.45, hf - 2.0 - len * 0.8] {
                let ay = arm_y as i32;
                let end = tx1 + arm_reach as i32;
                if cv.get(end + 1, ay) == G_HULL || cv.get(end + 1, ay) == G_FROST {
                    cv.set(end + 1, ay, 0);
                }
            }
        }

        cv.flush(grid);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Ferry crossing with wake
//     A side-view ferry — raked hull, two decks of windows and a funnel —
//     crosses from the left pier to the right pier.  Behind it a V wake opens
//     across the water (one arm toward the viewer, one toward the horizon)
//     with dashes running away from the stern; funnel smoke streams back and
//     wave crests travel with time.
// ─────────────────────────────────────────────────────────────────────────────

/// Ferry, bow to the right (32 x 10 dots).
const FERRY: [&str; 10] = [
    "...........##",
    "..........###",
    "..........###.........##",
    "......####################",
    "......#.#.#.#.#.#.#.#.#.###",
    "...###########################",
    "...##.##.##.##.##.##.##.##.##.#",
    "################################",
    ".##############################",
    "..###########################",
];

struct FerryCrossing;
impl ProgressStyle for FerryCrossing {
    fn name(&self) -> &str {
        "ferry-crossing"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Ferry with hull, two window decks and a funnel crosses between piers; a V wake opens behind, smoke streams back and waves roll"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let waterline = dh - 4; // hull's last row sits here

        // Piers: deck + pilings at both banks
        for (a, b) in [(0, 3), (dw - 4, dw - 1)] {
            hl(grid, a, b, waterline - 1);
            vl(grid, a + 1, waterline, dh - 1);
            vl(grid, b - 1, waterline, dh - 1);
        }

        // Wave crests on the water below the hull, travelling with time
        for (row, speed) in [(1, 5.0_f32), (3, 8.0)] {
            let y = waterline + row;
            let shift = (ctx.time * speed) as i32 + row * 4;
            for x in 5..dw - 5 {
                if (x + shift).rem_euclid(11) < 2 {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Ferry placement
        let fx = lerp_i(4, dw - 36, p);
        let top = waterline - FERRY.len() as i32 + 1;
        stamp(grid, fx, top, &FERRY, 0, 0.0);

        // V wake from the stern: near arm opens down toward the viewer, far
        // arm runs flat toward the horizon; dashes stream away from the boat.
        let wake_len = (fx - 4).clamp(0, 34);
        let apex_x = fx - 1;
        let apex_y = waterline + 1;
        let flow = (ctx.time * 10.0) as i32;
        for d in 0..wake_len {
            if (d + flow).rem_euclid(4) == 3 {
                continue;
            }
            let x = apex_x - d;
            let near = apex_y + (d * 2 + 6) / 14;
            draw::dot_i(grid, x, near.min(dh - 1));
            if d % 2 == 0 {
                draw::dot_i(grid, x, apex_y - (d + 8) / 16);
            }
        }
        // Churned water at the stern
        if (flow / 2) % 2 == 0 {
            draw::dot_i(grid, fx - 2, waterline);
            draw::dot_i(grid, fx - 3, waterline + 1);
        } else {
            draw::dot_i(grid, fx - 3, waterline);
            draw::dot_i(grid, fx - 2, waterline + 1);
        }

        // Funnel smoke streaming back
        for i in 0..3 {
            let age = (ctx.time * 0.8 + i as f32 / 3.0).fract();
            let sx = fx + 11 - (age * 14.0) as i32;
            let sy = top - 1 - (age * 2.0) as i32;
            hl(grid, sx - 1, sx + 1, sy);
            if age > 0.4 {
                draw::dot_i(grid, sx, sy - 1);
            }
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Bicycle
//     A diamond-frame bicycle — chainstays, seat and down tubes, top tube,
//     fork, saddle and handlebars — rolls rightward on two spoked wheels.
//     Spokes spin and the pedal crank turns with time.  Progress = distance.
// ─────────────────────────────────────────────────────────────────────────────

struct Bicycle;
impl ProgressStyle for Bicycle {
    fn name(&self) -> &str {
        "bicycle"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Diamond-frame bicycle with saddle and handlebars rolls right on spoked wheels; spokes spin and the pedal crank turns"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let ground = dh - 1;
        let r = ((dh - 7) / 2).max(2);
        let ay = ground - 1 - r;
        let wb = r * 2 + 10; // wheelbase

        let rear = lerp_i(r + 1, dw - wb - r - 2, ctx.eased);
        let front = rear + wb;
        let spin = ctx.time * 4.0;

        for wx in [rear, front] {
            circle(grid, wx, ay, r);
            for s in 0..3 {
                let a = spin + s as f32 * 2.0 * PI / 3.0;
                let sx = wx + ((r - 1) as f32 * a.cos()).round() as i32;
                let sy = ay + ((r - 1) as f32 * a.sin()).round() as i32;
                line(grid, wx, ay, sx, sy);
            }
        }

        // Frame key points
        let bb = (rear + wb * 9 / 20, ay + 1); // bottom bracket
        let seat = (rear + wb / 3, ay - r - 1); // seat cluster
        let head = (front - wb / 6, ay - r - 1); // head tube top
        let head_lo = (front - wb / 7, ay - r + 1);
        line(grid, rear, ay, bb.0, bb.1); // chainstay
        line(grid, rear, ay, seat.0, seat.1); // seatstay
        line(grid, bb.0, bb.1, seat.0, seat.1); // seat tube
        line(grid, seat.0, seat.1, head.0, head.1); // top tube
        line(grid, bb.0, bb.1, head_lo.0, head_lo.1); // down tube
        line(grid, head.0, head.1, front, ay); // fork

        // Saddle on a short post
        draw::dot_i(grid, seat.0 - 1, seat.1 - 1);
        hl(grid, seat.0 - 3, seat.0, seat.1 - 2);
        // Stem + handlebars sweeping back
        draw::dot_i(grid, head.0 - 1, head.1 - 1);
        hl(grid, head.0 - 3, head.0 - 1, head.1 - 2);
        draw::dot_i(grid, head.0 - 4, head.1 - 1);

        // Crank + pedals around the bottom bracket
        let crank = (r / 2).max(2) as f32;
        let ca = spin * 0.8;
        for sgn in [1.0_f32, -1.0] {
            let ex = bb.0 + (sgn * crank * ca.cos()).round() as i32;
            let ey = bb.1 + (sgn * crank * ca.sin()).round() as i32;
            line(grid, bb.0, bb.1, ex, ey);
            hl(grid, ex - 1, ex + 1, ey);
        }

        // Road with dashes scrolling back under the wheels
        let shift = (ctx.time * 12.0) as i32;
        for x in 0..dw {
            if (x + shift).rem_euclid(6) < 4 {
                draw::dot_i(grid, x, ground);
            }
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Helicopter
//     Side view: bubble cabin with a windscreen, tapering tail boom with fin
//     and tail rotor, skids on struts, and a two-blade main rotor whose
//     visible span sweeps with time.  It lifts off a helipad and climbs to
//     the right as progress grows, kicking up downwash while low.
// ─────────────────────────────────────────────────────────────────────────────

/// Helicopter body, nose to the right (26 x 9 dots).
const HELI: [&str; 9] = [
    ".#.................#",
    ".##.............######",
    ".###..........#######..#",
    ".############.#######....#",
    "..#.....##################",
    "..............###########",
    "................#######",
    "...............#.....#",
    "............##########.#",
];

struct Helicopter;
impl ProgressStyle for Helicopter {
    fn name(&self) -> &str {
        "helicopter"
    }
    fn theme(&self) -> &str {
        "transit"
    }
    fn describe(&self) -> &str {
        "Helicopter with bubble cabin, tail boom, tail rotor and skids lifts off a pad and climbs right; the main rotor sweeps and downwash kicks up"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let (dw, dh) = (dw as i32, dh as i32);
        let p = ctx.eased.clamp(0.0, 1.0);
        let ground = dh - 1;

        // Helipad: ground line with a raised pad on the left
        hl(grid, 0, dw - 1, ground);
        hl(grid, 2, 26, ground - 1);

        // Placement: lift off, then climb and fly right
        let rest_top = ground - 1 - HELI.len() as i32; // skids on the pad
        let lift = (p * 4.0).min(1.0);
        let alt = (lift * 2.0 + p * (rest_top as f32 - 1.0).max(0.0)) as i32;
        let hover = if p > 0.02 {
            ((ctx.time * 2.5).sin() * 0.8).round() as i32
        } else {
            0
        };
        let x = lerp_i(2, dw - 26, p);
        let y = rest_top - alt + hover;
        let tilt = if p > 0.05 { -0.06 } else { 0.0 }; // nose down in forward flight
        stamp(grid, x, y, &HELI, 13, tilt);

        // Main rotor: two blades seen edge-on — visible span sweeps with time,
        // faint tip marks hint at the full rotor disk.
        let mast_x = x + 19;
        let rotor_y = y - 1;
        let big_r = 14_i32;
        let ang = ctx.time * 9.0;
        let span = (big_r as f32 * ang.cos().abs()).round() as i32;
        hl(grid, mast_x - span, mast_x + span, rotor_y);
        draw::dot_i(grid, mast_x - big_r, rotor_y);
        draw::dot_i(grid, mast_x + big_r, rotor_y);

        // Tail rotor at the fin: a short spinning blade
        let ta = ctx.time * 13.0;
        let (tx, ty) = (x + 1, y + 1);
        let ex = (2.0 * ta.cos()).round() as i32;
        let ey = (2.0 * ta.sin()).round() as i32;
        line(grid, tx - ex, ty - ey, tx + ex, ty + ey);

        // Downwash: dust dashes blown outward along the ground while low
        if alt < 5 {
            let spread = (ctx.time * 10.0) as i32 % 6;
            let c = x + 18;
            for side in [-1, 1] {
                for k in 0..2 {
                    let d = 6 + spread + k * 6;
                    hl(grid, c + side * d, c + side * (d + 1), ground - 1);
                }
            }
        }

        tint_progress(grid, ctx);
        Ok(())
    }
}
