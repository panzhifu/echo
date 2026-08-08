use gpui::{
    App, AppContext, AsyncApp, ClickEvent, Context, Div, ParentElement, Styled as _, WeakEntity,
    Window, div,
};
use gpui_component::button::Button;
use gpui_component::{ActiveTheme as _, IconName, v_flex};

use super::VaultManagerView;

/// 渲染右侧功能区面板。
///
/// 包含两个功能区卡片（每个：左侧文字说明 + 右侧按钮）：
/// - 新建仓库
/// - 打开已有仓库
pub fn right_panel(cx: &mut Context<VaultManagerView>) -> Div {
    let create_button = Button::new("create-vault")
        .label("新建仓库")
        .icon(IconName::Plus)
        .on_click(pick_folder(cx));
    let open_button = Button::new("open-vault")
        .label("打开仓库")
        .icon(IconName::FolderOpen)
        .on_click(pick_folder(cx));

    v_flex()
        .size_full()
        .flex_1()
        .min_w_0()
        .p_6()
        .gap_4()
        .child(section(
            cx,
            "新建仓库".to_string(),
            "创建新的笔记仓库，选择一个文件夹作为仓库目录".to_string(),
            create_button,
        ))
        .child(section(
            cx,
            "打开仓库".to_string(),
            "选择已经存在的仓库文件夹".to_string(),
            open_button,
        ))
}

/// 渲染一个功能区卡片：左侧文字说明 + 右侧按钮。
fn section(
    cx: &mut Context<VaultManagerView>,
    title: String,
    description: String,
    button: Button,
) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .p_4()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        // 左侧：文字说明
        .child(
            v_flex()
                .gap_1()
                .child(div().text_base().child(title))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(description),
                ),
        )
        // 右侧：操作按钮
        .child(button)
}

/// 生成一个通过 rfd 选择文件夹并写入配置的点击处理器。
fn pick_folder(
    cx: &mut Context<VaultManagerView>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    cx.listener(|_, _, _window, cx| {
        // 在后台线程打开系统文件夹选择对话框
        let task = cx.background_spawn(async move { rfd::FileDialog::new().pick_folder() });
        // 后台任务结果通过 entity 更新，无需持有句柄
        cx.spawn(|this: WeakEntity<VaultManagerView>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                if let Some(path) = task.await {
                    let path = path.to_string_lossy().into_owned();
                    this.update(&mut cx, |this, cx| {
                        // 写入响应式配置，触发应用切换到工作区界面
                        this.config.update(cx, |data, cx| {
                            data.vault.path = Some(path.clone());
                            data.vault.add_recent(path);
                            cx.notify();
                        });
                        // 持久化到磁盘
                        let config = this.config.read(cx).clone();
                        let _ = echo_core::config::save_config_to_default(&config);
                        cx.notify();
                    })
                    .ok();
                }
            }
        })
        .detach();
    })
}
