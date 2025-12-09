#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null; then
  echo "cargo not found; install Rust toolchain" >&2
  exit 1
fi

if ! command -v curl >/dev/null; then
  echo "curl not found; install curl" >&2
  exit 1
fi

MODES=("fixed" "lightweight")
OUTDIR="chunker_reports"
mkdir -p "$OUTDIR"
DATA_FILE="$OUTDIR/metrics_$(date +%Y%m%d_%H%M%S).jsonl"

APP_BIN="./target/debug/ag"
HELPER_BIN="./target/debug/chunker_eval"

cat <<'RUST' > chunker_eval.rs
use std::env;
use std::process::Command;
use std::time::Duration;
use std::{thread};

fn start_server() -> std::process::Child {
    Command::new(env::var("APP_BIN").unwrap_or_else(|_| "./target/debug/ag".into()))
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start server")
}

fn wait_for_health() {
    for _ in 0..30 {
        if Command::new("curl")
            .args(["-sf", "http://127.0.0.1:3010/monitoring/health"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return;
        }
        thread::sleep(Duration::from_secs(1));
    }
    panic!("server did not become healthy in time");
}

fn trigger_reindex() {
    Command::new("curl")
        .args(["-sf", "-X", "POST", "http://127.0.0.1:3010/reindex"])
        .status()
        .expect("curl reindex failed");
    thread::sleep(Duration::from_millis(1500));
}

fn capture_logs(mut child: std::process::Child) -> String {
    let output = child.wait_with_output().expect("failed to capture logs");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn main() {
    let mode = env::var("CHUNKER_MODE").unwrap_or_else(|_| "fixed".into());
    env::set_var("CHUNKER_MODE", &mode);

    std::fs::write("compare_chunkers.log", "").unwrap();

    let mut child = start_server();
    wait_for_health();
    trigger_reindex();
    child.kill().ok();
    let logs = capture_logs(child);

    let last = logs
        .lines()
        .filter(|line| line.contains("index_file:"))
        .last()
        .unwrap_or("");

    if last.is_empty() {
        eprintln!("No index_file log line captured for mode {}", mode);
        std::process::exit(1);
    }

    println!("{}", last);
}
RUST

rustc chunker_eval.rs -o "$HELPER_BIN"
rm chunker_eval.rs

run_for_mode() {
  local mode="$1"
  echo "\n=== Running reindex for CHUNKER_MODE=$mode ==="
  export CHUNKER_MODE="$mode"
  cargo build --bin ag >/dev/null
  local last_log
  last_log=$(APP_BIN="$APP_BIN" CHUNKER_MODE="$mode" "$HELPER_BIN")
  LOG_LINES+=("$last_log")
}

declare -a LOG_LINES
for mode in "${MODES[@]}"; do
  run_for_mode "$mode"
done

echo "\nJSON summary (append to $DATA_FILE):"
for line in "${LOG_LINES[@]}"; do
  FILE=$(echo "$line" | sed -n "s/.*file='\([^']*\)'.*/\1/p")
  MODE=$(echo "$line" | sed -n "s/.*mode=\([^ ]*\).*/\1/p")
  CHUNKS=$(echo "$line" | sed -n "s/.*chunks=\([0-9]*\).*/\1/p")
  TOKENS=$(echo "$line" | sed -n "s/.*tokens=\([0-9]*\).*/\1/p")
  DURATION=$(echo "$line" | sed -n "s/.*duration_ms=\([0-9]*\).*/\1/p")
  JSON="{\"file\":\"$FILE\",\"mode\":\"$MODE\",\"chunks\":$CHUNKS,\"tokens\":$TOKENS,\"duration_ms\":$DURATION}"
  echo "$JSON" | tee -a "$DATA_FILE"
done

echo "\nComparison complete. Review $DATA_FILE or rerun for additional modes."
