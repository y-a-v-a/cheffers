use cheffers::error_formatter::ErrorFormatter;
use cheffers::{Interpreter, Parser};

use std::{env, fs, process};

fn main() {
    let result = run();
    if let Err(error) = result {
        eprintln!("{}", ErrorFormatter::format(&error));
        process::exit(1);
    }
}

fn run() -> cheffers::Result<()> {
    let path = recipe_path_from_args(env::args());
    let source = fs::read_to_string(&path)?;
    let parser = Parser::new(&source);
    let recipe = parser.parse_recipe()?;

    let mut interpreter = Interpreter::new();
    interpreter.add_recipe(recipe);
    interpreter.run()?;

    Ok(())
}

fn recipe_path_from_args<I>(mut args: I) -> String
where
    I: Iterator<Item = String>,
{
    // Skip binary name
    let _ = args.next();
    args.next().unwrap_or_else(|| "hello.chef".to_string())
}

#[cfg(test)]
mod tests {
    use super::recipe_path_from_args;

    #[test]
    fn defaults_to_hello_chef_when_no_argument() {
        let args = vec!["cheffers".to_string()];
        assert_eq!(recipe_path_from_args(args.into_iter()), "hello.chef");
    }

    #[test]
    fn uses_first_argument_as_path() {
        let args = vec![
            "cheffers".to_string(),
            "tests/fixtures/hello-world.chef".to_string(),
        ];
        assert_eq!(
            recipe_path_from_args(args.into_iter()),
            "tests/fixtures/hello-world.chef"
        );
    }
}
