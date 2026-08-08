use crate::config::ConfigResult;
use crate::config::schema::ConfigData;

/// 对合并后的配置进行语义校验。
///
/// 校验内容包括：
/// - 路径格式（如果设置了 `vault.path`）
/// - 日志配置（如果启用了文件输出，必须指定有效的文件路径）
/// - 编辑器配置（`tab_size` 必须 > 0）
/// - 主题配置（`font_size` 若设置则必须 > 0）
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

    // 编辑器校验：制表符宽度必须 > 0
    if config.editor.tab_size == 0 {
        return Err(super::ConfigError::Validation(
            "editor.tab_size must be greater than 0".into(),
        ));
    }

    // 主题校验：字体大小若设置则必须 > 0
    if let Some(size) = config.theme.font_size
        && size <= 0.0
    {
        return Err(super::ConfigError::Validation(
            "theme.font_size must be greater than 0".into(),
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

    #[test]
    fn zero_tab_size_fails() {
        let config = ConfigData {
            editor: crate::config::schema::EditorConfig {
                tab_size: 0,
                show_line_numbers: true,
            },
            ..Default::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn non_positive_font_size_fails() {
        let config = ConfigData {
            theme: crate::config::schema::ThemeConfig {
                mode: crate::config::schema::ThemeMode::Dark,
                font_family: None,
                font_size: Some(0.0),
            },
            ..Default::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn negative_font_size_fails() {
        let config = ConfigData {
            theme: crate::config::schema::ThemeConfig {
                mode: crate::config::schema::ThemeMode::Dark,
                font_family: None,
                font_size: Some(-1.0),
            },
            ..Default::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn valid_font_size_passes() {
        let config = ConfigData {
            theme: crate::config::schema::ThemeConfig {
                mode: crate::config::schema::ThemeMode::Dark,
                font_family: None,
                font_size: Some(14.0),
            },
            ..Default::default()
        };
        assert!(validate(&config).is_ok());
    }
}
