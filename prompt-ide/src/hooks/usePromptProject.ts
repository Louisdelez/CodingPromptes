import { useState, useCallback, useEffect, useRef } from 'react';
import { v4 as uuid } from 'uuid';
import * as backend from '../lib/backend';
import type { PromptProject, PromptBlock, BlockType, Workspace, CustomFramework } from '../lib/types';
import { WORKSPACE_COLORS } from '../lib/types';
import { getLang } from '../lib/i18n';

function backendProjectToLocal(bp: backend.BackendProject): PromptProject {
  return {
    id: bp.id,
    name: bp.name,
    userId: bp.user_id,
    workspaceId: bp.workspace_id ?? undefined,
    blocks: JSON.parse(bp.blocks_json),
    variables: JSON.parse(bp.variables_json),
    tags: JSON.parse(bp.tags_json || '[]'),
    createdAt: bp.created_at,
    updatedAt: bp.updated_at,
    framework: bp.framework ?? undefined,
  };
}

function createDefaultPrompt(workspaceId?: string): PromptProject {
  const lang = getLang();
  return {
    id: uuid(),
    name: lang === 'en' ? 'New prompt' : 'Nouveau prompt',
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

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function usePromptProject(_userId: string) {
  const [project, setProject] = useState<PromptProject>(() => createDefaultPrompt());
  const [isSaving, setIsSaving] = useState(false);
  const saveTimeout = useRef<ReturnType<typeof setTimeout>>(undefined);

  // --- Workspace management ---

  const createWorkspace = useCallback(async (name: string, color?: string): Promise<Workspace> => {
    const bw = await backend.createWorkspace({
      name,
      color: color ?? WORKSPACE_COLORS[Math.floor(Math.random() * WORKSPACE_COLORS.length)],
    });
    return {
      id: bw.id,
      name: bw.name,
      description: bw.description,
      color: bw.color,
      userId: bw.user_id,
      createdAt: bw.created_at,
      updatedAt: bw.updated_at,
    };
  }, []);

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const updateWorkspace = useCallback(async (_id: string, _changes: Partial<Workspace>) => {
    // No update endpoint yet — the Library will reload from backend
  }, []);

  const deleteWorkspace = useCallback(async (id: string) => {
    await backend.deleteWorkspace(id);
  }, []);

  // --- Prompt management ---

  const saveProject = useCallback(async (p: PromptProject) => {
    setIsSaving(true);
    try {
      await backend.updateProject(p.id, {
        name: p.name,
        blocks_json: JSON.stringify(p.blocks),
        variables_json: JSON.stringify(p.variables),
        workspace_id: p.workspaceId ?? null,
        framework: p.framework ?? null,
        tags_json: JSON.stringify(p.tags ?? []),
      });
    } catch (err) {
      // If the project doesn't exist yet (first save), create it
      const msg = err instanceof Error ? err.message : '';
      if (msg.includes('404') || msg.includes('not found') || msg.includes('Not Found')) {
        await backend.createProject({
          id: p.id,
          name: p.name,
          blocks_json: JSON.stringify(p.blocks),
          variables_json: JSON.stringify(p.variables),
          workspace_id: p.workspaceId ?? null,
          framework: p.framework ?? null,
          tags_json: JSON.stringify(p.tags ?? []),
        });
      }
    }
    setIsSaving(false);
  }, []);

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
    try {
      const all = await backend.listProjects();
      const bp = all.find((p) => p.id === id);
      if (bp) setProject(backendProjectToLocal(bp));
    } catch {
      // ignore
    }
  }, []);

  const newProject = useCallback(async (workspaceId?: string) => {
    const p = createDefaultPrompt(workspaceId);
    setProject(p);
    // Create on backend immediately
    try {
      await backend.createProject({
        id: p.id,
        name: p.name,
        blocks_json: JSON.stringify(p.blocks),
        variables_json: JSON.stringify(p.variables),
        workspace_id: workspaceId ?? null,
      });
    } catch {
      // ignore
    }
  }, []);

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
      await backend.createVersion(project.id, {
        blocks_json: JSON.stringify(project.blocks),
        variables_json: JSON.stringify(project.variables),
        label,
      });
    },
    [project]
  );

  // --- Custom framework management ---

  const createFramework = useCallback(async (name: string, description: string, blocks: Omit<PromptBlock, 'id'>[]): Promise<CustomFramework> => {
    const bf = await backend.createFramework({
      name,
      description,
      blocks_json: JSON.stringify(blocks),
    });
    return {
      id: bf.id,
      name: bf.name,
      description: bf.description,
      blocks: JSON.parse(bf.blocks_json),
      userId: bf.user_id,
      createdAt: bf.created_at,
      updatedAt: bf.updated_at,
    };
  }, []);

  const updateFramework = useCallback(async (id: string, changes: Partial<CustomFramework>) => {
    const data: Parameters<typeof backend.updateFramework>[1] = {};
    if (changes.name !== undefined) data.name = changes.name;
    if (changes.description !== undefined) data.description = changes.description;
    if (changes.blocks !== undefined) data.blocks_json = JSON.stringify(changes.blocks);
    await backend.updateFramework(id, data);
  }, []);

  const deleteFramework = useCallback(async (id: string) => {
    await backend.deleteFramework(id);
  }, []);

  const saveCurrentAsFramework = useCallback(async (name: string, description: string): Promise<CustomFramework> => {
    const blocks: Omit<PromptBlock, 'id'>[] = project.blocks.map((b) => ({
      type: b.type,
      content: b.content,
      enabled: b.enabled,
    }));
    return createFramework(name, description, blocks);
  }, [project.blocks, createFramework]);

  // Load most recent prompt on mount
  useEffect(() => {
    backend.listProjects()
      .then((all) => {
        if (all.length > 0) {
          // Sort by updated_at descending
          all.sort((a, b) => b.updated_at - a.updated_at);
          setProject(backendProjectToLocal(all[0]));
        } else {
          const p = createDefaultPrompt();
          setProject(p);
          // Create on backend
          backend.createProject({
            id: p.id,
            name: p.name,
            blocks_json: JSON.stringify(p.blocks),
            variables_json: JSON.stringify(p.variables),
          }).catch(() => {});
        }
      })
      .catch(() => {
        setProject(createDefaultPrompt());
      });
  }, []);

  const addTag = useCallback((tag: string) => {
    updateProject((p) => ({
      ...p,
      tags: [...(p.tags || []), tag].filter((v, i, a) => a.indexOf(v) === i),
    }));
  }, [updateProject]);

  const removeTag = useCallback((tag: string) => {
    updateProject((p) => ({
      ...p,
      tags: (p.tags || []).filter((t) => t !== tag),
    }));
  }, [updateProject]);

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
    addTag,
    removeTag,
  };
}
