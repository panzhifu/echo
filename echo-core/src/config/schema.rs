use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::defaults::default_true;

/// 顶层配置结构。
///
/// 当前只包含仓库配置。未知字段通过 `extra` 兜底，便于后续扩展。
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConfigData {
    #[serde(default)]
    pub vault: VaultConfig,

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
