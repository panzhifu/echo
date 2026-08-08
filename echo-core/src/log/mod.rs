//! 日志模块。
//!
//! 提供日志系统的初始化与配置功能。使用 [`tracing`] 作为底层实现，
//! 通过 [`tracing_log::LogTracer`] 桥接 [`log`] 门面，支持控制台输出、
//! 文件输出（按日期轮转）以及运行时级别热更新。
//!
//! [`log`] 宏（如 `log::info!`）无需改动即可转发到 tracing subscriber。
//!
//! # 使用示例
//!
//! ```
//! use echo_core::config::ConfigData;
//! use echo_core::log;
//!
//! let config = ConfigData::default();
//! // 持有 guard 直到程序结束，否则文件输出可能丢失日志
//! let _guard = log::init(&config.log).expect("failed to initialize logger");
//! ```

use std::path::Path;

use crate::config::{LogConfig, LogLevel, RotationKind};
use crate::error::LogError;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt, reload};

/// 持有日志系统的运行时资源。
///
/// **必须**在程序生命周期内持有此 guard：
/// - 文件输出使用非阻塞写入，drop `WorkerGuard` 前日志会被刷新；
/// - 持有的 reload handle 支持 [`LogGuard::set_level`] 运行时改变日志级别。
pub struct LogGuard {
    file_guard: Option<WorkerGuard>,
    reload_handle: Option<Handle<LevelFilter, Registry>>,
}

impl LogGuard {
    /// 运行时修改日志级别（热更新）。
    ///
    /// 无需重新初始化日志系统即可调整输出级别。
    ///
    /// # Errors
    ///
    /// 返回 [`LogError::Init`] 当 reload 失败时。
    pub fn set_level(&self, level: LogLevel) -> Result<(), LogError> {
        if let Some(handle) = &self.reload_handle {
            handle
                .reload(LevelFilter::from(level))
                .map_err(|e| LogError::Init(e.to_string()))?;
        }
        Ok(())
    }
}

/// 从 [`LogConfig`] 初始化全局日志系统。
///
/// 应在应用启动时调用一次。返回的 [`LogGuard`] 必须持有到程序结束。
/// 重复调用会返回错误（全局 subscriber 仅可设置一次）。
///
/// # Errors
///
/// 返回 [`LogError::Init`] 当 logger 已初始化或初始化失败时。
/// 返回 [`LogError::File`] 当创建日志目录失败时。
pub fn init(config: &LogConfig) -> Result<LogGuard, LogError> {
    let level_filter: LevelFilter = config.level.into();

    // 桥接 log crate -> tracing，让 `log::info!` 等宏也走 tracing subscriber。
    tracing_log::LogTracer::init().map_err(|e| LogError::Init(e.to_string()))?;
    // log 侧不阻挡任何级别，由 tracing 的 reload filter 统一控制。
    log::set_max_level(log::LevelFilter::Trace);

    let (filter_layer, reload_handle) = reload::Layer::new(level_filter);

    let mut guard = LogGuard {
        file_guard: None,
        reload_handle: Some(reload_handle),
    };

    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();
    layers.push(filter_layer.boxed());

    // 控制台输出
    if config.console_output {
        layers.push(fmt::Layer::default().with_target(true).boxed());
    }

    // 文件输出（含按日期轮转）
    if config.file_output {
        let path = config
            .file_path
            .as_deref()
            .map_or_else(|| Path::new("echo.log"), Path::new);
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| Path::new("."), Path::new);
        let file_name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("echo.log");

        // 确保目录存在
        std::fs::create_dir_all(dir)?;

        let appender = match config.rotation {
            RotationKind::Daily => tracing_appender::rolling::daily(dir, file_name),
            RotationKind::Hourly => tracing_appender::rolling::hourly(dir, file_name),
            RotationKind::Minutely => tracing_appender::rolling::minutely(dir, file_name),
            RotationKind::Never => tracing_appender::rolling::never(dir, file_name),
        };

        let (non_blocking, file_guard) = tracing_appender::non_blocking(appender);
        guard.file_guard = Some(file_guard);

        layers.push(
            fmt::Layer::default()
                .with_ansi(false)
                .with_writer(non_blocking)
                .boxed(),
        );
    }

    let subscriber = Registry::default().with(layers);
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| LogError::Init(e.to_string()))?;

    Ok(guard)
}

/// 便捷函数：从 [`ConfigData`] 初始化日志系统。
///
/// # Errors
///
/// 参见 [`init`]。
pub fn init_from_config(config: &crate::config::ConfigData) -> Result<LogGuard, LogError> {
    init(&config.log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LogLevel;
    use tracing_subscriber::filter::LevelFilter;

    #[test]
    fn test_log_config_default() {
        let config = crate::config::LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.console_output);
        assert!(!config.file_output);
        assert!(config.file_path.is_none());
        assert_eq!(config.rotation, RotationKind::Never);
    }

    #[test]
    fn test_log_level_into_level_filter() {
        let cases = [
            (LogLevel::Error, LevelFilter::ERROR),
            (LogLevel::Warn, LevelFilter::WARN),
            (LogLevel::Info, LevelFilter::INFO),
            (LogLevel::Debug, LevelFilter::DEBUG),
            (LogLevel::Trace, LevelFilter::TRACE),
        ];

        for (level, expected) in cases {
            let filter: LevelFilter = level.into();
            assert_eq!(filter, expected);
        }
    }

    #[test]
    fn set_level_updates_reload_handle() {
        // 挂到一个本地 subscriber 以确定类型参数 S = Registry，
        // 但不调用 set_global_default，避免污染全局状态。
        let (filter_layer, handle) = reload::Layer::new(LevelFilter::INFO);
        let _subscriber = Registry::default().with(filter_layer);
        let guard = LogGuard {
            file_guard: None,
            reload_handle: Some(handle),
        };
        assert!(guard.set_level(LogLevel::Debug).is_ok());
    }

    #[test]
    fn set_level_without_handle_is_noop() {
        let guard = LogGuard {
            file_guard: None,
            reload_handle: None,
        };
        assert!(guard.set_level(LogLevel::Trace).is_ok());
    }
}
