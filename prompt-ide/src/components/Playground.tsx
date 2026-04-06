import { useState, useCallback, useEffect, useMemo } from 'react';
import { Play, Loader2, Settings, AlertCircle, Clock, Hash, Coins, Server, Wifi, WifiOff } from 'lucide-react';
import { v4 as uuid } from 'uuid';
import type { PromptBlock, ApiKeys, ExecutionResult, ModelConfig } from '../lib/types';
import { MODELS, getLocalServerUrl, setLocalServerUrl } from '../lib/types';
import { compilePrompt } from '../lib/prompt';
import { callLLMStream, fetchLocalModels } from '../lib/api';
import { estimateCost, formatCost, formatTokens } from '../lib/tokens';
import { getApiKeys, setApiKeys } from '../lib/db';
import * as backend from '../lib/backend';
import { useT } from '../lib/i18n';
import { renderMarkdown } from '../lib/markdown';

interface PlaygroundProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
  projectId: string;
  triggerExecute?: boolean;
  onExecuteTriggered?: () => void;
}

export function Playground({ blocks, variables, projectId, triggerExecute, onExecuteTriggered }: PlaygroundProps) {
  const t = useT();
  const [selectedModels, setSelectedModels] = useState<string[]>(['gpt-4o-mini']);
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState(2048);
  const [results, setResults] = useState<(ExecutionResult & { error?: string; streaming?: boolean })[]>([]);
  const [loading, setLoading] = useState<Record<string, boolean>>({});
  const [showSettings, setShowSettings] = useState(false);
  const [apiKeys, setApiKeysState] = useState<ApiKeys>(getApiKeys);

  // Local server
  const [localUrl, setLocalUrl] = useState(getLocalServerUrl);
  const [localModels, setLocalModels] = useState<{ id: string; name: string }[]>([]);
  const [localConnected, setLocalConnected] = useState(false);

  // Fetch local models on mount and when URL changes
  useEffect(() => {
    const check = async () => {
      setLocalServerUrl(localUrl);
      const models = await fetchLocalModels();
      setLocalModels(models);
      setLocalConnected(models.length > 0);
    };
    check();
    const interval = setInterval(check, 10000);
    return () => clearInterval(interval);
  }, [localUrl]);

  // Build combined model list: local models first, then cloud
  const allModels: ModelConfig[] = useMemo(() => [
    ...localModels.map((m) => ({
      id: m.id,
      name: m.name,
      provider: 'local' as const,
      inputCostPer1k: 0,
      outputCostPer1k: 0,
      maxContext: 128000,
    })),
    ...MODELS,
  ], [localModels]);

  const updateApiKey = (provider: keyof ApiKeys, value: string) => {
    const updated = { ...apiKeys, [provider]: value };
    setApiKeysState(updated);
    setApiKeys(updated);
  };

  const toggleModel = (modelId: string) => {
    setSelectedModels((prev) =>
      prev.includes(modelId) ? prev.filter((m) => m !== modelId) : [...prev, modelId]
    );
  };

  const runPrompt = useCallback(async () => {
    const compiled = compilePrompt(blocks, variables);
    if (!compiled.trim()) return;

    setResults([]);

    for (const modelId of selectedModels) {
      const model = allModels.find((m) => m.id === modelId);
      if (!model) continue;

      // Local models don't need API keys
      if (model.provider !== 'local') {
        const providerKey = apiKeys[model.provider as keyof ApiKeys];
        if (!providerKey) {
          setResults((prev) => [
            ...prev,
            {
              id: uuid(), projectId, prompt: compiled, model: modelId,
              provider: model.provider, response: '', tokensIn: 0, tokensOut: 0,
              costEstimate: 0, latencyMs: 0, temperature, maxTokens, createdAt: Date.now(),
              error: t('playground.missingKey'),
            },
          ]);
          continue;
        }
      }

      const entryId = uuid();

      // Create entry with empty response immediately
      setResults((prev) => [
        ...prev,
        {
          id: entryId, projectId, prompt: compiled, model: modelId,
          provider: model.provider, response: '', tokensIn: 0, tokensOut: 0,
          costEstimate: 0, latencyMs: 0, temperature, maxTokens, createdAt: Date.now(),
          streaming: true,
        },
      ]);

      setLoading((prev) => ({ ...prev, [modelId]: true }));

      try {
        const result = await callLLMStream(compiled, model, apiKeys, { temperature, maxTokens }, (chunk) => {
          setResults((prev) =>
            prev.map((r) =>
              r.id === entryId ? { ...r, response: r.response + chunk } : r
            )
          );
        });

        const cost = estimateCost(result.tokensIn, result.tokensOut, model);

        const execution: ExecutionResult = {
          id: entryId, projectId, prompt: compiled, model: modelId,
          provider: model.provider, response: result.text,
          tokensIn: result.tokensIn, tokensOut: result.tokensOut,
          costEstimate: cost, latencyMs: result.latencyMs,
          temperature, maxTokens, createdAt: Date.now(),
        };

        // Save to backend
        backend.createExecution(projectId, {
          model: modelId,
          provider: model.provider,
          prompt: compiled,
          response: result.text,
          tokens_in: result.tokensIn,
          tokens_out: result.tokensOut,
          cost,
          latency_ms: result.latencyMs,
        }).catch(() => {});

        // Update the entry with final data (tokens, cost, latency)
        setResults((prev) =>
          prev.map((r) =>
            r.id === entryId ? { ...execution, streaming: false } : r
          )
        );
      } catch (err) {
        setResults((prev) =>
          prev.map((r) =>
            r.id === entryId
              ? {
                  ...r,
                  streaming: false,
                  error: err instanceof Error ? err.message : 'Erreur inconnue',
                }
              : r
          )
        );
      } finally {
        setLoading((prev) => ({ ...prev, [modelId]: false }));
      }
    }
  }, [blocks, variables, selectedModels, allModels, apiKeys, temperature, maxTokens, projectId, t]);

  // Handle triggerExecute from keyboard shortcut
  useEffect(() => {
    if (triggerExecute) {
      runPrompt();
      onExecuteTriggered?.();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [triggerExecute]);

  const isAnyLoading = Object.values(loading).some(Boolean);

  return (
    <div className="flex flex-col h-full">
      {/* Controls */}
      <div className="p-4 border-b border-[var(--color-border)] space-y-3">
        <div className="flex items-center gap-2">
          <button
            onClick={runPrompt}
            disabled={isAnyLoading || selectedModels.length === 0}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white font-medium text-sm disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isAnyLoading ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} />}
            {isAnyLoading ? t('playground.executing') : t('playground.execute')}
          </button>

          <button
            onClick={() => setShowSettings(!showSettings)}
            className={`p-2 rounded-lg border transition-colors ${
              showSettings
                ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10 text-[var(--color-accent)]'
                : 'border-[var(--color-border)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-secondary)]'
            }`}
          >
            <Settings size={16} />
          </button>
        </div>

        {/* Local models */}
        {localModels.length > 0 && (
          <div>
            <div className="flex items-center gap-1.5 mb-1.5">
              <Server size={11} className="text-[var(--color-success)]" />
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wider font-medium">
                {t('playground.localModels')}
              </span>
              <Wifi size={10} className="text-[var(--color-success)]" />
            </div>
            <div className="flex flex-wrap gap-1.5">
              {localModels.map((m) => (
                <button
                  key={m.id}
                  onClick={() => toggleModel(m.id)}
                  className={`px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
                    selectedModels.includes(m.id)
                      ? 'bg-[var(--color-success)] text-white'
                      : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-success)]/30'
                  }`}
                >
                  {m.name}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Cloud models */}
        <div>
          {localModels.length > 0 && (
            <div className="flex items-center gap-1.5 mb-1.5">
              <span className="text-[10px] text-[var(--color-text-muted)] uppercase tracking-wider font-medium">
                {t('playground.cloudModels')}
              </span>
            </div>
          )}
          <div className="flex flex-wrap gap-1.5">
            {MODELS.map((m) => (
              <button
                key={m.id}
                onClick={() => toggleModel(m.id)}
                className={`px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
                  selectedModels.includes(m.id)
                    ? 'bg-[var(--color-accent)] text-white'
                    : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                }`}
              >
                {m.name}
              </button>
            ))}
          </div>
        </div>

        {/* Settings panel */}
        {showSettings && (
          <div className="space-y-3 p-3 rounded-lg bg-[var(--color-bg-tertiary)] animate-fadeIn">
            {/* Local server */}
            <div className="space-y-1.5">
              <h4 className="text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider flex items-center gap-1.5">
                <Server size={11} />
                {t('playground.localServer')}
              </h4>
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={localUrl}
                  onChange={(e) => setLocalUrl(e.target.value)}
                  placeholder="http://localhost:8910"
                  className="flex-1 px-2 py-1 text-xs bg-[var(--color-bg-primary)] border border-[var(--color-border)] rounded focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)] font-mono"
                />
                {localConnected ? (
                  <Wifi size={14} className="text-[var(--color-success)]" />
                ) : (
                  <WifiOff size={14} className="text-[var(--color-danger)]" />
                )}
              </div>
              <div className="text-[10px] text-[var(--color-text-muted)]">
                {localConnected
                  ? `${t('playground.connected')} — ${localModels.length} ${t('playground.modelsDetected')}`
                  : t('playground.notConnected')}
              </div>
            </div>

            {/* Temperature / Max tokens */}
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs text-[var(--color-text-muted)] mb-1">
                  {t('playground.temperature')}: {temperature.toFixed(1)}
                </label>
                <input
                  type="range" min="0" max="2" step="0.1"
                  value={temperature}
                  onChange={(e) => setTemperature(parseFloat(e.target.value))}
                  className="w-full accent-[var(--color-accent)]"
                />
              </div>
              <div>
                <label className="block text-xs text-[var(--color-text-muted)] mb-1">
                  {t('playground.maxTokens')}: {maxTokens}
                </label>
                <input
                  type="range" min="256" max="8192" step="256"
                  value={maxTokens}
                  onChange={(e) => setMaxTokens(parseInt(e.target.value))}
                  className="w-full accent-[var(--color-accent)]"
                />
              </div>
            </div>

            {/* API keys */}
            <div className="space-y-2">
              <h4 className="text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider">{t('playground.apiKeys')}</h4>
              {(['openai', 'anthropic', 'google'] as const).map((provider) => (
                <div key={provider} className="flex items-center gap-2">
                  <label className="text-xs text-[var(--color-text-secondary)] w-20 capitalize">{provider}</label>
                  <input
                    type="password"
                    value={apiKeys[provider] ?? ''}
                    onChange={(e) => updateApiKey(provider, e.target.value)}
                    placeholder={`${provider === 'openai' ? 'sk-...' : provider === 'anthropic' ? 'sk-ant-...' : 'AI...'}`}
                    className="flex-1 px-2 py-1 text-xs bg-[var(--color-bg-primary)] border border-[var(--color-border)] rounded focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)] font-mono"
                  />
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Results */}
      <div className="flex-1 overflow-auto">
        {results.length === 0 ? (
          <div className="flex items-center justify-center h-full text-sm text-[var(--color-text-muted)]">
            {t('playground.selectModels')}
          </div>
        ) : (
          <div className={`grid gap-0 h-full ${results.length > 1 ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-1'}`}>
            {results.map((r) => {
              const model = allModels.find((m) => m.id === r.model);
              const isLocal = model?.provider === 'local';
              return (
                <div key={r.id} className="border-b border-r border-[var(--color-border)] p-4 overflow-auto animate-fadeIn">
                  <div className="flex items-center justify-between mb-3">
                    <span className={`text-sm font-medium ${isLocal ? 'text-[var(--color-success)]' : 'text-[var(--color-accent)]'}`}>
                      {isLocal && <Server size={12} className="inline mr-1" />}
                      {model?.name ?? r.model}
                    </span>
                    {!r.error && (
                      <div className="flex items-center gap-3 text-xs text-[var(--color-text-muted)]">
                        {r.streaming ? (
                          <span className="flex items-center gap-1 text-[var(--color-accent)]">
                            <Loader2 size={10} className="animate-spin" />
                            {t('playground.streaming')}
                          </span>
                        ) : (
                          <>
                            <span className="flex items-center gap-1"><Clock size={10} /> {r.latencyMs}ms</span>
                            <span className="flex items-center gap-1"><Hash size={10} /> {formatTokens(r.tokensIn + r.tokensOut)}</span>
                            {!isLocal && <span className="flex items-center gap-1"><Coins size={10} /> {formatCost(r.costEstimate)}</span>}
                            {isLocal && <span className="text-[var(--color-success)]">{t('playground.free')}</span>}
                          </>
                        )}
                      </div>
                    )}
                  </div>
                  {r.error ? (
                    <div className="flex items-start gap-2 text-[var(--color-danger)] text-sm">
                      <AlertCircle size={16} className="flex-shrink-0 mt-0.5" />
                      <span>{r.error}</span>
                    </div>
                  ) : (
                    <div
                      className="text-sm text-[var(--color-text-primary)] leading-relaxed prose-sm"
                      dangerouslySetInnerHTML={{ __html: renderMarkdown(r.response) }}
                    />
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
