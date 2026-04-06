import { getLocalServerUrl } from './types';

const TOKEN_KEY = 'prompt-ide-jwt-token';

// --- Token management ---

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

function getBaseUrl(): string {
  return getLocalServerUrl().replace(/\/$/, '');
}

// --- HTTP helper ---

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  const token = getToken();
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${getBaseUrl()}/api${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  if (res.status === 401) {
    clearToken();
    throw new Error('UNAUTHORIZED');
  }

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || `HTTP ${res.status}`);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

// --- Auth ---

export interface BackendUser {
  id: string;
  email: string;
  display_name: string;
  avatar: string;
}

interface AuthResponse {
  token: string;
  user: BackendUser;
}

export async function register(email: string, password: string, displayName: string): Promise<AuthResponse> {
  const resp = await request<AuthResponse>('POST', '/auth/register', { email, password, display_name: displayName });
  setToken(resp.token);
  return resp;
}

export async function login(email: string, password: string): Promise<AuthResponse> {
  const resp = await request<AuthResponse>('POST', '/auth/login', { email, password });
  setToken(resp.token);
  return resp;
}

export async function getMe(): Promise<BackendUser> {
  return request<BackendUser>('GET', '/auth/me');
}

export function logout(): void {
  clearToken();
}

export function isLoggedIn(): boolean {
  return !!getToken();
}

// --- Workspaces ---

export interface BackendWorkspace {
  id: string;
  name: string;
  description: string;
  color: string;
  user_id: string;
  created_at: number;
  updated_at: number;
}

export async function listWorkspaces(): Promise<BackendWorkspace[]> {
  return request('GET', '/workspaces');
}

export async function createWorkspace(data: { name: string; color: string; description?: string }): Promise<BackendWorkspace> {
  return request('POST', '/workspaces', data);
}

export async function deleteWorkspace(id: string): Promise<void> {
  return request('DELETE', `/workspaces/${id}`);
}

// --- Projects ---

export interface BackendProject {
  id: string;
  name: string;
  user_id: string;
  workspace_id: string | null;
  blocks_json: string;
  variables_json: string;
  framework: string | null;
  tags_json: string;
  created_at: number;
  updated_at: number;
}

export async function listProjects(): Promise<BackendProject[]> {
  return request('GET', '/projects');
}

export async function createProject(data: {
  id?: string; name: string; workspace_id?: string | null;
  blocks_json: string; variables_json?: string; framework?: string | null; tags_json?: string;
}): Promise<BackendProject> {
  return request('POST', '/projects', data);
}

export async function updateProject(id: string, data: {
  name?: string; workspace_id?: string | null;
  blocks_json?: string; variables_json?: string; framework?: string | null; tags_json?: string;
}): Promise<BackendProject> {
  return request('PUT', `/projects/${id}`, data);
}

export async function deleteProject(id: string): Promise<void> {
  return request('DELETE', `/projects/${id}`);
}

// --- Versions ---

export interface BackendVersion {
  id: string;
  project_id: string;
  blocks_json: string;
  variables_json: string;
  label: string;
  created_at: number;
}

export async function listVersions(projectId: string): Promise<BackendVersion[]> {
  return request('GET', `/projects/${projectId}/versions`);
}

export async function createVersion(projectId: string, data: { blocks_json: string; variables_json: string; label: string }): Promise<BackendVersion> {
  return request('POST', `/projects/${projectId}/versions`, data);
}

// --- Executions ---

export interface BackendExecution {
  id: string;
  project_id: string;
  model: string;
  provider: string;
  prompt: string;
  response: string;
  tokens_in: number;
  tokens_out: number;
  cost: number;
  latency_ms: number;
  created_at: number;
}

export async function listExecutions(projectId: string): Promise<BackendExecution[]> {
  return request('GET', `/projects/${projectId}/executions`);
}

export async function createExecution(projectId: string, data: {
  model: string; provider: string; prompt: string; response: string;
  tokens_in: number; tokens_out: number; cost: number; latency_ms: number;
}): Promise<BackendExecution> {
  return request('POST', `/projects/${projectId}/executions`, data);
}

// --- Frameworks ---

export interface BackendFramework {
  id: string;
  name: string;
  description: string;
  blocks_json: string;
  user_id: string;
  created_at: number;
  updated_at: number;
}

export async function listFrameworks(): Promise<BackendFramework[]> {
  return request('GET', '/frameworks');
}

export async function createFramework(data: { name: string; description?: string; blocks_json: string }): Promise<BackendFramework> {
  return request('POST', '/frameworks', data);
}

export async function updateFramework(id: string, data: { name?: string; description?: string; blocks_json?: string }): Promise<BackendFramework> {
  return request('PUT', `/frameworks/${id}`, data);
}

export async function deleteFramework(id: string): Promise<void> {
  return request('DELETE', `/frameworks/${id}`);
}

// --- Config ---

export async function getConfig(): Promise<Record<string, string>> {
  return request('GET', '/config');
}

export async function setConfig(config: Record<string, string>): Promise<void> {
  return request('PUT', '/config', { config });
}
