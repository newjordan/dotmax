import type { StylePack } from "./types";

const HEX_COLOR = /^#[0-9a-f]{6}$/;

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

const htmlCache = new WeakMap<StylePack, (string | undefined)[]>();

/**
 * Render one catalog frame to an HTML string with per-run color spans.
 * Memoized per (style, frame) — the grid replays frames in a loop, so each
 * frame is built exactly once.
 */
export function frameHtml(style: StylePack, frameIndex: number): string {
  let cache = htmlCache.get(style);
  if (!cache) {
    cache = [];
    htmlCache.set(style, cache);
  }
  const index = ((frameIndex % style.frames.length) + style.frames.length) % style.frames.length;
  const cached = cache[index];
  if (cached !== undefined) return cached;

  const frame = style.frames[index];
  const pieces: string[] = [];
  frame.t.forEach((row, y) => {
    const colorRow = frame.c?.[y];
    if (!colorRow) {
      pieces.push(escapeHtml(row));
      return;
    }
    let html = "";
    let runColor: string | null = null;
    let run = "";
    const flush = () => {
      if (run.length === 0) return;
      html += runColor ? `<span style="color:${runColor}">${escapeHtml(run)}</span>` : escapeHtml(run);
      run = "";
    };
    Array.from(row).forEach((ch, x) => {
      const code = colorRow.slice(x * 2, x * 2 + 2);
      let color: string | null = null;
      if (code !== ".." && code.length === 2) {
        const paletteEntry = style.palette[Number.parseInt(code, 16)];
        if (paletteEntry && HEX_COLOR.test(paletteEntry)) color = paletteEntry;
      }
      if (color !== runColor) {
        flush();
        runColor = color;
      }
      run += ch;
    });
    flush();
    pieces.push(html);
  });

  const result = pieces.join("\n");
  cache[index] = result;
  return result;
}
