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

  await step("the editor highlights Chef syntax", async () => {
    // "Ingredients." and "Method." are sections (CodeMirror only renders the
    // lines in view, so "Serves 1." may be off-screen); Put, Pour are verbs.
    assert.ok((await page.locator(".cm-content .tok-section").count()) >= 2);
    assert.ok((await page.locator(".cm-content .tok-verb").count()) > 0);
    assert.ok((await page.locator(".cm-content .tok-bowl").count()) > 0);
  });

  await step("switching to the Countdown example outputs '54321'", async () => {
    await page.selectOption("#examples", "countdown-cake");
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "54321",
      { timeout: 10000 },
    );
  });

  await step("the Doubler example reads its prefilled input and outputs '42'", async () => {
    await page.selectOption("#examples", "doubler-delight");
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "42",
      { timeout: 10000 },
    );
    const stdin = await page.inputValue("#stdin");
    assert.equal(stdin, "21", "selecting the example should prefill its input");
  });

  await step("editing the input re-runs the recipe", async () => {
    await page.fill("#stdin", "100");
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "200",
      { timeout: 10000 },
    );
  });

  await step("clearing the input surfaces the cannot-read-input error", async () => {
    await page.fill("#stdin", "");
    await page.waitForFunction(
      () => {
        const o = document.getElementById("output");
        return o?.classList.contains("error") && o.textContent.includes("cannot read input");
      },
      { timeout: 10000 },
    );
  });

  await step("the Wrap toggle switches CodeMirror line wrapping", async () => {
    // Wrapping is on by default (Chef methods are one long line).
    const content = page.locator(".cm-content");
    assert.match(await content.getAttribute("class"), /cm-lineWrapping/);

    await page.uncheck("#wrap");
    await page.waitForFunction(
      () => !document.querySelector(".cm-content")?.classList.contains("cm-lineWrapping"),
      { timeout: 5000 },
    );

    await page.check("#wrap");
    await page.waitForFunction(
      () => document.querySelector(".cm-content")?.classList.contains("cm-lineWrapping"),
      { timeout: 5000 },
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
    assert.match(html, /<span class="[^"]*ansi-/, "expected a colored span in the error output");
    assert.ok(!html.includes("\x1b"), "raw ANSI escape leaked into the DOM");
    const status = await page.locator("#status").textContent();
    assert.equal(status.trim(), "error");
  });

  await step("⌘/Ctrl+Enter runs the recipe when auto-run is off", async () => {
    await page.uncheck("#autorun");
    await page.locator(".cm-content").click();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.type(
      "Seven Soup.\n\nIngredients.\n7 g salt\n\nMethod.\n" +
        "Put salt into the mixing bowl. Pour contents of the mixing bowl into the baking dish.\n\n" +
        "Serves 1.",
    );
    // Auto-run is off: the stale error stays put...
    await page.waitForTimeout(700);
    assert.ok(
      (await page.locator("#output").textContent()).includes("invalid title"),
      "recipe ran without auto-run or the shortcut",
    );
    // ...until the shortcut runs it (and must not insert a blank line).
    const before = await page.locator(".cm-content").textContent();
    await page.keyboard.press("ControlOrMeta+Enter");
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "7",
      { timeout: 10000 },
    );
    assert.equal(await page.locator(".cm-content").textContent(), before);
    await page.check("#autorun");
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

  /* ----- Cookbook integration ----- */

  const cookbookUrl = new URL("../cookbook/", BASE_URL).href;

  await step("the playground header links to the cookbook", async () => {
    assert.equal(await page.locator('.site-nav a[href="../cookbook/"]').count(), 1);
  });

  await step("the cookbook menu lists seven courses plus the reference", async () => {
    await page.goto(cookbookUrl, { waitUntil: "networkidle" });
    assert.equal(await page.locator(".menu-grid .menu-card").count(), 8);
  });

  await step("cookbook recipe cards are highlighted without changing their source", async () => {
    await page.goto(new URL("01-your-first-dish.html", cookbookUrl).href, {
      waitUntil: "networkidle",
    });
    const code = page.locator(".recipe-card pre code").first();
    assert.ok((await code.locator(".tok-verb").count()) > 0, "no highlighted verbs");
    assert.ok(
      (await code.textContent()).includes("Put mystery beans into the mixing bowl."),
      "highlighting altered the recipe text",
    );
  });

  await step("the cookbook picks up the theme saved in the playground", async () => {
    // The earlier theme step left "cast-iron" in localStorage; the shared
    // key must carry it across to the cookbook pages.
    const theme = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );
    assert.equal(theme, "cast-iron");
  });

  await step("open-in-playground loads the recipe, its input, and runs it", async () => {
    await page.goto(new URL("04-raiding-the-refrigerator.html", cookbookUrl).href, {
      waitUntil: "networkidle",
    });
    // First card on the page is Porridge Weather Report (data-input="20").
    const link = page.locator(".recipe-card .open-playground").first();
    assert.match(await link.getAttribute("href"), /#recipe=/);
    await link.click();
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "68",
      { timeout: 15000 },
    );
    assert.equal(await page.inputValue("#stdin"), "20");
    const doc = await page.locator(".cm-content").textContent();
    assert.ok(doc.includes("Porridge Weather Report."), "recipe source not loaded");
    const selected = await page.$eval("#examples", (el) => el.selectedOptions[0]?.textContent);
    assert.equal(selected, "From the cookbook");
  });

  /* ----- Landing page ----- */

  await step("the landing page highlights its recipe and runs it in the playground", async () => {
    await page.goto(new URL("../", BASE_URL).href, { waitUntil: "networkidle" });
    assert.ok((await page.locator(".preview code .tok-verb").count()) > 0, "hero recipe not highlighted");
    assert.equal(await page.locator('.site-nav a[href="./editor/"]').count(), 1);
    assert.equal(await page.locator('.site-nav a[href="./cookbook/"]').count(), 1);
    await page.locator('.preview a[href*="#recipe="]').click();
    await page.waitForFunction(
      () => document.getElementById("output")?.textContent.trim() === "42",
      { timeout: 15000 },
    );
    const doc = await page.locator(".cm-content").textContent();
    assert.ok(doc.includes("The Answer Broth."), "hero recipe not loaded");
  });

  await step("every cookbook page serves without console errors", async () => {
    const pages = [
      "index.html",
      "01-your-first-dish.html",
      "02-cooking-with-letters.html",
      "03-kitchen-math.html",
      "04-raiding-the-refrigerator.html",
      "05-stir-until-done.html",
      "06-the-great-bowl-shuffle.html",
      "07-call-in-the-sous-chef.html",
      "reference.html",
    ];
    for (const p of pages) {
      const before = consoleErrors.length;
      await page.goto(new URL(p, cookbookUrl).href, { waitUntil: "networkidle" });
      assert.equal(
        consoleErrors.length,
        before,
        `console errors on ${p}: ${consoleErrors.slice(before).join("; ")}`,
      );
    }
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
