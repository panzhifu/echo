# echo-core 开发文档

## 任务清单

### 统一错误类型

- [x] 定义 `EchoError` 枚举 (`error.rs` - 11 种变体)
- [x] 统一配置错误 (`ConfigError` → `EchoError::ConfigValidation` / `EchoError::ConfigParse`)
- [x] 统一 Markdown 错误 (`MarkdownError` → `EchoError::Markdown`)
- [x] 统一 Vault 错误 (`VaultError` → `EchoError::VaultInit` / `EchoError::VaultNotify`)
- [x] 日志错误保留 (`LogError` 独立枚举 + `impl From<LogError> for EchoError`)
- [x] 便捷构造函数 (`EchoError::markdown()` / `vault_init()` / `vault_notify()` / `config_validation()`)
- [x] 向后兼容类型别名 (`ConfigError` / `MarkdownError` / `VaultError` → `EchoError`)

### ID 类型系统

- [x] 定义 `Id<T>` newtype 包装 Uuid (`id.rs`)
- [x] 定义具体 ID 类型 (`NodeId` / `BlockId` / `FileId` / `VaultId`)
- [x] 定义 `Timestamp` 类型 (毫秒时间戳)
- [x] 实现公共 trait (Copy / Eq / Hash / Debug / Display / Serialize)
- [x] 支持编译时类型区分 (`NodeId` 与 `BlockId` 不可混用)
- [x] `now()` 和 `zero_timestamp()` 便捷函数

### 配置系统

- [x] 定义配置 schema (`schema.rs` - `ConfigData` / `VaultConfig` / `VaultEntry` / `LogConfig` / `LogLevel` / `RotationKind` / `ThemeConfig` / `EditorConfig` / `SidebarConfig`)
- [x] `extra` 兜底字段 (`ConfigData.extra: HashMap`，`#[serde(flatten)]`，未知字段向前兼容)
- [x] 定义默认值 (`defaults.rs` - `default_true()` / `default_tab_size()` / `default_sidebar_width()`，配合 `#[serde(default = "...")]`)
- [x] `VaultConfig::add_recent` (去重 + 移到最前)
- [x] 分层加载 (`layers.rs` - `Layers` / `merge()` / `load_layers()`，递归合并 TOML)
- [x] 持久化 (`persist.rs` - `save_config` / `load_config_from_path` / `default_config_path`，跨平台路径)
- [x] 统一保存入口 (`mod.rs` - `save_config_to_default()`)
- [x] 语义校验 (`validate.rs` - vault 路径非空 + 日志文件路径非空 + editor.tab_size>0 + theme.font_size>0 校验)
- [x] 统一加载入口 (`mod.rs` - `load_config()` 串联 加载->合并->校验)
- [x] 支持 theme / editor / sidebar 配置组 (`schema.rs` - `ThemeConfig` / `EditorConfig` / `SidebarConfig`，含 `ThemeMode`)
- [x] Clippy 清理 (消除 redundant_closure / semicolon / dead_code 等警告)
- [ ] 运行时可变 + 变更通知（由 echo-app 层 GPUI `Model<Config>` + `cx.notify()` 实现，echo-core 仅提供数据模型）

### 日志系统

- [x] 定义日志模块 (`log/mod.rs` - `init()` / `init_from_config()` 返回 `LogGuard`，基于 `tracing`)
- [x] 日志配置 (`schema.rs` - `LogLevel` 枚举 / `LogConfig` 结构体 / `RotationKind` 轮转策略)
- [x] `LogLevel -> tracing` LevelFilter 转换 (`impl From<LogLevel>`)
- [x] 日志错误类型 (`error.rs` - `LogError` / `LogResult`，`impl From<LogError> for EchoError`)
- [x] 控制台输出 + 文件输出（含父目录自动创建）
- [x] 日志文件轮转 (`RotationKind` - daily/hourly/minutely/never，按日期分割；按大小暂不支持)
- [x] 日志级别热更新 (`LogGuard::set_level()` + `reload::Layer`，运行时动态调整，无需重启)
- [x] `log` 门面桥接 (`tracing_log::LogTracer`，`log::info!` 等宏无需改动即转发到 tracing)

### 测试与基准

- [x] 单元测试（config / error / log / id 模块，56 passed）
- [x] 配置基准 (`config_bench` - 9 项)：序列化 / 反序列化 / 合并 / 校验 / add_recent / 保存 / 磁盘加载 / 日志级别转换 / 默认路径

### UI 层 (echo-app)

> 详见 `doc/echo-app.md`。echo-core 仅提供配置与日志能力，UI 由 echo-app 实现。

## 当前架构

```
echo-core/src/
├── lib.rs              # 模块导出
├── error.rs            # 统一错误类型 (EchoError 11 种变体 + 便捷构造函数 + 向后兼容别名)
├── id.rs               # ID 类型系统 (Id<T> / NodeId / BlockId / FileId / VaultId / Timestamp)
├── log/                # 日志模块
│   └── mod.rs          # 日志初始化 (init / init_from_config / LogGuard)
└── config/
    ├── mod.rs          # 统一入口 (load_config / save_config_to_default)
    ├── schema.rs       # 数据结构 (ConfigData / VaultConfig / VaultEntry / LogConfig / LogLevel / RotationKind / ThemeConfig / EditorConfig / SidebarConfig)
    ├── defaults.rs     # 默认值函数
    ├── layers.rs       # 分层加载 + 递归合并
    ├── validate.rs     # 语义校验 (vault + log + editor + theme)
    └── persist.rs      # 读写 TOML + 跨平台路径 + CachedConfig
```

## 统一错误类型

所有 crate 的错误统一到 `EchoError`，通过 `From` trait 实现自动转换。

### EchoError 变体

```
EchoError (统一错误类型)
├── Io (std::io::Error)           ← 文件/网络 IO 错误
├── VaultNotFound { path }        ← vault 路径不存在
├── ConfigNotFound { path }       ← 配置文件不存在
├── ConfigParse { message }       ← TOML 解析失败
├── InvalidPath { path }          ← 无效路径
├── VersionMismatch { expected, actual }  ← 版本不匹配
├── InvalidId { message }         ← ID 格式错误
├── ConfigValidation { message }  ← 配置语义校验失败
├── Markdown { message }          ← Markdown 解析/序列化错误
├── VaultInit { message }         ← vault watcher 初始化失败
└── VaultNotify { message }       ← 文件系统通知错误 (notify::Error 作为字符串)
```

### 便捷构造函数

| 函数 | 说明 |
|------|------|
| `EchoError::markdown(msg)` | 创建 Markdown 错误 |
| `EchoError::vault_init(msg)` | 创建 vault 初始化错误 |
| `EchoError::vault_notify(msg)` | 创建 vault 通知错误 |
| `EchoError::config_validation(msg)` | 创建配置校验错误 |

### 向后兼容类型别名

| 类型别名 | 实际类型 | 说明 |
|---------|---------|------|
| `ConfigError` | `EchoError` | 配置错误（已弃用，直接使用 `EchoError`） |
| `MarkdownError` | `EchoError` | Markdown 错误（已弃用，直接使用 `EchoError`） |
| `VaultError` | `EchoError` | Vault 错误（已弃用，直接使用 `EchoError`） |
| `ConfigResult<T>` | `EchoResult<T>` | 配置结果别名 |
| `LogResult<T>` | `EchoResult<T>` | 日志结果别名 |
| `MarkdownResult<T>` | `EchoResult<T>` | Markdown 结果别名 |
| `VaultResult<T>` | `EchoResult<T>` | Vault 结果别名 |

### LogError (保留独立枚举)

```
LogError (日志细粒度错误)
├── Init (String)         → 转换为 EchoError::ConfigParse
└── File (std::io::Error) → 转换为 EchoError::Io
```

## ID 类型系统

使用 Rust 的类型系统实现编译时区分的 ID 类型，避免运行时混淆。

### 类型定义

```
Id<T> (newtype 包装 Uuid)
├── NodeId    ← 文档节点 ID
├── BlockId   ← 块 ID
├── FileId    ← 文件 ID
└── VaultId   ← Vault ID

Timestamp    ← 毫秒时间戳 (u64)
```

### 公共 Trait

所有 ID 类型自动实现以下 trait：

| Trait | 用途 |
|-------|------|
| `Copy` | 值语义，无需 clone |
| `Eq` + `Hash` | 可用作 HashMap/HashSet 键 |
| `Debug` | 调试输出（含类型名，如 `NodeId(abc-123)`） |
| `Display` | 格式化输出（Uuid 字符串） |
| `Serialize` / `Deserialize` | serde 序列化支持 |

### 使用示例

```rust
use echo_core::{NodeId, BlockId, VaultId, Timestamp};

// 类型安全：NodeId 和 BlockId 不可混用
let node = NodeId::new();
let block = BlockId::new();

// 编译错误：类型不匹配
// let wrong: BlockId = node;

// Timestamp
let ts = Timestamp::now();
let zero = Timestamp::zero();
```

## 配置加载流程

```
defaults -> ~/.config/echo/config.toml -> <workspace>/.echo.toml -> 运行时覆盖
   ↓              ↓                           ↓
   └─────────────-> merge() ←─────────────────┘
                      ↓
                  validate()
                      ↓
               ConfigData (返回)
```

## 日志系统

基于 `tracing` 实现，通过 `tracing_log::LogTracer` 桥接 `log` 门面，
因此 `log::info!` 等宏可直接使用。支持控制台输出、文件输出（按日期轮转）
与运行时级别热更新。

### 日志初始化

`init` 返回 `LogGuard`，**必须持有到程序结束**，否则非阻塞文件写入可能丢失日志。
通过 `LogGuard::set_level` 可运行时改变日志级别（热更新）。

```rust
use echo_core::log;

// 从 ConfigData 初始化，持有 guard
let _guard = log::init_from_config(&config)?;

// 或直接传入 LogConfig
let _guard = log::init(&config.log)?;

// 运行时热更新日志级别
_guard.set_level(echo_core::config::LogLevel::Debug)?;
```

### 日志配置 (TOML)

```toml
[log]
level = "info"          # error / warn / info / debug / trace
console_output = true   # 是否输出到控制台
file_output = false     # 是否输出到文件
file_path = "echo.log"  # 日志文件路径（可选，默认 echo.log）
rotation = "never"      # daily / hourly / minutely / never（仅 file_output=true 生效）
```

### 配置组 (TOML)

```toml
[theme]
mode = "dark"           # light / dark / auto
# font_family = "monospace"
# font_size = 14.0      # 若设置必须 > 0

[editor]
tab_size = 4            # 必须 > 0
show_line_numbers = true

[sidebar]
width = 240.0
collapsed = false
```

## 公共 API

### 核心函数

| 函数 | 签名 | 说明 |
|------|------|------|
| `load_config` | `fn(Option<&Path>) -> ConfigResult<ConfigData>` | 加载完整配置（加载→合并→校验） |
| `save_config_to_default` | `fn(&ConfigData) -> ConfigResult<()>` | 保存配置到默认路径 |
| `save_config` | `fn(&ConfigData, &Path) -> ConfigResult<()>` | 配置序列化 + 文件写入 |
| `load_config_from_path` | `fn(&Path) -> ConfigResult<ConfigData>` | 文件读取 + 配置反序列化 |
| `default_config_path` | `fn() -> PathBuf` | 获取默认配置文件路径 |

### 配置模型

| 类型 | 说明 |
|------|------|
| `ConfigData` | 顶层配置结构（vault / log / theme / editor / sidebar / extra） |
| `VaultConfig` | 仓库配置（path / auto_index / recent） |
| `VaultEntry` | 最近使用的仓库条目（path / name / last_opened） |
| `LogConfig` | 日志配置（level / console_output / file_output / file_path / rotation） |
| `LogLevel` | 日志级别（Error / Warn / Info / Debug / Trace） |
| `RotationKind` | 日志轮转策略（Daily / Hourly / Minutely / Never） |
| `ThemeConfig` | 主题配置（mode / font_family / font_size） |
| `ThemeMode` | 主题模式（Light / Dark / Auto） |
| `EditorConfig` | 编辑器配置（tab_size / show_line_numbers） |
| `SidebarConfig` | 侧边栏配置（width / collapsed） |

### ID 类型

| 类型 | 说明 |
|------|------|
| `NodeId` | 文档节点 ID (Uuid newtype) |
| `BlockId` | 块 ID (Uuid newtype) |
| `FileId` | 文件 ID (Uuid newtype) |
| `VaultId` | Vault ID (Uuid newtype) |
| `Timestamp` | 毫秒时间戳 (u64 包装) |
| `now()` | 获取当前时间戳 |
| `zero_timestamp()` | 获取零值时间戳 |

### 分层加载

| 类型 | 说明 |
|------|------|
| `Layers` | 配置层（global / workspace） |
| `load_layers` | `fn(Option<&Path>) -> ConfigResult<Layers>` | 加载所有配置层 |
| `Layers::merge` | `fn(&self) -> ConfigResult<ConfigData>` | 递归合并配置层 |

### 缓存优化

| 类型 | 说明 |
|------|------|
| `CachedConfig` | 带缓存的配置包装（inner + cached_toml） |
| `CachedConfig::to_toml` | `fn(&self) -> ConfigResult<String>` | 返回缓存的 TOML 字符串（若脏则重新序列化） |
| `CachedConfig::mark_dirty` | `fn(&self)` | 标记缓存为脏 |
| `CachedConfig::save` | `fn(&self, &Path) -> ConfigResult<()>` | 保存到文件 |

### 校验

| 函数 | 说明 |
|------|------|
| `validate` | `fn(&ConfigData) -> ConfigResult<()>` | 语义校验（vault 路径 / 日志路径 / tab_size / font_size） |

### 日志

| 函数 | 说明 |
|------|------|
| `log::init` | `fn(&LogConfig) -> LogResult<LogGuard>` | 从配置初始化日志 |
| `log::init_from_config` | `fn(&ConfigData) -> LogResult<LogGuard>` | 从 ConfigData 初始化日志 |
| `LogGuard::set_level` | `fn(&self, LogLevel) -> LogResult<()>` | 运行时热更新日志级别 |

### 错误类型

| 类型 | 说明 |
|------|------|
| `EchoError` | 核心错误枚举（11 种变体） |
| `LogError` | 日志错误枚举（保留独立枚举用于细粒度处理） |
| `EchoResult<T>` | 核心结果别名 |
| `ConfigError` | **已弃用**：`EchoError` 类型别名 |
| `ConfigResult<T>` | **已弃用**：`EchoResult<T>` 类型别名 |
| `LogResult<T>` | **已弃用**：`EchoResult<T>` 类型别名 |
| `MarkdownResult<T>` | **已弃用**：`EchoResult<T>` 类型别名 |
| `VaultResult<T>` | **已弃用**：`EchoResult<T>` 类型别名 |

## 性能基准

| 基准测试 | 耗时 | 说明 |
|---------|------|------|
| `serialize_config` | 340.48 µs | TOML 序列化 |
| `deserialize_config` | 269.56 µs | TOML 反序列化 |
| `merge_configs` | 10.52 µs | 配置合并 |
| `serialize_cached_hit` | 146.77 ns | 缓存命中序列化（比 serialize 快 2317x） |
| `validate_config` | 4.62 ns | 配置校验 |
| `add_recent` | 27.41 µs | 添加 100 个最近仓库 |
| `save_config` | 833 µs | 序列化 + 文件写入 |
| `load_config_from_path` | 321.89 µs | 文件读取 + 反序列化 |
| `log_level_to_filter` | 1.52 ns | 日志级别转换 |
| `default_config_path` | 665.57 ns | 默认路径计算 |

## 注意事项

- `EchoError` 是整个 workspace 的统一错误类型，所有 crate 的错误都通过 `From` trait 转换到 `EchoError`
- 外部 crate 错误（如 `notify::Error`）以字符串形式表示，避免 echo-core 依赖外部 crate
- `VaultError` 和 `MarkdownError` 是 `EchoError` 的类型别名，保留用于向后兼容
- `LogError` 保留为独立枚举，用于日志初始化/文件操作的细粒度错误处理
- `CachedConfig` 使用 `Cell<Option<String>>` 实现内部可变性缓存，线程安全由调用方保证
- ID 类型使用编译时泛型区分，`NodeId` 和 `BlockId` 不可混用，避免运行时类型错误
