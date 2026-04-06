import { useState, useRef, useEffect, useCallback, useMemo } from 'react';
import { Send, Trash2, Loader2, FileText } from 'lucide-react';
import type { PromptBlock, ModelConfig } from '../lib/types';
import { MODELS } from '../lib/types';
import { compilePrompt } from '../lib/prompt';
import { callLLMStreamMessages, fetchLocalModels } from '../lib/api';
import { getApiKeys } from '../lib/db';
import { useT } from '../lib/i18n';
import { renderMarkdown } from '../lib/markdown';

interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

interface ConversationModeProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
}

export function ConversationMode({ blocks, variables }: ConversationModeProps) {
  const t = useT();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [systemPrompt, setSystemPrompt] = useState('');
  const [selectedModelId, setSelectedModelId] = useState<string>('gpt-4o-mini');
  const [temperature, setTemperature] = useState(0.7);
  const [sending, setSending] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const [localModels, setLocalModels] = useState<{ id: string; name: string }[]>([]);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Fetch local models
  useEffect(() => {
    const check = async () => {
      const models = await fetchLocalModels();
      setLocalModels(models);
    };
    check();
    const interval = setInterval(check, 10000);
    return () => clearInterval(interval);
  }, []);

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

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingText]);

  const useCurrentPrompt = () => {
    const compiled = compilePrompt(blocks, variables);
    setSystemPrompt(compiled);
  };

  const clearConversation = () => {
    setMessages([]);
    setStreamingText('');
  };

  const sendMessage = useCallback(async () => {
    const trimmed = input.trim();
    if (!trimmed || sending) return;

    const apiKeys = getApiKeys();
    const model = allModels.find((m) => m.id === selectedModelId) ?? MODELS[0];

    const userMsg: ChatMessage = { role: 'user', content: trimmed };
    const updatedMessages = [...messages, userMsg];
    setMessages(updatedMessages);
    setInput('');
    setSending(true);
    setStreamingText('');

    // Build the full message list for the API
    const apiMessages: ChatMessage[] = [];
    if (systemPrompt.trim()) {
      apiMessages.push({ role: 'system', content: systemPrompt.trim() });
    }
    apiMessages.push(...updatedMessages);

    try {
      let accumulated = '';
      const response = await callLLMStreamMessages(
        apiMessages,
        model,
        apiKeys,
        { temperature, maxTokens: 4096 },
        (chunk) => {
          accumulated += chunk;
          setStreamingText(accumulated);
        },
      );

      setMessages((prev) => [
        ...prev,
        { role: 'assistant', content: response.text },
      ]);
      setStreamingText('');
    } catch (err) {
      setMessages((prev) => [
        ...prev,
        { role: 'assistant', content: `Error: ${err instanceof Error ? err.message : 'Unknown error'}` },
      ]);
      setStreamingText('');
    } finally {
      setSending(false);
      inputRef.current?.focus();
    }
  }, [input, sending, messages, systemPrompt, selectedModelId, temperature, allModels]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header controls */}
      <div className="p-3 space-y-3 border-b border-[var(--color-border)] flex-shrink-0">
        <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">
          {t('chat.title')}
        </h3>

        {/* Model selector */}
        <select
          value={selectedModelId}
          onChange={(e) => setSelectedModelId(e.target.value)}
          className="w-full px-3 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
        >
          {localModels.length > 0 && (
            <optgroup label="Local (Ollama)">
              {localModels.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </optgroup>
          )}
          <optgroup label="Cloud">
            {MODELS.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name} ({m.provider})
              </option>
            ))}
          </optgroup>
        </select>

        {/* Temperature */}
        <div className="flex items-center gap-2">
          <span className="text-xs text-[var(--color-text-muted)] flex-shrink-0">Temp:</span>
          <input
            type="range"
            min="0"
            max="2"
            step="0.1"
            value={temperature}
            onChange={(e) => setTemperature(parseFloat(e.target.value))}
            className="flex-1 h-1 accent-[var(--color-accent)]"
          />
          <span className="text-xs text-[var(--color-text-secondary)] w-8 text-right">
            {temperature.toFixed(1)}
          </span>
        </div>

        {/* System prompt */}
        <div className="space-y-1">
          <div className="flex items-center justify-between">
            <label className="text-xs text-[var(--color-text-muted)]">
              {t('chat.systemPrompt')}
            </label>
            <button
              onClick={useCurrentPrompt}
              className="flex items-center gap-1 text-xs text-[var(--color-accent)] hover:opacity-80 transition-opacity"
            >
              <FileText size={10} />
              {t('chat.useCurrentPrompt')}
            </button>
          </div>
          <textarea
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            placeholder={t('chat.systemPrompt')}
            rows={2}
            className="w-full px-2 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)] resize-none"
          />
        </div>

        {/* Clear button */}
        {messages.length > 0 && (
          <button
            onClick={clearConversation}
            className="flex items-center gap-1 text-xs text-red-400 hover:text-red-300 transition-colors"
          >
            <Trash2 size={11} />
            {t('chat.clear')}
          </button>
        )}
      </div>

      {/* Message history */}
      <div className="flex-1 overflow-auto p-3 space-y-3">
        {messages.map((msg, i) => (
          <div
            key={i}
            className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[85%] rounded-lg px-3 py-2 text-xs leading-relaxed ${
                msg.role === 'user'
                  ? 'bg-[var(--color-accent)] text-white'
                  : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]'
              }`}
            >
              {msg.role === 'assistant' ? (
                <div
                  className="prose-sm"
                  dangerouslySetInnerHTML={{ __html: renderMarkdown(msg.content) }}
                />
              ) : (
                <pre className="whitespace-pre-wrap break-words font-inherit m-0">
                  {msg.content}
                </pre>
              )}
            </div>
          </div>
        ))}

        {/* Streaming indicator */}
        {sending && streamingText && (
          <div className="flex justify-start">
            <div className="max-w-[85%] rounded-lg px-3 py-2 text-xs leading-relaxed bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
              <div
                className="prose-sm"
                dangerouslySetInnerHTML={{ __html: renderMarkdown(streamingText) }}
              />
            </div>
          </div>
        )}

        {sending && !streamingText && (
          <div className="flex justify-start">
            <div className="rounded-lg px-3 py-2 bg-[var(--color-bg-tertiary)]">
              <Loader2 size={14} className="animate-spin text-[var(--color-text-muted)]" />
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div className="p-3 border-t border-[var(--color-border)] flex-shrink-0">
        <div className="flex gap-2">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('chat.placeholder')}
            rows={2}
            className="flex-1 px-3 py-2 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)] resize-none"
          />
          <button
            onClick={sendMessage}
            disabled={sending || !input.trim()}
            className="self-end px-3 py-2 rounded-lg bg-[var(--color-accent)] text-white hover:opacity-90 disabled:opacity-40 transition-opacity"
          >
            {sending ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Send size={14} />
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
