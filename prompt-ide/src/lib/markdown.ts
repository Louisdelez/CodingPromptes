export function renderMarkdown(text: string): string {
  const html = text
    // Escape HTML
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    // Code blocks (must be before other transformations)
    .replace(/```[\w]*\n([\s\S]*?)```/g, '<pre class="p-2 rounded bg-[var(--color-bg-primary)] border border-[var(--color-border)] text-xs overflow-auto my-2"><code>$1</code></pre>')
    // Headers
    .replace(/^### (.+)$/gm, '<h4 class="text-sm font-bold mt-3 mb-1">$1</h4>')
    .replace(/^## (.+)$/gm, '<h3 class="text-base font-bold mt-3 mb-1">$1</h3>')
    .replace(/^# (.+)$/gm, '<h2 class="text-lg font-bold mt-3 mb-1">$1</h2>')
    // Bold
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    // Italic
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code class="px-1 py-0.5 rounded bg-[var(--color-bg-hover)] text-[var(--color-role)] text-xs">$1</code>')
    // Unordered lists
    .replace(/^- (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
    // Ordered lists
    .replace(/^\d+\. (.+)$/gm, '<li class="ml-4 list-decimal">$1</li>')
    // Line breaks (but not inside code blocks)
    .replace(/\n/g, '<br/>');

  return html;
}
