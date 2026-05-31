import {
  BookOpen,
  Boxes,
  Braces,
  Camera,
  Check,
  ChevronRight,
  Clapperboard,
  Clipboard,
  Code2,
  ExternalLink,
  Gauge,
  Github,
  Image as ImageIcon,
  LoaderCircle,
  Monitor,
  Orbit,
  PackagePlus,
  Palette,
  Play,
  Radio,
  Search,
} from "lucide-react";
import { useState } from "react";

const links = {
  github: "https://github.com/newjordan/dotmax",
  docs: "https://docs.rs/dotmax",
  crate: "https://crates.io/crates/dotmax",
  repoDocs: "https://github.com/newjordan/dotmax/tree/main/docs",
  gettingStarted: "https://github.com/newjordan/dotmax/blob/main/docs/getting_started.md",
  examples: "https://github.com/newjordan/dotmax/tree/main/examples",
  terminal: "https://github.com/newjordan/dotmax/blob/main/docs/terminal-compatibility.md",
  performance: "https://github.com/newjordan/dotmax/blob/main/docs/performance.md",
};

const installCommand = "cargo add dotmax --features image";
const dependencySnippet = `[dependencies]
dotmax = { version = "0.1", features = ["image"] }`;

const codeSnippets = [
  {
    title: "Grid drawing",
    icon: Braces,
    code: `use dotmax::prelude::*;

let mut grid = grid()?;
draw_line(&mut grid, 0, 0, 100, 50)?;
draw_circle(&mut grid, 50, 25, 20)?;
show(&grid)?;`,
  },
  {
    title: "Images",
    icon: ImageIcon,
    code: `use dotmax::quick;

quick::show_file("photo.png")?;
quick::show_file("cat.gif")?;`,
  },
  {
    title: "Animation",
    icon: Play,
    code: `use dotmax::animation::AnimationLoop;

AnimationLoop::new(80, 24)
    .fps(30)
    .on_frame(|frame, grid| {
        grid.clear();
        grid.set_dot((frame * 2) % 160, 48)?;
        Ok(true)
    })
    .run()?;`,
  },
  {
    title: "Color",
    icon: Palette,
    code: `use dotmax::{BrailleGrid, Color};
use dotmax::primitives::draw_circle_colored;

let mut grid = BrailleGrid::new(80, 24)?;
grid.enable_color_support();
draw_circle_colored(&mut grid, 80, 48, 30, Color::new(255, 80, 40))?;`,
  },
];

const gallery = [
  {
    title: "Color tiger",
    caption: "Full color image rendering through braille cells.",
    src: "/gallery/color_tiger.png",
  },
  {
    title: "Viper tuner",
    caption: "Terminal art with live render controls.",
    src: "/gallery/viper_ascii_art.png",
  },
  {
    title: "Snake closeup",
    caption: "High detail color terminal output.",
    src: "/gallery/snake_color_closeup.png",
  },
  {
    title: "GIF playback",
    caption: "Animated media rendered in terminal space.",
    src: "/gallery/gottem.png",
  },
  {
    title: "Grid formation",
    caption: "Oscilloscope-style visualization output.",
    src: "/gallery/grid_formation.png",
  },
  {
    title: "Audio spectrograph",
    caption: "Dense signal visualizations for CLI tools.",
    src: "/gallery/audio_spectrograph.png",
  },
  {
    title: "Sphere",
    caption: "Wireframe 3D rendered as terminal graphics.",
    src: "/gallery/sphere.png",
  },
  {
    title: "OBJ model",
    caption: "Model previews and experimental 3D output.",
    src: "/gallery/snake_head_obj.png",
  },
];

const useCases = [
  ["Ratatui dashboards", "Charts, status panels, thumbnails, and compact visual summaries."],
  ["CLI visual tools", "Inspect images, video frames, generated output, and model artifacts in place."],
  ["Terminal media", "Render PNG, JPG, GIF, APNG, SVG, video, and webcam streams with features enabled."],
  ["Games", "Build tiny terminal scenes, particles, board overlays, and animation loops."],
  ["Monitoring", "Turn telemetry into heatmaps, spectrographs, and dense status surfaces."],
  ["Generative art", "Use braille cells as a high-resolution terminal canvas."],
];

const widgetCategories = [
  {
    title: "Rich media players",
    icon: Clapperboard,
    kicker: "GIF, APNG, video, image browser, universal media",
    text: "Drop media into terminal tools without building your own frame loop first. Start with quick display, then move into dedicated players and tuners.",
    command: `cargo run --example video_player --features video -- demo.mp4
cargo run --example animated_gif --features image -- demo.gif
cargo run --example render_tuner --features video -- demo.mp4`,
    options: ["VideoPlayer", "GifPlayer", "ApngPlayer", "render_tuner", "image_browser"],
  },
  {
    title: "Webcam player",
    icon: Camera,
    kicker: "Live camera feed, device selection, interactive tuning",
    text: "Use the webcam examples as a ready-made terminal camera surface, then tune dithering, threshold, color mode, brightness, contrast, and gamma.",
    command: `cargo run --example webcam_viewer --features video
cargo run --example webcam_tuner --features video`,
    options: ["show_webcam()", "WebcamPlayer", "list_webcams()", "webcam_tuner"],
  },
  {
    title: "Raytracing",
    icon: Orbit,
    kicker: "Braille wireframes, OBJ-style scenes, shaded terminal forms",
    text: "Feature-gated raytracing turns the braille canvas into a compact renderer for spheres, rotating solids, model previews, and experimental scenes.",
    command: `cargo run --example raytrace_sphere --features raytracer
cargo run --example zone_stream --release --features "raytracer image"`,
    options: ["raytrace_sphere", "wireframe", "zone_stream", "BrailleGrid output"],
  },
  {
    title: "Loading bar animations",
    icon: LoaderCircle,
    kicker: "Progress widgets that behave like tiny terminal scenes",
    text: "Present progress as more than a filled row: meters, lasers, plants, music, cosmos, quantum, platonic solids, yantras, and medieval gates.",
    command: `cargo run --example loading_bars
cargo run --example loading_bar_sheet`,
    options: ["meter", "lasers", "plants", "music", "cosmos", "quantum", "platonic", "yantra", "medieval"],
  },
];

const docs = [
  {
    title: "Getting started",
    icon: BookOpen,
    href: links.gettingStarted,
    text: "Install dotmax, create your first BrailleGrid, draw shapes, and render to a terminal.",
  },
  {
    title: "Examples",
    icon: Boxes,
    href: links.examples,
    text: "Run image, animation, color, webcam, video, SVG, and primitive drawing examples.",
  },
  {
    title: "Terminal compatibility",
    icon: Monitor,
    href: links.terminal,
    text: "Understand viewport behavior across Windows Terminal, WSL, macOS, Linux, and unknown terminals.",
  },
  {
    title: "Performance",
    icon: Gauge,
    href: links.performance,
    text: "Use differential rendering, frame timing, and batching to keep terminal output responsive.",
  },
  {
    title: "API docs",
    icon: Code2,
    href: links.docs,
    text: "Browse modules for grids, primitives, color schemes, rendering, animation, and quick helpers.",
  },
];

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  return (
    <button className="icon-button" onClick={copy} type="button" aria-label="Copy command">
      {copied ? <Check size={16} /> : <Clipboard size={16} />}
    </button>
  );
}

function CodeBlock({ code, label }: { code: string; label: string }) {
  return (
    <div className="code-shell">
      <div className="code-shell-bar">
        <span>{label}</span>
        <CopyButton value={code} />
      </div>
      <pre>
        <code>{code}</code>
      </pre>
    </div>
  );
}

function Nav() {
  return (
    <header className="sticky top-0 z-40 border-b border-line/80 bg-ink/90 backdrop-blur">
      <div className="mx-auto flex max-w-7xl items-center gap-3 px-4 py-3 sm:px-6 lg:px-8">
        <a className="flex min-w-0 items-center gap-2 font-mono text-sm font-semibold text-paper" href="#">
          <span className="flex size-8 items-center justify-center border border-terminal/40 bg-terminal/10 text-terminal">
            dm
          </span>
          <span>dotmax</span>
        </a>
        <nav className="ml-auto hidden items-center gap-6 text-sm text-muted md:flex">
          <a href="#install">Install</a>
          <a href="#build">Build</a>
          <a href="#widgets">Widgets</a>
          <a href="#gallery">Gallery</a>
          <a href="#docs">Docs</a>
        </nav>
        <a className="command-link ml-auto md:ml-4" href="#docs">
          <Search size={15} />
          <span className="hidden sm:inline">Jump to docs</span>
          <kbd>/</kbd>
        </a>
        <a className="icon-button" href={links.github} target="_blank" rel="noreferrer" aria-label="Open GitHub">
          <Github size={17} />
        </a>
      </div>
    </header>
  );
}

function TerminalPreview() {
  const rows = [
    "$ cargo run --example color_image --features image",
    "dotmax: detected terminal 120x36, truecolor",
    "",
    "frame 183  fps 60  palette heat_map",
    "set_dot batch        0.18 ms",
    "resize pipeline      7.90 ms",
    "terminal render      2.10 ms",
  ];

  return (
    <div className="terminal-preview">
      <div className="terminal-top">
        <div className="window-controls" aria-hidden="true">
          <span className="bg-coral" />
          <span className="bg-amber" />
          <span className="bg-terminal" />
        </div>
        <span>examples/color_image.rs</span>
      </div>
      <div className="terminal-body">
        <img src="/gallery/snake_color_closeup.png" alt="dotmax terminal screenshot" />
        <div className="terminal-overlay">
          {rows.map((row, index) => (
            <p key={index}>{row || "\u00a0"}</p>
          ))}
        </div>
      </div>
    </div>
  );
}

function Hero() {
  return (
    <section className="relative overflow-hidden border-b border-line">
      <div className="dot-matrix" aria-hidden="true" />
      <div className="mx-auto grid max-w-7xl gap-6 px-4 py-8 sm:gap-10 sm:px-6 sm:py-20 lg:grid-cols-[0.95fr_1.05fr] lg:px-8 lg:py-24">
        <div className="relative z-10 flex flex-col justify-center">
          <div className="mb-5 flex flex-wrap items-center gap-2 font-mono text-xs uppercase tracking-[0.18em] text-terminal">
            <span className="status-dot" />
            Open source Rust terminal graphics
          </div>
          <h1 className="text-5xl font-black leading-none text-paper sm:text-8xl lg:text-9xl">dotmax</h1>
          <p className="mt-5 max-w-2xl text-xl leading-8 text-muted sm:text-2xl">
            Braille and terminal graphics for Rust TUIs.
          </p>
          <p className="mt-3 max-w-2xl text-base leading-7 text-muted">
            Build dense terminal canvases, image previews, animation loops, color visualizations, and media-aware CLI tools with a small Rust API.
          </p>
          <div className="mt-6 flex flex-col gap-3 sm:flex-row">
            <a className="button button-primary" href="#install">
              <PackagePlus size={18} />
              Get Started
            </a>
            <a className="button button-secondary" href="#gallery">
              <ImageIcon size={18} />
              View Examples
            </a>
            <a className="button button-ghost" href={links.github} target="_blank" rel="noreferrer">
              <Github size={18} />
              GitHub
            </a>
          </div>
        </div>
        <TerminalPreview />
      </div>
    </section>
  );
}

function InstallSection() {
  return (
    <section id="install" className="section">
      <div className="section-heading">
        <span className="eyebrow">Install in one command</span>
        <h2>Start with image-ready terminal graphics.</h2>
        <p>
          Use the `image` feature for PNG, JPG, GIF, APNG, BMP, WebP, and TIFF rendering. Pin with the `0.1` line until you need an exact patch.
        </p>
      </div>
      <div className="grid gap-5 lg:grid-cols-[1.05fr_0.95fr]">
        <CodeBlock code={installCommand} label="shell" />
        <CodeBlock code={dependencySnippet} label="Cargo.toml" />
      </div>
      <div className="mt-5 flex flex-wrap gap-3 text-sm text-muted">
        <a className="pill-link" href={links.crate} target="_blank" rel="noreferrer">
          crates.io <ExternalLink size={14} />
        </a>
        <a className="pill-link" href={links.docs} target="_blank" rel="noreferrer">
          docs.rs <ExternalLink size={14} />
        </a>
        <a className="pill-link" href={links.gettingStarted} target="_blank" rel="noreferrer">
          getting started <ExternalLink size={14} />
        </a>
      </div>
    </section>
  );
}

function BuildSection() {
  return (
    <section id="build" className="section border-y border-line bg-panel/60">
      <div className="section-heading">
        <span className="eyebrow">Build terminal graphics</span>
        <h2>Grid primitives, images, animation, and color share one canvas model.</h2>
        <p>
          Dotmax gives Rust TUI developers a direct path from a terminal-sized grid to media-rich braille output.
        </p>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        {codeSnippets.map((snippet) => (
          <article className="snippet-card" key={snippet.title}>
            <div className="mb-4 flex items-center gap-3">
              <span className="feature-icon">
                <snippet.icon size={18} />
              </span>
              <h3>{snippet.title}</h3>
            </div>
            <CodeBlock code={snippet.code} label="rust" />
          </article>
        ))}
      </div>
    </section>
  );
}

function WidgetsSection() {
  return (
    <section id="widgets" className="section">
      <div className="section-heading">
        <span className="eyebrow">Immediate options</span>
        <h2>Ship a widget first, then expand it into your own terminal surface.</h2>
        <p>
          Dotmax examples map cleanly to useful app primitives: media players, camera panels, raytraced previews, and animated progress widgets.
        </p>
      </div>
      <div className="widget-grid">
        {widgetCategories.map((category) => (
          <article className="widget-card" key={category.title}>
            <div className="widget-card-head">
              <span className="feature-icon">
                <category.icon size={18} />
              </span>
              <div>
                <h3>{category.title}</h3>
                <p>{category.kicker}</p>
              </div>
            </div>
            <p className="widget-card-copy">{category.text}</p>
            <CodeBlock code={category.command} label="try it" />
            <div className="widget-options">
              {category.options.map((option) => (
                <span key={option}>{option}</span>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function GallerySection() {
  return (
    <section id="gallery" className="section">
      <div className="section-heading">
        <span className="eyebrow">Gallery</span>
        <h2>Real terminal graphics from the dotmax examples.</h2>
        <p>
          These are existing repo screenshots, copied from the docs gallery so the site can deploy as a standalone static app.
        </p>
      </div>
      <div className="gallery-grid">
        {gallery.map((item, index) => (
          <figure className={index === 0 ? "gallery-item gallery-featured" : "gallery-item"} key={item.src}>
            <img src={item.src} alt={item.title} loading={index > 1 ? "lazy" : "eager"} />
            <figcaption>
              <strong>{item.title}</strong>
              <span>{item.caption}</span>
            </figcaption>
          </figure>
        ))}
      </div>
    </section>
  );
}

function UseCasesSection() {
  return (
    <section className="section border-y border-line bg-panel/60">
      <div className="section-heading">
        <span className="eyebrow">Use cases</span>
        <h2>Designed for serious terminal-native interfaces.</h2>
      </div>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {useCases.map(([title, text]) => (
          <article className="feature-card" key={title}>
            <Radio size={18} />
            <h3>{title}</h3>
            <p>{text}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

function DocsSection() {
  return (
    <section id="docs" className="section">
      <div className="section-heading">
        <span className="eyebrow">Docs</span>
        <h2>Follow the same path from first render to production tuning.</h2>
        <p>
          Start locally, run examples, confirm terminal behavior, then reach for API docs when you need specific modules.
        </p>
      </div>
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        {docs.map((doc) => (
          <a className="doc-card" href={doc.href} key={doc.title} target="_blank" rel="noreferrer">
            <span className="feature-icon">
              <doc.icon size={18} />
            </span>
            <h3>{doc.title}</h3>
            <p>{doc.text}</p>
            <ChevronRight size={16} />
          </a>
        ))}
      </div>
    </section>
  );
}

function OpenSourceSection() {
  return (
    <section className="section border-t border-line">
      <div className="open-source-band">
        <div>
          <span className="eyebrow">Open source</span>
          <h2>MIT or Apache-2.0, built in public for Rust terminal developers.</h2>
          <p>
            The crate includes focused modules for rendering, grids, primitives, animation, color, quick helpers, and the `dotmax-braille` CLI binary for image-based workflows.
          </p>
        </div>
        <div className="open-source-actions">
          <a className="button button-primary" href={links.github} target="_blank" rel="noreferrer">
            <Github size={18} />
            Repository
          </a>
          <a className="button button-secondary" href={links.docs} target="_blank" rel="noreferrer">
            <BookOpen size={18} />
            API Docs
          </a>
          <a className="button button-ghost" href={links.crate} target="_blank" rel="noreferrer">
            <PackagePlus size={18} />
            Crate
          </a>
        </div>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer className="border-t border-line px-4 py-8 text-sm text-muted sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-7xl flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <p>dotmax: braille and terminal graphics for Rust TUIs.</p>
        <div className="flex flex-wrap gap-4">
          <a href={links.github} target="_blank" rel="noreferrer">
            GitHub
          </a>
          <a href={links.docs} target="_blank" rel="noreferrer">
            docs.rs
          </a>
          <a href={links.repoDocs} target="_blank" rel="noreferrer">
            Docs
          </a>
        </div>
      </div>
    </footer>
  );
}

export default function App() {
  return (
    <main>
      <Nav />
      <Hero />
      <InstallSection />
      <BuildSection />
      <WidgetsSection />
      <GallerySection />
      <UseCasesSection />
      <DocsSection />
      <OpenSourceSection />
      <Footer />
    </main>
  );
}
