//! 移行互換モジュール - 廃止予定
//!
//! 新しい構造:
//! - `traits` - Foundation操作トレイト群  
//! - `abstracts` - 最小責務抽象化
//! - `geometry` - 基本幾何Foundation

// 下位互換性のための移行aliasモジュール
pub mod foundation {
    pub use crate::geometry::*;
    pub use crate::traits::*;
}

pub mod abstracts {
    pub use crate::abstracts::*;
}

pub mod geometry {
    pub use crate::geometry::*;
}

// Re-exports for backward compatibility
pub use abstracts::Arc2D;
pub use foundation::{
    AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
    MultipleIntersection, PointDistance, SelfIntersection, TransformHelpers,
};
