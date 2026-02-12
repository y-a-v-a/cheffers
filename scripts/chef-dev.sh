#!/bin/bash
# Chef Development Environment with Tmux
# Creates a split-pane interface: editor on left, auto-runner on right

set -e

SESSION="chef-dev"
RECIPE="${1:-recipe.chef}"

# Check if recipe file exists, create template if not
if [ ! -f "$RECIPE" ]; then
    echo "Creating new recipe: $RECIPE"
    cat > "$RECIPE" << 'EOF'
Hello World Souffle.

This is a simple Chef program.

Ingredients.
72 g haricot beans
101 g lard
108 g eggs
111 g milk
32 g zucchini
119 g butter
114 g flour
100 g sugar

Method.
Put haricot beans into the mixing bowl.
Put lard into the mixing bowl.
Put eggs into the mixing bowl.
Put eggs into the mixing bowl.
Put milk into the mixing bowl.
Put zucchini into the mixing bowl.
Put butter into the mixing bowl.
Put milk into the mixing bowl.
Put flour into the mixing bowl.
Put eggs into the mixing bowl.
Put sugar into the mixing bowl.
Liquefy contents of the mixing bowl.
Pour contents of the mixing bowl into the baking dish.

Serves 1.
EOF
fi

# Determine editor (default to vim if not set)
EDITOR="${EDITOR:-vim}"

# Check for file watching tools
if command -v entr &> /dev/null; then
    WATCH_CMD="echo '$RECIPE' | entr -c cargo run '$RECIPE'"
elif command -v watch &> /dev/null; then
    echo "Note: 'entr' not found. Using 'watch' instead (less efficient)."
    echo "Install entr for better performance: brew install entr  # or apt-get install entr"
    WATCH_CMD="watch -n 1 -c 'cargo run \"$RECIPE\" 2>&1'"
else
    echo "Warning: Neither 'entr' nor 'watch' found. Using manual loop."
    echo "Install entr for best experience: brew install entr"
    WATCH_CMD="while true; do clear; date; echo '---'; cargo run '$RECIPE' 2>&1; sleep 1; done"
fi

# Check if session already exists
if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Session '$SESSION' already exists. Attaching..."
    tmux attach -t "$SESSION"
    exit 0
fi

# Create new tmux session
tmux new-session -d -s "$SESSION" -n editor

# Left pane: Editor
tmux send-keys -t "$SESSION:editor.0" "$EDITOR $RECIPE" C-m

# Split window vertically (right pane)
tmux split-window -h -t "$SESSION:editor"

# Wait for the shell in the new pane to initialize
sleep 0.3

# Right pane: File watcher + auto-runner
tmux send-keys -t "$SESSION:editor.1" "./scripts/chef-dev-welcome.sh '$RECIPE'" C-m
tmux send-keys -t "$SESSION:editor.1" "$WATCH_CMD" C-m

# Set pane sizes (60% left, 40% right)
tmux resize-pane -t "$SESSION:editor.0" -x 60%

# Focus on editor pane
tmux select-pane -t "$SESSION:editor.0"

# Attach to session
echo ""
echo "Starting Chef development environment..."
echo "File: $RECIPE"
echo ""
tmux attach -t "$SESSION"
