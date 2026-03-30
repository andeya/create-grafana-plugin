# create-grafana-plugin

**一条命令，生成可投产的 Grafana 插件项目。**

支持 Panel、Data Source、App 三种插件类型，搭配最快的现代前端工具链 —— **Rspack** 构建、**Bun** 运行时、**Biome** 格式化与代码检查；可选 **Rust WASM** 提供浏览器原生性能，可选 **Docker 全链路可观测栈**（Grafana + Prometheus + Tempo + Loki），以及可选的 **Mock 遥测数据生成器**（开箱即用的关联 traces、logs、metrics）。

[English](README.md) · **中文**

---

## 为什么选择 create-grafana-plugin？

| 痛点 | 解决方式 |
| --- | --- |
| Grafana 官方无针对自定义技术栈的脚手架工具 | 秒级生成完整、有观点的项目 |
| Rspack + AMD 输出配置复杂 | 预置 `rspack.config.js`，JSDoc 类型 + SWC + AMD externals 开箱即用 |
| Rust WASM 集成到 Grafana 插件需大量手工接线 | `--wasm` 自动添加 wasm-pack crate、TypeScript 桥接及构建脚本 |
| 本地开发需要 Grafana 与后端服务协同运行 | `--docker` 一键编排 Grafana、Prometheus、Tempo、Loki |
| 可观测仪表盘缺少真实测试数据 | `--mock` 提供 Rust 实现的多服务分布式链路 + 关联日志 + Prometheus 指标生成器 |
| 团队间模板同步困难 | `update` 子命令按模板刷新受管理文件，自定义代码不受影响 |
| 多插件项目同时运行端口冲突 | `--port-offset` 全局偏移所有 Docker 宿主端口 |

## 亮点

- **Rust 驱动的 CLI** —— 单一静态二进制，毫秒级启动，跨平台（macOS、Linux、Windows）。
- **同类最快工具链** —— Rspack（Rust 打包器）、Bun（运行时 + 测试）、Biome（Rust 格式化/检查）。无 webpack、无 Jest、无 Prettier。
- **三种插件类型** —— Panel、Data Source、App，各自携带专属组件、类型定义与入口。
- **可选 Rust → WASM** —— wasm-pack crate、TypeScript 桥接（`wasm-bridge.ts`）、Cargo 工作区，适合浏览器端计算密集型逻辑。
- **完整可观测开发栈** —— Docker Compose 编排 Grafana、Prometheus、Tempo、Loki，数据源自动配置，项目级网络隔离。
- **生产级 Mock 数据** —— `otel-mock` 生成逼真的多服务分布式链路（OTLP → Tempo）、关联 JSON 日志（→ Loki）、可被 Prometheus 抓取的指标，支持配置心跳间隔。
- **端口隔离** —— `--port-offset N` 将所有宿主端口偏移 N（如 `--port-offset 100` → Grafana 3100、Prometheus 9190）。
- **配置驱动** —— 交互提示、CLI 参数或 `.grafana-plugin.toml` 文件，对 CI 友好。
- **智能更新** —— `create-grafana-plugin update` 将受管理文件与最新模板 diff；`--dry-run` 预览变更。
- **生成即规范** —— 脚手架后自动执行 Biome 和 `cargo fmt`，输出代码零 lint 错误。
- **CI 开箱即用** —— 每个生成项目附带 GitHub Actions（lint + test + build）。

## 快速开始

```bash
npx create-grafana-plugin@latest
```

按提示操作 —— 或完全非交互：

```bash
npx create-grafana-plugin \
  --name my-dashboard \
  --type panel \
  --org acme \
  --author "Jane Doe" \
  --wasm \
  --docker \
  --mock
```

然后：

```bash
cd my-dashboard
bun run setup        # 安装依赖 + 构建 WASM（如启用）
docker compose up -d # 启动 Grafana + 后端（如 --docker）
bun run dev          # Rspack watch 模式
```

打开 `http://localhost:3000` —— 数据源、插件、Mock 数据均已预配置就绪。

## 安装

### npm（推荐）

```bash
npx create-grafana-plugin@latest
```

只需 Node/Bun 即可，npm 包会自动拉取对应平台的原生二进制。

### Cargo（Rust）

```bash
cargo install create-grafana-plugin
```

### 从源码安装

```bash
git clone https://github.com/andeya/create-grafana-plugin.git
cd create-grafana-plugin
cargo install --path cli
```

## 用法

### 交互模式

```bash
npx create-grafana-plugin
```

依次提示：插件名称、描述、作者、组织、类型、WASM、Docker、Mock 数据。

### 非交互模式

提供 `--name`、`--type`、`--author`、`--org` 即可跳过所有提示：

```bash
npx create-grafana-plugin \
  --name my-plugin \
  --type datasource \
  --org myorg \
  --author "Your Name" \
  --description "实时指标数据源" \
  --docker \
  --mock \
  --port-offset 200
```

### 配置文件

```bash
npx create-grafana-plugin --config .grafana-plugin.toml
```

```toml
name = "my-org-panel"
description = "My Grafana plugin"
author = "Your Name"
org = "myorg"
type = "panel"
wasm = true
docker = true
mock = true
port_offset = 100
```

CLI 参数覆盖 TOML 值。生成后的项目**运行时不依赖**该文件。

### 更新已有项目

```bash
cd my-plugin
npx create-grafana-plugin update
```

更新器从 `plugin.json`、`package.json` 及目录结构自动推断插件类型、组织、WASM/Docker/Mock 布局与端口偏移。仅覆盖 **受管理** 文件（带 `@managed` 标记），自定义代码不受影响。

```bash
npx create-grafana-plugin update --dry-run   # 预览差异，不写盘
```

## 配置参考（`.grafana-plugin.toml`）

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `name` | string | 插件名称（规范为 kebab-case） |
| `description` | string | 描述 |
| `author` | string | 作者显示名 |
| `org` | string | Grafana 组织段（插件 id = `org-name`） |
| `type` | string | `panel`、`datasource` 或 `app` |
| `wasm` | bool | 是否包含 Rust WASM 工作区与桥接 |
| `docker` | bool | 是否包含 Docker Compose + provisioning |
| `mock` | bool | 是否包含 Mock 数据生成器（须启用 `docker`） |
| `port_offset` | integer | Docker 服务全局宿主端口偏移量 |

## 生成项目结构

以 **Panel 插件 + WASM + Docker + Mock** 为例：

```text
my-plugin/
├── .github/workflows/ci.yml       # lint + test + build
├── .grafana-plugin-version         # 脚手架工具版本
├── biome.json                      # Biome 配置（格式化 + 检查）
├── bunfig.toml                     # Bun 配置
├── Cargo.toml                      # Rust 工作区（--wasm 时）
├── docker-compose.yml              # Grafana + Prometheus + Tempo + Loki
├── provisioning/                   # 自动配置的数据源与服务
├── otel-mock/                      # Rust Mock 遥测数据生成器
│   └── src/
│       ├── main.rs                 # OTLP traces + Loki logs + Prometheus metrics
│       ├── graph.rs                # 合成多服务调用图
│       └── loki_push.rs            # Loki push API 客户端
├── src/
│   ├── components/MainPanel.tsx    # 插件 UI
│   ├── module.ts                   # Grafana 入口
│   ├── types/index.ts              # 共享类型
│   └── services/wasm-bridge.ts     # WASM 桥接（--wasm 时）
├── my_plugin/src/lib.rs            # Rust WASM crate（--wasm 时）
├── scripts/
│   ├── bump-version.ts             # semver 版本升级工具
│   └── clean-plugin-dist.ts        # dist 清理
├── tests/                          # Bun 测试套件
├── plugin.json                     # Grafana 插件清单
├── package.json                    # Bun 脚本
├── tsconfig.json
├── rspack.config.js                # Rspack（Grafana AMD 输出）
├── README.md
└── AGENTS.md                       # AI 编码规范
```

插件类型决定 `src/` 内容：Panel → `MainPanel.tsx`，DataSource → `QueryEditor.tsx` + `ConfigEditor.tsx` + `DataSource.ts`，App → `AppRootPage.tsx` + `AppConfig.tsx`。

## CLI 参考

### 搭建（默认命令）

| 选项 | 说明 |
| --- | --- |
| `--name <NAME>` | 插件名称（kebab-case） |
| `--description <TEXT>` | 插件描述 |
| `--author <NAME>` | 作者名 |
| `--org <ORG>` | 插件 id 中的组织段 |
| `--type <TYPE>` | `panel`、`datasource` 或 `app` |
| `--wasm` | 包含 Rust WASM crate 与桥接 |
| `--docker` | 包含基于 Docker 的开发环境 |
| `--mock` | 包含 Mock 数据生成器（须同时 `--docker`） |
| `--port-offset <N>` | 全局偏移所有 Docker 宿主端口 |
| `--config <FILE>` | 从 TOML 加载配置 |

### `update` 子命令

| 选项 | 说明 |
| --- | --- |
| `--dry-run` | 仅显示差异与新文件，不写盘 |

内置：`-h` / `--help`，`-V` / `--version`。

## 模板定制

模板位于 `templates/` 下，使用 [Tera](https://keats.github.io/tera/) 引擎（Jinja2 风格语法）。模板栈：`base`、`panel`、`datasource`、`app`、`wasm`、`docker`、`mock`。

- **Fork 或 vendor** 本仓库以修改默认依赖、目录布局或脚手架内容。
- **上下文变量**：`plugin_name`、`org`、`plugin_id`、`crate_name`、`has_wasm`、`has_docker`、`has_mock`、`port_offset` 等。详见 [`TemplateContext`](cli/src/template.rs)。
- **受管理标记**：生成文件中的 `@managed by create-grafana-plugin — do not edit` 标记确保 `update` 命令可安全合并模板变更。

## 开发（本仓库）

```bash
bun run fmt          # cargo fmt
bun run lint         # clippy + Biome
bun run test         # cargo test --workspace
bun run verify       # fmt + lint + test
```

## 版本号

Rust crate 与 npm 包共享统一 semver，唯一真实来源为 `Cargo.toml` 中的 `[workspace.package] version`。

```bash
bun run bump:patch   # 0.1.0 → 0.1.1
bun run bump:minor   # 0.1.0 → 0.2.0
bun run bump:major   # 0.1.0 → 1.0.0
```

发布：提交后推送 `vX.Y.Z` 标签，GitHub Actions 自动完成 crates.io + npm 发布。

## 参与贡献

欢迎贡献。

1. 较大改动建议先开 issue 对齐范围。
2. 提交说明与代码注释使用 **英文**。
3. 提交 PR 前请执行 `bun run verify`。
4. 模板与 CLI 行为请与现有风格一致；行为变更时请补充或更新测试。

## 许可证

[MIT](LICENSE)
