// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

import test from 'node:test';
import assert from 'node:assert/strict';

test('Query parameter formatting and filtering logic', () => {
  const filter = {
    service: 'auth-service',
    statusGroup: '5xx',
    minLatency: 100,
    keyword: 'timeout',
  };

  const params = new URLSearchParams();
  if (filter.service) params.set('service', filter.service);
  if (filter.statusGroup === '5xx') {
    params.set('min_status', '500');
    params.set('max_status', '599');
  }
  if (filter.minLatency) params.set('min_latency', filter.minLatency.toString());
  if (filter.keyword) params.set('keyword', filter.keyword);

  assert.equal(params.get('service'), 'auth-service');
  assert.equal(params.get('min_status'), '500');
  assert.equal(params.get('max_status'), '599');
  assert.equal(params.get('min_latency'), '100');
  assert.equal(params.get('keyword'), 'timeout');
});

test('Statistical latency percentile calculations', () => {
  const latencies = [10, 15, 20, 25, 30, 45, 60, 85, 120, 350];
  latencies.sort((a, b) => a - b);
  const n = latencies.length;

  const p50 = latencies[Math.floor(n * 0.5)];
  const p90 = latencies[Math.floor(n * 0.9)];
  const p99 = latencies[Math.min(n - 1, Math.floor(n * 0.99))];

  assert.equal(p50, 45);
  assert.equal(p90, 350);
  assert.equal(p99, 350);
  assert.ok(p50 <= p90);
  assert.ok(p90 <= p99);
});

test('Alert state evaluation logic', () => {
  const rule = {
    metric: 'error_rate',
    operator: '>',
    threshold: 5.0,
  };

  const currentValNormal = 2.1;
  const currentValBreach = 7.4;

  const isTriggeredNormal = currentValNormal > rule.threshold;
  const isTriggeredBreach = currentValBreach > rule.threshold;

  assert.equal(isTriggeredNormal, false);
  assert.equal(isTriggeredBreach, true);
});
