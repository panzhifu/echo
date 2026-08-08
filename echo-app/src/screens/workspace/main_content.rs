use gpui::{App, IntoElement, ParentElement, Styled as _, div};
use gpui_component::{ActiveTheme as _, v_flex};

/// 渲染主内容区域：内容面板。
pub fn main_content(cx: &mut App) -> impl IntoElement {
    v_flex().h_full().flex_1().min_w_0().p_4().child(
        div()
            .flex_1()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .p_5()
            .child("Welcome to Echo!"),
    )
}
