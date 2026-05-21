# Configuration

Orca reads configuration from `.orca/config.yaml` by default. You can override with `--config`.

## Schema

```yaml
project:
  name: my-project
  repo_path: .
  vault_path: null
  orca_dir: .orca

execution:
  dry_run: false
  max_iterations: 10
  require_approval: true

models:
  ollama:
    enabled: true
    base_url: "${OLLAMA_HOST:-http://localhost:11434}"
    model: llama3
    role: context_and_memory
  anthropic:
    enabled: true
    api_key: "${ANTHROPIC_API_KEY}"
    model: claude-sonnet-4-6
  openai:
    enabled: true
    api_key: "${OPENAI_API_KEY}"
    model: gpt-4o
  cursor:
    enabled: false
    api_key: "${CURSOR_API_KEY}"
    composer_model: "${CURSOR_COMPOSER_MODEL_ID}"
    premium_model: "${CURSOR_PREMIUM_MODEL_ID}"

  # Phase 3 bridge providers (optional)
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

  codex:
    enabled: true
    command: codex
    args: []
    stdin: true
    timeout_seconds: 600
    requires_approval: true
    model: null
    working_directory: null
    env: {}
    max_output_bytes: 100000

routing:
  default_provider: ollama
  budget_policy: conservative
  fallback_enabled: true

context:
  max_tokens: 3000
  include_tests: true

memory:
  vault_format: obsidian
  max_summary_words: 200
```

## Environment variable substitution

Values can reference environment variables:

- `${VAR}` — replaced with the value of `VAR`
- `${VAR:-default}` — replaced with `VAR` or `default` if `VAR` is unset

## Ollama Cloud example

To use Ollama Cloud instead of a local instance:

```yaml
models:
  ollama:
    enabled: true
    base_url: https://ollama.com
    default_model: kimi-k2.6
    api_key_env_var: OLLAMA_API_KEY
    role: context_and_memory
```

Set the environment variable before running Orca:

```bash
export OLLAMA_API_KEY="your-api-key"
```

If `api_key_env_var` is omitted or empty, no `Authorization` header is sent, and local Ollama at `http://localhost:11434` continues to work without authentication.

## Validation

Run `orca config validate` to check:

- Required fields are present
- Paths exist
- Model IDs are set for enabled providers
- Secrets are redacted from output
