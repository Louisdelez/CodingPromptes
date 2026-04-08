import { useState, useRef, useCallback } from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
  GripVertical,
  Trash2,
  Eye,
  EyeOff,
  User,
  BookOpen,
  Target,
  Lightbulb,
  Shield,
  Layout,
  Mic,
  MicOff,
  Loader2,
  Sparkles,
  Wand2,
  HelpCircle,
  ExternalLink,
} from 'lucide-react';
import type { PromptBlock as PromptBlockType } from '../lib/types';
import { BLOCK_CONFIG, MODELS } from '../lib/types';
import { BlockEditor } from './BlockEditor';
import { createRecorder, transcribe, getSttConfig } from '../lib/stt';
import { callLLM } from '../lib/api';
import { getApiKeys } from '../lib/db';
import { buildGeneratePrompt, buildImprovePrompt, buildClarifyPrompt } from '../lib/sdd-prompts';
import { AudioBars } from './AudioBars';
import { useT, type TranslationKey } from '../lib/i18n';

const ICONS: Record<string, React.ComponentType<{ size?: number }>> = {
  user: User,
  'book-open': BookOpen,
  target: Target,
  lightbulb: Lightbulb,
  shield: Shield,
  layout: Layout,
};

interface PromptBlockProps {
  block: PromptBlockType;
  allBlocks?: PromptBlockType[];
  onUpdate: (changes: Partial<PromptBlockType>) => void;
  onRemove: () => void;
  onToggle: () => void;
  variables: string[];
}

export function PromptBlockComponent({ block, allBlocks, onUpdate, onRemove, onToggle, variables }: PromptBlockProps) {
  const t = useT();
  const config = BLOCK_CONFIG[block.type];
  const Icon = ICONS[config.icon];

  const isSddBlock = block.type.startsWith('sdd-');

  const [recording, setRecording] = useState(false);
  const [transcribing, setTranscribing] = useState(false);
  const [sttError, setSttError] = useState<string | null>(null);
  const [audioStream, setAudioStream] = useState<MediaStream | null>(null);
  const [sddLoading, setSddLoading] = useState<string | null>(null); // 'generate' | 'improve' | 'clarify'
  const [clarifyResult, setClarifyResult] = useState<string | null>(null);
  const recorderRef = useRef(createRecorder());

  const handleSddAction = useCallback(async (action: 'generate' | 'improve' | 'clarify') => {
    if (!allBlocks) return;
    setSddLoading(action);
    setClarifyResult(null);
    try {
      const model = MODELS[1]; // gpt-4o-mini default
      const apiKeys = getApiKeys();
      let systemPrompt: string, userPrompt: string;

      if (action === 'generate') {
        ({ systemPrompt, userPrompt } = buildGeneratePrompt(allBlocks, block.type));
      } else if (action === 'improve') {
        ({ systemPrompt, userPrompt } = buildImprovePrompt(allBlocks, block.type, block.content));
      } else {
        ({ systemPrompt, userPrompt } = buildClarifyPrompt(allBlocks, block.type, block.content));
      }

      const result = await callLLM(userPrompt, model, apiKeys, {
        systemPrompt,
        temperature: action === 'clarify' ? 0.5 : 0.3,
        maxTokens: 4096,
      });

      if (result.text) {
        if (action === 'clarify') {
          setClarifyResult(result.text);
        } else {
          onUpdate({ content: result.text });
        }
      }
    } catch (err) {
      console.error('SDD action error:', err);
    } finally {
      setSddLoading(null);
    }
  }, [allBlocks, block.type, block.content, onUpdate]);

  const handleMicClick = useCallback(async () => {
    setSttError(null);

    if (recording) {
      // Stop recording and transcribe
      setRecording(false);
      setAudioStream(null);
      setTranscribing(true);

      try {
        const audioBlob = await recorderRef.current.stop();
        if (audioBlob.size === 0) {
          setTranscribing(false);
          return;
        }

        const sttConfig = getSttConfig();
        const text = await transcribe(audioBlob, sttConfig);

        if (text) {
          const separator = block.content && !block.content.endsWith('\n') && !block.content.endsWith(' ') ? ' ' : '';
          onUpdate({ content: block.content + separator + text });
        }
      } catch (err) {
        setSttError(err instanceof Error ? err.message : 'Erreur de transcription');
        setTimeout(() => setSttError(null), 4000);
      } finally {
        setTranscribing(false);
      }
    } else {
      // Start recording
      try {
        recorderRef.current = createRecorder();
        await recorderRef.current.start();
        setRecording(true);
        setAudioStream(recorderRef.current.getStream());
      } catch (err) {
        setSttError(err instanceof Error ? err.message : 'Impossible d\'acceder au micro');
        setTimeout(() => setSttError(null), 4000);
      }
    }
  }, [recording, block.content, onUpdate]);

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: block.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`group rounded-lg border transition-all animate-fadeIn ${
        block.enabled
          ? 'border-[var(--color-border)] bg-[var(--color-bg-secondary)]'
          : 'border-[var(--color-border)] bg-[var(--color-bg-primary)] opacity-50'
      } ${recording ? 'ring-2 ring-[var(--color-danger)]/50' : ''}`}
    >
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-[var(--color-border)]">
        <button
          {...attributes}
          {...listeners}
          className="cursor-grab active:cursor-grabbing text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] touch-none"
        >
          <GripVertical size={16} />
        </button>

        <div className="flex items-center gap-2 flex-1 min-w-0">
          <div className="w-2 h-2 rounded-full flex-shrink-0" style={{ backgroundColor: config.color }} />
          {Icon && <Icon size={14} />}
          <span className="text-sm font-medium truncate" style={{ color: config.color }}>
            {t(`block.${block.type}` as TranslationKey)}
          </span>
        </div>

        <div className="flex items-center gap-1">
          {/* Audio waveform bars when recording */}
          {recording && <AudioBars stream={audioStream} barCount={5} height={18} />}

          {/* Mic button - always visible */}
          <button
            onClick={handleMicClick}
            disabled={transcribing}
            className={`p-1 rounded transition-all ${
              recording
                ? 'bg-[var(--color-danger)]/20 text-[var(--color-danger)]'
                : transcribing
                ? 'text-[var(--color-accent)]'
                : 'text-[var(--color-text-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-hover)] opacity-0 group-hover:opacity-100'
            }`}
            title={recording ? t('block.stopDictate') : transcribing ? t('block.transcribing') : t('block.dictate')}
          >
            {transcribing ? (
              <Loader2 size={14} className="animate-spin" />
            ) : recording ? (
              <MicOff size={14} />
            ) : (
              <Mic size={14} />
            )}
          </button>

          {/* SDD action buttons */}
          {isSddBlock && allBlocks && (
            <>
              <button
                onClick={() => handleSddAction('generate')}
                disabled={!!sddLoading}
                className="p-1 rounded text-[var(--color-text-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-hover)] opacity-0 group-hover:opacity-100 transition-all"
                title={t('sdd.generate')}
              >
                {sddLoading === 'generate' ? <Loader2 size={14} className="animate-spin" /> : <Wand2 size={14} />}
              </button>
              <button
                onClick={() => handleSddAction('improve')}
                disabled={!!sddLoading}
                className="p-1 rounded text-[var(--color-text-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-hover)] opacity-0 group-hover:opacity-100 transition-all"
                title={t('sdd.improve')}
              >
                {sddLoading === 'improve' ? <Loader2 size={14} className="animate-spin" /> : <Sparkles size={14} />}
              </button>
              <button
                onClick={() => handleSddAction('clarify')}
                disabled={!!sddLoading}
                className="p-1 rounded text-[var(--color-text-muted)] hover:text-yellow-400 hover:bg-[var(--color-bg-hover)] opacity-0 group-hover:opacity-100 transition-all"
                title={t('sdd.clarify')}
              >
                {sddLoading === 'clarify' ? <Loader2 size={14} className="animate-spin" /> : <HelpCircle size={14} />}
              </button>
              {block.type === 'sdd-tasks' && (
                <button
                  onClick={() => {
                    const GH_REPO_KEY = 'inkwell-github-repo';
                    const GH_PAT_KEY = 'inkwell-github-pat';
                    const savedRepo = localStorage.getItem(GH_REPO_KEY) || '';
                    const savedPat = localStorage.getItem(GH_PAT_KEY) || '';
                    const repo = prompt('GitHub repo (owner/repo):', savedRepo);
                    if (!repo) return;
                    const token = savedPat || prompt('GitHub PAT (saved for next time):');
                    if (!token) return;
                    localStorage.setItem(GH_REPO_KEY, repo);
                    localStorage.setItem(GH_PAT_KEY, token);

                    import('../lib/github').then(async ({ parseTasksMarkdown, taskToIssueBody, createGitHubIssue, updateTasksWithIssueLinks }) => {
                      const tasks = parseTasksMarkdown(block.content).filter(t => !t.done);
                      if (tasks.length === 0) { alert('No pending tasks'); return; }
                      if (!confirm(`Push ${tasks.length} tasks to ${repo}?`)) return;
                      const links = new Map<string, number>();
                      for (const task of tasks) {
                        try {
                          const body = taskToIssueBody(task, 'Inkwell SDD');
                          const issue = await createGitHubIssue(repo, token, `${task.id}: ${task.title}`, body, ['sdd']);
                          links.set(task.id, issue.number);
                        } catch (e) { console.error(`Failed to create issue for ${task.id}:`, e); }
                      }
                      if (links.size > 0) {
                        onUpdate({ content: updateTasksWithIssueLinks(block.content, links) });
                      }
                      alert(`Created ${links.size}/${tasks.length} GitHub issues`);
                    });
                  }}
                  className="p-1 rounded text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] opacity-0 group-hover:opacity-100 transition-all"
                  title="Push tasks to GitHub Issues"
                >
                  <ExternalLink size={14} />
                </button>
              )}
            </>
          )}

          {/* Toggle & Delete - visible on hover */}
          <button
            onClick={onToggle}
            className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] opacity-0 group-hover:opacity-100 transition-opacity"
            title={block.enabled ? t('block.disable') : t('block.enable')}
          >
            {block.enabled ? <Eye size={14} /> : <EyeOff size={14} />}
          </button>
          <button
            onClick={onRemove}
            className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-danger)] opacity-0 group-hover:opacity-100 transition-opacity"
            title={t('block.delete')}
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {/* Recording indicator */}
      {recording && (
        <div className="flex items-center gap-2 px-3 py-1.5 bg-[var(--color-danger)]/10 text-[var(--color-danger)] text-xs animate-fadeIn">
          <div className="w-2 h-2 rounded-full bg-[var(--color-danger)] animate-pulse" />
          {t('block.recording')}
        </div>
      )}

      {/* STT Error */}
      {sttError && (
        <div className="px-3 py-1.5 bg-[var(--color-danger)]/10 text-[var(--color-danger)] text-xs animate-fadeIn">
          {sttError}
        </div>
      )}

      {/* Editor */}
      <div className="px-2 py-1">
        <BlockEditor
          value={block.content}
          onChange={(content) => onUpdate({ content })}
          placeholder={t(`placeholder.${block.type}` as TranslationKey)}
          variables={variables}
        />

        {/* Clarify result */}
        {clarifyResult && (
          <div className="mx-3 mb-3 p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/20 text-xs text-[var(--color-text-primary)] space-y-1">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-yellow-400">{t('sdd.clarifyResult')}</span>
              <button onClick={() => setClarifyResult(null)} className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]">&times;</button>
            </div>
            <pre className="whitespace-pre-wrap text-[11px] leading-relaxed">{clarifyResult}</pre>
          </div>
        )}
      </div>
    </div>
  );
}
