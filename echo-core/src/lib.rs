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
pub use log::LogError;
pub use log::{init as init_log, init_from_config};
