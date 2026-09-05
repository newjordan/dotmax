import { useEffect } from "react";

/**
 * Scroll-reveal for every `[data-reveal]` element: once it enters the
 * viewport it gets `data-reveal="in"` and the CSS transition runs. One shared
 * observer, each element unobserved after its first reveal. Reduced-motion
 * users see everything immediately (the CSS disables the transition too).
 */
export function useReveal() {
  useEffect(() => {
    if (typeof IntersectionObserver === "undefined") {
      document.querySelectorAll("[data-reveal]").forEach((el) => el.setAttribute("data-reveal", "in"));
      return undefined;
    }
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.setAttribute("data-reveal", "in");
          observer.unobserve(entry.target);
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.08 },
    );
    const attach = () => {
      document.querySelectorAll("[data-reveal]:not([data-reveal='in'])").forEach((el) => {
        if (reduced) el.setAttribute("data-reveal", "in");
        else observer.observe(el);
      });
    };
    attach();
    // Sections mount their content lazily (catalog fetch), so re-scan on DOM growth.
    const mutation = new MutationObserver(() => attach());
    mutation.observe(document.body, { childList: true, subtree: true });
    return () => {
      observer.disconnect();
      mutation.disconnect();
    };
  }, []);
}
