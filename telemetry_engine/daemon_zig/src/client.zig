// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
const std = @import("std");

pub const IngestionClient = struct {
    host: []const u8,
    port: u16,
    fd: ?std.posix.fd_t = null,
    connected: bool = false,

    pub fn init(host: []const u8, port: u16) IngestionClient {
        return .{
            .host = host,
            .port = port,
            .fd = null,
            .connected = false,
        };
    }

    fn openSocket(self: *IngestionClient) !std.posix.fd_t {
        const sock_fd = std.posix.system.socket(std.posix.AF.INET, std.posix.SOCK.STREAM, 0);
        if (sock_fd < 0) return error.SocketCreationFailed;

        var addr: std.posix.sockaddr.in = std.mem.zeroes(std.posix.sockaddr.in);
        addr.family = std.posix.AF.INET;
        addr.port = std.mem.nativeToBig(u16, self.port);
        // Default to 127.0.0.1 (0x7F000001)
        addr.addr = std.mem.nativeToBig(u32, 0x7F000001);

        const res = std.posix.system.connect(
            sock_fd,
            @ptrCast(&addr),
            @sizeOf(std.posix.sockaddr.in),
        );

        if (res != 0) {
            _ = std.posix.system.close(sock_fd);
            return error.ConnectionFailed;
        }

        return sock_fd;
    }

    pub fn sendPayload(self: *IngestionClient, payload: []const u8) !void {
        const sock = try self.openSocket();
        defer _ = std.posix.system.close(sock);
        const rc = std.posix.system.write(sock, payload.ptr, payload.len);
        if (rc < 0) return error.WriteFailed;
    }

    pub fn sendHttpPost(self: *IngestionClient, path: []const u8, body: []const u8) !void {
        const sock = try self.openSocket();
        defer _ = std.posix.system.close(sock);

        var header_buf: [512]u8 = undefined;
        const headers = try std.fmt.bufPrint(
            &header_buf,
            "POST {s} HTTP/1.1\r\nHost: {s}:{d}\r\nContent-Length: {d}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
            .{ path, self.host, self.port, body.len },
        );
        _ = std.posix.system.write(sock, headers.ptr, headers.len);
        _ = std.posix.system.write(sock, body.ptr, body.len);
    }

    pub fn disconnect(self: *IngestionClient) void {
        if (self.fd) |fd| {
            _ = std.posix.system.close(fd);
            self.fd = null;
        }
        self.connected = false;
    }
};

test "IngestionClient initialization" {
    const client_inst = IngestionClient.init("127.0.0.1", 8080);
    try std.testing.expectEqualStrings("127.0.0.1", client_inst.host);
    try std.testing.expectEqual(@as(u16, 8080), client_inst.port);
    try std.testing.expect(!client_inst.connected);
}
