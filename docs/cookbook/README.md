# The Cheffers Cookbook

A seven-course web tutorial for the Chef programming language, served next to
the [playground](../editor/). Static HTML/CSS/JS — no build step; GitHub Pages
serves it as-is.

Live at: <https://y-a-v-a.github.io/cheffers/cookbook/>

## Structure

| Page                             | Teaches                                                |
| -------------------------------- | ------------------------------------------------------ |
| `index.html`                     | What Chef is + the kitchen-to-code translation table   |
| `01-your-first-dish.html`        | Recipe anatomy, `Put`/`Pour`/`Serves`                  |
| `02-cooking-with-letters.html`   | Dry vs. liquid, `Liquefy`, character codes, stack order |
| `03-kitchen-math.html`           | `Add`/`Remove`/`Combine`/`Divide`, truncation          |
| `04-raiding-the-refrigerator.html` | Input with `Take`, valueless ingredients             |
| `05-stir-until-done.html`        | Loops, `Set aside`, separator tricks                   |
| `06-the-great-bowl-shuffle.html` | `Fold`, 2nd bowls, `Clean`/`Stir`/`Mix`, reversal      |
| `07-call-in-the-sous-chef.html`  | Auxiliary recipes, `Refrigerate`, recursion            |
| `reference.html`                 | The whole language on one page                         |

## How it stays correct

Every teaching recipe is a real program in `recipes/*.chef`, embedded verbatim
in the chapter pages. `tests/cookbook_recipes.rs` (run by `cargo test`) pins
each recipe's exact output, so the tutorial cannot drift from the interpreter.

The pages share the playground's look through `../assets/` — `site.css`
(fonts, palettes, header, footer, syntax colors), `theme.js` (the theme toggle,
stored under the `cheffers-theme` localStorage key, so the theme follows the
reader between the two) and `chef-syntax.js` (the Chef highlighter, which
`cookbook.js` applies to recipe cards and snippets). `cookbook.js` is an ES
module, so preview the pages over HTTP (`python3 -m http.server` in `docs/`)
rather than from `file://`.
"Open in playground" links pack a recipe (and Input panel text) into the URL
fragment as `#recipe=<base64url JSON>`; the decoding lives in
`../editor/editor.js` (`decodeRecipeHash`) and the encoding in `cookbook.js`
(`encodeRecipeHash`) — keep them in sync. The hand-off is covered by the
playground's browser e2e test (`../editor/tests/browser.test.mjs`, run via
`scripts/test-browser.sh`).
