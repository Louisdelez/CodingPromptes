import Dexie, { type EntityTable } from 'dexie';

// Local-first database — source of truth for UI, synced to backend in background

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
  dirty: number; // 1 = needs sync, 0 = synced
}

export interface LocalWorkspace {
  id: string;
  name: string;
  description: string;
  color: string;
  createdAt: number;
  updatedAt: number;
  dirty: number;
}

export interface LocalVersion {
  id: string;
  projectId: string;
  blocksJson: string;
  variablesJson: string;
  label: string;
  createdAt: number;
  dirty: number;
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
  dirty: number;
}

export interface LocalFramework {
  id: string;
  name: string;
  description: string;
  blocksJson: string;
  createdAt: number;
  updatedAt: number;
  dirty: number;
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
      projects: 'id, workspaceId, updatedAt, dirty',
      workspaces: 'id, updatedAt, dirty',
      versions: 'id, projectId, createdAt, dirty',
      executions: 'id, projectId, createdAt, dirty',
      frameworks: 'id, updatedAt, dirty',
    });
  }
}

export const localdb = new InkwellLocalDB();

// --- Sync engine ---

import * as backend from './backend';

let syncInterval: ReturnType<typeof setInterval> | null = null;
let syncing = false;

export function startSync() {
  if (syncInterval) return;
  // Sync every 3 seconds
  syncInterval = setInterval(syncAll, 3000);
  // Initial sync immediately
  syncAll();
}

export function stopSync() {
  if (syncInterval) {
    clearInterval(syncInterval);
    syncInterval = null;
  }
}

async function syncAll() {
  if (syncing || !backend.getToken()) return;
  syncing = true;
  try {
    await pushDirty();
    await pullFromBackend();
  } catch {
    // Silently ignore sync errors — we're offline or server is down
  }
  syncing = false;
}

// Push local dirty records to backend
async function pushDirty() {
  // Push dirty workspaces
  const dirtyWs = await localdb.workspaces.where('dirty').equals(1).toArray();
  for (const ws of dirtyWs) {
    try {
      await backend.createWorkspace({ name: ws.name, color: ws.color, description: ws.description });
      await localdb.workspaces.update(ws.id, { dirty: 0 });
    } catch {
      // Might already exist, try update
      try {
        await backend.updateWorkspace(ws.id, { name: ws.name, color: ws.color });
        await localdb.workspaces.update(ws.id, { dirty: 0 });
      } catch { /* ignore */ }
    }
  }

  // Push dirty projects
  const dirtyPj = await localdb.projects.where('dirty').equals(1).toArray();
  for (const p of dirtyPj) {
    try {
      await backend.updateProject(p.id, {
        name: p.name, blocks_json: p.blocksJson, variables_json: p.variablesJson,
        workspace_id: p.workspaceId ?? null, framework: p.framework ?? null,
        tags_json: p.tagsJson,
      });
      await localdb.projects.update(p.id, { dirty: 0 });
    } catch {
      try {
        await backend.createProject({
          id: p.id, name: p.name, blocks_json: p.blocksJson,
          variables_json: p.variablesJson, workspace_id: p.workspaceId ?? null,
          framework: p.framework ?? null, tags_json: p.tagsJson,
        });
        await localdb.projects.update(p.id, { dirty: 0 });
      } catch { /* ignore */ }
    }
  }

  // Push dirty versions
  const dirtyVer = await localdb.versions.where('dirty').equals(1).toArray();
  for (const v of dirtyVer) {
    try {
      await backend.createVersion(v.projectId, {
        blocks_json: v.blocksJson, variables_json: v.variablesJson, label: v.label,
      });
      await localdb.versions.update(v.id, { dirty: 0 });
    } catch { /* ignore */ }
  }

  // Push dirty executions
  const dirtyEx = await localdb.executions.where('dirty').equals(1).toArray();
  for (const e of dirtyEx) {
    try {
      await backend.createExecution(e.projectId, {
        model: e.model, provider: e.provider, prompt: e.prompt, response: e.response,
        tokens_in: e.tokensIn, tokens_out: e.tokensOut, cost: e.cost, latency_ms: e.latencyMs,
      });
      await localdb.executions.update(e.id, { dirty: 0 });
    } catch { /* ignore */ }
  }

  // Push dirty frameworks
  const dirtyFw = await localdb.frameworks.where('dirty').equals(1).toArray();
  for (const f of dirtyFw) {
    try {
      await backend.createFramework({
        name: f.name, description: f.description, blocks_json: f.blocksJson,
      });
      await localdb.frameworks.update(f.id, { dirty: 0 });
    } catch { /* ignore */ }
  }
}

// Pull from backend and merge into local
async function pullFromBackend() {
  try {
    const [bws, bpj, bfw] = await Promise.all([
      backend.listWorkspaces(),
      backend.listProjects(),
      backend.listFrameworks(),
    ]);

    // Merge workspaces
    for (const bw of bws) {
      const local = await localdb.workspaces.get(bw.id);
      if (!local || (local.dirty === 0 && bw.updated_at > local.updatedAt)) {
        await localdb.workspaces.put({
          id: bw.id, name: bw.name, description: bw.description, color: bw.color,
          createdAt: bw.created_at, updatedAt: bw.updated_at, dirty: 0,
        });
      }
    }

    // Merge projects
    for (const bp of bpj) {
      const local = await localdb.projects.get(bp.id);
      if (!local || (local.dirty === 0 && bp.updated_at > local.updatedAt)) {
        await localdb.projects.put({
          id: bp.id, name: bp.name, workspaceId: bp.workspace_id ?? undefined,
          blocksJson: bp.blocks_json, variablesJson: bp.variables_json,
          framework: bp.framework ?? undefined, tagsJson: bp.tags_json,
          createdAt: bp.created_at, updatedAt: bp.updated_at, dirty: 0,
        });
      }
    }

    // Merge frameworks
    for (const bf of bfw) {
      const local = await localdb.frameworks.get(bf.id);
      if (!local || (local.dirty === 0 && bf.updated_at > local.updatedAt)) {
        await localdb.frameworks.put({
          id: bf.id, name: bf.name, description: bf.description,
          blocksJson: bf.blocks_json, createdAt: bf.created_at, updatedAt: bf.updated_at, dirty: 0,
        });
      }
    }
  } catch { /* offline */ }
}

// Force a full sync now
export async function syncNow() {
  await syncAll();
}
