# Chef Interpreter Implementation Gaps - TODO List

## Test Results Summary

**Total: 59 spec fixture tests**
- ✅ **50 passing** (85%)
- ❌ **9 failing** (15%)
- ⏱️ **1 hanging** (timeout)

---

## CRITICAL ISSUES (Blocking Tests)

### 1. Loop with Different Condition vs Decrement Ingredient - INFINITE LOOP
**Test:** `spec_loop_different_ingredients`
**Status:** Hangs (timeout after 60+ seconds)
**Priority:** 🔴 CRITICAL

**Issue:** The loop instruction appears to enter an infinite loop when the condition ingredient is different from the decrement ingredient.

**Fixture:** `tests/fixtures/spec/loop-different-ingredients-test.chef`
```chef
Beat the condition.
Put step into mixing bowl.
Add base to mixing bowl.
Beat the step until beaten.  # Loop: condition='condition', decrement='step'
```

**Root Cause:** The loop implementation likely decrements the wrong ingredient or doesn't check the correct condition.

**Location:** `src/interpreter.rs` - Loop instruction execution

**Fix Needed:**
- Verify loop logic correctly handles `condition_var` vs `decrement_var`
- The loop should check `condition_var` for zero but decrement `decrement_var`

---

### 2. AddDry Instruction Runtime Error
**Test:** `spec_add_dry_ingredients`
**Error:** `Runtime(UndefinedIngredient)`
**Priority:** 🔴 HIGH

**Issue:** The "Add dry ingredients" instruction fails with UndefinedIngredient error.

**Fixture:** `tests/fixtures/spec/add-dry-ingredients-test.chef`
```chef
Ingredients.
2 g flour
3 g sugar
5 g salt

Method.
Add dry ingredients to the mixing bowl.
```

**Expected:** Sum all dry ingredients (2+3+5=10) and push to bowl

**Current:** Runtime error suggests ingredient lookup failure

**Location:** `src/interpreter.rs:133-146`

**Investigation Needed:**
- Check if ingredients are properly initialized in execution context
- Verify the AddDry implementation correctly accesses `context.variables`
- May be related to how dry ingredients are filtered

---

## PARSER GAPS

### 3. Optional Metadata - Cooking Time
**Tests:** `spec_cooking_time`
**Error:** `InvalidIngredient("Cooking time: 15 minutes.")`
**Priority:** 🟡 MEDIUM

**Issue:** Parser treats "Cooking time:" as an invalid ingredient declaration instead of optional metadata.

**Fixture:**
```chef
Ingredients.
0 g zero

Cooking time: 15 minutes.  # <-- Should be parsed as metadata, not ingredient

Method.
...
```

**Fix Needed:**
- Add optional "Cooking time:" parsing after ingredients section
- Should support formats: "X minutes", "X hours", "X hours and Y minutes"
- Can be ignored or stored as metadata

**Location:** `src/parser.rs` - Add between ingredients and method parsing

---

### 4. Optional Metadata - Oven Temperature
**Tests:** `spec_oven_temperature`, `spec_oven_temperature_gas_mark`
**Error:** `InvalidIngredient("Pre-heat oven to X degrees Celsius.")`
**Priority:** 🟡 MEDIUM

**Issue:** Parser treats "Pre-heat oven:" as invalid ingredient.

**Fixtures:**
```chef
Pre-heat oven to 180 degrees Celsius.
Pre-heat oven to 180 degrees Celsius (gas mark 4).
```

**Fix Needed:**
- Add optional "Pre-heat oven to:" parsing
- Support: "X degrees Celsius", "X degrees Fahrenheit", "gas mark X"
- Can be ignored or stored as metadata

**Location:** `src/parser.rs` - Add between ingredients/cooking time and method

---

### 5. Optional Ingredients Section
**Test:** `spec_stdin_echo`
**Error:** `MissingSection("Ingredients")`
**Priority:** 🟡 MEDIUM

**Issue:** Parser requires "Ingredients" section even when recipe uses only stdin.

**Fixture:**
```chef
Stdin Echo Dessert.

Method.  # <-- No Ingredients section
Take chocolate from refrigerator.
...
```

**Fix Needed:**
- Make "Ingredients." section optional
- When missing, start with empty ingredients HashMap
- "Take" instruction should create ingredients dynamically

**Location:** `src/parser.rs` - Recipe parsing logic

---

## PARSER VALIDATION GAPS (Error Detection)

### 6. Title Validation - Missing Period
**Test:** `spec_wrong_title`
**Error:** Expected parse error, but parses successfully
**Priority:** 🟢 LOW

**Issue:** Chef spec requires recipe title to end with a period. Parser doesn't validate this.

**Fixture:**
```chef
Wrong title without full stop  # <-- Should error: missing period
```

**Fix Needed:**
- Validate title ends with `.`
- Return `ParseError::InvalidTitle` if missing

**Location:** `src/parser.rs` - Title parsing

---

### 7. Title Validation - Must Start on Line 1
**Test:** `spec_wrong_title_line_start`
**Error:** Expected parse error, but parses successfully
**Priority:** 🟢 LOW

**Issue:** Chef spec requires title on first line. Parser allows blank lines before title.

**Fixture:**
```chef
# Line 1 is blank
Wrong title line.  # <-- Should error: title not on line 1
```

**Fix Needed:**
- Validate title is on line 1 (after trimming)
- Return `ParseError::InvalidTitle` if blank lines precede it

**Location:** `src/parser.rs` - Recipe parsing initialization

---

### 8. Ingredient Validation - Invalid Units
**Test:** `spec_wrong_single_dry_ingredient`
**Error:** Expected parse error, but parses successfully
**Priority:** 🟢 LOW

**Issue:** Parser accepts invalid measurement units (e.g., "tons").

**Fixture:**
```chef
42 tons salt  # <-- "tons" is not a valid Chef measurement unit
```

**Valid Units:**
- Dry: g, kg, pinch/pinches
- Liquid: ml, l, dash/dashes
- Either: cup/cups, teaspoon/teaspoons, tablespoon/tablespoons, heaped, level

**Fix Needed:**
- Add validation for measurement units
- Return `ParseError::InvalidMeasure` for unknown units

**Location:** `src/parser.rs` - Ingredient parsing

---

### 9. Ingredient Validation - Redeclared Ingredients
**Test:** `spec_redeclared_ingredient`
**Error:** Expected parse error, but parses successfully
**Priority:** 🟢 LOW

**Issue:** Parser allows same ingredient declared multiple times.

**Fixture:**
```chef
Ingredients.
5 g value
0 g value  # <-- Should error: duplicate ingredient name
```

**Current Behavior:** Last declaration wins (HashMap overwrites)

**Fix Needed:**
- Track declared ingredient names
- Return `ParseError::DuplicateIngredient` on redeclaration
- Or document as intentional behavior if Chef spec allows it

**Location:** `src/parser.rs` - Ingredients section parsing

---

## IMPLEMENTATION PRIORITY

### Phase 1: Critical Fixes (Unblock Tests)
1. **Fix infinite loop bug** (spec_loop_different_ingredients)
2. **Fix AddDry runtime error** (spec_add_dry_ingredients)

### Phase 2: Core Features (Expand Spec Support)
3. Make Ingredients section optional (spec_stdin_echo)
4. Add Cooking time metadata parsing (spec_cooking_time)
5. Add Oven temperature metadata parsing (spec_oven_temperature*)

### Phase 3: Validation (Improve Error Messages)
6. Validate invalid measurement units (spec_wrong_single_dry_ingredient)
7. Validate duplicate ingredients (spec_redeclared_ingredient)
8. Validate title format (spec_wrong_title*)

---

## FILES TO MODIFY

- **`src/interpreter.rs`** - Fix Loop and AddDry instructions
- **`src/parser.rs`** - Add metadata parsing, make Ingredients optional, add validations
- **`src/types.rs`** - Add new ParseError variants if needed

---

## TEST COVERAGE ACHIEVED

After fixing all issues, the interpreter will support:

**✅ Currently Working (50/59 tests):**
- All ingredient types and measurements (dry, liquid, either)
- Basic arithmetic (add, subtract, multiply, divide)
- Bowl operations (put, fold, clean, combine, remove)
- Stir and mix operations
- Loops (basic, nested, same ingredient, empty body, set aside)
- Output operations (pour, serve, liquefy, unicode)
- Auxiliary recipes
- Error detection (division by zero, empty bowl, undefined ingredient)

**🔧 Needs Implementation (9/59 tests):**
- Optional metadata (cooking time, oven temperature)
- Optional ingredients section
- Loop with different condition/decrement ingredients
- AddDry instruction bug
- Enhanced validation (title format, measurement units, duplicate ingredients)

---

## NOTES

- Total lines of test fixtures: 59 spec files + 3 example recipes
- Interpreter already handles majority of Chef language spec
- Most failures are edge cases and optional features
- Two critical runtime bugs need immediate attention
- Validation errors are lowest priority (nice-to-have)
