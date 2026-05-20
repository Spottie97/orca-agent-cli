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

## Validation

Run `orca config validate` to check:

- Required fields are present
- Paths exist
- Model IDs are set for enabled providers
- Secrets are redacted from output
