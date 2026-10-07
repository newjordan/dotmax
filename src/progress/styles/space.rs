//! Space / cosmic progress bars — twelve scenes, each drawn to look like its
//! name: a rocket lifting off its pad in a gravity turn, a warp-speed
//! starfield, a planet on a tilted orbit around a flaring sun, a comet with ion
//! and dust tails, a waxing cratered moon, a rotating spiral galaxy, a dish
//! beaming to a satellite, ringed Saturn, a lensed black hole, the Big Dipper
//! linking up, the space shuttle over the Earth's limb and the lunar module
//! touching down. Progress comes from `ctx.eased`, motion from `ctx.time`, and
//! each scene is coloured per cell (warm suns, pale moons, blue-white stars).
//! Every style returns `"space"` from `theme()`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Deterministic pseudo-random helper (no external crates).
// Returns a stable float in [0, 1) for star index `n`.
// ---------------------------------------------------------------------------
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

fn hash_f(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

// ---------------------------------------------------------------------------
// Public registry
// ---------------------------------------------------------------------------

/// All styles in the `space` theme, in display order.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(RocketLaunch),
        Box::new(StarfieldWarp),
        Box::new(PlanetOrbit),
        Box::new(CometTail),
        Box::new(MoonPhase),
        Box::new(GalaxySpiral),
        Box::new(SatelliteDish),
        Box::new(SaturnRings),
        Box::new(BlackHole),
        Box::new(Constellation),
        Box::new(SpaceShuttle),
        Box::new(LunarLander),
    ]
}

// ---------------------------------------------------------------------------
// 1 — Rocket launch
// ---------------------------------------------------------------------------

/// Materials of the launch scene, in rising priority: each cell takes the
/// colour of the material most of its dots hold (ties go to the later one).
const L_STAR: u8 = 1;
const L_SMOKE: u8 = 2;
const L_OCEAN: u8 = 3;
const L_CLOUD: u8 = 4;
const L_LIMB: u8 = 5;
const L_GROUND: u8 = 6;
const L_TILE: u8 = 7;
const L_FLAME: u8 = 8;
const L_CORE: u8 = 9;
const L_HULL: u8 = 10;
const L_TRIM: u8 = 11;
const L_GOLD: u8 = 12;
/// Number of material slots, `0` (empty) included.
const L_COUNT: usize = 13;

/// A dot-space canvas of launch-scene materials, flushed into a grid at once.
struct LaunchCanvas {
    w: usize,
    h: usize,
    ink: Vec<u8>,
}

impl LaunchCanvas {
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

    /// Paint a rocket whose nozzle sits at `(tx, ty)`, heading `ang` radians
    /// (0 = right, PI/2 = up): pointed nose cone, hull with a porthole, two
    /// swept fins and a flickering two-tone flame `flame` dots long.
    #[allow(clippy::too_many_arguments)]
    fn rocket(&mut self, tx: f32, ty: f32, ang: f32, len: f32, r: f32, flame: f32, t: f32) {
        let (ca, sa) = (ang.cos(), ang.sin());
        let half = len / 2.0;
        let (cx, cy) = (tx + ca * half, ty - sa * half);
        let nose = len * 0.3;
        let nose_at = half - nose;
        let fin = len * 0.36;
        let flare = r * 0.8 + 0.6;
        let reach = half + flame + flare + 2.0;
        let port_u = nose_at - r * 1.1;
        for y in (cy - reach).floor() as i32..=(cy + reach).ceil() as i32 {
            for x in (cx - reach).floor() as i32..=(cx + reach).ceil() as i32 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                // Rocket-local axes: u runs nozzle -> nose, v across the hull.
                let u = dx * ca - dy * sa;
                let v = (dx * sa + dy * ca).abs();
                let ink = if u > nose_at && u <= half {
                    let k = (u - nose_at) / nose;
                    (v <= r * (1.0 - k).powf(0.85) + 0.2).then_some(L_TRIM)
                } else if u >= -half && u <= nose_at {
                    let porthole = r >= 2.4 && (u - port_u).powi(2) + v * v < 1.1;
                    if v <= r && !porthole {
                        Some(L_HULL)
                    } else if u < -half + fin && v > r && v <= r + flare * (1.0 - (u + half) / fin)
                    {
                        Some(L_TRIM)
                    } else {
                        None
                    }
                } else if u < -half && u >= -half - 1.5 && v > r * 0.75 && v <= r + flare {
                    // Fin tips sweep back past the nozzle.
                    Some(L_TRIM)
                } else if u < -half && u >= -half - flame {
                    let k = (-half - u) / flame.max(0.1);
                    let jag = 0.75 + 0.25 * (t * 37.0 + v * 2.3 + u * 0.9).sin();
                    let width = r * 0.8 * (1.0 - k).powf(0.6) * jag + 0.3;
                    if v <= width {
                        Some(if k < 0.4 && v < width * 0.6 {
                            L_CORE
                        } else {
                            L_FLAME
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

    /// Set every inked dot, then colour each cell by its majority material.
    fn flush(&self, grid: &mut BrailleGrid, color: impl Fn(u8) -> Color) {
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
                let mut counts = [0u8; L_COUNT];
                for y in cy * 4..(cy * 4 + 4).min(self.h) {
                    for x in cx * 2..(cx * 2 + 2).min(self.w) {
                        counts[self.ink[y * self.w + x] as usize] += 1;
                    }
                }
                // The rocket outweighs the scenery it overlaps.
                let weight = |i: u8| {
                    counts[i as usize]
                        * match i {
                            L_FLAME | L_CORE => 4,
                            L_TILE | L_HULL | L_TRIM | L_GOLD => 3,
                            _ => 1,
                        }
                };
                let best = (1..L_COUNT as u8)
                    .rev()
                    .max_by_key(|&i| weight(i))
                    .unwrap_or(0);
                if counts[best as usize] > 0 {
                    let _ = grid.set_cell_color(cx, cy, color(best));
                }
            }
        }
    }
}

/// The nozzle's flight path at `q` in `0..=1`: a quarter-circle gravity turn
/// of radius `turn` off the pad, then level flight to `x_end`. Returns the
/// nozzle position and heading.
fn launch_path(q: f32, pad: (f32, f32), turn: f32, x_end: f32) -> (f32, f32, f32) {
    const TURN_SHARE: f32 = 0.3;
    if q < TURN_SHARE {
        let a = q / TURN_SHARE * PI / 2.0;
        (
            pad.0 + turn * (1.0 - a.cos()),
            pad.1 - turn * a.sin(),
            PI / 2.0 - a,
        )
    } else {
        let k = (q - TURN_SHARE) / (1.0 - TURN_SHARE);
        let x0 = pad.0 + turn;
        (x0 + (x_end - x0).max(0.0) * k, pad.1 - turn, 0.0)
    }
}

/// A finned rocket lifts off its pad, pitches over in a gravity turn and
/// flies on, leaving a J-shaped smoke trail under the stars.
struct RocketLaunch;
impl ProgressStyle for RocketLaunch {
    fn name(&self) -> &str {
        "rocket-launch"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "A rocket lifts off its pad, pitches over and flies on, trailing smoke"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mut cv = LaunchCanvas::new(w, h);
        let (wf, hf) = (w as f32, h as f32);
        let len = (hf - 4.0).min(wf * 0.18);
        let r = len / 7.2;

        // Twinkling stars across the sky.
        let slot = (ctx.time * 3.0) as u32;
        for i in 0..(w / 6) as u32 {
            let sx = (hash_f(i * 7 + 1) * wf) as i32;
            let sy = (hash_f(i * 7 + 2) * hf * 0.8) as i32;
            if hash_f(i * 7 + 3 + slot * 131) > 0.25 {
                cv.set(sx, sy, L_STAR);
            }
        }

        if r < 1.0 {
            // Too small for a silhouette: a smoke trail with a bright nose.
            let head = (ctx.eased * wf) as i32;
            let y = (h / 2) as i32;
            for x in 0..head {
                cv.set(x, y, L_SMOKE);
            }
            cv.set(head, y, L_TRIM);
        } else {
            let pad = (len * 0.7 + 2.0, hf - 2.0);
            let turn = (pad.1 - hf / 2.0).max(1.0);
            let x_end = wf - len - 1.0;

            // Launch pad and service tower.
            for x in 0..(pad.0 + len * 0.9) as i32 {
                cv.set(x, h as i32 - 1, L_GROUND);
            }
            let tower = (pad.0 - r - 3.0) as i32;
            for y in (pad.1 - len * 0.85) as i32..h as i32 {
                cv.set(tower, y, L_GROUND);
                if y % 3 == 0 {
                    cv.set(tower - 1, y, L_GROUND);
                    cv.set(tower + 1, y, L_GROUND);
                }
            }

            // Smoke trail along the path flown so far: overlapping puffs that
            // swell and hollow out with age. A ground cloud rolls out from the
            // pad while the launch is young.
            let q = ctx.eased;
            let (tx, ty, ang) = launch_path(q, pad, turn, x_end);
            let flame = r * (2.3 + 0.7 * (ctx.time * 23.0).sin().abs());
            let flown = (tx - pad.0).abs() + (pad.1 - ty).abs();
            let puffs = (flown / 2.0) as u32;
            let boil = (ctx.time * 4.0) as u32;
            for i in 0..puffs {
                let qi = q * i as f32 / puffs.max(1) as f32;
                let (sx, sy, _) = launch_path(qi, pad, turn, x_end);
                if (sx - tx).powi(2) + (sy - ty).powi(2) < (flame + 1.0).powi(2) {
                    continue;
                }
                let age = q - qi;
                let wobble = 0.8 + 0.4 * hash_f(i * 17 + boil * 101);
                let rad = (0.6 + age * 6.0).min(r * 1.3) * wobble;
                let rise = age * 2.0;
                for dy in -rad.ceil() as i32..=rad.ceil() as i32 {
                    for dx in -rad.ceil() as i32..=rad.ceil() as i32 {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d <= rad && (age < 0.1 || d > rad - 1.2) {
                            cv.set(sx as i32 + dx, (sy - rise) as i32 + dy, L_SMOKE);
                        }
                    }
                }
            }
            let cloud = (1.0 - q * 2.5).max(0.0) * len;
            for i in 0..(cloud / 2.0) as u32 {
                let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                let ox = side * (1.0 + hash_f(i * 29 + boil * 7) * cloud);
                let rad = 1.0 + hash_f(i * 31 + boil * 3) * 1.6;
                for dy in -2..=0 {
                    for dx in -2..=2 {
                        if ((dx * dx + dy * dy) as f32) <= rad * rad {
                            cv.set((pad.0 + ox) as i32 + dx, h as i32 - 2 + dy, L_SMOKE);
                        }
                    }
                }
            }

            cv.rocket(tx, ty, ang, len, r, flame, ctx.time);
        }

        cv.flush(grid, |ink| match ink {
            L_HULL => Color::rgb(235, 238, 245),
            L_TRIM => Color::rgb(235, 70, 60),
            L_CORE => Color::rgb(255, 236, 150),
            L_FLAME => Color::rgb(255, 140, 40),
            L_GROUND => Color::rgb(120, 120, 135),
            L_SMOKE => Color::rgb(170, 170, 185),
            _ => Color::rgb(150, 165, 230),
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2 — Starfield warp
// ---------------------------------------------------------------------------

/// Stars stream out of a central vanishing point and accelerate toward the
/// edges; progress raises the warp factor, stretching dots into long streaks.
struct StarfieldWarp;
impl ProgressStyle for StarfieldWarp {
    fn name(&self) -> &str {
        "starfield-warp"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Stars stream out of a vanishing point; progress raises the warp until they smear into long streaks"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 2 || h < 2 {
            return Ok(());
        }
        grid.enable_color_support();

        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let warp = ctx.eased.clamp(0.0, 1.0);
        // Lifecycles per second, and how far back along its path each streak reaches.
        let speed = 0.16 + warp * 0.85;
        let trail = 0.012 + warp * warp * 0.3;
        let n = ((w * h) / 22).clamp(6, 90) as u32;
        // Perspective-ish radial position: slow near the vanishing point,
        // accelerating as the star rushes past the viewer.
        let radius = |u: f32| 0.12 + 1.25 * u.clamp(0.0, 1.0).powf(1.7);
        // Radial directions use a less squashed vertical scale than the
        // frame, so steep streaks leave through the top and bottom edges.
        let ys = (cy * 2.4).min(cx);

        for i in 0..n {
            let ang = hash_f(i * 7 + 1) * 2.0 * PI;
            let (dx, dy) = (ang.cos() * cx, ang.sin() * ys);
            let rate = 0.75 + hash_f(i * 7 + 3) * 0.5;
            let u = (hash_f(i * 7 + 2) + ctx.time * speed * rate).fract();
            let r_head = radius(u);
            let r_tail = radius(u - trail * rate);
            let col = mix(STAR_DEEP, STAR_WHITE, (r_head * 1.4 - 0.1).min(1.0));
            line(
                grid,
                (cx + dx * r_tail) as i32,
                (cy + dy * r_tail) as i32,
                (cx + dx * r_head) as i32,
                (cy + dy * r_head) as i32,
                col,
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3 — Planet orbit
// ---------------------------------------------------------------------------

/// A sun with a tilted elliptical orbit; the planet (with its own little moon)
/// travels the orbit with progress, and the path it has covered lights up.
struct PlanetOrbit;
impl ProgressStyle for PlanetOrbit {
    fn name(&self) -> &str {
        "planet-orbit"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Planet circles a flaring sun on a tilted orbit; the travelled arc lights up and passes behind the sun"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        grid.enable_color_support();

        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let rx = w as f32 * 0.43;
        let ry = h as f32 * 0.42;
        let sun_r = (h as f32 * 0.28).min(w as f32 * 0.2).max(1.0);
        let pr = (h as f32 * 0.12).max(0.8);
        let ang = ctx.eased.clamp(0.0, 1.0) * 2.0 * PI;
        let pos = |a: f32| (cx - rx * a.cos(), cy + ry * a.sin());

        // Orbit path: solid where the planet has been, dotted ahead of it.
        let steps = (w * 3).clamp(24, 600);
        for s in 0..steps {
            let a = s as f32 / steps as f32 * 2.0 * PI;
            let (x, y) = pos(a);
            if a.sin() < 0.0 && ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() < sun_r + 1.5 {
                continue; // far side hidden behind the sun
            }
            if a <= ang {
                plotf(grid, x, y, ORBIT_LIT);
            } else if s % 4 == 0 {
                plotf(grid, x, y, ORBIT_DIM);
            }
        }

        let (px, py) = pos(ang);
        let behind = ang.sin() < 0.0;
        if behind {
            planet_with_moon(grid, px, py, pr, ctx.time);
        }

        // Sun: a filled disc with a slowly turning, flickering corona.
        disc(grid, cx, cy, sun_r, |d| {
            if d < 0.6 {
                SUN_CORE
            } else {
                SUN_EDGE
            }
        });
        let rays = 12;
        for k in 0..rays {
            let a = k as f32 / rays as f32 * 2.0 * PI + ctx.time * 0.25;
            let flick = 0.5 + 0.5 * (ctx.time * 5.0 + k as f32 * 2.3).sin();
            let r0 = sun_r + 1.2;
            let r1 = r0 + 1.0 + flick * 1.6;
            line(
                grid,
                (cx + a.cos() * r0).round() as i32,
                (cy + a.sin() * r0).round() as i32,
                (cx + a.cos() * r1).round() as i32,
                (cy + a.sin() * r1).round() as i32,
                SUN_RAY,
            );
        }

        if !behind {
            planet_with_moon(grid, px, py, pr, ctx.time);
        }
        Ok(())
    }
}

/// A small blue planet with a pale moon circling it in `time`.
fn planet_with_moon(grid: &mut BrailleGrid, px: f32, py: f32, pr: f32, time: f32) {
    let ma = time * 2.6;
    let (mx, my) = (px + ma.cos() * (pr + 2.5), py + ma.sin() * (pr * 0.6 + 0.6));
    let moon_behind = ma.sin() < 0.0;
    if moon_behind {
        plotf(grid, mx, my, MOON_LIT);
    }
    disc(grid, px, py, pr, |_| PLANET_BLUE);
    if !moon_behind {
        plotf(grid, mx, my, MOON_LIT);
    }
}

// ---------------------------------------------------------------------------
// 4 — Comet with tail
// ---------------------------------------------------------------------------

/// A bright comet nucleus crosses the sky with progress, trailing a straight
/// blue ion tail and a broad, curving dust tail that streams behind it.
struct CometTail;
impl ProgressStyle for CometTail {
    fn name(&self) -> &str {
        "comet-tail"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Comet nucleus crosses with progress, trailing a blue ion tail and a long fanning dust tail"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 2 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        background_stars(grid, w, h, ctx.time, 70);

        let p = ctx.eased.clamp(0.0, 1.0);
        let start = (wf * 0.3).min(wf - 2.0);
        let hx = start + p * (wf - 5.0 - start).max(0.0);
        let hy = hf * 0.45;
        let len = hx.min(wf * 0.85).max(1.0);

        // Dust tail: a cone of particles streaming away from the head,
        // fanning wider and curving gently downward, thinning at the far end.
        let n = ((w * h) / 3).clamp(8, 600) as u32;
        for j in 0..n {
            let rate = 0.6 + hash_f(j * 5 + 1) * 0.6;
            let u = (hash_f(j * 5 + 2) + ctx.time * 0.3 * rate).fract();
            if hash_f(j * 5 + 3) < u * 0.85 {
                continue;
            }
            let d = u * len;
            let s = hash_f(j * 5 + 4) * 2.0 - 1.0;
            let spread = (0.8 + d * 0.17).min(hf * 0.5);
            let curve = u * u * hf * 0.22;
            plotf(
                grid,
                hx - 1.0 - d,
                hy + curve + s * spread,
                mix(DUST_NEAR, DUST_FAR, u),
            );
        }

        // Ion tail: two straight streamers with dashes flowing backward.
        let ion_len = (len * 0.95) as i32;
        let flow = ctx.time * 26.0;
        for (k, slope) in [(0u32, -0.05f32), (1, -0.015)] {
            for d in 0..ion_len {
                let df = d as f32;
                let phase = (df + flow + k as f32 * 4.0) as i32;
                let keep = d < 8 || phase.rem_euclid(10) < 6;
                if !keep || hash_f(k * 997 + d as u32) < df / ion_len.max(1) as f32 * 0.5 {
                    continue;
                }
                plotf(grid, hx - 2.0 - df, hy + df * slope, ION_BLUE);
            }
        }

        // Coma: a soft, shimmering halo around the nucleus.
        for k in 0..16u32 {
            let a = k as f32 / 16.0 * 2.0 * PI + ctx.time * 1.7;
            let r = 2.8 + 0.5 * (ctx.time * 6.0 + k as f32).sin();
            plotf(grid, hx + a.cos() * r, hy + a.sin() * r * 0.85, COMA);
        }
        disc(grid, hx, hy, 1.6, |_| STAR_WHITE);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5 — Moon phase
// ---------------------------------------------------------------------------

/// A cratered moon waxing from a thin crescent to full, with a true elliptical
/// terminator, set against a twinkling sky with the odd shooting star.
struct MoonPhase;
impl ProgressStyle for MoonPhase {
    fn name(&self) -> &str {
        "moon-phase"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Cratered moon waxes from crescent to full along a curved terminator; stars twinkle around it"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 2 || h < 2 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        let (cx, cy) = (wf / 2.0, hf / 2.0);
        let r = (hf / 2.0 - 0.3).min(wf / 2.0 - 0.3).max(0.8);

        // Sky first, keeping clear of the moon.
        let n = ((w * h) / 45).clamp(2, 60) as u32;
        for i in 0..n {
            let x = hash_f(i * 3 + 11) * wf;
            let y = hash_f(i * 3 + 12) * hf;
            if (x - cx).abs() < r + 3.0 {
                continue;
            }
            let tw = (ctx.time * 2.2 + hash_f(i * 3 + 13) * 6.3).sin();
            if tw > -0.6 {
                let c = if tw > 0.6 { STAR_WHITE } else { STAR_DIM };
                plotf(grid, x, y, c);
            }
        }
        // Shooting star: one quick streak every four seconds.
        let s = (ctx.time / 4.0 + 0.3).fract();
        if s < 0.18 && w > 20 {
            let k = s / 0.18;
            let x = wf * 0.92 - k * wf * 0.3;
            let y = hf * 0.1 + k * hf * 0.25;
            line(
                grid,
                x as i32,
                y as i32,
                (x + 6.0) as i32,
                (y - 1.6) as i32,
                STAR_DIM,
            );
            plotf(grid, x, y, STAR_WHITE);
        }

        // Waxing: lit from the right; the terminator is a half-ellipse whose
        // half-width runs from +chord (new) through 0 (quarter) to -chord (full).
        let phase = 0.2 + ctx.eased.clamp(0.0, 1.0) * 0.8;
        let k = (PI * phase).cos();
        // Maria: darker seas rendered as half-density patches.
        let maria = [
            (-0.42f32, -0.08f32, 0.3f32, 0.48f32),
            (0.08, -0.42, 0.26, 0.18),
            (0.4, -0.02, 0.18, 0.26),
        ];
        let x0 = (cx - r - 1.0).max(0.0) as usize;
        let x1 = ((cx + r + 1.0) as usize).min(w);
        for y in 0..h {
            let dy = y as f32 + 0.5 - cy;
            if dy.abs() > r {
                continue;
            }
            let chord = (r * r - dy * dy).sqrt();
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let d2 = dx * dx + dy * dy;
                if d2 > r * r {
                    continue;
                }
                if dx >= chord * k {
                    let in_mare = maria.iter().any(|&(ox, oy, ex, ey)| {
                        let (u, v) = ((dx / r - ox) / ex, (dy / r - oy) / ey);
                        u * u + v * v < 1.0
                    });
                    if !in_mare || hash((x * 31 + y * 17) as u32) % 10 < 6 {
                        plot(grid, x as i32, y as i32, MOON_LIT);
                    }
                } else if d2 > (r - 1.1).powi(2) {
                    plot(grid, x as i32, y as i32, MOON_DARK);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6 — Galaxy spiral
// ---------------------------------------------------------------------------

/// A tilted two-armed spiral galaxy: bright core, logarithmic arms of stars
/// that light outward with progress, the whole disc slowly rotating.
struct GalaxySpiral;
impl ProgressStyle for GalaxySpiral {
    fn name(&self) -> &str {
        "galaxy-spiral"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Rotating two-armed spiral galaxy; its logarithmic arms fill with stars outward from the core"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        let (cx, cy) = (wf / 2.0, hf / 2.0);
        let (sx, sy) = (wf * 0.38, hf * 0.48);
        background_stars(grid, w, h, ctx.time, 90);

        // r = r0 * e^(b*theta); trailing arms, so the disc turns against the winding.
        let (r0, b) = (0.18f32, 0.36f32);
        let theta_max = (1.0 / r0).ln() / b;
        let rot = -ctx.time * 0.45;
        let lit = 0.12 + ctx.eased.clamp(0.0, 1.0) * 0.88;
        let steps = (w * 3).clamp(30, 400);
        for arm in 0..2u32 {
            let off = arm as f32 * PI;
            for s in 0..steps {
                let f = s as f32 / steps as f32;
                let th = f * theta_max;
                let r = r0 * (b * th).exp();
                let a = th + off + rot;
                let (x, y) = (cx + r * a.cos() * sx, cy + r * a.sin() * sy);
                if f <= lit {
                    let c = mix(CORE_WARM, ARM_BLUE, (r - r0) / (1.0 - r0));
                    plotf(grid, x, y, c);
                    // Scatter a cloud of stars around the spine, wider outward.
                    let seed = arm * 4001 + s as u32;
                    if hash_f(seed) < 0.6 {
                        let ja = a + (hash_f(seed + 17) - 0.5) * 0.3;
                        let jr = r * (1.0 + (hash_f(seed + 31) - 0.5) * 0.25);
                        plotf(grid, cx + jr * ja.cos() * sx, cy + jr * ja.sin() * sy, c);
                    }
                } else if s % 5 == 0 {
                    plotf(grid, x, y, ARM_DIM);
                }
            }
        }

        // Bright bulge at the hub.
        let (core_rx, core_ry) = ((sx * 0.08).max(1.0), (sy * 0.18).max(0.6));
        ellipse_fill(grid, cx, cy, core_rx, core_ry, CORE_HOT);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7 — Satellite dish
// ---------------------------------------------------------------------------

/// A parabolic dish on a tripod mount beams signal waves toward a satellite;
/// progress pushes the waves further out until they reach it.
struct SatelliteDish;
impl ProgressStyle for SatelliteDish {
    fn name(&self) -> &str {
        "satellite-dish"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Parabolic dish on a tripod beams signal waves toward a satellite; progress sets how far they reach"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        let sc = (hf / 16.0).min(wf / 40.0).max(0.5);

        // Dish in profile, aimed up-right at angle alpha: `u` runs along the
        // dish axis, `v` across the bowl.
        let alpha = 0.3f32;
        let (dx, dy) = (alpha.cos(), -alpha.sin());
        let (nx, ny) = (alpha.sin(), alpha.cos());
        let (vx, vy) = (2.0 + 6.0 * sc, hf * 0.45);
        let half = 6.0 * sc;
        let depth = 3.0 * sc;
        let at = |u: f32, v: f32| (vx + dx * u + nx * v, vy + dy * u + ny * v);

        // Mount: pivot behind the bowl, a post, and splayed tripod legs.
        let (pvx, pvy) = at(-1.8 * sc, 0.0);
        let ground = hf - 1.0;
        let knee = ground - 2.5 * sc;
        line(grid, pvx as i32, pvy as i32, pvx as i32, knee as i32, MOUNT);
        line(
            grid,
            pvx as i32,
            knee as i32,
            (pvx - 4.0 * sc) as i32,
            ground as i32,
            MOUNT,
        );
        line(
            grid,
            pvx as i32,
            knee as i32,
            (pvx + 4.0 * sc) as i32,
            ground as i32,
            MOUNT,
        );
        disc(grid, pvx, pvy, 1.0, |_| MOUNT);

        // Bowl: a crescent between the bright reflector face and its shaded
        // back, thickest at the vertex and tapering to the rim.
        let samples = (half * 8.0) as i32;
        for i in -samples..=samples {
            let v = i as f32 / samples.max(1) as f32 * half;
            let face = depth * (v / half).powi(2);
            let back = face * 0.7 - 1.8 * sc * (1.0 - (v / half).powi(2));
            let layers = ((face - back) * 2.0).clamp(0.0, 64.0) as i32;
            for j in 0..layers {
                let (x, y) = at(back + j as f32 * 0.5, v);
                plotf(grid, x, y, DISH_BACK);
            }
            let (x, y) = at(face, v);
            plotf(grid, x, y, DISH);
        }
        // Feed horn on a boom from the vertex, just past the focus.
        let (fx, fy) = at(half * half / (4.0 * depth) + 0.5 * sc, 0.0);
        let (bx0, by0) = at(0.5, 0.0);
        line(
            grid,
            bx0.round() as i32,
            by0.round() as i32,
            fx.round() as i32,
            fy.round() as i32,
            MOUNT,
        );
        disc(grid, fx, fy, 0.9, |_| FEED);

        // Satellite: body with two solar panels, upper right.
        let (satx, saty) = (wf - 9.0 * sc, (hf * 0.25).max(3.0));
        let reach_max = (satx - 8.0 * sc - fx).max(4.0);
        let reach = 5.0 + ctx.eased.clamp(0.0, 1.0) * (reach_max - 5.0).max(0.0);
        let linked = ctx.eased >= 0.97;
        let panel = if linked { PANEL_LIT } else { PANEL };
        let pw = (7.0 * sc).max(2.0) as i32;
        let ph = (4.0 * sc).max(2.0) as i32;
        let (bx, by) = (satx.round() as i32 + 1, saty.round() as i32);
        for x0 in [bx - 3 - pw, bx + 3] {
            let y0 = by - ph / 2;
            for yy in 0..ph {
                for xx in 0..pw {
                    let edge = yy == 0 || yy + 1 == ph || xx == 0 || xx + 1 == pw;
                    if edge || xx % 2 == 0 {
                        plot(grid, x0 + xx, y0 + yy, panel);
                    }
                }
            }
            line(grid, x0.min(bx), by, (x0 + pw).max(bx), by, MOUNT);
        }
        disc(grid, satx + 1.0, saty, 1.3, |_| SAT_BODY);
        if (ctx.time * 3.0).sin() > 0.0 || linked {
            plotf(grid, satx + 1.0, saty - 2.5 * sc, FEED);
        }

        // Signal waves: ")" fronts leave the feed and travel up to `reach`.
        let (ux, uy) = {
            let (ex, ey) = (satx - fx, saty - fy);
            let l = (ex * ex + ey * ey).sqrt().max(1.0);
            (ex / l, ey / l)
        };
        let spacing = 7.0 * sc;
        let travel = (ctx.time * 14.0 * sc).rem_euclid(spacing);
        let count = ((reach - travel - 2.0) / spacing).max(-1.0) as i32;
        for i in 0..=count {
            let r = travel + 2.0 + i as f32 * spacing;
            if r > reach {
                break;
            }
            let fade = r / reach.max(1.0);
            let col = mix(WAVE, WAVE_FAR, fade);
            let hv = (1.2 + r * 0.22).min(hf * 0.42);
            let bow = hv * 0.55;
            let (ccx, ccy) = (fx + ux * r, fy + uy * r);
            let rows = (hv * 2.0) as i32;
            for i in 0..=rows {
                let t = i as f32 / rows.max(1) as f32 * 2.0 - 1.0;
                plotf(grid, ccx - bow * t * t, ccy + t * hv, col);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8 — Saturn rings
// ---------------------------------------------------------------------------

/// Banded Saturn inside a tilted ring system that passes in front of the
/// planet below and behind it above; the rings sweep in with progress.
struct SaturnRings;
impl ProgressStyle for SaturnRings {
    fn name(&self) -> &str {
        "saturn-rings"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Banded Saturn whose tilted rings sweep around it with progress, passing in front and behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        let (cx, cy) = (wf / 2.0, hf / 2.0);
        let pr = (hf * 0.32).min(wf * 0.2).max(1.0);
        let outer = (wf * 0.33).min(pr * 6.0).max(pr + 2.0);
        let tilt = 0.17 + 0.025 * (ctx.time * 0.7).sin();
        let ry = outer * tilt;
        let fill = ctx.eased.clamp(0.0, 1.0);
        background_stars(grid, w, h, ctx.time, 140);

        // Ring edges in units of the outer radius: C/B rings, the Cassini
        // division, then the A ring.
        const RINGS: [(f32, bool); 3] = [(0.6, true), (0.78, true), (1.0, false)];
        let steps = ((outer * 8.0) as usize).clamp(16, 2000);
        for pass in 0..3 {
            if pass == 1 {
                // Banded globe, with a dark gap where the near rings cross it.
                let x0 = (cx - pr - 1.0).max(0.0) as usize;
                let x1 = ((cx + pr + 1.0) as usize).min(w);
                for y in 0..h {
                    for x in x0..x1 {
                        let dx = x as f32 + 0.5 - cx;
                        let dy = y as f32 + 0.5 - cy;
                        if dx * dx + dy * dy > pr * pr {
                            continue;
                        }
                        let q = ((dx / outer).powi(2) + (dy / ry).powi(2)).sqrt();
                        let swept = (PI - (dy / ry).atan2(dx / outer)) / (2.0 * PI);
                        if dy > 0.0 && (0.5..1.1).contains(&q) && swept <= fill {
                            continue;
                        }
                        let lat = dy / pr;
                        let belt = (-0.55..-0.35).contains(&lat) || (0.0..0.18).contains(&lat);
                        if !belt {
                            plot(grid, x as i32, y as i32, PLANET_TAN);
                        } else if (x + y) % 2 == 0 {
                            plot(grid, x as i32, y as i32, PLANET_BELT);
                        }
                    }
                }
                continue;
            }
            let front = pass == 2;
            for &(f, bright) in &RINGS {
                for s in 0..steps {
                    let theta = s as f32 / steps as f32 * 2.0 * PI - PI;
                    let (x, y) = (cx + outer * f * theta.cos(), cy + ry * f * theta.sin());
                    if (theta.sin() > 0.0) != front {
                        continue;
                    }
                    if !front && (x - cx).powi(2) + (y - cy).powi(2) < (pr + 0.4).powi(2) {
                        continue; // far side hidden behind the globe
                    }
                    // Sweep starts at the left limb and runs round through the front.
                    let swept = (PI - theta) / (2.0 * PI);
                    let gap = (theta * 11.0 - ctx.time * 1.6 + f * 9.0).sin() > 0.95;
                    if swept <= fill {
                        if !gap {
                            plotf(grid, x, y, if bright { RING_BRIGHT } else { RING_DIM });
                        }
                    } else if s % 5 == 0 && (x - cx).powi(2) + (y - cy).powi(2) > pr * pr {
                        plotf(grid, x, y, RING_GHOST);
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9 — Black hole
// ---------------------------------------------------------------------------

/// A dark event-horizon shadow ringed by a photon ring and a lensed halo,
/// with a swirling accretion disc that grows outward with progress.
struct BlackHole;
impl ProgressStyle for BlackHole {
    fn name(&self) -> &str {
        "black-hole"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "Dark event horizon in a photon ring and lensed halo; the swirling accretion disc widens with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 4 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        let (cx, cy) = (wf / 2.0, hf / 2.0);
        let sr = (hf * 0.26).min(wf * 0.2).max(1.0);
        let ring_r = sr + 1.0;
        let halo_in = sr + 2.0;
        let halo_out = sr + 3.1;
        let halo_sx = 1.35;
        let tilt = 0.11;
        let inner = halo_in + 1.0;
        let full = (wf * 0.47).max(inner + 2.0);
        let outer = inner + 3.0 + ctx.eased.clamp(0.0, 1.0) * (full - inner - 3.0).max(0.0);
        background_stars(grid, w, h, ctx.time, 160);

        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let dr = (dx * dx + dy * dy).sqrt();
                let q = (dx * dx + (dy / tilt).powi(2)).sqrt();
                let theta = (dy / tilt).atan2(dx);
                let hq = ((dx / halo_sx).powi(2) + dy * dy).sqrt();

                // Accretion disc, Keplerian swirl: inner material laps faster.
                if q >= inner && q <= outer {
                    let behind_halo = dy < 0.0 && hq < halo_out + 0.5;
                    let over_shadow = dy >= 0.0 && dr < ring_r + 0.5 && q > inner + 4.0;
                    if !behind_halo && !over_shadow {
                        let omega = 9.0 / q.max(1.0).sqrt();
                        let swirl = (theta * 2.0 + q * 0.45 + ctx.time * omega).sin();
                        let depth = (q - inner) / (full - inner).max(1.0);
                        if swirl > -0.2 + depth * 0.9 {
                            let mut c = mix(DISK_HOT, DISK_COOL, depth);
                            if dx < 0.0 {
                                c = mix(c, STAR_WHITE, 0.25); // Doppler-bright approaching side
                            }
                            plot(grid, x as i32, y as i32, c);
                        }
                        continue;
                    }
                }
                // Lensed image of the far disc, bent over the top (and faintly below).
                if hq >= halo_in && hq <= halo_out {
                    let a = dy.atan2(dx / halo_sx);
                    let glint = (a * 3.0 - ctx.time * 2.0).sin() > 0.9;
                    if dy < 0.0 && !glint || dy >= 0.0 && hq < halo_in + 0.6 && (x + y) % 2 == 0 {
                        plot(grid, x as i32, y as i32, HALO);
                    }
                    continue;
                }
                // Photon ring hugging the shadow; the shadow itself stays dark.
                if (dr - ring_r).abs() < 0.5 {
                    plot(grid, x as i32, y as i32, PHOTON);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10 — Constellation
// ---------------------------------------------------------------------------

/// The Big Dipper: its seven stars are always there; progress draws the
/// connecting lines one by one, a spark tracing each new segment.
struct Constellation;
impl ProgressStyle for Constellation {
    fn name(&self) -> &str {
        "constellation"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "The Big Dipper links up star by star with progress; a spark traces each new line as stars twinkle"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 2 || h < 2 {
            return Ok(());
        }
        grid.enable_color_support();

        let (wf, hf) = (w as f32, h as f32);
        // Alkaid, Mizar, Alioth, Megrez, Phecda, Merak, Dubhe (normalised sky).
        const STARS: [(f32, f32); 7] = [
            (0.04, 0.82),
            (0.24, 0.52),
            (0.41, 0.42),
            (0.57, 0.38),
            (0.62, 0.86),
            (0.90, 0.80),
            (0.94, 0.10),
        ];
        const EDGES: [(usize, usize); 7] = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 3)];
        let mx = wf * 0.06;
        let my = 1.5f32.min(hf * 0.2);
        let pos = |i: usize| -> (f32, f32) {
            let (fx, fy) = STARS[i];
            (mx + fx * (wf - 2.0 * mx), my + fy * (hf - 1.0 - 2.0 * my))
        };

        background_stars(grid, w, h, ctx.time, 60);

        let links = ctx.eased.clamp(0.0, 1.0) * EDGES.len() as f32;
        let mut joined = [false; 7];
        for (e, &(a, b)) in EDGES.iter().enumerate() {
            let f = (links - e as f32).clamp(0.0, 1.0);
            if f <= 0.0 {
                break;
            }
            let (ax, ay) = pos(a);
            let (bx, by) = pos(b);
            let (tx, ty) = (ax + (bx - ax) * f, ay + (by - ay) * f);
            line(
                grid,
                ax.round() as i32,
                ay.round() as i32,
                tx.round() as i32,
                ty.round() as i32,
                LINK,
            );
            joined[a] = true;
            if f >= 1.0 {
                joined[b] = true;
            } else {
                plotf(grid, tx, ty, STAR_WHITE);
            }
        }

        // Alcor, Mizar's faint companion.
        let (mzx, mzy) = pos(1);
        plotf(grid, mzx + 2.0, mzy - 2.5, STAR_DIM);

        for (i, &lit) in joined.iter().enumerate() {
            let (x, y) = pos(i);
            let (xi, yi) = (x.round() as i32, y.round() as i32);
            let twinkle = (ctx.time * 2.6 + i as f32 * 1.9).sin();
            let c = if lit { STAR_WHITE } else { STAR_BLUE };
            plot(grid, xi, yi, c);
            if lit || twinkle > 0.2 {
                plot(grid, xi - 1, yi, c);
                plot(grid, xi + 1, yi, c);
                plot(grid, xi, yi - 1, c);
                plot(grid, xi, yi + 1, c);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shared colours and drawing helpers (per-cell colour, bounds-safe)
// ---------------------------------------------------------------------------

const STAR_WHITE: Color = Color::rgb(236, 242, 255);
const STAR_BLUE: Color = Color::rgb(150, 175, 245);
const STAR_DIM: Color = Color::rgb(95, 110, 165);
const STAR_DEEP: Color = Color::rgb(70, 95, 190);
const ORBIT_LIT: Color = Color::rgb(120, 170, 255);
const ORBIT_DIM: Color = Color::rgb(70, 80, 125);
const SUN_CORE: Color = Color::rgb(255, 222, 110);
const SUN_EDGE: Color = Color::rgb(255, 170, 60);
const SUN_RAY: Color = Color::rgb(255, 128, 40);
const PLANET_BLUE: Color = Color::rgb(80, 175, 255);
const MOON_LIT: Color = Color::rgb(236, 232, 205);
const MOON_DARK: Color = Color::rgb(80, 86, 115);
const DUST_NEAR: Color = Color::rgb(255, 232, 190);
const DUST_FAR: Color = Color::rgb(170, 115, 85);
const ION_BLUE: Color = Color::rgb(110, 165, 255);
const COMA: Color = Color::rgb(170, 225, 255);
const CORE_HOT: Color = Color::rgb(255, 244, 210);
const CORE_WARM: Color = Color::rgb(245, 205, 150);
const ARM_BLUE: Color = Color::rgb(120, 155, 255);
const ARM_DIM: Color = Color::rgb(60, 70, 125);
const DISH: Color = Color::rgb(215, 220, 230);
const MOUNT: Color = Color::rgb(135, 140, 160);
const DISH_BACK: Color = Color::rgb(165, 172, 190);
const FEED: Color = Color::rgb(255, 110, 90);
const WAVE: Color = Color::rgb(110, 245, 200);
const WAVE_FAR: Color = Color::rgb(55, 140, 130);
const PANEL: Color = Color::rgb(80, 120, 230);
const PANEL_LIT: Color = Color::rgb(140, 200, 255);
const SAT_BODY: Color = Color::rgb(235, 195, 95);
const PLANET_TAN: Color = Color::rgb(236, 204, 145);
const PLANET_BELT: Color = Color::rgb(195, 150, 100);
const RING_BRIGHT: Color = Color::rgb(232, 218, 180);
const RING_DIM: Color = Color::rgb(175, 155, 125);
const RING_GHOST: Color = Color::rgb(85, 80, 75);
const DISK_HOT: Color = Color::rgb(255, 228, 170);
const DISK_COOL: Color = Color::rgb(225, 85, 35);
const HALO: Color = Color::rgb(255, 185, 105);
const PHOTON: Color = Color::rgb(255, 245, 220);
const LINK: Color = Color::rgb(90, 125, 215);

/// Linear blend of two colours, `t` in `[0, 1]`.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    Color::rgb(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b))
}

/// Set one dot and colour its cell; out-of-bounds writes are ignored.
fn plot(grid: &mut BrailleGrid, x: i32, y: i32, c: Color) {
    if x < 0 || y < 0 {
        return;
    }
    let (w, h) = draw::dot_dims(grid);
    let (xu, yu) = (x as usize, y as usize);
    if xu < w && yu < h {
        let _ = grid.set_dot(xu, yu);
        let _ = grid.set_cell_color(xu / 2, yu / 4, c);
    }
}

/// [`plot`] at a rounded float position.
fn plotf(grid: &mut BrailleGrid, x: f32, y: f32, c: Color) {
    plot(grid, x.round() as i32, y.round() as i32, c);
}

/// Bresenham line between two dot positions (endpoints clamped to a sane range).
fn line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let lim = |v: i32| v.clamp(-2048, 2048);
    let (mut x, mut y, x1, y1) = (lim(x0), lim(y0), lim(x1), lim(y1));
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        plot(grid, x, y, c);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Filled disc; `shade` maps normalised distance from the centre to a colour.
fn disc(grid: &mut BrailleGrid, cx: f32, cy: f32, r: f32, shade: impl Fn(f32) -> Color) {
    let r = r.clamp(0.0, 512.0);
    let (x0, x1) = ((cx - r).floor() as i32, (cx + r).ceil() as i32);
    let (y0, y1) = ((cy - r).floor() as i32, (cy + r).ceil() as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            if d <= r + 0.25 {
                plot(grid, x, y, shade(d / r.max(0.01)));
            }
        }
    }
}

/// Filled axis-aligned ellipse.
fn ellipse_fill(grid: &mut BrailleGrid, cx: f32, cy: f32, rx: f32, ry: f32, c: Color) {
    let (rx, ry) = (rx.clamp(0.1, 512.0), ry.clamp(0.1, 512.0));
    let (x0, x1) = ((cx - rx).floor() as i32, (cx + rx).ceil() as i32);
    let (y0, y1) = ((cy - ry).floor() as i32, (cy + ry).ceil() as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (ex, ey) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
            if ex * ex + ey * ey <= 1.05 {
                plot(grid, x, y, c);
            }
        }
    }
}

/// Sparse, dim, slowly twinkling field stars (`density` = dots per star).
fn background_stars(grid: &mut BrailleGrid, w: usize, h: usize, time: f32, density: usize) {
    let n = ((w * h) / density.max(1)).min(64) as u32;
    for i in 0..n {
        let x = hash_f(i * 13 + 500) * w as f32;
        let y = hash_f(i * 13 + 501) * h as f32;
        if (time * 1.7 + hash_f(i * 13 + 502) * 6.3).sin() > -0.7 {
            plot(grid, x as i32, y as i32, STAR_DIM);
        }
    }
}

// ---------------------------------------------------------------------------
// 11 — Space shuttle
// ---------------------------------------------------------------------------

/// The orbiter in side profile, nose right: `T` tail fin, `W` white hull,
/// `K` cockpit windows (left unlit), `B` black heat-shield tiles, `E` engines.
const SHUTTLE: [&str; 11] = [
    "..TTT...........................",
    "..TTTT..........................",
    "..TTTTT.........................",
    "..TTTTTT........................",
    ".WWWWWWWWWWWWWWWWWWWWWWWWWW.....",
    "EWWWWWWWWWWWWWWWWWWWWWWWWWWKKW..",
    "EWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW.",
    "EWWWWWWWWWWWWWWWWWWWWWWWWWWWWWBB",
    ".BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.",
    "...BBBBBBBBBBBBBBBB.............",
    ".....BBBBBBBBBB.................",
];

/// The orbiter glides over the curved, cloud-streaked limb of the Earth.
struct SpaceShuttle;
impl ProgressStyle for SpaceShuttle {
    fn name(&self) -> &str {
        "space-shuttle"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "The orbiter glides over the Earth's curved blue limb, firing its OMS engines"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mut cv = LaunchCanvas::new(w, h);
        let (wf, hf) = (w as f32, h as f32);

        // Stars, twinkling.
        let slot = (ctx.time * 3.0) as u32;
        for i in 0..(w / 7) as u32 {
            let sx = (hash_f(i * 5 + 11) * wf) as i32;
            let sy = (hash_f(i * 5 + 12) * hf * 0.7) as i32;
            if hash_f(i * 5 + 13 + slot * 131) > 0.3 {
                cv.set(sx, sy, L_STAR);
            }
        }

        // The Earth's limb: a vast circle whose top just enters the frame,
        // a bright atmosphere line over cloud bands that drift with time.
        let big_r = wf * 1.1;
        let earth_top = hf - (hf * 0.2).max(2.0);
        let drift = ctx.time * 6.0;
        for x in 0..w as i32 {
            let off = x as f32 - wf / 2.0;
            let edge = earth_top + big_r - (big_r * big_r - off * off).max(0.0).sqrt();
            let ey = edge.round() as i32;
            cv.set(x, ey, L_LIMB);
            cv.set(x, ey + 1, L_LIMB);
            for y in ey + 2..h as i32 {
                let n = hash_f(((x as f32 + drift) as u32 / 5) * 7 + y as u32 * 131);
                cv.set(x, y, if n > 0.68 { L_CLOUD } else { L_OCEAN });
            }
        }

        let (sw, sh) = (SHUTTLE[0].len() as i32, SHUTTLE.len() as i32);
        if w as i32 >= sw + 4 && h as i32 >= sh {
            // Rows land on whole cells so hull, tiles and fin keep their colours.
            let top = ((h as i32 - sh - 3).max(0) / 4) * 4;
            let x0 = (ctx.eased * (wf - sw as f32 - 3.0)) as i32 + 3;
            for (dy, row) in SHUTTLE.iter().enumerate() {
                for (dx, ch) in row.chars().enumerate() {
                    let ink = match ch {
                        'W' | 'T' => L_HULL,
                        'B' => L_TILE,
                        'E' => L_GROUND,
                        _ => continue,
                    };
                    cv.set(x0 + dx as i32, top + dy as i32, ink);
                }
            }
            // OMS burn: a short flickering flame off the tail.
            let flen = 2 + ((ctx.time * 19.0).sin().abs() * 3.0) as i32;
            for k in 1..=flen {
                cv.set(x0 - k, top + 6, if k <= 1 { L_CORE } else { L_FLAME });
                if k < flen - 1 {
                    cv.set(x0 - k, top + 5, L_FLAME);
                    cv.set(x0 - k, top + 7, L_FLAME);
                }
            }
        } else {
            // Too small for the orbiter: a bright dot gliding along the limb.
            let x = (ctx.eased * (wf - 1.0)) as i32;
            cv.set(x, 0, L_HULL);
        }

        cv.flush(grid, |ink| match ink {
            L_HULL => Color::rgb(238, 240, 246),
            L_TILE => Color::rgb(100, 100, 118),
            L_CORE => Color::rgb(255, 236, 150),
            L_FLAME => Color::rgb(255, 140, 40),
            L_GROUND => Color::rgb(120, 120, 135),
            L_LIMB => Color::rgb(120, 200, 255),
            L_CLOUD => Color::rgb(205, 222, 240),
            L_OCEAN => Color::rgb(40, 90, 200),
            _ => Color::rgb(150, 165, 230),
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12 — Lunar lander
// ---------------------------------------------------------------------------

/// The Apollo lunar module, side on: `a` silver ascent stage (its window left
/// dark), `d` gold-foil descent stage, `l` splayed legs and footpads, `e` the
/// descent engine bell.
const LANDER: [&str; 10] = [
    ".....aaaaa.....",
    "....aa.aaaa....",
    "....aaaaaaa....",
    "...ddddddddd...",
    "..ddddddddddd..",
    "...ddddddddd...",
    "..l...eee...l..",
    ".l....eee....l.",
    ".l...........l.",
    "ll...........ll",
];

/// The lunar module descends across a cratered surface and touches down by
/// the flag, the Earth hanging in the black sky.
struct LunarLander;
impl ProgressStyle for LunarLander {
    fn name(&self) -> &str {
        "lunar-lander"
    }
    fn theme(&self) -> &str {
        "space"
    }
    fn describe(&self) -> &str {
        "The lunar module descends over craters on its engine and touches down by the flag"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mut cv = LaunchCanvas::new(w, h);
        let (wi, hi) = (w as i32, h as i32);
        let ground = (hi - 3).max(0);

        // Stars, and the Earth hanging low over the horizon.
        let slot = (ctx.time * 2.0) as u32;
        for i in 0..(w / 8) as u32 {
            let sx = (hash_f(i * 3 + 21) * w as f32) as i32;
            let sy = (hash_f(i * 3 + 22) * (ground as f32 - 2.0).max(1.0)) as i32;
            if hash_f(i * 3 + 23 + slot * 131) > 0.3 {
                cv.set(sx, sy, L_STAR);
            }
        }
        let (ex, ey) = (wi * 3 / 5, 3);
        for y in ey - 3..=ey + 3 {
            for x in ex - 3..=ex + 3 {
                if (x - ex) * (x - ex) + (y - ey) * (y - ey) <= 10 {
                    let land = (x - ex + 1) * (y - ey) > 1;
                    cv.set(x, y, if land { L_CLOUD } else { L_OCEAN });
                }
            }
        }

        // Regolith with crater rims, and the flag at the landing site.
        for x in 0..wi {
            let rim = ((x as f32 * 0.31).sin() + (x as f32 * 0.13).sin() > 1.3) as i32;
            cv.set(x, ground - rim, L_GROUND);
            for y in ground + 1..hi {
                if hash_f(x as u32 * 53 + y as u32 * 7) > 0.6 {
                    cv.set(x, y, L_GROUND);
                }
            }
        }
        let flag_x = wi - 4;
        for y in ground - 6..ground {
            cv.set(flag_x, y, L_HULL);
        }
        for y in ground - 6..ground - 4 {
            for x in flag_x + 1..flag_x + 4 {
                cv.set(x, y, L_TRIM);
            }
        }

        let (lw, lh) = (LANDER[0].len() as i32, LANDER.len() as i32);
        if wi < lw + 10 || hi < lh + 3 {
            return {
                cv.flush(grid, lander_color);
                Ok(())
            };
        }
        // Glide in from high on the left; feet meet the ground at 100%.
        let q = ctx.eased;
        let x0 = 2 + (q * (flag_x - lw - 4) as f32) as i32;
        let land_top = ground - lh;
        let top = (land_top as f32 * q.powf(0.7)).round() as i32;
        let landed = q >= 0.99;
        if !landed {
            // Descent engine: a plume that shortens as the ground comes up,
            // kicking up regolith once the module is low.
            let room = (ground - (top + lh)).max(0);
            let plume = (2 + room).min(5) - ((ctx.time * 17.0).sin().abs() * 1.5) as i32;
            for k in 0..plume {
                let y = top + 8 + k;
                cv.set(x0 + 7, y, if k < 2 { L_CORE } else { L_FLAME });
                if k < plume - 1 {
                    cv.set(x0 + 6, y, L_FLAME);
                    cv.set(x0 + 8, y, L_FLAME);
                }
            }
            if room <= 2 {
                for i in 0..10u32 {
                    let spread = (hash_f(i * 41 + slot * 13) * 14.0) as i32;
                    let side = if i % 2 == 0 { 1 } else { -1 };
                    cv.set(x0 + 7 + side * (4 + spread), ground - 1, L_SMOKE);
                }
            }
        }
        for (dy, row) in LANDER.iter().enumerate() {
            for (dx, ch) in row.chars().enumerate() {
                let ink = match ch {
                    'a' => L_HULL,
                    'd' => L_GOLD,
                    'l' | 'e' => L_GROUND,
                    _ => continue,
                };
                cv.set(x0 + dx as i32, top + dy as i32, ink);
            }
        }

        cv.flush(grid, lander_color);
        Ok(())
    }
}

/// Colours for the lunar scene.
fn lander_color(ink: u8) -> Color {
    match ink {
        L_HULL => Color::rgb(215, 220, 230),
        L_GOLD => Color::rgb(235, 185, 60),
        L_TRIM => Color::rgb(235, 70, 60),
        L_CORE => Color::rgb(255, 236, 150),
        L_FLAME => Color::rgb(255, 140, 40),
        L_GROUND => Color::rgb(150, 150, 160),
        L_SMOKE => Color::rgb(190, 190, 200),
        L_OCEAN => Color::rgb(60, 120, 230),
        L_CLOUD => Color::rgb(120, 200, 140),
        _ => Color::rgb(150, 165, 230),
    }
}
