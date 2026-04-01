# AI Coding Standards

This file defines coding standards for AI assistants working on this project.

## Project Context

- **Project**: create-grafana-plugin — CLI scaffolding tool for Grafana plugins
- **Language**: Rust (CLI core)
- **Build**: Cargo workspace
- **Test**: cargo test

## Language & Style

- Git commit messages: **English**
- Code comments: **English**
- User-facing documentation: **Chinese (Simplified)** unless otherwise specified
- Comments explain _why_, not _what_

## Verification

```bash
bun run verify
```

This runs: `bun run lint` (Biome check + Clippy) then `bun run test` (cargo test).

## Version bumps

Keep `[workspace.package].version` in `Cargo.toml` in sync with every `package.json` (root and `packaging/npm/*`), including `optionalDependencies` in `packaging/npm/create-grafana-plugin`.

```bash
bun run bump:patch   # or bump:minor / bump:major
cargo build          # refresh Cargo.lock after version change
```

Then commit, tag `vX.Y.Z`, and push the tag to trigger the release workflow.

## Rust Rules

- Never use `.unwrap()` in library code — use `Result` and `?`
- Every `unsafe` has a `// SAFETY:` comment
- `cargo clippy -- -D warnings` zero tolerance
- `bun run format` before commit
- All public items have `///` doc comments
- Prefer `pub(crate)` over `pub` when not part of public API
