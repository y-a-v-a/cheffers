// Cheffers Playground — wires a CodeMirror editor to the Chef interpreter
// compiled to WebAssembly. Everything runs client-side; no server involved.
//
// CodeMirror 6 is loaded as an ES module from a CDN, and the interpreter is
// the locally-built wasm-bindgen output in ./pkg/.

import { EditorView, basicSetup } from "codemirror";
import init, { run_chef } from "./pkg/cheffers_wasm.js";

const EXAMPLES = {
  "hello-world": {
    label: "Hello World",
    source: `Hello World Souffle.

This recipe prints the immortal words "Hello world!", in a basically brute force way. It also makes a lot of food for one person.

Ingredients.
72 g haricot beans
101 eggs
108 g lard
111 cups oil
32 zucchinis
119 ml water
114 g red salmon
100 g dijon mustard
33 potatoes

Method.
Put potatoes into the mixing bowl. Put dijon mustard into the mixing bowl. Put lard into the mixing bowl. Put red salmon into the mixing bowl. Put oil into the mixing bowl. Put water into the mixing bowl. Put zucchinis into the mixing bowl. Put oil into the mixing bowl. Put lard into the mixing bowl. Put lard into the mixing bowl. Put eggs into the mixing bowl. Put haricot beans into the mixing bowl. Liquefy contents of the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "countdown-cake": {
    label: "Countdown Cake",
    source: `Countdown Cake.

A festive countdown recipe that counts from 5 down to 1. Perfect for New Year's Eve celebrations!

Ingredients.
5 g flour

Method.
Bake the flour. Put flour into the mixing bowl. Bake the flour until baked. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
};

const DEFAULT_EXAMPLE = "hello-world";

const outputEl = document.getElementById("output");
const statusEl = document.getElementById("status");
const runBtn = document.getElementById("run");
const autorunEl = document.getElementById("autorun");
const examplesEl = document.getElementById("examples");

let editor;
let ready = false;
let debounceTimer = null;

function setStatus(text, kind) {
  statusEl.textContent = text;
  statusEl.className = "status" + (kind ? " " + kind : "");
}

const ANSI_COLORS = {
  31: "#ff6b6b", // red
  32: "#5ad19a", // green
  33: "#ffd166", // yellow
  34: "#6ea8ff", // blue
  35: "#c792ea", // magenta
  36: "#56d4dd", // cyan
  37: "#e6e6ef", // white
};

function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

// Convert the interpreter's ANSI-colored error output into safe HTML so the
// browser shows the same rich diagnostics the CLI prints. Text is escaped
// before being wrapped in spans, so recipe content echoed in errors is inert.
function ansiToHtml(text) {
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

function render(result) {
  if (result == null) {
    outputEl.textContent = "Internal error: no result returned.";
    outputEl.classList.add("error");
    setStatus("error", "err");
    return;
  }

  if (result.ok) {
    outputEl.textContent = result.output.length ? result.output : "(no output)";
    outputEl.classList.remove("error");
    setStatus("ok", "ok");
  } else {
    // Show any partial output, then the rich (ANSI-colored) error beneath it.
    let html = "";
    if (result.output.length) html += escapeHtml(result.output) + "\n\n";
    html += ansiToHtml(result.error);
    outputEl.innerHTML = html;
    outputEl.classList.add("error");
    setStatus("error", "err");
  }
}

function runNow() {
  if (!ready) return;
  const source = editor.state.doc.toString();
  try {
    render(run_chef(source));
  } catch (err) {
    outputEl.textContent = "Failed to run interpreter: " + err;
    outputEl.classList.add("error");
    setStatus("error", "err");
  }
}

function scheduleRun() {
  if (!autorunEl.checked) return;
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(runNow, 400);
}

function buildEditor(initialDoc) {
  editor = new EditorView({
    doc: initialDoc,
    extensions: [
      basicSetup,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) scheduleRun();
      }),
    ],
    parent: document.getElementById("editor"),
  });
}

function setEditorContent(text) {
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: text },
  });
}

function populateExamples() {
  for (const [key, { label }] of Object.entries(EXAMPLES)) {
    const opt = document.createElement("option");
    opt.value = key;
    opt.textContent = label;
    examplesEl.appendChild(opt);
  }
  examplesEl.value = DEFAULT_EXAMPLE;
}

async function main() {
  populateExamples();
  buildEditor(EXAMPLES[DEFAULT_EXAMPLE].source);

  setStatus("loading interpreter…");
  await init();
  ready = true;
  setStatus("");

  runBtn.addEventListener("click", runNow);
  examplesEl.addEventListener("change", () => {
    const example = EXAMPLES[examplesEl.value];
    if (example) {
      setEditorContent(example.source);
      runNow();
    }
  });

  // Run once on load so the user immediately sees output.
  runNow();
}

main();
