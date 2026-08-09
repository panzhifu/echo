//! echo-vault — 文件夹监控模块。
//!
//! 提供对 vault 文件夹的递归监控，检测文件的创建、修改、删除、重命名操作。
//!
//! # 功能
//!
//! - 多路径递归监控
//! - gitignore 风格的忽略模式过滤
//! - 事件防抖（合并短时间内的重复事件）
//! - 可通过 [`WatchGuard`] 停止监控
//!
//! # 使用示例
//!
//! ```ignore
//! use echo_vault::VaultWatcher;
//! use std::time::Duration;
//!
//! let watcher = VaultWatcher::new("/path/to/vault")
//!     .ignore_patterns(vec!["*.tmp".to_string(), ".git/".to_string()])
//!     .debounce(Duration::from_millis(200));
//!
//! let (events, guard) = watcher.watch().expect("failed to start watcher");
//!
//! for event in events {
//!     match event {
//!         echo_vault::VaultEvent::Create { path } => println!("created: {}", path.display()),
//!         echo_vault::VaultEvent::Modify { path } => println!("modified: {}", path.display()),
//!         echo_vault::VaultEvent::Delete { path } => println!("deleted: {}", path.display()),
//!         echo_vault::VaultEvent::Rename { from, to } => println!("renamed: {} -> {}", from.display(), to.display()),
//!     }
//! }
//! ```

#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unimplemented,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic
)]
#![forbid(unsafe_code)]

mod debounce;
mod filter;
pub mod filter_cache;
mod watcher;

pub use filter::IgnoreFilter;
pub use filter_cache::get_or_create as get_or_create_filter;
pub use watcher::{VaultError, VaultEvent, VaultWatcher, WatchGuard};
