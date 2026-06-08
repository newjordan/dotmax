import { Pause, Play } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";

type DotmaxFrame = {
  rows: string[];
  colors?: Array<Array<[number, number, number] | null>>;
};

type DotmaxFramePack = {
  name: string;
  command: string;
  width: number;
  height: number;
  fps: number;
  frames: DotmaxFrame[];
};

export type ExamplePreview = {
  title: string;
  rows: string[];
  framePack?: string;
  image?: string;
  imageAlt?: string;
};

function MiniTerminal({ label, src, fallbackRows }: { label: string; src: string; fallbackRows: string[] }) {
  const rootRef = useRef<HTMLDivElement | null>(null);
  const lastTickRef = useRef<number | null>(null);
  const [pack, setPack] = useState<DotmaxFramePack | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [isVisible, setIsVisible] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const prefersReducedMotion = usePrefersReducedMotion();
  const shouldAnimate = pack && isVisible && !isPaused && !prefersReducedMotion && pack.frames.length > 1;

  useEffect(() => {
    const node = rootRef.current;
    if (!node) return;

    if (!("IntersectionObserver" in window)) {
      setIsVisible(true);
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting);
      },
      { rootMargin: "160px 0px" },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!isVisible || pack) return;

    let cancelled = false;
    fetch(src)
      .then((response) => {
        if (!response.ok) throw new Error(`Failed to load ${src}`);
        return response.json() as Promise<DotmaxFramePack>;
      })
      .then((data) => {
        if (!cancelled) setPack(data);
      })
      .catch(() => {
        if (!cancelled) setPack(null);
      });

    return () => {
      cancelled = true;
    };
  }, [isVisible, pack, src]);

  useEffect(() => {
    if (prefersReducedMotion) {
      setFrameIndex(0);
      return;
    }
  }, [prefersReducedMotion]);

  useEffect(() => {
    if (!pack || !shouldAnimate) {
      lastTickRef.current = null;
      return;
    }

    let animationFrame = 0;
    const interval = 1000 / Math.max(1, pack.fps);

    const tick = (time: number) => {
      if (lastTickRef.current === null) lastTickRef.current = time;
      if (time - lastTickRef.current >= interval) {
        setFrameIndex((current) => (current + 1) % pack.frames.length);
        lastTickRef.current = time;
      }
      animationFrame = window.requestAnimationFrame(tick);
    };

    animationFrame = window.requestAnimationFrame(tick);
    return () => window.cancelAnimationFrame(animationFrame);
  }, [pack, shouldAnimate]);

  const frame = pack?.frames[frameIndex % pack.frames.length];
  const rows = frame?.rows ?? fallbackRows;
  const width = pack?.width ?? Math.max(...fallbackRows.map((row) => row.length), 1);
  const height = pack?.height ?? fallbackRows.length;
  const displayHeight = Math.max(height, 12);

  return (
    <div className="example-terminal mini-terminal" data-frame-index={frameIndex} data-loaded={pack ? "true" : "false"} ref={rootRef}>
      <div className="mini-terminal-top">
        <div className="window-controls" aria-hidden="true">
          <span className="bg-coral" />
          <span className="bg-amber" />
          <span className="bg-terminal" />
        </div>
        <span>{pack?.name ?? label}</span>
        <button
          className="mini-terminal-toggle"
          disabled={!pack || prefersReducedMotion}
          onClick={() => setIsPaused((value) => !value)}
          title={prefersReducedMotion ? "Reduced motion" : isPaused ? "Play preview" : "Pause preview"}
          type="button"
          aria-label={prefersReducedMotion ? "Reduced motion preview" : isPaused ? "Play preview" : "Pause preview"}
        >
          {prefersReducedMotion || isPaused ? <Play size={14} /> : <Pause size={14} />}
        </button>
      </div>
      <div className="mini-terminal-screen" style={{ minHeight: `${displayHeight * 0.9 + 2.5}rem` }}>
        <div className="mini-terminal-grid" style={{ gridTemplateColumns: `repeat(${width}, 1ch)` }} aria-label="Terminal output preview">
          {rows.map((row, rowIndex) =>
            Array.from(row.padEnd(width, "⠀")).slice(0, width).map((character, cellIndex) => {
              const color = frame?.colors?.[rowIndex]?.[cellIndex];
              return (
                <span
                  className="mini-terminal-cell"
                  key={`${rowIndex}-${cellIndex}`}
                  style={color ? { color: `rgb(${color[0]}, ${color[1]}, ${color[2]})` } : undefined}
                >
                  {character}
                </span>
              );
            }),
          )}
        </div>
      </div>
    </div>
  );
}

export function ExampleTerminalPreview({ preview }: { preview: ExamplePreview }) {
  if (preview.framePack) {
    return <MiniTerminal fallbackRows={preview.rows} label={preview.title} src={preview.framePack} />;
  }

  return (
    <div className="example-terminal">
      <div className={preview.image ? "example-terminal-body example-terminal-body-image" : "example-terminal-body"}>
        {preview.image && (
          <div className="example-terminal-media">
            <img className="example-terminal-image" src={preview.image} alt={preview.imageAlt ?? ""} loading="lazy" />
          </div>
        )}
        <div className="example-terminal-bar">
          <div className="window-controls" aria-hidden="true">
            <span className="bg-coral" />
            <span className="bg-amber" />
            <span className="bg-terminal" />
          </div>
          <span>{preview.title}</span>
        </div>
        <div className="example-terminal-lines" aria-label="Terminal output preview">
          {preview.rows.map((row, index) => (
            <p key={`${row}-${index}`}>{row || "\u00a0"}</p>
          ))}
        </div>
      </div>
    </div>
  );
}
