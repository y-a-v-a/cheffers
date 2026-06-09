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

// Macro to generate a test that parses and executes a fixture
macro_rules! spec_test {
    ($test_name:ident, $fixture_path:literal) => {
        #[test]
        fn $test_name() -> TestResult<()> {
            let source = read_fixture(concat!("tests/fixtures/spec/", $fixture_path))?;
            let recipe = parse_recipe(&source)?;

            // Execute the recipe
            let mut interpreter = Interpreter::new();
            interpreter.add_recipe(recipe);
            interpreter.run()?;

            Ok(())
        }
    };
}

// Macro to generate a test that only parses (for parse-error tests)
macro_rules! spec_parse_only_test {
    ($test_name:ident, $fixture_path:literal) => {
        #[test]
        fn $test_name() -> TestResult<()> {
            let source = read_fixture(concat!("tests/fixtures/spec/", $fixture_path))?;
            let _recipe = parse_recipe(&source)?;
            Ok(())
        }
    };
}

// Macro to generate a test that expects a parse error
macro_rules! spec_parse_error_test {
    ($test_name:ident, $fixture_path:literal) => {
        #[test]
        fn $test_name() -> TestResult<()> {
            let source = read_fixture(concat!("tests/fixtures/spec/", $fixture_path))?;
            let result = parse_recipe(&source);
            assert!(
                result.is_err(),
                "Expected parse error for {}",
                $fixture_path
            );
            Ok(())
        }
    };
}

// Macro to generate a test that expects a runtime error
macro_rules! spec_runtime_error_test {
    ($test_name:ident, $fixture_path:literal) => {
        #[test]
        fn $test_name() -> TestResult<()> {
            let source = read_fixture(concat!("tests/fixtures/spec/", $fixture_path))?;
            let recipe = parse_recipe(&source)?;

            let mut interpreter = Interpreter::new();
            interpreter.add_recipe(recipe);
            let result = interpreter.run();

            assert!(
                result.is_err(),
                "Expected runtime error for {}",
                $fixture_path
            );
            Ok(())
        }
    };
}

// Macro to generate a test that executes a fixture and checks its output
macro_rules! spec_output_test {
    ($test_name:ident, $fixture_path:literal, $expected:literal) => {
        #[test]
        fn $test_name() -> TestResult<()> {
            let source = read_fixture(concat!("tests/fixtures/spec/", $fixture_path))?;
            let recipe = parse_recipe(&source)?;

            let mut interpreter = Interpreter::new();
            interpreter.add_recipe(recipe);
            interpreter.run()?;

            assert_eq!(interpreter.output(), $expected);
            Ok(())
        }
    };
}

// ============================================================================
// INGREDIENT & MEASUREMENT TESTS
// ============================================================================

spec_parse_only_test!(
    spec_single_dry_ingredient_g,
    "single-dry-ingredient-g-test.chef"
);
spec_parse_only_test!(
    spec_single_dry_ingredient_kg,
    "single-dry-ingredient-kg-test.chef"
);
spec_parse_only_test!(
    spec_single_dry_ingredient_pinch,
    "single-dry-ingredient-pinch-test.chef"
);
spec_parse_only_test!(
    spec_single_dry_ingredient_pinches,
    "single-dry-ingredient-pinches-test.chef"
);

spec_parse_only_test!(
    spec_single_liquid_ingredient_ml,
    "single-liquid-ingredient-ml-test.chef"
);
spec_parse_only_test!(
    spec_single_liquid_ingredient_l,
    "single-liquid-ingredient-l-test.chef"
);
// Per the spec dash[es] are liquid measures, so the value 67 prints as 'C'.
spec_output_test!(
    spec_single_liquid_ingredient_dash,
    "single-liquid-ingredient-dash-test.chef",
    "C"
);
spec_output_test!(
    spec_single_liquid_ingredient_dashes,
    "single-liquid-ingredient-dashes-test.chef",
    "C"
);

spec_parse_only_test!(
    spec_single_either_ingredient_cup,
    "single-either-ingredient-cup-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_cups,
    "single-either-ingredient-cups-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_tablespoon,
    "single-either-ingredient-tablespoon-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_tablespoons,
    "single-either-ingredient-tablespoons-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_teaspoon,
    "single-either-ingredient-teaspoon-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_teaspoons,
    "single-either-ingredient-teaspoons-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_heaped,
    "single-either-ingredient-heaped-test.chef"
);
spec_parse_only_test!(
    spec_single_either_ingredient_level,
    "single-either-ingredient-level-test.chef"
);

spec_parse_only_test!(spec_fractional_quantity, "fractional-quantity-test.chef");
spec_parse_only_test!(spec_zero_ingredient, "zero-ingredient-test.chef");

// ============================================================================
// BASIC ARITHMETIC OPERATIONS
// ============================================================================

spec_test!(spec_addition, "addition-test.chef");
spec_test!(spec_subtraction, "subtraction-test.chef");
spec_test!(spec_multiplication, "multiplication-test.chef");
spec_test!(spec_division, "division-test.chef");

// ============================================================================
// BOWL OPERATIONS
// ============================================================================

spec_test!(spec_clean_bowl, "clean-bowl-test.chef");
spec_test!(spec_clean_two_bowls, "clean-two-bowls-test.chef");
spec_output_test!(
    spec_add_dry_ingredients,
    "add-dry-ingredients-test.chef",
    "10"
);
spec_test!(spec_add_second_bowl, "add-second-bowl-test.chef");
spec_test!(spec_combine_second_bowl, "combine-second-bowl-test.chef");
spec_test!(spec_divide_second_bowl, "divide-second-bowl-test.chef");
spec_test!(spec_remove_second_bowl, "remove-second-bowl-test.chef");

// ============================================================================
// FOLD & STIR OPERATIONS
// ============================================================================

spec_test!(spec_fold_basic, "fold-basic-test.chef");
spec_test!(spec_fold_second_bowl, "fold-second-bowl-test.chef");
spec_test!(spec_stir_ingredient, "stir-ingredient-test.chef");
spec_test!(spec_stir_rollover, "stir-rollover-test.chef");
spec_test!(spec_mix_randomization, "mix-randomization-test.chef");
spec_test!(spec_mix_second_bowl, "mix-second-bowl-test.chef");

// ============================================================================
// OUTPUT & I/O OPERATIONS
// ============================================================================

spec_test!(spec_liquefy_second_bowl, "liquefy-second-bowl-test.chef");
spec_test!(spec_to_unicode_singular, "to-unicode-singular-test.chef");
spec_test!(spec_to_unicode_pair, "to-unicode-pair-test.chef");

#[test]
fn spec_stdin_echo() -> TestResult<()> {
    let source = read_fixture("tests/fixtures/spec/stdin-echo-test.chef")?;
    let recipe = parse_recipe(&source)?;

    let mut interpreter = Interpreter::new();
    interpreter.set_input_values(vec![42]);
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    assert_eq!(interpreter.output(), "42");
    Ok(())
}

// ============================================================================
// LOOP TESTS
// ============================================================================

spec_test!(spec_loop, "loop-test.chef");
spec_test!(spec_loop_same_ingredient, "loop-same-ingredient-test.chef");
// The loop checks the START-statement ingredient; "until" only decrements
// its (different) ingredient. See the fixtures for the full walk-through.
spec_output_test!(
    spec_loop_different_ingredients,
    "loop-different-ingredients-test.chef",
    "6"
);
spec_output_test!(
    spec_loop_different_decrement,
    "loop-different-decrement-test.chef",
    "C"
);
spec_test!(spec_nested_loops, "nested-loops-test.chef");
spec_test!(spec_empty_loop_body, "empty-loop-body-test.chef");
spec_test!(
    spec_loop_with_clean_instruction,
    "loop-with-clean-instruction-test.chef"
);
spec_test!(
    spec_loop_with_verb_pattern_instructions,
    "loop-with-verb-pattern-instructions-test.chef"
);
spec_test!(spec_set_aside, "set-aside-test.chef");

// ============================================================================
// MULTIPLE SERVING & DISH OPERATIONS
// ============================================================================

spec_test!(spec_serves_two, "serves-two-test.chef");

// ============================================================================
// INGREDIENT REDECLARATION
// ============================================================================

// Spec: "If an ingredient is repeated, the new value is used and previous
// values for that ingredient are ignored." The fixture declares value as 5
// and then as 0; the recipe must serve 0.
spec_output_test!(
    spec_redeclared_ingredient,
    "redeclared-ingredient-test.chef",
    "0"
);

// ============================================================================
// METADATA PARSING (COOKING TIME, TEMPERATURE, ETC.)
// ============================================================================

spec_parse_only_test!(spec_cooking_time, "cooking-time-test.chef");
spec_parse_only_test!(spec_refrigerate_hours, "refrigerate-hours-test.chef");
spec_parse_only_test!(spec_oven_temperature, "oven-temperature-test.chef");
spec_parse_only_test!(
    spec_oven_temperature_gas_mark,
    "oven-temperature-gas-mark-test.chef"
);

// ============================================================================
// AUXILIARY RECIPES
// ============================================================================

spec_test!(spec_simple_auxiliary, "simple-auxiliary-test.chef");
spec_test!(spec_auxiliary_fold, "auxiliary-fold-test.chef");
// Sum 5+4+3+2+1 computed by a recursive sous-chef.
spec_output_test!(
    spec_recursive_auxiliary,
    "recursive-auxiliary-test.chef",
    "15"
);

// ============================================================================
// ERROR HANDLING TESTS - PARSE ERRORS
// ============================================================================

spec_parse_error_test!(spec_wrong_title, "wrong-title-test.chef");
spec_parse_error_test!(
    spec_wrong_title_line_start,
    "wrong-title-line-start-test.chef"
);
spec_parse_error_test!(
    spec_wrong_single_dry_ingredient,
    "wrong-single-dry-ingredient-test.chef"
);
spec_parse_error_test!(
    spec_wrong_ingredients_definition,
    "wrong-ingredients-definition-test.chef"
);

// ============================================================================
// ERROR HANDLING TESTS - RUNTIME ERRORS
// ============================================================================

spec_runtime_error_test!(
    spec_division_by_zero_error,
    "division-by-zero-error-test.chef"
);
spec_runtime_error_test!(spec_empty_bowl_error, "empty-bowl-error-test.chef");
spec_runtime_error_test!(
    spec_undefined_ingredient_error,
    "undefined-ingredient-error-test.chef"
);
