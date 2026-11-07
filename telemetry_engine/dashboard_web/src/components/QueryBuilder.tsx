// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

'use client';

import React from 'react';
import { QueryFilter } from '../lib/types';

interface QueryBuilderProps {
  filter: QueryFilter;
  onFilterChange: (newFilter: QueryFilter) => void;
  onRefresh: () => void;
}

export const QueryBuilder: React.FC<QueryBuilderProps> = ({
  filter,
  onFilterChange,
  onRefresh,
}) => {
  const services = [
    { id: 'all', label: 'All Services' },
    { id: 'auth-service', label: 'auth-service' },
    { id: 'payment-gateway', label: 'payment-gateway' },
    { id: 'order-router', label: 'order-router' },
    { id: 'inventory-db', label: 'inventory-db' },
    { id: 'search-indexer', label: 'search-indexer' },
  ];

  const statusGroups: Array<'ALL' | '2xx' | '4xx' | '5xx'> = ['ALL', '2xx', '4xx', '5xx'];

  return (
    <div className="bg-card border-y border-border px-6 py-4 mt-4">
      <div className="flex flex-wrap items-center gap-3">
        {/* Service Selector */}
        <div className="flex items-center gap-2">
          <label className="text-xs text-gray-400 font-medium">Service:</label>
          <select
            value={filter.service || 'all'}
            onChange={(e) => onFilterChange({ ...filter, service: e.target.value })}
            className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-gray-200 focus:outline-none focus:border-blue-500"
          >
            {services.map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
        </div>

        {/* Status Code Pills */}
        <div className="flex items-center gap-1.5">
          <label className="text-xs text-gray-400 font-medium mr-1">Status:</label>
          {statusGroups.map((group) => {
            const active = (filter.statusGroup || 'ALL') === group;
            return (
              <button
                key={group}
                onClick={() => onFilterChange({ ...filter, statusGroup: group })}
                className={`px-2.5 py-1 text-xs font-mono rounded transition-colors ${
                  active
                    ? 'bg-blue-600 text-white font-medium'
                    : 'bg-slate-900 text-gray-400 border border-slate-800 hover:text-white'
                }`}
              >
                {group}
              </button>
            );
          })}
        </div>

        {/* Min Latency Threshold */}
        <div className="flex items-center gap-2">
          <label className="text-xs text-gray-400 font-medium">Latency &gt;=</label>
          <select
            value={filter.minLatency || 0}
            onChange={(e) => onFilterChange({ ...filter, minLatency: Number(e.target.value) })}
            className="bg-slate-900 border border-slate-700 rounded px-2 py-1.5 text-xs text-gray-200 focus:outline-none focus:border-blue-500"
          >
            <option value={0}>0ms (All)</option>
            <option value={50}>50ms</option>
            <option value={100}>100ms</option>
            <option value={200}>200ms</option>
            <option value={300}>300ms</option>
          </select>
        </div>

        {/* Free-text Keyword / Regex */}
        <div className="flex-1 min-w-[200px]">
          <input
            type="text"
            placeholder="Search regex or token (e.g. timeout, JWT, connection)..."
            value={filter.keyword || ''}
            onChange={(e) => onFilterChange({ ...filter, keyword: e.target.value })}
            className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1.5 text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-blue-500 font-mono"
          />
        </div>

        {/* Refresh / Run Query Button */}
        <button
          onClick={onRefresh}
          className="bg-blue-600 hover:bg-blue-500 text-white text-xs px-4 py-1.5 rounded font-medium transition-colors shadow-sm"
        >
          Run Analytical Query
        </button>
      </div>
    </div>
  );
};
