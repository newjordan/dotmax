import { ChevronDown, Search } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";

const DEFAULT_VISIBLE = 12;

type LoadingBarStylePreview = {
  id: string;
  theme: string;
  name: string;
  description: string;
  command: string;
  frames: string[][];
};

type LoadingBarCatalog = {
  total: number;
  width: number;
  height: number;
  fps: number;
  frames_per_style: number;
  themes: Array<{ name: string; count: number }>;
  styles: LoadingBarStylePreview[];
};

function LoadingBarTerminal({
  style,
  width,
  frameIndex,
}: {
  style: LoadingBarStylePreview;
  width: number;
  frameIndex: number;
}) {
  const rows = style.frames[frameIndex % style.frames.length] ?? style.frames[0] ?? [];

  return (
    <div className="loading-bar-terminal" aria-label={`${style.name} loading bar preview`}>
      <div className="loading-bar-terminal-top">
        <span>{style.name}</span>
        <span>{style.theme}</span>
      </div>
      <pre className="loading-bar-output">
        {rows.map((row) => row.padEnd(width, " ")).join("\n")}
      </pre>
    </div>
  );
}

export function LoadingBarsSection() {
  const sectionRef = useRef<HTMLElement | null>(null);
  const lastTickRef = useRef<number | null>(null);
  const [catalog, setCatalog] = useState<LoadingBarCatalog | null>(null);
  const [query, setQuery] = useState("");
  const [activeTheme, setActiveTheme] = useState("All");
  const [frameIndex, setFrameIndex] = useState(0);
  const [isVisible, setIsVisible] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const prefersReducedMotion = usePrefersReducedMotion();

  useEffect(() => {
    let cancelled = false;
    fetch("/examples/loading_bar_catalog.json")
      .then((response) => {
        if (!response.ok) throw new Error("Failed to load loading bar catalog");
        return response.json() as Promise<LoadingBarCatalog>;
      })
      .then((data) => {
        if (!cancelled) setCatalog(data);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const node = sectionRef.current;
    if (!node) return;

    if (!("IntersectionObserver" in window)) {
      setIsVisible(true);
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting);
      },
      { rootMargin: "220px 0px" },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (prefersReducedMotion) {
      setFrameIndex(0);
      return;
    }
  }, [prefersReducedMotion]);

  useEffect(() => {
    if (!catalog || !isVisible || prefersReducedMotion) {
      lastTickRef.current = null;
      return;
    }

    let animationFrame = 0;
    const interval = 1000 / Math.max(1, catalog.fps);

    const tick = (time: number) => {
      if (lastTickRef.current === null) lastTickRef.current = time;
      if (time - lastTickRef.current >= interval) {
        setFrameIndex((current) => (current + 1) % catalog.frames_per_style);
        lastTickRef.current = time;
      }
      animationFrame = window.requestAnimationFrame(tick);
    };

    animationFrame = window.requestAnimationFrame(tick);
    return () => window.cancelAnimationFrame(animationFrame);
  }, [catalog, isVisible, prefersReducedMotion]);

  const filteredStyles = useMemo(() => {
    if (!catalog) return [];

    const normalized = query.trim().toLowerCase();
    return catalog.styles.filter((style) => {
      const themeMatches = activeTheme === "All" || style.theme === activeTheme;
      const queryMatches =
        normalized.length === 0 ||
        [style.name, style.theme, style.description, style.command].join(" ").toLowerCase().includes(normalized);
      return themeMatches && queryMatches;
    });
  }, [activeTheme, catalog, query]);

  const total = catalog?.total ?? 0;
  const themeOptions = catalog?.themes ?? [];
  const isFiltering = query.trim().length > 0 || activeTheme !== "All";
  const displayStyles = isFiltering || expanded ? filteredStyles : filteredStyles.slice(0, DEFAULT_VISIBLE);
  const hiddenCount = filteredStyles.length - displayStyles.length;

  return (
    <section id="loading-bars" className="section loading-bars-section" ref={sectionRef}>
      <div className="loading-bars-head">
        <div className="section-heading">
          <span className="eyebrow">Loading bars</span>
          <h2>{total > 0 ? `${total} bundled progress loaders, rendered from the Rust registry.` : "Bundled progress loaders, rendered from the Rust registry."}</h2>
          <p>
            Every card below is generated from `dotmax::progress::all_styles()`, using compact pre-rendered terminal frames from the real Rust style implementations.
          </p>
        </div>
        <div className="loading-bars-count" aria-label="Loading bar catalog count">
          <strong>{displayStyles.length}</strong>
          <span>shown</span>
          <strong>{total}</strong>
          <span>total</span>
        </div>
      </div>

      <div className="loading-bars-toolbar">
        <label className="search-box">
          <Search size={16} />
          <span className="sr-only">Search loading bars</span>
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search wildlife, quantum, sinewave, spinner..."
          />
        </label>
        <div className="loading-theme-strip" aria-label="Loading bar theme filters">
          <button
            className={activeTheme === "All" ? "loading-theme-pill loading-theme-pill-active" : "loading-theme-pill"}
            onClick={() => setActiveTheme("All")}
            type="button"
          >
            All <span>{total}</span>
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

      <div className="loading-bars-grid" data-loading-bar-count={total} data-loading-bar-frame={frameIndex}>
        {displayStyles.map((style) => (
          <article className="loading-bar-card" data-theme={style.theme} key={style.id}>
            <LoadingBarTerminal frameIndex={frameIndex} style={style} width={catalog?.width ?? 44} />
            <div className="loading-bar-card-body">
              <div className="loading-bar-card-meta">
                <span>{style.theme}</span>
                <span>{style.name}</span>
              </div>
              <p>{style.description}</p>
              <code>{style.command}</code>
            </div>
          </article>
        ))}
      </div>

      {!isFiltering && hiddenCount > 0 && (
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
    </section>
  );
}
