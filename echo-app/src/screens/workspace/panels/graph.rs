use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, Styled as _, Window, div,
};
use gpui_component::ActiveTheme as _;
use gpui_component::dock::{Panel, PanelEvent};

/// 图谱面板：位于 Dock 左侧边缘，展示笔记之间的双向链接关系。
pub struct GraphPanel {
    focus_handle: FocusHandle,
}

impl GraphPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for GraphPanel {}

impl Focusable for GraphPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for GraphPanel {
    fn panel_name(&self) -> &'static str {
        "GraphPanel"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("图谱".into())
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "图谱"
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }
}

impl Render for GraphPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("graph-panel")
            .size_full()
            .p_2()
            .text_color(cx.theme().colors.foreground)
            .child("图谱视图")
    }
}
