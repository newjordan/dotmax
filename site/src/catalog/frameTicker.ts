import { useEffect, useState } from "react";

/**
 * One shared requestAnimationFrame loop driving every animated preview at the
 * catalog's fps. It only runs while something is subscribed, so off-screen
 * cards cost nothing.
 */

const FPS = 12;

type Listener = (frame: number) => void;

const listeners = new Set<Listener>();
let frame = 0;
let rafId = 0;
let lastTick: number | null = null;

function loop(time: number) {
  if (lastTick === null) lastTick = time;
  if (time - lastTick >= 1000 / FPS) {
    frame += 1;
    lastTick = time;
    listeners.forEach((listener) => listener(frame));
  }
  rafId = window.requestAnimationFrame(loop);
}

export function currentFrame(): number {
  return frame;
}

export function subscribeFrames(listener: Listener): () => void {
  listeners.add(listener);
  if (listeners.size === 1) {
    lastTick = null;
    rafId = window.requestAnimationFrame(loop);
  }
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) window.cancelAnimationFrame(rafId);
  };
}

/** The advancing frame counter while `active`; frozen otherwise. */
export function useTickerFrame(active: boolean): number {
  const [value, setValue] = useState(frame);
  useEffect(() => {
    if (!active) return undefined;
    setValue(frame);
    return subscribeFrames(setValue);
  }, [active]);
  return value;
}
