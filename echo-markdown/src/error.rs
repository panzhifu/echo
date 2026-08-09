//! Markdown 错误类型。
//!
//! 本模块已统一到 [`echo_core::EchoError`]。
//! 保留此模块以维持向后兼容的公共 API。
//!
//! # 迁移说明
//!
//! 所有 Markdown 相关错误现在使用 [`echo_core::EchoError::Markdown`] 变体。
//! 请直接使用 [`echo_core::EchoError`] 和 [`echo_core::MarkdownResult`]。

pub use echo_core::{EchoError, MarkdownResult};

/// Markdown 处理错误类型。
///
/// **已弃用**：使用 [`echo_core::EchoError`] 代替。
/// 保留此类型以维持向后兼容性。
pub type MarkdownError = EchoError;
