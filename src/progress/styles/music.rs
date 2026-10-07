//! Music-themed progress bars for dotmax.
//!
//! Every style in this theme derives its visual structure from a distinct
//! musical object or phenomenon — not just a colour change.
//!
//! - [`MusicalStaff`]     — 5-line staff with a treble clef; stemmed notes are written L→R
//! - [`PianoKeyboard`]    — keyboard with 2+3 black-key groups; keys light L→R
//! - [`Metronome`]        — pyramid metronome swinging; beat ticks light up beside it
//! - [`VinylRecord`]      — turntable; spinning grooved record, stylus tracks inward
//! - [`DrumKit`]          — drum kit assembling piece by piece while a groove plays
//! - [`SoundWaveform`]    — centered oscillating amplitude trace
//! - [`CassetteReels`]    — cassette shell; spoked reels wind tape left → right
//! - [`ConductorBaton`]   — conductor beating 4/4 with a baton; notes stream out
//! - [`SheetScroll`]      — paper scroll between rollers; notes slide, left roll fattens
//! - [`TuningFork`]       — buzzing tuning fork; sound-wave arcs ripple further out
//! - [`VolumeKnob`]       — amp knob turning up its tick scale; stereo LED meter
//! - [`BeatPulse`]        — concentric rings expanding on each beat
//! - [`AcousticGuitar`]   — fretted guitar; finger slides up the neck, strings ring
//! - [`Trumpet`]          — trumpet in profile; valves press, brass lights to the bell

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Theme tint — stage magenta into cyan. Applied to styles that draw monochrome.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(255, 104, 196);
const TINT_END: Color = Color::rgb(84, 204, 255);

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

/// All styles in the `music` theme.
///
/// Returns 14 structurally distinct bars, each modelling a different concept
/// from the world of music — notation, instruments, rhythm, and acoustics.
/// Safe to render at any size from 1×1 upward.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(MusicalStaff),
        Box::new(PianoKeyboard),
        Box::new(Metronome),
        Box::new(VinylRecord),
        Box::new(DrumKit),
        Box::new(Tinted(SoundWaveform)),
        Box::new(CassetteReels),
        Box::new(ConductorBaton),
        Box::new(SheetScroll),
        Box::new(TuningFork),
        Box::new(VolumeKnob),
        Box::new(BeatPulse),
        Box::new(AcousticGuitar),
        Box::new(Trumpet),
    ]
}

// ---------------------------------------------------------------------------
// Pictorial helpers — a scaled "pen" over a fixed design canvas, so the
// instrument drawings keep their proportions at any grid size. Every write
// goes through `draw::dot_i`, so off-grid geometry is silently clipped.
// ---------------------------------------------------------------------------

/// Maps design units onto dot space: `dot = origin + unit * s`.
#[derive(Clone, Copy)]
struct Pen {
    ox: f32,
    oy: f32,
    s: f32,
}

impl Pen {
    const fn new(ox: f32, oy: f32, s: f32) -> Self {
        Self { ox, oy, s }
    }

    fn map(&self, x: f32, y: f32) -> (f32, f32) {
        (self.ox + x * self.s, self.oy + y * self.s)
    }

    /// Straight stroke between two design points.
    fn line(&self, g: &mut BrailleGrid, x0: f32, y0: f32, x1: f32, y1: f32) {
        let (a, b) = self.map(x0, y0);
        let (c, d) = self.map(x1, y1);
        let n = (c - a).abs().max((d - b).abs()).ceil().clamp(1.0, 4096.0) as i32;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            draw::dot_i(
                g,
                (a + (c - a) * t).round() as i32,
                (b + (d - b) * t).round() as i32,
            );
        }
    }

    /// Solid box `[x, x+w) × [y, y+h)` in design units (at least one dot).
    fn rect(&self, g: &mut BrailleGrid, x: f32, y: f32, w: f32, h: f32) {
        let (a, b) = self.map(x, y);
        let (c, d) = self.map(x + w, y + h);
        let (x0, y0) = (a.round() as i32, b.round() as i32);
        let (x1, y1) = (
            (c.round() as i32).max(x0 + 1),
            (d.round() as i32).max(y0 + 1),
        );
        for py in y0.max(0)..y1.min(4096) {
            for px in x0.max(0)..x1.min(4096) {
                draw::dot_i(g, px, py);
            }
        }
    }

    /// Ellipse centred on a design point; `fill == false` draws a one-dot rim.
    fn ellipse(&self, g: &mut BrailleGrid, cx: f32, cy: f32, rx: f32, ry: f32, fill: bool) {
        let (pcx, pcy) = self.map(cx, cy);
        let (rx, ry) = (
            (rx * self.s).clamp(0.5, 2048.0),
            (ry * self.s).clamp(0.5, 2048.0),
        );
        let (x0, x1) = ((pcx - rx).floor() as i32, (pcx + rx).ceil() as i32);
        let (y0, y1) = ((pcy - ry).floor() as i32, (pcy + ry).ceil() as i32);
        for py in y0.max(0)..=y1.min(4096) {
            for px in x0.max(0)..=x1.min(4096) {
                let (dx, dy) = (px as f32 - pcx, py as f32 - pcy);
                let outer = (dx / rx).powi(2) + (dy / ry).powi(2) <= 1.0;
                let inner = rx > 1.2
                    && ry > 1.2
                    && (dx / (rx - 1.0)).powi(2) + (dy / (ry - 1.0)).powi(2) <= 1.0;
                if outer && (fill || !inner) {
                    draw::dot_i(g, px, py);
                }
            }
        }
    }

    /// Colour every cell overlapping the design box.
    fn tint(&self, g: &mut BrailleGrid, x: f32, y: f32, w: f32, h: f32, col: Color) {
        g.enable_color_support();
        let (a, b) = self.map(x, y);
        let (c, d) = self.map(x + w, y + h);
        let (cw, ch) = g.dimensions();
        let cell = |v: f32, step: f32| (v.max(0.0) / step) as usize;
        let (x0, x1) = (cell(a, 2.0), cell(c - 1.0, 2.0).min(cw.saturating_sub(1)));
        let (y0, y1) = (cell(b, 4.0), cell(d - 1.0, 4.0).min(ch.saturating_sub(1)));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let _ = g.set_cell_color(cx, cy, col);
            }
        }
    }

    /// Colour the single cell under a design point.
    fn tint_at(&self, g: &mut BrailleGrid, x: f32, y: f32, col: Color) {
        self.tint(g, x, y, 0.0, 0.0, col);
    }
}

/// An eighth note: oval head, stem and flag, head centred on `(x, y)`.
/// `up` picks the stem side, as in engraved music.
fn eighth_note(pen: &Pen, g: &mut BrailleGrid, x: f32, y: f32, up: bool) {
    pen.ellipse(g, x, y, 1.7, 1.2, true);
    if up {
        pen.line(g, x + 1.5, y, x + 1.5, y - 6.0);
        pen.line(g, x + 1.5, y - 6.0, x + 3.5, y - 3.5);
    } else {
        pen.line(g, x - 1.5, y, x - 1.5, y + 6.0);
        pen.line(g, x - 1.5, y + 6.0, x + 0.5, y + 3.5);
    }
}

// ---------------------------------------------------------------------------
// 1. Musical Staff
//    Five staff lines with a dot-drawn treble clef at the left. Engraved
//    note heads with stems are written onto the staff left-to-right as
//    progress grows, and a playhead colour sweeps over the written notes.
// ---------------------------------------------------------------------------
struct MusicalStaff;

/// Treble clef bitmap, one design unit per `#`, sized for a 16-unit staff.
const TREBLE_CLEF: [&str; 16] = [
    "....##..", "...#..#.", "...#.#..", "....#...", "...##...", "..#.#...", ".#..#...", "#..###..",
    "#.#.#.#.", "#.#.#..#", "#..##..#", ".#..#.#.", "..####..", "....#...", ".#..#...", "..##....",
];

impl ProgressStyle for MusicalStaff {
    fn name(&self) -> &str {
        "musical-staff"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Five-line staff with a treble clef; stemmed notes are written left-to-right as progress grows while a playhead lights them in turn"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = dh as f32 / 16.0;
        let pen = Pen::new(0.0, 0.0, s);
        let units_w = dw as f32 / s;

        // Staff lines on design rows 2, 5, 8, 11, 14.
        for i in 0..5 {
            pen.line(
                grid,
                0.0,
                2.0 + 3.0 * i as f32,
                units_w,
                2.0 + 3.0 * i as f32,
            );
        }
        pen.tint(grid, 0.0, 0.0, units_w, 16.0, Color::rgb(90, 100, 140));

        // Treble clef.
        for (y, row) in TREBLE_CLEF.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == '#' {
                    pen.rect(grid, x as f32, y as f32, 1.0, 1.0);
                }
            }
        }
        pen.tint(grid, 0.0, 0.0, 8.0, 16.0, Color::rgb(240, 200, 90));

        // Notes: one every 7 units after the clef; pitch in staff half-steps
        // (0 = bottom line, 8 = top line).
        let melody = [2usize, 4, 6, 4, 5, 7, 8, 6, 3, 5, 4, 2, 1, 3, 5, 7];
        let slots = ((units_w - 14.0) / 7.0).max(0.0) as usize;
        let shown = ((ctx.eased * slots as f32).ceil() as usize).min(slots);
        let playing = if shown > 0 {
            (ctx.time * 4.0).max(0.0) as usize % shown
        } else {
            0
        };
        for i in 0..shown {
            let step = melody[i % melody.len()];
            let x = 13.0 + 7.0 * i as f32;
            let y = 14.0 - 1.5 * step as f32;
            eighth_note(&pen, grid, x, y, step < 4);
            let col = if i == playing {
                Color::rgb(255, 104, 196)
            } else {
                Color::rgb(235, 230, 250)
            };
            pen.tint(grid, x - 2.0, y - 6.5, 6.0, 13.0, col);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Piano Keyboard
//    A real keyboard layout: ivory white keys (two cells each) with short
//    black keys grouped in the 2-and-3 pattern of every octave. Keys left of
//    the progress head glow in the theme gradient; the key at the head is
//    held down, and a melody presses keys among the played ones on `time`.
// ---------------------------------------------------------------------------
struct PianoKeyboard;
impl ProgressStyle for PianoKeyboard {
    fn name(&self) -> &str {
        "piano-keyboard"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Piano keyboard with black keys in groups of two and three; keys light up left-to-right with progress while a melody presses them"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        let (cw, ch) = grid.dimensions();
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        grid.enable_color_support();

        // White keys are 3 dots + a 1-dot gap; black keys cover the top
        // ~60% (whole cell rows when there is room) of the cell straddling
        // the gap after C, D, F, G and A.
        let whites = (dw + 3) / 4;
        let black_h = if ch >= 2 {
            ((dh * 5 / 8) / 4 * 4).max(4)
        } else {
            dh / 2
        };
        let head = ((ctx.eased * whites as f32) as usize).min(whites);
        let beat = (ctx.time * 3.0).max(0.0) as usize;
        let tune = [0usize, 2, 4, 2, 5, 4, 2, 0, 4, 5, 7, 5];
        let melody = (head > 1).then(|| (tune[beat % tune.len()] + beat / 12 * 3) % head);

        for k in 0..whites {
            let x = k * 4;
            let held = k == head || Some(k) == melody;
            let top = usize::from(held && dh > 2);
            draw::fill_rect(grid, x, top, 3, dh - top);
            let col = if k == head {
                Color::rgb(255, 255, 255)
            } else if Some(k) == melody {
                Color::rgb(255, 230, 120)
            } else if k < head {
                sample_tint(k as f32 / whites.max(1) as f32)
            } else {
                Color::rgb(232, 228, 214)
            };
            for cy in 0..ch {
                for cx in [x / 2, x / 2 + 1] {
                    if cx < cw {
                        let _ = grid.set_cell_color(cx, cy, col);
                    }
                }
            }
        }
        // Black keys drawn last so their cells take the dark colour.
        for k in 0..whites {
            if matches!(k % 7, 0 | 1 | 3 | 4 | 5) && k + 1 < whites {
                let x = k * 4 + 2;
                draw::fill_rect(grid, x, 0, 2, black_h);
                for cy in 0..(black_h / 4).max(1) {
                    let _ = grid.set_cell_color(x / 2, cy, Color::rgb(78, 70, 110));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Metronome
//    A wooden pyramid metronome at the left: tapered case, plinth, and a
//    pendulum rod with a sliding weight swinging at 120 BPM. To its right a
//    row of beat ticks lights up with progress (downbeats taller), and the
//    newest tick flashes on every swing.
// ---------------------------------------------------------------------------
struct Metronome;
impl ProgressStyle for Metronome {
    fn name(&self) -> &str {
        "metronome"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Wooden pyramid metronome swinging its weighted pendulum; beat ticks light up to its right as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 40.0);
        let pen = Pen::new(0.0, dh as f32 - 16.0 * s, s);

        // Case: tapered sides, cap and a solid plinth.
        pen.line(grid, 1.5, 12.0, 5.5, 0.0);
        pen.line(grid, 14.5, 12.0, 10.5, 0.0);
        pen.line(grid, 5.5, 0.0, 10.5, 0.0);
        pen.rect(grid, 0.0, 12.0, 16.0, 4.0);
        pen.tint(grid, 0.0, 0.0, 16.0, 16.0, Color::rgb(185, 115, 60));

        // Pendulum: pivots just above the plinth; swings ±0.45 rad, one beat
        // per half-period (120 BPM).
        let angle = (ctx.time * PI * 2.0).sin() * 0.45;
        let (sa, ca) = angle.sin_cos();
        let (px, py) = (8.0, 11.5);
        let tip = (px + sa * 11.0, py - ca * 11.0);
        pen.line(grid, px, py, tip.0, tip.1);
        let wt = (px + sa * 7.5, py - ca * 7.5);
        pen.rect(grid, wt.0 - 1.5, wt.1 - 1.0, 3.0, 2.0);
        for k in 0..=6 {
            let f = k as f32 / 6.0;
            pen.tint_at(
                grid,
                px + (tip.0 - px) * f,
                py + (tip.1 - py) * f,
                Color::rgb(215, 220, 232),
            );
        }
        pen.tint_at(grid, wt.0, wt.1, Color::rgb(245, 200, 80));

        // Beat ticks: one per two cells, 4/4 downbeats taller.
        let x0 = (18.0 * s).ceil() as usize + 2;
        let n = dw.saturating_sub(x0) / 4;
        let lit = ((ctx.eased * n as f32).floor() as usize).min(n);
        let flash = (ctx.time * 2.0).fract() < 0.25;
        let mid = dh / 2;
        for i in 0..n {
            let x = x0 + i * 4;
            let half = if i % 4 == 0 { dh / 3 } else { dh / 6 };
            let cell = (x / 2, mid / 4);
            if i < lit {
                draw::vline(grid, x, mid.saturating_sub(half), (mid + half).min(dh - 1));
                draw::vline(
                    grid,
                    x + 1,
                    mid.saturating_sub(half),
                    (mid + half).min(dh - 1),
                );
                let col = if i + 1 == lit && flash {
                    Color::rgb(255, 255, 255)
                } else if i % 4 == 0 {
                    Color::rgb(245, 200, 80)
                } else {
                    sample_tint(i as f32 / n.max(1) as f32)
                };
                for cy in 0..(dh + 3) / 4 {
                    let _ = grid.set_cell_color(cell.0, cy, col);
                }
            } else {
                draw::dot(grid, x, mid);
                let _ = grid.set_cell_color(cell.0, cell.1, Color::rgb(70, 70, 90));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Vinyl Record
//    A turntable from above: plinth with a pitch slider, a grooved record
//    with a red centre label whose notch turns as the platter spins, and a
//    tonearm (pivot, counterweight, headshell) whose stylus tracks inward
//    with progress. Grooves already played dim behind the stylus.
// ---------------------------------------------------------------------------
struct VinylRecord;
impl ProgressStyle for VinylRecord {
    fn name(&self) -> &str {
        "vinyl-record"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Turntable with a spinning grooved record and red label; the tonearm's stylus tracks inward and played grooves dim with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 38.0);
        let pen = Pen::new((dw as f32 - 36.0 * s) / 2.0, dh as f32 - 16.0 * s, s);

        // Plinth outline and pitch slider.
        pen.line(grid, 1.0, 0.0, 35.0, 0.0);
        pen.line(grid, 1.0, 15.0, 35.0, 15.0);
        pen.line(grid, 0.0, 1.0, 0.0, 14.0);
        pen.line(grid, 36.0, 1.0, 36.0, 14.0);
        pen.line(grid, 33.0, 7.0, 33.0, 13.0);
        pen.rect(grid, 32.0, 9.0, 3.0, 1.0);
        pen.tint(grid, 0.0, 0.0, 37.0, 16.0, Color::rgb(140, 142, 165));

        // Record: groove stripes, played band dims from the rim inward.
        let (rcx, rcy) = (9.5, 8.0);
        let stylus_r = 6.6 - 4.0 * ctx.eased;
        let (pcx, pcy) = pen.map(rcx, rcy);
        let rim = 7.0 * s;
        for py in (pcy - rim).floor().max(0.0) as i32..=(pcy + rim).ceil().min(4096.0) as i32 {
            for px in (pcx - rim).floor().max(0.0) as i32..=(pcx + rim).ceil().min(4096.0) as i32 {
                let d = ((px as f32 - pcx).powi(2) + (py as f32 - pcy).powi(2)).sqrt() / s;
                if d <= 7.0 && d > 2.6 && (d * 1.4).fract() < 0.6 {
                    draw::dot_i(grid, px, py);
                }
            }
        }
        let (cw, ch) = grid.dimensions();
        for cy in 0..ch {
            for cx in 0..cw {
                let (x, y) = ((cx * 2) as f32 + 1.0, (cy * 4) as f32 + 2.0);
                let d = ((x - pcx).powi(2) + (y - pcy).powi(2)).sqrt() / s;
                if d <= 7.5 {
                    let col = if d > stylus_r {
                        Color::rgb(110, 85, 190)
                    } else {
                        Color::rgb(195, 200, 220)
                    };
                    let _ = grid.set_cell_color(cx, cy, col);
                }
            }
        }

        // Label with a turning notch so the spin reads.
        let spin = ctx.time * 3.5;
        pen.ellipse(grid, rcx, rcy, 2.2, 2.2, true);
        pen.line(
            grid,
            rcx + spin.cos() * 0.6,
            rcy + spin.sin() * 0.6,
            rcx + spin.cos() * 2.6,
            rcy + spin.sin() * 2.6,
        );
        pen.tint(
            grid,
            rcx - 2.0,
            rcy - 2.0,
            4.0,
            4.0,
            Color::rgb(235, 70, 85),
        );

        // Tonearm: pivot base, counterweight behind, arm to the stylus.
        let a = -0.25_f32;
        let (sx, sy) = (rcx + stylus_r * a.cos(), rcy + stylus_r * a.sin());
        let (qx, qy) = (27.0, 3.0);
        pen.ellipse(grid, qx, qy, 1.6, 1.6, false);
        let (dx, dy) = (sx - qx, sy - qy);
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        pen.rect(
            grid,
            qx - dx / len * 3.5 - 1.0,
            qy - dy / len * 3.5 - 1.0,
            2.0,
            2.0,
        );
        pen.line(grid, qx, qy, sx, sy);
        pen.rect(grid, sx - 1.0, sy - 0.5, 2.0, 1.5);
        for k in 0..=8 {
            let f = k as f32 / 8.0;
            pen.tint_at(grid, qx + dx * f, qy + dy * f, Color::rgb(230, 232, 242));
        }
        pen.tint_at(grid, sx, sy, Color::rgb(255, 255, 255));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Drum Kit
//    A front-on drum kit assembles piece by piece with progress: bass drum,
//    snare, floor tom, hi-hat, two rack toms and finally the crash cymbal.
//    A rock groove plays on `time` — the bass drum head flashes on the kick,
//    the snare head on the backbeat, the hi-hat opens and the crash rocks.
//    A rug under the kit fills left-to-right as a plain progress gauge.
// ---------------------------------------------------------------------------
struct DrumKit;
impl ProgressStyle for DrumKit {
    fn name(&self) -> &str {
        "drum-kit"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Front-on drum kit that assembles piece by piece with progress while a rock groove flashes the kick, snare and cymbals"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 52.0);
        let pen = Pen::new((dw as f32 - 50.0 * s) / 2.0, dh as f32 - 16.0 * s, s);
        let p = ctx.eased;

        // Groove: 8 eighth-note steps per bar at 4 steps/s.
        let step = (ctx.time * 4.0).max(0.0) as usize % 8;
        let kick = matches!(step, 0 | 3 | 4);
        let snare = matches!(step, 2 | 6);
        let hat_open = step == 7;

        let shell = Color::rgb(215, 55, 70);
        let head = Color::rgb(240, 232, 210);
        let brass = Color::rgb(240, 195, 70);
        let steel = Color::rgb(150, 155, 175);
        let flash = Color::rgb(255, 255, 255);

        // Rug — the progress gauge.
        let rug = (p * dw as f32).round() as usize;
        if rug > 0 {
            draw::hline(grid, 0, rug.min(dw) - 1, dh - 1);
            for cx in 0..(rug + 1) / 2 {
                let col = ctx.palette.sample(cx as f32 * 2.0 / dw as f32);
                let _ = grid.set_cell_color(cx, (dh - 1) / 4, col);
            }
        }

        // Hi-hat (left): stand plus two plates, opened on the off-beat.
        if p >= 0.45 {
            pen.line(grid, 4.0, 3.0, 4.0, 15.0);
            pen.line(grid, 4.0, 13.0, 1.5, 15.0);
            pen.line(grid, 4.0, 13.0, 6.5, 15.0);
            pen.tint(grid, 3.0, 6.0, 3.0, 10.0, steel);
            let gap = if hat_open { 2.0 } else { 1.0 };
            pen.line(grid, 0.0, 4.0 - gap, 8.0, 4.0 - gap);
            pen.line(grid, 0.0, 4.0, 8.0, 4.0);
            pen.tint(grid, 0.0, 1.0, 8.0, 4.0, brass);
        }

        // Crash cymbal (right): stand and a tilted plate that rocks.
        if p >= 0.85 {
            pen.line(grid, 46.0, 2.0, 46.0, 15.0);
            pen.line(grid, 46.0, 13.0, 43.5, 15.0);
            pen.line(grid, 46.0, 13.0, 48.5, 15.0);
            pen.tint(grid, 45.0, 5.0, 3.0, 11.0, steel);
            let rock = (ctx.time * 6.0).sin() * 1.2;
            pen.line(grid, 41.0, 2.5 + rock, 50.0, 0.5 - rock);
            pen.tint(grid, 41.0, 0.0, 9.0, 4.0, brass);
        }

        // Snare (left of the bass drum) on its stand.
        if p >= 0.12 {
            pen.line(grid, 13.0, 11.0, 13.0, 15.0);
            pen.line(grid, 13.0, 13.5, 10.5, 15.0);
            pen.line(grid, 13.0, 13.5, 15.5, 15.0);
            pen.tint(grid, 10.0, 12.0, 6.0, 4.0, steel);
            pen.rect(grid, 9.0, 8.0, 8.0, 3.0);
            pen.ellipse(grid, 13.0, 8.0, 4.0, 1.0, true);
            pen.tint(grid, 9.0, 8.0, 8.0, 3.0, shell);
            pen.tint(grid, 9.0, 7.0, 8.0, 1.0, if snare { flash } else { head });
        }

        // Floor tom (right of the bass drum) on three legs.
        if p >= 0.3 {
            pen.rect(grid, 33.0, 8.0, 8.0, 6.0);
            pen.ellipse(grid, 37.0, 8.0, 4.0, 1.0, true);
            pen.line(grid, 34.0, 14.0, 34.0, 15.0);
            pen.line(grid, 40.0, 14.0, 40.0, 15.0);
            pen.tint(grid, 33.0, 8.0, 8.0, 8.0, shell);
            pen.tint(grid, 33.0, 7.0, 8.0, 1.0, head);
        }

        // Bass drum: thick hoop, head ring, and a beater flash on the kick.
        pen.ellipse(grid, 25.0, 9.5, 6.0, 6.0, false);
        pen.ellipse(grid, 25.0, 9.5, 5.0, 5.0, false);
        pen.ellipse(grid, 25.0, 9.5, if kick { 3.0 } else { 1.5 }, 1.5, kick);
        pen.tint(grid, 19.0, 3.5, 12.0, 12.0, shell);
        pen.tint(grid, 22.0, 7.5, 6.0, 4.0, if kick { flash } else { head });

        // Rack toms perched on the bass drum.
        for (i, &(x, at)) in [(20.0, 0.6), (30.0, 0.7)].iter().enumerate() {
            if p >= at {
                let lift = if step == 5 + i { 0.5 } else { 0.0 };
                pen.rect(grid, x - 3.0, 1.0 + lift, 6.0, 3.0);
                pen.ellipse(grid, x, 1.0 + lift, 3.0, 0.8, true);
                pen.tint(grid, x - 3.0, 0.0, 6.0, 4.0, shell);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Sound Waveform
//    A continuous amplitude trace oscillates around the vertical centre.
//    Amplitude envelope is `eased` (silent at 0, full-scale at 1).
//    The waveform morphs from sine → sawtooth as time advances.
// ---------------------------------------------------------------------------
struct SoundWaveform;
impl ProgressStyle for SoundWaveform {
    fn name(&self) -> &str {
        "sound-waveform"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Centered oscillating waveform; amplitude grows with progress, morph sine→saw with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cy = (dh / 2) as i32;
        let half_h = (dh / 2) as f32;
        let amp = ctx.eased * (half_h - 1.0).max(0.0);

        // Morph parameter: slowly shifts between sine and sawtooth.
        let morph = ((ctx.time * 0.1).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        // Frequency: ~3 cycles across the bar.
        let freq = 3.0_f32;

        let prev_y: Option<i32> = None;
        let _ = prev_y;

        let mut last_y: Option<i32> = None;
        for xi in 0..dw {
            let xn = xi as f32 / dw.max(1) as f32;
            let phase = xn * freq * 2.0 * PI - ctx.time * 2.0;
            // Sine component.
            let sine = phase.sin();
            // Sawtooth component: 2*(phase/(2π) - floor(phase/(2π)+0.5)).
            let saw_phase = phase / (2.0 * PI);
            let saw = 2.0 * (saw_phase - (saw_phase + 0.5).floor());
            let sample = sine * (1.0 - morph) + saw * morph;
            let dot_y = cy - (sample * amp).round() as i32;

            // Connect previous and current with a vertical run.
            if let Some(ly) = last_y {
                let (lo, hi) = if ly <= dot_y {
                    (ly, dot_y)
                } else {
                    (dot_y, ly)
                };
                for y in lo..=hi {
                    draw::dot_i(grid, xi as i32, y);
                }
            } else {
                draw::dot_i(grid, xi as i32, dot_y);
            }
            last_y = Some(dot_y);
        }

        // Centre-line when amplitude is zero.
        if ctx.eased < 0.01 {
            draw::hline(grid, 0, dw - 1, cy as usize);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. Cassette Reels
//    A compact cassette: chamfered shell, corner screws, a label strip that
//    fills with progress, and the head-opening ramp at the bottom. Through
//    the window two spoked hubs turn; tape winds off the left pack onto the
//    right one as progress grows (the smaller pack spins faster).
// ---------------------------------------------------------------------------
struct CassetteReels;
impl ProgressStyle for CassetteReels {
    fn name(&self) -> &str {
        "cassette-reels"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Compact cassette whose spoked reels turn while tape winds from the left pack to the right and the label strip fills with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        // Design canvas: 40×16 shell, centred.
        let s = (dh as f32 / 16.0).min(dw as f32 / 42.0);
        let pen = Pen::new((dw as f32 - 40.0 * s) / 2.0, dh as f32 - 16.0 * s, s);
        let p = ctx.eased;

        // Shell with chamfered corners and the head-opening ramp.
        pen.line(grid, 1.0, 0.0, 39.0, 0.0);
        pen.line(grid, 1.0, 15.0, 39.0, 15.0);
        pen.line(grid, 0.0, 1.0, 0.0, 14.0);
        pen.line(grid, 40.0, 1.0, 40.0, 14.0);
        pen.line(grid, 9.0, 15.0, 11.0, 12.0);
        pen.line(grid, 11.0, 12.0, 29.0, 12.0);
        pen.line(grid, 29.0, 12.0, 31.0, 15.0);
        for &(x, y) in &[(2.0, 2.0), (38.0, 2.0), (2.0, 13.0), (38.0, 13.0)] {
            pen.ellipse(grid, x, y, 0.6, 0.6, true);
        }
        pen.tint(grid, 0.0, 0.0, 41.0, 16.0, Color::rgb(170, 175, 195));

        // Label strip filling with progress.
        let fill = 34.0 * p;
        if fill > 0.0 {
            pen.rect(grid, 3.0, 2.0, fill, 1.0);
            pen.tint(grid, 3.0, 2.0, fill, 1.0, Color::rgb(255, 104, 196));
        }

        // Window outline between the reels.
        pen.line(grid, 14.0, 5.0, 26.0, 5.0);
        pen.line(grid, 14.0, 10.0, 26.0, 10.0);

        // Tape packs (rings) and turning hubs.
        let reels = [(10.0, 1.0 - p), (30.0, p)];
        for &(cx, amount) in &reels {
            let r = 2.2 + 3.0 * amount;
            pen.ellipse(grid, cx, 7.5, r, r, false);
            if r > 3.4 {
                pen.ellipse(grid, cx, 7.5, r - 1.0, r - 1.0, false);
            }
            pen.tint(
                grid,
                cx - r,
                7.5 - r,
                2.0 * r,
                2.0 * r,
                Color::rgb(160, 95, 55),
            );
            let spin = ctx.time * 9.0 / r;
            for k in 0..3 {
                let a = spin + k as f32 * PI * 2.0 / 3.0;
                let (sa, ca) = a.sin_cos();
                pen.line(grid, cx, 7.5, cx + ca * 1.6, 7.5 + sa * 1.6);
            }
            pen.tint(grid, cx - 1.6, 5.9, 3.2, 3.2, Color::rgb(245, 240, 230));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 8. Conductor's Baton
//    A tailcoated conductor seen from behind, baton raised in the right hand.
//    The baton tip traces the 4/4 pattern (down, in, out, up) with a fading
//    trail; to the right, eighth notes stream out across the width as
//    progress grows, bobbing in time with the beat.
// ---------------------------------------------------------------------------
struct ConductorBaton;

/// Hand position of the 4/4 conducting pattern at beat-time `b` (design units).
fn conduct_hand(b: f32) -> (f32, f32) {
    // Ictus points: 1 down, 2 in (left), 3 out (right), 4 up.
    const KEYS: [(f32, f32); 4] = [(17.0, 9.0), (13.5, 7.0), (22.0, 7.5), (18.0, 2.5)];
    let b = b.rem_euclid(4.0);
    let i = b as usize % 4;
    let f = b.fract();
    let f = f * f * (3.0 - 2.0 * f);
    let (a, c) = (KEYS[(i + 3) % 4], KEYS[i]);
    (a.0 + (c.0 - a.0) * f, a.1 + (c.1 - a.1) * f)
}

impl ProgressStyle for ConductorBaton {
    fn name(&self) -> &str {
        "conductor-baton"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Tailcoated conductor whose baton traces the 4/4 beat pattern; eighth notes stream out across the stage as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = dh as f32 / 16.0;
        let pen = Pen::new(0.0, 0.0, s);
        let units_w = dw as f32 / s;
        let beat = ctx.time * 2.0;

        // Figure: head, tailcoat with split tails, left arm holding the score.
        pen.ellipse(grid, 8.0, 2.5, 2.2, 2.2, true);
        pen.rect(grid, 4.0, 5.5, 9.0, 6.5);
        pen.rect(grid, 4.0, 12.0, 4.0, 4.0);
        pen.rect(grid, 9.0, 12.0, 4.0, 4.0);
        pen.line(grid, 4.0, 6.0, 1.0, 10.0);
        pen.tint(grid, 0.0, 5.0, 14.0, 11.0, Color::rgb(120, 130, 175));
        pen.tint(grid, 6.0, 0.0, 4.0, 5.0, Color::rgb(235, 185, 145));

        // Baton arm: shoulder to hand, baton extends the forearm line.
        let (hx, hy) = conduct_hand(beat);
        pen.line(grid, 12.0, 6.0, hx, hy);
        let (dx, dy) = (hx - 12.0, hy - 6.0);
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let tip = (hx + dx / len * 7.0, hy + dy / len * 7.0 - 3.0);
        pen.line(grid, hx, hy, tip.0, tip.1);
        for k in 1..=6 {
            let (tx, ty) = conduct_hand(beat - k as f32 * 0.07);
            let (ex, ey) = (tx - 12.0, ty - 6.0);
            let l = (ex * ex + ey * ey).sqrt().max(0.01);
            let (bx, by) = (tx + ex / l * 7.0, ty + ey / l * 7.0 - 3.0);
            pen.ellipse(grid, bx, by, 0.3, 0.3, true);
            pen.tint_at(grid, bx, by, Color::rgb(140, 120, 220));
        }
        pen.tint_at(grid, hx, hy, Color::rgb(235, 185, 145));
        pen.tint_at(grid, tip.0, tip.1, Color::rgb(255, 255, 255));
        pen.tint_at(
            grid,
            (hx + tip.0) / 2.0,
            (hy + tip.1) / 2.0,
            Color::rgb(255, 255, 255),
        );

        // Notes stream out to the right, one per 8 units of stage.
        let start = 32.0;
        let slots = ((units_w - start) / 8.0).max(0.0) as usize;
        let shown = ((ctx.eased * slots as f32).ceil() as usize).min(slots);
        for i in 0..shown {
            let x = start + 8.0 * i as f32;
            let y = 10.5 + (beat * PI + i as f32 * 0.9).sin() * 2.0;
            eighth_note(&pen, grid, x, y, true);
            let col = sample_tint(i as f32 / slots.max(1) as f32);
            pen.tint(grid, x - 2.0, 0.0, 6.0, 16.0, col);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. Sheet Music Scroll
//    A paper scroll of sheet music held between two rollers. The staff,
//    stemmed notes and bar lines slide left across the open span with
//    `time`; as progress grows the paper winds off the right roller onto the
//    left one, so the left roll fattens while the right one thins.
// ---------------------------------------------------------------------------
struct SheetScroll;
impl ProgressStyle for SheetScroll {
    fn name(&self) -> &str {
        "sheet-scroll"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Paper scroll of sheet music between two rollers; notes slide past and the paper winds onto the left roll as progress grows"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = dh as f32 / 16.0;
        let pen = Pen::new(0.0, 0.0, s);
        let units_w = dw as f32 / s;
        let p = ctx.eased;
        let parchment = Color::rgb(240, 222, 170);

        // Rollers: vertical ellipses whose girth tracks the wound paper.
        let rl = 1.5 + 3.0 * p;
        let rr = 1.5 + 3.0 * (1.0 - p);
        let (xl, xr) = (rl + 0.5, units_w - rr - 1.0);
        for &(cx, r) in &[(xl, rl), (xr, rr)] {
            pen.ellipse(grid, cx, 8.0, r, 8.0, false);
            pen.line(grid, cx, 0.0, cx, 15.0);
            pen.tint(grid, cx - r, 0.0, 2.0 * r + 1.0, 16.0, parchment);
        }

        // Open span: staff, bar lines and notes scrolling left.
        let (a, b) = (xl + rl + 1.0, xr - rr - 1.0);
        if b > a {
            for i in 0..5 {
                let y = 2.0 + 3.0 * i as f32;
                pen.line(grid, a, y, b, y);
            }
            pen.tint(grid, a, 0.0, b - a, 16.0, Color::rgb(110, 120, 160));
            let melody = [3usize, 5, 7, 5, 4, 6, 8, 2, 1, 3, 5, 6];
            let scroll = ctx.time * 6.0;
            let first = ((scroll + a - 6.0) / 7.0).floor() as i32;
            let last = ((scroll + b + 6.0) / 7.0).ceil() as i32;
            for n in first.max(0)..=last.min(first.max(0) + 512) {
                let x = n as f32 * 7.0 - scroll;
                if n % 4 == 0 {
                    let bx = x - 3.5;
                    if bx > a && bx < b {
                        pen.line(grid, bx, 2.0, bx, 14.0);
                    }
                }
                if x - 2.0 > a && x + 4.0 < b {
                    let step = melody[n as usize % melody.len()];
                    let y = 14.0 - 1.5 * step as f32;
                    eighth_note(&pen, grid, x, y, step < 4);
                    pen.tint(grid, x - 2.0, y - 6.5, 6.0, 13.0, sample_tint(x / units_w));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. Tuning Fork
//    A steel tuning fork lies on its side: ball-ended handle, U-shaped yoke
//    and two long tines that buzz apart and together. Sound-wave arcs ripple
//    out from the tine tips and reach further across the width as progress
//    grows.
// ---------------------------------------------------------------------------
struct TuningFork;
impl ProgressStyle for TuningFork {
    fn name(&self) -> &str {
        "tuning-fork"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Steel tuning fork on its side with buzzing tines; sound-wave arcs ripple out further with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 70.0);
        let pen = Pen::new(0.0, (dh as f32 - 16.0 * s) / 2.0, s);
        let steel = Color::rgb(205, 212, 228);

        // Handle with a ball end, and the U-shaped yoke.
        pen.ellipse(grid, 1.5, 8.0, 1.5, 1.5, true);
        pen.rect(grid, 1.0, 7.0, 10.0, 2.0);
        pen.ellipse(grid, 13.0, 8.0, 3.0, 5.0, false);
        pen.ellipse(grid, 13.0, 8.0, 2.5, 4.5, false);

        // Tines: deflection grows towards the tips; they buzz in antiphase.
        let buzz = (ctx.time * 37.0).sin() * 1.2;
        let (t0, t1) = (13.0, 36.0);
        let n = ((t1 - t0) * s).ceil().max(1.0) as usize;
        for k in 0..=n {
            let x = t0 + (t1 - t0) * k as f32 / n as f32;
            let f = (x - t0) / (t1 - t0);
            let d = buzz * f * f;
            pen.rect(grid, x, 3.0 - d, 0.5, 1.5);
            pen.rect(grid, x, 11.5 + d, 0.5, 1.5);
        }
        // Mask the yoke's right half so the tines read as one U.
        pen.tint(grid, 0.0, 2.0, t1 + 1.0, 12.0, steel);

        // Sound arcs centred between the tips, travelling outwards.
        let (cx, cy) = pen.map(t1 + 1.0, 8.0);
        let reach = (dw as f32 - cx) * ctx.eased;
        let spacing = 6.0 * s.max(0.5);
        let phase = (ctx.time * 10.0 * s.max(0.5)).rem_euclid(spacing);
        let rings = ((reach - phase - 2.0) / spacing).max(-1.0).floor() as i32 + 1;
        for k in 0..rings.clamp(0, 512) {
            let r = phase + 2.0 + spacing * k as f32;
            let fade = 1.0 - r / (dw as f32 - cx).max(1.0);
            let col = Color::rgb(
                (90.0 + 140.0 * fade) as u8,
                (140.0 + 80.0 * fade) as u8,
                255,
            );
            let steps = (r * 2.0).ceil().max(4.0) as usize;
            for k in 0..=steps {
                let a = -1.0 + 2.0 * k as f32 / steps as f32;
                let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
                if x >= 0.0 && y >= 0.0 {
                    let _ = grid.set_cell_color(x as usize / 2, y as usize / 4, col);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 11. Volume Knob
//    A ridged amp knob with a pointer, ringed by a 270° scale of tick marks
//    that light from 7 o'clock round to 5 o'clock as progress turns it up.
//    Beside it a stereo LED meter (green → amber → red) dances around the
//    current level.
// ---------------------------------------------------------------------------
struct VolumeKnob;
impl ProgressStyle for VolumeKnob {
    fn name(&self) -> &str {
        "volume-knob"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Amp volume knob turning up its 7-to-5 o'clock tick scale with progress, beside a dancing stereo LED meter"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 20.0);
        let pen = Pen::new(0.0, (dh as f32 - 16.0 * s) / 2.0, s);
        let p = ctx.eased;

        // Knob body: thick ridged rim, pointer from the hub.
        pen.ellipse(grid, 8.0, 8.0, 5.0, 5.0, false);
        pen.ellipse(grid, 8.0, 8.0, 4.0, 4.0, false);
        let a = PI * 0.75 + p * PI * 1.5;
        let (sa, ca) = a.sin_cos();
        pen.line(grid, 8.0, 8.0, 8.0 + ca * 3.5, 8.0 + sa * 3.5);
        pen.tint(grid, 3.5, 3.5, 9.0, 9.0, Color::rgb(200, 205, 220));
        pen.tint_at(
            grid,
            8.0 + ca * 3.0,
            8.0 + sa * 3.0,
            Color::rgb(255, 255, 255),
        );

        // Scale ticks: 11 marks over 270°, starting at 7 o'clock.
        let dim = Color::rgb(95, 95, 122);
        for i in 0..11 {
            let f = i as f32 / 10.0;
            let a = PI * 0.75 + f * PI * 1.5;
            let (sa, ca) = a.sin_cos();
            pen.line(
                grid,
                8.0 + ca * 6.5,
                8.0 + sa * 6.5,
                8.0 + ca * 7.8,
                8.0 + sa * 7.8,
            );
            let col = if f <= p + 0.001 { sample_tint(f) } else { dim };
            pen.tint_at(grid, 8.0 + ca * 7.2, 8.0 + sa * 7.2, col);
        }

        // Stereo LED meter: one segment per cell, two channels.
        let x0 = (19.0 * s).ceil() as usize + 1;
        let x0 = x0 + x0 % 2;
        let segs = dw.saturating_sub(x0) / 2;
        let rows = [(dh / 4, dh * 3 / 8), (dh * 5 / 8, dh * 3 / 4)];
        for (chn, &(y0, y1)) in rows.iter().enumerate() {
            let wob = (ctx.time * (7.0 + chn as f32 * 2.3)).sin() * 0.05
                + (ctx.time * 13.0 + chn as f32).sin() * 0.03;
            let level = ((p + wob) * segs as f32).clamp(0.0, segs as f32) as usize;
            for i in 0..segs {
                let x = x0 + i * 2;
                let f = i as f32 / segs.max(1) as f32;
                let col = if i >= level {
                    dim
                } else if f < 0.7 {
                    Color::rgb(80, 220, 110)
                } else if f < 0.88 {
                    Color::rgb(245, 200, 60)
                } else {
                    Color::rgb(255, 70, 70)
                };
                if i < level {
                    draw::fill_rect(grid, x, y0, 1, y1.saturating_sub(y0) + 1);
                } else {
                    draw::dot(grid, x, (y0 + y1) / 2);
                }
                for cy in y0 / 4..=y1 / 4 {
                    let _ = grid.set_cell_color(x / 2, cy, col);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. Beat Pulse
//    Concentric circles expand outward from the centre on each beat.
//    Multiple rings are in flight simultaneously, each at a different expansion
//    phase.  Progress determines the overall brightness / ring count.
// ---------------------------------------------------------------------------
struct BeatPulse;
impl ProgressStyle for BeatPulse {
    fn name(&self) -> &str {
        "beat-pulse"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Concentric rings expanding outward on each beat; ring density grows with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }

        let cx = (dw / 2) as i32;
        let cy = (dh / 2) as i32;
        let max_r = ((dh / 2).min(dw / 2)) as i32;
        if max_r == 0 {
            return Ok(());
        }

        // Number of simultaneous rings scales with progress.
        let ring_count = (1.0 + ctx.eased * 3.0).round() as usize;
        // Ring period: 0.5 s per beat.
        let beat_period = 0.5_f32;

        for ring in 0..ring_count {
            // Stagger rings evenly across one beat period.
            let offset = ring as f32 / ring_count as f32;
            let phase = ((ctx.time / beat_period) + offset).fract(); // 0..1
            let r = (phase * max_r as f32).round() as i32;
            if r <= 0 {
                continue;
            }

            // Fade out as ring expands (bright at centre, dim at edge).
            let fade = 1.0 - phase;

            // Draw the ring circle.
            let steps = (2.0 * PI * r as f32).ceil() as usize;
            let steps = steps.max(4);
            for s in 0..steps {
                // Skip some dots for fainter rings.
                let skip = if fade > 0.5 { 1 } else { 2 };
                if s % skip != 0 {
                    continue;
                }
                let theta = 2.0 * PI * s as f32 / steps as f32;
                let dx = (r as f32 * theta.cos()).round() as i32;
                let dy = (r as f32 * theta.sin()).round() as i32;
                draw::dot_i(grid, cx + dx, cy + dy);
            }
        }

        // Centre bright dot on the beat onset.
        let beat_phase = (ctx.time / beat_period).fract();
        if beat_phase < 0.1 {
            draw::dot_i(grid, cx, cy);
        }

        // Tint filled cells proportional to progress.
        let (_, ch) = grid.dimensions();
        let (cw, _) = grid.dimensions();
        for cy_cell in 0..ch {
            draw::tint_row(
                grid,
                cy_cell,
                0,
                cw.saturating_sub(1),
                ctx.palette.sample(ctx.eased),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 13. Acoustic Guitar
//    A full acoustic guitar seen from the front, headstock left: pegged
//    headstock, nut, a fretted neck with realistic (12-TET) fret spacing, a
//    figure-eight body with sound hole and bridge, and strings running nut to
//    bridge. A fretting finger slides up the neck with progress (the played
//    frets glow), and the strings between finger and bridge ring after each
//    strum.
// ---------------------------------------------------------------------------
struct AcousticGuitar;
impl ProgressStyle for AcousticGuitar {
    fn name(&self) -> &str {
        "acoustic-guitar"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Acoustic guitar with headstock, fretted neck and sound-holed body; a fretting finger slides up the neck with progress while strummed strings ring"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 88.0);
        let pen = Pen::new(
            (dw as f32 - 88.0 * s) / 2.0,
            (dh as f32 - 16.0 * s) / 2.0,
            s,
        );
        let fret_x = |f: f32| 8.0 + 96.0 * (1.0 - 2f32.powf(-f / 12.0));
        let (nut, bridge) = (8.0, 81.0);

        // Body: rim of the union of two bouts, then sound hole and bridge.
        let (ox, oy) = pen.map(0.0, 0.0);
        let inside = |x: f32, y: f32, shrink: f32| {
            let bout = |cx: f32, rx: f32, ry: f32| {
                ((x - cx) / (rx - shrink)).powi(2) + ((y - 8.0) / (ry - shrink)).powi(2) <= 1.0
            };
            bout(64.0, 8.0, 6.2) || bout(78.0, 10.0, 7.9)
        };
        let (bx0, bx1) = (
            pen.map(55.0, 0.0).0 as i32,
            pen.map(88.0, 0.0).0.ceil() as i32,
        );
        for py in 0..dh.min(4096) as i32 {
            for px in bx0.max(0)..bx1.min(4096) {
                let (x, y) = ((px as f32 - ox) / s, (py as f32 - oy) / s);
                if inside(x, y, 0.0) && !inside(x, y, 1.0 / s) {
                    draw::dot_i(grid, px, py);
                }
            }
        }
        pen.tint(grid, 55.0, 0.0, 33.0, 16.0, Color::rgb(235, 180, 105));
        pen.ellipse(grid, 67.0, 8.0, 3.0, 3.0, false);
        pen.tint(grid, 64.0, 5.0, 6.0, 6.0, Color::rgb(150, 85, 40));
        pen.rect(grid, bridge, 5.5, 2.0, 5.0);
        pen.tint(grid, bridge, 5.5, 2.0, 5.0, Color::rgb(150, 85, 40));

        // Headstock with tuning pegs, nut and frets (strings outline the neck).
        pen.rect(grid, 0.0, 5.0, 7.0, 6.0);
        for k in 0..3 {
            let x = 1.5 + 2.0 * k as f32;
            pen.rect(grid, x, 3.5, 1.0, 1.0);
            pen.rect(grid, x, 11.5, 1.0, 1.0);
        }
        pen.tint(grid, 0.0, 3.0, 7.0, 10.0, Color::rgb(150, 90, 50));
        pen.line(grid, nut, 5.0, nut, 11.0);
        let finger_f = ctx.eased * 12.0;
        for f in 1..=12 {
            let x = fret_x(f as f32);
            pen.line(grid, x, 5.0, x, 11.0);
            let x_prev = fret_x((f - 1) as f32);
            let col = if (f as f32) <= finger_f.ceil() && finger_f > 0.0 {
                sample_tint(f as f32 / 12.0)
            } else {
                Color::rgb(160, 105, 60)
            };
            pen.tint(grid, x_prev + 1.0, 5.0, x - x_prev - 1.0, 6.0, col);
        }

        // Strings: still behind the finger, ringing between finger and bridge.
        let finger_x = if finger_f > 0.0 {
            fret_x(finger_f.ceil()) - 1.5
        } else {
            nut
        };
        let ring = (-(ctx.time * 2.0).fract() * 3.0).exp();
        let n = ((bridge - nut) * s).ceil().max(1.0) as usize;
        for k in 0..=n {
            let x = nut + (bridge - nut) * k as f32 / n as f32;
            for st in 0..3 {
                let mut y = 6.0 + 2.0 * st as f32;
                if x > finger_x {
                    let u = (x - finger_x) / (bridge - finger_x).max(1.0);
                    y += (PI * u).sin() * ring * (ctx.time * 40.0 + st as f32 * 2.1).sin();
                }
                pen.rect(grid, x, y.round(), 0.01, 0.01);
            }
        }

        // Fretting fingertip.
        if finger_f > 0.0 {
            pen.ellipse(grid, finger_x, 8.0, 1.6, 3.2, true);
            pen.tint(
                grid,
                finger_x - 1.0,
                5.0,
                2.0,
                6.0,
                Color::rgb(255, 240, 245),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 14. Trumpet
//    A trumpet in profile: mouthpiece at the left, the lead pipe running to
//    a flared bell at the right, a return loop of tubing underneath and three
//    piston valves with finger buttons that press in a fanfare on `time`.
//    The brass lights up from mouthpiece to bell as progress grows.
// ---------------------------------------------------------------------------
struct Trumpet;
impl ProgressStyle for Trumpet {
    fn name(&self) -> &str {
        "trumpet"
    }
    fn theme(&self) -> &str {
        "music"
    }
    fn describe(&self) -> &str {
        "Trumpet in profile with flared bell, tubing loop and three piston valves pressing a fanfare; the brass lights from mouthpiece to bell with progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let s = (dh as f32 / 16.0).min(dw as f32 / 88.0);
        let pen = Pen::new((dw as f32 - 86.0 * s) / 2.0, dh as f32 - 16.0 * s, s);

        // Mouthpiece cup and lead pipe up to the bell.
        pen.line(grid, 0.0, 6.0, 0.0, 9.0);
        pen.line(grid, 0.0, 7.5, 6.0, 7.5);
        pen.rect(grid, 6.0, 7.0, 58.0, 2.0);

        // Bell: edges flare cubically to a rim.
        let n = (22.0 * s).ceil().max(1.0) as usize;
        for k in 0..=n {
            let x = 64.0 + 22.0 * k as f32 / n as f32;
            let f = ((x - 64.0) / 22.0).powi(3) * 6.5;
            pen.rect(grid, x, 7.0 - f, 0.3, 0.6);
            pen.rect(grid, x, 8.4 + f, 0.3, 0.6);
        }
        pen.line(grid, 86.0, 0.5, 86.0, 15.5);

        // Return loop below: right bend, lower pipe, left bend into the valves.
        pen.ellipse(grid, 58.0, 10.5, 3.0, 3.0, false);
        pen.rect(grid, 26.0, 12.5, 32.0, 1.0);
        pen.ellipse(grid, 26.0, 10.5, 2.5, 2.5, false);

        // Valves: casings, caps and buttons (pressed ones sit lower).
        let beat = (ctx.time * 4.0).max(0.0) as usize;
        let fanfare = [0b001u8, 0b011, 0b000, 0b101, 0b010, 0b110, 0b000, 0b100];
        let pressed = fanfare[beat % fanfare.len()];
        for v in 0..3 {
            let x = 33.0 + 6.0 * v as f32;
            pen.rect(grid, x, 4.0, 3.0, 10.0);
            let down = if pressed & (1 << v) != 0 { 1.5 } else { 0.0 };
            pen.line(grid, x + 1.5, 4.0, x + 1.5, 1.5 + down);
            pen.rect(grid, x - 0.5, down, 4.0, 1.5);
        }

        // Brass colour: lit from the mouthpiece to the progress point, with a
        // rule underneath so progress still reads on monochrome terminals.
        let lit = 87.0 * ctx.eased;
        if lit > 0.5 {
            pen.rect(grid, 0.0, 15.2, lit.min(85.0), 0.6);
        }
        let cols = ((87.0 * s / 2.0).ceil() as usize).min(4096);
        for i in 0..cols {
            let x = i as f32 * 2.0 / s;
            let col = if x < lit {
                Color::rgb(255, 205, 80)
            } else {
                Color::rgb(115, 88, 48)
            };
            pen.tint(grid, x, 0.0, 2.0 / s, 16.0, col);
        }
        for v in 0..3 {
            let x = 33.0 + 6.0 * v as f32;
            let down = if pressed & (1 << v) != 0 { 1.5 } else { 0.0 };
            pen.tint(grid, x - 0.5, down, 4.0, 1.5, Color::rgb(245, 240, 230));
        }
        Ok(())
    }
}
