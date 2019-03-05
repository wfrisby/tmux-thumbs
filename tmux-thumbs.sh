#!/usr/bin/env bash

CURRENT_PANE_ID=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)
COMMAND="/home/ubuntu/dev/tmux-thumbs/target/debug/tmux-thumbs -a qwerty -r -u --tmux-pane ${CURRENT_PANE_ID}"

tmux new-window -n "[thumbs]" ${COMMAND}
