import { useState } from 'react';
import { Mail, Lock, User, AlertCircle, Loader2, Globe, Sun, Moon, Monitor, ChevronDown } from 'lucide-react';
import { login, register, type AuthSession } from '../lib/auth';
import { useT } from '../lib/i18n';
import type { Lang } from '../lib/i18n';
import type { ThemeMode } from '../lib/theme';

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
