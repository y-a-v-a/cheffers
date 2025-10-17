use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Debug)]
enum Measure {
    Dry,
    Liquid,
    Unspecified,
}

#[derive(Clone, Copy, Debug)]
struct Value {
    amount: i64,
    measure: Measure,
}

type Ingredient = String;
type MixingBowl = VecDeque<Value>;
type BakingDish = VecDeque<Value>;

struct Recipe {
    title: String,
    ingredients: HashMap<Ingredient, Value>,
    instructions: Vec<Instruction>,
    auxiliary_recipes: HashMap<String, Recipe>,
}

struct ExecutionContext {
    variables: HashMap<Ingredient, Value>,
    mixing_bowls: Vec<MixingBowl>,
    baking_dishes: Vec<BakingDish>,
    call_stack: Vec<CallFrame>,
}

struct CallFrame {
    variables: HashMap<Ingredient, Value>,
    mixing_bowls: Vec<MixingBowl>,
    baking_dishes: Vec<BakingDish>,
    return_address: usize,
}