# Testing Guide

This document describes how to run tests in the Cheffers Chef interpreter project.

## Running Tests

### Using Cargo Aliases (Recommended)

The project includes custom cargo aliases defined in `.cargo/config.toml` for convenient testing:

```bash
# Run all tests
cargo test-all

# Run only the spec fixture tests (64 tests)
cargo test-spec

# Run only the recipe integration tests
cargo test-recipes

# Run spec tests with full output visible
cargo test-spec-verbose

# Run all tests with full output visible
cargo test-verbose

# Run a specific spec test by name
cargo test-spec-one addition

# Run tests showing only failures
cargo test-quiet

# Build and run all tests with all features
cargo check-all
```

### Using Standard Cargo Commands

You can also use standard cargo test commands:

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test spec_fixtures
cargo test --test recipes

# Run specific test by name
cargo test --test spec_fixtures spec_addition

# Run with output visible
cargo test -- --nocapture

# Run with verbose output
cargo test -- --show-output
```

## WebAssembly & Web Playground Tests

The browser playground (`docs/editor/`, built from `crates/cheffers-wasm`) is
covered by its own layers. Run everything — Rust, wasm, and web — at once with:

```bash
./scripts/test-all.sh
```

Or run a single layer:

| Layer | What it covers | Command |
| ----- | -------------- | ------- |
| Rust (host) | Interpreter + the wasm wrapper's pure logic (`execute`) | `cargo test --workspace` |
| Real wasm | The `run_chef` binding inside an actual wasm runtime (Node) | `./scripts/test-wasm.sh` |
| JS unit | The `ansiToHtml`/`escapeHtml` helpers | `cd docs/editor && npm test` |
| Browser e2e | The full page in headless Chromium (CodeMirror + wasm + auto-run + errors) | `./scripts/test-browser.sh` |

The wasm and browser layers need a one-time toolchain: the `wasm32-unknown-unknown`
target plus `wasm-bindgen-cli` (the scripts install these on demand), and
`npm install` inside `docs/editor` for the Node/Playwright pieces. CI runs all of
these automatically (see `.github/workflows/ci.yml`).

## Test Structure

### Integration Tests

- **`tests/spec_fixtures.rs`** - Tests for all 59 language spec fixtures in `tests/fixtures/spec/`
- **`tests/recipes.rs`** - Tests for example recipes in `tests/fixtures/`
- **`crates/cheffers-wasm/src/lib.rs`** (`mod tests`) - Host-target unit tests for the wasm wrapper logic
- **`crates/cheffers-wasm/tests/web.rs`** - `wasm-bindgen-test` integration tests run in a real wasm runtime
- **`docs/editor/tests/ansi.test.mjs`** - Node unit tests for the playground's ANSI→HTML rendering
- **`docs/editor/tests/browser.test.mjs`** - Playwright end-to-end test of the playground page

### Test Fixtures

- **`tests/fixtures/`** - Example Chef recipes (hello-world, fibonacci, etc.)
- **`tests/fixtures/spec/`** - Language specification test files (68 .chef files)

## Test Categories in spec_fixtures.rs

The spec fixture tests are organized by feature:

- **Ingredient & Measurement Tests** - dry, liquid, and either-type measurements
- **Basic Arithmetic Operations** - addition, subtraction, multiplication, division
- **Bowl Operations** - clean, add (incl. "Add dry ingredients"), combine, divide, remove
- **Fold & Stir Operations** - folding, stirring, mixing (seeded shuffle)
- **Output & I/O Operations** - liquefy, unicode output, stdin (via `set_input_values`)
- **Loop Tests** - basic loops, nested loops, condition-vs-decrement ingredients, break (set aside)
- **Metadata Parsing** - cooking time, temperature, gas mark
- **Auxiliary Recipes** - simple, fold-based, and recursive sous-chefs
- **Ingredient Redeclaration** - the newest declaration wins
- **Error Handling Tests** - parse errors and runtime errors

Spec-conformance regression tests (one per deviation found and fixed in the
spec audit: stdin input, loop condition semantics, pour-copies semantics,
liquid dashes, optional ingredient values, and more) live in
`tests/recipes.rs`.

## Notes

- Some tests are parse-only tests (they verify the recipe parses correctly but don't execute)
- Error tests verify that appropriate errors are raised for invalid recipes

## Adding New Tests

To add a new spec fixture test:

1. Add your `.chef` file to `tests/fixtures/spec/`
2. Add a test macro call in `tests/spec_fixtures.rs`:
   ```rust
   spec_test!(test_name, "your-file.chef");  // For execution tests
   spec_parse_only_test!(test_name, "your-file.chef");  // For parse-only tests
   spec_parse_error_test!(test_name, "your-file.chef");  // Expects parse error
   spec_runtime_error_test!(test_name, "your-file.chef");  // Expects runtime error
   ```
