//! 工作区界面：已配置仓库后的主界面。
//!
//! 布局：标题栏 + 侧边栏 +（主内容 + 状态栏）。

pub mod main_content;
pub mod sidebar;
pub mod status_bar;
pub mod title_bar;

use gpui::{App, IntoElement, ParentElement, Styled as _};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

/// 渲染工作区主布局。
pub fn workspace_view(cx: &mut App) -> impl IntoElement {
    v_flex()
        .size_full()
        .bg(cx.theme().background)
        // 自定义标题栏
        .child(title_bar::title_bar())
        // 主体区域：侧边栏 + 主内容
        .child(
            h_flex()
                .flex_1()
                .min_h_0()
                .child(sidebar::sidebar())
                // 主内容区域：内容面板 + 底部状态栏
                .child(
                    v_flex()
                        .h_full()
                        .flex_1()
                        .min_w_0()
                        .child(main_content::main_content(cx))
                        // 底部状态栏
                        .child(status_bar::status_bar()),
                ),
        )
}
