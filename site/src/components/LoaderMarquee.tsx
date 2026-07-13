import { frameHtml } from "../catalog/frameHtml";
import { useTickerFrame } from "../catalog/frameTicker";
import { useInView } from "../catalog/inView";
import { useHeroPack } from "../catalog/packs";
import type { StylePack } from "../catalog/types";
import { usePrefersReducedMotion } from "../hooks/usePrefersReducedMotion";

// Logical cells = the hero styles twice over, so the strip stays dense; the
// whole track is then duplicated once more (aria-hidden) for a seamless wrap.
const REPEATS = 2;

function openStyle(id: string) {
  window.dispatchEvent(new CustomEvent("dotmax:open-style", { detail: { id } }));
}

type CellProps = {
  style: StylePack;
  frame: number;
  phase: number;
  interactive: boolean;
};

function MarqueeCell({ style, frame, phase, interactive }: CellProps) {
  const [, name = ""] = style.id.split("/");
  return (
    <button
      className="loader-marquee-cell"
      type="button"
      tabIndex={interactive ? 0 : -1}
      onClick={() => openStyle(style.id)}
      aria-label={interactive ? `Open ${name} style details` : undefined}
    >
      <pre
        className="loader-marquee-output"
        dangerouslySetInnerHTML={{ __html: frameHtml(style, frame + phase) }}
      />
      <span className="loader-marquee-name">{name}</span>
    </button>
  );
}

export function LoaderMarquee() {
  const pack = useHeroPack();
  const reducedMotion = usePrefersReducedMotion();
  const [ref, inView] = useInView<HTMLDivElement>();
  const frame = useTickerFrame(inView && !reducedMotion && pack !== null);

  if (!pack) return <div className="loader-marquee loader-marquee-empty" ref={ref} />;

  const staticFrame = Math.floor(pack.frames_per_style / 4);
  const shownFrame = reducedMotion ? staticFrame : frame;
  const cells: StylePack[] = [];
  for (let repeat = 0; repeat < REPEATS; repeat += 1) {
    cells.push(...pack.styles);
  }

  return (
    <div
      className="loader-marquee"
      ref={ref}
      data-marquee-cell-count={cells.length}
      aria-label="Live loading-style marquee"
    >
      <div className="loader-marquee-track">
        <div className="loader-marquee-half">
          {cells.map((style, index) => (
            <MarqueeCell
              key={`a-${index}-${style.id}`}
              style={style}
              frame={shownFrame}
              phase={index * 7}
              interactive
            />
          ))}
        </div>
        <div className="loader-marquee-half" aria-hidden="true">
          {cells.map((style, index) => (
            <MarqueeCell
              key={`b-${index}-${style.id}`}
              style={style}
              frame={shownFrame}
              phase={index * 7}
              interactive={false}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
