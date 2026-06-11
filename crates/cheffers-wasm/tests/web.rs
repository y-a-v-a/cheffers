//! Integration tests that run the real `run_chef` binding inside an actual
//! WebAssembly runtime (Node.js by default), exercising the full path through
//! `wasm-bindgen` and the JS-value serialization.
//!
//! Run with:
//!     cargo test -p cheffers-wasm --target wasm32-unknown-unknown --test web
//!
//! The whole file is gated to wasm32 so a plain host `cargo test` skips it.
#![cfg(target_arch = "wasm32")]

use cheffers_wasm::run_chef;
use serde::Deserialize;
use wasm_bindgen_test::*;

/// Mirror of the wrapper's (private) result struct, for reading the JS value
/// back. Field names must match what `run_chef` serializes.
#[derive(Deserialize)]
struct RunResult {
    ok: bool,
    output: String,
    error: String,
}

fn run(source: &str) -> RunResult {
    run_with_input(source, None)
}

fn run_with_input(source: &str, input: Option<String>) -> RunResult {
    serde_wasm_bindgen::from_value(run_chef(source, input)).expect("run_chef must return an object")
}

const HELLO_WORLD: &str = "Hello World Souffle.\n\nIngredients.\n\
    72 g haricot beans\n101 eggs\n108 g lard\n111 cups oil\n32 zucchinis\n\
    119 ml water\n114 g red salmon\n100 g dijon mustard\n33 potatoes\n\n\
    Method.\nPut potatoes into the mixing bowl. Put dijon mustard into the mixing bowl. \
    Put lard into the mixing bowl. Put red salmon into the mixing bowl. \
    Put oil into the mixing bowl. Put water into the mixing bowl. \
    Put zucchinis into the mixing bowl. Put oil into the mixing bowl. \
    Put lard into the mixing bowl. Put lard into the mixing bowl. \
    Put eggs into the mixing bowl. Put haricot beans into the mixing bowl. \
    Liquefy contents of the mixing bowl. \
    Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n";

#[wasm_bindgen_test]
fn valid_recipe_returns_output_object() {
    let result = run(HELLO_WORLD);
    assert!(result.ok);
    assert_eq!(result.output, "Hello world!");
    assert!(result.error.is_empty());
}

#[wasm_bindgen_test]
fn parse_error_returns_error_object() {
    let result = run("Totally not a recipe");
    assert!(!result.ok);
    assert!(result.output.is_empty());
    assert!(result.error.contains("invalid title"));
}

#[wasm_bindgen_test]
fn runtime_error_returns_error_object() {
    let result = run("Bad Soup.\n\nIngredients.\n1 g salt\n\n\
        Method.\nPut pepper into the mixing bowl.\n\nServes 1.\n");
    assert!(!result.ok);
    assert!(result.error.contains("undefined ingredient"));
}

#[wasm_bindgen_test]
fn binding_never_throws_on_garbage_input() {
    // Any input must come back as a result object, never a thrown exception.
    let result = run("\0\u{1}\u{2} not even close to a recipe \u{fffd}");
    assert!(!result.ok);
    assert!(!result.error.is_empty());
}

const DOUBLER: &str = "Doubler Delight.\n\nIngredients.\n0 g sugar\n\n\
    Method.\nTake sugar from refrigerator. Put sugar into the mixing bowl. \
    Add sugar to the mixing bowl. \
    Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n";

#[wasm_bindgen_test]
fn input_round_trips_through_the_binding() {
    let result = run_with_input(DOUBLER, Some("21".to_string()));
    assert!(result.ok, "expected success, got error: {}", result.error);
    assert_eq!(result.output, "42");
}

#[wasm_bindgen_test]
fn take_without_input_reports_an_error_object() {
    let result = run(DOUBLER);
    assert!(!result.ok);
    assert!(result.error.contains("cannot read input"));
}
