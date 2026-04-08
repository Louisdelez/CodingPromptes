/**
 * SDD Phase Hooks — configurable before/after actions per phase
 */

import type { SddPhase } from './sdd-conventions';

export interface PhaseHook {
  type: 'webhook' | 'log';
  url?: string;       // for webhook
  method?: string;     // GET/POST
  message?: string;    // for log
}

export interface SddHooksConfig {
  hooks: Partial<Record<SddPhase, {
    before?: PhaseHook[];
    after?: PhaseHook[];
  }>>;
}

const HOOKS_KEY = 'inkwell-sdd-hooks';

export function getHooksConfig(): SddHooksConfig {
  try {
    return JSON.parse(localStorage.getItem(HOOKS_KEY) || '{"hooks":{}}');
  } catch { return { hooks: {} }; }
}

export function setHooksConfig(config: SddHooksConfig): void {
  localStorage.setItem(HOOKS_KEY, JSON.stringify(config));
}

export async function executeHooks(phase: SddPhase, timing: 'before' | 'after', context?: { content?: string; projectName?: string }): Promise<void> {
  const config = getHooksConfig();
  const phaseHooks = config.hooks[phase];
  if (!phaseHooks) return;

  const hooks = timing === 'before' ? phaseHooks.before : phaseHooks.after;
  if (!hooks?.length) return;

  for (const hook of hooks) {
    try {
      if (hook.type === 'webhook' && hook.url) {
        await fetch(hook.url, {
          method: hook.method || 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            phase,
            timing,
            projectName: context?.projectName,
            content: context?.content?.slice(0, 5000), // Limit payload size
            timestamp: new Date().toISOString(),
          }),
        });
      } else if (hook.type === 'log') {
        console.log(`[SDD Hook] ${timing} ${phase}: ${hook.message || 'executed'}`);
      }
    } catch (err) {
      console.error(`[SDD Hook] Failed ${timing} ${phase}:`, err);
    }
  }
}
