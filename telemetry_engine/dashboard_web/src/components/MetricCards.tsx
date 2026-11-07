// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

'use client';

import React, { useState } from 'react';
import { AggregationResult, SystemMetrics } from '../lib/types';

interface MetricCardsProps {
  metrics: SystemMetrics[];
  aggregation: AggregationResult | null;
}

export const MetricCards: React.FC<MetricCardsProps> = ({ metrics, aggregation }) => {
  const [cardOrder, setCardOrder] = useState<string[]>(['cpu', 'memory', 'latency', 'errors']);
  const [draggedCard, setDraggedCard] = useState<string | null>(null);

  const latestMetric = metrics[metrics.length - 1] || {
    cpu_user: 18.5,
    cpu_sys: 6.2,
    mem_used: 8590000000,
    mem_total: 17179869184,
  };

  const handleDragStart = (id: string) => {
    setDraggedCard(id);
  };

  const handleDragOver = (e: React.DragEvent, targetId: string) => {
    e.preventDefault();
    if (!draggedCard || draggedCard === targetId) return;

    const newOrder = [...cardOrder];
    const dragIdx = newOrder.indexOf(draggedCard);
    const targetIdx = newOrder.indexOf(targetId);

    newOrder.splice(dragIdx, 1);
    newOrder.splice(targetIdx, 0, draggedCard);
    setCardOrder(newOrder);
  };

  const renderCardContent = (id: string) => {
    switch (id) {
      case 'cpu': {
        const totalCpu = Math.round(latestMetric.cpu_user + latestMetric.cpu_sys);
        const points = metrics.map((m, idx) => {
          const x = (idx / (metrics.length - 1 || 1)) * 180;
          const y = 45 - ((m.cpu_user + m.cpu_sys) / 100) * 40;
          return `${x.toFixed(1)},${y.toFixed(1)}`;
        }).join(' ');

        return (
          <div className="p-4 flex flex-col justify-between h-full">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-gray-400 uppercase tracking-wider">Node CPU Usage</span>
              <span className="text-xs text-gray-500 font-mono">/proc/stat</span>
            </div>
            <div className="my-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold font-mono text-white">{totalCpu}%</span>
              <span className="text-xs text-gray-400">
                (User: {latestMetric.cpu_user.toFixed(1)}% | Sys: {latestMetric.cpu_sys.toFixed(1)}%)
              </span>
            </div>
            <div className="h-12 w-full mt-1">
              <svg className="w-full h-full overflow-visible" viewBox="0 0 180 45">
                <polyline
                  fill="none"
                  stroke="#3b82f6"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  points={points || '0,35 90,25 180,30'}
                />
              </svg>
            </div>
          </div>
        );
      }
      case 'memory': {
        const usedGb = (latestMetric.mem_used / (1024 * 1024 * 1024)).toFixed(1);
        const totalGb = (latestMetric.mem_total / (1024 * 1024 * 1024)).toFixed(1);
        const pct = Math.round((latestMetric.mem_used / (latestMetric.mem_total || 1)) * 100);

        return (
          <div className="p-4 flex flex-col justify-between h-full">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-gray-400 uppercase tracking-wider">Host Memory</span>
              <span className="text-xs text-gray-500 font-mono">/proc/meminfo</span>
            </div>
            <div className="my-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold font-mono text-white">{pct}%</span>
              <span className="text-xs text-gray-400">
                {usedGb} GB / {totalGb} GB
              </span>
            </div>
            <div className="w-full bg-slate-800 rounded-full h-2.5 mt-4 overflow-hidden border border-slate-700">
              <div
                className="bg-emerald-500 h-2.5 rounded-full transition-all duration-300"
                style={{ width: `${pct}%` }}
              />
            </div>
          </div>
        );
      }
      case 'latency': {
        const p50 = aggregation ? aggregation.p50 : 24;
        const p95 = aggregation ? aggregation.p95 : 110;
        const p99 = aggregation ? aggregation.p99 : 280;

        return (
          <div className="p-4 flex flex-col justify-between h-full">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-gray-400 uppercase tracking-wider">Latency Percentiles</span>
              <span className="text-xs text-gray-500 font-mono">Carbon Vector Query</span>
            </div>
            <div className="my-2 flex items-baseline gap-4 font-mono">
              <div>
                <span className="text-xs text-gray-400 block">p50</span>
                <span className="text-lg font-bold text-emerald-400">{p50}ms</span>
              </div>
              <div>
                <span className="text-xs text-gray-400 block">p95</span>
                <span className="text-lg font-bold text-amber-400">{p95}ms</span>
              </div>
              <div>
                <span className="text-xs text-gray-400 block">p99</span>
                <span className="text-lg font-bold text-rose-400">{p99}ms</span>
              </div>
            </div>
            <div className="flex items-center gap-1 text-[11px] text-gray-500">
              <span>Avg: {aggregation ? aggregation.avg : 42}ms</span>
              <span>•</span>
              <span>Range: {aggregation ? aggregation.min : 10}ms - {aggregation ? aggregation.max : 350}ms</span>
            </div>
          </div>
        );
      }
      case 'errors': {
        const errorRate = aggregation ? aggregation.error_rate : 2.5;
        const errCount = aggregation ? aggregation.errors : 8;

        return (
          <div className="p-4 flex flex-col justify-between h-full">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-gray-400 uppercase tracking-wider">Error Rate (5xx)</span>
              <span className="text-xs text-gray-500 font-mono">SLO Window</span>
            </div>
            <div className="my-2 flex items-baseline gap-2">
              <span
                className={`text-2xl font-bold font-mono ${
                  errorRate > 5.0 ? 'text-rose-400' : errorRate > 2.0 ? 'text-amber-400' : 'text-emerald-400'
                }`}
              >
                {errorRate.toFixed(2)}%
              </span>
              <span className="text-xs text-gray-400">({errCount} error logs in window)</span>
            </div>
            <div className="flex items-center gap-2 mt-2">
              <span
                className={`text-[10px] px-2 py-0.5 rounded font-mono uppercase ${
                  errorRate > 5.0
                    ? 'bg-rose-950 text-rose-300 border border-rose-800'
                    : 'bg-emerald-950 text-emerald-300 border border-emerald-800'
                }`}
              >
                {errorRate > 5.0 ? 'SLO BREACH' : 'HEALTHY'}
              </span>
              <span className="text-[11px] text-gray-500">Threshold: 5.0%</span>
            </div>
          </div>
        );
      }
      default:
        return null;
    }
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 px-6 pt-4">
      {cardOrder.map((id) => (
        <div
          key={id}
          draggable
          onDragStart={() => handleDragStart(id)}
          onDragOver={(e) => handleDragOver(e, id)}
          className="rounded-lg bg-card border border-border cursor-grab active:cursor-grabbing hover:border-slate-600 transition-all shadow-sm"
        >
          {renderCardContent(id)}
        </div>
      ))}
    </div>
  );
};
