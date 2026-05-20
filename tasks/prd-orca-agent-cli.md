# PRD: Orca Agent CLI (Rust MVP)

## Project
Build a production-quality Rust CLI named `orca` that orchestrates AI coding tasks through project memory, context packets, model routing, and provider adapters.

## Language
Rust (edition 2021). Binary `orca`, crate `orca-agent-cli`.

## Core Philosophy
- Project memory is the source of truth, not chat history.
- Token minimization by design: context packets, compression, and cheap-first routing.
- Human approval gates for premium/risky operations.
- Dry-run and mock providers work without real API keys.

## Architecture Overview

```
CLI (clap)
  ├─ Config (serde_yaml + typed schema)
  ├─ Commands (init, config, scan, context, route, plan, execute, review, memory, run, status)
  ├─ Router (policy + decision engine)
  ├─ Providers (trait-based adapters)
  │   ├─ Mock
  │   ├─ Ollama
  │   ├─ Anthropic (Claude)
  │   ├─ OpenAI / Codex
  │   ├─ Cursor (placeholder + command adapter)
  │   └─ Command / subprocess
  ├─ Memory (file-based store + Obsidian notes + lightweight graph)
  ├─ Context (packet schema + markdown/JSON renderer)
  ├─ Tasks (model + lifecycle)
  ├─ Review (result reviewer)
  ├─ Approvals (gate system)
  ├─ Prompts (prompt library as Rust string constants)
  └─ Utils (fs helpers, git helpers, secret redaction)
```

## MVP Commands
1. `orca init`
2. `orca config validate`
3. `orca scan`
4. `orca context <TASK-ID>`
5. `orca route <TASK-ID>`
6. `orca plan`
7. `orca execute --dry-run <TASK-ID>`
8. `orca review <TASK-ID>`
9. `orca memory update <TASK-ID>`
10. `orca run --dry-run <TASK-ID>`
11. `orca status`

## Config System
- Default path: `.orca/orca.config.yaml`
- Supports project name, repo path, memory paths, provider settings, routing policy, budget policy, approval policy, quality commands.
- Env var substitution for secrets: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `CURSOR_API_KEY`, `OLLAMA_BASE_URL`, `OLLAMA_CLOUD_API_KEY`, `CURSOR_COMPOSER_MODEL_ID`, `CURSOR_PREMIUM_MODEL_ID`.
- Config validation with helpful errors.

## Provider System
- Trait `Provider` with methods for name, kind, capabilities, cost estimate, execute, supports.
- `ProviderKind` enum: Mock, Ollama, Anthropic, OpenAi, Cursor, Command.
- `ProviderCapability` enum.
- `ProviderRequest` / `ProviderResponse` structs.
- `CostEstimate` struct.
- `ProviderError` enum using `thiserror`.
- Mock provider must work with no keys.
- Cursor adapter is a safe placeholder with command fallback; no Node.js core dependency.

## Routing Policy
- Ollama: summaries, compression, memory updates, docs, cheap review, embeddings.
- Cursor Composer 2.5 (configurable via `CURSOR_COMPOSER_MODEL_ID`): repo-aware coding, multi-file implementation, semantic work, moderate bugs, tests, small/medium refactors.
- Codex / GPT: clean scoped implementation, patch generation, test generation, isolated modules, exact context packets.
- Claude Opus: architecture, planning, decomposition, risk analysis, failure diagnosis.
- Cursor premium (configurable via `CURSOR_PREMIUM_MODEL_ID`): last resort, high-risk repo-wide debugging, failed tasks, requires manual approval.

## Memory System
- File-based under `.orca/memory/`.
- Obsidian-compatible vault output if configured.
- Lightweight graph memory as JSON nodes/edges under `.orca/graph/`.
- Context packets under `.orca/context-packets/`.

## Context Packet System
- Typed Rust struct with task summary, goal, constraints, relevant files, architecture notes, risks, acceptance criteria, routing recommendation, suggested provider, test commands, memory links, prior attempts.
- Render to markdown and JSON.

## Task Model
- ID, title, description, type, complexity, risk, status, acceptance criteria, relevant files, required capabilities, routing decision, attempts, result summary, memory update proposal.

## Safety
- Approval gates for premium Cursor and destructive fs operations.
- Secret redaction before logging or model calls.
- Non-zero exit codes on failure.

## Testing
- Config loading/validation tests.
- Routing decision tests.
- Provider mock tests.
- Memory file write tests.
- Context packet generation tests.
- Approval gate tests.
- Secret redaction tests.
- CLI help smoke tests with `assert_cmd`.

## Quality Gates (every story)
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo build`
