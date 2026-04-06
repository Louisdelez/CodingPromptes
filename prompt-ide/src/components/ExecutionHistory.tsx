import { useState, useEffect } from 'react';
import { Trash2, Clock, Hash, Coins, ChevronDown, ChevronRight } from 'lucide-react';
import type { ExecutionResult } from '../lib/types';
import { MODELS } from '../lib/types';
import * as backend from '../lib/backend';
import { formatCost, formatTokens } from '../lib/tokens';
import { useT } from '../lib/i18n';

interface ExecutionHistoryProps {
  projectId: string;
}

function backendExecutionToLocal(be: backend.BackendExecution): ExecutionResult {
  return {
    id: be.id,
    projectId: be.project_id,
    prompt: be.prompt,
    model: be.model,
    provider: be.provider,
    response: be.response,
    tokensIn: be.tokens_in,
    tokensOut: be.tokens_out,
    costEstimate: be.cost,
    latencyMs: be.latency_ms,
    temperature: 0,
    maxTokens: 0,
    createdAt: be.created_at,
  };
}

export function ExecutionHistory({ projectId }: ExecutionHistoryProps) {
  const t = useT();
  const [executions, setExecutions] = useState<ExecutionResult[]>([]);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const results = await backend.listExecutions(projectId);
        setExecutions(results.map(backendExecutionToLocal).sort((a, b) => b.createdAt - a.createdAt));
      } catch {
        // ignore
      }
    };
    load();
    const interval = setInterval(load, 2000);
    return () => clearInterval(interval);
  }, [projectId]);

  const clearHistory = () => {
    // No bulk delete endpoint; just clear local state
    setExecutions([]);
  };

  const toggleExpand = (id: string) => {
    setExpandedId((prev) => (prev === id ? null : id));
  };

  const getModelName = (modelId: string) => {
    return MODELS.find((m) => m.id === modelId)?.name ?? modelId;
  };

  const formatDate = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }) +
      ' ' +
      d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
        <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">
          {t('history.title')}
        </h3>
        {executions.length > 0 && (
          <button
            onClick={clearHistory}
            className="flex items-center gap-1.5 px-2 py-1 rounded text-xs text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
          >
            <Trash2 size={12} />
            {t('history.clear')}
          </button>
        )}
      </div>

      {/* List */}
      <div className="flex-1 overflow-auto">
        {executions.length === 0 ? (
          <div className="flex items-center justify-center h-full text-sm text-[var(--color-text-muted)]">
            {t('history.empty')}
          </div>
        ) : (
          <div className="divide-y divide-[var(--color-border)]">
            {executions.map((exec) => {
              const isExpanded = expandedId === exec.id;
              const preview = exec.response.length > 100
                ? exec.response.slice(0, 100) + '...'
                : exec.response;

              return (
                <div key={exec.id} className="animate-fadeIn">
                  <button
                    onClick={() => toggleExpand(exec.id)}
                    className="w-full text-left p-3 hover:bg-[var(--color-bg-hover)] transition-colors"
                  >
                    <div className="flex items-center justify-between mb-1.5">
                      <span className="text-xs font-medium text-[var(--color-accent)]">
                        {getModelName(exec.model)}
                      </span>
                      <div className="flex items-center gap-0.5">
                        {isExpanded ? (
                          <ChevronDown size={12} className="text-[var(--color-text-muted)]" />
                        ) : (
                          <ChevronRight size={12} className="text-[var(--color-text-muted)]" />
                        )}
                      </div>
                    </div>

                    <div className="flex items-center gap-3 text-[10px] text-[var(--color-text-muted)] mb-1.5">
                      <span>{formatDate(exec.createdAt)}</span>
                      <span className="flex items-center gap-0.5">
                        <Clock size={9} /> {exec.latencyMs}ms
                      </span>
                      <span className="flex items-center gap-0.5">
                        <Hash size={9} /> {formatTokens(exec.tokensIn + exec.tokensOut)}
                      </span>
                      {exec.costEstimate > 0 && (
                        <span className="flex items-center gap-0.5">
                          <Coins size={9} /> {formatCost(exec.costEstimate)}
                        </span>
                      )}
                    </div>

                    {!isExpanded && (
                      <p className="text-xs text-[var(--color-text-secondary)] leading-relaxed line-clamp-2">
                        {preview}
                      </p>
                    )}
                  </button>

                  {isExpanded && (
                    <div className="px-3 pb-3">
                      <pre className="whitespace-pre-wrap text-xs font-mono text-[var(--color-text-primary)] leading-relaxed p-2 rounded bg-[var(--color-bg-tertiary)] max-h-64 overflow-auto">
                        {exec.response}
                      </pre>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
