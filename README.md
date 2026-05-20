# Orca Agent CLI

Orca Agent CLI is a Rust-native AI coding orchestration tool. It learns your project, builds persistent memory, creates context packets, routes tasks across AI providers (Ollama, Claude, Codex, Cursor), and minimizes premium token usage.

## Quick start

```bash
# Build
cargo build --release

# Initialize a project workspace
orca init

# Validate configuration
orca config validate

# Scan the repository
orca scan

# Build a context packet for a task
orca context TASK-001

# See routing recommendation
orca route TASK-001

# Dry-run execution
orca execute --dry-run TASK-001

# Review results
orca review TASK-001

# Update memory
orca memory update TASK-001

# Full dry-run orchestration
orca run --dry-run TASK-001

# Check project status
orca status
```

## Installation

Requires Rust 1.75+.

```bash
cargo install --path .
```

## Documentation

- [Getting started](docs/getting-started.md)
- [Configuration](docs/configuration.md)
- [Model routing](docs/model-routing.md)
- [Memory system](docs/memory-system.md)
- [Providers](docs/providers.md)
- [Ralph development workflow](docs/ralph-development-workflow.md)
- [Cursor integration](docs/cursor-integration.md)

## Quality

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build
```

## License

MIT
