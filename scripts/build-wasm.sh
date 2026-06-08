#!/bin/bash
# Build the WebAssembly playground for the Cheffers Chef interpreter.
#
# Compiles the `cheffers-wasm` crate to wasm32 and runs wasm-bindgen to
# generate the JS glue + .wasm consumed by docs/editor/. The output lands in
# docs/editor/pkg/ so the GitHub Pages site can serve it directly.
#
# Usage: ./scripts/build-wasm.sh

set -euo pipefail

# wasm-bindgen's CLI version must match the wasm-bindgen crate version pinned
# in crates/cheffers-wasm/Cargo.toml.
WASM_BINDGEN_VERSION="0.2.100"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$REPO_ROOT/docs/editor/pkg"
WASM_FILE="$REPO_ROOT/target/wasm32-unknown-unknown/release/cheffers_wasm.wasm"

cd "$REPO_ROOT"

# 1. Ensure the wasm target is installed.
if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# 2. Ensure a matching wasm-bindgen CLI is available.
if ! command -v wasm-bindgen >/dev/null 2>&1 \
    || [ "$(wasm-bindgen --version | awk '{print $2}')" != "$WASM_BINDGEN_VERSION" ]; then
    echo "Installing wasm-bindgen-cli $WASM_BINDGEN_VERSION..."
    cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --force
fi

# 3. Compile the wasm crate in release mode.
echo "Building cheffers-wasm (release, wasm32)..."
cargo build --release --target wasm32-unknown-unknown -p cheffers-wasm

# 4. Generate JS bindings + processed wasm into the Pages directory.
echo "Running wasm-bindgen -> $OUT_DIR"
mkdir -p "$OUT_DIR"
wasm-bindgen \
    --target web \
    --no-typescript \
    --out-dir "$OUT_DIR" \
    "$WASM_FILE"

echo "Done. Playground assets are in docs/editor/pkg/"
echo "Serve locally with:  (cd docs && python3 -m http.server 8000)"
echo "Then open:           http://localhost:8000/editor/"
