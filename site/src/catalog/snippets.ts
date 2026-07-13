import type { StyleMeta, StylePack, ThemePack } from "./types";

/** Rust snippet using the dotmax crate — the max-efficiency path. */
export function dotmaxSnippet(meta: StyleMeta): string {
  return `// cargo add dotmax
use dotmax::progress::{styles_for_theme, render_string, BarContext};

let style = styles_for_theme("${meta.theme}")
    .into_iter()
    .find(|s| s.name() == "${meta.name}")
    .expect("bundled with dotmax");

// Per tick: progress in 0..=1, elapsed seconds drive the animation.
// render_string returns the frame as plain text rows; reprint it each
// tick (e.g. cursor-up between frames). For colored output render the
// style into a BrailleGrid and draw with TerminalRenderer instead.
let ctx = BarContext::new(progress, elapsed_secs, 44, 4);
let frame = render_string(style.as_ref(), &ctx)?;
println!("{frame}");`;
}

/** Patch a theme's standalone program so it defaults to this style. */
export function patchStandalone(source: string, styleName: string): string {
  return source.replace(
    /const DEFAULT_STYLE: &str = "[^"]*";/,
    `const DEFAULT_STYLE: &str = "${styleName}";`,
  );
}

function hexToRgb(hex: string): [number, number, number] | null {
  if (!/^#[0-9a-f]{6}$/.test(hex)) return null;
  return [
    Number.parseInt(hex.slice(1, 3), 16),
    Number.parseInt(hex.slice(3, 5), 16),
    Number.parseInt(hex.slice(5, 7), 16),
  ];
}

/** One frame as a bash `$'...'` string body with truecolor ANSI runs. */
function frameAnsi(style: StylePack, frameIndex: number): string {
  const frame = style.frames[frameIndex];
  const rows: string[] = [];
  frame.t.forEach((row, y) => {
    const colorRow = frame.c?.[y];
    let out = "";
    let current: string | null = null;
    Array.from(row).forEach((ch, x) => {
      let color: string | null = null;
      if (colorRow) {
        const code = colorRow.slice(x * 2, x * 2 + 2);
        if (code !== "..") color = style.palette[Number.parseInt(code, 16)] ?? null;
      }
      if (color !== current) {
        const rgb = color ? hexToRgb(color) : null;
        out += rgb ? `\\033[38;2;${rgb[0]};${rgb[1]};${rgb[2]}m` : "\\033[0m";
        current = color;
      }
      // $'...' body: escape backslashes and single quotes.
      out += ch === "\\" ? "\\\\" : ch === "'" ? "\\'" : ch;
    });
    rows.push(out + "\\033[0m");
  });
  return rows.join("\\n");
}

/** Zero-dependency shell script that plays the pre-rendered frames. */
export function shellScript(meta: StyleMeta, style: StylePack, pack: ThemePack): string {
  const delay = (1 / Math.max(1, pack.fps)).toFixed(3);
  const lines: string[] = [
    "#!/usr/bin/env bash",
    `# ${meta.id} — ${meta.description}`,
    "# Pre-rendered ANSI frames from dotmax (https://github.com/newjordan/dotmax)",
    `# ${pack.width}x${pack.height} cells, ${pack.fps} fps, loops until ctrl-c. MIT OR Apache-2.0.`,
    "",
    "frames=(",
  ];
  for (let i = 0; i < style.frames.length; i += 1) {
    lines.push(`$'${frameAnsi(style, i)}'`);
  }
  lines.push(
    ")",
    "",
    "printf '\\033[?25l'",
    "trap \"printf '\\033[?25h\\033[0m\\n'; exit\" INT TERM",
    "while :; do",
    '  for f in "${frames[@]}"; do',
    "    printf '%s\\n' \"$f\"",
    `    sleep ${delay}`,
    `    printf '\\033[${pack.height}A'`,
    "  done",
    "done",
  );
  return lines.join("\n");
}
