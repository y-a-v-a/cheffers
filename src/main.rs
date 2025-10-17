fn main() -> Result<()> {
    let source = std::fs::read_to_string("hello.chef")?;
    let mut parser = Parser::new(&source);
    let recipe = parser.parse_recipe()?;
    
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;
    
    Ok(())
}