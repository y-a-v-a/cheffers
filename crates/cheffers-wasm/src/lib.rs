//! WebAssembly bindings for the Cheffers Chef interpreter.
//!
//! This crate is a thin wrapper around the `cheffers` library. It exposes a
//! single entry point, [`run_chef`], which parses and executes a Chef recipe
//! and returns the result to JavaScript as a plain object:
//!
//! ```js
//! import init, { run_chef } from "./pkg/cheffers_wasm.js";
//! await init();
//! const { ok, output, error } = run_chef(source);
//! ```
//!
//! Errors reuse the same [`ErrorFormatter`] the CLI uses, so the web editor
//! shows the exact same rich, spec-referenced diagnostics.

use cheffers::error_formatter::ErrorFormatter;
use cheffers::types::ChefError;
use cheffers::{Interpreter, Parser};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// The outcome of running a Chef recipe, mirrored to a JS object.
#[derive(Serialize)]
struct RunResult {
    /// `true` when the recipe parsed and executed without error.
    ok: bool,
    /// Whatever the recipe served. Present even on runtime errors, so partial
    /// output produced before a failure is still shown.
    output: String,
    /// A formatted, human-readable error message, or empty on success.
    error: String,
}

/// Parses and runs a Chef recipe, returning `{ ok, output, error }`.
///
/// This never throws: parse and runtime failures are reported through the
/// `error` field so the caller can render them however it likes.
#[wasm_bindgen]
pub fn run_chef(source: &str) -> JsValue {
    let result = execute(source);
    // Serializing a small, owned struct cannot realistically fail; fall back
    // to null so the binding still never throws.
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn execute(source: &str) -> RunResult {
    let recipe = match Parser::new(source).parse_recipe() {
        Ok(recipe) => recipe,
        Err(error) => {
            return RunResult {
                ok: false,
                output: String::new(),
                error: ErrorFormatter::format(&ChefError::from(error)),
            };
        }
    };

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);

    match interpreter.run() {
        Ok(()) => RunResult {
            ok: true,
            output: interpreter.output().to_string(),
            error: String::new(),
        },
        Err(error) => RunResult {
            ok: false,
            output: interpreter.output().to_string(),
            error: ErrorFormatter::format(&error),
        },
    }
}

#[cfg(test)]
mod tests {
    //! Host-target unit tests for the wrapper's pure logic. These exercise
    //! [`execute`] directly (no JS runtime needed) and run under `cargo test`.
    //! The real `run_chef` binding is covered separately by the
    //! `wasm-bindgen-test` suite in `tests/web.rs`.

    use super::*;

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

    #[test]
    fn valid_recipe_reports_success_and_output() {
        let result = execute(HELLO_WORLD);
        assert!(result.ok, "expected success, got error: {}", result.error);
        assert_eq!(result.output, "Hello world!");
        assert!(result.error.is_empty());
    }

    #[test]
    fn numeric_output_is_rendered() {
        // A single dry ingredient liquefied would be a char; left as-is it is
        // serialized as its decimal amount.
        let recipe = "Number Nibble.\n\nIngredients.\n42 g answer\n\n\
            Method.\nPut answer into the mixing bowl. \
            Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n";
        let result = execute(recipe);
        assert!(result.ok, "expected success, got error: {}", result.error);
        assert_eq!(result.output, "42");
    }

    #[test]
    fn parse_error_reports_failure_with_message() {
        let result = execute("Totally not a recipe");
        assert!(!result.ok);
        assert!(result.output.is_empty());
        assert!(
            result.error.contains("invalid title"),
            "unexpected error text: {}",
            result.error
        );
    }

    #[test]
    fn runtime_error_reports_failure_with_message() {
        // "pepper" is never declared as an ingredient.
        let recipe = "Bad Soup.\n\nIngredients.\n1 g salt\n\n\
            Method.\nPut pepper into the mixing bowl.\n\nServes 1.\n";
        let result = execute(recipe);
        assert!(!result.ok);
        assert!(
            result.error.contains("undefined ingredient"),
            "unexpected error text: {}",
            result.error
        );
    }

    #[test]
    fn empty_source_is_a_handled_error_not_a_panic() {
        let result = execute("");
        assert!(!result.ok);
        assert!(!result.error.is_empty());
    }
}
