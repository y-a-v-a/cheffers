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
