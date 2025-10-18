use std::collections::{HashMap, VecDeque};

use crate::instruction::Instruction;
use crate::types::{
    CallFrame, ExecutionContext, Measure, Recipe, Result, RuntimeError, RuntimeResult, Value,
};

const MAX_CALL_DEPTH: usize = 64;

pub struct Interpreter {
    context: ExecutionContext,
    recipes: HashMap<String, Recipe>,
    main_recipe_key: Option<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
            recipes: HashMap::new(),
            main_recipe_key: None,
        }
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
        self.context.mixing_bowls.clear();
        self.context.mixing_bowls.push(VecDeque::new());
        self.context.baking_dishes.clear();
        self.context.baking_dishes.push(VecDeque::new());

        for instruction in &recipe.instructions {
            self.execute_instruction(instruction)?;
        }

        Ok(())
    }

    fn execute_instruction(&mut self, inst: &Instruction) -> RuntimeResult<()> {
        match inst {
            Instruction::Put(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let value = *self
                    .context
                    .variables
                    .get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                self.context.mixing_bowls[*bowl_idx].push_front(value);
            }
            Instruction::Fold(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let value = self.context.mixing_bowls[*bowl_idx]
                    .pop_front()
                    .ok_or(RuntimeError::EmptyBowl)?;
                self.context.variables.insert(ingredient.clone(), value);
            }
            Instruction::Add(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self
                    .context
                    .variables
                    .get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                if let Some(top) = self.context.mixing_bowls[*bowl_idx].front_mut() {
                    top.amount += ing_val.amount;
                }
            }
            Instruction::Remove(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self
                    .context
                    .variables
                    .get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or(RuntimeError::EmptyBowl)?;
                top.amount -= ing_val.amount;
            }
            Instruction::Combine(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self
                    .context
                    .variables
                    .get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or(RuntimeError::EmptyBowl)?;
                top.amount *= ing_val.amount;
            }
            Instruction::Divide(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let ing_val = self
                    .context
                    .variables
                    .get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                if ing_val.amount == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                let top = self.context.mixing_bowls[*bowl_idx]
                    .front_mut()
                    .ok_or(RuntimeError::EmptyBowl)?;
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
                let depth = self
                    .context
                    .variables
                    .get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?
                    .amount;
                if depth > 0 {
                    self.stir_bowl(*bowl_idx, depth as usize);
                }
            }
            Instruction::Mix(bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let bowl = &mut self.context.mixing_bowls[*bowl_idx];
                if bowl.len() > 1 {
                    bowl.make_contiguous().reverse();
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
                let value = self
                    .context
                    .variables
                    .get_mut(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                value.measure = Measure::Liquid;
            }
            Instruction::LiquefyBowl(bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                for value in self.context.mixing_bowls[*bowl_idx].iter_mut() {
                    value.measure = Measure::Liquid;
                }
            }
            Instruction::Pour(from_idx, to_idx) => {
                self.ensure_bowl(*from_idx);
                self.ensure_dish(*to_idx);
                let drained: Vec<_> = {
                    let bowl = &mut self.context.mixing_bowls[*from_idx];
                    bowl.drain(..).collect()
                };
                let dish = &mut self.context.baking_dishes[*to_idx];
                for value in drained {
                    dish.push_back(value);
                }
            }
            Instruction::Serves(count) => {
                self.output(*count)?;
            }
            Instruction::SetAside => {}
            _ => {}
        }

        Ok(())
    }

    fn call_auxiliary(&mut self, recipe_name: &str) -> RuntimeResult<()> {
        let key = normalize_recipe_name(recipe_name);
        let aux_recipe = self
            .recipes
            .get(&key)
            .cloned()
            .ok_or_else(|| RuntimeError::UnknownRecipe(recipe_name.to_string()))?;

        if self.context.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(RuntimeError::RecursionLimit);
        }

        let frame = CallFrame {
            variables: self.context.variables.clone(),
            mixing_bowls: self.context.mixing_bowls.clone(),
            baking_dishes: self.context.baking_dishes.clone(),
            return_address: 0,
        };
        self.context.call_stack.push(frame);

        self.execute(&aux_recipe)?;

        if let Some(frame) = self.context.call_stack.pop() {
            self.context.variables = frame.variables;
            self.context.mixing_bowls = frame.mixing_bowls;
            self.context.baking_dishes = frame.baking_dishes;
        }

        Ok(())
    }

    fn output(&mut self, dish_count: usize) -> RuntimeResult<()> {
        for dish in self.context.baking_dishes.iter_mut().take(dish_count) {
            while let Some(value) = dish.pop_front() {
                match value.measure {
                    Measure::Liquid => {
                        if let Some(c) = char::from_u32(value.amount as u32) {
                            print!("{}", c);
                        }
                    }
                    _ => print!("{}", value.amount),
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
            .get(0)
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
    fn mix_reverses_bowl_contents() {
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
            .execute_instruction(&Instruction::Mix(0))
            .expect("mix should succeed");

        let amounts: Vec<_> = interpreter.context.mixing_bowls[0]
            .iter()
            .map(|value| value.amount)
            .collect();
        assert_eq!(amounts, vec![3, 2, 1]);
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
        assert!(matches!(err, RuntimeError::DivisionByZero));
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
            .get(0)
            .expect("bowl created by add dry");
        let value = bowl.front().expect("value pushed");
        assert_eq!(value.amount, 5);
        assert!(matches!(value.measure, Measure::Dry));
    }

    fn recipe_with_auxiliary() -> Recipe {
        let aux_recipe = Recipe {
            title: "Auxiliary Sauce.".to_string(),
            ingredients: HashMap::new(),
            instructions: vec![Instruction::Serves(0)],
            auxiliary_recipes: HashMap::new(),
        };

        let mut aux_map = HashMap::new();
        aux_map.insert(aux_recipe.title.clone(), aux_recipe.clone());

        Recipe {
            title: "Main Dish.".to_string(),
            ingredients: HashMap::new(),
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
    fn pour_moves_values_to_dish_in_order() {
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

        assert!(interpreter.context.mixing_bowls[0].is_empty());
        let dish = &interpreter.context.baking_dishes[0];
        let amounts: Vec<_> = dish.iter().map(|value| value.amount).collect();
        assert_eq!(amounts, vec![1, 2]);
    }
}
