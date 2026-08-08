//! 配置项的默认值常量与函数。
//!
//! 这些函数通过 `#[serde(default = "...")]` 属性在反序列化时被调用。

pub fn default_true() -> bool {
    true
}

/// 编辑器制表符默认宽度（空格数）。
pub fn default_tab_size() -> usize {
    4
}

/// 侧边栏默认宽度（像素）。
pub fn default_sidebar_width() -> f32 {
    240.0
}
