use gpui::{IntoElement, SharedString};
use gpui_component::status_bar::StatusBar;

/// 构建底部状态栏。
///
/// 左侧显示应用状态，右侧显示版本号。
pub fn status_bar() -> impl IntoElement {
    let version = SharedString::from(format!("v{}", env!("CARGO_PKG_VERSION")));
    StatusBar::new().left("Ready").right(version)
}
