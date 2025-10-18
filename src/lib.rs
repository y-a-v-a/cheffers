pub mod instruction;
pub mod interpreter;
pub mod parser;
pub mod types;

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
