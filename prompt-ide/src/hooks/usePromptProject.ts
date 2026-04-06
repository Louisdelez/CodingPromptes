import { useState, useCallback, useEffect, useRef } from 'react';
import { v4 as uuid } from 'uuid';
import { localdb, startSync, type LocalProject } from '../lib/localdb';
import type { PromptProject, PromptBlock, BlockType, Workspace, CustomFramework } from '../lib/types';
import { WORKSPACE_COLORS } from '../lib/types';
import { getLang } from '../lib/i18n';

function localToProject(lp: LocalProject): PromptProject {
  return {
    id: lp.id, name: lp.name, workspaceId: lp.workspaceId,
    blocks: JSON.parse(lp.blocksJson || '[]'),
    variables: JSON.parse(lp.variablesJson || '{}'),
    tags: JSON.parse(lp.tagsJson || '[]'),
    framework: lp.framework,
    createdAt: lp.createdAt, updatedAt: lp.updatedAt,
  };
}

function createDefaultPrompt(workspaceId?: string): PromptProject {
  const lang = getLang();
  return {
    id: uuid(), name: lang === 'en' ? 'New prompt' : 'Nouveau prompt',
    workspaceId, blocks: [
      { id: uuid(), type: 'role' as BlockType, content: '', enabled: true },
      { id: uuid(), type: 'context' as BlockType, content: '', enabled: true },
      { id: uuid(), type: 'task' as BlockType, content: '', enabled: true },
    ],
    variables: {}, createdAt: Date.now(), updatedAt: Date.now(), tags: [],
  };
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function usePromptProject(_userId: string) {
  const [project, setProject] = useState<PromptProject>(() => createDefaultPrompt());
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'syncing' | 'synced'>('idle');
  const [libraryRefreshKey, setLibraryRefreshKey] = useState(0);
  const triggerLibraryRefresh = useCallback(() => setLibraryRefreshKey((k) => k + 1), []);
  const saveTimeout = useRef<ReturnType<typeof setTimeout>>(undefined);
  const statusTimeout = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Start sync engine on mount + listen to sync events
  useEffect(() => {
    startSync();
    // Poll dirty count to show sync status
    const interval = setInterval(async () => {
      const dirtyCount = await localdb.projects.where('dirty').equals(1).count();
      if (dirtyCount > 0) {
        setSaveStatus('syncing');
      } else if (saveStatus === 'syncing') {
        setSaveStatus('synced');
        if (statusTimeout.current) clearTimeout(statusTimeout.current);
        statusTimeout.current = setTimeout(() => setSaveStatus('idle'), 2000);
      }
    }, 3500);
    return () => clearInterval(interval);
  }, [saveStatus]);

  // --- Save to local DB (instant) ---
  const saveProject = useCallback(async (p: PromptProject) => {
    setSaveStatus('saving');
    const now = Date.now();
    await localdb.projects.put({
      id: p.id, name: p.name, workspaceId: p.workspaceId,
      blocksJson: JSON.stringify(p.blocks), variablesJson: JSON.stringify(p.variables),
      framework: p.framework, tagsJson: JSON.stringify(p.tags ?? []),
      createdAt: p.createdAt, updatedAt: now, dirty: 1,
    });
    setSaveStatus('saved');
    if (statusTimeout.current) clearTimeout(statusTimeout.current);
    statusTimeout.current = setTimeout(() => setSaveStatus('idle'), 1500);
  }, []);

  const updateProject = useCallback(
    (updater: (prev: PromptProject) => PromptProject) => {
      setProject((prev) => {
        const next = updater(prev);
        if (saveTimeout.current) clearTimeout(saveTimeout.current);
        saveTimeout.current = setTimeout(() => saveProject(next), 300);
        return next;
      });
    },
    [saveProject]
  );

  // --- Blocks ---
  const addBlock = useCallback((type: BlockType) => {
    updateProject((p) => ({ ...p, blocks: [...p.blocks, { id: uuid(), type, content: '', enabled: true }] }));
  }, [updateProject]);

  const removeBlock = useCallback((blockId: string) => {
    updateProject((p) => ({ ...p, blocks: p.blocks.filter((b) => b.id !== blockId) }));
  }, [updateProject]);

  const updateBlock = useCallback((blockId: string, changes: Partial<PromptBlock>) => {
    updateProject((p) => ({ ...p, blocks: p.blocks.map((b) => (b.id === blockId ? { ...b, ...changes } : b)) }));
  }, [updateProject]);

  const toggleBlock = useCallback((blockId: string) => {
    updateProject((p) => ({ ...p, blocks: p.blocks.map((b) => (b.id === blockId ? { ...b, enabled: !b.enabled } : b)) }));
  }, [updateProject]);

  const reorderBlocks = useCallback((newBlocks: PromptBlock[]) => {
    updateProject((p) => ({ ...p, blocks: newBlocks }));
  }, [updateProject]);

  const setVariable = useCallback((key: string, value: string) => {
    updateProject((p) => ({ ...p, variables: { ...p.variables, [key]: value } }));
  }, [updateProject]);

  // --- Load project ---
  const loadProject = useCallback(async (id: string) => {
    const lp = await localdb.projects.get(id);
    if (lp) setProject(localToProject(lp));
  }, []);

  // --- New project (instant) ---
  const newProject = useCallback((workspaceId?: string) => {
    const p = createDefaultPrompt(workspaceId);
    setProject(p);
    // Write to local DB immediately (< 1ms)
    localdb.projects.put({
      id: p.id, name: p.name, workspaceId: p.workspaceId,
      blocksJson: JSON.stringify(p.blocks), variablesJson: JSON.stringify(p.variables),
      framework: undefined, tagsJson: '[]',
      createdAt: p.createdAt, updatedAt: p.updatedAt, dirty: 1,
    }).then(() => triggerLibraryRefresh());
  }, [triggerLibraryRefresh]);

  const movePromptToWorkspace = useCallback((workspaceId: string | undefined) => {
    updateProject((p) => ({ ...p, workspaceId }));
  }, [updateProject]);

  const loadFramework = useCallback((frameworkId: string, blocks: Omit<PromptBlock, 'id'>[]) => {
    updateProject((p) => ({ ...p, framework: frameworkId, blocks: blocks.map((b) => ({ ...b, id: uuid() })) }));
  }, [updateProject]);

  // --- Workspaces (instant) ---
  const createWorkspace = useCallback(async (name: string, color?: string): Promise<Workspace> => {
    const now = Date.now();
    const ws: Workspace = {
      id: uuid(), name, description: '',
      color: color ?? WORKSPACE_COLORS[Math.floor(Math.random() * WORKSPACE_COLORS.length)],
      createdAt: now, updatedAt: now,
    };
    await localdb.workspaces.put({ ...ws, dirty: 1 });
    triggerLibraryRefresh();
    return ws;
  }, [triggerLibraryRefresh]);

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const updateWorkspace = useCallback(async (_id: string, _changes: Partial<Workspace>) => {}, []);

  const deleteWorkspace = useCallback(async (id: string) => {
    await localdb.workspaces.delete(id);
    // Move projects out of this workspace
    const inWs = await localdb.projects.where('workspaceId').equals(id).toArray();
    for (const p of inWs) {
      await localdb.projects.update(p.id, { workspaceId: undefined, dirty: 1 });
    }
    triggerLibraryRefresh();
  }, [triggerLibraryRefresh]);

  // --- Versions (instant) ---
  const saveVersion = useCallback(async (label: string) => {
    await localdb.versions.add({
      id: uuid(), projectId: project.id,
      blocksJson: JSON.stringify(project.blocks),
      variablesJson: JSON.stringify(project.variables),
      label, createdAt: Date.now(), dirty: 1,
    });
  }, [project]);

  // --- Frameworks (instant) ---
  const createFramework = useCallback(async (name: string, description: string, blocks: Omit<PromptBlock, 'id'>[]): Promise<CustomFramework> => {
    const now = Date.now();
    const fw: CustomFramework = { id: uuid(), name, description, blocks, createdAt: now, updatedAt: now };
    await localdb.frameworks.put({
      id: fw.id, name, description, blocksJson: JSON.stringify(blocks),
      createdAt: now, updatedAt: now, dirty: 1,
    });
    return fw;
  }, []);

  const updateFramework = useCallback(async (id: string, changes: Partial<CustomFramework>) => {
    const existing = await localdb.frameworks.get(id);
    if (existing) {
      await localdb.frameworks.update(id, {
        ...(changes.name !== undefined ? { name: changes.name } : {}),
        ...(changes.description !== undefined ? { description: changes.description } : {}),
        ...(changes.blocks !== undefined ? { blocksJson: JSON.stringify(changes.blocks) } : {}),
        updatedAt: Date.now(), dirty: 1,
      });
    }
  }, []);

  const deleteFramework = useCallback(async (id: string) => {
    await localdb.frameworks.delete(id);
  }, []);

  const saveCurrentAsFramework = useCallback(async (name: string, description: string): Promise<CustomFramework> => {
    const blocks: Omit<PromptBlock, 'id'>[] = project.blocks.map((b) => ({ type: b.type, content: b.content, enabled: b.enabled }));
    return createFramework(name, description, blocks);
  }, [project.blocks, createFramework]);

  // --- Tags ---
  const addTag = useCallback((tag: string) => {
    updateProject((p) => ({ ...p, tags: [...(p.tags || []), tag].filter((v, i, a) => a.indexOf(v) === i) }));
  }, [updateProject]);

  const removeTag = useCallback((tag: string) => {
    updateProject((p) => ({ ...p, tags: (p.tags || []).filter((t) => t !== tag) }));
  }, [updateProject]);

  // --- Load most recent on mount ---
  useEffect(() => {
    localdb.projects.orderBy('updatedAt').reverse().first().then((lp) => {
      if (lp) {
        setProject(localToProject(lp));
      } else {
        const p = createDefaultPrompt();
        setProject(p);
        localdb.projects.put({
          id: p.id, name: p.name, workspaceId: p.workspaceId,
          blocksJson: JSON.stringify(p.blocks), variablesJson: JSON.stringify(p.variables),
          framework: undefined, tagsJson: '[]',
          createdAt: p.createdAt, updatedAt: p.updatedAt, dirty: 1,
        });
      }
    });
  }, []);

  return {
    project, saveStatus, libraryRefreshKey,
    addBlock, removeBlock, updateBlock, toggleBlock, reorderBlocks, setVariable,
    loadProject, newProject, movePromptToWorkspace, loadFramework,
    saveVersion, updateProject,
    createWorkspace, updateWorkspace, deleteWorkspace,
    createFramework, updateFramework, deleteFramework, saveCurrentAsFramework,
    addTag, removeTag,
  };
}
