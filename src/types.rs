use std::collections::{HashMap, VecDeque};

use thiserror::Error;

use crate::instruction::Instruction;

#[derive(Clone, Copy, Debug, Default)]
pub enum Measure {
    Dry,
    Liquid,
    #[default]
    Unspecified,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Value {
    pub amount: i64,
    pub measure: Measure,
}

pub type Ingredient = String;
pub type MixingBowl = VecDeque<Value>;
pub type BakingDish = VecDeque<Value>;

#[derive(Clone, Debug, Default)]
pub struct Recipe {
    pub title: String,
    pub ingredients: HashMap<Ingredient, Value>,
    pub instructions: Vec<Instruction>,
    pub auxiliary_recipes: HashMap<String, Recipe>,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionContext {
    pub variables: HashMap<Ingredient, Value>,
    pub mixing_bowls: Vec<MixingBowl>,
    pub baking_dishes: Vec<BakingDish>,
    pub call_stack: Vec<CallFrame>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            mixing_bowls: vec![VecDeque::new()],
            baking_dishes: vec![VecDeque::new()],
            call_stack: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CallFrame {
    pub variables: HashMap<Ingredient, Value>,
    pub mixing_bowls: Vec<MixingBowl>,
    pub baking_dishes: Vec<BakingDish>,
    #[allow(dead_code)]
    pub return_address: usize,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing section: {0}")]
    MissingSection(String),
    #[error("invalid ingredient line: {0}")]
    InvalidIngredient(String),
    #[error("invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("unknown instruction: {0}")]
    UnknownInstruction(String),
    #[error("invalid loop structure")]
    InvalidLoop,
    #[error("unmatched loop markers")]
    UnmatchedLoop,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("ingredient is not defined")]
    UndefinedIngredient,
    #[error("mixing bowl is empty")]
    EmptyBowl,
    #[error("recipe '{0}' is not known")]
    UnknownRecipe(String),
    #[error("no recipe available to execute")]
    NoRecipe,
    #[error("call stack limit reached")]
    RecursionLimit,
    #[error("division by zero")]
    DivisionByZero,
}

#[derive(Debug, Error)]
pub enum ChefError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ChefError>;
pub type ParseResult<T> = std::result::Result<T, ParseError>;
pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_default_is_unspecified() {
        assert!(matches!(Measure::default(), Measure::Unspecified));
    }

    #[test]
    fn execution_context_initial_state() {
        let context = ExecutionContext::new();
        assert_eq!(context.mixing_bowls.len(), 1);
        assert_eq!(context.baking_dishes.len(), 1);
        assert!(context.call_stack.is_empty());
    }
}
