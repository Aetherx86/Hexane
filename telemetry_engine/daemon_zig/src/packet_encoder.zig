// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
const std = @import("std");
const log_tailer = @import("log_tailer.zig");
const metrics = @import("metrics.zig");

pub const MAGIC_BYTES = [4]u8{ 'T', 'E', 'L', 'M' };
pub const PROTOCOL_VERSION: u8 = 1;

pub const PacketType = enum(u8) {
    logs_batch = 1,
    metrics_batch = 2,
    heartbeat = 3,
};

pub const PacketHeader = extern struct {
    magic: [4]u8,
    version: u8,
    packet_type: u8,
    item_count: u16,
    payload_len: u32,
    checksum: u32,
};

pub const BufferWriter = struct {
    buf: []u8,
    pos: usize = 0,

    pub fn writeAll(self: *BufferWriter, slice: []const u8) !void {
        if (self.pos + slice.len > self.buf.len) return error.NoSpaceLeft;
        @memcpy(self.buf[self.pos .. self.pos + slice.len], slice);
        self.pos += slice.len;
    }

    pub fn writeByte(self: *BufferWriter, byte: u8) !void {
        if (self.pos >= self.buf.len) return error.NoSpaceLeft;
        self.buf[self.pos] = byte;
        self.pos += 1;
    }

    pub fn print(self: *BufferWriter, comptime fmt: []const u8, args: anytype) !void {
        var temp: [256]u8 = undefined;
        const formatted = try std.fmt.bufPrint(&temp, fmt, args);
        try self.writeAll(formatted);
    }
};

pub const PacketEncoder = struct {
    pub fn encodeLogsJson(entries: []const log_tailer.LogEntry, buffer: []u8) !usize {
        var w = BufferWriter{ .buf = buffer };

        try w.writeAll("{\"type\":\"logs\",\"count\":");
        try w.print("{d}", .{entries.len});
        try w.writeAll(",\"items\":[");

        for (entries, 0..) |e, i| {
            if (i > 0) try w.writeByte(',');
            try w.writeAll("{\"timestamp\":");
            try w.print("{d}", .{e.timestamp});
            try w.writeAll(",\"service\":\"");
            try w.writeAll(e.getService());
            try w.writeAll("\",\"level\":\"");
            try w.writeAll(e.level.asString());
            try w.writeAll("\",\"status\":");
            try w.print("{d}", .{e.status_code});
            try w.writeAll(",\"latency\":");
            try w.print("{d}", .{e.latency_ms});
            try w.writeAll(",\"trace_id\":\"");
            try w.writeAll(e.getTraceId());
            try w.writeAll("\",\"message\":\"");

            // Escape message quotes
            for (e.getMessage()) |ch| {
                if (ch == '"') {
                    try w.writeAll("\\\"");
                } else if (ch == '\n') {
                    try w.writeAll("\\n");
                } else {
                    try w.writeByte(ch);
                }
            }
            try w.writeAll("\"}");
        }

        try w.writeAll("]}\n");
        return w.pos;
    }

    pub fn encodeMetricsJson(sample: metrics.SystemMetrics, buffer: []u8) !usize {
        var w = BufferWriter{ .buf = buffer };

        try w.writeAll("{\"type\":\"metrics\",\"sample\":{");
        try w.print("\"timestamp\":{d},", .{sample.timestamp});
        try w.print("\"cpu_user\":{d:.2},", .{sample.cpu_user_pct});
        try w.print("\"cpu_sys\":{d:.2},", .{sample.cpu_sys_pct});
        try w.print("\"mem_used\":{d},", .{sample.mem_used_bytes});
        try w.print("\"mem_total\":{d},", .{sample.mem_total_bytes});
        try w.print("\"load_1m\":{d:.2},", .{sample.load_avg_1m});
        try w.print("\"load_5m\":{d:.2},", .{sample.load_avg_5m});
        try w.print("\"net_rx_bps\":{d},", .{sample.net_rx_bps});
        try w.print("\"net_tx_bps\":{d}", .{sample.net_tx_bps});
        try w.writeAll("}}\n");

        return w.pos;
    }

    pub fn encodeBinaryFrame(packet_type: PacketType, payload: []const u8, buffer: []u8) !usize {
        const header_size = @sizeOf(PacketHeader);
        if (buffer.len < header_size + payload.len) return error.BufferTooSmall;

        // Simple crc-like checksum
        var sum: u32 = 0;
        for (payload) |b| {
            sum = sum *% 31 +% b;
        }

        const header = PacketHeader{
            .magic = MAGIC_BYTES,
            .version = PROTOCOL_VERSION,
            .packet_type = @intFromEnum(packet_type),
            .item_count = 1,
            .payload_len = @intCast(payload.len),
            .checksum = sum,
        };

        const header_bytes = std.mem.asBytes(&header);
        @memcpy(buffer[0..header_size], header_bytes);
        @memcpy(buffer[header_size .. header_size + payload.len], payload);

        return header_size + payload.len;
    }
};

test "PacketEncoder generates valid JSON and Binary frames" {
    var buf: [4096]u8 = undefined;

    const sample = metrics.SystemMetrics{
        .timestamp = 1700000000,
        .cpu_user_pct = 25.5,
        .cpu_sys_pct = 5.2,
        .mem_used_bytes = 4096,
        .mem_total_bytes = 8192,
        .load_avg_1m = 1.05,
        .load_avg_5m = 0.95,
        .net_rx_bps = 50000,
        .net_tx_bps = 25000,
    };

    const written = try PacketEncoder.encodeMetricsJson(sample, &buf);
    try std.testing.expect(written > 0);
    const json_str = buf[0..written];
    try std.testing.expect(std.mem.indexOf(u8, json_str, "\"type\":\"metrics\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, json_str, "\"cpu_user\":25.50") != null);

    // Test binary envelope
    var bin_buf: [4096]u8 = undefined;
    const bin_len = try PacketEncoder.encodeBinaryFrame(.metrics_batch, json_str, &bin_buf);
    try std.testing.expect(bin_len > @sizeOf(PacketHeader));
    try std.testing.expectEqualStrings("TELM", bin_buf[0..4]);
}
