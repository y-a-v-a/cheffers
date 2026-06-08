#!/bin/bash
# Build the complete Cheffers web playground: the WebAssembly module and the
# bundled CodeMirror editor. Outputs land in docs/editor/ ready for GitHub Pages.
#
# Usage: ./scripts/build-web.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 1. Build the wasm module into docs/editor/pkg/.
"$REPO_ROOT/scripts/build-wasm.sh"

# 2. Bundle the editor JS (with CodeMirror) into docs/editor/editor.bundle.js.
echo "Bundling editor JS..."
cd "$REPO_ROOT/docs/editor"
npm install
npm run build

echo "Web playground built. Preview with:"
echo "  (cd docs && python3 -m http.server 8000)  ->  http://localhost:8000/editor/"
