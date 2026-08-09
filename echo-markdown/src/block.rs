//! Block 块模型。

use crate::inline::InlineTextTree;

/// 块类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    /// 段落。
    Paragraph,
    /// 标题，level 1-6（含 ATX `#` 与 Setext `===` / `---`）。
    Heading {
        /// 标题层级。
        level: u8,
    },
    /// 围栏或缩进代码块。
    CodeBlock {
        /// 代码语言（围栏信息字符串）。
        language: Option<String>,
    },
    /// 无序列表项（`-` / `*` / `+`）。
    BulletedListItem,
    /// 任务列表项（`- [ ]` / `- [x]`）。
    TaskListItem {
        /// 是否勾选。
        checked: bool,
    },
    /// 有序列表项（`1.`）。
    NumberedListItem {
        /// 序号。
        ordinal: usize,
    },
    /// 引用块（`>`）。
    BlockQuote,
    /// 表格。
    Table,
    /// 分隔线（`---` / `***` / `___`）。
    ThematicBreak,
    /// 脚注定义（`[^label]: content`）。
    FootnoteDefinition {
        /// 脚注标签。
        label: String,
    },
    /// 定义列表（Term / Description 对）。
    DefinitionList,
    /// 定义列表标题（术语）。
    DefinitionTerm,
    /// 定义列表描述。
    DefinitionDescription,
    /// 数学块（`$$...$$`）。
    MathBlock,
    /// HTML 块。
    HtmlBlock,
    /// Callout（`> [!NOTE]` / `> [!WARNING]` 等，Obsidian 风格）。
    Callout {
        /// Callout 变体类型。
        variant: CalloutVariant,
        /// Callout 标题。
        title: String,
        /// 折叠状态（`> [!NOTE]-` 折叠，`> [!NOTE]+` 展开，`None` 无折叠）。
        folded: Option<bool>,
    },
    /// YAML frontmatter（`---\n...\n---`）。
    Frontmatter,
    /// Mermaid 图表（` ```mermaid ` 代码块）。
    Mermaid,
}

/// Callout 变体类型。
///
/// 覆盖 Obsidian 内置的所有 callout 类型及其别名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalloutVariant {
    /// `[!note]` — 提示。
    Note,
    /// `[!info]` / `[!todo]` — 信息 / 待办。
    Info,
    /// `[!tip]` / `[!hint]` / `[!important]` — 技巧 / 提示 / 重要。
    Tip,
    /// `[!warning]` / `[!caution]` / `[!attention]` — 警告 / 注意 / 关注。
    Warning,
    /// `[!danger]` / `[!error]` — 危险 / 错误。
    Danger,
    /// `[!success]` / `[!check]` / `[!done]` — 成功 / 完成。
    Success,
    /// `[!question]` / `[!help]` / `[!faq]` — 问题 / 帮助。
    Question,
    /// `[!abstract]` / `[!summary]` / `[!tldr]` — 摘要。
    Abstract,
    /// `[!quote]` / `[!cite]` — 引用。
    Quote,
    /// `[!bug]` — Bug。
    Bug,
    /// `[!example]` — 示例。
    Example,
    /// `[!failure]` / `[!fail]` / `[!missing]` — 失败。
    Failure,
    /// 其他自定义变体。
    Other(String),
}

/// 一个文档块。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// 块类型。
    pub kind: BlockKind,
    /// 内联内容（段落 / 标题 / 列表项文本）。
    pub title: InlineTextTree,
    /// 嵌套子块（列表项 / 引用的内容）。
    pub children: Vec<Block>,
    /// 代码块原文（仅 `CodeBlock`）。
    pub code: Option<String>,
    /// 表格数据（仅 `Table`）。
    pub table: Option<TableData>,
    /// Obsidian 块 ID（`^blockid` 语法，如 `[[page#^blockid]]`）。
    ///
    /// 用于块级引用，在序列化时输出为 `^blockid` 附加在块末尾。
    pub block_id: Option<String>,
}

impl Block {
    /// 创建指定类型的空块。
    #[must_use]
    pub fn new(kind: BlockKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            kind: BlockKind::Paragraph,
            title: InlineTextTree::new(),
            children: Vec::new(),
            code: None,
            table: None,
            block_id: None,
        }
    }
}

/// 表格数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableData {
    /// 表头单元格。
    pub headers: Vec<TableCell>,
    /// 数据行。
    pub rows: Vec<Vec<TableCell>>,
}

/// 表格单元格。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    /// 单元格内联内容。
    pub content: InlineTextTree,
    /// 列对齐方式（来自分隔行 `:---`）。
    pub align: Option<TableAlign>,
}

/// 表格列对齐。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlign {
    /// 左对齐。
    Left,
    /// 居中。
    Center,
    /// 右对齐。
    Right,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;

    #[test]
    fn new_block_is_empty() {
        let block = Block::new(BlockKind::Paragraph);
        assert!(block.title.is_empty());
        assert!(block.children.is_empty());
        assert!(block.code.is_none());
        assert!(block.table.is_none());
    }

    #[test]
    fn heading_kind_carries_level() {
        let block = Block::new(BlockKind::Heading { level: 2 });
        match block.kind {
            BlockKind::Heading { level } => assert_eq!(level, 2),
            _ => panic!("expected Heading"),
        }
    }
}
