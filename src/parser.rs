struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn parse_recipe(&mut self) -> Result<Recipe> {
        let sections = self.split_sections();
        
        let title = self.parse_title(&sections[0])?;
        let ingredients = self.parse_ingredients(&sections)?;
        let instructions = self.parse_method(&sections)?;
        let auxiliary = self.parse_auxiliary_recipes(&sections)?;
        
        Ok(Recipe { title, ingredients, instructions, auxiliary_recipes: auxiliary })
    }
    
    fn parse_method(&mut self, sections: &[&str]) -> Result<Vec<Instruction>> {
        let method_text = self.find_section("Method")?;
        let sentences = self.split_sentences(method_text);
        
        let mut instructions = Vec::new();
        let mut idx = 0;
        
        while idx < sentences.len() {
            if self.is_loop_start(&sentences[idx]) {
                let (loop_inst, consumed) = self.parse_loop(&sentences[idx..])?;
                instructions.push(loop_inst);
                idx += consumed;
            } else {
                instructions.push(self.parse_instruction(&sentences[idx])?);
                idx += 1;
            }
        }
        
        Ok(instructions)
    }
    
    fn parse_instruction(&mut self, sentence: &str) -> Result<Instruction> {
        // Pattern matching on sentence structure
        if let Some(caps) = regex!(r"Take (\w+(?:\s+\w+)*) from refrigerator")
            .captures(sentence) {
            return Ok(Instruction::Take(caps[1].to_string()));
        }
        
        if let Some(caps) = regex!(r"Put (\w+(?:\s+\w+)*) into(?: the)?(?: (\d+)(?:st|nd|rd|th))? mixing bowl")
            .captures(sentence) {
            let bowl = caps.get(2).map_or(0, |m| m.as_str().parse::<usize>().unwrap() - 1);
            return Ok(Instruction::Put(caps[1].to_string(), bowl));
        }
        
        if let Some(caps) = regex!(r"Fold (\w+(?:\s+\w+)*) into(?: the)?(?: (\d+)(?:st|nd|rd|th))? mixing bowl")
            .captures(sentence) {
            let bowl = caps.get(2).map_or(0, |m| m.as_str().parse::<usize>().unwrap() - 1);
            return Ok(Instruction::Fold(caps[1].to_string(), bowl));
        }
        
        // Similar patterns for Add, Remove, Combine, Divide...
        // Stir patterns, Liquefy, Pour, Serve with, etc.
        
        Err(ParseError::UnknownInstruction(sentence.to_string()))
    }
    
    fn parse_loop(&mut self, sentences: &[&str]) -> Result<(Instruction, usize)> {
        // Match "Verb the ingredient"
        let loop_regex = regex!(r"^(\w+) the (\w+(?:\s+\w+)*)\.?$");
        let caps = loop_regex.captures(sentences[0])
            .ok_or(ParseError::InvalidLoop)?;
        
        let verb = caps[1].to_string();
        let var = caps[2].to_string();
        
        // Find matching "Verb ... until verbed"
        let until_pattern = format!(r"{} .* until {}ed", verb, verb);
        let until_regex = regex!(&until_pattern);
        
        let end_idx = sentences.iter()
            .position(|s| until_regex.is_match(s))
            .ok_or(ParseError::UnmatchedLoop)?;
        
        // Parse loop body
        let body = sentences[1..end_idx]
            .iter()
            .map(|s| self.parse_instruction(s))
            .collect::<Result<Vec<_>>>()?;
        
        Ok((
            Instruction::Loop { condition_var: var, verb, body },
            end_idx + 1
        ))
    }
}