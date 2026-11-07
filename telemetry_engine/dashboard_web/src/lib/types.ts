// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

export type LogLevel = 'TRACE' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'FATAL';

export interface LogEntry {
  timestamp: number;
  service: string;
  level: LogLevel;
  status: number;
  latency: number;
  trace_id: string;
  message: string;
}

export interface SystemMetrics {
  timestamp: number;
  cpu_user: number;
  cpu_sys: number;
  mem_used: number;
  mem_total: number;
  load_1m: number;
  load_5m: number;
  net_rx_bps: number;
  net_tx_bps: number;
}

export interface QueryFilter {
  service?: string;
  statusGroup?: 'ALL' | '2xx' | '4xx' | '5xx';
  minLatency?: number;
  keyword?: string;
  level?: LogLevel;
  timeRange?: '5m' | '15m' | '1h' | '24h';
}

export interface AggregationResult {
  total: number;
  matched: number;
  errors: number;
  error_rate: number;
  p50: number;
  p90: number;
  p95: number;
  p99: number;
  avg: number;
  min: number;
  max: number;
  records: LogEntry[];
}

export type AlertSeverity = 'INFO' | 'WARNING' | 'CRITICAL';
export type AlertState = 'OK' | 'PENDING' | 'FIRING';

export interface AlertRule {
  id: string;
  name: string;
  metric: 'error_rate' | 'p99_latency' | 'cpu_user' | 'load_1m';
  operator: '>' | '<' | '>=';
  threshold: number;
  duration_secs: number;
  severity: AlertSeverity;
  state: AlertState;
  last_triggered?: number;
}

export interface Incident {
  id: string;
  rule_id: string;
  rule_name: string;
  severity: AlertSeverity;
  triggered_at: number;
  value: number;
  threshold: number;
  status: 'ACTIVE' | 'ACKNOWLEDGED' | 'RESOLVED';
}
