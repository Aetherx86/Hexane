#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================================="
echo " Carbon Analytical Query Engine: Vectorized Analytics Demo"
echo "=========================================================="

echo "[1/3] Compiling Carbon Analytics Package with Graphene (-O3)..."
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin graphene -- \
  compile src/telemetry_lib.carbon src/carbon_driver.c -O3 -o carbon_analytics

echo "[2/3] Running Analytical Validation Tests..."
bash tests/test_analytics.sh

echo "[3/3] Executing Carbon Analytics Engine on 10,000 Records..."
./carbon_analytics

echo "=========================================================="
echo " Carbon Analytical Query Engine Demo Completed Successfully."
echo "=========================================================="
