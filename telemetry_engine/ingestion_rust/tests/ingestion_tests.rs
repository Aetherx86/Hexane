// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

use telemetry_ingestion::{
    CarbonBridge, FastParser, LogLevel, LogRecord, QueryFilter, TimeSeriesDB,
};

#[test]
fn test_fast_parser_json_log_batch() {
    let json = r#"{"type":"logs","count":2,"items":[{"timestamp":1700000000,"service":"auth","level":"INFO","status":200,"latency":14,"trace_id":"tr-1","message":"Login successful"},{"timestamp":1700000001,"service":"billing","level":"ERROR","status":500,"latency":140,"trace_id":"tr-2","message":"Card declined"}]}"#;

    let records = FastParser::parse_log_batch(json);
    assert_eq!(records.len(), 2);

    assert_eq!(records[0].service, "auth");
    assert_eq!(records[0].level, LogLevel::Info);
    assert_eq!(records[0].status_code, 200);
    assert_eq!(records[0].latency_ms, 14);

    assert_eq!(records[1].service, "billing");
    assert_eq!(records[1].level, LogLevel::Error);
    assert_eq!(records[1].status_code, 500);
    assert_eq!(records[1].latency_ms, 140);
}

#[test]
fn test_fast_parser_metrics() {
    let json = r#"{"type":"metrics","sample":{"timestamp":1700000000,"cpu_user":18.5,"cpu_sys":6.2,"mem_used":4000000,"mem_total":8000000,"load_1m":1.15,"load_5m":1.05,"net_rx_bps":50000,"net_tx_bps":25000}}"#;

    let sample = FastParser::parse_metrics(json).expect("Failed to parse metrics");
    assert_eq!(sample.timestamp, 1700000000);
    assert!((sample.cpu_user_pct - 18.5).abs() < 0.01);
    assert_eq!(sample.mem_used_bytes, 4000000);
}

#[test]
fn test_malicious_code_masking_and_security_warning() {
    let raw_log = r#"{"timestamp":1700000000,"service":"auth-service","level":"INFO","status":200,"latency":15,"trace_id":"tr-malicious","message":"User login attempt with payload <script>alert('xss')</script> and ' OR 1=1 --"}"#;

    let record = FastParser::parse_single_log(raw_log).expect("should parse log");

    // Must mask malicious script and SQL injection
    assert!(!record.message.contains("<script"));
    assert!(!record.message.contains("' OR 1=1"));
    assert!(record.message.contains("[MASKED_MALICIOUS_PAYLOAD: XSS_SCRIPT]"));
    assert!(record.message.contains("[MASKED_MALICIOUS_PAYLOAD: SQL_TAUTOLOGY]"));

    // Must elevate to WARN and prepend security warning
    assert_eq!(record.level, LogLevel::Warn);
    assert!(record.message.starts_with("[SECURITY WARNING: Malicious code masked]"));
}

#[test]
fn test_timeseries_db_insertion_and_query() {
    let db = TimeSeriesDB::new(1000);

    let mut logs = Vec::new();
    for i in 0..100 {
        let is_err = i % 10 == 0;
        logs.push(LogRecord {
            timestamp: 1000 + i,
            service: if i % 2 == 0 {
                "payment".to_string()
            } else {
                "orders".to_string()
            },
            level: if is_err {
                LogLevel::Error
            } else {
                LogLevel::Info
            },
            status_code: if is_err { 500 } else { 200 },
            latency_ms: (10 + i * 2) as u32,
            trace_id: format!("tr-{}", i),
            message: if is_err {
                "Connection timeout".to_string()
            } else {
                "Handshake completed".to_string()
            },
        });
    }

    db.insert_logs(logs);

    // Query 1: Filter by service
    let res_payment = db.query_logs(&QueryFilter {
        service: Some("payment".to_string()),
        ..Default::default()
    });
    assert_eq!(res_payment.matched_logs, 50);

    // Query 2: Filter by error status
    let res_errors = db.query_logs(&QueryFilter {
        min_status: Some(500),
        ..Default::default()
    });
    assert_eq!(res_errors.matched_logs, 10);
    assert_eq!(res_errors.error_count, 10);
    assert!((res_errors.error_rate_pct - 100.0).abs() < 0.01);

    // Query 3: Statistical Percentiles
    let res_all = db.query_logs(&QueryFilter::default());
    assert_eq!(res_all.matched_logs, 100);
    assert!(res_all.latency_p50 > 0);
    assert!(res_all.latency_p99 >= res_all.latency_p50);
}

#[test]
fn test_carbon_bridge_binary_export() {
    let db = TimeSeriesDB::new(100);
    let mut logs = Vec::new();
    for i in 0..20 {
        logs.push(LogRecord {
            timestamp: 2000 + i,
            service: "cache".to_string(),
            level: LogLevel::Info,
            status_code: 200,
            latency_ms: 15 + i as u32,
            trace_id: format!("tr-{}", i),
            message: "Get key".to_string(),
        });
    }
    db.insert_logs(logs);

    let temp_file = std::env::temp_dir().join("test_carbon_export.bin");
    let written = CarbonBridge::export_to_binary_file(&db, &temp_file).unwrap();
    assert_eq!(written, 20);
    assert!(temp_file.exists());
    let _ = std::fs::remove_file(temp_file);
}
