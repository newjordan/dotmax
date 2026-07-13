import { useEffect, useState } from "react";
import { withBase } from "../lib/withBase";
import type { CatalogIndex, ThemePack } from "./types";

const themeCache = new Map<string, Promise<ThemePack>>();
let indexPromise: Promise<CatalogIndex> | null = null;
const standaloneCache = new Map<string, Promise<string>>();

export function loadCatalogIndex(): Promise<CatalogIndex> {
  indexPromise ??= fetch(withBase("/catalog/index.json")).then((response) => {
    if (!response.ok) {
      indexPromise = null;
      throw new Error("Failed to load catalog index");
    }
    return response.json() as Promise<CatalogIndex>;
  });
  return indexPromise;
}

export function loadThemePack(theme: string): Promise<ThemePack> {
  let promise = themeCache.get(theme);
  if (!promise) {
    promise = fetch(withBase(`/catalog/themes/${theme}.json`)).then((response) => {
      if (!response.ok) {
        themeCache.delete(theme);
        throw new Error(`Failed to load theme pack: ${theme}`);
      }
      return response.json() as Promise<ThemePack>;
    });
    themeCache.set(theme, promise);
  }
  return promise;
}

/** The full standalone `.rs` program for a theme, fetched once. */
export function loadStandalone(theme: string): Promise<string> {
  let promise = standaloneCache.get(theme);
  if (!promise) {
    promise = fetch(withBase(`/catalog/assets/${theme}.rs`)).then((response) => {
      if (!response.ok) {
        standaloneCache.delete(theme);
        throw new Error(`Failed to load standalone file: ${theme}`);
      }
      return response.text();
    });
    standaloneCache.set(theme, promise);
  }
  return promise;
}

let heroPromise: Promise<ThemePack> | null = null;

/** The curated hero pack for the landing showcase, fetched once. */
export function loadHeroPack(): Promise<ThemePack> {
  heroPromise ??= fetch(withBase("/catalog/hero.json")).then((response) => {
    if (!response.ok) {
      heroPromise = null;
      throw new Error("Failed to load hero pack");
    }
    return response.json() as Promise<ThemePack>;
  });
  return heroPromise;
}

export function useHeroPack(): ThemePack | null {
  const [pack, setPack] = useState<ThemePack | null>(null);
  useEffect(() => {
    let live = true;
    loadHeroPack()
      .then((data) => {
        if (live) setPack(data);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, []);
  return pack;
}

export function useCatalogIndex(): CatalogIndex | null {
  const [index, setIndex] = useState<CatalogIndex | null>(null);
  useEffect(() => {
    let live = true;
    loadCatalogIndex()
      .then((data) => {
        if (live) setIndex(data);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, []);
  return index;
}

export function useThemePack(theme: string | null): ThemePack | null {
  const [pack, setPack] = useState<ThemePack | null>(null);
  useEffect(() => {
    if (!theme) {
      setPack(null);
      return undefined;
    }
    let live = true;
    loadThemePack(theme)
      .then((data) => {
        if (live) setPack(data);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [theme]);
  return pack;
}
