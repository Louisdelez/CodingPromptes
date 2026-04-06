import { useState, useCallback, useEffect, useRef } from 'react';
import { v4 as uuid } from 'uuid';
import { db } from '../lib/db';
import type { PromptProject, PromptBlock, BlockType, Workspace, CustomFramework } from '../lib/types';
import { WORKSPACE_COLORS } from '../lib/types';
import { getLang } from '../lib/i18n';

function createDefaultPrompt(userId: string, workspaceId?: string): PromptProject {
  const lang = getLang();
  return {
    id: uuid(),
    name: lang === 'en' ? 'New prompt' : 'Nouveau prompt',
    userId,
    workspaceId,
    blocks: [
      { id: uuid(), type: 'role', content: '', enabled: true },
      { id: uuid(), type: 'context', content: '', enabled: true },
      { id: uuid(), type: 'task', content: '', enabled: true },
    ],
    variables: {},
    createdAt: Date.now(),
    updatedAt: Date.now(),
    tags: [],
  };
}

export function usePromptProject(userId: string) {
  const [project, setProject] = useState<PromptProject>(() => createDefaultPrompt(userId));
  const [isSaving, setIsSaving] = useState(false);
  const saveTimeout = useRef<ReturnType<typeof setTimeout>>(undefined);

  // --- Workspace management ---

  const createWorkspace = useCallback(async (name: string, color?: string): Promise<Workspace> => {
    const ws: Workspace = {
      id: uuid(),
      name,
      description: '',
      color: color ?? WORKSPACE_COLORS[Math.floor(Math.random() * WORKSPACE_COLORS.length)],
      userId,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    await db.workspaces.add(ws);
    return ws;
  }, [userId]);

  const updateWorkspace = useCallback(async (id: string, changes: Partial<Workspace>) => {
    await db.workspaces.update(id, { ...changes, updatedAt: Date.now() });
  }, []);

  const deleteWorkspace = useCallback(async (id: string) => {
    const prompts = await db.projects.where('workspaceId').equals(id).toArray();
    for (const p of prompts) {
      await db.projects.update(p.id, { workspaceId: undefined });
    }
    await db.workspaces.delete(id);
  }, []);

  // --- Prompt management ---

  const saveProject = useCallback(async (p: PromptProject) => {
    setIsSaving(true);
    const updated = { ...p, userId, updatedAt: Date.now() };
    await db.projects.put(updated);
    setIsSaving(false);
  }, [userId]);

  const updateProject = useCallback(
    (updater: (prev: PromptProject) => PromptProject) => {
      setProject((prev) => {
        const next = updater(prev);
        if (saveTimeout.current) clearTimeout(saveTimeout.current);
        saveTimeout.current = setTimeout(() => saveProject(next), 500);
        return next;
      });
    },
    [saveProject]
  );

  const addBlock = useCallback(
    (type: BlockType) => {
      updateProject((p) => ({
        ...p,
        blocks: [...p.blocks, { id: uuid(), type, content: '', enabled: true }],
      }));
    },
    [updateProject]
  );

  const removeBlock = useCallback(
    (blockId: string) => {
      updateProject((p) => ({
        ...p,
        blocks: p.blocks.filter((b) => b.id !== blockId),
      }));
    },
    [updateProject]
  );

  const updateBlock = useCallback(
    (blockId: string, changes: Partial<PromptBlock>) => {
      updateProject((p) => ({
        ...p,
        blocks: p.blocks.map((b) => (b.id === blockId ? { ...b, ...changes } : b)),
      }));
    },
    [updateProject]
  );

  const toggleBlock = useCallback(
    (blockId: string) => {
      updateProject((p) => ({
        ...p,
        blocks: p.blocks.map((b) => (b.id === blockId ? { ...b, enabled: !b.enabled } : b)),
      }));
    },
    [updateProject]
  );

  const reorderBlocks = useCallback(
    (newBlocks: PromptBlock[]) => {
      updateProject((p) => ({ ...p, blocks: newBlocks }));
    },
    [updateProject]
  );

  const setVariable = useCallback(
    (key: string, value: string) => {
      updateProject((p) => ({
        ...p,
        variables: { ...p.variables, [key]: value },
      }));
    },
    [updateProject]
  );

  const loadProject = useCallback(async (id: string) => {
    const p = await db.projects.get(id);
    if (p) setProject(p);
  }, []);

  const newProject = useCallback((workspaceId?: string) => {
    const p = createDefaultPrompt(userId, workspaceId);
    setProject(p);
  }, [userId]);

  const movePromptToWorkspace = useCallback(
    (workspaceId: string | undefined) => {
      updateProject((p) => ({ ...p, workspaceId }));
    },
    [updateProject]
  );

  const loadFramework = useCallback(
    (frameworkId: string, blocks: Omit<PromptBlock, 'id'>[]) => {
      updateProject((p) => ({
        ...p,
        framework: frameworkId,
        blocks: blocks.map((b) => ({ ...b, id: uuid() })),
      }));
    },
    [updateProject]
  );

  const saveVersion = useCallback(
    async (label: string) => {
      await db.versions.add({
        id: uuid(),
        projectId: project.id,
        blocks: JSON.parse(JSON.stringify(project.blocks)),
        variables: { ...project.variables },
        label,
        createdAt: Date.now(),
      });
    },
    [project]
  );

  // --- Custom framework management ---

  const createFramework = useCallback(async (name: string, description: string, blocks: Omit<PromptBlock, 'id'>[]): Promise<CustomFramework> => {
    const fw: CustomFramework = {
      id: uuid(),
      name,
      description,
      blocks,
      userId,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    await db.frameworks.add(fw);
    return fw;
  }, [userId]);

  const updateFramework = useCallback(async (id: string, changes: Partial<CustomFramework>) => {
    await db.frameworks.update(id, { ...changes, updatedAt: Date.now() });
  }, []);

  const deleteFramework = useCallback(async (id: string) => {
    await db.frameworks.delete(id);
  }, []);

  const saveCurrentAsFramework = useCallback(async (name: string, description: string): Promise<CustomFramework> => {
    const blocks: Omit<PromptBlock, 'id'>[] = project.blocks.map((b) => ({
      type: b.type,
      content: b.content,
      enabled: b.enabled,
    }));
    return createFramework(name, description, blocks);
  }, [project.blocks, createFramework]);

  // Load most recent prompt for this user on mount
  useEffect(() => {
    db.projects
      .where('userId')
      .equals(userId)
      .reverse()
      .sortBy('updatedAt')
      .then((all) => {
        if (all.length > 0) {
          setProject(all[0]);
        } else {
          setProject(createDefaultPrompt(userId));
        }
      });
  }, [userId]);

  return {
    project,
    isSaving,
    addBlock,
    removeBlock,
    updateBlock,
    toggleBlock,
    reorderBlocks,
    setVariable,
    loadProject,
    newProject,
    movePromptToWorkspace,
    loadFramework,
    saveVersion,
    updateProject,
    createWorkspace,
    updateWorkspace,
    deleteWorkspace,
    createFramework,
    updateFramework,
    deleteFramework,
    saveCurrentAsFramework,
  };
}
