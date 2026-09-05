import {
  Activity,
  BookOpen,
  Bot,
  Boxes,
  Braces,
  Camera,
  ChevronRight,
  Clapperboard,
  Code2,
  Command,
  Copy,
  ExternalLink,
  FileText,
  Gauge,
  Github,
  Image as ImageIcon,
  Layers3,
  LoaderCircle,
  Menu,
  Monitor,
  Orbit,
  PackagePlus,
  Palette,
  Play,
  Search,
  SlidersHorizontal,
  Sparkles,
  Terminal,
  X,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { withBase } from "./lib/withBase";
import { AgentSection } from "./components/AgentSection";
import { CodeBlock } from "./components/CodeBlock";
import { CommandPalette, type CommandItem } from "./components/CommandPalette";
import { CopyButton } from "./components/CopyButton";
import { ExampleTerminalPreview, type ExamplePreview } from "./components/ExampleTerminalPreview";
import { HeroBackdrop } from "./components/HeroBackdrop";
import { HeroShowcase } from "./components/HeroShowcase";
import { LoaderMarquee } from "./components/LoaderMarquee";
import { QuickStartSection } from "./components/QuickStartSection";
import { StyleBrowserSection } from "./components/StyleBrowserSection";
import { TuiPatternsSection } from "./components/TuiPatternsSection";
import { WidgetShowcaseSection } from "./components/WidgetShowcaseSection";
import { useReveal } from "./hooks/useReveal";

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

// Secondary sections (quick start, install, examples, gallery) are parked for
// now. Flip SHOW_SECONDARY_SECTIONS to true to bring them back; the nav,
// section observer, command palette, and hero anchors are all gated to match.
const SHOW_SECONDARY_SECTIONS = false;

const navItems = [
  ["Styles", "#loading-bars"],
  ["Build with AI", "#build-with-ai"],
  ["Docs", "#docs"],
] as const;

const heroBadges = [
  {
    src: "https://img.shields.io/github/stars/newjordan/dotmax?style=flat-square&logo=github&logoColor=white&label=stars&color=2a302c&labelColor=101312",
    alt: "GitHub stars",
    href: links.github,
  },
  {
    src: "https://img.shields.io/crates/v/dotmax?style=flat-square&logo=rust&logoColor=white&label=crates.io&color=2a302c&labelColor=101312",
    alt: "crates.io version",
    href: links.crate,
  },
  {
    src: "https://img.shields.io/crates/d/dotmax?style=flat-square&label=downloads&color=2a302c&labelColor=101312",
    alt: "crates.io downloads",
    href: links.crate,
  },
  {
    src: "https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-2a302c?style=flat-square&labelColor=101312",
    alt: "License MIT or Apache-2.0",
    href: links.github,
  },
] as const;

const heroStats = [
  ["BrailleGrid", "cell canvas"],
  ["TerminalRenderer", "ratatui path"],
  ["Frame packs", "browser previews"],
] as const;

type ExampleStatus = "Ready" | "Feature gated" | "Wishlist";
type ExampleCategory = "Rich media" | "Animations" | "Experimental";

type ExampleBlock = {
  title: string;
  category: ExampleCategory;
  icon: LucideIcon;
  status: ExampleStatus;
  command: string;
  source: string;
  description: string;
  preview: ExamplePreview;
  snippet: string;
  tags: string[];
};

const categories = ["All", "Rich media", "Animations", "Experimental"] as const;

const categoryVisuals: Record<(typeof categories)[number], string> = {
  All: withBase("/gallery/color_tiger.png"),
  "Rich media": withBase("/gallery/color_tiger.png"),
  Animations: withBase("/gallery/gottem.png"),
  Experimental: withBase("/gallery/sphere.png"),
};

const exampleBlocks: ExampleBlock[] = [
  {
    title: "Hello Braille",
    category: "Rich media",
    icon: Terminal,
    status: "Ready",
    command: "cargo run --example hello_braille",
    source: `${links.github}/blob/main/examples/hello_braille.rs`,
    description: "The smallest useful start: create a braille grid, set dots, and render it to the terminal.",
    preview: {
      title: "examples/hello_braille.rs",
      image: withBase("/gallery/grid_formation.png"),
      imageAlt: "Braille grid formation rendered in terminal space",
      rows: [
        "dotmax: grid 80x24 ready",
        "braille canvas active",
        "render complete in 1.8 ms",
      ],
    },
    snippet: `use dotmax::prelude::*;

let mut grid = BrailleGrid::new(80, 24)?;
grid.set_dot(12, 8)?;
grid.set_dot(13, 9)?;
show(&grid)?;`,
    tags: ["no feature flags", "grid", "first render"],
  },
  {
    title: "Drawing Primitives",
    category: "Rich media",
    icon: Braces,
    status: "Ready",
    command: "cargo run --example shapes_demo",
    source: `${links.github}/blob/main/examples/shapes_demo.rs`,
    description: "Lines, circles, rectangles, polygons, filled shapes, and density glyphs on one terminal canvas.",
    preview: {
      title: "examples/shapes_demo.rs",
      framePack: withBase("/examples/shapes_demo.json"),
      rows: [
        "primitives",
        "rectangles circles polygons",
        "animated sweep line",
      ],
    },
    snippet: `use dotmax::prelude::*;

let mut grid = BrailleGrid::new(100, 40)?;
draw_line(&mut grid, 0, 0, 160, 60)?;
draw_circle(&mut grid, 72, 32, 18)?;
show(&grid)?;`,
    tags: ["lines", "circles", "shapes"],
  },
  {
    title: "Color Schemes",
    category: "Rich media",
    icon: Palette,
    status: "Ready",
    command: "cargo run --example color_schemes_demo",
    source: `${links.github}/blob/main/examples/color_schemes_demo.rs`,
    description: "Terminal color capability detection, RGB conversion, predefined schemes, and custom palettes.",
    preview: {
      title: "examples/color_schemes_demo.rs",
      framePack: withBase("/examples/color_schemes_demo.json"),
      rows: [
        "terminal truecolor: yes",
        "heatmap | ocean | sunset",
        "palettes rendered: 12",
      ],
    },
    snippet: `use dotmax::{apply_color_scheme, ColorScheme};

let mut grid = BrailleGrid::new(80, 24)?;
apply_color_scheme(&mut grid, ColorScheme::HeatMap)?;
show(&grid)?;`,
    tags: ["truecolor", "ansi", "palettes"],
  },
  {
    title: "Image Viewer",
    category: "Rich media",
    icon: ImageIcon,
    status: "Ready",
    command: "cargo run --example view_image --features image -- image.png",
    source: `${links.github}/blob/main/examples/view_image.rs`,
    description: "Load local image files, resize them for the current terminal, and render them as braille output.",
    preview: {
      title: "examples/view_image.rs",
      image: withBase("/gallery/color_tiger.png"),
      imageAlt: "Color image rendered as terminal braille output",
      rows: ["resized 1024x768 -> 96x42", "braille cells written: 4032", "truecolor render path"],
    },
    snippet: `use dotmax::quick;

quick::show_file("image.png")?;
quick::show_file("animated.gif")?;`,
    tags: ["png", "jpg", "gif", "resize"],
  },
  {
    title: "Dithering Lab",
    category: "Rich media",
    icon: SlidersHorizontal,
    status: "Ready",
    command: "cargo run --example dither_comparison --features image",
    source: `${links.github}/blob/main/examples/dither_comparison.rs`,
    description: "Compare clean thresholding, Floyd-Steinberg, Bayer, and Atkinson output for the same image.",
    preview: {
      title: "examples/dither_comparison.rs",
      image: withBase("/gallery/viper_ascii_art.png"),
      imageAlt: "Dithered terminal art output",
      rows: ["methods: clean | floyd | bayer | atkinson", "threshold 0.52 | gamma 1.00", "side-by-side preview ready"],
    },
    snippet: `ImageRenderer::new()
    .load_from_path("sample.png")?
    .resize(80, 40, true)?
    .dithering(DitheringMethod::FloydSteinberg)
    .render()?;`,
    tags: ["dither", "threshold", "image quality"],
  },
  {
    title: "SVG Rendering",
    category: "Rich media",
    icon: Layers3,
    status: "Feature gated",
    command: "cargo run --example svg_demo --features svg",
    source: `${links.github}/blob/main/examples/svg_demo.rs`,
    description: "Render SVG assets and text-heavy SVG fixtures into terminal-safe raster output.",
    preview: {
      title: "examples/svg_demo.rs",
      image: withBase("/gallery/snake_color_closeup.png"),
      imageAlt: "Detailed terminal rendering used as a raster output preview",
      rows: [
        "svg viewport 512x512",
        "raster pass complete",
        "terminal-safe cells: 3,840",
      ],
    },
    snippet: `cargo run --example svg_demo --features svg
cargo run --example svg_font_quality --features "image svg"`,
    tags: ["svg", "text", "raster"],
  },
  {
    title: "Animation Loop",
    category: "Animations",
    icon: Play,
    status: "Ready",
    command: "cargo run --example simple_animation",
    source: `${links.github}/blob/main/examples/simple_animation.rs`,
    description: "Frame timing, redraw loops, double buffering, and cached sequences for responsive TUI motion.",
    preview: {
      title: "examples/simple_animation.rs",
      framePack: withBase("/examples/simple_animation.json"),
      rows: [
        "fps target 30 | buffer on",
        "frame 0142  x=44  y=21",
        "dirty cells: 18 | draw 0.7 ms",
      ],
    },
    snippet: `AnimationLoop::new(80, 24)
    .fps(30)
    .on_frame(|frame, grid| {
        grid.clear();
        grid.set_dot((frame * 2) % 160, 42)?;
        Ok(true)
    })
    .run()?;`,
    tags: ["fps", "frame loop", "buffering"],
  },
  {
    title: "Loading Spinner",
    category: "Animations",
    icon: LoaderCircle,
    status: "Ready",
    command: "cargo run --example loading_spinner",
    source: `${links.github}/blob/main/examples/animations/loading_spinner.rs`,
    description: "Rotating braille spinner states captured as a static frame stream for lightweight web playback.",
    preview: {
      title: "examples/animations/loading_spinner.rs",
      framePack: withBase("/examples/loading_spinner.json"),
      rows: ["loading.", "spinner states", "braille arc trail"],
    },
    snippet: `let angle = frame as f64 * 36.0_f64.to_radians();
draw_spinner(&mut grid, center_x, center_y, angle, style)?;
renderer.render(&grid)?;`,
    tags: ["spinner", "frame timing", "braille"],
  },
  {
    title: "Waveform",
    category: "Animations",
    icon: Activity,
    status: "Ready",
    command: "cargo run --example waveform",
    source: `${links.github}/blob/main/examples/animations/waveform.rs`,
    description: "Scrolling sine waves rendered with colored line primitives and replayed from compact frames.",
    preview: {
      title: "examples/animations/waveform.rs",
      framePack: withBase("/examples/waveform.json"),
      rows: ["waveform demo", "three phase-shifted waves", "line primitives"],
    },
    snippet: `draw_line_colored(&mut grid, px, py, x, y, color, None)?;
renderer.render(&grid)?;`,
    tags: ["waveform", "lines", "truecolor"],
  },
  {
    title: "Fireworks",
    category: "Animations",
    icon: Sparkles,
    status: "Ready",
    command: "cargo run --example fireworks",
    source: `${links.github}/blob/main/examples/animations/fireworks.rs`,
    description: "Particle bursts with per-cell color fading, captured without running a browser-side Rust process.",
    preview: {
      title: "examples/animations/fireworks.rs",
      framePack: withBase("/examples/fireworks.json"),
      rows: ["fireworks particles", "deterministic bursts", "color fades"],
    },
    snippet: `grid.set_dot(x, y)?;
grid.set_cell_color(x / 2, y / 4, particle.faded_color())?;`,
    tags: ["particles", "color", "deterministic"],
  },
  {
    title: "Loading Bars",
    category: "Rich media",
    icon: LoaderCircle,
    status: "Ready",
    command: "cargo run --example loading_bars",
    source: `${links.github}/blob/main/examples/loading_bars.rs`,
    description: "A catalog of terminal loading bars and progress loaders built as reusable dotmax rendering styles.",
    preview: {
      title: "examples/loading_bars.rs",
      framePack: withBase("/examples/loading_bars.json"),
      rows: [
        "classic       [############------] 62%",
        "sinewave      ~~~^^^~~~^^^~~~",
        "styles rendered: 64",
      ],
    },
    snippet: `for style in dotmax::all_styles() {
    style.render(&BarContext {
        progress: 0.62,
        width: 48,
        frame: 120,
    })?;
}`,
    tags: ["progress", "loaders", "widgets"],
  },
  {
    title: "Video Player",
    category: "Rich media",
    icon: Clapperboard,
    status: "Feature gated",
    command: "cargo run --example video_player --features video -- demo.mp4",
    source: `${links.github}/blob/main/examples/video_player.rs`,
    description: "Decode video frames and route them through the same terminal image renderer.",
    preview: {
      title: "examples/video_player.rs",
      image: withBase("/gallery/gottem.png"),
      imageAlt: "Animated media rendered in a terminal preview",
      rows: ["frame 0188 / 0420 | 24 fps", "decoder -> resize -> braille", "terminal media path"],
    },
    snippet: `cargo run --example video_player --features video -- demo.mp4
cargo run --example render_tuner --features video -- demo.mp4`,
    tags: ["video", "ffmpeg", "tuning"],
  },
  {
    title: "Webcam Tuner",
    category: "Rich media",
    icon: Camera,
    status: "Feature gated",
    command: "cargo run --example webcam_tuner --features video",
    source: `${links.github}/blob/main/examples/webcam_tuner.rs`,
    description: "Live camera input with interactive controls for brightness, contrast, gamma, threshold, and dithering.",
    preview: {
      title: "examples/webcam_tuner.rs",
      image: withBase("/gallery/snake_color_closeup.png"),
      imageAlt: "Live input style terminal tuning preview",
      rows: [
        "device /dev/video0 | 1280x720",
        "brightness  +08  contrast +12",
        "dither floyd-steinberg",
      ],
    },
    snippet: `cargo run --example webcam_viewer --features video
cargo run --example webcam_tuner --features video`,
    tags: ["webcam", "live input", "controls"],
  },
  {
    title: "Raytrace Sphere",
    category: "Experimental",
    icon: Orbit,
    status: "Feature gated",
    command: "cargo run --example raytrace_sphere --features raytracer",
    source: `${links.github}/blob/main/examples/raytrace_sphere.rs`,
    description: "CPU raytraced forms and wireframe experiments rendered back into braille terminal output.",
    preview: {
      title: "examples/raytrace_sphere.rs",
      image: withBase("/gallery/sphere.png"),
      imageAlt: "Wireframe sphere rendered as terminal graphics",
      rows: ["samples 32 | lights 2", "wireframe + shaded braille", "terminal 3D output"],
    },
    snippet: `cargo run --example raytrace_sphere --features raytracer
cargo run --example zone_stream --release --features "raytracer image"`,
    tags: ["3d", "wireframe", "braille"],
  },
  {
    title: "Desktop Pets",
    category: "Experimental",
    icon: Sparkles,
    status: "Wishlist",
    command: "wishlist: not shipped",
    source: `${links.github}/tree/main/dotamax_pets`,
    description: "Parked for now. The pet experiment did not work well enough, so it belongs on the wishlist instead of the examples path.",
    preview: {
      title: "dotamax_pets",
      image: withBase("/gallery/gottem.png"),
      imageAlt: "Experimental animated terminal output placeholder",
      rows: [
        "status: wishlist",
        "ship gate: core examples first",
        "future visual experiment",
      ],
    },
    snippet: `// Wishlist item
// Revisit only after the core examples and docs
// are solid enough for Rust TUI developers.`,
    tags: ["wishlist", "not shipped", "future"],
  },
];

const gallery = [
  {
    title: "Color tiger",
    caption: "Full color image rendering through braille cells.",
    src: withBase("/gallery/color_tiger.png"),
  },
  {
    title: "Viper tuner",
    caption: "Terminal art with live render controls.",
    src: withBase("/gallery/viper_ascii_art.png"),
  },
  {
    title: "Snake closeup",
    caption: "High detail color terminal output.",
    src: withBase("/gallery/snake_color_closeup.png"),
  },
  {
    title: "GIF playback",
    caption: "Animated media rendered in terminal space.",
    src: withBase("/gallery/gottem.png"),
  },
  {
    title: "Grid formation",
    caption: "Oscilloscope-style visualization output.",
    src: withBase("/gallery/grid_formation.png"),
  },
  {
    title: "Audio spectrograph",
    caption: "Dense signal visualizations for CLI tools.",
    src: withBase("/gallery/audio_spectrograph.png"),
  },
  {
    title: "Sphere",
    caption: "Wireframe 3D rendered as terminal graphics.",
    src: withBase("/gallery/sphere.png"),
  },
  {
    title: "OBJ model",
    caption: "Model previews and experimental 3D output.",
    src: withBase("/gallery/snake_head_obj.png"),
  },
];

const docs = [
  {
    title: "Getting started",
    icon: BookOpen,
    href: links.gettingStarted,
    visual: withBase("/gallery/grid_formation.png"),
    text: "Install dotmax, create your first BrailleGrid, draw shapes, and render to a terminal.",
  },
  {
    title: "Examples",
    icon: Boxes,
    href: links.examples,
    visual: withBase("/gallery/viper_ascii_art.png"),
    text: "Run image, animation, color, webcam, video, SVG, primitive, and progress examples.",
  },
  {
    title: "Terminal compatibility",
    icon: Monitor,
    href: links.terminal,
    visual: withBase("/gallery/snake_color_closeup.png"),
    text: "Check viewport behavior across Windows Terminal, WSL, macOS, Linux, and unknown terminals.",
  },
  {
    title: "Performance",
    icon: Gauge,
    href: links.performance,
    visual: withBase("/gallery/audio_spectrograph.png"),
    text: "Use differential rendering, frame timing, and batching to keep terminal output responsive.",
  },
  {
    title: "API docs",
    icon: Code2,
    href: links.docs,
    visual: withBase("/gallery/sphere.png"),
    text: "Browse modules for grids, primitives, color schemes, rendering, animation, and quick helpers.",
  },
];

function Nav({ onOpenPalette, activeSection }: { onOpenPalette: () => void; activeSection: string }) {
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    if (!menuOpen) return undefined;
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") setMenuOpen(false);
    }
    const media = window.matchMedia("(min-width: 1024px)");
    const onResize = () => {
      if (media.matches) setMenuOpen(false);
    };
    window.addEventListener("keydown", onKey);
    media.addEventListener("change", onResize);
    return () => {
      window.removeEventListener("keydown", onKey);
      media.removeEventListener("change", onResize);
    };
  }, [menuOpen]);

  return (
    <header className="sticky top-0 z-40 border-b border-line/70 bg-page/85 backdrop-blur-xl">
      <div className="mx-auto flex max-w-7xl items-center gap-3 px-4 py-3 sm:px-6 lg:px-8">
        <a className="brand-mark" href="#">
          <span>dm</span>
          <strong>dotmax</strong>
        </a>
        <nav className="ml-6 hidden items-center gap-6 text-sm lg:flex" aria-label="Primary">
          {navItems.map(([label, href]) => (
            <a
              className={activeSection === href.slice(1) ? "nav-link nav-link-active" : "nav-link"}
              href={href}
              key={href}
            >
              {label}
            </a>
          ))}
        </nav>
        <button className="nav-cmdk ml-auto" type="button" onClick={onOpenPalette} aria-label="Open command menu">
          <Search size={15} />
          <span className="hidden sm:inline">Search</span>
          <kbd>
            <Command size={11} />K
          </kbd>
        </button>
        <a
          className="icon-button icon-button-light hidden sm:inline-flex"
          href={links.crate}
          target="_blank"
          rel="noreferrer"
          aria-label="Open crates.io"
        >
          <PackagePlus size={17} />
        </a>
        <a
          className="icon-button icon-button-light hidden sm:inline-flex"
          href={links.github}
          target="_blank"
          rel="noreferrer"
          aria-label="Open GitHub"
        >
          <Github size={17} />
        </a>
        <button
          className="icon-button icon-button-light lg:hidden"
          type="button"
          aria-label={menuOpen ? "Close menu" : "Open menu"}
          aria-expanded={menuOpen}
          aria-controls="mobile-nav"
          onClick={() => setMenuOpen((open) => !open)}
        >
          {menuOpen ? <X size={17} /> : <Menu size={17} />}
        </button>
      </div>
      {menuOpen && (
        <nav id="mobile-nav" className="mobile-nav lg:hidden" aria-label="Mobile">
          {navItems.map(([label, href]) => (
            <a
              className={activeSection === href.slice(1) ? "mobile-nav-link mobile-nav-link-active" : "mobile-nav-link"}
              href={href}
              key={href}
              onClick={() => setMenuOpen(false)}
            >
              {label}
              <ChevronRight size={16} />
            </a>
          ))}
          <div className="mobile-nav-links">
            <a className="pill-link" href={links.github} target="_blank" rel="noreferrer">
              <Github size={14} /> GitHub
            </a>
            <a className="pill-link" href={links.crate} target="_blank" rel="noreferrer">
              <PackagePlus size={14} /> crates.io
            </a>
            <a className="pill-link" href={links.docs} target="_blank" rel="noreferrer">
              <BookOpen size={14} /> docs.rs
            </a>
          </div>
        </nav>
      )}
    </header>
  );
}

function Hero() {
  return (
    <section className="hero-docs-shell">
      <HeroBackdrop />
      <div className="hero-docs-grid">
        <div className="relative z-10 flex flex-col justify-center hero-copy-col">
          <span className="eyebrow mb-5 self-start">Open source Rust terminal graphics</span>
          <h1>
            Turn any media into
            <br className="hidden sm:block" /> terminal graphics.
          </h1>
          <p className="hero-lede">
            A Rust crate that renders images, GIFs, video, webcam, SVG, and 3D wireframes as dense Unicode braille — one line of code.
          </p>

          <div className="hero-install-inline" role="group" aria-label="Install command">
            <span className="prompt">$</span>
            <code>{installCommand}</code>
            <CopyButton value={installCommand} />
          </div>

          <div className="mt-6 flex flex-col gap-3 sm:flex-row">
            <a className="button button-primary" href="#loading-bars">
              <Boxes size={18} />
              Browse styles
            </a>
            <a className="button button-secondary" href="#patterns">
              <Play size={18} />
              TUI patterns
            </a>
            <a className="button button-ghost" href={links.github} target="_blank" rel="noreferrer">
              <Github size={18} />
              GitHub
            </a>
          </div>

          <div className="hero-badges">
            {heroBadges.map((badge) => (
              <a href={badge.href} key={badge.alt} target="_blank" rel="noreferrer">
                <img src={badge.src} alt={badge.alt} loading="lazy" />
              </a>
            ))}
          </div>

          <div className="hero-stat-row" aria-label="dotmax API entry points">
            {heroStats.map(([label, value]) => (
              <span key={label}>
                <strong>{label}</strong>
                {value}
              </span>
            ))}
          </div>
        </div>
        <div className="hero-preview-stack">
          <HeroShowcase />
        </div>
      </div>
    </section>
  );
}

const featureRows = [
  ["core", "default", "Grid, primitives, color, animation, progress — no flags."],
  ["image", "+ image", "PNG, JPG, GIF, APNG, BMP, WebP, TIFF."],
  ["svg", "+ svg", "SVG vector graphics rasterized to braille."],
  ["video", "+ video", "Video files & webcam (requires FFmpeg)."],
  ["raytracer", "+ raytracer", "CPU 3D shapes and glTF/OBJ wireframes."],
  ["chess", "+ chess", "Chess board rendering from PGN."],
] as const;

function InstallSection() {
  return (
    <section id="install" className="section border-y border-line bg-surface">
      <div className="section-heading">
        <span className="eyebrow">Install</span>
        <h2>Start small, enable richer renderers when you need them.</h2>
        <p>The base path needs no feature flags. Opt into images, SVG, video, raytracing, and chess as you go.</p>
      </div>
      <div className="grid gap-5 lg:grid-cols-[minmax(0,0.85fr)_1fr] lg:items-start">
        <div className="grid gap-3">
          <CodeBlock code={dependencySnippet} label="Cargo.toml" />
          <CodeBlock code={installCommand} label="terminal" />
          <div className="flex flex-wrap gap-3 text-sm text-muted">
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
        </div>
        <div className="overflow-hidden rounded-xl border border-line bg-panel2">
          {featureRows.map(([name, flag, text]) => (
            <div className="flex items-start gap-4 border-b border-line p-4 last:border-b-0" key={name}>
              <span className="badge badge-accent min-w-[5.5rem] justify-center">{name}</span>
              <div className="min-w-0">
                <p className="text-sm text-paper">{text}</p>
                <code className="mt-1 block font-mono text-[0.7rem] text-subtle">{flag}</code>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function statusClass(status: ExampleStatus) {
  if (status === "Ready") return "status-ready";
  if (status === "Feature gated") return "status-gated";
  return "status-wishlist";
}

function ExamplesSection() {
  const [activeCategory, setActiveCategory] = useState<(typeof categories)[number]>("All");
  const [query, setQuery] = useState("");

  const categoryCounts = useMemo(() => {
    return categories.reduce(
      (counts, category) => {
        counts[category] =
          category === "All" ? exampleBlocks.length : exampleBlocks.filter((example) => example.category === category).length;
        return counts;
      },
      {} as Record<(typeof categories)[number], number>,
    );
  }, []);

  const filteredExamples = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return exampleBlocks.filter((example) => {
      const categoryMatches = activeCategory === "All" || example.category === activeCategory;
      const queryMatches =
        normalized.length === 0 ||
        [example.title, example.category, example.description, example.command, ...example.tags]
          .join(" ")
          .toLowerCase()
          .includes(normalized);

      return categoryMatches && queryMatches;
    });
  }, [activeCategory, query]);

  const categoryGroups = useMemo(() => {
    const shownCategories = activeCategory === "All" ? categories.filter((category) => category !== "All") : [activeCategory];

    return shownCategories
      .map((category) => ({
        category,
        examples: filteredExamples.filter((example) => example.category === category),
      }))
      .filter((group) => group.examples.length > 0);
  }, [activeCategory, filteredExamples]);

  return (
    <section id="examples" className="section">
      <div className="examples-head">
        <div className="section-heading">
          <span className="eyebrow">Examples</span>
          <h2>Output first. Copy attached.</h2>
          <p>
            Each example pairs the visual result with a runnable command and a small code pattern. Search when you know what you need, browse when you do not.
          </p>
        </div>
        <div className="examples-actions">
          <a className="plain-action-link" href={links.examples} target="_blank" rel="noreferrer">
            <Github size={18} />
            Source Folder
          </a>
          <a className="plain-action-link" href={links.docs} target="_blank" rel="noreferrer">
            <BookOpen size={18} />
            API Docs
          </a>
        </div>
      </div>

      <div className="example-browser">
        <aside className="category-sidebar" aria-label="Example categories">
          <label className="search-box">
            <Search size={16} />
            <span className="sr-only">Search examples</span>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search examples..."
            />
          </label>
          <div className="category-menu">
            {categories.map((category) => (
              <button
                aria-pressed={activeCategory === category}
                className={activeCategory === category ? "category-menu-item category-menu-item-active" : "category-menu-item"}
                key={category}
                onClick={() => setActiveCategory(category)}
                type="button"
              >
                <span className="category-menu-thumb">
                  <img src={categoryVisuals[category]} alt="" loading="lazy" />
                </span>
                <span className="category-menu-copy">
                  <strong>{category}</strong>
                  <span>{categoryCounts[category]} panels</span>
                </span>
              </button>
            ))}
          </div>
        </aside>

        <div className="category-gallery">
          {categoryGroups.length === 0 ? (
            <div className="empty-gallery-panel">
              <Search size={18} />
              <span>No visual panels match that search.</span>
            </div>
          ) : (
            categoryGroups.map((group) => (
              <section className="category-panel" key={group.category} aria-labelledby={`category-${group.category}`}>
                <div className="category-panel-head">
                  <div>
                    <span className="eyebrow">Gallery / {group.category}</span>
                    <h3 id={`category-${group.category}`}>{group.category} visual effects</h3>
                  </div>
                  <span>{group.examples.length} panels</span>
                </div>

                <div className="category-panel-grid">
                  {group.examples.map((example) => (
                    <article className="example-card" key={example.title}>
                      <ExampleTerminalPreview preview={example.preview} />
                      <div className="example-card-top">
                        <span className="feature-icon">
                          <example.icon size={18} />
                        </span>
                        <div>
                          <div className="example-meta">
                            <span>{example.category}</span>
                            <span className={statusClass(example.status)}>{example.status}</span>
                          </div>
                          <h3>{example.title}</h3>
                        </div>
                      </div>
                      <p>{example.description}</p>
                      <details className="example-code-details">
                        <summary>
                          <Terminal size={15} />
                          Command and code
                        </summary>
                        <CodeBlock code={example.command} label="run" />
                        <CodeBlock code={example.snippet} label={example.status === "Wishlist" ? "note" : "pattern"} />
                      </details>
                      <div className="example-footer">
                        <div className="tag-row">
                          {example.tags.map((tag) => (
                            <span key={tag}>#{tag}</span>
                          ))}
                        </div>
                        <a className="source-link" href={example.source} target="_blank" rel="noreferrer">
                          Source <ChevronRight size={15} />
                        </a>
                      </div>
                    </article>
                  ))}
                </div>
              </section>
            ))
          )}
        </div>
      </div>
    </section>
  );
}

function GallerySection() {
  return (
    <section id="gallery" className="section gallery-section">
      <div className="section-heading">
        <span className="eyebrow">Gallery</span>
        <h2>Terminal output with enough resolution to matter.</h2>
        <p>
          Real repo artifacts: full-color images, signal views, wireframes, and dense braille surfaces rendered for terminal space.
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
  const useCases = [
    ["Ratatui dashboards", "Charts, status panels, thumbnails, and compact visual summaries.", withBase("/gallery/grid_formation.png")],
    ["CLI inspection tools", "Inspect images, video frames, generated output, and model artifacts in place.", withBase("/gallery/snake_head_obj.png")],
    ["Terminal media", "Render PNG, JPG, GIF, APNG, SVG, video, and webcam streams with features enabled.", withBase("/gallery/gottem.png")],
    ["Games and simulations", "Build tiny terminal scenes, particles, board overlays, and animation loops.", withBase("/gallery/snake_color_closeup.png")],
    ["Monitoring", "Turn telemetry into heatmaps, spectrographs, and dense status surfaces.", withBase("/gallery/audio_spectrograph.png")],
    ["Generative art", "Use braille cells as a high-resolution terminal canvas.", withBase("/gallery/color_tiger.png")],
  ];

  return (
    <section className="section border-y border-line bg-panel/60">
      <div className="section-heading" data-reveal>
        <span className="eyebrow">Use cases</span>
        <h2>Useful when your terminal app needs more than rows of text.</h2>
      </div>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {useCases.map(([title, text, visual]) => (
          <article className="feature-card" key={title} data-reveal>
            <div className="feature-card-visual">
              <img src={visual} alt="" loading="lazy" />
              <Activity size={18} />
            </div>
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
      <div className="section-heading" data-reveal>
        <span className="eyebrow">Docs</span>
        <h2>Follow the same path from first render to production tuning.</h2>
        <p>
          Start locally, run examples, confirm terminal behavior, then reach for API docs when you need specific modules.
        </p>
      </div>
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        {docs.map((doc) => (
          <a className="doc-card" href={doc.href} key={doc.title} target="_blank" rel="noreferrer" data-reveal>
            <div className="doc-card-visual">
              <img src={doc.visual} alt="" loading="lazy" />
              <span className="feature-icon">
                <doc.icon size={18} />
              </span>
            </div>
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
      <div className="open-source-band" data-reveal>
        <div className="open-source-visual">
          <img src={withBase("/gallery/color_tiger.png")} alt="Full color terminal rendering from dotmax" loading="lazy" />
          <div className="open-source-visual-caption">
            <span>terminal render</span>
            <strong>MIT / Apache-2.0</strong>
          </div>
        </div>
        <div>
          <span className="eyebrow">Open source</span>
          <h2>MIT or Apache-2.0, built in public for Rust terminal developers.</h2>
          <p>
            The crate includes focused modules for rendering, grids, primitives, animation, color, quick helpers, media routing, and the `dotmax-braille` CLI binary for image-based workflows.
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

const sectionIds = ["loading-bars", "patterns", "build-with-ai", "docs"] as const;

function useCommandItems(): CommandItem[] {
  return useMemo(() => {
    const origin = typeof window !== "undefined" ? window.location.origin : "";
    const quickActions: CommandItem[] = [
      { id: "qa-install", group: "Quick actions", label: "Copy install command", hint: "⌘I", icon: Copy, action: { type: "copy", value: installCommand } },
      { id: "qa-llms", group: "Quick actions", label: "Copy llms.txt URL", icon: FileText, action: { type: "copy", value: `${origin}${withBase("/llms.txt")}` } },
      { id: "qa-github", group: "Quick actions", label: "Open GitHub repository", icon: Github, action: { type: "link", href: links.github } },
      { id: "qa-docs", group: "Quick actions", label: "Open API docs (docs.rs)", icon: BookOpen, action: { type: "link", href: links.docs } },
      { id: "qa-crate", group: "Quick actions", label: "Open crates.io", icon: PackagePlus, action: { type: "link", href: links.crate } },
    ];

    // Per-example palette entries scroll to #examples, which is parked behind
    // SHOW_SECONDARY_SECTIONS. They return automatically when the flag is on.
    const exampleItems: CommandItem[] = SHOW_SECONDARY_SECTIONS
      ? exampleBlocks.map((example) => ({
          id: `ex-${example.title}`,
          group: "Examples",
          label: example.title,
          hint: example.category,
          icon: example.icon,
          keywords: example.tags.join(" "),
          action: { type: "scroll", target: "#examples" },
        }))
      : [];

    const docItems: CommandItem[] = docs.map((doc) => ({
      id: `doc-${doc.title}`,
      group: "Docs",
      label: doc.title,
      icon: doc.icon,
      action: { type: "link", href: doc.href },
    }));

    const sectionItems: CommandItem[] = [
      // Quick start / Examples / Gallery jump items are parked with their
      // sections behind SHOW_SECONDARY_SECTIONS.
      { id: "sec-bars", group: "Jump to", label: "Style library", icon: LoaderCircle, action: { type: "scroll", target: "#loading-bars" } },
      { id: "sec-patterns", group: "Jump to", label: "TUI patterns", icon: Layers3, action: { type: "scroll", target: "#patterns" } },
      { id: "sec-ai", group: "Jump to", label: "Build with AI", icon: Bot, action: { type: "scroll", target: "#build-with-ai" } },
      { id: "sec-docs", group: "Jump to", label: "Docs", icon: BookOpen, action: { type: "scroll", target: "#docs" } },
    ];

    return [...quickActions, ...exampleItems, ...docItems, ...sectionItems];
  }, []);
}

export default function App() {
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [activeSection, setActiveSection] = useState<string>("");
  const commandItems = useCommandItems();
  useReveal();

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      const typing = target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
      if ((event.key === "k" || event.key === "K") && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      } else if (event.key === "/" && !typing && !paletteOpen) {
        event.preventDefault();
        setPaletteOpen(true);
      } else if ((event.key === "i" || event.key === "I") && (event.metaKey || event.ctrlKey) && !typing) {
        event.preventDefault();
        navigator.clipboard?.writeText(installCommand).catch(() => undefined);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [paletteOpen]);

  useEffect(() => {
    const elements = sectionIds.map((id) => document.getElementById(id)).filter((el): el is HTMLElement => Boolean(el));
    if (elements.length === 0) return undefined;
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries.filter((entry) => entry.isIntersecting).sort((a, b) => b.intersectionRatio - a.intersectionRatio);
        if (visible[0]) setActiveSection(visible[0].target.id);
      },
      { rootMargin: "-45% 0px -45% 0px", threshold: [0, 0.25, 0.5] },
    );
    elements.forEach((el) => observer.observe(el));
    return () => observer.disconnect();
  }, []);

  return (
    <main>
      <Nav onOpenPalette={() => setPaletteOpen(true)} activeSection={activeSection} />
      <Hero />
      <LoaderMarquee />
      <StyleBrowserSection />
      <TuiPatternsSection />
      <WidgetShowcaseSection />
      <AgentSection />
      <DocsSection />
      <UseCasesSection />
      <OpenSourceSection />
      {/* Parked behind SHOW_SECONDARY_SECTIONS — kept in their old relative
          order so re-enabling drops them back in a sane spot. */}
      {SHOW_SECONDARY_SECTIONS && (
        <>
          <QuickStartSection />
          <InstallSection />
          <ExamplesSection />
          <GallerySection />
        </>
      )}
      <Footer />
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} items={commandItems} />
    </main>
  );
}
