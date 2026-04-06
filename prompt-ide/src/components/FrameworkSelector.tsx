import { useState, useEffect } from 'react';
import {
  Layers,
  Plus,
  Save,
  Trash2,
  Pencil,
  X,
  ChevronDown,
  ChevronRight,
  Copy,
  BookMarked,
  User as UserIcon,
} from 'lucide-react';
import type { PromptBlock, CustomFramework, BlockType } from '../lib/types';
import { FRAMEWORKS, BLOCK_CONFIG } from '../lib/types';
import { db } from '../lib/db';
import { useT } from '../lib/i18n';

const BLOCK_TYPES: BlockType[] = ['role', 'context', 'task', 'examples', 'constraints', 'format'];

interface FrameworkSelectorProps {
  userId: string;
  currentFramework?: string;
  onSelect: (frameworkId: string, blocks: Omit<PromptBlock, 'id'>[]) => void;
  onCreateFramework: (name: string, description: string, blocks: Omit<PromptBlock, 'id'>[]) => Promise<CustomFramework>;
  onUpdateFramework: (id: string, changes: Partial<CustomFramework>) => Promise<void>;
  onDeleteFramework: (id: string) => Promise<void>;
  onSaveCurrentAsFramework: (name: string, description: string) => Promise<CustomFramework>;
  currentBlocks: PromptBlock[];
}

type EditStep = 'name' | 'blocks';

interface BlockDraft {
  type: BlockType;
  content: string;
  enabled: boolean;
}

export function FrameworkSelector({
  userId,
  currentFramework,
  onSelect,
  onCreateFramework,
  onUpdateFramework,
  onDeleteFramework,
  onSaveCurrentAsFramework,
  currentBlocks,
}: FrameworkSelectorProps) {
  const t = useT();
  const [customFrameworks, setCustomFrameworks] = useState<CustomFramework[]>([]);
  const [showBuiltin, setShowBuiltin] = useState(true);
  const [showCustom, setShowCustom] = useState(true);

  // Creation / edition state
  const [mode, setMode] = useState<'browse' | 'create' | 'edit' | 'save-current'>('browse');
  const [editStep, setEditStep] = useState<EditStep>('name');
  const [editId, setEditId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [editDesc, setEditDesc] = useState('');
  const [editBlocks, setEditBlocks] = useState<BlockDraft[]>([]);

  useEffect(() => {
    const load = async () => {
      const all = await db.frameworks.where('userId').equals(userId).reverse().sortBy('updatedAt');
      setCustomFrameworks(all);
    };
    load();
    const interval = setInterval(load, 3000);
    return () => clearInterval(interval);
  }, [userId]);

  const resetForm = () => {
    setMode('browse');
    setEditStep('name');
    setEditId(null);
    setEditName('');
    setEditDesc('');
    setEditBlocks([]);
  };

  const startCreate = () => {
    setMode('create');
    setEditStep('name');
    setEditName('');
    setEditDesc('');
    setEditBlocks([{ type: 'role', content: '', enabled: true }, { type: 'task', content: '', enabled: true }]);
  };

  const startSaveCurrent = () => {
    setMode('save-current');
    setEditStep('name');
    setEditName('');
    setEditDesc('');
  };

  const startEdit = (fw: CustomFramework) => {
    setMode('edit');
    setEditStep('name');
    setEditId(fw.id);
    setEditName(fw.name);
    setEditDesc(fw.description);
    setEditBlocks(fw.blocks.map((b) => ({ ...b })));
  };

  const addEditBlock = (type: BlockType) => {
    setEditBlocks((prev) => [...prev, { type, content: '', enabled: true }]);
  };

  const removeEditBlock = (index: number) => {
    setEditBlocks((prev) => prev.filter((_, i) => i !== index));
  };

  const updateEditBlock = (index: number, changes: Partial<BlockDraft>) => {
    setEditBlocks((prev) => prev.map((b, i) => (i === index ? { ...b, ...changes } : b)));
  };

  const handleSave = async () => {
    if (!editName.trim()) return;

    if (mode === 'save-current') {
      await onSaveCurrentAsFramework(editName.trim(), editDesc.trim());
    } else if (mode === 'create') {
      await onCreateFramework(editName.trim(), editDesc.trim(), editBlocks);
    } else if (mode === 'edit' && editId) {
      await onUpdateFramework(editId, { name: editName.trim(), description: editDesc.trim(), blocks: editBlocks });
    }

    resetForm();
  };

  const handleDelete = async (id: string) => {
    await onDeleteFramework(id);
    setCustomFrameworks((prev) => prev.filter((f) => f.id !== id));
  };

  // --- BROWSE MODE ---
  if (mode === 'browse') {
    return (
      <div className="flex flex-col h-full">
        <div className="p-3 border-b border-[var(--color-border)] space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
              <Layers size={14} />
              <span>{t('frameworks.title')}</span>
            </div>
          </div>
          <div className="flex gap-1.5">
            <button
              onClick={startCreate}
              className="flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded text-xs bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] transition-colors"
            >
              <Plus size={12} />
              {t('frameworks.create')}
            </button>
            <button
              onClick={startSaveCurrent}
              disabled={currentBlocks.length === 0}
              className="flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded text-xs bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] disabled:opacity-40 transition-colors"
            >
              <Save size={12} />
              {t('frameworks.fromCurrent')}
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-auto">
          {/* Custom frameworks */}
          {customFrameworks.length > 0 && (
            <div>
              <button
                onClick={() => setShowCustom(!showCustom)}
                className="w-full flex items-center gap-1.5 px-3 py-2 text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider hover:bg-[var(--color-bg-hover)]"
              >
                {showCustom ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                <UserIcon size={11} />
                {t('frameworks.myFrameworks')} ({customFrameworks.length})
              </button>
              {showCustom && (
                <div className="px-3 pb-2 grid grid-cols-1 gap-1.5">
                  {customFrameworks.map((fw) => (
                    <div
                      key={fw.id}
                      className={`group relative text-left p-2.5 rounded-lg border transition-all ${
                        currentFramework === `custom:${fw.id}`
                          ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10'
                          : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)]'
                      }`}
                    >
                      <button
                        onClick={() => onSelect(`custom:${fw.id}`, fw.blocks)}
                        className="w-full text-left"
                      >
                        <div className="text-sm font-semibold text-[var(--color-text-primary)] flex items-center gap-1.5">
                          <BookMarked size={12} className="text-[var(--color-accent)]" />
                          {fw.name}
                        </div>
                        {fw.description && (
                          <div className="text-xs text-[var(--color-text-muted)] mt-0.5">{fw.description}</div>
                        )}
                        <div className="flex flex-wrap gap-1 mt-1.5">
                          {fw.blocks.map((b, i) => (
                            <span
                              key={i}
                              className="text-[10px] px-1.5 py-0.5 rounded-full"
                              style={{ backgroundColor: `${BLOCK_CONFIG[b.type].color}20`, color: BLOCK_CONFIG[b.type].color }}
                            >
                              {BLOCK_CONFIG[b.type].label.split(' ')[0]}
                            </span>
                          ))}
                        </div>
                      </button>
                      {/* Actions */}
                      <div className="absolute top-2 right-2 flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button
                          onClick={() => startEdit(fw)}
                          className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-accent)]"
                          title={t('frameworks.modify')}
                        >
                          <Pencil size={11} />
                        </button>
                        <button
                          onClick={() => {
                            // Duplicate
                            onCreateFramework(`${fw.name} (copie)`, fw.description, fw.blocks);
                          }}
                          className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)]"
                          title={t('frameworks.duplicate')}
                        >
                          <Copy size={11} />
                        </button>
                        <button
                          onClick={() => handleDelete(fw.id)}
                          className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-danger)]"
                          title={t('frameworks.delete')}
                        >
                          <Trash2 size={11} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Built-in frameworks */}
          <div>
            <button
              onClick={() => setShowBuiltin(!showBuiltin)}
              className="w-full flex items-center gap-1.5 px-3 py-2 text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider hover:bg-[var(--color-bg-hover)]"
            >
              {showBuiltin ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
              <Layers size={11} />
              {t('frameworks.builtIn')} ({Object.keys(FRAMEWORKS).length})
            </button>
            {showBuiltin && (
              <div className="px-3 pb-2 grid grid-cols-2 gap-1.5">
                {Object.entries(FRAMEWORKS).map(([id, fw]) => (
                  <button
                    key={id}
                    onClick={() => onSelect(id, fw.blocks)}
                    className={`text-left p-2.5 rounded-lg border transition-all ${
                      currentFramework === id
                        ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10'
                        : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)]'
                    }`}
                  >
                    <div className="text-sm font-semibold text-[var(--color-text-primary)]">{fw.name}</div>
                    <div className="text-xs text-[var(--color-text-muted)] mt-0.5">{fw.description}</div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }

  // --- CREATE / EDIT / SAVE-CURRENT MODE ---
  const isFromCurrent = mode === 'save-current';
  const title = mode === 'edit' ? t('frameworks.editFramework') : isFromCurrent ? t('frameworks.saveAsFw') : t('frameworks.newFramework');

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-3 border-b border-[var(--color-border)]">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-[var(--color-text-primary)]">{title}</span>
          <button onClick={resetForm} className="p-1 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)]">
            <X size={14} />
          </button>
        </div>
        {/* Steps indicator */}
        {!isFromCurrent && (
          <div className="flex gap-2 mt-2">
            <button
              onClick={() => setEditStep('name')}
              className={`flex-1 text-center py-1 text-xs rounded transition-colors ${
                editStep === 'name'
                  ? 'bg-[var(--color-accent)] text-white'
                  : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)]'
              }`}
            >
              {t('frameworks.stepInfo')}
            </button>
            <button
              onClick={() => editName.trim() ? setEditStep('blocks') : undefined}
              className={`flex-1 text-center py-1 text-xs rounded transition-colors ${
                editStep === 'blocks'
                  ? 'bg-[var(--color-accent)] text-white'
                  : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)]'
              } ${!editName.trim() ? 'opacity-40 cursor-not-allowed' : ''}`}
            >
              {t('frameworks.stepBlocks')}
            </button>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-auto p-3 space-y-3">
        {/* Step 1: Name & Description (always shown for save-current) */}
        {(editStep === 'name' || isFromCurrent) && (
          <div className="space-y-3 animate-fadeIn">
            <div>
              <label className="block text-xs text-[var(--color-text-muted)] mb-1">{t('frameworks.name')}</label>
              <input
                autoFocus
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                placeholder={t('frameworks.namePlaceholder')}
                className="w-full px-2.5 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
              />
            </div>
            <div>
              <label className="block text-xs text-[var(--color-text-muted)] mb-1">{t('frameworks.description')}</label>
              <input
                value={editDesc}
                onChange={(e) => setEditDesc(e.target.value)}
                placeholder={t('frameworks.descPlaceholder')}
                className="w-full px-2.5 py-2 text-sm bg-[var(--color-bg-tertiary)] border border-[var(--color-border)] rounded-lg focus:border-[var(--color-accent)] outline-none text-[var(--color-text-primary)]"
              />
            </div>

            {isFromCurrent && (
              <>
                <div className="p-2.5 rounded-lg bg-[var(--color-bg-tertiary)] border border-[var(--color-border)]">
                  <div className="text-xs text-[var(--color-text-muted)] mb-1.5">{t('frameworks.blocksToSave')}</div>
                  <div className="flex flex-wrap gap-1">
                    {currentBlocks.map((b, i) => (
                      <span
                        key={i}
                        className="text-[10px] px-1.5 py-0.5 rounded-full"
                        style={{ backgroundColor: `${BLOCK_CONFIG[b.type].color}20`, color: BLOCK_CONFIG[b.type].color }}
                      >
                        {BLOCK_CONFIG[b.type].label}
                      </span>
                    ))}
                  </div>
                </div>
                <button
                  onClick={handleSave}
                  disabled={!editName.trim()}
                  className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 text-sm font-medium transition-colors"
                >
                  <Save size={14} />
                  {t('frameworks.saveFramework')}
                </button>
              </>
            )}

            {!isFromCurrent && (
              <button
                onClick={() => editName.trim() && setEditStep('blocks')}
                disabled={!editName.trim()}
                className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 text-sm font-medium transition-colors"
              >
                {t('frameworks.nextBlocks')}
              </button>
            )}
          </div>
        )}

        {/* Step 2: Block editor (create/edit only) */}
        {editStep === 'blocks' && !isFromCurrent && (
          <div className="space-y-2 animate-fadeIn">
            <div className="text-xs text-[var(--color-text-muted)]">
              {t('frameworks.blocksHint')}
            </div>

            {editBlocks.map((block, index) => (
              <div
                key={index}
                className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] overflow-hidden"
              >
                <div className="flex items-center gap-2 px-2.5 py-1.5 border-b border-[var(--color-border)]">
                  <div className="w-2 h-2 rounded-full flex-shrink-0" style={{ backgroundColor: BLOCK_CONFIG[block.type].color }} />
                  <select
                    value={block.type}
                    onChange={(e) => updateEditBlock(index, { type: e.target.value as BlockType })}
                    className="flex-1 text-xs bg-transparent text-[var(--color-text-primary)] outline-none"
                  >
                    {BLOCK_TYPES.map((t) => (
                      <option key={t} value={t}>{BLOCK_CONFIG[t].label}</option>
                    ))}
                  </select>
                  <button
                    onClick={() => removeEditBlock(index)}
                    disabled={editBlocks.length <= 1}
                    className="p-0.5 rounded hover:bg-[var(--color-bg-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-danger)] disabled:opacity-30"
                  >
                    <Trash2 size={11} />
                  </button>
                </div>
                <textarea
                  value={block.content}
                  onChange={(e) => updateEditBlock(index, { content: e.target.value })}
                  placeholder={`Contenu pre-rempli (ex: ## ${BLOCK_CONFIG[block.type].label}\n...)`}
                  rows={2}
                  className="w-full px-2.5 py-1.5 text-xs bg-transparent text-[var(--color-text-primary)] outline-none resize-none placeholder:text-[var(--color-text-muted)] font-mono"
                />
              </div>
            ))}

            {/* Add block to framework */}
            <div className="flex flex-wrap gap-1">
              {BLOCK_TYPES.map((type) => (
                <button
                  key={type}
                  onClick={() => addEditBlock(type)}
                  className="flex items-center gap-1 px-2 py-1 rounded text-[10px] border border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] transition-colors"
                >
                  <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: BLOCK_CONFIG[type].color }} />
                  + {BLOCK_CONFIG[type].label.split(' ')[0]}
                </button>
              ))}
            </div>

            {/* Save */}
            <button
              onClick={handleSave}
              disabled={editBlocks.length === 0}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 text-sm font-medium transition-colors"
            >
              <Save size={14} />
              {mode === 'edit' ? t('frameworks.saveChanges') : t('frameworks.createFramework')}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
