//! 日志模块。
//!
//! 提供日志系统的初始化与配置功能。使用 [`log`] crate 作为日志门面，
//! [`fern`] 作为底层实现，支持控制台输出和文件输出。
//!
//! # 使用示例
//!
//! ```
//! use echo_core::config::ConfigData;
//! use echo_core::log;
//!
//! let config = ConfigData::default();
//! log::init(&config.log).expect("failed to initialize logger");
//! ```

use std::path::Path;

use crate::config::LogConfig;

/// 日志模块错误类型。
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    /// 日志系统初始化失败（例如重复初始化）。
    #[error("failed to initialize logger: {0}")]
    Init(String),

    /// 创建日志文件或目录失败。
    #[error("failed to create log file: {0}")]
    File(#[from] std::io::Error),
}

/// 从 [`LogConfig`] 初始化全局日志系统。
///
/// 应在应用启动时调用一次。重复调用会返回错误（`fern` 限制）。
pub fn init(config: &LogConfig) -> Result<(), LogError> {
    let level_filter: log::LevelFilter = config.level.into();

    let mut logger = fern::Dispatch::new().level(level_filter);

    // 控制台输出
    if config.console_output {
        logger = logger.chain(
            fern::Dispatch::new()
                .level(level_filter)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {:<5} {}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .chain(std::io::stdout()),
        );
    }

    // 文件输出
    if config.file_output {
        let path = config
            .file_path
            .as_deref()
            .map(Path::new)
            .unwrap_or_else(|| Path::new("echo.log"));

        // 确保父目录存在
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        logger = logger.chain(
            fern::Dispatch::new()
                .level(level_filter)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {:<5} {}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .chain(fern::log_file(path)?),
        );
    }

    logger.apply().map_err(|e| LogError::Init(e.to_string()))?;

    Ok(())
}

/// 便捷函数：从 [`ConfigData`] 初始化日志系统。
pub fn init_from_config(config: &crate::config::ConfigData) -> Result<(), LogError> {
    init(&config.log)
}

#[cfg(test)]
mod tests {
    use crate::config::LogLevel;

    #[test]
    fn test_log_config_default() {
        let config = crate::config::LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.console_output);
        assert!(!config.file_output);
        assert!(config.file_path.is_none());
    }

    #[test]
    fn test_log_level_into_level_filter() {
        use log::LevelFilter;

        let cases = [
            (LogLevel::Error, LevelFilter::Error),
            (LogLevel::Warn, LevelFilter::Warn),
            (LogLevel::Info, LevelFilter::Info),
            (LogLevel::Debug, LevelFilter::Debug),
            (LogLevel::Trace, LevelFilter::Trace),
        ];

        for (level, expected) in cases {
            let filter: LevelFilter = level.into();
            assert_eq!(filter, expected);
        }
    }
}
