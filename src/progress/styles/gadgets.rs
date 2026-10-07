//! Gadgets / consumer-tech progress bars — device UIs in braille.
//!
//! Every style mimics a familiar consumer device interaction: phone charging,
//! WiFi connecting, Bluetooth pairing, disk defragging, CRT power-on, dial-up
//! modem handshake, a vinyl record spinning up, a quadcopter, an activity ring,
//! USB file transfer, a gear train, and an e-ink page refresh. All are
//! stateless — animation is driven entirely by `ctx.time` and `ctx.eased`.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ─── dot-art helpers ─────────────────────────────────────────────────────────

/// Stamp a sprite: every `#` in `rows` sets the dot at `(x + col, y + row)`.
fn sprite(grid: &mut BrailleGrid, x: i32, y: i32, rows: &[&str]) {
    for (r, line) in rows.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                draw::dot_i(grid, x + c as i32, y + r as i32);
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

// ─── tiny deterministic hash ─────────────────────────────────────────────────

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hashf(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

/// All styles in the `gadgets` theme.
///
/// Returns 12 structurally distinct consumer-device progress bar
/// implementations, each stateless and animatable via `ctx.time`.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(PhoneBattery),
        Box::new(WifiConnect),
        Box::new(BluetoothPair),
        Box::new(DiskDefrag),
        Box::new(CrtPowerOn),
        Box::new(DialUpModem),
        Box::new(VinylSpinUp),
        Box::new(DroneProps),
        Box::new(ActivityRing),
        Box::new(UsbTransfer),
        Box::new(GearTrain),
        Box::new(EinkRefresh),
    ]
}

// ─── 1. Smartphone battery charging ──────────────────────────────────────────

/// Battery outline that fills with lightning bolt and rising charge bubbles.
struct PhoneBattery;
impl ProgressStyle for PhoneBattery {
    fn name(&self) -> &str {
        "phone-battery"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Smartphone battery outline fills with charge; lightning bolt pulses"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Battery body outline — leave a 2-dot border.
        // "Terminal nub" on the right: a 2-dot-wide bump.
        let nub_w = (w / 16).max(1);
        let body_w = w.saturating_sub(nub_w + 2);
        let body_h = h.saturating_sub(2).max(1);
        let bx = 0usize;
        let by = 1usize;

        draw::rect_outline(grid, bx, by, body_w.max(2), body_h.max(2));

        // Nub
        let nub_y0 = by + body_h / 4;
        let nub_h = body_h / 2;
        draw::fill_rect(grid, bx + body_w, nub_y0, nub_w, nub_h.max(1));

        // Fill level inside the battery.
        let inner_w = body_w.saturating_sub(4).max(1);
        let inner_h = body_h.saturating_sub(4).max(1);
        let filled_w = ((ctx.eased * inner_w as f32) as usize).min(inner_w);
        if filled_w > 0 {
            draw::fill_rect(grid, bx + 2, by + 2, filled_w, inner_h);
        }

        // Lightning bolt: drawn with dots in the center when filling > 0.
        // Shape: top-right diagonal, then bottom-left diagonal.
        let bolt_x = (bx + body_w / 2) as i32;
        let bolt_mid = (by + body_h / 2) as i32;
        let bolt_h = inner_h as i32;
        let blink = (ctx.time * 2.0).fract() > 0.3 || ctx.eased > 0.5;
        if blink {
            // Top half of bolt: slants right.
            for dy in 0..bolt_h / 2 {
                let y = bolt_mid - dy as i32;
                let x = bolt_x + (dy as f32 * 0.6) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
            // Bottom half: slants left.
            for dy in 0..=bolt_h / 2 {
                let y = bolt_mid + dy as i32;
                let x = bolt_x - (dy as f32 * 0.6) as i32;
                draw::dot_i(grid, x, y);
                draw::dot_i(grid, x + 1, y);
            }
        }

        // Rising charge bubbles — dots that travel upward inside the fill.
        let n_bubbles = 4usize;
        for b in 0..n_bubbles {
            let phase = b as f32 / n_bubbles as f32;
            let t = (ctx.time * 0.5 + phase).fract();
            let bub_x = bx + 2 + (hashf(b as u32 * 7 + 3) * inner_w as f32) as usize;
            let bub_x = bub_x.min(bx + body_w.saturating_sub(3));
            let bub_y = by
                + 2
                + inner_h
                    .saturating_sub(1)
                    .saturating_sub((t * inner_h as f32) as usize);
            // Only show bubble in the filled zone.
            let fill_right = bx + 2 + filled_w;
            if bub_x < fill_right {
                draw::dot(grid, bub_x, bub_y);
                if bub_x + 1 < fill_right {
                    draw::dot(grid, bub_x + 1, bub_y);
                }
            }
        }

        // Tint filled region green→yellow by charge level.
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        for cx in 0..filled_cells {
            let t = cx as f32 / cells_w.max(1) as f32;
            let color = ctx.palette.sample(t);
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 2. WiFi connecting ───────────────────────────────────────────────────────

/// Concentric arcs pulse outward, then lock solid once connected.
struct WifiConnect;
impl ProgressStyle for WifiConnect {
    fn name(&self) -> &str {
        "wifi-connect"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "WiFi arcs pulse outward while connecting, then lock solid at 100%"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        // Origin: bottom-center, like a standard WiFi symbol.
        let cx_dot = w / 2;
        let cy_dot = h.saturating_sub(1);

        // Number of arcs lit by progress (0–4).
        let n_arcs = 4usize;
        let arcs_lit = (ctx.eased * n_arcs as f32).floor() as usize;
        // Connected: all 4 arcs lit.
        let connected = arcs_lit >= n_arcs;

        // Draw the base dot (the WiFi "device" dot).
        draw::dot(grid, cx_dot, cy_dot);
        if cx_dot + 1 < w {
            draw::dot(grid, cx_dot + 1, cy_dot);
        }

        for arc in 0..n_arcs {
            let radius = (arc + 1) as f32 * (h as f32 / (n_arcs + 1) as f32);
            let lit = arc < arcs_lit;

            // Pulsing alpha for unlocked arcs: sine pulse that propagates outward.
            let pulse_phase = (ctx.time * 1.5 - arc as f32 * 0.4).fract();
            let pulse = (pulse_phase * PI * 2.0).sin() * 0.5 + 0.5;
            let draw_it = lit || (!connected && pulse > 0.5);

            if draw_it {
                // Draw a semicircular arc (top half only — like WiFi symbol).
                // Sweep 180° (from left to right, convex upward).
                let steps = ((PI * radius) as usize).max(8);
                for step in 0..=steps {
                    let angle = PI * (step as f32 / steps as f32);
                    // angle 0 = right, PI = left; we want arcs opening downward.
                    let dx = (angle.cos() * radius) as i32;
                    let dy = -(angle.sin() * radius * 0.6) as i32; // flatten vertically
                    draw::dot_i(grid, cx_dot as i32 + dx, cy_dot as i32 + dy);
                    if !lit {
                        // Dashed: only draw alternating steps for the pulsing arcs.
                        // Already drawn above; skip extra for pulsing effect.
                    }
                }

                // Color: lit arcs get palette gradient.
                if lit {
                    let t = arc as f32 / n_arcs.max(1) as f32;
                    let color = ctx.palette.sample(t);
                    let cell_cx = cx_dot / 2;
                    let r_cells = (radius as usize / 2).max(1);
                    let cx0 = cell_cx.saturating_sub(r_cells);
                    let cx1 = (cell_cx + r_cells).min(cells_w.saturating_sub(1));
                    let cy_cell = cy_dot / 4;
                    let r_cell_h = (radius as usize / 4).max(1);
                    let cy0 = cy_cell.saturating_sub(r_cell_h);
                    for cy in cy0..cells_h {
                        draw::tint_row(grid, cy, cx0, cx1, color);
                    }
                }
            }
        }

        // "Locked" indicator: horizontal bar at bottom when connected.
        if connected {
            let blink_off = (ctx.time * 3.0).fract() < 0.2;
            if !blink_off {
                draw::hline(
                    grid,
                    cx_dot.saturating_sub(2),
                    (cx_dot + 2).min(w - 1),
                    cy_dot,
                );
                let full_color = ctx.palette.sample(1.0);
                draw::tint_row(
                    grid,
                    cells_h.saturating_sub(1),
                    (cells_w / 2).saturating_sub(1),
                    (cells_w / 2 + 1).min(cells_w - 1),
                    full_color,
                );
            }
        }

        Ok(())
    }
}

// ─── 3. Bluetooth pairing ─────────────────────────────────────────────────────

/// The ᛒ-like Bluetooth glyph with handshake blips radiating outward.
struct BluetoothPair;
impl ProgressStyle for BluetoothPair {
    fn name(&self) -> &str {
        "bluetooth-pair"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Bluetooth rune glyph at center; handshake blips pulse outward while pairing"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let cx = w / 2;
        let cy = h / 2;

        // ── Bluetooth glyph in dot space (approximated) ──
        // Vertical spine.
        let spine_h = (h * 3 / 4).max(2);
        let y0 = cy.saturating_sub(spine_h / 2);
        let y1 = (cy + spine_h / 2).min(h.saturating_sub(1));
        draw::vline(grid, cx, y0, y1);

        // Upper-right arm and return (forms the top lobe of ᛒ).
        let arm = (h / 5).max(1);
        // Upper-right diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (y0 + arm) as i32 - i as i32);
        }
        // Upper-right return diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (y0 + arm) as i32 + i as i32);
        }
        // Lower-right diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, cy as i32 + i as i32);
        }
        // Lower-right return diagonal.
        for i in 0..arm {
            draw::dot_i(grid, cx as i32 + i as i32, (cy + arm) as i32 - i as i32);
        }

        // ── Pairing signal arcs ──
        // Nested arcs flank the rune on both sides; the count grows with
        // progress (1 faint arc at the start, 5 when pairing is nearly done)
        // so completion stays legible in monochrome.
        let max_arcs = 5usize;
        let n_arcs = ((ctx.eased * max_arcs as f32).ceil() as usize).clamp(1, max_arcs);
        let arc_steps = 13usize;
        for k in 0..n_arcs {
            let r = (arm + 4 + k * 7) as f32;
            for s in 0..arc_steps {
                let a = (s as f32 / (arc_steps - 1) as f32 - 0.5) * 1.2; // ±0.6 rad
                let dx = (r * a.cos()).round() as i32;
                let dy = (r * 0.35 * a.sin()).round() as i32;
                draw::dot_i(grid, cx as i32 + dx, cy as i32 + dy);
                draw::dot_i(grid, cx as i32 - dx, cy as i32 + dy);
            }
        }

        // ── Handshake blips ──
        // Blips travel horizontally outward from the glyph. The 0.75 Hz rate
        // is a multiple of 0.25 Hz so the 4-second loop stays seamless.
        let n_blips = 5usize;
        let paired = ctx.eased >= 1.0;
        for b in 0..n_blips {
            let phase = b as f32 / n_blips as f32;
            let t = if paired {
                // Frozen in place when paired.
                1.0f32
            } else {
                (ctx.time * 0.75 + phase).fract()
            };
            let reach = (t * (w / 2) as f32) as usize;

            // Left blip.
            let lx = cx.saturating_sub(reach);
            draw::dot(grid, lx, cy);

            // Right blip.
            let rx = (cx + reach).min(w.saturating_sub(1));
            draw::dot(grid, rx, cy);
        }

        // Tint: progress drives gradient, center stays brightest.
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        let mid_cell = cells_w / 2;
        let reach_cells = (filled_cells / 2).max(1);
        let cx0 = mid_cell.saturating_sub(reach_cells);
        let cx1 = (mid_cell + reach_cells).min(cells_w.saturating_sub(1));
        for cx_c in cx0..=cx1 {
            let t = (cx_c.saturating_sub(cx0)) as f32 / (cx1.saturating_sub(cx0) + 1) as f32;
            let color = ctx.palette.sample(t);
            for cy_c in 0..cells_h {
                draw::tint_row(grid, cy_c, cx_c, cx_c, color);
            }
        }
        Ok(())
    }
}

// ─── 4. Disk defragmenter ────────────────────────────────────────────────────

/// The classic defrag cluster map: a grid of small cluster blocks. Behind the
/// sweep the used clusters are packed solid and contiguous; ahead of it the
/// disk is still fragmented (scattered hollow clusters with gaps). A blinking
/// read/write head marks the cluster being moved.
struct DiskDefrag;
impl ProgressStyle for DiskDefrag {
    fn name(&self) -> &str {
        "disk-defrag"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Disk defrag cluster map: scattered hollow fragments ahead of the sweep, packed solid clusters behind it, with a blinking read/write head"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let cols = (w / 4).max(1);
        let rows = (h / 4).max(1);
        let total = cols * rows;
        // Column-major order so the sweep runs left to right across all rows.
        let done = (ctx.eased * total as f32) as usize;
        let head_on = (ctx.time * 5.0).fract() < 0.5;
        for i in 0..total {
            let (col, row) = (i / rows, i % rows);
            let (x, y) = (col * 4, row * 4);
            if i < done {
                // Defragmented: 70% of the swept area is packed data.
                if (i as f32) < done as f32 * 0.7 + 0.5 {
                    draw::fill_rect(grid, x, y, 3, 3);
                }
            } else if i == done {
                if head_on {
                    draw::fill_rect(grid, x, y, 3, 3);
                } else {
                    draw::rect_outline(grid, x, y, 3, 3);
                }
            } else if hashf(i as u32 + 17) < 0.6 {
                draw::rect_outline(grid, x, y, 3, 3);
            }
        }
        Ok(())
    }
}

// ─── 5. CRT TV power-on ──────────────────────────────────────────────────────

/// A bright horizontal line at center expands vertically to fill the screen.
struct CrtPowerOn;
impl ProgressStyle for CrtPowerOn {
    fn name(&self) -> &str {
        "crt-power-on"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "CRT power-on: bright center line expands vertically to fill the screen"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = h / 2;
        // The beam starts as a single scanline and grows to full height.
        let beam_h = ((ctx.eased * h as f32) as usize).max(1).min(h);
        let y0 = mid.saturating_sub(beam_h / 2);
        let y1 = (y0 + beam_h).min(h);

        // Fill the beam area — horizontal scanlines.
        for y in y0..y1 {
            draw::hline(grid, 0, w.saturating_sub(1), y);
        }

        // Bright edge lines (phosphor glow at the expanding boundary).
        // The leading edges are extra dense (draw them twice — one line extra).
        if y0 > 0 {
            draw::hline(grid, 0, w.saturating_sub(1), y0.saturating_sub(1));
        }
        if y1 < h {
            draw::hline(grid, 0, w.saturating_sub(1), y1);
        }

        // Horizontal scanline "noise" inside the beam: a few dotted lines fade in.
        if beam_h > 4 {
            let noise_lines = beam_h / 4;
            for nl in 0..noise_lines {
                let y_nl = y0 + 2 + nl * 4;
                if y_nl >= y1 {
                    break;
                }
                // Sparse noise dots derived from hash.
                for x in 0..w {
                    let on = (hash((x as u32).wrapping_add(y_nl as u32 * 17)) % 5) == 0;
                    if on {
                        draw::dot(grid, x, y_nl);
                    }
                }
            }
        }

        // Tint: white-ish center → palette edge glow.
        let cy_center = cells_h / 2;
        let beam_cells = (beam_h / 4).max(1);
        let cy0 = cy_center.saturating_sub(beam_cells / 2);
        let cy1 = (cy0 + beam_cells).min(cells_h.saturating_sub(1));
        for cy in cy0..=cy1 {
            let t = (cy.saturating_sub(cy0)) as f32 / (cy1.saturating_sub(cy0) + 1).max(1) as f32;
            let color = ctx.palette.sample(t);
            draw::tint_row(grid, cy, 0, cells_w.saturating_sub(1), color);
        }
        Ok(())
    }
}

// ─── 6. Dial-up modem handshake ──────────────────────────────────────────────

/// Noisy oscillation across the bar gradually resolves to a steady carrier tone.
struct DialUpModem;
impl ProgressStyle for DialUpModem {
    fn name(&self) -> &str {
        "dialup-modem"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Dial-up modem: chaotic noise waveform settles to a clean carrier tone as progress rises"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (cells_w, cells_h) = grid.dimensions();

        let mid = (h / 2) as i32;
        let amp = (h / 2).saturating_sub(1) as f32;

        for x in 0..w {
            // "Clean" carrier: a smooth sine at this column.
            let carrier_phase = x as f32 / w as f32 * 4.0 * PI + ctx.time * 3.0;
            let clean_y = mid + (carrier_phase.sin() * amp * 0.8) as i32;

            // "Noisy" signal: carrier + random hash jitter.
            let epoch = (ctx.time * 8.0) as u32;
            let noise = hashf(x as u32 * 13 + epoch * 37) * 2.0 - 1.0;
            let noisy_y = mid + ((carrier_phase.sin() * 0.8 + noise * 1.5) * amp) as i32;

            // Blend: lerp from noisy (progress=0) → clean (progress=1).
            let y = (noisy_y as f32 * (1.0 - ctx.eased) + clean_y as f32 * ctx.eased) as i32;
            draw::dot_i(grid, x as i32, y);

            // Draw a second dot one above/below to fatten the trace.
            draw::dot_i(grid, x as i32, y + 1);
        }

        // Tint: chaotic region dim, settled region bright (left→right by progress).
        let filled_cells = ((ctx.eased * cells_w as f32) as usize).min(cells_w);
        for cx in 0..cells_w {
            let t = cx as f32 / cells_w.max(1) as f32;
            let is_settled = cx < filled_cells;
            let color = if is_settled {
                ctx.palette.sample(t)
            } else {
                ctx.palette.sample(0.05)
            };
            for cy in 0..cells_h {
                draw::tint_row(grid, cy, cx, cx, color);
            }
        }
        Ok(())
    }
}

// ─── 7. CD / Vinyl spinning up ────────────────────────────────────────────────

/// A round black record (grooves, paper label, spindle) spins on the platter,
/// its sheen gap and label mark turning faster as progress rises; the tonearm
/// tracks inward and a playback timeline beside it fills.
struct VinylSpinUp;
impl ProgressStyle for VinylSpinUp {
    fn name(&self) -> &str {
        "vinyl-spin"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Vinyl record: a grooved black disc with a paper label spins up on the turntable while the tonearm tracks inward and a playback timeline fills"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let r = ((hi as f32 - 1.0) / 2.0).clamp(0.5, 7.5);
        let (cx, cy) = (r + 1.0, (hi as f32 - 1.0) / 2.0);

        // The record: rim, concentric grooves broken by a turning sheen gap,
        // and a solid paper label with a spindle hole and one print mark.
        let spin = ctx.time * (1.5 + ctx.eased * 9.0);
        let (mx, my) = (cx + 1.5 * spin.cos(), cy + 1.5 * spin.sin());
        for y in 0..hi {
            for x in 0..(2.0 * r + 3.0) as i32 {
                let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                let d = (dx * dx + dy * dy).sqrt();
                let diff = (dy.atan2(dx) - spin).rem_euclid(PI);
                let gap = !(0.35..PI - 0.35).contains(&diff);
                let mark = (x as f32 - mx).abs() < 0.6 && (y as f32 - my).abs() < 0.6;
                let on = if d > r + 0.3 {
                    false
                } else if d > r - 0.7 {
                    true // rim
                } else if d < 0.7 {
                    false // spindle hole
                } else if d < 2.6 {
                    !mark // label
                } else {
                    let groove = (d - 2.6).rem_euclid(1.6) < 0.6;
                    groove && !gap
                };
                if on {
                    draw::dot_i(grid, x, y);
                }
            }
        }

        // Tonearm: pivot block, arm swinging the stylus inward with progress.
        let px = (cx + r + 5.0) as i32;
        draw::fill_rect(grid, (px - 1).max(0) as usize, 0, 3, 3);
        let reach = r - 0.5 - ctx.eased * (r - 3.5).max(0.0);
        let (sx, sy) = ((cx + reach * 0.7) as i32, (cy + reach * 0.7) as i32);
        let steps = 16;
        for k in 0..=steps {
            let t = k as f32 / steps as f32;
            draw::dot_i(
                grid,
                px + ((sx - px) as f32 * t).round() as i32,
                1 + ((sy - 1) as f32 * t).round() as i32,
            );
        }

        // Playback timeline: track line, elapsed fill and a round playhead.
        let x0 = px + 5;
        let span = (wi - x0 - 3).max(0);
        let mid = hi / 2;
        for x in x0..x0 + span {
            draw::dot_i(grid, x, mid);
        }
        let head = x0 + (ctx.eased * span as f32) as i32;
        for x in x0..head {
            draw::dot_i(grid, x, mid - 1);
            draw::dot_i(grid, x, mid + 1);
        }
        for (ox, oy) in [
            (0, -2),
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (0, 2),
        ] {
            draw::dot_i(grid, head + ox, mid + oy);
        }
        Ok(())
    }
}

// ─── 8. Drone quad-propellers ────────────────────────────────────────────────

/// Top-down quadcopter: a central body with a blinking nose LED, four arms
/// and four rotor rings whose twin blades spin faster as progress rises,
/// while the drone flies left to right.
struct DroneProps;
impl ProgressStyle for DroneProps {
    fn name(&self) -> &str {
        "drone-props"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Quadcopter seen from above: body, four arms and rotor rings whose blades spin up with progress as the drone flies across"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let cx = 14 + (ctx.eased * (wi - 29).max(0) as f32) as i32;
        let cy = hi / 2;
        let bob = i32::from((ctx.time * 2.0).fract() >= 0.5 && ctx.eased < 1.0);

        // Body: a rounded hull with a blinking nose LED on the leading edge.
        for (dx, dy) in (-3..=3_i32).flat_map(|x| (-2..=2_i32).map(move |y| (x, y))) {
            if dx.abs() + dy.abs() < 5 {
                draw::dot_i(grid, cx + dx, cy + dy + bob);
            }
        }
        if (ctx.time * 3.0).fract() < 0.5 {
            draw::dot_i(grid, cx + 5, cy + bob);
        }

        // Arms and rotors at the four corners.
        let spin = ctx.time * (3.0 + ctx.eased * 25.0);
        for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let (rx, ry) = (cx + sx * 10, cy + sy * 4 + bob);
            for k in 1..8 {
                draw::dot_i(grid, cx + sx * (k * 10 / 8), cy + sy * (k * 4 / 8) + bob);
            }
            // Rotor guard ring.
            for s in 0..16 {
                let a = s as f32 * PI / 8.0;
                draw::dot_i(
                    grid,
                    rx + (a.cos() * 3.5).round() as i32,
                    ry + (a.sin() * 3.5).round() as i32,
                );
            }
            // Twin blades, counter-rotating on diagonal pairs.
            let a = spin * (sx * sy) as f32;
            for k in -3..=3 {
                let t = k as f32;
                draw::dot_i(
                    grid,
                    rx + (a.cos() * t).round() as i32,
                    ry + (a.sin() * t * 0.9).round() as i32,
                );
            }
        }
        Ok(())
    }
}

// ─── 9. Smartwatch activity ring ─────────────────────────────────────────────

/// Three concentric round rings — red Move, green Exercise, blue Stand —
/// close clockwise from twelve o'clock over dim tracks, drawn as true circles
/// (braille dots are square). Matching summary bars sit to the right.
struct ActivityRing;

/// Ring colours: Move, Exercise, Stand.
const RING_COLORS: [Color; 3] = [
    Color::rgb(250, 40, 90),
    Color::rgb(150, 250, 40),
    Color::rgb(40, 220, 250),
];

impl ProgressStyle for ActivityRing {
    fn name(&self) -> &str {
        "activity-ring"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Smartwatch activity rings: concentric red, green and blue circles close clockwise as progress grows, with matching summary bars"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w == 0 || h == 0 {
            return Ok(());
        }
        let e = ctx.eased.clamp(0.0, 1.0);
        let vals = [e, e.powf(0.8), e.powf(1.3)];
        let r_out = (h.min(w) as f32 / 2.0).max(1.0);
        let (ccx, ccy) = (r_out, h as f32 / 2.0);
        let s = r_out / 8.0;
        let radii = [7.0 * s, 4.6 * s, 2.2 * s];
        let half = 0.95 * s;
        let box_w = (2.0 * r_out).ceil() as usize + 1;
        // Per-cell ring ownership: (ring, lit) with the strongest vote.
        let mut votes = vec![[0u8; 6]; cw * ch];
        for y in 0..h {
            for x in 0..box_w.min(w) {
                let dx = x as f32 + 0.5 - ccx;
                let dy = y as f32 + 0.5 - ccy;
                let d = (dx * dx + dy * dy).sqrt();
                for (k, &r) in radii.iter().enumerate() {
                    if (d - r).abs() > half {
                        continue;
                    }
                    let ang = dx.atan2(-dy).rem_euclid(2.0 * PI) / (2.0 * PI);
                    let lit = ang <= vals[k] && vals[k] > 0.0;
                    if lit || (x + y) % 2 == 0 {
                        draw::dot(grid, x, y);
                    }
                    let cell = (y / 4) * cw + x / 2;
                    if let Some(v) = votes.get_mut(cell) {
                        let slot = k * 2 + usize::from(lit);
                        v[slot] = v[slot].saturating_add(if lit { 2 } else { 1 });
                    }
                }
            }
        }
        // Summary bars to the right, one per ring.
        let bx0 = box_w + 3;
        let bar_w = w.saturating_sub(bx0 + 1);
        if bar_w > 2 {
            for (k, &v) in vals.iter().enumerate() {
                let y = (h * (2 * k + 1)) / 6;
                let fill = (v * bar_w as f32) as usize;
                for x in 0..bar_w {
                    if x < fill {
                        draw::dot(grid, bx0 + x, y);
                        if y + 1 < h {
                            draw::dot(grid, bx0 + x, y + 1);
                        }
                    } else if x % 3 == 0 {
                        draw::dot(grid, bx0 + x, y);
                    }
                }
            }
        }

        grid.enable_color_support();
        let dim = |c: Color| Color::rgb(c.r / 3, c.g / 3, c.b / 3);
        for cy in 0..ch {
            for cx in 0..cw {
                let px = cx * 2 + 1;
                let color = if px < box_w {
                    let v = votes[cy * cw + cx];
                    let (best, _) =
                        v.iter().enumerate().fold(
                            (0, 0u8),
                            |acc, (i, &n)| if n > acc.1 { (i, n) } else { acc },
                        );
                    let c = RING_COLORS[best / 2];
                    if best % 2 == 1 {
                        c
                    } else {
                        dim(c)
                    }
                } else if px >= bx0 && bar_w > 2 {
                    let k = ((cy * 4 + 2) * 3 / h.max(1)).min(2);
                    let fill = (vals[k] * bar_w as f32) as usize;
                    if px - bx0 < fill {
                        RING_COLORS[k]
                    } else {
                        dim(RING_COLORS[k])
                    }
                } else {
                    dim(RING_COLORS[0])
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 10. USB file transfer ────────────────────────────────────────────────────

/// The USB trident logo on the left points at a flash drive on the right;
/// little file packets (dog-eared pages) stream along into the drive's
/// metal plug, the drive's storage fills with progress and its LED blinks.
struct UsbTransfer;
impl ProgressStyle for UsbTransfer {
    fn name(&self) -> &str {
        "usb-transfer"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "USB transfer: file packets stream from the USB trident logo into a flash drive's plug; the drive fills with progress, LED blinking"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        let (wi, hi) = (w as i32, h as i32);
        let t = ctx.time;
        let my = hi / 2;

        // USB trident logo: base circle, shaft, arrowhead, two branches.
        sprite(grid, 0, my - 2, &[".##.", "#..#", "#..#", ".##."]);
        for x in 4..20 {
            draw::dot_i(grid, x, my);
        }
        sprite(grid, 20, my - 2, &["#..", "##.", "###", "##.", "#.."]);
        for k in 0..4 {
            draw::dot_i(grid, 7 + k, my - 1 - k);
            draw::dot_i(grid, 10 + k, my + 1 + k);
        }
        for x in 11..14 {
            draw::dot_i(grid, x, my - 4);
        }
        sprite(grid, 14, my - 5, &[".##.", "#..#", ".##."]);
        for x in 14..16 {
            draw::dot_i(grid, x, my + 4);
        }
        sprite(grid, 16, my + 3, &["###", "###", "###"]);

        // Flash drive: plug (with two contact holes) + body.
        let body_w = 24.min(wi / 3).max(6);
        let bx = wi - body_w;
        let px = bx - 9;
        sprite(
            grid,
            px,
            my - 4,
            &[
                "#########",
                "#........",
                "#.##..##.",
                "#.##..##.",
                "#........",
                "#........",
                "#........",
                "#########",
            ],
        );
        draw::rect_outline(
            grid,
            bx.max(0) as usize,
            (my - 7).max(0) as usize,
            body_w.max(1) as usize - 1,
            (hi - 2).max(1) as usize,
        );
        // Stored data fills the body.
        let inner = (body_w - 6).max(0);
        let fill = (ctx.eased * inner as f32).round() as i32;
        for x in 0..fill {
            for y in my - 4..=my + 4 {
                if (x + y) % 2 == 0 || x % 3 == 0 {
                    draw::dot_i(grid, bx + 2 + x, y);
                }
            }
        }
        // Activity LED near the end.
        if (t * 6.0).fract() < 0.5 {
            block(grid, wi - 3, my - 1, 2, 2);
        }

        // File packets streaming from the arrowhead into the plug.
        let (x0, x1) = (25, px - 1);
        let span = (x1 - x0).max(1);
        for i in 0..3 {
            let s = (t * 0.9 + i as f32 / 3.0).fract();
            let x = x0 + (s * span as f32) as i32;
            if x + 4 > x1 {
                continue;
            }
            sprite(grid, x, my - 3, &["##.", "###", "###", "###"]);
        }
        for x in (x0..x1).step_by(3) {
            draw::dot_i(grid, x, my + 3);
        }

        grid.enable_color_support();
        for cy in 0..ch {
            for cx in 0..cw {
                let qx = cx as i32 * 2 + 1;
                let color = if qx < 24 {
                    Color::rgb(220, 226, 236) // logo
                } else if qx < px - 1 {
                    Color::rgb(110, 250, 150) // packets
                } else if qx < bx {
                    Color::rgb(196, 200, 210) // metal plug
                } else if qx >= wi - 4 {
                    Color::rgb(80, 255, 120) // LED
                } else {
                    Color::rgb(60, 120, 250)
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}

// ─── 11. Gear train ──────────────────────────────────────────────────────────

/// A row of round toothed gears, alternating large and small, each meshed with
/// its neighbour. Progress engages the train gear by gear: engaged gears turn
/// (neighbours counter-rotating, small ones faster), idle gears sit still.
struct GearTrain;
impl ProgressStyle for GearTrain {
    fn name(&self) -> &str {
        "gear-train"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "Gear train: a row of meshed toothed gears, large and small; progress engages them one by one (idle gears ghosted) and engaged gears counter-rotate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let cy = (h as f32 - 1.0) / 2.0;
        let big = (cy - 1.2).clamp(1.0, 6.3);
        let small = (big * 0.6).max(1.0);
        let pitch = big + small + 1.6;
        let n = (((w as f32 - 2.0 * big - 2.0) / pitch) as usize + 1).max(1);
        let engaged = (ctx.eased * n as f32).ceil() as usize;

        for i in 0..n {
            let r = if i % 2 == 0 { big } else { small };
            let teeth = if i % 2 == 0 { 10.0 } else { 6.0 };
            let cx = big + 1.0 + i as f32 * pitch;
            // Meshed gears share surface speed: ω ∝ 1/r, alternating sign.
            let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rot = if i < engaged {
                ctx.time * 3.0 * dir / r
            } else {
                0.0
            };
            // Idle gears are ghosted (every third dot) so engagement reads.
            let samples = (r * 14.0) as usize;
            let stride = if i < engaged { 1 } else { 3 };
            for s in (0..samples).step_by(stride) {
                let a = s as f32 / samples as f32 * 2.0 * PI;
                let tooth = ((a - rot) * teeth / (2.0 * PI)).rem_euclid(1.0) < 0.5;
                let rr = if tooth { r + 1.2 } else { r };
                draw::dot_i(
                    grid,
                    (cx + a.cos() * rr).round() as i32,
                    (cy + a.sin() * rr).round() as i32,
                );
            }
            // Hub with a spoke mark so rotation reads.
            draw::dot_i(grid, cx.round() as i32, cy.round() as i32);
            let (sx, sy) = (cx + rot.cos() * r * 0.55, cy + rot.sin() * r * 0.55);
            draw::dot_i(grid, sx.round() as i32, sy.round() as i32);
        }
        Ok(())
    }
}

// ─── 12. E-ink page refresh ───────────────────────────────────────────────────

/// An e-reader: bezel with page-turn buttons around a paper-white screen of
/// justified text lines. Every page turn the whole screen flashes black, then
/// blank, then the new page's text draws in. A location bar at the bottom of
/// the screen fills with progress.
struct EinkRefresh;
impl ProgressStyle for EinkRefresh {
    fn name(&self) -> &str {
        "eink-refresh"
    }
    fn theme(&self) -> &str {
        "gadgets"
    }
    fn describe(&self) -> &str {
        "E-reader page turn: the screen flashes full black, then white, then a new page of text lines appears; the location bar fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if w < 4 || h < 4 {
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        // Bezel, with page-turn buttons on the right.
        draw::rect_outline(grid, 0, 0, w, h);
        let sx1 = (wi - 6).max(2); // screen right edge (exclusive)
        for y in 1..hi - 1 {
            draw::dot_i(grid, sx1, y);
        }
        for y in [hi / 2 - 4, hi / 2 + 1] {
            for k in 0..3 {
                draw::dot_i(grid, sx1 + 2, y + k);
                draw::dot_i(grid, sx1 + 3, y + k);
            }
        }

        let period = 1.6;
        let page = (ctx.time / period) as u32;
        let u = (ctx.time / period).fract();
        let (x0, x1, y0, y1) = (2, sx1 - 1, 2, hi - 3);
        if u < 0.08 {
            // Full black flash.
            for y in 1..hi - 1 {
                for x in 1..sx1 {
                    draw::dot_i(grid, x, y);
                }
            }
        } else if u >= 0.16 {
            // Text lines revealed top to bottom.
            let reveal = ((u - 0.16) / 0.2).min(1.0);
            let rows = ((y1 - y0) / 2).max(0);
            let shown = (reveal * rows as f32).ceil() as i32;
            for r in 0..shown {
                let y = y0 + r * 2;
                if y >= y1 - 1 {
                    break;
                }
                // Words of varying length; last line of a paragraph short.
                let end = if r % 4 == 3 { x0 + (x1 - x0) / 2 } else { x1 };
                let mut x = x0 + if r % 4 == 0 { 3 } else { 0 };
                let mut k = 0u32;
                while x < end {
                    let len = 2 + (hash(page * 977 + r as u32 * 31 + k) % 6) as i32;
                    for d in 0..len.min(end - x) {
                        draw::dot_i(grid, x + d, y);
                    }
                    x += len + 2;
                    k += 1;
                }
            }
        }
        // Location bar along the bottom of the screen.
        let fill = x0 + (ctx.eased * (x1 - x0) as f32) as i32;
        for x in x0..x1 {
            if x <= fill || x % 2 == 0 {
                draw::dot_i(grid, x, hi - 2);
            }
        }

        grid.enable_color_support();
        let flash = u < 0.08;
        for cy in 0..ch {
            for cx in 0..cw {
                let qx = cx as i32 * 2 + 1;
                let color = if qx >= sx1 {
                    Color::rgb(130, 132, 140) // bezel
                } else if flash {
                    Color::rgb(70, 70, 74)
                } else {
                    Color::rgb(232, 228, 214) // paper white
                };
                let _ = grid.set_cell_color(cx, cy, color);
            }
        }
        Ok(())
    }
}
