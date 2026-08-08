use gpui::{Context, Entity, IntoElement, ParentElement, Render, Styled as _, Window, div};
use gpui_component::{ActiveTheme as _, TitleBar, h_flex, v_flex};

use echo_core::config::ConfigData;

mod panel;

/// 仓库管理视图。
///
/// 布局：默认标题栏 +（左侧仓库列表侧边栏 + 右侧功能区面板）。
/// 右侧提供两个功能区：新建仓库 / 打开已有仓库。
/// 选中仓库后写入响应式配置，由应用观察配置变化切换到工作区。
pub struct VaultManagerView {
    /// 共享的响应式配置实体。
    config: Entity<ConfigData>,
}

impl VaultManagerView {
    /// 创建一个新的 [`VaultManagerView`] 实例。
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>, config: Entity<ConfigData>) -> Self {
        Self { config }
    }
}

impl Render for VaultManagerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            // 默认标题栏
            .child(default_title_bar())
            // 主体：左侧侧边栏 + 右侧功能区
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(vault_sidebar(cx))
                    .child(panel::right_panel(cx)),
            )
    }
}

/// 渲染默认标题栏（右侧为窗口控制按钮，左侧显示应用名）。
fn default_title_bar() -> impl IntoElement {
    TitleBar::new().child(div().flex().items_center().child("Echo"))
}

/// 渲染左侧仓库列表侧边栏。
fn vault_sidebar(cx: &mut Context<VaultManagerView>) -> impl IntoElement {
    use gpui::px;

    v_flex()
        .h_full()
        .w(px(240.))
        .border_r_1()
        .border_color(cx.theme().border)
        .p_3()
        .gap_3()
        // 标题
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("仓库"),
        )
        // 仓库列表占位
        .child(v_flex().flex_1().gap_1().child("暂无仓库"))
}
