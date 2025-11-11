use std::error::Error;
use std::fs;

use cheffers::parser::Parser;
use cheffers::types::{ParseError, Recipe};
use cheffers::Interpreter;

type TestResult<T> = Result<T, Box<dyn Error>>;

fn read_fixture(path: &str) -> TestResult<String> {
    Ok(fs::read_to_string(path)?)
}

fn parse_recipe(source: &str) -> Result<Recipe, ParseError> {
    Parser::new(source).parse_recipe()
}

#[test]
fn hello_world_fixture_parses() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/hello-world.chef")?;
    let recipe = parse_recipe(&source)?;
    assert_eq!(recipe.title, "Hello World Souffle.");
    assert!(
        recipe.auxiliary_recipes.is_empty(),
        "hello world should not include auxiliary recipes"
    );
    Ok(())
}

#[test]
fn fibonacci_fixture_parses_with_aux_recipe() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/fibonacci.chef")?;
    let recipe = parse_recipe(&source)?;
    assert!(
        recipe.auxiliary_recipes.contains_key("Caramel Sauce."),
        "expected auxiliary recipe named 'Caramel Sauce.'; available: {:?}",
        recipe.auxiliary_recipes.keys().collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn loop_test_recipe_parses_and_executes() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/spec/loop-test.chef")?;
    let recipe = parse_recipe(&source)?;

    // Verify loop was parsed
    assert!(
        recipe
            .instructions
            .iter()
            .any(|inst| matches!(inst, cheffers::instruction::Instruction::Loop { .. })),
        "expected loop instruction to be parsed"
    );

    // Verify it executes without error
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    Ok(())
}
