use std::collections::{HashMap, VecDeque};

use crate::instruction::Instruction;
use crate::types::{
    CallFrame, ExecutionContext, Measure, Recipe, Result, RuntimeError, RuntimeResult,
};

pub struct Interpreter {
    context: ExecutionContext,
    recipes: HashMap<String, Recipe>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
            recipes: HashMap::new(),
        }
    }

    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.insert(recipe.title.clone(), recipe);
    }

    pub fn run(&mut self) -> Result<()> {
        let recipe = self.recipes.values().next().ok_or(RuntimeError::NoRecipe)?;
        self.execute(recipe)?;
        Ok(())
    }

    fn execute(&mut self, recipe: &Recipe) -> RuntimeResult<()> {
        self.context = ExecutionContext::new();
        self.context.variables = recipe.ingredients.clone();

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
            Instruction::ServeWith(recipe_name) => {
                self.call_auxiliary(recipe_name)?;
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
        let aux_recipe = self
            .recipes
            .get(recipe_name)
            .ok_or_else(|| RuntimeError::UnknownRecipe(recipe_name.to_string()))?;

        let frame = CallFrame {
            variables: self.context.variables.clone(),
            mixing_bowls: self.context.mixing_bowls.clone(),
            baking_dishes: self.context.baking_dishes.clone(),
            return_address: 0,
        };
        self.context.call_stack.push(frame);

        self.execute(aux_recipe)?;

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
}
