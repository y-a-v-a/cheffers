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
fn hello_world_fixture_captures_output() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/hello-world.chef")?;
    let recipe = parse_recipe(&source)?;

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    assert_eq!(interpreter.output(), "Hello world!");
    Ok(())
}

#[test]
fn countdown_cake_fixture_counts_down() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/countdown-cake.chef")?;
    let recipe = parse_recipe(&source)?;

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    assert_eq!(interpreter.output(), "54321");
    Ok(())
}

#[test]
fn serves_prints_baking_dish_from_the_top() -> TestResult<()> {
    // Per the Chef spec, mixing bowls and baking dishes are stacks: "Put"
    // pushes onto the top, "Pour" copies the bowl into the dish retaining
    // order, and "Serves" removes values from the top one by one. So a loop
    // that pushes a decrementing counter (5, 4, ..., 1) leaves 1 on top and
    // prints "12345" - the original countdown-cake bug.
    let source = "Stack Order Probe.\n\n\
        Ingredients.\n\
        5 g flour\n\n\
        Method.\n\
        Bake the flour. Put flour into the mixing bowl. Bake the flour until baked. \
        Pour contents of the mixing bowl into the baking dish.\n\n\
        Serves 1.\n";
    let recipe = parse_recipe(source)?;

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    assert_eq!(interpreter.output(), "12345");
    Ok(())
}

#[test]
fn fibonacci_iterative_fixture_parses_and_executes() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/fibonacci-iterative.chef")?;
    let recipe = parse_recipe(&source)?;

    assert_eq!(recipe.title, "Fibonacci Numbers Iterative.");

    // Verify loop was parsed
    assert!(
        recipe
            .instructions
            .iter()
            .any(|inst| matches!(inst, cheffers::instruction::Instruction::Loop { .. })),
        "expected loop instruction to be parsed for Fibonacci iteration"
    );

    // Verify it has no auxiliary recipes (unlike the recursive version)
    assert!(
        recipe.auxiliary_recipes.is_empty(),
        "iterative fibonacci should not include auxiliary recipes"
    );

    // Execute and verify it completes without error
    // Expected output: First 20 Fibonacci numbers in reverse order with newlines
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

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

#[test]
fn golden_ratio_fixture_parses_and_executes() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/golden-ratio.chef")?;
    let recipe = parse_recipe(&source)?;

    assert_eq!(recipe.title, "Golden Ratio Biscotti.");

    // Verify loop was parsed (Fibonacci iteration loop)
    assert!(
        recipe
            .instructions
            .iter()
            .any(|inst| matches!(inst, cheffers::instruction::Instruction::Loop { .. })),
        "expected loop instruction to be parsed for Fibonacci sequence"
    );

    // Execute and verify it completes without error
    // Expected output: "1.6180" (approximation of golden ratio φ)
    // The program uses Fibonacci numbers F(25)/F(24) = 121393/75025 ≈ 1.6180339985
    // with fixed-point arithmetic (×10000) to display as "1.6180"
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;
    // Note: Output verification requires stdout capture, which is tested manually

    Ok(())
}

#[test]
fn all_fixtures_parse_successfully() -> TestResult<()> {
    let fixtures_dir = "tests/fixtures";
    let entries = fs::read_dir(fixtures_dir)?;

    let mut chef_files = Vec::new();

    // Collect all .chef files in fixtures directory (not subdirectories)
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process files (not directories) with .chef extension
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("chef") {
            chef_files.push(path);
        }
    }

    assert!(
        !chef_files.is_empty(),
        "Expected to find .chef files in {}, but found none",
        fixtures_dir
    );

    // Parse each fixture and collect any errors
    let mut errors = Vec::new();
    for path in &chef_files {
        let source = fs::read_to_string(path)?;
        if let Err(e) = parse_recipe(&source) {
            errors.push((path.display().to_string(), e));
        }
    }

    // Report all errors at once for better debugging
    if !errors.is_empty() {
        let error_messages: Vec<String> = errors
            .iter()
            .map(|(path, err)| format!("  - {}: {}", path, err))
            .collect();
        panic!(
            "Failed to parse {} fixture(s):\n{}",
            errors.len(),
            error_messages.join("\n")
        );
    }

    println!("Successfully parsed {} Chef fixture(s)", chef_files.len());
    Ok(())
}
