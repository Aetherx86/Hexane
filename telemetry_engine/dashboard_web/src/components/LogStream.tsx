// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

'use client';

import React, { useState } from 'react';
import { LogEntry } from '../lib/types';

interface LogStreamProps {
  logs: LogEntry[];
  isStreaming: boolean;
  onClear: () => void;
}

export const LogStream: React.FC<LogStreamProps> = ({ logs, isStreaming, onClear }) => {
  const [expandedIndex, setExpandedIndex] = useState<number | null>(null);

  const toggleExpand = (idx: number) => {
    setExpandedIndex(expandedIndex === idx ? null : idx);
  };

  return (
    <div className="flex-1 bg-slate-950 border border-border rounded-lg overflow-hidden flex flex-col font-mono text-xs">
      <div className="bg-slate-900 border-b border-border px-4 py-2.5 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-gray-200 uppercase tracking-wider text-[11px]">
            Live Telemetry Log Stream
          </span>
          <span className="text-[10px] text-gray-400 bg-slate-800 px-2 py-0.5 rounded">
            {logs.length} records buffered
          </span>
          {isStreaming && (
            <span className="flex items-center gap-1.5 text-[10px] text-emerald-400">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-ping" />
              Tail -f
            </span>
          )}
        </div>

        <button
          onClick={onClear}
          className="text-gray-400 hover:text-gray-200 text-[11px] underline"
        >
          Clear Stream
        </button>
      </div>

      <div className="flex-1 overflow-y-auto max-h-[460px] divide-y divide-slate-900/60 p-2">
        {logs.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            No telemetry records match the current query criteria.
          </div>
        ) : (
          logs.map((log, idx) => {
            const dateStr = new Date(log.timestamp).toISOString().split('T')[1]?.replace('Z', '') || '';
            const isError = log.level === 'ERROR' || log.status >= 500;
            const isWarn = log.level === 'WARN';
            const isExpanded = expandedIndex === idx;

            return (
              <div
                key={`${log.timestamp}-${idx}`}
                onClick={() => toggleExpand(idx)}
                className={`p-2 rounded cursor-pointer transition-colors ${
                  isExpanded
                    ? 'bg-slate-900 border border-blue-500/30'
                    : 'hover:bg-slate-900/80'
                }`}
              >
                <div className="flex items-center gap-2.5 flex-wrap">
                  {/* Timestamp */}
                  <span className="text-gray-500 text-[11px]">{dateStr}</span>

                  {/* Level Badge */}
                  <span
                    className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                      isError
                        ? 'bg-rose-950 text-rose-400 border border-rose-800'
                        : isWarn
                        ? 'bg-amber-950 text-amber-400 border border-amber-800'
                        : 'bg-blue-950 text-blue-400 border border-blue-800'
                    }`}
                  >
                    {log.level}
                  </span>

                  {/* Service */}
                  <span className="text-purple-300 font-medium">[{log.service}]</span>

                  {/* Status Code */}
                  <span
                    className={`px-1.5 py-0.5 rounded text-[10px] ${
                      log.status >= 500
                        ? 'bg-rose-900/40 text-rose-300'
                        : log.status >= 400
                        ? 'bg-amber-900/40 text-amber-300'
                        : 'bg-emerald-900/40 text-emerald-300'
                    }`}
                  >
                    {log.status}
                  </span>

                  {/* Latency */}
                  <span
                    className={`text-[10px] ${
                      log.latency > 200
                        ? 'text-rose-400 font-bold'
                        : log.latency > 100
                        ? 'text-amber-400'
                        : 'text-gray-400'
                    }`}
                  >
                    {log.latency}ms
                  </span>

                  {/* Trace ID */}
                  <span className="text-gray-500 text-[11px]">{log.trace_id}</span>

                  {/* Message */}
                  <span
                    className={`flex-1 truncate ${
                      log.message.includes('[SECURITY WARNING')
                        ? 'text-amber-300 font-semibold bg-amber-950/40 px-2 py-0.5 rounded border border-amber-800/60'
                        : 'text-gray-300'
                    }`}
                  >
                    {log.message}
                  </span>
                </div>

                {/* Expanded Payload Viewer */}
                {isExpanded && (
                  <div className="mt-2.5 p-3 rounded bg-[#090d16] border border-border text-[11px] text-gray-300 overflow-x-auto">
                    <div className="text-xs text-blue-400 font-bold mb-1">
                      Structured Telemetry Payload:
                    </div>
                    <pre className="text-gray-300">
                      {JSON.stringify(
                        {
                          timestamp: log.timestamp,
                          iso_time: new Date(log.timestamp).toISOString(),
                          service: log.service,
                          level: log.level,
                          status: log.status,
                          latency_ms: log.latency,
                          trace_id: log.trace_id,
                          message: log.message,
                          host_agent: 'zig-k8s-daemon',
                          parser: 'rust-fast-streaming',
                        },
                        null,
                        2
                      )}
                    </pre>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
