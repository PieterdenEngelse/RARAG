#!/usr/bin/env bash
set -euo pipefail

# Generic command wrapper with:
# - TTY-aware color handling
# - Robust cwd via env -C
# - Optional clean environment mode

usage() {
  echo "Usage: $0 [--workdir DIR] [--clean-env] -- <command> [args...]"
  echo "  --workdir DIR  Run command with cwd set to DIR (default: /home/pde/ag)"
  echo "  --clean-env    Run with env -i (clean environment), keeping only PATH, HOME, USER"
  echo "  Everything after -- is treated as the command and its arguments."
}

WORKDIR="/home/pde/ag"
CLEAN_ENV=false

# Parse flags
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workdir)
      WORKDIR="$2"; shift 2;;
    --clean-env)
      CLEAN_ENV=true; shift;;
    --)
      shift; break;;
    -h|--help)
      usage; exit 0;;
    *)
      break;;
  esac
done

if [[ $# -lt 1 ]]; then
  usage; exit 2
fi

if [[ ! -d "$WORKDIR" ]]; then
  echo "[cmd_wrap] Error: workdir not found: $WORKDIR" >&2
  exit 2
fi

# TTY-aware env: keep colors in real terminals, suppress in headless
if [ -t 1 ]; then
  :
else
  export TERM="${TERM:-dumb}"
  export CLICOLOR=0
  export NO_COLOR=1
fi

echo "[cmd_wrap] cwd=$WORKDIR $*"

if $CLEAN_ENV; then
  # Keep minimal env; preserve PATH, HOME, USER if set
  /usr/bin/env -i \
    PATH="${PATH:-/usr/bin:/bin}" \
    HOME="${HOME:-/home/pde}" \
    USER="${USER:-pde}" \
    /usr/bin/env -C "$WORKDIR" "$@"
else
  /usr/bin/env -C "$WORKDIR" "$@"
fi
