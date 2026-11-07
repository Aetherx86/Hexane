// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
package TelemetryAnalytics api;

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

fn SortLatencies(n: i32) -> i32 {
  var lat: [i32; 64];
  var k: i32 = 0;
  while (k < n) {
    lat[k] = (k * 37 + 13) % 450;
    k = k + 1;
  }

  // Insertion sort for deterministic percentile extraction
  var i: i32 = 1;
  while (i < n) {
    var key: i32 = lat[i];
    var j: i32 = i - 1;
    while (j >= 0 and lat[j] > key) {
      lat[j + 1] = lat[j];
      j = j - 1;
    }
    lat[j + 1] = key;
    i = i + 1;
  }

  var p50_idx: i32 = ComputePercentileIndex(n, 50, 100);
  var p95_idx: i32 = ComputePercentileIndex(n, 95, 100);
  var p99_idx: i32 = ComputePercentileIndex(n, 99, 100);

  var p50: i32 = lat[p50_idx];
  var p95: i32 = lat[p95_idx];
  var p99: i32 = lat[p99_idx];

  // Validate monotonic percentile ordering
  if (p50 <= p95 and p95 <= p99) {
    return 1;
  }
  return 0;
}

fn ExecuteTelemetryRollup(total_records: i32) -> i32 {
  var bucket_counts: [i32; 6];
  var b: i32 = 0;
  while (b < 6) {
    bucket_counts[b] = 0;
    b = b + 1;
  }

  var error_count: i32 = 0;
  var sum_latency: i32 = 0;
  var i: i32 = 0;

  while (i < total_records) {
    var lat: i32 = (i * 17 + 23) % 400;
    var status: i32 = 200;
    if ((i % 13) == 0) {
      status = 503;
      lat = 280;
    }

    if (status >= 500) {
      error_count = error_count + 1;
    }

    sum_latency = sum_latency + lat;
    var b_idx: i32 = LatencyToBucket(lat);
    bucket_counts[b_idx] = bucket_counts[b_idx] + 1;
    i = i + 1;
  }

  var err_bps: i32 = ComputeErrorRateBps(error_count, total_records);
  var avg_lat: i32 = ComputeAvgLatency(sum_latency, total_records);

  // Return success signature if error rate and avg latency are within expected bounds
  if (err_bps > 0 and avg_lat > 0) {
    return 42;
  }
  return 0;
}

fn Main() -> i32 {
  var sort_ok: i32 = SortLatencies(50);
  if (sort_ok != 1) {
    return 1;
  }

  var rollup_res: i32 = ExecuteTelemetryRollup(64);
  if (rollup_res == 42) {
    return 0;
  }
  return 2;
}
