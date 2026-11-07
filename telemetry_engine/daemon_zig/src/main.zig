// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
const std = @import("std");
pub const time_util = @import("time_util.zig");
pub const metrics = @import("metrics.zig");
pub const log_tailer = @import("log_tailer.zig");
pub const packet_encoder = @import("packet_encoder.zig");
pub const client = @import("client.zig");

pub fn main(init: std.process.Init) !void {
    const raw_args = init.minimal.args.vector;

    var host: []const u8 = "127.0.0.1";
    var port: u16 = 8080;
    var interval_ms: u64 = 500;
    var is_demo: bool = false;
    var run_once: bool = false;
    var stdout_only: bool = false;
    var max_iterations: usize = 0;

    var i: usize = 1;
    while (i < raw_args.len) : (i += 1) {
        const arg = std.mem.span(raw_args[i]);
        if (std.mem.eql(u8, arg, "--host") and i + 1 < raw_args.len) {
            i += 1;
            host = std.mem.span(raw_args[i]);
        } else if (std.mem.eql(u8, arg, "--port") and i + 1 < raw_args.len) {
            i += 1;
            port = try std.fmt.parseInt(u16, std.mem.span(raw_args[i]), 10);
        } else if (std.mem.eql(u8, arg, "--interval-ms") and i + 1 < raw_args.len) {
            i += 1;
            interval_ms = try std.fmt.parseInt(u64, std.mem.span(raw_args[i]), 10);
        } else if (std.mem.eql(u8, arg, "--demo")) {
            is_demo = true;
        } else if (std.mem.eql(u8, arg, "--once")) {
            run_once = true;
        } else if (std.mem.eql(u8, arg, "--stdout")) {
            stdout_only = true;
        } else if (std.mem.eql(u8, arg, "--count") and i + 1 < raw_args.len) {
            i += 1;
            max_iterations = try std.fmt.parseInt(usize, std.mem.span(raw_args[i]), 10);
        } else if (std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h")) {
            printHelp();
            return;
        }
    }

    time_util.printOut("[telemetry-daemon] Starting Zig Telemetry Daemon (Kubernetes Node Agent)\n", .{});
    time_util.printOut("[telemetry-daemon] Target: {s}:{d} | Interval: {d}ms | Mode: {s}\n", .{
        host,
        port,
        interval_ms,
        if (is_demo) "demo-generator" else "node-scraper",
    });

    var collector = metrics.MetricsCollector.init();
    var ing_client = client.IngestionClient.init(host, port);
    defer ing_client.disconnect();

    var json_buf: [32768]u8 = undefined;
    var batch_ring = log_tailer.RingBuffer(log_tailer.LogEntry, 64).init();
    var iter_count: usize = 0;
    var total_logs_sent: usize = 0;
    var total_metrics_sent: usize = 0;

    const services = [_][]const u8{ "auth-service", "payment-gateway", "order-router", "inventory-db", "search-indexer" };
    const messages = [_][]const u8{
        "Processed JWT session token verification",
        "Database connection pool acquired in 1.4ms",
        "Payment webhook received from stripe",
        "Cache hit on key user_session:99482",
        "Slow query detected on orders table (142ms)",
        "Connection reset by peer during downstream call",
    };

    while (true) {
        iter_count += 1;

        // 1. Collect host/node system metrics
        const sample = collector.collect();
        const metrics_len = try packet_encoder.PacketEncoder.encodeMetricsJson(sample, &json_buf);
        const metrics_json = json_buf[0..metrics_len];

        if (stdout_only) {
            _ = std.posix.system.write(1, metrics_json.ptr, metrics_json.len);
        } else {
            ing_client.sendHttpPost("/api/v1/metrics", metrics_json) catch {
                // In demo/offline mode, quietly proceed
            };
        }
        total_metrics_sent += 1;

        // 2. Collect or simulate structured logs
        const log_batch_count: usize = if (is_demo) 5 else 2;
        var batch_entries: [16]log_tailer.LogEntry = undefined;
        var batch_idx: usize = 0;

        var k: usize = 0;
        while (k < log_batch_count and batch_idx < 16) : (k += 1) {
            const srv = services[(iter_count + k) % services.len];
            const msg = messages[(iter_count * 3 + k) % messages.len];
            const is_err = (iter_count + k) % 7 == 0;
            const status: u16 = if (is_err) 500 else 200;
            const lvl: log_tailer.LogLevel = if (is_err) .error_level else .info;
            const latency: u32 = if (is_err) 240 else 8 + @as(u32, @intCast((iter_count * 13 + k * 17) % 65));

            var trace_buf: [36]u8 = undefined;
            const trace_str = try std.fmt.bufPrint(&trace_buf, "tr-{x:0>8}-{x:0>4}", .{ iter_count * 1000 + k, k * 256 });

            const entry = log_tailer.LogEntry.create(
                time_util.currentMillis(),
                srv,
                lvl,
                status,
                latency,
                trace_str,
                msg,
            );

            batch_ring.push(entry);
            batch_entries[batch_idx] = entry;
            batch_idx += 1;
        }

        var log_buf: [32768]u8 = undefined;
        const logs_len = try packet_encoder.PacketEncoder.encodeLogsJson(batch_entries[0..batch_idx], &log_buf);
        const logs_json = log_buf[0..logs_len];

        if (stdout_only) {
            _ = std.posix.system.write(1, logs_json.ptr, logs_json.len);
        } else {
            ing_client.sendHttpPost("/api/v1/logs", logs_json) catch {
                // In demo/offline mode, quietly proceed
            };
        }
        total_logs_sent += batch_idx;

        if (iter_count % 10 == 0 or run_once) {
            time_util.printOut("[telemetry-daemon] Batch #{d}: Sent {d} logs, {d} metric samples (CPU: {d:.1}%, Mem: {d}MB)\n", .{
                iter_count,
                total_logs_sent,
                total_metrics_sent,
                sample.cpu_user_pct + sample.cpu_sys_pct,
                sample.mem_used_bytes / (1024 * 1024),
            });
        }

        if (run_once or (max_iterations > 0 and iter_count >= max_iterations)) {
            time_util.printOut("[telemetry-daemon] Completed execution successfully.\n", .{});
            break;
        }

        time_util.sleepMillis(interval_ms);
    }
}

fn printHelp() void {
    time_util.printOut(
        \\Usage: telemetry-agent [OPTIONS]
        \\
        \\Lightweight Kubernetes DaemonSet telemetry scraper agent in Zig.
        \\
        \\Options:
        \\  --host <IP/HOST>       Ingestion server host (default: 127.0.0.1)
        \\  --port <PORT>          Ingestion server port (default: 8080)
        \\  --interval-ms <MS>     Scrape and flush interval in milliseconds (default: 500)
        \\  --demo                 Enable demo synthetic log stream generator
        \\  --once                 Run a single scrape and emit cycle, then exit
        \\  --stdout               Output scraped JSON batches directly to stdout
        \\  --count <N>            Run exactly N iterations, then exit
        \\  --help, -h             Show this help message
        \\
    , .{});
}

test {
    _ = @import("time_util.zig");
    _ = @import("metrics.zig");
    _ = @import("log_tailer.zig");
    _ = @import("packet_encoder.zig");
    _ = @import("client.zig");
}
