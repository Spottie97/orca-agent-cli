# Phase 3: External Coding-Agent Bridges

Phase 3 adds subprocess-based bridge providers that let Orca delegate tasks to external coding agents: **Claude Code**, **Codex**, and **Cursor Composer**. These bridges use the `BridgeProvider` implementation, which wraps external CLI commands and captures their output.

## Overview

Bridge providers turn Orca into an orchestration layer for specialized AI coding tools:

- **Claude Code** — handles high-complexity planning and architecture tasks
- **Codex** — executes scoped implementation tasks with precise file targeting
- **Cursor Composer** — performs repo-aware multi-file editing (when configured with a bridge command)

Each bridge is optional. When not configured, the router falls back to existing HTTP providers (Anthropic, OpenAI, Ollama).

## Configuration

Bridge providers are configured under `models:` in `.orca/config.yaml`.

### Claude Code Bridge

```yaml
models:
  claude_code:
    enabled: true
    command: claude
    args: []
    stdin: true
    timeout_seconds: 600
    requires_approval: true
    model: claude-code
    working_directory: null
    env: {}
    max_output_bytes: 100000
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Whether the bridge is active |
| `command` | `null` | Path to the Claude Code CLI executable |
| `args` | `[]` | Additional arguments passed to the command |
| `stdin` | `true` | Send prompts via stdin instead of arguments |
| `timeout_seconds` | `600` | Subprocess timeout |
| `requires_approval` | `true` | Block execution unless `--yes` is passed |
| `model` | `claude-code` | Model label for routing and reporting |
| `working_directory` | `null` | Working directory for the subprocess |
| `env` | `{}` | Extra environment variables |
| `max_output_bytes` | `100000` | Max stdout/stderr bytes to capture |

### Codex Bridge

```yaml
models:
  codex:
    enabled: true
    provider: codex
    default_model: gpt-5.5
    role: scoped_execution
    command: codex
    args: []
    stdin: true
    timeout_seconds: 600
    requires_approval: true
    model: null
    working_directory: null
    env: {}
    max_output_bytes: 100000
```

The Codex bridge uses the same `ExternalBridgeModelConfig` schema as Claude Code. When `command` is present, Orca creates a `BridgeProvider` instead of using the HTTP OpenAI-compatible skeleton.

### Cursor Composer Bridge

```yaml
models:
  cursor:
    enabled: true
    composer_model_id: composer-2.5
    premium_model_id: cursor-premium
    provider: cursor_sdk
    default_runtime: local
    cloud_auto_create_pr: false
    command: cursor
    args: []
    stdin: true
    timeout_seconds: 900
    requires_approval: true
    model: null
    working_directory: null
    env: {}
    max_output_bytes: 100000
```

When `command` is present and `enabled` is `true`, the factory creates a `BridgeProvider` for Cursor. If `command` is absent, the existing placeholder `CursorProvider` is used.

## Routing

The router selects bridge providers automatically when they are enabled:

| Task Type | Condition | Selected Provider |
|-----------|-----------|-------------------|
| Planning | High complexity + `claude_code.enabled` | ClaudeCode |
| Implementation | Scoped + `codex.command` present | Codex |
| Repo-aware | `cursor.command` present | Cursor (bridge mode) |

If a bridge provider is disabled or unconfigured, the router falls back to the next suitable provider:

- ClaudeCode falls back to Anthropic
- Codex falls back to OpenAI / default scoped executor
- Cursor bridge falls back to the placeholder Cursor provider

## Approval Gates

All bridge providers require manual approval by default. The approval gate blocks execution with a message like:

```
claude_code bridge provider requires manual approval. Use --yes to bypass.
```

Pass `--yes` to auto-approve all gates:

```bash
orca execute --yes TASK-001
orca run --yes TASK-001
```

Set `requires_approval: false` in config to disable the gate for a specific bridge provider.

## Diagnostics

Use `orca providers` to inspect bridge provider configuration and command availability:

```bash
$ orca providers
Bridge Provider Diagnostics
============================

Provider: claude_code
  Enabled:           true
  Command:           claude
  Command on PATH:   yes
  Model:             claude-code
  Requires approval: yes

Provider: codex
  Enabled:           true
  Command:           codex
  Command on PATH:   no
  Model:             gpt-5.5
  Requires approval: yes

Provider: cursor
  Enabled:           false
  Command:           (not configured)
  Command on PATH:   no
  Model:             composer-2.5
  Requires approval: yes
```

JSON output is supported:

```bash
orca providers --json
```

## Result Integration

Bridge executions populate `ProviderResponse` with `bridge_diagnostics`:

```json
{
  "bridge_diagnostics": {
    "command": "claude",
    "exit_code": 0,
    "timed_out": false
  }
}
```

These diagnostics are preserved in execution result artifacts under `.orca/results/` and are available for review. The review pipeline handles bridge results identically to HTTP provider results, including exact-output acceptance criteria enforcement.

## Error Handling

Bridge providers map subprocess errors to `ProviderError` variants:

| Subprocess Error | Provider Error | Meaning |
|------------------|----------------|---------|
| Command not found | `NotConfigured` | Bridge CLI is not installed |
| Timeout | `ExecutionFailed` | Task exceeded `timeout_seconds` |
| Non-zero exit | `ExecutionFailed` | The external tool returned an error |
| Empty output | `ExecutionFailed` | The bridge produced no stdout |
| Invalid UTF-8 | `ExecutionFailed` | Output contained binary data |

## Security

- Secrets in bridge output are redacted before logging, review, or memory updates
- Bridge commands run with the same privileges as Orca
- Working directory and env vars are configurable per bridge provider
- Approval gates prevent accidental bridge execution

## Migration from Phase 2

Existing `.orca/config.yaml` files are fully backward compatible. New bridge fields have serde defaults and do not need to be present. The Cursor and Codex configurations continue to work as HTTP skeletons until `command` is explicitly added.
