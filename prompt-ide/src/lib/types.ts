export type BlockType = 'role' | 'context' | 'task' | 'examples' | 'constraints' | 'format' | 'sdd-constitution' | 'sdd-specification' | 'sdd-plan' | 'sdd-tasks' | 'sdd-implementation';

export interface PromptBlock {
  id: string;
  type: BlockType;
  content: string;
  enabled: boolean;
}

export interface Workspace {
  id: string;
  name: string;
  description: string;
  color: string;
  constitution?: string;
  userId?: string;
  createdAt: number;
  updatedAt: number;
}

export const WORKSPACE_COLORS = [
  '#6366f1', '#8b5cf6', '#a855f7', '#d946ef',
  '#ec4899', '#f43f5e', '#ef4444', '#f97316',
  '#eab308', '#84cc16', '#22c55e', '#14b8a6',
  '#06b6d4', '#3b82f6', '#6b7280',
];

export interface PromptProject {
  id: string;
  name: string;
  userId?: string;
  workspaceId?: string;
  blocks: PromptBlock[];
  variables: Record<string, string>;
  createdAt: number;
  updatedAt: number;
  tags: string[];
  framework?: string;
}

export interface PromptVersion {
  id: string;
  projectId: string;
  blocks: PromptBlock[];
  variables: Record<string, string>;
  label: string;
  createdAt: number;
}

export interface ExecutionResult {
  id: string;
  projectId: string;
  prompt: string;
  model: string;
  provider: string;
  response: string;
  tokensIn: number;
  tokensOut: number;
  costEstimate: number;
  latencyMs: number;
  temperature: number;
  maxTokens: number;
  createdAt: number;
}

export interface CustomFramework {
  id: string;
  name: string;
  description: string;
  blocks: Omit<PromptBlock, 'id'>[];
  userId?: string;
  createdAt: number;
  updatedAt: number;
}

export interface ApiKeys {
  openai?: string;
  anthropic?: string;
  google?: string;
}

export interface ModelConfig {
  id: string;
  name: string;
  provider: 'openai' | 'anthropic' | 'google' | 'local';
  inputCostPer1k: number;
  outputCostPer1k: number;
  maxContext: number;
  nodeAddress?: string;
  nodeName?: string;
}

export const LOCAL_SERVER_URL_KEY = 'inkwell-local-server-url';

export function getLocalServerUrl(): string {
  return localStorage.getItem(LOCAL_SERVER_URL_KEY) || 'http://localhost:8910';
}

export function setLocalServerUrl(url: string): void {
  localStorage.setItem(LOCAL_SERVER_URL_KEY, url);
}

export const BLOCK_CONFIG: Record<BlockType, { label: string; color: string; icon: string; placeholder: string }> = {
  role: {
    label: 'Role / Persona',
    color: 'var(--color-role)',
    icon: 'user',
    placeholder: 'Tu es un expert en...',
  },
  context: {
    label: 'Contexte',
    color: 'var(--color-context)',
    icon: 'book-open',
    placeholder: "Informations de fond, données, contexte de la tâche...",
  },
  task: {
    label: 'Tâche / Directive',
    color: 'var(--color-task)',
    icon: 'target',
    placeholder: 'Rédige, Analyse, Compare, Génère...',
  },
  examples: {
    label: 'Exemples (Few-shot)',
    color: 'var(--color-examples)',
    icon: 'lightbulb',
    placeholder: '<example>\nInput: ...\nOutput: ...\n</example>',
  },
  constraints: {
    label: 'Contraintes',
    color: 'var(--color-constraints)',
    icon: 'shield',
    placeholder: '- Maximum 500 mots\n- Ton professionnel\n- Format liste',
  },
  format: {
    label: 'Format de sortie',
    color: 'var(--color-format)',
    icon: 'layout',
    placeholder: 'JSON, Markdown, liste numérotée, tableau...',
  },
  'sdd-constitution': {
    label: 'Constitution',
    color: '#a78bfa',
    icon: 'shield',
    placeholder: '# Constitution du Projet\n\n## Nom du Projet\n## Vision\n## Principes Fondamentaux',
  },
  'sdd-specification': {
    label: 'Specification',
    color: '#60a5fa',
    icon: 'book-open',
    placeholder: '# Specification Technique\n\n## Vue d\'Ensemble\n## Modele de Donnees\n## Surface API',
  },
  'sdd-plan': {
    label: 'Plan',
    color: '#34d399',
    icon: 'target',
    placeholder: '# Plan d\'Implementation\n\n## Decisions Architecturales\n## Decomposition en Modules',
  },
  'sdd-tasks': {
    label: 'Tasks',
    color: '#fbbf24',
    icon: 'lightbulb',
    placeholder: '# Liste des Taches\n\n## Task 1:\n- Fichier(s):\n- Complexite: S/M/L/XL',
  },
  'sdd-implementation': {
    label: 'Implementation',
    color: '#f87171',
    icon: 'layout',
    placeholder: '# Notes d\'Implementation\n\n## Taches Completees\n## Tache en Cours',
  },
};

export const MODELS: ModelConfig[] = [
  { id: 'gpt-4o', name: 'GPT-4o', provider: 'openai', inputCostPer1k: 0.0025, outputCostPer1k: 0.01, maxContext: 128000 },
  { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'openai', inputCostPer1k: 0.00015, outputCostPer1k: 0.0006, maxContext: 128000 },
  { id: 'gpt-4.1', name: 'GPT-4.1', provider: 'openai', inputCostPer1k: 0.002, outputCostPer1k: 0.008, maxContext: 1047576 },
  { id: 'gpt-4.1-mini', name: 'GPT-4.1 Mini', provider: 'openai', inputCostPer1k: 0.0004, outputCostPer1k: 0.0016, maxContext: 1047576 },
  { id: 'gpt-4.1-nano', name: 'GPT-4.1 Nano', provider: 'openai', inputCostPer1k: 0.0001, outputCostPer1k: 0.0004, maxContext: 1047576 },
  { id: 'o3-mini', name: 'o3-mini', provider: 'openai', inputCostPer1k: 0.0011, outputCostPer1k: 0.0044, maxContext: 200000 },
  { id: 'claude-sonnet-4-6', name: 'Claude Sonnet 4.6', provider: 'anthropic', inputCostPer1k: 0.003, outputCostPer1k: 0.015, maxContext: 200000 },
  { id: 'claude-opus-4-6', name: 'Claude Opus 4.6', provider: 'anthropic', inputCostPer1k: 0.015, outputCostPer1k: 0.075, maxContext: 1000000 },
  { id: 'claude-haiku-4-5', name: 'Claude Haiku 4.5', provider: 'anthropic', inputCostPer1k: 0.0008, outputCostPer1k: 0.004, maxContext: 200000 },
  { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro', provider: 'google', inputCostPer1k: 0.00125, outputCostPer1k: 0.01, maxContext: 1048576 },
  { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash', provider: 'google', inputCostPer1k: 0.00015, outputCostPer1k: 0.0006, maxContext: 1048576 },
];

export const FRAMEWORKS: Record<string, { name: string; description: string; blocks: Omit<PromptBlock, 'id'>[] }> = {
  'co-star': {
    name: 'CO-STAR',
    description: 'Context, Objective, Style, Tone, Audience, Response',
    blocks: [
      { type: 'context', content: '## Contexte\n', enabled: true },
      { type: 'task', content: '## Objectif\n', enabled: true },
      { type: 'role', content: '## Style\n', enabled: true },
      { type: 'constraints', content: '## Ton\n', enabled: true },
      { type: 'format', content: '## Audience\n', enabled: true },
      { type: 'format', content: '## Format de réponse\n', enabled: true },
    ],
  },
  risen: {
    name: 'RISEN',
    description: 'Role, Instructions, Steps, End Goal, Narrowing',
    blocks: [
      { type: 'role', content: '## Rôle\n', enabled: true },
      { type: 'task', content: '## Instructions\n', enabled: true },
      { type: 'task', content: '## Étapes\n1. \n2. \n3. ', enabled: true },
      { type: 'format', content: '## Objectif final\n', enabled: true },
      { type: 'constraints', content: '## Restrictions\n', enabled: true },
    ],
  },
  race: {
    name: 'RACE',
    description: 'Role, Action, Context, Expect',
    blocks: [
      { type: 'role', content: '## Rôle\n', enabled: true },
      { type: 'task', content: '## Action\n', enabled: true },
      { type: 'context', content: '## Contexte\n', enabled: true },
      { type: 'format', content: '## Résultat attendu\n', enabled: true },
    ],
  },
  create: {
    name: 'CREATE',
    description: 'Character, Request, Examples, Adjustments, Type, Extras',
    blocks: [
      { type: 'role', content: '## Personnage\n', enabled: true },
      { type: 'task', content: '## Requête\n', enabled: true },
      { type: 'examples', content: '## Exemples\n<example>\nInput: \nOutput: \n</example>', enabled: true },
      { type: 'constraints', content: '## Ajustements\n', enabled: true },
      { type: 'format', content: '## Type de sortie\n', enabled: true },
      { type: 'context', content: '## Extras\n', enabled: true },
    ],
  },
  ape: {
    name: 'APE',
    description: 'Action, Purpose, Expectation',
    blocks: [
      { type: 'task', content: '## Action\n', enabled: true },
      { type: 'context', content: '## But\n', enabled: true },
      { type: 'format', content: '## Résultat attendu\n', enabled: true },
    ],
  },
  stoke: {
    name: 'STOKE',
    description: 'Situation, Task, Objective, Knowledge, Examples',
    blocks: [
      { type: 'context', content: '## Situation\n', enabled: true },
      { type: 'task', content: '## Tâche\n', enabled: true },
      { type: 'format', content: '## Objectif\n', enabled: true },
      { type: 'context', content: '## Connaissances\n', enabled: true },
      { type: 'examples', content: '## Exemples\n<example>\n\n</example>', enabled: true },
    ],
  },
  sdd: {
    name: 'SDD (Spec-Driven)',
    description: 'Constitution → Specification → Plan → Tasks → Implementation (Spec Kit compatible)',
    blocks: [
      { type: 'sdd-constitution' as BlockType, content: '# Project Constitution\n\n## Core Principles\n\n### Principle 1\nDescription.\n\n### Principle 2\nDescription.\n\n### Principle 3\nDescription.\n\n## Technical Stack\n- **Language/Version:**\n- **Framework:**\n- **Database:**\n- **Testing:**\n\n## Coding Conventions\n- **Style:**\n- **Tests:**\n\n## Governance\n- Review process\n- Branching strategy\n\n**Version**: 1.0.0', enabled: true },
      { type: 'sdd-specification' as BlockType, content: '# Feature Specification\n\n**Status**: Draft\n\n## User Scenarios & Testing\n\n### User Story 1 - [Title] (Priority: P1)\n[Description]\n**Acceptance Scenarios**:\n1. **Given** [...], **When** [...], **Then** [...]\n\n### Edge Cases\n- What happens when [...]?\n\n## Requirements\n\n### Functional Requirements\n- **FR-001**: System MUST [...]\n- **FR-002**: System MUST [...]\n\n### Key Entities\n- **Entity 1**: [...]\n\n## Success Criteria\n- **SC-001**: [Measurable metric]\n- **SC-002**: [Measurable metric]\n\n## Assumptions\n- [...]', enabled: true },
      { type: 'sdd-plan' as BlockType, content: '# Implementation Plan\n\n## Summary\n[Overview]\n\n## Technical Context\n- **Language/Version:**\n- **Dependencies:**\n- **Performance Goals:**\n\n## Constitution Check\n- [ ] Principle 1: [how respected]\n- [ ] Principle 2: [how respected]\n\n## Architecture Decisions\n\n### ADR-001: [Title]\n- **Context**: [...]\n- **Decision**: [...]\n- **Consequences**: [...]\n\n## Module Breakdown\n\n### Module: [Name]\n- **Responsibility**: [...]\n- **Files**: `src/path/`\n\n## Risk Assessment\n| Risk | Probability | Impact | Mitigation |\n|------|------------|--------|------------|', enabled: true },
      { type: 'sdd-tasks' as BlockType, content: '# Task Breakdown\n\n## Phase 1: Setup\n- [ ] T001 Create project structure\n- [ ] T002 Configure dependencies\n\n## Phase 2: Foundational\n- [ ] T003 [P] Create data models in `src/models/`\n- [ ] T004 [P] Setup database in `src/db/`\n- [ ] T005 Implement base services in `src/services/`\n\n## Phase 3: User Stories\n\n### Story US1 - [Title] (P1)\n- [ ] T006 [P] [US1] Create model `src/models/file.ts`\n- [ ] T007 [US1] Create service `src/services/file.ts`\n- [ ] T008 [US1] Create route `src/routes/file.ts`\n- [ ] T009 [US1] Write tests `tests/file.test.ts`\n\n## Phase 4: Polish\n- [ ] T010 Error handling\n- [ ] T011 Integration tests\n- [ ] T012 Documentation', enabled: true },
      { type: 'sdd-implementation' as BlockType, content: '# Implementation Notes\n\n## Completed Tasks\n\n## Current Task\n\n## Deviations from Spec\n| Change | Reason | Impact |\n|--------|--------|--------|\n\n## Blockers & Open Questions\n- [ ] [...]\n\n## Retrospective\n- **What worked**: [...]\n- **What to improve**: [...]', enabled: true },
    ],
  },
};
