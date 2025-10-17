use crate::types::Ingredient;

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
    },
    SetAside,
    ServeWith(String), // auxiliary recipe name
    Refrigerate(Option<usize>),
    Serves(usize),
}
