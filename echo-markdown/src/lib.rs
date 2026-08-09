//! echo-markdown - `Markdown` 块模型。
//!
//! 提供 `Markdown` 文本到块树的解析、块树到 `Markdown` 的序列化，
//! 以及 Obsidian 风格 `WikiLink`（`[[target|alias]]`）的后处理支持。
//!
//! 数据结构按 `WYSIWYG` 编辑器需求设计：
//! - 块树为 source of truth，`Markdown` 文本是序列化产物；
//! - `Inline` 采用扁平 `InlineTextTree` + `InlineStyle` bitfield，
//!   支持未来增量编辑（切片 / 就地更新），无需递归 `AST` 重构。
//!
//! # 使用示例
//!
//! ```
//! use echo_markdown::{parse, to_markdown};
//!
//! let doc = parse("# Hello\n\n正文带 [[wiki link]]。").expect("parse");
//! let md = to_markdown(&doc);
//! assert!(md.contains("# Hello"));
//! ```

#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unimplemented,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic
)]
#![forbid(unsafe_code)]

mod block;
mod document;
mod error;
mod inline;
mod parser;
mod serialize;
mod wikilink;

pub use block::{Block, BlockKind, CalloutVariant, TableAlign, TableCell, TableData};
pub use document::Document;
pub use error::{MarkdownError, MarkdownResult};
pub use inline::{EmbedKind, InlineAttachment, InlineFragment, InlineStyle, InlineTextTree};
pub use parser::parse;
pub use serialize::to_markdown;
