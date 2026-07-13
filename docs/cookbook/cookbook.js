// The Cheffers Cookbook — small enhancements shared by every tutorial page:
//
//   1. The theme toggle, identical to the playground's and stored under the
//      same localStorage key, so the reader's choice follows them between
//      the cookbook and the playground.
//   2. "Copy" buttons on recipe cards.
//   3. "Open in playground" links: the recipe source (and Input panel text,
//      if any) is packed into the URL fragment as base64url JSON, which the
//      playground unpacks on load (see docs/editor/editor.js).
//
// No build step: this file is served as-is.

(function () {
  "use strict";

  /* ----- Theme toggle (mirrors docs/editor/editor.js) ----- */

  var THEMES = [
    { id: "system", label: "System", icon: "🖥️" },
    { id: "parchment", label: "Parchment", icon: "📜" },
    { id: "cast-iron", label: "Cast Iron", icon: "🍳" },
    { id: "espresso", label: "Espresso", icon: "☕" },
  ];
  var THEME_STORAGE_KEY = "cheffers-theme";
  var themeBtn = document.getElementById("theme");

  function storedThemeId() {
    try {
      return localStorage.getItem(THEME_STORAGE_KEY) || "system";
    } catch (e) {
      return "system";
    }
  }

  function applyTheme(id) {
    var root = document.documentElement;
    if (id === "system") root.removeAttribute("data-theme");
    else root.setAttribute("data-theme", id);

    var theme = THEMES.find(function (t) { return t.id === id; }) || THEMES[0];
    if (themeBtn) {
      themeBtn.textContent = theme.icon;
      themeBtn.title = "Theme: " + theme.label + " — click to change";
      themeBtn.setAttribute("aria-label", "Theme: " + theme.label + ". Click to change.");
    }
  }

  function cycleTheme() {
    var index = THEMES.findIndex(function (t) { return t.id === storedThemeId(); });
    var next = THEMES[(index + 1) % THEMES.length].id;
    try {
      localStorage.setItem(THEME_STORAGE_KEY, next);
    } catch (e) {
      /* storage may be unavailable; theme still applies for this session */
    }
    applyTheme(next);
  }

  applyTheme(storedThemeId());
  if (themeBtn) themeBtn.addEventListener("click", cycleTheme);

  /* ----- Recipe cards: copy + open-in-playground ----- */

  // base64url-encode a Unicode string (mirrored by decodeRecipeHash in the
  // playground's editor.js — keep the two in sync).
  function encodeRecipeHash(source, input) {
    var json = JSON.stringify({ c: source, i: input || "" });
    var bytes = new TextEncoder().encode(json);
    var bin = "";
    for (var k = 0; k < bytes.length; k++) bin += String.fromCharCode(bytes[k]);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  document.querySelectorAll(".recipe-card").forEach(function (card) {
    var pre = card.querySelector("pre");
    if (!pre) return;
    var source = pre.textContent.replace(/\s+$/, "") + "\n";
    var input = card.getAttribute("data-input") || "";

    var copyBtn = card.querySelector(".copy-btn");
    if (copyBtn) {
      copyBtn.addEventListener("click", function () {
        navigator.clipboard.writeText(source).then(
          function () {
            copyBtn.textContent = "Copied!";
            setTimeout(function () { copyBtn.textContent = "Copy"; }, 1200);
          },
          function () {
            copyBtn.textContent = "Press Ctrl+C";
            setTimeout(function () { copyBtn.textContent = "Copy"; }, 1600);
          }
        );
      });
    }

    var openLink = card.querySelector(".open-playground");
    if (openLink) {
      openLink.href = "../editor/#recipe=" + encodeRecipeHash(source, input);
    }
  });
})();
