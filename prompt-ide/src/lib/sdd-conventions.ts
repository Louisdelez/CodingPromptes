/**
 * Spec-Driven Development (SDD) — Strict Conventions & Templates
 * Based on GitHub Spec Kit format (100% compatible)
 */

export type SddPhase = 'sdd-constitution' | 'sdd-specification' | 'sdd-plan' | 'sdd-tasks' | 'sdd-implementation';

export const SDD_PHASE_ORDER: SddPhase[] = [
  'sdd-constitution', 'sdd-specification', 'sdd-plan', 'sdd-tasks', 'sdd-implementation',
];

export const SDD_PHASE_LABELS: Record<SddPhase, { fr: string; en: string }> = {
  'sdd-constitution': { fr: 'Constitution', en: 'Constitution' },
  'sdd-specification': { fr: 'Specification', en: 'Specification' },
  'sdd-plan': { fr: 'Plan', en: 'Plan' },
  'sdd-tasks': { fr: 'Taches', en: 'Tasks' },
  'sdd-implementation': { fr: 'Implementation', en: 'Implementation' },
};

/**
 * Templates matching Spec Kit's exact format
 */
export const SDD_TEMPLATES: Record<SddPhase, string> = {
  'sdd-constitution': `# Project Constitution

## Core Principles

### Principle 1
Description of the first non-negotiable principle.

### Principle 2
Description of the second non-negotiable principle.

### Principle 3
Description of the third non-negotiable principle.

## Technical Stack
- **Language/Version:**
- **Framework:**
- **Database:**
- **Testing:**
- **Target Platform:**

## Coding Conventions
- **Style:**
- **Tests:**
- **Documentation:**

## Governance
- Review process
- Branching strategy
- Deployment rules

**Version**: 1.0.0 | **Ratified**: ${new Date().toISOString().split('T')[0]}
`,

  'sdd-specification': `# Feature Specification

**Status**: Draft
**Created**: ${new Date().toISOString().split('T')[0]}

## User Scenarios & Testing

### User Story 1 - [Title] (Priority: P1)
[Description]
**Why this priority**: [...]
**Independent Test**: [...]
**Acceptance Scenarios**:
1. **Given** [...], **When** [...], **Then** [...]

### User Story 2 - [Title] (Priority: P2)
[Description]
**Why this priority**: [...]
**Acceptance Scenarios**:
1. **Given** [...], **When** [...], **Then** [...]

### Edge Cases
- What happens when [boundary condition]?

## Requirements

### Functional Requirements
- **FR-001**: System MUST [...]
- **FR-002**: System MUST [...]
- **FR-003**: System SHOULD [...]

### Key Entities
- **Entity 1**: [description, fields, relationships]

## Success Criteria

### Measurable Outcomes
- **SC-001**: [Measurable metric]
- **SC-002**: [Measurable metric]

## Assumptions
- [Assumption 1]
- [Assumption 2]
`,

  'sdd-plan': `# Implementation Plan

## Summary
[1-2 paragraph overview of the implementation approach]

## Technical Context
- **Language/Version:**
- **Primary Dependencies:**
- **Storage:**
- **Testing:**
- **Performance Goals:**
- **Constraints:**

## Constitution Check
- [ ] Principle 1: [how this plan respects it]
- [ ] Principle 2: [how this plan respects it]

## Architecture Decisions

### ADR-001: [Decision Title]
- **Context**: [why this decision is needed]
- **Decision**: [what we decided]
- **Consequences**: [trade-offs]
- **Alternatives Rejected**: [what else was considered]

## Module Breakdown

### Module: [Name]
- **Responsibility**: [what it does]
- **Files**: \`src/path/to/module\`
- **Dependencies**: [other modules]

## Project Structure
\`\`\`
src/
  models/
  services/
  routes/
  middleware/
  utils/
tests/
\`\`\`

## Risk Assessment
| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| [risk] | Low/Med/High | Low/Med/High | [mitigation] |
`,

  'sdd-tasks': `# Task Breakdown

## Phase 1: Setup
- [ ] T001 Create project structure per implementation plan
- [ ] T002 Configure dependencies and tooling

## Phase 2: Foundational (blocks all stories)
- [ ] T003 [P] Create data models in \`src/models/\`
- [ ] T004 [P] Setup database connection in \`src/db/\`
- [ ] T005 Implement base service layer in \`src/services/\`

## Phase 3: User Stories

### Story US1 - [Title] (P1)
- [ ] T006 [P] [US1] Create [model] in \`src/models/file.ts\`
- [ ] T007 [P] [US1] Create [service] in \`src/services/file.ts\`
- [ ] T008 [US1] Create [route/endpoint] in \`src/routes/file.ts\`
- [ ] T009 [US1] Write tests in \`tests/file.test.ts\`

### Story US2 - [Title] (P2)
- [ ] T010 [P] [US2] Create [model] in \`src/models/file.ts\`
- [ ] T011 [US2] Create [service] in \`src/services/file.ts\`

## Phase 4: Polish & Cross-Cutting
- [ ] T012 Add error handling and validation
- [ ] T013 Write integration tests
- [ ] T014 Documentation and cleanup
`,

  'sdd-implementation': `# Implementation Notes

## Progress
<!-- Updated as tasks complete -->

## Completed Tasks
- [x] T001 — commit \`abc1234\`

## Current Task
### T002: [Description]
- **Status**: in-progress
- **Notes**: [...]

## Deviations from Spec
<!-- Document any changes from the original specification -->
| Change | Reason | Impact |
|--------|--------|--------|

## Blockers & Open Questions
- [ ] [Question or blocker]

## Retrospective
- **What worked**: [...]
- **What to improve**: [...]
`,
};

/**
 * Required sections per phase for validation
 */
export const SDD_REQUIRED_SECTIONS: Record<SddPhase, string[]> = {
  'sdd-constitution': ['# Project Constitution', '## Core Principles', '## Technical Stack'],
  'sdd-specification': ['# Feature Specification', '## User Scenarios', '## Requirements', '## Success Criteria'],
  'sdd-plan': ['# Implementation Plan', '## Technical Context', '## Architecture Decisions'],
  'sdd-tasks': ['# Task Breakdown', '## Phase 1', '## Phase 2'],
  'sdd-implementation': ['# Implementation Notes'],
};

export function validatePhase(phase: SddPhase, content: string): { valid: boolean; missing: string[] } {
  const required = SDD_REQUIRED_SECTIONS[phase] || [];
  const missing = required.filter(section => !content.includes(section));
  return { valid: missing.length === 0, missing };
}
