use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Core error type for the echo system.
///
/// This enum defines structured error variants shared across all echo crates.
/// Each variant carries sufficient context (paths, messages, version info) to
/// aid debugging and to present actionable information to users.
///
/// `EchoError` serves as the foundation of echo's error handling hierarchy:
/// - Lower-level crates use `thiserror` to define their own error enums and
///   convert into `EchoError` via `#[from]`.
/// - Application layers aggregate errors using `anyhow`.
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
}

/// Type alias for [`std::result::Result`] with [`EchoError`] as the error type.
///
/// Use this throughout `echo-core` and lower-level crates for consistent
/// error handling.
pub type EchoResult<T> = Result<T, EchoError>;

/// Configuration-specific error type.
///
/// Used for errors that occur during configuration loading, validation,
/// and persistence. Wraps [`EchoError`] for underlying IO/parse failures.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Semantic validation failed (e.g., out-of-range values).
    #[error("config validation failed: {0}")]
    Validation(String),

    /// Underlying error from IO or serialization.
    #[error(transparent)]
    Echo(#[from] EchoError),
}

/// Type alias for [`std::result::Result`] with [`ConfigError`] as the error type.
pub type ConfigResult<T> = Result<T, ConfigError>;

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
