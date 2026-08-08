use crate::config::ConfigResult;
use crate::config::schema::ConfigData;

/// 对合并后的配置进行语义校验。
///
/// 校验内容包括：
/// - 路径格式（如果设置了 `vault.path`）
pub fn validate(config: &ConfigData) -> ConfigResult<()> {
    if let Some(path) = &config.vault.path
        && path.is_empty()
    {
        return Err(super::ConfigError::Validation(
            "vault.path must not be empty".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn valid_config_passes() {
        let config = ConfigData::default();
        validate(&config).expect("default config should be valid");
    }

    #[test]
    fn empty_vault_path_fails() {
        let config = ConfigData {
            vault: crate::config::schema::VaultConfig {
                path: Some(String::new()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate(&config).is_err());
    }
}
