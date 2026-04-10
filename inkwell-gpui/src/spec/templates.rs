//! SpecKit-inspired templates for SDD block generation.
//! These are used as structure guides when generating content via LLM.

pub const CONSTITUTION_TEMPLATE: &str = r#"# {PROJECT} Constitution

## Core Principles

### I. [Principle Name]
[Description of the principle and why it matters]

### II. [Principle Name]
[Description]

### III. [Principle Name]
[Description]

## Constraints
- [Technology constraint]
- [Scope constraint]
- [Performance constraint]

## Governance
- Constitution supersedes all other practices
- Amendments require documentation and justification
- All features must validate against these principles

**Version**: 1.0 | **Created**: {DATE}
"#;

pub const SPEC_TEMPLATE: &str = r#"# Feature Specification: {FEATURE}

**Created**: {DATE}
**Status**: Draft

## User Scenarios & Testing

### User Story 1 - [Title] (Priority: P1)
[Description]

**Why this priority**: [Explanation]
**Independent Test**: [How to test standalone]

**Acceptance Scenarios**:
1. **Given** [state], **When** [action], **Then** [outcome]
2. **Given** [state], **When** [action], **Then** [outcome]

### Edge Cases
- What happens when [boundary condition]?
- How does system handle [error scenario]?

## Requirements

### Functional Requirements
- **FR-001**: System MUST [capability]
- **FR-002**: System MUST [capability]

## Success Criteria
- **SC-001**: [Measurable metric]
- **SC-002**: [Measurable metric]

## Assumptions
- [Assumption about users/scope]
"#;

pub const PLAN_TEMPLATE: &str = r#"# Implementation Plan: {FEATURE}

**Date**: {DATE}

## Summary
[Primary requirement + technical approach]

## Technical Context
**Language/Version**: [e.g., Rust 1.75]
**Primary Dependencies**: [e.g., GPUI, tokio]
**Storage**: [e.g., JSON files, SQLite]
**Testing**: [e.g., cargo test]
**Target Platform**: [e.g., Linux desktop]

## Constitution Check
[Verify alignment with project principles]

## Project Structure
```
src/
  [proposed file layout]
```

## Design Decisions
- [Decision 1]: [Rationale]
- [Decision 2]: [Rationale]

## Error Handling
- [Strategy for error cases]
"#;

pub const TASKS_TEMPLATE: &str = r#"# Tasks: {FEATURE}

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel
- **[Story]**: User story reference (US1, US2...)

## Phase 1: Setup
- [ ] T001 Create project structure
- [ ] T002 Initialize dependencies

## Phase 2: Core Implementation
- [ ] T003 [US1] Implement [core feature]
- [ ] T004 [P] [US1] Add [supporting feature]

## Phase 3: Polish
- [ ] T005 Add error handling
- [ ] T006 [P] Write documentation
"#;

pub const CHECKLIST_TEMPLATE: &str = r#"# Checklist: {FEATURE}

## Requirements Coverage
- [ ] CHK001 All functional requirements implemented
- [ ] CHK002 All acceptance scenarios pass

## Quality
- [ ] CHK003 No compiler warnings
- [ ] CHK004 Error handling complete
- [ ] CHK005 Edge cases covered

## Documentation
- [ ] CHK006 Code comments where needed
- [ ] CHK007 User-facing docs updated
"#;
