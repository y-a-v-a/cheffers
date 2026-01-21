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

# Check if entr is available for efficient file watching
if command -v entr &> /dev/null; then
    WATCH_CMD="echo '$RECIPE' | entr -c cargo run '$RECIPE'"
else
    echo "Note: 'entr' not found. Using 'watch' instead (less efficient)."
    echo "Install entr for better performance: apt-get install entr"
    WATCH_CMD="watch -n 1 -c 'cargo run \"$RECIPE\" 2>&1'"
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

# Right pane: File watcher + auto-runner
tmux send-keys -t "$SESSION:editor.1" "clear" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo 'Chef Development Environment'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo 'Watching: $RECIPE'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo ''" C-m
tmux send-keys -t "$SESSION:editor.1" "echo 'Save the file in the left pane to see results here.'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo ''" C-m
tmux send-keys -t "$SESSION:editor.1" "echo 'Keybindings:'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '  Ctrl-b %  - Split pane vertically'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '  Ctrl-b \"  - Split pane horizontally'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '  Ctrl-b o  - Switch panes'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '  Ctrl-b d  - Detach (resume: tmux attach -t chef-dev)'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo '  Ctrl-b x  - Close pane'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo ''" C-m
tmux send-keys -t "$SESSION:editor.1" "echo 'Starting file watcher...'" C-m
tmux send-keys -t "$SESSION:editor.1" "echo ''" C-m
tmux send-keys -t "$SESSION:editor.1" "sleep 2" C-m
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
