import { chromium } from "playwright";
import { mkdirSync } from "node:fs";

const base = process.env.SHOT_URL ?? "http://127.0.0.1:5174/";
const outDir = process.env.SHOT_DIR ?? "/tmp/dotmax_shots";
const tag = process.env.SHOT_TAG ?? "baseline";
mkdirSync(outDir, { recursive: true });

const anchors = ["#install", "#examples", "#loading-bars", "#patterns", "#gallery", "#docs"];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
await page.goto(base, { waitUntil: "networkidle", timeout: 60000 });
await page.waitForTimeout(1000);

for (const a of anchors) {
  await page.evaluate((sel) => {
    const el = document.querySelector(sel);
    if (el) el.scrollIntoView({ behavior: "instant", block: "start" });
  }, a);
  await page.waitForTimeout(700);
  const name = a.replace("#", "");
  await page.screenshot({ path: `${outDir}/${tag}-sec-${name}.png` });
}
await browser.close();
console.log("section shots done");
