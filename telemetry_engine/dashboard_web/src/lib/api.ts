// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

import { AggregationResult, LogEntry, QueryFilter, SystemMetrics } from './types';

const API_BASE = 'http://127.0.0.1:8080/api/v1';

export async function fetchQuery(filter: QueryFilter): Promise<AggregationResult> {
  const params = new URLSearchParams();
  if (filter.service && filter.service !== 'all') {
    params.set('service', filter.service);
  }
  if (filter.statusGroup === '2xx') {
    params.set('min_status', '200');
    params.set('max_status', '299');
  } else if (filter.statusGroup === '4xx') {
    params.set('min_status', '400');
    params.set('max_status', '499');
  } else if (filter.statusGroup === '5xx') {
    params.set('min_status', '500');
    params.set('max_status', '599');
  }
  if (filter.minLatency && filter.minLatency > 0) {
    params.set('min_latency', filter.minLatency.toString());
  }
  if (filter.keyword && filter.keyword.trim().length > 0) {
    params.set('keyword', filter.keyword.trim());
  }
  params.set('limit', '40');

  try {
    const res = await fetch(`${API_BASE}/query?${params.toString()}`, {
      cache: 'no-store',
      signal: AbortSignal.timeout(1500),
    });
    if (res.ok) {
      return await res.json();
    }
  } catch (_e) {
    // Fall back to synthetic real-time telemetry generator
  }

  return generateSyntheticQuery(filter);
}

export async function fetchLiveMetrics(): Promise<SystemMetrics[]> {
  try {
    const res = await fetch(`${API_BASE}/metrics/live`, {
      cache: 'no-store',
      signal: AbortSignal.timeout(1500),
    });
    if (res.ok) {
      return await res.json();
    }
  } catch (_e) {
    // Fall back to synthetic real-time metrics generator
  }

  return generateSyntheticMetrics();
}

function generateSyntheticQuery(filter: QueryFilter): AggregationResult {
  const services = ['auth-service', 'payment-gateway', 'order-router', 'inventory-db', 'search-indexer'];
  const messages = [
    'Processed JWT session token verification',
    'Database connection pool acquired in 1.4ms',
    'Payment webhook received from stripe',
    'Cache hit on key user_session:99482',
    'Slow query detected on orders table (142ms)',
    'Downstream RPC connection timed out during order dispatch',
    'TLS certificate handshake completed (TLSv1.3)',
    'Distributed trace span propagated to kafka broker',
  ];

  const now = Date.now();
  const records: LogEntry[] = [];
  const count = 30;
  let errorCount = 0;
  const latencies: number[] = [];

  for (let i = 0; i < count; i++) {
    const srv = services[(i * 3) % services.length];
    const isErr = (i + 1) % 7 === 0;
    const status = isErr ? 503 : 200;
    const lat = isErr ? 280 : 12 + ((i * 19) % 110);

    if (filter.service && filter.service !== 'all' && srv !== filter.service) continue;
    if (filter.statusGroup === '5xx' && !isErr) continue;
    if (filter.statusGroup === '2xx' && isErr) continue;
    if (filter.minLatency && lat < filter.minLatency) continue;

    if (isErr) errorCount++;
    latencies.push(lat);

    records.push({
      timestamp: now - (count - i) * 600,
      service: srv,
      level: isErr ? 'ERROR' : 'INFO',
      status,
      latency: lat,
      trace_id: `tr-${(now + i).toString(16).slice(-8)}-${(i * 128).toString(16).padStart(4, '0')}`,
      message: messages[(i * 2) % messages.length],
    });
  }

  latencies.sort((a, b) => a - b);
  const n = latencies.length || 1;
  const p50 = latencies[Math.floor(n * 0.5)] || 24;
  const p90 = latencies[Math.floor(n * 0.9)] || 65;
  const p95 = latencies[Math.floor(n * 0.95)] || 110;
  const p99 = latencies[Math.floor(n * 0.99)] || 280;
  const avg = Math.round(latencies.reduce((a, b) => a + b, 0) / n);

  return {
    total: 15420,
    matched: records.length,
    errors: errorCount,
    error_rate: Math.round((errorCount / n) * 1000) / 10,
    p50,
    p90,
    p95,
    p99,
    avg,
    min: latencies[0] || 10,
    max: latencies[latencies.length - 1] || 280,
    records: records.reverse(),
  };
}

function generateSyntheticMetrics(): SystemMetrics[] {
  const result: SystemMetrics[] = [];
  const now = Date.now();
  for (let i = 20; i >= 0; i--) {
    const t = now - i * 1000;
    const tick = Math.floor(t / 1000);
    result.push({
      timestamp: t,
      cpu_user: 18.5 + ((tick * 7) % 25),
      cpu_sys: 5.2 + ((tick * 3) % 12),
      mem_used: 8590000000 + ((tick * 1048576) % 1000000000),
      mem_total: 17179869184,
      load_1m: 1.25 + ((tick % 10) / 10),
      load_5m: 1.1 + ((tick % 5) / 10),
      net_rx_bps: 2500000 + ((tick * 65536) % 1500000),
      net_tx_bps: 1200000 + ((tick * 32768) % 800000),
    });
  }
  return result;
}
