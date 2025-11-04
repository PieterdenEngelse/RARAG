#!/usr/bin/env bash
set -euo pipefail

# Config
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-3010}
QPS=${QPS:-1}
BURST=${BURST:-3}
SEARCH_QPS=${SEARCH_QPS:-}
SEARCH_BURST=${SEARCH_BURST:-}
UPLOAD_QPS=${UPLOAD_QPS:-}
UPLOAD_BURST=${UPLOAD_BURST:-}
TRUST_PROXY=${TRUST_PROXY:-true}
DURATION=${DURATION:-10} # seconds to run the server for this test

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

echo "Building..."
cargo build >/dev/null

echo "Starting server (QPS=$QPS, BURST=$BURST, TRUST_PROXY=$TRUST_PROXY)"
timeout ${DURATION}s cargo run > /tmp/ag_rate_limit_test.out 2>&1 &
PID=$!
sleep 2

echo "Health:"
curl -s "http://$HOST:$PORT/health" | jq '.status,.documents,.vectors' || true

echo "Burst testing /search (direct)"
CODES=()
for i in $(seq 1 $((BURST+5))); do
  CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://$HOST:$PORT/search?q=test")
  CODES+=("$CODE")
  echo "req=$i code=$CODE"
done

# Evaluate: first BURST should be 200, others 429
OK=1
for i in $(seq 1 $((BURST))); do
  if [ "${CODES[$((i-1))]}" != "200" ]; then OK=0; fi
done
for i in $(seq $((BURST+1)) $((BURST+5))); do
  if [ "${CODES[$((i-1))]}" != "429" ]; then OK=0; fi
done

# Proxy header test: use different X-Forwarded-For IPs to bypass same-socket bucket when TRUST_PROXY=true
if [[ "$TRUST_PROXY" == "true" ]]; then
  echo "Testing X-Forwarded-For differentiation"
  code_a=$(curl -s -o /dev/null -w "%{http_code}" -H 'X-Forwarded-For: 1.1.1.1' "http://$HOST:$PORT/search?q=a")
  code_b=$(curl -s -o /dev/null -w "%{http_code}" -H 'X-Forwarded-For: 2.2.2.2' "http://$HOST:$PORT/search?q=b")
  echo "xff a=$code_a b=$code_b"
fi

if [ "$OK" -eq 1 ]; then
  echo "PASS: Rate limiting behaves as expected."
  exit 0
else
  echo "FAIL: Unexpected HTTP codes sequence: ${CODES[*]}"
  echo "--- Server log tail ---"
  tail -n 100 /tmp/ag_rate_limit_test.out || true
  exit 1
fi
