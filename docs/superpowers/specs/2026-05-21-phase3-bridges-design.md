# Phase 3 External Coding-Agent Bridge Design

## Date
2026-05-21

## Status
Approved — ready for implementation

## Overview
Add subprocess-based external bridge providers to Orca Agent CLI so it can route tasks to Claude Code, Codex, and Cursor Composer without rewriting the Rust core. Bridges execute configured external commands with structured prompts and capture stdout/stderr into standard Orca result artifacts.

## Principles

1. Rust core owns everything — routing, context, approval, review, memory.
2. External tools receive only a constructed prompt and return stdout.
3. No real paid-provider calls in automated tests.
4. All real external calls are opt-in via config and approval-gated.
5. Backwards compatibility with existing config and providers.

## Architecture

```
Orca Core (Rust)
  routing policy ──► decides provider
  prompt builder ──► constructs markdown prompt
  approval gate  ──► blocks if required
  bridge runner  ──► spawns subprocess, captures stdout/stderr
  provider trait ──► same interface as Ollama/OpenAI
  result artifact ──► same schema + bridge diagnostics
  review         ──► unchanged
```

## New Modules

| Module | Purpose |
|--------|---------|
| `src/bridge/mod.rs` | Module exports |
| `src/bridge/subprocess.rs` | Subprocess runner with timeout, capture, redaction |
| `src/bridge/config.rs` | `ExternalBridgeModelConfig` struct |

## Config Schema

### ExternalBridgeModelConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalBridgeModelConfig {
    #[serde(default)]
    pub enabled: bool,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_stdin_true")]
    pub stdin: bool,
    #[serde(default = "default_timeout_600")]
    pub timeout_seconds: u64,
    #[serde(default = "default_approval_true")]
    pub requires_approval: bool,
    pub model: Option<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_max_output_100000")]
    pub max_output_bytes: usize,
}
```

Notes:
- `enabled` defaults to `false`.
- `args` defaults to an empty vector.
- Only `enabled: true` requires `command` to be present.

### YAML Shape

```yaml
models:
  ollama:
    enabled: true
    # ... existing, unchanged
  claude_code:
    enabled: false
    command: claude
    args: ["--print"]
    stdin: true
    timeout_seconds: 600
    requires_approval: true
    model: claude-code
  codex:
    enabled: false
    command: codex
    args: []
    stdin: true
    timeout_seconds: 600
    requires_approval: true
    model: codex
  cursor:
    enabled: false
    command: node
    args: [".orca/bridges/cursor-bridge/index.js"]
    stdin: true
    timeout_seconds: 900
    requires_approval: true
    model: composer-2.5
    working_directory: "."
```

### Backwards Compatibility

- Existing `models.cursor` fields (`composer_model`, `premium_model`) preserved via `serde(default)`.
- New bridge fields default safely when absent.
- `enabled: false` means incomplete config is acceptable.
- `enabled: true` requires `command` present; validation errors are clear.

### Cursor Model Fallback

For `models.cursor` in bridge mode:
- `model` field is used if present.
- If `model` is absent, fall back to `composer_model` if present.
- `premium_model` remains parsed for backwards compatibility but is not used for Phase 3 routing unless existing code already uses it.
- `composer_model` and `premium_model` are not removed.

## Provider Integration

### Provider Name Derivation

Provider name is derived from the models key: `models.cursor` → provider `"cursor"`. Model label comes from the `model` field.

### ProviderKind

Add `ClaudeCode` and `Codex` variants to `ProviderKind` if the existing architecture requires enum variants for routing/factory. Otherwise use string-based lookup. Cursor already has a `ProviderKind::Cursor` variant.

### Factory

`create_provider(kind, config)` constructs the appropriate provider:
- For bridge kinds: wraps `ExternalBridgeModelConfig` in a generic `BridgeProvider`.
- Cursor: if `command` present and enabled, runs in bridge mode; otherwise preserves placeholder behavior.

### Routing Policy

Hierarchy:

1. **Ollama** — low-risk planning, summaries
2. **Cursor Composer** — repo-aware implementation (if enabled)
3. **Codex** — scoped implementation, tests (if enabled)
4. **Claude Code** — complex planning, architecture, review (if enabled)
5. **Fallback** — existing behavior (Manual, Mock)

Every routing decision includes a `reason` string.

## Approval Gates

All bridge providers default to `requires_approval: true`. Approval gate blocks unless `--yes` is passed. Dry-run shows approval requirement clearly.

## Subprocess Runner

- Executes `command` directly as argv[0], with `args` as subsequent entries.
- No shell interpolation.
- Supports stdin input (prompt text).
- Supports working directory override:
  - If `working_directory` is `None`, run from the Orca project root.
  - If `working_directory` is `Some(path)`, resolve relative paths against the Orca project root.
  - Never resolve relative paths against an arbitrary current shell directory.
- Supports environment variable injection.
- Captures stdout and stderr separately.
- Distinguishes: command not found, timeout, non-zero exit, invalid UTF-8, empty output.
- Returns structured errors compatible with `ProviderError`.

## Result Artifact Fields

Standard `ProviderResponse` plus bridge diagnostics:

```json
{
  "bridge": {
    "command": "claude",
    "exit_code": 0,
    "timed_out": false
  }
}
```

### Redaction and Truncation Order

1. Raw stdout/stderr captured.
2. Secrets redacted via `redact_secrets()`.
3. `max_output_bytes` applied.
4. Written to artifact.

### Token Estimates

When exact counts unavailable:
- `input_tokens` = prompt character count / 4
- `output_tokens` = output character count / 4

## Testing Strategy

- Subprocess runner tested with harmless commands (`echo`, `cmd /C echo hello` on Windows).
- Bridge provider tested with fake commands returning known outputs.
- No real Claude, Codex, or Cursor calls in any test.
- Routing tests use config-driven enabled/disabled flags.
- Approval gate tests verify bridge blocking.
- Exact-output acceptance criteria tested against bridge results.

## Stories (Implementation Order)

1. **STORY 1** — Subprocess bridge runner (`src/bridge/`)
2. **STORY 2** — Bridge config schema (`ExternalBridgeModelConfig`)
3. **STORY 3** — Generic bridge provider (`BridgeProvider` implementing `Provider`)
4. **STORY 4** — Claude Code bridge provider + routing
5. **STORY 5** — Codex bridge provider + routing
6. **STORY 6** — Cursor Composer bridge provider + routing (extends existing placeholder)
7. **STORY 7** — Approval gates for bridge providers
8. **STORY 8** — CLI diagnostics / provider availability check
9. **STORY 9** — Result/review integration polish
10. **STORY 10** — Documentation (`docs/phase3-bridges.md`, update `README.md`, `docs/providers.md`)

## Quality Gates

Run before each meaningful commit:

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build
```

All existing tests must continue passing.

## Out of Scope

- Local llama.cpp provider
- Qdrant retrieval expansion
- Obsidian graph memory
- Cursor premium model escalation
- Automatic multi-agent planning loops
- Long-running daemon/watch mode
- Background scheduling
- Browser automation
- GitHub PR automation
- Complex patch application system
- Direct TypeScript SDK implementation inside Rust core
- MCP server adapter
- Cursor extension (TypeScript)
