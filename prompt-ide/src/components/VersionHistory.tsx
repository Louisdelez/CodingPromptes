import { useState, useEffect } from 'react';
import { History, Save, RotateCcw, ChevronDown, ChevronRight, ArrowLeftRight } from 'lucide-react';
import type { PromptVersion, PromptBlock } from '../lib/types';
import { db } from '../lib/db';
import { useT } from '../lib/i18n';
import { compilePrompt } from '../lib/prompt';

interface VersionHistoryProps {
  projectId: string;
  currentBlocks: PromptBlock[];
  variables: Record<string, string>;
  onSaveVersion: (label: string) => void;
  onRestoreVersion: (blocks: PromptBlock[], variables: Record<string, string>) => void;
}

function computeDiff(oldText: string, newText: string): { type: 'same' | 'added' | 'removed'; text: string }[] {
  const oldLines = oldText.split('\n');
  const newLines = newText.split('\n');
  const result: { type: 'same' | 'added' | 'removed'; text: string }[] = [];
  const maxLen = Math.max(oldLines.length, newLines.length);

  for (let i = 0; i < maxLen; i++) {
    const oldLine = oldLines[i];
    const newLine = newLines[i];
    if (oldLine === newLine) {
      result.push({ type: 'same', text: oldLine ?? '' });
    } else {
      if (oldLine !== undefined) result.push({ type: 'removed', text: oldLine });
      if (newLine !== undefined) result.push({ type: 'added', text: newLine });
    }
  }
  return result;
}

export function VersionHistory({ projectId, currentBlocks, variables, onSaveVersion, onRestoreVersion }: VersionHistoryProps) {
  const t = useT();
  const [versions, setVersions] = useState<PromptVersion[]>([]);
  const [label, setLabel] = useState('');
  const [expanded, setExpanded] = useState<string | null>(null);
  const [diffVersionId, setDiffVersionId] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      const all = await db.versions.where('projectId').equals(projectId).reverse().sortBy('createdAt');
      setVersions(all);
    };
    load();
    const interval = setInterval(load, 3000);
    return () => clearInterval(interval);
  }, [projectId]);

  const handleSave = () => {
    if (!label.trim()) return;
    onSaveVersion(label.trim());
    setLabel('');
  };

  const formatDate = (ts: number) =>
    new Date(ts).toLocaleString('fr-FR', {
      day: 'numeric',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit',
    });

  const diffVersion = diffVersionId ? versions.find((v) => v.id === diffVersionId) : null;
  const diff = diffVersion
    ? computeDiff(
        compilePrompt(diffVersion.blocks, diffVersion.variables),
        compilePrompt(currentBlocks, variables)
      )
    : null;

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 space-y-3 border-b border-[var(--color-border)]">
        <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
          <History size={14} />
          <span>{t('versions.title')}</span>
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSave()}
            placeholder={t('versions.label')}
            className="flex-1 px-2 py-1.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
          />
          <button
            onClick={handleSave}
            disabled={!label.trim()}
            className="flex items-center gap-1 px-3 py-1.5 rounded text-xs bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 transition-colors"
          >
            <Save size={12} />
            {t('versions.save')}
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        {versions.length === 0 ? (
          <div className="p-4 text-sm text-[var(--color-text-muted)] text-center">
            {t('versions.empty')}
          </div>
        ) : (
          <div className="divide-y divide-[var(--color-border)]">
            {versions.map((v) => (
              <div key={v.id} className="animate-fadeIn">
                <button
                  onClick={() => setExpanded(expanded === v.id ? null : v.id)}
                  className="w-full text-left px-4 py-3 hover:bg-[var(--color-bg-hover)] transition-colors flex items-center gap-2"
                >
                  {expanded === v.id ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium text-[var(--color-text-primary)] truncate">{v.label}</div>
                    <div className="text-xs text-[var(--color-text-muted)]">{formatDate(v.createdAt)}</div>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setDiffVersionId(diffVersionId === v.id ? null : v.id);
                    }}
                    className={`p-1.5 rounded hover:bg-[var(--color-bg-tertiary)] transition-colors ${
                      diffVersionId === v.id
                        ? 'text-[var(--color-accent)]'
                        : 'text-[var(--color-text-muted)] hover:text-[var(--color-accent)]'
                    }`}
                    title={t('versions.compare')}
                  >
                    <ArrowLeftRight size={14} />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onRestoreVersion(v.blocks, v.variables);
                    }}
                    className="p-1.5 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] hover:text-[var(--color-accent)]"
                    title={t('versions.restore')}
                  >
                    <RotateCcw size={14} />
                  </button>
                </button>
                {expanded === v.id && (
                  <div className="px-4 pb-3">
                    <pre className="text-xs font-mono text-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)] p-2 rounded max-h-40 overflow-auto whitespace-pre-wrap">
                      {v.blocks
                        .filter((b) => b.enabled)
                        .map((b) => b.content)
                        .join('\n\n')}
                    </pre>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Diff view */}
        {diff && diffVersion && (
          <div className="border-t border-[var(--color-border)] p-4 space-y-2">
            <div className="flex items-center justify-between text-xs font-medium">
              <span className="text-red-400">{t('versions.version')}: {diffVersion.label}</span>
              <span className="text-[var(--color-text-muted)]">vs</span>
              <span className="text-green-400">{t('versions.current')}</span>
            </div>
            <div className="space-y-0 font-mono text-xs max-h-60 overflow-auto rounded bg-[var(--color-bg-tertiary)]">
              {diff.map((line, i) => (
                <div key={i} className={`px-2 py-0.5 ${
                  line.type === 'removed' ? 'bg-red-500/10 text-red-400' :
                  line.type === 'added' ? 'bg-green-500/10 text-green-400' :
                  'text-[var(--color-text-muted)]'
                }`}>
                  <span className="inline-block w-4 text-right mr-2 opacity-50">
                    {line.type === 'removed' ? '-' : line.type === 'added' ? '+' : ' '}
                  </span>
                  {line.text || ' '}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
