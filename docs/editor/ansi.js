// Pure helpers for turning the interpreter's ANSI-colored error output into
// safe HTML. Kept free of any browser/DOM/CodeMirror imports so it can be
// unit-tested directly under Node (`node --test`).

export const ANSI_COLORS = {
  31: "#ff6b6b", // red
  32: "#5ad19a", // green
  33: "#ffd166", // yellow
  34: "#6ea8ff", // blue
  35: "#c792ea", // magenta
  36: "#56d4dd", // cyan
  37: "#e6e6ef", // white
};

/** Escape the HTML-significant characters so text renders literally. */
export function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

// Convert ANSI SGR sequences (bold + the colors above) into <span> elements.
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
    const styles = [];
    if (bold) styles.push("font-weight:600");
    if (color) styles.push("color:" + color);
    if (styles.length) {
      html += '<span style="' + styles.join(";") + '">';
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
      } else if (ANSI_COLORS[code]) {
        color = ANSI_COLORS[code];
      }
    }
    openSpan();
  }
  html += escapeHtml(text.slice(last));
  flushSpan();
  return html;
}
