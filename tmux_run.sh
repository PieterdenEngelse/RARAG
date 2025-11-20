#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/ag"
: > qodo_gui_launch.log
: > qodo_pane.log
: > server.log
{
  echo "==== qodo --login output ===="
  QODO_LOG_LEVEL=debug qodo --login
} &> qodo_gui_launch.log
{
  echo "==== qodo --gui output ===="
  QODO_LOG_LEVEL=debug qodo --gui
} &> qodo_pane.log
