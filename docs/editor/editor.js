// Cheffers Playground — wires a CodeMirror editor to the Chef interpreter
// compiled to WebAssembly. Everything runs client-side; no server involved.
//
// CodeMirror is bundled into editor.bundle.js at build time (see package.json),
// and the interpreter is the locally-built wasm-bindgen output in ./pkg/.

import { EditorView, basicSetup } from "codemirror";
import { Compartment } from "@codemirror/state";
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
  "ratatouille": {
    label: "Remy's Ratatouille",
    source: `Remy's Ratatouille.

Anyone can cook! This recipe spells out "Ratatouille!" the way a certain rat would: every vegetable is measured so its quantity is the Unicode code point of one letter. The letters go into the bowl in reverse, because a mixing bowl is a stack and the last ingredient in is the first one served.

Ingredients.
82 g tomatoes
97 g aubergines
116 g zucchinis
111 ml olive oil
117 g red peppers
105 g onions
108 g garlic cloves
101 g herbes de provence
33 dashes tabasco

Method.
Put tabasco into the mixing bowl. Put herbes de provence into the mixing bowl. Put garlic cloves into the mixing bowl. Put garlic cloves into the mixing bowl. Put onions into the mixing bowl. Put red peppers into the mixing bowl. Put olive oil into the mixing bowl. Put zucchinis into the mixing bowl. Put aubergines into the mixing bowl. Put zucchinis into the mixing bowl. Put aubergines into the mixing bowl. Put tomatoes into the mixing bowl. Liquefy contents of the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "saturday-pancakes": {
    label: "Saturday Pancakes",
    source: `Saturday Pancakes.

A real Saturday-morning pancake recipe that is also a computer program: it works out how many pancakes the batter makes. The eggs are whisked away to nothing, the sugar is worked into the flour, and the butter is cut in, dividing the mixture into portions. The milk and salt rest in a second bowl and never reach the plate. Serve a stack of twelve.

Ingredients.
250 g flour
50 g sugar
25 g butter
3 eggs
500 ml milk
1 pinch salt

Method.
Whisk the eggs until whisked. Liquefy the butter. Put flour into the mixing bowl. Add sugar to the mixing bowl. Divide butter into the mixing bowl. Put milk into the 2nd mixing bowl. Put salt into the 2nd mixing bowl. Stir the mixing bowl for 2 minutes. Pour contents of the mixing bowl into the baking dish.

Serves 4.
`,
  },
  "gauss-layer-cake": {
    label: "Gauss's Layer Cake (input)",
    source: `Gauss's Layer Cake.

Young Gauss added the numbers 1 to 100 in seconds; this cake does it with layers. Tell the kitchen how many layers you want (the input) and each pass of the loop stacks the current layer count onto the icing sugar, counting down one layer at a time. Ten layers make 55, one hundred make 5050.

Ingredients.
0 g layers
0 g icing sugar

Method.
Take layers from the refrigerator. Put icing sugar into the mixing bowl. Stack the layers. Add layers to the mixing bowl. Stack the layers until stacked. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
    input: "10",
  },
  "tapas-times-table": {
    label: "Tapas Times Table (input)",
    source: `Tapas Times Table.

Ten guests arrive and each orders one more helping of tapas than nobody at all would: the kitchen prints the full times table of your chosen portion size, one line per guest. The second mixing bowl is the kitchen's scratch pad, where the running total is reduced by one portion per round, and the sparkling water (character 10) pours out as newlines.

Ingredients.
0 g portions
10 g guests
0 g serving
10 ml sparkling water

Method.
Take portions from the refrigerator. Put portions into the 2nd mixing bowl. Combine guests into the 2nd mixing bowl. Fold serving into the 2nd mixing bowl. Serve the guests. Put sparkling water into the mixing bowl. Put serving into the mixing bowl. Put serving into the 2nd mixing bowl. Remove portions from the 2nd mixing bowl. Fold serving into the 2nd mixing bowl. Serve the guests until served. Pour contents of the mixing bowl into the baking dish.

Serves 10.
`,
    input: "7",
  },
  "steak-au-poivre": {
    label: "Steak au Poivre (sous-chef)",
    source: `Bistro Steak au Poivre.

The steak is nothing without its sauce, so the head chef calls for a sous-chef. The sous-chef works in a copy of the kitchen with their own ingredient shelf, and whatever ends up in their first mixing bowl is handed back to the head chef. Six crushed peppercorns combined with seven spoons of cognac: the sauce knows the answer to everything.

Ingredients.
1 sirloin steak

Method.
Serve with peppercorn sauce. Pour contents of the mixing bowl into the baking dish.

Serves 1.

Peppercorn Sauce.

Ingredients.
6 g crushed peppercorns
7 ml cognac

Method.
Put crushed peppercorns into the mixing bowl. Combine cognac into the mixing bowl.
`,
  },
  "alphabet-soup": {
    label: "Alphabet Soup (shuffle)",
    source: `Alphabet Soup.

Pasta letters spelling ALPHABET go into the pot, and then the soup is given a good mix, which shuffles the order of everything in the bowl. What ladles out is an anagram of "alphabet" chosen by the interpreter's random shuffle. Natively every run stirs differently; in the playground the shuffle is seeded per page load.

Ingredients.
97 g letter a
98 g letter b
101 g letter e
104 g letter h
108 g letter l
112 g letter p
116 g letter t

Method.
Put letter t into the mixing bowl. Put letter e into the mixing bowl. Put letter b into the mixing bowl. Put letter a into the mixing bowl. Put letter h into the mixing bowl. Put letter p into the mixing bowl. Put letter l into the mixing bowl. Put letter a into the mixing bowl. Liquefy contents of the mixing bowl. Mix the mixing bowl well. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "melon-sorbet": {
    label: "Stirred Melon Sorbet",
    source: `Stirred Melon Sorbet.

The bowl starts out spelling MELON, top to bottom. Stirring rolls the top ingredient down into the bowl by the number of minutes stirred: two minutes bury the m two places, one more minute tucks the e just beneath the l, and the sorbet is served as LEMON. Same five ingredients, entirely different fruit.

Ingredients.
109 g melon balls
101 ml elderflower cordial
108 ml lime juice
111 g orange zest
110 g nutmeg

Method.
Put nutmeg into the mixing bowl. Put orange zest into the mixing bowl. Put lime juice into the mixing bowl. Put elderflower cordial into the mixing bowl. Put melon balls into the mixing bowl. Liquefy contents of the mixing bowl. Stir the mixing bowl for 2 minutes. Stir for 1 minute. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "bakers-dozen-scones": {
    label: "Baker's Dozen Scones",
    source: `Baker's Dozen Scones.

One instruction does all the measuring: adding the dry ingredients drops the sum of every dry ingredient into the bowl in a single move. Flour, sugar, baking powder and salt are dry and count; buttermilk and melted butter are liquid and stay out of the tally. The tray comes out of the oven with a baker's dozen.

Ingredients.
8 g self-raising flour
3 g caster sugar
1 pinch baking powder
1 pinch salt
150 ml buttermilk
55 ml melted butter

Method.
Add dry ingredients to the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 13.
`,
  },
  "overnight-oats": {
    label: "Overnight Oats",
    source: `Overnight Oats.

Some recipes end not with a bang but with a nap. Refrigerating prints the first baking dish and then stops the recipe on the spot, so the kitchen mumbles "zzz" and goes to sleep. The burnt toast (a suspicious 666) is queued up after the fridge closes and is never served, proving nothing runs past a Refrigerate.

Ingredients.
122 ml oat milk
666 g burnt toast

Method.
Put oat milk into the mixing bowl. Put oat milk into the mixing bowl. Put oat milk into the mixing bowl. Pour contents of the mixing bowl into the baking dish. Refrigerate for 1 hour. Put burnt toast into the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "two-course-supper": {
    label: "Two Course Supper",
    source: `Two Course Supper.

A proper supper needs two courses, and this kitchen runs two mixing bowls and two baking dishes to plate them. The first bowl builds the word "soup" with a splash of sparkling water for the newline, the second bowl builds "cake", and each is poured into its own dish. Serving two dishes prints the first course, then dessert.

Ingredients.
115 ml stock
111 g onions
117 g udon noodles
112 g parsnips
99 g cocoa
97 g ground almonds
107 ml kirsch
101 g beaten eggs
10 ml sparkling water

Method.
Put sparkling water into the mixing bowl. Put parsnips into the mixing bowl. Put udon noodles into the mixing bowl. Put onions into the mixing bowl. Put stock into the mixing bowl. Liquefy contents of the mixing bowl. Put beaten eggs into the 2nd mixing bowl. Put kirsch into the 2nd mixing bowl. Put ground almonds into the 2nd mixing bowl. Put cocoa into the 2nd mixing bowl. Liquefy contents of the 2nd mixing bowl. Pour contents of the mixing bowl into the baking dish. Pour contents of the 2nd mixing bowl into the 2nd baking dish.

Serves 2.
`,
  },
};

const DEFAULT_EXAMPLE = "hello-world";

const outputEl = document.getElementById("output");
const statusEl = document.getElementById("status");
const runBtn = document.getElementById("run");
const autorunEl = document.getElementById("autorun");
const examplesEl = document.getElementById("examples");
const stdinEl = document.getElementById("stdin");
const wrapEl = document.getElementById("wrap");
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
const WRAP_STORAGE_KEY = "cheffers-wrap";

// Line wrapping is toggled live by reconfiguring this compartment. Chef
// methods are traditionally written as one long line, so wrapping is on by
// default; the choice persists like the theme.
const wrapCompartment = new Compartment();

function storedWrap() {
  try {
    return localStorage.getItem(WRAP_STORAGE_KEY) !== "0";
  } catch {
    return true;
  }
}

function wrapExtension(enabled) {
  return enabled ? EditorView.lineWrapping : [];
}

function applyWrap(enabled) {
  editor.dispatch({
    effects: wrapCompartment.reconfigure(wrapExtension(enabled)),
  });
  try {
    localStorage.setItem(WRAP_STORAGE_KEY, enabled ? "1" : "0");
  } catch {
    /* storage may be unavailable; the setting still applies this session */
  }
}

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

// A recipe passed in the URL fragment (the cookbook's "Open in playground"
// links): #recipe=<base64url of JSON {c: source, i: input}>. Mirrors
// encodeRecipeHash in docs/cookbook/cookbook.js — keep the two in sync.
function decodeRecipeHash() {
  const match = /[#&]recipe=([A-Za-z0-9_-]+)/.exec(location.hash);
  if (!match) return null;
  try {
    const b64 = match[1].replace(/-/g, "+").replace(/_/g, "/");
    const bytes = Uint8Array.from(atob(b64), (ch) => ch.charCodeAt(0));
    const parsed = JSON.parse(new TextDecoder().decode(bytes));
    if (typeof parsed.c !== "string") return null;
    return { source: parsed.c, input: typeof parsed.i === "string" ? parsed.i : "" };
  } catch {
    return null;
  }
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
      wrapCompartment.of(wrapExtension(wrapEl.checked)),
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
  wrapEl.checked = storedWrap();

  // A recipe in the URL fragment (from the cookbook) beats the default
  // example; it also gets its own entry in the examples dropdown so the
  // selection reflects what's loaded.
  const shared = decodeRecipeHash();
  if (shared) {
    const opt = document.createElement("option");
    opt.value = "__shared";
    opt.textContent = "From the cookbook";
    examplesEl.appendChild(opt);
    examplesEl.value = "__shared";
    stdinEl.value = shared.input;
  }
  buildEditor(shared ? shared.source : EXAMPLES[DEFAULT_EXAMPLE].source);
  wrapEl.addEventListener("change", () => applyWrap(wrapEl.checked));

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
