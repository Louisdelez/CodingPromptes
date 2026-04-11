# inkwell-devtools-mcp

MCP server that exposes the running `inkwell-gpui` app to Claude Code (and any other MCP-compatible client) for live read/write control.

Architecture:

```
Claude Code ─stdio/MCP─> inkwell-devtools-mcp ─Unix socket/JSON-RPC─> inkwell-gpui (devtools module)
```

Socket: `~/.local/share/inkwell/devtools.sock`

## Tool reference (44 tools)

### Health & introspection

| Tool | Description |
|---|---|
| `devtools_health_check` | Check if the app is running, return uptime |
| `devtools_app_state` | Full snapshot: project, blocks, variables, chat, executions, UI |
| `devtools_list_tabs` | Current tabs + panel open/closed state |
| `devtools_get_logs(lines?)` | Recent log lines from the tracing ring buffer |
| `devtools_validate_state` | Detect project inconsistencies (issues + info) |
| `devtools_screenshot` | Capture a PNG of the app window (filtered by PID) |
| `devtools_get_settings` | dark_mode, selected_model, project_name |

### Projects

| Tool | Description |
|---|---|
| `devtools_get_project` | Current project with all blocks |
| `devtools_list_projects` | All projects on disk with metadata |
| `devtools_new_project(name?)` | Create a new project (3 default blocks) |
| `devtools_rename_project(name)` | Rename current project |
| `devtools_open_project(project_id)` | Switch to another project (flushes pending edits) |
| `devtools_delete_project(project_id)` | Delete a project (cannot delete the open one) |
| `devtools_save_project` | Force a synchronous save to disk |

### Blocks

| Tool | Description |
|---|---|
| `devtools_get_block(index)` | Read a single block by index |
| `devtools_get_metrics` | tokens / chars / words / lines / enabled count |
| `devtools_set_block(index, content)` | Update a block's content |
| `devtools_add_block(block_type, content?)` | Append a new block. Types: `role`, `context`, `task`, `examples`, `constraints`, `format`, `sdd-constitution`, `sdd-specification`, `sdd-plan`, `sdd-tasks`, `sdd-implementation` |
| `devtools_delete_block(index)` | Remove a block |
| `devtools_toggle_block(index)` | Enable/disable a block |
| `devtools_reorder_blocks(from, to)` | Move a block. Note: `to` is an index *in the pre-remove list* — `reorder(3, 4)` is a no-op |

### Variables (template substitution)

| Tool | Description |
|---|---|
| `devtools_get_variables` | HashMap of all variables on the current project |
| `devtools_set_variable(key, value)` | Set `{{key}}` → `value` |
| `devtools_delete_variable(key)` | Remove a variable |

Variable syntax: `{{name}}`, `{{user.name}}`, `{{api-key}}`, `{{v1.2}}` — supports letters, digits, underscore, dot, hyphen. Undefined variables are left as-is in the compiled prompt.

### Models & execution

| Tool | Description |
|---|---|
| `devtools_set_model(model)` | Change the active LLM. Whitelisted: `gpt-4o`, `gpt-4o-mini`, `gpt-4.1`, `gpt-4.1-mini`, `gpt-4.1-nano`, `o3-mini`, `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5`, `gemini-2.5-pro`, `gemini-2.5-flash`, `llama3.2`, `llama3.1`, `qwen2.5`, `mistral`, plus any `ollama/*` or `*:*` |
| `devtools_run_prompt` | Execute the current prompt (like Ctrl+Enter). Records an execution. |
| `devtools_run_sdd` | Run the full SDD pipeline: Constitution → Specification → Plan → Tasks → Implementation |
| `devtools_get_playground_response` | Last run_prompt response + loading state |
| `devtools_get_executions(limit?)` | Recent executions with metrics and previews (default 20) |

### Chat

| Tool | Description |
|---|---|
| `devtools_send_chat(message)` | Post a message to the Chat tab (appends a user + empty assistant entry, triggers LLM call) |
| `devtools_get_chat_messages(limit?)` | Full chat history (role + content) |

### UI navigation

| Tool | Description |
|---|---|
| `devtools_select_tab(tab)` | Switch right-panel tab. Values: `Preview`, `Playground`, `Chat`, `Sdd`, `History`, `Export`, `Terminal`, `Optimize`, `Lint`, `Analytics`, `Chain`, `Stt`, `Fleet`, `Collab` |
| `devtools_select_left_tab(tab)` | `Library`, `Frameworks`, `Versions` |
| `devtools_toggle_panel(panel)` | `left` or `right` |

### App settings

| Tool | Description |
|---|---|
| `devtools_set_dark_mode(enabled)` | Switch dark/light mode |
| `devtools_set_lang(lang)` | Switch language: `fr` or `en` |
| `devtools_set_api_key(provider, key)` | `openai`, `anthropic`, or `google` |
| `devtools_set_github_repo(repo)` | Set GitHub repo URL for push sync |

### Custom frameworks

| Tool | Description |
|---|---|
| `devtools_list_frameworks` | List saved frameworks on disk |
| `devtools_save_framework(name)` | Save current enabled blocks as a reusable framework |
| `devtools_delete_framework(name)` | Remove a framework |

## Validation & errors

All write handlers strictly validate inputs:

- **Index validation**: negative or non-integer indices are rejected with `"'index' must be >= 0 (got N)"`.
- **Bounds checking**: out-of-range indices return `"Block index N out of range (len=M)"`.
- **Type whitelists**: `add_block` rejects unknown types with the full valid list. `set_model` rejects models not in `SUPPORTED_MODELS`.
- **Missing params**: every required parameter returns `"Missing 'X' parameter"` on absence.
- **Unknown tools**: JSON-RPC errors with code -32700 (parse) or custom messages.

## Known limitations

- `chat_messages` and `executions` are in-memory only — they are cleared when the app is killed.
- Multi-model execution, STT recording, steering rules, GitHub push, project versions, terminal session input, and hook engine triggers are **not** exposed via MCP yet. Use the app UI for these.
- The `Fleet` and `Collab` tabs are switchable via MCP but read-only (no write tools for GPU nodes / collab users yet).
