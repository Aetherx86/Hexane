// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

'use client';

import React, { useState } from 'react';
import { AlertRule, Incident } from '../lib/types';

interface AlertingManagerProps {
  rules: AlertRule[];
  incidents: Incident[];
  onAddRule: (rule: Omit<AlertRule, 'id' | 'state'>) => void;
  onAcknowledgeIncident: (id: string) => void;
}

export const AlertingManager: React.FC<AlertingManagerProps> = ({
  rules,
  incidents,
  onAddRule,
  onAcknowledgeIncident,
}) => {
  const [showAddModal, setShowAddModal] = useState(false);
  const [name, setName] = useState('');
  const [metric, setMetric] = useState<'error_rate' | 'p99_latency' | 'cpu_user' | 'load_1m'>('error_rate');
  const [operator, setOperator] = useState<'>' | '<' | '>='>('>');
  const [threshold, setThreshold] = useState(5.0);
  const [severity, setSeverity] = useState<'INFO' | 'WARNING' | 'CRITICAL'>('CRITICAL');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    onAddRule({
      name: name.trim(),
      metric,
      operator,
      threshold,
      duration_secs: 60,
      severity,
    });

    setName('');
    setShowAddModal(false);
  };

  return (
    <div className="bg-card border border-border rounded-lg p-5 flex flex-col gap-4">
      <div className="flex items-center justify-between border-b border-border pb-3">
        <div>
          <h2 className="text-sm font-semibold text-white uppercase tracking-wider">
            Alerting Rules & Incident Response
          </h2>
          <p className="text-xs text-gray-400">
            Real-time metric thresholds with automated state evaluation
          </p>
        </div>

        <button
          onClick={() => setShowAddModal(true)}
          className="bg-blue-600 hover:bg-blue-500 text-white text-xs px-3 py-1.5 rounded font-medium transition-colors"
        >
          + Create Alert Rule
        </button>
      </div>

      {/* Rules Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-left text-xs">
          <thead>
            <tr className="text-gray-400 border-b border-slate-800">
              <th className="pb-2 font-medium">Rule Name</th>
              <th className="pb-2 font-medium">Condition</th>
              <th className="pb-2 font-medium">Severity</th>
              <th className="pb-2 font-medium">Evaluation State</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60 font-mono">
            {rules.map((rule) => {
              const isFiring = rule.state === 'FIRING';
              const isPending = rule.state === 'PENDING';

              return (
                <tr key={rule.id} className="hover:bg-slate-900/50">
                  <td className="py-2.5 font-sans font-medium text-gray-200">{rule.name}</td>
                  <td className="py-2.5 text-gray-400">
                    {rule.metric} {rule.operator} {rule.threshold}
                  </td>
                  <td className="py-2.5">
                    <span
                      className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                        rule.severity === 'CRITICAL'
                          ? 'bg-rose-950 text-rose-300 border border-rose-800'
                          : 'bg-amber-950 text-amber-300 border border-amber-800'
                      }`}
                    >
                      {rule.severity}
                    </span>
                  </td>
                  <td className="py-2.5">
                    <span
                      className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-bold ${
                        isFiring
                          ? 'bg-rose-950 text-rose-400 animate-pulse border border-rose-700'
                          : isPending
                          ? 'bg-amber-950 text-amber-400 border border-amber-700'
                          : 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      }`}
                    >
                      <span
                        className={`h-1.5 w-1.5 rounded-full ${
                          isFiring ? 'bg-rose-400' : isPending ? 'bg-amber-400' : 'bg-emerald-400'
                        }`}
                      />
                      {rule.state}
                    </span>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Active Incidents Banner */}
      {incidents.length > 0 && (
        <div className="mt-2 p-3 bg-rose-950/40 border border-rose-800/60 rounded flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold text-rose-300 uppercase tracking-wide flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-rose-500 animate-ping" />
              Active Triggered Incidents ({incidents.length})
            </span>
          </div>

          <div className="space-y-1.5">
            {incidents.map((inc) => (
              <div
                key={inc.id}
                className="flex items-center justify-between text-xs font-mono bg-slate-900/80 p-2 rounded border border-rose-900/50"
              >
                <div>
                  <span className="text-gray-200 font-sans font-medium">{inc.rule_name}</span>
                  <span className="text-rose-400 ml-2">
                    (Current: {inc.value.toFixed(1)} &gt; Threshold: {inc.threshold})
                  </span>
                </div>
                <button
                  onClick={() => onAcknowledgeIncident(inc.id)}
                  className="bg-slate-800 hover:bg-slate-700 text-gray-300 text-[11px] px-2 py-1 rounded border border-slate-700"
                >
                  {inc.status === 'ACKNOWLEDGED' ? 'Acknowledged' : 'Acknowledge'}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Add Rule Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50">
          <div className="bg-[#111827] border border-border rounded-lg max-w-md w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between border-b border-border pb-2">
              <h3 className="text-sm font-semibold text-white">Create New Alerting Rule</h3>
              <button
                onClick={() => setShowAddModal(false)}
                className="text-gray-400 hover:text-white text-sm"
              >
                x
              </button>
            </div>

            <form onSubmit={handleSubmit} className="space-y-3 text-xs">
              <div>
                <label className="block text-gray-400 mb-1">Rule Name</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. High Error Rate Spike"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-white"
                />
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-gray-400 mb-1">Metric Target</label>
                  <select
                    value={metric}
                    onChange={(e) => setMetric(e.target.value as any)}
                    className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-white"
                  >
                    <option value="error_rate">error_rate (%)</option>
                    <option value="p99_latency">p99_latency (ms)</option>
                    <option value="cpu_user">cpu_user (%)</option>
                    <option value="load_1m">load_1m (avg)</option>
                  </select>
                </div>

                <div>
                  <label className="block text-gray-400 mb-1">Operator</label>
                  <select
                    value={operator}
                    onChange={(e) => setOperator(e.target.value as any)}
                    className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-white"
                  >
                    <option value=">">&gt; (Greater than)</option>
                    <option value=">=">&gt;= (Greater or equal)</option>
                    <option value="<">&lt; (Less than)</option>
                  </select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-gray-400 mb-1">Threshold Value</label>
                  <input
                    type="number"
                    step="any"
                    required
                    value={threshold}
                    onChange={(e) => setThreshold(parseFloat(e.target.value) || 0)}
                    className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-white font-mono"
                  />
                </div>

                <div>
                  <label className="block text-gray-400 mb-1">Severity</label>
                  <select
                    value={severity}
                    onChange={(e) => setSeverity(e.target.value as any)}
                    className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-white"
                  >
                    <option value="WARNING">WARNING</option>
                    <option value="CRITICAL">CRITICAL</option>
                    <option value="INFO">INFO</option>
                  </select>
                </div>
              </div>

              <div className="flex justify-end gap-2 pt-2 border-t border-border">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  className="px-3 py-1.5 rounded bg-slate-800 text-gray-300 hover:bg-slate-700"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 rounded bg-blue-600 text-white font-medium hover:bg-blue-500"
                >
                  Save Alert Rule
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
