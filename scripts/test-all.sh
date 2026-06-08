#!/bin/bash
# Run the entire Cheffers test suite across all layers:
#   1. Rust tests (interpreter + wasm wrapper host logic)      — cargo test --workspace
#   2. Real-wasm integration tests (run_chef in Node)          — scripts/test-wasm.sh
#   3. JS unit tests for the playground (ANSI->HTML helpers)    — npm test
#   4. Browser end-to-end test of the playground (Playwright)   — scripts/test-browser.sh
#
# Each layer is independent; this is the single "run everything" entry point.
# Mirrors what CI runs across its jobs.
#
# Usage: ./scripts/test-all.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> [1/4] Rust tests (workspace)"
cargo test --workspace

echo "==> [2/4] Real-wasm integration tests"
"$REPO_ROOT/scripts/test-wasm.sh"

echo "==> [3/4] Playground JS unit tests"
( cd docs/editor && { [ -d node_modules ] || npm install; } && npm test )

echo "==> [4/4] Playground browser end-to-end test"
"$REPO_ROOT/scripts/test-browser.sh"

echo
echo "All test layers passed."
