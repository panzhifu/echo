//! 文本后处理：WikiLink / Embed / Comment / Highlight / Tag 扫描。
//!
//! `pulldown-cmark` 不解析 `[[...]]`、`![[...]]`、`%%...%%`、`==...==`、`#tag`，
//! 将它们作为普通 `Text` 事件吐出。本模块在 inline 收集阶段对每个 `Text` 片段
//! 依次扫描这些模式，拆分为纯文本与各类片段。
//!
//! 扫描顺序：WikiLink/Embed → Comment → Highlight → Tag。
//! 注释内容不再进一步扫描（注释内的标记不应被解析）。

/// `WikiLink` / `Embed` 匹配结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiSegment {
    /// 纯文本。
    Text(String),
    /// `WikiLink` `[[target]]` 或 `[[target|alias]]`。
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
    /// Obsidian 嵌入 `![[target]]`。
    Embed {
        /// 嵌入目标（文件名或笔记名）。
        target: String,
        /// 嵌入类型。
        kind: crate::inline::EmbedKind,
        /// 可选别名。
        alias: Option<String>,
        /// 可选宽度。
        width: Option<u32>,
    },
}

/// 将一段文本扫描拆分为纯文本、`WikiLink` 与 `Embed` 片段。
///
/// 未闭合的 `[[` 不视为 `WikiLink`，按普通文本保留。
/// 优先检测 `![[...]]`（嵌入），再检测 `[[...]]`（链接）。
pub fn split_wikilinks(text: &str) -> Vec<WikiSegment> {
    // 快速预检：不含 [[ 则直接返回单个 Text 段
    if !text.contains("[[") {
        return vec![WikiSegment::Text(text.to_string())];
    }

    let bytes = text.as_bytes();
    let mut segments = Vec::new();
    let mut last = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        // 优先检测 ![[...]] (嵌入)
        let is_embed =
            i + 2 < bytes.len() && bytes[i] == b'!' && bytes[i + 1] == b'[' && bytes[i + 2] == b'[';
        if is_embed {
            let after = i + 3;
            match find_close(&text[after..]) {
                Some(close) => {
                    if i > last {
                        segments.push(WikiSegment::Text(text[last..i].to_string()));
                    }
                    let inner = &text[after..after + close];
                    let embed = parse_embed(inner);
                    segments.push(embed);
                    i = after + close + 2;
                    last = i;
                },
                None => {
                    i += 1;
                },
            }
            continue;
        }

        // 检测 [[...]] (链接)
        let is_open = i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[';
        if is_open {
            let after = i + 2;
            match find_close(&text[after..]) {
                Some(close) => {
                    if i > last {
                        segments.push(WikiSegment::Text(text[last..i].to_string()));
                    }
                    let inner = &text[after..after + close];
                    let link = parse_wikilink(inner);
                    segments.push(link);
                    i = after + close + 2;
                    last = i;
                },
                None => {
                    i += 1;
                },
            }
        } else {
            i += 1;
        }
    }
    if last < bytes.len() {
        segments.push(WikiSegment::Text(text[last..].to_string()));
    }
    segments
}

/// 在 `s` 中查找 `]]` 的起始字节偏移。
///
/// 使用 `memchr` SIMD 加速查找 `]` 位置，然后检查下一个字节。
fn find_close(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    loop {
        // 使用 memchr SIMD 加速查找下一个 ']'
        let found = pos + memchr::memchr(b']', &bytes[pos..])?;
        // 检查是否构成 ']]
        if found + 1 < bytes.len() && bytes[found + 1] == b']' {
            return Some(found);
        }
        pos = found + 1;
    }
}

/// 解析 `[[...]]` 内部：`target`、`target|alias`、`target#heading`、`target#^blockid`。
fn parse_wikilink(inner: &str) -> WikiSegment {
    let (target, alias) = match inner.split_once('|') {
        Some((t, a)) => (t.trim().to_string(), Some(a.trim().to_string())),
        None => (inner.trim().to_string(), None),
    };

    // 检测标题链接或块链接
    let (target, heading, block_id) = if let Some(idx) = target.find('#') {
        let (t, h) = target.split_at(idx);
        let h = &h[1..]; // 去掉 #
        if let Some(stripped) = h.strip_prefix('^') {
            (
                t.trim().to_string(),
                None,
                Some(stripped.trim().to_string()),
            )
        } else {
            (t.trim().to_string(), Some(h.trim().to_string()), None)
        }
    } else {
        (target, None, None)
    };

    WikiSegment::WikiLink {
        target,
        alias,
        heading,
        block_id,
    }
}

/// 解析 `![[...]]` 内部：`target`、`target|alias`、`target|width`。
fn parse_embed(inner: &str) -> WikiSegment {
    let (name, alias, width) = if let Some(idx) = inner.rfind('|') {
        let (n, rest) = inner.split_at(idx);
        let rest = &rest[1..]; // 去掉 |
        // 尝试将 rest 解析为宽度数字，否则作为别名
        if let Ok(w) = rest.trim().parse::<u32>() {
            (n.trim().to_string(), None, Some(w))
        } else {
            (n.trim().to_string(), Some(rest.trim().to_string()), None)
        }
    } else {
        (inner.trim().to_string(), None, None)
    };

    // 判断嵌入类型：根据扩展名
    let kind = if name.contains('.') {
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico" => {
                crate::inline::EmbedKind::Image
            },
            _ => crate::inline::EmbedKind::File,
        }
    } else {
        crate::inline::EmbedKind::Note
    };

    WikiSegment::Embed {
        target: name,
        kind,
        alias,
        width,
    }
}

// ========== Comment 注释 ==========

/// `Comment` 匹配结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentSegment {
    /// 纯文本。
    Text(String),
    /// `%%comment%%` 注释。
    Comment {
        /// 注释内容。
        content: String,
    },
}

/// 将一段文本扫描拆分为纯文本与 `Comment` 片段。
pub fn split_comments(text: &str) -> Vec<CommentSegment> {
    // 快速预检：不含 %% 则直接返回单个 Text 段
    if !text.contains("%%") {
        return vec![CommentSegment::Text(text.to_string())];
    }

    let bytes = text.as_bytes();
    let mut segments = Vec::new();
    let mut last = 0usize;
    let mut i = 0usize;

    while i + 1 < bytes.len() {
        if bytes[i] == b'%' && bytes[i + 1] == b'%' {
            let after = i + 2;
            // 查找关闭的 %%
            if let Some(close) = text[after..].find("%%") {
                let close = after + close;
                if i > last {
                    segments.push(CommentSegment::Text(text[last..i].to_string()));
                }
                let content = text[after..close].to_string();
                segments.push(CommentSegment::Comment { content });
                i = close + 2;
                last = i;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    if last < bytes.len() {
        segments.push(CommentSegment::Text(text[last..].to_string()));
    }
    segments
}

// ========== Highlight 高亮 ==========

/// `Highlight` 匹配结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightSegment {
    /// 纯文本。
    Text(String),
    /// `==text==` 高亮。
    Highlight(String),
}

/// 将一段文本扫描拆分为纯文本与 `Highlight` 片段。
///
/// 匹配 `==text==` 模式，但排除 `===`（三个等号不构成高亮）。
pub fn split_highlights(text: &str) -> Vec<HighlightSegment> {
    // 快速预检：不含 == 则直接返回单个 Text 段
    if !text.contains("==") {
        return vec![HighlightSegment::Text(text.to_string())];
    }

    let bytes = text.as_bytes();
    let mut segments = Vec::new();
    let mut last = 0usize;
    let mut i = 0usize;

    while i + 1 < bytes.len() {
        if bytes[i] == b'=' && bytes[i + 1] == b'=' {
            // 排除 === 的情况
            if i + 2 < bytes.len() && bytes[i + 2] == b'=' {
                i += 1;
                continue;
            }
            let after = i + 2;
            // 查找关闭的 ==
            if let Some(close) = find_close_equals(&text[after..]) {
                let close = after + close;
                // 确保关闭的 == 后面不是 =
                if close < text.len() - 1 && text.as_bytes().get(close + 2) == Some(&b'=') {
                    i += 1;
                    continue;
                }
                if i > last {
                    segments.push(HighlightSegment::Text(text[last..i].to_string()));
                }
                let content = text[after..close].to_string();
                segments.push(HighlightSegment::Highlight(content));
                i = close + 2;
                last = i;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    if last < bytes.len() {
        segments.push(HighlightSegment::Text(text[last..].to_string()));
    }
    segments
}

/// 在 `s` 中查找 `==` 的起始字节偏移。
///
/// 使用 `memchr` SIMD 加速查找 `=` 位置，然后检查下一个字节。
fn find_close_equals(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut pos = 0;
    loop {
        let found = pos + memchr::memchr(b'=', &bytes[pos..])?;
        if found + 1 < bytes.len() && bytes[found + 1] == b'=' {
            return Some(found);
        }
        pos = found + 1;
    }
}

// ========== Tag 标签 ==========

/// `Tag` 匹配结果：一段纯文本或一个标签。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagSegment {
    /// 纯文本。
    Text(String),
    /// `#tag` 标签。
    Tag {
        /// 标签名称（不含 `#`）。
        name: String,
    },
}

/// 将一段文本扫描拆分为纯文本与 `Tag` 片段。
///
/// 标签规则：`#` 前必须是空白或文本开头，后跟字母数字/下划线/连字符/斜杠。
/// 避免匹配 URL 锚点（如 `example.com#section`）、颜色值（如 `#fff`）。
pub fn split_tags(text: &str) -> Vec<TagSegment> {
    // 快速预检：不含 # 则直接返回单个 Text 段
    if !text.contains('#') {
        return vec![TagSegment::Text(text.to_string())];
    }

    let bytes = text.as_bytes();
    let mut segments = Vec::new();
    let mut last = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'#' {
            // # 前必须是空白或文本开头（避免 URL 锚点、颜色值等）
            let preceded_by_word = if i > 0 {
                bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_'
            } else {
                false
            };

            if preceded_by_word {
                i += 1;
                continue;
            }

            // # 后必须至少有一个有效字符
            let after = i + 1;
            if after < bytes.len() && (bytes[after].is_ascii_alphanumeric() || bytes[after] == b'_')
            {
                // 找到标签名称的结束位置
                let mut end = after;
                while end < bytes.len() {
                    let c = bytes[end];
                    if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'/' {
                        end += 1;
                    } else {
                        break;
                    }
                }

                if i > last {
                    segments.push(TagSegment::Text(text[last..i].to_string()));
                }
                let name = text[after..end].to_string();
                segments.push(TagSegment::Tag { name });
                last = end;
                i = end;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    if last < bytes.len() {
        segments.push(TagSegment::Text(text[last..].to_string()));
    }
    segments
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;

    // ========== split_wikilinks 测试 ==========

    #[test]
    fn plain_text_no_wikilink() {
        let segs = split_wikilinks("hello world");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], WikiSegment::Text(t) if t == "hello world"));
    }

    #[test]
    fn simple_wikilink() {
        let segs = split_wikilinks("[[note]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::WikiLink { target, alias, .. } => {
                assert_eq!(target, "note");
                assert!(alias.is_none());
            },
            _ => panic!("expected WikiLink"),
        }
    }

    #[test]
    fn wikilink_with_alias() {
        let segs = split_wikilinks("[[note|display text]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::WikiLink { target, alias, .. } => {
                assert_eq!(target, "note");
                assert_eq!(alias.as_deref(), Some("display text"));
            },
            _ => panic!("expected WikiLink"),
        }
    }

    #[test]
    fn mixed_text_and_wikilink() {
        let segs = split_wikilinks("see [[note]] for details");
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], WikiSegment::Text(t) if t == "see "));
        assert!(matches!(&segs[1], WikiSegment::WikiLink { .. }));
        assert!(matches!(&segs[2], WikiSegment::Text(t) if t == " for details"));
    }

    #[test]
    fn unclosed_brackets_preserved_as_text() {
        let segs = split_wikilinks("foo [[bar");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], WikiSegment::Text(t) if t == "foo [[bar"));
    }

    #[test]
    fn multiple_wikilinks() {
        let segs = split_wikilinks("[[a]] and [[b|B]]");
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], WikiSegment::WikiLink { .. }));
        assert!(matches!(&segs[1], WikiSegment::Text(t) if t == " and "));
        assert!(matches!(&segs[2], WikiSegment::WikiLink { .. }));
    }

    #[test]
    fn wikilink_heading_link() {
        let segs = split_wikilinks("[[note#Section]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::WikiLink {
                target,
                heading,
                block_id,
                ..
            } => {
                assert_eq!(target, "note");
                assert_eq!(heading.as_deref(), Some("Section"));
                assert!(block_id.is_none());
            },
            _ => panic!("expected WikiLink with heading"),
        }
    }

    #[test]
    fn wikilink_block_link() {
        let segs = split_wikilinks("[[note#^block123]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::WikiLink {
                target,
                heading,
                block_id,
                ..
            } => {
                assert_eq!(target, "note");
                assert!(heading.is_none());
                assert_eq!(block_id.as_deref(), Some("block123"));
            },
            _ => panic!("expected WikiLink with block_id"),
        }
    }

    #[test]
    fn wikilink_heading_link_with_alias() {
        let segs = split_wikilinks("[[note#Section|显示文本]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::WikiLink {
                target,
                alias,
                heading,
                ..
            } => {
                assert_eq!(target, "note");
                assert_eq!(heading.as_deref(), Some("Section"));
                assert_eq!(alias.as_deref(), Some("显示文本"));
            },
            _ => panic!("expected WikiLink with heading and alias"),
        }
    }

    // ========== embed 测试 ==========

    #[test]
    fn embed_image() {
        let segs = split_wikilinks("![[photo.png]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::Embed { target, kind, .. } => {
                assert_eq!(target, "photo.png");
                assert_eq!(*kind, crate::inline::EmbedKind::Image);
            },
            _ => panic!("expected Embed"),
        }
    }

    #[test]
    fn embed_file() {
        let segs = split_wikilinks("![[document.pdf]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::Embed { target, kind, .. } => {
                assert_eq!(target, "document.pdf");
                assert_eq!(*kind, crate::inline::EmbedKind::File);
            },
            _ => panic!("expected Embed"),
        }
    }

    #[test]
    fn embed_note() {
        let segs = split_wikilinks("![[My Note]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::Embed { target, kind, .. } => {
                assert_eq!(target, "My Note");
                assert_eq!(*kind, crate::inline::EmbedKind::Note);
            },
            _ => panic!("expected Embed"),
        }
    }

    #[test]
    fn embed_with_width() {
        let segs = split_wikilinks("![[image.png|200]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::Embed { target, width, .. } => {
                assert_eq!(target, "image.png");
                assert_eq!(*width, Some(200));
            },
            _ => panic!("expected Embed with width"),
        }
    }

    #[test]
    fn embed_with_alias() {
        let segs = split_wikilinks("![[image.png|我的图片]]");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            WikiSegment::Embed { target, alias, .. } => {
                assert_eq!(target, "image.png");
                assert_eq!(alias.as_deref(), Some("我的图片"));
            },
            _ => panic!("expected Embed with alias"),
        }
    }

    #[test]
    fn embed_mixed_text() {
        let segs = split_wikilinks("看 ![[img.png]] 这张图");
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], WikiSegment::Text(t) if t == "看 "));
        assert!(matches!(&segs[1], WikiSegment::Embed { .. }));
        assert!(matches!(&segs[2], WikiSegment::Text(t) if t == " 这张图"));
    }

    // ========== split_comments 测试 ==========

    #[test]
    fn split_comments_plain_text() {
        let segs = split_comments("hello world");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], CommentSegment::Text(t) if t == "hello world"));
    }

    #[test]
    fn split_comments_single() {
        let segs = split_comments("%%这是注释%%");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            CommentSegment::Comment { content } => assert_eq!(content, "这是注释"),
            CommentSegment::Text(_) => panic!("expected Comment"),
        }
    }

    #[test]
    fn split_comments_in_text() {
        let segs = split_comments("前面 %%注释%% 后面");
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], CommentSegment::Text(t) if t == "前面 "));
        assert!(matches!(&segs[1], CommentSegment::Comment { content } if content == "注释"));
        assert!(matches!(&segs[2], CommentSegment::Text(t) if t == " 后面"));
    }

    #[test]
    fn split_comments_unclosed() {
        let segs = split_comments("前面 %%未关闭");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], CommentSegment::Text(_)));
    }

    #[test]
    fn split_comments_does_not_nested_parse() {
        // 注释内的 #tag 不应被解析为标签
        let segs = split_comments("%%内容 #tag%%");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], CommentSegment::Comment { .. }));
    }

    // ========== split_highlights 测试 ==========

    #[test]
    fn split_highlights_plain_text() {
        let segs = split_highlights("hello world");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], HighlightSegment::Text(t) if t == "hello world"));
    }

    #[test]
    fn split_highlights_single() {
        let segs = split_highlights("==高亮文本==");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            HighlightSegment::Highlight(text) => assert_eq!(text, "高亮文本"),
            HighlightSegment::Text(_) => panic!("expected Highlight"),
        }
    }

    #[test]
    fn split_highlights_in_text() {
        let segs = split_highlights("前面 ==高亮== 后面");
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], HighlightSegment::Text(t) if t == "前面 "));
        assert!(matches!(&segs[1], HighlightSegment::Highlight(t) if t == "高亮"));
        assert!(matches!(&segs[2], HighlightSegment::Text(t) if t == " 后面"));
    }

    #[test]
    fn split_highlights_triple_equals_not_match() {
        let segs = split_highlights("===不分隔===");
        // === 不应被识别为高亮
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], HighlightSegment::Text(_)));
    }

    #[test]
    fn split_highlights_unclosed() {
        let segs = split_highlights("==未关闭");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], HighlightSegment::Text(_)));
    }

    // ========== split_tags 测试 ==========

    #[test]
    fn split_tags_plain_text() {
        let segs = split_tags("hello world");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], TagSegment::Text(t) if t == "hello world"));
    }

    #[test]
    fn split_tags_single_tag() {
        let segs = split_tags("#rust");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            TagSegment::Tag { name } => assert_eq!(name, "rust"),
            TagSegment::Text(_) => panic!("expected Tag"),
        }
    }

    #[test]
    fn split_tags_tag_in_text() {
        let segs = split_tags("学习 #rust 和 #python");
        assert_eq!(segs.len(), 4);
        assert!(matches!(&segs[0], TagSegment::Text(t) if t == "学习 "));
        assert!(matches!(&segs[1], TagSegment::Tag { name } if name == "rust"));
        assert!(matches!(&segs[2], TagSegment::Text(t) if t == " 和 "));
        assert!(matches!(&segs[3], TagSegment::Tag { name } if name == "python"));
    }

    #[test]
    fn split_tags_url_anchor_not_tag() {
        let segs = split_tags("访问 example.com#section");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], TagSegment::Text(_)));
    }

    #[test]
    fn split_tags_color_hex_not_tag() {
        let segs = split_tags("#fff");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], TagSegment::Tag { name } if name == "fff"));
    }

    #[test]
    fn split_tags_with_hyphen_and_slash() {
        let segs = split_tags("#rust-lang/programming");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            TagSegment::Tag { name } => assert_eq!(name, "rust-lang/programming"),
            TagSegment::Text(_) => panic!("expected Tag"),
        }
    }

    #[test]
    fn split_tags_no_space_before_hash_not_tag() {
        let segs = split_tags("C# 编程");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], TagSegment::Text(_)));
    }
}
