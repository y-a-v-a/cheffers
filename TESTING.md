# Testing Guide

This document describes how to run tests in the Cheffers Chef interpreter project.

## Running Tests

### Using Cargo Aliases (Recommended)

The project includes custom cargo aliases defined in `.cargo/config.toml` for convenient testing:

```bash
# Run all tests
cargo test-all

# Run only the spec fixture tests (59 tests)
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

## Test Structure

### Integration Tests

- **`tests/spec_fixtures.rs`** - Tests for all 59 language spec fixtures in `tests/fixtures/spec/`
- **`tests/recipes.rs`** - Tests for example recipes in `tests/fixtures/`

### Test Fixtures

- **`tests/fixtures/`** - Example Chef recipes (hello-world, fibonacci, etc.)
- **`tests/fixtures/spec/`** - Language specification test files (59 .chef files)

## Test Categories in spec_fixtures.rs

The spec fixture tests are organized by feature:

- **Ingredient & Measurement Tests** (17 tests) - dry, liquid, and either-type measurements
- **Basic Arithmetic Operations** (4 tests) - addition, subtraction, multiplication, division
- **Bowl Operations** (7 tests) - clean, add, combine, divide, remove operations
- **Fold & Stir Operations** (6 tests) - folding, stirring, mixing
- **Output & I/O Operations** (4 tests) - liquefy, unicode output, stdin
- **Loop Tests** (6 tests) - basic loops, nested loops, break (set aside)
- **Metadata Parsing** (4 tests) - cooking time, temperature, gas mark
- **Auxiliary Recipes** (1 test)
- **Error Handling Tests** (8 tests) - parse errors and runtime errors

## Notes

- This is a work in progress - not all tests are expected to pass yet
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
