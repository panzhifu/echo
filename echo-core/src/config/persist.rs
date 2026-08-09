use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::schema::ConfigData;
use crate::config::{ConfigResult, EchoError};

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
/// 返回 [`EchoError`] 当序列化失败或写入磁盘失败时。
pub fn save_config(config: &ConfigData, path: &Path) -> ConfigResult<()> {
    let toml_str = toml::to_string_pretty(config).map_err(|e| EchoError::ConfigParse {
        message: format!("failed to serialize config: {e}"),
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(EchoError::Io)?;
    }

    let _ = fs::write(path, toml_str);
    Ok(())
}

/// 带缓存的配置包装器。
///
/// 缓存序列化的 TOML 字符串，只在配置数据变化时重新序列化。
/// 适用于需要重复序列化同一配置的场景。
///
/// # 使用示例
///
/// ```
/// use echo_core::config::{CachedConfig, ConfigData};
///
/// let mut cached = CachedConfig::new(ConfigData::default());
/// let _ = cached.to_toml();  // 首次序列化
/// let _ = cached.to_toml();  // 命中缓存，几乎零开销
/// cached.mark_dirty();       // 标记为已修改
/// let _ = cached.to_toml();  // 重新序列化
/// ```
pub struct CachedConfig {
    inner: ConfigData,
    cached_toml: Cell<Option<String>>,
}

impl Clone for CachedConfig {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cached_toml: Cell::new(self.cached_toml.take()),
        }
    }
}

impl std::fmt::Debug for CachedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedConfig")
            .field("inner", &self.inner)
            .field("cached_toml", &"...")
            .finish()
    }
}

impl Default for CachedConfig {
    fn default() -> Self {
        Self::new(ConfigData::default())
    }
}

impl CachedConfig {
    /// 创建一个新的缓存配置。
    #[must_use]
    pub fn new(config: ConfigData) -> Self {
        Self {
            inner: config,
            cached_toml: Cell::new(None),
        }
    }

    /// 获取内部配置的引用。
    #[must_use]
    pub fn data(&self) -> &ConfigData {
        &self.inner
    }

    /// 获取内部配置的可变引用（自动标记为 dirty）。
    pub fn data_mut(&mut self) -> &mut ConfigData {
        self.mark_dirty();
        &mut self.inner
    }

    /// 标记为已修改，下次调用 [`to_toml`](Self::to_toml) 时重新序列化。
    pub fn mark_dirty(&self) {
        self.cached_toml.take();
    }

    /// 序列化为 TOML 字符串（美化格式）。
    ///
    /// 若配置未变化，返回缓存的字符串；否则重新序列化并缓存。
    ///
    /// # Errors
    ///
    /// 返回 [`EchoError`] 当序列化失败时。
    pub fn to_toml(&self) -> ConfigResult<String> {
        if let Some(cached) = self.cached_toml.take() {
            self.cached_toml.set(Some(cached.clone()));
            return Ok(cached);
        }
        let toml_str = toml::to_string_pretty(&self.inner).map_err(|e| EchoError::ConfigParse {
            message: format!("failed to serialize config: {e}"),
        })?;
        self.cached_toml.set(Some(toml_str.clone()));
        Ok(toml_str)
    }

    /// 保存配置到指定路径（利用缓存）。
    ///
    /// # Errors
    ///
    /// 返回 [`EchoError`] 当序列化失败或写入磁盘失败时。
    pub fn save(&self, path: &Path) -> ConfigResult<()> {
        let toml_str = self.to_toml()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(EchoError::Io)?;
        }
        fs::write(path, toml_str).map_err(EchoError::Io)?;
        Ok(())
    }
}

impl From<ConfigData> for CachedConfig {
    fn from(config: ConfigData) -> Self {
        Self::new(config)
    }
}

/// 从指定路径加载并反序列化配置。
///
/// # Errors
///
/// 返回 [`EchoError`] 当读取文件或解析 TOML 失败时。
pub fn load_config_from_path(path: &Path) -> ConfigResult<ConfigData> {
    let content = fs::read_to_string(path).map_err(EchoError::Io)?;
    toml::from_str(&content).map_err(|e| EchoError::ConfigParse {
        message: format!("failed to parse {}: {e}", path.display()),
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
