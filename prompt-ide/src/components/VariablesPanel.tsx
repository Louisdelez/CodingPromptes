import { Variable } from 'lucide-react';
import type { PromptBlock } from '../lib/types';
import { extractVariables } from '../lib/prompt';
import { useT } from '../lib/i18n';

interface VariablesPanelProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
  onSetVariable: (key: string, value: string) => void;
}

export function VariablesPanel({ blocks, variables, onSetVariable }: VariablesPanelProps) {
  const t = useT();
  const detectedVars = extractVariables(blocks);

  if (detectedVars.length === 0) {
    return (
      <div className="px-4 py-3 text-sm text-[var(--color-text-muted)] italic">
        {t('variables.use')} <code className="text-[var(--color-role)] bg-[var(--color-bg-tertiary)] px-1 rounded">{'{{variable}}'}</code> {t('variables.hint')}
      </div>
    );
  }

  return (
    <div className="p-4 space-y-3">
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        <Variable size={14} />
        <span>{t('variables.title')} ({detectedVars.length})</span>
      </div>
      {detectedVars.map((v) => (
        <div key={v} className="flex items-center gap-2">
          <span className="text-sm font-mono text-[var(--color-role)] min-w-[100px] truncate">
            {`{{${v}}}`}
          </span>
          <input
            type="text"
            value={variables[v] ?? ''}
            onChange={(e) => onSetVariable(v, e.target.value)}
            placeholder={`${t('variables.valueFor')} ${v}`}
            className="flex-1 px-2 py-1.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)]"
          />
        </div>
      ))}
    </div>
  );
}
