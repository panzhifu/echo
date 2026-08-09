# echo-vault 开发文档

## 任务清单

### 文件监控

- [x] 定义错误类型 (`watcher.rs` - `VaultError` → `echo_core::EchoError` 类型别名)
- [x] 定义事件类型 (`watcher.rs` - `VaultEvent` / `Create` / `Modify` / `Delete` / `Rename`)
- [x] 实现监控器 (`watcher.rs` - `VaultWatcher` / `new()` / `with_paths()` / `watch()`)
- [x] 后台线程 + channel 事件传递
- [x] 路径不存在校验
- [x] 重命名事件成对路径处理
- [x] 编译验证通过 (`cargo build -p echo-vault`)
- [x] 单元测试通过 (`cargo test -p echo-vault`，19 passed)
- [x] 可停止监控 (`WatchGuard::stop()` / `Drop`)
- [x] 支持忽略模式 (`filter.rs` - `IgnoreFilter`，gitignore 风格)
- [x] 支持多路径监控 (`VaultWatcher::with_paths`)
- [x] 事件防抖 (`debounce.rs` - `run_debouncer`)
- [x] 过滤器缓存 (`filter_cache.rs` - `get_or_create()`)
- [x] 错误统一 (VaultError → echo_core::EchoError)
- [x] Clippy 清理 (消除 unused_import / dead_code / doc_markdown 等警告)

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
    ├── watcher.rs        # 文件监控实现 + 错误辅助函数
    ├── filter.rs         # gitignore 风格忽略过滤
    ├── filter_cache.rs   # 过滤器编译缓存
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
| echo-core | path | 统一错误类型 (EchoError) |

### 监控流程

```
VaultWatcher::new(path) / with_paths(paths)
        │  可选: .ignore_patterns(..) / .debounce(..)
        ▼
    watch()
        │
        ├── 路径存在? ──No──▶ Err(VaultNotFound)
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

## 公共 API

### 监控器

| 类型 | 说明 |
|------|------|
| `VaultWatcher` | 文件监控器（builder 模式） |
| `VaultWatcher::new` | `fn(&str) -> Self` | 创建单路径监控器 |
| `VaultWatcher::with_paths` | `fn(Vec<PathBuf>) -> Self` | 创建多路径监控器 |
| `VaultWatcher::ignore_patterns` | `fn(Vec<String>) -> Self` | 设置忽略模式 |
| `VaultWatcher::debounce` | `fn(Duration) -> Self` | 设置防抖时间 |
| `VaultWatcher::watch` | `fn(self) -> VaultResult<(Receiver<VaultEvent>, WatchGuard)>` | 开始监控 |

### 事件类型

| 类型 | 说明 |
|------|------|
| `VaultEvent` | 文件事件枚举（Create / Modify / Delete / Rename） |
| `VaultEvent::path` | `fn(&self) &Path` | 获取事件路径 |
| `WatchGuard` | 监控守护（调用 stop() 或 drop 停止监控） |

### 忽略过滤

| 类型 | 说明 |
|------|------|
| `IgnoreFilter` | gitignore 风格过滤器 |
| `IgnoreFilter::new` | `fn(&Path, &[String]) -> VaultResult<Self>` | 创建过滤器 |
| `IgnoreFilter::is_ignored` | `fn(&self, &Path) -> bool` | 检查路径是否被忽略 |

### 缓存优化

| 函数 | 说明 |
|------|------|
| `get_or_create_filter` | `fn(PathBuf, Vec<String>) -> Result<IgnoreFilter, VaultError>` | 从缓存获取或创建过滤器 |

### 错误类型

| 类型 | 说明 |
|------|------|
| `VaultError` | **已弃用**：使用 [`echo_core::EchoError`] 代替，保留以维持向后兼容 |
| `VaultResult<T>` | **已弃用**：`echo_core::EchoResult<T>` 类型别名 |

### 错误辅助函数 (内部)

| 函数 | 说明 |
|------|------|
| `notify_error(&notify::Error) -> EchoError` | 将 notify::Error 转换为 EchoError::VaultNotify |
| `vault_init_error(msg) -> EchoError` | 创建 EchoError::VaultInit |
| `path_not_found(path) -> EchoError` | 创建 EchoError::VaultNotFound |

## 错误层级

```
EchoError (统一错误类型 — 来自 echo-core)
├── VaultNotFound { path }    ← 监控路径不存在
├── VaultInit { message }     ← watcher 初始化失败
└── VaultNotify { message }   ← 底层 notify 错误

向后兼容类型别名：
├── VaultError = EchoError
└── VaultResult<T> = EchoResult<T>
```

## 性能基准

| 基准测试 | 耗时 | 说明 |
|---------|------|------|
| `ignore_filter_new_50` | 75.11 µs | 编译 50 个忽略模式 |
| `ignore_filter_new_3` | 4.42 µs | 编译 3 个忽略模式 |
| `ignore_filter_cached_hit` | 5.28 µs | 缓存命中（比 new_50 快 14.2x） |
| `ignore_filter_match_ignored` | 337 ns | 匹配被忽略的路径 |
| `ignore_filter_match_not_ignored` | 422 ns | 匹配未被忽略的路径 |
| `ignore_filter_match_nested_dir` | 504 ns | 匹配嵌套目录 |
| `event_path_create` | 601 ps | 提取 Create 事件路径 |
| `event_path_rename` | 587 ps | 提取 Rename 事件路径 |
| `watcher_new_single` | 91.40 ns | 创建单路径监控器 |
| `watcher_with_paths_10` | 424.17 ns | 创建 10 路径监控器 |
| `watcher_builder_full` | 898.80 ns | 完整 builder 链式调用 |

## 注意事项

- `watch()` 启动两个后台线程：notify 监控线程（持有 watcher，保持存活）和防抖线程
- 防抖线程按路径去重，在 `debounce` 时间窗口内同一路径的多个事件只保留最后一个；设为零禁用防抖
- 忽略模式基于 `ignore` crate，支持标准 gitignore 语法（`*`、`**`、`/` 锚定、`!` 取反等）
- 通过 [`WatchGuard`] 停止监控：调用 `stop()` 或 drop 守护即可；重复调用安全
- `notify` crate 在 Linux 上使用 inotify，Windows 上使用 ReadDirectoryChangesW，macOS 上使用 FSEvents
- 重命名事件在某些平台上可能表现为 Create + Delete 而非单个 Rename
- 错误类型已统一到 `echo_core::EchoError`，`VaultError` 为向后兼容的类型别名
- 过滤器编译结果通过 `FILTER_CACHE` 全局缓存，相同 `(root, patterns)` 组合复用已编译的 `IgnoreFilter`
