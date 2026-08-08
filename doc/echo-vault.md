# echo-vault 开发文档

## 任务清单

### 文件监控

- [x] 定义错误类型 (`watcher.rs` - `VaultError` / `PathNotFound` / `Init` / `Notify`)
- [x] 定义事件类型 (`watcher.rs` - `VaultEvent` / `Create` / `Modify` / `Delete` / `Rename`)
- [x] 实现监控器 (`watcher.rs` - `VaultWatcher` / `new()` / `with_paths()` / `watch()`)
- [x] 后台线程 + channel 事件传递
- [x] 路径不存在校验
- [x] 重命名事件成对路径处理
- [x] 编译验证通过 (`cargo build -p echo-vault`)
- [x] 单元测试通过 (`cargo test -p echo-vault`，17 passed)
- [x] 可停止监控 (`WatchGuard::stop()` / `Drop`)
- [x] 支持忽略模式 (`filter.rs` - `IgnoreFilter`，gitignore 风格)
- [x] 支持多路径监控 (`VaultWatcher::with_paths`)
- [x] 事件防抖 (`debounce.rs` - `run_debouncer`)

### 与 echo-app 集成

- [ ] 在 app 中集成 VaultWatcher
- [ ] 启动时自动监控 vault 路径
- [ ] 文件变化时触发 UI 更新

## 当前架构

```
echo-vault/
├── Cargo.toml            # 依赖: notify 6, thiserror, log, ignore, echo-core
├── benches/
│   └── watcher_bench.rs  # criterion 基准测试
└── src/
    ├── lib.rs            # 模块入口，导出公共 API
    ├── watcher.rs        # 文件监控实现
    ├── filter.rs         # gitignore 风格忽略过滤
    └── debounce.rs       # 事件防抖
```

## 技术实现

### 依赖选择

| 依赖 | 版本 | 用途 |
|------|------|------|
| notify | 6 | 跨平台文件系统事件监听 |
| thiserror | 2 | 结构化错误类型 |
| log | 0.4 | 日志门面 |
| ignore | 0.4 | gitignore 风格模式匹配 |
| echo-core | path | 共享错误类型 |

### 监控流程

```
VaultWatcher::new(path) / with_paths(paths)
        │  可选: .ignore_patterns(..) / .debounce(..)
        ▼
    watch()
        │
        ├── 路径存在? ──No──▶ Err(PathNotFound)
        │
        ▼
    notify::recommended_watcher(callback)   ── 过滤忽略模式
        │
        ▼
    watcher.watch(path, RecursiveMode::Recursive)
        │
        ├──▶ notify 线程 (持有 watcher，保持存活)
        │
        └──▶ 防抖线程 (按路径去重，窗口内保留最后一个)
                │
                ▼
        Ok((Receiver<VaultEvent>, WatchGuard))
```

### 事件类型

| VaultEvent | 触发条件 | 来源 |
|------------|----------|------|
| `Create { path }` | 文件/目录创建 | `EventKind::Create` |
| `Modify { path }` | 文件内容修改 | `EventKind::Modify` |
| `Delete { path }` | 文件/目录删除 | `EventKind::Remove` |
| `Rename { from, to }` | 文件/目录重命名 | `EventKind::Modify(Name)` |

## 使用示例

```rust
use echo_vault::VaultWatcher;
use std::time::Duration;

let watcher = VaultWatcher::new("/path/to/vault")
    .ignore_patterns(vec!["*.tmp".to_string(), ".git/".to_string()])
    .debounce(Duration::from_millis(200));

let (events, guard) = watcher.watch().expect("failed to start watcher");

for event in events {
    match event {
        echo_vault::VaultEvent::Create { path } => println!("created: {}", path.display()),
        echo_vault::VaultEvent::Modify { path } => println!("modified: {}", path.display()),
        echo_vault::VaultEvent::Delete { path } => println!("deleted: {}", path.display()),
        echo_vault::VaultEvent::Rename { from, to } => println!("renamed: {} -> {}", from.display(), to.display()),
    }
}

// guard 被 drop 时自动停止监控
drop(guard);
```

## 错误层级

```
VaultError (vault 监控错误)
├── PathNotFound { path }     -> 监控路径不存在
├── Init(String)              -> watcher 初始化失败
└── Notify(notify::Error)     -> 底层 notify 错误
```

## 注意事项

- `watch()` 启动两个后台线程：notify 监控线程（持有 watcher，保持存活）和防抖线程
- 防抖线程按路径去重，在 `debounce` 时间窗口内同一路径的多个事件只保留最后一个；设为零禁用防抖
- 忽略模式基于 `ignore` crate，支持标准 gitignore 语法（`*`、`**`、`/` 锚定、`!` 取反等）
- 通过 [`WatchGuard`] 停止监控：调用 `stop()` 或 drop 守护即可；重复调用安全
- `notify` crate 在 Linux 上使用 inotify，Windows 上使用 ReadDirectoryChangesW，macOS 上使用 FSEvents
- 重命名事件在某些平台上可能表现为 Create + Delete 而非单个 Rename
