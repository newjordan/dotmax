import { useEffect, useRef, useState } from "react";

/** Shared IntersectionObserver — one instance for all catalog cards. */

const callbacks = new WeakMap<Element, (visible: boolean) => void>();
let observer: IntersectionObserver | null = null;

function ensureObserver(): IntersectionObserver | null {
  if (typeof IntersectionObserver === "undefined") return null;
  observer ??= new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        callbacks.get(entry.target)?.(entry.isIntersecting);
      }
    },
    { rootMargin: "320px 0px" },
  );
  return observer;
}

export function useInView<T extends Element>(): [React.RefObject<T>, boolean] {
  const ref = useRef<T>(null);
  const [inView, setInView] = useState(false);
  useEffect(() => {
    const element = ref.current;
    if (!element) return undefined;
    const shared = ensureObserver();
    if (!shared) {
      setInView(true);
      return undefined;
    }
    callbacks.set(element, setInView);
    shared.observe(element);
    return () => {
      callbacks.delete(element);
      shared.unobserve(element);
    };
  }, []);
  return [ref, inView];
}
