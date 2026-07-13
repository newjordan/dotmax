//! Scratch QA tool: dump text renders of one theme's styles at several
//! progress/time points. Run: `cargo run --example qa_render -- <theme> [p] [t]`

use dotmax::progress::{render_string, styles_for_theme, BarContext, Easing};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let theme = args.get(1).map_or("synthwave", String::as_str);
    let samples: Vec<(f32, f32)> = if let (Some(p), Some(t)) = (args.get(2), args.get(3)) {
        vec![(p.parse().unwrap(), t.parse().unwrap())]
    } else {
        vec![(0.2, 0.7), (0.55, 2.0), (0.9, 3.3)]
    };
    for style in styles_for_theme(theme) {
        for &(p, t) in &samples {
            let ctx = BarContext::new(p, t, 44, 4)
                .with_easing(Easing::CubicInOut)
                .with_label(format!("{:.0}%", p * 100.0));
            let s = render_string(style.as_ref(), &ctx).unwrap();
            println!(
                "┌─ {}/{} p={p} t={t} {}",
                style.theme(),
                style.name(),
                "─".repeat(20)
            );
            for line in s.lines() {
                println!("│{line}│");
            }
        }
    }
}
