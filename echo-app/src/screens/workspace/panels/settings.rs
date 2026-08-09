use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, Styled as _, Window, div,
};
use gpui_component::ActiveTheme as _;
use gpui_component::dock::{Panel, PanelEvent};

/// 设置面板：位于 Dock 左侧边缘，配置应用与 vault 选项。
pub struct SettingsPanel {
    focus_handle: FocusHandle,
}

impl SettingsPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for SettingsPanel {}

impl Focusable for SettingsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for SettingsPanel {
    fn panel_name(&self) -> &'static str {
        "SettingsPanel"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("设置".into())
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "设置"
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-panel")
            .size_full()
            .p_2()
            .text_color(cx.theme().colors.foreground)
            .child("设置视图")
    }
}
