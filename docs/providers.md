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
| Ollama | Ollama | Summarize, compress | Skeleton |
| Anthropic | Anthropic | Plan, stream, multi-file | Skeleton |
| OpenAI | OpenAi | Tool-use, multi-file | Skeleton |
| Cursor | Cursor | Repo-aware, multi-file, stream | Placeholder |
| Command | Command | Chat, completion | Subprocess placeholder |

## Skeletons

Skeleton providers implement the trait but return placeholder responses. They are ready for real HTTP integration:

- `reqwest::Client` field for HTTP calls
- `base_url` and `api_key` fields
- Clear capability definitions

## Cursor adapter

The Cursor adapter is intentionally a safe placeholder. Cursor integration is handled externally (see [Cursor integration](cursor-integration.md)) to avoid a Node.js dependency in the Rust core.

## Adding a provider

1. Create a struct in `src/providers/`
2. Implement the `Provider` trait
3. Add tests
4. Register in routing policy
