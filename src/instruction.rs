use crate::types::Ingredient;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Instruction {
    Take(Ingredient),
    Put(Ingredient, usize),
    Fold(Ingredient, usize),
    Add(Ingredient, usize),
    Remove(Ingredient, usize),
    Combine(Ingredient, usize),
    Divide(Ingredient, usize),
    AddDry(usize),
    Liquefy(Ingredient),
    LiquefyBowl(usize),
    Stir(usize, usize), // bowl_idx, minutes
    StirIngredient(Ingredient, usize),
    Mix(usize),
    Clean(usize),
    Pour(usize, usize), // from_bowl, to_dish
    Loop {
        condition_var: Ingredient,
        verb: String,
        body: Vec<Instruction>,
        decrement_var: Option<Ingredient>,
    },
    SetAside,
    ServeWith(String), // auxiliary recipe name
    Refrigerate(Option<usize>),
    Serves(usize),
    NoOp(String),
}

#[cfg(test)]
mod tests {
    use super::Instruction;

    #[test]
    fn serve_with_stores_recipe_name() {
        let instruction = Instruction::ServeWith("Caramel Sauce".to_string());
        match instruction {
            Instruction::ServeWith(name) => assert_eq!(name, "Caramel Sauce"),
            _ => panic!("expected ServeWith variant"),
        }
    }

    #[test]
    fn loop_variant_captures_body() {
        let nested = Instruction::SetAside;
        let instruction = Instruction::Loop {
            condition_var: "batter".to_string(),
            verb: "Beat".to_string(),
            body: vec![nested.clone()],
            decrement_var: None,
        };

        match instruction {
            Instruction::Loop {
                condition_var,
                verb,
                body,
                decrement_var,
            } => {
                assert_eq!(condition_var, "batter");
                assert_eq!(verb, "Beat");
                assert_eq!(body.len(), 1);
                assert!(decrement_var.is_none());
            }
            _ => panic!("expected Loop variant"),
        }
    }

    #[test]
    fn noop_variant_stores_reason() {
        let instruction = Instruction::NoOp("Sift the flour.".to_string());
        match instruction {
            Instruction::NoOp(text) => assert_eq!(text, "Sift the flour."),
            _ => panic!("expected NoOp variant"),
        }
    }
}
