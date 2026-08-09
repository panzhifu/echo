use std::path::Path;

mod defaults;
mod layers;
mod persist;
mod schema;
mod validate;

pub use crate::error::{ConfigError, ConfigResult};
pub use layers::{Layers, load_layers};
pub use persist::{CachedConfig, default_config_path, load_config_from_path, save_config};
pub use schema::*;
pub use validate::validate;

/// 加载完整的配置（所有层合并后）。
///
/// 依次执行：
/// 1. [`load_layers`] — 加载所有配置层
/// 2. [`Layers::merge`] — 合并为最终配置
/// 3. [`validate`] — 语义校验
///
/// # Errors
///
/// 返回 [`ConfigError`] 当任何步骤失败时。
pub fn load_config(workspace: Option<&Path>) -> ConfigResult<ConfigData> {
    let layers = load_layers(workspace)?;
    let config = layers.merge()?;
    validate(&config)?;
    Ok(config)
}

/// 将配置保存到默认的用户全局配置文件路径。
///
/// # Errors
///
/// 返回 [`ConfigError`] 当保存失败时。
pub fn save_config_to_default(config: &ConfigData) -> ConfigResult<()> {
    save_config(config, &default_config_path())?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn default_config_data_is_valid() {
        let config = ConfigData::default();
        validate(&config).expect("default config should be valid");
    }

    #[test]
    fn vault_config_default_has_empty_recent() {
        let vault = crate::config::schema::VaultConfig::default();
        assert!(vault.path.is_none());
        assert!(vault.recent.is_empty());
        assert!(vault.auto_index);
    }

    #[test]
    fn vault_entry_roundtrip() {
        let entry = crate::config::schema::VaultEntry {
            path: "/home/user/notes".into(),
            last_opened: Some("2026-08-08T10:00:00Z".into()),
            name: Some("My Notes".into()),
        };
        let toml_str = toml::to_string(&entry).expect("serialize");
        let parsed: crate::config::schema::VaultEntry =
            toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(entry, parsed);
    }
}
