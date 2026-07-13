import { Boxes, PackagePlus, Play } from "lucide-react";
import { withBase } from "../lib/withBase";
import { CopyButton } from "./CopyButton";

const installLine = "cargo add dotmax --features image";

const helloSnippet = `use dotmax::prelude::*;

fn main() -> dotmax::Result<()> {
    let mut grid = grid()?;            // terminal-sized braille canvas
    draw_circle(&mut grid, 50, 25, 20)?;
    draw_line(&mut grid, 0, 0, 100, 50)?;
    show(&grid)?;                      // render to the terminal
    Ok(())
}`;

const steps = [
  {
    icon: PackagePlus,
    title: "Add the crate",
    text: "cargo add dotmax --features image. Core drawing needs no feature flags at all.",
  },
  {
    icon: Play,
    title: "Render anything",
    text: "quick::show_file(\"cat.gif\") plays media in one line; primitives draw straight to a grid.",
  },
  {
    icon: Boxes,
    title: "Drop into your TUI",
    text: "Embed the BrailleGrid output inside any ratatui widget for dense dashboards.",
  },
];

export function QuickStartSection() {
  return (
    <section id="quickstart" className="section">
      <div className="section-heading">
        <span className="eyebrow">Quick start</span>
        <h2>From zero to braille in under a minute.</h2>
        <p>Install, draw a few primitives, and render — then reach for images, animation, and color when you need them.</p>
      </div>

      <div className="quickstart-grid">
        <div className="quickstart-card">
          <div className="quickstart-card-head">
            <span>src/main.rs</span>
            <CopyButton value={helloSnippet} />
          </div>
          <pre>
            <code>{helloSnippet}</code>
          </pre>
        </div>

        <div className="quickstart-card">
          <div className="quickstart-card-head">
            <span>terminal output</span>
            <CopyButton value={installLine} />
          </div>
          <div className="relative flex-1 bg-ink p-3">
            <img
              src={withBase("/gallery/grid_formation.png")}
              alt="A circle and diagonal line drawn with dotmax primitives, rendered as green braille dots in a terminal"
              className="h-full max-h-[280px] w-full rounded-lg object-contain opacity-90"
              loading="lazy"
            />
          </div>
        </div>
      </div>

      <div className="quickstart-steps">
        {steps.map((step, index) => (
          <article className="quickstart-step" key={step.title}>
            <span>{index + 1}</span>
            <strong>{step.title}</strong>
            <p>{step.text}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
