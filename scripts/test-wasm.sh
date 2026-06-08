#!/bin/bash
# Run the cheffers-wasm integration tests inside a real WebAssembly runtime.
#
# These use `wasm-bindgen-test`, compiled to wasm32 and executed by
# wasm-bindgen-test-runner (Node.js by default), so they validate the actual
# run_chef binding and its JS-value output — not just the host-side logic.
#
# The fast host-side unit tests run with the normal suite (cargo test); this
# script covers the slower, true-wasm layer.
#
# Usage: ./scripts/test-wasm.sh

set -euo pipefail

# Must match the wasm-bindgen version pinned in crates/cheffers-wasm/Cargo.toml.
WASM_BINDGEN_VERSION="0.2.100"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if ! command -v node >/dev/null 2>&1; then
    echo "error: Node.js is required to run the wasm tests (wasm-bindgen-test-runner uses it)." >&2
    exit 1
fi

# Ensure the wasm target is installed.
if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Ensure the test runner (shipped with wasm-bindgen-cli) is present and matches.
if ! command -v wasm-bindgen-test-runner >/dev/null 2>&1 \
    || [ "$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')" != "$WASM_BINDGEN_VERSION" ]; then
    echo "Installing wasm-bindgen-cli $WASM_BINDGEN_VERSION (provides the test runner)..."
    cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --force
fi

echo "Running cheffers-wasm integration tests in wasm (Node)..."
cargo test -p cheffers-wasm --target wasm32-unknown-unknown --test web
