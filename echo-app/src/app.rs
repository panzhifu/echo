use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled as _, Subscription,
    Window, div,
};
use gpui_component::ActiveTheme as _;

use crate::app_logic::AppState;
use crate::screens::vault_manager::VaultManagerView;
use crate::screens::workspace::WorkspaceView;
use echo_core::config::ConfigData;

/// Echo 应用的主结构体。
///
/// 持有响应式配置实体，启动时根据仓库配置决定界面：
/// - 无仓库 → 仓库管理界面
/// - 有仓库 → 工作区界面（Dock 布局）
///
/// 仓库管理界面写入配置后，通过观察配置变化自动切换到工作区界面。
pub struct EchoApp {
    config: Entity<ConfigData>,
    state: AppState,
    /// 未配置仓库时持有的仓库管理视图实体。
    vault_manager: Option<Entity<VaultManagerView>>,
    /// 工作区视图实体（Dock 布局）。
    workspace: Option<Entity<WorkspaceView>>,
    /// 保持订阅存活，避免被 drop 后取消订阅（无需读取）。
    #[expect(dead_code)]
    subscriptions: Vec<Subscription>,
}

impl EchoApp {
    /// 创建一个新的 [`EchoApp`] 实例。
    ///
    /// `config_data` 为已加载的配置数据，避免重复加载。
    /// 根据仓库配置决定初始界面。
    pub fn new(window: &mut Window, cx: &mut Context<Self>, config_data: ConfigData) -> Self {
        let config = cx.new(|_| config_data);

        let state = AppState::from_vault(&config.read(cx).vault);
        match state {
            AppState::VaultLoaded => log::info!("Vault configured: entering workspace"),
            AppState::NoVault => log::info!("No vault configured: showing vault manager"),
        }

        let vault_manager = if matches!(state, AppState::NoVault) {
            Some(cx.new(|cx| VaultManagerView::new(window, cx, config.clone())))
        } else {
            None
        };

        // 提前创建工作区视图：观察回调拿不到 window，无法在那里惰性创建。
        // 即使当前处于 NoVault，创建后也不会渲染。
        let vault_path = config.read(cx).vault.path.clone();
        let workspace = Some(cx.new(|cx| WorkspaceView::new(window, cx, vault_path.as_deref())));

        // 订阅配置变化：仓库变为有效时切换到工作区界面
        let subscriptions = vec![cx.observe(&config, |this, _, cx| {
            if matches!(this.state, AppState::NoVault) && this.config.read(cx).vault.is_valid() {
                log::info!("Vault configured via UI: switching to workspace");
                this.state = AppState::VaultLoaded;
                cx.notify();
            }
        })];

        Self {
            config,
            state,
            vault_manager,
            workspace,
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
                // 工作区界面：Dock 布局
                if let Some(workspace) = &self.workspace {
                    root = root.child(workspace.clone());
                }
            },
        }
        root
    }
}
