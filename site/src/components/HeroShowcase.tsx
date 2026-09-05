import { ChevronLeft, ChevronRight, Shuffle } from "lucide-react";
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
  const [focused, setFocused] = useState(false);
  // Bumps whenever the user picks a style so the rotation timer (and the
  // progress bar) restart from zero instead of mid-cycle.
  const [cycle, setCycle] = useState(0);
  // Frame offset so every style starts its run at frame 0 when it appears.
  const offsetRef = useRef(0);
  // Touch swipe: horizontal drags over the terminal step styles.
  const touchRef = useRef<{ x: number; y: number; t: number } | null>(null);

  const frame = useTickerFrame(inView && !reducedMotion && pack !== null);

  const styles = pack?.styles ?? [];
  const count = styles.length;
  const style = count > 0 ? styles[shown % count] : null;
  const paused = hovered || focused;

  useEffect(() => {
    if (!pack || reducedMotion || !inView || paused) return undefined;
    const timer = window.setInterval(() => {
      offsetRef.current = currentFrame();
      setShown((current) => (current + 1) % pack.styles.length);
    }, ROTATE_MS);
    return () => window.clearInterval(timer);
    // `cycle` is a dependency on purpose: a manual pick restarts the interval.
  }, [pack, reducedMotion, inView, paused, cycle]);

  const staticFrame = pack ? Math.floor(pack.frames_per_style / 4) : 0;
  const html = style
    ? frameHtml(style, reducedMotion ? staticFrame : frame - offsetRef.current)
    : null;
  const [theme = "", name = ""] = style ? style.id.split("/") : [];

  function select(index: number) {
    if (count === 0) return;
    offsetRef.current = currentFrame();
    setShown(((index % count) + count) % count);
    setCycle((current) => current + 1);
  }

  function shuffle() {
    if (count < 2) return;
    let pick = shown % count;
    while (pick === shown % count) pick = Math.floor(Math.random() * count);
    select(pick);
  }

  function openStyle() {
    if (!style) return;
    window.dispatchEvent(new CustomEvent("dotmax:open-style", { detail: { id: style.id } }));
  }

  function onKeyDown(event: React.KeyboardEvent) {
    if (event.key === "ArrowRight") {
      event.preventDefault();
      select(shown + 1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      select(shown - 1);
    }
  }

  const active = shown % Math.max(1, count);

  function onTouchStart(event: React.TouchEvent) {
    const touch = event.touches[0];
    touchRef.current = touch ? { x: touch.clientX, y: touch.clientY, t: Date.now() } : null;
  }

  function onTouchEnd(event: React.TouchEvent) {
    const start = touchRef.current;
    touchRef.current = null;
    const touch = event.changedTouches[0];
    if (!start || !touch) return;
    const dx = touch.clientX - start.x;
    const dy = touch.clientY - start.y;
    // A deliberate horizontal flick: mostly sideways, at least 40px, under 600ms.
    if (Math.abs(dx) < 40 || Math.abs(dx) < Math.abs(dy) * 1.5 || Date.now() - start.t > 600) return;
    select(dx < 0 ? shown + 1 : shown - 1);
  }

  return (
    <div
      className="hero-showcase"
      ref={ref}
      data-hero-style={style?.id ?? ""}
      data-hero-frame={reducedMotion ? staticFrame : frame}
      data-hero-paused={paused ? "true" : "false"}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onFocus={() => setFocused(true)}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setFocused(false);
      }}
      onKeyDown={onKeyDown}
      onTouchStart={onTouchStart}
      onTouchEnd={onTouchEnd}
    >
      <div className="terminal-preview hero-showcase-terminal">
        <div className="terminal-top">
          <div className="window-controls" aria-hidden="true">
            <span className="bg-coral" />
            <span className="bg-amber" />
            <span className="bg-terminal" />
          </div>
          <span className="hero-showcase-title">{style ? `dotmax · ${theme}/${name}` : "dotmax · loading styles"}</span>
          {count > 1 && (
            <div className="hero-showcase-controls" role="group" aria-label="Showcase controls">
              <button type="button" className="hero-showcase-control" onClick={() => select(shown - 1)} aria-label="Previous style">
                <ChevronLeft size={14} />
              </button>
              <button type="button" className="hero-showcase-control" onClick={shuffle} aria-label="Random style">
                <Shuffle size={13} />
              </button>
              <button type="button" className="hero-showcase-control" onClick={() => select(shown + 1)} aria-label="Next style">
                <ChevronRight size={14} />
              </button>
            </div>
          )}
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
        {!reducedMotion && style && (
          <div className="hero-showcase-progress" aria-hidden="true">
            <span key={`${style.id}-${cycle}`} style={{ animationDuration: `${ROTATE_MS}ms` }} />
          </div>
        )}
      </div>
      {count > 1 && (
        <div className="hero-showcase-dots" role="tablist" aria-label="Showcase styles">
          {styles.map((entry, index) => (
            <button
              key={entry.id}
              role="tab"
              type="button"
              aria-selected={index === active}
              aria-label={`Show ${entry.name}`}
              title={entry.id}
              className={index === active ? "hero-showcase-dot hero-showcase-dot-active" : "hero-showcase-dot"}
              onClick={() => select(index)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
