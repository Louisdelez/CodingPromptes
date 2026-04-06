import { useMemo } from 'react';
import { Coins, Hash, Zap, Type, WrapText, AlignLeft } from 'lucide-react';
import type { PromptBlock } from '../lib/types';
import { MODELS } from '../lib/types';
import { countTokens, estimateCost, formatCost, formatTokens } from '../lib/tokens';
import { compilePrompt } from '../lib/prompt';
import { useT } from '../lib/i18n';

interface TokenCounterProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
  selectedModel: string;
  onModelChange: (modelId: string) => void;
}

function formatNumber(n: number): string {
  if (n < 1000) return n.toString();
  if (n < 1000000) return `${(n / 1000).toFixed(1)}k`;
  return `${(n / 1000000).toFixed(1)}M`;
}

export function TokenCounter({ blocks, variables, selectedModel, onModelChange }: TokenCounterProps) {
  const t = useT();
  const model = MODELS.find((m) => m.id === selectedModel) ?? MODELS[0];

  const stats = useMemo(() => {
    const compiled = compilePrompt(blocks, variables);
    const tokens = countTokens(compiled);
    const enabledBlocks = blocks.filter((b) => b.enabled).length;
    const totalBlocks = blocks.length;
    const vars = new Set(compiled.match(/\{\{\w+\}\}/g) ?? []);
    const unresolvedVars = vars.size;
    const cost = estimateCost(tokens, Math.round(tokens * 0.5), model);
    const pctContext = model.maxContext > 0 ? (tokens / model.maxContext) * 100 : 0;

    // Characters, words, lines
    const chars = compiled.length;
    const words = compiled.trim() ? compiled.trim().split(/\s+/).length : 0;
    const lines = compiled ? compiled.split('\n').length : 0;

    return { tokens, enabledBlocks, totalBlocks, unresolvedVars, cost, pctContext, chars, words, lines };
  }, [blocks, variables, model]);

  return (
    <div className="bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)] text-xs">
      {/* Row 1: chars, words, lines, tokens, cost */}
      <div className="flex flex-wrap items-center gap-3 px-4 py-2">
        <div className="flex items-center gap-1 text-[var(--color-text-secondary)]">
          <Type size={12} />
          <span className="font-mono text-[var(--color-text-primary)]">{formatNumber(stats.chars)}</span>
          <span>{t('counter.chars')}</span>
        </div>

        <div className="flex items-center gap-1 text-[var(--color-text-secondary)]">
          <WrapText size={12} />
          <span className="font-mono text-[var(--color-text-primary)]">{formatNumber(stats.words)}</span>
          <span>{t('counter.words')}</span>
        </div>

        <div className="flex items-center gap-1 text-[var(--color-text-secondary)]">
          <AlignLeft size={12} />
          <span className="font-mono text-[var(--color-text-primary)]">{stats.lines}</span>
          <span>{t('counter.lines')}</span>
        </div>

        <div className="w-px h-3 bg-[var(--color-border)]" />

        <div className="flex items-center gap-1 text-[var(--color-text-secondary)]">
          <Hash size={12} />
          <span className="font-mono font-medium text-[var(--color-text-primary)]">{formatTokens(stats.tokens)}</span>
          <span>{t('counter.tokens')}</span>
        </div>

        <div className="flex items-center gap-1 text-[var(--color-text-secondary)]">
          <Coins size={12} />
          <span className="font-mono text-[var(--color-text-primary)]">~{formatCost(stats.cost)}</span>
        </div>

        <div className="flex items-center gap-1 text-[var(--color-text-secondary)]">
          <Zap size={12} />
          <span>{stats.enabledBlocks}/{stats.totalBlocks} {t('counter.blocks')}</span>
        </div>

        {stats.unresolvedVars > 0 && (
          <div className="flex items-center gap-1 text-[var(--color-warning)]">
            <span>{stats.unresolvedVars} {t('counter.unresolvedVars')}</span>
          </div>
        )}

        {/* Context bar + model selector pushed right */}
        <div className="flex items-center gap-2 flex-1 min-w-[140px] justify-end">
          <div className="w-20 h-1.5 rounded-full bg-[var(--color-bg-hover)] overflow-hidden">
            <div
              className="h-full rounded-full transition-all"
              style={{
                width: `${Math.min(stats.pctContext, 100)}%`,
                backgroundColor: stats.pctContext > 80 ? 'var(--color-danger)' : stats.pctContext > 50 ? 'var(--color-warning)' : 'var(--color-accent)',
              }}
            />
          </div>
          <span className="text-[10px] text-[var(--color-text-muted)]">{stats.pctContext.toFixed(1)}%</span>

          <select
            value={selectedModel}
            onChange={(e) => onModelChange(e.target.value)}
            className="bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded px-1.5 py-0.5 text-xs text-[var(--color-text-primary)] focus:border-[var(--color-accent)] outline-none"
          >
            {MODELS.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
