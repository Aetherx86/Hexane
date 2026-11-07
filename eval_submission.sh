#!/usr/bin/env bash
# Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
# Academic Evaluation and Grading Verification Runner
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================================="
echo " Academic Submission Evaluation: Hexene & Telemetry Engine"
echo " Author: jk9576 (Jeong Hoon Kim / Sushi Kim)"
echo " Institution: Hanyang University"
echo " Date: 2026-11-06"
echo "=========================================================="
echo ""

TOTAL_SUITES=7
PASSED_SUITES=0

run_step() {
  local num="$1"
  local title="$2"
  local cmd="$3"
  echo "[$num/$TOTAL_SUITES] Running: $title..."
  if eval "$cmd"; then
    echo "[$num/$TOTAL_SUITES] [PASS] $title"
    PASSED_SUITES=$((PASSED_SUITES + 1))
  else
    echo "[$num/$TOTAL_SUITES] [FAIL] $title"
    exit 1
  fi
  echo ""
}

run_step "1" "Carbonation/Hexene Compiler Core Test Suite" "cargo test --quiet"
run_step "2" "N-Queens 8-Stage Polyglot Benchmark Suite" "cargo test --test queen_full_suite_tests --quiet"
run_step "3" "Zig 0.17 DaemonSet Metrics & Log Tailer Agent" "(cd telemetry_engine/daemon_zig && zig build test)"
run_step "4" "Rust Columnar Time-Series Ingestion & Threat Sanitizer" "(cd telemetry_engine/ingestion_rust && cargo test --quiet)"
run_step "5" "Carbon Analytical Query Engine Rollups" "./telemetry_engine/query_carbon/tests/test_analytics.sh"

# Ensure dashboard dependencies are installed if clean clone
if [ ! -d "telemetry_engine/dashboard_web/node_modules" ]; then
  (cd telemetry_engine/dashboard_web && npm install --no-audit --no-fund --silent)
fi

run_step "6" "Next.js & TypeScript Observability Dashboard Suite" "(cd telemetry_engine/dashboard_web && npm test)"
run_step "7" "4-Tier End-to-End Distributed Telemetry Pipeline" "./telemetry_engine/integration_test.sh"

echo "=========================================================="
echo " Evaluation Scorecard: $PASSED_SUITES / $TOTAL_SUITES Suites Passed (100%)"
echo " Status: SUBMISSION VERIFIED"
echo "=========================================================="
