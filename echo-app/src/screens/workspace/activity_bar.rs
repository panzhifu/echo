//! 功能按钮栏（Activity Bar）。
//!
//! 位于工作区最左侧的一排垂直图标按钮，类似 VS Code 的 Activity Bar。
//! 点击按钮切换左侧 Dock 面板内容；再次点击已激活的按钮折叠/展开。

use gpui_component::IconName;

/// 功能按钮栏的工具项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityTool {
    /// 文件树。
    Files,
    /// 搜索。
    Search,
    /// 图谱。
    Graph,
    /// 设置。
    Settings,
}

impl ActivityTool {
    /// 全部工具项（按显示顺序）。
    pub const ALL: [Self; 4] = [Self::Files, Self::Search, Self::Graph, Self::Settings];

    /// 按钮元素 ID（在 `DockArea` 内唯一）。
    pub fn id(self) -> &'static str {
        match self {
            Self::Files => "tool-files",
            Self::Search => "tool-search",
            Self::Graph => "tool-graph",
            Self::Settings => "tool-settings",
        }
    }

    /// 按钮图标。
    pub fn icon(self) -> IconName {
        match self {
            Self::Files => IconName::FolderOpen,
            Self::Search => IconName::Search,
            Self::Graph => IconName::Globe,
            Self::Settings => IconName::Settings,
        }
    }

    /// 悬浮提示文本。
    pub fn label(self) -> &'static str {
        match self {
            Self::Files => "文件",
            Self::Search => "搜索",
            Self::Graph => "图谱",
            Self::Settings => "设置",
        }
    }
}
