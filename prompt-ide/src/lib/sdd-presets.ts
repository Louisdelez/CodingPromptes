/**
 * SDD Presets — tech stack-specific template modifications
 * Stackable: multiple presets can be active simultaneously
 */

import type { SddPhase } from './sdd-conventions';

export interface SddPreset {
  id: string;
  name: string;
  description: string;
  icon: string;
  // Extra instructions injected into LLM prompts per phase
  phaseInstructions: Partial<Record<SddPhase, string>>;
  // Tech stack defaults for constitution
  techStack: string;
}

export const BUILTIN_PRESETS: SddPreset[] = [
  {
    id: 'react-nextjs',
    name: 'React / Next.js',
    description: 'Frontend web avec React, Next.js, TypeScript, Tailwind',
    icon: 'R',
    techStack: '- **Language**: TypeScript 5.x\n- **Framework**: Next.js 15 (App Router)\n- **Styling**: Tailwind CSS 4\n- **State**: React hooks + Context\n- **Testing**: Vitest + Testing Library\n- **Package Manager**: pnpm',
    phaseInstructions: {
      'sdd-specification': 'This is a React/Next.js project. Specs should include: page routes, components hierarchy, data fetching strategy (SSR/SSG/CSR), API routes, middleware.',
      'sdd-plan': 'Architecture should follow Next.js App Router conventions: app/ directory, layout.tsx, page.tsx, server components by default, client components with "use client".',
      'sdd-tasks': 'Files follow Next.js conventions: app/page.tsx, app/layout.tsx, components/*.tsx, lib/*.ts, api/route.ts. Group tasks by route/feature.',
    },
  },
  {
    id: 'rust-axum',
    name: 'Rust / Axum',
    description: 'Backend API avec Rust, Axum, SQLite, Tokio',
    icon: 'Rs',
    techStack: '- **Language**: Rust (stable)\n- **Framework**: Axum 0.8\n- **Runtime**: Tokio\n- **Database**: SQLite (rusqlite)\n- **Serialization**: serde + serde_json\n- **Testing**: cargo test\n- **Auth**: JWT (jsonwebtoken)',
    phaseInstructions: {
      'sdd-specification': 'This is a Rust/Axum API. Specs should include: route handlers, extractors, shared state (Arc), error types, middleware chain.',
      'sdd-plan': 'Architecture follows Axum patterns: mod.rs modules, Router composition, State with Arc<AppState>, tower middleware layers.',
      'sdd-tasks': 'Files follow Rust conventions: src/main.rs, src/routes/*.rs, src/models/*.rs, src/db.rs. Each module in its own file.',
    },
  },
  {
    id: 'python-fastapi',
    name: 'Python / FastAPI',
    description: 'Backend API avec Python, FastAPI, SQLAlchemy, Pydantic',
    icon: 'Py',
    techStack: '- **Language**: Python 3.12+\n- **Framework**: FastAPI\n- **ORM**: SQLAlchemy 2.0\n- **Validation**: Pydantic v2\n- **Testing**: pytest + httpx\n- **Package Manager**: uv',
    phaseInstructions: {
      'sdd-specification': 'This is a FastAPI project. Specs should include: endpoint decorators, Pydantic models, dependency injection, middleware.',
      'sdd-plan': 'Architecture follows FastAPI conventions: app/main.py, app/routers/, app/models/, app/schemas/, app/dependencies.py.',
      'sdd-tasks': 'Files follow FastAPI conventions: app/main.py, app/routers/*.py, app/models/*.py, app/schemas/*.py, tests/test_*.py.',
    },
  },
  {
    id: 'mobile-react-native',
    name: 'React Native / Expo',
    description: 'Application mobile avec React Native, Expo, TypeScript',
    icon: 'RN',
    techStack: '- **Language**: TypeScript\n- **Framework**: React Native + Expo SDK 52\n- **Navigation**: Expo Router\n- **State**: Zustand\n- **Testing**: Jest + React Native Testing Library\n- **CI**: EAS Build',
    phaseInstructions: {
      'sdd-specification': 'This is a React Native/Expo app. Specs should include: screens, navigation flow, native modules, platform differences (iOS/Android), offline support.',
      'sdd-plan': 'Architecture follows Expo Router: app/(tabs)/, app/(auth)/, components/, hooks/, lib/. Consider platform-specific code.',
      'sdd-tasks': 'Files follow Expo conventions: app/*.tsx (screens), components/*.tsx, hooks/use*.ts, lib/*.ts. Note platform differences.',
    },
  },
  {
    id: 'fullstack-t3',
    name: 'T3 Stack',
    description: 'Fullstack avec Next.js, tRPC, Prisma, NextAuth, Tailwind',
    icon: 'T3',
    techStack: '- **Language**: TypeScript\n- **Framework**: Next.js 15 (App Router)\n- **API**: tRPC v11\n- **ORM**: Prisma\n- **Auth**: NextAuth.js v5\n- **Styling**: Tailwind CSS 4\n- **Database**: PostgreSQL',
    phaseInstructions: {
      'sdd-specification': 'This is a T3 stack project. Specs should include: tRPC procedures (queries/mutations), Prisma schema, NextAuth providers, protected routes.',
      'sdd-plan': 'Architecture follows T3 conventions: server/api/routers/, server/db.ts, prisma/schema.prisma. Type-safe end-to-end via tRPC.',
      'sdd-tasks': 'Files follow T3 conventions: server/api/routers/*.ts, prisma/schema.prisma, app/api/trpc/[trpc]/route.ts. Schema migrations first.',
    },
  },
];

const PRESETS_KEY = 'inkwell-sdd-active-presets';

export function getActivePresets(): string[] {
  try {
    return JSON.parse(localStorage.getItem(PRESETS_KEY) || '[]');
  } catch { return []; }
}

export function setActivePresets(ids: string[]): void {
  localStorage.setItem(PRESETS_KEY, JSON.stringify(ids));
}

export function togglePreset(id: string): string[] {
  const active = getActivePresets();
  const next = active.includes(id) ? active.filter(x => x !== id) : [...active, id];
  setActivePresets(next);
  return next;
}

/**
 * Get merged instructions from all active presets for a given phase
 */
export function getPresetInstructions(phase: SddPhase): string {
  const active = getActivePresets();
  return BUILTIN_PRESETS
    .filter(p => active.includes(p.id))
    .map(p => p.phaseInstructions[phase])
    .filter(Boolean)
    .join('\n\n');
}

/**
 * Get merged tech stack from all active presets
 */
export function getPresetTechStack(): string {
  const active = getActivePresets();
  return BUILTIN_PRESETS
    .filter(p => active.includes(p.id))
    .map(p => p.techStack)
    .join('\n\n');
}
