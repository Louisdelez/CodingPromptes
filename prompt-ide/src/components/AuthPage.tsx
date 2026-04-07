import { useState, useEffect, useCallback } from 'react';
import { Mail, Lock, User, AlertCircle, Loader2, Globe, Sun, Moon, Monitor, ChevronDown, Server } from 'lucide-react';
import { login, register, oauthGoogle, oauthGithub, type AuthSession } from '../lib/auth';
import { useT } from '../lib/i18n';
import type { Lang } from '../lib/i18n';
import type { ThemeMode } from '../lib/theme';
import { getLocalServerUrl, setLocalServerUrl } from '../lib/types';

interface AuthPageProps {
  onAuth: (session: AuthSession) => void;
  language: Lang;
  onLanguageChange: (lang: Lang) => void;
  themeMode: ThemeMode;
  onThemeChange: (mode: ThemeMode) => void;
}

export function AuthPage({ onAuth, language, onLanguageChange, themeMode, onThemeChange }: AuthPageProps) {
  const t = useT();
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [showThemeMenu, setShowThemeMenu] = useState(false);
  const [showLangMenu, setShowLangMenu] = useState(false);
  const [oauthLoading, setOauthLoading] = useState(false);
  const [serverUrl, setServerUrl] = useState(getLocalServerUrl);

  const GITHUB_CLIENT_ID = (typeof import.meta !== 'undefined' && ((import.meta as unknown) as Record<string, Record<string, string>>).env?.VITE_GITHUB_CLIENT_ID) || '';

  // Handle GitHub OAuth callback
  const handleGitHubCallback = useCallback(async (code: string) => {
    setOauthLoading(true);
    setError(null);
    try {
      const session = await oauthGithub(code);
      onAuth(session);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'GitHub OAuth failed';
      setError(msg);
    } finally {
      setOauthLoading(false);
    }
  }, [onAuth]);

  useEffect(() => {
    // Check if we're returning from GitHub OAuth
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    if (code) {
      // Clean URL
      window.history.replaceState({}, '', window.location.pathname);
      handleGitHubCallback(code);
    }
  }, [handleGitHubCallback]);

  // Google OAuth handler
  const handleGoogleLogin = useCallback(async () => {
    setOauthLoading(true);
    setError(null);
    try {
      // Dynamically load Google Identity Services script
      await new Promise<void>((resolve, reject) => {
        if (document.querySelector('script[src*="accounts.google.com/gsi/client"]')) {
          resolve();
          return;
        }
        const script = document.createElement('script');
        script.src = 'https://accounts.google.com/gsi/client';
        script.async = true;
        script.onload = () => resolve();
        script.onerror = () => reject(new Error('Failed to load Google script'));
        document.head.appendChild(script);
      });

      // Initialize and prompt
      const google = (window as unknown as Record<string, unknown>).google as {
        accounts: { id: {
          initialize: (config: Record<string, unknown>) => void;
          prompt: (cb: (notification: { isNotDisplayed: () => boolean }) => void) => void;
        } };
      };
      if (!google?.accounts?.id) {
        throw new Error('Google Identity Services not available');
      }

      google.accounts.id.initialize({
        client_id: (typeof import.meta !== 'undefined' && ((import.meta as unknown) as Record<string, Record<string, string>>).env?.VITE_GOOGLE_CLIENT_ID) || '',
        callback: async (response: { credential: string }) => {
          try {
            const session = await oauthGoogle(response.credential);
            onAuth(session);
          } catch (err) {
            const msg = err instanceof Error ? err.message : 'Google OAuth failed';
            setError(msg);
            setOauthLoading(false);
          }
        },
      });
      google.accounts.id.prompt((notification) => {
        if (notification.isNotDisplayed()) {
          setError('Google sign-in popup was blocked. Please allow popups.');
          setOauthLoading(false);
        }
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Google OAuth failed';
      setError(msg);
      setOauthLoading(false);
    }
  }, [onAuth]);

  const handleGitHubRedirect = () => {
    if (!GITHUB_CLIENT_ID) {
      setError('GitHub OAuth is not configured');
      return;
    }
    window.location.href = `https://github.com/login/oauth/authorize?client_id=${GITHUB_CLIENT_ID}&scope=user:email`;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (mode === 'register') {
        if (password !== confirmPassword) {
          throw new Error('PASSWORD_MISMATCH');
        }
        const session = await register(email, password, displayName || email.split('@')[0]);
        onAuth(session);
      } else {
        const session = await login(email, password);
        onAuth(session);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'UNKNOWN';
      switch (msg) {
        case 'INVALID_EMAIL': setError(t('auth.invalidEmail')); break;
        case 'EMAIL_EXISTS': setError(t('auth.emailExists')); break;
        case 'INVALID_CREDENTIALS': setError(t('auth.invalidCredentials')); break;
        case 'PASSWORD_TOO_SHORT': setError(t('auth.passwordTooShort')); break;
        case 'PASSWORD_MISMATCH': setError(t('auth.passwordMismatch')); break;
        default: setError(msg);
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex flex-col bg-[var(--color-bg-primary)]">
      {/* Top bar with theme/lang */}
      <div className="flex justify-end gap-1.5 p-3">
        <div className="relative" data-dropdown>
          <button
            onClick={() => { setShowThemeMenu(!showThemeMenu); setShowLangMenu(false); }}
            className="flex items-center gap-1 px-2 py-1 rounded text-xs font-medium bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-secondary)] transition-colors"
          >
            {themeMode === 'light' ? <Sun size={12} /> : themeMode === 'dark' ? <Moon size={12} /> : <Monitor size={12} />}
            <ChevronDown size={10} />
          </button>
          {showThemeMenu && (
            <div className="absolute right-0 top-full mt-1 w-32 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1 animate-fadeIn">
              {([
                { mode: 'light' as ThemeMode, icon: Sun, label: 'Light' },
                { mode: 'dark' as ThemeMode, icon: Moon, label: 'Dark' },
                { mode: 'system' as ThemeMode, icon: Monitor, label: 'System' },
              ]).map(({ mode: m, icon: Icon, label }) => (
                <button
                  key={m}
                  onClick={() => { onThemeChange(m); setShowThemeMenu(false); }}
                  className={`w-full flex items-center gap-2 px-3 py-1.5 text-xs transition-colors ${
                    themeMode === m ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10' : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                  }`}
                >
                  <Icon size={13} /> {label}
                </button>
              ))}
            </div>
          )}
        </div>
        <div className="relative" data-dropdown>
          <button
            onClick={() => { setShowLangMenu(!showLangMenu); setShowThemeMenu(false); }}
            className="flex items-center gap-1 px-2 py-1 rounded text-xs font-medium bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-secondary)] transition-colors"
          >
            <Globe size={12} /> {language.toUpperCase()} <ChevronDown size={10} />
          </button>
          {showLangMenu && (
            <div className="absolute right-0 top-full mt-1 w-32 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1 animate-fadeIn">
              {([{ code: 'fr' as Lang, label: 'Francais' }, { code: 'en' as Lang, label: 'English' }]).map(({ code, label }) => (
                <button
                  key={code}
                  onClick={() => { onLanguageChange(code); setShowLangMenu(false); }}
                  className={`w-full flex items-center gap-2 px-3 py-1.5 text-xs transition-colors ${
                    language === code ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10' : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Auth form */}
      <div className="flex-1 flex items-center justify-center p-4">
        <div className="w-full max-w-sm space-y-6">
          {/* Logo */}
          <div className="text-center space-y-2">
            <img src="/icon-192.png" alt="Inkwell" className="w-14 h-14 rounded-2xl mx-auto shadow-lg" />
            <h1 className="text-xl font-bold text-[var(--color-text-primary)]">{t('auth.welcome')}</h1>
            <p className="text-sm text-[var(--color-text-muted)]">{t('auth.subtitle')}</p>
          </div>

          {/* Tabs */}
          <div className="flex rounded-lg bg-[var(--color-bg-tertiary)] p-1">
            <button
              onClick={() => { setMode('login'); setError(null); }}
              className={`flex-1 py-2 text-sm font-medium rounded-md transition-colors ${
                mode === 'login'
                  ? 'bg-[var(--color-accent)] text-white shadow-sm'
                  : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
              }`}
            >
              {t('auth.login')}
            </button>
            <button
              onClick={() => { setMode('register'); setError(null); }}
              className={`flex-1 py-2 text-sm font-medium rounded-md transition-colors ${
                mode === 'register'
                  ? 'bg-[var(--color-accent)] text-white shadow-sm'
                  : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
              }`}
            >
              {t('auth.register')}
            </button>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Server URL */}
            <div className="space-y-1">
              <label className="text-xs text-[var(--color-text-muted)]">{t('auth.serverUrl')}</label>
              <div className="relative">
                <Server size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
                <input
                  type="url"
                  value={serverUrl}
                  onChange={(e) => { setServerUrl(e.target.value); setLocalServerUrl(e.target.value); }}
                  placeholder="http://192.168.1.x:8910"
                  className="w-full pl-10 pr-4 py-2.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
                />
              </div>
            </div>

            {mode === 'register' && (
              <div className="space-y-1">
                <label className="text-xs text-[var(--color-text-muted)]">{t('auth.displayName')}</label>
                <div className="relative">
                  <User size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
                  <input
                    type="text"
                    value={displayName}
                    onChange={(e) => setDisplayName(e.target.value)}
                    placeholder="John Doe"
                    className="w-full pl-10 pr-4 py-2.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
                  />
                </div>
              </div>
            )}

            <div className="space-y-1">
              <label className="text-xs text-[var(--color-text-muted)]">{t('auth.email')}</label>
              <div className="relative">
                <Mail size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="email@example.com"
                  required
                  className="w-full pl-10 pr-4 py-2.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
                />
              </div>
            </div>

            <div className="space-y-1">
              <label className="text-xs text-[var(--color-text-muted)]">{t('auth.password')}</label>
              <div className="relative">
                <Lock size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  required
                  className="w-full pl-10 pr-4 py-2.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
                />
              </div>
            </div>

            {mode === 'register' && (
              <div className="space-y-1">
                <label className="text-xs text-[var(--color-text-muted)]">{t('auth.confirmPassword')}</label>
                <div className="relative">
                  <Lock size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
                  <input
                    type="password"
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder="••••••••"
                    required
                    className="w-full pl-10 pr-4 py-2.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
                  />
                </div>
              </div>
            )}

            {error && (
              <div className="flex items-center gap-2 p-3 rounded-lg bg-[var(--color-danger)]/10 text-[var(--color-danger)] text-sm animate-fadeIn">
                <AlertCircle size={16} />
                <span>{error}</span>
              </div>
            )}

            <button
              type="submit"
              disabled={loading}
              className="w-full flex items-center justify-center gap-2 py-2.5 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white font-medium text-sm disabled:opacity-50 transition-colors"
            >
              {loading && <Loader2 size={16} className="animate-spin" />}
              {mode === 'login' ? t('auth.loginButton') : t('auth.registerButton')}
            </button>
          </form>

          {/* OAuth divider */}
          <div className="flex items-center gap-3">
            <div className="flex-1 h-px bg-[var(--color-border)]" />
            <span className="text-xs text-[var(--color-text-muted)]">{t('auth.or')}</span>
            <div className="flex-1 h-px bg-[var(--color-border)]" />
          </div>

          {/* OAuth buttons */}
          <div className="space-y-2">
            <button
              type="button"
              onClick={handleGoogleLogin}
              disabled={oauthLoading}
              className="w-full flex items-center justify-center gap-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-primary)] font-medium text-sm disabled:opacity-50 transition-colors"
            >
              <svg width="18" height="18" viewBox="0 0 24 24">
                <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
              </svg>
              {t('auth.google')}
            </button>

            <button
              type="button"
              onClick={handleGitHubRedirect}
              disabled={oauthLoading}
              className="w-full flex items-center justify-center gap-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-primary)] font-medium text-sm disabled:opacity-50 transition-colors"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              {t('auth.github')}
            </button>
          </div>

          {/* Switch mode */}
          <p className="text-center text-sm text-[var(--color-text-muted)]">
            {mode === 'login' ? t('auth.noAccount') : t('auth.hasAccount')}{' '}
            <button
              onClick={() => { setMode(mode === 'login' ? 'register' : 'login'); setError(null); }}
              className="text-[var(--color-accent)] hover:underline font-medium"
            >
              {mode === 'login' ? t('auth.register') : t('auth.login')}
            </button>
          </p>
        </div>
      </div>
    </div>
  );
}
