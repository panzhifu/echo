use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, Styled as _, Window, div,
};
use gpui_component::ActiveTheme as _;
use gpui_component::dock::{Panel, PanelEvent};

/// 大纲面板：位于 Dock 右侧边缘，展示当前文件标题结构。
pub struct OutlinePanel {
    focus_handle: FocusHandle,
}

impl OutlinePanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for OutlinePanel {}

impl Focusable for OutlinePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for OutlinePanel {
    fn panel_name(&self) -> &'static str {
        "OutlinePanel"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("大纲".into())
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "大纲"
    }

    fn closable(&self, _cx: &App) -> bool {
        true
    }
}

impl Render for OutlinePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("outline")
            .size_full()
            .p_2()
            .text_color(cx.theme().colors.foreground)
            .child("大纲视图")
    }
}
