//! `WikiLink` 集成测试：经 `parse` 全链路拆分。

use echo_markdown::{InlineAttachment, parse};

#[test]
fn plain_wikilink_in_paragraph() {
    let doc = parse("[[note]]").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    assert_eq!(frags.len(), 1);
    match &frags[0].attachment {
        Some(InlineAttachment::WikiLink { target, alias, .. }) => {
            assert_eq!(target, "note");
            assert!(alias.is_none());
            assert_eq!(frags[0].text, "note");
        },
        other => panic!("expected WikiLink, got {other:?}"),
    }
}

#[test]
fn alias_becomes_display_text() {
    let doc = parse("[[target|shown]]").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    assert_eq!(frags.len(), 1);
    match &frags[0].attachment {
        Some(InlineAttachment::WikiLink { target, alias, .. }) => {
            assert_eq!(target, "target");
            assert_eq!(alias.as_deref(), Some("shown"));
            assert_eq!(frags[0].text, "shown");
        },
        other => panic!("expected WikiLink, got {other:?}"),
    }
}

#[test]
fn wikilink_surrounded_by_text_splits_into_three() {
    let doc = parse("see [[note]] here").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    assert_eq!(frags.len(), 3);
    assert_eq!(frags[0].text, "see ");
    assert!(frags[1].attachment.is_some());
    assert_eq!(frags[2].text, " here");
}

#[test]
fn multiple_wikilinks_in_one_paragraph() {
    let doc = parse("[[a]] and [[b|B]]").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    // [[a]] + " and " + [[b|B]] => 3 fragments
    assert_eq!(frags.len(), 3);
    assert!(matches!(
        &frags[0].attachment,
        Some(InlineAttachment::WikiLink { target, .. }) if target == "a"
    ));
    assert_eq!(frags[1].text, " and ");
    assert!(matches!(
        &frags[2].attachment,
        Some(InlineAttachment::WikiLink { target, alias, .. }) if target == "b" && alias.as_deref() == Some("B")
    ));
}

#[test]
fn unclosed_brackets_remain_plain_text() {
    let doc = parse("foo [[bar").unwrap();
    let frags = &doc.blocks[0].title.fragments;
    // 单个纯文本片段，保留原始 "foo [[bar"
    assert_eq!(frags.len(), 1);
    assert!(frags[0].attachment.is_none());
    assert_eq!(frags[0].text, "foo [[bar");
}
