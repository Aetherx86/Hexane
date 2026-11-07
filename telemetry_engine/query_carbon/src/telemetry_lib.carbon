// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
package TelemetryLib api;

fn ComputeErrorRateBps(errors: i32, total: i32) -> i32 {
  if (total == 0) {
    return 0;
  }
  return (errors * 10000) / total;
}

fn ComputeAvgLatency(sum_latency: i32, count: i32) -> i32 {
  if (count == 0) {
    return 0;
  }
  return sum_latency / count;
}

fn ComputePercentileIndex(count: i32, numerator: i32, denominator: i32) -> i32 {
  if (count <= 0) {
    return 0;
  }
  var idx: i32 = (count * numerator) / denominator;
  if (idx >= count) {
    idx = count - 1;
  }
  return idx;
}

fn LatencyToBucket(latency_ms: i32) -> i32 {
  if (latency_ms < 20) {
    return 0;
  }
  if (latency_ms < 50) {
    return 1;
  }
  if (latency_ms < 100) {
    return 2;
  }
  if (latency_ms < 250) {
    return 3;
  }
  if (latency_ms < 500) {
    return 4;
  }
  return 5;
}
