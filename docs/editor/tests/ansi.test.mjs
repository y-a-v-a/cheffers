// Unit tests for the pure ANSI->HTML helpers. Run with `node --test` (or
// `npm test` from docs/editor). No browser required.

import { test } from "node:test";
import assert from "node:assert/strict";
import { escapeHtml, ansiToHtml } from "../ansi.js";

test("escapeHtml neutralizes markup characters", () => {
  assert.equal(escapeHtml("<b>&</b>"), "&lt;b&gt;&amp;&lt;/b&gt;");
});

test("plain text passes through unchanged", () => {
  assert.equal(ansiToHtml("just text"), "just text");
});

test("a color code wraps following text in a colored span", () => {
  // ESC[31m red ESC[0m
  const html = ansiToHtml("\x1b[31mred\x1b[0m");
  assert.equal(html, '<span style="color:#ff6b6b">red</span>');
});

test("bold + color combine into one span", () => {
  const html = ansiToHtml("\x1b[1;31mbold red\x1b[0m");
  assert.equal(
    html,
    '<span style="font-weight:600;color:#ff6b6b">bold red</span>',
  );
});

test("reset closes the open span", () => {
  const html = ansiToHtml("\x1b[34mblue\x1b[0m plain");
  assert.equal(html, '<span style="color:#6ea8ff">blue</span> plain');
});

test("an unterminated sequence still closes its span at end of input", () => {
  const html = ansiToHtml("\x1b[32mgreen");
  assert.equal(html, '<span style="color:#5ad19a">green</span>');
});

test("text inside a colored span is HTML-escaped (no injection)", () => {
  const html = ansiToHtml("\x1b[31m<script>alert(1)</script>\x1b[0m");
  assert.equal(
    html,
    '<span style="color:#ff6b6b">&lt;script&gt;alert(1)&lt;/script&gt;</span>',
  );
});

test("escaping also applies to text outside any span", () => {
  assert.equal(ansiToHtml("a & b < c"), "a &amp; b &lt; c");
});

test("unknown SGR codes are ignored but don't break parsing", () => {
  // 7 = reverse video (unsupported) — should be dropped, text preserved.
  const html = ansiToHtml("\x1b[7mx\x1b[0m");
  assert.equal(html, "x");
});

test("a realistic interpreter error renders colored, escaped HTML", () => {
  const sample =
    "\x1b[1m\x1b[31merror\x1b[0m: \x1b[1m\x1b[37minvalid title\x1b[0m\n";
  const html = ansiToHtml(sample);
  assert.match(html, /<span style="[^"]*color:#ff6b6b[^"]*">error<\/span>/);
  assert.ok(html.includes("invalid title"));
  // No raw escape bytes survive in the output.
  assert.ok(!html.includes("\x1b"));
});
