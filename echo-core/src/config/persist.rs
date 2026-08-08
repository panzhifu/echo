use std::fs;
use std::path::{Path, PathBuf};

use crate::config::schema::ConfigData;
use crate::config::{ConfigError, ConfigResult};

/// 获取默认的全局配置文件路径。
///
/// - Unix: `~/.config/echo/config.toml`
/// - Windows: `%USERPROFILE%\.config\echo\config.toml`
///
/// 如果环境变量不可用，回退到当前目录。
#[must_use]
pub fn default_config_path() -> PathBuf {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("echo").join("config.toml")
}

/// 获取用户家目录。
fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// 将配置序列化为 TOML 并写入指定路径。
///
/// 如果父目录不存在，会自动创建。
///
/// # Errors
///
/// 返回 [`ConfigError`] 当序列化失败或写入磁盘失败时。
pub fn save_config(config: &ConfigData, path: &Path) -> ConfigResult<()> {
    let toml_str = toml::to_string_pretty(config).map_err(|e| {
        ConfigError::Echo(crate::EchoError::ConfigParse {
            message: format!("failed to serialize config: {e}"),
        })
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ConfigError::Echo(crate::EchoError::Io(e)))?;
    }

    fs::write(path, toml_str).map_err(|e| ConfigError::Echo(crate::EchoError::Io(e)))?;
    Ok(())
}

/// 从指定路径加载并反序列化配置。
///
/// # Errors
///
/// 返回 [`ConfigError`] 当读取文件或解析 TOML 失败时。
pub fn load_config_from_path(path: &Path) -> ConfigResult<ConfigData> {
    let content =
        fs::read_to_string(path).map_err(|e| ConfigError::Echo(crate::EchoError::Io(e)))?;
    toml::from_str(&content).map_err(|e| {
        ConfigError::Echo(crate::EchoError::ConfigParse {
            message: format!("failed to parse {}: {e}", path.display()),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::expect_used)]
    fn save_and_load_roundtrip() {
        let config = ConfigData {
            vault: crate::config::schema::VaultConfig {
                path: Some("/home/user/notes".into()),
                recent: vec![crate::config::schema::VaultEntry {
                    path: "/home/user/notes".into(),
                    last_opened: Some("2026-08-08T10:00:00Z".into()),
                    name: Some("My Notes".into()),
                }],
                auto_index: true,
            },
            ..Default::default()
        };

        let dir = std::env::temp_dir().join(format!("echo-config-test-{}", std::process::id()));
        let path = dir.join("config.toml");

        save_config(&config, &path).expect("save should succeed");
        let loaded = load_config_from_path(&path).expect("load should succeed");

        assert_eq!(config, loaded);

        // 清理
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_nonexistent_path_returns_error() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        assert!(load_config_from_path(&path).is_err());
    }
}
