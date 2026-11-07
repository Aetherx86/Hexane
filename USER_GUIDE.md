# Hexene Telemetry Engine - User Guide

Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

This guide provides instructions for deploying, operating, querying, and running demos across all four tiers of the Hexene Distributed Log Analytics and Telemetry Engine.

---

## Table of Contents
1. System Requirements
2. Quick Start: Standalone Demos
3. Running the End-to-End Integration Pipeline
4. Kubernetes DaemonSet Deployment (Zig Agent)
5. Operating the Rust Ingestion Engine
6. Executing Carbon Analytical Queries
7. Launching the Next.js Dashboard
8. Test Suite Execution Reference

---

## 1. System Requirements

* **Zig**: Version 0.13.0 or higher (0.17.1 verified)
* **Rust & Cargo**: Rust 1.80+ (2024 edition supported)
* **LLVM / Clang**: Clang 16+ (supporting -O3 compilation)
* **Node.js & npm**: Node v18+ (Node v20+ recommended)
* **OS**: Linux (x86_64, aarch64) or macOS (Apple Silicon / Intel)

---

## 2. Quick Start: Standalone Demos

Each tier contains a standalone demo script with its own self-contained test suite:

### Tier 1: Zig Kubernetes Node Daemon
```bash
cd telemetry_engine/daemon_zig
./demo.sh
```
* **What it does**: Compiles the native Zig agent with `-Doptimize=ReleaseFast`, executes unit tests, and emits 5 structured scrape batches simulating host metrics and container logs.

### Tier 2: Rust Ingestion Engine
```bash
cd telemetry_engine/ingestion_rust
./demo.sh
```
* **What it does**: Compiles the Rust server in release mode, runs unit and integration tests, and executes a high-scale microbenchmark ingesting 100,000 records at >1.3 million logs/sec.

### Tier 3: Carbon Analytical Query Engine
```bash
cd telemetry_engine/query_carbon
./demo.sh
```
* **What it does**: Compiles `telemetry_lib.carbon` and `carbon_driver.c` using Hexene / Graphene at `-O3`, runs regression tests, and computes vectorized percentiles (p50, p95, p99) and histogram rollups over 10,000 records.

### Tier 4: Next.js & TypeScript Observability Dashboard
```bash
cd telemetry_engine/dashboard_web
./demo.sh
```
* **What it does**: Executes Node.js unit tests for query parsing and percentile math, builds an optimized Next.js production bundle, and confirms dashboard readiness.

---

## 3. Running the End-to-End Integration Pipeline

To run the complete 4-tier pipeline where the Zig agent streams data into the Rust ingestion engine, queries are served over HTTP, and Carbon processes vectorized rollups:

```bash
cd telemetry_engine
./integration_test.sh
```

---

## 4. Kubernetes DaemonSet Deployment (Zig Agent)

The Zig agent is packaged to deploy across all Kubernetes nodes via a lightweight DaemonSet.

### Manifest Configuration (`k8s/daemonset.yaml`)
```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: telemetry-daemon
  namespace: monitoring
spec:
  selector:
    matchLabels:
      name: telemetry-daemon
  template:
    metadata:
      labels:
        name: telemetry-daemon
    spec:
      hostPID: true
      hostNetwork: true
      containers:
        - name: telemetry-agent
          image: telemetry-agent:latest
          args:
            - "--host"
            - "telemetry-ingestion.monitoring.svc.cluster.local"
            - "--port"
            - "8080"
            - "--interval-ms"
            - "500"
          resources:
            requests:
              cpu: "10m"
              memory: "8Mi"
            limits:
              cpu: "50m"
              memory: "16Mi"
          volumeMounts:
            - name: varlog
              mountPath: /var/log
              readOnly: true
            - name: procfs
              mountPath: /proc
              readOnly: true
```

### Deployment Commands
```bash
# 1. Build container image
docker build -t telemetry-agent:latest -f telemetry_engine/daemon_zig/Dockerfile .

# 2. Apply Kubernetes manifest
kubectl apply -f telemetry_engine/daemon_zig/k8s/daemonset.yaml

# 3. Verify daemon pods running on all nodes
kubectl get pods -n monitoring -l name=telemetry-daemon -o wide
```

---

## 5. Operating the Rust Ingestion Engine

### CLI Options
```
Usage: telemetry-ingestion [OPTIONS]

Options:
  --host <IP>          Server bind host (default: 0.0.0.0)
  --port <PORT>        Server bind port (default: 8080)
  --capacity <N>       In-memory ring buffer record limit (default: 500000)
  --bench              Execute throughput microbenchmark (100k records) and exit
  --demo               Pre-populate realistic telemetry datasets for dashboard
  --help, -h           Show this help message
```

### Running the Server
```bash
cargo run --release --manifest-path telemetry_engine/ingestion_rust/Cargo.toml -- --port 8080 --demo
```

### REST API Endpoints
* `POST /api/v1/logs`: Ingest JSON batch of log records.
* `POST /api/v1/metrics`: Ingest JSON metric sample.
* `GET /api/v1/query`: Filter logs and return statistical percentiles.
  * Parameters: `service`, `min_status`, `max_status`, `min_latency`, `keyword`, `limit`.
* `GET /api/v1/metrics/live`: Returns recent host/node system metrics.
* `GET /api/v1/stats`: Returns total ingested count and active buffer sizes.
* `GET /health`: Healthcheck endpoint.

---

## 6. Executing Carbon Analytical Queries

The Carbon analytics engine can be compiled and executed directly from the terminal.

### Compilation
```bash
cargo run --bin graphene -- compile \
  telemetry_engine/query_carbon/src/telemetry_lib.carbon \
  telemetry_engine/query_carbon/src/carbon_driver.c \
  -O3 -o /tmp/carbon_analytics
```

### Execution
```bash
# Run with self-test dataset:
/tmp/carbon_analytics

# Run with exported binary telemetry dataset:
/tmp/carbon_analytics /path/to/exported_data.bin
```

---

## 7. Launching the Next.js Dashboard

### Development Mode
```bash
cd telemetry_engine/dashboard_web
npm run dev
```
Navigate to `http://localhost:3000` in your web browser.

### Production Mode
```bash
cd telemetry_engine/dashboard_web
npm run build
npm run start
```

### Dashboard Features
* **Live Stream Terminal**: Displays live-updating logs with severity badges (INFO, WARN, ERROR), latency indicators, and expandable structured JSON payloads.
* **Complex Query Builder**: Filter by service, HTTP status classes (2xx, 4xx, 5xx), minimum latency threshold, and free-text regex tokens.
* **Drag-and-Drop Metrics**: Reorder CPU, Memory, Latency Percentiles, and Error Rate cards.
* **Alert Rule Manager**: Configure threshold triggers, monitor alert states (`OK`, `PENDING`, `FIRING`), and acknowledge active incidents.

---

## 8. Test Suite Execution Reference

To verify every test suite across the entire project repository:

```bash
# 1. Main Hexene compiler, interpreter, and N-Queens test suites
cargo test

# 2. Queen benchmark full suite
cargo test --test queen_full_suite_tests

# 3. Zig Node Agent unit tests
(cd telemetry_engine/daemon_zig && zig build test)

# 4. Rust Ingestion Engine tests
(cd telemetry_engine/ingestion_rust && cargo test)

# 5. Carbon Analytical Query tests
bash telemetry_engine/query_carbon/tests/test_analytics.sh

# 6. Next.js Dashboard unit tests
(cd telemetry_engine/dashboard_web && npm test)

# 7. End-to-end integration test
bash telemetry_engine/integration_test.sh
```
