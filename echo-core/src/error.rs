use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Core error type for the echo system.
///
/// This enum defines structured error variants shared across all echo crates.
/// Each variant carries sufficient context (paths, messages, version info) to
/// aid debugging and to present actionable information to users.
///
/// `EchoError` serves as the single error type for the entire workspace:
/// - Lower-level crates convert their errors into `EchoError` via `From`.
/// - Application layers aggregate errors using `anyhow`.
///
/// # Design Notes
///
/// External crate errors (e.g., `notify::Error`, `ignore::Error`) are represented
/// as string messages rather than wrapped types. This keeps `echo-core` free of
/// dependencies on crates that only specific modules need.
///
/// # Examples
///
/// ```
/// use echo_core::{EchoError, EchoResult};
///
/// fn find_vault(path: &std::path::Path) -> EchoResult<String> {
///     if !path.exists() {
///         return Err(EchoError::VaultNotFound {
///             path: path.to_path_buf(),
///         });
///     }
///     Ok("vault".to_string())
/// }
/// ```
#[derive(Debug, Error)]
pub enum EchoError {
    /// An IO error occurred during a file or network operation.
    ///
    /// Wraps [`std::io::Error`] to provide a structured representation
    /// that can propagate through echo's error hierarchy.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// A vault was not found at the specified path.
    #[error("vault not found: {path}")]
    VaultNotFound { path: PathBuf },

    /// A configuration file was not found at the specified path.
    #[error("config not found: {path}")]
    ConfigNotFound { path: PathBuf },

    /// Failed to parse a configuration file.
    #[error("config parse error: {message}")]
    ConfigParse { message: String },

    /// An invalid or malformed path was encountered.
    #[error("invalid path: {path}")]
    InvalidPath { path: PathBuf },

    /// A version or schema mismatch was detected.
    ///
    /// Typically used during configuration or index migrations.
    #[error("version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    /// An invalid or malformed identifier was encountered.
    ///
    /// Used when parsing IDs (e.g. node, block, file, or vault IDs) from
    /// strings.
    #[error("invalid id: {message}")]
    InvalidId { message: String },

    /// Configuration semantic validation failed (e.g., out-of-range values).
    #[error("config validation failed: {message}")]
    ConfigValidation { message: String },

    /// Markdown parsing or serialization failed.
    ///
    /// Used by `echo-markdown` for structural errors during document processing.
    #[error("markdown error: {message}")]
    Markdown { message: String },

    /// Vault watcher initialization failed.
    ///
    /// Used by `echo-vault` when the file system watcher cannot be created
    /// or configured.
    #[error("vault watcher init failed: {message}")]
    VaultInit { message: String },

    /// File system notification error.
    ///
    /// Used by `echo-vault` when the underlying `notify` crate encounters
    /// an error. The original error message is preserved as a string to
    /// avoid a hard dependency on `notify` in `echo-core`.
    #[error("vault notify error: {message}")]
    VaultNotify { message: String },
}

/// Type alias for [`std::result::Result`] with [`EchoError`] as the error type.
///
/// Use this throughout the entire workspace for consistent error handling.
pub type EchoResult<T> = Result<T, EchoError>;

impl EchoError {
    /// Create a markdown error from a message.
    ///
    /// This is a convenience constructor for `EchoError::Markdown`.
    ///
    /// # Examples
    ///
    /// ```
    /// use echo_core::EchoError;
    /// let err = EchoError::markdown("unexpected eof");
    /// ```
    #[must_use]
    pub fn markdown(message: impl Into<String>) -> Self {
        EchoError::Markdown {
            message: message.into(),
        }
    }

    /// Create a vault initialization error from a message.
    ///
    /// This is a convenience constructor for `EchoError::VaultInit`.
    #[must_use]
    pub fn vault_init(message: impl Into<String>) -> Self {
        EchoError::VaultInit {
            message: message.into(),
        }
    }

    /// Create a vault notify error from a message.
    ///
    /// This is a convenience constructor for `EchoError::VaultNotify`.
    /// Used to convert `notify::Error` without depending on the `notify` crate.
    #[must_use]
    pub fn vault_notify(message: impl Into<String>) -> Self {
        EchoError::VaultNotify {
            message: message.into(),
        }
    }

    /// Create a config validation error from a message.
    ///
    /// This is a convenience constructor for `EchoError::ConfigValidation`.
    #[must_use]
    pub fn config_validation(message: impl Into<String>) -> Self {
        EchoError::ConfigValidation {
            message: message.into(),
        }
    }
}

// ========== 向后兼容的 Result 类型别名 ==========

/// Configuration-specific error type.
///
/// Alias for `EchoError` — provided for backward compatibility.
pub type ConfigError = EchoError;

/// Configuration-specific result type.
///
/// Alias for `EchoResult<T>` — provided for backward compatibility.
pub type ConfigResult<T> = EchoResult<T>;

/// Logging-specific result type.
///
/// Alias for `EchoResult<T>` — provided for backward compatibility.
pub type LogResult<T> = EchoResult<T>;

/// Markdown-specific result type.
///
/// Alias for `EchoResult<T>` — provided for backward compatibility.
pub type MarkdownResult<T> = EchoResult<T>;

/// Vault-specific result type.
///
/// Alias for `EchoResult<T>` — provided for backward compatibility.
pub type VaultResult<T> = EchoResult<T>;

// ========== 日志错误（保留用于细粒度错误处理） ==========

/// Logging-specific error type.
///
/// Used for errors that occur during logger initialization and log file operations.
/// Can be converted into [`EchoError`] via `From` for unified error handling.
#[derive(Debug, Error)]
pub enum LogError {
    /// Logger initialization failed (e.g., already initialized).
    #[error("failed to initialize logger: {0}")]
    Init(String),

    /// Failed to create log file or directory.
    #[error("failed to create log file: {0}")]
    File(#[from] std::io::Error),
}

impl From<LogError> for EchoError {
    fn from(err: LogError) -> Self {
        match err {
            LogError::Init(msg) => EchoError::ConfigParse { message: msg },
            LogError::File(io_err) => EchoError::Io(io_err),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;

    #[test]
    fn io_error_converts_via_from() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let echo_err: EchoError = io_err.into();
        match echo_err {
            EchoError::Io(_) => {},
            _ => panic!("Expected EchoError::Io variant"),
        }
    }

    #[test]
    fn vault_not_found_contains_path() {
        let err = EchoError::VaultNotFound {
            path: PathBuf::from("/tmp/nonexistent"),
        };
        let msg = err.to_string();
        assert!(msg.contains("vault not found"));
        assert!(msg.contains("/tmp/nonexistent"));
    }

    #[test]
    fn config_not_found_contains_path() {
        let err = EchoError::ConfigNotFound {
            path: PathBuf::from("/etc/echo/config.json"),
        };
        let msg = err.to_string();
        assert!(msg.contains("config not found"));
        assert!(msg.contains("/etc/echo/config.json"));
    }

    #[test]
    fn config_parse_contains_message() {
        let err = EchoError::ConfigParse {
            message: "missing field 'name'".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("config parse error"));
        assert!(msg.contains("missing field"));
    }

    #[test]
    fn invalid_path_contains_path() {
        let err = EchoError::InvalidPath {
            path: PathBuf::from(""),
        };
        let msg = err.to_string();
        assert!(msg.contains("invalid path"));
    }

    #[test]
    fn version_mismatch_shows_both_versions() {
        let err = EchoError::VersionMismatch {
            expected: "1.0".to_string(),
            actual: "0.9".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("version mismatch"));
        assert!(msg.contains("1.0"));
        assert!(msg.contains("0.9"));
    }

    #[test]
    fn invalid_id_contains_message() {
        let err = EchoError::InvalidId {
            message: "bad format".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("invalid id"));
        assert!(msg.contains("bad format"));
    }

    #[test]
    fn markdown_error_constructor() {
        let err = EchoError::markdown("unexpected eof");
        match err {
            EchoError::Markdown { message } => assert_eq!(message, "unexpected eof"),
            _ => panic!("Expected Markdown variant"),
        }
    }

    #[test]
    fn vault_init_error_constructor() {
        let err = EchoError::vault_init("failed to create watcher");
        match err {
            EchoError::VaultInit { message } => assert_eq!(message, "failed to create watcher"),
            _ => panic!("Expected VaultInit variant"),
        }
    }

    #[test]
    fn vault_notify_error_constructor() {
        let err = EchoError::vault_notify("permission denied");
        match err {
            EchoError::VaultNotify { message } => assert_eq!(message, "permission denied"),
            _ => panic!("Expected VaultNotify variant"),
        }
    }

    #[test]
    fn config_validation_error_constructor() {
        let err = EchoError::config_validation("tab_size must be > 0");
        match err {
            EchoError::ConfigValidation { message } => {
                assert_eq!(message, "tab_size must be > 0");
            },
            _ => panic!("Expected ConfigValidation variant"),
        }
    }

    #[test]
    fn echo_result_ok_value() {
        let result: EchoResult<u32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn echo_result_err_variant() {
        let result: EchoResult<u32> = Err(EchoError::InvalidId {
            message: "bad format".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn log_error_converts_to_echo_error() {
        let log_err = LogError::Init("already initialized".to_string());
        let echo_err: EchoError = log_err.into();
        match echo_err {
            EchoError::ConfigParse { message } => assert_eq!(message, "already initialized"),
            _ => panic!("Expected ConfigParse variant"),
        }
    }

    #[test]
    fn log_file_error_converts_to_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let log_err = LogError::File(io_err);
        let echo_err: EchoError = log_err.into();
        match echo_err {
            EchoError::Io(_) => {},
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn all_error_messages_are_ascii() {
        let errors = vec![
            EchoError::Io(io::Error::other("test")),
            EchoError::VaultNotFound {
                path: PathBuf::from("/test"),
            },
            EchoError::ConfigNotFound {
                path: PathBuf::from("/test"),
            },
            EchoError::ConfigParse {
                message: "test".to_string(),
            },
            EchoError::InvalidPath {
                path: PathBuf::from("/test"),
            },
            EchoError::VersionMismatch {
                expected: "1.0".to_string(),
                actual: "0.9".to_string(),
            },
            EchoError::InvalidId {
                message: "test".to_string(),
            },
            EchoError::ConfigValidation {
                message: "test".to_string(),
            },
            EchoError::markdown("test"),
            EchoError::vault_init("test"),
            EchoError::vault_notify("test"),
        ];
        for err in errors {
            let msg = err.to_string();
            assert!(
                msg.is_ascii(),
                "Error message should be ASCII (English): {msg}"
            );
        }
    }
}
