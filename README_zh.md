# create-grafana-plugin

**用于搭建可投入生产的 Grafana 插件项目** —— 基于 Rspack 的前端构建（Grafana AMD 输出）、可选的 Rust WASM、Docker 开发环境、Mock 数据工具，以及生成项目统一使用 **Bun**（安装、测试、构建脚本）。提供 `update` 子命令，用于将已有项目与最新模板对齐。

[English](README.md) · **中文**

## 特性

- **插件类型**：Panel、数据源（datasource）、应用（app）插件 —— 按 Grafana 插件模型选择合适形态。
- **可选 Rust WASM**：提供 wasm-pack 就绪的 crate 与 TypeScript 桥接，在浏览器中获得接近原生的性能。
- **Docker 开发环境**：可选 Compose 编排，统一 Grafana + 插件开发体验。
- **Mock 数据生成器**：可选 `otel-mock` 服务（通常与 Docker 工作流配合使用）。
- **Bun**：生成的插件项目使用 **Bun**（`packageManager`、`bun test`、`bun run build`、TypeScript 工具脚本）。
- **非交互与配置文件**：通过命令行参数或 `.grafana-plugin.toml` 便于 CI 与可重复搭建。
- **update 子命令**：在已有项目中按当前工具版本刷新「受管理」文件，支持 `--dry-run` 预览差异。

## 快速开始

```bash
npx create-grafana-plugin@latest
```

按提示输入插件名称、类型、是否包含 WASM、Docker、Mock 数据（启用 Docker 时）。工具会创建与插件同名的目录并输出后续步骤（`cd`、`bun install` 或 `bun run setup`、构建、可选 `docker compose up`）。

## 安装

### crates.io

```bash
cargo install create-grafana-plugin
```

将 `~/.cargo/bin` 加入 `PATH` 后，直接运行 `create-grafana-plugin`。

### npm（npx）

```bash
npx create-grafana-plugin@latest
```

npm 包会调用同一 CLI；无需全局安装 Rust 即可用 `npx` 单次执行。

### 从源码安装

```bash
git clone https://github.com/andeya/create-grafana-plugin.git
cd create-grafana-plugin
cargo install --path cli
```

从工作区中的 `cli` crate 构建并安装 `create-grafana-plugin` 二进制。

## 用法

### 交互模式

在未提供非交互所需参数时直接运行：

```bash
npx create-grafana-plugin
```

将依次提示：插件名称、描述、作者、组织、插件类型、WASM、Docker、Mock（在启用 Docker 时）。

### 非交互模式

提供 **name**、**type**、**author**、**org** 可跳过提示：

```bash
npx create-grafana-plugin \
  --name my-plugin \
  --type panel \
  --org myorg \
  --author "Your Name" \
  --description "My Grafana panel"
```

可选参数：`--wasm`、`--docker`、`--mock`（`--mock` 须同时指定 `--docker`）。

### 同时启用 WASM 与 Docker

```bash
npx create-grafana-plugin \
  --name my-plugin \
  --type panel \
  --org myorg \
  --author "Your Name" \
  --wasm \
  --docker
```

仅在启用 Docker 且设置 `--mock`（或在交互流程中选择）时才会包含 Mock 数据相关文件。

### 使用配置文件代替冗长参数

```bash
npx create-grafana-plugin --config .grafana-plugin.toml
```

### 更新已有项目

在**此前由本工具生成**的项目根目录（存在 `plugin.json`、`package.json`）下执行：

```bash
cd my-plugin
npx create-grafana-plugin update
```

更新器会从 `plugin.json`、`package.json` 以及目录结构（如是否存在 `Cargo.toml`、`docker-compose.yml`、`otel-mock/`）推断插件类型、组织、名称及 WASM/Docker/Mock。仅覆盖**受管理**的文件（生成代码中带标记，或属于已知的 JSON 管理路径）；其余文件不覆盖或跳过。

### 预演更新（dry run）

```bash
npx create-grafana-plugin update --dry-run
```

打印差异与将创建的新文件，不写入磁盘。

## 配置文件（`.grafana-plugin.toml`）

使用 TOML 驱动搭建，避免超长命令行。通过 `--config <路径>` 指定。

| 字段          | 类型   | 说明                                                                       |
| ------------- | ------ | -------------------------------------------------------------------------- |
| `name`        | string | 插件名称（规范为 kebab-case）。                                            |
| `description` | string | 描述。                                                                     |
| `author`      | string | 作者显示名。                                                               |
| `org`         | string | Grafana 插件 id 中的组织段（完整 id：`org-name`）。                        |
| `type`        | string | `panel`、`datasource` 或 `app`（`data-source` 可作为 datasource 的别名）。 |
| `wasm`        | bool   | 是否包含 Rust WASM 工作区与桥接代码。                                      |
| `docker`      | bool   | 是否包含 Docker Compose 与 provisioning。                                  |
| `mock`        | bool   | 是否包含 Mock 数据生成器（通常与 Docker 一起使用）。                       |

示例：

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

合并规则上，部分 CLI 参数会覆盖 TOML（例如 `--wasm` 会强制启用 WASM）。生成后的项目**运行时不依赖**该文件；它仅作为 `create` 流程的可选输入。

## 生成项目结构

随插件类型与选项变化。以下为启用 **panel + WASM + Docker + Mock** 时的大致结构：

```text
my-plugin/
├── .github/workflows/ci.yml
├── .grafana-plugin-version      # 记录 scaffold / update 工具版本
├── Cargo.toml                   # WASM 工作区
├── docker-compose.yml           # Docker 选项
├── docker/provisioning/         # Prometheus、Loki、Tempo、数据源、插件等
├── otel-mock/                   # Mock 选项（配合 Docker）
├── scripts/
├── src/
│   ├── components/MainPanel.tsx
│   ├── module.ts
│   ├── types/index.ts
│   └── services/wasm-bridge.ts  # WASM 选项
├── my_plugin/                   # Rust crate 目录（由插件名推导）
│   └── src/lib.rs
├── plugin.json
├── package.json
├── tsconfig.json
├── tsconfig.test.json
├── rspack.config.ts
├── bunfig.toml
├── tests/
├── README.md
└── AGENTS.md
```

数据源与应用插件会替换为各自特有的 `src/` 文件（如查询编辑器、应用页面）。未选 WASM 则无 Rust 工作区与 `wasm-bridge`；未选 Docker 则无 `docker-compose.yml`、`docker/`、`otel-mock/`。

## 模板定制

本仓库在 `templates/` 下提供 **Tera** 模板（`base`、`panel`、`datasource`、`app`、`wasm`、`docker`、`mock`）。带 `.tera` 后缀的文件参与渲染；二进制资源原样复制。

- **Fork 或 vendor** 本仓库以修改默认依赖、目录布局或脚手架内容。
- **上下文变量**（如 `plugin_name`、`org`、`plugin_id`、`crate_name`、`has_wasm`）由 CLI 中的 `TemplateContext` 填充。
- **更新机制**：受管理的生成文件带有标记（例如 `// @managed by create-grafana-plugin — do not edit`），以便 `create-grafana-plugin update` 安全合并模板变更。自定义逻辑建议放在未标记的文件中，否则更新时可能被跳过或无法覆盖。

## CLI 参考

搭建（主命令）全局选项：

| 选项                   | 说明                                                   |
| ---------------------- | ------------------------------------------------------ |
| `--name <NAME>`        | 插件名称（kebab-case）。                               |
| `--description <TEXT>` | 插件描述。                                             |
| `--author <NAME>`      | 作者名。                                               |
| `--org <ORG>`          | 插件 id 中的组织段。                                   |
| `--type <TYPE>`        | `panel`、`datasource` 或 `app`。                       |
| `--wasm`               | 包含 Rust WASM crate 与桥接。                          |
| `--docker`             | 包含基于 Docker 的开发环境。                           |
| `--mock`               | 包含 Mock 数据生成器（在 scaffold 中与 Docker 配合）。 |
| `--config <FILE>`      | 从 TOML 加载配置。                                     |

`update` 子命令：

| 选项        | 说明                         |
| ----------- | ---------------------------- |
| `--dry-run` | 仅显示差异与新文件，不写盘。 |

内置：`-h` / `--help`，`-V` / `--version`。

**非交互搭建**须同时提供 `--name`、`--type`、`--author`、`--org`。若缺少任一字段，将进入交互模式（在提示信息足够的前提下）。

## 开发

```bash
bun run fmt          # 格式化所有 Rust 代码（cargo fmt）
bun run lint         # 检查格式 + clippy 警告
bun run test         # 运行所有测试（cargo test）
bun run verify       # lint + test 一步到位
```

## 版本号

Rust crate 与 npm 包共用同一套 semver。版本号唯一真实来源为根目录 `Cargo.toml` 的 `[workspace.package] version`。一键同步所有位置：

```bash
bun run bump:patch          # 0.1.0 → 0.1.1
bun run bump:minor          # 0.1.0 → 0.2.0
bun run bump:major          # 0.1.0 → 1.0.0
bun run bump -- 2.0.0       # 直接指定版本号
```

该命令会同时更新 `Cargo.toml`、根 `package.json`、所有 npm 平台包以及 meta 包中的 `optionalDependencies` 版本。

发布时提交版本变更后，推送 Git 标签 `vX.Y.Z`，其数字部分须与 `Cargo.toml` 中版本一致（见 `.github/workflows/release.yml`）。维护说明见 [`AGENTS.md`](AGENTS.md)。

## 参与贡献

欢迎贡献。

1. 较大改动建议先开 issue 对齐范围。
2. 提交保持原子化；**提交说明请使用英文**。
3. 提交 PR 前请执行：

   ```bash
   bun run verify
   ```

4. 模板与 CLI 行为请与现有风格一致；行为变更时请补充或更新测试。

## 许可证

本项目使用 [MIT License](LICENSE) 授权。
