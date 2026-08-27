# Skill: AI App Router

Use the `aiar` CLI to discover which configured tool (CLI, webapp, or API) should handle a given task.

## When to use

- The user asks for a task that has a dedicated tool, but you are not sure which one to use.
- You need to invoke a configured CLI, open a webapp, or call an API entry point.

## How to use

Run:

```bash
aiar resolve "<task description>" --json
```

Example:

```bash
aiar resolve "time tracking" --json
```

### Interpreting the result

The JSON response has the following shape:

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

Field meanings:

- `match` — the best matching tool. If the top candidates are too close, this is omitted and all options are in `candidates`.
- `candidates` — alternative tools sorted by confidence.
- `type` — one of `cli`, `webapp`, `api`.
- `invocation` — the command, URL, or base URL to use.
- `auth_env` (only shown for `api` tools) — the environment variable that holds credentials.

### What to do next per type

- **cli**: run the returned `invocation` as a shell command.
- **webapp**: open the returned `url` in the user's browser.
- **api**: make HTTP requests to `base_url` using the credentials in the named `auth_env` variable if provided.

## Discover available tools

To get an overview, run:

```bash
aiar list
```

Add `--json` for structured output.

## Common mappings

| Task | Tool | Type |
| --- | --- | --- |
| time tracking, timesheet | fltw | cli |
| dividends, stocks, investing | dividends-cli | cli |
| markdown vault, brain, notes | mvapi | cli |
| notion, wiki, docs | notion | webapp |

## Example queries

- `"time tracking"` → fltw
- `"check dividends"` → dividends-cli
- `"read brain note"` → mvapi
- `"open notion"` → notion
