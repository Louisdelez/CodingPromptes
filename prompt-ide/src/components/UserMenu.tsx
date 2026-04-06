import { useState } from 'react';
import { LogOut, ChevronDown, User } from 'lucide-react';
import { logout, type AuthSession } from '../lib/auth';
import { useT } from '../lib/i18n';
import { ProfileModal } from './ProfileModal';

interface UserMenuProps {
  session: AuthSession;
  onLogout: () => void;
  onSessionUpdate: (session: AuthSession) => void;
}

export function UserMenu({ session, onLogout, onSessionUpdate }: UserMenuProps) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const [showProfile, setShowProfile] = useState(false);

  const avatar = (() => {
    try { return JSON.parse(session.avatar); }
    catch { return { color: '#6366f1', initials: '?' }; }
  })();

  const handleLogout = () => {
    logout();
    onLogout();
    setOpen(false);
  };

  return (
    <div className="relative" data-dropdown>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 px-1.5 py-1 rounded-lg hover:bg-[var(--color-bg-hover)] transition-colors"
      >
        <div
          className="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0"
          style={{ backgroundColor: avatar.color }}
        >
          {avatar.initials}
        </div>
        <span className="text-xs text-[var(--color-text-secondary)] max-w-[80px] truncate hidden sm:block">
          {session.displayName}
        </span>
        <ChevronDown size={10} className="text-[var(--color-text-muted)]" />
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-1 w-48 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1 animate-fadeIn">
          {/* User info */}
          <div className="px-3 py-2 border-b border-[var(--color-border)]">
            <div className="flex items-center gap-2">
              <div
                className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold text-white"
                style={{ backgroundColor: avatar.color }}
              >
                {avatar.initials}
              </div>
              <div className="min-w-0">
                <div className="text-sm font-medium text-[var(--color-text-primary)] truncate">{session.displayName}</div>
                <div className="text-[10px] text-[var(--color-text-muted)] truncate">{session.email}</div>
              </div>
            </div>
          </div>

          {/* Actions */}
          <button
            onClick={() => {
              setShowProfile(true);
              setOpen(false);
            }}
            className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] transition-colors"
          >
            <User size={13} />
            {t('auth.profile')}
          </button>
          <button
            onClick={handleLogout}
            className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-danger)] hover:bg-[var(--color-bg-hover)] transition-colors"
          >
            <LogOut size={13} />
            {t('auth.logout')}
          </button>
        </div>
      )}

      {showProfile && (
        <ProfileModal
          session={session}
          onClose={() => setShowProfile(false)}
          onSessionUpdate={onSessionUpdate}
        />
      )}
    </div>
  );
}
