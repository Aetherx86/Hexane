// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

use crate::model::{LogLevel, LogRecord, MetricSample};

pub const MAGIC_BYTES: &[u8; 4] = b"TELM";
pub const HEADER_SIZE: usize = 16;

pub struct FastParser;

impl FastParser {
    /// Zero-copy streaming parser for log batches
    pub fn parse_log_batch(input: &str) -> Vec<LogRecord> {
        let mut records = Vec::new();

        // Check if this is wrapped in {"type":"logs", ..., "items":[...]}
        let items_slice = if let Some(idx) = input.find("\"items\":") {
            let rest = &input[idx + 8..];
            if let Some(start) = rest.find('[') {
                if let Some(end) = rest.rfind(']') {
                    &rest[start + 1..end]
                } else {
                    rest
                }
            } else {
                input
            }
        } else {
            input
        };

        // Parse individual objects within the items slice or newline-delimited stream
        let mut cursor = 0;
        let bytes = items_slice.as_bytes();

        while cursor < bytes.len() {
            // Find start of next JSON object '{'
            while cursor < bytes.len() && bytes[cursor] != b'{' {
                cursor += 1;
            }
            if cursor >= bytes.len() {
                break;
            }

            let obj_start = cursor;
            let mut depth = 0;
            let mut in_str = false;
            let mut escape = false;

            while cursor < bytes.len() {
                let b = bytes[cursor];
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_str = !in_str;
                } else if !in_str {
                    if b == b'{' {
                        depth += 1;
                    } else if b == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            cursor += 1;
                            break;
                        }
                    }
                }
                cursor += 1;
            }

            let obj_str = &items_slice[obj_start..cursor];
            if let Some(rec) = Self::parse_single_log(obj_str) {
                records.push(rec);
            }
        }

        records
    }

    /// Detects and masks malicious code or attack payloads in log messages,
    /// returning the sanitized text and whether an attack pattern was intercepted.
    pub fn sanitize_malicious_payload(msg: &str) -> (String, bool) {
        let lower = msg.to_lowercase();
        let mut sanitized = msg.to_string();
        let mut detected = false;

        let patterns: &[(&str, &str)] = &[
            ("<script", "[MASKED_MALICIOUS_PAYLOAD: XSS_SCRIPT]"),
            ("javascript:", "[MASKED_MALICIOUS_PAYLOAD: JS_URI]"),
            ("onerror=", "[MASKED_MALICIOUS_PAYLOAD: EVENT_HANDLER]"),
            ("onload=", "[MASKED_MALICIOUS_PAYLOAD: EVENT_HANDLER]"),
            ("${jndi:", "[MASKED_MALICIOUS_PAYLOAD: JNDI_EXPLOIT]"),
            ("union select", "[MASKED_MALICIOUS_PAYLOAD: SQL_INJECTION]"),
            ("drop table", "[MASKED_MALICIOUS_PAYLOAD: SQL_DESTRUCTIVE]"),
            ("' or 1=1", "[MASKED_MALICIOUS_PAYLOAD: SQL_TAUTOLOGY]"),
            ("../../", "[MASKED_MALICIOUS_PAYLOAD: PATH_TRAVERSAL]"),
            ("..\\..\\", "[MASKED_MALICIOUS_PAYLOAD: PATH_TRAVERSAL]"),
            ("cmd.exe", "[MASKED_MALICIOUS_PAYLOAD: SHELL_EXEC]"),
            ("powershell -enc", "[MASKED_MALICIOUS_PAYLOAD: POWERSHELL_EXEC]"),
        ];

        for &(pattern, replacement) in patterns {
            if lower.contains(pattern) {
                detected = true;
                let mut start = 0;
                let mut res = String::new();
                let lower_cur = sanitized.to_lowercase();
                while let Some(idx) = lower_cur[start..].find(pattern) {
                    let actual_idx = start + idx;
                    res.push_str(&sanitized[start..actual_idx]);
                    res.push_str(replacement);
                    start = actual_idx + pattern.len();
                }
                res.push_str(&sanitized[start..]);
                sanitized = res;
            }
        }

        (sanitized, detected)
    }

    pub fn parse_single_log(json: &str) -> Option<LogRecord> {
        let timestamp = Self::extract_int(json, "\"timestamp\":").unwrap_or_else(current_millis);
        let service = Self::extract_str(json, "\"service\":").unwrap_or("unknown");
        let level_str = Self::extract_str(json, "\"level\":").unwrap_or("INFO");
        let status = Self::extract_int(json, "\"status\":").unwrap_or(200) as u16;
        let latency = Self::extract_int(json, "\"latency\":").unwrap_or(10) as u32;
        let trace_id = Self::extract_str(json, "\"trace_id\":").unwrap_or("none");
        let message = Self::extract_str(json, "\"message\":").unwrap_or("");

        let (sanitized_msg, is_malicious) = Self::sanitize_malicious_payload(message);
        let mut level: LogLevel = level_str.parse().unwrap_or(LogLevel::Info);
        let final_message = if is_malicious {
            if level < LogLevel::Warn {
                level = LogLevel::Warn;
            }
            format!("[SECURITY WARNING: Malicious code masked] {}", sanitized_msg)
        } else {
            sanitized_msg
        };

        Some(LogRecord {
            timestamp,
            service: service.to_string(),
            level,
            status_code: status,
            latency_ms: latency,
            trace_id: trace_id.to_string(),
            message: final_message,
        })
    }

    pub fn parse_metrics(json: &str) -> Option<MetricSample> {
        let target = if let Some(idx) = json.find("\"sample\":") {
            &json[idx + 9..]
        } else {
            json
        };

        let timestamp = Self::extract_int(target, "\"timestamp\":").unwrap_or_else(current_millis);
        let cpu_user = Self::extract_float(target, "\"cpu_user\":").unwrap_or(0.0);
        let cpu_sys = Self::extract_float(target, "\"cpu_sys\":").unwrap_or(0.0);
        let mem_used = Self::extract_int(target, "\"mem_used\":").unwrap_or(0) as u64;
        let mem_total = Self::extract_int(target, "\"mem_total\":").unwrap_or(0) as u64;
        let load_1m = Self::extract_float(target, "\"load_1m\":").unwrap_or(0.0);
        let load_5m = Self::extract_float(target, "\"load_5m\":").unwrap_or(0.0);
        let net_rx = Self::extract_int(target, "\"net_rx_bps\":").unwrap_or(0) as u64;
        let net_tx = Self::extract_int(target, "\"net_tx_bps\":").unwrap_or(0) as u64;

        Some(MetricSample {
            timestamp,
            cpu_user_pct: cpu_user,
            cpu_sys_pct: cpu_sys,
            mem_used_bytes: mem_used,
            mem_total_bytes: mem_total,
            load_avg_1m: load_1m,
            load_avg_5m: load_5m,
            net_rx_bps: net_rx,
            net_tx_bps: net_tx,
        })
    }

    /// Fast extraction of string values without allocation
    fn extract_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
        let pos = json.find(key)?;
        let start_pos = pos + key.len();
        let bytes = json.as_bytes();

        let mut quote_start = None;
        let mut i = start_pos;
        while i < bytes.len() {
            if bytes[i] == b'"' {
                quote_start = Some(i + 1);
                break;
            }
            i += 1;
        }

        let start = quote_start?;
        i = start;
        let mut escape = false;
        while i < bytes.len() {
            if escape {
                escape = false;
            } else if bytes[i] == b'\\' {
                escape = true;
            } else if bytes[i] == b'"' {
                return Some(&json[start..i]);
            }
            i += 1;
        }

        None
    }

    /// Fast extraction of integer values
    fn extract_int(json: &str, key: &str) -> Option<i64> {
        let pos = json.find(key)?;
        let mut i = pos + key.len();
        let bytes = json.as_bytes();

        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b':') {
            i += 1;
        }

        let num_start = i;
        let mut is_neg = false;
        if i < bytes.len() && bytes[i] == b'-' {
            is_neg = true;
            i += 1;
        }

        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }

        if i == num_start {
            return None;
        }

        let s = &json[num_start..i];
        s.parse::<i64>()
            .ok()
            .map(|v| if is_neg { -v.abs() } else { v })
    }

    /// Fast extraction of float values
    fn extract_float(json: &str, key: &str) -> Option<f32> {
        let pos = json.find(key)?;
        let mut i = pos + key.len();
        let bytes = json.as_bytes();

        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b':') {
            i += 1;
        }

        let num_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'-')
        {
            i += 1;
        }

        if i == num_start {
            return None;
        }

        json[num_start..i].parse::<f32>().ok()
    }
}

pub fn current_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
