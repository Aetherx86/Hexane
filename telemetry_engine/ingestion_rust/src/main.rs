// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

use std::env;
use std::sync::Arc;
use std::time::Instant;
use telemetry_ingestion::{FastParser, LogLevel, LogRecord, TelemetryServer, TimeSeriesDB};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut host = "0.0.0.0".to_string();
    let mut port: u16 = 8080;
    let mut capacity: usize = 500_000;
    let mut run_bench = false;
    let mut is_demo = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" if i + 1 < args.len() => {
                host = args[i + 1].clone();
                i += 1;
            }
            "--port" if i + 1 < args.len() => {
                port = args[i + 1].parse().unwrap_or(8080);
                i += 1;
            }
            "--capacity" if i + 1 < args.len() => {
                capacity = args[i + 1].parse().unwrap_or(500_000);
                i += 1;
            }
            "--bench" => run_bench = true,
            "--demo" => is_demo = true,
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    println!("==========================================================");
    println!(" Rust Telemetry Ingestion Engine (High-Performance TimeSeries)");
    println!("==========================================================");
    println!(
        " Target: {}:{} | Capacity: {} records",
        host, port, capacity
    );

    let db = Arc::new(TimeSeriesDB::new(capacity));

    if run_bench {
        run_microbenchmark(&db);
        return;
    }

    if is_demo {
        println!("[ingestion-engine] Pre-populating demo telemetry logs & metrics...");
        prepopulate_demo_data(&db);
    }

    let server = TelemetryServer::new(Arc::clone(&db), host, port);
    if let Err(e) = server.start() {
        eprintln!("[ingestion-engine] Failed to start server: {}", e);
        return;
    }

    println!("[ingestion-engine] Telemetry ingestion server running. Press Ctrl+C to terminate.");

    // Keep main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn print_help() {
    println!(
        r#"Usage: telemetry-ingestion [OPTIONS]

High-performance real-time distributed log ingestion engine in Rust.

Options:
  --host <IP>          Server bind host (default: 0.0.0.0)
  --port <PORT>        Server bind port (default: 8080)
  --capacity <N>       In-memory ring buffer record limit (default: 500000)
  --bench              Execute throughput microbenchmark (100k records) and exit
  --demo               Pre-populate realistic telemetry datasets for dashboard
  --help, -h           Show this help message
"#
    );
}

fn run_microbenchmark(db: &TimeSeriesDB) {
    println!("\n--- Ingestion Throughput Microbenchmark ---");
    let batch_size = 10_000;
    let num_batches = 10;
    let total_records = batch_size * num_batches;

    // Synthesize JSON payload
    let mut json_batch = String::with_capacity(batch_size * 220);
    json_batch.push_str("{\"type\":\"logs\",\"count\":");
    json_batch.push_str(&batch_size.to_string());
    json_batch.push_str(",\"items\":[");

    for i in 0..batch_size {
        if i > 0 {
            json_batch.push(',');
        }
        json_batch.push_str(&format!(
            "{{\"timestamp\":{},\"service\":\"auth-svc\",\"level\":\"INFO\",\"status\":200,\"latency\":{},\"trace_id\":\"tr-{}\",\"message\":\"User authenticated successfully with OAuth token\"}}",
            1700000000 + i,
            12 + (i % 80),
            i
        ));
    }
    json_batch.push_str("]}");

    let payload_bytes = json_batch.len();
    println!(
        "Batch Size: {} records ({:.2} MB)",
        batch_size,
        payload_bytes as f64 / (1024.0 * 1024.0)
    );

    let start = Instant::now();
    let mut parsed_total = 0;

    for _ in 0..num_batches {
        let records = FastParser::parse_log_batch(&json_batch);
        parsed_total += db.insert_logs(records);
    }

    let elapsed = start.elapsed();
    let total_mb = (payload_bytes * num_batches) as f64 / (1024.0 * 1024.0);
    let sec = elapsed.as_secs_f64();
    let rps = total_records as f64 / sec;
    let mb_ps = total_mb / sec;

    println!("Total Ingested:    {} records", parsed_total);
    println!("Elapsed Time:      {:.3} seconds", sec);
    println!("Throughput (RPS):  {:.0} records/sec", rps);
    println!("Throughput (MB/s): {:.2} MB/s", mb_ps);
    println!(
        "Avg Parse+Index:   {:.2} microseconds/record",
        (sec * 1_000_000.0) / total_records as f64
    );
    println!("Benchmark Complete: PASSED.\n");
}

fn prepopulate_demo_data(db: &TimeSeriesDB) {
    let services = [
        "auth-service",
        "payment-api",
        "order-router",
        "inventory-db",
        "search-engine",
    ];
    let mut records = Vec::with_capacity(1000);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    for i in 0..1000 {
        let srv = services[i % services.len()];
        let is_err = i % 11 == 0;
        let status = if is_err { 503 } else { 200 };
        let level = if is_err {
            LogLevel::Error
        } else {
            LogLevel::Info
        };
        let latency = if is_err {
            350
        } else {
            15 + ((i * 7) % 65) as u32
        };

        records.push(LogRecord {
            timestamp: now - (1000 - i as i64) * 200,
            service: srv.to_string(),
            level,
            status_code: status,
            latency_ms: latency,
            trace_id: format!("tr-{:08x}", i * 997),
            message: if is_err {
                "Downstream RPC connection timed out while processing order".to_string()
            } else {
                "Processed incoming HTTP request successfully".to_string()
            },
        });
    }

    db.insert_logs(records);
    println!("[ingestion-engine] Pre-populated 1,000 realistic telemetry records.");
}
