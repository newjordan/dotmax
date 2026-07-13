export type StyleKind = "bar" | "spinner" | "border" | "wipe" | "meter";

export type StyleMeta = {
  id: string;
  theme: string;
  name: string;
  kind: StyleKind;
  description: string;
};

export type ThemeMeta = {
  name: string;
  count: number;
  kind: StyleKind;
};

export type CatalogIndex = {
  version: number;
  total: number;
  width: number;
  height: number;
  fps: number;
  frames_per_style: number;
  themes: ThemeMeta[];
  styles: StyleMeta[];
};

export type CatalogFrame = {
  /** Text rows — one string of `width` chars per cell row. */
  t: string[];
  /** Color rows — 2 hex chars per cell indexing `palette`, ".." = none. */
  c?: string[];
};

export type StylePack = {
  id: string;
  name: string;
  description: string;
  palette: string[];
  frames: CatalogFrame[];
  source: string;
};

export type ThemePack = {
  theme: string;
  width: number;
  height: number;
  fps: number;
  frames_per_style: number;
  styles: StylePack[];
};
