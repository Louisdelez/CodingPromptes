import { db } from './db';

export interface User {
  id: string;
  email: string;
  displayName: string;
  passwordHash: string;
  salt: string;
  avatar: string;
  createdAt: number;
}

export interface AuthSession {
  userId: string;
  email: string;
  displayName: string;
  avatar: string;
}

const SESSION_KEY = 'prompt-ide-session';

// --- Password hashing with Web Crypto (PBKDF2) ---

async function generateSalt(): Promise<string> {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return Array.from(bytes).map((b) => b.toString(16).padStart(2, '0')).join('');
}

async function hashPassword(password: string, salt: string): Promise<string> {
  const encoder = new TextEncoder();
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    'PBKDF2',
    false,
    ['deriveBits']
  );
  const bits = await crypto.subtle.deriveBits(
    { name: 'PBKDF2', salt: encoder.encode(salt), iterations: 100000, hash: 'SHA-256' },
    keyMaterial,
    256
  );
  return Array.from(new Uint8Array(bits)).map((b) => b.toString(16).padStart(2, '0')).join('');
}

// --- Avatar generation ---

function generateAvatar(name: string): string {
  const colors = ['#6366f1', '#8b5cf6', '#ec4899', '#f43f5e', '#f97316', '#eab308', '#22c55e', '#06b6d4', '#3b82f6'];
  const color = colors[Math.abs(name.split('').reduce((a, c) => a + c.charCodeAt(0), 0)) % colors.length];
  const initials = name.split(' ').map((w) => w[0]).join('').toUpperCase().slice(0, 2) || '?';
  return JSON.stringify({ color, initials });
}

// --- Auth operations ---

export async function register(email: string, password: string, displayName: string): Promise<AuthSession> {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    throw new Error('INVALID_EMAIL');
  }

  const existing = await db.users.where('email').equals(email.toLowerCase()).first();
  if (existing) {
    throw new Error('EMAIL_EXISTS');
  }

  if (password.length < 6) {
    throw new Error('PASSWORD_TOO_SHORT');
  }

  const salt = await generateSalt();
  const passwordHash = await hashPassword(password, salt);

  const user: User = {
    id: crypto.randomUUID(),
    email: email.toLowerCase().trim(),
    displayName: displayName.trim(),
    passwordHash,
    salt,
    avatar: generateAvatar(displayName),
    createdAt: Date.now(),
  };

  await db.users.add(user);

  const session: AuthSession = {
    userId: user.id,
    email: user.email,
    displayName: user.displayName,
    avatar: user.avatar,
  };

  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  return session;
}

export async function login(email: string, password: string): Promise<AuthSession> {
  const user = await db.users.where('email').equals(email.toLowerCase().trim()).first();
  if (!user) {
    throw new Error('INVALID_CREDENTIALS');
  }

  const hash = await hashPassword(password, user.salt);
  if (hash !== user.passwordHash) {
    throw new Error('INVALID_CREDENTIALS');
  }

  const session: AuthSession = {
    userId: user.id,
    email: user.email,
    displayName: user.displayName,
    avatar: user.avatar,
  };

  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  return session;
}

export function logout(): void {
  localStorage.removeItem(SESSION_KEY);
}

export function getSession(): AuthSession | null {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

export async function updateProfile(userId: string, changes: { displayName?: string; avatar?: string }): Promise<AuthSession> {
  await db.users.update(userId, changes);
  const session = getSession();
  if (session && session.userId === userId) {
    const updated = { ...session, ...changes };
    localStorage.setItem(SESSION_KEY, JSON.stringify(updated));
    return updated;
  }
  return session!;
}

export async function changePassword(userId: string, currentPassword: string, newPassword: string): Promise<void> {
  const user = await db.users.get(userId);
  if (!user) throw new Error('USER_NOT_FOUND');

  const hash = await hashPassword(currentPassword, user.salt);
  if (hash !== user.passwordHash) throw new Error('INVALID_CURRENT_PASSWORD');

  if (newPassword.length < 6) throw new Error('PASSWORD_TOO_SHORT');

  const salt = await generateSalt();
  const passwordHash = await hashPassword(newPassword, salt);
  await db.users.update(userId, { passwordHash, salt });
}
