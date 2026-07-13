import { Bot, Check, ClipboardCopy, ExternalLink, FileText, Terminal } from "lucide-react";
import { withBase } from "../lib/withBase";
import { useState } from "react";

const githubUrl = "https://github.com/newjordan/dotmax";

function siteOrigin() {
  if (typeof window !== "undefined" && window.location?.origin) return window.location.origin;
  return "https://github.com/newjordan/dotmax";
}

function CopyPill({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      className={copied ? "copy-pill copy-pill-copied" : "copy-pill"}
      onClick={() => {
        navigator.clipboard?.writeText(value).catch(() => undefined);
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1400);
      }}
    >
      {copied ? <Check size={16} /> : <ClipboardCopy size={16} />}
      {copied ? "Copied" : label}
    </button>
  );
}

export function AgentSection() {
  const llmsUrl = `${siteOrigin()}${withBase("/llms.txt")}`;

  const agentPrompt = `You are adding terminal graphics to a Rust project with the \`dotmax\` crate.

Install: cargo add dotmax --features image
Render images, GIFs, video, and webcam as Unicode braille in the terminal.

Core API:
- dotmax::quick::show_file("path")        // any image/gif/video, one line
- dotmax::grid::BrailleGrid               // the cell canvas
- dotmax::prelude::*                       // draw_line / draw_circle / draw_rectangle
- dotmax::animation::AnimationLoop         // .fps(n).on_frame(..).run()
- dotmax::progress::all_styles()           // 664 progress/loading-bar styles

Full machine-readable reference: ${llmsUrl}
Write idiomatic, compiling Rust. Ask before enabling the "video" feature (it needs FFmpeg).`;

  const cursorSetup = `# Point your AI editor at the dotmax reference
# Cursor / Claude Code / Copilot: add this URL as context
${llmsUrl}`;

  return (
    <section id="build-with-ai" className="section ai-section">
      <div className="section-heading">
        <span className="eyebrow">Agents &amp; AI tools</span>
        <h2>Built to be built by agents.</h2>
        <p>
          dotmax ships a machine-readable reference and copy-ready prompts so coding agents write
          correct, compiling Rust on the first try — no hallucinated APIs.
        </p>
      </div>

      <div className="ai-grid">
        <article className="ai-card">
          <span className="ai-card-icon">
            <Bot size={20} />
          </span>
          <h3>Copy an agent prompt</h3>
          <p>
            A ready-to-paste prompt for Cursor, Claude Code, or Copilot with the install command and
            the full API surface baked in.
          </p>
          <div className="ai-prompt-box" aria-hidden="true">
            cargo add dotmax --features image · quick::show_file · BrailleGrid · AnimationLoop · …
          </div>
          <div className="ai-card-action">
            <CopyPill value={agentPrompt} label="Copy prompt" />
          </div>
        </article>

        <article className="ai-card">
          <span className="ai-card-icon">
            <FileText size={20} />
          </span>
          <h3>llms.txt reference</h3>
          <p>
            A token-efficient Markdown index of the crate, its API, features, and examples — served at
            the site root for any agent under context pressure.
          </p>
          <div className="ai-editor-row">
            <a className="copy-pill" href={withBase("/llms.txt")} target="_blank" rel="noreferrer">
              <FileText size={16} />
              Open llms.txt
            </a>
            <CopyPill value={llmsUrl} label="Copy URL" />
          </div>
        </article>

        <article className="ai-card">
          <span className="ai-card-icon">
            <Terminal size={20} />
          </span>
          <h3>Wire up your editor</h3>
          <p>
            Add the reference as context in your AI editor, then browse the full rustdoc on docs.rs
            and the source on GitHub.
          </p>
          <div className="ai-prompt-box">{cursorSetup}</div>
          <div className="ai-editor-row">
            <a className="pill-link" href="https://docs.rs/dotmax" target="_blank" rel="noreferrer">
              docs.rs <ExternalLink size={13} />
            </a>
            <a className="pill-link" href={githubUrl} target="_blank" rel="noreferrer">
              GitHub <ExternalLink size={13} />
            </a>
          </div>
        </article>
      </div>

      <div className="ai-roadmap">
        <Bot size={18} className="shrink-0 text-terminal" />
        <p>
          <strong>On the roadmap:</strong> a dotmax MCP server and a widget registry for one-command,
          agent-driven setup. The llms.txt reference and copy-prompts above work today.
        </p>
      </div>
    </section>
  );
}
