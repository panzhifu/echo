//! 往返测试：`parse -> to_markdown -> parse` 块树保持一致。

use echo_markdown::{BlockKind, Document, parse, to_markdown};

/// `parse(md)` 与 `parse(to_markdown(parse(md)))` 应得到同一棵块树。
fn assert_roundtrips(md: &str) {
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    assert_eq!(
        first, second,
        "roundtrip diverged\nserialized:\n{serialized}"
    );
}

#[test]
fn heading_roundtrips() {
    assert_roundtrips("# Title");
}

#[test]
fn paragraph_roundtrips() {
    assert_roundtrips("hello world");
}

#[test]
fn bulleted_list_roundtrips() {
    assert_roundtrips("- a\n- b\n- c\n");
}

#[test]
fn numbered_list_roundtrips() {
    assert_roundtrips("1. one\n2. two\n");
}

#[test]
fn task_list_roundtrips() {
    assert_roundtrips("- [ ] todo\n- [x] done\n");
}

#[test]
fn code_block_roundtrips() {
    assert_roundtrips("```rust\nfn main() {}\n```\n");
}

#[test]
fn blockquote_roundtrips() {
    assert_roundtrips("> quoted\n");
}

#[test]
fn thematic_break_roundtrips() {
    assert_roundtrips("---\n");
}

#[test]
fn table_roundtrips() {
    assert_roundtrips("| a | b |\n|---|---|\n| 1 | 2 |\n");
}

#[test]
fn wikilink_roundtrips() {
    assert_roundtrips("[[note|alias]]");
}

#[test]
fn mixed_document_roundtrips() {
    let md = "\
# 标题

段落文本，含 [[wiki|链接]]。

- 列表项一
- 列表项二

```rust
fn main() {}
```
";
    assert_roundtrips(md);
}

#[test]
fn empty_document_serializes_stably() {
    let doc = Document::new();
    let s = to_markdown(&doc);
    assert!(s.is_empty());
    let reparsed = parse(&s).expect("reparse empty");
    assert!(reparsed.is_empty());
}

// ========== 新格式往返测试 ==========

#[test]
fn math_block_roundtrips() {
    assert_roundtrips(
        "$$
x^2 + y^2 = z^2
$$",
    );
}

#[test]
fn inline_math_roundtrips() {
    assert_roundtrips("公式 $E=mc^2$ 著名。");
}

#[test]
fn definition_list_roundtrips() {
    let md = "术语\n:   定义内容\n";
    assert_roundtrips(md);
}

#[test]
fn html_block_roundtrips() {
    // HTML 块往返：pulldown-cmark 会捕获尾部换行作为 HTML 内容，
    // 因此仅验证序列化后的 HTML 内容（去除尾部空白）一致。
    let md = "<div>hello</div>";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    let first_code = first.blocks[0]
        .code
        .as_deref()
        .unwrap_or_default()
        .trim_end();
    let second_code = second.blocks[0]
        .code
        .as_deref()
        .unwrap_or_default()
        .trim_end();
    assert_eq!(first_code, second_code);
}

#[test]
fn setext_h1_roundtrips() {
    let md = "标题\n===\n";
    assert_roundtrips(md);
}

#[test]
fn setext_h2_roundtrips() {
    let md = "副标题\n---\n";
    assert_roundtrips(md);
}

#[test]
fn footnote_definition_roundtrips() {
    let md = "[^1]: 脚注内容。\n";
    assert_roundtrips(md);
}

#[test]
fn footnote_reference_roundtrips() {
    // 脚注引用需有定义才能识别。往返后脚注定义与引用结构保持一致。
    let md = "正文[^1]。\n\n[^1]: 脚注。\n";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    // 块数量一致（段落 + 脚注定义）
    assert_eq!(
        first.blocks.len(),
        second.blocks.len(),
        "block count should match\nserialized:\n{serialized}"
    );
    // 脚注定义标签一致
    let first_label = match &first.blocks[1].kind {
        BlockKind::FootnoteDefinition { label } => label.clone(),
        _ => String::new(),
    };
    let second_label = match &second.blocks[1].kind {
        BlockKind::FootnoteDefinition { label } => label.clone(),
        _ => String::new(),
    };
    assert_eq!(first_label, second_label);
}
