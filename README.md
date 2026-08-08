# Echo

一个使用 Rust + GPUI 构建的桌面笔记应用。

## 项目结构

```
echo/
├── echo-core/          # 后端核心：配置系统、错误处理
│   └── src/
│       ├── config/     # 配置加载、分层合并、持久化、校验
│       │   ├── mod.rs
│       │   ├── schema.rs
│       │   ├── defaults.rs
│       │   ├── layers.rs
│       │   ├── persist.rs
│       │   └── validate.rs
│       ├── error.rs    # 结构化错误类型
│       └── lib.rs
├── echo-app/           # 前端 UI：仓库管理、工作区
│   └── src/
│       ├── app.rs
│       ├── screens/
│       │   ├── vault_manager/   # 仓库选择界面
│       │   └── workspace/       # 工作区界面
│       └── lib.rs
├── doc/                # 开发文档
├── scripts/            # 本地检查脚本
└── .github/workflows/  # CI / Build / Release
```

## 功能特性

- **分层配置系统**：默认值 → 全局配置 (`~/.config/echo/config.toml`) → 工作区配置 (`<workspace>/.echo.toml`)
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
```

## 技术栈

- **语言**：Rust 2024
- **UI 框架**：[GPUI](https://github.com/zed-industries/zed)（Zed 编辑器的 UI 框架）
- **序列化**：serde + toml
- **错误处理**：thiserror

## 许可证

待定
