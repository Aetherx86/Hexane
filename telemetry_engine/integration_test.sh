#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================================="
echo " End-to-End Integration Test: 4-Tier Distributed Telemetry"
echo "=========================================================="

PORT=8089
INGEST_BIN="$SCRIPT_DIR/ingestion_rust/target/release/telemetry-ingestion"
ZIG_AGENT_BIN="$SCRIPT_DIR/daemon_zig/zig-out/bin/telemetry-agent"
CARBON_BIN="$SCRIPT_DIR/query_carbon/carbon_analytics"

# 1. Build all components
echo "[1/6] Building Ingestion Engine (Rust)..."
(cd "$SCRIPT_DIR/ingestion_rust" && cargo build --release)

echo "[2/6] Building Kubernetes Daemon Agent (Zig)..."
(cd "$SCRIPT_DIR/daemon_zig" && zig build -Doptimize=ReleaseFast)

echo "[3/6] Compiling Carbon Analytical Query Engine (Hexene/Graphene)..."
(cd "$SCRIPT_DIR/query_carbon" && cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin graphene -- compile src/telemetry_lib.carbon src/carbon_driver.c -O3 -o carbon_analytics)

# 2. Launch Ingestion Engine in Background
echo "[4/6] Spawning Rust Ingestion Server on port $PORT..."
"$INGEST_BIN" --port "$PORT" --capacity 200000 > /tmp/ingest_test.log 2>&1 &
SERVER_PID=$!

cleanup() {
  echo "Terminating test background server (PID: $SERVER_PID)..."
  kill -9 "$SERVER_PID" 2>/dev/null || true
}
trap cleanup EXIT

# Allow server to bind
sleep 1

# Check health
curl -s "http://127.0.0.1:$PORT/health" | grep "healthy" > /dev/null
echo "[OK] Server is healthy."

# 3. Stream from Zig Daemon Agent
echo "[5/6] Streaming 10 telemetry batches from Zig Node Agent..."
"$ZIG_AGENT_BIN" --port "$PORT" --count 10 --interval-ms 20

# Query Rust Ingestion Engine
QUERY_OUT=$(curl -s "http://127.0.0.1:$PORT/api/v1/query")
echo "Ingested telemetry response: $QUERY_OUT" | head -c 200
echo ""

# 4. Run Carbon Query Engine over synthetic telemetry dataset
echo "[6/6] Executing Carbon Query Engine Aggregations..."
CARBON_OUT=$("$CARBON_BIN")
echo "$CARBON_OUT" | grep "Carbon Analytics Query Executed Successfully." > /dev/null

echo "=========================================================="
echo " 4-Tier Distributed Telemetry Engine Test: PASSED (100%)"
echo "=========================================================="
