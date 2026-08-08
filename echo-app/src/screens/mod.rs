//! 应用界面模块。
//!
//! 每个子模块对应一个完整的界面（Screen），
//! 界面及其专属组件都放在同一目录下，便于整体维护：
//!
//! - [`vault_manager`] — 仓库选择界面（首次启动 / 未配置仓库时）
//! - [`workspace`] — 工作区界面（已配置仓库后）

pub mod vault_manager;
pub mod workspace;
