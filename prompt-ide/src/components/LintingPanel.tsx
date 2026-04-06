import { useMemo } from 'react';
import { AlertTriangle, CheckCircle, Info, XCircle } from 'lucide-react';
import type { PromptBlock } from '../lib/types';
import { extractVariables, compilePrompt } from '../lib/prompt';
import { countTokens } from '../lib/tokens';
import { useT } from '../lib/i18n';

interface LintIssue {
  level: 'error' | 'warning' | 'info';
  message: string;
}

interface LintingPanelProps {
  blocks: PromptBlock[];
  variables: Record<string, string>;
}

export function LintingPanel({ blocks, variables }: LintingPanelProps) {
  const t = useT();

  const issues = useMemo(() => {
    const result: LintIssue[] = [];
    const enabledBlocks = blocks.filter((b) => b.enabled);
    const compiled = compilePrompt(blocks, variables);
    const tokens = countTokens(compiled);

    // Check: no blocks
    if (enabledBlocks.length === 0) {
      result.push({ level: 'error', message: t('lint.noBlocks') });
    }

    // Check: empty blocks
    const emptyEnabled = enabledBlocks.filter((b) => !b.content.trim());
    if (emptyEnabled.length > 0) {
      result.push({
        level: 'warning',
        message: `${emptyEnabled.length} ${t('lint.emptyBlocks')}`,
      });
    }

    // Check: no task block
    if (!enabledBlocks.some((b) => b.type === 'task')) {
      result.push({ level: 'warning', message: t('lint.noTask') });
    }

    // Check: unresolved variables
    const vars = extractVariables(blocks);
    const unresolved = vars.filter((v) => !variables[v]?.trim());
    if (unresolved.length > 0) {
      result.push({
        level: 'warning',
        message: `${t('lint.unresolvedVars')} ${unresolved.map((v) => `{{${v}}}`).join(', ')}`,
      });
    }

    // Check: prompt too short
    if (compiled.length > 0 && compiled.length < 20) {
      result.push({ level: 'info', message: t('lint.tooShort') });
    }

    // Check: prompt too long
    if (tokens > 100000) {
      result.push({ level: 'warning', message: t('lint.tooLong', { n: String(tokens) }) });
    }

    // Check: no examples for complex prompts
    if (tokens > 200 && !enabledBlocks.some((b) => b.type === 'examples')) {
      result.push({ level: 'info', message: t('lint.noExamples') });
    }

    // Check: negative instructions
    const negativePatterns = /\b(ne pas|n'utilise pas|ne fais pas|jamais|ne génère pas|ne mentionne pas)\b/gi;
    if (negativePatterns.test(compiled)) {
      result.push({
        level: 'info',
        message: t('lint.negativeInstructions'),
      });
    }

    // All good
    if (result.length === 0) {
      result.push({ level: 'info', message: t('lint.allGood') });
    }

    return result;
  }, [blocks, variables, t]);

  const icons = {
    error: <XCircle size={14} className="text-[var(--color-danger)] flex-shrink-0" />,
    warning: <AlertTriangle size={14} className="text-[var(--color-warning)] flex-shrink-0" />,
    info: <Info size={14} className="text-[var(--color-accent)] flex-shrink-0" />,
  };

  return (
    <div className="p-4 space-y-3">
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)]">
        {issues.every((i) => i.level === 'info') ? (
          <CheckCircle size={14} className="text-[var(--color-success)]" />
        ) : (
          <AlertTriangle size={14} className="text-[var(--color-warning)]" />
        )}
        <span>{t('lint.title')}</span>
      </div>
      <div className="space-y-2">
        {issues.map((issue, i) => (
          <div
            key={i}
            className="flex items-start gap-2 p-2 rounded-lg bg-[var(--color-bg-tertiary)] text-sm animate-fadeIn"
          >
            {icons[issue.level]}
            <span className="text-[var(--color-text-secondary)]">{issue.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
