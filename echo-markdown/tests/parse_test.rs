//! 解析集成测试：`Markdown` 文本 -> `Document` 块树。

use echo_markdown::{BlockKind, InlineAttachment, parse};

#[test]
fn heading_with_level() {
    let doc = parse("# Title").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    let BlockKind::Heading { level } = &doc.blocks[0].kind else {
        panic!("expected heading, got {:?}", doc.blocks[0].kind);
    };
    assert_eq!(*level, 1);
    assert_eq!(doc.blocks[0].title.plain_text(), "Title");
}

#[test]
fn paragraph_plain_text() {
    let doc = parse("hello world").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    assert!(matches!(&doc.blocks[0].kind, BlockKind::Paragraph));
    assert_eq!(doc.blocks[0].title.plain_text(), "hello world");
}

#[test]
fn fenced_code_block_carries_language_and_code() {
    let md = "```rust\nfn main() {}\n```\n";
    let doc = parse(md).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    let BlockKind::CodeBlock { language } = &doc.blocks[0].kind else {
        panic!("expected code block, got {:?}", doc.blocks[0].kind);
    };
    assert_eq!(language.as_deref(), Some("rust"));
    let code = doc.blocks[0].code.as_deref().unwrap_or_default();
    assert!(code.contains("fn main"));
}

#[test]
fn bulleted_list_two_items() {
    let doc = parse("- a\n- b\n").unwrap();
    assert_eq!(doc.blocks.len(), 2);
    assert!(matches!(&doc.blocks[0].kind, BlockKind::BulletedListItem));
    assert!(matches!(&doc.blocks[1].kind, BlockKind::BulletedListItem));
    assert_eq!(doc.blocks[0].title.plain_text(), "a");
    assert_eq!(doc.blocks[1].title.plain_text(), "b");
}

#[test]
fn task_list_checked_and_unchecked() {
    let doc = parse("- [ ] todo\n- [x] done\n").unwrap();
    assert_eq!(doc.blocks.len(), 2);
    assert!(matches!(
        &doc.blocks[0].kind,
        BlockKind::TaskListItem { checked: false }
    ));
    assert!(matches!(
        &doc.blocks[1].kind,
        BlockKind::TaskListItem { checked: true }
    ));
    assert_eq!(doc.blocks[0].title.plain_text(), "todo");
    assert_eq!(doc.blocks[1].title.plain_text(), "done");
}

#[test]
fn numbered_list_increments_ordinal() {
    let doc = parse("1. first\n2. second\n").unwrap();
    assert_eq!(doc.blocks.len(), 2);
    assert!(matches!(
        &doc.blocks[0].kind,
        BlockKind::NumberedListItem { ordinal: 1 }
    ));
    assert!(matches!(
        &doc.blocks[1].kind,
        BlockKind::NumberedListItem { ordinal: 2 }
    ));
}

#[test]
fn blockquote_wraps_paragraph() {
    let doc = parse("> quoted\n").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    assert!(matches!(&doc.blocks[0].kind, BlockKind::BlockQuote));
    assert_eq!(doc.blocks[0].children.len(), 1);
    assert!(matches!(
        &doc.blocks[0].children[0].kind,
        BlockKind::Paragraph
    ));
    assert_eq!(doc.blocks[0].children[0].title.plain_text(), "quoted");
}

#[test]
fn thematic_break() {
    let doc = parse("---\n").unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::ThematicBreak))
    );
}

#[test]
fn table_headers_and_rows() {
    let md = "| a | b |\n|---|---|\n| 1 | 2 |\n";
    let doc = parse(md).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    let BlockKind::Table = &doc.blocks[0].kind else {
        panic!("expected table, got {:?}", doc.blocks[0].kind);
    };
    let table = doc.blocks[0]
        .table
        .as_ref()
        .expect("table data should be present");
    assert_eq!(table.headers.len(), 2);
    assert_eq!(table.headers[0].content.plain_text(), "a");
    assert_eq!(table.headers[1].content.plain_text(), "b");
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0][0].content.plain_text(), "1");
    assert_eq!(table.rows[0][1].content.plain_text(), "2");
}

#[test]
fn inline_bold_fragment() {
    let doc = parse("**bold**").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    let frag = &doc.blocks[0].title.fragments[0];
    assert!(frag.style.bold);
    assert!(!frag.style.italic);
    assert_eq!(frag.text, "bold");
}

#[test]
fn inline_wikilink_attachment() {
    let doc = parse("[[note|alias]]").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    let frag = &doc.blocks[0].title.fragments[0];
    match &frag.attachment {
        Some(InlineAttachment::WikiLink { target, alias, .. }) => {
            assert_eq!(target, "note");
            assert_eq!(alias.as_deref(), Some("alias"));
            assert_eq!(frag.text, "alias");
        },
        other => panic!("expected WikiLink attachment, got {other:?}"),
    }
}
