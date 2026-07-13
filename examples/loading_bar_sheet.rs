//! Generate a single-page loading-bar cell sheet for every bundled style.
//!
//! ```bash
//! cargo run --example loading_bar_sheet
//! ```
//!
//! By default this writes `docs/progress_loader_sheet.html`. Pass an output
//! path as the first argument to write elsewhere.

use dotmax::progress::{all_styles, render_lines, themes, BarContext, Easing};
use std::env;
use std::fs;
use std::path::PathBuf;

const CELL_W: usize = 34;
const CELL_H: usize = 5;
const FRAMES: usize = 12;
const FRAME_STEP_SECS: f32 = 0.18;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/progress_loader_sheet.html"));

    let styles = all_styles();
    let mut html = String::new();

    push_head(&mut html, styles.len());
    html.push_str("<main class=\"sheet\">\n");

    for (idx, style) in styles.iter().enumerate() {
        let mut frames = Vec::with_capacity(FRAMES);
        for frame in 0..FRAMES {
            let t = frame as f32 * FRAME_STEP_SECS;
            let phase = frame as f32 / (FRAMES - 1) as f32;
            let progress = if phase < 0.5 {
                phase * 2.0
            } else {
                (1.0 - phase) * 2.0
            };
            let ctx = BarContext::new(progress, t, CELL_W, CELL_H)
                .with_easing(Easing::CubicInOut)
                .with_label(format!("{:.0}%", progress * 100.0));
            frames.push(render_lines(style.as_ref(), &ctx)?);
        }

        html.push_str("<article class=\"cell\" data-theme=\"");
        html_escape_into(style.theme(), &mut html);
        html.push_str("\">\n");
        html.push_str("  <div class=\"corner top-left\">");
        html_escape_into(style.theme(), &mut html);
        html.push_str(" / ");
        html_escape_into(style.name(), &mut html);
        html.push_str("</div>\n");
        html.push_str("  <div class=\"corner top-right\">#");
        html.push_str(&(idx + 1).to_string());
        html.push_str("</div>\n");
        html.push_str("  <pre class=\"frames\" aria-label=\"");
        html_escape_into(style.describe(), &mut html);
        html.push_str("\">");
        for (frame_idx, frame_lines) in frames.iter().enumerate() {
            html.push_str("<span class=\"frame");
            if frame_idx == 0 {
                html.push_str(" is-active");
            }
            html.push_str("\">");
            let frame_text = frame_lines
                .iter()
                .map(|line| line.trim_end_matches(' '))
                .collect::<Vec<_>>()
                .join("\n");
            html_escape_into(&frame_text, &mut html);
            html.push_str("</span>");
        }
        html.push_str("</pre>\n");
        html.push_str("  <div class=\"corner bottom-left\">");
        html_escape_into(style.describe(), &mut html);
        html.push_str("</div>\n");
        html.push_str("  <div class=\"corner bottom-right\">");
        html.push_str(&format!("{CELL_W}x{CELL_H} / {FRAMES}f"));
        html.push_str("</div>\n");
        html.push_str("</article>\n");
    }

    html.push_str("</main>\n");
    push_script(&mut html);
    html.push_str("</body>\n</html>\n");

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, html)?;
    println!(
        "Wrote {} styles across {} themes to {}",
        styles.len(),
        themes().len(),
        output.display()
    );
    Ok(())
}

fn push_head(html: &mut String, style_count: usize) {
    html.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("  <title>dotmax loader cell sheet</title>\n");
    html.push_str("  <style>\n");
    html.push_str(
        r#":root {
  color-scheme: dark;
  --bg: #101114;
  --panel: #17191d;
  --ink: #f1f5f9;
  --muted: #9aa4b2;
  --line: #2a2f37;
  --accent: #2dd4bf;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--ink);
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
header {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 24px;
  padding: 16px 20px;
  background: rgba(16, 17, 20, 0.96);
  border-bottom: 1px solid var(--line);
}
h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 0;
}
.meta {
  color: var(--muted);
  font-size: 13px;
  white-space: nowrap;
}
.sheet {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 12px;
  padding: 12px;
}
.cell {
  position: relative;
  min-height: 178px;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--panel);
}
.cell::before {
  content: "";
  position: absolute;
  inset: 0;
  border-top: 2px solid var(--accent);
  opacity: 0.35;
  pointer-events: none;
}
.corner {
  position: absolute;
  z-index: 2;
  max-width: calc(100% - 28px);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--muted);
  font-size: 11px;
  line-height: 1;
  background: rgba(23, 25, 29, 0.88);
}
.top-left { top: 8px; left: 9px; color: var(--ink); }
.top-right { top: 8px; right: 9px; }
.bottom-left { bottom: 8px; left: 9px; max-width: 72%; }
.bottom-right { bottom: 8px; right: 9px; }
.frames {
  position: absolute;
  left: 9px;
  right: 9px;
  top: 34px;
  bottom: 28px;
  display: grid;
  place-items: center start;
  margin: 0;
  overflow: hidden;
  color: #e6edf3;
  font: 16px/1.05 "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
  letter-spacing: 0;
}
.frame { display: none; }
.frame.is-active { display: block; }
@media (max-width: 520px) {
  header { align-items: flex-start; flex-direction: column; gap: 4px; }
  .meta { white-space: normal; }
  .sheet { grid-template-columns: 1fr; padding: 8px; }
  .frames { font-size: 14px; }
}
"#,
    );
    html.push_str("  </style>\n</head>\n<body>\n<header>\n");
    html.push_str("  <h1>dotmax loader cell sheet</h1>\n");
    html.push_str("  <div class=\"meta\">");
    html.push_str(&format!(
        "{} styles, {} themes, {} animation frames per cell",
        style_count,
        themes().len(),
        FRAMES
    ));
    html.push_str("</div>\n</header>\n");
}

fn push_script(html: &mut String) {
    html.push_str(
        r#"<script>
const cells = Array.from(document.querySelectorAll(".frames"));
let tick = 0;
setInterval(() => {
  tick += 1;
  for (const pre of cells) {
    const frames = pre.children;
    if (!frames.length) continue;
    const active = tick % frames.length;
    for (let i = 0; i < frames.length; i += 1) {
      frames[i].classList.toggle("is-active", i === active);
    }
  }
}, 120);
</script>
"#,
    );
}

fn html_escape_into(input: &str, output: &mut String) {
    for ch in input.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
}
