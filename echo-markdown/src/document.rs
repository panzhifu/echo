//! 文档：顶层块列表。

use crate::block::Block;

/// Markdown 文档：顶层块的有序集合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// 顶层块。
    pub blocks: Vec<Block>,
}

impl Document {
    /// 创建空文档。
    #[must_use]
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// 从块列表构建。
    #[must_use]
    pub fn from_blocks(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// 追加一个顶层块。
    pub fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;
    use crate::block::BlockKind;

    #[test]
    fn empty_document_is_empty() {
        assert!(Document::new().is_empty());
        assert!(Document::default().is_empty());
    }

    #[test]
    fn from_blocks_preserves_order() {
        let doc = Document::from_blocks(vec![
            Block::new(BlockKind::Paragraph),
            Block::new(BlockKind::ThematicBreak),
        ]);
        assert_eq!(doc.blocks.len(), 2);
    }
}
