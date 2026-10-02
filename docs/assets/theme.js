// Cheffers — theme toggle shared by the playground and the cookbook.
//
// Each page applies the saved theme in an inline <head> script before first
// paint (no flash of the default palette); this file draws the header button
// and cycles System → Parchment → Cast Iron → Espresso on click. The choice is
// stored under one localStorage key, so it follows the reader between pages.
//
// Plain script, no build step: loaded as-is by both halves of the site.

(function () {
  "use strict";

  var SVG_OPEN =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" ' +
    'stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">';

  var THEMES = [
    {
      id: "system",
      label: "System",
      icon: '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
    },
    {
      id: "parchment",
      label: "Parchment",
      icon:
        '<circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M4.6 4.6 6 6M18 18l1.4 1.4' +
        'M2.5 12h2M19.5 12h2M4.6 19.4 6 18M18 6l1.4-1.4"/>',
    },
    {
      id: "cast-iron",
      label: "Cast Iron",
      icon: '<path d="M20.5 14.2A8.5 8.5 0 1 1 9.8 3.5a6.6 6.6 0 0 0 10.7 10.7z"/>',
    },
    {
      id: "espresso",
      label: "Espresso",
      icon:
        '<path d="M4 9h12v5a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5V9z"/>' +
        '<path d="M16 10.5h1.5a2.5 2.5 0 0 1 0 5H16M8 3.5v2.5M12 3.5v2.5"/>',
    },
  ];
  var STORAGE_KEY = "cheffers-theme";
  var button = document.getElementById("theme");

  function storedThemeId() {
    try {
      return localStorage.getItem(STORAGE_KEY) || "system";
    } catch (e) {
      return "system";
    }
  }

  function applyTheme(id) {
    var root = document.documentElement;
    if (id === "system") root.removeAttribute("data-theme");
    else root.setAttribute("data-theme", id);

    var theme = THEMES.find(function (t) { return t.id === id; }) || THEMES[0];
    if (button) {
      button.innerHTML = SVG_OPEN + theme.icon + "</svg>";
      button.title = "Theme: " + theme.label + " — click to change";
      button.setAttribute("aria-label", "Theme: " + theme.label + ". Click to change.");
    }
  }

  function cycleTheme() {
    var index = THEMES.findIndex(function (t) { return t.id === storedThemeId(); });
    var next = THEMES[(index + 1) % THEMES.length].id;
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch (e) {
      /* storage may be unavailable; the theme still applies for this visit */
    }
    applyTheme(next);
  }

  applyTheme(storedThemeId());
  if (button) button.addEventListener("click", cycleTheme);
})();
