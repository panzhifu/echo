//! Dock 面板模块。
//!
//! 每个面板实现 [`gpui_component::dock::Panel`] trait，
//! 由 `DockArea` 统一管理布局（分割 / 标签 / 折叠）。

pub mod editor;
pub mod file_tree;
pub mod graph;
pub mod outline;
pub mod search;
pub mod settings;

pub use editor::EditorPanel;
pub use file_tree::FileTreePanel;
pub use graph::GraphPanel;
pub use outline::OutlinePanel;
pub use search::SearchPanel;
pub use settings::SettingsPanel;
