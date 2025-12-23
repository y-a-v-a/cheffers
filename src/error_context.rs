/// Context information for errors to enable rich error messages
use std::fmt;

/// Location in source code where an error occurred
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: Option<usize>,
    pub snippet: Option<String>,
}

impl SourceLocation {
    pub fn new(line: usize) -> Self {
        Self {
            line,
            column: None,
            snippet: None,
        }
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.snippet = Some(snippet);
        self
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(col) = self.column {
            write!(f, "line {}, column {}", self.line, col)
        } else {
            write!(f, "line {}", self.line)
        }
    }
}

/// Context for runtime errors
#[derive(Debug, Clone)]
pub enum RuntimeContext {
    /// An undefined ingredient was referenced
    UndefinedIngredient {
        ingredient: String,
        instruction: String,
    },
    /// Attempted to fold/remove from an empty bowl
    EmptyBowl {
        bowl_index: usize,
        operation: String,
    },
    /// Division by zero
    DivisionByZero {
        divisor_ingredient: String,
        bowl_index: usize,
    },
    /// Recursion limit exceeded
    RecursionLimit {
        recipe_name: String,
        depth: usize,
        max_depth: usize,
    },
    /// Unknown auxiliary recipe
    UnknownRecipe {
        recipe_name: String,
        available_recipes: Vec<String>,
    },
}

/// Language spec references for different error types
pub struct SpecReference {
    pub title: &'static str,
    pub excerpt: &'static str,
    pub section: &'static str,
}

impl SpecReference {
    pub const INGREDIENTS: Self = SpecReference {
        title: "Ingredients",
        section: "Ingredient List",
        excerpt: "Attempting to use an ingredient without a defined value is a run-time error. \
                  The ingredient list declares ingredients with given initial values and measures.",
    };

    pub const MIXING_BOWLS: Self = SpecReference {
        title: "Mixing Bowls",
        section: "Mixing Bowls and Baking Dishes",
        excerpt: "The ingredients in a mixing bowl or baking dish are ordered, like a stack of pancakes. \
                  New ingredients are placed on top, and if values are removed they are removed from the top.",
    };

    pub const AUXILIARY_RECIPES: Self = SpecReference {
        title: "Auxiliary Recipes",
        section: "Method - Serve with",
        excerpt: "Serve with auxiliary-recipe. This invokes a sous-chef to immediately prepare the \
                  named auxiliary-recipe. The calling chef waits until the sous-chef is finished before continuing.",
    };

    pub const ARITHMETIC: Self = SpecReference {
        title: "Arithmetic Operations",
        section: "Method - Divide",
        excerpt: "Divide ingredient [into [nth] mixing bowl]. This divides the value of ingredient \
                  into the value of the ingredient on top of the nth mixing bowl and stores the result \
                  in the nth mixing bowl.",
    };

    pub const LOOPS: Self = SpecReference {
        title: "Loops",
        section: "Method - Loop Structure",
        excerpt: "Verb the ingredient. This marks the beginning of a loop. The value of ingredient \
                  is checked. If it is non-zero, the body of the loop executes. The value of ingredient \
                  is rechecked. If it is non-zero, the loop executes again.",
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_location_display() {
        let loc = SourceLocation::new(42);
        assert_eq!(format!("{}", loc), "line 42");

        let loc = SourceLocation::new(42).with_column(10);
        assert_eq!(format!("{}", loc), "line 42, column 10");
    }

    #[test]
    fn source_location_with_snippet() {
        let loc = SourceLocation::new(10).with_snippet("Add sugar to mixing bowl.".to_string());
        assert_eq!(loc.snippet, Some("Add sugar to mixing bowl.".to_string()));
    }
}
