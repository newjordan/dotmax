import { ChevronDown, Search } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { frameHtml } from "../catalog/frameHtml";
import { subscribeFrames, useTickerFrame } from "../catalog/frameTicker";
import { useInView } from "../catalog/inView";
import { loadThemePack, useCatalogIndex } from "../catalog/packs";
import type { CatalogIndex, StyleKind, StyleMeta, StylePack, ThemePack } from "../catalog/types";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";
import { StyleDetailDialog } from "./StyleDetailDialog";

const DEFAULT_VISIBLE = 12;

const kindTabs: Array<{ id: StyleKind | "all"; label: string }> = [
  { id: "all", label: "All" },
  { id: "bar", label: "Bars" },
  { id: "spinner", label: "Spinners" },
  { id: "border", label: "Borders" },
  { id: "wipe", label: "Wipes" },
  { id: "meter", label: "Meters" },
];

type StyleCardProps = {
  meta: StyleMeta;
  style: StylePack | null;
  staticFrame: number;
  reducedMotion: boolean;
  stagger: number;
  onOpen: (meta: StyleMeta) => void;
  requestTheme: (theme: string) => void;
};

const StyleCard = memo(function StyleCard({
  meta,
  style,
  staticFrame,
  reducedMotion,
  stagger,
  onOpen,
  requestTheme,
}: StyleCardProps) {
  const [ref, inView] = useInView<HTMLElement>();
  const frame = useTickerFrame(inView && !reducedMotion && style !== null);

  useEffect(() => {
    if (inView) requestTheme(meta.theme);
  }, [inView, meta.theme, requestTheme]);

  const html = style ? frameHtml(style, reducedMotion ? staticFrame : frame) : null;

  return (
    <article
      className="loading-bar-card"
      data-theme={meta.theme}
      ref={ref}
      style={{ "--card-i": stagger } as React.CSSProperties}
    >
      <button className="loading-bar-card-trigger" type="button" onClick={() => onOpen(meta)}>
        <div className="loading-bar-terminal" aria-label={`${meta.name} preview`}>
          <div className="loading-bar-terminal-top">
            <span>{meta.name}</span>
            <span>{meta.theme}</span>
          </div>
          {html ? (
            <pre className="loading-bar-output" dangerouslySetInnerHTML={{ __html: html }} />
          ) : (
            <pre className="loading-bar-output style-skeleton" aria-hidden="true">
              {Array.from({ length: 4 }, () => "⠐⠀⠂⠀⠁".repeat(9)).join("\n")}
            </pre>
          )}
        </div>
        <div className="loading-bar-card-body">
          <div className="loading-bar-card-meta">
            <span>{meta.kind}</span>
            <span>{meta.name}</span>
          </div>
          <p>{meta.description}</p>
          <span className="loading-bar-card-copyhint">copy as: dotmax crate · standalone .rs · shell script</span>
        </div>
      </button>
    </article>
  );
});

function filterStyles(
  index: CatalogIndex,
  kind: StyleKind | "all",
  theme: string,
  query: string,
): StyleMeta[] {
  const normalized = query.trim().toLowerCase();
  return index.styles.filter((style) => {
    if (kind !== "all" && style.kind !== kind) return false;
    if (theme !== "All" && style.theme !== theme) return false;
    if (normalized.length === 0) return true;
    return [style.name, style.theme, style.kind, style.description].join(" ").toLowerCase().includes(normalized);
  });
}

export function StyleBrowserSection() {
  const index = useCatalogIndex();
  const [sectionRef, sectionInView] = useInView<HTMLElement>();
  const [query, setQuery] = useState("");
  const [activeKind, setActiveKind] = useState<StyleKind | "all">("all");
  const [activeTheme, setActiveTheme] = useState("All");
  const [expanded, setExpanded] = useState(false);
  const [selected, setSelected] = useState<StyleMeta | null>(null);
  const [packs, setPacks] = useState<Record<string, ThemePack>>({});
  const gridRef = useRef<HTMLDivElement | null>(null);
  const prefersReducedMotion = usePrefersReducedMotion();

  const requestTheme = useCallback((theme: string) => {
    loadThemePack(theme)
      .then((pack) => {
        setPacks((current) => (current[theme] ? current : { ...current, [theme]: pack }));
      })
      .catch(() => undefined);
  }, []);

  // Expose the advancing frame on the grid without re-rendering every card.
  useEffect(() => {
    if (!sectionInView || prefersReducedMotion) return undefined;
    return subscribeFrames((frame) => {
      gridRef.current?.setAttribute("data-loading-bar-frame", String(frame));
    });
  }, [sectionInView, prefersReducedMotion]);

  // The command palette deep-links styles into this browser.
  useEffect(() => {
    function onOpenStyle(event: Event) {
      const id = (event as CustomEvent<{ id?: string }>).detail?.id;
      const meta = index?.styles.find((style) => style.id === id);
      if (meta) setSelected(meta);
    }
    window.addEventListener("dotmax:open-style", onOpenStyle);
    return () => window.removeEventListener("dotmax:open-style", onOpenStyle);
  }, [index]);

  const styleById = useMemo(() => {
    const map = new Map<string, StylePack>();
    for (const pack of Object.values(packs)) {
      for (const style of pack.styles) map.set(style.id, style);
    }
    return map;
  }, [packs]);

  const themeOptions = useMemo(() => {
    if (!index) return [];
    return activeKind === "all" ? index.themes : index.themes.filter((theme) => theme.kind === activeKind);
  }, [index, activeKind]);

  const kindCounts = useMemo(() => {
    const counts: Record<string, number> = { all: index?.total ?? 0 };
    for (const theme of index?.themes ?? []) {
      counts[theme.kind] = (counts[theme.kind] ?? 0) + theme.count;
    }
    return counts;
  }, [index]);

  const filtered = useMemo(
    () => (index ? filterStyles(index, activeKind, activeTheme, query) : []),
    [index, activeKind, activeTheme, query],
  );

  const isFiltering = query.trim().length > 0 || activeTheme !== "All" || activeKind !== "all";
  const displayStyles = isFiltering || expanded ? filtered : filtered.slice(0, DEFAULT_VISIBLE);
  const total = index?.total ?? 0;
  const staticFrame = index ? Math.floor(index.frames_per_style / 4) : 0;

  function pickKind(kind: StyleKind | "all") {
    setActiveKind(kind);
    setActiveTheme("All");
  }

  return (
    <section id="loading-bars" className="section loading-bars-section" ref={sectionRef}>
      <div className="loading-bars-head">
        <div className="section-heading">
          <span className="eyebrow">Style library</span>
          <h2>
            {total > 0
              ? `${total} terminal loaders, borders, spinners, wipes & meters — copy-paste ready.`
              : "Terminal loaders, borders, spinners, wipes & meters — copy-paste ready."}
          </h2>
          <p>
            Rendered live from the Rust styles with full color. Click any card for three ways to take it home: the
            dotmax crate, a single standalone .rs file with zero dependencies, or a plain shell script.
          </p>
        </div>
        <div className="loading-bars-count" aria-label="Style catalog count">
          <strong>{displayStyles.length}</strong>
          <span>shown</span>
          <strong>{total}</strong>
          <span>total</span>
        </div>
      </div>

      <div className="loading-bars-toolbar">
        <div className="loading-bars-toolbar-left">
          <label className="search-box">
            <Search size={16} />
            <span className="sr-only">Search styles</span>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search wildlife, quantum, sinewave, spinner..."
            />
          </label>
          <div className="loading-kind-tabs" role="tablist" aria-label="Component kinds">
            {kindTabs.map(({ id, label }) => (
              <button
                key={id}
                role="tab"
                aria-selected={activeKind === id}
                className={activeKind === id ? "loading-kind-tab loading-kind-tab-active" : "loading-kind-tab"}
                onClick={() => pickKind(id)}
                type="button"
              >
                {label} <span>{kindCounts[id] ?? 0}</span>
              </button>
            ))}
          </div>
        </div>
        <div className="loading-theme-strip" aria-label="Style theme filters">
          <button
            className={activeTheme === "All" ? "loading-theme-pill loading-theme-pill-active" : "loading-theme-pill"}
            onClick={() => setActiveTheme("All")}
            type="button"
          >
            All <span>{activeKind === "all" ? total : kindCounts[activeKind] ?? 0}</span>
          </button>
          {themeOptions.map((theme) => (
            <button
              className={activeTheme === theme.name ? "loading-theme-pill loading-theme-pill-active" : "loading-theme-pill"}
              key={theme.name}
              onClick={() => setActiveTheme(theme.name)}
              type="button"
            >
              {theme.name} <span>{theme.count}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="loading-bars-grid" data-loading-bar-count={total} ref={gridRef}>
        {displayStyles.map((meta, index) => (
          <StyleCard
            key={meta.id}
            meta={meta}
            style={styleById.get(meta.id) ?? null}
            staticFrame={staticFrame}
            reducedMotion={prefersReducedMotion}
            stagger={index % DEFAULT_VISIBLE}
            onOpen={setSelected}
            requestTheme={requestTheme}
          />
        ))}
      </div>

      {!isFiltering && !expanded && filtered.length > DEFAULT_VISIBLE && (
        <button className="expander-button" type="button" onClick={() => setExpanded(true)}>
          <ChevronDown size={16} />
          Browse all {total} styles
        </button>
      )}
      {!isFiltering && expanded && (
        <button className="expander-button" type="button" onClick={() => setExpanded(false)}>
          Show fewer
        </button>
      )}

      {selected && <StyleDetailDialog meta={selected} onClose={() => setSelected(null)} />}
    </section>
  );
}
