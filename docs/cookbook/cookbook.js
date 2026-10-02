// The Cheffers Cookbook — small enhancements shared by every tutorial page:
//
//   1. Chef syntax highlighting on recipe cards and snippets, using the same
//      tokenizer as the playground editor (../assets/chef-syntax.js).
//   2. "Copy" buttons on recipe cards.
//   3. "Open in playground" links: the recipe source (and Input panel text,
//      if any) is packed into the URL fragment as base64url JSON, which the
//      playground unpacks on load (see docs/editor/editor.js).
//
// The theme toggle is shared with the playground (../assets/theme.js).
// No build step: this file is served as-is, as an ES module.

import { tokenizeChef } from "../assets/chef-syntax.js";

/* ----- Syntax highlighting ----- */

// Rebuild a <pre>/<code> element's text as plain text nodes plus classed
// spans. textContent is unchanged, so copying still yields the exact source.
function highlight(el) {
  const text = el.textContent;
  const fragment = document.createDocumentFragment();
  let last = 0;
  for (const { from, to, type } of tokenizeChef(text)) {
    if (from > last) fragment.append(text.slice(last, from));
    const span = document.createElement("span");
    span.className = "tok-" + type;
    span.textContent = text.slice(from, to);
    fragment.append(span);
    last = to;
  }
  if (last < text.length) fragment.append(text.slice(last));
  el.replaceChildren(fragment);
}

document.querySelectorAll(".recipe-card pre code, pre.snippet").forEach(highlight);

/* ----- Recipe cards: copy + open-in-playground ----- */

// base64url-encode a Unicode string (mirrored by decodeRecipeHash in the
// playground's editor.js — keep the two in sync).
function encodeRecipeHash(source, input) {
  const json = JSON.stringify({ c: source, i: input || "" });
  const bytes = new TextEncoder().encode(json);
  let bin = "";
  for (const byte of bytes) bin += String.fromCharCode(byte);
  return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

document.querySelectorAll(".recipe-card").forEach((card) => {
  const pre = card.querySelector("pre");
  if (!pre) return;
  const source = pre.textContent.replace(/\s+$/, "") + "\n";
  const input = card.getAttribute("data-input") || "";

  const copyBtn = card.querySelector(".copy-btn");
  if (copyBtn) {
    const label = copyBtn.querySelector(".copy-label") || copyBtn;
    const icon = copyBtn.querySelector(".icon");
    let timer;
    const flash = (text, ok, ms) => {
      label.textContent = text;
      icon?.classList.toggle("icon-check", ok);
      icon?.classList.toggle("icon-copy", !ok);
      clearTimeout(timer);
      timer = setTimeout(() => {
        label.textContent = "Copy";
        icon?.classList.replace("icon-check", "icon-copy");
      }, ms);
    };
    copyBtn.addEventListener("click", () => {
      navigator.clipboard.writeText(source).then(
        () => flash("Copied", true, 1400),
        () => flash("Press Ctrl+C", false, 1800),
      );
    });
  }

  const openLink = card.querySelector(".open-playground");
  if (openLink) {
    openLink.href = "../editor/#recipe=" + encodeRecipeHash(source, input);
  }
});
