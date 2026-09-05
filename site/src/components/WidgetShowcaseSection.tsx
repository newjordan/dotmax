import { Camera, Clapperboard, LoaderCircle, Orbit, type LucideIcon } from "lucide-react";

type Widget = {
  title: string;
  icon: LucideIcon;
  kicker: string;
  text: string;
  command: string;
  options: string[];
};

const widgets: Widget[] = [
  {
    title: "Rich media players",
    icon: Clapperboard,
    kicker: "GIF, APNG, video, image browser, universal media",
    text: "Start with quick display, then move into dedicated players and tuners when the experience needs more control.",
    command: "cargo run --example video_player --features video -- demo.mp4\ncargo run --example animated_gif --features image -- demo.gif",
    options: ["VideoPlayer", "GifPlayer", "ApngPlayer", "render_tuner"],
  },
  {
    title: "Webcam player",
    icon: Camera,
    kicker: "Live camera feed, device selection, interactive tuning",
    text: "Use the webcam examples as a ready-made terminal camera surface with controls for dithering, color, brightness, contrast, and gamma.",
    command: "cargo run --example webcam_viewer --features video\ncargo run --example webcam_tuner --features video",
    options: ["show_webcam()", "WebcamPlayer", "list_webcams()", "webcam_tuner"],
  },
  {
    title: "Raytracing",
    icon: Orbit,
    kicker: "Braille wireframes, OBJ-style scenes, shaded terminal forms",
    text: "Feature-gated raytracing turns the braille canvas into a compact renderer for spheres, rotating solids, and model previews.",
    command: "cargo run --example raytrace_sphere --features raytracer\ncargo run --example zone_stream --release --features \"raytracer image\"",
    options: ["raytrace_sphere", "wireframe", "zone_stream", "BrailleGrid output"],
  },
  {
    title: "Loading bar animations",
    icon: LoaderCircle,
    kicker: "Progress widgets that behave like tiny terminal scenes",
    text: "Present progress as more than a filled row: meters, lasers, plants, music, cosmos, quantum, platonic solids, and yantras.",
    command: "cargo run --example loading_bars\ncargo run --example loading_bar_sheet",
    options: ["meter", "lasers", "plants", "music", "cosmos", "quantum"],
  },
];

export function WidgetShowcaseSection() {
  return (
    <section id="widgets" className="section border-y border-line bg-surface">
      <div className="section-heading" data-reveal>
        <span className="eyebrow">Community widgets</span>
        <h2>Ship a widget first, then expand it into your own terminal surface.</h2>
        <p>Dotmax examples map cleanly to useful app primitives: media players, camera panels, raytraced previews, and animated progress widgets.</p>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        {widgets.map(({ title, icon: Icon, kicker, text, command, options }) => (
          <article className="widget-showcase-card rounded-xl border border-line bg-panel p-5 shadow-card" key={title} data-reveal>
            <div className="flex items-start gap-3">
              <span className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-terminal/25 bg-terminal/10 text-terminal"><Icon size={19} /></span>
              <div><h3 className="text-lg font-semibold text-paper">{title}</h3><p className="mt-1 text-sm text-muted">{kicker}</p></div>
            </div>
            <p className="mt-4 text-sm leading-6 text-muted">{text}</p>
            <pre className="mt-4 overflow-x-auto rounded-lg border border-line bg-ink text-xs"><code>{command}</code></pre>
            <div className="mt-4 flex flex-wrap gap-2">{options.map((option) => <span className="rounded-full border border-line bg-panel2 px-2.5 py-1 font-mono text-[0.68rem] text-subtle" key={option}>{option}</span>)}</div>
          </article>
        ))}
      </div>
    </section>
  );
}
