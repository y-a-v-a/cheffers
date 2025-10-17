mod instruction;
mod interpreter;
mod parser;
mod types;

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::types::Result;

use std::fs;

fn main() -> Result<()> {
    let source = fs::read_to_string("hello.chef")?;
    let mut parser = Parser::new(&source);
    let recipe = parser.parse_recipe()?;

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    Ok(())
}
