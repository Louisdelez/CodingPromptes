import { useState, useEffect, useRef, useCallback } from 'react';
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  FolderPlus,
  Plus,
  Search,
  Trash2,
  Clock,
  Tag,
  ChevronRight,
  ChevronDown,
  FileText,
  MoreHorizontal,
  Pencil,
  FolderInput,
  X,
  Copy,
} from 'lucide-react';
import type { PromptProject, Workspace } from '../lib/types';
import { WORKSPACE_COLORS } from '../lib/types';
import * as backend from '../lib/backend';
import { useT } from '../lib/i18n';

interface LibraryProps {
  currentProjectId: string;
  onLoadProject: (id: string) => void;
  onNewProject: (workspaceId?: string) => void;
  onCreateWorkspace: (name: string, color?: string) => Promise<Workspace>;
  onUpdateWorkspace?: (id: string, changes: Partial<Workspace>) => Promise<void>;
  onDeleteWorkspace: (id: string) => Promise<void>;
  onMovePrompt: (workspaceId: string | undefined) => void;
  currentWorkspaceId?: string;
}

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

function backendWorkspaceToLocal(bw: backend.BackendWorkspace): Workspace {
  return {
    id: bw.id,
    name: bw.name,
    description: bw.description,
    color: bw.color,
    userId: bw.user_id,
    createdAt: bw.created_at,
    updatedAt: bw.updated_at,
  };
}

function DraggablePromptItem({ prompt, children }: { prompt: PromptProject; children: React.ReactNode }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: prompt.id });
  return (
    <div ref={setNodeRef} {...attributes} {...listeners} style={{ opacity: isDragging ? 0.4 : 1, cursor: 'grab' }}>
      {children}
    </div>
  );
}

function DroppableWorkspace({ wsId, children }: { wsId: string; children: React.ReactNode }) {
  const { setNodeRef, isOver } = useDroppable({ id: wsId });
  return (
    <div ref={setNodeRef} style={{ backgroundColor: isOver ? 'var(--color-accent-hover)' : undefined, borderRadius: '4px', transition: 'background-color 0.15s' }}>
      {children}
    </div>
  );
}

export function Library({
  currentProjectId,
  onLoadProject,
  onNewProject,
  onCreateWorkspace,
  onDeleteWorkspace,
  onMovePrompt,
  currentWorkspaceId,
}: LibraryProps) {
  const t = useT();
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [projects, setProjects] = useState<PromptProject[]>([]);
  const [search, setSearch] = useState('');
  const [expandedWs, setExpandedWs] = useState<Set<string>>(new Set());
  const [creatingWs, setCreatingWs] = useState(false);
  const [newWsName, setNewWsName] = useState('');
  const [editingWsId, setEditingWsId] = useState<string | null>(null);
  const [editWsName, setEditWsName] = useState('');
  const [contextMenu, setContextMenu] = useState<{ type: 'workspace' | 'prompt'; id: string; x: number; y: number } | null>(null);
  const [colorPickerWsId, setColorPickerWsId] = useState<string | null>(null);
  const [newWsColor, setNewWsColor] = useState<string>(WORKSPACE_COLORS[0]);
  const newWsInputRef = useRef<HTMLInputElement>(null);
  const [activeDragId, setActiveDragId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 10 } })
  );

  const loadData = useCallback(async () => {
    try {
      const [bws, bpj] = await Promise.all([
        backend.listWorkspaces(),
        backend.listProjects(),
      ]);
      setWorkspaces(bws.map(backendWorkspaceToLocal).sort((a, b) => b.updatedAt - a.updatedAt));
      setProjects(bpj.map(backendProjectToLocal).sort((a, b) => b.updatedAt - a.updatedAt));
    } catch {
      // ignore
    }
  }, []);

  // Load once on mount
  useEffect(() => { loadData(); }, [loadData]);

  const handleNewProject = (wsId?: string) => {
    onNewProject(wsId);
    // Refresh after backend creates it
    setTimeout(loadData, 500);
  };

  // Close context menu on click outside
  useEffect(() => {
    const handler = () => setContextMenu(null);
    window.addEventListener('click', handler);
    return () => window.removeEventListener('click', handler);
  }, []);

  const searchLower = search.toLowerCase();
  const filtered = search
    ? projects.filter(
        (p) =>
          p.name.toLowerCase().includes(searchLower) ||
          p.tags.some((t) => t.toLowerCase().includes(searchLower)) ||
          p.blocks.some((b) => b.content.toLowerCase().includes(searchLower))
      )
    : projects;

  const filteredWorkspaces = search
    ? workspaces.filter((ws) => ws.name.toLowerCase().includes(search.toLowerCase()))
    : workspaces;

  const orphanPrompts = filtered.filter((p) => !p.workspaceId);
  const promptsByWs = (wsId: string) => filtered.filter((p) => p.workspaceId === wsId);

  const toggleExpand = (wsId: string) => {
    setExpandedWs((prev) => {
      const next = new Set(prev);
      if (next.has(wsId)) next.delete(wsId);
      else next.add(wsId);
      return next;
    });
  };

  const handleCreateWs = () => {
    if (!newWsName.trim()) return;
    const name = newWsName.trim();
    const color = newWsColor;
    // Optimistic: add to UI immediately with temp id
    const tempId = `temp-${Date.now()}`;
    const now = Date.now();
    setWorkspaces((prev) => [{ id: tempId, name, description: '', color, userId: '', createdAt: now, updatedAt: now }, ...prev]);
    setExpandedWs((prev) => new Set(prev).add(tempId));
    // Reset form immediately
    setCreatingWs(false);
    setColorPickerWsId(null);
    setNewWsName('');
    setNewWsColor(WORKSPACE_COLORS[0]);
    // Create on backend then refresh to get real id
    onCreateWorkspace(name, color).then(() => loadData());
  };

  const handleRenameWs = (id: string) => {
    if (!editWsName.trim()) return;
    const name = editWsName.trim();
    // Optimistic
    setWorkspaces((prev) => prev.map((w) => w.id === id ? { ...w, name } : w));
    setEditingWsId(null);
    setEditWsName('');
    backend.updateWorkspace(id, { name }).catch(() => {});
  };

  const handleDeletePrompt = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    // Optimistic: remove from UI immediately
    setProjects((prev) => prev.filter((p) => p.id !== id));
    // Sync backend in background
    backend.deleteProject(id).catch(() => loadData());
  };

  const handleContextMenu = (e: React.MouseEvent, type: 'workspace' | 'prompt', id: string) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ type, id, x: e.clientX, y: e.clientY });
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    setActiveDragId(null);
    if (!over || active.id === over.id) return;

    const promptId = active.id as string;
    const targetId = over.id as string;
    const newWorkspaceId = targetId === '__free__' ? undefined : targetId;

    // Update in backend
    backend.updateProject(promptId, { workspace_id: newWorkspaceId ?? null });

    // If it's the currently active prompt, update state too
    if (promptId === currentProjectId) {
      onMovePrompt(newWorkspaceId);
    }
  };

  const formatDate = (ts: number) => {
    const d = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    if (diff < 60000) return "a l'instant";
    if (diff < 3600000) return `il y a ${Math.floor(diff / 60000)}min`;
    if (diff < 86400000) return `il y a ${Math.floor(diff / 3600000)}h`;
    return d.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' });
  };

  const renderPromptItem = (p: PromptProject) => (
    <DraggablePromptItem key={p.id} prompt={p}>
      <button
        onClick={() => onLoadProject(p.id)}
        onContextMenu={(e) => handleContextMenu(e, 'prompt', p.id)}
        className={`w-full text-left pl-8 pr-3 py-2 hover:bg-[var(--color-bg-hover)] transition-colors group flex items-center gap-2 ${
          p.id === currentProjectId
            ? 'bg-[var(--color-accent)]/10 border-l-2 border-l-[var(--color-accent)]'
            : ''
        }`}
      >
        <FileText size={13} className="flex-shrink-0 text-[var(--color-text-muted)]" />
        <div className="flex-1 min-w-0">
          <div className="text-sm text-[var(--color-text-primary)] truncate">{p.name}</div>
          <div className="flex items-center gap-2 mt-0.5">
            <span className="text-[10px] text-[var(--color-text-muted)] flex items-center gap-0.5">
              <Clock size={9} />
              {formatDate(p.updatedAt)}
            </span>
            {p.framework && (
              <span className="text-[10px] px-1 py-px rounded bg-[var(--color-bg-tertiary)] text-[var(--color-accent)]">
                {p.framework.toUpperCase()}
              </span>
            )}
            {p.tags.length > 0 && p.tags.map((tag) => (
              <span key={tag} className="text-[10px] px-1 py-px rounded bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] flex items-center gap-0.5">
                <Tag size={9} />
                {tag}
              </span>
            ))}
          </div>
        </div>
        <button
          onClick={(e) => handleDeletePrompt(p.id, e)}
          className="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] hover:text-[var(--color-danger)] transition-all flex-shrink-0"
        >
          <Trash2 size={11} />
        </button>
      </button>
    </DraggablePromptItem>
  );

  const renderWorkspace = (ws: Workspace) => {
    const wsPrompts = promptsByWs(ws.id);
    const isExpanded = expandedWs.has(ws.id) || ws.id === currentWorkspaceId;
    const isEditing = editingWsId === ws.id;

    return (
      <DroppableWorkspace key={ws.id} wsId={ws.id}>
        <div className="animate-fadeIn">
          {/* Workspace header */}
          <div
            className="flex items-center gap-1.5 px-3 py-2 hover:bg-[var(--color-bg-hover)] cursor-pointer transition-colors group"
            onClick={() => toggleExpand(ws.id)}
            onContextMenu={(e) => handleContextMenu(e, 'workspace', ws.id)}
          >
            {isExpanded ? (
              <ChevronDown size={14} className="flex-shrink-0 text-[var(--color-text-muted)]" />
            ) : (
              <ChevronRight size={14} className="flex-shrink-0 text-[var(--color-text-muted)]" />
            )}
            <div
              className="w-2.5 h-2.5 rounded flex-shrink-0 cursor-pointer hover:ring-2 hover:ring-white/30 transition-all"
              style={{ backgroundColor: ws.color }}
              onClick={(e) => {
                e.stopPropagation();
                setColorPickerWsId(colorPickerWsId === ws.id ? null : ws.id);
              }}
              title={t('library.changeColor')}
            />
            {isEditing ? (
              <input
                autoFocus
                value={editWsName}
                onChange={(e) => setEditWsName(e.target.value)}
                onBlur={() => handleRenameWs(ws.id)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleRenameWs(ws.id);
                  if (e.key === 'Escape') setEditingWsId(null);
                }}
                onClick={(e) => e.stopPropagation()}
                className="flex-1 px-1 py-0 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-accent)] rounded outline-none text-[var(--color-text-primary)]"
              />
            ) : (
              <span className="flex-1 text-sm font-medium text-[var(--color-text-primary)] truncate">
                {ws.name}
              </span>
            )}
            <span className="text-[10px] text-[var(--color-text-muted)] tabular-nums">
              {wsPrompts.length}
            </span>
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleNewProject(ws.id);
              }}
              className="p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] hover:text-[var(--color-accent)] transition-all"
              title={t('library.newPromptInProject')}
            >
              <Plus size={13} />
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleContextMenu(e, 'workspace', ws.id);
              }}
              className="p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] transition-all"
            >
              <MoreHorizontal size={13} />
            </button>
          </div>

          {/* Inline color picker */}
          {colorPickerWsId === ws.id && (
            <div className="flex flex-wrap gap-1.5 px-3 py-2 bg-[var(--color-bg-tertiary)] border-b border-[var(--color-border)] animate-fadeIn">
              {WORKSPACE_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => {
                    // Optimistic: update color locally immediately
                    setWorkspaces((prev) => prev.map((w) => w.id === ws.id ? { ...w, color: c } : w));
                    setColorPickerWsId(null);
                    // Sync to backend
                    backend.updateWorkspace(ws.id, { color: c }).catch(() => {});
                  }}
                  className="w-5 h-5 rounded-full transition-transform hover:scale-125"
                  style={{
                    backgroundColor: c,
                    outline: c === ws.color ? '2px solid white' : 'none',
                    outlineOffset: '2px',
                  }}
                />
              ))}
            </div>
          )}

          {/* Workspace prompts */}
          {isExpanded && (
            <div className="border-l border-[var(--color-border)] ml-[18px]">
              {wsPrompts.length === 0 ? (
                <div className="pl-6 pr-3 py-2 text-xs text-[var(--color-text-muted)] italic">
                  {t('library.noPrompts')}
                </div>
              ) : (
                wsPrompts.map(renderPromptItem)
              )}
            </div>
          )}
        </div>
      </DroppableWorkspace>
    );
  };

  const activeDragPrompt = activeDragId ? projects.find((p) => p.id === activeDragId) : null;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-3 space-y-2 border-b border-[var(--color-border)]">
        <div className="flex items-center gap-1.5">
          <div className="relative flex-1">
            <Search size={13} className="absolute left-2 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t('library.search')}
              className="w-full pl-7 pr-3 py-1.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
            />
          </div>
          <button
            onClick={() => {
              setCreatingWs(true);
              setTimeout(() => newWsInputRef.current?.focus(), 50);
            }}
            className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-accent)] transition-colors"
            title={t('library.newWorkspace')}
          >
            <FolderPlus size={16} />
          </button>
          <button
            onClick={() => handleNewProject()}
            className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-accent)] transition-colors"
            title={t('library.newPrompt')}
          >
            <Plus size={16} />
          </button>
        </div>

        {/* New workspace input */}
        {creatingWs && (
          <div className="space-y-2 animate-fadeIn">
            <div className="flex items-center gap-1.5">
              <div
                className="w-4 h-4 rounded flex-shrink-0 cursor-pointer ring-1 ring-white/20"
                style={{ backgroundColor: newWsColor }}
                onClick={() => setColorPickerWsId('__new__')}
                title={t('library.changeColor')}
              />
              <input
                ref={newWsInputRef}
                value={newWsName}
                onChange={(e) => setNewWsName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreateWs();
                  if (e.key === 'Escape') { setCreatingWs(false); setNewWsName(''); }
                }}
                placeholder={t('library.workspaceName')}
                className="flex-1 px-2 py-1 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-accent)] rounded outline-none text-[var(--color-text-primary)]"
              />
              <button
                onClick={handleCreateWs}
                disabled={!newWsName.trim()}
                className="px-2 py-1 rounded text-xs bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 transition-colors"
              >
                {t('library.create')}
              </button>
              <button
                onClick={() => { setCreatingWs(false); setNewWsName(''); setColorPickerWsId(null); }}
                className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)]"
              >
                <X size={14} />
              </button>
            </div>
            {colorPickerWsId === '__new__' && (
              <div className="flex flex-wrap gap-1.5 p-2 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
                {WORKSPACE_COLORS.map((c) => (
                  <button
                    key={c}
                    onClick={() => { setNewWsColor(c); setColorPickerWsId(null); }}
                    className="w-5 h-5 rounded-full transition-transform hover:scale-125"
                    style={{
                      backgroundColor: c,
                      outline: c === newWsColor ? '2px solid white' : 'none',
                      outlineOffset: '2px',
                    }}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Tree */}
      <div className="flex-1 overflow-auto">
        <DndContext
          sensors={sensors}
          onDragStart={(event) => setActiveDragId(event.active.id as string)}
          onDragEnd={handleDragEnd}
          onDragCancel={() => setActiveDragId(null)}
        >
          {/* Workspaces */}
          {filteredWorkspaces.map(renderWorkspace)}

          {/* Orphan prompts (no workspace) */}
          {orphanPrompts.length > 0 && (
            <DroppableWorkspace wsId="__free__">
              <div>
                {workspaces.length > 0 && (
                  <div className="flex items-center gap-1.5 px-3 py-2 text-xs text-[var(--color-text-muted)] uppercase tracking-wider font-medium border-t border-[var(--color-border)] mt-1">
                    <FileText size={11} />
                    {t('library.freePrompts')}
                  </div>
                )}
                {orphanPrompts.map(renderPromptItem)}
              </div>
            </DroppableWorkspace>
          )}

          {/* Show free drop zone even when no orphan prompts exist, so user can drop prompts here */}
          {orphanPrompts.length === 0 && workspaces.length > 0 && (
            <DroppableWorkspace wsId="__free__">
              <div className="flex items-center gap-1.5 px-3 py-2 text-xs text-[var(--color-text-muted)] uppercase tracking-wider font-medium border-t border-[var(--color-border)] mt-1">
                <FileText size={11} />
                {t('library.freePrompts')}
              </div>
            </DroppableWorkspace>
          )}

          {filteredWorkspaces.length === 0 && orphanPrompts.length === 0 && (
            <div className="p-4 text-center">
              <p className="text-sm text-[var(--color-text-muted)]">
                {search ? t('library.noResults') : t('library.empty')}
              </p>
              {!search && (
                <p className="text-xs text-[var(--color-text-muted)] mt-2">
                  {t('library.emptyHint')}
                </p>
              )}
            </div>
          )}

          <DragOverlay>
            {activeDragPrompt ? (
              <div className="px-3 py-2 bg-[var(--color-bg-secondary)] border border-[var(--color-accent)] rounded shadow-lg text-sm text-[var(--color-text-primary)] flex items-center gap-2">
                <FileText size={13} />
                {activeDragPrompt.name}
              </div>
            ) : null}
          </DragOverlay>
        </DndContext>
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <div
          className="fixed z-50 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl py-1 min-w-[180px] animate-fadeIn"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(e) => e.stopPropagation()}
        >
          {contextMenu.type === 'workspace' && (
            <>
              <button
                onClick={() => {
                  handleNewProject(contextMenu.id);
                  setContextMenu(null);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <Plus size={13} /> {t('library.newPromptHere')}
              </button>
              <button
                onClick={() => {
                  const ws = workspaces.find((w) => w.id === contextMenu.id);
                  if (ws) {
                    setEditingWsId(ws.id);
                    setEditWsName(ws.name);
                  }
                  setContextMenu(null);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <Pencil size={13} /> {t('library.rename')}
              </button>
              <button
                onClick={() => {
                  setColorPickerWsId(contextMenu.id);
                  setContextMenu(null);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <div className="w-3 h-3 rounded-full border border-white/30" style={{ backgroundColor: workspaces.find((w) => w.id === contextMenu.id)?.color }} />
                {t('library.changeColor')}
              </button>
              <div className="h-px bg-[var(--color-border)] my-1" />
              <button
                onClick={() => {
                  const id = contextMenu.id;
                  setContextMenu(null);
                  setWorkspaces((prev) => prev.filter((w) => w.id !== id));
                  onDeleteWorkspace(id).then(() => loadData());
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-danger)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <Trash2 size={13} /> {t('library.deleteWorkspace')}
              </button>
            </>
          )}
          {contextMenu.type === 'prompt' && (
            <>
              {/* Move to workspace */}
              <div className="px-3 py-1 text-[10px] text-[var(--color-text-muted)] uppercase tracking-wider font-medium">
                {t('library.moveTo')}
              </div>
              {workspaces.map((ws) => (
                <button
                  key={ws.id}
                  onClick={() => {
                    backend.updateProject(contextMenu.id, { workspace_id: ws.id });
                    if (contextMenu.id === currentProjectId) {
                      onMovePrompt(ws.id);
                    }
                    setContextMenu(null);
                  }}
                  className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
                >
                  <div className="w-2 h-2 rounded" style={{ backgroundColor: ws.color }} />
                  {ws.name}
                </button>
              ))}
              <button
                onClick={() => {
                  backend.updateProject(contextMenu.id, { workspace_id: null });
                  if (contextMenu.id === currentProjectId) {
                    onMovePrompt(undefined);
                  }
                  setContextMenu(null);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <FolderInput size={13} /> {t('library.freePromptOption')}
              </button>
              <div className="h-px bg-[var(--color-border)] my-1" />
              <button
                onClick={async () => {
                  setContextMenu(null);
                  // Use local data instead of re-fetching
                  const original = projects.find((p) => p.id === contextMenu.id);
                  if (original) {
                    backend.createProject({
                      name: `${original.name} (copie)`,
                      blocks_json: JSON.stringify(original.blocks),
                      variables_json: JSON.stringify(original.variables),
                      workspace_id: original.workspaceId ?? null,
                      framework: original.framework ?? null,
                      tags_json: JSON.stringify(original.tags || []),
                    }).then(() => loadData());
                  }
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <Copy size={13} /> {t('library.duplicate')}
              </button>
              <div className="h-px bg-[var(--color-border)] my-1" />
              <button
                onClick={async (e) => {
                  await handleDeletePrompt(contextMenu.id, e);
                  setContextMenu(null);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-danger)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <Trash2 size={13} /> {t('library.deletePrompt')}
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
