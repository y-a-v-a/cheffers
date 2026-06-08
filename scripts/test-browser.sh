#!/bin/bash
# End-to-end browser test for the web playground.
#
# Rebuilds the editor bundle, serves docs/ locally, and drives the page in
# headless Chromium (Playwright) to verify the full stack: CodeMirror, the
# wasm interpreter, auto-run, example switching, and error rendering.
#
# Requires: node, the playground's dev dependencies (npm install), a Chromium
# for Playwright, and the wasm artifacts in docs/editor/pkg/ (build with
# ./scripts/build-wasm.sh if missing).
#
# Usage: ./scripts/test-browser.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EDITOR_DIR="$REPO_ROOT/docs/editor"
PORT="${PORT:-8123}"

if [ ! -f "$EDITOR_DIR/pkg/cheffers_wasm.js" ]; then
    echo "error: wasm artifacts missing. Run ./scripts/build-wasm.sh first." >&2
    exit 1
fi

cd "$EDITOR_DIR"

# Ensure dev dependencies (CodeMirror, esbuild, Playwright, http-server).
if [ ! -d node_modules ]; then
    echo "Installing playground dev dependencies..."
    npm install
fi

# Rebuild the bundle so the test exercises the current editor source.
echo "Building editor bundle..."
npm run build

# A Chromium is needed. Honor a preinstalled location, else fetch one.
if [ -z "${PLAYWRIGHT_BROWSERS_PATH:-}" ] && [ -d /opt/pw-browsers ]; then
    export PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers
fi
if ! npx playwright install chromium >/dev/null 2>&1; then
    echo "note: could not download Chromium; relying on a preinstalled browser." >&2
fi

# Serve docs/ in the background and make sure it is torn down on exit.
echo "Serving docs/ on port $PORT..."
npx http-server "$REPO_ROOT/docs" -p "$PORT" -s >/tmp/cheffers-http.log 2>&1 &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true' EXIT

# Wait for the server to accept connections.
for _ in $(seq 1 30); do
    if curl -sf -o /dev/null "http://localhost:$PORT/editor/"; then
        break
    fi
    sleep 0.5
done

echo "Running browser test..."
BASE_URL="http://localhost:$PORT/editor/" node tests/browser.test.mjs
