#!/bin/bash
# Visual demonstration of the tmux development environment

clear

cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║             🧑‍🍳 Chef Development Environment - Visual Demo 🧑‍🍳               ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝

Let me show you how the tmux development environment works!

EOF

read -p "Press ENTER to start..."
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 1: Starting the Development Environment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

$ ./scripts/chef-dev.sh demo-recipe.chef

Starting Chef development environment...
File: demo-recipe.chef

EOF

sleep 2

cat << 'EOF'
Creating tmux session 'chef-dev'...
Opening editor in left pane...
Starting file watcher in right pane...

EOF

sleep 2
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 2: The Split-Pane Interface
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌────────────────────────────────────┬──────────────────────────────────────┐
│                                    │                                      │
│  📝 EDITOR (vim)                   │  🖥️  CONSOLE OUTPUT                  │
│  File: demo-recipe.chef            │                                      │
│  ═══════════════════════════       │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                    │  Chef Development Environment        │
│  Fibonacci Test Recipe.            │  Watching: demo-recipe.chef          │
│                                    │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  A simple Chef program.            │                                      │
│                                    │  Save the file to see results...     │
│  Ingredients.                      │                                      │
│  10 g counter                      │                                      │
│  0 g current                       │  Keybindings:                        │
│  1 g next                          │    Ctrl-b o  - Switch panes          │
│                                    │    Ctrl-b [  - Scroll mode           │
│  Method.                           │    Ctrl-b d  - Detach                │
│  Put counter into mixing bowl.     │                                      │
│  Put current into mixing bowl.     │  Ready to run!                       │
│  Put next into mixing bowl.        │                                      │
│                                    │                                      │
│  Liquefy contents of mixing bowl.  │                                      │
│  Pour contents into baking dish.   │                                      │
│                                    │                                      │
│  Serves 1.                         │                                      │
│                                    │                                      │
│  -- INSERT --                      │                                      │
└────────────────────────────────────┴──────────────────────────────────────┘
                60% width                         40% width

EOF

read -p "Press ENTER to see what happens when you save the file..."
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 3: Saving the File (`:w` in vim)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌────────────────────────────────────┬──────────────────────────────────────┐
│                                    │                                      │
│  📝 EDITOR (vim)                   │  🖥️  CONSOLE OUTPUT                  │
│  File: demo-recipe.chef            │                                      │
│  ═══════════════════════════       │  ⚡ File changed! Running...          │
│                                    │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  Fibonacci Test Recipe.            │  $ cargo run demo-recipe.chef        │
│                                    │                                      │
│  A simple Chef program.            │  Compiling cheffers v0.2.0           │
│                                    │  Finished dev profile in 0.06s       │
│  Ingredients.                      │  Running target/debug/cheffers       │
│  10 g counter                      │                                      │
│  0 g current                       │  Output:                            │
│  1 g next                          │

                     │
│                                    │                                      │
│  Method.                           │  ✓ Success!                          │
│  Put counter into mixing bowl.     │                                      │
│  Put current into mixing bowl.     │                                      │
│  Put next into mixing bowl.        │                                      │
│                                    │                                      │
│  Liquefy contents of mixing bowl.  │                                      │
│  Pour contents into baking dish.   │                                      │
│                                    │                                      │
│  Serves 1.                         │                                      │
│                                    │                                      │
│  "demo-recipe.chef" 18L, 342B      │                                      │
└────────────────────────────────────┴──────────────────────────────────────┘

🎉 The right pane updated INSTANTLY when you saved!

EOF

read -p "Press ENTER to introduce an error..."
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 4: Introducing an Error
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

You add a line that uses an undefined ingredient:

┌────────────────────────────────────┬──────────────────────────────────────┐
│                                    │                                      │
│  📝 EDITOR (vim)                   │  🖥️  CONSOLE OUTPUT                  │
│  File: demo-recipe.chef [Modified] │                                      │
│  ═══════════════════════════       │  Previous output...                  │
│                                    │                                      │
│  Fibonacci Test Recipe.            │                                      │
│                                    │                                      │
│  A simple Chef program.            │                                      │
│                                    │                                      │
│  Ingredients.                      │                                      │
│  10 g counter                      │                                      │
│  0 g current                       │                                      │
│  1 g next                          │                                      │
│                                    │                                      │
│  Method.                           │                                      │
│  Put counter into mixing bowl.     │                                      │
│  Put current into mixing bowl.     │                                      │
│  Put next into mixing bowl.        │                                      │
│  Add sugar to mixing bowl. ← NEW! │                                      │
│                                    │                                      │
│  Liquefy contents of mixing bowl.  │                                      │
│  Pour contents into baking dish.   │                                      │
│                                    │                                      │
│  Serves 1.                         │                                      │
│                                    │                                      │
│  -- INSERT --                      │                                      │
└────────────────────────────────────┴──────────────────────────────────────┘

EOF

read -p "Press ENTER to save and see the error message..."
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 5: Enhanced Error Message Appears!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌────────────────────────────────────┬──────────────────────────────────────┐
│                                    │                                      │
│  📝 EDITOR (vim)                   │  🖥️  CONSOLE OUTPUT                  │
│  File: demo-recipe.chef            │                                      │
│  ═══════════════════════════       │  ⚡ File changed! Running...          │
│                                    │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  Fibonacci Test Recipe.            │  $ cargo run demo-recipe.chef        │
│                                    │                                      │
│  A simple Chef program.            │  error: undefined ingredient         │
│                                    │    ingredient: sugar                 │
│  Ingredients.                      │    in instruction:                   │
│  10 g counter                      │                                      │
│  0 g current                       │  = This instruction references an    │
│  1 g next                          │    ingredient that hasn't been       │
│                                    │    declared                          │
│  Method.                           │                                      │
│  Put counter into mixing bowl.     │  According to the Chef language      │
│  Put current into mixing bowl.     │  specification:                      │
│  Put next into mixing bowl.        │  Attempting to use an ingredient     │
│  Add sugar to mixing bowl. ← 💥   │  without a defined value is a        │
│                                    │  run-time error.                     │
│  Liquefy contents of mixing bowl.  │                                      │
│  Pour contents into baking dish.   │  note: Ingredients must be declared  │
│                                    │  in the ingredients section...       │
│  Serves 1.                         │                                      │
│                                    │  suggestion:                         │
│  "demo-recipe.chef" 19L, 373B      │  Add the ingredient to your list:    │
│                                    │    Ingredients.                      │
│                                    │    100 g sugar                       │
│                                    │    ...                               │
└────────────────────────────────────┴──────────────────────────────────────┘

🎯 Rich error message tells you EXACTLY what's wrong and HOW to fix it!

EOF

read -p "Press ENTER to fix the error..."
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 6: Fixing the Error
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

You add 'sugar' to the ingredients:

┌────────────────────────────────────┬──────────────────────────────────────┐
│                                    │                                      │
│  📝 EDITOR (vim)                   │  🖥️  CONSOLE OUTPUT                  │
│  File: demo-recipe.chef [Modified] │                                      │
│  ═══════════════════════════       │  Previous error message...           │
│                                    │                                      │
│  Fibonacci Test Recipe.            │                                      │
│                                    │                                      │
│  A simple Chef program.            │                                      │
│                                    │                                      │
│  Ingredients.                      │                                      │
│  10 g counter                      │                                      │
│  0 g current                       │                                      │
│  1 g next                          │                                      │
│  5 g sugar          ← FIXED! ✓    │                                      │
│                                    │                                      │
│  Method.                           │                                      │
│  Put counter into mixing bowl.     │                                      │
│  Put current into mixing bowl.     │                                      │
│  Put next into mixing bowl.        │                                      │
│  Add sugar to mixing bowl.         │                                      │
│                                    │                                      │
│  Liquefy contents of mixing bowl.  │                                      │
│  Pour contents into baking dish.   │                                      │
│                                    │                                      │
│  Serves 1.                         │                                      │
│                                    │                                      │
│  -- INSERT --                      │                                      │
└────────────────────────────────────┴──────────────────────────────────────┘

EOF

read -p "Press ENTER to save and see it work..."
clear

cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 7: Success! It Works!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌────────────────────────────────────┬──────────────────────────────────────┐
│                                    │                                      │
│  📝 EDITOR (vim)                   │  🖥️  CONSOLE OUTPUT                  │
│  File: demo-recipe.chef            │                                      │
│  ═══════════════════════════       │  ⚡ File changed! Running...          │
│                                    │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  Fibonacci Test Recipe.            │  $ cargo run demo-recipe.chef        │
│                                    │                                      │
│  A simple Chef program.            │  Finished dev profile in 0.06s       │
│                                    │  Running target/debug/cheffers       │
│  Ingredients.                      │                                      │
│  10 g counter                      │  Output:                            │
│  0 g current                       │
                     │
│  1 g next                          │                                      │
│  5 g sugar          ✓             │  ✓ Success!                          │
│                                    │                                      │
│  Method.                           │                                      │
│  Put counter into mixing bowl.     │                                      │
│  Put current into mixing bowl.     │                                      │
│  Put next into mixing bowl.        │                                      │
│  Add sugar to mixing bowl.         │                                      │
│                                    │                                      │
│  Liquefy contents of mixing bowl.  │                                      │
│  Pour contents into baking dish.   │                                      │
│                                    │                                      │
│  Serves 1.                         │                                      │
│                                    │                                      │
│  "demo-recipe.chef" 20L, 392B      │                                      │
└────────────────────────────────────┴──────────────────────────────────────┘

🎉 It works! The error is fixed and output appears instantly!

EOF

sleep 2
clear

cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║                          🎯 KEY TAKEAWAYS 🎯                              ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝

✅ INSTANT FEEDBACK
   • Save your file → See results immediately
   • No manual rerun needed
   • No switching between windows

✅ ENHANCED ERRORS
   • Colored, formatted output
   • Shows exactly what's wrong
   • References Chef language spec
   • Suggests how to fix it

✅ RAPID ITERATION
   • Edit → Save → See → Fix → Repeat
   • Perfect for learning and debugging
   • REPL-like experience for Chef

✅ YOUR TOOLS
   • Uses your preferred editor ($EDITOR)
   • Familiar tmux keybindings
   • No new tools to learn

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

TO GET STARTED:

  $ ./scripts/chef-dev.sh myrecipe.chef

Or create a new recipe from template:

  $ ./scripts/chef-dev.sh

Then:
  • Edit in left pane
  • Save to see results in right pane
  • Ctrl-b o to switch panes
  • Ctrl-b d to detach
  • tmux attach -t chef-dev to return

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

See scripts/README.md for complete documentation!

Happy Chef cooking! 👨‍🍳🧑‍🍳👩‍🍳

EOF
