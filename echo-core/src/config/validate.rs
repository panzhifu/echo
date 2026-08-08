use crate::config::ConfigResult;
use crate::config::schema::ConfigData;

/// 对合并后的配置进行语义校验。
///
/// 校验内容包括：
/// - 路径格式（如果设置了 `vault.path`）
/// - 日志配置（如果启用了文件输出，必须指定有效的文件路径）
///
/// # Errors
///
/// 返回 [`ConfigError::Validation`] 当校验失败时。
pub fn validate(config: &ConfigData) -> ConfigResult<()> {
    if let Some(path) = &config.vault.path
        && path.is_empty()
    {
        return Err(super::ConfigError::Validation(
            "vault.path must not be empty".into(),
        ));
    }

    // 日志校验：启用文件输出时，文件路径不能是空字符串
    if config.log.file_output
        && let Some(path) = &config.log.file_path
        && path.trim().is_empty()
    {
        return Err(super::ConfigError::Validation(
            "log.file_path must not be empty when log.file_output is true".into(),
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

    #[test]
    fn log_file_output_with_empty_path_fails() {
        let config = ConfigData {
            log: crate::config::schema::LogConfig {
                file_output: true,
                file_path: Some(String::new()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn log_file_output_without_path_is_valid() {
        let config = ConfigData {
            log: crate::config::schema::LogConfig {
                file_output: true,
                file_path: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate(&config).is_ok());
    }
}
