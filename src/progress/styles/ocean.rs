//! Ocean / aquatic progress bars.
//!
//! Eleven animated braille bars inspired by the sea: tides, waves, bubbles,
//! a fish, a sonar submarine, a jellyfish, a coral reef, kelp, ripple
//! interference, a deep-ocean depth gauge and a spouting whale. Every bar uses `ctx.time` for continuous animation
//! and `ctx.eased` for fill/advance driven by progress.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

/// All styles in the `ocean` theme.
///
/// Returns eleven boxed ocean-themed bars ready to be mixed into any registry
/// or rendered directly via [`ProgressStyle::render`].
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(RisingTide),
        Box::new(BubblesRising),
        Box::new(FishSwim),
        Box::new(WaveCrest),
        Box::new(SonarPing),
        Box::new(DepthGauge),
        Box::new(JellyfishPulse),
        Box::new(CoralReef),
        Box::new(Seaweed),
        Box::new(RippleInterference),
        Box::new(WhaleSpout),
    ]
}

// ---------------------------------------------------------------------------
// 1. Rising Tide
// ---------------------------------------------------------------------------
// Water level climbs with eased progress; the surface is a live sine wave.

struct RisingTide;
impl ProgressStyle for RisingTide {
    fn name(&self) -> &str {
        "rising-tide"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Water level rises with eased progress; animated sine-wave surface"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Water level: 0 = empty (bottom), h = full (top). eased controls it.
        let water_h = (ctx.eased * h as f32).round() as usize;
        let water_top = h.saturating_sub(water_h); // dot-y of the surface line

        // Fill body below the surface.
        if water_h > 1 {
            draw::fill_rect(grid, 0, water_top + 1, w, h.saturating_sub(water_top + 1));
        }

        // Animated surface wave — one row of sine-displaced dots.
        let amp = (h as f32 * 0.08).max(1.0);
        for x in 0..w {
            let phase = ctx.time * 2.5 + x as f32 * 0.35;
            let dy = (phase.sin() * amp).round() as i32;
            let sy = water_top as i32 + dy;
            draw::dot_i(grid, x as i32, sy);
        }

        // Tint: deep ocean blue at bottom, sea-foam cyan at surface.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let frac = 1.0 - cy as f32 / cells_h.max(1) as f32; // 0 at top, 1 at bottom
            let color = ctx.palette.sample(frac);
            let dot_y_of_cell = cy * 4;
            if dot_y_of_cell >= water_top {
                draw::tint_row(grid, cy, 0, ctx.width.saturating_sub(1), color);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Bubbles Rising
// ---------------------------------------------------------------------------
// Hollow round bubbles of three sizes float upward with sinusoidal lateral
// wobble; spawn density ∝ progress.

struct BubblesRising;
impl ProgressStyle for BubblesRising {
    fn name(&self) -> &str {
        "bubbles-rising"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Round, glinting bubbles of three sizes wobble upward; more bubbles rise as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of bubbles tied to progress (minimum 1 while progress > 0).
        let n_bubbles = ((ctx.eased * 14.0).round() as usize).max(usize::from(ctx.progress > 0.0));

        for i in 0..n_bubbles {
            // Each bubble has a fixed column origin spread across the width.
            let base_x = if n_bubbles == 1 {
                w / 2
            } else {
                (i * w) / n_bubbles + w / (n_bubbles * 2).max(1)
            };
            // Independent speed and phase per bubble via prime-ish offsets.
            let speed = 0.5 + (i as f32 * 0.17) % 0.8;
            let phase_offset = i as f32 * 1.3;
            // y travels 0 (top) to h (bottom) and wraps.
            let travel = (ctx.time * speed + phase_offset) % 1.0;
            let y = h.saturating_sub(1) - (travel * h as f32) as usize;
            // Lateral sine wobble.
            let wobble = ((ctx.time * 1.8 + i as f32 * 0.9).sin() * 2.0).round() as i32;
            let bx = base_x as i32 + wobble;
            // Hollow round bubble (radius 1–3) with a glint inside.
            let r = 1 + (i % 3) as i32;
            let y = y as i32;
            for s in 0..(r * 8) {
                let a = s as f32 / (r * 8) as f32 * 2.0 * PI;
                draw::dot_i(
                    grid,
                    bx + (a.cos() * r as f32).round() as i32,
                    y + (a.sin() * r as f32).round() as i32,
                );
            }
            if r > 1 {
                draw::dot_i(grid, bx - 1, y - 1);
            }
        }

        // Tint the whole grid with a soft deep-ocean gradient.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            let color = ctx.palette.sample(1.0 - t * 0.6);
            draw::tint_row(grid, cy, 0, ctx.width.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Fish Swimming
// ---------------------------------------------------------------------------
// A side-view fish — oval body, open eye, gill slit, dorsal fin and a forked
// tail — swims across with eased progress. `time` wags the tail, bobs the
// body and puffs bubbles from its mouth.

struct FishSwim;
impl ProgressStyle for FishSwim {
    fn name(&self) -> &str {
        "fish-swim"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A fish with eye, gill, dorsal fin and forked wagging tail swims across with progress, puffing bubbles"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let hf = h as f32;
        let rx = (hf * 0.55).max(3.0);
        let ry = (hf * 0.3).max(2.0);
        let tail = (rx * 0.7).round() as i32;
        let span = rx * 2.0 + tail as f32 + 4.0;
        let fx = rx + tail as f32 + 1.0 + ctx.eased * (w as f32 - span).max(0.0);
        let fy = hf * 0.5 + (ctx.time * 2.0).sin() * 1.0;
        let (cx, cy) = (fx.round() as i32, fy.round() as i32);
        let eye = (cx + (rx * 0.5) as i32, cy - (ry * 0.35).round() as i32);

        // Body: filled oval with a negative-space eye and gill slit.
        let rxi = rx as i32;
        for dx in -rxi..=rxi {
            let half = ry * (1.0 - (dx as f32 / rx).powi(2)).max(0.0).sqrt();
            let half = half.round() as i32;
            for dy in -half..=half {
                let (x, y) = (cx + dx, cy + dy);
                let in_eye = (x - eye.0).abs() <= 1
                    && (y - eye.1).abs() <= 1
                    && !(x == eye.0 + 1 && y == eye.1);
                let gill = dx == -1 + dy.abs() / 3 && dy.abs() < half - 1;
                let mouth = dx == rxi && dy == 1;
                if !(in_eye || gill || mouth) {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Pupil inside the eye ring.
        draw::dot_i(grid, eye.0 + 1, eye.1);

        // Dorsal fin: a swept triangle on the back.
        let top = cy - ry.round() as i32;
        for i in 0..5 {
            draw::vline(
                grid,
                (cx - 3 + i).max(0) as usize,
                (top - 3 + i / 2).max(0) as usize,
                top.max(0) as usize,
            );
        }

        // Forked tail: two lobes splaying from the wrist, wagging with time.
        let wag = (ctx.time * 7.0).sin() * 1.5;
        let wrist = cx - rxi;
        for t in 0..=tail {
            let k = t as f32 / tail.max(1) as f32;
            let off = (wag * k).round() as i32;
            let spread = (k * ry * 1.2).round() as i32;
            let x = wrist - t;
            for y in (cy - spread)..=(cy - spread / 3) {
                draw::dot_i(grid, x, y + off);
            }
            for y in (cy + spread / 3)..=(cy + spread) {
                draw::dot_i(grid, x, y + off);
            }
        }

        // Bubbles puffed from the mouth drift up and back.
        for i in 0..3 {
            let ph = (ctx.time * 0.8 + i as f32 / 3.0).fract();
            let bx = cx + rxi + 2 - (ph * 4.0) as i32;
            let by = cy - (ph * hf * 0.6) as i32;
            draw::dot_i(grid, bx, by);
            draw::dot_i(grid, bx + 1, by - 1);
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
        paint(grid, cx - rxi, top - 3, cx + rxi, cy + ry as i32, FISH);
        paint(grid, wrist - tail - 1, 0, wrist - 1, h as i32, FIN);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Wave Crest
// ---------------------------------------------------------------------------
// A tall sine wave rides across the grid; its amplitude and position track
// eased progress, with additional time-driven ripple.

struct WaveCrest;
impl ProgressStyle for WaveCrest {
    fn name(&self) -> &str {
        "wave-crest"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A rolling sine-wave crest advances with progress; fill trails behind"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let mid = h as f32 / 2.0;
        let amp = h as f32 * 0.4;
        // The crest center (in dot-x) tracks eased progress.
        let crest_center = (ctx.eased * w as f32) as i32;

        for x in 0..w {
            // Phase: combination of crest position and time scroll.
            let phase = (x as f32 - crest_center as f32) * 0.28 - ctx.time * 2.8;
            let envelope = {
                // Gaussian-ish envelope peaking at the crest, fading in front.
                let dist = (x as f32 - crest_center as f32) / w as f32 * 4.0;
                (-dist * dist).exp()
            };
            let wave_y = mid + phase.sin() * amp * envelope;
            let iy = wave_y.round() as i32;

            // Draw a vertical smear around the wave height for thickness.
            for dy in -1i32..=1 {
                draw::dot_i(grid, x as i32, iy + dy);
            }

            // Fill water below the wave for x < crest.
            if x < crest_center.max(0) as usize {
                let floor = (wave_y.round() as usize).min(h - 1);
                draw::vline(grid, x, floor, h - 1);
            }
        }

        // Palette tint: columns behind crest get deeper hue.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let filled_cx = (ctx.eased * ctx.width as f32) as usize;
            if filled_cx > 0 {
                let t = cy as f32 / cells_h.max(1) as f32;
                draw::tint_row(
                    grid,
                    cy,
                    0,
                    filled_cx.saturating_sub(1),
                    ctx.palette.sample(t),
                );
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Sonar Ping
// ---------------------------------------------------------------------------
// A little yellow submarine on the left sends sonar arcs rippling out from
// its bow; arc count tracks eased progress, `time` expands the rings.

struct SonarPing;
impl ProgressStyle for SonarPing {
    fn name(&self) -> &str {
        "sonar-ping"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A yellow submarine sends sonar arcs rippling out from its bow; ring count scales with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cy = (h / 2) as i32;
        let big = h >= 12 && w >= 30;
        let cx = if big { SUB[0].len() as i32 } else { 0 };
        // Number of active rings — progress controls how many appear.
        let max_rings = 6usize;
        let active_rings = ((ctx.eased * max_rings as f32).round() as usize).max(1);

        for ring_idx in 0..active_rings {
            // Each ring has an independent oscillating radius driven by time.
            let speed = 1.5 + ring_idx as f32 * 0.4;
            let phase = ctx.time * speed + ring_idx as f32 * 0.9;
            let radius = ((phase % (2.0 * PI)) / (2.0 * PI) * w as f32).round() as i32;

            // Draw a quarter-circle arc (right-facing semicircle from the origin).
            let steps = (radius * 3).max(8) as usize;
            for s in 0..steps {
                let angle = (s as f32 / steps as f32) * PI - PI / 2.0;
                let ax = cx + (angle.cos() * radius as f32).round() as i32;
                let ay = cy + (angle.sin() * radius as f32).round() as i32;
                draw::dot_i(grid, ax, ay);
            }
        }

        // Origin: the submarine itself, or a blip on tiny grids.
        if big {
            for (ry, row) in SUB.iter().enumerate() {
                for (rx, ch) in row.bytes().enumerate() {
                    if ch == b'#' {
                        draw::dot_i(grid, rx as i32, cy - 5 + ry as i32);
                    }
                }
            }
        } else {
            draw::dot_i(grid, cx, cy);
            draw::dot_i(grid, cx + 1, cy);
        }

        // Palette tint across rows.
        let (_, cells_h) = grid.dimensions();
        for cy_cell in 0..cells_h {
            let t = cy_cell as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t * 0.7),
            );
        }
        if big {
            paint(grid, 0, cy - 5, cx, cy + 3, SUB_HULL);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Depth Gauge
// ---------------------------------------------------------------------------
// A vertical depth gauge on the left fills downward with eased progress;
// tick marks label depth levels.

struct DepthGauge;
impl ProgressStyle for DepthGauge {
    fn name(&self) -> &str {
        "depth-gauge"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Submarine depth gauge: fills downward with depth markers and tint"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Left vertical spine.
        draw::vline(grid, 1, 0, h - 1);

        // Fill downward: submarine goes deeper with progress.
        let filled_h = (ctx.eased * h as f32).round() as usize;
        draw::fill_rect(grid, 2, 0, (w / 3).max(1), filled_h);

        // Tick marks every ~20% down the spine.
        for i in 0..=5usize {
            let tick_y = (i as f32 / 5.0 * (h - 1) as f32).round() as usize;
            let tick_len = if i % 5 == 0 { 5usize } else { 3 };
            draw::hline(grid, 0, tick_len.min(w - 1), tick_y);
        }

        // Animated pressure bubbles racing up the right side.
        for i in 0..4usize {
            let bspeed = 0.6 + i as f32 * 0.15;
            let bt = (ctx.time * bspeed + i as f32 * 0.7) % 1.0;
            let by = h.saturating_sub(1) - (bt * h as f32) as usize;
            let bx = (w as i32 - 3) + (((ctx.time * 2.0 + i as f32).sin()) * 1.5) as i32;
            draw::dot_i(grid, bx, by as i32);
            draw::dot_i(grid, bx + 1, by as i32);
        }

        // Depth-gradient tint: lighter near surface, darkening with depth.
        let (_, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, ctx.width.saturating_sub(1), color);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Jellyfish Pulse
// ---------------------------------------------------------------------------
// A domed jellyfish bell with a scalloped rim trails wavy tentacles and two
// frilly oral arms. `time` pulses the bell (squeeze = a little hop upward)
// and ripples the tentacles; `eased` drifts the jellyfish across the bar.

struct JellyfishPulse;
impl ProgressStyle for JellyfishPulse {
    fn name(&self) -> &str {
        "jellyfish-pulse"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A domed, pulsing jellyfish trailing wavy tentacles drifts across with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let hi = h as i32;
        let pulse = (ctx.time * 3.0).sin();
        let rxb = (h as f32 * 0.42 * (1.0 - 0.12 * pulse)).max(3.0);
        let bh = (h as f32 * 0.38).max(2.0) as i32;
        let jx = (rxb + 2.0 + ctx.eased * (w as f32 - 2.0 * rxb - 4.0).max(0.0)) as i32;
        let top = (1.0 + pulse.max(0.0)).round() as i32;
        let rim = top + bh;

        // Bell: a dome with a hollow ring pattern inside.
        for y in top..=rim {
            let k = (rim - y) as f32 / bh.max(1) as f32;
            let half = (rxb * (1.0 - k * k).max(0.0).sqrt()).round() as i32;
            for x in (jx - half)..=(jx + half) {
                let d = ((x - jx) as f32 / (rxb * 0.45)).powi(2)
                    + ((y - rim + 1) as f32 / (bh as f32 * 0.55)).powi(2);
                if !(0.75..1.1).contains(&d) {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Scalloped rim.
        let rxi = rxb as i32;
        for x in (jx - rxi)..=(jx + rxi) {
            if (x - jx).rem_euclid(3) != 0 {
                draw::dot_i(grid, x, rim + 1);
            }
        }

        // Tentacles ripple down to the bottom; two frilly oral arms in the middle.
        for i in 0..6 {
            let tx = jx - rxi + 1 + i * (2 * rxi - 2) / 5;
            let len = hi - rim - 2 - (i % 2) * 2;
            for d in 0..len {
                let y = rim + 2 + d;
                let x = tx
                    + ((y as f32 * 0.6 - ctx.time * 4.0 + i as f32).sin() * (1.0 + d as f32 * 0.08))
                        .round() as i32;
                if d % 4 != 3 || i % 2 == 0 {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        for side in [-1, 1] {
            for d in 0..(hi - rim - 4).max(0) {
                let y = rim + 2 + d;
                let x =
                    jx + side * 2 + ((y as f32 * 0.9 - ctx.time * 3.0).sin() * 1.2).round() as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + side, y);
            }
        }

        let (cells_w, cells_h) = grid.dimensions();
        for r in 0..cells_h {
            draw::tint_row(
                grid,
                r,
                0,
                cells_w.saturating_sub(1),
                ctx.palette.sample(0.4 + 0.6 * r as f32 / cells_h as f32),
            );
        }
        paint(grid, jx - rxi - 2, rim + 2, jx + rxi + 2, hi - 1, TENTACLE);
        paint(grid, jx - rxi, top, jx + rxi, rim + 1, JELLY);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Coral Reef
// ---------------------------------------------------------------------------
// A reef assembles left to right on a sandy bed as progress grows: branching
// staghorn, a grooved brain-coral dome, a fan coral and a cluster of tube
// coral repeat across the bar, each in its own colour. Corals grow up out of
// the sand as they arrive; `time` sways branch tips and fan edges.

struct CoralReef;
impl ProgressStyle for CoralReef {
    fn name(&self) -> &str {
        "coral-reef"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A colourful reef of staghorn, brain, fan and tube corals grows out of the sand left to right with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let bed = hi - 1;
        // Sandy bed: dotted, full width.
        for x in (0..wi).step_by(2) {
            draw::dot_i(grid, x, bed);
        }
        let slot = 12;
        let n = (wi / slot).max(1);
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
        for i in 0..n {
            let g = (ctx.eased * n as f32 - i as f32).clamp(0.0, 1.0);
            if g <= 0.0 {
                continue;
            }
            let x0 = i * slot + 1;
            let mid = x0 + slot / 2 - 1;
            let full = (hi - 2) as f32;
            let tall = (full * g).round() as i32;
            let sway = ((ctx.time * 1.5 + i as f32).sin() * 1.2).round() as i32;
            let kind = i % 4;
            match kind {
                0 => {
                    // Staghorn: a trunk forking twice into antler tips.
                    for d in 0..tall {
                        draw::dot_i(grid, mid, bed - 1 - d);
                    }
                    for (side, at) in [(-1, 0.3f32), (1, 0.5), (-1, 0.7)] {
                        let y0 = bed - 1 - (tall as f32 * at) as i32;
                        let arm = (tall as f32 * 0.6) as i32;
                        for d in 0..arm {
                            let x = mid + side * (d / 2 + 1) + if d > arm / 2 { sway } else { 0 };
                            draw::dot_i(grid, x, y0 - d);
                        }
                    }
                }
                1 => {
                    // Brain coral: a grooved dome.
                    let ry = (full * 0.6 * g).max(1.0);
                    let rx = (slot as f32 * 0.5).max(2.0);
                    for x in (mid - rx as i32)..=(mid + rx as i32) {
                        let k = (x - mid) as f32 / rx;
                        let top = bed - 1 - (ry * (1.0 - k * k).max(0.0).sqrt()) as i32;
                        for y in top..bed {
                            if ((x as f32 * 1.4).sin() + (y as f32 * 1.1).cos()).abs() > 0.35 {
                                draw::dot_i(grid, x, y);
                            }
                        }
                    }
                }
                2 => {
                    // Fan coral: ribs radiating from a stalk, joined by a mesh.
                    let base = (mid, bed - 1);
                    for rib in 0..7 {
                        let a = PI * (0.2 + rib as f32 * 0.1) + sway as f32 * 0.05;
                        for d in 0..tall {
                            let x = base.0 - (a.cos() * d as f32 * 0.9) as i32;
                            let y = base.1 - (a.sin() * d as f32) as i32;
                            if d % 3 != 2 || rib % 2 == 0 {
                                draw::dot_i(grid, x, y);
                            }
                        }
                    }
                }
                _ => {
                    // Tube coral: open-topped pipes of differing heights.
                    for (j, frac) in [0.7f32, 1.0, 0.55].iter().enumerate() {
                        let tx = mid - 4 + j as i32 * 3;
                        let th = (tall as f32 * frac) as i32;
                        for d in 0..th {
                            draw::dot_i(grid, tx, bed - 1 - d);
                            draw::dot_i(grid, tx + 2, bed - 1 - d);
                        }
                        draw::dot_i(grid, tx + 1, bed - 1);
                        draw::dot_i(grid, tx - 1 + sway.signum(), bed - th);
                    }
                }
            }
            paint(grid, x0, 0, x0 + slot - 2, bed - 1, CORALS[kind as usize]);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Seaweed
// ---------------------------------------------------------------------------
// Kelp sprouts from a sandy seafloor, one strand at a time from the left as
// progress grows. Each strand is a swaying stalk with long leaf blades
// alternating off each side and little air bladders; `time` sways the tips
// further than the roots.

struct Seaweed;
impl ProgressStyle for Seaweed {
    fn name(&self) -> &str {
        "seaweed"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Green kelp strands with leaf blades sprout from the sand left to right with progress and sway in the current"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let hi = h as i32;
        let floor = hi - 1;

        // Sandy floor, pebbled.
        for x in 0..w as i32 {
            if x % 3 != 1 {
                draw::dot_i(grid, x, floor);
            }
        }
        let (cells_w, cells_h) = grid.dimensions();
        for cy in 0..cells_h {
            let t = cy as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                cy,
                0,
                cells_w.saturating_sub(1),
                ctx.palette.sample(0.3 + t * 0.5),
            );
        }

        let gap = 7;
        let n = (w as i32 / gap).max(1);
        let grown = ctx.eased * n as f32;
        for fi in 0..n {
            let g = (grown - fi as f32).clamp(0.0, 1.0);
            if g <= 0.0 {
                continue;
            }
            let fx = fi * gap + gap / 2;
            let height_frac = 0.6 + (fi as f32 * 0.37) % 0.4;
            let frond_h = ((hi - 1) as f32 * height_frac * g).round() as i32;
            let freq = 1.2 + (fi as f32 * 0.31) % 0.8;
            for seg in 0..frond_h {
                let y = floor - 1 - seg;
                let amp = seg as f32 / frond_h.max(1) as f32 * 2.5;
                let x = fx
                    + ((ctx.time * freq + fi as f32 * 0.7 - seg as f32 * 0.25).sin() * amp).round()
                        as i32;
                draw::dot_i(grid, x, y);
                // Leaf blades angle up and out, alternating sides.
                if seg % 4 == 1 && seg + 2 < frond_h {
                    let side = if seg % 8 == 1 { 1 } else { -1 };
                    draw::dot_i(grid, x + side, y - 1);
                    draw::dot_i(grid, x + side * 2, y - 2);
                    draw::dot_i(grid, x + side * 2, y - 1);
                }
            }
            paint(
                grid,
                fx - 3,
                floor - frond_h,
                fx + 3,
                floor - 1,
                KELP[(fi % 2) as usize],
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Ripple Interference
// ---------------------------------------------------------------------------
// Two sinusoidal wave sources produce an interference pattern; source
// separation tracks eased progress.

struct RippleInterference;
impl ProgressStyle for RippleInterference {
    fn name(&self) -> &str {
        "ripple-interference"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "Two point sources create animated ripple interference patterns"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Source positions: start centered, spread apart with progress.
        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let spread = ctx.eased * (w as f32 * 0.35);
        let s1x = cx - spread;
        let s2x = cx + spread;

        for py in 0..h {
            for px in 0..w {
                let pxf = px as f32;
                let pyf = py as f32;
                // Distance from each source.
                let d1 = ((pxf - s1x).powi(2) + (pyf - cy).powi(2)).sqrt();
                let d2 = ((pxf - s2x).powi(2) + (pyf - cy).powi(2)).sqrt();
                // Superposition of two circular waves.
                let wave = (d1 * 0.6 - ctx.time * 4.0).sin() + (d2 * 0.6 - ctx.time * 4.0).sin();
                // Dot when combined amplitude exceeds threshold.
                if wave > 1.2 {
                    draw::dot(grid, px, py);
                }
            }
        }

        // Tint smoothly across both axes.
        let (_, cells_h) = grid.dimensions();
        for cy_cell in 0..cells_h {
            let t = cy_cell as f32 / cells_h.max(1) as f32;
            draw::tint_row(
                grid,
                cy_cell,
                0,
                ctx.width.saturating_sub(1),
                ctx.palette.sample(t),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Whale Spout
// ---------------------------------------------------------------------------
// A big whale cruises right along the surface with progress: rounded head,
// eye and mouth line, a pectoral flipper, a tapering tail stock ending in a
// wagging fluke, and a fountain spout that bursts from the blowhole every few
// seconds. Small waves roll beneath it.

struct WhaleSpout;
impl ProgressStyle for WhaleSpout {
    fn name(&self) -> &str {
        "whale-spout"
    }
    fn theme(&self) -> &str {
        "ocean"
    }
    fn describe(&self) -> &str {
        "A whale with flipper and wagging fluke swims right with progress, blowing a fountain spout"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 8 || h < 8 {
            draw::fill_rect(grid, 0, 0, (ctx.eased * w as f32) as usize, h);
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let rx = (h as f32 * 0.85).max(4.0);
        let ry = (h as f32 * 0.28).max(2.0);
        let stock = (rx * 0.7) as i32;
        let span = 2.0 * rx + stock as f32 + 6.0;
        let cx = (rx + stock as f32 + 5.0 + ctx.eased * (w as f32 - span).max(0.0)) as i32;
        let cy = hi - 1 - ry as i32 - ((ctx.time * 1.5).sin() * 0.6).round() as i32;
        let rxi = rx as i32;

        // Waves rolling beneath.
        for x in 0..wi {
            let y = hi - 1 - ((x as f32 * 0.4 - ctx.time * 3.0).sin() + 1.0).round() as i32 / 2;
            if x % 2 == 0 {
                draw::dot_i(grid, x, y);
            }
        }

        // Body: an egg shape, fuller toward the head, with an eye and mouth.
        let eye = (cx + rxi * 6 / 10, cy);
        for dx in -rxi..=rxi {
            let u = dx as f32 / rx;
            let top = (ry * (1.0 - u * u).max(0.0).sqrt() * (1.1 + 0.2 * u)).round() as i32;
            let bot = (ry * (1.0 - u * u).max(0.0).sqrt() * 0.9).round() as i32;
            for dy in -top..=bot {
                let (x, y) = (cx + dx, cy + dy);
                let mouth = dx > rxi / 3 && dy == 2 && dx < rxi;
                if !(mouth || (x == eye.0 && y == eye.1)) {
                    draw::dot_i(grid, x, y);
                }
            }
        }
        // Pectoral flipper sweeping back under the body.
        for i in 0..4 {
            draw::dot_i(grid, cx + rxi / 3 - i, cy + (ry * 0.9) as i32 + i / 2);
            draw::dot_i(grid, cx + rxi / 3 - i - 1, cy + (ry * 0.9) as i32 + i / 2);
        }

        // Tail stock tapering back, then a fluke that wags up and down.
        let wag = (ctx.time * 3.0).sin();
        let lift =
            |t: i32| (wag * t as f32 * t as f32 / (stock * stock).max(1) as f32 * 3.0) as i32;
        for t in 0..stock {
            let half = ((1.0 - t as f32 / stock as f32) * ry * 0.6).round() as i32;
            let x = cx - rxi - t;
            for dy in -half..=half {
                draw::dot_i(grid, x, cy - 1 + dy - lift(t));
            }
        }
        let fx = cx - rxi - stock;
        let fy = cy - 1 - lift(stock);
        for i in 0..4 {
            draw::vline(
                grid,
                (fx - i).max(0) as usize,
                (fy - 1 - i).max(0) as usize,
                (fy + 1 + i).max(0) as usize,
            );
        }

        // Spout: bursts from the blowhole for part of every cycle.
        let ph = (ctx.time * 0.5).fract();
        let blow = if ph < 0.5 { (ph / 0.5 * PI).sin() } else { 0.0 };
        let hole = (cx + rxi / 4, cy - (ry * 1.1) as i32 - 1);
        let tall = (blow * (hole.1 - 1).max(0) as f32) as i32;
        for d in 0..tall {
            draw::dot_i(grid, hole.0, hole.1 - d);
        }
        if tall > 1 {
            for i in 1..4 {
                let spread = i * 2;
                draw::dot_i(grid, hole.0 - spread, hole.1 - tall + i);
                draw::dot_i(grid, hole.0 + spread, hole.1 - tall + i);
            }
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
        paint(grid, fx - 4, cy - ry as i32 - 3, cx + rxi, hi - 2, WHALE);
        if tall > 1 {
            paint(grid, hole.0 - 6, 0, hole.0 + 6, hole.1, SPOUT);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Cell colours for the pictorial bars.
/// Side-view submarine: periscope, conning tower, hull with portholes.
const SUB: [&str; 9] = [
    "..........##......",
    "..........#.......",
    ".........####.....",
    ".........####.....",
    "...##############.",
    "..################",
    "#.##.##.##.##.####",
    "..################",
    "...#############..",
];
const SUB_HULL: Color = Color::rgb(245, 205, 60);
const KELP: [Color; 2] = [Color::rgb(90, 200, 90), Color::rgb(160, 210, 70)];
const WHALE: Color = Color::rgb(110, 150, 230);
const SPOUT: Color = Color::rgb(225, 240, 255);
const FISH: Color = Color::rgb(255, 140, 50);
const FIN: Color = Color::rgb(255, 200, 80);
const JELLY: Color = Color::rgb(240, 130, 220);
const TENTACLE: Color = Color::rgb(190, 160, 255);
const CORALS: [Color; 4] = [
    Color::rgb(255, 110, 150),
    Color::rgb(255, 170, 70),
    Color::rgb(180, 120, 255),
    Color::rgb(240, 220, 90),
];

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
