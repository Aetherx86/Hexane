// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

use crate::fast_parser::FastParser;
use crate::timeseries_db::{QueryFilter, TimeSeriesDB};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub struct TelemetryServer {
    db: Arc<TimeSeriesDB>,
    host: String,
    port: u16,
    running: Arc<AtomicBool>,
}

impl TelemetryServer {
    pub fn new(db: Arc<TimeSeriesDB>, host: String, port: u16) -> Self {
        Self {
            db,
            host,
            port,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> std::io::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)?;
        listener.set_nonblocking(true)?;
        self.running.store(true, Ordering::SeqCst);

        println!("[ingestion-engine] Listening on http://{}", addr);

        let running_clone = Arc::clone(&self.running);
        let db_clone = Arc::clone(&self.db);

        thread::spawn(move || {
            while running_clone.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let db = Arc::clone(&db_clone);
                        thread::spawn(move || {
                            let _ = handle_connection(stream, db);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("[ingestion-engine] Accept error: {}", e);
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn handle_connection(mut stream: TcpStream, db: Arc<TimeSeriesDB>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let uri = parts[1];

    let mut content_length = 0;
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if line.to_lowercase().starts_with("content-length:")
            && let Some(val_str) = line.split(':').nth(1)
        {
            content_length = val_str.trim().parse::<usize>().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body_str = String::from_utf8_lossy(&body);

    let (status_code, response_body) = if method == "OPTIONS" {
        (200, String::new())
    } else if uri.starts_with("/api/v1/logs") && method == "POST" {
        let records = FastParser::parse_log_batch(&body_str);
        let count = db.insert_logs(records);
        (200, format!("{{\"status\":\"ok\",\"ingested\":{}}}", count))
    } else if uri.starts_with("/api/v1/metrics") && method == "POST" {
        if let Some(metric) = FastParser::parse_metrics(&body_str) {
            db.insert_metric(metric);
            (200, "{\"status\":\"ok\",\"ingested\":1}".to_string())
        } else {
            (400, "{\"error\":\"invalid metric json\"}".to_string())
        }
    } else if uri.starts_with("/api/v1/query") {
        let filter = parse_query_params(uri);
        let res = db.query_logs(&filter);
        let sample_json: Vec<String> = res.sample_records.iter().map(|r| r.to_json()).collect();

        let json = format!(
            "{{\"total\":{},\"matched\":{},\"errors\":{},\"error_rate\":{:.2},\"p50\":{},\"p90\":{},\"p95\":{},\"p99\":{},\"avg\":{:.1},\"min\":{},\"max\":{},\"records\":[{}]}}",
            res.total_logs,
            res.matched_logs,
            res.error_count,
            res.error_rate_pct,
            res.latency_p50,
            res.latency_p90,
            res.latency_p95,
            res.latency_p99,
            res.latency_avg,
            res.latency_min,
            res.latency_max,
            sample_json.join(",")
        );
        (200, json)
    } else if uri.starts_with("/api/v1/metrics/live") {
        let metrics = db.get_latest_metrics(30);
        let m_json: Vec<String> = metrics.iter().map(|m| m.to_json()).collect();
        (200, format!("[{}]", m_json.join(",")))
    } else if uri.starts_with("/api/v1/stats") {
        let (tot_logs, tot_metrics, cur_logs, cur_metrics) = db.stats();
        let json = format!(
            "{{\"total_logs\":{},\"total_metrics\":{},\"cached_logs\":{},\"cached_metrics\":{}}}",
            tot_logs, tot_metrics, cur_logs, cur_metrics
        );
        (200, json)
    } else if uri.starts_with("/health") {
        (200, "{\"status\":\"healthy\"}".to_string())
    } else {
        (404, "{\"error\":\"not found\"}".to_string())
    };

    let response = format!(
        "HTTP/1.1 {} OK\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        status_code,
        response_body.len(),
        response_body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn parse_query_params(uri: &str) -> QueryFilter {
    let mut filter = QueryFilter::default();
    if let Some(pos) = uri.find('?') {
        let query_str = &uri[pos + 1..];
        for pair in query_str.split('&') {
            let mut kv = pair.split('=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                let v = url_decode(v);
                match k {
                    "service" => filter.service = Some(v),
                    "keyword" => filter.keyword = Some(v),
                    "min_status" => filter.min_status = v.parse().ok(),
                    "max_status" => filter.max_status = v.parse().ok(),
                    "min_latency" => filter.min_latency = v.parse().ok(),
                    "max_latency" => filter.max_latency = v.parse().ok(),
                    "limit" => filter.limit = v.parse().ok(),
                    _ => {}
                }
            }
        }
    }
    filter
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex = format!("{}{}", h1, h2);
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                result.push(b as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}
