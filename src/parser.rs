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
        let mut found_first_non_empty = false;

        for (start_idx, end_idx) in line_positions {
            let line = input[start_idx..end_idx].trim();

            // For the FIRST recipe, always start from position 0 (to catch blank lines before title)
            if expect_title && !found_first_non_empty && !line.is_empty() {
                starts.push(0); // Start from beginning to preserve blank lines for validation
                found_first_non_empty = true;
                expect_title = false;
                continue;
            }

            if line.is_empty() {
                continue;
            }

            // For auxiliary recipes, look for titles after "Serves"
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
            // For the first block (idx == 0), don't trim leading whitespace to preserve blank lines for validation
            // For auxiliary recipes, trim as before
            let block = if idx == 0 {
                input[*start_idx..end_idx].trim_end()
            } else {
                input[*start_idx..end_idx].trim()
            };
            if !block.is_empty() {
                blocks.push(block);
            }
        }

        blocks
    }

    fn parse_single_recipe(block: &str) -> ParseResult<Recipe> {
        let title = Self::parse_title(block)?;

        // Validate title ends with period (Chef spec requirement)
        if !title.ends_with('.') {
            return Err(ParseError::InvalidTitle(format!(
                "Recipe title must end with a period: '{}'",
                title
            )));
        }

        let method_idx = block
            .find("Method.")
            .ok_or_else(|| ParseError::MissingSection("Method".into()))?;

        // Ingredients section is optional
        let (ingredients, unset_ingredients) =
            if let Some(ingredients_idx) = block.find("Ingredients.") {
                if method_idx <= ingredients_idx {
                    return Err(ParseError::MissingSection("Method".into()));
                }
                let ingredients_text = &block[ingredients_idx + "Ingredients.".len()..method_idx];
                Self::parse_ingredients(ingredients_text)?
            } else {
                // Check if "Ingredients" (without period) exists - this is an error
                if block.contains("Ingredients\n") || block.contains("Ingredients ") {
                    return Err(ParseError::MissingSection(
                        "Ingredients section must end with a period: 'Ingredients.'".into(),
                    ));
                }
                (HashMap::new(), HashMap::new())
            };

        let method_text = &block[method_idx + "Method.".len()..];
        let instructions = Self::parse_method(method_text)?;

        Ok(Recipe {
            title,
            ingredients,
            unset_ingredients,
            instructions,
            auxiliary_recipes: HashMap::new(),
        })
    }

    fn parse_title(block: &str) -> ParseResult<String> {
        // Chef spec requires title to be on the first line
        let first_line = block.lines().next().unwrap_or("").trim();

        if first_line.is_empty() {
            return Err(ParseError::InvalidTitle(
                "Recipe title must be on the first line (no blank lines before title)".into(),
            ));
        }

        if first_line == "Ingredients." || first_line == "Method." {
            return Err(ParseError::MissingSection("Title".into()));
        }

        Ok(first_line.to_string())
    }

    /// Parses the ingredient list into ingredients with values and ingredients
    /// declared without an initial value (the spec makes the value optional;
    /// using a valueless ingredient is a run-time error). If an ingredient is
    /// repeated, the new declaration replaces earlier ones, as the spec
    /// requires.
    #[allow(clippy::type_complexity)]
    fn parse_ingredients(
        text: &str,
    ) -> ParseResult<(HashMap<Ingredient, Value>, HashMap<Ingredient, Measure>)> {
        let mut ingredients = HashMap::new();
        let mut unset_ingredients = HashMap::new();

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip optional metadata lines (cooking time, oven temperature)
            let line_lower = line.to_lowercase();
            if line_lower.starts_with("cooking time:") || line_lower.starts_with("pre-heat oven") {
                continue;
            }

            let (quantity, rest) = match ingredient_regex().captures(line) {
                Some(caps) => (
                    Some(Self::parse_quantity(caps.name("amount").unwrap().as_str())?),
                    caps.name("rest").unwrap().as_str().trim(),
                ),
                // No initial value: the whole line is "[[measure-type] measure] name"
                None => (None, line),
            };

            let (measure_kind, ingredient) = Self::split_measure_and_ingredient(rest);
            if ingredient.is_empty() || !is_reasonable_ingredient_name(&ingredient) {
                return Err(ParseError::InvalidIngredient(line.to_string()));
            }

            // Always validate measurement units, even if no valid unit was found
            // This catches invalid units like "tons", "meters", etc.
            Self::validate_measure_line(rest)?;

            let measure = measure_kind.unwrap_or(Measure::Unspecified);
            // A repeated declaration replaces the previous one in either map.
            ingredients.remove(&ingredient);
            unset_ingredients.remove(&ingredient);
            match quantity {
                Some(amount) => {
                    ingredients.insert(ingredient, Value { amount, measure });
                }
                None => {
                    unset_ingredients.insert(ingredient, measure);
                }
            }
        }

        Ok((ingredients, unset_ingredients))
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

        let mut measure_kind = measure_tokens
            .iter()
            .rev()
            .find_map(|token| Self::measure_from_word(normalize_word(token).as_str()));

        // Per the spec, the "heaped" and "level" measure-types indicate that
        // the measure is dry, even for measures that may be either.
        let has_dry_modifier = measure_tokens
            .iter()
            .any(|token| matches!(normalize_word(token).as_str(), "heaped" | "level"));
        if has_dry_modifier {
            measure_kind = Some(Measure::Dry);
        }

        (measure_kind, ingredient.trim().to_string())
    }

    fn measure_from_word(word: &str) -> Option<Measure> {
        match word {
            // Spec: g | kg | pinch[es] always indicate dry measures.
            "g" | "kg" | "pinch" | "pinches" | "gram" | "grams" | "kilogram" | "kilograms"
            | "oz" | "ounce" | "ounces" | "lb" | "pound" | "pounds" => Some(Measure::Dry),
            // Spec: ml | l | dash[es] always indicate liquid measures.
            "ml" | "l" | "dash" | "dashes" | "liter" | "liters" | "litre" | "litres" | "cl"
            | "dl" => Some(Measure::Liquid),
            // Spec: cup[s] | teaspoon[s] | tablespoon[s] may be either dry or
            // liquid; they stay unspecified (output as numbers) unless a
            // heaped/level modifier marks them dry.
            "cup" | "cups" | "teaspoon" | "teaspoons" | "tablespoon" | "tablespoons" | "tbsp"
            | "tsp" => Some(Measure::Unspecified),
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

    fn validate_measure_line(ingredient_line: &str) -> ParseResult<()> {
        // Check if there are invalid measurement-like words
        // Common invalid units that people might try to use
        let invalid_units = [
            "ton",
            "tons",
            "metric ton",
            "tonne",
            "tonnes",
            "stone",
            "stones",
            "yard",
            "yards",
            "meter",
            "meters",
            "metre",
            "metres",
            "inch",
            "inches",
            "foot",
            "feet",
            "mile",
            "miles",
        ];

        let words: Vec<&str> = ingredient_line.split_whitespace().collect();
        for word in words.iter() {
            let normalized = normalize_word(word);
            if invalid_units.contains(&normalized.as_str()) {
                return Err(ParseError::InvalidMeasure(format!(
                    "Invalid measurement unit '{}' - not a valid Chef unit. Valid units are: g, kg, ml, l, cup(s), teaspoon(s), tablespoon(s), pinch(es), dash(es)",
                    word
                )));
            }
        }

        Ok(())
    }

    fn parse_method(text: &str) -> ParseResult<Vec<Instruction>> {
        let sentences = Self::split_sentences(text);
        let mut instructions = Vec::new();
        let mut idx = 0;

        while idx < sentences.len() {
            let (instruction, consumed) = Self::parse_statement(&sentences[idx..])?;
            instructions.push(instruction);
            idx += consumed;
        }

        Ok(instructions)
    }

    /// Parses the next method statement, returning the instruction and the
    /// number of sentences consumed. Known instructions take precedence; any
    /// other sentence of the shape "Verb [the] ingredient" starts a loop, and
    /// everything else is a parse error (the spec has no comments inside the
    /// method, so a typo'd instruction must not be silently dropped).
    fn parse_statement(sentences: &[String]) -> ParseResult<(Instruction, usize)> {
        let sentence = sentences[0].as_str();
        if let Some(instruction) = Self::parse_known_instruction(sentence)? {
            return Ok((instruction, 1));
        }
        if loop_start_regex().is_match(sentence) {
            return Self::parse_loop(sentences);
        }
        Err(ParseError::UnknownInstruction(sentence.to_string()))
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

    /// Returns true if the sentence is an "until" statement closing a loop
    /// opened with `verb`: "AnyVerb [the ingredient] until verbed". The spec
    /// says _verbed_ must match the loop verb; we use a prefix heuristic so
    /// "Bake ... until baked" matches without conjugation tables.
    fn until_matches_verb(sentence: &str, verb: &str) -> bool {
        let lower = sentence.to_lowercase();
        if let Some(until_pos) = lower.find(" until ") {
            let after_until = lower[until_pos + " until ".len()..].trim_start();
            after_until.starts_with(&verb.to_lowercase())
        } else {
            false
        }
    }

    /// Extracts the optional decrement ingredient from an "until" statement:
    /// "AnyVerb [the ingredient] until verbed". The leading verb is arbitrary
    /// and ignored; the ingredient between it and "until", if present, is
    /// decremented each time the statement executes.
    fn decrement_var_from_until(until_stmt: &str) -> Option<String> {
        let lower_stmt = until_stmt.to_lowercase();
        let until_pos = lower_stmt.find(" until")?;
        let before_until = &until_stmt[..until_pos];

        let parts: Vec<&str> = before_until.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        // Skip the first word (the verb), join the rest
        let ingredient_with_the = parts[1..].join(" ");
        let ingredient = ingredient_with_the
            .strip_prefix("the ")
            .or_else(|| ingredient_with_the.strip_prefix("The "))
            .unwrap_or(ingredient_with_the.as_str())
            .trim();

        if ingredient.is_empty() {
            None
        } else {
            Some(ingredient.to_string())
        }
    }

    /// Parses a loop opened by `sentences[0]` ("Verb [the] ingredient"). Body
    /// statements are parsed sequentially, so nested loops consume their own
    /// "until" statements before this loop looks for its own; this makes
    /// nested loops using the same verb pair up correctly.
    fn parse_loop(sentences: &[String]) -> ParseResult<(Instruction, usize)> {
        let start = sentences[0].as_str();
        let caps = loop_start_regex()
            .captures(start)
            .ok_or(ParseError::InvalidLoop)?;
        let verb = caps.name("verb").unwrap().as_str().to_string();
        let condition_var = caps.name("ingredient").unwrap().as_str().to_string();

        // Single-sentence loop with an empty body: "Verb the x until verbed."
        if Self::until_matches_verb(start, &verb) {
            let decrement_var = Self::decrement_var_from_until(start);
            return Ok((
                Instruction::Loop {
                    condition_var,
                    verb,
                    body: Vec::new(),
                    decrement_var,
                },
                1,
            ));
        }

        let mut body = Vec::new();
        let mut idx = 1;
        loop {
            if idx >= sentences.len() {
                return Err(ParseError::UnmatchedLoop(start.to_string()));
            }
            let sentence = sentences[idx].as_str();
            if Self::until_matches_verb(sentence, &verb) {
                let decrement_var = Self::decrement_var_from_until(sentence);
                return Ok((
                    Instruction::Loop {
                        condition_var,
                        verb,
                        body,
                        decrement_var,
                    },
                    idx + 1,
                ));
            }
            let (instruction, consumed) = Self::parse_statement(&sentences[idx..])?;
            body.push(instruction);
            idx += consumed;
        }
    }

    /// Parses a sentence as one of the spec's named instructions. Returns
    /// `Ok(None)` when the sentence matches no known instruction form (it may
    /// still be a loop start or end, which the caller handles).
    fn parse_known_instruction(sentence: &str) -> ParseResult<Option<Instruction>> {
        if let Some(caps) = take_regex().captures(sentence) {
            return Ok(Some(Instruction::Take(
                caps.name("ingredient").unwrap().as_str().to_string(),
            )));
        }

        if let Some(caps) = put_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::Put(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        if let Some(caps) = fold_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::Fold(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        // Check add_dry_regex BEFORE add_regex since "Add dry ingredients" matches both
        // More specific patterns must be checked first
        if let Some(caps) = add_dry_regex().captures(sentence) {
            return Ok(Some(Instruction::AddDry(ordinal_to_index(
                caps.name("bowl"),
            ))));
        }

        if let Some(caps) = add_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::Add(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        if let Some(caps) = remove_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::Remove(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        if let Some(caps) = combine_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::Combine(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        if let Some(caps) = divide_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::Divide(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        if let Some(caps) = liquefy_bowl_regex().captures(sentence) {
            return Ok(Some(Instruction::LiquefyBowl(ordinal_to_index(
                caps.name("bowl"),
            ))));
        }

        if let Some(caps) = liquefy_regex().captures(sentence) {
            return Ok(Some(Instruction::Liquefy(
                caps.name("ingredient").unwrap().as_str().to_string(),
            )));
        }

        if let Some(caps) = stir_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            let minutes = caps
                .name("minutes")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| ParseError::UnknownInstruction(sentence.to_string()))?;
            return Ok(Some(Instruction::Stir(bowl, minutes)));
        }

        if let Some(caps) = stir_ingredient_regex().captures(sentence) {
            let bowl = ordinal_to_index(caps.name("bowl"));
            return Ok(Some(Instruction::StirIngredient(
                caps.name("ingredient").unwrap().as_str().to_string(),
                bowl,
            )));
        }

        if let Some(caps) = mix_regex().captures(sentence) {
            return Ok(Some(Instruction::Mix(ordinal_to_index(caps.name("bowl")))));
        }

        if let Some(caps) = clean_regex().captures(sentence) {
            return Ok(Some(Instruction::Clean(ordinal_to_index(
                caps.name("bowl"),
            ))));
        }

        if let Some(caps) = pour_regex().captures(sentence) {
            let from = ordinal_to_index(caps.name("from"));
            let to = ordinal_to_index(caps.name("to"));
            return Ok(Some(Instruction::Pour(from, to)));
        }

        if set_aside_regex().is_match(sentence) {
            return Ok(Some(Instruction::SetAside));
        }

        if let Some(caps) = serve_with_regex().captures(sentence) {
            return Ok(Some(Instruction::ServeWith(
                caps.name("recipe").unwrap().as_str().trim().to_string(),
            )));
        }

        if let Some(caps) = refrigerate_regex().captures(sentence) {
            let time = caps
                .name("hours")
                .map(|m| m.as_str().parse::<usize>())
                .transpose()
                .map_err(|_| ParseError::UnknownInstruction(sentence.to_string()))?;
            return Ok(Some(Instruction::Refrigerate(time)));
        }

        if let Some(caps) = serves_regex().captures(sentence) {
            let count = caps
                .name("count")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| ParseError::UnknownInstruction(sentence.to_string()))?;
            return Ok(Some(Instruction::Serves(count)));
        }

        Ok(None)
    }
}

fn ingredient_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?P<amount>[-\d\s/]+)\s+(?P<rest>.+)$").unwrap())
}

fn loop_start_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?P<verb>[A-Za-z]+)(?: the)? (?P<ingredient>.+?)(?:\s+until\s|$)").unwrap()
    })
}

fn take_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^Take(?: the)? (?P<ingredient>.+) from(?: the)? refrigerator$").unwrap()
    })
}

fn put_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Put(?: the)? (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn fold_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Fold(?: the)? (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn add_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Add(?: the)? (?P<ingredient>.+?)(?:\s+to(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl)?$",
        )
        .unwrap()
    })
}

fn remove_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Remove(?: the)? (?P<ingredient>.+?)(?:\s+from(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl)?$",
        )
        .unwrap()
    })
}

fn combine_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Combine(?: the)? (?P<ingredient>.+?)(?:\s+into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl)?$",
        )
        .unwrap()
    })
}

fn divide_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Divide(?: the)? (?P<ingredient>.+?)(?:\s+into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl)?$",
        )
        .unwrap()
    })
}

fn add_dry_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Add dry ingredients(?: to(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl)?$",
        )
        .unwrap()
    })
}

fn liquefy_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Liqu[ei]fy(?: the)? (?P<ingredient>.+)$").unwrap())
}

fn liquefy_bowl_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Liqu[ei]fy(?: the)? contents of(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
        )
        .unwrap()
    })
}

fn stir_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Stir(?:(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl)? for (?P<minutes>\d+) minutes?$",
        )
        .unwrap()
    })
}

fn stir_ingredient_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^Stir(?: the)? (?P<ingredient>.+) into(?: the)?(?: (?P<bowl>\d+)(?:st|nd|rd|th))? mixing bowl$",
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

/// A valueless ingredient line is just a name (with optional measure), so
/// almost anything matches; require a sane shape so that garbage in the
/// ingredient list is still reported instead of becoming an "ingredient".
fn is_reasonable_ingredient_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_alphabetic())
        && chars.all(|c| c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' || c == '\'')
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
    fn loop_start_without_until_is_a_parse_error() {
        let source = "\
Loop Dish.

Ingredients.
1 g batter

Method.
Beat the batter.
Serves 1.";

        let error = Parser::new(source)
            .parse_recipe()
            .expect_err("a loop start without a matching 'until' must not parse");
        assert!(
            matches!(error, ParseError::UnmatchedLoop(_)),
            "expected UnmatchedLoop, got: {:?}",
            error
        );
    }

    #[test]
    fn unknown_sentence_is_a_parse_error() {
        let source = "\
Typo Dish.

Ingredients.
1 g sugar

Method.
Pur sugar into the mixing bowl.
Serves 1.";

        let error = Parser::new(source)
            .parse_recipe()
            .expect_err("a typo'd instruction must not be silently dropped");
        assert!(
            matches!(
                error,
                ParseError::UnknownInstruction(_) | ParseError::UnmatchedLoop(_)
            ),
            "expected an unknown-instruction error, got: {:?}",
            error
        );
    }

    #[test]
    fn split_recipes_handles_auxiliary_sections() {
        let source = include_str!("../tests/fixtures/fibonacci.chef");
        let blocks = Parser::split_recipes(source);
        assert_eq!(blocks.len(), 2, "blocks: {blocks:?}");
    }

    #[test]
    fn parses_simple_loop_with_decrement() {
        let source = "\
Simple Loop.

Ingredients.
3 g counter

Method.
Beat the counter until beaten.
Serves 0.";

        let recipe = Parser::new(source)
            .parse_recipe()
            .expect("simple loop should parse");

        assert_eq!(recipe.instructions.len(), 2);
        match &recipe.instructions[0] {
            Instruction::Loop {
                condition_var,
                decrement_var,
                ..
            } => {
                assert_eq!(condition_var, "counter");
                assert_eq!(
                    decrement_var.as_deref(),
                    Some("counter"),
                    "decrement_var should be 'counter'"
                );
            }
            other => panic!("expected Loop instruction, got: {:?}", other),
        }
    }

    #[test]
    fn parses_loop_with_different_ending_verb() {
        let source = "\
Loop with Different Ending.

Ingredients.
3 g ingredient

Method.
Melt the ingredient.
Put ingredient into mixing bowl.
Heat the ingredient until melted.
Serves 0.";

        let recipe = Parser::new(source)
            .parse_recipe()
            .expect("loop with different ending verb should parse");

        // Should have: Loop, Serves
        assert_eq!(recipe.instructions.len(), 2);
        match &recipe.instructions[0] {
            Instruction::Loop {
                condition_var,
                body,
                decrement_var,
                ..
            } => {
                assert_eq!(condition_var, "ingredient");
                assert_eq!(body.len(), 1, "loop body should have 1 instruction");
                assert_eq!(
                    decrement_var.as_deref(),
                    Some("ingredient"),
                    "decrement_var should be 'ingredient'"
                );
            }
            other => panic!("expected Loop instruction, got: {:?}", other),
        }
    }
}
