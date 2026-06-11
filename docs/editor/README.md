# Cheffers Playground

A JsBin-style web editor for the Chef language, powered by the Cheffers
interpreter compiled to WebAssembly. It runs entirely client-side.

Live at: <https://y-a-v-a.github.io/cheffers/editor/>

## How it works

- `crates/cheffers-wasm` wraps the `cheffers` library with a single
  `run_chef(source, input?)` binding (parse + interpret, returns `{ ok, output, error }`;
  `input` is optional whitespace-separated numbers for `Take ... from refrigerator`).
- `wasm-bindgen` generates the JS glue + `.wasm` into `pkg/`.
- `editor.js` wires a [CodeMirror 6](https://codemirror.net/) editor to the
  interpreter with debounced auto-run. It is bundled (with CodeMirror) into the
  committed `editor.bundle.js` so the page has no runtime CDN dependency.
- Errors reuse the interpreter's rich, ANSI-colored diagnostics, converted to
  safe HTML for display.

## Building

The generated artifacts (`pkg/` and `editor.bundle.js`) are committed so the
GitHub Pages site serves them directly. Regenerate them after changing the Rust
interpreter or the editor source:

```bash
# 1. Build the wasm module -> docs/editor/pkg/
./scripts/build-wasm.sh

# 2. Bundle the editor JS -> docs/editor/editor.bundle.js
cd docs/editor
npm install
npm run build
```

Or run both steps at once from the repo root:

```bash
./scripts/build-web.sh
```

## Developing locally

```bash
cd docs
python3 -m http.server 8000
# open http://localhost:8000/editor/
```

## Testing

The playground is tested at several layers (see the repo's `TESTING.md` for the
full picture). The ones rooted here:

```bash
cd docs/editor
npm install

npm test            # Node unit tests for the ANSI->HTML helpers (ansi.js)
npm run test:browser   # Playwright e2e (expects a server; use the script below)
```

The browser test is normally driven by the repo-root helper, which builds the
bundle, serves `docs/`, and runs the Playwright checks:

```bash
./scripts/test-browser.sh
```

The wasm binding itself is tested from the Rust side (`cargo test --workspace`
for the host logic, `./scripts/test-wasm.sh` for the real-wasm integration
tests). Run the whole suite across all layers with `./scripts/test-all.sh`.

## Files

| Path                | Purpose                                              |
| ------------------- | ---------------------------------------------------- |
| `index.html`        | Page markup                                          |
| `editor.css`        | Styling                                              |
| `editor.js`         | Editor source (imports CodeMirror + the wasm module) |
| `editor.bundle.js`  | Built bundle loaded by the page (generated)          |
| `pkg/`              | wasm-bindgen output (generated)                      |
| `package.json`      | Dev dependencies + bundle script                     |
