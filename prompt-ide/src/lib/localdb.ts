import Dexie, { type EntityTable } from 'dexie';

// Local-first database — THE source of truth. No sync.

export interface LocalProject {
  id: string;
  name: string;
  workspaceId?: string;
  blocksJson: string;
  variablesJson: string;
  framework?: string;
  tagsJson: string;
  createdAt: number;
  updatedAt: number;
}

export interface LocalWorkspace {
  id: string;
  name: string;
  description: string;
  color: string;
  createdAt: number;
  updatedAt: number;
}

export interface LocalVersion {
  id: string;
  projectId: string;
  blocksJson: string;
  variablesJson: string;
  label: string;
  createdAt: number;
}

export interface LocalExecution {
  id: string;
  projectId: string;
  model: string;
  provider: string;
  prompt: string;
  response: string;
  tokensIn: number;
  tokensOut: number;
  cost: number;
  latencyMs: number;
  createdAt: number;
}

export interface LocalFramework {
  id: string;
  name: string;
  description: string;
  blocksJson: string;
  createdAt: number;
  updatedAt: number;
}

class InkwellLocalDB extends Dexie {
  projects!: EntityTable<LocalProject, 'id'>;
  workspaces!: EntityTable<LocalWorkspace, 'id'>;
  versions!: EntityTable<LocalVersion, 'id'>;
  executions!: EntityTable<LocalExecution, 'id'>;
  frameworks!: EntityTable<LocalFramework, 'id'>;

  constructor() {
    super('InkwellLocalDB');
    this.version(2).stores({
      projects: 'id, workspaceId, updatedAt',
      workspaces: 'id, updatedAt',
      versions: 'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
      frameworks: 'id, updatedAt',
    });
  }
}

export const localdb = new InkwellLocalDB();

// No sync engine. No push. No pull.
// Data lives 100% in IndexedDB. Like VS Code.
export function startSync() { /* no-op */ }
export function stopSync() { /* no-op */ }
export function markDeleted(_id: string) { /* no-op */ }
export async function syncNow() { /* no-op */ }
