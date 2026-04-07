import { useState, useEffect, useCallback } from 'react';
import { Cpu, Trash2, RefreshCw, Wifi, WifiOff, HardDrive } from 'lucide-react';
import { listNodes, deleteNode, parseCapabilities, type GpuNode } from '../lib/backend';
import { useT } from '../lib/i18n';

export function GpuFleet() {
  const t = useT();
  const [nodes, setNodes] = useState<GpuNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listNodes();
      setNodes(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Error');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  // Auto-refresh every 20s
  useEffect(() => {
    const iv = setInterval(refresh, 20000);
    return () => clearInterval(iv);
  }, [refresh]);

  const handleDelete = async (id: string) => {
    if (!confirm(t('fleet.confirm_delete'))) return;
    await deleteNode(id);
    setNodes(prev => prev.filter(n => n.id !== id));
  };

  return (
    <div className="p-4 space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Cpu size={16} className="text-[var(--color-accent)]" />
          <span className="text-sm font-semibold text-[var(--color-text-primary)]">{t('fleet.title')}</span>
          <span className="text-xs text-[var(--color-text-muted)]">({nodes.length})</span>
        </div>
        <button
          onClick={refresh}
          className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)]"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>

      {error && (
        <div className="text-xs text-red-400 bg-red-400/10 px-3 py-2 rounded">{error}</div>
      )}

      {nodes.length === 0 && !loading && (
        <div className="text-xs text-[var(--color-text-muted)] text-center py-8">
          {t('fleet.empty')}
        </div>
      )}

      {nodes.map(node => {
        const caps = parseCapabilities(node);
        const isOnline = node.status === 'online';
        const ago = isOnline ? '' : ` (${formatAgo(node.last_heartbeat)})`;

        return (
          <div key={node.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3 space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {isOnline ? (
                  <Wifi size={13} className="text-green-400" />
                ) : (
                  <WifiOff size={13} className="text-zinc-500" />
                )}
                <span className="text-sm font-medium text-[var(--color-text-primary)]">{node.name}</span>
              </div>
              <button
                onClick={() => handleDelete(node.id)}
                className="p-1 rounded hover:bg-red-500/10 text-[var(--color-text-muted)] hover:text-red-400"
              >
                <Trash2 size={12} />
              </button>
            </div>

            <div className="flex flex-wrap gap-2 text-[10px]">
              {node.gpu_info && (
                <span className="flex items-center gap-1 text-[var(--color-text-muted)]">
                  <HardDrive size={10} />
                  {node.gpu_info}
                </span>
              )}
              <span className={`px-1.5 py-0.5 rounded ${isOnline ? 'bg-green-500/10 text-green-400' : 'bg-zinc-500/10 text-zinc-400'}`}>
                {node.status}{ago}
              </span>
            </div>

            {/* Capabilities */}
            <div className="space-y-1">
              {caps.stt.model_loaded && (
                <div className="flex items-center gap-1.5 text-[10px]">
                  <span className="px-1.5 py-0.5 rounded bg-[var(--color-accent)]/10 text-[var(--color-accent)]">STT</span>
                  <span className="text-[var(--color-text-muted)]">{caps.stt.active_model || 'loaded'}</span>
                </div>
              )}
              {caps.llm.ollama_connected && caps.llm.models.length > 0 && (
                <div className="flex items-center gap-1.5 text-[10px] flex-wrap">
                  <span className="px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-400">LLM</span>
                  {caps.llm.models.map(m => (
                    <span key={m.name} className="text-[var(--color-text-muted)]">{m.name}</span>
                  ))}
                </div>
              )}
              {!caps.stt.model_loaded && !caps.llm.ollama_connected && (
                <span className="text-[10px] text-[var(--color-text-muted)]">{t('fleet.no_capabilities')}</span>
              )}
            </div>

            <div className="text-[9px] text-[var(--color-text-muted)] truncate">
              {node.address}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function formatAgo(ts: number): string {
  const diff = Date.now() - ts;
  if (diff < 60_000) return '<1m';
  if (diff < 3600_000) return `${Math.floor(diff / 60_000)}m`;
  if (diff < 86400_000) return `${Math.floor(diff / 3600_000)}h`;
  return `${Math.floor(diff / 86400_000)}d`;
}
