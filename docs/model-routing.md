# Model routing

Orca routes tasks to the cheapest capable provider based on task metadata.

## Routing rules

| Task type | Provider | Reason |
|-----------|----------|--------|
| Summaries, compression, docs | Ollama | Cheap, local |
| Architecture, planning, decomposition | Claude Opus | Reasoning |
| Repo-aware implementation | Cursor Composer | Context window |
| Scoped exact implementation | Codex | Precision |
| Debugging after failures | Cursor premium | Escalation |

## Factors

The router considers:

- **Task type** — summary, architecture, implementation, debugging
- **Complexity** — low, medium, high
- **Risk** — safe, moderate, high, destructive
- **File count** — more files may need larger context
- **Failure history** — repeated failures trigger escalation
- **Budget policy** — conservative vs. aggressive spend

## Approval gates

Some routing decisions require approval:

- Premium providers (Cursor premium)
- High-risk tasks
- Destructive operations (file deletion, overwrites)

Bypass with `--yes` if configured.

## Fallback

Every routing decision includes a fallback provider. If the primary fails, Orca can retry with the fallback.
