# Echo

一个使用 Rust + GPUI 构建的桌面笔记应用。

## 项目结构

```
echo/
├── echo-core/          # 后端核心：配置系统、统一错误、ID 类型、日志系统
│   └── src/
│       ├── config/     # 配置加载、分层合并、持久化、校验
│       │   ├── mod.rs
│       │   ├── schema.rs
│       │   ├── defaults.rs
│       │   ├── layers.rs
│       │   ├── persist.rs
│       │   └── validate.rs
│       ├── error.rs    # 统一错误类型 (EchoError 11 种变体)
│       ├── id.rs       # ID 类型系统 (NodeId / BlockId / FileId / VaultId)
│       ├── log/        # 日志模块
│       └── lib.rs
├── echo-markdown/        # Markdown 引擎：解析、序列化、WikiLink 后处理
│   └── src/
│       ├── block.rs      # 块模型 (BlockKind / Block / CalloutVariant / TableData)
│       ├── document.rs   # 文档结构 (Document)
│       ├── inline.rs     # 内联模型 (InlineTextTree / InlineFragment / InlineStyle)
│       ├── parser.rs     # Markdown 解析器 (parse)
│       ├── serialize.rs  # 序列化 (to_markdown)
│       ├── wikilink.rs   # WikiLink 后处理
│       └── error.rs      # 错误类型 (已统一到 echo-core)
├── echo-vault/          # 文件监控：跨平台递归监控 + 忽略过滤 + 防抖
│   └── src/
│       ├── watcher.rs    # 文件监控实现
│       ├── filter.rs     # gitignore 风格忽略过滤
│       ├── filter_cache.rs  # 过滤器编译缓存
│       └── debounce.rs   # 事件防抖
├── echo-app/            # 前端 UI：仓库管理、工作区
│   └── src/
│       ├── app.rs
│       ├── app_logic.rs
│       ├── screens/
│       │   ├── vault_manager/   # 仓库选择界面
│       │   └── workspace/       # 工作区界面
│       └── lib.rs
├── doc/                 # 开发文档
├── scripts/             # 本地检查脚本
└── .github/workflows/   # CI / Build / Release
```

## 功能特性

- **分层配置系统**：默认值 → 全局配置 → 工作区配置，递归合并
- **统一错误类型**：`EchoError` 统一所有 crate 的错误处理
- **ID 类型系统**：编译时区分的 `NodeId` / `BlockId` / `FileId` / `VaultId`
- **Markdown 引擎**：CommonMark 解析 + Obsidian 扩展（WikiLink / Callout / Mermaid / 标签 / 注释 / 数学公式）
- **文件监控**：跨平台文件夹递归监控 + gitignore 风格忽略模式 + 事件防抖
- **日志系统**：基于 tracing，支持控制台输出、文件输出（按日期轮转）与运行时级别热更新
- **跨平台**：支持 macOS / Windows / Linux
- **响应式 UI**：基于 GPUI 框架，配置变更自动切换界面
- **CI/CD**：GitHub Actions 自动格式检查、Clippy 静态分析、三平台测试与构建

## 快速开始

### 环境要求

- Rust 工具链（edition 2024）
- Cargo

### 开发

```bash
# 克隆仓库
git clone git@github.com:panzhifu/echo.git
cd echo

# 构建
cargo build --workspace

# 运行
cargo run

# 运行测试
cargo test --workspace

# 本地检查（格式 + Clippy + 测试）
./scripts/check.sh        # Linux / macOS
.\scripts\check.ps1       # Windows
```

## 配置

首次启动时，应用会显示仓库管理界面。选择一个本地目录作为笔记仓库后，配置会自动保存到 `~/.config/echo/config.toml`。

配置示例：

```toml
[vault]
path = "/home/user/notes"
auto_index = true

[[vault.recent]]
path = "/home/user/notes"
name = "My Notes"
last_opened = "2026-08-08T10:00:00Z"

[log]
level = "info"
console_output = true

[editor]
tab_size = 4
show_line_numbers = true

[theme]
mode = "dark"
```

## 文档

| 文档 | 说明 |
|------|------|
| [doc/base.md](doc/base.md) | 自动化 CI/CD 配置 |
| [doc/echo-core.md](doc/echo-core.md) | echo-core 开发文档 |
| [doc/echo-markdown.md](doc/echo-markdown.md) | echo-markdown 开发文档 |
| [doc/echo-vault.md](doc/echo-vault.md) | echo-vault 开发文档 |
| [doc/echo-app.md](doc/echo-app.md) | echo-app 开发文档 |

## 技术栈

- **语言**：Rust 2024
- **UI 框架**：[GPUI](https://github.com/zed-industries/zed)（Zed 编辑器的 UI 框架）
- **Markdown**：pulldown-cmark（CommonMark + 扩展）
- **序列化**：serde + toml
- **错误处理**：thiserror + 统一 EchoError
- **文件监控**：notify 6 + ignore
- **日志**：tracing + tracing-subscriber

## 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。
