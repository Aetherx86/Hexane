#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "[test] Running Carbon Analytical Query Engine Unit Tests..."

OUTPUT_BIN="/tmp/carbon_query_test_bin"
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin graphene -- \
  compile "$SCRIPT_DIR/src/telemetry_lib.carbon" "$SCRIPT_DIR/src/carbon_driver.c" -O3 -o "$OUTPUT_BIN"

# Execute test run
OUTPUT=$("$OUTPUT_BIN")

echo "$OUTPUT" | grep "TOTAL_RECORDS:   10000" > /dev/null
echo "$OUTPUT" | grep "ERROR_COUNT:     770" > /dev/null
echo "$OUTPUT" | grep "LATENCY_AVG:     118 ms" > /dev/null
echo "$OUTPUT" | grep "Carbon Analytics Query Executed Successfully." > /dev/null

echo "[test] Carbon Analytical Query Engine Unit Tests: PASSED."
