# Providers

Orca supports multiple AI providers through a unified async trait.

## Provider trait

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> ProviderKind;
    fn capabilities(&self) -> Vec<ProviderCapability>;
    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError>;
    fn estimate_cost(&self, request: &ProviderRequest) -> CostEstimate;
    fn supports(&self, capability: ProviderCapability) -> bool;
}
```

## Supported providers

| Provider | Kind | Capabilities | Status |
|----------|------|--------------|--------|
| Mock | Mock | All (deterministic) | Fully implemented |
| Ollama | Ollama | Summarize, compress | Fully implemented |
| Anthropic | Anthropic | Plan, stream, multi-file | Skeleton |
| OpenAI | OpenAi | Tool-use, multi-file | Fully implemented |
| Claude Code | ClaudeCode | Chat, completion, planning | Bridge (subprocess) |
| Codex | Codex | Chat, completion, planning | Bridge (subprocess) |
| Cursor | Cursor | Repo-aware, multi-file, stream | Bridge (subprocess) or placeholder |
| Command | Command | Chat, completion | Subprocess placeholder |

## Skeletons

Skeleton providers implement the trait but return placeholder responses. They are ready for real HTTP integration:

- `reqwest::Client` field for HTTP calls
- `base_url` and `api_key` fields
- Clear capability definitions

## Bridge providers (Phase 3)

Bridge providers execute external CLI tools as subprocesses. They are configured with `command`, `args`, `stdin`, `timeout_seconds`, and optional `working_directory` and `env`.

### Claude Code Bridge

Routes high-complexity planning tasks to the Claude Code CLI. Configured under `models.claude_code`.

### Codex Bridge

Routes scoped implementation tasks to the OpenAI Codex CLI. Configured under `models.codex`.

### Cursor Composer Bridge

When `models.cursor.command` is set, routes repo-aware tasks to the Cursor CLI. Falls back to the placeholder `CursorProvider` when the bridge is not configured.

### Bridge diagnostics

Every bridge execution includes diagnostics in the result artifact:

- `command` — the executed command
- `exit_code` — subprocess exit code
- `timed_out` — whether the subprocess timed out

See [Phase 3 Bridge Providers](phase3-bridges.md) for full configuration, routing, and approval documentation.

## Cursor adapter

The Cursor adapter is intentionally a safe placeholder when the bridge is not configured. Cursor integration can also be handled externally (see [Cursor integration](cursor-integration.md)) to avoid a Node.js dependency in the Rust core.

## Adding a provider

1. Create a struct in `src/providers/`
2. Implement the `Provider` trait
3. Add tests
4. Register in routing policy
