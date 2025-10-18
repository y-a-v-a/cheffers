use std::error::Error;
use std::fs;

use cheffers::parser::Parser;
use cheffers::types::{ParseError, Recipe};

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
fn fibonacci_fixture_currently_fails_with_unmatched_loop() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/fibonacci.chef")?;
    match parse_recipe(&source) {
        Err(ParseError::UnmatchedLoop) => Ok(()),
        Ok(_) => Err("expected fibonacci fixture to report unmatched loop".into()),
        Err(err) => Err(format!("unexpected parse error: {err:?}").into()),
    }
}
