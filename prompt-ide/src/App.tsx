import { useState, useCallback, useEffect, lazy, Suspense } from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  arrayMove,
} from '@dnd-kit/sortable';
import {
  Plus,
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
  PanelRightOpen,
  FileText,
  Play,
  Layers,
  History,
  Clock,
  FolderOpen,
  Download,
  Sparkles,
  AlertTriangle,
  ChevronDown,
  Mic,
  Globe,
  Sun,
  Moon,
  Monitor,
  BarChart3,
  Link,
  MessageSquare,
  Users,
  Cpu,
} from 'lucide-react';
import { PromptBlockComponent } from './components/PromptBlock';
import { TokenCounter } from './components/TokenCounter';
import { VariablesPanel } from './components/VariablesPanel';
import { TagsEditor } from './components/TagsEditor';
import { PreviewPanel } from './components/PreviewPanel';
import { Library } from './components/Library';

// Lazy-loaded panels (code-splitting)
const Playground = lazy(() => import('./components/Playground').then(m => ({ default: m.Playground })));
const FrameworkSelector = lazy(() => import('./components/FrameworkSelector').then(m => ({ default: m.FrameworkSelector })));
const VersionHistory = lazy(() => import('./components/VersionHistory').then(m => ({ default: m.VersionHistory })));
const ExportPanel = lazy(() => import('./components/ExportPanel').then(m => ({ default: m.ExportPanel })));
const PromptOptimizer = lazy(() => import('./components/PromptOptimizer').then(m => ({ default: m.PromptOptimizer })));
const LintingPanel = lazy(() => import('./components/LintingPanel').then(m => ({ default: m.LintingPanel })));
const SttSettings = lazy(() => import('./components/SttSettings').then(m => ({ default: m.SttSettings })));
const ExecutionHistory = lazy(() => import('./components/ExecutionHistory').then(m => ({ default: m.ExecutionHistory })));
const AnalyticsPanel = lazy(() => import('./components/AnalyticsPanel').then(m => ({ default: m.AnalyticsPanel })));
const PromptChain = lazy(() => import('./components/PromptChain').then(m => ({ default: m.PromptChain })));
const ConversationMode = lazy(() => import('./components/ConversationMode').then(m => ({ default: m.ConversationMode })));
const CollaborationPanel = lazy(() => import('./components/CollaborationPanel').then(m => ({ default: m.CollaborationPanel })));
const GpuFleet = lazy(() => import('./components/GpuFleet').then(m => ({ default: m.GpuFleet })));
const TerminalPanel = lazy(() => import('./components/TerminalPanel').then(m => ({ default: m.TerminalPanel })));
const AuthPage = lazy(() => import('./components/AuthPage').then(m => ({ default: m.AuthPage })));
import { usePromptProject } from './hooks/usePromptProject';
// AuthPage loaded lazily above
import { UserMenu } from './components/UserMenu';
import { getSession, type AuthSession } from './lib/auth';
import { extractVariables } from './lib/prompt';
import type { BlockType, PromptBlock } from './lib/types';
import { BLOCK_CONFIG } from './lib/types';
import { I18nContext, getLang, setLang, useT, type Lang } from './lib/i18n';
import { ThemeContext, getThemeMode, setThemeMode, resolveTheme, applyTheme, type ThemeMode, type ResolvedTheme } from './lib/theme';

type LeftTab = 'library' | 'frameworks' | 'versions';
type RightTab = 'preview' | 'playground' | 'history' | 'stt' | 'export' | 'optimize' | 'lint' | 'analytics' | 'chain' | 'chat' | 'collab' | 'fleet';

const BLOCK_TYPES: BlockType[] = ['role', 'context', 'task', 'examples', 'constraints', 'format'];

export default function App() {
  const [language, setLanguage] = useState<Lang>(getLang);
  const [themeMode, setThemeModeState] = useState<ThemeMode>(getThemeMode);
  const [resolved, setResolved] = useState<ResolvedTheme>(() => resolveTheme(getThemeMode()));
  const [session, setSession] = useState<AuthSession | null>(() => getSession());

  const handleLanguageChange = (l: Lang) => { setLanguage(l); setLang(l); };
  const handleThemeChange = (m: ThemeMode) => { setThemeModeState(m); setThemeMode(m); const r = resolveTheme(m); setResolved(r); applyTheme(r); };

  // Apply theme on mount + listen for system changes
  useEffect(() => {
    applyTheme(resolved);
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      if (themeMode === 'system') {
        const r = resolveTheme('system');
        setResolved(r);
        applyTheme(r);
      }
    };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, [themeMode, resolved]);

  return (
    <I18nContext.Provider value={language}>
      <ThemeContext.Provider value={resolved}>
        {session ? (
          <AppInner
            session={session}
            setSession={setSession}
            onLogout={() => setSession(null)}
            language={language}
            onLanguageChange={handleLanguageChange}
            themeMode={themeMode}
            onThemeChange={handleThemeChange}
          />
        ) : (
          <Suspense fallback={<div className="min-h-screen bg-[var(--color-bg-primary)]" />}>
            <AuthPage
              onAuth={setSession}
              language={language}
              onLanguageChange={handleLanguageChange}
              themeMode={themeMode}
              onThemeChange={handleThemeChange}
            />
          </Suspense>
        )}
      </ThemeContext.Provider>
    </I18nContext.Provider>
  );
}

interface AppInnerProps {
  session: AuthSession;
  setSession: (session: AuthSession) => void;
  onLogout: () => void;
  language: Lang;
  onLanguageChange: (lang: Lang) => void;
  themeMode: ThemeMode;
  onThemeChange: (mode: ThemeMode) => void;
}

function AppInner({ session, setSession, onLogout, language, onLanguageChange, themeMode, onThemeChange }: AppInnerProps) {
  const t = useT();
  const {
    project,
    saveStatus,
    addBlock,
    removeBlock,
    updateBlock,
    toggleBlock,
    reorderBlocks,
    setVariable,
    loadProject,
    newProject,
    deleteProject,
    loadFramework,
    saveVersion,
    updateProject,
    movePromptToWorkspace,
    createWorkspace,
    updateWorkspace,
    deleteWorkspace,
    createFramework,
    updateFramework,
    deleteFramework,
    saveCurrentAsFramework,
    addTag,
    removeTag,
    libraryRefreshKey,
  } = usePromptProject(session.userId);

  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);

  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const [leftOpen, setLeftOpen] = useState(() => window.innerWidth >= 768);
  const [rightOpen, setRightOpen] = useState(() => window.innerWidth >= 768);
  const [leftTab, setLeftTab] = useState<LeftTab>('library');
  const [rightTab, setRightTab] = useState<RightTab>('preview');
  const [selectedModel, setSelectedModel] = useState('gpt-4o-mini');
  const [showAddMenu, setShowAddMenu] = useState(false);
  const [isEditingName, setIsEditingName] = useState(false);
  const [showThemeMenu, setShowThemeMenu] = useState(false);
  const [showLangMenu, setShowLangMenu] = useState(false);
  const [showRightTabMenu, setShowRightTabMenu] = useState(false);
  const [showLeftTabMenu, setShowLeftTabMenu] = useState(false);
  const [triggerExecute, setTriggerExecute] = useState(false);
  const [terminalOpen, setTerminalOpen] = useState(false);
  const [terminalHeight, setTerminalHeight] = useState(300);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.ctrlKey || e.metaKey;
      if (!mod) return;

      if (e.key === 'Enter') {
        e.preventDefault();
        setTriggerExecute(true);
      } else if (e.key === 's') {
        e.preventDefault();
        const now = new Date();
        const label = `Auto-save ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
        saveVersion(label);
      } else if (e.key === 'n') {
        e.preventDefault();
        newProject();
      } else if (e.key === '`') {
        e.preventDefault();
        setTerminalOpen((v) => !v);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [saveVersion, newProject]);

  // Close dropdowns on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('[data-dropdown]')) {
        setShowThemeMenu(false);
        setShowLangMenu(false);
        setShowRightTabMenu(false);
        setShowLeftTabMenu(false);
      }
    };
    document.addEventListener('click', handler);
    return () => document.removeEventListener('click', handler);
  }, []);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      if (over && active.id !== over.id) {
        const oldIndex = project.blocks.findIndex((b) => b.id === active.id);
        const newIndex = project.blocks.findIndex((b) => b.id === over.id);
        reorderBlocks(arrayMove(project.blocks, oldIndex, newIndex));
      }
    },
    [project.blocks, reorderBlocks]
  );

  const variables = extractVariables(project.blocks);

  const handleRestoreVersion = useCallback(
    (blocks: PromptBlock[], vars: Record<string, string>) => {
      updateProject((p) => ({ ...p, blocks, variables: vars }));
    },
    [updateProject]
  );

  const handleImport = useCallback(
    (blocks: PromptBlock[], variables: Record<string, string>, name?: string) => {
      updateProject((p) => ({
        ...p,
        blocks,
        variables,
        ...(name ? { name } : {}),
      }));
    },
    [updateProject]
  );

  const handleOptimizedApply = useCallback(
    (text: string) => {
      const taskBlock = project.blocks.find((b) => b.type === 'task');
      if (taskBlock) {
        updateBlock(taskBlock.id, { content: text });
      } else {
        addBlock('task');
        setTimeout(() => {
          updateProject((p) => {
            const last = p.blocks[p.blocks.length - 1];
            if (last?.type === 'task') {
              return { ...p, blocks: p.blocks.map((b) => (b.id === last.id ? { ...b, content: text } : b)) };
            }
            return p;
          });
        }, 50);
      }
    },
    [project.blocks, updateBlock, addBlock, updateProject]
  );

  const leftTabs: { id: LeftTab; icon: React.ComponentType<{ size?: number }>; label: string }[] = [
    { id: 'library', icon: FolderOpen, label: t('tab.library') },
    { id: 'frameworks', icon: Layers, label: t('tab.frameworks') },
    { id: 'versions', icon: History, label: t('tab.versions') },
  ];

  const rightTabs: { id: RightTab; icon: React.ComponentType<{ size?: number }>; label: string }[] = [
    { id: 'preview', icon: FileText, label: t('tab.preview') },
    { id: 'playground', icon: Play, label: t('tab.playground') },
    { id: 'history', icon: Clock, label: t('tab.history') },
    { id: 'stt', icon: Mic, label: t('tab.stt') },
    { id: 'optimize', icon: Sparkles, label: t('tab.optimize') },
    { id: 'lint', icon: AlertTriangle, label: t('tab.lint') },
    { id: 'export', icon: Download, label: t('tab.export') },
    { id: 'analytics', icon: BarChart3, label: t('tab.analytics') },
    { id: 'chain', icon: Link, label: t('tab.chain') },
    { id: 'chat', icon: MessageSquare, label: t('tab.chat') },
    { id: 'collab', icon: Users, label: t('tab.collab') },
    { id: 'fleet', icon: Cpu, label: t('tab.fleet') },
  ];

  // Block labels for the add menu
  const blockLabels: Record<BlockType, string> = {
    role: t('block.role'),
    context: t('block.context'),
    task: t('block.task'),
    examples: t('block.examples'),
    constraints: t('block.constraints'),
    format: t('block.format'),
  };

  return (
    <div className="flex flex-col h-screen bg-[var(--color-bg-primary)]">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <img src="/favicon.png" alt="Inkwell" className="w-7 h-7 rounded-lg" />
            <span className="text-sm font-semibold text-[var(--color-text-primary)]">{t('app.title')}</span>
          </div>
          <div className="w-px h-5 bg-[var(--color-border)]" />
          {isEditingName ? (
            <input
              autoFocus
              value={project.name}
              onChange={(e) => updateProject((p) => ({ ...p, name: e.target.value }))}
              onBlur={() => setIsEditingName(false)}
              onKeyDown={(e) => e.key === 'Enter' && setIsEditingName(false)}
              className="px-2 py-0.5 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-accent)] rounded outline-none text-[var(--color-text-primary)]"
            />
          ) : (
            <button
              onClick={() => setIsEditingName(true)}
              className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
            >
              {project.name}
            </button>
          )}
          {saveStatus !== 'idle' && (
            <span className={`text-xs ${
              saveStatus === 'saving' ? 'text-[var(--color-warning)] animate-pulse' :
              'text-[var(--color-success)]'
            }`}>
              {saveStatus === 'saving' ? t('app.saving') : `✓ ${t('app.saved')}`}
            </span>
          )}
        </div>

        <div className="flex items-center gap-1.5">
          {/* Theme dropdown */}
          <div className="relative" data-dropdown>
            <button
              onClick={() => { setShowThemeMenu(!showThemeMenu); setShowLangMenu(false); }}
              className="flex items-center gap-1 px-2 py-1 rounded text-xs font-medium bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-secondary)] transition-colors"
            >
              {themeMode === 'light' ? <Sun size={12} /> : themeMode === 'dark' ? <Moon size={12} /> : <Monitor size={12} />}
              <ChevronDown size={10} />
            </button>
            {showThemeMenu && (
              <div className="absolute right-0 top-full mt-1 w-36 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1 animate-fadeIn">
                {([
                  { mode: 'light' as ThemeMode, icon: Sun, label: 'Light' },
                  { mode: 'dark' as ThemeMode, icon: Moon, label: 'Dark' },
                  { mode: 'system' as ThemeMode, icon: Monitor, label: 'System' },
                ]).map(({ mode, icon: Icon, label }) => (
                  <button
                    key={mode}
                    onClick={() => { onThemeChange(mode); setShowThemeMenu(false); }}
                    className={`w-full flex items-center gap-2 px-3 py-1.5 text-xs transition-colors ${
                      themeMode === mode
                        ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10'
                        : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                    }`}
                  >
                    <Icon size={13} />
                    {label}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Language dropdown */}
          <div className="relative" data-dropdown>
            <button
              onClick={() => { setShowLangMenu(!showLangMenu); setShowThemeMenu(false); }}
              className="flex items-center gap-1 px-2 py-1 rounded text-xs font-medium bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-hover)] text-[var(--color-text-secondary)] transition-colors"
            >
              <Globe size={12} />
              {language.toUpperCase()}
              <ChevronDown size={10} />
            </button>
            {showLangMenu && (
              <div className="absolute right-0 top-full mt-1 w-36 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1 animate-fadeIn">
                {([
                  { code: 'fr' as Lang, label: 'Francais' },
                  { code: 'en' as Lang, label: 'English' },
                ]).map(({ code, label }) => (
                  <button
                    key={code}
                    onClick={() => { onLanguageChange(code); setShowLangMenu(false); }}
                    className={`w-full flex items-center gap-2 px-3 py-1.5 text-xs transition-colors ${
                      language === code
                        ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10'
                        : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                    }`}
                  >
                    {label}
                  </button>
                ))}
              </div>
            )}
          </div>

          <div className="w-px h-4 bg-[var(--color-border)]" />
          <UserMenu session={session} onLogout={onLogout} onSessionUpdate={setSession} />
          <div className="w-px h-4 bg-[var(--color-border)]" />
          <button
            onClick={() => setLeftOpen(!leftOpen)}
            className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)]"
          >
            {leftOpen ? <PanelLeftClose size={16} /> : <PanelLeftOpen size={16} />}
          </button>
          <button
            onClick={() => setRightOpen(!rightOpen)}
            className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)]"
          >
            {rightOpen ? <PanelRightClose size={16} /> : <PanelRightOpen size={16} />}
          </button>
        </div>
      </header>

      {/* Main Layout */}
      <div className="flex flex-1 overflow-hidden relative">
        {/* Mobile backdrop for left panel */}
        {isMobile && leftOpen && (
          <div className="mobile-backdrop" onClick={() => setLeftOpen(false)} />
        )}
        {/* Mobile backdrop for right panel */}
        {isMobile && rightOpen && (
          <div className="mobile-backdrop" onClick={() => setRightOpen(false)} />
        )}
        {/* Left Panel */}
        {leftOpen && (
          <div className={`${isMobile ? 'mobile-panel-left' : 'w-72'} flex-shrink-0 border-r border-[var(--color-border)] bg-[var(--color-bg-secondary)] flex flex-col animate-slideIn`}>
            {/* Left panel tab selector — dropdown */}
            <div className="relative border-b border-[var(--color-border)]" data-dropdown>
              <button
                onClick={() => setShowLeftTabMenu(!showLeftTabMenu)}
                className="w-full flex items-center justify-between px-4 py-2.5 text-sm font-medium text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] transition-colors"
              >
                <div className="flex items-center gap-2">
                  {(() => { const active = leftTabs.find((t) => t.id === leftTab); return active ? <><span className="text-[var(--color-accent)]"><active.icon size={14} /></span>{active.label}</> : null; })()}
                </div>
                <ChevronDown size={14} className={`text-[var(--color-text-muted)] transition-transform ${showLeftTabMenu ? 'rotate-180' : ''}`} />
              </button>
              {showLeftTabMenu && (
                <div className="absolute left-0 right-0 top-full bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)] shadow-xl z-50 py-1 animate-fadeIn">
                  {leftTabs.map((tab) => (
                    <button
                      key={tab.id}
                      onClick={() => { setLeftTab(tab.id); setShowLeftTabMenu(false); }}
                      className={`w-full flex items-center gap-2.5 px-4 py-2 text-sm transition-colors ${
                        leftTab === tab.id
                          ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10'
                          : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                      }`}
                    >
                      <tab.icon size={14} />
                      {tab.label}
                    </button>
                  ))}
                </div>
              )}
            </div>

            <div className="flex-1 overflow-auto">
              <Suspense fallback={<div className="p-4 text-sm text-[var(--color-text-muted)]">...</div>}>
              {leftTab === 'library' && (
                <Library
                  currentProjectId={project.id}
                  currentWorkspaceId={project.workspaceId}
                  refreshKey={libraryRefreshKey}
                  onLoadProject={loadProject}
                  onNewProject={newProject}
                  onCreateWorkspace={createWorkspace}
                  onUpdateWorkspace={updateWorkspace}
                  onDeleteWorkspace={deleteWorkspace}
                  onDeleteProject={deleteProject}
                  onMovePrompt={movePromptToWorkspace}
                />
              )}
              {leftTab === 'frameworks' && (
                <FrameworkSelector
                  currentFramework={project.framework}
                  onSelect={loadFramework}
                  onCreateFramework={createFramework}
                  onUpdateFramework={updateFramework}
                  onDeleteFramework={deleteFramework}
                  onSaveCurrentAsFramework={saveCurrentAsFramework}
                  currentBlocks={project.blocks}
                />
              )}
              {leftTab === 'versions' && (
                <VersionHistory
                  projectId={project.id}
                  currentBlocks={project.blocks}
                  variables={project.variables}
                  onSaveVersion={saveVersion}
                  onRestoreVersion={handleRestoreVersion}
                />
              )}
              </Suspense>
            </div>
          </div>
        )}

        {/* Center: Block Editor */}
        <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
          <div className="flex-1 overflow-auto p-4 space-y-3">
            <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
              <SortableContext items={project.blocks.map((b) => b.id)} strategy={verticalListSortingStrategy}>
                {project.blocks.map((block) => (
                  <PromptBlockComponent
                    key={block.id}
                    block={block}
                    onUpdate={(changes) => updateBlock(block.id, changes)}
                    onRemove={() => removeBlock(block.id)}
                    onToggle={() => toggleBlock(block.id)}
                    variables={variables}
                  />
                ))}
              </SortableContext>
            </DndContext>

            {/* Add block button */}
            <div className="relative">
              <button
                onClick={() => setShowAddMenu(!showAddMenu)}
                className="w-full flex items-center justify-center gap-2 py-3 rounded-lg border-2 border-dashed border-[var(--color-border)] hover:border-[var(--color-accent)] text-[var(--color-text-muted)] hover:text-[var(--color-accent)] transition-colors"
              >
                <Plus size={16} />
                <span className="text-sm">{t('block.add')}</span>
                <ChevronDown size={14} />
              </button>

              {showAddMenu && (
                <div className="absolute top-full left-0 right-0 mt-1 p-2 bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg shadow-xl z-10 grid grid-cols-2 gap-1 animate-fadeIn">
                  {BLOCK_TYPES.map((type) => {
                    const config = BLOCK_CONFIG[type];
                    return (
                      <button
                        key={type}
                        onClick={() => {
                          addBlock(type);
                          setShowAddMenu(false);
                        }}
                        className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-[var(--color-bg-hover)] transition-colors text-left"
                      >
                        <div className="w-2 h-2 rounded-full" style={{ backgroundColor: config.color }} />
                        <span className="text-sm text-[var(--color-text-primary)]">{blockLabels[type]}</span>
                      </button>
                    );
                  })}
                </div>
              )}
            </div>

            {/* Tags */}
            <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
              <TagsEditor tags={project.tags || []} onAddTag={addTag} onRemoveTag={removeTag} />
            </div>

            {/* Variables */}
            <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
              <VariablesPanel blocks={project.blocks} variables={project.variables} onSetVariable={setVariable} />
            </div>
          </div>

          {/* Terminal Panel (center column only, like VSCode) */}
          {terminalOpen && (
            <div style={{ height: terminalHeight }} className="flex-shrink-0 flex flex-col border-t border-[var(--color-border)]">
              {/* Resize handle */}
              <div
                className="h-1 cursor-row-resize hover:bg-[var(--color-accent)]/30 active:bg-[var(--color-accent)]/50 transition-colors"
                onMouseDown={(e) => {
                  e.preventDefault();
                  const startY = e.clientY;
                  const startH = terminalHeight;
                  const onMove = (ev: MouseEvent) => {
                    const delta = startY - ev.clientY;
                    setTerminalHeight(Math.max(120, Math.min(startH + delta, window.innerHeight - 200)));
                  };
                  const onUp = () => {
                    document.removeEventListener('mousemove', onMove);
                    document.removeEventListener('mouseup', onUp);
                  };
                  document.addEventListener('mousemove', onMove);
                  document.addEventListener('mouseup', onUp);
                }}
              />
              <div className="flex-1 min-h-0">
                <Suspense fallback={<div className="p-4 text-sm text-[var(--color-text-muted)]">...</div>}>
                  <TerminalPanel />
                </Suspense>
              </div>
            </div>
          )}
        </div>

        {/* Right Panel */}
        {rightOpen && (
          <div className={`${isMobile ? 'mobile-panel-right' : 'w-96'} flex-shrink-0 border-l border-[var(--color-border)] bg-[var(--color-bg-secondary)] flex flex-col animate-slideIn`}>
            {/* Right panel tab selector — dropdown */}
            <div className="relative border-b border-[var(--color-border)]" data-dropdown>
              <button
                onClick={() => setShowRightTabMenu(!showRightTabMenu)}
                className="w-full flex items-center justify-between px-4 py-2.5 text-sm font-medium text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] transition-colors"
              >
                <div className="flex items-center gap-2">
                  {(() => { const active = rightTabs.find((t) => t.id === rightTab); return active ? <><span className="text-[var(--color-accent)]"><active.icon size={14} /></span>{active.label}</> : null; })()}
                </div>
                <ChevronDown size={14} className={`text-[var(--color-text-muted)] transition-transform ${showRightTabMenu ? 'rotate-180' : ''}`} />
              </button>
              {showRightTabMenu && (
                <div className="absolute left-0 right-0 top-full bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)] shadow-xl z-50 py-1 animate-fadeIn">
                  {rightTabs.map((tab) => (
                    <button
                      key={tab.id}
                      onClick={() => { setRightTab(tab.id); setShowRightTabMenu(false); }}
                      className={`w-full flex items-center gap-2.5 px-4 py-2 text-sm transition-colors ${
                        rightTab === tab.id
                          ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10'
                          : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)]'
                      }`}
                    >
                      <tab.icon size={14} />
                      {tab.label}
                    </button>
                  ))}
                </div>
              )}
            </div>

            <div className="flex-1 overflow-auto">
              <Suspense fallback={<div className="p-4 text-sm text-[var(--color-text-muted)]">...</div>}>
                {rightTab === 'preview' && <PreviewPanel blocks={project.blocks} variables={project.variables} />}
                {rightTab === 'playground' && (
                  <Playground
                    blocks={project.blocks}
                    variables={project.variables}
                    projectId={project.id}
                    triggerExecute={triggerExecute}
                    onExecuteTriggered={() => setTriggerExecute(false)}
                  />
                )}
                {rightTab === 'history' && <ExecutionHistory projectId={project.id} />}
                {rightTab === 'stt' && <SttSettings />}
                {rightTab === 'export' && <ExportPanel project={project} onImport={handleImport} />}
                {rightTab === 'optimize' && <PromptOptimizer blocks={project.blocks} variables={project.variables} onApply={handleOptimizedApply} />}
                {rightTab === 'lint' && <LintingPanel blocks={project.blocks} variables={project.variables} />}
                {rightTab === 'analytics' && <AnalyticsPanel />}
                {rightTab === 'chain' && <PromptChain />}
                {rightTab === 'chat' && <ConversationMode blocks={project.blocks} variables={project.variables} />}
                {rightTab === 'collab' && <CollaborationPanel projectId={project.id} currentUserId={session.userId} />}
                {rightTab === 'fleet' && <GpuFleet />}
              </Suspense>
            </div>
          </div>
        )}
      </div>

      {/* Bottom bar (full width) */}
      <TokenCounter
        blocks={project.blocks}
        variables={project.variables}
        selectedModel={selectedModel}
        onModelChange={setSelectedModel}
        terminalOpen={terminalOpen}
        onToggleTerminal={() => setTerminalOpen((v) => !v)}
      />
    </div>
  );
}
