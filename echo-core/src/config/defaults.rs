//! 配置项的默认值常量与函数。
//!
//! 这些函数通过 `#[serde(default = "...")]` 属性在反序列化时被调用。

pub fn default_true() -> bool {
    true
}
