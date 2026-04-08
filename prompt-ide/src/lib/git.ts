/**
 * Git integration for SDD workflow
 * Works via: Tauri (desktop) or Backend API (web)
 */

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

async function tauriExec(cmd: string, args: string[]): Promise<string> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke('run_git_command', { cmd, args });
}

async function backendExec(cmd: string, args: string[]): Promise<string> {
  const { getLocalServerUrl } = await import('./types');
  const baseUrl = getLocalServerUrl().replace(/\/$/, '');
  const res = await fetch(`${baseUrl}/api/git/exec`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ cmd, args }),
  });
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return data.output;
}

async function gitExec(args: string[]): Promise<string> {
  if (isTauri) {
    return tauriExec('git', args);
  }
  return backendExec('git', args);
}

export async function getGitStatus(): Promise<{ branch: string; clean: boolean; available: boolean }> {
  try {
    const branch = (await gitExec(['rev-parse', '--abbrev-ref', 'HEAD'])).trim();
    const status = (await gitExec(['status', '--porcelain'])).trim();
    return { branch, clean: status === '', available: true };
  } catch {
    return { branch: '', clean: true, available: false };
  }
}

export async function getNextFeatureNumber(): Promise<number> {
  try {
    const branches = await gitExec(['branch', '--list', '--format=%(refname:short)']);
    const nums = branches.split('\n')
      .map(b => b.trim().match(/^(\d+)-/))
      .filter(Boolean)
      .map(m => parseInt(m![1]));
    return nums.length > 0 ? Math.max(...nums) + 1 : 1;
  } catch {
    return 1;
  }
}

export async function createFeatureBranch(name: string): Promise<string> {
  const num = await getNextFeatureNumber();
  const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  const branchName = `${String(num).padStart(3, '0')}-${slug}`;
  await gitExec(['checkout', '-b', branchName]);
  return branchName;
}

export async function commitPhase(phase: string, message?: string): Promise<string> {
  const msg = message || `spec: update ${phase.replace('sdd-', '')}`;
  await gitExec(['add', '-A']);
  await gitExec(['commit', '-m', msg]);
  const hash = (await gitExec(['rev-parse', '--short', 'HEAD'])).trim();
  return hash;
}

export async function commitAll(message: string): Promise<string> {
  await gitExec(['add', '-A']);
  await gitExec(['commit', '-m', message]);
  const hash = (await gitExec(['rev-parse', '--short', 'HEAD'])).trim();
  return hash;
}
