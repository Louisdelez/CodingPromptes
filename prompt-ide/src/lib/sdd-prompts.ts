/**
 * SDD System Prompts — Based on Spec Kit's actual command logic
 * Implements: Generate, Improve, Clarify, Analyze, Checklist, Run All
 */

import type { PromptBlock, ModelConfig, ApiKeys } from './types';
import { SDD_PHASE_ORDER, SDD_PHASE_LABELS, type SddPhase } from './sdd-conventions';
import { callLLM } from './api';
import { executeHooks } from './sdd-hooks';
import { getPresetInstructions } from './sdd-presets';
import { commitPhase as gitCommitPhase } from './git';

// ========================
// Context Building
// ========================

function getPreviousPhases(blocks: PromptBlock[], currentType: string, workspaceConstitution?: string): string {
  const idx = SDD_PHASE_ORDER.indexOf(currentType as SddPhase);
  if (idx <= 0) return '';

  let ctx = '';
  for (let i = 0; i < idx; i++) {
    const phase = SDD_PHASE_ORDER[i];
    const block = blocks.find(b => b.type === phase && b.enabled);
    let content = block?.content.trim() || '';

    // If constitution block is empty/placeholder but workspace has one, use workspace's
    if (phase === 'sdd-constitution' && (!content || content.includes('[Title]') || content.length < 100) && workspaceConstitution) {
      content = workspaceConstitution;
    }

    if (content) {
      ctx += `\n### ${SDD_PHASE_LABELS[phase].en}\n${content}\n`;
    }
  }
  return ctx ? `\n--- PREVIOUS PHASES ---\n${ctx}\n--- END PREVIOUS PHASES ---\n` : '';
}

// ========================
// Phase-specific instructions (matching Spec Kit logic)
// ========================

const CONSTITUTION_PROMPT = `You are writing a PROJECT CONSTITUTION — the immutable governing document.

RULES:
- Core Principles must be NON-NEGOTIABLE design rules (3-5 principles)
- Each principle has a name and concrete description
- Technical Stack must specify exact languages, frameworks, versions
- Governance defines review/branching/deployment rules
- Include a version number and date
- Keep it concise — this is a reference document, not a novel

OUTPUT FORMAT: Follow the exact markdown structure with # Project Constitution, ## Core Principles, ## Technical Stack, ## Coding Conventions, ## Governance`;

const SPECIFICATION_PROMPT = `You are writing a FEATURE SPECIFICATION based on the constitution.

RULES (from Spec Kit):
- User Stories are prioritized P1 (critical), P2 (important), P3 (nice-to-have)
- Each User Story must be independently testable
- Acceptance Scenarios use Given/When/Then format
- Functional Requirements use FR-001, FR-002... numbering
- Success Criteria use SC-001, SC-002... numbering and must be MEASURABLE
- Max 3 [NEEDS CLARIFICATION] markers allowed
- NO implementation details — only WHAT, not HOW
- Key Entities describe data objects without implementation
- Edge Cases section is mandatory

OUTPUT FORMAT: Follow the exact structure with # Feature Specification, ## User Scenarios & Testing, ## Requirements, ## Success Criteria`;

const PLAN_PROMPT = `You are writing an IMPLEMENTATION PLAN based on the constitution and specification.

RULES (from Spec Kit):
- Start with a Constitution Check: verify each principle is respected
- Architecture Decisions use ADR format (Context, Decision, Consequences, Alternatives Rejected)
- Module Breakdown lists responsibility, files, and dependencies
- Project Structure shows the actual directory tree
- Risk Assessment uses a table with Probability/Impact/Mitigation
- Technical Context specifies exact tools, versions, constraints
- Include a Summary (1-2 paragraphs)

OUTPUT FORMAT: Follow the exact structure with # Implementation Plan, ## Summary, ## Technical Context, ## Constitution Check, ## Architecture Decisions, ## Module Breakdown`;

const TASKS_PROMPT = `You are decomposing the plan into TASKS following the Spec Kit task format.

STRICT TASK FORMAT (each line MUST follow this):
- [ ] T001 [P] [US1] Description with exact file path \`src/path/file.ts\`

RULES:
- T001, T002... sequential IDs (mandatory)
- [P] = parallelizable (different files, no deps) — only add when truly parallel
- [US1], [US2] = user story label (only for story phases, NOT setup/foundational/polish)
- Every task MUST include the exact file path in backticks
- Complexity: keep each task to ONE commit
- Phase structure:
  Phase 1: Setup (project init, config)
  Phase 2: Foundational (BLOCKS all stories — models, DB, base services)
  Phase 3: User Stories (grouped by US label, P1 first)
  Phase 4: Polish & Cross-Cutting (error handling, tests, docs)
- Within stories: Models → Services → Routes/Endpoints → Tests
- Dependencies are implicit from phase ordering`;

const IMPLEMENTATION_PROMPT = `You are updating IMPLEMENTATION NOTES to track progress.

RULES:
- Completed tasks include commit hash: - [x] T001 — commit \`abc1234\`
- Current task shows status and notes
- Deviations from spec are documented with reason and impact
- Blockers and open questions use checkboxes
- Retrospective captures lessons learned`;

// ========================
// Prompt Builders
// ========================

export function buildGeneratePrompt(blocks: PromptBlock[], blockType: string, workspaceConstitution?: string): {
  systemPrompt: string;
  userPrompt: string;
} {
  const prev = getPreviousPhases(blocks, blockType, workspaceConstitution);
  const instructions: Record<string, string> = {
    'sdd-constitution': CONSTITUTION_PROMPT,
    'sdd-specification': SPECIFICATION_PROMPT,
    'sdd-plan': PLAN_PROMPT,
    'sdd-tasks': TASKS_PROMPT,
    'sdd-implementation': IMPLEMENTATION_PROMPT,
  };

  const presetExtra = getPresetInstructions(blockType as SddPhase);
  const sysPrompt = [instructions[blockType] || '', presetExtra].filter(Boolean).join('\n\n');

  return {
    systemPrompt: sysPrompt,
    userPrompt: prev
      ? `Based on the previous phases below, generate the complete content for this phase.\n${prev}`
      : 'Generate the initial content for this phase. Use placeholder text where specific project details are needed.',
  };
}

export function buildImprovePrompt(blocks: PromptBlock[], blockType: string, content: string, instruction?: string, workspaceConstitution?: string): {
  systemPrompt: string;
  userPrompt: string;
} {
  const prev = getPreviousPhases(blocks, blockType, workspaceConstitution);
  const instructions: Record<string, string> = {
    'sdd-constitution': CONSTITUTION_PROMPT,
    'sdd-specification': SPECIFICATION_PROMPT,
    'sdd-plan': PLAN_PROMPT,
    'sdd-tasks': TASKS_PROMPT,
    'sdd-implementation': IMPLEMENTATION_PROMPT,
  };

  let userPrompt = '';
  if (instruction) userPrompt += `USER INSTRUCTION: ${instruction}\n\n`;
  if (prev) userPrompt += `CONTEXT:\n${prev}\n\n`;
  userPrompt += `CURRENT CONTENT TO IMPROVE:\n${content}\n\nImprove this content: fix formatting, add missing sections, make it more precise and concrete. Keep the user's intent but enhance quality.`;

  return { systemPrompt: instructions[blockType] || '', userPrompt };
}

export function buildClarifyPrompt(blocks: PromptBlock[], blockType: string, content: string, workspaceConstitution?: string): {
  systemPrompt: string;
  userPrompt: string;
} {
  const prev = getPreviousPhases(blocks, blockType, workspaceConstitution);
  const label = SDD_PHASE_LABELS[blockType as SddPhase]?.en || blockType;

  return {
    systemPrompt: `You are a technical reviewer performing a structured ambiguity scan.

Scan across these categories (from Spec Kit):
1. Functional Scope & Behavior
2. Domain & Data Model
3. Interaction & UX Flow
4. Non-Functional Quality Attributes (performance, security, observability)
5. Integration & External Dependencies
6. Edge Cases & Failure Handling
7. Constraints & Tradeoffs
8. Terminology & Consistency
9. Completion Signals
10. Placeholders & TODOs

For each issue found:
1. Quote the section
2. Explain what's missing or ambiguous
3. Ask a precise question with a recommended answer

Output max 5 prioritized questions. Only high-impact issues.`,
    userPrompt: `${prev ? `CONTEXT:\n${prev}\n\n` : ''}DOCUMENT TO ANALYZE (${label}):\n${content}`,
  };
}

export function buildValidatePrompt(blocks: PromptBlock[]): {
  systemPrompt: string;
  userPrompt: string;
} {
  return {
    systemPrompt: `You are a quality auditor performing cross-artifact consistency analysis (from Spec Kit's /speckit.analyze).

Run these detection passes:
A. Duplication Detection
B. Ambiguity Detection (vague adjectives, unresolved placeholders)
C. Underspecification
D. Constitution Alignment (CRITICAL if MUST principles violated)
E. Coverage Gaps (requirements with zero tasks, tasks with no requirement)
F. Inconsistency (terminology drift, ordering contradictions)

For each finding, assign severity: CRITICAL / HIGH / MEDIUM / LOW

Output format:
- COHERENT: [element] — well traced from [phase A] to [phase B]
- MISSING: [element] in [phase A] not covered in [phase B]
- CONTRADICTION: [description]
- RECOMMENDATION: [suggestion]

End with a Coverage Summary table and overall score.`,
    userPrompt: (() => {
      let content = 'PHASES:\n\n';
      for (const phase of SDD_PHASE_ORDER) {
        const block = blocks.find(b => b.type === phase && b.enabled);
        if (block?.content.trim()) {
          content += `### ${SDD_PHASE_LABELS[phase].en}\n${block.content}\n\n---\n\n`;
        }
      }
      return content + '\nAnalyze cross-phase consistency.';
    })(),
  };
}

export function buildChecklistPrompt(blocks: PromptBlock[]): {
  systemPrompt: string;
  userPrompt: string;
} {
  return {
    systemPrompt: `You generate quality validation checklists — "Unit Tests for English" (from Spec Kit's /speckit.checklist).

RULES:
- Validate REQUIREMENTS QUALITY, not implementation behavior
- Format: - [ ] CHK001 [Quality Dimension, Spec section] Question
- PROHIBITED patterns: "Verify", "Test", "Confirm" + implementation behavior
- REQUIRED patterns: "Are [X] defined?", "Is [vague term] quantified?", "Can [X] be measured?"

Categories:
- Requirement Completeness
- Clarity & Precision
- Consistency
- Acceptance Criteria Quality
- Scenario Coverage
- Edge Case Coverage
- Non-Functional Requirements
- Dependencies & Assumptions`,
    userPrompt: (() => {
      let content = 'DOCUMENTS:\n\n';
      for (const phase of SDD_PHASE_ORDER) {
        const block = blocks.find(b => b.type === phase && b.enabled);
        if (block?.content.trim()) {
          content += `### ${SDD_PHASE_LABELS[phase].en}\n${block.content}\n\n`;
        }
      }
      return content + '\nGenerate a comprehensive quality checklist.';
    })(),
  };
}

// ========================
// Run All Cascade
// ========================

export async function runAllCascade(
  blocks: PromptBlock[],
  model: ModelConfig,
  apiKeys: ApiKeys,
  description: string,
  onPhaseStart: (phase: SddPhase) => void,
  onPhaseComplete: (phase: SddPhase, content: string) => void,
  onError: (phase: SddPhase, error: string) => void,
  workspaceConstitution?: string,
  autoCommit?: boolean,
): Promise<void> {
  const workingBlocks = blocks.map(b => ({ ...b }));

  for (const phase of SDD_PHASE_ORDER) {
    onPhaseStart(phase);
    await executeHooks(phase, 'before');

    const blockIdx = workingBlocks.findIndex(b => b.type === phase);
    if (blockIdx === -1) continue;

    const instructions: Record<string, string> = {
      'sdd-constitution': CONSTITUTION_PROMPT,
      'sdd-specification': SPECIFICATION_PROMPT,
      'sdd-plan': PLAN_PROMPT,
      'sdd-tasks': TASKS_PROMPT,
      'sdd-implementation': IMPLEMENTATION_PROMPT,
    };

    const prev = getPreviousPhases(workingBlocks, phase, workspaceConstitution);
    const systemPrompt = instructions[phase] || '';

    let userPrompt: string;
    if (phase === 'sdd-constitution') {
      const presetStack = getPresetInstructions('sdd-constitution');
      const { getPresetTechStack } = await import('./sdd-presets');
      const techStack = getPresetTechStack();
      userPrompt = `Project description: ${description}\n\n${techStack ? `Suggested tech stack:\n${techStack}\n\n` : ''}${presetStack ? `Additional context:\n${presetStack}\n\n` : ''}Generate the complete constitution for this project.`;
    } else if (phase === 'sdd-implementation') {
      userPrompt = `${prev}\n\nGenerate initial implementation notes based on the tasks above. Mark all tasks as pending.`;
    } else {
      userPrompt = `${prev}\n\nBased on the previous phases, generate the complete content for this phase.`;
    }

    try {
      const result = await callLLM(userPrompt, model, apiKeys, {
        systemPrompt,
        temperature: 0.3,
        maxTokens: 4096,
      });

      if (result.text) {
        workingBlocks[blockIdx].content = result.text;
        onPhaseComplete(phase, result.text);
        await executeHooks(phase, 'after', { content: result.text });
        if (autoCommit) {
          try { await gitCommitPhase(phase); } catch { /* git not available */ }
        }
      }
    } catch (err) {
      onError(phase, err instanceof Error ? err.message : 'Unknown error');
      return; // Stop cascade on error
    }
  }
}
