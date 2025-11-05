#!/usr/bin/env bash
set -euo pipefail

# Config
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-3010}
# Global defaults
QPS=${QPS:-1}
BURST=${BURST:-3}
# Route-specific overrides
SEARCH_QPS=${SEARCH_QPS:-}
SEARCH_BURST=${SEARCH_BURST:-}
UPLOAD_QPS=${UPLOAD_QPS:-}
UPLOAD_BURST=${UPLOAD_BURST:-}
TRUST_PROXY=${TRUST_PROXY:-false}
DURATION=${DURATION:-12} # seconds to run the server for this test

export RUST_LOG=${RUST_LOG:-info}
export RATE_LIMIT_ENABLED=true
export RATE_LIMIT_QPS=$QPS
export RATE_LIMIT_BURST=$BURST
export TRUST_PROXY=$TRUST_PROXY

# Optional route-specific
if [[ -n "${SEARCH_QPS}" ]]; then export RATE_LIMIT_SEARCH_QPS="$SEARCH_QPS"; fi
if [[ -n "${SEARCH_BURST}" ]]; then export RATE_LIMIT_SEARCH_BURST="$SEARCH_BURST"; fi
if [[ -n "${UPLOAD_QPS}" ]]; then export RATE_LIMIT_UPLOAD_QPS="$UPLOAD_QPS"; fi
if [[ -n "${UPLOAD_BURST}" ]]; then export RATE_LIMIT_UPLOAD_BURST="$UPLOAD_BURST"; fi

# Build
cargo build >/dev/null

echo "Starting server (global QPS=$QPS BURST=$BURST, SEARCH_QPS=${SEARCH_QPS:-$QPS}, UPLOAD_QPS=${UPLOAD_QPS:-$QPS})"
timeout ${DURATION}s cargo run > /tmp/ag_rate_limit_rw_test.out 2>&1 &
PID=$!
sleep 2

pass() { echo "PASS: $1"; }
fail() { echo "FAIL: $1"; tail -n 120 /tmp/ag_rate_limit_rw_test.out || true; exit 1; }

# Health
curl -s "http://$HOST:$PORT/health" >/dev/null || fail "health endpoint not reachable"

# 1) Verify READ bucket on /search
codes=()
for i in $(seq 1 $(( ${SEARCH_BURST:-$BURST} + 4 ))); do
  c=$(curl -s -o /dev/null -w "%{http_code}" "http://$HOST:$PORT/search?q=ping")
  codes+=("$c")
  echo "search req=$i code=$c"
done
ok=1
for i in $(seq 1 ${SEARCH_BURST:-$BURST}); do [[ "${codes[$((i-1))]}" == "200" ]] || ok=0; done
for i in $(seq $(( ${SEARCH_BURST:-$BURST} + 1 )) $(( ${SEARCH_BURST:-$BURST} + 4 ))); do [[ "${codes[$((i-1))]}" == "429" ]] || ok=0; done
[[ $ok -eq 1 ]] || fail "search read bucket mismatch: ${codes[*]}"
pass "/search read bucket ok"

# 2) Verify READ bucket also applies to /rerank
codes=()
for i in $(seq 1 $(( ${SEARCH_BURST:-$BURST} + 2 ))); do
  c=$(curl -s -o /dev/null -w "%{http_code}" -X POST "http://$HOST:$PORT/rerank" \
      -H 'Content-Type: application/json' \
      -d '{"query":"x","candidates":["a","b"]}')
  codes+=("$c")
  echo "rerank req=$i code=$c"
done
ok=1
for i in $(seq 1 ${SEARCH_BURST:-$BURST}); do [[ "${codes[$((i-1))]}" == "200" ]] || ok=0; done
for i in $(seq $(( ${SEARCH_BURST:-$BURST} + 1 )) $(( ${SEARCH_BURST:-$BURST} + 2 ))); do [[ "${codes[$((i-1))]}" == "429" ]] || ok=0; done
[[ $ok -eq 1 ]] || fail "rerank read bucket mismatch: ${codes[*]}"
pass "/rerank read bucket ok"

# 3) Verify WRITE bucket on /upload
# Prepare tiny file payload
TMPFILE=$(mktemp)
echo "hello" > "$TMPFILE"

codes=()
for i in $(seq 1 $(( ${UPLOAD_BURST:-$BURST} + 2 ))); do
  c=$(curl -s -o /dev/null -w "%{http_code}" -X POST "http://$HOST:$PORT/upload" \
    -F "file=@${TMPFILE};filename=test_${i}.txt" )
  codes+=("$c")
  echo "upload req=$i code=$c"
done
rm -f "$TMPFILE"

ok=1
for i in $(seq 1 ${UPLOAD_BURST:-$BURST}); do [[ "${codes[$((i-1))]}" == "200" ]] || ok=0; done
for i in $(seq $(( ${UPLOAD_BURST:-$BURST} + 1 )) $(( ${UPLOAD_BURST:-$BURST} + 2 ))); do [[ "${codes[$((i-1))]}" == "429" ]] || ok=0; done
[[ $ok -eq 1 ]] || fail "upload write bucket mismatch: ${codes[*]}"
pass "/upload write bucket ok"

# Optional: print rate limit metrics
echo "--- rate limit metrics ---"
curl -s "http://$HOST:$PORT/metrics" | grep -E "^rate_limit_drops_" || true

exit 0
