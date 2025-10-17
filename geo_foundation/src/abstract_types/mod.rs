//! 移行互換モジュール - 新実装アーキテクチャへの互換性ブリッジ
//!
//! 新しい構造への完全移行完了:
//! - `traits/` - Foundation操作トレイト群
//! - `abstracts/` - 最小責務抽象化  
//! - `geometry/` - 基本幾何Foundation

// 下位互換性のための再エクスポート
pub mod abstracts {
    pub use crate::abstracts::*;
}

// Virtual foundation module for compatibility
pub mod foundation {
    // 高次操作トレイト - traits/ から再エクスポート
    pub use crate::traits::{
        AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
        MultipleIntersection, PointDistance, SelfIntersection, TransformHelpers,
    };

    // Core Foundation - src直下から再エクスポート
    pub use crate::core_foundation::*;

    // 形状特化トレイト - abstracts/ から再エクスポート
    pub use crate::abstracts::{
        arc_core::*, arc_extensions::*, circle_core::*, ellipse_arc_core::*, point_extensions::*,
    };
}

// Re-exports for backward compatibility
pub use crate::abstracts::arc_traits::Arc2D;
pub use foundation::{
    AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
    MultipleIntersection, PointDistance, SelfIntersection, TransformHelpers,
};
