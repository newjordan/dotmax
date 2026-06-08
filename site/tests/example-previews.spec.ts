import { expect, test } from "@playwright/test";

test("example cards render terminal previews without horizontal overflow", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.locator("#examples").scrollIntoViewIfNeeded();

  const cards = page.locator(".example-card");
  const previews = page.locator(".example-terminal");
  const previewImages = page.locator(".example-terminal-image");
  const miniTerminals = page.locator(".mini-terminal");
  const collapsedCode = page.locator(".example-code-details:not([open])");
  const categoryThumbnails = page.locator(".category-menu-thumb img");

  await expect(cards).toHaveCount(15);
  await expect(previews).toHaveCount(15);
  await expect(previewImages).toHaveCount(8);
  await expect(miniTerminals).toHaveCount(7);
  await expect(collapsedCode).toHaveCount(15);
  await expect(categoryThumbnails).toHaveCount(4);

  const pageWidth = await page.evaluate(() => document.documentElement.clientWidth);
  const pageOverflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(pageOverflow).toBeLessThanOrEqual(1);

  const previewBoxes = await previews.evaluateAll((nodes) =>
    nodes.map((node) => {
      const rect = node.getBoundingClientRect();
      return {
        left: rect.left,
        right: rect.right,
        height: rect.height,
      };
    }),
  );

  for (const box of previewBoxes) {
    expect(box.left).toBeGreaterThanOrEqual(0);
    expect(box.right).toBeLessThanOrEqual(pageWidth + 1);
    expect(box.height).toBeGreaterThan(220);
    expect(box.height).toBeLessThan(430);
  }

  const previewImageFits = await page.locator(".example-terminal-image, .filter-pill img").evaluateAll((nodes) =>
    nodes.every((node) => getComputedStyle(node).objectFit === "contain"),
  );
  expect(previewImageFits).toBe(true);

  await cards.first().scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`examples-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("mini terminals load frame packs and advance visible animations", async ({ page }) => {
  await page.goto("/");
  await page.locator("#examples").scrollIntoViewIfNeeded();

  const terminals = page.locator(".mini-terminal");
  await expect(terminals).toHaveCount(7);

  for (let index = 0; index < 7; index += 1) {
    await terminals.nth(index).scrollIntoViewIfNeeded();
  }

  await expect(page.locator(".mini-terminal[data-loaded='true']")).toHaveCount(7);

  const nonEmptyCount = await terminals.evaluateAll((nodes) =>
    nodes.filter((node) => (node.textContent ?? "").replace(/\s/g, "").length > 0).length,
  );
  expect(nonEmptyCount).toBeGreaterThanOrEqual(6);

  const first = terminals.first();
  await first.scrollIntoViewIfNeeded();
  const initialFrame = await first.getAttribute("data-frame-index");
  await page.waitForTimeout(450);
  const nextFrame = await first.getAttribute("data-frame-index");
  expect(nextFrame).not.toBe(initialFrame);
});

test("reduced motion shows static mini terminal frames", async ({ browser }) => {
  const context = await browser.newContext({ reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto("/");
  await page.locator("#examples").scrollIntoViewIfNeeded();

  const first = page.locator(".mini-terminal").first();
  await first.scrollIntoViewIfNeeded();
  await expect(first).toHaveAttribute("data-loaded", "true");
  await expect(first).toHaveAttribute("data-frame-index", "0");
  await page.waitForTimeout(450);
  await expect(first).toHaveAttribute("data-frame-index", "0");

  await context.close();
});

test("copy buttons expose copied feedback", async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (value: string) => {
          window.localStorage.setItem("copied-value", value);
        },
      },
    });
  });

  await page.goto("/");
  await page.locator("#patterns").scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Open the pattern workbench" }).click();

  const firstCopy = page.locator("#patterns .copy-button").first();
  await expect(firstCopy).toHaveAttribute("data-copy-state", "idle");
  await firstCopy.click();
  await expect(firstCopy).toHaveAttribute("data-copy-state", "copied");
  await expect(firstCopy.locator(".copy-button-status")).toContainText("Copied");
  await expect(firstCopy).toHaveAccessibleName("Copied");
});

test("loading bar catalog defaults to a curated set and expands to all 586", async ({ page }) => {
  await page.goto("/");
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();

  const grid = page.locator(".loading-bars-grid");
  await expect(grid).toHaveAttribute("data-loading-bar-count", "586");
  // The catalog is gated: a curated 12 render by default to keep the page scannable.
  await expect(page.locator(".loading-bar-card")).toHaveCount(12);
  await expect(page.locator(".loading-theme-pill")).toHaveCount(53);

  // Expander reveals the full catalog on demand.
  await page.getByRole("button", { name: /Browse all 586 styles/ }).click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(586);

  const catalogTiming = await page.evaluate(async () => {
    const response = await fetch("/examples/loading_bar_catalog.json");
    const catalog = await response.json();
    return {
      fps: catalog.fps,
      framesPerStyle: catalog.frames_per_style,
      firstProgress: catalog.styles[0].frames[0].join("\n"),
      quarterProgress: catalog.styles[0].frames[Math.floor(catalog.frames_per_style / 4)].join("\n"),
      middleProgress: catalog.styles[0].frames[Math.floor(catalog.frames_per_style / 2)].join("\n"),
      threeQuarterProgress: catalog.styles[0].frames[Math.floor((catalog.frames_per_style * 3) / 4)].join("\n"),
      lastProgress: catalog.styles[0].frames[catalog.frames_per_style - 1].join("\n"),
    };
  });
  expect(catalogTiming.fps).toBe(4);
  expect(catalogTiming.framesPerStyle).toBe(32);
  expect(catalogTiming.firstProgress).not.toBe(catalogTiming.middleProgress);
  expect(catalogTiming.quarterProgress).toBe(catalogTiming.threeQuarterProgress);
  expect(catalogTiming.lastProgress).not.toBe(catalogTiming.middleProgress);

  const firstCard = page.locator(".loading-bar-card").first();
  await expect(firstCard.locator(".loading-bar-output")).not.toBeEmpty();
  const initialFrame = await grid.getAttribute("data-loading-bar-frame");
  await page.waitForTimeout(350);
  await expect(grid).not.toHaveAttribute("data-loading-bar-frame", initialFrame ?? "");

  await page.getByPlaceholder("Search wildlife, quantum, sinewave, spinner...").fill("quantum");
  const filteredCount = await page.locator(".loading-bar-card").count();
  expect(filteredCount).toBeGreaterThan(0);
  expect(filteredCount).toBeLessThan(586);
});

test("TUI pattern workbench exposes tables folders tabs and schematics", async ({ page }) => {
  await page.goto("/");
  await page.locator("#patterns").scrollIntoViewIfNeeded();

  // The workbench is collapsed by default; the resource index stays visible.
  await expect(page.locator(".pattern-resource-card")).toHaveCount(8);
  await expect(page.locator(".pattern-resource-card").first()).toContainText("Columns, filters");
  await page.getByRole("button", { name: "Open the pattern workbench" }).click();

  await expect(page.locator(".pattern-tab")).toHaveCount(8);
  await page.getByRole("button", { name: "Open Schematic resources" }).click();
  await expect(page.locator(".schematic-recipe-row")).toHaveCount(5);
  await page.getByRole("button", { name: "Open Research table resources" }).click();
  await expect(page.locator(".resource-table tbody tr")).toHaveCount(8);
  await expect(page.locator(".table-recipe-row")).toHaveCount(5);
  await expect(page.locator(".table-recipe-detail")).toContainText("Column schema");
  await page.getByRole("button", { name: /faceted-filter-bar/ }).click();
  await expect(page.locator(".table-recipe-code")).toContainText("struct FilterState");
  await expect(page.locator(".table-recipe-preview")).toContainText("visible  42 / 586 rows");
  await page.getByPlaceholder("Search table, folders, schematic, inspector...").fill("folder");
  await expect(page.locator(".resource-table tbody tr")).toHaveCount(1);

  await page.getByRole("button", { name: "Folders", exact: true }).click();
  await expect(page.locator(".folder-node")).toHaveCount(5);
  await page.getByRole("button", { name: /src\/views\/schematic.rs/ }).click();
  await expect(page.locator(".folder-detail-panel")).toContainText("stable node ids");
  await expect(page.locator(".folder-browser-row")).toHaveCount(4);
  await expect(page.locator(".folder-browser-detail")).toContainText("Arena tree state");
  await page.getByRole("button", { name: /preview-cache/ }).click();
  await expect(page.locator(".folder-browser-code")).toContainText("enum PreviewModel");
  await expect(page.locator(".folder-browser-preview")).toContainText("dotmax thumb");

  await page.getByRole("button", { name: "Schematic", exact: true }).click();
  await expect(page.locator(".schematic-node")).toHaveCount(6);
  await expect(page.locator(".schematic-notes")).toContainText("dotmax owns dense visual previews");
  await expect(page.locator(".schematic-recipe-row")).toHaveCount(5);
  await expect(page.locator(".schematic-recipe-detail")).toContainText("Stable node model");
  await page.getByRole("button", { name: /health-propagation/ }).click();
  await expect(page.locator(".schematic-recipe-code")).toContainText("fn derived_health");
  await expect(page.locator(".schematic-recipe-preview")).toContainText("export blocked");

  await page.getByRole("button", { name: "Tabs", exact: true }).click();
  await expect(page.locator(".tab-recipe-row")).toHaveCount(4);
  await expect(page.locator(".tab-strip-preview span")).toHaveCount(5);
  await expect(page.locator(".tab-recipe-detail")).toContainText("Typed view router");
  await page.getByRole("button", { name: /preview-backed-tabs/ }).click();
  await expect(page.locator(".tab-recipe-code")).toContainText("struct TabSummary");

  await page.getByRole("button", { name: "Blueprints", exact: true }).click();
  await expect(page.locator(".blueprint-card")).toHaveCount(5);
  await expect(page.locator(".blueprint-card").first()).toContainText("struct TableState");
  await expect(page.locator(".blueprint-card").first()).toContainText("rows: Vec<RowModel>");

  await page.getByRole("button", { name: "Resource kits", exact: true }).click();
  await expect(page.locator(".kit-card")).toHaveCount(4);
  await expect(page.locator(".kit-card").first()).toContainText("Research dashboard");
  await expect(page.locator(".kit-card").first()).toContainText("cargo add ratatui crossterm");
  await expect(page.locator(".kit-card").nth(2)).toContainText("petgraph");

  await page.getByRole("button", { name: "Contracts", exact: true }).click();
  await expect(page.locator(".contract-row")).toHaveCount(6);
  await expect(page.locator(".contract-detail")).toContainText("table.move-selection");
  await page.getByRole("button", { name: /schematic.focus-node/ }).click();
  await expect(page.locator(".contract-detail")).toContainText("focus only lands on visible nodes");
  await expect(page.locator(".contract-snippet")).toContainText("Command::FocusNode");

  await page.getByRole("button", { name: "Layouts", exact: true }).click();
  await expect(page.locator(".layout-card")).toHaveCount(4);
  await expect(page.locator(".layout-wireframe span")).toHaveCount(15);
  await expect(page.locator(".layout-card").first()).toContainText("Table + inspector shell");
  await expect(page.locator(".layout-code").first()).toContainText("Layout::vertical");
});

test("supporting cards stay visual instead of text-only", async ({ page }, testInfo) => {
  await page.goto("/");

  await expect(page.locator(".feature-card-visual img")).toHaveCount(6);
  await expect(page.locator(".doc-card-visual img")).toHaveCount(5);
  await expect(page.locator(".open-source-visual img")).toHaveCount(1);

  const containedImages = await page.locator(".gallery-item img, .feature-card-visual img, .doc-card-visual img, .open-source-visual img").evaluateAll((nodes) =>
    nodes.every((node) => getComputedStyle(node).objectFit === "contain"),
  );
  expect(containedImages).toBe(true);

  await page.locator(".feature-card").first().scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`supporting-cards-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("install docs and open-source sections are visually led", async ({ page }, testInfo) => {
  await page.goto("/");

  await page.locator("#install").scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`install-${testInfo.project.name}.png`),
    fullPage: false,
  });

  await page.locator("#docs").scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`docs-open-source-${testInfo.project.name}.png`),
    fullPage: false,
  });

  await page.locator(".open-source-band").scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`open-source-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("command palette opens with Cmd/Ctrl+K and searches the catalog", async ({ page }) => {
  await page.goto("/");

  // Trigger via the nav button.
  await page.getByRole("button", { name: "Open command menu" }).click();
  const dialog = page.getByRole("dialog", { name: "Command menu" });
  await expect(dialog).toBeVisible();

  // Static quick actions show before typing.
  await expect(page.locator(".cmdk-item")).not.toHaveCount(0);

  // Fuzzy search reaches into the 586-style loading-bar catalog.
  await page.getByPlaceholder("Search examples, loading bars, docs…").fill("gradient");
  await expect(page.locator(".cmdk-group-label", { hasText: "Loading bars" })).toBeVisible();

  // Escape closes it.
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);

  // Keyboard shortcut re-opens it.
  await page.keyboard.press("ControlOrMeta+k");
  await expect(page.getByRole("dialog", { name: "Command menu" })).toBeVisible();
});

test("build-with-ai section exposes agent prompt and llms.txt", async ({ page }) => {
  await page.goto("/");
  await page.locator("#build-with-ai").scrollIntoViewIfNeeded();

  await expect(page.getByRole("heading", { name: "Built to be built by agents." })).toBeVisible();
  await expect(page.getByRole("button", { name: "Copy prompt" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Open llms.txt" })).toHaveAttribute("href", "/llms.txt");
  // Roadmap items are labeled honestly, not claimed as shipped.
  await expect(page.locator(".ai-roadmap")).toContainText("roadmap");
});

test("llms.txt is served at the site root", async ({ page }) => {
  const response = await page.request.get("/llms.txt");
  expect(response.ok()).toBe(true);
  const body = await response.text();
  expect(body).toContain("# dotmax");
  expect(body).toContain("cargo add dotmax");
  expect(body).toContain("For AI agents");
});
