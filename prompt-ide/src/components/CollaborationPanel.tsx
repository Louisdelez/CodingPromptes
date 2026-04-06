import { useState, useEffect, useCallback } from 'react';
import { RefreshCw, Users, Circle } from 'lucide-react';
import { useT } from '../lib/i18n';
import * as backend from '../lib/backend';

interface CollaborationPanelProps {
  projectId: string;
  currentUserId: string;
}

const USER_COLORS = ['#6366f1', '#8b5cf6', '#ec4899', '#f43f5e', '#f97316', '#22c55e', '#06b6d4', '#3b82f6'];

function colorForUser(userId: string): string {
  const idx = userId.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % USER_COLORS.length;
  return USER_COLORS[idx];
}

export function CollaborationPanel({ projectId, currentUserId }: CollaborationPanelProps) {
  const t = useT();
  const [activeUsers, setActiveUsers] = useState<backend.PresenceUser[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchPresence = useCallback(async () => {
    if (!projectId) return;
    try {
      const users = await backend.getPresence(projectId);
      // Filter out the current user
      setActiveUsers(users.filter(u => u.user_id !== currentUserId));
    } catch {
      // silently ignore presence errors
    }
  }, [projectId, currentUserId]);

  const sendPresence = useCallback(async () => {
    if (!projectId) return;
    try {
      await backend.setPresence(projectId);
    } catch {
      // silently ignore
    }
  }, [projectId]);

  // Poll presence every 10 seconds
  useEffect(() => {
    sendPresence();
    fetchPresence();

    const interval = setInterval(() => {
      sendPresence();
      fetchPresence();
    }, 10_000);

    return () => clearInterval(interval);
  }, [sendPresence, fetchPresence]);

  const handleRefresh = async () => {
    setLoading(true);
    await sendPresence();
    await fetchPresence();
    setLoading(false);
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">{t('collab.title')}</h3>
        <button
          onClick={handleRefresh}
          disabled={loading}
          className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] disabled:opacity-50 transition-colors"
          title={t('collab.refresh')}
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>

      <div className="space-y-1">
        <p className="text-xs text-[var(--color-text-muted)] font-medium">{t('collab.active')}</p>

        {activeUsers.length === 0 ? (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-[var(--color-bg-tertiary)]">
            <Users size={16} className="text-[var(--color-text-muted)]" />
            <span className="text-sm text-[var(--color-text-muted)]">{t('collab.alone')}</span>
          </div>
        ) : (
          <div className="space-y-1">
            {activeUsers.map((user) => (
              <div
                key={user.user_id}
                className="flex items-center gap-2.5 p-2.5 rounded-lg bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] transition-colors"
              >
                <Circle size={8} fill={colorForUser(user.user_id)} stroke="none" />
                <span className="text-sm text-[var(--color-text-primary)] font-medium">{user.display_name}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {activeUsers.length > 0 && (
        <div className="text-xs text-[var(--color-text-muted)] flex items-center gap-1.5">
          <Circle size={6} fill="#22c55e" stroke="none" />
          {activeUsers.length} {activeUsers.length === 1 ? 'collaborateur' : 'collaborateurs'} actif{activeUsers.length > 1 ? 's' : ''}
        </div>
      )}
    </div>
  );
}
