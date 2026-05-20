# Memory system

Orca uses a file-based memory store under `.orca/memory/`.

## Layout

```
.orca/memory/
  tasks/
    TASK-001.md
    TASK-002.md
  decisions/
    ROUTING-001.md
  notes/
    architecture.md
```

## Obsidian compatibility

Notes are markdown with YAML frontmatter:

```markdown
---
id: TASK-001
title: Add auth middleware
type: task
status: complete
created: 2026-05-20T12:00:00Z
---

## Summary

Implemented JWT-based auth middleware.
```

This format is compatible with Obsidian and other markdown-based knowledge tools.

## Graph

A lightweight JSON graph tracks relationships:

- `nodes.json` — files, modules, concepts
- `edges.json` — dependencies, references, implementations

## Safe writes

All memory writes are atomic (temp file + rename) to prevent corruption.

## Secret redaction

Before writing to memory or logs, secrets are redacted:

- API keys
- Bearer tokens
- Private keys
- URL credentials
- Env file contents
