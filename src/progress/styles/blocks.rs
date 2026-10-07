//! Unicode block-element progress bars — the **blocky ↔ smooth** spectrum.
//!
//! Every style in this theme showcases a distinct structural form built from
//! Unicode block, shade, or box-drawing glyphs. Color is secondary; the
//! *shape* is the point. Styles span the full contrast axis from
//! `draw::hbar`'s eighth-precise smoothness down to coarse single-cell
//! snap, with dithering, stacking, masonry, and spectrum animation in between.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Theme tint — cool block blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(96, 202, 255);
const TINT_END: Color = Color::rgb(52, 88, 222);

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

/// All styles in the `blocks` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per structural form, covering the
/// full range of block-element technique: smooth sub-cell hbar, coarse snap,
/// segmented LED, equalizer columns, thermometer, shade dither, brick masonry,
/// Bayer ordered dither, stacked rows, waterfall, double-ended, a shade
/// back-fill gradient meter, and a battery cell with a terminal nub.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(SmoothHbar)),
        Box::new(Tinted(BlockyHbar)),
        Box::new(Tinted(SegmentedLed)),
        Box::new(Tinted(Equalizer)),
        Box::new(Tinted(Thermometer)),
        Box::new(Tinted(DitherRamp)),
        Box::new(Tinted(BrickWall)),
        Box::new(Tinted(BayerDither)),
        Box::new(Tinted(StackedBars)),
        Box::new(Tinted(Waterfall)),
        Box::new(Tinted(DoubleEnded)),
        Box::new(Tinted(GradientMeter)),
        Box::new(Tinted(Battery)),
    ]
}

// ---------------------------------------------------------------------------
// 1. smooth-hbar — eighth-precise sub-character bar via draw::hbar
// ---------------------------------------------------------------------------

struct SmoothHbar;
impl ProgressStyle for SmoothHbar {
    fn name(&self) -> &str {
        "smooth-hbar"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Eighth-precise sub-cell smooth bar — the gold standard crisp fill using draw::hbar"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (_, cells_h) = grid.dimensions();
        // Fill every row with the same smooth bar so tall grids are solid.
        for cy in 0..cells_h {
            draw::hbar(grid, cy, ctx.eased);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. blocky-hbar — same fill SNAPPED to whole █ cells only (coarse)
// ---------------------------------------------------------------------------

struct BlockyHbar;
impl ProgressStyle for BlockyHbar {
    fn name(&self) -> &str {
        "blocky-hbar"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Full-cell █ snap only — explicit coarse counterpart to smooth-hbar, no sub-cell precision"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Snap to whole cells — no partial edge glyph.
        let filled = (ctx.eased * cells_w as f32).floor() as usize;
        let filled = filled.min(cells_w);
        for cy in 0..cells_h {
            for cx in 0..filled {
                draw::glyph(grid, cx, cy, '█');
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. segmented-led — lit █ cells with one-cell gaps between segments
// ---------------------------------------------------------------------------

struct SegmentedLed;
impl ProgressStyle for SegmentedLed {
    fn name(&self) -> &str {
        "segmented-led"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Discrete lit █ segments separated by single-cell gaps, like a VU meter"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Each segment: 2 wide, 1 gap. Minimum segment width = 1 with no gap if tiny.
        let seg_w = 2usize;
        let gap_w = 1usize;
        let stride = (seg_w + gap_w).max(1);
        let n_segs = (cells_w / stride).max(1);
        let lit = (ctx.eased * n_segs as f32).round() as usize;
        let lit = lit.min(n_segs);
        for s in 0..lit {
            let x0 = s * stride;
            for cx in x0..(x0 + seg_w).min(cells_w) {
                for cy in 0..cells_h {
                    draw::glyph(grid, cx, cy, '█');
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. equalizer — animated spectrum columns of vblock heights
// ---------------------------------------------------------------------------

struct Equalizer;
impl ProgressStyle for Equalizer {
    fn name(&self) -> &str {
        "equalizer"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Animated spectrum equalizer: vblock columns with sinusoidal heights, lit count gated by eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        let lit_cols = (ctx.eased * cells_w as f32).round() as usize;
        let lit_cols = lit_cols.min(cells_w);
        let cells_h_f = cells_h as f32;

        for cx in 0..lit_cols {
            // Synthetic frequency per column, animated by time.
            let freq = 1.0 + (cx as f32 / cells_w.max(1) as f32) * 3.0;
            let phase = ctx.time * 2.5 + cx as f32 * 0.4;
            let raw = (phase * freq).sin() * 0.5 + 0.5; // 0..1
            let col_height_f = raw * cells_h_f;
            let full_cells = col_height_f.floor() as usize;
            let frac_eighth = ((col_height_f.fract() * 8.0).round() as usize).min(8);

            // Draw from the bottom up.
            for row in 0..full_cells.min(cells_h) {
                let cy = cells_h.saturating_sub(1).saturating_sub(row);
                draw::vblock(grid, cx, cy, 8);
            }
            // Partial top cell.
            if full_cells < cells_h && frac_eighth > 0 {
                let cy = cells_h.saturating_sub(1).saturating_sub(full_cells);
                draw::vblock(grid, cx, cy, frac_eighth);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. thermometer — single column filling bottom-to-top via vblock
// ---------------------------------------------------------------------------

struct Thermometer;
impl ProgressStyle for Thermometer {
    fn name(&self) -> &str {
        "thermometer"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Vertical column filling bottom-to-top with vblock eighths, like a mercury thermometer"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }
        let (dw, _dh) = draw::dot_dims(grid);
        let dwi = dw as i32;

        // Mercury columns: the two centre cell columns form the capillary.
        let col_r = cells_w / 2;
        let col_l = col_r.saturating_sub(1);
        // Bottom cell row is the bulb; the rows above are the tube.
        let tube_rows = cells_h.saturating_sub(1).max(1);
        // Mercury level with a faint 0.25 Hz simmer (seamless 4 s loop).
        let simmer = (ctx.time * PI * 0.5).sin() * 2.0;
        let total_eighths = ((ctx.eased * tube_rows as f32 * 8.0) + simmer)
            .clamp(0.0, tube_rows as f32 * 8.0)
            .round() as usize;
        let full_cells = (total_eighths / 8).min(tube_rows);
        let rem_eighths = total_eighths % 8;

        // Fill the capillary bottom-to-top with vblock eighths.
        for row in 0..full_cells {
            let cy = cells_h.saturating_sub(2).saturating_sub(row);
            draw::vblock(grid, col_l, cy, 8);
            draw::vblock(grid, col_r, cy, 8);
        }
        if full_cells < tube_rows && rem_eighths > 0 {
            let cy = cells_h.saturating_sub(2).saturating_sub(full_cells);
            draw::vblock(grid, col_l, cy, rem_eighths);
            draw::vblock(grid, col_r, cy, rem_eighths);
        }

        // Everything below is braille-dot scenery, kept out of the two glyph
        // columns so no vblock ever buries dot artwork.
        let tube_x0 = col_l as i32 * 2; // leftmost dot of the capillary
        let tube_x1 = col_r as i32 * 2 + 1; // rightmost dot of the capillary
        let tube_bot = (cells_h as i32 - 1) * 4; // dot row where the bulb starts

        // Glass walls flanking the capillary, with rounded top corners.
        for y in 1..tube_bot {
            draw::dot_i(grid, tube_x0 - 1, y);
            draw::dot_i(grid, tube_x1 + 1, y);
        }
        draw::dot_i(grid, tube_x0 - 1, 0);
        draw::dot_i(grid, tube_x1 + 1, 0);

        // Bulb: filled mercury reservoir, wider than the tube, in the bottom
        // cell row (that row holds no vblocks, so dots are safe there).
        let bx = (tube_x0 + tube_x1) as f32 / 2.0;
        let by = tube_bot as f32 + 1.5;
        for dy in tube_bot..tube_bot + 4 {
            let fy = (dy as f32 - by) / 2.0;
            let span = ((1.0 - fy * fy).max(0.0)).sqrt() * 4.5;
            for dx in (bx - span).round() as i32..=(bx + span).round() as i32 {
                draw::dot_i(grid, dx, dy);
            }
        }

        // Graduation ticks on both sides of the glass: long every other mark.
        let mut mark = 0i32;
        let mut y = 1i32;
        while y < tube_bot {
            let len = if mark % 2 == 0 { 3 } else { 2 };
            for j in 0..len {
                draw::dot_i(grid, tube_x0 - 4 - j, y);
                draw::dot_i(grid, tube_x1 + 4 + j, y);
            }
            mark += 1;
            y += 3;
        }

        // Reading line: a dotted rule across the full width at the mercury
        // top, skipping the instrument itself. The dashes crawl on a 4/s
        // slot (16 per 4 s loop, mod 4 → seamless).
        let level_y = (tube_bot - (total_eighths as i32 + 1) / 2).clamp(0, tube_bot);
        let crawl = ((ctx.time * 4.0) as i32).rem_euclid(4);
        for x in (0..dwi + 4).step_by(4) {
            let x = x - crawl;
            if x < tube_x0 - 8 || x > tube_x1 + 8 {
                draw::dot_i(grid, x, level_y);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. dither-ramp — left-to-right SHADES gradient advancing with eased
// ---------------------------------------------------------------------------

struct DitherRamp;
impl ProgressStyle for DitherRamp {
    fn name(&self) -> &str {
        "dither-ramp"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Left-to-right shade ramp ░▒▓█ whose density front advances with eased progress"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // The fill frontier in fractional cells.
        let frontier = ctx.eased * cells_w as f32;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let cell_f = cx as f32;
                // Distance behind the frontier determines density.
                let behind = frontier - cell_f;
                let level = if behind <= 0.0 {
                    0 // unfilled
                } else if behind < 1.0 {
                    // Transitional cell: interpolate shade level.
                    (behind * 4.0).ceil() as usize
                } else {
                    // Fully behind frontier.
                    let t = (cell_f / frontier.max(1.0)).clamp(0.0, 1.0);
                    // Gradient: cells near the start are lighter.
                    (t * 4.0).round() as usize
                };
                draw::shade(grid, cx, cy, level.min(4));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. brick-wall — running-bond courses laid with a stepped lead
// ---------------------------------------------------------------------------

struct BrickWall;
impl ProgressStyle for BrickWall {
    fn name(&self) -> &str {
        "brick-wall"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Running-bond brick wall laid course by course: ▇ bricks with thin \
         mortar lines and ▊ head joints, half-brick offsets on alternate \
         courses, a stepped lead, and a fresh ▒/▓ brick being set at each frontier"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }
        // Brick length in cells including its head joint; half for the bond.
        let brick: i32 = if cells_w >= 20 { 5 } else { 3 };
        let half = brick / 2;
        // Masons build a stepped lead: each course up trails by half a brick.
        let lag = half as f32;
        let total = cells_w as f32 + lag * cells_h.saturating_sub(1) as f32;
        let reach = ctx.eased.clamp(0.0, 1.0) * total;
        let fresh = if (ctx.time * 6.0).sin() > 0.0 {
            '▓'
        } else {
            '▒'
        };

        for cy in 0..cells_h {
            let course = cells_h - 1 - cy; // 0 = bottom course
            let frontier = reach - course as f32 * lag;
            let offset = if course % 2 == 1 { half } else { 0 };
            let mut x0 = -offset;
            while x0 < cells_w as i32 {
                let x1 = x0 + brick;
                let end = x1.min(cells_w as i32);
                let laid = end as f32 <= frontier + 0.01;
                let laying = !laid && (x0.max(0) as f32) < frontier;
                if laid || laying {
                    for cx in x0.max(0)..end {
                        let c = if laying {
                            fresh
                        } else if cx == x1 - 1 {
                            '▊' // head joint: a sliver of mortar on the right
                        } else {
                            '▇' // bed joint: the 1/8 gap on top is the mortar line
                        };
                        draw::glyph(grid, cx as usize, cy, c);
                    }
                }
                x0 = x1;
            }
        }
        Ok(())
    }
}

struct BayerDither;

/// 4×4 Bayer matrix values in 0..16.
const BAYER4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

impl ProgressStyle for BayerDither {
    fn name(&self) -> &str {
        "bayer-dither"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "4×4 Bayer ordered-dither reveal — threshold rises with eased, producing a stippled fill"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // Threshold rises 0..=16 with progress.
        let threshold = (ctx.eased * 16.0).round() as u8;
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let bx = cx % 4;
                let by = cy % 4;
                let bval = BAYER4[by][bx];
                if bval < threshold {
                    draw::glyph(grid, cx, cy, '█');
                } else if bval == threshold && threshold > 0 {
                    draw::shade(grid, cx, cy, 2); // ▒ at the transition band
                }
                // else: leave blank (space)
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 9. stacked-bars — multi-row stacked bar chart filling
// ---------------------------------------------------------------------------

struct StackedBars;
impl ProgressStyle for StackedBars {
    fn name(&self) -> &str {
        "stacked-bars"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Multi-row stacked bar chart: each row a differently-shaded series filling with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // We use shade levels cycling per row to give each series a distinct density.
        // Series allocation: each row gets a fraction of the total progress.
        let n_series = cells_h.max(1);
        for cy in 0..cells_h {
            // Each series has slightly different progress fractions (staggered).
            let series_offset = (cy as f32 / n_series as f32) * 0.2;
            let series_frac = (ctx.eased - series_offset).clamp(0.0, 1.0);
            let filled = (series_frac * cells_w as f32).round() as usize;
            let filled = filled.min(cells_w);
            // Shade level cycles across rows: row 0 = █, row 1 = ▓, row 2 = ▒, …
            let shade_level = 4 - (cy % 4);
            for cx in 0..filled {
                draw::shade(grid, cx, cy, shade_level);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 10. waterfall — ledge, falling shade streams, splashing pool
// ---------------------------------------------------------------------------

struct Waterfall;
impl ProgressStyle for Waterfall {
    fn name(&self) -> &str {
        "waterfall"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Waterfall: water pours over a ▀ ledge and falls in ░▒▓█ streams that \
         run downward at slightly different speeds into a ▄ pool churned by \
         splashes; the falls widen left → right with eased"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        let frontier = (ctx.eased.clamp(0.0, 1.0) * cells_w as f32).round() as usize;
        let lip = cells_h >= 3;
        let pool = cells_h >= 3;
        for cx in 0..frontier.min(cells_w) {
            // Per-column hash: each stream has its own speed and offset.
            let hash = ((cx as f32 * 12.9898).sin() * 43_758.547).fract().abs();
            let speed = 1.6 + hash * 1.2;
            for cy in 0..cells_h {
                let c = if lip && cy == 0 {
                    '▀' // the sheet of water curling over the ledge
                } else if pool && cy == cells_h - 1 {
                    // Pool surface, with splashes kicking up where streams land.
                    let splash = (cx as f32 * 1.7 + ctx.time * 7.0 + hash * 6.0).sin();
                    if splash > 0.75 {
                        '▓'
                    } else {
                        '▄'
                    }
                } else {
                    // Falling streak: the pattern slides down as time runs.
                    let phase = (hash * 3.0 + ctx.time * speed - cy as f32 * 0.45).rem_euclid(1.0);
                    draw::SHADES[(phase * 5.0).floor().clamp(1.0, 4.0) as usize]
                };
                draw::glyph(grid, cx, cy, c);
            }
        }
        Ok(())
    }
}

struct DoubleEnded;
impl ProgressStyle for DoubleEnded {
    fn name(&self) -> &str {
        "double-ended"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Bar fills simultaneously from both ends toward the centre using smooth hbar logic"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w == 0 || cells_h == 0 {
            return Ok(());
        }
        // Each side fills half the width.
        let half = cells_w as f32 / 2.0;
        // Eighths of total half-width filled.
        let eighths_total = (ctx.eased * half * 8.0).round() as usize;
        let full_cells = (eighths_total / 8).min(cells_w / 2);
        let rem_eighths = eighths_total % 8;

        for cy in 0..cells_h {
            // Left side: fill from left edge inward.
            for cx in 0..full_cells.min(cells_w) {
                draw::glyph(grid, cx, cy, '█');
            }
            // Left partial edge cell.
            if rem_eighths > 0 && full_cells < cells_w {
                draw::glyph(grid, full_cells, cy, draw::H_BLOCKS[rem_eighths]);
            }

            // Right side: fill from right edge inward (mirror).
            let r_start = cells_w.saturating_sub(full_cells);
            for cx in r_start..cells_w {
                draw::glyph(grid, cx, cy, '█');
            }
            // Right partial edge cell (mirrored — use reverse H_BLOCK or full block).
            if rem_eighths > 0 && r_start > 0 {
                // Mirror: the partial cell is at r_start - 1.
                let pcx = r_start.saturating_sub(1);
                // Ensure no overlap with left fill.
                if pcx > full_cells {
                    draw::glyph(grid, pcx, cy, draw::H_BLOCKS[rem_eighths]);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 12. gradient-meter — smooth hbar over a shade backdrop for the track
// ---------------------------------------------------------------------------

struct GradientMeter;
impl ProgressStyle for GradientMeter {
    fn name(&self) -> &str {
        "gradient-meter"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Smooth hbar fill combined with a ░ shade backdrop on the unfilled track"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        // First, lay down the track shade in every cell.
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                draw::shade(grid, cx, cy, 1); // ░ track
            }
        }
        // Then overwrite the filled portion with the smooth hbar (all rows).
        // We replicate hbar manually so it overwrites shade with full/partial blocks.
        let eighths_total = (ctx.eased * cells_w as f32 * 8.0).round() as usize;
        let full_cells = (eighths_total / 8).min(cells_w);
        let rem_eighths = eighths_total % 8;
        for cy in 0..cells_h {
            for cx in 0..full_cells {
                draw::glyph(grid, cx, cy, '█');
            }
            if rem_eighths > 0 && full_cells < cells_w {
                draw::glyph(grid, full_cells, cy, draw::H_BLOCKS[rem_eighths]);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 13. battery — rounded cell with a terminal nub, charging left → right
// ---------------------------------------------------------------------------

struct Battery;
impl ProgressStyle for Battery {
    fn name(&self) -> &str {
        "battery"
    }
    fn theme(&self) -> &str {
        "blocks"
    }
    fn describe(&self) -> &str {
        "Battery cell: a rounded ╭─╮ casing with a ▌ terminal nub, filled with \
         eighth-precise █ charge; a ▓ charging ripple runs through the charge \
         and the fill blinks while the level is low"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (cells_w, cells_h) = grid.dimensions();
        if cells_w < 4 || cells_h == 0 {
            draw::hbar(grid, 0, ctx.eased);
            return Ok(());
        }
        // Last column is the terminal nub; the casing takes the rest.
        let body_w = cells_w - 1;
        let boxed = cells_h >= 3;
        let (row0, row1) = if boxed {
            (1, cells_h - 1)
        } else {
            (0, cells_h)
        };
        if boxed {
            draw::glyph(grid, 0, 0, '╭');
            draw::glyph(grid, body_w - 1, 0, '╮');
            draw::glyph(grid, 0, cells_h - 1, '╰');
            draw::glyph(grid, body_w - 1, cells_h - 1, '╯');
            for x in 1..body_w - 1 {
                draw::glyph(grid, x, 0, '─');
                draw::glyph(grid, x, cells_h - 1, '─');
            }
        }
        for y in row0..row1 {
            draw::glyph(grid, 0, y, '│');
            draw::glyph(grid, body_w - 1, y, '│');
        }
        // Nub: the middle rows only (all rows on a 1–2 row grid).
        let (n0, n1) = if cells_h >= 4 {
            (cells_h / 2 - 1, cells_h / 2 + 1)
        } else if boxed {
            (1, 2)
        } else {
            (0, cells_h)
        };
        for y in n0..n1 {
            draw::glyph(grid, cells_w - 1, y, '▌');
        }

        // Charge: eighth-precise fill of the interior.
        let inner = body_w.saturating_sub(2);
        let eased = ctx.eased.clamp(0.0, 1.0);
        let eighths = (eased * (inner * 8) as f32).round() as usize;
        let full = (eighths / 8).min(inner);
        let rem = eighths % 8;
        // Low charge blinks; otherwise a ripple sweeps through the charge.
        let blink_off = eased < 0.2 && (ctx.time * 4.0).sin() < 0.0;
        let ripple = ((ctx.time * 0.9).fract() * (full + 4) as f32) as usize;
        for y in row0..row1 {
            for i in 0..full {
                let c = if blink_off {
                    '▒'
                } else if i + 1 == ripple || i == ripple {
                    '▓'
                } else {
                    '█'
                };
                draw::glyph(grid, 1 + i, y, c);
            }
            if rem > 0 && full < inner && !blink_off {
                draw::glyph(grid, 1 + full, y, draw::H_BLOCKS[rem]);
            }
        }
        Ok(())
    }
}
