// End-to-end browser test for the playground. Loads the page in headless
// Chromium and exercises the full stack: CodeMirror, the wasm interpreter,
// auto-run, example switching, and rich (ANSI->HTML) error rendering.
//
// Expects a server already serving the docs/ directory. The base URL of the
// editor is taken from the BASE_URL env var (default http://localhost:8123/editor/).
// scripts/test-browser.sh wires up the server and runs this.
//
// Exits non-zero on the first failed assertion.

import { chromium } from "playwright";
import assert from "node:assert/strict";

const BASE_URL = process.env.BASE_URL || "http://localhost:8123/editor/";

let browser;
let failed = false;

async function step(name, fn) {
  try {
    await fn();
    console.log(`ok - ${name}`);
  } catch (err) {
    failed = true;
    console.error(`not ok - ${name}\n    ${err.message}`);
  }
}

const browserArgs = ["--no-sandbox", "--disable-dev-shm-usage"];

try {
  browser = await chromium.launch({ args: browserArgs });
  const page = await browser.newPage();
  const consoleErrors = [];
  page.on("console", (m) => {
    if (m.type() === "error") consoleErrors.push(m.text());
  });
  page.on("pageerror", (e) => consoleErrors.push("pageerror: " + e.message));

  await page.goto(BASE_URL, { waitUntil: "networkidle" });

  await step("default recipe auto-runs to 'Hello world!'", async () => {
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.includes("Hello world!"),
      { timeout: 15000 },
    );
  });

  await step("status shows 'ok' after a successful run", async () => {
    const status = await page.locator("#status").textContent();
    assert.equal(status.trim(), "ok");
  });

  await step("switching to the Countdown example outputs '54321'", async () => {
    await page.selectOption("#examples", "countdown-cake");
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "54321",
      { timeout: 10000 },
    );
  });

  await step("an invalid recipe renders a colored, escaped error", async () => {
    await page.locator(".cm-content").click();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.type("Totally not a recipe");
    await page.waitForFunction(
      () => {
        const o = document.getElementById("output");
        return o?.classList.contains("error") && o.textContent.includes("invalid title");
      },
      { timeout: 10000 },
    );
    const html = await page.locator("#output").innerHTML();
    assert.match(html, /<span style="[^"]*color:/, "expected a colored span in the error output");
    assert.ok(!html.includes("\x1b"), "raw ANSI escape leaked into the DOM");
    const status = await page.locator("#status").textContent();
    assert.equal(status.trim(), "error");
  });

  await step("theme toggle cycles to Cast Iron and darkens the gutter", async () => {
    const getTheme = () =>
      page.evaluate(() => document.documentElement.getAttribute("data-theme") || "system");
    // Cycle (system -> parchment -> cast-iron -> espresso -> ...) until Cast Iron.
    for (let i = 0; i < 4 && (await getTheme()) !== "cast-iron"; i++) {
      await page.click("#theme");
      await page.waitForTimeout(50);
    }
    assert.equal(await getTheme(), "cast-iron", "expected Cast Iron after cycling");

    // The original bug: the gutter kept a light background in dark mode.
    const gutterBg = await page.$eval(
      ".cm-gutters",
      (el) => getComputedStyle(el).backgroundColor,
    );
    const [r, g, b] = gutterBg.match(/\d+/g).map(Number);
    assert.ok(r < 80 && g < 80 && b < 80, `gutter background not dark: ${gutterBg}`);

    // Choice is persisted.
    const saved = await page.evaluate(() => localStorage.getItem("cheffers-theme"));
    assert.equal(saved, "cast-iron");
  });

  await step("no uncaught console/page errors occurred", async () => {
    assert.deepEqual(consoleErrors, [], `console errors: ${consoleErrors.join("; ")}`);
  });
} catch (err) {
  failed = true;
  console.error("not ok - harness error\n    " + err.stack);
} finally {
  if (browser) await browser.close();
}

if (failed) {
  console.error("\nBrowser test FAILED");
  process.exit(1);
}
console.log("\nAll browser checks passed");
