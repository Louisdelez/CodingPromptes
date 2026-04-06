import Dexie, { type EntityTable } from 'dexie';

// Local-first database — source of truth for UI

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
  synced: number; // 0 = needs push, 1 = synced with backend
}

export interface LocalWorkspace {
  id: string;
  name: string;
  description: string;
  color: string;
  createdAt: number;
  updatedAt: number;
  synced: number;
}

export interface LocalVersion {
  id: string;
  projectId: string;
  blocksJson: string;
  variablesJson: string;
  label: string;
  createdAt: number;
  synced: number;
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
  synced: number;
}

export interface LocalFramework {
  id: string;
  name: string;
  description: string;
  blocksJson: string;
  createdAt: number;
  updatedAt: number;
  synced: number;
}

// Deleted IDs — persisted in localStorage so they survive page refresh
const DELETED_KEY = 'inkwell-deleted-ids';

function getDeletedIds(): Set<string> {
  try {
    const raw = localStorage.getItem(DELETED_KEY);
    return raw ? new Set(JSON.parse(raw)) : new Set();
  } catch { return new Set(); }
}

function saveDeletedIds(ids: Set<string>) {
  localStorage.setItem(DELETED_KEY, JSON.stringify([...ids]));
}

export function markDeleted(id: string) {
  const ids = getDeletedIds();
  ids.add(id);
  saveDeletedIds(ids);
}

class InkwellLocalDB extends Dexie {
  projects!: EntityTable<LocalProject, 'id'>;
  workspaces!: EntityTable<LocalWorkspace, 'id'>;
  versions!: EntityTable<LocalVersion, 'id'>;
  executions!: EntityTable<LocalExecution, 'id'>;
  frameworks!: EntityTable<LocalFramework, 'id'>;

  constructor() {
    super('InkwellLocalDB');
    this.version(1).stores({
      projects: 'id, workspaceId, updatedAt, synced',
      workspaces: 'id, updatedAt, synced',
      versions: 'id, projectId, createdAt, synced',
      executions: 'id, projectId, createdAt, synced',
      frameworks: 'id, updatedAt, synced',
    });
  }
}

export const localdb = new InkwellLocalDB();

// --- Sync engine ---
import * as backend from './backend';

let syncInterval: ReturnType<typeof setInterval> | null = null;
let hasPopulated = false;

export function startSync() {
  if (syncInterval) return;

  // Check if we need initial population (empty local DB)
  if (!hasPopulated) {
    hasPopulated = true;
    localdb.projects.count().then((count) => {
      if (count === 0) {
        populateFromBackend();
      }
    });
  }

  // Only PUSH every 5 seconds — never pull
  syncInterval = setInterval(pushToBackend, 5000);
}

export function stopSync() {
  if (syncInterval) {
    clearInterval(syncInterval);
    syncInterval = null;
  }
}

// One-time population from backend (only when local is empty)
async function populateFromBackend() {
  if (!backend.getToken()) return;
  try {
    const [bws, bpj] = await Promise.all([
      backend.listWorkspaces(),
      backend.listProjects(),
    ]);
    const deletedIds = getDeletedIds();

    for (const bw of bws) {
      if (deletedIds.has(bw.id)) continue;
      await localdb.workspaces.put({
        id: bw.id, name: bw.name, description: bw.description, color: bw.color,
        createdAt: bw.created_at, updatedAt: bw.updated_at, synced: 1,
      });
    }
    for (const bp of bpj) {
      if (deletedIds.has(bp.id)) continue;
      await localdb.projects.put({
        id: bp.id, name: bp.name, workspaceId: bp.workspace_id ?? undefined,
        blocksJson: bp.blocks_json, variablesJson: bp.variables_json,
        framework: bp.framework ?? undefined, tagsJson: bp.tags_json,
        createdAt: bp.created_at, updatedAt: bp.updated_at, synced: 1,
      });
    }
  } catch { /* offline */ }
}

// Push unsynced local records to backend
async function pushToBackend() {
  if (!backend.getToken()) return;

  // 1. Push deletes first
  const deletedIds = getDeletedIds();
  for (const id of deletedIds) {
    await backend.deleteProject(id).catch(() => {});
    await backend.deleteWorkspace(id).catch(() => {});
  }
  if (deletedIds.size > 0) {
    deletedIds.clear();
    saveDeletedIds(deletedIds);
  }

  // 2. Push unsynced workspaces
  const unsyncedWs = await localdb.workspaces.where('synced').equals(0).toArray();
  for (const ws of unsyncedWs) {
    try {
      await backend.createWorkspace({ name: ws.name, color: ws.color, description: ws.description });
      await localdb.workspaces.update(ws.id, { synced: 1 });
    } catch {
      try {
        await backend.updateWorkspace(ws.id, { name: ws.name, color: ws.color });
        await localdb.workspaces.update(ws.id, { synced: 1 });
      } catch { /* ignore */ }
    }
  }

  // 3. Push unsynced projects
  const unsyncedPj = await localdb.projects.where('synced').equals(0).toArray();
  for (const p of unsyncedPj) {
    // Don't push if it was deleted
    if (getDeletedIds().has(p.id)) continue;
    try {
      await backend.updateProject(p.id, {
        name: p.name, blocks_json: p.blocksJson, variables_json: p.variablesJson,
        workspace_id: p.workspaceId ?? null, framework: p.framework ?? null, tags_json: p.tagsJson,
      });
      await localdb.projects.update(p.id, { synced: 1 });
    } catch {
      try {
        await backend.createProject({
          id: p.id, name: p.name, blocks_json: p.blocksJson, variables_json: p.variablesJson,
          workspace_id: p.workspaceId ?? null, framework: p.framework ?? null, tags_json: p.tagsJson,
        });
        await localdb.projects.update(p.id, { synced: 1 });
      } catch { /* ignore */ }
    }
  }

  // 4. Push unsynced versions
  const unsyncedVer = await localdb.versions.where('synced').equals(0).toArray();
  for (const v of unsyncedVer) {
    try {
      await backend.createVersion(v.projectId, { blocks_json: v.blocksJson, variables_json: v.variablesJson, label: v.label });
      await localdb.versions.update(v.id, { synced: 1 });
    } catch { /* ignore */ }
  }

  // 5. Push unsynced executions
  const unsyncedEx = await localdb.executions.where('synced').equals(0).toArray();
  for (const e of unsyncedEx) {
    try {
      await backend.createExecution(e.projectId, {
        model: e.model, provider: e.provider, prompt: e.prompt, response: e.response,
        tokens_in: e.tokensIn, tokens_out: e.tokensOut, cost: e.cost, latency_ms: e.latencyMs,
      });
      await localdb.executions.update(e.id, { synced: 1 });
    } catch { /* ignore */ }
  }
}

export async function syncNow() {
  await pushToBackend();
}
