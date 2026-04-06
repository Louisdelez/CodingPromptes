import { useMemo, useState } from 'react';
import { Copy, Check, FileText } from 'lucide-react';
import type { PromptBlock } from '../lib/types';
import { compilePrompt } from '../lib/prompt';
import { useT } from '../lib/i18n';

interface PreviewPanelProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
}

export function PreviewPanel({ blocks, variables }: PreviewPanelProps) {
  const t = useT();
  const [copied, setCopied] = useState(false);
  const compiled = useMemo(() => compilePrompt(blocks, variables), [blocks, variables]);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(compiled);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--color-border)]">
        <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
          <FileText size={14} />
          <span>{t('preview.title')}</span>
        </div>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1.5 px-2 py-1 rounded text-xs bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
        >
          {copied ? <Check size={12} className="text-[var(--color-success)]" /> : <Copy size={12} />}
          {copied ? t('preview.copied') : t('preview.copy')}
        </button>
      </div>
      <div className="flex-1 overflow-auto p-4">
        {compiled ? (
          <pre className="whitespace-pre-wrap text-sm font-mono text-[var(--color-text-primary)] leading-relaxed">
            {compiled}
          </pre>
        ) : (
          <p className="text-sm text-[var(--color-text-muted)] italic">
            {t('preview.empty')}
          </p>
        )}
      </div>
    </div>
  );
}
