// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

use crate::model::{LogLevel, LogRecord, MetricSample};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct TimeSeriesDB {
    capacity: usize,
    inner: RwLock<DBInner>,
}

struct DBInner {
    logs: Vec<LogRecord>,
    metrics: Vec<MetricSample>,
    // Columnar projection arrays for vector operations
    col_timestamps: Vec<i64>,
    col_latencies: Vec<u32>,
    col_status_codes: Vec<u16>,
    col_levels: Vec<u8>,
    // Inverted index: lowercase token -> list of indices in `logs`
    token_index: HashMap<String, Vec<usize>>,
    // Service index: service_name -> list of indices in `logs`
    service_index: HashMap<String, Vec<usize>>,
    total_ingested_logs: u64,
    total_ingested_metrics: u64,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub service: Option<String>,
    pub min_status: Option<u16>,
    pub max_status: Option<u16>,
    pub min_latency: Option<u32>,
    pub max_latency: Option<u32>,
    pub level: Option<LogLevel>,
    pub keyword: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AggregationResult {
    pub total_logs: usize,
    pub matched_logs: usize,
    pub error_count: usize,
    pub error_rate_pct: f64,
    pub latency_p50: u32,
    pub latency_p90: u32,
    pub latency_p95: u32,
    pub latency_p99: u32,
    pub latency_avg: f64,
    pub latency_max: u32,
    pub latency_min: u32,
    pub sample_records: Vec<LogRecord>,
}

impl TimeSeriesDB {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: RwLock::new(DBInner {
                logs: Vec::with_capacity(capacity.min(65536)),
                metrics: Vec::with_capacity(capacity.min(65536)),
                col_timestamps: Vec::with_capacity(capacity.min(65536)),
                col_latencies: Vec::with_capacity(capacity.min(65536)),
                col_status_codes: Vec::with_capacity(capacity.min(65536)),
                col_levels: Vec::with_capacity(capacity.min(65536)),
                token_index: HashMap::new(),
                service_index: HashMap::new(),
                total_ingested_logs: 0,
                total_ingested_metrics: 0,
            }),
        }
    }

    pub fn insert_logs(&self, records: Vec<LogRecord>) -> usize {
        let mut inner = self.inner.write().unwrap();
        let count = records.len();

        for record in records {
            if inner.logs.len() >= self.capacity {
                // Ring buffer eviction
                inner.logs.remove(0);
                inner.col_timestamps.remove(0);
                inner.col_latencies.remove(0);
                inner.col_status_codes.remove(0);
                inner.col_levels.remove(0);
                // Note: In high-scale engines, index rebuild happens periodically
            }

            let idx = inner.logs.len();
            inner.col_timestamps.push(record.timestamp);
            inner.col_latencies.push(record.latency_ms);
            inner.col_status_codes.push(record.status_code);
            inner.col_levels.push(record.level as u8);

            // Index service
            inner
                .service_index
                .entry(record.service.clone())
                .or_default()
                .push(idx);

            // Index message tokens
            for token in record.message.split_whitespace() {
                let clean = token
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                if !clean.is_empty() {
                    inner.token_index.entry(clean).or_default().push(idx);
                }
            }

            inner.logs.push(record);
            inner.total_ingested_logs += 1;
        }

        count
    }

    pub fn insert_metric(&self, metric: MetricSample) {
        let mut inner = self.inner.write().unwrap();
        if inner.metrics.len() >= self.capacity {
            inner.metrics.remove(0);
        }
        inner.metrics.push(metric);
        inner.total_ingested_metrics += 1;
    }

    pub fn get_latest_metrics(&self, count: usize) -> Vec<MetricSample> {
        let inner = self.inner.read().unwrap();
        let len = inner.metrics.len();
        let start = len.saturating_sub(count);
        inner.metrics[start..len].to_vec()
    }

    pub fn query_logs(&self, filter: &QueryFilter) -> AggregationResult {
        let inner = self.inner.read().unwrap();
        let total_logs = inner.logs.len();

        let mut matched_indices: Vec<usize> = Vec::new();
        let kw_lower = filter.keyword.as_ref().map(|k| k.to_lowercase());

        // 1. Candidate selection
        for i in 0..total_logs {
            let log = &inner.logs[i];

            if let Some(ref srv) = filter.service
                && !log.service.eq_ignore_ascii_case(srv)
            {
                continue;
            }

            if let Some(min_s) = filter.min_status
                && log.status_code < min_s
            {
                continue;
            }

            if let Some(max_s) = filter.max_status
                && log.status_code > max_s
            {
                continue;
            }

            if let Some(min_l) = filter.min_latency
                && log.latency_ms < min_l
            {
                continue;
            }

            if let Some(max_l) = filter.max_latency
                && log.latency_ms > max_l
            {
                continue;
            }

            if let Some(lvl) = filter.level
                && log.level < lvl
            {
                continue;
            }

            if let Some(st) = filter.start_time
                && log.timestamp < st
            {
                continue;
            }

            if let Some(et) = filter.end_time
                && log.timestamp > et
            {
                continue;
            }

            if let Some(ref kw) = kw_lower
                && !log.message.to_lowercase().contains(kw)
                && !log.service.to_lowercase().contains(kw)
            {
                continue;
            }

            matched_indices.push(i);
        }

        // 2. Statistical Aggregations
        let matched_count = matched_indices.len();
        if matched_count == 0 {
            return AggregationResult {
                total_logs,
                matched_logs: 0,
                error_count: 0,
                error_rate_pct: 0.0,
                latency_p50: 0,
                latency_p90: 0,
                latency_p95: 0,
                latency_p99: 0,
                latency_avg: 0.0,
                latency_max: 0,
                latency_min: 0,
                sample_records: Vec::new(),
            };
        }

        let mut latencies: Vec<u32> = matched_indices
            .iter()
            .map(|&idx| inner.col_latencies[idx])
            .collect();
        latencies.sort_unstable();

        let error_count = matched_indices
            .iter()
            .filter(|&&idx| inner.col_status_codes[idx] >= 500)
            .count();

        let error_rate_pct = (error_count as f64 / matched_count as f64) * 100.0;
        let sum_lat: u64 = latencies.iter().map(|&l| l as u64).sum();
        let avg_lat = sum_lat as f64 / matched_count as f64;

        let p50 = latencies[percentile_index(matched_count, 0.50)];
        let p90 = latencies[percentile_index(matched_count, 0.90)];
        let p95 = latencies[percentile_index(matched_count, 0.95)];
        let p99 = latencies[percentile_index(matched_count, 0.99)];
        let min_lat = latencies[0];
        let max_lat = *latencies.last().unwrap();

        let limit = filter.limit.unwrap_or(50);
        let sample_records = matched_indices
            .iter()
            .rev()
            .take(limit)
            .map(|&idx| inner.logs[idx].clone())
            .collect();

        AggregationResult {
            total_logs,
            matched_logs: matched_count,
            error_count,
            error_rate_pct,
            latency_p50: p50,
            latency_p90: p90,
            latency_p95: p95,
            latency_p99: p99,
            latency_avg: avg_lat,
            latency_max: max_lat,
            latency_min: min_lat,
            sample_records,
        }
    }

    /// Extract columnar slices for Carbon query acceleration
    pub fn export_columnar_slice(&self) -> (Vec<i64>, Vec<u32>, Vec<u16>, Vec<u8>) {
        let inner = self.inner.read().unwrap();
        (
            inner.col_timestamps.clone(),
            inner.col_latencies.clone(),
            inner.col_status_codes.clone(),
            inner.col_levels.clone(),
        )
    }

    pub fn stats(&self) -> (u64, u64, usize, usize) {
        let inner = self.inner.read().unwrap();
        (
            inner.total_ingested_logs,
            inner.total_ingested_metrics,
            inner.logs.len(),
            inner.metrics.len(),
        )
    }
}

fn percentile_index(len: usize, pct: f64) -> usize {
    if len == 0 {
        return 0;
    }
    let idx = ((len as f64) * pct).floor() as usize;
    idx.min(len - 1)
}
