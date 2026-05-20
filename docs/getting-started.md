# Getting started

## Prerequisites

- Rust 1.75 or later
- A project repository you want to orchestrate

## Initialize a workspace

```bash
cd my-project
orca init
```

This creates a `.orca/` directory with:

- `config.yaml` — project configuration
- `state.json` — task state tracking
- `memory/` — persistent markdown notes
- `graph/` — lightweight relationship graph
- `context-packets/` — generated task context
- `prompts/` — agent prompt library
- `input/` — task input files
- `results/` — execution results
- `logs/` — execution logs

## First workflow

1. **Scan** the repository:
   ```bash
   orca scan
   ```

2. **Plan** work:
   ```bash
   orca plan --planner manual
   ```

3. **Run** the full dry-run loop:
   ```bash
   orca run --dry-run TASK-001
   ```

4. Check **status**:
   ```bash
   orca status
   ```

## Environment variables

Secrets are passed via environment variables, never committed:

- `ANTHROPIC_API_KEY` — Claude provider
- `OPENAI_API_KEY` — OpenAI/Codex provider
- `CURSOR_API_KEY` — Cursor provider
- `OLLAMA_HOST` — Ollama base URL

Use `${VAR}` or `${VAR:-default}` syntax in `config.yaml` to reference them.
