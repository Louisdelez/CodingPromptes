import Dexie, { type EntityTable } from 'dexie';
import type { PromptProject, PromptVersion, ExecutionResult, ApiKeys, Workspace, CustomFramework } from './types';
import type { User } from './auth';

class PromptIdeDB extends Dexie {
  users!: EntityTable<User, 'id'>;
  workspaces!: EntityTable<Workspace, 'id'>;
  projects!: EntityTable<PromptProject, 'id'>;
  versions!: EntityTable<PromptVersion, 'id'>;
  executions!: EntityTable<ExecutionResult, 'id'>;
  frameworks!: EntityTable<CustomFramework, 'id'>;

  constructor() {
    super('PromptIdeDB');

    this.version(1).stores({
      projects: 'id, name, updatedAt, *tags',
      versions: 'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
    });

    this.version(2).stores({
      workspaces: 'id, name, updatedAt',
      projects: 'id, name, workspaceId, updatedAt, *tags',
      versions: 'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
    });

    this.version(3).stores({
      workspaces: 'id, name, updatedAt',
      projects: 'id, name, workspaceId, updatedAt, *tags',
      versions: 'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
      frameworks: 'id, name, updatedAt',
    });

    this.version(4).stores({
      users: 'id, email',
      workspaces: 'id, name, userId, updatedAt',
      projects: 'id, name, userId, workspaceId, updatedAt, *tags',
      versions: 'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
      frameworks: 'id, name, userId, updatedAt',
    });
  }
}

export const db = new PromptIdeDB();

const API_KEYS_STORAGE_KEY = 'prompt-ide-api-keys';

export function getApiKeys(): ApiKeys {
  try {
    const raw = localStorage.getItem(API_KEYS_STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

export function setApiKeys(keys: ApiKeys): void {
  localStorage.setItem(API_KEYS_STORAGE_KEY, JSON.stringify(keys));
}
