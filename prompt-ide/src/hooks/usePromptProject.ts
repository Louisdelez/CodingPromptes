import { useState, useCallback, useEffect, useRef } from 'react';
import { v4 as uuid } from 'uuid';
import { localdb, type LocalProject } from '../lib/localdb';
import type { PromptProject, PromptBlock, BlockType, Workspace, CustomFramework } from '../lib/types';
import { WORKSPACE_COLORS } from '../lib/types';
import { getLang } from '../lib/i18n';

function localToProject(lp: LocalProject): PromptProject {
  return {
    id: lp.id, name: lp.name, workspaceId: lp.workspaceId,
    blocks: JSON.parse(lp.blocksJson || '[]'),
    variables: JSON.parse(lp.variablesJson || '{}'),
    tags: JSON.parse(lp.tagsJson || '[]'),
    framework: lp.framework, createdAt: lp.createdAt, updatedAt: lp.updatedAt,
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
  // Empty shell — will be replaced by useEffect loading from IndexedDB
  const [project, setProject] = useState<PromptProject>({
    id: '', name: '', blocks: [], variables: {}, tags: [],
    createdAt: 0, updatedAt: 0,
  });
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved'>('idle');
  const [libraryRefreshKey, setLibraryRefreshKey] = useState(0);
  const triggerLibraryRefresh = useCallback(() => setLibraryRefreshKey((k) => k + 1), []);
  const saveTimeout = useRef<ReturnType<typeof setTimeout>>(undefined);
  const statusTimeout = useRef<ReturnType<typeof setTimeout>>(undefined);

  const flashSaved = useCallback(() => {
    setSaveStatus('saving');
    setTimeout(() => {
      setSaveStatus('saved');
      if (statusTimeout.current) clearTimeout(statusTimeout.current);
      statusTimeout.current = setTimeout(() => setSaveStatus('idle'), 1500);
    }, 50);
  }, []);

  // Save to local IndexedDB (instant, < 1ms)
  const saveProject = useCallback(async (p: PromptProject) => {
    setSaveStatus('saving');
    await localdb.projects.put({
      id: p.id, name: p.name, workspaceId: p.workspaceId,
      blocksJson: JSON.stringify(p.blocks), variablesJson: JSON.stringify(p.variables),
      framework: p.framework, tagsJson: JSON.stringify(p.tags ?? []),
      createdAt: p.createdAt, updatedAt: Date.now(),
    });
    setSaveStatus('saved');
    if (statusTimeout.current) clearTimeout(statusTimeout.current);
    statusTimeout.current = setTimeout(() => setSaveStatus('idle'), 1500);
  }, []);

  const updateProject = useCallback((updater: (prev: PromptProject) => PromptProject) => {
    setProject((prev) => {
      const next = updater(prev);
      if (saveTimeout.current) clearTimeout(saveTimeout.current);
      saveTimeout.current = setTimeout(() => saveProject(next), 300);
      return next;
    });
  }, [saveProject]);

  const addBlock = useCallback((type: BlockType) => {
    updateProject((p) => ({ ...p, blocks: [...p.blocks, { id: uuid(), type, content: '', enabled: true }] }));
  }, [updateProject]);

  const removeBlock = useCallback((blockId: string) => {
    updateProject((p) => ({ ...p, blocks: p.blocks.filter((b: PromptBlock) => b.id !== blockId) }));
  }, [updateProject]);

  const updateBlock = useCallback((blockId: string, changes: Partial<PromptBlock>) => {
    updateProject((p) => ({ ...p, blocks: p.blocks.map((b: PromptBlock) => (b.id === blockId ? { ...b, ...changes } : b)) }));
  }, [updateProject]);

  const toggleBlock = useCallback((blockId: string) => {
    updateProject((p) => ({ ...p, blocks: p.blocks.map((b: PromptBlock) => (b.id === blockId ? { ...b, enabled: !b.enabled } : b)) }));
  }, [updateProject]);

  const reorderBlocks = useCallback((newBlocks: PromptBlock[]) => {
    updateProject((p) => ({ ...p, blocks: newBlocks }));
  }, [updateProject]);

  const setVariable = useCallback((key: string, value: string) => {
    updateProject((p) => ({ ...p, variables: { ...p.variables, [key]: value } }));
  }, [updateProject]);

  const loadProject = useCallback(async (id: string) => {
    const lp = await localdb.projects.get(id);
    if (lp) setProject(localToProject(lp));
  }, []);

  const newProject = useCallback((workspaceId?: string) => {
    const p = createDefaultPrompt(workspaceId);
    setProject(p);
    localdb.projects.put({
      id: p.id, name: p.name, workspaceId: p.workspaceId,
      blocksJson: JSON.stringify(p.blocks), variablesJson: JSON.stringify(p.variables),
      framework: undefined, tagsJson: '[]', createdAt: p.createdAt, updatedAt: p.updatedAt,
    }).then(() => { triggerLibraryRefresh(); flashSaved(); });
  }, [triggerLibraryRefresh, flashSaved]);

  const deleteProject = useCallback(async (id: string) => {
    await localdb.projects.delete(id);
    triggerLibraryRefresh();
    flashSaved();
  }, [triggerLibraryRefresh, flashSaved]);

  const movePromptToWorkspace = useCallback((workspaceId: string | undefined) => {
    updateProject((p) => ({ ...p, workspaceId }));
  }, [updateProject]);

  const loadFramework = useCallback((frameworkId: string, blocks: Omit<PromptBlock, 'id'>[]) => {
    updateProject((p) => ({ ...p, framework: frameworkId, blocks: blocks.map((b) => ({ ...b, id: uuid() })) }));
  }, [updateProject]);

  const updateWorkspace = useCallback(async (id: string, changes: Partial<Workspace>) => {
    const existing = await localdb.workspaces.get(id);
    if (existing) {
      await localdb.workspaces.update(id, {
        ...(changes.name !== undefined ? { name: changes.name } : {}),
        ...(changes.description !== undefined ? { description: changes.description } : {}),
        ...(changes.color !== undefined ? { color: changes.color } : {}),
        ...(changes.constitution !== undefined ? { constitution: changes.constitution } : {}),
        updatedAt: Date.now(),
      });
      triggerLibraryRefresh();
      flashSaved();
    }
  }, [triggerLibraryRefresh, flashSaved]);

  const createWorkspace = useCallback(async (name: string, color?: string): Promise<Workspace> => {
    const now = Date.now();
    const ws: Workspace = {
      id: uuid(), name, description: '',
      color: color ?? WORKSPACE_COLORS[Math.floor(Math.random() * WORKSPACE_COLORS.length)],
      createdAt: now, updatedAt: now,
    };
    await localdb.workspaces.put({ ...ws });
    triggerLibraryRefresh();
    flashSaved();
    return ws;
  }, [triggerLibraryRefresh, flashSaved]);

  const deleteWorkspace = useCallback(async (id: string) => {
    await localdb.workspaces.delete(id);
    const inWs = await localdb.projects.where('workspaceId').equals(id).toArray();
    for (const p of inWs) {
      await localdb.projects.update(p.id, { workspaceId: undefined });
    }
    triggerLibraryRefresh();
    flashSaved();
  }, [triggerLibraryRefresh, flashSaved]);

  const saveVersion = useCallback(async (label: string) => {
    await localdb.versions.add({
      id: uuid(), projectId: project.id,
      blocksJson: JSON.stringify(project.blocks),
      variablesJson: JSON.stringify(project.variables),
      label, createdAt: Date.now(),
    });
  }, [project]);

  const createFramework = useCallback(async (name: string, description: string, blocks: Omit<PromptBlock, 'id'>[]): Promise<CustomFramework> => {
    const now = Date.now();
    const fw: CustomFramework = { id: uuid(), name, description, blocks, createdAt: now, updatedAt: now };
    await localdb.frameworks.put({ id: fw.id, name, description, blocksJson: JSON.stringify(blocks), createdAt: now, updatedAt: now });
    return fw;
  }, []);

  const updateFramework = useCallback(async (id: string, changes: Partial<CustomFramework>) => {
    const existing = await localdb.frameworks.get(id);
    if (existing) {
      await localdb.frameworks.update(id, {
        ...(changes.name !== undefined ? { name: changes.name } : {}),
        ...(changes.description !== undefined ? { description: changes.description } : {}),
        ...(changes.blocks !== undefined ? { blocksJson: JSON.stringify(changes.blocks) } : {}),
        updatedAt: Date.now(),
      });
    }
  }, []);

  const deleteFramework = useCallback(async (id: string) => {
    await localdb.frameworks.delete(id);
  }, []);

  const saveCurrentAsFramework = useCallback(async (name: string, description: string): Promise<CustomFramework> => {
    const blocks: Omit<PromptBlock, 'id'>[] = project.blocks.map((b: PromptBlock) => ({ type: b.type, content: b.content, enabled: b.enabled }));
    return createFramework(name, description, blocks);
  }, [project.blocks, createFramework]);

  const addTag = useCallback((tag: string) => {
    updateProject((p) => ({ ...p, tags: [...(p.tags || []), tag].filter((v: string, i: number, a: string[]) => a.indexOf(v) === i) }));
  }, [updateProject]);

  const removeTag = useCallback((tag: string) => {
    updateProject((p) => ({ ...p, tags: (p.tags || []).filter((t: string) => t !== tag) }));
  }, [updateProject]);

  // Load most recent on mount — DON'T create a default if empty
  useEffect(() => {
    localdb.projects.orderBy('updatedAt').reverse().first().then((lp) => {
      if (lp) {
        setProject(localToProject(lp));
      }
      // If empty, just keep the in-memory default — don't save to DB
      // The user will explicitly create their first prompt
    });
  }, []);

  // Get workspace constitution for SDD chaining
  const [workspaceConstitution, setWorkspaceConstitution] = useState<string>('');
  useEffect(() => {
    if (project.workspaceId) {
      localdb.workspaces.get(project.workspaceId).then(ws => {
        setWorkspaceConstitution(ws?.constitution || '');
      });
    } else {
      setWorkspaceConstitution('');
    }
  }, [project.workspaceId]);

  return {
    project, saveStatus, libraryRefreshKey, workspaceConstitution,
    addBlock, removeBlock, updateBlock, toggleBlock, reorderBlocks, setVariable,
    loadProject, newProject, deleteProject, movePromptToWorkspace, loadFramework,
    saveVersion, updateProject,
    createWorkspace, updateWorkspace, deleteWorkspace,
    createFramework, updateFramework, deleteFramework, saveCurrentAsFramework,
    addTag, removeTag,
  };
}
