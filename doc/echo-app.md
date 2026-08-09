# echo-app 开发文档

## 任务清单

### 应用框架

- [x] 应用入口 (`lib.rs` - `run()` 函数)
- [x] GPUI 窗口初始化 (900x600 居中窗口)
- [x] 配置加载与日志初始化
- [x] 响应式界面切换 (VaultManagerView ↔ WorkspaceView)
- [x] 内置图标资源注册

### 应用逻辑

- [x] 定义 `AppState` 枚举 (`app_logic.rs` - `NoVault` / `VaultLoaded`)
- [x] 实现 `apply_selection()` (选择/添加 vault)
- [x] 实现配置变更监听与界面切换

### 界面组件

- [x] 仓库管理界面 (`screens/vault_manager/`)
- [x] 工作区界面 (`screens/workspace/`)
- [x] 文件树面板 (`screens/workspace/panels/file_tree.rs`)
- [ ] 自定义标题栏
- [ ] 底部状态栏
- [ ] 仓库管理设置界面

### 测试

- [x] 单元测试 (app_logic 模块，9 passed)
- [ ] VaultWatcher 集成测试

## 当前架构

```
echo-app/
├── Cargo.toml            # 依赖: gpui, gpui-component, gpui-component-assets, echo-core
├── src/
│   ├── main.rs           # 可执行入口 (调用 lib::run)
│   ├── lib.rs            # 应用入口 (GPUI 初始化)
│   ├── app.rs            # EchoApp 主结构体
│   ├── app_logic.rs      # 应用状态机 (AppState + apply_selection)
│   └── screens/
│       ├── vault_manager/  # 仓库选择界面
│       └── workspace/      # 工作区界面 (Dock 布局)
│           └── panels/
│               └── file_tree.rs  # 文件树面板
└── examples/             # 示例 (若有)
```

## 技术实现

### 依赖选择

| 依赖 | 用途 |
|------|------|
| gpui | Zed 编辑器的 UI 框架 |
| gpui-component | GPUI 组件库 (ActiveTheme / TitleBar / Root) |
| gpui-component-assets | 内置图标资源 |
| echo-core | 配置加载、日志初始化 |

### 启动流程

```
run()
  │
  ├── echo_core::config::load_config(None)  ── 加载配置
  │
  ├── echo_core::log::init(&config.log)      ── 初始化日志
  │
  ├── gpui::application().run(cx => {
  │       gpui_component::init(cx)            ── 注册组件
  │
  │       open_window(900x600) => EchoApp::new(cx, config)
  │           │
  │           ├── 有 vault → WorkspaceView
  │           └── 无 vault → VaultManagerView
  │   })
```

### 状态机

```
AppState
├── NoVault      → 显示 VaultManagerView
└── VaultLoaded  → 显示 WorkspaceView
```

## 公共 API

### 应用入口

| 函数 | 签名 | 说明 |
|------|------|------|
| `run` | `fn()` | 应用主入口，加载配置并启动 GPUI |

### 应用结构体

| 类型 | 说明 |
|------|------|
| `EchoApp` | 应用主结构体，持有配置实体和界面视图 |

### 应用逻辑

| 类型/函数 | 说明 |
|---------|------|
| `AppState` | 应用状态枚举 (NoVault / VaultLoaded) |
| `apply_selection` | 选择/添加 vault 并更新配置 |

### 文件树面板

| 函数 | 说明 |
|------|------|
| `build_items` | 构建文件树项列表 (递归遍历，跳过隐藏目录，文件夹优先) |

## 使用示例

```rust
// main.rs
fn main() {
    echo_app::run();
}
```

## 注意事项

- `run()` 必须持有 `LogGuard` 直到程序结束，否则非阻塞文件写入可能丢失日志
- 配置通过 GPUI `Entity<ConfigData>` 持有，变更自动触发界面重绘
- `VaultManagerView` 写入配置后，`EchoApp` 通过观察 `config` 实体自动切换到 `WorkspaceView`
- `gpui_component::init(cx)` 必须在任何 GPUI Component 功能之前调用
