import { useEffect, useRef, useState } from "react";
import { frameHtml } from "../catalog/frameHtml";
import { currentFrame, useTickerFrame } from "../catalog/frameTicker";
import { useInView } from "../catalog/inView";
import { loadHeroPack, useHeroPack } from "../catalog/packs";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";

// Each 48-frame pack loops in 4s at 12fps; 8s shows exactly two full sweeps
// of a style before the showcase fades to the next one.
const ROTATE_MS = 8000;

// Start the hero.json fetch the moment the bundle evaluates — the preload
// link in index.html has usually finished by the time React mounts.
if (typeof window !== "undefined") {
  loadHeroPack().catch(() => undefined);
}

export function HeroShowcase() {
  const pack = useHeroPack();
  const reducedMotion = usePrefersReducedMotion();
  const [ref, inView] = useInView<HTMLDivElement>();
  const [shown, setShown] = useState(0);
  const [hovered, setHovered] = useState(false);
  // Frame offset so every style starts its run at frame 0 when it appears.
  const offsetRef = useRef(0);

  const frame = useTickerFrame(inView && !reducedMotion && pack !== null);

  const styles = pack?.styles ?? [];
  const style = styles.length > 0 ? styles[shown % styles.length] : null;

  useEffect(() => {
    if (!pack || reducedMotion || !inView || hovered) return undefined;
    const timer = window.setInterval(() => {
      offsetRef.current = currentFrame();
      setShown((current) => (current + 1) % pack.styles.length);
    }, ROTATE_MS);
    return () => window.clearInterval(timer);
  }, [pack, reducedMotion, inView, hovered]);

  const staticFrame = pack ? Math.floor(pack.frames_per_style / 4) : 0;
  const html = style
    ? frameHtml(style, reducedMotion ? staticFrame : frame - offsetRef.current)
    : null;
  const [theme = "", name = ""] = style ? style.id.split("/") : [];

  function select(index: number) {
    offsetRef.current = currentFrame();
    setShown(index);
  }

  function openStyle() {
    if (!style) return;
    window.dispatchEvent(new CustomEvent("dotmax:open-style", { detail: { id: style.id } }));
  }

  return (
    <div
      className="hero-showcase"
      ref={ref}
      data-hero-style={style?.id ?? ""}
      data-hero-frame={reducedMotion ? staticFrame : frame}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div className="terminal-preview hero-showcase-terminal">
        <div className="terminal-top">
          <div className="window-controls" aria-hidden="true">
            <span className="bg-coral" />
            <span className="bg-amber" />
            <span className="bg-terminal" />
          </div>
          <span>{style ? `dotmax · ${theme}/${name}` : "dotmax · loading styles"}</span>
        </div>
        <button
          className="hero-showcase-body"
          type="button"
          onClick={openStyle}
          aria-label={style ? `Open ${name} style details` : "Loading style showcase"}
        >
          {html ? (
            <pre
              key={style?.id}
              className="hero-showcase-output"
              dangerouslySetInnerHTML={{ __html: html }}
            />
          ) : (
            <pre className="hero-showcase-output style-skeleton" aria-hidden="true">
              {Array.from({ length: 4 }, () => "⠐⠀⠂⠀⠁".repeat(9)).join("\n")}
            </pre>
          )}
          <div className="hero-showcase-caption">
            <span className="hero-showcase-live" aria-hidden="true" />
            {style ? (
              <>
                <strong>{name}</strong>
                <span>{theme}</span>
                <span className="hero-showcase-hint">live · 12 fps · click for source</span>
              </>
            ) : (
              <span>warming up the phosphor…</span>
            )}
          </div>
        </button>
      </div>
      {styles.length > 1 && (
        <div className="hero-showcase-dots" role="tablist" aria-label="Showcase styles">
          {styles.map((entry, index) => (
            <button
              key={entry.id}
              role="tab"
              type="button"
              aria-selected={index === shown % styles.length}
              aria-label={`Show ${entry.name}`}
              className={
                index === shown % styles.length
                  ? "hero-showcase-dot hero-showcase-dot-active"
                  : "hero-showcase-dot"
              }
              onClick={() => select(index)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
