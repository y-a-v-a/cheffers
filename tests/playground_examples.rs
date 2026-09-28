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

// Showpieces: bigger recipes that push the language hard. Their outputs were
// checked against independent reference implementations (Python models of the
// same integer arithmetic, and Machin's formula for pi) when they were written.

/// The first 1501 decimals of pi, from Machin's formula (computed independently
/// of Chef, with arbitrary-precision integers).
const PI_DECIMALS: &str = concat!(
    "141592653589793238462643383279502884197169399375105820974944592307816406286",
    "208998628034825342117067982148086513282306647093844609550582231725359408128",
    "481117450284102701938521105559644622948954930381964428810975665933446128475",
    "648233786783165271201909145648566923460348610454326648213393607260249141273",
    "724587006606315588174881520920962829254091715364367892590360011330530548820",
    "466521384146951941511609433057270365759591953092186117381932611793105118548",
    "074462379962749567351885752724891227938183011949129833673362440656643086021",
    "394946395224737190702179860943702770539217176293176752384674818467669405132",
    "000568127145263560827785771342757789609173637178721468440901224953430146549",
    "585371050792279689258923542019956112129021960864034418159813629774771309960",
    "518707211349999998372978049951059731732816096318595024459455346908302642522",
    "308253344685035261931188171010003137838752886587533208381420617177669147303",
    "598253490428755468731159562863882353787593751957781857780532171226806613001",
    "927876611195909216420198938095257201065485863278865936153381827968230301952",
    "035301852968995773622599413891249721775283479131515574857242454150695950829",
    "533116861727855889075098381754637464939319255060400927701671139009848824012",
    "858361603563707660104710181942955596198946767837449448255379774726847104047",
    "534646208046684259069491293313677028989152104752162056966024058038150193511",
    "253382430035587640247496473263914199272604269922796782354781636009341721641",
    "219924586315030286182974555706749838505494588586926995690927210797509302955",
    "3",
);

#[test]
fn pi_pie_is_filled_with_1501_correct_decimals_of_pi() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/pi-pie.chef", None)?;
    let filling: String = output
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    assert_eq!(filling, format!("3.{PI_DECIMALS}"));

    // The filling sits in a round dish: 33 rows, and nothing but digits,
    // crimped "()" crust and the space around it.
    assert_eq!(output.lines().count(), 33);
    assert!(
        output
            .chars()
            .all(|c| c.is_ascii_digit() || "().\n ".contains(c)),
        "unexpected character in:\n{output}"
    );
    Ok(())
}

#[test]
fn mandelbrot_mille_feuille_renders_the_mandelbrot_set() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/mandelbrot-mille-feuille.chef", None)?;
    let expected = fs::read_to_string("tests/fixtures/mandelbrot-mille-feuille.expected.txt")?;
    assert_eq!(output, expected);
    Ok(())
}

#[test]
fn mirror_glaze_bombe_ray_traces_a_sphere_over_a_checkerboard() -> TestResult<()> {
    let output = run_fixture("tests/fixtures/mirror-glaze-bombe.chef", None)?;
    let expected = fs::read_to_string("tests/fixtures/mirror-glaze-bombe.expected.txt")?;
    assert_eq!(output, expected);
    Ok(())
}
