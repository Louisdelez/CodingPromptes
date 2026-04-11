# Inkwell

A complete AI prompt engineering platform — build, test, optimize, and manage your prompts with a powerful web editor and optional local AI inference.

![License](https://img.shields.io/github/license/Louisdelez/CodingPromptes)
![TypeScript](https://img.shields.io/badge/TypeScript-6.0-blue)
![React](https://img.shields.io/badge/React-19-61DAFB)
![Rust](https://img.shields.io/badge/Rust-1.94-orange)
![Docker](https://img.shields.io/badge/Docker-ready-2496ED)

---

## Features

### Editor
- Block-based prompt editor with **6 block types** (Role, Context, Task, Examples, Constraints, Format)
- **CodeMirror 6** with syntax highlighting for `{{variables}}`, XML tags, sections, comments
- **Drag-and-drop** block reordering
- Line numbers, word/char/line/token counter with cost estimation
- **Variable system** `{{variable}}` with auto-completion and live preview

### Frameworks
- **6 built-in frameworks**: CO-STAR, RISEN, RACE, CREATE, APE, STOKE
- Create your own **custom frameworks** from scratch or from the current prompt

### Testing
- **Playground** — test prompts against 11+ cloud models (GPT-4o, Claude, Gemini) and local models (Ollama)
- **Streaming** responses (word-by-word, like ChatGPT)
- **Multi-model comparison** side by side
- **Conversation mode** — multi-turn chat with full context
- **Prompt chaining** — chain prompts from a project, output of step N becomes `{{chain_output_N}}`

### AI Tools
- **Prompt Optimizer** — AI analyzes and improves your prompt
- **Linting** — 8 validation rules (empty blocks, missing task, negative instructions, etc.)
- **Speech-to-Text** — dictate prompts via mic (4 providers: local Whisper, OpenAI, Groq, Deepgram)

### Organization
- **Workspaces** (project folders) with color picker and drag-and-drop
- **Version history** with snapshots, restore, and **visual diff**
- **Execution history** with metrics (latency, tokens, cost)
- **Analytics** dashboard (total executions, cost, top model, bar chart)
- **Library** with tree view, context menus, full-text search (names + block content)

### Import / Export
- **Import** JSON prompts
- **Export** to: plain text, Markdown, JSON, OpenAI API format, Anthropic API format

### Platform
- **i18n** — French and English
- **Themes** — Light, Dark, System (follows OS)
- **Accounts** — Login/Register with PBKDF2 password hashing, data isolated per user
- **Responsive** — mobile-friendly with overlay panels
- **PWA** — installable, works offline
- **Docker** — one command to deploy
- **Markdown rendering** in responses

---

## Quick Start

### Web App (development)

```bash
cd prompt-ide
npm install
npm run dev
# → http://localhost:5173
```

### Docker (production)

```bash
cd prompt-ide
docker compose up -d
# → http://localhost:3000
```

---

## Native Desktop App (GPUI)

Application native GPU-acceleree construite avec GPUI (le framework de Zed).

```bash
make install
inkwell-gpui
```

Fonctionnalites :
- Editeur de blocs avec drag-and-drop, numeros de ligne, selecteur de type
- Pipeline SDD complet (Constitution, Specification, Plan, Tasks, Implementation)
- Sidebar avec projets, workspaces, recherche, steering, hooks
- Panneau droit avec 13+ onglets (SDD, Git, Credits, Chat, Autopilot, Catalog, ...)
- Multi-provider LLM (OpenAI, Anthropic, Google, Ollama)

---

## CLI

```bash
make install
# ou: cd inkwell-cli && cargo build --release

# Pipeline SDD
inkwell init mon-projet
inkwell constitution "API REST de gestion de recettes"
inkwell specify "CRUD complet avec auth JWT"
inkwell plan rust
inkwell tasks
inkwell implement

# Validation et audit
inkwell validate
inkwell checklist
inkwell analyze

# Chat interactif avec contexte projet
inkwell chat

# Configuration multi-provider
inkwell config set model claude-sonnet-4.6
inkwell config set openai-key sk-...
inkwell config set anthropic-key sk-...

# Auto-completion shell
inkwell completions bash >> ~/.bashrc
inkwell completions zsh >> ~/.zshrc
inkwell completions fish > ~/.config/fish/completions/inkwell.fish
```

---

## MCP Server (Claude Code)

```bash
inkwell mcp-install    # Configure automatiquement ~/.claude.json
```

10 outils MCP : `inkwell_status`, `inkwell_read_phase`, `inkwell_write_phase`, `inkwell_list_projects`, `inkwell_validate`, `inkwell_read_steering`, `inkwell_write_steering`, `inkwell_read_tasks`, `inkwell_search`, `inkwell_read_project`.

---

## Local AI Server (optional)

Serveur Rust desktop pour Whisper (STT) et proxy Ollama (LLM). L'app web se connecte via une seule URL.

### Prerequisites

- Rust, g++, CMake, libssl-dev, libclang-dev
- [Ollama](https://ollama.com) installed separately

### Build & Run

```bash
cd inkwell-gpu-server
cargo build --release
./target/release/inkwell-server
```

### Install Ollama Models

```bash
ollama pull mistral-small3.1    # Best for French
ollama pull deepseek-r1:32b     # Best for reasoning
ollama pull qwen2.5:7b          # Fast, lightweight
```

---

## Supported Models

### Cloud (API)

| Provider | Models |
|----------|--------|
| OpenAI | GPT-4o, GPT-4o Mini, GPT-4.1, GPT-4.1 Mini/Nano, o3-mini |
| Anthropic | Claude Sonnet 4.6, Claude Opus 4.6, Claude Haiku 4.5 |
| Google | Gemini 2.5 Pro, Gemini 2.5 Flash |

### Local (Ollama)

Any Ollama model works. Recommended:

| Model | VRAM | Best for |
|-------|------|----------|
| Mistral Small 3.1 (24B) | 14 GB | French, general use |
| Qwen 2.5-32B | 19 GB | Code, reasoning |
| DeepSeek R1-32B | 19 GB | Prompt optimization |
| Llama 3.3-70B | 42 GB | Highest quality |
| Qwen 2.5-7B | 4.5 GB | Fast testing |

### STT (Speech-to-Text)

| Provider | Model | Cost |
|----------|-------|------|
| Local (Rust server) | Whisper tiny → large-v3 | Free |
| OpenAI | gpt-4o-mini-transcribe | $0.003/min |
| Groq | Whisper v3-turbo | $0.0007/min |
| Deepgram | Nova-3 | $0.004/min |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Enter` | Execute prompt |
| `Ctrl+S` | Save version (auto-labeled) |
| `Ctrl+N` | New prompt |

---

## Tech Stack

### Web App

| Layer | Technology |
|-------|-----------|
| Framework | React 19 + TypeScript 6 |
| Build | Vite 8 |
| CSS | Tailwind CSS 4 |
| Editor | CodeMirror 6 |
| Drag & Drop | dnd-kit |
| Database | Dexie (IndexedDB) |
| Tokenizer | gpt-tokenizer |
| Icons | Lucide React |

### Rust Server

| Layer | Technology |
|-------|-----------|
| GUI | Iced 0.13 |
| HTTP | axum 0.8 |
| STT | whisper-rs (whisper.cpp) |
| Async | tokio |

---

## Documentation

- [Technical Documentation](inkwell/docs/DOCUMENTATION.md) — 31 sections, architecture, API reference, types
- [User Guide](inkwell/docs/GUIDE_UTILISATEUR.md) — 28 sections, step-by-step tutorials, FAQ

---

## Project Structure

```
CodingPromptes/
├── inkwell-core/               # Types partages (PromptBlock, BlockType, ...)
├── inkwell-gpui/               # App desktop native (GPUI/Zed)
├── inkwell-cli/                # CLI (clap, tokio, reqwest)
├── inkwell-mcp/                # Serveur MCP (JSON-RPC stdio)
├── inkwell-gpu-server/         # Serveur local Whisper + Ollama proxy
├── prompt-ide/                 # Web app (React + TypeScript)
├── Makefile                    # Build & install des 3 binaires Rust
└── README.md
```

---

## License

[MIT](LICENSE)
