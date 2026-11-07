#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================================="
echo " Rust Telemetry Ingestion Engine: Throughput & Query Demo"
echo "=========================================================="

echo "[1/3] Compiling Ingestion Engine in Release Mode..."
cargo build --release

echo "[2/3] Running Comprehensive Unit & Integration Tests..."
cargo test

echo "[3/3] Running High-Scale Ingestion Microbenchmark (100k Records)..."
./target/release/telemetry-ingestion --bench

echo "=========================================================="
echo " Rust Telemetry Ingestion Engine Demo Completed Successfully."
echo "=========================================================="
