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
pub mod log;

pub use error::{ConfigError, ConfigResult, EchoError, EchoResult};
pub use error::{LogError, LogResult};
pub use log::{LogGuard, init as init_log, init_from_config};
