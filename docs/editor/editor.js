// Cheffers Playground — wires a CodeMirror editor to the Chef interpreter
// compiled to WebAssembly. Everything runs client-side; no server involved.
//
// CodeMirror is bundled into editor.bundle.js at build time (see package.json),
// and the interpreter is the locally-built wasm-bindgen output in ./pkg/.

import { EditorView, basicSetup } from "codemirror";
import init, { run_chef } from "./pkg/cheffers_wasm.js";
import { escapeHtml, ansiToHtml } from "./ansi.js";

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

A festive countdown recipe that counts from 5 down to 1. Perfect for New Year's Eve celebrations! The sugar rises from 1 to 5 as it is stacked into the mixing bowl, so the bowl pours out 5 4 3 2 1 when served.

Ingredients.
5 g flour
0 g sugar
1 g salt

Method.
Bake the flour. Put salt into the mixing bowl. Add sugar to the mixing bowl. Fold sugar into the mixing bowl. Put sugar into the mixing bowl. Bake the flour until baked. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "doubler-delight": {
    label: "Doubler Delight (input)",
    source: `Doubler Delight.

A simple dessert that takes any number and doubles it using the magic of addition. Try it with your favorite number!

Ingredients.
0 g sugar

Method.
Take sugar from refrigerator. Put sugar into the mixing bowl. Add sugar to the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
    input: "21",
  },
};

const DEFAULT_EXAMPLE = "hello-world";

const outputEl = document.getElementById("output");
const statusEl = document.getElementById("status");
const runBtn = document.getElementById("run");
const autorunEl = document.getElementById("autorun");
const examplesEl = document.getElementById("examples");
const stdinEl = document.getElementById("stdin");
const themeBtn = document.getElementById("theme");

let editor;
let ready = false;
let debounceTimer = null;

// CodeMirror theme wired entirely to the page's --cm-* custom properties, so
// it follows whichever palette is active. A real EditorView.theme() is needed
// to beat CodeMirror's default light theme (its generated `.ͼ2` classes
// otherwise win on specificity).
const cookTheme = EditorView.theme({
  "&": { backgroundColor: "var(--cm-bg)", color: "var(--cm-text)", height: "100%" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "var(--mono)", fontSize: "14px" },
  ".cm-content": { caretColor: "var(--cm-cursor)" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--cm-cursor)" },
  ".cm-gutters": {
    backgroundColor: "var(--cm-gutter-bg)",
    color: "var(--cm-gutter-text)",
    border: "none",
    borderRight: "1px solid var(--border)",
  },
  ".cm-lineNumbers .cm-gutterElement": { color: "var(--cm-gutter-text)" },
  ".cm-activeLine": { backgroundColor: "var(--cm-active-line)" },
  ".cm-activeLineGutter": {
    backgroundColor: "var(--cm-active-gutter)",
    color: "var(--cm-text)",
  },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
    { backgroundColor: "var(--cm-selection)" },
});

// Available themes, in cycle order. "system" follows prefers-color-scheme.
const THEMES = [
  { id: "system", label: "System", icon: "🖥️" },
  { id: "parchment", label: "Parchment", icon: "📜" },
  { id: "cast-iron", label: "Cast Iron", icon: "🍳" },
  { id: "espresso", label: "Espresso", icon: "☕" },
];
const THEME_STORAGE_KEY = "cheffers-theme";

function storedThemeId() {
  try {
    return localStorage.getItem(THEME_STORAGE_KEY) || "system";
  } catch {
    return "system";
  }
}

function applyTheme(id) {
  const root = document.documentElement;
  if (id === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", id);

  const theme = THEMES.find((t) => t.id === id) ?? THEMES[0];
  if (themeBtn) {
    themeBtn.textContent = theme.icon;
    themeBtn.title = `Theme: ${theme.label} — click to change`;
    themeBtn.setAttribute("aria-label", `Theme: ${theme.label}. Click to change.`);
  }
}

function cycleTheme() {
  const index = THEMES.findIndex((t) => t.id === storedThemeId());
  const next = THEMES[(index + 1) % THEMES.length].id;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, next);
  } catch {
    /* storage may be unavailable; theme still applies for this session */
  }
  applyTheme(next);
}

function setStatus(text, kind) {
  statusEl.textContent = text;
  statusEl.className = "status" + (kind ? " " + kind : "");
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
    // The input box stands in for stdin: whitespace-separated numbers, one
    // consumed per "Take ... from refrigerator" instruction.
    render(run_chef(source, stdinEl.value));
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
      cookTheme,
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
  applyTheme(storedThemeId());
  themeBtn.addEventListener("click", cycleTheme);

  populateExamples();
  buildEditor(EXAMPLES[DEFAULT_EXAMPLE].source);

  setStatus("loading interpreter…");
  await init();
  ready = true;
  setStatus("");

  runBtn.addEventListener("click", runNow);
  stdinEl.addEventListener("input", scheduleRun);
  examplesEl.addEventListener("change", () => {
    const example = EXAMPLES[examplesEl.value];
    if (example) {
      setEditorContent(example.source);
      stdinEl.value = example.input ?? "";
      runNow();
    }
  });

  // Run once on load so the user immediately sees output.
  runNow();
}

main();
