//! Inline 内联内容模型。
//!
//! 采用扁平 `Vec<InlineFragment>` + `InlineStyle` bitfield 表示，
//! 而非递归 AST。嵌套格式（如 `**bold _italic_**`）通过 OR 组合
//! `InlineStyle` 标记到重叠区域的单个 fragment，便于未来增量编辑
//! （切片、就地更新）与序列化。

/// Inline 内容树：扁平 fragment 列表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineTextTree {
    /// 有序的内联片段。
    pub fragments: Vec<InlineFragment>,
}

impl InlineTextTree {
    /// 创建空树。
    #[must_use]
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
        }
    }

    /// 从单个纯文本片段构建。
    #[must_use]
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            fragments: vec![InlineFragment {
                text: text.into(),
                style: InlineStyle::default(),
                attachment: None,
            }],
        }
    }

    /// 是否为空（无片段或所有片段文本为空）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fragments.iter().all(|f| f.text.is_empty())
    }

    /// 追加一个片段。
    pub fn push(&mut self, fragment: InlineFragment) {
        self.fragments.push(fragment);
    }

    /// 拼接所有片段的可见文本（不含格式标记）。
    #[must_use]
    pub fn plain_text(&self) -> String {
        self.fragments.iter().map(|f| f.text.as_str()).collect()
    }
}

impl Default for InlineTextTree {
    fn default() -> Self {
        Self::new()
    }
}

/// 单个内联片段：文本 + 样式 + 可选附件（链接/图片）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineFragment {
    /// 片段文本。
    pub text: String,
    /// 片段样式（bold / italic / strikethrough / code）。
    pub style: InlineStyle,
    /// 若为链接或图片，承载附件信息。
    pub attachment: Option<InlineAttachment>,
}

/// 内联样式 bitfield。
///
/// 嵌套格式通过 [`InlineStyle::merge`] 组合，
/// 例如 `**bold _italic_**` 重叠区域得到 `bold=true && italic=true`。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct InlineStyle {
    /// 粗体 `**text**`。
    pub bold: bool,
    /// 斜体 `*text*`。
    pub italic: bool,
    /// 删除线 `~~text~~`。
    pub strikethrough: bool,
    /// 行内代码 `` `code` ``。
    pub code: bool,
    /// 高亮 `==text==`。
    pub highlight: bool,
}

impl InlineStyle {
    /// 无样式。
    #[must_use]
    pub const fn none() -> Self {
        Self {
            bold: false,
            italic: false,
            strikethrough: false,
            code: false,
            highlight: false,
        }
    }

    /// 与另一个样式 OR 组合（用于嵌套格式）。
    #[must_use]
    pub const fn merge(self, other: InlineStyle) -> Self {
        Self {
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
            strikethrough: self.strikethrough || other.strikethrough,
            code: self.code || other.code,
            highlight: self.highlight || other.highlight,
        }
    }
}

/// 链接或图片附件类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineAttachment {
    /// 行内链接 `[text](url)`。
    Link {
        /// 链接目标 URL。
        destination: String,
        /// 可选标题 `title`。
        title: Option<String>,
    },
    /// 引用链接 `[text][label]`。
    Reference {
        /// 引用标签。
        label: String,
    },
    /// 自动链接 `<url>`。
    Autolink {
        /// 自动链接目标。
        target: String,
    },
    /// Obsidian `WikiLink` `[[target]]` 或 `[[target|alias]]`。
    ///
    /// 片段的 `text` 字段承载显示文本（alias 或 target）。
    WikiLink {
        /// 链接目标（笔记名）。
        target: String,
        /// 可选别名（显示文本）。
        alias: Option<String>,
        /// 可选标题链接（`[[page#heading]]`）。
        heading: Option<String>,
        /// 可选块链接（`[[page#^blockid]]`）。
        block_id: Option<String>,
    },
    /// 图片 `![alt](url)`。
    ///
    /// 替代文本由 fragment 的 `text` 字段承载。
    Image {
        /// 图片 URL。
        destination: String,
        /// 可选标题 `title`。
        title: Option<String>,
        /// 可选宽度（Obsidian `![alt|100](url)` 语法）。
        width: Option<u32>,
    },
    /// 脚注引用 `[^label]`。
    FootnoteRef {
        /// 脚注标签。
        label: String,
    },
    /// 行内数学公式 `$...$`。
    MathInline {
        /// 数学公式内容。
        content: String,
    },
    /// 行内 HTML。
    InlineHtml {
        /// HTML 内容。
        content: String,
    },
    /// 标签 `#tag`。
    Tag {
        /// 标签名称（不含 `#`）。
        name: String,
    },
    /// Obsidian 嵌入 `![[target]]`（图片 / 文件 / 笔记）。
    Embed {
        /// 嵌入目标（文件名或笔记名）。
        target: String,
        /// 嵌入类型。
        kind: EmbedKind,
        /// 可选别名（`![[target|alias]]`）。
        alias: Option<String>,
        /// 可选宽度（`![[image|200]]`）。
        width: Option<u32>,
    },
    /// Obsidian 注释 `%%comment%%`。
    Comment {
        /// 注释内容。
        content: String,
    },
}

/// 嵌入类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbedKind {
    /// 图片嵌入 `![[image.png]]`。
    Image,
    /// 文件嵌入 `![[file.pdf]]`。
    File,
    /// 笔记嵌入 `![[page]]`。
    Note,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_is_empty() {
        assert!(InlineTextTree::new().is_empty());
        assert!(InlineTextTree::default().is_empty());
    }

    #[test]
    fn from_text_plain_text() {
        let tree = InlineTextTree::from_text("hello");
        assert_eq!(tree.plain_text(), "hello");
        assert!(!tree.is_empty());
    }

    #[test]
    fn style_merge_or_combines_flags() {
        let a = InlineStyle {
            bold: true,
            ..InlineStyle::none()
        };
        let b = InlineStyle {
            italic: true,
            ..InlineStyle::none()
        };
        let merged = a.merge(b);
        assert!(merged.bold);
        assert!(merged.italic);
        assert!(!merged.strikethrough);
        assert!(!merged.code);
    }

    #[test]
    fn plain_text_concatenates_fragments() {
        let mut tree = InlineTextTree::new();
        tree.push(InlineFragment {
            text: "foo".to_string(),
            style: InlineStyle::none(),
            attachment: None,
        });
        tree.push(InlineFragment {
            text: "bar".to_string(),
            style: InlineStyle::none(),
            attachment: None,
        });
        assert_eq!(tree.plain_text(), "foobar");
    }
}
