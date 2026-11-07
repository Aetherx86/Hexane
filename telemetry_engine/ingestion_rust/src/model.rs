// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
    Fatal = 6,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" | "WARNING" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            "FATAL" => LogLevel::Fatal,
            _ => LogLevel::Info,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: i64,
    pub service: String,
    pub level: LogLevel,
    pub status_code: u16,
    pub latency_ms: u32,
    pub trace_id: String,
    pub message: String,
}

impl LogRecord {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"timestamp\":{},\"service\":\"{}\",\"level\":\"{}\",\"status\":{},\"latency\":{},\"trace_id\":\"{}\",\"message\":\"{}\"}}",
            self.timestamp,
            self.service,
            self.level.as_str(),
            self.status_code,
            self.latency_ms,
            self.trace_id,
            self.message.replace('"', "\\\"")
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetricSample {
    pub timestamp: i64,
    pub cpu_user_pct: f32,
    pub cpu_sys_pct: f32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub load_avg_1m: f32,
    pub load_avg_5m: f32,
    pub net_rx_bps: u64,
    pub net_tx_bps: u64,
}

impl MetricSample {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"timestamp\":{},\"cpu_user\":{:.2},\"cpu_sys\":{:.2},\"mem_used\":{},\"mem_total\":{},\"load_1m\":{:.2},\"load_5m\":{:.2},\"net_rx_bps\":{},\"net_tx_bps\":{}}}",
            self.timestamp,
            self.cpu_user_pct,
            self.cpu_sys_pct,
            self.mem_used_bytes,
            self.mem_total_bytes,
            self.load_avg_1m,
            self.load_avg_5m,
            self.net_rx_bps,
            self.net_tx_bps
        )
    }
}
