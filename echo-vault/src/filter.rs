//! gitignore 风格的文件忽略过滤器。
//!
//! 基于 [`ignore`] crate 实现，支持标准的 `.gitignore` 模式语法，
//! 包括 `*` 通配符、`**` 递归匹配、`/` 路径锚定、`!` 取反等。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::watcher::VaultError;

/// gitignore 风格的文件忽略过滤器。
///
/// 基于 [`ignore`] crate 的 [`Gitignore`] 实现，支持标准 gitignore 模式。
/// 内部使用 [`Arc`] 共享 `Gitignore`，使过滤器本身可廉价克隆。
///
/// # 示例
///
/// ```
/// use echo_vault::IgnoreFilter;
/// use std::path::Path;
///
/// let filter = IgnoreFilter::new(
///     Path::new("/home/user/vault"),
///     &["*.tmp".to_string(), ".git/".to_string()],
/// ).unwrap();
///
/// assert!(filter.is_ignored(Path::new("/home/user/vault/note.tmp")));
/// assert!(!filter.is_ignored(Path::new("/home/user/vault/note.md")));
/// ```
#[derive(Clone)]
pub struct IgnoreFilter {
    matcher: Arc<Gitignore>,
    root: PathBuf,
}

impl IgnoreFilter {
    /// 创建一个新的忽略过滤器。
    ///
    /// # 参数
    ///
    /// - `root`: 模式匹配的根目录，用于将绝对路径转换为相对路径
    /// - `patterns`: gitignore 风格的模式列表
    ///
    /// # Errors
    ///
    /// 返回 [`VaultError::Init`] 当模式解析失败时。
    pub fn new(root: &Path, patterns: &[String]) -> Result<Self, VaultError> {
        let mut builder = GitignoreBuilder::new(root);
        for pattern in patterns {
            builder
                .add_line(None, pattern)
                .map_err(|e| VaultError::Init(e.to_string()))?;
        }
        let matcher = builder
            .build()
            .map_err(|e| VaultError::Init(e.to_string()))?;
        Ok(Self {
            matcher: Arc::new(matcher),
            root: root.to_path_buf(),
        })
    }

    /// 检查给定路径是否应被忽略。
    ///
    /// 路径会先相对于 `root` 转换为相对路径，然后与所有模式进行匹配。
    /// 同时检查路径本身及其所有父目录（例如 `.git/config` 会匹配 `.git/`）。
    ///
    /// 对于不在 `root` 下的路径，使用文件名进行最佳匹配。
    ///
    /// # 参数
    ///
    /// - `path`: 要检查的文件/目录路径
    #[must_use]
    pub fn is_ignored(&self, path: &Path) -> bool {
        if let Ok(relative) = path.strip_prefix(&self.root) {
            self.matcher
                .matched_path_or_any_parents(relative, false)
                .is_ignore()
        } else {
            // 路径不在 root 下 - 使用文件名进行最佳匹配
            let file_name = path.file_name().map_or(path, Path::new);
            self.matcher.matched(file_name, false).is_ignore()
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn make_filter(patterns: &[&str]) -> IgnoreFilter {
        let patterns: Vec<String> = patterns.iter().map(|s| (*s).to_string()).collect();
        IgnoreFilter::new(Path::new("/home/user/vault"), &patterns).unwrap()
    }

    #[test]
    fn matches_glob_pattern() {
        let filter = make_filter(&["*.tmp"]);
        assert!(filter.is_ignored(Path::new("/home/user/vault/note.tmp")));
        assert!(!filter.is_ignored(Path::new("/home/user/vault/note.md")));
    }

    #[test]
    fn matches_directory_pattern() {
        let filter = make_filter(&[".git/"]);
        assert!(filter.is_ignored(Path::new("/home/user/vault/.git/config")));
        assert!(!filter.is_ignored(Path::new("/home/user/vault/notes.md")));
    }

    #[test]
    fn matches_recursive_pattern() {
        let filter = make_filter(&["**/node_modules/"]);
        assert!(filter.is_ignored(Path::new(
            "/home/user/vault/subdir/node_modules/package.json"
        )));
    }

    #[test]
    fn non_matching_path_not_ignored() {
        let filter = make_filter(&["*.log", "*.tmp", ".git/"]);
        assert!(!filter.is_ignored(Path::new("/home/user/vault/src/main.rs")));
    }

    #[test]
    fn path_outside_root_uses_filename_fallback() {
        let filter = make_filter(&["*.tmp"]);
        // 路径不在 root 下，使用文件名进行匹配
        assert!(filter.is_ignored(Path::new("/other/path/file.tmp")));
        assert!(!filter.is_ignored(Path::new("/other/path/file.md")));
    }
}
