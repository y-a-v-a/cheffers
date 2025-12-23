/// Rich error formatting for Chef interpreter errors
use crate::error_context::{RuntimeContext, SpecReference};
use crate::types::{ChefError, ParseError, RuntimeError};

/// ANSI color codes for terminal output
struct Colors;

impl Colors {
    const RESET: &'static str = "\x1b[0m";
    const BOLD: &'static str = "\x1b[1m";
    const RED: &'static str = "\x1b[31m";
    const YELLOW: &'static str = "\x1b[33m";
    const BLUE: &'static str = "\x1b[34m";
    const CYAN: &'static str = "\x1b[36m";
    const WHITE: &'static str = "\x1b[37m";
}

/// Helper to format colored text
fn colorize(text: &str, color: &str, bold: bool) -> String {
    if bold {
        format!("{}{}{}{}", Colors::BOLD, color, text, Colors::RESET)
    } else {
        format!("{}{}{}", color, text, Colors::RESET)
    }
}

/// Formats Chef errors with rich contextual information
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format a ChefError with full context and helpful information
    pub fn format(error: &ChefError) -> String {
        match error {
            ChefError::Runtime(runtime_err) => Self::format_runtime_error(runtime_err),
            ChefError::Parse(parse_err) => Self::format_parse_error(parse_err),
            ChefError::Io(io_err) => Self::format_io_error(io_err),
        }
    }

    fn format_runtime_error(error: &RuntimeError) -> String {
        match error {
            RuntimeError::UndefinedIngredient => Self::format_undefined_ingredient_error(None),
            RuntimeError::EmptyBowl => Self::format_empty_bowl_error(None),
            RuntimeError::DivisionByZero => Self::format_division_by_zero_error(None),
            RuntimeError::RecursionLimit => Self::format_recursion_limit_error(None),
            RuntimeError::UnknownRecipe(name) => Self::format_unknown_recipe_error(name, None),
            RuntimeError::NoRecipe => Self::format_no_recipe_error(),
            RuntimeError::EarlyTermination => {
                // This is not really an error, just a control flow signal
                String::from("Recipe terminated early (Refrigerate instruction)")
            }
            RuntimeError::BreakLoop => {
                // This is not really an error, just a control flow signal
                String::from("Loop break (Set aside instruction)")
            }
        }
    }

    fn format_undefined_ingredient_error(context: Option<&RuntimeContext>) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize("undefined ingredient", Colors::WHITE, true));
        output.push('\n');

        if let Some(RuntimeContext::UndefinedIngredient {
            ingredient,
            instruction,
        }) = context
        {
            output.push_str(&format!(
                "  {} {}\n",
                colorize("ingredient:", Colors::CYAN, false),
                colorize(ingredient, Colors::WHITE, true)
            ));
            output.push_str(&format!(
                "  {} {}\n",
                colorize("in instruction:", Colors::CYAN, false),
                instruction
            ));
        }

        output.push('\n');
        output.push_str(&format!(
            "  {} This instruction references an ingredient that hasn't been declared\n",
            colorize("=", Colors::BLUE, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize(
                "According to the Chef language specification:",
                Colors::CYAN,
                true
            )
        ));
        output.push_str(&format!("  {}\n", SpecReference::INGREDIENTS.excerpt));
        output.push('\n');
        output.push_str(&format!(
            "  {} Ingredients must be declared in the ingredients section at the top of your\n",
            colorize("note:", Colors::YELLOW, true)
        ));
        output.push_str("  recipe before they can be used in the method.\n");
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str("  Add the ingredient to your ingredients list:\n\n");
        output.push_str("    Ingredients.\n");
        output.push_str("    100 g sugar\n");
        output.push_str("    ...\n");

        output
    }

    fn format_empty_bowl_error(context: Option<&RuntimeContext>) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize("mixing bowl is empty", Colors::WHITE, true));
        output.push('\n');

        if let Some(RuntimeContext::EmptyBowl {
            bowl_index,
            operation,
        }) = context
        {
            output.push_str(&format!(
                "  {} {}{}\n",
                colorize("bowl:", Colors::CYAN, false),
                if *bowl_index == 0 {
                    "the mixing bowl".to_string()
                } else {
                    format!("the {} mixing bowl", ordinal(*bowl_index + 1))
                },
                ""
            ));
            output.push_str(&format!(
                "  {} {}\n",
                colorize("operation:", Colors::CYAN, false),
                operation
            ));
        }

        output.push('\n');
        output.push_str(&format!(
            "  {} You're trying to remove or access a value from an empty mixing bowl\n",
            colorize("=", Colors::BLUE, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize(
                "According to the Chef language specification:",
                Colors::CYAN,
                true
            )
        ));
        output.push_str(&format!("  {}\n", SpecReference::MIXING_BOWLS.excerpt));
        output.push('\n');
        output.push_str(&format!(
            "  {} This error commonly occurs when:\n",
            colorize("note:", Colors::YELLOW, true)
        ));
        output.push_str("  - You fold from a bowl that has no ingredients in it\n");
        output.push_str("  - You try to perform arithmetic on an empty bowl\n");
        output.push_str("  - You forgot to 'Put' ingredients into the bowl first\n");
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output
            .push_str("  Make sure to put ingredients into the mixing bowl before using them:\n\n");
        output.push_str("    Put sugar into mixing bowl.\n");
        output.push_str("    Fold flour into mixing bowl.\n");

        output
    }

    fn format_division_by_zero_error(context: Option<&RuntimeContext>) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize("division by zero", Colors::WHITE, true));
        output.push('\n');

        if let Some(RuntimeContext::DivisionByZero {
            divisor_ingredient,
            bowl_index,
        }) = context
        {
            output.push_str(&format!(
                "  {} {}\n",
                colorize("divisor ingredient:", Colors::CYAN, false),
                colorize(divisor_ingredient, Colors::WHITE, true)
            ));
            output.push_str(&format!(
                "  {} {}\n",
                colorize("mixing bowl:", Colors::CYAN, false),
                if *bowl_index == 0 {
                    "the mixing bowl".to_string()
                } else {
                    format!("the {} mixing bowl", ordinal(*bowl_index + 1))
                }
            ));
        }

        output.push('\n');
        output.push_str(&format!(
            "  {} Cannot divide by an ingredient with value zero\n",
            colorize("=", Colors::BLUE, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize(
                "According to the Chef language specification:",
                Colors::CYAN,
                true
            )
        ));
        output.push_str(&format!("  {}\n", SpecReference::ARITHMETIC.excerpt));
        output.push('\n');
        output.push_str(&format!(
            "  {} Division by zero is mathematically undefined\n",
            colorize("note:", Colors::YELLOW, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str("  Check that your ingredient has a non-zero value before dividing:\n\n");
        output.push_str("    Ingredients.\n");
        output.push_str("    4 g flour  ");
        output.push_str(&colorize("← make sure this isn't 0\n", Colors::CYAN, false));

        output
    }

    fn format_recursion_limit_error(context: Option<&RuntimeContext>) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize(
            "maximum recursion depth exceeded",
            Colors::WHITE,
            true,
        ));
        output.push('\n');

        let (recipe_name, depth, max_depth) = if let Some(RuntimeContext::RecursionLimit {
            recipe_name,
            depth,
            max_depth,
        }) = context
        {
            (recipe_name.clone(), *depth, *max_depth)
        } else {
            ("unknown recipe".to_string(), 64, 64)
        };

        output.push_str(&format!(
            "  {} {}\n",
            colorize("recipe:", Colors::CYAN, false),
            colorize(&recipe_name, Colors::WHITE, true)
        ));
        output.push_str(&format!(
            "  {} {}\n",
            colorize("current depth:", Colors::CYAN, false),
            depth
        ));
        output.push_str(&format!(
            "  {} {}\n",
            colorize("maximum allowed:", Colors::CYAN, false),
            max_depth
        ));

        output.push('\n');
        output.push_str(&format!(
            "  {} Recursion depth limited to {} calls to prevent infinite loops\n",
            colorize("=", Colors::BLUE, true),
            max_depth
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize(
                "According to the Chef language specification:",
                Colors::CYAN,
                true
            )
        ));
        output.push_str(&format!("  {}\n", SpecReference::AUXILIARY_RECIPES.excerpt));
        output.push('\n');
        output.push_str(&format!(
            "  {} Your recipe is calling auxiliary recipes recursively too deeply. This\n",
            colorize("note:", Colors::YELLOW, true)
        ));
        output.push_str("  typically happens when:\n");
        output.push_str("  - A recipe calls itself without a proper termination condition\n");
        output.push_str("  - Multiple recipes form a circular calling pattern\n");
        output.push_str("  - Loop conditions never reach zero\n");
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str(
            "  Check your loop conditions and ensure recursive calls have a base case:\n\n",
        );
        output.push_str("    Verb the counter.\n");
        output.push_str("      ");
        output.push_str(&colorize("...", Colors::CYAN, false));
        output.push_str("\n    Verb counter until verbed.  ");
        output.push_str(&colorize(
            "← this decrements counter\n",
            Colors::CYAN,
            false,
        ));

        output
    }

    fn format_unknown_recipe_error(recipe_name: &str, context: Option<&RuntimeContext>) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize("unknown auxiliary recipe", Colors::WHITE, true));
        output.push('\n');

        output.push_str(&format!(
            "  {} {}\n",
            colorize("recipe:", Colors::CYAN, false),
            colorize(recipe_name, Colors::WHITE, true)
        ));

        if let Some(RuntimeContext::UnknownRecipe {
            available_recipes, ..
        }) = context
        {
            if !available_recipes.is_empty() {
                output.push_str(&format!(
                    "\n  {} Available recipes:\n",
                    colorize("note:", Colors::YELLOW, true)
                ));
                for recipe in available_recipes {
                    output.push_str(&format!("    - {}\n", recipe));
                }
            }
        }

        output.push('\n');
        output.push_str(&format!(
            "  {} The recipe you're trying to serve with doesn't exist\n",
            colorize("=", Colors::BLUE, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize(
                "According to the Chef language specification:",
                Colors::CYAN,
                true
            )
        ));
        output.push_str(&format!("  {}\n", SpecReference::AUXILIARY_RECIPES.excerpt));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str("  Make sure the auxiliary recipe is defined after your main recipe:\n\n");
        output.push_str("    Main Recipe.\n");
        output.push_str("    ...\n");
        output.push_str("    Serve with sauce.  ");
        output.push_str(&colorize(
            "← references auxiliary recipe\n",
            Colors::CYAN,
            false,
        ));
        output.push('\n');
        output.push_str("    Sauce.\n");
        output.push_str("    ...  ");
        output.push_str(&colorize(
            "← auxiliary recipe definition\n",
            Colors::CYAN,
            false,
        ));

        output
    }

    fn format_no_recipe_error() -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize("no recipe to execute", Colors::WHITE, true));
        output.push('\n');
        output.push('\n');
        output.push_str(&format!(
            "  {} No recipe was loaded or parsed\n",
            colorize("=", Colors::BLUE, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str("  Make sure you're providing a valid Chef recipe file:\n\n");
        output.push_str("    cheffers recipe.chef\n");

        output
    }

    fn format_parse_error(error: &ParseError) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize(&format!("{}", error), Colors::WHITE, true));
        output.push('\n');
        output.push('\n');
        output.push_str(&format!(
            "  {} Failed to parse the recipe\n",
            colorize("=", Colors::BLUE, true)
        ));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str("  Check the Chef language specification for proper syntax.\n");
        output.push_str("  Make sure your recipe follows the required structure:\n");
        output.push_str("    1. Recipe Title.\n");
        output.push_str("    2. Ingredients. (optional)\n");
        output.push_str("    3. Method. (required)\n");
        output.push_str("    4. Serves N. (optional)\n");

        output
    }

    fn format_io_error(error: &std::io::Error) -> String {
        let mut output = String::new();

        output.push_str(&colorize("error", Colors::RED, true));
        output.push_str(": ");
        output.push_str(&colorize("I/O error", Colors::WHITE, true));
        output.push('\n');
        output.push_str(&format!("  {}\n", error));
        output.push('\n');
        output.push_str(&format!(
            "  {}\n",
            colorize("suggestion:", Colors::CYAN, true)
        ));
        output.push_str("  Make sure the file exists and you have permission to read it.\n");

        output
    }
}

/// Helper function to convert a number to ordinal form (1st, 2nd, 3rd, etc.)
fn ordinal(n: usize) -> String {
    let suffix = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{}{}", n, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinal_numbers() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(4), "4th");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(12), "12th");
        assert_eq!(ordinal(13), "13th");
        assert_eq!(ordinal(21), "21st");
        assert_eq!(ordinal(22), "22nd");
        assert_eq!(ordinal(23), "23rd");
    }

    #[test]
    fn format_undefined_ingredient() {
        let output = ErrorFormatter::format_undefined_ingredient_error(None);
        assert!(output.contains("undefined ingredient"));
        assert!(output.contains("Chef language specification"));
    }

    #[test]
    fn format_empty_bowl() {
        let output = ErrorFormatter::format_empty_bowl_error(None);
        assert!(output.contains("mixing bowl is empty"));
        assert!(output.contains("suggestion"));
    }
}
