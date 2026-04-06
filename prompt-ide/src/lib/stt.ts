import { getApiKeys } from './db';

export type SttProvider = 'local' | 'openai' | 'deepgram' | 'groq';

export interface SttConfig {
  provider: SttProvider;
  localServerUrl: string; // e.g. http://192.168.1.50:8910
  language: string;       // "auto", "fr", "en", etc.
}

const STT_CONFIG_KEY = 'prompt-ide-stt-config';

export function getSttConfig(): SttConfig {
  try {
    const raw = localStorage.getItem(STT_CONFIG_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return { provider: 'local', localServerUrl: 'http://localhost:8910', language: 'auto' };
}

export function setSttConfig(config: SttConfig): void {
  localStorage.setItem(STT_CONFIG_KEY, JSON.stringify(config));
}

// Record audio from microphone as WAV
export async function recordAudio(): Promise<{ blob: Blob; stop: () => void }> {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: { sampleRate: 16000, channelCount: 1 } });
  const mediaRecorder = new MediaRecorder(stream, { mimeType: 'audio/webm;codecs=opus' });
  const chunks: Blob[] = [];

  return new Promise((resolve) => {
    mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunks.push(e.data);
    };

    mediaRecorder.onstop = () => {
      stream.getTracks().forEach((t) => t.stop());
      const blob = new Blob(chunks, { type: 'audio/webm' });
      resolve({ blob, stop: () => {} });
    };

    mediaRecorder.start(100); // collect in 100ms chunks

    const stop = () => {
      if (mediaRecorder.state !== 'inactive') {
        mediaRecorder.stop();
      }
    };

    resolve({ blob: new Blob(), stop });
  });
}

// Simpler approach: return start/stop controls
export function createRecorder(): {
  start: () => Promise<void>;
  stop: () => Promise<Blob>;
  isRecording: () => boolean;
} {
  let mediaRecorder: MediaRecorder | null = null;
  let stream: MediaStream | null = null;
  let chunks: Blob[] = [];
  let resolveStop: ((blob: Blob) => void) | null = null;

  return {
    start: async () => {
      chunks = [];
      stream = await navigator.mediaDevices.getUserMedia({
        audio: { sampleRate: 16000, channelCount: 1, echoCancellation: true, noiseSuppression: true },
      });
      mediaRecorder = new MediaRecorder(stream);
      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunks.push(e.data);
      };
      mediaRecorder.onstop = () => {
        stream?.getTracks().forEach((t) => t.stop());
        const blob = new Blob(chunks, { type: mediaRecorder?.mimeType || 'audio/webm' });
        resolveStop?.(blob);
      };
      mediaRecorder.start(100);
    },

    stop: () => {
      return new Promise<Blob>((resolve) => {
        resolveStop = resolve;
        if (mediaRecorder && mediaRecorder.state !== 'inactive') {
          mediaRecorder.stop();
        } else {
          resolve(new Blob());
        }
      });
    },

    isRecording: () => mediaRecorder?.state === 'recording',
  };
}

// Convert audio blob to base64 WAV via AudioContext
async function blobToWavBase64(blob: Blob): Promise<string> {
  const arrayBuffer = await blob.arrayBuffer();
  const audioCtx = new AudioContext({ sampleRate: 16000 });
  const decoded = await audioCtx.decodeAudioData(arrayBuffer);
  await audioCtx.close();

  // Get mono PCM data
  const pcm = decoded.getChannelData(0);

  // Encode as WAV
  const wavBuffer = encodeWav(pcm, 16000);
  const base64 = btoa(String.fromCharCode(...new Uint8Array(wavBuffer)));
  return base64;
}

function encodeWav(samples: Float32Array, sampleRate: number): ArrayBuffer {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);

  // WAV header
  writeString(view, 0, 'RIFF');
  view.setUint32(4, 36 + samples.length * 2, true);
  writeString(view, 8, 'WAVE');
  writeString(view, 12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeString(view, 36, 'data');
  view.setUint32(40, samples.length * 2, true);

  // PCM data
  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(44 + i * 2, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
  }

  return buffer;
}

function writeString(view: DataView, offset: number, str: string) {
  for (let i = 0; i < str.length; i++) {
    view.setUint8(offset + i, str.charCodeAt(i));
  }
}

// Transcribe via selected provider
export async function transcribe(audioBlob: Blob, config: SttConfig): Promise<string> {
  switch (config.provider) {
    case 'local':
      return transcribeLocal(audioBlob, config);
    case 'openai':
      return transcribeOpenAI(audioBlob, config);
    case 'deepgram':
      return transcribeDeepgram(audioBlob, config);
    case 'groq':
      return transcribeGroq(audioBlob, config);
    default:
      throw new Error(`Provider inconnu: ${config.provider}`);
  }
}

async function transcribeLocal(blob: Blob, config: SttConfig): Promise<string> {
  const audio = await blobToWavBase64(blob);
  const url = `${config.localServerUrl.replace(/\/$/, '')}/transcribe`;
  const lang = config.language === 'auto' ? undefined : config.language;

  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ audio, language: lang }),
  });

  if (!res.ok) {
    const err = await res.text();
    throw new Error(`Serveur local: ${err}`);
  }

  const data = await res.json();
  return data.text;
}

async function transcribeOpenAI(blob: Blob, config: SttConfig): Promise<string> {
  const keys = getApiKeys();
  if (!keys.openai) throw new Error('Cle API OpenAI manquante');

  const formData = new FormData();
  formData.append('file', blob, 'audio.webm');
  formData.append('model', 'gpt-4o-mini-transcribe');
  if (config.language !== 'auto') formData.append('language', config.language);

  const res = await fetch('https://api.openai.com/v1/audio/transcriptions', {
    method: 'POST',
    headers: { Authorization: `Bearer ${keys.openai}` },
    body: formData,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || `OpenAI STT error ${res.status}`);
  }

  const data = await res.json();
  return data.text;
}

async function transcribeDeepgram(blob: Blob, config: SttConfig): Promise<string> {
  const keys = getApiKeys();
  const key = (keys as Record<string, string | undefined>)['deepgram'];
  if (!key) throw new Error('Cle API Deepgram manquante. Ajoutez-la dans les parametres STT.');

  const langParam = config.language !== 'auto' ? `&language=${config.language}` : '&detect_language=true';
  const res = await fetch(`https://api.deepgram.com/v1/listen?model=nova-3&punctuate=true${langParam}`, {
    method: 'POST',
    headers: {
      Authorization: `Token ${key}`,
      'Content-Type': blob.type || 'audio/webm',
    },
    body: blob,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.err_msg || `Deepgram error ${res.status}`);
  }

  const data = await res.json();
  return data.results?.channels?.[0]?.alternatives?.[0]?.transcript ?? '';
}

async function transcribeGroq(blob: Blob, config: SttConfig): Promise<string> {
  const keys = getApiKeys();
  const key = (keys as Record<string, string | undefined>)['groq'];
  if (!key) throw new Error('Cle API Groq manquante. Ajoutez-la dans les parametres STT.');

  const formData = new FormData();
  formData.append('file', blob, 'audio.webm');
  formData.append('model', 'whisper-large-v3-turbo');
  if (config.language !== 'auto') formData.append('language', config.language);

  const res = await fetch('https://api.groq.com/openai/v1/audio/transcriptions', {
    method: 'POST',
    headers: { Authorization: `Bearer ${key}` },
    body: formData,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || `Groq error ${res.status}`);
  }

  const data = await res.json();
  return data.text;
}
