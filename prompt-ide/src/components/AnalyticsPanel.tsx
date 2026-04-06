import { useState, useEffect, useMemo } from 'react';
import { BarChart3, Activity, Zap, Coins, Clock, Hash } from 'lucide-react';
import type { ExecutionResult } from '../lib/types';
import { MODELS } from '../lib/types';
import { db } from '../lib/db';
import { formatCost, formatTokens } from '../lib/tokens';
import { useT } from '../lib/i18n';

type TimeRange = '7days' | '30days' | 'allTime';

interface AnalyticsPanelProps {
  userId: string;
}

export function AnalyticsPanel({ userId }: AnalyticsPanelProps) {
  const t = useT();
  const [executions, setExecutions] = useState<ExecutionResult[]>([]);
  const [timeRange, setTimeRange] = useState<TimeRange>('allTime');

  // Load all executions for the user's projects
  useEffect(() => {
    const load = async () => {
      const projects = await db.projects.where('userId').equals(userId).toArray();
      const projectIds = projects.map((p) => p.id);
      if (projectIds.length === 0) {
        setExecutions([]);
        return;
      }
      const allExecs: ExecutionResult[] = [];
      for (const pid of projectIds) {
        const execs = await db.executions.where('projectId').equals(pid).toArray();
        allExecs.push(...execs);
      }
      setExecutions(allExecs);
    };
    load();
    const interval = setInterval(load, 5000);
    return () => clearInterval(interval);
  }, [userId]);

  // Filter by time range
  const filtered = useMemo(() => {
    if (timeRange === 'allTime') return executions;
    const days = timeRange === '7days' ? 7 : 30;
    // Use latest execution timestamp as reference (pure, deterministic)
    const ref = executions.length > 0 ? Math.max(...executions.map((e) => e.createdAt)) : 0;
    const cutoff = ref - days * 24 * 60 * 60 * 1000;
    return executions.filter((e) => e.createdAt >= cutoff);
  }, [executions, timeRange]);

  // Compute stats
  const stats = useMemo(() => {
    const totalExec = filtered.length;
    const totalTokens = filtered.reduce((sum, e) => sum + e.tokensIn + e.tokensOut, 0);
    const totalCost = filtered.reduce((sum, e) => sum + e.costEstimate, 0);
    const avgLatency = totalExec > 0
      ? Math.round(filtered.reduce((sum, e) => sum + e.latencyMs, 0) / totalExec)
      : 0;

    // Per-model stats
    const modelCounts: Record<string, number> = {};
    for (const e of filtered) {
      modelCounts[e.model] = (modelCounts[e.model] || 0) + 1;
    }

    const modelEntries = Object.entries(modelCounts).sort((a, b) => b[1] - a[1]);
    const topModel = modelEntries.length > 0 ? modelEntries[0] : null;
    const maxCount = modelEntries.length > 0 ? modelEntries[0][1] : 1;

    return { totalExec, totalTokens, totalCost, avgLatency, modelEntries, topModel, maxCount };
  }, [filtered]);

  const getModelName = (modelId: string) => {
    return MODELS.find((m) => m.id === modelId)?.name ?? modelId;
  };

  const ranges: { id: TimeRange; label: string }[] = [
    { id: '7days', label: t('analytics.7days') },
    { id: '30days', label: t('analytics.30days') },
    { id: 'allTime', label: t('analytics.allTime') },
  ];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
        <h3 className="text-sm font-semibold text-[var(--color-text-primary)] flex items-center gap-2">
          <BarChart3 size={14} />
          {t('analytics.title')}
        </h3>
      </div>

      {/* Time range toggle */}
      <div className="flex gap-1 p-3 border-b border-[var(--color-border)]">
        {ranges.map((r) => (
          <button
            key={r.id}
            onClick={() => setTimeRange(r.id)}
            className={`flex-1 px-2 py-1.5 rounded text-xs font-medium transition-colors ${
              timeRange === r.id
                ? 'bg-[var(--color-accent)] text-white'
                : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
            }`}
          >
            {r.label}
          </button>
        ))}
      </div>

      {/* Stats cards */}
      <div className="flex-1 overflow-auto p-3 space-y-3">
        {/* KPI grid */}
        <div className="grid grid-cols-2 gap-2">
          <div className="p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
            <div className="flex items-center gap-1.5 mb-1">
              <Activity size={12} className="text-[var(--color-accent)]" />
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wide">{t('analytics.totalExec')}</span>
            </div>
            <span className="text-lg font-bold text-[var(--color-text-primary)]">{stats.totalExec}</span>
          </div>

          <div className="p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
            <div className="flex items-center gap-1.5 mb-1">
              <Hash size={12} className="text-[var(--color-context)]" />
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wide">{t('analytics.totalTokens')}</span>
            </div>
            <span className="text-lg font-bold text-[var(--color-text-primary)]">{formatTokens(stats.totalTokens)}</span>
          </div>

          <div className="p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
            <div className="flex items-center gap-1.5 mb-1">
              <Coins size={12} className="text-[var(--color-examples)]" />
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wide">{t('analytics.totalCost')}</span>
            </div>
            <span className="text-lg font-bold text-[var(--color-text-primary)]">{formatCost(stats.totalCost)}</span>
          </div>

          <div className="p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
            <div className="flex items-center gap-1.5 mb-1">
              <Clock size={12} className="text-[var(--color-task)]" />
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wide">{t('analytics.avgLatency')}</span>
            </div>
            <span className="text-lg font-bold text-[var(--color-text-primary)]">{stats.avgLatency}ms</span>
          </div>
        </div>

        {/* Top model */}
        {stats.topModel && (
          <div className="p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
            <div className="flex items-center gap-1.5 mb-1">
              <Zap size={12} className="text-[var(--color-role)]" />
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wide">{t('analytics.topModel')}</span>
            </div>
            <span className="text-sm font-semibold text-[var(--color-accent)]">{getModelName(stats.topModel[0])}</span>
            <span className="text-xs text-[var(--color-text-muted)] ml-2">({stats.topModel[1]}x)</span>
          </div>
        )}

        {/* Per-model bar chart */}
        {stats.modelEntries.length > 0 && (
          <div className="p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
            <h4 className="text-xs font-medium text-[var(--color-text-secondary)] mb-3">{t('analytics.perModel')}</h4>
            <div className="space-y-2">
              {stats.modelEntries.map(([modelId, count]) => {
                const pct = Math.max((count / stats.maxCount) * 100, 4);
                return (
                  <div key={modelId}>
                    <div className="flex items-center justify-between mb-0.5">
                      <span className="text-[11px] text-[var(--color-text-primary)] truncate max-w-[60%]">{getModelName(modelId)}</span>
                      <span className="text-[10px] text-[var(--color-text-muted)]">{count}</span>
                    </div>
                    <div className="w-full h-2 rounded-full bg-[var(--color-bg-hover)]">
                      <div
                        className="h-2 rounded-full bg-[var(--color-accent)] transition-all duration-300"
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
