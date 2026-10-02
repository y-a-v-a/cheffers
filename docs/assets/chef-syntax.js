// Chef syntax tokenizer, shared by the playground (CodeMirror decorations,
// bundled into editor.bundle.js), the cookbook and the landing page (static
// <pre> blocks, loaded as an ES module). tokenizeChef is pure — text in, token
// ranges out; highlightChef applies it to a DOM element. No dependencies.
//
// Chef has no reserved words in the usual sense, so highlighting follows the
// shape of a recipe instead: the title, the comment, the ingredient list
// (value, measure, name) and the method, where the first word of every
// sentence is the instruction. Mixing bowls, baking dishes and the
// refrigerator — the language's stacks and input — get their own color so a
// long one-paragraph method can be scanned.
//
// Token types map to CSS classes `tok-<type>` (see docs/assets/site.css):
//   title · comment · meta · section · number · unit · verb · bowl

const SECTION = /^(?:Ingredients|Method)\.$/;
const SECTION_ANYWHERE = /^\s*(?:Ingredients|Method)\.\s*$/m;
const META = /^(?:Cooking time|Pre-?heat oven)\b/;
const SERVES = /^(Serves)(\s+)(\d+)/;
const QUANTITY = /^\d+/;
const MEASURE =
  /^(?:(?:heaped|level)\s+)?(?:kg|g|ml|l|pinch(?:es)?|dash(?:es)?|cups?|teaspoons?|tablespoons?)(?=\s)/;
const BOWL = /\b(?:\d+(?:st|nd|rd|th)\s+)?(?:mixing bowls?|baking dish(?:es)?)\b|\brefrigerator\b/g;
const NUMBER = /\b\d+\b/g;
const PLACEHOLDER = /\[[^\]]*\]/g;
// The cookbook's margin notes inside snippets: "...bowl.   ← puts 5 into the bowl"
const ANNOTATION = /\s←.*$/;

function paint(types, from, to, type) {
  for (let i = from; i < to; i++) types[i] = type;
}

function paintAll(types, text, pattern, type, offset = 0) {
  for (const m of text.matchAll(pattern)) {
    paint(types, offset + m.index, offset + m.index + m[0].length, type);
  }
}

// Collapse a per-character type array into [from, to) runs.
function pushRuns(types, offset, out) {
  let i = 0;
  while (i < types.length) {
    const type = types[i];
    let j = i + 1;
    while (j < types.length && types[j] === type) j++;
    if (type) out.push({ from: offset + i, to: offset + j, type });
    i = j;
  }
}

function ingredientLine(line, types) {
  let i = line.length - line.trimStart().length;
  const quantity = QUANTITY.exec(line.slice(i));
  if (quantity) {
    paint(types, i, i + quantity[0].length, "number");
    i += quantity[0].length;
    i += line.slice(i).length - line.slice(i).trimStart().length;
  }
  const measure = MEASURE.exec(line.slice(i));
  if (measure) paint(types, i, i + measure[0].length, "unit");
  paintAll(types, line, PLACEHOLDER, "comment");
}

function methodLine(line, types, from = 0) {
  const note = ANNOTATION.exec(line);
  const end = note ? note.index : line.length;
  const body = line.slice(from, end);

  // Lowest priority first: later paints win.
  paintAll(types, body, NUMBER, "number", from);
  paintAll(types, body, BOWL, "bowl", from);

  for (const m of body.matchAll(/[^.]+/g)) {
    const sentence = m[0];
    const lead = sentence.length - sentence.trimStart().length;
    const word = /^\S+/.exec(sentence.slice(lead));
    if (word) {
      const start = from + m.index + lead;
      let stop = start + word[0].length;
      // Two-word instructions: "Set aside", "Serve with".
      const second = /^\s+(?:aside|with)\b/.exec(sentence.slice(lead + word[0].length));
      if (second && (word[0] === "Set" || word[0] === "Serve")) stop += second[0].length;
      paint(types, start, stop, "verb");
    }
    // Loop ends: "Stir the batter until stirred."
    const until = /\buntil\s+\S+/.exec(sentence);
    if (until) {
      const start = from + m.index + until.index;
      paint(types, start, start + until[0].length, "verb");
    }
  }

  paintAll(types, body, PLACEHOLDER, "comment", from);
  if (note) paint(types, note.index, line.length, "comment");
}

/**
 * Tokenize Chef source into sorted, non-overlapping `{ from, to, type }`
 * ranges. Text that is a full recipe (it has an "Ingredients." or "Method."
 * line) is read from the title down; anything else — a one-line snippet such
 * as "Fold pancakes into the mixing bowl." — is read as method sentences.
 */
export function tokenizeChef(text) {
  const tokens = [];
  const isRecipe = SECTION_ANYWHERE.test(text);
  let state = isRecipe ? "title" : "method";
  let offset = 0;

  for (const line of text.split("\n")) {
    const start = offset;
    offset += line.length + 1;
    const trimmed = line.trim();

    if (!trimmed) {
      // A blank line ends the method; what follows is "Serves N." or the
      // title of an auxiliary recipe.
      if (isRecipe && state === "method") state = "after-method";
      continue;
    }

    const lead = line.length - line.trimStart().length;
    const types = new Array(line.length).fill(null);
    const whole = (type) => paint(types, lead, lead + trimmed.length, type);
    const serves = SERVES.exec(trimmed);

    if (SECTION.test(trimmed)) {
      whole("section");
      state = trimmed.startsWith("I") ? "ingredients" : "method";
    } else if (state !== "method" && /^Method\.\s/.test(trimmed)) {
      // "Method. Put ... " squeezed onto one line.
      paint(types, lead, lead + 7, "section");
      methodLine(line, types, lead + 7);
      state = "method";
    } else if (state !== "method" && META.test(trimmed)) {
      whole("meta");
    } else if (serves && (state === "method" || state === "after-method")) {
      paint(types, lead, lead + serves[1].length, "section");
      const n = lead + serves[1].length + serves[2].length;
      paint(types, n, n + serves[3].length, "number");
      state = "title";
    } else if (state === "title" || state === "after-method") {
      whole("title");
      state = "comment";
    } else if (state === "comment") {
      whole("comment");
    } else if (state === "ingredients") {
      ingredientLine(line, types);
    } else {
      methodLine(line, types);
    }

    pushRuns(types, start, tokens);
  }
  return tokens;
}

/**
 * Highlight a <pre>/<code> element in place: its text is rebuilt as plain
 * text nodes plus `tok-*` spans. textContent is unchanged, so copying the
 * element still yields the exact source.
 */
export function highlightChef(el) {
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
