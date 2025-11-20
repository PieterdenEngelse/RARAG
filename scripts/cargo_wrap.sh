#!/usr/bin/env bash
set -euo pipefail

# Config
WORKDIR="${CARGO_WORKDIR:-/home/pde/ag}"

# TTY-aware environment:
# - If stdout is a TTY, keep normal terminal behavior (colors, etc.)
# - If not a TTY, disable terminal-dependent features for clean headless runs
if [ -t 1 ]; then
  # Interactive terminal: do not force dumb/no-color
  :
else
  export TERM="${TERM:-dumb}"
  export CLICOLOR=0
  export NO_COLOR=1
fi

if [ ! -f "$WORKDIR/Cargo.toml" ]; then
  echo "Error: Cargo.toml not found in $WORKDIR" >&2
  exit 2
fi

echo "[cargo_wrap] cwd=$WORKDIR cargo $*"
/usr/bin/env -C "$WORKDIR" cargo "$@"
