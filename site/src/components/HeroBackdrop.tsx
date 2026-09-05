import { useEffect, useRef } from "react";
import { useInView } from "../catalog/inView";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";

/**
 * Ambient braille field behind the hero: a slow interference pattern quantised
 * to braille glyphs by dot count, drawn on one canvas at 10 fps. It only runs
 * while the hero is on screen and freezes to a single frame under
 * prefers-reduced-motion.
 */

// Braille glyphs ordered by how many dots are lit (0..8).
const LEVELS = ["⠀", "⠁", "⠃", "⠇", "⡇", "⡧", "⡷", "⡿", "⣿"];
const CELL_W = 11;
const CELL_H = 18;
const FPS = 10;

function field(x: number, y: number, t: number): number {
  const a = Math.sin(x * 0.55 + t * 0.9);
  const b = Math.sin(y * 0.9 - t * 0.6);
  const c = Math.sin((x + y) * 0.35 + t * 0.4);
  const d = Math.sin(Math.hypot(x - 30, y - 8) * 0.6 - t * 1.1);
  return (a + b + c + d) / 4; // -1..1
}

export function HeroBackdrop() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [ref, inView] = useInView<HTMLDivElement>();
  const reducedMotion = usePrefersReducedMotion();

  useEffect(() => {
    const canvas = canvasRef.current;
    const host = ref.current;
    if (!canvas || !host) return undefined;
    const ctx = canvas.getContext("2d");
    if (!ctx) return undefined;

    let raf = 0;
    let last = 0;
    let cols = 0;
    let rows = 0;
    let dpr = 1;

    function resize() {
      if (!canvas || !host) return;
      dpr = Math.min(2, window.devicePixelRatio || 1);
      const { width, height } = host.getBoundingClientRect();
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      cols = Math.ceil(width / CELL_W) + 1;
      rows = Math.ceil(height / CELL_H) + 1;
    }

    function draw(t: number) {
      if (!canvas || !ctx) return;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.font = `${CELL_H - 4}px ui-monospace, SFMono-Regular, Consolas, monospace`;
      ctx.textBaseline = "top";
      for (let y = 0; y < rows; y += 1) {
        for (let x = 0; x < cols; x += 1) {
          const v = field(x, y, t);
          if (v < 0.05) continue;
          const level = Math.min(8, Math.floor(v * 9));
          if (level === 0) continue;
          const alpha = 0.05 + v * 0.22;
          ctx.fillStyle = `rgba(110, 231, 160, ${alpha.toFixed(3)})`;
          ctx.fillText(LEVELS[level], x * CELL_W, y * CELL_H);
        }
      }
    }

    function loop(time: number) {
      if (time - last >= 1000 / FPS) {
        last = time;
        draw(time / 1000);
      }
      raf = window.requestAnimationFrame(loop);
    }

    resize();
    const observer = typeof ResizeObserver !== "undefined" ? new ResizeObserver(resize) : null;
    observer?.observe(host);

    if (reducedMotion || !inView) {
      draw(1.7);
      return () => observer?.disconnect();
    }
    raf = window.requestAnimationFrame(loop);
    return () => {
      window.cancelAnimationFrame(raf);
      observer?.disconnect();
    };
  }, [inView, reducedMotion, ref]);

  return (
    <div className="hero-backdrop" ref={ref} aria-hidden="true">
      <canvas ref={canvasRef} />
    </div>
  );
}
