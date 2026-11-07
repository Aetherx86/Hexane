// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
const std = @import("std");

pub fn currentMillis() i64 {
    var ts: std.posix.timespec = undefined;
    _ = std.posix.system.clock_gettime(.REALTIME, &ts);
    return @as(i64, ts.sec) * 1000 + @divTrunc(@as(i64, ts.nsec), 1000000);
}

pub fn sleepMillis(ms: u64) void {
    const sec: i64 = @intCast(ms / 1000);
    const nsec: i64 = @intCast((ms % 1000) * 1000000);
    var req = std.posix.timespec{ .sec = sec, .nsec = nsec };
    _ = std.posix.system.nanosleep(&req, null);
}

pub fn printOut(comptime fmt: []const u8, args: anytype) void {
    var buf: [4096]u8 = undefined;
    const msg = std.fmt.bufPrint(&buf, fmt, args) catch return;
    _ = std.posix.system.write(1, msg.ptr, msg.len);
}
