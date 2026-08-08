//! 事件防抖模块。
//!
//! 合并短时间内的重复事件，减少事件风暴。
//!
//! # 防抖策略
//!
//! 对于同一路径，在 `debounce` 时间窗口内的多个事件只保留最后一个。
//! 不同路径的事件独立防抖，互不影响。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use crate::watcher::VaultEvent;

/// 防抖循环的轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// 运行事件防抖循环。
///
/// 从 `raw_rx` 接收原始事件，按路径去重后发送到 `tx`。
/// 同一路径在 `debounce` 时间窗口内的多个事件只保留最后一个。
///
/// 当 `stop` 被设置为 `true` 或 `raw_rx` 断开时退出循环，退出前刷新所有待发事件。
pub(crate) fn run_debouncer(
    raw_rx: &Receiver<VaultEvent>,
    tx: &Sender<VaultEvent>,
    debounce: Duration,
    stop: &std::sync::Arc<AtomicBool>,
) {
    if debounce.is_zero() {
        run_passthrough(raw_rx, tx, stop);
        return;
    }

    let mut pending: HashMap<PathBuf, (VaultEvent, Instant)> = HashMap::new();

    while !stop.load(Ordering::Relaxed) {
        // 尝试接收新事件
        match raw_rx.recv_timeout(POLL_INTERVAL) {
            Ok(event) => {
                let path = event.path().to_path_buf();
                let now = Instant::now();
                pending.insert(path, (event, now));
            },
            Err(RecvTimeoutError::Timeout) => {},
            Err(RecvTimeoutError::Disconnected) => break,
        }

        // 刷新已超过防抖窗口的事件
        let now = Instant::now();
        let expired: Vec<PathBuf> = pending
            .iter()
            .filter(|(_, (_, ts))| now.duration_since(*ts) >= debounce)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired {
            if let Some((event, _)) = pending.remove(&key)
                && tx.send(event).is_err()
            {
                return;
            }
        }
    }

    // 刷新剩余事件
    for (_, (event, _)) in pending {
        let _ = tx.send(event);
    }
}

/// 无防抖模式：直接转发所有事件。
fn run_passthrough(
    raw_rx: &Receiver<VaultEvent>,
    tx: &Sender<VaultEvent>,
    stop: &std::sync::Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        match raw_rx.recv_timeout(POLL_INTERVAL) {
            Ok(event) => {
                if tx.send(event).is_err() {
                    return;
                }
            },
            Err(RecvTimeoutError::Timeout) => {},
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn passthrough_forwards_all_events() {
        let (raw_tx, raw_rx) = std::sync::mpsc::channel();
        let (tx, rx) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));

        let stop_clone = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            run_debouncer(&raw_rx, &tx, Duration::ZERO, &stop_clone);
        });

        for i in 0..5 {
            raw_tx
                .send(VaultEvent::Modify {
                    path: PathBuf::from(format!("/tmp/file{i}")),
                })
                .unwrap();
        }

        let mut count = 0;
        while let Ok(_event) = rx.recv_timeout(Duration::from_secs(1)) {
            count += 1;
        }

        stop.store(true, Ordering::Relaxed);
        let _ = handle.join();

        assert_eq!(count, 5, "passthrough should forward all 5 events");
    }

    #[test]
    fn debounce_merges_same_path_events() {
        let (raw_tx, raw_rx) = std::sync::mpsc::channel();
        let (tx, rx) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));

        let stop_clone = Arc::clone(&stop);
        let debounce = Duration::from_millis(200);
        let handle = thread::spawn(move || {
            run_debouncer(&raw_rx, &tx, debounce, &stop_clone);
        });

        // 同一路径快速发送 5 个 Modify 事件
        let path = PathBuf::from("/tmp/file.md");
        for _ in 0..5 {
            raw_tx
                .send(VaultEvent::Modify { path: path.clone() })
                .unwrap();
        }

        // 等待防抖窗口 + 余量
        thread::sleep(Duration::from_millis(400));

        stop.store(true, Ordering::Relaxed);
        let _ = handle.join();

        let mut count = 0;
        while let Ok(_event) = rx.recv_timeout(Duration::from_millis(100)) {
            count += 1;
        }

        assert_eq!(count, 1, "debounce should merge 5 rapid events into 1");
    }

    #[test]
    fn debounce_keeps_different_path_events() {
        let (raw_tx, raw_rx) = std::sync::mpsc::channel();
        let (tx, rx) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));

        let stop_clone = Arc::clone(&stop);
        let debounce = Duration::from_millis(200);
        let handle = thread::spawn(move || {
            run_debouncer(&raw_rx, &tx, debounce, &stop_clone);
        });

        // 不同路径发送事件
        for i in 0..3 {
            raw_tx
                .send(VaultEvent::Create {
                    path: PathBuf::from(format!("/tmp/file{i}")),
                })
                .unwrap();
        }

        thread::sleep(Duration::from_millis(400));

        stop.store(true, Ordering::Relaxed);
        let _ = handle.join();

        let mut count = 0;
        while let Ok(_event) = rx.recv_timeout(Duration::from_millis(100)) {
            count += 1;
        }

        assert_eq!(count, 3, "different paths should not be merged");
    }
}
