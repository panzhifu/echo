//! 工作区界面：已配置仓库后的主界面。
//!
//! 布局：标题栏 +（功能按钮栏 + Dock 布局）+ 状态栏。
//! Dock 布局由 [`DockArea`] 统一管理，支持分割、标签、折叠、拖拽；
//! 功能按钮栏（Activity Bar）点击切换左侧 Dock 面板。

pub mod activity_bar;
pub mod panels;
pub mod status_bar;
pub mod title_bar;

use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Edges, Entity, InteractiveElement as _, IntoElement, ParentElement,
    Render, Styled as _, Subscription, Window, px,
};
use gpui_component::ActiveTheme as _;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants as _};
use gpui_component::dock::{DockArea, DockEvent, DockItem, DockPlacement};
use gpui_component::{h_flex, v_flex};

use self::activity_bar::ActivityTool;
use self::panels::{
    EditorPanel, FileTreePanel, GraphPanel, OutlinePanel, SearchPanel, SettingsPanel,
};

/// 工作区视图：由 `DockArea` 管理全部面板布局。
pub struct WorkspaceView {
    dock_area: Entity<DockArea>,
    /// 当前激活的功能按钮工具。
    active_tool: ActivityTool,
    /// vault 根路径，用于文件树面板展示。
    vault_path: String,
    /// 保持订阅存活，避免被 drop 后取消订阅（无需读取）。
    #[expect(dead_code)]
    subscriptions: Vec<Subscription>,
}

impl WorkspaceView {
    /// 创建工作区视图。
    ///
    /// `vault_path` 为当前 vault 路径，用于文件树面板展示。
    pub fn new(window: &mut Window, cx: &mut Context<Self>, vault_path: Option<&str>) -> Self {
        let dock_area = cx.new(|cx| DockArea::new("main-dock", Some(5), window, cx));

        // 订阅布局变更事件（后续可用于持久化布局）
        let subscriptions =
            vec![
                cx.subscribe_in(&dock_area, window, |_, _, ev: &DockEvent, _, _| match ev {
                    DockEvent::LayoutChanged => {
                        log::info!("dock layout changed");
                    },
                    DockEvent::DragDrop(_) => {},
                }),
            ];

        let vault_path = vault_path.unwrap_or(".").to_string();
        let active_tool = ActivityTool::Files;
        setup_default_layout(&dock_area, &vault_path, window, cx);

        Self {
            dock_area,
            active_tool,
            vault_path,
            subscriptions,
        }
    }

    /// 切换功能按钮工具：重建左侧 Dock 面板。
    ///
    /// 点击当前已激活的工具时，折叠/展开左侧 Dock。
    fn switch_tool(&mut self, tool: ActivityTool, window: &mut Window, cx: &mut Context<Self>) {
        if tool == self.active_tool {
            self.dock_area.update(cx, |area, cx| {
                area.toggle_dock(DockPlacement::Left, window, cx);
            });
            return;
        }

        self.active_tool = tool;
        self.dock_area.update(cx, |area, cx| {
            area.remove_left_dock(window, cx);

            let weak = cx.entity().downgrade();
            let root = self.vault_path.clone();
            let item = match tool {
                ActivityTool::Files => DockItem::tab(
                    cx.new(|cx| FileTreePanel::new(root, window, cx)),
                    &weak,
                    window,
                    cx,
                ),
                ActivityTool::Search => {
                    DockItem::tab(cx.new(|cx| SearchPanel::new(window, cx)), &weak, window, cx)
                },
                ActivityTool::Graph => {
                    DockItem::tab(cx.new(|cx| GraphPanel::new(window, cx)), &weak, window, cx)
                },
                ActivityTool::Settings => DockItem::tab(
                    cx.new(|cx| SettingsPanel::new(window, cx)),
                    &weak,
                    window,
                    cx,
                ),
            };

            area.set_left_dock(item, Some(px(260.)), true, window, cx);
        });
        cx.notify();
    }

    /// 渲染功能按钮栏：垂直排列的一排图标按钮。
    fn activity_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut bar = v_flex()
            .id("activity-bar")
            .w(px(48.))
            .h_full()
            .items_center()
            .pt_2()
            .gap_1()
            .border_r_1()
            .border_color(cx.theme().colors.border)
            .bg(cx.theme().colors.sidebar);

        for tool in ActivityTool::ALL {
            let active = tool == self.active_tool;
            let button = if active {
                Button::new(tool.id()).icon(tool.icon()).custom(
                    ButtonCustomVariant::new(cx)
                        .color(cx.theme().colors.sidebar_accent)
                        .foreground(cx.theme().colors.sidebar_accent_foreground),
                )
            } else {
                Button::new(tool.id()).ghost().icon(tool.icon())
            };

            bar = bar.child(button.tooltip(tool.label()).on_click(cx.listener(
                move |this, _, window, cx| {
                    this.switch_tool(tool, window, cx);
                },
            )));
        }
        bar
    }
}

/// 构建默认布局：左侧文件树 + 中心编辑器标签页 + 右侧大纲。
fn setup_default_layout(
    dock_area: &Entity<DockArea>,
    vault_path: &str,
    window: &mut Window,
    cx: &mut App,
) {
    let weak = dock_area.downgrade();

    // 左侧边缘 dock：文件树
    let left_item = DockItem::tab(
        cx.new(|cx| FileTreePanel::new(vault_path.to_string(), window, cx)),
        &weak,
        window,
        cx,
    );

    // 中心：编辑器标签页
    let center_item = DockItem::tabs(
        vec![
            Arc::new(cx.new(|cx| EditorPanel::new("未命名.md", window, cx))),
            Arc::new(cx.new(|cx| EditorPanel::new("README.md", window, cx))),
        ],
        &weak,
        window,
        cx,
    );

    // 右侧边缘 dock：大纲
    let right_item = DockItem::tab(
        cx.new(|cx| OutlinePanel::new(window, cx)),
        &weak,
        window,
        cx,
    );

    dock_area.update(cx, |this, cx| {
        this.set_center(center_item, window, cx);
        this.set_left_dock(left_item, Some(px(260.)), true, window, cx);
        this.set_right_dock(right_item, Some(px(280.)), true, window, cx);
        // 左右边缘 dock 支持折叠（顶部/底部无边缘 dock）
        this.set_dock_collapsible(
            Edges {
                left: true,
                bottom: false,
                right: true,
                top: false,
            },
            window,
            cx,
        );
    });
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(title_bar::title_bar())
            // 主体：功能按钮栏 + Dock 布局
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.activity_bar(cx))
                    .child(self.dock_area.clone()),
            )
            .child(status_bar::status_bar())
    }
}
