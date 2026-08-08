# echo-core 开发文档

## 任务清单

### 配置系统

- [x] 定义错误文件 (`error.rs` — `EchoError` / `ConfigError` / `EchoResult` / `ConfigResult`)
- [x] 定义配置 schema (`schema.rs` — `ConfigData` / `VaultConfig` / `VaultEntry`，vault-only)
- [x] 定义默认值 (`defaults.rs` — `default_true()`)
- [x] 分层加载 (`layers.rs` — `Layers` / `merge()` / `load_layers()`，递归合并 TOML)
- [x] 持久化 (`persist.rs` — `save_config` / `load_config_from_path` / `default_config_path`，跨平台路径)
- [x] 语义校验 (`validate.rs` — vault 路径非空校验)
- [x] 统一入口 (`mod.rs` — `load_config()` 串联加载→合并→校验)
- [ ] 运行时可变 + 变更通知（GPUI `Model<Config>` + `cx.notify()`）
- [ ] 支持 theme/editor/sidebar 配置组

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
├── error.rs            # EchoError + ConfigError + 结果类型
└── config/
    ├── mod.rs          # 统一入口 (load_config / save_config_to_default)
    ├── schema.rs       # 数据结构 (ConfigData / VaultConfig / VaultEntry)
    ├── defaults.rs     # 默认值函数
    ├── layers.rs       # 分层加载 + 递归合并
    ├── validate.rs     # 语义校验
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
