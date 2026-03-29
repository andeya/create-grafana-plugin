# create-grafana-plugin

**Scaffold production-ready Grafana plugin projects** — Rspack-based frontend (AMD for Grafana), optional Rust WASM, Docker development stack, mock data tooling, and **Bun** for the generated frontend (install, test, build scripts). Includes an `update` command to align existing projects with the latest template.

**English** · [中文](README_zh.md)

## Features

- **Plugin types**: Panel, data source, and app plugins — choose the integration shape that fits Grafana’s plugin model.
- **Optional Rust WASM**: wasm-pack–ready crate and TypeScript bridge when you need native performance in the browser.
- **Docker development environment**: Optional Compose stack for a repeatable Grafana + plugin workflow.
- **Mock data generator**: Optional `otel-mock` service (intended alongside the Docker workflow).
- **Bun**: Generated plugins use **Bun** (`packageManager`, `bun test`, `bun run build`, TypeScript utility scripts).
- **Non-interactive and config-driven**: Pass flags or a `.grafana-plugin.toml` file for CI and repeatable scaffolds.
- **Update subcommand**: Refresh managed files in an existing project against the current tool version, with `--dry-run` to preview diffs.

## Quick start

```bash
npx create-grafana-plugin@latest
```

Follow the prompts for plugin name, type, optional WASM, Docker, and mock data (when Docker is enabled). The tool creates a new directory named after your plugin and prints next steps (`cd`, `bun install` or `bun run setup`, build, optional `docker compose up`).

## Installation

### crates.io

```bash
cargo install create-grafana-plugin
```

Ensure `~/.cargo/bin` is on your `PATH`, then run `create-grafana-plugin`.

### npm (npx)

```bash
npx create-grafana-plugin@latest
```

The npm package invokes the same CLI binary; use `npx` for one-off runs without a global Rust install.

### From source

```bash
git clone https://github.com/andeya/create-grafana-plugin.git
cd create-grafana-plugin
cargo install --path cli
```

This builds and installs the `create-grafana-plugin` binary from the workspace `cli` crate.

## Usage

### Interactive mode

Run without the flags required for non-interactive mode:

```bash
npx create-grafana-plugin
```

You will be prompted for plugin name, description, author, organization, plugin type, WASM, Docker, and mock data (when Docker is enabled).

### Non-interactive mode

Provide **name**, **type**, **author**, and **org** to skip prompts:

```bash
npx create-grafana-plugin \
  --name my-plugin \
  --type panel \
  --org myorg \
  --author "Your Name" \
  --description "My Grafana panel"
```

Optional flags: `--wasm`, `--docker`, `--mock` (`--mock` requires `--docker`).

### With WASM and Docker

```bash
npx create-grafana-plugin \
  --name my-plugin \
  --type panel \
  --org myorg \
  --author "Your Name" \
  --wasm \
  --docker
```

Mock data is only included when Docker is enabled and `--mock` is set (or chosen in interactive mode).

### Configuration file instead of repeating flags

```bash
npx create-grafana-plugin --config .grafana-plugin.toml
```

### Update an existing project

Run inside the **root directory** of a project previously created by this tool (where `plugin.json` and `package.json` live):

```bash
cd my-plugin
npx create-grafana-plugin update
```

The updater discovers plugin type, org, name, and optional WASM/Docker/mock layout from `plugin.json`, `package.json`, and the filesystem. It only overwrites **managed** files (marked in generated sources or listed as known JSON paths); other files are left unchanged or skipped.

### Dry run (preview changes)

```bash
npx create-grafana-plugin update --dry-run
```

Prints diffs and planned new files without writing to disk.

## Configuration file (`.grafana-plugin.toml`)

Use a TOML file to drive scaffolding without long command lines. Pass it with `--config <path>`.

| Field         | Type   | Description                                                                             |
| ------------- | ------ | --------------------------------------------------------------------------------------- |
| `name`        | string | Plugin name (normalized to kebab-case).                                                 |
| `description` | string | Human-readable description.                                                             |
| `author`      | string | Author display name.                                                                    |
| `org`         | string | Grafana plugin org segment (plugin id: `org-name`).                                     |
| `type`        | string | `panel`, `datasource`, or `app` (`data-source` is accepted as an alias for datasource). |
| `wasm`        | bool   | Include Rust WASM workspace and bridge.                                                 |
| `docker`      | bool   | Include Docker Compose and provisioning.                                                |
| `mock`        | bool   | Include mock data generator (typically with Docker).                                    |

Example:

```toml
name = "my-org-panel"
description = "My Grafana plugin"
author = "Your Name"
org = "myorg"
type = "panel"
wasm = true
docker = true
mock = true
```

CLI flags override TOML values where the implementation merges them (e.g. `--wasm` forces WASM on). The generated project does **not** require this file at runtime; it is optional input for the scaffold command only.

## Generated project structure

Layout varies by plugin type and selected options. A **panel** with WASM, Docker, and mock data resembles:

```text
my-plugin/
├── .github/workflows/ci.yml
├── .grafana-plugin-version      # scaffold / update tool version
├── Cargo.toml                   # workspace (WASM)
├── docker-compose.yml           # Docker option
├── docker/provisioning/         # Prometheus, Loki, Tempo, datasources, plugins
├── otel-mock/                   # mock option (with Docker)
├── scripts/
├── src/
│   ├── components/MainPanel.tsx
│   ├── module.ts
│   ├── types/index.ts
│   └── services/wasm-bridge.ts  # WASM option
├── my_plugin/                   # Rust crate dir (name from plugin name)
│   └── src/lib.rs
├── plugin.json
├── package.json
├── tsconfig.json
├── tsconfig.test.json
├── rspack.config.js
├── bunfig.toml
├── tests/
├── README.md
└── AGENTS.md
```

Data source and app plugins swap in type-specific `src/` files (e.g. query editor, app pages). Without WASM, the Rust workspace and `wasm-bridge` are omitted. Without Docker, `docker-compose.yml`, `docker/`, and `otel-mock/` are omitted.

## Template customization

This repository ships **Tera** templates under `templates/` (`base`, `panel`, `datasource`, `app`, `wasm`, `docker`, `mock`). Files may use the `.tera` suffix for rendering; binary assets are copied as-is.

- **Fork or vendor** the repo to change defaults, dependencies, or folder layout.
- **Context variables** (e.g. `plugin_name`, `org`, `plugin_id`, `crate_name`, `has_wasm`) are filled from [`TemplateContext`](cli/src/template.rs) in the CLI.
- **Updates**: Managed generated files include markers (e.g. `// @managed by create-grafana-plugin — do not edit`) so `create-grafana-plugin update` can merge template changes safely. Custom edits should live in unmanged files or files without those markers, or they may be skipped on update.

## CLI reference

Global options (scaffold):

| Option                 | Description                                            |
| ---------------------- | ------------------------------------------------------ |
| `--name <NAME>`        | Plugin name (kebab-case).                              |
| `--description <TEXT>` | Plugin description.                                    |
| `--author <NAME>`      | Author name.                                           |
| `--org <ORG>`          | Organization segment for the plugin id.                |
| `--type <TYPE>`        | `panel`, `datasource`, or `app`.                       |
| `--wasm`               | Include Rust WASM crate and bridge.                    |
| `--docker`             | Include Docker-based dev environment.                  |
| `--mock`               | Include mock data generator (with Docker in scaffold). |
| `--config <FILE>`      | Load settings from a TOML file.                        |

Subcommand `update`:

| Option      | Description                               |
| ----------- | ----------------------------------------- |
| `--dry-run` | Show diffs and new files without writing. |

Built-in: `-h` / `--help`, `-V` / `--version`.

**Non-interactive scaffold** requires `--name`, `--type`, `--author`, and `--org`. If any of these are missing, the tool runs interactively (unless insufficient for prompts).

## Development

```bash
bun run fmt          # format all Rust code (cargo fmt)
bun run lint         # check formatting + clippy warnings
bun run test         # run all tests (cargo test)
bun run verify       # lint + test in one step
```

## Versioning

The Rust crate and npm packages share one semver. The source of truth is `[workspace.package] version` in the root `Cargo.toml`. Bump everything in one step:

```bash
bun run bump:patch          # 0.1.0 → 0.1.1
bun run bump:minor          # 0.1.0 → 0.2.0
bun run bump:major          # 0.1.0 → 1.0.0
bun run bump -- 2.0.0       # set an explicit version
```

This updates `Cargo.toml`, root `package.json`, all npm platform packages, and `optionalDependencies` versions in the meta package.

To publish, commit the version bump, then push a Git tag `vX.Y.Z` whose numeric part matches `Cargo.toml` (see `.github/workflows/release.yml`). Maintainer notes: [`AGENTS.md`](AGENTS.md).

## Contributing

Contributions are welcome.

1. For larger changes, open an issue first to align on scope.
2. Keep commits focused; use **English** commit messages.
3. Before submitting a pull request, run:

   ```bash
   bun run verify
   ```

4. Match existing style for templates and CLI behavior; add or update tests when behavior changes.

## License

This project is licensed under the [MIT License](LICENSE).
