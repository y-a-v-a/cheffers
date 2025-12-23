pub mod error_context;
pub mod error_formatter;
pub mod instruction;
pub mod interpreter;
pub mod parser;
pub mod types;

pub use error_formatter::ErrorFormatter;
pub use interpreter::Interpreter;
pub use parser::Parser;
pub use types::{
    ChefError, ExecutionContext, Measure, ParseError, ParseResult, Recipe, Result, RuntimeError,
    RuntimeResult, Value,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreter_default_is_available() {
        let _ = Interpreter::default();
    }
}
