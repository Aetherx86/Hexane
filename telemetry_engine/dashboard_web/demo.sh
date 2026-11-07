#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================================="
echo " Next.js & TypeScript Observability Dashboard Demo"
echo "=========================================================="

echo "[1/3] Running Unit Test Suite..."
npm test

echo "[2/3] Verifying Production Next.js Build..."
npm run build

echo "[3/3] Dashboard Ready. To launch locally, run: npm run start"
echo "=========================================================="
echo " Next.js Dashboard Demo Completed Successfully."
echo "=========================================================="
