import { Check, CornerDownLeft, Search, type LucideIcon } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

export type CommandAction =
  | { type: "scroll"; target: string }
  | { type: "link"; href: string }
  | { type: "copy"; value: string };

export type CommandItem = {
  id: string;
  group: string;
  label: string;
  hint?: string;
  icon: LucideIcon;
  keywords?: string;
  action: CommandAction;
};

type LoadingBarStyle = { id: string; theme: string; name: string; description: string; command: string };

export function CommandPalette({
  open,
  onClose,
  items,
}: {
  open: boolean;
  onClose: () => void;
  items: CommandItem[];
}) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [barItems, setBarItems] = useState<CommandItem[]>([]);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);

  // Lazily pull the loading-bar catalog so all 586 styles stay findable here.
  useEffect(() => {
    if (!open || barItems.length > 0) return;
    let cancelled = false;
    fetch("/examples/loading_bar_catalog.json")
      .then((r) => (r.ok ? r.json() : Promise.reject()))
      .then((data: { styles: LoadingBarStyle[] }) => {
        if (cancelled) return;
        setBarItems(
          data.styles.map((s) => ({
            id: `bar-${s.id}`,
            group: "Loading bars",
            label: s.name,
            hint: s.theme,
            icon: Search,
            keywords: `${s.theme} ${s.description}`,
            action: { type: "scroll", target: "#loading-bars" } as CommandAction,
          })),
        );
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [open, barItems.length]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActiveIndex(0);
      const id = window.setTimeout(() => inputRef.current?.focus(), 20);
      return () => window.clearTimeout(id);
    }
    return undefined;
  }, [open]);

  const all = useMemo(() => [...items, ...barItems], [items, barItems]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const matched =
      q.length === 0
        ? items // when empty, show only the curated static items (not 586 bars)
        : all.filter((item) => `${item.label} ${item.group} ${item.hint ?? ""} ${item.keywords ?? ""}`.toLowerCase().includes(q));
    return matched.slice(0, q.length === 0 ? matched.length : 40);
  }, [query, all, items]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  const groups = useMemo(() => {
    const map = new Map<string, CommandItem[]>();
    for (const item of filtered) {
      const list = map.get(item.group) ?? [];
      list.push(item);
      map.set(item.group, list);
    }
    return Array.from(map.entries());
  }, [filtered]);

  function runItem(item: CommandItem) {
    const action = item.action;
    if (action.type === "copy") {
      navigator.clipboard?.writeText(action.value).catch(() => undefined);
      setCopiedId(item.id);
      window.setTimeout(() => setCopiedId(null), 1200);
      return;
    }
    if (action.type === "link") {
      window.open(action.href, "_blank", "noopener,noreferrer");
      onClose();
      return;
    }
    // scroll
    onClose();
    window.setTimeout(() => {
      const el = document.querySelector(action.target);
      if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
    }, 50);
  }

  useEffect(() => {
    if (!open) return undefined;
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      } else if (event.key === "Tab") {
        // Keep focus inside the dialog; navigation is via arrow keys.
        event.preventDefault();
        inputRef.current?.focus();
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex((i) => Math.max(i - 1, 0));
      } else if (event.key === "Enter") {
        event.preventDefault();
        const item = filtered[activeIndex];
        if (item) runItem(item);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, filtered, activeIndex]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const node = listRef.current?.querySelector("[data-active='true']");
    if (node) node.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (!open) return null;

  let runningIndex = -1;

  return (
    <div className="cmdk-overlay" role="dialog" aria-modal="true" aria-label="Command menu" onMouseDown={onClose}>
      <div className="cmdk-panel" onMouseDown={(e) => e.stopPropagation()}>
        <div className="cmdk-input-row">
          <Search size={18} className="shrink-0 text-subtle" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search examples, loading bars, docs…"
            aria-label="Search"
          />
          <kbd className="cmdk-esc">Esc</kbd>
        </div>
        <div className="cmdk-list" ref={listRef}>
          {filtered.length === 0 ? (
            <div className="cmdk-empty">No matches — try "gif", "gradient", or "webcam".</div>
          ) : (
            groups.map(([group, groupItems]) => (
              <div key={group}>
                <div className="cmdk-group-label">{group}</div>
                {groupItems.map((item) => {
                  runningIndex += 1;
                  const index = runningIndex;
                  const isActive = index === activeIndex;
                  const Icon = item.icon;
                  const isCopied = copiedId === item.id;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      data-active={isActive}
                      className={isActive ? "cmdk-item cmdk-item-active" : "cmdk-item"}
                      onMouseMove={() => setActiveIndex(index)}
                      onClick={() => runItem(item)}
                    >
                      {isCopied ? <Check size={16} className="text-terminal" /> : <Icon size={16} />}
                      <span className="cmdk-item-title">{isCopied ? "Copied" : item.label}</span>
                      {item.hint && <span className="cmdk-item-tag">{item.hint}</span>}
                      {isActive && item.action.type === "scroll" && <CornerDownLeft size={14} className="text-subtle" />}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
