# Tmux Development Environment - Live Demo

## What You Just Saw

### 1. **Working Recipe** (`demo-recipe.chef`)
When you run a working recipe, you get clean output:
```bash
$ cargo run demo-recipe.chef
# Output shows the Fibonacci values as Unicode characters
```

### 2. **Recipe with Error** (`demo-error.chef`)
When there's an error, you get rich, helpful output:
```
error: undefined ingredient
  ingredient: sugar
  in instruction:

  = This instruction references an ingredient that hasn't been declared

  According to the Chef language specification:
  Attempting to use an ingredient without a defined value is a run-time error.

  note: Ingredients must be declared in the ingredients section...

  suggestion:
  Add the ingredient to your ingredients list:
    Ingredients.
    100 g sugar
    ...
```

## How the Tmux Environment Works

### Visual Layout

When you run `./scripts/chef-dev.sh demo-recipe.chef`, you get:

```
┌──────────────────────────────────┬──────────────────────────────────┐
│                                  │                                  │
│  EDITOR PANE (LEFT)              │  CONSOLE PANE (RIGHT)           │
│  vim demo-recipe.chef            │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                  │  Chef Development Environment    │
│  Fibonacci Test Recipe.          │  Watching: demo-recipe.chef      │
│                                  │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│  Ingredients.                    │                                  │
│  10 g counter                    │  Save file to see results...     │
│  0 g current                     │                                  │
│  1 g next                        │  Output:                        │
│                                  │  [output appears here on save]  │
│  Method.                         │                                  │
│  Put counter into mixing bowl.   │                                  │
│  Put current into mixing bowl.   │                                  │
│  ...                             │                                  │
│                                  │                                  │
│  [INSERT MODE]                   │                                  │
│                                  │                                  │
└──────────────────────────────────┴──────────────────────────────────┘
```

### The Workflow

1. **Start the environment:**
   ```bash
   $ ./scripts/chef-dev.sh demo-recipe.chef
   ```

2. **Tmux creates the session:**
   - Left pane (60% width): Opens your editor
   - Right pane (40% width): Starts file watcher

3. **You edit the recipe:**
   - Change `10 g counter` to `5 g counter`
   - Press `:w` (in vim) to save

4. **Right pane updates INSTANTLY:**
   - File watcher detects the change
   - Runs `cargo run demo-recipe.chef`
   - Shows output or error messages

5. **Iterate rapidly:**
   - See error? Fix it in left pane
   - Save
   - See results immediately in right pane
   - No need to switch windows or manually rerun

### Live Example Session

Here's what a real session looks like:

```bash
# Terminal: Start the environment
$ ./scripts/chef-dev.sh demo-recipe.chef

# Tmux creates session and splits screen
# Left pane: vim opens demo-recipe.chef
# Right pane shows:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Chef Development Environment
Watching: demo-recipe.chef
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Save the file in the left pane to see results here.

Keybindings:
  Ctrl-b o  - Switch panes
  Ctrl-b [  - Scroll mode
  Ctrl-b d  - Detach

Starting file watcher...

# You edit, save...
# Right pane updates:

Running: cargo run demo-recipe.chef
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/cheffers demo-recipe.chef`


# You introduce an error (add "Add sugar to mixing bowl")
# Save again...
# Right pane shows:

Running: cargo run demo-recipe.chef
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/cheffers demo-recipe.chef`
error: undefined ingredient
  ingredient: sugar
  in instruction:

  = This instruction references an ingredient that hasn't been declared

  suggestion:
  Add the ingredient to your ingredients list:
    Ingredients.
    100 g sugar
    ...

# You fix it by adding sugar to ingredients
# Save...
# Right pane shows successful output again!
```

## Key Features in Action

### 1. **Auto-reload**
- Uses `entr` (efficient) or `watch` (fallback)
- Detects saves in real-time
- No manual rerun needed

### 2. **Rich Error Messages**
- Colored, formatted output
- Shows ingredient names, bowl indices, etc.
- References Chef language spec
- Suggests fixes

### 3. **Your Editor**
- Uses `$EDITOR` environment variable
- Defaults to vim if not set
- Can override: `EDITOR=nano ./scripts/chef-dev.sh`

### 4. **Session Management**
- Detach with `Ctrl-b d`
- Reattach with `tmux attach -t chef-dev`
- Keeps running in background

### 5. **Flexible Layout**
- 60% editor, 40% console (configurable)
- Switch panes with `Ctrl-b o`
- Scroll output with `Ctrl-b [`

## Try It Yourself!

### Quick Start:
```bash
# 1. Create or use existing recipe
./scripts/chef-dev.sh demo-recipe.chef

# 2. Edit in left pane
# 3. Save (`:w` in vim)
# 4. See instant results in right pane

# 5. Detach to work on other things
Ctrl-b d

# 6. Return later
tmux attach -t chef-dev
```

### Pro Tips:

**Multiple recipes:**
```bash
# While in tmux, create another pane
Ctrl-b "         # Horizontal split
# Now you have 3 panes for different recipes
```

**Scroll through long output:**
```bash
Ctrl-b [         # Enter scroll mode
↑/↓ or PgUp/PgDn # Navigate
q                # Exit scroll mode
```

**Manual commands:**
```bash
# Switch to right pane (Ctrl-b o)
# Stop watcher (Ctrl-C)
# Run custom commands:
cargo test
cargo clippy
cargo run --release recipe.chef
```

## What Makes This Powerful

✅ **Instant Feedback Loop**
- Write code → Save → See results
- No context switching
- No terminal juggling

✅ **Learning Tool**
- See errors immediately
- Understand what went wrong
- Try fixes and see results instantly

✅ **Development Speed**
- No manual rerun commands
- Error messages guide you to fixes
- Quick iteration cycle

✅ **Works with Everything**
- Your familiar editor
- Your familiar terminal
- Your familiar keybindings

## Comparison

### Before (Manual Workflow):
```bash
# Terminal 1
$ vim recipe.chef
# Edit, save, exit

# Terminal 2
$ cargo run recipe.chef
# See error

# Back to Terminal 1
$ vim recipe.chef
# Fix error, save, exit

# Back to Terminal 2
$ cargo run recipe.chef
# Repeat...
```

### After (Tmux Workflow):
```bash
# Single terminal with split panes
$ ./scripts/chef-dev.sh recipe.chef

# Left: Edit
# Right: Auto-updates on save
# No switching, no manual reruns
# Just edit → save → see results
```

## The Result

You get a **REPL-like experience** for Chef programming:
- Immediate visual feedback
- Enhanced error messages guide you
- Fast iteration cycle
- Professional development environment

All with just a shell script and tmux! 🎉
