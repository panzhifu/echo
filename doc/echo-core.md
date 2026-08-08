# echo-core 开发文档

## 任务清单

### 配置系统

- [x] 定义错误文件 (`error.rs` — `EchoError` / `ConfigError` / `LogError` / 结果类型)
- [x] 定义配置 schema (`schema.rs` — `ConfigData` / `VaultConfig` / `VaultEntry` / `LogConfig` / `LogLevel`)
- [x] 定义默认值 (`defaults.rs` — `default_true()`)
- [x] 分层加载 (`layers.rs` — `Layers` / `merge()` / `load_layers()`，递归合并 TOML)
- [x] 持久化 (`persist.rs` — `save_config` / `load_config_from_path` / `default_config_path`，跨平台路径)
- [x] 语义校验 (`validate.rs` — vault 路径非空校验 + 日志文件路径校验)
- [x] 统一入口 (`mod.rs` — `load_config()` 串联加载→合并→校验)
- [ ] 运行时可变 + 变更通知（GPUI `Model<Config>` + `cx.notify()`）
- [ ] 支持 theme/editor/sidebar 配置组

### 日志系统

- [x] 定义日志模块 (`log/mod.rs` — `init()` / `init_from_config()`，基于 `log` + `fern`)
- [x] 日志配置 (`schema.rs` — `LogLevel` 枚举 / `LogConfig` 结构体)
- [x] 日志错误类型 (`error.rs` — `LogError` / `LogResult`，可转换为 `EchoError`)
- [ ] 日志文件轮转（按日期/大小分割）
- [ ] 日志级别热更新（运行时动态调整）

### UI 层 (echo-app)

- [ ] 自定义标题栏（左侧 `SidebarCollapsible` 图标 + 右侧静态图标）
- [ ] 底部状态栏置于主面板下方
- [ ] Welcome 视图（文件夹选择器 + 最近仓库列表）
- [ ] 仓库管理界面（settings/vault）
- [ ] 启动流程：无配置 → Welcome，有配置 → 主界面

## 当前架构

```
echo-core/src/
├── lib.rs              # 模块导出
├── error.rs            # EchoError + ConfigError + LogError + 结果类型
├── log/                # 日志模块
│   └── mod.rs          # 日志初始化 (init / init_from_config)
└── config/
    ├── mod.rs          # 统一入口 (load_config / save_config_to_default)
    ├── schema.rs       # 数据结构 (ConfigData / VaultConfig / VaultEntry / LogConfig / LogLevel)
    ├── defaults.rs     # 默认值函数
    ├── layers.rs       # 分层加载 + 递归合并
    ├── validate.rs     # 语义校验 (vault + log)
    └── persist.rs      # 读写 TOML + 跨平台路径
```

## 配置加载流程

```
defaults → ~/.config/echo/config.toml → <workspace>/.echo.toml → 运行时覆盖
   ↓              ↓                           ↓
   └─────────────→ merge() ←─────────────────┘
                      ↓
                  validate()
                      ↓
               ConfigData (返回)
```

## 日志系统

### 日志初始化

```rust
use echo_core::log;

// 从 ConfigData 初始化
log::init_from_config(&config)?;

// 或直接传入 LogConfig
log::init(&config.log)?;
```

### 日志配置 (TOML)

```toml
[log]
level = "info"          # error / warn / info / debug / trace
console_output = true   # 是否输出到控制台
file_output = false      # 是否输出到文件
file_path = "echo.log"  # 日志文件路径（可选，默认 echo.log）
```

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
├── Init (String)  → 转换为 EchoError::ConfigParse
└── File (std::io::Error) → 转换为 EchoError::Io
```
