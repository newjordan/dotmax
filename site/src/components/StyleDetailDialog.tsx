import {
  Box,
  ChevronLeft,
  ChevronRight,
  Dices,
  FileCode2,
  Package,
  Pause,
  Play,
  ScrollText,
  Snail,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { densestFrame, frameHtml } from "../catalog/frameHtml";
import { currentFrame, useTickerFrame } from "../catalog/frameTicker";
import { loadStandalone, useThemePack } from "../catalog/packs";
import { dotmaxSnippet, patchStandalone, shellScript } from "../catalog/snippets";
import type { StyleMeta } from "../catalog/types";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";
import { CopyButton } from "./CopyButton";

type TabId = "dotmax" | "standalone" | "shell" | "source";

const tabs: Array<{ id: TabId; label: string; icon: typeof Package }> = [
  { id: "dotmax", label: "dotmax crate", icon: Package },
  { id: "standalone", label: "standalone .rs", icon: FileCode2 },
  { id: "shell", label: "shell script", icon: ScrollText },
  { id: "source", label: "style source", icon: Box },
];

const tabNotes: Record<TabId, string> = {
  dotmax: "Smallest integration — the style ships inside the dotmax crate.",
  standalone:
    "One self-contained file with this theme's styles and a minimal grid runtime. No cargo, no dependencies: rustc -O file.rs && ./file",
  shell: "Pre-rendered ANSI frames in plain bash — works in any terminal, no Rust at all.",
  source: "Excerpt for reading. Theme helper functions live in the standalone file, which is the runnable copy.",
};

const HEX_COLOR = /^#[0-9a-f]{6}$/;

/** Themes with a named designer, shown as a credit in the dialog. */
const themeCredits: Record<string, { name: string; href: string }> = {
  fable: { name: "Claude Fable 5.1", href: "https://www.anthropic.com/claude/fable" },
};

type Props = {
  meta: StyleMeta;
  onClose: () => void;
  /** The list the dialog can step through with ← → (the browser's current filter). */
  siblings?: StyleMeta[];
  onNavigate?: (meta: StyleMeta) => void;
};

export function StyleDetailDialog({ meta, onClose, siblings, onNavigate }: Props) {
  const pack = useThemePack(meta.theme);
  const [tab, setTab] = useState<TabId>("dotmax");
  const [standalone, setStandalone] = useState<string | null>(null);
  const [paused, setPaused] = useState(false);
  const [halfSpeed, setHalfSpeed] = useState(false);
  const prefersReducedMotion = usePrefersReducedMotion();

  const style = pack?.styles.find((entry) => entry.name === meta.name) ?? null;
  const frame = useTickerFrame(!prefersReducedMotion && !paused && style !== null);
  // Styles ping-pong progress across the loop, so the shared ticker can land
  // on the near-empty 0% phase the instant the dialog opens. Re-anchor the
  // playhead per style so the first visible frame is its densest one.
  const anchorRef = useRef<{ id: string; offset: number } | null>(null);
  if (style && anchorRef.current?.id !== meta.id) {
    anchorRef.current = { id: meta.id, offset: currentFrame() - densestFrame(style) };
  }
  const localFrame = frame - (anchorRef.current?.id === meta.id ? anchorRef.current.offset : 0);
  const shownFrame = halfSpeed ? Math.floor(localFrame / 2) : localFrame;

  const list = siblings && siblings.length > 1 ? siblings : null;
  const position = list ? list.findIndex((entry) => entry.id === meta.id) : -1;
  const canNavigate = list !== null && position >= 0 && typeof onNavigate === "function";

  function step(delta: number) {
    if (!canNavigate || !list || !onNavigate) return;
    const next = (position + delta + list.length) % list.length;
    onNavigate(list[next]);
  }

  function random() {
    if (!canNavigate || !list || !onNavigate) return;
    if (list.length < 2) return;
    let pick = position;
    while (pick === position) pick = Math.floor(Math.random() * list.length);
    onNavigate(list[pick]);
  }

  useEffect(() => {
    let live = true;
    loadStandalone(meta.theme)
      .then((text) => {
        if (live) setStandalone(text);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [meta.theme]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      } else if (event.key === "ArrowRight" && canNavigate) {
        event.preventDefault();
        step(1);
      } else if (event.key === "ArrowLeft" && canNavigate) {
        event.preventDefault();
        step(-1);
      } else if ((event.key === "r" || event.key === "R") && canNavigate && !event.metaKey && !event.ctrlKey) {
        event.preventDefault();
        random();
      } else if (event.key === " " && style && !prefersReducedMotion) {
        const target = event.target as HTMLElement | null;
        if (target && (target.tagName === "BUTTON" || target.tagName === "INPUT")) return;
        event.preventDefault();
        setPaused((current) => !current);
      }
    }
    window.addEventListener("keydown", onKey);
    document.body.style.overflow = "hidden";
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = "";
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [onClose, canNavigate, position, list, style, prefersReducedMotion]);

  const tabContent = useMemo<string | null>(() => {
    if (tab === "dotmax") return dotmaxSnippet(meta);
    if (tab === "standalone") return standalone ? patchStandalone(standalone, meta.name) : null;
    if (tab === "shell") return style && pack ? shellScript(meta, style, pack) : null;
    return style ? style.source : null;
  }, [tab, meta, standalone, style, pack]);

  const previewHtml = style
    ? frameHtml(style, prefersReducedMotion ? densestFrame(style) : shownFrame)
    : null;

  // Distinct palette swatches, in order of first appearance (max 12).
  const swatches = useMemo(() => {
    if (!style) return [];
    const seen: string[] = [];
    for (const color of style.palette) {
      if (HEX_COLOR.test(color) && !seen.includes(color)) seen.push(color);
      if (seen.length >= 12) break;
    }
    return seen;
  }, [style]);

  return (
    <div className="style-dialog-overlay" role="dialog" aria-modal="true" aria-label={`${meta.name} style`} onMouseDown={onClose}>
      <div className="style-dialog-panel" onMouseDown={(event) => event.stopPropagation()}>
        <div className="style-dialog-head">
          <div className="style-dialog-title">
            <span className="badge badge-accent">{meta.theme}</span>
            <h3>{meta.name}</h3>
            <span className="style-dialog-kind">{meta.kind}</span>
          </div>
          <div className="style-dialog-head-actions">
            {canNavigate && list && (
              <>
                <span className="style-dialog-position" aria-live="polite">
                  {position + 1} / {list.length}
                </span>
                <button className="icon-button" type="button" onClick={() => step(-1)} aria-label="Previous style (←)" title="Previous (←)">
                  <ChevronLeft size={16} />
                </button>
                <button className="icon-button" type="button" onClick={() => step(1)} aria-label="Next style (→)" title="Next (→)">
                  <ChevronRight size={16} />
                </button>
                <button className="icon-button" type="button" onClick={random} aria-label="Random style (R)" title="Random (R)">
                  <Dices size={16} />
                </button>
              </>
            )}
            <button className="icon-button" type="button" onClick={onClose} aria-label="Close style details">
              <X size={16} />
            </button>
          </div>
        </div>
        <p className="style-dialog-description">
          {meta.description}
          {themeCredits[meta.theme] && (
            <>
              {" "}
              <span className="style-dialog-credit">
                designed by{" "}
                <a href={themeCredits[meta.theme].href} target="_blank" rel="noreferrer">
                  {themeCredits[meta.theme].name}
                </a>
              </span>
            </>
          )}
        </p>

        <div className="style-dialog-preview" data-style-id={meta.id}>
          {!prefersReducedMotion && style && (
            <div className="style-dialog-preview-tools">
              <button
                className={halfSpeed ? "style-dialog-tool style-dialog-tool-active" : "style-dialog-tool"}
                type="button"
                aria-pressed={halfSpeed}
                aria-label="Play at half speed"
                title="Half speed"
                onClick={() => setHalfSpeed((current) => !current)}
              >
                <Snail size={14} />
              </button>
              <button
                className="style-dialog-tool"
                type="button"
                aria-pressed={paused}
                aria-label={paused ? "Resume playback" : "Pause playback"}
                title={paused ? "Play (space)" : "Pause (space)"}
                onClick={() => setPaused((current) => !current)}
              >
                {paused ? <Play size={14} /> : <Pause size={14} />}
              </button>
            </div>
          )}
          {previewHtml ? (
            <pre key={meta.id} className="style-dialog-preview-output" dangerouslySetInnerHTML={{ __html: previewHtml }} />
          ) : (
            <pre aria-hidden="true" className="style-skeleton">
              {"⠀".repeat(44)}
            </pre>
          )}
        </div>

        {swatches.length > 0 && (
          <div className="style-dialog-palette" aria-label="Style palette">
            {swatches.map((color) => (
              <span key={color} style={{ background: color }} title={color} />
            ))}
            <span className="style-dialog-palette-label">
              {style ? `${style.palette.length} colors` : ""}
            </span>
          </div>
        )}

        <div className="style-dialog-tabs" role="tablist" aria-label="Copy formats">
          {tabs.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              role="tab"
              aria-selected={tab === id}
              className={tab === id ? "style-dialog-tab style-dialog-tab-active" : "style-dialog-tab"}
              onClick={() => setTab(id)}
              type="button"
            >
              <Icon size={14} />
              {label}
            </button>
          ))}
        </div>

        <p className="style-dialog-note">{tabNotes[tab]}</p>

        <div className="style-dialog-code-wrap">
          <div className="style-dialog-code-bar">
            <span>
              {tab === "dotmax" && "snippet.rs"}
              {tab === "standalone" && `${meta.theme}.rs`}
              {tab === "shell" && `${meta.name}.sh`}
              {tab === "source" && `src/progress/styles/${meta.theme}.rs (excerpt)`}
            </span>
            {tabContent !== null && <CopyButton value={tabContent} />}
          </div>
          <pre className="style-dialog-code">
            {tabContent ?? "Loading…"}
          </pre>
        </div>
      </div>
    </div>
  );
}
