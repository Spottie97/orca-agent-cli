# CLAUDE.md — Orca Agent CLI

## Project
Orca Agent CLI is a Rust CLI tool that orchestrates AI coding tasks through project memory, task routing, context packets, and provider adapters.

## Language
Rust (2021 edition preferred, 2024 acceptable).

## Binary name
`orca`

## Crate name
`orca-agent-cli`

## Core crates
- clap (CLI parsing)
- tokio (async runtime)
- serde, serde_json, serde_yaml (config/schemas)
- reqwest with rustls-tls (HTTP provider calls)
- anyhow (app errors)
- thiserror (library errors)
- async-trait (async provider traits)
- tracing, tracing-subscriber (logging)
- ignore or walkdir (repo scanning)
- directories (config paths)
- uuid (IDs)
- schemars (JSON schema)
- toml (optional config support)
- git2 (only if shelling out to git is insufficient)

## Test crates
- assert_cmd
- predicates
- tempfile
- insta
- wiremock

## Quality commands
```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build
```

## Config
- `.orca/orca.config.yaml` (default)
- Environment variables for secrets (ANTHROPIC_API_KEY, OPENAI_API_KEY, CURSOR_API_KEY, etc.)
- CURSOR_COMPOSER_MODEL_ID and CURSOR_PREMIUM_MODEL_ID must be configurable, never hardcoded.

## Project memory paths
- `.orca/memory/`
- `.orca/graph/`
- `.orca/context-packets/`

## Non-goals
- No TypeScript/Node.js core
- No database in MVP
- No unsafe Rust
- No global mutable state
- No fully autonomous execution without approval gates

## Ralph workflow
Small, testable, committed iterations. One story per iteration. Update prd.json and progress.txt after each.
