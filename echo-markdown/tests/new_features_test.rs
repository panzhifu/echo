//! 新特性集成测试：Callout、YAML Frontmatter、Tag、Mermaid。

use echo_markdown::{BlockKind, CalloutVariant, InlineAttachment, parse, to_markdown};

// ========== Callout ==========

#[test]
fn callout_note() {
    let md = "> [!NOTE] 这是一个提示\n> 内容在这里。\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Callout { .. }))
    );
    let callout = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Callout { .. }))
        .expect("callout should exist");
    match &callout.kind {
        BlockKind::Callout {
            variant,
            title,
            folded,
        } => {
            assert_eq!(*variant, CalloutVariant::Note);
            assert_eq!(title, "这是一个提示");
            assert!(folded.is_none());
        },
        _ => panic!("expected Callout"),
    }
}

#[test]
fn callout_warning() {
    let md = "> [!WARNING]\n> 警告内容。\n";
    let doc = parse(md).unwrap();
    let callout = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Callout { .. }))
        .expect("callout should exist");
    match &callout.kind {
        BlockKind::Callout {
            variant,
            title,
            folded,
        } => {
            assert_eq!(*variant, CalloutVariant::Warning);
            assert!(title.is_empty());
            assert!(folded.is_none());
        },
        _ => panic!("expected Callout"),
    }
}

#[test]
fn callout_tip() {
    let md = "> [!TIP] 小技巧\n> 提示内容。\n";
    let doc = parse(md).unwrap();
    let callout = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Callout { .. }))
        .expect("callout should exist");
    match &callout.kind {
        BlockKind::Callout {
            variant,
            title,
            folded,
        } => {
            assert_eq!(*variant, CalloutVariant::Tip);
            assert_eq!(title, "小技巧");
            assert!(folded.is_none());
        },
        _ => panic!("expected Callout"),
    }
}

#[test]
fn callout_danger() {
    let md = "> [!DANGER] 危险！\n> 不要这样做。\n";
    let doc = parse(md).unwrap();
    let callout = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Callout { .. }))
        .expect("callout should exist");
    match &callout.kind {
        BlockKind::Callout {
            variant,
            title,
            folded,
        } => {
            assert_eq!(*variant, CalloutVariant::Danger);
            assert_eq!(title, "危险！");
            assert!(folded.is_none());
        },
        _ => panic!("expected Callout"),
    }
}

#[test]
fn callout_other_variant() {
    let md = "> [!CUSTOM] 自定义\n> 内容。\n";
    let doc = parse(md).unwrap();
    let callout = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Callout { .. }))
        .expect("callout should exist");
    match &callout.kind {
        BlockKind::Callout {
            variant,
            title,
            folded,
        } => {
            assert_eq!(*variant, CalloutVariant::Other("CUSTOM".to_string()));
            assert_eq!(title, "自定义");
            assert!(folded.is_none());
        },
        _ => panic!("expected Callout"),
    }
}

#[test]
fn callout_preserves_children() {
    let md = "> [!NOTE] 标题\n> 第一行内容\n>\n> 第二段内容\n";
    let doc = parse(md).unwrap();
    let callout = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Callout { .. }))
        .expect("callout should exist");
    // callout 应包含子块（段落）
    assert!(!callout.children.is_empty());
}

#[test]
fn regular_blockquote_not_callout() {
    // 普通引用块不应被识别为 callout
    let md = "> 这是一个普通引用\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .all(|b| !matches!(b.kind, BlockKind::Callout { .. }))
    );
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::BlockQuote))
    );
}

#[test]
fn callout_roundtrips() {
    let md = "> [!NOTE] 标题\n> 内容。\n";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    assert_eq!(
        first, second,
        "roundtrip diverged\nserialized:\n{serialized}"
    );
}

#[test]
fn callout_warning_roundtrips() {
    let md = "> [!WARNING]\n> 警告内容。\n";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    assert_eq!(
        first, second,
        "roundtrip diverged\nserialized:\n{serialized}"
    );
}

// ========== YAML Frontmatter ==========

#[test]
fn frontmatter_basic() {
    let md = "---\ntitle: 我的文档\n---\n\n正文内容。\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Frontmatter))
    );
    let fm = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Frontmatter))
        .expect("frontmatter should exist");
    let code = fm.code.as_deref().unwrap_or_default();
    assert!(code.contains("title: 我的文档"));
}

#[test]
fn frontmatter_first_block() {
    let md = "---\ntags: [rust, markdown]\n---\n\n# 标题\n";
    let doc = parse(md).unwrap();
    // frontmatter 应该是第一个块
    assert!(matches!(doc.blocks[0].kind, BlockKind::Frontmatter));
}

#[test]
fn frontmatter_with_content() {
    let md = "---\nauthor: Alice\ndate: 2024-01-01\n---\n\n正文在这里。\n";
    let doc = parse(md).unwrap();
    let fm = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Frontmatter))
        .expect("frontmatter should exist");
    let code = fm.code.as_deref().unwrap_or_default();
    assert!(code.contains("author: Alice"));
    assert!(code.contains("date: 2024-01-01"));
}

#[test]
fn frontmatter_not_thematic_break() {
    // 单独的分隔线不应被识别为 frontmatter
    let md = "---\n\n正文。\n";
    let doc = parse(md).unwrap();
    // 没有配对的 ---，所以不应有 frontmatter
    assert!(
        doc.blocks
            .iter()
            .all(|b| !matches!(b.kind, BlockKind::Frontmatter))
    );
}

#[test]
fn frontmatter_no_closing() {
    // 只有开头没有闭合的 --- 不是 frontmatter
    let md = "---\n没有闭合\n\n正文。\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .all(|b| !matches!(b.kind, BlockKind::Frontmatter))
    );
}

#[test]
fn frontmatter_roundtrips() {
    let md = "---\ntitle: Test\n---\n\n正文。\n";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    assert_eq!(
        first, second,
        "roundtrip diverged\nserialized:\n{serialized}"
    );
}

// ========== Tag ==========

#[test]
fn tag_in_paragraph() {
    let doc = parse("学习 #rust 编程语言。").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    let tag_frag = frags
        .iter()
        .find(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
        .expect("should have tag fragment");
    match &tag_frag.attachment {
        Some(InlineAttachment::Tag { name }) => {
            assert_eq!(name, "rust");
        },
        other => panic!("expected Tag, got {other:?}"),
    }
}

#[test]
fn multiple_tags_in_paragraph() {
    let doc = parse("#rust 和 #python 都很棒。").unwrap();
    let tag_count = doc.blocks[0]
        .title
        .fragments
        .iter()
        .filter(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
        .count();
    assert_eq!(tag_count, 2);
}

#[test]
fn tag_with_hyphen() {
    let doc = parse("#rust-lang 标签。").unwrap();
    let tag_frag = doc.blocks[0]
        .title
        .fragments
        .iter()
        .find(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
        .expect("should have tag");
    match &tag_frag.attachment {
        Some(InlineAttachment::Tag { name }) => {
            assert_eq!(name, "rust-lang");
        },
        _ => panic!("expected Tag"),
    }
}

#[test]
fn tag_with_slash() {
    let doc = parse("#programming/rust 嵌套标签。").unwrap();
    let tag_frag = doc.blocks[0]
        .title
        .fragments
        .iter()
        .find(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
        .expect("should have tag");
    match &tag_frag.attachment {
        Some(InlineAttachment::Tag { name }) => {
            assert_eq!(name, "programming/rust");
        },
        _ => panic!("expected Tag"),
    }
}

#[test]
fn tag_no_space_before_not_tag() {
    // C# 中的 # 不应被识别为标签
    let doc = parse("C# 编程语言。").unwrap();
    let tag_count = doc.blocks[0]
        .title
        .fragments
        .iter()
        .filter(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
        .count();
    assert_eq!(tag_count, 0);
}

#[test]
fn tag_display_text() {
    let doc = parse("#rust").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    let tag_frag = frags
        .iter()
        .find(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
        .unwrap();
    assert_eq!(tag_frag.text, "#rust");
}

#[test]
fn tag_roundtrips() {
    let md = "学习 #rust 编程。";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    assert_eq!(
        first, second,
        "roundtrip diverged\nserialized:\n{serialized}"
    );
}

// ========== Mermaid ==========

#[test]
fn mermaid_block() {
    let md = "```mermaid\ngraph TD\n    A-->B\n```\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Mermaid))
    );
    let mermaid = doc
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::Mermaid))
        .expect("mermaid should exist");
    let code = mermaid.code.as_deref().unwrap_or_default();
    assert!(code.contains("graph TD"));
}

#[test]
fn mermaid_not_code_block() {
    // mermaid 代码块不应被识别为普通 CodeBlock
    let md = "```mermaid\ngraph TD\n    A-->B\n```\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .all(|b| !matches!(b.kind, BlockKind::CodeBlock { .. }))
    );
}

#[test]
fn regular_code_block_not_mermaid() {
    let md = "```rust\nfn main() {}\n```\n";
    let doc = parse(md).unwrap();
    assert!(
        doc.blocks
            .iter()
            .all(|b| !matches!(b.kind, BlockKind::Mermaid))
    );
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::CodeBlock { .. }))
    );
}

#[test]
fn mermaid_roundtrips() {
    let md = "```mermaid\ngraph TD\n    A-->B\n```\n";
    let first = parse(md).expect("first parse");
    let serialized = to_markdown(&first);
    let second = parse(&serialized).expect("second parse");
    assert_eq!(
        first, second,
        "roundtrip diverged\nserialized:\n{serialized}"
    );
}

// ========== 混合文档 ==========

#[test]
fn mixed_new_features() {
    let md = "---\ntitle: 测试文档\ntags: [rust]\n---\n\n# 标题\n\n正文含 #rust 和 $E=mc^2$。\n\n> [!NOTE] 提示\n> 这是一个 callout。\n\n```mermaid\ngraph TD\n    A-->B\n```\n";
    let doc = parse(md).unwrap();
    // frontmatter
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Frontmatter))
    );
    // 标题
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Heading { level: 1 }))
    );
    // 标签
    let has_tag = doc.blocks.iter().any(|b| {
        b.title
            .fragments
            .iter()
            .any(|f| matches!(&f.attachment, Some(InlineAttachment::Tag { .. })))
    });
    assert!(has_tag);
    // callout
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Callout { .. }))
    );
    // mermaid
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Mermaid))
    );
}
