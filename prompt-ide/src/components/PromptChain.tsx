import { useState, useEffect } from 'react';
import { Play, Loader2 } from 'lucide-react';
import type { PromptProject, Workspace } from '../lib/types';
import { MODELS } from '../lib/types';
import { compilePrompt } from '../lib/prompt';
import { callLLM } from '../lib/api';
import { db, getApiKeys } from '../lib/db';
import { useT } from '../lib/i18n';

interface PromptChainProps {
  userId: string;
}

interface StepResult {
  stepIndex: number;
  promptName: string;
  output: string;
  error?: string;
}

export function PromptChain({ userId }: PromptChainProps) {
  const t = useT();
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [projects, setProjects] = useState<PromptProject[]>([]);
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string>('');
  const [selectedModelId, setSelectedModelId] = useState<string>('gpt-4o-mini');
  const [running, setRunning] = useState(false);
  const [results, setResults] = useState<StepResult[]>([]);

  useEffect(() => {
    db.workspaces.where('userId').equals(userId).toArray().then(setWorkspaces);
  }, [userId]);

  useEffect(() => {
    if (selectedWorkspaceId) {
      db.projects.where('workspaceId').equals(selectedWorkspaceId).toArray().then(setProjects);
    } else {
      setProjects([]);
    }
    setResults([]);
  }, [selectedWorkspaceId]);

  const sortedProjects = [...projects].sort((a, b) => a.createdAt - b.createdAt);

  const runChain = async () => {
    if (sortedProjects.length === 0) return;
    setRunning(true);
    setResults([]);

    const apiKeys = getApiKeys();
    const model = MODELS.find((m) => m.id === selectedModelId) ?? MODELS[0];
    const chainVars: Record<string, string> = {};

    for (let i = 0; i < sortedProjects.length; i++) {
      const project = sortedProjects[i];
      const mergedVars = { ...project.variables, ...chainVars };
      const compiled = compilePrompt(project.blocks, mergedVars);

      try {
        const response = await callLLM(compiled, model, apiKeys, { temperature: 0.7, maxTokens: 2048 });
        chainVars[`chain_output_${i + 1}`] = response.text;
        setResults((prev) => [...prev, { stepIndex: i + 1, promptName: project.name, output: response.text }]);
      } catch (err) {
        setResults((prev) => [...prev, {
          stepIndex: i + 1, promptName: project.name, output: '',
          error: err instanceof Error ? err.message : 'Unknown error',
        }]);
        break;
      }
    }
    setRunning(false);
  };

  return (
    <div className="p-4 space-y-4">
      <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">{t('chain.title')}</h3>

      <select
        value={selectedWorkspaceId}
        onChange={(e) => setSelectedWorkspaceId(e.target.value)}
        className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
      >
        <option value="">{t('chain.selectWorkspace')}</option>
        {workspaces.map((ws) => (
          <option key={ws.id} value={ws.id}>{ws.name}</option>
        ))}
      </select>

      <select
        value={selectedModelId}
        onChange={(e) => setSelectedModelId(e.target.value)}
        className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
      >
        {MODELS.map((m) => (
          <option key={m.id} value={m.id}>{m.name} ({m.provider})</option>
        ))}
      </select>

      {!selectedWorkspaceId ? (
        <p className="text-xs text-[var(--color-text-muted)] italic">{t('chain.noWorkspace')}</p>
      ) : sortedProjects.length === 0 ? (
        <p className="text-xs text-[var(--color-text-muted)] italic">{t('chain.noPrompts')}</p>
      ) : (
        <>
          <div className="space-y-1">
            {sortedProjects.map((project, i) => (
              <div key={project.id} className="flex items-center gap-2 px-3 py-2 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
                <span className="flex-shrink-0 w-6 h-6 rounded-full bg-[var(--color-accent)] text-white text-xs font-bold flex items-center justify-center">{i + 1}</span>
                <span className="text-sm text-[var(--color-text-primary)] truncate">{project.name}</span>
              </div>
            ))}
          </div>

          <button onClick={runChain} disabled={running}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-white bg-[var(--color-accent)] hover:opacity-90 disabled:opacity-50 transition-opacity">
            {running ? <><Loader2 size={14} className="animate-spin" />{t('chain.running')}</> : <><Play size={14} />{t('chain.run')}</>}
          </button>
        </>
      )}

      {results.length > 0 && (
        <div className="space-y-3">
          {results.map((r) => (
            <div key={r.stepIndex} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] overflow-hidden">
              <div className="px-3 py-1.5 bg-[var(--color-bg-tertiary)] border-b border-[var(--color-border)]">
                <span className="text-xs font-medium text-[var(--color-text-secondary)]">{t('chain.step')} {r.stepIndex} — {r.promptName}</span>
              </div>
              <div className="p-3">
                {r.error ? (
                  <p className="text-xs text-[var(--color-danger)]">{r.error}</p>
                ) : (
                  <pre className="text-xs text-[var(--color-text-primary)] whitespace-pre-wrap font-mono leading-relaxed max-h-48 overflow-auto">{r.output}</pre>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
