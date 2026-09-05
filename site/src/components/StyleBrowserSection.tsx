import { ArrowDownAZ, ChevronDown, Dices, ListOrdered, Search, Shuffle } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { frameHtml } from "../catalog/frameHtml";
import { subscribeFrames, useTickerFrame } from "../catalog/frameTicker";
import { useInView } from "../catalog/inView";
import { loadThemePack, useCatalogIndex } from "../catalog/packs";
import type { CatalogIndex, StyleKind, StyleMeta, StylePack, ThemePack } from "../catalog/types";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";
import { StyleDetailDialog } from "./StyleDetailDialog";

const DEFAULT_VISIBLE = 12;
const PAGE_SIZE = 24;

const kindTabs: Array<{ id: StyleKind | "all"; label: string }> = [
  { id: "all", label: "All" },
  { id: "bar", label: "Bars" },
  { id: "spinner", label: "Spinners" },
  { id: "border", label: "Borders" },
  { id: "wipe", label: "Wipes" },
  { id: "meter", label: "Meters" },
];

type SortKey = "catalog" | "name" | "shuffle";

const sortOptions: Array<{ id: SortKey; label: string; icon: typeof Shuffle; title: string }> = [
  { id: "catalog", label: "Catalog", icon: ListOrdered, title: "Catalog order" },
  { id: "name", label: "A–Z", icon: ArrowDownAZ, title: "Alphabetical" },
  { id: "shuffle", label: "Shuffle", icon: Shuffle, title: "Shuffle (click again to reshuffle)" },
];

/**
 * Curated theme groupings — a fast way to narrow 600+ styles to a mood.
 * Themes missing from the index are ignored, so the list survives catalog churn.
 */
const collections: Array<{ id: string; label: string; themes: string[] }> = [
  { id: "fable", label: "Fable 5.1", themes: ["fable"] },
  {
    id: "scifi",
    label: "Sci-fi & code",
    themes: ["matrix", "tech", "glitch", "synthwave", "demoscene", "space", "cosmos", "lasers", "quantum", "electronics", "gadgets"],
  },
  {
    id: "nature",
    label: "Nature & life",
    themes: ["nature", "ocean", "animals", "wildlife", "plants", "fruits", "food", "weather", "biology", "surf"],
  },
  { id: "retro", label: "Retro games", themes: ["retro", "atari", "nintendo", "gameboy", "blocks"] },
  {
    id: "math",
    label: "Math & science",
    themes: ["fractal", "chaos", "waves", "cellular", "geometry", "noise", "numbertheory", "topology", "physics", "chemistry", "sinewave", "goldenratio", "penrose", "platonic", "perspective"],
  },
  {
    id: "culture",
    label: "Culture & craft",
    themes: ["floweroflife", "yantra", "mythology", "medieval", "cultures", "architecture", "music", "sports", "transit", "cars"],
  },
  { id: "light", label: "Light & fire", themes: ["aurora", "inferno", "fireworks", "classic"] },
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
      data-kind={meta.kind}
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
          <span className="loading-bar-card-open" aria-hidden="true">
            open ↗
          </span>
        </div>
        <div className="loading-bar-card-body">
          <div className="loading-bar-card-meta">
            <span className={`kind-chip kind-chip-${meta.kind}`}>{meta.kind}</span>
            <span>{meta.name}</span>
          </div>
          <p>{meta.description}</p>
          <span className="loading-bar-card-copyhint">copy as: dotmax crate · standalone .rs · shell script</span>
        </div>
      </button>
    </article>
  );
});

/** Deterministic shuffle so the order is stable across re-renders for one seed. */
function shuffled<T>(items: T[], seed: number): T[] {
  const out = items.slice();
  let state = seed >>> 0 || 1;
  for (let i = out.length - 1; i > 0; i -= 1) {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    const j = state % (i + 1);
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

function filterStyles(
  index: CatalogIndex,
  kind: StyleKind | "all",
  theme: string,
  collectionThemes: Set<string> | null,
  query: string,
): StyleMeta[] {
  const normalized = query.trim().toLowerCase();
  return index.styles.filter((style) => {
    if (kind !== "all" && style.kind !== kind) return false;
    if (collectionThemes && !collectionThemes.has(style.theme)) return false;
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
  const [activeCollection, setActiveCollection] = useState<string | null>(null);
  const [sortKey, setSortKey] = useState<SortKey>("catalog");
  const [shuffleSeed, setShuffleSeed] = useState(() => Date.now());
  const [visibleCount, setVisibleCount] = useState(DEFAULT_VISIBLE);
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

  // The command palette, hero, and marquee deep-link styles into this browser.
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

  const knownThemes = useMemo(() => new Set((index?.themes ?? []).map((theme) => theme.name)), [index]);

  const collectionThemes = useMemo(() => {
    if (!activeCollection) return null;
    const collection = collections.find((entry) => entry.id === activeCollection);
    if (!collection) return null;
    return new Set(collection.themes.filter((theme) => knownThemes.has(theme)));
  }, [activeCollection, knownThemes]);

  const themeOptions = useMemo(() => {
    if (!index) return [];
    return index.themes.filter((theme) => {
      if (activeKind !== "all" && theme.kind !== activeKind) return false;
      if (collectionThemes && !collectionThemes.has(theme.name)) return false;
      return true;
    });
  }, [index, activeKind, collectionThemes]);

  const kindCounts = useMemo(() => {
    const counts: Record<string, number> = { all: index?.total ?? 0 };
    for (const theme of index?.themes ?? []) {
      counts[theme.kind] = (counts[theme.kind] ?? 0) + theme.count;
    }
    return counts;
  }, [index]);

  const collectionCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    if (!index) return counts;
    for (const collection of collections) {
      counts[collection.id] = index.themes
        .filter((theme) => collection.themes.includes(theme.name))
        .reduce((sum, theme) => sum + theme.count, 0);
    }
    return counts;
  }, [index]);

  const filtered = useMemo(() => {
    if (!index) return [];
    const base = filterStyles(index, activeKind, activeTheme, collectionThemes, query);
    if (sortKey === "name") return base.slice().sort((a, b) => a.name.localeCompare(b.name) || a.theme.localeCompare(b.theme));
    if (sortKey === "shuffle") return shuffled(base, shuffleSeed);
    return base;
  }, [index, activeKind, activeTheme, collectionThemes, query, sortKey, shuffleSeed]);

  const isFiltering =
    query.trim().length > 0 || activeTheme !== "All" || activeKind !== "all" || activeCollection !== null;
  const displayStyles = isFiltering ? filtered : filtered.slice(0, visibleCount);
  const total = index?.total ?? 0;
  const staticFrame = index ? Math.floor(index.frames_per_style / 4) : 0;
  const remaining = filtered.length - displayStyles.length;

  function pickKind(kind: StyleKind | "all") {
    setActiveKind(kind);
    setActiveTheme("All");
    if (kind !== "all" && kind !== "bar") setActiveCollection(null);
  }

  function pickCollection(id: string | null) {
    setActiveCollection((current) => (current === id ? null : id));
    setActiveTheme("All");
    if (id && activeKind !== "all" && activeKind !== "bar") setActiveKind("all");
  }

  function pickSort(key: SortKey) {
    if (key === "shuffle") setShuffleSeed(Date.now());
    setSortKey(key);
  }

  function surprise() {
    const pool = filtered.length > 0 ? filtered : index?.styles ?? [];
    if (pool.length === 0) return;
    setSelected(pool[Math.floor(Math.random() * pool.length)]);
  }

  function clearFilters() {
    setQuery("");
    setActiveKind("all");
    setActiveTheme("All");
    setActiveCollection(null);
  }

  return (
    <section id="loading-bars" className="section loading-bars-section" ref={sectionRef}>
      <div className="loading-bars-head">
        <div className="section-heading" data-reveal>
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
        <div className="loading-bars-head-side" data-reveal>
          <div className="loading-bars-count" aria-label="Style catalog count">
            <strong>{displayStyles.length}</strong>
            <span>shown</span>
            <strong>{total}</strong>
            <span>total</span>
          </div>
          <button className="surprise-button" type="button" onClick={surprise} disabled={!index}>
            <Dices size={16} />
            Surprise me
          </button>
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
          <div className="loading-sort" role="group" aria-label="Sort styles">
            {sortOptions.map(({ id, label, icon: Icon, title }) => (
              <button
                key={id}
                type="button"
                title={title}
                aria-pressed={sortKey === id}
                className={sortKey === id ? "loading-sort-button loading-sort-button-active" : "loading-sort-button"}
                onClick={() => pickSort(id)}
              >
                <Icon size={13} />
                {label}
              </button>
            ))}
          </div>
        </div>
        <div className="loading-theme-column">
          {(activeKind === "all" || activeKind === "bar") && (
            <div className="loading-collection-strip" role="group" aria-label="Style collections">
              {collections.map((collection) => (
                <button
                  key={collection.id}
                  type="button"
                  aria-pressed={activeCollection === collection.id}
                  className={
                    activeCollection === collection.id
                      ? "loading-collection-chip loading-collection-chip-active"
                      : "loading-collection-chip"
                  }
                  onClick={() => pickCollection(collection.id)}
                >
                  {collection.label}
                  <span>{collectionCounts[collection.id] ?? 0}</span>
                </button>
              ))}
            </div>
          )}
          <div className="loading-theme-strip" aria-label="Style theme filters">
            <button
              className={activeTheme === "All" ? "loading-theme-pill loading-theme-pill-active" : "loading-theme-pill"}
              onClick={() => setActiveTheme("All")}
              type="button"
            >
              All{" "}
              <span>
                {collectionThemes
                  ? themeOptions.reduce((sum, theme) => sum + theme.count, 0)
                  : activeKind === "all"
                    ? total
                    : (kindCounts[activeKind] ?? 0)}
              </span>
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
      </div>

      <div className="loading-bars-grid" data-loading-bar-count={total} ref={gridRef}>
        {displayStyles.map((meta, position) => (
          <StyleCard
            key={meta.id}
            meta={meta}
            style={styleById.get(meta.id) ?? null}
            staticFrame={staticFrame}
            reducedMotion={prefersReducedMotion}
            stagger={position % DEFAULT_VISIBLE}
            onOpen={setSelected}
            requestTheme={requestTheme}
          />
        ))}
      </div>

      {index && filtered.length === 0 && (
        <div className="loading-bars-empty">
          <Search size={18} />
          <span>No styles match that filter.</span>
          <button className="button button-secondary" type="button" onClick={clearFilters}>
            Clear filters
          </button>
        </div>
      )}

      {!isFiltering && remaining > 0 && (
        <div className="expander-row">
          <button className="expander-button" type="button" onClick={() => setVisibleCount((count) => count + PAGE_SIZE)}>
            <ChevronDown size={16} />
            Show {Math.min(PAGE_SIZE, remaining)} more
          </button>
          <button className="expander-button expander-button-ghost" type="button" onClick={() => setVisibleCount(filtered.length)}>
            Browse all {total} styles
          </button>
        </div>
      )}
      {!isFiltering && remaining === 0 && filtered.length > DEFAULT_VISIBLE && (
        <div className="expander-row">
          <button className="expander-button" type="button" onClick={() => setVisibleCount(DEFAULT_VISIBLE)}>
            Show fewer
          </button>
        </div>
      )}

      {selected && (
        <StyleDetailDialog
          meta={selected}
          onClose={() => setSelected(null)}
          siblings={filtered.length > 0 ? filtered : index?.styles}
          onNavigate={setSelected}
        />
      )}
    </section>
  );
}
