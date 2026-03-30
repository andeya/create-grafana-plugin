# create-grafana-plugin

**One command. A production-ready Grafana plugin project.**

```bash
npx create-grafana-plugin@latest
```

Scaffold **panel**, **data source**, or **app** plugins with:

- **Rspack** — Rust-based bundler with AMD output for Grafana
- **Bun** — fast runtime, test runner, and package manager
- **Biome** — Rust-powered formatter and linter (replaces Prettier + ESLint)
- **Rust WASM** (optional) — wasm-pack crate + TypeScript bridge for native-speed computation
- **Docker observability stack** (optional) — Grafana + Prometheus + Tempo + Loki, auto-provisioned
- **Mock telemetry** (optional) — production-grade generator: correlated traces, logs, and metrics
- **Smart updates** — `update` subcommand refreshes managed boilerplate while preserving custom code

## Quick example

```bash
npx create-grafana-plugin \
  --name my-dashboard \
  --type panel \
  --org acme \
  --author "Jane Doe" \
  --wasm --docker --mock

cd my-dashboard
bun run setup
docker compose up -d
bun run dev
```

Open `http://localhost:3000` — Grafana is pre-configured with datasources and live mock data.

## How it works

This npm package is a thin wrapper that downloads the native Rust CLI binary for your platform via optional dependencies. The CLI generates a complete, opinionated project structure with all config, build scripts, CI workflow, and optional Docker stack ready to go.

## Documentation

Full documentation, configuration reference, and template customization guide:

**https://github.com/andeya/create-grafana-plugin**
