use cheffers::{Interpreter, Parser, Result};

use std::fs;

fn main() -> Result<()> {
    let source = fs::read_to_string("hello.chef")?;
    let parser = Parser::new(&source);
    let recipe = parser.parse_recipe()?;

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    Ok(())
}
