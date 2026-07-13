/**
 * Prefix a root-relative asset path with Vite's configured base URL, so the
 * site works both at the domain root (dev, preview, e2e) and under a subpath
 * (GitHub Pages serves this site from /dotmax/). BASE_URL always ends in "/".
 */
export function withBase(path: string): string {
  return import.meta.env.BASE_URL + path.replace(/^\//, "");
}
