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
} from 'lucide-react';
import type { PromptBlock as PromptBlockType } from '../lib/types';
import { BLOCK_CONFIG } from '../lib/types';
import { BlockEditor } from './BlockEditor';
import { createRecorder, transcribe, getSttConfig } from '../lib/stt';
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
  onUpdate: (changes: Partial<PromptBlockType>) => void;
  onRemove: () => void;
  onToggle: () => void;
  variables: string[];
}

export function PromptBlockComponent({ block, onUpdate, onRemove, onToggle, variables }: PromptBlockProps) {
  const t = useT();
  const config = BLOCK_CONFIG[block.type];
  const Icon = ICONS[config.icon];

  const [recording, setRecording] = useState(false);
  const [transcribing, setTranscribing] = useState(false);
  const [sttError, setSttError] = useState<string | null>(null);
  const recorderRef = useRef(createRecorder());

  const handleMicClick = useCallback(async () => {
    setSttError(null);

    if (recording) {
      // Stop recording and transcribe
      setRecording(false);
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
          {/* Mic button - always visible */}
          <button
            onClick={handleMicClick}
            disabled={transcribing}
            className={`p-1 rounded transition-all ${
              recording
                ? 'bg-[var(--color-danger)]/20 text-[var(--color-danger)] animate-pulse'
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
      </div>
    </div>
  );
}
