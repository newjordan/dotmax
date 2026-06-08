import { chromium } from "playwright";

const base = process.env.SHOT_URL ?? "http://127.0.0.1:5174/";
const outDir = process.env.SHOT_DIR ?? "/tmp/dotmax_shots";
const tag = process.env.SHOT_TAG ?? "baseline";

import { mkdirSync } from "node:fs";
mkdirSync(outDir, { recursive: true });

const browser = await chromium.launch();

// Desktop full page
const desktop = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
await desktop.goto(base, { waitUntil: "networkidle", timeout: 60000 });
await desktop.waitForTimeout(1200);
await desktop.screenshot({ path: `${outDir}/${tag}-desktop-full.png`, fullPage: true });
// Desktop hero only (above the fold)
await desktop.screenshot({ path: `${outDir}/${tag}-desktop-fold.png` });
await desktop.close();

// Mobile full page
const mobile = await browser.newPage({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 });
await mobile.goto(base, { waitUntil: "networkidle", timeout: 60000 });
await mobile.waitForTimeout(1200);
await mobile.screenshot({ path: `${outDir}/${tag}-mobile-full.png`, fullPage: true });
await mobile.screenshot({ path: `${outDir}/${tag}-mobile-fold.png` });
await mobile.close();

await browser.close();
console.log(`shots written to ${outDir} with tag ${tag}`);
