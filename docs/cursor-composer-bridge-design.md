# Cursor Composer Bridge Design

## Status
Design-only document. No implementation code is written as part of this story.

## Problem

Orca routes repo-aware implementation tasks to Cursor Composer, but Cursor is a desktop IDE with no public headless API. We need a bridge that lets Orca send tasks to Cursor Composer and receive results back, without embedding Node.js/Electron in the Rust core.

## Constraints

1. **No Node.js in core** — The Rust binary must not depend on Electron or a Node runtime.
2. **Cursor has no public REST API** — We cannot call Cursor over HTTP like Ollama or OpenAI.
3. **Cursor Composer is GUI-driven** — It runs inside the Cursor desktop app and operates on the open workspace.
4. **Dry-run and mock must still work** — The bridge must not break existing dry-run behavior.
5. **Secrets must not leak** — Bridge files must never contain API keys or tokens.

## Approaches Considered

### A. Enhanced Context Packet Export (Human-in-the-Loop)

Orca generates a context packet and writes it to `.orca/context-packets/{TASK-ID}.md`. A human opens the file in Cursor Composer, executes the task, and pastes the result back into Orca's memory system.

**Pros**
- Works today with zero new code.
- No Cursor extension required.
- Fully compatible with dry-run.

**Cons**
- Not automated; requires manual copy/paste.
- No structured result capture.
- Round-trip latency is human-dependent.

### B. `.cursorrules` File Bridge

Orca writes task context into a `.cursorrules` file in the project root. Cursor Composer reads this file automatically when loading the workspace, using it as system instructions.

**Pros**
- Cursor natively supports `.cursorrules` (no extension needed).
- Persistent context across Composer sessions.
- Simple file-based protocol.

**Cons**
- `.cursorrules` is meant for project-level rules, not per-task instructions.
- No structured way to get results back.
- Overwriting `.cursorrules` per task could interfere with user-defined rules.

### C. File-Based Bridge Protocol with Lightweight Cursor Extension

Orca writes a structured task file to `.orca/cursor-bridge/inbox/{TASK-ID}.json`. A small Cursor extension (JavaScript/TypeScript, running inside Cursor's extension host) watches the inbox, loads the task into Composer, captures the result, and writes it to `.orca/cursor-bridge/outbox/{TASK-ID}.json`. Orca polls the outbox for completion.

**Pros**
- Fully automated bidirectional communication.
- Structured data exchange (JSON schema).
- Extension runs inside Cursor's existing Node/Electron host, so no extra runtime is needed.
- Rust core stays pure; extension is optional.

**Cons**
- Requires building and distributing a Cursor extension.
- Extension API stability depends on Cursor.
- Polling adds complexity.

### D. MCP Server Adapter

Orca exposes a Model Context Protocol (MCP) server. Cursor connects to it as an MCP client, allowing Cursor to call Orca tools and vice versa.

**Pros**
- Standard protocol with growing ecosystem support.
- Cursor has added MCP client support.
- Bidirectional tool invocation.

**Cons**
- MCP is primarily for tool exposure, not task delegation.
- Requires Cursor to initiate connections; not ideal for headless orchestration.
- Overkill for the current use case.

## Recommended Approach: Layered Bridge

Use a **layered strategy** that starts simple and grows toward full automation:

| Phase | Bridge Layer | Trigger | Result Capture |
|-------|--------------|---------|----------------|
| 1 | Enhanced context packet export | `orca context TASK-001` | Manual paste |
| 2 | `.cursorrules` + context packet | `orca context TASK-001` | Manual paste |
| 3 | File-based protocol (no extension) | `orca execute TASK-001` | Skeleton only |
| 4 | File-based protocol + Cursor extension | `orca execute TASK-001` | Auto-captured |
| 5 | MCP server adapter | Future | Future |

### Phase 1 — Enhanced Context Packet Export (Now)

Extend the existing context packet with a **Composer-specific section**:

- `composer_prompt` — A pre-formatted prompt optimized for Cursor Composer.
- `relevant_files` — Absolute paths to files that should be opened or referenced.
- `test_command` — Command to run after edits (e.g., `cargo test`).
- `acceptance_criteria` — Checklist Composer can self-verify against.

Orca writes the packet to `.orca/context-packets/{TASK-ID}.md`. The user opens it in Cursor Composer.

### Phase 2 — `.cursorrules` Integration (Next)

When a task is routed to Cursor Composer, Orca **appends** task-specific instructions to a `.cursorrules` file in the project root (or creates one if absent). The file includes:

- Project context (from `orca scan`).
- Current task goal and constraints.
- Acceptance criteria.

Orca backs up the existing `.cursorrules` before modification and restores it after task completion.

### Phase 3 — File-Based Protocol (Skeleton)

Introduce a `CursorBridge` module in Rust that:

1. Writes a `BridgeTask` JSON file to `.orca/cursor-bridge/inbox/{TASK-ID}.json`.
2. The `CursorProvider::execute` implementation writes the bridge file and returns a skeleton response indicating the bridge file is ready.
3. If a future extension exists, it processes the inbox file.

```json
{
  "bridge_version": "1.0",
  "task_id": "TASK-001",
  "timestamp": "2026-05-20T12:00:00Z",
  "prompt": "Refactor the auth module...",
  "relevant_files": ["src/auth.rs", "src/main.rs"],
  "acceptance_criteria": ["Tests pass", "No clippy warnings"],
  "test_command": "cargo test",
  "output_format": "diff"
}
```

### Phase 4 — Cursor Extension (Future)

Build a minimal Cursor extension that:

- Watches `.orca/cursor-bridge/inbox/` via `fs.watch`.
- On new file, reads the task and opens the relevant files in Composer.
- Runs the prompt through Composer.
- Captures the generated diff or file changes.
- Writes a `BridgeResult` JSON to `.orca/cursor-bridge/outbox/{TASK-ID}.json`.

```json
{
  "bridge_version": "1.0",
  "task_id": "TASK-001",
  "status": "success",
  "files_changed": [
    {
      "path": "src/auth.rs",
      "diff": "..."
    }
  ],
  "summary": "Refactored auth module to use...",
  "test_output": "test result: ok. 42 passed",
  "duration_ms": 45000
}
```

Orca polls the outbox or uses a file-watcher to detect completion.

### Phase 5 — MCP Server (Future)

If Cursor's MCP client support matures and Orca needs bidirectional tool access, expose an MCP server from the Rust core. This is out of scope for the current design.

## Data Formats

### BridgeTask Schema (v1.0)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| bridge_version | string | yes | "1.0" |
| task_id | string | yes | Orca task ID |
| timestamp | string (ISO 8601) | yes | Creation time |
| prompt | string | yes | Full prompt for Composer |
| relevant_files | string[] | no | Absolute paths to open |
| acceptance_criteria | string[] | no | Criteria to verify |
| test_command | string | no | Post-edit verification command |
| output_format | string | no | "diff", "full_file", or "summary" |
| context_packet_path | string | no | Path to the `.md` context packet |

### BridgeResult Schema (v1.0)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| bridge_version | string | yes | "1.0" |
| task_id | string | yes | Matches BridgeTask |
| status | string | yes | "success", "failure", "cancelled", "partial" |
| files_changed | FileChange[] | no | List of changes |
| summary | string | no | Human-readable summary |
| test_output | string | no | Output from test_command |
| error_message | string | no | Error details if status != success |
| duration_ms | number | no | Time spent in Composer |

### FileChange Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| path | string | yes | Relative file path |
| diff | string | no | Unified diff or null if new file |
| full_content | string | no | Complete file content if output_format == "full_file" |
| explanation | string | no | Why this change was made |

## Execution Flow

```text
Orca CLI                      Cursor Bridge                     Cursor Composer
  |                                 |                                   |
  |-- route TASK-001 ------------->|                                   |
  |                                 |                                   |
  |-- context TASK-001 ----------->|                                   |
  |                                 |-- writes context packet ---------->| (manual)
  |                                 |-- writes .cursorrules ----------->| (auto)
  |                                 |-- writes BridgeTask ------------->| (via ext)
  |                                 |                                   |
  |-- execute TASK-001 ---------->|                                   |
  |                                 |-- detects bridge state            |
  |                                 |-- Phase 1/2: skeleton response     |
  |                                 |-- Phase 4: polls outbox           |
  |<-- ProviderResponse -----------|                                   |
  |                                 |<-- BridgeResult (via ext) --------|
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Cursor not running | Phase 1/2 still work (human opens file later). Phase 4 returns skeleton with a note that Cursor is not connected. |
| Bridge file write fails | `ProviderError::IoError` with clear message. No partial files left. |
| Outbox poll timeout | Configurable timeout (default 5 min). Returns `ProviderError::ApiError("Cursor bridge timed out")`. |
| Invalid BridgeResult JSON | Log warning, return `ProviderError::ApiError("Invalid bridge result")`. |
| Extension not installed | Graceful degradation to Phase 2 behavior. |

## Security

1. **No secrets in bridge files** — API keys, tokens, and env vars are redacted before writing.
2. **Atomic writes** — Bridge files are written via `safe_write` (temp file + rename).
3. **Path validation** — `relevant_files` are validated against the project repo path to prevent path traversal.
4. **Backup and restore** — `.cursorrules` is backed up before modification.

## Provider Interface Changes (Proposed)

The `CursorProvider` will be updated in a future implementation story to:

1. Accept a `CursorBridgeConfig` with bridge phase, inbox/outbox paths, and timeout.
2. Implement `execute` as a state machine:
   - Phase 1/2: Write files, return skeleton response.
   - Phase 4: Write `BridgeTask`, poll for `BridgeResult`, parse into `ProviderResponse`.
3. Add `RepoAware` and `MultiFileEdit` capabilities (already declared in skeleton).

## Implementation Roadmap

| Story | Work |
|-------|------|
| PHASE2-008 (this story) | Design document approved |
| Future — Cursor bridge Phase 2 | Add `.cursorrules` backup/restore to `CursorProvider` |
| Future — Cursor bridge Phase 3 | Implement `BridgeTask` writer in `CursorProvider::execute` |
| Future — Cursor bridge Phase 4 | Build Cursor extension (TypeScript), add outbox polling to Rust |
| Future — Cursor bridge Phase 5 | MCP server adapter (optional) |

## Appendix: Context Packet Composer Prompt Template

```markdown
# Task: {task_id} — {title}

## Goal
{goal}

## Relevant Files
{files}

## Constraints
{constraints}

## Acceptance Criteria
{criteria}

## Test Command
```bash
{test_command}
```

Please implement the above. When done, summarize changes and test results.
```

This template will be added to the prompt library as `CURSOR_COMPOSER_CONTEXT_PROMPT` in a future implementation story.
