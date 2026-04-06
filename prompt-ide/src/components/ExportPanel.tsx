import { useState, useRef } from 'react';
import { Download, Check, Upload, AlertCircle, Archive } from 'lucide-react';
import JSZip from 'jszip';
import type { PromptProject, PromptBlock } from '../lib/types';
import { compilePrompt } from '../lib/prompt';
import * as backend from '../lib/backend';
import { useT } from '../lib/i18n';

interface ExportPanelProps {
  project: PromptProject;
  onImport: (blocks: PromptBlock[], variables: Record<string, string>, name?: string) => void;
}

type ExportFormat = 'txt' | 'json' | 'json-openai' | 'json-anthropic' | 'md';

export function ExportPanel({ project, onImport }: ExportPanelProps) {
  const t = useT();
  const [exported, setExported] = useState<string | null>(null);
  const [importStatus, setImportStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [zipImportStatus, setZipImportStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const fileInputRef = useRef<HTMLInputElement>(null);
  const zipInputRef = useRef<HTMLInputElement>(null);

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = () => {
      try {
        const data = JSON.parse(reader.result as string);
        if (!data.blocks || !Array.isArray(data.blocks)) {
          setImportStatus('error');
          setTimeout(() => setImportStatus('idle'), 3000);
          return;
        }
        onImport(data.blocks, data.variables || {}, data.name);
        setImportStatus('success');
        setTimeout(() => setImportStatus('idle'), 3000);
      } catch {
        setImportStatus('error');
        setTimeout(() => setImportStatus('idle'), 3000);
      }
    };
    reader.readAsText(file);
    // Reset file input so the same file can be re-imported
    e.target.value = '';
  };

  const handleExport = (format: ExportFormat) => {
    const compiled = compilePrompt(project.blocks, project.variables);

    let content: string;
    let filename: string;
    let mime: string;

    switch (format) {
      case 'txt':
        content = compiled;
        filename = `${project.name}.txt`;
        mime = 'text/plain';
        break;
      case 'md':
        content = `# ${project.name}\n\n${compiled}`;
        filename = `${project.name}.md`;
        mime = 'text/markdown';
        break;
      case 'json':
        content = JSON.stringify({
          name: project.name,
          framework: project.framework,
          blocks: project.blocks,
          variables: project.variables,
          compiled,
          exportedAt: new Date().toISOString(),
        }, null, 2);
        filename = `${project.name}.json`;
        mime = 'application/json';
        break;
      case 'json-openai':
        content = JSON.stringify({
          model: 'gpt-4o',
          messages: [{ role: 'user', content: compiled }],
          temperature: 0.7,
        }, null, 2);
        filename = `${project.name}-openai.json`;
        mime = 'application/json';
        break;
      case 'json-anthropic':
        content = JSON.stringify({
          model: 'claude-sonnet-4-6',
          max_tokens: 2048,
          messages: [{ role: 'user', content: compiled }],
        }, null, 2);
        filename = `${project.name}-anthropic.json`;
        mime = 'application/json';
        break;
    }

    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
    setExported(format);
    setTimeout(() => setExported(null), 2000);
  };

  const formats: { id: ExportFormat; label: string; desc: string }[] = [
    { id: 'txt', label: t('export.txt'), desc: '.txt' },
    { id: 'md', label: t('export.md'), desc: '.md' },
    { id: 'json', label: t('export.json'), desc: t('export.jsonDesc') },
    { id: 'json-openai', label: t('export.openai'), desc: t('export.apiDesc') },
    { id: 'json-anthropic', label: t('export.anthropic'), desc: t('export.apiDesc') },
  ];

  const handleExportWorkspace = async () => {
    try {
      const allProjects = await backend.listProjects();
      const wsProjects = allProjects.filter(p => p.workspace_id === project.workspaceId);

      const zip = new JSZip();
      for (const p of wsProjects) {
        const data = {
          name: p.name,
          blocks: JSON.parse(p.blocks_json),
          variables: JSON.parse(p.variables_json),
          framework: p.framework,
        };
        zip.file(`${p.name}.json`, JSON.stringify(data, null, 2));
      }

      const blob = await zip.generateAsync({ type: 'blob' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `workspace-export.zip`;
      a.click();
      URL.revokeObjectURL(url);
      setExported('zip');
      setTimeout(() => setExported(null), 2000);
    } catch {
      // ignore
    }
  };

  const handleImportZip = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      const zip = await JSZip.loadAsync(file);
      const jsonFiles = Object.keys(zip.files).filter(name => name.endsWith('.json'));

      for (const name of jsonFiles) {
        const content = await zip.files[name].async('text');
        const data = JSON.parse(content);
        if (data.blocks && Array.isArray(data.blocks)) {
          await backend.createProject({
            name: data.name || name.replace('.json', ''),
            blocks_json: JSON.stringify(data.blocks),
            variables_json: JSON.stringify(data.variables || {}),
            workspace_id: project.workspaceId ?? null,
            framework: data.framework ?? null,
          });
        }
      }

      setZipImportStatus('success');
      setTimeout(() => setZipImportStatus('idle'), 3000);
    } catch {
      setZipImportStatus('error');
      setTimeout(() => setZipImportStatus('idle'), 3000);
    }
    e.target.value = '';
  };

  return (
    <div className="p-4 space-y-3">
      {/* Import section */}
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        <Upload size={14} />
        <span>{t('import.title')}</span>
      </div>
      <div>
        <input
          ref={fileInputRef}
          type="file"
          accept=".json"
          onChange={handleImport}
          className="hidden"
        />
        <button
          onClick={() => fileInputRef.current?.click()}
          className="w-full flex items-center justify-between p-2.5 rounded-lg border border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)] transition-colors text-left"
        >
          <div>
            <div className="text-sm text-[var(--color-text-primary)]">{t('import.button')}</div>
            <div className="text-xs text-[var(--color-text-muted)]">.json</div>
          </div>
          {importStatus === 'success' ? (
            <Check size={16} className="text-[var(--color-success)]" />
          ) : importStatus === 'error' ? (
            <AlertCircle size={16} className="text-[var(--color-danger)]" />
          ) : (
            <Upload size={14} className="text-[var(--color-text-muted)]" />
          )}
        </button>
        {importStatus === 'success' && (
          <p className="text-xs text-[var(--color-success)] mt-1">{t('import.success')}</p>
        )}
        {importStatus === 'error' && (
          <p className="text-xs text-[var(--color-danger)] mt-1">{t('import.error')}</p>
        )}
      </div>

      <div className="h-px bg-[var(--color-border)]" />

      {/* Export section */}
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        <Download size={14} />
        <span>{t('export.title')}</span>
      </div>
      <div className="grid gap-2">
        {formats.map((f) => (
          <button
            key={f.id}
            onClick={() => handleExport(f.id)}
            className="flex items-center justify-between p-2.5 rounded-lg border border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)] transition-colors text-left"
          >
            <div>
              <div className="text-sm text-[var(--color-text-primary)]">{f.label}</div>
              <div className="text-xs text-[var(--color-text-muted)]">{f.desc}</div>
            </div>
            {exported === f.id ? (
              <Check size={16} className="text-[var(--color-success)]" />
            ) : (
              <Download size={14} className="text-[var(--color-text-muted)]" />
            )}
          </button>
        ))}
      </div>

      {/* Workspace ZIP export */}
      {project.workspaceId && (
        <>
          <div className="h-px bg-[var(--color-border)]" />
          <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
            <Archive size={14} />
            <span>{t('export.workspace')}</span>
          </div>
          <button
            onClick={handleExportWorkspace}
            className="w-full flex items-center justify-between p-2.5 rounded-lg border border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)] transition-colors text-left"
          >
            <div>
              <div className="text-sm text-[var(--color-text-primary)]">{t('export.workspace')}</div>
              <div className="text-xs text-[var(--color-text-muted)]">{t('export.workspaceDesc')}</div>
            </div>
            {exported === 'zip' ? (
              <Check size={16} className="text-[var(--color-success)]" />
            ) : (
              <Archive size={14} className="text-[var(--color-text-muted)]" />
            )}
          </button>
        </>
      )}

      {/* Import ZIP */}
      <div className="h-px bg-[var(--color-border)]" />
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        <Upload size={14} />
        <span>{t('import.zip')}</span>
      </div>
      <div>
        <input
          ref={zipInputRef}
          type="file"
          accept=".zip"
          onChange={handleImportZip}
          className="hidden"
        />
        <button
          onClick={() => zipInputRef.current?.click()}
          className="w-full flex items-center justify-between p-2.5 rounded-lg border border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)] transition-colors text-left"
        >
          <div>
            <div className="text-sm text-[var(--color-text-primary)]">{t('import.zip')}</div>
            <div className="text-xs text-[var(--color-text-muted)]">.zip</div>
          </div>
          {zipImportStatus === 'success' ? (
            <Check size={16} className="text-[var(--color-success)]" />
          ) : zipImportStatus === 'error' ? (
            <AlertCircle size={16} className="text-[var(--color-danger)]" />
          ) : (
            <Upload size={14} className="text-[var(--color-text-muted)]" />
          )}
        </button>
        {zipImportStatus === 'success' && (
          <p className="text-xs text-[var(--color-success)] mt-1">{t('import.zipSuccess')}</p>
        )}
        {zipImportStatus === 'error' && (
          <p className="text-xs text-[var(--color-danger)] mt-1">{t('import.error')}</p>
        )}
      </div>
    </div>
  );
}
