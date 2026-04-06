import type { PromptBlock } from './types';

export function compilePrompt(blocks: PromptBlock[], variables: Record<string, string>): string {
  const enabledBlocks = blocks.filter((b) => b.enabled);
  let text = enabledBlocks.map((b) => b.content).join('\n\n');

  // Replace variables
  for (const [key, value] of Object.entries(variables)) {
    text = text.replaceAll(`{{${key}}}`, value);
  }

  return text.trim();
}

export function extractVariables(blocks: PromptBlock[]): string[] {
  const all = blocks.map((b) => b.content).join('\n');
  const matches = all.matchAll(/\{\{(\w+)\}\}/g);
  const unique = new Set<string>();
  for (const m of matches) {
    unique.add(m[1]);
  }
  return [...unique];
}
