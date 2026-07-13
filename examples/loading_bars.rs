//! Loading-bar gallery — browse every bundled progress-bar style.
//!
//! Cycles through all styles in `dotmax::progress`, animating each from 0% to
//! 100% and looping. This is the showcase / smoke-test harness for the
//! modular loading bars.
//!
//! ```bash
//! cargo run --example loading_bars
//! ```
//!
//! # Controls
//! - `→` / `space` : next style      `←` : previous style
//! - `]` / `[`     : jump to next / previous theme (52 themes, ~500 styles)
//! - `a`           : auto-advance on/off (default on)
//! - `p`           : pause/resume the fill animation
//! - `t`           : cycle the easing curve applied to the fill
//! - `q` / `Esc`   : quit

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use dotmax::animation::FrameTimer;
use dotmax::progress::{all_styles, BarContext, Easing};
use dotmax::{BrailleGrid, TerminalRenderer};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

const BAR_W: usize = 50; // cells
const BAR_H: usize = 4; // cells
const FPS: u32 = 30;
const AUTO_SECS: f32 = 4.0; // seconds per style when auto-advancing

const EASINGS: [Easing; 6] = [
    Easing::Linear,
    Easing::QuadInOut,
    Easing::CubicInOut,
    Easing::ExpoOut,
    Easing::BackOut,
    Easing::BounceOut,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let styles = all_styles();
    if styles.is_empty() {
        eprintln!("No progress styles registered.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, Clear(ClearType::All), Hide, MoveTo(0, 0))?;

    let mut renderer = TerminalRenderer::new()?;
    let mut timer = FrameTimer::new(FPS);

    let start = Instant::now();
    let mut idx = 0usize;
    let mut style_started = start;
    let mut auto = true;
    let mut paused = false;
    let mut easing_idx = 2usize; // CubicInOut
    let mut pause_accum = 0.0f32;
    let mut last = start;

    loop {
        // ---- input ----
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Right | KeyCode::Char(' ') => {
                        idx = (idx + 1) % styles.len();
                        style_started = Instant::now();
                    }
                    KeyCode::Left => {
                        idx = (idx + styles.len() - 1) % styles.len();
                        style_started = Instant::now();
                    }
                    KeyCode::Char(']') => {
                        // Jump to the first style of the next theme.
                        let cur = styles[idx].theme().to_string();
                        let mut j = idx;
                        while styles[j].theme() == cur {
                            j = (j + 1) % styles.len();
                        }
                        idx = j;
                        style_started = Instant::now();
                    }
                    KeyCode::Char('[') => {
                        // Jump to the first style of the previous theme.
                        let cur = styles[idx].theme().to_string();
                        let mut j = idx;
                        while styles[j].theme() == cur {
                            j = (j + styles.len() - 1) % styles.len();
                        }
                        let prev = styles[j].theme().to_string();
                        while j > 0 && styles[j - 1].theme() == prev {
                            j -= 1;
                        }
                        idx = j;
                        style_started = Instant::now();
                    }
                    KeyCode::Char('a') => auto = !auto,
                    KeyCode::Char('p') => paused = !paused,
                    KeyCode::Char('t') => easing_idx = (easing_idx + 1) % EASINGS.len(),
                    _ => {}
                }
            }
        }

        // ---- timing ----
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;
        if paused {
            pause_accum += dt;
        }
        let elapsed = now.duration_since(start).as_secs_f32();
        let anim_time = elapsed - pause_accum; // frozen while paused

        if auto && now.duration_since(style_started).as_secs_f32() > AUTO_SECS {
            idx = (idx + 1) % styles.len();
            style_started = now;
        }

        // Ping-pong progress 0->1->0 over ~3s.
        let phase = (anim_time / 3.0).fract();
        let progress = if phase < 0.5 {
            phase * 2.0
        } else {
            (1.0 - phase) * 2.0
        };

        let style = &styles[idx];
        let easing = EASINGS[easing_idx];

        let mut grid = BrailleGrid::new(BAR_W, BAR_H)?;
        let ctx = BarContext::new(progress, anim_time, BAR_W, BAR_H).with_easing(easing);
        style.render(&mut grid, &ctx)?;
        renderer.render(&grid)?;

        execute!(
            out,
            MoveTo(0, BAR_H as u16 + 1),
            Clear(ClearType::FromCursorDown),
            Print(format!(
                "[{:>3}/{}] {:<14} theme: {:<9} {:>3.0}%\r\n",
                idx + 1,
                styles.len(),
                style.name(),
                style.theme(),
                progress * 100.0,
            )),
            MoveTo(0, BAR_H as u16 + 2),
            Print(format!("  {}\r\n", style.describe())),
            MoveTo(0, BAR_H as u16 + 4),
            Print(format!(
                "easing: {:?}  | auto:{}  paused:{}  | [<-/->] style  [/] theme  [t]ween  [a]uto  [p]ause  [q]uit",
                easing,
                if auto { "on" } else { "off" },
                if paused { "yes" } else { "no" },
            )),
        )?;
        out.flush()?;

        timer.wait_for_next_frame();
    }

    execute!(out, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    renderer.cleanup()?;
    disable_raw_mode()?;
    println!("Browsed {} loading-bar styles.", styles.len());
    Ok(())
}
