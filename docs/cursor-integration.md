# Cursor integration

Orca routes repo-aware implementation tasks to Cursor Composer, but the Rust core does not embed Cursor directly.

## Strategy

Cursor is a desktop IDE with its own AI subsystem. Rather than trying to run Cursor headless from Rust (which would require Node.js/electron), Orca treats Cursor as an external execution agent:

1. Orca **plans** and **routes** the task
2. Orca generates a **context packet** with relevant files
3. A human or external script opens the context packet in Cursor
4. Cursor performs the edits using its native Composer
5. Results are fed back into Orca's memory system

## Context packets

Context packets include:

- Goal and constraints
- Relevant file paths
- Architecture notes
- Risks and acceptance criteria
- Suggested provider and model

These are written to `.orca/context-packets/{TASK-ID}.md`.

## Future directions

- Cursor CLI integration if/when a native CLI becomes available
- MCP server adapter for Cursor's context protocol
- Export context packets directly into Cursor workspace files

## Configuration

Cursor is disabled by default. Enable in `config.yaml`:

```yaml
models:
  cursor:
    enabled: true
    composer_model: "${CURSOR_COMPOSER_MODEL_ID}"
    premium_model: "${CURSOR_PREMIUM_MODEL_ID}"
```

`CURSOR_COMPOSER_MODEL_ID` and `CURSOR_PREMIUM_MODEL_ID` are never hardcoded.
