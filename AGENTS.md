# AGENTS.md — Orca Agent CLI Development Agents

## Project
Orca Agent CLI — Rust CLI for AI coding orchestration.

## Active Agents

### Rust Core Agent
Responsible for all Rust code, tests, and Cargo.toml changes.
- Prefers small modules with clear responsibilities.
- Uses anyhow for app errors, thiserror for library errors.
- Uses clap derive for CLI commands.
- Uses serde for config and schema serialization.
- Never panics for normal user errors.

### Test Agent
Responsible for writing tests alongside implementation.
- Uses ordinary cargo test.
- Uses assert_cmd for CLI tests.
- Uses tempfile for isolated workspace tests.
- Uses wiremock for HTTP provider adapter tests.
- Uses insta for snapshot tests where useful.

### Docs Agent
Responsible for README and docs/ updates.
- Keeps docs practical and concise.
- Updates docs when commands or config change.

## Agent Rules
1. Every story must be small enough for one focused iteration.
2. Run quality checks after every story:
   - cargo fmt --check
   - cargo clippy --all-targets --all-features -- -D warnings
   - cargo test --all-features
   - cargo build
3. Commit after every passing story with a clear message.
4. Update prd.json and progress.txt after every story.
5. Do not implement ahead of the current story.
6. Do not refactor unrelated code unless the story requires it.
7. Mock providers and dry-run must work without real API keys.
8. Never hardcode secrets or model IDs.
