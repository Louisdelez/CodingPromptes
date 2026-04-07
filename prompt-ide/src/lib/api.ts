import type { ApiKeys, ModelConfig } from './types';
import { getLocalServerUrl } from './types';

interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface ApiResponse {
  text: string;
  tokensIn: number;
  tokensOut: number;
  latencyMs: number;
}

export async function callLLM(
  prompt: string,
  model: ModelConfig,
  apiKeys: ApiKeys,
  options: { temperature?: number; maxTokens?: number; systemPrompt?: string } = {}
): Promise<ApiResponse> {
  const { temperature = 0.7, maxTokens = 2048, systemPrompt } = options;
  const start = performance.now();

  if (model.provider === 'local') {
    return callLocal(prompt, model, { temperature, maxTokens, systemPrompt, start });
  }
  if (model.provider === 'openai') {
    return callOpenAI(prompt, model, apiKeys.openai!, { temperature, maxTokens, systemPrompt, start });
  }
  if (model.provider === 'anthropic') {
    return callAnthropic(prompt, model, apiKeys.anthropic!, { temperature, maxTokens, systemPrompt, start });
  }
  if (model.provider === 'google') {
    return callGoogle(prompt, model, apiKeys.google!, { temperature, maxTokens, start });
  }
  throw new Error(`Provider non supporté: ${model.provider}`);
}

async function callOpenAI(
  prompt: string,
  model: ModelConfig,
  apiKey: string,
  opts: { temperature: number; maxTokens: number; systemPrompt?: string; start: number }
): Promise<ApiResponse> {
  const messages: ChatMessage[] = [];
  if (opts.systemPrompt) messages.push({ role: 'system', content: opts.systemPrompt });
  messages.push({ role: 'user', content: prompt });

  const res = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${apiKey}` },
    body: JSON.stringify({
      model: model.id,
      messages,
      temperature: opts.temperature,
      max_tokens: opts.maxTokens,
    }),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || `OpenAI error ${res.status}`);
  }

  const data = await res.json();
  return {
    text: data.choices[0].message.content,
    tokensIn: data.usage?.prompt_tokens ?? 0,
    tokensOut: data.usage?.completion_tokens ?? 0,
    latencyMs: Math.round(performance.now() - opts.start),
  };
}

async function callAnthropic(
  prompt: string,
  model: ModelConfig,
  apiKey: string,
  opts: { temperature: number; maxTokens: number; systemPrompt?: string; start: number }
): Promise<ApiResponse> {
  const res = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': apiKey,
      'anthropic-version': '2023-06-01',
      'anthropic-dangerous-direct-browser-access': 'true',
    },
    body: JSON.stringify({
      model: model.id,
      max_tokens: opts.maxTokens,
      temperature: opts.temperature,
      ...(opts.systemPrompt ? { system: opts.systemPrompt } : {}),
      messages: [{ role: 'user', content: prompt }],
    }),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || `Anthropic error ${res.status}`);
  }

  const data = await res.json();
  return {
    text: data.content.map((c: { text: string }) => c.text).join(''),
    tokensIn: data.usage?.input_tokens ?? 0,
    tokensOut: data.usage?.output_tokens ?? 0,
    latencyMs: Math.round(performance.now() - opts.start),
  };
}

async function callGoogle(
  prompt: string,
  model: ModelConfig,
  apiKey: string,
  opts: { temperature: number; maxTokens: number; start: number }
): Promise<ApiResponse> {
  const res = await fetch(
    `https://generativelanguage.googleapis.com/v1beta/models/${model.id}:generateContent?key=${apiKey}`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        contents: [{ parts: [{ text: prompt }] }],
        generationConfig: {
          temperature: opts.temperature,
          maxOutputTokens: opts.maxTokens,
        },
      }),
    }
  );

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || `Google error ${res.status}`);
  }

  const data = await res.json();
  const text = data.candidates?.[0]?.content?.parts?.map((p: { text: string }) => p.text).join('') ?? '';
  return {
    text,
    tokensIn: data.usageMetadata?.promptTokenCount ?? 0,
    tokensOut: data.usageMetadata?.candidatesTokenCount ?? 0,
    latencyMs: Math.round(performance.now() - opts.start),
  };
}

async function callLocal(
  prompt: string,
  model: ModelConfig,
  opts: { temperature: number; maxTokens: number; systemPrompt?: string; start: number }
): Promise<ApiResponse> {
  const baseUrl = (model.nodeAddress || getLocalServerUrl()).replace(/\/$/, '');
  // Strip node prefix from model ID (e.g. "node-uuid::qwen3.5:9b" -> "qwen3.5:9b")
  const modelId = model.id.includes('::') ? model.id.split('::').slice(1).join('::') : model.id;
  const messages: ChatMessage[] = [];
  if (opts.systemPrompt) messages.push({ role: 'system', content: opts.systemPrompt });
  messages.push({ role: 'user', content: prompt });

  const res = await fetch(`${baseUrl}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: modelId,
      messages,
      temperature: opts.temperature,
      max_tokens: opts.maxTokens,
      stream: false,
    }),
  });

  if (!res.ok) {
    const err = await res.text();
    throw new Error(`Serveur local: ${err}`);
  }

  const data = await res.json();
  return {
    text: data.choices?.[0]?.message?.content ?? '',
    tokensIn: data.usage?.prompt_tokens ?? 0,
    tokensOut: data.usage?.completion_tokens ?? 0,
    latencyMs: Math.round(performance.now() - opts.start),
  };
}

export async function callLLMStream(
  prompt: string,
  model: ModelConfig,
  apiKeys: ApiKeys,
  options: { temperature?: number; maxTokens?: number; systemPrompt?: string },
  onChunk: (text: string) => void,
): Promise<ApiResponse> {
  // For local and openai: real SSE streaming
  if (model.provider === 'local' || model.provider === 'openai') {
    return callOpenAICompatibleStream(prompt, model, apiKeys, options, onChunk);
  }
  // For anthropic and google: fallback to non-streaming, call onChunk once
  const result = await callLLM(prompt, model, apiKeys, options);
  onChunk(result.text);
  return result;
}

async function callOpenAICompatibleStream(
  prompt: string,
  model: ModelConfig,
  apiKeys: ApiKeys,
  options: { temperature?: number; maxTokens?: number; systemPrompt?: string },
  onChunk: (text: string) => void,
): Promise<ApiResponse> {
  const { temperature = 0.7, maxTokens = 2048, systemPrompt } = options;
  const start = performance.now();

  const messages: ChatMessage[] = [];
  if (systemPrompt) messages.push({ role: 'system', content: systemPrompt });
  messages.push({ role: 'user', content: prompt });

  const isLocal = model.provider === 'local';
  const baseUrl = isLocal
    ? (model.nodeAddress || getLocalServerUrl()).replace(/\/$/, '') + '/v1/chat/completions'
    : 'https://api.openai.com/v1/chat/completions';

  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (!isLocal) {
    headers['Authorization'] = `Bearer ${apiKeys.openai!}`;
  }

  const modelId = model.id.includes('::') ? model.id.split('::').slice(1).join('::') : model.id;

  const res = await fetch(baseUrl, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      model: modelId,
      messages,
      temperature,
      max_tokens: maxTokens,
      stream: true,
    }),
  });

  if (!res.ok) {
    const err = await res.text();
    throw new Error(isLocal ? `Serveur local: ${err}` : `OpenAI error ${res.status}: ${err}`);
  }

  const reader = res.body!.getReader();
  const decoder = new TextDecoder();
  let fullText = '';
  let buffer = '';
  let tokensIn = 0;
  let tokensOut = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() ?? '';

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || !trimmed.startsWith('data: ')) continue;
      const data = trimmed.slice(6);
      if (data === '[DONE]') continue;

      try {
        const parsed = JSON.parse(data);
        const delta = parsed.choices?.[0]?.delta?.content;
        if (delta) {
          fullText += delta;
          onChunk(delta);
        }
        // Capture usage if present (some providers send it in the last chunk)
        if (parsed.usage) {
          tokensIn = parsed.usage.prompt_tokens ?? 0;
          tokensOut = parsed.usage.completion_tokens ?? 0;
        }
      } catch {
        // skip malformed SSE lines
      }
    }
  }

  // Estimate tokens if not provided by the API
  if (tokensOut === 0 && fullText.length > 0) {
    tokensOut = Math.ceil(fullText.length / 4);
  }
  if (tokensIn === 0 && prompt.length > 0) {
    tokensIn = Math.ceil(prompt.length / 4);
  }

  return {
    text: fullText,
    tokensIn,
    tokensOut,
    latencyMs: Math.round(performance.now() - start),
  };
}

export async function callLLMStreamMessages(
  messages: ChatMessage[],
  model: ModelConfig,
  apiKeys: ApiKeys,
  options: { temperature?: number; maxTokens?: number },
  onChunk: (text: string) => void,
): Promise<ApiResponse> {
  const { temperature = 0.7, maxTokens = 2048 } = options;
  const start = performance.now();

  // For anthropic: separate system from messages, use non-streaming fallback
  if (model.provider === 'anthropic') {
    const systemMsg = messages.find(m => m.role === 'system');
    const nonSystemMsgs = messages.filter(m => m.role !== 'system');
    const apiKey = apiKeys.anthropic!;
    const res = await fetch('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': apiKey,
        'anthropic-version': '2023-06-01',
        'anthropic-dangerous-direct-browser-access': 'true',
      },
      body: JSON.stringify({
        model: model.id,
        max_tokens: maxTokens,
        temperature,
        ...(systemMsg ? { system: systemMsg.content } : {}),
        messages: nonSystemMsgs.map(m => ({ role: m.role, content: m.content })),
      }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error?.message || `Anthropic error ${res.status}`);
    }
    const data = await res.json();
    const text = data.content.map((c: { text: string }) => c.text).join('');
    onChunk(text);
    return {
      text,
      tokensIn: data.usage?.input_tokens ?? 0,
      tokensOut: data.usage?.output_tokens ?? 0,
      latencyMs: Math.round(performance.now() - start),
    };
  }

  // For google: fallback to non-streaming single turn
  if (model.provider === 'google') {
    const lastUser = messages.filter(m => m.role === 'user').pop();
    const prompt = lastUser?.content ?? '';
    const systemMsg = messages.find(m => m.role === 'system');
    const result = await callLLM(prompt, model, apiKeys, { temperature, maxTokens, systemPrompt: systemMsg?.content });
    onChunk(result.text);
    return result;
  }

  // For openai and local: SSE streaming with full messages
  const isLocal = model.provider === 'local';
  const baseUrl = isLocal
    ? (model.nodeAddress || getLocalServerUrl()).replace(/\/$/, '') + '/v1/chat/completions'
    : 'https://api.openai.com/v1/chat/completions';

  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (!isLocal) {
    headers['Authorization'] = `Bearer ${apiKeys.openai!}`;
  }

  const modelId = model.id.includes('::') ? model.id.split('::').slice(1).join('::') : model.id;

  const res = await fetch(baseUrl, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      model: modelId,
      messages,
      temperature,
      max_tokens: maxTokens,
      stream: true,
    }),
  });

  if (!res.ok) {
    const err = await res.text();
    throw new Error(isLocal ? `Serveur local: ${err}` : `OpenAI error ${res.status}: ${err}`);
  }

  const reader = res.body!.getReader();
  const decoder = new TextDecoder();
  let fullText = '';
  let buffer = '';
  let tokensIn = 0;
  let tokensOut = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() ?? '';

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || !trimmed.startsWith('data: ')) continue;
      const data = trimmed.slice(6);
      if (data === '[DONE]') continue;

      try {
        const parsed = JSON.parse(data);
        const delta = parsed.choices?.[0]?.delta?.content;
        if (delta) {
          fullText += delta;
          onChunk(delta);
        }
        if (parsed.usage) {
          tokensIn = parsed.usage.prompt_tokens ?? 0;
          tokensOut = parsed.usage.completion_tokens ?? 0;
        }
      } catch {
        // skip malformed SSE lines
      }
    }
  }

  if (tokensOut === 0 && fullText.length > 0) tokensOut = Math.ceil(fullText.length / 4);
  if (tokensIn === 0) tokensIn = Math.ceil(messages.map(m => m.content).join('').length / 4);

  return {
    text: fullText,
    tokensIn,
    tokensOut,
    latencyMs: Math.round(performance.now() - start),
  };
}

// Fetch available models from local server (Ollama proxy)
export async function fetchLocalModels(serverUrl?: string): Promise<{ id: string; name: string }[]> {
  const baseUrl = (serverUrl || getLocalServerUrl()).replace(/\/$/, '');
  try {
    const res = await fetch(`${baseUrl}/v1/models`, { signal: AbortSignal.timeout(3000) });
    if (!res.ok) return [];
    const data = await res.json();
    return (data.data ?? []).map((m: { id: string }) => ({
      id: m.id,
      name: m.id,
    }));
  } catch {
    return [];
  }
}

export async function optimizePrompt(
  prompt: string,
  model: ModelConfig,
  apiKeys: ApiKeys
): Promise<string> {
  const metaPrompt = `Tu es un expert en prompt engineering. Analyse le prompt suivant et retourne une version optimisée.

Règles:
- Garde la même intention et le même objectif
- Améliore la clarté, la structure et la précision
- Ajoute des contraintes utiles si manquantes
- Utilise des balises XML si approprié
- Retourne UNIQUEMENT le prompt optimisé, sans explication

Prompt à optimiser:
---
${prompt}
---

Prompt optimisé:`;

  const result = await callLLM(metaPrompt, model, apiKeys, { temperature: 0.3, maxTokens: 4096 });
  return result.text.trim();
}
