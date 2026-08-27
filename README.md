# AI App Router

[![CI](https://github.com/okiess/ai-app-router/actions/workflows/ci.yml/badge.svg)](https://github.com/okiess/ai-app-router/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`aiar` is a small Rust CLI that maps natural-language tasks to configured tools.
Agents and users ask "what should I use for X?" and get back the right CLI, web app, or API entry point.

## Installation

### From source

```bash
cargo install ai-app-router
```

### Pre-built binary

Download a release binary from the [releases page](https://github.com/okiess/ai-app-router/releases).

## Quick start

Create a configuration file at `~/.config/ai-app-router/config.toml`:

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
```

Then run:

```bash
aiar resolve "time tracking"
```

## Usage

### Resolve a task

```bash
aiar resolve "time tracking"           # human readable
aiar resolve "time tracking" --json    # structured output for agents
```

### List tools

```bash
aiar list
aiar list --tags --json
```

### Show a tool

```bash
aiar show fltw
```

### Add a tool

Non-interactive:

```bash
aiar add --id todo --name todo-cli --description "Task tracker" \
         --type cli --command todo --tags todo,tasks,planning
```

Interactive:

```bash
aiar add -i
```

Use `--force` to overwrite an existing tool.

### Which invocation?

```bash
aiar which markdown-vault
```

## Configuration

The config file is searched in this order:

1. `./.ai-app-router.toml`
2. `~/.config/ai-app-router/config.toml`
3. `~/.ai-app-router.toml`

### Tool types

- `cli` — a command-line tool (`command`)
- `webapp` — a web application (`url`)
- `api` — a REST API (`base_url`, optional `auth_env`)

See `examples/tools.toml` for a full example.

## Agent integration

See [`SKILL.md`](./SKILL.md) for the agent-facing usage contract.

## License

MIT — see [LICENSE](./LICENSE).
