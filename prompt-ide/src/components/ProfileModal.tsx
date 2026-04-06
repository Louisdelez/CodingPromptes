import { useState } from 'react';
import { X } from 'lucide-react';
import type { AuthSession } from '../lib/auth';
import { useT } from '../lib/i18n';

interface ProfileModalProps {
  session: AuthSession;
  onClose: () => void;
  onSessionUpdate: (session: AuthSession) => void;
}

export function ProfileModal({ session, onClose }: ProfileModalProps) {
  const t = useT();
  const [displayName] = useState(session.displayName);

  const avatar = (() => {
    try { return JSON.parse(session.avatar); }
    catch { return { color: '#6366f1', initials: '?' }; }
  })();

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
              <div className="text-sm font-medium text-[var(--color-text-primary)]">{displayName}</div>
              <div className="text-xs text-[var(--color-text-muted)]">{session.email}</div>
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
