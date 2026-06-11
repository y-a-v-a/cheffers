use std::collections::{HashMap, VecDeque};
use std::fmt::Write as _;

use crate::instruction::Instruction;
use crate::types::{
    CallFrame, ExecutionContext, Measure, Recipe, Result, RuntimeError, RuntimeResult, Value,
};

const MAX_CALL_DEPTH: usize = 64;

/// Safety net for non-terminating loops (the spec loop condition can simply
/// never reach zero). Reported as a runtime error instead of hanging the CLI
/// or the browser.
const MAX_LOOP_ITERATIONS: usize = 10_000_000;

/// Where `Take _ingredient_ from refrigerator` reads its numbers from.
enum InputSource {
    /// Read a line from stdin per `Take` (the spec behavior for the CLI).
    Stdin,
    /// Pop pre-supplied tokens (used by tests and embedders without stdin).
    /// Tokens are parsed when consumed so a bad value can be reported against
    /// the ingredient that tried to read it.
    Buffer(VecDeque<String>),
}

pub struct Interpreter {
    context: ExecutionContext,
    recipes: HashMap<String, Recipe>,
    main_recipe_key: Option<String>,
    output: String,
    input: InputSource,
    rng_state: u64,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
            recipes: HashMap::new(),
            main_recipe_key: None,
            output: String::new(),
            input: InputSource::Stdin,
            rng_state: default_rng_seed(),
        }
    }

    /// Supplies the values that `Take _ingredient_ from refrigerator` will
    /// read, instead of reading stdin. Each `Take` consumes one value; a
    /// `Take` beyond the last value is a runtime error.
    pub fn set_input_values(&mut self, values: Vec<i64>) {
        self.input = InputSource::Buffer(values.iter().map(i64::to_string).collect());
    }

    /// Like [`set_input_values`](Self::set_input_values), but takes raw text
    /// split on whitespace (so both "1 2 3" and one number per line work).
    /// Tokens are validated when a `Take` consumes them, so a non-numeric
    /// token is reported against the ingredient that tried to read it.
    pub fn set_input_text(&mut self, text: &str) {
        self.input = InputSource::Buffer(text.split_whitespace().map(String::from).collect());
    }

    /// Seeds the pseudo-random generator behind `Mix [the bowl] well` so a
    /// shuffle can be made reproducible.
    pub fn set_mix_seed(&mut self, seed: u64) {
        // xorshift cannot leave the all-zero state, so nudge a zero seed.
        self.rng_state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
    }

    /// Returns the output produced so far by `run`.
    ///
    /// The interpreter accumulates everything that a recipe "serves" into an
    /// internal buffer instead of writing straight to stdout. The CLI prints
    /// this buffer; the WASM bindings hand it back to JavaScript.
    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn add_recipe(&mut self, recipe: Recipe) {
        let main_key = normalize_recipe_name(&recipe.title);
        self.main_recipe_key = Some(main_key.clone());
        self.recipes.insert(main_key.clone(), recipe.clone());

        for (title, aux) in &recipe.auxiliary_recipes {
            let key = normalize_recipe_name(title);
            self.recipes.insert(key, aux.clone());
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let key = self.main_recipe_key.clone().ok_or(RuntimeError::NoRecipe)?;
        let recipe = self
            .recipes
            .get(&key)
            .cloned()
            .ok_or(RuntimeError::NoRecipe)?;
        self.execute(&recipe)?;
        Ok(())
    }

    fn execute(&mut self, recipe: &Recipe) -> RuntimeResult<()> {
        self.context.variables = recipe.ingredients.clone();
        self.context.unset_ingredients = recipe.unset_ingredients.clone();
        self.context.mixing_bowls.clear();
        self.context.mixing_bowls.push(VecDeque::new());
        self.context.baking_dishes.clear();
        self.context.baking_dishes.push(VecDeque::new());

        for instruction in &recipe.instructions {
            match self.execute_instruction(instruction) {
                Err(RuntimeError::EarlyTermination) => break,
                // "Set aside" ends the innermost loop; outside of any loop the
                // signal would otherwise leak to the caller as a phantom error.
                Err(RuntimeError::BreakLoop) => return Err(RuntimeError::SetAsideOutsideLoop),
                Err(e) => return Err(e),
                Ok(()) => {}
            }
        }

        Ok(())
    }

    /// Looks up an ingredient's current value, distinguishing "never
    /// declared" from "declared without a value" in the resulting error.
    fn get_variable(&self, ingredient: &str) -> RuntimeResult<Value> {
        if let Some(value) = self.context.variables.get(ingredient) {
            return Ok(*value);
        }
        if self.context.unset_ingredients.contains_key(ingredient) {
            Err(RuntimeError::IngredientWithoutValue {
                ingredient: ingredient.to_string(),
            })
        } else {
            Err(RuntimeError::UndefinedIngredient {
                ingredient: ingredient.to_string(),
            })
        }
    }

    /// Reads one numeric value for `Take _ingredient_ from refrigerator`.
    fn read_input(&mut self, ingredient: &str) -> RuntimeResult<i64> {
        match &mut self.input {
            InputSource::Buffer(tokens) => {
                let token = tokens
                    .pop_front()
                    .ok_or_else(|| RuntimeError::InputUnavailable {
                        ingredient: ingredient.to_string(),
                        reason: "no more input values are available".to_string(),
                    })?;
                token
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::InputUnavailable {
                        ingredient: ingredient.to_string(),
                        reason: format!("'{}' is not a numeric value", token),
                    })
            }
            InputSource::Stdin => {
                let mut line = String::new();
                let read = std::io::stdin().read_line(&mut line).map_err(|error| {
                    RuntimeError::InputUnavailable {
                        ingredient: ingredient.to_string(),
                        reason: format!("failed to read from stdin: {}", error),
                    }
                })?;
                if read == 0 {
                    return Err(RuntimeError::InputUnavailable {
                        ingredient: ingredient.to_string(),
                        reason: "end of input reached (stdin is empty)".to_string(),
                    });
                }
                let trimmed = line.trim();
                trimmed
                    .parse::<i64>()
                    .map_err(|_| RuntimeError::InputUnavailable {
                        ingredient: ingredient.to_string(),
                        reason: format!("'{}' is not a numeric value", trimmed),
                    })
            }
        }
    }

    /// xorshift64*: small, deterministic-per-seed PRNG for `Mix well`.
    fn next_random(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn execute_instruction(&mut self, inst: &Instruction) -> RuntimeResult<()> {
        match inst {
            Instruction::Put(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let value = self.get_variable(ingredient)?;
                self.context.mixing_bowls[*bowl_idx].push_front(value);
            }
            Instruction::Fold(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let value = self.context.mixing_bowls[*bowl_idx]
                    .pop_front()
                    .ok_or_else(|| RuntimeError::EmptyBowl {
                        bowl_index: *bowl_idx,
                        operation: format!("Fold {} into mixing bowl", ingredient),
                    })?;
                self.context.variables.insert(ingredient.clone(), value);
            }
            Instruction::Add(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self.get_variable(ingredient)?;
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or_else(|| RuntimeError::EmptyBowl {
                        bowl_index: *bowl_idx,
                        operation: format!("Add {} to mixing bowl", ingredient),
                    })?;
                top.amount += ing_val.amount;
            }
            Instruction::Remove(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self.get_variable(ingredient)?;
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or_else(|| RuntimeError::EmptyBowl {
                        bowl_index: *bowl_idx,
                        operation: format!("Remove {} from mixing bowl", ingredient),
                    })?;
                top.amount -= ing_val.amount;
            }
            Instruction::Combine(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self.get_variable(ingredient)?;
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or_else(|| RuntimeError::EmptyBowl {
                        bowl_index: *bowl_idx,
                        operation: format!("Combine {} into mixing bowl", ingredient),
                    })?;
                top.amount *= ing_val.amount;
            }
            Instruction::Divide(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self.get_variable(ingredient)?;
                if ing_val.amount == 0 {
                    return Err(RuntimeError::DivisionByZero {
                        ingredient: ingredient.clone(),
                        bowl_index: *bowl_idx,
                    });
                }
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or_else(|| RuntimeError::EmptyBowl {
                        bowl_index: *bowl_idx,
                        operation: format!("Divide {} into mixing bowl", ingredient),
                    })?;
                // All Chef values are integers, so division truncates toward
                // zero (the spec is silent on fractional results).
                top.amount /= ing_val.amount;
            }
            Instruction::AddDry(bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let sum: i64 = self
                    .context
                    .variables
                    .values()
                    .filter(|value| matches!(value.measure, Measure::Dry))
                    .map(|value| value.amount)
                    .sum();
                self.context.mixing_bowls[*bowl_idx].push_front(Value {
                    amount: sum,
                    measure: Measure::Dry,
                });
            }
            Instruction::Stir(_, 0) => {}
            Instruction::Stir(bowl_idx, minutes) => {
                self.stir_bowl(*bowl_idx, *minutes);
            }
            Instruction::StirIngredient(ingredient, bowl_idx) => {
                let depth = self.get_variable(ingredient)?.amount;
                if depth > 0 {
                    self.stir_bowl(*bowl_idx, depth as usize);
                }
            }
            Instruction::Mix(bowl_idx) => {
                // Spec: "This randomises the order of the ingredients."
                // Fisher-Yates with the interpreter's seedable PRNG.
                self.ensure_bowl(*bowl_idx);
                let len = self.context.mixing_bowls[*bowl_idx].len();
                for i in (1..len).rev() {
                    let j = (self.next_random() % (i as u64 + 1)) as usize;
                    self.context.mixing_bowls[*bowl_idx].swap(i, j);
                }
            }
            Instruction::Clean(bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                self.context.mixing_bowls[*bowl_idx].clear();
            }
            Instruction::ServeWith(recipe_name) => {
                self.call_auxiliary(recipe_name)?;
            }
            Instruction::Liquefy(ingredient) => {
                // Reuse the lookup for its declared-without-value diagnostics.
                self.get_variable(ingredient)?;
                if let Some(value) = self.context.variables.get_mut(ingredient) {
                    value.measure = Measure::Liquid;
                }
            }
            Instruction::LiquefyBowl(bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                for value in self.context.mixing_bowls[*bowl_idx].iter_mut() {
                    value.measure = Measure::Liquid;
                }
            }
            Instruction::Pour(from_idx, to_idx) => {
                // Spec: "This copies all the ingredients from the nth mixing
                // bowl to the pth baking dish, retaining the order and putting
                // them on top of anything already in the baking dish." The
                // bowl keeps its contents.
                self.ensure_bowl(*from_idx);
                self.ensure_dish(*to_idx);
                let copied = self.context.mixing_bowls[*from_idx].clone();
                let dish = &mut self.context.baking_dishes[*to_idx];
                // Front is the top: push bottom-most first so the copy lands
                // on top of the dish in its original order.
                for value in copied.iter().rev() {
                    dish.push_front(*value);
                }
            }
            Instruction::Serves(count) => {
                self.write_output(*count)?;
            }
            Instruction::Loop {
                condition_var,
                verb: _,
                body,
                decrement_var,
            } => {
                let mut iterations = 0usize;
                loop {
                    // Spec: the ingredient named in the loop START statement is
                    // checked before every pass; the (possibly different)
                    // ingredient in the "until" statement is only decremented.
                    let condition_value = self.get_variable(condition_var)?.amount;
                    if condition_value == 0 {
                        break;
                    }

                    iterations += 1;
                    if iterations > MAX_LOOP_ITERATIONS {
                        return Err(RuntimeError::LoopLimit {
                            ingredient: condition_var.clone(),
                            max_iterations: MAX_LOOP_ITERATIONS,
                        });
                    }

                    for instruction in body {
                        match self.execute_instruction(instruction) {
                            Err(RuntimeError::BreakLoop) => return Ok(()), // Break out of loop
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }

                    // Decrement the ingredient if specified in the until statement
                    if let Some(ref decr_var) = decrement_var {
                        // Validate first for declared-without-value diagnostics.
                        self.get_variable(decr_var)?;
                        if let Some(value) = self.context.variables.get_mut(decr_var) {
                            value.amount -= 1;
                        }
                    }
                }
            }
            Instruction::SetAside => {
                return Err(RuntimeError::BreakLoop);
            }
            Instruction::Take(ingredient) => {
                // Spec: reads a numeric value from STDIN into the ingredient,
                // overwriting any previous value. The declared measure (if
                // any) is kept; an undeclared ingredient becomes unspecified.
                let amount = self.read_input(ingredient)?;
                let measure = self
                    .context
                    .variables
                    .get(ingredient)
                    .map(|value| value.measure)
                    .or_else(|| self.context.unset_ingredients.get(ingredient).copied())
                    .unwrap_or(Measure::Unspecified);
                self.context
                    .variables
                    .insert(ingredient.clone(), Value { amount, measure });
            }
            Instruction::Refrigerate(hours) => {
                if let Some(dish_count) = hours {
                    self.write_output(*dish_count)?;
                }
                return Err(RuntimeError::EarlyTermination);
            }
        }

        Ok(())
    }

    fn call_auxiliary(&mut self, recipe_name: &str) -> RuntimeResult<()> {
        let key = normalize_recipe_name(recipe_name);
        let aux_recipe =
            self.recipes
                .get(&key)
                .cloned()
                .ok_or_else(|| RuntimeError::UnknownRecipe {
                    recipe_name: recipe_name.to_string(),
                })?;

        if self.context.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(RuntimeError::RecursionLimit {
                recipe_name: recipe_name.to_string(),
                depth: self.context.call_stack.len(),
                max_depth: MAX_CALL_DEPTH,
            });
        }

        let frame = CallFrame {
            variables: self.context.variables.clone(),
            unset_ingredients: self.context.unset_ingredients.clone(),
            mixing_bowls: self.context.mixing_bowls.clone(),
            baking_dishes: self.context.baking_dishes.clone(),
            return_address: 0,
        };
        self.context.call_stack.push(frame);

        // The sous-chef gets copies of the caller's bowls and dishes (the
        // current ones are restored from the frame afterwards), but only the
        // auxiliary recipe's own ingredient list.
        self.context.variables = aux_recipe.ingredients.clone();
        self.context.unset_ingredients = aux_recipe.unset_ingredients.clone();

        // Execute auxiliary recipe's instructions without clearing mixing bowls
        for instruction in &aux_recipe.instructions {
            match self.execute_instruction(instruction) {
                Err(RuntimeError::EarlyTermination) => break,
                Err(e) => {
                    // Clean up call stack before propagating error
                    let error = if matches!(e, RuntimeError::BreakLoop) {
                        RuntimeError::SetAsideOutsideLoop
                    } else {
                        e
                    };
                    self.context.call_stack.pop();
                    return Err(error);
                }
                Ok(()) => {}
            }
        }

        // Get the auxiliary's first mixing bowl before restoring state
        let aux_first_bowl = if !self.context.mixing_bowls.is_empty() {
            self.context.mixing_bowls[0].clone()
        } else {
            VecDeque::new()
        };

        if let Some(frame) = self.context.call_stack.pop() {
            self.context.variables = frame.variables;
            self.context.unset_ingredients = frame.unset_ingredients;
            self.context.mixing_bowls = frame.mixing_bowls;
            self.context.baking_dishes = frame.baking_dishes;

            // Transfer auxiliary's first mixing bowl to caller's first mixing bowl
            // "empties it into his first mixing bowl" means we add all values from aux bowl
            self.ensure_bowl(0);
            for value in aux_first_bowl.iter().rev() {
                self.context.mixing_bowls[0].push_front(*value);
            }
        }

        Ok(())
    }

    fn write_output(&mut self, dish_count: usize) -> RuntimeResult<()> {
        for dish in self.context.baking_dishes.iter_mut().take(dish_count) {
            while let Some(value) = dish.pop_front() {
                match value.measure {
                    Measure::Liquid => {
                        let c = u32::try_from(value.amount)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or(RuntimeError::InvalidCharacter {
                                amount: value.amount,
                            })?;
                        self.output.push(c);
                    }
                    _ => {
                        // Writing to a String is infallible.
                        let _ = write!(self.output, "{}", value.amount);
                    }
                }
            }
        }

        Ok(())
    }

    fn ensure_bowl(&mut self, idx: usize) {
        while self.context.mixing_bowls.len() <= idx {
            self.context.mixing_bowls.push(VecDeque::new());
        }
    }

    fn ensure_dish(&mut self, idx: usize) {
        while self.context.baking_dishes.len() <= idx {
            self.context.baking_dishes.push(VecDeque::new());
        }
    }

    fn stir_bowl(&mut self, idx: usize, positions: usize) {
        if positions == 0 {
            return;
        }
        self.ensure_bowl(idx);
        let bowl = &mut self.context.mixing_bowls[idx];
        if bowl.len() <= 1 {
            return;
        }
        if let Some(top) = bowl.pop_front() {
            let len = bowl.len();
            if len == 0 {
                bowl.push_front(top);
                return;
            }
            let target = positions.min(len);
            bowl.insert(target, top);
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_recipe_name(name: &str) -> String {
    name.trim().trim_end_matches('.').to_lowercase()
}

/// Default seed for the `Mix well` shuffle. Natively this is taken from the
/// system clock so each run shuffles differently; on wasm32 (where
/// `SystemTime::now` is unavailable) a fixed seed is used, making playground
/// shuffles deterministic per page load.
fn default_rng_seed() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        if let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let nanos = elapsed.as_nanos() as u64;
            if nanos != 0 {
                return nanos;
            }
        }
    }
    0x9E37_79B9_7F4A_7C15
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChefError, Value};
    use std::collections::HashMap;

    fn recipe_with_put_instruction() -> Recipe {
        let mut ingredients = HashMap::new();
        ingredients.insert(
            "sugar".to_string(),
            Value {
                amount: 1,
                measure: Measure::Dry,
            },
        );

        Recipe {
            title: "Test Recipe".to_string(),
            ingredients,
            unset_ingredients: HashMap::new(),
            instructions: vec![
                Instruction::Put("sugar".to_string(), 0),
                Instruction::Serves(0),
            ],
            auxiliary_recipes: HashMap::new(),
        }
    }

    #[test]
    fn run_without_recipe_returns_error() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.run();
        assert!(matches!(
            result,
            Err(ChefError::Runtime(RuntimeError::NoRecipe))
        ));
    }

    #[test]
    fn run_executes_put_instruction() {
        let mut interpreter = Interpreter::new();
        interpreter.add_recipe(recipe_with_put_instruction());

        interpreter.run().expect("recipe should execute");

        let bowl = interpreter
            .context
            .mixing_bowls
            .first()
            .expect("mixing bowl should exist");
        let value = bowl.front().expect("mixing bowl should contain a value");
        assert_eq!(value.amount, 1);
    }

    #[test]
    fn serve_with_uses_normalized_recipe_names() {
        let mut interpreter = Interpreter::new();
        interpreter.add_recipe(recipe_with_auxiliary());
        interpreter
            .run()
            .expect("recipe with auxiliary should execute");
    }

    #[test]
    fn stir_moves_top_down_by_minutes() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 3,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 2,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 1,
            measure: Measure::Dry,
        });

        interpreter
            .execute_instruction(&Instruction::Stir(0, 2))
            .expect("stir should succeed");

        let amounts: Vec<_> = interpreter.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(amounts, vec![2, 3, 1]);
    }

    #[test]
    fn stir_with_large_minutes_moves_top_to_bottom() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 3,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 2,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 1,
            measure: Measure::Dry,
        });

        interpreter
            .execute_instruction(&Instruction::Stir(0, 10))
            .expect("stir should succeed");

        let amounts: Vec<_> = interpreter.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(amounts, vec![2, 3, 1]);
    }

    #[test]
    fn stir_ingredient_uses_value_for_depth() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 3,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 2,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 1,
            measure: Measure::Dry,
        });
        interpreter.context.variables.insert(
            "depth".to_string(),
            Value {
                amount: 1,
                measure: Measure::Dry,
            },
        );

        interpreter
            .execute_instruction(&Instruction::StirIngredient("depth".to_string(), 0))
            .expect("stir ingredient should succeed");

        let amounts: Vec<_> = interpreter.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(amounts, vec![2, 1, 3]);
    }

    #[test]
    fn mix_randomises_bowl_preserving_contents() {
        // Spec: "Mix well" randomises the order of the ingredients. With a
        // fixed seed the shuffle is reproducible; the multiset of values must
        // always be preserved.
        let mut interpreter = Interpreter::new();
        interpreter.set_mix_seed(42);
        interpreter.ensure_bowl(0);
        for amount in (1..=8).rev() {
            interpreter.context.mixing_bowls[0].push_front(Value {
                amount,
                measure: Measure::Dry,
            });
        }

        interpreter
            .execute_instruction(&Instruction::Mix(0))
            .expect("mix should succeed");

        let mut amounts: Vec<_> = interpreter.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        let shuffled = amounts.clone();
        amounts.sort_unstable();
        assert_eq!(amounts, (1..=8).collect::<Vec<_>>(), "values must survive");

        // Same seed, same shuffle.
        let mut second = Interpreter::new();
        second.set_mix_seed(42);
        second.ensure_bowl(0);
        for amount in (1..=8).rev() {
            second.context.mixing_bowls[0].push_front(Value {
                amount,
                measure: Measure::Dry,
            });
        }
        second
            .execute_instruction(&Instruction::Mix(0))
            .expect("mix should succeed");
        let shuffled_again: Vec<_> = second.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(shuffled, shuffled_again, "same seed must shuffle the same");

        // A different seed should give a different order for 8 elements (the
        // chance of an identical permutation is 1/40320 per seed; these two
        // seeds are fixed, so this stays deterministic).
        let mut third = Interpreter::new();
        third.set_mix_seed(43);
        third.ensure_bowl(0);
        for amount in (1..=8).rev() {
            third.context.mixing_bowls[0].push_front(Value {
                amount,
                measure: Measure::Dry,
            });
        }
        third
            .execute_instruction(&Instruction::Mix(0))
            .expect("mix should succeed");
        let other: Vec<_> = third.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_ne!(shuffled, other, "different seeds should differ");
    }

    #[test]
    fn clean_empties_bowl() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 1,
            measure: Measure::Dry,
        });

        interpreter
            .execute_instruction(&Instruction::Clean(0))
            .expect("clean should succeed");

        assert!(interpreter.context.mixing_bowls[0].is_empty());
    }

    #[test]
    fn remove_subtracts_from_top_value() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 10,
            measure: Measure::Dry,
        });
        interpreter.context.variables.insert(
            "salt".to_string(),
            Value {
                amount: 4,
                measure: Measure::Dry,
            },
        );

        interpreter
            .execute_instruction(&Instruction::Remove("salt".to_string(), 0))
            .expect("remove should succeed");

        assert_eq!(
            interpreter.context.mixing_bowls[0]
                .front()
                .expect("value remains")
                .amount,
            6
        );
    }

    #[test]
    fn combine_multiplies_top_value() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 3,
            measure: Measure::Dry,
        });
        interpreter.context.variables.insert(
            "flour".to_string(),
            Value {
                amount: 5,
                measure: Measure::Dry,
            },
        );

        interpreter
            .execute_instruction(&Instruction::Combine("flour".to_string(), 0))
            .expect("combine should succeed");

        assert_eq!(
            interpreter.context.mixing_bowls[0]
                .front()
                .expect("value remains")
                .amount,
            15
        );
    }

    #[test]
    fn divide_performs_integer_division() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 20,
            measure: Measure::Dry,
        });
        interpreter.context.variables.insert(
            "water".to_string(),
            Value {
                amount: 4,
                measure: Measure::Liquid,
            },
        );

        interpreter
            .execute_instruction(&Instruction::Divide("water".to_string(), 0))
            .expect("divide should succeed");

        assert_eq!(
            interpreter.context.mixing_bowls[0]
                .front()
                .expect("value remains")
                .amount,
            5
        );
    }

    #[test]
    fn divide_by_zero_returns_error() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 20,
            measure: Measure::Dry,
        });
        interpreter.context.variables.insert(
            "zero".to_string(),
            Value {
                amount: 0,
                measure: Measure::Dry,
            },
        );

        let err = interpreter
            .execute_instruction(&Instruction::Divide("zero".to_string(), 0))
            .expect_err("division by zero should error");
        assert!(matches!(
            err,
            RuntimeError::DivisionByZero {
                ingredient: _,
                bowl_index: _
            }
        ));
    }

    #[test]
    fn add_dry_pushes_sum() {
        let mut interpreter = Interpreter::new();
        interpreter.context.variables.insert(
            "flour".to_string(),
            Value {
                amount: 3,
                measure: Measure::Dry,
            },
        );
        interpreter.context.variables.insert(
            "butter".to_string(),
            Value {
                amount: 2,
                measure: Measure::Dry,
            },
        );
        interpreter.context.variables.insert(
            "water".to_string(),
            Value {
                amount: 7,
                measure: Measure::Liquid,
            },
        );

        interpreter
            .execute_instruction(&Instruction::AddDry(0))
            .expect("add dry should succeed");

        let bowl = interpreter
            .context
            .mixing_bowls
            .first()
            .expect("bowl created by add dry");
        let value = bowl.front().expect("value pushed");
        assert_eq!(value.amount, 5);
        assert!(matches!(value.measure, Measure::Dry));
    }

    fn recipe_with_auxiliary() -> Recipe {
        let aux_recipe = Recipe {
            title: "Auxiliary Sauce.".to_string(),
            ingredients: HashMap::new(),
            unset_ingredients: HashMap::new(),
            instructions: vec![Instruction::Serves(0)],
            auxiliary_recipes: HashMap::new(),
        };

        let mut aux_map = HashMap::new();
        aux_map.insert(aux_recipe.title.clone(), aux_recipe.clone());

        Recipe {
            title: "Main Dish.".to_string(),
            ingredients: HashMap::new(),
            unset_ingredients: HashMap::new(),
            instructions: vec![
                Instruction::ServeWith("auxiliary sauce".to_string()),
                Instruction::Serves(0),
            ],
            auxiliary_recipes: aux_map,
        }
    }

    #[test]
    fn liquefy_bowl_sets_all_values_to_liquid() {
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_back(Value {
            amount: 42,
            measure: Measure::Dry,
        });
        interpreter
            .execute_instruction(&Instruction::LiquefyBowl(0))
            .expect("liquefy bowl should succeed");
        assert!(interpreter.context.mixing_bowls[0]
            .iter()
            .all(|value| matches!(value.measure, Measure::Liquid)));
    }

    #[test]
    fn loop_decrements_ingredient() {
        let mut interpreter = Interpreter::new();
        interpreter.context.variables.insert(
            "counter".to_string(),
            Value {
                amount: 3,
                measure: Measure::Dry,
            },
        );

        let loop_inst = Instruction::Loop {
            condition_var: "counter".to_string(),
            verb: "Beat".to_string(),
            body: vec![],
            decrement_var: Some("counter".to_string()),
        };

        interpreter
            .execute_instruction(&loop_inst)
            .expect("loop should execute");

        let counter_value = interpreter
            .context
            .variables
            .get("counter")
            .expect("counter should exist")
            .amount;
        assert_eq!(counter_value, 0, "counter should be decremented to 0");
    }

    #[test]
    fn pour_copies_values_to_dish_in_order() {
        // Spec: pour COPIES the bowl into the dish, retaining the order; the
        // bowl keeps its contents.
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 2,
            measure: Measure::Liquid,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 1,
            measure: Measure::Liquid,
        });

        interpreter
            .execute_instruction(&Instruction::Pour(0, 0))
            .expect("pour should succeed");

        let bowl: Vec<_> = interpreter.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(bowl, vec![1, 2], "the bowl must keep its contents");
        let dish = &interpreter.context.baking_dishes[0];
        let amounts: Vec<_> = dish.iter().map(|value| value.amount).collect();
        assert_eq!(amounts, vec![1, 2]);
    }

    #[test]
    fn pour_stacks_on_top_of_existing_dish_contents() {
        // Spec: poured values go ON TOP of anything already in the dish.
        let mut interpreter = Interpreter::new();
        interpreter.ensure_bowl(0);
        interpreter.ensure_dish(0);
        interpreter.context.baking_dishes[0].push_front(Value {
            amount: 9,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 2,
            measure: Measure::Dry,
        });
        interpreter.context.mixing_bowls[0].push_front(Value {
            amount: 1,
            measure: Measure::Dry,
        });

        interpreter
            .execute_instruction(&Instruction::Pour(0, 0))
            .expect("pour should succeed");

        let dish: Vec<_> = interpreter.context.baking_dishes[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(dish, vec![1, 2, 9], "copy lands on top of the 9");
    }
}
