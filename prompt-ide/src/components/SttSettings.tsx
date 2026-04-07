import { useState, useEffect } from 'react';
import { Mic, Wifi, WifiOff, Globe, Cpu } from 'lucide-react';
import { getSttConfig, setSttConfig, type SttConfig, type SttProvider } from '../lib/stt';
import { useT, type TranslationKey } from '../lib/i18n';
import { listNodes, parseCapabilities, type GpuNode } from '../lib/backend';

const PROVIDER_ICONS: Record<SttProvider, React.ComponentType<{ size?: number }>> = {
  local: Cpu,
  openai: Globe,
  groq: Globe,
  deepgram: Globe,
};

const LANGUAGE_CODES = ['auto', 'fr', 'en', 'es', 'de', 'it', 'pt', 'nl', 'ja', 'zh', 'ko', 'ru', 'ar'] as const;

export function SttSettings() {
  const t = useT();

  const PROVIDERS: { id: SttProvider; label: string; icon: React.ComponentType<{ size?: number }>; desc: string }[] = [
    { id: 'local', label: t('stt.local'), icon: PROVIDER_ICONS.local, desc: t('stt.localDesc') },
    { id: 'openai', label: 'OpenAI Whisper', icon: PROVIDER_ICONS.openai, desc: t('stt.openaiDesc') },
    { id: 'groq', label: 'Groq Whisper', icon: PROVIDER_ICONS.groq, desc: t('stt.groqDesc') },
    { id: 'deepgram', label: 'Deepgram Nova-3', icon: PROVIDER_ICONS.deepgram, desc: t('stt.deepgramDesc') },
  ];

  const LANGUAGES = LANGUAGE_CODES.map((code) => ({
    code,
    label: t(`lang.${code}` as TranslationKey),
  }));
  const [config, setConfig] = useState<SttConfig>(getSttConfig);
  const [serverStatus, setServerStatus] = useState<'checking' | 'online' | 'offline'>('checking');
  const [sttNodes, setSttNodes] = useState<GpuNode[]>([]);

  // Fetch GPU nodes with STT capability
  useEffect(() => {
    const fetchNodes = async () => {
      try {
        const nodes = await listNodes();
        const withStt = nodes.filter(n => {
          if (n.status !== 'online') return false;
          const caps = parseCapabilities(n);
          return caps.stt.model_loaded;
        });
        setSttNodes(withStt);
      } catch { /* fleet not available */ }
    };
    fetchNodes();
    const iv = setInterval(fetchNodes, 15000);
    return () => clearInterval(iv);
  }, []);

  const [apiKeys, setApiKeysLocal] = useState<Record<string, string>>(() => {
    try {
      const raw = localStorage.getItem('prompt-ide-stt-extra-keys');
      return raw ? JSON.parse(raw) : {};
    } catch { return {}; }
  });

  // Check local server health
  useEffect(() => {
    if (config.provider !== 'local') return;
    const check = async () => {
      try {
        const res = await fetch(`${config.localServerUrl}/health`, { signal: AbortSignal.timeout(3000) });
        setServerStatus(res.ok ? 'online' : 'offline');
      } catch {
        setServerStatus('offline');
      }
    };
    check();
    const interval = setInterval(check, 5000);
    return () => clearInterval(interval);
  }, [config.localServerUrl, config.provider]);

  const updateConfig = (changes: Partial<SttConfig>) => {
    const updated = { ...config, ...changes };
    setConfig(updated);
    setSttConfig(updated);
  };

  const updateExtraKey = (key: string, value: string) => {
    const updated = { ...apiKeys, [key]: value };
    setApiKeysLocal(updated);
    localStorage.setItem('prompt-ide-stt-extra-keys', JSON.stringify(updated));
    // Also merge into main api keys for STT use
    try {
      const main = JSON.parse(localStorage.getItem('prompt-ide-api-keys') || '{}');
      main[key] = value;
      localStorage.setItem('prompt-ide-api-keys', JSON.stringify(main));
    } catch { /* ignore */ }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        <Mic size={14} />
        <span>{t('stt.title')}</span>
      </div>

      {/* Provider selection */}
      <div className="space-y-1.5">
        <label className="text-xs text-[var(--color-text-muted)]">{t('stt.provider')}</label>
        {PROVIDERS.map((p) => (
          <button
            key={p.id}
            onClick={() => updateConfig({ provider: p.id })}
            className={`w-full flex items-center gap-3 p-2.5 rounded-lg border transition-all text-left ${
              config.provider === p.id
                ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10'
                : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)]'
            }`}
          >
            <p.icon size={14} />
            <div className="flex-1 min-w-0">
              <div className="text-sm text-[var(--color-text-primary)]">{p.label}</div>
              <div className="text-[10px] text-[var(--color-text-muted)]">{p.desc}</div>
            </div>
            {config.provider === p.id && p.id === 'local' && (
              <div className="flex items-center gap-1">
                {serverStatus === 'online' ? (
                  <Wifi size={12} className="text-[var(--color-success)]" />
                ) : serverStatus === 'offline' ? (
                  <WifiOff size={12} className="text-[var(--color-danger)]" />
                ) : (
                  <div className="w-3 h-3 rounded-full border-2 border-[var(--color-text-muted)] border-t-transparent animate-spin" />
                )}
              </div>
            )}
          </button>
        ))}
      </div>

      {/* Local STT: GPU node selector */}
      {config.provider === 'local' && (
        <div className="space-y-2 animate-fadeIn">
          <label className="text-xs text-[var(--color-text-muted)]">{t('stt.selectNode')}</label>
          {sttNodes.length > 0 ? (
            sttNodes.map(node => {
              const caps = parseCapabilities(node);
              const isSelected = config.nodeAddress === node.address;
              return (
                <button
                  key={node.id}
                  onClick={() => updateConfig({ nodeAddress: node.address, nodeName: node.name, localServerUrl: node.address })}
                  className={`w-full flex items-center gap-3 p-2.5 rounded-lg border transition-all text-left ${
                    isSelected
                      ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10'
                      : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)]'
                  }`}
                >
                  <Cpu size={14} className="text-[var(--color-accent)]" />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-[var(--color-text-primary)]">{node.name}</div>
                    <div className="text-[10px] text-[var(--color-text-muted)]">
                      Whisper {caps.stt.active_model || '?'} — {node.gpu_info || node.address}
                    </div>
                  </div>
                  {isSelected && <Wifi size={12} className="text-green-400" />}
                </button>
              );
            })
          ) : (
            <div className="text-xs text-[var(--color-text-muted)] p-3 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
              {t('stt.noSttNodes')}
            </div>
          )}
        </div>
      )}

      {/* Deepgram API key */}
      {config.provider === 'deepgram' && (
        <div className="space-y-1.5 animate-fadeIn">
          <label className="text-xs text-[var(--color-text-muted)]">Cle API Deepgram</label>
          <input
            type="password"
            value={apiKeys['deepgram'] ?? ''}
            onChange={(e) => updateExtraKey('deepgram', e.target.value)}
            placeholder="Token..."
            className="w-full px-2.5 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)] font-mono"
          />
        </div>
      )}

      {/* Groq API key */}
      {config.provider === 'groq' && (
        <div className="space-y-1.5 animate-fadeIn">
          <label className="text-xs text-[var(--color-text-muted)]">Cle API Groq</label>
          <input
            type="password"
            value={apiKeys['groq'] ?? ''}
            onChange={(e) => updateExtraKey('groq', e.target.value)}
            placeholder="gsk_..."
            className="w-full px-2.5 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)] font-mono"
          />
        </div>
      )}

      {config.provider === 'openai' && (
        <div className="text-xs text-[var(--color-text-muted)]">
          {t('stt.useOpenaiKey')}
        </div>
      )}

      {/* Language */}
      <div className="space-y-1.5">
        <label className="text-xs text-[var(--color-text-muted)]">{t('stt.language')}</label>
        <select
          value={config.language}
          onChange={(e) => updateConfig({ language: e.target.value })}
          className="w-full px-2.5 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
        >
          {LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>{l.label}</option>
          ))}
        </select>
      </div>

      {/* Usage hint */}
      <div className="p-2.5 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
        <div className="text-xs text-[var(--color-text-muted)]">
          {t('stt.hint')}
        </div>
      </div>
    </div>
  );
}
