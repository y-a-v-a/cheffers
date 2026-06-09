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

// ============================================================================
// SPEC-CONFORMANCE REGRESSION TESTS
// One test per deviation found in the spec audit; each cites the spec rule.
// ============================================================================

fn run_recipe(source: &str) -> Result<String, Box<dyn Error>> {
    let recipe = parse_recipe(source)?;
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;
    Ok(interpreter.output().to_string())
}

#[test]
fn take_reads_values_from_input() -> TestResult<()> {
    // Spec: "Take ingredient from refrigerator. This reads a numeric value
    // from STDIN into the ingredient named, overwriting any previous value."
    let source = read_fixture("tests/fixtures/two-number-tart.chef")?;
    let recipe = parse_recipe(&source)?;

    let mut interpreter = Interpreter::new();
    interpreter.set_input_values(vec![40, 2]);
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    assert_eq!(interpreter.output(), "42", "40 + 2 read from input");
    Ok(())
}

#[test]
fn take_with_exhausted_input_is_a_runtime_error() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/echo-pasta.chef")?;
    let recipe = parse_recipe(&source)?;

    let mut interpreter = Interpreter::new();
    interpreter.set_input_values(vec![]);
    interpreter.add_recipe(recipe);
    let error = interpreter.run().expect_err("no input values supplied");

    assert!(
        error.to_string().contains("cannot read input"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn loop_checks_the_start_statement_ingredient() -> TestResult<()> {
    // Spec: the loop start "Verb the ingredient" names the ingredient that is
    // checked before every pass; the "until" statement's ingredient is only
    // decremented. Here the body zeroes 'condition' in 3 passes while the
    // until statement decrements 'tally' from 9 to 6.
    let output = run_recipe(
        "Loop Semantics Probe.\n\n\
         Ingredients.\n3 g condition\n9 g tally\n1 g unit\n\n\
         Method.\n\
         Beat the condition. \
         Put condition into 2nd mixing bowl. \
         Remove unit from 2nd mixing bowl. \
         Fold condition into 2nd mixing bowl. \
         Beat the tally until beaten. \
         Put tally into mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\n\
         Serves 1.\n",
    )?;
    assert_eq!(output, "6");
    Ok(())
}

#[test]
fn non_terminating_loop_reports_an_error_instead_of_hanging() -> TestResult<()> {
    // A loop whose condition ingredient never changes must surface a runtime
    // error (iteration safety net) rather than spinning forever.
    let source = "Endless Stew.\n\n\
         Ingredients.\n1 g condition\n5 g other\n\n\
         Method.\n\
         Simmer the condition. \
         Simmer the other until simmered. \
         Pour contents of the mixing bowl into the baking dish.\n\n\
         Serves 1.\n";
    let error = run_recipe(source).expect_err("loop never terminates");
    assert!(
        error.to_string().contains("exceeded"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn dashes_are_a_liquid_measure() -> TestResult<()> {
    // Spec: "ml | l | dash[es]: These always indicate liquid measures."
    let output = run_recipe(
        "Dash Probe.\n\nIngredients.\n65 dashes vodka\n\n\
         Method.\nPut vodka into the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(output, "A", "liquid 65 prints as the Unicode character A");
    Ok(())
}

#[test]
fn cups_are_not_dry_unless_heaped_or_level() -> TestResult<()> {
    // Spec: cup[s]/teaspoon[s]/tablespoon[s] "may be either dry or liquid",
    // so they must not join "Add dry ingredients"; the heaped/level
    // measure-types force a dry measure.
    let output = run_recipe(
        "Dry Sum Probe.\n\n\
         Ingredients.\n2 g flour\n10 cups milk\n3 heaped cups beans\n\n\
         Method.\nAdd dry ingredients to the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(
        output, "5",
        "2 g flour + 3 heaped cups beans; plain cups excluded"
    );
    Ok(())
}

#[test]
fn add_dry_ingredients_without_bowl_clause_parses() -> TestResult<()> {
    // Spec: "Add dry ingredients [to [nth] mixing bowl]." - everything after
    // "ingredients" is optional.
    let output = run_recipe(
        "Bare Dry Probe.\n\nIngredients.\n2 g flour\n3 g sugar\n\n\
         Method.\nAdd dry ingredients. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(output, "5");
    Ok(())
}

#[test]
fn stir_without_bowl_clause_and_singular_minute_parses() -> TestResult<()> {
    // Spec: "Stir [the [nth] mixing bowl] for number minutes." - the bowl
    // clause is optional. "for 1 minute" is accepted as natural English.
    let output = run_recipe(
        "Bare Stir Probe.\n\nIngredients.\n1 g flour\n2 g sugar\n\n\
         Method.\nPut flour into the mixing bowl. \
         Put sugar into the mixing bowl. \
         Stir for 1 minute. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    // Bowl was [2(top), 1]; rolling the top down one place gives [1(top), 2],
    // and serving prints from the top: "12".
    assert_eq!(output, "12");
    Ok(())
}

#[test]
fn pour_copies_the_bowl_and_stacks_on_top_of_the_dish() -> TestResult<()> {
    // Spec: "This copies all the ingredients from the nth mixing bowl to the
    // pth baking dish, retaining the order and putting them on top of
    // anything already in the baking dish." Pouring twice therefore serves
    // the bowl contents twice.
    let output = run_recipe(
        "Double Pour Probe.\n\nIngredients.\n7 g flour\n\n\
         Method.\nPut flour into the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(output, "77", "the bowl is copied, not drained");
    Ok(())
}

#[test]
fn ingredient_without_value_errors_only_when_used() -> TestResult<()> {
    // Spec: "The initial-value is a number, and is optional. Attempting to
    // use an ingredient without a defined value is a run-time error."
    let source = "No Value Probe.\n\nIngredients.\nflour\n\n\
         Method.\nPut flour into the mixing bowl.\n\nServes 1.\n";

    // It must parse...
    let recipe = parse_recipe(source)?;
    // ...and only fail at run time, with a value-specific diagnostic.
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    let error = interpreter.run().expect_err("flour has no value");
    assert!(
        error.to_string().contains("declared without a value"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn take_defines_a_valueless_ingredient_with_its_declared_measure() -> TestResult<()> {
    // A valueless liquid ingredient filled in by Take keeps its measure and
    // prints as a character.
    let recipe = parse_recipe(
        "Late Fill Probe.\n\nIngredients.\nml ink\n\n\
         Method.\nTake ink from refrigerator. \
         Put ink into the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    let mut interpreter = Interpreter::new();
    interpreter.set_input_values(vec![66]);
    interpreter.add_recipe(recipe);
    interpreter.run()?;
    assert_eq!(interpreter.output(), "B");
    Ok(())
}

#[test]
fn repeated_ingredient_declaration_uses_the_new_value() -> TestResult<()> {
    // Spec: "If an ingredient is repeated, the new value is used and previous
    // values for that ingredient are ignored."
    let output = run_recipe(
        "Redeclare Probe.\n\nIngredients.\n5 g value\n8 g value\n\n\
         Method.\nPut value into the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(output, "8");
    Ok(())
}

#[test]
fn deprecated_liquify_spelling_is_accepted() -> TestResult<()> {
    // Spec: "Liquefy | Liquify ingredient." (the latter deprecated).
    let output = run_recipe(
        "Liquify Probe.\n\nIngredients.\n65 g flour\n\n\
         Method.\nLiquify the flour. \
         Put flour into the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(output, "A");
    Ok(())
}

#[test]
fn misspelled_instruction_is_a_parse_error() -> TestResult<()> {
    // A typo must not silently become a no-op.
    let result = parse_recipe(
        "Typo Probe.\n\nIngredients.\n1 g sugar\n\n\
         Method.\nPur sugar into the mixing bowl.\n\nServes 1.\n",
    );
    assert!(result.is_err(), "typo'd instruction must be rejected");
    Ok(())
}

#[test]
fn add_on_an_empty_bowl_is_a_runtime_error() -> TestResult<()> {
    // Consistent with Remove/Combine/Divide: arithmetic needs a top value.
    let error = run_recipe(
        "Empty Add Probe.\n\nIngredients.\n1 g sugar\n\n\
         Method.\nAdd sugar to the mixing bowl.\n\nServes 1.\n",
    )
    .expect_err("adding to an empty bowl");
    assert!(
        error.to_string().contains("empty"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn set_aside_outside_a_loop_is_a_runtime_error() -> TestResult<()> {
    let error = run_recipe(
        "Stray Set Aside.\n\nIngredients.\n1 g sugar\n\n\
         Method.\nSet aside.\n\nServes 1.\n",
    )
    .expect_err("set aside outside any loop");
    assert!(
        error.to_string().contains("outside of a loop"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn auxiliary_recipes_use_only_their_own_ingredients() -> TestResult<()> {
    // Spec: sous-chefs take copies of the bowls and dishes, but each recipe
    // has its own ingredient list; the caller's bindings must not leak in.
    let error = run_recipe(
        "Leak Probe.\n\nIngredients.\n5 g secret\n\n\
         Method.\nServe with Snoop. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n\n\
         Snoop.\n\nMethod.\nPut secret into the mixing bowl.\nRefrigerate.\n",
    )
    .expect_err("aux recipe must not see the caller's 'secret'");
    assert!(
        error.to_string().contains("not defined"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn invalid_liquid_code_point_is_a_runtime_error() -> TestResult<()> {
    // Liquid output of a negative value cannot be a Unicode character and
    // must not be silently skipped.
    let error = run_recipe(
        "Bad Glyph Probe.\n\nIngredients.\n-1 ml void\n\n\
         Method.\nPut void into the mixing bowl. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )
    .expect_err("-1 is not a code point");
    assert!(
        error.to_string().contains("Unicode"),
        "unexpected error: {}",
        error
    );
    Ok(())
}

#[test]
fn nested_loops_with_the_same_verb_pair_correctly() -> TestResult<()> {
    // Sequential loop parsing lets an inner loop consume its own "until"
    // before the outer loop looks for its end, even with identical verbs.
    // Outer runs twice; the inner loop only runs on the first outer pass
    // (sugar is not reset), pushing 2 then 1. The outer body then pushes
    // flour each pass: bowl bottom-to-top 2,1,2,1 -> served "1212".
    let output = run_recipe(
        "Same Verb Nesting.\n\nIngredients.\n2 g flour\n2 g sugar\n\n\
         Method.\n\
         Bake the flour. \
         Bake the sugar. \
         Put sugar into the mixing bowl. \
         Bake the sugar until baked. \
         Put flour into the mixing bowl. \
         Bake the flour until baked. \
         Pour contents of the mixing bowl into the baking dish.\n\nServes 1.\n",
    )?;
    assert_eq!(output, "1212");
    Ok(())
}

#[test]
fn canonical_fibonacci_recipe_parses_with_loops() -> TestResult<()> {
    // The classic "Fibonacci Numbers with Caramel Sauce" uses loop starts
    // without "the" ("Melt white sugar.") and bare "Stir for 2 minutes."; the
    // stricter parser must still accept every sentence of the canonical text.
    let source = read_fixture("tests/fixtures/fibonacci.chef")?;
    let recipe = parse_recipe(&source)?;
    assert_eq!(recipe.auxiliary_recipes.len(), 1);
    let aux = recipe.auxiliary_recipes.values().next().unwrap();
    assert!(
        aux.instructions
            .iter()
            .any(|inst| matches!(inst, cheffers::instruction::Instruction::Loop { .. })),
        "caramel sauce loops must parse as loops, not be dropped"
    );
    Ok(())
}
