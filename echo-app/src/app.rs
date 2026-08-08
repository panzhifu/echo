use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled as _, Subscription,
    Window, div,
};
use gpui_component::ActiveTheme as _;

use crate::screens::vault_manager::VaultManagerView;
use crate::screens::workspace::workspace_view;
use echo_core::config::ConfigData;

/// 应用状态：是否已加载仓库。
enum AppState {
    /// 未配置仓库，显示仓库管理界面。
    NoVault,
    /// 已配置仓库，显示工作区界面。
    VaultLoaded,
}

/// Echo 应用的主结构体。
///
/// 持有响应式配置实体，启动时根据仓库配置决定界面：
/// - 无仓库 → 仓库管理界面
/// - 有仓库 → 工作区界面
///
/// 仓库管理界面写入配置后，通过观察配置变化自动切换到工作区界面。
pub struct EchoApp {
    config: Entity<ConfigData>,
    state: AppState,
    /// 未配置仓库时持有的仓库管理视图实体。
    vault_manager: Option<Entity<VaultManagerView>>,
    /// 保持订阅存活，避免被 drop 后取消订阅（无需读取）。
    #[expect(dead_code)]
    subscriptions: Vec<Subscription>,
}

impl EchoApp {
    /// 创建一个新的 [`EchoApp`] 实例。
    ///
    /// 启动时加载配置，判断是否已配置仓库。
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config_data = echo_core::config::load_config(None).unwrap_or_default();
        let config = cx.new(|_| config_data);

        let state = if config.read(cx).vault.is_valid() {
            AppState::VaultLoaded
        } else {
            AppState::NoVault
        };

        let vault_manager = if matches!(state, AppState::NoVault) {
            Some(cx.new(|cx| VaultManagerView::new(window, cx, config.clone())))
        } else {
            None
        };

        // 订阅配置变化：仓库变为有效时切换到工作区界面
        let subscriptions = vec![cx.observe(&config, |this, _, cx| {
            if matches!(this.state, AppState::NoVault) && this.config.read(cx).vault.is_valid() {
                this.state = AppState::VaultLoaded;
                cx.notify();
            }
        })];

        Self {
            config,
            state,
            vault_manager,
            subscriptions,
        }
    }
}

impl Render for EchoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().size_full().bg(cx.theme().background);

        match self.state {
            AppState::NoVault => {
                // 仓库选择界面：直接渲染，无标题栏
                if let Some(vault_manager) = &self.vault_manager {
                    root = root.child(vault_manager.clone());
                }
            },
            AppState::VaultLoaded => {
                // 工作区界面：包含标题栏
                root = root.child(workspace_view(cx));
            },
        }
        root
    }
}
