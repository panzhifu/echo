use thiserror::Error;

/// Markdown 处理错误类型。
///
/// 用于解析、序列化过程中遇到的结构性错误。
/// 注意 `pulldown-cmark` 解析本身高度容错（几乎不失败），
/// 此类型主要为未来编辑运行时与 IO 预留统一错误出口。
#[derive(Debug, Error)]
pub enum MarkdownError {
    /// Markdown 解析失败。
    #[error("markdown parse error: {0}")]
    Parse(String),
}

/// `Result` 别名，错误类型为 [`MarkdownError`]。
pub type MarkdownResult<T> = Result<T, MarkdownError>;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::unnecessary_literal_unwrap)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_message_contains_context() {
        let err = MarkdownError::Parse("unexpected eof".to_string());
        let msg = err.to_string();
        assert!(msg.contains("markdown parse error"));
        assert!(msg.contains("unexpected eof"));
    }

    #[test]
    fn error_messages_are_ascii() {
        let err = MarkdownError::Parse("test".to_string());
        assert!(err.to_string().is_ascii(), "error message should be ASCII");
    }
}
