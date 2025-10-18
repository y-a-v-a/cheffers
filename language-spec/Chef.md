DM's Esoteric Programming Languages - Chef  

# Chef

- - -

## Introduction

Chef is a programming language in which programs look like recipes.

NEW: Additional syntax specifications added 17 July, 2003, marked in red. Fixed spelling of "liquefy" keyword.

## Design Principles

*   Program recipes should not only generate valid output, but be easy to prepare and delicious.
*   Recipes may appeal to cooks with different budgets.
*   Recipes will be metric, but may use traditional cooking measures such as cups and tablespoons.

## Language Concepts

### Ingredients

All recipes have ingredients! The ingredients hold individual data values. All ingredients are numerical, though they can be interpreted as Unicode for I/O purposes. Liquid ingredients will be output as Unicode characters, while dry or unspecified ingredients will be output as numbers.

### Mixing Bowls and Baking Dishes

Chef has access to an unlimited supply of mixing bowls and baking dishes. These can contain ingredient values. The ingredients in a mixing bowl or baking dish are ordered, like a stack of pancakes. New ingredients are placed on top, and if values are removed they are removed from the top. Note that if the value of an ingredient changes, the value in the mixing bowl or baking dish does not. The values in the mixing bowls and baking dishes also retain their dry or liquid designations.

Multiple mixing bowls and baking dishes are referred to by an ordinal identifier - "the 2nd mixing bowl". If no identifier is used, the recipe only has one of the relevant utensil. Ordinal identifiers must be digits followed by "st", "nd", "rd" or "th", not words.

## Syntax Elements

The following items appear in a Chef recipe. Some are optional. Items must appear in the order shown below, with a blank line (two newlines) between each item.

### Recipe Title

The recipe title describes in a few words what the program does. For example: "Hello World Souffle", or "Fibonacci Numbers with Caramel Sauce". The recipe title is always the first line of a Chef recipe, and is followed by a full stop.

_recipe-title_.

### Comments

Comments are placed in a free-form paragraph after the recipe title. Comments are optional.

### Ingredient List

The next item in a Chef recipe is the ingredient list. This lists the ingredients to be used by the program. The syntax is

Ingredients.  
_\[initial-value\] \[\[measure-type\] measure\] ingredient-name_  
\[_further ingredients_\]

Ingredients are listed one per line. The _intial-value_ is a number, and is optional. Attempting to use an ingredient without a defined value is a run-time error. The optional _measure_ can be any of the following:

*   `g | kg | pinch[es]` : These always indicate dry measures.
*   `ml | l | dash[es]` : These always indicate liquid measures.
*   `cup[s] | teaspoon[s] | tablespoon[s]` : These indicate measures which may be either dry or liquid.

The optional _measure-type_ may be any of the following:

*   `heaped | level` : These indicate that the measure is dry.

The _ingredient-name_ may be anything reasonable, and may include space characters. The ingredient list is optional. If present, it declares ingredients with the given initial values and measures. If an ingredient is repeated, the new vaue is used and previous values for that ingredient are ignored.

### Cooking Time

Cooking time: _time_ (hour\[s\] | minute\[s\]).

The cooking time statement is optional. The time is a number.

### Oven Temperature

Pre-heat oven to _temperature_ degrees Celsius \[(gas mark _mark_)\].

Some recipes require baking. If so, there will be an oven temperature statement. This is optional. The _temperature_ and _mark_ are numbers.

### Method

Method.  
_method statements_

The method contains the actual recipe instructions. These are written in sentences. Line breaks are ignored in the method of a recipe. Valid method instructions are:

*   `Take _ingredient_ from refrigerator.`  
    This reads a numeric value from STDIN into the _ingredient_ named, overwriting any previous value.
*   `Put _ingredient_ into [_nth_] mixing bowl.`  
    This puts the _ingredient_ into the _nth_ mixing bowl.
*   `Fold _ingredient_ into [_nth_] mixing bowl.`  
    This removes the top value from the _nth_ mixing bowl and places it in the _ingredient_.
*   `Add _ingredient_ [to [_nth_] mixing bowl].`  
    This adds the value of _ingredient_ to the value of the ingredient on top of the _nth_ mixing bowl and stores the result in the _nth_ mixing bowl.
*   `Remove _ingredient_ [from [_nth_] mixing bowl].`  
    This subtracts the value of _ingredient_ from the value of the ingredient on top of the _nth_ mixing bowl and stores the result in the _nth_ mixing bowl.
*   `Combine _ingredient_ [into [_nth_] mixing bowl].`  
    This multiplies the value of _ingredient_ by the value of the ingredient on top of the _nth_ mixing bowl and stores the result in the _nth_ mixing bowl.
*   `Divide _ingredient_ [into [_nth_] mixing bowl].`  
    This divides the value of _ingredient_ into the value of the ingredient on top of the _nth_ mixing bowl and stores the result in the _nth_ mixing bowl.
*   `Add dry ingredients [to [_nth_] mixing bowl].`  
    This adds the values of all the dry ingredients together and places the result into the _nth_ mixing bowl.
*   `Liquefy | Liquify _ingredient_.`  
    This turns the ingredient into a liquid, i.e. a Unicode character for output purposes. (Note: The original specification used the word "Liquify", which is a spelling error. "Liquify" is deprecated. Use "Liquefy" in all new code.)
*   `Liquefy | Liquify contents of the [_nth_] mixing bowl.`  
    This turns all the ingredients in the _nth_ mixing bowl into a liquid, i.e. a Unicode characters for output purposes.
*   `Stir [the [_nth_] mixing bowl] for _number_ minutes.`  
    This "rolls" the top _number_ ingredients in the _nth_ mixing bowl, such that the top ingredient goes down that number of ingredients and all ingredients above it rise one place. If there are not that many ingredients in the bowl, the top ingredient goes to tbe bottom of the bowl and all the others rise one place.
*   `Stir _ingredient_ into the [_nth_] mixing bowl.`  
    This rolls the number of ingredients in the _nth_ mixing bowl equal to the value of _ingredient_, such that the top ingredient goes down that number of ingredients and all ingredients above it rise one place. If there are not that many ingredients in the bowl, the top ingredient goes to the bottom of the bowl and all the others rise one place.
*   `Mix [the [_nth_] mixing bowl] well.`  
    This randomises the order of the ingredients in the _nth_ mixing bowl.
*   `Clean [_nth_] mixing bowl.`  
    This removes all the ingredients from the _nth_ mixing bowl.
*   `Pour contents of the [_nth_] mixing bowl into the [_pth_] baking dish.`  
    This copies all the ingredients from the _nth_ mixing bowl to the _pth_ baking dish, retaining the order and putting them on top of anything already in the baking dish.
*   `_Verb_ the _ingredient_.`  
    This marks the beginning of a loop. It must appear as a matched pair with the following statement. The loop executes as follows: The value of _ingredient_ is checked. If it is non-zero, the body of the loop executes until it reaches the "until" statement. The value of _ingredient_ is rechecked. If it is non-zero, the loop executes again. If at any check the value of _ingredient_ is zero, the loop exits and execution continues at the statement after the "until". Loops may be nested.
*   `_Verb_ [the _ingredient_] until _verbed_.`  
    This marks the end of a loop. It must appear as a matched pair with the above statement. _verbed_ must match the _Verb_ in the matching loop start statement. The _Verb_ in this statement may be arbitrary and is ignored. If the _ingredient_ appears in this statement, its value is decremented by 1 when this statement executes. The _ingredient_ does not have to match the _ingredient_ in the matching loop start statement.
*   `Set aside.`  
    This causes execution of the innermost loop in which it occurs to end immediately and execution to continue at the statement after the "until".
*   `Serve with _auxiliary-recipe_.`  
    This invokes a sous-chef to immediately prepare the named _auxiliary-recipe_. The calling chef waits until the sous-chef is finished before continuing. See the section on auxiliary recipes below.
*   `Refrigerate [for _number_ hours].`  
    This causes execution of the recipe in which it appears to end immediately. If in an auxiliary recipe, the auxiliary recipe ends and the sous-chef's first mixing bowl is passed back to the calling chef as normal. If a _number_ of hours is specified, the recipe will print out its first _number_ baking dishes (see the Serves statement below) before ending.

### Serves

The final statement in a Chef recipe is a statement of how many people it serves.

Serves _number-of-diners_.

This statement writes to STDOUT the contents of the first _number-of-diners_ baking dishes. It begins with the 1st baking dish, removing values from the top one by one and printing them until the dish is empty, then progresses to the next dish, until all the dishes have been printed. The serves statement is optional, but is required if the recipe is to output anything!

### Auxiliary Recipes

These are small recipes which are needed to produce specialised ingredients for the main recipe (such as sauces). They are listed after the main recipe. Auxiliary recipes are made by sous-chefs, so they have their own set of mixing bowls and baking dishes which the head Chef never sees, but take copies of all the mixing bowls and baking dishes currently in use by the calling chef when they are called upon. When the auxiliary recipe is finished, the ingredients in its first mixing bowl are placed in the same order into the calling chef's first mixing bowl.

For example, the main recipe calls for a sauce at some point. The sauce recipe is begun by the sous-chef with an exact copy of all the calling chef's mixing bowls and baking dishes. Changes to these bowls and dishes do not affect the calling chef's bowls and dishes. When the sous-chef is finished, he passes his first mixing bowl back to the calling chef, who empties it into his first mixing bowl.

An auxiliary recipe may have all the same items as a main recipe.

## Sample Recipes

*   [Hello World Souffle](chef_hello.html)
*   [Fibonacci Numbers with Caramel Sauce](chef_fib.html)

## Chef on the Net

*   Chef was featured in [Round Two](http://www.mit.edu/~puzzle/02/round2/05/Puzzle.html) of the 2002 [MIT Mystery Hunt](http://www.mit.edu/~puzzle/)!
*   [Steffen Müller](http://steffen-mueller.net/) has written a [Chef interpeter perl module](http://search.cpan.org/author/SMUELLER/Acme-Chef/).
*   Chef was assigned for a student project in the [LCC 2700 Introduction to Computational Media](http://bogost.com/teaching/introduction_to_computational/) course at the Georgia Institute of Technology, Fall 2005 semester. The Chef component remains on the curriculum as of 2017, and it is claimed that students of this course have written 99% of all Chef programs in the world.
*   Mike Worth has created a recipe which not only prints "Hello world!", but [also works as an actual recipe for a chocolate cake](http://www.mike-worth.com/2013/03/31/baking-a-hello-world-cake/)! And he baked it and ate it!
*   Wesley Janssen, Joost Rijneveld, and Mathijs Vos have a [Chef interpreter on GitHub](https://github.com/joostrijneveld/Chef-Interpreter).
*   Cary Abend reports that Chef was featured on a question in his company's monthly hacker trivia contest:  
    ![quiz question](chef_trivia_sm.jpg)
*   Daniel Temkin has written an article on "[Chef and the Aesthetics of Multicoding](https://esoteric.codes/blog/chef-multicoding-esolang-aesthetics)" on his esoteric languages blog, [esoteric.codes](https://esoteric.codes/).
*   DorIsch has written a Chef program to [approximate pi and at the same time produce a nice pie](https://github.com/DorIsch0/Pi-e). Released to Github on Pi Day, 2022.

- - -

[Home](../) | [Esoteric Programming Languages](./)  
_Last updated: Monday, 14 March, 2022; 15:36:45 PDT._  
Copyright © 1990-2025, David Morgan-Mar. _dmm@dangermouse.net_  
_Hosted by: [DreamHost](http://www.dreamhost.com/rewards.cgi?dmmaus)_