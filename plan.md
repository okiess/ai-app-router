# AI App Router — Project Plan

## Goal

A Rust CLI that maps a natural-language task to a configured tool.
Agents and users ask "what should I use for X?" and get back the right CLI, webapp, or API entry point.

## Scope (MVP)

- Configuration file with tool definitions.
- Tool types: `cli`, `webapp`, `api`.
- Commands: `list`, `show`, `resolve`, `which`.
- Human-readable and JSON output.
- Fuzzy tag matching with confidence scores.
- Agent skill documentation (`SKILL.md`).

## Out of Scope (for now)

- `exec` command.
- Subcommand schemas.
- Automatic indexing of external agent command directories.

## Configuration

### Search order

1. `./.ai-app-router.toml`
2. `~/.config/ai-app-router/config.toml`
3. `~/.ai-app-router.toml`

### Schema

```toml
[tool.fltw]
name = "fltw"
description = "Freelance Timewise CLI"
tags = ["time tracking", "timesheet", "freelance", "work"]
type = "cli"
command = "fltw"

[tool.dividends]
name = "dividends-cli"
description = "Dividend portfolio CLI"
tags = ["dividends", "dividenden", "stocks", "investing"]
type = "cli"
command = "dividends-cli"

[tool.mvapi]
name = "mvapi"
description = "Markdown Vault API CLI"
tags = ["markdown vault", "brain", "notes", "vault"]
type = "cli"
command = "mvapi"

[tool.notion]
name = "notion"
description = "Notion workspace"
tags = ["notes", "wiki", "docs"]
type = "webapp"
url = "https://notion.so"

[tool.brain-api]
name = "brain-api"
description = "AI Brain REST API"
tags = ["brain", "vault", "api"]
type = "api"
base_url = "https://brain.oliver-kiessler-ki-agent.de"
auth_env = "BRAIN_API_KEY"
```

## Tool model

```
Tool {
  id: String
  name: String
  description: String
  tags: Vec<String>
  tool_type: ToolType
}

ToolType {
  Cli { command: String }
  Webapp { url: String }
  Api { base_url: String, auth_env: Option<String> }
}
```

## Matching logic

1. Exact tag match (confidence 1.0).
2. Substring match in tags, name, or description.
3. Token overlap between query and tags/name/description.
4. Levenshtein distance fallback for typos.
5. If multiple tools have the same best score, return all as candidates.

## CLI

```bash
aiar list                       # all tools
aiar list --tags                # include tags in output
aiar list --json

aiar resolve "time tracking"    # best match + metadata + invocation
aiar resolve "dividenden" --json

aiar which markdown-vault       # only command/url for shell pipes

aiar show fltw                  # detail view of one tool
```

## JSON output (resolve)

```json
{
  "query": "time tracking",
  "match": {
    "id": "fltw",
    "name": "fltw",
    "type": "cli",
    "description": "Freelance Timewise CLI",
    "invocation": "fltw",
    "matched_tag": "time tracking",
    "confidence": 1.0
  },
  "candidates": []
}
```

## Project structure

```
ai-app-router/
├── Cargo.toml
├── plan.md
├── SKILL.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── tool.rs
│   ├── resolver.rs
│   └── output.rs
└── examples/
    └── tools.toml
```

## Dependencies

- `clap` — CLI arguments
- `serde` + `toml` — configuration
- `anyhow` — error handling
- `strsim` — fuzzy string matching
- `comfy-table` — human-readable tables

## Acceptance criteria

- `cargo build` succeeds.
- `aiar list` prints all configured tools.
- `aiar resolve "time tracking"` resolves to `fltw`.
- `aiar resolve "dividenden"` resolves to `dividends-cli`.
- `aiar show fltw` prints details.
- JSON output is parseable and contains `match` + `candidates`.
- Example config in `examples/tools.toml` works with all commands.
