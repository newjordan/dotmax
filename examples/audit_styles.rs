//! Scratch QA tool: screen every bundled style for weak renders.
//! Flags near-empty output, progress-illegible bars, and static bars.
//! Run: `cargo run --example audit_styles`

use dotmax::progress::{all_styles, render_string, BarContext, Easing};

fn frame(style: &dyn dotmax::progress::ProgressStyle, p: f32, t: f32) -> String {
    let ctx = BarContext::new(p, t, 44, 4)
        .with_easing(Easing::CubicInOut)
        .with_label(format!("{:.0}%", p * 100.0));
    render_string(style, &ctx).unwrap()
}

fn ink(s: &str) -> usize {
    s.chars()
        .filter(|&c| c != '\u{2800}' && c != ' ' && c != '\n')
        .count()
}

fn main() {
    for style in all_styles() {
        let id = format!("{}/{}", style.theme(), style.name());
        let mid = frame(style.as_ref(), 0.55, 2.0);
        let lo = frame(style.as_ref(), 0.2, 1.0);
        let hi = frame(style.as_ref(), 0.9, 3.0);
        let mid_t2 = frame(style.as_ref(), 0.55, 2.25);

        let mut flags = Vec::new();
        if ink(&mid) < 8 {
            flags.push(format!("near-empty(mid ink={})", ink(&mid)));
        }
        if lo == hi {
            flags.push("progress-illegible(20%==90%)".to_string());
        }
        if mid == mid_t2 {
            flags.push("static(no time motion)".to_string());
        }
        if !flags.is_empty() {
            println!("{id}: {}", flags.join(", "));
        }
    }
}
