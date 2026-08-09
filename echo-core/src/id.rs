//! ID 类型定义。
//!
//! 提供统一的 ID newtype 模式，防止不同类型 ID 混用。
//! 所有 ID 基于 UUID v4 生成，保证全局唯一性。
//!
//! # 设计意图
//!
//! 使用 newtype 模式而非 type alias，在编译期即可捕获 ID 类型误用：
//!
//! ```compile_fail
//! use echo_core::id::{BlockId, FileId};
//! let block = BlockId::new();
//! let file = FileId::new();
//! // 下面这行无法编译：类型不匹配
//! // let _: FileId = block;
//! ```

use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// ID 公共 trait。
///
/// 所有 ID 类型（`NodeId`、`BlockId`、`FileId`、`VaultId`）均实现此 trait，
/// 提供统一的 ID 行为：生成、字符串转换、相等比较。
///
/// # 实现要求
///
/// 实现此 trait 的类型必须同时满足：
/// - `Copy` + `Eq` + `Hash`：可作为 HashMap/HashSet 的键
/// - `Debug` + `Display`：可打印和格式化
/// - `Serialize` + `Deserialize`：可序列化（用于 TOML/JSON 持久化）
/// - `Send` + `Sync`：可在线程间安全传递
pub trait Id:
    Copy
    + Eq
    + Hash
    + fmt::Debug
    + fmt::Display
    + Serialize
    + for<'de> Deserialize<'de>
    + Send
    + Sync
    + 'static
{
    /// 从原始 UUID 构造 ID。
    ///
    /// 这是唯一的构造入口，确保所有 ID 实例都拥有合法的 UUID。
    fn from_uuid(uuid: Uuid) -> Self;

    /// 获取原始 UUID。
    fn as_uuid(&self) -> Uuid;

    /// 生成新的唯一 ID（基于 UUID v4）。
    #[must_use]
    fn new() -> Self {
        Self::from_uuid(Uuid::new_v4())
    }

    /// 获取 ID 的字符串表示。
    fn as_str(&self) -> String {
        self.as_uuid().to_string()
    }

    /// 从字符串解析 ID。
    ///
    /// # Errors
    ///
    /// 当字符串不是合法的 UUID 格式时返回 `None`。
    #[must_use]
    fn parse(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self::from_uuid)
    }
}

/// 为所有实现了底层操作的类型自动提供完整的 `Id` trait 实现。
///
/// 新 ID 类型只需实现 `from_uuid` 和 `as_uuid`，其余方法由此宏自动生成。
macro_rules! impl_id {
    ($type:ident) => {
        impl Id for $type {
            fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
            fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $type {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }

        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl Eq for $type {}

        impl Hash for $type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl fmt::Debug for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($type), self.0)
            }
        }

        impl Serialize for $type {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.to_string().serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                Uuid::parse_str(&s)
                    .map(Self)
                    .map_err(serde::de::Error::custom)
            }
        }
    };
}

/// 节点 ID — 用于标识 AST 中的任意节点。
///
/// 适用于需要唯一标识树中任意位置的场景，如：
/// - 编辑器光标位置锚定
/// - 增量更新的变更追踪
/// - 跨文档引用节点
#[derive(Clone, Copy)]
pub struct NodeId(Uuid);
impl_id!(NodeId);

/// 块 ID — 用于标识文档中的块级元素。
///
/// 对应 Obsidian 的块引用语法 `^blockid`，用于：
/// - 块级引用 `[[page#^blockid]]`
/// - 块的持久化标识（跨会话不变）
/// - 编辑器中块的增量编辑锚点
#[derive(Clone, Copy)]
pub struct BlockId(Uuid);
impl_id!(BlockId);

/// 文件 ID — 用于标识文件系统中的文件。
///
/// 适用于需要持久化标识文件的场景，如：
/// - 文件监控中的唯一标识（路径可能变化）
/// - 最近打开文件列表
/// - 文件与块的关联
#[derive(Clone, Copy)]
pub struct FileId(Uuid);
impl_id!(FileId);

/// 仓库 ID — 用于标识 Vault（笔记仓库）。
///
/// 适用于：
/// - 多仓库管理中的唯一标识
/// - 仓库级别的配置关联
/// - 跨仓库引用
#[derive(Clone, Copy)]
pub struct VaultId(Uuid);
impl_id!(VaultId);

/// 时间戳类型 — 统一的时刻表示。
///
/// 使用 Unix 时间戳（秒精度），保证跨平台一致性。
/// 基于 `u64` 存储，支持序列化和比较操作。
pub type Timestamp = u64;

/// 获取当前时间戳（秒精度）。
#[must_use]
pub fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 创建零值时间戳（表示"未设置"）。
#[must_use]
pub const fn zero_timestamp() -> Timestamp {
    0
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn id_types_are_distinct() {
        // 不同类型 ID 的 UUID 互不重复
        let node = NodeId::new();
        let block = BlockId::new();
        let file = FileId::new();
        let vault = VaultId::new();

        let mut uuids = std::collections::HashSet::new();
        assert!(uuids.insert(node.as_uuid()));
        assert!(uuids.insert(block.as_uuid()));
        assert!(uuids.insert(file.as_uuid()));
        assert!(uuids.insert(vault.as_uuid()));
        assert_eq!(uuids.len(), 4);
    }

    #[test]
    fn id_display_formats_uuid() {
        let id = BlockId::new();
        let s = id.to_string();
        assert!(Uuid::parse_str(&s).is_ok());
    }

    #[test]
    fn id_parse_roundtrip() {
        let id = FileId::new();
        let s = id.to_string();
        let parsed = FileId::parse(&s).expect("parse");
        assert_eq!(id, parsed);
    }

    #[test]
    fn id_from_str() {
        let id = VaultId::new();
        let s = id.to_string();
        let parsed: VaultId = s.parse().expect("from_str");
        assert_eq!(id, parsed);
    }

    #[test]
    fn id_debug_includes_type_name() {
        let id = NodeId::new();
        let debug = format!("{id:?}");
        assert!(debug.starts_with("NodeId("));
        assert!(debug.ends_with(')'));
    }

    use serde::{Deserialize, Serialize};

    #[test]
    fn id_serialization_roundtrip() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper {
            id: BlockId,
        }
        let wrapper = Wrapper { id: BlockId::new() };
        let toml_str = toml::to_string(&wrapper).expect("serialize");
        let parsed: Wrapper = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(wrapper.id, parsed.id);
    }

    #[test]
    fn id_parse_invalid_returns_none() {
        assert!(NodeId::parse("not-a-uuid").is_none());
        assert!(BlockId::parse("").is_none());
    }

    #[test]
    fn timestamp_now_is_positive() {
        assert!(now() > 0);
    }

    #[test]
    fn timestamp_zero() {
        assert_eq!(zero_timestamp(), 0);
    }

    #[test]
    fn id_can_be_hashmap_key() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let id = NodeId::new();
        map.insert(id, "value");
        assert_eq!(map.get(&id), Some(&"value"));
    }

    #[test]
    fn id_is_copy() {
        let id = BlockId::new();
        let id2 = id;
        // 如果 BlockId 不是 Copy，这里会编译失败
        assert_eq!(id, id2);
    }
}
