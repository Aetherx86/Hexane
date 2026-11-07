// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

use crate::timeseries_db::TimeSeriesDB;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct CarbonBridge;

impl CarbonBridge {
    pub const CARBON_MAGIC: &[u8; 4] = b"CARB";

    /// Export active columnar dataset to a fast binary file for Carbon analytics
    pub fn export_to_binary_file<P: AsRef<Path>>(
        db: &TimeSeriesDB,
        output_path: P,
    ) -> std::io::Result<usize> {
        let (_ts, latencies, status_codes, levels) = db.export_columnar_slice();
        let count = latencies.len();

        let mut file = File::create(output_path)?;
        file.write_all(Self::CARBON_MAGIC)?;
        file.write_all(&(count as u32).to_le_bytes())?;

        // Write latencies as i32
        for &lat in &latencies {
            file.write_all(&(lat as i32).to_le_bytes())?;
        }

        // Write status_codes as i32
        for &st in &status_codes {
            file.write_all(&(st as i32).to_le_bytes())?;
        }

        // Write levels as i32
        for &lvl in &levels {
            file.write_all(&(lvl as i32).to_le_bytes())?;
        }

        file.flush()?;
        Ok(count)
    }

    /// Read Carbon query results from generated binary or text output
    pub fn run_carbon_query_binary<P: AsRef<Path>>(
        binary_path: P,
        data_path: P,
    ) -> std::io::Result<CarbonQueryResult> {
        let output = Command::new(binary_path.as_ref())
            .arg(data_path.as_ref())
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::other(format!(
                "Carbon query binary failed with exit code {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        Self::parse_carbon_output(&stdout_str)
    }

    pub fn parse_carbon_output(output: &str) -> std::io::Result<CarbonQueryResult> {
        let mut result = CarbonQueryResult::default();

        for line in output.lines() {
            let line = line.trim();
            if let Some(val) = extract_kv(line, "TOTAL_RECORDS:") {
                result.total_records = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "MATCHED_RECORDS:") {
                result.matched_records = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "ERROR_COUNT:") {
                result.error_count = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "ERROR_RATE_BPS:") {
                result.error_rate_bps = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "LATENCY_P50:") {
                result.latency_p50 = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "LATENCY_P95:") {
                result.latency_p95 = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "LATENCY_P99:") {
                result.latency_p99 = val.parse().unwrap_or(0);
            } else if let Some(val) = extract_kv(line, "LATENCY_AVG:") {
                result.latency_avg = val.parse().unwrap_or(0);
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CarbonQueryResult {
    pub total_records: usize,
    pub matched_records: usize,
    pub error_count: usize,
    pub error_rate_bps: u32,
    pub latency_p50: u32,
    pub latency_p95: u32,
    pub latency_p99: u32,
    pub latency_avg: u32,
}

fn extract_kv<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    if let Some(pos) = line.find(key) {
        Some(line[pos + key.len()..].trim())
    } else {
        None
    }
}
