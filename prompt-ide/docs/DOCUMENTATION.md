# Prompt IDE — Documentation Technique

> Version 2.0 | Derniere mise a jour : Avril 2026

---

## Table des matieres

| # | Section | Ligne |
|---|---------|-------|
| 1 | [Presentation du projet](#1-presentation-du-projet) | L.45 |
| 2 | [Architecture](#2-architecture) | L.66 |
| 3 | [Installation et demarrage](#3-installation-et-demarrage) | L.153 |
| 4 | [Structure des fichiers](#4-structure-des-fichiers) | L.175 |
| 5 | [Types et modeles de donnees](#5-types-et-modeles-de-donnees) | L.232 |
| 6 | [Base de donnees (IndexedDB)](#6-base-de-donnees-indexeddb) | L.362 |
| 7 | [Modules principaux (lib/)](#7-modules-principaux-lib) | L.423 |
| 8 | [Composants React (components/)](#8-composants-react-components) | L.479 |
| 9 | [Hook principal (usePromptProject)](#9-hook-principal-usepromptproject) | L.592 |
| 10 | [Extensions CodeMirror](#10-extensions-codemirror) | L.642 |
| 11 | [Integration des APIs LLM](#11-integration-des-apis-llm) | L.661 |
| 12 | [Modeles supportes et tarification](#12-modeles-supportes-et-tarification) | L.705 |
| 13 | [Frameworks de prompts](#13-frameworks-de-prompts) | L.749 |
| 14 | [Speech-to-Text (STT)](#14-speech-to-text-stt) | L.775 |
| 15 | [Application Rust — Prompt AI Server](#15-application-rust--prompt-ai-server) | L.845 |
| 16 | [Systeme d'authentification](#16-systeme-dauthentification) | L.1008 |
| 17 | [Internationalisation (i18n)](#17-internationalisation-i18n) | L.1050 |
| 18 | [Themes (Light / Dark / System)](#18-themes-light--dark--system) | L.1075 |
| 19 | [Import / Export](#19-import--export) | L.1123 |
| 20 | [Streaming des reponses](#20-streaming-des-reponses) | L.1148 |
| 21 | [Raccourcis clavier](#21-raccourcis-clavier) | L.1170 |
| 22 | [Historique des executions](#22-historique-des-executions) | L.1182 |
| 23 | [Chainage de prompts (Workflows)](#23-chainage-de-prompts-workflows) | L.1196 |
| 24 | [Mode Conversation (Multi-turn)](#24-mode-conversation-multi-turn) | L.1217 |
| 25 | [Statistiques (Analytics)](#25-statistiques-analytics) | L.1232 |
| 26 | [Responsive Mobile](#26-responsive-mobile) | L.1263 |
| 27 | [PWA (Progressive Web App)](#27-pwa-progressive-web-app) | L.1281 |
| 28 | [Docker](#28-docker) | L.1305 |
| 29 | [Scripts et commandes](#29-scripts-et-commandes) | L.1352 |
| 30 | [Dependances](#30-dependances) | L.1363 |
| 31 | [Deploiement](#31-deploiement) | L.1388 |

---

## 1. Presentation du projet

Prompt IDE est une application web mono-page (SPA) conçue pour creer, editer, tester et optimiser des prompts pour les modeles de langage (LLM). Elle fonctionne entierement dans le navigateur avec stockage local.

### Stack technique

| Couche | Technologie | Version |
|--------|------------|---------|
| Framework UI | React | 19.x |
| Langage | TypeScript | 6.x |
| Build | Vite | 8.x |
| CSS | Tailwind CSS | 4.x |
| Editeur de code | CodeMirror | 6.x |
| Drag & Drop | dnd-kit | 6.x / 10.x |
| Base de donnees | Dexie (IndexedDB) | 4.x |
| Tokenizer | gpt-tokenizer | 3.x |
| Icones | Lucide React | 1.x |
| IDs | uuid | 13.x |

---

## 2. Architecture

```
┌────────────────────────────────────────────────────────┐
│                      App.tsx                           │
│  ┌──────────┐  ┌─────────────────┐  ┌──────────────┐  │
│  │  Left     │  │  Center         │  │  Right       │  │
│  │  Panel    │  │  Panel          │  │  Panel       │  │
│  │          │  │                 │  │              │  │
│  │ Library  │  │ PromptBlock[]   │  │ PreviewPanel │  │
│  │ (tree)   │  │   BlockEditor   │  │ Playground   │  │
│  │ Framework│  │   (CodeMirror)  │  │ Optimizer    │  │
│  │ Selector │  │                 │  │ LintingPanel │  │
│  │ Version  │  │ VariablesPanel  │  │ ExportPanel  │  │
│  │ History  │  │                 │  │              │  │
│  └──────────┘  ├─────────────────┤  └──────────────┘  │
│                │  TokenCounter   │                     │
│                └─────────────────┘                     │
└────────────────────────────────────────────────────────┘
         │                │                │
         ▼                ▼                ▼
┌─────────────────────────────────────────────┐
│          usePromptProject (hook)            │
│  State: PromptProject + Workspace CRUD      │
│  Actions: add/remove/update/toggle/reorder  │
│  Persistence: auto-save debounced (500ms)   │
└──────────────────┬──────────────────────────┘
                   │
         ┌─────────┴──────────┐
         ▼                    ▼
┌─────────────────┐  ┌──────────────────┐
│  Dexie/IndexedDB │  │  localStorage    │
│  - workspaces    │  │  - API keys      │
│  - projects      │  └──────────────────┘
│  - versions      │
│  - executions    │
└──────────────────┘
```

### Hierarchie des donnees

```
Workspace (projet/dossier)       ← Regroupe plusieurs prompts
  └── PromptProject (prompt)     ← Un prompt avec ses blocs
        ├── PromptBlock[]        ← Les blocs du prompt
        ├── PromptVersion[]      ← Historique des versions
        └── ExecutionResult[]    ← Historique des executions
```

Un `PromptProject` peut exister sans `Workspace` (prompt libre).
Un `Workspace` peut contenir 0 ou N prompts.

### Architecture reseau (mode local)

```
┌─────────────────────────────────────────┐
│  prompt-ai-server (Rust/Iced)           │
│  http://0.0.0.0:8910                    │
│                                         │
│  /transcribe        → Whisper local     │
│  /v1/chat/completions → proxy Ollama ───┼──► Ollama (localhost:11434)
│  /v1/models           → proxy Ollama    │
│  /ollama/status       → etat Ollama     │
│  /health              → sante globale   │
└──────────────────┬──────────────────────┘
                   │ UNE SEULE URL
         ┌─────────▼──────────┐
         │  App Web            │
         │  (navigateur)       │
         │                     │
         │  Playground → LLM   │  (local via proxy OU cloud direct)
         │  Micro → STT        │  (local via Whisper OU cloud direct)
         └─────────────────────┘
```

L'app web peut fonctionner en mode 100% cloud (APIs directes), en mode 100% local (via prompt-ai-server), ou en mode hybride (local pour certains modeles, cloud pour d'autres).

### Flux de donnees

1. L'utilisateur edite les blocs dans le **Center Panel**
2. Le hook `usePromptProject` met a jour le state React
3. Apres 500ms d'inactivite, le prompt est sauvegarde en IndexedDB
4. Les panneaux lateraux lisent le state pour preview, lint, export, etc.
5. Le Playground compile le prompt et l'envoie soit au serveur local (proxy Ollama), soit aux APIs cloud

---

## 3. Installation et demarrage

```bash
# Cloner ou acceder au repertoire
cd prompt-ide

# Installer les dependances
npm install

# Demarrer en mode developpement
npm run dev
# → http://localhost:5173/

# Build de production
npm run build

# Previsualiser le build de production
npm run preview
```

---

## 4. Structure des fichiers

```
prompt-ide/
├── public/                     # Fichiers statiques
├── dist/                       # Build de production (genere)
├── src/
│   ├── main.tsx                # Point d'entree React
│   ├── App.tsx                 # Composant racine (layout 3 panneaux)
│   ├── index.css               # Styles globaux + theme + CodeMirror
│   │
│   ├── lib/                    # Logique metier pure (sans React)
│   │   ├── types.ts            # Types, constantes, configurations
│   │   ├── db.ts               # Base de donnees Dexie + stockage cles API
│   │   ├── api.ts              # Appels HTTP aux APIs LLM
│   │   ├── prompt.ts           # Compilation de prompt + extraction variables
│   │   ├── tokens.ts           # Comptage de tokens + estimation des couts
│   │   ├── stt.ts              # Speech-to-Text (enregistrement, encodage, transcription)
│   │   └── codemirror-prompt.ts # Extensions CodeMirror (highlighting, autocomplete)
│   │
│   ├── hooks/
│   │   └── usePromptProject.ts # Hook principal de gestion d'etat
│   │
│   └── components/             # Composants React
│       ├── BlockEditor.tsx     # Editeur CodeMirror pour un bloc
│       ├── PromptBlock.tsx     # Bloc draggable (header + editeur)
│       ├── TokenCounter.tsx    # Barre de status (tokens, cout, modele)
│       ├── VariablesPanel.tsx  # Detection et saisie des {{variables}}
│       ├── PreviewPanel.tsx    # Prompt compile en lecture seule
│       ├── Playground.tsx      # Test multi-modeles + parametres API
│       ├── FrameworkSelector.tsx # Selection de frameworks (CO-STAR, etc.)
│       ├── Library.tsx         # Bibliotheque de projets
│       ├── VersionHistory.tsx  # Historique et restauration de versions
│       ├── ExportPanel.tsx     # Export multi-format
│       ├── PromptOptimizer.tsx # Optimisation IA du prompt
│       ├── LintingPanel.tsx    # Validation et bonnes pratiques
│       └── SttSettings.tsx    # Configuration Speech-to-Text
│
├── package.json
├── tsconfig.json
├── vite.config.ts
└── docs/
    ├── DOCUMENTATION.md        # Ce fichier
    └── GUIDE_UTILISATEUR.md    # Guide d'utilisation

prompt-stt-server/                  # Application Rust (serveur AI local — STT + LLM)
├── Cargo.toml
└── src/
    ├── main.rs                     # GUI Iced + point d'entree
    ├── server.rs                   # Serveur HTTP axum (endpoints STT)
    ├── whisper_engine.rs           # Inference whisper-rs (wrapper whisper.cpp)
    ├── models.rs                   # Catalogue et gestion des modeles Whisper
    └── downloader.rs               # Telechargement des modeles depuis HuggingFace
```

---

## 5. Types et modeles de donnees

### Workspace

```typescript
interface Workspace {
  id: string;          // UUID unique
  name: string;        // Nom du projet (dossier)
  description: string; // Description optionnelle
  color: string;       // Couleur d'affichage (hex)
  createdAt: number;   // Timestamp de creation
  updatedAt: number;   // Timestamp de derniere modification
}
```

Un Workspace represente un **projet** (dossier) qui regroupe plusieurs prompts. 15 couleurs sont disponibles via la constante `WORKSPACE_COLORS`.

### CustomFramework

```typescript
interface CustomFramework {
  id: string;                          // UUID unique
  name: string;                        // Nom du framework
  description: string;                 // Description courte
  blocks: Omit<PromptBlock, 'id'>[];   // Blocs-modeles (type, content, enabled)
  createdAt: number;                   // Timestamp de creation
  updatedAt: number;                   // Timestamp de derniere modification
}
```

Un framework custom est un modele reutilisable de structure de prompt. Il contient une liste de blocs pre-definis qui seront instancies (avec de nouveaux IDs) quand l'utilisateur l'applique. Les frameworks custom sont stockes en IndexedDB (table `frameworks`), contrairement aux 6 frameworks integres qui sont en dur dans le code.

### BlockType

```typescript
type BlockType = 'role' | 'context' | 'task' | 'examples' | 'constraints' | 'format';
```

Les 6 types de blocs qui composent un prompt.

### PromptBlock

```typescript
interface PromptBlock {
  id: string;          // UUID unique
  type: BlockType;     // Type du bloc
  content: string;     // Contenu texte du bloc
  enabled: boolean;    // true = inclus dans la compilation
}
```

### PromptProject

```typescript
interface PromptProject {
  id: string;                          // UUID unique
  name: string;                        // Nom du prompt
  workspaceId?: string;                // ID du workspace parent (optionnel)
  blocks: PromptBlock[];               // Liste ordonnee des blocs
  variables: Record<string, string>;   // Valeurs des variables {{}}
  createdAt: number;                   // Timestamp de creation
  updatedAt: number;                   // Timestamp de derniere modification
  tags: string[];                      // Tags de categorisation
  framework?: string;                  // Framework utilise (optionnel)
}
```

Si `workspaceId` est `undefined`, le prompt est un "prompt libre" (non range dans un projet).

### PromptVersion

```typescript
interface PromptVersion {
  id: string;                          // UUID unique
  projectId: string;                   // ID du projet parent
  blocks: PromptBlock[];               // Snapshot des blocs
  variables: Record<string, string>;   // Snapshot des variables
  label: string;                       // Label descriptif (ex: "v1 - brouillon")
  createdAt: number;                   // Timestamp
}
```

### ExecutionResult

```typescript
interface ExecutionResult {
  id: string;
  projectId: string;
  prompt: string;        // Prompt compile envoye
  model: string;         // ID du modele (ex: "gpt-4o")
  provider: string;      // "openai" | "anthropic" | "google"
  response: string;      // Reponse du modele
  tokensIn: number;      // Tokens en entree
  tokensOut: number;     // Tokens en sortie
  costEstimate: number;  // Cout estime en USD
  latencyMs: number;     // Temps de reponse en ms
  temperature: number;
  maxTokens: number;
  createdAt: number;
}
```

### ModelConfig

```typescript
interface ModelConfig {
  id: string;                                  // ID API du modele
  name: string;                                // Nom d'affichage
  provider: 'openai' | 'anthropic' | 'google'; // Fournisseur
  inputCostPer1k: number;                      // Cout en $ par 1000 tokens input
  outputCostPer1k: number;                     // Cout en $ par 1000 tokens output
  maxContext: number;                           // Taille max du contexte en tokens
}
```

### BLOCK_CONFIG

Configuration visuelle de chaque type de bloc :

| Type | Label | Couleur | Icone | Placeholder |
|------|-------|---------|-------|-------------|
| role | Role / Persona | Violet (#a78bfa) | user | "Tu es un expert en..." |
| context | Contexte | Bleu (#60a5fa) | book-open | "Informations de fond..." |
| task | Tache / Directive | Vert (#34d399) | target | "Redige, Analyse, Compare..." |
| examples | Exemples (Few-shot) | Ambre (#fbbf24) | lightbulb | "\<example\>..." |
| constraints | Contraintes | Rouge (#f87171) | shield | "- Maximum 500 mots..." |
| format | Format de sortie | Gris (#94a3b8) | layout | "JSON, Markdown, liste..." |

---

## 6. Base de donnees (IndexedDB)

### Schema Dexie

```typescript
class PromptIdeDB extends Dexie {
  workspaces!: EntityTable<Workspace, 'id'>;
  projects!: EntityTable<PromptProject, 'id'>;
  versions!: EntityTable<PromptVersion, 'id'>;
  executions!: EntityTable<ExecutionResult, 'id'>;

  constructor() {
    super('PromptIdeDB');

    this.version(1).stores({
      projects:   'id, name, updatedAt, *tags',
      versions:   'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
    });

    this.version(2).stores({
      workspaces: 'id, name, updatedAt',
      projects:   'id, name, workspaceId, updatedAt, *tags',
      versions:   'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
    });

    this.version(3).stores({
      workspaces: 'id, name, updatedAt',
      projects:   'id, name, workspaceId, updatedAt, *tags',
      versions:   'id, projectId, createdAt',
      executions: 'id, projectId, createdAt',
      frameworks: 'id, name, updatedAt',
    });
  }
}
```

Les migrations sont automatiques. La version 3 ajoute la table `frameworks` pour les frameworks personnalises.

### Tables

| Table | Cle primaire | Index | Contenu |
|-------|-------------|-------|---------|
| `workspaces` | `id` | `name`, `updatedAt` | Projets (dossiers) |
| `projects` | `id` | `name`, `workspaceId`, `updatedAt`, `*tags` (multi) | Prompts |
| `versions` | `id` | `projectId`, `createdAt` | Snapshots de versions |
| `executions` | `id` | `projectId`, `createdAt` | Resultats d'execution |
| `frameworks` | `id` | `name`, `updatedAt` | Frameworks personnalises |

### Stockage des cles API

Les cles API sont stockees dans `localStorage` (cle : `prompt-ide-api-keys`) sous forme JSON, **pas** dans IndexedDB. Cela evite les problemes de migration de schema pour des donnees sensibles.

```typescript
getApiKeys(): ApiKeys       // Lecture
setApiKeys(keys: ApiKeys)   // Ecriture
```

---

## 7. Modules principaux (lib/)

### lib/prompt.ts

```typescript
compilePrompt(blocks: PromptBlock[], variables: Record<string, string>): string
```
Filtre les blocs actifs (`enabled: true`), joint leurs contenus avec `\n\n`, puis remplace toutes les occurrences `{{variable}}` par leurs valeurs.

```typescript
extractVariables(blocks: PromptBlock[]): string[]
```
Scanne tous les blocs (actifs ou non) avec la regex `/\{\{(\w+)\}\}/g` et retourne les noms uniques des variables detectees.

### lib/tokens.ts

```typescript
countTokens(text: string): number
```
Utilise `gpt-tokenizer` (tokenizer GPT compatible). Fallback : estimation a ~4 caracteres par token si le tokenizer echoue.

```typescript
estimateCost(tokensIn: number, tokensOut: number, model: ModelConfig): number
```
Calcule : `(tokensIn / 1000) * inputCostPer1k + (tokensOut / 1000) * outputCostPer1k`

```typescript
formatCost(cost: number): string    // "$0.002" ou "$0.000123"
formatTokens(count: number): string // "247" ou "1.2k"
```

### lib/api.ts

```typescript
callLLM(prompt, model, apiKeys, options?): Promise<ApiResponse>
```

Dispatch vers le bon provider selon `model.provider` :

| Provider | Endpoint | Headers specifiques |
|----------|----------|---------------------|
| OpenAI | `POST /v1/chat/completions` | `Authorization: Bearer sk-...` |
| Anthropic | `POST /v1/messages` | `x-api-key`, `anthropic-version`, `anthropic-dangerous-direct-browser-access` |
| Google | `POST /v1beta/models/{id}:generateContent?key=...` | Cle dans l'URL |

**Options** : `temperature` (defaut 0.7), `maxTokens` (defaut 2048), `systemPrompt` (optionnel).

**Retour** : `{ text, tokensIn, tokensOut, latencyMs }`

```typescript
optimizePrompt(prompt, model, apiKeys): Promise<string>
```
Envoie un meta-prompt demandant au modele d'ameliorer le prompt fourni. Temperature fixe a 0.3 pour la coherence.

---

## 8. Composants React (components/)

### BlockEditor

| Prop | Type | Description |
|------|------|-------------|
| `value` | `string` | Contenu du bloc |
| `onChange` | `(value: string) => void` | Callback de modification |
| `placeholder?` | `string` | Texte placeholder |
| `variables?` | `string[]` | Variables pour l'autocomplete |

Encapsule une instance CodeMirror 6 avec le theme One Dark, le line wrapping, la coloration syntaxique custom et l'auto-completion des variables. La synchronisation externe (chargement de projet) est geree via un `useEffect` qui compare la valeur actuelle du document.

### PromptBlockComponent

Bloc draggable avec :
- Handle de drag (GripVertical)
- Pastille de couleur selon le type
- Icone et label du type
- Bouton toggle (activer/desactiver)
- Bouton supprimer
- BlockEditor integre

Utilise `useSortable` de dnd-kit pour le drag-and-drop.

### TokenCounter

Barre de status en bas du panneau central :
- Nombre de tokens (via `countTokens`)
- Cout estime (input + 50% output estime)
- Nombre de blocs actifs / total
- Alerte si variables non resolues
- Barre de progression d'utilisation du contexte (vert < 50%, orange < 80%, rouge > 80%)
- Selecteur de modele

### Library

Explorateur de fichiers hierarchique avec :
- **Workspaces** (dossiers) depliables contenant leurs prompts
- **Prompts libres** affiches en bas (hors d'un workspace)
- Barre de recherche (filtre workspaces et prompts)
- Bouton creation de workspace (icone dossier+)
- Bouton creation de prompt libre (icone +)
- Bouton + sur chaque workspace pour creer un prompt a l'interieur
- **Menu contextuel** (clic droit) sur les workspaces : nouveau prompt, renommer, supprimer
- **Menu contextuel** (clic droit) sur les prompts : deplacer vers un workspace, retirer d'un workspace, supprimer
- Pastille de couleur par workspace
- Compteur de prompts par workspace
- Le workspace du prompt actif est auto-deplie

| Prop | Type | Description |
|------|------|-------------|
| `currentProjectId` | `string` | ID du prompt actuellement edite |
| `currentWorkspaceId` | `string?` | ID du workspace du prompt actuel |
| `onLoadProject` | `(id: string) => void` | Charger un prompt |
| `onNewProject` | `(workspaceId?: string) => void` | Creer un prompt (optionnellement dans un workspace) |
| `onCreateWorkspace` | `(name: string) => Promise<Workspace>` | Creer un workspace |
| `onUpdateWorkspace` | `(id: string, changes: Partial<Workspace>) => Promise<void>` | Modifier un workspace |
| `onDeleteWorkspace` | `(id: string) => Promise<void>` | Supprimer un workspace |
| `onMovePrompt` | `(workspaceId: string \| undefined) => void` | Deplacer le prompt actuel |

### FrameworkSelector

Interface de gestion des frameworks avec deux sections :
- **Mes frameworks** : liste depliable des frameworks custom avec actions (modifier, dupliquer, supprimer)
- **Frameworks integres** : les 6 frameworks de base (CO-STAR, RISEN, etc.)

Trois modes de creation :
- **Creer** : formulaire en 2 etapes (infos → definition des blocs avec type, contenu pre-rempli)
- **Depuis actuel** : capture les blocs du prompt courant et les sauvegarde comme framework en 1 etape
- **Modifier** : edition d'un framework existant (memes 2 etapes que la creation)

| Prop | Type | Description |
|------|------|-------------|
| `currentFramework` | `string?` | ID du framework applique au prompt courant |
| `onSelect` | `(id, blocks) => void` | Appliquer un framework |
| `onCreateFramework` | `(name, desc, blocks) => Promise` | Creer un framework custom |
| `onUpdateFramework` | `(id, changes) => Promise` | Modifier un framework |
| `onDeleteFramework` | `(id) => Promise` | Supprimer un framework |
| `onSaveCurrentAsFramework` | `(name, desc) => Promise` | Sauvegarder le prompt actuel comme framework |
| `currentBlocks` | `PromptBlock[]` | Blocs du prompt actuel (pour "Depuis actuel") |

### Playground

Interface de test des prompts avec support local + cloud :
- **Detection automatique des modeles locaux** via `GET {serverUrl}/v1/models` (refresh toutes les 10s)
- **Deux sections de modeles** : "Modeles locaux (Ollama)" en vert + "Modeles cloud (API)" en indigo
- Selection multi-modeles (boutons toggle) — on peut mixer local et cloud
- Sliders temperature (0-2) et max tokens (256-8192)
- Configuration du serveur local (URL + indicateur connexion)
- Configuration des cles API cloud (champs password)
- Bouton Executer → appels sequentiels aux modeles selectionnes
- Affichage des resultats cote a cote avec metriques (latence, tokens, cout ou "gratuit" pour le local)
- Gestion des erreurs (cle manquante, serveur inaccessible, erreur API)
- Les modeles locaux n'ont pas besoin de cle API

### LintingPanel

Regles de validation :

| Regle | Niveau | Condition |
|-------|--------|-----------|
| Aucun bloc actif | error | 0 blocs avec `enabled: true` |
| Blocs vides | warning | Blocs actifs avec `content.trim() === ""` |
| Pas de bloc Tache | warning | Aucun bloc de type `task` actif |
| Variables non resolues | warning | Variables detectees sans valeur dans `variables` |
| Prompt trop court | info | Contenu compile < 20 caracteres |
| Prompt trop long | warning | > 100 000 tokens |
| Pas d'exemples | info | > 200 tokens et aucun bloc `examples` |
| Instructions negatives | info | Detection de patterns "ne pas", "jamais", etc. |

---

## 9. Hook principal (usePromptProject)

```typescript
function usePromptProject(): {
  // --- Etat ---
  project: PromptProject;
  isSaving: boolean;

  // --- Gestion des blocs ---
  addBlock: (type: BlockType) => void;
  removeBlock: (blockId: string) => void;
  updateBlock: (blockId: string, changes: Partial<PromptBlock>) => void;
  toggleBlock: (blockId: string) => void;
  reorderBlocks: (newBlocks: PromptBlock[]) => void;
  setVariable: (key: string, value: string) => void;

  // --- Gestion des prompts ---
  loadProject: (id: string) => Promise<void>;
  newProject: (workspaceId?: string) => void;
  movePromptToWorkspace: (workspaceId: string | undefined) => void;
  loadFramework: (frameworkId: string, blocks: Omit<PromptBlock, 'id'>[]) => void;
  saveVersion: (label: string) => Promise<void>;
  updateProject: (updater: (prev: PromptProject) => PromptProject) => void;

  // --- Gestion des workspaces ---
  createWorkspace: (name: string) => Promise<Workspace>;
  updateWorkspace: (id: string, changes: Partial<Workspace>) => Promise<void>;
  deleteWorkspace: (id: string) => Promise<void>;

  // --- Gestion des frameworks custom ---
  createFramework: (name: string, description: string, blocks: Omit<PromptBlock, 'id'>[]) => Promise<CustomFramework>;
  updateFramework: (id: string, changes: Partial<CustomFramework>) => Promise<void>;
  deleteFramework: (id: string) => Promise<void>;
  saveCurrentAsFramework: (name: string, description: string) => Promise<CustomFramework>;
}
```

### Comportements cles

- **Auto-sauvegarde** : chaque modification declenche un timeout de 500ms. Si aucune modification supplementaire n'arrive, le prompt est persiste en IndexedDB.
- **Initialisation** : au montage, charge le prompt le plus recent depuis IndexedDB. Si aucun n'existe, cree un prompt par defaut avec 3 blocs vides (Role, Contexte, Tache).
- **Immutabilite** : toutes les mises a jour passent par `updateProject` qui cree un nouvel objet prompt (spread operator).
- **newProject(workspaceId?)** : cree un nouveau prompt. Si un `workspaceId` est passe, le prompt est automatiquement range dans ce workspace.
- **movePromptToWorkspace** : change le `workspaceId` du prompt actuel (ou le met a `undefined` pour en faire un prompt libre).
- **deleteWorkspace** : supprime un workspace mais **ne supprime pas** ses prompts — ils deviennent des "prompts libres".
- **createFramework** : cree un nouveau framework custom avec un nom, une description et une liste de blocs-modeles.
- **saveCurrentAsFramework** : raccourci qui capture les blocs du prompt actuellement edite et en fait un framework reutilisable.

---

## 10. Extensions CodeMirror

### Coloration syntaxique (codemirror-prompt.ts)

4 decorateurs bases sur `MatchDecorator` :

| Pattern | Classe CSS | Apparence |
|---------|-----------|-----------|
| `\{\{\w+\}\}` | `cm-prompt-variable` | Violet, gras, fond leger |
| `</?[tag]>` | `cm-prompt-xml-tag` | Bleu, semi-gras |
| `//.*$` | `cm-prompt-comment` | Gris, italique |
| `^## .+$` | `cm-prompt-section` | Vert, gras |

### Auto-completion

Le plugin `promptAutoComplete` ecoute la saisie de `{{` et propose toutes les variables detectees dans les blocs. La liste est mise a jour dynamiquement via une ref.

---

## 11. Integration des APIs LLM

### Serveur local (Ollama via prompt-ai-server)

```
POST {localServerUrl}/v1/chat/completions
Headers: Content-Type: application/json
Body: { model: "mistral-small3.1", messages: [{role, content}], temperature, max_tokens, stream: false }
```

Le serveur Rust agit comme proxy transparent vers Ollama. Le format est identique a l'API OpenAI (`/v1/chat/completions`). Aucune cle API requise. Les modeles disponibles sont ceux installes dans Ollama sur la machine distante.

La fonction `fetchLocalModels()` dans `api.ts` interroge `GET {localServerUrl}/v1/models` pour decouvrir automatiquement les modeles Ollama disponibles. Ils apparaissent dans le Playground sans configuration manuelle.

L'URL du serveur local est stockee dans `localStorage` (cle `prompt-ide-local-server-url`, defaut `http://localhost:8910`) et partagee entre le Playground (LLM) et le STT.

### OpenAI

```
POST https://api.openai.com/v1/chat/completions
Headers: Authorization: Bearer {key}
Body: { model, messages: [{role, content}], temperature, max_tokens }
```

### Anthropic

```
POST https://api.anthropic.com/v1/messages
Headers: x-api-key: {key}, anthropic-version: 2023-06-01,
         anthropic-dangerous-direct-browser-access: true
Body: { model, messages: [{role, content}], max_tokens, temperature, system? }
```

> Note : le header `anthropic-dangerous-direct-browser-access` est necessaire pour les appels directement depuis le navigateur (sans proxy backend). Anthropic deconseille cette pratique en production car la cle API est exposee cote client.

### Google Gemini

```
POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={key}
Body: { contents: [{parts: [{text}]}], generationConfig: {temperature, maxOutputTokens} }
```

---

## 12. Modeles supportes et tarification

### Modeles locaux (Ollama)

Les modeles locaux sont detectes automatiquement via le serveur Rust. Voici les modeles recommandes :

| Modele | Commande Ollama | VRAM (Q4) | Ideal pour |
|--------|----------------|-----------|------------|
| Mistral Small 3.1 | `ollama pull mistral-small3.1` | 14 Go | **Meilleur en francais**, usage general |
| Qwen 2.5-32B | `ollama pull qwen2.5:32b` | 19 Go | Polyvalent, fort en code |
| DeepSeek R1-Distill-32B | `ollama pull deepseek-r1:32b` | 19 Go | Raisonnement, meta-prompting |
| Llama 3.3-70B | `ollama pull llama3.3:70b` | 42 Go | Meilleure qualite absolue |
| Qwen 2.5-7B | `ollama pull qwen2.5:7b` | 4.5 Go | Tests rapides, petit GPU |

Cout : **$0** — tout est local. N'importe quel modele Ollama est utilisable, la liste ci-dessus est une recommandation.

### OpenAI

| Modele | ID API | Contexte | Input $/1k | Output $/1k |
|--------|--------|----------|-----------|-------------|
| GPT-4o | gpt-4o | 128k | $0.0025 | $0.010 |
| GPT-4o Mini | gpt-4o-mini | 128k | $0.00015 | $0.0006 |
| GPT-4.1 | gpt-4.1 | 1M+ | $0.002 | $0.008 |
| GPT-4.1 Mini | gpt-4.1-mini | 1M+ | $0.0004 | $0.0016 |
| GPT-4.1 Nano | gpt-4.1-nano | 1M+ | $0.0001 | $0.0004 |
| o3-mini | o3-mini | 200k | $0.0011 | $0.0044 |

### Anthropic

| Modele | ID API | Contexte | Input $/1k | Output $/1k |
|--------|--------|----------|-----------|-------------|
| Claude Sonnet 4.6 | claude-sonnet-4-6 | 200k | $0.003 | $0.015 |
| Claude Opus 4.6 | claude-opus-4-6 | 1M | $0.015 | $0.075 |
| Claude Haiku 4.5 | claude-haiku-4-5 | 200k | $0.0008 | $0.004 |

### Google

| Modele | ID API | Contexte | Input $/1k | Output $/1k |
|--------|--------|----------|-----------|-------------|
| Gemini 2.5 Pro | gemini-2.5-pro | 1M+ | $0.00125 | $0.010 |
| Gemini 2.5 Flash | gemini-2.5-flash | 1M+ | $0.00015 | $0.0006 |

---

## 13. Frameworks de prompts

| ID | Nom | Blocs generes |
|----|-----|--------------|
| `co-star` | CO-STAR | Contexte, Objectif, Style, Ton, Audience, Format de reponse |
| `risen` | RISEN | Role, Instructions, Etapes (numerotees), Objectif final, Restrictions |
| `race` | RACE | Role, Action, Contexte, Resultat attendu |
| `create` | CREATE | Personnage, Requete, Exemples (avec balises), Ajustements, Type de sortie, Extras |
| `ape` | APE | Action, But, Resultat attendu |
| `stoke` | STOKE | Situation, Tache, Objectif, Connaissances, Exemples |

Chaque framework genere des blocs pre-remplis avec des titres `## Section` que l'utilisateur complete.

### Frameworks personnalises

En plus des 6 frameworks integres, l'utilisateur peut creer ses propres frameworks :

- **Creation manuelle** : definir un nom, une description, puis composer les blocs un par un (choisir le type, ecrire le contenu pre-rempli)
- **Depuis le prompt actuel** : capture la structure du prompt en cours d'edition et la sauvegarde comme framework reutilisable
- **Modification** : les frameworks custom sont editables a tout moment
- **Duplication** : un framework peut etre duplique pour servir de base a un nouveau

Les frameworks custom sont stockes dans la table `frameworks` d'IndexedDB. L'identifiant utilise lors de l'application est prefixe `custom:` (ex: `custom:abc-123`) pour les distinguer des integres.

---

## 14. Speech-to-Text (STT)

### Architecture STT

Le systeme STT fonctionne en deux parties independantes :

```
┌─────────────────────────────┐         ┌──────────────────────────────┐
│     App Web (navigateur)    │         │   Prompt STT Server (Rust)   │
│                             │  HTTP   │                              │
│  Micro → MediaRecorder      │◄───────►│  whisper-rs (whisper.cpp)    │
│  → WAV base64 → POST /transcribe     │  Modeles Whisper locaux      │
│                             │         │  GUI Iced                    │
│  OU                         │         │                              │
│  Micro → FormData           │         │  Port 8910 (configurable)    │
│  → API cloud (OpenAI/Groq/  │         └──────────────────────────────┘
│    Deepgram)                │         (peut etre sur une autre machine)
└─────────────────────────────┘
```

### Module lib/stt.ts

```typescript
type SttProvider = 'local' | 'openai' | 'deepgram' | 'groq';

interface SttConfig {
  provider: SttProvider;       // Fournisseur choisi
  localServerUrl: string;      // URL du serveur Rust (ex: http://192.168.1.50:8910)
  language: string;            // "auto", "fr", "en", etc.
}
```

**Fonctions principales :**

| Fonction | Description |
|----------|-------------|
| `getSttConfig()` / `setSttConfig()` | Lecture/ecriture de la config STT dans localStorage |
| `createRecorder()` | Cree un enregistreur audio (start/stop/isRecording) via MediaRecorder |
| `transcribe(audioBlob, config)` | Envoie l'audio au provider choisi et retourne le texte |

**Providers de transcription :**

| Provider | Endpoint | Authentification | Format envoye |
|----------|----------|-----------------|---------------|
| `local` | `POST {url}/transcribe` | Aucune | WAV base64 dans JSON |
| `openai` | `POST /v1/audio/transcriptions` | Bearer token | FormData (webm) |
| `groq` | `POST /openai/v1/audio/transcriptions` | Bearer token | FormData (webm) |
| `deepgram` | `POST /v1/listen` | Token header | Audio brut (webm) |

**Encodage audio :** L'audio du navigateur (webm/opus) est decode via AudioContext, converti en PCM mono 16kHz, puis encode en WAV pour le serveur local. Les APIs cloud acceptent le webm directement.

### Composant SttSettings

Panneau de configuration (onglet STT dans le panneau droit) :
- Selection du provider (local, OpenAI, Groq, Deepgram)
- Champ URL pour le serveur local avec indicateur de connexion (online/offline)
- Champs cles API pour Deepgram et Groq
- Selecteur de langue (13 langues + auto-detection)
- Verification automatique du serveur local toutes les 5 secondes

### Bouton micro dans PromptBlock

Chaque bloc a un bouton micro dans son header :
- **Clic 1** : commence l'enregistrement (bordure rouge pulsante, indicateur "Enregistrement en cours...")
- **Clic 2** : arrete l'enregistrement, envoie l'audio au provider, insere le texte transcrit a la fin du contenu du bloc
- **Pendant la transcription** : icone spinner
- **Erreur** : message affiche sous le header du bloc pendant 4 secondes

---

## 15. Application Rust — Prompt AI Server

### Presentation

Application desktop cross-platform (v0.2.0) ecrite en Rust, servant de **hub unifie** pour l'IA locale. Elle combine :
- **STT** : inference Whisper locale pour la dictee vocale
- **Proxy LLM** : transmission transparente des requetes vers Ollama pour les modeles de langage

L'app web ne configure qu'**une seule URL** pour acceder a tous les services locaux.

### Stack technique

| Composant | Technologie | Role |
|-----------|------------|------|
| GUI | Iced 0.13 | Interface graphique native |
| Serveur HTTP | axum 0.8 | Endpoints REST (STT + proxy LLM) |
| Inference STT | whisper-rs 0.13 (whisper.cpp) | Reconnaissance vocale locale |
| Proxy LLM | reqwest 0.12 | Relay vers Ollama |
| HTTP client | reqwest 0.12 | Telechargement modeles + proxy |
| Serialisation | serde + serde_json | JSON |
| Audio | hound 3.5 | Decodage WAV |
| CORS | tower-http 0.6 | Autoriser les appels depuis le navigateur |
| Async | tokio | Runtime asynchrone |

### Modules source

| Fichier | Role |
|---------|------|
| `main.rs` | GUI Iced (serveur, Ollama, Whisper, telechargements, journal) |
| `server.rs` | Serveur HTTP axum (tous les endpoints) |
| `whisper_engine.rs` | Inference Whisper (load, transcribe) |
| `ollama.rs` | Connexion Ollama (config, status, proxy chat/models) |
| `models.rs` | Catalogue des modeles Whisper |
| `downloader.rs` | Telechargement depuis HuggingFace |

### Modeles Whisper disponibles

| ID | Nom | Taille | Parametres | Utilisation recommandee |
|----|-----|--------|-----------|------------------------|
| `tiny` | Whisper Tiny | 75 Mo | 39M | CPU faible, temps reel |
| `base` | Whisper Base | 142 Mo | 74M | CPU, bonne qualite |
| `small` | Whisper Small | 466 Mo | 244M | Bon compromis |
| `medium` | Whisper Medium | 1.5 Go | 769M | CPU puissant ou GPU |
| `large-v3` | Whisper Large v3 | 2.9 Go | 1.5B | GPU 10Go+ recommande |
| `large-v3-turbo` | Whisper Large v3 Turbo | 1.5 Go | 809M | GPU 6Go+, quasi aussi precis que v3 |

Les modeles sont telecharges depuis HuggingFace (format GGML) et stockes dans `{data_local_dir}/prompt-stt-server/models/`.

### Endpoints HTTP

#### `GET /health`

Retourne la sante globale du serveur (STT + LLM).

```json
{
  "status": "ok",
  "version": "0.2.0",
  "stt": { "model_loaded": true },
  "llm": { "ollama_connected": true, "ollama_url": "http://localhost:11434", "models_count": 3 }
}
```

#### `GET /models`

Liste les modeles Whisper (STT) disponibles et leur statut d'installation.

```json
{
  "available": [
    { "id": "tiny", "name": "Whisper Tiny", "size_mb": 75, "installed": true, "description": "..." }
  ],
  "active": "small"
}
```

#### `POST /transcribe`

Transcrit un audio WAV encode en base64.

**Requete :**
```json
{ "audio": "UklGR...", "language": "fr" }
```

**Reponse :**
```json
{ "text": "Bonjour, ceci est un test.", "language": "fr", "duration_ms": 1234 }
```

#### `POST /v1/chat/completions` (proxy Ollama)

Proxy transparent vers `Ollama /v1/chat/completions`. Meme format que l'API OpenAI. Le serveur Rust relaie la requete et la reponse sans modification.

```json
{
  "model": "mistral-small3.1",
  "messages": [{ "role": "user", "content": "Bonjour" }],
  "temperature": 0.7,
  "max_tokens": 2048
}
```

#### `GET /v1/models` (proxy Ollama)

Proxy transparent vers `Ollama /v1/models`. Retourne la liste des modeles Ollama installes au format OpenAI.

#### `GET /ollama/status`

Etat de la connexion Ollama.

```json
{
  "connected": true,
  "url": "http://localhost:11434",
  "models": [
    { "name": "mistral-small3.1:latest", "size_gb": 14.2, "parameter_size": "24B", "quantization": "Q4_K_M" }
  ],
  "error": null
}
```

### Compilation et installation

```bash
cd prompt-stt-server

# Build (premiere compilation longue a cause de whisper.cpp)
cargo build --release

# Lancer
./target/release/prompt-ai-server
```

**Prerequis systeme :**
- Rust (cargo)
- Compilateur C++ (g++)
- CMake
- libssl-dev, libclang-dev (Linux)
- **Ollama** installe et lance separement (pour les LLM)

### Interface graphique (Iced)

L'application affiche 5 sections :

1. **Serveur unifie** : toggle on/off, indicateur de statut, URL affichee (`http://0.0.0.0:8910`)
2. **Ollama (LLM proxy)** : toggle on/off, champ URL Ollama (defaut `http://localhost:11434`), bouton "Tester", indicateur connecte/deconnecte, liste des modeles detectes avec taille/quantification
3. **Whisper (STT)** : selecteur parmi les modeles installes, bouton "Charger", statut (pret / erreur)
4. **Modeles Whisper** : liste des 6 modeles avec taille, statut installe/non-installe, bouton telecharger, barre de progression
5. **Journal** : 6 derniers messages de log

### Utilisation a distance

Le serveur ecoute sur `0.0.0.0:{port}`, accessible depuis toute machine du reseau local. L'app web ne configure qu'une seule URL (ex: `http://192.168.1.100:8910`) pour acceder a la fois au STT (Whisper) et aux LLM (Ollama proxy).

### Connexion Ollama

Ollama doit etre installe et lance separement sur la meme machine (ou une machine accessible). Par defaut l'app se connecte a `http://localhost:11434`. Le statut est verifie toutes les 5 secondes. Si Ollama est sur une autre machine, modifiez l'URL dans la GUI.

Pour exposer Ollama sur le reseau : `OLLAMA_HOST=0.0.0.0:11434 ollama serve`

---

## 16. Systeme d'authentification

### Stockage et securite

L'authentification est geree entierement cote client avec IndexedDB (table `users`). Les mots de passe sont hashes avec **PBKDF2** via l'API **Web Crypto** du navigateur (jamais stockes en clair).

### Inscription (Register)

- Validation de l'email (format valide)
- Mot de passe minimum 6 caracteres
- Hash PBKDF2 du mot de passe avant stockage
- Creation d'un utilisateur dans la table `users` d'IndexedDB

### Connexion (Login)

- Verification email + hash du mot de passe fourni contre le hash stocke
- Session persistee dans `localStorage` (ID utilisateur + infos de session)
- Reconnexion automatique au rechargement de la page

### Isolation des donnees

Toutes les donnees sont filtrees par `userId` :
- Les **workspaces**, **projects** et **frameworks** sont associes a l'utilisateur connecte
- Un utilisateur ne voit que ses propres donnees
- La deconnexion nettoie la session mais conserve les donnees en IndexedDB

### Gestion du profil

Le composant **ProfileModal** permet de :
- Modifier le nom d'affichage
- Changer le mot de passe (ancien mot de passe requis + nouveau mot de passe min 6 caracteres)

### Composants

| Composant | Role |
|-----------|------|
| `AuthPage` | Page de connexion / inscription (affichee si non connecte) |
| `UserMenu` | Menu utilisateur dans le header (nom, deconnexion, acces profil) |
| `ProfileModal` | Modal de modification du profil (nom, mot de passe) |

---

## 17. Internationalisation (i18n)

### Architecture

Le systeme de traduction est implemente dans `src/lib/i18n.ts` avec environ **170 cles de traduction** couvrant l'integralite de l'interface.

### Langues supportees

| Code | Langue |
|------|--------|
| `fr` | Francais (par defaut) |
| `en` | Anglais |

### Implementation React

- **I18nContext** : contexte React fournissant la langue courante et la fonction de traduction
- **useT()** : hook personnalise retournant la fonction `t(key)` pour traduire une cle
- Tous les composants utilisent `t('cle')` au lieu de chaines en dur

### Selecteur de langue

Un **dropdown** dans le header permet de basculer entre FR et EN. Le choix est persiste dans `localStorage` et restaure au chargement de l'application.

---

## 18. Themes (Light / Dark / System)

### Architecture

Le systeme de themes est gere par `src/lib/theme.ts` :

| Export | Type | Description |
|--------|------|-------------|
| `ThemeMode` | Type | `'light' \| 'dark' \| 'system'` |
| `ThemeContext` | React Context | Fournit le theme courant et la fonction de changement |
| `useTheme()` | Hook | Acces au theme depuis n'importe quel composant |

### Fonctionnement

- L'attribut `data-theme` est applique sur l'element `<html>` (`light` ou `dark`)
- Les **variables CSS** definies dans `index.css` sont surchargees par theme via des selecteurs `[data-theme="light"]` et `[data-theme="dark"]`
- Le theme **CodeMirror** bascule automatiquement : `oneDark` pour le mode sombre, theme clair personnalise pour le mode light

### Variables CSS (exemple mode sombre)

```css
@theme {
  --color-bg-primary: #0f1117;
  --color-bg-secondary: #1a1b23;
  --color-bg-tertiary: #22232d;
  --color-bg-hover: #2a2b37;
  --color-border: #2e303a;
  --color-border-focus: #6366f1;
  --color-text-primary: #f3f4f6;
  --color-text-secondary: #9ca3af;
  --color-text-muted: #6b7280;
  --color-accent: #6366f1;
  --color-accent-hover: #818cf8;
}
```

### Modes disponibles

| Mode | Comportement |
|------|-------------|
| **Light** | Theme clair force |
| **Dark** | Theme sombre force |
| **System** | Suit la preference OS (`prefers-color-scheme`) |

Un **dropdown** dans le header permet de selectionner le mode. Le choix est persiste dans `localStorage`.

---

## 19. Import / Export

### Import

Le composant **ExportPanel** inclut desormais une fonctionnalite d'import :

- Bouton **Importer** qui ouvre un selecteur de fichier JSON
- Le fichier JSON importe doit contenir une structure valide : `blocks`, `variables`, `name`
- **Validation de la structure** avant import : verification de la presence et du format des champs requis
- Les blocs importes remplacent le prompt courant

### Export

5 formats d'export disponibles :

| Format | Extension | Description |
|--------|-----------|-------------|
| **Texte brut** | `.txt` | Prompt compile sans formatage |
| **Markdown** | `.md` | Prompt avec titres de sections en `##` |
| **JSON** | `.json` | Structure complete (blocs, variables, nom) — reimportable |
| **OpenAI** | `.json` | Format `messages[]` compatible API OpenAI |
| **Anthropic** | `.json` | Format `messages[]` + `system` compatible API Anthropic |

---

## 20. Streaming des reponses

### Implementation

Le streaming est implemente dans `src/lib/api.ts` via deux fonctions :

| Fonction | Utilisation |
|----------|-------------|
| `callLLMStream()` | Streaming pour le Playground (execution simple) |
| `callLLMStreamMessages()` | Streaming pour le Mode Conversation (multi-turn) |

### Protocole

- **OpenAI et modeles locaux (Ollama)** : streaming SSE (Server-Sent Events) via `stream: true` dans la requete. Les chunks sont parses en temps reel et affiches progressivement.
- **Anthropic / Google** : fallback en mode non-streaming (reponse complete) car le streaming direct depuis le navigateur n'est pas supporte de maniere fiable.

### Affichage progressif

Dans le **Playground**, la reponse s'affiche caractere par caractere au fur et a mesure de la reception des chunks SSE. L'utilisateur voit la generation en temps reel au lieu d'attendre la reponse complete.

---

## 21. Raccourcis clavier

| Raccourci | Action |
|-----------|--------|
| `Ctrl+Enter` / `Cmd+Enter` | Executer le prompt dans le Playground |
| `Ctrl+S` / `Cmd+S` | Sauvegarder une version (label automatique avec timestamp) |
| `Ctrl+N` / `Cmd+N` | Creer un nouveau prompt |

Les raccourcis sont actifs globalement dans l'application. `Cmd` est utilise sur macOS, `Ctrl` sur Windows/Linux.

---

## 22. Historique des executions

### Composant ExecutionHistory

Affiche l'historique des executions passees pour le prompt courant :

- **Informations par execution** : modele utilise, date, latence (ms), tokens (in/out), cout estime, apercu de la reponse
- **Clic pour developper** : affiche la reponse complete de l'execution
- **Bouton vider l'historique** : supprime toutes les executions du prompt courant

Les executions sont stockees dans la table `executions` d'IndexedDB, indexees par `projectId` et `createdAt`.

---

## 23. Chainage de prompts (Workflows)

### Composant PromptChain

Permet d'executer plusieurs prompts en sequence, ou la sortie de chaque prompt alimente le suivant :

- **Selection d'un workspace** : les prompts du workspace sont listes par date de creation
- **Variable de chainage** : la sortie du prompt N est injectee dans le prompt N+1 via la variable `{{chain_output_N}}`
- **Execution sequentielle** : les prompts sont executes un par un dans l'ordre
- **Gestion des erreurs** : si un prompt echoue, l'execution de la chaine s'arrete et l'erreur est affichee

### Exemple de flux

```
Prompt 1 (Recherche)     → reponse → {{chain_output_1}}
Prompt 2 (Analyse)       → reponse → {{chain_output_2}}
Prompt 3 (Redaction)     → reponse finale
```

---

## 24. Mode Conversation (Multi-turn)

### Composant ConversationMode

Interface de chat multi-tour permettant d'interagir avec un LLM de maniere conversationnelle :

- **Historique des messages** : affichage de la conversation complete (messages utilisateur + reponses du modele)
- **Selecteur de modele** : choix du modele LLM (local ou cloud)
- **Slider temperature** : reglage de la temperature (0-2)
- **Champ system prompt** : permet de definir un system prompt ; peut utiliser le prompt compile courant comme system prompt
- **Streaming** : les reponses sont affichees en streaming via `callLLMStreamMessages()` (pour les providers compatibles)
- **Contexte complet** : l'integralite de l'historique de conversation est envoyee avec chaque requete pour maintenir le contexte

---

## 25. Statistiques (Analytics)

### Composant AnalyticsPanel

Tableau de bord affichant les statistiques d'utilisation :

### Metriques affichees

| Metrique | Description |
|----------|-------------|
| Total executions | Nombre total d'executions de prompts |
| Total tokens | Somme des tokens (input + output) |
| Cout cumule | Somme des couts estimes en USD |
| Latence moyenne | Temps de reponse moyen en ms |
| Modele le plus utilise | Modele avec le plus grand nombre d'executions |

### Visualisation

- **Graphique en barres CSS** : repartition des executions par modele
- Pas de dependance externe (pas de bibliotheque de charts) — barres construites en CSS pur

### Filtre temporel

| Filtre | Periode |
|--------|---------|
| 7 jours | Executions des 7 derniers jours |
| 30 jours | Executions des 30 derniers jours |
| Tout | Toutes les executions |

---

## 26. Responsive Mobile

### Adaptations pour ecrans < 768px

- **Panneaux lateraux en overlay** : les panneaux gauche et droit s'affichent en overlay par-dessus le contenu principal (au lieu d'etre cote a cote)
- **Backdrop** : un fond semi-transparent est affiche derriere le panneau ouvert
- **Auto-collapse** : les panneaux se ferment automatiquement sur mobile pour maximiser l'espace d'edition

### Classes CSS

| Classe | Role |
|--------|------|
| `mobile-panel-left` | Panneau gauche en overlay sur mobile |
| `mobile-panel-right` | Panneau droit en overlay sur mobile |
| `mobile-backdrop` | Fond semi-transparent cliquable pour fermer le panneau |

---

## 27. PWA (Progressive Web App)

### Manifest

Le fichier `manifest.json` declare l'application comme installable :
- Nom, icones, couleurs du theme
- `display: standalone` pour un rendu natif
- `start_url` pointant vers la racine

### Service Worker

Le fichier `sw.js` implemente une strategie de cache **stale-while-revalidate** :
- Les assets statiques sont mis en cache a la premiere visite
- Les requetes suivantes servent le cache immediatement tout en mettant a jour en arriere-plan
- Support **offline** pour les assets statiques (HTML, CSS, JS, icones)

### Installation

L'application est installable en tant qu'application native sur :
- **Desktop** : Chrome, Edge (bouton d'installation dans la barre d'adresse)
- **Mobile** : Android (Chrome), iOS (Safari — "Ajouter a l'ecran d'accueil")

---

## 28. Docker

### Fichiers

| Fichier | Description |
|---------|-------------|
| `Dockerfile` | Build multi-stage : Node 22 (build) → Nginx Alpine (serve) |
| `nginx.conf` | Configuration Nginx (gzip, cache assets, fallback SPA) |
| `docker-compose.yml` | Orchestration avec un seul service |

### Architecture du Dockerfile

```
Stage 1: Node 22 Alpine
  → npm install
  → npm run build
  → Produit dist/

Stage 2: Nginx Alpine
  → Copie dist/ dans /usr/share/nginx/html
  → Copie nginx.conf
  → Expose port 80
```

### Configuration Nginx

- **Gzip** active pour HTML, CSS, JS, JSON
- **Cache** longue duree pour les assets statiques (`/assets/`)
- **Fallback SPA** : toute route non trouvee redirige vers `index.html`

### Commandes

```bash
# Build et lancer
docker compose up -d

# Reconstruire apres modifications
docker compose up -d --build

# Arreter
docker compose down
```

**Taille de l'image** : ~97 Mo (grace a Nginx Alpine et au build multi-stage).

---

## 29. Scripts et commandes

| Commande | Description |
|----------|-------------|
| `npm run dev` | Demarrer le serveur de dev (HMR, port 5173) |
| `npm run build` | Type-check TypeScript + build Vite de production |
| `npm run preview` | Previsualiser le build de production |
| `npm run lint` | Lancer ESLint |

---

## 30. Dependances

### Production

| Package | Role |
|---------|------|
| `react`, `react-dom` | Framework UI |
| `@codemirror/*` (7 packages) | Editeur de code (state, view, commands, autocomplete, markdown, theme) |
| `@dnd-kit/core`, `sortable`, `utilities` | Drag-and-drop des blocs |
| `dexie`, `dexie-react-hooks` | Base de donnees IndexedDB |
| `gpt-tokenizer` | Comptage de tokens (compatible GPT) |
| `lucide-react` | Bibliotheque d'icones |
| `uuid` | Generation d'identifiants uniques |

### Developpement

| Package | Role |
|---------|------|
| `typescript` | Type-checking |
| `vite`, `@vitejs/plugin-react` | Build et HMR |
| `tailwindcss`, `@tailwindcss/vite` | Framework CSS |
| `eslint`, `typescript-eslint` | Linting |

---

## 31. Deploiement

### Build statique

```bash
npm run build
# Genere dist/ avec :
# - index.html (0.5 KB)
# - assets/index-*.css (~23 KB gzip: 5 KB)
# - assets/index-*.js (~2.7 MB gzip: 1.2 MB)
```

Le contenu de `dist/` peut etre deploye sur n'importe quel hebergement statique :

- **Vercel** : `vercel --prod`
- **Netlify** : drag-and-drop du dossier `dist/`
- **GitHub Pages** : avec `base: './'` dans vite.config.ts
- **Serveur Apache/Nginx** : servir `dist/` avec fallback sur `index.html`

### Considerations de securite

- Les cles API sont stockees en `localStorage` dans le navigateur de l'utilisateur
- Aucun backend n'est necessaire — les appels API se font directement depuis le navigateur
- Pour une utilisation en production partagee, il est recommande d'ajouter un backend proxy pour proteger les cles API
- Le header `anthropic-dangerous-direct-browser-access` est utilise pour les appels directs a Anthropic — cela expose la cle dans les requetes reseau

### Optimisations possibles

- **Code-splitting** : decouvrir `Playground` et `PromptOptimizer` en lazy-load pour reduire le bundle initial
- **Service Worker** : ajouter un SW pour le fonctionnement hors-ligne complet
- **Backend proxy** : Node.js / Edge Functions pour securiser les cles API
