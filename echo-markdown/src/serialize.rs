//! 序列化：`Document` 块树 -> `Markdown` 文本。
//!
//! 递归遍历块树，按类型输出前缀与内容。`Inline` 片段按样式包裹。
//! 注意：扁平 fragment 模型下，嵌套格式的序列化可能不完全还原
//! 原始嵌套结构（如 `**bold _italic_**`），但 `parse -> serialize -> parse`
//! 的块树往返保持一致。

use std::fmt::Write;

use crate::block::{Block, BlockKind, CalloutVariant, TableAlign, TableData};
use crate::document::Document;
use crate::inline::{InlineAttachment, InlineFragment, InlineTextTree};

/// 将文档序列化为 `Markdown` 文本。
#[must_use]
pub fn to_markdown(doc: &Document) -> String {
    let mut out = String::new();
    for block in &doc.blocks {
        serialize_block(&mut out, block, 0);
        out.push('\n');
    }
    out
}

fn serialize_block(out: &mut String, block: &Block, depth: usize) {
    match &block.kind {
        BlockKind::Paragraph | BlockKind::DefinitionTerm => {
            serialize_inline(out, &block.title);
        },
        BlockKind::Heading { level } => {
            out.push_str(&"#".repeat(*level as usize));
            out.push(' ');
            serialize_inline(out, &block.title);
        },
        BlockKind::CodeBlock { language } => {
            out.push_str("```");
            if let Some(lang) = language {
                out.push_str(lang);
            }
            out.push('\n');
            if let Some(code) = &block.code {
                out.push_str(code);
            }
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```");
        },
        BlockKind::BulletedListItem => serialize_list_item(out, "- ", block, depth),
        BlockKind::TaskListItem { checked } => {
            serialize_list_item(
                out,
                if *checked { "- [x] " } else { "- [ ] " },
                block,
                depth,
            );
        },
        BlockKind::NumberedListItem { ordinal } => {
            let marker = format!("{ordinal}. ");
            serialize_list_item(out, &marker, block, depth);
        },
        BlockKind::BlockQuote => {
            serialize_blockquote(out, block, depth);
        },
        BlockKind::Table => {
            if let Some(table) = &block.table {
                serialize_table(out, table);
            }
        },
        BlockKind::ThematicBreak => out.push_str("---"),
        BlockKind::FootnoteDefinition { label } => {
            let _ = write!(out, "[^{label}]: ");
            serialize_inline(out, &block.title);
            for (i, child) in block.children.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                serialize_block(out, child, depth + 1);
            }
        },
        BlockKind::DefinitionList => {
            for (i, child) in block.children.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                serialize_block(out, child, depth);
            }
        },
        BlockKind::DefinitionDescription => {
            out.push_str(":   ");
            serialize_inline(out, &block.title);
            for child in &block.children {
                out.push('\n');
                serialize_block(out, child, depth + 1);
            }
        },
        BlockKind::MathBlock => {
            out.push_str("$$");
            if let Some(code) = &block.code {
                out.push_str(code);
            }
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("$$");
        },
        BlockKind::HtmlBlock => {
            if let Some(html) = &block.code {
                out.push_str(html);
            }
        },
        BlockKind::Callout { .. } => serialize_callout(out, block),
        BlockKind::Frontmatter => serialize_frontmatter(out, block),
        BlockKind::Mermaid => serialize_mermaid(out, block),
    }
}

fn serialize_blockquote(out: &mut String, block: &Block, depth: usize) {
    let mut inner = String::new();
    for (i, child) in block.children.iter().enumerate() {
        if i > 0 {
            inner.push('\n');
        }
        serialize_block(&mut inner, child, depth);
    }
    for line in inner.lines() {
        out.push_str("> ");
        out.push_str(line);
        out.push('\n');
    }
    if out.ends_with('\n') {
        out.pop();
    }
}

fn serialize_callout(out: &mut String, block: &Block) {
    let (variant, title, folded) = match &block.kind {
        BlockKind::Callout {
            variant,
            title,
            folded,
        } => (variant, title.clone(), *folded),
        _ => return,
    };
    let variant_str = match variant {
        CalloutVariant::Note => "NOTE",
        CalloutVariant::Info => "INFO",
        CalloutVariant::Tip => "TIP",
        CalloutVariant::Warning => "WARNING",
        CalloutVariant::Danger => "DANGER",
        CalloutVariant::Success => "SUCCESS",
        CalloutVariant::Question => "QUESTION",
        CalloutVariant::Abstract => "ABSTRACT",
        CalloutVariant::Quote => "QUOTE",
        CalloutVariant::Bug => "BUG",
        CalloutVariant::Example => "EXAMPLE",
        CalloutVariant::Failure => "FAILURE",
        CalloutVariant::Other(s) => s.as_str(),
    };
    let fold_marker = match folded {
        Some(true) => "-",
        Some(false) => "+",
        None => "",
    };
    let heading = if title.is_empty() {
        format!("[!{variant_str}{fold_marker}]")
    } else {
        format!("[!{variant_str}{fold_marker}] {title}")
    };
    out.push_str("> ");
    out.push_str(&heading);
    for child in &block.children {
        out.push('\n');
        out.push_str("> ");
        let mut inner = String::new();
        serialize_block(&mut inner, child, 0);
        for line in inner.lines() {
            out.push_str(line);
            out.push('\n');
            out.push_str("> ");
        }
    }
}

fn serialize_frontmatter(out: &mut String, block: &Block) {
    out.push_str("---\n");
    if let Some(yaml) = &block.code {
        out.push_str(yaml);
        if !yaml.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("---");
}

fn serialize_mermaid(out: &mut String, block: &Block) {
    out.push_str("```mermaid\n");
    if let Some(code) = &block.code {
        out.push_str(code);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```");
}

fn serialize_list_item(out: &mut String, marker: &str, block: &Block, depth: usize) {
    out.push_str(&"  ".repeat(depth));
    out.push_str(marker);
    serialize_inline(out, &block.title);
    if !block.children.is_empty() {
        out.push('\n');
        for (i, child) in block.children.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            serialize_block(out, child, depth + 1);
        }
    }
}

fn serialize_table(out: &mut String, table: &TableData) {
    out.push('|');
    for cell in &table.headers {
        out.push(' ');
        serialize_inline(out, &cell.content);
        out.push_str(" |");
    }
    out.push('\n');
    out.push('|');
    for cell in &table.headers {
        match cell.align.unwrap_or(TableAlign::Left) {
            TableAlign::Left => out.push_str(" --- |"),
            TableAlign::Center => out.push_str(" :---: |"),
            TableAlign::Right => out.push_str(" ---: |"),
        }
    }
    out.push('\n');
    for row in &table.rows {
        out.push('|');
        for cell in row {
            out.push(' ');
            serialize_inline(out, &cell.content);
            out.push_str(" |");
        }
        out.push('\n');
    }
    if out.ends_with('\n') {
        out.pop();
    }
}

fn serialize_inline(out: &mut String, tree: &InlineTextTree) {
    for frag in &tree.fragments {
        serialize_fragment(out, frag);
    }
}

#[allow(clippy::too_many_lines)]
fn serialize_fragment(out: &mut String, frag: &InlineFragment) {
    match &frag.attachment {
        Some(InlineAttachment::Image {
            destination,
            title,
            width,
        }) => {
            out.push_str("![");
            out.push_str(&frag.text);
            if let Some(w) = width {
                out.push('|');
                out.push_str(&w.to_string());
            }
            out.push_str("](");
            out.push_str(destination);
            if let Some(t) = title {
                out.push_str(" \"");
                out.push_str(t);
                out.push('"');
            }
            out.push(')');
        },
        Some(InlineAttachment::WikiLink {
            target,
            alias,
            heading,
            block_id,
        }) => {
            out.push_str("[[");
            out.push_str(target);
            if let Some(h) = heading {
                out.push('#');
                out.push_str(h);
            }
            if let Some(b) = block_id {
                out.push('#');
                out.push('^');
                out.push_str(b);
            }
            if let Some(a) = alias {
                out.push('|');
                out.push_str(a);
            }
            out.push_str("]]");
        },
        Some(InlineAttachment::Embed {
            target,
            kind: _,
            alias,
            width,
        }) => {
            out.push_str("![[");
            out.push_str(target);
            if let Some(w) = width {
                out.push('|');
                out.push_str(&w.to_string());
            }
            if let Some(a) = alias {
                out.push('|');
                out.push_str(a);
            }
            out.push_str("]]");
        },
        Some(InlineAttachment::Comment { content }) => {
            out.push_str("%%");
            out.push_str(content);
            out.push_str("%%");
        },
        Some(InlineAttachment::Autolink { target }) => {
            out.push('<');
            out.push_str(target);
            out.push('>');
        },
        Some(InlineAttachment::Link { destination, title }) => {
            out.push('[');
            out.push_str(&frag.text);
            out.push_str("](");
            out.push_str(destination);
            if let Some(t) = title {
                out.push_str(" \"");
                out.push_str(t);
                out.push('"');
            }
            out.push(')');
        },
        Some(InlineAttachment::Reference { label }) => {
            out.push('[');
            out.push_str(&frag.text);
            out.push_str("][");
            out.push_str(label);
            out.push(']');
        },
        Some(InlineAttachment::FootnoteRef { label }) => {
            out.push_str("[^");
            out.push_str(label);
            out.push(']');
        },
        Some(InlineAttachment::MathInline { content }) => {
            out.push('$');
            out.push_str(content);
            out.push('$');
        },
        Some(InlineAttachment::InlineHtml { content }) => {
            out.push_str(content);
        },
        Some(InlineAttachment::Tag { name }) => {
            out.push('#');
            out.push_str(name);
        },
        None => {
            if frag.style.code {
                out.push('`');
                out.push_str(&frag.text);
                out.push('`');
            } else {
                if frag.style.bold {
                    out.push_str("**");
                }
                if frag.style.italic {
                    out.push('*');
                }
                if frag.style.strikethrough {
                    out.push_str("~~");
                }
                if frag.style.highlight {
                    out.push_str("==");
                }
                out.push_str(&frag.text);
                if frag.style.highlight {
                    out.push_str("==");
                }
                if frag.style.strikethrough {
                    out.push_str("~~");
                }
                if frag.style.italic {
                    out.push('*');
                }
                if frag.style.bold {
                    out.push_str("**");
                }
            }
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;
    use crate::inline::InlineStyle;

    #[test]
    fn serialize_empty_document() {
        assert_eq!(to_markdown(&Document::new()), "");
    }

    #[test]
    fn serialize_heading_and_paragraph() {
        let mut doc = Document::new();
        doc.push(Block {
            kind: BlockKind::Heading { level: 2 },
            title: InlineTextTree::from_text("Title"),
            children: Vec::new(),
            code: None,
            table: None,
            block_id: None,
        });
        doc.push(Block {
            kind: BlockKind::Paragraph,
            title: InlineTextTree::from_text("Body"),
            children: Vec::new(),
            code: None,
            table: None,
            block_id: None,
        });
        let md = to_markdown(&doc);
        assert!(md.contains("## Title"));
        assert!(md.contains("Body"));
    }

    #[test]
    fn serialize_bold_fragment() {
        let mut tree = InlineTextTree::new();
        tree.push(InlineFragment {
            text: "bold".to_string(),
            style: InlineStyle {
                bold: true,
                ..InlineStyle::none()
            },
            attachment: None,
        });
        let mut out = String::new();
        serialize_inline(&mut out, &tree);
        assert_eq!(out, "**bold**");
    }
}
