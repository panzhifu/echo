use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::defaults::{default_sidebar_width, default_tab_size, default_true};

/// 日志级别。
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing_subscriber::filter::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
            LogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            LogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            LogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            LogLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
        }
    }
}

/// 日志文件轮转策略。
///
/// 按时间维度分割日志文件。底层由 [`tracing_appender`] 提供，
/// 仅支持按时间轮转，不支持按大小轮转。
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RotationKind {
    /// 按天轮转：文件名追加 `YYYY-MM-DD`。
    Daily,
    /// 按小时轮转：文件名追加 `YYYY-MM-DD-HH`。
    Hourly,
    /// 按分钟轮转：文件名追加 `YYYY-MM-DD-HH-MM`。
    Minutely,
    /// 不轮转，写入单一文件。
    #[default]
    Never,
}

/// 日志配置。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LogConfig {
    /// 日志级别。
    #[serde(default)]
    pub level: LogLevel,

    /// 是否输出到控制台。
    #[serde(default = "default_true")]
    pub console_output: bool,

    /// 是否输出到文件。
    #[serde(default)]
    pub file_output: bool,

    /// 日志文件路径（可选，默认为 `echo.log`）。
    #[serde(default)]
    pub file_path: Option<String>,

    /// 日志文件轮转策略（仅当 `file_output = true` 时生效）。
    #[serde(default)]
    pub rotation: RotationKind,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            console_output: true,
            file_output: false,
            file_path: None,
            rotation: RotationKind::default(),
        }
    }
}

/// 主题模式。
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
    /// 跟随系统。
    Auto,
}

/// 主题配置。
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    /// 主题模式。
    #[serde(default)]
    pub mode: ThemeMode,

    /// 字体族（可选，留空使用平台默认）。
    #[serde(default)]
    pub font_family: Option<String>,

    /// 字体大小（pt，可选）。
    #[serde(default)]
    pub font_size: Option<f32>,
}

/// 编辑器配置。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditorConfig {
    /// 制表符宽度（空格数）。
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,

    /// 是否显示行号。
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: default_tab_size(),
            show_line_numbers: default_true(),
        }
    }
}

/// 侧边栏配置。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SidebarConfig {
    /// 宽度（像素）。
    #[serde(default = "default_sidebar_width")]
    pub width: f32,

    /// 是否折叠。
    #[serde(default)]
    pub collapsed: bool,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            width: default_sidebar_width(),
            collapsed: false,
        }
    }
}

/// 顶层配置结构。
///
/// 当前包含仓库、日志、主题、编辑器、侧边栏配置。未知字段通过 `extra` 兜底，
/// 便于后续扩展。
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConfigData {
    #[serde(default)]
    pub vault: VaultConfig,

    #[serde(default)]
    pub log: LogConfig,

    #[serde(default)]
    pub theme: ThemeConfig,

    #[serde(default)]
    pub editor: EditorConfig,

    #[serde(default)]
    pub sidebar: SidebarConfig,

    /// 兜底字段：未被上面声明的字段进入这里，保证向前兼容。
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VaultConfig {
    /// 当前仓库路径。
    #[serde(default)]
    pub path: Option<String>,

    /// 最近使用的仓库列表。
    #[serde(default)]
    pub recent: Vec<VaultEntry>,

    /// 启动时自动构建索引。
    #[serde(default = "default_true")]
    pub auto_index: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            path: None,
            recent: Vec::new(),
            auto_index: default_true(),
        }
    }
}

impl VaultConfig {
    /// 检查当前是否配置了有效的仓库路径。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.path.is_some()
    }

    /// 添加一个仓库到最近列表（如果已存在则移到最前）。
    pub fn add_recent(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.recent.retain(|e| e.path != path);
        self.recent.insert(
            0,
            VaultEntry {
                path,
                last_opened: None,
                name: None,
            },
        );
    }
}

/// 最近使用的仓库条目。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VaultEntry {
    pub path: String,

    /// ISO8601 格式的最后打开时间。
    #[serde(default)]
    pub last_opened: Option<String>,

    /// 用户自定义名称（可选）。
    #[serde(default)]
    pub name: Option<String>,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn log_config_defaults() {
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.console_output);
        assert!(!config.file_output);
        assert!(config.file_path.is_none());
        assert_eq!(config.rotation, RotationKind::Never);
    }

    #[test]
    fn log_level_to_level_filter() {
        use tracing_subscriber::filter::LevelFilter;

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
    fn rotation_kind_default_is_never() {
        assert_eq!(RotationKind::default(), RotationKind::Never);
    }

    #[test]
    fn theme_config_defaults() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert!(theme.font_family.is_none());
        assert!(theme.font_size.is_none());
    }

    #[test]
    fn editor_config_defaults() {
        let editor = EditorConfig::default();
        assert_eq!(editor.tab_size, 4);
        assert!(editor.show_line_numbers);
    }

    #[test]
    fn sidebar_config_defaults() {
        let sidebar = SidebarConfig::default();
        assert!((sidebar.width - 240.0).abs() < 1e-6);
        assert!(!sidebar.collapsed);
    }

    #[test]
    fn config_data_roundtrip_with_new_groups() {
        let config = ConfigData {
            theme: ThemeConfig {
                mode: ThemeMode::Light,
                font_family: Some("monospace".into()),
                font_size: Some(14.0),
            },
            editor: EditorConfig {
                tab_size: 2,
                show_line_numbers: false,
            },
            sidebar: SidebarConfig {
                width: 300.0,
                collapsed: true,
            },
            ..Default::default()
        };

        let toml_str = toml::to_string(&config).expect("serialize");
        let parsed: ConfigData = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(config, parsed);
    }

    #[test]
    fn theme_mode_serde_lowercase() {
        #[derive(Deserialize)]
        struct Wrap {
            mode: ThemeMode,
        }
        let w: Wrap = toml::from_str("mode = \"auto\"").expect("deserialize");
        assert_eq!(w.mode, ThemeMode::Auto);
    }

    #[test]
    fn rotation_kind_serde_lowercase() {
        #[derive(Deserialize)]
        struct Wrap {
            rotation: RotationKind,
        }
        let w: Wrap = toml::from_str("rotation = \"daily\"").expect("deserialize");
        assert_eq!(w.rotation, RotationKind::Daily);
    }
}
