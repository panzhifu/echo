//! 编译过滤器缓存。
//!
//! 缓存已编译的 [`IgnoreFilter`]，避免重复编译相同的 glob 模式。
//! 使用 [`std::sync::LazyLock`] + [`std::sync::Mutex`] 实现线程安全的全局缓存。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use crate::filter::IgnoreFilter;
use crate::watcher::VaultError;

/// 过滤器缓存类型：(root, patterns) -> `IgnoreFilter`。
type FilterCache = HashMap<(PathBuf, Vec<String>), IgnoreFilter>;

/// 全局过滤器缓存：(root, patterns) -> [`IgnoreFilter`]。
///
/// 使用 `LazyLock` 延迟初始化，`Mutex` 保证线程安全。
static FILTER_CACHE: LazyLock<Mutex<FilterCache>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 从缓存获取或创建过滤器。
///
/// 若缓存中已存在相同的 `(root, patterns)` 组合，直接返回克隆；
/// 否则编译新过滤器并插入缓存。
///
/// # Errors
///
/// 返回 [`VaultError::Init`] 当模式解析失败时。
pub fn get_or_create(root: PathBuf, patterns: Vec<String>) -> Result<IgnoreFilter, VaultError> {
    let mut cache = FILTER_CACHE
        .lock()
        .map_err(|e| VaultError::Init(format!("filter cache lock poisoned: {e}")))?;

    if let Some(filter) = cache.get(&(root.clone(), patterns.clone())) {
        return Ok(filter.clone());
    }

    let filter = IgnoreFilter::new(&root, &patterns)?;
    cache.insert((root, patterns), filter.clone());
    Ok(filter)
}

/// 清除过滤器缓存。
///
/// 主要用于测试或内存紧张时。
pub fn clear_cache() {
    if let Ok(mut cache) = FILTER_CACHE.lock() {
        cache.clear();
    }
}

/// 返回当前缓存中的条目数。
#[cfg(test)]
pub fn cache_len() -> usize {
    FILTER_CACHE.lock().map_or(0, |cache| cache.len())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::Path;

    /// 测试专用锁，确保缓存测试串行执行。
    static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn cache_returns_same_filter_for_same_input() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_cache();
        let root = PathBuf::from("/tmp/test-vault");
        let patterns = vec!["*.tmp".to_string(), ".git/".to_string()];

        let filter1 = get_or_create(root.clone(), patterns.clone()).unwrap();
        let filter2 = get_or_create(root, patterns).unwrap();

        // 验证缓存命中
        assert_eq!(cache_len(), 1);

        // 验证过滤器功能一致
        assert!(filter1.is_ignored(Path::new("/tmp/test-vault/test.tmp")));
        assert!(filter2.is_ignored(Path::new("/tmp/test-vault/test.tmp")));
    }

    #[test]
    fn cache_differentiates_different_patterns() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_cache();
        let root = PathBuf::from("/tmp/test-vault");

        let _ = get_or_create(root.clone(), vec!["*.tmp".to_string()]).unwrap();
        let _ = get_or_create(root.clone(), vec!["*.log".to_string()]).unwrap();

        assert_eq!(cache_len(), 2);
    }
}
