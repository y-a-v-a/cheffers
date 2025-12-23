# Chef Interpreter Error Enhancement Plan

## Goal
Transform the Chef interpreter's basic error messages into rich, helpful error output similar to Rust's compiler errors, with natural language explanations and references to the Chef language specification.

## Current State Analysis

### Existing Errors
The interpreter currently has these error types:

**RuntimeError:**
- `UndefinedIngredient` - "ingredient is not defined"
- `EmptyBowl` - "mixing bowl is empty"
- `UnknownRecipe(String)` - "recipe '{0}' is not known"
- `NoRecipe` - "no recipe available to execute"
- `RecursionLimit` - "call stack limit reached"
- `DivisionByZero` - "division by zero"
- `EarlyTermination` - "early termination"
- `BreakLoop` - "break loop"

**ParseError:**
- `MissingSection(String)` - "missing section: {0}"
- `InvalidIngredient(String)` - "invalid ingredient line: {0}"
- `InvalidQuantity(String)` - "invalid quantity: {0}"
- `UnknownInstruction(String)` - "unknown instruction: {0}"
- `InvalidLoop` - "invalid loop structure"
- `UnmatchedLoop` - "unmatched loop markers"
- `InvalidTitle(String)` - "invalid title: {0}"
- `InvalidMeasure(String)` - "invalid measure: {0}"
- `DuplicateIngredient(String)` - "duplicate ingredient: {0}"

### Current Limitations
1. No contextual information (line numbers, ingredient names, bowl indices)
2. No helpful suggestions or explanations
3. No references to language specification
4. Basic error messages without formatting or structure

## Implementation Plan

### Phase 1: Enhanced Error Context System

#### 1.1 Create Error Context Types
Create `src/error_context.rs` with:
- `SourceLocation` struct (line number, column, snippet)
- `ErrorContext` enum for different error contexts
- Helper methods to capture context during interpretation

#### 1.2 Extend Error Types
Enhance error enums in `src/types.rs`:
- Add context fields to error variants
- Store ingredient names, bowl indices, recipe names, etc.
- Add source location information where applicable

### Phase 2: Rich Error Formatting

#### 2.1 Create Error Formatter Module
Create `src/error_formatter.rs` with:
- `ErrorFormatter` struct
- Methods to format each error type with:
  - Clear error title
  - Contextual information
  - Language spec reference
  - Helpful suggestions
  - Visual formatting (colors via `colored` crate)

#### 2.2 Enhanced Error Messages
For each error type, create:
- **Main Message**: Clear description of what went wrong
- **Context**: Where it happened and what was being attempted
- **Spec Reference**: Relevant excerpt from Chef.md
- **Suggestion**: How to fix the issue
- **Example**: Optional code example if helpful

### Phase 3: Update Error Sites

#### 3.1 Interpreter Error Sites
Update all error returns in `src/interpreter.rs`:
- Capture ingredient names for `UndefinedIngredient`
- Capture bowl/dish indices for `EmptyBowl`
- Add call stack information for `RecursionLimit`
- Include divisor ingredient name for `DivisionByZero`

#### 3.2 Parser Error Sites
Update error returns in `src/parser.rs`:
- Add line numbers and text snippets
- Include attempted instruction text
- Capture context around parsing failures

### Phase 4: Main Error Display

#### 4.1 Update main.rs
Modify error handling in `main.rs`:
- Catch errors at the top level
- Use ErrorFormatter to display rich error messages
- Return appropriate exit codes

### Phase 5: Error Test Suite

#### 5.1 Create Error Test Directory
Create `tests/errors/` with failing Chef programs:
- `undefined-ingredient.chef` - uses undefined ingredient
- `empty-bowl.chef` - folds from empty bowl
- `division-by-zero.chef` - divides by zero ingredient
- `recursion-limit.chef` - exceeds recursion depth
- `unknown-recipe.chef` - serves with non-existent recipe
- `unmatched-loop.chef` - loop structure errors
- `invalid-ingredient.chef` - parsing errors

#### 5.2 Create Error Integration Tests
Create `tests/error_messages.rs`:
- Test that each error produces expected output format
- Verify spec references are included
- Check that suggestions are present

### Phase 6: Quality Assurance

#### 6.1 Code Quality
- Run `cargo fmt` after each major change
- Run `cargo clippy` and fix warnings
- Add documentation comments

#### 6.2 Testing
- Test each error scenario manually
- Run full test suite
- Verify error output is helpful and accurate

## Example Enhanced Error Output

### Before:
```
Error: ingredient is not defined
```

### After:
```
error: undefined ingredient 'sugar'
  --> fibonacci.chef:15
   |
15 | Add sugar to mixing bowl.
   |     ^^^^^ ingredient not found in ingredients list
   |
   = help: This instruction references an ingredient that hasn't been declared

   According to the Chef language specification:
   "Attempting to use an ingredient without a defined value is a run-time error."

   Ingredients must be declared in the ingredients section at the top of your
   recipe before they can be used in the method.

   suggestion: Add 'sugar' to your ingredients list:

   Ingredients.
   100 g sugar
   ...
```

### Recursion Error Example:
```
error: maximum recursion depth exceeded
  --> fibonacci.chef:25
   |
25 | Serve with fibonacci numbers.
   |            ^^^^^^^^^^^^^^^^^^ recursive call depth limit reached
   |
   = note: Recursion depth limited to 64 calls

   According to the Chef language specification:
   "Serve with auxiliary-recipe. This invokes a sous-chef to immediately
   prepare the named auxiliary-recipe."

   Your recipe is calling auxiliary recipes recursively too deeply. This
   typically happens when:
   - A recipe calls itself without a proper termination condition
   - Multiple recipes form a circular calling pattern
   - Loop conditions never reach zero

   Current call stack:
     1. fibonacci numbers (main)
     2. fibonacci numbers (auxiliary, depth: 1)
     3. fibonacci numbers (auxiliary, depth: 2)
     ...
    64. fibonacci numbers (auxiliary, depth: 63)

   suggestion: Check your loop conditions and ensure recursive calls have
   a base case that terminates.
```

## Implementation Order

1. ✓ Analyze current codebase (COMPLETE)
2. Create error_context.rs module
3. Create error_formatter.rs module
4. Extend RuntimeError variants with context
5. Update interpreter.rs error sites
6. Extend ParseError variants with context
7. Update parser.rs error sites
8. Update main.rs error display
9. Create tests/errors/ directory and test programs
10. Create error message integration tests
11. Run cargo fmt and cargo clippy throughout
12. Manual testing of all error scenarios
13. Documentation and final polish

## Dependencies

May need to add:
- `colored` or `owo-colors` for terminal colors
- `annotate-snippets` for Rust-style error formatting (optional)

## Success Criteria

- All error messages include contextual information
- Each error references the relevant language specification
- Error messages provide actionable suggestions
- Visual formatting makes errors easy to read
- Test suite validates error message quality
- cargo fmt and cargo clippy pass without warnings
