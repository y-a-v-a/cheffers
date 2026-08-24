//! The playground dropdown examples (docs/editor/editor.js) are copies of
//! these fixtures; each test pins the output the playground shows for it.

use std::error::Error;
use std::fs;

use cheffers::parser::Parser;
use cheffers::Interpreter;

type TestResult<T> = Result<T, Box<dyn Error>>;

fn run_fixture(path: &str, input: Option<&str>) -> TestResult<String> {
    let source = fs::read_to_string(path)?;
    let recipe = Parser::new(&source).parse_recipe()?;
    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    if let Some(text) = input {
        interpreter.set_input_text(text);
    }
    interpreter.run()?;
    Ok(interpreter.output().to_string())
}

#[test]
fn ratatouille_spells_ratatouille() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/ratatouille.chef", None)?;
    assert_eq!(output, "Ratatouille!");
    Ok(())
}

#[test]
fn saturday_pancakes_yield_a_stack_of_twelve() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/saturday-pancakes.chef", None)?;
    assert_eq!(output, "12");
    Ok(())
}

#[test]
fn gauss_layer_cake_sums_one_to_n() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/gauss-layer-cake.chef", Some("10"))?;
    assert_eq!(output, "55");

    let output = run_fixture("tests/fixtures/gauss-layer-cake.chef", Some("100"))?;
    assert_eq!(output, "5050", "young Gauss would approve");
    Ok(())
}

#[test]
fn tapas_times_table_prints_ten_multiples() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/tapas-times-table.chef", Some("7"))?;
    assert_eq!(output, "7\n14\n21\n28\n35\n42\n49\n56\n63\n70\n");
    Ok(())
}

#[test]
fn steak_au_poivre_sous_chef_returns_the_answer() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/steak-au-poivre.chef", None)?;
    assert_eq!(output, "42", "6 peppercorns times 7 spoons of cognac");
    Ok(())
}

#[test]
fn alphabet_soup_serves_an_anagram_of_alphabet() -> TestResult<()> {
    // "Mix well" shuffles, so pin only the multiset of letters, not the order.
    let output = run_fixture("tests/fixtures/alphabet-soup.chef", None)?;
    let mut letters: Vec<char> = output.chars().collect();
    letters.sort_unstable();
    let mut expected: Vec<char> = "alphabet".chars().collect();
    expected.sort_unstable();
    assert_eq!(letters, expected, "output was: {output}");
    Ok(())
}

#[test]
fn melon_sorbet_stirs_melon_into_lemon() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/melon-sorbet.chef", None)?;
    assert_eq!(output, "lemon");
    Ok(())
}

#[test]
fn bakers_dozen_scones_counts_only_dry_ingredients() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/bakers-dozen-scones.chef", None)?;
    assert_eq!(output, "13");
    Ok(())
}

#[test]
fn overnight_oats_refrigerate_stops_the_recipe() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/overnight-oats.chef", None)?;
    assert_eq!(output, "zzz", "the burnt toast must never be served");
    Ok(())
}

#[test]
fn two_course_supper_serves_two_dishes() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/two-course-supper.chef", None)?;
    assert_eq!(output, "soup\ncake");
    Ok(())
}
