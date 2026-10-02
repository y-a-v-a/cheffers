// Pure helpers for turning the interpreter's ANSI-colored error output into
// safe HTML. Kept free of any browser/DOM/CodeMirror imports so it can be
// unit-tested directly under Node (`node --test`).

// SGR color codes -> CSS classes. The actual colors are defined per palette
// in docs/assets/site.css (.ansi-*), so diagnostics stay legible on the light
// theme as well as the dark ones.
export const ANSI_CLASSES = {
  31: "ansi-red",
  32: "ansi-green",
  33: "ansi-yellow",
  34: "ansi-blue",
  35: "ansi-magenta",
  36: "ansi-cyan",
  37: "ansi-white",
};

/** Escape the HTML-significant characters so text renders literally. */
export function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

// Convert ANSI SGR sequences (bold + the colors above) into classed <span>s.
// All literal text is escaped before being wrapped, so recipe content echoed
// inside error messages can never inject markup.
export function ansiToHtml(text) {
  let html = "";
  let open = false;
  let bold = false;
  let color = null;
  const pattern = /\x1b\[([0-9;]*)m/g;
  let last = 0;
  let match;

  const flushSpan = () => {
    if (open) {
      html += "</span>";
      open = false;
    }
  };
  const openSpan = () => {
    const classes = [];
    if (bold) classes.push("ansi-bold");
    if (color) classes.push(color);
    if (classes.length) {
      html += '<span class="' + classes.join(" ") + '">';
      open = true;
    }
  };

  while ((match = pattern.exec(text)) !== null) {
    html += escapeHtml(text.slice(last, match.index));
    last = pattern.lastIndex;
    flushSpan();
    for (const codeStr of match[1].split(";")) {
      const code = Number(codeStr || "0");
      if (code === 0) {
        bold = false;
        color = null;
      } else if (code === 1) {
        bold = true;
      } else if (ANSI_CLASSES[code]) {
        color = ANSI_CLASSES[code];
      }
    }
    openSpan();
  }
  html += escapeHtml(text.slice(last));
  flushSpan();
  return html;
}
