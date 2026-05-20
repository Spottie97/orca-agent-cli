# Ralph development workflow

Orca is developed using the Ralph Loop technique: small, testable, committed iterations.

## Principles

- One story per iteration
- Quality gates after every change
- Commit after every passing story
- No large rewrites

## Quality gates

Before committing, all of the following must pass:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build
```

## Story format

Stories are defined in `prd.json`:

```json
{
  "id": "ORCA-XXX",
  "title": "...",
  "description": "...",
  "acceptance_criteria": [...],
  "quality_checks": [...],
  "passes": false
}
```

## Workflow

1. Pick the next unimplemented story
2. Implement the smallest change that satisfies acceptance criteria
3. Run quality gates
4. Fix any errors
5. Commit with story ID in message
6. Update `prd.json` and `progress.txt`
7. Continue to next story

## Benefits

- Predictable progress
- Always green tests
- Easy rollback
- Clear history
