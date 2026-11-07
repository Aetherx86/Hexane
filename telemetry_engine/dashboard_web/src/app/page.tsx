// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

'use client';

import React, { useEffect, useState } from 'react';
import { Header } from '../components/Header';
import { MetricCards } from '../components/MetricCards';
import { QueryBuilder } from '../components/QueryBuilder';
import { LogStream } from '../components/LogStream';
import { AlertingManager } from '../components/AlertingManager';
import { fetchLiveMetrics, fetchQuery } from '../lib/api';
import {
  AggregationResult,
  AlertRule,
  Incident,
  LogEntry,
  QueryFilter,
  SystemMetrics,
} from '../lib/types';

export default function DashboardPage() {
  const [filter, setFilter] = useState<QueryFilter>({
    service: 'all',
    statusGroup: 'ALL',
    minLatency: 0,
    keyword: '',
  });

  const [isLive, setIsLive] = useState(true);
  const [metrics, setMetrics] = useState<SystemMetrics[]>([]);
  const [aggregation, setAggregation] = useState<AggregationResult | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const [rules, setRules] = useState<AlertRule[]>([
    {
      id: 'rule-1',
      name: 'High 5xx Error Rate Surge',
      metric: 'error_rate',
      operator: '>',
      threshold: 5.0,
      duration_secs: 60,
      severity: 'CRITICAL',
      state: 'OK',
    },
    {
      id: 'rule-2',
      name: 'P99 Latency SLA Violation',
      metric: 'p99_latency',
      operator: '>',
      threshold: 250,
      duration_secs: 120,
      severity: 'WARNING',
      state: 'OK',
    },
    {
      id: 'rule-3',
      name: 'Kube Node CPU Saturation',
      metric: 'cpu_user',
      operator: '>',
      threshold: 80.0,
      duration_secs: 300,
      severity: 'CRITICAL',
      state: 'OK',
    },
  ]);

  const [incidents, setIncidents] = useState<Incident[]>([]);

  // Core telemetry polling loop
  useEffect(() => {
    let isSubscribed = true;

    const poll = async () => {
      if (!isLive) return;

      const [resMetrics, resQuery] = await Promise.all([
        fetchLiveMetrics(),
        fetchQuery(filter),
      ]);

      if (!isSubscribed) return;

      setMetrics(resMetrics);
      setAggregation(resQuery);

      if (resQuery && resQuery.records) {
        setLogs((prev) => {
          const combined = [...resQuery.records, ...prev];
          // Deduplicate by trace_id
          const seen = new Set<string>();
          const deduped: LogEntry[] = [];
          for (const item of combined) {
            if (!seen.has(item.trace_id)) {
              seen.add(item.trace_id);
              deduped.push(item);
            }
          }
          return deduped.slice(0, 100);
        });
      }

      // Evaluate alert rules
      if (resQuery) {
        evaluateAlerts(resQuery, resMetrics);
      }
    };

    poll();
    const interval = setInterval(poll, 1500);

    return () => {
      isSubscribed = false;
      clearInterval(interval);
    };
  }, [isLive, filter]);

  const evaluateAlerts = (agg: AggregationResult, sysMetrics: SystemMetrics[]) => {
    const latestSys = sysMetrics[sysMetrics.length - 1];
    const newIncidents: Incident[] = [];

    setRules((prevRules) =>
      prevRules.map((r) => {
        let val = 0;
        if (r.metric === 'error_rate') val = agg.error_rate;
        else if (r.metric === 'p99_latency') val = agg.p99;
        else if (r.metric === 'cpu_user' && latestSys) val = latestSys.cpu_user;

        let triggered = false;
        if (r.operator === '>') triggered = val > r.threshold;
        else if (r.operator === '>=') triggered = val >= r.threshold;
        else if (r.operator === '<') triggered = val < r.threshold;

        if (triggered) {
          newIncidents.push({
            id: `inc-${r.id}-${Date.now()}`,
            rule_id: r.id,
            rule_name: r.name,
            severity: r.severity,
            triggered_at: Date.now(),
            value: val,
            threshold: r.threshold,
            status: 'ACTIVE',
          });
        }

        return {
          ...r,
          state: triggered ? 'FIRING' : 'OK',
        };
      })
    );

    if (newIncidents.length > 0) {
      setIncidents((prev) => {
        // Keep unique by rule_id
        const existingRuleIds = new Set(prev.map((i) => i.rule_id));
        const added = newIncidents.filter((i) => !existingRuleIds.has(i.rule_id));
        return [...added, ...prev].slice(0, 5);
      });
    }
  };

  const handleAddRule = (newRuleData: Omit<AlertRule, 'id' | 'state'>) => {
    const newRule: AlertRule = {
      ...newRuleData,
      id: `rule-${Date.now()}`,
      state: 'OK',
    };
    setRules((prev) => [...prev, newRule]);
  };

  const handleAcknowledgeIncident = (id: string) => {
    setIncidents((prev) =>
      prev.map((inc) => (inc.id === id ? { ...inc, status: 'ACKNOWLEDGED' } : inc))
    );
  };

  return (
    <div className="flex-1 flex flex-col bg-[#0b0f19]">
      <Header
        ingestionRate={125000}
        nodeCount={24}
        activeIncidents={incidents.filter((i) => i.status === 'ACTIVE').length}
        isLive={isLive}
        onToggleLive={() => setIsLive(!isLive)}
      />

      <MetricCards metrics={metrics} aggregation={aggregation} />

      <QueryBuilder
        filter={filter}
        onFilterChange={setFilter}
        onRefresh={() => fetchQuery(filter).then(setAggregation)}
      />

      <main className="flex-1 p-6 grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 flex flex-col">
          <LogStream
            logs={logs}
            isStreaming={isLive}
            onClear={() => setLogs([])}
          />
        </div>

        <div className="lg:col-span-1">
          <AlertingManager
            rules={rules}
            incidents={incidents}
            onAddRule={handleAddRule}
            onAcknowledgeIncident={handleAcknowledgeIncident}
          />
        </div>
      </main>
    </div>
  );
}
