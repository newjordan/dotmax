import { CopyButton } from "./CopyButton";

export function CodeBlock({ code, label }: { code: string; label: string }) {
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
