# Chef Development Scripts

## chef-dev.sh - Tmux Development Environment

A split-pane development environment for writing Chef programs with instant feedback.

### Features

- **Left Pane:** Text editor (uses `$EDITOR` environment variable, defaults to vim)
- **Right Pane:** Auto-runner that watches your Chef file and shows output on save
- **Auto-reload:** Save your file in the editor to see results immediately
- **Error messages:** Rich, colored error output with helpful suggestions

### Prerequisites

**Required:**
- `tmux` - Terminal multiplexer
- `cargo` - To run the Chef interpreter

**Optional (for better performance):**
- `entr` - Efficient file watcher (recommended)
  - Install: `apt-get install entr` or `brew install entr`
  - Without entr, the script falls back to `watch` (polls every second)

### Usage

```bash
# Start development environment with existing file
./scripts/chef-dev.sh myrecipe.chef

# Start with default name (creates recipe.chef template if it doesn't exist)
./scripts/chef-dev.sh

# Or from anywhere in the repo
cd /path/to/cheffers
./scripts/chef-dev.sh path/to/recipe.chef
```

### First Time Setup

```bash
# Make sure the script is executable
chmod +x scripts/chef-dev.sh

# Set your preferred editor (optional)
export EDITOR=vim   # or nano, emacs, etc.

# Add to ~/.bashrc or ~/.zshrc to make permanent
echo 'export EDITOR=vim' >> ~/.bashrc
```

### Tmux Keybindings

Once inside the tmux session:

| Keybinding | Action |
|------------|--------|
| `Ctrl-b %` | Split pane vertically |
| `Ctrl-b "` | Split pane horizontally |
| `Ctrl-b o` | Switch between panes |
| `Ctrl-b ←/→/↑/↓` | Navigate between panes |
| `Ctrl-b d` | Detach session (keeps running in background) |
| `Ctrl-b x` | Close current pane |
| `Ctrl-b [` | Enter scroll mode (q to exit) |

### Workflow

1. **Start the environment:**
   ```bash
   ./scripts/chef-dev.sh myrecipe.chef
   ```

2. **Edit your recipe** in the left pane (editor opens automatically)

3. **Save the file** (`:w` in vim, `Ctrl-O Enter` in nano)

4. **See results instantly** in the right pane
   - Successful output shows immediately
   - Errors displayed with helpful suggestions and spec references

5. **Iterate quickly** - edit, save, see results, repeat

### Reattaching to Session

If you detach from the session (Ctrl-b d) or close your terminal:

```bash
# Reattach to existing session
tmux attach -t chef-dev

# List all sessions
tmux ls

# Kill the session when done
tmux kill-session -t chef-dev
```

### Exiting

To exit the development environment:

1. **Close the editor** (`:q` in vim, `Ctrl-X` in nano)
2. **Close the runner pane** with `Ctrl-b x` (confirm with `y`)
3. Or kill the entire session: `tmux kill-session -t chef-dev`

### Tips

**Use multiple panes:**
```bash
# While in tmux, you can create more panes:
Ctrl-b %    # Vertical split
Ctrl-b "    # Horizontal split

# Useful for having multiple recipes open, or viewing docs
```

**Scroll through output:**
```bash
# In the right pane, press Ctrl-b [ to enter scroll mode
# Use arrow keys or Page Up/Down to scroll
# Press q to exit scroll mode
```

**Run specific commands manually:**
```bash
# Switch to right pane (Ctrl-b o)
# Press Ctrl-C to stop the watcher
# Run your own commands:
cargo run myrecipe.chef
cargo test
cargo clippy
```

### Troubleshooting

**"Session already exists"**
- The script detects existing sessions and attaches to them
- To start fresh: `tmux kill-session -t chef-dev`

**File not auto-reloading**
- Make sure you're saving the file (not just exiting insert mode)
- If using `watch` (no entr), there's a 1-second delay
- Install `entr` for instant updates

**Editor not starting**
- Check `echo $EDITOR` to see what editor is set
- Script defaults to `vim` if `$EDITOR` is not set
- Set explicitly: `EDITOR=nano ./scripts/chef-dev.sh recipe.chef`

**Colors not showing**
- Make sure your terminal supports 256 colors
- Try: `export TERM=xterm-256color`

### Examples

**Hello World workflow:**
```bash
$ ./scripts/chef-dev.sh hello.chef
# Creates hello.chef with template
# Edit the recipe in left pane
# Save to see "Hello World" in right pane
```

**Debugging errors:**
```bash
$ ./scripts/chef-dev.sh test.chef
# Intentionally use undefined ingredient
# Save the file
# Right pane shows:
#   - What went wrong
#   - Where the error occurred
#   - How to fix it
#   - Relevant Chef language spec
```

**Quick testing:**
```bash
$ ./scripts/chef-dev.sh fibonacci.chef
# Edit fibonacci recipe
# Save to see the Fibonacci sequence
# Tweak the counter value
# Save again to see different output
```

### Advanced Configuration

Create an alias in your `~/.bashrc` or `~/.zshrc`:

```bash
alias chef='cd ~/projects/cheffers && ./scripts/chef-dev.sh'

# Then use it anywhere:
chef myrecipe.chef
```

### Integration with Git

The script creates a `recipe.chef` template if no file is specified. Add to `.gitignore` if you don't want to commit it:

```bash
echo "recipe.chef" >> .gitignore
```
