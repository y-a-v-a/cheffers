# Cheffers Playground

A JsBin-style web editor for the Chef language, powered by the Cheffers
interpreter compiled to WebAssembly. It runs entirely client-side.

Live at: <https://y-a-v-a.github.io/cheffers/editor/>

## How it works

- `crates/cheffers-wasm` wraps the `cheffers` library with a single
  `run_chef(source)` binding (parse + interpret, returns `{ ok, output, error }`).
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

## Files

| Path                | Purpose                                              |
| ------------------- | ---------------------------------------------------- |
| `index.html`        | Page markup                                          |
| `editor.css`        | Styling                                              |
| `editor.js`         | Editor source (imports CodeMirror + the wasm module) |
| `editor.bundle.js`  | Built bundle loaded by the page (generated)          |
| `pkg/`              | wasm-bindgen output (generated)                      |
| `package.json`      | Dev dependencies + bundle script                     |
