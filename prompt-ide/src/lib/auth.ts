import * as backend from './backend';

export interface AuthSession {
  userId: string;
  email: string;
  displayName: string;
  avatar: string;
}

const SESSION_KEY = 'prompt-ide-session';

function toSession(user: backend.BackendUser): AuthSession {
  return {
    userId: user.id,
    email: user.email,
    displayName: user.display_name,
    avatar: user.avatar,
  };
}

function saveSession(session: AuthSession): void {
  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
}

export async function register(email: string, password: string, displayName: string): Promise<AuthSession> {
  const resp = await backend.register(email, password, displayName);
  const session = toSession(resp.user);
  saveSession(session);
  return session;
}

export async function login(email: string, password: string): Promise<AuthSession> {
  const resp = await backend.login(email, password);
  const session = toSession(resp.user);
  saveSession(session);
  return session;
}

export function logout(): void {
  backend.logout();
  localStorage.removeItem(SESSION_KEY);
}

export function getSession(): AuthSession | null {
  try {
    if (!backend.isLoggedIn()) return null;
    const raw = localStorage.getItem(SESSION_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

export async function validateSession(): Promise<AuthSession | null> {
  try {
    if (!backend.isLoggedIn()) return null;
    const user = await backend.getMe();
    const session = toSession(user);
    saveSession(session);
    return session;
  } catch {
    logout();
    return null;
  }
}
