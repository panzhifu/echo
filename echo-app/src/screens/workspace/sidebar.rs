use gpui::{IntoElement, ParentElement, Styled as _};
use gpui_component::sidebar::{
    Sidebar, SidebarCollapsible, SidebarFooter, SidebarGroup, SidebarMenu, SidebarMenuItem,
};
use gpui_component::{Icon, IconName};

pub fn sidebar() -> impl IntoElement {
    // 主要菜单项
    let main_items = [
        ("Dashboard", IconName::LayoutDashboard, true),
        ("Analytics", IconName::ChartPie, false),
        ("Reports", IconName::File, false),
    ];

    // 设置菜单项
    let settings_items = [
        ("Settings", IconName::Settings, false),
        ("Help", IconName::Info, false),
    ];

    Sidebar::new("app-sidebar")
        // 固定为折叠（图标模式）
        .collapsible(SidebarCollapsible::Icon)
        .collapsed(true)
        // 主要分组
        .child(
            SidebarGroup::new("Main").child(
                SidebarMenu::new().children(main_items.map(|(label, icon, active)| {
                    SidebarMenuItem::new(label)
                        .icon(icon)
                        .active(active)
                })),
            ),
        )
        // 设置分组
        .child(
            SidebarGroup::new("Settings").child(
                SidebarMenu::new().children(settings_items.map(|(label, icon, active)| {
                    SidebarMenuItem::new(label)
                        .icon(icon)
                        .active(active)
                })),
            ),
        )
        // 底部：仅显示用户头像图标
        .footer(
            SidebarFooter::new().child(
                Icon::new(IconName::CircleUser).size_5()
            ),
        )
}
