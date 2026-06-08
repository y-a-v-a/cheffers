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
