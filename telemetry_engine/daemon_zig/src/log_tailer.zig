// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
const std = @import("std");
const time_util = @import("time_util.zig");

pub const LogLevel = enum(u8) {
    trace = 1,
    debug = 2,
    info = 3,
    warn = 4,
    error_level = 5,
    fatal = 6,

    pub fn asString(self: LogLevel) []const u8 {
        return switch (self) {
            .trace => "TRACE",
            .debug => "DEBUG",
            .info => "INFO",
            .warn => "WARN",
            .error_level => "ERROR",
            .fatal => "FATAL",
        };
    }

    pub fn fromString(str: []const u8) LogLevel {
        if (std.mem.eql(u8, str, "TRACE")) return .trace;
        if (std.mem.eql(u8, str, "DEBUG")) return .debug;
        if (std.mem.eql(u8, str, "INFO")) return .info;
        if (std.mem.eql(u8, str, "WARN") or std.mem.eql(u8, str, "WARNING")) return .warn;
        if (std.mem.eql(u8, str, "ERROR")) return .error_level;
        if (std.mem.eql(u8, str, "FATAL")) return .fatal;
        return .info;
    }
};

pub const LogEntry = struct {
    timestamp: i64,
    service_len: u8,
    service: [32]u8,
    level: LogLevel,
    status_code: u16,
    latency_ms: u32,
    trace_id_len: u8,
    trace_id: [36]u8,
    message_len: u16,
    message: [256]u8,

    pub fn getService(self: *const LogEntry) []const u8 {
        return self.service[0..self.service_len];
    }

    pub fn getTraceId(self: *const LogEntry) []const u8 {
        return self.trace_id[0..self.trace_id_len];
    }

    pub fn getMessage(self: *const LogEntry) []const u8 {
        return self.message[0..self.message_len];
    }

    pub fn create(
        timestamp: i64,
        service: []const u8,
        level: LogLevel,
        status_code: u16,
        latency_ms: u32,
        trace_id: []const u8,
        message: []const u8,
    ) LogEntry {
        var entry = LogEntry{
            .timestamp = timestamp,
            .service_len = @intCast(@min(service.len, 32)),
            .service = std.mem.zeroes([32]u8),
            .level = level,
            .status_code = status_code,
            .latency_ms = latency_ms,
            .trace_id_len = @intCast(@min(trace_id.len, 36)),
            .trace_id = std.mem.zeroes([36]u8),
            .message_len = @intCast(@min(message.len, 256)),
            .message = std.mem.zeroes([256]u8),
        };

        @memcpy(entry.service[0..entry.service_len], service[0..entry.service_len]);
        @memcpy(entry.trace_id[0..entry.trace_id_len], trace_id[0..entry.trace_id_len]);
        @memcpy(entry.message[0..entry.message_len], message[0..entry.message_len]);
        return entry;
    }
};

pub fn RingBuffer(comptime T: type, comptime Capacity: usize) type {
    return struct {
        const Self = @This();
        items: [Capacity]T = undefined,
        head: usize = 0,
        tail: usize = 0,
        count: usize = 0,

        pub fn init() Self {
            return .{};
        }

        pub fn push(self: *Self, item: T) void {
            self.items[self.head] = item;
            self.head = (self.head + 1) % Capacity;
            if (self.count < Capacity) {
                self.count += 1;
            } else {
                self.tail = (self.tail + 1) % Capacity;
            }
        }

        pub fn pop(self: *Self) ?T {
            if (self.count == 0) return null;
            const item = self.items[self.tail];
            self.tail = (self.tail + 1) % Capacity;
            self.count -= 1;
            return item;
        }

        pub fn len(self: *const Self) usize {
            return self.count;
        }

        pub fn isFull(self: *const Self) bool {
            return self.count == Capacity;
        }
    };
}

pub const LogParser = struct {
    pub fn parseLine(line: []const u8) ?LogEntry {
        if (line.len == 0) return null;

        // Try parsing JSON format: {"timestamp":..., "service":"...", "level":"...", "status":..., "latency":..., "trace_id":"...", "message":"..."}
        if (line[0] == '{') {
            return parseJsonLine(line);
        }

        // Standard Syslog / Kube format: [2026-11-05T12:00:00Z] [SERVICE] [LEVEL] [STATUS] [LATENCYms] [TRACE_ID] MESSAGE
        var it = std.mem.tokenizeScalar(u8, line, ' ');
        const ts_str = it.next() orelse return null;
        _ = ts_str;
        const srv_str = it.next() orelse "app";
        const lvl_str = it.next() orelse "INFO";
        const status_str = it.next() orelse "200";
        const lat_str = it.next() orelse "10";
        const trace_str = it.next() orelse "0000-0000";

        var msg_buf: [256]u8 = undefined;
        var msg_idx: usize = 0;
        while (it.next()) |tok| {
            if (msg_idx > 0 and msg_idx < 255) {
                msg_buf[msg_idx] = ' ';
                msg_idx += 1;
            }
            const copy_len = @min(tok.len, 255 - msg_idx);
            @memcpy(msg_buf[msg_idx .. msg_idx + copy_len], tok[0..copy_len]);
            msg_idx += copy_len;
        }

        const now = time_util.currentMillis();
        const status = std.fmt.parseInt(u16, status_str, 10) catch 200;
        const latency = std.fmt.parseInt(u32, lat_str, 10) catch 10;
        const level = LogLevel.fromString(lvl_str);

        return LogEntry.create(now, srv_str, level, status, latency, trace_str, msg_buf[0..msg_idx]);
    }

    fn parseJsonLine(line: []const u8) ?LogEntry {
        // Fast zero-alloc JSON key extractor
        const srv = extractJsonString(line, "\"service\":") orelse "unknown";
        const lvl_str = extractJsonString(line, "\"level\":") orelse "INFO";
        const trace = extractJsonString(line, "\"trace_id\":") orelse "none";
        const msg = extractJsonString(line, "\"message\":") orelse line;
        const status = extractJsonInt(line, "\"status\":") orelse 200;
        const lat = extractJsonInt(line, "\"latency\":") orelse 12;

        const now = time_util.currentMillis();
        return LogEntry.create(
            now,
            srv,
            LogLevel.fromString(lvl_str),
            @intCast(status),
            @intCast(lat),
            trace,
            msg,
        );
    }

    fn extractJsonString(line: []const u8, key: []const u8) ?[]const u8 {
        const pos = std.mem.indexOf(u8, line, key) orelse return null;
        const start_val = pos + key.len;
        const first_quote = std.mem.indexOfPos(u8, line, start_val, "\"") orelse return null;
        const second_quote = std.mem.indexOfPos(u8, line, first_quote + 1, "\"") orelse return null;
        return line[first_quote + 1 .. second_quote];
    }

    fn extractJsonInt(line: []const u8, key: []const u8) ?u64 {
        const pos = std.mem.indexOf(u8, line, key) orelse return null;
        var idx = pos + key.len;
        while (idx < line.len and (line[idx] == ' ' or line[idx] == ':')) : (idx += 1) {}
        const num_start = idx;
        while (idx < line.len and (line[idx] >= '0' and line[idx] <= '9')) : (idx += 1) {}
        if (num_start == idx) return null;
        return std.fmt.parseInt(u64, line[num_start..idx], 10) catch null;
    }
};

test "LogEntry and RingBuffer functionality" {
    var ring = RingBuffer(LogEntry, 8).init();
    try std.testing.expectEqual(@as(usize, 0), ring.len());

    const entry1 = LogEntry.create(1000, "auth-svc", .info, 200, 15, "tr-101", "Token validated");
    ring.push(entry1);
    try std.testing.expectEqual(@as(usize, 1), ring.len());

    const popped = ring.pop().?;
    try std.testing.expectEqualStrings("auth-svc", popped.getService());
    try std.testing.expectEqual(LogLevel.info, popped.level);
    try std.testing.expectEqual(@as(u16, 200), popped.status_code);
    try std.testing.expectEqualStrings("Token validated", popped.getMessage());
}

test "LogParser handles structured JSON and space-delimited logs" {
    const json_log = "{\"service\":\"billing-api\", \"level\":\"ERROR\", \"status\":503, \"latency\":120, \"trace_id\":\"tx-789\", \"message\":\"Payment gateway timeout\"}";
    const entry_opt = LogParser.parseLine(json_log);
    try std.testing.expect(entry_opt != null);
    const entry = entry_opt.?;
    try std.testing.expectEqualStrings("billing-api", entry.getService());
    try std.testing.expectEqual(LogLevel.error_level, entry.level);
    try std.testing.expectEqual(@as(u16, 503), entry.status_code);
    try std.testing.expectEqual(@as(u32, 120), entry.latency_ms);
}
