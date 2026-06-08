import { Check, Clipboard } from "lucide-react";
import { useEffect, useRef, useState } from "react";

export function CopyButton({ value }: { value: string }) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">("idle");
  const resetTimerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (resetTimerRef.current !== null) window.clearTimeout(resetTimerRef.current);
    };
  }, []);

  async function copy() {
    if (resetTimerRef.current !== null) window.clearTimeout(resetTimerRef.current);

    try {
      await navigator.clipboard.writeText(value);
      setCopyState("copied");
    } catch {
      setCopyState("error");
    }

    resetTimerRef.current = window.setTimeout(() => setCopyState("idle"), 1600);
  }

  const copied = copyState === "copied";
  const failed = copyState === "error";

  return (
    <button
      className={copied ? "icon-button copy-button copy-button-copied" : failed ? "icon-button copy-button copy-button-error" : "icon-button copy-button"}
      data-copy-state={copyState}
      onClick={copy}
      type="button"
      aria-label={copied ? "Copied" : failed ? "Copy failed" : "Copy"}
    >
      {copied ? <Check size={16} /> : <Clipboard size={16} />}
      <span className="copy-button-status" aria-live="polite">
        {copied ? "Copied" : failed ? "Copy failed" : "Copy"}
      </span>
    </button>
  );
}
