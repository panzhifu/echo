//! 新格式集成测试：脚注、数学公式、定义列表、HTML、Setext 标题。

use echo_markdown::{BlockKind, InlineAttachment, parse};

// ========== 脚注 ==========

#[test]
fn footnote_reference_in_paragraph() {
    // pulldown-cmark 仅在存在脚注定义时将 [^1] 识别为脚注引用
    let md = "文本[^1]结尾。\n\n[^1]: 脚注。\n";
    let doc = parse(md).unwrap();
    let paragraph = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Paragraph))
        .unwrap();
    let frags = &paragraph.title.fragments;
    assert_eq!(frags.len(), 3);
    assert_eq!(frags[0].text, "文本");
    assert_eq!(frags[2].text, "结尾。");
    match &frags[1].attachment {
        Some(InlineAttachment::FootnoteRef { label }) => {
            assert_eq!(label, "1");
        },
        other => panic!("expected FootnoteRef, got {other:?}"),
    }
}

#[test]
fn footnote_definition_block() {
    let doc = parse("[^1]: 脚注内容。\n").unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::FootnoteDefinition { .. }))
    );
    let foot = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::FootnoteDefinition { .. }))
        .expect("footnote definition should exist");
    match &foot.kind {
        BlockKind::FootnoteDefinition { label } => assert_eq!(label, "1"),
        _ => unreachable!(),
    }
}

#[test]
fn footnote_reference_with_label() {
    let md = "见[^note]。\n\n[^note]: 命名脚注。\n";
    let doc = parse(md).unwrap();
    let paragraph = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Paragraph))
        .unwrap();
    let frags = &paragraph.title.fragments;
    assert_eq!(frags.len(), 3);
    assert_eq!(frags[0].text, "见");
    assert_eq!(frags[2].text, "。");
    match &frags[1].attachment {
        Some(InlineAttachment::FootnoteRef { label }) => {
            assert_eq!(label, "note");
            assert_eq!(frags[1].text, "[^note]");
        },
        other => panic!("expected FootnoteRef, got {other:?}"),
    }
}

// ========== 数学公式 ==========

#[test]
fn display_math_block() {
    let doc = parse("$$\nx^2 + y^2 = z^2\n$$\n").unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::MathBlock))
    );
    let math = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::MathBlock))
        .expect("math block should exist");
    let code = math.code.as_deref().unwrap_or_default();
    assert!(
        code.contains("x^2"),
        "math content should contain x^2: {code}"
    );
}

#[test]
fn inline_math_in_paragraph() {
    let doc = parse("公式 $E=mc^2$ 著名。").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    let frags = &doc.blocks[0].title.fragments;
    // 公式 + 数学 + 公式
    let math_frag = frags
        .iter()
        .find(|f| matches!(&f.attachment, Some(InlineAttachment::MathInline { .. })))
        .expect("should have inline math fragment");
    match &math_frag.attachment {
        Some(InlineAttachment::MathInline { content }) => {
            assert_eq!(content, "E=mc^2");
        },
        other => panic!("expected MathInline, got {other:?}"),
    }
}

#[test]
fn multiple_inline_math_in_paragraph() {
    let doc = parse("$a$ 和 $b$ 相加。").unwrap();
    let math_count = doc.blocks[0]
        .title
        .fragments
        .iter()
        .filter(|f| matches!(&f.attachment, Some(InlineAttachment::MathInline { .. })))
        .count();
    assert_eq!(math_count, 2);
}

// ========== 定义列表 ==========

#[test]
fn definition_list_basic() {
    let md = "\
术语
:   定义内容
";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::DefinitionList))
    );
    let def_list = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::DefinitionList))
        .expect("definition list should exist");
    // 应包含 DefinitionTerm 和 DefinitionDescription
    assert!(
        def_list
            .children
            .iter()
            .any(|b| matches!(b.kind, BlockKind::DefinitionTerm))
    );
    assert!(
        def_list
            .children
            .iter()
            .any(|b| matches!(b.kind, BlockKind::DefinitionDescription))
    );
}

#[test]
fn definition_term_text() {
    let md = "\
Apple
:   A fruit
";
    let doc = parse(md).unwrap();
    let def_list = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::DefinitionList))
        .unwrap();
    let term = def_list
        .children
        .iter()
        .find(|b| matches!(b.kind, BlockKind::DefinitionTerm))
        .unwrap();
    assert_eq!(term.title.plain_text(), "Apple");
}

// ========== HTML ==========

#[test]
fn html_block_preserved() {
    let md = "<div>\nhello\n</div>\n";
    let doc = parse(md).unwrap();
    let html_block = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::HtmlBlock))
        .expect("html block should exist");
    let code = html_block.code.as_deref().unwrap_or_default();
    assert!(code.contains("<div>"));
}

#[test]
fn inline_html_in_paragraph() {
    let doc = parse("文本 <em>强调</em> 结尾。").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    let html_frag = frags
        .iter()
        .find(|f| matches!(&f.attachment, Some(InlineAttachment::InlineHtml { .. })))
        .expect("should have inline html fragment");
    match &html_frag.attachment {
        Some(InlineAttachment::InlineHtml { content }) => {
            assert!(content.contains("<em>"));
        },
        other => panic!("expected InlineHtml, got {other:?}"),
    }
}

// ========== Setext 标题 ==========

#[test]
fn setext_h1() {
    let md = "\
标题
===
";
    let doc = parse(md).unwrap();
    let heading = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Heading { level: 1 }));
    assert!(heading.is_some(), "Setext H1 should be parsed");
    assert_eq!(heading.unwrap().title.plain_text(), "标题");
}

#[test]
fn setext_h2() {
    let md = "\
副标题
---
";
    let doc = parse(md).unwrap();
    let heading = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Heading { level: 2 }));
    assert!(heading.is_some(), "Setext H2 should be parsed");
    assert_eq!(heading.unwrap().title.plain_text(), "副标题");
}

// ========== 混合文档 ==========

#[test]
fn mixed_new_formats() {
    let md = "\
# 标题

正文含 $E=mc^2$ 与[^1]。

术语
:   定义

$$
a^2 + b^2 = c^2
$$

[^1]: 脚注。
";
    let doc = parse(md).unwrap();
    // 标题
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Heading { level: 1 }))
    );
    // 数学块
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::MathBlock))
    );
    // 定义列表
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::DefinitionList))
    );
    // 脚注定义
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::FootnoteDefinition { .. }))
    );
}
