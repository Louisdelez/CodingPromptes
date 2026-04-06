import { useState } from 'react';
import { X, Tag } from 'lucide-react';
import { useT } from '../lib/i18n';

interface TagsEditorProps {
  tags: string[];
  onAddTag: (tag: string) => void;
  onRemoveTag: (tag: string) => void;
}

export function TagsEditor({ tags, onAddTag, onRemoveTag }: TagsEditorProps) {
  const t = useT();
  const [input, setInput] = useState('');

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim()) {
      e.preventDefault();
      onAddTag(input.trim());
      setInput('');
    }
  };

  return (
    <div className="p-4 space-y-2">
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        <Tag size={14} />
        <span>{t('tags.title')}</span>
      </div>
      <div className="flex flex-wrap items-center gap-1.5">
        {tags.map((tag) => (
          <span
            key={tag}
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
          >
            {tag}
            <button
              onClick={() => onRemoveTag(tag)}
              className="hover:text-[var(--color-danger)] transition-colors"
            >
              <X size={10} />
            </button>
          </span>
        ))}
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t('tags.add')}
          className="flex-1 min-w-[120px] px-2 py-1 text-xs bg-transparent border-none outline-none text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)]"
        />
      </div>
    </div>
  );
}
