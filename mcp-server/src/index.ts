#!/usr/bin/env node

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { InkwellDB } from './db.js';

const DB_PATH = process.env.INKWELL_DB_PATH ||
  (process.platform === 'win32'
    ? `${process.env.LOCALAPPDATA || process.env.APPDATA}/inkwell-server/data.db`
    : `${process.env.HOME}/.local/share/inkwell-server/data.db`);

const db = new InkwellDB(DB_PATH);
const server = new McpServer({
  name: 'inkwell',
  version: '0.1.0',
});

// --- Tools ---

server.tool(
  'list_projects',
  'List all Inkwell projects. Use type=spec to filter spec-driven projects only.',
  { type: z.string().optional().describe('Filter by project type: "prompt" or "spec"') },
  async ({ type }) => {
    const projects = db.listProjects(type);
    return {
      content: [{
        type: 'text',
        text: JSON.stringify(projects, null, 2),
      }],
    };
  },
);

server.tool(
  'read_project',
  'Read a full Inkwell project (all blocks or all SDD phases concatenated as markdown).',
  { projectId: z.string().describe('The project ID') },
  async ({ projectId }) => {
    const content = db.readProject(projectId);
    if (!content) {
      return { content: [{ type: 'text', text: 'Project not found' }] };
    }
    return { content: [{ type: 'text', text: content }] };
  },
);

server.tool(
  'read_phase',
  'Read a specific SDD phase from a spec project (constitution, specification, plan, tasks, implementation).',
  {
    projectId: z.string().describe('The project ID'),
    phase: z.enum(['constitution', 'specification', 'plan', 'tasks', 'implementation']).describe('The SDD phase'),
  },
  async ({ projectId, phase }) => {
    const content = db.readPhase(projectId, phase);
    if (!content) {
      return { content: [{ type: 'text', text: 'Phase not found or project is not a spec project' }] };
    }
    return { content: [{ type: 'text', text: content }] };
  },
);

server.tool(
  'read_tasks',
  'Read the tasks from a spec project, parsed into structured task objects.',
  { projectId: z.string().describe('The project ID') },
  async ({ projectId }) => {
    const tasks = db.readTasks(projectId);
    return {
      content: [{
        type: 'text',
        text: JSON.stringify(tasks, null, 2),
      }],
    };
  },
);

server.tool(
  'search_projects',
  'Search across all Inkwell projects by keyword.',
  { query: z.string().describe('Search query') },
  async ({ query }) => {
    const results = db.search(query);
    return {
      content: [{
        type: 'text',
        text: JSON.stringify(results, null, 2),
      }],
    };
  },
);

// --- Start ---

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
