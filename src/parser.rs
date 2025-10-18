use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;

use crate::instruction::Instruction;
use crate::types::{Ingredient, Measure, ParseError, ParseResult, Recipe, Value};

pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub fn parse_recipe(&self) -> ParseResult<Recipe> {
        let blocks = Self::split_recipes(self.input);
        if blocks.is_empty() {
            return Err(ParseError::MissingSection("Recipe".into()));
        }

        let mut parsed = Vec::with_capacity(blocks.len());
        for block in blocks {
            parsed.push(Self::parse_single_recipe(block)?);
        }

        let mut recipes = parsed.into_iter();
        let mut main = recipes
            .next()
            .ok_or_else(|| ParseError::MissingSection("Recipe".into()))?;
        let auxiliary = recipes
            .map(|recipe| (recipe.title.clone(), recipe))
            .collect::<HashMap<_, _>>();
        main.auxiliary_recipes = auxiliary;

        Ok(main)
    }

    fn split_recipes(input: &str) -> Vec<&str> {
        let mut line_positions = Vec::new();
        let mut offset = 0usize;
        for segment in input.split_inclusive('\n') {
            let len = segment.len();
            let end = offset + len;
            let line_end = if segment.ends_with('\n') {
                end - 1
            } else {
                end
            };
            line_positions.push((offset, line_end));
            offset = end;
        }

        let mut starts = Vec::new();
        let mut expect_title = true;
        for (start_idx, end_idx) in line_positions {
            let line = input[start_idx..end_idx].trim();
            if line.is_empty() {
                continue;
            }

            if expect_title && line.ends_with('.') {
                starts.push(start_idx);
                expect_title = false;
                continue;
            }

            if line.to_lowercase().starts_with("serves ") {
                expect_title = true;
            }
        }

        if starts.is_empty() {
            return vec![input.trim()];
        }

        let mut blocks = Vec::new();
        let total_len = input.len();
        for (idx, start_idx) in starts.iter().enumerate() {
            let end_idx = if idx + 1 < starts.len() {
                starts[idx + 1]
            } else {
                total_len
            };
            let block = input[*start_idx..end_idx].trim();
            if !block.is_empty() {
                blocks.push(block);
            }
        }

        blocks
    }

    fn parse_single_recipe(block: &str) -> ParseResult<Recipe> {
        let title = Self::parse_title(block)?;
        let ingredients_idx = block
            .find("Ingredients.")
            .ok_or_else(|| ParseError::MissingSection("Ingredients".into()))?;
        let method_idx = block
            .find("Method.")
            .ok_or_else(|| ParseError::MissingSection("Method".into()))?;

        if method_idx <= ingredients_idx {
            return Err(ParseError::MissingSection("Method".into()));
        }

        let ingredients_text = &block[ingredients_idx + "Ingredients.".len()..method_idx];
        let method_text = &block[method_idx + "Method.".len()..];

        let ingredients = Self::parse_ingredients(ingredients_text)?;
        let instructions = Self::parse_method(method_text)?;

        Ok(Recipe {
            title,
            ingredients,
            instructions,
            auxiliary_recipes: HashMap::new(),
        })
    }

    fn parse_title(block: &str) -> ParseResult<String> {
        block
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && *line != "Ingredients." && *line != "Method.")
            .map(|line| line.to_string())
            .ok_or_else(|| ParseError::MissingSection("Title".into()))
    }

    fn parse_ingredients(text: &str) -> ParseResult<HashMap<Ingredient, Value>> {
        let mut ingredients = HashMap::new();

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            let caps = ingredient_regex()
                .captures(line)
                .ok_or_else(|| ParseError::InvalidIngredient(line.to_string()))?;

            let quantity = Self::parse_quantity(caps.name("amount").unwrap().as_str())?;
            let rest = caps.name("rest").unwrap().as_str().trim();

            let (measure_kind, ingredient) = Self::split_measure_and_ingredient(rest);
            if ingredient.is_empty() {
                return Err(ParseError::InvalidIngredient(line.to_string()));
            }
            ingredients.insert(
                ingredient,
                Value {
                    amount: quantity,
                    measure: measure_kind.unwrap_or(Measure::Unspecified),
                },
            );
        }

        Ok(ingredients)
    }

    fn parse_quantity(raw: &str) -> ParseResult<i64> {
        let mut total = 0.0_f64;

        for token in raw.split_whitespace() {
            if let Some((numerator, denominator)) = token.split_once('/') {
                let num: f64 = numerator
                    .parse()
                    .map_err(|_| ParseError::InvalidQuantity(raw.to_string()))?;
                let denom: f64 = denominator
                    .parse()
                    .map_err(|_| ParseError::InvalidQuantity(raw.to_string()))?;
                if denom.abs() < f64::EPSILON {
                    return Err(ParseError::InvalidQuantity(raw.to_string()));
                }
                total += num / denom;
            } else {
                let value: f64 = token
                    .parse()
                    .map_err(|_| ParseError::InvalidQuantity(raw.to_string()))?;
                total += value;
            }
        }

        Ok(total.round() as i64)
    }

    fn split_measure_and_ingredient(rest: &str) -> (Option<Measure>, String) {
        let mut tokens = rest.split_whitespace().peekable();
        let mut measure_tokens = Vec::new();
        let mut consumed = 0;

        while let Some(&token) = tokens.peek() {
            let normalized = normalize_word(token);
            if Self::is_measure_modifier(&normalized) {
                measure_tokens.push(token);
                tokens.next();
                consumed += 1;
                continue;
            }

            if Self::is_measure_word(&normalized) {
                measure_tokens.push(token);
                tokens.next();
                consumed += 1;
                continue;
            }

            break;
        }

        let ingredient = rest
            .split_whitespace()
            .skip(consumed)
            .collect::<Vec<_>>()
            .join(" ");

        let measure_kind = measure_tokens
            .iter()
            .rev()
            .find_map(|token| Self::measure_from_word(normalize_word(token).as_str()));

        (measure_kind, ingredient.trim().to_string())
    }

    fn measure_from_word(word: &str) -> Option<Measure> {
        match word {
            "cup" | "cups" | "teaspoon" | "teaspoons" | "tablespoon" | "tablespoons" | "tbsp"
            | "tsp" | "pinch" | "pinches" | "dash" | "dashes" | "g" | "kg" | "gram" | "grams"
            | "kilogram" | "kilograms" | "oz" | "ounce" | "ounces" | "lb" | "pound" | "pounds" => {
                Some(Measure::Dry)
            }
            "ml" | "l" | "liter" | "liters" | "litre" | "litres" | "cl" | "dl" => {
                Some(Measure::Liquid)
            }
            _ => None,
        }
    }

    fn is_measure_word(word: &str) -> bool {
        Self::measure_from_word(word).is_some()
    }

    fn is_measure_modifier(word: &str) -> bool {
        matches!(
            word,
            "heaped" | "level" | "rounded" | "flat" | "large" | "small" | "fluid"
        )
    }

    fn parse_method(text: &str) -> ParseResult<Vec<Instruction>> {
        let sentences = Self::split_sentences(text);
        let mut instructions = Vec::new();
        let mut idx = 0;

        while idx < sentences.len() {
            let sentence = sentences[idx].as_str();
            if Self::is_loop_start(sentence) {
                let (loop_inst, consumed) = Self::parse_loop(&sentences[idx..])?;
                instructions.push(loop_inst);
                idx += consumed;
            } else {
                instructions.push(Self::parse_instruction(sentence)?);
                idx += 1;
            }
        }

        Ok(instructions)
    }

    fn split_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            match ch {
                '.' | '!' | '?' => {
                    if !current.trim().is_empty() {
                        sentences.push(current.trim().to_string());
                    }
                    current.clear();
                }
                '\n' => {
                    if !current.ends_with(' ') && !current.is_empty() {
                        current.push(' ');
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            sentences.push(current.trim().to_string());
        }

        sentences
    }

    fn is_loop_start(_sentence: &str) -> bool {
        false
    }

    fn parse_loop(sentences: &[String]) -> ParseResult<(Instruction, usize)> {
        let caps = loop_start_regex()
            .captures(&sentences[0])
            .ok_or(ParseError::InvalidLoop)?;
        let verb = caps.name("verb").unwrap().as_str().to_string();
        let condition_var = caps.name("ingredient").unwrap().as_str().to_string();

        let until_regex =
            Regex::new(&format!(r"(?i)^{}\b.*\buntil\b", regex::escape(&verb))).unwrap();

        let end_idx = sentences
            .iter()
            .position(|s| until_regex.is_match(s))
            .ok_or(ParseError::UnmatchedLoop)?;

        if end_idx == 0 {
            return Err(ParseError::InvalidLoop);
        }

        let mut body = Vec::new();
        let mut idx = 1;
        while idx < end_idx {
            let sentence = sentences[idx].as_str();
            if Self::is_loop_start(sentence) {
                let (nested, consumed) = Self::parse_loop(&sentences[idx..])?;
                body.push(nested);
                idx += consumed;
            } else {
                body.push(Self::parse_instruction(sentence)?);
                idx += 1;
            }
        }

        Ok((
            Instruction::Loop {
                condition_var,
                verb,
                body,
            },
            end_idx + 1,
        ))
    }

    fn parse_instruction(sentence: &str) -> ParseResult<Instruction> {
        if let Some(caps) = take_regex().captures(sentence) {
            return Ok(Instruction::Take(
                caps.name("ingredient").unwrap().as_str().to_string(),
            ));
        }

        if let Some(caps) = put_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::Put(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = fold_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::Fold(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = add_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::Add(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = remove_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::Remove(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = combine_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::Combine(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = divide_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::Divide(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = add_dry_regex().captures(sentence) {
            return Ok(Instruction::AddDry(ordinal_to_index(caps.name("bowl"))));
        }

        if let Some(caps) = liquefy_bowl_regex().captures(sentence) {
            return Ok(Instruction::LiquefyBowl(ordinal_to_index(
                caps.name("bowl"),
            )));
        }

        if let Some(caps) = liquefy_regex().captures(sentence) {
            return Ok(Instruction::Liquefy(
                caps.name("ingredient").unwrap().as_str().to_string(),
            ));
        }

        if let Some(caps) = stir_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            let minutes = caps
                .name("minutes")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| ParseError::UnknownInstruction(sentence.to_string()))?;
            return Ok(Instruction::Stir(bowl, minutes));
        }

        if let Some(caps) = stir_ingredient_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Instruction::StirIngredient(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            ));
        }

        if let Some(caps) = mix_regex().captures(sentence) {
            return Ok(Instruction::Mix(ordinal_to_index(caps.name("bowl"))));
        }

        if let Some(caps) = clean_regex().captures(sentence) {
            return Ok(Instruction::Clean(ordinal_to_index(caps.name("bowl"))));
        }

        if let Some(caps) = pour_regex().captures(sentence) {
            let from = ordinal_to_index(caps.name("from"));
            let to = ordinal_to_index(caps.name("to"));
            return Ok(Instruction::Pour(from, to));
        }

        if set_aside_regex().is_match(sentence) {
            return Ok(Instruction::SetAside);
        }

        if let Some(caps) = serve_with_regex().captures(sentence) {
            return Ok(Instruction::ServeWith(
                caps.name("recipe").unwrap().as_str().trim().to_string(),
            ));
        }

        if let Some(caps) = refrigerate_regex().captures(sentence) {
            let time = caps
                .name("hours")
                .map(|m| m.as_str().parse::<usize>())
                .transpose()
                .map_err(|_| ParseError::UnknownInstruction(sentence.to_string()))?;
            return Ok(Instruction::Refrigerate(time));
        }

        if let Some(caps) = serves_regex().captures(sentence) {
            let count = caps
                .name("count")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| ParseError::UnknownInstruction(sentence.to_string()))?;
            return Ok(Instruction::Serves(count));
        }

        Ok(Instruction::NoOp(sentence.to_string()))
    }
}

fn ingredient_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?P<amount>[-\d\s/]+)\s+(?P<rest>.+)$").unwrap())
}

fn loop_start_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?P<verb>[A-Za-z]+) the (?P<ingredient>.+)$").unwrap())
}

fn take_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^Take (?P<ingredient>.+) from(?: the)? refrigerator$").unwrap()
    })
}

fn put_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Put (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn fold_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Fold (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn add_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Add (?P<ingredient>.+) to(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn remove_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Remove (?P<ingredient>.+) from(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn combine_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Combine (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn divide_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Divide (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn add_dry_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Add dry ingredients to(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn liquefy_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Liquefy(?: the)? (?P<ingredient>.+)$").unwrap())
}

fn liquefy_bowl_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Liquefy(?: the)? contents of(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn stir_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Stir(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl for (?P<minutes>\d+) minutes$",
        )
        .unwrap()
    })
}

fn stir_ingredient_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Stir (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn mix_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Mix(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl well$|^Mix well$",
        )
        .unwrap()
    })
}

fn clean_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^Clean(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$").unwrap()
    })
}

fn pour_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Pour contents of(?: the)?(?: (?P<from>\d+)(?:st|nd|rd|th))? mixing bowl into(?: the)?(?: (?P<to>\d+)(?:st|nd|rd|th))? baking dish(?:es)?$",
        )
        .unwrap()
    })
}

fn set_aside_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Set aside$").unwrap())
}

fn serve_with_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Serve with (?P<recipe>.+)$").unwrap())
}

fn refrigerate_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Refrigerate(?: for (?P<hours>\d+))?(?: hours?)?$").unwrap())
}

fn serves_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Serves (?P<count>\d+)$").unwrap())
}

fn ordinal_to_index(value: Option<regex::Match<'_>>) -> usize {
    value
        .and_then(|m| m.as_str().parse::<usize>().ok())
        .map(|n| n.saturating_sub(1))
        .unwrap_or(0)
}

fn normalize_word(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_recipe() {
        let source = "\
Test Dish.

Ingredients.
1 g sugar

Method.
Put sugar into the mixing bowl.
Serves 1.";

        let recipe = Parser::new(source)
            .parse_recipe()
            .expect("recipe should parse");
        assert_eq!(recipe.title, "Test Dish.");
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(recipe.instructions.len(), 2);
    }

    #[test]
    fn loop_like_sentence_is_treated_as_noop() {
        let source = "\
Loop Dish.

Ingredients.
1 g batter

Method.
Stir the batter.
Serves 1.";

        let recipe = Parser::new(source)
            .parse_recipe()
            .expect("loop-like sentence should parse");
        assert_eq!(recipe.instructions.len(), 2);
        matches!(
            recipe.instructions[0],
            Instruction::NoOp(ref text) if text == "Stir the batter"
        );
    }

    #[test]
    fn split_recipes_handles_auxiliary_sections() {
        let source = include_str!("../tests/fixtures/fibonacci.chef");
        let blocks = Parser::split_recipes(source);
        assert_eq!(blocks.len(), 2, "blocks: {blocks:?}");
    }
}
