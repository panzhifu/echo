use std::path::Path;

use serde::Deserialize as _;

use crate::config::persist::load_config_from_path;
use crate::config::schema::ConfigData;
use crate::config::{ConfigError, ConfigResult};

/// 分层配置来源。
///
/// 按优先级从低到高：默认值 < 全局配置 < 工作区配置。
pub struct Layers {
    /// 全局用户配置（`~/.config/echo/config.toml`）。
    pub global: Option<ConfigData>,
    /// 工作区配置（`<workspace>/.echo.toml`）。
    pub workspace: Option<ConfigData>,
}

impl Default for Layers {
    fn default() -> Self {
        Self::new()
    }
}

impl Layers {
    #[must_use]
    pub fn new() -> Self {
        Self {
            global: None,
            workspace: None,
        }
    }

    /// 将所有层合并为最终的 [`ConfigData`]。
    ///
    /// 合并规则：
    /// - 标量值（string/bool/number）：上层覆盖下层
    /// - table/map：递归合并（`extra` 字段也递归合并）
    /// - array：上层整表替换下层
    ///
    /// # Errors
    ///
    /// 返回 [`ConfigError::Echo`] 当序列化或反序列化失败时。
    pub fn merge(self) -> ConfigResult<ConfigData> {
        let mut merged = toml::Value::try_from(ConfigData::default()).map_err(|e| {
            ConfigError::Echo(crate::EchoError::ConfigParse {
                message: format!("failed to serialize defaults: {e}"),
            })
        })?;

        if let Some(global) = self.global {
            let global_value = toml::Value::try_from(&global).map_err(|e| {
                ConfigError::Echo(crate::EchoError::ConfigParse {
                    message: format!("failed to serialize global config: {e}"),
                })
            })?;
            merge_toml_value(&mut merged, global_value);
        }

        if let Some(workspace) = self.workspace {
            let workspace_value = toml::Value::try_from(&workspace).map_err(|e| {
                ConfigError::Echo(crate::EchoError::ConfigParse {
                    message: format!("failed to serialize workspace config: {e}"),
                })
            })?;
            merge_toml_value(&mut merged, workspace_value);
        }

        // 直接从 toml::Value 反序列化，避免多余的 to_string + from_str 往返
        ConfigData::deserialize(merged).map_err(|e| {
            ConfigError::Echo(crate::EchoError::ConfigParse {
                message: format!("failed to deserialize merged config: {e}"),
            })
        })
    }
}

/// 递归合并两个 `toml::Value`。
///
/// - 两边都是 table：递归合并每个 key
/// - 否则：用 overlay 替换 base
fn merge_toml_value(base: &mut toml::Value, overlay: toml::Value) {
    if matches!(base, toml::Value::Table(_))
        && matches!(&overlay, toml::Value::Table(_))
        && let toml::Value::Table(base_table) = base
        && let toml::Value::Table(overlay_table) = overlay
    {
        for (key, value) in overlay_table {
            match base_table.remove(&key) {
                Some(base_value) => {
                    let mut merged = base_value;
                    merge_toml_value(&mut merged, value);
                    base_table.insert(key, merged);
                },
                None => {
                    base_table.insert(key, value);
                },
            }
        }
        return;
    }

    *base = overlay;
}

/// 加载所有配置层。
///
/// # Errors
///
/// 返回 [`ConfigError`] 当读取或解析配置文件失败时。
pub fn load_layers(workspace: Option<&Path>) -> ConfigResult<Layers> {
    let mut layers = Layers::new();

    let global_path = super::persist::default_config_path();
    if global_path.exists() {
        layers.global = Some(load_config_from_path(&global_path)?);
    }

    if let Some(workspace) = workspace {
        let workspace_path = workspace.join(".echo.toml");
        if workspace_path.exists() {
            layers.workspace = Some(load_config_from_path(&workspace_path)?);
        }
    }

    Ok(layers)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn merge_scalars_override() {
        let mut base = toml::Value::Table(toml::toml! {
            vault = { path = "/old/path", auto_index = true }
        });
        let overlay = toml::Value::Table(toml::toml! {
            vault = { path = "/new/path" }
        });
        merge_toml_value(&mut base, overlay);

        let vault = base.get("vault").unwrap();
        assert_eq!(vault.get("path").unwrap().as_str(), Some("/new/path"));
        assert_eq!(vault.get("auto_index").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn merge_tables_recursive() {
        let mut base = toml::Value::Table(toml::toml! {
            vault = { path = "/base", auto_index = false }
        });
        let overlay = toml::Value::Table(toml::toml! {
            vault = { auto_index = true }
        });
        merge_toml_value(&mut base, overlay);

        let vault = base.get("vault").unwrap();
        assert_eq!(vault.get("path").unwrap().as_str(), Some("/base"));
        assert_eq!(vault.get("auto_index").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn merge_extra_fields_preserved() {
        let mut base = toml::Value::Table(toml::toml! {
            extra = { plugin_x = "value_a" }
        });
        let overlay = toml::Value::Table(toml::toml! {
            extra = { plugin_y = "value_b" }
        });
        merge_toml_value(&mut base, overlay);

        let extra = base.get("extra").unwrap().as_table().unwrap();
        assert_eq!(extra.get("plugin_x").unwrap().as_str(), Some("value_a"));
        assert_eq!(extra.get("plugin_y").unwrap().as_str(), Some("value_b"));
    }
}
