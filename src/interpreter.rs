struct Interpreter {
    context: ExecutionContext,
    recipes: HashMap<String, Recipe>,
}

impl Interpreter {
    fn execute(&mut self, recipe: &Recipe) -> Result<()> {
        // Initialize context with ingredient values
        self.context.variables = recipe.ingredients.clone();
        
        for instruction in &recipe.instructions {
            self.execute_instruction(instruction)?;
        }
        
        Ok(())
    }
    
    fn execute_instruction(&mut self, inst: &Instruction) -> Result<ControlFlow> {
        match inst {
            Instruction::Put(ingredient, bowl_idx) => {
                self.ensure_bowl(*bowl_idx);
                let value = *self.context.variables.get(ingredient)
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
                let ing_val = self.context.variables.get(ingredient)
                    .ok_or(RuntimeError::UndefinedIngredient)?;
                if let Some(top) = self.context.mixing_bowls[*bowl_idx].front_mut() {
                    top.amount += ing_val.amount;
                }
            }
            
            Instruction::Stir(bowl_idx, minutes) => {
                self.ensure_bowl(*bowl_idx);
                self.context.mixing_bowls[*bowl_idx].rotate_left(*minutes % 
                    self.context.mixing_bowls[*bowl_idx].len().max(1));
            }
            
            Instruction::Loop { condition_var, body, .. } => {
                while self.context.variables.get(condition_var)
                    .map_or(false, |v| v.amount != 0) {
                    
                    for inst in body {
                        match self.execute_instruction(inst)? {
                            ControlFlow::Break => return Ok(ControlFlow::Continue),
                            _ => {}
                        }
                    }
                    
                    // Decrement loop variable
                    if let Some(var) = self.context.variables.get_mut(condition_var) {
                        var.amount -= 1;
                    }
                }
            }
            
            Instruction::SetAside => return Ok(ControlFlow::Break),
            
            Instruction::ServeWith(recipe_name) => {
                self.call_auxiliary(recipe_name)?;
            }
            
            Instruction::Serves(count) => {
                self.output(*count)?;
            }
            
            // Other instructions...
            _ => {}
        }
        
        Ok(ControlFlow::Continue)
    }
    
    fn call_auxiliary(&mut self, recipe_name: &str) -> Result<()> {
        // Save current state
        let frame = CallFrame {
            variables: self.context.variables.clone(),
            mixing_bowls: self.context.mixing_bowls.clone(),
            baking_dishes: self.context.baking_dishes.clone(),
            return_address: 0,
        };
        self.context.call_stack.push(frame);
        
        // Execute auxiliary
        let aux_recipe = self.recipes.get(recipe_name)
            .ok_or(RuntimeError::UnknownRecipe)?;
        self.execute(aux_recipe)?;
        
        // Return first mixing bowl to caller's first bowl
        let returned_bowl = self.context.mixing_bowls[0].clone();
        
        // Restore caller state
        let frame = self.context.call_stack.pop().unwrap();
        self.context.variables = frame.variables;
        self.context.mixing_bowls = frame.mixing_bowls;
        self.context.baking_dishes = frame.baking_dishes;
        
        // Merge returned bowl
        self.context.mixing_bowls[0] = returned_bowl;
        
        Ok(())
    }
    
    fn output(&mut self, dish_count: usize) -> Result<()> {
        for i in 0..dish_count.min(self.context.baking_dishes.len()) {
            while let Some(value) = self.context.baking_dishes[i].pop_front() {
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
}

enum ControlFlow {
    Continue,
    Break,
}