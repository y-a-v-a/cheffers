#!/usr/bin/env bash
# Display welcome message for chef-dev environment

RECIPE="$1"

clear
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo 'Chef Development Environment'
echo "Watching: $RECIPE"
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo ''
echo 'Save the file in the left pane to see results here.'
echo ''
echo 'Keybindings:'
echo '  Ctrl-b %  - Split pane vertically'
echo '  Ctrl-b "  - Split pane horizontally'
echo '  Ctrl-b o  - Switch panes'
echo '  Ctrl-b d  - Detach (resume: tmux attach -t chef-dev)'
echo '  Ctrl-b x  - Close pane'
echo ''
echo 'Starting file watcher...'
echo ''
sleep 2
