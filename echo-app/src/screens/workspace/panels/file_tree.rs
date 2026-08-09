use std::path::{Path, PathBuf};
use std::time::Duration;

use echo_vault::{VaultEvent, VaultWatcher, WatchGuard};
use futures::StreamExt;
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled as _, Task, Window, div, px,
};
use gpui_component::ActiveTheme as _;
use gpui_component::button::Button;
use gpui_component::dock::{Panel, PanelEvent};
use gpui_component::input::{Input, InputState};
use gpui_component::list::ListItem;
use gpui_component::tree::{Tree, TreeItem, TreeState};
use gpui_component::{Icon, IconName, WindowExt, h_flex};

/// 新建条目的类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NewEntryKind {
    File,
    Folder,
}

/// 文件树面板：位于 Dock 左侧边缘，展示 vault 目录结构。
///
/// 使用 [`Tree`] 组件渲染递归扫描出的目录树，文件夹可展开/折叠。
/// 通过 [`VaultWatcher`] 监听文件夹变化，文件系统变更时自动刷新。
/// 标题栏提供新建文件/新建文件夹功能。
pub struct FileTreePanel {
    focus_handle: FocusHandle,
    root_path: PathBuf,
    tree_state: Entity<TreeState>,
    /// 新建条目的目标目录（点击树中的文件夹时更新，默认根目录）。
    selected_dir: PathBuf,
    /// 保持文件监控存活；面板销毁时自动停止。
    watch_guard: Option<WatchGuard>,
    /// 保持事件消费任务存活。
    watch_task: Task<()>,
}

impl FileTreePanel {
    pub fn new(root_path: String, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_path = PathBuf::from(root_path);
        let tree_state = cx.new(|cx| TreeState::new(cx));

        let mut panel = Self {
            focus_handle: cx.focus_handle(),
            root_path: root_path.clone(),
            selected_dir: root_path.clone(),
            tree_state: tree_state.clone(),
            watch_guard: None,
            watch_task: cx.spawn(async |_, _| {}),
        };

        // 同步扫描目录构建文件树（后续可改为异步加载）
        panel.reload(cx);
        // 监听文件夹变化，自动刷新文件树
        panel.start_watching(cx);
        panel
    }

    /// 重新扫描目录并刷新文件树。
    fn reload<C: AppContext>(&mut self, cx: &mut C) {
        let items = build_file_items(&self.root_path);
        self.tree_state.update(cx, |state, cx| {
            state.set_items(items, cx);
        });
    }

    /// 启动文件夹监听：vault 目录变化时自动重新扫描。
    ///
    /// 监听失败（如路径不存在）时降级为仅手动刷新，不影响面板使用。
    fn start_watching(&mut self, cx: &mut Context<Self>) {
        let watcher = VaultWatcher::new(&self.root_path)
            .ignore_patterns(vec![".git/".to_string(), "*.tmp".to_string()])
            .debounce(Duration::from_millis(200));

        let Ok((event_rx, guard)) = watcher.watch() else {
            log::warn!("failed to watch vault: {}", self.root_path.display());
            return;
        };

        // 持有 guard，否则监听在函数返回后立即停止
        self.watch_guard = Some(guard);

        // 后台线程：阻塞接收 std mpsc 事件，转发到异步 channel
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<VaultEvent>();
        cx.background_executor()
            .spawn(async move {
                while let Ok(event) = event_rx.recv() {
                    if tx.unbounded_send(event).is_err() {
                        break;
                    }
                }
            })
            .detach();

        // 主线程：消费事件，刷新文件树（第一个参数为面板弱引用）
        self.watch_task = cx.spawn(async move |this, cx| {
            while let Some(_event) = rx.next().await {
                let _ = this.update(cx, |panel, cx| {
                    panel.reload(cx);
                });
            }
        });
    }
    /// 在目标目录下创建文件/文件夹，并刷新文件树。
    fn create_entry<C: AppContext>(
        &mut self,
        kind: NewEntryKind,
        target: &Path,
        name: &str,
        cx: &mut C,
    ) {
        let path = target.join(name);
        let result = match kind {
            NewEntryKind::File => std::fs::File::create(&path).map(|_| ()),
            NewEntryKind::Folder => std::fs::create_dir_all(&path),
        };

        match result {
            Ok(()) => {
                log::info!("created {}", path.display());
                self.reload(cx);
            },
            Err(e) => {
                log::error!("failed to create {}: {e}", path.display());
            },
        }
    }
}

/// 打开新建条目对话框：输入名称，确定后在目标目录创建。
fn open_create_dialog(
    panel: &Entity<FileTreePanel>,
    target: &Path,
    kind: NewEntryKind,
    window: &mut Window,
    cx: &mut App,
) {
    let (title, placeholder) = match kind {
        NewEntryKind::File => ("新建文件", "文件名，如 notes.md"),
        NewEntryKind::Folder => ("新建文件夹", "文件夹名"),
    };

    let input = cx.new(|cx| {
        let mut input = InputState::new(window, cx);
        input.set_placeholder(placeholder, window, cx);
        input
    });

    let panel = panel.clone();
    let target = target.to_path_buf();
    window.open_alert_dialog(cx, move |alert, _window, _cx| {
        let input_state = input.clone();
        let target = target.clone();
        alert
            .title(title)
            .description(format!("创建到：{}", target.display()))
            .content({
                let input_state = input_state.clone();
                move |content, _window, _cx| content.child(Input::new(&input_state))
            })
            .on_ok({
                let input_state = input_state.clone();
                let target = target.clone();
                let panel = panel.clone();
                move |_, _window, cx| {
                    let name = input_state.read(cx).value().to_string();
                    if name.trim().is_empty() {
                        return false;
                    }
                    panel.update(cx, |panel, cx| {
                        panel.create_entry(kind, &target, &name, cx);
                    });
                    true
                }
            })
    });
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

    /// 不显示面板标题。
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }

    /// 标题栏按钮：新建文件 / 新建文件夹。
    ///
    /// 文件系统变化由后台监听自动刷新，无需手动刷新按钮。
    fn toolbar_buttons(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        let this = cx.entity();
        let target = self.selected_dir.clone();

        Some(vec![
            Button::new("new-file")
                .icon(IconName::Plus)
                .tooltip("新建文件")
                .on_click({
                    let this = this.clone();
                    let target = target.clone();
                    move |_, window, cx| {
                        open_create_dialog(&this, &target, NewEntryKind::File, window, cx);
                    }
                }),
            Button::new("new-folder")
                .icon(IconName::Folder)
                .tooltip("新建文件夹")
                .on_click({
                    let this = this.clone();
                    let target = target.clone();
                    move |_, window, cx| {
                        open_create_dialog(&this, &target, NewEntryKind::Folder, window, cx);
                    }
                }),
        ])
    }
}

impl Render for FileTreePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let tree_state = self.tree_state.clone();

        Tree::new(&tree_state, move |ix, entry, selected, _window, cx| {
            let item = entry.item();
            let icon = if !entry.is_folder() {
                IconName::File
            } else if entry.is_expanded() {
                IconName::FolderOpen
            } else {
                IconName::Folder
            };

            // 记录选中文件路径（后续可在此打开编辑器）
            let path = item.id.clone();
            let this = this.clone();

            ListItem::new(ix)
                .selected(selected)
                .w_full()
                .rounded(cx.theme().radius)
                .py_0p5()
                .px_2()
                .pl(px(16.) * entry.depth() + px(8.))
                .child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(icon).size_4())
                        .child(item.label.clone()),
                )
                .on_click({
                    let this = this.clone();
                    move |_, _window, cx| {
                        // 点击文件夹时将其设为新建条目的目标目录
                        let path = PathBuf::from(path.to_string());
                        this.update(cx, |panel, _cx| {
                            if path.is_dir() {
                                panel.selected_dir.clone_from(&path);
                            }
                            log::info!("selected: {}", path.display());
                        });
                    }
                })
        })
        .text_sm()
        .p_1()
        .h_full()
        .bg(cx.theme().sidebar)
        .text_color(cx.theme().sidebar_foreground)
    }
}

/// 递归扫描目录构建文件树条目。
///
/// 规则：
/// - 跳过隐藏文件/目录（以 `.` 开头，如 `.git`）
/// - 文件夹在前，文件在后，各自按名称升序排列
fn build_file_items(path: &Path) -> Vec<TreeItem> {
    let mut items = Vec::new();

    let Ok(entries) = std::fs::read_dir(path) else {
        return items;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let file_name = entry.file_name();

        // 跳过隐藏文件/目录（如 .git、.DS_Store）
        if file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let id = entry_path.to_string_lossy().to_string();
        let label = file_name.to_string_lossy().to_string();

        if entry_path.is_dir() {
            let children = build_file_items(&entry_path);
            items.push(TreeItem::new(id, label).children(children));
        } else {
            items.push(TreeItem::new(id, label));
        }
    }

    items.sort_by(|a, b| {
        b.is_folder()
            .cmp(&a.is_folder())
            .then_with(|| a.label.cmp(&b.label))
    });
    items
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_vault() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("echo-file-tree-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::write(dir.join("README.md"), "# test").unwrap();
        fs::write(dir.join("src").join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.join("src").join("lib.rs"), "").unwrap();
        dir
    }

    #[test]
    fn build_items_puts_folders_first() {
        let dir = setup_temp_vault();
        let items = build_file_items(&dir);
        let _ = fs::remove_dir_all(&dir);

        // 只有 src 文件夹 + README.md（.git 被跳过）
        assert_eq!(items.len(), 2);
        assert!(items[0].is_folder());
        assert_eq!(items[0].label, "src");
        assert!(!items[1].is_folder());
        assert_eq!(items[1].label, "README.md");
    }

    #[test]
    fn build_items_skips_hidden_dirs() {
        let dir = setup_temp_vault();
        let items = build_file_items(&dir);
        let _ = fs::remove_dir_all(&dir);

        assert!(!items.iter().any(|i| i.label == ".git"));
    }

    #[test]
    fn build_items_recurses_into_subdirs() {
        let dir = setup_temp_vault();
        let items = build_file_items(&dir);
        let _ = fs::remove_dir_all(&dir);

        let src = items.iter().find(|i| i.label == "src").unwrap();
        assert_eq!(src.children.len(), 2);
        assert!(src.children.iter().any(|c| c.label == "main.rs"));
        assert!(src.children.iter().any(|c| c.label == "lib.rs"));
    }

    #[test]
    fn build_items_handles_missing_dir() {
        let items = build_file_items(Path::new("/nonexistent/echo/vault"));
        assert!(items.is_empty());
    }
}
