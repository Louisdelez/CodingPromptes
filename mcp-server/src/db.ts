import Database from 'better-sqlite3';
import { existsSync } from 'fs';

interface DbProject {
  id: string;
  name: string;
  user_id: string;
  workspace_id: string | null;
  blocks_json: string;
  variables_json: string;
  framework: string | null;
  tags_json: string;
  created_at: number;
  updated_at: number;
}

interface ProjectSummary {
  id: string;
  name: string;
  type: 'prompt' | 'spec';
  updatedAt: number;
  blockCount?: number;
  currentPhase?: string;
}

interface ParsedTask {
  id: string;
  title: string;
  files: string[];
  dependencies: string[];
  complexity: string;
  criteria: string[];
  notes: string;
}

export class InkwellDB {
  private db: Database.Database | null = null;
  private dbPath: string;

  constructor(dbPath: string) {
    this.dbPath = dbPath;
  }

  private open(): Database.Database {
    if (this.db) return this.db;
    if (!existsSync(this.dbPath)) {
      throw new Error(`Inkwell database not found at: ${this.dbPath}`);
    }
    this.db = new Database(this.dbPath, { readonly: true });
    return this.db;
  }

  listProjects(type?: string): ProjectSummary[] {
    const db = this.open();
    const rows = db.prepare('SELECT id, name, blocks_json, updated_at FROM projects ORDER BY updated_at DESC').all() as DbProject[];

    return rows
      .map(row => {
        const blocks = this.parseBlocks(row.blocks_json);
        const sddMeta = this.extractSddMeta(blocks);
        const projectType = sddMeta ? 'spec' : 'prompt';

        if (type && type !== projectType) return null;

        return {
          id: row.id,
          name: row.name,
          type: projectType,
          updatedAt: row.updated_at,
          blockCount: projectType === 'prompt' ? blocks.length : undefined,
          currentPhase: sddMeta?.currentPhase,
        } as ProjectSummary;
      })
      .filter((p): p is ProjectSummary => p !== null);
  }

  readProject(projectId: string): string | null {
    const db = this.open();
    const row = db.prepare('SELECT blocks_json, variables_json, name FROM projects WHERE id = ?').get(projectId) as DbProject | undefined;
    if (!row) return null;

    const blocks = this.parseBlocks(row.blocks_json);
    const sddMeta = this.extractSddMeta(blocks);

    if (sddMeta) {
      // Spec project: concatenate all phases
      const phases = ['constitution', 'specification', 'plan', 'tasks', 'implementation'] as const;
      let output = `# ${row.name}\n\n`;
      for (const phase of phases) {
        const content = sddMeta.phases[phase];
        if (content && content.trim()) {
          output += `---\n\n${content}\n\n`;
        }
      }
      return output;
    }

    // Regular prompt project: compile blocks
    let output = `# ${row.name}\n\n`;
    for (const block of blocks) {
      if (block.enabled) {
        output += `## [${block.type}]\n${block.content}\n\n`;
      }
    }
    return output;
  }

  readPhase(projectId: string, phase: string): string | null {
    const db = this.open();
    const row = db.prepare('SELECT blocks_json FROM projects WHERE id = ?').get(projectId) as DbProject | undefined;
    if (!row) return null;

    const blocks = this.parseBlocks(row.blocks_json);
    const sddMeta = this.extractSddMeta(blocks);
    if (!sddMeta) return null;

    return sddMeta.phases[phase as keyof typeof sddMeta.phases] || null;
  }

  readTasks(projectId: string): ParsedTask[] {
    const tasksContent = this.readPhase(projectId, 'tasks');
    if (!tasksContent) return [];
    return this.parseTasks(tasksContent);
  }

  search(query: string): { projectId: string; projectName: string; match: string }[] {
    const db = this.open();
    const rows = db.prepare('SELECT id, name, blocks_json FROM projects ORDER BY updated_at DESC').all() as DbProject[];
    const results: { projectId: string; projectName: string; match: string }[] = [];
    const q = query.toLowerCase();

    for (const row of rows) {
      const blocks = this.parseBlocks(row.blocks_json);
      const sddMeta = this.extractSddMeta(blocks);

      if (sddMeta) {
        for (const [phase, content] of Object.entries(sddMeta.phases)) {
          if (content.toLowerCase().includes(q)) {
            const idx = content.toLowerCase().indexOf(q);
            const start = Math.max(0, idx - 50);
            const end = Math.min(content.length, idx + query.length + 50);
            results.push({
              projectId: row.id,
              projectName: row.name,
              match: `[${phase}] ...${content.slice(start, end)}...`,
            });
          }
        }
      } else {
        for (const block of blocks) {
          if (block.content.toLowerCase().includes(q)) {
            const idx = block.content.toLowerCase().indexOf(q);
            const start = Math.max(0, idx - 50);
            const end = Math.min(block.content.length, idx + query.length + 50);
            results.push({
              projectId: row.id,
              projectName: row.name,
              match: `[${block.type}] ...${block.content.slice(start, end)}...`,
            });
          }
        }
      }
    }

    return results.slice(0, 20);
  }

  // --- Helpers ---

  private parseBlocks(blocksJson: string): Array<{ type: string; content: string; enabled: boolean }> {
    try {
      return JSON.parse(blocksJson || '[]');
    } catch {
      return [];
    }
  }

  private extractSddMeta(blocks: Array<{ type: string; content: string; enabled: boolean }>): { currentPhase: string; phases: Record<string, string> } | null {
    // SDD projects store metadata differently — check if any block has sdd type
    // or if the project has sddMeta in the variables/tags
    // For now, SDD projects use projectType field which isn't in blocks_json
    // The sddMeta is stored as part of the project serialization

    // Try parsing from a special block
    const sddBlock = blocks.find(b => b.type === 'sdd-meta');
    if (sddBlock) {
      try {
        return JSON.parse(sddBlock.content);
      } catch {
        return null;
      }
    }
    return null;
  }

  private parseTasks(content: string): ParsedTask[] {
    const tasks: ParsedTask[] = [];
    const sections = content.split(/^## Task \d+:/gm).slice(1);

    for (let i = 0; i < sections.length; i++) {
      const section = sections[i];
      const title = section.split('\n')[0].trim();

      const files = (section.match(/\*\*Fichier\(s\):\*\*\s*`([^`]+)`/g) || [])
        .map(m => m.match(/`([^`]+)`/)?.[1] || '');

      const deps = section.match(/\*\*Dependances:\*\*\s*(.*)/)?.[1]?.trim() || 'Aucune';
      const complexity = section.match(/\*\*Complexite:\*\*\s*(\w+)/)?.[1] || 'M';
      const criteria = (section.match(/- \[[ x]\].*/g) || []).map(c => c.replace(/^- \[[ x]\]\s*/, ''));
      const notes = section.match(/\*\*Notes:\*\*\s*([\s\S]*?)(?=\n## |$)/)?.[1]?.trim() || '';

      tasks.push({
        id: `task-${i + 1}`,
        title,
        files: files.filter(Boolean),
        dependencies: deps === 'Aucune' ? [] : deps.split(',').map(d => d.trim()),
        complexity,
        criteria,
        notes,
      });
    }

    return tasks;
  }
}
