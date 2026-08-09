use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, Styled as _, Window, div,
};
use gpui_component::ActiveTheme as _;
use gpui_component::dock::{Panel, PanelEvent};

/// 文件树面板：位于 Dock 左侧边缘，展示 vault 目录。
pub struct FileTreePanel {
    focus_handle: FocusHandle,
    root_path: String,
}

impl FileTreePanel {
    pub fn new(root_path: String, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            root_path,
        }
    }
}

impl EventEmitter<PanelEvent> for FileTreePanel {}

impl Focusable for FileTreePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for FileTreePanel {
    fn panel_name(&self) -> &'static str {
        "FileTreePanel"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("文件".into())
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "文件资源管理器"
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }
}

impl Render for FileTreePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("file-tree")
            .size_full()
            .p_2()
            .text_color(cx.theme().colors.foreground)
            .child(self.root_path.clone())
    }
}
