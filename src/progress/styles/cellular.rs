//! Cellular-automata progress bars.
//!
//! Every style in this module is a pure function of `(ctx.eased, ctx.time)` —
//! no mutable state is stored between frames. The CA state at each frame is
//! recomputed from a fixed initial condition, with the number of generations
//! evolved being a deterministic function of `eased` (for progress) and/or
//! `time` (for animation).
//!
//! Included algorithms:
//! - Wolfram elementary 1D CA (Rule 30, Rule 90, Rule 110, Rule 184, Rule 54, Rule 150)
//! - Conway's Game of Life (glider / r-pentomino seeds)
//! - Brian's Brain (3-state excitable CA)
//! - Langton's Ant
//! - Gray-Scott reaction-diffusion (spot regime, steady state rendered analytically)
//! - Griffeath cyclic CA (spiral demons)
//! - Wireworld clock (electron loop feeding a wire)
//! - Forest fire model

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};

// ---------------------------------------------------------------------------
// Deterministic hash (no external crates)
// ---------------------------------------------------------------------------

#[inline]
fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2_654_435_761);
    x ^= x >> 15;
    x.wrapping_mul(2_246_822_519)
}

#[inline]
fn hash_f(n: u32) -> f32 {
    (hash(n) % 1000) as f32 / 1000.0
}

/// Mix two seeds into a hash (useful for 2D coordinates).
#[inline]
fn hash2(x: u32, y: u32) -> u32 {
    hash(x.wrapping_mul(1_000_003).wrapping_add(y))
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Theme tint — petri-dish green.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(96, 226, 130);
const TINT_END: Color = Color::rgb(24, 128, 88);

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

/// All styles in the `cellular` theme.
///
/// Returns one `Box<dyn ProgressStyle>` per distinct cellular-automaton style.
/// Every style's `theme()` returns `"cellular"` and every `name()` is unique
/// within this theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(WolframRule { rule: 30 })),
        Box::new(Tinted(WolframRule { rule: 90 })),
        Box::new(Tinted(WolframRule { rule: 110 })),
        Box::new(Tinted(WolframRule { rule: 184 })),
        Box::new(Tinted(WolframRule { rule: 54 })),
        Box::new(Tinted(WolframRule { rule: 150 })),
        Box::new(Tinted(GameOfLife {
            seed: GoLSeed::Glider,
        })),
        Box::new(Tinted(GameOfLife {
            seed: GoLSeed::RPentomino,
        })),
        Box::new(Tinted(BriansBrain)),
        Box::new(Tinted(LangtonsAnt)),
        Box::new(Tinted(CyclicCA)),
        Box::new(Tinted(Wireworld)),
        Box::new(Tinted(GrayScott)),
        Box::new(Tinted(ForestFire)),
    ]
}

// ===========================================================================
// 1–6: Wolfram Elementary 1D CA (Rules 30, 90, 110, 184, 54, 150)
// ===========================================================================
//
// Layout: the dot grid is treated as a 1D tape (width) evolving over time
// (height). Row 0 is the initial condition (single centre-cell alive). Each
// subsequent row is one generation. We reveal rows top-to-bottom up to
// `ceil(eased * h)` rows. Time shifts a horizontal scroll offset so the
// pattern slowly drifts when the bar is held at a fixed progress.

struct WolframRule {
    rule: u8,
}

impl WolframRule {
    /// Apply one generation of the elementary CA rule.
    /// `row` must be length `w`; returns a new row of the same length.
    fn step(row: &[u8], rule: u8) -> Vec<u8> {
        let w = row.len();
        (0..w)
            .map(|i| {
                let l = if i == 0 { row[w - 1] } else { row[i - 1] };
                let c = row[i];
                let r = if i + 1 == w { row[0] } else { row[i + 1] };
                let pattern = (l << 2) | (c << 1) | r;
                (rule >> pattern) & 1
            })
            .collect()
    }
}

impl ProgressStyle for WolframRule {
    fn name(&self) -> &str {
        match self.rule {
            30 => "ca-rule30",
            90 => "ca-rule90",
            110 => "ca-rule110",
            184 => "ca-rule184",
            54 => "ca-rule54",
            150 => "ca-rule150",
            _ => "ca-wolfram",
        }
    }

    fn theme(&self) -> &str {
        "cellular"
    }

    fn describe(&self) -> &str {
        match self.rule {
            30 => "Rule 30: chaotic braille texture — Wolfram's fractal entropy engine",
            90 => "Rule 90: Sierpinski triangle — XOR diffusion fills the bar row by row",
            110 => "Rule 110: Turing-complete edge of chaos — complex local structures emerge",
            184 => "Rule 184: traffic flow CA — particles drift rightward with eased density",
            54 => "Rule 54: nested gliders — quasi-periodic wave fronts cascade down the bar",
            150 => "Rule 150: additive XOR diffusion — Pascal's triangle mod 2 in braille",
            _ => "Wolfram elementary CA",
        }
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Rows to reveal: 0 = empty, h = fully revealed.
        let reveal = (ctx.eased * h as f32).ceil() as usize;
        // Slow horizontal drift driven by time (wraps around tape). Rule 184
        // laps the tape exactly once per 4 s loop so the traffic flows
        // seamlessly; the other rules keep their subtle drift.
        let scroll = if self.rule == 184 {
            ((ctx.time * 0.25).fract() * w as f32) as usize % w
        } else {
            (ctx.time * 0.3) as usize % w.max(1)
        };

        // Seed: single live cell at the centre — except Rule 184 (traffic
        // flow), where a lone car is invisible: seed a whole rush hour whose
        // density rises with eased, so jams condense as the bar fills.
        let mut row = vec![0u8; w];
        if self.rule == 184 {
            let density = 0.30 + 0.45 * ctx.eased;
            for (x, cell) in row.iter_mut().enumerate() {
                if hash_f(x as u32 * 3 + 11) < density {
                    *cell = 1;
                }
            }
        } else {
            row[w / 2] = 1;
        }

        for gen in 0..reveal.min(h) {
            // Draw this generation into dot row `gen`.
            for x in 0..w {
                let sx = (x + scroll) % w;
                if row[sx] == 1 {
                    draw::dot(grid, x, gen);
                }
            }
            row = Self::step(&row, self.rule);
        }

        Ok(())
    }
}

// ===========================================================================
// 7–8: Conway's Game of Life
// ===========================================================================
//
// We run GoL on a cell grid of `w × h` dot-pixels (not braille cells).
// Seed is placed at a canonical position. Generations = floor(time * speed).
// `eased` gates how much of the grid is rendered (left-to-right reveal mask).

#[derive(Clone, Copy)]
enum GoLSeed {
    Glider,
    RPentomino,
}

struct GameOfLife {
    seed: GoLSeed,
}

impl GameOfLife {
    fn make_board(w: usize, h: usize, seed: GoLSeed) -> Vec<Vec<bool>> {
        let mut board = vec![vec![false; w]; h];
        match seed {
            GoLSeed::Glider => {
                // Standard glider pattern, placed top-left.
                // . X .
                // . . X
                // X X X
                let patterns: &[(usize, usize)] = &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
                for &(px, py) in patterns {
                    if px < w && py < h {
                        board[py][px] = true;
                    }
                }
            }
            GoLSeed::RPentomino => {
                // r-pentomino: . X X / X X . / . X .  — centred.
                let cx = w / 2;
                let cy = h / 2;
                let patterns: &[(i32, i32)] = &[(1, -1), (2, -1), (0, 0), (1, 0), (1, 1)];
                for &(dx, dy) in patterns {
                    let px = cx as i32 + dx;
                    let py = cy as i32 + dy;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        board[py as usize][px as usize] = true;
                    }
                }
            }
        }
        board
    }

    fn step(board: &[Vec<bool>]) -> Vec<Vec<bool>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let mut next = vec![vec![false; w]; h];
        for y in 0..h {
            for x in 0..w {
                let mut n = 0u8;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                        let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                        if board[ny][nx] {
                            n += 1;
                        }
                    }
                }
                next[y][x] = if board[y][x] {
                    n == 2 || n == 3
                } else {
                    n == 3
                };
            }
        }
        next
    }
}

impl ProgressStyle for GameOfLife {
    fn name(&self) -> &str {
        match self.seed {
            GoLSeed::Glider => "ca-gol-glider",
            GoLSeed::RPentomino => "ca-gol-rpentomino",
        }
    }

    fn theme(&self) -> &str {
        "cellular"
    }

    fn describe(&self) -> &str {
        match self.seed {
            GoLSeed::Glider =>
                "Game of Life glider — a diagonal spaceship animates across the bar as progress fills",
            GoLSeed::RPentomino =>
                "Game of Life r-pentomino — explosive chaotic growth, progress gates the reveal column",
        }
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Speed: glider moves ~1 cell/5 gens; r-pentomino expands faster.
        let speed = match self.seed {
            GoLSeed::Glider => 4.0f32,
            GoLSeed::RPentomino => 8.0f32,
        };
        let gens = (ctx.time * speed) as usize;
        // Cap at a reasonable bound to prevent runaway cost.
        let gens = gens.min(500);

        let mut board = Self::make_board(w, h, self.seed);
        for _ in 0..gens {
            board = Self::step(&board);
        }

        // eased controls a left-to-right reveal column.
        let reveal_x = (ctx.eased * w as f32).round() as usize;

        for (y, row) in board.iter().enumerate().take(h) {
            for x in 0..reveal_x.min(w) {
                if x < row.len() && row[x] {
                    draw::dot(grid, x, y);
                }
            }
        }

        // A sparse live population can leave the bar unreadable, so a dotted
        // baseline plus an edge tick carries the progress reading.
        for x in (0..reveal_x.min(w)).step_by(3) {
            draw::dot(grid, x, h - 1);
        }
        let edge = reveal_x.min(w.saturating_sub(1));
        draw::vline(grid, edge, h.saturating_sub(3), h - 1);

        Ok(())
    }
}

// ===========================================================================
// 9: Brian's Brain
// ===========================================================================
//
// 3-state excitable CA: OFF(0), FIRING(1), REFRACTORY(2).
// A cell fires if exactly 2 FIRING neighbours; fires→refractory→off.
// Seeded with a deterministic hash pattern. Generations driven by time.

struct BriansBrain;

impl BriansBrain {
    fn initial(w: usize, h: usize) -> Vec<Vec<u8>> {
        let mut board = vec![vec![0u8; w]; h];
        // Scatter some firing seeds using the hash function.
        for (y, row) in board.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let v = hash2(x as u32, y as u32) % 5;
                *cell = u8::from(v == 0);
            }
        }
        board
    }

    fn step(board: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let mut next = vec![vec![0u8; w]; h];
        for y in 0..h {
            for x in 0..w {
                let state = board[y][x];
                next[y][x] = match state {
                    1 => 2, // firing → refractory
                    2 => 0, // refractory → off
                    _ => {
                        // off → firing if exactly 2 firing neighbours
                        let mut n = 0u8;
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                                let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                                if board[ny][nx] == 1 {
                                    n += 1;
                                }
                            }
                        }
                        u8::from(n == 2)
                    }
                };
            }
        }
        next
    }
}

impl ProgressStyle for BriansBrain {
    fn name(&self) -> &str {
        "ca-brians-brain"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Brian's Brain: 3-state excitable CA — firing cells with refractory tails swarm as gliders, revealed left-to-right by progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        let gens = ((ctx.time * 6.0) as usize).min(300);
        let mut board = Self::initial(w, h);
        for _ in 0..gens {
            board = Self::step(&board);
        }

        // Progress gates a left-to-right reveal (a few columns always show).
        // Firing cells are solid; refractory cells — the "tails" that make
        // Brian's Brain gliders travel — are drawn at half density behind them.
        let reveal_x = ((ctx.eased * w as f32) as usize).max(6).min(w);
        for (y, row) in board.iter().enumerate().take(h) {
            for (x, &cell) in row.iter().enumerate().take(reveal_x) {
                if cell == 1 || (cell == 2 && (x + y) % 2 == 0) {
                    draw::dot(grid, x, y);
                }
            }
        }
        if reveal_x < w {
            for y in (0..h).step_by(2) {
                draw::dot(grid, reveal_x, y);
            }
        }

        Ok(())
    }
}

// ===========================================================================
// 10: Langton's Ant
// ===========================================================================
//
// Ant turns right on white (0), left on black (1), flips cell, moves forward.
// We run `eased * MAX_STEPS` steps from a blank grid, then render the trail.
// `time` adds a small periodic "wander" offset to produce animation.

struct LangtonsAnt;

impl LangtonsAnt {
    const MAX_STEPS: usize = 2000;
}

impl ProgressStyle for LangtonsAnt {
    fn name(&self) -> &str {
        "ca-langtons-ant"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Langton's Ant: deterministic trail grows with progress — the highway emerges near 10,000 steps"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        let steps = (ctx.eased * Self::MAX_STEPS as f32) as usize;
        // Time animates a slow starting offset (cycles through small offsets).
        let time_offset = (ctx.time * 0.5) as usize % 8;
        let steps = steps.saturating_add(time_offset).min(Self::MAX_STEPS + 8);

        let mut cells = vec![0u8; w * h];
        // Ant starts at centre.
        let mut ax = (w / 2) as i32;
        let mut ay = (h / 2) as i32;
        // Direction: 0=up, 1=right, 2=down, 3=left.
        let mut dir = 0i32;

        for _ in 0..steps {
            // Clamp ant to grid (toroidal wrap).
            ax = ax.rem_euclid(w as i32);
            ay = ay.rem_euclid(h as i32);
            let idx = ay as usize * w + ax as usize;
            let c = cells[idx];
            if c == 0 {
                dir = (dir + 1).rem_euclid(4); // turn right
                cells[idx] = 1;
            } else {
                dir = (dir - 1).rem_euclid(4); // turn left
                cells[idx] = 0;
            }
            // Move forward.
            match dir {
                0 => ay -= 1,
                1 => ax += 1,
                2 => ay += 1,
                _ => ax -= 1,
            }
        }

        for y in 0..h {
            for x in 0..w {
                if cells[y * w + x] == 1 {
                    draw::dot(grid, x, y);
                }
            }
        }

        Ok(())
    }
}

// ===========================================================================
// 11: Cyclic / Rock-Paper-Scissors CA
// ===========================================================================
//
// Griffeath's cyclic CA: each cell holds a state in [0, N). A cell advances
// (+1 mod N) if at least one of its 4 von Neumann neighbours is already in
// the next state. From random soup, self-sustaining spiral "demons" emerge
// and sweep the field with rotating wave bands. N=8 states.

struct CyclicCA;

impl CyclicCA {
    const N_STATES: u8 = 8;
    const THRESHOLD: usize = 1; // neighbours needed to advance

    fn initial(w: usize, h: usize, density_seed: u32) -> Vec<Vec<u8>> {
        let n = Self::N_STATES;
        (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| {
                        // High hash bits: the low bits of this hash are weak.
                        let v = hash(hash2(x as u32, y as u32) ^ density_seed) >> 16;
                        (v % n as u32) as u8
                    })
                    .collect()
            })
            .collect()
    }

    fn step(board: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let h = board.len();
        let w = if h == 0 { 0 } else { board[0].len() };
        let n = CyclicCA::N_STATES;
        let thresh = CyclicCA::THRESHOLD;
        let mut next = board.to_vec();
        for y in 0..h {
            for x in 0..w {
                let cur = board[y][x];
                let target = (cur + 1) % n;
                let mut count = 0usize;
                for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                    let nx = (x as i32 + dx).rem_euclid(w as i32) as usize;
                    let ny = (y as i32 + dy).rem_euclid(h as i32) as usize;
                    if board[ny][nx] == target {
                        count += 1;
                    }
                }
                if count >= thresh {
                    next[y][x] = target;
                }
            }
        }
        next
    }
}

impl ProgressStyle for CyclicCA {
    fn name(&self) -> &str {
        "ca-cyclic"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Griffeath cyclic CA: 8 states, each eats its predecessor — spiral demons sweep rotating wave bands across a field revealed by progress"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);

        // Wave density is roughly constant at any generation count, so the
        // generation count alone cannot carry the progress reading. Run the
        // waves on time and gate the field behind a left-to-right reveal.
        // After the warm-up the demons dominate and the field cycles with
        // period N, so looping over a multiple of N bounds the cost.
        let period = Self::N_STATES as usize * 12;
        let gens = 48 + (ctx.time.max(0.0) * 6.0) as usize % period;
        let seed = 7u32;
        let mut board = Self::initial(w, h, seed);
        for _ in 0..gens {
            board = Self::step(&board);
        }

        let reveal_x = (ctx.eased * w as f32).round() as usize;

        // Draw cells that are in "high" states (upper half of N_STATES range).
        let half = Self::N_STATES / 2;
        for (y, row) in board.iter().enumerate().take(h) {
            for x in 0..reveal_x.min(w) {
                if x < row.len() && row[x] >= half {
                    draw::dot(grid, x, y);
                }
            }
        }
        let edge = reveal_x.min(w.saturating_sub(1));
        draw::vline(grid, edge, 0, h - 1);

        Ok(())
    }
}

// ===========================================================================
// 12: Wireworld
// ===========================================================================
//
// 4 states: EMPTY, CONDUCTOR, ELECTRON_HEAD, ELECTRON_TAIL.
// Rules: head → tail, tail → conductor, conductor → head if exactly 1 or 2 of
// its 8 neighbours are heads. On a closed loop a single head/tail pair
// circulates one cell per generation; tapped by a wire, the loop is the
// classic Wireworld *clock*, emitting one electron down the wire every lap.
// On a one-cell-wide wire the rule moves each electron exactly one cell per
// generation, so the state at generation `g` is computed in closed form.
// The wire is laid left-to-right as progress grows; time drives the clock.
// Wire heads are drawn as 3x3 blocks with a split tail behind them (the loop
// head as a 3-dot bar), so the pulses read even without colour.

struct Wireworld;

impl ProgressStyle for Wireworld {
    fn name(&self) -> &str {
        "ca-wireworld"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Wireworld clock: an electron circling a conductor loop fires pulses down a wire laid out to the progress point"
    }

    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 4 || h < 3 {
            // Too small for a circuit: just lay the wire.
            if w > 0 && h > 0 {
                let end = (ctx.eased * w as f32) as usize;
                draw::hline(grid, 0, end.min(w - 1), h / 2);
            }
            return Ok(());
        }
        let (wi, hi) = (w as i32, h as i32);
        let mid = hi / 2;
        // Clock loop: a rectangle at the left whose right side taps the wire.
        let half_h = (mid - 1).clamp(1, 3);
        let (lx0, lx1) = (1i32, 7i32.min(wi - 2));
        let (ly0, ly1) = (mid - half_h, mid + half_h);
        // Perimeter cells, clockwise from the top-left corner.
        let mut loop_cells: Vec<(i32, i32)> = Vec::new();
        for x in lx0..=lx1 {
            loop_cells.push((x, ly0));
        }
        for y in ly0 + 1..=ly1 {
            loop_cells.push((lx1, y));
        }
        for x in (lx0..lx1).rev() {
            loop_cells.push((x, ly1));
        }
        for y in (ly0 + 1..ly1).rev() {
            loop_cells.push((lx0, y));
        }
        let lap = loop_cells.len().max(1) as i32;
        for &(x, y) in &loop_cells {
            draw::dot_i(grid, x, y);
        }
        // Index of the tap (the loop cell on the wire row, right side).
        let tap = loop_cells
            .iter()
            .position(|&(x, y)| x == lx1 && y == mid)
            .unwrap_or(0) as i32;

        // Wire: laid from the loop to the progress point (visible stub at 0).
        let wire_x0 = lx1 + 1;
        let wire_end = ((ctx.eased * w as f32) as i32).max(wire_x0 + 6).min(wi - 1);
        draw::hline(grid, wire_x0 as usize, wire_end as usize, mid as usize);

        let g = (ctx.time.max(0.0) * 10.0) as i32;
        // Head marker: a 3-dot bar across the conductor.
        let head = |grid: &mut BrailleGrid, x: i32, y: i32, vertical_wire: bool| {
            draw::dot_i(grid, x, y);
            if vertical_wire {
                draw::dot_i(grid, x - 1, y);
                draw::dot_i(grid, x + 1, y);
            } else {
                draw::dot_i(grid, x, y - 1);
                draw::dot_i(grid, x, y + 1);
            }
        };
        // The circulating electron: head at loop position g.
        let hp = g % lap;
        let (hx, hy) = loop_cells[hp as usize];
        head(grid, hx, hy, hx == lx0 || hx == lx1);
        // Pulses on the wire: a head that passed the tap `d` gens ago sits at
        // wire_x0 + d - 1 — one pulse per lap.
        let mut d = (g - tap).rem_euclid(lap);
        while wire_x0 + d - 1 <= wire_end {
            let px = wire_x0 + d - 1;
            if px >= wire_x0 {
                // Head: a 3x3 block; tail: a bar right behind it.
                head(grid, px, mid, false);
                head(grid, px - 1, mid, false);
                if px > wire_x0 + 1 {
                    draw::dot_i(grid, px - 2, mid - 1);
                    draw::dot_i(grid, px - 2, mid + 1);
                }
            }
            d += lap;
        }
        Ok(())
    }
}

// ===========================================================================
// 13: Gray-Scott reaction-diffusion
// ===========================================================================
//
// Two chemicals u, v diffuse and react: u + 2v -> 3v. In the spot ("mitosis")
// regime the field settles into a hexagonal array of round v-spots, but that
// takes thousands of integration steps — far too many per frame. So the
// steady state is rendered analytically (three plane waves 120° apart sum to
// the hexagonal spot lattice) inside a colony that spreads with `eased`, with
// small freshly-nucleated spots at its rim.

struct GrayScott;
impl ProgressStyle for GrayScott {
    fn name(&self) -> &str {
        "ca-gray-scott"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Gray-Scott reaction-diffusion, spot (mitosis) regime: a colony of round Turing spots spreads from the centre as progress feeds it, new spots budding at the rim"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        // Spot lattice: three plane waves 120° apart sum to a hexagonal array
        // of peaks — the steady state the spot regime settles into.
        let k = std::f32::consts::TAU / 7.0;
        let dirs = [(1.0f32, 0.0f32), (-0.5, 0.866), (-0.5, -0.866)];
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        // Colony half-width: a visible seed colony at the start, the full bar
        // at completion. The front is a soft band where spots are still small.
        let radius = 5.0 + ctx.eased * (cx + 2.0);
        const FRONT: f32 = 5.0;
        let t = ctx.time;
        for y in 0..h {
            for x in 0..w {
                let (px, py) = (x as f32, y as f32);
                // Elliptical distance: the bar is wide, so spread mostly sideways.
                let d = ((px - cx).powi(2) + ((py - cy) * 2.5).powi(2)).sqrt();
                if d > radius {
                    continue;
                }
                let f: f32 = dirs
                    .iter()
                    .map(|&(ex, ey)| (k * (ex * px + ey * py)).cos())
                    .sum();
                // Spots breathe gently; spots in the front band are smaller
                // (higher threshold) — freshly nucleated, still growing.
                let grow = ((radius - d) / FRONT).clamp(0.0, 1.0);
                let thresh = 1.0 + 1.3 * (1.0 - grow) + 0.15 * (t * 1.7 + px * 0.09).sin();
                if f > thresh {
                    draw::dot(grid, x, y);
                }
            }
        }
        Ok(())
    }
}

// ===========================================================================
// 14: Forest-fire model
// ===========================================================================
//
// Trees are scattered by a fixed hash; a fire front sweeps left-to-right with
// `eased` (the burn extent = progress). Trees ahead of the front stand;
// trees at the front flicker as flames (animated by `time`); behind it is ash.

struct ForestFire;
impl ProgressStyle for ForestFire {
    fn name(&self) -> &str {
        "ca-forest-fire"
    }
    fn theme(&self) -> &str {
        "cellular"
    }
    fn describe(&self) -> &str {
        "Forest-fire CA: a flame front burns through scattered trees as progress advances"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        let w = w.max(1);
        let h = h.max(1);
        let front = (ctx.eased * w as f32) as i32;
        for y in 0..h {
            for x in 0..w {
                // Tree if the hash for this cell clears the density threshold.
                if hash_f((y * w + x) as u32 ^ 0x9e37) >= 0.55 {
                    continue;
                }
                let d = x as i32 - front;
                if d > 1 {
                    // Standing tree ahead of the fire.
                    draw::dot(grid, x, y);
                    if y > 0 && hash_f((x * 7 + y) as u32) > 0.5 {
                        draw::dot(grid, x, y - 1);
                    }
                } else if d >= -2 {
                    // Flame front: flicker upward with time.
                    let flick = ((ctx.time * 12.0 + (x + y) as f32).sin() * 2.0) as i32;
                    draw::dot_i(grid, x as i32, y as i32 - flick.abs());
                    draw::dot(grid, x, y);
                }
                // Behind the front (d < -2): ash, drawn empty.
            }
        }
        Ok(())
    }
}
