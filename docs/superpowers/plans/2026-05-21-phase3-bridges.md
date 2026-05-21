# Phase 3 External Coding-Agent Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add subprocess-based external bridge providers (Claude Code, Codex, Cursor Composer) to Orca Agent CLI so tasks can be routed to external coding-agent tools while preserving the Rust core.

**Architecture:** A reusable subprocess runner (`src/bridge/`) executes configured external commands with prompts via stdin. A generic `BridgeProvider` implements the existing `Provider` trait. Bridge config is embedded in `ModelsConfig` under named keys (`claude_code`, `codex`, `cursor`). Routing policy extends existing provider selection without breaking current behavior.

**Tech Stack:** Rust 2021, tokio::process, serde, existing provider trait, existing routing/approval/result/review infrastructure.

---

## File Structure

### New Files

| File | Responsibility |
|------|----------------|
| `src/bridge/mod.rs` | Module exports for bridge subsystem |
| `src/bridge/subprocess.rs` | Subprocess runner with timeout, capture, redaction, structured errors |
| `src/bridge/config.rs` | `ExternalBridgeModelConfig` struct with serde defaults |
| `src/providers/bridge.rs` | Generic `BridgeProvider` implementing the `Provider` trait |

### Modified Files

| File | Changes |
|------|---------|
| `src/providers/traits.rs` | Add `ClaudeCode` and `Codex` to `ProviderKind` |
| `src/providers/mod.rs` | Declare `bridge` module, re-export `BridgeProvider` |
| `src/providers/factory.rs` | Add factory arms for ClaudeCode, Codex, Cursor bridge mode |
| `src/config/schema.rs` | Add `ExternalBridgeModelConfig` to `ModelsConfig`, extend `CursorModelConfig` with bridge fields |
| `src/router/policy.rs` | Add `provider_enabled` / `provider_from_name` / `model_for_provider` support for bridge providers; add bridge-aware routing decisions |
| `src/approvals/gate.rs` | Extend approval checks for all bridge providers |
| `src/lib.rs` | Declare `bridge` module |
| `tests/cli_tests.rs` | Add integration tests for bridge routing, dry-run, execution |

---

## Task 1: Subprocess Bridge Runner

**Files:**
- Create: `src/bridge/mod.rs`
- Create: `src/bridge/config.rs`
- Create: `src/bridge/subprocess.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create bridge module scaffolding**

Create `src/bridge/mod.rs`:

```rust
pub mod config;
pub mod subprocess;

pub use config::ExternalBridgeModelConfig;
pub use subprocess::{run_subprocess, SubprocessError, SubprocessResult};
```

Create `src/bridge/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn default_stdin_true() -> bool {
    true
}

fn default_timeout_600() -> u64 {
    600
}

fn default_approval_true() -> bool {
    true
}

fn default_max_output_100000() -> usize {
    100_000
}
```

- [ ] **Step 2: Implement subprocess runner**

Create `src/bridge/subprocess.rs`:

```rust
use crate::utils::redact::redact_secrets;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct SubprocessResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SubprocessError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("Timed out after {0}s")]
    Timeout(u64),
    #[error("Non-zero exit code: {0}")]
    NonZeroExit(i32),
    #[error("Invalid UTF-8 in output")]
    InvalidUtf8,
    #[error("IO error: {0}")]
    Io(String),
    #[error("Empty output")]
    EmptyOutput,
}

pub async fn run_subprocess(
    command: &str,
    args: &[String],
    stdin_input: Option<&str>,
    working_dir: Option<&std::path::Path>,
    env_vars: &HashMap<String, String>,
    timeout_seconds: u64,
    max_output_bytes: usize,
) -> Result<SubprocessResult, SubprocessError> {
    let start = Instant::now();
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.envs(env_vars);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Spawn child
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SubprocessError::CommandNotFound(command.to_string())
            } else {
                SubprocessError::Io(e.to_string())
            }
        })?;

    // Write stdin if provided
    if let Some(input) = stdin_input {
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| SubprocessError::Io(e.to_string()))?;
        }
    }

    // Wait with timeout
    let result = timeout(Duration::from_secs(timeout_seconds), child.wait()).await;

    match result {
        Ok(Ok(status)) => {
            let exit_code = status.code().unwrap_or(-1);
            let stdout = read_pipe(child.stdout, max_output_bytes).await;
            let stderr = read_pipe(child.stderr, max_output_bytes).await;

            let stdout = redact_secrets(&stdout);
            let stderr = redact_secrets(&stderr);

            if stdout.is_empty() && stderr.is_empty() {
                return Err(SubprocessError::EmptyOutput);
            }

            Ok(SubprocessResult {
                stdout,
                stderr,
                exit_code,
                timed_out: false,
                duration_ms: start.elapsed().as_millis() as u64,
            })
        }
        Ok(Err(e)) => Err(SubprocessError::Io(e.to_string())),
        Err(_) => {
            let _ = child.kill().await;
            Err(SubprocessError::Timeout(timeout_seconds))
        }
    }
}

async fn read_pipe(
    pipe: Option<tokio::process::ChildStdout>,
    max_bytes: usize,
) -> String {
    use tokio::io::AsyncReadExt;
    if let Some(mut pipe) = pipe {
        let mut buf = Vec::new();
        if let Err(_) = pipe.read_to_end(&mut buf).await {
            return String::new();
        }
        if buf.len() > max_bytes {
            buf.truncate(max_bytes);
            let mut s = String::from_utf8_lossy(&buf).to_string();
            s.push_str("\n[truncated]");
            return s;
        }
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}
```

Wait — `read_pipe` above only handles `ChildStdout`. We need a generic version for both stdout and stderr. Let me rewrite that part:

```rust
async fn read_pipe_generic<T>(
    pipe: Option<T>,
    max_bytes: usize,
) -> String
where
    T: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    if let Some(mut pipe) = pipe {
        let mut buf = Vec::new();
        if pipe.read_to_end(&mut buf).await.is_err() {
            return String::new();
        }
        if buf.len() > max_bytes {
            buf.truncate(max_bytes);
            let mut s = String::from_utf8_lossy(&buf).to_string();
            s.push_str("\n[truncated]");
            return s;
        }
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}
```

Actually, we can use a simpler approach with tokio::io::AsyncReadExt. Let me keep the implementation simpler:

```rust
async fn read_pipe<T>(pipe: Option<T>, max_bytes: usize) -> String
where
    T: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    if let Some(mut pipe) = pipe {
        let mut buf = Vec::new();
        if pipe.read_to_end(&mut buf).await.is_err() {
            return String::new();
        }
        if buf.len() > max_bytes {
            buf.truncate(max_bytes);
            let mut s = String::from_utf8_lossy(&buf).to_string();
            s.push_str("\n[truncated]");
            return s;
        }
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}
```

- [ ] **Step 3: Add unit tests for subprocess runner**

In `src/bridge/subprocess.rs`, add tests at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subprocess_echo_stdout() {
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "echo hello".to_string()]
        } else {
            vec!["-c".to_string(), "echo hello".to_string()]
        };
        let result = run_subprocess(
            cmd, &args, None, None, &HashMap::new(), 30, 100_000,
        )
        .await
        .unwrap();
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.exit_code, 0);
        assert!(!result.timed_out);
    }

    #[tokio::test]
    async fn test_subprocess_non_zero_exit() {
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "exit 1".to_string()]
        } else {
            vec!["-c".to_string(), "exit 1".to_string()]
        };
        let result = run_subprocess(
            cmd, &args, None, None, &HashMap::new(), 30, 100_000,
        )
        .await
        .unwrap();
        assert_eq!(result.exit_code, 1);
    }

    #[tokio::test]
    async fn test_subprocess_command_not_found() {
        let result = run_subprocess(
            "definitely_not_a_real_command_12345",
            &[],
            None,
            None,
            &HashMap::new(),
            30,
            100_000,
        )
        .await;
        assert!(matches!(result, Err(SubprocessError::CommandNotFound(_))));
    }

    #[tokio::test]
    async fn test_subprocess_redacts_secrets() {
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "echo secret is sk-abc123".to_string()]
        } else {
            vec!["-c".to_string(), "echo secret is sk-abc123".to_string()]
        };
        let result = run_subprocess(
            cmd, &args, None, None, &HashMap::new(), 30, 100_000,
        )
        .await
        .unwrap();
        assert!(!result.stdout.contains("sk-abc123"));
        assert!(result.stdout.contains("[REDACTED]"));
    }

    #[tokio::test]
    async fn test_subprocess_stdin_input() {
        let cmd = if cfg!(windows) { "cmd" } else { "cat" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "type con".to_string()]
        } else {
            vec![]
        };
        let input = if cfg!(windows) {
            // On Windows, type con reads from stdin but needs a newline
            "hello from stdin\n"
        } else {
            "hello from stdin"
        };
        let result = run_subprocess(
            cmd, &args, Some(input), None, &HashMap::new(), 30, 100_000,
        )
        .await
        .unwrap();
        // Note: Windows type con may not work reliably in tests; just verify it ran
        assert_eq!(result.exit_code, 0);
    }
}
```

Wait, Windows `type con` is unreliable in automated tests. Let's use a cross-platform approach. For stdin tests, we can skip the stdin test on Windows or use a different approach. Actually, the simplest cross-platform stdin test is to use `cat` on Unix and skip on Windows:

```rust
    #[tokio::test]
    async fn test_subprocess_stdin_input() {
        if cfg!(windows) {
            // Windows cmd /C type con is unreliable in tests; skip
            return;
        }
        let result = run_subprocess(
            "cat",
            &[],
            Some("hello from stdin"),
            None,
            &HashMap::new(),
            30,
            100_000,
        )
        .await
        .unwrap();
        assert!(result.stdout.contains("hello from stdin"));
        assert_eq!(result.exit_code, 0);
    }
```

- [ ] **Step 4: Register bridge module in lib.rs**

Modify `src/lib.rs`:

```rust
pub mod approvals;
pub mod bridge;  // ADD THIS LINE
pub mod cli;
// ... rest unchanged
```

- [ ] **Step 5: Run quality gates**

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build
```

Expected: all pass. Subprocess tests should run.

- [ ] **Step 6: Commit**

```bash
git add src/bridge/ src/lib.rs
git commit -m "feat: add subprocess bridge runner"
```

---

## Task 2: Bridge Config Schema

**Files:**
- Modify: `src/config/schema.rs`
- Modify: `src/config/validation.rs`

- [ ] **Step 1: Extend CursorModelConfig with bridge fields**

In `src/config/schema.rs`, add to `CursorModelConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cursor_composer_model")]
    pub composer_model_id: String,
    #[serde(default = "default_cursor_premium_model")]
    pub premium_model_id: String,
    #[serde(default = "default_cursor_provider")]
    pub provider: String,
    #[serde(default = "default_cursor_runtime")]
    pub default_runtime: String,
    #[serde(default = "default_false")]
    pub cloud_auto_create_pr: bool,
    // Bridge fields (all optional with defaults)
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_stdin_true")]
    pub stdin: bool,
    #[serde(default = "default_timeout_900")]
    pub timeout_seconds: u64,
    #[serde(default = "default_approval_true")]
    pub requires_approval: bool,
    pub model: Option<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default = "default_max_output_100000")]
    pub max_output_bytes: usize,
}
```

Add default functions:

```rust
fn default_timeout_900() -> u64 {
    900
}

fn default_stdin_true() -> bool {
    true
}

fn default_approval_true() -> bool {
    true
}

fn default_max_output_100000() -> usize {
    100_000
}
```

- [ ] **Step 2: Add bridge config fields to ModelsConfig**

In `src/config/schema.rs`, modify `ModelsConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModelsConfig {
    #[serde(default)]
    pub ollama: OllamaModelConfig,
    #[serde(default)]
    pub claude: ClaudeModelConfig,
    #[serde(default)]
    pub openai: OpenAiModelConfig,
    #[serde(default)]
    pub codex: CodexModelConfig,
    #[serde(default)]
    pub cursor: CursorModelConfig,
    #[serde(default)]
    pub claude_code: ExternalBridgeModelConfig,
    #[serde(default)]
    pub codex_bridge: ExternalBridgeModelConfig,
}
```

Note: We use `codex_bridge` to avoid name collision with the existing `codex` field which has a different type (`CodexModelConfig`). The provider key for routing is still "codex".

Actually, re-reading the spec and config: the existing `codex` field in `ModelsConfig` is of type `CodexModelConfig`. The user wants bridge config under named model keys. Since `CodexModelConfig` already exists, we have two options:

1. Replace `CodexModelConfig` with `ExternalBridgeModelConfig` — breaks backwards compat
2. Add new fields `claude_code: ExternalBridgeModelConfig` and keep existing `codex: CodexModelConfig`

Wait, the user said "One generic ExternalBridgeModelConfig reused under models" and "No top-level bridges section." But the existing config already has `models.codex` as a `CodexModelConfig`. We need to add NEW keys for the bridge providers. The existing `codex` key has its own config type.

Let me re-read: "A reusable ExternalBridgeModelConfig. Named model/provider entries for: claude_code, codex, cursor. All three should reuse the same bridge config shape."

But `models.codex` already exists as a `CodexModelConfig`. So we can't reuse the same key without breaking backwards compat.

Options:
1. Add new fields: `claude_code: ExternalBridgeModelConfig`, `codex_bridge: ExternalBridgeModelConfig`
2. Convert existing `CodexModelConfig` to include bridge fields

Given backwards compatibility is critical, I think the cleanest approach is to add bridge fields to the EXISTING config structs where possible. For `CursorModelConfig`, we already extend it. For `CodexModelConfig`, we could extend it similarly. But the user said "use one generic bridge config unless a provider truly needs unique fields."

Actually, looking again at the existing config: `CodexModelConfig` has `enabled`, `provider`, `default_model`, `role`. The `ExternalBridgeModelConfig` has `enabled`, `command`, `args`, `stdin`, `timeout_seconds`, `requires_approval`, `model`, `working_directory`, `env`, `max_output_bytes`.

These are quite different. The existing `CodexModelConfig` is meant for HTTP API config (like ClaudeModelConfig and OpenAiModelConfig). The bridge config is for subprocess execution.

I think the cleanest approach is:
1. Keep existing `codex: CodexModelConfig` for backwards compat
2. Add NEW fields: `claude_code: ExternalBridgeModelConfig` and `codex_bridge: ExternalBridgeModelConfig`
3. For cursor, extend the existing `CursorModelConfig` with bridge fields

But the user said "All three should reuse the same bridge config shape." This means claude_code, codex, and cursor should all use `ExternalBridgeModelConfig`.

For cursor, we already have `CursorModelConfig`. We need to either:
1. Replace it (breaks backwards compat)
2. Add bridge fields to it (which is what we did above)
3. Add a separate `cursor_bridge: ExternalBridgeModelConfig`

The user said: "If existing cursor config already exists under models.cursor, extend it carefully or migrate it in a backwards-compatible way. Do not break old cursor config parsing."

So for cursor, we extend `CursorModelConfig` with bridge fields. For claude_code, we add a NEW field `claude_code: ExternalBridgeModelConfig`. For codex, we add a NEW field `codex_bridge: ExternalBridgeModelConfig`.

But wait, the user also said: "Provider name derived from the models key. Example: models.cursor produces provider = cursor."

So the routing uses the KEY name, not the type. The factory uses the KEY name to look up the right config.

Let me revise: I'll add `claude_code: ExternalBridgeModelConfig` and `codex_bridge: ExternalBridgeModelConfig` as new fields in `ModelsConfig`. The factory will check these keys. For cursor, the factory checks `models.cursor.command`.

Actually, rethinking: the factory receives a `ProviderKind`, not a config key name. The factory maps `ProviderKind::Cursor` to `models.cursor`. So for cursor, we check `models.cursor.enabled` and `models.cursor.command`.

For ClaudeCode and Codex, we add new `ProviderKind` variants, and the factory maps them to `models.claude_code` and `models.codex_bridge`.

Wait, but the user said "Provider name derived from the models key." So the provider name string is derived from the YAML key. But in Rust, the factory uses `ProviderKind` enum.

Let me look at how `provider_from_name` works:

```rust
fn provider_from_name(name: &str) -> ProviderKind {
    match name {
        "ollama" => ProviderKind::Ollama,
        "claude" | "anthropic" => ProviderKind::Anthropic,
        "codex" | "openai" => ProviderKind::OpenAi,
        "cursor" | "cursor_composer" => ProviderKind::Cursor,
        _ => ProviderKind::Manual,
    }
}
```

So "codex" maps to `ProviderKind::OpenAi`. That's existing behavior. The existing `models.codex` config is used for OpenAI-compatible Codex API.

For the bridge, we need new provider names: "claude_code", "codex_bridge" (or just keep it as a separate concept). Actually, looking at the user's spec again, they said the provider name is derived from the models key. For claude_code, the key is `claude_code`. For codex, hmm, the user said `models.codex` in the YAML example but that conflicts with existing `codex`.

I think the cleanest approach is:
1. Add `claude_code: ExternalBridgeModelConfig` to `ModelsConfig` (new field, new provider)
2. For codex bridge, we can either:
   a. Add `codex_bridge: ExternalBridgeModelConfig` (new field)
   b. Extend `CodexModelConfig` with bridge fields (but then it's not generic)
3. For cursor, extend `CursorModelConfig` with bridge fields

Actually, the user explicitly said: "One generic ExternalBridgeModelConfig reused under models." This means ALL THREE should use the same type. So we need to either:
1. Replace existing config types (breaks backwards compat — user said don't break)
2. Add NEW fields with different names

I'll go with option 2: add `claude_code: ExternalBridgeModelConfig` and `codex_bridge: ExternalBridgeModelConfig` as NEW fields. Cursor gets bridge fields added to existing `CursorModelConfig`.

But wait, the user said: "If existing cursor config already exists under models.cursor, extend it carefully or migrate it in a backwards-compatible way."

So cursor gets extended. Claude Code and Codex get new fields.

For Codex, should the new field be called `codex` (overwriting the existing type) or `codex_bridge`? The user showed this YAML:

```yaml
models:
  codex:
    enabled: false
    kind: external_bridge
    command: codex
    args: []
    stdin: true
    timeout_seconds: 600
    requires_approval: true
    model: codex
```

This uses `models.codex` key. But `models.codex` already exists as `CodexModelConfig`. If we change the type to `ExternalBridgeModelConfig`, old configs break.

Unless... we make `ModelsConfig` NOT use `deny_unknown_fields` and use a custom deserializer. But that complicates things.

Actually, looking at the existing `CodexModelConfig`:

```rust
pub struct CodexModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_codex_provider")]
    pub provider: String,
    #[serde(default = "default_codex_model")]
    pub default_model: String,
    #[serde(default = "default_codex_role")]
    pub role: String,
}
```

We could add bridge fields to `CodexModelConfig` just like we do for `CursorModelConfig`:

```rust
pub struct CodexModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_codex_provider")]
    pub provider: String,
    #[serde(default = "default_codex_model")]
    pub default_model: String,
    #[serde(default = "default_codex_role")]
    pub role: String,
    // Bridge fields
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_stdin_true")]
    pub stdin: bool,
    // ... etc
}
```

This keeps backwards compatibility. When `command` is present, the factory creates a `BridgeProvider` instead of an HTTP provider. When `command` is absent, it falls back to the existing HTTP skeleton.

But the user said "One generic ExternalBridgeModelConfig." If we add bridge fields to each existing model config, we end up with duplicated fields across multiple structs.

Hmm, let me re-read: "Use one generic bridge config unless a provider truly needs unique fields."

I think the practical approach is:
1. Add bridge fields to `CursorModelConfig` (user explicitly said to extend it)
2. For `claude_code`, add a new field `claude_code: ExternalBridgeModelConfig` to `ModelsConfig`
3. For `codex`, similarly add bridge fields or create a new field

Actually, I think the simplest clean approach that satisfies "one generic config" AND backwards compatibility:

**Add a `bridges: HashMap<String, ExternalBridgeModelConfig>` field to `ModelsConfig`.**

Wait, the user said "No top-level bridges section in this phase" for YAML. But a `bridges` field INSIDE `ModelsConfig` is different from a top-level `bridges:` section.

Actually, re-reading: "Do not create a top-level bridges section in this phase." So a `bridges` map inside `models` is OK.

But then the YAML would look like:
```yaml
models:
  bridges:
    claude_code:
      enabled: true
      command: claude
    codex:
      enabled: false
      command: codex
    cursor:
      enabled: false
      command: node
```

This doesn't match what the user showed. The user showed:
```yaml
models:
  claude_code:
    enabled: true
    command: claude
  codex:
    enabled: false
    command: codex
  cursor:
    enabled: false
    command: node
```

OK so the user wants them as direct children of `models`. Let me think about how to achieve this with serde.

With serde, we can't have two fields with the same name but different types. So `models.codex` can only be ONE type. Since `CodexModelConfig` already exists, we have to extend it.

I think the right approach is:
1. **CursorModelConfig** — extend with bridge fields (user explicitly said to)
2. **Add `claude_code: ExternalBridgeModelConfig`** to `ModelsConfig` (new field)
3. **Extend `CodexModelConfig`** with bridge fields (or add `codex_bridge: ExternalBridgeModelConfig`)

For Codex, I'll extend `CodexModelConfig` with bridge fields, same pattern as CursorModelConfig. This keeps backwards compat and allows `models.codex` to work as either HTTP or bridge.

Wait, but the user said "One generic ExternalBridgeModelConfig reused under models." If I add the same fields to multiple structs, that's not "one generic config".

I think what the user means is: the CONCEPTUAL config shape is the same. Whether we achieve that through composition (embedding a generic struct) or through duplication is an implementation detail.

Let me use composition: embed `ExternalBridgeModelConfig` into each model config that supports bridging. Or rather, just add the bridge fields directly to each model config that needs them.

Actually, the simplest thing that works: since only Cursor, Claude Code, and Codex need bridges, and cursor already has its own config struct, let me:

1. Add `claude_code: ExternalBridgeModelConfig` to `ModelsConfig`
2. For `codex`, extend `CodexModelConfig` with the same bridge fields (to keep `models.codex` key)
3. For `cursor`, extend `CursorModelConfig` with bridge fields

This is pragmatic. The `ExternalBridgeModelConfig` is used for `claude_code` and the fields are duplicated (or composed) into `CodexModelConfig` and `CursorModelConfig`.

Actually wait, I just realized: we can use serde's `flatten` feature to embed a generic struct!

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_codex_provider")]
    pub provider: String,
    #[serde(default = "default_codex_model")]
    pub default_model: String,
    #[serde(default = "default_codex_role")]
    pub role: String,
    #[serde(flatten)]
    pub bridge: ExternalBridgeModelConfig,
}
```

This would put bridge fields at the same level as existing fields in YAML:
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
```

But `ExternalBridgeModelConfig` has `enabled: bool` and `model: Option<String>`, which would conflict with `CodexModelConfig`'s `enabled: bool` and `default_model: String`. The `enabled` field would be ambiguous.

So flatten won't work cleanly due to field name collisions.

OK, I'll take the simplest approach that satisfies all constraints:

1. Add `claude_code: ExternalBridgeModelConfig` as a new field in `ModelsConfig`
2. For `cursor`, extend `CursorModelConfig` with bridge fields (user explicitly said to)
3. For `codex`, extend `CodexModelConfig` with bridge fields

For the "one generic config" requirement, I'll document that the SHAPE is the same, even if embedded differently due to backwards compatibility.

Actually, you know what, let me re-read the user's message more carefully:

"Use one generic bridge config unless a provider truly needs unique fields."

I think they want us to literally use the same struct type for all three. But we can't because of backwards compatibility with existing config keys.

The pragmatic solution: for NEW providers (claude_code), use `ExternalBridgeModelConfig`. For EXISTING providers (cursor, codex), extend their existing config structs with the same fields. The fields have the same names and defaults, so the YAML looks the same.

Let me proceed with this approach in the implementation plan.

Actually, hold on. I just realized there's another option: we can make the bridge config a SEPARATE section but NOT top-level. Like:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    // ... existing fields
    #[serde(default)]
    pub bridge: BridgeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BridgeConfig {
    #[serde(default)]
    pub claude_code: ExternalBridgeModelConfig,
    #[serde(default)]
    pub codex: ExternalBridgeModelConfig,
    #[serde(default)]
    pub cursor: ExternalBridgeModelConfig,
}
```

YAML:
```yaml
models:
  bridge:
    claude_code:
      enabled: true
      command: claude
    codex:
      enabled: false
      command: codex
    cursor:
      enabled: false
      command: node
```

This gives us "one generic config" but it's under `models.bridge` not `models.codex`. The user showed `models.codex` in their example. So this doesn't match what the user wants.

OK, I think the user's example was conceptual. They said "Suggested conceptual YAML shape, but adapt to existing config style." So we should adapt.

I'll go with: extend existing structs where needed, add new fields where there's no conflict.

Final plan:
1. `claude_code`: Add NEW field `claude_code: ExternalBridgeModelConfig` to `ModelsConfig`
2. `cursor`: Extend `CursorModelConfig` with bridge fields
3. `codex`: Extend `CodexModelConfig` with bridge fields

For the factory, `ProviderKind::ClaudeCode` maps to `models.claude_code`. `ProviderKind::Codex` maps to `models.codex` (checking `command` presence for bridge mode). `ProviderKind::Cursor` maps to `models.cursor` (checking `command` presence).

But wait, existing `ProviderKind::Codex` doesn't exist yet. The existing `provider_from_name` maps "codex" to `ProviderKind::OpenAi`. If we add `ProviderKind::Codex`, we need to update `provider_from_name` which might break existing routing.

Looking at routing: `default_scoped_executor` is "codex". `provider_from_name("codex")` returns `ProviderKind::OpenAi`. `model_for_provider(ProviderKind::OpenAi, models)` returns `models.openai.default_model`.

If we change `provider_from_name("codex")` to return `ProviderKind::Codex`, then `model_for_provider` would need to handle `ProviderKind::Codex`.

But the existing routing uses "codex" as the default scoped executor, which maps to OpenAI. If we change that, existing behavior breaks.

I think the safest approach:
1. Keep "codex" mapping to `ProviderKind::OpenAi` for backwards compat
2. Add a NEW provider name "codex_bridge" that maps to `ProviderKind::Codex`
3. For Claude Code, "claude_code" maps to `ProviderKind::ClaudeCode`
4. For cursor, keep "cursor" → `ProviderKind::Cursor`

Then in routing, the default_repo_executor is "cursor_composer" → `ProviderKind::Cursor`. The default_scoped_executor is "codex" → `ProviderKind::OpenAi`.

For bridge routing, we add new logic that checks if a bridge provider is enabled and selects it based on task type.

Hmm, this is getting complex. Let me simplify.

The user's spec says the routing hierarchy is:
1. Ollama
2. Cursor Composer
3. Codex
4. Claude Code
5. Fallback

But the existing routing already routes to "cursor_composer" (maps to Cursor) and "codex" (maps to OpenAi). The question is: should we REPLACE the existing routing to use bridge providers, or ADD bridge providers as ALTERNATIVES?

The user says: "These providers should integrate with the existing provider trait and result artifact system similarly to the existing Ollama provider."

I think the intent is: when enabled, the bridge provider takes the place of the existing provider. When disabled, existing behavior continues.

So for cursor: when `models.cursor.enabled` AND `models.cursor.command` is present, use bridge mode. Otherwise use existing placeholder.

For codex: the existing `models.codex` has `enabled`, `provider`, `default_model`, `role`. It doesn't have `command`. We add `command` field. When present and enabled, use bridge mode.

For claude_code: this is a NEW provider, so we add `models.claude_code` field.

For the routing: we need to decide whether "codex" routes to `ProviderKind::OpenAi` or `ProviderKind::Codex`. I think the cleanest approach is:

1. The factory determines the actual provider based on config
2. Routing can select `ProviderKind::OpenAi` for HTTP codex or `ProviderKind::Codex` for bridge codex
3. But this means routing needs to know about bridge mode

Actually, I think the simplest approach is:
1. Routing selects `ProviderKind::Cursor`, `ProviderKind::OpenAi`, `ProviderKind::Anthropic` as it does today
2. The FACTORY decides whether to create a bridge provider or HTTP provider based on config
3. For NEW providers (claude_code), routing explicitly selects `ProviderKind::ClaudeCode`

Wait, but the routing uses `provider_from_name(config.routing.default_repo_executor)` which returns `ProviderKind::Cursor` for "cursor_composer". And `model_for_provider(ProviderKind::Cursor, models)` returns `models.cursor.composer_model_id`.

If cursor is in bridge mode, the model comes from `models.cursor.model` (or fallback to `composer_model_id`). The factory creates a `BridgeProvider` with cursor config.

I think this works! The routing stays the same (selects ProviderKind), but the FACTORY interprets the config to create the right provider implementation.

For codex: routing selects `ProviderKind::OpenAi` (because `provider_from_name("codex")` returns `ProviderKind::OpenAi`). The factory sees `ProviderKind::OpenAi` and creates `OpenAiProvider`. But we want it to create a bridge provider when `models.codex.command` is present.

Hmm, the factory only receives `ProviderKind`, not the config key name. So `ProviderKind::OpenAi` could mean either the OpenAI HTTP provider or the Codex bridge provider. We need a way to distinguish.

Options:
1. Add `ProviderKind::Codex` and change `provider_from_name("codex")` to return it. This would break existing routing that expects OpenAi for codex.
2. Keep `ProviderKind::OpenAi` for codex, but have the factory check `models.codex.command` when creating OpenAi provider.
3. Add a new provider kind `CodexBridge` and have routing explicitly select it when bridge mode is enabled.

Option 2 seems safest for backwards compatibility. The factory for `ProviderKind::OpenAi` checks: if `models.codex.command` is present, create a bridge provider instead of OpenAiProvider.

But that's hacky — the OpenAi factory arm now creates a Codex bridge provider.

Option 3 is cleaner: add `ProviderKind::CodexBridge` (or just use the existing `ProviderKind::Codex` if we add it). Routing explicitly selects it when the config says to use bridge codex.

Wait, I just realized: the routing config already has `default_repo_executor` and `default_scoped_executor`. If the user sets `default_scoped_executor: codex_bridge`, then `provider_from_name("codex_bridge")` could return a new `ProviderKind::Codex`.

But then existing tests that use `default_scoped_executor: codex` would break if we change `provider_from_name("codex")`.

OK, I think the simplest approach is:
1. Keep `provider_from_name("codex")` returning `ProviderKind::OpenAi`
2. Add `provider_from_name("codex_bridge")` returning `ProviderKind::Codex` (new variant)
3. For cursor, keep `provider_from_name("cursor")` returning `ProviderKind::Cursor`
4. Add `provider_from_name("claude_code")` returning `ProviderKind::ClaudeCode`

For routing, we add NEW decisions that explicitly check for bridge providers. For example, `implementation_decision` currently uses `default_repo_executor` which is "cursor_composer". We can modify it to check if cursor bridge is enabled, and if so, create a cursor bridge provider. The routing still selects `ProviderKind::Cursor`, but the factory checks config to decide bridge vs placeholder.

Actually, I think the cleanest factory approach:

```rust
pub fn create_provider(kind: ProviderKind, config: &Config) -> Box<dyn Provider> {
    match kind {
        ProviderKind::OpenAi => {
            // Check if codex bridge mode is configured
            if config.models.codex.command.is_some() && config.models.codex.enabled {
                Box::new(BridgeProvider::from_config("codex", &config.models.codex))
            } else {
                Box::new(OpenAiProvider::from_config(&config.models.openai))
            }
        }
        ProviderKind::Ollama => { ... }
        ProviderKind::Cursor => {
            if config.models.cursor.command.is_some() && config.models.cursor.enabled {
                Box::new(BridgeProvider::from_config("cursor", &config.models.cursor))
            } else {
                Box::new(CursorProvider::new(...))
            }
        }
        ProviderKind::ClaudeCode => {
            if config.models.claude_code.enabled {
                Box::new(BridgeProvider::from_config("claude_code", &config.models.claude_code))
            } else {
                Box::new(MockProvider::default())
            }
        }
        ProviderKind::Codex => {
            // This is the NEW variant for explicit codex bridge selection
            if config.models.codex.command.is_some() && config.models.codex.enabled {
                Box::new(BridgeProvider::from_config("codex", &config.models.codex))
            } else {
                Box::new(MockProvider::default())
            }
        }
        _ => Box::new(MockProvider::default()),
    }
}
```

This way:
- Existing routing that selects `ProviderKind::OpenAi` for "codex" will check bridge mode first
- New routing can explicitly select `ProviderKind::Codex` for bridge codex
- `ProviderKind::Cursor` checks bridge mode for cursor
- `ProviderKind::ClaudeCode` is always bridge (no existing HTTP provider)

This maintains backwards compatibility while supporting bridges.

But wait, there's a subtle issue: if `models.codex.command` is present and `models.codex.enabled` is true, then ALL `ProviderKind::OpenAi` selections will use the codex bridge instead of the OpenAi HTTP provider. That might not be desired if the user wants both.

I think that's actually OK for Phase 3. If the user configures a codex bridge, they probably want to use it instead of the generic OpenAI provider. In the future, we can add more explicit routing.

Alternatively, we can be more conservative: only create a bridge provider when the routing EXPLICITLY selects the bridge provider kind. For `ProviderKind::OpenAi`, always use the HTTP provider.

But then how do we route to codex bridge? We'd need a new routing path that selects `ProviderKind::Codex`.

I think the user wants us to be conservative and safe. Let me re-read:

"If a tool's command-line interface is uncertain, design the bridge config to be command-template driven and configurable, with safe defaults disabled or marked experimental."

OK so the bridges are disabled by default. When enabled, the user explicitly wants to use them.

Let me just keep it simple:
1. Add `ProviderKind::ClaudeCode` and `ProviderKind::Codex`
2. Factory creates bridge providers for these kinds
3. For cursor, keep `ProviderKind::Cursor` and have factory check `command` presence
4. For codex, the NEW `ProviderKind::Codex` factory arm creates bridge provider
5. Keep `ProviderKind::OpenAi` as HTTP only
6. Routing adds explicit checks for bridge providers when enabled

This is the cleanest. Existing "codex" → `ProviderKind::OpenAi` behavior is preserved. New bridge codex uses `ProviderKind::Codex`.

For routing, we need to add logic that selects bridge providers. For example, in `implementation_decision`, after the existing logic, we can add:

```rust
// If codex bridge is enabled and task matches codex criteria, route to it
if provider_enabled(ProviderKind::Codex, models) {
    // Check if this is a good codex task
    return RoutingDecision::simple(
        ProviderKind::Codex,
        models.codex.model.clone().unwrap_or_else(|| "codex".to_string()),
        "Scoped implementation routed to Codex bridge.",
    );
}
```

But this changes routing behavior. The user said "Preserve existing Ollama routing for cheap/low-risk planning. Do not break existing routing tests."

I think the right approach is to add bridge providers as HIGHER-PRIORITY options in routing. For example:

```rust
fn implementation_decision(task: &Task, models: &ModelsConfig, config: &Config) -> RoutingDecision {
    // NEW: Check for bridge providers first
    if provider_enabled(ProviderKind::Cursor, models) && models.cursor.command.is_some() {
        // Route to cursor bridge for repo-aware tasks
        if task.requires_repo_search || task.estimated_files_touched >= 3 {
            return RoutingDecision::simple(
                ProviderKind::Cursor,
                models.cursor.model.clone().unwrap_or_else(|| models.cursor.composer_model_id.clone()),
                "Repo-aware implementation routed to Cursor Composer bridge.",
            );
        }
    }
    
    if provider_enabled(ProviderKind::Codex, models) && models.codex.command.is_some() {
        // Route to codex bridge for scoped implementation
        if task.context_is_exact && task.estimated_files_touched < 3 {
            return RoutingDecision::simple(
                ProviderKind::Codex,
                models.codex.model.clone().unwrap_or_else(|| "codex".to_string()),
                "Scoped implementation routed to Codex bridge.",
            );
        }
    }
    
    // Existing logic
    ...
}
```

But this is getting complicated and might break existing tests. Let me think of a simpler approach.

Actually, I think the safest routing approach is:
1. Keep ALL existing routing logic unchanged
2. Add NEW routing decisions ONLY for explicitly bridge-oriented tasks
3. For the default routing, keep it as-is but add bridge providers as alternative options when the default provider is disabled

For example, in `implementation_decision`, the existing logic routes to `default_repo_executor` ("cursor_composer") or `default_scoped_executor` ("codex"). These map to `ProviderKind::Cursor` and `ProviderKind::OpenAi`.

If we keep this, then:
- `ProviderKind::Cursor` factory checks if cursor bridge is enabled. If so, creates bridge. Otherwise, creates placeholder.
- `ProviderKind::OpenAi` factory checks if codex bridge is enabled. If so, creates codex bridge. Otherwise, creates OpenAi HTTP provider.
- `ProviderKind::ClaudeCode` factory always creates claude_code bridge.

This means we don't need to change routing at all! The factory just interprets the config differently.

For Claude Code, we need NEW routing that explicitly selects it. We can add a `planning_decision` branch that selects `ProviderKind::ClaudeCode` for complex planning when enabled.

For the user's spec, this is what they want:
"Routing: Add routing support so tasks can route to claude_code when: task complexity is high, risk is medium/high, task type is planning/review/design, config enables the provider"

So we DO need to add new routing logic for Claude Code. And possibly for Codex and Cursor if we want them to be selected based on criteria.

But for Cursor and Codex, the existing routing already routes to them via `default_repo_executor` and `default_scoped_executor`. The only thing we need to do is make the factory create bridge providers when configured.

For Claude Code, we need NEW routing. For Codex, we might need NEW routing if we want to distinguish from OpenAI.

Let me simplify:

**Cursor:** Existing routing already routes to `ProviderKind::Cursor` via `default_repo_executor`. Factory checks `cursor.command` and creates bridge when present. No routing changes needed.

**Codex:** Existing routing routes to `ProviderKind::OpenAi` via `default_scoped_executor`. We need to decide: should codex bridge replace OpenAi HTTP when configured? Or should we add a separate route?

Given backwards compatibility, I think: if `models.codex.command` is present and enabled, the factory for `ProviderKind::OpenAi` (when the routing reason is "codex") creates a codex bridge. But the factory doesn't know the routing reason.

Alternative: change `provider_from_name` so "codex" returns `ProviderKind::Codex` (new variant) instead of `ProviderKind::OpenAi`. But this breaks existing tests.

OK, I think the simplest backwards-compatible approach:

1. Keep "codex" → `ProviderKind::OpenAi` in `provider_from_name`
2. For the factory, when `ProviderKind::OpenAi` is requested, check if `models.codex.command` is present. If so AND the task's default executor is "codex" (which we can't easily know in the factory), create a bridge.

This is messy. Let me just accept that we need to add `ProviderKind::Codex` and update `provider_from_name`.

Looking at the existing tests:
- `test_route_scoped_exact_to_codex` expects `ProviderKind::OpenAi` when "codex" is the scoped executor
- If we change `provider_from_name("codex")` to return `ProviderKind::Codex`, this test would fail

But we can update the test! The user said "Do not break existing routing tests." Hmm, actually the user said to preserve existing tests. But if the test is checking that "codex" maps to OpenAi, and we want it to map to Codex, we'd need to change the test.

Wait, maybe we can keep "codex" mapping to OpenAi for backwards compatibility, but add a NEW alias "codex_bridge" that maps to `ProviderKind::Codex`. Then the routing config can use `default_scoped_executor: codex_bridge`.

But the default config has `default_scoped_executor: "codex"`. If users want the bridge, they change it to `codex_bridge`.

This is actually clean! Let me use this approach:

1. `provider_from_name("codex")` → `ProviderKind::OpenAi` (unchanged)
2. `provider_from_name("codex_bridge")` → `ProviderKind::Codex` (NEW)
3. `provider_from_name("claude_code")` → `ProviderKind::ClaudeCode` (NEW)
4. `provider_from_name("cursor")` → `ProviderKind::Cursor` (unchanged)
5. `provider_from_name("cursor_composer")` → `ProviderKind::Cursor` (unchanged)

Then in routing, we keep "codex" as the default scoped executor. Users who want the bridge change it to "codex_bridge" in their config.

But the user's spec says: "Routing: Add routing support so tasks can route to codex when: task type is implementation, complexity is medium/high, acceptance criteria are explicit, risk is not extreme, config enables the provider"

This suggests the routing should AUTOMATICALLY select codex when appropriate, not just when the user changes config.

I think for Phase 3, the pragmatic approach is:
1. Keep all existing routing as-is
2. Add NEW routing branches that check if bridge providers are enabled
3. These new branches are checked AFTER existing logic but can override if criteria match

For example:

```rust
fn implementation_decision(...) {
    // Existing logic first
    let repo_aware = ...;
    let config_name = ...;
    let kind = provider_from_name(config_name);
    
    // NEW: If default executor is cursor_composer and cursor bridge is enabled, use it
    if kind == ProviderKind::Cursor {
        if provider_enabled(ProviderKind::Cursor, models) {
            // Use cursor, whether bridge or placeholder
            return RoutingDecision { provider: ProviderKind::Cursor, ... };
        }
    }
    
    // If no provider available, try bridge providers
    if kind == ProviderKind::Manual {
        if provider_enabled(ProviderKind::Cursor, models) {
            return RoutingDecision::simple(ProviderKind::Cursor, ..., "Fallback to Cursor bridge.");
        }
        if provider_enabled(ProviderKind::Codex, models) {
            return RoutingDecision::simple(ProviderKind::Codex, ..., "Fallback to Codex bridge.");
        }
    }
    
    // Existing return
    return RoutingDecision { ... };
}
```

This keeps existing behavior and only adds bridge providers as fallbacks or when explicitly configured.

OK, I realize I've been overthinking this. Let me just write the plan with a pragmatic approach:

1. For cursor: existing routing already routes to `ProviderKind::Cursor`. Factory checks `cursor.command` and creates bridge when present. No routing changes.
2. For codex: add `ProviderKind::Codex`. Update `provider_from_name` to map "codex" → `ProviderKind::Codex`. Update `model_for_provider` and `provider_enabled`. Update tests. This is a breaking change but it's what the user wants.
3. For claude_code: add `ProviderKind::ClaudeCode`. Add routing that selects it for complex planning when enabled.

Actually, I just re-read the user's constraint: "Do not break existing routing tests." So I CANNOT change `provider_from_name("codex")` to return `ProviderKind::Codex` because that would break `test_route_scoped_exact_to_codex`.

OK, final approach:
1. Keep `provider_from_name("codex")` → `ProviderKind::OpenAi` (unchanged)
2. Add `provider_from_name("codex_bridge")` → `ProviderKind::Codex` (NEW)
3. Add `provider_from_name("claude_code")` → `ProviderKind::ClaudeCode` (NEW)
4. In routing, when selecting the default scoped executor, if it's "codex", we check if codex bridge is enabled. If so, route to `ProviderKind::Codex`. Otherwise, route to `ProviderKind::OpenAi`.

Actually, this is getting way too complex for a plan. Let me just simplify and write a clean plan:

**For the factory:**
- `ProviderKind::Cursor`: if `cursor.command` present and enabled → BridgeProvider. Otherwise → existing CursorProvider placeholder.
- `ProviderKind::OpenAi`: always → OpenAiProvider (HTTP). Codex bridge is separate.
- `ProviderKind::Codex` (NEW): if `codex.command` present and enabled → BridgeProvider. Otherwise → MockProvider.
- `ProviderKind::ClaudeCode` (NEW): if `claude_code.enabled` → BridgeProvider. Otherwise → MockProvider.

**For routing:**
- Keep all existing routing logic
- Add NEW explicit routing for Claude Code in planning decisions
- For Codex, we add a new routing path that explicitly selects `ProviderKind::Codex` when the task matches codex criteria and the bridge is enabled
- For Cursor, existing routing already works

This is clean and backwards-compatible. Let me write the plan now.

OK, one more thing: I need to check if `deny_unknown_fields` on `ModelsConfig` will break when we add new fields. `deny_unknown_fields` means serde will reject unknown fields. But adding new fields WITH `serde(default)` should be fine because they're known fields, not unknown.

However, `deny_unknown_fields` combined with `#[serde(default)]` on the struct means: if a field is missing in YAML, use default. If an extra field is present in YAML, error. So adding new fields is safe as long as we add them to the struct.

But wait, `ModelsConfig` has `#[serde(deny_unknown_fields)]`. If we add `claude_code: ExternalBridgeModelConfig` to `ModelsConfig`, old YAML configs that don't have `claude_code` will still parse fine because `#[serde(default)]` on the field means it defaults to `ExternalBridgeModelConfig::default()`.

But `ExternalBridgeModelConfig` doesn't have a `Default` impl. We need to derive it or implement it. The `enabled` field defaults to `false` via `#[serde(default)]`, but the struct itself needs a `Default` impl for `ModelsConfig`'s `#[serde(default)]` to work.

Wait, `ModelsConfig` uses `#[serde(default)]` on fields, not on the struct. Actually looking at the code:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModelsConfig {
    #[serde(default)]
    pub ollama: OllamaModelConfig,
    ...
}
```

The `#[serde(default)]` on each field means: if the field is missing in YAML, use `Default::default()` for that field's type. So `OllamaModelConfig::default()` is called. `ExternalBridgeModelConfig` needs to implement `Default`.

Let me make sure `ExternalBridgeModelConfig` derives `Default` or implements it.

Also, `deny_unknown_fields` on `ModelsConfig` means: if YAML has an extra field not in `ModelsConfig`, error. Adding new fields to `ModelsConfig` is fine. But adding bridge fields to `CursorModelConfig` (which also has `deny_unknown_fields`) means old YAML with `models.cursor` that only has `enabled`, `composer_model_id`, etc. will still parse fine because the new fields have `serde(default)`.

OK, I'm confident in the approach now. Let me write the plan.

One more consideration: the user said "For cursor, preserve all existing config fields." The existing `CursorModelConfig` has these fields:
- enabled
- composer_model_id
- premium_model_id
- provider
- default_runtime
- cloud_auto_create_pr

I'll add bridge fields to it. The `model` field in the bridge config serves as the bridge model. For cursor, the fallback is `composer_model_id` if `model` is absent.

For `CodexModelConfig`, existing fields:
- enabled
- provider
- default_model
- role

I'll add bridge fields to it too. The `model` field is the bridge model, fallback to `default_model`.

For `ClaudeCode`, this is a NEW field in `ModelsConfig`, so it's purely `ExternalBridgeModelConfig`.

OK, now let me write the actual plan document. I'll keep it focused and practical.

Actually, I just realized there's one more issue. The existing `provider_enabled` function:

```rust
fn provider_enabled(kind: ProviderKind, models: &ModelsConfig) -> bool {
    match kind {
        ProviderKind::Ollama => models.ollama.enabled,
        ProviderKind::Anthropic => models.claude.enabled,
        ProviderKind::OpenAi => models.openai.enabled,
        ProviderKind::Cursor => models.cursor.enabled,
        _ => true,
    }
}
```

For `ProviderKind::Codex` and `ProviderKind::ClaudeCode`, the default is `true` (because of `_ => true`). We need to add explicit arms for these.

For `model_for_provider`:
```rust
fn model_for_provider(kind: ProviderKind, models: &ModelsConfig) -> String {
    match kind {
        ProviderKind::Ollama => models.ollama.default_model.clone(),
        ProviderKind::Anthropic => models.claude.default_model.clone(),
        ProviderKind::OpenAi => models.openai.default_model.clone(),
        ProviderKind::Cursor => models.cursor.composer_model_id.clone(),
        ProviderKind::Manual => "manual".to_string(),
        _ => "mock".to_string(),
    }
}
```

For `ProviderKind::Codex` and `ProviderKind::ClaudeCode`, we need to add arms.

OK, I have enough context. Let me write the plan.

I realize the plan is going to be VERY LONG. The writing-plans skill says "Write comprehensive implementation plans." Let me be thorough but focused.

I'll organize it into 10 tasks matching the user's stories. Each task will have specific file changes and test code.

Let me write it now.