use gpui::{IntoElement, ParentElement, Styled as _, div};
use gpui_component::TitleBar;
use gpui_component::{Icon, IconName};

/// 构建标题栏。
///
/// 左侧放置一个 Panel 图标，右侧同样放置一个 Panel 图标（静态，无功能）。
pub fn title_bar() -> impl IntoElement {
    TitleBar::new()
        .child(
            div()
                .flex()
                .items_center()
                .child(Icon::new(IconName::PanelLeft).size_4()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .child(Icon::new(IconName::PanelRight).size_4()),
        )
}
