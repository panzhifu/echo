//! `Markdown` 解析：`pulldown-cmark` `Event` 流 -> `Document` 块树。
//!
//! 遍历 [`pulldown_cmark::Parser`] 产生的事件流，用容器栈折叠成块树。
//! 容器（`BlockQuote` / `List` / `Item`）在 `Start` 时压入新层，`End` 时弹出并
//! 包装为对应块挂回父层；叶块（`Paragraph` / `Heading` / `CodeBlock` / `TableCell`）
//! 在期间收集 inline 片段。`[[...]]` `WikiLink` 在 inline 收集阶段后处理拆分。

use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd,
};

use crate::block::{Block, BlockKind, CalloutVariant, TableAlign, TableCell, TableData};
use crate::document::Document;
use crate::inline::{InlineAttachment, InlineFragment, InlineStyle, InlineTextTree};
use crate::wikilink::{
    CommentSegment, HighlightSegment, TagSegment, WikiSegment, split_comments, split_highlights,
    split_tags, split_wikilinks,
};
use echo_core::MarkdownResult;

/// 解析 `Markdown` 文本为块树。
///
/// 开启 table / strikethrough / `task_lists` / footnotes / `definition_list` / math 选项；
/// `[[target|alias]]` `WikiLink` 在 inline 收集阶段后处理拆分。
///
/// # Errors
///
/// 当前实现基于 `pulldown-cmark`，解析高度容错，实际不返回错误；
/// 返回 `Result` 以为未来编辑运行时与 IO 预留统一出口。
pub fn parse(markdown: &str) -> MarkdownResult<Document> {
    let (frontmatter, rest) = extract_frontmatter(markdown);

    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_MATH;
    let parser = Parser::new_ext(rest, options);

    let mut state = ParseState::new();
    for event in parser {
        state.handle(event);
    }
    let mut doc = state.finish();

    // 若存在 frontmatter，将其作为首个块插入。
    if let Some(yaml) = frontmatter {
        let mut blocks = Vec::with_capacity(doc.blocks.len() + 1);
        blocks.push(Block {
            kind: BlockKind::Frontmatter,
            title: InlineTextTree::new(),
            children: Vec::new(),
            code: Some(yaml),
            table: None,
            block_id: None,
        });
        blocks.append(&mut doc.blocks);
        return Ok(Document::from_blocks(blocks));
    }
    Ok(doc)
}

/// 预处理：提取文档开头的 YAML frontmatter（`---\n...\n---`）。
///
/// 若文档以 `---` 开头且存在配对的闭合 `---`，返回 YAML 内容与剩余文本；
/// 否则返回 `None` 与原文本。
fn extract_frontmatter(markdown: &str) -> (Option<String>, &str) {
    let text = markdown.trim_start_matches('\u{feff}');

    // 收集每行的 (起始偏移, 结束偏移, 内容)
    let mut line_ranges: Vec<(usize, usize, &str)> = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            line_ranges.push((start, i, &text[start..i]));
            start = i + 1;
        }
    }
    if start < text.len() {
        line_ranges.push((start, text.len(), &text[start..]));
    }

    // 第一行必须是 "---"（仅含空白）
    if line_ranges.is_empty() || line_ranges[0].2.trim() != "---" {
        return (None, markdown);
    }

    // 查找配对的闭合 "---"
    for (_, &(line_start, line_end, line)) in line_ranges.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            // YAML 内容在第一行之后、闭合行之前
            let yaml_start = line_ranges[0].1 + 1; // 跳过第一行的 \n
            let yaml_end = line_start;
            let yaml_content = &text[yaml_start..yaml_end];

            // 剩余文本在闭合行的 \n 之后
            let remaining = if line_end < text.len() {
                &text[line_end + 1..]
            } else {
                ""
            };

            return (Some(yaml_content.to_string()), remaining);
        }
    }

    (None, markdown)
}

struct ParseState {
    /// 容器栈，栈底为文档顶层块。
    container_stack: Vec<Vec<Block>>,
    /// 当前叶块 inline 收集（None 表示不在叶块 / 单元格内）。
    inline: Option<Vec<InlineFragment>>,
    /// inline 样式栈（Strong / Emphasis / Strikethrough）。
    style_stack: Vec<InlineStyle>,
    /// 链接 / 图片附件栈。
    attachment_stack: Vec<InlineAttachment>,
    /// 代码块缓冲与语言。
    code_buffer: String,
    code_lang: Option<String>,
    in_code: bool,
    /// 当前是否在 Image 内（用于解析 alt 文本中的宽度信息）。
    in_image: bool,
    /// 表格构建器。
    table: Option<TableBuilder>,
    /// 列表上下文。
    list_stack: Vec<ListContext>,
    /// 当前列表项的 task 勾选（若有）。
    task_marker: Option<bool>,
    /// 文本累积缓冲：`pulldown-cmark` 0.13 会把 `[[note|alias]]` 拆成
    /// 多个独立 `Text` 事件（`[` / `[` / `note|alias` / `]` / `]`），
    /// `WikiLink` 后处理必须在拼接后的完整文本上执行，因此先累积
    /// 连续的 `Text` 事件，在遇到非 `Text` 事件或块结束时统一 flush。
    text_buffer: String,
    /// 当前列表项嵌套深度。
    ///
    /// `pulldown-cmark` 0.13 不再用 `Paragraph` 包裹列表项内容，
    /// `Item` 内的 inline 文本直接出现。该计数用于在 `Item` 内遇到
    /// 子块（嵌套列表 / 代码块 / 引用 / 表格）时，先把已收集的 inline
    /// flush 为隐式 `Paragraph`，以便 `take_first_paragraph_title` 提取。
    /// 用计数而非 bool 以正确处理嵌套列表项。
    item_depth: usize,
    /// 当前脚注定义的标签（用于 `FootnoteDefinition` 容器）。
    footnote_label: Option<String>,
}

struct ListContext {
    ordered: bool,
    ordinal: usize,
}

struct TableBuilder {
    align: Vec<Option<TableAlign>>,
    headers: Vec<TableCell>,
    rows: Vec<Vec<TableCell>>,
    current_row: Vec<TableCell>,
    in_head: bool,
}

impl TableBuilder {
    fn new(align: Vec<Option<TableAlign>>) -> Self {
        Self {
            align,
            headers: Vec::new(),
            rows: Vec::new(),
            current_row: Vec::new(),
            in_head: false,
        }
    }
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            container_stack: vec![Vec::new()],
            inline: None,
            style_stack: Vec::new(),
            attachment_stack: Vec::new(),
            code_buffer: String::new(),
            code_lang: None,
            in_code: false,
            in_image: false,
            table: None,
            list_stack: Vec::new(),
            task_marker: None,
            text_buffer: String::new(),
            item_depth: 0,
            footnote_label: None,
        }
    }
}

impl ParseState {
    fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::too_many_lines)]
    fn handle(&mut self, event: Event) {
        // 非 Text 事件前先 flush 文本缓冲：pulldown-cmark 0.13 会把
        // `[[note|alias]]` 拆成多个独立 Text 事件，WikiLink 后处理
        // 必须在拼接后的完整文本上执行。
        if !matches!(event, Event::Text(_)) {
            self.flush_text_buffer();
        }
        // 列表项内遇到子块（嵌套列表 / 代码块 / 引用 / 表格）时，
        // 先把已收集的 inline flush 为隐式 Paragraph，保留为 children[0]，
        // 以便 End(Item) 时 take_first_paragraph_title 提取为 title。
        if self.item_depth > 0
            && self.inline.is_some()
            && matches!(
                event,
                Event::Start(Tag::List(_) | Tag::CodeBlock(_) | Tag::BlockQuote(_) | Tag::Table(_))
            )
        {
            self.flush_inline_as_paragraph();
        }
        match event {
            // 叶块开始：收集 inline
            Event::Start(
                Tag::Paragraph | Tag::Heading { .. } | Tag::TableCell | Tag::DefinitionListTitle,
            ) => {
                self.inline = Some(Vec::new());
            },
            Event::Start(Tag::CodeBlock(kind)) => {
                self.in_code = true;
                self.code_buffer.clear();
                self.code_lang = match kind {
                    CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.into_string()),
                    _ => None,
                };
            },
            // 容器开始
            Event::Start(Tag::BlockQuote(_)) => self.container_stack.push(Vec::new()),
            Event::Start(Tag::List(start)) => {
                let ordered = start.is_some();
                let ordinal = start.map_or(0, |n| usize::try_from(n).unwrap_or(0));
                self.list_stack.push(ListContext { ordered, ordinal });
            },
            Event::Start(Tag::Item) => {
                self.container_stack.push(Vec::new());
                // pulldown-cmark 0.13 不再用 Paragraph 包裹列表项内容，
                // Item 内的 inline 文本直接出现，因此启动 inline 收集。
                self.inline = Some(Vec::new());
                self.task_marker = None;
                self.item_depth += 1;
            },
            // 表格
            Event::Start(Tag::Table(aligns)) => {
                let align = aligns.into_iter().map(to_table_align).collect();
                self.table = Some(TableBuilder::new(align));
            },
            Event::Start(Tag::TableHead) => {
                if let Some(t) = self.table.as_mut() {
                    t.in_head = true;
                }
            },
            Event::Start(Tag::TableRow) => {
                if let Some(t) = self.table.as_mut() {
                    t.in_head = false;
                }
            },
            // inline 格式开始
            Event::Start(Tag::Emphasis) => self.style_stack.push(InlineStyle {
                italic: true,
                ..InlineStyle::none()
            }),
            Event::Start(Tag::Strong) => self.style_stack.push(InlineStyle {
                bold: true,
                ..InlineStyle::none()
            }),
            Event::Start(Tag::Strikethrough) => self.style_stack.push(InlineStyle {
                strikethrough: true,
                ..InlineStyle::none()
            }),
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            }) => {
                self.attachment_stack.push(make_link_attachment(
                    link_type,
                    dest_url.as_ref(),
                    title.as_ref(),
                ));
            },
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                self.in_image = true;
                self.attachment_stack.push(InlineAttachment::Image {
                    destination: dest_url.into_string(),
                    title: if title.is_empty() {
                        None
                    } else {
                        Some(title.into_string())
                    },
                    width: None,
                });
            },
            // 叶块结束
            Event::End(TagEnd::Paragraph) => self.finish_leaf(BlockKind::Paragraph),
            Event::End(TagEnd::Heading(level)) => {
                self.finish_leaf(BlockKind::Heading {
                    level: heading_level_to_u8(level),
                });
            },
            Event::End(TagEnd::CodeBlock) => {
                let language = self.code_lang.take();
                let kind = if language.as_deref() == Some("mermaid") {
                    BlockKind::Mermaid
                } else {
                    BlockKind::CodeBlock { language }
                };
                let block = Block {
                    kind,
                    title: InlineTextTree::new(),
                    children: Vec::new(),
                    code: Some(std::mem::take(&mut self.code_buffer)),
                    table: None,
                    block_id: None,
                };
                self.push_block(block);
                self.in_code = false;
            },
            // 容器结束
            Event::End(TagEnd::BlockQuote(_)) => {
                let children = self.container_stack.pop().unwrap_or_default();
                let block = Block {
                    kind: BlockKind::BlockQuote,
                    title: InlineTextTree::new(),
                    children,
                    code: None,
                    table: None,
                    block_id: None,
                };
                // 检测是否为 Callout（`> [!NOTE]`）并转换
                let block = try_make_callout(block);
                self.push_block(block);
            },
            Event::End(TagEnd::Item) => {
                self.item_depth = self.item_depth.saturating_sub(1);
                // 把 Item 内剩余 inline 包装为隐式 Paragraph，作为 children[0]。
                self.flush_inline_as_paragraph();
                let mut children = self.container_stack.pop().unwrap_or_default();
                let title = take_first_paragraph_title(&mut children);
                let kind = self.make_list_item_kind();
                self.push_block(Block {
                    kind,
                    title,
                    children,
                    code: None,
                    table: None,
                    block_id: None,
                });
            },
            Event::End(TagEnd::List(_)) => {
                self.list_stack.pop();
            },
            Event::End(TagEnd::Table) => {
                if let Some(t) = self.table.take() {
                    let data = TableData {
                        headers: t.headers,
                        rows: t.rows,
                    };
                    self.push_block(Block {
                        kind: BlockKind::Table,
                        title: InlineTextTree::new(),
                        children: Vec::new(),
                        code: None,
                        table: Some(data),
                        block_id: None,
                    });
                }
            },
            Event::End(TagEnd::TableHead | TagEnd::TableRow) => {
                if let Some(t) = self.table.as_mut() {
                    let row = std::mem::take(&mut t.current_row);
                    if t.in_head {
                        t.headers = row;
                    } else {
                        t.rows.push(row);
                    }
                }
            },
            Event::End(TagEnd::TableCell) => {
                if let Some(frags) = self.inline.take()
                    && let Some(t) = self.table.as_mut()
                {
                    let col = t.current_row.len();
                    let align = t.align.get(col).and_then(|a| *a);
                    t.current_row.push(TableCell {
                        content: InlineTextTree { fragments: frags },
                        align,
                    });
                }
            },
            // inline 格式结束
            Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough) => {
                self.style_stack.pop();
            },
            Event::End(TagEnd::Link) => {
                self.attachment_stack.pop();
            },
            Event::End(TagEnd::Image) => {
                self.in_image = false;
                // 解析 alt 文本中的宽度信息（`![alt|100](url)`）
                if let Some(att) = self.attachment_stack.last_mut()
                    && let InlineAttachment::Image { width, .. } = att
                {
                    // 从 inline 收集器中查找 Image 附件对应的文本片段
                    let combined: String = self
                        .inline
                        .as_ref()
                        .map(|frags| {
                            frags
                                .iter()
                                .filter(|f| {
                                    matches!(&f.attachment, Some(InlineAttachment::Image { .. }))
                                })
                                .map(|f| f.text.as_str())
                                .collect()
                        })
                        .unwrap_or_default();
                    if let Some(idx) = combined.rfind('|') {
                        let potential_width = combined[idx + 1..].trim();
                        if let Ok(w) = potential_width.parse::<u32>() {
                            *width = Some(w);
                        }
                    }
                }
                self.attachment_stack.pop();
            },
            // inline 内容
            Event::Text(text) => self.push_text(text.as_ref()),
            Event::Code(code) => {
                self.push_fragment(InlineFragment {
                    text: code.into_string(),
                    style: InlineStyle {
                        code: true,
                        ..InlineStyle::none()
                    },
                    attachment: None,
                });
            },
            Event::SoftBreak | Event::HardBreak => {
                self.push_fragment(InlineFragment {
                    text: "\n".to_string(),
                    style: self.current_style(),
                    attachment: None,
                });
            },
            Event::Rule => {
                self.push_block(Block::new(BlockKind::ThematicBreak));
            },
            Event::TaskListMarker(checked) => {
                self.task_marker = Some(checked);
            },
            // 脚注引用（inline）
            Event::FootnoteReference(label) => {
                let label = label.into_string();
                self.push_fragment(InlineFragment {
                    text: format!("[^{label}]"),
                    style: self.current_style(),
                    attachment: Some(InlineAttachment::FootnoteRef { label }),
                });
            },
            // 脚注定义（block）
            Event::Start(Tag::FootnoteDefinition(label)) => {
                let label = label.into_string();
                self.container_stack.push(Vec::new());
                self.footnote_label = Some(label);
            },
            Event::End(TagEnd::FootnoteDefinition) => {
                let label = self.footnote_label.take().unwrap_or_default();
                let mut children = self.container_stack.pop().unwrap_or_default();
                // 将首个 Paragraph 的 inline 提升为 title，并从 children 中移除，
                // 避免序列化时重复输出。
                let title = if children
                    .first()
                    .is_some_and(|b| matches!(b.kind, BlockKind::Paragraph))
                {
                    children.remove(0).title
                } else {
                    InlineTextTree::new()
                };
                self.push_block(Block {
                    kind: BlockKind::FootnoteDefinition { label },
                    title,
                    children,
                    code: None,
                    table: None,
                    block_id: None,
                });
            },
            // 定义列表
            Event::Start(Tag::DefinitionList | Tag::DefinitionListDefinition) => {
                self.container_stack.push(Vec::new());
            },
            Event::End(TagEnd::DefinitionList) => {
                let children = self.container_stack.pop().unwrap_or_default();
                self.push_block(Block {
                    kind: BlockKind::DefinitionList,
                    title: InlineTextTree::new(),
                    children,
                    code: None,
                    table: None,
                    block_id: None,
                });
            },
            Event::End(TagEnd::DefinitionListTitle) => {
                let frags = self.inline.take().unwrap_or_default();
                self.push_block(Block {
                    kind: BlockKind::DefinitionTerm,
                    title: InlineTextTree { fragments: frags },
                    children: Vec::new(),
                    code: None,
                    table: None,
                    block_id: None,
                });
            },
            Event::End(TagEnd::DefinitionListDefinition) => {
                let children = self.container_stack.pop().unwrap_or_default();
                self.push_block(Block {
                    kind: BlockKind::DefinitionDescription,
                    title: InlineTextTree::new(),
                    children,
                    code: None,
                    table: None,
                    block_id: None,
                });
            },
            // 数学公式：pulldown-cmark 0.13 的 DisplayMath / InlineMath
            // 事件直接携带公式内容（CowStr），无需等待后续 Text 事件。
            Event::DisplayMath(content) => {
                self.flush_text_buffer();
                self.push_block(Block {
                    kind: BlockKind::MathBlock,
                    title: InlineTextTree::new(),
                    children: Vec::new(),
                    code: Some(content.into_string()),
                    table: None,
                    block_id: None,
                });
            },
            Event::InlineMath(content) => {
                self.flush_text_buffer();
                let content = content.into_string();
                self.push_fragment(InlineFragment {
                    text: content.clone(),
                    style: self.current_style(),
                    attachment: Some(InlineAttachment::MathInline { content }),
                });
            },
            // HTML
            Event::Html(html) => {
                self.push_block(Block {
                    kind: BlockKind::HtmlBlock,
                    title: InlineTextTree::new(),
                    children: Vec::new(),
                    code: Some(html.into_string()),
                    table: None,
                    block_id: None,
                });
            },
            Event::InlineHtml(html) => {
                let content = html.into_string();
                self.push_fragment(InlineFragment {
                    text: content.clone(),
                    style: self.current_style(),
                    attachment: Some(InlineAttachment::InlineHtml { content }),
                });
            },
            _ => {},
        }
    }

    fn make_list_item_kind(&mut self) -> BlockKind {
        match (self.list_stack.last(), self.task_marker) {
            (
                Some(ListContext {
                    ordered: true,
                    ordinal,
                }),
                _,
            ) => {
                let o = *ordinal;
                if let Some(c) = self.list_stack.last_mut() {
                    c.ordinal = o.saturating_add(1);
                }
                BlockKind::NumberedListItem { ordinal: o }
            },
            (_, Some(checked)) => BlockKind::TaskListItem { checked },
            _ => BlockKind::BulletedListItem,
        }
    }

    fn current_style(&self) -> InlineStyle {
        self.style_stack
            .iter()
            .copied()
            .fold(InlineStyle::none(), InlineStyle::merge)
    }

    fn push_text(&mut self, text: &str) {
        if self.in_code {
            self.code_buffer.push_str(text);
            return;
        }
        // 累积连续 Text 事件，flush 时统一调用 split_wikilinks。
        // pulldown-cmark 0.13 会把 `[[note|alias]]` 拆成多个 Text 事件，
        // 必须在拼接后的完整文本上才能识别 WikiLink 模式。
        self.text_buffer.push_str(text);
    }

    /// flush 文本缓冲：对累积的文本依次调用 `split_wikilinks`、`split_comments`、
    /// `split_highlights`、`split_tags`，拆分后的片段以当前 inline 样式 / 附件
    /// push 到 inline 收集器。
    ///
    /// 性能优化：先做快速字节级预检，若文本中不含任何特殊字符
    /// (`[` / `%` / `=` / `#`)，则直接作为单个纯文本片段 push，
    /// 跳过全部 4 段扫描链。
    fn flush_text_buffer(&mut self) {
        if self.text_buffer.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.text_buffer);
        let style = self.current_style();

        // 快速预检：不含特殊字符时直接 push 纯文本，跳过 4 段扫描
        let needs_scan = text.bytes().any(|b| matches!(b, b'[' | b'%' | b'=' | b'#'));

        if !needs_scan {
            if !text.is_empty() {
                let attachment = self.attachment_stack.last().cloned();
                self.push_fragment(InlineFragment {
                    text,
                    style,
                    attachment,
                });
            }
            return;
        }

        // 一次性 clone attachment，后续各阶段共享引用
        let attachment = self.attachment_stack.last().cloned();
        for seg in split_wikilinks(&text) {
            match seg {
                WikiSegment::Text(t) => {
                    if !t.is_empty() {
                        self.process_text_with_comments(&t, style, attachment.as_ref());
                    }
                },
                WikiSegment::WikiLink {
                    target,
                    alias,
                    heading,
                    block_id,
                } => {
                    let display = alias.clone().unwrap_or_else(|| target.clone());
                    let att = InlineAttachment::WikiLink {
                        target,
                        alias,
                        heading,
                        block_id,
                    };
                    self.push_fragment(InlineFragment {
                        text: display,
                        style,
                        attachment: Some(att),
                    });
                },
                WikiSegment::Embed {
                    target,
                    kind,
                    alias,
                    width,
                } => {
                    let display = alias.clone().unwrap_or_else(|| target.clone());
                    let att = InlineAttachment::Embed {
                        target,
                        kind,
                        alias,
                        width,
                    };
                    self.push_fragment(InlineFragment {
                        text: display,
                        style,
                        attachment: Some(att),
                    });
                },
            }
        }
    }

    /// 对纯文本片段依次扫描注释、高亮、标签。
    ///
    /// 性能优化：attachment 按引用传递，只在最终 `push_fragment` 时 clone 一次。
    fn process_text_with_comments(
        &mut self,
        text: &str,
        style: InlineStyle,
        attachment: Option<&InlineAttachment>,
    ) {
        for comment_seg in split_comments(text) {
            match comment_seg {
                CommentSegment::Text(plain) => {
                    if !plain.is_empty() {
                        self.process_text_with_highlights(&plain, style, attachment);
                    }
                },
                CommentSegment::Comment { content } => {
                    self.push_fragment(InlineFragment {
                        text: format!("%%{content}%%"),
                        style,
                        attachment: Some(InlineAttachment::Comment { content }),
                    });
                },
            }
        }
    }

    /// 对纯文本片段依次扫描高亮、标签。
    ///
    /// 性能优化：attachment 按引用传递，只在最终 `push_fragment` 时 clone 一次。
    fn process_text_with_highlights(
        &mut self,
        text: &str,
        style: InlineStyle,
        attachment: Option<&InlineAttachment>,
    ) {
        for hl_seg in split_highlights(text) {
            match hl_seg {
                HighlightSegment::Text(plain) => {
                    if !plain.is_empty() {
                        self.process_text_with_tags(&plain, style, attachment);
                    }
                },
                HighlightSegment::Highlight(content) => {
                    let mut hl_style = style;
                    hl_style.highlight = true;
                    self.push_fragment(InlineFragment {
                        text: content,
                        style: hl_style,
                        attachment: attachment.cloned(),
                    });
                },
            }
        }
    }

    /// 对纯文本片段扫描标签。
    ///
    /// 性能优化：attachment 按引用传递，只在最终 `push_fragment` 时 clone 一次。
    fn process_text_with_tags(
        &mut self,
        text: &str,
        style: InlineStyle,
        attachment: Option<&InlineAttachment>,
    ) {
        for tag_seg in split_tags(text) {
            match tag_seg {
                TagSegment::Text(plain) => {
                    if !plain.is_empty() {
                        self.push_fragment(InlineFragment {
                            text: plain,
                            style,
                            attachment: attachment.cloned(),
                        });
                    }
                },
                TagSegment::Tag { name } => {
                    self.push_fragment(InlineFragment {
                        text: format!("#{name}"),
                        style,
                        attachment: Some(InlineAttachment::Tag { name }),
                    });
                },
            }
        }
    }

    /// 把当前 inline 收集器的内容包装为隐式 `Paragraph` 块，push 到当前容器层。
    ///
    /// 用于 `pulldown-cmark` 0.13 列表项：Item 内的 inline 文本不再被
    /// `Paragraph` 包裹，需在遇到子块或 `End(Item)` 时手动 flush 为 Paragraph，
    /// 以便 `take_first_paragraph_title` 能从中提取 title。
    fn flush_inline_as_paragraph(&mut self) {
        // 先把文本缓冲 flush 成 fragment
        self.flush_text_buffer();
        let frags = self.inline.take();
        if let Some(frags) = frags
            && !frags.is_empty()
        {
            self.push_block(Block {
                kind: BlockKind::Paragraph,
                title: InlineTextTree { fragments: frags },
                children: Vec::new(),
                code: None,
                table: None,
                block_id: None,
            });
        }
        // 重新初始化 inline 收集器，以便 Item 内后续子块之间继续收集
        if self.item_depth > 0 {
            self.inline = Some(Vec::new());
        }
    }

    fn push_fragment(&mut self, fragment: InlineFragment) {
        if let Some(frags) = self.inline.as_mut() {
            frags.push(fragment);
        }
    }

    fn finish_leaf(&mut self, kind: BlockKind) {
        let frags = self.inline.take().unwrap_or_default();
        self.push_block(Block {
            kind,
            title: InlineTextTree { fragments: frags },
            children: Vec::new(),
            code: None,
            table: None,
            block_id: None,
        });
    }

    fn push_block(&mut self, block: Block) {
        if let Some(top) = self.container_stack.last_mut() {
            top.push(block);
        }
    }

    fn finish(self) -> Document {
        let blocks = self.container_stack.into_iter().next().unwrap_or_default();
        Document { blocks }
    }
}

/// 检测引用块是否为 Obsidian 风格的 Callout（`> [!NOTE]`）。
///
/// 若引用块的首个子块是段落，且文本以 `[!VARIANT]` 开头，
/// 则提取变体类型与标题，返回 `BlockKind::Callout`；否则返回原引用块。
#[allow(clippy::too_many_lines)]
fn try_make_callout(block: Block) -> Block {
    if !matches!(block.kind, BlockKind::BlockQuote) {
        return block;
    }
    let Block {
        kind: _,
        title: _,
        children,
        code: _,
        table: _,
        block_id: _,
    } = block;

    if children.is_empty() {
        return Block {
            kind: BlockKind::BlockQuote,
            title: InlineTextTree::new(),
            children,
            code: None,
            table: None,
            block_id: None,
        };
    }

    // 首个子块必须是段落
    if !matches!(children[0].kind, BlockKind::Paragraph) {
        return Block {
            kind: BlockKind::BlockQuote,
            title: InlineTextTree::new(),
            children,
            code: None,
            table: None,
            block_id: None,
        };
    }

    let first_text = children[0].title.plain_text();
    let first_line = first_text.lines().next().unwrap_or_default();

    // 匹配 `[!VARIANT] Title`
    let Some(after_bracket) = first_line.strip_prefix("[!") else {
        return Block {
            kind: BlockKind::BlockQuote,
            title: InlineTextTree::new(),
            children,
            code: None,
            table: None,
            block_id: None,
        };
    };
    let Some(after_close) = after_bracket.find(']') else {
        return Block {
            kind: BlockKind::BlockQuote,
            title: InlineTextTree::new(),
            children,
            code: None,
            table: None,
            block_id: None,
        };
    };
    let variant_str = &after_bracket[..after_close];
    // 检测折叠状态：] 后面紧跟 - 或 +
    let after_close = &after_bracket[after_close + 1..];
    let folded = if after_close.starts_with('-') {
        Some(true)
    } else if after_close.starts_with('+') {
        Some(false)
    } else {
        None
    };
    let title = after_close
        .strip_prefix('-')
        .unwrap_or(after_close)
        .strip_prefix('+')
        .unwrap_or(after_close)
        .trim()
        .to_string();

    let variant = match variant_str.to_uppercase().as_str() {
        "NOTE" => CalloutVariant::Note,
        "INFO" | "TODO" => CalloutVariant::Info,
        "TIP" | "HINT" | "IMPORTANT" => CalloutVariant::Tip,
        "WARN" | "WARNING" | "CAUTION" | "ATTENTION" => CalloutVariant::Warning,
        "DANGER" | "ERROR" => CalloutVariant::Danger,
        "SUCCESS" | "CHECK" | "DONE" => CalloutVariant::Success,
        "QUESTION" | "HELP" | "FAQ" => CalloutVariant::Question,
        "ABSTRACT" | "SUMMARY" | "TLDR" => CalloutVariant::Abstract,
        "QUOTE" | "CITE" => CalloutVariant::Quote,
        "BUG" => CalloutVariant::Bug,
        "EXAMPLE" => CalloutVariant::Example,
        "FAILURE" | "FAIL" | "MISSING" => CalloutVariant::Failure,
        other => CalloutVariant::Other(other.to_string()),
    };

    // 构建 callout 子块：首段落的剩余内容 + 其余子块
    let mut callout_children = Vec::new();
    let remaining = first_text
        .find('\n')
        .map_or("", |idx| &first_text[idx + 1..]);
    if !remaining.is_empty() {
        callout_children.push(Block {
            kind: BlockKind::Paragraph,
            title: InlineTextTree::from_text(remaining.to_string()),
            children: Vec::new(),
            code: None,
            table: None,
            block_id: None,
        });
    }
    callout_children.extend(children.into_iter().skip(1));

    Block {
        kind: BlockKind::Callout {
            variant,
            title,
            folded,
        },
        title: InlineTextTree::new(),
        children: callout_children,
        code: None,
        table: None,
        block_id: None,
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn to_table_align(a: Alignment) -> Option<TableAlign> {
    match a {
        Alignment::None => None,
        Alignment::Left => Some(TableAlign::Left),
        Alignment::Center => Some(TableAlign::Center),
        Alignment::Right => Some(TableAlign::Right),
    }
}

/// 首版简化：`LinkType::Inline` 为行内链接，其余引用类链接
/// （Reference / Collapsed / Shortcut 等）统一作为 Reference 附件。
fn make_link_attachment(lt: LinkType, dest: &str, title: &str) -> InlineAttachment {
    match lt {
        LinkType::Inline => InlineAttachment::Link {
            destination: dest.to_string(),
            title: if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            },
        },
        _ => InlineAttachment::Reference {
            label: dest.to_string(),
        },
    }
}

/// 将 children 的第一个 `Paragraph` 的 inline 提升为列表项 title。
fn take_first_paragraph_title(children: &mut Vec<Block>) -> InlineTextTree {
    if children
        .first()
        .is_some_and(|first| matches!(first.kind, BlockKind::Paragraph))
    {
        return children.remove(0).title;
    }
    InlineTextTree::new()
}
