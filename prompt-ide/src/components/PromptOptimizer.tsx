import { useState } from 'react';
import { Sparkles, Loader2, ArrowRight, AlertCircle } from 'lucide-react';
import type { PromptBlock } from '../lib/types';
import { MODELS } from '../lib/types';
import { compilePrompt } from '../lib/prompt';
import { optimizePrompt } from '../lib/api';
import { getApiKeys } from '../lib/db';
import { useT } from '../lib/i18n';

interface PromptOptimizerProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
  onApply: (optimizedText: string) => void;
}

export function PromptOptimizer({ blocks, variables, onApply }: PromptOptimizerProps) {
  const t = useT();
  const [optimized, setOptimized] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleOptimize = async () => {
    const compiled = compilePrompt(blocks, variables);
    if (!compiled.trim()) return;

    setLoading(true);
    setError(null);
    setOptimized(null);

    try {
      const apiKeys = getApiKeys();
      // Prefer Claude, then GPT, then Gemini
      let model = MODELS.find((m) => m.id === 'claude-sonnet-4-6');
      let hasKey = !!apiKeys.anthropic;

      if (!hasKey) {
        model = MODELS.find((m) => m.id === 'gpt-4o');
        hasKey = !!apiKeys.openai;
      }
      if (!hasKey) {
        model = MODELS.find((m) => m.id === 'gemini-2.5-flash');
        hasKey = !!apiKeys.google;
      }

      if (!hasKey || !model) {
        throw new Error(t('optimizer.needKey'));
      }

      const result = await optimizePrompt(compiled, model, apiKeys);
      setOptimized(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur inconnue');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-4 space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
          <Sparkles size={14} />
          <span>{t('optimizer.title')}</span>
        </div>
        <button
          onClick={handleOptimize}
          disabled={loading}
          className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white text-xs font-medium disabled:opacity-50 transition-all"
        >
          {loading ? <Loader2 size={14} className="animate-spin" /> : <Sparkles size={14} />}
          {loading ? t('optimizer.improving') : t('optimizer.improve')}
        </button>
      </div>

      {error && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-[var(--color-danger)]/10 text-[var(--color-danger)] text-sm animate-fadeIn">
          <AlertCircle size={16} className="flex-shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}

      {optimized && (
        <div className="space-y-2 animate-fadeIn">
          <pre className="whitespace-pre-wrap text-sm font-mono text-[var(--color-text-primary)] bg-[var(--color-bg-tertiary)] p-3 rounded-lg max-h-60 overflow-auto leading-relaxed border border-[var(--color-border)]">
            {optimized}
          </pre>
          <button
            onClick={() => onApply(optimized)}
            className="flex items-center gap-2 px-3 py-1.5 rounded text-xs bg-[var(--color-success)]/20 text-[var(--color-success)] hover:bg-[var(--color-success)]/30 transition-colors"
          >
            <ArrowRight size={14} />
            {t('optimizer.apply')}
          </button>
        </div>
      )}
    </div>
  );
}
