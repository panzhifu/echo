//! 文件夹监控实现。
//!
//! 支持：
//! - 多路径递归监控
//! - gitignore 风格的忽略模式过滤
//! - 事件防抖（合并短时间内的重复事件）
//! - 可通过 [`WatchGuard`] 停止监控

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::filter::IgnoreFilter;

/// 默认防抖时间。
const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(100);

/// 监控线程轮询停止标志的间隔。
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Vault 监控错误类型。
///
/// **已弃用**：使用 [`echo_core::EchoError`] 代替。
/// 保留此类型别名以维持向后兼容性。
pub type VaultError = echo_core::EchoError;

/// Vault 监控结果类型。
#[allow(dead_code)]
pub type VaultResult<T> = echo_core::EchoResult<T>;

/// 辅助函数：将 `notify::Error` 转换为 `EchoError`。
///
/// 由于 echo-core 不依赖 notify crate，这里手动转换。
#[allow(dead_code)]
pub(crate) fn notify_error(err: &notify::Error) -> echo_core::EchoError {
    echo_core::EchoError::vault_notify(err.to_string())
}

/// 辅助函数：创建 vault 初始化错误。
pub(crate) fn vault_init_error(msg: impl Into<String>) -> echo_core::EchoError {
    echo_core::EchoError::vault_init(msg)
}

/// 辅助函数：创建路径不存在错误。
pub(crate) fn path_not_found(path: PathBuf) -> echo_core::EchoError {
    echo_core::EchoError::VaultNotFound { path }
}

/// 文件夹变化事件。
#[derive(Clone, Debug)]
pub enum VaultEvent {
    /// 文件或目录被创建。
    Create { path: PathBuf },
    /// 文件或目录被修改。
    Modify { path: PathBuf },
    /// 文件或目录被删除。
    Delete { path: PathBuf },
    /// 文件或目录被重命名。
    Rename { from: PathBuf, to: PathBuf },
}

impl VaultEvent {
    /// 返回事件的主路径。
    ///
    /// - [`VaultEvent::Create`] / [`VaultEvent::Modify`] / [`VaultEvent::Delete`]: 返回事件路径
    /// - [`VaultEvent::Rename`][]: 返回源路径 (`from`)
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::Create { path } | Self::Modify { path } | Self::Delete { path } => path,
            Self::Rename { from, .. } => from,
        }
    }
}

/// 监控守护，用于停止监控。
///
/// 当 [`WatchGuard`] 被 drop 时自动停止监控，无需手动调用 [`stop`](Self::stop)。
pub struct WatchGuard {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl WatchGuard {
    /// 停止监控。
    ///
    /// 设置停止标志并等待监控线程退出。重复调用是安全的。
    pub fn stop(&mut self) {
        if self.handle.is_some() {
            self.stop.store(true, Ordering::SeqCst);
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }

    /// 检查监控是否仍在运行。
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Vault 文件夹监控器。
///
/// 支持多路径监控、gitignore 风格的忽略模式和事件防抖。
///
/// # 使用示例
///
/// ```ignore
/// use echo_vault::VaultWatcher;
/// use std::time::Duration;
///
/// let watcher = VaultWatcher::new("/path/to/vault")
///     .ignore_patterns(vec!["*.tmp".to_string(), ".git/".to_string()])
///     .debounce(Duration::from_millis(200));
///
/// let (events, guard) = watcher.watch().expect("failed to start watcher");
///
/// for event in events {
///     println!("event: {:?}", event);
/// }
/// ```
pub struct VaultWatcher {
    paths: Vec<PathBuf>,
    ignore_patterns: Vec<String>,
    debounce: Duration,
}

impl VaultWatcher {
    /// 创建一个新的 [`VaultWatcher`]，监控单个路径。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            paths: vec![path.into()],
            ignore_patterns: Vec::new(),
            debounce: DEFAULT_DEBOUNCE,
        }
    }

    /// 创建一个新的 [`VaultWatcher`]，监控多个路径。
    #[must_use]
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            ignore_patterns: Vec::new(),
            debounce: DEFAULT_DEBOUNCE,
        }
    }

    /// 设置 gitignore 风格的忽略模式。
    ///
    /// 匹配的模式对应的文件事件将被过滤掉。
    /// 根目录为第一个监控路径。
    #[must_use]
    pub fn ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    /// 设置事件防抖时间。
    ///
    /// 在此时间窗口内，同一路径的多个事件只保留最后一个。
    /// 设为零可禁用防抖（事件立即转发）。
    #[must_use]
    pub fn debounce(mut self, duration: Duration) -> Self {
        self.debounce = duration;
        self
    }

    /// 开始监控文件夹变化。
    ///
    /// 返回一个 [`Receiver`] 用于接收 [`VaultEvent`]，和一个 [`WatchGuard`] 用于停止监控。
    ///
    /// # Errors
    ///
    /// - [`EchoError::VaultNotFound`][]: 任一监控路径不存在
    /// - [`EchoError::VaultInit`]: watcher 初始化或添加监控路径失败
    pub fn watch(&self) -> Result<(Receiver<VaultEvent>, WatchGuard), VaultError> {
        if self.paths.is_empty() {
            return Err(vault_init_error("no paths to watch"));
        }

        // 验证所有路径
        for path in &self.paths {
            if !path.exists() {
                return Err(path_not_found(path.clone()));
            }
        }

        // 构建忽略过滤器
        let ignore_filter: Option<Arc<IgnoreFilter>> = if self.ignore_patterns.is_empty() {
            None
        } else {
            let root = self.paths[0].as_path();
            Some(Arc::new(IgnoreFilter::new(root, &self.ignore_patterns)?))
        };

        // 创建 channel
        let (raw_tx, raw_rx): (Sender<VaultEvent>, Receiver<VaultEvent>) = mpsc::channel();
        let (tx, rx): (Sender<VaultEvent>, Receiver<VaultEvent>) = mpsc::channel();

        // 创建 notify watcher
        let ignore_for_callback = ignore_filter.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    let vault_events = Self::convert_event(event);
                    for vault_event in vault_events {
                        // 应用忽略过滤器
                        if let Some(ref filter) = ignore_for_callback
                            && filter.is_ignored(vault_event.path())
                        {
                            continue;
                        }
                        if raw_tx.send(vault_event).is_err() {
                            break;
                        }
                    }
                },
                Err(e) => log::error!("watch error: {e}"),
            }
        })
        .map_err(|e| vault_init_error(e.to_string()))?;

        // 监控所有路径
        for path in &self.paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| vault_init_error(e.to_string()))?;
        }

        // 停止标志
        let stop = Arc::new(AtomicBool::new(false));

        // 监控线程（持有 notify watcher，保持其存活）
        let stop_for_watcher = Arc::clone(&stop);
        let watcher_handle = thread::spawn(move || {
            let _watcher = watcher;
            while !stop_for_watcher.load(Ordering::Relaxed) {
                thread::sleep(STOP_POLL_INTERVAL);
            }
            // watcher 在此被 drop，停止监控
        });

        // 防抖线程
        let stop_for_debounce = Arc::clone(&stop);
        let debounce = self.debounce;
        thread::spawn(move || {
            crate::debounce::run_debouncer(&raw_rx, &tx, debounce, &stop_for_debounce);
        });

        let guard = WatchGuard {
            stop,
            handle: Some(watcher_handle),
        };

        Ok((rx, guard))
    }

    /// 将 notify 事件转换为 [`VaultEvent`]。
    fn convert_event(event: Event) -> Vec<VaultEvent> {
        match event.kind {
            EventKind::Create(_) => event
                .paths
                .into_iter()
                .map(|p| VaultEvent::Create { path: p })
                .collect(),
            EventKind::Remove(_) => event
                .paths
                .into_iter()
                .map(|p| VaultEvent::Delete { path: p })
                .collect(),
            EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
                let mut paths = event.paths.into_iter();
                match (paths.next(), paths.next()) {
                    (Some(from), Some(to)) => vec![VaultEvent::Rename { from, to }],
                    _ => vec![],
                }
            },
            EventKind::Modify(_) => event
                .paths
                .into_iter()
                .map(|p| VaultEvent::Modify { path: p })
                .collect(),
            _ => vec![],
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{Duration, Instant};

    /// 生成唯一的临时目录路径。
    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("echo-vault-test-{}-{}", std::process::id(), name))
    }

    #[test]
    fn test_vault_watcher_creates_channel() {
        let dir = temp_dir("creates_channel");
        fs::create_dir_all(&dir).unwrap();

        let watcher = VaultWatcher::new(&dir);
        let (rx, mut guard) = watcher.watch().expect("watch should succeed");

        thread::sleep(Duration::from_millis(200));

        let test_file = dir.join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        let received = rx.recv_timeout(Duration::from_secs(2));
        assert!(received.is_ok(), "should receive an event");

        guard.stop();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_path_not_found_returns_error() {
        let watcher = VaultWatcher::new("/nonexistent/path/that/does/not/exist");
        let result = watcher.watch();
        assert!(matches!(result, Err(VaultError::VaultNotFound { .. })));
    }

    #[test]
    fn test_empty_paths_returns_error() {
        let watcher = VaultWatcher::with_paths(Vec::new());
        let result = watcher.watch();
        assert!(matches!(result, Err(VaultError::VaultInit { .. })));
    }

    #[test]
    fn test_stop_guard_stops_watcher() {
        let dir = temp_dir("stop_guard");
        fs::create_dir_all(&dir).unwrap();

        let watcher = VaultWatcher::new(&dir).debounce(Duration::ZERO);
        let (rx, mut guard) = watcher.watch().expect("watch should succeed");

        assert!(guard.is_running());

        guard.stop();
        assert!(!guard.is_running());

        // 监控停止后不应收到事件
        thread::sleep(Duration::from_millis(200));
        let test_file = dir.join("after_stop.txt");
        fs::write(&test_file, "nope").unwrap();

        let result = rx.recv_timeout(Duration::from_millis(500));
        assert!(result.is_err(), "should not receive events after stop");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_drop_guard_stops_watcher() {
        let dir = temp_dir("drop_guard");
        fs::create_dir_all(&dir).unwrap();

        let watcher = VaultWatcher::new(&dir).debounce(Duration::ZERO);
        let (rx, guard) = watcher.watch().expect("watch should succeed");
        assert!(guard.is_running());

        drop(guard);

        thread::sleep(Duration::from_millis(200));
        let test_file = dir.join("after_drop.txt");
        fs::write(&test_file, "nope").unwrap();

        let result = rx.recv_timeout(Duration::from_millis(500));
        assert!(result.is_err(), "should not receive events after drop");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ignore_patterns_filters_events() {
        let dir = temp_dir("ignore_patterns");
        fs::create_dir_all(&dir).unwrap();

        let watcher = VaultWatcher::new(&dir)
            .ignore_patterns(vec!["*.tmp".to_string()])
            .debounce(Duration::ZERO);

        let (rx, mut guard) = watcher.watch().expect("watch should succeed");

        thread::sleep(Duration::from_millis(200));

        // 创建 .tmp 文件（应被忽略）和 .md 文件（应通过）
        fs::write(dir.join("ignored.tmp"), "temp").unwrap();
        fs::write(dir.join("visible.md"), "note").unwrap();

        // 收集事件
        let mut found_md = false;
        let mut found_tmp = false;
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    let path = event.path();
                    if path.file_name().is_some_and(|n| n == "visible.md") {
                        found_md = true;
                    }
                    if path.file_name().is_some_and(|n| n == "ignored.tmp") {
                        found_tmp = true;
                    }
                },
                Err(_) => break,
            }
        }

        guard.stop();
        let _ = fs::remove_dir_all(&dir);

        assert!(found_md, ".md event should pass through");
        assert!(!found_tmp, ".tmp event should be ignored");
    }

    #[test]
    fn test_multi_path_watching() {
        let dir1 = temp_dir("multi_path_1");
        let dir2 = temp_dir("multi_path_2");
        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();

        let watcher =
            VaultWatcher::with_paths(vec![dir1.clone(), dir2.clone()]).debounce(Duration::ZERO);
        let (rx, mut guard) = watcher.watch().expect("watch should succeed");

        thread::sleep(Duration::from_millis(200));

        fs::write(dir1.join("file1.txt"), "one").unwrap();
        fs::write(dir2.join("file2.txt"), "two").unwrap();

        let mut found_dir1 = false;
        let mut found_dir2 = false;
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    let path = event.path();
                    if path.starts_with(&dir1) {
                        found_dir1 = true;
                    }
                    if path.starts_with(&dir2) {
                        found_dir2 = true;
                    }
                },
                Err(_) => break,
            }
        }

        guard.stop();
        let _ = fs::remove_dir_all(&dir1);
        let _ = fs::remove_dir_all(&dir2);

        assert!(found_dir1, "should receive events from dir1");
        assert!(found_dir2, "should receive events from dir2");
    }

    #[test]
    fn test_debounce_merges_events() {
        let dir = temp_dir("debounce");
        fs::create_dir_all(&dir).unwrap();

        let watcher = VaultWatcher::new(&dir).debounce(Duration::from_millis(200));
        let (rx, mut guard) = watcher.watch().expect("watch should succeed");

        thread::sleep(Duration::from_millis(200));

        // 对同一文件快速写入多次
        let file = dir.join("debounce_test.txt");
        for i in 0..5 {
            fs::write(&file, format!("content{i}")).unwrap();
        }

        // 等待防抖窗口 + 余量
        thread::sleep(Duration::from_millis(500));

        guard.stop();
        let _ = fs::remove_dir_all(&dir);

        // 收到的事件数应远少于 5
        let mut count = 0;
        while rx.recv_timeout(Duration::from_millis(200)).is_ok() {
            count += 1;
        }

        assert!(
            count <= 2,
            "debounce should reduce 5 rapid writes to at most 2 events, got {count}"
        );
    }

    #[test]
    fn test_event_path() {
        let create = VaultEvent::Create {
            path: PathBuf::from("/a/b.txt"),
        };
        assert_eq!(create.path(), Path::new("/a/b.txt"));

        let rename = VaultEvent::Rename {
            from: PathBuf::from("/a/old.txt"),
            to: PathBuf::from("/a/new.txt"),
        };
        assert_eq!(rename.path(), Path::new("/a/old.txt"));
    }
}
