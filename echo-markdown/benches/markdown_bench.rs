//! echo-markdown 基准测试
//!
//! 覆盖以下场景：
//! - 解析各种复杂度的 `Markdown` 文档（段落 / 标题 / 列表 / 代码块 / 表格 / `WikiLink`）
//! - 序列化 `Document` → `Markdown` 文本
//! - 往返 `parse → to_markdown → parse` 一致性
//! - `WikiLink` 后处理 `split_wikilinks`
//! - 内联树与样式操作

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use echo_markdown::{
    Block, BlockKind, Document, InlineFragment, InlineStyle, InlineTextTree, parse, to_markdown,
};

/// 生成简单的纯段落文档。
fn make_simple_doc() -> String {
    let mut s = String::new();
    for i in 0..50 {
        s.push_str(&format!(
            "这是第 {i} 段普通文本，包含一些中文与 ASCII content。\n\n"
        ));
    }
    s
}

/// 生成包含多种块类型的复杂文档。
fn make_complex_doc() -> String {
    let mut s = String::new();
    s.push_str("# 文档标题\n\n");
    s.push_str("引言段落，含 **粗体**、*斜体*、~~删除线~~ 与 `行内代码`。\n\n");
    s.push_str("含 [[wiki link|别名]] 与 [普通链接](https://example.com) 的段落。\n\n");

    // 列表
    s.push_str("## 列表\n\n");
    for i in 0..20 {
        s.push_str(&format!("- 列表项 {i}\n"));
    }
    s.push('\n');
    for i in 0..10 {
        s.push_str(&format!("{}. 有序项 {i}\n", i + 1));
    }
    s.push('\n');
    s.push_str("- [ ] 未完成\n- [x] 已完成\n\n");

    // 代码块
    s.push_str("## 代码\n\n");
    s.push_str("```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\n");
    s.push_str("```python\nprint('hi')\n```\n\n");

    // 引用
    s.push_str("> 引用块第一行\n> 引用块第二行\n\n");

    // 表格
    s.push_str("| 名称 | 值 | 描述 |\n");
    s.push_str("|:-----|--:|:----:|\n");
    for i in 0..20 {
        s.push_str(&format!("| name{i} | {i} | desc{i} |\n"));
    }
    s.push('\n');

    // 分隔线
    s.push_str("---\n\n");

    // 多个段落
    for i in 0..30 {
        s.push_str(&format!("尾随段落 {i}，含 [[note{i}]]。\n\n"));
    }
    s
}

/// 生成纯 `WikiLink` 密集文档。
fn make_wikilink_heavy_doc() -> String {
    let mut s = String::new();
    for i in 0..200 {
        s.push_str(&format!("参见 [[note{i}|显示名{i}]] 与 [[plain{i}]]。\n\n"));
    }
    s
}

/// 基准测试：解析简单段落文档
fn bench_parse_simple(c: &mut Criterion) {
    let md = make_simple_doc();
    c.bench_function("parse_simple_50_paragraphs", |b| {
        b.iter(|| {
            black_box(parse(black_box(&md)).unwrap());
        });
    });
}

/// 基准测试：解析复杂文档
fn bench_parse_complex(c: &mut Criterion) {
    let md = make_complex_doc();
    c.bench_function("parse_complex_mixed", |b| {
        b.iter(|| {
            black_box(parse(black_box(&md)).unwrap());
        });
    });
}

/// 基准测试：解析 `WikiLink` 密集文档
fn bench_parse_wikilink_heavy(c: &mut Criterion) {
    let md = make_wikilink_heavy_doc();
    c.bench_function("parse_wikilink_heavy_200", |b| {
        b.iter(|| {
            black_box(parse(black_box(&md)).unwrap());
        });
    });
}

/// 基准测试：解析空文档
fn bench_parse_empty(c: &mut Criterion) {
    c.bench_function("parse_empty", |b| {
        b.iter(|| {
            black_box(parse(black_box("")).unwrap());
        });
    });
}

/// 基准测试：序列化复杂文档
fn bench_serialize_complex(c: &mut Criterion) {
    let md = make_complex_doc();
    let doc = parse(&md).unwrap();
    c.bench_function("serialize_complex", |b| {
        b.iter(|| {
            black_box(to_markdown(black_box(&doc)));
        });
    });
}

/// 基准测试：序列化 `WikiLink` 密集文档
fn bench_serialize_wikilink_heavy(c: &mut Criterion) {
    let md = make_wikilink_heavy_doc();
    let doc = parse(&md).unwrap();
    c.bench_function("serialize_wikilink_heavy", |b| {
        b.iter(|| {
            black_box(to_markdown(black_box(&doc)));
        });
    });
}

/// 基准测试：往返 `parse → to_markdown → parse`
fn bench_roundtrip_complex(c: &mut Criterion) {
    let md = make_complex_doc();
    c.bench_function("roundtrip_complex", |b| {
        b.iter(|| {
            let first = parse(black_box(&md)).unwrap();
            let serialized = to_markdown(&first);
            let second = parse(&serialized).unwrap();
            black_box((first, second));
        });
    });
}

/// 基准测试：`WikiLink` 后处理 — 无 `WikiLink` 文本
fn bench_wikilink_split_plain(c: &mut Criterion) {
    // 通过 parse 触发 split_wikilinks：纯文本段落
    let md = "纯文本段落，没有任何 wiki link，只有普通中文与 ASCII content。";
    c.bench_function("wikilink_split_plain_text", |b| {
        b.iter(|| {
            black_box(parse(black_box(md)).unwrap());
        });
    });
}

/// 基准测试：`WikiLink` 后处理 — 密集 `WikiLink` 文本
fn bench_wikilink_split_dense(c: &mut Criterion) {
    // 单个段落内含大量 WikiLink
    let mut md = String::from("段落起始 ");
    for i in 0..100 {
        md.push_str(&format!("[[note{i}|alias{i}]] 之后 "));
    }
    md.push_str("段落结束。");
    c.bench_function("wikilink_split_dense_100", |b| {
        b.iter(|| {
            black_box(parse(black_box(&md)).unwrap());
        });
    });
}

/// 基准测试：内联树构建
fn bench_inline_tree_build(c: &mut Criterion) {
    c.bench_function("inline_tree_build_100_fragments", |b| {
        b.iter(|| {
            let mut tree = InlineTextTree::new();
            for i in 0..100 {
                tree.push(InlineFragment {
                    text: format!("fragment-{i}"),
                    style: InlineStyle {
                        bold: i % 2 == 0,
                        italic: i % 3 == 0,
                        ..InlineStyle::none()
                    },
                    attachment: None,
                });
            }
            black_box(tree);
        });
    });
}

/// 基准测试：内联树 `plain_text` 提取
fn bench_inline_tree_plain_text(c: &mut Criterion) {
    let mut tree = InlineTextTree::new();
    for i in 0..100 {
        tree.push(InlineFragment {
            text: format!("fragment-{i} "),
            style: InlineStyle::none(),
            attachment: None,
        });
    }
    c.bench_function("inline_tree_plain_text_100", |b| {
        b.iter(|| {
            black_box(tree.plain_text());
        });
    });
}

/// 基准测试：样式 OR 合并
fn bench_style_merge(c: &mut Criterion) {
    let style_a = InlineStyle {
        bold: true,
        ..InlineStyle::none()
    };
    let style_b = InlineStyle {
        italic: true,
        strikethrough: true,
        ..InlineStyle::none()
    };
    c.bench_function("style_merge", |b| {
        b.iter(|| {
            black_box(black_box(style_a).merge(black_box(style_b)));
        });
    });
}

/// 基准测试：`Block` 构建
fn bench_block_new(c: &mut Criterion) {
    c.bench_function("block_new", |b| {
        b.iter(|| {
            black_box(Block::new(black_box(BlockKind::Paragraph)));
        });
    });
}

/// 基准测试：`Document` 构建与 `push`
fn bench_document_push(c: &mut Criterion) {
    c.bench_function("document_push_100_blocks", |b| {
        b.iter(|| {
            let mut doc = Document::new();
            for _ in 0..100 {
                doc.push(Block::new(BlockKind::Paragraph));
            }
            black_box(doc);
        });
    });
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_complex,
    bench_parse_wikilink_heavy,
    bench_parse_empty,
    bench_serialize_complex,
    bench_serialize_wikilink_heavy,
    bench_roundtrip_complex,
    bench_wikilink_split_plain,
    bench_wikilink_split_dense,
    bench_inline_tree_build,
    bench_inline_tree_plain_text,
    bench_style_merge,
    bench_block_new,
    bench_document_push,
);
criterion_main!(benches);
