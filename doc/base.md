# base

## 自动化

本项目通过 GitHub Actions 实现持续集成（CI），确保代码质量。自动化流程涵盖格式检查、静态分析、测试验证和跨平台构建。

### CI 工作流

| 工作流 | 触发条件 | 说明 |
|--------|----------|------|
| `ci.yml` | push to main / pull_request | 格式检查 + clippy + 三平台测试矩阵 |
| `build.yml` | push to main / pull_request | 跨平台编译检查（macOS / Windows / Linux） |
| `release.yml` | push tag `v*` | 三平台 release 构建并上传 artifact |

### 工作流详情

#### ci.yml — 持续集成

1. **Format 检查**：安装 rustfmt，执行 `cargo fmt --all -- --check`，确保全 workspace 格式一致
2. **Clippy 静态分析**：安装 clippy，执行 `cargo clippy --workspace --all-targets -- -D warnings`
3. **测试矩阵**：在三平台（ubuntu / windows / macos）上执行 `cargo test --workspace`

#### build.yml — 跨平台编译

三平台并行执行 `cargo build --workspace`，验证代码在各平台上可编译。

#### release.yml — 发布构建

推送 `v*` 标签时触发，在三平台（linux-gnu / windows-msvc / macos-darwin）上执行 release 构建，并上传 artifact。

### 质量门禁

- **格式检查**：`cargo fmt --all -- --check`，确保全 workspace 格式一致
  - 配置：`rustfmt.toml`，包含行宽、缩进、换行符、导入排序等规则
- **Clippy 静态分析**：`cargo clippy --workspace --all-targets -- -D warnings`
  - 采用 `clippy::all` + `clippy::pedantic` 严格模式
  - 各 crate `lib.rs` 配置：`#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented)]`
  - 测试代码可通过 `#[allow(...)]` 局部放宽
- **测试**：`cargo test --workspace`，覆盖单元测试、集成测试和文档测试

### 配置文件

| 文件 | 作用 |
|------|------|
| `rustfmt.toml` | rustfmt 配置，定义行宽、缩进、换行符、字段简写、导入排序、注释格式化等规则 |
| `clippy.toml` | clippy 阈值配置（cognitive-complexity、too-many-arguments、type-complexity） |
| `.gitattributes` | 跨平台换行符规范（源码 LF、Windows 批处理 CRLF、二进制文件不转换） |

### 本地检查

`scripts/` 目录提供本地一键检查脚本，在 push 前可先跑一遍，避免 CI 失败。

#### Linux / macOS

```bash
./scripts/check.sh
```

#### Windows (PowerShell)

```powershell
.\scripts\check.ps1
```

> **注意**：Windows 上首次运行 PowerShell 脚本可能需要修改执行策略。如遇权限问题，请执行：
> ```powershell
> Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
> ```

#### 执行流程

两个脚本执行相同的检查流程：

1. **Format 检查**：`cargo fmt --all -- --check`
2. **Clippy 分析**：`cargo clippy --workspace --all-targets -- -D warnings`
3. **测试运行**：`cargo test --workspace`

任一环节失败立即终止，后续步骤不再执行。

#### 手动执行单条检查

也可以直接运行单条命令：

```bash
# 格式检查（只检查，不修改）
cargo fmt --all -- --check

# 自动修复格式
cargo fmt --all

# Clippy 静态分析
cargo clippy --workspace --all-targets -- -D warnings

# 运行测试
cargo test --workspace
```

### 后续规划

- 性能基线测试纳入 CI（criterion benchmark）
- PR 性能回归检测（超过 10% 拦截合并）
- 大 vault 测试数据回归测试
