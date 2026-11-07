// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

'use client';

import React from 'react';

interface HeaderProps {
  ingestionRate: number;
  nodeCount: number;
  activeIncidents: number;
  isLive: boolean;
  onToggleLive: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  ingestionRate,
  nodeCount,
  activeIncidents,
  isLive,
  onToggleLive,
}) => {
  return (
    <header className="border-b border-border bg-[#0f172a] px-6 py-4 flex flex-wrap items-center justify-between gap-4">
      <div className="flex items-center gap-3">
        <div className="h-8 w-8 rounded bg-blue-600 flex items-center justify-center font-bold text-white tracking-wider">
          NX
        </div>
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-white flex items-center gap-2">
            HEXENE TELEMETRY ENGINE
            <span className="text-xs px-2 py-0.5 rounded bg-blue-900/60 text-blue-400 border border-blue-700/50">
              DISTRIBUTED
            </span>
          </h1>
          <p className="text-xs text-gray-400">
            Real-Time Observability | Zig Agent + Rust Ingestion + Carbon Analytics
          </p>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900 border border-border text-xs">
          <span className="text-gray-400">DaemonSet Nodes:</span>
          <span className="font-mono text-emerald-400 font-medium">{nodeCount} active</span>
        </div>

        <div className="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900 border border-border text-xs">
          <span className="text-gray-400">Ingestion:</span>
          <span className="font-mono text-cyan-400 font-medium">{ingestionRate.toLocaleString()} logs/s</span>
        </div>

        <div className="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900 border border-border text-xs">
          <span className="text-gray-400">Incidents:</span>
          <span className={`font-mono font-medium ${activeIncidents > 0 ? 'text-rose-400' : 'text-emerald-400'}`}>
            {activeIncidents} firing
          </span>
        </div>

        <button
          onClick={onToggleLive}
          className={`flex items-center gap-2 px-3 py-1.5 rounded text-xs font-medium transition-colors border ${
            isLive
              ? 'bg-emerald-950 text-emerald-300 border-emerald-700/50 hover:bg-emerald-900'
              : 'bg-amber-950 text-amber-300 border-amber-700/50 hover:bg-amber-900'
          }`}
        >
          <span className={`h-2 w-2 rounded-full ${isLive ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'}`} />
          {isLive ? 'STREAMING LIVE' : 'PAUSED'}
        </button>
      </div>
    </header>
  );
};
