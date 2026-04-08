import { useState, useEffect, useRef, useCallback, memo } from 'react';
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
  Shield,
} from 'lucide-react';
import type { PromptProject, Workspace } from '../lib/types';
import { WORKSPACE_COLORS } from '../lib/types';
import * as backend from '../lib/backend';
import { localdb } from '../lib/localdb';
import { useT } from '../lib/i18n';

interface LibraryProps {
  currentProjectId: string;
  refreshKey?: number;
  onLoadProject: (id: string) => void;
  onNewProject: (workspaceId?: string) => void;
  onCreateWorkspace: (name: string, color?: string) => Promise<Workspace>;
  onUpdateWorkspace?: (id: string, changes: Partial<Workspace>) => Promise<void>;
  onDeleteWorkspace: (id: string) => Promise<void>;
  onDeleteProject?: (id: string) => Promise<void>;
  onMovePrompt: (workspaceId: string | undefined) => void;
  currentWorkspaceId?: string;
}



const DraggablePromptItem = memo(function DraggablePromptItem({ prompt, children }: { prompt: PromptProject; children: React.ReactNode }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: prompt.id });
  return (
    <div ref={setNodeRef} {...attributes} {...listeners} style={{ opacity: isDragging ? 0.4 : 1, cursor: 'grab' }}>
      {children}
    </div>
  );
});

const DroppableWorkspace = memo(function DroppableWorkspace({ wsId, children }: { wsId: string; children: React.ReactNode }) {
  const { setNodeRef, isOver } = useDroppable({ id: wsId });
  return (
    <div ref={setNodeRef} style={{ backgroundColor: isOver ? 'var(--color-accent-hover)' : undefined, borderRadius: '4px', transition: 'background-color 0.15s' }}>
      {children}
    </div>
  );
});

export function Library({
  currentProjectId,
  refreshKey,
  onLoadProject,
  onNewProject,
  onCreateWorkspace,
  onUpdateWorkspace,
  onDeleteWorkspace,
  onDeleteProject,
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
  const [constitutionWsId, setConstitutionWsId] = useState<string | null>(null);
  const [constitutionText, setConstitutionText] = useState('');
  const [newWsColor, setNewWsColor] = useState<string>(WORKSPACE_COLORS[0]);
  const newWsInputRef = useRef<HTMLInputElement>(null);
  const [activeDragId, setActiveDragId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 10 } })
  );

  const loadData = useCallback(async () => {
    // Read from LOCAL IndexedDB — instant (< 1ms)
    const lws = await localdb.workspaces.orderBy('updatedAt').reverse().toArray();
    const lpj = await localdb.projects.orderBy('updatedAt').reverse().toArray();
    setWorkspaces(lws.map((w) => ({
      id: w.id, name: w.name, description: w.description, color: w.color,
      createdAt: w.createdAt, updatedAt: w.updatedAt,
    })));
    setProjects(lpj.map((p) => ({
      id: p.id, name: p.name, workspaceId: p.workspaceId,
      blocks: JSON.parse(p.blocksJson || '[]'),
      variables: JSON.parse(p.variablesJson || '{}'),
      tags: JSON.parse(p.tagsJson || '[]'),
      framework: p.framework, createdAt: p.createdAt, updatedAt: p.updatedAt,
    })));
  }, []);

  // Load on mount + when refreshKey changes (triggered by hook mutations)
  useEffect(() => { const t = setTimeout(loadData, 0); return () => clearTimeout(t); }, [loadData, refreshKey]);

  const handleNewProject = (wsId?: string) => {
    // Optimistic: add to local list immediately
    const tempId = `temp-${Date.now()}`;
    const now = Date.now();
    const newPrompt: PromptProject = {
      id: tempId, name: 'Nouveau prompt', workspaceId: wsId,
      blocks: [], variables: {}, tags: [], createdAt: now, updatedAt: now,
    };
    setProjects((prev) => [newPrompt, ...prev]);
    // Create on backend (fire-and-forget), then soft-refresh to get real ID
    onNewProject(wsId);
    setTimeout(loadData, 1500); // Gentle background sync, not blocking
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
    // Create on backend (fire-and-forget), soft-refresh later for real ID
    onCreateWorkspace(name, color).catch(() => {});
    setTimeout(loadData, 1500);
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
    if (!window.confirm(t('library.confirmDeletePrompt'))) return;
    setProjects((prev) => prev.filter((p) => p.id !== id));
    if (onDeleteProject) {
      onDeleteProject(id);
    } else {
      localdb.projects.delete(id);
      backend.deleteProject(id).catch(() => {});
    }
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
    localdb.projects.update(promptId, { workspaceId: newWorkspaceId });

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
              <button
                onClick={async () => {
                  const ws = await localdb.workspaces.get(contextMenu.id);
                  setConstitutionWsId(contextMenu.id);
                  setConstitutionText(ws?.constitution || '');
                  setContextMenu(null);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] flex items-center gap-2"
              >
                <Shield size={13} className="text-[var(--color-accent)]" /> Constitution
              </button>
              <div className="h-px bg-[var(--color-border)] my-1" />
              <button
                onClick={() => {
                  if (!window.confirm(t('library.confirmDeleteWorkspace'))) return;
                  const id = contextMenu.id;
                  setContextMenu(null);
                  setWorkspaces((prev) => prev.filter((w) => w.id !== id));
                  onDeleteWorkspace(id).catch(() => {});
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
                    localdb.projects.update(contextMenu.id, { workspaceId: ws.id });
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
                  localdb.projects.update(contextMenu.id, { workspaceId: undefined });
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
                    const copyId = crypto.randomUUID();
                    const now = Date.now();
                    const copy: PromptProject = {
                      ...original, id: copyId,
                      name: `${original.name} (copie)`,
                      createdAt: now, updatedAt: now,
                    };
                    // Write to local DB (instant) + update UI
                    setProjects((prev) => [copy, ...prev]);
                    localdb.projects.put({
                      id: copyId, name: copy.name, workspaceId: copy.workspaceId,
                      blocksJson: JSON.stringify(original.blocks),
                      variablesJson: JSON.stringify(original.variables),
                      framework: original.framework, tagsJson: JSON.stringify(original.tags || []),
                      createdAt: now, updatedAt: now,
                    });
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

      {/* Constitution Editor Modal */}
      {constitutionWsId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => setConstitutionWsId(null)}>
          <div className="bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-xl p-4 w-[90%] max-w-lg max-h-[80vh] shadow-2xl animate-fadeIn" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <Shield size={16} className="text-[var(--color-accent)]" />
                <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">
                  Constitution — {workspaces.find(w => w.id === constitutionWsId)?.name}
                </h3>
              </div>
              <button onClick={() => setConstitutionWsId(null)} className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]">
                <X size={16} />
              </button>
            </div>
            <textarea
              value={constitutionText}
              onChange={e => setConstitutionText(e.target.value)}
              placeholder="# Project Constitution&#10;&#10;## Core Principles&#10;..."
              className="w-full h-64 px-3 py-2 text-xs font-mono bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)] resize-none"
            />
            <div className="flex items-center justify-between mt-3">
              <span className="text-[10px] text-[var(--color-text-muted)]">
                Shared across all SDD prompts in this workspace
              </span>
              <button
                onClick={async () => {
                  if (onUpdateWorkspace) {
                    await onUpdateWorkspace(constitutionWsId, { constitution: constitutionText });
                  }
                  setConstitutionWsId(null);
                }}
                className="px-3 py-1.5 rounded text-xs font-medium bg-[var(--color-accent)] text-white hover:opacity-90"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
