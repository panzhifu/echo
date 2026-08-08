//! 应用逻辑层：从 GUI 代码中提取的可测试纯逻辑。
//!
//! 这些函数不依赖 GPUI 的窗口与上下文，可在普通单元测试与基准中验证。

use echo_core::config::{ConfigData, VaultConfig};

/// 应用状态：是否已加载仓库。
///
/// 由 [`AppState::from_vault`] 根据仓库配置派生。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    /// 未配置仓库，显示仓库管理界面。
    NoVault,
    /// 已配置仓库，显示工作区界面。
    VaultLoaded,
}

impl AppState {
    /// 根据仓库配置判断应用初始状态。
    ///
    /// - 仓库路径有效 -> [`AppState::VaultLoaded`]
    /// - 否则 -> [`AppState::NoVault`]
    #[must_use]
    pub fn from_vault(vault: &VaultConfig) -> Self {
        if vault.is_valid() {
            Self::VaultLoaded
        } else {
            Self::NoVault
        }
    }
}

/// 将用户选择的仓库路径应用到配置数据。
///
/// 设置当前仓库路径，并将其加入最近使用列表（若已存在则移到最前）。
/// 此函数只更新内存中的配置，持久化与 UI 通知由调用方负责。
pub fn apply_vault_selection(config: &mut ConfigData, path: impl Into<String>) {
    let path = path.into();
    config.vault.path = Some(path.clone());
    config.vault.add_recent(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_core::config::ConfigData;

    #[test]
    fn no_vault_when_path_absent() {
        let config = ConfigData::default();
        assert_eq!(AppState::from_vault(&config.vault), AppState::NoVault);
    }

    #[test]
    fn vault_loaded_when_path_set() {
        let mut config = ConfigData::default();
        config.vault.path = Some("/notes".into());
        assert_eq!(AppState::from_vault(&config.vault), AppState::VaultLoaded);
    }

    #[test]
    fn apply_selection_sets_path_and_recent() {
        let mut config = ConfigData::default();
        apply_vault_selection(&mut config, "/notes");
        assert_eq!(config.vault.path.as_deref(), Some("/notes"));
        assert_eq!(config.vault.recent.len(), 1);
        assert_eq!(config.vault.recent[0].path, "/notes");
    }

    #[test]
    fn apply_selection_promotes_existing_to_front() {
        let mut config = ConfigData::default();
        apply_vault_selection(&mut config, "/a");
        apply_vault_selection(&mut config, "/b");
        // 再次选择 /a：移到最前且不重复
        apply_vault_selection(&mut config, "/a");

        assert_eq!(config.vault.path.as_deref(), Some("/a"));
        assert_eq!(config.vault.recent.len(), 2);
        assert_eq!(config.vault.recent[0].path, "/a");
        assert_eq!(config.vault.recent[1].path, "/b");
    }

    #[test]
    fn apply_selection_accepts_owned_string() {
        let mut config = ConfigData::default();
        let path: String = String::from("/notes");
        apply_vault_selection(&mut config, path);
        assert_eq!(config.vault.path.as_deref(), Some("/notes"));
    }
}
