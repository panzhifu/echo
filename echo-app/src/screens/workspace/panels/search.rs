use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, Styled as _, Window, div,
};
use gpui_component::ActiveTheme as _;
use gpui_component::dock::{Panel, PanelEvent};

/// 搜索面板：位于 Dock 左侧边缘，搜索 vault 内容。
pub struct SearchPanel {
    focus_handle: FocusHandle,
}

impl SearchPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for SearchPanel {}

impl Focusable for SearchPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for SearchPanel {
    fn panel_name(&self) -> &'static str {
        "SearchPanel"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("搜索".into())
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "搜索"
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }
}

impl Render for SearchPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("search-panel")
            .size_full()
            .p_2()
            .text_color(cx.theme().colors.foreground)
            .child("搜索视图")
    }
}
