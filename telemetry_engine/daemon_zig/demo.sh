#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================================="
echo " Zig Telemetry Node Daemon: Standalone Scraper & Stream Demo"
echo "=========================================================="

echo "[1/3] Building Zig telemetry-agent binary..."
zig build -Doptimize=ReleaseFast

echo "[2/3] Running Agent Unit Test Suite..."
zig build test

echo "[3/3] Emitting 5 Scrape Batches (Synthetic Kube Node Mode)..."
./zig-out/bin/telemetry-agent --demo --stdout --count 5 --interval-ms 150

echo "=========================================================="
echo " Zig Telemetry Daemon Demo Completed Successfully."
echo "=========================================================="
