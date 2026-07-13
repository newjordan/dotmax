import { Box, FileCode2, Package, Pause, Play, ScrollText, Snail, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { frameHtml } from "../catalog/frameHtml";
import { useTickerFrame } from "../catalog/frameTicker";
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

export function StyleDetailDialog({ meta, onClose }: { meta: StyleMeta; onClose: () => void }) {
  const pack = useThemePack(meta.theme);
  const [tab, setTab] = useState<TabId>("dotmax");
  const [standalone, setStandalone] = useState<string | null>(null);
  const [paused, setPaused] = useState(false);
  const [halfSpeed, setHalfSpeed] = useState(false);
  const prefersReducedMotion = usePrefersReducedMotion();

  const style = pack?.styles.find((entry) => entry.name === meta.name) ?? null;
  const frame = useTickerFrame(!prefersReducedMotion && !paused && style !== null);
  const shownFrame = halfSpeed ? Math.floor(frame / 2) : frame;
  const staticFrame = pack ? Math.floor(pack.frames_per_style / 4) : 0;

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
      }
    }
    window.addEventListener("keydown", onKey);
    document.body.style.overflow = "hidden";
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = "";
    };
  }, [onClose]);

  const tabContent = useMemo<string | null>(() => {
    if (tab === "dotmax") return dotmaxSnippet(meta);
    if (tab === "standalone") return standalone ? patchStandalone(standalone, meta.name) : null;
    if (tab === "shell") return style && pack ? shellScript(meta, style, pack) : null;
    return style ? style.source : null;
  }, [tab, meta, standalone, style, pack]);

  const previewHtml = style
    ? frameHtml(style, prefersReducedMotion ? staticFrame : shownFrame)
    : null;

  return (
    <div className="style-dialog-overlay" role="dialog" aria-modal="true" aria-label={`${meta.name} style`} onMouseDown={onClose}>
      <div className="style-dialog-panel" onMouseDown={(event) => event.stopPropagation()}>
        <div className="style-dialog-head">
          <div className="style-dialog-title">
            <span className="badge badge-accent">{meta.theme}</span>
            <h3>{meta.name}</h3>
            <span className="style-dialog-kind">{meta.kind}</span>
          </div>
          <button className="icon-button" type="button" onClick={onClose} aria-label="Close style details">
            <X size={16} />
          </button>
        </div>
        <p className="style-dialog-description">{meta.description}</p>

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
                title={paused ? "Play" : "Pause"}
                onClick={() => setPaused((current) => !current)}
              >
                {paused ? <Play size={14} /> : <Pause size={14} />}
              </button>
            </div>
          )}
          {previewHtml ? (
            <pre dangerouslySetInnerHTML={{ __html: previewHtml }} />
          ) : (
            <pre aria-hidden="true" className="style-skeleton">
              {"⠀".repeat(44)}
            </pre>
          )}
        </div>

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
