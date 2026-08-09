# echo-markdown 开发文档

## 任务清单

### 块模型

- [x] 定义块类型 (`block.rs` - `BlockKind` 枚举，含 Paragraph / Heading / CodeBlock / List / Table / Callout / Frontmatter / Mermaid 等)
- [x] 定义块结构 (`block.rs` - `Block` 结构体，含 kind / title / children / code / table / block_id)
- [x] 定义 Callout 变体 (`block.rs` - `CalloutVariant` 枚举，覆盖 Obsidian 所有内置类型)
- [x] 定义表格模型 (`block.rs` - `TableData` / `TableCell` / `TableAlign`)
- [x] 定义文档结构 (`document.rs` - `Document` 结构体，顶层块的有序集合)

### 内联模型

- [x] 定义内联树 (`inline.rs` - `InlineTextTree` 扁平 fragment 列表)
- [x] 定义内联片段 (`inline.rs` - `InlineFragment` 结构体，含 text / style / attachment)
- [x] 定义内联样式 (`inline.rs` - `InlineStyle` bitfield，支持 bold / italic / strikethrough / code / highlight)
- [x] 定义附件类型 (`inline.rs` - `InlineAttachment` 枚举，含 Link / WikiLink / Image / Embed / Tag / Comment 等)
- [x] 定义嵌入类型 (`inline.rs` - `EmbedKind` 枚举，含 Image / File / Note)

### 解析器

- [x] 实现 Markdown 解析 (`parser.rs` - `parse()` 函数，基于 pulldown-cmark)
- [x] 支持 YAML frontmatter 提取 (`parser.rs` - `extract_frontmatter()`)
- [x] 支持表格解析 (`parser.rs` - `TableBuilder` 状态机)
- [x] 支持 Callout 解析 (`parser.rs` - `try_make_callout()`，识别 `[!VARIANT]` 语法)
- [x] 支持脚注解析 (`parser.rs` - `ENABLE_FOOTNOTES`)
- [x] 支持定义列表 (`parser.rs` - `ENABLE_DEFINITION_LIST`)
- [x] 支持数学公式 (`parser.rs` - `ENABLE_MATH`)
- [x] 支持任务列表 (`parser.rs` - `ENABLE_TASKLISTS`)
- [x] 支持删除线 (`parser.rs` - `ENABLE_STRIKETHROUGH`)

### 序列化

- [x] 实现文档序列化 (`serialize.rs` - `to_markdown()` 函数)
- [x] 支持所有块类型序列化 (Paragraph / Heading / CodeBlock / List / Table / Callout / Frontmatter / Mermaid)
- [x] 支持所有内联样式序列化 (bold / italic / strikethrough / code / highlight)
- [x] 支持所有附件类型序列化 (Link / WikiLink / Image / Embed / Tag / Comment / Math)

### WikiLink 后处理

- [x] 实现 WikiLink 分割 (`wikilink.rs` - `split_wikilinks()`)
- [x] 实现嵌入分割 (`wikilink.rs` - `split_embeds()`)
- [x] 实现注释分割 (`wikilink.rs` - `split_comments()`)
- [x] 实现高亮分割 (`wikilink.rs` - `split_highlights()`)
- [x] 实现标签分割 (`wikilink.rs` - `split_tags()`)
- [x] 使用 memchr 加速字节扫描 (`wikilink.rs` - `find_close()` / `find_close_equals()`)

### 测试与基准

- [x] 单元测试（block / document / inline / serialize / wikilink / error 模块，43 passed）
- [x] 集成测试（new_features / new_formats / parse / roundtrip / wikilink，76 passed）
- [x] 基准测试 (`markdown_bench` - 14 项)：解析 / 序列化 / 往返 / WikiLink 后处理 / 内联树操作

## 当前架构

```
echo-markdown/
├── Cargo.toml            # 依赖: pulldown-cmark, thiserror, memchr, echo-core
├── benches/
│   └── markdown_bench.rs # criterion 基准测试
└── src/
    ├── lib.rs            # 模块入口，导出公共 API
    ├── block.rs          # 块模型 (BlockKind / Block / CalloutVariant / TableData)
    ├── document.rs       # 文档结构 (Document)
    ├── inline.rs         # 内联模型 (InlineTextTree / InlineFragment / InlineStyle / InlineAttachment)
    ├── parser.rs         # Markdown 解析器 (parse / extract_frontmatter / try_make_callout)
    ├── serialize.rs      # 序列化 (to_markdown)
    ├── wikilink.rs       # WikiLink 后处理 (split_wikilinks / split_embeds / split_comments / split_highlights / split_tags)
    └── error.rs          # 错误类型 (MarkdownError / MarkdownResult，已统一到 echo_core::EchoError)
```

## 技术实现

### 依赖选择

| 依赖 | 版本 | 用途 |
|------|------|------|
| pulldown-cmark | 0.12 | Markdown 解析（CommonMark 兼容 + 扩展） |
| thiserror | 2 | 结构化错误类型 |
| memchr | 2.7 | SIMD 加速字节扫描（WikiLink 后处理） |
| echo-core | path | 统一错误类型 |

### 解析流程

```
Markdown 文本
    │
    ▼
extract_frontmatter()  ── 提取 YAML frontmatter
    │
    ▼
pulldown-cmark Parser  ── CommonMark + 扩展（表格/删除线/任务列表/脚注/数学）
    │
    ▼
ParseState.handle()    ── 事件驱动构建块树
    │
    ├── 块事件 (Start/End Block) → 维护容器栈
    ├── 内联事件 (Text/SoftBreak) → 收集到 text_buffer
    ├── 样式事件 (Strong/Emphasis) → 维护 style_stack
    └── 链接事件 (Link/Image) → 维护 attachment_stack
    │
    ▼
try_make_callout()     ── 识别引用块中的 [!VARIANT] 语法
    │
    ▼
split_wikilinks()      ── 后处理：分割 WikiLink / 嵌入 / 注释 / 高亮 / 标签
    │
    ▼
Document (块树)
```

### 支持的 Markdown 格式

| 格式 | 语法 | 解析 | 序列化 |
|------|------|:----:|:------:|
| 段落 | 纯文本 | ✅ | ✅ |
| 标题 | `# H1` ~ `###### H6` | ✅ | ✅ |
| Setext 标题 | `H1\n===` / `H2\n---` | ✅ | ✅ |
| 粗体 | `**bold**` | ✅ | ✅ |
| 斜体 | `*italic*` | ✅ | ✅ |
| 删除线 | `~~strike~~` | ✅ | ✅ |
| 行内代码 | `` `code` `` | ✅ | ✅ |
| 高亮 | `==highlight==` | ✅ | ✅ |
| 代码块 | ` ```lang ` | ✅ | ✅ |
| 无序列表 | `- item` / `* item` | ✅ | ✅ |
| 有序列表 | `1. item` | ✅ | ✅ |
| 任务列表 | `- [ ]` / `- [x]` | ✅ | ✅ |
| 引用块 | `> quote` | ✅ | ✅ |
| 表格 | `\| col \|` | ✅ | ✅ |
| 分隔线 | `---` / `***` | ✅ | ✅ |
| 行内链接 | `[text](url)` | ✅ | ✅ |
| 图片 | `![alt](url)` | ✅ | ✅ |
| 自动链接 | `<url>` | ✅ | ✅ |
| 脚注 | `[^label]` | ✅ | ✅ |
| 定义列表 | `term : description` | ✅ | ✅ |
| 行内数学 | `$...$` | ✅ | ✅ |
| 块级数学 | `$$...$$` | ✅ | ✅ |
| HTML 块 | `<div>` | ✅ | ✅ |
| 行内 HTML | `<span>` | ✅ | ✅ |
| **YAML Frontmatter** | `---\n...\n---` | ✅ | ✅ |
| **Callout** | `> [!NOTE] Title` | ✅ | ✅ |
| **Mermaid** | ` ```mermaid ` | ✅ | ✅ |
| **WikiLink** | `[[target|alias]]` | ✅ | ✅ |
| **Embed** | `![[file.png]]` | ✅ | ✅ |
| **Tag** | `#tag` | ✅ | ✅ |
| **Comment** | `%%comment%%` | ✅ | ✅ |
| **Block ID** | `^blockid` | ✅ | ✅ |

### Callout 支持

覆盖 Obsidian 所有内置 Callout 类型及其别名：

| 变体 | 语法 | 别名 |
|------|------|------|
| Note | `[!note]` | — |
| Info | `[!info]` | `[!todo]` |
| Tip | `[!tip]` | `[!hint]` / `[!important]` |
| Warning | `[!warning]` | `[!caution]` / `[!attention]` |
| Danger | `[!danger]` | `[!error]` |
| Success | `[!success]` | `[!check]` / `[!done]` |
| Question | `[!question]` | `[!help]` / `[!faq]` |
| Abstract | `[!abstract]` | `[!summary]` / `[!tldr]` |
| Quote | `[!quote]` | `[!cite]` |
| Bug | `[!bug]` | — |
| Example | `[!example]` | — |
| Failure | `[!failure]` | `[!fail]` / `[!missing]` |
| 自定义 | `[!custom]` | — |

Callout 支持折叠语法：
- `> [!NOTE]-` 折叠
- `> [!NOTE]+` 展开
- `> [!NOTE]` 无折叠

### WikiLink 后处理

解析完成后，`split_wikilinks()` 对段落文本进行后处理，识别 Obsidian 特有语法：

| 语法 | 类型 | 说明 |
|------|------|------|
| `[[target]]` | WikiLink | 笔记链接 |
| `[[target\|alias]]` | WikiLink | 带别名的链接 |
| `[[page#heading]]` | WikiLink | 标题链接 |
| `[[page#^blockid]]` | WikiLink | 块链接 |
| `![[file.png]]` | Embed | 图片/文件嵌入 |
| `![[note]]` | Embed | 笔记嵌入 |
| `%%comment%%` | Comment | 注释（不渲染） |
| `==text==` | Highlight | 高亮 |
| `#tag` | Tag | 标签 |

## 使用示例

### 解析与序列化

```rust
use echo_markdown::{parse, to_markdown};

let doc = parse("# Hello\n\n正文带 [[wiki link]]。").expect("parse");
let md = to_markdown(&doc);
assert!(md.contains("# Hello"));
```

### 构建文档

```rust
use echo_markdown::{Block, BlockKind, Document, InlineTextTree};

let mut doc = Document::new();
doc.push(Block {
    kind: BlockKind::Heading { level: 1 },
    title: InlineTextTree::from_text("标题"),
    children: Vec::new(),
    code: None,
    table: None,
    block_id: None,
});
doc.push(Block {
    kind: BlockKind::Paragraph,
    title: InlineTextTree::from_text("段落文本"),
    children: Vec::new(),
    code: None,
    table: None,
    block_id: None,
});
```

### 构建 Callout

```rust
use echo_markdown::{Block, BlockKind, CalloutVariant, InlineTextTree};

let callout = Block {
    kind: BlockKind::Callout {
        variant: CalloutVariant::Note,
        title: Some("提示".to_string()),
        folded: None,
    },
    title: InlineTextTree::new(),
    children: vec![Block {
        kind: BlockKind::Paragraph,
        title: InlineTextTree::from_text("Callout 内容"),
        children: Vec::new(),
        code: None,
        table: None,
        block_id: None,
    }],
    code: None,
    table: None,
    block_id: None,
};
```

### 构建表格

```rust
use echo_markdown::{Block, BlockKind, InlineTextTree, TableAlign, TableCell, TableData};

let table = TableData {
    headers: vec![
        TableCell {
            content: InlineTextTree::from_text("名称"),
            align: Some(TableAlign::Left),
        },
        TableCell {
            content: InlineTextTree::from_text("值"),
            align: Some(TableAlign::Right),
        },
    ],
    rows: vec![vec![
        TableCell {
            content: InlineTextTree::from_text("item1"),
            align: None,
        },
        TableCell {
            content: InlineTextTree::from_text("100"),
            align: None,
        },
    ]],
};
```

## 公共 API

### 核心函数

| 函数 | 签名 | 说明 |
|------|------|------|
| `parse` | `fn(&str) -> MarkdownResult<Document>` | 解析 Markdown 文本为文档块树 |
| `to_markdown` | `fn(&Document) -> String` | 将文档块树序列化为 Markdown 文本 |

### 块模型

| 类型 | 说明 |
|------|------|
| `Block` | 文档块：kind + title + children + code + table + block_id |
| `BlockKind` | 块类型枚举（17 种变体） |
| `CalloutVariant` | Callout 变体枚举（13 种内置 + Other） |
| `TableData` | 表格数据：headers + rows |
| `TableCell` | 表格单元格：content + align |
| `TableAlign` | 表格对齐：Left / Center / Right |

### 内联模型

| 类型 | 说明 |
|------|------|
| `InlineTextTree` | 内联内容树：扁平 fragment 列表 |
| `InlineFragment` | 内联片段：text + style + attachment |
| `InlineStyle` | 内联样式 bitfield（bold / italic / strikethrough / code / highlight） |
| `InlineAttachment` | 附件类型枚举（11 种变体） |
| `EmbedKind` | 嵌入类型：Image / File / Note |

### WikiLink 后处理

| 函数 | 签名 | 说明 |
|------|------|------|
| `split_wikilinks` | `fn(&str) -> Vec<WikiSegment>` | 分割文本中的 WikiLink |
| `split_embeds` | `fn(&str) -> Vec<WikiSegment>` | 分割文本中的嵌入 |
| `split_comments` | `fn(&str) -> Vec<CommentSegment>` | 分割文本中的注释 |
| `split_highlights` | `fn(&str) -> Vec<HighlightSegment>` | 分割文本中的高亮 |
| `split_tags` | `fn(&str) -> Vec<TagSegment>` | 分割文本中的标签 |

### 错误类型

| 类型 | 说明 |
|------|------|
| `MarkdownError` | **已弃用**：使用 [`echo_core::EchoError`] 代替，保留以维持向后兼容 |
| `MarkdownResult<T>` | 结果别名，等同于 `echo_core::EchoResult<T>` |

## 错误层级

```
EchoError (统一错误类型 — 来自 echo-core)
└── Markdown { message }     <- Markdown 处理错误

向后兼容类型别名：
├── MarkdownError = EchoError
└── MarkdownResult<T> = EchoResult<T>
```

## 性能基准

| 基准测试 | 耗时 | 说明 |
|---------|------|------|
| `parse_simple_50_paragraphs` | 18.56 µs | 50 个纯文本段落 |
| `parse_complex_mixed` | 92.50 µs | 混合内容（标题/列表/代码/表格/引用） |
| `parse_wikilink_heavy_200` | 510.21 µs | 200 个 WikiLink |
| `roundtrip_complex` | 165.69 µs | 解析+序列化+再解析 |
| `wikilink_split_plain_text` | 645 ns | 纯文本 WikiLink 分割 |
| `wikilink_split_dense_100` | 78.84 µs | 100 个 WikiLink 分割 |
| `serialize_complex` | 3.47 µs | 复杂文档序列化 |
| `serialize_wikilink_heavy` | 7.86 µs | WikiLink 密集文档序列化 |
| `inline_tree_build_100` | 6.65 µs | 构建 100 个片段的内联树 |
| `inline_tree_plain_text_100` | 597 ns | 提取 100 个片段的纯文本 |
| `style_merge` | 2.68 ns | 样式 OR 合并 |
| `block_new` | 14.43 ns | 创建空块 |
| `document_push_100` | 2.66 µs | 追加 100 个块 |

## 注意事项

- 解析基于 `pulldown-cmark`，CommonMark 兼容，支持表格/删除线/任务列表/脚注/数学等扩展
- `Inline` 采用扁平 `Vec<InlineFragment>` + `InlineStyle` bitfield，而非递归 AST
- 嵌套格式（如 `**bold _italic_**`）通过 OR 组合 `InlineStyle` 标记到重叠区域的单个 fragment
- 扁平 fragment 模型下，嵌套格式的序列化可能不完全还原原始嵌套结构，但 `parse -> serialize -> parse` 的块树往返保持一致
- WikiLink 后处理在解析完成后进行，使用 `memchr` 进行 SIMD 加速字节扫描
- Callout 解析在块树构建后执行，识别引用块中的 `[!VARIANT]` 语法
- YAML frontmatter 在解析前提取，作为首个块插入文档
- Mermaid 图表通过 ` ```mermaid ` 代码块识别，序列化为独立块类型
- 错误类型已统一到 `echo_core::EchoError`，`MarkdownError` 为向后兼容的类型别名
