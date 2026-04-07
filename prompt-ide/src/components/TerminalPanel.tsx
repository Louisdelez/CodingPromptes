import { useState, useEffect, useRef, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
// import { WebglAddon } from '@xterm/addon-webgl';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import '@xterm/xterm/css/xterm.css';
import { Plus, X, MonitorSmartphone, Globe, Search } from 'lucide-react';
import { useT } from '../lib/i18n';

// Detect if running inside Tauri
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const isTauri = !!(window as any).__TAURI_INTERNALS__;

// Dynamic imports for Tauri API (only available in desktop builds)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function tauriInvoke(cmd: string, args?: Record<string, unknown>): Promise<any> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke(cmd, args);
}
// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function tauriListen(event: string, handler: (e: any) => void): Promise<() => void> {
  const { listen } = await import('@tauri-apps/api/event');
  return listen(event, handler);
}

interface TerminalSession {
  id: string;
  label: string;
  type: 'local' | 'ssh';
  terminal: Terminal;
  fitAddon: FitAddon;
  searchAddon: SearchAddon;
  ws?: WebSocket;
  connected: boolean;
  needsConnect: boolean;
  sshConfig?: SSHConfig;
}

interface SSHConfig {
  host: string;
  port: string;
  username: string;
  authMethod: 'password' | 'key';
  password: string;
  keyPath: string;
}

// Terminal theme matching Inkwell dark/light
function getTermTheme(): Record<string, string> {
  const isDark = document.documentElement.getAttribute('data-theme') !== 'light';
  if (isDark) {
    return {
      background: '#0f1117',
      foreground: '#e4e4e7',
      cursor: '#6366f1',
      cursorAccent: '#0f1117',
      selectionBackground: '#6366f133',
      black: '#1a1b23',
      red: '#f87171',
      green: '#34d399',
      yellow: '#fbbf24',
      blue: '#60a5fa',
      magenta: '#a78bfa',
      cyan: '#22d3ee',
      white: '#e4e4e7',
      brightBlack: '#52525b',
      brightRed: '#fca5a5',
      brightGreen: '#6ee7b7',
      brightYellow: '#fde68a',
      brightBlue: '#93c5fd',
      brightMagenta: '#c4b5fd',
      brightCyan: '#67e8f9',
      brightWhite: '#fafafa',
    };
  }
  return {
    background: '#ffffff',
    foreground: '#1f2937',
    cursor: '#4f46e5',
    cursorAccent: '#ffffff',
    selectionBackground: '#4f46e533',
    black: '#f3f4f6',
    red: '#dc2626',
    green: '#16a34a',
    yellow: '#ca8a04',
    blue: '#2563eb',
    magenta: '#7c3aed',
    cyan: '#0891b2',
    white: '#1f2937',
    brightBlack: '#9ca3af',
    brightRed: '#ef4444',
    brightGreen: '#22c55e',
    brightYellow: '#eab308',
    brightBlue: '#3b82f6',
    brightMagenta: '#8b5cf6',
    brightCyan: '#06b6d4',
    brightWhite: '#111827',
  };
}

function generateId() {
  return Math.random().toString(36).slice(2, 10);
}

// Get server URL from localStorage or default
function getServerUrl(): string {
  return localStorage.getItem('inkwell-server-url') || 'localhost:8910';
}

export function TerminalPanel() {
  const t = useT();
  const [sessions, setSessions] = useState<TerminalSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [showNewMenu, setShowNewMenu] = useState(false);
  const [showSSHModal, setShowSSHModal] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [sshConfig, setSSHConfig] = useState<SSHConfig>({
    host: '', port: '22', username: '', authMethod: 'password', password: '', keyPath: '~/.ssh/id_ed25519',
  });
  const termContainerRef = useRef<HTMLDivElement>(null);
  const sessionsRef = useRef<TerminalSession[]>([]);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);

  sessionsRef.current = sessions;
  const activeSession = sessions.find(s => s.id === activeSessionId);

  // Create a terminal instance
  const createTerminal = useCallback(() => {
    const theme = getTermTheme();
    const term = new Terminal({
      cursorBlink: true,
      cursorStyle: 'bar',
      fontSize: 13,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      lineHeight: 1.2,
      scrollback: 10000,
      theme,
      allowProposedApi: true,
    });
    const fitAddon = new FitAddon();
    const searchAddon = new SearchAddon();
    const webLinksAddon = new WebLinksAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(searchAddon);
    term.loadAddon(webLinksAddon);
    return { term, fitAddon, searchAddon };
  }, []);

  // Open a local terminal session (connection deferred to attach effect)
  const openLocalSession = useCallback(() => {
    const id = generateId();
    const { term, fitAddon, searchAddon } = createTerminal();

    const session: TerminalSession = {
      id, label: 'Local', type: 'local',
      terminal: term, fitAddon, searchAddon,
      connected: false, needsConnect: true,
    };

    setSessions(prev => [...prev, session]);
    setActiveSessionId(id);
    setShowNewMenu(false);
  }, [createTerminal]);

  // Open an SSH session
  const openSSHSession = useCallback(() => {
    const id = generateId();
    const { term, fitAddon, searchAddon } = createTerminal();
    const label = `${sshConfig.username}@${sshConfig.host}`;

    const session: TerminalSession = {
      id, label, type: 'ssh',
      terminal: term, fitAddon, searchAddon,
      connected: false, needsConnect: true,
      sshConfig: { ...sshConfig },
    };

    setSessions(prev => [...prev, session]);
    setActiveSessionId(id);
    setShowSSHModal(false);
    setShowNewMenu(false);
  }, [createTerminal, sshConfig]);

  // Close a session
  const closeSession = useCallback((id: string) => {
    const s = sessionsRef.current.find(s => s.id === id);
    if (s) {
      s.ws?.close();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const unlisten = (s as any)._unlisten;
      if (typeof unlisten === 'function') unlisten();
      if (isTauri) {
        tauriInvoke('kill_pty', { sessionId: id }).catch(() => {});
      }
      s.terminal.dispose();
    }
    setSessions(prev => {
      const next = prev.filter(s => s.id !== id);
      if (id === activeSessionId) {
        setActiveSessionId(next.length > 0 ? next[next.length - 1].id : null);
      }
      return next;
    });
  }, [activeSessionId]);

  // Connect a session to PTY/WebSocket (called after terminal is in the DOM)
  const connectSession = useCallback((s: TerminalSession) => {
    const { id, terminal: term, type: sessionType } = s;

    if (isTauri) {
      const doConnect = async () => {
        try {
          if (sessionType === 'ssh' && s.sshConfig) {
            await tauriInvoke('spawn_ssh', {
              sessionId: id, host: s.sshConfig.host,
              port: parseInt(s.sshConfig.port), username: s.sshConfig.username,
              authMethod: s.sshConfig.authMethod,
              password: s.sshConfig.authMethod === 'password' ? s.sshConfig.password : undefined,
              keyPath: s.sshConfig.authMethod === 'key' ? s.sshConfig.keyPath : undefined,
              cols: term.cols, rows: term.rows,
            });
          } else {
            await tauriInvoke('spawn_pty', { sessionId: id, cols: term.cols, rows: term.rows });
          }

          const unlisten = await tauriListen(`pty-output-${id}`, (event) => {
            term.write(event.payload);
          });

          term.onData((data: string) => {
            tauriInvoke('write_pty', { sessionId: id, data }).catch(() => {});
          });
          term.onResize(({ cols, rows }: { cols: number; rows: number }) => {
            tauriInvoke('resize_pty', { sessionId: id, cols, rows }).catch(() => {});
          });

          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (s as any)._unlisten = unlisten;
          setSessions(prev => prev.map(x => x.id === id ? { ...x, connected: true, needsConnect: false } : x));
        } catch (e) {
          term.writeln(`\r\n\x1b[31mError: ${e}\x1b[0m`);
        }
      };
      doConnect();
    } else {
      // Web: WebSocket
      const serverUrl = getServerUrl();
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      let url = `${protocol}//${serverUrl}/ws/terminal?type=local`;

      if (sessionType === 'ssh' && s.sshConfig) {
        const params = new URLSearchParams({
          type: 'ssh', host: s.sshConfig.host, port: s.sshConfig.port,
          username: s.sshConfig.username, auth_method: s.sshConfig.authMethod,
        });
        if (s.sshConfig.authMethod === 'password') params.set('password', s.sshConfig.password);
        if (s.sshConfig.authMethod === 'key') params.set('key_path', s.sshConfig.keyPath);
        url = `${protocol}//${serverUrl}/ws/terminal?${params}`;
      }

      const ws = new WebSocket(url);
      ws.binaryType = 'arraybuffer';

      ws.onopen = () => {
        ws.send(JSON.stringify({ type: 'resize', cols: term.cols, rows: term.rows }));
        setSessions(prev => prev.map(x => x.id === id ? { ...x, connected: true, needsConnect: false, ws } : x));
      };
      ws.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer) {
          term.write(new Uint8Array(event.data));
        } else {
          term.write(event.data);
        }
      };
      ws.onclose = () => {
        term.writeln('\r\n\x1b[33m[Session ended]\x1b[0m');
        setSessions(prev => prev.map(x => x.id === id ? { ...x, connected: false } : x));
      };
      term.onData((data) => { if (ws.readyState === WebSocket.OPEN) ws.send(data); });
      term.onResize(({ cols, rows }) => {
        if (ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ type: 'resize', cols, rows }));
      });
      s.ws = ws;
      setSessions(prev => prev.map(x => x.id === id ? { ...x, needsConnect: false } : x));
    }
  }, []);

  // Attach active terminal to DOM, then connect if needed
  useEffect(() => {
    if (!activeSession || !termContainerRef.current) return;
    const container = termContainerRef.current;
    const term = activeSession.terminal;

    // Clear container and attach
    container.innerHTML = '';
    term.open(container);

    // Fit after DOM is ready, then connect
    const rafId = requestAnimationFrame(() => {
      try { activeSession.fitAddon.fit(); } catch {}
      term.focus();
      // Connect PTY/WebSocket now that terminal is in the DOM with correct dimensions
      if (activeSession.needsConnect) {
        connectSession(activeSession);
      }
    });

    // Resize observer
    if (resizeObserverRef.current) resizeObserverRef.current.disconnect();
    const ro = new ResizeObserver(() => {
      try { activeSession.fitAddon.fit(); } catch {}
    });
    ro.observe(container);
    resizeObserverRef.current = ro;

    return () => {
      cancelAnimationFrame(rafId);
      ro.disconnect();
    };
  }, [activeSession?.id, connectSession]); // eslint-disable-line react-hooks/exhaustive-deps

  // Search
  useEffect(() => {
    if (activeSession && searchQuery) {
      activeSession.searchAddon.findNext(searchQuery);
    }
  }, [searchQuery, activeSession]);

  // Keyboard shortcut for search
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f' && activeSession) {
        e.preventDefault();
        setShowSearch(v => !v);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [activeSession]);

  // Auto-open first session (once only)
  const didInit = useRef(false);
  useEffect(() => {
    if (!didInit.current) {
      didInit.current = true;
      openLocalSession();
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-primary)]">
      {/* Tab bar */}
      <div className="flex items-center border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] min-h-[36px]">
        <div className="flex-1 flex items-center overflow-x-auto no-scrollbar">
          {sessions.map(s => (
            <div
              key={s.id}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs cursor-pointer border-r border-[var(--color-border)] whitespace-nowrap ${
                s.id === activeSessionId
                  ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)]'
                  : 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg-hover)]'
              }`}
              onClick={() => setActiveSessionId(s.id)}
            >
              {s.type === 'ssh' ? (
                <Globe size={11} className="text-[var(--color-accent)]" />
              ) : (
                <MonitorSmartphone size={11} className="text-green-400" />
              )}
              <span>{s.label}</span>
              <span className={`w-1.5 h-1.5 rounded-full ${s.connected ? 'bg-green-400' : 'bg-zinc-500'}`} />
              <button
                onClick={(e) => { e.stopPropagation(); closeSession(s.id); }}
                className="ml-1 hover:text-red-400 transition-colors"
              >
                <X size={10} />
              </button>
            </div>
          ))}
        </div>

        {/* New session button */}
        <div className="relative flex items-center px-1">
          <button
            onClick={() => setShowNewMenu(v => !v)}
            className="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] rounded transition-colors"
            title={t('terminal.new')}
          >
            <Plus size={14} />
          </button>
          {showSearch && (
            <button
              onClick={() => setShowSearch(false)}
              className="p-1.5 text-[var(--color-accent)] hover:bg-[var(--color-bg-hover)] rounded"
            >
              <Search size={14} />
            </button>
          )}

          {showNewMenu && (
            <div className="absolute right-0 top-full mt-1 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1 w-48 animate-fadeIn">
              <button
                onClick={openLocalSession}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] transition-colors"
              >
                <MonitorSmartphone size={13} className="text-green-400" />
                {t('terminal.local')}
              </button>
              <button
                onClick={() => { setShowSSHModal(true); setShowNewMenu(false); }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] transition-colors"
              >
                <Globe size={13} className="text-[var(--color-accent)]" />
                {t('terminal.ssh')}
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Search bar */}
      {showSearch && (
        <div className="flex items-center gap-2 px-3 py-1.5 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]">
          <Search size={12} className="text-[var(--color-text-muted)]" />
          <input
            autoFocus
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter') activeSession?.searchAddon.findNext(searchQuery);
              if (e.key === 'Escape') setShowSearch(false);
            }}
            placeholder={t('terminal.search')}
            className="flex-1 bg-transparent text-xs text-[var(--color-text-primary)] outline-none placeholder:text-[var(--color-text-muted)]"
          />
          <button onClick={() => setShowSearch(false)} className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]">
            <X size={12} />
          </button>
        </div>
      )}

      {/* Terminal container */}
      <div ref={termContainerRef} className="flex-1 min-h-0" />

      {/* SSH Modal */}
      {showSSHModal && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl animate-fadeIn">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">{t('terminal.ssh_connect')}</h3>
              <button onClick={() => setShowSSHModal(false)} className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]">
                <X size={16} />
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-xs text-[var(--color-text-muted)] mb-1 block">{t('terminal.host')}</label>
                <input
                  value={sshConfig.host}
                  onChange={e => setSSHConfig(c => ({ ...c, host: e.target.value }))}
                  placeholder="192.168.1.100"
                  className="w-full px-2.5 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
                />
              </div>
              <div className="flex gap-2">
                <div className="flex-1">
                  <label className="text-xs text-[var(--color-text-muted)] mb-1 block">Port</label>
                  <input
                    value={sshConfig.port}
                    onChange={e => setSSHConfig(c => ({ ...c, port: e.target.value }))}
                    className="w-full px-2.5 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
                  />
                </div>
                <div className="flex-1">
                  <label className="text-xs text-[var(--color-text-muted)] mb-1 block">{t('terminal.username')}</label>
                  <input
                    value={sshConfig.username}
                    onChange={e => setSSHConfig(c => ({ ...c, username: e.target.value }))}
                    placeholder="root"
                    className="w-full px-2.5 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
                  />
                </div>
              </div>

              <div>
                <label className="text-xs text-[var(--color-text-muted)] mb-1 block">{t('terminal.auth_method')}</label>
                <div className="flex gap-2">
                  <button
                    onClick={() => setSSHConfig(c => ({ ...c, authMethod: 'password' }))}
                    className={`flex-1 px-2 py-1.5 text-xs rounded border ${
                      sshConfig.authMethod === 'password'
                        ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10 text-[var(--color-accent)]'
                        : 'border-[var(--color-border)] text-[var(--color-text-muted)]'
                    }`}
                  >
                    {t('terminal.password')}
                  </button>
                  <button
                    onClick={() => setSSHConfig(c => ({ ...c, authMethod: 'key' }))}
                    className={`flex-1 px-2 py-1.5 text-xs rounded border ${
                      sshConfig.authMethod === 'key'
                        ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10 text-[var(--color-accent)]'
                        : 'border-[var(--color-border)] text-[var(--color-text-muted)]'
                    }`}
                  >
                    {t('terminal.ssh_key')}
                  </button>
                </div>
              </div>

              {sshConfig.authMethod === 'password' ? (
                <div>
                  <label className="text-xs text-[var(--color-text-muted)] mb-1 block">{t('terminal.password')}</label>
                  <input
                    type="password"
                    value={sshConfig.password}
                    onChange={e => setSSHConfig(c => ({ ...c, password: e.target.value }))}
                    className="w-full px-2.5 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
                  />
                </div>
              ) : (
                <div>
                  <label className="text-xs text-[var(--color-text-muted)] mb-1 block">{t('terminal.key_path')}</label>
                  <input
                    value={sshConfig.keyPath}
                    onChange={e => setSSHConfig(c => ({ ...c, keyPath: e.target.value }))}
                    className="w-full px-2.5 py-1.5 text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
                  />
                </div>
              )}

              <button
                onClick={openSSHSession}
                disabled={!sshConfig.host || !sshConfig.username}
                className="w-full py-2 text-xs font-medium rounded-lg bg-[var(--color-accent)] text-white hover:opacity-90 transition-opacity disabled:opacity-40"
              >
                {t('terminal.connect')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
