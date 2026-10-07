//! Number-theory progress bars.
//!
//! Twelve styles, each grounded in a concrete mathematical structure:
//! sieves, spirals, trees, triangles, histograms, and sequence geometry.
//! Every bar maps `ctx.eased` to "how many integers have been processed" and
//! `ctx.time` to a highlight or scan animation running independently of
//! progress. All arithmetic is bounded and cannot panic.

use super::super::draw;
use super::super::{BarContext, ProgressStyle};
use crate::{BrailleGrid, Color, DotmaxError};
use std::f32::consts::PI;

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

/// Trial-division primality test. Returns `false` for 0 and 1.
/// Bounded: for any n ≤ 10_000 this completes in a handful of μs.
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3u64;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

// ────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────

// ---------------------------------------------------------------------------
// Theme tint — teal into deep blue.
// ---------------------------------------------------------------------------

/// Gradient endpoints for this theme's signature tint.
const TINT_START: Color = Color::rgb(118, 224, 202);
const TINT_END: Color = Color::rgb(44, 132, 204);

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

/// All styles in the `numbertheory` theme.
pub fn styles() -> Vec<Box<dyn ProgressStyle>> {
    vec![
        Box::new(Tinted(Sieve)),
        Box::new(Tinted(UlamSpiral)),
        Box::new(Tinted(PrimeCounting)),
        Box::new(Tinted(Collatz)),
        Box::new(Tinted(FibonacciSpiral)),
        Box::new(Tinted(PascalMod)),
        Box::new(Tinted(TotientHistogram)),
        Box::new(Tinted(SternBrocot)),
        Box::new(Tinted(ContinuedFraction)),
        Box::new(Tinted(ModularCircle)),
        Box::new(Tinted(Recaman)),
        Box::new(Tinted(DigitalRoot)),
    ]
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Sieve of Eratosthenes
// ────────────────────────────────────────────────────────────────────────────

/// Sieve of Eratosthenes: integers 1..N laid left→right; composites are
/// crossed out one by one, primes remain lit. `eased` controls how many
/// integers are visible; `time` sweeps a highlight over the current sieve
/// multiple.
struct Sieve;
impl ProgressStyle for Sieve {
    fn name(&self) -> &str {
        "sieve-eratosthenes"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Sieve of Eratosthenes: integers cross composites, primes survive as lit dots"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // N = number of integers we lay across the width (at least 2).
        let n = w.clamp(2, 2000);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n);

        // Build sieve for 1..=n.
        let mut composite = vec![false; n + 1];
        composite[0] = true;
        if n >= 1 {
            composite[1] = true;
        }
        for p in 2..=n {
            if !composite[p] {
                let mut m = p * 2;
                while m <= n {
                    composite[m] = true;
                    m += p;
                }
            }
        }

        // Animated highlight: current sieve multiple being crossed.
        let highlight_p = {
            let p_frac = (ctx.time * 0.4).fract();
            // Pick a prime index from time.
            let prime_count = (2..=n).filter(|&k| !composite[k]).count().max(1);
            let pi = ((p_frac * prime_count as f32) as usize).min(prime_count.saturating_sub(1));
            (2..=n).filter(|&k| !composite[k]).nth(pi).unwrap_or(2)
        };

        // Draw: one dot-column per integer.
        for (k, &comp) in composite.iter().enumerate().skip(1).take(revealed) {
            let x = ((k - 1) * w / n).min(w.saturating_sub(1));
            let is_p = !comp;
            // Primes: full column; composites: half-height bottom tick.
            if is_p {
                draw::vline(grid, x, 0, h.saturating_sub(1));
                // Highlight the current sieve prime's multiples.
                if k > 1 && k % highlight_p == 0 {
                    // draw a top cap to distinguish the animated sweep target.
                    draw::dot(grid, x, 0);
                }
            } else {
                let tick = (h / 4).max(1);
                draw::vline(grid, x, h.saturating_sub(tick), h.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Ulam Spiral
// ────────────────────────────────────────────────────────────────────────────

/// Ulam spiral, stretched to the bar: the integers wind outward from a short
/// central run in a rectangular spiral that exactly tiles the bar, and only
/// the primes are lit, so the diagonal streaks of prime-rich quadratics show
/// up in every corner. `eased` reveals integers 1..N; `time` blinks the
/// spiral's growing head.
struct UlamSpiral;
impl ProgressStyle for UlamSpiral {
    fn name(&self) -> &str {
        "ulam-spiral"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Ulam spiral: integers wind outward from the centre in a rectangular spiral that tiles \
         the bar, only primes lit — the tell-tale diagonal streaks appear as progress unwinds \
         it, with a blinking head marking the newest integer"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        // Rectangular spiral: horizontal legs a+1, a+2, …; vertical legs 1, 2, …
        // After k full rings it spans (a + 2k) × 2k, so pick a = w − h.
        let a = w.saturating_sub(h) as i32;
        let x0 = (h / 2) as i32 - 1;
        let y0 = (h / 2) as i32;
        let n = (w * h).clamp(1, 6000);
        let revealed = ((ctx.eased.clamp(0.0, 1.0) * n as f32).round() as usize).min(n);

        let (mut x, mut y) = (x0, y0);
        let mut leg = 0i32; // leg index: even = horizontal, odd = vertical
        let mut left = a + 1;
        let mut head = (x, y);
        for i in 1..=revealed {
            if is_prime(i as u64) {
                draw::dot_i(grid, x, y);
            }
            head = (x, y);
            // Advance along the current leg; directions: right, up, left, down.
            let dir = leg % 4;
            match dir {
                0 => x += 1,
                1 => y -= 1,
                2 => x -= 1,
                _ => y += 1,
            }
            left -= 1;
            if left <= 0 {
                leg += 1;
                let k = leg / 2 + 1;
                left = if leg % 2 == 0 { a + k } else { k };
            }
        }
        if revealed > 0 && revealed < n && (ctx.time * 4.0).sin() > -0.2 {
            draw::fill_rect(
                grid,
                (head.0 - 1).max(0) as usize,
                (head.1 - 1).max(0) as usize,
                2,
                2,
            );
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Prime counting function π(x)
// ────────────────────────────────────────────────────────────────────────────

/// Prime counting function π(x): a curve showing how many primes are ≤ x,
/// displayed as a rising step graph filling left to right with `eased`.
/// `time` scrolls a thin vertical scan line.
struct PrimeCounting;
impl ProgressStyle for PrimeCounting {
    fn name(&self) -> &str {
        "prime-counting"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "π(x) prime-counting function: a rising step curve filling as primes accumulate"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n = w.clamp(2, 3000);
        let revealed_x = ((ctx.eased * n as f32).round() as usize).min(n);

        // Precompute π(k) for k in 1..=n.
        let mut pi = 0usize;
        let mut prime_counts = vec![0usize; n + 1];
        for (k, count) in prime_counts.iter_mut().enumerate().skip(1) {
            if is_prime(k as u64) {
                pi += 1;
            }
            *count = pi;
        }
        let pi_max = prime_counts[n].max(1);

        // Draw the step curve up to revealed_x.
        for (k, &count) in prime_counts.iter().enumerate().skip(1).take(revealed_x) {
            let x = ((k - 1) * w / n).min(w.saturating_sub(1));
            // Map prime count to y (bottom = 0 primes, top = pi_max).
            let bar_h = (count * h / pi_max).min(h);
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
        }

        // Animated scan line.
        let scan_x = ((ctx.time * 0.3).fract() * w as f32) as usize;
        if scan_x < w {
            draw::vline(grid, scan_x, 0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Collatz (3n+1) trajectories
// ────────────────────────────────────────────────────────────────────────────

/// Collatz conjecture: for each seed n (swept by `eased` over 1..N), plot
/// the stopping trajectory (n→n/2 if even, 3n+1 if odd). Height encodes
/// the value normalised to the peak value in the window. `time` selects
/// which trajectory is currently highlighted.
struct Collatz;
impl ProgressStyle for Collatz {
    fn name(&self) -> &str {
        "collatz"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Collatz (3n+1): seeds swept left→right, trajectory height is stopping sequence depth"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        // Number of seeds = width in dots, max 500.
        let n_seeds = w.clamp(1, 500);
        let revealed = ((ctx.eased * n_seeds as f32).round() as usize).min(n_seeds);

        // Collatz stopping time for seed k.
        fn stopping_time(mut k: u64) -> u64 {
            let mut count = 0u64;
            while k != 1 && count < 10_000 {
                k = if k % 2 == 0 { k / 2 } else { 3 * k + 1 };
                count += 1;
            }
            count
        }

        // Collect stopping times for seeds 1..=n_seeds.
        let times: Vec<u64> = (1..=n_seeds).map(|k| stopping_time(k as u64)).collect();
        let max_t = times.iter().copied().max().unwrap_or(1).max(1);

        // Highlight column from time.
        let hi_x = ((ctx.time * 0.7).fract() * revealed as f32) as usize;

        for (i, &t) in times.iter().enumerate().take(revealed) {
            let x = (i * w / n_seeds).min(w.saturating_sub(1));
            let bar_h = ((t as f32 / max_t as f32) * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
            // Extra top dot for highlight column.
            if i == hi_x && y0 > 0 {
                draw::dot(grid, x, y0.saturating_sub(1));
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Fibonacci / Golden spiral
// ────────────────────────────────────────────────────────────────────────────

/// Fibonacci golden spiral: squares whose sides are Fibonacci numbers, each
/// crossed by a quarter-circle arc. `eased` builds the tiling square by square;
/// `time` runs a bead along finished spirals.
struct FibonacciSpiral;

/// A point in Fibonacci-tiling units.
type Pt = (f32, f32);

/// The Fibonacci tiling used by [`FibonacciSpiral`]: squares of side
/// 1, 1, 2, 3, 5, 8 (top-left corner, side) filling a 13×8 golden-ish
/// rectangle spanning x ∈ [-3, 10], y ∈ [-5, 3] (y down), and for each square
/// the quarter arc of the spiral through it as (centre, start corner).
const FIB_SQUARES: [(Pt, f32, Pt, Pt); 6] = [
    ((0.0, 0.0), 1.0, (1.0, 1.0), (0.0, 1.0)),
    ((1.0, 0.0), 1.0, (1.0, 1.0), (1.0, 0.0)),
    ((0.0, 1.0), 2.0, (0.0, 1.0), (2.0, 1.0)),
    ((-3.0, 0.0), 3.0, (0.0, 0.0), (0.0, 3.0)),
    ((-3.0, -5.0), 5.0, (2.0, 0.0), (-3.0, 0.0)),
    ((2.0, -5.0), 8.0, (2.0, 3.0), (2.0, -5.0)),
];

impl ProgressStyle for FibonacciSpiral {
    fn name(&self) -> &str {
        "fibonacci-spiral"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Fibonacci spiral: squares of side 1, 1, 2, 3, 5, 8 tile a 13×8 rectangle and a \
         quarter arc sweeps through each; three tilings build square by square as progress \
         rises, and a bead rides the finished spirals with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw < 2 || dh < 2 {
            return Ok(());
        }
        let n = if dw >= dh * 4 { 3 } else { 1 };
        let pw = dw as f32 / n as f32;
        let s = ((dh as f32 - 1.0) / 8.0).min((pw - 2.0) / 13.0).max(0.1);
        for k in 0..n {
            let local = (ctx.eased * n as f32 - k as f32).clamp(0.0, 1.0);
            // Panel origin so the 13×8 rectangle is centred in its slot.
            let ox = pw * (k as f32 + 0.5) - 6.5 * s + 3.0 * s;
            let oy = (dh as f32 - 1.0 - 8.0 * s) * 0.5 + 5.0 * s;
            let at = |x: f32, y: f32| ((ox + x * s).round() as i32, (oy + y * s).round() as i32);
            let span = local * 6.0;
            let bead = if local >= 1.0 {
                Some((ctx.time * 0.8 + k as f32).rem_euclid(6.0))
            } else {
                None
            };
            for (i, &((x, y), side, (cx, cy), (px, py))) in FIB_SQUARES.iter().enumerate() {
                let part = (span - i as f32).clamp(0.0, 1.0);
                if part <= 0.0 {
                    break;
                }
                // Square outline.
                let (x0, y0) = at(x, y);
                let (x1, y1) = at(x + side, y + side);
                for xx in x0..=x1 {
                    draw::dot_i(grid, xx, y0);
                    draw::dot_i(grid, xx, y1);
                }
                for yy in y0..=y1 {
                    draw::dot_i(grid, x0, yy);
                    draw::dot_i(grid, x1, yy);
                }
                // Quarter arc from the start corner, swept by `part`.
                let a0 = (py - cy).atan2(px - cx);
                // The end corner is diagonally opposite the start corner.
                let (qx, qy) = (2.0 * x + side - px, 2.0 * y + side - py);
                let mut da = (qy - cy).atan2(qx - cx) - a0;
                if da > PI {
                    da -= 2.0 * PI;
                } else if da < -PI {
                    da += 2.0 * PI;
                }
                let steps = (side * s * 2.0).ceil().max(4.0) as u32;
                for j in 0..=steps {
                    let a = a0 + da * part * j as f32 / steps as f32;
                    let (dx, dy) = at(cx + side * a.cos(), cy + side * a.sin());
                    draw::dot_i(grid, dx, dy);
                }
                if let Some(b) = bead {
                    if (b - i as f32) >= 0.0 && (b - i as f32) < 1.0 {
                        let a = a0 + da * (b - i as f32);
                        let (dx, dy) = at(cx + side * a.cos(), cy + side * a.sin());
                        draw::dot_i(grid, dx + 1, dy);
                        draw::dot_i(grid, dx, dy + 1);
                        draw::dot_i(grid, dx - 1, dy);
                        draw::dot_i(grid, dx, dy - 1);
                    }
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Pascal's triangle mod p
// ────────────────────────────────────────────────────────────────────────────

/// Pascal's triangle mod p, on its side: rows revealed = `eased × width`.
/// Mod 2 yields Sierpinski's gasket; `time` cycles through mod 2, 3, 5 to
/// show different self-similar patterns.
struct PascalMod;

/// `C(n, k) mod p != 0`, by Lucas' theorem: true iff every base-`p` digit of
/// `k` is ≤ the matching digit of `n`.
fn binom_nonzero_mod(mut n: u32, mut k: u32, p: u32) -> bool {
    if k > n {
        return false;
    }
    while k > 0 {
        if k % p > n % p {
            return false;
        }
        n /= p;
        k /= p;
    }
    true
}

impl ProgressStyle for PascalMod {
    fn name(&self) -> &str {
        "pascal-mod"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Pascal's triangle mod p laid on its side: row n runs along the bar, the middle \
         binomials C(n, k) stack up the height, and every entry not divisible by p is lit — \
         nested triangular holes open up as progress adds rows; p cycles 2, 3, 5 with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (dw, dh) = draw::dot_dims(grid);
        if dw == 0 || dh == 0 {
            return Ok(());
        }
        let p = [2u32, 3, 5][((ctx.time.max(0.0) / 6.0) as usize) % 3];
        let rows = (ctx.eased * dw as f32).ceil() as usize;
        let mid = dh as i32 / 2;
        for n in 0..rows.min(dw) {
            let nn = n as u32;
            for y in 0..dh as i32 {
                // Middle of row n sits on the bar's centre line.
                let k = (nn as i32 + 1) / 2 + (y - mid);
                if k >= 0 && binom_nonzero_mod(nn, k as u32, p) {
                    draw::dot(grid, n, y as usize);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Euler totient φ(n) histogram
// ────────────────────────────────────────────────────────────────────────────

/// Euler totient φ(n) histogram: bar height at column n is φ(n)/n, revealing
/// left to right as `eased` grows. Primes peek at the top (φ(p)=p−1≈p).
struct TotientHistogram;
impl ProgressStyle for TotientHistogram {
    fn name(&self) -> &str {
        "totient-histogram"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Euler totient φ(n): columns show φ(n)/n — primes spike to the top, composites sag"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let n = w.clamp(2, 2000);
        let revealed = ((ctx.eased * n as f32).round() as usize).min(n).max(1);

        // Compute totient for k in 2..=revealed.
        fn totient(n: usize) -> usize {
            if n == 0 {
                return 0;
            }
            // Euler's product formula via trial factoring.
            let mut phi = n;
            let mut temp = n;
            let mut p = 2;
            while p * p <= temp {
                if temp % p == 0 {
                    while temp % p == 0 {
                        temp /= p;
                    }
                    phi = phi - phi / p;
                }
                p += 1;
            }
            if temp > 1 {
                phi = phi - phi / temp;
            }
            phi
        }

        for k in 2..=revealed {
            let x = ((k - 2) * w / (n.saturating_sub(1).max(1))).min(w.saturating_sub(1));
            let phi_k = totient(k);
            // Ratio φ(k)/k ∈ (0, 1].
            let ratio = phi_k as f32 / k as f32;
            let bar_h = (ratio * h as f32).round() as usize;
            let y0 = h.saturating_sub(bar_h);
            draw::vline(grid, x, y0, h.saturating_sub(1));
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Stern–Brocot tree (mediant binary tree)
// ────────────────────────────────────────────────────────────────────────────

/// Stern–Brocot tree: every positive rational appears exactly once as the
/// mediant (a+c)/(b+d) of its two neighbours. Nodes sit at their value
/// in order along their row (the tree is a binary search tree) on rows by depth, joined to their
/// parents. `eased` grows the tree level by level, left to right; `time`
/// walks a blinking path down the tree towards the golden ratio.
struct SternBrocot;
impl ProgressStyle for SternBrocot {
    fn name(&self) -> &str {
        "stern-brocot"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Stern–Brocot tree: 1/1 at the top, each node's children the mediants with its \
         neighbours (1/2, 2/1, 1/3, 2/3, 3/2, 3/1…), in sorted order — the binary tree of all \
         positive rationals grows level by level with progress while a walker blinks down \
         the path that converges on the golden ratio"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let levels = (h / 3).clamp(1, 7);
        let row = |d: usize| -> i32 {
            if levels <= 1 {
                0
            } else {
                (d as f32 * (h as f32 - 1.0) / (levels - 1) as f32).round() as i32
            }
        };
        // The tree is a binary search tree on value, so laying level d out
        // in order (slot i of 2^d) keeps every fraction sorted left to right.
        let col = |d: usize, i: usize| -> i32 {
            let f = (2 * i + 1) as f32 / (1u32 << (d + 1)) as f32;
            (f * (w as f32 - 1.0)).round() as i32
        };
        let line = |grid: &mut BrailleGrid, a: (i32, i32), b: (i32, i32)| {
            let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).clamp(1, 512);
            for i in 0..=n {
                draw::dot_i(grid, a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n);
            }
        };

        let stage = ctx.eased.clamp(0.0, 1.0) * levels as f32;
        // Breadth-first: each node carries its bounding fractions (a/b, c/d).
        let mut frontier: Vec<(u32, u32, u32, u32)> = vec![(0, 1, 1, 0)];
        for d in 0..levels {
            let frac = (stage - d as f32).clamp(0.0, 1.0);
            if frac <= 0.0 {
                break;
            }
            let count = frontier.len();
            let shown = ((frac * count as f32).ceil() as usize).min(count);
            let mut next = Vec::with_capacity(count * 2);
            for (i, &(a, b, c, dd)) in frontier.iter().enumerate() {
                let (p, q) = (a + c, b + dd);
                let me = (col(d, i), row(d));
                if i < shown {
                    draw::hline(
                        grid,
                        (me.0 - 1).max(0) as usize,
                        (me.0 + 1).max(0) as usize,
                        me.1.max(0) as usize,
                    );
                    if d > 0 {
                        line(grid, (col(d - 1, i / 2), row(d - 1)), me);
                    }
                }
                next.push((a, b, p, q));
                next.push((p, q, c, dd));
            }
            frontier = next;
        }

        // Walker: descends towards φ = [1;1,1,…] (alternating R, L, R, L…).
        let depth_shown = (stage.floor() as usize).min(levels);
        if depth_shown > 0 {
            let step = ((ctx.time.max(0.0) * 2.0) as usize) % depth_shown;
            let (mut a, mut b, mut c, mut dd) = (0u32, 1u32, 1u32, 0u32);
            let mut slot = 0usize;
            for d in 0..=step {
                let (p, q) = (a + c, b + dd);
                if d == step {
                    let (x, y) = (col(d, slot), row(d));
                    for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        draw::dot_i(grid, x + ox, y + oy);
                    }
                }
                // φ > p/q ⇔ go right (towards larger values).
                let phi = 1.618_034f32;
                if (p as f32) < phi * q as f32 {
                    a = p;
                    b = q;
                    slot = slot * 2 + 1;
                } else {
                    c = p;
                    dd = q;
                    slot *= 2;
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 9. Continued fraction convergents of φ
// ────────────────────────────────────────────────────────────────────────────

/// Continued fraction convergents of φ = [1;1,1,1,…]: each convergent p_k/q_k
/// is a best rational approximation. `eased` reveals convergents left to
/// right, each plotted by its signed log-error from φ; they alternate above /
/// below the true value — the zig-zag visualises the approximation theorem.
struct ContinuedFraction;
impl ProgressStyle for ContinuedFraction {
    fn name(&self) -> &str {
        "continued-fraction-phi"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Convergents of φ = [1;1,1,…] — 1/1, 2/1, 3/2, 5/3, 8/5… — plotted left to right as \
         a zig-zag that alternates above and below the dotted golden-ratio line and closes in \
         on it (error on a log scale); progress adds convergents, a marker rides the newest"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let phi = (1.0 + 5f64.sqrt()) / 2.0;
        let mid = (h as f32 - 1.0) * 0.5;
        // Dotted reference line at φ.
        for x in (0..w).step_by(2) {
            draw::dot(grid, x, mid.round() as usize);
        }
        let total = 16usize;
        let pitch = (w as f32 - 1.0) / (total - 1) as f32;
        let shown = ((ctx.eased * total as f32).ceil() as usize).clamp(1, total);
        // Signed log-error: convergent k misses φ by ≈ φ^(−2k), alternating sign.
        let (mut p, mut q) = (1u64, 1u64);
        let mut prev: Option<(i32, i32)> = None;
        for k in 0..shown {
            let err = p as f64 / q as f64 - phi;
            let mag = ((err.abs().max(1e-12).ln() + 12.0) / 12.0).clamp(0.0, 1.0) as f32;
            let y = (mid - err.signum() as f32 * mag * mid).round() as i32;
            let x = (k as f32 * pitch).round() as i32;
            if let Some((px, py)) = prev {
                let n = (x - px).abs().max((y - py).abs()).max(1);
                for i in 0..=n {
                    draw::dot_i(grid, px + (x - px) * i / n, py + (y - py) * i / n);
                }
            }
            prev = Some((x, y));
            (p, q) = (p + q, p);
        }
        if let Some((x, y)) = prev {
            // Pulsing marker on the newest convergent.
            if (ctx.time * 3.0).sin() > -0.3 {
                draw::fill_rect(grid, (x - 1).max(0) as usize, (y - 1).max(0) as usize, 3, 3);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 10. Times-table circles (cardioid / nephroid string art)
// ────────────────────────────────────────────────────────────────────────────

/// Times-table chord diagram: N points around a circle, point k joined to
/// point m·k mod N. m=2 traces a cardioid, m=3 a nephroid, m=4 a three-cusped
/// epicycloid and so on. A wide bar becomes a row of small circles, one
/// multiplier each, whose chords (dotted) are strung in turn as progress
/// advances while their envelope — the epicycloid with m−1 cusps — is traced
/// solid; `time` turns the figures slowly.
struct ModularCircle;
impl ProgressStyle for ModularCircle {
    fn name(&self) -> &str {
        "modular-circle"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Times-table chord diagrams: points round a circle, chord k→m·k mod N — a row of \
         circles for m = 2, 3, 4, … strung chord by chord as progress advances, each \
         chord envelope (cardioid, nephroid, three-cusp epicycloid…) traced solid, the \
         figures turning slowly with time"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w < 3 || h < 3 {
            return Ok(());
        }
        // Square panels, as many as fit (at least one).
        let side = h.min(w);
        let gap = 2usize;
        let panels = ((w + gap) / (side + gap)).clamp(1, 12);
        let left = (w + gap).saturating_sub(panels * (side + gap)) / 2;
        let n_pts = 18usize;
        let r = (side as f32 - 1.0) * 0.5;
        let rot = ctx.time * 0.25;
        let total = ctx.eased.clamp(0.0, 1.0) * panels as f32;
        for k in 0..panels {
            let m = k + 2;
            let cx = (left + k * (side + gap)) as f32 + r;
            let cy = r;
            let pt = |i: usize| -> (i32, i32) {
                let a = i as f32 / n_pts as f32 * 2.0 * PI + rot - PI * 0.5;
                (
                    (cx + r * a.cos()).round() as i32,
                    (cy + r * a.sin()).round() as i32,
                )
            };
            // Dotted rim marks every panel, filled or waiting.
            for i in (0..n_pts).step_by(2) {
                let (x, y) = pt(i);
                draw::dot_i(grid, x, y);
            }
            let local = (total - k as f32).clamp(0.0, 1.0);
            let shown = (local * n_pts as f32).round() as usize;
            // Chords as dotted threads so the strings stay distinct.
            for i in 1..shown.min(n_pts) {
                let (x0, y0) = pt(i);
                let (x1, y1) = pt((m * i) % n_pts);
                let steps = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 256);
                for s in (0..=steps).step_by(3) {
                    draw::dot_i(grid, x0 + (x1 - x0) * s / steps, y0 + (y1 - y0) * s / steps);
                }
            }
            // The chords' envelope, drawn solid: the epicycloid
            // z(θ) = (m·e^{iθ} + e^{imθ}) / (m+1) with m−1 cusps.
            let env_steps = (local * 96.0) as usize;
            let mf = m as f32;
            let base = rot - PI * 0.5;
            for s in 0..=env_steps.min(96) {
                let th = s as f32 / 96.0 * 2.0 * PI;
                let ex = (mf * th.cos() + (mf * th).cos()) / (mf + 1.0);
                let ey = (mf * th.sin() + (mf * th).sin()) / (mf + 1.0);
                let (bc, bs) = (base.cos(), base.sin());
                let x = cx + r * (ex * bc - ey * bs);
                let y = cy + r * (ex * bs + ey * bc);
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 11. Recamán's sequence
// ────────────────────────────────────────────────────────────────────────────

/// Recamán's sequence: a(0)=0; a(n) = a(n−1)−n if that is positive and new,
/// else a(n−1)+n. Each jump is drawn as a half-arc over a number line,
/// alternating above and below — the classic Recamán picture. `eased`
/// reveals terms (the newest arc grows as it is drawn); `time` runs a bead
/// along the newest arc.
struct Recaman;
impl ProgressStyle for Recaman {
    fn name(&self) -> &str {
        "recaman"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Recamán's sequence 0, 1, 3, 6, 2, 7, 13, 20, 12, 21, 11, … drawn the classic way: each \
         jump a half-arc over the number line, alternating above and below, nesting into \
         rainbows as progress adds terms; a bead runs along the newest arc"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }
        const TERMS: usize = 26;
        let mut seq = [0usize; TERMS];
        for n in 1..TERMS {
            let prev = seq[n - 1];
            let back = prev.wrapping_sub(n);
            seq[n] = if prev > n && !seq[..n].contains(&back) {
                back
            } else {
                prev + n
            };
        }
        let max_val = seq.iter().copied().max().unwrap_or(1).max(1) as f32;
        let max_jump = (TERMS - 1) as f32;
        let sx = (w as f32 - 1.0) / max_val;
        let mid = (h as f32 - 1.0) * 0.5;
        let reach = mid.max(0.5);

        // Number line.
        draw::hline(grid, 0, w - 1, mid.round() as usize);

        let stage = ctx.eased.clamp(0.0, 1.0) * (TERMS - 1) as f32;
        let full = stage as usize;
        for n in 1..TERMS {
            let part = if n <= full {
                1.0
            } else if n == full + 1 {
                stage.fract()
            } else {
                break;
            };
            if part <= 0.0 {
                break;
            }
            let (a, b) = (seq[n - 1] as f32, seq[n] as f32);
            let cx = (a + b) * 0.5 * sx;
            let rx = (b - a).abs() * 0.5 * sx;
            // Every arc keeps the same squashed aspect (height ∝ jump), so the
            // nested arcs stay distinct and the biggest just fills the half-height.
            let ry = reach * (n as f32 / max_jump);
            let sign = if n % 2 == 1 { -1.0 } else { 1.0 };
            let steps = ((rx + ry) * 2.0).clamp(6.0, 200.0) as usize;
            let upto = ((steps as f32) * part).round() as usize;
            // Travel from a to b: θ = π at a's side when b > a.
            let dir = if b > a { 1.0 } else { -1.0 };
            for s in 0..=upto {
                let th = s as f32 / steps as f32 * PI;
                let x = cx - dir * rx * th.cos();
                let y = mid + sign * ry * th.sin();
                draw::dot_i(grid, x.round() as i32, y.round() as i32);
            }
            if n == full.max(1) && part >= 1.0 {
                let th = (ctx.time.max(0.0) * 0.6).fract() * PI;
                let x = (cx - dir * rx * th.cos()).round() as i32;
                let y = (mid + sign * ry * th.sin()).round() as i32;
                for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    draw::dot_i(grid, x + ox, y + oy);
                }
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 12. Digital root / Vortex math
// ────────────────────────────────────────────────────────────────────────────

/// Digital root vortex math: place digits 1–9 around a circle; connect the
/// digital root sequence of n (n, n+n, …) as a cycle of chords. `eased`
/// selects the base number n ∈ 1..9; `time` rotates and pulses the figure.
/// The vortex math pattern mod 9 reveals hidden symmetries (3–6–9, 1–2–4–8–7–5).
struct DigitalRoot;
impl ProgressStyle for DigitalRoot {
    fn name(&self) -> &str {
        "digital-root-vortex"
    }
    fn theme(&self) -> &str {
        "numbertheory"
    }
    fn describe(&self) -> &str {
        "Vortex math: digital-root cycles 1–9 drawn as chords on a circle, revealing mod-9 symmetry"
    }
    fn render(&self, grid: &mut BrailleGrid, ctx: &BarContext) -> Result<(), DotmaxError> {
        let (w, h) = draw::dot_dims(grid);
        if w == 0 || h == 0 {
            return Ok(());
        }

        let cx = (w / 2) as f32;
        let cy = (h / 2) as f32;
        let rx = (w / 2).saturating_sub(1) as f32;
        let ry = (h / 2).saturating_sub(1) as f32;

        // Digital root of n mod 9 (returns 1..=9; 0→9).
        fn digital_root(n: usize) -> usize {
            if n == 0 {
                return 9;
            }
            let r = n % 9;
            if r == 0 {
                9
            } else {
                r
            }
        }

        // Base digit from eased (1–9).
        let base_digit = ((ctx.eased * 8.0).floor() as usize + 1).min(9);

        // Build the cycle: starting from base_digit, keep adding base_digit mod 9.
        let mut cycle = vec![base_digit];
        let mut cur = base_digit;
        for _ in 0..8 {
            cur = digital_root(cur + base_digit);
            if cur == cycle[0] {
                break;
            }
            cycle.push(cur);
        }

        // Rotation from time.
        let rot = ctx.time * 0.15;

        // Draw the 9 node positions on the circle.
        for d in 1..=9usize {
            let angle = (d as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let px = (cx + rx * angle.cos()).round() as usize;
            let py = (cy + ry * angle.sin()).round() as usize;
            if px < w && py < h {
                draw::dot(grid, px, py);
            }
        }

        // Draw the cycle chords.
        for i in 0..cycle.len() {
            let a = cycle[i];
            let b = cycle[(i + 1) % cycle.len()];
            let angle_a = (a as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let angle_b = (b as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let x0 = (cx + rx * angle_a.cos()).round() as i32;
            let y0 = (cy + ry * angle_a.sin()).round() as i32;
            let x1 = (cx + rx * angle_b.cos()).round() as i32;
            let y1 = (cy + ry * angle_b.sin()).round() as i32;

            let steps = ((x1 - x0).abs().max((y1 - y0).abs())).max(1) as usize;
            let steps = steps.min(512);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let px = (x0 as f32 + t * (x1 - x0) as f32).round() as i32;
                let py = (y0 as f32 + t * (y1 - y0) as f32).round() as i32;
                draw::dot_i(grid, px, py);
            }
        }

        // Draw the 3-6-9 axis separately for emphasis.
        for d in [3usize, 6, 9] {
            let angle = (d as f32 - 1.0) / 9.0 * 2.0 * PI - PI / 2.0 + rot;
            let px = (cx + rx * angle.cos()).round() as i32;
            let py = (cy + ry * angle.sin()).round() as i32;
            draw::dot_i(grid, px, py);
            draw::dot_i(grid, px + 1, py);
            draw::dot_i(grid, px, py + 1);
            draw::dot_i(grid, px - 1, py);
            draw::dot_i(grid, px, py - 1);
        }
        Ok(())
    }
}
