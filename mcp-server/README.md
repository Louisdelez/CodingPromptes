# Inkwell MCP Server

Expose your Inkwell projects and specs to AI coding agents (Claude Code, Codex, OpenCode).

## Installation

```bash
cd mcp-server
npm install
npm run build
```

## Configuration

### Claude Code

Add to `~/.claude/mcp.json`:

```json
{
  "mcpServers": {
    "inkwell": {
      "command": "node",
      "args": ["/path/to/mcp-server/dist/index.js"]
    }
  }
}
```

### Codex (OpenAI)

```bash
codex --mcp-server inkwell="node /path/to/mcp-server/dist/index.js"
```

### OpenCode

Add to `opencode.json`:

```json
{
  "mcp": {
    "inkwell": {
      "type": "stdio",
      "command": "node",
      "args": ["/path/to/mcp-server/dist/index.js"]
    }
  }
}
```

## Available Tools

| Tool | Description |
|------|-------------|
| `list_projects` | List all projects (filter by type: "prompt" or "spec") |
| `read_project` | Read a full project (blocks or SDD phases) |
| `read_phase` | Read a specific SDD phase |
| `read_tasks` | Get parsed tasks from a spec project |
| `search_projects` | Search across all projects |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `INKWELL_DB_PATH` | `~/.local/share/inkwell-server/data.db` | Path to Inkwell SQLite database |
