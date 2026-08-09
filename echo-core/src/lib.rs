#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unimplemented,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic
)]
#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod id;
pub mod log;

pub use error::{ConfigResult, EchoError, EchoResult, LogError, LogResult};
pub use error::{MarkdownResult, VaultResult};
pub use id::{BlockId, FileId, Id, NodeId, Timestamp, VaultId, now, zero_timestamp};
pub use log::{LogGuard, init as init_log, init_from_config};
