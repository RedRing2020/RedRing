//! Foundation System Compatibility Bridge
//!
//! 新実装アーキテクチャへの互換性ブリッジ
//! すべてのトレイトは適切な新しい場所に移行済み

// 高次操作トレイト - traits/ に移行済み
pub use crate::traits::{BasicCollision, BasicIntersection, BasicTransform};

// Core Foundation - geometry/ に移行済み
pub use crate::geometry::core_foundation::*;

// 形状特化トレイト - abstracts/ に移行済み
pub use crate::abstracts::{
    arc_core::*, arc_extensions::*, circle_core::*, ellipse_arc_core::*, point_extensions::*,
};
