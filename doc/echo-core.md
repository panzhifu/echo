# echo-core 开发文档

## 任务清单

### 配置系统

- [x] 定义错误类型 (`error.rs` - `EchoError` / `ConfigError` / `LogError` + `EchoResult` / `ConfigResult` / `LogResult`)
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

- [x] 单元测试（config / error / log 模块，39 passed）
- [x] 配置基准 (`config_bench` - 9 项)：序列化 / 反序列化 / 合并 / 校验 / add_recent / 保存 / 磁盘加载 / 日志级别转换 / 默认路径

### UI 层 (echo-app)

> 详见 `doc/echo-app.md`（待建）。echo-core 仅提供配置与日志能力，UI 由 echo-app 实现。

- [ ] 自定义标题栏（左侧 `SidebarCollapsible` 图标 + 右侧静态图标）
- [ ] 底部状态栏置于主面板下方
- [ ] Welcome 视图（文件夹选择器 + 最近仓库列表）
- [ ] 仓库管理界面（settings/vault）
- [ ] 启动流程：无配置 -> Welcome，有配置 -> 主界面

## 当前架构

```
echo-core/src/
├── lib.rs              # 模块导出
├── error.rs            # EchoError + ConfigError + LogError + 结果类型
├── log/                # 日志模块
│   └── mod.rs          # 日志初始化 (init / init_from_config / LogGuard)
└── config/
    ├── mod.rs          # 统一入口 (load_config / save_config_to_default)
    ├── schema.rs       # 数据结构 (ConfigData / VaultConfig / VaultEntry / LogConfig / LogLevel / RotationKind / ThemeConfig / EditorConfig / SidebarConfig)
    ├── defaults.rs     # 默认值函数
    ├── layers.rs       # 分层加载 + 递归合并
    ├── validate.rs     # 语义校验 (vault + log + editor + theme)
    └── persist.rs      # 读写 TOML + 跨平台路径
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
| `EchoError` | 核心错误枚举 |
| `ConfigError` | 配置错误枚举 |
| `LogError` | 日志错误枚举 |
| `EchoResult<T>` | 核心结果别名 |
| `ConfigResult<T>` | 配置结果别名 |
| `LogResult<T>` | 日志结果别名 |

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

### 错误层级

```
EchoError (核心错误)
├── Io (std::io::Error)
├── VaultNotFound
├── ConfigNotFound
├── ConfigParse
├── InvalidPath
├── VersionMismatch
└── InvalidId

LogError (日志错误)
├── Init (String)  -> 转换为 EchoError::ConfigParse
└── File (std::io::Error) -> 转换为 EchoError::Io
```
