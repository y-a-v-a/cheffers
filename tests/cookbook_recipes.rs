//! The seven teaching recipes from the web cookbook (docs/cookbook/) are
//! real programs, embedded verbatim in the tutorial pages. These tests pin
//! their exact output so the cookbook can never drift out of sync with the
//! interpreter it teaches.

use std::error::Error;
use std::fs;

use cheffers::parser::Parser;
use cheffers::Interpreter;

type TestResult<T> = Result<T, Box<dyn Error>>;

fn run_cookbook_recipe(name: &str, input: Option<&str>) -> TestResult<String> {
    let source = fs::read_to_string(format!("docs/cookbook/recipes/{name}"))?;
    let recipe = Parser::new(&source).parse_recipe()?;
    let mut interpreter = Interpreter::new();
    if let Some(text) = input {
        interpreter.set_input_text(text);
    }
    interpreter.add_recipe(recipe);
    interpreter.run()?;
    Ok(interpreter.output().to_string())
}

#[test]
fn chapter_1_the_answer_broth_serves_42() -> TestResult<()> {
    assert_eq!(run_cookbook_recipe("01-the-answer-broth.chef", None)?, "42");
    Ok(())
}

#[test]
fn chapter_2_alphabet_soup_spells_yum() -> TestResult<()> {
    assert_eq!(run_cookbook_recipe("02-alphabet-soup.chef", None)?, "YUM!");
    Ok(())
}

#[test]
fn chapter_3_fair_share_cookies_computes_six_each() -> TestResult<()> {
    // (12 cookies x 3 batches - 4 sneaky nibbles) / 5 friends = 6 (truncated)
    assert_eq!(
        run_cookbook_recipe("03-fair-share-cookies.chef", None)?,
        "6"
    );
    Ok(())
}

#[test]
fn chapter_4_porridge_weather_report_converts_celsius() -> TestResult<()> {
    // 20 C x 9 / 5 + 32 = 68 F
    assert_eq!(
        run_cookbook_recipe("04-porridge-weather-report.chef", Some("20"))?,
        "68"
    );
    assert_eq!(
        run_cookbook_recipe("04-porridge-weather-report.chef", Some("25"))?,
        "77"
    );
    Ok(())
}

#[test]
fn chapter_5_rocket_salad_counts_down_and_lifts_off() -> TestResult<()> {
    assert_eq!(
        run_cookbook_recipe("05-rocket-salad-countdown.chef", None)?,
        "5 4 3 2 1 Liftoff!"
    );
    Ok(())
}

#[test]
fn chapter_6_stressed_desserts_reverses_the_word() -> TestResult<()> {
    // STRESSED pushed into bowl 1, flipped item by item into bowl 2.
    assert_eq!(
        run_cookbook_recipe("06-stressed-desserts.chef", None)?,
        "DESSERTS"
    );
    Ok(())
}

#[test]
fn chapter_7_squared_milkshake_squares_its_input() -> TestResult<()> {
    assert_eq!(
        run_cookbook_recipe("07-squared-milkshake.chef", Some("7"))?,
        "49"
    );
    assert_eq!(
        run_cookbook_recipe("07-squared-milkshake.chef", Some("12"))?,
        "144"
    );
    Ok(())
}
