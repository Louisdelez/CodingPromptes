import { useState } from 'react';
import { X } from 'lucide-react';
import { updateProfile, changePassword, type AuthSession } from '../lib/auth';
import { useT } from '../lib/i18n';

interface ProfileModalProps {
  session: AuthSession;
  onClose: () => void;
  onSessionUpdate: (session: AuthSession) => void;
}

export function ProfileModal({ session, onClose, onSessionUpdate }: ProfileModalProps) {
  const t = useT();
  const [displayName, setDisplayName] = useState(session.displayName);
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [profileSaved, setProfileSaved] = useState(false);
  const [passwordError, setPasswordError] = useState('');
  const [passwordSaved, setPasswordSaved] = useState(false);

  const avatar = (() => {
    try { return JSON.parse(session.avatar); }
    catch { return { color: '#6366f1', initials: '?' }; }
  })();

  const handleSaveProfile = async () => {
    const updated = await updateProfile(session.userId, { displayName: displayName.trim() });
    onSessionUpdate(updated);
    setProfileSaved(true);
    setTimeout(() => setProfileSaved(false), 2000);
  };

  const handleChangePassword = async () => {
    setPasswordError('');
    if (newPassword !== confirmPassword) {
      setPasswordError(t('auth.passwordMismatch'));
      return;
    }
    if (newPassword.length < 6) {
      setPasswordError(t('auth.passwordTooShort'));
      return;
    }
    try {
      await changePassword(session.userId, currentPassword, newPassword);
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setPasswordSaved(true);
      setTimeout(() => setPasswordSaved(false), 2000);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : '';
      if (message === 'INVALID_CURRENT_PASSWORD') {
        setPasswordError(t('auth.invalidCredentials'));
      } else if (message === 'PASSWORD_TOO_SHORT') {
        setPasswordError(t('auth.passwordTooShort'));
      } else {
        setPasswordError(t('misc.unknownError'));
      }
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-xl shadow-2xl w-full max-w-md mx-4 animate-fadeIn"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-[var(--color-border)]">
          <h2 className="text-base font-semibold text-[var(--color-text-primary)]">{t('auth.profile')}</h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        <div className="p-5 space-y-5">
          {/* Avatar display */}
          <div className="flex items-center gap-3">
            <div
              className="w-12 h-12 rounded-full flex items-center justify-center text-lg font-bold text-white"
              style={{ backgroundColor: avatar.color }}
            >
              {avatar.initials}
            </div>
            <div>
              <div className="text-sm font-medium text-[var(--color-text-primary)]">{session.displayName}</div>
              <div className="text-xs text-[var(--color-text-muted)]">{session.email}</div>
            </div>
          </div>

          {/* Display name */}
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-[var(--color-text-secondary)]">{t('auth.displayName')}</label>
            <input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
            />
            <div className="flex items-center gap-2">
              <button
                onClick={handleSaveProfile}
                disabled={!displayName.trim() || displayName.trim() === session.displayName}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 transition-colors"
              >
                {t('auth.save')}
              </button>
              {profileSaved && (
                <span className="text-xs text-[var(--color-success)]">{t('auth.saved')}</span>
              )}
            </div>
          </div>

          <div className="h-px bg-[var(--color-border)]" />

          {/* Change password */}
          <div className="space-y-3">
            <h3 className="text-sm font-medium text-[var(--color-text-secondary)]">{t('auth.changePassword')}</h3>
            <div className="space-y-2">
              <input
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                placeholder={t('auth.currentPassword')}
                className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
              />
              <input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder={t('auth.newPassword')}
                className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
              />
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder={t('auth.confirmPassword')}
                className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
              />
            </div>
            {passwordError && (
              <p className="text-xs text-[var(--color-danger)]">{passwordError}</p>
            )}
            <div className="flex items-center gap-2">
              <button
                onClick={handleChangePassword}
                disabled={!currentPassword || !newPassword || !confirmPassword}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 transition-colors"
              >
                {t('auth.changePassword')}
              </button>
              {passwordSaved && (
                <span className="text-xs text-[var(--color-success)]">{t('auth.saved')}</span>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end px-5 py-3 border-t border-[var(--color-border)]">
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-lg text-xs font-medium bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] transition-colors"
          >
            {t('auth.close')}
          </button>
        </div>
      </div>
    </div>
  );
}
